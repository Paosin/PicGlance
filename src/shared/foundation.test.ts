import assert from 'node:assert/strict';
import { test } from 'node:test';
import { actionForKey, resolveTheme, translate } from './preferences.ts';
import type { ImageDocument, WindowControls } from './contracts.ts';
import { toggleFullscreen } from './window.ts';
import { memoryTarget } from './memory.ts';
import fixture from './image-document.fixture.json' with { type: 'json' };

test('shares document metadata and an opaque binary preview handle with Rust', () => {
  const document: ImageDocument = fixture;
  assert.equal(document.width * document.height, 12_000_000);
  assert.deepEqual(document.preview, { id: 'preview-1', mimeType: 'image/jpeg' });
});

test('starts in graphite and only consults Windows in system mode', () => {
  assert.equal(resolveTheme(undefined, false), 'dark');
  assert.equal(resolveTheme('light', true), 'light');
  assert.equal(resolveTheme('system', true), 'dark');
  assert.equal(resolveTheme('system', false), 'light');
});

test('looks up the same controls in both languages', () => {
  assert.equal(translate(undefined, 'open'), '打开');
  assert.equal(translate('en', 'open'), 'Open');
  assert.equal(translate('en', 'thumbnailsTip'), 'Thumbnails (T)');
});

test('maps keyboard actions independently of locale and preserves form input', () => {
  assert.equal(actionForKey({ key: 'O', ctrlKey: true }), 'open');
  assert.equal(actionForKey({ key: 'R', shiftKey: true }), 'rotateLeft');
  assert.equal(actionForKey({ key: 'Z', ctrlKey: true, shiftKey: true }), 'redo');
  assert.equal(actionForKey({ key: 't' }, true), undefined);
  assert.equal(actionForKey({ key: 't', altKey: true }), undefined);
});

test('toggles fullscreen based on the native window state', async () => {
  let state = true;
  const window: Pick<WindowControls, 'isFullscreen' | 'setFullscreen'> = {
    isFullscreen: async () => state,
    setFullscreen: async value => { state = value; },
  };
  assert.equal(await toggleFullscreen(window), false);
  assert.equal(state, false);
  assert.equal(await toggleFullscreen(window), true);
  assert.equal(state, true);
});

test('uses a low memory target when empty or hidden and restores it for visible image work', () => {
  assert.equal(memoryTarget(false, false), 'low');
  assert.equal(memoryTarget(false, true), 'normal');
  assert.equal(memoryTarget(true, true), 'low');
});
