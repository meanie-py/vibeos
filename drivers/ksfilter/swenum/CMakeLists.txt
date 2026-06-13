
# ============================================================================
# swenum.sys — cargo-driven final link (sdk/lib/rust/swenum-cargo)
# ============================================================================
# swenum.c (the KS software bus enumerator) is translated to Rust
# (sdk/lib/rust/swenum-rust, kept on disk per the never-delete rule). Replaces
# the MODULE link + rust_swenum custom_command. swenum-cargo's build.rs
# (reactos-link, ModuleKind::KernelDll, user-target body + i386 -sse,-sse2)
# imports ks (ks.sys — the Ks* bus-enum entry points) + ntoskrnl + hal and
# reuses memcmp. Version resource (swenum.rc) dropped.
add_cargo_module(swenum swenum-cargo swenum.sys KIND kerneldll
    BODY_CRATE swenum-rust DESTINATION reactos/system32/drivers)
