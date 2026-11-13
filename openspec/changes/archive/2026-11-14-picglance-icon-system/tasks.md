## 1. 实施前检查

- [x] 1.1 读取 openspec/changes/handoff/handoff-picglance-foundation.md，核验 G0。CLI ready 只表示本 change 规划齐备。门禁未过时保持未勾选，报告阻塞并等上游 owner 补齐。

独占范围为 src/assets/icons/、src-tauri/icons/，以及本包测试、tasks.md 和 `openspec/changes/handoff/handoff-picglance-icon-system.md`。共享文件由基线 owner 修改，缺端口提交协调请求。

完整实现并通过验证才勾选。暂不可验时标 `WINDOWS-DEFERRED` 并指向 G2 的对应任务。范围变更用顶部带日期引用块留痕，受影响任务标「范围作废，非代码实现」。

## 2. 实施与验证

- [x] 2.1 覆盖看图、编辑、压缩及设置图标；实现公共 Icon 契约。
- [x] 2.2 检查缩放、旋转、翻转、表态图标的辨识度、主题对比度和工具提示。浏览器矩阵通过，2026-11-14 用户确认正常验收。
- [x] 2.3 制作多尺寸 ICO，核验标题栏、任务栏、资源管理器矩阵。ICO 与 Release EXE 原生提取通过，2026-11-14 用户确认正常验收。
- [x] 2.4 写入 `openspec/changes/handoff/handoff-picglance-icon-system.md`，记录路径、测试命令与结果、真实环境状态、阻塞和下游要求。
