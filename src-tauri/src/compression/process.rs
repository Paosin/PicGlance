use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::atomic::AtomicBool,
    thread,
    time::{Duration, Instant},
};

use crate::{check_cancel, Error};

const MAX_OUTPUT: u64 = 256 * 1024 * 1024;
const MAX_DIAGNOSTIC: u64 = 8192;

pub(super) fn run(
    path: &Path,
    args: &[String],
    input: &[u8],
    timeout: Duration,
    cancel: &AtomicBool,
) -> Result<Vec<u8>, Error> {
    check_cancel(cancel)?;
    let mut command = Command::new(path);
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let tool = path.display().to_string();
    let mut child = command.spawn().map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => Error::ToolNotFound(tool.clone()),
        _ => Error::Io(error),
    })?;
    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    thread::scope(|scope| {
        let write = scope.spawn(move || stdin.write_all(input));
        let read = scope.spawn(move || drain(stdout, MAX_OUTPUT));
        let diagnostics = scope.spawn(move || drain(stderr, MAX_DIAGNOSTIC));
        let started = Instant::now();
        let status = loop {
            let result = check_cancel(cancel).and_then(|()| {
                if started.elapsed() >= timeout {
                    Err(Error::ToolTimeout(tool.clone()))
                } else {
                    child.try_wait().map_err(Error::Io)
                }
            });
            match result {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(error);
                }
            }
        };
        let written = write.join().unwrap();
        let output = read.join().unwrap();
        let diagnostic = diagnostics.join().unwrap();
        if !status?.success() {
            let (diagnostic, _) = diagnostic.map_err(Error::Io)?;
            return Err(Error::ToolFailed(format!(
                "{tool}: {}",
                String::from_utf8_lossy(&diagnostic).trim()
            )));
        }
        written.map_err(Error::Io)?;
        let (output, overflow) = output.map_err(Error::Io)?;
        if overflow {
            return Err(Error::TooLarge);
        }
        Ok(output)
    })
}

fn drain(mut stream: impl Read, limit: u64) -> std::io::Result<(Vec<u8>, bool)> {
    let mut bytes = Vec::new();
    stream.by_ref().take(limit).read_to_end(&mut bytes)?;
    let overflow = std::io::copy(&mut stream, &mut std::io::sink())? > 0;
    Ok((bytes, overflow))
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    fn gifsicle() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tools/gifsicle.exe")
    }

    #[test]
    fn times_out_and_reaps_external_tool() {
        let error = run(
            &gifsicle(),
            &["-".into()],
            b"",
            Duration::ZERO,
            &AtomicBool::new(false),
        )
        .unwrap_err();
        assert!(matches!(error, Error::ToolTimeout(_)));
    }

    #[test]
    fn reports_missing_tool_and_invalid_tool_input() {
        let cancel = AtomicBool::new(false);
        let seconds = Duration::from_secs(5);
        assert!(matches!(
            run(
                &gifsicle().with_extension("missing"),
                &[],
                b"",
                seconds,
                &cancel
            ),
            Err(Error::ToolNotFound(_))
        ));
        assert!(matches!(
            run(&gifsicle(), &["-".into()], b"invalid GIF", seconds, &cancel),
            Err(Error::ToolFailed(_))
        ));
    }
}
