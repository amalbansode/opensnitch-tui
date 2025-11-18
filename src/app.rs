use crate::event::{AppEvent, Event, EventHandler};
use crate::opensnitch_proto::pb::{Alert, Notification, Statistics};
use crate::server::OpenSnitchUIServer;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    widgets::ListState,
};

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tonic::Status;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Rx Pings.
    pub rx_pings: u64,
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
    /// Channel sender to generate notifications for a daemon towards.
    /// The sender handle gets replaced to the latest client connection.
    /// Race protection enabled by the mutex.
    pub notification_sender: Arc<Mutex<mpsc::Sender<Result<Notification, Status>>>>,
}

impl Default for App {
    fn default() -> Self {
        let events_handler = EventHandler::new();
        let server = OpenSnitchUIServer::new(events_handler.sender.clone());
        // Hold a dummy sender channel until a client actually connects to server and swaps in a usable
        // sender handle.
        let (dummy_sender, _) = mpsc::channel(1);
        Self {
            running: true,
            rx_pings: 0,
            events: events_handler,
            server: server,
            current_stats: Statistics::default(),
            current_alerts: Vec::new(),
            alert_list_state: ListState::default(),
            notification_sender: Arc::new(Mutex::new(dummy_sender)),
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
        self.server.spawn_and_run(&self.notification_sender);
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
                    AppEvent::NotificationReplyTypeError(_) => {} // abtodo
                    AppEvent::TestNotify => self.test_notify().await,
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
            KeyCode::Char('t' | 'T') => self.events.send(AppEvent::TestNotify),
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
        self.rx_pings = self.rx_pings.saturating_add(1);
        self.current_stats = stats;
    }

    pub fn reset_counter(&mut self) {
        self.rx_pings = 0;
    }

    pub async fn test_notify(&mut self) {
        let sender = self.notification_sender.lock().await;
        let _ = sender
            .send(Ok(Notification {
                id: 123,
                client_name: String::new(),
                server_name: String::new(),
                r#type: 14, // abtodo: Task stop with invalid data, so expect just an error log from daemon?
                data: String::from("HELLO AMAL CATCH ME ON THE TCPDUMP"),
                rules: Vec::default(),
                sys_firewall: None,
            }))
            .await;
    }
}
