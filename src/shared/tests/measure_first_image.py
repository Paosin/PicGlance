"""Run on an interactive Windows desktop with Pillow and psutil installed."""

import argparse
import ctypes
import hashlib
import json
import subprocess
import tempfile
import time
from pathlib import Path

import psutil
from PIL import Image, ImageGrab


def process_sample(process):
    memory = process.memory_full_info()
    args = process.cmdline()[1:] if process.name().lower() == 'msedgewebview2.exe' else []
    role = next((arg.split('=', 1)[1] for arg in args if arg.startswith('--type=')), 'browser' if args else 'app')
    if role == 'utility':
        role = next((arg.split('=', 1)[1] for arg in args if arg.startswith('--utility-sub-type=')), role)
    return {'name': process.name(), 'role': role, 'working_set': memory.rss,
            'private_working_set': memory.uss, 'private_commit': memory.private}


def measure(exe, jpeg=None, marker=None, smoke=False, idle_seconds=5, marker_only=False,
            minimize=False):
    with subprocess.Popen([str(exe), *([] if jpeg is None else [str(jpeg)] if marker is None else [
        '--first-image-diagnostic', str(jpeg), str(marker)
    ])], cwd=exe.parent) as child:
        started = time.perf_counter()  # Popen has just returned from CreateProcess.
        process = psutil.Process(child.pid)
        seen = None
        marker_ms = None
        try:
            while time.perf_counter() - started < 15:
                if child.poll() is not None:
                    raise RuntimeError(f'{exe} exited before presentation with code {child.returncode}')
                if marker and marker.exists() and marker_ms is None:
                    marker_ms = round((time.perf_counter() - started) * 1000)
                windows = []

                def collect(hwnd, _):
                    pid = ctypes.c_ulong()
                    ctypes.windll.user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
                    if pid.value == child.pid and ctypes.windll.user32.IsWindowVisible(hwnd):
                        rect = (ctypes.c_long * 4)()
                        ctypes.windll.user32.GetClientRect(hwnd, ctypes.byref(rect))
                        origin = (ctypes.c_long * 2)()
                        ctypes.windll.user32.ClientToScreen(hwnd, ctypes.byref(origin))
                        if rect[2] >= 400 and rect[3] >= 300:
                            windows.append((hwnd, origin[0] + rect[2] // 2, origin[1] + rect[3] // 2))
                    return True

                callback = ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)(collect)
                ctypes.windll.user32.EnumWindows(callback, 0)
                if marker_only:
                    if marker_ms is not None and windows:
                        break
                    time.sleep(0.01)
                    continue
                for _, x, y in windows:
                    if smoke:
                        return {'visible_window_ms': round((time.perf_counter() - started) * 1000)}
                    if jpeg is None:
                        seen = round((time.perf_counter() - started) * 1000)
                        break
                    sample = ImageGrab.grab(bbox=(x - 8, y - 8, x + 8, y + 8))
                    # The synthetic JPEG has a saturated orange center, including after scaling.
                    orange = sum(r >= 225 and 65 <= g <= 155 and b < 85 for r, g, b in sample.get_flattened_data())
                    if orange >= 230 and seen is None:
                        seen = round((time.perf_counter() - started) * 1000)
                if seen is not None and (marker is None or marker_ms is not None):
                    break
                time.sleep(0.01)
            if not windows or (seen is None and not marker_only) or (marker is not None and marker_ms is None):
                raise RuntimeError(f'{exe} presentation timed out (visual={seen}, marker={marker_ms})')
            if minimize:
                hwnd = ctypes.c_void_p(windows[0][0])
                ctypes.windll.user32.ShowWindowAsync(hwnd, 6)  # SW_MINIMIZE
                deadline = time.perf_counter() + 3
                while not ctypes.windll.user32.IsIconic(hwnd):
                    if time.perf_counter() > deadline:
                        raise RuntimeError(f'{exe} did not minimize')
                    time.sleep(0.01)
            time.sleep(idle_seconds)
            snapshots = []
            for _ in range(3):
                descendants = []
                for item in process.children(recursive=True):
                    try:
                        descendants.append(process_sample(item))
                    except psutil.NoSuchProcess:
                        continue
                root = process_sample(process)
                snapshots.append({'main': root['working_set'], 'main_process': root, 'children': descendants,
                                  'total': root['working_set'] + sum(item['working_set'] for item in descendants),
                                  'private_working_set': root['private_working_set'] + sum(
                                      item['private_working_set'] for item in descendants),
                                  'private_commit': root['private_commit'] + sum(
                                      item['private_commit'] for item in descendants)})
                time.sleep(0.5)
            time_key = 'visible_window_ms' if jpeg is None else 'first_image_ms'
            return {time_key: seen, 'diagnostic_marker_ms': marker_ms, 'snapshots_bytes': snapshots}
        finally:
            descendants = process.children(recursive=True) if process.is_running() else []
            for item in [process, *descendants]:
                try:
                    item.terminate()
                except psutil.NoSuchProcess:
                    continue
            psutil.wait_procs(descendants, timeout=3)


def main():
    # Match EnumWindows client coordinates to the physical pixels read by ImageGrab.
    ctypes.windll.user32.SetProcessDpiAwarenessContext(ctypes.c_void_p(-4))
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--legacy', type=Path)
    parser.add_argument('--tauri', type=Path, required=True)
    parser.add_argument('--jpeg', type=Path)
    parser.add_argument('--runs', type=int, default=3)
    parser.add_argument('--smoke', action='store_true')
    parser.add_argument('--empty', action='store_true')
    parser.add_argument('--idle-seconds', type=float, default=5)
    parser.add_argument('--marker-only', action='store_true', help='Use the app paint marker if the desktop is occluded')
    parser.add_argument('--minimize', action='store_true', help='Minimize before memory sampling')
    args = parser.parse_args()
    if args.smoke:
        print(json.dumps(measure(args.tauri.resolve(), None, smoke=True)))
        return
    if args.runs < 1 or args.idle_seconds < 0:
        parser.error('--runs must be positive and --idle-seconds must be nonnegative')
    if args.empty:
        print(json.dumps([measure(args.tauri.resolve(), idle_seconds=args.idle_seconds, minimize=args.minimize)
                          for _ in range(args.runs)], indent=2))
        return
    if not args.jpeg:
        parser.error('--jpeg is required for first-image measurements')
    jpeg = args.jpeg.resolve()
    with Image.open(jpeg) as image:
        dimensions = image.size
    result = {'jpeg': {'dimensions': dimensions, 'bytes': jpeg.stat().st_size,
                       'sha256': hashlib.sha256(jpeg.read_bytes()).hexdigest()}, 'runs': []}
    with tempfile.TemporaryDirectory(dir=jpeg.parent) as folder:
        for index in range(args.runs):
            marker = Path(folder) / f'{index}.presented'
            run = {}
            if args.legacy:
                run['legacy'] = measure(args.legacy.resolve(), jpeg, idle_seconds=args.idle_seconds,
                                        minimize=args.minimize)
            run['tauri'] = measure(args.tauri.resolve(), jpeg, marker, idle_seconds=args.idle_seconds,
                                   marker_only=args.marker_only, minimize=args.minimize)
            result['runs'].append(run)
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
