
add_definitions(-DLE)
spec2def(ws2_32.dll ws2_32.spec ADD_IMPORTLIB)

include_directories(
    inc
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/winsock)

# ws2_32.dll — Rust cohabitation flip (all 34 compiled src/*.c → ws2_32-rust). The
# translated bodies come from libws2_32_rust.a (sdk/lib/rust/ws2_32-rust, kept on disk per
# the never-delete rule); the C is commented out below.
#
# TOP of the netstack→winsock vertical: apps call the public sockets API here, ws2_32
# dispatches through its transport-provider catalog into msafd's WSP* entry points, which
# become IOCTL_AFD_* DeviceIoControls down to \Device\Afd (the Rust afd.sys) → netstack →
# smoltcp. Every rung below this is already Rust (msafd-rust, afd-rust).
#
# The private object model (TCATALOG/TPROVIDER, NSCATALOG/NSQUERY, WSPROCESS/WSTHREAD/
# WSSOCKET, WSASYNCBLOCK — inc/ws2_32p.h) is bindgen-GENERATED per arch into the crate's
# `types` module by tools/rustconv/gen_driver_types.sh, not hand-transcribed: these are
# refcounted objects with hot field access on every socket call, so a mis-transcribed
# offset would be a silent ABI break (the scan_magic_offsets class).
#
# Validated by tests/qemu/test_net.py::test_usermode_winsock_roundtrip — a REAL usermode
# nettest.exe doing TCP + UDP round-trips through this DLL — not just by a boot.
set(WS2_32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(WS2_32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(WS2_32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(WS2_32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(WS2_32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-ws2_32-target" ws2_32-rust)
set(WS2_32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libws2_32_rust.a")
rust_crate_sources(WS2_32_RUST_SRCS "${WS2_32_RUST_WORKSPACE_DIR}" ws2_32-rust)
add_custom_command(
    OUTPUT ${WS2_32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${WS2_32_RUST_TARGET_DIR}
        cargo build --release --target ${WS2_32_RUST_TARGET}
            -p ws2_32-rust
            --manifest-path ${WS2_32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${WS2_32_RUST_TARGET_DIR}/${WS2_32_RUST_TARGET}/release/libws2_32_rust.a
        ${WS2_32_RUST_LIB}
    DEPENDS ${WS2_32_RUST_SRCS}
            ${WS2_32_RUST_WORKSPACE_DIR}/ws2_32-rust/Cargo.toml
    WORKING_DIRECTORY ${WS2_32_RUST_WORKSPACE_DIR}
    COMMENT "Building ws2_32-rust (${WS2_32_RUST_TARGET})"
)
add_custom_target(rust_ws2_32 DEPENDS ${WS2_32_RUST_LIB})

# 34 src/*.c bodies replaced by Rust (ws2_32-rust/src/src/*.rs), kept on disk per the
# never-delete rule. (src/scihlpr.c was already not compiled — it stays that way.)
#   src/addrconv.c — replaced by Rust (ws2_32-rust src/addrconv.rs — inet_addr/ntoa/pton/ntop + hton/ntoh family)
#   src/addrinfo.c — replaced by Rust (ws2_32-rust src/addrinfo.rs — getaddrinfo/GetAddrInfoW/getnameinfo + DNS lookup)
#   src/async.c    — replaced by Rust (ws2_32-rust src/async.rs — the WSAAsync* window-message worker thread)
#   src/bhook.c    — replaced by Rust (ws2_32-rust src/bhook.rs — the blocking-hook stubs)
#   src/dcatalog.c — replaced by Rust (ws2_32-rust src/dcatalog.rs — TCATALOG: the transport-provider catalog)
#   src/dcatitem.c — replaced by Rust (ws2_32-rust src/dcatitem.rs — TCATALOG_ENTRY)
#   src/dllmain.c  — replaced by Rust (ws2_32-rust src/dllmain.rs — DllMain + WsCreate/DestroyStartupSynchronization)
#   src/dprocess.c — replaced by Rust (ws2_32-rust src/dprocess.rs — WSPROCESS: the per-process object)
#   src/dprovide.c — replaced by Rust (ws2_32-rust src/dprovide.rs — TPROVIDER: a loaded WSP service provider)
#   src/dsocket.c  — replaced by Rust (ws2_32-rust src/dsocket.rs — WSSOCKET: the per-socket object + handle table)
#   src/dthread.c  — replaced by Rust (ws2_32-rust src/dthread.rs — WSTHREAD: the per-thread object)
#   src/dupsock.c  — replaced by Rust (ws2_32-rust src/dupsock.rs — WSADuplicateSocket)
#   src/enumprot.c — replaced by Rust (ws2_32-rust src/enumprot.rs — WSAEnumProtocols)
#   src/event.c    — replaced by Rust (ws2_32-rust src/event.rs — WSACreateEvent/SetEvent/WaitForMultipleEvents)
#   src/getproto.c — replaced by Rust (ws2_32-rust src/getproto.rs — getprotobyname/number)
#   src/getxbyxx.c — replaced by Rust (ws2_32-rust src/getxbyxx.rs — gethostbyname/addr, getservbyname/port)
#   src/ioctl.c    — replaced by Rust (ws2_32-rust src/ioctl.rs — ioctlsocket/WSAIoctl)
#   src/nscatalo.c — replaced by Rust (ws2_32-rust src/nscatalo.rs — NSCATALOG: the name-space catalog)
#   src/nscatent.c — replaced by Rust (ws2_32-rust src/nscatent.rs — NSCATALOG_ENTRY)
#   src/nspinstl.c — replaced by Rust (ws2_32-rust src/nspinstl.rs — WSCInstall/UnInstallNameSpace)
#   src/nsprovid.c — replaced by Rust (ws2_32-rust src/nsprovid.rs — NSPROVIDER: a loaded NSP)
#   src/nsquery.c  — replaced by Rust (ws2_32-rust src/nsquery.rs — NSQUERY: WSALookupService*)
#   src/qos.c      — replaced by Rust (ws2_32-rust src/qos.rs — the QOS stubs)
#   src/qshelpr.c  — replaced by Rust (ws2_32-rust src/qshelpr.rs — the WSALookupService helper/marshalling layer)
#   src/rasdial.c  — replaced by Rust (ws2_32-rust src/rasdial.rs — the RAS auto-dial stubs)
#   src/recv.c     — replaced by Rust (ws2_32-rust src/recv.rs — recv/recvfrom/WSARecv/WSARecvFrom)
#   src/rnr.c      — replaced by Rust (ws2_32-rust src/rnr.rs — the RNR (registration & name resolution) API)
#   src/select.c   — replaced by Rust (ws2_32-rust src/select.rs — select/__WSAFDIsSet)
#   src/send.c     — replaced by Rust (ws2_32-rust src/send.rs — send/sendto/WSASend/WSASendTo)
#   src/sockctrl.c — replaced by Rust (ws2_32-rust src/sockctrl.rs — bind/listen/connect/shutdown/get*name/get-setsockopt)
#   src/socklife.c — replaced by Rust (ws2_32-rust src/socklife.rs — socket/WSASocket/accept/WSAAccept/closesocket)
#   src/spinstal.c — replaced by Rust (ws2_32-rust src/spinstal.rs — WSCInstall/DeinstallProvider)
#   src/sputil.c   — replaced by Rust (ws2_32-rust src/sputil.rs — service-provider utility helpers)
#   src/startup.c  — replaced by Rust (ws2_32-rust src/startup.rs — WSAStartup/WSACleanup/WSApSetPostRoutine)
#   src/wsautil.c  — replaced by Rust (ws2_32-rust src/wsautil.rs — SetLastError-ish error plumbing + prologs)
# list(APPEND SOURCE
#     src/addrconv.c src/addrinfo.c src/async.c src/bhook.c src/dcatalog.c src/dcatitem.c
#     src/dllmain.c src/dprocess.c src/dprovide.c src/dsocket.c src/dthread.c src/dupsock.c
#     src/enumprot.c src/event.c src/getproto.c src/getxbyxx.c src/ioctl.c src/nscatalo.c
#     src/nscatent.c src/nspinstl.c src/nsprovid.c src/nsquery.c src/qos.c src/qshelpr.c
#     src/rasdial.c src/recv.c src/rnr.c src/select.c src/send.c src/sockctrl.c
#     src/socklife.c src/spinstal.c src/sputil.c src/startup.c src/wsautil.c)
list(APPEND SOURCE
    # src/wine.c — new upstream file adding one function, inet_addr(), which
    # duplicates ws2_32-rust's addrconv.rs (already owns inet_addr/ntoa/pton/
    # ntop). Declined: would collide with a Rust-flipped symbol.
    inc/ws2_32.h)

add_library(ws2_32 MODULE
    ${SOURCE}
    ws2_32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ws2_32_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/ws2_32.def)

set_module_type(ws2_32 win32dll)
# libws2_32_rust.a first so its #[no_mangle] symbols win. PSEH is no longer reachable (no C
# translation units remain), but the link is harmless and kept for parity with the other flips.
target_link_libraries(ws2_32 ${WS2_32_RUST_LIB} ${PSEH_LIB})
add_dependencies(ws2_32 rust_ws2_32)
if(DLL_EXPORT_VERSION GREATER_EQUAL 0x600)
    target_link_libraries(ws2_32 ws2help_static)
endif()
add_importlibs(ws2_32 user32 advapi32 ws2help msvcrt kernel32 ntdll)
# add_pch removed: no C translation units remain (all src/*.c → Rust), so nothing to precompile.
add_cd_file(TARGET ws2_32 DESTINATION reactos/system32 FOR all)
