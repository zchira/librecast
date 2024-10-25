use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::Widget;
use ratatui::widgets::canvas::Canvas;
use url2audio::player_engine::Playing;

pub struct Timeline<'a> {
    pub progress: f64,
    pub progress_display: String,
    pub total: f64,
    pub total_display: String,
    pub playing: Playing,
    pub error: Option<String>,
    pub title: String,
    pub buffer: &'a Vec<(f32, f32)>
}

impl<'a> Widget for Timeline<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::buffer::Buffer)
    where Self: Sized {

        // buff
        let canvas = Canvas::default()
            .block(Block::default())
            .paint(|ctx| {

                self.buffer.iter().for_each(|c| {
                    ctx.draw(&ratatui::widgets::canvas::Rectangle {
                        x: c.0 as f64,
                        y: 0.5,
                        width: (c.1 - c.0) as f64,
                        height: 0.5,
                        color: Color::Blue
                    });

                });



            })
        .x_bounds([0.0, 1.0])
            .y_bounds([0.0, 1.0]);

        canvas.render(Rect {
            x: area.x + 1,
            y: area.y + 2,
            width: area.width - 2,
            height: 1,
        }, buf);




        let mut ratio = f64::max(0.0, self.progress / self.total);
        ratio = f64::min(ratio, 1.0);

        let play_char = match self.playing {
            Playing::Playing => "▶",
            Playing::Paused => "Ⅱ",
            Playing::Finished => "⏹",
        };

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
