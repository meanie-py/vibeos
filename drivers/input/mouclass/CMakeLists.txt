# ============================================================================
# mouclass.sys — cargo-driven final link (sdk/lib/rust/mouclass-cargo)
# ============================================================================
# mouclass.c + misc.c are translated to Rust (sdk/lib/rust/mouclass-rust, kept
# on disk per the never-delete rule); the Rust #[no_mangle] DriverEntry is the
# PE entry point. The final mouclass.sys link is no longer cmake's
# add_library(MODULE) — it is driven by cargo (mouclass-cargo's build.rs →
# reactos-link, ModuleKind::KernelDll, entry=DriverEntry@8, no exports).
#
# guid.c is dropped (dead residue): mouclass-rust self-provides
# GUID_DEVINTERFACE_MOUSE as a Rust-internal static. The version resource
# (mouclass.rc) is dropped as in the other cargo cuts; the registry inf stays.
add_cargo_module(mouclass mouclass-cargo mouclass.sys KIND kerneldll
    BODY_CRATE mouclass-rust DESTINATION reactos/system32/drivers)

add_registry_inf(mouclass_reg.inf)
