include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl
    ${CMAKE_CURRENT_BINARY_DIR})

add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/winreg.idl)

# CARGO-DRIVEN LINK + PARTIAL Rust flip. winlogon.exe's link is now driven by
# cargo (winlogon-cargo → reactos-link ModuleKind::Win32Exe) instead of cmake
# add_executable — the third of OUR translated spine binaries moved cmake→cargo
# (after lsass + userinit). Unlike those two (100%-Rust bodies), winlogon is a
# PARTIAL flip: the GINA-independent leaf files are Rust (winlogon-rust), but
# the GINA core (wlx.c), SAS state machine (sas.c), main (winlogon.c), RPC
# server (rpcserver.c) and the generated winreg_s.c RPC stub stay C. So this is
# the first cargo-drive whose C residue is compiled IN-cargo via the engine's
# c_from_compile_db (compile_commands.json replay) rather than a static_libs
# reuse. Flipped .c stay on disk (never delete the C — translation philosophy).
#
# c_from_compile_db needs the residue's flag-exact compile entries in
# compile_commands.json (and the PCH generated), so cmake still COMPILES the
# residue via an OBJECT library `winlogon_c`. Its objects are not linked
# directly (cargo recompiles from the replayed command) — the library exists
# purely to (a) populate compile_commands.json under CMakeFiles/winlogon_c.dir
# and (b) generate cmake_pch.h. winlogon-cargo's build.rs points
# c_from_compile_db("winlogon_c", …) at it.
# ALL 4 GINA/SAS/RPC cores (rpcserver/winlogon/wlx/sas) flipped → winlogon-rust/
# src/*.rs; winlogon.exe is now a FULL Rust flip (zero C in the cargo link). The
# winlogon_c OBJECT lib survives only to compile the remaining vestigial sources
# (winlogon_rust_stub.c + the generated winreg_s.c) into compile_commands.json
# for the winreg-stub crate's decoration probe. The C stays on disk.
list(APPEND WINLOGON_C_RESIDUE
    winlogon_rust_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/winreg_s.c)
add_library(winlogon_c OBJECT ${WINLOGON_C_RESIDUE} winlogon.h)
add_pch(winlogon_c winlogon.h "")
# The Rust-cohab stub is self-contained — keep the PCH out of it (as the old
# add_executable did).
set_source_files_properties(winlogon_rust_stub.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON)

# winlogon.rc is LOAD-BEARING (the winlogon.ico application icon), so it cannot
# be omitted. Keep cmake compiling it via a resource-only OBJECT library; the
# cargo link reuses the .res through reactos-link .resource_built() (the same
# .res pattern userinit uses). winlogon is set_module_type win32gui WITHOUT
# UNICODE, so — unlike userinit_res — no UNICODE define here (matches the old
# winlogon.exe rc compile byte-for-byte).

# --- Rust cohabitation (winlogon-rust) ------------------------------------
# The Rust leaf functions (GetSetupType/RunSetup/environment/shutdown/
# screensaver/security/notify) come from libwinlogon_rust.a, now built by cargo
# (winlogon-cargo .rust_staticlib) — the cmake add_custom_command that used to
# build it is retired. FFI substrate is the self-contained winlogon-sys.
# --------------------------------------------------------------------------

# The CRT/libwine/libpseh statics + oldnames the cargo link reuses live in
# sdk/lib + 3rdparty (configured after base/), so DEPENDS on their .a FILE
# paths for parallel-build ordering. The 10 import libs are generated in-cargo
# from .spec. The residue (winlogon_c) + the .res (winlogon_res) are in-scope
# target names.

add_cargo_module(winlogon winlogon-cargo winlogon.exe
    KIND win32exe
    BODY_CRATE winlogon-rust
    DESTINATION reactos/system32
    EXTRA_DEPENDS winlogon_c psdk)
