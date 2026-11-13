"""Verify native Windows Shell icon extraction at common small and large sizes."""
import argparse
import ctypes as ct
from ctypes import wintypes as wt
from pathlib import Path

from PIL import Image

ICON = Path(__file__).resolve().parents[1] / "icon.ico"
USER32 = ct.WinDLL("user32", use_last_error=True)
GDI32 = ct.WinDLL("gdi32", use_last_error=True)


class IconInfo(ct.Structure):
    _fields_ = [("icon", wt.BOOL), ("x", wt.DWORD), ("y", wt.DWORD),
                ("mask", wt.HBITMAP), ("color", wt.HBITMAP)]


class BitmapInfoHeader(ct.Structure):
    _fields_ = [("size", wt.DWORD), ("width", wt.LONG), ("height", wt.LONG),
                ("planes", wt.WORD), ("bits_per_pixel", wt.WORD), ("compression", wt.DWORD),
                ("image_size", wt.DWORD), ("x", wt.LONG), ("y", wt.LONG),
                ("used_colors", wt.DWORD), ("important_colors", wt.DWORD)]


USER32.PrivateExtractIconsW.argtypes = [
    wt.LPCWSTR, ct.c_int, ct.c_int, ct.c_int, ct.POINTER(wt.HICON), ct.POINTER(wt.UINT), wt.UINT, wt.UINT,
]
USER32.PrivateExtractIconsW.restype = wt.UINT
USER32.GetIconInfo.argtypes = [wt.HICON, ct.POINTER(IconInfo)]
USER32.DestroyIcon.argtypes = [wt.HICON]
GDI32.DeleteObject.argtypes = [wt.HGDIOBJ]
GDI32.CreateCompatibleDC.argtypes = [wt.HDC]
GDI32.CreateCompatibleDC.restype = wt.HDC
GDI32.DeleteDC.argtypes = [wt.HDC]
GDI32.GetDIBits.argtypes = [
    wt.HDC, wt.HBITMAP, wt.UINT, wt.UINT, ct.c_void_p, ct.POINTER(BitmapInfoHeader), wt.UINT,
]


def extract(path, size):
    handle = wt.HICON()
    icon_info = IconInfo()
    dc = GDI32.CreateCompatibleDC(None)
    try:
        count = USER32.PrivateExtractIconsW(str(path), 0, size, size, ct.byref(handle), None, 1, 0)
        assert count == 1 and handle.value, (path, size, count)
        assert USER32.GetIconInfo(handle, ct.byref(icon_info)) and icon_info.color
        header = BitmapInfoHeader(size=ct.sizeof(BitmapInfoHeader))
        assert GDI32.GetDIBits(dc, icon_info.color, 0, 0, None, ct.byref(header), 0)
        assert (header.width, header.height) == (size, size), (path, size, header.width, header.height)
        header.height = -size
        header.bits_per_pixel = 32
        header.compression = 0
        pixels = (ct.c_ubyte * (4 * size * size))()
        assert GDI32.GetDIBits(dc, icon_info.color, 0, size, pixels, ct.byref(header), 0) == size
        return Image.frombytes("RGB", (size, size), bytes(pixels), "raw", "BGRX")
    finally:
        if icon_info.mask:
            GDI32.DeleteObject(icon_info.mask)
        if icon_info.color:
            GDI32.DeleteObject(icon_info.color)
        if handle.value:
            USER32.DestroyIcon(handle)
        if dc:
            GDI32.DeleteDC(dc)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", type=Path, help="Release executable built with the new ICO")
    args = parser.parse_args()
    for size in (16, 20, 24, 32, 40, 48, 64, 96, 128, 256):
        expected = extract(ICON, size)
        if args.exe:
            actual = extract(args.exe.resolve(), size)
            assert actual.tobytes() == expected.tobytes(), f"Embedded icon differs at {size}px"
    print("Windows decoded all 10 icon sizes" + (" from both ICO and Release EXE" if args.exe else " from the ICO"))
