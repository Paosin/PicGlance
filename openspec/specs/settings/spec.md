# settings Specification

## Purpose
TBD - created by archiving change picglance-desktop. Update Purpose after archive.

## Requirements

### Requirement: 设置分类
系统 SHALL 提供设置页面，采用左侧导航 + 右侧内容区布局。左侧导航栏宽度 200px（`bg.secondary` 背景），导航项高度 36px、圆角 4px，选中态以 2px accent 色左边框指示条 + `accent @ 15%` 底色高亮。右侧内容区 `bg.primary` 背景，最大宽度 600px，左对齐，32px 内边距。设置分类包括：通用（语言、启动时检查更新开关、关闭行为）、外观（主题模式）、查看（默认缩放模式、小图是否放大）、编辑（画笔默认颜色、画笔默认粗细、马赛克默认块大小）、文件关联（管理关联格式）、压缩（默认输出格式、默认目标大小）、关于（版本号、检查更新、开源许可信息）。

#### Scenario: 修改设置并持久化
- **WHEN** 用户修改任一设置项并关闭设置页
- **THEN** 设置立即生效并持久化，下次启动保持

#### Scenario: 管理文件关联
- **WHEN** 用户在设置的文件关联页取消勾选某格式
- **THEN** 该格式的系统文件关联被移除，双击该格式不再用本应用打开

### Requirement: 主题模式选择
外观设置中的主题模式 SHALL 采用下拉选择框（ComboBox）提供三个选项：深色、浅色、跟随系统。

#### Scenario: 选择深色主题
- **WHEN** 用户在主题下拉框中选择"深色"
- **THEN** 界面立即切换为深色主题

### Requirement: 关闭行为设置
通用设置中 SHALL 提供关闭行为选项，控制用户点击窗口关闭按钮时的行为：退出程序、最小化到系统托盘、或后台运行。

#### Scenario: 最小化到托盘
- **WHEN** 用户将关闭行为设为"最小化到托盘"并点击关闭按钮
- **THEN** 窗口隐藏，应用继续在系统托盘运行

### Requirement: 设置存储
设置 SHALL 存储在用户本地配置（如 %APPDATA% 或 INI/QSettings），便携版 SHALL 将设置存储在程序目录下（INI 格式），不污染系统注册表。

#### Scenario: 便携版设置隔离
- **WHEN** 用户使用便携版并修改主题
- **THEN** 设置文件写入程序所在目录（settings.ini），系统注册表无新增配置项
