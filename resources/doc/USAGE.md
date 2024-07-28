# Usage Guide for ntap

## Overview
`ntap` is a real-time network monitoring tool that provides comprehensive insights into your network activity.  
This guide describes how to use `ntap` and its various commands and options.

## Basic Usage
To run `ntap`, use the following syntax:

```bash
ntap [OPTIONS] [COMMAND]
```

## Default Behavior
By default, if no subcommand is specified, ntap enters the `stat` mode, which displays continuous network statistics:
```bash
ntap
```
This default mode captures packets on all available network interfaces and continuously displays live network statistics, providing a quick and easy way to monitor current network activity without the need for additional configuration.

## Commands

### stat: Continuously displays network statistics, including bytes/bandwidth usage, top remote hosts, connections, and processes.
```bash
ntap stat
```

### live: Start live packet capture, continuously display live network packet data.
```bash
ntap live
```

### monitor: Enters monitor mode to continuously display live network usage statistics with associated country and AS (or ISP) info.
```bash
ntap monitor
```

### socket: Displays active TCP connections and the ports for TCP and UDP that are listening.
```bash
ntap socket
```

### interfaces: Shows all network interfaces.
```bash
ntap interfaces
```

### interface: Displays the default network interface.
```bash
ntap interface
```

### route: Shows the IP routing table.
```bash
ntap route
```

### ipinfo: Displays public IP information.
```bash
ntap ipinfo
```

### help: Prints the main help message or help for a specific command.
```bash
ntap help
```

For help on a specific command, such as live:
```bash
ntap help live
```

## Options
Help (`-h`, `--help`): Prints help information.
```bash
ntap --help
```

Subcommand Help (`-h`, `--help`): Prints subcommand's help information.
```bash
ntap live --help
```

Tick Rate (`-r`, `--tickrate`): Sets the refresh rate in milliseconds.
```bash
ntap --tickrate 1000
```

Interfaces (`-i`, `--interfaces`): Specifies the interfaces by name.
```bash
ntap -i eth0,eth1
```

Protocols (`-P`, `--protocols`): Filters traffic by protocols.
```bash
ntap -P tcp,udp
```

IP Addresses (`-a`, `--ips`): Filters traffic by specific IP addresses.
```bash
ntap --ips 1.1.1.1,8.8.8.8
```

Ports (`-p`, `--ports`): Filters traffic by ports.
```bash
ntap -p 80,443
```

Version (`-V`, `--version`): Displays the version of the ntap.
```bash
ntap --version
```
