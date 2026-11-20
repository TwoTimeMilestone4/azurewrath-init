# Plasma Login Manager (PLM) and azurewrath-init

Author: TwoTimeMilestone4

## Background

Plasma Login Manager (Plasma 6.6+) requires systemd. It talks to `org.freedesktop.systemd1` and expects `plasmalogin.service`. It also wants exclusive access to tty1.

## How azurewrath-init handles it

1. `aw-spoofd` maps `plasmalogin.service` → `/etc/init.d/plasmalogin`
2. When StartUnit is called for plasmalogin.service:
   - If a getty is running on tty1, it is stopped first
   - PLM is started on tty1 (`plasmalogin --tty1`)
3. When StopUnit is called (PLM exits):
   - getty is restarted on tty1

Same logic applies to `gdm.service`, `sddm.service`, etc.

## Setup

```bash
# Enable in runlevel 5
azurewrath sow plasmalogin
azurewrath sow spoofd

# Or manually link
ln -s ../init.d/plasmalogin /etc/rc5.d/S99plasmalogin
ln -s ../init.d/spoofd /etc/rc5.d/S98spoofd
```

Runlevel 5 must start spoofd before the display manager.
