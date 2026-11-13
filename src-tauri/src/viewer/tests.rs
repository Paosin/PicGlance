use super::*;
use std::io::Write;

#[test]
fn file_limit_is_checked_before_reading_or_probing() {
    let folder = Fixture::new();
    let path = folder.0.join("大 图.png");
    let file = File::create(&path).unwrap();
    file.set_len(MAX_FILE_BYTES + 1).unwrap();
    let error = read_file(&path).unwrap_err();
    assert_eq!(error.code, "too-large");
    assert_eq!(error.size_bytes, Some(MAX_FILE_BYTES + 1));
}

#[test]
fn probe_uses_contents_and_scales_static_preview() {
    let image = DynamicImage::new_rgba8(2048, 1024);
    let bytes = encode_png(&image).unwrap();
    let preview = decode(&bytes, DecodeSize { width: 320, height: 320 }, &live()).unwrap();
    assert_eq!((preview.width, preview.height, preview.frames), (2048, 1024, 1));
    assert_eq!(image::load_from_memory(preview.bytes.as_deref().unwrap()).unwrap().dimensions(), (320, 160));
    assert!(decode(&bytes[..25], DecodeSize::THUMBNAIL, &live()).is_err());
    assert!(decode(b"", DecodeSize::THUMBNAIL, &live()).is_err());
    assert!(decode(b"unknown", DecodeSize::THUMBNAIL, &live()).is_err());
}

#[test]
fn decodes_supported_static_formats_and_rejects_oversized_canvas() {
    let source = DynamicImage::new_rgb8(16, 9);
    for format in [
        image::ImageFormat::Jpeg, image::ImageFormat::Png, image::ImageFormat::Bmp,
        image::ImageFormat::Gif, image::ImageFormat::WebP, image::ImageFormat::Tiff,
        image::ImageFormat::Ico,
    ] {
        let mut encoded = Cursor::new(Vec::new());
        if format == image::ImageFormat::Ico {
            DynamicImage::ImageRgba8(source.to_rgba8()).write_to(&mut encoded, format).unwrap();
        } else {
            source.write_to(&mut encoded, format).unwrap();
        }
        let preview = decode(encoded.get_ref(), DecodeSize::THUMBNAIL, &live())
            .unwrap_or_else(|error| panic!("{format:?}: {error:?}"));
        assert_eq!((preview.width, preview.height, preview.frames), (16, 9, 1), "{format:?}");
    }
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="9"><rect width="16" height="9" fill="red"/></svg>"#;
    let preview = decode(svg, DecodeSize::THUMBNAIL, &live()).unwrap();
    let rendered = image::load_from_memory(preview.bytes.as_deref().unwrap()).unwrap();
    assert_eq!(rendered.get_pixel(0, 0).0, [255, 0, 0, 255]);
    let oversized = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10000" height="10000"/>"#;
    assert_eq!(decode(oversized, DecodeSize::THUMBNAIL, &live()).err().unwrap().code, "dimensions");
}

#[test]
fn jpeg_decodes_at_preview_size_beyond_full_pixel_limit() {
    // A neutral grayscale JPEG, with a source canvas above the 48 MP full-decode limit.
    let source = DynamicImage::new_luma8(10_000, 5_000);
    let mut encoded = Cursor::new(Vec::new());
    source.write_to(&mut encoded, image::ImageFormat::Jpeg).unwrap();
    let bytes = encoded.into_inner();
    drop(source);
    let preview = decode(&bytes, DecodeSize { width: 1_000, height: 800 }, &live()).unwrap();
    assert_eq!((preview.width, preview.height), (10_000, 5_000));
    assert_eq!(image::load_from_memory(preview.bytes.as_deref().unwrap()).unwrap().dimensions(), (1_000, 500));
    assert_eq!(decode::pixels(&bytes).err().unwrap().code, "dimensions");
}

#[test]
fn oversized_png_is_rejected_before_allocating_source_pixels() {
    // The valid IHDR declares 200 MP. The compressed payload contains just one byte.
    let png = [
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82,
        0, 0, 78, 32, 0, 0, 39, 16, 8, 0, 0, 0, 0, 220, 79, 23, 126,
        0, 0, 0, 9, 73, 68, 65, 84, 120, 156, 99, 0, 0, 0, 1, 0, 1, 94, 255, 125, 249,
        0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ];
    assert_eq!(decode(&png, DecodeSize::THUMBNAIL, &live()).err().unwrap().code, "dimensions");
}

#[test]
fn directory_ids_are_sorted_and_unknown_paths_cannot_be_opened() {
    let folder = Fixture::new();
    for name in ["a.png", "B.jpg", "中文.webp", "skip.txt"] {
        File::create(folder.0.join(name)).unwrap().write_all(b"fixture").unwrap();
    }
    let viewer = Viewer::default();
    let id = viewer.accept_path(folder.0.join("B.jpg")).unwrap();
    let entries = viewer.directory(&id, &live()).unwrap();
    assert_eq!(entries.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), ["a.png", "B.jpg", "中文.webp"]);
    assert_eq!(entries[1].id, id);
    assert!(viewer.path("C:\\ungranted.png").is_err());
    fs::remove_file(folder.0.join("B.jpg")).unwrap();
    assert_eq!(read_file(&viewer.path(&id).unwrap()).unwrap_err().code, "missing");
}

#[test]
fn cancellation_is_checked_before_expensive_work() {
    let cancel = live();
    cancel.store(true, Ordering::Relaxed);
    let result = decode(b"whatever", DecodeSize::THUMBNAIL, &cancel);
    assert_eq!(result.err().unwrap().code, "cancelled");
}

#[test]
fn published_resource_keeps_the_validated_bytes_after_source_replacement() {
    let fixture = Fixture::new();
    let path = fixture.0.join("source.png");
    let bytes = encode_png(&DynamicImage::new_rgba8(16, 9)).unwrap();
    fs::write(&path, &bytes).unwrap();
    let viewer = Viewer::default();
    let details = viewer.publish(Some(path.clone()), bytes.clone(), DecodeSize::THUMBNAIL, &live()).unwrap();
    fs::write(path, b"replacement").unwrap();
    assert_eq!(viewer.resource(&details.original.id).unwrap().as_slice(), bytes);
    let input = viewer.editor_input(&details.document.id).unwrap();
    assert_eq!(input.document, details.document);
    assert_eq!(input.original_bytes, bytes);
    assert!(viewer.editor_input(&details.original.id).is_err());
    assert!(viewer.editor_input("C:\\ungranted.png").is_err());
}

#[test]
fn prepared_neighbor_is_never_reused_after_file_replacement() {
    let fixture = Fixture::new();
    let path = fixture.0.join("neighbor.png");
    let original = encode_png(&DynamicImage::new_rgba8(16, 9)).unwrap();
    let replacement = encode_png(&DynamicImage::new_rgba8(12, 8)).unwrap();
    fs::write(&path, &original).unwrap();
    let viewer = Viewer::default();
    let size = DecodeSize::THUMBNAIL;
    assert!(viewer.open_path(path.clone(), size, true, &live()).unwrap().is_none());
    fs::write(&path, &replacement).unwrap();
    let details = viewer.open_path(path, size, false, &live()).unwrap().unwrap();
    assert_eq!(details.document.width, 12);
    assert_eq!(viewer.resource(&details.original.id).unwrap().as_slice(), replacement);
}

#[test]
fn clipboard_document_keeps_original_dimensions_and_has_no_source() {
    let bytes = encode_png(&DynamicImage::new_rgba8(512, 256)).unwrap();
    let details = Viewer::default().publish(None, bytes, DecodeSize::THUMBNAIL, &live()).unwrap();
    assert_eq!(details.document.source_path, None);
    assert_eq!((details.document.width, details.document.height), (512, 256));
    assert!(details.metadata.is_empty());
    assert!(!details.gps);
}

#[cfg(windows)]
#[test]
fn rename_preserves_extension_when_omitted_and_updates_document_and_directory() {
    let folder = Fixture::new();
    let old = folder.0.join("b.PNG");
    let bytes = encode_png(&DynamicImage::new_rgba8(16, 9)).unwrap();
    fs::write(&old, &bytes).unwrap();
    fs::write(folder.0.join("a.PNG"), &bytes).unwrap();
    let viewer = Viewer::default();
    let grant = viewer.accept_path(old.clone()).unwrap();
    viewer.directory(&grant, &live()).unwrap();
    let details = viewer.publish(Some(old.clone()), bytes.clone(), DecodeSize::THUMBNAIL, &live()).unwrap();
    let id = &details.document.id;
    assert_eq!(viewer.rename(id, "a").unwrap_err().code, "name-exists");
    assert_eq!(viewer.rename(id, "..\\outside.png").unwrap_err().code, "invalid-name");
    assert_eq!(viewer.rename(id, "CON.png").unwrap_err().code, "invalid-name");
    assert_eq!(viewer.rename(id, "hidden.txt").unwrap_err().code, "invalid-name");
    assert!(old.exists());
    let updated = viewer.rename(id, "中文 名").unwrap();
    assert_eq!(updated.name, "中文 名.PNG");
    assert_eq!(updated.id, *id);
    assert!(!old.exists());
    assert_eq!(fs::read(folder.0.join("中文 名.PNG")).unwrap(), bytes);
    assert_eq!(viewer.editor_input(id).unwrap().source_path, Some(folder.0.join("中文 名.PNG")));
    let entries = viewer.directory(&grant, &live()).unwrap();
    assert_eq!(entries.iter().map(|item| item.name.as_str()).collect::<Vec<_>>(), ["a.PNG", "中文 名.PNG"]);
    assert_eq!(entries[1].id, grant);
    let updated = viewer.rename(id, "explicit.png").unwrap();
    assert_eq!(updated.name, "explicit.png");
}

#[test]
fn exif_location_is_flagged_and_malformed_exif_falls_back() {
    // EXIF APP1 payload with a camera maker and primary GPS coordinates.
    let exif = [
        0x45, 0x78, 0x69, 0x66, 0x00, 0x00, 0x4d, 0x4d, 0x00, 0x2a, 0x00, 0x00,
        0x00, 0x08, 0x00, 0x02, 0x01, 0x0f, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0a,
        0x00, 0x00, 0x00, 0x26, 0x88, 0x25, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x50, 0x69, 0x63, 0x47,
        0x6c, 0x61, 0x6e, 0x63, 0x65, 0x00, 0x00, 0x04, 0x00, 0x01, 0x00, 0x02,
        0x00, 0x00, 0x00, 0x02, 0x4e, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x05,
        0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x66, 0x00, 0x03, 0x00, 0x02,
        0x00, 0x00, 0x00, 0x02, 0x45, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x05,
        0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x7e, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0d,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x79, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x1c,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    ];
    let mut jpeg = vec![0xff, 0xd8, 0xff, 0xe1];
    jpeg.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
    jpeg.extend_from_slice(&exif);
    jpeg.extend_from_slice(&[0xff, 0xd9]);
    let (fields, gps) = metadata(&jpeg);
    assert!(gps);
    assert!(fields.iter().any(|field| field.tag == "Make" && field.value.contains("PicGlance")));
    assert!(fields.iter().any(|field| field.tag == "GPSLatitude"));
    assert!(!metadata(&jpeg[..30]).1);
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!("picglance-viewer-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn live() -> AtomicBool { AtomicBool::new(false) }

