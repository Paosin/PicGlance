use super::{EditError, EditorSource, Operation};
use crate::contracts::{ContractError, EditedResources, EditorInput, ErrorCode, ImageResource};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tauri::{ipc::Response, AppHandle, State};
use tauri_plugin_dialog::DialogExt;

type Result<T> = std::result::Result<T, ContractError>;
type Session = Arc<Mutex<EditorSource>>;

#[derive(Default)]
pub struct EditorState(Mutex<Option<(String, Session)>>, std::sync::atomic::AtomicU64);

fn error(code: ErrorCode) -> ContractError { ContractError { code } }

impl From<EditError> for ContractError {
    fn from(value: EditError) -> Self {
        error(match value {
            EditError::Unsupported => ErrorCode::Unsupported,
            EditError::TooLarge => ErrorCode::TooLarge,
            EditError::InvalidInput => ErrorCode::InvalidInput,
            EditError::Io | EditError::Conflict => ErrorCode::Io,
        })
    }
}

impl EditorState {
    pub fn open(&self, input: EditorInput) -> Result<String> {
        let source = EditorSource::open(&input.document, input.original_bytes, input.source_path)?;
        let mut active = self.0.lock().map_err(|_| error(ErrorCode::Busy))?;
        let id = self.1.fetch_add(1, std::sync::atomic::Ordering::Relaxed).to_string();
        *active = Some((id.clone(), Arc::new(Mutex::new(source))));
        Ok(id)
    }

    fn get(&self, id: &str) -> Result<Session> {
        let active = self.0.lock().map_err(|_| error(ErrorCode::Busy))?;
        active.as_ref().filter(|(key, _)| key == id).map(|(_, value)| Arc::clone(value))
            .ok_or_else(|| error(ErrorCode::InvalidInput))
    }
}

async fn background<T: Send + 'static>(work: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(work).await.map_err(|_| error(ErrorCode::Io))?
}

#[tauri::command]
pub async fn editor_render(state: State<'_, EditorState>, id: String, operations: Vec<Operation>) -> Result<Response> {
    let session = state.get(&id)?;
    background(move || {
        let source = session.lock().map_err(|_| error(ErrorCode::Busy))?;
        Ok(Response::new(source.render_png(&operations)?))
    }).await
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SaveMode { As, Overwrite }

#[tauri::command]
pub async fn editor_save(
    app: AppHandle, state: State<'_, EditorState>, id: String, operations: Vec<Operation>, mode: SaveMode,
) -> Result<bool> {
    let session = state.get(&id)?;
    match mode {
        SaveMode::As => {
            let filename = session.lock().map_err(|_| error(ErrorCode::Busy))?.suggested_name();
            let target = background(move || {
                Ok(app.dialog().file().set_file_name(filename)
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff", "ico"])
                    .blocking_save_file())
            }).await?;
            let Some(target) = target else { return Ok(false) };
            let mut target = target.into_path().map_err(|_| error(ErrorCode::InvalidInput))?;
            if target.extension().is_none() { target.set_extension("png"); }
            // A session closed while the picker was open cannot publish a file.
            let current = state.get(&id)?;
            background(move || {
                current.lock().map_err(|_| error(ErrorCode::Busy))?.save_new(&operations, &target)?;
                Ok(true)
            }).await
        }
        SaveMode::Overwrite => background(move || {
            session.lock().map_err(|_| error(ErrorCode::Busy))?.overwrite(&operations)?;
            Ok(true)
        }).await,
    }
}

#[tauri::command]
pub async fn editor_export(
    state: State<'_, EditorState>, exports: State<'_, EditedResources>, id: String, operations: Vec<Operation>,
) -> Result<ImageResource> {
    let session = state.get(&id)?;
    let bytes = background(move || {
        let source = session.lock().map_err(|_| error(ErrorCode::Busy))?;
        Ok(source.export_png(&operations)?)
    }).await?;
    state.get(&id)?;
    exports.publish_png(bytes)
}

#[tauri::command]
pub fn editor_close(state: State<'_, EditorState>, id: String) -> Result<()> {
    let mut active = state.0.lock().map_err(|_| error(ErrorCode::Busy))?;
    if active.as_ref().is_some_and(|(key, _)| key == &id) { *active = None; }
    Ok(())
}
