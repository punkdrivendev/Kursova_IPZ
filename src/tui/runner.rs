use crate::stat_services::system_info_service::SystemInfoService;
use crate::tui::app::{AppScreen, DiskRow, SortMode, TuiApp};
use crate::tui::ui;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::PathBuf, time::Duration};

pub fn run_tui() -> io::Result<()> {
    let disk_rows = load_disk_rows();
    let app = TuiApp::new(disk_rows);

    run_app(app)
}

pub fn run_app(mut app: TuiApp) -> io::Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    terminal.show_cursor()?;

    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            ui::render(frame, app);
        })?;

        if app.should_quit() {
            break;
        }

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                handle_key(app, key.code)?;
            }
        }
    }

    Ok(())
}

fn handle_key(app: &mut TuiApp, key_code: KeyCode) -> io::Result<()> {
    match app.screen() {
        AppScreen::MainMenu => handle_main_menu_key(app, key_code)?,
        AppScreen::DirectoryScanMenu => handle_directory_scan_menu_key(app, key_code)?,
        AppScreen::PathInput => handle_path_input_key(app, key_code)?,
        AppScreen::SavedDirectoryChoice => handle_saved_directory_choice_key(app, key_code)?,
        AppScreen::DiskSelection => handle_disk_selection_key(app, key_code)?,
        AppScreen::FileTree => handle_file_tree_key(app, key_code)?,
        AppScreen::DeleteConfirm => handle_delete_confirm_key(app, key_code)?,
    }

    Ok(())
}

fn handle_main_menu_key(app: &mut TuiApp, key_code: KeyCode) -> io::Result<()> {
    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),

        KeyCode::Down | KeyCode::Char('j') => app.next(),

        KeyCode::Up | KeyCode::Char('k') => app.previous(),

        KeyCode::Enter => match app.menu_index() {
            0 => app.set_screen(AppScreen::DirectoryScanMenu),
            1 => app.set_screen(AppScreen::DiskSelection),
            2 => app.toggle_system_dirs_mode(),
            3 => app.quit(),
            _ => {}
        },

        _ => {}
    }

    Ok(())
}

fn handle_directory_scan_menu_key(
    app: &mut TuiApp,
    key_code: KeyCode,
) -> io::Result<()> {
    match key_code {
        KeyCode::Esc => app.set_screen(AppScreen::MainMenu),

        KeyCode::Down | KeyCode::Char('j') => app.next(),

        KeyCode::Up | KeyCode::Char('k') => app.previous(),

        KeyCode::Enter => match app.directory_menu_index() {
            0 => app.request_current_directory_scan(),

            1 => {
                app.clear_input();
                app.set_screen(AppScreen::PathInput);
            }

            2 => app.set_screen(AppScreen::MainMenu),

            _ => {}
        },

        _ => {}
    }

    Ok(())
}

fn handle_path_input_key(app: &mut TuiApp, key_code: KeyCode) -> io::Result<()> {
    match key_code {
        KeyCode::Esc => {
            app.clear_input();
            app.set_screen(AppScreen::DirectoryScanMenu);
        }

        KeyCode::Enter => {
            let path = PathBuf::from(app.input_path());
            app.request_directory_scan(path);
        }

        KeyCode::Backspace => {
            app.pop_input_char();
        }

        KeyCode::Char(ch) => {
            app.push_input_char(ch);
        }

        _ => {}
    }

    Ok(())
}

fn handle_saved_directory_choice_key(
    app: &mut TuiApp,
    key_code: KeyCode,
) -> io::Result<()> {
    match key_code {
        KeyCode::Esc => app.set_screen(AppScreen::DirectoryScanMenu),

        KeyCode::Down | KeyCode::Char('j') => app.next(),

        KeyCode::Up | KeyCode::Char('k') => app.previous(),

        KeyCode::Enter => match app.saved_choice_index() {
            0 => app.open_saved_directory_version(),
            1 => app.scan_pending_path_again(),
            2 => app.set_screen(AppScreen::DirectoryScanMenu),
            _ => {}
        },

        _ => {}
    }

    Ok(())
}

fn handle_disk_selection_key(app: &mut TuiApp, key_code: KeyCode) -> io::Result<()> {
    match key_code {
        KeyCode::Char('q') => app.quit(),

        KeyCode::Esc => app.set_screen(AppScreen::MainMenu),

        KeyCode::Down | KeyCode::Char('j') => app.next(),

        KeyCode::Up | KeyCode::Char('k') => app.previous(),

        KeyCode::Enter => {
            if let Some(mount_point) = app.selected_disk_mount_point() {
                let path = PathBuf::from(mount_point);
                app.request_disk_scan(path);
            }
        }

        _ => {}
    }

    Ok(())
}

fn handle_file_tree_key(app: &mut TuiApp, key_code: KeyCode) -> io::Result<()> {
    match key_code {
        KeyCode::Char('q') => app.quit(),

        KeyCode::Esc => app.set_screen(AppScreen::MainMenu),

        KeyCode::Down | KeyCode::Char('j') => app.next(),

        KeyCode::Up | KeyCode::Char('k') => app.previous(),

        KeyCode::Enter => app.toggle_selected_directory(),

        KeyCode::Char('d') => app.request_delete_selected(),

        KeyCode::Char('n') => app.set_sort_mode(SortMode::Name),

        KeyCode::Char('s') => app.set_sort_mode(SortMode::Size),

        KeyCode::Char('t') => app.set_sort_mode(SortMode::Type),

        KeyCode::Char('h') => app.toggle_hidden_mode(),

        _ => {}
    }

    Ok(())
}

fn handle_delete_confirm_key(app: &mut TuiApp, key_code: KeyCode) -> io::Result<()> {
    match key_code {
        KeyCode::Esc => app.cancel_delete(),

        KeyCode::Enter => app.confirm_delete(),

        _ => {}
    }

    Ok(())
}

fn load_disk_rows() -> Vec<DiskRow> {
    let system_info = SystemInfoService::new();

    system_info
        .get_available_disks()
        .iter()
        .enumerate()
        .map(|(index, disk)| DiskRow {
            index,
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            total_space: disk.total_space(),
            free_space: disk.available_space(),
        })
        .collect()
}