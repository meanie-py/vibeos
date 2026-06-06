
# ============================================================================
# null.sys — cargo-driven final link (sdk/lib/rust/null-cargo via reactos-link)
# ============================================================================
# null.c is translated to Rust (sdk/lib/rust/null-rust, kept on disk per the
# never-delete rule); its DriverEntry is the PE entry point. The final null.sys
# link is no longer cmake's add_library(MODULE) — it is driven by cargo
# (null-cargo's build.rs → reactos-link, ModuleKind::KernelDll, entry=DriverEntry,
# no exports — a kernelmodedriver is a kerneldll whose entry is DriverEntry@8).
# add_cargo_module builds null-rust, links it with the ntoskrnl/hal import libs
# into the shipping PE, applies the kernel-dll post-link (native-pefixup +
# i386 forwarder fix) and registers it on the boot media. The version resource
# (null.rc) is dropped, exactly as in the bootvid/kdcom cargo cuts. null.c/null.rc
# are kept on disk but no longer compiled.
add_cargo_module(null null-cargo null.sys KIND kerneldll
    BODY_CRATE null-rust DESTINATION reactos/system32/drivers)
