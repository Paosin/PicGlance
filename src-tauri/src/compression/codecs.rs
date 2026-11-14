use std::{io::Cursor, path::Path, ptr::NonNull, sync::atomic::AtomicBool, time::Duration};

use image::{imageops::FilterType, ImageFormat, RgbaImage};
use libwebp_sys as webp;

use crate::{check_cancel, process, Error, Format, Input};

pub(super) trait Strategy {
    fn name(&self) -> &'static str;
    fn supports(&self, format: Format) -> bool;
    fn quality_range(&self) -> (u8, u8) {
        (10, 100)
    }
    fn can_drop_frames(&self) -> bool {
        false
    }
    fn quality_probe_radius(&self) -> u8 {
        0
    }
    fn maximum_long_edge(&self) -> u32 {
        u32::MAX
    }
    fn next_long_edge(&self, current: u32, _animated: bool) -> u32 {
        current / 2
    }
    fn encode(
        &self,
        input: &Input<'_>,
        dimensions: (u32, u32),
        quality: u8,
        frame_step: usize,
        cancel: &AtomicBool,
    ) -> Result<Vec<u8>, Error>;
}

fn scale(pixels: &RgbaImage, (width, height): (u32, u32)) -> std::borrow::Cow<'_, RgbaImage> {
    if pixels.dimensions() == (width, height) {
        return std::borrow::Cow::Borrowed(pixels);
    }
    if pixels.pixels().all(|pixel| pixel[3] == 255) {
        return std::borrow::Cow::Owned(image::imageops::resize(
            pixels,
            width,
            height,
            FilterType::Triangle,
        ));
    }
    // Filter premultiplied color so invisible RGB does not bleed into visible edges.
    let mut premultiplied = pixels.clone();
    for pixel in premultiplied.pixels_mut() {
        let alpha = u16::from(pixel[3]);
        for channel in &mut pixel.0[..3] {
            *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
        }
    }
    let mut resized = image::imageops::resize(&premultiplied, width, height, FilterType::Triangle);
    for pixel in resized.pixels_mut() {
        let alpha = u16::from(pixel[3]);
        if alpha > 0 {
            for channel in &mut pixel.0[..3] {
                *channel = ((u16::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
            }
        }
    }
    std::borrow::Cow::Owned(resized)
}

pub(super) struct Jpeg;
impl Strategy for Jpeg {
    fn name(&self) -> &'static str {
        "MozJPEG"
    }
    fn supports(&self, format: Format) -> bool {
        format == Format::Jpeg
    }
    fn encode(
        &self,
        input: &Input<'_>,
        dimensions: (u32, u32),
        quality: u8,
        _: usize,
        _: &AtomicBool,
    ) -> Result<Vec<u8>, Error> {
        let rgba = scale(&input.frames[0].pixels, dimensions);
        // Composite onto white for the explicitly selected opaque format.
        let mut rgb = Vec::with_capacity(rgba.as_raw().len() / 4 * 3);
        for pixel in rgba.pixels() {
            let alpha = u16::from(pixel[3]);
            for &channel in &pixel.0[..3] {
                let value = (u16::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255;
                rgb.push(value as u8);
            }
        }
        let encode = || -> std::io::Result<Vec<u8>> {
            let mut encoder = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
            encoder.set_size(dimensions.0 as usize, dimensions.1 as usize);
            encoder.set_quality(f32::from(quality));
            let mut started = encoder.start_compress(Vec::new())?;
            started.write_scanlines(&rgb)?;
            started.finish()
        };
        // The wrapper unwinds on libjpeg fatal errors.
        std::panic::catch_unwind(encode)
            .map_err(|_| Error::Codec(self.name().into()))?
            .map_err(|e| Error::Codec(format!("{}: {e}", self.name())))
    }
}

pub(super) struct Png<'a>(pub &'a Path);
impl Strategy for Png<'_> {
    fn quality_probe_radius(&self) -> u8 {
        4
    }
    fn name(&self) -> &'static str {
        "pngquant"
    }
    fn supports(&self, format: Format) -> bool {
        format == Format::Png
    }
    fn encode(
        &self,
        input: &Input<'_>,
        dimensions: (u32, u32),
        quality: u8,
        _: usize,
        cancel: &AtomicBool,
    ) -> Result<Vec<u8>, Error> {
        let pixels = scale(&input.frames[0].pixels, dimensions);
        let mut png = Cursor::new(Vec::new());
        pixels
            .write_to(&mut png, ImageFormat::Png)
            .map_err(|e| Error::Codec(e.to_string()))?;
        let colors = 16 + u16::from(quality.min(95) - 10) * 240 / 85;
        let args = [
            format!("--quality=0-{}", quality.saturating_add(5).min(100)),
            "--speed=1".into(),
            "--force".into(),
            colors.to_string(),
            "-".into(),
        ];
        process::run(
            &self.0.join("pngquant.exe"),
            &args,
            png.get_ref(),
            Duration::from_secs(30),
            cancel,
        )
    }
}

pub(super) struct Gif<'a>(pub &'a Path);
impl Strategy for Gif<'_> {
    fn quality_probe_radius(&self) -> u8 {
        4
    }
    fn name(&self) -> &'static str {
        "gifsicle"
    }
    fn supports(&self, format: Format) -> bool {
        format == Format::Gif
    }
    fn next_long_edge(&self, current: u32, animated: bool) -> u32 {
        if animated {
            current * 3 / 4
        } else {
            current / 2
        }
    }

    fn encode(
        &self,
        input: &Input<'_>,
        dimensions: (u32, u32),
        quality: u8,
        _: usize,
        cancel: &AtomicBool,
    ) -> Result<Vec<u8>, Error> {
        let lossy = u16::from(95 - quality.min(95)) * 200 / 85;
        let mut args = vec![
            "-O3".into(),
            "-Okeep-empty".into(),
            format!("--lossy={lossy}"),
            "--colors=256".into(),
        ];
        let buffer;
        let source = if let Some(source) = input.gif_source {
            if input.frames[0].pixels.dimensions() != dimensions {
                args.push(format!("--resize={}x{}", dimensions.0, dimensions.1));
            }
            source
        } else {
            buffer = gif_from_frames(input, dimensions, cancel)?;
            &buffer
        };
        args.extend(["-".into(), "-o".into(), "-".into()]);
        process::run(
            &self.0.join("gifsicle.exe"),
            &args,
            source,
            Duration::from_secs(30),
            cancel,
        )
    }
}

fn gif_from_frames(
    input: &Input<'_>,
    (w, h): (u32, u32),
    cancel: &AtomicBool,
) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    let error = |e: gif::EncodingError| Error::Codec(e.to_string());
    let mut encoder =
        gif::Encoder::new(&mut bytes, w as u16, h as u16, &[0, 0, 0]).map_err(error)?;
    let loops = loop_value(input.loops);
    if loops != 1 {
        let repeat = if loops == 0 {
            gif::Repeat::Infinite
        } else {
            gif::Repeat::Finite((loops - 1).min(u32::from(u16::MAX)) as u16)
        };
        encoder.set_repeat(repeat).map_err(error)?;
    }
    let mut elapsed = 0u64;
    let mut elapsed_cs = 0;
    for frame in &input.frames {
        check_cancel(cancel)?;
        let mut rgba = scale(&frame.pixels, (w, h)).into_owned().into_raw();
        for pixel in rgba.chunks_exact_mut(4) {
            pixel[3] = if pixel[3] < 128 { 0 } else { 255 };
        }
        let mut output = gif::Frame::from_rgba_speed(w as u16, h as u16, &mut rgba, 10);
        elapsed += u64::from(frame.duration_ms);
        let rounded = (elapsed + 5) / 10;
        output.delay = u16::try_from(rounded - elapsed_cs)
            .map_err(|_| Error::Codec("Frame duration exceeds GIF range".into()))?;
        elapsed_cs = rounded;
        output.dispose = gif::DisposalMethod::Background;
        encoder.write_frame(&output).map_err(error)?;
    }
    encoder.into_inner().map_err(error)?;
    Ok(bytes)
}

pub(super) struct Webp;
impl Strategy for Webp {
    fn name(&self) -> &'static str {
        "libwebp"
    }
    fn supports(&self, format: Format) -> bool {
        format == Format::Webp
    }
    fn can_drop_frames(&self) -> bool {
        true
    }
    fn maximum_long_edge(&self) -> u32 {
        16_383
    }

    fn encode(
        &self,
        input: &Input<'_>,
        dimensions: (u32, u32),
        quality: u8,
        frame_step: usize,
        cancel: &AtomicBool,
    ) -> Result<Vec<u8>, Error> {
        if dimensions.0.max(dimensions.1) > 16_383 {
            return Err(Error::TooLarge);
        }
        if input.frames.len() == 1 {
            return Ok(
                encode_webp(&scale(&input.frames[0].pixels, dimensions), quality)?
                    .bytes()
                    .to_vec(),
            );
        }
        // Each full-canvas frame replaces the previous one. Muxing explicitly keeps
        // identical consecutive frames and their original delays.
        let mux = Mux(NonNull::new(unsafe { webp::WebPMuxNew() }).ok_or_else(webp_error)?);
        let params = webp::WebPMuxAnimParams {
            bgcolor: 0,
            loop_count: loop_value(input.loops).min(u32::from(u16::MAX)) as i32,
        };
        mux_status(unsafe { webp::WebPMuxSetAnimationParams(mux.0.as_ptr(), &params) })?;
        for group in input.frames.chunks(frame_step) {
            check_cancel(cancel)?;
            let data = encode_webp(&scale(&group[0].pixels, dimensions), quality)?;
            let duration: u32 = group.iter().map(|frame| frame.duration_ms).sum();
            if duration > 0xff_ffff {
                return Err(Error::Codec("Frame duration exceeds WebP range".into()));
            }
            let frame = webp::WebPMuxFrameInfo {
                bitstream: data.0,
                x_offset: 0,
                y_offset: 0,
                duration: duration as i32,
                id: webp::WEBP_CHUNK_ANMF,
                dispose_method: webp::WEBP_MUX_DISPOSE_NONE,
                blend_method: webp::WEBP_MUX_NO_BLEND,
                pad: [0],
            };
            // copy_data asks libwebp to own a copy of the borrowed frame data.
            mux_status(unsafe { webp::WebPMuxPushFrame(mux.0.as_ptr(), &frame, 1) })?;
        }
        let mut data = WebpData(webp::WebPData {
            bytes: std::ptr::null(),
            size: 0,
        });
        mux_status(unsafe { webp::WebPMuxAssemble(mux.0.as_ptr(), &mut data.0) })?;
        Ok(data.bytes().to_vec())
    }
}

fn loop_value(loops: image::metadata::LoopCount) -> u32 {
    match loops {
        image::metadata::LoopCount::Infinite => 0,
        image::metadata::LoopCount::Finite(count) => count.get(),
    }
}

fn webp_error() -> Error {
    Error::Codec("libwebp encoding failed".into())
}

fn mux_status(status: webp::WebPMuxError) -> Result<(), Error> {
    if status == webp::WEBP_MUX_OK {
        Ok(())
    } else {
        Err(Error::Codec(format!("libwebp mux error {status}")))
    }
}

struct Mux(NonNull<webp::WebPMux>);
impl Drop for Mux {
    fn drop(&mut self) {
        unsafe { webp::WebPMuxDelete(self.0.as_ptr()) };
    }
}

struct WebpData(webp::WebPData);
impl WebpData {
    fn bytes(&self) -> &[u8] {
        // The buffer is allocated by libwebp and belongs to this wrapper.
        if self.0.size == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.0.bytes, self.0.size) }
        }
    }
}
impl Drop for WebpData {
    fn drop(&mut self) {
        unsafe { webp::WebPFree(self.0.bytes.cast_mut().cast()) };
    }
}

fn encode_webp(pixels: &RgbaImage, quality: u8) -> Result<WebpData, Error> {
    let mut bytes = std::ptr::null_mut();
    // The packed RGBA buffer has width * height * 4 bytes, with no row padding.
    let size = unsafe {
        webp::WebPEncodeRGBA(
            pixels.as_raw().as_ptr(),
            pixels.width() as i32,
            pixels.height() as i32,
            (pixels.width() * 4) as i32,
            f32::from(quality),
            &mut bytes,
        )
    };
    let data = WebpData(webp::WebPData { bytes, size });
    if size == 0 || bytes.is_null() {
        Err(webp_error())
    } else {
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparent_colors_do_not_bleed_through_resized_edges() {
        let source = RgbaImage::from_fn(2, 1, |x, _| {
            image::Rgba(if x == 0 {
                [0, 0, 255, 0]
            } else {
                [255, 0, 0, 255]
            })
        });
        let resized = scale(&source, (1, 1));
        let [red, _, blue, alpha] = resized.get_pixel(0, 0).0;
        assert_eq!((red, blue), (255, 0));
        assert!((127..=128).contains(&alpha));
    }
}
