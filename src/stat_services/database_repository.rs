use crate::models::*;
use rusqlite::{params, Connection, OptionalExtension, Result, Transaction};
use std::collections::HashMap;
use std::path::Path;

pub struct DatabaseRepository {
    connection: Connection,
}

#[derive(Debug, Clone)]
struct DbFileNode {
    id: i64,
    parent_id: Option<i64>,
    path: String,
    name: String,
    extension: Option<String>,
    size: u64,
    node_type: NodeType,
}

impl DatabaseRepository {
    pub fn new(database_path: &str) -> Result<Self> {
        if let Some(parent) = Path::new(database_path).parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let connection = Connection::open(database_path)?;

        connection.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA cache_size = -64000;
            ",
        )?;

        let schema = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/migrations/init.sql"
        ));

        connection.execute_batch(schema)?;

        Ok(Self { connection })
    }

    pub fn save_scan(&self, scan: &ScanSession) -> Result<i64> {
        self.connection.execute(
            "
            INSERT INTO scans (
                root_path,
                started_at,
                finished_at,
                total_size,
                files_count,
                dirs_count,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                scan.root_path,
                scan.started_at,
                scan.finished_at,
                scan.total_size as i64,
                scan.files_count as i64,
                scan.dirs_count as i64,
                scan.status,
            ],
        )?;

        Ok(self.connection.last_insert_rowid())
    }

    pub fn finish_scan(&self, scan_id: i64, status: &str) -> Result<()> {
        self.connection.execute(
            "
            UPDATE scans
            SET finished_at = datetime('now'),
                status = ?1
            WHERE id = ?2
            ",
            params![status, scan_id],
        )?;

        Ok(())
    }

    pub fn load_scan(&self, scan_id: i64) -> Result<ScanSession> {
        self.connection.query_row(
            "
            SELECT
                id,
                root_path,
                started_at,
                finished_at,
                total_size,
                files_count,
                dirs_count,
                status
            FROM scans
            WHERE id = ?1
            ",
            params![scan_id],
            |row| {
                Ok(ScanSession {
                    id: Some(row.get(0)?),
                    root_path: row.get(1)?,
                    started_at: row.get(2)?,
                    finished_at: row.get(3)?,
                    total_size: row.get::<_, i64>(4)? as u64,
                    files_count: row.get::<_, i64>(5)? as u64,
                    dirs_count: row.get::<_, i64>(6)? as u64,
                    status: row.get(7)?,
                })
            },
        )
    }

    pub fn find_latest_scan_by_root_path(&self, root_path: &str) -> Result<Option<ScanSession>> {
        let normalized_path = Self::normalize_path(root_path);

        self.connection
            .query_row(
                "
                SELECT
                    id,
                    root_path,
                    started_at,
                    finished_at,
                    total_size,
                    files_count,
                    dirs_count,
                    status
                FROM scans
                WHERE status = 'completed'
                  AND (
                        root_path = ?1
                        OR root_path = ?2
                        OR rtrim(root_path, '/') = rtrim(?2, '/')
                  )
                ORDER BY started_at DESC, id DESC
                LIMIT 1
                ",
                params![root_path, normalized_path],
                |row| {
                    Ok(ScanSession {
                        id: Some(row.get(0)?),
                        root_path: row.get(1)?,
                        started_at: row.get(2)?,
                        finished_at: row.get(3)?,
                        total_size: row.get::<_, i64>(4)? as u64,
                        files_count: row.get::<_, i64>(5)? as u64,
                        dirs_count: row.get::<_, i64>(6)? as u64,
                        status: row.get(7)?,
                    })
                },
            )
            .optional()
    }

    pub fn find_latest_scan_containing_path(&self, path: &str) -> Result<Option<ScanSession>> {
        let normalized_path = Self::normalize_path(path);

        self.connection
            .query_row(
                "
                SELECT
                    s.id,
                    s.root_path,
                    s.started_at,
                    s.finished_at,
                    s.total_size,
                    s.files_count,
                    s.dirs_count,
                    s.status
                FROM scans s
                JOIN file_nodes f ON f.scan_id = s.id
                WHERE s.status = 'completed'
                  AND f.node_type = 'directory'
                  AND (
                        f.path = ?1
                        OR f.path = ?2
                        OR rtrim(f.path, '/') = rtrim(?2, '/')
                  )
                ORDER BY s.started_at DESC, s.id DESC
                LIMIT 1
                ",
                params![path, normalized_path],
                |row| {
                    Ok(ScanSession {
                        id: Some(row.get(0)?),
                        root_path: row.get(1)?,
                        started_at: row.get(2)?,
                        finished_at: row.get(3)?,
                        total_size: row.get::<_, i64>(4)? as u64,
                        files_count: row.get::<_, i64>(5)? as u64,
                        dirs_count: row.get::<_, i64>(6)? as u64,
                        status: row.get(7)?,
                    })
                },
            )
            .optional()
    }

    pub fn save_nodes(&mut self, scan_id: i64, root: &FileNode) -> Result<()> {
        let tx = self.connection.transaction()?;

        Self::save_node_recursive_tx(&tx, scan_id, None, root)?;

        tx.commit()?;

        Ok(())
    }

    fn save_node_recursive_tx(
        tx: &Transaction<'_>,
        scan_id: i64,
        parent_id: Option<i64>,
        node: &FileNode,
    ) -> Result<i64> {
        tx.execute(
            "
            INSERT INTO file_nodes (
                scan_id,
                parent_id,
                path,
                name,
                extension,
                size,
                node_type,
                percentage
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ",
            params![
                scan_id,
                parent_id,
                node.path,
                node.name,
                node.extension,
                node.size as i64,
                node.node_type.as_str(),
                0.0,
            ],
        )?;

        let current_id = tx.last_insert_rowid();

        for child in &node.children {
            Self::save_node_recursive_tx(tx, scan_id, Some(current_id), child)?;
        }

        Ok(current_id)
    }

    pub fn save_statistics(&mut self, scan_id: i64, statistics: &[FileStatistic]) -> Result<()> {
        let tx = self.connection.transaction()?;

        Self::save_statistics_tx(&tx, scan_id, statistics)?;

        tx.commit()?;

        Ok(())
    }

    fn save_statistics_tx(
        tx: &Transaction<'_>,
        scan_id: i64,
        statistics: &[FileStatistic],
    ) -> Result<()> {
        for stat in statistics {
            tx.execute(
                "
                INSERT INTO file_statistics (
                    scan_id,
                    category,
                    files_count,
                    total_size,
                    percentage
                )
                VALUES (?1, ?2, ?3, ?4, ?5)
                ",
                params![
                    scan_id,
                    stat.category.as_str(),
                    stat.file_count as i64,
                    stat.total_size as i64,
                    stat.percentage,
                ],
            )?;
        }

        Ok(())
    }

    pub fn replace_statistics(&mut self, scan_id: i64, statistics: &[FileStatistic]) -> Result<()> {
        self.connection.execute(
            "DELETE FROM file_statistics WHERE scan_id = ?1",
            params![scan_id],
        )?;

        self.save_statistics(scan_id, statistics)
    }

    pub fn replace_scan_data(
        &mut self,
        scan_id: i64,
        root: &FileNode,
        statistics: &[FileStatistic],
        files_count: u64,
        dirs_count: u64,
    ) -> Result<()> {
        self.connection.execute(
            "DELETE FROM file_nodes WHERE scan_id = ?1",
            params![scan_id],
        )?;

        self.connection.execute(
            "DELETE FROM file_statistics WHERE scan_id = ?1",
            params![scan_id],
        )?;

        self.save_nodes(scan_id, root)?;
        self.save_statistics(scan_id, statistics)?;

        self.update_scan_summary(scan_id, root.size, files_count, dirs_count)?;

        Ok(())
    }

    pub fn update_scan_summary(
        &self,
        scan_id: i64,
        total_size: u64,
        files_count: u64,
        dirs_count: u64,
    ) -> Result<()> {
        self.connection.execute(
            "
            UPDATE scans
            SET total_size = ?1,
                files_count = ?2,
                dirs_count = ?3
            WHERE id = ?4
            ",
            params![
                total_size as i64,
                files_count as i64,
                dirs_count as i64,
                scan_id,
            ],
        )?;

        Ok(())
    }

    pub fn delete_path_from_scan(&self, scan_id: i64, path: &str) -> Result<()> {
        let normalized_path = Self::normalize_path(path);
        let like_pattern = format!("{}/%", normalized_path.trim_end_matches('/'));

        self.connection.execute(
            "
            DELETE FROM file_nodes
            WHERE scan_id = ?1
              AND (
                    path = ?2
                    OR path = ?3
                    OR path LIKE ?4
              )
            ",
            params![scan_id, path, normalized_path, like_pattern],
        )?;

        Ok(())
    }

    pub fn apply_delete_to_full_scan(
        &mut self,
        scan_id: i64,
        deleted_path: &str,
        deleted_size: u64,
        deleted_files_count: u64,
        deleted_dirs_count: u64,
        deleted_statistics: &[FileStatistic],
    ) -> Result<()> {
        let normalized_path = Self::normalize_path(deleted_path);
        let like_pattern = format!("{}/%", normalized_path.trim_end_matches('/'));
        let tx = self.connection.transaction()?;

        tx.execute(
            "
            DELETE FROM file_nodes
            WHERE scan_id = ?1
              AND (
                    path = ?2
                    OR path = ?3
                    OR path LIKE ?4
              )
            ",
            params![scan_id, deleted_path, normalized_path, like_pattern],
        )?;

        Self::subtract_size_from_ancestor_nodes_tx(&tx, scan_id, deleted_path, deleted_size)?;

        for statistic in deleted_statistics {
            tx.execute(
                "
                UPDATE file_statistics
                SET files_count = MAX(files_count - ?1, 0),
                    total_size = MAX(total_size - ?2, 0)
                WHERE scan_id = ?3 AND category = ?4
                ",
                params![
                    statistic.file_count as i64,
                    statistic.total_size as i64,
                    scan_id,
                    statistic.category.as_str(),
                ],
            )?;
        }

        tx.execute(
            "
            DELETE FROM file_statistics
            WHERE scan_id = ?1
              AND files_count <= 0
              AND total_size <= 0
            ",
            params![scan_id],
        )?;

        tx.execute(
            "
            UPDATE scans
            SET total_size = MAX(total_size - ?1, 0),
                files_count = MAX(files_count - ?2, 0),
                dirs_count = MAX(dirs_count - ?3, 0)
            WHERE id = ?4
            ",
            params![
                deleted_size as i64,
                deleted_files_count as i64,
                deleted_dirs_count as i64,
                scan_id,
            ],
        )?;

        tx.execute(
            "
            UPDATE file_statistics
            SET percentage = CASE
                WHEN (
                    SELECT COALESCE(SUM(total_size), 0)
                    FROM file_statistics
                    WHERE scan_id = ?1
                ) = 0 THEN 0
                ELSE total_size * 100.0 / (
                    SELECT COALESCE(SUM(total_size), 0)
                    FROM file_statistics
                    WHERE scan_id = ?1
                )
            END
            WHERE scan_id = ?1
            ",
            params![scan_id],
        )?;

        tx.commit()?;

        Ok(())
    }

    fn subtract_size_from_ancestor_nodes_tx(
        tx: &Transaction<'_>,
        scan_id: i64,
        deleted_path: &str,
        deleted_size: u64,
    ) -> Result<()> {
        let mut current = Path::new(deleted_path).parent();

        while let Some(path) = current {
            let path_string = path.to_string_lossy().to_string();

            tx.execute(
                "
                UPDATE file_nodes
                SET size = MAX(size - ?1, 0)
                WHERE scan_id = ?2 AND path = ?3
                ",
                params![deleted_size as i64, scan_id, path_string],
            )?;

            current = path.parent();
        }

        Ok(())
    }

    pub fn load_subtree_from_path(&self, scan_id: i64, root_path: &str) -> Result<FileNode> {
        let normalized_path = Self::normalize_path(root_path);

        let root_id: i64 = self.connection.query_row(
            "
            SELECT id
            FROM file_nodes
            WHERE scan_id = ?1
              AND (
                    path = ?2
                    OR path = ?3
                    OR rtrim(path, '/') = rtrim(?3, '/')
              )
            LIMIT 1
            ",
            params![scan_id, root_path, normalized_path],
            |row| row.get(0),
        )?;

        let mut statement = self.connection.prepare(
            "
            SELECT
                id,
                parent_id,
                path,
                name,
                extension,
                size,
                node_type
            FROM file_nodes
            WHERE scan_id = ?1
            ORDER BY id
            ",
        )?;

        let rows = statement.query_map(params![scan_id], |row| {
            let node_type_text: String = row.get(6)?;

            Ok(DbFileNode {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                path: row.get(2)?,
                name: row.get(3)?,
                extension: row.get(4)?,
                size: row.get::<_, i64>(5)? as u64,
                node_type: Self::parse_node_type(&node_type_text),
            })
        })?;

        let mut nodes_by_id: HashMap<i64, DbFileNode> = HashMap::new();
        let mut children_by_parent: HashMap<Option<i64>, Vec<i64>> = HashMap::new();

        for row in rows {
            let node = row?;

            children_by_parent
                .entry(node.parent_id)
                .or_default()
                .push(node.id);

            nodes_by_id.insert(node.id, node);
        }

        Ok(Self::build_tree_from_db_nodes(
            root_id,
            &nodes_by_id,
            &children_by_parent,
        ))
    }

    fn build_tree_from_db_nodes(
        node_id: i64,
        nodes_by_id: &HashMap<i64, DbFileNode>,
        children_by_parent: &HashMap<Option<i64>, Vec<i64>>,
    ) -> FileNode {
        let db_node = nodes_by_id.get(&node_id).expect("Node must exist").clone();

        let child_ids = children_by_parent
            .get(&Some(node_id))
            .cloned()
            .unwrap_or_default();

        let mut children = Vec::new();

        for child_id in child_ids {
            children.push(Self::build_tree_from_db_nodes(
                child_id,
                nodes_by_id,
                children_by_parent,
            ));
        }

        FileNode {
            id: Some(db_node.id),
            parent_id: db_node.parent_id,
            path: db_node.path,
            name: db_node.name,
            extension: db_node.extension,
            size: db_node.size,
            node_type: db_node.node_type,
            children,
            category: None,
        }
    }

    fn parse_node_type(value: &str) -> NodeType {
        match value {
            "file" => NodeType::File,
            "directory" => NodeType::Directory,
            "symlink" => NodeType::Symlink,
            _ => NodeType::File,
        }
    }

    fn normalize_path(path: &str) -> String {
        if path == "/" {
            String::from("/")
        } else {
            path.trim_end_matches('/').to_string()
        }
    }
}
