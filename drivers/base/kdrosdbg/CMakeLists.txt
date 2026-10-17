spec2def(kdcom.dll kdrosdbg.spec ADD_IMPORTLIB)

# ============================================================================
# kdcom.dll — cargo-driven final link (sdk/lib/rust/kdcom-cargo via reactos-link)
# ============================================================================
# kdrosdbg.c is translated to Rust (sdk/lib/rust/kdcom-rust, kept on disk per
# the never-delete rule). The 8 kdrosdbg.spec exports come from libkdcom_rust.a.
#
# The final kdcom.dll link is no longer cmake's add_library(MODULE) — it is
# driven by cargo (kdcom-cargo's build.rs → reactos-link, ModuleKind::KernelDll)
# which builds kdcom-rust + links it with cportlib/memcmp + the ntoskrnl import
# lib into the shipping PE. cmake keeps the import lib above (ntoskrnl/ntkrnlmp
# import kdcom) + the boot-media registration + the kernel-dll post-link, all
# handled by add_cargo_kernel_dll (sdk/cmake/gcc.cmake). First module whose
# cmake link is fully retired in favour of the cargo-drive.
# EXTRA_DEPENDS: cmake targets whose .a the cargo link reuses — cportlib
# (pulls rust_cport → libcport_rust.a, cport.c now being Rust) + memcmp — so
# `ninja kdcom` builds them before the opaque cargo link.
# The version resource: cmake compiles it in this OBJECT library, whose windres
# command the cargo link replays.
add_cargo_kernel_dll(kdcom kdcom-cargo kdcom.dll BODY_CRATE kdcom-rust
    EXTRA_DEPENDS psdk cportlib memcmp)
