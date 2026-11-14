## 1. 实施前检查

- [x] 1.1 读取 openspec/changes/handoff/handoff-picglance-foundation.md 与 G0 记录。CLI ready 只检查本 change 的规划文件。门禁缺失时报告阻塞，保留未勾选状态，由基线 owner 补齐。

独占写入范围为 src/features/compression/、src-tauri/src/compression/、tools/、压缩专用第三方依赖目录，以及本包测试、tasks.md 和 `openspec/changes/handoff/handoff-picglance-compression.md`。共享文件和其他 owner 的路径由对应 owner 修改。缺端口向基线 owner 提请求。

只有完整实现并通过对应验证才勾选。真实环境暂不可验时以 `WINDOWS-DEFERRED` 标注并指向 G2 的具体任务。实施范围变更在本文件顶部以引用块保留日期与内容，原任务标记「范围作废，非代码实现」。

## 2. 实施与验证

- [x] 2.1 实现质量和尺寸搜索，处理尺寸锁定和无法达标的结果。
- [x] 2.2 迁移 MozJPEG、pngquant、gifsicle，选择支持有损与动画的 WebP 编码适配。
- [x] 2.3 验证 GIF/WebP 的帧时长、透明度、取消、清理和输出格式。
- [x] 2.4 用真实图片验证前后参数、质量及目标体积，记录 GIF 临时 I/O 的处理。
- [x] 2.5 写入 `openspec/changes/handoff/handoff-picglance-compression.md`，记录改动路径、测试命令与结果、真实环境验证、阻塞和下游要求。
