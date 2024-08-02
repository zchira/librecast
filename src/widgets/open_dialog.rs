use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use ratatui::style::Color;
use tui_textbox::{Textbox, TextboxState};

pub struct OpenDialog {
    pub fg_color: Color,
    pub bg_color: Color,
    pub title: String,
}

impl OpenDialog {
    pub fn new(title: String) -> Self {
        OpenDialog {
            title,
            fg_color: Color::White,
            bg_color: Color::Black,
        }
    }
}

pub struct OpenDialogState {
    textbox_state: TextboxState
}

impl Default for OpenDialogState {
    fn default() -> Self {
        let mut textbox_state = TextboxState::default();
        textbox_state.hint_text = Some("<enter address...>".to_string());
        OpenDialogState {
            textbox_state
        }
    }
}

impl OpenDialogState {
    pub fn handle_events(&mut self, key_code: KeyCode, key_modifiers: KeyModifiers) {
        self.textbox_state.handle_events(key_code, key_modifiers)
    }

    pub fn text(&self) -> String {
        self.textbox_state.text.clone()
    }

    pub fn clear(&mut self) {
        self.textbox_state.text = String::new();
        self.textbox_state.cursor_pos = 0;
    }
}

impl StatefulWidget for OpenDialog {
    type State = OpenDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let width = area.width - 5;
        let height = 8;
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 3;
        let dialog_area = Rect {
            x,
            y,
            width,
            height
        };
        let clear = Clear::default();
        clear.render(dialog_area, buf);

        let block = Block::default().borders(Borders::all()).bg(self.bg_color).fg(self.fg_color).title(self.title);
        block.render(dialog_area, buf);

        let text_box_rect = Rect::new(x + 1, y + 1, width - 2, 1);
        let textbox = Textbox::default();
        textbox.render(text_box_rect, buf, &mut state.textbox_state);

        let mut lines = vec![];

        let line = Line::from(vec![Span::styled("<Enter> - add stream", Style::default())]);
        lines.push(line);

        let line = Line::from(vec![Span::styled("<Esc> - cancel", Style::default())]);
        lines.push(line);

        let open_dialog_paragraph = Paragraph::new(lines);
        let open_dialog_rect = Rect::new(x + 1, y + 3, width, height);
        open_dialog_paragraph.render(open_dialog_rect, buf);
    }
}
