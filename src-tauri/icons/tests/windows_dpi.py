"""Report live monitor count and DPI for the visible window of a PicGlance process."""
import argparse
import ctypes as ct
import json
from ctypes import wintypes as wt

USER32 = ct.WinDLL("user32", use_last_error=True)
USER32.SetProcessDpiAwarenessContext.argtypes = [wt.HANDLE]
USER32.GetDpiForWindow.argtypes = [wt.HWND]
USER32.GetDpiForWindow.restype = wt.UINT
USER32.IsWindowVisible.argtypes = [wt.HWND]
USER32.GetWindowThreadProcessId.argtypes = [wt.HWND, ct.POINTER(wt.DWORD)]
USER32.GetWindowTextLengthW.argtypes = [wt.HWND]
USER32.GetWindowTextW.argtypes = [wt.HWND, wt.LPWSTR, ct.c_int]
ENUM_WINDOW = ct.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM)
USER32.EnumWindows.argtypes = [ENUM_WINDOW, wt.LPARAM]


def measure(pid):
    if not USER32.SetProcessDpiAwarenessContext(ct.c_void_p(-4)):
        raise ct.WinError(ct.get_last_error())
    windows = []

    @ENUM_WINDOW
    def collect(hwnd, _):
        owner = wt.DWORD()
        USER32.GetWindowThreadProcessId(hwnd, ct.byref(owner))
        if owner.value == pid and USER32.IsWindowVisible(hwnd):
            title = ct.create_unicode_buffer(USER32.GetWindowTextLengthW(hwnd) + 1)
            USER32.GetWindowTextW(hwnd, title, len(title))
            if title.value == "PicGlance":
                windows.append({"title": title.value, "hwnd": hwnd, "dpi": USER32.GetDpiForWindow(hwnd)})
        return True

    USER32.EnumWindows(collect, 0)
    return {"system_dpi": USER32.GetDpiForSystem(), "monitor_count": USER32.GetSystemMetrics(80), "windows": windows}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pid", type=int)
    args = parser.parse_args()
    result = measure(args.pid)
    print(json.dumps(result, ensure_ascii=False))
    if args.pid is not None and not result["windows"]:
        raise SystemExit("The process has no visible PicGlance window")
