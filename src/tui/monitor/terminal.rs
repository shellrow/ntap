use crate::{
    config::AppConfig, net::stat::NetStatStorage, sys, tui::monitor::app::App, tui::monitor::ui,
    tui::terminal::TerminalGuard,
};
use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::prelude::*;
use std::{io, sync::Arc, time::Duration};
use tokio::time::{self, Instant};

pub async fn run(app_config: AppConfig, netstat_storage: &mut Arc<NetStatStorage>) -> Result<()> {
    let _guard = TerminalGuard::enter()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let title = sys::get_app_title();
    let mut app = App::new(title, app_config);
    let result = run_app(&mut terminal, &mut app, netstat_storage).await;

    result.map_err(Into::into)
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    netstat_storage: &mut Arc<NetStatStorage>,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(app.config.display.tick_rate);
    let entry_ttl = Duration::from_millis(app.config.network.entry_ttl);
    let mut tick = time::interval(tick_rate);
    let mut clear_tick = time::interval(entry_ttl);
    let mut events = EventStream::new();
    let mut last_draw = Instant::now();

    loop {
        tokio::select! {
            _ = tick.tick() => {
                app.on_tick(netstat_storage.clone_data_and_reset());
            }
            _ = clear_tick.tick() => {
                app.netstat_data.remove_old_entries(entry_ttl);
            }
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.should_quit = true,
                            KeyCode::Tab | KeyCode::Right => app.switch_next_tab(),
                            KeyCode::BackTab | KeyCode::Left => app.switch_prev_tab(),
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
