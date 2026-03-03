use duplff_core::models::DuplicateGroup;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;
use std::collections::HashSet;
use std::path::PathBuf;

/// Render the detail pane showing files in the selected group.
pub fn render_detail(
    frame: &mut Frame,
    area: Rect,
    group: Option<&DuplicateGroup>,
    cursor: usize,
    focused: bool,
    marked: &HashSet<PathBuf>,
) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let Some(group) = group else {
        let block = Block::default()
            .title(" Group Detail ")
            .borders(Borders::ALL)
            .border_style(border_style);
        frame.render_widget(block, area);
        return;
    };

    let total_files = 1 + group.duplicates.len();
    let title = format!(" Group Detail [{} files] ", total_files);

    // Build list items: keep file first, then duplicates
    let mut items: Vec<ListItem> = Vec::with_capacity(total_files);

    // Keep file
    let keep_path = group.keep.entry.path.display().to_string();
    let keep_line = Line::from(vec![
        Span::styled(
            "[KEEP] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(keep_path),
    ]);
    let reason_line = Line::from(vec![
        Span::raw("       Reason: "),
        Span::styled(
            group.keep.reason.to_string(),
            Style::default().fg(Color::Green),
        ),
    ]);
    items.push(ListItem::new(vec![keep_line, reason_line]));

    // Duplicate files
    for dup in &group.duplicates {
        let is_marked = marked.contains(&dup.entry.path);
        let dup_path = dup.entry.path.display().to_string();

        let (tag, style) = if is_marked {
            ("[DEL]  ", Style::default().fg(Color::Red))
        } else {
            ("[   ]  ", Style::default())
        };

        let line = Line::from(vec![
            Span::styled(tag, style.add_modifier(Modifier::BOLD)),
            Span::styled(dup_path, style),
        ]);
        items.push(ListItem::new(vec![line]));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(cursor));
    frame.render_stateful_widget(list, area, &mut list_state);
}
