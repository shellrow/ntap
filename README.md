[crates-badge]: https://img.shields.io/crates/v/ntap.svg
[crates-url]: https://crates.io/crates/ntap
[license-badge]: https://img.shields.io/crates/l/ntap.svg

# ntap [![Crates.io][crates-badge]][crates-url] ![License][license-badge]

ntap is a cross-platform terminal network traffic monitor and packet inspector for Linux, macOS, and Windows. It combines live traffic totals, remote hosts, connections, process attribution, and bounded packet inspection in a keyboard-driven TUI.

## Commands

```text
ntap [OPTIONS] [COMMAND]

Commands:
  monitor      Start the traffic monitor (default)
  live         Inspect captured packets
  interfaces  List available interfaces
  interface   Show the default interface
```

Global capture filters may be placed before or after a subcommand:

```text
-i, --interfaces <interfaces>  Interface names separated by commas
-P, --protocols <protocols>    Protocol names separated by commas
-a, --ips <ips>                Local or remote IP addresses separated by commas
-p, --ports <ports>            Local or remote ports separated by commas
-r, --tickrate <duration_ms>   UI refresh period from 16 through 60000 ms
```

Supported protocol names are `arp`, `rarp`, `aarp`, `ipv4`, `ipv6`, `vlan`, `mpls`, `wakeonlan`, `rldp`, `lldp`, `icmp`, `icmpv6`, `tcp`, and `udp`. Protocol names are case-insensitive. Unknown values and unavailable interfaces are rejected at startup.

Live mode accepts `-l, --limit <count>` to set the number of packet records retained in memory. The default is 255 and zero is rejected. Capture-to-UI buffering is bounded; ntap drops packets and records a warning when the consumer cannot keep up.

### Examples

```sh
# Start the monitor on all usable interfaces
ntap

# Monitor TCP and UDP on a specific interface
ntap monitor -i en0 -P tcp,udp

# Inspect HTTPS traffic and retain the latest 500 packet records
ntap -p 443 live --limit 500

# List interfaces before choosing a capture target
ntap interfaces
```

## Keyboard Controls

Monitor mode:

- `Tab`, `Right`: next tab
- `Shift+Tab`, `Left`: previous tab
- `Space`: pause or resume the display while continuing to aggregate traffic
- `T`: toggle totals and per-second rates
- `Q` or `Ctrl-C`: quit

Live mode:

- `Up`, `W`: select the previous packet
- `Down`, `S`: select the next packet
- `B`: follow the newest packet
- `Space`: pause or resume display updates
- `Q` or `Ctrl-C`: quit

## Installation

Install from crates.io:

```sh
cargo install ntap
```

Install a prebuilt release on Linux or macOS:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/shellrow/ntap/releases/latest/download/ntap-installer.sh | sh
```

Install a prebuilt release on Windows PowerShell:

```powershell
irm https://github.com/shellrow/ntap/releases/latest/download/ntap-installer.ps1 | iex
```

Build from source:

```sh
git clone https://github.com/shellrow/ntap
cd ntap
cargo build --release
./target/release/ntap
```

## Capture Permissions

### Linux

Raw packet capture and process attribution require additional privileges. For a trusted single-user machine, grant only the capabilities ntap needs:

```sh
sudo setcap 'cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep' "$(command -v ntap)"
ntap
```

Alternatively, run `sudo ntap` where requiring privilege escalation for every capture is preferable.

### macOS

ntap requires access to Berkeley Packet Filter devices. The `chmod-bpf` service can configure that access:

```sh
brew install shellrow/tap-chmod-bpf/chmod-bpf
chmod-bpf check
sudo chmod-bpf install
```

### Windows

Install [Npcap](https://npcap.com/#download) with WinPcap API-compatible mode enabled. ntap checks for Npcap before opening the TUI.

## Configuration and Logging

ntap creates `~/.ntap/ntap-config.json` on first monitor or live run. Missing fields receive defaults so configurations remain forward-compatible. Invalid JSON or unsafe timing values produce an actionable startup error rather than overwriting the file.

```json
{
  "logging": {
    "level": "INFO",
    "file_path": "/home/user/.ntap/ntap.log"
  },
  "network": {
    "interfaces": [],
    "reverse_dns": false,
    "entry_ttl": 60000
  },
  "display": {
    "top_remote_hosts": 20,
    "connection_count": 20,
    "tick_rate": 1000,
    "show_bandwidth": false
  }
}
```

Logs are appended to the configured file and filtered by `DEBUG`, `INFO`, `WARN`, or `ERROR`. Reverse DNS is disabled by default to avoid extra network queries; enable it explicitly when hostnames are useful.

## Troubleshooting

- `no usable capture interfaces were found`: connect or enable an interface, then inspect `ntap interfaces`.
- `unknown or unavailable interface`: use the exact interface name shown by `ntap interfaces`.
- `failed to capture on interface`: verify Linux capabilities, macOS BPF permissions, or the Windows Npcap installation.
- Missing process names: process attribution may require `cap_sys_ptrace` and `cap_dac_read_search` on Linux or elevated privileges on other platforms.
- Terminal display corruption after an external hard kill cannot be intercepted; run `reset` or `stty sane`. Normal exits, Ctrl-C, and runtime errors restore the terminal automatically.

## Privacy and Security

Live mode displays packet payload previews, which may contain credentials, tokens, personal data, or other sensitive content. Capture only networks and systems you are authorized to inspect. Log files contain operational errors and may include interface names and remote addresses; protect them according to your environment's retention policy.

## Development

The required local gates are:

```sh
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo audit
```

CI keeps pull-request feedback lightweight by running formatting, Clippy, and locked tests in one Linux job. Before a release, run the same checks on supported platforms and perform the RustSec audit locally. Releases and publishing are intentionally manual.

## License

ntap is released under the MIT License. See [LICENSE](LICENSE).
