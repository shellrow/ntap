pub mod interface;
pub mod monitor;
pub mod route;
pub mod socket;
pub mod ip_info;

pub enum AppCommands {
    Monitor,
    Interfaces,
    Interface,
    Route,
    Socket,
    IpInfo,
}

impl AppCommands {
    pub fn from_str(s: &str) -> AppCommands {
        match s {
            "monitor" => AppCommands::Monitor,
            "interfaces" => AppCommands::Interfaces,
            "interface" => AppCommands::Interface,
            "route" => AppCommands::Route,
            "socket" => AppCommands::Socket,
            "ipinfo" => AppCommands::IpInfo,
            _ => AppCommands::Monitor
        }
    }
}
