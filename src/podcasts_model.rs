use std::{borrow::BorrowMut, io::ErrorKind, str::FromStr, sync::{Arc, RwLock}};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, List, ListState, Paragraph}, Frame};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use tokio::sync::mpsc::UnboundedSender;
use tui_textbox::{Textbox, TextboxState};

use std::error::Error;
use rss::Channel;
use crate::{entity::{self, channel::Entity as ChannelEntity}, widgets::{simple_list::SimpleList, timeline::Timeline}, AsyncAction};

use crate::{config, player_engine::PlayerEngine};

pub struct PodcastsModel {
    db: DatabaseConnection,
    help_visible: bool,
    pub active_channel: Option<String>,
    pub active_item: Option<rss::Item>,
    pub error: Option<String>,
    pub items_collection: Vec<rss::Item>,
    pub list_state_channels: ListState,
    pub list_state_items: ListState,
    pub active_list_state: usize,
    pub player_engine: Arc<RwLock<PlayerEngine>>,
    pub podcasts_collection: Vec<String>,
    pub show_open_dialog: bool,
    pub textbox_state: TextboxState,
    tx: UnboundedSender<crate::AsyncAction>
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
            podcasts_collection: vec!["dasko i mladja".to_string(), "agelast".to_string()], // Default::default(),
            show_open_dialog: Default::default(),
            textbox_state: Default::default(),
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


        // list channels
        let fg_color  = |i: usize| if self.active_list_state == i { ratatui::style::Color::Blue } else { ratatui::style::Color::DarkGray };
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
        .fg(fg_color(0))
        .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ")
            .repeat_highlight_symbol(true);


        f.render_stateful_widget(list, horizontal_chunks[0], &mut self.list_state_channels);

        let selected = self.list_state_items.selected().unwrap_or(0);
        let offset = self.list_state_items.offset();

        let simple_list = SimpleList {
            fg_color: fg_color(1),
            items: self.items_collection.clone().into_iter().enumerate().map(|(index, c)| {

                if index < offset || index > offset + 80 {
                    String::new()
                } else if index == selected {
                    format!("{} {}/{} 🎵", c.title.clone().unwrap_or("".to_string()), selected, offset)
                } else {
                    format!("{} [{}]", c.title.clone().unwrap_or("".to_string()), index)
                }
            }).collect(),
        };
        f.render_stateful_widget(simple_list, horizontal_chunks[1], &mut self.list_state_items);

        //     .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        //     .highlight_symbol(">> ")
        //     .repeat_highlight_symbol(true);

        {
            let p = self.player_engine.read().unwrap();
            let timeline = Timeline {
                progress: p.current_position(),
                progress_display: p.current_position_display(),
                total: p.duration(),
                total_display: p.duration_display(),
                playing: !(p.is_paused()),
                error: None,
            };
            f.render_widget(timeline, vertical_chunks[1]);
        }

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
                    // self.items_collection.push("--reading--".to_string());
                    let tx = self.tx.clone();
                    let db = self.db.clone();

                    tokio::spawn(async move {
                        // tokio::time::sleep(Duration::from_secs(5)).await; // simulate network request
                        let channel = PodcastsModel::get_channel_from_url().await.map_err(|_| std::io::Error::new(ErrorKind::Other, "")).unwrap();

                        use entity::channel::{ Entity, ActiveModel };
                        let am: ActiveModel = ActiveModel {
                            title: ActiveValue::set(Some(channel.title().to_string())),
                            link: ActiveValue::set(Some(channel.link().to_string())),
                            description: ActiveValue::set(Some(channel.description().to_string())),
                            id: ActiveValue::NotSet
                        };

                        let exist = Entity::find().filter(entity::channel::Column::Link.eq(channel.link())).one(&db).await.unwrap();

                        let channel_id = if let Some(exist) = exist {
                            exist.id
                        } else {
                            // let res = Entity::update_many().filter(entity::channel::Column::Link.eq(channel.link())).exec(&db).await.unwrap();
                            let res = Entity::insert(am).exec(&db).await.unwrap();
                            res.last_insert_id
                        };

                        let items = channel.items().iter().map(|i| {
                            entity::channel_item::ActiveModel {
                                id: ActiveValue::NotSet,
                                channel_id: ActiveValue::set(channel_id),
                                title: ActiveValue::set(i.title().map(|t| t.to_string())),
                                link: ActiveValue::set(i.link().map(|l| l.to_string())),
                                source: ActiveValue::set(i.source().map(|s| s.url.to_string())),
                                enclosure: ActiveValue::set(i.enclosure().map(|e| e.url.to_string())),
                                description: ActiveValue::set(i.description().map(|d| d.to_string())),
                                guid: ActiveValue::set(i.guid().map(|g| g.value.clone())),
                                pub_date: ActiveValue::set(i.pub_date().map(|d| d.to_string())),
                            }
                        });

                        let _ = entity::channel_item::Entity::delete_many().filter(entity::channel_item::Column::ChannelId.eq(channel_id)).exec(&db).await;

                        let _ = entity::channel_item::Entity::insert_many(items).exec(&db).await;
                        tx.send(AsyncAction::ChannelAdded(channel_id)).map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string())).unwrap();
                    });
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
                        }
                    }
                }
                KeyCode::Left | KeyCode::Right => { 
                    self.active_list_state = (self.active_list_state + 1) % 2;
                }
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Enter => {
                    if self.active_list_state == 0 {
                        // load items then move items list
                        self.active_list_state = self.active_list_state + 1;
                    } else {
                        match self.active_item {
                            Some(_) => {
                                self.player_engine = Arc::new(RwLock::new(PlayerEngine::new()));
                                self.active_item = None;
                            },
                            None => {
                                let selected_episode = &self.items_collection[self.list_state_items.selected().unwrap_or_default()];
                                self.active_item = Some(selected_episode.clone());
                                let mut p = self.player_engine.write().unwrap();
                                match p.open(&selected_episode.enclosure().unwrap().url) {
                                    Ok(_) => {
                                        self.error = None;
                                    },
                                    Err(e) => self.error = Some(e.to_string()),
                                }
                            },
                        }
                    }
                },
                KeyCode::Char(' ') => {
                    if self.active_channel.is_some() {
                        let mut p = self.player_engine.write().unwrap();
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

    fn list_state_len(&self) -> usize {
        match self.active_list_state {
            0 => self.podcasts_collection.len(),
            _ => self.items_collection.len(),
        }
    }

}
