
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/wlansvc.idl)

# wlansvc.exe — Rust cohabitation flip (FULL: both compiled .c → wlansvc-rust).
# The WLAN AutoConfig service (own EXE; hivesys Start=2, so it really runs at boot): wmain +
# ServiceMain + the WLANSVCHANDLE list + the Rpc* wlansvc.idl server entry points, the SERVER
# half of the interface wlanapi.dll's client stubs call. Translated with DeepSeek. The .c
# files are COMMENTED OUT below, never deleted (per the never-delete rule); libwlansvc_rust.a
# supplies both bodies via #[no_mangle] and is linked FIRST so its symbols win. (An EXE, not a
# DLL — the staticlib cohabs into the EXE link exactly as it does for winlogon-rust.)
#
# STILL C (correctly): the MIDL product wlansvc_s.c — generated wire marshalling for wlansvc.idl.
set(WLANSVC_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(WLANSVC_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(WLANSVC_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(WLANSVC_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(WLANSVC_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-wlansvc-target" wlansvc-rust)
set(WLANSVC_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libwlansvc_rust.a")
rust_crate_sources(WLANSVC_RUST_SRCS "${WLANSVC_RUST_WORKSPACE_DIR}" wlansvc-rust)
add_custom_command(
    OUTPUT ${WLANSVC_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${WLANSVC_RUST_TARGET_DIR}
        cargo build --release --target ${WLANSVC_RUST_TARGET}
            -p wlansvc-rust
            --manifest-path ${WLANSVC_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${WLANSVC_RUST_TARGET_DIR}/${WLANSVC_RUST_TARGET}/release/libwlansvc_rust.a
        ${WLANSVC_RUST_LIB}
    DEPENDS ${WLANSVC_RUST_SRCS}
            ${WLANSVC_RUST_WORKSPACE_DIR}/wlansvc-rust/Cargo.toml
    WORKING_DIRECTORY ${WLANSVC_RUST_WORKSPACE_DIR}
    COMMENT "Building wlansvc-rust (${WLANSVC_RUST_TARGET})"
)
add_custom_target(rust_wlansvc DEPENDS ${WLANSVC_RUST_LIB})

list(APPEND SOURCE
    # wlansvc.c   (→ wlansvc-rust src/wlansvc/wlansvc.rs — wmain/ServiceMain + control handler)
    # rpcserver.c (→ wlansvc-rust src/wlansvc/rpcserver.rs — the Rpc* WLAN entry points)
    # precomp.h   (the PCH for the above — no C translation unit left to precompile)
    ${CMAKE_CURRENT_BINARY_DIR}/wlansvc_s.c)

add_executable(wlansvc ${SOURCE} wlansvc.rc)

add_dependencies(wlansvc rust_wlansvc)

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(wlansvc PRIVATE "-Wno-cast-calling-convention")
endif()

# Rust staticlib FIRST so its #[no_mangle] bodies win over any surviving C definition.
target_link_libraries(wlansvc ${WLANSVC_RUST_LIB} ${PSEH_LIB})

set_module_type(wlansvc win32cui UNICODE)
add_importlibs(wlansvc advapi32 rpcrt4 iphlpapi msvcrt kernel32 ntdll)
# add_pch removed: both .c that used precomp.h are now Rust; only the self-contained
# MIDL-generated wlansvc_s.c remains, which does not need (or use) the PCH.
add_cd_file(TARGET wlansvc DESTINATION reactos/system32 FOR all)
