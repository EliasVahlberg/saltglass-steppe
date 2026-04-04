use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub struct AriaInterface {
    pub response_text: String,
    pub options: Vec<String>,
    pub selected_option: usize,
}

impl Default for AriaInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl AriaInterface {
    pub fn new() -> Self {
        Self {
            response_text: String::new(),
            options: Vec::new(),
            selected_option: 0,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),                             // Header
                Constraint::Min(5),                                // Response area
                Constraint::Length(self.options.len() as u16 + 2), // Options
            ])
            .split(area);

        // Terminal header
        let header = Paragraph::new("ARIA - Archive Intelligence")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title("Terminal"),
            )
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(header, chunks[0]);

        // Response text area
        let response = Paragraph::new(self.response_text.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::Cyan))
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(response, chunks[1]);

        // Command options
        let option_items: Vec<ListItem> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let style = if i == self.selected_option {
                    Style::default().fg(Color::Black).bg(Color::White)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(format!("> {}", option), style)))
            })
            .collect();

        let options_list = List::new(option_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Commands"),
        );
        frame.render_widget(options_list, chunks[2]);
    }
}
