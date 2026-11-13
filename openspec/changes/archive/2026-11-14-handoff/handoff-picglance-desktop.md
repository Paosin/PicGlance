# PicGlance Windows 桌面交接

2026-11-14。OpenSpec `picglance-desktop`。按基线交接的 G0 例外继续使用 Tauri，G2 内存指标按原口径验收。

## 实现路径

- `src/features/desktop/` 提供七类设置、中英文、主题回调、更新横幅和现有 `DesktopServices` 的 Tauri 实现。
- `src-tauri/src/desktop/` 管理设置、托盘、关闭、路径转发、每用户注册和日志。安装配置位于 `%APPDATA%\PicGlance\settings.ini`，没有 INI 时尝试读取旧 Qt 的 HKCU 设置。便携设置和 WebView2 数据位于 exe 同目录。便携模式拒绝应用内的文件关联写入。
- `installer/PicGlance.iss` 和 `scripts/package-desktop.ps1` 生成在线引导安装包、包含 Evergreen 独立安装器的离线包，以及带 `portable.marker` 的 ZIP。便携 ZIP 需要主机已有 WebView2。安装器保留关联选择，给新增格式默认勾选并移除废弃格式。默认应用由用户通过 Windows 设置选择。
- 日志保存当前文件和三份轮转文件，阈值为 1 MiB。安装版首选安装目录的 `log`，打开失败时使用 `%LOCALAPPDATA%\PicGlance\log`。日志覆盖设置 I/O、系统集成失败、更新结果和 Rust panic。更新检查经 WinHTTP 访问 `Paosin/PicGlance` 的 latest release，后台检查失败只写日志。

## 给基线 owner 的接线

1. 在 Windows 下声明 `mod desktop`。在创建 Tauri builder 前调用 `desktop::handle_installer_command()`，如返回退出码，直接退出。生成可变 context 后、运行 builder 前调用 `desktop::configure_context(&mut context)`。便携窗口由 `desktop::setup` 使用配置创建，因此应在应用现有的 DWM 窗口初始化之前运行 setup。
2. 将 `desktop::single_instance()` 作为首个 Tauri 插件。保留现有查看器状态和拖放队列。调用 `desktop::setup(app.handle(), Some(Box::new(move |paths| viewer_host::queue_paths(&handle, paths))))`，其中 `handle` 为 `app.handle().clone()`。这条路径统一处理冷启动与转发，替换 Windows 侧的 `viewer_host::queue_startup`，沿用查看器原有可信路径 → 资源 ID 流程。独立测试入口向 setup 传 `None`，使用 `desktop-open-ready` 和 `desktop_take_paths`。
3. 按包内测试入口的 `Cargo.toml` 增加 Windows 依赖和 Win32 features，生产使用 Tauri `tray-icon`、`tauri-plugin-single-instance` 2.5.5、`semver`、`serde_json`、`winreg` 和现有 `windows-sys`。按测试入口 `generate_handler!` 注册生产用的 `desktop_*` 命令，在 `build.rs` 和 main capability 补权限。若生产使用原生路径队列，可省略 `desktop_take_paths` 的注册及前端监听。
4. 生产前端初始化一个 `WindowsDesktop`，先读快照。设置路由挂载 `settingsPage`，用其回调把新快照接回现有主题、locale 和功能状态。监听 `prefers-color-scheme`，按共享的 `resolveTheme` 解析 system。调用 `checkUpdate(true)`，有更新时在主窗口挂载 `updateBanner`。需要使查看器初始缩放、小图放大、编辑默认值、压缩输出格式及目标大小消费该快照。
5. 通过 `app.try_state::<desktop::Desktop>()` 取得日志服务，建议在图片加载失败、超大图提示和压缩结束处调用 `record(event, message)`，记录路径、错误码、耗时和结果。受影响包为 viewer 和 compression，验收需用损坏 JPEG、触发超大图提示的样本和成功及失败的压缩任务检查日志。
6. 用正式 Release exe 运行 `scripts/package-desktop.ps1`。脚本接受 `-Exe`、`-Compiler`、`-WebView2Installer`、`-Version`。离线版另传 `-Offline`。正式安装器沿用 `com.picglance.app-sim` 作为应用互斥量。

## 验证证据

| 命令或操作 | 结果 |
| --- | --- |
| `cargo test --locked --offline --manifest-path src-tauri/src/desktop/tests/Cargo.toml` | 8 项通过，含 5 项桌面测试和 3 项共享契约测试 |
| `cargo test --locked --offline --all-features` | 共享工作区 20 项通过，1 项需要旧版样本的测试默认忽略 |
| `npm test`、`npm run build` | 共享测试 6 项通过，TypeScript 和 Vite 生产构建通过 |
| `npx vite build src/features/desktop/tests --outDir <host>/dist --emptyOutDir`、`cargo build --release --locked --offline --manifest-path <host>/Cargo.toml` | 独立 Windows x64 Release 测试入口构建通过，`<host>` 为 `src-tauri/src/desktop/tests/host` |
| Inno Setup 6.7.4 与微软签名的 WebView2 引导器、x64 独立安装器 | 编译在线安装包、离线安装包和便携 ZIP |
| `node scripts/test-desktop-host.mjs <installed-dir> <portable-dir> <first.png> <second.png>` | Windows 11 25H2、WebView2 157.0.4341.46。设置页切换语言和主题，安装版及便携版同时运行且各自写入 INI。托盘和后台隐藏后复用 PID 恢复，退出模式结束进程。启动检查真实返回 404 并记日志，关闭自动检查后重启无新增记录 |
| 同一脚本调用 Explorer 的 `Shell.Application` 文件动词 | 第一张图片用启动参数打开，带中文和空格的第二张 PNG 经传统动词进入原进程，同一个 WebView 实际解码并显示图片。样本为 TEMP 中两张文件名以 `PicGlance-test-` 开头的 40×30 PNG |
| `scripts/test-desktop-upgrade.ps1 -InstallDirectory <installed-dir> -UpgradeInstaller <2.0.1-setup.exe>` | 测试安装 2.0.0 → 2.0.1 后设置哈希一致，保留已禁用 PNG，新增 SVG，注销模拟退役 AVIF。卸载移除程序、快捷方式、处理程序、默认应用注册和传统菜单，保留用户设置。两版使用同一 Release 测试 exe，变更安装器版本并模拟旧格式清单 |

测试卸载后，测试生成的 `%APPDATA%\PicGlance\settings.ini` 保留在原处。自动审批拒绝将该文件移入工作区，理由是该操作可能导出用户配置并超出本包写入范围。

## G2 待验

- `WINDOWS-DEFERRED` → `picglance-acceptance` 2.1、2.3。生产应用接线后的资源管理器默认双击、Win11 “显示更多选项”交互路径、Win10 菜单、托盘菜单实际交互，以及无 WebView2 的联网补装和断网离线补装。当前机器已有 WebView2，未移除系统运行时。
- `WINDOWS-DEFERRED` → `picglance-acceptance` 2.2、2.4。生产应用的业务日志、实际发现更高 GitHub Release 的横幅、真实升级前后两个版本的 exe，以及正式产物体积。测试入口包只用于复测。

## 生产候选复验

2026-11-14。基线已把桌面入口、原生打开队列、设置与压缩对话框接入生产应用。运行 `npm run build`，随后运行 `cargo build --release --offline --locked --features tauri/custom-protocol`。使用 `TAURI_CONFIG` 中的版本值分别构建 2.0.0 和 2.0.1，两个 EXE 的 PE 版本与内容哈希不同。启用 `tauri/custom-protocol` 后从安装目录启动内嵌前端。最初用普通 Cargo Release 打出的包启动时指向开发地址，已重建并复验。该候选构建有一条 `desktop_take_paths` 未使用警告，该命令供独立测试入口消费。

产物目录为 `installer/build/production/`。安装器和 ZIP 各包含一份 `pngquant.exe`、`gifsicle.exe` 和工具说明。ZIP 同时包含对应版本 EXE 和空的 `portable.marker`。并行修改引入的重复载荷已去掉，复验比较了 ZIP 条目名、数量和提取文件哈希。

| 产物 | 字节数 | SHA-256 |
| --- | ---: | --- |
| `PicGlance.exe` | 18,331,136 | `55d2e611fa96dd6b0832a2f6a443b450e38f2b8f1a0bc8f04ddcc67a5a9c95ed` |
| `PicGlance-2.0.0-x64-Setup.exe` | 8,229,201 | `37440e415fa42826329eda0abd05620055efc18b5e8be542ff73832e7f789078` |
| `PicGlance-2.0.0-x64-Setup-Offline.exe` | 221,541,736 | `c398f9d2465dd72d8954726d20dc948d068d895098116c2fbaebbcdde14e2bec` |
| `PicGlance-2.0.0-x64-Portable.zip` | 6,985,534 | `efabfed612eae9bd0856c9c005fa66471069c3ca20378824f553fe1c82f7e306` |
| `PicGlance-upgrade.exe`（2.0.1） | 18,331,136 | `3ce3251b54069e1e3ad55141a055744dc2f27dfa61542d4b0640b8f77ac583af` |
| `PicGlance-2.0.1-x64-Setup.exe` | 8,226,284 | `3e0170045196aac7d58a43351b6034fd8c62eb38b98db6146251a461b6b15def` |
| `PicGlance-2.0.1-x64-Portable.zip` | 6,985,533 | `c3c656a5dc6a2e6d6d719ac6db654e101e2da9dc33c02d379147f474ee394893` |

`scripts/test-desktop-production.ps1` 在 Windows 11 25H2（26200，WebView2 157.0.4341.46）完成当前用户的安装、运行、升级、卸载与便携验证，测试前确认当前用户没有 PicGlance 安装记录和快捷方式。两种安装器都部署了对应的生产 EXE 和工具，部署哈希与原件一致。在线安装后用 Explorer 的传统文件动词和 Windows Shell 的已注册 PNG 处理程序打开两张带中文和空格的 PNG，原生产 PID 39360 显示了两张图。设置页实时切换主题，便携 ZIP 在独立实例打开图片、写入自己的 INI 并拒绝关联写入。升级到 2.0.1 后安装文件与升级 EXE 哈希一致，设置哈希、PNG 取消选择和差量关联复验通过。卸载后快捷方式、处理程序、菜单、工具及主程序均已清理。离线包在已有运行时的机器完成部署与卸载，安装耗时约 130 秒。

生产 EXE 和三个安装器均通过 `python src-tauri/icons/tests/windows_shell.py --exe <file>`，Win32 原生提取的十个尺寸与新 ICO 像素一致。桌面和开始菜单快捷方式目标均指向此次安装的 EXE，PNG ProgID 的 `DefaultIcon` 指向该 EXE 的图标索引 0。传统菜单使用同一 EXE 的图标，托盘代码使用 Tauri 的 `default_window_icon`。当前用户 Windows 注册的缩放设置是 96 DPI，未做实际 Windows 100%/150%/200% 的资源管理器、运行及固定任务栏、托盘目视矩阵。原生提取只验证图标资源。

`WINDOWS-DEFERRED` → 验收 2.3。当前机器已有 WebView2，未发现可用的 Windows Sandbox 或干净 VM。干净 Windows 的联网补装、断网离线补装、从系统默认应用 UI 选中后双击、真实任务栏及 DPI 矩阵待验。离线候选 221,541,736 字节，超过 30 MB 预算。两个外部压缩工具的精确对应源代码和再分发材料由压缩交接记录为发布前待核查。

## 旧版并存与压缩工具材料复验

2026-11-14。Windows 11 25H2 的用户级旧版通配菜单与新版按扩展名注册的菜单曾并存。现由新版安装命令对已有旧通配菜单写入 `ProgrammaticAccessOnly`，保留旧版命令和 ProgID。卸载只撤销本次迁移所写的隐藏值。新版在独立通配菜单键中用 `AppliesTo` 限制为支持的图片格式，同时清理之前的按扩展名菜单。关联变更通过 `SHChangeNotify` 通知 Shell。JPEG 的 `UserChoice` ProgID 和 Hash 在覆盖安装前后相同。

最终离线安装包覆盖安装通过。新的 Shell 枚举对中文空格 JPEG 只返回一个 `Open with PicGlance`，对日志文件无 PicGlance 项。调用 JPEG 动词后出现新版可见窗口（PID 41448，`%LOCALAPPDATA%\Programs\PicGlance\PicGlance.exe`）。Explorer 图形界面的「显示更多选项」在重启后仍未显示此菜单，图形点击验收未通过。测试中临时移除过滤条件的注册表比较已恢复。桌面 Rust 9 项、npm 6 项及 OpenSpec 严格校验通过。

在线、离线与便携候选均包含原始 pngquant COPYRIGHT/README、Gifsicle COPYING、各自发布的版本源码归档和来源说明。已比较最终 ZIP 和离线安装目录中六份材料的字节或哈希。发布方的精确二进制构建参数及依赖解析仍缺证据，对外发布就绪待确认。包体积按用户要求标记待定。

| 最终候选（`installer/build/production/legacy-menu-fix/`） | 字节数 | SHA-256 |
| --- | ---: | --- |
| `PicGlance-2.0.0-x64-Setup-Offline.exe` | 222,274,278 | `a8c65b8069561ee17a268d4e8135fbbc7c90c207059204b89f1e367309cb276d` |
| `PicGlance-2.0.0-x64-Setup.exe` | 8,961,497 | `3c4edb1112f7424fe0c6cadeef72af5dbf8132309d56d78dc2ccb8a8ebcd0382` |
| `PicGlance-2.0.0-x64-Portable.zip` | 7,724,206 | `91a8b206d25b203cd75b21932cbf5e7cb713d45d18eef790c4abc898bcf4e475` |
