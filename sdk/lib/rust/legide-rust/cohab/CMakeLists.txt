
# ============================================================================
# legide.sys — Rust cohabitation flip (legide.c -> legide-rust), i386 only
# ============================================================================
# Upstream's legacy (ISA-port) PATA bus enumerator, a pciidex client like pciide. Translated
# to Rust in sdk/lib/rust/legide-rust (rustconv port, sealed DeepSeek); legide.c stays on disk.
set(LEGIDE_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(LEGIDE_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-legide-target" legide-rust)
set(LEGIDE_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/liblegide_rust.a")
set(LEGIDE_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
rust_crate_sources(LEGIDE_RUST_SRCS "${LEGIDE_RUST_WORKSPACE_DIR}" legide-rust)
add_custom_command(
    OUTPUT ${LEGIDE_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${LEGIDE_RUST_TARGET_DIR}
        RUSTFLAGS=${LEGIDE_RUST_RUSTFLAGS}
        cargo build --release --target i686-pc-windows-gnu
            -p legide-rust
            --manifest-path ${LEGIDE_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${LEGIDE_RUST_TARGET_DIR}/i686-pc-windows-gnu/release/liblegide_rust.a
        ${LEGIDE_RUST_LIB}
    DEPENDS ${LEGIDE_RUST_SRCS}
            ${LEGIDE_RUST_WORKSPACE_DIR}/legide-rust/Cargo.toml
    WORKING_DIRECTORY ${LEGIDE_RUST_WORKSPACE_DIR}
    COMMENT "Building legide-rust (i686-pc-windows-gnu)"
    VERBATIM)
add_custom_target(rust_legide DEPENDS ${LEGIDE_RUST_LIB})

# C body replaced by Rust — kept on disk as the translation reference.
# list(APPEND SOURCE legide.c)

# ${SOURCE} is empty while the list above stays commented (see pciidex/CMakeLists.txt).
add_library(legide MODULE ${SOURCE} rust_cohab_stub.c legide.rc)
set_module_type(legide kernelmodedriver)

# Required for NT5.x pciidex.sys
add_custom_command(
    TARGET legide POST_BUILD
    COMMAND native-pefixup --section:.rsrc,!D $<TARGET_FILE:legide>
    VERBATIM)

add_dependencies(legide rust_legide)
target_link_libraries(legide ${LEGIDE_RUST_LIB})
add_importlibs(legide pciidex ntoskrnl hal)
add_cd_file(TARGET legide DESTINATION reactos/system32/drivers NO_CAB FOR all)
# legide_reg.inf is registered by drivers/storage/ide/CMakeLists.txt under VIBEOS_ATA_STACK.
