
spec2def(rasdlg.dll rasdlg.spec)

# rasdlg.dll — Rust cohabitation flip (FULL: the one compiled src *.c → rasdlg-rust).
# The RAS dial-up dialog UI; Wine-sourced stubs, 8 of 9 functions lowered
# deterministically for $0. rasdlg.c is commented out below and kept on disk per the
# never-delete rule. rasdlg.rc stays (resources are not translated).
set(RASDLG_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(RASDLG_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(RASDLG_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(RASDLG_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(RASDLG_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-rasdlg-target" rasdlg-rust)
set(RASDLG_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/librasdlg_rust.a")
rust_crate_sources(RASDLG_RUST_SRCS "${RASDLG_RUST_WORKSPACE_DIR}" rasdlg-rust)
add_custom_command(
    OUTPUT ${RASDLG_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RASDLG_RUST_TARGET_DIR}
        cargo build --release --target ${RASDLG_RUST_TARGET}
            -p rasdlg-rust
            --manifest-path ${RASDLG_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${RASDLG_RUST_TARGET_DIR}/${RASDLG_RUST_TARGET}/release/librasdlg_rust.a
        ${RASDLG_RUST_LIB}
    DEPENDS ${RASDLG_RUST_SRCS}
            ${RASDLG_RUST_WORKSPACE_DIR}/rasdlg-rust/Cargo.toml
    WORKING_DIRECTORY ${RASDLG_RUST_WORKSPACE_DIR}
    COMMENT "Building rasdlg-rust (${RASDLG_RUST_TARGET})"
)
add_custom_target(rust_rasdlg DEPENDS ${RASDLG_RUST_LIB})

# The only compiled src *.c → rasdlg-rust (src/rasdlg/rasdlg.rs), kept on disk per the
# never-delete rule.
#   rasdlg.c  → rasdlg-rust
# list(APPEND SOURCE
#     rasdlg.c)
list(APPEND SOURCE
    rasdlg.rc
    ${CMAKE_CURRENT_BINARY_DIR}/rasdlg_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/rasdlg.def)

add_library(rasdlg MODULE ${SOURCE})
set_module_type(rasdlg win32dll)
# librasdlg_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(rasdlg ${RASDLG_RUST_LIB} wine uuid)
add_dependencies(rasdlg rust_rasdlg)
add_importlibs(rasdlg msvcrt kernel32 ntdll)
add_cd_file(TARGET rasdlg DESTINATION reactos/system32 FOR all)
