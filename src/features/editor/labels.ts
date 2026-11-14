const en = {
  editor: 'Edit image', undo: 'Undo', redo: 'Redo', cancel: 'Cancel', save: 'Save',
  saveAs: 'Save As…', discard: 'Discard', leave: 'Save your edits before leaving?',
  overwrite: 'Replace the original image?', brush: 'Brush', shape: 'Shape',
  crop: 'Crop', mosaic: 'Mosaic', transform: 'Transform', color: 'Color', width: 'Width',
  height: 'Height', opacity: 'Opacity', tip: 'Brush tip', round: 'Round', square: 'Square',
  line: 'Line', arrow: 'Arrow', rectangle: 'Rectangle', ellipse: 'Ellipse',
  ratio: 'Aspect ratio', free: 'Free', applyCrop: 'Apply crop', radius: 'Brush radius',
  block: 'Block size', pixel: 'Pixelate', blur: 'Gaussian blur', fill: 'Color fill',
  noise: 'Noise', circle: 'Circle', glass: 'Stained glass', rotateLeft: 'Rotate left 90°',
  rotateRight: 'Rotate right 90°', flipX: 'Flip horizontal', flipY: 'Flip vertical',
  resize: 'Resize', applyResize: 'Apply size', failed: 'The edit could not be completed.',
  vector: 'Vector images cannot be edited.', animated: 'Animated images cannot be edited.',
  pan: 'Pan', loading: 'Rendering…',
};
const zh: Record<keyof typeof en, string> = {
  editor: '编辑图片', undo: '撤销', redo: '重做', cancel: '取消', save: '保存',
  saveAs: '另存为…', discard: '放弃', leave: '离开前保存编辑结果？',
  overwrite: '覆盖原图片？', brush: '画笔', shape: '形状', crop: '裁剪',
  mosaic: '马赛克', transform: '变换', color: '颜色', width: '宽度', height: '高度',
  opacity: '透明度', tip: '笔刷形状', round: '圆形', square: '方形',
  line: '直线', arrow: '箭头', rectangle: '矩形', ellipse: '椭圆',
  ratio: '裁剪比例', free: '自由', applyCrop: '应用裁剪', radius: '笔刷半径',
  block: '块大小', pixel: '像素马赛克', blur: '高斯模糊', fill: '颜色填充',
  noise: '噪点', circle: '圆形马赛克', glass: '彩色玻璃', rotateLeft: '向左旋转 90°',
  rotateRight: '向右旋转 90°', flipX: '水平翻转', flipY: '垂直翻转',
  resize: '调整尺寸', applyResize: '应用尺寸', failed: '无法完成此编辑操作。',
  vector: '矢量格式暂不支持编辑。', animated: '动图暂不支持编辑。',
  pan: '平移', loading: '正在渲染…',
};
export type Label = keyof typeof en;
export function editorLabel(locale: 'zh-CN' | 'en', key: Label) { return (locale === 'en' ? en : zh)[key]; }
