use crate::event::{AppEvent, ConnectionEvent, Event, EventHandler};
use crate::opensnitch_proto::pb::{Alert, Notification, Operator, Rule, Statistics};
use crate::server::OpenSnitchUIServer;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    widgets::ListState,
};

use crate::constants;

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
    // Info on the current connection awaiting a rule determination.
    pub current_connection: Option<ConnectionEvent>,
    // Rule sender.
    pub rule_sender: mpsc::Sender<Rule>,
}

impl Default for App {
    fn default() -> Self {
        let events_handler = EventHandler::new();
        let server = OpenSnitchUIServer::new(events_handler.sender.clone());
        // Hold a dummy sender channel until a client actually connects to server and swaps in a usable
        // sender handle.
        let (dummy_notification_sender, _) = mpsc::channel(1);
        let (dummy_rule_sender, _) = mpsc::channel(1);

        Self {
            running: true,
            rx_pings: 0,
            events: events_handler,
            server: server,
            current_stats: Statistics::default(),
            current_alerts: Vec::new(),
            alert_list_state: ListState::default(),
            notification_sender: Arc::new(Mutex::new(dummy_notification_sender)),
            current_connection: None,
            rule_sender: dummy_rule_sender,
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
        // Rule receiver - borrowed by the server
        let (rule_sender, rule_receiver) = mpsc::channel(1);
        self.rule_sender = rule_sender;
        self.server
            .spawn_and_run(&self.notification_sender, rule_receiver);
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
                    AppEvent::Update(stats) => self.update_stats(stats),
                    AppEvent::Alert(alert) => self.current_alerts.push(alert),
                    AppEvent::NotificationReplyTypeError(_) => {} // abtodo
                    AppEvent::AskRule(evt) => self.update_connection(evt),
                    AppEvent::TestNotify => self.test_notify().await,
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Char('t' | 'T') => self.events.send(AppEvent::TestNotify),
            KeyCode::Char('a' | 'A') => self.send_temp_allow_rule(),
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        let now = std::time::SystemTime::now();
        if self.current_connection.is_some()
            && now >= self.current_connection.as_ref().unwrap().expiry_ts
        {
            self.clear_connection();
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn update_stats(&mut self, stats: Statistics) {
        self.rx_pings = self.rx_pings.saturating_add(1);
        self.current_stats = stats;
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

    pub fn update_connection(&mut self, evt: ConnectionEvent) {
        self.current_connection = Some(evt);
    }

    pub fn clear_connection(&mut self) {
        self.current_connection = None;
    }

    pub fn send_temp_allow_rule(&mut self) {
        if self.current_connection.is_none() {
            return;
        }

        let conn = &self.current_connection.as_ref().unwrap().connection;

        // Build up an array of "safe"ish default operators to match this process's
        // specific connection, though this can obviously be better validated/configured
        // in the future.
        // abtodo prettier generators
        let operators = vec![
            Operator {
                r#type: String::from(constants::rule_type::RULE_TYPE_SIMPLE),
                operand: String::from(constants::operand::OPERAND_USER_ID),
                data: conn.user_id.to_string(),
                sensitive: false,
                list: Vec::default(),
            },
            Operator {
                r#type: String::from(constants::rule_type::RULE_TYPE_SIMPLE),
                operand: String::from(constants::operand::OPERAND_PROCESS_PATH),
                data: conn.process_path.clone(),
                sensitive: false,
                list: Vec::default(),
            },
            Operator {
                r#type: String::from(constants::rule_type::RULE_TYPE_SIMPLE),
                operand: String::from(constants::operand::OPERAND_DEST_IP),
                data: conn.dst_ip.clone(),
                sensitive: false,
                list: Vec::default(),
            },
            Operator {
                r#type: String::from(constants::rule_type::RULE_TYPE_SIMPLE),
                operand: String::from(constants::operand::OPERAND_DEST_PORT),
                data: conn.dst_port.to_string(),
                sensitive: false,
                list: Vec::default(),
            },
            Operator {
                r#type: String::from(constants::rule_type::RULE_TYPE_SIMPLE),
                operand: String::from(constants::operand::OPERAND_PROTOCOL),
                data: conn.protocol.clone(),
                sensitive: false,
                list: Vec::default(),
            },
        ];

        let pretty_proc_path = conn.process_path.clone().replace("/", "-");
        let maybe_operator_json = serde_json::to_string(&operators);
        if maybe_operator_json.is_err() {
            panic!(
                "Operator list JSON serialization failed: {}",
                maybe_operator_json.unwrap_err()
            );
        }

        let rule = Rule {
            created: 0,
            name: format!("allow-12h-simple-{}", pretty_proc_path),
            description: String::default(),
            enabled: true,
            precedence: false,
            nolog: false,
            action: String::from(constants::action::ACTION_ALLOW),
            duration: String::from(constants::duration::DURATION_12h),
            operator: Some(Operator {
                r#type: String::from(constants::rule_type::RULE_TYPE_LIST),
                operand: String::from(constants::operand::OPERAND_LIST),
                data: maybe_operator_json.unwrap(),
                sensitive: false,
                list: operators,
            }),
        };

        let send_res = self.rule_sender.try_send(rule);
        match send_res {
            Err(err) => {
                panic!("Unable to send rule: {}", err);
            }
            _ => {}
        }
        self.clear_connection();
    }
}
