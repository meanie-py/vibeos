
include_directories(
    ${REACTOS_SOURCE_DIR}/subsystems/win32/csrsrv/include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

# --- CARGO-DRIVEN LINK + FULL Rust flip (smss-csrss-rust, csrss_main_only) ----
# csrss.c is flipped to Rust (smss-csrss-rust/src/csrss.rs). csrss.exe's LINK is
# now driven by cargo (csrss-cargo → reactos-link ModuleKind::NativeExe, native
# subsystem, entry _NtProcessStartup@4) — the LAST csrss-spine binary moved
# cmake→cargo (after smss + the csrsrv/winsrv/basesrv DLLs), and (like basesrv)
# the one still on the legacy pattern: cmake `add_executable(csrss)` + a
# standalone `add_custom_command` cargo build the link reused. That machinery is
# retired; the engine self-hosts the csrsrv/ntdll import libs, compiles the
# csrss_rust_stub.c residue IN-cargo (c_from_compile_db), and reserves the arm64
# 2MB stack (.stack_reserve — llvm-dlltool collapses delay-imports so all deps
# load at once and LdrpLoadDll recursion overflows the default 1MB stack).
#
# The LEAN csrss_main_only feature ships ONLY `_main` +
# CsrpSetDefaultProcessHardErrorMode (NOT the csrsrv/csrlib/smlib/smss/basesrv
# modules) so csrss.exe does not carry its own #[no_mangle] copies of csrsrv.dll's
# data exports (_CsrApiPort etc.) that would shadow the cross-DLL imports and
# break data sharing (winsrv → CsrQueryApiPort → csrsrv.dll's _CsrApiPort reads 0
# because csrss.exe wrote its OWN copy at NtCreatePort time). csrss.rc is
# version-only (non-load-bearing) → not linked. Keep all .c on disk.

# csrss_rust_stub.c (macro/intrinsic shim) is compiled IN-cargo by the cargo link
# (c_from_compile_db, CMakeFiles/csrss_c.dir marker). This OBJECT lib's objects
# are NOT linked here — it exists only to populate compile_commands.json.
# nativecui adds no UNICODE; the stub is self-contained (no SDK headers, no PCH).
add_library(csrss_c OBJECT csrss_rust_stub.c)
set_source_files_properties(csrss_rust_stub.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON)

# libnt.a (native CRT startup, _NtProcessStartup@4) reused, as a FILE path for
# parallel-build ordering. csrsrv/ntdll import libs are self-hosted in-cargo.
set(CSRSS_REUSED
    ${CMAKE_BINARY_DIR}/sdk/lib/nt/libnt.a)

add_cargo_module(csrss csrss-cargo csrss.exe
    KIND native
    BODY_CRATE smss-csrss-rust
    DESTINATION reactos/system32
    EXTRA_DEPENDS csrss_c ${CSRSS_REUSED})
