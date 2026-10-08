
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/winsock)
spec2def(rasadhlp.dll rasadhlp.spec)

# rasadhlp.dll — Rust cohabitation flip (FULL: all 3 compiled src *.c → rasadhlp-rust).
# The winsock NSP autodial hook: WSAttemptAutodialAddr/Name are called by ws2_32 when a
# name resolution fails, and they IOCTL \Device\RasAcd to trigger a dial-up attempt. It
# therefore rides on the ws2_32 surface already translated. autodial.c/init.c/winsock.c
# are commented out below and kept on disk per the never-delete rule.
#
# The island uses REAL Rust unions (REAL_UNIONS=1 in gen_driver_types.sh): AUTODIAL_ADDR
# and AUTODIAL_CONN are tagged unions whose payload is an ANONYMOUS union, so the C reads
# `Addr.Ip4Address` while bindgen names the member `__bindgen_anon_1`.
set(RASADHLP_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(RASADHLP_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(RASADHLP_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(RASADHLP_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(RASADHLP_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-rasadhlp-target" rasadhlp-rust)
set(RASADHLP_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/librasadhlp_rust.a")
rust_crate_sources(RASADHLP_RUST_SRCS "${RASADHLP_RUST_WORKSPACE_DIR}" rasadhlp-rust)
add_custom_command(
    OUTPUT ${RASADHLP_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RASADHLP_RUST_TARGET_DIR}
        cargo build --release --target ${RASADHLP_RUST_TARGET}
            -p rasadhlp-rust
            --manifest-path ${RASADHLP_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${RASADHLP_RUST_TARGET_DIR}/${RASADHLP_RUST_TARGET}/release/librasadhlp_rust.a
        ${RASADHLP_RUST_LIB}
    DEPENDS ${RASADHLP_RUST_SRCS}
            ${RASADHLP_RUST_WORKSPACE_DIR}/rasadhlp-rust/Cargo.toml
    WORKING_DIRECTORY ${RASADHLP_RUST_WORKSPACE_DIR}
    COMMENT "Building rasadhlp-rust (${RASADHLP_RUST_TARGET})"
)
add_custom_target(rust_rasadhlp DEPENDS ${RASADHLP_RUST_LIB})

# All 3 compiled src *.c → rasadhlp-rust (src/rasadhlp/*.rs), kept on disk per the
# never-delete rule.
#   autodial.c init.c winsock.c  → rasadhlp-rust
# list(APPEND SOURCE
#     autodial.c
#     init.c
#     winsock.c
#     precomp.h)
list(APPEND SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/rasadhlp_stubs.c)

add_library(rasadhlp MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/rasadhlp.def)

set_module_type(rasadhlp win32dll)
# librasadhlp_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(rasadhlp ${RASADHLP_RUST_LIB})
add_dependencies(rasadhlp rust_rasadhlp)
add_importlibs(rasadhlp ws2_32 msvcrt kernel32 ntdll)
# add_pch dropped with precomp.h: the only remaining C is generated (rasadhlp_stubs.c)
# and it does not include it.
add_cd_file(TARGET rasadhlp DESTINATION reactos/system32 FOR all)
