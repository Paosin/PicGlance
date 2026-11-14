use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use picglance_compression::{compress, Error, Format, Options};

// Run against the files listed in the compression handoff.
fn main() {
    let samples = std::env::args_os()
        .nth(1)
        .expect("pass the sample directory");
    let samples = Path::new(&samples);
    let tools = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tools");
    for (file, format, target, lock, drop) in [
        ("photo.jpg", None, 50_000, false, false),
        ("alpha.png", None, 55_000, false, false),
        ("alpha.png", Some(Format::Webp), 35_000, true, false),
        ("animated.gif", None, 600_000, false, false),
        ("animated.gif", Some(Format::Webp), 350_000, true, false),
        ("animated.webp", None, 300_000, false, false),
        ("animated.webp", Some(Format::Webp), 90_000, true, true),
        ("transparent.gif", None, 23_000, false, false),
        ("transparent.gif", Some(Format::Webp), 23_000, false, false),
        ("transparent.gif", Some(Format::Gif), 1, false, false),
    ] {
        let source = std::fs::read(samples.join(file)).unwrap();
        let options = Options {
            target_bytes: target,
            lock_dimensions: lock,
            allow_frame_drop: drop,
            format,
            ..Options::default()
        };
        let result = compress(&source, options, &tools, &AtomicBool::new(false), |_| {}).unwrap();
        assert_eq!(result.met_target, target > 1);
        let extension = result.format.mime_type().split_once('/').unwrap().1;
        let output = format!("{file}-{target}.{extension}");
        std::fs::write(samples.join(&output), &result.bytes).unwrap();
        println!(
            "{output}: {} -> {} bytes, {}x{}, Q{}, {} frames, target met={}, colors {:?} -> {:?}, bpp {:?} -> {:?}",
            source.len(),
            result.bytes.len(),
            result.width,
            result.height,
            result.quality,
            result.frame_count,
            result.met_target,
            result.original_color_count,
            result.output_color_count,
            result.original_bit_depth,
            result.output_bit_depth
        );
    }
    let source = std::fs::read(samples.join("animated.gif")).unwrap();
    let cancel = AtomicBool::new(false);
    let mut attempts = 0;
    let cancelled = compress(&source, Options::default(), &tools, &cancel, |_| {
        attempts += 1;
        cancel.store(true, Ordering::Relaxed);
    });
    assert!(matches!(cancelled, Err(Error::Cancelled)));
    assert_eq!(attempts, 1);
    println!("animated.gif: cancelled after first encode, no result bytes returned");
}
