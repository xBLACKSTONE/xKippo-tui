use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

/// Draw the help popup
pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();
    
    // Calculate popup size
    let width = size.width.min(60);
    let height = size.height.min(20);
    let x = (size.width - width) / 2;
    let y = (size.height - height) / 2;
    
    let popup_area = Rect::new(x, y, width, height);
    
    // Create a clear background
    f.render_widget(Clear, popup_area);
    
    // Create the popup block
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(block, popup_area);
    
    // Create inner area for content
    let inner_area = Rect::new(
        popup_area.x + 2,
        popup_area.y + 1,
        popup_area.width - 4,
        popup_area.height - 2,
    );
    
    // Help content
    let help_text = vec![
        Spans::from(vec![
            Span::styled("xKippo-tui: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Cowrie Honeypot Monitoring System"),
        ]),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("NAVIGATION", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("1-6", Style::default().fg(Color::Green)),
            Span::raw(" - Switch between tabs"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("Tab", Style::default().fg(Color::Green)),
            Span::raw(" - Next tab"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("Shift+Tab", Style::default().fg(Color::Green)),
            Span::raw(" - Previous tab"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("h", Style::default().fg(Color::Green)),
            Span::raw(" - Toggle help"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("q", Style::default().fg(Color::Green)),
            Span::raw(" - Quit"),
        ]),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("LOGS VIEW", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("j/k", Style::default().fg(Color::Green)),
            Span::raw(" - Scroll up/down"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("f", Style::default().fg(Color::Green)),
            Span::raw(" - Filter logs"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("s", Style::default().fg(Color::Green)),
            Span::raw(" - Search"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("n", Style::default().fg(Color::Green)),
            Span::raw(" - Next search result"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("N", Style::default().fg(Color::Green)),
            Span::raw(" - Previous search result"),
        ]),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("SESSIONS VIEW", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("r", Style::default().fg(Color::Green)),
            Span::raw(" - Replay session"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("Space", Style::default().fg(Color::Green)),
            Span::raw(" - Play/pause"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("+/-", Style::default().fg(Color::Green)),
            Span::raw(" - Adjust playback speed"),
        ]),
    ];
    
    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White));
    
    f.render_widget(help_paragraph, inner_area);
}