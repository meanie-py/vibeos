
spec2def(hidparse.sys hidparse.spec ADD_IMPORTLIB)
add_definitions(-DDEBUG_MODE)

include_directories(
    ${REACTOS_SOURCE_DIR}/ntoskrnl/include
    ${REACTOS_SOURCE_DIR}/sdk/lib/drivers/hidparser)

# ============================================================================
# hidparse.sys — Rust cohabitation flip (hidparse.c -> sdk/lib/rust/hidparse-rust)
# ============================================================================
# hidparse.c (DriverEntry + the 5 hidparser_km callbacks) is translated to Rust
# (hidparse-rust, kept on disk per the never-delete rule). DriverEntry + the
# callbacks come from libhidparse_rust.a; the hidparser_km C library stays C
# (Batch 1 — it links against the Rust callbacks). cmake keeps the MODULE link
# (set_module_type kernelmodedriver + spec2def .def + add_importlibs(ntoskrnl)).
set(HIDPARSE_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(HIDPARSE_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-hidparse-target")
set(HIDPARSE_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libhidparse_rust.a")
if(ARCH STREQUAL "i386")
    set(HIDPARSE_RUST_TARGET "i686-pc-windows-gnu")
    set(HIDPARSE_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(HIDPARSE_RUST_TARGET "x86_64-pc-windows-gnu")
    set(HIDPARSE_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(HIDPARSE_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(HIDPARSE_RUST_RUSTFLAGS "")
endif()
add_custom_command(
    OUTPUT ${HIDPARSE_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${HIDPARSE_RUST_TARGET_DIR}
        RUSTFLAGS=${HIDPARSE_RUST_RUSTFLAGS}
        cargo build --release --target ${HIDPARSE_RUST_TARGET}
            -p hidparse-rust
            --manifest-path ${HIDPARSE_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${HIDPARSE_RUST_TARGET_DIR}/${HIDPARSE_RUST_TARGET}/release/libhidparse_rust.a
        ${HIDPARSE_RUST_LIB}
    DEPENDS ${HIDPARSE_RUST_WORKSPACE_DIR}/hidparse-rust/Cargo.toml
            ${HIDPARSE_RUST_WORKSPACE_DIR}/hidparse-rust/src/lib.rs
    WORKING_DIRECTORY ${HIDPARSE_RUST_WORKSPACE_DIR}
    COMMENT "Building hidparse-rust (${HIDPARSE_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_hidparse DEPENDS ${HIDPARSE_RUST_LIB})

# add_library(hidparse MODULE hidparse.c ...)  # body replaced by Rust; kept on disk
add_library(hidparse MODULE rust_cohab_stub.c hidparse.rc ${CMAKE_CURRENT_BINARY_DIR}/hidparse.def)
set_module_type(hidparse kernelmodedriver)
add_importlibs(hidparse ntoskrnl)
add_dependencies(hidparse rust_hidparse)
target_link_libraries(hidparse hidparser_km ${HIDPARSE_RUST_LIB})
add_cd_file(TARGET hidparse DESTINATION reactos/system32/drivers NO_CAB FOR all)
