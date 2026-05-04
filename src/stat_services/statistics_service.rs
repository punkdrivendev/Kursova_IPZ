use crate::models::*;
use crate::stat_services::system_info_service::SystemInfoService;
use std::collections::HashMap;

/*
Сервіс групування файлів за категоріями
та обчислення відсоткового розподілу зайнятого простору.
*/

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "webm"];
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "ogg"];
const DOCUMENT_EXTENSIONS: &[&str] = &["pdf", "doc", "docx", "txt", "odt", "rtf"];
const CODE_EXTENSIONS: &[&str] = &["rs", "py", "js", "ts", "html", "css", "c", "cpp", "java"];
const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "rar", "7z", "tar", "gz"];
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "svg", "webp"];

#[derive(Debug, Clone)]
pub struct SystemStatistic {
    pub file_stats: Vec<FileStatistic>,
    pub total_disk_space: u64,
    pub free_disk_space: u64,
    pub used_disk_space: u64,
}

pub struct StatService;

impl StatService {
    pub fn group_by_categories(root: &FileNode) -> Vec<FileStatistic> {
        let mut categories: HashMap<FileCategory, (usize, u64)> = HashMap::new();

        Self::collect_stats(root, &mut categories);

        let total_size: u64 = categories
            .values()
            .map(|(_, size)| *size)
            .sum();

        let mut result: Vec<FileStatistic> = categories
            .into_iter()
            .map(|(category, (file_count, total_size_by_category))| {
                FileStatistic {
                    category,
                    file_count,
                    total_size: total_size_by_category,
                    percentage: Self::calculate_percentage(
                        total_size_by_category,
                        total_size,
                    ),
                }
            })
            .collect();

        result.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        result
    }

    fn collect_stats(
        node: &FileNode,
        categories: &mut HashMap<FileCategory, (usize, u64)>,
    ) {
        if matches!(node.node_type, NodeType::File) {
            let category = node.category.unwrap_or_else(|| {
                Self::category_by_extension(node.extension.as_deref())
            });

            let entry = categories.entry(category).or_insert((0, 0));

            entry.0 += 1;
            entry.1 += node.size;
        }

        for child in &node.children {
            Self::collect_stats(child, categories);
        }
    }

    pub fn category_by_extension(extension: Option<&str>) -> FileCategory {
        let Some(extension) = extension else {
            return FileCategory::NoExtension;
        };

        let extension = extension
            .trim_start_matches('.')
            .to_lowercase();

        let extension = extension.as_str();

        if VIDEO_EXTENSIONS.contains(&extension) {
            FileCategory::Video
        } else if AUDIO_EXTENSIONS.contains(&extension) {
            FileCategory::Audio
        } else if DOCUMENT_EXTENSIONS.contains(&extension) {
            FileCategory::Documents
        } else if CODE_EXTENSIONS.contains(&extension) {
            FileCategory::Code
        } else if ARCHIVE_EXTENSIONS.contains(&extension) {
            FileCategory::Archives
        } else if IMAGE_EXTENSIONS.contains(&extension) {
            FileCategory::Images
        } else {
            FileCategory::Other
        }
    }

    pub fn calculate_percentage(part: u64, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            part as f64 * 100.0 / total as f64
        }
    }

    pub fn build_system_statistic(
        root: &FileNode,
        system_info: &SystemInfoService,
    ) -> SystemStatistic {
        SystemStatistic {
            file_stats: Self::group_by_categories(root),
            total_disk_space: system_info.get_total_disk_space(),
            free_disk_space: system_info.get_free_disk_space(),
            used_disk_space: system_info.get_used_disk_space(),
        }
    }
}