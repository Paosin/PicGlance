import { invoke } from '@tauri-apps/api/core';
import type { ImageDocument, ImageResource } from '../../shared/contracts.ts';

export type ViewerError = {
  code: 'cancelled' | 'unsupported' | 'too-large' | 'invalid-input' | 'io' | 'busy' | 'missing' | 'dimensions'
    | 'clipboard-empty' | 'name-exists' | 'invalid-name' | 'source-changed';
  sizeBytes?: number;
};
export interface DirectoryEntry { id: string; name: string }
export interface ImageDetails {
  document: ImageDocument;
  original: ImageResource;
  metadata: { tag: string; value: string }[];
  gps: boolean;
}
export interface DecodeSize { width: number; height: number }
export interface ViewerBackend {
  choose(): Promise<string | null>;
  open(id: string, size: DecodeSize, signal: AbortSignal): Promise<ImageDetails>;
  warm?(id: string, size: DecodeSize, signal: AbortSignal): Promise<void>;
  directory(id: string, signal: AbortSignal): Promise<DirectoryEntry[]>;
  rename(id: string, name: string): Promise<ImageDocument>;
  thumbnail(id: string, signal: AbortSignal): Promise<ArrayBuffer>;
  resource(id: string, signal?: AbortSignal): Promise<ArrayBuffer>;
  copy(id: string): Promise<void>;
  paste(size: DecodeSize, signal: AbortSignal): Promise<ImageDetails>;
}

let sequence = 0;
async function cancellable<T>(command: string, args: Record<string, unknown>, signal: AbortSignal): Promise<T> {
  const request = String(++sequence);
  const abort = () => { void invoke('viewer_cancel', { request }).catch(() => {}); };
  signal.throwIfAborted();
  signal.addEventListener('abort', abort, { once: true });
  try {
    const value = await invoke<T>(command, { ...args, request });
    signal.throwIfAborted();
    return value;
  } finally {
    signal.removeEventListener('abort', abort);
  }
}

export const tauriViewerBackend: ViewerBackend = {
  choose: () => invoke('viewer_choose'),
  open: (id, size, signal) => cancellable('viewer_open', { id, size }, signal),
  warm: async (id, size, signal) => { await cancellable('viewer_open', { id, size, warm: true }, signal); },
  directory: (id, signal) => cancellable('viewer_directory', { id }, signal),
  rename: (id, name) => invoke('viewer_rename', { id, name }),
  thumbnail: (id, signal) => cancellable('viewer_thumbnail', { id }, signal),
  resource: (id, signal) => signal ? cancellable('viewer_resource', { id }, signal) : invoke('viewer_resource', { id }),
  copy: id => invoke('viewer_copy', { id }),
  paste: (size, signal) => cancellable('viewer_paste', { size }, signal),
};

