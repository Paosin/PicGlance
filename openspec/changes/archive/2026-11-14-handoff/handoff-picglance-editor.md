# PicGlance 编辑器交接

2026-11-14。变更 `picglance-editor`，工作区 `dev`。G0 按基线交接中的 Tauri 内存例外放行，G2 保留原性能指标。

## 实现

- `src/features/editor/` 提供编辑界面、缩放和平移、当前图像坐标映射、操作历史、双语工具和保存对话框。历史包含画笔、直线、箭头、矩形、椭圆、四档裁剪比例、六种马赛克、双向旋转、双向翻转和调整宽高。撤销后追加编辑会清除重做分支，返回已保存版本会清除未保存状态。
- `src-tauri/src/editor/` 消费公共 `EditorInput`，按 EXIF 方向解码 Viewer 提供的原图字节。预览和导出均由 Rust 按操作序列渲染。另存以同目录临时文件原子发布，拒绝已有文件和原文件别名。覆盖校验源字节，成功后刷新用于下次校验的字节。JPEG 输出质量 92，透明转 JPEG 时合成到白底。PNG 导出保留透明度。SVG 和多帧图像禁用编辑。
- `src/features/editor/backend.ts` 通过文档 ID 打开会话。基线负责命令注册、应用壳、窗口事件与设置接线。Viewer 负责授权文档 ID 与可信原图快照。保存路径由 Rust 原生选择器取得。
- `editor_export` 用 `EditedResources::publish_png` 发布受控 `ImageResource`。压缩接线应在销毁编辑会话前调用 `exportImage()`，并在 Rust 端通过共享资源 ID 读取 PNG。

## 验证

| 命令 | 结果 |
| --- | --- |
| `npm test` | 6 项共享前端测试通过 |
| `node --test src/features/editor/session.test.ts` | 6 项编辑测试通过 |
| `cargo test --locked --offline --all-features` | 21 项通过，旧版样本测试另跑 |
| `cargo test --locked --offline --all-features real_images_preserve_source_and_render_correct_pixels -- --ignored` | 通过。旧版 mozjpeg 样本 `testorig.jpg` 输出 149×227 JPEG，核对画笔与旋转后的像素。透明 `glass.png` 验证翻转位置及逐像素 RGBA。两份输入的原始字节相同 |
| `node src/features/editor/smoke.test.mjs` | Windows Edge 通过。基线 4000×3000 JPEG 在 400% 缩放并平移后映射到预期像素，覆盖取消和确认、撤销重做、离开另存通过。保存与渲染通过测试替身，实盘见下两项 |
| `npm run tauri -- build --no-bundle -- --locked` | Windows Release 构建成功，包含 TypeScript 检查与 Vite 构建 |
| `node src/features/editor/smoke.test.mjs target/release/picglance.exe` | Windows Release 通过。打开旧版真实 JPEG 的临时副本，落笔后从原生文件选择器另存 PNG，输出指定像素为 RGBA (255, 81, 60, 255)。关闭提示可取消，取消覆盖和另存后原 JPEG 字节相同，确认覆盖后文件改变 |
| `node src/features/editor/smoke.test.mjs target/release/picglance.exe svg` | Windows Release 通过。SVG 的编辑按钮禁用，按 B 出现本地化提示，未进入编辑 |
| `openspec validate picglance-editor --strict` | 通过 |

Rust 工具链可使用工作区已有的 `.foundation-tools/cargo` 和 `.foundation-tools/rustup`。浏览器和 Release 测试需要交互式 Windows 环境，测试只操作自己的窗口和临时样本副本。

## 旧版画笔设置兼容复测

2026-11-14。基线把 Desktop 持久化的 `brushColor` 和 `brushThickness` 传入编辑器。每次进入编辑时，画笔控件和实际像素绘制采用当前设置。

`node src/features/editor/smoke.test.mjs target/release/picglance.exe brush-settings` 在 Windows Release 通过。测试从临时便携 `settings.ini` 加载 `#29B45D` 和 17 px，检查控件及画笔内外像素，通过原生选择器另存为 227×149 BMP，并核对 JPEG 原件字节。用同一便携应用重开 BMP、再次进入编辑后，默认颜色和粗细仍正确。

## 下游联调

编辑包无独立实现阻塞。G1/G2 仍需从应用壳串联资源管理器打开、翻图、编辑、另存和目标体积压缩。压缩路由的编辑导出消费由基线与压缩包协调，验收对应 `picglance-acceptance` 2.1。跨 DPI、跨屏与多种真实窗口关闭和切图入口的完整矩阵在其 2.2、2.3 验收。大图操作会重放编辑序列并编码 PNG 预览，长历史与大图内存需纳入 2.4 性能验收。
