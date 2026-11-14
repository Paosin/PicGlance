## Why

PicGlance 需要在新版界面中迁移完整的旧版能力。工程基线负责先建立所有工作包依赖的工程基线。

## What Changes

- 建立 Rust + Tauri Windows 工程和现代照片优先的可点击看图原型。统一应用外壳、路由、快捷键分发、主题变量和中英文基础。

## Capabilities

### New Capabilities

- `app-shell-ui`: 工程基线的对应行为和验收约束。
- `localization`: 工程基线的对应行为和验收约束。

### Modified Capabilities

## Impact

- 独占写入范围为 Cargo.toml、Cargo.lock、package.json、前端构建配置和锁文件、应用入口、src/shared/、src-tauri/Cargo.toml、src-tauri/tauri.conf.json、src-tauri/capabilities/、src-tauri/src/contracts/、CI，以及本 change 的测试、任务进度和交接文件。
- 旧版工作树 `../PicGlance-tmp` 用于规格、实现和样本对照。旧版历史参照为 `legacy/v1` 上的 `3340f6c`，新版从独立的 `dev` 工作树起步。
