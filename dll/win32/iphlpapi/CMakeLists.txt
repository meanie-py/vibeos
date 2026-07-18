
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/wine
    ${REACTOS_SOURCE_DIR}/dll/win32/dhcpcsvc/include
    ${REACTOS_SOURCE_DIR}/sdk/lib/tdilib)

spec2def(iphlpapi.dll iphlpapi.spec ADD_IMPORTLIB)

add_definitions(
    -DGetAdaptersAddressesV2
    -D_CRT_NO_POSIX_ERROR_CODES)

# iphlpapi.dll — Rust cohabitation flip (PARTIAL: only iphlpapi_main.c → iphlpapi-rust).
# The translated bodies (74 IP-Helper entry points + qsort *Sorter comparators) come from
# libiphlpapi_rust.a (sdk/lib/rust/iphlpapi-rust, DeepSeek-V4 + c2rust, kept on disk per the
# never-delete rule); iphlpapi_main.c is commented out below. The ~10 sibling *_reactos.c/icmp.c/
# media.c/registry.c/address.c STAY C — iphlpapi_main.c calls into them via the local
# ifenum/ipstats/route/resinfo/dhcp prototypes (no symbol overlap: the Rust provides ONLY
# iphlpapi_main.c's exports). `wine` stays linked (ros_dbg_log, the debug channel the Rust
# prelude declares extern); iphlpapi_rust_shims.c provides the no-op TRACE/FIXME/ERR/WARN +
# __wine_dbch___default symbols the DeepSeek output references as bare symbols.
set(IPHLPAPI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(IPHLPAPI_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(IPHLPAPI_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(IPHLPAPI_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
set(IPHLPAPI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-iphlpapi-target")
set(IPHLPAPI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libiphlpapi_rust.a")
file(GLOB_RECURSE IPHLPAPI_RUST_SRCS "${IPHLPAPI_RUST_WORKSPACE_DIR}/iphlpapi-rust/src/*.rs")
add_custom_command(
    OUTPUT ${IPHLPAPI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${IPHLPAPI_RUST_TARGET_DIR}
        cargo build --release --target ${IPHLPAPI_RUST_TARGET}
            -p iphlpapi-rust
            --manifest-path ${IPHLPAPI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${IPHLPAPI_RUST_TARGET_DIR}/${IPHLPAPI_RUST_TARGET}/release/libiphlpapi_rust.a
        ${IPHLPAPI_RUST_LIB}
    DEPENDS ${IPHLPAPI_RUST_SRCS}
            ${IPHLPAPI_RUST_WORKSPACE_DIR}/iphlpapi-rust/Cargo.toml
    WORKING_DIRECTORY ${IPHLPAPI_RUST_WORKSPACE_DIR}
    COMMENT "Building iphlpapi-rust (${IPHLPAPI_RUST_TARGET})"
)
add_custom_target(rust_iphlpapi DEPENDS ${IPHLPAPI_RUST_LIB})

list(APPEND SOURCE
    address.c
    dhcp_reactos.c
    icmp.c
    ifenum_reactos.c
    ipstats_reactos.c
    iphlpapi_reactos.c
    # iphlpapi_main.c   (→ iphlpapi-rust src/iphlpapi/iphlpapi_main.rs — 74 fns, cohab-flipped)
    media.c
    registry.c
    resinfo_reactos.c
    route_reactos.c
    iphlpapi_private.h
    ${CMAKE_CURRENT_BINARY_DIR}/iphlpapi_stubs.c)

# The no-op wine-debug symbol shim (freestanding C, no PCH — the Rust references TRACE/FIXME/
# ERR/WARN + __wine_dbch___default as bare symbols; see iphlpapi_rust_shims.c).
list(APPEND PCH_SKIP_SOURCE
    iphlpapi_rust_shims.c)

add_library(iphlpapi MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    iphlpapi.rc
    iphlpapi_fakestubs.c
    iphlpapi.spec
    ${CMAKE_CURRENT_BINARY_DIR}/iphlpapi.def)

set_module_type(iphlpapi win32dll UNICODE)
# libiphlpapi_rust.a FIRST so its #[no_mangle] symbols win; wine provides ros_dbg_log.
target_link_libraries(iphlpapi ${IPHLPAPI_RUST_LIB} wine tdilib)
add_dependencies(iphlpapi rust_iphlpapi)
add_importlibs(iphlpapi dhcpcsvc advapi32 psapi ws2_32 msvcrt kernel32 ntdll)
# add_pch kept for the remaining C files; iphlpapi_rust_shims.c is SKIPPED (freestanding, and the
# PCH's wine/debug.h would macro-expand its no-op TRACE/FIXME/ERR/WARN definitions).
add_pch(iphlpapi iphlpapi_private.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET iphlpapi DESTINATION reactos/system32 FOR all)
