## Why

PicGlance 需要在新版界面中迁移完整的旧版能力。目标体积压缩负责在 G0 通过后与其他业务包并行实施。

## What Changes

- 迁移静态和动图目标大小、格式默认、质量/尺寸迭代、取消和结果对比。

## Capabilities

### New Capabilities

- `image-compression`: 目标体积压缩的对应行为和验收约束。
- `compress-format-lock`: 目标体积压缩的对应行为和验收约束。
- `compress-strategy-pattern`: 目标体积压缩的对应行为和验收约束。

### Modified Capabilities

## Impact

- 独占写入范围为 src/features/compression/、src-tauri/src/compression/、tools/、压缩专用第三方依赖目录，以及本 change 的测试、任务进度和交接文件。
- 旧版工作树 `../PicGlance-tmp` 用于规格、实现和样本对照。旧版历史参照为 `legacy/v1` 上的 `3340f6c`，新版从独立的 `dev` 工作树起步。
