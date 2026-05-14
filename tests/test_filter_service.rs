use disk_analyzer::filter_service::FilterService;
use disk_analyzer::models::*;

fn create_test_tree() -> FileNode {
    FileNode {
        id: None,
        parent_id: None,
        path: String::from("/test"),
        name: String::from("test"),
        extension: None,
        size: 1_500_000,
        node_type: NodeType::Directory,
        children: vec![
            FileNode {
                id: None,
                parent_id: None,
                path: String::from("/test/main.rs"),
                name: String::from("main.rs"),
                extension: Some(String::from("rs")),
                size: 100_000,
                node_type: NodeType::File,
                children: Vec::new(),
                category: Some(FileCategory::Code),
            },
            FileNode {
                id: None,
                parent_id: None,
                path: String::from("/test/video.mp4"),
                name: String::from("video.mp4"),
                extension: Some(String::from("mp4")),
                size: 1_200_000,
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
                size: 200_000,
                node_type: NodeType::Directory,
                children: vec![
                    FileNode {
                        id: None,
                        parent_id: None,
                        path: String::from("/test/docs/report.pdf"),
                        name: String::from("report.pdf"),
                        extension: Some(String::from("pdf")),
                        size: 200_000,
                        node_type: NodeType::File,
                        children: Vec::new(),
                        category: Some(FileCategory::Documents),
                    },
                ],
                category: None,
            },
        ],
        category: None,
    }
}

#[test]
fn test_filter_by_extension_rs() {
    let root = create_test_tree();

    let result = FilterService::by_extension(&root, "rs");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "main.rs");
    assert_eq!(result[0].extension.as_deref(), Some("rs"));
}

#[test]
fn test_filter_by_extension_with_dot() {
    let root = create_test_tree();

    let result = FilterService::by_extension(&root, ".mp4");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "video.mp4");
    assert_eq!(result[0].extension.as_deref(), Some("mp4"));
}

#[test]
fn test_filter_by_extension_pdf_in_nested_directory() {
    let root = create_test_tree();

    let result = FilterService::by_extension(&root, "pdf");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "report.pdf");
    assert_eq!(result[0].path, "/test/docs/report.pdf");
}

#[test]
fn test_filter_by_unknown_extension() {
    let root = create_test_tree();

    let result = FilterService::by_extension(&root, "zip");

    assert_eq!(result.len(), 0);
}

#[test]
fn test_filter_by_min_size() {
    let root = create_test_tree();

    let result = FilterService::by_min_size(&root, 1_000_000);

    assert_eq!(result.len(), 2);

    let names: Vec<&str> = result
        .iter()
        .map(|node| node.name.as_str())
        .collect();

    assert!(names.contains(&"test"));
    assert!(names.contains(&"video.mp4"));
}

#[test]
fn test_filter_by_min_size_no_results() {
    let root = create_test_tree();

    let result = FilterService::by_min_size(&root, 10_000_000);

    assert_eq!(result.len(), 0);
}