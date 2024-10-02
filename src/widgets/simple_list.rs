use ratatui::{prelude::*, widgets::*};
use ratatui::style::Color;

use crate::ui_models;

pub struct SimpleList<'a> {
    pub items: &'a Vec<ui_models::ChannelItem>,
    pub active: &'a Option<ui_models::ChannelItem>,
    pub fg_color: Color
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
            let text = &self.items[i].title.clone().unwrap_or("".to_string());
            let style = if i == sel_index {
                Style::default().fg(self.fg_color).reversed()
            } else {
                Style::default().fg(self.fg_color)
            };

            buf.set_string(area.x + 1,area.y + dx , text.to_string(), style);
            dx = dx + 1;
        }

    }
}
