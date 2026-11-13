use crate::contracts::{ContractError, EditedResources, ErrorCode, ImageResource};
use crate::viewer::Viewer;
use picglance_compression::{compress, Error, Format, Options, ResultImage};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc, Mutex},
};
use tauri::{ipc::{Channel, Response}, path::BaseDirectory, AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

type Result<T> = std::result::Result<T, ContractError>;
type Bytes = Arc<Vec<u8>>;

#[derive(Default)]
pub struct CompressionState {
    state: Mutex<JobState>,
    next: AtomicU64,
}

#[derive(Default)]
struct JobState {
    active: Option<Job>,
    result: Option<(ImageResource, Bytes)>,
}

struct Job {
    id: String,
    started: bool,
    cancel: Arc<AtomicBool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum OutputFormat { Jpeg, Png, Gif, Webp }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    target_bytes: usize,
    lock_dimensions: bool,
    format: OutputFormat,
    max_quality: u8,
    allow_frame_drop: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Completed {
    resource: ImageResource,
    size_bytes: usize,
    width: u32,
    height: u32,
    frame_count: usize,
    quality: u8,
    met_target: bool,
    original_color_count: Option<u32>,
    original_bit_depth: Option<u16>,
    output_color_count: Option<u32>,
    output_bit_depth: Option<u16>,
}

fn error(code: ErrorCode) -> ContractError { ContractError { code } }

impl CompressionState {
    fn reserve(&self) -> Result<String> {
        let mut state = self.state.lock().map_err(|_| error(ErrorCode::Busy))?;
        if state.active.is_some() { return Err(error(ErrorCode::Busy)); }
        let id = self.next.fetch_add(1, Ordering::Relaxed).to_string();
        state.active = Some(Job { id: id.clone(), started: false, cancel: Arc::new(AtomicBool::new(false)) });
        state.result = None;
        Ok(id)
    }

    fn begin(&self, id: &str) -> Result<Arc<AtomicBool>> {
        let mut state = self.state.lock().map_err(|_| error(ErrorCode::Busy))?;
        let job = state.active.as_mut().filter(|job| job.id == id && !job.started)
            .ok_or_else(|| error(ErrorCode::Cancelled))?;
        job.started = true;
        Ok(Arc::clone(&job.cancel))
    }

    fn cancel(&self, id: &str) -> Result<()> {
        let mut state = self.state.lock().map_err(|_| error(ErrorCode::Busy))?;
        if let Some(job) = state.active.as_ref().filter(|job| job.id == id) {
            job.cancel.store(true, Ordering::Relaxed);
            if !job.started { state.active = None; }
        }
        if state.result.as_ref().is_some_and(|(resource, _)| resource.id == format!("compression:{id}")) {
            state.result = None;
        }
        Ok(())
    }

    fn finish(&self, id: &str, result: Result<ResultImage>) -> Result<Completed> {
        let mut state = self.state.lock().map_err(|_| error(ErrorCode::Busy))?;
        if state.active.as_ref().is_none_or(|job| job.id != id) {
            return Err(error(ErrorCode::Cancelled));
        }
        let job = state.active.take().unwrap();
        if job.cancel.load(Ordering::Relaxed) { return Err(error(ErrorCode::Cancelled)); }
        let image = result?;
        let resource = ImageResource { id: format!("compression:{id}"), mime_type: image.format.mime_type().into() };
        let completed = Completed {
            resource: resource.clone(), size_bytes: image.bytes.len(), width: image.width, height: image.height,
            frame_count: image.frame_count, quality: image.quality, met_target: image.met_target,
            original_color_count: image.original_color_count, original_bit_depth: image.original_bit_depth,
            output_color_count: image.output_color_count, output_bit_depth: image.output_bit_depth,
        };
        state.result = Some((resource, Arc::new(image.bytes)));
        Ok(completed)
    }

    fn read(&self, id: &str) -> Result<(ImageResource, Bytes)> {
        self.state.lock().map_err(|_| error(ErrorCode::Busy))?.result.as_ref()
            .filter(|(resource, _)| resource.id == id).cloned().ok_or_else(|| error(ErrorCode::InvalidInput))
    }
}

#[tauri::command]
pub fn compression_reserve(state: State<'_, CompressionState>) -> Result<String> { state.reserve() }

#[tauri::command]
pub fn compression_cancel(state: State<'_, CompressionState>, job_id: String) -> Result<()> {
    state.cancel(&job_id)
}

#[tauri::command]
pub async fn compression_run(
    app: AppHandle, state: State<'_, CompressionState>, job_id: String, source_id: String,
    request: Request, progress: Channel<f64>,
) -> Result<Completed> {
    let cancel = state.begin(&job_id)?;
    let result = tauri::async_runtime::spawn_blocking(move || {
        let input = if source_id.starts_with("editor-export:") {
            app.state::<EditedResources>().read(&source_id)?
        } else {
            if !source_id.ends_with(":original") { return Err(error(ErrorCode::InvalidInput)); }
            app.state::<Viewer>().resource(&source_id).map_err(|_| error(ErrorCode::InvalidInput))?
        };
        let tools = app.path().resolve("tools", BaseDirectory::Resource).map_err(|_| error(ErrorCode::Io))?;
        #[cfg(debug_assertions)]
        let tools = if tools.is_dir() { tools } else { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tools") };
        let format = match request.format {
            OutputFormat::Jpeg => Format::Jpeg, OutputFormat::Png => Format::Png,
            OutputFormat::Gif => Format::Gif, OutputFormat::Webp => Format::Webp,
        };
        let options = Options {
            target_bytes: request.target_bytes, lock_dimensions: request.lock_dimensions,
            format: Some(format), max_quality: request.max_quality, allow_frame_drop: request.allow_frame_drop,
        };
        let result = compress(&input, options, &tools, &cancel, |value| {
            let attempt = value.attempt as f64;
            let _ = progress.send(attempt / (attempt + 5.0));
        });
        #[cfg(windows)]
        if let Some(desktop) = app.try_state::<crate::desktop::Desktop>() {
            let detail = match &result {
                Ok(output) => format!("target={} actual={} met={}", options.target_bytes, output.bytes.len(), output.met_target),
                Err(error) => format!("{error:?}"),
            };
            desktop.record("compression", &detail);
        }
        if result.is_ok() { let _ = progress.send(1.0); }
        result.map_err(|failure| error(match failure {
            Error::Cancelled => ErrorCode::Cancelled, Error::Unsupported => ErrorCode::Unsupported,
            Error::InvalidInput => ErrorCode::InvalidInput, Error::TooLarge => ErrorCode::TooLarge,
            _ => ErrorCode::Io,
        }))
    }).await.unwrap_or_else(|_| Err(error(ErrorCode::Io)));
    state.finish(&job_id, result)
}

#[tauri::command]
pub async fn compression_read(app: AppHandle, id: String) -> Result<Response> {
    let bytes = if id.starts_with("editor-export:") {
        app.state::<EditedResources>().read(&id)?
    } else if id.starts_with("compression:") {
        app.state::<CompressionState>().read(&id)?.1
    } else {
        app.state::<Viewer>().resource(&id).map_err(|_| error(ErrorCode::InvalidInput))?
    };
    tauri::async_runtime::spawn_blocking(move || Response::new(bytes.as_ref().clone()))
        .await.map_err(|_| error(ErrorCode::Io))
}

fn save_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    let parent = path.parent().ok_or(io::ErrorKind::InvalidInput)?;
    let (mut file, temporary) = loop {
        let name = format!(".picglance-compression-{}-{}.tmp", std::process::id(), NEXT_TEMP.fetch_add(1, Ordering::Relaxed));
        let temporary = parent.join(name);
        match OpenOptions::new().write(true).create_new(true).open(&temporary) {
            Ok(file) => break (file, temporary),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    let result = written.and_then(|()| fs::hard_link(&temporary, path));
    let _ = fs::remove_file(temporary);
    result
}

#[tauri::command]
pub async fn compression_save(app: AppHandle, id: String, suggested_name: String) -> Result<()> {
    let (resource, bytes) = app.state::<CompressionState>().read(&id)?;
    let extension = match resource.mime_type.as_str() {
        "image/jpeg" => "jpg", "image/png" => "png", "image/gif" => "gif", "image/webp" => "webp",
        _ => return Err(error(ErrorCode::Unsupported)),
    };
    let suggested = Path::new(&suggested_name).file_stem().and_then(|stem| stem.to_str()).unwrap_or("PicGlance");
    let suggested = format!("{suggested}-compressed.{extension}");
    tauri::async_runtime::spawn_blocking(move || {
        let selected = app.dialog().file().set_file_name(suggested).add_filter(extension, &[extension])
            .blocking_save_file().ok_or_else(|| error(ErrorCode::Cancelled))?;
        let mut path: PathBuf = selected.into_path().map_err(|_| error(ErrorCode::InvalidInput))?;
        if path.extension().is_none() { path.set_extension(extension); }
        if !path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
            ext.eq_ignore_ascii_case(extension) || (extension == "jpg" && ext.eq_ignore_ascii_case("jpeg"))
        }) { return Err(error(ErrorCode::InvalidInput)); }
        save_new(&path, &bytes).map_err(|_| error(ErrorCode::Io))
    }).await.map_err(|_| error(ErrorCode::Io))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_prevents_result_publication_and_releases_the_slot() {
        let state = CompressionState::default();
        let id = state.reserve().unwrap();
        let cancel = state.begin(&id).unwrap();
        assert!(state.reserve().is_err());
        state.cancel(&id).unwrap();
        assert!(cancel.load(Ordering::Relaxed));
        let output = ResultImage { bytes: vec![1], format: Format::Png, width: 1, height: 1,
            frame_count: 1, quality: 95, source_bytes: 1, met_target: true, original_color_count: None,
            original_bit_depth: None, output_color_count: None, output_bit_depth: None };
        assert_eq!(state.finish(&id, Ok(output)).err().unwrap().code, ErrorCode::Cancelled);
        assert!(state.read(&format!("compression:{id}")).is_err());
        assert!(state.reserve().is_ok());
    }
}
