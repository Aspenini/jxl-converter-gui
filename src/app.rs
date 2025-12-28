use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use egui::{Color32, RichText, Slider, ScrollArea};

use crate::engine::ConversionEngine;
use crate::types::{ConversionSettings, DecodeSettings, DecodeItem, LogEntry, OutputFormat, ProgressMessage};

pub struct JxlConverterApp {
    engine: ConversionEngine,
    
    // Encode tab
    settings: ConversionSettings,
    input_paths: Vec<PathBuf>,
    
    // Decode tab
    decode_settings: DecodeSettings,
    decode_items: Vec<DecodeItem>,
    
    // Shared conversion state
    is_converting: bool,
    cancel_flag: Arc<AtomicBool>,
    progress_rx: Option<Receiver<ProgressMessage>>,
    current_progress: usize,
    total_files: usize,
    current_file: String,
    
    // UI state
    active_tab: AppTab,
    log_entries: Vec<LogEntry>,
    scroll_to_bottom: bool,
}

#[derive(PartialEq)]
enum AppTab {
    Encode,
    Decode,
}

impl JxlConverterApp {
    pub fn new() -> Self {
        let engine = ConversionEngine::new();
        
        let mut app = Self {
            engine,
            settings: ConversionSettings::default(),
            input_paths: Vec::new(),
            decode_settings: DecodeSettings::default(),
            decode_items: Vec::new(),
            is_converting: false,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            progress_rx: None,
            current_progress: 0,
            total_files: 0,
            current_file: String::new(),
            active_tab: AppTab::Encode,
            log_entries: Vec::new(),
            scroll_to_bottom: false,
        };

        // Check if cjxl is available
        if let Some(error) = app.engine.get_error() {
            app.log_entries.push(LogEntry::Error(error));
        } else {
            app.log_entries.push(LogEntry::Info("cjxl found and ready.".to_string()));
        }

        // Check if djxl is available
        if let Some(error) = app.engine.get_decode_error() {
            app.log_entries.push(LogEntry::Error(error));
        } else {
            app.log_entries.push(LogEntry::Info("djxl found and ready.".to_string()));
        }

        app
    }

    fn add_log(&mut self, entry: LogEntry) {
        self.log_entries.push(entry);
        self.scroll_to_bottom = true;
    }

    fn start_conversion(&mut self) {
        if !self.engine.is_available() {
            self.add_log(LogEntry::Error("cjxl is not available.".to_string()));
            return;
        }

        if self.input_paths.is_empty() {
            self.add_log(LogEntry::Warning("No input files or folders selected.".to_string()));
            return;
        }

        if self.settings.output_dir.as_os_str().is_empty() {
            self.add_log(LogEntry::Warning("No output directory selected.".to_string()));
            return;
        }

        self.is_converting = true;
        self.cancel_flag.store(false, Ordering::Relaxed);
        self.current_progress = 0;
        self.total_files = 0;
        self.current_file.clear();

        let (tx, rx) = channel();
        self.progress_rx = Some(rx);

        let engine = ConversionEngine::new();
        let input_paths = self.input_paths.clone();
        let settings = self.settings.clone();
        let cancel_flag = Arc::clone(&self.cancel_flag);

        thread::spawn(move || {
            engine.convert_batch(input_paths, settings, tx, cancel_flag);
        });

        self.add_log(LogEntry::Info("Conversion started...".to_string()));
    }

    fn cancel_conversion(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        self.add_log(LogEntry::Warning("Cancelling conversion...".to_string()));
    }

    fn start_decode(&mut self) {
        if !self.engine.is_decode_available() {
            self.add_log(LogEntry::Error("djxl is not available.".to_string()));
            return;
        }

        if self.decode_items.is_empty() {
            self.add_log(LogEntry::Warning("No JXL files selected.".to_string()));
            return;
        }

        if self.decode_settings.output_dir.as_os_str().is_empty() {
            self.add_log(LogEntry::Warning("No output directory selected.".to_string()));
            return;
        }

        self.is_converting = true;
        self.cancel_flag.store(false, Ordering::Relaxed);
        self.current_progress = 0;
        self.total_files = 0;
        self.current_file.clear();

        let (tx, rx) = channel();
        self.progress_rx = Some(rx);

        let engine = ConversionEngine::new();
        let decode_items = self.decode_items.clone();
        let settings = self.decode_settings.clone();
        let cancel_flag = Arc::clone(&self.cancel_flag);

        thread::spawn(move || {
            engine.decode_batch(decode_items, settings, tx, cancel_flag);
        });

        self.add_log(LogEntry::Info("Decoding started...".to_string()));
    }

    fn process_progress_messages(&mut self) {
        // Collect all messages first to avoid borrow checker issues
        let mut messages = Vec::new();
        if let Some(rx) = &self.progress_rx {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }

        // Process collected messages
        for msg in messages {
            match msg {
                ProgressMessage::Started { total } => {
                    self.total_files = total;
                    self.add_log(LogEntry::Info(format!("Processing {} file(s)...", total)));
                }
                ProgressMessage::Progress { current, total, file } => {
                    self.current_progress = current;
                    self.total_files = total;
                    self.current_file = file;
                }
                ProgressMessage::Success { file } => {
                    self.add_log(LogEntry::Success(format!("✓ {}", file)));
                }
                ProgressMessage::Error { file, error } => {
                    self.add_log(LogEntry::Error(format!("✗ {}: {}", file, error)));
                }
                ProgressMessage::Skipped { file, reason } => {
                    self.add_log(LogEntry::Warning(format!("⊘ {}: {}", file, reason)));
                }
                ProgressMessage::Completed => {
                    self.is_converting = false;
                    self.progress_rx = None;
                    self.current_file.clear();
                    self.add_log(LogEntry::Info("Conversion completed.".to_string()));
                }
                ProgressMessage::Cancelled => {
                    self.is_converting = false;
                    self.progress_rx = None;
                    self.current_file.clear();
                    self.add_log(LogEntry::Warning("Conversion cancelled.".to_string()));
                }
            }
        }
    }

    fn render_input_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Input");
        ui.add_space(5.0);

        // Drop area
        let drop_area = ui.allocate_response(
            egui::vec2(ui.available_width(), 100.0),
            egui::Sense::click(),
        );

        ui.painter().rect_filled(
            drop_area.rect,
            4.0,
            if drop_area.hovered() {
                Color32::from_rgb(60, 60, 80)
            } else {
                Color32::from_rgb(40, 40, 60)
            },
        );

        ui.painter().rect_stroke(
            drop_area.rect,
            4.0,
            egui::Stroke::new(2.0, Color32::from_rgb(100, 100, 120)),
        );

        let text = if self.input_paths.is_empty() {
            "Drop files or folders here\nor use the buttons below"
        } else {
            &format!("{} item(s) selected", self.input_paths.len())
        };

        ui.put(
            drop_area.rect,
            egui::Label::new(RichText::new(text).size(14.0).color(Color32::LIGHT_GRAY)),
        );

        // Handle drag and drop
        ui.ctx().input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        if !self.input_paths.contains(path) {
                            self.input_paths.push(path.clone());
                        }
                    }
                }
            }
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("📁 Add Files").clicked() {
                if let Some(files) = rfd::FileDialog::new()
                    .set_title("Select Image Files")
                    .add_filter("Images", &["jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp", "ppm", "pgm", "pnm"])
                    .pick_files()
                {
                    for file in files {
                        if !self.input_paths.contains(&file) {
                            self.input_paths.push(file);
                        }
                    }
                }
            }

            if ui.button("📂 Add Folder").clicked() {
                if let Some(folder) = rfd::FileDialog::new()
                    .set_title("Select Folder")
                    .pick_folder()
                {
                    if !self.input_paths.contains(&folder) {
                        self.input_paths.push(folder);
                    }
                }
            }

            if ui.button("Clear").clicked() {
                self.input_paths.clear();
            }
        });

        ui.add_space(5.0);
        ui.checkbox(&mut self.settings.recursive, "Recursive (scan subfolders)");
    }

    fn render_output_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Output");
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Directory:");
            ui.add(
                egui::TextEdit::singleline(&mut self.settings.output_dir.to_string_lossy().to_string())
                    .desired_width(ui.available_width() - 80.0)
                    .interactive(false),
            );
            if ui.button("Browse").clicked() {
                if let Some(folder) = rfd::FileDialog::new()
                    .set_title("Select Output Directory")
                    .pick_folder()
                {
                    self.settings.output_dir = folder;
                }
            }
        });

        ui.add_space(5.0);
        ui.checkbox(&mut self.settings.keep_structure, "Keep input folder structure");
    }

    fn render_options_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Conversion Options");
        ui.add_space(5.0);

        ui.checkbox(&mut self.settings.lossless, "Lossless (all formats)");
        ui.add_space(3.0);
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.settings.jpeg_lossless, "JPEG Lossless");
            ui.label(RichText::new("(uses --lossless_jpeg=1)").small().color(Color32::GRAY));
        });
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Quality:");
            ui.add_enabled(
                !self.settings.lossless,
                Slider::new(&mut self.settings.quality, 1..=100),
            );
        });

        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Effort:");
            ui.add(Slider::new(&mut self.settings.effort, 1..=9));
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(5.0);

        // Command preview
        ui.label(RichText::new("Command Preview:").strong());
        ui.add_space(3.0);
        
        // JPEG example
        let jpeg_cmd = self.generate_command_preview(true);
        ui.label(RichText::new("JPEG files:").small().color(Color32::LIGHT_GRAY));
        ui.add(
            egui::TextEdit::multiline(&mut jpeg_cmd.as_str())
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(1)
                .interactive(false)
                .frame(true),
        );
        
        ui.add_space(3.0);
        
        // Non-JPEG example
        let other_cmd = self.generate_command_preview(false);
        ui.label(RichText::new("Other formats:").small().color(Color32::LIGHT_GRAY));
        ui.add(
            egui::TextEdit::multiline(&mut other_cmd.as_str())
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(1)
                .interactive(false)
                .frame(true),
        );
    }

    fn generate_command_preview(&self, is_jpeg: bool) -> String {
        let mut cmd_parts = vec!["cjxl".to_string()];
        
        // Add quality/lossless options
        if self.settings.lossless {
            if is_jpeg {
                cmd_parts.push("--lossless_jpeg=1".to_string());
            } else {
                cmd_parts.push("-d".to_string());
                cmd_parts.push("0".to_string());
            }
        } else if is_jpeg && self.settings.jpeg_lossless {
            cmd_parts.push("--lossless_jpeg=1".to_string());
        } else {
            cmd_parts.push("-q".to_string());
            cmd_parts.push(self.settings.quality.to_string());
        }
        
        // Add effort option
        cmd_parts.push("-e".to_string());
        cmd_parts.push(self.settings.effort.to_string());
        
        // Add placeholder paths
        if is_jpeg {
            cmd_parts.push("input.jpg".to_string());
            cmd_parts.push("output.jxl".to_string());
        } else {
            cmd_parts.push("input.png".to_string());
            cmd_parts.push("output.jxl".to_string());
        }
        
        cmd_parts.join(" ")
    }

    fn render_controls_section(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            let (can_start, button_text) = match self.active_tab {
                AppTab::Encode => {
                    let can_start = !self.is_converting 
                        && self.engine.is_available() 
                        && !self.input_paths.is_empty()
                        && !self.settings.output_dir.as_os_str().is_empty();
                    (can_start, "▶ Start Encoding")
                }
                AppTab::Decode => {
                    let can_start = !self.is_converting 
                        && self.engine.is_decode_available() 
                        && !self.decode_items.is_empty()
                        && !self.decode_settings.output_dir.as_os_str().is_empty();
                    (can_start, "▶ Start Decoding")
                }
            };

            if ui.add_enabled(can_start, egui::Button::new(button_text)).clicked() {
                match self.active_tab {
                    AppTab::Encode => self.start_conversion(),
                    AppTab::Decode => self.start_decode(),
                }
            }

            if ui.add_enabled(self.is_converting, egui::Button::new("⬛ Cancel")).clicked() {
                self.cancel_conversion();
            }
        });

        if self.is_converting {
            ui.add_space(10.0);
            let progress = if self.total_files > 0 {
                self.current_progress as f32 / self.total_files as f32
            } else {
                0.0
            };

            ui.add(egui::ProgressBar::new(progress).text(format!(
                "{} / {}",
                self.current_progress, self.total_files
            )));

            if !self.current_file.is_empty() {
                ui.label(RichText::new(&self.current_file).small().italics());
            }
        }
    }

    fn render_log_section(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(5.0);
        ui.heading("Log");
        ui.add_space(5.0);

        let scroll_area = ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true);

        scroll_area.show(ui, |ui| {
            for entry in &self.log_entries {
                let (color, text) = match entry {
                    LogEntry::Info(s) => (Color32::LIGHT_GRAY, s),
                    LogEntry::Success(s) => (Color32::from_rgb(100, 255, 100), s),
                    LogEntry::Error(s) => (Color32::from_rgb(255, 100, 100), s),
                    LogEntry::Warning(s) => (Color32::from_rgb(255, 200, 100), s),
                };

                ui.label(RichText::new(text).color(color).small());
            }

            if self.scroll_to_bottom {
                ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                self.scroll_to_bottom = false;
            }
        });
    }

    fn render_decode_input_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Input JXL Files");
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            if ui.button("📁 Add JXL Files").clicked() {
                if let Some(files) = rfd::FileDialog::new()
                    .set_title("Select JXL Files")
                    .add_filter("JPEG XL", &["jxl"])
                    .pick_files()
                {
                    for file in files {
                        if !self.decode_items.iter().any(|item| item.path == file) {
                            self.decode_items.push(DecodeItem {
                                path: file,
                                output_format: self.decode_settings.output_format,
                            });
                        }
                    }
                }
            }

            if ui.button("📂 Add Folder").clicked() {
                if let Some(folder) = rfd::FileDialog::new()
                    .set_title("Select Folder")
                    .pick_folder()
                {
                    self.add_jxl_files_from_folder(&folder);
                }
            }

            if ui.button("Clear").clicked() {
                self.decode_items.clear();
            }
        });

        ui.add_space(5.0);
        ui.checkbox(&mut self.decode_settings.recursive, "Recursive (scan subfolders)");
        
        ui.add_space(10.0);
        ui.label(format!("{} file(s) selected", self.decode_items.len()));
    }

    fn add_jxl_files_from_folder(&mut self, folder: &PathBuf) {
        use walkdir::WalkDir;
        
        let walker = if self.decode_settings.recursive {
            WalkDir::new(folder).follow_links(false).into_iter()
        } else {
            WalkDir::new(folder).max_depth(1).follow_links(false).into_iter()
        };

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "jxl" {
                        let path_buf = path.to_path_buf();
                        if !self.decode_items.iter().any(|item| item.path == path_buf) {
                            self.decode_items.push(DecodeItem {
                                path: path_buf,
                                output_format: self.decode_settings.output_format,
                            });
                        }
                    }
                }
            }
        }
    }

    fn render_decode_output_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Output");
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Directory:");
            ui.add(
                egui::TextEdit::singleline(&mut self.decode_settings.output_dir.to_string_lossy().to_string())
                    .desired_width(ui.available_width() - 80.0)
                    .interactive(false),
            );
            if ui.button("Browse").clicked() {
                if let Some(folder) = rfd::FileDialog::new()
                    .set_title("Select Output Directory")
                    .pick_folder()
                {
                    self.decode_settings.output_dir = folder;
                }
            }
        });

        ui.add_space(5.0);
        ui.checkbox(&mut self.decode_settings.keep_structure, "Keep input folder structure");

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(5.0);

        ui.label(RichText::new("Default Output Format:").strong());
        ui.add_space(3.0);
        
        egui::ComboBox::from_id_salt("default_output_format")
            .selected_text(self.decode_settings.output_format.name())
            .show_ui(ui, |ui| {
                for format in OutputFormat::all() {
                    if ui.selectable_value(&mut self.decode_settings.output_format, *format, format.name()).clicked() {
                        // Update all items to use the new default format
                        for item in &mut self.decode_items {
                            item.output_format = self.decode_settings.output_format;
                        }
                    }
                }
            });

        ui.add_space(5.0);
        ui.label(RichText::new("(applies to all files below)").small().color(Color32::GRAY));
    }

    fn render_decode_list_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Files to Decode");
        ui.add_space(5.0);

        if self.decode_items.is_empty() {
            ui.label(RichText::new("No files added yet").color(Color32::GRAY).italics());
            return;
        }

        ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut items_to_remove = Vec::new();
                
                for (idx, item) in self.decode_items.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        // File name
                        ui.label(
                            RichText::new(item.path.file_name().unwrap().to_string_lossy())
                                .strong()
                        );
                        
                        ui.label("→");
                        
                        // Format selector
                        egui::ComboBox::from_id_salt(format!("format_{}", idx))
                            .selected_text(item.output_format.name())
                            .width(80.0)
                            .show_ui(ui, |ui| {
                                for format in OutputFormat::all() {
                                    ui.selectable_value(&mut item.output_format, *format, format.name());
                                }
                            });
                        
                        // Remove button
                        if ui.button("✖").clicked() {
                            items_to_remove.push(idx);
                        }
                    });
                    
                    ui.label(RichText::new(item.path.display().to_string()).small().color(Color32::DARK_GRAY));
                    ui.add_space(3.0);
                }

                // Remove items in reverse order to preserve indices
                for idx in items_to_remove.into_iter().rev() {
                    self.decode_items.remove(idx);
                }
            });
    }
}

impl eframe::App for JxlConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_progress_messages();

        // Request repaint if converting
        if self.is_converting {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            // Tab selection
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, AppTab::Encode, "⚙ Encode (to JXL)");
                ui.selectable_value(&mut self.active_tab, AppTab::Decode, "📦 Decode (from JXL)");
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Render content based on active tab
            match self.active_tab {
                AppTab::Encode => self.render_encode_tab(ui),
                AppTab::Decode => self.render_decode_tab(ui),
            }

            // Controls and log are shared between tabs
            ui.group(|ui| {
                self.render_controls_section(ui);
            });

            ui.group(|ui| {
                self.render_log_section(ui);
            });
        });
    }
}

impl JxlConverterApp {
    fn render_encode_tab(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            // Left column
            columns[0].group(|ui| {
                self.render_input_section(ui);
            });

            columns[0].add_space(10.0);

            columns[0].group(|ui| {
                self.render_output_section(ui);
            });

            // Right column
            columns[1].group(|ui| {
                self.render_options_section(ui);
            });
        });
    }

    fn render_decode_tab(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            // Left column
            columns[0].group(|ui| {
                self.render_decode_input_section(ui);
            });

            columns[0].add_space(10.0);

            columns[0].group(|ui| {
                self.render_decode_output_section(ui);
            });

            // Right column
            columns[1].group(|ui| {
                self.render_decode_list_section(ui);
            });
        });
    }
}
