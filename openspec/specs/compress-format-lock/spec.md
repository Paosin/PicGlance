# compress-format-lock Specification

## Purpose
TBD - created by archiving change picglance-compression. Update Purpose after archive.

## Requirements

### Requirement: Default format preservation for single image compression

When compressing a single (non-animated) image, the system SHALL default the output format to match the source file's format. The output format selector in `CompressDialog` SHALL be pre-selected to the source format and visually indicate it as the recommended default.

#### Scenario: JPEG source defaults to JPEG output
- **WHEN** user opens compression dialog for a `.jpg` file
- **THEN** the output format combo box SHALL default to "JPEG"

#### Scenario: PNG source defaults to PNG output
- **WHEN** user opens compression dialog for a `.png` file
- **THEN** the output format combo box SHALL default to "PNG"

#### Scenario: GIF source defaults to GIF output
- **WHEN** user opens compression dialog for a `.gif` file
- **THEN** the output format combo box SHALL default to "GIF"

#### Scenario: WebP source defaults to WebP output
- **WHEN** user opens compression dialog for a `.webp` file
- **THEN** the output format combo box SHALL default to "WebP"

### Requirement: Format selection remains available

The system SHALL retain the output format selection UI (ComboBox) in `CompressDialog`, allowing users to choose a different output format if desired. The available output formats for a given source file SHALL remain:
- GIF source: GIF, WebP
- Other sources (JPEG, PNG, WebP, BMP): JPEG, PNG, WebP

#### Scenario: User changes output format
- **WHEN** user opens compression for a JPEG file and changes output format to PNG
- **THEN** the system SHALL compress and output a PNG file

#### Scenario: Format list for GIF source
- **WHEN** user opens compression for a GIF file
- **THEN** the format selector SHALL offer "GIF" and "WebP" options, with "GIF" pre-selected

### Requirement: Animated image format preservation

When compressing an animated image, the system SHALL restrict output formats to animation-capable formats only. Animated GIF SHALL default to GIF output; animated WebP SHALL default to WebP output.

#### Scenario: Animated GIF compression preserves format
- **WHEN** user compresses an animated GIF without changing format
- **THEN** the output SHALL be a GIF file with preserved animation

#### Scenario: Animated WebP compression preserves format
- **WHEN** user compresses an animated WebP without changing format
- **THEN** the output SHALL be a WebP file with preserved animation
