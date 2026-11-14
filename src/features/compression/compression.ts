import type { CompressionOptions, ImageDocument, ImageResource, Outcome } from '../../shared/contracts.ts';
import type { Locale } from '../../shared/preferences.ts';
import { createIcon } from '../../assets/icons/index.ts';


export type OutputFormat = 'jpeg' | 'png' | 'gif' | 'webp';

export interface CompressionRequest extends CompressionOptions {
  format: OutputFormat;
  maxQuality: number;
  allowFrameDrop: boolean;
}

export interface CompressionResult {
  resource: ImageResource;
  sizeBytes: number;
  width: number;
  height: number;
  frameCount: number;
  quality: number;
  metTarget: boolean;
  originalColorCount?: number | null;
  originalBitDepth?: number | null;
  outputColorCount?: number | null;
  outputBitDepth?: number | null;
}

interface CompressionJob {
  result: Promise<Outcome<CompressionResult>>;
  cancel(): Promise<void>;
  subscribe(listener: (progress: number) => void): () => void;
}

export interface CompressionPorts {
  start(resource: ImageResource, request: CompressionRequest): Promise<Outcome<CompressionJob>>;
  read(id: string): Promise<Outcome<ArrayBuffer>>;
  save(resource: ImageResource, suggestedName: string): Promise<Outcome<void>>;
}

export function formatsFor(mimeType: string, frames: number): OutputFormat[] {
  switch (mimeType.toLowerCase()) {
    case 'image/jpeg': return ['jpeg', 'png', 'webp'];
    case 'image/png': return frames > 1 ? [] : ['png', 'jpeg', 'webp'];
    case 'image/bmp':
    case 'image/x-ms-bmp': return ['jpeg', 'png', 'webp'];
    case 'image/gif': return ['gif', 'webp'];
    case 'image/webp': return frames > 1 ? ['webp', 'gif'] : ['webp', 'jpeg', 'png'];
    default: return [];
  }
}

export function reductionPercent(sourceBytes: number, resultBytes: number): number {
  return sourceBytes > 0 ? Math.round((sourceBytes - resultBytes) / sourceBytes * 100) : 0;
}

const labels = {
  'zh-CN': {
    title: '目标体积压缩', original: '原图', result: '压缩结果', resultQuality: '质量', frames: '帧', target: '目标大小（KiB）',
    format: '输出格式', keep: '保持尺寸', recommended: '（推荐）', quality: '最高质量',
    drop: '允许 WebP 抽帧', run: '压缩', cancel: '取消', close: '关闭', save: '另存为…',
    running: '正在压缩…', cancelling: '正在取消…', unsupported: '此图片格式暂不支持智能压缩。',
    colors: '调色板颜色数', bitDepth: '位深（位/像素）',
    failed: '压缩失败。', previewFailed: '无法加载对比预览。', saveFailed: '保存失败。',
    missed: '未达到目标大小。下方显示本次最小结果。', reduced: '减少', grown: '增加',
  },
  en: {
    title: 'Target size compression', original: 'Original', result: 'Compressed', resultQuality: 'Quality', frames: 'frames', target: 'Target size (KiB)',
    format: 'Output format', keep: 'Keep dimensions', recommended: ' (recommended)', quality: 'Maximum quality',
    drop: 'Allow WebP frame dropping', run: 'Compress', cancel: 'Cancel', close: 'Close', save: 'Save as…',
    running: 'Compressing…', cancelling: 'Cancelling…', unsupported: 'Compression is unavailable for this image format.',
    colors: 'Palette colors', bitDepth: 'Bit depth (bits/pixel)',
    failed: 'Compression failed.', previewFailed: 'Could not load the comparison preview.', saveFailed: 'Could not save.',
    missed: 'Target size could not be reached. The smallest result is shown below.', reduced: 'reduction', grown: 'increase',
  },
};

function element<K extends keyof HTMLElementTagNameMap>(tag: K, text = ''): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  node.textContent = text;
  return node;
}

function field(text: string, input: HTMLElement): HTMLLabelElement {
  const label = element('label');
  label.append(text, input);
  return label;
}

function input(type: string, value = ''): HTMLInputElement {
  const node = element('input');
  node.type = type;
  node.value = value;
  return node;
}

export function createCompressionDialog(source: ImageDocument, original: ImageResource, ports: CompressionPorts, locale: Locale) {
  const t = labels[locale];
  const dialog = element('dialog');
  dialog.className = 'compression-dialog';
  dialog.setAttribute('aria-label', t.title);
  const status = element('p');
  status.setAttribute('role', 'status');

  const formats = formatsFor(original.mimeType, source.frameCount);
  const target = input('number', '500');
  target.min = '1';
  target.max = String(Math.floor(Number.MAX_SAFE_INTEGER / 1024));
  target.required = true;
  const format = element('select');
  for (const [index, value] of formats.entries()) {
    format.add(new Option(value.toUpperCase() + (index === 0 ? t.recommended : ''), value));
  }
  const keep = input('checkbox');
  const quality = input('range', '95');
  quality.min = '10';
  quality.max = '100';
  const qualityValue = element('output', quality.value);
  quality.addEventListener('input', () => { qualityValue.value = quality.value; });
  const qualityField = field(t.quality, quality);
  qualityField.append(qualityValue);
  const drop = input('checkbox');
  const dropField = field(t.drop, drop);
  const animated = source.frameCount > 1;
  qualityField.hidden = !animated;
  const form = element('form');
  form.className = 'compression-controls';
  const run = element('button', t.run);
  const close = element('button');
  close.className = 'compression-close';
  close.type = 'button';
  close.title = t.close;
  close.setAttribute('aria-label', t.close);
  close.append(createIcon(key => key)({ name: 'close', size: 16 }));
  const cancelJob = element('button', t.cancel);
  cancelJob.type = 'button';
  const save = element('button', t.save);
  save.type = 'button';
  save.disabled = true;
  const progress = element('progress');
  progress.hidden = true;
  progress.max = 1;

  const comparison = element('div');
  comparison.className = 'compression-comparison';
  const originalImage = element('img');
  originalImage.alt = t.original;
  const resultImage = element('img');
  resultImage.alt = t.result;
  resultImage.hidden = true;
  for (const [title, img] of [[t.original, originalImage], [t.result, resultImage]] as const) {
    const figure = element('figure');
    figure.append(element('figcaption', title), img);
    comparison.append(figure);
  }
  const metadata = element('div');
  metadata.className = 'compression-metadata';
  form.append(field(t.target, target), field(t.format, format), field(t.keep, keep), qualityField, dropField);
  const actions = element('div');
  actions.className = 'compression-actions';
  actions.append(cancelJob, run, save);
  form.append(actions);
  const header = element('header');
  header.className = 'compression-header';
  header.append(element('h2', t.title), close);
  dialog.append(header, form, progress, status, comparison, metadata);

  let job: CompressionJob | undefined;
  let result: CompressionResult | undefined;
  let busy = false;
  let cancelled = false;
  let disposed = false;
  let resultPreview: string | undefined;
  const urls = new Set<string>();
  const untrack = (url: string) => {
    urls.delete(url);
    URL.revokeObjectURL(url);
  };

  function updateControls() {
    dropField.hidden = !animated || format.value !== 'webp';
    for (const control of [run, target, format, keep, quality, drop]) {
      control.disabled = busy || formats.length === 0;
    }
    cancelJob.hidden = !busy;
    cancelJob.disabled = cancelled;
    save.disabled = busy || !result;
    progress.hidden = !busy;
  }

  async function preview(resource: ImageResource, img: HTMLImageElement) {
    const response = await ports.read(resource.id);
    if (disposed) return;
    if (!response.ok) throw new Error(response.error.code);
    const url = URL.createObjectURL(new Blob([response.value], { type: resource.mimeType }));
    urls.add(url);
    img.src = url;
    img.hidden = false;
    return url;
  }

  function cancel() {
    if (!busy) {
      dialog.close();
      return;
    }
    if (cancelled) return;
    cancelled = true;
    status.textContent = t.cancelling;
    updateControls();
    void job?.cancel().catch(() => {});
  }

  function dispose() {
    disposed = true;
    if (busy && !cancelled) void job?.cancel().catch(() => {});
    for (const url of urls) URL.revokeObjectURL(url);
    urls.clear();
    dialog.remove();
  }

  async function compress() {
    if (busy || disposed || formats.length === 0 || !form.reportValidity()) return;
    cancelled = false;
    busy = true;
    result = undefined;
    resultImage.hidden = true;
    resultImage.removeAttribute('src');
    if (resultPreview) untrack(resultPreview);
    resultPreview = undefined;
    metadata.replaceChildren();
    progress.removeAttribute('value');
    status.textContent = t.running;
    updateControls();
    let unsubscribe = () => {};
    try {
      const started = await ports.start(original, {
        targetBytes: target.valueAsNumber * 1024,
        format: format.value as OutputFormat,
        lockDimensions: keep.checked,
        maxQuality: animated ? quality.valueAsNumber : 95,
        allowFrameDrop: !dropField.hidden && drop.checked,
      });
      if (!started.ok) {
        if (!cancelled && !disposed) status.textContent = started.error.code === 'unsupported' ? t.unsupported : t.failed;
        return;
      }
      job = started.value;
      if (cancelled || disposed) await job.cancel();
      else unsubscribe = job.subscribe((value) => {
        if (!cancelled && !disposed && Number.isFinite(value)) progress.value = Math.max(0, Math.min(1, value));
      });
      const completed = await job.result;
      if (disposed || cancelled) return;
      if (!completed.ok) {
        status.textContent = completed.error.code === 'cancelled' ? '' : t.failed;
        return;
      }
      result = completed.value;
      status.textContent = result.metTarget ? '' : t.missed;
      const percent = reductionPercent(source.sizeBytes, result.sizeBytes);
      const number = new Intl.NumberFormat(locale, { maximumFractionDigits: 1 });
      const size = (bytes: number) => number.format(bytes / 1024) + ' KiB';
      const resultSize = element('strong', size(result.sizeBytes));
      if (percent > 0) resultSize.className = 'compression-reduced-size';
      const difference = element('strong', locale === 'zh-CN'
        ? (percent < 0 ? t.grown : t.reduced) + Math.abs(percent) + '%'
        : Math.abs(percent) + '% ' + (percent < 0 ? t.grown : t.reduced));
      difference.className = 'compression-difference';
      metadata.append(size(source.sizeBytes), ' → ', resultSize, ' (', difference, ')',
        element('p', source.width + ' × ' + source.height + ' → ' + result.width + ' × ' + result.height
          + (animated ? ' · ' + source.frameCount + ' → ' + result.frameCount + ' ' + t.frames : '')
          + ' · ' + t.resultQuality + ' ' + result.quality));
      const stats = element('dl');
      stats.className = 'compression-color-stats';
      for (const [label, original, output] of [
        [t.colors, result.originalColorCount, result.outputColorCount],
        [t.bitDepth, result.originalBitDepth, result.outputBitDepth],
      ] as const) {
        if (original == null && output == null) continue;
        const display = (value: number | null | undefined) => value == null ? '—' : number.format(value);
        stats.append(element('dt', label), element('dd', display(original) + ' → ' + display(output)));
      }
      if (stats.childElementCount) metadata.append(stats);
      try {
        resultPreview = await preview(result.resource, resultImage);
        if (cancelled && resultPreview) {
          untrack(resultPreview);
          resultPreview = undefined;
          resultImage.hidden = true;
        }
      } catch {
        if (!cancelled && !disposed) status.textContent = [status.textContent, t.previewFailed].filter(Boolean).join(' ');
      }
    } catch {
      if (!cancelled && !disposed) status.textContent = t.failed;
    } finally {
      unsubscribe();
      job = undefined;
      busy = false;
      if (cancelled) {
        result = undefined;
        metadata.replaceChildren();
        status.textContent = '';
      }
      if (!disposed) updateControls();
    }
  }

  form.addEventListener('submit', (event) => {
    event.preventDefault();
    void compress();
  });
  format.addEventListener('change', updateControls);
  close.addEventListener('click', () => dialog.close());
  cancelJob.addEventListener('click', cancel);
  dialog.addEventListener('cancel', (event) => { event.preventDefault(); cancel(); });
  dialog.addEventListener('close', dispose, { once: true });
  save.addEventListener('click', async () => {
    if (!result) return;
    const extension = result.resource.mimeType === 'image/jpeg'
      ? 'jpg'
      : formatsFor(result.resource.mimeType, result.frameCount)[0];
    const name = source.name.replace(/\.[^.]+$/, '') + (extension ? '.' + extension : '');
    save.disabled = true;
    try {
      const saved = await ports.save(result.resource, name);
      if (!disposed && !saved.ok && saved.error.code !== 'cancelled') status.textContent = t.saveFailed;
    } catch {
      if (!disposed) status.textContent = t.saveFailed;
    } finally {
      if (!disposed) updateControls();
    }
  });

  updateControls();
  if (!formats.length) status.textContent = t.unsupported;
  void preview(animated && formats.length ? original : source.preview, originalImage).catch(() => {
    if (!disposed && !status.textContent) status.textContent = t.previewFailed;
  });
  return { dialog, dispose };
}
