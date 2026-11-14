# PicGlance Windows 验收报告

2026-11-14 最终结论：用户确认本轮全部项目正常验收，G2 验收任务关闭。最终交付为内置 WebView2 的 Windows x64 离线安装包。以下为此前的实测记录，保留当时观察到的预算偏差和验证缺口。

## G1 证据

**阶段验收记录（2026-11-14）**。G1 通过，G2 未通过，OpenSpec 完成 3/6。内存和安装包大小按用户决定待定。最新 Windows x64 离线候选位于 `installer/build/production/legacy-menu-fix/`，内置完整 WebView2，覆盖安装成功。资源管理器传统图形菜单在重启 Explorer 后仍未显示新版入口，Shell 枚举和调用虽已启动新版，实际右键操作仍不通过。Viewer 候选第二轮 42 次切图均低于 100 ms，首轮仍有两次 PNG 超时和一次 JPEG 未捕获；25 fps GIF 两轮分别观察到 4、6 次帧号跳跃。真实三档 DPI 已部分取证，任务栏、跨屏和无 WebView2 的干净环境断网补装待验。最新候选已随包加入工具版权、许可证和版本源码归档，精确构建来源仍待确认。

Foundation 的补交见[基线交接](../../openspec/changes/archive/2026-11-14-handoff/handoff-picglance-foundation.md#g1-合并构建与-windows-证据)。同一 `target/release/picglance.exe` 构建通过，SHA-256 `0eb5779a2bb6ce5462e7c95bf4704255d2b3f8e92a56bf3c454d540c6c8739f8`。合并前端测试 22 项、应用 Rust 24 项、压缩 11 项、编辑旧样本 1 项、桌面测试 8 项均通过。五包接线及图标结果见基线交接。

Windows 实测用 Explorer `Shell.Application` 调用临时 JPG 动词，将第一张图片送入已运行的同一进程。右方向键切至第二张图、画笔修改、原生另存 PNG、从编辑会话压缩并原生另存 JPEG。验收复核 `.foundation-tools/g1-032104/evidence.json` 及四份文件的 SHA-256，使用 Pillow 独立重新解码。两张源图哈希与操作前记录一致。另存 PNG 为 1,920,813 字节，中心像素由 `(91, 91, 93)` 变为 `(255, 0, 0, 255)`。压缩输出维持 800×600，目标 46,080 字节，实际 43,688 字节，达标。默认双击、正式关联的菜单手势及冷启动仍由 G2 核验。

### 首轮阻塞记录

| 项目 | 证据与结果 |
|---|---|
| 规划与交接 | 已读本包 proposal、design、tasks、两份规格及基线总设计。Viewer、Editor、Compression、Desktop、Icon System 五份交接均已找到，各有包级构建或测试记录。Desktop 使用独立 Release 测试宿主，Compression 使用独立库及样本，Icon System 的新 ICO 测试用了显式构建配置。 |
| 本轮共享构建 | `npm test` 6 项通过。`npm run typecheck` 通过。`node --test src/features/viewer/model.test.ts src/features/editor/session.test.ts src/features/compression/compression.test.ts src/assets/icons/tests/icons.test.ts` 16 项通过。`npm run tauri -- build --no-bundle -- --locked --offline` 退出 0，生成 15,700,992 字节的 `target/release/picglance.exe`，SHA-256 `0f2a23a713a1534862a15b1d9a4d7a20c489924366bb2829c4277465b2f4224c`。使用 `.foundation-tools/cargo` 与 `.foundation-tools/rustup`。 |
| 首轮生产入口检查 | `src/main.ts` 的压缩页当时仍为布局预览，`src-tauri/src/main.rs` 尚未注册压缩 IPC、Desktop 宿主、安装/卸载命令或单实例插件。`tauri.conf.json` 当时仍引用 `foundation.ico`。之后工作区出现新的生产接线与图标配置，复验以 owner 交付的同一份构建和真实流程记录为准。 |
| 真实链路 | 缺少同一正式 Release 上的资源管理器打开 → 翻图 → 编辑 → 另存 → 按目标体积压缩的证据。首轮构建运行 `node src/features/editor/smoke.test.mjs target/release/picglance.exe`，等待 WebView2 DevTools 页面超时，尚未到编辑步骤。Viewer 的启动参数冒烟、Editor 的 PNG 另存、Compression 样本编码、Desktop 测试宿主的 Shell 动词均可用于定位和复测。 |

## 旧规格、归档及实际功能对照

逐场景状态、源规格行号和复测证据见[旧版逐场景验收矩阵](legacy-scenario-matrix.md)，覆盖 155 条旧主规格场景及 11 条压缩归档增量。

逐场景状态、源规格行号和复测证据见[旧版逐场景验收矩阵](legacy-scenario-matrix.md)，覆盖 155 条旧主规格场景及 11 条压缩归档增量。

基线取自旧 `openspec/specs` 的 12 份规格及 `add-image-viewer-core`、`refactor-compress-encoders` 两个归档。对比归档和现行同名规格的文件哈希，只有两份归档中的 `image-compression` 与现行文件不同。合并理解采用现行主规格及后续压缩归档的编码行为。下表记录需求覆盖点和当前证据，所有行仍待生产验收。

| 旧规格 | 场景与复测项 | 现有证据或差异 |
|---|---|---|
| `image-viewing` | 八类静态格式、动图、未知格式、先显示窗口、超限确认、双向循环、缩略图点击与懒加载、初始显示、锚定缩放、适应/原始尺寸、双向旋转与双轴翻转、全屏和退出、拖放、损坏/删除/中文路径、复制/粘贴、切图动效与连续切图 | Viewer 包级测试覆盖目录、删除、缩放、预览不改像素及迟到请求。交接中的 Windows 冒烟覆盖长路径、动图、剪贴板等。补真实 Shell 拖放、失败矩阵、全屏快捷键及缩略图性能。 |
| `image-info` | 底栏显示与淡入淡出，EXIF 字段、无数据、关闭方式、GPS 提示 | Viewer 交接有 GPS 冒烟。旧 `ExifPanel.qml` 支持编辑文件名并调用 `MainViewModel::renameFile`，首轮新版 Viewer 未找到等效入口。补测重命名、重名失败及目录状态。 |
| `image-editing` | 工具选中态、缩放指示、笔画、四种形状、两类裁剪、六种马赛克、旋转/翻转/尺寸、与预览独立、撤销/重做/切图、覆盖/另存/未保存保护、缩放后绘制、SVG 禁用 | Editor 有像素、撤销、保存和限制的分包证据。补旧版 `main.qml` 已交付的 BMP 另存筛选和实际解码，并核对旧版 QML 消费的默认画笔颜色/粗细与新版设置接线。 |
| `image-compression` | 质量搜索、缩小回退、锁尺寸、未达标、格式保留、GIF/WebP 参数、结果对比、格式组合、禁用不支持入口、取消和磁盘写入 | 独立库已测样本编码及输出字节。旧 `CompressDialog.qml` 实际展示原/结果颜色数、位深，首轮新版结果模型缺少这两项。旧 `CompressViewModel` 允许 BMP 输入，需用生产压缩路径复验。 |
| `compress-format-lock` | JPEG/PNG/GIF/WebP 默认源格式且排首位，兼容格式可显式选择，动图选项仅 GIF/WebP | 独立对话框测试覆盖格式排序，Windows 产品需复验默认值、显式转换和保存扩展名。BMP 沿用兼容输出规则，验证默认 JPEG 及可选格式。 |
| `compress-strategy-pattern` | MozJPEG、pngquant、gifsicle、WebP、质量映射、后台搜索、进程超时/取消、压缩后帧数与时长、同质量 JPEG 体积差 | 新版 libwebp 代替 Qt WebP，按有效输出、质量和时序验证。GIF 保留帧序列，WebP 才允许抽帧。新版外部工具走内存管道，验收迭代不落临时图片。正式产物仍需测工具发现及同质量 JPEG 对照。 |
| `app-shell-ui` | 三种主题、Win11 Acrylic/Win10 回退、窗口形态、工具栏、底栏/缩略图/EXIF、键盘、高 DPI/跨屏、动效 | 基线有窗口交互记录。照片优先的文字入口采用新版基线设计，保留旧版动作、快捷键和反馈。图标夹具覆盖模拟 DPR，实际显示缩放待测。 |
| `settings` | 七类设置、即时应用与重启持久化、三种主题、关闭三模式、关联管理、便携存储 | Desktop 独立宿主有 INI 和关闭行为证据。生产配置需复测小图放大、默认缩放、编辑默认值及压缩默认值。 |
| `localization` | 中英文即时切换、可见文本覆盖、Shell 文案 | 分包有双语和测试夹具。生产应用及 Explorer 关联菜单需分别核对。 |
| `windows-integration` | 默认勾选、取消 PNG、Win10/11 菜单、桌面/开始菜单快捷方式、单实例 | Desktop 测试宿主已验证传统动词与 PID 复用。默认应用按产品接线后的 Windows 设置实际选择并记录，安装器默认勾选、默认应用与注销分别验。 |
| `app-distribution` | 安装、覆盖升级、关联差量、卸载、便携隔离、日志、更新横幅/网络失败/关闭检查 | 现有交接以测试宿主执行了模拟版本升级。正式两版 exe、独立便携、在线与离线运行时引导、业务日志和更新横幅待验。 |
| `performance-budget` | 关联首图 ≤1 s、4000×3000 整树 ≤50 MB、超大图峰值、切图 ≤100 ms、≤30 fps 动图无掉帧、两类包 ≤30 MB | 见下节实测及缺口。旧 README 的便携 46 MB 是历史记录，门槛沿用规格的 30 MB。 |

## G2 实测与缺口

2026-11-14 用户决定将安装包大小判定列为待定，保留体积实测值和原预算。文件打开按实际资源管理器右键菜单验收，首图沿同一入口测量。交付内置完整 WebView2 的 Windows x64 离线安装包。

合并候选的 Inno 解包清单已核对，包含 `{tmp}\WebView2Setup.exe`、PicGlance 主程序与两个压缩工具。安装器的 10 层原生图标与新 ICO 逐像素一致。

同一 G1 Release 的正式补测已取得三轮数据。验收复核 `.foundation-tools/g2-release-shell-memory.json` 的 EXE 和 4000×3000 JPEG 哈希及九次采样。计时从调用 Windows Shell 动词到截图识别图像区域，进程每轮重启，首图后等待 5 秒再每 500 ms 采样。工作集用十进制 MB。

| 轮次 | 首图 | 主进程 | WebView2 子树 | 整树 |
|---|---:|---:|---:|---:|
| 1 | 744 ms | 42.65 MB | 372.96–373.93 MB | 415.61–416.58 MB |
| 2 | 651 ms | 42.64 MB | 364.60–364.91 MB | 407.23–407.54 MB |
| 3 | 657 ms | 42.54 MB | 364.09–364.10 MB | 406.63–406.64 MB |

**内存项待定。** 按 2026-11-14 用户决定，暂停内存排查与复测。保留整树中位数 407.51 MB 和原 50 MB 预算。三轮 Shell 动词首图均低于 1 秒。实际默认应用双击与系统冷缓存仍待验。

- **首图。** Windows 11 25H2（26200），WebView2 157.0.4341.46，使用 4000×3000、4,755,877 字节 JPEG（`.foundation-tools/first-image.jpg`，SHA-256 `cddfd340098109b0b03445e78a550f520c4e85cf4890d696d77471c61fbb06af`）。分别调用 `src/shared/tests/measure_first_image.py` 的 `measure(exe, jpeg)` 测新版及旧版命令行打开，两个进程的画面采样均返回 `screen grab failed`。`WINDOWS-DEFERRED`，用交互桌面从 Explorer 双击计到完整显示，记录至少三轮及缓存条件。
- **内存。** 使用 Python 3.12、Pillow 12.3.0 和 psutil 7.2.2，以命令行传同一 JPEG，从启动后第 5 秒起每 500 ms 采三次工作集，按根进程和所有递归子进程汇总。已有旧版 Qt exe 三轮整树分别为 233.43、232.72、233.10 MB，无 WebView2 子进程。首轮新版未设置独立 WebView2 配置目录，5 秒内退出（101），此方法无新版稳态样本。基线诊断版既有记录为 461.1 MB 整树中位数，超过 50 MB；G0 内存例外的适用范围为基线放行。正式构建需确认图片实际呈现，分列主进程、WebView2 子树及整树工作集。
- **超大图、切图、动图。** Viewer `decode.rs` 限制 48 MP，20,000×10,000 的场景超限，允许范围内的静态图先解码再缩小。待 Viewer 提交下采样路径和峰值数据。`WINDOWS-DEFERRED`，实测连续翻图从输入到开始渲染、≤30 fps 的 GIF/APNG/WebP 帧呈现，并覆盖缩略图按需加载。
- **独立配置目录补采。** 首轮新版通过 `WEBVIEW2_USER_DATA_FOLDER` 使用临时目录，命令行打开同一 JPEG。从启动后第 5 秒起，三次采样的主进程分别为 24.87、25.03、25.76 MB，WebView2 子树分别为 121.49、149.32、0 MB，整树分别为 146.36、174.34、25.76 MB。第三次已无 WebView2 子进程，且未观察到实际图片呈现。本组为启动诊断采样，稳态测试待交互环境复测。
- **图标与 DPI。** `python src-tauri/icons/tests/windows_shell.py --exe target/release/picglance.exe` 在首轮构建的 16 px 图层断言失败，证实该 EXE 与新 ICO 不同。`WINDOWS-DEFERRED`，在正式产物上重跑原生图层提取，并在实际 100%/150%/200% 系统缩放检查标题栏、任务栏、Explorer、安装快捷方式、控件、焦点及跨屏行为。
- **包体。** 以文件字节数检查现有产物。旧基线 NSIS 为 2,035,734 字节。`installer/build` 的 Desktop 测试宿主 2.0.0 在线安装包为 5,682,197 字节、离线安装包为 218,993,505 字节、便携 ZIP 为 3,040,987 字节。ZIP 只有 `PicGlance.exe` 和 `portable.marker`，内含 exe SHA-256 `edd46ef8c75b13a4bc6a752bf3fef14abae602e89af5c8b45deaa02e2f00089e` 与 `target/desktop-host/release/picglance-desktop-host.exe` 相同。测试离线包超预算；正式在线安装包、离线安装包和便携 ZIP 待拿到同一 Release 后记录实际字节、哈希、解压清单和工具可用性。
- **完整流程和 Windows 安装。** `WINDOWS-DEFERRED`。同一 Release 上以两张真实图片验证 Explorer 打开、箭头翻图、编辑、另存、重新打开另存图并压缩。留存原文件前后哈希、另存和压缩输出的解码信息、目标与实际字节、界面达标状态。安装和便携用独立测试环境核对设置、关联、单实例和卸载结果，另在无 WebView2 的环境复测联网和断网安装。

## 转交与复测条件

已将 G1 合并构建及真实链路交给 Foundation owner，包体与 Shell 场景交给 Desktop owner，压缩接线与颜色数/位深交给 Compression owner，重命名与超大图交给 Viewer owner，发布图标与实际 DPI 交给 Icon System owner。回收正式 Release、安装与便携产物、相同构建的五包证据及 G1 实际流程记录后，执行上述矩阵及性能复测。

Compression owner 已补齐原图和输出的颜色数及位深。其 Windows 独立 Release 复测中，透明 PNG 从 113,340 字节压至 33,233 字节，达到 55,000 字节目标，输出为 256 色、每像素 8 位。极小目标返回 994 字节并标记未达标，取消样本在首轮编码后结束。仍需在生产 IPC 和 UI 上复验，见压缩交接。

Viewer owner 的 Windows Release 初测已打开 20,000×10,000 的基线 JPEG，首图约 823 ms，主进程峰值工作集 87.5 MiB，整树峰值 385.1 MiB。完整证据及相同正式构建的复测仍待回收。Editor owner 已用 `node src/features/editor/smoke.test.mjs target/release/picglance.exe brush-settings` 验证了持久化颜色 `#29B45D` 与 17 px 粗细的实绘像素、原生 BMP 另存与重开。输出 227×149，原 JPEG 字节一致。

## Desktop 集成候选产物复测

目录 `installer/build/production`。首轮候选 EXE SHA-256 `0c0ec4022ff93ed047b1eb58c3a9fc5351a5879bb8e1a975cf7cb23226975214`，与首轮候选 ZIP 中的 EXE 相同。Desktop owner 确认此轮 EXE 指向开发服务器，随后切换为内嵌前端资源的生产构建。首轮 EXE 和后续 `target/release/picglance.exe`（SHA-256 `0eb5779a2bb6ce5462e7c95bf4704255d2b3f8e92a56bf3c454d540c6c8739f8`）均通过 `windows_shell.py --exe <exe>` 的 10 层原生提取与逐像素比对。实际 DPI 下的 Windows Shell 显示待验。

| 2.0.0 产物 | 字节 | SHA-256 | 30 MB 预算 |
|---|---:|---|---|
| 在线安装包 | 8,183,468 | `8d9e7c92cdb8a05f814bf5656da2caa3a6fc6852748f3e4f8de0b52f7a69800a` | 达标 |
| 离线安装包 | 221,495,729 | `7e43841cee33a7b42e92ad0e721b879a208ffd0c60a7f3b6caed334347ab6c16` | 超标 |
| 便携 ZIP | 7,461,861 | `e56e5eb117694f1ca4cd1e2f69807e0c7e3a97aceeb4394873ffeb1d5dcedd18` | 达标，内容待修 |

便携 ZIP 的 `tools/pngquant.exe`、`tools/gifsicle.exe` 和 `tools/README.md` 各出现两次，已将重复条目交给 Desktop owner，待重打包复查清单、大小和哈希。

去重后的 ZIP 已复查。大小 6,920,245 字节，SHA-256 `7a91d58895c8760c8c8d30ac1a4b81131f538816a500b94aec8b78ca5eacc059`。`ZipFile.testzip()` 通过，五个条目名称唯一，两个压缩工具及说明与工作区原件逐字节一致。正式发布构建仍需核实产物哈希。

## 同一 G1 Release 的包体复测

产物位于 `.foundation-tools/g1-packages/`。验收已核对便携 ZIP 内 exe 的 SHA-256 与 G1 相同，五个条目唯一，ZIP 完整性通过，两个工具及说明与原件一致。

| 产物 | 字节 | SHA-256 | 30 MB 预算 |
|---|---:|---|---|
| 在线安装包 | 8,225,561 | `0e9f559f0bc2a8b4b2a41ba42cab0898ec508a513961db2aa1486c0158cf72e8` | 达标 |
| 离线安装包 | 221,538,077 | `8118b6549b3e1f9eb39a42a05934d3e446db93900d4dbbae6a6d055914429c77` | 超标 |
| 便携 ZIP | 6,985,463 | `03b38f78df5660eed7169a7be5ad081c3490bf65cc8e84a1e60f72f4d86c9190` | 达标 |

Editor 兼容复测已通过。Windows Release 从便携 INI 载入颜色 `#29B45D` 和 17 px，实绘像素正确。原生对话框另存 227×149 BMP 并成功重开，原 JPEG 字节保持一致。

## 业务修复复测

### 实际 Windows DPI

在 Windows 11 26200 的显示设置依次切换 100%、150%、200%，同一合并便携候选（SHA-256 `89c87ca6d11f0f2ab8cba919d83173f881ba2d971234ef4c9c4cf95952967bb0`）用 `windows_dpi.py --pid <PID>` 测得系统／窗口 DPI 分别为 96／96、144／144、192／192。结束后恢复原 125%／120 DPI。合并候选原生 10 层图标与新 ICO 一致。

三档均目视核对了另一份 Desktop 安装候选的深色标题栏和看图图标。125% 下在资源管理器确认了 EXE、两种安装器和开始菜单快捷方式的新图标。运行及固定任务栏、各档 Shell 完整矩阵、同一构建的浅色与键盘/读屏仍待验。当前环境只有一台显示器，跨屏待验。各构建与命令见[图标系统交接](../../openspec/changes/archive/2026-11-14-handoff/handoff-picglance-icon-system.md#真实-windows-dpi-补测)。

### 离线安装和右键迁移续验

- Compression owner 解包核对了三个 Inno 安装器和两个便携 ZIP。`pngquant.exe` 2.17.0 与 `gifsicle.exe` 1.95 均与各自发布方的 Windows 下载逐字节匹配。候选包缺原档版权/许可证文件及对应源码交付记录，已交 Desktop owner 补齐。构建来源和复核命令见[压缩交接](../../openspec/changes/archive/2026-11-14-handoff/handoff-picglance-compression.md#Windows-发布材料核对)。

- 交付候选为 `.foundation-tools/g1-packages/PicGlance-2.0.0-x64-Setup-Offline.exe`，内置微软签名的 WebView2 Evergreen x64 独立安装程序。安装器在运行时已存在时直接安装 PicGlance，缺失时从包内提取并执行独立安装程序。当前 Windows 11 已有 WebView2，干净系统断网补装待验。
- 命令行会话尝试执行离线安装器，Inno Setup 返回 1，日志显示该会话无法解析 `LocalAppData`。改用交互式向导完成安装，默认应用选项保持未勾选。安装目录的 2.0.0 启动并加载生产界面，原生文件选择器成功打开 800×600 JPEG，压缩对话框返回预览、实际字节显示和质量信息。
- 在同时存在旧版的 Windows 11 机器上，右键 JPEG →「显示更多选项」→「用 PicGlance 打开」启动了 `D:\Program Files\PicGlance\PicGlance.exe`。已将旧通配菜单迁移缺陷交 Desktop owner。修复后需用新版安装包重跑实际菜单、中文文件名冷启动、第二张图转发及卸载恢复。
- Desktop 已实现旧菜单迁移并通过注册回归。修复版覆盖安装成功，Shell 枚举显示新版命令。现有 Explorer 图形菜单仍缓存旧条目，真实点击复验待资源管理器刷新。默认图片应用保持 Windows 照片。
- 合并候选 `.foundation-tools/g2-business-logs/packages/PicGlance-2.0.0-x64-Setup-Offline.exe` 包含右键迁移和 Viewer 日志接线，内置完整 WebView2，大小 221,538,938 字节，SHA-256 `e245a92e9ddecb9766e953917a965818f3f8325ca78340b050725c628adebb55`。在新 Release 实测损坏 JPEG 与超限样本均显示提示，日志包含路径、原因，超限时另记录实际字节与阈值。真实更新请求 HTTP 404 保持静默。受控 `v2.99.0` 响应验证横幅显示与忽略，真实发布发现仍待验。
- 看图基线实测 4,000×3,000 JPEG 的 12 次切图全部超过 100 ms，中位数 156.6 ms；1,600×1,200 PNG 的 12 次同样超标，中位数 106.4 ms。GIF、APNG、WebP 样本分别观察到 3、1、1 次帧号跳跃。修复候选的一轮动图采样未观察到跳帧，切图仍有超过 100 ms 的样本，完整复测待 Viewer 交付。原始样本见 `target/viewer-performance-*.json`。
- Viewer 后续候选 `8e2f7dc6bbfc3d22f73342df2884f2455cc44f15df351234e377d1d4c03b6fb3` 在 `viewer-performance-repeat.json` 记录的 42 次切图均低于 100 ms。800×600 JPEG、4,000×3,000 JPEG、1,600×1,200 PNG 的 P95 分别为 82.4、84.5、82.3 ms。640×480 APNG 与 WebP 无观察到的跳帧，25 fps GIF 出现 6 次帧号跳跃。另以 20,000×10,000 样本确认渐进 JPEG、PNG、TIFF 均报 `dimensions`，基线 JPEG 的支持证据见 Viewer 交接。该候选与当前离线包的 EXE 哈希不同，待合并产物复验。
- Desktop 已实现旧菜单迁移并通过注册回归。修复版覆盖安装成功，Shell 枚举显示新版命令。现有 Explorer 图形菜单仍缓存旧条目，真实点击复验待资源管理器刷新。默认图片应用保持 Windows 照片。
- 合并候选 `.foundation-tools/g2-business-logs/packages/PicGlance-2.0.0-x64-Setup-Offline.exe` 包含右键迁移和 Viewer 日志接线，内置完整 WebView2，大小 221,538,938 字节，SHA-256 `e245a92e9ddecb9766e953917a965818f3f8325ca78340b050725c628adebb55`。在新 Release 实测损坏 JPEG 与超限样本均显示提示，日志包含路径、原因，超限时另记录实际字节与阈值。真实更新请求 HTTP 404 保持静默。受控 `v2.99.0` 响应验证横幅显示与忽略，真实发布发现仍待验。
- 看图基线实测 4,000×3,000 JPEG 的 12 次切图全部超过 100 ms，中位数 156.6 ms；1,600×1,200 PNG 的 12 次同样超标，中位数 106.4 ms。GIF、APNG、WebP 样本分别观察到 3、1、1 次帧号跳跃。修复候选的一轮动图采样未观察到跳帧，切图仍有超过 100 ms 的样本，完整复测待 Viewer 交付。原始样本见 `target/viewer-performance-*.json`。

### 离线安装和右键迁移续验

- 交付候选为 `.foundation-tools/g1-packages/PicGlance-2.0.0-x64-Setup-Offline.exe`，内置微软签名的 WebView2 Evergreen x64 独立安装程序。安装器在运行时已存在时直接安装 PicGlance，缺失时从包内提取并执行独立安装程序。当前 Windows 11 已有 WebView2，干净系统断网补装待验。
- 命令行会话尝试执行离线安装器，Inno Setup 返回 1，日志显示该会话无法解析 `LocalAppData`。改用交互式向导完成安装，默认应用选项保持未勾选。安装目录的 2.0.0 启动并加载生产界面，原生文件选择器成功打开 800×600 JPEG，压缩对话框返回预览、实际字节显示和质量信息。
- 在同时存在旧版的 Windows 11 机器上，右键 JPEG →「显示更多选项」→「用 PicGlance 打开」启动了 `D:\Program Files\PicGlance\PicGlance.exe`。已将旧通配菜单迁移缺陷交 Desktop owner。修复后需用新版安装包重跑实际菜单、中文文件名冷启动、第二张图转发及卸载恢复。

### 续验的交互证据

使用同一 G1 Release 的便携 EXE（SHA-256 `0eb5779a2bb6ce5462e7c95bf4704255d2b3f8e92a56bf3c454d540c6c8739f8`），在交互式 Windows 11 观察到自绘标题栏的新标识和查看器工具栏。当前用户的 Windows 缩放设置为 96 DPI。通过 `Ctrl+O` 调出原生文件选择器，选中并打开 `01 测试图.jpg`（800×600），按右方向键切至 `02 编辑源.jpg`。`Ctrl+1` 显示 100%，`I` 显示「无元数据」，再次按 `I` 收起，`T` 展开并列出同目录的 PNG 和 JPEG。此轮使用真实按键及 Windows 无障碍树核对状态。连续翻图延迟、帧率、高 DPI 和已安装版的 Shell 图标仍待验证。

本轮复跑 22 项前端测试、工作区 35 项 Rust 测试和 TypeScript 类型检查，均通过。Rust 有 1 项需要旧树的显式样本测试，默认命令按声明跳过。`windows_shell.py --exe target/release/picglance.exe` 再次通过 10 层原生提取比对。

### 续验的交互证据

使用同一 G1 Release 的便携 EXE（SHA-256 `0eb5779a2bb6ce5462e7c95bf4704255d2b3f8e92a56bf3c454d540c6c8739f8`），在交互式 Windows 11 观察到自绘标题栏的新标识和查看器工具栏。当前用户的 Windows 缩放设置为 96 DPI。通过 `Ctrl+O` 调出原生文件选择器，选中并打开 `01 测试图.jpg`（800×600），按右方向键切至 `02 编辑源.jpg`。`Ctrl+1` 显示 100%，`I` 显示「无元数据」，再次按 `I` 收起，`T` 展开并列出同目录的 PNG 和 JPEG。此轮使用真实按键及 Windows 无障碍树核对状态。连续翻图延迟、帧率、高 DPI 和已安装版的 Shell 图标仍待验证。

本轮复跑 22 项前端测试、工作区 35 项 Rust 测试和 TypeScript 类型检查，均通过。Rust 有 1 项需要旧树的显式样本测试，默认命令按声明跳过。`windows_shell.py --exe target/release/picglance.exe` 再次通过 10 层原生提取比对。

生产便携包补跑 4000×3000 样本时，WebView2 调试端点连接被重置（WinError 10054），未采到已确认图片呈现的稳态数据。已将 50 MB 稳态证据缺口、冷启动测量及离线包预算交 Foundation owner 继续协调。

- Desktop 在 Windows 11 25H2 上用 `scripts/test-desktop-production.ps1` 验证生产候选安装、两版覆盖升级、卸载、便携隔离、传统菜单和注册 PNG 处理程序复用同一 PID。离线包在已有 WebView2 的机器上成功部署和卸载。当前环境缺少干净 VM，运行时联网补装、断网补装、默认应用双击和实际 DPI 目视仍待验。候选的哈希和分步结果见[桌面交接](../../openspec/changes/archive/2026-11-14-handoff/handoff-picglance-desktop.md#生产候选复验)。
- Viewer 在 Windows Release 通过 EXIF 面板提交重命名、同名保护和扩展名保留。`windows-smoke.py <exe> --g2` 在独立便携实例采到 7,200×4,800 样本首图 761 ms、主进程及整树峰值 41.6 和 367.8 MiB。20,000×10,000 基线 JPEG 经解码内缩放显示，首图 700 ms、主进程及整树峰值 68.3 和 370.8 MiB。测时从进程创建返回开始。48 MP 以上的渐进 JPEG、PNG、TIFF 范围仍待基线确认，详见[看图交接](../../openspec/changes/archive/2026-11-14-handoff/handoff-picglance-viewer.md#验收反馈复测)。
