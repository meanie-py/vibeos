
spec2def(scsiport.sys scsiport.spec ADD_IMPORTLIB)

# Link GCC stack protector lib
if(STACK_PROTECTOR)
    target_link_libraries(libscsiport INTERFACE gcc_ssp_scsiport)
endif()

# ============================================================================
# scsiport.sys — Rust cohabitation flip (8 .c -> Rust; guid.c stays C)
# ============================================================================
# The SCSI port driver is translated to Rust (scsiport-rust, kept on disk per the
# never-delete rule). DriverEntry + the ~45 scsiport.spec ScsiPort* exports come
# from libscsiport_rust.a; the SRB/CDB/scsiport.h-private DDK types come from the
# per-driver bindgen substrate (scsiport-rust/src/types_*.rs). guid.c stays as the
# C GUID-data TU (DEFINE_GUID home for GUID_DEVINTERFACE_STORAGEPORT, referenced
# from Rust as an extern). ScsiPortNotification/ScsiDebugPrint are cdecl varargs
# (c_variadic) so the cargo build runs under RUSTC_BOOTSTRAP=1. Standard per-arch
# RUST_TARGET, i386 no-SSE RUSTFLAGS.
set(SCSIPORT_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(SCSIPORT_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-scsiport-target" scsiport-rust)
set(SCSIPORT_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libscsiport_rust.a")
if(ARCH STREQUAL "i386")
    set(SCSIPORT_RUST_TARGET "i686-pc-windows-gnu")
    set(SCSIPORT_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(SCSIPORT_RUST_TARGET "x86_64-pc-windows-gnu")
    set(SCSIPORT_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(SCSIPORT_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(SCSIPORT_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(SCSIPORT_RUST_SRCS "${SCSIPORT_RUST_WORKSPACE_DIR}" scsiport-rust)
add_custom_command(
    OUTPUT ${SCSIPORT_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SCSIPORT_RUST_TARGET_DIR}
        RUSTFLAGS=${SCSIPORT_RUST_RUSTFLAGS}
        cargo build --release --target ${SCSIPORT_RUST_TARGET}
            -p scsiport-rust
            --manifest-path ${SCSIPORT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${SCSIPORT_RUST_TARGET_DIR}/${SCSIPORT_RUST_TARGET}/release/libscsiport_rust.a
        ${SCSIPORT_RUST_LIB}
    DEPENDS ${SCSIPORT_RUST_SRCS}
            ${SCSIPORT_RUST_WORKSPACE_DIR}/scsiport-rust/Cargo.toml
    WORKING_DIRECTORY ${SCSIPORT_RUST_WORKSPACE_DIR}
    COMMENT "Building scsiport-rust (${SCSIPORT_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_scsiport DEPENDS ${SCSIPORT_RUST_LIB})

# C bodies replaced by Rust (fdo.c ioctl.c pdo.c power.c registry.c scsi.c
# scsiport.c stubs.c) — kept on disk per the never-delete rule.
list(APPEND PCH_SKIP_SOURCE
    guid.c)

# rust_varargs_shim.c — va_start/va_arg trampolines for the cdecl-varargs exports
# (ScsiDebugPrint / ScsiPortNotification). Stable Rust cannot DEFINE a C-variadic fn;
# the logic stays in scsiport-rust's *_v twins. Delete once c_variadic stabilizes
# (rust-lang#44930).
add_library(scsiport MODULE
    rust_cohab_stub.c
    rust_varargs_shim.c
    ${PCH_SKIP_SOURCE}
    scsiport.rc
    ${CMAKE_CURRENT_BINARY_DIR}/scsiport.def)

set_module_type(scsiport kernelmodedriver)
target_link_libraries(scsiport sptilib ${PSEH_LIB})
add_importlibs(scsiport ntoskrnl hal)
add_dependencies(scsiport rust_scsiport)
target_link_libraries(scsiport ${SCSIPORT_RUST_LIB})
add_cd_file(TARGET scsiport DESTINATION reactos/system32/drivers NO_CAB FOR all)
