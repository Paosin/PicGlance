import argparse
from pathlib import Path
from PIL import Image, ImageChops, ImageSequence, ImageStat

CASES = [
    ("photo.jpg", "photo.jpg-50000.jpeg", "JPEG", 1, True),
    ("alpha.png", "alpha.png-55000.png", "PNG", 1, True),
    ("alpha.png", "alpha.png-35000.webp", "WEBP", 1, True),
    ("animated.gif", "animated.gif-600000.gif", "GIF", 1, True),
    ("animated.gif", "animated.gif-350000.webp", "WEBP", 1, True),
    ("animated.webp", "animated.webp-300000.webp", "WEBP", 1, True),
    ("animated.webp", "animated.webp-90000.webp", "WEBP", 2, True),
    ("transparent.gif", "transparent.gif-23000.gif", "GIF", 1, True),
    ("transparent.gif", "transparent.gif-23000.webp", "WEBP", 1, True),
    ("transparent.gif", "transparent.gif-1.gif", "GIF", 1, False),
]


def frames(image):
    pixels, durations = [], []
    for frame in ImageSequence.Iterator(image):
        frame.load()
        durations.append(frame.info.get("duration", 0))
        pixels.append(frame.convert("RGBA"))
    return pixels, durations


def verify(directory):
    for source, output, expected_format, step, same_dimensions in CASES:
        with Image.open(directory / source) as before, Image.open(directory / output) as after:
            assert after.format == expected_format, (output, after.format)
            source_frames, source_ms = frames(before)
            result_frames, result_ms = frames(after)
            expected_ms = [sum(source_ms[i:i + step]) for i in range(0, len(source_ms), step)]
            assert result_ms == expected_ms, (output, result_ms, expected_ms)
            if len(source_frames) > 1:
                expected_loop = before.info.get("loop")
                if before.format == "GIF" and after.format == "WEBP":
                    expected_loop = 1 if expected_loop is None else expected_loop + (expected_loop != 0)
                assert after.info.get("loop") == expected_loop, (output, after.info.get("loop"), expected_loop)
            if not same_dimensions:
                assert after.size == (16, 12), (output, after.size)
                assert result_frames[0].getchannel("A").getextrema() == (0, 255)
                continue
            assert before.size == after.size, (output, before.size, after.size)
            max_alpha_mae, max_rgb_mae = 0, 0
            for left, right in zip(source_frames[::step], result_frames, strict=True):
                alpha = ImageChops.difference(left.getchannel("A"), right.getchannel("A"))
                max_alpha_mae = max(max_alpha_mae, ImageStat.Stat(alpha).mean[0])
                if source.endswith(".gif") or expected_format == "WEBP":
                    assert alpha.getbbox() is None, (output, "alpha changed")
                white = Image.new("RGBA", left.size, "white")
                rgb = ImageChops.difference(
                    Image.alpha_composite(white, left).convert("RGB"),
                    Image.alpha_composite(white, right).convert("RGB"),
                )
                max_rgb_mae = max(max_rgb_mae, max(ImageStat.Stat(rgb).mean))
            assert max_alpha_mae < 3, (output, max_alpha_mae)
            assert max_rgb_mae < 28, (output, max_rgb_mae)
            print(f"{output}: {len(result_ms)} frames/{sum(result_ms)}ms, "
                  f"worst-frame channel MAE alpha {max_alpha_mae:.2f}, RGB {max_rgb_mae:.2f}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("samples", type=Path)
    verify(parser.parse_args().samples)
