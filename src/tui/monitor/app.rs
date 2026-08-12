use crate::config::AppConfig;
use crate::net::host::HostDisplayInfo;
use crate::net::socket::SocketDisplayInfo;
use crate::net::stat::NetStatData;
use crate::process::ProcessDisplayInfo;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTab {
    Overview,
    Hosts,
    Connections,
    Processes,
}

impl FocusTab {
    pub const ALL: [FocusTab; 4] = [
        FocusTab::Overview,
        FocusTab::Hosts,
        FocusTab::Connections,
        FocusTab::Processes,
    ];

    pub fn title(self) -> &'static str {
        match self {
            FocusTab::Overview => "Overview",
            FocusTab::Hosts => "Hosts",
            FocusTab::Connections => "Connections",
            FocusTab::Processes => "Processes",
        }
    }
}

pub struct App {
    pub title: String,
    pub should_pause: bool,
    pub should_quit: bool,
    pub show_bandwidth: bool,
    pub selected_tab: FocusTab,
    pub netstat_data: NetStatData,
    pending_data: NetStatData,
    pending_duration: Duration,
    pub remote_hosts: Vec<HostDisplayInfo>,
    pub processes: Vec<ProcessDisplayInfo>,
    pub connections: Vec<SocketDisplayInfo>,
    pub config: AppConfig,
}

impl App {
    pub fn new(title: String, config: AppConfig) -> App {
        App {
            title,
            should_pause: false,
            should_quit: false,
            show_bandwidth: config.display.show_bandwidth,
            selected_tab: FocusTab::Overview,
            netstat_data: NetStatData::new(),
            pending_data: NetStatData::new(),
            pending_duration: Duration::ZERO,
            remote_hosts: vec![],
            processes: vec![],
            connections: vec![],
            config,
        }
    }

    pub fn switch_next_tab(&mut self) {
        let index = FocusTab::ALL
            .iter()
            .position(|tab| *tab == self.selected_tab)
            .unwrap_or(0);
        let next_index = (index + 1) % FocusTab::ALL.len();
        self.selected_tab = FocusTab::ALL[next_index];
    }

    pub fn switch_prev_tab(&mut self) {
        let index = FocusTab::ALL
            .iter()
            .position(|tab| *tab == self.selected_tab)
            .unwrap_or(0);
        let prev_index = if index == 0 {
            FocusTab::ALL.len() - 1
        } else {
            index - 1
        };
        self.selected_tab = FocusTab::ALL[prev_index];
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => self.should_quit = true,
            ' ' => self.should_pause = !self.should_pause,
            't' => self.show_bandwidth = !self.show_bandwidth,
            _ => {}
        }
    }

    pub fn on_tick(&mut self, netstat_data: NetStatData) {
        let tick_rate = Duration::from_millis(self.config.display.tick_rate);
        if self.should_pause {
            self.pending_data.merge(netstat_data, tick_rate);
            self.pending_duration = self.pending_duration.saturating_add(tick_rate);
            return;
        }
        if !self.pending_duration.is_zero() {
            self.netstat_data.merge(
                std::mem::replace(&mut self.pending_data, NetStatData::new()),
                self.pending_duration,
            );
            self.pending_duration = Duration::ZERO;
        }
        self.netstat_data.merge(netstat_data, tick_rate);
        self.remote_hosts = self
            .netstat_data
            .get_remote_hosts(Some(self.config.display.top_remote_hosts));
        self.connections = self
            .netstat_data
            .get_connections(Some(self.config.display.connection_count));
        self.processes = self
            .netstat_data
            .get_processes(Some(self.config.display.connection_count));
    }
}
