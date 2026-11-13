import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { test } from 'node:test';
import type { Icon, IconName } from '../../../shared/contracts.ts';
import { actionForKey, translate } from '../../../shared/preferences.ts';

const assetModule = new URL('../index.ts', import.meta.url);
const baselineIcons = [
  'open', 'settings', 'minimize', 'maximize', 'restore', 'close', 'minus', 'plus',
  'fit', 'actual', 'thumbnails', 'info', 'fullscreen', 'rotate', 'flip', 'image',
] satisfies IconName[];

test('provides distinct outline paths for the existing Icon contract and new tools', async () => {
  assert.ok(existsSync(assetModule), 'missing icon asset registry');
  const { iconPaths } = await import('../index.ts');
  for (const name of [...baselineIcons, 'edit', 'compress', 'brush', 'crop', 'mosaic', 'theme', 'language'] as const) {
    assert.match(iconPaths[name], /^M/, name);
  }
  const critical = [iconPaths.minus, iconPaths.plus, iconPaths.fit, iconPaths.actual,
    iconPaths.rotateRight, iconPaths.rotateLeft, iconPaths.flipHorizontal, iconPaths.flipVertical,
    iconPaths.info, iconPaths.thumbnails, iconPaths.fullscreen];
  assert.equal(new Set(critical).size, critical.length);
});

test('renders themed decorative and localized standalone SVGs through the public Icon signature', async t => {
  assert.ok(existsSync(assetModule), 'missing Icon implementation');
  const { createIcon } = await import('../index.ts');
  class ElementFixture {
    attributes = new Map<string, string>();
    children: ElementFixture[] = [];
    namespace: string;
    constructor(namespace: string) { this.namespace = namespace; }
    setAttribute(key: string, value: string) { this.attributes.set(key, value); }
    append(child: ElementFixture) { this.children.push(child); }
  }
  const documentFixture = { createElementNS: (namespace: string) => new ElementFixture(namespace) };
  Object.defineProperty(globalThis, 'document', { configurable: true, value: documentFixture });
  t.after(() => Reflect.deleteProperty(globalThis, 'document'));

  for (const locale of ['en', 'zh-CN'] as const) {
    const render: Icon = createIcon(key => translate(locale, key));
    const decorative = render({ name: 'fit', size: 20 }) as unknown as ElementFixture;
    assert.equal(decorative.namespace, 'http://www.w3.org/2000/svg');
    assert.equal(decorative.attributes.get('viewBox'), '0 0 20 20');
    assert.equal(decorative.attributes.get('width'), '20');
    assert.equal(decorative.attributes.get('stroke'), 'currentColor');
    assert.equal(decorative.attributes.get('aria-hidden'), 'true');
    assert.match(decorative.children[0].attributes.get('d') ?? '', /^M/);

    const labeled = render({ name: 'fit', size: 32, label: 'fitTip' }) as unknown as ElementFixture;
    assert.equal(labeled.attributes.get('height'), '32');
    assert.equal(labeled.attributes.get('role'), 'img');
    assert.equal(labeled.attributes.has('aria-hidden'), false);
    assert.equal(labeled.attributes.get('aria-label'), locale === 'en' ? 'Fit to window (Ctrl+0)' : '适应窗口 (Ctrl+0)');
  }
});

test('keeps the existing localized toolbar hints aligned with the keyboard dispatcher', () => {
  const actions = [
    ['fitTip', 'Ctrl+0', 'fit'], ['actualTip', 'Ctrl+1', 'actual'],
    ['rotateTip', 'R', 'rotateRight'], ['flipTip', 'H', 'flipHorizontal'],
    ['infoTip', 'I', 'info'], ['thumbnailsTip', 'T', 'thumbnails'],
    ['fullscreenTip', 'F11', 'fullscreen'], ['brushTip', 'B', 'brush'],
  ] as const;
  for (const [message, chord, action] of actions) {
    for (const locale of ['en', 'zh-CN'] as const) {
      assert.ok(translate(locale, message).endsWith(`(${chord})`), `${locale} ${action}`);
    }
    assert.equal(actionForKey({ key: chord.replace('Ctrl+', ''), ctrlKey: chord.startsWith('Ctrl+') }), action);
  }
  assert.equal(actionForKey({ key: 'R', shiftKey: true }), 'rotateLeft');
  assert.equal(actionForKey({ key: 'V' }), 'flipVertical');
});
