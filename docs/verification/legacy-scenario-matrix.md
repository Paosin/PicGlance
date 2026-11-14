# PicGlance 旧版逐场景验收矩阵

2026-11-14。依据相邻工作区 PicGlance-tmp 的 12 份主规格。源位置以相邻工作区内的行号标识。已取证的场景附结果。其余场景保留待验，表中的模块交接用于定位复测。

汇总、构建哈希、产物与冲突处置见[验收报告](picglance-acceptance.md)。

## app-distribution

源文件 `openspec/specs/app-distribution/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 6 | 安装包安装与卸载 | Desktop 生产候选通过，构建和执行细节见桌面交接。 |
| 10 | 便携版运行 | Desktop 生产候选通过，构建和执行细节见桌面交接。 |
| 17 | 覆盖安装升级 | Desktop 生产候选通过，构建和执行细节见桌面交接。 |
| 21 | 格式清单差量更新 | Desktop 生产候选通过，构建和执行细节见桌面交接。 |
| 28 | 加载失败留痕 | 合并 Release 实测损坏 JPEG，界面提示并记录完整路径和 invalid-input。 |
| 35 | 发现新版本 | 受控 v2.99.0 响应通过显示与忽略，真实 GitHub 新版本仍待验。 |
| 39 | 网络失败静默 | 实际更新请求返回 HTTP 404，日志留痕且界面保持静默；断网路径仍待验。 |
| 43 | 关闭自动检查 | 待完整复验。参考Desktop 生产候选交接。 |

## app-shell-ui

源文件 `openspec/specs/app-shell-ui/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 6 | 切换主题即时生效 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 10 | 跟随系统 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 17 | Win11 亚克力材质 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 21 | Win10 材质降级 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 25 | 工具栏悬停提示 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 32 | 无边框窗口拖动 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 36 | 边缘调整大小 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 43 | 工具栏按钮悬停 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 47 | 工具栏按钮按下 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 54 | 鼠标悬停显示 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 58 | 鼠标离开隐藏 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 65 | 展开缩略图条 | T 展开并出现同目录条目。 |
| 69 | 点击缩略图跳转 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 76 | 查看 EXIF | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 80 | 无 EXIF 数据 | I 打开后显示无元数据。 |
| 84 | 关闭面板 | 再次按 I 收起；关闭按钮路径仍待验。 |
| 88 | GPS 数据隐私标记 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 95 | 快捷键触发功能 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 99 | 快捷键与语言无关 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 106 | 高缩放比例显示 | 实际 100%／150%／200% 便携候选窗口 DPI 为 96／144／192；安装候选三档深色界面图标可辨。运行和固定任务栏、同一构建浅色及键盘/读屏待验。 |
| 110 | 跨屏拖动 | 验收环境只有一台显示器，待双屏复验。 |
| 117 | 图片切换淡入 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 121 | 面板呼出动效 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 125 | 按钮态动效 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |
| 129 | 减少动画模式 | 待完整复验。参考Foundation 窗口交接及 Icon System 模拟 DPR 夹具。 |

## compress-format-lock

源文件 `openspec/specs/compress-format-lock/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 7 | JPEG source defaults to JPEG output | 待完整复验。参考Compression 格式顺序与真实编码样本。 |
| 11 | PNG source defaults to PNG output | 待完整复验。参考Compression 格式顺序与真实编码样本。 |
| 15 | GIF source defaults to GIF output | 待完整复验。参考Compression 格式顺序与真实编码样本。 |
| 19 | WebP source defaults to WebP output | 待完整复验。参考Compression 格式顺序与真实编码样本。 |
| 29 | User changes output format | 待完整复验。参考Compression 格式顺序与真实编码样本。 |
| 33 | Format list for GIF source | 待完整复验。参考Compression 格式顺序与真实编码样本。 |
| 41 | Animated GIF compression preserves format | 待完整复验。参考Compression 格式顺序与真实编码样本。 |
| 45 | Animated WebP compression preserves format | 待完整复验。参考Compression 格式顺序与真实编码样本。 |

## compress-strategy-pattern

源文件 `openspec/specs/compress-strategy-pattern/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 13 | Strategy interface contract | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 17 | Strategy registration and lookup | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 25 | JPEG encoding produces valid output | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 29 | MozJPEG compression ratio improvement | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 43 | PNG compression via pngquant | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 47 | pngquant not found | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 55 | Static GIF compression | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 59 | Animated GIF compression with frame drop | 以现行 image-compression 和压缩重构归档的动图专节为准，GIF 保留帧，WebP 才可抽帧。 |
| 67 | WebP encoding | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 80 | External tool invocation with stdin/stdout | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 84 | Tool not found | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 88 | Tool timeout | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 102 | Format-based strategy selection | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 106 | Resolution fallback | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |
| 119 | Animated GIF compression pipeline | 待完整复验。参考Compression 编码、超时、取消及元数据测试。 |

## image-compression

源文件 `openspec/specs/image-compression/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 8 | 降质量满足目标 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 12 | 降质量不足时缩分辨率 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 16 | 保持尺寸模式 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 20 | 无法满足时提示 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 24 | 格式不被转换 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 31 | JPEG 源默认 JPEG 输出 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 35 | GIF 源默认 GIF 输出 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 42 | 不允许抽帧 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 46 | 允许抽帧（WebP） | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 50 | 动图分辨率缩小 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 57 | 压缩完成反馈 | G1 界面显示实际大小、减幅和维度，输出 43,688 字节，目标 46,080 字节。 |
| 64 | 输入输出格式组合 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 68 | 不可压缩格式的入口 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 75 | 压缩在后台运行 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 79 | 取消压缩 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 86 | 质量搜索不写盘 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |
| 90 | 外部工具临时文件清理 | 待完整复验。参考Compression 交接及 Rust/TS 测试。 |

## image-editing

源文件 `openspec/specs/image-editing/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 6 | 工具选中态 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 10 | 编辑模式缩放指示 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 17 | 自由手绘 | G1 另存像素与原图不同，源哈希一致。 |
| 21 | 撤销画笔操作 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 28 | 绘制直线 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 32 | 绘制箭头 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 36 | 绘制矩形 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 40 | 绘制椭圆 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 47 | 自由裁剪 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 51 | 固定比例裁剪 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 58 | 涂抹像素马赛克 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 62 | 涂抹高斯模糊 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 66 | 颜色填充 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 70 | 噪点马赛克 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 74 | 圆形马赛克 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 78 | 彩色玻璃马赛克 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 85 | 编辑模式旋转并保存 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 89 | 编辑模式水平翻转并保存 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 93 | 编辑模式垂直翻转并保存 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 97 | 调整尺寸 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 101 | 变换与预览操作相互独立 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 108 | 多步撤销 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 112 | 重做被撤销的操作 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 116 | 切换图片清空历史 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 123 | 覆盖保存确认 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 127 | 另存为 | G1 原生 PNG 另存及解码通过，Editor 补测原生 BMP 另存及重开。 |
| 131 | 未保存离开提示 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 138 | 放大后绘制细节 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |
| 145 | SVG 禁用编辑 | 待完整复验。参考Editor 交接及 Rust/TS 测试。 |

## image-info

源文件 `openspec/specs/image-info/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 6 | 鼠标悬停显示 | 待完整复验。参考Viewer 交接及 EXIF Rust 测试。 |
| 10 | 鼠标离开隐藏 | 待完整复验。参考Viewer 交接及 EXIF Rust 测试。 |
| 17 | 查看 EXIF | 待完整复验。参考Viewer 交接及 EXIF Rust 测试。 |
| 21 | 无 EXIF 数据 | I 打开后显示无元数据。 |
| 25 | 关闭面板 | 再次按 I 收起；关闭按钮路径仍待验。 |
| 29 | GPS 数据隐私标记 | 待完整复验。参考Viewer 交接及 EXIF Rust 测试。 |

## image-viewing

源文件 `openspec/specs/image-viewing/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 6 | 打开静态图片 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 10 | 播放动图 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 14 | 不支持的格式 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 21 | 右键打开大文件的窗口响应 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 28 | 打开超大文件被拒绝 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 35 | 切换到下一张 | 右方向键从 01 测试图切至 02 编辑源，见验收报告的交互复测。 |
| 39 | 切换到上一张 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 46 | 展开缩略图条 | T 展开且无障碍树出现同目录图片。 |
| 50 | 点击缩略图跳转 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 57 | 打开大图 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 61 | 打开小图 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 68 | 滚轮缩放 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 72 | 适应窗口还原图标 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 76 | 原始大小还原图标 | Ctrl+1 后状态显示 100%。 |
| 83 | 临时旋转 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 87 | 临时水平翻转 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 91 | 临时垂直翻转 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 98 | 进入与退出全屏 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 102 | 全屏时关闭程序 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 109 | 拖入文件 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 116 | 打开损坏图片 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 120 | 浏览中文件被删除 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 124 | 中文路径 | Ctrl+O 后原生选择器成功打开中文空格文件名。 |
| 131 | 复制当前图片 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 135 | 粘贴打开剪贴板图片 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 142 | 切换淡入过渡 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |
| 146 | 快速连切不卡顿 | 待完整复验。参考Viewer 交接及 Rust/TS 测试。 |

## localization

源文件 `openspec/specs/localization/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 6 | 默认中文 | 待完整复验。参考Foundation 词典测试及 Desktop 交接。 |
| 10 | 切换为英文 | 待完整复验。参考Foundation 词典测试及 Desktop 交接。 |
| 14 | 界面文本全覆盖 | 待完整复验。参考Foundation 词典测试及 Desktop 交接。 |
| 21 | 右键菜单文案 | 待完整复验。参考Foundation 词典测试及 Desktop 交接。 |

## performance-budget

源文件 `openspec/specs/performance-budget/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 6 | 冷启动计时 | Shell 动词 744、651、657 ms。按用户决定以资源管理器右键入口为验收路径。 |
| 13 | 常规图片内存 | 待定，暂停复测。整树工作集中位数 407.51 MB，原预算 50 MB。 |
| 17 | 超大图片降采样 | 基线 JPEG 20,000×10,000 已按解码内缩放显示，整树峰值 370.8 MiB；其他超大格式待裁决。 |
| 24 | 发布产物体积 | 安装包大小判定待定。离线包 221,538,077 字节，原预算 30 MB；在线包和 ZIP 实测见报告。 |
| 31 | 快速切换 | G1 基线超时。后续 Viewer 候选的三组样本共 42 次均低于 100 ms，待在合并安装包复验。该候选的 GIF 仍有帧号跳跃。 |

## settings

源文件 `openspec/specs/settings/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 6 | 修改设置并持久化 | 待完整复验。参考Desktop 独立宿主与生产候选交接。 |
| 10 | 管理文件关联 | 待完整复验。参考Desktop 独立宿主与生产候选交接。 |
| 17 | 选择深色主题 | 待完整复验。参考Desktop 独立宿主与生产候选交接。 |
| 24 | 最小化到托盘 | 待完整复验。参考Desktop 独立宿主与生产候选交接。 |
| 31 | 便携版设置隔离 | 待完整复验。参考Desktop 独立宿主与生产候选交接。 |

## windows-integration

源文件 `openspec/specs/windows-integration/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 6 | 安装时默认关联 | 旧默认双击场景按用户决定退出本轮验收，改验资源管理器右键打开。 |
| 10 | 用户取消部分关联 | Desktop 生产候选通过，构建和执行细节见桌面交接。 |
| 17 | 传统菜单打开 | 待完整复验。参考Desktop 生产候选交接。 |
| 21 | Win11 显示更多选项中打开 | 初次实测旧菜单启动旧版。修复后的 Shell 枚举识别新版命令，图形菜单缓存刷新后待实点复验。 |
| 28 | 创建快捷方式 | Desktop 生产候选通过，构建和执行细节见桌面交接。 |
| 35 | 复用已有实例 | Desktop 生产候选通过，构建和执行细节见桌面交接。 |

## 归档中的压缩增量

两个归档的其他规格场景已纳入对应主规格。下表列出 refactor-compress-encoders 的 image-compression 增量。源文件为 `openspec/changes/archive/refactor-compress-encoders/specs/image-compression/spec.md`。

| 行 | 场景 | 状态和证据 |
|---:|---|---|
| 9 | JPEG compression with MozJPEG | Rust 压缩集成测试通过 JPEG 目标体积与尺寸保持。 |
| 13 | PNG compression with pngquant | Rust 压缩集成测试通过透明 PNG 及调色板元数据。 |
| 17 | Binary search converges within iteration limit | 待完整复验。参考 Compression 交接。 |
| 25 | Downscaling meets target | 待完整复验。参考 Compression 交接。 |
| 29 | Resolution floor reached | 待完整复验。参考 Compression 交接。 |
| 44 | Animated GIF uses gifsicle | Compression Windows 样本已核对 GIF 尺寸、帧数和逐帧时长。 |
| 48 | Animated WebP uses Qt encoder | 新版以 libwebp 编码。Compression 样本已核对输出、逐帧时长及透明度。 |
| 56 | Compression runs off UI thread | 待完整复验。参考 Compression 交接。 |
| 60 | Cancellation | Rust 测试通过候选丢弃与取消后禁止发布。生产窗口取消待验。 |
| 68 | No disk writes during quality search | 待完整复验。参考 Compression 交接。 |
| 72 | External tool temp files cleaned up | 新版工具走 stdin/stdout。待记录完整生产压缩过程的文件 I/O。 |

add-image-viewer-core 的压缩初始归档包含同名基础场景。格式保留及动图行为以之后的压缩增量和现行主规格为准。
