use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::palette::tailwind::SLATE,
    style::{Color, Stylize},
    widgets::{Block, BorderType, List, ListItem, Paragraph, Widget},
};

use crate::app::App;

impl Widget for &App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas = Layout::vertical([
            Constraint::Max(7),
            Constraint::Max(5),
            Constraint::Max(10),
            Constraint::Max(1),
        ])
        .split(area);
        let stats_block = Block::bordered()
            .title("OpenSnitch")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let stats_text = format!(
            "STATISTICS\n\
                rx pings: {} | daemon version: {} | rules: {}\n\
                uptime: {} | dns_responses: {} | connections: {}\n\
                ignored: {} | accepted: {} | dropped: {}\n\
                rule_hits: {} | rule_misses: {}",
            self.rx_pings,
            self.current_stats.daemon_version,
            self.current_stats.rules,
            self.current_stats.uptime,
            self.current_stats.dns_responses,
            self.current_stats.connections,
            self.current_stats.ignored,
            self.current_stats.accepted,
            self.current_stats.dropped,
            self.current_stats.rule_hits,
            self.current_stats.rule_misses,
        );

        let stats_paragraph = Paragraph::new(stats_text)
            .block(stats_block)
            .fg(Color::Cyan)
            .bg(Color::Black);

        stats_paragraph.render(areas[0], buf);

        // Alerts list
        let alerts_block = Block::bordered()
            .title(format!("Alerts ({})", self.current_alerts.len()))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        // abtodo: prettify proto enums to text
        // abtodo: make this scrollable
        let items: Vec<ListItem> = self
            .current_alerts
            .iter()
            .enumerate()
            .map(|(i, alert)| {
                let color = alternate_colors(i);
                let alert_text = format!(
                    "type: {} action: {} priority: {} what: {}\n",
                    alert.r#type, alert.action, alert.priority, alert.what,
                );
                ListItem::from(alert_text).bg(color)
            })
            .collect();

        // Create a List from all list items
        let list = List::new(items)
            .block(alerts_block)
            .fg(Color::Cyan)
            .bg(Color::Black);

        list.render(areas[1], buf);

        // Connection controls
        let connection_block = Block::bordered()
            .title("New Connections")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let mut connection_text = String::default();
        match &self.current_connection {
            None => {}
            Some(info) => {
                // Don't just leave field blank if not populated.
                let dst_host_string = if info.connection.dst_host.is_empty() {
                    "-"
                } else {
                    &info.connection.dst_host
                };

                connection_text = format!(
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
                );
            }
        }

        let connection_paragraph = Paragraph::new(connection_text)
            .block(connection_block)
            .fg(Color::Cyan)
            .bg(Color::Black);

        connection_paragraph.render(areas[2], buf);

        // Controls
        let controls_text = format!(
            "\
        `ctrl+c` -> quit | `a` -> accept connection 12h | `d` -> deny connection 12h"
        );

        let controls_paragraph = Paragraph::new(controls_text)
            .bg(Color::DarkGray)
            .fg(Color::White);

        controls_paragraph.render(areas[3], buf);
    }
}

const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
