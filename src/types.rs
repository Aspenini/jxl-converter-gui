use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OutputFormat {
    Png,
    Jpeg,
    Ppm,
    Pgm,
    Pbm,
}

impl OutputFormat {
    pub fn extension(&self) -> &str {
        match self {
            OutputFormat::Png => "png",
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Ppm => "ppm",
            OutputFormat::Pgm => "pgm",
            OutputFormat::Pbm => "pbm",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            OutputFormat::Png => "PNG",
            OutputFormat::Jpeg => "JPEG",
            OutputFormat::Ppm => "PPM",
            OutputFormat::Pgm => "PGM",
            OutputFormat::Pbm => "PBM",
        }
    }

    pub fn all() -> &'static [OutputFormat] {
        &[
            OutputFormat::Png,
            OutputFormat::Jpeg,
            OutputFormat::Ppm,
            OutputFormat::Pgm,
            OutputFormat::Pbm,
        ]
    }
}

#[derive(Clone)]
pub struct ConversionSettings {
    pub output_dir: PathBuf,
    pub lossless: bool,
    pub jpeg_lossless: bool,
    pub quality: u8,
    pub effort: u8,
    pub recursive: bool,
    pub keep_structure: bool,
}

#[derive(Clone)]
pub struct DecodeSettings {
    pub output_dir: PathBuf,
    pub output_format: OutputFormat,
    pub recursive: bool,
    pub keep_structure: bool,
}

#[derive(Clone)]
pub struct DecodeItem {
    pub path: PathBuf,
    pub output_format: OutputFormat,
}

impl Default for ConversionSettings {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::new(),
            lossless: false,
            jpeg_lossless: true,
            quality: 90,
            effort: 7,
            recursive: true,
            keep_structure: false,
        }
    }
}

impl Default for DecodeSettings {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::new(),
            output_format: OutputFormat::Png,
            recursive: true,
            keep_structure: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProgressMessage {
    Started { total: usize },
    Progress { current: usize, total: usize, file: String },
    Success { file: String },
    Error { file: String, error: String },
    #[allow(dead_code)]
    Skipped { file: String, reason: String },
    Completed,
    Cancelled,
}

#[derive(Clone, Debug)]
pub enum LogEntry {
    Info(String),
    Success(String),
    Error(String),
    Warning(String),
}


