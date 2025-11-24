use crate::alert;
use crate::event::{AppEvent, ConnectionEvent, Event, EventHandler};
use crate::opensnitch_proto::pb;
use crate::server::OpenSnitchUIServer;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

use crate::constants;
use crate::operator_util;

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tonic::Status;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Event handler.
    pub events: EventHandler,
    /// Server
    pub server: OpenSnitchUIServer,
    /// Rx Pings.
    pub rx_pings: u64,
    /// Latest stats to present to UI.
    pub current_stats: pb::Statistics,
    /// Vector of alerts
    pub current_alerts: VecDeque<alert::Alert>,
    /// Alert list head in UI.
    pub alert_list_render_offset: usize,
    /// Channel sender to generate notifications for a daemon towards.
    /// The sender handle gets replaced to the latest client connection.
    /// Race protection enabled by the mutex.
    pub notification_sender: Arc<Mutex<mpsc::Sender<Result<pb::Notification, Status>>>>,
    /// Info on the current connection awaiting a rule determination.
    pub current_connection: Option<ConnectionEvent>,
    /// Rule sender.
    pub rule_sender: mpsc::Sender<pb::Rule>,
    /// gRPC server IP and port to bind to.
    bind_address: SocketAddr,
    /// Default action to be sent to connected daemons.
    default_action: constants::DefaultAction,
    /// Temporary rule lifetime.
    pub temp_rule_lifetime: constants::Duration,
    /// The duration up to which app waits for user to make a disposition
    /// (allow/deny) on a trapped connection.
    connection_disposition_timeout: std::time::Duration,
}

impl App {
    /// Constructs a new instance of [`App`].
    /// # Errors
    /// Returns an error for invalid input arg.
    #[allow(clippy::missing_panics_doc)]
    pub fn new(
        bind_string: &String,
        default_action_in: &String,
        temp_rule_lifetime: &String,
        connection_disposition_timeout_in: &u64,
    ) -> Result<Self, String> {
        if bind_string.starts_with("unix") {
            return Err(String::from("Unix domain sockets not supported"));
        }
        let maybe_bind_addr = bind_string.parse::<SocketAddr>();
        if maybe_bind_addr.is_err() {
            return Err(format!(
                "Error parsing bind address '{}' : {}",
                bind_string,
                maybe_bind_addr.unwrap_err()
            ));
        }

        let maybe_default_action = constants::DefaultAction::new(default_action_in);
        if maybe_default_action.is_err() {
            return Err(format!("Invalid default action: {default_action_in}"));
        }

        let maybe_temp_rule_lifetime = constants::Duration::new(temp_rule_lifetime);
        if maybe_temp_rule_lifetime.is_err() {
            return Err(format!(
                "Invalid temporary rule lifetime: {temp_rule_lifetime}"
            ));
        }

        // The client RPC context timeout in opensnitch/daemon/ui/client.go is set to 120s
        // Subtract a few seconds just to be nice.
        if *connection_disposition_timeout_in > 115 {
            return Err(format!(
                "Connection disposition timeout {connection_disposition_timeout_in} cannot be over 115"
            ));
        }
        let connection_disposition_timeout =
            std::time::Duration::from_secs(*connection_disposition_timeout_in);

        let events_handler = EventHandler::new();
        let server = OpenSnitchUIServer::default();

        // Hold a dummy sender channel until a client actually connects to server and swaps in a usable
        // sender handle.
        let (dummy_notification_sender, _) = mpsc::channel(1);
        let (dummy_rule_sender, _) = mpsc::channel(1);

        Ok(Self {
            running: true,
            rx_pings: 0,
            events: events_handler,
            server,
            current_stats: pb::Statistics::default(),
            current_alerts: VecDeque::new(),
            alert_list_render_offset: 0,
            notification_sender: Arc::new(Mutex::new(dummy_notification_sender)),
            current_connection: None,
            rule_sender: dummy_rule_sender,
            bind_address: maybe_bind_addr.unwrap(),
            default_action: maybe_default_action.unwrap(),
            temp_rule_lifetime: maybe_temp_rule_lifetime.unwrap(),
            connection_disposition_timeout,
        })
    }

    /// Run the application's main loop.
    /// # Errors
    /// Doesn't nominally return any errors at runtime.
    /// # Panics
    /// Largely upon runtime invariant violation, could be fixed in future versions.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        // Rule receiver gets borrowed by the server
        let (rule_sender, rule_receiver) = mpsc::channel(1);
        self.rule_sender = rule_sender;
        self.server.spawn_and_run(
            self.bind_address,
            self.events.sender.clone(),
            &self.notification_sender,
            rule_receiver,
            self.default_action,
            self.connection_disposition_timeout,
        );
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        self.handle_key_events(key_event)?;
                    }
                    _ => {}
                },
                Event::App(app_event) => match *app_event {
                    AppEvent::Update(stats) => self.update_stats(stats),
                    AppEvent::Alert(alert) => self.current_alerts.push_back(alert.clone()),
                    AppEvent::AskRule(evt) => self.update_connection(evt),
                    AppEvent::TestNotify => self.test_notify().await,
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    /// # Errors
    /// Not really...
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit);
            }
            KeyCode::Char('t' | 'T') => self.events.send(AppEvent::TestNotify),
            KeyCode::Char('a' | 'A') => {
                self.make_and_send_rule(constants::Action::Allow, self.temp_rule_lifetime);
            }
            KeyCode::Char('d' | 'D') => {
                self.make_and_send_rule(constants::Action::Deny, self.temp_rule_lifetime);
            }
            KeyCode::Char('j' | 'J') => {
                self.make_and_send_rule(constants::Action::Allow, constants::Duration::Always);
            }
            KeyCode::Char('l' | 'L') => {
                self.make_and_send_rule(constants::Action::Deny, constants::Duration::Always);
            }
            KeyCode::Up => {
                self.alert_list_render_offset = self.alert_list_render_offset.saturating_sub(1);
            }
            KeyCode::Down => {
                if !self.current_alerts.is_empty() {
                    self.alert_list_render_offset = std::cmp::min(
                        self.alert_list_render_offset.saturating_add(1),
                        self.current_alerts.len() - 1,
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        let now = std::time::SystemTime::now();
        if let Some(conn) = &self.current_connection
            && now >= conn.expiry_ts
        {
            // The daemon's gRPC call should time out and take some default action
            // in the absence of a Rule created by us.
            self.clear_connection();
        }

        // Routinely expire alerts.
        match self.current_alerts.front() {
            None => {}
            Some(alert) => {
                if let Ok(age) = now.duration_since(alert.timestamp) {
                    // Max alert duration is 60s, could be adjustable if needed
                    if age.as_secs() >= 60 {
                        // Pop this off but also correct the render offset in case it's set to back of list.
                        if self.alert_list_render_offset == (self.current_alerts.len() - 1) {
                            self.alert_list_render_offset =
                                self.alert_list_render_offset.saturating_sub(1);
                        }
                        self.current_alerts.pop_front();
                    }
                }
            }
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Update peer stats from incoming Ping payload.
    pub fn update_stats(&mut self, stats: pb::Statistics) {
        self.rx_pings = self.rx_pings.saturating_add(1);
        self.current_stats = stats;
    }

    /// Server to daemon notifications under development.
    pub async fn test_notify(&mut self) {
        let sender = self.notification_sender.lock().await;
        let _ = sender
            .send(Ok(pb::Notification {
                id: 123,
                client_name: String::default(),
                server_name: String::default(),
                r#type: 14, // This is a task stop notification.
                data: String::from("Test notification triggers an error"),
                rules: Vec::default(),
                sys_firewall: None,
            }))
            .await;
    }

    /// Update connection holder with latest inbound event.
    pub fn update_connection(&mut self, evt: ConnectionEvent) {
        self.current_connection = Some(evt);
    }

    /// Clear connection holder.
    pub fn clear_connection(&mut self) {
        self.current_connection = None;
    }

    /// Generate a rule for the current connection being handled by this server.
    /// Matches on user ID && process path && IP dst && l4 port && l4 protocol.
    /// TODO: Consider including process hash for extra strictness.
    /// Returns `none` if there is no current connection.
    /// * `is_allow`: Whether the rule for this connection should allow or deny the flow.
    fn make_rule(
        &self,
        action: constants::Action,
        duration: constants::Duration,
    ) -> Option<pb::Rule> {
        // Noop if there's no connection trapped.
        self.current_connection.as_ref()?;

        let conn = &self.current_connection.as_ref().unwrap().connection;

        // Build up an array of "safe"ish default operators to match this process's
        // specific connection, though this can obviously be better validated/configured
        // in the future.
        // This could have also been implemented with enum+trait magic, but using a simple
        // Operator factory lets us pass this vector into the larger Rule we are creating.
        let operators = vec![
            operator_util::match_user_id(conn.user_id),
            operator_util::match_proc_path(&conn.process_path),
            operator_util::match_dst_ip(&conn.dst_ip),
            operator_util::match_dst_port(conn.dst_port),
            operator_util::match_protocol(&conn.protocol),
        ];

        let action_str = action.get_str();
        let duration = String::from(duration.get_str());
        let pretty_proc_path = conn.process_path.clone().replace('/', "-");
        let maybe_operator_json = serde_json::to_string(&operators);
        // Shouldn't really happen due to serde_impl.rs, ideally something caught at build time.
        assert!(
            maybe_operator_json.is_ok(),
            "Operator list JSON serialization failed: {}",
            maybe_operator_json.unwrap_err()
        );

        Some(pb::Rule {
            created: 0,
            name: format!("{action_str}-{duration}-simple-via-tui-{pretty_proc_path}"),
            description: String::default(),
            enabled: true,
            precedence: false,
            nolog: false,
            action: String::from(action_str),
            duration,
            operator: Some(pb::Operator {
                r#type: String::from(constants::RuleType::List.get_str()),
                operand: String::from(constants::Operand::List.get_str()),
                data: maybe_operator_json.unwrap(),
                sensitive: false,
                list: operators,
            }),
        })
    }

    fn send_rule(&self, rule: pb::Rule) {
        let send_res = self.rule_sender.try_send(rule);
        if let Err(err) = send_res {
            // Shouldn't really happen so bail here.
            panic!("Unable to send rule: {err}");
        }
    }

    fn make_and_send_rule(&mut self, action: constants::Action, duration: constants::Duration) {
        if let Some(rule) = self.make_rule(action, duration) {
            self.send_rule(rule);
            self.clear_connection();
        }
    }
}
