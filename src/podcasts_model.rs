use std::{io::ErrorKind, str::FromStr, sync::{Arc, RwLock}};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, List, ListState, Paragraph}, Frame};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};
use tokio::sync::mpsc::UnboundedSender;
use tui_textbox::{Textbox, TextboxState};

use std::error::Error;
use rss::Channel;
use crate::{entity::channel::Entity as ChannelEntity, AsyncAction};

use crate::{config, player_engine::PlayerEngine};

pub struct PodcastsModel {
    pub list_state_channels: ListState,
    pub active_channel: Option<String>,
    pub list_state_items: ListState,
    pub active_item: Option<String>,
    pub podcasts_collection: Vec<String>,
    pub items_collection: Vec<String>,
    pub error: Option<String>,
    pub show_open_dialog: bool,
    pub textbox_state: TextboxState,
    pub player_engine: Arc<RwLock<PlayerEngine>>,
    help_visible: bool,
    db: DatabaseConnection,
    tx: UnboundedSender<crate::AsyncAction>
}


impl PodcastsModel {
    pub fn new(db: DatabaseConnection, tx: UnboundedSender<crate::AsyncAction>) -> Self {
        Self {
            list_state_channels: Default::default(),
            podcasts_collection: vec!["dasko i mladja".to_string(), "agelas".to_string()], // Default::default(),
            help_visible: Default::default(),
            active_channel: Default::default(),
            list_state_items: Default::default(),
            items_collection: Default::default(),
            active_item: Default::default(),
            error: Default::default(),
            show_open_dialog: Default::default(),
            textbox_state: Default::default(),
            player_engine: Default::default(),
            db,
            tx
        }
    }

    pub async fn get_channels_from_db(&self) -> Result<(), DbErr>  {
        let channels = ChannelEntity::find().all(&self.db).await?;


        Ok(())

    }

    pub async fn get_channel_from_url() -> Result<Channel, Box<dyn Error>> {
        let content = ureq::get("https://podcast.daskoimladja.com/feed.xml").call()?.into_string()?;
        // let content = ureq::get("https://feeds.transistor.fm/agelast-podcast").call()?.into_string()?;

        let channel = Channel::from_str(&content)?;
        Ok(channel)
    }

    pub fn ui(&mut self, rect: Rect, f: &mut Frame) {
        let size = rect;

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Min(3)])
            .split(size);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]).split(vertical_chunks[0]);

        let status_block = Block::default().borders(Borders::ALL).title(format!("status"));

        // list channels
        let active_channel = match self.active_channel.as_ref() {
            Some(s) => s.clone(),
            None => "".to_string(),
        };
        let list = List::new(self.podcasts_collection.clone().into_iter().map(|i| {
            if i == active_channel {
                format!(">{}<", i)
            } else {
                format!("{}", i)
            }
        }))
        .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ")
            .repeat_highlight_symbol(true);


        f.render_stateful_widget(list, horizontal_chunks[0], &mut self.list_state_channels);

        // list items
        let active_item = match self.active_item.as_ref() {
            Some(s) => s.clone(),
            None => "".to_string(),
        };
        let list = List::new(self.items_collection.clone().into_iter().map(|i| {
            if i == active_item {
                format!("{} 🎵", i)
            } else {
                format!("{}", i)
            }
        }))
        .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ")
            .repeat_highlight_symbol(true);

        f.render_stateful_widget(list, horizontal_chunks[1], &mut self.list_state_items);

        let status_line = match self.error.as_ref() {
            Some(e) => {
                Line::from(vec![
                           Span::styled(format!("Error: {}", e.to_string()), Style::default().fg(ratatui::style::Color::Red)),
                ])
            },
            None => {
                match self.active_channel.as_ref() {
                    Some(s) => {
                        let play_char = if self.player_engine.read().unwrap().is_paused() { "Ⅱ" } else { "▶" };
                        Line::from(vec![
                                   Span::styled(format!("{} {}", play_char, s), Style::default().fg(ratatui::style::Color::Blue)),
                        ])
                    },
                    None => {
                        Line::from(vec![
                                   Span::styled(format!("■"), Style::default()),
                        ])
                    }
                }
            },
        };

        let status_paragraph = Paragraph::new(vec![status_line]).block(status_block);
        f.render_widget(status_paragraph, vertical_chunks[1]);

        let volume = self.player_engine.read().unwrap().get_volume() * 100.0;
        let volume_line = Line::from(vec![Span::styled(format!("Volume: {:.0}%", volume), Style::default().fg(ratatui::style::Color::Blue))]);
        let volume_paragraph = Paragraph::new(vec![volume_line]);

        let volume_area = Rect::new(vertical_chunks[1].width - 13, vertical_chunks[1].y + 1, 13, 1);

        f.render_widget(volume_paragraph, volume_area);

        if self.show_open_dialog {
            let w = size.width - 5;
            let h = 8;
            let x = (size.width - w) / 2;
            let y = (size.height - h) / 3;
            let open_dialog_block = Block::default().borders(Borders::ALL).title("Open stream").bg(Color::DarkGray);

            let dialog_rect = Rect { width: w, height: h, x, y };
            f.render_widget(open_dialog_block, dialog_rect);

            let textbox = Textbox::default();

            f.render_stateful_widget(textbox, Rect::new(x + 1, y + 1, w - 2, 1), &mut self.textbox_state);

            let mut lines = vec![];

            let line = Line::from(vec![Span::styled("<Enter> - add stream", Style::default())]);
            lines.push(line);

            let line = Line::from(vec![Span::styled("<Esc> - cancel", Style::default())]);
            lines.push(line);

            let open_dialog_paragraph = Paragraph::new(lines); //.block(open_dialog_block);
            let open_dialog_rect = Rect::new(x + 1, y + 3, w, h);
            f.render_widget(open_dialog_paragraph, open_dialog_rect);
        }

        if self.help_visible {
            let w = 50;
            let h = 10;
            let x = (size.width - w) / 2;
            let y = (size.height - h) / 2;
            let help_block = Block::default().borders(Borders::ALL).title("Help").bg(Color::DarkGray);

            let mut lines = vec![];
            let line = Line::from(vec![Span::styled("h - toggle help", Style::default())]);
            lines.push(line);

            let line = Line::from(vec![Span::styled("1|2|<tab> - toggle tabs", Style::default())]);
            lines.push(line);

            let line = Line::from(vec![Span::styled("<enter> - open and play stream", Style::default())]);
            lines.push(line);

            let line = Line::from(vec![Span::styled("<space> - play/pause stream", Style::default())]);
            lines.push(line);

            let line = Line::from(vec![Span::styled("+/- - volume up/down", Style::default())]);
            lines.push(line);

            let line = Line::from(vec![Span::styled("o - add stream to collection", Style::default())]);
            lines.push(line);

            let line = Line::from(vec![Span::styled("d|<del> - remove stream from collection", Style::default())]);
            lines.push(line);

            let help_paragraph = Paragraph::new(lines).block(help_block);
            let help_rect = Rect::new(x, y, w, h);
            f.render_widget(help_paragraph, help_rect);
        }
    }


    pub async fn handle_events(&mut self, key: KeyEvent) -> std::io::Result<bool> {
        if self.show_open_dialog {
            self.handle_open_dialog_events(key)
        } else {
            match key.code {
                KeyCode::Char('o') => {
                    self.show_open_dialog = true;
                },
                KeyCode::Char('r') => {
                    self.items_collection.clear();
                    self.items_collection.push("--reading--".to_string());
                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        // tokio::time::sleep(Duration::from_secs(5)).await; // simulate network request
                        let channels = PodcastsModel::get_channel_from_url().await.map_err(|e| std::io::Error::new(ErrorKind::Other, "")).unwrap();
                        tx.send(AsyncAction::Channel(channels.clone()));
                    });



                    // self.items_collection.clear();
                    // channels.items().iter().for_each(|i| self.items_collection.push(i.title().unwrap_or("-").to_string()));

                }
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Enter => {
                    match self.active_channel {
                        Some(_) => {
                            self.player_engine = Arc::new(RwLock::new(PlayerEngine::new()));
                            self.active_channel = None;
                        },
                        None => {
                            let selected_stream = &self.podcasts_collection[self.list_state_channels.selected().unwrap_or_default()];
                            self.active_channel = Some(selected_stream.clone());
                            let mut p = self.player_engine.write().unwrap();
                            match p.open(selected_stream) {
                                Ok(_) => {
                                    self.error = None;
                                },
                                Err(e) => self.error = Some(e.to_string()),
                            }
                        },
                    }
                },
                KeyCode::Char(' ') => {
                    if self.active_channel.is_some() {
                        let p = self.player_engine.write().unwrap();
                        if p.is_paused() {
                            p.resume()
                        } else {
                            p.pause()
                        }
                    }
                },
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    let mut p = self.player_engine.write().unwrap();
                    p.increase_volume();
                },
                KeyCode::Char('-') => {
                    let mut p = self.player_engine.write().unwrap();
                    p.decrease_volume();
                },
                KeyCode::Down => {
                    let mut selected = self.list_state_channels.selected().unwrap_or_default();
                    let len = self.podcasts_collection.len();
                    selected = if selected >= len - 1 { 0 } else { selected + 1 };
                    self.list_state_channels.select(Some(selected));
                },
                KeyCode::Up => {
                    let mut selected = self.list_state_channels.selected().unwrap_or_default();
                    let len = self.podcasts_collection.len();
                    selected = if selected <= 0 { len - 1 } else { selected - 1 };
                    self.list_state_channels.select(Some(selected));
                },
                KeyCode::Char('d') | KeyCode::Delete => {
                    let selected = self.list_state_channels.selected().unwrap_or_default();
                    self.podcasts_collection.remove(selected);
                    if selected >= self.podcasts_collection.len() {
                        self.list_state_channels.select(Some(selected - 1));
                    }
                    config::save(self.podcasts_collection.clone())?;
                },
                KeyCode::Char('h') => {
                    self.help_visible = !self.help_visible;
                },
                _ => {}
            }
            Ok(false)
        }
    }

    fn handle_open_dialog_events(&mut self, key: KeyEvent) -> std::io::Result<bool> {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => self.show_open_dialog = false,
            (KeyCode::Enter, _) => {
                self.podcasts_collection.push(self.textbox_state.text.clone());
                // config::save(self.podcasts_collection.clone())?;
                self.show_open_dialog = false;
            },
            (key_code, key_modifiers) => {
                self.textbox_state.handle_events(key_code, key_modifiers);
            }
        }

        Ok(false)
    }

}
