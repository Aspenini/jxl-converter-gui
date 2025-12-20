use std::path::PathBuf;

#[derive(Clone)]
pub struct ConversionSettings {
    pub output_dir: PathBuf,
    pub lossless: bool,
    pub quality: u8,
    pub effort: u8,
    pub recursive: bool,
    pub keep_structure: bool,
}

impl Default for ConversionSettings {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::new(),
            lossless: false,
            quality: 90,
            effort: 7,
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


