use clipboard::{ClipboardProvider, ClipboardContext};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{widgets::StatefulWidget, style::{Style, Modifier}};

pub struct Textbox {}

pub struct TextboxState {
    pub cursor_pos: usize,
    pub text: String,
    pub hint_text: Option<String>,
}

impl Default for TextboxState {
    fn default() -> Self {
        Self {
            cursor_pos: Default::default(),
            text: Default::default(),
            hint_text: Some("<enter stream address>".to_string()),
        }
    }
}

impl TextboxState {
    pub(crate) fn handle_events(&mut self, key_code: KeyCode, key_modifiers: KeyModifiers) {
        match (key_code, key_modifiers) {
            (KeyCode::Left, _) => {
                self.cursor_pos = if self.cursor_pos > 0 { self.cursor_pos - 1 } else { self.cursor_pos };
            },
            (KeyCode::Right, _) => {
                self.cursor_pos = if self.cursor_pos < self.text.len() { self.cursor_pos + 1 } else { self.text.len() };
            },
            (KeyCode::Backspace, _) => {
                if self.cursor_pos > 0 {
                    self.cursor_pos = self.cursor_pos - 1;
                    self.text.remove(self.cursor_pos);
                }
            },
            (KeyCode::Delete, _) => {
                if self.cursor_pos < self.text.len() {
                    self.text.remove(self.cursor_pos);

                    if self.cursor_pos == self.text.len() && self.text.len() > 0 {
                        self.cursor_pos = self.cursor_pos - 1;
                    }
                }
            },
            (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                let content = ctx.get_contents().unwrap_or_default();
                self.text.insert_str(self.cursor_pos, &content);
                self.cursor_pos = self.cursor_pos + content.len();
            },
            (KeyCode::Char(x), _) => {
                self.text.insert(self.cursor_pos, x);
                self.cursor_pos = self.cursor_pos + 1;
            },
            (_, _) => {}
        }
    }
}

impl StatefulWidget for Textbox {
    type State = TextboxState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        buf.set_style(area, Style::default().bg(ratatui::style::Color::Black).fg(ratatui::style::Color::Blue));
        if state.text.len() > 0 {
            buf.set_string(area.x, area.y, state.text.clone(), Style::default().bg(ratatui::style::Color::Black).fg(ratatui::style::Color::Blue));
        } else {
            if let Some(hint) = state.hint_text.as_ref() {
                buf.set_string(area.x, area.y, hint.clone(), Style::default().add_modifier(Modifier::ITALIC).bg(ratatui::style::Color::Black).fg(ratatui::style::Color::DarkGray));
            }

        }
        let pos_char = state.text.chars().nth(state.cursor_pos).unwrap_or(' ');
        buf.set_string(area.x + u16::try_from(state.cursor_pos).unwrap(), area.y, format!("{}", &pos_char), Style::default().bg(ratatui::style::Color::White));
    }
}
