#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub id: Option<i64>,
    pub parent_id: Option<i64>,
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub node_type: NodeType,
    pub children: Vec<FileNode>,
    pub category: Option<FileCategory>,
}
#[derive(Debug, Clone)]
pub struct FileStatistic { // Енумератор для статистики файлів 
    pub category: FileCategory,
    pub file_count: usize,
    pub total_size: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileCategory { // Енумератор для категорій файлів
    Video,
    Audio,
    Documents,
    Code,
    Archives,
    Images,
    NoExtension,
    Other,
}

impl FileCategory {
    pub fn as_str(&self) -> &'static str { //
        match self {
            FileCategory::Video => "video",
            FileCategory::Audio => "audio",
            FileCategory::Documents => "documents",
            FileCategory::Code => "code",
            FileCategory::Archives => "archives",
            FileCategory::Images => "images",
            FileCategory::NoExtension => "no_extension",
            FileCategory::Other => "other",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanSession {
    pub id: Option<i64>,
    pub root_path: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub total_size: u64,
    pub files_count: u64,
    pub dirs_count: u64,
    pub status: String,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::File => "file",
            NodeType::Directory => "directory",
            NodeType::Symlink => "symlink",
        }
    }
}
