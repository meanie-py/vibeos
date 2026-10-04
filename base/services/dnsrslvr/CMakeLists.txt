
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/dnsrslvr.idl)

spec2def(dnsrslvr.dll dnsrslvr.spec ADD_IMPORTLIB)

# dnsrslvr.dll — Rust cohabitation flip (FULL: all 4 compiled .c → dnsrslvr-rust).
# The DNS Client service: the resolver cache + the hosts-file seed + the R_Resolver* RPC server.
# This is the SERVER half of dnsrslvr.idl whose CLIENT half dnsapi.dll already speaks in Rust,
# so the flip closes that MIDL loop. Translated with DeepSeek. The .c files are COMMENTED OUT
# below, never deleted (per the never-delete rule); libdnsrslvr_rust.a supplies every body via
# #[no_mangle] and is linked FIRST so its symbols win.
#
# STILL C (correctly): the MIDL product dnsrslvr_s.c — generated wire marshalling for the
# dnsrslvr.idl interface. The Rust side #[no_mangle]-defines the R_Resolver* entry points that
# _s.c dispatches into.
set(DNSRSLVR_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(DNSRSLVR_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(DNSRSLVR_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(DNSRSLVR_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(DNSRSLVR_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-dnsrslvr-target" dnsrslvr-rust)
set(DNSRSLVR_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libdnsrslvr_rust.a")
rust_crate_sources(DNSRSLVR_RUST_SRCS "${DNSRSLVR_RUST_WORKSPACE_DIR}" dnsrslvr-rust)
add_custom_command(
    OUTPUT ${DNSRSLVR_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${DNSRSLVR_RUST_TARGET_DIR}
        cargo build --release --target ${DNSRSLVR_RUST_TARGET}
            -p dnsrslvr-rust
            --manifest-path ${DNSRSLVR_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${DNSRSLVR_RUST_TARGET_DIR}/${DNSRSLVR_RUST_TARGET}/release/libdnsrslvr_rust.a
        ${DNSRSLVR_RUST_LIB}
    DEPENDS ${DNSRSLVR_RUST_SRCS}
            ${DNSRSLVR_RUST_WORKSPACE_DIR}/dnsrslvr-rust/Cargo.toml
    WORKING_DIRECTORY ${DNSRSLVR_RUST_WORKSPACE_DIR}
    COMMENT "Building dnsrslvr-rust (${DNSRSLVR_RUST_TARGET})"
)
add_custom_target(rust_dnsrslvr DEPENDS ${DNSRSLVR_RUST_LIB})

list(APPEND SOURCE
    # cache.c      (→ dnsrslvr-rust src/dnsrslvr/cache.rs — the RESOLVER_CACHE list + DnsIntCache* CRUD)
    # dnsrslvr.c   (→ dnsrslvr-rust src/dnsrslvr/dnsrslvr.rs — DllMain + ServiceMain + control handler)
    # hostsfile.c  (→ dnsrslvr-rust src/dnsrslvr/hostsfile.rs — ReadHostsFile)
    # rpcserver.c  (→ dnsrslvr-rust src/dnsrslvr/rpcserver.rs — the R_Resolver* entry points)
    # precomp.h    (the PCH for the above — no C translation unit left to precompile)
    ${CMAKE_CURRENT_BINARY_DIR}/dnsrslvr_s.c)

add_library(dnsrslvr SHARED ${SOURCE} dnsrslvr.rc ${CMAKE_CURRENT_BINARY_DIR}/dnsrslvr.def)

add_dependencies(dnsrslvr rust_dnsrslvr)

# Rust staticlib FIRST so its #[no_mangle] bodies win over any surviving C definition.
target_link_libraries(dnsrslvr ${DNSRSLVR_RUST_LIB} ${PSEH_LIB})

set_module_type(dnsrslvr win32dll UNICODE)
add_importlibs(dnsrslvr advapi32 rpcrt4 dnsapi iphlpapi msvcrt kernel32 ntdll)
# add_pch removed: the 4 .c that used precomp.h are now Rust; only the self-contained
# MIDL-generated dnsrslvr_s.c remains, which does not need (or use) the PCH.
add_cd_file(TARGET dnsrslvr DESTINATION reactos/system32 FOR all)
