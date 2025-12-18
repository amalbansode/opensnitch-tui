use crate::alert::{self, Alert};
use crate::event::{AppEvent, ConnectionEvent, Event, EventHandler, PingEvent};
use crate::opensnitch_proto::pb;
use crate::server::OpenSnitchUIServer;
use crossterm::event::MouseEvent;
use ratatui::layout::{Position, Rect};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

use crate::constants;
use crate::operator_util::{self, PresetCombination};

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tonic::Status;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    /// Event handler.
    events: EventHandler,
    /// Server
    server: OpenSnitchUIServer,
    /// Channel sender to generate notifications for a daemon towards.
    /// The sender handle gets replaced to the latest client connection.
    /// Race protection enabled by the mutex.
    notification_sender: Arc<Mutex<mpsc::Sender<Result<pb::Notification, Status>>>>,
    /// Rule sender.
    rule_sender: mpsc::Sender<pb::Rule>,
    /// gRPC server IP and port to bind to.
    bind_address: SocketAddr,
    /// The duration up to which app waits for user to make a disposition
    /// (allow/deny) on a trapped connection.
    connection_disposition_timeout: std::time::Duration,
    /// Combination of preset operators to use when creating rules.
    preset_combo: PresetCombination,
    /// Shared state between TUI and app driver.
    tui_state: TuiState,
    /// Shared **mutable** state between app driver and TUI rendering.
    tui_mut_state: TuiMutState,
}

/// Shared state between TUI and app driver.
#[derive(Debug)]
pub struct TuiState {
    /// Rx Pings from daemon - internal only.
    rx_pings: u64,
    /// Peer (`OpenSnitch` daemon) address.
    pub peer: Option<std::net::SocketAddr>,
    /// Latest stats to present to UI.
    pub current_stats: Option<pb::Statistics>,
    /// Vector of alerts
    pub current_alerts: VecDeque<alert::Alert>,
    /// Alert list head in UI.
    pub alert_list_render_offset: usize,
    /// Info on the current connection awaiting a rule determination.
    pub current_connection: Option<ConnectionEvent>,
    /// Default action to be sent to connected daemons.
    pub default_action: constants::DefaultAction,
    /// Temporary rule lifetime.
    pub temp_rule_lifetime: constants::Duration,
    /// UI footer `Controls` list.
    pub controls: Vec<Controls>,
}

/// Shared mutable state between both TUI and app driver.
#[derive(Debug, Default)]
pub struct TuiMutState {
    /// Connection panel area as determined by ratatui lib.
    pub connection_area: Rect,
    /// Controls footer area as determined by ratatui lib.
    pub controls_area: Rect,
}

/// Enum of controls and any associated data at the footer of TUI.
#[derive(Clone, Debug, Copy)]
pub enum Controls {
    Quit,
    AllowTemp(constants::Duration),
    DenyTemp(constants::Duration),
    AllowForever,
    DenyForever,
}

impl Controls {
    /// Get keybinding string.
    #[must_use]
    pub fn get_keybind_str(&self) -> &str {
        // Add in whitespace margins here for easy math later.
        match self {
            Self::Quit => " Ctrl+C ",
            Self::AllowTemp(_) => " A ",
            Self::DenyTemp(_) => " D ",
            Self::AllowForever => " J ",
            Self::DenyForever => " L ",
        }
    }

    /// Get the "control"/description.
    #[must_use]
    pub fn get_control_str(&self) -> String {
        // Add in whitespace margins here for easy math later.
        match self {
            Self::Quit => String::from(" Quit "),
            Self::AllowTemp(duration) => format!(" Allow {} ", duration.get_str()),
            Self::DenyTemp(duration) => format!(" Deny {} ", duration.get_str()),
            Self::AllowForever => String::from(" Allow Forever "),
            Self::DenyForever => String::from(" Deny Forever "),
        }
    }

    /// Return the sum of keybind strlen and control strlen.
    #[must_use]
    pub fn get_button_width(&self) -> usize {
        self.get_keybind_str().len() + self.get_control_str().len()
    }
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
        rule_presets: &str,
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

        if rule_presets.is_empty() {
            return Err("Rule preset operators cannot be empty".to_string());
            // Edge cases with invalid elements should be handled by from_str below.
        }
        let preset_combo = operator_util::PresetCombination::from_str(rule_presets)?;

        let events_handler = EventHandler::new();
        let server = OpenSnitchUIServer::default();

        // Hold a dummy sender channel until a client actually connects to server and swaps in a usable
        // sender handle.
        let (dummy_notification_sender, _) = mpsc::channel(1);
        let (dummy_rule_sender, _) = mpsc::channel(1);

        let controls = vec![
            Controls::Quit,
            Controls::AllowTemp(maybe_temp_rule_lifetime.clone().unwrap()),
            Controls::DenyTemp(maybe_temp_rule_lifetime.clone().unwrap()),
            Controls::AllowForever,
            Controls::DenyForever,
        ];

        Ok(Self {
            running: true,
            events: events_handler,
            server,
            notification_sender: Arc::new(Mutex::new(dummy_notification_sender)),
            rule_sender: dummy_rule_sender,
            bind_address: maybe_bind_addr.unwrap(),
            connection_disposition_timeout,
            preset_combo,
            tui_state: TuiState {
                rx_pings: 0,
                peer: None,
                current_stats: None,
                current_alerts: VecDeque::new(),
                alert_list_render_offset: 0,
                current_connection: None,
                default_action: maybe_default_action.unwrap(),
                temp_rule_lifetime: maybe_temp_rule_lifetime.unwrap(),
                controls,
            },
            tui_mut_state: TuiMutState {
                connection_area: Rect::default(),
                controls_area: Rect::default(),
            },
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
            self.tui_state.default_action,
            self.connection_disposition_timeout,
        );
        // Only need a draw if:
        // * This is the first cycle (see default value below)
        // * Tick resulted in a meaningful state update
        // * Crossterm event - some key was pressed
        // * We received an event from the gRPC server
        let mut draw_needed = true;
        while self.running {
            match self.events.next().await? {
                Event::Tick => draw_needed |= self.tick(), /* Doing an OR here lets first tick through */
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        draw_needed = true;
                        self.handle_key_events(key_event)?;
                    }
                    crossterm::event::Event::Mouse(mouse_event)
                        if mouse_event.kind
                            == crossterm::event::MouseEventKind::Up(
                                crossterm::event::MouseButton::Left,
                            ) =>
                    {
                        draw_needed |= self.handle_mouse_events(mouse_event)?;
                    }
                    _ => {}
                },
                Event::App(app_event) => {
                    draw_needed = true;
                    match *app_event {
                        AppEvent::Update(stats) => self.update_stats(stats),
                        AppEvent::Alert(alert) => {
                            self.tui_state.current_alerts.push_back(alert.clone());
                        }
                        AppEvent::AskRule(evt) => self.update_connection(evt),
                        AppEvent::TestNotify => self.test_notify().await,
                        AppEvent::Quit => self.quit(),
                    }
                }
            }
            if draw_needed {
                terminal.draw(|frame| {
                    frame.render_stateful_widget(
                        &self.tui_state,
                        frame.area(),
                        &mut self.tui_mut_state,
                    );
                })?;
                draw_needed = false;
            }
        }
        Ok(())
    }

    /// Handles key events and updates the state of [`App`].
    /// # Errors
    /// Not really...
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit);
            }
            KeyCode::Char('t' | 'T') => self.events.send(AppEvent::TestNotify),
            KeyCode::Char('a' | 'A') => {
                self.make_and_send_rule(
                    constants::Action::Allow,
                    self.tui_state.temp_rule_lifetime,
                );
            }
            KeyCode::Char('d' | 'D') => {
                self.make_and_send_rule(constants::Action::Deny, self.tui_state.temp_rule_lifetime);
            }
            KeyCode::Char('j' | 'J') => {
                self.make_and_send_rule(constants::Action::Allow, constants::Duration::Always);
            }
            KeyCode::Char('l' | 'L') => {
                self.make_and_send_rule(constants::Action::Deny, constants::Duration::Always);
            }
            KeyCode::Up => {
                self.tui_state.alert_list_render_offset =
                    self.tui_state.alert_list_render_offset.saturating_sub(1);
            }
            KeyCode::Down => {
                if !self.tui_state.current_alerts.is_empty() {
                    self.tui_state.alert_list_render_offset = std::cmp::min(
                        self.tui_state.alert_list_render_offset.saturating_add(1),
                        self.tui_state.current_alerts.len() - 1,
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles mouse events and updates the state of [`App`].
    /// # Errors
    /// Not really...
    pub fn handle_mouse_events(&mut self, mouse_event: MouseEvent) -> color_eyre::Result<bool> {
        let mut clicked_control = None;
        if self
            .tui_mut_state
            .controls_area
            .contains(Position::new(mouse_event.column, mouse_event.row))
        {
            // Controls footer faux button group is left aligned in TUI, so use a simple
            // accumulator pattern to determine which control overlaps with click coords.
            let mut column_accumulator: usize = 0;
            for control in &self.tui_state.controls {
                let lb = column_accumulator;
                column_accumulator += control.get_button_width();
                let rb = column_accumulator;
                if usize::from(mouse_event.column) >= lb && usize::from(mouse_event.column) < rb {
                    clicked_control = Some(control);
                    break;
                }
            }
        }

        if let Some(control) = clicked_control {
            match control {
                Controls::Quit => self.events.send(AppEvent::Quit),
                Controls::AllowTemp(duration) => {
                    self.make_and_send_rule(constants::Action::Allow, *duration);
                }
                Controls::DenyTemp(duration) => {
                    self.make_and_send_rule(constants::Action::Deny, *duration);
                }
                Controls::AllowForever => {
                    self.make_and_send_rule(constants::Action::Allow, constants::Duration::Always);
                }
                Controls::DenyForever => {
                    self.make_and_send_rule(constants::Action::Deny, constants::Duration::Always);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Handles the tick event of the terminal.
    /// Returns whether meaningful change occured, which should trigger a re-render of terminal.
    pub fn tick(&mut self) -> bool {
        let mut did_work = false;
        let now = std::time::SystemTime::now();
        if let Some(conn) = &self.tui_state.current_connection
            && now >= conn.expiry_ts
        {
            // The daemon's gRPC call should time out and take some default action
            // in the absence of a Rule created by us.
            self.clear_connection();
            did_work = true;
        }

        // Routinely expire alerts.
        match self.tui_state.current_alerts.front() {
            None => {}
            Some(alert) => {
                if let Ok(age) = now.duration_since(alert.timestamp) {
                    // Max alert duration is 60s, could be adjustable if needed
                    if age.as_secs() >= 60 {
                        // Pop this off but also correct the render offset in case it's set to back of list.
                        if self.tui_state.alert_list_render_offset
                            == (self.tui_state.current_alerts.len() - 1)
                        {
                            self.tui_state.alert_list_render_offset =
                                self.tui_state.alert_list_render_offset.saturating_sub(1);
                        }
                        self.tui_state.current_alerts.pop_front();
                        did_work = true;
                    }
                }
            }
        }

        did_work
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Update peer stats from incoming Ping payload.
    /// TODO: Set a staleness threshold so we can clearly flag a disconnected peer.
    /// For now, this can be simulated locally via:
    /// iptables -A INPUT -p tcp --dport 50051 -j DROP
    /// iptables -D INPUT -p tcp --dport 50051 -j DROP
    pub fn update_stats(&mut self, ping_event: PingEvent) {
        self.tui_state.rx_pings = self.tui_state.rx_pings.saturating_add(1);
        self.tui_state.peer = ping_event.peer;
        self.tui_state.current_stats = Some(ping_event.stats);
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
        self.tui_state.current_connection = Some(evt);
    }

    /// Clear connection holder.
    pub fn clear_connection(&mut self) {
        self.tui_state.current_connection = None;
    }

    /// Generate a rule for the current connection being handled by this server.
    /// Returns `none` if there is no current connection.
    /// * `is_allow`: Whether the rule for this connection should allow or deny the flow.
    fn make_rule(
        &self,
        action: constants::Action,
        duration: constants::Duration,
    ) -> Option<pb::Rule> {
        // Noop if there's no connection trapped.
        let conn: &pb::Connection = &self.tui_state.current_connection.as_ref()?.connection;

        // Fill in all the inputs to generator with best effort.
        let inputs = operator_util::GeneratedOperatorsInputs {
            user_id: Some(conn.user_id),
            ppath: Some(conn.process_path.clone()),
            dst_ip: Some(conn.dst_ip.clone()),
            dst_port: Some(conn.dst_port),
            protocol: Some(conn.protocol.clone()),
            hostname: if conn.dst_host.is_empty() {
                None
            } else {
                Some(conn.dst_host.clone())
            },
        };

        // Do the work - generate populated operator list based on desired presets and given inputs.
        let operators = inputs.generate_operators(&self.preset_combo);
        if operators.is_empty() {
            return None;
        }

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
            // TODO: Leading slash gets turned into double-dash, may be annoying
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
        } else {
            // Send an alert to self that no rule was generated due to missing data.
            self.tui_state
                .current_alerts
                .push_back(Alert::create_simple(
                    std::time::SystemTime::now(),
                    "No rule created due to lack of connection data",
                ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::opensnitch_proto::pb::{Connection, Rule};
    use std::time::SystemTime;

    use super::*;

    /// Convenience Alias for String-to-String Hashmap.
    type S2SMap = std::collections::HashMap<String, String>;

    /// Simple construction test.
    #[tokio::test]
    async fn test_new() {
        let _ = App::new(
            &"127.0.0.1:65534".to_string(),
            &"deny".to_string(),
            &"12h".to_string(),
            &60,
            &"exact_dst_ip".to_string(),
        )
        .expect("new failed");
    }

    /// Test that making a rule with no "current connection" generates a noop.
    #[tokio::test]
    async fn test_make_rule_no_conn() {
        let app = App::new(
            &"127.0.0.1:65534".to_string(),
            &"deny".to_string(),
            &"12h".to_string(),
            &60,
            &"exact_dst_ip".to_string(),
        )
        .expect("new failed");

        assert!(app.tui_state.current_connection.is_none());

        let maybe_rule = app.make_rule(constants::Action::Allow, constants::Duration::Once);
        assert!(maybe_rule.is_none());
    }

    /// Helper to make a fake connection object.
    fn make_fake_connection() -> Connection {
        Connection {
            protocol: String::from("tcp"),
            src_ip: String::from("192.168.0.3"),
            src_port: 1337,
            dst_ip: String::from("192.128.0.4"),
            dst_host: String::from("suspicious.local"),
            dst_port: 1338,
            user_id: 1000,
            process_id: 1339,
            process_path: String::from("/usr/bin/hello"),
            process_cwd: String::from("/home/spongebob"),
            process_args: vec![],
            process_env: S2SMap::default(),
            process_checksums: S2SMap::default(),
            process_tree: vec![],
        }
    }

    /// Test that making a rule with a valid "current connection" generates something meaningful.
    #[tokio::test]
    async fn test_make_rule_has_conn() {
        let mut app = App::new(
            &"127.0.0.1:65534".to_string(),
            &"deny".to_string(),
            &"12h".to_string(),
            &60,
            &"exact_dst_ip".to_string(),
        )
        .expect("new failed");

        let fake_conn = make_fake_connection();
        app.tui_state.current_connection = Some(ConnectionEvent {
            connection: fake_conn.clone(),
            expiry_ts: SystemTime::now() + app.connection_disposition_timeout,
        });

        let maybe_rule = app
            .make_rule(constants::Action::Allow, constants::Duration::Once)
            .expect("missing rule");

        // JSON blob representing the vector of operators we use for a rule.
        // Checking this also acts as a high-level test for serde_json not producing
        // unexpected output in the future.
        let expected_str = "[{\"type\":\"simple\",\"operand\":\"dest.ip\",\"data\":\"192.128.0.4\",\"sensitive\":false,\"list\":[]}]";

        let expected_rule = Rule {
            created: 0,
            name: String::from("allow-once-simple-via-tui--usr-bin-hello"),
            description: String::default(),
            enabled: true,
            precedence: false,
            nolog: false,
            action: String::from("allow"),
            duration: String::from("once"),
            operator: Some(pb::Operator {
                r#type: String::from(constants::RuleType::List.get_str()),
                operand: String::from(constants::Operand::List.get_str()),
                data: String::from(expected_str),
                sensitive: false,
                list: vec![pb::Operator {
                    r#type: String::from(constants::RuleType::Simple.get_str()),
                    operand: String::from(constants::Operand::DstIp.get_str()),
                    data: "192.128.0.4".to_string(),
                    sensitive: false,
                    list: Vec::default(),
                }],
            }),
        };

        assert_eq!(maybe_rule, expected_rule)
    }
}
