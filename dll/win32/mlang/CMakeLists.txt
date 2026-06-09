
add_definitions(
    -DCOM_NO_WINDOWS_H)

spec2def(mlang.dll mlang.spec ADD_IMPORTLIB)

# ============================================================================
# mlang.dll — Rust cohabitation flip (mlang.c -> sdk/lib/rust/mlang-rust)
# ============================================================================
# mlang.c is translated to Rust (mlang-rust, kept on disk per the never-delete
# rule). The mlang.spec exports + DllMain / COM entry points come from
# libmlang_rust.a; cmake keeps the win32dll MODULE link (spec2def .def +
# mlang_stubs.c for the `stub`-only spec entries + importlibs). Mirrors the
# wmilib-rust static-lib custom_command cohab.
set(MLANG_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(MLANG_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-mlang-target")
set(MLANG_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libmlang_rust.a")
if(ARCH STREQUAL "i386")
    set(MLANG_RUST_TARGET "i686-pc-windows-gnu")
    set(MLANG_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(MLANG_RUST_TARGET "x86_64-pc-windows-gnu")
    set(MLANG_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(MLANG_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(MLANG_RUST_RUSTFLAGS "")
endif()
add_custom_command(
    OUTPUT ${MLANG_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${MLANG_RUST_TARGET_DIR}
        RUSTFLAGS=${MLANG_RUST_RUSTFLAGS}
        cargo build --release --target ${MLANG_RUST_TARGET}
            -p mlang-rust
            --manifest-path ${MLANG_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${MLANG_RUST_TARGET_DIR}/${MLANG_RUST_TARGET}/release/libmlang_rust.a
        ${MLANG_RUST_LIB}
    DEPENDS ${MLANG_RUST_WORKSPACE_DIR}/mlang-rust/Cargo.toml
            ${MLANG_RUST_WORKSPACE_DIR}/mlang-rust/src/lib.rs
            ${MLANG_RUST_WORKSPACE_DIR}/mlang-rust/src/mlang/mlang.rs
    WORKING_DIRECTORY ${MLANG_RUST_WORKSPACE_DIR}
    COMMENT "Building mlang-rust (${MLANG_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_mlang DEPENDS ${MLANG_RUST_LIB})

list(APPEND SOURCE
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/mlang_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/mlang.def)
#   mlang.c  # replaced by Rust (mlang-rust); kept on disk

add_library(mlang MODULE ${SOURCE} mlang.rc)
set_module_type(mlang win32dll UNICODE)
target_link_libraries(mlang uuid wine oldnames)
add_delay_importlibs(mlang oleaut32)
add_importlibs(mlang gdi32 advapi32 msvcrt kernel32 ntdll)
add_dependencies(mlang rust_mlang)
target_link_libraries(mlang ${MLANG_RUST_LIB})
add_cd_file(TARGET mlang DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(mlang) # CORE-5743: No ARRAY_SIZE and CONST_VTABLE
