
spec2def(wshtcpip.dll wshtcpip.spec)

# wshtcpip.dll — Rust cohabitation flip (both compiled src *.c → wshtcpip-rust). The
# translated bodies come from libwshtcpip_rust.a (sdk/lib/rust/wshtcpip-rust, kept on disk
# per the never-delete rule); the C is commented out below.
#
# wshtcpip is the transport-specific WSH provider that ws2_32/msafd load to resolve the
# TCP/IP address family — it maps winsock protocol/socket params to TDI and answers
# WSHOpenSocket / WSHGetSocketInformation / WSHIoctl / WSHStringToAddress. openTcpFile/
# closeTcpFile/tdiGetEntityIDSet/tdiFreeThingSet still come from tdilib (linked below); the
# winsock/WSH/TDI struct layouts are bindgen-generated per arch into the crate's `types`
# module by tools/rustconv/gen_driver_types.sh, not hand-transcribed.
set(WSHTCPIP_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(WSHTCPIP_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(WSHTCPIP_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(WSHTCPIP_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(WSHTCPIP_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-wshtcpip-target" wshtcpip-rust)
set(WSHTCPIP_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libwshtcpip_rust.a")
rust_crate_sources(WSHTCPIP_RUST_SRCS "${WSHTCPIP_RUST_WORKSPACE_DIR}" wshtcpip-rust)
add_custom_command(
    OUTPUT ${WSHTCPIP_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${WSHTCPIP_RUST_TARGET_DIR}
        cargo build --release --target ${WSHTCPIP_RUST_TARGET}
            -p wshtcpip-rust
            --manifest-path ${WSHTCPIP_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${WSHTCPIP_RUST_TARGET_DIR}/${WSHTCPIP_RUST_TARGET}/release/libwshtcpip_rust.a
        ${WSHTCPIP_RUST_LIB}
    DEPENDS ${WSHTCPIP_RUST_SRCS}
            ${WSHTCPIP_RUST_WORKSPACE_DIR}/wshtcpip-rust/Cargo.toml
    WORKING_DIRECTORY ${WSHTCPIP_RUST_WORKSPACE_DIR}
    COMMENT "Building wshtcpip-rust (${WSHTCPIP_RUST_TARGET})"
)
add_custom_target(rust_wshtcpip DEPENDS ${WSHTCPIP_RUST_LIB})

# 2 src *.c bodies replaced by Rust (wshtcpip-rust/src/wshtcpip/*.rs), kept on disk per the
# never-delete rule.
#   iflist.c   — replaced by Rust (wshtcpip-rust src/wshtcpip/iflist.rs — WSHIoctl_GetInterfaceList + TDI interface enumeration)
#   wshtcpip.c — replaced by Rust (wshtcpip-rust src/wshtcpip/wshtcpip.rs — the WSH provider entry points + DllMain)
# list(APPEND SOURCE
#     iflist.c
#     wshtcpip.c)
list(APPEND SOURCE
    wshtcpip.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wshtcpip.def)

add_library(wshtcpip MODULE ${SOURCE})
set_module_type(wshtcpip win32dll UNICODE)
# libwshtcpip_rust.a first so its #[no_mangle] symbols win. tdilib provides openTcpFile/
# closeTcpFile/tdiGetEntityIDSet/tdiFreeThingSet that the Rust still imports.
target_link_libraries(wshtcpip ${WSHTCPIP_RUST_LIB} tdilib)
add_dependencies(wshtcpip rust_wshtcpip)
add_importlibs(wshtcpip ws2_32 msvcrt kernel32 ntdll)
add_cd_file(TARGET wshtcpip DESTINATION reactos/system32 FOR all)
