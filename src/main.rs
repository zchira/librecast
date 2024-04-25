mod player_engine;
mod textbox;
mod radio_model;
mod podcasts_model;
mod config;
mod entity;
mod event_handler;

use std::io::stdout;
use color_eyre::eyre;
use crossterm::{terminal::{EnterAlternateScreen, enable_raw_mode, disable_raw_mode, LeaveAlternateScreen}, execute, event::{DisableMouseCapture, self, KeyEventKind, KeyEvent, KeyCode}, ExecutableCommand};
use event_handler::Event;
use podcasts_model::PodcastsModel;
use radio_model::RadioModel;
use ratatui::{Terminal, prelude::{CrosstermBackend, Backend, Layout, Direction}, Frame, widgets::{Block, Borders, ListState, Tabs}};
use ratatui::layout::Constraint;
use rss::Channel;
use sea_orm::{Database, DatabaseConnection};
use tokio::sync::mpsc;

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
            KeyCode::Char('q') => return Ok(true),
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
    Channel(Channel)
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<AsyncAction>();

    let mut list_streams_state = ListState::default();
    list_streams_state.select(Some(0));

    let home = std::env::var("HOME").unwrap();
    let connection = std::env::var("DATABASE_URL").unwrap_or(format!("sqlite://{}/.librecast.db?mode=rwc", home));
    let db: DatabaseConnection = Database::connect(connection).await.unwrap();

    // run tui
    let mut app = App {
        // streams_collection: vec!["https://stream.daskoimladja.com:9000/stream".to_string(), "https://live.radio.fake".to_string(), "test".to_string()],
        active_tab: 0,
        radio_model: Default::default(),
        podcasts_model: PodcastsModel::new(db, action_tx)
    };
    app.radio_model.streams_collection = config::load()?;

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    let _res = run_app(&mut terminal, &mut app).await?;

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

            if let Ok(true) = res {
                return Ok(());
            }

        }

        // terminal.draw(|f| app.ui(f))?; //  ui(f, app))?;
        //
        // if event::poll(std::time::Duration::from_millis(16))? {
        //     if let event::Event::Key(key) = event::read()? {
        //         if key.kind == KeyEventKind::Press {
        //             if let Ok(true) = app.handle_events(key) {
        //                 // exit
        //                 return Ok(true);
        //             }
        //         }
        //     }
        // }
    }
}
