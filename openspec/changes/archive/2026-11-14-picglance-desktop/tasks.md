## 1. 实施前检查

- [x] 1.1 读取 openspec/changes/handoff/handoff-picglance-foundation.md，核验 G0。CLI ready 只表示本 change 规划齐备。门禁未过时保持未勾选，报告阻塞并等上游 owner 补齐。

独占范围为 src/features/desktop/、src-tauri/src/desktop/、installer/、scripts/，以及本包测试、tasks.md 和 `openspec/changes/handoff/handoff-picglance-desktop.md`。共享文件由基线 owner 修改，缺端口提交协调请求。

完整实现并通过验证才勾选。暂不可验时标 `WINDOWS-DEFERRED` 并指向 G2 的对应任务。范围变更用顶部带日期引用块留痕，受影响任务标「范围作废，非代码实现」。

## 2. 实施与验证

- [x] 2.1 完成设置页、三种关闭行为及主题/语言接线，验证便携 INI 与安装版配置隔离。2026-11-14 用户确认正常验收。
- [x] 2.2 完成单实例参数转发和传统右键菜单，验证带空格与中文路径。
  - 生产候选已通过 Explorer 传统动词、注册处理程序及同一进程显示中文空格路径的验证。
- [x] 2.3 安装时注册资源管理器右键入口，移除文件关联选择和目录外快捷方式选项，限制卸载的文件删除范围。
- [x] 2.4 打包内置 WebView2 的 x64 离线安装版和便携 ZIP，验收升级保留数据、日志轮转及 GitHub Releases 提示。2026-11-14 用户确认正常验收。
- [x] 2.5 写入 `openspec/changes/handoff/handoff-picglance-desktop.md`，记录路径、测试命令与结果、真实环境状态、阻塞和下游要求。
