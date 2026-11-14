import type { ImageDocument, ImageResource, WindowControls, IconName } from '../../shared/contracts.ts';
import { icon } from '../../shared/icons.ts';
import { actionForKey, type Locale, type ShortcutAction } from '../../shared/preferences.ts';
import { tauriViewerBackend, type ViewerBackend, type ImageDetails, type DirectoryEntry, type ViewerError } from './backend.ts';
import { DirectoryCursor, LatestRequest, fitScale, previewMatrix, zoomAt } from './model.ts';
import { labels, metadataLabels } from './labels.ts';
import './viewer.css';

export { tauriViewerBackend } from './backend.ts';
export type { ViewerBackend } from './backend.ts';

export interface ViewerOptions {
  backend?: ViewerBackend;
  window: WindowControls;
  locale?: Locale;
  onDocument?(document: ImageDocument | null, original?: ImageResource): void;
  onActivity?(active: boolean): void;
  keyEnabled?(): boolean;
}

export interface ViewerMount {
  open(id: string): Promise<void>;
  choose(): Promise<void>;
  dispatch(action: ShortcutAction | 'closeFullscreen'): boolean;
  setLocale(locale: Locale): void;
  setDisplayPreferences(zoomMode: 'fit' | 'actual', upscale: boolean): void;
  dispose(): void;
}

const THUMB_WIDTH = 68;
const THUMB_CACHE = 96;
const THUMB_WORKERS = 2;
const THUMB_BYTES = 16 * 1024 * 1024;

function node<K extends keyof HTMLElementTagNameMap>(tag: K, className: string) {
  const element = document.createElement(tag);
  element.className = className;
  return element;
}

export function mountViewer(host: HTMLElement, options: ViewerOptions): ViewerMount {
  const backend = options.backend ?? tauriViewerBackend;
  let locale = options.locale ?? 'zh-CN';
  let zoomMode: 'fit' | 'actual' = 'fit';
  let upscale = false;
  const t = (key: keyof typeof labels.en) => labels[locale][key];
  const root = node('section', 'pg-viewer');
  const stage = node('section', 'pg-viewer-stage');
  const picture = node('img', 'pg-viewer-picture');
  picture.draggable = false;
  picture.hidden = true;
  const placeholder = node('div', 'pg-viewer-placeholder');
  const notice = node('p', 'pg-viewer-notice');
  notice.setAttribute('role', 'status');
  const metadata = node('aside', 'pg-viewer-metadata material');
  const renameForm = node('form', 'pg-viewer-rename');
  const renameLabel = node('label', '');
  const renameInput = node('input', '');
  const renameButton = node('button', '');
  const renameNotice = node('p', 'pg-viewer-rename-notice');
  renameInput.type = 'text';
  renameInput.id = 'pg-viewer-rename-name';
  renameInput.maxLength = 255;
  renameLabel.htmlFor = renameInput.id;
  renameButton.type = 'submit';
  renameNotice.setAttribute('role', 'status');
  renameForm.append(renameLabel, renameInput, renameButton, renameNotice);
  const metadataBody = node('div', 'pg-viewer-metadata-body');
  const strip = node('div', 'pg-viewer-strip material');
  const rail = node('div', 'pg-viewer-rail');
  const info = node('div', 'pg-viewer-info material');
  const controls = node('div', 'pg-viewer-controls material');
  const percent = node('output', 'pg-viewer-percent');
  const cursor = new DirectoryCursor();
  const requests = new LatestRequest();
  const directoryRequests = new LatestRequest();
  const thumbnailRequests = new LatestRequest();
  const detailRequests = new LatestRequest();
  const warmRequests = new LatestRequest();
  const thumbnails = new Map<string, { url: string; bytes: number } | null>();
  let thumbnailBytes = 0;
  const controller = new AbortController();
  const signal = controller.signal;
  let entries: DirectoryEntry[] = [];
  let details: ImageDetails | undefined;
  let source: string | undefined;
  let navigationDirection = 1;
  let showStrip = false;
  let showMetadata = false;
  let previewUrl: string | undefined;
  let detailUrl: string | undefined;
  let detailRequested = false;
  let previewWidth = 0;
  let previewHeight = 0;
  let selection = 0;
  let renaming = false;
  let rotation = 0;
  let horizontal = false;
  let vertical = false;
  let scale: number | undefined;
  let offset = { x: 0, y: 0 };
  let message: keyof typeof labels.en | ViewerError = 'empty';
  let fullscreenWork = Promise.resolve();
  const buttons: [HTMLButtonElement, keyof typeof labels.en][] = [];

  function button(label: keyof typeof labels.en, handler: () => unknown, glyph?: IconName, text?: string) {
    const result = node('button', 'pg-viewer-button');
    result.type = 'button';
    if (glyph) result.append(icon(glyph));
    else if (text) result.textContent = text;
    result.addEventListener('click', () => void handler(), { signal });
    buttons.push([result, label]);
    return result;
  }

  const openButton = button('open', choose, 'open');
  const closeInfo = button('close', () => setMetadata(false), 'close');
  const previous = button('previous', () => navigate(-1), undefined, '‹');
  const next = button('next', () => navigate(1), undefined, '›');
  const stripButton = button('thumbnails', () => setStrip(!showStrip), 'thumbnails');
  const infoButton = button('info', () => setMetadata(!showMetadata), 'info');
  const fitButton = button('fit', () => resetScale(), 'fit');
  const actualButton = button('actual', () => resetScale(1), 'actual');
  const zoomOut = button('zoomOut', () => zoom(0.8), 'minus');
  const zoomIn = button('zoomIn', () => zoom(1.25), 'plus');
  const rotate = button('rotate', () => transform('rotateRight'), 'rotate');
  const flip = button('flip', () => transform('flipHorizontal'), 'flip');
  controls.append(previous, next, zoomOut, zoomIn, fitButton, actualButton, rotate, flip,
    stripButton, infoButton, button('fullscreen', () => fullscreen('toggle'), 'fullscreen'), percent);
  metadata.append(closeInfo, metadataBody, renameForm);
  strip.append(rail);
  placeholder.append(openButton, notice);
  stage.append(picture, placeholder, metadata, info);
  root.append(stage, strip, controls);
  host.append(root);

  function textForMessage() {
    if (typeof message === 'string') return t(message);
    if (message.code === 'too-large') {
      const size = new Intl.NumberFormat(locale, { maximumFractionDigits: 2 }).format((message.sizeBytes ?? 0) / (1024 * 1024));
      return t('tooLarge').replace('{size}', size);
    }
    return t(message.code);
  }

  function refreshText() {
    for (const [control, label] of buttons) {
      control.title = t(label);
      control.setAttribute('aria-label', control.title);
    }
    strip.setAttribute('aria-label', t('thumbnails'));
    openButton.replaceChildren(icon('open'), t('open'));
    notice.textContent = textForMessage();
    const image = details?.document;
    const name = image?.name || t('clipboard');
    renameForm.hidden = !image?.sourcePath;
    renameLabel.textContent = t('filename');
    renameButton.textContent = t('rename');
    renameInput.value = image?.name ?? '';
    if (image) {
      picture.alt = name;
      const size = new Intl.NumberFormat(locale, { maximumFractionDigits: 1, style: 'unit', unit: 'kilobyte' }).format(image.sizeBytes / 1024);
      info.textContent = `${name} · ${image.width}×${image.height} · ${size}`;
    } else info.textContent = '';
    metadataBody.replaceChildren();
    if (details?.gps) {
      const warning = node('p', 'pg-viewer-gps');
      warning.textContent = t('gps');
      metadataBody.append(warning);
    }
    if (details?.metadata.length) {
      const fields = node('dl', 'pg-viewer-fields');
      for (const { tag, value } of details.metadata) {
        const key = node('dt', '');
        key.textContent = metadataLabels[locale][tag] ?? tag;
        const field = node('dd', '');
        field.textContent = value;
        fields.append(key, field);
      }
      metadataBody.append(fields);
    } else {
      const empty = node('p', '');
      empty.textContent = t('noMetadata');
      metadataBody.append(empty);
    }
  }

  function status(value: typeof message) {
    message = value;
    notice.textContent = textForMessage();
    notice.hidden = value === 'empty' && Boolean(details);
    placeholder.dataset.overlay = String(Boolean(details));
    openButton.hidden = Boolean(details) || value === 'loading';
  }

  function asError(error: unknown): ViewerError {
    if (typeof error === 'object' && error && 'code' in error && typeof error.code === 'string'
      && (error.code === 'too-large' || Object.hasOwn(labels.en, error.code))) return error as ViewerError;
    return { code: 'io' };
  }

  function setMetadata(value: boolean) {
    showMetadata = value;
    metadata.dataset.open = String(value);
    metadata.inert = !value;
    metadata.setAttribute('aria-hidden', String(!value));
    infoButton.setAttribute('aria-pressed', String(value));
    if (value) closeInfo.focus({ preventScroll: true });
    if (!value && metadata.contains(document.activeElement)) infoButton.focus();
  }

  function fitted() {
    const image = details?.document;
    return image ? fitScale(image.width, image.height, stage.clientWidth, stage.clientHeight, rotation, upscale) : 1;
  }

  function layout() {
    const image = details?.document;
    const ratio = scale ?? fitted();
    percent.textContent = image ? `${Math.round(ratio * 100)}%` : '—';
    for (const item of [zoomOut, zoomIn, fitButton, actualButton, rotate, flip]) item.disabled = !image;
    fitButton.setAttribute('aria-pressed', String(scale === undefined));
    actualButton.setAttribute('aria-pressed', String(scale === 1));
    if (!image) return;
    picture.style.width = `${image.width * ratio}px`;
    picture.style.height = `${image.height * ratio}px`;
    const matrix = previewMatrix(rotation, horizontal, vertical);
    picture.style.transform = `translate(calc(-50% + ${offset.x}px), calc(-50% + ${offset.y}px)) matrix(${matrix.join(',')},0,0)`;
    if (scale !== undefined && (image.width * ratio * devicePixelRatio > previewWidth + 1
      || image.height * ratio * devicePixelRatio > previewHeight + 1)) void ensureDetail();
  }

  async function ensureDetail() {
    const current = details;
    if (detailRequested || !current || current.document.frameCount > 1
      || (current.document.width === previewWidth && current.document.height === previewHeight)) return;
    detailRequested = true;
    const request = detailRequests.start();
    try {
      const bytes = await backend.resource(`${current.document.id}:detail`, request);
      if (request.aborted || details !== current) return;
      const url = URL.createObjectURL(new Blob([bytes], { type: 'image/png' }));
      const decoded = new Image();
      decoded.src = url;
      try { await decoded.decode(); } catch { URL.revokeObjectURL(url); return; }
      if (request.aborted || details !== current) {
        URL.revokeObjectURL(url);
        return;
      }
      detailUrl = url;
      picture.src = url;
      if (previewUrl) URL.revokeObjectURL(previewUrl);
      previewUrl = undefined;
    } catch (error) {
      if (!request.aborted && details === current) status(asError(error));
    }
  }

  function resetScale(value?: number) {
    scale = value;
    offset = { x: 0, y: 0 };
    layout();
  }

  function zoom(factor: number, event?: WheelEvent) {
    if (!details) return;
    const before = scale ?? fitted();
    if (!before) return;
    scale = Math.min(16, Math.max(Math.min(0.05, fitted() || 0.05), before * factor));
    const rect = stage.getBoundingClientRect();
    const pointer = event ? { x: event.clientX - rect.left - rect.width / 2, y: event.clientY - rect.top - rect.height / 2 } : { x: 0, y: 0 };
    offset = zoomAt(before, scale, offset, pointer);
    layout();
  }

  function transform(action: string) {
    if (action === 'rotateRight') rotation = (rotation + 1) % 4;
    if (action === 'rotateLeft') rotation = (rotation + 3) % 4;
    if (action === 'flipHorizontal') horizontal = !horizontal;
    if (action === 'flipVertical') vertical = !vertical;
    layout();
  }

  function fullscreen(action: 'toggle' | 'exit' | 'close') {
    fullscreenWork = fullscreenWork.then(async () => {
      const active = await options.window.isFullscreen();
      if (signal.aborted) return;
      if (active || action === 'toggle') await options.window.setFullscreen(!active);
      if (active && action === 'close') await options.window.close();
    }).catch(error => { if (!signal.aborted) status(asError(error)); });
  }

  function setEntries(value: DirectoryEntry[], selected?: string) {
    entries = value;
    cursor.reset(entries.map(entry => entry.id), selected ?? '');
    previous.disabled = next.disabled = entries.length === 0;
    rail.style.width = `${entries.length * THUMB_WIDTH}px`;
    refreshStrip();
  }

  function setStrip(value: boolean) {
    showStrip = value;
    strip.dataset.open = String(value);
    strip.inert = !value;
    stripButton.setAttribute('aria-pressed', String(value));
    if (value) scrollToSelected();
    refreshStrip();
  }

  function scrollToSelected() {
    if (!showStrip) return;
    const index = entries.findIndex(entry => entry.id === source);
    if (index < 0) return;
    const start = index * THUMB_WIDTH;
    if (start < strip.scrollLeft) strip.scrollLeft = start;
    else if (start + THUMB_WIDTH > strip.scrollLeft + strip.clientWidth) strip.scrollLeft = start + THUMB_WIDTH - strip.clientWidth;
  }

  function clearThumbnails() {
    thumbnailRequests.cancel();
    for (const cached of thumbnails.values()) if (cached) URL.revokeObjectURL(cached.url);
    thumbnails.clear();
    thumbnailBytes = 0;
    strip.scrollLeft = 0;
  }

  function paintThumbnail(element: HTMLButtonElement, value: { url: string } | null) {
    element.replaceChildren();
    if (!value) {
      element.append(icon('image'));
      return;
    }
    const image = node('img', '');
    image.src = value.url;
    image.alt = '';
    image.draggable = false;
    element.append(image);
  }

  function refreshStrip() {
    const request = thumbnailRequests.start();
    if (!showStrip || !entries.length || signal.aborted) {
      rail.replaceChildren();
      return;
    }
    const first = Math.max(0, Math.floor(strip.scrollLeft / THUMB_WIDTH) - 2);
    const last = Math.min(entries.length, Math.ceil((strip.scrollLeft + strip.clientWidth) / THUMB_WIDTH) + 2);
    const visible = new Set(entries.slice(first, last).map(entry => entry.id));
    const pending: { id: string; element: HTMLButtonElement }[] = [];
    const fragment = document.createDocumentFragment();
    const focused = (document.activeElement as HTMLElement | null)?.dataset.entry;
    for (let index = first; index < last; index++) {
      const { id, name } = entries[index];
      const element = node('button', 'pg-viewer-thumb');
      element.type = 'button';
      element.dataset.entry = id;
      element.style.left = `${index * THUMB_WIDTH}px`;
      element.title = name;
      element.setAttribute('aria-label', name);
      if (id === source) element.setAttribute('aria-current', 'true');
      element.onclick = () => { cursor.select(id); void open(id, true); };
      const cached = thumbnails.get(id);
      paintThumbnail(element, cached ?? null);
      if (cached === undefined) pending.push({ id, element });
      else {
        thumbnails.delete(id);
        thumbnails.set(id, cached);
      }
      fragment.append(element);
    }
    rail.replaceChildren(fragment);
    if (focused) (Array.from(rail.children).find(child => (child as HTMLElement).dataset.entry === focused) as HTMLElement | undefined)
      ?.focus({ preventScroll: true });
    async function fill() {
      while (pending.length && !request.aborted) {
        const { id, element } = pending.shift()!;
        let value: { url: string; bytes: number } | null;
        try {
          const bytes = await backend.thumbnail(id, request);
          if (request.aborted) return;
          value = { url: URL.createObjectURL(new Blob([bytes], { type: 'image/png' })), bytes: bytes.byteLength };
        } catch (error) {
          if (request.aborted) return;
          if (asError(error).code === 'busy') continue;
          value = null;
        }
        thumbnails.set(id, value);
        thumbnailBytes += value?.bytes ?? 0;
        while (thumbnails.size > THUMB_CACHE || thumbnailBytes > THUMB_BYTES) {
          const key = Array.from(thumbnails.keys()).find(item => !visible.has(item)) ?? thumbnails.keys().next().value;
          if (key === undefined) break;
          const evicted = thumbnails.get(key);
          thumbnails.delete(key);
          if (evicted) {
            URL.revokeObjectURL(evicted.url);
            thumbnailBytes -= evicted.bytes;
          }
        }
        paintThumbnail(element, value);
      }
    }
    for (let worker = 0; worker < THUMB_WORKERS; worker++) void fill();
  }

  function displaySize() {
    const ratio = globalThis.devicePixelRatio || 1;
    return { width: Math.ceil(stage.clientWidth * ratio), height: Math.ceil(stage.clientHeight * ratio) };
  }

  function clearImage() {
    detailRequests.cancel();
    if (previewUrl) URL.revokeObjectURL(previewUrl);
    if (detailUrl) URL.revokeObjectURL(detailUrl);
    previewUrl = undefined;
    detailUrl = undefined;
    detailRequested = false;
    picture.removeAttribute('src');
    picture.hidden = true;
    details = undefined;
    renameNotice.textContent = '';
    options.onDocument?.(null);
    refreshText();
    layout();
  }

  async function load(id?: string, inDirectory = false) {
    warmRequests.cancel();
    selection++;
    const pasting = id === undefined;
    const request = requests.start();
    if (!inDirectory && !pasting) {
      directoryRequests.cancel();
      clearThumbnails();
      setEntries([]);
    }
    if (!pasting) {
      source = id;
      scrollToSelected();
      refreshStrip();
      clearImage();
    }
    status('loading');
    options.onActivity?.(true);
    const size = displaySize();
    // A fresh open gives the loading state a paint. Directory decode can start immediately.
    if (!inDirectory && !pasting) {
      await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
    }
    if (request.aborted) return;
    try {
      const result = id === undefined ? await backend.paste(size, request) : await backend.open(id, size, request);
      if (request.aborted) return;
      const bytes = await backend.resource(result.document.preview.id);
      if (request.aborted) return;
      if (pasting) {
        source = undefined;
        directoryRequests.cancel();
        clearThumbnails();
        setEntries([]);
        clearImage();
      }
      const url = URL.createObjectURL(new Blob([bytes], { type: result.document.preview.mimeType }));
      previewUrl = url;
      picture.src = url;
      await picture.decode();
      if (request.aborted) return;
      details = result;
      previewWidth = picture.naturalWidth;
      previewHeight = picture.naturalHeight;
      rotation = 0;
      horizontal = vertical = false;
      resetScale(zoomMode === 'actual' ? 1 : undefined);
      picture.hidden = false;
      picture.animate([{ opacity: 0 }, { opacity: 1 }], {
        duration: matchMedia('(prefers-reduced-motion: reduce)').matches ? 0 : 100, easing: 'ease-out',
      });
      refreshText();
      status('empty');
      options.onDocument?.(result.document, result.original);
      if (inDirectory && result.document.frameCount === 1) warmAdjacent();
    } catch (error) {
      if (request.aborted) return;
      if (!pasting || !details) clearImage();
      const failure = error instanceof DOMException ? { code: 'invalid-input' as const } : asError(error);
      if (id && failure.code === 'missing') {
        cursor.remove(id);
        entries = entries.filter(entry => entry.id !== id);
        rail.style.width = `${entries.length * THUMB_WIDTH}px`;
        previous.disabled = next.disabled = entries.length === 0;
        refreshStrip();
      }
      status(failure);
    } finally {
      if (!request.aborted) options.onActivity?.(Boolean(details));
    }
    if (!request.aborted && id && !inDirectory) {
      const listing = directoryRequests.start();
      try {
        const result = await backend.directory(id, listing);
        if (listing.aborted) return;
        setEntries(result, source);
        scrollToSelected();
        refreshStrip();
        if (details?.document.frameCount === 1) warmAdjacent();
      } catch (error) {
        if (!listing.aborted) status(asError(error));
      }
    }
  }

  function open(id: string, inDirectory = false) { return load(id, inDirectory); }
  async function choose() {
    const intent = ++selection;
    try {
      const id = await backend.choose();
      if (id && !signal.aborted && intent === selection) await open(id);
    } catch (error) { if (!signal.aborted) status(asError(error)); }
  }

  function navigate(direction: -1 | 1) {
    navigationDirection = direction;
    const id = cursor.step(direction);
    if (id && id !== source) void open(id, true);
  }

  function warmAdjacent() {
    if (!backend.warm || !details || entries.length < 2) return;
    const index = entries.findIndex(entry => entry.id === source);
    if (index < 0) return;
    const next = entries[(index + navigationDirection + entries.length) % entries.length];
    const request = warmRequests.start();
    void backend.warm(next.id, displaySize(), request).catch(() => {});
  }

  async function rename() {
    const current = details;
    const entry = source;
    if (renaming || !current?.document.sourcePath || !entry || renameInput.value === current.document.name) return;
    renaming = true;
    renameNotice.textContent = '';
    renameButton.disabled = renameInput.disabled = true;
    try {
      const document = await backend.rename(current.document.id, renameInput.value);
      if (signal.aborted) return;
      if (details === current) {
        current.document = document;
        options.onDocument?.(document, current.original);
        refreshText();
      }
      if (!entries.some(item => item.id === entry) && details !== current) return;
      const listing = directoryRequests.start();
      const result = await backend.directory(entry, listing);
      if (listing.aborted || signal.aborted) return;
      clearThumbnails();
      setEntries(result, source);
      scrollToSelected();
      refreshStrip();
    } catch (error) {
      if (!signal.aborted && details === current) {
        const failure = asError(error);
        renameNotice.textContent = failure.code === 'too-large' ? t('source-changed') : t(failure.code);
      }
    } finally {
      renaming = false;
      renameButton.disabled = renameInput.disabled = false;
      if (!signal.aborted && details === current && showMetadata) renameInput.focus({ preventScroll: true });
    }
  }

  renameForm.addEventListener('submit', event => event.preventDefault(), { signal });
  renameButton.addEventListener('click', () => void rename(), { signal });
  renameInput.addEventListener('keydown', event => {
    if (event.key === 'Enter' && !event.isComposing) {
      event.preventDefault();
      void rename();
      return;
    }
    if (event.key !== 'Escape') return;
    event.preventDefault();
    renameInput.value = details?.document.name ?? '';
    renameNotice.textContent = '';
    renameInput.blur();
  }, { signal });

  function dispatch(action: ShortcutAction | 'closeFullscreen') {
    switch (action) {
      case 'open': void choose(); break;
      case 'previous': navigate(-1); break;
      case 'next': navigate(1); break;
      case 'fit': resetScale(); break;
      case 'actual': resetScale(1); break;
      case 'rotateLeft': case 'rotateRight': case 'flipHorizontal': case 'flipVertical': transform(action); break;
      case 'fullscreen': fullscreen('toggle'); break;
      case 'closeFullscreen': fullscreen('close'); break;
      case 'escape': fullscreen('exit'); break;
      case 'info': setMetadata(!showMetadata); break;
      case 'thumbnails': setStrip(!showStrip); break;
      case 'paste': void load(); break;
      case 'copy':
        if (details) void backend.copy(details.document.id).catch(error => { if (!signal.aborted) status(asError(error)); });
        break;
      default: return false;
    }
    return true;
  }

  picture.addEventListener('dblclick', () => fullscreen('toggle'), { signal });
  stage.addEventListener('wheel', event => {
    if (!event.ctrlKey || !details || event.deltaY === 0) return;
    event.preventDefault();
    zoom(event.deltaY > 0 ? 0.8 : 1.25, event);
  }, { passive: false, signal });
  strip.addEventListener('scroll', refreshStrip, { passive: true, signal });
  const resize = new ResizeObserver(() => {
    layout();
    refreshStrip();
  });
  resize.observe(stage);
  const pointer = { id: -1, x: 0, y: 0 };
  stage.addEventListener('pointerdown', event => {
    if (event.target !== picture || event.button !== 0 || scale === undefined) return;
    pointer.id = event.pointerId;
    pointer.x = event.clientX - offset.x;
    pointer.y = event.clientY - offset.y;
    stage.setPointerCapture(event.pointerId);
  }, { signal });
  stage.addEventListener('pointermove', event => {
    if (event.pointerId !== pointer.id) return;
    offset = { x: event.clientX - pointer.x, y: event.clientY - pointer.y };
    layout();
  }, { signal });
  stage.addEventListener('lostpointercapture', () => { pointer.id = -1; }, { signal });
  document.addEventListener('keydown', event => {
    if (host.hidden || !host.isConnected || event.defaultPrevented || options.keyEnabled?.() === false) return;
    const editing = event.target instanceof Element && Boolean(event.target.closest('input, textarea, select, [contenteditable]:not([contenteditable="false"])'));
    const action = actionForKey(event, editing);
    if (action && (!event.repeat || action === 'previous' || action === 'next')
      && dispatch(event.key.toLowerCase() === 'w' ? 'closeFullscreen' : action)) event.preventDefault();
  }, { signal });
  refreshText();
  status('empty');
  setMetadata(false);
  setStrip(false);
  setEntries([]);
  layout();
  return {
    open,
    choose,
    dispatch,
    setLocale(value) { locale = value; refreshText(); },
    setDisplayPreferences(mode, allowUpscale) {
      if (zoomMode === mode && upscale === allowUpscale) return;
      zoomMode = mode;
      upscale = allowUpscale;
      resetScale(zoomMode === 'actual' ? 1 : undefined);
    },
    dispose() {
      controller.abort();
      requests.cancel();
      directoryRequests.cancel();
      detailRequests.cancel();
      warmRequests.cancel();
      clearThumbnails();
      resize.disconnect();
      if (previewUrl) URL.revokeObjectURL(previewUrl);
      if (detailUrl) URL.revokeObjectURL(detailUrl);
      root.remove();
    },
  };
}

