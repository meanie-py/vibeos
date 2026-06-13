
spec2def(usbd.sys usbd.spec ADD_IMPORTLIB)

# ============================================================================
# usbd.sys — cargo-driven final link (sdk/lib/rust/usbd-cargo)
# ============================================================================
# usbd.c (the USBD library driver) is translated to Rust (sdk/lib/rust/usbd-rust,
# kept on disk per the never-delete rule; USB DDK types via usb-sys). Replaces
# the MODULE link + rust_usbd custom_command. usbd-cargo's build.rs
# (reactos-link, ModuleKind::KernelDll, user-target body + i386 -sse,-sse2)
# EXPORTS the USBD_* API via spec2def(usbd.spec), imports ntoskrnl + hal and
# reuses memcmp. Version resource (usbd.rc) dropped. The spec2def(... ADD_IMPORTLIB)
# above STAYS (consumers link the libusbd import lib, independent of this target).
add_cargo_module(usbd usbd-cargo usbd.sys KIND kerneldll
    BODY_CRATE usbd-rust DESTINATION reactos/system32/drivers
    EXTRA_DEPENDS memcmp)
