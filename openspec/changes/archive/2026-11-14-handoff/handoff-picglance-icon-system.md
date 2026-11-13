# PicGlance 图标系统交接

2026-11-14。变更 `picglance-icon-system`，主工作区 `dev`。基线交接已记录 G0 放行及 Tauri 内存例外。G2 性能仍按原预算验收。

## 交付

- `src/assets/icons/index.ts` 提供看图、编辑、压缩、设置和窗口控件的轮廓图标。采用 20 px 画布、1.5 px 圆端描边和 `currentColor`。业务图标按钮的设计点击区为 36 × 36 CSS px。
- `fit` 用四角框围住画面，`actual` 使用 1:1。旋转的箭头方向相反，翻转以垂直和水平轴区分。交接中的「表态」按控件选中态验证，涵盖适应窗口、缩略图、详情、全屏和画笔。工作区业务规格没有定义额外的表态动作。
- `createIcon(translate)` 返回兼容现有 `Icon` 签名的 SVG 渲染函数。带 `label` 时生成本地化 `aria-label`，省略时生成装饰 SVG。按钮名称、选中态、提示和快捷键语义由调用控件设置。
- `src-tauri/icons/picglance.svg` 是蓝底、白色取景框和琥珀色圆点的新标识。`icon.ico` 包含 16、20、24、32、40、48、64、96、128、256 px 的透明图层。`generate_ico.py` 用本地 Edge 和 Pillow 从 SVG 生成 ICO。

## 复测

| 命令 | 结果 |
| --- | --- |
| `node --test src/assets/icons/tests/icons.test.ts` | 3 项通过，覆盖现有契约、装饰与独立图标名称、快捷键分发一致性 |
| `node src/assets/icons/tests/browser.mjs` | Edge 157.0.4341.46 的 12 组夹具通过（深浅主题 × 1/1.5/2 DPR × 中英文）。每组 12 个轮廓的像素形状均不同。图形为 20/30/40 设备像素，按钮为 36 × 36 CSS px。深色最低对比度 6.44:1，浅色 6.04:1。无障碍树识别按钮名称及选中态，快捷键提示与分发匹配 |
| `python src-tauri/icons/generate_ico.py` 和 `python src-tauri/icons/tests/test_ico.py` | 生成成功，10 帧尺寸、透明边缘及高亮轮廓检查通过 |
| `npm test` 和 `npm run build` | 基线 6 项通过，TypeScript 检查和 Vite 生产构建通过。构建期间曾遇到并行业务测试导入的 `compression.ts` 尚未出现，文件到位后重跑通过 |
| `cargo test --locked --offline --all-features` | 21 项通过，1 项因依赖旧版图片树按测试声明忽略 |
| `cargo build --release --locked --offline` | Windows x64 Release 成功。用临时环境变量 `TAURI_CONFIG={"bundle":{"icon":["icons/icon.ico"]}}` 和 `CARGO_TARGET_DIR=target/release` 测试新 ICO |
| `python src-tauri/icons/tests/windows_shell.py --exe target/release/release/picglance.exe` | Windows 从 ICO 和测试版 EXE 原生提取 10 个尺寸，逐帧 RGB 像素一致 |
| `python src/shared/tests/measure_first_image.py --tauri target/release/release/picglance.exe --smoke` | Win32 检测到测试版可见窗口 |
| `openspec validate picglance-icon-system --strict` | 通过 |

## Windows 状态

Windows 11 构建 26200，系统实际缩放 125%（120 DPI）。Edge 的 100%、150%、200% 是设备像素比模拟。Win32 原生图标提取检查覆盖大小图层，桌面截图接口返回 `screen grab failed`。真实任务栏、资源管理器和自绘标题栏的目视矩阵待验。快捷键和无障碍证据来自本包夹具，业务界面集成后的悬停、焦点与读屏复核待验。

`WINDOWS-DEFERRED` → `picglance-acceptance` 2.3。在基线接线后的 Release 上，按 Windows 实际 100%/150%/200% 缩放分别检查深浅主题、各方向旋转和翻转、状态反馈、中英名称、快捷键与工具提示。检查自绘标题栏、运行及固定任务栏图标、资源管理器 EXE 和安装后快捷方式。多屏时另查跨屏切换。对应本包 2.2、2.3 保持待验。

## 基线协调请求

1. 由基线 owner 把 `src/shared/icons.ts` 接到本包 `createIcon(translate)`，让 shell 和 viewer 等现有调用方切换到新图标。接线时可用兼容包装保留现有 `icon(name, size)` 调用。建议经基线评审后让公共 `IconName` 纳入 `AssetIconName`，保留 `rotate`、`flip` 作为兼容名称。各业务 owner 选择对应图标，并在图标按钮采用 20 px/36 px 规格。
2. 请基线 owner 协调业务 owner 补齐独立旋转与翻转按钮的文案和提示。公共 `Message` 可添加 `rotateLeftTip`（Shift+R）和 `flipVerticalTip`（V）。交互控件应保持名称、`title` 和 `aria-keyshortcuts` 与实际键位一致。复测现有 `npm test`、本包两项前端测试和实际界面的中英键盘矩阵。
3. 由基线 owner 把 `src-tauri/tauri.conf.json` 的 `bundle.icon` 切到 `icons/icon.ico`，在自绘标题栏接入 `picglance.svg`。desktop owner 同步引用新 ICO 检查托盘、快捷方式和关联图标。重建正式 Release 和安装包，执行 G2 2.3。

## 验收转交复核

2026-11-14。收到验收转交后复核，`bundle.icon` 已指向 `icons/icon.ico`。当时的 `target/release/picglance.exe` 用 `python src-tauri/icons/tests/windows_shell.py --exe target/release/picglance.exe` 仍在 16 px 报像素差异。生产接线、正式重建与同一产物的 Windows 复验已转交 foundation 和 desktop 对话。复验须记录正式 EXE 的路径和 SHA-256，并将其用于原生提取与实际 DPI 的标题栏、任务栏、快捷方式和资源管理器检查。

## 真实 Windows DPI 补测

2026-11-14，在 Windows 11 26200 的交互会话通过「设置 → 系统 → 屏幕」切换实际缩放。用 `python src-tauri/icons/tests/windows_dpi.py --pid <PID>` 对同一个运行中的便携候选主窗口读取 Win32 DPI。该 EXE 来自 `.foundation-tools/g2-business-logs/packages/PicGlance-2.0.0-x64-Portable.zip`，与本轮的 `target/release/picglance.exe` SHA-256 一致，均为 `89c87ca6d11f0f2ab8cba919d83173f881ba2d971234ef4c9c4cf95952967bb0`。

| 实际 Windows 缩放 | Win32 系统 DPI | 便携候选主窗口 DPI |
| --- | ---: | ---: |
| 100% | 96 | 96 |
| 150% | 144 | 144 |
| 200% | 192 | 192 |
| 恢复原 125% | 120 | 120 |

- `windows_shell.py` 对合并候选 Release、便携 ZIP 中的同哈希 EXE、合并候选离线安装器及桌面组后续 `installer/build/production/legacy-menu-fix/PicGlance.exe` 原生提取均通过 10 层对比。桌面组后续 EXE 的 SHA-256 为 `e28a4186c9902120c8521040ad1e4987d5fd0b38f5da9de83225ba5335086552`。
- 在交互桌面系统设置分别处于 100%、150%、200% 时，前台目视检查桌面组已安装版本的深色看图窗口。自绘标题栏照片标识和缩放、适应窗口、1:1、旋转、翻转、缩略图、信息与全屏图形可辨。这个安装进程的 HWND 在 Win32 测试进程中不可访问，三档 HWND 数值来自上述便携候选。仍待在同一个可追溯构建上汇合目视、Win32 DPI 和浅色及键盘/读屏证据。
- 恢复至 125% 后在资源管理器详情视图目视确认桌面组后续候选 EXE、在线与离线安装器、开始菜单程序目录下的 PicGlance 安装快捷方式显示新蓝底照片标识。离线安装日志确认创建桌面与开始菜单两份快捷方式。其他档位的 Shell 目视矩阵未完成。
- `WINDOWS-DEFERRED` → `picglance-acceptance` 2.3。窗口捕获未提供任务栏画面，运行和固定任务栏辨识度仍待交互桌面目视验收。Win32 只报告 1 台显示器，跨屏验证待具备第二台显示器。本包 2.2、2.3 保持待验。
