
# telnetd.exe — Rust cohabitation flip (FULL: both compiled .c → telnetd-rust).
# The telnet daemon: RFC 854 option negotiation (IAC DO/DONT/WILL/WONT) plus a child console
# process pumped over anonymous pipes. Translated with DeepSeek. The .c files are COMMENTED
# OUT below, never deleted (per the never-delete rule); libtelnetd_rust.a supplies both bodies
# via #[no_mangle] and is linked FIRST so its symbols win.
#
# NOT TRANSLATED, deliberately: syslog.c (308 LOC) sits in this directory but has never been in
# this SOURCE list upstream — it is dead source ReactOS does not compile. It is therefore absent
# from telnetd-rust too; paying to translate it would buy nothing. (Same shape as dhcpcsvc's
# dhcp/tree.c, which DID get translated before anyone checked the build's file set.)
#
# hivesys Start=3 (manual) — so this does NOT run at boot; a green smoke proves only that it
# links and does not perturb the boot. Exercising it needs a manual start + a telnet client.
set(TELNETD_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(TELNETD_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(TELNETD_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(TELNETD_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(TELNETD_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-telnetd-target" telnetd-rust)
set(TELNETD_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libtelnetd_rust.a")
rust_crate_sources(TELNETD_RUST_SRCS "${TELNETD_RUST_WORKSPACE_DIR}" telnetd-rust)
add_custom_command(
    OUTPUT ${TELNETD_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${TELNETD_RUST_TARGET_DIR}
        cargo build --release --target ${TELNETD_RUST_TARGET}
            -p telnetd-rust
            --manifest-path ${TELNETD_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${TELNETD_RUST_TARGET_DIR}/${TELNETD_RUST_TARGET}/release/libtelnetd_rust.a
        ${TELNETD_RUST_LIB}
    DEPENDS ${TELNETD_RUST_SRCS}
            ${TELNETD_RUST_WORKSPACE_DIR}/telnetd-rust/Cargo.toml
    WORKING_DIRECTORY ${TELNETD_RUST_WORKSPACE_DIR}
    COMMENT "Building telnetd-rust (${TELNETD_RUST_TARGET})"
)
add_custom_target(rust_telnetd DEPENDS ${TELNETD_RUST_LIB})

add_executable(telnetd
    # telnetd.c      (→ telnetd-rust src/telnetd/telnetd.rs — main, listener, negotiation, pipe pumps)
    # serviceentry.c (→ telnetd-rust src/telnetd/serviceentry.rs — ServiceMain + control handler)
    telnetd.rc)

# No C translation units remain (both .c → Rust), so cmake cannot infer a link language from
# the sources — only telnetd.rc is left. State it explicitly; every body comes from the
# staticlib below. (ws2help.dll is the same shape on the DLL side.)
set_target_properties(telnetd PROPERTIES LINKER_LANGUAGE C)

add_dependencies(telnetd rust_telnetd)
# Rust staticlib FIRST so its #[no_mangle] bodies win over any surviving C definition.
target_link_libraries(telnetd ${TELNETD_RUST_LIB} wine)
set_module_type(telnetd win32cui)
add_importlibs(telnetd advapi32 ws2_32 msvcrt kernel32 ntdll)
add_cd_file(TARGET telnetd DESTINATION reactos/system32 FOR all)
