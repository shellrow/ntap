use crate::{
    config::AppConfig, net::stat::NetStatStrage, sys, tui::monitor::app::App, tui::monitor::ui,
};
use anyhow::Result;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::prelude::*;
use std::{io, sync::Arc, time::Duration};
use tokio::time::{self, Instant};

pub async fn run(
    app_config: AppConfig,
    _enhanced_graphics: bool,
    netstat_strage: &mut Arc<NetStatStrage>,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let title = sys::get_app_title();
    let mut app = App::new(title, app_config);
    let result = run_app(&mut terminal, &mut app, netstat_strage).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(result?)
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    netstat_strage: &mut Arc<NetStatStrage>,
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
                if !app.should_pause {
                    app.on_tick(netstat_strage.clone_data_and_reset());
                }
            }
            _ = clear_tick.tick() => {
                app.netstat_data.remove_old_entries(entry_ttl);
            }
            maybe_event = events.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Tab | KeyCode::Right => app.switch_next_tab(),
                            KeyCode::BackTab | KeyCode::Left => app.switch_prev_tab(),
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
