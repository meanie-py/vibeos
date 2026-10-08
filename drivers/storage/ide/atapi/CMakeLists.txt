
# ============================================================================
# atapi.sys — Rust cohabitation flip (upstream's 2026 ATA port driver -> atapi-rust)
# ============================================================================
# This is upstream's NEW atapi.sys: a WDM port driver that sits on pciidex's channel PDOs,
# enumerates ATA/ATAPI devices and exposes them to the class drivers through a SATL (SCSI to
# ATA translation). Translated to Rust in sdk/lib/rust/atapi-rust (rustconv port, sealed
# DeepSeek). The name used to mean the NT4-DDK SCSIPORT miniport; that translation is parked
# as atapi-legacy-rust (never-delete) and its C is gone (DDK EULA, 2026-09-20).
# sptilib (SCSI pass-through helpers) and memcmp stay C static libs the Rust calls.
set(ATAPI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(ATAPI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-atapi-target" atapi-rust)
set(ATAPI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libatapi_rust.a")
if(ARCH STREQUAL "i386")
    set(ATAPI_RUST_TARGET "i686-pc-windows-gnu")
    set(ATAPI_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(ATAPI_RUST_TARGET "x86_64-pc-windows-gnu")
    set(ATAPI_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(ATAPI_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(ATAPI_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(ATAPI_RUST_SRCS "${ATAPI_RUST_WORKSPACE_DIR}" atapi-rust)
add_custom_command(
    OUTPUT ${ATAPI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${ATAPI_RUST_TARGET_DIR}
        RUSTFLAGS=${ATAPI_RUST_RUSTFLAGS}
        cargo build --release --target ${ATAPI_RUST_TARGET}
            -p atapi-rust
            --manifest-path ${ATAPI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${ATAPI_RUST_TARGET_DIR}/${ATAPI_RUST_TARGET}/release/libatapi_rust.a
        ${ATAPI_RUST_LIB}
    DEPENDS ${ATAPI_RUST_SRCS}
            ${ATAPI_RUST_WORKSPACE_DIR}/atapi-rust/Cargo.toml
    WORKING_DIRECTORY ${ATAPI_RUST_WORKSPACE_DIR}
    COMMENT "Building atapi-rust (${ATAPI_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_atapi DEPENDS ${ATAPI_RUST_LIB})

# C bodies replaced by Rust — kept on disk as the translation reference.
# list(APPEND SOURCE
#     atapi.c
#     dev_config.c
#     dev_error.c
#     dev_identify.c
#     dev_power.c
#     dev_timings.c
#     enum.c
#     fdo.c
#     ioctl.c
#     pdo.c
#     portstate.c
#     satl.c
#     scsi.c
#     smart.c
#     wmi.c)

# ${SOURCE} is empty while the list above stays commented (see pciidex/CMakeLists.txt).
add_library(atapi MODULE ${SOURCE} rust_cohab_stub.c rust_varargs_shim.c atapi.rc)

if(STACK_PROTECTOR)
    target_sources(atapi PRIVATE $<TARGET_OBJECTS:gcc_ssp_nt>)
endif()

set_module_type(atapi kernelmodedriver)
add_dependencies(atapi rust_atapi)
target_link_libraries(atapi ${ATAPI_RUST_LIB} memcmp sptilib)
add_importlibs(atapi ntoskrnl hal wmilib)
add_cd_file(TARGET atapi DESTINATION reactos/system32/drivers NO_CAB FOR all)
# atapi_reg.inf registers the service; it is part of the opt-in ATA stack and is added by
# drivers/storage/ide/CMakeLists.txt under VIBEOS_ATA_STACK, not unconditionally.
