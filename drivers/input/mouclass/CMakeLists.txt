
# ============================================================================
# mouclass.sys — cargo-driven final link (sdk/lib/rust/mouclass-cargo via reactos-link)
# ============================================================================
# The functional C sources (mouclass.c + misc.c) were translated to Rust
# (sdk/lib/rust/mouclass-rust, kept on disk per the never-delete rule); the
# Rust #[no_mangle] DriverEntry is the PE entry point. Until now mouclass-rust
# was a workspace member but UNREFERENCED — mouclass.sys was still the pure-C
# add_library(MODULE). This cut both activates the Rust mouse class driver and
# moves the final link off cmake onto cargo (mouclass-cargo's build.rs →
# reactos-link, ModuleKind::KernelDll, entry=DriverEntry, no exports), linking
# mouclass-rust with the ntoskrnl/hal import libs into the shipping PE.
#
# guid.c is NOT linked: mouclass-rust defines its sole interface GUID
# (GUID_DEVINTERFACE_MOUSE) as a Rust-internal static used via addr_of!, so the
# C guid.c symbol is unreferenced dead residue (kept on disk). The version
# resource (mouclass.rc) is dropped, as in the other cargo cuts. PSEH_LIB is
# dropped: the translation's _SEH2 handlers are dead code (DUMMY_PSEH).
add_cargo_module(mouclass mouclass-cargo mouclass.sys KIND kerneldll
    BODY_CRATE mouclass-rust DESTINATION reactos/system32/drivers)

# Registry setup is independent of the link and stays in cmake.
add_registry_inf(mouclass_reg.inf)
