
if(DBG)
    add_definitions(-D_DEBUG=1)
endif()

list(APPEND SOURCE
    # about.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/about.rs)
    # childwnd.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/childwnd.rs)
    # edit.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/edit.rs)
    # error.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/error.rs)
    # find.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/find.rs)
    # framewnd.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/framewnd.rs)
    # hexedit.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/hexedit.rs)
    # listview.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/listview.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/main.rs)
    # regedit.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/regedit.rs)
    # regproc.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/regproc.rs)
    # security.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/security.rs)
    # settings.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/settings.rs)
    # treeview.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/treeview.rs)
    # txtproc.c  # replaced by Rust (sdk/lib/rust/regedit-rust/src/regedit/txtproc.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    regedit.h)

file(GLOB regedit_rc_deps res/*.*)
add_rc_deps(regedit.rc ${regedit_rc_deps})
# Every translation unit is Rust now (regedit-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(regedit_rust regedit-rust "" TRUE)
add_executable(regedit ${SOURCE} regedit.rc)
add_dependencies(regedit rust_regedit_rust)
target_link_libraries(regedit ${regedit_rust_LIB})
add_pch(regedit regedit.h SOURCE)
set_module_type(regedit win32gui UNICODE)
target_compile_definitions(regedit PRIVATE _CRT_NON_CONFORMING_SWPRINTFS)
target_link_libraries(regedit uuid wine)
add_importlibs(regedit user32 gdi32 advapi32 ole32 shell32 comctl32 comdlg32 shlwapi msvcrt kernel32 ntdll)
add_cd_file(TARGET regedit DESTINATION reactos FOR all)

add_subdirectory(clb)
