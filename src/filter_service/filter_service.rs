use crate::models::*;

pub struct FilterService;

impl FilterService {
    pub fn by_extension<'a>(root: &'a FileNode, extension: &str) -> Vec<&'a FileNode> {
        let mut result = Vec::new();

        let normalized_extension = extension
            .trim_start_matches('.')
            .to_lowercase();

        Self::collect_by_extension(root, &normalized_extension, &mut result);

        result
    }

    fn collect_by_extension<'a>(
        node: &'a FileNode,
        extension: &str,
        result: &mut Vec<&'a FileNode>,
    ) {
        if matches!(node.node_type, NodeType::File) {
            if let Some(node_extension) = node.extension.as_deref() {
                let node_extension = node_extension
                    .trim_start_matches('.')
                    .to_lowercase();

                if node_extension == extension {
                    result.push(node);
                }
            }
        }

        for child in &node.children {
            Self::collect_by_extension(child, extension, result);
        }
    }

    pub fn by_min_size<'a>(root: &'a FileNode, min_size: u64) -> Vec<&'a FileNode> {
        let mut result = Vec::new();

        Self::collect_by_min_size(root, min_size, &mut result);

        result
    }

    fn collect_by_min_size<'a>(
        node: &'a FileNode,
        min_size: u64,
        result: &mut Vec<&'a FileNode>,
    ) {
        if node.size >= min_size {
            result.push(node);
        }

        for child in &node.children {
            Self::collect_by_min_size(child, min_size, result);
        }
    }
}