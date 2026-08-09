
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/winsock)
spec2def(ws2help.dll ws2help.spec ADD_IMPORTLIB)

# ws2help.dll — Rust cohabitation flip (all 5 compiled src *.c → ws2help-rust). The translated
# bodies come from libws2help_rust.a (sdk/lib/rust/ws2help-rust, kept on disk per the
# never-delete rule); the C is commented out below.
#
# ws2help provides the Wah* handle-context table / APC-helper / notification-handle helpers
# ws2_32/msafd use to map SOCKET handles to per-provider context. The WAH handle-context +
# WS2IFSL EA struct layouts are bindgen-GENERATED per arch into the crate's `types` module by
# tools/rustconv/gen_driver_types.sh (profile: ws2help), not hand-transcribed.
#
# Dual-consumer note: ws2help_static is linked into ws2help.dll when DLL_EXPORT_VERSION < 0x600
# (this build: 0x502) and into ws2_32.dll when >= 0x600 — mutually exclusive. It is converted
# to an INTERFACE library forwarding libws2help_rust.a below, so BOTH consumers get the Rust
# symbols regardless of which version branch is taken.
set(WS2HELP_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(WS2HELP_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(WS2HELP_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(WS2HELP_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
set(WS2HELP_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-ws2help-target")
set(WS2HELP_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libws2help_rust.a")
rust_crate_sources(WS2HELP_RUST_SRCS "${WS2HELP_RUST_WORKSPACE_DIR}" ws2help-rust)
add_custom_command(
    OUTPUT ${WS2HELP_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${WS2HELP_RUST_TARGET_DIR}
        cargo build --release --target ${WS2HELP_RUST_TARGET}
            -p ws2help-rust
            --manifest-path ${WS2HELP_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${WS2HELP_RUST_TARGET_DIR}/${WS2HELP_RUST_TARGET}/release/libws2help_rust.a
        ${WS2HELP_RUST_LIB}
    DEPENDS ${WS2HELP_RUST_SRCS}
            ${WS2HELP_RUST_WORKSPACE_DIR}/ws2help-rust/Cargo.toml
    WORKING_DIRECTORY ${WS2HELP_RUST_WORKSPACE_DIR}
    COMMENT "Building ws2help-rust (${WS2HELP_RUST_TARGET})"
)
add_custom_target(rust_ws2help DEPENDS ${WS2HELP_RUST_LIB})

# 5 src *.c bodies replaced by Rust (ws2help-rust/src/ws2help/*.rs), kept on disk per the
# never-delete rule.
#   apc.c     — replaced by Rust (ws2help-rust src/ws2help/apc.rs — WahOpenApcHelper/WahQueueUserApc)
#   context.c — replaced by Rust (ws2help-rust src/ws2help/context.rs — the WAH handle-context hash table)
#   dllmain.c — replaced by Rust (ws2help-rust src/ws2help/dllmain.rs — Ws2HelpDllMain + Ws2helpInitialize)
#   handle.c  — replaced by Rust (ws2help-rust src/ws2help/handle.rs — WahCreate/CloseSocketHandle + WS2IFSL)
#   notify.c  — replaced by Rust (ws2help-rust src/ws2help/notify.rs — WahCreateNotificationHandle/NotifyAllProcesses)
# list(APPEND SOURCE
#     apc.c
#     context.c
#     dllmain.c
#     handle.c
#     notify.c
#     precomp.h)

# ws2help_static → INTERFACE library forwarding the Rust staticlib to whichever consumer the
# DLL_EXPORT_VERSION branch links it into (ws2help.dll here at 0x502; ws2_32.dll at >= 0x600).
add_library(ws2help_static INTERFACE)
target_link_libraries(ws2help_static INTERFACE ${WS2HELP_RUST_LIB})

add_library(ws2help MODULE
    ws2help.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ws2help.def)

set_module_type(ws2help win32dll)
if(DLL_EXPORT_VERSION LESS 0x600)
    target_link_libraries(ws2help ws2help_static)
    set_entrypoint(ws2help Ws2HelpDllMain 12)
endif()
# libws2help_rust.a provides the Wah*/Ws2HelpDllMain #[no_mangle] symbols; force the cargo build.
add_dependencies(ws2help rust_ws2help)
add_importlibs(ws2help advapi32 ws2_32 msvcrt kernel32 ntdll)
# add_pch removed: no C translation units remain (all 5 src *.c → Rust), so nothing to precompile.
add_cd_file(TARGET ws2help DESTINATION reactos/system32 FOR all)
