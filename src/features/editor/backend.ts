import { invoke } from '@tauri-apps/api/core';
import type { ImageDocument, ImageResource, Outcome } from '../../shared/contracts.ts';
import type { EditorBackend } from './index.ts';
import type { Operation } from './session.ts';

export async function openNativeEditor(document: ImageDocument): Promise<EditorBackend> {
  const id = await invoke<string>('editor_open', { documentId: document.id });
  return {
    render: operations => invoke<ArrayBuffer>('editor_render', { id, operations }),
    save: (operations, mode) => invoke<boolean>('editor_save', { id, operations, mode }),
    async exportImage(operations: Operation[]): Promise<Outcome<ImageResource>> {
      try {
        return { ok: true, value: await invoke<ImageResource>('editor_export', { id, operations }) };
      } catch {
        return { ok: false, error: { code: 'io' } };
      }
    },
    dispose: () => { void invoke('editor_close', { id }).catch(() => {}); },
  };
}
