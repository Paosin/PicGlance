mod codecs;
mod color_info;
mod process;

use std::{
    io::Cursor,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use image::{
    codecs::{gif::GifDecoder, webp::WebPDecoder},
    metadata::LoopCount,
    AnimationDecoder, ImageDecoder, ImageFormat, ImageReader, Limits, RgbaImage,
};

use codecs::{Gif, Jpeg, Png, Strategy, Webp};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Jpeg,
    Png,
    Gif,
    Webp,
}

impl Format {
    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub target_bytes: usize,
    pub lock_dimensions: bool,
    pub format: Option<Format>,
    pub max_quality: u8,
    pub allow_frame_drop: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            target_bytes: 500 * 1024,
            lock_dimensions: false,
            format: None,
            max_quality: 95,
            allow_frame_drop: false,
        }
    }
}

#[derive(Debug)]
pub struct ResultImage {
    pub bytes: Vec<u8>,
    pub format: Format,
    pub width: u32,
    pub height: u32,
    pub frame_count: usize,
    pub quality: u8,
    pub source_bytes: usize,
    pub met_target: bool,
    pub original_color_count: Option<u32>,
    pub original_bit_depth: Option<u16>,
    pub output_color_count: Option<u32>,
    pub output_bit_depth: Option<u16>,
}

#[derive(Debug)]
pub enum Error {
    Cancelled,
    InvalidInput,
    Unsupported,
    TooLarge,
    ToolNotFound(String),
    ToolTimeout(String),
    ToolFailed(String),
    Codec(String),
    Io(std::io::Error),
}

#[derive(Clone, Copy, Debug)]
pub struct Progress {
    pub attempt: usize,
    pub width: u32,
    pub height: u32,
    pub quality: u8,
    pub bytes: usize,
}

struct Frame {
    pixels: RgbaImage,
    duration_ms: u32,
}

struct Input<'a> {
    frames: Vec<Frame>,
    gif_source: Option<&'a [u8]>,
    loops: LoopCount,
}

const MAX_INPUT: usize = 80 * 1024 * 1024;
const MAX_PIXELS: u64 = 64 * 1024 * 1024;
const MIN_LONG_EDGE: u32 = 16;

fn check_cancel(cancel: &AtomicBool) -> Result<(), Error> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}

fn decode_error(error: image::ImageError) -> Error {
    match error {
        image::ImageError::Limits(_) => Error::TooLarge,
        _ => Error::InvalidInput,
    }
}

fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_alloc = Some(MAX_PIXELS * 4);
    limits.max_image_width = Some(32_768);
    limits.max_image_height = Some(32_768);
    limits
}

fn check_dimensions((width, height): (u32, u32)) -> Result<(), Error> {
    if width == 0 || height == 0 {
        return Err(Error::InvalidInput);
    }
    if width > 32_768 || height > 32_768 || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(Error::TooLarge);
    }
    Ok(())
}

fn animated<'a>(
    mut decoder: impl AnimationDecoder<'a> + ImageDecoder,
    gif_source: Option<&'a [u8]>,
    cancel: &AtomicBool,
) -> Result<Input<'a>, Error> {
    let dimensions = decoder.dimensions();
    check_dimensions(dimensions)?;
    decoder.set_limits(limits()).map_err(decode_error)?;
    let loops = if let Some(source) = gif_source {
        // GIF counts repeats after the first play, whereas WebP counts total plays.
        match gif::DecodeOptions::new()
            .read_info(source)
            .map_err(|_| Error::InvalidInput)?
            .repeat()
        {
            gif::Repeat::Infinite => LoopCount::Infinite,
            gif::Repeat::Finite(repeats) => {
                LoopCount::Finite(std::num::NonZeroU32::new(u32::from(repeats) + 1).unwrap())
            }
        }
    } else {
        decoder.loop_count()
    };
    let max_frames = (MAX_PIXELS / (u64::from(dimensions.0) * u64::from(dimensions.1))).min(4096);
    let mut frames = Vec::new();
    let mut decoded = decoder.into_frames();
    loop {
        check_cancel(cancel)?;
        let Some(frame) = decoded.next() else { break };
        if frames.len() as u64 >= max_frames {
            return Err(Error::TooLarge);
        }
        let frame = frame.map_err(decode_error)?;
        let (n, d) = frame.delay().numer_denom_ms();
        frames.push(Frame {
            pixels: frame.into_buffer(),
            duration_ms: n / d,
        });
    }
    if frames.is_empty() {
        return Err(Error::InvalidInput);
    }
    Ok(Input {
        frames,
        gif_source,
        loops,
    })
}

fn decode<'a>(bytes: &'a [u8], kind: ImageFormat, cancel: &AtomicBool) -> Result<Input<'a>, Error> {
    if kind == ImageFormat::Gif {
        let decoder = GifDecoder::new(Cursor::new(bytes)).map_err(decode_error)?;
        return animated(decoder, Some(bytes), cancel);
    }
    if kind == ImageFormat::WebP {
        let decoder = WebPDecoder::new(Cursor::new(bytes)).map_err(decode_error)?;
        if decoder.has_animation() {
            return animated(decoder, None, cancel);
        }
    }
    if kind == ImageFormat::Png
        && image::codecs::png::PngDecoder::with_limits(Cursor::new(bytes), limits())
            .map_err(decode_error)?
            .is_apng()
            .map_err(decode_error)?
    {
        return Err(Error::Unsupported);
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), kind);
    reader.limits(limits());
    let mut decoder = reader.into_decoder().map_err(decode_error)?;
    check_dimensions(decoder.dimensions())?;
    let orientation = decoder.orientation().map_err(decode_error)?;
    let mut image = image::DynamicImage::from_decoder(decoder).map_err(decode_error)?;
    image.apply_orientation(orientation);
    Ok(Input {
        frames: vec![Frame {
            pixels: image.into_rgba8(),
            duration_ms: 0,
        }],
        gif_source: None,
        loops: LoopCount::Finite(std::num::NonZeroU32::MIN),
    })
}

fn formats(source: ImageFormat, animated: bool) -> &'static [Format] {
    use Format::*;
    match source {
        ImageFormat::Gif => &[Gif, Webp],
        ImageFormat::WebP if animated => &[Webp, Gif],
        ImageFormat::WebP => &[Webp, Jpeg, Png],
        ImageFormat::Png => &[Png, Jpeg, Webp],
        ImageFormat::Jpeg | ImageFormat::Bmp => &[Jpeg, Png, Webp],
        _ => &[],
    }
}

/// Run on a background worker. A cancelled job never returns candidate bytes.
pub fn compress(
    bytes: &[u8],
    options: Options,
    tools: &Path,
    cancel: &AtomicBool,
    mut progress: impl FnMut(Progress),
) -> Result<ResultImage, Error> {
    check_cancel(cancel)?;
    if options.target_bytes == 0 || !(10..=100).contains(&options.max_quality) {
        return Err(Error::InvalidInput);
    }
    if bytes.len() > MAX_INPUT {
        return Err(Error::TooLarge);
    }
    let kind = image::guess_format(bytes).map_err(|_| Error::Unsupported)?;
    if formats(kind, false).is_empty() {
        return Err(Error::Unsupported);
    }
    let input = decode(bytes, kind, cancel)?;
    let animated = input.frames.len() > 1;
    let formats = formats(kind, animated);
    let format = options.format.unwrap_or(formats[0]);
    if !formats.contains(&format) {
        return Err(Error::Unsupported);
    }

    let strategies: [&dyn Strategy; 4] = [&Jpeg, &Png(tools), &Gif(tools), &Webp];
    let strategy = strategies
        .into_iter()
        .find(|strategy| strategy.supports(format))
        .unwrap();
    let original = input.frames[0].pixels.dimensions();
    let mut dimensions = original;
    if original.0.max(original.1) > strategy.maximum_long_edge() {
        if options.lock_dimensions {
            return Err(Error::TooLarge);
        }
        dimensions = scaled_dimensions(original, strategy.maximum_long_edge());
    }
    let mut smallest: Option<ResultImage> = None;
    let mut attempt = 0;
    let (minimum, maximum) = strategy.quality_range();
    let maximum = maximum.min(options.max_quality);
    let mut frame_step = 1;
    let mut result = loop {
        let mut best = None;
        let mut low = minimum;
        let mut high = maximum;
        let mut quality = high;
        let mut tried = [false; 101];
        let mut probe = |quality: u8| {
            check_cancel(cancel)?;
            let encoded = strategy.encode(&input, dimensions, quality, frame_step, cancel)?;
            attempt += 1;
            progress(Progress {
                attempt,
                width: dimensions.0,
                height: dimensions.1,
                quality,
                bytes: encoded.len(),
            });
            check_cancel(cancel)?;
            let candidate = ResultImage {
                met_target: encoded.len() <= options.target_bytes,
                bytes: encoded,
                format,
                width: dimensions.0,
                height: dimensions.1,
                frame_count: input.frames.len().div_ceil(frame_step),
                quality,
                source_bytes: bytes.len(),
                original_color_count: None,
                original_bit_depth: None,
                output_color_count: None,
                output_bit_depth: None,
            };
            let met_target = candidate.met_target;
            if met_target {
                if best
                    .as_ref()
                    .is_none_or(|previous: &ResultImage| quality > previous.quality)
                {
                    best = Some(candidate);
                }
            } else if smallest
                .as_ref()
                .is_none_or(|previous| candidate.bytes.len() < previous.bytes.len())
            {
                smallest = Some(candidate);
            }
            Ok(met_target)
        };
        while low <= high {
            tried[usize::from(quality)] = true;
            if probe(quality)? {
                low = quality + 1;
            } else {
                high = quality - 1;
            }
            if low <= high {
                quality = low + (high - low) / 2;
            }
        }
        let radius = strategy.quality_probe_radius();
        // Palette sizes are not strictly monotonic. Verify nearby levels before resizing.
        for quality in low..low.saturating_add(radius).min(maximum + 1) {
            if !tried[usize::from(quality)] {
                probe(quality)?;
            }
        }
        if let Some(best) = best {
            break best;
        }

        if frame_step == 1
            && input.frames.len() >= 3
            && options.allow_frame_drop
            && strategy.can_drop_frames()
        {
            frame_step = 2;
            continue;
        }
        let long_edge = dimensions.0.max(dimensions.1);
        if options.lock_dimensions || long_edge <= MIN_LONG_EDGE {
            break smallest.ok_or(Error::InvalidInput)?;
        }
        let next_long_edge = strategy
            .next_long_edge(long_edge, animated)
            .max(MIN_LONG_EDGE);
        dimensions = scaled_dimensions(original, next_long_edge);
        frame_step = 1;
    };

    let output_kind = match result.format {
        Format::Jpeg => ImageFormat::Jpeg,
        Format::Png => ImageFormat::Png,
        Format::Gif => ImageFormat::Gif,
        Format::Webp => ImageFormat::WebP,
    };
    let original_color = color_info::inspect(bytes, kind, cancel)?;
    let output_color = color_info::inspect(&result.bytes, output_kind, cancel)?;
    result.original_color_count = original_color.palette_entries;
    result.original_bit_depth = original_color.bits_per_pixel;
    result.output_color_count = output_color.palette_entries;
    result.output_bit_depth = output_color.bits_per_pixel;
    check_cancel(cancel)?;
    Ok(result)
}

fn scaled_dimensions(original: (u32, u32), long_edge: u32) -> (u32, u32) {
    let scale = f64::from(long_edge) / f64::from(original.0.max(original.1));
    (
        ((f64::from(original.0) * scale).round() as u32).max(1),
        ((f64::from(original.1) * scale).round() as u32).max(1),
    )
}
