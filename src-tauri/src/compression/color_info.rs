use std::io::Cursor;
use std::sync::atomic::AtomicBool;

use image::{ImageDecoder, ImageFormat, ImageReader};

use crate::{check_cancel, limits, Error};

#[derive(Default)]
pub(super) struct ColorInfo {
    pub palette_entries: Option<u32>,
    pub bits_per_pixel: Option<u16>,
}

pub(super) fn inspect(
    bytes: &[u8],
    format: ImageFormat,
    cancel: &AtomicBool,
) -> Result<ColorInfo, Error> {
    Ok(match format {
        ImageFormat::Png => png(bytes).unwrap_or_default(),
        ImageFormat::Gif => gif(bytes, cancel)?,
        ImageFormat::Bmp => bmp(bytes).unwrap_or_default(),
        _ => {
            let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
            reader.limits(limits());
            ColorInfo {
                palette_entries: None,
                bits_per_pixel: reader
                    .into_decoder()
                    .ok()
                    .map(|d| u16::from(d.color_type().bits_per_pixel())),
            }
        }
    })
}

fn png(bytes: &[u8]) -> Option<ColorInfo> {
    let channels = match *bytes.get(25)? {
        0 | 3 => 1,
        2 => 3,
        4 => 2,
        6 => 4,
        _ => return None,
    };
    let bits_per_pixel = Some(u16::from(*bytes.get(24)?) * channels);
    let mut palette_entries = None;
    if bytes[25] == 3 {
        let mut chunks = bytes.get(33..)?;
        while let Some(header) = chunks.first_chunk::<8>() {
            let size = u32::from_be_bytes(header[..4].try_into().ok()?);
            match &header[4..] {
                b"PLTE" => {
                    palette_entries = Some(size / 3);
                    break;
                }
                b"IDAT" | b"IEND" => break,
                _ => chunks = chunks.get((size as usize).checked_add(12)?..)?,
            }
        }
    }
    Some(ColorInfo {
        palette_entries,
        bits_per_pixel,
    })
}

fn gif(bytes: &[u8], cancel: &AtomicBool) -> Result<ColorInfo, Error> {
    let mut options = gif::DecodeOptions::new();
    options.skip_frame_decoding(true);
    let Ok(mut decoder) = options.read_info(bytes) else {
        return Ok(ColorInfo::default());
    };
    let global = decoder
        .global_palette()
        .map_or(0, |palette| palette.len() / 3);
    let mut largest = 0;
    loop {
        check_cancel(cancel)?;
        match decoder.next_frame_info() {
            Ok(Some(frame)) => {
                let colors = frame
                    .palette
                    .as_ref()
                    .map_or(global, |palette| palette.len() / 3);
                largest = largest.max(colors);
            }
            Ok(None) => break,
            Err(_) => return Ok(ColorInfo::default()),
        }
    }
    Ok(ColorInfo {
        palette_entries: (largest > 0).then_some(largest as u32),
        bits_per_pixel: (largest >= 2).then(|| (largest - 1).ilog2() as u16 + 1),
    })
}

fn bmp(bytes: &[u8]) -> Option<ColorInfo> {
    let dib_size = u32::from_le_bytes(bytes.get(14..18)?.try_into().ok()?);
    let offset = if dib_size == 12 { 24 } else { 28 };
    let bits = u16::from_le_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?);
    let palette_entries = if (1..=8).contains(&bits) {
        let max_colors = 1_u32 << bits;
        let count = if dib_size >= 40 {
            u32::from_le_bytes(bytes.get(46..50)?.try_into().ok()?)
        } else {
            0
        };
        let pixel_offset = u32::from_le_bytes(bytes.get(10..14)?.try_into().ok()?);
        let entry_size = if dib_size == 12 { 3 } else { 4 };
        let available = pixel_offset.checked_sub(dib_size.checked_add(14)?)? / entry_size;
        let count = if count == 0 {
            max_colors
        } else {
            count.min(max_colors)
        };
        Some(count.min(available)).filter(|count| *count > 0)
    } else {
        None
    };
    Some(ColorInfo {
        palette_entries,
        bits_per_pixel: (bits > 0).then_some(bits),
    })
}
