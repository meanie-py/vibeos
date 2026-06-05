
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

# The importlib/CRT .a the cargo link reuses live in dll/ + sdk/lib/crt, which
# cmake configures AFTER base/ — so their TARGET names aren't defined here.
# DEPENDS on the .a FILE paths instead (order-independent; ninja edges each to
# its producer) so a parallel `ninja livecd` builds them before the cargo link.
set(LSASS_REUSED_LIBS
    ${CMAKE_BINARY_DIR}/dll/win32/advapi32/CMakeFiles/libadvapi32.dir/libadvapi32.a
    ${CMAKE_BINARY_DIR}/dll/win32/lsasrv/CMakeFiles/liblsasrv.dir/liblsasrv.a
    ${CMAKE_BINARY_DIR}/dll/win32/samsrv/CMakeFiles/libsamsrv.dir/libsamsrv.a
    ${CMAKE_BINARY_DIR}/dll/win32/msvcrt/CMakeFiles/libmsvcrt.dir/libmsvcrt.a
    ${CMAKE_BINARY_DIR}/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a
    ${CMAKE_BINARY_DIR}/dll/ntdll/CMakeFiles/libntdll.dir/libntdll.a
    ${CMAKE_BINARY_DIR}/sdk/lib/crt/libmsvcrtex.a
    ${CMAKE_BINARY_DIR}/sdk/lib/crt/CMakeFiles/oldnames.dir/oldnames.a)

add_cargo_module(lsass lsass-cargo lsass.exe
    KIND win32exe
    BODY_CRATE lsass-rust
    DESTINATION reactos/system32
    EXTRA_DEPENDS ${LSASS_REUSED_LIBS})
