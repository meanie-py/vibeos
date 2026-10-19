
remove_definitions(-D_WIN32_WINNT=0x502 -DWINVER=0x502)
add_definitions(-D_WIN32_WINNT=0x601 -DWINVER=0x601)

spec2def(fontext.dll fontext.spec)

list(APPEND SOURCE
    # CDataObject.cpp  # replaced by Rust (sdk/lib/rust/fontext-rust/src/fontext/CDataObject_cpp_h.rs)
    # CEnumFonts.cpp  # replaced by Rust (sdk/lib/rust/fontext-rust/src/fontext/CEnumFonts_cpp_h.rs)
    # CFontBackgroundMenu.cpp  # replaced by Rust (sdk/lib/rust/fontext-rust/src/fontext/CFontBackgroundMenu_cpp_h.rs)
    # CFontCache.cpp  # replaced by Rust (sdk/lib/rust/fontext-rust/src/fontext/CFontCache_cpp_h.rs)
    # CFontExt.cpp  # replaced by Rust (sdk/lib/rust/fontext-rust/src/fontext/CFontExt_cpp_h.rs)
    # CFontFolderViewCB.cpp  # replaced by Rust (sdk/lib/rust/fontext-rust/src/fontext/CFontFolderViewCB_cpp_h.rs)
    # fontext.cpp  # replaced by Rust (sdk/lib/rust/fontext-rust/src/fontext/fontext_cpp_h.rs)
    # fontpidl.cpp  # replaced by Rust (sdk/lib/rust/fontext-rust/src/fontext/fontpidl_cpp_h.rs)
    rust_cohab_stub.cpp
    precomp.h)

# Every translation unit is Rust now (fontext-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(fontext_rust fontext-rust "" TRUE)
add_library(fontext MODULE
    ${SOURCE}
    fontext.rc
    fontext.spec
    ${CMAKE_CURRENT_BINARY_DIR}/fontext.def)
add_dependencies(fontext rust_fontext_rust)
target_link_libraries(fontext ${fontext_rust_LIB})

set_module_type(fontext win32dll UNICODE)
target_link_libraries(fontext uuid wine cpprt atl_classes)
add_delay_importlibs(fontext ole32 oleaut32 shlwapi gdi32)
add_importlibs(fontext shell32 advapi32 user32 msvcrt kernel32 ntdll)
add_pch(fontext precomp.h SOURCE)
add_cd_file(TARGET fontext DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
