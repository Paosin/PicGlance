# image-compression Specification

## Purpose
TBD - created by archiving change picglance-compression. Update Purpose after archive.

## Requirements

### Requirement: 目标大小智能压缩
系统 SHALL 提供智能压缩功能：用户指定目标文件大小（如 500KB）并选择输出格式，系统 SHALL 自动迭代决策压缩参数，输出不超过目标大小的文件；静态图策略为：先从高质量开始逐步降低质量参数（二分搜索），质量到达下限仍未满足时再等比缩小分辨率，直至满足目标。系统 SHALL NOT 自动更改输出格式，输出格式由用户选择。系统 SHALL 提供"保持尺寸"选项，勾选后跳过分辨率缩小步骤。

压缩引擎采用策略模式（Strategy Pattern），各格式编码器实现统一的 `ICompressStrategy` 接口，包括：MozJPEG（JPEG）、pngquant（PNG）、Qt 内置编码器（WebP）。二分搜索和分辨率降级由 `CompressOrchestrator` 统一驱动，不含格式特定逻辑。

#### Scenario: 降质量满足目标
- **WHEN** 用户将一张 5MB 的 JPEG 压缩到 1MB（输出 JPEG）
- **THEN** 输出文件不大于 1MB，分辨率保持原始尺寸，质量尽可能高（使用 MozJPEG 编码）

#### Scenario: 降质量不足时缩分辨率
- **WHEN** 目标大小远小于图片尺寸，纯降质量到下限仍超出目标且未勾选"保持尺寸"
- **THEN** 系统等比缩小分辨率继续迭代，直至输出文件不大于目标大小

#### Scenario: 保持尺寸模式
- **WHEN** 用户勾选"保持尺寸"并压缩
- **THEN** 系统仅降低质量参数，不缩小分辨率

#### Scenario: 无法满足时提示
- **WHEN** 压缩到分辨率下限（如最长边 16px）仍无法满足目标大小
- **THEN** 系统提示无法压缩到目标大小并给出最接近的结果，不静默失败

#### Scenario: 格式不被转换
- **WHEN** 用户压缩一张 PNG 并选择输出 PNG
- **THEN** 输出文件为 PNG 格式（经 pngquant 量化），系统不擅自转为 JPEG 或 WebP

### Requirement: 默认格式保留
压缩单张图片时，输出格式 SHALL 默认与源文件格式一致（JPEG→JPEG、PNG→PNG、WebP→WebP、GIF→GIF）。格式选择器中源格式排在首位。用户仍可手动切换为其他支持的输出格式。动图输入时，输出格式限制为动图格式（GIF/WebP）。

#### Scenario: JPEG 源默认 JPEG 输出
- **WHEN** 用户对 `.jpg` 文件打开压缩对话框
- **THEN** 输出格式默认选中 JPEG

#### Scenario: GIF 源默认 GIF 输出
- **WHEN** 用户对 `.gif` 文件打开压缩对话框
- **THEN** 输出格式默认选中 GIF，可选格式为 GIF 和 WebP

### Requirement: 动图压缩参数
系统 SHALL 为动图（GIF/动图 WebP）压缩提供简单易懂的参数：目标大小、输出质量滑块、是否允许抽帧（开关，默认关）。允许抽帧时，WebP 格式可自动降低帧率以满足目标大小；GIF 格式当前不支持抽帧，该开关对 GIF 无效。GIF 编码委托 gifsicle 外部工具（lossy + O3 优化），WebP 编码使用 Qt 内置编码器。

#### Scenario: 不允许抽帧
- **WHEN** 用户关闭抽帧开关并压缩动图
- **THEN** 输出动图保持原始帧序列，仅通过质量（及必要时分辨率）压缩

#### Scenario: 允许抽帧（WebP）
- **WHEN** 用户开启抽帧开关并压缩动图 WebP 到指定目标大小
- **THEN** 系统可在质量与分辨率之外自动降低帧率（如隔帧保留）以满足目标

#### Scenario: 动图分辨率缩小
- **WHEN** 动图压缩需要缩小分辨率
- **THEN** GIF 格式以 0.75 倍缩放因子缩小，WebP 格式以 0.5 倍缩放因子缩小

### Requirement: 压缩结果反馈
压缩完成后系统 SHALL 显示压缩结果信息：原始大小 → 压缩后大小（绿色 `#6CCB5F` 深色 / `#107C10` 浅色）+ 减少百分比（accent 蓝色），并让用户选择保存位置。对话框宽度 520px，高度 600px，圆角 8px，模态居中显示。

#### Scenario: 压缩完成反馈
- **WHEN** 一次压缩完成
- **THEN** 对话框内显示"原始 X MB → 压缩后 Y KB（减少 Z%）"，压缩后大小以绿色显示，减少百分比以蓝色显示，底部提供"取消"与"保存为..."按钮

### Requirement: 压缩支持的格式
智能压缩 SHALL 支持 JPEG、PNG、WebP 静态图与 GIF 动图作为输入；输出格式 SHALL 限于 JPEG、PNG、WebP（静态）与 GIF、WebP（动图）。

#### Scenario: 输入输出格式组合
- **WHEN** 用户压缩一张 WebP 动图
- **THEN** 可选输出格式为 GIF 或 WebP（动图）

#### Scenario: 不可压缩格式的入口
- **WHEN** 当前图片为 SVG（或其它不在压缩支持清单内的格式）且用户尝试打开压缩功能
- **THEN** 压缩入口禁用或显示本地化提示说明该格式不支持智能压缩

### Requirement: 后台线程与取消
所有压缩操作（静态和动图）SHALL 在 `QtConcurrent` 后台线程运行，UI 保持响应。每轮迭代 SHALL 发送进度信号更新进度条。系统 SHALL 支持通过 `cancel()` 方法取消压缩，当前迭代完成后停止后续迭代，部分结果丢弃。

#### Scenario: 压缩在后台运行
- **WHEN** 压缩开始
- **THEN** UI 保持响应，进度条随迭代更新

#### Scenario: 取消压缩
- **WHEN** 用户在压缩过程中点击取消
- **THEN** 当前迭代完成后停止，不产生输出文件

### Requirement: 内存编码
所有压缩迭代 SHALL 编码到内存 `QBuffer` 测量大小，不写盘。例外：外部工具（pngquant、gifsicle）使用临时文件，工具完成后立即删除。

#### Scenario: 质量搜索不写盘
- **WHEN** 二分搜索遍历质量级别
- **THEN** 不向磁盘写入临时图片文件

#### Scenario: 外部工具临时文件清理
- **WHEN** pngquant 或 gifsicle 被调用
- **THEN** 所有临时文件在工具完成后（无论成功或失败）立即删除
