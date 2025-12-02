use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, List, ListItem, Paragraph, Widget},
};

use crate::app::App;

impl Widget for &App {
    /// Renders the user interface widgets.
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas = Layout::vertical([
            Constraint::Max(6),
            Constraint::Max(9),
            Constraint::Max(5),
            Constraint::Max(2),
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
        let connection_block = Block::bordered()
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

        // Get a clock reference timestamp to compute alert ages.
        let now = std::time::SystemTime::now();

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
        list.render(areas[2], buf);

        // Controls footer
        let controls_text = format!(
            "\
        `ctrl+C` → quit | `A/D` → (allow/deny) connection {}\n\
        `J/L` → (allow/deny) connection forever | `up/down` → scroll alerts",
            self.temp_rule_lifetime.get_str(),
        );

        let controls_paragraph = Paragraph::new(controls_text)
            .bg(Color::DarkGray)
            .fg(Color::White)
            .alignment(Alignment::Center);

        controls_paragraph.render(areas[3], buf);
    }
}

impl App {
    fn format_stats_panel(&self) -> String {
        match &self.current_stats {
            Some(stats) => {
                format!(
                    "\
                        rx pings: {} | daemon version: {} | rules: {}\n\
                        uptime: {} | dns_responses: {} | connections: {}\n\
                        ignored: {} | accepted: {} | dropped: {}\n\
                        rule_hits: {} | rule_misses: {}",
                    self.rx_pings,
                    stats.daemon_version,
                    stats.rules,
                    stats.uptime,
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

                format!(
                    "\
                src       {} / {}\n\
                dst       {} / {}\n\
                proto     {}\n\
                dst host  {}\n\
                uid       {}\n\
                pid       {}\n\
                ppath     {}",
                    info.connection.src_ip,
                    info.connection.src_port,
                    info.connection.dst_ip,
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
