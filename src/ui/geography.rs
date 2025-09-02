use crossterm::event::KeyCode;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;

use crate::app::App;
use crate::ui::components::map::WorldMap;

/// Render geography view - interface for UI module
pub fn render_geography(f: &mut Frame, app: &App, area: Rect) {
    draw(f, app);
}

/// Draw the geography view
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),    // Title
            Constraint::Min(0),       // Main content
            Constraint::Length(3),    // Status bar
        ])
        .split(f.size());
    
    draw_title(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);
}

/// Draw the title area
fn draw_title(f: &mut Frame, app: &App, area: Rect) {
    let title = Paragraph::new(vec![
        Spans::from(vec![
            Span::styled(
                app.title.clone(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Geographic Visualization"),
        ]),
        Spans::from(vec![
            Span::raw("Press "),
            Span::styled("1-6", Style::default().fg(Color::Yellow)),
            Span::raw(" to switch tabs, "),
            Span::styled("h", Style::default().fg(Color::Yellow)),
            Span::raw(" for help, "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" to quit"),
        ]),
    ])
    .block(Block::default().borders(Borders::BOTTOM));
    
    f.render_widget(title, area);
}

/// Draw the main content area
fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);
        
    draw_world_map(f, app, horizontal_chunks[0]);
    draw_country_stats(f, app, horizontal_chunks[1]);
}

/// Draw the world map with attack points
fn draw_world_map(f: &mut Frame, app: &App, area: Rect) {
    let map_block = Block::default()
        .title("Attack Origins")
        .borders(Borders::ALL);
    
    f.render_widget(map_block, area);
    
    // In a real implementation, we would use actual attacker coordinates
    // For demonstration, we'll create some example attack points
    let attack_points = vec![
        (40.7128, -74.0060, 5),  // New York
        (51.5074, -0.1278, 8),   // London
        (39.9042, 116.4074, 15), // Beijing
        (55.7558, 37.6173, 7),   // Moscow
        (35.6762, 139.6503, 10), // Tokyo
        (-33.8688, 151.2093, 3), // Sydney
    ];
    
    // Create a world map and draw it
    let inner_area = map_block.inner(area);
    let world_map = WorldMap::new(attack_points);
    f.render_widget(world_map, inner_area);
}

/// Draw country statistics
fn draw_country_stats(f: &mut Frame, app: &App, area: Rect) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);
        
    // Top country stats panel
    let country_block = Block::default()
        .title("Top Countries")
        .borders(Borders::ALL);
    
    // Example country data (in a real app, this would come from the data store)
    let countries = vec![
        ("Russia", 158),
        ("China", 143),
        ("United States", 87),
        ("Brazil", 42),
        ("Netherlands", 36),
        ("Germany", 29),
        ("France", 18),
        ("India", 15),
    ];
    
    let country_items: Vec<ListItem> = countries
        .iter()
        .map(|(country, count)| {
            ListItem::new(format!("{}: {} attacks", country, count))
        })
        .collect();
    
    let country_list = List::new(country_items)
        .block(country_block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    
    f.render_widget(country_list, vertical_chunks[0]);
    
    // Bottom ASN stats panel
    let asn_block = Block::default()
        .title("Top ASNs")
        .borders(Borders::ALL);
    
    // Example ASN data
    let asns = vec![
        ("AS4134 - China Telecom", 87),
        ("AS3462 - Hinet", 46),
        ("AS4837 - China Unicom", 38),
        ("AS7922 - Comcast", 27),
        ("AS16509 - Amazon", 22),
        ("AS45899 - VNPT Corp", 18),
        ("AS9121 - Turk Telekom", 15),
        ("AS8151 - Telmex", 12),
    ];
    
    let asn_items: Vec<ListItem> = asns
        .iter()
        .map(|(asn, count)| {
            ListItem::new(format!("{}: {} attacks", asn, count))
        })
        .collect();
    
    let asn_list = List::new(asn_items)
        .block(asn_block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    
    f.render_widget(asn_list, vertical_chunks[1]);
}

/// Draw the status bar
fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(vec![
        Spans::from(vec![
            Span::styled(
                "TOTAL COUNTRIES: ",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::raw("42 countries, "),
            Span::styled(
                "TOP ATTACKER: ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("185.156.73.54 (Russia)"),
            Span::raw(" | "),
            Span::styled(
                "ATTACKS TODAY: ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw("287"),
        ]),
    ])
    .block(Block::default().borders(Borders::TOP));
    
    f.render_widget(status, area);
}

/// Handle keyboard input for the geography view
pub fn handle_input(app: &mut App, key: KeyCode) {
    match key {
        // Add geography-specific key handlers here
        KeyCode::Char('z') => {
            // Zoom in on map (in a real implementation)
        },
        KeyCode::Char('Z') => {
            // Zoom out on map (in a real implementation)
        },
        _ => {}
    }
}