
list(APPEND SOURCE
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/boot.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/check.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/common.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/fat.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/file.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/io.c
    ${REACTOS_SOURCE_DIR}/vendor/dosfstools/lfn.c
    # common.c  # replaced by Rust (sdk/lib/rust/vfatlib-rust/src/vfatlib/common.rs)
    # fat12.c  # replaced by Rust (sdk/lib/rust/vfatlib-rust/src/vfatlib/fat12.rs)
    # fat16.c  # replaced by Rust (sdk/lib/rust/vfatlib-rust/src/vfatlib/fat16.rs)
    # fat32.c  # replaced by Rust (sdk/lib/rust/vfatlib-rust/src/vfatlib/fat32.rs)
    # vfatlib.c  # replaced by Rust (sdk/lib/rust/vfatlib-rust/src/vfatlib/vfatlib.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    vfatlib.h)

# dosfsck.h includes the vendored checker as "dosfstools/x.h".
include_directories(${REACTOS_SOURCE_DIR}/vendor)

# Every own translation unit is Rust now (vfatlib-rust, `rustconv port`); the vendored
# dosfstools TUs above stay C. The own C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(vfatlib_rust vfatlib-rust "" TRUE)
add_library(vfatlib ${SOURCE})
add_dependencies(vfatlib rust_vfatlib_rust)
target_link_libraries(vfatlib ${vfatlib_rust_LIB})
add_dependencies(vfatlib xdk)
add_pch(vfatlib vfatlib.h SOURCE)
