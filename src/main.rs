mod config;
mod db;
mod deps;
mod handler;
mod log;
mod net;
mod process;
mod sys;
mod tui;
mod util;

use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};
use std::net::IpAddr;

#[derive(Debug, Parser)]
#[command(
    name = clap::crate_name!(),
    version = clap::crate_version!(),
    about = clap::crate_description!()
)]
struct Cli {
    #[arg(
        short = 'i',
        long = "interfaces",
        value_delimiter = ',',
        value_name = "interfaces"
    )]
    interfaces: Vec<String>,
    #[arg(
        short = 'P',
        long = "protocols",
        value_delimiter = ',',
        value_name = "protocols"
    )]
    protocols: Vec<String>,
    #[arg(short = 'a', long = "ips", value_delimiter = ',', value_name = "ips")]
    ips: Vec<IpAddr>,
    #[arg(
        short = 'p',
        long = "ports",
        value_delimiter = ',',
        value_name = "ports"
    )]
    ports: Vec<u16>,
    #[arg(short = 'r', long = "tickrate", value_name = "duration_ms")]
    tickrate: Option<u64>,
    #[arg(long = "enhanced-graphics", action = ArgAction::SetTrue)]
    enhanced_graphics: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Monitor(MonitorArgs),
    Live(LiveArgs),
    Interfaces,
    Interface,
}

#[derive(Debug, Parser, Default)]
struct MonitorArgs {
    #[arg(
        short = 'i',
        long = "interfaces",
        value_delimiter = ',',
        value_name = "interfaces"
    )]
    interfaces: Vec<String>,
    #[arg(
        short = 'P',
        long = "protocols",
        value_delimiter = ',',
        value_name = "protocols"
    )]
    protocols: Vec<String>,
    #[arg(short = 'a', long = "ips", value_delimiter = ',', value_name = "ips")]
    ips: Vec<IpAddr>,
    #[arg(
        short = 'p',
        long = "ports",
        value_delimiter = ',',
        value_name = "ports"
    )]
    ports: Vec<u16>,
}

#[derive(Debug, Parser, Default)]
struct LiveArgs {
    #[arg(
        short = 'i',
        long = "interfaces",
        value_delimiter = ',',
        value_name = "interfaces"
    )]
    interfaces: Vec<String>,
    #[arg(
        short = 'P',
        long = "protocols",
        value_delimiter = ',',
        value_name = "protocols"
    )]
    protocols: Vec<String>,
    #[arg(short = 'a', long = "ips", value_delimiter = ',', value_name = "ips")]
    ips: Vec<IpAddr>,
    #[arg(
        short = 'p',
        long = "ports",
        value_delimiter = ',',
        value_name = "ports"
    )]
    ports: Vec<u16>,
    #[arg(short = 'l', long = "limit", value_name = "count")]
    limit: Option<usize>,
}

fn to_monitor_options(cli: &Cli, cmd: Option<&MonitorArgs>) -> handler::monitor::MonitorOptions {
    let interfaces = cmd
        .map(|c| c.interfaces.clone())
        .unwrap_or_else(|| cli.interfaces.clone());
    let protocols = cmd
        .map(|c| c.protocols.clone())
        .unwrap_or_else(|| cli.protocols.clone());
    let ips = cmd
        .map(|c| c.ips.clone())
        .unwrap_or_else(|| cli.ips.clone());
    let ports = cmd
        .map(|c| c.ports.clone())
        .unwrap_or_else(|| cli.ports.clone());
    handler::monitor::MonitorOptions {
        interfaces,
        protocols,
        ips,
        ports,
        tickrate: cli.tickrate,
        enhanced_graphics: cli.enhanced_graphics,
    }
}

fn to_live_options(cli: &Cli, cmd: Option<&LiveArgs>) -> handler::live::LiveOptions {
    let interfaces = cmd
        .map(|c| c.interfaces.clone())
        .unwrap_or_else(|| cli.interfaces.clone());
    let protocols = cmd
        .map(|c| c.protocols.clone())
        .unwrap_or_else(|| cli.protocols.clone());
    let ips = cmd
        .map(|c| c.ips.clone())
        .unwrap_or_else(|| cli.ips.clone());
    let ports = cmd
        .map(|c| c.ports.clone())
        .unwrap_or_else(|| cli.ports.clone());
    let limit = cmd.and_then(|c| c.limit);
    handler::live::LiveOptions {
        interfaces,
        protocols,
        ips,
        ports,
        tickrate: cli.tickrate,
        enhanced_graphics: cli.enhanced_graphics,
        limit,
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Some(Command::Monitor(args)) => {
            handler::monitor::monitor(to_monitor_options(&cli, Some(args))).await
        }
        Some(Command::Live(args)) => handler::live::live_capture(to_live_options(&cli, Some(args))).await,
        Some(Command::Interfaces) => handler::interface::show_interfaces(),
        Some(Command::Interface) => handler::interface::show_default_interface(),
        None => handler::monitor::monitor(to_monitor_options(&cli, None)).await,
    }
}
