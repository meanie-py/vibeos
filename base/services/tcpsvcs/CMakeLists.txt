
# tcpsvcs.exe — Rust cohabitation flip (FULL: all 8 compiled .c → tcpsvcs-rust).
# The "simple TCP/IP services": the RFC 862/863/867/865/864 quintet (echo, discard, daytime,
# qotd, chargen) behind one thread-per-service accept skeleton, plus the file/eventlog logger.
# A pure ws2_32 CLIENT of the winsock stack this lane flipped earlier. Translated with
# DeepSeek. The .c files are COMMENTED OUT below, never deleted (per the never-delete rule);
# libtcpsvcs_rust.a supplies every body via #[no_mangle] and is linked FIRST so its symbols win.
#
# hivesys Start=3 (manual) — so unlike wkssvc/browser/wlansvc this does NOT run at boot, and a
# green smoke proves only that it links and does not perturb the boot. Exercising it needs a
# manual service start + a client on ports 7/9/13/17/19.
set(TCPSVCS_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(TCPSVCS_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(TCPSVCS_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(TCPSVCS_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
set(TCPSVCS_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-tcpsvcs-target")
set(TCPSVCS_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libtcpsvcs_rust.a")
rust_crate_sources(TCPSVCS_RUST_SRCS "${TCPSVCS_RUST_WORKSPACE_DIR}" tcpsvcs-rust)
add_custom_command(
    OUTPUT ${TCPSVCS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${TCPSVCS_RUST_TARGET_DIR}
        cargo build --release --target ${TCPSVCS_RUST_TARGET}
            -p tcpsvcs-rust
            --manifest-path ${TCPSVCS_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${TCPSVCS_RUST_TARGET_DIR}/${TCPSVCS_RUST_TARGET}/release/libtcpsvcs_rust.a
        ${TCPSVCS_RUST_LIB}
    DEPENDS ${TCPSVCS_RUST_SRCS}
            ${TCPSVCS_RUST_WORKSPACE_DIR}/tcpsvcs-rust/Cargo.toml
    WORKING_DIRECTORY ${TCPSVCS_RUST_WORKSPACE_DIR}
    COMMENT "Building tcpsvcs-rust (${TCPSVCS_RUST_TARGET})"
)
add_custom_target(rust_tcpsvcs DEPENDS ${TCPSVCS_RUST_LIB})

list(APPEND SOURCE
    # tcpsvcs.c    (→ tcpsvcs-rust src/tcpsvcs/tcpsvcs.rs — wmain/ServiceMain + control handler)
    # skelserver.c (→ tcpsvcs-rust src/tcpsvcs/skelserver.rs — StartServer/ShutdownConnection)
    # echo.c       (→ tcpsvcs-rust src/tcpsvcs/echo.rs — RFC 862)
    # discard.c    (→ tcpsvcs-rust src/tcpsvcs/discard.rs — RFC 863)
    # daytime.c    (→ tcpsvcs-rust src/tcpsvcs/daytime.rs — RFC 867)
    # qotd.c       (→ tcpsvcs-rust src/tcpsvcs/qotd.rs — RFC 865)
    # chargen.c    (→ tcpsvcs-rust src/tcpsvcs/chargen.rs — RFC 864)
    # log.c        (→ tcpsvcs-rust src/tcpsvcs/log.rs — InitLogging/LogEvent)
    # tcpsvcs.h    (the PCH for the above — no C translation unit left to precompile)
    )

add_executable(tcpsvcs ${SOURCE} tcpsvcs.rc)

# No C translation units remain (all 8 .c → Rust), so cmake cannot infer a link language from
# the sources — only tcpsvcs.rc is left. State it explicitly; every body comes from the
# staticlib below. (ws2help.dll is the same shape on the DLL side.)
set_target_properties(tcpsvcs PROPERTIES LINKER_LANGUAGE C)

add_dependencies(tcpsvcs rust_tcpsvcs)
set_module_type(tcpsvcs win32cui UNICODE)
# Rust staticlib FIRST so its #[no_mangle] bodies win over any surviving C definition.
target_link_libraries(tcpsvcs ${TCPSVCS_RUST_LIB})
add_importlibs(tcpsvcs ws2_32 advapi32 msvcrt kernel32 ntdll)
# add_pch removed: no C translation units remain (all 8 .c → Rust), so nothing to precompile.
add_cd_file(TARGET tcpsvcs DESTINATION reactos/system32 FOR all)
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/quotes DESTINATION reactos/system32/drivers/etc FOR all)
