use crate::models::{FileNode, NodeType};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Scanner {
    pub show_hidden: bool,
    pub scan_system_dirs: bool,
}

impl Scanner {
    pub fn new(show_hidden: bool) -> Self {
        Self {
            show_hidden,
            scan_system_dirs: false,
        }
    }

    pub fn new_with_options(show_hidden: bool, scan_system_dirs: bool) -> Self {
        Self {
            show_hidden,
            scan_system_dirs,
        }
    }

    fn should_skip_path(&self, path: &Path) -> bool {
        if self.scan_system_dirs {
            return false;
        }

        path.starts_with("/proc")
            || path.starts_with("/sys")
            || path.starts_with("/dev")
            || path.starts_with("/run")
            || path.starts_with("/tmp")
    }

    pub fn scan(&self, path: &Path) -> Option<FileNode> {
        if self.should_skip_path(path) {
            return None;
        }

        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => return None,
        };

        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        if !self.show_hidden && name.starts_with('.') {
            return None;
        }

        let extension = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_string());

        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            return Some(FileNode {
                id: None,
                parent_id: None,
                path: path.to_string_lossy().to_string(),
                name,
                extension,
                size: metadata.len(),
                node_type: NodeType::Symlink,
                children: Vec::new(),
                category: None,
            });
        }

        if metadata.is_file() {
            return Some(FileNode {
                id: None,
                parent_id: None,
                path: path.to_string_lossy().to_string(),
                name,
                extension,
                size: metadata.len(),
                node_type: NodeType::File,
                children: Vec::new(),
                category: None,
            });
        }

        if metadata.is_dir() {
            let child_paths = match Self::read_child_paths(path) {
                Some(paths) => paths,
                None => {
                    return Some(FileNode {
                        id: None,
                        parent_id: None,
                        path: path.to_string_lossy().to_string(),
                        name,
                        extension: None,
                        size: 0,
                        node_type: NodeType::Directory,
                        children: Vec::new(),
                        category: None,
                    });
                }
            };

            let children: Vec<FileNode> = child_paths
                .par_iter()
                .filter_map(|child_path| self.scan(child_path))
                .collect();

            return Some(FileNode {
                id: None,
                parent_id: None,
                path: path.to_string_lossy().to_string(),
                name,
                extension: None,
                size: 0,
                node_type: NodeType::Directory,
                children,
                category: None,
            });
        }

        None
    }

    fn read_child_paths(path: &Path) -> Option<Vec<PathBuf>> {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => return None,
        };

        let paths: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect();

        Some(paths)
    }
}