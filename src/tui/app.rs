use crate::file_actions::DeleteService;
use crate::models::*;
use crate::scanner_services::Scanner;
use crate::size_calculator::SizeCalculator;
use crate::sort_service::SortService;
use crate::stat_services::database_repository::DatabaseRepository;
use crate::stat_services::statistics_service::StatService;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    MainMenu,
    DirectoryScanMenu,
    PathInput,
    SavedDirectoryChoice,
    DiskSelection,
    FileTree,
    DeleteConfirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Name,
    Size,
    Type,
}

#[derive(Debug, Clone)]
pub struct DiskRow {
    pub index: usize,
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub free_space: u64,
}

#[derive(Debug, Clone)]
pub struct TreeRow {
    pub depth: usize,
    pub name: String,
    pub path: String,
    pub extension: Option<String>,
    pub size: u64,
    pub node_type: NodeType,
    pub category: Option<FileCategory>,
    pub children_count: usize,
    pub is_expanded: bool,
}

impl TreeRow {
    fn from_node(node: &FileNode, depth: usize, is_expanded: bool) -> Self {
        Self {
            depth,
            name: node.name.clone(),
            path: node.path.clone(),
            extension: node.extension.clone(),
            size: node.size,
            node_type: node.node_type.clone(),
            category: node.category,
            children_count: node.children.len(),
            is_expanded,
        }
    }
}

pub struct TuiApp {
    screen: AppScreen,

    menu_items: Vec<String>,
    menu_index: usize,

    directory_menu_items: Vec<String>,
    directory_menu_index: usize,

    saved_choice_items: Vec<String>,
    saved_choice_index: usize,

    disk_rows: Vec<DiskRow>,
    disk_index: usize,

    root_node: Option<FileNode>,
    rows: Vec<TreeRow>,
    expanded_paths: HashSet<String>,
    selected_index: usize,

    sort_mode: SortMode,
    show_hidden: bool,
    scan_system_dirs: bool,
    should_quit: bool,
    status_message: String,

    input_path: String,
    pending_directory_path: Option<String>,
    pending_scan: Option<ScanSession>,
    pending_is_disk_scan: bool,

    current_scan_id: Option<i64>,
    current_is_full_persistent_scan: bool,

    delete_target: Option<TreeRow>,

    db_path: String,
}

impl TuiApp {
    pub fn new(disk_rows: Vec<DiskRow>) -> Self {
        let current_dir = std::env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| String::from("."));

        Self {
            screen: AppScreen::MainMenu,
            menu_items: vec![
                String::from("Scan directory"),
                String::from("Select disk and scan"),
                String::from("Toggle system directories: off"),
                String::from("Exit"),
            ],
            menu_index: 0,

            directory_menu_items: vec![
                format!("Scan current directory: {}", current_dir),
                String::from("Enter directory path"),
                String::from("Back"),
            ],
            directory_menu_index: 0,

            saved_choice_items: vec![
                String::from("Open saved version"),
                String::from("Scan again without saving"),
                String::from("Back"),
            ],
            saved_choice_index: 0,

            disk_rows,
            disk_index: 0,

            root_node: None,
            rows: Vec::new(),
            expanded_paths: HashSet::new(),
            selected_index: 0,

            sort_mode: SortMode::Name,
            show_hidden: false,
            scan_system_dirs: false,
            should_quit: false,
            status_message: String::from("Ready"),

            input_path: String::new(),
            pending_directory_path: None,
            pending_scan: None,
            pending_is_disk_scan: false,
            
            current_scan_id: None,
            current_is_full_persistent_scan: false,

            delete_target: None,

            db_path: String::from("data/disk_analyzer.db"),
        }
    }
    pub fn scan_system_dirs(&self) -> bool {
        self.scan_system_dirs
    }
    pub fn pending_is_disk_scan(&self) -> bool {
    self.pending_is_disk_scan
    }
    fn refresh_saved_choice_items(&mut self) {
    if self.pending_is_disk_scan {
        self.saved_choice_items = vec![
            String::from("Open saved disk scan"),
            String::from("Scan again and save"),
            String::from("Back"),
        ];
    } else {
        self.saved_choice_items = vec![
            String::from("Open saved directory version"),
            String::from("Scan again without saving"),
            String::from("Back"),
        ];
    }
    }
    pub fn toggle_system_dirs_mode(&mut self) {
        self.scan_system_dirs = !self.scan_system_dirs;

        self.refresh_menu_items();

        self.status_message = if self.scan_system_dirs {
            String::from("System directories scanning: enabled")
        } else {
            String::from("System directories scanning: disabled")
        };
    }

    fn refresh_menu_items(&mut self) {
        if self.menu_items.len() >= 3 {
            self.menu_items[2] = if self.scan_system_dirs {
                String::from("Toggle system directories: on")
            } else {
                String::from("Toggle system directories: off")
            };
        }
    }
    pub fn request_disk_scan(&mut self, path: PathBuf) {
        let canonical_path = match path.canonicalize() {
            Ok(path) => path,
            Err(_) => {
                self.status_message = String::from("Disk path does not exist");
                return;
            }
        };

        let path_string = canonical_path.to_string_lossy().to_string();

        let repository = match DatabaseRepository::new(&self.db_path) {
            Ok(repository) => repository,
            Err(_) => {
                self.status_message =
                    String::from("Cannot open database. Scanning disk without saved check.");
                self.scan_path(&canonical_path, true);
                return;
            }
        };

        match repository.find_latest_scan_by_root_path(&path_string) {
            Ok(Some(scan)) => {
                self.pending_directory_path = Some(path_string);
                self.pending_scan = Some(scan);
                self.pending_is_disk_scan = true;

                self.saved_choice_index = 0;
                self.refresh_saved_choice_items();

                self.screen = AppScreen::SavedDirectoryChoice;
            }

            Ok(None) => {
                self.scan_path(&canonical_path, true);
            }

            Err(_) => {
                self.status_message = String::from("Database search failed. Scanning disk again.");
                self.scan_path(&canonical_path, true);
            }
        }
    }
    pub fn screen(&self) -> AppScreen {
        self.screen
    }

    pub fn set_screen(&mut self, screen: AppScreen) {
        self.screen = screen;
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn menu_items(&self) -> &[String] {
        &self.menu_items
    }

    pub fn menu_index(&self) -> usize {
        self.menu_index
    }

    pub fn directory_menu_items(&self) -> &[String] {
        &self.directory_menu_items
    }

    pub fn directory_menu_index(&self) -> usize {
        self.directory_menu_index
    }

    pub fn saved_choice_items(&self) -> &[String] {
        &self.saved_choice_items
    }

    pub fn saved_choice_index(&self) -> usize {
        self.saved_choice_index
    }

    pub fn disk_rows(&self) -> &[DiskRow] {
        &self.disk_rows
    }

    pub fn disk_index(&self) -> usize {
        self.disk_index
    }

    pub fn rows(&self) -> &[TreeRow] {
        &self.rows
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selected_row(&self) -> Option<&TreeRow> {
        self.rows.get(self.selected_index)
    }

    pub fn status_message(&self) -> &str {
        &self.status_message
    }

    pub fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    pub fn show_hidden(&self) -> bool {
        self.show_hidden
    }

    pub fn input_path(&self) -> &str {
        &self.input_path
    }

    pub fn pending_directory_path(&self) -> Option<&str> {
        self.pending_directory_path.as_deref()
    }

    pub fn pending_scan(&self) -> Option<&ScanSession> {
        self.pending_scan.as_ref()
    }

    pub fn selected_disk_mount_point(&self) -> Option<String> {
        self.disk_rows
            .get(self.disk_index)
            .map(|disk| disk.mount_point.clone())
    }

    pub fn next(&mut self) {
        match self.screen {
            AppScreen::MainMenu => {
                if self.menu_index + 1 < self.menu_items.len() {
                    self.menu_index += 1;
                }
            }

            AppScreen::DirectoryScanMenu => {
                if self.directory_menu_index + 1 < self.directory_menu_items.len() {
                    self.directory_menu_index += 1;
                }
            }

            AppScreen::SavedDirectoryChoice => {
                if self.saved_choice_index + 1 < self.saved_choice_items.len() {
                    self.saved_choice_index += 1;
                }
            }

            AppScreen::DiskSelection => {
                if self.disk_index + 1 < self.disk_rows.len() {
                    self.disk_index += 1;
                }
            }

            AppScreen::FileTree => {
                if self.selected_index + 1 < self.rows.len() {
                    self.selected_index += 1;
                }
            }

            AppScreen::PathInput | AppScreen::DeleteConfirm => {}
        }
    }

    pub fn previous(&mut self) {
        match self.screen {
            AppScreen::MainMenu => {
                if self.menu_index > 0 {
                    self.menu_index -= 1;
                }
            }

            AppScreen::DirectoryScanMenu => {
                if self.directory_menu_index > 0 {
                    self.directory_menu_index -= 1;
                }
            }

            AppScreen::SavedDirectoryChoice => {
                if self.saved_choice_index > 0 {
                    self.saved_choice_index -= 1;
                }
            }

            AppScreen::DiskSelection => {
                if self.disk_index > 0 {
                    self.disk_index -= 1;
                }
            }

            AppScreen::FileTree => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }

            AppScreen::PathInput | AppScreen::DeleteConfirm => {}
        }
    }

    pub fn push_input_char(&mut self, ch: char) {
        self.input_path.push(ch);
    }

    pub fn pop_input_char(&mut self) {
        self.input_path.pop();
    }

    pub fn clear_input(&mut self) {
        self.input_path.clear();
    }

    pub fn request_current_directory_scan(&mut self) {
        match std::env::current_dir() {
            Ok(path) => self.request_directory_scan(path),
            Err(_) => {
                self.status_message = String::from("Cannot get current directory");
            }
        }
    }

    pub fn request_directory_scan(&mut self, path: PathBuf) {
        let canonical_path = match path.canonicalize() {
            Ok(path) => path,
            Err(_) => {
                self.status_message = String::from("Directory does not exist");
                return;
            }
        };

        if !canonical_path.is_dir() {
            self.status_message = String::from("Path is not a directory");
            return;
        }

        let path_string = canonical_path.to_string_lossy().to_string();

        let repository = match DatabaseRepository::new(&self.db_path) {
            Ok(repository) => repository,
            Err(_) => {
                self.status_message = String::from("Cannot open database");
                self.scan_path(&canonical_path, false);
                return;
            }
        };

        match repository.find_latest_scan_containing_path(&path_string) {
            Ok(Some(scan)) => {
                self.pending_directory_path = Some(path_string);
                self.pending_scan = Some(scan);
                self.saved_choice_index = 0;
                self.screen = AppScreen::SavedDirectoryChoice;
            }

            Ok(None) => {
                self.scan_path(&canonical_path, false);
            }

            Err(_) => {
                self.status_message = String::from("Database search failed. Scanning without saving.");
                self.scan_path(&canonical_path, false);
            }
        }
    }

    pub fn open_saved_directory_version(&mut self) {
        let Some(path) = self.pending_directory_path.clone() else {
            self.status_message = String::from("No pending directory");
            return;
        };

        let Some(scan) = self.pending_scan.clone() else {
            self.status_message = String::from("No saved scan");
            return;
        };

        let Some(scan_id) = scan.id else {
            self.status_message = String::from("Saved scan has no id");
            return;
        };
        let mut repository = match DatabaseRepository::new(&self.db_path) {
            Ok(repository) => repository,
            Err(_) => {
                self.status_message = String::from("Scan completed, but database open failed");
                return;
            }
        };

        match repository.load_subtree_from_path(scan_id, &path) {
            Ok(root_node) => {
                self.root_node = Some(root_node);
                self.expanded_paths.clear();
                self.selected_index = 0;
                self.current_scan_id = Some(scan_id);
                self.current_is_full_persistent_scan = false;
                self.sort_current_tree();
                self.rebuild_rows();
                self.screen = AppScreen::FileTree;
                self.status_message = format!("Opened saved version from {}", scan.started_at);
            }

            Err(_) => {
                self.status_message = String::from("Cannot load saved directory");
            }
        }
    }

    pub fn scan_pending_path_again(&mut self) {
        let Some(path) = self.pending_directory_path.clone() else {
            self.status_message = String::from("No pending path");
            return;
        };

        if self.pending_is_disk_scan {
            self.scan_path(Path::new(&path), true);
        } else {
            self.scan_path(Path::new(&path), false);
        }
    }

    pub fn scan_path(&mut self, path: &Path, persist_to_database: bool) {
        self.status_message = format!("Scanning: {}", path.display());

        let scanner = Scanner::new_with_options(
            self.show_hidden,
            self.scan_system_dirs,
        );

        let Some(mut root_node) = scanner.scan(path) else {
            self.status_message = String::from("Scan failed");
            return;
        };

        SizeCalculator::calculate(&mut root_node);

        self.root_node = Some(root_node);
        self.expanded_paths.clear();
        self.selected_index = 0;
        self.sort_current_tree();
        self.rebuild_rows();

        self.current_scan_id = None;
        self.current_is_full_persistent_scan = false;

        if persist_to_database {
            self.save_current_scan_to_database(path);
        }

        self.screen = AppScreen::FileTree;
        self.status_message = String::from("Scan completed");
    }

    fn save_current_scan_to_database(&mut self, path: &Path) {
        let Some(root_node) = &self.root_node else {
            return;
        };

        let mut repository = match DatabaseRepository::new(&self.db_path) {
            Ok(repository) => repository,
            Err(_) => {
                self.status_message = String::from("Scan completed, but database open failed");
                return;
            }
        };

        let (files_count, dirs_count) = Self::count_nodes(root_node);

        let scan = ScanSession {
            id: None,
            root_path: path.to_string_lossy().to_string(),
            started_at: chrono::Local::now().to_rfc3339(),
            finished_at: None,
            total_size: root_node.size,
            files_count,
            dirs_count,
            status: String::from("running"),
        };

        let scan_id = match repository.save_scan(&scan) {
            Ok(scan_id) => scan_id,
            Err(_) => {
                self.status_message = String::from("Scan completed, but save_scan failed");
                return;
            }
        };

        let statistics = StatService::group_by_categories(root_node);

        if repository.save_nodes(scan_id, root_node).is_err() {
            self.status_message = String::from("Scan completed, but save_nodes failed");
            return;
        }

        if repository.save_statistics(scan_id, &statistics).is_err() {
            self.status_message = String::from("Scan completed, but save_statistics failed");
            return;
        }

        repository.finish_scan(scan_id, "completed").ok();

        self.current_scan_id = Some(scan_id);
        self.current_is_full_persistent_scan = true;
        self.status_message = format!("Scan completed and saved to database. scan_id = {}", scan_id);
    }

    pub fn toggle_selected_directory(&mut self) {
        let Some(row) = self.selected_row() else {
            return;
        };

        if row.node_type != NodeType::Directory {
            return;
        }

        let path = row.path.clone();

        if self.expanded_paths.contains(&path) {
            self.expanded_paths.remove(&path);
        } else {
            self.expanded_paths.insert(path);
        }

        self.rebuild_rows();

        if self.selected_index >= self.rows.len() {
            self.selected_index = self.rows.len().saturating_sub(1);
        }
    }

    pub fn request_delete_selected(&mut self) {
        let Some(row) = self.selected_row() else {
            return;
        };

        self.delete_target = Some(row.clone());
        self.screen = AppScreen::DeleteConfirm;
    }

    pub fn cancel_delete(&mut self) {
        self.delete_target = None;
        self.screen = AppScreen::FileTree;
    }

    pub fn delete_target(&self) -> Option<&TreeRow> {
        self.delete_target.as_ref()
    }

    pub fn confirm_delete(&mut self) {
        let Some(target) = self.delete_target.clone() else {
            self.screen = AppScreen::FileTree;
            return;
        };

        if target.depth == 0 {
            self.status_message = String::from("Cannot delete root item from this view");
            self.delete_target = None;
            self.screen = AppScreen::FileTree;
            return;
        }

        match DeleteService::delete_from_disk(&target.path, &target.node_type) {
            Ok(_) => {}

            Err(_) => {
                self.status_message = String::from("Delete from disk failed");
                self.delete_target = None;
                self.screen = AppScreen::FileTree;
                return;
            }
        }

        if let Some(root_node) = &mut self.root_node {
            Self::remove_node_by_path(root_node, &target.path);
            SizeCalculator::calculate(root_node);
        }

        self.expanded_paths.remove(&target.path);

        self.sort_current_tree();
        self.rebuild_rows();

        if self.selected_index >= self.rows.len() {
            self.selected_index = self.rows.len().saturating_sub(1);
        }

        self.update_database_after_delete(&target.path);

        self.status_message = format!("Deleted: {}", target.path);
        self.delete_target = None;
        self.screen = AppScreen::FileTree;
    }

    fn update_database_after_delete(&mut self, deleted_path: &str) {
        let Some(scan_id) = self.current_scan_id else {
            return;
        };

        let mut repository = match DatabaseRepository::new(&self.db_path) {
            Ok(repository) => repository,
            Err(_) => {
                self.status_message = String::from("Deleted from disk, but database open failed");
                return;
            }
        };

        if self.current_is_full_persistent_scan {
            let Some(root_node) = &self.root_node else {
                return;
            };

            let statistics = StatService::group_by_categories(root_node);
            let (files_count, dirs_count) = Self::count_nodes(root_node);

            if repository
                .replace_scan_data(
                    scan_id,
                    root_node,
                    &statistics,
                    files_count,
                    dirs_count,
                )
                .is_err()
            {
                self.status_message = String::from("Deleted from disk, but database update failed");
            }
        } else {
            if repository
                .delete_path_from_scan(scan_id, deleted_path)
                .is_err()
            {
                self.status_message = String::from("Deleted from disk, but database path delete failed");
            }
        }
    }

    fn remove_node_by_path(node: &mut FileNode, target_path: &str) -> bool {
        let old_len = node.children.len();

        node.children.retain(|child| child.path != target_path);

        if node.children.len() != old_len {
            return true;
        }

        for child in &mut node.children {
            if Self::remove_node_by_path(child, target_path) {
                return true;
            }
        }

        false
    }

    pub fn set_sort_mode(&mut self, sort_mode: SortMode) {
        self.sort_mode = sort_mode;
        self.sort_current_tree();
        self.rebuild_rows();
    }

    pub fn toggle_hidden_mode(&mut self) {
        self.show_hidden = !self.show_hidden;

        self.status_message = if self.show_hidden {
            String::from("Hidden files: enabled. Rescan to apply.")
        } else {
            String::from("Hidden files: disabled. Rescan to apply.")
        };
    }

    fn rebuild_rows(&mut self) {
        let mut rows = Vec::new();

        if let Some(root_node) = &self.root_node {
            Self::collect_visible_rows(root_node, 0, &self.expanded_paths, &mut rows);
        }

        self.rows = rows;
    }

    fn collect_visible_rows(
        node: &FileNode,
        depth: usize,
        expanded_paths: &HashSet<String>,
        rows: &mut Vec<TreeRow>,
    ) {
        let is_expanded = expanded_paths.contains(&node.path);

        rows.push(TreeRow::from_node(node, depth, is_expanded));

        if node.node_type == NodeType::Directory && is_expanded {
            for child in &node.children {
                Self::collect_visible_rows(child, depth + 1, expanded_paths, rows);
            }
        }
    }

    fn sort_current_tree(&mut self) {
        if let Some(root_node) = &mut self.root_node {
            Self::sort_node_recursive(root_node, self.sort_mode);
        }
    }

    fn sort_node_recursive(node: &mut FileNode, sort_mode: SortMode) {
        let children = std::mem::take(&mut node.children);

        node.children = match sort_mode {
            SortMode::Name => SortService::by_name(children),
            SortMode::Size => SortService::by_size(children),
            SortMode::Type => SortService::by_type(children),
        };

        for child in &mut node.children {
            Self::sort_node_recursive(child, sort_mode);
        }
    }

    fn count_nodes(node: &FileNode) -> (u64, u64) {
        let mut files_count = 0;
        let mut dirs_count = 0;

        match node.node_type {
            NodeType::File | NodeType::Symlink => {
                files_count += 1;
            }

            NodeType::Directory => {
                dirs_count += 1;
            }
        }

        for child in &node.children {
            let (child_files, child_dirs) = Self::count_nodes(child);
            files_count += child_files;
            dirs_count += child_dirs;
        }

        (files_count, dirs_count)
    }
}