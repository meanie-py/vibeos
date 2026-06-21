add_definitions(-DDEBUG)

# storahci.c replaced by Rust (storahci-rust). AHCI storport miniport; links storport.
# Stays DISABLED from shipping (re-enable when AHCI is wired) -> built + link-clean x3 only.

set(STORAHCI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(STORAHCI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-storahci-target")
set(STORAHCI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libstorahci_rust.a")
if(ARCH STREQUAL "i386")
    set(STORAHCI_RUST_TARGET "i686-pc-windows-gnu")
    set(STORAHCI_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(STORAHCI_RUST_TARGET "x86_64-pc-windows-gnu")
    set(STORAHCI_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(STORAHCI_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(STORAHCI_RUST_RUSTFLAGS "")
endif()
file(GLOB_RECURSE STORAHCI_RUST_SRCS ${STORAHCI_RUST_WORKSPACE_DIR}/storahci-rust/src/*.rs)
add_custom_command(
    OUTPUT ${STORAHCI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${STORAHCI_RUST_TARGET_DIR} RUSTFLAGS=${STORAHCI_RUST_RUSTFLAGS} RUSTC_BOOTSTRAP=1
        cargo build --release --target ${STORAHCI_RUST_TARGET} -p storahci-rust --manifest-path ${STORAHCI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${STORAHCI_RUST_TARGET_DIR}/${STORAHCI_RUST_TARGET}/release/libstorahci_rust.a ${STORAHCI_RUST_LIB}
    DEPENDS ${STORAHCI_RUST_SRCS} ${STORAHCI_RUST_WORKSPACE_DIR}/storahci-rust/Cargo.toml
    WORKING_DIRECTORY ${STORAHCI_RUST_WORKSPACE_DIR}
    COMMENT "Building storahci-rust (${STORAHCI_RUST_TARGET})" VERBATIM)
add_custom_target(rust_storahci DEPENDS ${STORAHCI_RUST_LIB})

add_library(storahci MODULE rust_cohab_stub.c storahci.rc)
set_module_type(storahci kernelmodedriver)
add_importlibs(storahci storport ntoskrnl hal)
add_dependencies(storahci rust_storahci)
target_link_libraries(storahci ${STORAHCI_RUST_LIB})
#add_cd_file(TARGET storahci DESTINATION reactos/system32/drivers NO_CAB FOR all)
#add_driver_inf(storahci storahci.inf)
