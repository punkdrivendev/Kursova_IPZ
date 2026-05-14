use crate::models::NodeType;
use std::fs;
use std::io;
use std::path::Path;

pub struct DeleteService;

impl DeleteService {
    pub fn delete_from_disk(path: &str, node_type: &NodeType) -> io::Result<()> {
        let path = Path::new(path);

        match node_type {
            NodeType::File | NodeType::Symlink => fs::remove_file(path),
            NodeType::Directory => fs::remove_dir_all(path),
        }
    }
}