use disk_analyzer::models::*;
use disk_analyzer::stat_services::database_repository::DatabaseRepository;

use rusqlite::Connection;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn create_test_database() -> String {
    let unique_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let db_path = std::env::temp_dir().join(format!("disk_analyzer_test_{}.db", unique_id));

    let db_path_string = db_path.to_string_lossy().to_string();

    let connection = Connection::open(&db_path_string).unwrap();

    let schema = include_str!("../src/migrations/init.sql");

    connection.execute_batch(schema).unwrap();

    db_path_string
}

#[test]
fn test_save_nodes() {
    let db_path = create_test_database();

    let mut repository = DatabaseRepository::new(&db_path).unwrap();

    let scan = ScanSession {
        id: None,
        root_path: String::from("/test"),
        started_at: String::from("2026-05-04 12:00:00"),
        finished_at: None,
        total_size: 300,
        files_count: 2,
        dirs_count: 1,
        status: String::from("completed"),
    };

    let scan_id = repository.save_scan(&scan).unwrap();

    let root_node = FileNode {
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
                category: Some(FileCategory::Code),
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
                category: Some(FileCategory::Video),
            },
        ],
        category: None,
    };

    repository.save_nodes(scan_id, &root_node).unwrap();

    let connection = Connection::open(&db_path).unwrap();

    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM file_nodes WHERE scan_id = ?1",
            [scan_id],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(count, 3);

    fs::remove_file(db_path).ok();
}

#[test]
fn test_save_statistics() {
    let db_path = create_test_database();

    let mut repository = DatabaseRepository::new(&db_path).unwrap();

    let scan = ScanSession {
        id: None,
        root_path: String::from("/test"),
        started_at: String::from("2026-05-04 12:00:00"),
        finished_at: None,
        total_size: 300,
        files_count: 2,
        dirs_count: 1,
        status: String::from("completed"),
    };

    let scan_id = repository.save_scan(&scan).unwrap();

    let statistics = vec![
        FileStatistic {
            category: FileCategory::Code,
            file_count: 1,
            total_size: 100,
            percentage: 33.33,
        },
        FileStatistic {
            category: FileCategory::Video,
            file_count: 1,
            total_size: 200,
            percentage: 66.67,
        },
    ];

    repository.save_statistics(scan_id, &statistics).unwrap();

    let connection = Connection::open(&db_path).unwrap();

    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM file_statistics WHERE scan_id = ?1",
            [scan_id],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(count, 2);

    fs::remove_file(db_path).ok();
}

#[test]
fn test_apply_delete_to_full_scan_updates_database_without_replacing_scan() {
    let db_path = create_test_database();

    let mut repository = DatabaseRepository::new(&db_path).unwrap();

    let scan = ScanSession {
        id: None,
        root_path: String::from("/test"),
        started_at: String::from("2026-05-04 12:00:00"),
        finished_at: None,
        total_size: 300,
        files_count: 2,
        dirs_count: 2,
        status: String::from("completed"),
    };

    let scan_id = repository.save_scan(&scan).unwrap();

    let original_root = FileNode {
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
                path: String::from("/test/keep.txt"),
                name: String::from("keep.txt"),
                extension: Some(String::from("txt")),
                size: 100,
                node_type: NodeType::File,
                children: Vec::new(),
                category: None,
            },
            FileNode {
                id: None,
                parent_id: None,
                path: String::from("/test/remove"),
                name: String::from("remove"),
                extension: None,
                size: 200,
                node_type: NodeType::Directory,
                children: vec![FileNode {
                    id: None,
                    parent_id: None,
                    path: String::from("/test/remove/file.mp4"),
                    name: String::from("file.mp4"),
                    extension: Some(String::from("mp4")),
                    size: 200,
                    node_type: NodeType::File,
                    children: Vec::new(),
                    category: None,
                }],
                category: None,
            },
        ],
        category: None,
    };

    repository.save_nodes(scan_id, &original_root).unwrap();

    let updated_root = FileNode {
        children: vec![original_root.children[0].clone()],
        size: 100,
        ..original_root
    };

    let statistics = vec![FileStatistic {
        category: FileCategory::Documents,
        file_count: 1,
        total_size: 100,
        percentage: 100.0,
    }];

    repository
        .apply_delete_to_full_scan(scan_id, "/test/remove", &updated_root, &statistics, 1, 1)
        .unwrap();

    let connection = Connection::open(&db_path).unwrap();

    let removed_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM file_nodes WHERE scan_id = ?1 AND path LIKE '/test/remove%'",
            [scan_id],
            |row| row.get(0),
        )
        .unwrap();

    let root_size: i64 = connection
        .query_row(
            "SELECT size FROM file_nodes WHERE scan_id = ?1 AND path = '/test'",
            [scan_id],
            |row| row.get(0),
        )
        .unwrap();

    let summary: (i64, i64, i64) = connection
        .query_row(
            "SELECT total_size, files_count, dirs_count FROM scans WHERE id = ?1",
            [scan_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();

    assert_eq!(removed_count, 0);
    assert_eq!(root_size, 100);
    assert_eq!(summary, (100, 1, 1));

    fs::remove_file(db_path).ok();
}
