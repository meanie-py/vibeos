# ============================================================================
# condrv.sys — cargo-driven final link (sdk/lib/rust/condrv-cargo via reactos-link)
# ============================================================================
# condrv.c/control.c/dispatch.c are translated to Rust (sdk/lib/rust/condrv-rust,
# kept on disk per the never-delete rule); the Rust #[no_mangle] DriverEntry is
# the PE entry point. The final condrv.sys link is no longer cmake's
# add_library(MODULE) — it is driven by cargo (condrv-cargo's build.rs →
# reactos-link, ModuleKind::KernelDll, entry=DriverEntry@8, no exports). The
# version resource (condrv.rc) is dropped as in the other cargo cuts.
add_cargo_module(condrv condrv-cargo condrv.sys KIND kerneldll
    BODY_CRATE condrv-rust DESTINATION reactos/system32/drivers)
