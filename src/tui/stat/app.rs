#![allow(unused)]

use std::time::Duration;

use crate::{
    config::AppConfig,
    net::{
        host::HostDisplayInfo, service::ServiceDisplayInfo, socket::SocketDisplayInfo,
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
    pub fn new(titles: Vec<&'a str>) -> TabsState<'a> {
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
    pub netstat_data: NetStatData,
    pub remote_hosts: Vec<HostDisplayInfo>,
    pub processes: Vec<ProcessDisplayInfo>,
    pub connections: Vec<SocketDisplayInfo>,
    pub app_protocols: Vec<ServiceDisplayInfo>,
    pub enhanced_graphics: bool,
    pub config: AppConfig,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool, config: AppConfig) -> App<'a> {
        App {
            title,
            should_pause: false,
            should_quit: false,
            tabs: TabsState::new(vec!["Network Statistics"]),
            talbe_state: TableState::default(),
            netstat_data: NetStatData::new(),
            remote_hosts: vec![],
            processes: vec![],
            connections: vec![],
            app_protocols: vec![],
            enhanced_graphics: enhanced_graphics,
            config: config,
        }
    }

    pub fn on_up(&mut self) {
        if self.tabs.index == 0 {
            return;
        }
        // Select the previous row
        let row_count = match self.tabs.index {
            1 => self.remote_hosts.len(),
            2 => self.connections.len(),
            _ => 0,
        };
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
        if self.tabs.index == 0 {
            return;
        }
        // Select the next row
        let row_count = match self.tabs.index {
            1 => self.remote_hosts.len(),
            2 => self.connections.len(),
            _ => 0,
        };
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
            't' => {
                // Switch display mode (total/bandwidth)
                self.config.display.show_bandwidth = !self.config.display.show_bandwidth;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self, netstat_data: NetStatData) {
        // Update the state of the application
        self.netstat_data.merge(
            netstat_data,
            Duration::from_millis(self.config.display.tick_rate),
        );
        self.remote_hosts = self.netstat_data.get_remote_hosts(None);
        self.processes = self.netstat_data.get_processes(None);
        self.connections = self.netstat_data.get_connections(None);
    }
}
