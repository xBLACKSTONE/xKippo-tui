use anyhow::Result;
use chrono::Local;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table},
    Frame,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::app::App;
use crate::data::{EventType, Session};
use crate::ui::components::StatefulTable;

/// Session view state
pub struct SessionViewState {
    /// Table state for session list
    pub table: StatefulTable<Session>,
    /// Show details view
    pub show_details: bool,
    /// Selected session ID
    pub selected_session_id: Option<String>,
    /// Show command list for session
    pub show_commands: bool,
    /// Command list index
    pub command_index: usize,
    /// Show file list for session
    pub show_files: bool,
    /// File list index
    pub file_index: usize,
}

impl Default for SessionViewState {
    fn default() -> Self {
        Self {
            table: StatefulTable::new(),
            show_details: false,
            selected_session_id: None,
            show_commands: false,
            command_index: 0,
            show_files: false,
            file_index: 0,
        }
    }
}

/// Render the sessions view
pub fn render_sessions(f: &mut Frame, app: &App, area: Rect) {
    // Create sessions layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ].as_ref())
        .split(area);
    
    // Render filter bar (placeholder for now)
    render_filter_bar(f, app, chunks[0]);
    
    // Create main area layout
    let main_chunks = if app.selected_session_id.is_some() {
        // Split view for session list and details
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ].as_ref())
            .split(chunks[1])
    } else {
        // Full width for session list
        vec![chunks[1]]
    };
    
    // Render session list
    render_session_list(f, app, main_chunks[0]);
    
    // Render details if selected
    if app.selected_session_id.is_some() && main_chunks.len() > 1 {
        render_session_details(f, app, main_chunks[1]);
    }
}

/// Render the filter bar at the top
fn render_filter_bar(f: &mut Frame, app: &App, area: Rect) {
    // Simple filter bar for now
    let block = Block::default()
        .title("Filters [A]ctive [C]losed [M]alicious [All]")
        .borders(Borders::ALL);
    
    f.render_widget(block, area);
}

/// Render the list of sessions
fn render_session_list(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Get sessions
    let sessions = store_guard.get_sessions();
    
    // Create header row
    let header_cells = ["ID", "Source IP", "Username", "Status", "Commands", "Duration"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Create data rows
    let rows = sessions.iter().rev().map(|session| {
        let username = session.user.as_ref().map_or("N/A", |user| &user.username);
        let status = if session.end_time.is_some() { "Closed" } else { "Active" };
        let duration = session.duration.map_or("N/A".to_string(), |d| format!("{}s", d));
        
        let style = if session.is_malicious {
            Style::default().fg(Color::Red)
        } else if session.user.as_ref().map_or(false, |u| u.login_success) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };
        
        let cells = [
            Cell::from(session.id.chars().take(8).collect::<String>()),
            Cell::from(session.src_ip.clone()),
            Cell::from(username),
            Cell::from(status),
            Cell::from(session.commands.len().to_string()),
            Cell::from(duration),
        ];
        
        Row::new(cells).style(style)
    });
    
    // Create table
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().title("Sessions").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
        ])
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    
    f.render_widget(table, area);
}

/// Render the details of a selected session
fn render_session_details(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Get selected session
    let session = match &app.selected_session_id {
        Some(id) => store_guard.get_session(id),
        None => None,
    };
    
    let session = match session {
        Some(session) => session,
        None => return,
    };
    
    // Create layout for details
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ].as_ref())
        .split(area);
    
    // Create summary box
    let mut summary_lines = Vec::new();
    
    // Add basic information
    summary_lines.push(Line::from(vec![
        Span::styled("Session ID: ", Style::default().fg(Color::Yellow)),
        Span::raw(&session.id),
    ]));
    
    summary_lines.push(Line::from(vec![
        Span::styled("Start Time: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", session.start_time.with_timezone(&Local))),
    ]));
    
    if let Some(end_time) = session.end_time {
        summary_lines.push(Line::from(vec![
            Span::styled("End Time: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", end_time.with_timezone(&Local))),
        ]));
    }
    
    if let Some(duration) = session.duration {
        summary_lines.push(Line::from(vec![
            Span::styled("Duration: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} seconds", duration)),
        ]));
    }
    
    summary_lines.push(Line::from(vec![
        Span::styled("Source: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}:{}", session.src_ip, session.src_port)),
    ]));
    
    summary_lines.push(Line::from(vec![
        Span::styled("Destination: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}:{}", session.dst_ip, session.dst_port)),
    ]));
    
    summary_lines.push(Line::from(vec![
        Span::styled("Protocol: ", Style::default().fg(Color::Yellow)),
        Span::raw(&session.protocol),
    ]));
    
    if let Some(client_version) = &session.client_version {
        summary_lines.push(Line::from(vec![
            Span::styled("Client Version: ", Style::default().fg(Color::Yellow)),
            Span::raw(client_version),
        ]));
    }
    
    if let Some(user) = &session.user {
        let login_status = if user.login_success {
            Span::styled("Success", Style::default().fg(Color::Green))
        } else {
            Span::styled("Failed", Style::default().fg(Color::Red))
        };
        
        summary_lines.push(Line::from(vec![
            Span::styled("Login: ", Style::default().fg(Color::Yellow)),
            login_status,
        ]));
        
        summary_lines.push(Line::from(vec![
            Span::styled("Username: ", Style::default().fg(Color::Yellow)),
            Span::raw(&user.username),
        ]));
        
        if let Some(password) = &user.password {
            summary_lines.push(Line::from(vec![
                Span::styled("Password: ", Style::default().fg(Color::Yellow)),
                Span::raw(password),
            ]));
        }
    }
    
    // Risk information
    let risk_style = if session.malicious_score > 70 {
        Style::default().fg(Color::Red)
    } else if session.malicious_score > 30 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };
    
    summary_lines.push(Line::from(vec![
        Span::styled("Risk Score: ", Style::default().fg(Color::Yellow)),
        Span::styled(format!("{}/100", session.malicious_score), risk_style),
    ]));
    
    // Create summary box
    let summary = Paragraph::new(summary_lines)
        .block(Block::default().title("Session Summary").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(summary, chunks[0]);
    
    // Create details area with tabs for commands and files
    let details_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ].as_ref())
        .split(chunks[1]);
    
    // Create tabs
    let tabs = ratatui::widgets::Tabs::new(vec![
        Span::styled("Commands", Style::default().fg(Color::White)),
        Span::styled("Files", Style::default().fg(Color::White)),
    ])
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default())
    .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    .select(0); // TODO: Use actual tab index
    
    f.render_widget(tabs, details_chunks[0]);
    
    // Render commands tab (hardcoded for now)
    let header_cells = ["Time", "Command", "Success"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Create data rows for commands
    let rows = session.commands.iter().map(|cmd| {
        let time = cmd.timestamp.with_timezone(&Local).format("%H:%M:%S").to_string();
        let style = if cmd.success {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };
        
        let cells = [
            Cell::from(time),
            Cell::from(&cmd.command),
            Cell::from(if cmd.success { "Yes" } else { "No" }),
        ];
        
        Row::new(cells).style(style)
    });
    
    // Create table for commands
    let command_table = Table::new(rows)
        .header(header)
        .block(Block::default().title("Commands").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ]);
    
    f.render_widget(command_table, details_chunks[1]);
}

/// Handle input in the sessions view
pub async fn handle_sessions_input(key: crossterm::event::KeyEvent, app: &mut App) -> Result<()> {
    match key.code {
        crossterm::event::KeyCode::Down => {
            // TODO: Navigate sessions
        }
        crossterm::event::KeyCode::Up => {
            // TODO: Navigate sessions
        }
        crossterm::event::KeyCode::Enter => {
            // TODO: Show details for selected session
        }
        crossterm::event::KeyCode::Esc => {
            app.selected_session_id = None;
        }
        _ => {}
    }
    
    Ok(())
}