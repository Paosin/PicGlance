import './settings.css';
import type { Locale } from '../../shared/preferences.ts';
import type { DesktopSnapshot, DesktopSettings, WindowsDesktop } from './service.ts';

type Text = readonly [zh: string, en: string];
const groups = [
  ['通用', 'General'], ['外观', 'Appearance'], ['查看', 'Viewing'], ['编辑', 'Editing'],
  ['文件关联', 'File associations'], ['压缩', 'Compression'], ['关于', 'About'],
] as const;

function element<K extends keyof HTMLElementTagNameMap>(tag: K, text?: string) {
  const node = document.createElement(tag);
  if (text) node.textContent = text;
  return node;
}

const label = (locale: Locale, text: Text) => text[locale === 'en' ? 1 : 0];

export function settingsPage(
  service: WindowsDesktop,
  initial: DesktopSnapshot,
  changed: (snapshot: DesktopSnapshot) => void,
  update: (tag: string) => void,
  close: () => void,
) {
  let snapshot = initial;
  let active = 0;
  let revision = 0;
  const page = element('section');
  page.className = 'desktop-settings';
  const nav = element('nav');
  const content = element('div');
  const panel = element('div');
  panel.className = 'desktop-settings-content';
  panel.append(content);
  page.append(nav, panel);

  const render = () => {
    const rendered = ++revision;
    const t = (text: Text) => label(snapshot.settings.locale, text);
    page.setAttribute('aria-label', t(['设置', 'Settings']));
    const back = element('button', t(['返回看图', 'Back to viewer']));
    back.onclick = close;
    nav.replaceChildren(back, ...groups.map((group, index) => {
      const button = element('button', t(group));
      if (index === active) button.setAttribute('aria-current', 'page');
      button.onclick = () => {
        active = index;
        render();
        nav.querySelector<HTMLElement>('[aria-current]')?.focus();
      };
      return button;
    }));
    const status = element('p');
    status.setAttribute('role', 'status');
    const fail = () => { status.textContent = t(['无法完成操作。', 'The operation could not be completed.']); };
    content.replaceChildren(element('h2', t(groups[active])));
    const row = (name: Text, input: HTMLElement) => {
      const wrapper = element('label');
      wrapper.append(element('span', t(name)), input);
      content.append(wrapper);
    };
    const write = async <K extends keyof DesktopSettings>(key: K, value: DesktopSettings[K], input: HTMLInputElement | HTMLSelectElement) => {
      input.disabled = true;
      try {
        snapshot = await service.update({ [key]: value });
        changed(snapshot);
        if (key === 'locale') {
          render();
          content.querySelector<HTMLElement>('select')?.focus();
        }
      } catch {
        if (input instanceof HTMLInputElement && input.type === 'checkbox') {
          input.checked = Boolean(snapshot.settings[key]);
        } else input.value = String(snapshot.settings[key]);
        fail();
      } finally { input.disabled = false; }
    };
    const select = <K extends keyof DesktopSettings>(name: Text, key: K, options: readonly (readonly [DesktopSettings[K], Text])[]) => {
      const input = element('select');
      for (const [value, text] of options) {
        const option = element('option', t(text));
        option.value = String(value);
        input.append(option);
      }
      input.value = String(snapshot.settings[key]);
      input.onchange = () => void write(key, input.value as DesktopSettings[K], input);
      row(name, input);
    };
    const toggle = (name: Text, key: 'checkUpdates' | 'upscaleSmallImages') => {
      const input = element('input');
      input.type = 'checkbox';
      input.checked = snapshot.settings[key];
      input.onchange = () => void write(key, input.checked, input);
      row(name, input);
    };
    const number = (name: Text, key: 'brushThickness' | 'mosaicBlockSize' | 'compressTargetMb', min: number, max: number, step = 1) => {
      const input = element('input');
      input.type = 'number';
      input.min = String(min);
      input.max = String(max);
      input.step = String(step);
      input.value = String(snapshot.settings[key]);
      input.onchange = () => {
        if (input.checkValidity() && Number.isFinite(input.valueAsNumber)) void write(key, input.valueAsNumber, input);
        else { input.reportValidity(); input.value = String(snapshot.settings[key]); }
      };
      row(name, input);
    };
    const action = (text: Text, run: () => Promise<void>) => {
      const button = element('button', t(text));
      button.onclick = async () => {
        button.disabled = true;
        status.textContent = '';
        try { await run(); } catch { fail(); } finally { button.disabled = false; }
      };
      content.append(button);
    };
    switch (active) {
      case 0:
        select(['语言', 'Language'], 'locale', [['zh-CN', ['简体中文', '简体中文']], ['en', ['English', 'English']]]);
        toggle(['启动时检查更新', 'Check for updates on startup'], 'checkUpdates');
        select(['关闭窗口时', 'When closing the window'], 'closeBehavior', [
          ['quit', ['退出程序', 'Quit']], ['tray', ['最小化到系统托盘', 'Hide in system tray']],
          ['background', ['后台运行', 'Run in background']],
        ]);
        break;
      case 1:
        select(['主题', 'Theme'], 'theme', [['dark', ['深色', 'Dark']], ['light', ['浅色', 'Light']], ['system', ['跟随系统', 'System']]]);
        break;
      case 2:
        select(['默认缩放', 'Default zoom'], 'zoomMode', [['fit', ['适应窗口', 'Fit to window']], ['actual', ['实际大小', 'Actual size']]]);
        toggle(['放大小图', 'Upscale small images'], 'upscaleSmallImages');
        break;
      case 3: {
        const color = element('input');
        color.type = 'color';
        color.value = snapshot.settings.brushColor;
        color.onchange = () => void write('brushColor', color.value, color);
        row(['画笔颜色', 'Brush color'], color);
        number(['画笔粗细（像素）', 'Brush thickness (px)'], 'brushThickness', 1, 50);
        number(['马赛克块大小（像素）', 'Mosaic block size (px)'], 'mosaicBlockSize', 5, 100);
        break;
      }
      case 4:
        if (snapshot.portable) {
          content.append(element('p', t(['便携版不注册文件关联。', 'File associations are unavailable in portable mode.'])));
          break;
        }
        content.append(element('p', t(['在 Windows 默认应用中选择 PicGlance 作为默认处理程序。', 'Choose PicGlance as the default handler in Windows Default apps.'])));
        action(['打开 Windows 默认应用', 'Open Windows Default apps'], () => service.defaultApps());
        void service.associations().then(({ formats, selected }) => {
          if (rendered !== revision) return;
          const checks = formats.map(ext => {
            const input = element('input');
            input.type = 'checkbox';
            input.checked = selected.includes(ext);
            row(['.' + ext, '.' + ext], input);
            return { ext, input };
          });
          for (const { input } of checks) input.onchange = async () => {
            checks.forEach(({ input }) => { input.disabled = true; });
            try {
              await service.setAssociations(checks.filter(({ input }) => input.checked).map(({ ext }) => ext));
              status.textContent = '';
            } catch { input.checked = !input.checked; fail(); }
            finally { checks.forEach(({ input }) => { input.disabled = false; }); }
          };
          content.append(status);
        }).catch(fail);
        break;
      case 5:
        select(['默认输出格式', 'Default output format'], 'compressFormat', [
          ['JPEG', ['JPEG', 'JPEG']], ['PNG', ['PNG', 'PNG']], ['WebP', ['WebP', 'WebP']], ['GIF', ['GIF', 'GIF']],
        ]);
        number(['目标大小（MB）', 'Target size (MB)'], 'compressTargetMb', 0.01, 1000, 0.01);
        break;
      case 6:
        content.append(element('p', 'PicGlance ' + snapshot.version));
        action(['检查更新', 'Check for updates'], async () => {
          const tag = await service.checkUpdate(false);
          if (tag) update(tag);
          else status.textContent = t(['暂无可显示的更新。', 'No update to display.']);
        });
        action(['MIT 开源许可', 'MIT license'], () => service.openLicense());
        break;
    }
    content.append(status);
  };
  render();
  return page;
}

export function updateBanner(service: WindowsDesktop, locale: Locale, tag: string, dismiss: () => void) {
  const banner = element('aside');
  banner.className = 'desktop-update';
  banner.setAttribute('role', 'status');
  const open = element('button', label(locale, ['打开下载页面', 'Open download page']));
  const ignore = element('button', label(locale, ['忽略', 'Dismiss']));
  const status = element('span');
  open.onclick = () => { void service.openReleases().catch(() => {
    status.textContent = label(locale, ['无法打开下载页面。', 'Could not open the download page.']);
  }); };
  ignore.onclick = dismiss;
  banner.append(element('span', label(locale, ['发现新版本 ', 'New version available ']) + tag), open, ignore, status);
  return banner;
}
