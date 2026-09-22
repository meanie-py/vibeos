
spec2def(kdgdb.dll kdgdb.spec)

# ============================================================================
# kdcom.dll (kdgdb GDB-stub KD transport) — cargo-driven final link
# ============================================================================
# The kdgdb C sources are translated to Rust (sdk/lib/rust/kdgdb-rust, kept on
# disk per the never-delete rule). The 8 kdgdb.spec exports come from
# libkdgdb_rust.a; the final kdcom.dll link is driven by cargo (kdgdb-cargo's
# build.rs -> reactos-link, ModuleKind::KernelDll), which links kdgdb-rust with
# cportlib (Cp* serial) + libcntpr (CRT: sprintf/_snprintf/strcmp/strstr/
# _strupr/atol/strlen) + memcmp + the ntoskrnl/hal import libs. Mirrors the
# kdrosdbg (kdcom-cargo) cohab. This whole subdir is only added under
# _WINKD_ && GDB (see drivers/base/CMakeLists.txt). kdgdb is i386 + amd64 only
# (no arm64: i386_sup.c / amd64_sup.c, "TODO: ARM" in the C).
#
#   gdb_input.c gdb_receive.c gdb_send.c kdcom.c kdpacket.c utils.c
#   i386_sup.c / amd64_sup.c  -- replaced by Rust (kdgdb-rust); kept on disk.
add_cargo_kernel_dll(kdcom kdgdb-cargo kdcom.dll BODY_CRATE kdgdb-rust
    EXTRA_DEPENDS cportlib libcntpr memcmp)
