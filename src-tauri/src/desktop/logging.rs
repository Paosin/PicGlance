use std::{fs::{self, OpenOptions}, io::{self, Write}, path::PathBuf, sync::{Arc, Mutex}, time::SystemTime};

pub struct Log {
    dir: PathBuf,
    limit: u64,
    write_lock: Mutex<()>,
}

impl Log {
    pub fn open(primary: PathBuf, fallback: Option<PathBuf>, limit: u64) -> io::Result<Self> {
        let open = |dir: PathBuf| {
            fs::create_dir_all(&dir)?;
            OpenOptions::new().create(true).append(true).open(dir.join("picglance.log"))?;
            Ok(Self { dir, limit, write_lock: Mutex::new(()) })
        };
        open(primary).or_else(|error| fallback.map_or(Err(error), open))
    }

    pub fn record(&self, event: &str, message: &str) {
        let _guard = self.write_lock.lock().unwrap_or_else(|error| error.into_inner());
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
        let line = format!("{}.{:03} {} {}\n", timestamp.as_secs(), timestamp.subsec_millis(),
            event.escape_debug(), message.escape_debug());
        let path = self.dir.join("picglance.log");
        if let Ok(info) = path.metadata() {
            if info.len() > 0 && info.len().saturating_add(line.len() as u64) > self.limit {
                for index in (1..=3).rev() {
                    let old = if index == 1 { path.clone() } else { self.dir.join(format!("picglance.log.{}", index - 1)) };
                    if old.is_file() {
                        let _ = fs::rename(old, self.dir.join(format!("picglance.log.{index}")));
                    }
                }
            }
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = file.write_all(line.as_bytes());
        }
    }

    pub fn capture_panics(self: &Arc<Self>) {
        let previous = std::panic::take_hook();
        let log = Arc::clone(self);
        std::panic::set_hook(Box::new(move |info| {
            log.record("panic", &format!("{info}"));
            previous(info);
        }));
    }
}
