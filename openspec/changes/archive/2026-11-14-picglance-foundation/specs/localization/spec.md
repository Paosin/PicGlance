## ADDED Requirements

### Requirement: 界面语言
系统 SHALL 默认以中文（简体）显示界面，并 SHALL 支持切换为英文；语言切换 SHALL 即时生效无需重启。

#### Scenario: 默认中文
- **WHEN** 应用首次启动（无历史设置）
- **THEN** 界面以简体中文显示

#### Scenario: 切换为英文
- **WHEN** 用户在设置中将语言切换为 English
- **THEN** 所有界面文本（菜单、按钮、提示、对话框）立即切换为英文

#### Scenario: 界面文本全覆盖
- **WHEN** 界面语言为任一支持语言
- **THEN** 所有用户可见文本均为该语言，无遗漏的硬编码字符串（系统托盘菜单等系统级组件除外）

### Requirement: 系统集成文案本地化
文件关联描述、右键菜单项等系统集成文案 SHALL 随界面语言正确本地化（中文/英文）。

#### Scenario: 右键菜单文案
- **WHEN** 界面语言为英文时用户在资源管理器右键图片
- **THEN** 右键菜单项显示 "Open with PicGlance"；语言为中文时显示"用 PicGlance 打开"
