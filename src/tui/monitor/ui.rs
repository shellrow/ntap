use crate::tui::monitor::app::{App, FocusTab};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;

pub fn draw(f: &mut Frame, app: &mut App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_header(f, app, layout[0]);
    draw_summary(f, app, layout[1]);

    match app.selected_tab {
        FocusTab::Overview => draw_overview(f, app, layout[2]),
        FocusTab::Hosts => draw_hosts(f, app, layout[2]),
        FocusTab::Connections => draw_connections(f, app, layout[2]),
        FocusTab::Processes => draw_processes(f, app, layout[2]),
    }

    let footer = Paragraph::new(
        "<Tab>/<Shift+Tab> switch tabs | <Space> pause | <T> total/bandwidth | <Q> quit",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, layout[3]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let tabs = Tabs::new(
        FocusTab::ALL
            .iter()
            .map(|tab| Line::raw(tab.title()))
            .collect::<Vec<Line>>(),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(if app.should_pause {
                format!("{} [PAUSED]", app.title)
            } else {
                app.title.clone()
            }),
    )
    .select(
        FocusTab::ALL
            .iter()
            .position(|tab| *tab == app.selected_tab)
            .unwrap_or(0),
    )
    .highlight_style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .divider(" | ");
    f.render_widget(tabs, area);
}

fn draw_summary(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let ingress_packets = if app.show_bandwidth {
        app.netstat_data.traffic.formatted_ingress_packets_per_sec()
    } else {
        app.netstat_data.traffic.packet_received.to_string()
    };
    let ingress_bytes = if app.show_bandwidth {
        app.netstat_data.traffic.formatted_ingress_bytes_per_sec()
    } else {
        app.netstat_data.traffic.formatted_received_bytes()
    };
    let egress_packets = if app.show_bandwidth {
        app.netstat_data.traffic.formatted_egress_packets_per_sec()
    } else {
        app.netstat_data.traffic.packet_sent.to_string()
    };
    let egress_bytes = if app.show_bandwidth {
        app.netstat_data.traffic.formatted_egress_bytes_per_sec()
    } else {
        app.netstat_data.traffic.formatted_sent_bytes()
    };

    let ingress = Paragraph::new(vec![
        Line::raw(format!("Packets: {ingress_packets}")),
        Line::raw(format!("Bytes:   {ingress_bytes}")),
    ])
    .block(Block::default().borders(Borders::ALL).title("Ingress"));

    let egress = Paragraph::new(vec![
        Line::raw(format!("Packets: {egress_packets}")),
        Line::raw(format!("Bytes:   {egress_bytes}")),
    ])
    .block(Block::default().borders(Borders::ALL).title("Egress"));

    f.render_widget(ingress, chunks[0]);
    f.render_widget(egress, chunks[1]);
}

fn draw_overview(f: &mut Frame, app: &App, area: Rect) {
    draw_overview_flows(f, app, area);
}

fn draw_hosts(f: &mut Frame, app: &App, area: Rect) {
    draw_top_hosts(f, app, area, None);
}

fn draw_connections(f: &mut Frame, app: &App, area: Rect) {
    draw_top_connections(f, app, area, None);
}

fn draw_processes(f: &mut Frame, app: &App, area: Rect) {
    let rows = app.processes.iter().map(|process| {
        let ingress = if app.show_bandwidth {
            process.traffic.formatted_ingress_bytes_per_sec.clone()
        } else {
            process.traffic.formatted_received_bytes.clone()
        };
        let egress = if app.show_bandwidth {
            process.traffic.formatted_egress_bytes_per_sec.clone()
        } else {
            process.traffic.formatted_sent_bytes.clone()
        };
        Row::new(vec![
            process.pid.to_string(),
            process.name.clone(),
            ingress,
            egress,
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(14),
            Constraint::Length(14),
        ],
    )
    .header(
        Row::new(["PID", "Process", "Ingress", "Egress"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title("Processes"))
    .column_spacing(1);
    f.render_widget(table, area);
}

fn draw_top_hosts(f: &mut Frame, app: &App, area: Rect, limit: Option<usize>) {
    let rows = app
        .remote_hosts
        .iter()
        .take(limit.unwrap_or(app.remote_hosts.len()))
        .map(|host| {
            let ingress = if app.show_bandwidth {
                host.traffic.formatted_ingress_bytes_per_sec.clone()
            } else {
                host.traffic.formatted_received_bytes.clone()
            };
            let egress = if app.show_bandwidth {
                host.traffic.formatted_egress_bytes_per_sec.clone()
            } else {
                host.traffic.formatted_sent_bytes.clone()
            };
            Row::new(vec![
                host.ip_addr.to_string(),
                ingress,
                egress,
                host.hostname.clone(),
            ])
        });

    let table = Table::new(
        rows,
        [
            Constraint::Length(40),
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Min(16),
        ],
    )
    .header(
        Row::new(["IP", "Ingress", "Egress", "Hostname"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title("Remote Hosts"))
    .column_spacing(1);
    f.render_widget(table, area);
}

fn draw_top_connections(f: &mut Frame, app: &App, area: Rect, limit: Option<usize>) {
    let rows = app
        .connections
        .iter()
        .take(limit.unwrap_or(app.connections.len()))
        .map(|conn| {
            let remote_ip = conn
                .remote_ip_addr
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "-".to_string());
            let remote_port = conn
                .remote_port
                .map(|port| port.to_string())
                .unwrap_or_else(|| "-".to_string());
            let ingress = if app.show_bandwidth {
                conn.traffic.formatted_ingress_bytes_per_sec.clone()
            } else {
                conn.traffic.formatted_received_bytes.clone()
            };
            let egress = if app.show_bandwidth {
                conn.traffic.formatted_egress_bytes_per_sec.clone()
            } else {
                conn.traffic.formatted_sent_bytes.clone()
            };
            let process_name = conn
                .process
                .as_ref()
                .map(|process| process.name.clone())
                .unwrap_or_else(|| "-".to_string());
            Row::new(vec![
                conn.protocol.as_str().to_string(),
                format!("{}:{}", conn.local_ip_addr, conn.local_port),
                format!("{remote_ip}:{remote_port}"),
                ingress,
                egress,
                process_name,
            ])
        });

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(32),
            Constraint::Length(32),
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Min(16),
        ],
    )
    .header(
        Row::new(["Proto", "Local", "Remote", "Ingress", "Egress", "Process"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title("Connections"))
    .column_spacing(1);
    f.render_widget(table, area);
}

fn draw_overview_flows(f: &mut Frame, app: &App, area: Rect) {
    let hostnames: HashMap<_, _> = app
        .remote_hosts
        .iter()
        .filter_map(|host| {
            if host.hostname.is_empty() {
                None
            } else {
                Some((host.ip_addr, host.hostname.as_str()))
            }
        })
        .collect();

    let rows = app
        .connections
        .iter()
        .take(app.config.display.connection_count)
        .map(|conn| {
            let remote_ip = conn
                .remote_ip_addr
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "-".to_string());
            let remote_port = conn
                .remote_port
                .map(|port| port.to_string())
                .unwrap_or_else(|| "-".to_string());
            let ingress = if app.show_bandwidth {
                conn.traffic.formatted_ingress_bytes_per_sec.clone()
            } else {
                conn.traffic.formatted_received_bytes.clone()
            };
            let egress = if app.show_bandwidth {
                conn.traffic.formatted_egress_bytes_per_sec.clone()
            } else {
                conn.traffic.formatted_sent_bytes.clone()
            };
            let hostname = conn
                .remote_ip_addr
                .and_then(|ip| hostnames.get(&ip).copied())
                .unwrap_or("-");
            let process = conn
                .process
                .as_ref()
                .map(|p| p.name.as_str())
                .unwrap_or("-");

            Row::new(vec![
                format!("{}:{}", conn.local_ip_addr, conn.local_port),
                format!("{remote_ip}:{remote_port}"),
                ingress,
                egress,
                hostname.to_string(),
                process.to_string(),
            ])
        });

    let table = Table::new(
        rows,
        [
            Constraint::Length(31),
            Constraint::Length(31),
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Min(20),
            Constraint::Min(16),
        ],
    )
    .header(
        Row::new([
            "Local Address",
            "Remote Address",
            "Ingress",
            "Egress",
            "Hostname",
            "Process",
        ])
        .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Overview Connections"),
    )
    .column_spacing(1);

    f.render_widget(table, area);
}
