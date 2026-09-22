
spec2def(ksecdd.sys ksecdd.spec)

include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers/ksecdd)

# The 5 .c bodies replaced by Rust (ksecdd-rust). Kernel Security Support Provider.
# .c kept on disk (never-delete). Bar: link-clean x3.
set(KSECDD_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(KSECDD_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-ksecdd-target" ksecdd-rust)
set(KSECDD_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libksecdd_rust.a")
if(ARCH STREQUAL "i386")
    set(KSECDD_RUST_TARGET "i686-pc-windows-gnu")
    set(KSECDD_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(KSECDD_RUST_TARGET "x86_64-pc-windows-gnu")
    set(KSECDD_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(KSECDD_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(KSECDD_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(KSECDD_RUST_SRCS "${KSECDD_RUST_WORKSPACE_DIR}" ksecdd-rust)
add_custom_command(
    OUTPUT ${KSECDD_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${KSECDD_RUST_TARGET_DIR} RUSTFLAGS=${KSECDD_RUST_RUSTFLAGS}
        cargo build --release --target ${KSECDD_RUST_TARGET} -p ksecdd-rust --manifest-path ${KSECDD_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${KSECDD_RUST_TARGET_DIR}/${KSECDD_RUST_TARGET}/release/libksecdd_rust.a ${KSECDD_RUST_LIB}
    DEPENDS ${KSECDD_RUST_SRCS} ${KSECDD_RUST_WORKSPACE_DIR}/ksecdd-rust/Cargo.toml
    WORKING_DIRECTORY ${KSECDD_RUST_WORKSPACE_DIR}
    COMMENT "Building ksecdd-rust (${KSECDD_RUST_TARGET})" VERBATIM)
add_custom_target(rust_ksecdd DEPENDS ${KSECDD_RUST_LIB})

add_library(ksecdd MODULE
    rust_cohab_stub.c
    ksecdd.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ksecdd.def)

set_module_type(ksecdd kernelmodedriver)
add_dependencies(ksecdd rust_ksecdd)
target_link_libraries(ksecdd ${KSECDD_RUST_LIB} cryptlib)
add_importlibs(ksecdd ntoskrnl hal)
add_cd_file(TARGET ksecdd DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_registry_inf(ksecdd_reg.inf)
