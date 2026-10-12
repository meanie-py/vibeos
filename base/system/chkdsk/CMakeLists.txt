
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/fmifs)

# Every translation unit is Rust now (chkdsk-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(chkdsk_rust chkdsk-rust "" TRUE)
add_executable(chkdsk
    # chkdsk.c  # replaced by Rust (sdk/lib/rust/chkdsk-rust/src/chkdsk/chkdsk.rs)
    rust_cohab_stub.c
    chkdsk.rc)
add_dependencies(chkdsk rust_chkdsk_rust)
target_link_libraries(chkdsk ${chkdsk_rust_LIB})
set_module_type(chkdsk win32cui UNICODE)
target_compile_definitions(chkdsk PRIVATE _CRT_NON_CONFORMING_SWPRINTFS)
target_link_libraries(chkdsk conutils)
add_importlibs(chkdsk fmifs msvcrt kernel32)
add_cd_file(TARGET chkdsk DESTINATION reactos/system32 FOR all)
