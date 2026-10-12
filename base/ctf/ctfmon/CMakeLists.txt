
list(APPEND SOURCE
    # ctfmon.cpp  # replaced by Rust (sdk/lib/rust/ctfmon-rust/src/ctfmon/ctfmon_cpp_h.rs)
    # CLoaderWnd.cpp  # replaced by Rust (sdk/lib/rust/ctfmon-rust/src/ctfmon/CLoaderWnd_cpp_h.rs)
    # CRegWatcher.cpp  # replaced by Rust (sdk/lib/rust/ctfmon-rust/src/ctfmon/CRegWatcher_cpp_h.rs)
    rust_cohab_stub.cpp
    )

add_rc_deps(ctfmon.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/ctfmon.ico)
# Every translation unit is Rust now (ctfmon-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(ctfmon_rust ctfmon-rust "" TRUE)
add_executable(ctfmon ${SOURCE} ctfmon.rc)
add_dependencies(ctfmon rust_ctfmon_rust)
target_link_libraries(ctfmon ${ctfmon_rust_LIB})
set_module_type(ctfmon win32gui UNICODE)
add_dependencies(ctfmon msctf msutb)
target_link_libraries(ctfmon uuid cicero)
add_importlibs(ctfmon msctf msutb advapi32 shell32 user32 msvcrt kernel32)
# cicero's CUIF/IMC code (gdi32, imm32, comctl32) rides in ctfmon's Rust archive: its codegen
# units come along whole even though ctfmon calls none of it. Delay imports load only if called.
add_delay_importlibs(ctfmon gdi32 imm32 comctl32)
add_pch(ctfmon precomp.h SOURCE)
add_cd_file(TARGET ctfmon DESTINATION reactos/system32 FOR all)
