use crate::models::{FileCategory, NodeType};
use crate::tui::app::{AppScreen, TreeRow, TuiApp};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &TuiApp) {
    match app.screen() {
        AppScreen::MainMenu => render_main_menu(frame, app),
        AppScreen::DirectoryScanMenu => render_directory_scan_menu(frame, app),
        AppScreen::PathInput => render_path_input(frame, app),
        AppScreen::SavedDirectoryChoice => render_saved_directory_choice(frame, app),
        AppScreen::DiskSelection => render_disk_selection(frame, app),
        AppScreen::Scanning => render_scanning(frame, app),
        AppScreen::Deleting => render_deleting(frame, app),
        AppScreen::FileTree => render_file_tree_screen(frame, app),
        AppScreen::DeleteConfirm => render_delete_confirm(frame, app),
    }
}

fn render_main_menu(frame: &mut Frame, app: &TuiApp) {
    let hidden_status = if app.show_hidden() { "on" } else { "off" };

    render_list_screen(
        frame,
        "Disk Analyzer",
        app.menu_items(),
        app.menu_index(),
        &format!(
            "Enter - select | h - hidden: {} | q / Esc - quit",
            hidden_status
        ),
    );
}

fn render_directory_scan_menu(frame: &mut Frame, app: &TuiApp) {
    let hidden_status = if app.show_hidden() { "on" } else { "off" };

    render_list_screen(
        frame,
        "Scan directory",
        app.directory_menu_items(),
        app.directory_menu_index(),
        &format!(
            "Enter - select | h - hidden: {} | Esc - back",
            hidden_status
        ),
    );
}

fn render_saved_directory_choice(frame: &mut Frame, app: &TuiApp) {
    let hidden_status = if app.show_hidden() { "on" } else { "off" };

    let title = match (app.pending_directory_path(), app.pending_scan()) {
        (Some(path), Some(scan)) => {
            if app.pending_is_disk_scan() {
                format!(
                    "Saved disk scan found | last scan: {} | {}",
                    scan.started_at, path
                )
            } else {
                format!(
                    "Saved directory version found | last scan: {} | {}",
                    scan.started_at, path
                )
            }
        }
        _ => String::from("Saved version found"),
    };

    render_list_screen(
        frame,
        &title,
        app.saved_choice_items(),
        app.saved_choice_index(),
        &format!(
            "Enter - select | h - hidden: {} | Esc - back",
            hidden_status
        ),
    );
}

fn render_list_screen(
    frame: &mut Frame,
    title: &str,
    items: &[String],
    selected_index: usize,
    help: &str,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(frame.area());

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| ListItem::new(item.clone()))
        .collect();

    let list = List::new(list_items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(selected_index));

    frame.render_stateful_widget(list, layout[0], &mut state);

    let help = Paragraph::new(help).block(Block::default().title("Help").borders(Borders::ALL));

    frame.render_widget(help, layout[1]);
}

fn render_path_input(frame: &mut Frame, app: &TuiApp) {
    let text = format!("Enter directory path:\n\n{}", app.input_path());

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Path input").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, frame.area());
}

fn render_disk_selection(frame: &mut Frame, app: &TuiApp) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(frame.area());

    let items: Vec<ListItem> = app
        .disk_rows()
        .iter()
        .map(|disk| {
            let line = format!(
                "{}. {} | {} | total: {} | free: {}",
                disk.index,
                disk.name,
                disk.mount_point,
                format_size(disk.total_space),
                format_size(disk.free_space)
            );

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Select disk").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let mut state = ListState::default();

    if !app.disk_rows().is_empty() {
        state.select(Some(app.disk_index()));
    }

    frame.render_stateful_widget(list, layout[0], &mut state);

    let hidden_status = if app.show_hidden() { "on" } else { "off" };
    let help = Paragraph::new(format!(
        "Enter - scan selected disk | h - hidden: {} | Esc - back | q - quit",
        hidden_status
    ))
    .block(Block::default().title("Help").borders(Borders::ALL));

    frame.render_widget(help, layout[1]);
}

fn render_file_tree_screen(frame: &mut Frame, app: &TuiApp) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(4)])
        .split(frame.area());

    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(main_layout[0]);

    render_tree(frame, app, content_layout[0]);
    render_info(frame, app, content_layout[1]);
    render_help(frame, app, main_layout[1]);
}

fn render_tree(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let items: Vec<ListItem> = app.rows().iter().map(row_to_list_item).collect();

    let title = format!("File tree | sort: {:?}", app.sort_mode());

    let tree = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let mut state = ListState::default();

    if !app.rows().is_empty() {
        state.select(Some(app.selected_index()));
    }

    frame.render_stateful_widget(tree, area, &mut state);
}

fn render_info(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let text = match app.selected_row() {
        Some(row) => build_info_text(row),
        None => String::from("No selected item"),
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Info").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_help(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let hidden_status = if app.show_hidden() { "on" } else { "off" };
    let system_dirs_status = if app.scan_system_dirs() { "on" } else { "off" };

    let help_text = format!(
        "Enter - expand/collapse | d - delete | n - sort name | s - sort size | t - sort type | h - hidden: {} | system dirs: {} | Esc - menu | q - quit\nStatus: {}",
        hidden_status,
        system_dirs_status,
        app.status_message()
    );
    let help =
        Paragraph::new(help_text).block(Block::default().title("Help").borders(Borders::ALL));

    frame.render_widget(help, area);
}

fn render_delete_confirm(frame: &mut Frame, app: &TuiApp) {
    let text = match app.delete_target() {
        Some(target) => format!(
            "Delete selected item?\n\n{}\n\nEnter - confirm\nEsc - cancel",
            target.path
        ),
        None => String::from("No item selected"),
    };

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Confirm delete")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, frame.area());
}

fn row_to_list_item(row: &TreeRow) -> ListItem<'static> {
    let indent = "  ".repeat(row.depth);

    let icon = match &row.node_type {
        NodeType::Directory => {
            if row.children_count == 0 {
                "[D]"
            } else if row.is_expanded {
                "[-]"
            } else {
                "[+]"
            }
        }
        NodeType::File => "[F]",
        NodeType::Symlink => "[L]",
    };

    let line = Line::from(vec![
        Span::raw(indent),
        Span::raw(icon),
        Span::raw(" "),
        Span::raw(row.name.clone()),
        Span::raw("  "),
        Span::raw(format_size(row.size)),
    ]);

    ListItem::new(line)
}

fn build_info_text(row: &TreeRow) -> String {
    let node_type = match &row.node_type {
        NodeType::Directory => "Directory",
        NodeType::File => "File",
        NodeType::Symlink => "Symlink",
    };

    let extension = row.extension.clone().unwrap_or_else(|| String::from("-"));

    let category = match row.category {
        Some(category) => category_to_string(category),
        None => String::from("-"),
    };

    let expanded = if row.node_type == NodeType::Directory {
        if row.is_expanded {
            "yes"
        } else {
            "no"
        }
    } else {
        "-"
    };

    format!(
        "Name: {}\nPath: {}\nType: {}\nExtension: {}\nCategory: {}\nSize: {}\nChildren: {}\nExpanded: {}",
        row.name,
        row.path,
        node_type,
        extension,
        category,
        format_size(row.size),
        row.children_count,
        expanded
    )
}

fn category_to_string(category: FileCategory) -> String {
    category.as_str().to_string()
}

fn format_size(size: u64) -> String {
    let kb = 1024.0;
    let mb = kb * 1024.0;
    let gb = mb * 1024.0;

    let size = size as f64;

    if size >= gb {
        format!("{:.2} GB", size / gb)
    } else if size >= mb {
        format!("{:.2} MB", size / mb)
    } else if size >= kb {
        format!("{:.2} KB", size / kb)
    } else {
        format!("{:.0} B", size)
    }
}

fn render_scanning(frame: &mut Frame, app: &TuiApp) {
    let dots = ".".repeat(app.scanning_dots());

    let text = format!(
        "Scanning{}\n\n{}\n\nEsc - stop scanning\nq - stop and quit",
        dots,
        app.status_message()
    );

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Scanning").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, frame.area());
}

fn render_deleting(frame: &mut Frame, app: &TuiApp) {
    let dots = ".".repeat(app.deleting_dots());

    let text = format!("Deleting{}\n\n{}", dots, app.status_message());

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Deleting").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, frame.area());
}
