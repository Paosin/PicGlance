use std::path::{Path, PathBuf};

pub fn paths(args: &[String], cwd: &Path) -> Vec<PathBuf> {
    let mut args = args.iter().skip(1).peekable();
    match args.peek().map(|arg| arg.as_str()) {
        Some("--") => { args.next(); }
        Some(flag) if flag.starts_with('-') => return Vec::new(),
        _ => {}
    }
    args.map(|arg| cwd.join(arg)).filter(|path| path.is_file()).collect()
}
