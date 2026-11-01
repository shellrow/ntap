use crate::{config::AppConfig, net::packet::PacketFrame};
use ratatui::widgets::TableState;

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState<'a> {
        TabsState { titles, index: 0 }
    }
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_pause: bool,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub table_state: TableState,
    pub row_selecting: bool,
    pub packets: Vec<PacketFrame>,
    pub config: AppConfig,
    pub dropped_packets: usize,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, config: AppConfig) -> App<'a> {
        App {
            title,
            should_pause: false,
            should_quit: false,
            tabs: TabsState::new(vec!["PacketCapture"]),
            table_state: TableState::default(),
            row_selecting: false,
            packets: Vec::new(),
            config,
            dropped_packets: 0,
        }
    }

    pub fn on_up(&mut self) {
        // Select the previous row
        self.row_selecting = true;
        let row_count = self.packets.len();
        if row_count == 0 {
            self.table_state.select(None);
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    row_count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn on_down(&mut self) {
        // Select the next row
        self.row_selecting = true;
        let row_count = self.packets.len();
        if row_count == 0 {
            self.table_state.select(None);
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= row_count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn on_key(&mut self, c: char) {
        match c.to_ascii_lowercase() {
            'q' => {
                // Quit the application
                self.should_quit = true;
            }
            ' ' => {
                // Pause the application
                self.should_pause = !self.should_pause;
            }
            'b' => {
                // Scroll to the bottom
                if self.packets.is_empty() {
                    self.table_state.select(None);
                } else {
                    self.table_state.select(Some(self.packets.len() - 1));
                }
                self.row_selecting = false;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self, packets: Vec<PacketFrame>) {
        let selected = self
            .table_state
            .selected()
            .and_then(|index| self.packets.get(index))
            .map(|packet| packet.capture_no);
        self.packets = packets;
        let index = if self.packets.is_empty() {
            None
        } else if self.row_selecting {
            Some(
                selected
                    .and_then(|id| self.packets.iter().position(|p| p.capture_no == id))
                    .unwrap_or(0),
            )
        } else {
            Some(self.packets.len() - 1)
        };
        self.table_state.select(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn packets(ids: &[usize]) -> Vec<PacketFrame> {
        ids.iter()
            .map(|&capture_no| PacketFrame {
                capture_no,
                ..Default::default()
            })
            .collect()
    }
    #[test]
    fn selection_tracks_identity_and_clamps_after_eviction() {
        let mut app = App::new("test", AppConfig::default());
        app.on_tick(packets(&[1, 2, 3]));
        app.on_up();
        app.on_tick(packets(&[2, 3, 4]));
        assert_eq!(app.table_state.selected(), Some(0));
        assert_eq!(app.packets[0].capture_no, 2);
        app.on_tick(packets(&[3, 4, 5]));
        assert_eq!(app.table_state.selected(), Some(0));
        app.on_tick(vec![]);
        assert_eq!(app.table_state.selected(), None);
    }
    #[test]
    fn follow_latest_and_uppercase_controls_work() {
        let mut app = App::new("test", AppConfig::default());
        app.on_tick(packets(&[1, 2]));
        assert_eq!(app.table_state.selected(), Some(1));
        app.on_up();
        app.on_key('B');
        app.on_tick(packets(&[2, 3, 4]));
        assert_eq!(app.table_state.selected(), Some(2));
        app.on_key('Q');
        assert!(app.should_quit);
    }
}
