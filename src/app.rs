use crate::event::{AppEvent, Event, EventHandler};
use crate::opensnitch_proto::pb::{Alert, Statistics};
use crate::server::OpenSnitchUIServer;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    widgets::ListState,
};

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Counter.
    pub counter: u8,
    /// Event handler.
    pub events: EventHandler,
    /// Server
    pub server: OpenSnitchUIServer,
    /// Latest stats to present to UI.
    pub current_stats: Statistics,
    /// Vector of alerts
    pub current_alerts: Vec<Alert>,
    /// Alert list rendering state
    pub alert_list_state: ListState,
}

impl Default for App {
    fn default() -> Self {
        let events_handler = EventHandler::new();
        let server = OpenSnitchUIServer::new(events_handler.sender.clone());
        Self {
            running: true,
            counter: 0,
            events: events_handler,
            server: server,
            current_stats: Statistics::default(),
            current_alerts: Vec::new(),
            alert_list_state: ListState::default(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.server.spawn_and_run();
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Update(stats) => self.update(stats),
                    AppEvent::Alert(alert) => self.current_alerts.push(alert),
                    AppEvent::Reset => self.reset_counter(),
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Char('r' | 'R') => self.events.send(AppEvent::Reset),
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn update(&mut self, stats: Statistics) {
        self.counter = self.counter.saturating_add(1);
        self.current_stats = stats;
    }

    pub fn reset_counter(&mut self) {
        self.counter = 0;
    }
}
