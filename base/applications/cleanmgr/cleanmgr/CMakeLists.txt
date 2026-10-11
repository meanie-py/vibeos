
# Every translation unit is Rust now (cleanmgr-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(cleanmgr_rust cleanmgr-rust "" TRUE)
add_executable(cleanmgr
    # cleanmgr.cpp  # replaced by Rust (sdk/lib/rust/cleanmgr-rust/src/cleanmgr/cleanmgr_cpp_h.rs)
    cleanmgr.h
    cleanmgr.rc
    resource.h
    CEmptyVolumeCacheCallBack.hpp
    CProgressDlg.hpp
    # CSelectDriveDlg.cpp  # replaced by Rust (sdk/lib/rust/cleanmgr-rust/src/cleanmgr/CSelectDriveDlg_cpp_h.rs)
    # CCleanupHandler.cpp  # replaced by Rust (sdk/lib/rust/cleanmgr-rust/src/cleanmgr/CCleanupHandler_cpp_h.rs)
    CCleanupHandler.hpp
    # CCleanupHandlerList.cpp  # replaced by Rust (sdk/lib/rust/cleanmgr-rust/src/cleanmgr/CCleanupHandlerList_cpp_h.rs)
    rust_cohab_stub.cpp
    CCleanupHandlerList.hpp
)
add_dependencies(cleanmgr rust_cleanmgr_rust)
target_link_libraries(cleanmgr ${cleanmgr_rust_LIB})
set_module_type(cleanmgr win32gui UNICODE)
target_link_libraries(cleanmgr uuid cpprt atl_classes)
add_importlibs(cleanmgr shlwapi oleaut32 ole32 shell32 comctl32 user32 advapi32 msvcrt kernel32 ntdll)
add_dependencies(cleanmgr psdk)
add_cd_file(TARGET cleanmgr DESTINATION reactos/system32 FOR all)
