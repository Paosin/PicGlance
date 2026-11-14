## Why

PicGlance 需要在新版界面中迁移完整的旧版能力。轻编辑负责在 G0 通过后与其他业务包并行实施。

## What Changes

- 迁移画笔、形状、裁剪、六种马赛克、变换、撤销重做和保存。

## Capabilities

### New Capabilities

- `image-editing`: 轻编辑的对应行为和验收约束。

### Modified Capabilities

## Impact

- 独占写入范围为 src/features/editor/、src-tauri/src/editor/，以及本 change 的测试、任务进度和交接文件。
- 旧版工作树 `../PicGlance-tmp` 用于规格、实现和样本对照。旧版历史参照为 `legacy/v1` 上的 `3340f6c`，新版从独立的 `dev` 工作树起步。
