use super::*;

fn sample() -> RgbaImage {
    RgbaImage::from_fn(40, 32, |x, y| Rgba([(x * 6) as u8, (y * 7) as u8, ((x * y) % 255) as u8, 255]))
}

fn point(x: f64, y: f64) -> Point { Point { x, y } }

#[test]
fn transforms_apply_in_sequence_and_crop_rejects_out_of_bounds() {
    let source = sample();
    let output = render(&source, &[
        Operation::RotateRight,
        Operation::FlipX,
        Operation::Crop { x: 2, y: 3, width: 5, height: 8 },
    ]).unwrap();
    assert_eq!(output.dimensions(), (5, 8));
    assert_eq!(output[(0, 0)], source[(3, 2)]);
    assert!(render(&source, &[Operation::Crop { x: 39, y: 0, width: 2, height: 2 }]).is_err());
    assert_eq!(render(&source, &[Operation::Resize { width: 9, height: 6 }]).unwrap().dimensions(), (9, 6));
    assert_eq!(render(&source, &[Operation::RotateLeft]).unwrap()[(0, 0)], source[(39, 0)]);
    assert_eq!(render(&source, &[Operation::FlipY]).unwrap()[(0, 0)], source[(0, 31)]);
}

#[test]
fn stroke_covers_gaps_and_blends_once() {
    let source = RgbaImage::new(40, 32);
    let output = render(&source, &[Operation::Brush {
        points: vec![point(3.5, 4.5), point(33.5, 4.5), point(3.5, 4.5)],
        tip: Tip::Square, color: [200, 30, 40, 128], width: 3.0,
    }]).unwrap();
    assert_eq!(output[(20, 4)], Rgba([200, 30, 40, 128]));
    assert_eq!(output[(20, 8)], Rgba([0; 4]));
}

#[test]
fn each_shape_draws_at_its_outline() {
    let source = RgbaImage::new(40, 32);
    for (shape, hit) in [
        (Shape::Line, (15, 15)), (Shape::Arrow, (20, 20)),
        (Shape::Rectangle, (15, 5)), (Shape::Ellipse, (15, 5)),
    ] {
        let output = render(&source, &[Operation::Shape {
            shape, from: point(5.5, 5.5), to: point(25.5, 25.5),
            color: [200, 30, 40, 255], width: 2.0,
        }]).unwrap();
        assert_eq!(output[hit], Rgba([200, 30, 40, 255]));
        assert_eq!(output[(0, 0)], Rgba([0; 4]));
    }
}

#[test]
fn mosaics_are_distinct_local_and_repeatable() {
    let source = sample();
    let mut results = Vec::new();
    for mode in [Mosaic::Pixel, Mosaic::Blur, Mosaic::Fill, Mosaic::Noise, Mosaic::Circle, Mosaic::Glass] {
        let operation = Operation::Mosaic {
            mode, points: vec![point(15.5, 15.5), point(25.5, 15.5)], radius: 6.0,
            block: 7, color: [250, 50, 12, 255], seed: 42,
        };
        let output = render(&source, std::slice::from_ref(&operation)).unwrap();
        assert_eq!(output[(2, 2)], source[(2, 2)]);
        assert_ne!(output, source);
        assert_eq!(output, render(&source, &[operation]).unwrap());
        assert!(!results.contains(&output));
        results.push(output);
    }
}

fn document(name: &str, width: u32, height: u32, mime: &str) -> ImageDocument {
    ImageDocument {
        id: "sample".into(), source_path: None, name: name.into(), width, height,
        frame_count: 1, size_bytes: 0,
        preview: crate::contracts::ImageResource { id: "preview".into(), mime_type: mime.into() },
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let root = std::env::temp_dir();
        for index in 0.. {
            let path = root.join(format!("picglance-editor-test-{}-{index}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("{error}"),
            }
        }
        unreachable!()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        for entry in std::fs::read_dir(&self.0).unwrap().flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
        let _ = std::fs::remove_dir(&self.0);
    }
}

#[test]
fn rejects_vector_even_when_viewer_preview_is_png() {
    let bytes = encode(&sample(), ImageFormat::Png).unwrap();
    assert!(matches!(EditorSource::open(&document("asset.SVG", 40, 32, "image/png"), bytes, None),
        Err(EditError::Unsupported)));
}

#[test]
fn transparent_export_round_trips_and_save_as_rejects_existing_alias() {
    let scratch = Scratch::new();
    let original = scratch.0.join("original.png");
    let alias = scratch.0.join("alias.png");
    let mut image = sample();
    image[(0, 0)] = Rgba([21, 42, 63, 71]);
    let bytes = encode(&image, ImageFormat::Png).unwrap();
    std::fs::write(&original, &bytes).unwrap();
    std::fs::hard_link(&original, &alias).unwrap();
    let source = EditorSource::open(&document("original.png", 40, 32, "image/png"),
        bytes.clone(), Some(original.clone())).unwrap();
    let operation = [Operation::RotateRight];
    assert_eq!(source.save_new(&operation, &alias), Err(EditError::Conflict));
    let output = scratch.0.join("rotated.png");
    source.save_new(&operation, &output).unwrap();
    let saved = decode(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(saved[(31, 0)], Rgba([21, 42, 63, 71]));
    assert_eq!(decode(&source.export_png(&operation).unwrap()).unwrap(), saved);
    assert_eq!(std::fs::read(original).unwrap(), bytes);
}

#[test]
fn overwrite_rejects_external_change_and_preserves_history_across_two_saves() {
    let scratch = Scratch::new();
    let path = scratch.0.join("source.png");
    let original = sample();
    let bytes = encode(&original, ImageFormat::Png).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let mut source = EditorSource::open(&document("source.png", 40, 32, "image/png"), bytes, Some(path.clone())).unwrap();
    source.overwrite(&[Operation::RotateRight]).unwrap();
    assert_eq!(decode(&std::fs::read(&path).unwrap()).unwrap().dimensions(), (32, 40));
    source.overwrite(&[Operation::FlipX]).unwrap();
    assert_eq!(decode(&std::fs::read(&path).unwrap()).unwrap()[(0, 0)], original[(39, 0)]);
    std::fs::write(&path, b"external change").unwrap();
    assert_eq!(source.overwrite(&[Operation::FlipY]), Err(EditError::Conflict));
    assert_eq!(std::fs::read(&path).unwrap(), b"external change");
}

#[test]
#[ignore = "requires the legacy PicGlance-tmp sample tree"]
fn real_images_preserve_source_and_render_correct_pixels() {
    let legacy = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../PicGlance-tmp");
    let scratch = Scratch::new();
    let jpeg = legacy.join("third_party/mozjpeg/testimages/testorig.jpg");
    let bytes = std::fs::read(&jpeg).unwrap();
    let source = EditorSource::open(&document("testorig.jpg", 227, 149, "image/jpeg"), bytes.clone(), Some(jpeg.clone())).unwrap();
    let operations = [
        Operation::Brush {
            points: vec![point(10.0, 12.0)], tip: Tip::Round, color: [255, 20, 20, 255], width: 8.0,
        },
        Operation::RotateRight,
    ];
    let preview = decode(&source.render_png(&operations).unwrap()).unwrap();
    assert_eq!(preview[(136, 10)], Rgba([255, 20, 20, 255]));
    let output = scratch.0.join("edited.jpg");
    source.save_new(&operations, &output).unwrap();
    let edited = decode(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(edited.dimensions(), (149, 227));
    assert!(edited[(136, 10)][0] > 200 && edited[(136, 10)][1] < 60);
    assert_eq!(std::fs::read(jpeg).unwrap(), bytes);

    let png = legacy.join("third_party/mozjpeg/java/doc/resources/glass.png");
    let bytes = std::fs::read(&png).unwrap();
    let source = EditorSource::open(&document("glass.png", 16, 16, "image/png"), bytes.clone(), Some(png.clone())).unwrap();
    let output = scratch.0.join("edited.png");
    source.save_new(&[Operation::FlipY], &output).unwrap();
    let edited = decode(&std::fs::read(output).unwrap()).unwrap();
    let input = decode(&bytes).unwrap();
    assert!(edited.pixels().any(|pixel| pixel[3] == 0));
    for y in 0..16 {
        for x in 0..16 { assert_eq!(edited[(x, y)], input[(x, 15 - y)]); }
    }
    assert_eq!(std::fs::read(png).unwrap(), bytes);
}
