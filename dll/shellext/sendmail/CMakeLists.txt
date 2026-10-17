spec2def(sendmail.dll sendmail.spec)

add_definitions(
    -D_WINE)

file(GLOB_RECURSE sendmail_rc_deps res/*.*)
add_rc_deps(sendmail.rc ${sendmail_rc_deps})

list(APPEND SOURCE
    # CDeskLinkDropHandler.cpp  # replaced by Rust (sdk/lib/rust/sendmail-rust/src/sendmail/CDeskLinkDropHandler_cpp_h.rs)
    # sendmail.cpp  # replaced by Rust (sdk/lib/rust/sendmail-rust/src/sendmail/sendmail_cpp_h.rs)
    rust_cohab_stub.cpp
    precomp.hpp)

# Every translation unit is Rust now (sendmail-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(sendmail_rust sendmail-rust "" TRUE)
add_library(sendmail MODULE
    ${SOURCE}
    sendmail.rc
    ${CMAKE_CURRENT_BINARY_DIR}/sendmail.def)
add_dependencies(sendmail rust_sendmail_rust)
target_link_libraries(sendmail ${sendmail_rust_LIB})

set_module_type(sendmail win32dll UNICODE)
target_link_libraries(sendmail uuid wine cpprt atl_classes)
add_delay_importlibs(sendmail ole32 oleaut32)
add_importlibs(sendmail advapi32 shell32 user32 comctl32 shlwapi msvcrt kernel32 ntdll)
add_pch(sendmail precomp.hpp SOURCE)
add_cd_file(TARGET sendmail DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
