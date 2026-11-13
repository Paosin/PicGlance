type Point = { x: number; y: number };

export function fitScale(width: number, height: number, availableWidth: number, availableHeight: number, turns: number, upscale = false) {
  if (turns % 2) [width, height] = [height, width];
  return Math.max(0, Math.min(upscale ? Infinity : 1, availableWidth / width, availableHeight / height));
}

export function zoomAt(before: number, after: number, offset: Point, pointer: Point): Point {
  const ratio = after / before;
  return { x: pointer.x + (offset.x - pointer.x) * ratio, y: pointer.y + (offset.y - pointer.y) * ratio };
}

export function previewMatrix(turns: number, horizontal: boolean, vertical: boolean) {
  const [a, b, c, d] = [[1, 0, 0, 1], [0, 1, -1, 0], [-1, 0, 0, -1], [0, -1, 1, 0]][(turns % 4 + 4) % 4];
  const x = horizontal ? -1 : 1;
  const y = vertical ? -1 : 1;
  return [a * x || 0, b * y || 0, c * x || 0, d * y || 0];
}

export class DirectoryCursor {
  private ids: string[] = [];
  private index = -1;
  private removed = false;

  reset(ids: string[], selected: string) {
    this.ids = ids.slice();
    this.select(selected);
  }

  select(id: string) {
    this.index = this.ids.indexOf(id);
    this.removed = false;
  }

  step(direction: -1 | 1) {
    if (!this.ids.length) return;
    const next = this.index < 0 ? (direction > 0 ? 0 : this.ids.length - 1)
      : this.index + (this.removed && direction === 1 ? 0 : direction);
    this.index = (next + this.ids.length) % this.ids.length;
    this.removed = false;
    return this.ids[this.index];
  }

  remove(id: string) {
    const index = this.ids.indexOf(id);
    if (index < 0) return;
    this.ids.splice(index, 1);
    if (index < this.index) this.index--;
    else if (index === this.index) this.removed = true;
  }
}

export class LatestRequest {
  private current?: AbortController;

  start() {
    this.cancel();
    this.current = new AbortController();
    return this.current.signal;
  }

  cancel() {
    this.current?.abort();
    this.current = undefined;
  }
}

