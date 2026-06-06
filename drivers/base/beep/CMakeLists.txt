# ============================================================================
# beep.sys — cargo-driven final link (sdk/lib/rust/beep-cargo via reactos-link)
# ============================================================================
# beep.c is translated to Rust (sdk/lib/rust/beep-rust, kept on disk per the
# never-delete rule); the Rust #[no_mangle] DriverEntry is the PE entry point.
# The final beep.sys link is no longer cmake's add_library(MODULE) — it is
# driven by cargo (beep-cargo's build.rs → reactos-link, ModuleKind::KernelDll,
# entry=DriverEntry@8, no exports), linking beep-rust with the ntoskrnl/hal
# import libs (HalMakeBeep is a hal export). The version resource (beep.rc) is
# dropped as in the other cargo cuts; the registry inf stays.
add_cargo_module(beep beep-cargo beep.sys KIND kerneldll
    BODY_CRATE beep-rust DESTINATION reactos/system32/drivers)

add_registry_inf(beep_reg.inf)
