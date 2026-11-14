import { test } from 'node:test';
import assert from 'node:assert/strict';
import { EditHistory, cropSelection, imagePoint, zoomAt, editorAvailability, requestLeave, saveEdit } from './session.ts';
import type { ImageDocument } from '../../shared/contracts.ts';

const document: ImageDocument = {
  id: 'sample', name: 'sample.png', sourcePath: 'C:/sample.png',
  width: 100, height: 75, frameCount: 1, sizeBytes: 500,
  preview: { id: 'sample-resource', mimeType: 'image/png' },
};

test('400% zoom and panning map to source pixel edges', () => {
  const view = zoomAt({ x: 10, y: 20, scale: 1 }, { x: 30, y: 40 }, 4);
  assert.deepEqual(view, { x: -50, y: -40, scale: 4 });
  assert.deepEqual(imagePoint({ ...view, x: view.x + 17 }, { x: 7, y: 8 }, document), { x: 10, y: 12 });
});

test('reverse drag yields an exact 4:3 crop inside the image', () => {
  assert.deepEqual(cropSelection({ x: 89, y: 59 }, { x: 10, y: 3 }, [4, 3]), {
    kind: 'crop', x: 17, y: 5, width: 72, height: 54,
  });
});

test('history tracks the saved revision across undo, redo and branching', () => {
  const history = new EditHistory(document, 'session');
  history.push({ kind: 'rotate-right' });
  history.markSaved(history.revision);
  history.push({ kind: 'flip-x' });
  assert.equal(history.session.dirty, true);
  history.undo();
  assert.deepEqual(history.operations, [{ kind: 'rotate-right' }]);
  assert.equal(history.session.dirty, false);
  history.redo();
  assert.equal(history.session.dirty, true);
  history.undo();
  history.push({ kind: 'flip-y' });
  assert.equal(history.canRedo, false);
  assert.equal(history.session.dirty, true);
});

test('SVG and animated documents are blocked before opening a raster session', () => {
  assert.equal(editorAvailability({ ...document, name: 'A.SVG', preview: { ...document.preview, mimeType: 'image/png' } }), 'vector');
  assert.equal(editorAvailability({ ...document, frameCount: 2 }), 'animated');
  assert.equal(editorAvailability(document), null);
});

test('canceled overwrite keeps the history dirty and never calls the writer', async () => {
  const history = new EditHistory(document, 'session');
  history.push({ kind: 'rotate-right' });
  assert.equal(await saveEdit(history, 'overwrite', {
    confirmOverwrite: async () => false,
    save: async () => { assert.fail('unexpected write'); },
  }), false);
  assert.equal(history.session.dirty, true);
});

test('leave Save uses Save As and stays open after canceled path selection', async () => {
  const history = new EditHistory(document, 'session');
  history.push({ kind: 'flip-y' });
  let writes = 0;
  let pickerAccepted = false;
  const ports = {
    confirmOverwrite: async () => { assert.fail('unexpected overwrite'); },
    save: async (_operations: unknown, mode: 'overwrite' | 'as') => {
      assert.equal(mode, 'as');
      if (!pickerAccepted) return false;
      writes++;
      return true;
    },
  };
  assert.equal(await requestLeave(history, async () => 'cancel', ports), false);
  assert.equal(await requestLeave(history, async () => 'save', ports), false);
  pickerAccepted = true;
  assert.equal(await requestLeave(history, async () => 'save', ports), true);
  assert.equal(writes, 1);
  assert.equal(history.session.dirty, false);
});
