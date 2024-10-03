use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, ListState, StatefulWidget, Widget}};

use crate::ui_models;

pub struct SimpleList<'a> {
    pub items: &'a Vec<ui_models::ChannelItem>,
    pub active: &'a Option<ui_models::ChannelItem>,
    pub fg_color: Color
}

enum ItemState {
    Finished,
    InProgress(f32),
    None
}

impl<'a> StatefulWidget for SimpleList<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default().borders(Borders::all()).fg(self.fg_color);
        block.render(area, buf);

        if self.items.is_empty() {
            return;
        }

        let num_of_visible = (area.height - 2) as usize; 
        let mut sel_index = usize::MAX;

        if let Some(selected) = state.selected() {
            sel_index = selected;
            if selected < state.offset() {
                *state.offset_mut() = selected;
            }

            if selected >= state.offset() + num_of_visible {
                *state.offset_mut() = selected - num_of_visible + 1;
            }
        }

        let start = state.offset();
        let end = if self.items.len() - 1 < start + num_of_visible { self.items.len() - 1 } else { start + num_of_visible };

        let mut dx = 1;
        for i in start..end {
            let item = &self.items[i];

            let playing = match self.active.as_ref() {
                Some(a) => item.channel_id == a.channel_id && item.enclosure == a.enclosure,
                None => false
            };

            let playing_prefix = match playing {
                true => if playing { "▶ " } else { "" },
                false => "",
            };

            let text = format!("{}{}", playing_prefix, &item.title.clone().unwrap_or("".to_string()));

            let area: Rect = Rect {
                x: area.x + 1,
                y: area.y + dx,
                width: area.width,
                height: area.height,
            };
            dx = dx + 1;

            let style = if i == sel_index {
                Style::default().fg(self.fg_color).reversed()
            } else {
                Style::default().fg(self.fg_color)
            };

            let item_state = match self.items[i].listening_state.as_ref() {
                Some(ls) => if ls.finished { ItemState::Finished } else { ItemState::InProgress(ls.time) },
                None => { ItemState::None },
            };

            let line = match item_state {
                ItemState::Finished => {
                    Line::from(vec![
                        Span::styled(text, style.italic().dark_gray())
                    ])
                },
                ItemState::InProgress(t) => {
                    Line::from(vec![
                        if playing { Span::default() } else { Span::styled(format!("[{}] ", time_to_display(t)), style) },
                        Span::styled(text, style.italic())
                    ])
                },
                ItemState::None => {
                    Line::from(vec![
                        Span::styled(text, style)
                    ])
                }
            };

            line.render(area, buf);
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
