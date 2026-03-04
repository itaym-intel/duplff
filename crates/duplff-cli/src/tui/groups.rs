use crate::format::human_bytes;
use duplff_core::models::DuplicateGroup;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;

/// Render the duplicate groups table in the given area.
pub fn render_groups(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    groups: &[&DuplicateGroup],
    cursor: usize,
    focused: bool,
) {
    let header = Row::new(vec![
        Cell::from("#"),
        Cell::from("Files"),
        Cell::from("Size"),
        Cell::from("Wasted"),
        Cell::from("Sample Path"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD))
    .height(1);

    let rows: Vec<Row> = groups
        .iter()
        .enumerate()
        .map(|(i, group)| {
            let file_count = 1 + group.duplicates.len();
            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(format!("{}", file_count)),
                Cell::from(human_bytes(group.size)),
                Cell::from(human_bytes(group.wasted_bytes())),
                Cell::from(group.keep.entry.path.display().to_string()),
            ])
        })
        .collect();

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Duplicate Groups ")
            .borders(Borders::ALL)
            .border_style(border_style),
    )
    .row_highlight_style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::DarkGray),
    );

    let mut table_state = TableState::default();
    if !groups.is_empty() {
        table_state.select(Some(cursor));
    }
    frame.render_stateful_widget(table, area, &mut table_state);
}
