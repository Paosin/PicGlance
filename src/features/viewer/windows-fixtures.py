"""Create local test images for windows-smoke.py in the given directory."""
from pathlib import Path
import sys

from PIL import Image

root = Path(sys.argv[1])
root.mkdir(parents=True, exist_ok=True)
red = Image.new("RGB", (240, 120), (245, 28, 23))
blue = Image.new("RGB", red.size, (23, 37, 245))
exif = red.getexif()
exif[271] = "PicGlance"
exif[272] = "Viewer smoke test"
exif[34853] = {1: "N", 2: (31, 13, 0), 3: "E", 4: (121, 28, 0)}
red.save(root / "01 中文 空格.jpg", exif=exif)
blue.save(root / "02.png")
asymmetric = red.copy()
asymmetric.paste(blue.crop((0, 0, 120, 120)), (120, 0))
asymmetric.save(root / "04-clipboard.png")
red.save(root / "03-corrupt.jpg")
path = root / "03-corrupt.jpg"
path.write_bytes(path.read_bytes()[:100])
for extension in ("gif", "png", "webp"):
    red.save(root / f"motion-{extension}.{extension}", save_all=True, append_images=[blue],
             duration=[140, 260], loop=2, lossless=True)
with (root / "too-large.png").open("wb") as file:
    file.truncate(80 * 1024 * 1024 + 1)
Image.new("RGB", (7200, 4800), (245, 28, 23)).save(root / "large.jpg", quality=91)
if "--g2" in sys.argv[2:]:
    Image.new("RGB", (20_000, 10_000), (245, 28, 23)).save(root / "g2-color.jpg", quality=85)
long_path = root
while len(str(long_path / "长 路径.png")) < 280:
    long_path /= "长文件名 abcdefghijklmnopqrstuvwxyz"
long_path.mkdir(parents=True, exist_ok=True)
blue.save(long_path / "长 路径.png")
print(long_path / "长 路径.png")

