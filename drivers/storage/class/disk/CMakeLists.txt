
remove_definitions(-D_WIN32_WINNT=0x502)

# ============================================================================
# disk.sys — Rust cohabitation flip (all 5 C sources -> sdk/lib/rust/disk-rust)
# ============================================================================
# Every disk C source (data/disk/diskwmi/geometry/pnp) is translated to Rust
# (disk-rust, kept on disk per the never-delete rule). The PE DriverEntry comes
# from libdisk_rust.a; cmake keeps the MODULE link (set_module_type
# kernelmodedriver + add_importlibs(classpnp ntoskrnl hal)). disk.sys exports
# nothing but DriverEntry (no .spec), so this is simpler than classpnp's
# kerneldll cohab. Built on classpnp's Class* exports. Mirrors classpnp-rust.
set(DISK_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(DISK_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-disk-target")
set(DISK_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libdisk_rust.a")
if(ARCH STREQUAL "i386")
    set(DISK_RUST_TARGET "i686-pc-windows-gnu")
    set(DISK_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(DISK_RUST_TARGET "x86_64-pc-windows-gnu")
    set(DISK_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(DISK_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(DISK_RUST_RUSTFLAGS "")
endif()
add_custom_command(
    OUTPUT ${DISK_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${DISK_RUST_TARGET_DIR}
        RUSTFLAGS=${DISK_RUST_RUSTFLAGS}
        cargo build --release --target ${DISK_RUST_TARGET}
            -p disk-rust
            --manifest-path ${DISK_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${DISK_RUST_TARGET_DIR}/${DISK_RUST_TARGET}/release/libdisk_rust.a
        ${DISK_RUST_LIB}
    DEPENDS ${DISK_RUST_WORKSPACE_DIR}/disk-rust/Cargo.toml
            ${DISK_RUST_WORKSPACE_DIR}/disk-rust/src/lib.rs
    WORKING_DIRECTORY ${DISK_RUST_WORKSPACE_DIR}
    COMMENT "Building disk-rust (${DISK_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_disk DEPENDS ${DISK_RUST_LIB})

list(APPEND SOURCE
    rust_cohab_stub.c
    disk.h)
#   data.c disk.c diskwmi.c geometry.c pnp.c  # all replaced by Rust (disk-rust); kept on disk

add_library(disk MODULE ${SOURCE} disk.rc)

target_compile_definitions(disk PUBLIC
    DEBUG_USE_KDPRINT
    _WIN32_WINNT=0x602
    NTDDI_VERSION=0x06020000) # NTDDI_WIN8

if(CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(disk PRIVATE -Wno-format -Wno-pointer-sign)
    target_compile_options(disk PRIVATE -Wno-pointer-to-int-cast -Wno-switch)
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    target_compile_options(disk PRIVATE -Wno-unused-but-set-variable)
endif()

set_module_type(disk kernelmodedriver)
add_dependencies(disk rust_disk)
target_link_libraries(disk ${DISK_RUST_LIB} ntoskrnl_vista libcntpr wdmguid)
add_importlibs(disk classpnp ntoskrnl hal)
add_cd_file(TARGET disk DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_registry_inf(disk_reg.inf)
add_driver_inf(disk disk.inf)
