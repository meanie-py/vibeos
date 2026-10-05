
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

spec2def(netcfgx.dll netcfgx.spec)

# netcfgx.dll — Rust cohabitation flip (FULL: all 11 compiled src *.c → netcfgx-rust).
#
# The network-configuration COM server: the "Network Connections" component installer and the
# TCP/IP property sheets. C-STYLE COM throughout — precomp.h defines COBJMACROS, so every
# interface call in the module is the untyped macro `IFACE_Method(p,...)`, which expands to
# `(p)->lpVtbl->Method(p,...)`. The Rust stand-ins for those live in the crate's prelude and are
# GENERATED from the island's own `IFACEVtbl` structs, so an argument list cannot drift from the
# vtable it dispatches through. No C++ substrate is involved.
#
# Two things the island profile carries for this module, both load-bearing (gen_driver_types.sh):
#   * MACRO_SIGNED — a dialog module's constants are `int` in the C (control ids, window
#     messages, the LSTATUS every Reg*/Setup* call returns). bindgen's u32 default cost 715
#     E0308s on `rc != ERROR_SUCCESS` and `GetDlgItem(hDlg, IDC_*)` alone.
#   * UNBLOCK=DLGPROC — Rust compares fn-pointer ABIs NOMINALLY, and ntdll-sys spells DLGPROC
#     `extern "stdcall"` on i386 and `extern "C"` on the 64-bit targets. Neither unifies with
#     `extern "system"`. Emitting the island's own DLGPROC (which the ABI normaliser spells
#     "system" on all three) is what lets one definition of each dialog proc serve every target.
set(NETCFGX_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(NETCFGX_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(NETCFGX_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(NETCFGX_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(NETCFGX_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-netcfgx-target" netcfgx-rust)
set(NETCFGX_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libnetcfgx_rust.a")
rust_crate_sources(NETCFGX_RUST_SRCS "${NETCFGX_RUST_WORKSPACE_DIR}" netcfgx-rust)
add_custom_command(
    OUTPUT ${NETCFGX_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${NETCFGX_RUST_TARGET_DIR}
        cargo build --release --target ${NETCFGX_RUST_TARGET}
            -p netcfgx-rust
            --manifest-path ${NETCFGX_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${NETCFGX_RUST_TARGET_DIR}/${NETCFGX_RUST_TARGET}/release/libnetcfgx_rust.a
        ${NETCFGX_RUST_LIB}
    DEPENDS ${NETCFGX_RUST_SRCS}
            ${NETCFGX_RUST_WORKSPACE_DIR}/netcfgx-rust/Cargo.toml
    WORKING_DIRECTORY ${NETCFGX_RUST_WORKSPACE_DIR}
    COMMENT "Building netcfgx-rust (${NETCFGX_RUST_TARGET})"
)
add_custom_target(rust_netcfgx DEPENDS ${NETCFGX_RUST_LIB})

# All 11 compiled src *.c → netcfgx-rust (src/netcfgx/*.rs), kept on disk per the
# never-delete rule.
#   netcfgx.c classfactory.c installer.c netinstall.c netcfg_iface.c
#   netcfgbindinginterface_iface.c netcfgbindingpath_iface.c netcfgclass_iface.c
#   inetcfgcomp_iface.c tcpipconf_notify.c propertypage.c   → netcfgx-rust
# list(APPEND SOURCE
#     netcfgx.c
#     classfactory.c
#     installer.c
#     netinstall.c
#     netcfg_iface.c
#     netcfgbindinginterface_iface.c
#     netcfgbindingpath_iface.c
#     netcfgclass_iface.c
#     inetcfgcomp_iface.c
#     tcpipconf_notify.c
#     propertypage.c
#     precomp.h)

# rust_strsafe_shim.c is no longer built (retired 2026-09-25: the strsafe printf family is strsafe-rust macros that CALL
# msvcrt's variadic _snwprintf, which stable Rust can do; the file stays on disk, inert).
list(APPEND SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/netcfgx_stubs.c)

add_library(netcfgx MODULE
    ${SOURCE}
    netcfgx.rc
    ${CMAKE_CURRENT_BINARY_DIR}/netcfgx.def)

set_module_type(netcfgx win32dll UNICODE)
# libnetcfgx_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(netcfgx ${NETCFGX_RUST_LIB} uuid wine)
add_dependencies(netcfgx rust_netcfgx)
add_importlibs(netcfgx rpcrt4 setupapi advapi32 iphlpapi dhcpcsvc ole32 user32 comctl32 ws2_32 msvcrt kernel32 ntdll)
# add_pch dropped with the C: the only TU left is the generated netcfgx_stubs.c, which does not
# include precomp.h.
add_cd_file(TARGET netcfgx DESTINATION reactos/system32 FOR all)
