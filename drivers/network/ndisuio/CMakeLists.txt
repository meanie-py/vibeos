
add_definitions(
    -DNDIS50
    -D_NTDRIVER_)

include_directories(
    BEFORE include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers/ndisuio)

list(APPEND SOURCE
    # createclose.c  # replaced by Rust (sdk/lib/rust/ndisuio-rust/src/ndisuio/createclose.rs)
    # ioctl.c  # replaced by Rust (sdk/lib/rust/ndisuio-rust/src/ndisuio/ioctl.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/ndisuio-rust/src/ndisuio/main.rs)
    # misc.c  # replaced by Rust (sdk/lib/rust/ndisuio-rust/src/ndisuio/misc.rs)
    # protocol.c  # replaced by Rust (sdk/lib/rust/ndisuio-rust/src/ndisuio/protocol.rs)
    # readwrite.c  # replaced by Rust (sdk/lib/rust/ndisuio-rust/src/ndisuio/readwrite.rs)
    rust_cohab_stub.c
    ndisuio.h)

# Every translation unit is Rust now (ndisuio-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(ndisuio_rust ndisuio-rust "" TRUE)
add_library(ndisuio MODULE ${SOURCE} ndisuio.rc)
add_dependencies(ndisuio rust_ndisuio_rust)
target_link_libraries(ndisuio ${ndisuio_rust_LIB})
set_module_type(ndisuio kernelmodedriver)
add_importlibs(ndisuio ndis ntoskrnl hal)
add_pch(ndisuio ndisuio.h SOURCE)
add_cd_file(TARGET ndisuio DESTINATION reactos/system32/drivers FOR all)
