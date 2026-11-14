"""Measure input-to-pixel latency and visible animation frames on a Windows desktop."""
import argparse
import ctypes
from ctypes import wintypes
from hashlib import sha256
import json
import math
from pathlib import Path
from random import Random
import shutil
import tempfile
import time

from PIL import Image

from importlib import import_module

smoke = import_module("windows-smoke")
user = smoke.user
gdi = ctypes.windll.gdi32
user.GetDC.argtypes = [wintypes.HWND]
user.GetDC.restype = wintypes.HDC
user.ReleaseDC.argtypes = [wintypes.HWND, wintypes.HDC]
gdi.GetPixel.argtypes = [wintypes.HDC, ctypes.c_int, ctypes.c_int]
gdi.GetPixel.restype = wintypes.DWORD
PALETTE = [
    (238, 30, 30), (30, 238, 30), (30, 30, 238), (238, 238, 30),
    (238, 30, 238), (30, 238, 238), (238, 127, 30), (127, 30, 238),
    (30, 127, 238), (127, 238, 30), (238, 30, 127), (30, 238, 127),
]


def sampler(window):
    box = wintypes.RECT()
    user.GetClientRect(window, ctypes.byref(box))
    center = wintypes.POINT(box.right // 2, box.bottom // 2)
    user.ClientToScreen(window, ctypes.byref(center))
    dc = user.GetDC(None)

    def pixel():
        raw = gdi.GetPixel(dc, center.x, center.y)
        return (raw & 255, (raw >> 8) & 255, (raw >> 16) & 255)

    return pixel, lambda: user.ReleaseDC(None, dc)


def matching_frame(rgb):
    for index, color in enumerate(PALETTE):
        if max(abs(a - b) for a, b in zip(rgb, color)) <= 18:
            return index
    return None


def first_visible(rgb, color):
    # Qualify the new picture early in its fade, well below fully opaque.
    if max(rgb) - min(rgb) < 35:
        return False
    return all((pixel > 75 if expected > 180 else pixel < 95) for pixel, expected in zip(rgb, color))


def static_fixtures(folder, width, height, fmt):
    folder.mkdir()
    files = []
    for index, color in enumerate(PALETTE[:6]):
        data = Random(index + 137).randbytes(width * height * 3)
        image = Image.frombytes("RGB", (width, height), data)
        image.paste(color, (width // 3, height // 3, 2 * width // 3, 2 * height // 3))
        file = folder / f"{index:02}.{fmt}"
        image.save(file, quality=82)
        files.append(file)
    return files


def switching(exe, files, passes):
    timings = []
    with smoke.launch(exe, files[0]) as (window, _):
        pixel, release = sampler(window)
        try:
            smoke.until(lambda: matching_frame(pixel()) == 0, "first switch fixture")
            time.sleep(0.35)
            def focus_window():
                if user.GetForegroundWindow() == window:
                    return True
                point = wintypes.POINT()
                user.GetCursorPos(ctypes.byref(point))
                anchor = wintypes.POINT(40, 110)
                user.ClientToScreen(window, ctypes.byref(anchor))
                user.SetCursorPos(anchor.x, anchor.y)
                user.mouse_event(2, 0, 0, 0, 0)
                user.mouse_event(4, 0, 0, 0, 0)
                user.SetCursorPos(point.x, point.y)
                return False
            smoke.until(focus_window, "keyboard foreground")
            for index in range(1, len(files) * passes + 1):
                target = PALETTE[index % len(files)]
                smoke.until(focus_window, "keyboard foreground")
                start = time.perf_counter()
                smoke.press(0x27)
                measured = None
                last_rgb = None
                deadline = start + 2
                while time.perf_counter() < deadline:
                    last_rgb = pixel()
                    if first_visible(last_rgb, target):
                        measured = round((time.perf_counter() - start) * 1000, 1)
                        break
                timings.append(measured)
                if measured is None:
                    print(f"Switch {index} timed out; foreground={user.GetForegroundWindow() == window}, last RGB={last_rgb}, expected={target}", flush=True)
                    break
                # Keep a 200 ms cadence where possible, and allow 110 ms for the previous fade.
                time.sleep(max(0.11, 0.2 - (time.perf_counter() - start)))
        finally:
            release()
    values = sorted(item for item in timings if item is not None)
    return {
        "samples_ms": timings, "file_bytes": [file.stat().st_size for file in files],
        "p50_ms": values[(len(values) - 1) // 2] if values else None,
        "p95_ms": values[math.ceil(len(values) * 0.95) - 1] if values else None,
        "max_ms": max(values, default=None),
        "above_100ms": sum(item is None or item > 100 for item in timings),
    }


def animation(exe, folder, fmt, duration, width, height):
    frames = [Image.new("RGB", (width, height), color) for color in PALETTE]
    path = folder / f"animation-{width}-{duration}.{fmt}"
    frames[0].save(path, save_all=True, append_images=frames[1:], duration=duration, loop=0, lossless=True)
    with Image.open(path) as check:
        assert check.n_frames == len(frames)
        delays = []
        for frame in range(check.n_frames):
            check.seek(frame)
            check.load()
            delays.append(check.info["duration"])
    with smoke.launch(exe, path) as (window, _):
        pixel, release = sampler(window)
        try:
            smoke.until(lambda: matching_frame(pixel()) is not None, f"{fmt} animated pixels")
            time.sleep(0.2)
            start = time.perf_counter()
            samples = []
            transitions = []
            previous = None
            while time.perf_counter() - start < 2.5:
                current = matching_frame(pixel())
                timestamp = time.perf_counter()
                samples.append(timestamp)
                if current is not None and current != previous:
                    transitions.append((current, timestamp))
                    previous = current
        finally:
            release()
    ids = [frame for frame, _ in transitions]
    intervals = [round((b[1] - a[1]) * 1000, 1) for a, b in zip(transitions, transitions[1:])]
    skipped = [(a, b) for a, b in zip(ids, ids[1:]) if (a + 1) % len(frames) != b]
    long_intervals = [value for value in intervals if value > duration * 2 + 8]
    return {
        "fps_encoded": round(1000 / (sum(delays) / len(delays)), 2),
        "observed_fps": round((len(transitions) - 1) / (transitions[-1][1] - transitions[0][1]), 2) if len(transitions) > 1 else None,
        "frames": ids, "intervals_ms": intervals, "skipped": skipped, "long_intervals_ms": long_intervals,
        "max_sample_gap_ms": round(max(b - a for a, b in zip(samples, samples[1:])) * 1000, 1),
        "bytes": path.stat().st_size,
    }


def size_boundaries(exe, folder):
    results = {}
    for fmt, options in [("png", {}), ("tiff", {"compression": "tiff_adobe_deflate"}), ("jpeg", {"progressive": True})]:
        path = folder / f"oversized-{fmt}.{fmt}"
        Image.new("L", (20_000, 10_000), 220).save(path, **options)
        log_dir = exe.parent / "log"
        with smoke.launch(exe, path):
            def rejected():
                for log in log_dir.glob("*"):
                    if log.is_file() and f"{path.name} reason=dimensions" in log.read_text(encoding="utf-8", errors="replace"):
                        return True
                return False
            smoke.until(rejected, f"{fmt} dimensions rejection")
        results[fmt] = {"bytes": path.stat().st_size, "result": "dimensions"}
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("exe", type=Path)
    parser.add_argument("--only", choices=["switch", "animation", "limits"])
    parser.add_argument("--json", type=Path)
    args = parser.parse_args()
    binary = args.exe.resolve()
    result = {"sha256": sha256(binary.read_bytes()).hexdigest(), "switching": {}, "animation": {}, "limits": {}}
    with tempfile.TemporaryDirectory(prefix="picglance-performance-") as temporary:
        folder = Path(temporary)
        portable = folder / "app"
        portable.mkdir()
        exe = portable / binary.name
        shutil.copy2(binary, exe)
        (portable / "portable.marker").touch()
        if args.only in (None, "switch"):
            for width, height, fmt, passes in [(800, 600, "jpg", 3), (4000, 3000, "jpg", 2), (1600, 1200, "png", 2)]:
                key = f"{width}x{height}-{fmt}"
                case = switching(exe, static_fixtures(folder / key, width, height, fmt), passes)
                result["switching"][key] = case
                print(key, case, flush=True)
        if args.only in (None, "animation"):
            for fmt, duration in [("gif", 40), ("png", 34), ("webp", 34)]:
                key = f"640x480-{fmt}"
                case = animation(exe, folder, fmt, duration, 640, 480)
                result["animation"][key] = case
                print(key, {name: value for name, value in case.items() if name not in ("frames", "intervals_ms")}, flush=True)
        if args.only == "limits":
            result["limits"] = size_boundaries(exe, folder)
            print(result["limits"], flush=True)
    if args.json:
        args.json.write_text(json.dumps(result, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()

