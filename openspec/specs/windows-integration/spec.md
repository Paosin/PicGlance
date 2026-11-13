# windows-integration Specification

## Purpose
TBD - created by archiving change picglance-desktop. Update Purpose after archive.

## Requirements

### Requirement: 文件关联
安装器 SHALL 注册资源管理器右键入口，省略文件关联选择及默认应用引导，并保留 Windows 当前默认图片应用。应用设置中的手动关联管理仍可用。

#### Scenario: 安装后通过右键打开
- **WHEN** 用户以默认选项完成安装
- **THEN** 支持的图片格式可通过「用 PicGlance 打开」打开，安装器不为文件格式注册 PicGlance 处理程序

#### Scenario: 升级时移除旧版注册
- **WHEN** 覆盖安装检测到旧版 PicGlance 的格式处理程序注册
- **THEN** 移除旧版 PicGlance 的格式注册，保留其他应用的关联并继续提供 PicGlance 右键入口

### Requirement: 右键菜单集成
系统 SHALL 在资源管理器右键菜单中提供"用 PicGlance 打开"项：SHALL 注册传统右键菜单（Win10 直接可见；Win11 在"显示更多选项"中可见）。卸载时 SHALL 清理菜单注册。

#### Scenario: 传统菜单打开
- **WHEN** 用户在 Windows 10 资源管理器中右键图片
- **THEN** 菜单中出现"用 PicGlance 打开"项，点击后本应用打开该图片

#### Scenario: Win11 显示更多选项中打开
- **WHEN** 用户在 Windows 11 资源管理器中右键图片并点击"显示更多选项"
- **THEN** 菜单中出现"用 PicGlance 打开"项，点击后本应用打开该图片

### Requirement: 快捷方式
安装程序 SHALL 省略目录外的桌面和开始菜单快捷方式选项，卸载 SHALL 保留安装目录外的快捷方式。

#### Scenario: 保留已有快捷方式
- **WHEN** 用户卸载程序且安装目录外仍有以前创建的快捷方式
- **THEN** 卸载保留该快捷方式文件

### Requirement: 多实例文件打开
通过文件关联或右键菜单打开图片时，系统 SHALL 复用已运行实例（在其窗口中显示新图片）而非启动新进程，以控制内存占用。

#### Scenario: 复用已有实例
- **WHEN** 应用已运行且用户右键选择「用 PicGlance 打开」打开另一张图片
- **THEN** 已运行实例的窗口置前并显示新图片
