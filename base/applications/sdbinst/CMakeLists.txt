project(appcompat)
include_directories(${REACTOS_SOURCE_DIR}/dll/appcompat/apphelp)
# Every translation unit is Rust now (sdbinst-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(sdbinst_rust sdbinst-rust "" TRUE)
add_executable(sdbinst
    # sdbinst.c  # replaced by Rust (sdk/lib/rust/sdbinst-rust/src/sdbinst/sdbinst.rs)
    rust_cohab_stub.c
    )
add_dependencies(sdbinst rust_sdbinst_rust)
target_link_libraries(sdbinst ${sdbinst_rust_LIB})

set_module_type(sdbinst win32cui UNICODE)
add_importlibs(sdbinst shlwapi advapi32 apphelp ole32 msvcrt kernel32 ntdll)
add_cd_file(TARGET sdbinst DESTINATION reactos/system32 FOR all)
