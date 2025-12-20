# JPEG XL Converter

A cross-platform desktop GUI application for converting images to JPEG XL format using Rust and egui.

## Features

- **Drag & Drop Support**: Drag files and folders directly into the app
- **Native File Dialogs**: Browse for files and folders using system dialogs
- **Batch Processing**: Convert multiple files at once
- **Recursive Scanning**: Optionally scan subfolders
- **Folder Structure Preservation**: Keep the original folder structure in output
- **Lossless Conversion**: Special handling for JPEG lossless compression
- **Quality Control**: Adjustable quality (1-100) and effort (1-9) settings
- **Real-time Progress**: Live progress bar and detailed logging
- **Cancellable Operations**: Stop conversions at any time

## Requirements

### cjxl Binary

The app requires the `cjxl` command-line tool from the JPEG XL reference implementation.

**Option 1: Local tools folder (Recommended)**
Release comes with cjxl binary in `tools` folder next to the app executable:
- Windows: `tools/cjxl.exe`
- macOS/Linux: `tools/cjxl`

**Option 2: System PATH**
Install cjxl system-wide and ensure it's available in PATH.

## Usage

1. **Launch the Application**
   ```bash
   cargo run --release
   ```

2. **Add Input Files/Folders**
   - Drag and drop files or folders into the drop area
   - Or click "Add Files" / "Add Folder" buttons
   - Mix files and folders as needed
   - Toggle "Recursive" to scan subfolders

3. **Select Output Directory**
   - Click "Browse" to choose where converted files will be saved
   - Toggle "Keep input folder structure" to preserve directory hierarchy

4. **Configure Conversion Options**
   - **Lossless**: Enable for lossless compression
     - For JPEG inputs: uses `--lossless_jpeg=1`
     - For other formats: uses `-d 0`
   - **Quality**: 1-100 (disabled when lossless is on)
   - **Effort**: 1-9 (higher = slower but better compression)

5. **Start Conversion**
   - Click "Start Conversion"
   - Monitor progress in the log area
   - Click "Cancel" to stop if needed

## Supported Input Formats

- JPEG (.jpg, .jpeg)
- PNG (.png)
- GIF (.gif)
- BMP (.bmp)
- TIFF (.tiff, .tif)
- WebP (.webp)
- PNM formats (.ppm, .pgm, .pnm)
