## Why

PicGlance 需要在新版界面中迁移完整的旧版能力。图片浏览负责在 G0 通过后与其他业务包并行实施。

## What Changes

- 覆盖静态与 GIF/APNG/动图 WebP、同目录翻图、缩略图、缩放、预览变换、全屏、拖放、剪贴板、EXIF/GPS。

## Capabilities

### New Capabilities

- `image-viewing`: 图片浏览的对应行为和验收约束。
- `image-info`: 图片浏览的对应行为和验收约束。

### Modified Capabilities

## Impact

- 独占写入范围为 src/features/viewer/、src-tauri/src/viewer/，以及本 change 的测试、任务进度和交接文件。
- 旧版工作树 `../PicGlance-tmp` 用于规格、实现和样本对照。旧版历史参照为 `legacy/v1` 上的 `3340f6c`，新版从独立的 `dev` 工作树起步。
