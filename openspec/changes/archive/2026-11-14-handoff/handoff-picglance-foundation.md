# PicGlance 工程基线交接

日期：2026-11-14。OpenSpec 变更 `picglance-foundation`，主工作区 `dev`，旧版历史 `legacy/v1` → `3340f6c`。

## 实现路径

- 根目录的 Cargo、npm 和 TypeScript 配置及锁文件管理构建。 `.github/workflows/foundation.yml` 定义 Windows Release 构建和测试。
- `index.html`、`src/main.ts` 和 `src/shared/shell.css` 提供无边框窗口、看图画布、编辑和压缩布局预览、主题和语言切换、快捷键与窗口控制。暂用 WebView 文件选择器打开本地图片。安装图标暂用旧版标识。
- `src/shared/` 提供 TypeScript 端口、主题和中英文词典、图标及窗口夹具。 `src-tauri/src/contracts/` 提供 Rust 数据结构。共享 `image-document.fixture.json` 检查 JSON 形状。
- `src-tauri/src/main.rs` 提供 Windows 版本检测及 `first-image-diagnostic` 编译特性。诊断构建接受 `--first-image-diagnostic <jpeg> <marker>`，通过原始 IPC 字节传图。

## 验证

| 命令 | 结果 |
| --- | --- |
| `npm test` | 5 项通过 |
| `npm run build` | TypeScript 类型检查和 Vite 生产构建通过 |
| `cargo test --locked --offline --all-features` | 2 项通过 |
| `npm run tauri -- build --no-bundle --features first-image-diagnostic -- --locked` | Windows x64 诊断 Release 构建成功 |
| `npm run tauri -- build -- --locked` | 常规 Release exe 和 `target/release/bundle/nsis/PicGlance_2.0.0_x64-setup.exe` 构建成功，安装包 1.96 MiB。首次下载超时，续传并核验官方 NSIS 与 Tauri 插件后完成 |
| `python src/shared/tests/measure_first_image.py --tauri target/release/picglance.exe --smoke` | Windows 可见主窗口已验证 |
| `openspec validate picglance-foundation --strict` | 通过 |

## Windows 首图与内存

Windows 11 25H2（构建 26200），AMD Ryzen 7 9800X3D，WebView2 157.0.4341.46。使用 `PicGlance-tmp/build/PicGlance.exe` 的现有旧版产物，未从 `3340f6c` 重新构建核对产物来源。两版以命令行打开同一张合成 JPEG（4000×3000，4,755,877 字节，SHA-256 `cddfd340098109b0b03445e78a550f520c4e85cf4890d696d77471c61fbb06af`）。用窗口中心的已知颜色检测图像显示，从进程创建返回后开始计时。每轮首图后等待 5 秒，按 500 ms 间隔取三次工作集，以十进制 MB 报告。应用进程每轮重新启动，系统磁盘缓存未清空。工作集按进程求和，可能重复计入共享页。

| 轮次 | 旧版首图 ms | 旧版整树 MB | Tauri 首图 ms | Tauri 主进程 MB | WebView2 子树 MB | Tauri 整树 MB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 775 | 237.1 | 730 | 46.8 | 433.5 | 480.2 |
| 2 | 773 | 236.7 | 671 | 47.3 | 437.0–437.1 | 484.4 |
| 3 | 738 | 237.2 | 657 | 46.8 | 429.0–429.2 | 475.8–475.9 |

用 `src/shared/tests/measure_first_image.py` 复测。在交互式 Windows 桌面安装 Pillow 与 psutil 后，传入 `--legacy <旧版 exe> --tauri <诊断版 exe> --jpeg <样本> --runs 3`。本地原始结果位于 `.foundation-tools/first-image-results.json`。可用 Pillow 12.3.0 和 Python 3.12 复建样本。以 `Random(51).randbytes(36_000_000)` 填充 4000×3000 RGB 图，向 `(1100, 850, 2900, 2150)` 区域填入 `(247, 109, 28)`，以 `quality=62` 保存为 JPEG。

## G0 与下游

补测空壳：同一台 Windows 上，常规 Release 与旧版均不传入图片路径，各启动三轮。可见窗口出现 5 秒后，以 500 ms 间隔取三次工作集。新版主进程 46.4–46.7 MB，WebView2 子树 365.0–367.6 MB，整树 411.7–414.4 MB（中位数 412.0 MB）。旧版整树 137.0–138.0 MB（中位数 137.5 MB）。新版打开样本后的整树工作集比空壳高约 60–73 MB，两组分别使用诊断和常规构建。空壳已超过 50 MB 预算。本地原始结果位于 `.foundation-tools/empty-shell-memory.json`。

**2026-11-14：G0 放行。** 用户确认本轮先按内存验证通过推进，继续使用 Tauri。实测工作集仍高于 50 MB，此次放行作为 G0 例外，G2 性能规格继续按原口径验收。viewer、editor、compression、desktop 和 icon-system 可依公共契约开始集成。

`WINDOWS-DEFERRED`：Windows 10 材质降级、安装/升级/卸载及 100%/150%/200% DPI 与跨屏行为留给 `picglance-acceptance` 2.3。资源管理器文件关联首图、冷缓存和发布体积留给其 2.4。当前首图测量覆盖命令行诊断路径。

## WebView2 内存优化复测

使用 WebView2 的 `MemoryUsageTargetLevel`。空画布和隐藏窗口请求 Low，可见窗口加载或浏览图片时恢复 Normal。GPU 加速、进程隔离和亚克力保留。旧 WebView2 运行时缺少该接口时继续使用其默认策略。

下表是同一台机器、同一采样口径的进程树中位数，单位为十进制 MB。私有驻留（USS）只计入各进程独占的驻留页。私有提交包含可换出的私有页。带图实验用应用绘制标记确认加载，避免其他桌面窗口遮挡截图。

| 场景 | 整树工作集（前 → 后） | 私有驻留（前 → 后） | 私有提交（前 → 后） |
| --- | ---: | ---: | ---: |
| 常规 Release 空壳 | 408.6 → 369.1 | 97.2 → 85.3 | 204.8 → 189.5 |
| 诊断版加载同一张 JPEG | 491.3 → 461.1 | 139.8 → 135.3 | 324.0 → 319.1 |
| 诊断版加载后最小化 | 504.7 → 459.9 | 146.4 → 136.0 | 333.3 → 301.0 |

空白 WebView2 实验仍为 382.9 MB 整树工作集、82.6 MB 私有驻留。关闭亚克力的同构建对照仅节约约 0.4 MB 工作集、1.3 MB 私有驻留。带图前台的绘制标记为优化前 550–571 ms、优化后 551–552 ms。实测工作集仍高于 50 MB。

`src/shared/tests/measure_first_image.py` 支持 `--empty`、`--marker-only` 和 `--minimize`，记录进程角色及三项指标。本地原始采样见 `.foundation-tools/memory-*.json`。优化后 `npm test` 6 项、`cargo test --locked --all-features` 2 项通过，常规与诊断 Release 均已重建，NSIS 安装包 1.94 MiB。

## 无边框窗体验证

Windows 11 通过 DWM 隐藏系统边线，保留系统阴影；DWM 不支持该设置时关闭原生阴影。前端去掉外圈描边，保留 8 px 圆角和自绘标题栏。Windows 11 Release 已通过标题栏拖动、双击最大化、右侧和右下角缩放、窗口按钮最大化与还原、最小化与恢复、关闭检查。Windows 10 回退和多 DPI 场景仍由 G2 验证。

## Viewer 与 Editor 接入准备

共享 Cargo 依赖已锁定 `image 0.25.10`（JPEG、PNG、GIF、WebP、BMP、TIFF、ICO）、`resvg 0.45.1`、`kamadak-exif 0.6.2`、Windows `arboard 3.7.0`（图片剪贴板）和 `tauri-plugin-dialog 2.8.4`。两种构建均初始化 Rust 原生对话框插件，WebView ACL 尚未开放插件命令。文件选择和另存可由业务 Rust 命令经插件调用。

壳接线以 Viewer 和 Editor 交付的导出签名为准。Viewer 的打开命令消费 Rust 授权 ID，启动参数、系统打开事件、原生拖放和原生文件选择在 Rust 端授予 ID。启动队列由前端订阅后主动读取，运行时事件用于通知有待处理的 ID。共享 `ImageDocument.preview` 用于预览。编辑需要 Viewer 单独交付的受控全分辨率资源，并按原图 EXIF 方向对齐；应在 Viewer 与 Editor 的 Rust 边界校验该资源。应用壳接入 Editor 后，由编辑会话统一确认打开、切图、离开编辑页及关闭窗口（包括 Alt+F4）。

- Viewer 前端订阅 `picglance:viewer-open-pending` 后调用 `viewer_take_pending_open`，并在订阅成功后主动调用一次。它返回最新的 Rust 授权 ID 或空值，读取即消费。事件不携带路径或 ID。
- Viewer Rust 提供 `Viewer::editor_input(&self, document_id: &str) -> Result<contracts::EditorInput, ViewerError>`。返回服务端保存的文档元数据、该次打开的原始字节快照和可信路径。Editor 以文档 ID 调用，按 EXIF 解码并将返回值交给 `EditorSource::open`。读取磁盘路径当前指向的字节会破坏打开时快照及覆盖冲突检查。
- Editor Rust 可用已注册的 `tauri::State<contracts::EditedResources>` 调用 `publish_png(bytes)` 发布 `ImageResource`。消费者通过同一状态的 `read(&resource.id)` 取得共享字节。该状态保留最近一次导出，已开始的任务可持有返回的 `Arc<Vec<u8>>`。编辑 IPC 只接收文档 ID、编辑操作和保存模式，另存路径由 Rust 调用原生选择器获得。
- Viewer 已注册普通与诊断构建的 Rust 命令及主窗 ACL，应用壳已挂载 Viewer，并保留 `onDocument(document, original)` 返回的原图句柄。Windows Release 实测启动参数打开 JPEG，直接把绝对路径传给 `viewer_open` 返回 `invalid-input`。原生拖放已接线，真实拖放手势和 Desktop 单实例桥接待验。
- Editor 已注册两种构建的 `editor_open`、`editor_render`、`editor_save`、`editor_export` 和 `editor_close`。打开桥接在阻塞线程按可信文档 ID 取 Viewer 原图快照。应用壳挂载编辑器，路由、打开请求及关闭窗口先走未保存确认。无扩展名 SVG 按原图 MIME 类型禁用，按 B 显示本地化原因。剪贴板文档在 Viewer 路由屏蔽 Ctrl+S，Ctrl+Shift+S 直接走 Editor 原生另存。Windows Release 实测 JPEG 编辑笔画、路由和关闭确认、取消/放弃及无扩展名 SVG 禁用通过。
- 编辑器接收 Desktop 快照的 `brushColor`、`brushThickness` 与 `mosaicBlockSize`。压缩编辑结果时，应用壳等待 `exportImage()` 并用导出 PNG 的实际尺寸和字节数初始化压缩对话框。对话框关闭后返回原编辑会话。

## G1 合并构建与 Windows 证据

2026-11-14，Windows 11 25H2。使用 `npm run tauri -- build --no-bundle -- --locked --offline` 构建 `target/release/picglance.exe`（18,331,136 字节），SHA-256 为 `0eb5779a2bb6ce5462e7c95bf4704255d2b3f8e92a56bf3c454d540c6c8739f8`。合并入口启用了 Desktop 安装/卸载命令、单实例转发、设置与关闭行为、Viewer 原图、Editor 导出、压缩进度/取消/原生另存和新图标。资源目录 `tools/` 随 Release 生成。

| 包 | 合并验证 |
| --- | --- |
| Viewer | TypeScript 模型测试与工作区 Rust 测试通过。Release 通过 Explorer Shell 动词、单实例转发、翻图和原图压缩预览 |
| Editor | TypeScript 会话测试、工作区 Rust 测试和显式运行的旧版真实样本测试通过。Release 画笔编辑与原生另存通过 |
| Compression | TypeScript 测试和 11 项 Rust 库及集成测试通过。Release 压缩 Editor 导出，原生另存并独立解码；另验证 Viewer 原图能完成压缩并显示结果 |
| Desktop | 8 项独立测试通过。Release 的 Desktop 单实例将 Explorer 的 Unicode 路径送入同一个 WebView；现有独立宿主的生产安装器命令与安装升级卸载结果见桌面交接 |
| Icon system | 3 项 TypeScript 测试通过。`python src-tauri/icons/tests/windows_shell.py --exe target/release/picglance.exe` 从同一 Release 原生提取 10 个尺寸，均与新 ICO 一致。端到端确认自绘标题栏 SVG 已加载 |

复测命令包括 `node --test src/shared/*.test.ts src/features/viewer/*.test.ts src/features/editor/*.test.ts src/features/compression/*.test.ts src/assets/icons/tests/*.test.ts`（22 项），`cargo test --workspace --locked --offline --all-features`（应用 24 项，压缩 11 项，1 项真实样本默认跳过），`cargo test --locked --offline --all-features -- real_images_preserve_source_and_render_correct_pixels --ignored`（1 项），以及 `cargo test --locked --offline --manifest-path src-tauri/src/desktop/tests/Cargo.toml`（8 项）。

Explorer `Shell.Application` 调用独立临时 JPG 测试动词，保留当前账户已有的 PicGlance 关联。已有 Release 进程接收第一张图，右方向键切至第二张，画笔修改中心像素，原生对话框另存 PNG，随后从编辑会话导出并压缩。Pillow 独立重新解码 PNG 和结果 JPEG，均为 800×600。两张源文件 SHA-256 在操作前后相同。原始数据位于 `.foundation-tools/g1-032104/evidence.json`。

| 文件 | 字节或结果 | SHA-256 |
| --- | --- | --- |
| `01 测试图.jpg` | 413,980 | `a3981f9a1bd96a530317151f92e751d379601044781698763c0756cc85085b85` |
| `02 编辑源.jpg` | 413,809 | `b8443bedebd6aea74bddd60965161efbf64400a0fc2516659586b7cd01533c8f` |
| `03 已编辑.png` | 1,920,813 | `0ddf925f8a8f9fa6553384194bbc5c424ff717d0701e1107bb6ed364728fa91d` |
| `04 压缩.jpg` | 目标 46,080，实际 43,688，达标 | `6193a4deeb538a32ab13c4f5647d1358b5383a69257efabf4f15010d4af75db9` |

同一 exe 的 Inno 在线安装包 8,225,561 字节，离线安装包 221,538,077 字节，便携 ZIP 6,985,463 字节，均位于 `.foundation-tools/g1-packages/`。便携 ZIP 内 exe 的 SHA-256 与上述 Release 一致，包含两个编码器和 `portable.marker`。从 ZIP 解压后已实测启动并打开生产 Desktop 设置，`desktop_read` 返回 `portable=true`。编码器与既有 README 的摘要匹配，精确对应源码及再分发材料仍待发布前确认。

图标浏览器夹具的 12 组主题、语言与模拟 DPR 测试通过，ICO 原生尺寸检查通过。

验收可复测上述合并链路。Viewer、Desktop 和 Icon system 尚有任务待验，不能以本轮自动测试替代其任务状态。正式安装版 Explorer 默认双击、实际 100%/150%/200% DPI 下的标题栏/运行与固定任务栏/快捷方式、Windows 10、跨屏和安装生命周期继续交由 G2。G2 的 50 MB 预算仍适用，离线安装包实测超过 30 MB。200 MP 实测覆盖 8-bit sequential JPEG，其他格式和渐进式 JPEG 暂沿用现有解码限制。

## G2 发布协调

2026-11-14，交互式 Windows 11 25H2。使用上方同一正式 Release（SHA-256 `0eb5779a2bb6ce5462e7c95bf4704255d2b3f8e92a56bf3c454d540c6c8739f8`）和 4000×3000、4,755,877 字节 JPEG（SHA-256 `cddfd340098109b0b03445e78a550f520c4e85cf4890d696d77471c61fbb06af`）。每轮从无 PicGlance 进程开始，系统磁盘缓存未清空。通过 `ShellExecuteExW` 调用临时 JPG 动词，从调用前计时到窗口中心采样显示完整颜色。首图后等待 5 秒，每隔 500 ms 采样，共三次。每个样本均含 6 个 WebView2 子进程。

| 轮次 | Shell 首图 ms | 主进程工作集 MB | WebView2 子树工作集 MB | 整树工作集 MB |
| --- | ---: | ---: | ---: | ---: |
| 1 | 744 | 42.65 | 372.96–373.93 | 415.61–416.58 |
| 2 | 651 | 42.64 | 364.60–364.91 | 407.23–407.54 |
| 3 | 657 | 42.54 | 364.09–364.10 | 406.63–406.64 |

九次采样中位数为主进程 42.64 MB、WebView2 364.88 MB、整树 407.51 MB、整树私有驻留 107.44 MB、整树私有提交 234.48 MB。工作集按进程相加并可能重复计入共享页。原始记录在 `.foundation-tools/g2-release-shell-memory.json`，测量脚本在 `.foundation-tools/g2-shell-memory.py`。当前账户的 JPG 默认关联指向其他处理程序，临时动词已清理。默认关联双击计时仍需在已通过 Windows 设置选择 PicGlance 的验收账户复测。

- **内存发布阻塞。** 生产 Release 的 407.51 MB 中位数约为 G2 整树 50 MB 预算的 8.15 倍。G0 放行只覆盖工程推进。发布协调需决定是否保留现有门槛并重新评估运行时，或正式修订发布预算。
- **离线包发布阻塞。** 同一 Release 离线安装包为 221,538,077 字节（SHA-256 `8118b6549b3e1f9eb39a42a05934d3e446db93900d4dbbae6a6d055914429c77`），内嵌的 WebView2 独立安装器单文件为 215,195,856 字节。在线安装包与便携 ZIP 各自低于 30 MB。Desktop 发布协调需明确全离线自包含产物是否保留，以及它适用的体积门槛。现有 30 MB 规格仍适用。

### 续验口径和业务日志

按验收转达的用户决定，使用 Explorer 右键入口，不设置默认应用。内存和包体判定待发布协调，保留前述实测。

在 Windows 11 交互会话上使用合并 Release `89c87ca6d11f0f2ab8cba919d83173f881ba2d971234ef4c9c4cf95952967bb0`。Viewer 从 Rust 授权路径打开文件，在工作线程将失败原因交给 Desktop 日志。验证两个带中文和空格的文件名，界面分别显示损坏提示和超限提示。实际日志内容包括：

- `viewer-open-failed path=…损坏 样本.jpg reason=invalid-input`。
- `viewer-open-failed path=…超限 样本.jpg reason=too-large size_bytes=83886081 limit_bytes=83886080`。

原始日志位于 `.foundation-tools/g2-business-logs/{broken,large}/log/picglance.log`。两个独立便携会话均启用启动检查。GitHub 的实际请求返回 HTTP 404，写入 `update-check` 日志，界面没有显示更新横幅。另在隔离的 WebView2 测试会话用受控 `v2.99.0` 替换更新 IPC 返回值，验证同一 Release 的启动横幅显示版本并可点「忽略」关闭。真实 GitHub 更高版本发现仍无实测。

此构建的工作区测试为应用 25 项、压缩 11 项通过，1 项旧版样本默认跳过。桌面迁移测试需要可写的 HKCU，在交互账户通过。用同一 exe 构建的三个候选包位于 `.foundation-tools/g2-business-logs/packages/`，便携 ZIP 中 exe 哈希已核对。

| 候选包 | 字节数 | SHA-256 |
| --- | ---: | --- |
| 在线安装 | 8,226,505 | `d8fb81a3325fa45b3aa8a67664eec17f2253db0a8e26a45ad0820e447a8079a2` |
| 离线安装 | 221,538,938 | `e245a92e9ddecb9766e953917a965818f3f8325ca78340b050725c628adebb55` |
| 便携 ZIP | 6,972,859 | `09166d454cf83738d43124184047755bb720f4730b5a0b78607d4ae2a5c23d9d` |
