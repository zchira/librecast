use ratatui::{prelude::*, widgets::*};
use ratatui::style::Color;
use tokio::time::Instant;

pub struct WaitingMessageDialog {
    pub message: String,
    pub fg_color: Color,
    pub bg_color: Color
}

impl WaitingMessageDialog {
    pub fn new(message: String) -> Self {
        WaitingMessageDialog {
            message,
            fg_color: Color::White,
            bg_color: Color::Blue
        }
    }
}

pub struct WaitingMessageDialogState {
    start_time: Instant
}

impl Default for WaitingMessageDialogState {
    fn default() -> Self {
        WaitingMessageDialogState {
            start_time: Instant::now()
        }
    }
}

static PROGRESS: &str = "|/-\\";

impl StatefulWidget for WaitingMessageDialog {
    type State = WaitingMessageDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let dialog_area = Rect {
            x: area.x + 3,
            y: area.y + 3,
            width: std::cmp::min(area.width - 4, self.message.len() as u16 + 2),
            height: 3
        };

        let now = Instant::now();
        let d = now - state.start_time;
        let rem = d.as_secs() % 4;

        let title = PROGRESS.chars().nth(rem as usize).unwrap();
        let block = Block::default().borders(Borders::all()).bg(self.bg_color).fg(self.fg_color).title(title.to_string());
        block.render(dialog_area, buf);
        buf.set_string(dialog_area.x + 1, dialog_area.y + 1, self.message, Style::default().bg(self.bg_color).fg(self.fg_color));
    }
}
