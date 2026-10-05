
spec2def(netid.dll netid.spec)

# netid.dll — Rust cohabitation flip (FULL: the single compiled src *.c → netid-rust).
# The "Computer Name" system-properties page. The prsht.h notification codes it dispatches on
# are macro EXPRESSIONS (PSN_APPLY = PSN_FIRST - 2), so island completeness is a correctness
# property here, not a convenience (field manual §9.112) — every PSN_* is verified present in
# the generated types_<arch>.rs. netid.c is commented out below and kept on disk per the
# never-delete rule.
#
# netid_varargs_shim.c is NOT a translation left in C: stable Rust cannot DEFINE a C-variadic
# function, and `va_start` may only appear inside one. That file supplies the variadic entry
# point for FormatMessageBox and immediately hands control back to Rust with a POINTER to the
# va_list; every line of logic lives in netid-rust. See sdk/lib/rust/varargs-rust.
set(NETID_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(NETID_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(NETID_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(NETID_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(NETID_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-netid-target" netid-rust)
set(NETID_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libnetid_rust.a")
rust_crate_sources(NETID_RUST_SRCS "${NETID_RUST_WORKSPACE_DIR}" netid-rust)
add_custom_command(
    OUTPUT ${NETID_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${NETID_RUST_TARGET_DIR}
        cargo build --release --target ${NETID_RUST_TARGET}
            -p netid-rust
            --manifest-path ${NETID_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${NETID_RUST_TARGET_DIR}/${NETID_RUST_TARGET}/release/libnetid_rust.a
        ${NETID_RUST_LIB}
    DEPENDS ${NETID_RUST_SRCS}
            ${NETID_RUST_WORKSPACE_DIR}/netid-rust/Cargo.toml
    WORKING_DIRECTORY ${NETID_RUST_WORKSPACE_DIR}
    COMMENT "Building netid-rust (${NETID_RUST_TARGET})"
)
add_custom_target(rust_netid DEPENDS ${NETID_RUST_LIB})

# The only compiled src *.c → netid-rust (src/netid/netid.rs), kept on disk per the
# never-delete rule.
#   netid.c  → netid-rust
# list(APPEND SOURCE
#     netid.c)
list(APPEND SOURCE
    netid_varargs_shim.c
    netid.rc
    ${CMAKE_CURRENT_BINARY_DIR}/netid.def)

add_library(netid MODULE ${SOURCE})
set_module_type(netid win32dll UNICODE)
# libnetid_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(netid ${NETID_RUST_LIB})
add_dependencies(netid rust_netid)
add_importlibs(netid user32 gdi32 comctl32 advapi32 netapi32 dnsapi msvcrt kernel32 ntdll)
add_cd_file(TARGET netid DESTINATION reactos/system32 FOR all)
