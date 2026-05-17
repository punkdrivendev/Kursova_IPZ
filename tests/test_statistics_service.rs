use disk_analyzer::models::*;
use disk_analyzer::stat_services::statistics_service::StatService;
#[test]
fn test_category_by_extension() {
    assert_eq!(
        StatService::category_by_extension(Some("rs")),
        FileCategory::Code
    );

    assert_eq!(
        StatService::category_by_extension(Some("mp4")),
        FileCategory::Video
    );

    assert_eq!(
        StatService::category_by_extension(Some("pdf")),
        FileCategory::Documents
    );

    assert_eq!(
        StatService::category_by_extension(None),
        FileCategory::NoExtension
    );

    assert_eq!(
        StatService::category_by_extension(Some("unknown_ext")),
        FileCategory::Other
    );
}

#[test]
fn test_calculate_percentage() {
    let result = StatService::calculate_percentage(25, 100);

    assert_eq!(result, 25.0);
}

#[test]
fn test_calculate_percentage_with_zero_total() {
    let result = StatService::calculate_percentage(25, 0);

    assert_eq!(result, 0.0);
}

#[test]
fn test_group_by_categories() {
    let root = FileNode {
        id: None,
        parent_id: None,
        path: String::from("/test"),
        name: String::from("test"),
        extension: None,
        size: 300,
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
                category: None,
            },
            FileNode {
                id: None,
                parent_id: None,
                path: String::from("/test/video.mp4"),
                name: String::from("video.mp4"),
                extension: Some(String::from("mp4")),
                size: 200,
                node_type: NodeType::File,
                children: Vec::new(),
                category: None,
            },
        ],
        category: None,
    };

    let stats = StatService::group_by_categories(&root);

    assert_eq!(stats.len(), 2);

    let code_stat = stats
        .iter()
        .find(|stat| stat.category == FileCategory::Code)
        .unwrap();

    assert_eq!(code_stat.file_count, 1);
    assert_eq!(code_stat.total_size, 100);

    let video_stat = stats
        .iter()
        .find(|stat| stat.category == FileCategory::Video)
        .unwrap();

    assert_eq!(video_stat.file_count, 1);
    assert_eq!(video_stat.total_size, 200);
}