
# ============================================================================
# kbdclass.sys — cargo-driven final link (sdk/lib/rust/kbdclass-cargo via reactos-link)
# ============================================================================
# The functional C sources (kbdclass.c + misc.c) were translated to Rust
# (sdk/lib/rust/kbdclass-rust, kept on disk per the never-delete rule); the
# Rust #[no_mangle] DriverEntry is the PE entry point. Until now kbdclass-rust
# was a workspace member but UNREFERENCED — kbdclass.sys was still the pure-C
# add_library(MODULE). This cut both activates the Rust keyboard class driver
# and moves the final link off cmake onto cargo (kbdclass-cargo's build.rs →
# reactos-link, ModuleKind::KernelDll, entry=DriverEntry, no exports), linking
# kbdclass-rust with the ntoskrnl/hal import libs into the shipping PE.
#
# guid.c is NOT linked: kbdclass-rust defines its sole interface GUID
# (GUID_DEVINTERFACE_KEYBOARD) as a Rust-internal static used via addr_of!, so
# the C guid.c symbol is unreferenced dead residue (kept on disk). The version
# resource (kbdclass.rc) is dropped, as in the other cargo cuts. PSEH_LIB is
# dropped: the translation's _SEH2 handlers are dead code (DUMMY_PSEH).
add_cargo_module(kbdclass kbdclass-cargo kbdclass.sys KIND kerneldll
    BODY_CRATE kbdclass-rust DESTINATION reactos/system32/drivers)

# Registry setup is independent of the link and stays in cmake.
add_registry_inf(kbdclass_reg.inf)
