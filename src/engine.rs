use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::Sender;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

use crate::types::{ConversionSettings, DecodeSettings, DecodeItem, OutputFormat, ProgressMessage};

pub struct ConversionEngine {
    cjxl_path: Option<PathBuf>,
    djxl_path: Option<PathBuf>,
}

impl ConversionEngine {
    pub fn new() -> Self {
        let cjxl_path = Self::find_cjxl();
        let djxl_path = Self::find_djxl();
        Self { cjxl_path, djxl_path }
    }

    pub fn is_available(&self) -> bool {
        self.cjxl_path.is_some()
    }

    pub fn is_decode_available(&self) -> bool {
        self.djxl_path.is_some()
    }

    pub fn get_error(&self) -> Option<String> {
        if self.cjxl_path.is_none() {
            Some("cjxl executable not found. Please place it in the 'tools' folder or ensure it's in PATH.".to_string())
        } else {
            None
        }
    }

    pub fn get_decode_error(&self) -> Option<String> {
        if self.djxl_path.is_none() {
            Some("djxl executable not found. Please place it in the 'tools' folder or ensure it's in PATH.".to_string())
        } else {
            None
        }
    }

    fn find_cjxl() -> Option<PathBuf> {
        Self::find_tool("cjxl")
    }

    fn find_djxl() -> Option<PathBuf> {
        Self::find_tool("djxl")
    }

    fn find_tool(tool_name: &str) -> Option<PathBuf> {
        // First, try tools folder relative to executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let tool_path = if cfg!(windows) {
                    exe_dir.join("tools").join(format!("{}.exe", tool_name))
                } else {
                    exe_dir.join("tools").join(tool_name)
                };

                if tool_path.exists() {
                    // Ensure executable permission on Unix
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(metadata) = std::fs::metadata(&tool_path) {
                            let mut perms = metadata.permissions();
                            perms.set_mode(0o755);
                            let _ = std::fs::set_permissions(&tool_path, perms);
                        }
                    }
                    return Some(tool_path);
                }
            }
        }

        // Fall back to PATH
        let tool_exe = if cfg!(windows) { 
            format!("{}.exe", tool_name)
        } else { 
            tool_name.to_string()
        };
        
        if let Ok(output) = Command::new(if cfg!(windows) { "where" } else { "which" })
            .arg(&tool_exe)
            .output()
        {
            if output.status.success() {
                if let Ok(path_str) = String::from_utf8(output.stdout) {
                    let path = PathBuf::from(path_str.trim());
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }

        None
    }

    pub fn convert_batch(
        &self,
        input_paths: Vec<PathBuf>,
        settings: ConversionSettings,
        progress_tx: Sender<ProgressMessage>,
        cancel_flag: Arc<AtomicBool>,
    ) {
        let cjxl_path = match &self.cjxl_path {
            Some(p) => p.clone(),
            None => {
                let _ = progress_tx.send(ProgressMessage::Error {
                    file: String::new(),
                    error: "cjxl not found".to_string(),
                });
                return;
            }
        };

        // Expand all input paths to individual files
        let files = self.expand_paths(&input_paths, settings.recursive);

        // Filter for supported image formats
        let image_files: Vec<PathBuf> = files
            .into_iter()
            .filter(|p| self.is_supported_image(p))
            .collect();

        let total = image_files.len();
        let _ = progress_tx.send(ProgressMessage::Started { total });

        if total == 0 {
            let _ = progress_tx.send(ProgressMessage::Completed);
            return;
        }

        // Find common base path for structure preservation
        let base_path = if settings.keep_structure {
            self.find_common_base(&input_paths)
        } else {
            None
        };

        for (idx, input_file) in image_files.iter().enumerate() {
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = progress_tx.send(ProgressMessage::Cancelled);
                return;
            }

            let _ = progress_tx.send(ProgressMessage::Progress {
                current: idx + 1,
                total,
                file: input_file.display().to_string(),
            });

            match self.convert_single(
                &cjxl_path,
                input_file,
                &settings,
                base_path.as_ref(),
            ) {
                Ok(output) => {
                    let _ = progress_tx.send(ProgressMessage::Success {
                        file: format!("{} -> {}", input_file.display(), output.display()),
                    });
                }
                Err(e) => {
                    let _ = progress_tx.send(ProgressMessage::Error {
                        file: input_file.display().to_string(),
                        error: e,
                    });
                }
            }
        }

        let _ = progress_tx.send(ProgressMessage::Completed);
    }

    fn expand_paths(&self, paths: &[PathBuf], recursive: bool) -> Vec<PathBuf> {
        let mut result = Vec::new();

        for path in paths {
            if path.is_file() {
                result.push(path.clone());
            } else if path.is_dir() {
                if recursive {
                    for entry in WalkDir::new(path)
                        .follow_links(false)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_type().is_file() {
                            result.push(entry.path().to_path_buf());
                        }
                    }
                } else {
                    if let Ok(entries) = std::fs::read_dir(path) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                                result.push(entry.path());
                            }
                        }
                    }
                }
            }
        }

        result
    }

    fn is_supported_image(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            matches!(
                ext_lower.as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" | "ppm" | "pgm" | "pnm"
            )
        } else {
            false
        }
    }

    fn find_common_base(&self, paths: &[PathBuf]) -> Option<PathBuf> {
        if paths.is_empty() {
            return None;
        }

        // Get the first path's parent (if it's a file) or itself (if it's a dir)
        let mut base = if paths[0].is_file() {
            paths[0].parent()?.to_path_buf()
        } else {
            paths[0].clone()
        };

        // Find the common ancestor
        for path in &paths[1..] {
            let check_path = if path.is_file() {
                path.parent()?
            } else {
                path.as_path()
            };

            while !check_path.starts_with(&base) {
                base = base.parent()?.to_path_buf();
            }
        }

        Some(base)
    }

    fn convert_single(
        &self,
        cjxl_path: &Path,
        input_file: &Path,
        settings: &ConversionSettings,
        base_path: Option<&PathBuf>,
    ) -> Result<PathBuf, String> {
        // Determine output path
        let output_path = if settings.keep_structure {
            if let Some(base) = base_path {
                if let Ok(rel_path) = input_file.strip_prefix(base) {
                    settings.output_dir.join(rel_path)
                } else {
                    settings.output_dir.join(input_file.file_name().unwrap())
                }
            } else {
                settings.output_dir.join(input_file.file_name().unwrap())
            }
        } else {
            settings.output_dir.join(input_file.file_name().unwrap())
        };

        // Change extension to .jxl
        let output_path = output_path.with_extension("jxl");

        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }

        // Build cjxl command
        let mut cmd = Command::new(cjxl_path);
        
        // Use absolute paths
        let abs_input = std::fs::canonicalize(input_file)
            .map_err(|e| format!("Failed to resolve input path: {}", e))?;
        let abs_output = if output_path.exists() {
            std::fs::canonicalize(&output_path)
                .map_err(|e| format!("Failed to resolve output path: {}", e))?
        } else {
            // For non-existent paths, resolve parent and join filename
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create output directory: {}", e))?;
                let abs_parent = std::fs::canonicalize(parent)
                    .map_err(|e| format!("Failed to resolve output directory: {}", e))?;
                abs_parent.join(output_path.file_name().unwrap())
            } else {
                output_path.clone()
            }
        };

        cmd.arg(&abs_input);
        cmd.arg(&abs_output);

        // Add quality/lossless options
        let ext = input_file.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let is_jpeg = ext == "jpg" || ext == "jpeg";
        
        if settings.lossless {
            if is_jpeg {
                cmd.arg("--lossless_jpeg=1");
            } else {
                cmd.arg("-d").arg("0");
            }
        } else if is_jpeg && settings.jpeg_lossless {
            // JPEG-specific lossless conversion
            cmd.arg("--lossless_jpeg=1");
        } else {
            cmd.arg("-q").arg(settings.quality.to_string());
        }

        // Add effort option
        cmd.arg("-e").arg(settings.effort.to_string());

        // Execute
        let output = cmd.output()
            .map_err(|e| format!("Failed to execute cjxl: {}", e))?;

        if output.status.success() {
            Ok(abs_output)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("cjxl failed: {}", stderr))
        }
    }

    pub fn decode_batch(
        &self,
        decode_items: Vec<DecodeItem>,
        settings: DecodeSettings,
        progress_tx: Sender<ProgressMessage>,
        cancel_flag: Arc<AtomicBool>,
    ) {
        let djxl_path = match &self.djxl_path {
            Some(p) => p.clone(),
            None => {
                let _ = progress_tx.send(ProgressMessage::Error {
                    file: String::new(),
                    error: "djxl not found".to_string(),
                });
                return;
            }
        };

        let total = decode_items.len();
        let _ = progress_tx.send(ProgressMessage::Started { total });

        if total == 0 {
            let _ = progress_tx.send(ProgressMessage::Completed);
            return;
        }

        // Find common base path for structure preservation
        let base_path = if settings.keep_structure {
            let paths: Vec<PathBuf> = decode_items.iter().map(|item| item.path.clone()).collect();
            self.find_common_base(&paths)
        } else {
            None
        };

        for (idx, item) in decode_items.iter().enumerate() {
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = progress_tx.send(ProgressMessage::Cancelled);
                return;
            }

            let _ = progress_tx.send(ProgressMessage::Progress {
                current: idx + 1,
                total,
                file: item.path.display().to_string(),
            });

            match self.decode_single(
                &djxl_path,
                &item.path,
                item.output_format,
                &settings,
                base_path.as_ref(),
            ) {
                Ok(output) => {
                    let _ = progress_tx.send(ProgressMessage::Success {
                        file: format!("{} -> {}", item.path.display(), output.display()),
                    });
                }
                Err(e) => {
                    let _ = progress_tx.send(ProgressMessage::Error {
                        file: item.path.display().to_string(),
                        error: e,
                    });
                }
            }
        }

        let _ = progress_tx.send(ProgressMessage::Completed);
    }

    fn decode_single(
        &self,
        djxl_path: &Path,
        input_file: &Path,
        output_format: OutputFormat,
        settings: &DecodeSettings,
        base_path: Option<&PathBuf>,
    ) -> Result<PathBuf, String> {
        // Determine output path
        let output_path = if settings.keep_structure {
            if let Some(base) = base_path {
                if let Ok(rel_path) = input_file.strip_prefix(base) {
                    settings.output_dir.join(rel_path)
                } else {
                    settings.output_dir.join(input_file.file_name().unwrap())
                }
            } else {
                settings.output_dir.join(input_file.file_name().unwrap())
            }
        } else {
            settings.output_dir.join(input_file.file_name().unwrap())
        };

        // Change extension to the selected output format
        let output_path = output_path.with_extension(output_format.extension());

        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }

        // Build djxl command
        let mut cmd = Command::new(djxl_path);
        
        // Use absolute paths
        let abs_input = std::fs::canonicalize(input_file)
            .map_err(|e| format!("Failed to resolve input path: {}", e))?;
        let abs_output = if output_path.exists() {
            std::fs::canonicalize(&output_path)
                .map_err(|e| format!("Failed to resolve output path: {}", e))?
        } else {
            // For non-existent paths, resolve parent and join filename
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create output directory: {}", e))?;
                let abs_parent = std::fs::canonicalize(parent)
                    .map_err(|e| format!("Failed to resolve output directory: {}", e))?;
                abs_parent.join(output_path.file_name().unwrap())
            } else {
                output_path.clone()
            }
        };

        cmd.arg(&abs_input);
        cmd.arg(&abs_output);

        // Execute
        let output = cmd.output()
            .map_err(|e| format!("Failed to execute djxl: {}", e))?;

        if output.status.success() {
            Ok(abs_output)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("djxl failed: {}", stderr))
        }
    }
}

