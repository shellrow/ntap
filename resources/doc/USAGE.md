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
If no options or commands are specified, ntap will automatically enter the stat mode:
```bash
ntap
```
This default mode captures packets on all available network interfaces and continuously displays live network statistics, providing a quick and easy way to monitor current network activity without the need for additional configuration.

## Commands

### stat: Enters stat mode to continuously display live network statistics, covering bytes/bandwidth usage, top remote hosts, connections, and processes.
```bash
ntap stat
```

### live: Enters live capture mode to continuously display live network packet data.
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

For more detailed options for sockets:
```bash
ntap socket --help
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

For help on a specific command, such as monitor:
```bash
ntap help socket
```

## Options
--tickrate(-r) <duration_ms>: Specifies the time in milliseconds between two updates in monitor mode.
```bash
ntap --tickrate 1000 monitor
ntap --r 1000 monitor
```

-h, --help: Prints help information.
```bash
ntap --help
```

-V, --version: Displays the version of the ntap tool.
```bash
ntap --version
```
