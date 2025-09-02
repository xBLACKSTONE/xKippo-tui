use anyhow::Result;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, LineGauge, Paragraph, Row, Table, Tabs, Wrap, BarChart},
    Frame,
};
use std::collections::HashMap;
use chrono::{Utc, TimeZone, Duration};

use crate::app::App;
use crate::data::{EventType, Session};

/// Render the security analyst dashboard view
pub fn render_security_dashboard(f: &mut Frame, app: &App, area: Rect) {
    // Create dashboard layout based on the user's selected layout in config
    let layout = app.config.dashboard.layout.as_str();
    
    match layout {
        "security" => render_security_focused_layout(f, app, area),
        "analytics" => render_analytics_focused_layout(f, app, area),
        _ => render_standard_layout(f, app, area),
    }
}

/// Render standard security layout
fn render_standard_layout(f: &mut Frame, app: &App, area: Rect) {
    // Create dashboard layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
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
        .split(chunks[1]);
    
    // Render each component
    render_threat_overview(f, app, top_chunks[0]);
    render_attack_map(f, app, top_chunks[1]);
    render_high_risk_sessions(f, app, bottom_chunks[0]);
    render_alerts_panel(f, app, bottom_chunks[1]);
}

/// Render security-focused layout
fn render_security_focused_layout(f: &mut Frame, app: &App, area: Rect) {
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
    render_attack_map(f, app, top_chunks[0]);
    render_threat_scores(f, app, top_chunks[1]);
    render_high_risk_sessions(f, app, chunks[1]);
    render_alerts_panel(f, app, bottom_chunks[0]);
    render_malware_analysis(f, app, bottom_chunks[1]);
}

/// Render analytics-focused layout
fn render_analytics_focused_layout(f: &mut Frame, app: &App, area: Rect) {
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
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ].as_ref())
        .split(chunks[0]);
    
    // Create horizontal splits for the middle section
    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ].as_ref())
        .split(chunks[1]);
    
    // Render each component
    render_threat_summary(f, app, top_chunks[0]);
    render_time_series_chart(f, app, top_chunks[1]);
    render_command_distribution(f, app, middle_chunks[0]);
    render_geographic_distribution(f, app, middle_chunks[1]);
    render_attacker_correlation(f, app, chunks[2]);
}

/// Render the threat overview panel
fn render_threat_overview(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Get sessions and calculate statistics
    let sessions = store_guard.get_sessions();
    let total_sessions = sessions.len();
    
    // Count high-risk sessions
    let high_risk_count = sessions.iter()
        .filter(|s| s.malicious_score >= 70)
        .count();
    
    // Count medium-risk sessions
    let medium_risk_count = sessions.iter()
        .filter(|s| s.malicious_score >= 30 && s.malicious_score < 70)
        .count();
    
    // Count low-risk sessions
    let low_risk_count = sessions.iter()
        .filter(|s| s.malicious_score < 30)
        .count();
    
    // Count successful logins
    let successful_logins = sessions.iter()
        .filter(|s| s.user.as_ref().map_or(false, |u| u.login_success))
        .count();
    
    // Count file uploads
    let file_uploads = sessions.iter()
        .flat_map(|s| &s.files)
        .filter(|f| f.is_malware)
        .count();
    
    // Create text
    let text = vec![
        Line::from(vec![
            Span::styled("Threat Overview", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("High Risk Sessions: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{} ({}%)", high_risk_count, percentage(high_risk_count, total_sessions))),
        ]),
        Line::from(vec![
            Span::styled("Medium Risk Sessions: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} ({}%)", medium_risk_count, percentage(medium_risk_count, total_sessions))),
        ]),
        Line::from(vec![
            Span::styled("Low Risk Sessions: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{} ({}%)", low_risk_count, percentage(low_risk_count, total_sessions))),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Successful Logins: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", successful_logins)),
        ]),
        Line::from(vec![
            Span::styled("Malware Uploads: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", file_uploads)),
        ]),
    ];
    
    let block = Block::default()
        .title("Security Overview")
        .borders(Borders::ALL);
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

/// Render threat summary for the analytics view
fn render_threat_summary(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Get sessions
    let sessions = store_guard.get_sessions();
    
    // Calculate time ranges
    let now = Utc::now();
    let day_ago = now - Duration::days(1);
    let week_ago = now - Duration::days(7);
    
    // Count sessions in different time periods
    let sessions_today = sessions.iter()
        .filter(|s| s.start_time > day_ago)
        .count();
    
    let sessions_week = sessions.iter()
        .filter(|s| s.start_time > week_ago)
        .count();
    
    // Count high severity incidents
    let high_severity = sessions.iter()
        .filter(|s| s.malicious_score >= 70)
        .count();
    
    // Count successful logins
    let successful_logins = sessions.iter()
        .filter(|s| s.user.as_ref().map_or(false, |u| u.login_success))
        .count();
    
    // Count files uploaded
    let files_uploaded = sessions.iter()
        .flat_map(|s| &s.files)
        .count();
    
    // Count potentially malicious files
    let malicious_files = sessions.iter()
        .flat_map(|s| &s.files)
        .filter(|f| f.is_malware)
        .count();
    
    // Create text
    let text = vec![
        Line::from(vec![
            Span::styled("Threat Analytics", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Sessions (24h): ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{}", sessions_today)),
        ]),
        Line::from(vec![
            Span::styled("Sessions (7d): ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{}", sessions_week)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("High Severity: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", high_severity)),
        ]),
        Line::from(vec![
            Span::styled("Successful Logins: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", successful_logins)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Files Uploaded: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", files_uploaded)),
        ]),
        Line::from(vec![
            Span::styled("Malicious Files: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", malicious_files)),
        ]),
    ];
    
    let block = Block::default()
        .title("Threat Summary")
        .borders(Borders::ALL);
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

/// Render the attack map
fn render_attack_map(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Attack Map")
        .borders(Borders::ALL);
    
    // If GeoIP is enabled, render a placeholder for the map
    // In a real implementation, this would render an actual geographic map
    let text = vec![
        Line::from(""),
        Line::from("    World Map Visualization    "),
        Line::from("     (GeoIP Integration)       "),
        Line::from(""),
        Line::from(" • Attack sources shown as dots"),
        Line::from(" • Dot size indicates severity"),
        Line::from(" • Red: high risk, Yellow: medium risk"),
        Line::from(" • Green: low risk"),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

/// Render high risk sessions table
fn render_high_risk_sessions(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Get sessions and sort by risk score
    let mut sessions = store_guard.get_sessions().clone();
    sessions.sort_by(|a, b| b.malicious_score.cmp(&a.malicious_score));
    
    // Take top 10 risky sessions
    let sessions = sessions.iter().take(10).collect::<Vec<_>>();
    
    // Create header row
    let header_cells = ["IP", "User", "Risk", "Activities", "Files", "Commands"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Create data rows
    let rows = sessions.iter().map(|session| {
        let username = session.user.as_ref().map_or("N/A", |user| &user.username);
        let risk_color = match session.malicious_score {
            score if score >= 70 => Color::Red,
            score if score >= 30 => Color::Yellow,
            _ => Color::Green,
        };
        
        let cells = [
            Cell::from(session.src_ip.clone()),
            Cell::from(username),
            Cell::from(session.malicious_score.to_string()).style(Style::default().fg(risk_color)),
            Cell::from(risk_activities(session)),
            Cell::from(session.files.len().to_string()),
            Cell::from(session.commands.len().to_string()),
        ];
        
        Row::new(cells)
    });
    
    // Create table
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().title("High Risk Sessions").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(25),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
        ])
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    
    f.render_widget(table, area);
}

/// Render alerts panel
fn render_alerts_panel(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Get sessions
    let sessions = store_guard.get_sessions();
    
    // Create alerts based on suspicious activities
    let mut alerts = Vec::new();
    
    for session in sessions {
        // Alert for successful logins
        if app.config.alert.on_successful_login && 
           session.user.as_ref().map_or(false, |u| u.login_success) {
            alerts.push((
                session.start_time,
                format!("Successful login: {} -> {}", session.src_ip, 
                    session.user.as_ref().map_or("unknown".to_string(), |u| u.username.clone())),
                "high"
            ));
        }
        
        // Alert for file uploads
        if app.config.alert.on_file_upload && !session.files.is_empty() {
            for file in &session.files {
                alerts.push((
                    file.timestamp,
                    format!("File upload: {} uploaded {}", session.src_ip, file.filename),
                    "high"
                ));
            }
        }
        
        // Alert for specific commands
        if !app.config.alert.on_commands.is_empty() {
            for cmd in &session.commands {
                if app.config.alert.on_commands.iter().any(|c| cmd.command.contains(c)) {
                    alerts.push((
                        cmd.timestamp,
                        format!("Suspicious command: {} ran '{}'", session.src_ip, cmd.command),
                        "medium"
                    ));
                }
            }
        }
    }
    
    // Sort alerts by timestamp (most recent first)
    alerts.sort_by(|a, b| b.0.cmp(&a.0));
    
    // Create header row
    let header_cells = ["Time", "Alert", "Severity"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Create data rows
    let rows = alerts.iter().take(10).map(|(time, message, severity)| {
        let severity_style = match *severity {
            "high" => Style::default().fg(Color::Red),
            "medium" => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::Green),
        };
        
        let cells = [
            Cell::from(time.format("%H:%M:%S").to_string()),
            Cell::from(message.clone()),
            Cell::from(*severity).style(severity_style),
        ];
        
        Row::new(cells)
    });
    
    // Create table
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().title("Security Alerts").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(15),
            Constraint::Percentage(65),
            Constraint::Percentage(20),
        ]);
    
    f.render_widget(table, area);
}

/// Render threat score distribution
fn render_threat_scores(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Get sessions and count risk categories
    let sessions = store_guard.get_sessions();
    
    let mut risk_categories = [0; 5];
    for session in sessions {
        let category = match session.malicious_score {
            score if score >= 80 => 0, // Critical
            score if score >= 60 => 1, // High
            score if score >= 40 => 2, // Medium
            score if score >= 20 => 3, // Low
            _ => 4,                   // Info
        };
        risk_categories[category] += 1;
    }
    
    // Create data for bar chart
    let data = [
        ("Critical", risk_categories[0]),
        ("High", risk_categories[1]),
        ("Medium", risk_categories[2]),
        ("Low", risk_categories[3]),
        ("Info", risk_categories[4]),
    ];
    
    let max_value = *risk_categories.iter().max().unwrap_or(&1);
    
    // Create bar chart data
    let bar_data: Vec<(&str, u64)> = data
        .iter()
        .map(|(name, count)| (*name, *count as u64))
        .collect();
    
    // Create color mapping
    let bar_style = |i| {
        match i {
            0 => Style::default().fg(Color::Red),
            1 => Style::default().fg(Color::LightRed),
            2 => Style::default().fg(Color::Yellow),
            3 => Style::default().fg(Color::Green),
            _ => Style::default().fg(Color::Blue),
        }
    };
    
    // Create bar chart
    let barchart = BarChart::default()
        .block(Block::default().title("Threat Score Distribution").borders(Borders::ALL))
        .data(&bar_data)
        .bar_width(9)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Red))
        .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .label_style(Style::default().fg(Color::White));
    
    f.render_widget(barchart, area);
}

/// Render malware analysis panel
fn render_malware_analysis(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Get all files from all sessions
    let mut files = Vec::new();
    for session in store_guard.get_sessions() {
        for file in &session.files {
            files.push((session.src_ip.clone(), file.clone()));
        }
    }
    
    // Sort files by timestamp (most recent first)
    files.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
    
    // Create header row
    let header_cells = ["Filename", "Source IP", "Size", "Status"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Create data rows
    let rows = files.iter().take(10).map(|(ip, file)| {
        let status = if file.is_malware {
            "Malicious"
        } else if file.is_executable {
            "Executable"
        } else {
            "Normal"
        };
        
        let status_style = if file.is_malware {
            Style::default().fg(Color::Red)
        } else if file.is_executable {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        };
        
        let size = file.size.map_or("Unknown".to_string(), |s| {
            if s < 1024 {
                format!("{} B", s)
            } else if s < 1024 * 1024 {
                format!("{:.1} KB", s as f64 / 1024.0)
            } else {
                format!("{:.1} MB", s as f64 / (1024.0 * 1024.0))
            }
        });
        
        let cells = [
            Cell::from(file.filename.clone()),
            Cell::from(ip.clone()),
            Cell::from(size),
            Cell::from(status).style(status_style),
        ];
        
        Row::new(cells)
    });
    
    // Create table
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().title("Malware Analysis").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(40),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ]);
    
    f.render_widget(table, area);
}

/// Render time series chart
fn render_time_series_chart(f: &mut Frame, app: &App, area: Rect) {
    // This would render a time series chart of activity
    // For now, just render a placeholder
    let block = Block::default()
        .title("Activity Timeline")
        .borders(Borders::ALL);
    
    let text = vec![
        Line::from(""),
        Line::from("    Time Series Visualization    "),
        Line::from(""),
        Line::from(" Last 24 hours:"),
        Line::from(" ├─── Login attempts: █████████"),
        Line::from(" ├─── Sessions: ██████"),
        Line::from(" └─── Malware: ███"),
        Line::from(""),
        Line::from(" Time →"),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

/// Render command distribution chart
fn render_command_distribution(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Count command frequencies
    let mut cmd_counts = HashMap::new();
    for session in store_guard.get_sessions() {
        for cmd in &session.commands {
            // Extract the base command (first word)
            let base_cmd = cmd.command.split_whitespace().next().unwrap_or(&cmd.command).to_string();
            *cmd_counts.entry(base_cmd).or_insert(0) += 1;
        }
    }
    
    // Sort by frequency
    let mut cmd_counts = cmd_counts.into_iter().collect::<Vec<_>>();
    cmd_counts.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Take top 10
    let cmd_counts = cmd_counts.into_iter().take(10).collect::<Vec<_>>();
    
    // Create header row
    let header_cells = ["Command", "Count", "Distribution"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Find max count for bar scaling
    let max_count = cmd_counts.iter().map(|(_, count)| *count).max().unwrap_or(1);
    
    // Create data rows
    let rows = cmd_counts.iter().map(|(cmd, count)| {
        // Create a bar of # symbols proportional to count
        let bar_width = ((count * 20) / max_count).max(1);
        let bar = "█".repeat(bar_width);
        
        let cells = [
            Cell::from(cmd.clone()),
            Cell::from(count.to_string()),
            Cell::from(bar).style(Style::default().fg(Color::Blue)),
        ];
        
        Row::new(cells)
    });
    
    // Create table
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().title("Command Distribution").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(50),
        ]);
    
    f.render_widget(table, area);
}

/// Render geographic distribution
fn render_geographic_distribution(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Count sessions by country (if GeoIP is enabled)
    let mut country_counts = HashMap::new();
    
    for session in store_guard.get_sessions() {
        if let Some(geo) = &session.geo_location {
            let country = format!("{} ({})", geo.country_name, geo.country_code);
            *country_counts.entry(country).or_insert(0) += 1;
        } else {
            *country_counts.entry("Unknown".to_string()).or_insert(0) += 1;
        }
    }
    
    // Sort by frequency
    let mut country_counts = country_counts.into_iter().collect::<Vec<_>>();
    country_counts.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Take top 10
    let country_counts = country_counts.into_iter().take(10).collect::<Vec<_>>();
    
    // Create header row
    let header_cells = ["Country", "Count", "Distribution"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Find max count for bar scaling
    let max_count = country_counts.iter().map(|(_, count)| *count).max().unwrap_or(1);
    
    // Create data rows
    let rows = country_counts.iter().map(|(country, count)| {
        // Create a bar of # symbols proportional to count
        let bar_width = ((count * 20) / max_count).max(1);
        let bar = "█".repeat(bar_width);
        
        let cells = [
            Cell::from(country.clone()),
            Cell::from(count.to_string()),
            Cell::from(bar).style(Style::default().fg(Color::Green)),
        ];
        
        Row::new(cells)
    });
    
    // Create table
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().title("Geographic Distribution").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(40),
            Constraint::Percentage(15),
            Constraint::Percentage(45),
        ]);
    
    f.render_widget(table, area);
}

/// Render attacker correlation panel
fn render_attacker_correlation(f: &mut Frame, app: &App, area: Rect) {
    let store_guard = match app.store.try_read() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    // Group similar attacks
    let sessions = store_guard.get_sessions();
    
    // Group by unique IP/username combinations
    let mut correlations = HashMap::new();
    
    for session in sessions {
        let username = session.user.as_ref().map_or("N/A".to_string(), |u| u.username.clone());
        let key = format!("{} / {}", session.src_ip, username);
        
        correlations.entry(key).or_insert_with(Vec::new).push(session.id.clone());
    }
    
    // Sort by number of correlated sessions
    let mut correlations = correlations.into_iter().collect::<Vec<_>>();
    correlations.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    
    // Take top entries
    let correlations = correlations.into_iter().take(10).collect::<Vec<_>>();
    
    // Create header row
    let header_cells = ["Source / Username", "Sessions", "Pattern"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Create data rows
    let rows = correlations.iter().map(|(key, sessions)| {
        let cells = [
            Cell::from(key.clone()),
            Cell::from(sessions.len().to_string()),
            Cell::from(format!("{} related sessions", sessions.len())),
        ];
        
        Row::new(cells)
    });
    
    // Create table
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().title("Attack Correlation").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(40),
            Constraint::Percentage(15),
            Constraint::Percentage(45),
        ]);
    
    f.render_widget(table, area);
}

// Helper functions

/// Calculate percentage safely
fn percentage(part: usize, total: usize) -> usize {
    if total == 0 {
        0
    } else {
        (part * 100) / total
    }
}

/// Get risk activities as a formatted string
fn risk_activities(session: &Session) -> String {
    let mut activities = Vec::new();
    
    if session.user.as_ref().map_or(false, |u| u.login_success) {
        activities.push("Login");
    }
    
    if !session.files.is_empty() {
        activities.push("Files");
    }
    
    if session.commands.len() > 10 {
        activities.push("Many cmds");
    } else if !session.commands.is_empty() {
        activities.push("Commands");
    }
    
    if activities.is_empty() {
        "None".to_string()
    } else {
        activities.join(", ")
    }
}