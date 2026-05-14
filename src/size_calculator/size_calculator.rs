use crate::models::*;

pub struct SizeCalculator;

impl SizeCalculator {
    pub fn calculate(node: &mut FileNode) -> u64 {
        match node.node_type {
            NodeType::File => node.size,

            NodeType::Directory => {
                let mut total_size = 0;

                for child in &mut node.children {
                    total_size += Self::calculate(child);
                }

                node.size = total_size;

                total_size
            }

            NodeType::Symlink => node.size,
        }
    }
}