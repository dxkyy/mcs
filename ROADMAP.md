# ROADMAP.md for MCS :3

---

## Version 0.2.0: Mod/Plugin Installation Focus

**Goal:** Enhance server customization with easy mod/plugin addition.

**Dependencies:** Rust crates for HTTP (reqwest), JSON parsing (serde), and interactive prompts (dialoguer).

**Notes:** Limit to Modrinth API for fetching metadata, versions, and downloads. Auto-detect server type (e.g., Paper/Spigot for plugins, Fabric/Forge for mods). Prompt for version if multiple available. Handle errors like incompatible server types.

- Implement `mcs add [mod name or URL]`: Install mods via Modrinth (e.g., `mcs add fabric-api` or `mcs add https://modrinth.com/mod/fabric-api`), with version selection prompt and compatibility checks.
- Implement `mcs add [plugin name or URL]`: Similar to mods, but for plugins (e.g., `mcs add worldedit`). (Detects wether to add mod or plugin based on server type.)
- Modrinth parser: Modrinth API Client to search, fetch versions, and download files matching the server's Minecraft version.

## Version 0.2.1: Initial Mod/Plugin Polish

**Goal:** Stabilize basics of new feature.

- Basic mod/plugin listing: `mcs list mods` or `mcs list plugins` (simple output of installed files).
- Add version conflict resolution (e.g., warn if mod requires different MC version).

## Version 0.2.2: Advanced Mod/Plugin Features

**Goal:** Expand mod/plugin capabilities.

- Support modpack installation: `mcs add [modpack name or URL]` (downloads and extracts modpacks from Modrinth).
- Improve error handling: Graceful failures for invalid URLs, API rate limits, or offline mode.

## Version 0.3.0: Core Updates and Validation

**Goal:** Improve reliability and usability of existing setup.

**Dependencies:** nothing major; build on existing config system.

- `mcs update`: Auto-update server JAR to latest compatible version, with options for specific versions.
- `mcs validate`: Scan directory for issues (e.g., Java mismatch, missing files) and suggest fixes.
- `mcs apply`: Expand to re-validate and fix after manual `mcs.toml` edits.

## Version 0.4.0: Templates and Presets

**Goal:** Speed up common setups.

**Dependencies:** TOML parsing (already in use).

**Notes:** Templates stored in `~/.mcs/templates/` as .toml files. Users can manually git clone from GitHub repos. Future: registry repo (e.g., github.com/dxkyy/mcs-registry) for community contributions, installable via `mcs template install https://github.com/user/template`. (or `mcs template install [name]` from registry).

- Basic template support: `mcs new --template [name] [path]`, loading from `~/.mcs/templates/[name].toml` (presets for properties, mods, etc.).
- `mcs template add [path]`: Copy a local .toml to global templates folder.

## Version 0.4.1: Template Integration

**Goal:** Tie templates to existing features.

- Integration with mod/plugin system: Templates can include auto-install lists (e.g., optimization mods for "optimized" template).

## Version 0.5.0: Global Server Management Basics

**Goal:** Handle multiple servers centrally.

**Dependencies:** File system ops; maybe simple DB like (toml file?) for registry.

- `mcs global init`: Set up `~/.mcs/servers/` registry.
- `mcs create [server-name] --global`: Create in global dir.
- `mcs list`: Show all global servers with status/version.

## Version 0.5.1: Global Management Commands

**Goal:** Expand global usability.

- `mcs start/stop [server-name]`: Manage global servers.
- `mcs switch [server-name]`: cd to or set active context.

## Version 0.6.0: Extensibility (MCS Plugin System)

**Goal:** Open to third-party features early on.

**Dependencies:** Define rust traits/interfaces; crates for dynamic loading if needed.

**Notes:** Focus on core plugin framework first; advanced features like registry in future versions. enables community-driven extensions for things like tmux integration or cloud backups.

- Core MCS plugin system: `mcs pl add/remove/list/install [name or URL]`.
- Plugin types: Server types, subcommands, (later: scheduling), etc.
- Plugin discovery: Support installing from repositories (e.g., GitHub or later registry repo) via `mcs pl install [name or URL]`.

## Version 0.6.1: Plugin API Polish

**Goal:** Make plugins more dev friendly.

- Plugin API: Define better rust trait/interface for plugins, with docs for hooks into commands like `mcs new` or `mcs start`.

## Version 0.7.0: Process and Runtime Management Basics

**Goal:** Better control over running servers.

**Dependencies:** External crates like sysinfo for monitoring; tmux/docker integration optional for detaching (now via plugin if desired).

**Notes:** Build on global management; make tmux a sample plugin.

- `mcs start [server] --detach`: Background start (e.g., via nohup or tmux).
- Basic process commands: `mcs status [server]`, `mcs kill [server]`.

## Version 0.7.1: Runtime Enhancements

**Goal:** Improve process handling.

- `mcs logs [server]`: Tail/search logs.
- Auto-restart flag: `--auto-restart` for crash recovery.

## Version 0.8.0: Networking and Exposure Basics

**Goal:** Make servers easier to share.

**Dependencies:** UPnP crate (e.g., igd); OS-specific for iptables.

- `mcs port-forward [server]`: Auto-UPnP or suggest rules, with conflict detection.

## Version 0.8.1: Firewall Integration

**Goal:** Security-focused networking.

- `mcs firewall check/add`: Add rules for common OSes.

## Version 0.9.0: World Management Basics

**Goal:** better server content handling.

**Dependencies:** Zip/compression for backups.

- `mcs world add [name]`: Import/generate new worlds.
- `mcs world list/switch`: Manage active worlds.

## Version 0.9.1: World Backups

**Goal:** Add data protection for worlds.

- `mcs world backup [name]`: Targeted backups.

## Version 0.10.0: Remote Control Basics

**Goal:** Interact without console access.

**Dependencies:** RCON crate.

- `mcs exec [command]`: Send via RCON, with setup prompts.

## Version 0.10.1: Console and Scheduling

**Goal:** Expand remote and auto features.

- `mcs console [server]`: Attach to running console.
- `mcs schedule [task] [interval]`: Basic jobs (e.g., backup daily, restart weekly).

## Version 0.11.0: Backup and Restore Basics

**Goal:** Robust data protection.

**Dependencies:** Compression crates; cloud APIs optional via plugins.

- `mcs backup [server] --full`: Compress everything, with incremental options.
- `mcs restore [archive]`: Rollback support.

## Version 0.11.1: Backup Enhancements

**Goal:** Polish backups.

- Cloud backups: To S3/Dropbox (plugin?).
- `mcs backup prune --keep [n]`: Auto-delete old backups.

## Version 0.12.0: Performance Tuning

**Goal:** Optimize server performance.

- `mcs tune`: Suggest JVM flags based on system resources.

## Version 0.13.0: Monitoring and Dashboard

**Goal:** Visual oversight.

**Dependencies:** TUI crate like ratatui.

- `mcs dashboard`: TUI for monitoring (CPU, players, logs).

## Version 0.14.0: Cloning

**Goal:** Quick duplication of setups.

- `mcs clone [source] [new]`: Duplicate setups with tweaks.

## Version 1.0.0: Stability Milestone

**Goal:** Production-ready with all core features polished; full testing, docs, and community prep.

**Notes:** No new features—just integration tests, bug fixes, and refinements across all prior additions. Release when everything feels solid. IMPORTANT: Make sure to prioritize lightweight-ness and speed!!
