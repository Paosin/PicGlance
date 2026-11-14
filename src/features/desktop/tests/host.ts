import { invoke } from '@tauri-apps/api/core';
import { WindowsDesktop, type DesktopSnapshot } from '../service.ts';
import { settingsPage, updateBanner } from '../settings.ts';
import { resolveTheme } from '../../../shared/preferences.ts';
import '../../../shared/shell.css';

const desktop = new WindowsDesktop();
const opened = document.querySelector<HTMLOutputElement>('#opened')!;
const image = document.querySelector<HTMLImageElement>('#fixture')!;
const banner = document.querySelector<HTMLElement>('#banner')!;
const system = matchMedia('(prefers-color-scheme: dark)');
let snapshot = await desktop.read();
let update: string | null = null;
let resource: string | undefined;
let generation = 0;

const apply = () => {
  document.documentElement.lang = snapshot.settings.locale;
  document.documentElement.dataset.theme = resolveTheme(snapshot.settings.theme, system.matches);
  banner.replaceChildren(...(update ? [updateBanner(desktop, snapshot.settings.locale, update, () => {
    update = null;
    apply();
  })] : []));
};
const showUpdate = (tag: string) => { update = tag; apply(); };
const settings = document.querySelector<HTMLElement>('#settings')!;
settings.append(settingsPage(desktop, snapshot, (next: DesktopSnapshot) => {
  snapshot = next;
  apply();
}, showUpdate, () => { settings.hidden = !settings.hidden; }));
system.addEventListener('change', apply);
apply();
desktop.onOpenPath(path => {
  const requested = ++generation;
  void invoke<ArrayBuffer>('fixture_image', { path }).then(data => {
    if (requested !== generation) return;
    if (resource) URL.revokeObjectURL(resource);
    resource = URL.createObjectURL(new Blob([data]));
    image.src = resource;
    image.onload = () => {
      opened.value = path;
      document.title = 'PicGlance Desktop Test — ' + path.split(/[\\/]/).at(-1);
    };
  }).catch(console.error);
});
void desktop.checkUpdate(true).then(tag => { if (tag) showUpdate(tag); });
