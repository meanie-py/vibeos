
PROJECT(mmc)

include_directories(${REACTOS_SOURCE_DIR}/sdk/lib/atl)
add_definitions(-D_ATL_NO_EXCEPTIONS)

list(APPEND CPP_SOURCE
    # CAboutSnapinDialog.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/CAboutSnapinDialog_cpp_h.rs)
    # CActionWnd.cpp  # upstream 88da81d9b8b's action pane: not translated yet (the Rust is from before it); owed to mmc_new-rust
    # CAddAdvancedDialog.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/CAddAdvancedDialog_cpp_h.rs)
    # CAddDialog.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/CAddDialog_cpp_h.rs)
    # CConsoleWnd.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/CConsoleWnd_cpp_h.rs)
    # CMainWnd.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/CMainWnd_cpp_h.rs)
    # COptionsDialog.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/COptionsDialog_cpp_h.rs)
    # CRecentFileEntry.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/CRecentFileEntry_cpp_h.rs)
    # CSnapin.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/CSnapin_cpp_h.rs)
    # CSnapinCacheEntry.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/CSnapinCacheEntry_cpp_h.rs)
    # CSnapinAlias.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/CSnapinAlias_cpp_h.rs)
    # misc.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/misc_cpp_h.rs)
    # mscfile.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/mscfile_cpp_h.rs)
    # mmc.cpp  # replaced by Rust (sdk/lib/rust/mmc_new-rust/src/mmc_new/mmc_cpp_h.rs)
    rust_cohab_stub.cpp
    # This is here so that they show up in visual studio:
    CAddDialog.h
    CCustomizeDialog.h
    COptionsDialog.h
    CConsoleWnd.h
    CMainWnd.h
    CSnapin.h
    CImageList.h
    precomp.h
    resource.h)

add_rc_deps(mmc.rc ${CMAKE_CURRENT_SOURCE_DIR}/resources/mmc.ico)
# Every translation unit is Rust now (mmc_new-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mmc_new_rust mmc_new-rust "" TRUE)
add_executable(mmc_new ${CPP_SOURCE} mmc.rc)
add_dependencies(mmc_new rust_mmc_new_rust)
target_link_libraries(mmc_new ${mmc_new_rust_LIB})
set_module_type(mmc_new win32gui UNICODE)
target_link_libraries(mmc_new uuid cpprt mmc_uuids)
add_importlibs(mmc_new oleaut32 ole32 shlwapi shell32 advapi32 comdlg32 comctl32 user32 gdi32 msvcrt kernel32 ntdll)
add_pch(mmc_new precomp.h CPP_SOURCE)
add_dependencies(mmc_new psdk)
add_cd_file(TARGET mmc_new DESTINATION reactos/system32 FOR all)
