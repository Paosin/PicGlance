const en = {
  open: 'Open image', close: 'Close details', previous: 'Previous image (←)', next: 'Next image (→)',
  thumbnails: 'Thumbnails (T)', info: 'Image details (I)', fit: 'Fit to window (Ctrl+0)',
  actual: 'Actual size (Ctrl+1)', zoomOut: 'Zoom out', zoomIn: 'Zoom in', rotate: 'Rotate (R / Shift+R)',
  flip: 'Flip horizontally (H)', fullscreen: 'Full screen (F11)', clipboard: 'Clipboard image',
  empty: 'Drop an image here or open a file', loading: 'Loading image…', noMetadata: 'No metadata',
  gps: 'Contains GPS location information', tooLarge: 'File is too large ({size} MB). The limit is 80 MB.',
  'invalid-input': 'The image is damaged or could not be decoded.', unsupported: 'Unsupported image format.',
  missing: 'The file was moved, renamed or deleted.', dimensions: 'The image exceeds the decoding size limit.',
  io: 'The requested operation failed. Check that the file or clipboard is accessible.',
  busy: 'The viewer is busy. Try again.', cancelled: 'Operation cancelled.', 'clipboard-empty': 'The clipboard contains no image.',
  filename: 'File name', rename: 'Rename', 'name-exists': 'A file with that name already exists.',
  'invalid-name': 'Enter a valid file name.', 'source-changed': 'The file has changed. Reopen it before renaming.',
};

const zh: typeof en = {
  open: '打开图片', close: '关闭详情', previous: '上一张 (←)', next: '下一张 (→)',
  thumbnails: '缩略图 (T)', info: '图片详情 (I)', fit: '适应窗口 (Ctrl+0)',
  actual: '原始大小 (Ctrl+1)', zoomOut: '缩小', zoomIn: '放大', rotate: '旋转 (R / Shift+R)',
  flip: '水平翻转 (H)', fullscreen: '全屏 (F11)', clipboard: '剪贴板图片',
  empty: '拖入图片，或选择文件打开', loading: '正在加载图片…', noMetadata: '无元数据',
  gps: '含 GPS 位置信息', tooLarge: '文件过大（{size} MB），最大支持 80 MB',
  'invalid-input': '图片已损坏或无法解码。', unsupported: '不支持此图片格式。',
  missing: '文件已移动、改名或删除。', dimensions: '图片超过解码尺寸限制。',
  io: '操作失败，请检查文件或剪贴板是否可访问。', busy: '查看器正忙，请重试。',
  cancelled: '操作已取消。', 'clipboard-empty': '剪贴板中没有图片。',
  filename: '文件名', rename: '重命名', 'name-exists': '同名文件已存在。',
  'invalid-name': '请输入有效的文件名。', 'source-changed': '文件已更改，请重新打开后再重命名。',
};

export const labels = { en, 'zh-CN': zh };

export const metadataLabels: Record<keyof typeof labels, Record<string, string>> = {
  en: { Make: 'Camera maker', Model: 'Camera model', LensModel: 'Lens', FNumber: 'Aperture',
    ExposureTime: 'Exposure', PhotographicSensitivity: 'ISO', ISOSpeed: 'ISO', FocalLength: 'Focal length',
    DateTimeOriginal: 'Taken', DateTime: 'Modified', GPSLatitude: 'Latitude', GPSLongitude: 'Longitude', GPSAltitude: 'Altitude' },
  'zh-CN': { Make: '设备厂商', Model: '设备型号', LensModel: '镜头', FNumber: '光圈',
    ExposureTime: '快门', PhotographicSensitivity: 'ISO', ISOSpeed: 'ISO', FocalLength: '焦距',
    DateTimeOriginal: '拍摄时间', DateTime: '修改时间', GPSLatitude: '纬度', GPSLongitude: '经度', GPSAltitude: '海拔' },
};

