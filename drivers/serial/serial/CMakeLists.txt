
# circularbuffer/cleanup/close/create/devctrl/info/legacy/misc/pnp/power/rw/serial.c
# replaced by Rust (serial-rust); guid.c stays C (GUID data TU). NOT boot-loaded in smoke
# (kernel owns the KD COM port) -> link-clean x3 bar.

set(SERIAL_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(SERIAL_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-serial-target")
set(SERIAL_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libserial_rust.a")
if(ARCH STREQUAL "i386")
    set(SERIAL_RUST_TARGET "i686-pc-windows-gnu")
    set(SERIAL_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(SERIAL_RUST_TARGET "x86_64-pc-windows-gnu")
    set(SERIAL_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(SERIAL_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(SERIAL_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(SERIAL_RUST_SRCS "${SERIAL_RUST_WORKSPACE_DIR}" serial-rust)
add_custom_command(
    OUTPUT ${SERIAL_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SERIAL_RUST_TARGET_DIR} RUSTFLAGS=${SERIAL_RUST_RUSTFLAGS}
        cargo build --release --target ${SERIAL_RUST_TARGET} -p serial-rust --manifest-path ${SERIAL_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${SERIAL_RUST_TARGET_DIR}/${SERIAL_RUST_TARGET}/release/libserial_rust.a ${SERIAL_RUST_LIB}
    DEPENDS ${SERIAL_RUST_SRCS} ${SERIAL_RUST_WORKSPACE_DIR}/serial-rust/Cargo.toml
    WORKING_DIRECTORY ${SERIAL_RUST_WORKSPACE_DIR}
    COMMENT "Building serial-rust (${SERIAL_RUST_TARGET})" VERBATIM)
add_custom_target(rust_serial DEPENDS ${SERIAL_RUST_LIB})

add_library(serial MODULE guid.c serial.rc)
set_module_type(serial kernelmodedriver)
add_importlibs(serial ntoskrnl hal)
add_dependencies(serial rust_serial)
target_link_libraries(serial ${SERIAL_RUST_LIB})
add_cd_file(TARGET serial DESTINATION reactos/system32/drivers FOR all)
