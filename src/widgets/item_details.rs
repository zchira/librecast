use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style, Stylize}, text::{Line, Span, Text}, widgets::{Block, Borders, Paragraph, Widget, Wrap}};

use crate::ui_models::ChannelItem;


pub struct ItemDetails<'a> {
    pub currently_playing: bool,
    pub item: &'a Option<&'a ChannelItem>,
    pub fg_color: Color
}

impl<'a> Widget for ItemDetails<'a> {

    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::all()).fg(self.fg_color);
        block.render(area, buf);

        let height = 8;
        let style = Style::default().fg(self.fg_color);
        let no_data = "-".to_string();
        let mut lines = vec![];
        let empty = Line::default();
        match self.item {
            Some(item) => {
                if self.currently_playing {
                    let playing = Line::from(vec![
                        Span::styled("▶", style.blue()),
                    ]);
                    lines.push(playing);
                } else {
                    match item.listening_state.as_ref() {
                        Some(s) => {
                            let playing = Line::from(vec![
                                Span::styled(format!("[{}]", time_to_display(s.time)), style.blue()),
                            ]);
                            lines.push(playing);
                        },
                        None => {},
                    }
                }

                let t = item.title.as_ref().unwrap_or(&no_data);
                let title = Line::from(vec![
                    Span::styled("title: ", style.italic().dark_gray()),
                    Span::styled(t, style.dark_gray()),
                ]);
                lines.push(title);
                lines.push(empty.clone());

                let pd = item.pub_date.as_ref().unwrap_or(&Default::default()).format("%v");
                let pub_date = Line::from(vec![
                    Span::styled("date: ", style.italic().dark_gray()),
                    Span::styled(pd.to_string(), style.dark_gray()),
                ]);
                lines.push(pub_date);
                lines.push(empty.clone());

                let p = Paragraph::new(lines).wrap(Wrap { trim: true });
                p.render(Rect {
                    x: area.x + 1,
                    y: area.y + 1,
                    width: area.width - 2,
                    height,
                }, buf);

                let d = item.description.as_ref().unwrap_or(&no_data);
                let d = d.to_string();

                let t = Text::from(d); //.to_line();
                let p = Paragraph::new(t).wrap(Wrap { trim: true });

                p.render(Rect {
                    x: area.x + 1,
                    y: area.y + 1 + height,
                    width: area.width - 2,
                    height: area.height - 2 - height,
                }, buf);

            },
            None => {
                let line = Line::from(vec![
                    Span::styled("-", Style::default())
                ]);
                line.render(Rect {
                    x: area.x + 1,
                    y: area.y + 1,
                    width: area.width - 2,
                    height: area.height - 2,
                }, buf);
            },
        }
    }
}


fn time_to_display(seconds: f32) -> String {
    let is: i64 = seconds.round() as i64;
    let hours = is / (60 * 60);
    let mins = (is % (60 * 60)) / 60;
    let secs = seconds - 60.0 * mins as f32 - 60.0 * 60.0 * hours as f32; // is % 60;
    format!("{}:{:0>2}:{:0>4.1}", hours, mins, secs)
}
