
list(APPEND SOURCE
    # kmixer.c  # replaced by Rust (sdk/lib/rust/kmixer-rust/src/kmixer/kmixer.rs)
    # filter.c  # replaced by Rust (sdk/lib/rust/kmixer-rust/src/kmixer/filter.rs)
    # pin.c  # replaced by Rust (sdk/lib/rust/kmixer-rust/src/kmixer/pin.rs)
    rust_cohab_stub.c
    kmixer.h)

# Every translation unit is Rust now (kmixer-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(kmixer_rust kmixer-rust "" TRUE)
add_library(kmixer MODULE ${SOURCE})
add_dependencies(kmixer rust_kmixer_rust)
target_link_libraries(kmixer ${kmixer_rust_LIB})
add_pch(kmixer kmixer.h SOURCE)
set_module_type(kmixer kernelmodedriver)
target_link_libraries(kmixer libcntpr libsamplerate)
add_importlibs(kmixer ntoskrnl ks hal)
add_cd_file(TARGET kmixer DESTINATION reactos/system32/drivers FOR all)
