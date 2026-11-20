# Gentoo ebuild for azurewrath-init

Author: TwoTimeMilestone4  
Repository: https://github.com/TwoTimeMilestone4/azurewrath-init

## sys-apps/azurewrath-init

```ebuild
# Copyright 2026 TwoTimeMilestone4
# Distributed under the terms of the Apache-2.0 license

EAPI=8

DESCRIPTION="SysVinit-compatible init written in Rust with systemd spoofing"
HOMEPAGE="https://github.com/TwoTimeMilestone4/azurewrath-init"
SRC_URI="https://github.com/TwoTimeMilestone4/azurewrath-init/archive/refs/tags/v${PV}.tar.gz"

LICENSE="Apache-2.0"
SLOT="0"
KEYWORDS="~amd64 ~arm64"
IUSE="+systemd-spoof"

DEPEND="
    virtual/rust
    dev-libs/libdbus
"
RDEPEND="
    ${DEPEND}
    sys-apps/dbus
    !sys-apps/sysvinit
    !sys-apps/systemd
"

src_compile() {
    cargo build --release || die
}

src_install() {
    dobin target/release/azurewrath
    dosbin target/release/aw-pid1
    dosbin target/release/aw-spoofd
    dosbin target/release/aw-userd
    dosbin target/release/systemctl

    # Symlinks
    dosym aw-pid1 /sbin/init
    dosym azurewrath /sbin/halt
    dosym azurewrath /sbin/poweroff
    dosym azurewrath /sbin/reboot
    dosym azurewrath /sbin/shutdown

    # Examples
    insinto /usr/share/azurewrath-init/examples
    doins -r examples/*

    # Docs
    dodoc README.md docs/*
}

pkg_postinst() {
    elog "azurewrath-init installed."
    elog "It is SysVinit-compatible (uses config format, written from scratch in Rust)."
    elog "Set it as PID 1 via bootloader or OpenRC alternative."
    elog "For GNOME 49+ and Plasma Login Manager, enable spoofd in runlevel 5."
}
```

## virtual/service-manager

Provide alternative to sysvinit:

```
RDEPEND="
    || (
        sys-apps/azurewrath-init
        sys-apps/sysvinit
        sys-apps/openrc
    )
"
```

Block `sys-apps/sysvinit` when using azurewrath-init.
