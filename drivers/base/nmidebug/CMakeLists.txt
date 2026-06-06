
# ============================================================================
# nmidebug.sys — cargo-driven final link (sdk/lib/rust/nmidebug-cargo via reactos-link)
# ============================================================================
# nmidebug.c is translated to Rust (sdk/lib/rust/nmidebug-rust, kept on disk per
# the never-delete rule); its DriverEntry is the PE entry point. The final
# nmidebug.sys link is no longer cmake's add_library(MODULE) — it is driven by
# cargo (nmidebug-cargo's build.rs → reactos-link, ModuleKind::KernelDll,
# entry=DriverEntry, no exports). add_cargo_module builds nmidebug-rust, links it
# with the ntoskrnl/hal import libs (KiBugCheckData/KiEnableTimerWatchdog come in
# via raw-dylib in the Rust crate), applies the kernel-dll post-link and
# registers it on the boot media. The version resource (nmidebug.rc) is dropped,
# as in the bootvid/kdcom cargo cuts. nmidebug.c/nmidebug.rc kept on disk, no
# longer compiled.
add_cargo_module(nmidebug nmidebug-cargo nmidebug.sys KIND kerneldll
    BODY_CRATE nmidebug-rust DESTINATION reactos/system32/drivers)
