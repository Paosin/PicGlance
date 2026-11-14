use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use image::{
    codecs::{gif::GifDecoder, webp::WebPDecoder},
    AnimationDecoder,
};
use picglance_compression::{compress, Error, Format, Options};

const TOOLS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../tools");
const PHOTO: &[u8] = include_bytes!("fixtures/testorig.jpg");

fn options(target_bytes: usize) -> Options {
    Options {
        target_bytes,
        ..Options::default()
    }
}

#[test]
fn keeps_jpeg_and_reaches_target_at_original_dimensions() {
    let result = compress(
        PHOTO,
        options(5000),
        Path::new(TOOLS),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert_eq!(result.format, Format::Jpeg);
    assert!(result.met_target);
    assert!(result.bytes.len() <= 5000);
    let source = image::load_from_memory(PHOTO).unwrap();
    let output = image::load_from_memory(&result.bytes).unwrap();
    assert_eq!(
        (source.width(), source.height()),
        (output.width(), output.height())
    );
    eprintln!(
        "JPEG {} → {} bytes, {}×{}, Q{}",
        PHOTO.len(),
        result.bytes.len(),
        output.width(),
        output.height(),
        result.quality
    );
}

#[test]
fn dimension_lock_returns_smallest_attempt_when_target_is_impossible() {
    let mut request = options(1);
    request.lock_dimensions = true;
    let cancel = AtomicBool::new(false);
    let locked = compress(PHOTO, request, Path::new(TOOLS), &cancel, |_| {}).unwrap();
    let shrunk = compress(PHOTO, options(1), Path::new(TOOLS), &cancel, |_| {}).unwrap();
    assert!(!locked.met_target && !shrunk.met_target);
    assert!(shrunk.width < locked.width && shrunk.height < locked.height);
    assert!(shrunk.width.max(shrunk.height) >= 16);
    assert!(shrunk.bytes.len() < locked.bytes.len());
}

#[test]
fn cancellation_discards_a_candidate_that_met_the_target() {
    let cancel = AtomicBool::new(false);
    let result = compress(
        PHOTO,
        options(PHOTO.len()),
        Path::new(TOOLS),
        &cancel,
        |_| {
            cancel.store(true, Ordering::Relaxed);
        },
    );
    assert!(matches!(result, Err(Error::Cancelled)));
}

fn animation() -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder =
            gif::Encoder::new(&mut bytes, 32, 32, &[0, 0, 0, 255, 0, 0, 0, 0, 255]).unwrap();
        encoder.set_repeat(gif::Repeat::Infinite).unwrap();
        for (i, delay) in [7, 13, 29, 11].into_iter().enumerate() {
            let mut pixels = vec![0; 1024];
            for row in 8..24 {
                for col in (i + 4)..(i + 18) {
                    pixels[row * 32 + col] = (i % 2 + 1) as u8;
                }
            }
            encoder
                .write_frame(&gif::Frame {
                    width: 32,
                    height: 32,
                    delay,
                    transparent: Some(0),
                    dispose: gif::DisposalMethod::Background,
                    buffer: pixels.into(),
                    ..gif::Frame::default()
                })
                .unwrap();
        }
    }
    bytes
}

#[test]
fn gif_and_webp_keep_transparency_and_frame_durations() {
    let source = animation();
    for format in [Format::Gif, Format::Webp] {
        let mut request = options(10_000);
        request.format = Some(format);
        request.allow_frame_drop = format == Format::Gif;
        let result = compress(
            &source,
            request,
            Path::new(TOOLS),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        let frames = match format {
            Format::Gif => GifDecoder::new(std::io::Cursor::new(&result.bytes))
                .unwrap()
                .into_frames()
                .collect_frames()
                .unwrap(),
            Format::Webp => WebPDecoder::new(std::io::Cursor::new(&result.bytes))
                .unwrap()
                .into_frames()
                .collect_frames()
                .unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(frames.len(), 4);
        for (frame, delay) in frames.iter().zip([70, 130, 290, 110]) {
            let (numerator, denominator) = frame.delay().numer_denom_ms();
            assert_eq!(numerator, delay * denominator);
            assert_eq!(frame.buffer().get_pixel(0, 0)[3], 0);
            assert_eq!(frame.buffer().get_pixel(12, 12)[3], 255);
        }
    }
}

#[test]
fn pngquant_keeps_transparency() {
    let mut pixels = image::RgbaImage::from_pixel(32, 32, image::Rgba([0, 0, 0, 0]));
    pixels.put_pixel(16, 16, image::Rgba([200, 150, 100, 127]));
    let mut png = std::io::Cursor::new(Vec::new());
    pixels.write_to(&mut png, image::ImageFormat::Png).unwrap();
    let result = compress(
        png.get_ref(),
        options(1000),
        Path::new(TOOLS),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert_eq!(result.format, Format::Png);
    let output = image::load_from_memory(&result.bytes).unwrap().to_rgba8();
    assert_eq!(output.get_pixel(0, 0)[3], 0);
    assert_eq!(output.get_pixel(16, 16)[3], 127);
}

#[test]
fn pngquant_checks_nearby_quality_when_the_smallest_palette_misses_the_target() {
    let pixels = image::RgbaImage::from_fn(39, 43, |x, y| {
        image::Rgba([
            (2 * x + 12 * y) as u8,
            (8 * x + 3 * y) as u8,
            (2 * x + y) as u8,
            255,
        ])
    });
    let mut png = std::io::Cursor::new(Vec::new());
    pixels.write_to(&mut png, image::ImageFormat::Png).unwrap();
    let mut request = options(610);
    request.lock_dimensions = true;
    let result = compress(
        png.get_ref(),
        request,
        Path::new(TOOLS),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert!(result.met_target);
    assert_eq!((result.width, result.height), (39, 43));
    assert!(result.bytes.len() <= 610);
}

#[test]
fn webp_frame_dropping_retries_at_original_size_and_combines_delays() {
    let source = animation();
    let cancel = AtomicBool::new(false);
    let mut request = options(10_000);
    request.format = Some(Format::Webp);
    request.lock_dimensions = true;
    request.max_quality = 10;
    let full = compress(&source, request, Path::new(TOOLS), &cancel, |_| {}).unwrap();
    request.target_bytes = full.bytes.len() - 1;
    request.allow_frame_drop = true;
    let result = compress(&source, request, Path::new(TOOLS), &cancel, |_| {}).unwrap();
    assert!(result.met_target);
    assert_eq!(
        (result.width, result.height, result.frame_count),
        (32, 32, 2)
    );
    let frames = WebPDecoder::new(std::io::Cursor::new(&result.bytes))
        .unwrap()
        .into_frames()
        .collect_frames()
        .unwrap();
    let durations: Vec<_> = frames
        .iter()
        .map(|frame| {
            let (n, d) = frame.delay().numer_denom_ms();
            n / d
        })
        .collect();
    assert_eq!(durations, [200, 400]);

    request.target_bytes = 10_000;
    request.format = None;
    let webp = compress(&result.bytes, request, Path::new(TOOLS), &cancel, |_| {}).unwrap();
    assert_eq!(webp.format, Format::Webp);
    request.format = Some(Format::Gif);
    let gif = compress(&result.bytes, request, Path::new(TOOLS), &cancel, |_| {}).unwrap();
    let frames = GifDecoder::new(std::io::Cursor::new(&gif.bytes))
        .unwrap()
        .into_frames()
        .collect_frames()
        .unwrap();
    assert_eq!(frames.len(), 2);
    for (frame, delay) in frames.iter().zip([200, 400]) {
        let (n, d) = frame.delay().numer_denom_ms();
        assert_eq!(n, delay * d);
        assert_eq!(frame.buffer().get_pixel(0, 0)[3], 0);
    }
}

#[test]
fn reports_palette_and_bit_depth_from_the_encoded_images() {
    let mut pixels = image::RgbaImage::from_pixel(32, 32, image::Rgba([0, 0, 0, 0]));
    pixels.put_pixel(16, 16, image::Rgba([200, 150, 100, 127]));
    let mut png = std::io::Cursor::new(Vec::new());
    pixels.write_to(&mut png, image::ImageFormat::Png).unwrap();
    let result = compress(
        png.get_ref(),
        options(1000),
        Path::new(TOOLS),
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert_eq!(result.original_color_count, None);
    assert_eq!(result.original_bit_depth, Some(32));
    assert_eq!(result.output_color_count, Some(2));
    assert_eq!(result.output_bit_depth, Some(1));
}
