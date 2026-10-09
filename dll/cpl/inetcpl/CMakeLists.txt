
spec2def(inetcpl.cpl inetcpl.spec)

list(APPEND SOURCE
    # connections.c  # replaced by Rust (sdk/lib/rust/inetcpl-rust/src/inetcpl/connections.rs)
    # content.c  # replaced by Rust (sdk/lib/rust/inetcpl-rust/src/inetcpl/content.rs)
    # general.c  # replaced by Rust (sdk/lib/rust/inetcpl-rust/src/inetcpl/general.rs)
    # inetcpl.c  # replaced by Rust (sdk/lib/rust/inetcpl-rust/src/inetcpl/inetcpl.rs)
    # security.c  # replaced by Rust (sdk/lib/rust/inetcpl-rust/src/inetcpl/security.rs)
    rust_cohab_stub.c
    precomp.h)

file(GLOB inetcpl_rc_deps resources/*.*)
add_rc_deps(inetcpl.rc ${inetcpl_rc_deps})

# Every translation unit is Rust now (inetcpl-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(inetcpl_rust inetcpl-rust "" TRUE)
add_library(inetcpl MODULE
    ${SOURCE}
    inetcpl.rc
    ${CMAKE_CURRENT_BINARY_DIR}/inetcpl_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/inetcpl.def)
add_dependencies(inetcpl rust_inetcpl_rust)
target_link_libraries(inetcpl ${inetcpl_rust_LIB})

set_module_type(inetcpl cpl UNICODE)
target_link_libraries(inetcpl wine)
add_delay_importlibs(inetcpl cryptui wininet ole32 urlmon shell32)
add_importlibs(inetcpl advapi32 comctl32 user32 shlwapi msvcrt kernel32 ntdll)
add_pch(inetcpl precomp.h SOURCE)
add_cd_file(TARGET inetcpl DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(inetcpl) # CORE-5743: No ARRAY_SIZE macro
