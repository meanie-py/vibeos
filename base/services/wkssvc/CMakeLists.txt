
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/wkssvc.idl)
spec2def(wkssvc.dll wkssvc.spec ADD_IMPORTLIB)

# wkssvc.dll — Rust cohabitation flip (FULL: all 4 compiled .c → wkssvc-rust).
# The Workstation service (svchost -k netsvcs; hivesys Start=2, so it really runs at boot):
# the workgroup/domain join state, the WKSTA_INFO_502 settings + their registry backing, and
# the Netr* RPC server. This is the SERVER half of wkssvc.idl whose CLIENT half netapi32.dll
# already speaks in Rust (NetWksta*/NetJoinDomain/NetGetJoinInformation), so the flip closes
# that MIDL loop. Translated with DeepSeek. The .c files are COMMENTED OUT below, never
# deleted (per the never-delete rule); libwkssvc_rust.a supplies every body via #[no_mangle]
# and is linked FIRST so its symbols win.
#
# STILL C (correctly): the MIDL product wkssvc_s.c — generated wire marshalling for
# wkssvc.idl. The Rust side #[no_mangle]-defines the Netr* entry points _s.c dispatches into.
set(WKSSVC_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(WKSSVC_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(WKSSVC_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(WKSSVC_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(WKSSVC_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-wkssvc-target" wkssvc-rust)
set(WKSSVC_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libwkssvc_rust.a")
rust_crate_sources(WKSSVC_RUST_SRCS "${WKSSVC_RUST_WORKSPACE_DIR}" wkssvc-rust)
add_custom_command(
    OUTPUT ${WKSSVC_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${WKSSVC_RUST_TARGET_DIR}
        cargo build --release --target ${WKSSVC_RUST_TARGET}
            -p wkssvc-rust
            --manifest-path ${WKSSVC_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${WKSSVC_RUST_TARGET_DIR}/${WKSSVC_RUST_TARGET}/release/libwkssvc_rust.a
        ${WKSSVC_RUST_LIB}
    DEPENDS ${WKSSVC_RUST_SRCS}
            ${WKSSVC_RUST_WORKSPACE_DIR}/wkssvc-rust/Cargo.toml
    WORKING_DIRECTORY ${WKSSVC_RUST_WORKSPACE_DIR}
    COMMENT "Building wkssvc-rust (${WKSSVC_RUST_TARGET})"
)
add_custom_target(rust_wkssvc DEPENDS ${WKSSVC_RUST_LIB})

# 4 .c bodies replaced by Rust (wkssvc-rust/src/wkssvc/*.rs), kept on disk per the never-delete rule.
#   domain.c    — replaced by Rust (wkssvc-rust src/wkssvc/domain.rs — NetpJoinWorkgroup/NetpGetJoinInformation)
#   info.c      — replaced by Rust (wkssvc-rust src/wkssvc/info.rs — Init/SaveWorkstationInfo + WKSTA_INFO_502)
#   rpcserver.c — replaced by Rust (wkssvc-rust src/wkssvc/rpcserver.rs — the Netr* RPC entry points)
#   wkssvc.c    — replaced by Rust (wkssvc-rust src/wkssvc/wkssvc.rs — DllMain/ServiceMain + LSA setup)
add_library(wkssvc MODULE
    ${CMAKE_CURRENT_BINARY_DIR}/wkssvc_s.c
    wkssvc.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wkssvc.def)

add_dependencies(wkssvc rust_wkssvc)
set_module_type(wkssvc win32dll UNICODE)
# Rust staticlib FIRST so its #[no_mangle] bodies win over any surviving C definition.
target_link_libraries(wkssvc ${WKSSVC_RUST_LIB} wine)
add_importlibs(wkssvc secur32 advapi32 rpcrt4 msvcrt kernel32 ntdll)
add_cd_file(TARGET wkssvc DESTINATION reactos/system32 FOR all)
