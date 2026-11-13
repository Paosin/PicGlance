import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { DesktopServices, Outcome, Preferences } from '../../shared/contracts.ts';

export interface DesktopSettings extends Preferences {
  checkUpdates: boolean;
  closeBehavior: 'quit' | 'tray' | 'background';
  zoomMode: 'fit' | 'actual';
  upscaleSmallImages: boolean;
  brushColor: string;
  brushThickness: number;
  mosaicBlockSize: number;
  compressFormat: 'JPEG' | 'PNG' | 'WebP' | 'GIF';
  compressTargetMb: number;
}
export interface DesktopSnapshot {
  settings: DesktopSettings;
  portable: boolean;
  version: string;
}
export interface Associations { formats: string[]; selected: string[] }

export class WindowsDesktop implements DesktopServices {
  private listeners = new Set<(path: string) => void>();
  private subscription?: Promise<() => void>;
  private draining = false;
  private requested = false;
  private writes = Promise.resolve();
  private retained: string[] = [];

  read() { return invoke<DesktopSnapshot>('desktop_read'); }
  update(change: Partial<DesktopSettings>) {
    const write = this.writes.then(async () => {
      const snapshot = await this.read();
      const settings = { ...snapshot.settings, ...change };
      await invoke('desktop_write', { settings });
      return { ...snapshot, settings };
    });
    this.writes = write.then(() => undefined, () => undefined);
    return write;
  }
  async readPreferences(): Promise<Outcome<Preferences>> {
    try {
      const { settings: { theme, locale } } = await this.read();
      return { ok: true, value: { theme, locale } };
    } catch { return { ok: false, error: { code: 'io' } }; }
  }
  async writePreferences(preferences: Preferences): Promise<Outcome<void>> {
    try {
      await this.update(preferences);
      return { ok: true, value: undefined };
    } catch { return { ok: false, error: { code: 'io' } }; }
  }
  async setTrayVisible(visible: boolean): Promise<Outcome<void>> {
    try {
      await invoke('desktop_set_tray', { visible });
      return { ok: true, value: undefined };
    } catch { return { ok: false, error: { code: 'io' } }; }
  }

  onOpenPath(listener: (path: string) => void) {
    this.listeners.add(listener);
    if (!this.subscription) {
      this.subscription = getCurrentWebviewWindow().listen('desktop-open-ready', () => { void this.drain(); });
      void this.subscription.then(() => this.drain()).catch(console.error);
    } else void this.drain();
    return () => {
      this.listeners.delete(listener);
      if (!this.listeners.size && this.subscription) {
        void this.subscription.then(unlisten => unlisten()).catch(console.error);
        this.subscription = undefined;
      }
    };
  }

  private async drain() {
    this.requested = true;
    if (this.draining) return;
    this.draining = true;
    try {
      while (this.requested && this.listeners.size) {
        this.requested = false;
        this.retained.push(...await invoke<string[]>('desktop_take_paths'));
        while (this.listeners.size && this.retained.length) {
          const path = this.retained.shift()!;
          for (const listener of this.listeners) {
            try { listener(path); } catch (error) { console.error(error); }
          }
        }
      }
    } catch (error) { console.error(error); }
    finally { this.draining = false; }
  }

  associations() { return invoke<Associations>('desktop_associations'); }
  setAssociations(selected: string[]) { return invoke<void>('desktop_set_associations', { selected }); }
  defaultApps() { return invoke<void>('desktop_default_apps'); }
  checkUpdate(startup: boolean) { return invoke<string | null>('desktop_check_update', { startup }); }
  openReleases() { return invoke<void>('desktop_open_releases'); }
  openLicense() { return invoke<void>('desktop_open_license'); }
}
