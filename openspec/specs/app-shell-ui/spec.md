# app-shell-ui Specification

## Purpose
TBD - created by archiving change picglance-foundation. Update Purpose after archive.

## Requirements

### Requirement: 主题系统
系统 SHALL 提供深色、浅色两种主题，并 SHALL 支持三种模式选择：深色、浅色、跟随系统；默认深色。主题切换 SHALL 即时生效无需重启。

#### Scenario: 切换主题即时生效
- **WHEN** 用户在设置中将主题从深色切换为浅色
- **THEN** 界面立即以浅色主题渲染，无需重启

#### Scenario: 跟随系统
- **WHEN** 主题模式为"跟随系统"且系统深浅模式切换
- **THEN** 应用界面同步切换为对应主题

### Requirement: 视觉风格
系统 SHALL 采用极简扁平视觉风格（参考 Windows 11 Fluent Design）：无边框窗口（圆角 8px）、圆角元素、图标式工具栏（悬停显示文字提示）、微妙阴影；工具栏/浮层 SHALL 在 Windows 11 上使用亚克力（Acrylic）材质，在 Windows 10 上自动降级为普通半透明。

#### Scenario: Win11 亚克力材质
- **WHEN** 应用运行在 Windows 11 且工具栏显示
- **THEN** 工具栏背景呈现亚克力半透明模糊效果

#### Scenario: Win10 材质降级
- **WHEN** 应用运行在 Windows 10
- **THEN** 工具栏背景呈现普通半透明效果，功能与布局不受影响

#### Scenario: 工具栏悬停提示
- **WHEN** 鼠标悬停在工具栏图标上
- **THEN** 显示该功能的文字提示（当前语言），提示中包含快捷键（如 "画笔 (B)"）

### Requirement: 窗口形态
主窗口 SHALL 采用无边框设计，窗口圆角 8px，支持拖动移动、边缘调整大小、最大化/还原/最小化。标题栏高度 32px，窗口控制按钮（最小化/最大化/关闭）各 46×32px。窗口内容 SHALL 正确呈现圆角。

#### Scenario: 无边框窗口拖动
- **WHEN** 用户拖动窗口顶部区域
- **THEN** 窗口跟随移动

#### Scenario: 边缘调整大小
- **WHEN** 用户将鼠标移到窗口边缘并拖动
- **THEN** 窗口对应方向调整大小

### Requirement: 工具栏布局
顶部 SHALL 提供打开和文件名，以及带文字的编辑与压缩入口。画布承载浏览，底部 SHALL 提供缩放、适应窗口、原始大小和缩略图开关。图标按钮提供状态反馈及本地化快捷键提示。

#### Scenario: 浏览入口可识别
- **WHEN** 用户打开图片
- **THEN** 用户可从顶部进入编辑和压缩，从底部切换缩放和缩略图，并可用快捷键访问预览变换

### Requirement: 底部信息条
窗口底部 SHALL 提供半透明亚克力信息条，高度 32px，显示当前图片的文件名、分辨率、文件大小（格式："文件名 · 宽×高 · 文件大小"，12px 灰色文字）。信息条默认隐藏，当鼠标悬停到信息条自身区域时以 200ms ease-out 淡入显示，离开后淡出隐藏。

#### Scenario: 鼠标悬停显示
- **WHEN** 鼠标移动到信息条区域
- **THEN** 信息条以 200ms ease-out 淡入并显示"文件名 · 宽×高 · 文件大小"

#### Scenario: 鼠标离开隐藏
- **WHEN** 鼠标离开信息条区域
- **THEN** 信息条淡出隐藏，不遮挡图片查看

### Requirement: 缩略图导航条
系统 SHALL 提供缩略图导航条，高度 72px，默认隐藏，支持通过 T 键或工具栏按钮切换显示/隐藏。导航条 SHALL 显示当前目录所有支持图片的缩略图（64×64px，圆角 4px，间距 4px），当前图片以 2px accent 色边框高亮，其他缩略图悬停时显示 1px `border.subtle` 边框。导航条以 200ms ease-out 从底部滑入/滑出。

#### Scenario: 展开缩略图条
- **WHEN** 用户按 T 键
- **THEN** 缩略图导航条以 200ms ease-out 从底部滑入，显示目录内图片缩略图，当前图片以 2px accent 边框高亮

#### Scenario: 点击缩略图跳转
- **WHEN** 缩略图条展开且用户点击某个缩略图
- **THEN** 主窗口显示该缩略图对应的图片

### Requirement: EXIF 侧边面板
系统 SHALL 提供按 I 键呼出/关闭的 EXIF 详情侧边面板，宽度 280px，从窗口右侧以 200ms ease-out 滑入/滑出。面板 SHALL 显示 JPEG 等格式中可用的 EXIF 元数据（拍摄设备、镜头、光圈、快门、ISO、焦距、拍摄时间、GPS 等）；无 EXIF 数据时 SHALL 显示"无元数据"占位提示。当 EXIF 含 GPS 位置数据时，面板顶部 SHALL 显示 warning 色背景条标记"含 GPS 位置信息"。

#### Scenario: 查看 EXIF
- **WHEN** 当前图片为含 EXIF 的 JPEG 且用户按 I
- **THEN** 右侧 280px 面板以 200ms ease-out 滑入，列出可用的 EXIF 字段及值

#### Scenario: 无 EXIF 数据
- **WHEN** 当前图片（如 PNG）不含 EXIF 且用户按 I
- **THEN** 面板显示"无元数据"提示而非空白或报错

#### Scenario: 关闭面板
- **WHEN** 面板展开时用户再按 I 或点击面板右上角关闭按钮
- **THEN** 面板以 200ms ease-out 滑出收起

#### Scenario: GPS 数据隐私标记
- **WHEN** 当前图片的 EXIF 含 GPS 位置数据
- **THEN** EXIF 面板顶部显示 warning 色背景条"含 GPS 位置信息"，提醒用户分享前注意隐私

### Requirement: 快捷键体系
系统 SHALL 实现以下全局快捷键：← / →（上/下一张）、Ctrl+O（打开文件）、Ctrl+滚轮（缩放）、Ctrl+0（适应窗口）、Ctrl+1（原始大小）、F11/双击（全屏）、R / Shift+R（顺/逆时针旋转）、H（水平翻转）、V（垂直翻转）、B（画笔）、Ctrl+Z（撤销）、Ctrl+Y / Ctrl+Shift+Z（重做）、Ctrl+S（覆盖保存，带确认）、Ctrl+Shift+S（另存为）、Ctrl+C / Ctrl+V（复制 / 粘贴图片）、I（信息面板）、T（缩略图条）、Esc/W（退出全屏/关闭）。

#### Scenario: 快捷键触发功能
- **WHEN** 在非输入焦点状态下用户按任一上述快捷键
- **THEN** 对应功能立即执行

#### Scenario: 快捷键与语言无关
- **WHEN** 界面语言为英文时用户按 B 键
- **THEN** 画笔工具激活（快捷键不随语言变化）

### Requirement: 高 DPI 与多显示器
系统 SHALL 以 Per-Monitor V2 DPI 感知运行：在 100%/150%/200% 等系统缩放下 UI 与图片渲染清晰无模糊；跨不同 DPI 显示器拖动窗口时内容 SHALL 按目标屏 DPI 正确重缩放，布局不跳变、不错乱。

#### Scenario: 高缩放比例显示
- **WHEN** 系统显示缩放为 200%
- **THEN** 界面图标、文字、图片均清晰渲染，无模糊或错位

#### Scenario: 跨屏拖动
- **WHEN** 用户将窗口从 100% 缩放显示器拖到 150% 缩放显示器
- **THEN** 窗口内容按目标屏 DPI 正确重缩放，布局不错乱

### Requirement: 动效规范
界面过渡动画 SHALL 采用分级时长与缓出（ease-out）曲线：图片切换淡入约 100ms；面板呼出/收起（EXIF 面板、缩略图条）约 200ms；信息条淡入淡出约 200ms；按钮态、浮层显示等约 150-200ms。系统开启"减少动态效果"辅助功能时 SHALL 全局禁用过渡动画，状态切换即时完成。

#### Scenario: 图片切换淡入
- **WHEN** 用户按 → 切换到下一张图片
- **THEN** 新图片以约 100ms ease-out 淡入过渡显示

#### Scenario: 面板呼出动效
- **WHEN** 用户按 I 呼出 EXIF 面板
- **THEN** 面板以约 200ms ease-out 动画从右侧滑入显示

#### Scenario: 按钮态动效
- **WHEN** 用户悬停/按下工具栏按钮
- **THEN** 背景色与图标缩放以 150ms ease-out 过渡

#### Scenario: 减少动画模式
- **WHEN** Windows 辅助功能开启"减少动态效果"
- **THEN** 所有过渡动画被跳过，状态切换即时完成
