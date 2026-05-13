use crate::models::*;

pub struct SortService;

impl SortService {
    pub fn by_name(mut nodes: Vec<FileNode>) -> Vec<FileNode> {
        nodes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        nodes
    }

    pub fn by_size(mut nodes: Vec<FileNode>) -> Vec<FileNode> {
        nodes.sort_by(|a, b| b.size.cmp(&a.size));
        nodes
    }

    pub fn by_type(mut nodes: Vec<FileNode>) -> Vec<FileNode> {
        nodes.sort_by(|a, b| {
            let type_order_a = Self::node_type_order(&a.node_type);
            let type_order_b = Self::node_type_order(&b.node_type);

            type_order_a
                .cmp(&type_order_b)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        nodes
    }

    fn node_type_order(node_type: &NodeType) -> u8 {
        match node_type {
            NodeType::Directory => 0,
            NodeType::File => 1,
            NodeType::Symlink => 2,
        }
    }
}