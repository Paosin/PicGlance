import { zh, en, type Message } from './messages.ts';

export type Locale = 'zh-CN' | 'en';
export type ThemeMode = 'dark' | 'light' | 'system';

export function resolveTheme(mode: ThemeMode | undefined, systemDark: boolean) {
  return mode === 'system' ? (systemDark ? 'dark' : 'light') : (mode ?? 'dark');
}

export function translate(locale: Locale | undefined, message: Message): string {
  return (locale === 'en' ? en : zh)[message];
}

const shortcuts = {
  arrowleft: 'previous', arrowright: 'next', 'ctrl+o': 'open', 'ctrl+0': 'fit',
  'ctrl+1': 'actual', f11: 'fullscreen', r: 'rotateRight', 'shift+r': 'rotateLeft',
  h: 'flipHorizontal', v: 'flipVertical', b: 'brush', 'ctrl+z': 'undo',
  'ctrl+y': 'redo', 'ctrl+shift+z': 'redo', 'ctrl+s': 'save',
  'ctrl+shift+s': 'saveAs', 'ctrl+c': 'copy', 'ctrl+v': 'paste',
  i: 'info', t: 'thumbnails', escape: 'escape', w: 'escape',
} as const;

export type ShortcutAction = (typeof shortcuts)[keyof typeof shortcuts];

export function actionForKey(
  event: Pick<KeyboardEvent, 'key'> & Partial<Pick<KeyboardEvent,
    'ctrlKey' | 'shiftKey' | 'altKey' | 'metaKey' | 'isComposing'>>,
  editing = false,
): ShortcutAction | undefined {
  if (editing || event.isComposing || event.altKey || event.metaKey) return;
  const chord = `${event.ctrlKey ? 'ctrl+' : ''}${event.shiftKey ? 'shift+' : ''}${event.key.toLowerCase()}`;
  return Object.hasOwn(shortcuts, chord) ? shortcuts[chord as keyof typeof shortcuts] : undefined;
}
