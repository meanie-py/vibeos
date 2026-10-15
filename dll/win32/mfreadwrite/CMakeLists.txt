
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(mfreadwrite.dll mfreadwrite.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # reader.c  # replaced by Rust (sdk/lib/rust/mfreadwrite-rust/src/mfreadwrite/reader.rs)
    # writer.c  # replaced by Rust (sdk/lib/rust/mfreadwrite-rust/src/mfreadwrite/writer.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/mfreadwrite_stubs.c)

# Every translation unit is Rust now (mfreadwrite-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mfreadwrite_rust mfreadwrite-rust "" TRUE)
add_library(mfreadwrite MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/mfreadwrite.def)
add_dependencies(mfreadwrite rust_mfreadwrite_rust)
target_link_libraries(mfreadwrite ${mfreadwrite_rust_LIB})

set_module_type(mfreadwrite win32dll)
add_idl_reg_scripts(mfreadwrite registry mf_classes.idl)
target_link_libraries(mfreadwrite wine wine_dll_register wine_dll_canunload uuid mfuuid oldnames)
add_delay_importlibs(mfreadwrite mf)
add_importlibs(mfreadwrite mfplat ole32 kernelbase msvcrt kernel32 kernel32_vista ntdll)
add_dependencies(mfreadwrite psdk wineheaders)
add_cd_file(TARGET mfreadwrite DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(mfreadwrite)
