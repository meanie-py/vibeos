
list(APPEND SOURCE
    # toolspage.c  # replaced by Rust (sdk/lib/rust/msconfig-rust/src/msconfig/toolspage.rs)
    # srvpage.c  # replaced by Rust (sdk/lib/rust/msconfig-rust/src/msconfig/srvpage.rs)
    # systempage.c  # replaced by Rust (sdk/lib/rust/msconfig-rust/src/msconfig/systempage.rs)
    # startuppage.c  # replaced by Rust (sdk/lib/rust/msconfig-rust/src/msconfig/startuppage.rs)
    # freeldrpage.c  # replaced by Rust (sdk/lib/rust/msconfig-rust/src/msconfig/freeldrpage.rs)
    # generalpage.c  # replaced by Rust (sdk/lib/rust/msconfig-rust/src/msconfig/generalpage.rs)
    # msconfig.c  # replaced by Rust (sdk/lib/rust/msconfig-rust/src/msconfig/msconfig.rs)
    rust_cohab_stub.c
    precomp.h)

add_rc_deps(msconfig.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/msconfig.ico)
# Every translation unit is Rust now (msconfig-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(msconfig_rust msconfig-rust "" TRUE)
add_executable(msconfig ${SOURCE} msconfig.rc)
add_dependencies(msconfig rust_msconfig_rust)
target_link_libraries(msconfig ${msconfig_rust_LIB})
set_module_type(msconfig win32gui UNICODE)
add_importlibs(msconfig user32 advapi32 version comctl32 shell32 shlwapi msvcrt kernel32)
add_pch(msconfig precomp.h SOURCE)
add_cd_file(TARGET msconfig DESTINATION reactos/system32 FOR all)
