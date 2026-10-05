

remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

# ============================================================================
# usbaudio.sys — Rust cohabitation flip (usbaudio/pool/filter/pin.c -> usbaudio-rust)
# ============================================================================
# The USB Audio class driver (4 .c bodies) is translated to Rust (sdk/lib/rust/usbaudio-rust,
# libusbaudio_rust.a; DeepSeek-chat V3 + c2rust-anchored, constant-safeguard-verified clean).
# The .c files are kept on disk per the never-delete rule; guid.c stays compiled as a data-only
# TU (DEFINE_GUID). The KS/PortCls dispatch tables + #[no_mangle] DriverEntry/AddDevice in the
# rlib take over the link. usbaudio is a KS/PortCls miniport (imports usbd + ks). It was
# previously EXCLUDED from the build (drivers/usb/CMakeLists.txt) and is re-enabled by this flip;
# NOT boot-loaded in smoke -> link-clean x3 is the verification bar. NTDDI Vista (0x600) preserved.
set(USBAUDIO_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(USBAUDIO_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-usbaudio-target" usbaudio-rust)
set(USBAUDIO_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libusbaudio_rust.a")
if(ARCH STREQUAL "i386")
    set(USBAUDIO_RUST_TARGET "i686-pc-windows-gnu")
    set(USBAUDIO_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(USBAUDIO_RUST_TARGET "x86_64-pc-windows-gnu")
    set(USBAUDIO_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(USBAUDIO_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(USBAUDIO_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(USBAUDIO_RUST_SRCS "${USBAUDIO_RUST_WORKSPACE_DIR}" usbaudio-rust)
add_custom_command(
    OUTPUT ${USBAUDIO_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USBAUDIO_RUST_TARGET_DIR}
        RUSTFLAGS=${USBAUDIO_RUST_RUSTFLAGS}
        cargo build --release --target ${USBAUDIO_RUST_TARGET}
            -p usbaudio-rust
            --manifest-path ${USBAUDIO_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${USBAUDIO_RUST_TARGET_DIR}/${USBAUDIO_RUST_TARGET}/release/libusbaudio_rust.a
        ${USBAUDIO_RUST_LIB}
    DEPENDS ${USBAUDIO_RUST_SRCS} ${USBAUDIO_RUST_WORKSPACE_DIR}/usbaudio-rust/Cargo.toml
    WORKING_DIRECTORY ${USBAUDIO_RUST_WORKSPACE_DIR}
    COMMENT "Building usbaudio-rust (${USBAUDIO_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_usbaudio DEPENDS ${USBAUDIO_RUST_LIB})

# C bodies replaced by Rust (usbaudio/pool/filter/pin.c) — kept on disk per the never-delete rule.
# guid.c stays as the data-only TU (DEFINE_GUID).
add_library(usbaudio MODULE
    rust_cohab_stub.c
    guid.c
    usbaudio.rc)

target_link_libraries(usbaudio
    libcntpr
    ${PSEH_LIB})

set_module_type(usbaudio kernelmodedriver)
add_importlibs(usbaudio ntoskrnl hal usbd ks)
add_dependencies(usbaudio rust_usbaudio)
target_link_libraries(usbaudio ${USBAUDIO_RUST_LIB})

add_cd_file(TARGET usbaudio DESTINATION reactos/system32/drivers NO_CAB FOR all)
