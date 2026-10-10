
list(APPEND SOURCE
    # control.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/control.rs)
    # create.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/create.rs)
    # delete.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/delete.rs)
    # dependencies_tv1.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/dependencies_tv1.rs)
    # dependencies_tv2.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/dependencies_tv2.rs)
    # export.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/export.rs)
    # listview.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/listview.rs)
    # mainwnd.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/mainwnd.rs)
    # misc.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/misc.rs)
    # progress.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/progress.rs)
    # propsheet.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/propsheet.rs)
    # propsheet_depends.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/propsheet_depends.rs)
    # propsheet_general.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/propsheet_general.rs)
    # propsheet_logon.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/propsheet_logon.rs)
    # propsheet_recovery.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/propsheet_recovery.rs)
    # query.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/query.rs)
    # servman.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/servman.rs)
    # start.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/start.rs)
    # stop.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/stop.rs)
    # stop_dependencies.c  # replaced by Rust (sdk/lib/rust/servman-rust/src/servman/stop_dependencies.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    precomp.h)

file(GLOB servman_rc_deps res/*.*)
add_rc_deps(servman.rc ${servman_rc_deps})
# Every translation unit is Rust now (servman-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(servman_rust servman-rust "" TRUE)
add_executable(servman ${SOURCE} servman.rc)
add_dependencies(servman rust_servman_rust)
target_link_libraries(servman ${servman_rust_LIB})
set_module_type(servman win32gui UNICODE)
add_importlibs(servman user32 gdi32 advapi32 version comctl32 shell32 comdlg32 msvcrt kernel32 ntdll)
add_pch(servman precomp.h SOURCE)
add_cd_file(TARGET servman DESTINATION reactos/system32 FOR all)
