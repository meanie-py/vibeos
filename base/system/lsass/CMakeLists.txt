
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

# FULL flip + CARGO-DRIVEN LINK. lsass.c is replaced by Rust (lsass-rust), and
# the lsass.exe LINK itself is now driven by cargo (lsass-cargo → the shared
# reactos-link engine, ModuleKind::Win32Exe) instead of cmake add_executable —
# the first of OUR translated spine binaries to move its link cmake→cargo.
# lsass-cargo's build.rs cargo-builds lsass-rust (.rust_staticlib) and links it
# with the CRT startup (libmsvcrtex.a → _wWinMainCRTStartup → wWinMain) + the
# cmake-built importlib group (advapi32/lsasrv/samsrv/msvcrt/kernel32/ntdll).
# Keep lsass.c on disk (never delete the C — translation philosophy).
#
# lsass.exe is the Local Security Authority Subsystem host on the security boot
# spine (smss/wininit → lsass → lsasrv/samsrv init; winlogon → LsaLogonUser
# authenticates against it), so `shell ready` proves the cargo-linked lsass runs.
#
# OWED: lsass.rc (pure version resource) is dropped by this first mover — re-host
# it via gen_lsass_res.py or a cmake rc target (non-load-bearing: version info
# only, no manifest/icons).

# lsass's 6 import libs are now GENERATED in-cargo from each DLL's .spec
# (lsass-cargo .import_lib) — no cmake reuse. oldnames.a (_name aliases) was
# dropped (probe: zero oldname refs in liblsass_rust.a, proven by the clean
# non-shared exe link). The CRT startup is crt-startup-rust via .msvcrtex_crt(),
# built in-cargo; it needs nothing cmake-built (the old libmsvcrtex.a anchor is gone
# with the C archive).

add_cargo_module(lsass lsass-cargo lsass.exe
    KIND win32exe
    BODY_CRATE lsass-rust
    DESTINATION reactos/system32
)
