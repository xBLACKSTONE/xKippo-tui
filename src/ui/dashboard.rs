use anyhow::Result;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, LineGauge, Paragraph, Row, Table, Wrap},
    Frame,
};
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::app::App;
use crate::data::EventType;

/// Render the dashboard view
pub fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    // Create dashboard layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ].as_ref())
        .split(area);
    
    // Create horizontal splits for the top section
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ].as_ref())
        .split(chunks[0]);
    
    // Create horizontal splits for the bottom section
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ].as_ref())
        .split(chunks[2]);
    
    // Render each component
    render_summary(f, app, top_chunks[0]);
    render_activity(f, app, top_chunks[1]);
    render_sessions(f, app, chunks[1]);
    render_attackers(f, app, bottom_chunks[0]);
    render_credentials(f, app, bottom_chunks[1]);
}

/// Render honeypot summary
fn render_summary(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Calculate statistics
    let total_logs = store_guard.get_log_entry_count();
    let total_sessions = store_guard.get_session_count();
    let active_sessions = store_guard.get_active_sessions().len();
    let unique_ips = store_guard.get_unique_source_ips().len();
    let unique_usernames = store_guard.get_unique_usernames().len();
    let unique_passwords = store_guard.get_unique_passwords().len();
    
    // Create summary text
    let text = vec![
        Line::from(vec![
            Span::styled("Total Sessions: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", total_sessions)),
        ]),
        Line::from(vec![
            Span::styled("Active Sessions: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", active_sessions)),
        ]),
        Line::from(vec![
            Span::styled("Total Log Entries: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", total_logs)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Unique Source IPs: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", unique_ips)),
        ]),
        Line::from(vec![
            Span::styled("Unique Usernames: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", unique_usernames)),
        ]),
        Line::from(vec![
            Span::styled("Unique Passwords: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", unique_passwords)),
        ]),
    ];
    
    let block = Block::default()
        .title("Honeypot Summary")
        .borders(Borders::ALL);
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

/// Render activity gauges
fn render_activity(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    let logs = store_guard.get_log_entries();
    
    // Count event types
    let mut event_counts = HashMap::new();
    for entry in logs {
        *event_counts.entry(entry.event_type.clone()).or_insert(0) += 1;
    }
    
    // Get specific event counts
    let login_attempts = event_counts.get(&EventType::LoginAttempt)
        .unwrap_or(&0) + 
        event_counts.get(&EventType::LoginSuccess)
        .unwrap_or(&0) +
        event_counts.get(&EventType::LoginFailed)
        .unwrap_or(&0);
    
    let commands = event_counts.get(&EventType::Command).unwrap_or(&0);
    let connections = event_counts.get(&EventType::Connect).unwrap_or(&0);
    let file_transfers = event_counts.get(&EventType::FileUpload).unwrap_or(&0) + 
        event_counts.get(&EventType::FileDownload).unwrap_or(&0);
    
    // Create layout for gauges
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ].as_ref())
        .margin(1)
        .split(area);
    
    // Calculate percentages (with a max value to prevent division by zero)
    let max_value = [login_attempts, *commands, *connections, file_transfers].iter().max().copied().unwrap_or(1).max(1) as f64;
    
    // Render activity block
    let block = Block::default()
        .title("Activity")
        .borders(Borders::ALL);
    f.render_widget(block, area);
    
    // Render gauges
    render_gauge(f, "Logins", login_attempts as f64 / max_value, login_attempts, Color::Red, chunks[0]);
    render_gauge(f, "Commands", *commands as f64 / max_value, *commands, Color::Blue, chunks[1]);
    render_gauge(f, "Connections", *connections as f64 / max_value, *connections, Color::Green, chunks[2]);
    render_gauge(f, "Files", file_transfers as f64 / max_value, file_transfers, Color::Yellow, chunks[3]);
}

/// Helper to render a single gauge
fn render_gauge(
    f: &mut Frame, 
    label: &str, 
    ratio: f64, 
    value: usize, 
    color: Color, 
    area: Rect
) {
    let gauge = LineGauge::default()
        .block(Block::default().title(format!("{}: {}", label, value)))
        .gauge_style(Style::default().fg(color).bg(Color::Black))
        .line_set(ratatui::symbols::line::THICK)
        .ratio(ratio);
    
    f.render_widget(gauge, area);
}

/// Render recent sessions
fn render_sessions(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Get recent sessions (up to 10)
    let sessions = store_guard.get_sessions();
    let sessions = sessions.iter().rev().take(10).collect::<Vec<_>>();
    
    // Create header row
    let header_cells = ["ID", "Source IP", "Username", "Status", "Commands", "Duration"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Create data rows
    let rows = sessions.iter().map(|session| {
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
        .block(Block::default().title("Recent Sessions").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
        ])
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    
    f.render_widget(table, area);
}

/// Render top attackers
fn render_attackers(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Count sessions by source IP
    let mut ip_counts = HashMap::new();
    for session in store_guard.get_sessions() {
        *ip_counts.entry(session.src_ip.clone()).or_insert(0) += 1;
    }
    
    // Sort by count
    let mut ip_counts = ip_counts.into_iter().collect::<Vec<_>>();
    ip_counts.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Take top 10
    let ip_counts = ip_counts.into_iter().take(10).collect::<Vec<_>>();
    
    // Create header row
    let header_cells = ["IP Address", "Sessions"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Create data rows
    let rows = ip_counts.iter().map(|(ip, count)| {
        let cells = [
            Cell::from(ip.clone()),
            Cell::from(count.to_string()),
        ];
        
        Row::new(cells)
    });
    
    // Create table
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().title("Top Attackers").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ]);
    
    f.render_widget(table, area);
}

/// Render top credentials
fn render_credentials(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Count username/password combinations
    let mut cred_counts = HashMap::new();
    for entry in store_guard.get_log_entries() {
        if entry.event_type == EventType::LoginAttempt || 
           entry.event_type == EventType::LoginSuccess || 
           entry.event_type == EventType::LoginFailed {
            if let (Some(username), Some(password)) = (&entry.username, &entry.password) {
                *cred_counts.entry((username.clone(), password.clone())).or_insert(0) += 1;
            }
        }
    }
    
    // Sort by count
    let mut cred_counts = cred_counts.into_iter().collect::<Vec<_>>();
    cred_counts.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Take top 10
    let cred_counts = cred_counts.into_iter().take(10).collect::<Vec<_>>();
    
    // Create header row
    let header_cells = ["Username", "Password", "Count"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Create data rows
    let rows = cred_counts.iter().map(|((username, password), count)| {
        let cells = [
            Cell::from(username.clone()),
            Cell::from(password.clone()),
            Cell::from(count.to_string()),
        ];
        
        Row::new(cells)
    });
    
    // Create table
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().title("Top Credentials").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(35),
            Constraint::Percentage(45),
            Constraint::Percentage(20),
        ]);
    
    f.render_widget(table, area);
}