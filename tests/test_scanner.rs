use disk_analyzer::models::NodeType;
use disk_analyzer::scanner_services::Scanner;
use std::fs;
use std::path::{Path, PathBuf};

fn make_temp_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("scanner_tests")
        .join(format!(
            "disk_analyzer_{}_{}_{}",
            name,
            std::process::id(),
            chrono::Local::now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));

    fs::create_dir_all(&path).unwrap();
    path
}

fn child_names(path: &Path, show_hidden: bool) -> Vec<String> {
    let scanner = Scanner::new_with_options(show_hidden, false);
    let root = scanner.scan(path).unwrap();

    root.children
        .iter()
        .map(|child| child.name.clone())
        .collect()
}

#[test]
fn scanner_hides_hidden_directories_by_default() {
    let root = make_temp_dir("hide_hidden_dirs");
    fs::create_dir(root.join(".hidden_dir")).unwrap();
    fs::create_dir(root.join("visible_dir")).unwrap();

    let names = child_names(&root, false);

    assert!(!names.contains(&String::from(".hidden_dir")));
    assert!(names.contains(&String::from("visible_dir")));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn scanner_includes_hidden_directories_when_enabled() {
    let root = make_temp_dir("show_hidden_dirs");
    fs::create_dir(root.join(".hidden_dir")).unwrap();
    fs::write(root.join(".hidden_dir").join("file.txt"), "content").unwrap();

    let scanner = Scanner::new_with_options(true, false);
    let scanned_root = scanner.scan(&root).unwrap();
    let hidden_dir = scanned_root
        .children
        .iter()
        .find(|child| child.name == ".hidden_dir")
        .unwrap();

    assert_eq!(hidden_dir.node_type, NodeType::Directory);
    assert!(
        hidden_dir
            .children
            .iter()
            .any(|child| child.name == "file.txt")
    );

    fs::remove_dir_all(root).unwrap();
}
