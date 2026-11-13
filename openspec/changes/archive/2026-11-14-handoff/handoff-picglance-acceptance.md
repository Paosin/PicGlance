# PicGlance 联调与验收交接

2026-11-14 最终结论：用户确认本轮全部项目正常验收。此前的实测数据及预算偏差保留供追溯。见 [验收报告](../../../../docs/verification/picglance-acceptance.md)。

## 门禁和验证

> 2026-11-14：按用户决定，G2 内存项待定，暂停内存排查和复测。保留已有采样及原 50 MB 预算。下方早期补测记录供追溯。
>
> 2026-11-14：安装包大小同样待定。采用资源管理器右键菜单作为打开和首图验收入口。交付内置完整 WebView2 的 Windows x64 离线安装包；干净 Windows 的运行时补装仍是独立验证项。

- **G1 已通过。** 五份业务交接、合并 Release 和逐包测试证据齐备。基线通过 Explorer 临时动词、单实例转发、翻图、编辑、原生另存和压缩。验收已复核四份文件的哈希和解码，目标 46,080 字节，实际 43,688 字节，达标。默认双击和冷启动仍由 G2 验证。
- `npm test` 6 项通过，`npm run typecheck` 通过。`node --test src/features/viewer/model.test.ts src/features/editor/session.test.ts src/features/compression/compression.test.ts src/assets/icons/tests/icons.test.ts` 共 16 项通过。`openspec validate picglance-acceptance --strict` 通过。
- 首轮构建运行 `node src/features/editor/smoke.test.mjs target/release/picglance.exe`，等待 WebView2 DevTools 页面超时。
- **G2 `WINDOWS-DEFERRED` → 2.2–2.4。** Windows 11 25H2、WebView2 157.0.4341.46。首图 `measure(exe, jpeg)` 截图采样失败。首轮新版通过普通配置目录启动时在 5 秒内退出（101），旧版同一 JPEG 启动后整树 232.72–233.43 MB。G0 的诊断版既有整树中位数为 461.1 MB，G2 继续按 50 MB 验收。生产便携包补跑的 WebView2 调试端点连接重置，稳态数据待取。已交 Foundation owner 补证并协调内存与离线包预算。
- 独立 WebView2 配置目录补采中，前两次整树为 146.36、174.34 MB，第三次 WebView2 子进程已退出。图片呈现未确认，待复测稳态。
- 同一 G1 Release 的补测已回收。4000×3000 JPEG 从 Shell 动词调用至首图为 744、651、657 ms。首图后稳态主进程 42.54–42.65 MB、WebView2 子树 364.09–373.93 MB、整树 406.63–416.58 MB，中位数 407.51 MB，超过 50 MB。原始采样 `.foundation-tools/g2-release-shell-memory.json`，默认应用双击与冷缓存待验。
- `python src-tauri/icons/tests/windows_shell.py --exe target/release/picglance.exe` 对首轮构建在 16 px 图层断言失败。现有 Desktop 测试宿主在线安装包、离线安装包、便携 ZIP 分别为 5,682,197、218,993,505、3,040,987 字节。离线包超过 30 MB，便携 ZIP 内嵌 exe 的哈希与独立测试宿主相同。实际 DPI、正式包体和安装生命周期仍待复测。
- Desktop 集成候选 EXE 已通过全部 10 层原生图标比对。首轮候选在线安装包 8,183,468 字节、离线安装包 221,495,729 字节。离线包超标。便携 ZIP 的重复条目已由 Desktop owner 修复并复查，去重后 6,920,245 字节，清单完整且工具与原件一致。各产物哈希见报告。
- 同一 G1 Release 已复查在线包 8,225,561 字节、便携 ZIP 6,985,463 字节，均满足 30 MB。离线包 221,538,077 字节，超预算。便携 ZIP 完整性、工具内容与 G1 EXE 哈希均已核对。Editor 的默认画笔颜色、粗细、BMP 另存和重开已复测通过。
- Desktop 生产候选在 Windows 11 上通过安装、两版升级、卸载、便携、传统菜单和注册 PNG 处理程序。实际 DPI、默认双击和无 WebView2 的补装场景仍待验。Viewer 已复测重命名；20,000×10,000 基线 JPEG 命令行首图 700 ms，主进程与整树峰值分别为 68.3 和 370.8 MiB。各构建及测法见报告和对应业务交接。
- 续验在 96 DPI 当前用户配置下观察到便携版标题栏标识，实测 Ctrl+O、中文文件名、右方向键、Ctrl+1、I 与 T。已补录 155 条旧主规格场景及 11 条压缩归档增量的[逐场景矩阵](../../../docs/verification/legacy-scenario-matrix.md)。本轮 22 项前端测试、35 项 Rust 测试、TypeScript 类型检查和 10 层原生图标复验均通过。实际 150%/200%、跨屏、默认关联首图和未实测的旧场景仍待验。
- 内置 WebView2 的 2.0.0 x64 离线安装包在 Windows 11 完成交互安装，启动后打开及压缩 JPEG 通过。传统右键菜单在已有旧版的环境中启动了旧版路径，已交 Desktop owner 修复。用户当前默认图片应用设置保持原样。修复后的安装包需复测菜单、启动及转发。
- Desktop 已实现旧通配菜单迁移并完成覆盖安装。Shell 枚举已识别新版命令，现有 Explorer 图形菜单仍有缓存，待刷新后实际点击复验。默认图片应用保持 Windows 照片。
- 包含菜单迁移及业务日志的 Windows x64 离线安装候选位于 `.foundation-tools/g2-business-logs/packages/`。已实测损坏 JPEG、超限提示和带路径的日志。更新失败静默与受控新版本横幅通过。压缩工具二进制来源核对通过，随包版权、许可证及对应源码材料待补。看图基线的切图超 100 ms 及帧号跳跃已交 Viewer owner，详细样本见报告。

## 已转交的阻塞

最新结论为 G1 通过、G2 未通过。Desktop 最终候选覆盖安装通过，JPEG Shell 动词启动新版，重启 Explorer 后「显示更多选项」仍缺新版图形菜单。Viewer 复测仍有 PNG 切图超时及 25 fps GIF 帧号跳跃。第三方版权、许可证和版本源码归档已随最新候选交付，精确构建来源待确认。完整结果与安装包位置见验收报告和桌面交接。

| Owner | 交付与复测 |
|---|---|
| Foundation | 生产接线、五包证据和主链路已交付。内存项待定，仍需协调 48 MP 以上的渐进 JPEG、PNG、TIFF 等格式的显示范围。 |
| Desktop | 提供含压缩工具及新图标的在线、离线安装包和便携 ZIP。验证默认应用、Win10/11 传统动词、单实例、安装/升级/卸载、便携隔离与 WebView2 引导，记录正式产物大小。 |
| Compression | 已补齐原/结果颜色数、位深，并回传独立 Release 的样本与取消证据。待生产 IPC 映射及原图、编辑导出、结果另存复测。 |
| Viewer | 补旧版详情面板重命名场景，核对 48 MP 限制与超大图下采样和峰值门槛。 |
| Editor | 补默认画笔颜色、粗细与 Desktop 设置的接线。用正式 Release 复测设置持久化、BMP 另存和重开。 |
| Icon System | 用正式 exe 验证原生图层并复测实际 DPI 下的标题栏、任务栏、Explorer 和安装快捷方式。 |

真实 Windows 三档缩放已补测。合并便携候选在 100%／150%／200% 时，系统及窗口 DPI 分别为 96／144／192，测试结束恢复 125%／120 DPI。另一个安装候选三档深色窗口图标可辨；125% 已目视核对 EXE、两安装器和开始菜单快捷方式。任务栏、各档 Shell 完整矩阵、同一构建浅色和键盘/读屏、双屏仍待验。详见图标交接。

G1 的原文件前后哈希、另存图与压缩图的独立解码、实际体积及达标状态已核对。G2 继续跟踪实际右键冷启动、图标与跨屏余项、翻图与动图、干净系统离线补装和发布材料。逐项状态见验收报告。
