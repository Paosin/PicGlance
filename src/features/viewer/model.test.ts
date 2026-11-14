import { test } from 'node:test';
import assert from 'node:assert/strict';
import { fitScale, zoomAt, previewMatrix, DirectoryCursor, LatestRequest } from './model.ts';

test('fit preserves small images and accounts for quarter turns', () => {
  assert.equal(fitScale(64, 64, 800, 600, 0), 1);
  assert.equal(fitScale(1200, 400, 800, 600, 1), 0.5);
});

test('zoom keeps the same source point under the pointer', () => {
  assert.deepEqual(zoomAt(0.5, 1, { x: 20, y: 10 }, { x: 60, y: 30 }), { x: -20, y: -10 });
});

test('preview matrices rotate and flip without touching the document', () => {
  assert.deepEqual(previewMatrix(1, false, false), [0, 1, -1, 0]);
  assert.deepEqual(previewMatrix(1, true, false), [0, 1, 1, 0]);
});

test('directory navigation wraps and continues after a removed file', () => {
  const cursor = new DirectoryCursor();
  cursor.reset(['a', 'b', 'c'], 'a');
  assert.equal(cursor.step(-1), 'c');
  assert.equal(cursor.step(1), 'a');
  assert.equal(cursor.step(1), 'b');
  cursor.remove('b');
  assert.equal(cursor.step(1), 'c');
  cursor.remove('c');
  assert.equal(cursor.step(-1), 'a');
  cursor.remove('a');
  assert.equal(cursor.step(1), undefined);
});

test('a new request aborts the previous request and makes its result stale', () => {
  const requests = new LatestRequest();
  const first = requests.start();
  const second = requests.start();
  assert.equal(first.aborted, true);
  assert.equal(second.aborted, false);
  requests.cancel();
  assert.equal(second.aborted, true);
});

