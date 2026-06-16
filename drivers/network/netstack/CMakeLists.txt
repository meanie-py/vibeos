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
# This gives a full ws2_32 -> msafd -> AFD -> netstack -> smoltcp -> slirp
# round-trip (tests/qemu/test_net.py::test_usermode_winsock_roundtrip, GREEN).
#
# All 3 arches: this was amd64-only while working winsock advanced i386 boot to
# services.exe and hit a BUGCHECK 0x1E. That 0x1E was NOT in netstack and NOT an
# Mm bug — it was kernel32 FindFirstFileExW cast-before-offset pointer arith
# (`(p as PFIND_FILE_DATA).offset(1)` advances by sizeof(FIND_FILE_DATA) ~16KB
# instead of the handle size, overrunning the FindFileData buffer into reserved-
# uncommitted process heap; the dir-fill memset then AVs at the page boundary).
# Fixed in the kernel32 wave, landed on continue-rust-port (15176738c02). With
# that fix present, winsock is enabled on i386/amd64/arm64 alike.
# See docs/engineering/translation-debugging-field-manual.md and task 77.
add_registry_inf(netstack_winsock.inf)
