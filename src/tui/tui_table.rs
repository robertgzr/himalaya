use std::io::Stdout;

use tui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Row, Table},
    Frame,
};

pub trait TuiTable<'a>
where
    Self: Sized,
{
    fn head() -> Row<'a>;
    fn row(&self) -> Row<'a>;
    fn render(frame: &mut Frame<'a, CrosstermBackend<Stdout>>, items: &[Self]) {
        let size = frame.size();
        let table = Table::new(items.iter().map(|item| item.row()).collect::<Vec<_>>())
            .header(
                Self::head().style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED),
                ),
            )
            .block(Block::default().title("Mailboxes"))
            .column_spacing(1)
            .widths(&[
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(10),
            ])
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">>");

        frame.render_widget(table, size);
    }
}

pub trait RenderTuiTable {
    fn render_tui_table<'a>(&self, frame: &mut Frame<'a, CrosstermBackend<Stdout>>);
}
