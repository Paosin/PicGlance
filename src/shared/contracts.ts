import type { Locale, ThemeMode } from './preferences.ts';
import type { Message } from './messages.ts';
import type { AssetIconName } from '../assets/icons/index.ts';

export type ErrorCode = 'cancelled' | 'invalid-input' | 'unsupported' | 'too-large' | 'io' | 'busy';
export type Outcome<T> = { ok: true; value: T } | { ok: false; error: { code: ErrorCode } };

export interface ImageResource {
  id: string;
  mimeType: string;
}

export interface ImageDocument {
  id: string;
  sourcePath: string | null;
  name: string;
  width: number;
  height: number;
  frameCount: number;
  sizeBytes: number;
  /** Display resource that may be downsampled. */
  preview: ImageResource;
}

export type ReadImageResource = (id: ImageResource['id']) => Promise<Outcome<ArrayBuffer>>;
/** Opens a source identified by an opaque ID granted by the native host. */
export type OpenImage = (authorizedId: string) => Promise<Outcome<ImageDocument>>;
export interface ViewerState { document: ImageDocument | null; position: number; count: number }
export interface EditSession { id: string; documentId: string; dirty: boolean }
export type ExportImage = (sessionId: string) => Promise<Outcome<ImageResource>>;
export interface CompressionJob {
  id: string;
  result: Promise<Outcome<ImageResource>>;
  cancel(): Promise<void>;
  subscribe(listener: (progress: number) => void): () => void;
}
export interface CompressionOptions { targetBytes: number; lockDimensions: boolean }
export type StartCompression = (resource: ImageResource, options: CompressionOptions) => Promise<Outcome<CompressionJob>>;
export interface Preferences { theme: ThemeMode; locale: Locale }
export interface DesktopServices {
  onOpenPath(listener: (path: string) => void): () => void;
  readPreferences(): Promise<Outcome<Preferences>>;
  writePreferences(preferences: Preferences): Promise<Outcome<void>>;
  setTrayVisible(visible: boolean): Promise<Outcome<void>>;
}
export interface WindowControls {
  minimize(): Promise<void>;
  toggleMaximize(): Promise<void>;
  isMaximized(): Promise<boolean>;
  isFullscreen(): Promise<boolean>;
  setFullscreen(fullscreen: boolean): Promise<void>;
  close(): Promise<void>;
}
export type IconName = AssetIconName;
export interface IconOptions { name: IconName; size: number; label?: Message }
export type Icon = (options: IconOptions) => SVGSVGElement;
