use disk_analyzer::models::*;
use disk_analyzer::size_calculator::SizeCalculator;

fn create_test_tree() -> FileNode {
    FileNode {
        id: None,
        parent_id: None,
        path: String::from("/test"),
        name: String::from("test"),
        extension: None,
        size: 0,
        node_type: NodeType::Directory,
        children: vec![
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
                path: String::from("/test/docs"),
                name: String::from("docs"),
                extension: None,
                size: 0,
                node_type: NodeType::Directory,
                children: vec![
                    FileNode {
                        id: None,
                        parent_id: None,
                        path: String::from("/test/docs/report.pdf"),
                        name: String::from("report.pdf"),
                        extension: Some(String::from("pdf")),
                        size: 200,
                        node_type: NodeType::File,
                        children: Vec::new(),
                        category: Some(FileCategory::Documents),
                    },
                    FileNode {
                        id: None,
                        parent_id: None,
                        path: String::from("/test/docs/image.png"),
                        name: String::from("image.png"),
                        extension: Some(String::from("png")),
                        size: 300,
                        node_type: NodeType::File,
                        children: Vec::new(),
                        category: Some(FileCategory::Images),
                    },
                ],
                category: None,
            },
        ],
        category: None,
    }
}

#[test]
fn test_size_calculator_calculates_root_directory_size() {
    let mut root = create_test_tree();

    let total_size = SizeCalculator::calculate(&mut root);

    assert_eq!(total_size, 600);
    assert_eq!(root.size, 600);
}

#[test]
fn test_size_calculator_updates_nested_directory_size() {
    let mut root = create_test_tree();

    SizeCalculator::calculate(&mut root);

    let docs_dir = root
        .children
        .iter()
        .find(|node| node.name == "docs")
        .unwrap();

    assert_eq!(docs_dir.size, 500);
}

#[test]
fn test_size_calculator_keeps_file_size() {
    let mut file = FileNode {
        id: None,
        parent_id: None,
        path: String::from("/test/file.txt"),
        name: String::from("file.txt"),
        extension: Some(String::from("txt")),
        size: 123,
        node_type: NodeType::File,
        children: Vec::new(),
        category: Some(FileCategory::Documents),
    };

    let result = SizeCalculator::calculate(&mut file);

    assert_eq!(result, 123);
    assert_eq!(file.size, 123);
}