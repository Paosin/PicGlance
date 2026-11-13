"""Exercise the Windows Release viewer in a visible desktop session with Pillow and psutil."""
import argparse
import ctypes
import shutil
from ctypes import wintypes
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import time
from contextlib import contextmanager

import psutil
from PIL import ImageGrab

user = ctypes.windll.user32
user.SetProcessDpiAwarenessContext(ctypes.c_void_p(-4))
user.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
user.GetClientRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
user.ClientToScreen.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.POINT)]
user.SetForegroundWindow.argtypes = [wintypes.HWND]
user.IsWindowVisible.argtypes = [wintypes.HWND]
user.GetWindowRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
user.GetDpiForWindow.argtypes = [wintypes.HWND]
user.GetClassNameW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]
user.GetClipboardSequenceNumber.restype = wintypes.DWORD


def until(check, description, seconds=12):
    deadline = time.perf_counter() + seconds
    while time.perf_counter() < deadline:
        value = check()
        if value:
            return value
        time.sleep(0.015)
    raise AssertionError(f"Timed out waiting for {description}")


def window_for(pid):
    matches = []

    @ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
    def collect(window, _):
        owner = wintypes.DWORD()
        user.GetWindowThreadProcessId(window, ctypes.byref(owner))
        if owner.value == pid and user.IsWindowVisible(window):
            box = wintypes.RECT()
            user.GetClientRect(window, ctypes.byref(box))
            if box.right >= 400 and box.bottom >= 300:
                matches.append(window)
        return True

    user.EnumWindows(collect, 0)
    return next(iter(matches), None)


def color_at_center(window, dx=0, dy=0):
    rect = wintypes.RECT()
    user.GetClientRect(window, ctypes.byref(rect))
    center = wintypes.POINT(rect.right // 2 + dx, rect.bottom // 2 + dy)
    user.ClientToScreen(window, ctypes.byref(center))
    red, _, blue = ImageGrab.grab(bbox=(center.x, center.y, center.x + 1, center.y + 1)).getpixel((0, 0))
    if red > 180 and blue < 100:
        return "red"
    if blue > 180 and red < 100:
        return "blue"
    return "other"


@contextmanager
def launch(exe, path, sample=None):
    child = subprocess.Popen([str(exe), str(path)], cwd=exe.parent)
    started = time.perf_counter()
    process = psutil.Process(child.pid)
    stop = threading.Event()
    def measure():
        while not stop.is_set():
            try:
                processes = [process, *process.children(recursive=True)]
                totals = [0, 0, 0, 0]
                for item in processes:
                    try:
                        memory = item.memory_info()
                        totals[0] += memory.rss
                        totals[1] += memory.private
                        if item == process:
                            totals[2] = memory.rss
                            totals[3] = memory.private
                    except (psutil.NoSuchProcess, psutil.AccessDenied):
                        pass
                sample.append(totals)
            except psutil.NoSuchProcess:
                return
            stop.wait(0.01)
    monitor = threading.Thread(target=measure) if sample is not None else None
    if monitor:
        monitor.start()
    try:
        def find_window():
            if child.poll() is not None:
                raise RuntimeError(f"PicGlance exited with {child.returncode}; close other PicGlance instances before the smoke test")
            return window_for(child.pid)
        window = until(find_window, f"{path.name} window")
        user.SetForegroundWindow(window)
        if user.GetForegroundWindow() != window:
            user.SwitchToThisWindow(ctypes.c_void_p(window), True)
        # Keep test windows above parallel workspace smoke runs.
        user.SetWindowPos(ctypes.c_void_p(window), ctypes.c_void_p(-1), 0, 0, 0, 0, 0x0003)
        yield window, started
    finally:
        stop.set()
        if monitor:
            monitor.join()
        descendants = process.children(recursive=True) if process.is_running() else []
        for item in [process, *descendants]:
            try:
                item.terminate()
            except psutil.NoSuchProcess:
                pass
        child.wait(timeout=5)
        psutil.wait_procs(descendants, timeout=3)


def static_image(exe, path, color, check_loading=False, measure_peak=False):
    samples = [] if measure_peak else None
    with launch(exe, path, samples) as (window, started):
        visible = round((time.perf_counter() - started) * 1000)
        if check_loading:
            assert color_at_center(window) == "other", "The large image appeared before the loading window was observed"
        until(lambda: color_at_center(window) == color, f"{path.name} pixels")
        first = round((time.perf_counter() - started) * 1000)
        print(f"{path.name}: visible window {visible} ms, image {first} ms", flush=True)
        if measure_peak:
            time.sleep(0.75)
    if samples:
        peak = [round(max(values) / (1024 * 1024), 1) for values in zip(*samples)]
        print(f"{path.name}: {path.stat().st_size} bytes, {len(samples)} samples, peak tree RSS/private {peak[0]}/{peak[1]} MiB, main RSS/private {peak[2]}/{peak[3]} MiB", flush=True)


def animation(exe, path):
    with launch(exe, path) as (window, _):
        until(lambda: color_at_center(window) in {"red", "blue"}, f"{path.name} first frame")
        deadline = time.perf_counter() + 1.85
        transitions = []
        while time.perf_counter() < deadline:
            color = color_at_center(window)
            if color != "other" and (not transitions or transitions[-1][0] != color):
                transitions.append((color, time.perf_counter()))
            time.sleep(0.01)
        colors = [color for color, _ in transitions]
        # A screen capture can miss the first red frame during the opening fade.
        assert colors.count("red") >= 1 and colors.count("blue") >= 2, (path.name, colors)
        assert colors[-1] == "blue", (path.name, colors)
        assert len(colors) <= 6, (path.name, colors)
        assert time.perf_counter() - transitions[-1][1] > 0.35, (path.name, colors)
        spans = [round((second[1] - first[1]) * 1000) for first, second in zip(transitions, transitions[1:])]
        assert any(85 <= span <= 220 for span in spans), (path.name, spans)
        if colors[0] == "red":
            assert any(200 <= span <= 340 for span in spans), (path.name, spans)
        print(f"{path.name}: {colors}, visible frame spans {spans} ms", flush=True)


def press(*keys):
    for key in keys:
        user.keybd_event(key, 0, 0, 0)
    for key in reversed(keys):
        user.keybd_event(key, 0, 2, 0)


def window_size(window):
    rect = wintypes.RECT()
    user.GetWindowRect(window, ctypes.byref(rect))
    return rect.right - rect.left, rect.bottom - rect.top


def rename_file(exe, folder):
    from PIL import Image
    source = folder / "rename-source.png"
    Image.new("RGB", (640, 480), (23, 37, 245)).save(source)
    original = source.read_bytes()
    occupied = folder / "24173.png"
    occupied.write_bytes(b"keep existing bytes")
    with launch(exe, source) as (window, _):
        until(lambda: color_at_center(window) == "blue", "rename source")
        box = wintypes.RECT()
        user.GetClientRect(window, ctypes.byref(box))
        dpi = user.GetDpiForWindow(window) / 96
        details = wintypes.POINT(int(353 * dpi), box.bottom - int(22 * dpi))
        field = wintypes.POINT(box.right - int(170 * dpi), int(208 * dpi))
        button = wintypes.POINT(box.right - int(42 * dpi), int(208 * dpi))
        user.ClientToScreen(window, ctypes.byref(details))
        user.ClientToScreen(window, ctypes.byref(field))
        user.ClientToScreen(window, ctypes.byref(button))
        cursor = wintypes.POINT()
        user.GetCursorPos(ctypes.byref(cursor))
        def click(point):
            user.SetCursorPos(point.x, point.y)
            user.mouse_event(2, 0, 0, 0, 0)
            user.mouse_event(4, 0, 0, 0, 0)
        def submit(name):
            click(field)
            time.sleep(0.12)
            press(0x11, 0x41)
            for character in name:
                press(ord(character))
                time.sleep(0.035)
            click(button)
        try:
            click(details)
            time.sleep(0.3)
            submit("24173")
            time.sleep(0.4)
            assert source.read_bytes() == original and occupied.read_bytes() == b"keep existing bytes"
            submit("58136")
            destination = folder / "58136.png"
            until(destination.exists, "renamed file with preserved extension", seconds=4)
        except (AssertionError, FileNotFoundError):
            rect = wintypes.RECT()
            user.GetWindowRect(window, ctypes.byref(rect))
            capture = Path("target/viewer-rename-failure.png")
            capture.parent.mkdir(exist_ok=True)
            ImageGrab.grab(bbox=(rect.left, rect.top, rect.right, rect.bottom)).save(capture)
            print("Rename test directory:", sorted(item.name for item in folder.iterdir()), flush=True)
            raise
        finally:
            user.SetCursorPos(cursor.x, cursor.y)
        assert destination.read_bytes() == original and not source.exists()
        assert color_at_center(window) == "blue"
    print("Windows EXIF panel rename, existing name protection and extension: passed", flush=True)


def dialog_for(window):
    owner = wintypes.DWORD()
    user.GetWindowThreadProcessId(window, ctypes.byref(owner))
    dialogs = []

    @ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
    def collect(candidate, _):
        pid = wintypes.DWORD()
        user.GetWindowThreadProcessId(candidate, ctypes.byref(pid))
        class_name = ctypes.create_unicode_buffer(64)
        user.GetClassNameW(candidate, class_name, len(class_name))
        if pid.value == owner.value and class_name.value == "#32770" and user.IsWindowVisible(candidate):
            dialogs.append(candidate)
        return True

    user.EnumWindows(collect, 0)
    return next(iter(dialogs), None)


def navigation(exe, folder):
    with launch(exe, folder / "01 中文 空格.jpg") as (window, _):
        until(lambda: color_at_center(window) == "red", "initial JPEG")
        time.sleep(0.3)
        press(0x49)  # I
        def warning_visible():
            box = wintypes.RECT()
            user.GetClientRect(window, ctypes.byref(box))
            dpi = user.GetDpiForWindow(window) / 96
            point = wintypes.POINT(box.right - int(40 * dpi), int(160 * dpi))
            user.ClientToScreen(window, ctypes.byref(point))
            image = ImageGrab.grab(bbox=(point.x - 10, point.y - 32, point.x + 5, point.y + 24))
            return any(red > 220 and green > 150 and 60 < blue < 160 for red, green, blue in image.get_flattened_data())
        until(warning_visible, "GPS warning color")
        press(0x49)
        press(0x54)  # T
        def selected_thumbnail_visible():
            box = wintypes.RECT()
            user.GetClientRect(window, ctypes.byref(box))
            dpi = user.GetDpiForWindow(window) / 96
            point = wintypes.POINT(int(32 * dpi), box.bottom - int(80 * dpi))
            user.ClientToScreen(window, ctypes.byref(point))
            pixels = ImageGrab.grab(bbox=(point.x - 7, point.y - 7, point.x + 8, point.y + 8))
            return any(red > 180 and blue < 100 for red, _, blue in pixels.get_flattened_data())
        until(selected_thumbnail_visible, "selected JPEG thumbnail")
        press(0x27)  # Right
        until(lambda: color_at_center(window) == "blue", "next PNG")
        (folder / "03-corrupt.jpg").rename(folder / "removed.txt")
        press(0x27)
        until(lambda: color_at_center(window) == "other", "removed entry placeholder")
        time.sleep(0.2)
        press(0x27)
        until(lambda: color_at_center(window, -50) == "red" and color_at_center(window, 50) == "blue", "next surviving image")
        original = window_size(window)
        press(0x7A)  # F11
        until(lambda: window_size(window) != original, "full screen")
        press(0x1B)  # Escape
        until(lambda: window_size(window) == original, "windowed mode")
        print("Windows GPS warning, thumbnail, directory navigation, removed entry, F11 and Escape: passed", flush=True)


def clipboard(exe, folder):
    with launch(exe, folder / "04-clipboard.png") as (window, _):
        original = lambda: color_at_center(window, -50) == "red" and color_at_center(window, 50) == "blue"
        until(original, "original colors")
        press(0x48)  # H
        until(lambda: color_at_center(window, -50) == "blue" and color_at_center(window, 50) == "red", "horizontal flip")
        sequence = user.GetClipboardSequenceNumber()
        press(0x11, 0x43)  # Ctrl+C
        until(lambda: user.GetClipboardSequenceNumber() != sequence, "image copied")
        press(0x11, 0x56)  # Ctrl+V
        until(original, "original colors after paste")
        press(0x11, 0x53)  # Ctrl+S
        time.sleep(0.3)
        assert not dialog_for(window), "Ctrl+S opened a dialog for a clipboard image"
        press(0x11, 0x10, 0x53)  # Ctrl+Shift+S
        until(lambda: dialog_for(window), "clipboard Save As dialog")
        press(0x1B)
        until(lambda: not dialog_for(window), "Save As cancel")
        assert original()
        print("Windows clipboard original pixels, Ctrl+S and Ctrl+Shift+S: passed", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("exe", type=Path)
    parser.add_argument("--clipboard", action="store_true", help="Also replace the system clipboard with a synthetic test image")
    parser.add_argument("--g2", action="store_true", help="Measure Windows process-tree peaks for 7200×4800 and 20000×10000 JPEGs")
    parser.add_argument("--rename", action="store_true", help="Run only the file-rename interaction")
    args = parser.parse_args()
    exe = args.exe.resolve()
    with tempfile.TemporaryDirectory(prefix="picglance-viewer-") as temporary:
        folder = Path(temporary)
        portable = folder / "app"
        portable.mkdir()
        shutil.copy2(exe, portable / exe.name)
        (portable / "portable.marker").touch()
        exe = portable / exe.name
        fixtures = Path(__file__).with_name("windows-fixtures.py")
        flags = ["--g2"] if args.g2 else []
        long_path = Path(subprocess.check_output([sys.executable, "-X", "utf8", str(fixtures), str(folder), *flags], encoding="utf-8").strip())
        if args.rename:
            rename_file(exe, folder)
            return
        if args.g2:
            static_image(exe, folder / "large.jpg", "red", check_loading=True, measure_peak=True)
            static_image(exe, folder / "g2-color.jpg", "red", check_loading=True, measure_peak=True)
            return
        static_image(exe, folder / "01 中文 空格.jpg", "red")
        static_image(exe, long_path, "blue")
        static_image(exe, folder / "large.jpg", "red", check_loading=True)
        for extension in ("gif", "png", "webp"):
            animation(exe, folder / f"motion-{extension}.{extension}")
        navigation(exe, folder)
        if args.clipboard:
            clipboard(exe, folder)
        rename_file(exe, folder)


if __name__ == "__main__":
    main()

