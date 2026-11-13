use super::{encode, format_for_path, EditError, RgbaImage};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) { let _ = fs::remove_file(&self.0); }
}

fn stage(path: &Path, bytes: &[u8]) -> Result<TempFile, EditError> {
    let parent = path.parent().ok_or(EditError::InvalidInput)?;
    let (temporary, mut file): (TempFile, File) = loop {
        let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".picglance-edit-{}-{sequence}.tmp", std::process::id()));
        match OpenOptions::new().create_new(true).write(true).open(&candidate) {
            Ok(file) => break (TempFile(candidate), file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(EditError::Io),
        }
    };
    file.write_all(bytes).and_then(|_| file.sync_all()).map_err(|_| EditError::Io)?;
    Ok(temporary)
}

pub fn save_new(path: &Path, image: &RgbaImage) -> Result<(), EditError> {
    let temporary = stage(path, &encode(image, format_for_path(path)?)?)?;
    publish_new(&temporary.0, path).map_err(|error| match error.kind() {
        io::ErrorKind::AlreadyExists => EditError::Conflict,
        _ => EditError::Io,
    })
}

#[cfg(windows)]
fn publish_new(temporary: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(existing: *const u16, destination: *const u16, flags: u32) -> i32;
    }

    fn wide(path: &Path) -> io::Result<Vec<u16>> {
        let mut value: Vec<_> = path.as_os_str().encode_wide().collect();
        if value.contains(&0) { return Err(io::ErrorKind::InvalidInput.into()); }
        value.push(0);
        Ok(value)
    }

    // Canonicalize the parent to preserve long Windows paths. A zero flag set
    // keeps the operation on the same volume and rejects existing destinations.
    let parent = fs::canonicalize(temporary.parent().ok_or(io::ErrorKind::InvalidInput)?)?;
    let from = wide(&parent.join(temporary.file_name().ok_or(io::ErrorKind::InvalidInput)?))?;
    let to = wide(&parent.join(destination.file_name().ok_or(io::ErrorKind::InvalidInput)?))?;
    // Both buffers are NUL-terminated and live for the duration of the call.
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(windows))]
fn publish_new(temporary: &Path, destination: &Path) -> io::Result<()> {
    fs::hard_link(temporary, destination)
}

pub fn overwrite(path: &Path, image: &RgbaImage, expected_source: &[u8]) -> Result<Vec<u8>, EditError> {
    let format = format_for_path(path)?;
    if image::guess_format(expected_source).map_err(|_| EditError::Unsupported)? != format {
        return Err(EditError::Unsupported);
    }
    let bytes = encode(image, format)?;
    let temporary = stage(path, &bytes)?;
    let metadata = fs::symlink_metadata(path).map_err(|_| EditError::Io)?;
    if !metadata.file_type().is_file() || metadata.len() != expected_source.len() as u64 ||
        fs::read(path).map_err(|_| EditError::Io)? != expected_source {
        return Err(EditError::Conflict);
    }
    fs::set_permissions(&temporary.0, metadata.permissions()).map_err(|_| EditError::Io)?;
    fs::rename(&temporary.0, path).map_err(|_| EditError::Io)?;
    Ok(bytes)
}
