import type { WindowControls } from './contracts.ts';

export async function toggleFullscreen(window: Pick<WindowControls, 'isFullscreen' | 'setFullscreen'>) {
  const next = !(await window.isFullscreen());
  await window.setFullscreen(next);
  return next;
}
