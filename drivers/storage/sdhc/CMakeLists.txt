# ============================================================================
# sdhc.sys — SD host controller driver (arm64), cargo-driven link
# ============================================================================
# The Raspberry Pi 4's EMMC2 card slot, published by the arm64 HAL's ACPI PnP
# root as ACPI\BRCME88C (hal-rust/src/arm64/pnpdd.rs) and bound through the
# critical device database (boot/bootdata/hivesys.inf). The driver is Rust
# with no C original: sdk/lib/rust/sdhc-rust (body) + sdhc-cargo (link).
add_cargo_module(sdhc sdhc-cargo sdhc.sys KIND kerneldll
    BODY_CRATE sdhc-rust DESTINATION reactos/system32/drivers)
