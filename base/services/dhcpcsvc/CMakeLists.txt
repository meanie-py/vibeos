include_directories(include ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)

add_rpc_file(client ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/dhcpcsvc.idl PREFIX_CLIENT Client_ PREFIX_SERVER Server_)
add_rpc_file(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/dhcpcsvc.idl PREFIX_CLIENT Client_ PREFIX_SERVER Server_)

spec2def(dhcpcsvc.dll dhcpcsvc.spec ADD_IMPORTLIB)

# dhcpcsvc.dll — Rust cohabitation flip (FULL: all 12 compiled .c → dhcpcsvc-rust).
# The DHCP client service: the DhcpC* API + the BSD-derived ISC dhclient state machine (dhcp/).
# Translated with DeepSeek-reasoner (pure-LLM — c2rust cannot anchor these). The .c files are
# COMMENTED OUT below, never deleted (per the never-delete rule); libdhcpcsvc_rust.a supplies
# every body via #[no_mangle] and is linked FIRST so its symbols win.
#
# STILL C (correctly): the three MIDL products dhcpcsvc_c.c / _s.c / _stubs.c (generated wire
# marshalling for the single dhcpcsvc.idl interface — the Rust side declares the Client_* stubs
# and calls them), and rust_varargs_shim.c below.
#
# NOT TRANSLATED-AND-DROPPED: dhcp/tree.c is absent from this SOURCE list UPSTREAM — it is dead
# ISC parse-tree source that ReactOS never compiled (its fns are referenced only from within
# tree.c itself). DeepSeek translated it anyway; the output is kept at dhcpcsvc-rust/src/dhcp/
# tree.rs but is excluded in that crate's lib.rs to mirror this file set. It cannot compile
# against ReactOS's trimmed tree.h, and its file-static do_host_lookup was never translated —
# both are noted in lib.rs should tree.c ever be brought into the build.
set(DHCPCSVC_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(DHCPCSVC_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(DHCPCSVC_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(DHCPCSVC_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(DHCPCSVC_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-dhcpcsvc-target" dhcpcsvc-rust)
set(DHCPCSVC_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libdhcpcsvc_rust.a")
rust_crate_sources(DHCPCSVC_RUST_SRCS "${DHCPCSVC_RUST_WORKSPACE_DIR}" dhcpcsvc-rust)
add_custom_command(
    OUTPUT ${DHCPCSVC_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${DHCPCSVC_RUST_TARGET_DIR}
        cargo build --release --target ${DHCPCSVC_RUST_TARGET}
            -p dhcpcsvc-rust
            --manifest-path ${DHCPCSVC_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${DHCPCSVC_RUST_TARGET_DIR}/${DHCPCSVC_RUST_TARGET}/release/libdhcpcsvc_rust.a
        ${DHCPCSVC_RUST_LIB}
    DEPENDS ${DHCPCSVC_RUST_SRCS}
            ${DHCPCSVC_RUST_WORKSPACE_DIR}/dhcpcsvc-rust/Cargo.toml
    WORKING_DIRECTORY ${DHCPCSVC_RUST_WORKSPACE_DIR}
    COMMENT "Building dhcpcsvc-rust (${DHCPCSVC_RUST_TARGET})"
)
add_custom_target(rust_dhcpcsvc DEPENDS ${DHCPCSVC_RUST_LIB})

list(APPEND SOURCE
    # dhcp/adapter.c   (→ dhcpcsvc-rust src/dhcp/adapter.rs)
    # dhcp/alloc.c     (→ dhcpcsvc-rust src/dhcp/alloc.rs)
    # dhcp/api.c       (→ dhcpcsvc-rust src/dhcp/api.rs)
    # dhcp/compat.c    (→ dhcpcsvc-rust src/dhcp/compat.rs)
    # dhcp/dhclient.c  (→ dhcpcsvc-rust src/dhcp/dhclient.rs — the state machine, the big one)
    # dhcp/dispatch.c  (→ dhcpcsvc-rust src/dhcp/dispatch.rs)
    # dhcp/hash.c      (→ dhcpcsvc-rust src/dhcp/hash.rs)
    # dhcp/options.c   (→ dhcpcsvc-rust src/dhcp/options.rs)
    # dhcp/socket.c    (→ dhcpcsvc-rust src/dhcp/socket.rs)
    # dhcp/tables.c    (→ dhcpcsvc-rust src/dhcp/tables.rs)
    # dhcp/util.c      (→ dhcpcsvc-rust src/dhcp/util.rs + rust_varargs_shim.c below)
    # dhcpcsvc.c       (→ dhcpcsvc-rust src/dhcpcsvc/dhcpcsvc.rs — ServiceMain + RPC server glue)
    # C-variadic ENTRY points for util.c's ISC log helpers (note/warn/warning/debug/error).
    # Stable Rust cannot DEFINE a variadic fn, so these do the va_start and tail-call the
    # fixed-arity *_v twins in dhcpcsvc-rust/src/dhcp/util.rs, which hold the translated logic.
    # This is the sanctioned varargs-rust trampoline, NOT a translation reverted to C.
    rust_varargs_shim.c)

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/dhcpcsvc_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/dhcpcsvc_s.c
    ${CMAKE_CURRENT_BINARY_DIR}/dhcpcsvc_stubs.c)

add_library(dhcpcsvc MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    dhcpcsvc.rc
    ${CMAKE_CURRENT_BINARY_DIR}/dhcpcsvc.def)

add_dependencies(dhcpcsvc rust_dhcpcsvc)
set_module_type(dhcpcsvc win32dll)
# Rust staticlib FIRST so its #[no_mangle] bodies win over any surviving C definition.
target_link_libraries(dhcpcsvc ${DHCPCSVC_RUST_LIB} oldnames ${PSEH_LIB})
add_importlibs(dhcpcsvc ws2_32 rpcrt4 iphlpapi advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET dhcpcsvc DESTINATION reactos/system32 FOR all)
