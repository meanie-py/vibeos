
add_definitions(-D_COMDDK_)

# All five .c bodies are translated to Rust (sdk/lib/rust/sysaudio-rust: pin.rs from the 2026
# pilot, sysaudio/{control,deviface,dispatcher,main}.rs from the A' batch) and stay on disk,
# uncompiled. The MODULE link needs one C source, the placeholder stub.
#   control.c deviface.c dispatcher.c main.c pin.c
list(APPEND SOURCE rust_cohab_stub.c)
set(SYSAUDIO_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(SYSAUDIO_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-sysaudio-target" sysaudio-rust)
set(SYSAUDIO_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libsysaudio_rust.a")
if(ARCH STREQUAL "i386")
    set(SYSAUDIO_RUST_TARGET "i686-pc-windows-gnu")
    set(SYSAUDIO_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(SYSAUDIO_RUST_TARGET "x86_64-pc-windows-gnu")
    set(SYSAUDIO_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(SYSAUDIO_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(SYSAUDIO_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(SYSAUDIO_RUST_SRCS "${SYSAUDIO_RUST_WORKSPACE_DIR}" sysaudio-rust)
add_custom_command(
    OUTPUT ${SYSAUDIO_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SYSAUDIO_RUST_TARGET_DIR}
        RUSTFLAGS=${SYSAUDIO_RUST_RUSTFLAGS}
        cargo build --release --target ${SYSAUDIO_RUST_TARGET}
            -p sysaudio-rust
            --manifest-path ${SYSAUDIO_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${SYSAUDIO_RUST_TARGET_DIR}/${SYSAUDIO_RUST_TARGET}/release/libsysaudio_rust.a
        ${SYSAUDIO_RUST_LIB}
    DEPENDS ${SYSAUDIO_RUST_SRCS} ${SYSAUDIO_RUST_WORKSPACE_DIR}/sysaudio-rust/Cargo.toml
    WORKING_DIRECTORY ${SYSAUDIO_RUST_WORKSPACE_DIR}
    COMMENT "Building sysaudio-rust (${SYSAUDIO_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_sysaudio DEPENDS ${SYSAUDIO_RUST_LIB})


add_library(sysaudio MODULE ${SOURCE} sysaudio.rc)
set_module_type(sysaudio kernelmodedriver)
add_dependencies(sysaudio rust_sysaudio)
target_link_libraries(sysaudio ${SYSAUDIO_RUST_LIB} libcntpr)
add_importlibs(sysaudio ntoskrnl ks hal)
add_cd_file(TARGET sysaudio DESTINATION reactos/system32/drivers FOR all)
