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
        match c {
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
        // Update the state of the application
        // Set the latest packets
        self.packets = packets;
        // If the user is not selecting a row, scroll to the bottom
        if !self.row_selecting && !self.packets.is_empty() {
            self.table_state.select(Some(self.packets.len() - 1));
        } else if self.packets.is_empty() {
            self.table_state.select(None);
        }
    }
}
