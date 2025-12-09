use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, List, ListItem, Paragraph, Widget},
};

use crate::app::{App, Button};
use crate::constants;

/// Button labels and their actions.
const BUTTONS: &[(&str, constants::Action, constants::Duration)] = &[
    (" Allow(A) ", constants::Action::Allow, constants::Duration::UntilRestart),
    (" Deny(D) ", constants::Action::Deny, constants::Duration::UntilRestart),
    (" Always(J) ", constants::Action::Allow, constants::Duration::Always),
    (" Never(L) ", constants::Action::Deny, constants::Duration::Always),
];

const BTN_SPACING: u16 = 1;
const BTN_GROUP_SPACING: u16 = 2;

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Delegate to stateful render (buttons already updated before draw)
        self.render_inner(area, buf);
    }
}

impl App {
    /// Updates button areas based on layout. Call before drawing.
    pub fn update_button_areas(&mut self, area: Rect) {
        self.buttons.clear();
        if self.current_connection.is_none() {
            return;
        }
        let areas = Layout::vertical([
            Constraint::Max(6),
            Constraint::Max(9),
            Constraint::Max(5),
            Constraint::Max(2),
        ])
        .split(area);
        let conn_inner = Block::bordered().inner(areas[1]);
        // Buttons on second-to-last line of connection panel, centered
        let btn_y = conn_inner.y + conn_inner.height.saturating_sub(2);
        // Calculate total width: buttons + spacing (group spacing after index 1)
        let total_width: u16 = BUTTONS.iter().enumerate()
            .map(|(i, (l, _, _))| {
                let spacing = if i == 1 { BTN_GROUP_SPACING } else if i < BUTTONS.len() - 1 { BTN_SPACING } else { 0 };
                l.len() as u16 + spacing
            })
            .sum();
        let start_x = conn_inner.x + conn_inner.width.saturating_sub(total_width) / 2;
        let mut x = start_x;
        for (i, (label, action, duration)) in BUTTONS.iter().enumerate() {
            let w = label.len() as u16;
            if x + w <= conn_inner.x + conn_inner.width {
                self.buttons.push(Button {
                    area: Rect::new(x, btn_y, w, 1),
                    action: *action,
                    duration: *duration,
                });
            }
            let spacing = if i == 1 { BTN_GROUP_SPACING } else { BTN_SPACING };
            x += w + spacing;
        }
    }

    fn render_inner(&self, area: Rect, buf: &mut Buffer) {
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
        let has_conn = self.current_connection.is_some();
        let connection_block = Block::bordered()
            .title(" New Connections ")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .title_style(if has_conn { Style::default().bold() } else { Style::default() })
            .style(if has_conn {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            });

        let connection_text = self.format_connection_panel();
        let connection_paragraph = Paragraph::new(connection_text)
            .block(connection_block)
            .bg(Color::Black)
            .alignment(Alignment::Center);

        connection_paragraph.render(areas[1], buf);

        // Render buttons if connection pending
        if has_conn {
            for btn in &self.buttons {
                let label = BUTTONS.iter()
                    .find(|(_, a, d)| std::mem::discriminant(a) == std::mem::discriminant(&btn.action)
                        && std::mem::discriminant(d) == std::mem::discriminant(&btn.duration))
                    .map(|(l, _, _)| *l)
                    .unwrap_or("");
                let style = match btn.action {
                    constants::Action::Allow => Style::default().fg(Color::Black).bg(Color::Green),
                    constants::Action::Deny => Style::default().fg(Color::Black).bg(Color::Red),
                    _ => Style::default(),
                };
                buf.set_string(btn.area.x, btn.area.y, label, style);
            }
        }

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
                let dst_host = if info.connection.dst_host.is_empty() {
                    "-"
                } else {
                    &info.connection.dst_host
                };
                let src = format!("{}:{}", info.connection.src_ip, info.connection.src_port);
                let dst = format!("{}:{}", info.connection.dst_ip, info.connection.dst_port);
                format!(
                    "\n{}\n{} → {} [{}]\nuid:{} pid:{} {}",
                    dst_host,
                    src,
                    dst,
                    info.connection.protocol,
                    info.connection.user_id,
                    info.connection.process_id,
                    info.connection.process_path,
                )
            }
        }
    }
}
