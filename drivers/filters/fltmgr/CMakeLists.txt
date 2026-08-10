
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers/fltmgr
    includes)

# fltmgr export lib + .def (the Flt* API minifilters import) — kept; Rust provides the symbols.
spec2def(fltmgr.sys fltmgr.spec ADD_IMPORTLIB)

# The 10 .c bodies replaced by Rust (fltmgr-rust). Filesystem Filter Manager.
# .c kept on disk (never-delete). Bar: link-clean x3.
set(FLTMGR_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(FLTMGR_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-fltmgr-target")
set(FLTMGR_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libfltmgr_rust.a")
if(ARCH STREQUAL "i386")
    set(FLTMGR_RUST_TARGET "i686-pc-windows-gnu")
    set(FLTMGR_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(FLTMGR_RUST_TARGET "x86_64-pc-windows-gnu")
    set(FLTMGR_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(FLTMGR_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(FLTMGR_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(FLTMGR_RUST_SRCS "${FLTMGR_RUST_WORKSPACE_DIR}" fltmgr-rust)
add_custom_command(
    OUTPUT ${FLTMGR_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${FLTMGR_RUST_TARGET_DIR} RUSTFLAGS=${FLTMGR_RUST_RUSTFLAGS}
        cargo build --release --target ${FLTMGR_RUST_TARGET} -p fltmgr-rust --manifest-path ${FLTMGR_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${FLTMGR_RUST_TARGET_DIR}/${FLTMGR_RUST_TARGET}/release/libfltmgr_rust.a ${FLTMGR_RUST_LIB}
    DEPENDS ${FLTMGR_RUST_SRCS} ${FLTMGR_RUST_WORKSPACE_DIR}/fltmgr-rust/Cargo.toml
    WORKING_DIRECTORY ${FLTMGR_RUST_WORKSPACE_DIR}
    COMMENT "Building fltmgr-rust (${FLTMGR_RUST_TARGET})" VERBATIM)
add_custom_target(rust_fltmgr DEPENDS ${FLTMGR_RUST_LIB})

add_library(fltmgr MODULE
    rust_cohab_stub.c
    fltmgr.rc
    ${CMAKE_CURRENT_BINARY_DIR}/fltmgr.def)

set_module_type(fltmgr kernelmodedriver)
add_dependencies(fltmgr rust_fltmgr)
target_link_libraries(fltmgr ${FLTMGR_RUST_LIB})
add_importlibs(fltmgr ntoskrnl hal)
add_cd_file(TARGET fltmgr DESTINATION reactos/system32/drivers NO_CAB FOR all)
