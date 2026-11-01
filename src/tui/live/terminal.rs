use crate::config::AppConfig;
use crate::{net::packet::PacketStorage, sys, tui::terminal::TerminalGuard};
use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::{Stream, StreamExt};
use ratatui::prelude::*;
use std::future::Future;
use std::{io, sync::Arc, time::Duration};
use tokio::time::{self, Instant};

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
        &mut app,
        packet_storage,
        &capture_status,
        dropped_packets,
        EventStream::new(),
        crate::tui::terminal::shutdown_signal(),
        |app| {
            terminal
                .draw(|frame| ui::draw(frame, app))
                .map(|_| ())
                .map_err(|error| io::Error::other(error.to_string()))
        },
    )
    .await;

    result.map_err(Into::into)
}

// Round up so even an ideal scheduler cannot exceed 60 frames per second.
const FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000u64.div_ceil(60));

async fn run_app(
    app: &mut App<'_>,
    packet_storage: &PacketStorage,
    capture_status: &std::sync::mpsc::Receiver<Result<(), String>>,
    dropped_packets: &std::sync::atomic::AtomicUsize,
    mut events: impl Stream<Item = io::Result<Event>> + Unpin,
    shutdown: impl Future<Output = io::Result<()>>,
    mut draw: impl FnMut(&mut App<'_>) -> io::Result<()>,
) -> io::Result<()> {
    let mut redraw = true;
    let mut packets_changed = true;
    let mut next_draw = Instant::now();
    let mut health_tick = time::interval(Duration::from_millis(200));
    health_tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            result = &mut shutdown => return result,
            _ = health_tick.tick() => {
                for status in capture_status.try_iter() {
                    status.map_err(io::Error::other)?;
                }
                let dropped = dropped_packets.load(std::sync::atomic::Ordering::Relaxed);
                if app.dropped_packets != dropped {
                    app.dropped_packets = dropped;
                    redraw = true;
                }
            }
            // Once dirty, stop consuming arrival notifications until the next
            // snapshot. Bursts therefore cannot turn into per-packet UI work.
            _ = packet_storage.updated(), if !packets_changed && !app.should_pause => {
                packets_changed = true;
                redraw = true;
            }
            _ = time::sleep_until(next_draw), if redraw => {
                if packets_changed && !app.should_pause {
                    app.on_tick(packet_storage.get_packets());
                }
                draw(app)?;
                next_draw = Instant::now() + FRAME_INTERVAL;
                redraw = false;
                packets_changed = false;
            }
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                        let was_paused = app.should_pause;
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.should_quit = true,
                            KeyCode::Up | KeyCode::Char('w' | 'W') => app.on_up(),
                            KeyCode::Down | KeyCode::Char('s' | 'S') => app.on_down(),
                            KeyCode::Char(c @ ('q' | 'Q' | ' ' | 'b' | 'B')) => app.on_key(c),
                            _ => continue,
                        }
                        if app.should_quit { return Ok(()); }
                        if was_paused && !app.should_pause {
                            // Resume from current storage even if no further packet arrives.
                            packets_changed = true;
                        }
                        redraw = true;
                    }
                    Some(Ok(Event::Resize(..))) => redraw = true,
                    Some(Ok(_)) => {}
                    Some(Err(error)) => return Err(error),
                    None => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "terminal event stream closed")),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::packet::PacketFrame;
    use crossterm::event::KeyEvent;
    use futures::channel::mpsc::unbounded;
    use std::sync::atomic::AtomicUsize;

    fn key(value: char) -> io::Result<Event> {
        Ok(Event::Key(KeyEvent::new(
            KeyCode::Char(value),
            KeyModifiers::NONE,
        )))
    }

    fn packet(capture_no: usize) -> PacketFrame {
        PacketFrame {
            capture_no,
            ..Default::default()
        }
    }

    #[tokio::test(start_paused = true)]
    async fn arrivals_and_resize_bursts_are_coalesced_under_sixty_fps() {
        let storage = PacketStorage::with_capacity(200);
        let mut config = AppConfig::default();
        config.display.tick_rate = 60_000;
        let mut app = App::new("test", config);
        let (_status_tx, status_rx) = std::sync::mpsc::channel();
        let dropped = AtomicUsize::new(0);
        let (events_tx, events_rx) = unbounded();
        let mut frames = Vec::new();
        let started = Instant::now();
        let ui = run_app(
            &mut app,
            &storage,
            &status_rx,
            &dropped,
            events_rx,
            std::future::pending(),
            |app| {
                frames.push((Instant::now(), app.packets.len()));
                Ok(())
            },
        );
        let traffic = async {
            for id in 1..=100 {
                time::sleep(Duration::from_millis(1)).await;
                storage.add_packet(packet(id));
                events_tx
                    .unbounded_send(Ok(Event::Resize(120, 30)))
                    .unwrap();
            }
            // The trailing packet must render even after traffic stops.
            time::sleep(Duration::from_millis(50)).await;
            events_tx.unbounded_send(key('q')).unwrap();
        };
        let (result, ()) = tokio::join!(ui, traffic);
        result.unwrap();
        assert_eq!(frames.first().unwrap().1, 0);
        assert_eq!(frames.last().unwrap().1, 100);
        assert!(frames.len() < 15, "packet bursts must be batched");
        for pair in frames.windows(2) {
            assert!(pair[1].0.duration_since(pair[0].0) >= FRAME_INTERVAL);
        }
        let first_packet = frames.iter().find(|(_, count)| *count > 0).unwrap();
        assert!(first_packet.0.duration_since(started) < Duration::from_millis(50));
    }

    #[tokio::test(start_paused = true)]
    async fn idle_ui_does_not_redraw_and_isolated_arrival_wakes_it() {
        let storage = PacketStorage::with_capacity(10);
        let mut app = App::new("test", AppConfig::default());
        let (_status_tx, status_rx) = std::sync::mpsc::channel();
        let dropped = AtomicUsize::new(0);
        let (events_tx, events_rx) = unbounded();
        let mut frames = Vec::new();
        let started = Instant::now();
        let ui = run_app(
            &mut app,
            &storage,
            &status_rx,
            &dropped,
            events_rx,
            std::future::pending(),
            |app| {
                frames.push((Instant::now(), app.packets.len()));
                Ok(())
            },
        );
        let traffic = async {
            time::sleep(Duration::from_millis(1250)).await;
            storage.add_packet(packet(1));
            time::sleep(Duration::from_secs(2)).await;
            events_tx.unbounded_send(key('q')).unwrap();
        };
        let (result, ()) = tokio::join!(ui, traffic);
        result.unwrap();
        assert_eq!(
            frames.len(),
            2,
            "health checks must not redraw idle screens"
        );
        assert_eq!(frames[1].1, 1);
        let latency = frames[1]
            .0
            .duration_since(started + Duration::from_millis(1250));
        assert!(latency <= FRAME_INTERVAL);
    }

    #[tokio::test(start_paused = true)]
    async fn pause_freezes_packets_and_resume_refreshes_without_another_arrival() {
        let storage = PacketStorage::with_capacity(10);
        storage.add_packet(packet(1));
        let mut app = App::new("test", AppConfig::default());
        let (_status_tx, status_rx) = std::sync::mpsc::channel();
        let dropped = AtomicUsize::new(0);
        let (events_tx, events_rx) = unbounded();
        let mut frames = Vec::new();
        let ui = run_app(
            &mut app,
            &storage,
            &status_rx,
            &dropped,
            events_rx,
            std::future::pending(),
            |app| {
                frames.push((app.should_pause, app.packets.len()));
                Ok(())
            },
        );
        let traffic = async {
            time::sleep(Duration::from_millis(50)).await;
            events_tx.unbounded_send(key(' ')).unwrap();
            time::sleep(Duration::from_millis(50)).await;
            storage.add_packet(packet(2));
            time::sleep(Duration::from_millis(100)).await;
            events_tx.unbounded_send(key(' ')).unwrap();
            time::sleep(Duration::from_millis(50)).await;
            events_tx.unbounded_send(key('q')).unwrap();
        };
        let (result, ()) = tokio::join!(ui, traffic);
        result.unwrap();
        assert_eq!(frames, [(false, 1), (true, 1), (false, 2)]);
    }

    #[tokio::test(start_paused = true)]
    async fn quit_does_not_wait_for_a_pending_frame() {
        let storage = PacketStorage::with_capacity(10);
        let mut app = App::new("test", AppConfig::default());
        let (_status_tx, status_rx) = std::sync::mpsc::channel();
        let dropped = AtomicUsize::new(0);
        let (events_tx, events_rx) = unbounded();
        let mut draws = 0;
        let started = Instant::now();
        let ui = run_app(
            &mut app,
            &storage,
            &status_rx,
            &dropped,
            events_rx,
            std::future::pending(),
            |_| {
                draws += 1;
                Ok(())
            },
        );
        let traffic = async {
            time::sleep(Duration::from_millis(1)).await;
            storage.add_packet(packet(1));
            events_tx.unbounded_send(key('q')).unwrap();
        };
        let (result, ()) = tokio::join!(ui, traffic);
        result.unwrap();
        assert_eq!(draws, 1);
        assert!(started.elapsed() < FRAME_INTERVAL);
    }
}
