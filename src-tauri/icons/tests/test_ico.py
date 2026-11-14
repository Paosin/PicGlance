import unittest
from pathlib import Path

from PIL import Image

ICON = Path(__file__).resolve().parents[1] / "icon.ico"
SIZES = (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)


class AppIconTests(unittest.TestCase):
    def test_ico_has_decodable_transparent_frames_at_shell_sizes(self):
        self.assertTrue(ICON.is_file(), "missing PicGlance ICO")
        with Image.open(ICON) as icon:
            self.assertEqual(icon.format, "ICO")
            self.assertEqual(icon.ico.sizes(), {(size, size) for size in SIZES})
            for size in SIZES:
                with self.subTest(size=size):
                    frame = icon.ico.getimage((size, size)).convert("RGBA")
                    self.assertEqual(frame.size, (size, size))
                    self.assertLessEqual(frame.getpixel((0, 0))[3], 4)
                    self.assertEqual(frame.getpixel((size // 2, size // 2))[3], 255)
                    colors = frame.getcolors(size * size)
                    self.assertGreater(sum(n for n, (r, g, b, a) in colors if r > 225 and g > 225 and b > 225 and a == 255), size)


if __name__ == "__main__":
    unittest.main()
