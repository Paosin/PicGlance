use crate::contracts::ImageDocument;
use image::{imageops, DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits, Rgba, RgbaImage};
use serde::Deserialize;
use std::{io::Cursor, path::{Path, PathBuf}};

mod paint;
mod save;
pub mod ipc;
pub use save::{overwrite, save_new};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditError { InvalidInput, Unsupported, TooLarge, Io, Conflict }

const MAX_PIXELS: u64 = 64_000_000;
const MAX_SIDE: u32 = 32_768;

pub struct EditorSource {
    source_path: Option<PathBuf>,
    original_bytes: Vec<u8>,
    raster: RgbaImage,
}

impl EditorSource {
    fn suggested_name(&self) -> String {
        let stem = self.source_path.as_deref().and_then(Path::file_stem)
            .and_then(std::ffi::OsStr::to_str).unwrap_or("image");
        format!("{stem}-edited.png")
    }

    pub fn open(document: &ImageDocument, original_bytes: Vec<u8>, source_path: Option<PathBuf>) -> Result<Self, EditError> {
        if document.frame_count != 1 || document.preview.mime_type == "image/svg+xml" ||
            document.name.rsplit_once('.').is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("svg") || ext.eq_ignore_ascii_case("svgz")) {
            return Err(EditError::Unsupported);
        }
        let raster = decode(&original_bytes)?;
        if raster.dimensions() != (document.width, document.height) {
            return Err(EditError::Conflict);
        }
        Ok(Self { source_path, original_bytes, raster })
    }

    pub fn render_png(&self, operations: &[Operation]) -> Result<Vec<u8>, EditError> {
        if operations.is_empty() {
            return encode(&self.raster, ImageFormat::Png);
        }
        encode(&render(&self.raster, operations)?, ImageFormat::Png)
    }

    pub fn export_png(&self, operations: &[Operation]) -> Result<Vec<u8>, EditError> {
        self.render_png(operations)
    }

    pub fn save_new(&self, operations: &[Operation], path: &Path) -> Result<(), EditError> {
        save::save_new(path, &render(&self.raster, operations)?)
    }

    pub fn overwrite(&mut self, operations: &[Operation]) -> Result<(), EditError> {
        let path = self.source_path.as_ref().ok_or(EditError::Unsupported)?;
        let raster = render(&self.raster, operations)?;
        self.original_bytes = save::overwrite(path, &raster, &self.original_bytes)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Point { pub x: f64, pub y: f64 }

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tip { Round, Square }

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shape { Line, Arrow, Rectangle, Ellipse }

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mosaic { Pixel, Blur, Fill, Noise, Circle, Glass }

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Operation {
    Brush { points: Vec<Point>, tip: Tip, color: [u8; 4], width: f64 },
    Shape { shape: Shape, from: Point, to: Point, color: [u8; 4], width: f64 },
    Mosaic { mode: Mosaic, points: Vec<Point>, radius: f64, block: u32, color: [u8; 4], seed: u32 },
    Crop { x: u32, y: u32, width: u32, height: u32 },
    Resize { width: u32, height: u32 },
    RotateRight,
    RotateLeft,
    FlipX,
    FlipY,
}

fn validate_size(width: u32, height: u32) -> Result<(), EditError> {
    if width == 0 || height == 0 {
        return Err(EditError::InvalidInput);
    }
    if width > MAX_SIDE || height > MAX_SIDE || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(EditError::TooLarge);
    }
    Ok(())
}

pub fn decode(bytes: &[u8]) -> Result<RgbaImage, EditError> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format().map_err(|_| EditError::InvalidInput)?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_PIXELS * 8);
    reader.limits(limits.clone());
    let mut decoder = reader.into_decoder().map_err(decode_error)?;
    let (width, height) = decoder.dimensions();
    validate_size(width, height)?;
    decoder.set_limits(limits).map_err(decode_error)?;
    let orientation = decoder.orientation().map_err(decode_error)?;
    let mut raster = DynamicImage::from_decoder(decoder).map_err(decode_error)?;
    raster.apply_orientation(orientation);
    Ok(raster.into_rgba8())
}

fn decode_error(error: image::ImageError) -> EditError {
    match error {
        image::ImageError::Unsupported(_) => EditError::Unsupported,
        image::ImageError::Limits(_) => EditError::TooLarge,
        _ => EditError::InvalidInput,
    }
}

pub fn render(source: &RgbaImage, operations: &[Operation]) -> Result<RgbaImage, EditError> {
    validate_size(source.width(), source.height())?;
    let mut raster = source.clone();
    for operation in operations {
        match operation {
            Operation::Brush { points, tip, color, width } => {
                let mask = paint::Mask::stroke(&raster, points, *width / 2.0, *tip)?;
                mask.paint(&mut raster, Rgba(*color));
            }
            Operation::Shape { shape, from, to, color, width } => {
                let mut points = paint::outline(*shape, *from, *to)?;
                // Arrow wings can extend beyond the image at an edge.
                for point in &mut points {
                    point.x = point.x.clamp(0.0, f64::from(raster.width()));
                    point.y = point.y.clamp(0.0, f64::from(raster.height()));
                }
                let mask = paint::Mask::stroke(&raster, &points, *width / 2.0, Tip::Round)?;
                mask.paint(&mut raster, Rgba(*color));
            }
            Operation::Mosaic { mode, points, radius, block, color, seed } => {
                let mask = paint::Mask::stroke(&raster, points, *radius, Tip::Round)?;
                paint::mosaic(&mut raster, &mask, *mode, *block, Rgba(*color), *seed)?;
            }
            Operation::Crop { x, y, width, height } => {
                validate_size(*width, *height)?;
                if *width > raster.width().saturating_sub(*x) || *height > raster.height().saturating_sub(*y) {
                    return Err(EditError::InvalidInput);
                }
                raster = imageops::crop_imm(&raster, *x, *y, *width, *height).to_image();
            }
            Operation::Resize { width, height } => {
                validate_size(*width, *height)?;
                // Interpolate premultiplied colors so transparent RGB cannot bleed into edges.
                paint::premultiply(&mut raster);
                raster = imageops::resize(&raster, *width, *height, imageops::FilterType::Lanczos3);
                paint::unpremultiply(&mut raster);
            }
            Operation::RotateRight => raster = imageops::rotate90(&raster),
            Operation::RotateLeft => raster = imageops::rotate270(&raster),
            Operation::FlipX => imageops::flip_horizontal_in_place(&mut raster),
            Operation::FlipY => imageops::flip_vertical_in_place(&mut raster),
        }
    }
    Ok(raster)
}

pub fn format_for_path(path: &Path) -> Result<ImageFormat, EditError> {
    match ImageFormat::from_path(path) {
        Ok(format @ (ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP | ImageFormat::Bmp |
            ImageFormat::Tiff | ImageFormat::Ico)) => Ok(format),
        _ => Err(EditError::Unsupported),
    }
}

pub fn encode(raster: &RgbaImage, format: ImageFormat) -> Result<Vec<u8>, EditError> {
    let mut bytes = Cursor::new(Vec::new());
    if format == ImageFormat::Jpeg {
        let flattened = image::RgbImage::from_fn(raster.width(), raster.height(), |x, y| {
            let Rgba([r, g, b, alpha]) = raster[(x, y)];
            let alpha = u32::from(alpha);
            image::Rgb([r, g, b].map(|channel| ((u32::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255) as u8))
        });
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 92)
            .encode_image(&flattened).map_err(decode_error)?;
    } else {
        raster.write_to(&mut bytes, format).map_err(decode_error)?;
    }
    Ok(bytes.into_inner())
}

#[cfg(test)]
mod tests;
