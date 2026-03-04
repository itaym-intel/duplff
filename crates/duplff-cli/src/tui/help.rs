use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Render the help overlay centered in the given area.
pub fn render_help(frame: &mut Frame, area: Rect) {
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Key", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("          "),
            Span::styled("Action", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("j / Down    ", Style::default().fg(Color::Yellow)),
            Span::raw("Move cursor down"),
        ]),
        Line::from(vec![
            Span::styled("k / Up      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move cursor up"),
        ]),
        Line::from(vec![
            Span::styled("Tab         ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch focus (groups / detail)"),
        ]),
        Line::from(vec![
            Span::styled("Enter       ", Style::default().fg(Color::Yellow)),
            Span::raw("Select group, focus detail"),
        ]),
        Line::from(vec![
            Span::styled("Space       ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle file for deletion"),
        ]),
        Line::from(vec![
            Span::styled("D           ", Style::default().fg(Color::Yellow)),
            Span::raw("Mark all duplicates in group"),
        ]),
        Line::from(vec![
            Span::styled("u           ", Style::default().fg(Color::Yellow)),
            Span::raw("Unmark all in group"),
        ]),
        Line::from(vec![
            Span::styled("d           ", Style::default().fg(Color::Yellow)),
            Span::raw("Trash marked files (confirms first)"),
        ]),
        Line::from(vec![
            Span::styled("/           ", Style::default().fg(Color::Yellow)),
            Span::raw("Filter groups by path"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("q / Esc     ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
        Line::from(vec![
            Span::styled("?           ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help"),
        ]),
    ];

    // Center a box in the terminal
    let width = 50.min(area.width.saturating_sub(4));
    let height = (help_lines.len() as u16 + 2).min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let help_area = Rect::new(x, y, width, height);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    let paragraph = Paragraph::new(help_lines).block(block);

    frame.render_widget(Clear, help_area);
    frame.render_widget(paragraph, help_area);
}
