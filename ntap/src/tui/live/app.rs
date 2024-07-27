#![allow(unused)]

use std::time::Duration;

use crate::{
    config::AppConfig,
    net::{
        host::HostDisplayInfo,
        packet::{PacketFrame, PacketStorage},
        service::ServiceDisplayInfo,
        socket::SocketDisplayInfo,
        stat::NetStatData,
    },
    process::ProcessDisplayInfo,
};
use ratatui::widgets::TableState;

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_pause: bool,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub talbe_state: TableState,
    pub row_selecting: bool,
    pub packets: Vec<PacketFrame>,
    pub enhanced_graphics: bool,
    pub config: AppConfig,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool, config: AppConfig) -> App<'a> {
        App {
            title,
            should_pause: false,
            should_quit: false,
            tabs: TabsState::new(vec!["PacketCapture"]),
            talbe_state: TableState::default(),
            row_selecting: false,
            packets: Vec::new(),
            enhanced_graphics: enhanced_graphics,
            config: config,
        }
    }

    pub fn on_up(&mut self) {
        // Select the previous row
        self.row_selecting = true;
        let row_count = self.packets.len();
        let i = match self.talbe_state.selected() {
            Some(i) => {
                if i == 0 {
                    row_count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.talbe_state.select(Some(i));
    }

    pub fn on_down(&mut self) {
        // Select the next row
        self.row_selecting = true;
        let row_count = self.packets.len();
        let i = match self.talbe_state.selected() {
            Some(i) => {
                if i >= row_count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.talbe_state.select(Some(i));
    }

    pub fn on_right(&mut self) {
        // Select the next tab
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        // Select the previous tab
        self.tabs.previous();
    }

    pub fn on_tab(&mut self) {
        // Select the next tab
        self.tabs.next();
    }

    pub fn on_shift_tab(&mut self) {
        // Select the previous tab
        self.tabs.previous();
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
                self.talbe_state.select(Some(self.packets.len() - 1));
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
        if !self.row_selecting {
            self.talbe_state.select(Some(self.packets.len() - 1));
        }
    }
}
