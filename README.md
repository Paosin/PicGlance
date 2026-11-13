<p align="center">
  <img src="src-tauri/icons/picglance.svg" width="88" height="88" alt="PicGlance logo">
</p>

<h1 align="center">PicGlance</h1>

<p align="center"><strong>View, edit, and compress images in one Windows app.</strong></p>

<p align="center">
  <a href="https://github.com/Paosin/PicGlance/releases">Downloads</a> ·
  <a href="#features">Features</a> ·
  <a href="#development">Development</a> ·
  <a href="README_zh_CN.md">简体中文</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Windows-x64-0078D4" alt="Windows x64">
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB" alt="Tauri 2">
</p>

Open an image, browse its folder with the arrow keys, then crop or annotate it and export a copy sized for sharing.

## Download

Check [GitHub Releases](https://github.com/Paosin/PicGlance/releases) for available builds. The 2.x build pipeline produces two Windows x64 packages.

| Package | How to use it |
| --- | --- |
| **Offline installer** (`*-Setup-Offline.exe`) | Run the setup wizard. It includes the Microsoft Edge WebView2 Runtime installer for machines that need it. |
| **Portable ZIP** (`*-Portable.zip`) | Extract the entire ZIP and run `PicGlance.exe`. It requires an existing [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) and stores settings next to the executable. |

## Features

- **Browse** — Open images from a file picker, by drag and drop, or from the clipboard. Flip through the current folder with the arrow keys or thumbnail strip. Zoom, rotate the preview, or switch to full screen.
- **Inspect** — See the file name, dimensions, and size at a glance. Open image details to read available EXIF metadata.
- **Edit** — Draw with a brush, add arrows and shapes, crop to a preset ratio, apply a mosaic, or resize. Undo and redo edits, or use **Save As** to keep the original.
- **Compress to a target size** — Set a size in KiB, choose an output format, and optionally lock the dimensions. Compare the original and result before saving. PicGlance shows the actual output size and reports whether it met the target.
- **Make it yours** — Switch between English and Simplified Chinese, and choose a light, dark, or system theme.

### Image formats

| Operation | Formats |
| --- | --- |
| View | JPEG, PNG, APNG, BMP, GIF, WebP, SVG, TIFF, ICO |
| Play animations | GIF, APNG, animated WebP |
| Save edits | JPEG, PNG, BMP |
| Compress or convert | JPEG, PNG, and WebP output for still JPEG, PNG, BMP, and WebP inputs. GIF and WebP output for GIF and animated WebP inputs. |

Pixel editing is available for still raster images.

### Keyboard shortcuts

| Shortcut | Action |
| --- | --- |
| `Ctrl+O` | Open an image |
| `←` / `→` | Previous / next image |
| `Ctrl+0` / `Ctrl+1` | Fit to window / actual size |
| `T` / `I` | Toggle thumbnails / image details |
| `F11` | Toggle full screen |
| `Ctrl+C` / `Ctrl+V` | Copy / paste an image in the viewer |

## Development

PicGlance uses Tauri 2, Rust, TypeScript, and Vite. On Windows, use Node.js 24, the stable Rust MSVC toolchain, Visual Studio C++ Build Tools, and WebView2. See the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#windows) for setup.

Run from the repository root in PowerShell.

```powershell
npm ci
npm run tauri -- dev
```

Build the desktop executable at `target/release/picglance.exe`.

```powershell
npm run tauri -- build --no-bundle -- --locked
```

For distribution, the [Windows workflow](.github/workflows/foundation.yml) uses [the packaging script](scripts/package-desktop.ps1) with Inno Setup 6 and the Microsoft-signed WebView2 Evergreen x64 Standalone Installer. The script writes both packages to `installer/build/` by default.

<details>
<summary>Run the tests</summary>

```powershell
node --test src/shared/*.test.ts src/features/viewer/*.test.ts src/features/editor/*.test.ts src/features/compression/*.test.ts src/assets/icons/tests/*.test.ts
npm run typecheck
cargo +stable test --workspace --locked --all-features
cargo +stable test --locked --manifest-path src-tauri/src/desktop/tests/Cargo.toml
```

</details>

## Contributing

Open a [GitHub issue](https://github.com/Paosin/PicGlance/issues) for bugs or feature ideas. For a bug report, include your Windows version, PicGlance version, and steps to reproduce it. If the problem depends on a particular image, include its format and dimensions and, if you can share it, a sample.

## Third-party software

The Windows packages bundle pngquant and Gifsicle. See [third-party notices and source archives](installer/third-party/SOURCES.md).
