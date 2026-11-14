## 1. 实施前检查

- [x] 1.1 读取 openspec/changes/handoff/handoff-picglance-foundation.md 与 G0 记录。CLI ready 只检查本 change 的规划文件。门禁缺失时报告阻塞，保留未勾选状态，由基线 owner 补齐。

独占写入范围为 src/features/viewer/、src-tauri/src/viewer/，以及本包测试、tasks.md 和 `openspec/changes/handoff/handoff-picglance-viewer.md`。共享文件和其他 owner 的路径由对应 owner 修改。缺端口向基线 owner 提请求。

只有完整实现并通过对应验证才勾选。真实环境暂不可验时以 `WINDOWS-DEFERRED` 标注并指向 G2 的具体任务。实施范围变更在本文件顶部以引用块保留日期与内容，原任务标记「范围作废，非代码实现」。

## 2. 实施与验证

- [x] 2.1 接入打开入口、格式探测、大文件限制和后台解码。
- [x] 2.2 实现目录排序导航、按需缩略图、有界缓存与过期请求取消。
- [x] 2.3 实现缩放、翻转、旋转、全屏和状态栏，验证预览不修改像素。
- [x] 2.4 验证三类动画的帧时间、循环，剪贴板和 EXIF/GPS 边界。
- [x] 2.5 写入 `openspec/changes/handoff/handoff-picglance-viewer.md`，记录改动路径、测试命令与结果、真实环境验证、阻塞和下游要求。

`WINDOWS-DEFERRED`：真实资源管理器拖放、关联、单实例和另存输出留给 `picglance-acceptance` 2.1–2.3。大图峰值、冷启动、翻图与动画性能留给其 2.4。

## 3. 验收反馈补齐

- [x] 3.1 恢复旧版 EXIF 面板重命名，验证同名冲突、扩展名、目录排序与当前选中。
- [x] 3.2 对 20,000×10,000 基线 JPEG 在解码时降采样，测量 Windows Release 峰值并交接其余格式的尺寸边界。
