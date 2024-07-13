use ratatui::{prelude::*, widgets::*};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.size());
    let titles = app
        .tabs
        .titles
        .iter()
        .map(|t| text::Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();
    let tabs = if app.should_pause {
        let pause_title = format!("{} [Paused] press <SPACE> to resume", app.title);
        Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(pause_title)
                    .style(Style::default().fg(Color::Yellow)),
            )
            .highlight_style(Style::default().fg(Color::LightBlue))
            .select(app.tabs.index)
    } else {
        Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(app.title))
            .highlight_style(Style::default().fg(Color::LightBlue))
            .select(app.tabs.index)
    };
    f.render_widget(tabs, chunks[0]);
    match app.tabs.index {
        0 => draw_overview_tab(f, app, chunks[1]),
        1 => draw_remotehosts_tab(f, app, chunks[1]),
        2 => draw_connections_tab(f, app, chunks[1]),
        _ => {}
    };
    // Draw footer
    let footer = format!("Press <Q> to quit, <TAB> to switch tabs, <SPACE> to pause, <T> to toggle bandwidth display, <Up>/<Down> to scroll");
    let footer = Paragraph::new(text::Line::from(Span::styled(
        footer,
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(footer, chunks[2]);
}

fn draw_summary(f: &mut Frame, app: &mut App, area: Rect) {
    // Draw network interface
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        //.margin(1)
        .split(area);
    let ipv4_addr: String = if let Some(ipv4) =
        crate::net::interface::get_interface_ipv4(&app.netstat_data.default_interface)
    {
        ipv4.to_string()
    } else {
        "".to_string()
    };
    let ipv6_addr: String = if let Some(ipv6) =
        crate::net::interface::get_interface_local_ipv6(&app.netstat_data.default_interface)
    {
        ipv6.to_string()
    } else {
        "".to_string()
    };
    let text1 = vec![
        text::Line::from(format!("IPv4: {}", ipv4_addr)),
        text::Line::from(format!("IPv6: {}", ipv6_addr)),
    ];
    let title = format!(
        "Default Interface: [{}] {}",
        app.netstat_data.default_interface.index, app.netstat_data.default_interface.name
    );
    let block1 = Block::default().borders(Borders::ALL).title(title);
    let paragraph1 = Paragraph::new(text1)
        .block(block1)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph1, chunks[0]);

    // Draw total ingress
    let ingress_packets: String = if app.config.display.show_bandwidth {
        app.netstat_data.traffic.formatted_ingress_packets_per_sec()
    } else {
        app.netstat_data.traffic.packet_received.to_string()
    };
    let ingress_traffic: String = if app.config.display.show_bandwidth {
        app.netstat_data.traffic.formatted_ingress_bytes_per_sec()
    } else {
        app.netstat_data.traffic.formatted_received_bytes()
    };
    let text2 = vec![
        text::Line::from(format!("Packets: {}", ingress_packets)),
        text::Line::from(format!("Bytes: {}", ingress_traffic)),
    ];
    let block2 = Block::default()
        .borders(Borders::ALL)
        .title("Total Ingress");
    let paragraph2 = Paragraph::new(text2)
        .block(block2)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph2, chunks[1]);

    // Draw total egress
    let egress_packets: String = if app.config.display.show_bandwidth {
        app.netstat_data.traffic.formatted_egress_packets_per_sec()
    } else {
        app.netstat_data.traffic.packet_sent.to_string()
    };
    let eggress_traffic: String = if app.config.display.show_bandwidth {
        app.netstat_data.traffic.formatted_egress_bytes_per_sec()
    } else {
        app.netstat_data.traffic.formatted_sent_bytes()
    };
    let text3 = vec![
        text::Line::from(format!("Packets: {}", egress_packets)),
        text::Line::from(format!("Bytes: {}", eggress_traffic)),
    ];
    let block3 = Block::default().borders(Borders::ALL).title("Total Egress");
    let paragraph3 = Paragraph::new(text3)
        .block(block3)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph3, chunks[2]);
}

fn draw_top_data(f: &mut Frame, app: &mut App, area: Rect) {
    let area_chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(100)])
        .direction(Direction::Horizontal)
        .split(area);
    {
        let inner_chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area_chunks[0]);

        // Draw top Remote Address Table
        let rows = app
            .remote_hosts
            .iter()
            .take(app.config.display.top_remote_hosts)
            .map(|host| {
                let ingress_traffic: String = if app.config.display.show_bandwidth {
                    host.traffic.formatted_ingress_bytes_per_sec.clone()
                } else {
                    host.traffic.formatted_received_bytes.clone()
                };
                let egress_traffic: String = if app.config.display.show_bandwidth {
                    host.traffic.formatted_egress_bytes_per_sec.clone()
                } else {
                    host.traffic.formatted_sent_bytes.clone()
                };
                Row::new(vec![
                    host.ip_addr.to_string(),
                    ingress_traffic,
                    egress_traffic,
                    host.country_code.clone(),
                    host.asn.to_string(),
                    host.as_name.clone(),
                ])
            })
            .collect::<Vec<Row>>();
        let widths = [
            Constraint::Length(40),
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(24),
        ];

        //let mut table_state = TableState::default();
        let table = Table::new(rows, widths)
            .column_spacing(1)
            .header(
                Row::new(vec![
                    "IP Address",
                    "↓ Bytes",
                    "↑ Bytes",
                    "Country",
                    "ASN",
                    "AS Name",
                ])
                .style(Style::new().bold()), //.bottom_margin(1),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Top Remote Addresses"),
            )
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>");

        f.render_widget(table, inner_chunks[0]);

        let rows = app
            .connections
            .iter()
            .take(app.config.display.connection_count)
            .map(|conn| {
                let remote_ip_string = if let Some(remote_ip_addr) = &conn.remote_ip_addr {
                    remote_ip_addr.to_string()
                } else {
                    "".to_string()
                };
                let remote_port_string = if let Some(remote_port) = &conn.remote_port {
                    remote_port.to_string()
                } else {
                    "".to_string()
                };
                let mut process_id_string = "".to_string();
                let mut process_name_string = "".to_string();
                if let Some(process) = &conn.process {
                    process_id_string = process.pid.to_string();
                    process_name_string = process.name.clone();
                }
                let ingress_traffic: String = if app.config.display.show_bandwidth {
                    conn.traffic.formatted_ingress_bytes_per_sec.clone()
                } else {
                    conn.traffic.formatted_received_bytes.clone()
                };
                let egress_traffic: String = if app.config.display.show_bandwidth {
                    conn.traffic.formatted_egress_bytes_per_sec.clone()
                } else {
                    conn.traffic.formatted_sent_bytes.clone()
                };
                Row::new(vec![
                    conn.protocol.as_str().to_string(),
                    format!("{}:{}", conn.local_ip_addr.to_string(), conn.local_port.to_string()),
                    format!("{}:{}", remote_ip_string, remote_port_string),
                    ingress_traffic,
                    egress_traffic,
                    process_id_string,
                    process_name_string,
                ])
            })
            .collect::<Vec<Row>>();
        let widths = [
            Constraint::Length(8),
            Constraint::Length(46),
            Constraint::Length(46),
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Length(5),
            Constraint::Length(20),
        ];
        let table = Table::new(rows, widths)
            .column_spacing(1)
            .header(
                Row::new(vec![
                    "Protocol",
                    "Local Socket",
                    "Remote Socket",
                    "↓ Bytes",
                    "↑ Bytes",
                    "PID",
                    "Process Name",
                ])
                .style(Style::new().bold()), //.bottom_margin(1),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Top Connections"),
            )
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>");
        f.render_widget(table, inner_chunks[1]);
        //f.render_stateful_widget(table, inner_chunks[1], &mut app.talbe_state);
    }
}

fn draw_remotehosts_table(f: &mut Frame, app: &mut App, area: Rect) {
    // Draw top Remote Address Table
    let rows = app
        .remote_hosts
        .iter()
        .map(|host| {
            let ingress_traffic: String = if app.config.display.show_bandwidth {
                host.traffic.formatted_ingress_bytes_per_sec.clone()
            } else {
                host.traffic.formatted_received_bytes.clone()
            };
            let egress_traffic: String = if app.config.display.show_bandwidth {
                host.traffic.formatted_egress_bytes_per_sec.clone()
            } else {
                host.traffic.formatted_sent_bytes.clone()
            };
            Row::new(vec![
                host.ip_addr.to_string(),
                ingress_traffic,
                egress_traffic,
                host.country_code.clone(),
                host.asn.to_string(),
                host.as_name.clone(),
            ])
        })
        .collect::<Vec<Row>>();
    let widths = [
        Constraint::Length(40),
        Constraint::Length(11),
        Constraint::Length(11),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(24),
    ];

    //let mut table_state = TableState::default();
    let table = Table::new(rows, widths)
        .column_spacing(1)
        //.style(Style::new().blue())
        .header(
            Row::new(vec![
                "IP Address",
                "↓ Bytes",
                "↑ Bytes",
                "Country",
                "ASN",
                "AS Name",
            ])
            .style(Style::new().bold()), //.bottom_margin(1),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Remote Addresses"),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    //f.render_widget(table, area);
    f.render_stateful_widget(table, area, &mut app.talbe_state);
}

fn draw_connection_table(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = app
        .connections
        .iter()
        .map(|conn| {
            let remote_ip_string = if let Some(remote_ip_addr) = &conn.remote_ip_addr {
                remote_ip_addr.to_string()
            } else {
                "".to_string()
            };
            let remote_port_string = if let Some(remote_port) = &conn.remote_port {
                remote_port.to_string()
            } else {
                "".to_string()
            };
            let mut process_id_string = "".to_string();
            let mut process_name_string = "".to_string();
            if let Some(process) = &conn.process {
                process_id_string = process.pid.to_string();
                process_name_string = process.name.clone();
            }
            let ingress_traffic: String = if app.config.display.show_bandwidth {
                conn.traffic.formatted_ingress_bytes_per_sec.clone()
            } else {
                conn.traffic.formatted_received_bytes.clone()
            };
            let egress_traffic: String = if app.config.display.show_bandwidth {
                conn.traffic.formatted_egress_bytes_per_sec.clone()
            } else {
                conn.traffic.formatted_sent_bytes.clone()
            };
            Row::new(vec![
                conn.protocol.as_str().to_string(),
                format!("{}:{}", conn.local_ip_addr.to_string(), conn.local_port.to_string()),
                format!("{}:{}", remote_ip_string, remote_port_string),
                ingress_traffic,
                egress_traffic,
                process_id_string,
                process_name_string,
            ])
        })
        .collect::<Vec<Row>>();
    let widths = [
        Constraint::Length(8),
        Constraint::Length(46),
        Constraint::Length(46),
        Constraint::Length(11),
        Constraint::Length(11),
        Constraint::Length(5),
        Constraint::Length(20),
    ];
    let table = Table::new(rows, widths)
        .column_spacing(1)
        .header(
            Row::new(vec![
                "Protocol",
                "Local Socket",
                "Remote Socket",
                "↓ Bytes",
                "↑ Bytes",
                "PID",
                "Process Name",
            ])
            .style(Style::new().bold()), //.bottom_margin(1),
        )
        .block(Block::default().borders(Borders::ALL).title("Connections"))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>");
    //f.render_widget(table, area);
    f.render_stateful_widget(table, area, &mut app.talbe_state);
}

fn draw_overview_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(4), Constraint::Min(8)])
        .split(area);
    draw_summary(f, app, chunks[0]);
    draw_top_data(f, app, chunks[1]);
}

fn draw_remotehosts_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(100)])
        .split(area);
    draw_remotehosts_table(f, app, chunks[0]);
}

fn draw_connections_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(100)])
        .split(area);
    draw_connection_table(f, app, chunks[0]);
}
