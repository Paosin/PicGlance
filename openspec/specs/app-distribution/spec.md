# app-distribution Specification

## Purpose
TBD - created by archiving change picglance-desktop. Update Purpose after archive.

## Requirements

### Requirement: 分发形式
项目 SHALL 提供一个内置 WebView2 完整运行时的 Windows x64 离线安装包，以及包含 `portable.marker` 的便携 ZIP。安装向导 SHALL 提供简体中文和 English，并在首次安装和覆盖安装时显示可编辑的安装目录与「浏览」按钮。便携版 SHALL 解压即用，设置存于程序目录，不注册系统集成。

#### Scenario: 离线安装
- **WHEN** 用户在未联网的电脑上运行安装包
- **THEN** 用户可选择安装语言和目录，缺少 WebView2 时使用包内运行时完成安装

#### Scenario: 卸载文件范围
- **WHEN** 用户在系统「应用」中卸载 PicGlance
- **THEN** 仅删除安装目录内的程序文件和已知日志，清除本程序注册的右键入口及卸载登记，保留目录外的文件和用户设置

#### Scenario: 安装路径指向外部目录
- **WHEN** 卸载前检查发现安装路径或受管理的子目录经目录联接或符号链接重定向
- **THEN** 停止卸载，不删除外部目标中的文件

#### Scenario: 便携版运行
- **WHEN** 用户解压便携版并直接运行 exe
- **THEN** 应用正常启动，不向注册表写入任何项，设置保存在程序目录

### Requirement: 升级安装保留用户数据
覆盖安装新版本时，安装器 SHALL 预填原路径，保留用户设置，并更新资源管理器的 PicGlance 右键入口。

#### Scenario: 覆盖安装升级
- **WHEN** 用户在已安装 v1.0 的机器上直接运行 v1.1 安装包完成安装
- **THEN** 用户可查看或更改预填路径，用户设置保留，右键入口指向新版本

#### Scenario: 替换旧卸载记录
- **WHEN** 用户在原目录覆盖安装带有旧卸载记录的 PicGlance
- **THEN** 新卸载记录不继承旧版针对安装目录外快捷方式的文件删除操作

### Requirement: 运行日志
系统 SHALL 将运行日志写入文件：安装版写入安装目录下 `log/` 目录（该目录不可写时自动回退到 `%LOCALAPPDATA%\PicGlance\log`），便携版写入程序目录下 `log/` 目录。日志 SHALL 记录关键日志点：图片加载失败、超大文件风险提示触发、更新检查结果、压缩耗时与结果、异常/崩溃信息。

#### Scenario: 加载失败留痕
- **WHEN** 一张损坏的 JPEG 解码失败
- **THEN** 用户看到本地化错误提示，且 log/ 目录的日志文件记录该失败（文件路径、失败原因）

### Requirement: 自动更新检查
应用 SHALL 在启动时（可关闭）后台检查 GitHub Releases 是否有新版本；发现新版本时 SHALL 在主窗口显示更新提示横幅，由用户决定是否下载更新（点击后打开下载页面）；检查失败 SHALL 静默处理，不影响正常使用。

#### Scenario: 发现新版本
- **WHEN** 启动时检查到 GitHub Releases 上有比当前更高的版本号
- **THEN** 主窗口显示"发现新版本 vX.Y"提示横幅，用户可点击打开下载页面或忽略

#### Scenario: 网络失败静默
- **WHEN** 启动时无网络或 GitHub 不可达
- **THEN** 不弹错误提示，应用正常启动使用

#### Scenario: 关闭自动检查
- **WHEN** 用户在设置中关闭"启动时检查更新"
- **THEN** 后续启动不再发起检查请求
