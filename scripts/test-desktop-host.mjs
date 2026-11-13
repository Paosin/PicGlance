import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { once } from 'node:events';
import { existsSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';

const [installedArg, portableArg, fixture, secondFixture, mode] = process.argv.slice(2);
const production = mode === 'production';
const installed = resolve(installedArg);
const portable = resolve(portableArg);
const children = [];
const installedIni = join(process.env.APPDATA, 'PicGlance', 'settings.ini');

async function until(check, description) {
  let lastError;
  for (let attempt = 0; attempt < 80; attempt++) {
    try {
      const value = await check();
      if (value) return value;
    } catch (error) { lastError = error; }
    await delay(200);
  }
  throw new Error(description, { cause: lastError });
}

function launch(dir, port, args = []) {
  const child = spawn(join(dir, 'PicGlance.exe'), args, {
    env: {
      ...process.env,
      ...(production && dir === installed ? { WEBVIEW2_USER_DATA_FOLDER: join(dir, 'validation-webview') } : {}),
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=' + port,
    },
    stdio: 'ignore',
  });
  children.push(child);
  return child;
}

async function debuggerFor(port) {
  const target = await until(async () => {
    const response = await fetch('http://127.0.0.1:' + port + '/json/list', { signal: AbortSignal.timeout(500) });
    return (await response.json()).find(target => target.type === 'page' && target.url.includes('tauri.localhost'));
  }, 'Tauri WebView did not appear on port ' + port);
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await once(socket, 'open');
  let sequence = 0;
  return {
    close: () => socket.close(),
    evaluate: expression => new Promise((resolve, reject) => {
      const id = ++sequence;
      const receive = ({ data }) => {
        const response = JSON.parse(data);
        if (response.id !== id) return;
        clearTimeout(timer);
        socket.removeEventListener('message', receive);
        if (response.error || response.result.exceptionDetails) reject(new Error(JSON.stringify(response)));
        else resolve(response.result.result.value);
      };
      const timer = setTimeout(() => {
        socket.removeEventListener('message', receive);
        reject(new Error('CDP evaluate timed out'));
      }, 3000);
      socket.addEventListener('message', receive);
      socket.send(JSON.stringify({ id, method: 'Runtime.evaluate', params: {
        expression, awaitPromise: true, returnByValue: true,
      } }));
    }),
  };
}

const invoke = (command, payload) => 'window.__TAURI_INTERNALS__.invoke(' + JSON.stringify(command) + ',' + JSON.stringify(payload) + ')';
const configure = (db, settings) => db.evaluate(invoke('desktop_read', {}) + '.then(x => ' +
  'window.__TAURI_INTERNALS__.invoke("desktop_write", {settings: {...x.settings, ...' + JSON.stringify(settings) + '}}))');
const waitImage = (db, path) => until(async () => {
  const shown = await db.evaluate('({path: document.querySelector("#opened").value,width: document.querySelector("#fixture").naturalWidth})');
  return shown.path === path && shown.width === 40;
}, 'Fixture was not displayed: ' + path);

const helper = (...args) => execFileSync('powershell.exe', [
  '-NoProfile', '-File', fileURLToPath(new URL('./test-desktop-windows.ps1', import.meta.url)), ...args,
], { encoding: 'utf8', timeout: 15000 }).trim();

async function waitProductionImage(db, path) {
  const filename = path.split(/[\\/]/).at(-1);
  await until(async () => {
    const image = await db.evaluate('({name: document.querySelector(".filename")?.textContent, width: document.querySelector(".pg-viewer-picture")?.naturalWidth})');
    return image.name === filename && image.width === 40;
  }, 'Production viewer did not display ' + filename);
}

try {
  if (production) {
    const app = launch(installed, 19281);
    const db = await debuggerFor(19281);
    helper('explorer', fixture);
    await waitProductionImage(db, fixture);
    helper('explorer', secondFixture);
    await waitProductionImage(db, secondFixture);
    assert.equal(app.exitCode, null);
    console.log('Explorer verb opened Unicode paths in the original production viewer PID ' + app.pid);
    await db.evaluate('document.querySelector("#settings").click()');
    await until(() => db.evaluate('!!document.querySelector(".desktop-settings")'), 'Production settings page did not render');
    const themeBefore = (await db.evaluate(invoke('desktop_read', {}))).settings.theme;
    const theme = themeBefore === 'light' ? 'dark' : 'light';
    await db.evaluate('document.querySelectorAll(".desktop-settings nav button")[2].click()');
    await db.evaluate('(() => { const x = document.querySelector(".desktop-settings select"); x.value = ' +
      JSON.stringify(theme) + '; x.dispatchEvent(new Event("change")); })()');
    await until(() => db.evaluate('document.documentElement.dataset.theme === ' + JSON.stringify(theme)), 'Production theme did not switch');
    assert.match(readFileSync(installedIni, 'utf8'), new RegExp('theme=' + theme));
    console.log('Production settings page persisted the theme and updated the app.');

    const portableApp = launch(portable, 19282, [fixture]);
    const pdb = await debuggerFor(19282);
    await waitProductionImage(pdb, fixture);
    assert.ok(existsSync(join(portable, 'webview')), 'Portable WebView2 profile must be beside the executable');
    const previous = existsSync(installedIni) ? readFileSync(installedIni) : null;
    await configure(pdb, { theme: 'dark', checkUpdates: false });
    assert.match(readFileSync(join(portable, 'settings.ini'), 'utf8'), /checkUpdates=false/);
    assert.deepEqual(existsSync(installedIni) ? readFileSync(installedIni) : null, previous);
    assert.equal(await pdb.evaluate(invoke('desktop_set_associations', { selected: ['png'] }) + '.catch(e => e.code)'), 'unsupported');
    assert.equal(portableApp.exitCode, null);
    pdb.close();
    db.close();
    console.log('Production portable ZIP opened the fixture, saved its own INI, and rejected an association write.');
  } else {
  const app = launch(installed, 19281, [fixture]);
  const db = await debuggerFor(19281);
  await waitImage(db, fixture);
  helper('explorer', secondFixture);
  await waitImage(db, secondFixture);
  assert.equal(app.exitCode, null);
  console.log('Explorer Shell.Application verb displayed a second Unicode image in the original WebView.');

  await until(() => db.evaluate('!!document.querySelector(".desktop-settings")'), 'Settings page did not render');
  await db.evaluate('(() => { const x = document.querySelector(".desktop-settings select"); x.value = "en"; x.dispatchEvent(new Event("change")); })()');
  await until(() => db.evaluate('document.documentElement.lang === "en"'), 'Language did not switch');
  await db.evaluate('document.querySelectorAll(".desktop-settings nav button")[2].click()');
  await db.evaluate('(() => { const x = document.querySelector(".desktop-settings select"); x.value = "light"; x.dispatchEvent(new Event("change")); })()');
  await until(() => db.evaluate('document.documentElement.dataset.theme === "light"'), 'Theme did not switch');
  assert.match(readFileSync(installedIni, 'utf8'), /theme=light/);

  const portableApp = launch(portable, 19282);
  const pdb = await debuggerFor(19282);
  await until(() => pdb.evaluate('!!document.querySelector(".desktop-settings")'), 'Portable settings did not render');
  assert.ok(existsSync(join(portable, 'webview')), 'Portable WebView2 profile must be beside the executable');
  await configure(pdb, { theme: 'dark', checkUpdates: false });
  assert.match(readFileSync(join(portable, 'settings.ini'), 'utf8'), /checkUpdates=false/);
  assert.match(readFileSync(installedIni, 'utf8'), /theme=light/);
  assert.equal(await pdb.evaluate(invoke('desktop_set_associations', { selected: ['png'] }) + '.catch(e => e.code)'), 'unsupported');
  pdb.close();
  portableApp.kill();
  console.log('Installed and portable processes used separate settings and rejected portable association writes.');

  const pid = app.pid;
  for (const [behavior, path] of [['tray', fixture], ['background', secondFixture]]) {
    await configure(db, { closeBehavior: behavior });
    helper('close', pid);
    await until(() => helper('visible', pid) === 'False', behavior + ' did not hide the window');
    assert.equal(app.exitCode, null);
    const child = launch(installed, 19281, [path]);
    await once(child, 'exit');
    await waitImage(db, path);
    await until(() => helper('visible', pid) === 'True', behavior + ' did not restore on relaunch');
    console.log(behavior + ' hid and restored PID ' + pid);
  }
  await configure(db, { closeBehavior: 'quit', checkUpdates: false });
  const logPath = join(installed, 'log', 'picglance.log');
  await until(() => readFileSync(logPath, 'utf8').includes('update-check'), 'Startup update check did not log');
  const log = readFileSync(logPath, 'utf8');
  console.log(log.trim());
  helper('close', pid);
  await until(() => app.exitCode === 0, 'Quit did not terminate the primary process');
  db.close();

  const restart = launch(installed, 19281);
  const rdb = await debuggerFor(19281);
  await until(() => rdb.evaluate('!!document.querySelector(".desktop-settings")'), 'Restart settings did not render');
  assert.equal((await rdb.evaluate(invoke('desktop_read', {}))).settings.checkUpdates, false);
  await delay(1000);
  assert.equal(readFileSync(logPath, 'utf8'), log);
  helper('close', restart.pid);
  await until(() => restart.exitCode === 0, 'Restart did not quit');
  rdb.close();
  console.log('Disabled startup update check persisted and produced no new check record.');
  }
} finally {
  for (const child of children) if (child.exitCode === null && child.signalCode === null) child.kill();
}
