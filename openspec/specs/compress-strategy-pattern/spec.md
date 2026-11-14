# compress-strategy-pattern Specification

## Purpose
TBD - created by archiving change picglance-compression. Update Purpose after archive.

## Requirements

### Requirement: Unified compress strategy interface

The system SHALL define an `ICompressStrategy` pure virtual interface in `src/core/compress/icompressstrategy.h` that all format-specific compress encoders implement. The interface SHALL declare:
- `encode(const 像素缓冲区 &image, int quality) -> EncodeResult` — encode a single image at given quality, return byte array + actual size
- `supportsFormat(format: ImageFormat) -> bool` — whether this strategy handles the given format
- `qualityRange() -> (u8, u8)` — min/max quality values for binary search
- `name() -> &str` — human-readable strategy name

Each strategy MUST be independently testable without QML or UI dependencies.

#### Scenario: Strategy interface contract
- **WHEN** a new format encoder is implemented
- **THEN** it MUST inherit `ICompressStrategy` and implement all four pure virtual methods

#### Scenario: Strategy registration and lookup
- **WHEN** the compress orchestrator needs to compress an image of format "JPEG"
- **THEN** it SHALL iterate registered strategies, call `supportsFormat("JPEG")`, and select the first matching strategy

### Requirement: MozJPEG strategy for JPEG

The system SHALL provide a `MozJpegStrategy` class that links against the MozJPEG C library (compiled as a static library from `third_party/mozjpeg/`). It SHALL encode 像素缓冲区 data to JPEG format using MozJPEG's `jpeg_write_scanlines` API. The quality parameter SHALL map directly to MozJPEG's quality setting (0-100).

#### Scenario: JPEG encoding produces valid output
- **WHEN** `MozJpegStrategy::encode()` is called with a valid 像素缓冲区 and quality 80
- **THEN** the returned byte array SHALL be a valid JPEG that can be decoded by `像素缓冲区Reader`

#### Scenario: MozJPEG compression ratio improvement
- **WHEN** the same image is encoded at the same quality by MozJPEG vs Qt default JPEG
- **THEN** MozJPEG output size SHALL be smaller (typically 20-40% reduction)

### Requirement: pngquant strategy for PNG

The system SHALL provide a `PngQuantStrategy` class that uses the external `pngquant.exe` tool (located in `tools/` directory) via `ExternalProcess`. The strategy SHALL:
1. Write the input 像素缓冲区 to a temporary PNG buffer
2. Invoke `pngquant --quality=min-max --speed=1 --force -` with stdin/stdout piping
3. Read the quantized PNG from stdout
4. Return the quantized result

The quality parameter SHALL control pngquant's color count (quality 10 → 16 colors, quality 95 → 256 colors).

#### Scenario: PNG compression via pngquant
- **WHEN** `PngQuantStrategy::encode()` is called with a valid 像素缓冲区
- **THEN** the returned byte array SHALL be a valid PNG with reduced color palette

#### Scenario: pngquant not found
- **WHEN** `pngquant.exe` is not present in the `tools/` directory
- **THEN** the strategy SHALL report an error via the `failed` signal with a descriptive message

### Requirement: gifsicle strategy for GIF

The system SHALL provide a `GifsicleStrategy` class that uses the external `gifsicle.exe` tool (located in `tools/` directory) via `ExternalProcess`. It SHALL handle both static and animated GIF compression. For static GIFs, it SHALL invoke `gifsicle --lossy=N -O3`. For animated GIFs, it SHALL support frame dropping and lossy optimization.

#### Scenario: Static GIF compression
- **WHEN** `GifsicleStrategy::encode()` is called with a single-frame GIF
- **THEN** gifsicle SHALL optimize the GIF with lossy compression and output a valid GIF

#### Scenario: Animated GIF compression with frame drop
- **WHEN** `GifsicleStrategy` compresses an animated GIF with frame dropping enabled
- **THEN** the output SHALL have fewer frames (every other frame dropped) and be optimized with `--lossy`

### Requirement: WebP strategy

The system SHALL provide a WebP strategy using a native encoder with adjustable lossy quality.

#### Scenario: WebP encoding
- **WHEN** `WebpStrategy::encode()` is called with a valid 像素缓冲区 and quality 80
- **THEN** the returned byte array SHALL be a valid WebP image

### Requirement: ExternalProcess utility

The system SHALL provide an `ExternalProcess` class in `src/core/compress/externalprocess.h` that wraps `Rust 进程管理` for invoking external command-line tools. It SHALL support:
- Running a tool with given arguments and stdin data
- Capturing stdout as `字节缓冲区`
- Timeout handling (configurable, default 30s)
- Synchronous execution (blocking call from background thread)
- Error reporting (tool not found, crashed, timeout)

#### Scenario: External tool invocation with stdin/stdout
- **WHEN** `ExternalProcess::run()` is called with tool path, arguments, and stdin data
- **THEN** it SHALL execute the tool, pipe stdin, capture stdout, and return the result

#### Scenario: Tool not found
- **WHEN** the specified executable does not exist
- **THEN** `ExternalProcess` SHALL return an error result with `ErrorReason::ToolNotFound`

#### Scenario: Tool timeout
- **WHEN** the external tool does not complete within the timeout period
- **THEN** `ExternalProcess` SHALL kill the process and return an error result with `ErrorReason::Timeout`

### Requirement: CompressOrchestrator replaces CompressIterator

The system SHALL provide a `CompressOrchestrator` class that replaces the current `CompressIterator`. It SHALL:
1. Select the appropriate strategy based on input format
2. Drive the binary search loop (quality → resolution scaling) using the strategy's `encode()` and `qualityRange()`
3. Emit progress and finished signals
4. Support cancellation

The orchestrator SHALL NOT contain any format-specific encoding logic.

#### Scenario: Format-based strategy selection
- **WHEN** orchestrator receives a JPEG image for compression
- **THEN** it SHALL select `MozJpegStrategy` and use its quality range for binary search

#### Scenario: Resolution fallback
- **WHEN** quality binary search exhausts without meeting target size
- **THEN** the orchestrator SHALL progressively downscale resolution (halving) until target is met or long edge < 16px

### Requirement: Animated compression uses gifsicle for GIF

The system SHALL replace `AnimatedCompressor` with an `AnimatedCompressOrchestrator` that delegates GIF encoding to `GifsicleStrategy`. The orchestration phases SHALL be:
1. Encode all frames at given quality via gifsicle with `--lossy`
2. If still over target: reduce quality in steps
3. If still over target: downscale resolution (GIF uses 0.75 scale factor)

Note: Frame dropping is currently only supported for animated WebP output; GIF animation does not drop frames.

#### Scenario: Animated GIF compression pipeline
- **WHEN** an animated GIF is compressed with target size 500KB
- **THEN** the orchestrator SHALL use gifsicle for all encoding phases and produce a valid animated GIF
