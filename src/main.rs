mod player_engine;
mod radio_model;
mod podcasts_model;
mod config;
mod entity;
mod event_handler;
mod widgets;
mod data_layer;
mod ui_models;

use entity::channel;
use migration::{Migrator, MigratorTrait};
use ui_models::ChannelItem;
use std::io::stdout;
use color_eyre::eyre;
use crossterm::{terminal::{EnterAlternateScreen, enable_raw_mode, disable_raw_mode, LeaveAlternateScreen}, execute, event::{DisableMouseCapture, KeyCode}, ExecutableCommand};
use event_handler::Event;
use podcasts_model::PodcastsModel;
use radio_model::RadioModel;
use ratatui::{Terminal, prelude::{CrosstermBackend, Backend, Layout, Direction}, Frame, widgets::{Block, Borders, ListState, Tabs}};
use ratatui::layout::Constraint;
use rss::Channel;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, DbErr, EntityTrait};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use data_layer::{data_provider::DataProvider, listening_state_data_layer::ListeningStateDataLayer};


pub struct App {
    radio_model: RadioModel,
    podcasts_model: PodcastsModel,
    active_tab: usize,
}

impl App {
    pub fn ui(&mut self, f: &mut Frame) {
        let size = f.size();

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)])
            .split(size);

        let tabs = Tabs::new(vec!["[1] Radio", "[2] Podcasts"])
            .block(Block::default().title(format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))).borders(Borders::ALL))
            .select(self.active_tab);

        f.render_widget(tabs, vertical_chunks[0]);

        match self.active_tab {
            0 => self.radio_model.ui(vertical_chunks[1], f),
            1 => self.podcasts_model.ui(vertical_chunks[1], f),
            _ => {}
        }
    }

    async fn handle_events(&mut self, event: Event) -> std::io::Result<bool> {
        if let Event::Key(key) = event {
        match key.code {
            KeyCode::Char('q') => {
                self.podcasts_model.on_quit().await;
                return Ok(true);
            },
            KeyCode::Char('1') => {
                self.active_tab = 0;
            },
            KeyCode::Char('2') => {
                self.active_tab = 1;
            },
            KeyCode::Tab => {
                self.active_tab = (self.active_tab + 1) % 2;
            }
            _ => {
                match self.active_tab {
                    0 => { self.radio_model.handle_events(key).await?; },
                    1 => { self.podcasts_model.handle_events(key).await?; },
                    _ => {}
                }
            }
        }
        }
        Ok(false)
    }
}

pub enum AsyncAction {
    Channel(Channel), // remove?
    ChannelAdded(i32),
    RefreshChannelsList,
    WriteListeningState(ChannelItem)
}

async fn init_data(db: &DatabaseConnection) -> Result<(), DbErr>{
    let res = channel::Entity::find().all(db).await?;
    if res.len() > 0 {
        return Ok(());
    }

    let c1 = channel::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        title: sea_orm::ActiveValue::Set(Some("Dasko i Mladja".to_string())),
        link: sea_orm::ActiveValue::Set(Some("https://podcast.daskoimladja.com/feed.xml".to_string())),
        description: sea_orm::ActiveValue::Set(Some("fake...".to_string())),
    };

    c1.insert(db).await?;

    let c2 = channel::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        title: sea_orm::ActiveValue::Set(Some("Agelast".to_string())),
        link: sea_orm::ActiveValue::Set(Some("https://feeds.transistor.fm/agelast-podcast".to_string())),
        description: sea_orm::ActiveValue::Set(Some("fake...".to_string())),
    };

    c2.insert(db).await?;
    Ok(())
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<AsyncAction>();

    let mut list_streams_state = ListState::default();
    list_streams_state.select(Some(0));

    let home = std::env::var("HOME").unwrap();
    let connection = std::env::var("DATABASE_URL").unwrap_or(format!("sqlite://{}/.librecast.db?mode=rwc", home));
    let db: DatabaseConnection = Database::connect(connection).await?;

    Migrator::up(&db, None).await?;
    init_data(&db).await?;
    // run tui
    let mut app = App {
        // streams_collection: vec!["https://stream.daskoimladja.com:9000/stream".to_string(), "https://live.radio.fake".to_string(), "test".to_string()],
        active_tab: 0,
        radio_model: Default::default(),
        podcasts_model: PodcastsModel::new(db.clone(), action_tx)
    };
    app.radio_model.streams_collection = config::load()?;
    app.podcasts_model.podcasts_collection = app.podcasts_model.get_channels_from_db().await?;

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    let res = run_app(&mut terminal, &mut app, &mut action_rx, &db).await;

    match res {
        Ok(_) => {},
        Err(e) => println!("{:#?}", e)
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
        )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    action_rx: &mut UnboundedReceiver<AsyncAction>,
    db: &DatabaseConnection
) -> eyre::Result<()> {
    loop {
        let mut events = event_handler::EventHandler::new();

        loop {
            let event = events.next().await?;

            if let Event::Render = event.clone() {
                // application render
                terminal.draw(|f| app.ui(f))?;
            }

            let res = app.handle_events(event).await;

            if let Ok(a) = action_rx.try_recv() {
                match a {
                    AsyncAction::Channel(_channel) => {},
                    AsyncAction::ChannelAdded(id) => {
                        let mut items = DataProvider::get_items_from_db(id, &db.clone()).await?;
                        let items_len = items.len();
                        app.podcasts_model.items_collection.clear();
                        app.podcasts_model.items_collection.append(&mut items);

                        let select_item = match app.podcasts_model.active_item.as_ref() {
                            Some(ai) => {
                                ai.channel_id != id
                            },
                            None => true,
                        };

                        if items_len > 0 && select_item{
                            app.podcasts_model.list_state_items.select(Some(0));
                        }
                        app.podcasts_model.waiting_message = None;
                    },
                    AsyncAction::RefreshChannelsList =>{
                        app.podcasts_model.podcasts_collection = app.podcasts_model.get_channels_from_db().await?;
                    },
                    AsyncAction::WriteListeningState(channel_item) => {
                        match channel_item.listening_state.as_ref() {
                            Some(ls) => {
                                let _ = ListeningStateDataLayer::update_current_time_for_item(db.clone(),
                                    channel_item.enclosure, channel_item.channel_id, ls.time).await;
                            },
                            None => {},
                        }
                        
                    },
                }
            }

            if let Ok(true) = res {
                return Ok(());
            }

        }
    }
}
