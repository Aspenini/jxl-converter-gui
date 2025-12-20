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
- **Responsive UI**: Background threading keeps the interface responsive

## Requirements

### cjxl Binary

The app requires the `cjxl` command-line tool from the JPEG XL reference implementation.

**Option 1: Local tools folder (Recommended)**
Place the cjxl binary in a `tools` folder next to the executable:
- Windows: `tools/cjxl.exe`
- macOS/Linux: `tools/cjxl`

**Option 2: System PATH**
Install cjxl system-wide and ensure it's available in PATH.

### Getting cjxl

Download from: https://github.com/libjxl/libjxl/releases

Or build from source: https://github.com/libjxl/libjxl

## Building

### Prerequisites

- Rust 1.70 or newer
- Cargo (comes with Rust)

### Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run --release
```

The compiled executable will be in:
- Debug: `target/debug/jxl-converter` (or `.exe` on Windows)
- Release: `target/release/jxl-converter` (or `.exe` on Windows)

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

## Project Structure

```
jxl-converter/
├── src/
│   ├── main.rs      # Entry point and eframe bootstrap
│   ├── app.rs       # GUI implementation and state management
│   ├── engine.rs    # Conversion logic and cjxl interfacing
│   └── types.rs     # Data structures and enums
├── Cargo.toml       # Dependencies and build configuration
└── README.md        # This file
```

## Technical Details

### Architecture

- **Frontend**: egui/eframe for cross-platform GUI
- **Backend**: Spawns cjxl processes for actual conversion
- **Threading**: Conversions run on background threads
- **Communication**: mpsc channels for progress updates

### Platform-Specific Notes

**Windows**
- Looks for `tools/cjxl.exe`
- Uses `where` command to search PATH

**macOS/Linux**
- Looks for `tools/cjxl`
- Automatically sets executable permissions
- Uses `which` command to search PATH

### Dependencies

- `eframe` / `egui` 0.29 - GUI framework
- `walkdir` 2.4 - Recursive directory traversal
- `rfd` 0.15 - Native file dialogs

## License

This project is provided as-is for educational and practical use.

## Troubleshooting

**"cjxl executable not found" error**
- Ensure cjxl is in the `tools` folder or system PATH
- On Unix systems, verify the binary has execute permissions: `chmod +x tools/cjxl`

**UI not responding during conversion**
- This shouldn't happen as conversions run on a background thread
- If it does, please report the issue

**Conversion fails for specific files**
- Check the log for error messages from cjxl
- Verify the input file is a valid image
- Try converting the file manually with cjxl to see detailed errors

