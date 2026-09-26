
spec2def(advapi32.dll advapi32.spec ADD_IMPORTLIB)
add_definitions(-D_ADVAPI32_)

include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers/ksecdd
    ${CMAKE_CURRENT_BINARY_DIR})

add_rpc_files(client
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/eventlogrpc.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/lsa.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/seclogon.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/svcctl.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/winreg.idl)

list(APPEND SOURCE
    # misc/dllmain.c — replaced by Rust (advapi32-rust)
    # misc/efs.c — replaced by Rust (advapi32-rust)
    # misc/evntrace.c — replaced by Rust (advapi32-rust)
    # misc/hwprofiles.c — replaced by Rust (advapi32-rust)
    # misc/logon.c — replaced by Rust (advapi32-rust)
    # misc/msi.c — replaced by Rust (advapi32-rust)
    # misc/shutdown.c — replaced by Rust (advapi32-rust)
    # misc/sysfunc.c — replaced by Rust (advapi32-rust)
    # misc/unicode.c — replaced by Rust (advapi32-rust)
    # reg/hkcr.c — replaced by Rust (advapi32-rust)
    # reg/reg.c — replaced by Rust (advapi32-rust)
    # sec/ac.c — replaced by Rust (advapi32-rust)
    # sec/audit.c — replaced by Rust (advapi32-rust)
    # sec/lsa.c — replaced by Rust (advapi32-rust)
    # sec/misc.c — replaced by Rust (advapi32-rust)
    # sec/safer.c — replaced by Rust (advapi32-rust)
    # sec/sec.c — replaced by Rust (advapi32-rust)
    # sec/trustee.c — replaced by Rust (advapi32-rust)
    # service/eventlog.c — replaced by Rust (advapi32-rust)
    # service/rpc.c — replaced by Rust (advapi32-rust)
    # service/scm.c — replaced by Rust (advapi32-rust)
    # service/sctrl.c — replaced by Rust (advapi32-rust)
    # service/winreg.c — replaced by Rust (advapi32-rust)
    # token/token.c — replaced by Rust (advapi32-rust)
    # wine/cred.c — replaced by Rust (advapi32-rust)
    # wine/crypt.c — replaced by Rust (advapi32-rust)
    # wine/crypt_des.c — replaced by Rust (advapi32-rust)
    # wine/crypt_lmhash.c — replaced by Rust (advapi32-rust)
    # wine/security.c — replaced by Rust (advapi32-rust/src/wine/security.rs)
)

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/advapi32_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/eventlogrpc_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/lsa_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/seclogon_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/svcctl_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/winreg_c.c)

# service/eventlog_seh.c — native-SEH wrappers restoring the RpcTryExcept the
# advapi32-rust cohab dropped around the EventLog RPC client stubs (see the file
# header). PCH-skipped so it carries its own include set + per-file define.
# amd64/arm64 opt into a real __try via _USE_NATIVE_SEH (pseh2.h checks it before
# the DUMMY_PSEH default); i386 keeps the DUMMY pass-through (no reliable clang
# native SEH there — same rationale the kmtest native-SEH opt-in excludes i386).
set(EVENTLOG_SEH_SHIM ${CMAKE_CURRENT_SOURCE_DIR}/service/eventlog_seh.c)
list(APPEND PCH_SKIP_SOURCE ${EVENTLOG_SEH_SHIM})
if(ARCH STREQUAL "amd64" OR ARCH STREQUAL "arm64")
    set_source_files_properties(${EVENTLOG_SEH_SHIM} PROPERTIES COMPILE_DEFINITIONS _USE_NATIVE_SEH)
endif()

# --- Rust cohabitation (advapi32-rust) ------------------------------------
# Build libadvapi32_rust.a from the workspace crate (STANDARD user-mode per-arch
# targets; i386 gets -sse,-sse2 via sdk/lib/rust/.cargo/config.toml). The
# #[no_mangle] bodies replace the commented-out .c TUs above; lld first-match
# (--allow-multiple-definition) lets the Rust staticlib's panic_handler/
# eh_personality coexist with whatever the C cohab pulls in.
set(ADVAPI32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(ADVAPI32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-advapi32-target" advapi32-rust)
set(ADVAPI32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libadvapi32_rust.a")
if(ARCH STREQUAL "i386")
    set(ADVAPI32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(ADVAPI32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(ADVAPI32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_crate_sources(ADVAPI32_RUST_SRCS "${ADVAPI32_RUST_WORKSPACE_DIR}" advapi32-rust)
add_custom_command(
    OUTPUT ${ADVAPI32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${ADVAPI32_RUST_TARGET_DIR}
        cargo build --release --target ${ADVAPI32_RUST_TARGET}
            -p advapi32-rust
            --manifest-path ${ADVAPI32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${ADVAPI32_RUST_TARGET_DIR}/${ADVAPI32_RUST_TARGET}/release/libadvapi32_rust.a
        ${ADVAPI32_RUST_LIB}
    DEPENDS ${ADVAPI32_RUST_SRCS}
            ${ADVAPI32_RUST_WORKSPACE_DIR}/advapi32-rust/Cargo.toml
    WORKING_DIRECTORY ${ADVAPI32_RUST_WORKSPACE_DIR}
    COMMENT "Building advapi32-rust (${ADVAPI32_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_advapi32 DEPENDS ${ADVAPI32_RUST_LIB})
# --------------------------------------------------------------------------

add_library(advapi32 MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    advapi32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/advapi32.def)
add_pch(advapi32 advapi32.h "${PCH_SKIP_SOURCE}")

set_module_type(advapi32 win32dll UNICODE ENTRYPOINT DllMain 12)
# ADVAPI32_RUST_LIB first so its #[no_mangle] bodies win over any lingering C.
target_link_libraries(advapi32 ${ADVAPI32_RUST_LIB} cryptlib wine ${PSEH_LIB})
target_link_options(advapi32 PRIVATE LINKER:--allow-multiple-definition)
if(DLL_EXPORT_VERSION GREATER_EQUAL 0x600)
    target_link_libraries(advapi32 etwtrace)
endif()
add_delay_importlibs(advapi32 secur32)
add_importlibs(advapi32 advapi32_vista rpcrt4 kernel32 ntdll)
add_pch(advapi32 advapi32.h "${PCH_SKIP_SOURCE}")
add_dependencies(advapi32 rust_advapi32)
add_cd_file(TARGET advapi32 DESTINATION reactos/system32 FOR all)
