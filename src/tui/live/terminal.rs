use crate::config::AppConfig;
use crate::{net::packet::PacketStorage, sys, tui::terminal::TerminalGuard};
use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::prelude::*;
use std::{io, sync::Arc, time::Duration};
use tokio::time;

use super::app::App;
use super::ui;

pub async fn run(
    app_config: AppConfig,
    packet_storage: &Arc<PacketStorage>,
    capture_status: std::sync::mpsc::Receiver<Result<(), String>>,
    dropped_packets: &std::sync::atomic::AtomicUsize,
) -> Result<()> {
    let _guard = TerminalGuard::enter()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let title = sys::get_app_title();
    let mut app = App::new(&title, app_config);
    let result = run_app(
        &mut terminal,
        &mut app,
        packet_storage,
        &capture_status,
        dropped_packets,
    )
    .await;

    result.map_err(Into::into)
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App<'_>,
    packet_storage: &Arc<PacketStorage>,
    capture_status: &std::sync::mpsc::Receiver<Result<(), String>>,
    dropped_packets: &std::sync::atomic::AtomicUsize,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(app.config.display.tick_rate);
    let mut tick = time::interval(tick_rate);
    let mut events = EventStream::new();
    tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    let mut health_tick = time::interval(Duration::from_millis(200));
    let shutdown = crate::tui::terminal::shutdown_signal();
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            result = &mut shutdown => { result?; return Ok(()); }

            _ = health_tick.tick() => {
                for status in capture_status.try_iter() {
                    status.map_err(io::Error::other)?;
                }
                let dropped = dropped_packets.load(std::sync::atomic::Ordering::Relaxed);
                if app.dropped_packets == dropped { continue; }
                app.dropped_packets = dropped;
            }

            _ = tick.tick() => {
                if !app.should_pause {
                    app.on_tick(packet_storage.get_packets());
                }
            }
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.should_quit = true,
                            KeyCode::Up | KeyCode::Char('w' | 'W') => app.on_up(),
                            KeyCode::Down | KeyCode::Char('s' | 'S') => app.on_down(),
                            KeyCode::Char(c) => app.on_key(c),
                            _ => {}
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(error)) => return Err(error),
                    None => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "terminal event stream closed")),
                }
            }
        }

        {
            terminal
                .draw(|f| ui::draw(f, app))
                .map_err(|e| io::Error::other(e.to_string()))?;
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
