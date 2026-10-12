
spec2def(indicdll.dll indicdll.spec)

file(GLOB indicdll_rc_deps res/*.*)
add_rc_deps(indicdll.rc ${indicdll_rc_deps})

list(APPEND SOURCE
    # indicdll.c  # replaced by Rust (sdk/lib/rust/indicdll-rust/src/indicdll/indicdll.rs)
    rust_cohab_stub.c
    indicdll.rc
    ${CMAKE_CURRENT_BINARY_DIR}/indicdll.def)

# Every translation unit is Rust now (indicdll-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(indicdll_rust indicdll-rust "" TRUE)
add_library(indicdll MODULE ${SOURCE})
add_dependencies(indicdll rust_indicdll_rust)
target_link_libraries(indicdll ${indicdll_rust_LIB})
set_module_type(indicdll win32dll UNICODE)
target_link_libraries(indicdll wine)
add_importlibs(indicdll user32 comctl32 msvcrt kernel32 ntdll)
add_cd_file(TARGET indicdll DESTINATION reactos/system32 FOR all)
