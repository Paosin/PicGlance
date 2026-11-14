use super::{EditError, Mosaic, Point, Rgba, RgbaImage, Shape, Tip};
use image::Pixel;

pub(super) struct Mask {
    left: u32,
    top: u32,
    width: u32,
    height: u32,
    pixels: Vec<bool>,
}

impl Mask {
    pub(super) fn stroke(image: &RgbaImage, points: &[Point], radius: f64, tip: Tip) -> Result<Self, EditError> {
        if !radius.is_finite() || !(0.5..=512.0).contains(&radius) || points.is_empty() || points.len() > 32_768 {
            return Err(EditError::InvalidInput);
        }
        let (width, height) = (image.width(), image.height());
        if points.iter().any(|p| !valid(*p) || p.x < 0.0 || p.y < 0.0 ||
            p.x > f64::from(width) || p.y > f64::from(height)) {
            return Err(EditError::InvalidInput);
        }
        let (mut left, mut top, mut right, mut bottom) = (width, height, 0, 0);
        for p in points {
            left = left.min(lower(p.x, radius));
            top = top.min(lower(p.y, radius));
            right = right.max(upper(p.x, radius, width));
            bottom = bottom.max(upper(p.y, radius, height));
        }
        let mut mask = Self { left, top, width: right - left, height: bottom - top,
            pixels: vec![false; ((right - left) * (bottom - top)) as usize] };
        mask.segment(points[0], points[0], radius, tip);
        for pair in points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            // Bound each scan rectangle for long diagonal segments.
            let steps = ((b.x - a.x).abs().max((b.y - a.y).abs()) / (radius * 2.0).max(16.0)).ceil() as u32;
            for step in 0..steps {
                let interpolate = |i| {
                    let t = f64::from(i) / f64::from(steps);
                    Point { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t }
                };
                mask.segment(interpolate(step), interpolate(step + 1), radius, tip);
            }
        }
        Ok(mask)
    }

    fn segment(&mut self, a: Point, b: Point, radius: f64, tip: Tip) {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let length_squared = dx * dx + dy * dy;
        for y in lower(a.y.min(b.y), radius)..upper(a.y.max(b.y), radius, self.top + self.height) {
            for x in lower(a.x.min(b.x), radius)..upper(a.x.max(b.x), radius, self.left + self.width) {
                let px = f64::from(x) + 0.5 - a.x;
                let py = f64::from(y) + 0.5 - a.y;
                let covered = match tip {
                    Tip::Round => {
                        let t = if length_squared == 0.0 { 0.0 } else { ((px * dx + py * dy) / length_squared).clamp(0.0, 1.0) };
                        (px - t * dx).hypot(py - t * dy) <= radius
                    }
                    Tip::Square => {
                        // Intersect the parameter intervals where the moving square covers this pixel.
                        let interval = |p: f64, delta: f64| {
                            if delta == 0.0 {
                                return if p.abs() <= radius { (0.0, 1.0) } else { (f64::INFINITY, f64::NEG_INFINITY) };
                            }
                            let (lo, hi) = ((p - radius) / delta, (p + radius) / delta);
                            (lo.min(hi), lo.max(hi))
                        };
                        let (x0, x1) = interval(px, dx);
                        let (y0, y1) = interval(py, dy);
                        x0.max(y0).max(0.0) <= x1.min(y1).min(1.0)
                    }
                };
                if covered {
                    let index = ((y - self.top) * self.width + x - self.left) as usize;
                    self.pixels[index] = true;
                }
            }
        }
    }

    fn covered(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.pixels.iter().enumerate().filter(|(_, covered)| **covered).map(|(i, _)| {
            let i = i as u32;
            (self.left + i % self.width, self.top + i / self.width)
        })
    }

    pub(super) fn paint(&self, image: &mut RgbaImage, color: Rgba<u8>) {
        for (x, y) in self.covered() {
            image.get_pixel_mut(x, y).blend(&color);
        }
    }
}

fn lower(value: f64, padding: f64) -> u32 { (value - padding).floor() as u32 }
fn upper(value: f64, padding: f64, limit: u32) -> u32 { ((value + padding).ceil() as u32).min(limit) }
fn valid(p: Point) -> bool { p.x.is_finite() && p.y.is_finite() }

pub(super) fn outline(shape: Shape, from: Point, to: Point) -> Result<Vec<Point>, EditError> {
    if !valid(from) || !valid(to) { return Err(EditError::InvalidInput); }
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let points = match shape {
        Shape::Line => vec![from, to],
        Shape::Arrow => {
            let angle = dy.atan2(dx);
            let length = (dx.hypot(dy) * 0.28).min(30.0);
            let wing = |offset: f64| Point { x: to.x - length * (angle + offset).cos(),
                y: to.y - length * (angle + offset).sin() };
            vec![from, to, wing(0.5), to, wing(-0.5)]
        }
        Shape::Rectangle => vec![from, Point { x: to.x, y: from.y }, to, Point { x: from.x, y: to.y }, from],
        Shape::Ellipse => {
            let steps = ((dx.abs().max(dy.abs()) * 2.0).ceil() as usize).clamp(24, 4096);
            (0..=steps).map(|step| {
                let angle = std::f64::consts::TAU * step as f64 / steps as f64;
                Point { x: from.x + dx * (1.0 + angle.cos()) / 2.0,
                    y: from.y + dy * (1.0 + angle.sin()) / 2.0 }
            }).collect()
        }
    };
    Ok(points)
}

pub(super) fn premultiply(raster: &mut RgbaImage) {
    for p in raster.pixels_mut() {
        let alpha = u32::from(p[3]);
        for channel in &mut p.0[..3] { *channel = ((u32::from(*channel) * alpha + 127) / 255) as u8; }
    }
}

pub(super) fn unpremultiply(raster: &mut RgbaImage) {
    for p in raster.pixels_mut() {
        let alpha = u32::from(p[3]);
        if alpha != 0 {
            for channel in &mut p.0[..3] { *channel = ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8; }
        }
    }
}

pub(super) fn mosaic(image: &mut RgbaImage, mask: &Mask, mode: Mosaic, block: u32, color: Rgba<u8>, seed: u32) -> Result<(), EditError> {
    if !(2..=128).contains(&block) { return Err(EditError::InvalidInput); }
    if matches!(mode, Mosaic::Fill) {
        mask.paint(image, color);
        return Ok(());
    }

    let padding = block * 3;
    let left = mask.left.saturating_sub(padding);
    let top = mask.top.saturating_sub(padding);
    let right = (mask.left + mask.width + padding).min(image.width());
    let bottom = (mask.top + mask.height + padding).min(image.height());
    let mut source = image::imageops::crop_imm(image, left, top, right - left, bottom - top).to_image();
    if matches!(mode, Mosaic::Blur) {
        premultiply(&mut source);
        source = image::imageops::blur(&source, block as f32 / 3.0);
        unpremultiply(&mut source);
    }
    let sample = |x: u32, y: u32| source[(x.clamp(left, right - 1) - left, y.clamp(top, bottom - 1) - top)];
    for (x, y) in mask.covered() {
        let cx = (x / block * block + block / 2).min(image.width() - 1);
        let cy = (y / block * block + block / 2).min(image.height() - 1);
        let result = match mode {
            Mosaic::Pixel => sample(cx, cy),
            Mosaic::Blur => sample(x, y),
            Mosaic::Noise => {
                let offset = (hash(x, y, seed) % 129) as i16 - 64;
                let mut pixel = sample(x, y);
                for channel in &mut pixel.0[..3] { *channel = (i16::from(*channel) + offset).clamp(0, 255) as u8; }
                pixel
            }
            Mosaic::Circle => {
                if f64::from(x.abs_diff(cx)).hypot(f64::from(y.abs_diff(cy))) > f64::from(block) / 2.0 { continue; }
                sample(cx, cy)
            }
            Mosaic::Glass => {
                let (sx, sy) = glass_center(x, y, block, seed);
                sample(sx, sy)
            }
            Mosaic::Fill => unreachable!(),
        };
        image.put_pixel(x, y, result);
    }
    Ok(())
}

fn hash(x: u32, y: u32, seed: u32) -> u32 {
    let h = x.wrapping_mul(0x9e3779b9) ^ y.wrapping_mul(0x85ebca6b) ^ seed;
    let h = (h ^ (h >> 16)).wrapping_mul(0x7feb352d);
    let h = (h ^ (h >> 15)).wrapping_mul(0x846ca68b);
    h ^ (h >> 16)
}

fn glass_center(x: u32, y: u32, block: u32, seed: u32) -> (u32, u32) {
    let (gx, gy) = (x / block, y / block);
    let mut closest = (x, y);
    let mut distance = u64::MAX;
    for row in gy.saturating_sub(2)..=gy + 2 {
        for col in gx.saturating_sub(2)..=gx + 2 {
            let random = hash(col, row, seed);
            let sx = col * block + random % block;
            let sy = row * block + (random >> 16) % block;
            let d = u64::from(x.abs_diff(sx)).pow(2) + u64::from(y.abs_diff(sy)).pow(2);
            if d < distance {
                closest = (sx, sy);
                distance = d;
            }
        }
    }
    closest
}
