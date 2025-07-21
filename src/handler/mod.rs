pub mod interface;
pub mod live;
pub mod monitor;
pub mod route;
pub mod socket;
pub mod stat;
pub mod update;

pub enum AppCommands {
    Stat,
    Live,
    Monitor,
    Interfaces,
    Interface,
    Route,
    Socket,
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
            "update" => AppCommands::Update,
            _ => AppCommands::Default,
        }
    }
}
