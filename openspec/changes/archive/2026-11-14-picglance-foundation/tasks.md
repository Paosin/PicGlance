## 1. 实施前检查

- [x] 1.1 核对主工作区位于新版 dev，旧版提交 3340f6c 保存在 legacy/v1，复查已确认的独占范围及 G0 判据。

独占写入范围为 Cargo.toml、Cargo.lock、package.json、前端构建配置和锁文件、应用入口、src/shared/、src-tauri/Cargo.toml、src-tauri/tauri.conf.json、src-tauri/capabilities/、src-tauri/src/contracts/、CI，以及本包测试、tasks.md 和 `openspec/changes/handoff/handoff-picglance-foundation.md`。共享文件和其他 owner 的路径由对应 owner 修改。缺端口向基线 owner 提请求。

只有完整实现并通过对应验证才勾选。真实环境暂不可验时以 `WINDOWS-DEFERRED` 标注并指向 G2 的具体任务。实施范围变更在本文件顶部以引用块保留日期与内容，原任务标记「范围作废，非代码实现」。

## 2. 实施与验证

- [x] 2.1 搭建可启动的 Windows 空壳与发布构建。
- [x] 2.2 提供主题、语言、图标和窗口测试夹具，验证 TypeScript 与 Rust 契约。
- [x] 2.3 用临时诊断入口和代表性 JPEG 对照旧版测首图与整个进程树内存，记录 Tauri 是否达到目标及是否需回到决策。
- [x] 2.4 验证 G0 并发布交接。
- [x] 2.5 写入 `openspec/changes/handoff/handoff-picglance-foundation.md`，记录改动路径、测试命令与结果、真实环境验证、阻塞和下游要求。
