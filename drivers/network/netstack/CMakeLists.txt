# ============================================================================
# netstack.sys — smoltcp-based network stack (L4 Rust rewrite, NOT a C
# translation). docs/engineering/smoltcp-netstack-design.md.
# ============================================================================
# Cargo-driven kernel driver: the Rust body crate netstack-rust (smoltcp +
# virtio-drivers) provides DriverEntry; netstack-cargo's build.rs drives the
# final PE link via reactos-link (KernelDll, entry=DriverEntry, no exports yet
# — the TDI device interface arrives at M4). System-start, non-PnP driver.
add_cargo_module(netstack netstack-cargo netstack.sys KIND kerneldll
    BODY_CRATE netstack-rust DESTINATION reactos/system32/drivers)

# netstack_reg.inf: the netstack.sys service (Start=1).
add_registry_inf(netstack_reg.inf)

# netstack_winsock.inf: the usermode winsock enablement that network setup would
# normally write from media/inf/nettcpip.inf (the LiveCD never runs setup). Two
# halves, both required, or usermode socket() fails:
#   (1) the Winsock2 protocol catalog (msafd providers) — ws2_32 finds a provider;
#   (2) the Winsock-v1 transport map (wshtcpip + Mapping) — msafd!SockGetTdiName
#       resolves (af,type,proto) -> \Device\Tcp|Udp|RawIp (which netstack owns).
# On amd64 this gives a full ws2_32 -> msafd -> AFD -> netstack -> smoltcp -> slirp
# round-trip (tests/qemu/test_net.py::test_usermode_winsock_roundtrip, GREEN).
#
# amd64-ONLY for now: enabling it on i386 advances boot far enough to reach a
# KNOWN PRE-EXISTING ntoskrnl Mm bug — a BUGCHECK 0x1E (0xc0000005) at services.exe
# creation, an unmapped i386 session-space pointer read (0xf6xxxxxx) during
# MmSessionCreate-family code. That bug is NOT in netstack (catalog-only i386
# boots green; the netstack TDI/query code is identical across arches) — it is the
# same i386 session-space Mm fault the kernel32-wave/Mm-0x1E lane is fixing, just
# newly *reached* because working winsock lets boot get to service startup. Re-
# enable for i386/arm64 once that ntoskrnl Mm 0x1E lands fixed.
# See docs/engineering/translation-debugging-field-manual.md and task 77.
if(ARCH STREQUAL "amd64")
    add_registry_inf(netstack_winsock.inf)
endif()
