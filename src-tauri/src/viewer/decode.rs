use super::{check, DecodeSize, ViewerError};
use image::codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder};
#[cfg(test)]
use image::metadata::LoopCount;
use image::{AnimationDecoder, DynamicImage, GenericImageView, ImageDecoder, ImageError, ImageFormat, ImageReader, Limits};
use std::io::Cursor;
use std::sync::atomic::AtomicBool;

const MAX_SIDE: u32 = 32_768;
const MAX_PIXELS: u64 = 48_000_000;
const MAX_BASELINE_JPEG_PIXELS: u64 = 200_000_000;
const MAX_JPEG_PREVIEW_BYTES: usize = 64 * 1024 * 1024;
const MAX_ANIMATION_PIXELS: u64 = 256_000_000;
const MAX_FRAMES: u32 = 20_000;

pub(super) struct Preview {
    pub width: u32,
    pub height: u32,
    pub frames: u32,
    pub bytes: Option<Vec<u8>>,
    pub mime: &'static str,
}

pub(super) enum Format {
    Raster(ImageFormat, &'static str),
    Svg,
}

pub(super) fn format(bytes: &[u8]) -> Result<Format, ViewerError> {
    use ImageFormat::*;
    let (kind, mime) = match image::guess_format(bytes) {
        Ok(Jpeg) => (Jpeg, "image/jpeg"),
        Ok(Png) => (Png, "image/png"),
        Ok(Gif) => (Gif, "image/gif"),
        Ok(WebP) => (WebP, "image/webp"),
        Ok(Bmp) => (Bmp, "image/bmp"),
        Ok(Tiff) => (Tiff, "image/tiff"),
        Ok(Ico) => (Ico, "image/x-icon"),
        Err(_) if bytes.starts_with(&[0xff, 0xd8]) => (Jpeg, "image/jpeg"),
        Err(_) if std::str::from_utf8(bytes).is_ok_and(|s| s.trim_start_matches('\u{feff}').trim_start().starts_with('<')) => return Ok(Format::Svg),
        _ if bytes.is_empty() => return Err(ViewerError::new("invalid-input")),
        _ => return Err(ViewerError::new("unsupported")),
    };
    Ok(Format::Raster(kind, mime))
}

pub(super) fn decode(bytes: &[u8], size: DecodeSize, cancel: &AtomicBool) -> Result<Preview, ViewerError> {
    check(cancel)?;
    let (kind, mime) = match format(bytes)? {
        Format::Raster(kind, mime) => (kind, mime),
        Format::Svg => return render_svg(bytes, size, cancel),
    };
    if kind == ImageFormat::Jpeg && !bytes.windows(2).skip(2).any(|marker| marker == [0xff, 0xd9]) {
        return Err(ViewerError::new("invalid-input"));
    }
    if kind == ImageFormat::Jpeg {
        return jpeg_preview(bytes, size, cancel);
    }
    let input = Cursor::new(bytes);
    let animation = match kind {
        ImageFormat::Gif => {
            let mut decoder = GifDecoder::new(input)?;
            decoder.set_limits(limits())?;
            Some(read_animation(decoder.dimensions(), decoder, size, cancel)?)
        }
        ImageFormat::Png => {
            let decoder = PngDecoder::with_limits(input, limits())?;
            if decoder.is_apng()? {
                Some(read_animation(decoder.dimensions(), decoder.apng()?, size, cancel)?)
            } else {
                None
            }
        }
        ImageFormat::WebP => {
            let decoder = WebPDecoder::new(input)?;
            if decoder.has_animation() {
                Some(read_animation(decoder.dimensions(), decoder, size, cancel)?)
            } else {
                None
            }
        }
        _ => None,
    };
    if let Some(animation) = animation {
        if !size.is_thumbnail() && (animation.frames > 1 || kind != ImageFormat::Gif) {
            return Ok(Preview { width: animation.width, height: animation.height, frames: animation.frames, bytes: None, mime });
        }
        return scaled(animation.first, size, animation.frames, cancel);
    }
    scaled(raster_pixels(bytes, kind)?, size, 1, cancel)
}

fn jpeg_preview(bytes: &[u8], size: DecodeSize, cancel: &AtomicBool) -> Result<Preview, ViewerError> {
    use image::metadata::Orientation;
    use jpeg_decoder::{CodingProcess, Decoder, PixelFormat};

    let mut decoder = Decoder::new(Cursor::new(bytes));
    let malformed = |_| ViewerError::new("invalid-input");
    decoder.read_info().map_err(malformed)?;
    let info = decoder.info().ok_or_else(|| ViewerError::new("invalid-input"))?;
    let (width, height) = (u32::from(info.width), u32::from(info.height));
    let area = u64::from(width) * u64::from(height);
    if width > MAX_SIDE || height > MAX_SIDE || area > MAX_BASELINE_JPEG_PIXELS {
        return Err(ViewerError::new("dimensions"));
    }
    // Progressive JPEGs can allocate a full-resolution coefficient buffer even with scaled IDCT.
    if info.coding_process != CodingProcess::DctSequential
        || !matches!(info.pixel_format, PixelFormat::L8 | PixelFormat::RGB24)
    {
        dimensions(width, height)?;
        return scaled(raster_pixels(bytes, ImageFormat::Jpeg)?, size, 1, cancel);
    }
    let orientation = decoder.exif_data().and_then(Orientation::from_exif_chunk)
        .unwrap_or(Orientation::NoTransforms);
    let swaps_axes = matches!(orientation, Orientation::Rotate90 | Orientation::Rotate270
        | Orientation::Rotate90FlipH | Orientation::Rotate270FlipH);
    let (original_width, original_height) = if swaps_axes { (height, width) } else { (width, height) };
    let (w, h) = target(original_width, original_height, size);
    let (wanted_w, wanted_h) = if swaps_axes { (h, w) } else { (w, h) };
    let (decoded_w, decoded_h) = decoder.scale(wanted_w as u16, wanted_h as u16).map_err(malformed)?;
    if usize::from(decoded_w) * usize::from(decoded_h) * info.pixel_format.pixel_bytes() > MAX_JPEG_PREVIEW_BYTES {
        if area <= MAX_PIXELS {
            return scaled(raster_pixels(bytes, ImageFormat::Jpeg)?, size, 1, cancel);
        }
        return Err(ViewerError::new("dimensions"));
    }
    decoder.set_max_decoding_buffer_size(MAX_JPEG_PREVIEW_BYTES);
    check(cancel)?;
    let pixels = decoder.decode().map_err(malformed)?;
    let (decoded_w, decoded_h) = (u32::from(decoded_w), u32::from(decoded_h));
    let mut image = match info.pixel_format {
        PixelFormat::L8 => image::GrayImage::from_raw(decoded_w, decoded_h, pixels).map(DynamicImage::ImageLuma8),
        PixelFormat::RGB24 => image::RgbImage::from_raw(decoded_w, decoded_h, pixels).map(DynamicImage::ImageRgb8),
        _ => None,
    }.ok_or_else(|| ViewerError::new("invalid-input"))?;
    image.apply_orientation(orientation);
    check(cancel)?;
    if image.dimensions() != (w, h) {
        image = image.resize_exact(w, h, image::imageops::FilterType::Triangle);
    }
    check(cancel)?;
    Ok(Preview { width: original_width, height: original_height, frames: 1,
        bytes: Some(encode_preview(&image)?), mime: "image/png" })
}

pub(super) fn pixels(bytes: &[u8]) -> Result<DynamicImage, ViewerError> {
    let Format::Raster(kind, _) = format(bytes)? else {
        let preview = render_svg(bytes, DecodeSize::ORIGINAL, &AtomicBool::new(false))?;
        return Ok(image::load_from_memory(&preview.bytes.unwrap_or_default())?);
    };
    let input = Cursor::new(bytes);
    let cancel = AtomicBool::new(false);
    let first = match kind {
        ImageFormat::Gif => {
            let decoder = GifDecoder::new(input)?;
            Some(read_animation(decoder.dimensions(), decoder, DecodeSize::THUMBNAIL, &cancel)?.first)
        }
        ImageFormat::Png => {
            let decoder = PngDecoder::with_limits(input, limits())?;
            if decoder.is_apng()? {
                Some(read_animation(decoder.dimensions(), decoder.apng()?, DecodeSize::THUMBNAIL, &cancel)?.first)
            } else { None }
        }
        ImageFormat::WebP => {
            let decoder = WebPDecoder::new(input)?;
            if decoder.has_animation() {
                Some(read_animation(decoder.dimensions(), decoder, DecodeSize::THUMBNAIL, &cancel)?.first)
            } else { None }
        }
        _ => None,
    };
    first.map(Ok).unwrap_or_else(|| raster_pixels(bytes, kind))
}

fn raster_pixels(bytes: &[u8], kind: ImageFormat) -> Result<DynamicImage, ViewerError> {
    let mut reader = ImageReader::with_format(Cursor::new(bytes), kind);
    reader.limits(limits());
    let mut decoder = reader.into_decoder()?;
    let (width, height) = decoder.dimensions();
    dimensions(width, height)?;
    if decoder.total_bytes() > 256 * 1024 * 1024 {
        return Err(ViewerError::new("dimensions"));
    }
    let orientation = decoder.orientation()?;
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    Ok(image)
}

struct Animation {
    width: u32,
    height: u32,
    frames: u32,
    first: DynamicImage,
    #[cfg(test)]
    delays: Vec<(u32, u32)>,
    #[cfg(test)]
    loops: LoopCount,
}

fn read_animation<'a>(canvas: (u32, u32), decoder: impl AnimationDecoder<'a>, size: DecodeSize, cancel: &AtomicBool) -> Result<Animation, ViewerError> {
    let (width, height) = canvas;
    dimensions(width, height)?;
    #[cfg(test)]
    let loops = decoder.loop_count();
    let mut first = None;
    let mut frames = 0;
    #[cfg(test)]
    let mut delays = Vec::new();
    let mut decoded = decoder.into_frames();
    loop {
        check(cancel)?;
        let Some(frame) = decoded.next().transpose()? else { break };
        frames += 1;
        if frames > MAX_FRAMES || u64::from(width) * u64::from(height) * u64::from(frames) > MAX_ANIMATION_PIXELS {
            return Err(ViewerError::new("dimensions"));
        }
        #[cfg(test)]
        delays.push(frame.delay().numer_denom_ms());
        if frames == 1 {
            first = Some(DynamicImage::ImageRgba8(frame.into_buffer()));
            if size.is_thumbnail() { break; }
        }
    }
    Ok(Animation {
        width, height, frames, first: first.ok_or_else(|| ViewerError::new("invalid-input"))?,
        #[cfg(test)]
        delays,
        #[cfg(test)]
        loops,
    })
}

fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(256 * 1024 * 1024);
    limits
}

pub(super) fn dimensions(width: u32, height: u32) -> Result<(), ViewerError> {
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(ViewerError::new("dimensions"));
    }
    Ok(())
}

fn target(width: u32, height: u32, size: DecodeSize) -> (u32, u32) {
    let scale = (f64::from(size.width) / f64::from(width)).min(f64::from(size.height) / f64::from(height)).min(1.0);
    (((f64::from(width) * scale).round() as u32).max(1), ((f64::from(height) * scale).round() as u32).max(1))
}

pub(super) fn scaled(image: DynamicImage, size: DecodeSize, frames: u32, cancel: &AtomicBool) -> Result<Preview, ViewerError> {
    check(cancel)?;
    let (width, height) = image.dimensions();
    dimensions(width, height)?;
    let (w, h) = target(width, height, size);
    let image = if (w, h) == (width, height) { image } else { image.resize_exact(w, h, image::imageops::FilterType::Triangle) };
    check(cancel)?;
    Ok(Preview { width, height, frames, bytes: Some(encode_preview(&image)?), mime: "image/png" })
}

fn render_svg(bytes: &[u8], size: DecodeSize, cancel: &AtomicBool) -> Result<Preview, ViewerError> {
    use resvg::{tiny_skia, usvg};
    // Resolve neither remote nor local hrefs. Rasterize so the webview never executes source SVG.
    let mut options = usvg::Options::default();
    static FONTS: std::sync::OnceLock<std::sync::Arc<usvg::fontdb::Database>> =
        std::sync::OnceLock::new();
    options.fontdb = FONTS.get_or_init(|| {
        let mut fonts = usvg::fontdb::Database::new();
        fonts.load_system_fonts();
        std::sync::Arc::new(fonts)
    }).clone();
    options.image_href_resolver.resolve_string = Box::new(|_, _| None);
    let data_resolver = usvg::ImageHrefResolver::default_data_resolver();
    options.image_href_resolver.resolve_data = Box::new(move |mime, data, options| {
        let Format::Raster(kind, _) = format(&data).ok()? else { return None };
        if !matches!(kind, ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::Gif | ImageFormat::WebP) {
            return None;
        }
        let (width, height) = ImageReader::with_format(Cursor::new(data.as_slice()), kind).into_dimensions().ok()?;
        dimensions(width, height).ok()?;
        if u64::from(width) * u64::from(height) > 16_000_000 { return None; }
        data_resolver(mime, data, options)
    });
    let tree = usvg::Tree::from_data(bytes, &options).map_err(|_| ViewerError::new("invalid-input"))?;
    let width = tree.size().width().ceil() as u32;
    let height = tree.size().height().ceil() as u32;
    dimensions(width, height)?;
    let (w, h) = target(width, height, size);
    check(cancel)?;
    let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or_else(|| ViewerError::new("dimensions"))?;
    let scale = (w as f32 / tree.size().width()).min(h as f32 / tree.size().height());
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    let bytes = pixmap.encode_png().map_err(|_| ViewerError::new("invalid-input"))?;
    Ok(Preview { width, height, frames: 1, bytes: Some(bytes), mime: "image/png" })
}

pub(super) fn encode_png(image: &DynamicImage) -> Result<Vec<u8>, ViewerError> {
    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, ImageFormat::Png)?;
    Ok(buffer.into_inner())
}

fn encode_preview(image: &DynamicImage) -> Result<Vec<u8>, ViewerError> {
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    let mut bytes = Vec::new();
    let encoder = PngEncoder::new_with_quality(&mut bytes, CompressionType::Fast, FilterType::NoFilter);
    image.write_with_encoder(encoder)?;
    Ok(bytes)
}

impl From<ImageError> for ViewerError {
    fn from(value: ImageError) -> Self {
        Self::new(match value {
            ImageError::Limits(_) => "dimensions",
            ImageError::Unsupported(_) => "unsupported",
            _ => "invalid-input",
        })
    }
}

#[cfg(test)]
#[path = "animation_tests.rs"]
mod animation_tests;

