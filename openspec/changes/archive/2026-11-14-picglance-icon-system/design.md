总设计、写入边界及公共接口见[基线总设计](../picglance-foundation/design.md)。

## Context

建立统一轮廓图标、Logo 与 Windows ICO。

## Goals / Non-Goals

本包完成图标和应用标识，公共接口定义和应用接线归基线 owner。

## Decisions

- 覆盖看图、编辑、压缩及设置图标；实现公共 Icon 契约。
- 检查缩放、旋转、翻转、表态图标的辨识度、主题对比度和工具提示。
- 制作多尺寸 ICO，核验标题栏、任务栏、资源管理器矩阵。
- 采用统一 20 px 轮廓和 36 px 点击区。原始大小使用 1:1，适应窗口使用四角框，两种翻转使用不同镜像符号。

## Risks / Trade-offs

- 把真实 Windows 环境差异、失败场景与证据写入交接。
