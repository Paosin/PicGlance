import { createIcon, iconSize } from '../assets/icons/index.ts';
import type { IconName } from './contracts.ts';
import { translate } from './preferences.ts';

const render = createIcon(message => translate(document.documentElement.lang === 'en' ? 'en' : 'zh-CN', message));

export function icon(name: IconName, size = iconSize): SVGSVGElement {
  return render({ name, size });
}
