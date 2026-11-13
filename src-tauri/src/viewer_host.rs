use std::{path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::viewer::Viewer;

/// The single-image shell opens the most recent request. The event is only a wakeup.
#[derive(Default)]
pub struct PendingViewerOpens(Mutex<Option<String>>);

#[tauri::command]
pub fn viewer_take_pending_open(pending: State<'_, PendingViewerOpens>) -> Option<String> {
    pending.0.lock().ok()?.take()
}

pub fn queue_paths(app: &AppHandle, paths: impl IntoIterator<Item = PathBuf>) {
    let viewer = app.state::<Viewer>();
    let Some(id) = paths.into_iter().find_map(|path| viewer.accept_path(path).ok()) else {
        return;
    };
    if let Ok(mut slot) = app.state::<PendingViewerOpens>().0.lock() {
        *slot = Some(id);
    }
    let _ = app.emit_to("main", "picglance:viewer-open-pending", ());
}

#[cfg(not(windows))]
pub fn queue_startup(app: &AppHandle) {
    let mut args = std::env::args_os().skip(1);
    let Some(first) = args.next() else { return };
    let Some(path) = (if first == "--" {
        args.next()
    } else if first.to_string_lossy().starts_with('-') {
        None
    } else {
        Some(first)
    }) else { return };
    if let Ok(cwd) = std::env::current_dir() {
        queue_paths(app, [cwd.join(path)]);
    }
}
