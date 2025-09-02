use anyhow::Result;
use chrono::Local;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs},
    Frame,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::app::App;
use crate::data::EventType;

/// Log view state
pub struct LogViewState {
    /// Table state for log entry list
    pub table_state: ListState,
    /// Current filter
    pub filter: String,
    /// Selected event type filter
    pub event_type_filter: Option<EventType>,
    /// Show details view
    pub show_details: bool,
    /// Selected log entry ID
    pub selected_log_id: Option<String>,
}

impl Default for LogViewState {
    fn default() -> Self {
        let mut table_state = ListState::default();
        table_state.select(Some(0));
        
        Self {
            table_state,
            filter: String::new(),
            event_type_filter: None,
            show_details: false,
            selected_log_id: None,
        }
    }
}

/// Render the logs view
pub fn render_logs(f: &mut Frame, app: &App, area: Rect) {
    // Create logs layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ].as_ref())
        .split(area);
    
    // Render filter bar
    render_filter_bar(f, app, chunks[0]);
    
    // Create main area layout
    let main_chunks = if app.selected_log_entry_id.is_some() {
        // Split view for log list and details
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .split(chunks[1])
    } else {
        // Full width for log list
        vec![chunks[1]]
    };
    
    // Render log list
    render_log_list(f, app, main_chunks[0]);
    
    // Render details if selected
    if app.selected_log_entry_id.is_some() && main_chunks.len() > 1 {
        render_log_details(f, app, main_chunks[1]);
    }
}

/// Render the filter bar at the top
fn render_filter_bar(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Filters")
        .borders(Borders::ALL);
    
    // Create event type filters
    let event_types = vec![
        "All", "Login", "Command", "Connect", "File", "Other"
    ];
    
    let tabs = Tabs::new(event_types.iter().map(|t| Span::raw(*t)).collect())
        .block(block)
        .style(Style::default())
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(0); // TODO: Use actual filter state
    
    f.render_widget(tabs, area);
}

/// Render the list of log entries
fn render_log_list(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Get log entries
    let logs = store_guard.get_log_entries();
    
    // Create header row
    let header_cells = ["Time", "Event", "Session", "Source IP", "Username", "Details"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Create data rows
    let rows = logs.iter().rev().map(|log| {
        // Format timestamp
        let timestamp = log.timestamp.with_timezone(&Local)
            .format("%H:%M:%S").to_string();
        
        // Get event-specific details
        let details = match log.event_type {
            EventType::Command => log.command.clone().unwrap_or_default(),
            EventType::LoginAttempt | EventType::LoginSuccess | EventType::LoginFailed => {
                format!("{}:{}", 
                        log.username.clone().unwrap_or_default(),
                        log.password.clone().unwrap_or_default())
            },
            EventType::FileUpload => {
                if let Some(file) = &log.file {
                    format!("Upload: {}", file.filename)
                } else {
                    "File upload".to_string()
                }
            },
            EventType::FileDownload => {
                if let Some(file) = &log.file {
                    format!("Download: {}", file.filename)
                } else {
                    "File download".to_string()
                }
            },
            _ => String::new(),
        };
        
        // Style based on event type
        let style = match log.event_type {
            EventType::LoginSuccess => Style::default().fg(Color::Green),
            EventType::LoginFailed => Style::default().fg(Color::Red),
            EventType::Command => Style::default().fg(Color::Blue),
            EventType::FileUpload | EventType::FileDownload => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        };
        
        let cells = [
            Cell::from(timestamp),
            Cell::from(format!("{}", log.event_type)),
            Cell::from(log.session.clone().unwrap_or_default()),
            Cell::from(log.src_ip.clone().unwrap_or_default()),
            Cell::from(log.username.clone().unwrap_or_default()),
            Cell::from(details),
        ];
        
        Row::new(cells).style(style)
    });
    
    // Create table
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().title("Log Entries").borders(Borders::ALL))
        .widths(&[
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Percentage(40),
        ])
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    
    f.render_widget(table, area);
}

/// Render the details of a selected log entry
fn render_log_details(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Get selected log entry
    let log_entry = match &app.selected_log_entry_id {
        Some(id) => store_guard.get_log_entry(id),
        None => None,
    };
    
    let log_entry = match log_entry {
        Some(entry) => entry,
        None => return,
    };
    
    // Create detail lines
    let mut lines = Vec::new();
    
    // Add basic information
    lines.push(Line::from(vec![
        Span::styled("Event: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", log_entry.event_type)),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Timestamp: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", log_entry.timestamp.with_timezone(&Local))),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Session: ", Style::default().fg(Color::Yellow)),
        Span::raw(log_entry.session.clone().unwrap_or_default()),
    ]));
    
    if let Some(src_ip) = &log_entry.src_ip {
        lines.push(Line::from(vec![
            Span::styled("Source IP: ", Style::default().fg(Color::Yellow)),
            Span::raw(src_ip),
        ]));
    }
    
    if let Some(src_port) = log_entry.src_port {
        lines.push(Line::from(vec![
            Span::styled("Source Port: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", src_port)),
        ]));
    }
    
    if let Some(dst_ip) = &log_entry.dst_ip {
        lines.push(Line::from(vec![
            Span::styled("Destination IP: ", Style::default().fg(Color::Yellow)),
            Span::raw(dst_ip),
        ]));
    }
    
    if let Some(dst_port) = log_entry.dst_port {
        lines.push(Line::from(vec![
            Span::styled("Destination Port: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", dst_port)),
        ]));
    }
    
    if let Some(username) = &log_entry.username {
        lines.push(Line::from(vec![
            Span::styled("Username: ", Style::default().fg(Color::Yellow)),
            Span::raw(username),
        ]));
    }
    
    if let Some(password) = &log_entry.password {
        lines.push(Line::from(vec![
            Span::styled("Password: ", Style::default().fg(Color::Yellow)),
            Span::raw(password),
        ]));
    }
    
    if let Some(command) = &log_entry.command {
        lines.push(Line::from(vec![
            Span::styled("Command: ", Style::default().fg(Color::Yellow)),
            Span::raw(command),
        ]));
    }
    
    if let Some(file) = &log_entry.file {
        lines.push(Line::from(vec![
            Span::styled("File: ", Style::default().fg(Color::Yellow)),
            Span::raw(&file.filename),
        ]));
        
        if let Some(shasum) = &file.shasum {
            lines.push(Line::from(vec![
                Span::styled("SHA256: ", Style::default().fg(Color::Yellow)),
                Span::raw(shasum),
            ]));
        }
    }
    
    // Add additional fields
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Additional Fields:", Style::default().fg(Color::Yellow))));
    
    for (key, value) in &log_entry.fields {
        if let Some(value_str) = value.as_str() {
            lines.push(Line::from(vec![
                Span::styled(format!("{}: ", key), Style::default().fg(Color::Cyan)),
                Span::raw(value_str),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(format!("{}: ", key), Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}", value)),
            ]));
        }
    }
    
    // Create paragraph
    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Log Details").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

/// Handle input in the logs view
pub async fn handle_logs_input(key: crossterm::event::KeyEvent, app: &mut App) -> Result<()> {
    match key.code {
        crossterm::event::KeyCode::Down => {
            // TODO: Navigate logs
        }
        crossterm::event::KeyCode::Up => {
            // TODO: Navigate logs
        }
        crossterm::event::KeyCode::Enter => {
            // TODO: Show details for selected log
        }
        crossterm::event::KeyCode::Esc => {
            // TODO: Close details view
        }
        _ => {}
    }
    
    Ok(())
}