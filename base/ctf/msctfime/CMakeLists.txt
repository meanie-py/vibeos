
spec2def(msctfime.ime msctfime.spec)

list(APPEND SOURCE
    # bridge.cpp  # replaced by Rust (sdk/lib/rust/msctfime-rust/src/msctfime/bridge_cpp_h.rs)
    # inputcontext.cpp  # replaced by Rust (sdk/lib/rust/msctfime-rust/src/msctfime/inputcontext_cpp_h.rs)
    # misc.cpp  # replaced by Rust (sdk/lib/rust/msctfime-rust/src/msctfime/misc_cpp_h.rs)
    # msctfime.cpp  # replaced by Rust (sdk/lib/rust/msctfime-rust/src/msctfime/msctfime_cpp_h.rs)
    # profile.cpp  # replaced by Rust (sdk/lib/rust/msctfime-rust/src/msctfime/profile_cpp_h.rs)
    # sinks.cpp  # replaced by Rust (sdk/lib/rust/msctfime-rust/src/msctfime/sinks_cpp_h.rs)
    # tls.cpp  # replaced by Rust (sdk/lib/rust/msctfime-rust/src/msctfime/tls_cpp_h.rs)
    # ui.cpp  # replaced by Rust (sdk/lib/rust/msctfime-rust/src/msctfime/ui_cpp_h.rs)
    rust_cohab_stub.cpp
    )

file(GLOB msctfime_rc_deps res/*.*)
add_rc_deps(msctfime.rc ${msctfime_rc_deps})

# Every translation unit is Rust now (msctfime-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(msctfime_rust msctfime-rust "" TRUE)
add_library(msctfime MODULE
    ${SOURCE}
    msctfime.rc
    ${CMAKE_CURRENT_BINARY_DIR}/msctfime.def)
add_dependencies(msctfime rust_msctfime_rust)
target_link_libraries(msctfime ${msctfime_rust_LIB})
set_module_type(msctfime win32dll UNICODE)
set_target_properties(msctfime PROPERTIES SUFFIX ".ime")
target_link_libraries(msctfime cpprt wine uuid cicero)
add_importlibs(msctfime user32 gdi32 advapi32 msvcrt kernel32 ntdll)
add_delay_importlibs(msctfime uxtheme shlwapi comctl32 msctf oleaut32 imm32)
add_cd_file(TARGET msctfime DESTINATION reactos/system32 FOR all)
