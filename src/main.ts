import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Effect, getCurrentWindow } from '@tauri-apps/api/window';
import { icon } from './shared/icons.ts';
import { actionForKey, resolveTheme, translate, type Locale, type ShortcutAction, type ThemeMode } from './shared/preferences.ts';
import type { IconName, ImageDocument, ImageResource } from './shared/contracts.ts';
import type { Message } from './shared/messages.ts';
import { toggleFullscreen } from './shared/window.ts';
import { memoryTarget } from './shared/memory.ts';
import { mountViewer } from './features/viewer/index.ts';
import { editorUnavailableReason, mountEditor } from './features/editor/index.ts';
import { openNativeEditor } from './features/editor/backend.ts';
import { WindowsDesktop, type DesktopSnapshot } from './features/desktop/service.ts';
import { settingsPage, updateBanner } from './features/desktop/settings.ts';
import { createCompressionDialog } from './features/compression/index.ts';
import { nativeCompression } from './shared/compression.ts';
import appIcon from '../src-tauri/icons/picglance.svg?no-inline';
import './shared/shell.css';

type Route = 'viewer' | 'edit' | 'compress';
const resizeDirections = ['East', 'North', 'NorthEast', 'NorthWest', 'South', 'SouthEast', 'SouthWest', 'West'] as const;
const native = '__TAURI_INTERNALS__' in window ? getCurrentWindow() : undefined;
const desktop = native ? new WindowsDesktop() : undefined;
const systemTheme = matchMedia('(prefers-color-scheme: dark)');
const root = document.querySelector<HTMLDivElement>('#app')!;
const input = document.createElement('input');
input.type = 'file';
input.accept = 'image/jpeg,image/png,image/gif,image/webp,image/bmp,image/svg+xml,image/avif';
let locale: Locale = 'zh-CN';
let theme: ThemeMode = 'dark';
let route: Route = 'viewer';
let maximized = false;
let fullscreen = false;
let settings = false;
let thumbnails = false;
let info = false;
let scale: number | null = null;
let rotation = 0;
let horizontal = 1;
let vertical = 1;
let status: Message | undefined;
let sequence = 0;
let decoding = false;
let photo: { url: string; name: string; width: number; height: number; bytes: number } | undefined;
let restoreFocus = '';
let viewedImage: { document: ImageDocument; original?: ImageResource } | null = null;
let viewerActivity = false;
let diagnosticPreview = false;
const editorHost = document.createElement('main');
editorHost.className = 'stage edit editor-host';
let editor: Awaited<ReturnType<typeof mountEditor>> | undefined;
let openingEditor = false;
let leavingEditor = false;
let pendingViewerWakeup = false;
let editorNotice: string | undefined;
let desktopSnapshot: DesktopSnapshot | undefined;
let desktopSettingsPage: HTMLElement | undefined;
let updateTag: string | undefined;
let compression: ReturnType<typeof createCompressionDialog> | undefined;
let openingCompression = false;
const viewerHost = document.createElement('main');
viewerHost.className = 'stage viewer';
const viewer = native && mountViewer(viewerHost, {
  window: native,
  keyEnabled: () => !settings && !compression,
  onActivity(active) {
    viewerActivity = active;
    void updateMemoryTarget();
  },
  onDocument(value, original) {
    viewedImage = value ? { document: value, original } : null;
    void updateMemoryTarget();
    const filename = root.querySelector('.filename');
    if (filename) filename.textContent = value?.name ?? '';
    const entry = root.querySelector<HTMLButtonElement>('#edit');
    if (entry) updateEditEntry(entry);
  },
});
const t = (key: Message) => translate(locale, key);

let requestedMemoryTarget: ReturnType<typeof memoryTarget> | undefined;
let memoryUpdate = Promise.resolve();

function updateMemoryTarget() {
  if (!native) return memoryUpdate;
  const target = memoryTarget(document.hidden, decoding || viewerActivity || Boolean(photo || viewedImage));
  if (target !== requestedMemoryTarget) {
    requestedMemoryTarget = target;
    memoryUpdate = memoryUpdate.then(() => invoke<void>('set_webview_memory_target', { low: target === 'low' }))
      .catch(error => console.warn('WebView memory target:', error));
  }
  return memoryUpdate;
}

function element<K extends keyof HTMLElementTagNameMap>(tag: K, className = '', text = '') {
  const node = document.createElement(tag);
  node.className = className;
  if (text) node.textContent = text;
  return node;
}

function button(id: string, label: Message, click: () => unknown, name?: IconName, text = false) {
  const node = element('button');
  node.type = 'button';
  node.id = id;
  node.title = t(label);
  node.setAttribute('aria-label', node.title);
  if (name) node.append(icon(name));
  if (text || !name) node.append(t(label));
  node.addEventListener('click', () => void click());
  return node;
}

function refreshTheme() {
  document.documentElement.dataset.theme = resolveTheme(theme, systemTheme.matches);
}

async function windowAction(action: () => Promise<unknown> | undefined) {
  try { await action(); } catch { status = 'windowError'; render(); }
}

function applyDesktop(snapshot: DesktopSnapshot) {
  desktopSnapshot = snapshot;
  theme = snapshot.settings.theme;
  locale = snapshot.settings.locale;
  viewer && viewer.setLocale(locale);
  viewer && viewer.setDisplayPreferences(snapshot.settings.zoomMode, snapshot.settings.upscaleSmallImages);
  refreshTheme();
  render();
}

function showSettings(value: boolean) {
  settings = value;
  desktopSettingsPage = undefined;
  if (value && desktop && desktopSnapshot) {
    desktopSettingsPage = settingsPage(desktop, desktopSnapshot, applyDesktop, tag => {
      updateTag = tag;
      render();
    }, () => showSettings(false));
  }
  restoreFocus = value && !desktopSettingsPage ? 'theme' : 'settings';
  render();
}

async function showCompression() {
  if (openingCompression || openingEditor || leavingEditor || compression) return;
  if (!viewedImage?.original) {
    editorNotice = t('compressEmpty');
    render();
    return;
  }
  openingCompression = true;
  try {
    let { document: source, original } = viewedImage;
    if (editor) {
      const exported = await editor.exportImage();
      if (!exported.ok) throw new Error(exported.error.code);
      original = exported.value;
      const bytes = await nativeCompression.read(original.id);
      if (!bytes.ok) throw new Error(bytes.error.code);
      const bitmap = await createImageBitmap(new Blob([bytes.value], { type: original.mimeType }));
      source = { ...source, preview: original, width: bitmap.width, height: bitmap.height,
        sizeBytes: bytes.value.byteLength, frameCount: 1 };
      bitmap.close();
    }
    const view = createCompressionDialog(source, original!, nativeCompression, locale);
    compression = view;
    const target = view.dialog.querySelector<HTMLInputElement>('input[type=number]');
    const format = view.dialog.querySelector<HTMLSelectElement>('select');
    if (desktopSnapshot) {
      if (target) target.value = String(Math.max(1, Math.round(desktopSnapshot.settings.compressTargetMb * 1024)));
      const preferred = desktopSnapshot.settings.compressFormat.toLowerCase();
      if (format && [...format.options].some(option => option.value === preferred)) {
        format.value = preferred;
        format.dispatchEvent(new Event('change'));
      }
    }
    view.dialog.addEventListener('close', () => {
      compression = undefined;
      render();
      if (pendingViewerWakeup) drainViewerOpens();
    }, { once: true });
    root.append(view.dialog);
    view.dialog.showModal();
  } catch {
    compression?.dispose();
    compression = undefined;
    editorNotice = t('compressionError');
    render();
  } finally {
    openingCompression = false;
    if (pendingViewerWakeup && !compression) drainViewerOpens();
  }
}

function editUnavailable() {
  if (!viewedImage) return t('editEmpty');
  if (viewedImage.original?.mimeType === 'image/svg+xml') return t('editVector');
  return editorUnavailableReason(viewedImage.document, locale);
}

function updateEditEntry(entry: HTMLButtonElement) {
  const reason = editUnavailable();
  entry.title = reason ?? t('edit');
  if (reason) entry.setAttribute('aria-disabled', 'true');
  else entry.removeAttribute('aria-disabled');
}

function disposeEditor() {
  editor?.dispose();
  editor = undefined;
}

async function afterLeavingEditor(action: () => void | Promise<void>) {
  if (openingEditor || leavingEditor) return;
  leavingEditor = true;
  try {
    if (editor && !await editor.requestLeave()) return;
    disposeEditor();
    await action();
  } finally {
    leavingEditor = false;
    if (pendingViewerWakeup) drainViewerOpens();
  }
}

async function navigate(page: Route) {
  if (page === 'compress' && native) { await showCompression(); return; }
  if (page === route || openingEditor || leavingEditor) return;
  editorNotice = undefined;
  if (page !== 'edit' || !native) {
    await afterLeavingEditor(() => { route = page; render(); });
    return;
  }
  const reason = editUnavailable();
  if (reason || !viewedImage) {
    editorNotice = reason ?? t('editEmpty');
    render();
    return;
  }
  openingEditor = true;
  route = 'edit';
  render();
  try {
    const desktopSettings = desktopSnapshot?.settings;
    editor = await mountEditor(editorHost, viewedImage.document, {
      locale,
      open: openNativeEditor,
      brushColor: desktopSettings?.brushColor,
      brushThickness: desktopSettings?.brushThickness,
      mosaicBlockSize: desktopSettings?.mosaicBlockSize,
      onCancel() {
        disposeEditor();
        route = 'viewer';
        render();
      },
    });
  } catch {
    route = 'viewer';
    editorNotice = t('editError');
  } finally {
    openingEditor = false;
    render();
    if (pendingViewerWakeup) drainViewerOpens();
  }
}

function toolbarButton(action: ShortcutAction, label: Message, name: IconName, pressed?: boolean) {
  const node = button(action, label, () => dispatch(action), name);
  if (pressed !== undefined) node.setAttribute('aria-pressed', String(pressed));
  return node;
}

function image(className = '') {
  const node = element('img', className);
  if (photo) { node.src = photo.url; node.alt = photo.name; }
  return node;
}

function picker<T extends string>(id: string, label: Message, value: T, options: [T, string][], change: (value: T) => void) {
  const field = element('label', 'field', t(label));
  const select = element('select');
  select.id = id;
  for (const [key, title] of options) select.add(new Option(title, key));
  select.value = value;
  select.onchange = () => change(select.value as T);
  field.append(select);
  return field;
}

function render() {
  if (compression?.dialog.open) return;
  const focus = document.activeElement instanceof HTMLElement ? document.activeElement.id : '';
  root.dataset.maximized = String(maximized || fullscreen);
  root.dataset.fullscreen = String(fullscreen);
  document.documentElement.lang = locale;
  const title = element('header', 'title');
  const drag = element('div', 'drag', 'PicGlance');
  drag.setAttribute('data-tauri-drag-region', '');
  const brand = element('img', 'brand');
  brand.src = appIcon;
  brand.alt = '';
  brand.draggable = false;
  drag.prepend(brand);
  title.append(drag);
  if (native) title.append(
    button('minimize', 'minimize', () => windowAction(() => native.minimize()), 'minimize'),
    button('maximize', maximized ? 'restore' : 'maximize', () => windowAction(async () => {
      await native.toggleMaximize();
      await updateWindow();
    }), maximized ? 'restore' : 'maximize'),
    button('close', 'close', () => windowAction(() => native.close()), 'close'),
  );

  const toolbar = element('nav', 'toolbar material');
  const open = button('open', 'open', () => dispatch('open'), 'open', true);
  open.title = t('openTip');
  toolbar.append(open, element('span', 'filename', viewedImage?.document.name ?? photo?.name ?? ''));
  const routes: [Route, Message][] = [['edit', 'edit'], ['compress', 'compress']];
  if (route !== 'viewer') routes.unshift(['viewer', 'back']);
  for (const [page, label] of routes) {
    const entry = button(page, label, () => void navigate(page));
    if (page === 'edit' && native && route !== 'edit') updateEditEntry(entry);
    if (route === page) entry.setAttribute('aria-current', 'page');
    toolbar.append(entry);
  }
  const settingsButton = button('settings', 'settings', () => {
    showSettings(!settings);
  }, 'settings');
  settingsButton.setAttribute('aria-expanded', String(settings));
  toolbar.append(settingsButton);

  const stage = element('main', `stage ${route}`);
  const canvas = element('section', 'canvas');
  stage.append(canvas);
  if (route === 'compress' && photo) {
    canvas.classList.add('comparison');
    for (const key of ['original', 'result'] as const) {
      const panel = element('figure');
      panel.append(element('figcaption', '', t(key)), image());
      canvas.append(panel);
    }
  } else if (photo) {
    const preview = image('preview');
    preview.style.transform = `rotate(${rotation}deg) scale(${horizontal}, ${vertical})`;
    if (scale !== null) {
      preview.classList.add('manual');
      preview.style.width = `${photo.width * scale}px`;
    }
    canvas.append(preview);
  } else {
    const empty = element('div', 'empty');
    const mark = element('div', 'empty-icon');
    mark.append(icon('image', 33));
    empty.append(mark);
    if (route === 'viewer') {
      empty.append(button('empty-open', 'empty', () => input.click()), element('p', '', t('emptyHint')));
    } else empty.append(element('p', '', t(route === 'edit' ? 'editEmpty' : 'compressEmpty')));
    canvas.append(empty);
  }

  if (route === 'viewer') {
    canvas.ondblclick = () => dispatch('fullscreen');
    canvas.addEventListener('wheel', event => {
      if (event.ctrlKey && photo) { event.preventDefault(); zoom(event.deltaY > 0 ? 0.8 : 1.25); }
    }, { passive: false });
    const panel = element('aside', 'metadata material');
    panel.dataset.open = String(info);
    panel.inert = !info;
    panel.append(button('close-info', 'close', () => { info = false; restoreFocus = 'info'; render(); }, 'close'),
      element('p', '', t('noMetadata')));
    stage.append(panel);
  } else {
    const panel = element('aside', 'tool-panel material');
    const label = element('label', 'field', t(route === 'edit' ? 'size' : 'targetSize'));
    const parameter = element('input');
    parameter.type = route === 'edit' ? 'range' : 'number';
    parameter.min = '1';
    parameter.defaultValue = route === 'edit' ? '24' : '512';
    parameter.disabled = true;
    label.append(parameter);
    if (route === 'edit') panel.append(element('p', '', t('brushTip')));
    panel.append(label);
    stage.append(panel);
  }

  const statusbar = element('div', 'status material');
  if (photo) {
    const bytes = new Intl.NumberFormat(locale, { maximumFractionDigits: 1 }).format(photo.bytes / 1000);
    statusbar.textContent = `${photo.name} · ${photo.width}×${photo.height} · ${bytes} KB`;
  }
  const footer = element('footer', 'controls material');
  const filmstrip = element('div', 'filmstrip material');
  filmstrip.dataset.open = String(route === 'viewer' && thumbnails);
  filmstrip.inert = !(route === 'viewer' && thumbnails);
  filmstrip.setAttribute('aria-label', t('thumbnails'));
  if (photo) filmstrip.append(image('selected-thumbnail'));
  if (route === 'viewer') {
    const zoomOut = button('zoom-out', 'zoomOut', () => zoom(0.8), 'minus');
    const zoomIn = button('zoom-in', 'zoomIn', () => zoom(1.25), 'plus');
    zoomIn.disabled = zoomOut.disabled = !photo;
    footer.append(zoomOut, element('output', '', scale === null ? '—' : `${Math.round(scale * 100)}%`), zoomIn,
      toolbarButton('fit', 'fitTip', 'fit', scale === null), toolbarButton('actual', 'actualTip', 'actual', scale === 1),
      element('span', 'divider'), toolbarButton('rotateRight', 'rotateTip', 'rotate'), toolbarButton('flipHorizontal', 'flipTip', 'flip'),
      toolbarButton('info', 'infoTip', 'info', info), toolbarButton('thumbnails', 'thumbnailsTip', 'thumbnails', thumbnails),
      toolbarButton('fullscreen', 'fullscreenTip', 'fullscreen', fullscreen));
  }

  const useViewer = viewer && route === 'viewer' && !diagnosticPreview;
  viewerHost.hidden = !useViewer;
  if (native && route === 'edit') {
    if (openingEditor && !editor) editorHost.replaceChildren(element('p', 'notice material', t('loading')));
    if (editorHost.parentElement === root) {
      root.querySelector('.title')?.replaceWith(title);
      root.querySelector('.toolbar')?.replaceWith(toolbar);
      while (editorHost.nextSibling) editorHost.nextSibling.remove();
    } else root.replaceChildren(title, toolbar, editorHost);
  } else if (useViewer) {
    root.replaceChildren(title, toolbar, viewerHost);
  } else {
    root.replaceChildren(title, toolbar, stage, filmstrip, statusbar, footer);
  }
  if (settings && desktopSettingsPage) {
    root.append(desktopSettingsPage);
  } else if (settings) {
    const panel = element('section', 'settings material');
    panel.setAttribute('aria-label', t('settings'));
    panel.append(picker('theme', 'theme', theme, [['dark', t('dark')], ['light', t('light')], ['system', t('system')]], value => {
      theme = value; refreshTheme();
    }), picker('language', 'language', locale, [['zh-CN', '简体中文'], ['en', 'English']], value => {
      locale = value;
      if (viewer) viewer.setLocale(value);
      render();
    }));
    root.append(panel);
  }
  if (desktop && updateTag) root.append(updateBanner(desktop, locale, updateTag, () => {
    updateTag = undefined;
    render();
  }));
  if (status || editorNotice) {
    const notice = element('div', 'notice material', editorNotice ?? t(status!));
    notice.setAttribute('role', 'status');
    root.append(notice);
  }
  if (native && !maximized && !fullscreen) {
    for (const direction of resizeDirections) {
      const handle = element('div', `resize ${direction.toLowerCase()}`);
      handle.onpointerdown = event => {
        if (event.button === 0) void windowAction(() => native.startResizeDragging(direction));
      };
      root.append(handle);
    }
  }
  if (restoreFocus || focus) document.getElementById(restoreFocus || focus)?.focus({ preventScroll: true });
  restoreFocus = '';
}

function zoom(factor: number) {
  if (!photo) return;
  const canvas = root.querySelector('.canvas')!;
  const fitted = Math.min(1, canvas.clientWidth / photo.width, canvas.clientHeight / photo.height);
  scale = Math.max(0.05, Math.min(8, (scale ?? fitted) * factor));
  render();
}

async function updateWindow() {
  if (native) {
    const [nextMaximized, nextFullscreen] = await Promise.all([native.isMaximized(), native.isFullscreen()]);
    if (maximized === nextMaximized && fullscreen === nextFullscreen) return;
    maximized = nextMaximized;
    fullscreen = nextFullscreen;
    render();
  }
}

function dispatch(action: ShortcutAction): boolean {
  if (editor?.handleShortcut(action)) return true;
  if (route === 'viewer' && viewedImage && viewedImage.document.sourcePath === null) {
    if (action === 'save') return true;
    if (action === 'saveAs') {
      const source = viewedImage.document;
      void afterLeavingEditor(async () => {
        try {
          const session = await openNativeEditor(source);
          try { await session.save([], 'as'); } finally { session.dispose(); }
        } catch {
          editorNotice = t('saveError');
          render();
        }
      });
      return true;
    }
  }
  switch (action) {
    case 'open':
      if (viewer) {
        void afterLeavingEditor(async () => {
          diagnosticPreview = false;
          editorNotice = undefined;
          route = 'viewer';
          render();
          await viewer.choose();
        });
      } else input.click();
      return true;
    case 'brush': void navigate('edit'); return true;
    case 'fit': scale = null; break;
    case 'actual': scale = 1; break;
    case 'rotateRight': rotation = (rotation + 90) % 360; break;
    case 'rotateLeft': rotation = (rotation - 90) % 360; break;
    case 'flipHorizontal': horizontal *= -1; break;
    case 'flipVertical': vertical *= -1; break;
    case 'thumbnails': thumbnails = !thumbnails; break;
    case 'info': info = !info; break;
    case 'fullscreen':
      void windowAction(async () => {
        if (native) { await toggleFullscreen(native); await updateWindow(); }
        else if (document.fullscreenElement) await document.exitFullscreen();
        else await root.requestFullscreen();
      });
      return true;
    case 'escape':
      if (settings) { showSettings(false); return true; }
      if (fullscreen || document.fullscreenElement) return dispatch('fullscreen');
      if (native) void windowAction(() => native.close());
      return Boolean(native);
    default: return false;
  }
  render();
  return true;
}

async function load(file: Blob, name: string) {
  const request = ++sequence;
  if (file.size > 80_000_000) {
    decoding = false;
    void updateMemoryTarget();
    status = 'tooLarge';
    render();
    return false;
  }
  decoding = true;
  status = 'loading';
  render();
  await updateMemoryTarget();
  if (request !== sequence) return false;
  const url = URL.createObjectURL(file);
  const decoded = new Image();
  decoded.src = url;
  try { await decoded.decode(); }
  catch {
    URL.revokeObjectURL(url);
    if (request === sequence) {
      decoding = false;
      void updateMemoryTarget();
      status = 'invalidImage';
      render();
    }
    return false;
  }
  if (request !== sequence) { URL.revokeObjectURL(url); return false; }
  if (photo) URL.revokeObjectURL(photo.url);
  photo = { url, name, width: decoded.naturalWidth, height: decoded.naturalHeight, bytes: file.size };
  decoding = false;
  scale = null;
  rotation = 0;
  horizontal = vertical = 1;
  status = undefined;
  render();
  return true;
}

input.onchange = () => {
  const file = input.files?.[0];
  if (file) void load(file, file.name);
  input.value = '';
};
document.addEventListener('keydown', event => {
  if (event.defaultPrevented) return;
  if (compression) return;
  const editing = event.target instanceof Element && Boolean(event.target.closest('input, textarea, select, [contenteditable]:not([contenteditable="false"])'));
  const action = actionForKey(event, editing);
  if (action && !event.repeat && dispatch(action)) event.preventDefault();
});
systemTheme.addEventListener('change', refreshTheme);
refreshTheme();
render();

function drainViewerOpens() {
  if (!viewer) return;
  if (openingEditor || leavingEditor || openingCompression || compression) {
    pendingViewerWakeup = true;
    return;
  }
  pendingViewerWakeup = false;
  leavingEditor = true;
  void (async () => {
    const id = await invoke<string | null>('viewer_take_pending_open');
    if (!id || (editor && !await editor.requestLeave())) return;
    disposeEditor();
    diagnosticPreview = false;
    editorNotice = undefined;
    route = 'viewer';
    render();
    await viewer.open(id);
  })().catch(console.error).finally(() => {
    leavingEditor = false;
    if (pendingViewerWakeup) drainViewerOpens();
  });
}

if (native) {
  void native.onCloseRequested(async event => {
    event.preventDefault();
    if (compression || openingCompression) return;
    await afterLeavingEditor(async () => {
      if (desktopSnapshot?.settings.closeBehavior !== 'quit') {
        route = 'viewer';
        render();
      }
      await windowAction(() => invoke('desktop_close_from_host'));
    });
  }).catch(console.error);
  void desktop?.read().then(applyDesktop).catch(console.error);
  void desktop?.checkUpdate(true).then(tag => {
    if (tag) { updateTag = tag; render(); }
  }).catch(console.error);
  void listen('picglance:viewer-open-pending', drainViewerOpens).then(drainViewerOpens).catch(console.error);
  document.addEventListener('visibilitychange', updateMemoryTarget);
  void updateMemoryTarget();
  void native.onResized(() => { void windowAction(updateWindow); });
  void invoke<number>('windows_build').then(async build => {
    if (build >= 22000) {
      await native.setEffects({ effects: [Effect.Acrylic] });
      root.classList.add('acrylic');
    }
  }).catch(console.error);
  void invoke<ArrayBuffer>('diagnostic_image').then(async bytes => {
    if (!bytes.byteLength) return;
    diagnosticPreview = true;
    if (await load(new Blob([bytes], { type: 'image/jpeg' }), 'Diagnostic JPEG')) {
      await root.querySelector<HTMLImageElement>('img.preview')?.decode();
      requestAnimationFrame(() => requestAnimationFrame(() => void invoke('diagnostic_presented')));
    }
  }).catch(() => { /* Only registered in the diagnostic build. */ });
}
