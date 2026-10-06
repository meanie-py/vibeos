
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl
    ${REACTOS_SOURCE_DIR}/sdk/include/wine)

add_definitions(-D__WINESRC__)
spec2def(samlib.dll samlib.spec ADD_IMPORTLIB)
add_rpc_files(client ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/sam.idl)

# samlib.dll — Rust cohabitation flip (FULL: both compiled src *.c → samlib-rust). This is the
# CLIENT half of sam.idl, whose server half is samsrv, so it closes that loop the same way
# wlanapi-rust closes wlansvc.idl. sam_c.c (the generated MIDL client stubs, _Samr*) stays C
# and provides the 68 stubs the Rust imports; those prototypes are bindgen-derived from
# sam_c.h, not hand-transcribed. dllmain.c/samlib.c are commented out below and kept on disk
# per the never-delete rule.
#
# The RpcExcept(EXCEPTION_EXECUTE_HANDLER) handlers throughout samlib.c are compiled but never
# entered (-D_USE_DUMMY_PSEH=1), so the translation legitimately drops them; 36 of them were
# accepted as dead-region omissions and 4 that had been emitted as LIVE code were removed by
# hand (they clobbered the real RPC status).
set(SAMLIB_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(SAMLIB_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(SAMLIB_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(SAMLIB_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(SAMLIB_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-samlib-target" samlib-rust)
set(SAMLIB_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libsamlib_rust.a")
rust_crate_sources(SAMLIB_RUST_SRCS "${SAMLIB_RUST_WORKSPACE_DIR}" samlib-rust)
add_custom_command(
    OUTPUT ${SAMLIB_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SAMLIB_RUST_TARGET_DIR}
        cargo build --release --target ${SAMLIB_RUST_TARGET}
            -p samlib-rust
            --manifest-path ${SAMLIB_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${SAMLIB_RUST_TARGET_DIR}/${SAMLIB_RUST_TARGET}/release/libsamlib_rust.a
        ${SAMLIB_RUST_LIB}
    DEPENDS ${SAMLIB_RUST_SRCS}
            ${SAMLIB_RUST_WORKSPACE_DIR}/samlib-rust/Cargo.toml
    WORKING_DIRECTORY ${SAMLIB_RUST_WORKSPACE_DIR}
    COMMENT "Building samlib-rust (${SAMLIB_RUST_TARGET})"
)
add_custom_target(rust_samlib DEPENDS ${SAMLIB_RUST_LIB})

# Both compiled src *.c → samlib-rust (src/samlib/*.rs), kept on disk per the never-delete
# rule. sam_c.c (the MIDL client stubs) stays C.
#   dllmain.c samlib.c  → samlib-rust
# list(APPEND SOURCE
#     dllmain.c
#     samlib.c
#     precomp.h)
list(APPEND SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/sam_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/samlib_stubs.c)

add_library(samlib MODULE
    ${SOURCE}
    samlib.rc
    ${CMAKE_CURRENT_BINARY_DIR}/samlib.def)

set_module_type(samlib win32dll UNICODE)
# libsamlib_rust.a FIRST so its #[no_mangle] symbols win; sam_c.c provides the MIDL client
# stubs the Rust imports.
target_link_libraries(samlib ${SAMLIB_RUST_LIB} wine ${PSEH_LIB})
add_dependencies(samlib rust_samlib)
add_importlibs(samlib rpcrt4 advapi32 msvcrt kernel32 ntdll)
# add_pch dropped with precomp.h: the only remaining C is generated (sam_c.c, samlib_stubs.c)
# and neither includes it.
add_cd_file(TARGET samlib DESTINATION reactos/system32 FOR all)
