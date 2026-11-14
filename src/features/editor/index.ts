import type { ImageDocument, ImageResource, Outcome } from '../../shared/contracts.ts';
import type { Locale, ShortcutAction } from '../../shared/preferences.ts';
import { createIcon, type AssetIconName } from '../../assets/icons/index.ts';
import { editorLabel, type Label } from './labels.ts';
import { cropSelection, EditHistory, editorAvailability, imagePoint, requestLeave, saveEdit, zoomAt,
  type Crop, type Mosaic, type Operation, type Point, type SavePorts, type Shape, type View } from './session.ts';
import './editor.css';

export interface EditorBackend {
  render(operations: Operation[]): Promise<ArrayBuffer>;
  save(operations: Operation[], mode: 'overwrite' | 'as'): Promise<boolean>;
  exportImage(operations: Operation[]): Promise<Outcome<ImageResource>>;
  dispose(): void;
}
export interface EditorOptions {
  locale: Locale;
  open(document: ImageDocument): Promise<EditorBackend>;
  onCancel(): void;
  brushColor?: string;
  brushThickness?: number;
  mosaicBlockSize?: number;
}

export function editorUnavailableReason(document: ImageDocument, locale: Locale): string | null {
  const reason = editorAvailability(document);
  return reason && editorLabel(locale, reason);
}

function element<K extends keyof HTMLElementTagNameMap>(tag: K, className = '') {
  const node = document.createElement(tag);
  node.className = className;
  return node;
}

async function decodePng(bytes: ArrayBuffer) {
  return await createImageBitmap(new Blob([bytes], { type: 'image/png' }));
}

export async function mountEditor(host: HTMLElement, image: ImageDocument, options: EditorOptions) {
  const reason = editorUnavailableReason(image, options.locale);
  if (reason) throw new Error(reason);
  const backend = await options.open(image);
  try {
    return createEditor(host, image, options, backend, await decodePng(await backend.render([])));
  } catch (error) {
    backend.dispose();
    throw error;
  }
}

function createEditor(host: HTMLElement, image: ImageDocument, options: EditorOptions,
  backend: EditorBackend, initial: ImageBitmap) {
  const t = (key: Label) => editorLabel(options.locale, key);
  const icon = createIcon(key => key);
  const history = new EditHistory(image, crypto.randomUUID());
  const root = element('section', 'pg-editor');
  root.setAttribute('aria-label', t('editor'));
  const bar = element('div', 'pg-editor-bar');
  const stage = element('div', 'pg-editor-stage');
  const canvas = element('canvas');
  const ctx = canvas.getContext('2d');
  if (!ctx) { initial.close(); throw new Error('Canvas 2D unavailable'); }
  const zoom = element('output');
  const brushCursor = element('span', 'pg-editor-brush-cursor');
  brushCursor.hidden = true;
  brushCursor.setAttribute('aria-hidden', 'true');
  stage.append(canvas, zoom, brushCursor);
  const sidebar = element('aside', 'pg-editor-tools');
  const toolList = element('div', 'pg-editor-tool-list');
  const parameters = element('div', 'pg-editor-parameters');
  sidebar.append(toolList, parameters);
  const status = element('p', 'pg-editor-status');
  status.setAttribute('role', 'status');
  const dialog = element('dialog', 'pg-editor-dialog');
  root.append(bar, stage, sidebar, dialog);

  const tools = ['brush', 'shape', 'crop', 'mosaic', 'transform', 'pan'] as const;
  let tool: typeof tools[number] = 'brush';
  stage.dataset.tool = tool;
  let bitmap = initial;
  let view: View = { x: 0, y: 0, scale: 1 };
  let fitMode = true;
  let busy = false;
  let disposed = false;
  let dismissDialog: (() => void) | undefined;
  let crop: Crop | null = null;
  let ratio: readonly [number, number] | null = null;
  let shape: Shape = 'line';
  let mosaic: Mosaic = 'pixel';
  let tip: 'round' | 'square' = 'round';
  const defaultColor = options.brushColor ?? '#ff513c';
  let color = /^#[\da-f]{6}$/i.test(defaultColor) ? defaultColor.toLowerCase() : '#ff513c';
  let opacity = 100;
  const defaultThickness = options.brushThickness ?? 8;
  let strokeWidth = Number.isFinite(defaultThickness) ? Math.round(Math.max(1, Math.min(128, defaultThickness))) : 8;
  let radius = 18;
  let block = Math.round(Math.min(128, Math.max(2, options.mosaicBlockSize || 10)));
  let gesture: { id: number; start: Point; view: View; points: Point[]; pan: boolean } | null = null;
  let hoverAt: Point | null = null;

  function button(label: Label, action: () => void, shortcut?: string) {
    const node = element('button');
    node.type = 'button';
    node.textContent = t(label);
    if (shortcut) {
      node.title = `${t(label)} (${shortcut})`;
      node.setAttribute('aria-keyshortcuts', shortcut);
    }
    node.addEventListener('click', action);
    return node;
  }

  const undo = button('undo', () => void step('undo'), 'Control+Z');
  const redo = button('redo', () => void step('redo'), 'Control+Y Control+Shift+Z');
  const cancel = button('cancel', () => { void leave().then(allowed => { if (allowed) options.onCancel(); }); });
  const save = button('save', () => void store('overwrite'), 'Control+S');
  const saveAs = button('saveAs', () => void store('as'), 'Control+Shift+S');
  for (const [name, control] of [['undo', undo], ['redo', redo], ['cancel', cancel],
    ['save', save], ['saveAs', saveAs]] as const) control.prepend(icon({ name, size: 16 }));
  bar.append(undo, redo, status, element('span', 'pg-editor-spacer'), cancel, saveAs, save);

  function updateControls() {
    undo.disabled = busy || !history.canUndo;
    redo.disabled = busy || !history.canRedo;
    cancel.disabled = busy;
    save.disabled = busy || !history.session.dirty || !image.sourcePath;
    saveAs.disabled = busy;
    sidebar.inert = busy;
  }

  async function run(work: () => Promise<void>) {
    if (disposed || busy) return false;
    busy = true;
    gesture = null;
    status.textContent = t('loading');
    updateControls();
    try {
      await work();
      status.textContent = '';
      return true;
    } catch {
      status.textContent = t('failed');
      return false;
    } finally {
      busy = false;
      updateControls();
      draw();
    }
  }

  async function refresh(operations: Operation[]) {
    const next = await decodePng(await backend.render(operations));
    if (disposed) { next.close(); return; }
    const changedSize = bitmap.width !== next.width || bitmap.height !== next.height;
    bitmap.close();
    bitmap = next;
    if (changedSize) fit();
  }

  async function apply(operation: Operation) {
    await run(async () => {
      await refresh([...history.operations, operation]);
      if (!disposed) history.push(operation);
      crop = null;
      showParameters();
    });
  }

  async function step(direction: 'undo' | 'redo') {
    if (direction === 'undo' ? !history.canUndo : !history.canRedo) return;
    await run(async () => {
      history[direction]();
      try {
        await refresh(history.operations);
        crop = null;
        showParameters();
      } catch (error) {
        history[direction === 'undo' ? 'redo' : 'undo']();
        throw error;
      }
    });
  }

  function ask<T extends Label>(question: Label, choices: T[], fallback: T): Promise<T> {
    const heading = element('p');
    heading.textContent = t(question);
    heading.id = `editor-dialog-${history.id}`;
    dialog.setAttribute('aria-labelledby', heading.id);
    const actions = element('div', 'pg-editor-dialog-actions');
    return new Promise(resolve => {
      function choose(choice: T) {
        dismissDialog = undefined;
        dialog.oncancel = null;
        dialog.close();
        resolve(choice);
      }
      dismissDialog = () => choose(fallback);
      dialog.oncancel = event => { event.preventDefault(); choose(fallback); };
      for (const choice of choices) {
        const control = button(choice, () => choose(choice));
        control.value = choice;
        if (choice === fallback) control.autofocus = true;
        actions.append(control);
      }
      dialog.replaceChildren(heading, actions);
      dialog.showModal();
    });
  }

  const ports: SavePorts = {
    confirmOverwrite: async () => await ask('overwrite', ['cancel', 'save'], 'cancel') === 'save',
    save: (operations, mode) => backend.save(operations, mode),
  };

  async function withSave(action: () => Promise<boolean>) {
    let allowed = false;
    await run(async () => { allowed = await action(); });
    return allowed;
  }

  const store = (mode: 'overwrite' | 'as') => withSave(() => saveEdit(history, mode, ports));
  const leave = () => withSave(() => requestLeave(history,
    async () => {
      const choice = await ask('leave', ['cancel', 'discard', 'saveAs'], 'cancel');
      return choice === 'saveAs' ? 'save' : choice;
    }, ports));

  function select<T extends string>(label: Label, items: readonly (readonly [T, string])[], current: T, change: (value: T) => void) {
    const node = element('select');
    for (const [key, title] of items) node.add(new Option(title, key, false, key === current));
    node.addEventListener('change', () => change(node.value as T));
    field(label, node);
  }

  function field(label: Label, node: HTMLElement) {
    const row = element('label');
    row.append(t(label), node);
    parameters.append(row);
  }

  function number(label: Label, value: number, min: number, max: number, change?: (value: number) => void) {
    const node = element('input');
    node.type = 'number';
    node.min = String(min);
    node.max = String(max);
    node.value = String(value);
    node.addEventListener('change', () => {
      const next = Number.isFinite(node.valueAsNumber) ? Math.max(min, Math.min(max, Math.round(node.valueAsNumber))) : value;
      node.valueAsNumber = next;
      change?.(next);
    });
    field(label, node);
    return node;
  }

  function colorField() {
    const node = element('input');
    node.type = 'color';
    node.value = color;
    node.addEventListener('input', () => { color = node.value; });
    field('color', node);
  }

  function showParameters() {
    parameters.replaceChildren();
    if (tool === 'brush' || tool === 'shape') {
      colorField();
      number('width', strokeWidth, 1, 128, value => { strokeWidth = value; updateBrushCursor(); });
      number('opacity', opacity, 0, 100, value => { opacity = value; });
    }
    if (tool === 'brush') select('tip', [['round', t('round')], ['square', t('square')]], tip, value => {
      tip = value;
      updateBrushCursor();
    });
    if (tool === 'shape') select('shape', (['line', 'arrow', 'rectangle', 'ellipse'] as const).map(key => [key, t(key)]),
      shape, value => { shape = value; });
    if (tool === 'crop') {
      const ratios = { free: null, '1:1': [1, 1], '4:3': [4, 3], '16:9': [16, 9] } as const;
      const current = ratio ? `${ratio[0]}:${ratio[1]}` as keyof typeof ratios : 'free';
      select('ratio', (['free', '1:1', '4:3', '16:9'] as const).map(key => [key, key === 'free' ? t('free') : key]),
        current, value => { ratio = ratios[value]; crop = null; showParameters(); draw(); });
      const confirm = button('applyCrop', () => { if (crop) void apply(crop); });
      confirm.disabled = !crop;
      parameters.append(confirm);
    }
    if (tool === 'mosaic') {
      select('mosaic', (['pixel', 'blur', 'fill', 'noise', 'circle', 'glass'] as const).map(key => [key, t(key)]),
        mosaic, value => { mosaic = value; showParameters(); });
      number('radius', radius, 1, 128, value => { radius = value; updateBrushCursor(); });
      if (mosaic === 'fill') colorField();
      else number('block', block, 2, 128, value => { block = value; });
    }
    if (tool === 'transform') {
      for (const [label, kind] of [
        ['rotateLeft', 'rotate-left'], ['rotateRight', 'rotate-right'], ['flipX', 'flip-x'], ['flipY', 'flip-y'],
      ] as const) parameters.append(button(label, () => void apply({ kind })));
      const width = number('width', bitmap.width, 1, 32_768);
      const height = number('height', bitmap.height, 1, 32_768);
      parameters.append(button('applyResize', () => {
        if (width.reportValidity() && height.reportValidity()) {
          void apply({ kind: 'resize', width: width.valueAsNumber, height: height.valueAsNumber });
        }
      }));
      width.required = height.required = true;
    }
  }

  function fit() {
    const scale = Math.min(stage.clientWidth / bitmap.width, stage.clientHeight / bitmap.height, 1) || 1;
    view = { scale, x: (stage.clientWidth - bitmap.width * scale) / 2, y: (stage.clientHeight - bitmap.height * scale) / 2 };
    fitMode = true;
  }

  function draw() {
    if (!ctx || disposed) return;
    updateBrushCursor();
    const dpi = window.devicePixelRatio;
    const width = Math.round(stage.clientWidth * dpi);
    const height = Math.round(stage.clientHeight * dpi);
    if (!width || !height) return;
    if (canvas.width !== width) canvas.width = width;
    if (canvas.height !== height) canvas.height = height;
    ctx.setTransform(dpi, 0, 0, dpi, 0, 0);
    ctx.clearRect(0, 0, stage.clientWidth, stage.clientHeight);
    ctx.translate(view.x, view.y);
    ctx.scale(view.scale, view.scale);
    ctx.imageSmoothingEnabled = view.scale < 1;
    ctx.drawImage(bitmap, 0, 0);
    ctx.save();
    ctx.beginPath();
    ctx.rect(0, 0, bitmap.width, bitmap.height);
    ctx.clip();
    if (crop && tool === 'crop') {
      ctx.fillStyle = '#0008';
      ctx.beginPath();
      ctx.rect(0, 0, bitmap.width, bitmap.height);
      ctx.rect(crop.x, crop.y, crop.width, crop.height);
      ctx.fill('evenodd');
      ctx.strokeStyle = '#fff';
      ctx.lineWidth = 1 / view.scale;
      ctx.strokeRect(crop.x, crop.y, crop.width, crop.height);
    }
    if (gesture && !gesture.pan && tool !== 'crop') drawGesture(ctx, gesture.points);
    ctx.restore();
    zoom.value = `${Math.round(view.scale * 100)}%`;
  }

  function drawGesture(context: CanvasRenderingContext2D, points: Point[]) {
    const first = points[0];
    const last = points[points.length - 1];
    context.beginPath();
    context.strokeStyle = tool === 'mosaic' && mosaic !== 'fill' ? '#c7d8ff' : color;
    context.fillStyle = context.strokeStyle;
    context.globalAlpha = tool === 'mosaic' ? 0.5 : opacity / 100;
    context.lineWidth = tool === 'mosaic' ? radius * 2 : strokeWidth;
    context.lineCap = tool === 'brush' && tip === 'square' ? 'square' : 'round';
    context.lineJoin = 'round';
    if (tool === 'shape') {
      const dx = last.x - first.x;
      const dy = last.y - first.y;
      if (shape === 'rectangle') context.rect(first.x, first.y, dx, dy);
      else if (shape === 'ellipse') context.ellipse(first.x + dx / 2, first.y + dy / 2, Math.abs(dx) / 2, Math.abs(dy) / 2, 0, 0, 2 * Math.PI);
      else {
        context.moveTo(first.x, first.y);
        context.lineTo(last.x, last.y);
        if (shape === 'arrow') {
          const length = Math.min(30, Math.hypot(dx, dy) * 0.28);
          const angle = Math.atan2(dy, dx);
          for (const offset of [-0.5, 0.5]) {
            context.moveTo(last.x, last.y);
            context.lineTo(last.x - length * Math.cos(angle + offset), last.y - length * Math.sin(angle + offset));
          }
        }
      }
    } else if (points.length === 1) {
      const r = context.lineWidth / 2;
      if (tool === 'brush' && tip === 'square') context.rect(first.x - r, first.y - r, 2 * r, 2 * r);
      else context.arc(first.x, first.y, r, 0, 2 * Math.PI);
      context.fill();
      return;
    } else {
      context.moveTo(first.x, first.y);
      for (const point of points.slice(1)) context.lineTo(point.x, point.y);
    }
    context.stroke();
  }

  function local({ clientX, clientY }: Pick<MouseEvent, 'clientX' | 'clientY'>): Point {
    const rect = stage.getBoundingClientRect();
    return { x: clientX - rect.left, y: clientY - rect.top };
  }

  function insideImage({ x, y }: Point) {
    return x >= view.x && y >= view.y
      && x < view.x + bitmap.width * view.scale && y < view.y + bitmap.height * view.scale;
  }

  function updateBrushCursor() {
    const at = hoverAt && !gesture?.pan && !dialog.open && (tool === 'brush' || tool === 'mosaic')
      ? local({ clientX: hoverAt.x, clientY: hoverAt.y }) : null;
    if (!at || at.x < 0 || at.y < 0 || at.x >= stage.clientWidth || at.y >= stage.clientHeight || !insideImage(at)) {
      brushCursor.hidden = true;
      stage.removeAttribute('data-brush-cursor');
      return;
    }
    const size = (tool === 'mosaic' ? radius * 2 : strokeWidth) * view.scale;
    brushCursor.style.left = `${at.x}px`;
    brushCursor.style.top = `${at.y}px`;
    brushCursor.style.width = brushCursor.style.height = `${size}px`;
    brushCursor.toggleAttribute('data-square', tool === 'brush' && tip === 'square');
    brushCursor.hidden = false;
    stage.toggleAttribute('data-brush-cursor', true);
  }

  function trackHover(event: PointerEvent) {
    hoverAt = event.pointerType === 'touch' ? null : { x: event.clientX, y: event.clientY };
  }

  stage.addEventListener('wheel', event => {
    if (!event.ctrlKey) return;
    event.preventDefault();
    if (gesture) return;
    const magnitude = event.deltaY * (event.deltaMode === 0 ? 0.0025 : 0.04);
    view = zoomAt(view, local(event), Math.max(Math.min(view.scale, 0.05), Math.min(16, view.scale * Math.exp(-magnitude))));
    fitMode = false;
    draw();
  }, { passive: false });

  stage.addEventListener('pointerdown', event => {
    if (busy || gesture || (event.button !== 0 && event.button !== 1 && event.button !== 2)) return;
    const pan = event.button !== 0 || tool === 'pan';
    const start = local(event);
    if (!pan && (tool === 'transform' || !insideImage(start))) return;
    trackHover(event);
    stage.setPointerCapture(event.pointerId);
    gesture = { id: event.pointerId, start, pan, view, points: [imagePoint(view, start, bitmap)] };
    if (tool === 'crop' && !pan) { crop = null; showParameters(); }
    draw();
  });

  function move(event: PointerEvent) {
    if (gesture && event.pointerId !== gesture.id) return;
    trackHover(event);
    if (!gesture) {
      updateBrushCursor();
      return;
    }
    const at = local(event);
    if (gesture.pan) {
      view = { scale: view.scale, x: gesture.view.x + at.x - gesture.start.x, y: gesture.view.y + at.y - gesture.start.y };
      fitMode = false;
    } else {
      const next = imagePoint(view, at, bitmap);
      const first = gesture.points[0];
      const last = gesture.points[gesture.points.length - 1];
      if (tool === 'crop') crop = cropSelection(first, next, ratio);
      else if (tool === 'shape') gesture.points[1] = next;
      else if ((next.x !== last.x || next.y !== last.y) && gesture.points.length < 32_768) gesture.points.push(next);
    }
    draw();
  }

  stage.addEventListener('pointerenter', event => {
    trackHover(event);
    updateBrushCursor();
  });
  stage.addEventListener('pointermove', move);
  function clearHover() {
    hoverAt = null;
    updateBrushCursor();
  }
  stage.addEventListener('pointerleave', clearHover);
  stage.addEventListener('pointercancel', clearHover);
  stage.addEventListener('contextmenu', event => event.preventDefault());
  stage.addEventListener('pointerup', event => {
    if (!gesture || event.pointerId !== gesture.id) return;
    move(event);
    const { points, pan } = gesture;
    gesture = null;
    updateBrushCursor();
    if (pan) return;
    const [r, g, b] = [1, 3, 5].map(index => parseInt(color.slice(index, index + 2), 16));
    const rgba: [number, number, number, number] = [r, g, b, Math.round(opacity * 255 / 100)];
    if (tool === 'crop') showParameters();
    if (tool === 'brush') void apply({ kind: 'brush', points, tip, color: rgba, width: strokeWidth });
    if (tool === 'shape') void apply({ kind: 'shape', shape, from: points[0], to: points.at(-1)!, color: rgba, width: strokeWidth });
    if (tool === 'mosaic') void apply({ kind: 'mosaic', mode: mosaic, points, radius, block,
      color: [r, g, b, 255], seed: crypto.getRandomValues(new Uint32Array(1))[0] });
  });
  stage.addEventListener('lostpointercapture', event => {
    if (gesture?.id !== event.pointerId) return;
    gesture = null;
    if (tool === 'crop') { crop = null; showParameters(); }
    draw();
  });

  for (const name of tools) {
    const entry = button(name, () => {
      tool = name;
      gesture = null;
      crop = null;
      stage.dataset.tool = name;
      for (const child of toolList.children) child.setAttribute('aria-pressed', String(child === entry));
      showParameters();
      draw();
    });
    const toolIcons: Partial<Record<typeof tools[number], AssetIconName>> = {
      brush: 'brush', shape: 'shapes', crop: 'crop', mosaic: 'mosaic', transform: 'resize',
    };
    const iconName = toolIcons[name];
    if (iconName) entry.prepend(icon({ name: iconName, size: 16 }));
    entry.setAttribute('aria-pressed', String(name === tool));
    toolList.append(entry);
  }

  host.replaceChildren(root);
  fit();
  showParameters();
  updateControls();
  draw();
  const observer = new ResizeObserver(() => { if (fitMode) fit(); draw(); });
  observer.observe(stage);

  return {
    get session() { return history.session; },
    handleShortcut(action: ShortcutAction): boolean {
      switch (action) {
        case 'undo': void step('undo'); return true;
        case 'redo': void step('redo'); return true;
        case 'save': if (!save.disabled) void store('overwrite'); return true;
        case 'saveAs': void store('as'); return true;
        case 'escape': if (!dialog.open) cancel.click(); return true;
        case 'fit': fit(); draw(); return true;
        case 'actual': view = zoomAt(view, { x: stage.clientWidth / 2, y: stage.clientHeight / 2 }, 1);
          fitMode = false; draw(); return true;
        default: return false;
      }
    },
    requestLeave: leave,
    exportImage(): Promise<Outcome<ImageResource>> {
      return busy || disposed ? Promise.resolve({ ok: false, error: { code: 'busy' } }) : backend.exportImage(history.operations);
    },
    dispose() {
      disposed = true;
      dismissDialog?.();
      observer.disconnect();
      bitmap.close();
      root.remove();
      backend.dispose();
    },
  };
}
