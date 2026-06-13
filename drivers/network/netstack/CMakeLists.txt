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

# netstack_winsock.inf (PRESERVED BUT NOT REGISTERED): the Winsock2 protocol
# catalog (msafd providers for TCP/UDP/RAW). This is REQUIRED for usermode ws2_32
# socket() to find a provider and reach AFD/netstack (without it socket() returns
# WSAEINVAL). It is normally written by network setup (media/inf/nettcpip.inf),
# which the LiveCD never runs. HOWEVER: baking it into the shared system hive
# deterministically trips a BUGCHECK 0x50 in an early kernel driver during boot
# (PC in the 0xfffff880 driver region, read-fault on a 0xfffff8a0 paged-pool VA),
# BEFORE shell ready — the same amd64 layout-sensitive 0x50 family as the known
# taskbar flake, made DETERMINISTIC by the hive-size/pool-layout shift this adds.
# So enabling it breaks boot on ALL arches. Re-enable (add it back to the
# add_registry_inf call above) once that latent 0x50 is root-caused — this gives
# a deterministic repro of it. See docs/engineering/translation-debugging-field-manual.md.
