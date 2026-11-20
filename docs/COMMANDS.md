# azurewrath Commands

Author: TwoTimeMilestone4  
Repository: https://github.com/TwoTimeMilestone4/azurewrath-init

All user-visible output starts with `Azurewrath:`. Quotes rotate and never repeat the same line twice in a row. Full quote list: [QUOTES.md](QUOTES.md).

| Command | SysV Equivalent | Example output |
|---------|-----------------|----------------|
| `azurewrath sow <service>` | `update-rc.d <service> enable` / `chkconfig on` | `Azurewrath: Sow dbus - "Grow, so you may serve me."` |
| `azurewrath uproot <service>` | `update-rc.d <service> disable` / `chkconfig off` | `Azurewrath: Uproot dbus - "Be gone!"` |
| `azurewrath bloom <service>` | `service <name> start` | `Azurewrath: Bloom dbus - "Blossom, be reborn!"` |
| `azurewrath wilt <service>` | `service <name> stop` | `Azurewrath: Wilt dbus - "CRUMBLE!"` |
| `azurewrath regrow <service>` | `service <name> restart` | `Azurewrath: Regrow dbus - "Rise, ONCE MORE!"` |
| `azurewrath revive` | `shutdown -r now` | `Azurewrath: Revive - "From the depths, I ARISE AGAIN!"` |
| `azurewrath reap` | `shutdown -h now` | `Azurewrath: Reap - "And I'll do it, AGAIN!"` |
| `azurewrath status [service]` | `service status` / `runlevel` | `Azurewrath: Status - "My heart stopped its beat, and soon, yours will too..."` |
| `azurewrath list` | `ls /etc/init.d` + enabled check | `Azurewrath: "A revenant to that accursed village, I arise..."` |
| `azurewrath logs [service]` | `cat /var/log/...` | `Azurewrath: "I thank the one who granted me this... retribution."` |
| `azurewrath reload` | `telinit q` | `Azurewrath: Reload - "Rise, ONCE MORE!"` |
| `azurewrath telinit <N>` | `telinit <N>` | `Azurewrath: Telinit 5 - "Rise, once more, Azurewrath."` |
| `azurewrath runlevel [N]` | `runlevel` / `telinit` | `Azurewrath: Runlevel - "Was the second life worth it?"` |
| `azurewrath session list` | `loginctl list-sessions` | `Azurewrath: sessions (via spoofd):` |
| `azurewrath coredump list` | `coredumpctl list` | `Azurewrath: coredumps:` |
| `azurewrath version` | - | `Azurewrath: azurewrath 0.1.0` |
| `azurewrath help` | - | clap help |

## systemctl shim

The `systemctl` binary (same crate) translates:

| systemctl | azurewrath |
|-----------|------------|
| start | bloom |
| stop | wilt |
| restart | regrow |
| enable | sow |
| disable | uproot |
| status / is-active / is-enabled | status |
| daemon-reload | reload |
| poweroff / halt | reap |
| reboot | revive |
| --user ... | aw-userd |

## Boot / Shutdown examples

```
Azurewrath: "Lament, mourn and despair."
Azurewrath: Sow dbus - "Grow, so you may serve me."
Azurewrath: Bloom dbus - "Blossom, be reborn!"
Azurewrath: "Hahah! I'll kill you again, and AGAIN... AND AGAIN!!"
```
