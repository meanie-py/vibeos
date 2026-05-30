# ============================================================================
# Rust framebuffer GDI display driver
# (sdk/lib/rust/framebuf-rust -> libframebuf_rust.a)
# ============================================================================
# enable.c, palette.c, pointer.c, screen.c, surface.c are translated to Rust.
# Their #[no_mangle] DrvEnableDriver (and all DDI/Int* functions) replace the
# C sources (commented out of the SOURCE list below; the .c files are kept on
# disk per the never-delete-a-translation rule). Mirrors the arm64vid-rust
# cohabitation (drivers/video/arm64vid/CMakeLists.txt): one staticlib linked
# into the cmake-built kerneldll.
#
# framebuf is the display driver for every UEFI GOP/linear-framebuffer boot on
# all three arches, so the Rust target is selected per-arch (like
# hal/halx86/CMakeLists.txt). The Rust crate links win32k (the GDI engine
# exports Eng*/PALOBJ_*/DDI types via win32k-sys) rather than ntoskrnl.

set(FRAMEBUF_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(FRAMEBUF_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-framebuf-target")
if(ARCH STREQUAL "i386")
    set(FRAMEBUF_RUST_TARGET "i686-pc-windows-gnu")
    # i386: disable SSE — same constraint as kernel-rust / hal-rust.
    set(FRAMEBUF_RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(FRAMEBUF_RUST_TARGET "x86_64-pc-windows-gnu")
    set(FRAMEBUF_RUST_EXTRA_ENV "")
else()
    set(FRAMEBUF_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(FRAMEBUF_RUST_EXTRA_ENV "")
endif()
set(FRAMEBUF_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libframebuf_rust.a")

add_custom_command(
    OUTPUT ${FRAMEBUF_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${FRAMEBUF_RUST_TARGET_DIR} ${FRAMEBUF_RUST_EXTRA_ENV}
        cargo build --release --target ${FRAMEBUF_RUST_TARGET}
            -p framebuf-rust
            --manifest-path ${FRAMEBUF_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${FRAMEBUF_RUST_TARGET_DIR}/${FRAMEBUF_RUST_TARGET}/release/libframebuf_rust.a
        ${FRAMEBUF_RUST_LIB}
    DEPENDS ${FRAMEBUF_RUST_WORKSPACE_DIR}/framebuf-rust/Cargo.toml
            ${FRAMEBUF_RUST_WORKSPACE_DIR}/framebuf-rust/src/lib.rs
            ${FRAMEBUF_RUST_WORKSPACE_DIR}/framebuf-rust/src/framebuf.rs
            ${FRAMEBUF_RUST_WORKSPACE_DIR}/framebuf-rust/src/enable.rs
            ${FRAMEBUF_RUST_WORKSPACE_DIR}/framebuf-rust/src/palette.rs
            ${FRAMEBUF_RUST_WORKSPACE_DIR}/framebuf-rust/src/pointer.rs
            ${FRAMEBUF_RUST_WORKSPACE_DIR}/framebuf-rust/src/screen.rs
            ${FRAMEBUF_RUST_WORKSPACE_DIR}/framebuf-rust/src/surface.rs
    WORKING_DIRECTORY ${FRAMEBUF_RUST_WORKSPACE_DIR}
    COMMENT "Building framebuf-rust (${FRAMEBUF_RUST_TARGET})"
    VERBATIM
)
add_custom_target(rust_framebuf DEPENDS ${FRAMEBUF_RUST_LIB})

# enable.c palette.c pointer.c screen.c surface.c -- replaced by Rust
# (sdk/lib/rust/framebuf-rust/src/*.rs). framebuf.h kept for reference.
add_library(framebuf MODULE
    rust_cohab_stub.c
    framebuf.rc)

set_module_type(framebuf kerneldll ENTRYPOINT DrvEnableDriver 12)
target_link_libraries(framebuf ${FRAMEBUF_RUST_LIB} libcntpr)
add_importlibs(framebuf win32k)
add_dependencies(framebuf psdk rust_framebuf)
add_cd_file(TARGET framebuf DESTINATION reactos/system32 FOR all)
