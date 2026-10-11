
remove_definitions(-D_WIN32_WINNT=0x502 -DWINVER=0x502)
add_definitions(-D_WIN32_WINNT=0x601)

# fltmc.cpp is replaced by Rust (sdk/lib/rust/fltmc-rust/src/fltmc/fltmc.rs, `rustconv port`);
# the C++ stays on disk. The target keeps a stub TU and links the Rust archive, which takes
# ATL's CStringW from atl-rust -- so no cpprt/atl_classes, and user32 for its LoadStringW.
rust_staticlib_variant(fltmc_rust fltmc-rust "" TRUE)
add_executable(fltmc rust_cohab_stub.cpp fltmc.rc)
add_dependencies(fltmc rust_fltmc_rust)
target_link_libraries(fltmc ${fltmc_rust_LIB})
set_module_type(fltmc win32cui UNICODE)
add_importlibs(fltmc fltlib msvcrt kernel32 advapi32 user32)
add_cd_file(TARGET fltmc DESTINATION reactos/system32 FOR all)
