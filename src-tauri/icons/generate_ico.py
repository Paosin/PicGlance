"""Render the canonical logo in Edge and write the Windows icon frames."""
import argparse
import base64
import io
import os
from pathlib import Path
import re
import subprocess
import tempfile

from PIL import Image

ROOT = Path(__file__).resolve().parent
SIZES = (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)
EDGE = Path(os.environ.get("ProgramFiles(x86)", "C:/Program Files (x86)")) / "Microsoft/Edge/Application/msedge.exe"


def render(edge):
    source = base64.b64encode((ROOT / "picglance.svg").read_bytes()).decode()
    with tempfile.TemporaryDirectory(prefix="picglance-logo-", ignore_cleanup_errors=True) as directory:
        profile = Path(directory) / "profile"
        page = Path(directory) / "render.html"
        page.write_text(f"""<!doctype html><body><script>
const image = new Image();
image.onload = () => {{
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = 1024;
  canvas.getContext('2d').drawImage(image, 0, 0, 1024, 1024);
  document.body.textContent = canvas.toDataURL();
}};
image.src = 'data:image/svg+xml;base64,{source}';
</script></body>""", encoding="utf-8")
        result = subprocess.run(
            [str(edge), "--headless=new", "--no-sandbox", "--disable-gpu", "--password-store=basic",
             "--disable-features=EdgeSync,msEdgeProfilePasswordSaving,PasswordManagerOnboarding",
             "--no-first-run", f"--user-data-dir={profile}", "--dump-dom", page.as_uri()],
            capture_output=True, encoding="utf-8", errors="replace", timeout=30, check=True,
            creationflags=subprocess.CREATE_NO_WINDOW,
        )
        match = re.search(r"<body>data:image/png;base64,([A-Za-z0-9+/=]+)</body>", result.stdout)
        if match is None:
            raise RuntimeError("Edge did not render the PicGlance SVG")
        with Image.open(io.BytesIO(base64.b64decode(match[1]))) as image:
            return image.convert("RGBA")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--edge", type=Path, default=EDGE)
    args = parser.parse_args()
    render(args.edge).save(ROOT / "icon.ico", sizes=[(size, size) for size in SIZES])
