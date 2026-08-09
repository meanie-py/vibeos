
spec2def(wmilib.sys wmilib.spec ADD_IMPORTLIB)

# ============================================================================
# wmilib.sys — Rust cohabitation flip (wmilib.c -> sdk/lib/rust/wmilib-rust)
# ============================================================================
# wmilib.c (WmiCompleteRequest / WmiFireEvent / WmiSystemControl) is translated
# to Rust (wmilib-rust, kept on disk per the never-delete rule). The 3
# wmilib.spec exports come from libwmilib_rust.a; cmake keeps the MODULE link
# (set_module_type ... ENTRYPOINT 0 + spec2def .def + add_importlibs(ntoskrnl)).
# Mirrors cportlib's static-lib custom_command + bootvid's export-DLL cohab.
set(WMILIB_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(WMILIB_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-wmilib-target")
set(WMILIB_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libwmilib_rust.a")
if(ARCH STREQUAL "i386")
    set(WMILIB_RUST_TARGET "i686-pc-windows-gnu")
    set(WMILIB_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(WMILIB_RUST_TARGET "x86_64-pc-windows-gnu")
    set(WMILIB_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(WMILIB_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(WMILIB_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(WMILIB_RUST_SRCS "${WMILIB_RUST_WORKSPACE_DIR}" wmilib-rust)
add_custom_command(
    OUTPUT ${WMILIB_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${WMILIB_RUST_TARGET_DIR}
        RUSTFLAGS=${WMILIB_RUST_RUSTFLAGS}
        cargo build --release --target ${WMILIB_RUST_TARGET}
            -p wmilib-rust
            --manifest-path ${WMILIB_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${WMILIB_RUST_TARGET_DIR}/${WMILIB_RUST_TARGET}/release/libwmilib_rust.a
        ${WMILIB_RUST_LIB}
    DEPENDS ${WMILIB_RUST_SRCS}
            ${WMILIB_RUST_WORKSPACE_DIR}/wmilib-rust/Cargo.toml
    WORKING_DIRECTORY ${WMILIB_RUST_WORKSPACE_DIR}
    COMMENT "Building wmilib-rust (${WMILIB_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_wmilib DEPENDS ${WMILIB_RUST_LIB})

list(APPEND SOURCE
    rust_cohab_stub.c
    wmilib.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wmilib.def)
#   wmilib.c  # replaced by Rust (wmilib-rust); kept on disk

add_library(wmilib MODULE ${SOURCE})
set_module_type(wmilib kernelmodedriver ENTRYPOINT 0)
add_importlibs(wmilib ntoskrnl)
add_dependencies(wmilib rust_wmilib)
target_link_libraries(wmilib ${WMILIB_RUST_LIB})
add_cd_file(TARGET wmilib DESTINATION reactos/system32/drivers NO_CAB FOR all)
