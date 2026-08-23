# fault

Network fault injection for processes and destinations

Fault uses Linux traffic control to simulate poor network conditions such as latency, packet loss, jitter, corruption, duplication, reordering, and bandwidth limits

Fault can either isolate and fault a process:

```bash
sudo fault ./server --latency 200 --loss 5
```

or fault traffic between your machine and one or more destinations:

```bash
sudo fault discord.com github.com --latency 200 --loss 5
```

## Requirements

Fault currently requires Linux and root privileges

The following commands must be available:

- `ip`
- `tc`
- `nft`

`ip` and `tc` are provided by `iproute2`. `nft` is provided by `nftables`

The kernel must also support network namespaces, NetEm, and IFB devices

## Installation

Clone and build Fault:

```bash
git clone https://github.com/gavrh/fault
cd fault
cargo build --release
```

The binary will be available at:

```text
target/release/fault
```

Optionally install it globally:

```bash
sudo install target/release/fault /usr/local/bin/fault
```

## Usage

```text
fault [OPTIONS] <TARGET>...
```

Fault automatically chooses between process and destination mode based on the first target

### Destination mode

Pass one or more domains, URLs, or IP addresses:

```bash
sudo fault discord.com --latency 200
```

```bash
sudo fault github.com discord.com --loss 5
```

```bash
sudo fault 1.1.1.1 --latency 100 --loss 2
```

URLs are also accepted:

```bash
sudo fault https://discord.com --latency 200
```

Domains are resolved to their IP addresses when Fault starts. Fault then applies the configured conditions to traffic traveling both to and from those addresses

For example:

```bash
sudo fault discord.com --latency 200
```

adds approximately 200 ms of delay to outgoing Discord traffic and 200 ms to incoming Discord traffic

Fault continues running until `Ctrl+C` is pressed

### Process mode

Pass an existing executable or script path as the first target:

```bash
sudo fault ./server --latency 200
```

Fault creates an isolated network namespace for the process and applies the configured conditions to both its incoming and outgoing traffic

Arguments can also be passed to the process. Use `--` when the process arguments could be interpreted as Fault options:

```bash
sudo fault --latency 200 -- ./server --port 3000
```

The first target must currently be an existing file path. Commands resolved only through `$PATH`, such as:

```bash
sudo fault cargo run
```

are not currently detected as process targets

### Publishing ports

Processes run inside their own network namespace. TCP ports can be published back to the host with:

```text
--publish HOST:CONTAINER
```

For example, if the process listens on port `3000`:

```bash
sudo fault ./server --latency 200 --publish 8080:3000
```

traffic sent to host port `8080` is forwarded to port `3000` inside the process namespace

Multiple ports can be published:

```bash
sudo fault ./server \
    --publish 8080:3000 \
    --publish 8081:3001 \
    --latency 200
```

`--publish` is only available in process mode and currently supports TCP

## Fault options

| Option | Short | Unit | Description |
| --- | --- | --- | --- |
| `--corrupt` | `-c` | % | Packet corruption |
| `--duplicate` | `-d` | % | Packet duplication |
| `--jitter` | `-j` | ms | Network latency variation |
| `--latency` | `-t` | ms | Added network latency |
| `--loss` | `-l` | % | Packet loss |
| `--rate` | `-r` | Mbit/s | Network rate limit |
| `--reorder` | `-o` | % | Packet reordering |

Options can be combined:

```bash
sudo fault discord.com \
    --latency 200 \
    --jitter 50 \
    --loss 5 \
    --rate 10
```

Percentage values must be between `0` and `100`

Options that are not specified default to `0` and are not included in the NetEm configuration

### Verbose logging

Use `-v` for debug logging:

```bash
sudo fault -v discord.com --latency 200
```

or `-vv` for additional logging:

```bash
sudo fault -vv ./server --loss 5
```

## How it works

For destination targets, Fault resolves the targets to IP addresses and uses `tc` filters to select matching traffic. Outgoing traffic is shaped on the host interface, while incoming traffic is redirected through an IFB device so NetEm can apply the same conditions in the opposite direction

For process targets, Fault creates a network namespace and veth pair. The process runs inside the namespace while nftables provides forwarding and NAT to the host network. NetEm is applied on both sides of the veth pair

Fault tracks the networking resources it creates and removes them when the session finishes normally or is interrupted with `Ctrl+C`

## Current limitations

- Linux only
- Root privileges are currently required
- Process mode only detects existing file paths as process targets
- Domains are resolved when Fault starts; DNS changes are not continuously monitored
- `--publish` currently supports TCP only
- Destination mode modifies the traffic-control configuration of the default network interface. Do not use it on an interface with important custom `tc`/qdisc configuration
- Forcefully killing Fault with `SIGKILL` or crashing the machine can prevent normal cleanup

## License

Fault is licensed under the GNU Affero General Public License v3.0
