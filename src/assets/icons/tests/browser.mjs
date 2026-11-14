import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';
import { build } from 'esbuild';

const fixtureFiles = await build({
  stdin: { contents: `
    import * as icons from './src/assets/icons/index.ts';
    import * as preferences from './src/shared/preferences.ts';
    globalThis.iconFixture = { ...icons, ...preferences };
  `, resolveDir: process.cwd() },
  bundle: true, write: false, format: 'iife',
});
const css = await readFile('src/shared/shell.css', 'utf8');
const locales = ['zh-CN', 'en'];
const profile = await mkdtemp(join(tmpdir(), 'picglance-icons-edge-'));
const fixturePath = join(profile, 'fixture.html');
await writeFile(fixturePath, `<!doctype html><meta charset="utf-8"><style>${css}</style><body><script>${fixtureFiles.outputFiles[0].text}</script>`);
const fixtureUrl = pathToFileURL(fixturePath).href;
const edge = process.env.PICGLANCE_EDGE ?? 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe';
const browser = spawn(edge, ['--headless=new', '--no-sandbox', '--disable-gpu', '--no-first-run',
  '--remote-debugging-port=0', '--user-data-dir=' + join(profile, 'edge'), fixtureUrl], {
  windowsHide: true, stdio: 'ignore',
});

async function waitFor(work) {
  for (let attempt = 0; attempt < 100; attempt++) {
    const result = await work().catch(() => undefined);
    if (result) return result;
    await delay(100);
  }
  throw new Error('Timed out waiting for Edge');
}

let socket;
try {
  const port = await waitFor(async () => (await readFile(join(profile, 'edge', 'DevToolsActivePort'), 'utf8')).split('\n')[0]);
  const page = await waitFor(async () => {
    const pages = await (await fetch('http://127.0.0.1:' + port + '/json/list')).json();
    return pages.find(page => page.type === 'page' && page.url === fixtureUrl);
  });
  socket = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });
  let nextId = 0;
  const pending = new Map();
  socket.addEventListener('message', event => {
    const { id, error, result } = JSON.parse(event.data);
    pending.get(id)?.(error, result);
    pending.delete(id);
  });
  function call(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = ++nextId;
      pending.set(id, (error, result) => error ? reject(new Error(error.message)) : resolve(result));
      socket.send(JSON.stringify({ id, method, params }));
    });
  }
  async function evaluate(work, ...args) {
    const expression = `(${work})(...${JSON.stringify(args)})`;
    const { result, exceptionDetails } = await call('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (exceptionDetails) throw new Error(exceptionDetails.exception?.description ?? exceptionDetails.text);
    return result.value;
  }

  await waitFor(() => evaluate(() => Boolean(globalThis.iconFixture)));
  for (const dpr of [1, 1.5, 2]) {
    await call('Emulation.setDeviceMetricsOverride', { width: 800, height: 400, deviceScaleFactor: dpr, mobile: false });
    for (const [theme, locale] of ['dark', 'light'].flatMap(theme => locales.map(locale => [theme, locale]))) {
      const fixture = await evaluate(async (theme, locale) => {
        const { createIcon, iconSize, iconButtonSize, translate, actionForKey } = globalThis.iconFixture;
        document.documentElement.lang = locale;
        document.documentElement.dataset.theme = theme;
        const toolbar = document.createElement('nav');
        toolbar.className = 'toolbar';
        toolbar.style.background = 'var(--surface)';
        document.body.replaceChildren(toolbar);
        const render = createIcon(key => translate(locale, key));
        const actions = [
          ['minus', 'zoomOut'], ['plus', 'zoomIn'], ['fit', 'fitTip', 'Ctrl+0', 'fit'],
          ['actual', 'actualTip', 'Ctrl+1', 'actual'], ['rotateRight', 'rotateTip', 'R', 'rotateRight'],
          ['rotateLeft', ['逆时针旋转', 'Rotate counterclockwise'], 'Shift+R', 'rotateLeft'],
          ['flipHorizontal', 'flipTip', 'H', 'flipHorizontal'],
          ['flipVertical', ['垂直翻转', 'Flip vertically'], 'V', 'flipVertical'],
          ['thumbnails', 'thumbnailsTip', 'T', 'thumbnails'], ['info', 'infoTip', 'I', 'info'],
          ['fullscreen', 'fullscreenTip', 'F11', 'fullscreen'], ['brush', 'brushTip', 'B', 'brush'],
        ];
        const dimensions = [];
        const names = [];
        const rasters = [];
        for (const [name, label, chord, action] of actions) {
          const button = document.createElement('button');
          const title = Array.isArray(label)
            ? label[locale === 'en' ? 1 : 0] + ' (' + chord + ')'
            : translate(locale, label);
          button.setAttribute('aria-label', title.replace(/ \([^()]+\)$/, ''));
          button.title = title;
          if (chord) {
            button.setAttribute('aria-keyshortcuts', chord.replace('Ctrl+', 'Control+'));
            const mapped = actionForKey({
              key: chord.split('+').at(-1), ctrlKey: chord.includes('Ctrl+'), shiftKey: chord.includes('Shift+'),
            });
            if (mapped !== action || !title.endsWith('(' + chord + ')')) throw new Error('Shortcut mismatch for ' + name);
          }
          if (['fit', 'thumbnails', 'info', 'fullscreen', 'brush'].includes(name)) {
            button.setAttribute('aria-pressed', 'true');
          }
          button.style.cssText = `width:${iconButtonSize}px;height:${iconButtonSize}px;padding:0`;
          const svg = render({ name, size: iconSize });
          button.append(svg);
          toolbar.append(button);
          dimensions.push([button.offsetWidth, button.offsetHeight, svg.getBoundingClientRect().width * devicePixelRatio]);
          names.push(button.getAttribute('aria-label'));

          const source = svg.cloneNode(true);
          source.setAttribute('color', getComputedStyle(svg).color);
          const url = URL.createObjectURL(new Blob([new XMLSerializer().serializeToString(source)], { type: 'image/svg+xml' }));
          const image = new Image();
          image.src = url;
          await image.decode();
          const canvas = document.createElement('canvas');
          canvas.width = canvas.height = iconSize * devicePixelRatio;
          const context = canvas.getContext('2d');
          context.drawImage(image, 0, 0, canvas.width, canvas.height);
          URL.revokeObjectURL(url);
          const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data;
          const mask = Array.from({ length: canvas.width * canvas.height }, (_, pixel) => Number(pixels[pixel * 4 + 3] > 127));
          if (mask.filter(Boolean).length < canvas.width) throw new Error('Missing outline for ' + name);
          rasters.push(mask.join(''));
        }
        const luminance = color => color.match(/\d+/g).slice(0, 3).map(value => Number(value) / 255)
          .map(value => value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4)
          .reduce((sum, value, i) => sum + value * [0.2126, 0.7152, 0.0722][i], 0);
        const contrast = [...toolbar.children].map(button => {
          const fg = luminance(getComputedStyle(button).color);
          const bg = luminance(getComputedStyle(button.hasAttribute('aria-pressed') ? button : toolbar).backgroundColor);
          return (Math.max(fg, bg) + 0.05) / (Math.min(fg, bg) + 0.05);
        });
        return { dimensions, names, distinct: new Set(rasters).size, contrast: Math.min(...contrast) };
      }, theme, locale);
      assert.equal(fixture.distinct, fixture.names.length);
      assert.ok(fixture.contrast >= 3, String(fixture.contrast));
      for (const dimensions of fixture.dimensions) assert.deepEqual(dimensions, [36, 36, 20 * dpr]);
      const { nodes } = await call('Accessibility.getFullAXTree');
      const names = nodes.filter(node => node.role?.value === 'button').map(node => node.name.value);
      assert.deepEqual(names, fixture.names);
      assert.equal(nodes.filter(node => node.role?.value === 'image' && !node.ignored).length, 0);
      assert.equal(nodes.filter(node => node.properties?.some(property => property.name === 'pressed')).length, 5);
      console.log(`${theme} ${dpr * 100}% ${locale}: ${names.length} distinct glyphs and accessible names, minimum contrast ${fixture.contrast.toFixed(2)}:1`);
    }
  }
} finally {
  socket?.close();
  browser.kill();
  await rm(profile, { recursive: true, force: true, maxRetries: 5 }).catch(() => {});
}
