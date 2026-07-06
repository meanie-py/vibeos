# ============================================================================
# csq (Cancel-Safe IRP Queue) — Rust cohabitation flip (csq.c -> csq-rust)
# ============================================================================
# csq.c (the 6 IoCsq* exports) is translated to Rust (csq-rust, kept on disk per
# the never-delete rule). libcsq_rust.a replaces the C object; the `csq` static
# lib target is preserved (ntoskrnl links it by name) as a stub TU that PUBLIC-
# links the Rust archive (usbdex-style cohab). csq-rust depends on ntkrnl-sys
# (the IO_CSQ types live in the ddk bindgen). Built for all 3 arches.
add_definitions(-D_NTOSKRNL_)

set(CSQ_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(CSQ_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-csq-target")
set(CSQ_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcsq_rust.a")
if(ARCH STREQUAL "i386")
    set(CSQ_RUST_TARGET "i686-pc-windows-gnu")
    set(CSQ_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(CSQ_RUST_TARGET "x86_64-pc-windows-gnu")
    set(CSQ_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(CSQ_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(CSQ_RUST_RUSTFLAGS "")
endif()
file(GLOB_RECURSE CSQ_RUST_SRCS ${CSQ_RUST_WORKSPACE_DIR}/csq-rust/src/*.rs)
add_custom_command(
    OUTPUT ${CSQ_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${CSQ_RUST_TARGET_DIR}
        RUSTFLAGS=${CSQ_RUST_RUSTFLAGS}
        cargo build --release --target ${CSQ_RUST_TARGET}
            -p csq-rust
            --manifest-path ${CSQ_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CSQ_RUST_TARGET_DIR}/${CSQ_RUST_TARGET}/release/libcsq_rust.a
        ${CSQ_RUST_LIB}
    DEPENDS ${CSQ_RUST_SRCS}
            ${CSQ_RUST_WORKSPACE_DIR}/csq-rust/Cargo.toml
    WORKING_DIRECTORY ${CSQ_RUST_WORKSPACE_DIR}
    COMMENT "Building csq-rust (${CSQ_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_csq DEPENDS ${CSQ_RUST_LIB})

# add_library(csq csq.c)  # body replaced by Rust; kept on disk
add_library(csq rust_cohab_stub.c)
add_dependencies(csq bugcodes xdk rust_csq)
target_link_libraries(csq PUBLIC ${CSQ_RUST_LIB})
