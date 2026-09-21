
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/wine)

spec2def(iphlpapi.dll iphlpapi.spec ADD_IMPORTLIB)

add_definitions(
    -DGetAdaptersAddressesV2
    -D_CRT_NO_POSIX_ERROR_CODES)

# iphlpapi.dll — Rust cohabitation flip (FULL: all 11 compiled .c → iphlpapi-rust).
# iphlpapi_main.c was c2rust-anchored (74 IP-Helper entry points); the other 10 are pure-LLM
# (DeepSeek V4-pro), except iphlpapi_fakestubs.c whose 16 Pf* bodies are byte-identical stubs
# and were transcribed by hand. Every .c is commented out below, none deleted (never-delete
# rule); libiphlpapi_rust.a links FIRST so its #[no_mangle] bodies win.
#
# iphlpapi_rust_shims.c is GONE (it was: no-op TRACE/FIXME/ERR/WARN + __wine_dbch___default).
# Those no-ops were out-of-line FUNCTIONS, which is the field-manual §9.65 hazard — an
# out-of-line no-op is an optimization barrier that can uncloak a latent ABI mismatch into a
# ud2. The 158 call sites are now `macro_rules!` no-ops in the crate prelude, and the wine
# default debug channel is a Rust static with flags=0 (statically false at every c2rust-
# expanded `if (*chan).flags & (1<<CLASS)` site — exactly the non-DBG C build).
#
# STILL C (correctly): only the generated iphlpapi_stubs.c. `wine` stays linked for
# ros_dbg_log; tdilib supplies the TDI helper library.
set(IPHLPAPI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(IPHLPAPI_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(IPHLPAPI_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(IPHLPAPI_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(IPHLPAPI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-iphlpapi-target" iphlpapi-rust)
set(IPHLPAPI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libiphlpapi_rust.a")
rust_crate_sources(IPHLPAPI_RUST_SRCS "${IPHLPAPI_RUST_WORKSPACE_DIR}" iphlpapi-rust)
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
    # address.c            (→ iphlpapi-rust src/iphlpapi/address.rs — GetAdaptersAddresses + adapter/prefix enumeration)
    # dhcp_reactos.c       (→ iphlpapi-rust src/iphlpapi/dhcp_reactos.rs)
    # icmp.c               (→ iphlpapi-rust src/iphlpapi/icmp.rs — Icmp/Icmp6 CreateFile/SendEcho/ParseReplies)
    # ifenum_reactos.c     (→ iphlpapi-rust src/iphlpapi/ifenum_reactos.rs — interface enumeration over TDI (40 fns))
    # ipstats_reactos.c    (→ iphlpapi-rust src/iphlpapi/ipstats_reactos.rs — IP/TCP/UDP/ICMP stats + arp/route tables)
    # iphlpapi_reactos.c   (→ iphlpapi-rust src/iphlpapi/iphlpapi_reactos.rs)
    # iphlpapi_main.c   (→ iphlpapi-rust src/iphlpapi/iphlpapi_main.rs — 74 fns, cohab-flipped)
    # media.c              (→ iphlpapi-rust src/iphlpapi/media.rs)
    # registry.c           (→ iphlpapi-rust src/iphlpapi/registry.rs)
    # resinfo_reactos.c    (→ iphlpapi-rust src/iphlpapi/resinfo_reactos.rs)
    # route_reactos.c      (→ iphlpapi-rust src/iphlpapi/route_reactos.rs)
    iphlpapi_private.h
    ${CMAKE_CURRENT_BINARY_DIR}/iphlpapi_stubs.c)

add_library(iphlpapi MODULE
    ${SOURCE}
    iphlpapi.rc
    iphlpapi.spec
    ${CMAKE_CURRENT_BINARY_DIR}/iphlpapi.def)
add_pch(iphlpapi iphlpapi_private.h SOURCE)

set_module_type(iphlpapi win32dll UNICODE)
# libiphlpapi_rust.a FIRST so its #[no_mangle] symbols win; wine provides ros_dbg_log.
target_link_libraries(iphlpapi ${IPHLPAPI_RUST_LIB} wine tdilib)
add_dependencies(iphlpapi rust_iphlpapi)
add_importlibs(iphlpapi dhcpcsvc advapi32 psapi ws2_32 msvcrt kernel32 ntdll)
# add_pch kept for the remaining C files; iphlpapi_rust_shims.c is SKIPPED (freestanding, and the
# PCH's wine/debug.h would macro-expand its no-op TRACE/FIXME/ERR/WARN definitions).
add_cd_file(TARGET iphlpapi DESTINATION reactos/system32 FOR all)
