# ============================================================================
# kbdclass.sys — cargo-driven final link (sdk/lib/rust/kbdclass-cargo)
# ============================================================================
# kbdclass.c + misc.c are translated to Rust (sdk/lib/rust/kbdclass-rust, kept
# on disk per the never-delete rule); the Rust #[no_mangle] DriverEntry is the
# PE entry point. The final kbdclass.sys link is no longer cmake's
# add_library(MODULE) — it is driven by cargo (kbdclass-cargo's build.rs →
# reactos-link, ModuleKind::KernelDll, entry=DriverEntry@8, no exports).
#
# guid.c is dropped (dead residue): kbdclass-rust self-provides
# GUID_DEVINTERFACE_KEYBOARD as a Rust-internal static. The version resource
# (kbdclass.rc) is dropped as in the other cargo cuts; the registry inf stays.
add_cargo_module(kbdclass kbdclass-cargo kbdclass.sys KIND kerneldll
    BODY_CRATE kbdclass-rust DESTINATION reactos/system32/drivers)

add_registry_inf(kbdclass_reg.inf)
