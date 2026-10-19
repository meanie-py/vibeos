
project(SHELL)

spec2def(ntobjshex.dll ntobjshex.spec)

file(GLOB_RECURSE ntobjshex_rc_deps resources/*.*)
add_rc_deps(ntobjshex.rc ${ntobjshex_rc_deps})

list(APPEND SOURCE
    # ntobjenum.cpp  # replaced by Rust (sdk/lib/rust/ntobjshex-rust/src/ntobjshex/ntobjenum_cpp_h.rs)
    # ntobjfolder.cpp  # replaced by Rust (sdk/lib/rust/ntobjshex-rust/src/ntobjshex/ntobjfolder_cpp_h.rs)
    # ntobjshex.cpp  # replaced by Rust (sdk/lib/rust/ntobjshex-rust/src/ntobjshex/ntobjshex_cpp_h.rs)
    # regfolder.cpp  # replaced by Rust (sdk/lib/rust/ntobjshex-rust/src/ntobjshex/regfolder_cpp_h.rs)
    rust_cohab_stub.cpp
    precomp.h)

# Every translation unit is Rust now (ntobjshex-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(ntobjshex_rust ntobjshex-rust "" TRUE)
add_library(ntobjshex MODULE
    ${SOURCE}
    ntobjshex.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ntobjshex.def)
add_dependencies(ntobjshex rust_ntobjshex_rust)
target_link_libraries(ntobjshex ${ntobjshex_rust_LIB})

set_module_type(ntobjshex win32dll UNICODE)
target_link_libraries(ntobjshex uuid wine cpprt atl_classes)

add_importlibs(ntobjshex
    advapi32
    winmm
    ole32
    oleaut32
    shlwapi
    shell32
    comctl32
    msvcrt
    gdi32
    user32
    kernel32
    ntdll)

add_pch(ntobjshex precomp.h SOURCE)
add_cd_file(TARGET ntobjshex DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
