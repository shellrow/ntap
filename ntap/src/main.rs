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
mod tui;
mod util;

use clap::{crate_description, crate_name, crate_version, value_parser};
use clap::{Arg, ArgMatches, Command};
use handler::AppCommands;
use std::error::Error;
use std::net::IpAddr;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args: ArgMatches = parse_args();
    let subcommand_name = args.subcommand_name().unwrap_or("");
    let app_command = AppCommands::from_str(subcommand_name);
    match app_command {
        AppCommands::Stat => handler::stat::show_stat(&args),
        AppCommands::Live => handler::live::live_capture(&args),
        AppCommands::Monitor => handler::monitor::monitor(&args),
        AppCommands::Interfaces => handler::interface::show_interfaces(),
        AppCommands::Interface => handler::interface::show_default_interface(),
        AppCommands::Route => handler::route::show_routes(),
        AppCommands::Socket => handler::socket::show_socket_info(&args),
        AppCommands::IpInfo => handler::ip_info::show_public_ip_info(),
        AppCommands::Update => handler::update::download_db_files(),
        AppCommands::Default => {
            // If no subcommand is specified, enter stat mode by default
            handler::stat::show_stat_default(&args)
        }
    }
}

fn parse_args() -> ArgMatches {
    let app: Command = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .after_help("By default, if no options are specified, ntap enters the stat mode.")
        .arg(
            Arg::new("interfaces")
                .help("Specify the interfaces by name. Example: ntap -i eth0,eth1")
                .short('i')
                .long("interfaces")
                .value_name("interfaces")
                .value_delimiter(',')
                .value_parser(value_parser!(String))
        )
        .arg(
            Arg::new("protocols")
                .help("Specify protocols. Example: ntap -P tcp,udp")
                .short('P')
                .long("protocols")
                .value_name("protocols")
                .value_delimiter(',')
                .value_parser(value_parser!(String))
        )
        .arg(
            Arg::new("ips")
                .help("Specify IP addresses. Example: ntap -a 1.1.1.1,8.8.8.8")
                .short('a')
                .long("ips")
                .value_name("ips")
                .value_delimiter(',')
                .value_parser(value_parser!(IpAddr))
        )
        .arg(
            Arg::new("ports")
                .help("Specify ports. Example: ntap -p 80,443")
                .short('p')
                .long("ports")
                .value_name("ports")
                .value_delimiter(',')
                .value_parser(value_parser!(u16))
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
        // Sub-command for stat mode. This is the default mode of ntap
        .subcommand(Command::new("stat")
            .about("Continuously displays network statistics, covering bytes/bandwidth usage, top remote hosts, connections, and processes.")
            .arg(
                Arg::new("interfaces")
                    .help("Specify the interfaces by name. Example: ntap stat -i eth0,eth1")
                    .short('i')
                    .long("interfaces")
                    .value_name("interfaces")
                    .value_delimiter(',')
                    .value_parser(value_parser!(String))
            )
            .arg(
                Arg::new("protocols")
                    .help("Specify protocols. Example: ntap stat -P tcp,udp")
                    .short('P')
                    .long("protocols")
                    .value_name("protocols")
                    .value_delimiter(',')
                    .value_parser(value_parser!(String))
            )
            .arg(
                Arg::new("ips")
                    .help("Specify IP addresses. Example: ntap stat -a 1.1.1.1,8.8.8.8")
                    .short('a')
                    .long("ips")
                    .value_name("ips")
                    .value_delimiter(',')
                    .value_parser(value_parser!(IpAddr))
            )
            .arg(
                Arg::new("ports")
                    .help("Specify ports. Example: ntap stat -p 80,443")
                    .short('p')
                    .long("ports")
                    .value_name("ports")
                    .value_delimiter(',')
                    .value_parser(value_parser!(u16))
            )
        )
        // Sub-command for live mode.
        .subcommand(Command::new("live")
            .about("Start live packet capture. Live mode captures and displays live network packets.")
            .arg(
                Arg::new("limit")
                    .help("Limit the number of packets to display")
                    .short('l')
                    .long("limit")
                    .value_name("count")
                    .value_parser(value_parser!(u8)),
            )
            .arg(
                Arg::new("interfaces")
                    .help("Specify the interfaces by name. Example: ntap live -i eth0,eth1")
                    .short('i')
                    .long("interfaces")
                    .value_name("interfaces")
                    .value_delimiter(',')
                    .value_parser(value_parser!(String))
            )
            .arg(
                Arg::new("protocols")
                    .help("Specify protocols. Example: ntap live -P tcp,udp")
                    .short('P')
                    .long("protocols")
                    .value_name("protocols")
                    .value_delimiter(',')
                    .value_parser(value_parser!(String))
            )
            .arg(
                Arg::new("ips")
                    .help("Specify IP addresses. Example: ntap live -a 1.1.1.1,8.8.8.8")
                    .short('a')
                    .long("ips")
                    .value_name("ips")
                    .value_delimiter(',')
                    .value_parser(value_parser!(IpAddr))
            )
            .arg(
                Arg::new("ports")
                    .help("Specify ports. Example: ntap live -p 80,443")
                    .short('p')
                    .long("ports")
                    .value_name("ports")
                    .value_delimiter(',')
                    .value_parser(value_parser!(u16))
            )
        )
        // Sub-command for monitor mode.
        .subcommand(Command::new("monitor")
            .about("Enter monitor mode. Monitor mode continuously displays live network statistics with Country and AS (or ISP) info.")
            .arg(
                Arg::new("interfaces")
                    .help("Specify the interfaces by name. Example: ntap monitor -i eth0,eth1")
                    .short('i')
                    .long("interfaces")
                    .value_name("interfaces")
                    .value_delimiter(',')
                    .value_parser(value_parser!(String))
            )
            .arg(
                Arg::new("protocols")
                    .help("Specify protocols. Example: ntap monitor -P tcp,udp")
                    .short('P')
                    .long("protocols")
                    .value_name("protocols")
                    .value_delimiter(',')
                    .value_parser(value_parser!(String))
            )
            .arg(
                Arg::new("ips")
                    .help("Specify IP addresses. Example: ntap monitor -a 1.1.1.1,8.8.8.8")
                    .short('a')
                    .long("ips")
                    .value_name("ips")
                    .value_delimiter(',')
                    .value_parser(value_parser!(IpAddr))
            )
            .arg(
                Arg::new("ports")
                    .help("Specify ports. Example: ntap monitor -p 80,443")
                    .short('p')
                    .long("ports")
                    .value_name("ports")
                    .value_delimiter(',')
                    .value_parser(value_parser!(u16))
            )
        )
        // Sub-command for show active TCP connections and the TCP and UDP ports on which is listening
        .subcommand(
            Command::new("socket")
                .about("Show active TCP connections and the TCP and UDP ports on which is listening. ntap socket --help for more information")
                .arg(
                    Arg::new("protocols")
                        .help("Specify protocols. The protocol can be ipv4, ipv6, tcp, udp. Example: ntap socket -P tcp")
                        .short('P')
                        .long("protocols")
                        .value_name("protocols")
                        .value_delimiter(',')
                        .value_parser(value_parser!(String))
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
        // Sub-command for update ntap database
        .subcommand(Command::new("update")
            .about("Update ntap database")
        )
        ;
    app.get_matches()
}
