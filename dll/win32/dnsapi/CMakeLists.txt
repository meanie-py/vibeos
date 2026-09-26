
include_directories(
    include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)

add_rpc_files(client ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/dnsrslvr.idl)

add_definitions(-DADNS_JGAA_WIN32 -D_CRT_NO_POSIX_ERROR_CODES)

spec2def(dnsapi.dll dnsapi.spec ADD_IMPORTLIB)

# dnsapi.dll — Rust cohabitation flip (FULL: all 8 compiled src *.c → dnsapi-rust). The
# translated bodies come from libdnsapi_rust.a (sdk/lib/rust/dnsapi-rust, DeepSeek-reasoner,
# kept on disk per the never-delete rule); all 8 .c are commented out below. The windns/adns/
# dnsrslvr struct layouts are bindgen-generated per arch into the crate's `types` module by
# tools/rustconv/gen_driver_types.sh (profile: dnsapi), not hand-transcribed. `adns` (the
# 3rdparty async resolver staticlib) stays linked — the Rust calls adns_init/synchronous/finish
# etc.; dnsrslvr_c.c (the single MIDL client stub, R_Resolver*) also stays C and provides the
# stubs the Rust imports. No symbol overlap (the Rust provides ONLY the flipped .c's exports).
set(DNSAPI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(DNSAPI_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(DNSAPI_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(DNSAPI_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(DNSAPI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-dnsapi-target" dnsapi-rust)
set(DNSAPI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libdnsapi_rust.a")
rust_crate_sources(DNSAPI_RUST_SRCS "${DNSAPI_RUST_WORKSPACE_DIR}" dnsapi-rust)
add_custom_command(
    OUTPUT ${DNSAPI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${DNSAPI_RUST_TARGET_DIR}
        cargo build --release --target ${DNSAPI_RUST_TARGET}
            -p dnsapi-rust
            --manifest-path ${DNSAPI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${DNSAPI_RUST_TARGET_DIR}/${DNSAPI_RUST_TARGET}/release/libdnsapi_rust.a
        ${DNSAPI_RUST_LIB}
    DEPENDS ${DNSAPI_RUST_SRCS}
            ${DNSAPI_RUST_WORKSPACE_DIR}/dnsapi-rust/Cargo.toml
    WORKING_DIRECTORY ${DNSAPI_RUST_WORKSPACE_DIR}
    COMMENT "Building dnsapi-rust (${DNSAPI_RUST_TARGET})"
)
add_custom_target(rust_dnsapi DEPENDS ${DNSAPI_RUST_LIB})

# All 8 compiled src *.c → dnsapi-rust (src/dnsapi/*.rs), kept on disk per the never-delete rule.
#   adns.c context.c memory.c names.c query.c record.c rpc.c stubs.c  → dnsapi-rust
# dnsrslvr_c.c (the MIDL client stub) stays C.
# list(APPEND SOURCE
#     adns.c
#     context.c
#     memory.c
#     names.c
#     query.c
#     record.c
#     rpc.c
#     stubs.c)
list(APPEND SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/dnsrslvr_c.c)

add_library(dnsapi MODULE
    ${SOURCE}
    dnsapi.rc
    ${CMAKE_CURRENT_BINARY_DIR}/dnsapi_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/dnsapi.def)
add_pch(dnsapi precomp.h SOURCE)

set_module_type(dnsapi win32dll)
# libdnsapi_rust.a FIRST so its #[no_mangle] symbols win; adns provides the resolver, dnsrslvr_c.c
# the MIDL client stubs the Rust imports.
target_link_libraries(dnsapi ${DNSAPI_RUST_LIB} adns ${PSEH_LIB})
add_dependencies(dnsapi rust_dnsapi)
add_importlibs(dnsapi advapi32 rpcrt4 user32 ws2_32 iphlpapi msvcrt kernel32 ntdll)
add_cd_file(TARGET dnsapi DESTINATION reactos/system32 FOR all)
