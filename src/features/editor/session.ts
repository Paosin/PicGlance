import type { EditSession, ImageDocument } from '../../shared/contracts.ts';

export interface Point { x: number; y: number }
export interface Size { width: number; height: number }
export interface View extends Point { scale: number }
export type Shape = 'line' | 'arrow' | 'rectangle' | 'ellipse';
export type Mosaic = 'pixel' | 'blur' | 'fill' | 'noise' | 'circle' | 'glass';
type Color = [number, number, number, number];
interface Paint { color: Color; width: number }
export type Crop = { kind: 'crop' } & Point & Size;
export type Operation =
  | ({ kind: 'brush'; points: Point[]; tip: 'round' | 'square' } & Paint)
  | ({ kind: 'shape'; shape: Shape; from: Point; to: Point } & Paint)
  | { kind: 'mosaic'; mode: Mosaic; points: Point[]; radius: number; block: number; color: Color; seed: number }
  | Crop
  | { kind: 'resize'; width: number; height: number }
  | { kind: 'rotate-right' | 'rotate-left' | 'flip-x' | 'flip-y' };

export function editorAvailability(document: ImageDocument): 'vector' | 'animated' | null {
  if (/\.svgz?$/i.test(document.name) || /\.svgz?$/i.test(document.sourcePath ?? '') ||
      document.preview.mimeType.toLowerCase().split(';')[0] === 'image/svg+xml') return 'vector';
  return document.frameCount > 1 ? 'animated' : null;
}

export function zoomAt(view: View, anchor: Point, scale: number): View {
  const ratio = scale / view.scale;
  return { scale, x: anchor.x - (anchor.x - view.x) * ratio, y: anchor.y - (anchor.y - view.y) * ratio };
}

export function imagePoint(view: View, point: Point, size: Size): Point {
  return {
    x: Math.max(0, Math.min(size.width, (point.x - view.x) / view.scale)),
    y: Math.max(0, Math.min(size.height, (point.y - view.y) / view.scale)),
  };
}

export function cropSelection(from: Point, to: Point, ratio: readonly [number, number] | null): Crop | null {
  const start = { x: Math.round(from.x), y: Math.round(from.y) };
  const end = { x: Math.round(to.x), y: Math.round(to.y) };
  let width = Math.abs(end.x - start.x);
  let height = Math.abs(end.y - start.y);
  if (ratio) {
    const units = Math.floor(Math.min(width / ratio[0], height / ratio[1]));
    width = units * ratio[0];
    height = units * ratio[1];
  }
  if (!width || !height) return null;
  return { kind: 'crop', x: start.x - (end.x < start.x ? width : 0),
    y: start.y - (end.y < start.y ? height : 0), width, height };
}

export class EditHistory {
  readonly document: ImageDocument;
  readonly id: string;
  private entries: { revision: number; operation: Operation }[] = [];
  private cursor = 0;
  private sequence = 0;
  private saved = 0;

  constructor(document: ImageDocument, id: string) {
    this.document = document;
    this.id = id;
  }

  get revision() { return this.entries[this.cursor - 1]?.revision ?? 0; }
  get session(): EditSession {
    return { id: this.id, documentId: this.document.id, dirty: this.revision !== this.saved };
  }
  get operations(): Operation[] { return this.entries.slice(0, this.cursor).map(entry => entry.operation); }
  get canUndo() { return this.cursor > 0; }
  get canRedo() { return this.cursor < this.entries.length; }

  push(operation: Operation) {
    this.entries.splice(this.cursor, this.entries.length - this.cursor, { operation, revision: ++this.sequence });
    this.cursor++;
  }

  undo() { if (this.canUndo) this.cursor--; }
  redo() { if (this.canRedo) this.cursor++; }
  markSaved(revision: number) { this.saved = revision; }
}

export interface SavePorts {
  confirmOverwrite(path: string): Promise<boolean>;
  save(operations: Operation[], mode: 'overwrite' | 'as'): Promise<boolean>;
}

export async function saveEdit(history: EditHistory, mode: 'overwrite' | 'as', ports: SavePorts): Promise<boolean> {
  const revision = history.revision;
  const operations = history.operations;
  if (mode === 'overwrite') {
    if (!history.document.sourcePath || !(await ports.confirmOverwrite(history.document.sourcePath))) return false;
  }
  if (!(await ports.save(operations, mode))) return false;
  history.markSaved(revision);
  return true;
}

export async function requestLeave(
  history: EditHistory, prompt: () => Promise<'save' | 'discard' | 'cancel'>, ports: SavePorts,
): Promise<boolean> {
  if (!history.session.dirty) return true;
  const choice = await prompt();
  if (choice === 'save') return await saveEdit(history, 'as', ports) && !history.session.dirty;
  return choice === 'discard';
}
