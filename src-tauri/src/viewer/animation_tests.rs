use super::*;

const GIF: &str = "47494638396103000200810000ff000000000000000000000021ff0b4e45545343415045322e30030102000021f90400080000002c00000000030002000008060001081c18100021f904010d0001002c0000000003000200810000ff00000000000000000008060001081c1810003b";
const PNG: &str = "89504e470d0a1a0a0000000d49484452000000030000000208060000009d74661a000000086163544c00000002000000021d83f25c0000001a6663544c000000000000000300000002000000000000000000020019000066763ba10000001449444154789c63fccfc0f09f010a98600c100d002f1f0202b42aa96a0000001a6663544c0000000100000003000000020000000000000000000d00640000237cbda7000000186664415400000002789c636460f8ff9f010a98600c100d002d21020236336c750000000049454e44ae426082";
const WEBP: &str = "524946468400000057454250565038580a00000002000000020000010000414e494d06000000000000000200414e4d4628000000000000000000020000010000500000025650384c0f0000002f024000000710fd8ffe0722a2ff0100414e4d4628000000000000000000020000010000820000005650384c0f0000002f024000000710d1fffe0722a2ff0100";

fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes().chunks_exact(2).map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()).collect()
}

fn verify(decoder: impl AnimationDecoder<'static>, kind: ImageFormat, encoded: &[u8], loops: u32) {
    let cancel = AtomicBool::new(false);
    let size = DecodeSize { width: 800, height: 600 };
    let animation = read_animation((3, 2), decoder, size, &cancel).unwrap();
    assert_eq!(animation.frames, 2);
    assert!(matches!(animation.loops, LoopCount::Finite(count) if count.get() == loops));
    assert_eq!(animation.delays.iter().map(|(n, d)| n / d).collect::<Vec<_>>(), [80, 130]);
    let preview = decode(encoded, size, &cancel).unwrap();
    assert_eq!(preview.frames, 2);
    assert!(matches!(format(encoded), Ok(Format::Raster(format, mime)) if format == kind && mime == preview.mime));
    assert!(preview.bytes.is_none(), "Playback uses the original stream");
    let thumb = decode(encoded, DecodeSize::THUMBNAIL, &cancel).unwrap();
    assert_eq!(thumb.frames, 1);
    assert_eq!(thumb.mime, "image/png");
}

#[test]
fn gif_timing_and_loop_are_preserved() {
    let data = bytes(GIF);
    verify(GifDecoder::new(Cursor::new(data.clone())).unwrap(), ImageFormat::Gif, &data, 2);
}

#[test]
fn apng_timing_and_loop_are_preserved() {
    let data = bytes(PNG);
    verify(PngDecoder::new(Cursor::new(data.clone())).unwrap().apng().unwrap(), ImageFormat::Png, &data, 2);
}

#[test]
fn webp_timing_and_loop_are_preserved() {
    let data = bytes(WEBP);
    verify(WebPDecoder::new(Cursor::new(data.clone())).unwrap(), ImageFormat::WebP, &data, 2);
}

