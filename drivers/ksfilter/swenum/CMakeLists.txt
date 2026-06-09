
# ============================================================================
# swenum.sys — Rust cohabitation flip (swenum.c -> sdk/lib/rust/swenum-rust)
# ============================================================================
# swenum.c (the KS software bus enumerator) is translated to Rust (swenum-rust,
# kept on disk per the never-delete rule). DriverEntry + the Sw* dispatch
# routines come from libswenum_rust.a; the Ks* bus-enum entry points stay
# ks.sys imports. cmake keeps the MODULE link (set_module_type kernelmodedriver
# + add_importlibs(ks ntoskrnl hal)). swenum-rust depends on ntkrnl-sys, so the
# custom_command mirrors the cmlib/cport cohab (standard per-arch RUST_TARGET,
# i386 no-SSE RUSTFLAGS).
set(SWENUM_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(SWENUM_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-swenum-target")
set(SWENUM_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libswenum_rust.a")
if(ARCH STREQUAL "i386")
    set(SWENUM_RUST_TARGET "i686-pc-windows-gnu")
    set(SWENUM_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(SWENUM_RUST_TARGET "x86_64-pc-windows-gnu")
    set(SWENUM_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(SWENUM_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(SWENUM_RUST_RUSTFLAGS "")
endif()
add_custom_command(
    OUTPUT ${SWENUM_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SWENUM_RUST_TARGET_DIR}
        RUSTFLAGS=${SWENUM_RUST_RUSTFLAGS}
        cargo build --release --target ${SWENUM_RUST_TARGET}
            -p swenum-rust
            --manifest-path ${SWENUM_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${SWENUM_RUST_TARGET_DIR}/${SWENUM_RUST_TARGET}/release/libswenum_rust.a
        ${SWENUM_RUST_LIB}
    DEPENDS ${SWENUM_RUST_WORKSPACE_DIR}/swenum-rust/Cargo.toml
            ${SWENUM_RUST_WORKSPACE_DIR}/swenum-rust/src/lib.rs
            ${SWENUM_RUST_WORKSPACE_DIR}/swenum-rust/src/swenum.rs
    WORKING_DIRECTORY ${SWENUM_RUST_WORKSPACE_DIR}
    COMMENT "Building swenum-rust (${SWENUM_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_swenum DEPENDS ${SWENUM_RUST_LIB})

# add_library(swenum MODULE swenum.c swenum.rc)  # body replaced by Rust; kept on disk
add_library(swenum MODULE rust_cohab_stub.c swenum.rc)
set_module_type(swenum kernelmodedriver)
add_importlibs(swenum ks ntoskrnl hal)
add_dependencies(swenum rust_swenum)
target_link_libraries(swenum ${SWENUM_RUST_LIB})
add_cd_file(TARGET swenum DESTINATION reactos/system32/drivers NO_CAB FOR all)
