
spec2def(msutb.dll msutb.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # msutb.cpp  # replaced by Rust (sdk/lib/rust/msutb-rust/src/msutb/msutb_cpp_h.rs)
    rust_cohab_stub.cpp
    )

#file(GLOB msutb_rc_deps res/*.*)
#add_rc_deps(msutb.rc ${msutb_rc_deps})

# Every translation unit is Rust now (msutb-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(msutb_rust msutb-rust "" TRUE)
add_library(msutb MODULE
    ${SOURCE}
    msutb.rc
    ${CMAKE_CURRENT_BINARY_DIR}/msutb.def)
add_dependencies(msutb rust_msutb_rust)
target_link_libraries(msutb ${msutb_rust_LIB})
set_module_type(msutb win32dll UNICODE)
add_dependencies(msutb msctf psdk)
target_link_libraries(msutb cpprt wine uuid atl_classes cicero)
add_importlibs(msutb user32 gdi32 advapi32 msvcrt kernel32 ntdll)
add_delay_importlibs(msutb uxtheme imm32 shlwapi comctl32 msctf ole32 oleacc oleaut32 shell32)
add_cd_file(TARGET msutb DESTINATION reactos/system32 FOR all)
