
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/browser.idl)
spec2def(browser.dll browser.spec ADD_IMPORTLIB)

# browser.dll — Rust cohabitation flip (FULL: both compiled .c → browser-rust).
# The Computer Browser service (svchost -k netsvcs; hivesys Start=2, so it really runs at
# boot): SvchostPushServiceGlobals + ServiceMain + the browser.idl RPC listener. Translated
# with DeepSeek. The .c files are COMMENTED OUT below, never deleted (per the never-delete
# rule); libbrowser_rust.a supplies both bodies via #[no_mangle] and is linked FIRST so its
# symbols win.
#
# STILL C (correctly): the MIDL products browser_s.c + browser_stubs.c — generated wire
# marshalling for browser.idl.
set(BROWSER_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(BROWSER_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(BROWSER_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(BROWSER_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
set(BROWSER_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-browser-target")
set(BROWSER_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libbrowser_rust.a")
file(GLOB_RECURSE BROWSER_RUST_SRCS "${BROWSER_RUST_WORKSPACE_DIR}/browser-rust/src/*.rs")
add_custom_command(
    OUTPUT ${BROWSER_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${BROWSER_RUST_TARGET_DIR}
        cargo build --release --target ${BROWSER_RUST_TARGET}
            -p browser-rust
            --manifest-path ${BROWSER_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${BROWSER_RUST_TARGET_DIR}/${BROWSER_RUST_TARGET}/release/libbrowser_rust.a
        ${BROWSER_RUST_LIB}
    DEPENDS ${BROWSER_RUST_SRCS}
            ${BROWSER_RUST_WORKSPACE_DIR}/browser-rust/Cargo.toml
    WORKING_DIRECTORY ${BROWSER_RUST_WORKSPACE_DIR}
    COMMENT "Building browser-rust (${BROWSER_RUST_TARGET})"
)
add_custom_target(rust_browser DEPENDS ${BROWSER_RUST_LIB})

list(APPEND SOURCE
    # browser.c   (→ browser-rust src/browser/browser.rs — DllMain/SvchostPushServiceGlobals/ServiceMain)
    # rpcserver.c (→ browser-rust src/browser/rpcserver.rs — RpcThreadRoutine)
    # precomp.h   (the PCH for the above — no C translation unit left to precompile)
    )

add_library(browser MODULE
    ${SOURCE}
    browser.rc
    ${CMAKE_CURRENT_BINARY_DIR}/browser_s.c
    ${CMAKE_CURRENT_BINARY_DIR}/browser_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/browser.def)

add_dependencies(browser rust_browser)
set_module_type(browser win32dll UNICODE)
# Rust staticlib FIRST so its #[no_mangle] bodies win over any surviving C definition.
target_link_libraries(browser ${BROWSER_RUST_LIB} wine ${PSEH_LIB})
add_importlibs(browser advapi32 rpcrt4 msvcrt kernel32 ntdll)
# add_pch removed: both .c that used precomp.h are now Rust; only the self-contained
# MIDL-generated browser_s.c / browser_stubs.c remain, which do not need (or use) the PCH.
add_cd_file(TARGET browser DESTINATION reactos/system32 FOR all)
