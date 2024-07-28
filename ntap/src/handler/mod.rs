pub mod interface;
pub mod ip_info;
pub mod live;
pub mod monitor;
pub mod route;
pub mod socket;
pub mod update;
pub mod stat;

pub enum AppCommands {
    Stat,
    Live,
    Monitor,
    Interfaces,
    Interface,
    Route,
    Socket,
    IpInfo,
    Update,
    Default,
}

impl AppCommands {
    pub fn from_str(s: &str) -> AppCommands {
        match s {
            "stat" => AppCommands::Stat,
            "live" => AppCommands::Live,
            "monitor" => AppCommands::Monitor,
            "interfaces" => AppCommands::Interfaces,
            "interface" => AppCommands::Interface,
            "route" => AppCommands::Route,
            "socket" => AppCommands::Socket,
            "ipinfo" => AppCommands::IpInfo,
            "update" => AppCommands::Update,
            _ => AppCommands::Default,
        }
    }
}
