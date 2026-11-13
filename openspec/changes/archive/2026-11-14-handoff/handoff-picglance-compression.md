# PicGlance 压缩包交接

日期 2026-11-14。变更 `picglance-compression`。已核对基线交接的 G0 放行记录。Tauri 内存预算按基线记录留在 G2 验收。

## 实现

- `src-tauri/src/compression/` 提供独立 Cargo 库。按内容识别 JPEG、PNG、GIF、WebP、BMP。兼容源格式优先，BMP 默认 JPEG，APNG 返回不支持。静态和动图均在当前尺寸从最高质量开始搜索。需要缩小时，动画 GIF 按 0.75 倍，其余按 0.5 倍逐级尝试，最长边下限为 16 px。锁定尺寸或达到下限后仍超目标，返回已编码的最小候选和 `met_target=false`。
- JPEG 使用 Rust `mozjpeg` 静态编码，PNG 使用旧版 pngquant，GIF 使用旧版 gifsicle，WebP 使用原生 libwebp 有损编码和动画 mux。GIF 采用 `-O3 -Okeep-empty` 保留帧序列。规格中的 GIF 抽帧通用场景与动图专节冲突，执行动图专节的规则，只对 WebP 抽帧。每组保留首帧，累计该组时长，优先保留尺寸。WebP 转 GIF 的 alpha 按 128 阈值转为一位透明，帧时长按累积时间轴取整到 GIF 的 10 ms 刻度。透明图显式转 JPEG 时以白色合成。
- `tools/` 存放旧版 x64 编码工具及校验值。pngquant 和 gifsicle 均走 stdin/stdout。原 GIF 直接走管道，需要转换时生成内存 GIF。每次外部调用在 30 秒超时或取消时终止并回收子进程，输出和诊断分别限制在 256 MiB 和 8 KiB。迭代期间无临时图像文件。结果只在另存调用后由宿主写出。
- `src/features/compression/` 提供中英文模态对话框、格式选项、进度、取消、原图/结果并排预览、体积差、尺寸、质量、帧数和另存。入口为 `index.ts` 的 `createCompressionDialog(document, originalResource, ports, locale)`。

## Windows 验证

Windows x64、MSVC、Rust 1.100.0、Pillow 12.3.0。在仓库根目录运行。

| 命令 | 结果 |
| --- | --- |
| `cargo test --locked --offline --manifest-path src-tauri/src/compression/Cargo.toml` | 10 项通过，覆盖尺寸锁定、超目标、调色板大小波动、透明边缘缩放、取消、丢帧合并时长、透明度、格式、外部进程超时及错误返回 |
| `node --test src/features/compression/compression.test.ts` | 2 项通过 |
| `npm test` 与 `npm run build` | 6 项共享测试通过，类型检查和生产构建通过 |
| `cargo test --locked --offline --all-features` | 当时共享工作区 17 项通过，1 项 editor 真实样本测试按其标记跳过 |
| `cargo run --release --locked --offline --manifest-path src-tauri/src/compression/Cargo.toml --example real_samples -- src-tauri/src/compression/target/samples` | Windows Release 样本编码通过 |
| `python src-tauri/src/compression/tests/samples/verify.py src-tauri/src/compression/target/samples` | Pillow 独立解码并验证格式、尺寸、帧数、逐帧时长、循环、alpha 和像素误差 |

样本分别保存为以下文件名。Google 样本和 Pillow 动图库夹具只下载到测试输出目录。

- `photo.jpg` 来自 [Google WebP Gallery](https://www.gstatic.com/webp/gallery/2.jpg)。
- `alpha.png` 来自 [Google 透明图样本](https://www.gstatic.com/webp/gallery3/1.png)。
- `animated.gif` 和 `animated.webp` 来自 [Google 动画 GIF 样本](https://www.gstatic.com/webp/animated/1.gif) 与 [WebP 对照样本](https://www.gstatic.com/webp/animated/1.webp)。
- `transparent.gif` 来自 [Pillow 12.3.0 透明 GIF 夹具](https://raw.githubusercontent.com/python-pillow/Pillow/12.3.0/Tests/images/dispose_none_load_end.gif)。

| 输入 → 输出 | 目标和实测字节 | 输出尺寸 | 质量/帧数 |
| --- | ---: | --- | --- |
| JPEG → JPEG | 50,000，88,316 → 49,197 | 550 × 404 | Q66 / 1 |
| PNG → PNG | 55,000，113,340 → 33,233 | 400 × 301 | Q95 / 1 |
| PNG → WebP | 35,000，113,340 → 25,836 | 400 × 301 | Q95 / 1 |
| GIF → GIF | 600,000，890,847 → 599,029 | 300 × 225 | Q74 / 100 |
| GIF → WebP | 350,000，890,847 → 347,886 | 300 × 225 | Q61 / 100 |
| WebP → WebP | 300,000，380,850 → 296,538 | 300 × 225 | Q47 / 100 |
| WebP → WebP（允许抽帧） | 90,000，380,850 → 89,764 | 300 × 225 | Q13 / 50 |
| 透明 GIF → GIF | 23,000，24,601 → 16,419 | 200 × 150 | Q95 / 2 |
| 透明 GIF → WebP | 23,000，24,601 → 17,310 | 200 × 150 | Q95 / 2 |
| 透明 GIF → GIF（极小目标） | 1，24,601 → 994，未达标 | 16 × 12 | Q30 / 2 |

100 帧输入总时长 10,000 ms，保留帧的三项输出逐帧时长相同，抽帧输出每组 200 ms、总时长仍为 10,000 ms。等尺寸 GIF 和 WebP 的 alpha 与对应原帧逐像素一致。PNG 量化的 alpha 平均绝对误差为 0.17/255。原尺寸样本合成到白底后，最差单帧单 RGB 通道平均绝对误差为 10.48/255，对应 JPEG。

## 基线接线请求

### 验收回传后的颜色统计

旧版静态对话框取 `QImage::colorCount()`（索引调色板条目数）和 `QImage::depth()`（每像素位数），结果颜色数为零时隐藏该行。旧版动图路径未填源图颜色统计。新版从实际源字节和最终输出读取，PNG/BMP/GIF 以文件的调色板和像素位宽为准，GIF 取各帧实际使用的最大调色板，JPEG/WebP 使用解码像素格式的位宽。真彩没有调色板值，显示为「—」，两侧都不适用时隐藏该行。

基线的 `src-tauri/src/compression_host.rs` 需要在 `Completed` 序列化 `original_color_count: Option<u32>`、`original_bit_depth: Option<u16>`、`output_color_count: Option<u32>`、`output_bit_depth: Option<u16>`，从引擎 `ResultImage` 转发。前端对话框接收对应的四个 camelCase 字段。宿主测试中直接构造 `ResultImage` 的夹具也需要补这四个字段。当前工作区已由基线将库接入根 workspace，包级复测请使用 `cargo test -p picglance-compression --locked --offline`。

回传复测中，压缩库 11 项和前端 2 项测试通过。用 `cargo run --release --locked --offline -p picglance-compression --example real_samples -- src-tauri/src/compression/target/samples` 重跑上述真实样本，独立 Pillow 校验全部通过。透明 PNG 原始为 RGBA 32 位/像素，量化后为 256 色索引、8 位/像素，实际 33,233 字节，达到 55,000 字节目标。GIF 达标样本为 599,029 字节，1 字节目标样本返回 994 字节并报告未达标。真实 100 帧 GIF 在第一次编码回调取消，返回 `Error::Cancelled`，只执行一次编码并且没有候选字节返回。

最新一轮根工作区测试只剩 `compression_host.rs` 的测试夹具缺少四个新增字段（E0063）。此前 viewer 的借用错误已在复跑中消失。生产壳入口、IPC 全链路和原图/编辑导出的另存仍需宿主集成验证。

- 调用方为压缩对话框、viewer 原图和 editor 导出。基线把 `picglance-compression` 作为路径依赖接入应用工作区并更新公共锁文件。将 tools 作为只读应用资源，用 Tauri 解析出绝对路径。发布前核查这两个外部程序的对应源代码与再分发材料。包内未测固定质量下相对于 Qt 默认 JPEG 的体积差。
- Viewer 已区分原始资源和显示预览。把 `ImageDetails.original` 或编辑导出的二进制资源传给对话框的 `originalResource`。静态预览使用 `document.preview`，动图左侧使用原始资源播放。压缩输入和比较用的尺寸、大小、帧数应来自同一份原始或编辑导出资源。
- 公共 TypeScript 与 Rust 参数增加 `format`、`maxQuality`、`allowFrameDrop`。结果增加 `resource`、`sizeBytes`、`width`、`height`、`frameCount`、`quality`、`metTarget`。无法达到目标仍返回可另存结果。外部工具和编码失败映射到公共 I/O 错误，详细诊断保留在宿主。另存应根据宿主保存对话框和结果资源 ID 写入。
- 在宿主的阻塞线程执行 `compress`。每个任务持有自己的取消标记和结果资源。接收进度回调，把迭代信息映射为单调的进度，完成时置为 1。取消后释放候选资源，并在发布结果前再次检查取消标记。输入、结果和预览均通过现有的受控资源传递。
- `WINDOWS-DEFERRED`。本包已验证 Windows 原生编码。公共 IPC、窗口内操作与另存的端到端验证待基线接线后执行，对应 `picglance-acceptance` 2.1。安装包内工具发现及发布体验证对应 2.3、2.4。

## Windows 发布材料核对

2026-11-14，供 `picglance-acceptance` 2.3/2.4 使用。对 `installer/build/production/` 下的 2.0.0 在线/离线安装器及 2.0.1 升级安装器执行只读解包。使用 Innounp 2.71.3 的 `-s -b <setup>` 列清单、`-e -b -q -d<目录> <setup> "*\tools\*"` 提取工具。两个 Portable ZIP 用 Python `zipfile` 直接读取条目，所有工具用 SHA-256 比较。三套安装器对提取出的两份 EXE 执行 `--version`，与仓库及发布方原档结果一致。

| 压缩工具 | Windows x64 二进制 | SHA-256 |
| --- | --- | --- |
| pngquant 2.17.0（Rust，2021-09） | 743,424 字节 | `7aadad6a50aa5c40cfcb20f9478c92f6030360a3424bc262ce383dc1e97fb86a` |
| LCDF Gifsicle 1.95 | 294,616 字节 | `6f60cc7f696ab4b861bf9e6fb5b4fd940b3cb6b9731e2ef04708334af95a7de4` |

五个候选包的压缩工具载荷完全一致，只含上述 EXE 与 `tools/README.md`，README 哈希 `07ee882d5999266d36b83cfd3b085f9ff58426dbdb76e97ff11cc37717b0d189`。在线 2.0.0、离线 2.0.0、升级 2.0.1 安装器 SHA-256 依次为 `37440e415fa42826329eda0abd05620055efc18b5e8be542ff73832e7f789078`、`c398f9d2465dd72d8954726d20dc948d068d895098116c2fbaebbcdde14e2bec`、`3e0170045196aac7d58a43351b6034fd8c62eb38b98db6146251a461b6b15def`。两份 Portable ZIP SHA-256 依次为 `efabfed612eae9bd0856c9c005fa66471069c3ca20378824f553fe1c82f7e306`、`c3c656a5dc6a2e6d6d719ac6db654e101e2da9dc33c02d379147f474ee394893`。

- **pngquant 二进制来源已确认。** 官方网站提供的 `pngquant-windows.zip`（SHA-256 `bd0257aeeccfe446a4cd764927e26f8af6051796f28abed104307284107b120d`）内 EXE 逐字节匹配。原档同时有 `pngquant/COPYRIGHT`（GPLv3 文本和版权声明）及说明文件，候选包里缺少这些原档材料。官方 2.17.0 源码包 SHA-256 `a27cf0e64db499ccb3ddae9b36036e881f78293e46ec27a9e7a86a3802fcda66`，与项目发布档案给出的哈希相同。源码包含 Rust 构建脚本、Cargo 清单和 libimagequant 源码，`COPYRIGHT` 与 Windows ZIP 中的文件仅换行符有别。未查到将匹配 EXE 绑定到该源码包和具体 Cargo 依赖解析、构建选项的记录。源码包没有 Cargo.lock。 [官方 Windows 下载](https://pngquant.org/pngquant-windows.zip) [官方源码发布档案](https://pngquant.org/releases.html)
- **Gifsicle 二进制来源已确认。** Jernej Simončič 发布的 `gifsicle-1.95-win64.zip`（SHA-256 `7e47dd0bfd5ee47f911464c57faeed89a8709a7625dd1c449b16579889539ee8`）内 EXE 逐字节匹配。原档含 `doc/COPYING`（GPLv2 文本），候选包缺少该文件。Windows 发布方与 LCDF 上游提供的 1.95 源码 tarball 逐字节相同，SHA-256 `b2711647009fd2a13130f3be160532ed46538e762bfc0f020dea50618a7dc950`。源码包含 `src/Makefile.w32` 与 `src/win32cfg.h`，其中 `COPYING` 与 Windows ZIP 中的文件仅换行符有别。发布方同时提供同版本源码归档，但未找到针对匹配 EXE 的构建日志、编译选项或补丁清单。 [Windows ZIP](https://eternallybored.org/misc/gifsicle/releases/gifsicle-1.95-win64.zip) [Windows 发布方源码](https://eternallybored.org/misc/gifsicle/src/gifsicle-1.95.tar.gz) [LCDF 源码](https://www.lcdf.org/gifsicle/gifsicle-1.95.tar.gz)

`target/release/bundle/nsis/PicGlance_2.0.0_x64-setup.exe` 也用 7-Zip 26.01 查过。该较早产物 SHA-256 `4c9f9e134b2624036e9e3632f872d4ae9917f8ba6c59e6cc824f9843c73a0c73`，只列出应用和 NSIS 自身载荷，没有两份工具，不能作为当前安装器载荷的证据。

**验收状态：候选包二进制载荷核对通过，第三方发布材料待补。** 当前包没有原档版权/许可证文件，未见随候选发布的对应源码载荷或面向接收方的对应源码交付记录。已找到可核对的版本源码，但精确构建来源保留上述缺口。可将发布方的原始声明与可确认的对应源码和构建材料交给打包 owner，补齐后重新检查实际下载包。比对所用下载归档和提取物位于本地 `src-tauri/src/compression/target/release-materials/`。 [GPLv2 §3](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html) [GPLv3 §6](https://www.gnu.org/licenses/gpl-3.0.html)
