
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
# retired; the engine self-hosts the csrsrv/ntdll import libs and reserves the
# arm64 2MB stack (.stack_reserve — llvm-dlltool collapses delay-imports so all
# deps load at once and LdrpLoadDll recursion overflows the default 1MB stack).
# The csrss link compiles ZERO C: csrss_rust_stub.c was a cmake-era cohabitation
# stub (an empty TU to force Rust-archive extraction under add_executable),
# vestigial under add_cargo_module — dropped from the cargo link (3199e96cca8),
# so its csrss_c OBJECT lib is gone too. The .c stays on disk (never delete C).
#
# The LEAN csrss_main_only feature ships ONLY `_main` +
# CsrpSetDefaultProcessHardErrorMode (NOT the csrsrv/csrlib/smlib/smss/basesrv
# modules) so csrss.exe does not carry its own #[no_mangle] copies of csrsrv.dll's
# data exports (_CsrApiPort etc.) that would shadow the cross-DLL imports and
# break data sharing (winsrv → CsrQueryApiPort → csrsrv.dll's _CsrApiPort reads 0
# because csrss.exe wrote its OWN copy at NtCreatePort time). csrss.rc is
# version-only (non-load-bearing) → not linked. Keep all .c on disk.

# ZERO cmake .a: libnt.a dropped — the native CRT startup _NtProcessStartup@4
# is now a Rust port in smss-csrss-rust (crate::entry_point). csrsrv/ntdll
# import libs are self-hosted in-cargo, so csrss.exe links no cmake .a at all.
add_cargo_module(csrss csrss-cargo csrss.exe
    KIND native
    BODY_CRATE smss-csrss-rust
    DESTINATION reactos/system32)
