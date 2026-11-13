mod decode;
#[cfg(test)]
mod tests;

use crate::contracts::{EditorInput, ImageDocument, ImageResource};
use decode::{decode, encode_png, Format};
use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{ipc::Response, AppHandle, State};
use tauri_plugin_dialog::DialogExt;

const MAX_FILE_BYTES: u64 = 80 * 1024 * 1024;
const MAX_DIRECTORY: usize = 50_000;
const EXTENSIONS: &[&str] = &["jpg", "jpeg", "jpe", "png", "apng", "bmp", "gif", "webp", "svg", "tif", "tiff", "ico"];

type Result<T, E = ViewerError> = std::result::Result<T, E>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerError {
    code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_bytes: Option<u64>,
}

impl ViewerError {
    fn new(code: &'static str) -> Self { Self { code, size_bytes: None } }

    fn too_large(size: u64) -> Self { Self { code: "too-large", size_bytes: Some(size) } }
}

impl From<io::Error> for ViewerError {
    fn from(error: io::Error) -> Self {
        Self::new(if error.kind() == io::ErrorKind::NotFound { "missing" } else { "io" })
    }
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
pub struct DecodeSize {
    width: u32,
    height: u32,
}

impl DecodeSize {
    const THUMBNAIL: Self = Self { width: 128, height: 128 };
    const ORIGINAL: Self = Self { width: u32::MAX, height: u32::MAX };

    fn bounded(self) -> Self {
        Self { width: self.width.clamp(129, 3840), height: self.height.clamp(129, 3840) }
    }

    fn is_thumbnail(self) -> bool { self.width == 128 && self.height == 128 }
}

#[derive(Serialize)]
pub struct DirectoryEntry { id: String, name: String }

#[derive(Serialize)]
struct Metadata { tag: String, value: String }

#[derive(Serialize)]
pub struct ImageDetails {
    document: ImageDocument,
    original: ImageResource,
    metadata: Vec<Metadata>,
    gps: bool,
}

type Resource = Arc<Vec<u8>>;

struct DocumentResources {
    document: ImageDocument,
    source: Option<PathBuf>,
    preview: Resource,
    original: Resource,
}

struct Prepared {
    original: Resource,
    preview: decode::Preview,
    mime: &'static str,
    metadata: Vec<Metadata>,
    gps: bool,
}

const MAX_PREPARED_BYTES: usize = 32 * 1024 * 1024;

#[derive(Default)]
struct Grants {
    external: VecDeque<(String, PathBuf)>,
    directory: HashMap<String, PathBuf>,
}

#[derive(Default)]
struct Requests {
    active: HashMap<String, Arc<AtomicBool>>,
    cancelled: VecDeque<String>,
}

#[derive(Clone, Default)]
pub struct Viewer {
    grants: Arc<Mutex<Grants>>,
    resources: Arc<Mutex<VecDeque<DocumentResources>>>,
    requests: Arc<Mutex<Requests>>,
    decoder: Arc<Mutex<()>>,
    directory_operation: Arc<Mutex<()>>,
    prepared: Arc<Mutex<VecDeque<(PathBuf, DecodeSize, Prepared)>>>,
    next: Arc<AtomicU64>,
}

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
    mutex.lock().map_err(|_| ViewerError::new("busy"))
}

impl Viewer {
    fn id(&self) -> String { self.next.fetch_add(1, Ordering::Relaxed).to_string() }

    /// Called by the native picker, OS open handler and native drag/drop handler.
    pub fn accept_path(&self, path: PathBuf) -> Result<String> {
        if !path.is_absolute() { return Err(ViewerError::new("invalid-input")); }
        let mut grants = lock(&self.grants)?;
        let id = self.id();
        if grants.external.len() == 32 { grants.external.pop_front(); }
        grants.external.push_back((id.clone(), path));
        Ok(id)
    }

    fn path(&self, id: &str) -> Result<PathBuf> {
        let grants = lock(&self.grants)?;
        grants.directory.get(id).or_else(|| grants.external.iter().find(|(key, _)| key == id).map(|(_, path)| path))
            .cloned().ok_or_else(|| ViewerError::new("invalid-input"))
    }

    fn directory(&self, id: &str, cancel: &AtomicBool) -> Result<Vec<DirectoryEntry>> {
        let _operation = lock(&self.directory_operation)?;
        check(cancel)?;
        let source = self.path(id)?;
        let parent = source.parent().ok_or_else(|| ViewerError::new("io"))?;
        let mut paths = Vec::new();
        for entry in fs::read_dir(parent)? {
            check(cancel)?;
            let entry = entry?;
            let path = entry.path();
            let eligible = path == source || path.extension().and_then(|ext| ext.to_str())
                .is_some_and(|ext| EXTENSIONS.iter().any(|known| ext.eq_ignore_ascii_case(known)));
            if eligible && entry.file_type()?.is_file() {
                let name = entry.file_name().to_string_lossy().into_owned();
                paths.push((name.to_lowercase(), name, path));
                if paths.len() > MAX_DIRECTORY { return Err(ViewerError::new("busy")); }
            }
        }
        paths.sort_unstable();
        let mut grants = lock(&self.grants)?;
        check(cancel)?;
        // Keep opaque IDs stable when reopening entries in the same directory.
        let mut previous: HashMap<_, _> = grants.directory.drain().map(|(key, path)| (path, key)).collect();
        let entries = paths.into_iter().map(|(_, name, path)| {
            let key = if path == source { id.to_owned() } else { previous.remove(&path).unwrap_or_else(|| self.id()) };
            grants.directory.insert(key.clone(), path);
            DirectoryEntry { id: key, name }
        }).collect();
        Ok(entries)
    }

    pub(crate) fn resource(&self, id: &str) -> Result<Resource> {
        let (id, kind) = id.split_once(':').ok_or_else(|| ViewerError::new("invalid-input"))?;
        let resources = lock(&self.resources)?;
        let resource = resources.iter().find(|entry| entry.document.id == id).ok_or_else(|| ViewerError::new("missing"))?;
        match kind {
            "preview" => Ok(resource.preview.clone()),
            "original" => Ok(resource.original.clone()),
            _ => Err(ViewerError::new("invalid-input")),
        }
    }

    pub fn editor_input(&self, document_id: &str) -> Result<EditorInput> {
        let (document, source_path, original) = {
            let resources = lock(&self.resources)?;
            let entry = resources.iter().find(|entry| entry.document.id == document_id)
                .ok_or_else(|| ViewerError::new("missing"))?;
            (entry.document.clone(), entry.source.clone(), Arc::clone(&entry.original))
        };
        Ok(EditorInput { document, source_path, original_bytes: original.as_ref().clone() })
    }

    #[cfg(windows)]
    fn rename(&self, document_id: &str, name: &str) -> Result<ImageDocument> {
        let _decode = lock(&self.decoder)?;
        let _operation = lock(&self.directory_operation)?;
        let mut resources = lock(&self.resources)?;
        let entry = resources.iter_mut().find(|entry| entry.document.id == document_id)
            .ok_or_else(|| ViewerError::new("missing"))?;
        let old = entry.source.as_ref().ok_or_else(|| ViewerError::new("invalid-name"))?;
        let name = rename_filename(old, name)?;
        let new = old.with_file_name(&name);
        if old.file_name() == new.file_name() {
            return Ok(entry.document.clone());
        }
        let mut grants = lock(&self.grants)?;
        if read_file(old)?.as_slice() != entry.original.as_slice() {
            return Err(ViewerError::new("source-changed"));
        }
        move_without_replacement(old, &new)?;
        let Grants { external, directory } = &mut *grants;
        for path in external.iter_mut().map(|(_, path)| path).chain(directory.values_mut()) {
            if path == old { path.clone_from(&new); }
        }
        entry.document.name = name;
        entry.document.source_path = Some(new.to_string_lossy().into_owned());
        entry.source = Some(new);
        Ok(entry.document.clone())
    }

    fn publish(&self, source: Option<PathBuf>, bytes: Vec<u8>, size: DecodeSize, cancel: &AtomicBool) -> Result<ImageDetails> {
        self.publish_prepared(source, Self::prepare(bytes, size, cancel)?, cancel)
    }

    fn prepare(bytes: Vec<u8>, size: DecodeSize, cancel: &AtomicBool) -> Result<Prepared> {
        let preview = decode(&bytes, size, cancel)?;
        let mime = match decode::format(&bytes)? {
            Format::Raster(_, mime) => mime,
            Format::Svg => "image/svg+xml",
        };
        let (metadata, gps) = metadata(&bytes);
        Ok(Prepared { original: Arc::new(bytes), preview, mime, metadata, gps })
    }

    fn publish_prepared(&self, source: Option<PathBuf>, prepared: Prepared, cancel: &AtomicBool) -> Result<ImageDetails> {
        let Prepared { original, preview, mime, metadata, gps } = prepared;
        let size_bytes = original.len() as u64;
        let name = source.as_ref().and_then(|path| path.file_name()).map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let source_path = source.as_ref().map(|path| path.to_string_lossy().into_owned());
        let preview_resource = preview.bytes.map(Arc::new).unwrap_or_else(|| original.clone());
        let mut resources = lock(&self.resources)?;
        check(cancel)?;
        let id = self.id();
        if resources.len() == 2 { resources.pop_front(); }
        let document = ImageDocument {
            id: id.clone(), source_path, name, width: preview.width, height: preview.height,
            frame_count: preview.frames, size_bytes,
            preview: ImageResource { id: format!("{id}:preview"), mime_type: preview.mime.into() },
        };
        resources.push_back(DocumentResources {
            document: document.clone(), source, preview: preview_resource, original,
        });
        Ok(ImageDetails {
            document,
            original: ImageResource { id: format!("{id}:original"), mime_type: mime.into() },
            metadata, gps,
        })
    }

    fn open_path(&self, path: PathBuf, size: DecodeSize, warm: bool, cancel: &AtomicBool) -> Result<Option<ImageDetails>> {
        let bytes = read_file(&path)?;
        let prepared = {
            let mut cache = lock(&self.prepared)?;
            cache.iter().position(|(key, dimensions, _)| key == &path && *dimensions == size)
                .and_then(|index| cache.remove(index)).map(|(_, _, image)| image)
        };
        let prepared = match prepared {
            Some(image) if image.original.as_slice() == bytes => image,
            _ => Self::prepare(bytes, size, cancel)?,
        };
        check(cancel)?;
        if warm {
            let byte_count = |image: &Prepared| image.original.len() + image.preview.bytes.as_ref().map_or(0, Vec::len);
            let incoming = byte_count(&prepared);
            if incoming <= MAX_PREPARED_BYTES {
                let mut cache = lock(&self.prepared)?;
                while cache.len() >= 2 || cache.iter().map(|(_, _, image)| byte_count(image)).sum::<usize>() + incoming > MAX_PREPARED_BYTES {
                    cache.pop_front();
                }
                cache.push_back((path, size, prepared));
            }
            Ok(None)
        } else {
            self.publish_prepared(Some(path), prepared, cancel).map(Some)
        }
    }

    async fn background<T: Send + 'static>(
        &self, request: String, action: impl FnOnce(&Self, &AtomicBool) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let viewer = self.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        {
            let mut requests = lock(&viewer.requests)?;
            if let Some(index) = requests.cancelled.iter().position(|id| id == &request) {
                requests.cancelled.remove(index);
                return Err(ViewerError::new("cancelled"));
            }
            if requests.active.len() >= 16 || requests.active.contains_key(&request) {
                return Err(ViewerError::new("busy"));
            }
            requests.active.insert(request.clone(), cancel.clone());
        }
        let worker = viewer.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            check(&cancel)?;
            let result = action(&worker, &cancel)?;
            check(&cancel)?;
            Ok(result)
        }).await;
        lock(&viewer.requests)?.active.remove(&request);
        result.map_err(|_| ViewerError::new("io"))?
    }
}

#[cfg(windows)]
fn rename_filename(old: &Path, input: &str) -> Result<String> {
    let reserved = input.split('.').next().unwrap_or_default().trim_end().to_uppercase();
    let device = matches!(reserved.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
        || ["COM", "LPT"].iter().any(|prefix| {
            reserved.strip_prefix(prefix).is_some_and(|suffix| {
                matches!(suffix, "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³")
            })
        });
    if input.trim().is_empty() || input.ends_with(['.', ' ']) || device
        || input.chars().any(|ch| ch.is_control() || "<>:\"/\\|?*".contains(ch))
    {
        return Err(ViewerError::new("invalid-name"));
    }
    let mut name = input.to_owned();
    if Path::new(input).extension().is_none() {
        if let Some(extension) = old.extension() {
            name.push('.');
            name.push_str(&extension.to_string_lossy());
        }
    } else if Path::new(input).extension() != old.extension()
        && !Path::new(input).extension().and_then(|ext| ext.to_str())
            .is_some_and(|ext| EXTENSIONS.iter().any(|known| ext.eq_ignore_ascii_case(known)))
    {
        return Err(ViewerError::new("invalid-name"));
    }
    if name.encode_utf16().count() > 255 { return Err(ViewerError::new("invalid-name")); }
    Ok(name)
}

#[cfg(windows)]
fn move_without_replacement(old: &Path, new: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::MoveFileExW;

    let parent = old.parent().ok_or_else(|| ViewerError::new("invalid-name"))?.canonicalize()?;
    let wide = |path: &Path| -> Result<Vec<u16>> {
        let filename = path.file_name().ok_or_else(|| ViewerError::new("invalid-name"))?;
        Ok(parent.join(filename).as_os_str().encode_wide().chain([0]).collect())
    };
    let from = wide(old)?;
    let to = wide(new)?;
    // No replace-existing flag. Windows resolves the collision during the move.
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 0) } != 0 { return Ok(()); }
    let error = io::Error::last_os_error();
    if error.kind() == io::ErrorKind::AlreadyExists {
        Err(ViewerError::new("name-exists"))
    } else {
        Err(error.into())
    }
}

fn check(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) { Err(ViewerError::new("cancelled")) } else { Ok(()) }
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    let file = File::open(path)?;
    let info = file.metadata()?;
    if !info.is_file() { return Err(ViewerError::new("invalid-input")); }
    if info.len() > MAX_FILE_BYTES { return Err(ViewerError::too_large(info.len())); }
    let mut bytes = Vec::with_capacity(info.len() as usize);
    file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE_BYTES { return Err(ViewerError::too_large(bytes.len() as u64)); }
    Ok(bytes)
}

fn metadata(bytes: &[u8]) -> (Vec<Metadata>, bool) {
    use exif::{In, Tag};
    let Ok(data) = exif::Reader::new().read_from_container(&mut Cursor::new(bytes)) else { return (Vec::new(), false) };
    let tags = [Tag::Make, Tag::Model, Tag::LensModel, Tag::FNumber, Tag::ExposureTime,
        Tag::PhotographicSensitivity, Tag::ISOSpeed, Tag::FocalLength, Tag::DateTimeOriginal, Tag::DateTime,
        Tag::GPSLatitude, Tag::GPSLongitude, Tag::GPSAltitude];
    let gps = [Tag::GPSLatitude, Tag::GPSLongitude, Tag::GPSAltitude].iter().any(|tag| {
        data.get_field(*tag, In::PRIMARY).is_some_and(|field| {
            matches!(&field.value, exif::Value::Rational(values) if !values.is_empty() && values.iter().all(|value| value.denom != 0))
        })
    });
    let fields = tags.iter().filter_map(|tag| data.get_field(*tag, In::PRIMARY)).map(|field| Metadata {
        tag: field.tag.to_string(),
        value: field.display_value().with_unit(&data).to_string().chars().take(512).collect(),
    }).collect();
    (fields, gps)
}

#[tauri::command]
pub async fn viewer_choose(app: AppHandle, viewer: State<'_, Viewer>) -> Result<Option<String>> {
    let viewer = viewer.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog().file().add_filter("Images", EXTENSIONS).blocking_pick_file()
            .map(|file| viewer.accept_path(file.into_path().map_err(|_| ViewerError::new("invalid-input"))?)).transpose()
    }).await.map_err(|_| ViewerError::new("io"))?
}

#[tauri::command]
pub async fn viewer_open(app: AppHandle, viewer: State<'_, Viewer>, id: String, size: DecodeSize, request: String, warm: Option<bool>) -> Result<Option<ImageDetails>> {
    #[cfg(not(windows))]
    let _ = app;
    viewer.background(request, move |viewer, cancel| {
        let _guard = lock(&viewer.decoder)?;
        check(cancel)?;
        let path = viewer.path(&id)?;
        let warm = warm.unwrap_or(false);
        let result = viewer.open_path(path.clone(), size.bounded(), warm, cancel);
        #[cfg(windows)]
        if let Err(error) = &result {
            use tauri::Manager;
            if !warm && error.code != "cancelled" {
                if let Some(desktop) = app.try_state::<crate::desktop::Desktop>() {
                    let mut detail = format!("path={} reason={}", path.display(), error.code);
                    if let Some(size) = error.size_bytes {
                        detail.push_str(&format!(" size_bytes={size} limit_bytes={MAX_FILE_BYTES}"));
                    }
                    desktop.record("viewer-open-failed", &detail);
                }
            }
        }
        result
    }).await
}

#[tauri::command]
pub async fn viewer_directory(viewer: State<'_, Viewer>, id: String, request: String) -> Result<Vec<DirectoryEntry>> {
    viewer.background(request, move |viewer, cancel| viewer.directory(&id, cancel)).await
}

#[tauri::command]
pub async fn viewer_rename(viewer: State<'_, Viewer>, id: String, name: String) -> Result<ImageDocument> {
    #[cfg(windows)]
    {
        let viewer = viewer.inner().clone();
        tauri::async_runtime::spawn_blocking(move || viewer.rename(&id, &name))
            .await.map_err(|_| ViewerError::new("io"))?
    }
    #[cfg(not(windows))]
    {
        let _ = (viewer, id, name);
        Err(ViewerError::new("unsupported"))
    }
}

#[tauri::command]
pub async fn viewer_thumbnail(viewer: State<'_, Viewer>, id: String, request: String) -> Result<Response> {
    viewer.background(request, move |viewer, cancel| {
        let _guard = lock(&viewer.decoder)?;
        check(cancel)?;
        let bytes = read_file(&viewer.path(&id)?)?;
        let preview = decode(&bytes, DecodeSize::THUMBNAIL, cancel)?;
        Ok(Response::new(preview.bytes.ok_or_else(|| ViewerError::new("invalid-input"))?))
    }).await
}

#[tauri::command]
pub fn viewer_cancel(viewer: State<'_, Viewer>, request: String) -> Result<()> {
    let mut requests = lock(&viewer.requests)?;
    if let Some(cancel) = requests.active.get(&request) {
        cancel.store(true, Ordering::Relaxed);
    } else if request.len() <= 24 {
        if requests.cancelled.len() == 32 { requests.cancelled.pop_front(); }
        requests.cancelled.push_back(request);
    }
    Ok(())
}

#[tauri::command]
pub async fn viewer_resource(viewer: State<'_, Viewer>, id: String, request: Option<String>) -> Result<Response> {
    if let Some(document) = id.strip_suffix(":detail") {
        let original = viewer.resource(&format!("{document}:original"))?;
        let request = request.ok_or_else(|| ViewerError::new("invalid-input"))?;
        return viewer.background(request, move |viewer, cancel| {
            let _guard = lock(&viewer.decoder)?;
            check(cancel)?;
            Ok(Response::new(encode_png(&decode::pixels(&original)?)?))
        }).await;
    }
    let resource = viewer.resource(&id)?;
    tauri::async_runtime::spawn_blocking(move || Ok(Response::new(resource.as_ref().clone())))
        .await.map_err(|_| ViewerError::new("io"))?
}

#[tauri::command]
pub async fn viewer_copy(viewer: State<'_, Viewer>, id: String) -> Result<()> {
    let viewer = viewer.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = lock(&viewer.decoder)?;
        let bytes = viewer.resource(&format!("{id}:original"))?;
        write_clipboard(decode::pixels(&bytes)?)
    }).await.map_err(|_| ViewerError::new("io"))?
}

#[tauri::command]
pub async fn viewer_paste(viewer: State<'_, Viewer>, size: DecodeSize, request: String) -> Result<ImageDetails> {
    viewer.background(request, move |viewer, cancel| {
        let _guard = lock(&viewer.decoder)?;
        check(cancel)?;
        let image = read_clipboard()?;
        let bytes = encode_png(&image)?;
        if bytes.len() as u64 > MAX_FILE_BYTES { return Err(ViewerError::too_large(bytes.len() as u64)); }
        viewer.publish(None, bytes, size.bounded(), cancel)
    }).await
}

#[cfg(windows)]
fn write_clipboard(image: DynamicImage) -> Result<()> {
    let (width, height) = image.dimensions();
    let mut clipboard = arboard::Clipboard::new().map_err(clipboard_error)?;
    clipboard.set_image(arboard::ImageData {
        width: width as usize, height: height as usize, bytes: image.into_rgba8().into_raw().into(),
    }).map_err(clipboard_error)
}

#[cfg(windows)]
fn read_clipboard() -> Result<DynamicImage> {
    let mut clipboard = arboard::Clipboard::new().map_err(clipboard_error)?;
    let data = clipboard.get_image().map_err(clipboard_error)?;
    let width = u32::try_from(data.width).map_err(|_| ViewerError::new("dimensions"))?;
    let height = u32::try_from(data.height).map_err(|_| ViewerError::new("dimensions"))?;
    decode::dimensions(width, height)?;
    let image = image::RgbaImage::from_raw(width, height, data.bytes.into_owned()).ok_or_else(|| ViewerError::new("invalid-input"))?;
    Ok(DynamicImage::ImageRgba8(image))
}

#[cfg(windows)]
fn clipboard_error(error: arboard::Error) -> ViewerError {
    ViewerError::new(if matches!(error, arboard::Error::ContentNotAvailable) { "clipboard-empty" } else { "io" })
}

#[cfg(not(windows))]
fn write_clipboard(_: DynamicImage) -> Result<()> { Err(ViewerError::new("unsupported")) }

#[cfg(not(windows))]
fn read_clipboard() -> Result<DynamicImage> { Err(ViewerError::new("unsupported")) }

