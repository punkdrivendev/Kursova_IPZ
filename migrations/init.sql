PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS scans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    root_path TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    total_size INTEGER DEFAULT 0,
    files_count INTEGER DEFAULT 0,
    dirs_count INTEGER DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'running'
);

CREATE TABLE IF NOT EXISTS file_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id INTEGER NOT NULL,
    parent_id INTEGER,
    path TEXT NOT NULL,
    name TEXT NOT NULL,
    extension TEXT,
    size INTEGER NOT NULL,
    node_type TEXT NOT NULL,
    percentage REAL DEFAULT 0,

    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES file_nodes(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS file_statistics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id INTEGER NOT NULL,
    category TEXT NOT NULL,
    files_count INTEGER DEFAULT 0,
    total_size INTEGER DEFAULT 0,
    percentage REAL DEFAULT 0,

    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS scan_errors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    error_kind TEXT NOT NULL,
    message TEXT,

    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);