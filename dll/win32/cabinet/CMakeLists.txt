
add_definitions(
    -DHAVE_ZLIB)

include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/zlib)

spec2def(cabinet.dll cabinet.spec ADD_IMPORTLIB)

# ============================================================================
# cabinet.dll — Rust cohabitation flip (cabinet_main/fci/fdi/stubs.c -> Rust)
# ============================================================================
# All four cabinet C sources are translated to Rust (cabinet-rust, kept on disk
# per the never-delete rule). Every cabinet.spec export comes from
# libcabinet_rust.a; cmake keeps the win32dll MODULE link (spec2def .def +
# zlib/wine for the deflate/inflate calls the Rust still makes + importlibs).
# The LZX/QTM/MSZIP decompression engine + FDICopy are hand-ported in
# cabinet-rust/src/cabinet/fdi.rs. Mirrors the wmilib-rust static-lib cohab.
set(CABINET_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(CABINET_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-cabinet-target")
set(CABINET_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcabinet_rust.a")
if(ARCH STREQUAL "i386")
    set(CABINET_RUST_TARGET "i686-pc-windows-gnu")
    set(CABINET_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(CABINET_RUST_TARGET "x86_64-pc-windows-gnu")
    set(CABINET_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(CABINET_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(CABINET_RUST_RUSTFLAGS "")
endif()
add_custom_command(
    OUTPUT ${CABINET_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${CABINET_RUST_TARGET_DIR}
        RUSTFLAGS=${CABINET_RUST_RUSTFLAGS}
        cargo build --release --target ${CABINET_RUST_TARGET}
            -p cabinet-rust
            --manifest-path ${CABINET_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CABINET_RUST_TARGET_DIR}/${CABINET_RUST_TARGET}/release/libcabinet_rust.a
        ${CABINET_RUST_LIB}
    DEPENDS ${CABINET_RUST_WORKSPACE_DIR}/cabinet-rust/Cargo.toml
            ${CABINET_RUST_WORKSPACE_DIR}/cabinet-rust/src/lib.rs
            ${CABINET_RUST_WORKSPACE_DIR}/cabinet-rust/src/cabinet/cabinet_main.rs
            ${CABINET_RUST_WORKSPACE_DIR}/cabinet-rust/src/cabinet/fci.rs
            ${CABINET_RUST_WORKSPACE_DIR}/cabinet-rust/src/cabinet/fdi.rs
            ${CABINET_RUST_WORKSPACE_DIR}/cabinet-rust/src/cabinet/stubs.rs
            ${CABINET_RUST_WORKSPACE_DIR}/cabinet-rust/src/cabinet/types.rs
            ${CABINET_RUST_WORKSPACE_DIR}/cabinet-rust/src/cabinet/wine_list.rs
    WORKING_DIRECTORY ${CABINET_RUST_WORKSPACE_DIR}
    COMMENT "Building cabinet-rust (${CABINET_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_cabinet DEPENDS ${CABINET_RUST_LIB})

list(APPEND SOURCE
    rust_cohab_stub.c)
#   cabinet_main.c fci.c fdi.c stubs.c  # replaced by Rust (cabinet-rust); kept on disk

add_library(cabinet MODULE
    ${SOURCE}
    cabinet.rc
    ${CMAKE_CURRENT_BINARY_DIR}/cabinet.def)

set_module_type(cabinet win32dll)
target_link_libraries(cabinet wine zlib)
add_importlibs(cabinet msvcrt kernel32 ntdll)
add_dependencies(cabinet rust_cabinet)
target_link_libraries(cabinet ${CABINET_RUST_LIB})
add_cd_file(TARGET cabinet DESTINATION reactos/system32 FOR all)
set_wine_module(cabinet)
