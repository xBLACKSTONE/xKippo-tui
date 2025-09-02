mod components;
mod dashboard;
mod logs;
mod sessions;
mod settings;
mod help;
mod security;
mod geography;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::app::{App, AppEvent, AppState};

// Re-export for easy access
pub use components::*;
pub use dashboard::*;
pub use logs::*;
pub use sessions::*;
pub use settings::*;
pub use help::*;
pub use security::*;
pub use geography::*;

/// Starts the UI event loop
pub fn start_ui(mut app: App) -> Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create UI event channel
    let (ui_tx, mut ui_rx) = mpsc::channel(100);
    
    // Spawn input handling thread
    let ui_tx_clone = ui_tx.clone();
    std::thread::spawn(move || {
        let tick_rate = Duration::from_millis(200);
        let mut last_tick = Instant::now();
        
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            
            if event::poll(timeout).unwrap() {
                if let Ok(event) = event::read() {
                    if let Err(_) = ui_tx_clone.blocking_send(UIEvent::Input(event)) {
                        break;
                    }
                }
            }
            
            if last_tick.elapsed() >= tick_rate {
                if let Err(_) = ui_tx_clone.blocking_send(UIEvent::Tick) {
                    break;
                }
                last_tick = Instant::now();
            }
        }
    });

    // Start connecting to honeypot
    app.connect().await?;
    
    // Subscribe to application events
    let mut app_events = app.event_tx.subscribe();
    
    // Spawn app event handling task
    let ui_tx_clone = ui_tx.clone();
    tokio::spawn(async move {
        while let Ok(event) = app_events.recv().await {
            if let Err(_) = ui_tx_clone.send(UIEvent::AppEvent(event)).await {
                break;
            }
        }
    });

    // Main event loop
    terminal.draw(|f| ui(f, &app))?;

    while app.state != AppState::ShuttingDown {
        match ui_rx.recv().await {
            Some(UIEvent::Input(event)) => {
                if !handle_input(event, &mut app).await? {
                    break;
                }
            }
            Some(UIEvent::Tick) => {
                app.update()?;
            }
            Some(UIEvent::AppEvent(event)) => {
                handle_app_event(event, &mut app).await?;
            }
            None => break,
        }
        
        terminal.draw(|f| ui(f, &app))?;
    }

    // Clean up
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    Ok(())
}

/// UI event types
enum UIEvent {
    Input(Event),
    Tick,
    AppEvent(AppEvent),
}

/// Handle input events
async fn handle_input(event: Event, app: &mut App) -> Result<bool> {
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return Ok(true);
        }

        match key.code {
            KeyCode::Char('q') => {
                app.quit().await?;
                return Ok(false);
            }
            KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                app.quit().await?;
                return Ok(false);
            }
            KeyCode::Tab => {
                app.selected_tab = (app.selected_tab + 1) % 6;
            }
            KeyCode::BackTab => {
                app.selected_tab = (app.selected_tab + 5) % 6;
            }
            KeyCode::Left => {
                if app.selected_tab > 0 {
                    app.selected_tab -= 1;
                } else {
                    app.selected_tab = 5;
                }
            }
            KeyCode::Right => {
                app.selected_tab = (app.selected_tab + 1) % 6;
            }
            KeyCode::Char('1') => app.selected_tab = 0,
            KeyCode::Char('2') => app.selected_tab = 1,
            KeyCode::Char('3') => app.selected_tab = 2,
            KeyCode::Char('4') => app.selected_tab = 3,
            KeyCode::Char('5') => app.selected_tab = 4,
            KeyCode::Char('6') => app.selected_tab = 5,
            _ => {
                // Handle tab-specific input
                match app.selected_tab {
                    0 => handle_dashboard_input(key, app).await?,
                    1 => handle_security_input(key, app).await?,
                    2 => handle_logs_input(key, app).await?,
                    3 => handle_sessions_input(key, app).await?,
                    4 => handle_geography_input(key, app).await?,
                    5 => handle_settings_input(key, app).await?,
                    _ => {}
                }
            }
        }
    } else if let Event::Mouse(_) = event {
        // Handle mouse events
    }

    Ok(true)
}

/// Handle application events
async fn handle_app_event(event: AppEvent, app: &mut App) -> Result<()> {
    match event {
        AppEvent::Quit => {
            app.state = AppState::ShuttingDown;
        }
        _ => {}
    }

    Ok(())
}

/// Main UI layout and rendering
fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ].as_ref())
        .split(size);
    
    // Create tabs
    let titles = vec!["Dashboard", "Security", "Logs", "Sessions", "Geography", "Settings"];
    let tabs = Tabs::new(titles.iter().map(|t| Span::styled(*t, Style::default())).collect())
        .select(app.selected_tab)
        .block(Block::default().title("xKippo Honeypot Monitor").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    
    f.render_widget(tabs, chunks[0]);
    
    // Render the selected tab
    match app.selected_tab {
        0 => render_dashboard(f, app, chunks[1]),
        1 => render_security_dashboard(f, app, chunks[1]),
        2 => render_logs(f, app, chunks[1]),
        3 => render_sessions(f, app, chunks[1]),
        4 => render_geography(f, app, chunks[1]),
        5 => render_settings(f, app, chunks[1]),
        _ => {}
    }
    
    // Render status bar
    render_status_bar(f, app, chunks[2]);
}

/// Render the status bar at the bottom of the screen
fn render_status_bar<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let status = format!(
        "{} | {} sessions | {} logs | Press '?' for help",
        match app.connection_status {
            crate::app::ConnectionStatus::Disconnected => "Not Connected",
            crate::app::ConnectionStatus::Connecting => "Connecting...",
            crate::app::ConnectionStatus::Connected => "Connected",
            crate::app::ConnectionStatus::Failed(_) => "Connection Failed",
        },
        app.store.try_read().map_or(0, |store| store.get_session_count()),
        app.store.try_read().map_or(0, |store| store.get_log_entry_count()),
    );
    
    let text = ratatui::text::Line::from(status);
    let status_bar = ratatui::widgets::Paragraph::new(text)
        .style(Style::default().fg(Color::White).bg(Color::Black));
    
    f.render_widget(status_bar, area);
}

// Tab-specific input handlers
async fn handle_dashboard_input(key: event::KeyEvent, app: &mut App) -> Result<()> {
    // Implementation will be added later
    Ok(())
}

async fn handle_logs_input(key: event::KeyEvent, app: &mut App) -> Result<()> {
    // Implementation will be added later
    Ok(())
}

async fn handle_sessions_input(key: event::KeyEvent, app: &mut App) -> Result<()> {
    // Implementation will be added later
    Ok(())
}

async fn handle_settings_input(key: event::KeyEvent, app: &mut App) -> Result<()> {
    // Implementation will be added later
    Ok(())
}

/// Handle input for the security dashboard
async fn handle_security_input(key: event::KeyEvent, app: &mut App) -> Result<()> {
    match key.code {
        KeyCode::Char('f') => {
            // Filter by risk score
        },
        KeyCode::Char('s') => {
            // Sort by different metrics
        },
        KeyCode::Char('t') => {
            // Toggle different views
        },
        _ => {}
    }
    Ok(())
}

/// Handle input for the geography view
async fn handle_geography_input(key: event::KeyEvent, app: &mut App) -> Result<()> {
    match key.code {
        KeyCode::Char('z') => {
            // Zoom in
        },
        KeyCode::Char('Z') => {
            // Zoom out
        },
        KeyCode::Char('r') => {
            // Reset view
        },
        _ => {}
    }
    Ok(())
}