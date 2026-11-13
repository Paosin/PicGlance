## 1. 实施前检查

- [x] 1.1 读取 openspec/changes/handoff/handoff-picglance-foundation.md 与 G0 记录。CLI ready 只检查本 change 的规划文件。门禁缺失时报告阻塞，保留未勾选状态，由基线 owner 补齐。

独占写入范围为 src/features/editor/、src-tauri/src/editor/，以及本包测试、tasks.md 和 `openspec/changes/handoff/handoff-picglance-editor.md`。共享文件和其他 owner 的路径由对应 owner 修改。缺端口向基线 owner 提请求。

只有完整实现并通过对应验证才勾选。真实环境暂不可验时以 `WINDOWS-DEFERRED` 标注并指向 G2 的具体任务。实施范围变更在本文件顶部以引用块保留日期与内容，原任务标记「范围作废，非代码实现」。

## 2. 实施与验证

- [x] 2.1 实现编辑会话和原图坐标映射，处理编辑区缩放与平移。
- [x] 2.2 完成画笔、形状、裁剪、六种马赛克、像素变换和调整尺寸。
- [x] 2.3 验证撤销重做、有损和透明图保存、覆盖确认、另存为及未保存提示。
- [x] 2.4 验证 SVG 等受限输入的可用操作和导出约束。
- [x] 2.5 写入 `openspec/changes/handoff/handoff-picglance-editor.md`，记录改动路径、测试命令与结果、真实环境验证、阻塞和下游要求。
