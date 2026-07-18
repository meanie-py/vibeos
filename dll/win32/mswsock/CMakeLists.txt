
add_definitions(-DLE)
spec2def(mswsock.dll mswsock.spec ADD_IMPORTLIB)

# mswsock.dll — Rust cohabitation flip (all 4 compiled src *.c → mswsock-rust). The translated
# bodies come from libmswsock_rust.a (sdk/lib/rust/mswsock-rust, kept on disk per the
# never-delete rule); the C is commented out below.
#
# mswsock provides the Microsoft WinSock extensions (AcceptEx/TransmitFile/GetAcceptExSockaddrs),
# the RNR/namespace-provider wrappers (Get/SetService*, NSP* dispatch), and the WSPStartup glue.
# The winsock/NSP/DNS struct layouts are bindgen-GENERATED per arch into the crate's `types`
# module by tools/rustconv/gen_driver_types.sh (profile: mswsock), not hand-transcribed.
set(MSWSOCK_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(MSWSOCK_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(MSWSOCK_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(MSWSOCK_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
set(MSWSOCK_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-mswsock-target")
set(MSWSOCK_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libmswsock_rust.a")
file(GLOB_RECURSE MSWSOCK_RUST_SRCS "${MSWSOCK_RUST_WORKSPACE_DIR}/mswsock-rust/src/*.rs")
add_custom_command(
    OUTPUT ${MSWSOCK_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${MSWSOCK_RUST_TARGET_DIR}
        cargo build --release --target ${MSWSOCK_RUST_TARGET}
            -p mswsock-rust
            --manifest-path ${MSWSOCK_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${MSWSOCK_RUST_TARGET_DIR}/${MSWSOCK_RUST_TARGET}/release/libmswsock_rust.a
        ${MSWSOCK_RUST_LIB}
    DEPENDS ${MSWSOCK_RUST_SRCS}
            ${MSWSOCK_RUST_WORKSPACE_DIR}/mswsock-rust/Cargo.toml
    WORKING_DIRECTORY ${MSWSOCK_RUST_WORKSPACE_DIR}
    COMMENT "Building mswsock-rust (${MSWSOCK_RUST_TARGET})"
)
add_custom_target(rust_mswsock DEPENDS ${MSWSOCK_RUST_LIB})

# 4 src *.c bodies replaced by Rust (mswsock-rust/src/mswsock/*.rs), kept on disk per the
# never-delete rule. (DllMain lives in stubs.rs; win32dll's default entry calls it.)
#   extensions.c — replaced by Rust (mswsock-rust src/mswsock/extensions.rs — AcceptEx/TransmitFile/GetAcceptExSockaddrs)
#   mswhelper.c  — replaced by Rust (mswsock-rust src/mswsock/mswhelper.rs — MigrateWinsockConfiguration + winsock-config helpers)
#   nsplookup.c  — replaced by Rust (mswsock-rust src/mswsock/nsplookup.rs — RNR/namespace-provider Get/Set/EnumService* + NSP* dispatch)
#   stubs.c      — replaced by Rust (mswsock-rust src/mswsock/stubs.rs — ServiceMain/StartWsdpService/WSPStartup/DllMain + export stubs)
# list(APPEND SOURCE
#     extensions.c
#     mswhelper.c
#     nsplookup.c
#     stubs.c
#     precomp.h)

add_library(mswsock MODULE
    mswsock.rc
    ${CMAKE_CURRENT_BINARY_DIR}/mswsock.def)

set_module_type(mswsock win32dll UNICODE)
# libmswsock_rust.a provides the #[no_mangle] export + DllMain symbols; force the cargo build.
target_link_libraries(mswsock ${MSWSOCK_RUST_LIB})
add_dependencies(mswsock rust_mswsock)
add_importlibs(mswsock ws2_32 advapi32 dnsapi msvcrt kernel32 ntdll)
# add_pch removed: no C translation units remain (all 4 src *.c → Rust), so nothing to precompile.
add_cd_file(TARGET mswsock DESTINATION reactos/system32 FOR all)
