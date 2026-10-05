
include_directories( ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(client ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/wlansvc.idl)
spec2def(wlanapi.dll wlanapi.spec ADD_IMPORTLIB)

# wlanapi.dll — Rust cohabitation flip (FULL: both compiled src *.c → wlanapi-rust). This is
# the CLIENT half of wlansvc.idl, whose server half is already wlansvc-rust, so the flip
# closes that loop. The WLAN_*/DOT11_* wire layouts are bindgen-generated per arch into the
# crate's `types` module by tools/rustconv/gen_driver_types.sh (profile: wlanapi), not
# hand-transcribed. wlansvc_c.c (the generated MIDL client stubs, _Rpc*) stays C and provides
# the stubs the Rust imports; no symbol overlap. main.c/profile.c are commented out below and
# kept on disk per the never-delete rule.
set(WLANAPI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(WLANAPI_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(WLANAPI_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(WLANAPI_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(WLANAPI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-wlanapi-target" wlanapi-rust)
set(WLANAPI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libwlanapi_rust.a")
rust_crate_sources(WLANAPI_RUST_SRCS "${WLANAPI_RUST_WORKSPACE_DIR}" wlanapi-rust)
add_custom_command(
    OUTPUT ${WLANAPI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${WLANAPI_RUST_TARGET_DIR}
        cargo build --release --target ${WLANAPI_RUST_TARGET}
            -p wlanapi-rust
            --manifest-path ${WLANAPI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${WLANAPI_RUST_TARGET_DIR}/${WLANAPI_RUST_TARGET}/release/libwlanapi_rust.a
        ${WLANAPI_RUST_LIB}
    DEPENDS ${WLANAPI_RUST_SRCS}
            ${WLANAPI_RUST_WORKSPACE_DIR}/wlanapi-rust/Cargo.toml
    WORKING_DIRECTORY ${WLANAPI_RUST_WORKSPACE_DIR}
    COMMENT "Building wlanapi-rust (${WLANAPI_RUST_TARGET})"
)
add_custom_target(rust_wlanapi DEPENDS ${WLANAPI_RUST_LIB})

# Both compiled src *.c → wlanapi-rust (src/wlanapi/*.rs), kept on disk per the never-delete rule.
#   main.c profile.c  → wlanapi-rust
# wlansvc_c.c (the MIDL client stubs) stays C.
# list(APPEND SOURCE
#     main.c
#     profile.c)
list(APPEND SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/wlanapi_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/wlanapi.def
    ${CMAKE_CURRENT_BINARY_DIR}/wlansvc_c.c)

add_library(wlanapi MODULE ${SOURCE})
set_module_type(wlanapi win32dll)
# libwlanapi_rust.a FIRST so its #[no_mangle] symbols win; wlansvc_c.c provides the MIDL
# client stubs the Rust imports.
target_link_libraries(wlanapi ${WLANAPI_RUST_LIB} wine ${PSEH_LIB})
add_dependencies(wlanapi rust_wlanapi)
add_importlibs(wlanapi rpcrt4 msvcrt kernel32 ntdll)
add_cd_file(TARGET wlanapi DESTINATION reactos/system32 FOR all)
