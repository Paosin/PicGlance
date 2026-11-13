# PicGlance 看图交接

日期 2026-11-14。G0 已按基线交接放行。工作区为 PicGlance `dev`。

## 实现

- `src/features/viewer/` 提供看图画布、目录翻图、虚拟化缩略图条、适应窗口、鼠标锚定缩放、原始大小、预览变换、全屏、状态栏、EXIF/GPS 面板和中英文反馈。缩略图仅加载可见区及两侧预取范围，两路并发，最多缓存 96 项和 16 MiB。换目录、关闭缩略图条和后续请求会取消或丢弃过期工作。
- `src-tauri/src/viewer/` 提供可信路径授权、后台目录读取和解码、内容识别、只读预览与原图字节资源、动画、EXIF 和 Windows 剪贴板。原图字节按打开时快照保留，预览变换仅作用于显示。静态图打开时按窗口尺寸输出预览，缩放需要更清晰像素时才请求完整分辨率。
- 基线对话负责共享依赖、命令注册、ACL、窗口接线、启动和原生拖放桥接。剪贴板无源文档由 Viewer 交付，基线连接 Editor 的另存入口。跨包变更通过基线对话协调。

## 验证

| 命令 | 结果 |
| --- | --- |
| `node --test src/features/viewer/model.test.ts` | 5 项通过，覆盖旋转后适配、锚点缩放、预览矩阵、目录循环及删除、过期请求 |
| `npm test` | 6 项共享测试通过 |
| `cargo test --locked --offline --all-features` | 21 项通过，其中 Viewer 11 项。另有 1 项 Editor 旧版样本测试由测试定义标记 ignored |
| `npm run tauri -- build --no-bundle -- --locked --offline` | TypeScript、Vite 生产构建和 Windows x64 Release exe 构建通过。用 `CARGO_TARGET_DIR=target/viewer-verification` 避免共享 exe 被并行测试占用 |
| `python -X utf8 src/features/viewer/windows-smoke.py target/viewer-verification/release/picglance.exe --clipboard` | Windows 11 真实 Release 窗口通过。需要交互桌面、Pillow 和 psutil。可选参数会把系统剪贴板设为合成测试图 |
| `openspec validate picglance-viewer --strict` | 通过 |

Windows 冒烟脚本从进程启动返回后计时。中文空格路径 JPEG 约 33 ms 窗口可见、569 ms 首图。超过 260 字符的 PNG 路径约 33 ms、556 ms。7200×4800 JPEG 约 32 ms、701 ms，采样先观测到未呈现图片的窗口，再观测到图像像素。上述为单轮冒烟结果。

Rust 动画夹具确认 GIF、APNG、WebP 各有两帧，延时为 80/130 ms，有限循环字段及首帧缩略图正确。Windows 合成动画使用 140/260 ms，屏幕采样观察到 GIF 6 次、APNG 和 WebP 各 4 次交替颜色，均在末帧停留。打开淡入影响首帧的可见时长。Windows 同轮还验证 GPS 警示、缩略图显示、目录按键及删除后的恢复、F11/Esc、预览水平翻转后剪贴板复制仍得到原始像素、Ctrl+S 禁用和 Ctrl+Shift+S 原生另存对话框。

## 实现边界与下游

- 文件上限沿用旧版 80 × 1024² 字节，单边上限 32,768 像素。8 位顺序式灰度/RGB JPEG 预览上限 200 MP，采用解码内降采样。渐进式 JPEG、其他栅格及复制和原始分辨率等完整像素操作保留 48 MP 上限，栅格输出缓冲上限 256 MiB。动画限制为 20,000 帧和累计 256 MP，保留原始码流，由 WebView2 按源帧延时与循环字段播放。SVG 在 Rust 栅格化，支持系统字体与有界内嵌栅格，外部引用不读取。
- Editor 通过 `Viewer::editor_input(document_id)` 读取可信原图快照及源路径。Compression 使用 `onDocument(document, original)` 返回的原图资源 ID，经 `viewer_resource` 读取，预览资源用于显示。Viewer 保留最近两份文档资源，长期任务应在开始时取得自己的快照。
- `WINDOWS-DEFERRED`，资源管理器真实拖放手势、文件关联和单实例转发，由 `picglance-acceptance` 2.1、2.3 联验。Viewer 原生拖放桥接已启用。Windows 另存冒烟验证到对话框出现并取消，实际输出文件与重新打开由 2.1、2.2 验收。
- `WINDOWS-DEFERRED`，不同 DPI、系统减少动态效果与 UI 键盘矩阵由 `picglance-acceptance` 2.2、2.3 验收。冷缓存、稳态和大图峰值内存、切换及动图性能由 2.4 验收，沿用原 G2 预算。

## 验收反馈复测

- EXIF 面板恢复文件名编辑。省略扩展名时保留原扩展名，显式填写时可用受支持的扩展名，文件内容保持原样。Windows 同目录移动使用不覆盖选项。同名失败保留两份文件；成功后更新当前文档路径、目录排序与缩略图选中。Rust Viewer 13 项通过，Windows Release `windows-smoke.py <exe> --rename` 通过按钮提交、同名保护及扩展名保留。
- Windows Release `windows-smoke.py <exe> --g2` 以独立便携实例复测合成纯色、8 位基线 4:2:0 JPEG。进程树每轮在首图后继续采样约 750 ms，RSS 是各进程工作集之和，private 是私有提交，两项各取采样最大值。

| 样本 | 文件字节 | 窗口 / 首图 | 主进程 RSS / private 峰值 | 整树 RSS / private 峰值 |
| --- | ---: | ---: | ---: | ---: |
| 7,200×4,800 | 540,630 | 55 / 761 ms | 41.6 / 15.7 MiB | 367.8 / 213.2 MiB |
| 20,000×10,000 | 3,125,629 | 39 / 700 ms | 68.3 / 51.2 MiB | 370.8 / 230.4 MiB |

20,000×10,000 样本使用解码内缩放，64 MiB 解码输出限制包含其缩小后的像素缓冲。此单轮测试覆盖指定基线 JPEG 和峰值对照。渐进式 JPEG 的解码器仍可能分配全尺寸系数，48 MP 以上的渐进式 JPEG、PNG、TIFF 等是否纳入验收已提交基线协调。50 MB 稳态预算仍由 G2 核验。

## G2 翻图与动图量化

Windows 11 build 26200、WebView2 157.0.4341.46。下表两轮和超大图格式边界使用同一 Windows Release，EXE SHA-256 `8e2f7dc6bbfc3d22f73342df2884f2455cc44f15df351234e377d1d4c03b6fb3`。复测命令如下，需要交互桌面、Pillow 和 psutil。

```powershell
python -B -X utf8 src/features/viewer/windows-performance.py target/viewer-verification/release/picglance.exe --json target/viewer-performance-repeat.json
python -B -X utf8 src/features/viewer/windows-performance.py target/viewer-verification/release/picglance.exe --only limits --json target/viewer-performance-limits.json
```

每组六张同尺寸合成图，前台注入右方向键并循环，按键起始到中心点首次出现目标色计时。采样使用 Windows GDI GetPixel，目标色阈值允许捕获淡入早期，时间包含桌面合成与采样延迟。按键约每 200 ms 发一次，前次结果至少有 110 ms 完成淡入。每组在首图后等待 350 ms，超时 2 s 后停止该组。缩略图关闭，文件为新生成，未清除 OS 文件缓存。P95 用最近秩法，未捕获样本单列。

| 样本 | 第一轮 P50 / P95 / 最大 | 第一轮异常 | 第二轮 P50 / P95 / 最大 | 第二轮超 100 ms |
| --- | ---: | --- | ---: | ---: |
| 800×600 JPEG，约 0.30 MB/张 | — | 第一次按键未捕获目标像素，其余 17 次未执行，原因未定 | 62.8 / 82.4 / 82.4 ms | 0/18 |
| 4000×3000 JPEG，约 7.49 MB/张 | 81.0 / 82.7 / 82.7 ms | 0/12 超时 | 80.8 / 84.5 / 84.5 ms | 0/12 |
| 1600×1200 PNG，约 5.13 MB/张 | 76.6 / 109.7 / 109.7 ms | 2/12 超时，105.9、109.7 ms | 66.6 / 82.3 / 82.3 ms | 0/12 |

初始 G1 二进制的 4000×3000 JPEG 为 12/12 超 100 ms、P95 173.3 ms，PNG 为 12/12、P95 116.5 ms。Viewer 现从目录切图直接启动解码，静态预览采用更快的 PNG 编码。后台预解码当前方向的相邻图，缓存至多 2 项和 32 MiB。打开时重新读文件并逐字节确认缓存有效，文件替换和删除不使用旧预览。动图播放期间不启动相邻图预解码。15 项 Viewer Rust 测试及 5 项前端模型测试通过。已记录的超时和未捕获样本保留，100 ms 项尚无全通过证据。

动图各 12 帧，640×480，每帧独立中心色，源循环无限。首帧出现后等待 200 ms 再采样 2.5 s。仅记录颜色变化，序号跳跃视为至少一帧未被观察到。最大采样间隔为 17.9–20.2 ms。该方法测桌面可见帧，无法区分 WebView2 内部丢帧与合成后短暂显示被漏采。最长帧间隔包含桌面刷新量化。

| 格式 | 源 FPS | 两轮观测 FPS | 两轮跳帧事件 | 两轮最长间隔 |
| --- | ---: | ---: | ---: | ---: |
| GIF | 25.00 | 23.76 / 22.80 | 4 / 6 | 85.1 / 84.1 ms |
| APNG | 29.41 | 29.61 / 29.80 | 0 / 0 | 66.4 / 66.7 ms |
| WebP | 29.41 | 29.79 / 29.79 | 0 / 0 | 66.5 / 66.7 ms |

GIF 无掉帧项未通过。APNG 和 WebP 的结论限于本轮样本，开发过程中的另一构建曾观测到 WebP 跳帧。原始 JSON 包含按键延迟、帧序列和采样间隔，位于 `target/viewer-performance-final.json`、`target/viewer-performance-repeat.json` 与 `target/viewer-performance-limits.json`。G2 内存指标本轮暂缓。

同一 Release 实测打开 20,000×10,000 的灰度 PNG（219,269 字节）、Deflate TIFF（300,144 字节）和渐进式 JPEG（782,582 字节），三者的日志均返回 `dimensions`。200 MP 降采样目前仅用于 8 位顺序式 JPEG 预览。其余栅格及全尺寸操作仍受 48 MP 等解码限制，其他格式的 200 MP 支持未纳入此次修复。

