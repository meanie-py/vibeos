add_definitions(-D_COMDDK_)

include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/sound)

list(APPEND SOURCE
    # control.c  # replaced by Rust (sdk/lib/rust/wdmaud-rust/src/wdmaud/control.rs)
    # deviface.c  # replaced by Rust (sdk/lib/rust/wdmaud-rust/src/wdmaud/deviface.rs)
    # entry.c  # replaced by Rust (sdk/lib/rust/wdmaud-rust/src/wdmaud/entry.rs)
    # mmixer.c  # replaced by Rust (sdk/lib/rust/wdmaud-rust/src/wdmaud/mmixer.rs)
    # sup.c  # replaced by Rust (sdk/lib/rust/wdmaud-rust/src/wdmaud/sup.rs)
    rust_cohab_stub.c
    wdmaud.h)

# Every translation unit is Rust now (wdmaud-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wdmaud_rust wdmaud-rust "" TRUE)
add_library(wdmaud MODULE ${SOURCE} wdmaud.rc)
add_dependencies(wdmaud rust_wdmaud_rust)
target_link_libraries(wdmaud ${wdmaud_rust_LIB})
add_pch(wdmaud wdmaud.h SOURCE)
set_module_type(wdmaud kernelmodedriver)
target_link_libraries(wdmaud mmixer libcntpr pseh)
add_importlibs(wdmaud ntoskrnl ks hal)
add_cd_file(TARGET wdmaud DESTINATION reactos/system32/drivers FOR all)
