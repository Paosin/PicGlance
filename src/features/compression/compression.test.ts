import assert from 'node:assert/strict';
import test from 'node:test';
import { formatsFor, reductionPercent } from './compression.ts';

test('prioritizes the source format and keeps animation-compatible options', () => {
  assert.deepEqual(formatsFor('image/png', 1), ['png', 'jpeg', 'webp']);
  assert.deepEqual(formatsFor('image/gif', 1), ['gif', 'webp']);
  assert.deepEqual(formatsFor('image/webp', 2), ['webp', 'gif']);
  assert.deepEqual(formatsFor('image/bmp', 1), ['jpeg', 'png', 'webp']);
  assert.deepEqual(formatsFor('image/svg+xml', 1), []);
});

test('displays growth as a negative reduction', () => {
  assert.equal(reductionPercent(100, 120), -20);
  assert.equal(reductionPercent(100, 20), 80);
});
