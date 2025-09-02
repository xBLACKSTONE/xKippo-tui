use anyhow::Result;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::sync::Arc;

use crate::app::App;
use crate::config::{Config, HoneypotConfig, UIConfig};

/// Settings view state
pub struct SettingsViewState {
    /// Currently selected setting category
    pub selected_category: usize,
    /// Settings categories list state
    pub category_state: ListState,
    /// Currently selected setting in the category
    pub selected_setting: usize,
    /// Settings list state
    pub setting_state: ListState,
    /// Editing mode
    pub editing: bool,
    /// Current edit value
    pub edit_value: String,
}

impl Default for SettingsViewState {
    fn default() -> Self {
        let mut category_state = ListState::default();
        category_state.select(Some(0));
        
        let mut setting_state = ListState::default();
        setting_state.select(Some(0));
        
        Self {
            selected_category: 0,
            category_state,
            selected_setting: 0,
            setting_state,
            editing: false,
            edit_value: String::new(),
        }
    }
}

/// Settings category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingCategory {
    General,
    Honeypot,
    Interface,
    Filtering,
    Alerts,
    GeoIP,
}

impl SettingCategory {
    fn as_str(&self) -> &'static str {
        match self {
            SettingCategory::General => "General",
            SettingCategory::Honeypot => "Honeypot",
            SettingCategory::Interface => "Interface",
            SettingCategory::Filtering => "Filtering",
            SettingCategory::Alerts => "Alerts",
            SettingCategory::GeoIP => "GeoIP",
        }
    }
    
    fn all() -> Vec<SettingCategory> {
        vec![
            SettingCategory::General,
            SettingCategory::Honeypot,
            SettingCategory::Interface,
            SettingCategory::Filtering,
            SettingCategory::Alerts,
            SettingCategory::GeoIP,
        ]
    }
}

/// Render settings view
pub fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    // Create settings layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ].as_ref())
        .split(area);
    
    // Render categories list
    render_categories(f, app, chunks[0]);
    
    // Render settings for selected category
    render_settings_for_category(f, app, chunks[1], SettingCategory::all()[0]); // TODO: Use actual selected category
}

/// Render settings categories
fn render_categories(f: &mut Frame, app: &App, area: Rect) {
    // Create categories
    let categories = SettingCategory::all();
    
    let category_items: Vec<ListItem> = categories.iter()
        .map(|category| {
            ListItem::new(Line::from(Span::raw(category.as_str())))
        })
        .collect();
    
    let categories_list = List::new(category_items)
        .block(Block::default().title("Categories").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    
    // TODO: Use proper state here
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    
    f.render_stateful_widget(categories_list, area, &mut list_state);
}

/// Render settings for selected category
fn render_settings_for_category(f: &mut Frame, app: &App, area: Rect, category: SettingCategory) {
    // Get settings for the category
    let settings_lines = match category {
        SettingCategory::General => render_general_settings(&app.config),
        SettingCategory::Honeypot => render_honeypot_settings(&app.config),
        SettingCategory::Interface => render_interface_settings(&app.config),
        SettingCategory::Filtering => render_filter_settings(&app.config),
        SettingCategory::Alerts => render_alert_settings(&app.config),
        SettingCategory::GeoIP => render_geoip_settings(&app.config),
    };
    
    // Render settings
    let settings = Paragraph::new(settings_lines)
        .block(Block::default().title(format!("{} Settings", category.as_str())).borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(settings, area);
}

/// Render general settings
fn render_general_settings(config: &Config) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled("Settings file: ", Style::default().fg(Color::Yellow)),
        Span::raw("~/.config/xkippo/config.toml"),
    ]));
    
    lines
}

/// Render honeypot settings
fn render_honeypot_settings(config: &Config) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled("Honeypot Name: ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.honeypot.name),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Honeypot Type: ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.honeypot.honeypot_type),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Auto-detect Log Files: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.honeypot.auto_detect { "Yes" } else { "No" }),
    ]));
    
    if let Some(log_paths) = &config.honeypot.log_paths {
        lines.push(Line::from(vec![
            Span::styled("Log Paths: ", Style::default().fg(Color::Yellow)),
        ]));
        
        for path in log_paths {
            lines.push(Line::from(vec![
                Span::raw(format!("  - {}", path)),
            ]));
        }
    }
    
    lines.push(Line::from(vec![
        Span::styled("History Hours: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", config.honeypot.history_hours)),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Check Interval: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{} ms", config.honeypot.check_interval_ms)),
    ]));
    
    lines
}

/// Render interface settings
fn render_interface_settings(config: &Config) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled("Theme: ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.ui.theme),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Color Enabled: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.ui.color_enabled { "Yes" } else { "No" }),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Mouse Enabled: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.ui.mouse_enabled { "Yes" } else { "No" }),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Refresh Interval: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{} ms", config.ui.refresh_interval_ms)),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Animations: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.ui.animations { "Yes" } else { "No" }),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Show Border: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.ui.show_border { "Yes" } else { "No" }),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Border Type: ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.ui.border_type),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Date Format: ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.ui.date_format),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Time Format: ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.ui.time_format),
    ]));
    
    lines
}

/// Render filter settings
fn render_filter_settings(config: &Config) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled("Case Sensitive Search: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.filter.case_sensitive { "Yes" } else { "No" }),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Max Sessions: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", config.filter.max_sessions)),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Max Logs: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", config.filter.max_logs)),
    ]));
    
    if !config.filter.default_event_types.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Default Event Types: ", Style::default().fg(Color::Yellow)),
            Span::raw(config.filter.default_event_types.join(", ")),
        ]));
    }
    
    lines
}

/// Render alert settings
fn render_alert_settings(config: &Config) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled("Alerts Enabled: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.alert.enabled { "Yes" } else { "No" }),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Alert on Login: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.alert.on_successful_login { "Yes" } else { "No" }),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Alert on File Upload: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.alert.on_file_upload { "Yes" } else { "No" }),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Alert on New Source IP: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.alert.on_new_source_ip { "Yes" } else { "No" }),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Sound Alerts: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.alert.sound_enabled { "Yes" } else { "No" }),
    ]));
    
    lines.push(Line::from(vec![
        Span::styled("Visual Alerts: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.alert.visual_enabled { "Yes" } else { "No" }),
    ]));
    
    if !config.alert.on_commands.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Alert on Commands: ", Style::default().fg(Color::Yellow)),
        ]));
        
        for cmd in &config.alert.on_commands {
            lines.push(Line::from(vec![
                Span::raw(format!("  - {}", cmd)),
            ]));
        }
    }
    
    if !config.alert.ip_blacklist.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("IP Blacklist: ", Style::default().fg(Color::Yellow)),
            Span::raw(config.alert.ip_blacklist.join(", ")),
        ]));
    }
    
    if !config.alert.ip_whitelist.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("IP Whitelist: ", Style::default().fg(Color::Yellow)),
            Span::raw(config.alert.ip_whitelist.join(", ")),
        ]));
    }
    
    lines
}

/// Render GeoIP settings
fn render_geoip_settings(config: &Config) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled("GeoIP Enabled: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.geoip.enabled { "Yes" } else { "No" }),
    ]));
    
    if let Some(path) = &config.geoip.database_path {
        lines.push(Line::from(vec![
            Span::styled("Database Path: ", Style::default().fg(Color::Yellow)),
            Span::raw(path),
        ]));
    }
    
    lines.push(Line::from(vec![
        Span::styled("Auto-download Database: ", Style::default().fg(Color::Yellow)),
        Span::raw(if config.geoip.auto_download { "Yes" } else { "No" }),
    ]));
    
    if config.geoip.license_key.is_some() {
        lines.push(Line::from(vec![
            Span::styled("License Key: ", Style::default().fg(Color::Yellow)),
            Span::raw("[configured]"),
        ]));
    }
    
    lines
}

/// Handle input in the settings view
pub async fn handle_settings_input(key: crossterm::event::KeyEvent, app: &mut App) -> Result<()> {
    match key.code {
        crossterm::event::KeyCode::Down => {
            // TODO: Navigate settings
        }
        crossterm::event::KeyCode::Up => {
            // TODO: Navigate settings
        }
        crossterm::event::KeyCode::Tab => {
            // TODO: Switch between categories and settings
        }
        crossterm::event::KeyCode::Enter => {
            // TODO: Edit selected setting
        }
        crossterm::event::KeyCode::Esc => {
            // TODO: Exit edit mode or back to categories
        }
        _ => {}
    }
    
    Ok(())
}