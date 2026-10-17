
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/wine
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl
    ${CMAKE_CURRENT_BINARY_DIR})

add_definitions(
    -D_SETUPAPI_
    -D_CFGMGR32_
)

add_rpc_files(client ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/pnp.idl)
spec2def(setupapi.dll setupapi.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # cfgmgr.c — replaced by Rust (setupapi-rust)
    devclass.c
    devprop.c
    devinst.c
    dialog.c
    dirid.c
    diskspace.c
    driver.c
    install.c
    interface.c
    misc.c
    parser.c
    query.c
    queue.c
    setupcab.c
    stringtable.c
    stubs.c
    rpc.c)

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/setupapi_stubs.c)

# --- Rust cohabitation (setupapi-rust) ------------------------------------
# libsetupapi_rust.a carries the TUs flipped so far (cfgmgr.c, whose RpcTryExcept guards
# compile to nothing under DUMMY_PSEH and are real rpc_try calls in Rust); the rest of
# the module is still the C above.
set(SETUPAPI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(SETUPAPI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-setupapi-target" setupapi-rust)
set(SETUPAPI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libsetupapi_rust.a")
if(ARCH STREQUAL "i386")
    set(SETUPAPI_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(SETUPAPI_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(SETUPAPI_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_crate_sources(SETUPAPI_RUST_SRCS "${SETUPAPI_RUST_WORKSPACE_DIR}" setupapi-rust)
add_custom_command(
    OUTPUT ${SETUPAPI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SETUPAPI_RUST_TARGET_DIR}
        cargo build --release --target ${SETUPAPI_RUST_TARGET}
            -p setupapi-rust
            --manifest-path ${SETUPAPI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${SETUPAPI_RUST_TARGET_DIR}/${SETUPAPI_RUST_TARGET}/release/libsetupapi_rust.a
        ${SETUPAPI_RUST_LIB}
    DEPENDS ${SETUPAPI_RUST_SRCS}
            ${SETUPAPI_RUST_WORKSPACE_DIR}/setupapi-rust/Cargo.toml
    WORKING_DIRECTORY ${SETUPAPI_RUST_WORKSPACE_DIR}
    COMMENT "Building setupapi-rust (${SETUPAPI_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_setupapi DEPENDS ${SETUPAPI_RUST_LIB})
# --------------------------------------------------------------------------

add_library(setupapi MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/pnp_c.c
    setupapi.rc
    ${CMAKE_CURRENT_BINARY_DIR}/setupapi.def)

set_module_type(setupapi win32dll UNICODE)
# SETUPAPI_RUST_LIB first so its #[no_mangle] bodies win over any lingering C.
target_link_libraries(setupapi ${SETUPAPI_RUST_LIB} uuid wine ${PSEH_LIB} oldnames)
add_delay_importlibs(setupapi comdlg32 shell32 winspool.drv wintrust)
add_importlibs(setupapi gdi32 comctl32 advapi32 user32 rpcrt4 version msvcrt kernel32 ntdll)
add_pch(setupapi setupapi_private.h "${PCH_SKIP_SOURCE}")
add_dependencies(setupapi rust_setupapi)
add_cd_file(TARGET setupapi DESTINATION reactos/system32 FOR all)
