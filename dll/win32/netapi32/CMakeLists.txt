
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)

spec2def(netapi32.dll netapi32.spec ADD_IMPORTLIB)
add_rpc_files(client
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/atsvc.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/browser.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/dssetup.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/netdfs.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/netlogon.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/srvsvc.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/wkssvc.idl)

# netapi32.dll — Rust cohabitation flip (PARTIAL: 14 of the 21 compiled src *.c → netapi32-rust).
# The translated bodies come from libnetapi32_rust.a (sdk/lib/rust/netapi32-rust, DeepSeek-V4 +
# c2rust, kept on disk per the never-delete rule); those 14 .c are commented out below. The 7
# RPC-stub-heavy files (browser/dssetup/misc/netlogon/schedule/srvsvc/wksta_new) CANNOT be
# c2rust-anchored, so they stay C and are OWED a pure-LLM translation pass — netapi32.dll links
# both halves (no symbol overlap: verified disjoint). The MIDL *_c.c client stubs + netapi32_stubs.c
# also stay C. `wine` still provides ros_dbg_log (the debug channel the Rust prelude declares
# extern) + wine unicode; keep it linked.
set(NETAPI32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(NETAPI32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(NETAPI32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(NETAPI32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
set(NETAPI32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-netapi32-target")
set(NETAPI32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libnetapi32_rust.a")
file(GLOB_RECURSE NETAPI32_RUST_SRCS "${NETAPI32_RUST_WORKSPACE_DIR}/netapi32-rust/src/*.rs")
add_custom_command(
    OUTPUT ${NETAPI32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${NETAPI32_RUST_TARGET_DIR}
        cargo build --release --target ${NETAPI32_RUST_TARGET}
            -p netapi32-rust
            --manifest-path ${NETAPI32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${NETAPI32_RUST_TARGET_DIR}/${NETAPI32_RUST_TARGET}/release/libnetapi32_rust.a
        ${NETAPI32_RUST_LIB}
    DEPENDS ${NETAPI32_RUST_SRCS}
            ${NETAPI32_RUST_WORKSPACE_DIR}/netapi32-rust/Cargo.toml
    WORKING_DIRECTORY ${NETAPI32_RUST_WORKSPACE_DIR}
    COMMENT "Building netapi32-rust (${NETAPI32_RUST_TARGET})"
)
add_custom_target(rust_netapi32 DEPENDS ${NETAPI32_RUST_LIB})

# 14 src *.c bodies replaced by Rust (netapi32-rust/src/netapi32/*.rs), kept on disk per the
# never-delete rule. (DllMain lives in netapi32.rs; win32dll's default entry calls it.)
#   apibuf.c dfs.c display.c group_new.c local_group.c nbcmdqueue.c nbnamecache.c
#   nbt.c netapi32.c netbios.c obsolete.c user.c utils.c wksta.c  → netapi32-rust
# The 7 files below stay C (cannot c2rust-anchor; owe a pure-LLM pass).
list(APPEND SOURCE
    # apibuf.c        (→ netapi32-rust src/netapi32/apibuf.rs)
    browser.c
    # dfs.c           (→ netapi32-rust src/netapi32/dfs.rs)
    # display.c       (→ netapi32-rust src/netapi32/display.rs)
    dssetup.c
    # group_new.c     (→ netapi32-rust src/netapi32/group_new.rs)
    # local_group.c   (→ netapi32-rust src/netapi32/local_group.rs)
    misc.c
    # nbcmdqueue.c    (→ netapi32-rust src/netapi32/nbcmdqueue.rs)
    # nbnamecache.c   (→ netapi32-rust src/netapi32/nbnamecache.rs)
    # nbt.c           (→ netapi32-rust src/netapi32/nbt.rs)
    # netapi32.c      (→ netapi32-rust src/netapi32/netapi32.rs — DllMain)
    # netbios.c       (→ netapi32-rust src/netapi32/netbios.rs)
    netlogon.c
    # obsolete.c      (→ netapi32-rust src/netapi32/obsolete.rs)
    schedule.c
    srvsvc.c
    # user.c          (→ netapi32-rust src/netapi32/user.rs)
    # utils.c         (→ netapi32-rust src/netapi32/utils.rs)
    # wksta.c         (→ netapi32-rust src/netapi32/wksta.rs)
    wksta_new.c)

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/netapi32_stubs.c)

add_library(netapi32 MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/atsvc_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/browser_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/dssetup_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/netdfs_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/netlogon_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/srvsvc_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/wkssvc_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/netapi32.def)

set_module_type(netapi32 win32dll)
# libnetapi32_rust.a FIRST so its #[no_mangle] symbols win; wine provides ros_dbg_log + unicode.
target_link_libraries(netapi32 ${NETAPI32_RUST_LIB} wine ${PSEH_LIB})
add_dependencies(netapi32 rust_netapi32)
add_delay_importlibs(netapi32 samlib secur32)
add_importlibs(netapi32 iphlpapi ws2_32 advapi32 rpcrt4 msvcrt kernel32 ntdll)
# add_pch kept: the 7 remaining C files (browser/dssetup/misc/netlogon/schedule/srvsvc/wksta_new)
# still use the netapi32.h PCH.
add_pch(netapi32 netapi32.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET netapi32 DESTINATION reactos/system32 FOR all)

if(MSVC)
    # Disable warning C4267: '=': conversion from 'size_t' to 'USHORT', possible loss of data
    target_compile_options(netapi32 PRIVATE /wd4267)
endif()
