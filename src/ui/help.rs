use anyhow::Result;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

/// Render help dialog
pub fn render_help<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL);
    
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    
    let help_text = vec![
        Line::from(vec![
            Span::styled("xKippo-tui: ", Style::default().fg(Color::Yellow)),
            Span::raw("Cowrie Honeypot Monitoring Tool"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("General Navigation", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  Tab/Shift+Tab: ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch between tabs"),
        ]),
        Line::from(vec![
            Span::styled("  1-4: ", Style::default().fg(Color::Yellow)),
            Span::raw("Select tab directly"),
        ]),
        Line::from(vec![
            Span::styled("  q: ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit the application"),
        ]),
        Line::from(vec![
            Span::styled("  ?: ", Style::default().fg(Color::Yellow)),
            Span::raw("Show/hide this help dialog"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Dashboard", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  The dashboard shows an overview of honeypot activity.", Style::default()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Logs View", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓: ", Style::default().fg(Color::Yellow)),
            Span::raw("Navigate log entries"),
        ]),
        Line::from(vec![
            Span::styled("  Enter: ", Style::default().fg(Color::Yellow)),
            Span::raw("View log details"),
        ]),
        Line::from(vec![
            Span::styled("  Esc: ", Style::default().fg(Color::Yellow)),
            Span::raw("Close details view"),
        ]),
        Line::from(vec![
            Span::styled("  /: ", Style::default().fg(Color::Yellow)),
            Span::raw("Search logs"),
        ]),
        Line::from(vec![
            Span::styled("  f: ", Style::default().fg(Color::Yellow)),
            Span::raw("Filter logs"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Sessions View", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓: ", Style::default().fg(Color::Yellow)),
            Span::raw("Navigate sessions"),
        ]),
        Line::from(vec![
            Span::styled("  Enter: ", Style::default().fg(Color::Yellow)),
            Span::raw("View session details"),
        ]),
        Line::from(vec![
            Span::styled("  Esc: ", Style::default().fg(Color::Yellow)),
            Span::raw("Close details view"),
        ]),
        Line::from(vec![
            Span::styled("  c/f: ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch between Commands and Files tabs"),
        ]),
        Line::from(vec![
            Span::styled("  s: ", Style::default().fg(Color::Yellow)),
            Span::raw("Show session summary"),
        ]),
        Line::from(vec![
            Span::styled("  r: ", Style::default().fg(Color::Yellow)),
            Span::raw("Replay session (TTY logs)"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Settings View", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓: ", Style::default().fg(Color::Yellow)),
            Span::raw("Navigate settings"),
        ]),
        Line::from(vec![
            Span::styled("  Tab: ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch between categories and settings"),
        ]),
        Line::from(vec![
            Span::styled("  Enter: ", Style::default().fg(Color::Yellow)),
            Span::raw("Edit selected setting"),
        ]),
        Line::from(vec![
            Span::styled("  Esc: ", Style::default().fg(Color::Yellow)),
            Span::raw("Exit edit mode"),
        ]),
        Line::from(vec![
            Span::styled("  s: ", Style::default().fg(Color::Yellow)),
            Span::raw("Save settings"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press any key to close help", Style::default().fg(Color::White)),
        ]),
    ];
    
    let help_paragraph = Paragraph::new(help_text)
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(help_paragraph, inner_area);
}