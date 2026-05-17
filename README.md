# Disk Analyzer

Disk Analyzer is a Rust terminal application for scanning directories and
inspecting disk usage. It builds an in-memory file tree, calculates directory
sizes, groups files by category, stores scan history in SQLite, and provides a
keyboard-driven TUI for navigation.

## Features

- scan a selected directory or disk mount point;
- skip hidden files by default, with an option to include them;
- optionally include system directories during scanning;
- calculate file and directory sizes;
- sort visible tree rows by name, size, or type;
- group files into categories such as video, audio, documents, code, archives,
  images, no extension, and other;
- save scan results and statistics to SQLite;
- open saved scan results and rescan paths;
- delete selected files or directories with confirmation.

## Requirements

- Rust toolchain with Cargo;
- a terminal that supports alternate screen mode.

## Build And Run

```bash
cargo build
cargo run
```

Run the test suite with:

```bash
cargo test
```

## TUI Controls

- `Enter` - select menu item, open directory, or confirm action;
- `Esc` - go back, cancel scanning, or cancel delete confirmation;
- `q` - quit the application;
- `Up` / `k` - move selection up;
- `Down` / `j` - move selection down;
- `h` - toggle hidden files mode;
- `n` - sort tree by name;
- `s` - sort tree by size;
- `t` - sort tree by type;
- `d` - request deletion of the selected tree item.

The application starts in the main TUI menu. From there, choose a directory
scan, select a disk mount point, or toggle system directory scanning.
