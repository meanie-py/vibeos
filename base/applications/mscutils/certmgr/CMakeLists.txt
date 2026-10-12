
# Every translation unit is Rust now (certmgr-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(certmgr_rust certmgr-rust "" TRUE)
add_executable(certmgr
    # certmgr.cpp  # replaced by Rust (sdk/lib/rust/certmgr-rust/src/certmgr/certmgr_cpp_h.rs)
    certmgr.h
    certmgr.rc
    resource.h

    CCert.hpp
    CStore.hpp
    # CStoreList.cpp  # replaced by Rust (sdk/lib/rust/certmgr-rust/src/certmgr/CStoreList_cpp_h.rs)
    rust_cohab_stub.cpp
    CStoreList.hpp
    CSplitter.hpp
    CSortableHeader.hpp
)
add_dependencies(certmgr rust_certmgr_rust)
target_link_libraries(certmgr ${certmgr_rust_LIB})
set_module_type(certmgr win32gui UNICODE)
target_link_libraries(certmgr uuid cpprt atl_classes)
add_importlibs(certmgr cryptui crypt32 ole32 user32 msvcrt kernel32 ntdll)
add_dependencies(certmgr psdk)
add_cd_file(TARGET certmgr DESTINATION reactos/system32 FOR all)
