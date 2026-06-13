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
# deterministically trips a BUGCHECK 0x50 during boot — but the bug is NOT in
# netstack. ROOT-CAUSED 2026-06-13 (still survives the crp merge): the hive-size
# shift moves a LATENT STORAGE-DRIVER bug onto an unmapped page. It is an async
# read-fault of paged pool (0xfffff8a0...) at a STABLE PC (0xfffff880055cd0x0)
# that fires right AFTER the uniata/scsiport IDE adapter-init completes (the
# adapter-init's IoConnectInterrupt fails ~33×, then an async callback faults).
# The PC does NOT move when scsiport-rust is rebuilt → the faulting code is in
# uniata.sys (C miniport) or a scsiport function linked before the init path,
# reached via the failed-interrupt adapter; ScsiPortIoTimer was RULED OUT (its
# entry instrumentation never fired). To finish: GDB-break at the faulting PC,
# read the register holding 0xfffff8a0..., confirm module (uniata vs scsiport),
# fix the bad pointer / pool-type / NULL-Interrupt[0] deref; THEN re-enable the
# catalog (add netstack_winsock.inf back above) + flip the usermode winsock test.
# See docs/engineering/translation-debugging-field-manual.md §9.9 and task 79.
