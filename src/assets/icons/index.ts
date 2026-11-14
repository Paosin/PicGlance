import type { Icon, IconOptions } from '../../shared/contracts.ts';
import type { Message } from '../../shared/messages.ts';

export const iconSize = 20;
export const iconButtonSize = 36;

const rotateRight = 'M16.5 7.5A6.75 6.75 0 1 0 16 14M17 3v5h-5';
const flipHorizontal = 'M10 2.5v15M7 5l-4 10h4ZM13 5l4 10h-4Z';
const close = 'M5 5l10 10M15 5 5 15';

export const iconPaths = {
  open: 'M2.5 7V5h5l2 2h7.5v2M2.5 7v9h13l2.5-6h-13l-2.5 6',
  settings: 'M8 3h4l.8 2.2 2.3.4 2 3.4-1.5 1.7.8 2.2-2 2-2.3-.5L11 17H8.5l-1-2.3-2.4-.5-2-3.4L4.6 9l-.8-2.2 2-2 2.2.5ZM12.3 10a2.3 2.3 0 1 1-4.6 0 2.3 2.3 0 1 1 4.6 0',
  minimize: 'M4 10h12',
  maximize: 'M4 4h12v12H4Z',
  restore: 'M7 4h9v9M4 7h9v9H4Z',
  close,
  minus: 'M4 10h12',
  plus: 'M4 10h12M10 4v12',
  fit: 'M7 3H3v4m10-4h4v4m0 6v4h-4m-6 0H3v-4M6.5 7.5h7v5h-7Z',
  actual: 'M3 7.5l2-1v7m-2 0h4M10 8.5h.1m-.1 3h.1m3-4 2-1v7m-2 0h4',
  thumbnails: 'M2.5 5.5h15v9h-15Zm5 0v9m5-9v9',
  info: 'M17 10a7 7 0 1 1-14 0 7 7 0 1 1 14 0ZM10 9v4.5m0-7h.1',
  fullscreen: 'M7.5 3H3v4.5M12.5 3H17v4.5m0 5V17h-4.5m-5 0H3v-4.5',
  rotate: rotateRight,
  flip: flipHorizontal,
  image: 'M3 4h14v12H3ZM3 13l4-4 3.5 3.5 2.5-2 4 4m-4-7h.1',
  rotateRight,
  rotateLeft: 'M3.5 7.5A6.75 6.75 0 1 1 4 14M3 3v5h5',
  flipHorizontal,
  flipVertical: 'M2.5 10h15M5 7l10-4v4Zm0 6 10 4v-4Z',
  previous: 'M12.5 4.5 7 10l5.5 5.5',
  next: 'M7.5 4.5 13 10l-5.5 5.5',
  back: 'M16 10H4m5-5-5 5 5 5',
  copy: 'M8 7h8v10H8ZM5 13H4V3h8v1',
  paste: 'M7 4H4v13h12V4h-3M7 3h6v3H7Zm0 7h6m-6 3h4',
  edit: 'M4 12.5 13.7 3l3.3 3.3-9.5 9.7-4.5 1ZM11.5 5.2l3.3 3.3',
  brush: 'M8.5 11.5 15 3.5a1.5 1.5 0 0 1 2 2l-7.5 7.2M9.5 12.7c1 3-2.3 4.9-6.5 3.8 2-1 1.1-2.5 2.5-4 1.2-1.2 3-1.2 4 .2Z',
  shapes: 'M3 3.5h7v7H3Zm14.5 10a4 4 0 1 1-8 0 4 4 0 1 1 8 0Z',
  crop: 'M6 2.5V14h11.5M2.5 6H14v11.5',
  mosaic: 'M3.5 3.5h13v13h-13Zm0 6.5h13M10 3.5v13m3.25-3.25h.1m-6.6-6.5h.1',
  resize: 'M3.5 10v6.5H10M10 3.5h6.5V10M6 14l8-8',
  undo: 'M7 3.5 3 7.5l4 4M3 7.5h8.5a4.5 4.5 0 0 1 0 9H8',
  redo: 'M13 3.5l4 4-4 4m4-4H8.5a4.5 4.5 0 0 0 0 9H12',
  save: 'M3.5 3.5h10l3 3v10h-13ZM7 3.5v5h6v-5m-6 13v-5h6v5',
  saveAs: 'M12 16.5H3.5v-13h10l3 3V10M7 3.5v5h6v-5m-6 13v-5h4M14 14h4m-2-2v4',
  cancel: close,
  apply: 'M4 10l4 4 8-8',
  compress: 'M3 3h4m-4 0v4m14-4h-4m4 0v4M3 17h4m-4 0v-4m14 4h-4m4 0v-4M6.5 6.5 9 9m4.5-2.5L11 9m-4.5 4.5L9 11m4.5 2.5L11 11',
  compare: 'M3 4h14v12H3Zm7 0v12m-5-4 2-2 3 2m2-1 2-2 3 3',
  lock: 'M5 9h10v8H5Zm2 0V6a3 3 0 0 1 6 0v3m-3 3v2',
  unlock: 'M5 9h10v8H5Zm2 0V6a3 3 0 0 1 5.6-1.5M10 12v2',
  theme: 'M15.5 13A6.5 6.5 0 0 1 7 4.5 6.5 6.5 0 1 0 15.5 13Z',
  language: 'M17 10a7 7 0 1 1-14 0 7 7 0 1 1 14 0ZM3 10h14M10 3c-4 4-4 10 0 14 4-4 4-10 0-14Z',
  association: 'M10 6l2-2a3 3 0 0 1 4 4l-3 3a3 3 0 0 1-4 0m1 3-2 2a3 3 0 0 1-4-4l3-3a3 3 0 0 1 4 0',
  update: 'M16.5 9a6.6 6.6 0 1 0-1 5M17 4.5V9h-4.5M10 6.5v4l2 1.5',
  tray: 'M3 10l2-5h10l2 5v6H3Zm0 0h4l1.5 2h3l1.5-2h4',
} as const satisfies Record<string, string>;

export type AssetIconName = keyof typeof iconPaths;
type AssetIconOptions = Omit<IconOptions, 'name'> & { name: AssetIconName };
const svgNamespace = 'http://www.w3.org/2000/svg';

export function createIcon(translate: (message: Message) => string) {
  const render = ({ name, size, label }: AssetIconOptions): SVGSVGElement => {
    const svg = document.createElementNS(svgNamespace, 'svg');
    const attributes = {
      viewBox: '0 0 20 20',
      width: size,
      height: size,
      fill: 'none',
      stroke: 'currentColor',
      'stroke-width': 1.5,
      'stroke-linecap': 'round',
      'stroke-linejoin': 'round',
      focusable: 'false',
    };
    for (const [key, value] of Object.entries(attributes)) svg.setAttribute(key, String(value));

    if (label === undefined) {
      svg.setAttribute('aria-hidden', 'true');
    } else {
      svg.setAttribute('role', 'img');
      svg.setAttribute('aria-label', translate(label));
    }

    const path = document.createElementNS(svgNamespace, 'path');
    path.setAttribute('d', iconPaths[name]);
    svg.append(path);
    return svg;
  };
  return render satisfies Icon;
}
