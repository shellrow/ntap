mod config;
mod handler;
mod log;
mod net;
mod process;
mod sys;
mod tui;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::net::IpAddr;

#[derive(Debug, Parser)]
#[command(
    name = clap::crate_name!(),
    version = clap::crate_version!(),
    about = clap::crate_description!()
)]
struct Cli {
    /// Capture only these comma-separated interface names.
    #[arg(
        short = 'i',
        long = "interfaces",
        value_delimiter = ',',
        value_name = "interfaces",
        global = true
    )]
    interfaces: Vec<String>,
    /// Capture only these comma-separated protocol names.
    #[arg(
        short = 'P',
        long = "protocols",
        value_delimiter = ',',
        value_name = "protocols",
        value_parser = ["arp", "rarp", "aarp", "ipv4", "ipv6", "vlan", "mpls", "wakeonlan", "rldp", "lldp", "icmp", "icmpv6", "tcp", "udp"],
        ignore_case = true,
        global = true
    )]
    protocols: Vec<String>,
    /// Capture packets where either endpoint matches one of these IP addresses.
    #[arg(
        short = 'a',
        long = "ips",
        value_delimiter = ',',
        value_name = "ips",
        global = true
    )]
    ips: Vec<IpAddr>,
    /// Capture packets where either endpoint matches one of these ports.
    #[arg(
        short = 'p',
        long = "ports",
        value_delimiter = ',',
        value_name = "ports",
        global = true
    )]
    ports: Vec<u16>,
    /// Set the UI refresh period in milliseconds.
    #[arg(
        short = 'r',
        long = "tickrate",
        value_name = "duration_ms",
        value_parser = clap::value_parser!(u64).range(16..=60_000),
        global = true
    )]
    tickrate: Option<u64>,
    #[command(subcommand)]
    command: Option<Command>,
}

fn parse_nonzero_usize(value: &str) -> Result<usize, String> {
    let value = value
        .parse::<usize>()
        .map_err(|error| format!("invalid positive integer: {error}"))?;
    if value == 0 {
        Err("value must be greater than zero".to_string())
    } else {
        Ok(value)
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the network traffic monitor.
    Monitor,
    /// Inspect captured packets and payload previews.
    Live(LiveArgs),
    /// List all network interfaces.
    Interfaces,
    /// Show the default network interface.
    Interface,
}

#[derive(Debug, Parser, Default)]
struct LiveArgs {
    /// Retain at most this many packet records in memory.
    #[arg(
        short = 'l',
        long = "limit",
        value_name = "count",
        value_parser = parse_nonzero_usize
    )]
    limit: Option<usize>,
}

fn to_monitor_options(cli: &Cli) -> handler::monitor::MonitorOptions {
    handler::monitor::MonitorOptions {
        interfaces: cli.interfaces.clone(),
        protocols: cli.protocols.clone(),
        ips: cli.ips.clone(),
        ports: cli.ports.clone(),
        tickrate: cli.tickrate,
    }
}

fn to_live_options(cli: &Cli, cmd: &LiveArgs) -> handler::live::LiveOptions {
    handler::live::LiveOptions {
        interfaces: cli.interfaces.clone(),
        protocols: cli.protocols.clone(),
        ips: cli.ips.clone(),
        ports: cli.ports.clone(),
        tickrate: cli.tickrate,
        limit: cmd.limit,
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Some(Command::Monitor) => handler::monitor::monitor(to_monitor_options(&cli)).await,
        Some(Command::Live(args)) => handler::live::live_capture(to_live_options(&cli, args)).await,
        Some(Command::Interfaces) => handler::interface::show_interfaces(),
        Some(Command::Interface) => handler::interface::show_default_interface(),
        None => handler::monitor::monitor(to_monitor_options(&cli)).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_filters_are_accepted_around_subcommands() {
        let before = Cli::try_parse_from(["ntap", "-P", "tcp", "monitor"]);
        let after = Cli::try_parse_from(["ntap", "monitor", "-P", "udp"]);
        assert_eq!(before.unwrap().protocols, ["tcp"]);
        assert_eq!(after.unwrap().protocols, ["udp"]);
    }

    #[test]
    fn invalid_runtime_limits_are_rejected() {
        assert!(Cli::try_parse_from(["ntap", "--tickrate", "0"]).is_err());
        assert!(Cli::try_parse_from(["ntap", "live", "--limit", "0"]).is_err());
        assert!(Cli::try_parse_from(["ntap", "-P", "not-a-protocol"]).is_err());
    }
}
