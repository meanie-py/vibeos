
add_definitions(-DIDR_EXTRACTICON=101)
spec2def(mmcshext.dll mmcshext.spec)

list(APPEND SOURCE
    # mmcshext.cpp  # replaced by Rust (sdk/lib/rust/mmcshext-rust/src/mmcshext/mmcshext_cpp_h.rs)
    rust_cohab_stub.cpp
    )

# Every translation unit is Rust now (mmcshext-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mmcshext_rust mmcshext-rust "" TRUE)
add_library(mmcshext MODULE
    ${SOURCE}
    mmcshext.rc
    mmcshext.spec
    ${CMAKE_CURRENT_BINARY_DIR}/mmcshext.def)
add_dependencies(mmcshext rust_mmcshext_rust)
target_link_libraries(mmcshext ${mmcshext_rust_LIB})

set_module_type(mmcshext win32dll UNICODE)
target_link_libraries(mmcshext uuid cpprt atl_classes)
add_importlibs(mmcshext oleaut32 ole32 shlwapi comctl32 shell32 user32 advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET mmcshext DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
