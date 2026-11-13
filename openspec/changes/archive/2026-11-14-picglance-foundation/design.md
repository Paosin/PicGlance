## Context

旧版是 Qt 6/QML Windows 图片工具。新版采用照片优先的视觉和统一图标，迁移完整功能。建立 Rust + Tauri Windows 工程和现代照片优先的可点击看图原型。统一应用外壳、路由、快捷键分发、主题变量和中英文基础。

## Goals / Non-Goals

**Goals**

- 建立 Rust + Tauri Windows 工程和现代照片优先的可点击看图原型。统一应用外壳、路由、快捷键分发、主题变量和中英文基础。

**Non-Goals**

- 本包范围限于 工程基线。

## Decisions

- 搭建可启动的 Windows 空壳与发布构建。
- 提供主题、语言、图标和窗口测试夹具，验证 TypeScript 与 Rust 契约。
- 用临时诊断入口和代表性 JPEG 对照旧版测首图与整个进程树内存，记录 Tauri 是否达到目标及是否需回到决策。
- 验证 G0 并发布交接。

默认深石墨灰，提供浅色与系统模式。看图顶部显示文件名和文字入口，画布居中，底部承载缩放与状态。编辑工具参数跟随当前工具，压缩结果并列对比。图像数据经受控二进制资源传递。旧版纯图标密排方案会降低常用动作辨识度，因此采用分组和文字入口。保留原生 Windows UI 作为性能门禁失败时的候选。

## Risks / Trade-offs

- WebView2 启动、内存和离线分发可能突破旧版预算。用 Windows Release 样本记录结果，保留失败证据和决策。

## 总设计与多对话执行指南

```text
picglance-foundation -- G0 --> viewer | editor | compression | desktop | icon-system
五个业务包 -- G1 --> picglance-acceptance -- G2 --> 发布验收
```
- G0（基线 owner）：Windows Release 空壳启动成功，Rust 与 TypeScript 契约检查通过，首图原型测量记录齐备。若 Tauri 预算未达标，协调对话记录决策后再冻结平台契约。
- G1（基线汇总、验收 owner 核验）：五份交接与逐包构建、测试通过的证据齐备。合并构建成功并跑通打开、编辑、另存、目标体积压缩。
- G2（验收 owner）：完整功能矩阵、DPI、安装升级卸载、性能与体积均有真实 Windows 证据，未通过项明确报告。

| Change | 能力 | 前置 | 独占写入 |
|---|---|---|---|
| picglance-foundation | 工程基线 | 新版 dev 工作树已准备，旧版历史位于 legacy/v1 | Cargo.toml、Cargo.lock、package.json、前端构建配置和锁文件、应用入口、src/shared/、src-tauri/Cargo.toml、src-tauri/tauri.conf.json、src-tauri/capabilities/、src-tauri/src/contracts/、CI |
| picglance-viewer | 图片浏览 | G0 | src/features/viewer/、src-tauri/src/viewer/ |
| picglance-editor | 轻编辑 | G0 | src/features/editor/、src-tauri/src/editor/ |
| picglance-compression | 目标体积压缩 | G0 | src/features/compression/、src-tauri/src/compression/、tools/、压缩专用第三方依赖目录 |
| picglance-desktop | Windows 集成与设置 | G0 | src/features/desktop/、src-tauri/src/desktop/、installer/、scripts/ |
| picglance-icon-system | 图标和应用标识 | G0 | src/assets/icons/、src-tauri/icons/ |
| picglance-acceptance | 联调与验收 | G1 | tests/acceptance/、docs/verification/ |

子模块专用测试放在各自目录内。根构建、应用启动接线、共享 registry 和公共契约只能由基线 owner 修改。各 change 只写自己的规划目录与交接文件。每份 change 同一时间由一个写入对话执行。

| Port | 实现 owner | 能力 | 消费者 |
|---|---|---|---|
| OpenImage / ViewerState | viewer | 打开路径、选图和浏览状态 | foundation、desktop、viewer UI |
| ImageDocument | viewer | 源路径、尺寸、帧和受控图像数据 | editor、compression |
| EditSession / ExportImage | editor | 会话、保存状态及导出图像 | foundation、compression |
| CompressionJob | compression | 参数、进度、取消、结果 | foundation、压缩界面 |
| DesktopServices | desktop | 系统打开、持久化配置、主题语言及托盘 | foundation、viewer |
| Icon | icon-system | 稳定名称、尺寸、主题色与可访问名称 | 所有 UI 包 |
| ThemeTokens / Locale | foundation | 色彩、间距、控件状态及文案键约定 | 所有 UI 包 |

由基线 owner 在公共目录定义契约签名、错误类型及调用约定，业务状态由对应实现 owner 持有，消费者只经契约访问。依赖未就绪时用契约测试替身验证本包行为。提交公共变更请求时写明缺失能力、调用方、建议签名、受影响包和测试，由基线 owner 更新并重新验证。

交接统一写入 `openspec/changes/handoff/handoff-<change>.md`，记录改动路径、测试命令和结果、真实 Windows 验证状态、未解阻塞和下游集成要求。
