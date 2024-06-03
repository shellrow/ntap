# ntap

## Overview
**ntap** is a real-time network utilization monitoring tool.  
It provides comprehensive insights into your network's activity, enabling users to monitor traffic, manage connections, and view network configurations with ease.

## Features
- **Real-time Monitoring**: Track network utilization as it happens.
- **Connection Management**: Analyze active connections quickly and effectively.
- **Interface and Routing Insights**: Get detailed views of network interfaces and routing tables.
- **Customizable Options**: Tailor the monitoring experience to your needs with adjustable settings.
- **Two versions available**: CLI app and Desktop app are available, catering to different user preferences.

## Prerequisites
- Ensure you have a compatible operating system (Linux, macOS, Windows).

## Installation

### CLI: ntap

#### Pre-built binaries
Download the pre-built binaries for your platform from the [releases page](https://github.com/shellrow/ntap/releases).

#### Cargo
If you have Rust installed, you can install ntap directly using cargo:
```bash
cargo install ntap
```

#### Build from source
First, clone the repository:
```
git clone https://github.com/shellrow/ntap
```
Then, build the project:
```
cd ntap
cargo build --release
./target/release/ntap
```

### Desktop Application: ntap-desktop

#### Using Installer
Download the installer for your platform from the [releases page](https://github.com/shellrow/ntap/releases).

#### Build from source
First, clone the repository:
```
git clone https://github.com/shellrow/ntap
```
Then, build the project (assuming Rust and Tauri are already installed):
```
cd ntap-desktop
cargo tauri build
```
Run the installer in the dist directory.


### Usage
For detailed usage instructions, please refer to the README files within each version's directory:

- [ntap](https://github.com/shellrow/ntap/tree/main/ntap)
- [ntap-desktop](https://github.com/shellrow/ntap/tree/main/ntap-desktop)

### License
ntap is released under the MIT License. See the LICENSE file for more details.
