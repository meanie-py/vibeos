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

add_registry_inf(netstack_reg.inf)
