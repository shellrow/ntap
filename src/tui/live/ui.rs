use super::app::App;
use nex::packet::dns::DnsPacket;
use nex::packet::tcp::TcpFlags;
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
    if app.tabs.index == 0 {
        draw_live_capture_tab(f, app, chunks[1]);
    };
    // Draw footer
    let footer = "Press <Q> quit | <SPACE> pause | <Up>/<Down> select | <B> jump latest";
    let footer = Paragraph::new(text::Line::from(Span::styled(
        footer,
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(footer, chunks[2]);
}

fn packet_info(packet: &crate::net::packet::PacketFrame) -> String {
    let app_hint = packet_app_hint(packet);
    if let Some(transport) = &packet.transport {
        if let Some(tcp) = &transport.tcp {
            let flags = tcp.flags;
            let has = |bit: u8| flags & bit == bit;
            let mut flag_text = String::new();
            if has(TcpFlags::SYN) {
                flag_text.push('S');
            }
            if has(TcpFlags::FIN) {
                flag_text.push('F');
            }
            if has(TcpFlags::PSH) {
                flag_text.push('P');
            }
            if has(TcpFlags::RST) {
                flag_text.push('R');
            }
            if has(TcpFlags::URG) {
                flag_text.push('U');
            }
            if has(TcpFlags::ECE) {
                flag_text.push('E');
            }
            if has(TcpFlags::CWR) {
                flag_text.push('W');
            }
            if has(TcpFlags::ACK) {
                flag_text.push('.');
            }
            if flag_text.is_empty() {
                flag_text.push('.');
            }

            let payload_len = packet.payload.len() as u32;
            let seq_end = tcp.sequence.saturating_add(payload_len);
            if payload_len > 0 {
                let base = format!(
                    "Flags [{}], seq {}:{}, ack {}, win {}, length {}",
                    flag_text, tcp.sequence, seq_end, tcp.acknowledgement, tcp.window, payload_len
                );
                return if app_hint.is_empty() {
                    base
                } else {
                    format!("{base} | {app_hint}")
                };
            }
            let base = format!(
                "Flags [{}], seq {}, ack {}, win {}, length 0",
                flag_text, tcp.sequence, tcp.acknowledgement, tcp.window
            );
            return if app_hint.is_empty() {
                base
            } else {
                format!("{base} | {app_hint}")
            };
        }
        if let Some(udp) = &transport.udp {
            let base = format!(
                "{} > {}: UDP, length {}",
                udp.source,
                udp.destination,
                packet.payload.len()
            );
            return if app_hint.is_empty() {
                base
            } else {
                format!("{base} | {app_hint}")
            };
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
    if let Some(datalink) = &packet.datalink
        && datalink.arp.is_some()
    {
        return "ARP".to_string();
    }
    String::new()
}

fn is_tls_client_hello(payload: &[u8]) -> bool {
    // TLS record: handshake(0x16), then client_hello(0x01)
    payload.len() > 5 && payload[0] == 0x16 && payload[5] == 0x01
}

fn first_http_line(payload: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(payload).ok()?;
    let first = s.lines().next()?.trim();
    if first.starts_with("GET ")
        || first.starts_with("POST ")
        || first.starts_with("PUT ")
        || first.starts_with("DELETE ")
        || first.starts_with("HEAD ")
        || first.starts_with("PATCH ")
        || first.starts_with("OPTIONS ")
        || first.starts_with("HTTP/")
    {
        Some(first.to_string())
    } else {
        None
    }
}

fn http_host(payload: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(payload).ok()?;
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("Host:") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn dns_hint(payload: bytes::Bytes) -> Option<String> {
    let dns = DnsPacket::try_from_buf(&payload).ok()?;
    if let Some(query) = dns.queries.first()
        && let Ok(name) = query.qname_parsed()
    {
        return Some(format!("DNS query {}", name));
    }
    if !dns.responses.is_empty() {
        return Some(format!("DNS response {} record(s)", dns.responses.len()));
    }
    Some("DNS".to_string())
}

fn packet_app_hint(packet: &crate::net::packet::PacketFrame) -> String {
    if let Some(transport) = &packet.transport {
        if let Some(udp) = &transport.udp
            && (udp.source == 53 || udp.destination == 53)
            && let Some(hint) = dns_hint(packet.payload.clone())
        {
            return hint;
        }
        if let Some(tcp) = &transport.tcp {
            let payload = packet.payload.as_ref();
            if payload.is_empty() {
                return String::new();
            }
            if (tcp.source == 53 || tcp.destination == 53)
                && let Some(hint) = dns_hint(packet.payload.clone())
            {
                return hint;
            }
            if (tcp.source == 443 || tcp.destination == 443) && is_tls_client_hello(payload) {
                return "TLS ClientHello".to_string();
            }
            if (tcp.source == 80
                || tcp.destination == 80
                || tcp.source == 8080
                || tcp.destination == 8080)
                && let Some(line) = first_http_line(payload)
            {
                if let Some(host) = http_host(payload) {
                    return format!("HTTP {} host={}", line, host);
                }
                return format!("HTTP {}", line);
            }
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
    app.table_state
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

    let table_title: String = if app.config.network.interfaces.is_empty() {
        "Capturing from all available interfaces".to_string()
    } else {
        format!(
            "Capturing from {}",
            app.config.network.interfaces.join(", ")
        )
    };

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
    f.render_stateful_widget(table, area, &mut app.table_state);
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
                    udp.source, udp.destination, udp.length, udp.checksum
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Packet Details"),
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tls_client_hello_detector_works() {
        let payload = [0x16, 0x03, 0x03, 0x00, 0x20, 0x01, 0x00];
        assert!(is_tls_client_hello(&payload));
        assert!(!is_tls_client_hello(&[]));
        assert!(!is_tls_client_hello(&[0x17, 0x03, 0x03, 0x00, 0x20, 0x01]));
    }

    #[test]
    fn http_first_line_and_host_parse() {
        let req = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: x\r\n\r\n";
        assert_eq!(
            first_http_line(req).as_deref(),
            Some("GET /index.html HTTP/1.1")
        );
        assert_eq!(http_host(req).as_deref(), Some("example.com"));

        let res = b"HTTP/1.1 200 OK\r\nServer: test\r\n\r\n";
        assert_eq!(first_http_line(res).as_deref(), Some("HTTP/1.1 200 OK"));
        assert_eq!(http_host(res), None);
    }

    #[test]
    fn payload_hex_preview_formats_and_truncates() {
        assert_eq!(payload_hex_preview(&[], 8), "-");
        assert_eq!(payload_hex_preview(&[0x01, 0xab, 0xff], 8), "01 ab ff");
        assert_eq!(
            payload_hex_preview(&[0xde, 0xad, 0xbe, 0xef, 0x01], 4),
            "de ad be ef ..."
        );
    }
}
