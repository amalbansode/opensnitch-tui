use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, Paragraph, StatefulWidget, Widget},
};

use crate::app::{TuiMutState, TuiScreen, TuiState};

impl StatefulWidget for &TuiState {
    type State = TuiMutState;

    /// Renders the user interface widgets.
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match self.current_screen {
            TuiScreen::Main => {
                self.render_main_screen(area, buf, state);
            }
            TuiScreen::Help => {
                TuiState::render_help_screen(area, buf, state);
            }
        }
    }
}

impl TuiState {
    /// Renders main screen.
    fn render_main_screen(&self, area: Rect, buf: &mut Buffer, state: &mut TuiMutState) {
        // Get a clock reference timestamp.
        let now = std::time::SystemTime::now();

        // Fill greediness prioritizes connections > controls > stats > alerts
        let areas = Layout::vertical([
            Constraint::Max(6),      // Stats
            Constraint::Min(9),      // Connections
            Constraint::Fill(10000), // Alerts - high fill ratio prevents Mins from growing
            Constraint::Min(1),      // Controls
        ])
        .split(area);
        let stats_title = match self.peer {
            Some(addr) => format!(" OpenSnitch ({addr}) "),
            _ => String::from(" OpenSnitch "),
        };
        let stats_block = Block::bordered()
            .title(stats_title)
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let stats_text = self.format_stats_panel();
        let stats_paragraph = Paragraph::new(stats_text)
            .block(stats_block)
            .fg(Color::Cyan)
            .bg(Color::Black);

        stats_paragraph.render(areas[0], buf);

        // Connection controls
        let mut connection_block = Block::bordered()
            .title(" New Connections ")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .title_style(match &self.current_connection {
                None => Style::default(),
                Some(_) => Style::default().bold(),
            })
            .style(match &self.current_connection {
                None => Style::default().fg(Color::Cyan),
                Some(_) => Style::default().fg(Color::Yellow),
            });
        // Also render a "bottom title" with countdown to dispo the connection.
        if let Some(conn) = &self.current_connection
            && let Ok(remaining_time) = conn.expiry_ts.duration_since(now)
        {
            connection_block = connection_block.title_bottom(
                Line::from(format!(
                    " {}s to disposition, else {} ",
                    remaining_time.as_secs(),
                    self.default_action.get_str()
                ))
                .alignment(Alignment::Right),
            );
        }

        let connection_text = self.format_connection_panel();
        let connection_paragraph = Paragraph::new(connection_text)
            .block(connection_block)
            .bg(Color::Black);

        connection_paragraph.render(areas[1], buf);

        // Alerts list
        let alerts_block = Block::bordered()
            .title(format!(" Alerts ({}) ", self.current_alerts.len()))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        // We want to render the alert list from some stateful head index,
        // so get an iterator and skip forward to that head.
        let items_iter = self
            .current_alerts
            .iter()
            .skip(self.alert_list_render_offset);

        let items: Vec<ListItem> = items_iter
            .map(|alert| {
                let maybe_age = now.duration_since(alert.timestamp);
                let age_s: u64 = match maybe_age {
                    Ok(age) => age.as_secs(),
                    Err(_) => 0, // Just default to 0s in case time goes backwards
                };
                let alert_text = format!(
                    "{}s ago : {:?} : {:?} : {:?} : {}\n",
                    age_s, alert.r#type, alert.priority, alert.what, alert.msg,
                );
                ListItem::from(alert_text)
            })
            .collect();

        // Create a List from all list items
        let list = List::new(items)
            .block(alerts_block)
            .fg(Color::Cyan)
            .bg(Color::Black);
        Widget::render(list, areas[2], buf);

        // Controls footer
        let mut controls_spans = Vec::new();
        for control in &self.controls {
            controls_spans.push(Span::styled(
                control.get_keybind_str(),
                Style::default().bg(Color::Black).fg(Color::White),
            ));
            controls_spans.push(Span::styled(
                control.get_control_str(),
                Style::default().bg(Color::Gray).fg(Color::Black),
            ));
        }
        let controls_text = vec![controls_spans.into()];

        // Important: Left alignment makes it super easy to calculate whether
        // mouse clicks occurred over a control "button"
        let controls_paragraph = Paragraph::new(controls_text)
            .bg(Color::Black)
            .alignment(Alignment::Left);

        controls_paragraph.render(areas[3], buf);
        state.controls_area = areas[3];
    }

    fn render_help_screen(area: Rect, buf: &mut Buffer, _state: &mut TuiMutState) {
        let help_title = String::from(" OpenSnitch TUI Help ");
        let help_block = Block::bordered()
            .title(help_title)
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let mut help_lines: Vec<Line<'_>> = Vec::default();
        let version = env!("CARGO_PKG_VERSION");
        let author = env!("CARGO_PKG_AUTHORS");
        help_lines.push(Line::styled(
            format!("opensnitch-tui {version} by {author}. Released under the GNU GPLv3."),
            Style::default().fg(Color::Cyan),
        ));
        help_lines.push(Line::default()); // blank

        help_lines.push(Line::styled(
            "Run the binary with --help to see details on CLI arguments.",
            Style::default().fg(Color::Cyan),
        ));
        help_lines.push(Line::default()); // blank

        help_lines.push(Line::styled(
            "Keybindings",
            Style::default().fg(Color::Cyan).bold(),
        ));
        // Very scrappy vec of per-line k-vs in help screen
        // 1st => some keybinding
        // 2nd => description
        // Technically slightly repetitive of main screen's footer.
        let kv_raw_lines = vec![
            ("Ctrl+C", "Quit"),
            ("ESC", "Return to main screen"),
            ("H", "Display this help screen"),
            ("A", "Allow connection temporarily"),
            ("D", "Deny connection temporarily"),
            ("J", "Allow connection forever"),
            ("L", "Deny connection forever"),
            ("Arrows", "Scroll alert list"),
        ];
        for (raw_k, raw_v) in kv_raw_lines {
            help_lines.push(Line::from(vec![
                Span::styled(format!("{raw_k:>7} "), Style::default().fg(Color::White)),
                Span::styled(raw_v.to_string(), Style::default().fg(Color::Cyan)),
            ]));
        }
        help_lines.push(Line::default()); // blank

        help_lines.push(Line::styled(
            "The main screen's footer with keybinding hints is clickable.",
            Style::default().fg(Color::Cyan),
        ));

        let help_paragraph = Paragraph::new(help_lines)
            .block(help_block)
            .fg(Color::Cyan)
            .bg(Color::Black);

        help_paragraph.render(area, buf);
    }

    fn format_stats_panel(&self) -> String {
        match &self.current_stats {
            Some(stats) => {
                format!(
                    "\
                        daemon version: {} | uptime: {}\n\
                        rules: {} | dns responses: {} | connections: {}\n\
                        ignored: {} | accepted: {} | dropped: {}\n\
                        rule hits: {} | rule misses: {}",
                    stats.daemon_version,
                    stats.uptime,
                    stats.rules,
                    stats.dns_responses,
                    stats.connections,
                    stats.ignored,
                    stats.accepted,
                    stats.dropped,
                    stats.rule_hits,
                    stats.rule_misses,
                )
            }
            None => String::default(), // Consider a more useful message in the future?
        }
    }

    fn format_connection_panel(&self) -> String {
        match &self.current_connection {
            None => String::default(),
            Some(info) => {
                // Don't just leave field blank if not populated.
                let dst_host_string = if info.connection.dst_host.is_empty() {
                    "-"
                } else {
                    &info.connection.dst_host
                };

                let src_ip = format_ip_address_string(&info.connection.src_ip);
                let dst_ip = format_ip_address_string(&info.connection.dst_ip);

                format!(
                    "\
                src       {}:{}\n\
                dst       {}:{}\n\
                proto     {}\n\
                dst host  {}\n\
                uid       {}\n\
                pid       {}\n\
                ppath     {}",
                    src_ip,
                    info.connection.src_port,
                    dst_ip,
                    info.connection.dst_port,
                    info.connection.protocol,
                    dst_host_string,
                    info.connection.user_id,
                    info.connection.process_id,
                    info.connection.process_path,
                )
            }
        }
    }
}

/// Format IPv6 addresses (that are already strings) with square brackets. Noop if IPv4.
fn format_ip_address_string(ip: &String) -> String {
    if ip.contains(':') {
        format!("[{ip}]")
    } else {
        ip.clone()
    }
}
