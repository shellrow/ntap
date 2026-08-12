use crate::config::AppConfig;
use crate::{net::packet::PacketStorage, sys, tui::terminal::TerminalGuard};
use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::prelude::*;
use std::{io, sync::Arc, time::Duration};
use tokio::time::{self, Instant};

use super::app::App;
use super::ui;

pub async fn run(app_config: AppConfig, packet_storage: &Arc<PacketStorage>) -> Result<()> {
    let _guard = TerminalGuard::enter()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let title = sys::get_app_title();
    let mut app = App::new(&title, app_config);
    let result = run_app(&mut terminal, &mut app, packet_storage).await;

    result.map_err(Into::into)
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App<'_>,
    packet_storage: &Arc<PacketStorage>,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(app.config.display.tick_rate);
    let mut tick = time::interval(tick_rate);
    let mut events = EventStream::new();
    let mut last_draw = Instant::now();

    loop {
        tokio::select! {
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
                            KeyCode::Up | KeyCode::Char('w') => app.on_up(),
                            KeyCode::Down | KeyCode::Char('s') => app.on_down(),
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

        if last_draw.elapsed() >= Duration::from_millis(16) {
            terminal
                .draw(|f| ui::draw(f, app))
                .map_err(|e| io::Error::other(e.to_string()))?;
            last_draw = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
