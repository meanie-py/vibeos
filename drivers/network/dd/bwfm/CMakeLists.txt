# ============================================================================
# bwfm.sys — the Raspberry Pis' CYW43455 Wi-Fi (arm64), cargo-driven link
# ============================================================================
# The Pi 5's Wi-Fi SDIO host (ACPI SDC1, child WLAN), published by the arm64
# HAL's ACPI PnP root as VIBEOS\CYW43455 (hal-rust/src/arm64/pnpdd.rs) and bound
# through the critical device database (boot/bootdata/hivesys.inf). Rust with
# no C original: sdk/lib/rust/bwfm-rust (body) + bwfm-cargo (link). Design:
# docs/engineering/pi-wifi-design.md.
add_cargo_module(bwfm bwfm-cargo bwfm.sys KIND kerneldll
    BODY_CRATE bwfm-rust DESTINATION reactos/system32/drivers)
