use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::Widget;

pub struct Timeline {
    pub progress: f64,
    pub progress_display: String,
    pub total: f64,
    pub total_display: String,
    pub playing: bool,
    pub error: Option<String>,
    pub title: String
}

impl Widget for Timeline {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::buffer::Buffer)
    where Self: Sized {
        let ratio = if self.total > 0.0 {
            self.progress / self.total
        } else {
            0.0
        };


        let play_char = if self.playing  {"▶" } else { "Ⅱ" };

        let playing_line = Line::from(vec![
            Span::styled(format!("{} {}", play_char, self.title), Style::default().fg(ratatui::style::Color::Blue)),
        ]);
        // Line::from(vec![
        //     Span::styled(format!("■"), Style::default()),
        // ])
        let error_line = if self.error.is_some() {
            Line::from(vec![
                Span::styled(format!("Err: {}", self.error.clone().unwrap()), Style::default().fg(ratatui::style::Color::Red)),
            ])
        } else {
            Line::default()
        };

        let gauge = Gauge::default()
            .block(Block::bordered().title(playing_line).title_bottom(error_line)) //"Progress"))
            .label(format!("{} - {}", self.progress_display, self.total_display))
            .gauge_style(
                Style::default()
                .fg(Color::Blue)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            )
            .use_unicode(true)
            .ratio(ratio);
        gauge.render(area, buf);

    }
}
