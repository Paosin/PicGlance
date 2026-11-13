export const zh = {
  open: '打开', edit: '编辑', compress: '压缩', settings: '设置', back: '返回看图',
  minimize: '最小化', maximize: '最大化', restore: '还原', close: '关闭',
  empty: '打开一张图片', emptyHint: '使用 Ctrl+O 选择图片',
  editEmpty: '请先打开一张图片', editVector: '无法编辑矢量图片。', editError: '无法打开编辑会话。', compressEmpty: '打开图片后即可预览对比布局',
  original: '原图', result: '结果预览', imageFile: '图片文件',
  loading: '正在打开图片…', invalidImage: '无法显示这张图片。', saveError: '无法保存图片。', compressionError: '无法打开压缩对话框。',
  tooLarge: '图片超过 80 MB。', windowError: '窗口操作失败。',
  fitTip: '适应窗口 (Ctrl+0)', actualTip: '原始大小 (Ctrl+1)',
  zoomIn: '放大', zoomOut: '缩小', thumbnailsTip: '缩略图 (T)', infoTip: '图片信息 (I)',
  fullscreenTip: '全屏 (F11)', openTip: '打开 (Ctrl+O)',
  rotateTip: '顺时针旋转 (R)', flipTip: '水平翻转 (H)',
  theme: '主题', dark: '深色', light: '浅色', system: '跟随系统',
  language: '语言', tool: '当前工具', brushTip: '画笔 (B)', size: '大小',
  targetSize: '目标大小 (KB)', noMetadata: '无元数据', thumbnails: '缩略图',
  systemOpen: '用 PicGlance 打开', fileAssociation: 'PicGlance 图片文件',
} as const;

export type Message = keyof typeof zh;

export const en: Record<Message, string> = {
  open: 'Open', edit: 'Edit', compress: 'Compress', settings: 'Settings', back: 'Back to viewer',
  minimize: 'Minimize', maximize: 'Maximize', restore: 'Restore', close: 'Close',
  empty: 'Open an image', emptyHint: 'Press Ctrl+O to choose a file',
  editEmpty: 'Open an image first', editVector: 'Vector images cannot be edited.', editError: 'Could not open the editor.', compressEmpty: 'Open an image to preview the comparison layout',
  original: 'Original', result: 'Result preview', imageFile: 'Image files',
  loading: 'Opening image…', invalidImage: 'This image could not be displayed.', saveError: 'Could not save the image.', compressionError: 'Could not open the compression dialog.',
  tooLarge: 'The image exceeds 80 MB.', windowError: 'The window action failed.',
  fitTip: 'Fit to window (Ctrl+0)', actualTip: 'Actual size (Ctrl+1)',
  zoomIn: 'Zoom in', zoomOut: 'Zoom out', thumbnailsTip: 'Thumbnails (T)', infoTip: 'Image information (I)',
  fullscreenTip: 'Full screen (F11)', openTip: 'Open (Ctrl+O)',
  rotateTip: 'Rotate clockwise (R)', flipTip: 'Flip horizontally (H)',
  theme: 'Theme', dark: 'Dark', light: 'Light', system: 'Use system setting',
  language: 'Language', tool: 'Current tool', brushTip: 'Brush (B)', size: 'Size',
  targetSize: 'Target size (KB)', noMetadata: 'No metadata', thumbnails: 'Thumbnails',
  systemOpen: 'Open with PicGlance', fileAssociation: 'PicGlance image file',
};
