
spec2def(hidparse.sys hidparse.spec ADD_IMPORTLIB)

# ============================================================================
# hidparse.sys — cargo-driven final link (sdk/lib/rust/hidparse-cargo)
# ============================================================================
# hidparse.c (DriverEntry + the 5 hidparser_km callbacks) is translated to Rust
# (sdk/lib/rust/hidparse-rust); the hidparser_km report-descriptor parser
# (sdk/lib/drivers/hidparser/{api,context,hidp,hidparser,parser}.c) is
# translated to Rust (sdk/lib/rust/hidparser-rust, a sibling static lib). Both
# kept on disk per the never-delete rule. Replaces the MODULE link +
# rust_hidparse / rust_hidparser custom_commands. hidparse-cargo's build.rs
# (reactos-link, ModuleKind::KernelDll, user-target body + i386 -sse,-sse2)
# links hidparser-rust as a sibling rust_lib, EXPORTS the HidP_* API via
# spec2def(hidparse.spec), imports ntoskrnl, and reuses memcmp. Version resource
# (hidparse.rc) dropped. hid.dll links a user-mode build of the same crate. The
# spec2def(... ADD_IMPORTLIB) above STAYS (consumers link the libhidparse implib).
add_cargo_module(hidparse hidparse-cargo hidparse.sys KIND kerneldll
    BODY_CRATES hidparse-rust hidparser-rust DESTINATION reactos/system32/drivers)
