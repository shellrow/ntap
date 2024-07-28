use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use crate::{config::AppConfig, net::stat::NetStatStrage};
use crate::{sys, tui::stat::app::App, tui::stat::ui};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::sync::Arc;

pub fn run(
    app_config: AppConfig,
    enhanced_graphics: bool,
    netstat_strage: &mut Arc<NetStatStrage>,
) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let title = sys::get_app_title();
    let app = App::new(&title, enhanced_graphics, app_config);
    let res = run_app(&mut terminal, app, netstat_strage);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    netstat_strage: &mut Arc<NetStatStrage>,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(app.config.display.tick_rate);
    let entry_ttl = Duration::from_millis(app.config.network.entry_ttl);
    let mut last_tick = Instant::now();
    let mut last_clear = Instant::now();
    loop {
        if last_clear.elapsed() >= entry_ttl {
            app.netstat_data.remove_old_entries(entry_ttl);
            last_clear = Instant::now();
        }

        if last_tick.elapsed() >= tick_rate {
            if !app.should_pause {
                app.on_tick(netstat_strage.clone_data_and_reset());
            }
            last_tick = Instant::now();
        }

        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Left | KeyCode::Char('a') => app.on_left(),
                        KeyCode::Up | KeyCode::Char('w') => app.on_up(),
                        KeyCode::Right | KeyCode::Char('d') => app.on_right(),
                        KeyCode::Down | KeyCode::Char('s') => app.on_down(),
                        KeyCode::Tab => app.on_tab(),
                        KeyCode::BackTab => app.on_shift_tab(),
                        KeyCode::Char(c) => app.on_key(c),
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
