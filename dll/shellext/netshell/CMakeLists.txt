
project(SHELL)

remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

add_definitions(-D_NETSHELL_)
spec2def(netshell.dll netshell.spec)

list(APPEND SOURCE
    # netshell.cpp  # replaced by Rust (sdk/lib/rust/netshell-rust/src/netshell/netshell_cpp_h.rs)
    # shfldr_netconnect.cpp  # replaced by Rust (sdk/lib/rust/netshell-rust/src/netshell/shfldr_netconnect_cpp_h.rs)
    # enumlist.cpp  # replaced by Rust (sdk/lib/rust/netshell-rust/src/netshell/enumlist_cpp_h.rs)
    # connectmanager.cpp  # replaced by Rust (sdk/lib/rust/netshell-rust/src/netshell/connectmanager_cpp_h.rs)
    # lanconnectui.cpp  # replaced by Rust (sdk/lib/rust/netshell-rust/src/netshell/lanconnectui_cpp_h.rs)
    # lanstatusui.cpp  # replaced by Rust (sdk/lib/rust/netshell-rust/src/netshell/lanstatusui_cpp_h.rs)
    # setup.cpp  # replaced by Rust (sdk/lib/rust/netshell-rust/src/netshell/setup_cpp_h.rs)
    rust_cohab_stub.cpp
    )

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/netshell_stubs.c)

file(GLOB netshell_rc_deps res/*.*)
add_rc_deps(netshell.rc ${netshell_rc_deps})

# Every translation unit is Rust now (netshell-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(netshell_rust netshell-rust "" TRUE)
add_library(netshell MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    netshell.rc
    ${CMAKE_CURRENT_BINARY_DIR}/netshell.def)
add_dependencies(netshell rust_netshell_rust)
target_link_libraries(netshell ${netshell_rust_LIB})

set_module_type(netshell win32dll UNICODE)
target_link_libraries(netshell uuid wine cpprt atl_classes)
add_delay_importlibs(netshell ole32 oleaut32 shlwapi shell32)
add_importlibs(netshell version iphlpapi gdi32 user32 advapi32 setupapi ws2_32 comctl32 msvcrt kernel32 ntdll)
add_pch(netshell precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET netshell DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
