# Compression tools

The Windows x64 executables were copied from the legacy PicGlance tools directory. The compression worker receives the absolute directory path from the native host and passes only fixed encoder options and image bytes to the executables.

| Executable | Version | SHA-256 | Upstream source and license |
| --- | --- | --- | --- |
| `pngquant.exe` | 2.17.0 (Rust) | `7aadad6a50aa5c40cfcb20f9478c92f6030360a3424bc262ce383dc1e97fb86a` | [pngquant 2.17.0](https://github.com/kornelski/pngquant/tree/2.17.0) |
| `gifsicle.exe` | 1.95 | `6f60cc7f696ab4b861bf9e6fb5b4fd940b3cb6b9731e2ef04708334af95a7de4` | [Gifsicle 1.95](https://github.com/kohler/gifsicle/tree/v1.95) |

The original Windows Gifsicle build distributor is listed in the [legacy download instructions](https://eternallybored.org/misc/gifsicle/). Before publishing installers containing these executables, verify the corresponding source and redistribution materials for the exact binaries.
