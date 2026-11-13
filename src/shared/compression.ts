import { Channel, invoke } from '@tauri-apps/api/core';
import type { CompressionPorts, CompressionResult } from '../features/compression/index.ts';
import type { ErrorCode, Outcome } from './contracts.ts';

const codes: ReadonlySet<string> = new Set(['cancelled', 'invalid-input', 'unsupported', 'too-large', 'io', 'busy']);
async function outcome<T>(request: Promise<T>): Promise<Outcome<T>> {
  try { return { ok: true, value: await request }; }
  catch (error) {
    const code = error && typeof error === 'object' && 'code' in error && typeof error.code === 'string'
      && codes.has(error.code) ? error.code as ErrorCode : 'io';
    return { ok: false, error: { code } };
  }
}

export const nativeCompression: CompressionPorts = {
  async start(resource, request) {
    const reserved = await outcome(invoke<string>('compression_reserve'));
    if (!reserved.ok) return reserved;
    const jobId = reserved.value;
    const progress = new Channel<number>();
    let latest = 0;
    const listeners = new Set<(value: number) => void>();
    progress.onmessage = value => {
      latest = value;
      for (const listener of listeners) listener(value);
    };
    const cancel = () => invoke<void>('compression_cancel', { jobId });
    const result = outcome(invoke<CompressionResult>('compression_run', { jobId, sourceId: resource.id, request, progress }))
      .then(value => {
        if (!value.ok) void cancel().catch(console.error);
        return value;
      });
    return {
      ok: true,
      value: {
        result, cancel,
        subscribe(listener) {
          listeners.add(listener);
          listener(latest);
          return () => { listeners.delete(listener); };
        },
      },
    };
  },
  read: id => outcome(invoke<ArrayBuffer>('compression_read', { id })),
  save: (resource, suggestedName) => outcome(invoke<void>('compression_save', { id: resource.id, suggestedName })),
};
