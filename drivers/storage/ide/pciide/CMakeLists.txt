# ============================================================================
# pciide.sys — cargo-driven final link (sdk/lib/rust/pciide-cargo via reactos-link)
# ============================================================================
# pciide.c is translated to Rust (sdk/lib/rust/pciide-rust, kept on disk per the
# never-delete rule); the Rust #[no_mangle] DriverEntry is the PE entry point.
# The final pciide.sys link is no longer cmake's add_library(MODULE) — it is
# driven by cargo (pciide-cargo's build.rs → reactos-link, ModuleKind::KernelDll,
# entry=DriverEntry@8, no exports), linking pciide-rust with the pciidex +
# ntoskrnl import libs (PciIdeXInitialize/PciIdeXGetBusData are pciidex exports).
# The version resource (pciide.rc, and its .rsrc-discardable pefixup) is dropped
# as in the other cargo cuts. Wrapped in ENABLE_UNIATA (upstream addition, since
# our fork: uniata is the alternative IDE bus driver and the two are mutually
# exclusive) -- this is a build-option gate, not a code change, so it composes
# fine with the Rust cohab.
if(NOT ENABLE_UNIATA)
add_cargo_module(pciide pciide-cargo pciide.sys KIND kerneldll
    BODY_CRATE pciide-rust DESTINATION reactos/system32/drivers
    EXTRA_DEPENDS pciidex)
endif()
add_registry_inf(pciide_reg.inf)
