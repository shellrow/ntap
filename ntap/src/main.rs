mod app;
mod handler;
mod sys;
mod terminal;
mod ui;

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
        AppCommands::Monitor => {
            handler::monitor::monitor(&args)
        },
        AppCommands::Interfaces => handler::interface::show_interfaces(),
        AppCommands::Interface => handler::interface::show_default_interface(),
        AppCommands::Route => handler::route::show_routes(),
        AppCommands::Socket => handler::socket::show_socket_info(),
    }
}

fn parse_args() -> ArgMatches {
    let app: Command = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .after_help("By default, if no options are specified, ntap enters the monitor mode which continuously displays live network usage statistics.")
        .arg(
            Arg::new("tick_rate")
                .help("Time in ms between two ticks")
                .long("tick_rate")
                .value_name("duration_ms")
                .value_parser(value_parser!(u64)),
        )
        .arg(
            Arg::new("enhanced_graphics")
                .help("Whether unicode symbols are used to improve the overall look of the app")
                .long("enhanced_graphics")
                .num_args(0),
        )
        // Sub-command for monitor mode. This is the default mode of ntap
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
        ;
    app.get_matches()
}
