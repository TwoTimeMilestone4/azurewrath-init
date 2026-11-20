# GNOME 49 and azurewrath-init

Author: TwoTimeMilestone4

## Background

GNOME 49 removed the non-systemd fallback code paths in `gnome-session` and related components. Previously, GNOME could run under SysVinit or other inits by detecting the absence of systemd and using alternative session startup.

In GNOME 49+, the code assumes `org.freedesktop.systemd1` and `org.freedesktop.login1` are always present.

## How azurewrath-init solves this

`aw-spoofd` implements the required D-Bus interfaces:

- `org.freedesktop.systemd1.Manager` (StartUnit, ListUnits, GetUnitFileState, etc.)
- `org.freedesktop.login1.Manager` + Session/Seat/User objects with sane properties (State=active, Type=wayland, Class=user, etc.)
- sd_notify protocol on `/run/systemd/notify`
- Runtime directories under `/run/systemd/`

Because spoofd answers the D-Bus calls that GNOME makes, **no patching of GNOME is required**.

## Setup

1. Install azurewrath-init as PID 1
2. Ensure dbus is running
3. In runlevel 5, start `/etc/init.d/spoofd`
4. Start gdm via `/etc/init.d/gdm` or let spoofd map `gdm.service`

gnome-session, gsd-*, etc. will be launched by `aw-userd` via XDG autostart.
