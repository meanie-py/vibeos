
include_directories(BEFORE ${REACTOS_SOURCE_DIR}/sdk/include/wine)
spec2def(usrmgr.cpl usrmgr.spec)

list(APPEND SOURCE
    # extra.c  # replaced by Rust (sdk/lib/rust/usrmgr-rust/src/usrmgr/extra.rs)
    # groupprops.c  # replaced by Rust (sdk/lib/rust/usrmgr-rust/src/usrmgr/groupprops.rs)
    # groups.c  # replaced by Rust (sdk/lib/rust/usrmgr-rust/src/usrmgr/groups.rs)
    # misc.c  # replaced by Rust (sdk/lib/rust/usrmgr-rust/src/usrmgr/misc.rs)
    # userprops.c  # replaced by Rust (sdk/lib/rust/usrmgr-rust/src/usrmgr/userprops.rs)
    # users.c  # replaced by Rust (sdk/lib/rust/usrmgr-rust/src/usrmgr/users.rs)
    # usrmgr.c  # replaced by Rust (sdk/lib/rust/usrmgr-rust/src/usrmgr/usrmgr.rs)
    rust_cohab_stub.c
    usrmgr.h)

file(GLOB usrmgr_rc_deps resources/*.*)
add_rc_deps(usrmgr.rc ${usrmgr_rc_deps})

# Every translation unit is Rust now (usrmgr-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(usrmgr_rust usrmgr-rust "" TRUE)
add_library(usrmgr MODULE
    ${SOURCE}
    usrmgr.rc
    ${CMAKE_CURRENT_BINARY_DIR}/usrmgr.def)
add_dependencies(usrmgr rust_usrmgr_rust)
target_link_libraries(usrmgr ${usrmgr_rust_LIB})

set_module_type(usrmgr cpl UNICODE)
target_link_libraries(usrmgr wine)
add_importlibs(usrmgr advapi32 user32 gdi32 comctl32 netapi32 msvcrt kernel32 ntdll)
add_pch(usrmgr usrmgr.h SOURCE)
add_cd_file(TARGET usrmgr DESTINATION reactos/system32 FOR all)
