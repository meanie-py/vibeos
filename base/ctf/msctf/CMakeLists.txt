
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

spec2def(msctf.dll msctf.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    precomp.h
    # categorymgr.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/categorymgr_cpp_h.rs)
    # compartmentmgr.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/compartmentmgr_cpp_h.rs)
    # context.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/context_cpp_h.rs)
    # displayattributemgr.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/displayattributemgr_cpp_h.rs)
    # documentmgr.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/documentmgr_cpp_h.rs)
    # inputprocessor.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/inputprocessor_cpp_h.rs)
    # langbarmgr.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/langbarmgr_cpp_h.rs)
    # mlng.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/mlng_cpp_h.rs)
    # msctf.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/msctf_cpp_h.rs)
    # range.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/range_cpp_h.rs)
    # threadmgr.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/threadmgr_cpp_h.rs)
    # utils.cpp  # replaced by Rust (sdk/lib/rust/msctf-rust/src/msctf/utils_cpp_h.rs)
    rust_cohab_stub.cpp
    ${CMAKE_CURRENT_BINARY_DIR}/msctf_stubs.c)

# Every translation unit is Rust now (msctf-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(msctf_rust msctf-rust "" TRUE)
add_library(msctf MODULE
    ${SOURCE}
    version.rc
    ${CMAKE_CURRENT_BINARY_DIR}/msctf.def)
add_dependencies(msctf rust_msctf_rust)
target_link_libraries(msctf ${msctf_rust_LIB})

set_module_type(msctf win32dll UNICODE)
target_link_libraries(msctf uuid wine cicero)
add_importlibs(msctf user32 advapi32 advapi32_vista msvcrt kernel32 ntdll)
# comctl32: cicero's CUIF classes (ImageList_*) now ride in msctf's Rust archive; a codegen unit
# brings them along even though msctf never calls them, and a delay import is loaded only if
# something does.
add_delay_importlibs(msctf shell32 shlwapi ole32 oleaut32 imm32 gdi32 comctl32)
add_pch(msctf precomp.h SOURCE)
add_cd_file(TARGET msctf DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(msctf) # CORE-5743: No CONST_VTABLE
