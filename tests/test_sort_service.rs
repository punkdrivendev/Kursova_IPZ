use disk_analyzer::models::*;
use disk_analyzer::sort_service::SortService;

fn create_test_nodes() -> Vec<FileNode> {
    vec![
        FileNode {
            id: None,
            parent_id: None,
            path: String::from("/test/video.mp4"),
            name: String::from("video.mp4"),
            extension: Some(String::from("mp4")),
            size: 300,
            node_type: NodeType::File,
            children: Vec::new(),
            category: Some(FileCategory::Video),
        },
        FileNode {
            id: None,
            parent_id: None,
            path: String::from("/test/docs"),
            name: String::from("docs"),
            extension: None,
            size: 500,
            node_type: NodeType::Directory,
            children: Vec::new(),
            category: None,
        },
        FileNode {
            id: None,
            parent_id: None,
            path: String::from("/test/main.rs"),
            name: String::from("main.rs"),
            extension: Some(String::from("rs")),
            size: 100,
            node_type: NodeType::File,
            children: Vec::new(),
            category: Some(FileCategory::Code),
        },
        FileNode {
            id: None,
            parent_id: None,
            path: String::from("/test/link"),
            name: String::from("link"),
            extension: None,
            size: 0,
            node_type: NodeType::Symlink,
            children: Vec::new(),
            category: None,
        },
    ]
}

#[test]
fn test_sort_by_name() {
    let nodes = create_test_nodes();

    let sorted = SortService::by_name(nodes);

    let names: Vec<&str> = sorted
        .iter()
        .map(|node| node.name.as_str())
        .collect();

    assert_eq!(
        names,
        vec!["docs", "link", "main.rs", "video.mp4"]
    );
}

#[test]
fn test_sort_by_size_descending() {
    let nodes = create_test_nodes();

    let sorted = SortService::by_size(nodes);

    let sizes: Vec<u64> = sorted
        .iter()
        .map(|node| node.size)
        .collect();

    assert_eq!(sizes, vec![500, 300, 100, 0]);
}

#[test]
fn test_sort_by_type() {
    let nodes = create_test_nodes();

    let sorted = SortService::by_type(nodes);

    assert_eq!(sorted[0].node_type, NodeType::Directory);
    assert_eq!(sorted[1].node_type, NodeType::File);
    assert_eq!(sorted[2].node_type, NodeType::File);
    assert_eq!(sorted[3].node_type, NodeType::Symlink);
}

#[test]
fn test_sort_by_type_keeps_files_sorted_by_name_inside_type() {
    let nodes = create_test_nodes();

    let sorted = SortService::by_type(nodes);

    let names: Vec<&str> = sorted
        .iter()
        .map(|node| node.name.as_str())
        .collect();

    assert_eq!(
        names,
        vec!["docs", "main.rs", "video.mp4", "link"]
    );
}