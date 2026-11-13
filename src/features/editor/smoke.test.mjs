import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { mkdtemp, readFile, rm, copyFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { createServer as netServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';
import { createServer } from 'vite';

const native = process.argv[2];
const vector = process.argv[3] === 'svg';
const brushSettings = process.argv[3] === 'brush-settings';
const server = native ? null : await createServer({
  server: { host: '127.0.0.1', port: 1420, strictPort: true, hmr: false, watch: null },
});
await server?.listen();
const profile = await mkdtemp(join(tmpdir(), 'picglance-editor-edge-'));
const edge = process.env.PICGLANCE_EDGE ?? 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe';
const fixture = join(profile, vector ? 'fixture.svg' : 'fixture.jpg');
let nativePort;
let nativeExe = native ? resolve(native) : null;
if (native) {
  const listener = netServer();
  await new Promise(resolve => listener.listen(0, '127.0.0.1', resolve));
  nativePort = listener.address().port;
  await new Promise(resolve => listener.close(resolve));
  if (vector) {
    await writeFile(fixture, '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><rect width="20" height="20" fill="red"/></svg>');
  } else {
    await copyFile('../PicGlance-tmp/third_party/mozjpeg/testimages/testorig.jpg', fixture);
  }
  if (brushSettings) {
    nativeExe = join(profile, 'PicGlance.exe');
    await copyFile(resolve(native), nativeExe);
    await writeFile(join(profile, 'portable.marker'), '');
    await writeFile(join(profile, 'settings.ini'),
      '[general]\ncheckUpdates=false\n[editing]\nbrushColor=#29B45D\nbrushThickness=17\n');
  }
}
const browser = native ? spawn(nativeExe, [fixture], {
  env: { ...process.env, WEBVIEW2_USER_DATA_FOLDER: join(profile, 'webview'),
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${nativePort}` },
  stdio: ['ignore', 'ignore', 'pipe'],
}) : spawn(edge, ['--headless=new', '--no-sandbox', '--disable-gpu', '--no-first-run', '--remote-debugging-port=0',
  '--user-data-dir=' + profile, 'http://127.0.0.1:1420/src/features/editor/smoke.html'], { stdio: ['ignore', 'ignore', 'pipe'] });
let diagnostics = '';
browser.stderr.on('data', data => { diagnostics += data; });

async function until(work) {
  for (let attempt = 0; attempt < 100; attempt++) {
    const result = await work();
    if (result) return result;
    await delay(100);
  }
  throw new Error('Timed out waiting for browser state');
}

let socket;
try {
  const port = nativePort ?? await until(async () => {
    try { return (await readFile(join(profile, 'DevToolsActivePort'), 'utf8')).split('\n')[0]; }
    catch { return null; }
  });
  const target = await until(async () => {
    try {
      const targets = await (await fetch('http://127.0.0.1:' + port + '/json/list')).json();
      return targets.find(target => target.type === 'page' && (native ? target.url.includes('tauri.localhost') : target.url.includes('/smoke.html')));
    } catch { return null; }
  });
  socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });
  let next = 0;
  const pending = new Map();
  socket.addEventListener('close', event => {
    for (const complete of pending.values()) complete({ error: new Error(`DevTools disconnected (${event.code})`) });
  });
  socket.addEventListener('message', event => {
    const response = JSON.parse(event.data);
    pending.get(response.id)?.(response);
  });
  const call = (method, params) => new Promise((resolve, reject) => {
    const id = ++next;
    pending.set(id, ({ error, result }) => {
      pending.delete(id);
      if (error) reject(new Error(`${method}: ${error.message}\n${diagnostics}`));
      else resolve(result);
    });
    socket.send(JSON.stringify({ id, method, params }));
  });
  const evaluate = async expression => {
    const { result, exceptionDetails } = await call('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (exceptionDetails) throw new Error(exceptionDetails.exception?.description ?? exceptionDetails.text);
    return result.value;
  };

  if (native && vector) {
    await until(() => evaluate('document.querySelector(".filename")?.textContent === "fixture.svg"'));
    assert.equal(await evaluate('document.querySelector("#edit").getAttribute("aria-disabled")'), 'true');
    await evaluate('document.dispatchEvent(new KeyboardEvent("keydown", { key: "b", bubbles: true, cancelable: true }))');
    await until(() => evaluate('/矢量|Vector/i.test(document.querySelector("#app").textContent)'));
    assert.equal(await evaluate('Boolean(document.querySelector(".pg-editor-stage"))'), false);
    console.log('Native SVG edit button is disabled and B shows the localized restriction');
  } else if (native) {
    await until(() => evaluate('document.querySelector(".filename")?.textContent === "fixture.jpg"'));
    const key = key => evaluate(`document.dispatchEvent(new KeyboardEvent('keydown', { key: '${key}', bubbles: true, cancelable: true }))`);
    await until(() => evaluate('document.querySelector("#edit")?.getAttribute("aria-disabled") !== "true"'));
    await key('b');
    await until(() => evaluate('Boolean(document.querySelector(".pg-editor-stage canvas"))'))
      .catch(async error => {
        const status = await evaluate('Array.from(document.querySelectorAll("[role=status], .notice")).map(node => node.textContent)');
        throw new Error(`Editor failed to mount: ${JSON.stringify(status)}`, { cause: error });
      });
    const color = await evaluate('document.querySelector(".pg-editor-parameters input[type=color]").value');
    if (brushSettings) {
      assert.equal(color, '#29b45d');
      assert.equal(await evaluate('document.querySelector(".pg-editor-parameters input[type=number]").valueAsNumber'), 17);
    }
    const pixel = await evaluate(`(() => {
      const stage = document.querySelector('.pg-editor-stage');
      const rect = stage.getBoundingClientRect();
      return { x: rect.left + (rect.width - 227) / 2 + 30, y: rect.top + (rect.height - 149) / 2 + 35 };
    })()`);
    const mouse = (type, buttons) => call('Input.dispatchMouseEvent', { type, ...pixel, button: 'left', buttons, clickCount: 1 });
    await mouse('mousePressed', 1);
    await mouse('mouseReleased', 0);
    await until(() => evaluate('!document.querySelector(".pg-editor-bar button[aria-keyshortcuts=\'Control+S\']").disabled'));
    const original = await readFile(fixture);
    const output = join(profile, brushSettings ? 'saved.bmp' : 'saved.png');
    await evaluate('document.querySelector(".pg-editor-bar button[aria-keyshortcuts=\'Control+Shift+S\']").click()');
    execFileSync('python', ['-c', `
import ctypes, sys, time
from ctypes import wintypes
user = ctypes.windll.user32
pid, target = int(sys.argv[1]), sys.argv[2]
dialog = None
@ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
def find(hwnd, _):
    global dialog
    owner = wintypes.DWORD()
    kind = ctypes.create_unicode_buffer(32)
    user.GetWindowThreadProcessId(hwnd, ctypes.byref(owner))
    user.GetClassNameW(hwnd, kind, len(kind))
    if owner.value == pid and kind.value == '#32770' and user.IsWindowVisible(hwnd):
        dialog = hwnd
    return True
for _ in range(80):
    user.EnumWindows(find, 0)
    if dialog: break
    time.sleep(.1)
assert dialog, 'Save As picker not visible'
field = None
user.SendMessageW.argtypes = [wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM]
user.SendMessageW.restype = wintypes.LPARAM
@ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
def find_field(hwnd, _):
    global field
    kind = ctypes.create_unicode_buffer(32)
    user.GetClassNameW(hwnd, kind, len(kind))
    if kind.value == 'Edit' and user.GetDlgCtrlID(hwnd) == 1001:
        text = ctypes.create_unicode_buffer(1024)
        user.SendMessageW(hwnd, 0xD, len(text), ctypes.addressof(text))
        if text.value == 'fixture-edited.png':
            field = hwnd
    return True
for _ in range(40):
    user.EnumChildWindows(dialog, find_field, 0)
    if field: break
    time.sleep(.1)
assert field, 'Save As filename edit not available'
text = ctypes.create_unicode_buffer(target)
assert user.SendMessageW(field, 0xC, 0, ctypes.addressof(text))
user.GetDlgItem.argtypes = [wintypes.HWND, ctypes.c_int]
user.GetDlgItem.restype = wintypes.HWND
button = user.GetDlgItem(dialog, 1)
assert button, 'Save As button not available'
user.SendMessageW(button, 0xF5, 0, 0)  # BM_CLICK
`, String(browser.pid), output], { timeout: 15000, stdio: 'inherit' });
    const saved = await until(async () => { try { return await readFile(output); } catch { return null; } });
    if (brushSettings) {
      assert.equal(saved.subarray(0, 2).toString(), 'BM');
      execFileSync('python', ['-c', `
from PIL import Image
import sys
with Image.open(sys.argv[1]) as output, Image.open(sys.argv[2]) as source:
    assert output.format == 'BMP' and output.size == (227, 149)
    pixels = output.convert('RGB')
    assert pixels.getpixel((30, 35)) == (41, 180, 93)
    assert pixels.getpixel((36, 35)) == (41, 180, 93)
    assert pixels.getpixel((40, 35)) == source.convert('RGB').getpixel((40, 35))
`, output, fixture], { stdio: 'inherit' });
      assert.deepEqual(await readFile(fixture), original);
      const forwarded = spawn(nativeExe, [output], { env: { ...process.env, WEBVIEW2_USER_DATA_FOLDER: join(profile, 'webview') }, stdio: 'ignore' });
      try {
        await until(() => evaluate('document.querySelector(".filename")?.textContent === "saved.bmp"'));
        assert.equal(await evaluate('Boolean(document.querySelector(".pg-editor-stage"))'), false);
        await key('b');
        await until(() => evaluate('Boolean(document.querySelector(".pg-editor-stage canvas"))'));
        assert.equal(await evaluate('document.querySelector(".pg-editor-parameters input[type=color]").value'), '#29b45d');
        assert.equal(await evaluate('document.querySelector(".pg-editor-parameters input[type=number]").valueAsNumber'), 17);
      } finally {
        forwarded.kill();
      }
      console.log('Native Editor loaded persisted brush defaults, saved the expected BMP pixels and reopened the BMP');
    } else {
    assert.equal(saved.subarray(1, 4).toString(), 'PNG');
    assert.deepEqual(await readFile(fixture), original);
    await until(() => evaluate('!document.querySelector(".pg-editor-bar button[aria-keyshortcuts=\'Control+Shift+S\']").disabled'));
    await evaluate('document.querySelector(".pg-editor-bar button[aria-keyshortcuts=\'Control+Z\']").click()');
    await until(() => evaluate('!document.querySelector(".pg-editor-bar button[aria-keyshortcuts^=\'Control+Y\']").disabled'));
    await evaluate('document.querySelector(".title #close").click()');
    await until(() => evaluate('document.querySelector(".pg-editor-dialog button[value=discard]")?.closest("dialog").open'));
    await evaluate('document.querySelector(".pg-editor-dialog button[value=cancel]").click()');
    await until(() => evaluate('!document.querySelector(".pg-editor-bar button[aria-keyshortcuts=\'Control+S\']").disabled'));
    assert.deepEqual(await readFile(fixture), original);
    await evaluate('document.querySelector(".pg-editor-bar button[aria-keyshortcuts=\'Control+S\']").click()');
    await until(() => evaluate('document.querySelector(".pg-editor-dialog").open'));
    await evaluate('document.querySelector(".pg-editor-dialog button[value=cancel]").click()');
    assert.deepEqual(await readFile(fixture), original);
    await until(() => evaluate('!document.querySelector(".pg-editor-bar button[aria-keyshortcuts=\'Control+S\']").disabled'));
    await evaluate('document.querySelector(".pg-editor-bar button[aria-keyshortcuts=\'Control+S\']").click()');
    await until(() => evaluate('document.querySelector(".pg-editor-dialog").open'));
    await evaluate('document.querySelector(".pg-editor-dialog button[value=save]").click()');
    await until(async () => !(await readFile(fixture)).equals(original));
    const result = await evaluate(`(async () => {
      const bitmap = await createImageBitmap(new Blob([new Uint8Array([${saved.join(',')}])], { type: 'image/png' }));
      const sample = document.createElement('canvas');
      sample.width = bitmap.width;
      sample.height = bitmap.height;
      sample.getContext('2d').drawImage(bitmap, 0, 0);
      bitmap.close();
      return Array.from(sample.getContext('2d').getImageData(30, 35, 1, 1).data);
    })()`);
    const rgb = [1, 3, 5].map(offset => parseInt(color.slice(offset, offset + 2), 16));
    assert.deepEqual(result, [...rgb, 255]);
    console.log('Native Editor saved the selected JPEG pixel to PNG, preserved the source on Save As and cancel, and overwrote after confirmation');
    }
  } else {
  await until(() => evaluate('Boolean(window.smoke)'));
  const hover = (x, y) => call('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
  const cursor = () => evaluate('getComputedStyle(document.querySelector(".pg-editor-stage")).cursor');
  await hover(315, 298);
  assert.equal(await cursor(), 'none', 'Brush hover over the image shows an outline in place of the system cursor');
  assert.equal(await evaluate('smoke.operations.length'), 0);
  await hover(315, 40);
  assert.equal(await cursor(), 'default', 'The margin outside the image uses the system cursor');
  await hover(315, 298);
  await hover(680, 298);
  assert.equal(await cursor(), 'default', 'Leaving the stage clears the brush cursor');

  const themeColors = await evaluate(`(() => {
    const option = document.querySelector('.pg-editor-parameters option');
    return ['dark', 'light'].map(theme => {
      document.documentElement.dataset.theme = theme;
      const style = getComputedStyle(option);
      return [style.backgroundColor, style.color];
    });
  })()`);
  assert.deepEqual(themeColors, [
    ['rgb(41, 43, 50)', 'rgb(243, 243, 246)'],
    ['rgb(255, 255, 255)', 'rgb(32, 33, 42)'],
  ]);

  await evaluate('smoke.openCompression()');
  const closePosition = await evaluate(`(() => {
    const dialog = document.querySelector('.compression-dialog');
    const close = dialog.querySelector('button[aria-label="Close"]');
    if (!close) return null;
    const box = close.getBoundingClientRect();
    const bounds = dialog.getBoundingClientRect();
    return { top: box.top - bounds.top, right: bounds.right - box.right };
  })()`);
  assert.ok(closePosition && closePosition.top < 24 && closePosition.right < 24, 'The compress dialog closes from its top-right corner');
  const closeCompression = () => evaluate('document.querySelector(\'.compression-dialog button[aria-label="Close"]\').click()');
  await closeCompression();
  await until(() => evaluate('!document.querySelector(".compression-dialog")'));
  await evaluate('smoke.openCompression(); document.querySelector(".compression-dialog form").requestSubmit();');
  await until(() => evaluate('document.querySelector(".compression-dialog progress")?.value === 0.5'));
  await closeCompression();
  await until(() => evaluate('!document.querySelector(".compression-dialog")'));
  assert.equal(await evaluate('smoke.compressionCancels'), 1);

  const zoom = await evaluate(`
    (() => {
      const stage = document.querySelector('.pg-editor-stage');
      const rect = stage.getBoundingClientRect();
      stage.dispatchEvent(new WheelEvent('wheel', {
        ctrlKey: true, clientX: rect.left + rect.width / 2, clientY: rect.top + rect.height / 2,
        deltaY: -Math.log(4 / (rect.width / 4000)) / 0.0025, cancelable: true,
      }));
      return stage.querySelector('output').value;
    })()
  `);
  assert.equal(zoom, '400%');
  const center = await evaluate(`
    (() => {
      const rect = document.querySelector('.pg-editor-stage').getBoundingClientRect();
      return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
    })()
  `);
  const mouse = (type, x, y, button, buttons) => call('Input.dispatchMouseEvent', {
    type, x, y, button, buttons, clickCount: type === 'mouseMoved' ? 0 : 1,
  });
  const outline = () => evaluate(`(() => {
    const outline = document.querySelector('.pg-editor-brush-cursor');
    if (!outline || outline.hidden) return null;
    const { x, y, width } = outline.getBoundingClientRect();
    return { x: x + width / 2, y: y + width / 2, width, round: getComputedStyle(outline).borderRadius === '50%' };
  })()`);
  await hover(center.x, center.y);
  assert.deepEqual(await outline(), { ...center, width: 32, round: true });
  await evaluate(`(() => {
    const tip = document.querySelector('.pg-editor-parameters select');
    tip.value = 'square';
    tip.dispatchEvent(new Event('change'));
  })()`);
  assert.deepEqual(await outline(), { ...center, width: 32, round: false });
  await evaluate('Array.from(document.querySelectorAll(".pg-editor-tool-list button")).find(button => button.textContent === "Mosaic").click()');
  assert.deepEqual(await outline(), { ...center, width: 144, round: true });
  await evaluate('Array.from(document.querySelectorAll(".pg-editor-tool-list button")).find(button => button.textContent === "Brush").click()');
  await mouse('mousePressed', center.x, center.y, 'right', 2);
  assert.equal(await outline(), null, 'Panning replaces the brush outline with the system cursor');
  await mouse('mouseMoved', center.x + 20, center.y + 8, 'right', 2);
  await mouse('mouseReleased', center.x + 20, center.y + 8, 'right', 0);
  assert.equal((await outline()).width, 32);
  await mouse('mousePressed', center.x + 24, center.y + 16, 'left', 1);
  await mouse('mouseMoved', 680, center.y, 'left', 1);
  assert.equal(await outline(), null, 'Pointer capture does not keep the outline visible outside the stage');
  await mouse('mouseMoved', center.x + 24, center.y + 16, 'left', 1);
  assert.equal((await outline()).width, 32);
  await mouse('mouseReleased', center.x + 24, center.y + 16, 'left', 0);
  await until(() => evaluate('smoke.editor.session.dirty'));
  const first = await evaluate('smoke.operations[0].points[0]');
  assert.ok(Math.abs(first.x - 2001) < 0.3 && Math.abs(first.y - 1502) < 0.3, JSON.stringify(first));

  await evaluate("smoke.editor.handleShortcut('save')");
  await until(() => evaluate('document.querySelector("dialog").open'));
  await evaluate('document.querySelector("dialog button[value=cancel]").click()');
  await until(() => evaluate('!document.querySelector(".pg-editor-bar button[aria-keyshortcuts=\'Control+S\']").disabled'))
    .catch(async error => { throw new Error(await evaluate('JSON.stringify({ dirty: smoke.editor.session.dirty, dialog: document.querySelector("dialog").open, status: document.querySelector("[role=status]").textContent, writes: smoke.writes.length })'), { cause: error }); });
  assert.equal(await evaluate('smoke.writes.length'), 0);
  await evaluate("smoke.editor.handleShortcut('save')");
  await until(() => evaluate('document.querySelector("dialog").open'));
  await evaluate('document.querySelector("dialog button[value=save]").click()');
  await until(() => evaluate('!smoke.editor.session.dirty'));
  assert.equal(await evaluate('smoke.writes[0].mode'), 'overwrite');

  await evaluate("smoke.editor.handleShortcut('undo')");
  await until(() => evaluate('smoke.operations.length === 0 && smoke.editor.session.dirty && !document.querySelector(".pg-editor-bar button[aria-keyshortcuts^=\'Control+Y\']").disabled'));
  await evaluate("smoke.editor.handleShortcut('redo')");
  await until(() => evaluate('!smoke.editor.session.dirty && !document.querySelector(".pg-editor-bar button[aria-keyshortcuts=\'Control+Z\']").disabled'));
  await evaluate("smoke.editor.handleShortcut('undo')");
  await until(() => evaluate('!document.querySelector(".pg-editor-bar button[aria-keyshortcuts^=\'Control+Y\']").disabled'));
  await evaluate('window.leaveResult = smoke.editor.requestLeave(); null');
  await until(() => evaluate('document.querySelector("dialog").open'));
  await evaluate('document.querySelector("dialog button[value=saveAs]").click()');
  assert.equal(await evaluate('window.leaveResult'), true);
  assert.equal(await evaluate('smoke.writes.at(-1).mode'), 'as');
  console.log('Browser smoke passed: theme colors, compression close and cancellation, brush cursors, pan, overwrite and undo/redo');
  }
} finally {
  socket?.close();
  browser.kill();
  await server?.close();
  await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
