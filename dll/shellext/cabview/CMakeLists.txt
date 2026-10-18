
spec2def(cabview.dll cabview.spec)

list(APPEND SOURCE
    # cabview.cpp  # replaced by Rust (sdk/lib/rust/cabview-rust/src/cabview/cabview_cpp_h.rs)
    # extract.cpp  # replaced by Rust (sdk/lib/rust/cabview-rust/src/cabview/extract_cpp_h.rs)
    # folder.cpp  # replaced by Rust (sdk/lib/rust/cabview-rust/src/cabview/folder_cpp_h.rs)
    rust_cohab_stub.cpp
    resource.h)

# cabview-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(CABVIEW_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(CABVIEW_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (cabview-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(cabview_rust cabview-rust "" TRUE ${CABVIEW_RUST_ENV})
add_library(cabview MODULE
    ${SOURCE}
    cabview.spec
    cabview.rc
    ${CMAKE_CURRENT_BINARY_DIR}/cabview.def)
add_dependencies(cabview rust_cabview_rust)
target_link_libraries(cabview ${cabview_rust_LIB})

set_module_type(cabview win32dll UNICODE)
target_link_libraries(cabview uuid cpprt atl_classes)
set_target_cpp_properties(cabview WITH_EXCEPTIONS)
add_importlibs(cabview cabinet oleaut32 ole32 shlwapi comctl32 shell32 user32 advapi32 msvcrt kernel32 ntdll)
add_pch(cabview precomp.h SOURCE)
add_cd_file(TARGET cabview DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
