# migration-acceptance Specification

## Purpose
TBD - created by archiving change picglance-acceptance. Update Purpose after archive.

## Requirements

### Requirement: 全量迁移覆盖
新版 SHALL 覆盖旧版规格和现有实现全部用户功能。验收 SHALL 记录每项场景、实测证据和冲突处置。

#### Scenario: 真实图片完整工作流
- **WHEN** 用户从资源管理器打开图片，编辑、另存并压缩到目标大小
- **THEN** 原文件保持完整、另存文件可重新打开，压缩结果显示实际大小和达标状态
