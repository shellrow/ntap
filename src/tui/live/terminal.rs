use crate::config::AppConfig;
use crate::{net::packet::PacketStorage, sys};
use anyhow::Result;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::prelude::*;
use std::{io, sync::Arc, time::Duration};
use tokio::time::{self, Instant};

use super::app::App;
use super::ui;

pub async fn run(
    app_config: AppConfig,
    enhanced_graphics: bool,
    packet_strage: &Arc<PacketStorage>,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let title = sys::get_app_title();
    let mut app = App::new(&title, enhanced_graphics, app_config);
    let result = run_app(&mut terminal, &mut app, packet_strage).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(result?)
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App<'_>,
    packet_strage: &Arc<PacketStorage>,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(app.config.display.tick_rate);
    let mut tick = time::interval(tick_rate);
    let mut events = EventStream::new();
    let mut last_draw = Instant::now();

    loop {
        tokio::select! {
            _ = tick.tick() => {
                if !app.should_pause {
                    app.on_tick(packet_strage.get_packets());
                }
            }
            maybe_event = events.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Up | KeyCode::Char('w') => app.on_up(),
                            KeyCode::Down | KeyCode::Char('s') => app.on_down(),
                            KeyCode::Char(c) => app.on_key(c),
                            _ => {}
                        }
                    }
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
