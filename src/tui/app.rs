use crate::file_actions::DeleteService;
use crate::models::*;
use crate::scanner_services::Scanner;
use crate::size_calculator::SizeCalculator;
use crate::stat_services::database_repository::DatabaseRepository;
use crate::stat_services::statistics_service::StatService;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver},
    Arc,
};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    MainMenu,
    DirectoryScanMenu,
    PathInput,
    SavedDirectoryChoice,
    DiskSelection,
    Scanning,
    Deleting,
    FileTree,
    DeleteConfirm,
}

enum ScanWorkerMessage {
    Finished {
        root_node: FileNode,
        scan_id: Option<i64>,
        persistent: bool,
        message: String,
    },
    Failed(String),
    Cancelled,
}

enum DeleteWorkerMessage {
    Finished {
        root_node: FileNode,
        deleted_path: String,
        message: String,
    },
    Failed {
        root_node: FileNode,
        message: String,
    },
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

    scan_receiver: Option<Receiver<ScanWorkerMessage>>,
    scan_cancel_flag: Option<Arc<AtomicBool>>,
    scanning_dots: usize,

    delete_receiver: Option<Receiver<DeleteWorkerMessage>>,
    deleting_dots: usize,
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
                String::from("Scan again"),
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

            db_path: Self::default_db_path(),

            scan_receiver: None,
            scan_cancel_flag: None,
            scanning_dots: 0,

            delete_receiver: None,
            deleting_dots: 0,
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

    pub fn scan_system_dirs(&self) -> bool {
        self.scan_system_dirs
    }

    pub fn pending_is_disk_scan(&self) -> bool {
        self.pending_is_disk_scan
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

    pub fn scanning_dots(&self) -> usize {
        self.scanning_dots
    }

    pub fn deleting_dots(&self) -> usize {
        self.deleting_dots
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

            AppScreen::PathInput
            | AppScreen::DeleteConfirm
            | AppScreen::Scanning
            | AppScreen::Deleting => {}
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

            AppScreen::PathInput
            | AppScreen::DeleteConfirm
            | AppScreen::Scanning
            | AppScreen::Deleting => {}
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
                String::from("Scan again and save"),
                String::from("Back"),
            ];
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
                self.start_scan_async(canonical_path, true);
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
                self.start_scan_async(canonical_path, true);
            }

            Err(error) => {
                self.status_message = format!("Database search failed: {}", error);
                self.start_scan_async(canonical_path, true);
            }
        }
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
                self.start_scan_async(canonical_path, true);
                return;
            }
        };

        match repository.find_latest_scan_containing_path(&path_string) {
            Ok(Some(scan)) => {
                self.pending_directory_path = Some(path_string);
                self.pending_scan = Some(scan);
                self.pending_is_disk_scan = false;

                self.saved_choice_index = 0;
                self.refresh_saved_choice_items();

                self.screen = AppScreen::SavedDirectoryChoice;
            }

            Ok(None) => {
                self.start_scan_async(canonical_path, true);
            }

            Err(error) => {
                self.status_message = format!("Database search failed: {}", error);
                self.start_scan_async(canonical_path, true);
            }
        }
    }

    pub fn open_saved_directory_version(&mut self) {
        let Some(path) = self.pending_directory_path.clone() else {
            self.status_message = String::from("No pending path");
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

        let repository = match DatabaseRepository::new(&self.db_path) {
            Ok(repository) => repository,
            Err(error) => {
                self.status_message = format!("Cannot open database: {}", error);
                return;
            }
        };

        match repository.load_subtree_from_path(scan_id, &path) {
            Ok(mut root_node) => {
                SizeCalculator::calculate(&mut root_node);

                self.root_node = Some(root_node);
                self.expanded_paths.clear();
                self.selected_index = 0;

                self.current_scan_id = Some(scan_id);
                self.current_is_full_persistent_scan = self.pending_is_disk_scan;

                self.rebuild_rows();

                self.screen = AppScreen::FileTree;
                self.status_message = format!("Opened saved version from {}", scan.started_at);
            }

            Err(error) => {
                self.status_message = format!("Cannot load saved version: {}", error);
            }
        }
    }

    pub fn scan_pending_path_again(&mut self) {
        let Some(path) = self.pending_directory_path.clone() else {
            self.status_message = String::from("No pending path");
            return;
        };

        self.start_scan_async(PathBuf::from(path), true);
    }

    pub fn scan_path(&mut self, path: &Path, persist_to_database: bool) {
        self.start_scan_async(path.to_path_buf(), persist_to_database);
    }

    fn start_scan_async(&mut self, path: PathBuf, persist_to_database: bool) {
        self.status_message = format!("Scanning: {}", path.display());
        self.screen = AppScreen::Scanning;
        self.scanning_dots = 0;

        let (sender, receiver) = mpsc::channel();

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let worker_cancel_flag = Arc::clone(&cancel_flag);

        let show_hidden = self.show_hidden;
        let scan_system_dirs = self.scan_system_dirs;
        let db_path = self.db_path.clone();

        thread::spawn(move || {
            let scanner = Scanner::new_with_options(show_hidden, scan_system_dirs);

            let Some(mut root_node) = scanner.scan_with_cancel(&path, worker_cancel_flag.as_ref())
            else {
                if worker_cancel_flag.load(Ordering::Relaxed) {
                    sender.send(ScanWorkerMessage::Cancelled).ok();
                } else {
                    sender
                        .send(ScanWorkerMessage::Failed(String::from("Scan failed")))
                        .ok();
                }

                return;
            };

            if worker_cancel_flag.load(Ordering::Relaxed) {
                sender.send(ScanWorkerMessage::Cancelled).ok();
                return;
            }

            SizeCalculator::calculate(&mut root_node);

            if worker_cancel_flag.load(Ordering::Relaxed) {
                sender.send(ScanWorkerMessage::Cancelled).ok();
                return;
            }

            let mut result_message = String::from("Scan completed");
            let mut persistent = false;

            let scan_id = if persist_to_database {
                match Self::save_scan_to_database_from_worker(&db_path, &path, &root_node) {
                    Ok(scan_id) => {
                        persistent = true;
                        result_message = format!("Scan completed and saved. scan_id = {}", scan_id);
                        Some(scan_id)
                    }

                    Err(message) => {
                        result_message =
                            format!("Scan completed, but database save failed: {}", message);
                        None
                    }
                }
            } else {
                None
            };

            sender
                .send(ScanWorkerMessage::Finished {
                    root_node,
                    scan_id,
                    persistent,
                    message: result_message,
                })
                .ok();
        });

        self.scan_receiver = Some(receiver);
        self.scan_cancel_flag = Some(cancel_flag);
    }

    fn save_scan_to_database_from_worker(
        db_path: &str,
        path: &Path,
        root_node: &FileNode,
    ) -> Result<i64, String> {
        let mut repository = DatabaseRepository::new(db_path)
            .map_err(|error| format!("Database open failed: {}", error))?;

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

        let scan_id = repository
            .save_scan(&scan)
            .map_err(|error| format!("save_scan failed: {}", error))?;

        let statistics = StatService::group_by_categories(root_node);

        repository
            .save_nodes(scan_id, root_node)
            .map_err(|error| format!("save_nodes failed: {}", error))?;

        repository
            .save_statistics(scan_id, &statistics)
            .map_err(|error| format!("save_statistics failed: {}", error))?;

        repository
            .finish_scan(scan_id, "completed")
            .map_err(|error| format!("finish_scan failed: {}", error))?;

        Ok(scan_id)
    }

    pub fn cancel_scan(&mut self) {
        if let Some(cancel_flag) = &self.scan_cancel_flag {
            cancel_flag.store(true, Ordering::Relaxed);
            self.status_message = String::from("Cancelling scan...");
        }
    }

    pub fn tick(&mut self) {
        match self.screen {
            AppScreen::Scanning => self.tick_scan(),
            AppScreen::Deleting => self.tick_delete(),
            _ => {}
        }
    }

    fn tick_scan(&mut self) {
        self.scanning_dots = (self.scanning_dots + 1) % 4;

        let message = self
            .scan_receiver
            .as_ref()
            .and_then(|receiver| receiver.try_recv().ok());

        match message {
            Some(ScanWorkerMessage::Finished {
                root_node,
                scan_id,
                persistent,
                message,
            }) => {
                self.root_node = Some(root_node);
                self.expanded_paths.clear();
                self.selected_index = 0;

                self.current_scan_id = scan_id;
                self.current_is_full_persistent_scan = persistent;

                self.rebuild_rows();

                self.scan_receiver = None;
                self.scan_cancel_flag = None;

                self.screen = AppScreen::FileTree;
                self.status_message = message;
            }

            Some(ScanWorkerMessage::Failed(message)) => {
                self.scan_receiver = None;
                self.scan_cancel_flag = None;
                self.screen = AppScreen::MainMenu;
                self.status_message = message;
            }

            Some(ScanWorkerMessage::Cancelled) => {
                self.scan_receiver = None;
                self.scan_cancel_flag = None;
                self.screen = AppScreen::MainMenu;
                self.status_message = String::from("Scan cancelled");
            }

            None => {}
        }
    }

    fn tick_delete(&mut self) {
        self.deleting_dots = (self.deleting_dots + 1) % 4;

        let message = self
            .delete_receiver
            .as_ref()
            .and_then(|receiver| receiver.try_recv().ok());

        match message {
            Some(DeleteWorkerMessage::Finished {
                root_node,
                deleted_path,
                message,
            }) => {
                self.root_node = Some(root_node);
                self.expanded_paths.remove(&deleted_path);

                self.rebuild_rows();

                if self.selected_index >= self.rows.len() {
                    self.selected_index = self.rows.len().saturating_sub(1);
                }

                self.delete_receiver = None;
                self.delete_target = None;
                self.screen = AppScreen::FileTree;
                self.status_message = message;
            }

            Some(DeleteWorkerMessage::Failed { root_node, message }) => {
                self.root_node = Some(root_node);
                self.delete_receiver = None;
                self.delete_target = None;
                self.screen = AppScreen::FileTree;
                self.status_message = message;
            }

            None => {}
        }
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

        let Some(root_node) = self.root_node.take() else {
            self.status_message = String::from("No scanned tree loaded");
            self.delete_target = None;
            self.screen = AppScreen::FileTree;
            return;
        };

        self.start_delete_async(target, root_node);
    }

    fn start_delete_async(&mut self, target: TreeRow, mut root_node: FileNode) {
        self.status_message = format!("Deleting: {}", target.path);
        self.screen = AppScreen::Deleting;
        self.deleting_dots = 0;

        let (sender, receiver) = mpsc::channel();
        let db_path = self.db_path.clone();
        let scan_id = self.current_scan_id;
        let current_is_full_persistent_scan = self.current_is_full_persistent_scan;

        thread::spawn(move || {
            if let Err(error) = DeleteService::delete_from_disk(&target.path, &target.node_type) {
                sender
                    .send(DeleteWorkerMessage::Failed {
                        root_node,
                        message: format!("Delete from disk failed: {}", error),
                    })
                    .ok();
                return;
            }

            let Some(deleted_node) =
                Self::remove_node_by_path_and_update_sizes(&mut root_node, &target.path)
            else {
                sender
                    .send(DeleteWorkerMessage::Failed {
                        root_node,
                        message: String::from("Deleted from disk, but node was not found in tree"),
                    })
                    .ok();
                return;
            };

            let deleted_statistics = StatService::group_by_categories(&deleted_node);
            let (deleted_files_count, deleted_dirs_count) = Self::count_nodes(&deleted_node);

            let message = match Self::update_database_after_delete_from_worker(
                &db_path,
                scan_id,
                current_is_full_persistent_scan,
                &target.path,
                deleted_node.size,
                deleted_files_count,
                deleted_dirs_count,
                &deleted_statistics,
            ) {
                Ok(()) => format!("Deleted: {}", target.path),
                Err(message) => format!("Deleted from disk, but {}", message),
            };

            sender
                .send(DeleteWorkerMessage::Finished {
                    root_node,
                    deleted_path: target.path,
                    message,
                })
                .ok();
        });

        self.delete_receiver = Some(receiver);
    }

    fn update_database_after_delete_from_worker(
        db_path: &str,
        scan_id: Option<i64>,
        current_is_full_persistent_scan: bool,
        deleted_path: &str,
        deleted_size: u64,
        deleted_files_count: u64,
        deleted_dirs_count: u64,
        deleted_statistics: &[FileStatistic],
    ) -> Result<(), String> {
        let Some(scan_id) = scan_id else {
            return Ok(());
        };

        let mut repository = match DatabaseRepository::new(db_path) {
            Ok(repository) => repository,
            Err(_) => {
                return Err(String::from("database open failed"));
            }
        };

        if current_is_full_persistent_scan {
            repository
                .apply_delete_to_full_scan(
                    scan_id,
                    deleted_path,
                    deleted_size,
                    deleted_files_count,
                    deleted_dirs_count,
                    deleted_statistics,
                )
                .map_err(|error| format!("database update failed: {}", error))?;
        } else {
            repository
                .delete_path_from_scan(scan_id, deleted_path)
                .map_err(|error| format!("database path delete failed: {}", error))?;
        }

        Ok(())
    }

    fn remove_node_by_path_and_update_sizes(
        node: &mut FileNode,
        target_path: &str,
    ) -> Option<FileNode> {
        let child_index = node
            .children
            .iter()
            .position(|child| child.path == target_path);

        if let Some(index) = child_index {
            let removed = node.children.remove(index);
            node.size = node.size.saturating_sub(removed.size);
            return Some(removed);
        }

        for child in &mut node.children {
            if let Some(removed) = Self::remove_node_by_path_and_update_sizes(child, target_path) {
                node.size = node.size.saturating_sub(removed.size);
                return Some(removed);
            }
        }

        None
    }

    pub fn set_sort_mode(&mut self, sort_mode: SortMode) {
        self.sort_mode = sort_mode;
        self.sort_visible_tree();
        self.rebuild_rows();
        self.status_message = format!("Sort mode: {:?}", self.sort_mode);
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

    fn sort_visible_tree(&mut self) {
        if let Some(root_node) = &mut self.root_node {
            Self::sort_visible_node(root_node, &self.expanded_paths, self.sort_mode);
        }
    }

    fn sort_visible_node(
        node: &mut FileNode,
        expanded_paths: &HashSet<String>,
        sort_mode: SortMode,
    ) {
        Self::sort_nodes(&mut node.children, sort_mode);

        if node.node_type == NodeType::Directory && expanded_paths.contains(&node.path) {
            for child in &mut node.children {
                Self::sort_visible_node(child, expanded_paths, sort_mode);
            }
        }
    }

    fn sort_nodes(children: &mut Vec<FileNode>, sort_mode: SortMode) {
        match sort_mode {
            SortMode::Name => {
                children.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            }
            SortMode::Size => children.sort_by(|a, b| b.size.cmp(&a.size)),
            SortMode::Type => children.sort_by(|a, b| {
                Self::node_type_order(&a.node_type)
                    .cmp(&Self::node_type_order(&b.node_type))
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            }),
        }
    }

    fn node_type_order(node_type: &NodeType) -> u8 {
        match node_type {
            NodeType::Directory => 0,
            NodeType::File => 1,
            NodeType::Symlink => 2,
        }
    }

    fn count_nodes(node: &FileNode) -> (u64, u64) {
        let mut files_count = 0;
        let mut dirs_count = 0;

        match &node.node_type {
            NodeType::File | NodeType::Symlink => files_count += 1,
            NodeType::Directory => dirs_count += 1,
        }

        for child in &node.children {
            let (child_files, child_dirs) = Self::count_nodes(child);
            files_count += child_files;
            dirs_count += child_dirs;
        }

        (files_count, dirs_count)
    }
    fn default_db_path() -> String {
        format!("{}/data/disk_analyzer.db", env!("CARGO_MANIFEST_DIR"))
    }
}
