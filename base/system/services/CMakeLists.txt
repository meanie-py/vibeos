
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl
    ${CMAKE_CURRENT_BINARY_DIR})

add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/svcctl.idl)

# CARGO-DRIVEN LINK + PARTIAL Rust flip (services-rust). services.exe (SCM)'s
# link is now driven by cargo (services-cargo → reactos-link
# ModuleKind::Win32Exe) instead of cmake add_executable — the fourth of OUR
# translated spine binaries moved cmake→cargo (after lsass + userinit +
# winlogon). The 8 flipped SCM .c come from libservices_rust.a; the still-C
# residue (services.c main + services_rust_stub.c + the generated svcctl_s.c
# RPC server stub) is compiled IN-cargo via the engine's c_from_compile_db
# (the winlogon partial-flip pattern). Flipped .c stay on disk (never delete
# the C — translation philosophy).
#
# c_from_compile_db needs the residue's flag-exact compile entries in
# compile_commands.json (and the PCH generated), so cmake still COMPILES the
# residue via an OBJECT library `services_c`. Its objects are not linked
# directly (cargo recompiles from the replayed command) — the library exists
# purely to (a) populate CMakeFiles/services_c.dir entries and (b) generate
# cmake_pch.h. services-cargo's build.rs points c_from_compile_db("services_c",
# …) at it. The UNICODE compile define matches the old win32gui-UNICODE module.
list(APPEND SERVICES_C_RESIDUE
    services.c
    services_rust_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/svcctl_s.c)
add_library(services_c OBJECT ${SERVICES_C_RESIDUE} services.h)
# services.exe was win32gui UNICODE → its .c compiled with -DUNICODE -D_UNICODE.
# Replicate just the compile define on the OBJECT lib (not full set_module_type,
# which does link-side work that misbehaves on an object library).
target_compile_definitions(services_c PRIVATE UNICODE _UNICODE)
add_pch(services_c services.h "")
# The Rust-cohab stub is self-contained — keep the PCH out of it.
set_source_files_properties(services_rust_stub.c PROPERTIES SKIP_PRECOMPILE_HEADERS ON)

# services.rc is LOAD-BEARING (localized STRINGTABLEs + version info). Keep
# cmake compiling it via a resource-only OBJECT library; the cargo link reuses
# the .res through reactos-link .resource_built() (the userinit/winlogon .res
# pattern). services is win32gui UNICODE, so — like userinit_res — UNICODE.
add_library(services_res OBJECT services.rc)
target_compile_definitions(services_res PRIVATE UNICODE _UNICODE)
target_include_directories(services_res PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})

# --- Rust cohabitation (services-rust) ------------------------------------
# The 8 flipped SCM .c (config/controlset/database/driver/groupdb/lock/
# rpcserver/security) come from libservices_rust.a, now built by cargo
# (services-cargo .rust_staticlib) — the cmake add_custom_command that used to
# build it is retired. FFI substrate is winsrv-sys over ntdll-sys.
# --------------------------------------------------------------------------

# The CRT/libpseh statics + oldnames the cargo link reuses live in sdk/lib
# (configured after base/), so DEPENDS on their .a FILE paths for parallel-build
# ordering. The 8 import libs are generated in-cargo from .spec. The residue
# (services_c) + the .res (services_res) are in-scope target names.
set(SERVICES_REUSED
    ${CMAKE_BINARY_DIR}/sdk/lib/pseh/libpseh.a
    ${CMAKE_BINARY_DIR}/sdk/lib/crt/libchkstk.a
    ${CMAKE_BINARY_DIR}/sdk/lib/crt/libmsvcrtex.a
    ${CMAKE_BINARY_DIR}/sdk/lib/crt/CMakeFiles/oldnames.dir/oldnames.a)

add_cargo_module(services services-cargo services.exe
    KIND win32exe
    BODY_CRATE services-rust
    DESTINATION reactos/system32
    EXTRA_DEPENDS services_c services_res ${SERVICES_REUSED})
