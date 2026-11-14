mod associations;
mod launch;
mod logging;
mod settings;
mod updates;
mod windows;

pub use settings::{CloseBehavior, Settings};
use crate::contracts::{ContractError, ErrorCode, Locale};
use serde::Serialize;
use std::{io, path::{Path, PathBuf}, sync::{Arc, Mutex}};
use tauri::{AppHandle, Emitter, Manager, Runtime, State, WebviewWindow, WindowEvent,
    menu::{Menu, MenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}};

const TRAY: &str = "picglance-desktop";
const RELEASES: &str = "https://github.com/Paosin/PicGlance/releases";
const OPEN_READY: &str = "desktop-open-ready";

pub struct Desktop {
    store: settings::SettingsStore,
    settings: Mutex<Settings>,
    pending: Mutex<Vec<PathBuf>>,
    native_open: Option<Box<dyn Fn(Vec<PathBuf>) + Send + Sync>>,
    log: Option<Arc<logging::Log>>,
}

impl Desktop {
    pub fn settings(&self) -> Settings {
        self.settings.lock().expect("desktop settings").clone()
    }

    pub fn record(&self, event: &str, message: &str) {
        if let Some(log) = &self.log { log.record(event, message); }
    }
}

pub fn handle_installer_command() -> Option<i32> {
    let mut args = std::env::args().skip(1);
    let action = args.next()?;
    if !matches!(action.as_str(), "--desktop-install" | "--desktop-uninstall") { return None; }
    let result = (|| {
        let exe = std::env::current_exe()?;
        if exe.with_file_name("portable.marker").is_file() {
            return Err(io::Error::other("portable association changes disabled"));
        }
        if action == "--desktop-uninstall" { associations::uninstall() }
        else {
            let selection = args.next().unwrap_or_default();
            let enabled = selection.split(',').filter(|ext| !ext.is_empty()).map(str::to_owned).collect::<Vec<_>>();
            associations::install(&exe, &enabled)
        }
    })();
    Some(if result.is_ok() { 0 } else { 1 })
}

pub fn configure_context<R: Runtime>(context: &mut tauri::Context<R>) -> io::Result<()> {
    let exe = std::env::current_exe()?;
    if exe.with_file_name("portable.marker").is_file() {
        // Keep both the WebView profile and preferences beside the portable executable.
        let profile = exe.with_file_name("webview");
        if let Some(window) = context.config_mut().app.windows.iter_mut().find(|window| window.label == "main") {
            window.create = false;
        }
        // Portable copies can run next to the installation and each other.
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hash = DefaultHasher::new();
        profile.to_string_lossy().to_lowercase().hash(&mut hash);
        context.config_mut().identifier.push_str(&format!(".portable.{:016x}", hash.finish()));
    }
    Ok(())
}

pub fn single_instance<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_single_instance::init(|app, args, cwd| {
        if let Some(state) = app.try_state::<Desktop>() {
            queue_paths(app, &state, &args, Path::new(&cwd));
        }
        restore(app);
    })
}

fn queue_paths<R: Runtime>(app: &AppHandle<R>, state: &Desktop, args: &[String], cwd: &Path) {
    let paths = launch::paths(args, cwd);
    if paths.is_empty() { return; }
    if let Some(open) = &state.native_open {
        open(paths);
        return;
    }
    state.pending.lock().expect("desktop open queue").extend(paths);
    let _ = app.emit_to("main", OPEN_READY, ());
}

pub fn setup<R: Runtime>(
    app: &AppHandle<R>,
    native_open: Option<Box<dyn Fn(Vec<PathBuf>) + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()?;
    let dir = exe.parent().ok_or_else(|| io::Error::other("missing executable directory"))?;
    let store = settings::SettingsStore::new(dir, app.path().config_dir()?.join("PicGlance"));
    if store.portable && app.get_webview_window("main").is_none() {
        let window = app.config().app.windows.iter().find(|window| window.label == "main")
            .ok_or_else(|| io::Error::other("missing main window configuration"))?;
        tauri::WebviewWindowBuilder::from_config(app, window)?
            .data_directory(dir.join("webview"))
            .build()?;
    }
    let loaded = store.load();
    let fallback = if store.portable { None } else { app.path().local_data_dir().ok().map(|p| p.join("PicGlance/log")) };
    let log = logging::Log::open(dir.join("log"), fallback, 1024 * 1024).ok().map(Arc::new);
    if let Some(log) = &log {
        log.capture_panics();
        if let Err(error) = &loaded { log.record("settings-read", &error.to_string()); }
    }
    let host_handles_close = native_open.is_some();
    app.manage(Desktop { settings: Mutex::new(loaded.unwrap_or_default()), store, log, pending: Mutex::new(Vec::new()), native_open });
    let state = app.state::<Desktop>();
    let window = app.get_webview_window("main").ok_or_else(|| io::Error::other("missing main window"))?;
    let initial = state.settings();
    let _ = window.set_theme(native_theme(initial.theme));
    setup_tray(app, initial.locale)?;
    queue_paths(app, &state, &std::env::args().collect::<Vec<_>>(), &std::env::current_dir()?);
    let handle = app.clone();
    let observed_window = window.clone();
    if !host_handles_close { window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            let state = handle.state::<Desktop>();
            let close = state.settings().close_behavior;
            if close == CloseBehavior::Quit { return; }
            api.prevent_close();
            let result = tray_visible(&handle, close == CloseBehavior::Tray).and_then(|()| observed_window.hide());
            if let Err(error) = result {
                state.record("window-close", &error.to_string());
            }
        }
    }); }
    Ok(())
}

pub fn close_from_host<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    let app = window.app_handle();
    let behavior = app.state::<Desktop>().settings().close_behavior;
    if behavior == CloseBehavior::Quit {
        window.destroy()
    } else {
        tray_visible(app, behavior == CloseBehavior::Tray)?;
        window.hide()
    }
}

fn native_theme(theme: crate::contracts::ThemeMode) -> Option<tauri::Theme> {
    match theme {
        crate::contracts::ThemeMode::Dark => Some(tauri::Theme::Dark),
        crate::contracts::ThemeMode::Light => Some(tauri::Theme::Light),
        crate::contracts::ThemeMode::System => None,
    }
}

fn restore<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        let _ = tray_visible(app, false);
    }
}

fn tray_menu<R: Runtime>(app: &AppHandle<R>, locale: Locale) -> tauri::Result<Menu<R>> {
    let (show, quit) = match locale { Locale::English => ("Show PicGlance", "Quit"), _ => ("显示 PicGlance", "退出") };
    Menu::with_items(app, &[
        &MenuItem::with_id(app, "desktop-show", show, true, None::<&str>)?,
        &MenuItem::with_id(app, "desktop-quit", quit, true, None::<&str>)?,
    ])
}

fn setup_tray<R: Runtime>(app: &AppHandle<R>, locale: Locale) -> tauri::Result<()> {
    let mut tray = TrayIconBuilder::with_id(TRAY)
        .tooltip("PicGlance")
        .menu(&tray_menu(app, locale)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "desktop-show" => restore(app),
            "desktop-quit" => app.exit(0),
            _ => (),
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(event, TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }) {
                restore(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() { tray = tray.icon(icon.clone()); }
    tray.build(app)?.set_visible(false)
}

fn tray_visible<R: Runtime>(app: &AppHandle<R>, visible: bool) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY) { tray.set_visible(visible)?; }
    Ok(())
}

fn error(code: ErrorCode) -> ContractError { ContractError { code } }

fn io_error(state: &Desktop, event: &str, detail: impl std::fmt::Display) -> ContractError {
    state.record(event, &detail.to_string());
    error(ErrorCode::Io)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    settings: Settings,
    portable: bool,
    version: String,
}

#[tauri::command]
pub fn desktop_read<R: Runtime>(app: AppHandle<R>, state: State<'_, Desktop>) -> Snapshot {
    Snapshot { settings: state.settings(), portable: state.store.portable, version: app.package_info().version.to_string() }
}

#[tauri::command]
pub fn desktop_write<R: Runtime>(window: WebviewWindow<R>, state: State<'_, Desktop>, settings: Settings) -> Result<(), ContractError> {
    if !settings.valid() { return Err(error(ErrorCode::InvalidInput)); }
    let mut current = state.settings.lock().expect("desktop settings");
    state.store.save(&settings).map_err(|err| io_error(&state, "settings-write", err))?;
    let locale_changed = current.locale != settings.locale;
    let theme_changed = current.theme != settings.theme;
    *current = settings;
    let locale = current.locale;
    let theme = current.theme;
    drop(current);
    if theme_changed {
        if let Err(err) = window.set_theme(native_theme(theme)) { state.record("window-theme", &err.to_string()); }
    }
    if locale_changed {
        let app = window.app_handle();
        if let Some(tray) = app.tray_by_id(TRAY) {
            if let Err(err) = tray_menu(app, locale).and_then(|menu| tray.set_menu(Some(menu))) {
                state.record("tray-menu", &err.to_string());
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn desktop_take_paths(state: State<'_, Desktop>) -> Vec<PathBuf> {
    std::mem::take(&mut *state.pending.lock().expect("desktop open queue"))
}

#[tauri::command]
pub fn desktop_set_tray<R: Runtime>(app: AppHandle<R>, visible: bool) -> Result<(), ContractError> {
    tray_visible(&app, visible).map_err(|_| error(ErrorCode::Io))
}

#[derive(Serialize)]
pub struct Associations {
    formats: &'static [&'static str],
    selected: Vec<String>,
}

#[tauri::command]
pub fn desktop_associations(state: State<'_, Desktop>) -> Result<Associations, ContractError> {
    let selected = if state.store.portable { Vec::new() } else {
        associations::selected().map_err(|err| io_error(&state, "association-read", err))?
    };
    Ok(Associations { formats: &associations::FORMATS, selected })
}

#[tauri::command]
pub fn desktop_set_associations(state: State<'_, Desktop>, selected: Vec<String>) -> Result<(), ContractError> {
    if state.store.portable { return Err(error(ErrorCode::Unsupported)); }
    if selected.iter().any(|ext| !associations::FORMATS.contains(&ext.as_str())) {
        return Err(error(ErrorCode::InvalidInput));
    }
    let exe = std::env::current_exe().map_err(|err| io_error(&state, "association-write", err))?;
    associations::configure(&exe, &selected).map_err(|err| io_error(&state, "association-write", err))
}

#[tauri::command]
pub fn desktop_default_apps(state: State<'_, Desktop>) -> Result<(), ContractError> {
    if state.store.portable { return Err(error(ErrorCode::Unsupported)); }
    windows::open("ms-settings:defaultapps").map_err(|err| io_error(&state, "default-apps", err))
}

#[derive(serde::Deserialize)]
struct Release { tag_name: String }

#[tauri::command]
pub async fn desktop_check_update<R: Runtime>(app: AppHandle<R>, startup: bool) -> Option<String> {
    if startup && !app.state::<Desktop>().settings().check_updates { return None; }
    let current = app.package_info().version.to_string();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<Desktop>();
        let result = windows::latest_release().and_then(|body|
            serde_json::from_slice::<Release>(&body).map_err(io::Error::other));
        match result {
            Ok(release) => {
                let latest = updates::newer_tag(&current, &release.tag_name);
                state.record("update-check", &format!("current={current} latest={} available={}", release.tag_name, latest.is_some()));
                latest
            }
            Err(err) => {
                state.record("update-check", &err.to_string());
                None
            }
        }
    }).await.ok().flatten()
}

#[tauri::command]
pub fn desktop_open_releases(state: State<'_, Desktop>) -> Result<(), ContractError> {
    windows::open(RELEASES).map_err(|err| io_error(&state, "open-releases", err))
}

#[tauri::command]
pub fn desktop_open_license(state: State<'_, Desktop>) -> Result<(), ContractError> {
    windows::open("https://github.com/Paosin/PicGlance/blob/legacy/v1/LICENSE")
        .map_err(|err| io_error(&state, "open-license", err))
}
