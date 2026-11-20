# azurewrath-init

**SysVinit-compatible** init system written from scratch in Rust by TwoTimeMilestone4.

Author: TwoTimeMilestone4  
Repository: https://github.com/TwoTimeMilestone4/azurewrath-init  
License: Apache-2.0  
Copyright 2026 TwoTimeMilestone4

## Overview

azurewrath-init is a modern PID 1 implementation that is **SysVinit-compatible**. It uses the SysVinit configuration format (`/etc/inittab`, `/etc/init.d/*`, runlevel directories) but is a complete clean-room reimplementation written in Rust. It is **not** a SysVinit fork or clone.

Key features:

- Non-blocking PID 1 architecture using `signalfd` + `epoll` event loop
- Full SysVinit config compatibility (inittab actions, runlevels 0-6, rc scripts)
- systemd spoofing daemon (`aw-spoofd`) so GNOME 49+, Plasma Login Manager (PLM / Plasma 6.6), and DrKonqi work without patches
- User session manager (`aw-userd`)
- `azurewrath` CLI with Forsaken-inspired skill names
- `systemctl` compatibility shim

## Positioning

**SysVinit-compatible**, not a fork.  
Uses SysVinit config format, but written from scratch in Rust by TwoTimeMilestone4.

## Architecture

```
PID 1 (aw-pid1)
  ├── signalfd + epoll non-blocking event loop
  ├── inittab parser + runlevel manager
  ├── async child reaping + respawn
  └── starts aw-spoofd on graphical runlevels

aw-spoofd (systemd + logind spoof)
  ├── org.freedesktop.systemd1.Manager
  ├── org.freedesktop.login1.Manager / Session / Seat / User
  ├── sd_notify listener
  ├── coredump spoof for DrKonqi
  └── plasmalogin.service / gdm.service → SysV script mapping + tty1 handling

aw-userd
  └── per-user service manager + XDG autostart

azurewrath CLI
  └── sow / bloom / wilt / regrow / revive / reap / ...
```

## Quick Start

```bash
# Build
cargo build --release

# Install (as root)
cp target/release/aw-pid1 /sbin/init
cp target/release/azurewrath /usr/bin/azurewrath
cp target/release/systemctl /usr/bin/systemctl   # shim
# Create symlinks for halt, reboot, poweroff, shutdown
```

See `docs/` for Gentoo ebuild, GNOME 49 notes, PLM notes, and full command reference.

## Commands

See [docs/COMMANDS.md](docs/COMMANDS.md) for the complete table.

## License

Apache-2.0. Copyright 2026 TwoTimeMilestone4.
