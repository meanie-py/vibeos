include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/pnp.idl)
spec2def(umpnpmgr.dll umpnpmgr.spec ADD_IMPORTLIB)

# umpnpmgr.exe — Rust cohabitation flip (FULL: all 4 compiled src *.c → umpnpmgr-rust).
#
# The user-mode PnP manager: the SERVER half of pnp.idl, which cfgmgr32 and setupapi speak as
# clients. It owns device enumeration and installation — the PNP_* RPC surface, the kernel PnP
# event pump (NtGetPlugPlayEvent), and the device-install worker thread.
#
# pnp_s.c stays C: it is GENERATED (the MIDL server stubs), so there is nothing to translate —
# a build product, not source. spec2def's stubs are Rust (umpnpmgr_stubs.rs, gcc.cmake).
#
# Three profile decisions (gen_driver_types.sh), all load-bearing:
#   * BLOCKLIST_FUNCS_FROM_CRATE — a thin service over cfgmgr32/setupapi/advapi32/userenv/rpcrt4.
#     Hand-writing that surface would be ~200 prototypes, each of which links and then corrupts
#     silently if it is wrong by one pointer level.
#   * MACRO_SIGNED=0, and that was MEASURED. netcfgx and aclui both want it ON because their
#     constants are `int` control ids and window messages; umpnpmgr looks similar but spends its
#     time passing DBT_/WM_/CM_/SERVICE_ constants into UNSIGNED Win32 parameters, so signing
#     them cost 90 extra E0308s — 165 errors signed against 76 unsigned.
#   * UNBLOCK=LPTHREAD_START_ROUTINE/PTHREAD_START_ROUTINE — Rust compares fn-pointer ABIs
#     NOMINALLY, and ntdll-sys spells the thread-start callback with the arch's literal
#     convention. Emitting this island's own copy (which the ABI normaliser spells "system" on
#     all three arches) is what lets ONE definition of each thread body serve every target.
set(UMPNPMGR_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(UMPNPMGR_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(UMPNPMGR_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(UMPNPMGR_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(UMPNPMGR_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-umpnpmgr-target" umpnpmgr-rust)
set(UMPNPMGR_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libumpnpmgr_rust.a")
rust_crate_sources(UMPNPMGR_RUST_SRCS "${UMPNPMGR_RUST_WORKSPACE_DIR}" umpnpmgr-rust)
add_custom_command(
    OUTPUT ${UMPNPMGR_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${UMPNPMGR_RUST_TARGET_DIR}
        cargo build --release --target ${UMPNPMGR_RUST_TARGET}
            -p umpnpmgr-rust
            --manifest-path ${UMPNPMGR_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${UMPNPMGR_RUST_TARGET_DIR}/${UMPNPMGR_RUST_TARGET}/release/libumpnpmgr_rust.a
        ${UMPNPMGR_RUST_LIB}
    DEPENDS ${UMPNPMGR_RUST_SRCS}
            ${UMPNPMGR_RUST_WORKSPACE_DIR}/umpnpmgr-rust/Cargo.toml
    WORKING_DIRECTORY ${UMPNPMGR_RUST_WORKSPACE_DIR}
    COMMENT "Building umpnpmgr-rust (${UMPNPMGR_RUST_TARGET})"
)
add_custom_target(rust_umpnpmgr DEPENDS ${UMPNPMGR_RUST_LIB})

# All 4 compiled src *.c → umpnpmgr-rust (src/umpnpmgr/*.rs), kept on disk per the
# never-delete rule.
#   event.c install.c rpcserver.c umpnpmgr.c   → umpnpmgr-rust
# add_library(umpnpmgr MODULE
#     event.c
#     install.c
#     rpcserver.c
#     umpnpmgr.c

add_library(umpnpmgr MODULE
    umpnpmgr.rc
    ${CMAKE_CURRENT_BINARY_DIR}/pnp_s.c
    ${CMAKE_CURRENT_BINARY_DIR}/umpnpmgr_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/umpnpmgr.def)

# libumpnpmgr_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(umpnpmgr ${UMPNPMGR_RUST_LIB} wdmguid ${PSEH_LIB})
add_dependencies(umpnpmgr rust_umpnpmgr)
set_module_type(umpnpmgr win32dll UNICODE)
add_importlibs(umpnpmgr advapi32 rpcrt4 userenv shlwapi msvcrt user32 kernel32 ntdll)
add_cd_file(TARGET umpnpmgr DESTINATION reactos/system32 FOR all)
