mod tui;
mod config;
mod db;
mod deps;
mod handler;
mod net;
mod notification;
mod process;
mod sys;
mod thread_log;
mod time;

use clap::{crate_description, crate_name, crate_version, value_parser};
use clap::{Arg, ArgMatches, Command};
use handler::AppCommands;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args: ArgMatches = parse_args();
    let subcommand_name = args.subcommand_name().unwrap_or("");
    let app_command = AppCommands::from_str(subcommand_name);
    match app_command {
        AppCommands::Live => handler::live::live_capture(&args),
        AppCommands::Monitor => handler::monitor::monitor(&args),
        AppCommands::Interfaces => handler::interface::show_interfaces(),
        AppCommands::Interface => handler::interface::show_default_interface(),
        AppCommands::Route => handler::route::show_routes(),
        AppCommands::Socket => handler::socket::show_socket_info(),
        AppCommands::IpInfo => handler::ip_info::show_public_ip_info(),
        AppCommands::Default => {
            // If no subcommand is specified, enter live mode by default
            handler::live::live_capture(&args)
        }
    }
}

fn parse_args() -> ArgMatches {
    let app: Command = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .after_help("By default, if no options are specified, ntap enters the monitor mode which continuously displays live network usage statistics.")
        .arg(
            Arg::new("limit")
                .help("Limit the number of packets to display")
                .short('l')
                .long("limit")
                .value_name("count")
                .value_parser(value_parser!(u8)),
        )
        .arg(
            Arg::new("tickrate")
                .help("Time in milliseconds between refreshes")
                .short('r')
                .long("tickrate")
                .value_name("duration_ms")
                .value_parser(value_parser!(u64)),
        )
        .arg(
            Arg::new("enhanced-graphics")
                .help("Whether unicode symbols are used to improve the overall look of the app")
                .long("enhanced-graphics")
                .num_args(0),
        )
        // Sub-command for live mode. This is the default mode of ntap
        .subcommand(Command::new("live")
            .about("Start live packet capture. Live mode captures and displays live network packets.")
        )
        // Sub-command for monitor mode.
        .subcommand(Command::new("monitor")
            .about("Enter monitor mode. Monitor mode continuously displays live network usage statistics.")
        )
        // Sub-command for show active TCP connections and the TCP and UDP ports on which is listening
        .subcommand(
            Command::new("socket")
                .about("Show active TCP connections and the TCP and UDP ports on which is listening. ntap socket --help for more information")
                .arg(
                    Arg::new("protocol")
                        .help("specify protocols. The protocol can be ipv4, ipv6, tcp, udp")
                        .short('p')
                        .long("protocol")
                        .value_name("protocols")
                        .value_delimiter(',')
                ),
        )
        // Sub-command for show network interfaces
        .subcommand(Command::new("interfaces")
            .about("Show network interfaces")
        )
        // Sub-command for show default network interface
        .subcommand(Command::new("interface")
            .about("Show default network interface")
        )
        // Sub-command for show IP routing table
        .subcommand(Command::new("route")
            .about("Show IP routing table")
        )
        // Sub-command for show public IP info
        .subcommand(Command::new("ipinfo")
            .about("Show public IP info")
        )
        ;
    app.get_matches()
}
