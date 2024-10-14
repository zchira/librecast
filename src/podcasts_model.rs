use std::{borrow::BorrowMut, io::ErrorKind, str::FromStr, sync::{Arc, RwLock}};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, List, ListState, Paragraph}, Frame};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};
use tokio::sync::mpsc::UnboundedSender;

use std::error::Error;
use rss::Channel;
use crate::{data_layer::data_provider::DataProvider, entity::channel::Entity as ChannelEntity, ui_models::{self, ListeningState}, widgets::{item_details::ItemDetails, open_dialog::{OpenDialog, OpenDialogState}, simple_list::SimpleList, timeline::Timeline, waiting_message_dialog::{WaitingMessageDialog, WaitingMessageDialogState}}, AsyncAction};

use crate::player_engine::PlayerEngine;
use crate::entity::channel::Model as ChannelModel;

pub struct PodcastsModel {
    db: DatabaseConnection,
    help_visible: bool,
    pub active_channel: Option<ChannelModel>,
    pub active_item: Option<ui_models::ChannelItem>,
    pub error: Option<String>,
    pub items_collection: Vec<ui_models::ChannelItem>,
    pub list_state_channels: ListState,
    pub list_state_items: ListState,
    pub active_list_state: usize,
    pub player_engine: Arc<RwLock<PlayerEngine>>,
    pub podcasts_collection: Vec<ChannelModel>,
    pub show_open_dialog: bool,
    tx: UnboundedSender<crate::AsyncAction>,
    pub waiting_dialog_state: WaitingMessageDialogState,
    pub waiting_message: Option<String>,
    pub open_dialog_state: OpenDialogState,
}

impl PodcastsModel {
    pub fn new(db: DatabaseConnection, tx: UnboundedSender<crate::AsyncAction>) -> Self {
        let mut list_state_channels: ListState = Default::default();
        list_state_channels.select(Some(0));
        Self {
            active_channel: Default::default(),
            active_item: Default::default(),
            db,
            error: Default::default(),
            help_visible: Default::default(),
            items_collection: Default::default(),
            list_state_channels,
            list_state_items: Default::default(),
            active_list_state: 0,
            player_engine: Default::default(),
            podcasts_collection: vec![],
            show_open_dialog: Default::default(),
            tx,
            waiting_dialog_state: Default::default(),
            waiting_message: None,
            open_dialog_state: Default::default()
        }
    }

    pub async fn get_channels_from_db(&self) -> Result<Vec<ChannelModel>, DbErr>  {
        let channels = ChannelEntity::find().all(&self.db).await?;
        Ok(channels)
    }

    pub async fn get_channel_from_url(url: &str) -> Result<Channel, Box<dyn Error>> {
        let content = ureq::get(url).call()?.into_string()?;
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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]).split(horizontal_chunks[0]);

        let channels_chunk = chunks[0];
        let channel_items_chunk = chunks[1];
        let item_details_chunk = horizontal_chunks[1];

        // list channels
        let fg_color  = |i: usize| if self.active_list_state == i { ratatui::style::Color::Blue } else { ratatui::style::Color::DarkGray };

        let list = List::new(self.podcasts_collection.clone().into_iter().map(|i| i.title.unwrap_or("-".to_string())))
        .fg(fg_color(0))
        .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ")
            .repeat_highlight_symbol(true);

        f.render_stateful_widget(list, channels_chunk, &mut self.list_state_channels);

        // list channel items
        let simple_list = SimpleList {
            items: &self.items_collection,
            active: &self.active_item,
            fg_color: fg_color(1),
        };

        f.render_stateful_widget(simple_list, channel_items_chunk, &mut self.list_state_items);

        // item details
        let selected_episode = match self.list_state_items.selected() {
            Some(i) => {
                self.items_collection.get(i).clone()
            },
            None => None,
        };
        let currently_playing = match (self.active_item.as_ref(), selected_episode) {
            (Some(a), Some(b)) => a.enclosure == b.enclosure,
            (_, _) => false,
        };

        let item_details = ItemDetails {
            currently_playing,
            item: &selected_episode,
            fg_color: ratatui::style::Color::DarkGray
        };
        f.render_widget(item_details, item_details_chunk);

        // timeline
        {
            let title = if self.active_item.is_some() {
                self.active_item.clone().unwrap().title.unwrap_or("".to_string())
            } else { 
                "".to_string()
            };
            let p = self.player_engine.read().unwrap();
            let timeline = Timeline {
                progress: p.current_position(),
                progress_display: p.current_position_display(),
                total: p.duration(),
                total_display: p.duration_display(),
                playing: !(p.is_paused()),
                error: p.get_error(),
                title
            };
            f.render_widget(timeline, vertical_chunks[1]);
        }

        if let Some(waiting_message) = self.waiting_message.clone() {
            let waiting = WaitingMessageDialog::new(waiting_message);
            f.render_stateful_widget(waiting, vertical_chunks[0], &mut self.waiting_dialog_state);
        }

        if self.show_open_dialog {
            let open_dialog = OpenDialog::new("Add new podcast".to_string());
            f.render_stateful_widget(open_dialog, size, &mut self.open_dialog_state);
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
                    self.open_dialog_state.clear();
                    self.show_open_dialog = true;
                },
                KeyCode::Char('r') => {
                    self.items_collection.clear();
                    let tx = self.tx.clone();
                    let db = self.db.clone();

                    if let Some(selected) = self.list_state_channels.selected() {
                        let selected_channel = self.podcasts_collection[selected].clone();
                        if let Some(podcast_url) = selected_channel.link {
                            self.waiting_message = Some("Fetching podcast info...".to_string());
                            tokio::spawn(async move {
                                match DataProvider::fetch_data(podcast_url, selected_channel.id, db).await {
                                    Ok(channel_id) => {
                                        tx.send(AsyncAction::ChannelAdded(channel_id)).map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string())).unwrap();
                                        tx.send(AsyncAction::RefreshChannelsList).map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string())).unwrap();
                                    },
                                    Err(e) => {
                                        // handle fetch error
                                        println!("Err fetching {:#?}", e)
                                    },
                                };
                            });
                        }
                    };
                }
                KeyCode::Char('.') => {
                    if self.active_item.is_some() {
                        let p = self.player_engine.read().unwrap();
                        p.seek_forward();
                    }
                }
                KeyCode::Char(',') => {
                    if self.active_item.is_some() {
                        let p = self.player_engine.read().unwrap();
                        p.seek_backward();
                    }
                }
                KeyCode::Char(' ') => {
                    if self.active_item.is_some() {
                        let mut p = self.player_engine.write().unwrap();
                        if p.is_paused() {
                            p.resume();
                        } else {
                            p.pause();
                            self.write_listening_state(p.current_position() as f32);
                        }
                    }
                }
                KeyCode::Left | KeyCode::Right => { 
                    self.active_list_state = (self.active_list_state + 1) % 2;
                }
                KeyCode::Enter => {
                    if self.active_list_state == 0 {
                        // load items then move items list
                        if let Some(selected) = self.list_state_channels.selected() {
                            let selected_channel = self.podcasts_collection[selected].clone();
                            self.tx.send(AsyncAction::ChannelAdded(selected_channel.id)).map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string())).unwrap();
                        }
                        self.active_list_state = self.active_list_state + 1;
                    } else {
                        let mut p = self.player_engine.write().unwrap();
                        self.write_listening_state(p.current_position() as f32);
                        let selected_episode = &self.items_collection[self.list_state_items.selected().unwrap_or_default()];
                        self.active_item = Some(selected_episode.clone());

                        match p.open(&selected_episode.enclosure) {
                            Ok(_) => {
                                self.error = None;
                                match selected_episode.listening_state.as_ref() {
                                    Some(ls) => {
                                        p.seek(ls.time as f64);
                                    },
                                    None => (),
                                }
                            },
                            Err(e) => self.error = Some(e.to_string()),
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
                    let len = self.list_state_len();
                    let list_state = match self.active_list_state {
                        0 => self.list_state_channels.borrow_mut(),
                        _ => self.list_state_items.borrow_mut(),
                    };
                    let mut selected = list_state.selected().unwrap_or_default();
                    selected = if selected >= len - 1 { 0 } else { selected + 1 };
                    list_state.select(Some(selected));
                },
                KeyCode::Up => {
                    let len = self.list_state_len();
                    let list_state = match self.active_list_state {
                        0 => self.list_state_channels.borrow_mut(),
                        _ => self.list_state_items.borrow_mut(),

                    };
                    let mut selected = list_state.selected().unwrap_or_default();
                    selected = if selected <= 0 { len - 1 } else { selected - 1 };
                    list_state.select(Some(selected));
                },
                KeyCode::Char('d') | KeyCode::Delete => {
                    // let selected = self.list_state_channels.selected().unwrap_or_default();
                    // self.podcasts_collection.remove(selected);
                    // if selected >= self.podcasts_collection.len() {
                    //     self.list_state_channels.select(Some(selected - 1));
                    // }
                    // config::save(self.podcasts_collection.clone())?;
                },
                KeyCode::Char('h') => {
                    self.help_visible = !self.help_visible;
                },
                _ => {}
            }
            Ok(false)
        }
    }

    fn write_listening_state(&self, time: f32) {
        match self.active_item.as_ref() {
            Some(active_item) => {
                let mut ci = active_item.clone();
                ci.listening_state = Some(ListeningState {
                        time,
                        finished: false,
                    });
                let _ = self.tx.send(AsyncAction::WriteListeningState(ci));

                if let Some(selected) = self.list_state_channels.selected() {
                    let selected_channel = self.podcasts_collection[selected].clone();
                    self.tx.send(AsyncAction::ChannelAdded(selected_channel.id))
                        .map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string())).unwrap();
                }
            },
            None => (),
        }
    }

    fn handle_open_dialog_events(&mut self, key: KeyEvent) -> std::io::Result<bool> {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => self.show_open_dialog = false,
            (KeyCode::Enter, _) => {
                // self.podcasts_collection.push(self.textbox_state.text.clone());
                // config::save(self.podcasts_collection.clone())?;
                self.show_open_dialog = false;
            },
            (key_code, key_modifiers) => {
                self.open_dialog_state.handle_events(key_code, key_modifiers);
            }
        }

        Ok(false)
    }

    fn list_state_len(&self) -> usize {
        match self.active_list_state {
            0 => self.podcasts_collection.len(),
            _ => self.items_collection.len(),
        }
    }

    pub async fn on_quit(&self) {
        let time = self.player_engine.read().unwrap().current_position();
        self.write_listening_state(time as f32);
    }

}


// #[test]
// fn test_rss() {
//     let url = "https://podcast.daskoimladja.com/feed.xml";
//     let content = ureq::get(url).call().unwrap().into_string().unwrap();
//     let mut channel = Channel::from_str(&content).unwrap();
//     channel.set_items(vec![]);
//     let ext = channel.extensions().get("atom").and_then(|a| a.get("link"));
//
//     println!("channel: {:#?}", channel);
//     println!("-----------");
//     println!("{:#?}", ext);
// }
