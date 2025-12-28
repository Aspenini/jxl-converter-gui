# JPEG XL Converter

Cross-platform GUI for converting images to and from JPEG XL format (Rust + egui).

## Features

- **Encode to JXL**: Batch convert JPEG, PNG, GIF, BMP, TIFF, WebP, PNM to JXL
  - JPEG lossless mode (enabled by default) or quality settings (1-100)
  - Effort control (1-9) and command preview
- **Decode from JXL**: Convert to PNG, JPEG, PPM, PGM, or PBM
  - Global or per-file format selection
- **Drag & drop** files/folders, recursive scanning, folder structure preservation
- **Real-time progress** with cancellation support

## Requirements

Requires `cjxl` and `djxl` binaries from libjxl:
- Place in `tools/` folder next to executable (recommended), or
- Install system-wide in PATH

## Usage

### Encode Tab
1. Add files/folders (drag & drop or buttons)
2. Choose output directory
3. Configure options: JPEG lossless, quality (1-100), effort (1-9)
4. Click "Start Encoding"

### Decode Tab
1. Add JXL files/folders
2. Choose output directory
3. Select output format (PNG, JPEG, etc.)
4. Optionally customize individual file formats
5. Click "Start Decoding"
