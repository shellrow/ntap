use ratatui::{prelude::*, widgets::*};
use super::app::App;

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
        0 => draw_live_capture_tab(f, app, chunks[1]),
        _ => {}
    };
    // Draw footer
    let footer = format!("Press <Q> to quit, <SPACE> to pause, <Up>/<Down> to scroll, <B> to scroll to the bottom");
    let footer = Paragraph::new(text::Line::from(Span::styled(
        footer,
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(footer, chunks[2]);
}

fn draw_packet_table(f: &mut Frame, app: &mut App, area: Rect) {
    // Draw top Remote Address Table
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
                packet.get_src_port(),
                packet.get_dst_port(),
                packet.if_name.clone(),
            ])
        })
        .collect::<Vec<Row>>();
    let widths = [
        Constraint::Length(6),
        Constraint::Length(16),
        Constraint::Length(40),
        Constraint::Length(40),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(10),
    ];

    //let mut table_state = TableState::default();
    let table = Table::new(rows, widths)
        .column_spacing(1)
        //.style(Style::new().blue())
        .header(
            Row::new(vec![
                "No.",
                "Timestamp",
                "SRC Address",
                "DST Address",
                "Protocol",
                "Length",
                "SRC Port",
                "DST Port",
                "Interface",
            ])
            .style(Style::new().bold()), //.bottom_margin(1),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Packets"),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    //f.render_widget(table, area);
    f.render_stateful_widget(table, area, &mut app.talbe_state);
}

fn draw_live_capture_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(100)])
        .split(area);
    draw_packet_table(f, app, chunks[0]);
}
