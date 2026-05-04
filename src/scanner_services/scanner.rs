
/*
Модуль скануваня файлової системи
Рекурсовино обходить директорії
Визначає типи об'єктів файлової системи та передає 
знайдені дані на подальшу обробку.
*/
use crate::models::{FileNode, NodeType};
use std::fs;
use std::path::Path;

pub struct Scanner {
    pub show_hidden: bool, // чи показувати приховані файли та директорії
}

impl Scanner {
    pub fn new(show_hidden: bool) -> Self {
        Self {
            show_hidden, // 
        }
    }

    pub fn scan(&self, path: &Path) -> Option<FileNode> {  // функція сканування одного шляху
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                println!("Cannot read metadata {:?}: {}", path, error); // помилка читання метаданних
                return None;
            }
        };

        let name = path  
            .file_name() // бере останню частину шляху: назву файлу або директорії
            .map(|name| name.to_string_lossy().to_string()) // якщо назва є, перетворюємо її у String
            .unwrap_or_else(|| path.to_string_lossy().to_string()); // якщо назви немає, використовуємо повний шлях

        if !self.show_hidden && name.starts_with('.') {
            return None;  // якщо приховані файли вимкнені, пропускаємо елементи, назва яких починається з крапки
        }

        let extension = path
            .extension()  // отримує розширення файлу, якщо воно є
            .map(|ext| ext.to_string_lossy().to_string()); // якщо розширення є, перетворюємо його у String

        if metadata.is_file() { // якщо поточний шлях є файлом, створюємо FileNode з даними цього файлу
            return Some(FileNode {
                id: None,
                parent_id: None,
                path: path.to_string_lossy().to_string(),
                name,
                extension,
                size: metadata.len(),
                node_type: NodeType::File,
                children: Vec::new(),
                category: None,
            });
        }

        if metadata.is_dir() { //якщо поточних шлях є директорією
            let mut children = Vec::new(); // список дочірніх файлів і директорій
            let mut total_size = 0; // сумарний розмір вмісту директорії у байтах

            let entries = match fs::read_dir(path) {
                Ok(entries) => entries,
                Err(error) => {
                    println!("Cannot read directory {:?}: {}", path, error);

                    return Some(FileNode {
                        id: None,
                        parent_id: None,
                        path: path.to_string_lossy().to_string(),
                        name,
                        extension: None,
                        size: 0,
                        node_type: NodeType::Directory,
                        children,
                        category: None,
                    });
                }
            };

            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        println!("Cannot read entry: {}", error);
                        continue;
                    }
                };

                let child_path = entry.path(); // шлях до дочірнього елемента

                if let Some(child_node) = self.scan(&child_path) {
                    total_size += child_node.size;  // додаємо розмір дочірнього елемента до розміру директорії
                    children.push(child_node); // додаємо знайдений файл або папку у список children поточної директорії
                }
            }

            return Some(FileNode {
                id: None,
                parent_id: None,
                path: path.to_string_lossy().to_string(),
                name,
                extension: None,
                size: total_size,
                node_type: NodeType::Directory,
                children,
                category: None,
            });
        }

        None
    }
}