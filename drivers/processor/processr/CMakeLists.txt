# ============================================================================
# processr.sys — cargo-driven final link (sdk/lib/rust/processr-cargo)
# ============================================================================
# processr.c/misc.c/pnp.c are translated to Rust (sdk/lib/rust/processr-rust,
# kept on disk per the never-delete rule). Replaces the MODULE link +
# rust_processr custom_command. processr-cargo's build.rs (reactos-link,
# ModuleKind::KernelDll, user-target body + i386 -sse,-sse2) imports hal +
# ntoskrnl and reuses memcmp. Version resource (processr.rc) dropped; cpu.inf
# install stays.
add_cargo_module(processr processr-cargo processr.sys KIND kerneldll
    BODY_CRATE processr-rust DESTINATION reactos/system32/drivers)
add_driver_inf(processr cpu.inf)
