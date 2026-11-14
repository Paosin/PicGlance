## ADDED Requirements

### Requirement: 底部信息条
系统 SHALL 在窗口底部提供半透明亚克力信息条，高度 32px，显示当前图片的文件名、分辨率、文件大小（格式："文件名 · 宽×高 · 文件大小"，12px 灰色文字）。信息条 SHALL 在鼠标悬停到信息条自身区域时以 200ms ease-out 淡入显示、离开后淡出隐藏。

#### Scenario: 鼠标悬停显示
- **WHEN** 鼠标移动到信息条区域
- **THEN** 信息条淡入并显示"文件名 · 宽×高 · 文件大小"

#### Scenario: 鼠标离开隐藏
- **WHEN** 鼠标离开信息条区域
- **THEN** 信息条淡出隐藏，不遮挡图片查看

### Requirement: EXIF 详情面板
系统 SHALL 提供按 I 键呼出/关闭的 EXIF 详情侧边面板，宽度 280px，从窗口右侧以 200ms ease-out 滑入/滑出，显示 JPEG 等格式中可用的 EXIF 元数据（拍摄设备、镜头、光圈、快门、ISO、焦距、拍摄时间、GPS 等）；无 EXIF 数据时 SHALL 显示"无元数据"占位提示。当 EXIF 含 GPS 位置数据时，面板顶部 SHALL 显示 warning 色背景条标记"含 GPS 位置信息"。

#### Scenario: 查看 EXIF
- **WHEN** 当前图片为含 EXIF 的 JPEG 且用户按 I
- **THEN** 侧边浮层列出可用的 EXIF 字段及值

#### Scenario: 无 EXIF 数据
- **WHEN** 当前图片（如 PNG）不含 EXIF 且用户按 I
- **THEN** 面板显示"无元数据"提示而非空白或报错

#### Scenario: 关闭面板
- **WHEN** 面板展开时用户再按 I 或点击关闭按钮
- **THEN** 面板收起

#### Scenario: GPS 数据隐私标记
- **WHEN** 当前图片的 EXIF 含 GPS 位置数据
- **THEN** EXIF 面板在显眼位置显示"含 GPS 位置信息"标记，提醒用户分享前注意隐私
