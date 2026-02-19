use super::app::App;
use ratatui::{prelude::*, widgets::*};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());
    let titles: Vec<Line<'_>> = app
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
        0 => draw_live_capture_tab(f, app, chunks[1]),
        _ => {}
    };
    // Draw footer
    let footer = format!(
        "Press <Q> quit | <SPACE> pause | <Up>/<Down> select | <B> jump latest"
    );
    let footer = Paragraph::new(text::Line::from(Span::styled(
        footer,
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(footer, chunks[2]);
}

fn packet_info(packet: &crate::net::packet::PacketFrame) -> String {
    if let Some(transport) = &packet.transport {
        if let Some(tcp) = &transport.tcp {
            let flags = tcp.flags;
            return format!("TCP flags=0x{flags:02x} seq={} ack={}", tcp.sequence, tcp.acknowledgement);
        }
        if let Some(udp) = &transport.udp {
            return format!("UDP {} -> {}", udp.source, udp.destination);
        }
    }
    if let Some(ip) = &packet.ip {
        if ip.icmp.is_some() || ip.icmpv6.is_some() {
            return "ICMP".to_string();
        }
        if let Some(ipv4) = &ip.ipv4 {
            return format!("IPv4 ttl={} id={}", ipv4.ttl, ipv4.identification);
        }
        if let Some(ipv6) = &ip.ipv6 {
            return format!("IPv6 hop_limit={}", ipv6.hop_limit);
        }
    }
    if let Some(datalink) = &packet.datalink {
        if datalink.arp.is_some() {
            return "ARP".to_string();
        }
    }
    String::new()
}

fn payload_hex_preview(payload: &[u8], max_len: usize) -> String {
    if payload.is_empty() {
        return String::from("-");
    }
    let take_len = payload.len().min(max_len);
    let mut out = String::new();
    for (idx, b) in payload.iter().take(take_len).enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        out.push_str(&format!("{b:02x}"));
    }
    if payload.len() > max_len {
        out.push_str(" ...");
    }
    out
}

fn selected_packet<'a>(app: &'a App) -> Option<&'a crate::net::packet::PacketFrame> {
    app.talbe_state
        .selected()
        .and_then(|idx| app.packets.get(idx))
        .or_else(|| app.packets.last())
}

fn draw_packet_table(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = app
        .packets
        .iter()
        .map(|packet| {
            Row::new(vec![
                packet.capture_no.to_string(),
                packet.get_time(),
                packet.get_src_addr(),
                packet.get_dst_addr(),
                packet.get_protocol(),
                packet.packet_len.to_string(),
                packet_info(packet),
            ])
        })
        .collect::<Vec<Row>>();
    let widths = [
        Constraint::Length(6),
        Constraint::Length(14),
        Constraint::Length(24),
        Constraint::Length(24),
        Constraint::Length(7),
        Constraint::Length(8),
        Constraint::Min(20),
    ];

    let table_title: String;
    if app.config.network.interfaces.is_empty() {
        table_title = "Capturing from all available interfaces".to_string();
    } else {
        table_title = format!(
            "Capturing from {}",
            app.config.network.interfaces.join(", ")
        );
    }

    //let mut table_state = TableState::default();
    let table = Table::new(rows, widths)
        .column_spacing(1)
        .header(
            Row::new(vec![
                "No.",
                "Time",
                "Source",
                "Destination",
                "Proto",
                "Len",
                "Info",
            ])
            .style(Style::new().bold()), //.bottom_margin(1),
        )
        .block(Block::default().borders(Borders::ALL).title(table_title))
        .row_highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    //f.render_widget(table, area);
    f.render_stateful_widget(table, area, &mut app.talbe_state);
}

fn draw_packet_detail(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line<'_>> = Vec::new();
    if let Some(packet) = selected_packet(app) {
        lines.push(Line::raw(format!(
            "No={} Iface={} Time={}",
            packet.capture_no, packet.if_name, packet.timestamp
        )));
        lines.push(Line::raw(format!(
            "Source={}  Destination={}",
            packet.get_src_addr(),
            packet.get_dst_addr()
        )));
        lines.push(Line::raw(format!(
            "Proto={} Len={} {}",
            packet.get_protocol(),
            packet.packet_len,
            packet_info(packet)
        )));

        if let Some(ip) = &packet.ip {
            if let Some(ipv4) = &ip.ipv4 {
                lines.push(Line::raw(format!(
                    "IPv4: {} -> {} ttl={} id={} proto={}",
                    ipv4.source,
                    ipv4.destination,
                    ipv4.ttl,
                    ipv4.identification,
                    ipv4.next_level_protocol.as_str()
                )));
            }
            if let Some(ipv6) = &ip.ipv6 {
                lines.push(Line::raw(format!(
                    "IPv6: {} -> {} hop_limit={} next={}",
                    ipv6.source,
                    ipv6.destination,
                    ipv6.hop_limit,
                    ipv6.next_header.as_str()
                )));
            }
        }

        if let Some(transport) = &packet.transport {
            if let Some(tcp) = &transport.tcp {
                lines.push(Line::raw(format!(
                    "TCP: {} -> {} flags=0x{:02x} seq={} ack={} win={}",
                    tcp.source,
                    tcp.destination,
                    tcp.flags,
                    tcp.sequence,
                    tcp.acknowledgement,
                    tcp.window
                )));
            }
            if let Some(udp) = &transport.udp {
                lines.push(Line::raw(format!(
                    "UDP: {} -> {} len={} checksum=0x{:04x}",
                    udp.source,
                    udp.destination,
                    udp.length,
                    udp.checksum
                )));
            }
        }

        if let Some(datalink) = &packet.datalink {
            if let Some(eth) = &datalink.ethernet {
                lines.push(Line::raw(format!(
                    "Ethernet: {} -> {} type={}",
                    eth.source,
                    eth.destination,
                    eth.ethertype.name()
                )));
            }
            if let Some(arp) = &datalink.arp {
                lines.push(Line::raw(format!(
                    "ARP: {} -> {}",
                    arp.sender_proto_addr, arp.target_proto_addr
                )));
            }
        }

        lines.push(Line::raw(format!(
            "Payload({}B): {}",
            packet.payload.len(),
            payload_hex_preview(packet.payload.as_ref(), 48)
        )));
    } else {
        lines.push(Line::raw("No packets captured yet."));
    }

    let detail = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Packet Details"))
        .wrap(Wrap { trim: true });
    f.render_widget(detail, area);
}

fn draw_live_capture_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(area);
    draw_packet_table(f, app, chunks[0]);
    draw_packet_detail(f, app, chunks[1]);
}
