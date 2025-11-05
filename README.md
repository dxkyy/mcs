# MCS - Minecraft Server CLI

A fast and user-friendly command-line tool written in Rust for creating and managing Minecraft servers.

## Features

- Interactive server configuration with searchable version selection
- Automatic server JAR download from official sources
- Auto-generated start scripts for Windows and Linux/Mac
- Automatic EULA acceptance
- Configuration persistence via `mcs.toml`
- Supports Paper and Vanilla servers

## Installation

### From Source

```bash
git clone <repository-url>
cd mcs
cargo install --path .
```

## Usage

### Create a New Server

Create a new Minecraft server in the specified directory:

```bash
mcs new ./my-server
```

This will launch an interactive prompt asking you to configure:

- **Server Type**: Paper or Vanilla (more types coming soon)
- **Minecraft Version**: Select from all available versions using arrow keys or type to search
- **Memory Allocation**: Amount of RAM to allocate (e.g., 2G, 4G, 8G)

After configuration, the following files will be created:

- `server.jar` - The Minecraft server executable
- `start.bat` - Windows start script
- `start.sh` - Linux/Mac start script
- `eula.txt` - EULA file (automatically accepted)
- `mcs.toml` - Server configuration file

### Starting Your Server

On Windows:

```bash
start.bat
```

On Linux/Mac:

```bash
./start.sh
```

### Reconfigure an Existing Server

Navigate to your server directory and run:

```bash
cd my-server
mcs configure
```

This will prompt you to select new configuration options and re-download the server files accordingly.

### Apply Configuration Changes

If you manually edited the `mcs.toml` file, apply the changes with:

```bash
cd my-server
mcs apply
```

This will re-download the server files based on the updated configuration.

## Configuration File

The `mcs.toml` file stores your server configuration:

```toml
version = "1.21.8"
memory = "2G"

[server_type]
Paper
```

You can manually edit this file and run `mcs apply` to update your server, or use `mcs configure` for an interactive reconfiguration.

## Supported Server Types

- [x] Paper
- [x] Vanilla
- [ ] Spigot (coming soon)
- [ ] Forge (coming soon)
- [ ] Fabric (coming soon)

## Requirements

- Rust 1.70+ (for building from source)
- Java 21+ (for running Minecraft 1.21+)
- Java 17+ (for running Minecraft 1.18-1.20)
- Java 16+ (for running Minecraft 1.17)

## Contributing

Contributions are welcome! Feel free to submit issues or pull requests.
