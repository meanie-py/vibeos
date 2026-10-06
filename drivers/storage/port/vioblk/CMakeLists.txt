
# ============================================================================
# vioblk.sys — virtio-blk SCSIPORT miniport, written in Rust (sdk/lib/rust/vioblk-rust).
# ============================================================================
# There is no C body: before this the x86 kernel had no virtio-blk driver at all.
# Cargo-driven link (sdk/lib/rust/vioblk-cargo), like netstack.sys and for the same
# reason: virtio-drivers' zerocopy dependency only builds for i386 under the build-std
# soft-float kernel target. Boot-start (vioblk_reg.inf) and bound to
# PCI\VEN_1AF4&DEV_1001/1042 through the CriticalDeviceDatabase (boot/bootdata/hivesys.inf).
add_cargo_module(vioblk vioblk-cargo vioblk.sys KIND kerneldll
    BODY_CRATE vioblk-rust DESTINATION reactos/system32/drivers)

add_registry_inf(vioblk_reg.inf)
