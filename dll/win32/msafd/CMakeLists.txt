
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers)

spec2def(msafd.dll msafd.spec)

# msafd.dll — Rust cohabitation flip (all 5 misc/*.c → msafd-rust). The translated
# bodies (WSP* Winsock service-provider entrypoints, DllMain, async-select worker,
# helper-DLL glue) come from libmsafd_rust.a (sdk/lib/rust/msafd-rust, kept on disk
# per the never-delete rule); the C is commented out below. Next rung up the
# netstack→winsock vertical above afd.sys: ws2_32's WSP* calls land here and become
# IOCTL_AFD_* DeviceIoControls down to \Device\Afd (the Rust afd.sys).
set(MSAFD_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(MSAFD_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(MSAFD_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(MSAFD_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(MSAFD_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-msafd-target" msafd-rust)
set(MSAFD_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libmsafd_rust.a")
rust_crate_sources(MSAFD_RUST_SRCS "${MSAFD_RUST_WORKSPACE_DIR}" msafd-rust)
add_custom_command(
    OUTPUT ${MSAFD_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${MSAFD_RUST_TARGET_DIR}
        cargo build --release --target ${MSAFD_RUST_TARGET}
            -p msafd-rust
            --manifest-path ${MSAFD_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${MSAFD_RUST_TARGET_DIR}/${MSAFD_RUST_TARGET}/release/libmsafd_rust.a
        ${MSAFD_RUST_LIB}
    DEPENDS ${MSAFD_RUST_SRCS}
            ${MSAFD_RUST_WORKSPACE_DIR}/msafd-rust/Cargo.toml
    WORKING_DIRECTORY ${MSAFD_RUST_WORKSPACE_DIR}
    COMMENT "Building msafd-rust (${MSAFD_RUST_TARGET})"
)
add_custom_target(rust_msafd DEPENDS ${MSAFD_RUST_LIB})

# 5 misc/*.c bodies replaced by Rust (msafd-rust), kept on disk per the never-delete rule:
#   misc/dllmain.c — replaced by Rust (msafd-rust/src/misc/dllmain.rs); DllMain + WSPStartup + most WSP*
#   misc/event.c   — replaced by Rust (msafd-rust/src/misc/event.rs); WSPEventSelect/WSPEnumNetworkEvents
#   misc/helpers.c — replaced by Rust (msafd-rust/src/misc/helpers.rs); helper-DLL load/transport list
#   misc/sndrcv.c  — replaced by Rust (msafd-rust/src/misc/sndrcv.rs); WSPSend/Recv/AsyncSelect
#   misc/stubs.c   — replaced by Rust (msafd-rust/src/misc/stubs.rs); WSPAcceptEx/DisconnectEx/... stubs
# list(APPEND SOURCE misc/dllmain.c misc/event.c misc/helpers.c misc/sndrcv.c misc/stubs.c)
list(APPEND SOURCE
    msafd.h)

add_library(msafd MODULE
    ${SOURCE}
    msafd.rc
    ${CMAKE_CURRENT_BINARY_DIR}/msafd.def)

set_module_type(msafd win32dll UNICODE)
# libmsafd_rust.a first so its #[no_mangle] WSP*/DllMain symbols win. --allow-multiple-definition
# is harmless here (no C bodies remain) but kept for parity with the other cohab flips.
target_link_libraries(msafd ${MSAFD_RUST_LIB} wine)
add_dependencies(msafd rust_msafd)
add_importlibs(msafd advapi32 ws2_32 msvcrt kernel32 ntdll)
# add_pch removed: no C translation units remain (all misc/*.c → Rust), so nothing to precompile.
add_cd_file(TARGET msafd DESTINATION reactos/system32 FOR all)
