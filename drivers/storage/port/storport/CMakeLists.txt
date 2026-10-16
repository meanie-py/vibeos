
# fdo/miniport/misc/pdo/storport/stubs.c replaced by Rust (storport-rust); guid.c stays C
# (GUID data TU). spec2def still exports the StorPort* API (satisfied by Rust #[no_mangle])
# + the import lib storahci links. NOT boot-loaded in smoke -> link-clean x3 bar.

spec2def(storport.sys storport.spec ADD_IMPORTLIB)
include_directories(${REACTOS_SOURCE_DIR}/sdk/lib/drivers/sptilib)

set(STORPORT_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(STORPORT_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-storport-target" storport-rust)
set(STORPORT_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libstorport_rust.a")
if(ARCH STREQUAL "i386")
    set(STORPORT_RUST_TARGET "i686-pc-windows-gnu")
    set(STORPORT_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(STORPORT_RUST_TARGET "x86_64-pc-windows-gnu")
    set(STORPORT_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(STORPORT_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(STORPORT_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(STORPORT_RUST_SRCS "${STORPORT_RUST_WORKSPACE_DIR}" storport-rust)
add_custom_command(
    OUTPUT ${STORPORT_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${STORPORT_RUST_TARGET_DIR} RUSTFLAGS=${STORPORT_RUST_RUSTFLAGS}
        cargo build --release --target ${STORPORT_RUST_TARGET} -p storport-rust --manifest-path ${STORPORT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${STORPORT_RUST_TARGET_DIR}/${STORPORT_RUST_TARGET}/release/libstorport_rust.a ${STORPORT_RUST_LIB}
    DEPENDS ${STORPORT_RUST_SRCS} ${STORPORT_RUST_WORKSPACE_DIR}/storport-rust/Cargo.toml
    WORKING_DIRECTORY ${STORPORT_RUST_WORKSPACE_DIR}
    COMMENT "Building storport-rust (${STORPORT_RUST_TARGET})" VERBATIM)
add_custom_target(rust_storport DEPENDS ${STORPORT_RUST_LIB})

# rust_varargs_shim.c — va_start/va_arg trampolines for the cdecl-varargs exports
# (StorPortDebugPrint/ExtendedFunction/Notification). Stable Rust cannot DEFINE a
# C-variadic fn; the logic stays in storport-rust's *_v twins. Delete once c_variadic
# stabilizes (rust-lang#44930).
# C bodies replaced by Rust (storport-rust), kept on disk as the translation reference:
# commented entries of the list the target expands, so the translator can prove their owner.
list(APPEND SOURCE
    #fdo.c
    #ioctl.c
    #miniport.c
    #misc.c
    #pdo.c
    #srb.c
    #storport.c
    #stubs.c
    )
add_library(storport MODULE ${SOURCE} guid.c rust_varargs_shim.c storport.rc ${CMAKE_CURRENT_BINARY_DIR}/storport.def)
set_module_type(storport kernelmodedriver)
add_importlibs(storport ntoskrnl hal)
add_dependencies(storport rust_storport)
target_link_libraries(storport ${STORPORT_RUST_LIB})
add_cd_file(TARGET storport DESTINATION reactos/system32/drivers NO_CAB FOR all)
