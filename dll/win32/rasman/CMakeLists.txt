
spec2def(rasman.dll rasman.spec)

# rasman.dll — Rust cohabitation flip (FULL: the one compiled src *.c → rasman-rust).
# The RAS connection manager's security-dialog entry points; Wine-sourced stubs, 5 of 7
# functions lowered deterministically for $0. rasman.c is commented out below and kept
# on disk per the never-delete rule. rasman.rc stays (resources are not translated).
set(RASMAN_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(RASMAN_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(RASMAN_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(RASMAN_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(RASMAN_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-rasman-target" rasman-rust)
set(RASMAN_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/librasman_rust.a")
rust_crate_sources(RASMAN_RUST_SRCS "${RASMAN_RUST_WORKSPACE_DIR}" rasman-rust)
add_custom_command(
    OUTPUT ${RASMAN_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RASMAN_RUST_TARGET_DIR}
        cargo build --release --target ${RASMAN_RUST_TARGET}
            -p rasman-rust
            --manifest-path ${RASMAN_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${RASMAN_RUST_TARGET_DIR}/${RASMAN_RUST_TARGET}/release/librasman_rust.a
        ${RASMAN_RUST_LIB}
    DEPENDS ${RASMAN_RUST_SRCS}
            ${RASMAN_RUST_WORKSPACE_DIR}/rasman-rust/Cargo.toml
    WORKING_DIRECTORY ${RASMAN_RUST_WORKSPACE_DIR}
    COMMENT "Building rasman-rust (${RASMAN_RUST_TARGET})"
)
add_custom_target(rust_rasman DEPENDS ${RASMAN_RUST_LIB})

# The only compiled src *.c → rasman-rust (src/rasman/rasman.rs), kept on disk per the
# never-delete rule.
#   rasman.c  → rasman-rust
# list(APPEND SOURCE
#     rasman.c)
list(APPEND SOURCE
    rasman.rc
    ${CMAKE_CURRENT_BINARY_DIR}/rasman_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/rasman.def)

add_library(rasman MODULE ${SOURCE})
set_module_type(rasman win32dll)
# librasman_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(rasman ${RASMAN_RUST_LIB} wine uuid)
add_dependencies(rasman rust_rasman)
add_importlibs(rasman msvcrt kernel32 ntdll)
add_cd_file(TARGET rasman DESTINATION reactos/system32 FOR all)
