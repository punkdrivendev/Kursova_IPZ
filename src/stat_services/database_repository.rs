use crate::models::*;
use rusqlite::{params, Connection, Result};

pub struct DatabaseRepository {
    connection: Connection,
}

impl DatabaseRepository {
    pub fn new(database_path: &str) -> Result<Self> {
        let connection = Connection::open(database_path)?;

        connection.execute("PRAGMA foreign_keys = ON;", [])?;

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

    pub fn save_nodes(&self, scan_id: i64, root: &FileNode) -> Result<()> {
        self.save_node_recursive(scan_id, None, root)?;

        Ok(())
    }

    fn save_node_recursive(
        &self,
        scan_id: i64,
        parent_id: Option<i64>,
        node: &FileNode,
    ) -> Result<i64> {
        self.connection.execute(
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

        let current_id = self.connection.last_insert_rowid();

        for child in &node.children {
            self.save_node_recursive(scan_id, Some(current_id), child)?;
        }

        Ok(current_id)
    }

    pub fn save_statistics(
        &self,
        scan_id: i64,
        statistics: &[FileStatistic],
    ) -> Result<()> {
        for stat in statistics {
            self.connection.execute(
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
}