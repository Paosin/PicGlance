<p align="center">
  <img src="src-tauri/icons/picglance.svg" width="88" height="88" alt="PicGlance 标志">
</p>

<h1 align="center">PicGlance</h1>

<p align="center"><strong>在一个 Windows 应用里看图、编辑和压缩。</strong></p>

<p align="center">
  <a href="https://github.com/Paosin/PicGlance/releases">下载</a> ·
  <a href="#功能">功能</a> ·
  <a href="#开发">开发</a> ·
  <a href="README.md">English</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Windows-x64-0078D4" alt="Windows x64">
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB" alt="Tauri 2">
</p>

打开一张图，用方向键浏览同一文件夹里的图片。需要分享时，可以直接裁剪、标注，再按目标体积导出副本。

## 下载

可前往 [GitHub Releases](https://github.com/Paosin/PicGlance/releases) 查看可下载的版本。2.x 构建流程生成两种 Windows x64 包。

| 下载包 | 使用方式 |
| --- | --- |
| **离线安装包**（`*-Setup-Offline.exe`） | 运行安装向导。安装包内附 Microsoft Edge WebView2 运行时安装程序，可在缺少运行时的电脑上使用。 |
| **便携 ZIP**（`*-Portable.zip`） | 完整解压后运行 `PicGlance.exe`。系统需已安装 [WebView2 运行时](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)，应用设置保存在程序目录。 |

## 功能

- **看图** · 通过文件选择器、拖放或剪贴板打开图片。用方向键或缩略图浏览同目录图片，支持缩放、预览旋转和全屏。
- **查看图片信息** · 查看文件名、像素尺寸和文件大小，在图片详情中读取可用的 EXIF 信息。
- **编辑** · 使用画笔、箭头和形状标注图片，按预设比例裁剪、打马赛克或调整尺寸。支持撤销和重做，也可以通过「另存为」保留原图。
- **按目标体积压缩** · 以 KiB 设置目标体积，选择输出格式，也可以锁定像素尺寸。保存前可对比原图和压缩结果，查看实际体积及是否达标。
- **界面设置** · 支持简体中文和英文，可选浅色、深色或跟随系统主题。

### 图片格式

| 操作 | 格式 |
| --- | --- |
| 查看 | JPEG、PNG、APNG、BMP、GIF、WebP、SVG、TIFF、ICO |
| 播放动图 | GIF、APNG、动态 WebP |
| 保存编辑结果 | JPEG、PNG、BMP |
| 压缩或转换 | 静态 JPEG、PNG、BMP 和 WebP 可输出 JPEG、PNG 或 WebP。GIF 和动态 WebP 可输出 GIF 或 WebP。 |

像素编辑适用于静态位图。

### 常用快捷键

| 快捷键 | 操作 |
| --- | --- |
| `Ctrl+O` | 打开图片 |
| `←` / `→` | 上一张 / 下一张 |
| `Ctrl+0` / `Ctrl+1` | 适应窗口 / 原始大小 |
| `T` / `I` | 显示或隐藏缩略图 / 图片详情 |
| `F11` | 切换全屏 |
| `Ctrl+C` / `Ctrl+V` | 在看图界面复制 / 粘贴图片 |

## 开发

PicGlance 使用 Tauri 2、Rust、TypeScript 和 Vite。在 Windows 上准备 Node.js 24、Rust stable MSVC 工具链、Visual Studio C++ 生成工具和 WebView2。安装步骤见 [Tauri 环境要求](https://v2.tauri.app/start/prerequisites/#windows)。

在仓库根目录使用 PowerShell 运行。

```powershell
npm ci
npm run tauri -- dev
```

构建桌面程序，产物位于 `target/release/picglance.exe`。

```powershell
npm run tauri -- build --no-bundle -- --locked
```

[Windows 工作流](.github/workflows/foundation.yml)通过[打包脚本](scripts/package-desktop.ps1)调用 Inno Setup 6，并使用微软签名的 WebView2 Evergreen x64 独立安装程序。脚本默认将两种安装产物输出到 `installer/build/`。

<details>
<summary>运行测试</summary>

```powershell
node --test src/shared/*.test.ts src/features/viewer/*.test.ts src/features/editor/*.test.ts src/features/compression/*.test.ts src/assets/icons/tests/*.test.ts
npm run typecheck
cargo +stable test --workspace --locked --all-features
cargo +stable test --locked --manifest-path src-tauri/src/desktop/tests/Cargo.toml
```

</details>

## 参与贡献

欢迎通过 [GitHub Issues](https://github.com/Paosin/PicGlance/issues) 报告问题或提出功能建议。报告问题时，请附上 Windows 版本、PicGlance 版本和复现步骤。若问题只在特定图片上出现，请注明图片格式和尺寸，并在方便公开时附上样图。

## 第三方软件

Windows 包随附 pngquant 和 Gifsicle。相关许可声明和源码归档见[第三方软件说明](installer/third-party/SOURCES.md)。
