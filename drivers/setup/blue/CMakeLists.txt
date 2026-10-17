
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers)

list(APPEND SOURCE
    # blue.c  # replaced by Rust (sdk/lib/rust/blue-rust/src/blue/blue.rs)
    # font.c  # replaced by Rust (sdk/lib/rust/blue-rust/src/blue/font.rs)
    rust_cohab_stub.c
    blue.h)

# Every translation unit is Rust now (blue-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(blue_rust blue-rust "" TRUE)
add_library(blue MODULE ${SOURCE} blue.rc)
add_dependencies(blue rust_blue_rust)
target_link_libraries(blue ${blue_rust_LIB})
set_module_type(blue kernelmodedriver)
add_importlibs(blue ntoskrnl hal)
add_pch(blue blue.h SOURCE)
add_cd_file(TARGET blue DESTINATION reactos/system32/drivers NO_CAB FOR all)
