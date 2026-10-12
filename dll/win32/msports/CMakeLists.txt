
spec2def(msports.dll msports.spec)

list(APPEND SOURCE
    # classinst.c  # replaced by Rust (sdk/lib/rust/msports-rust/src/msports/classinst.rs)
    # comdb.c  # replaced by Rust (sdk/lib/rust/msports-rust/src/msports/comdb.rs)
    # msports.c  # replaced by Rust (sdk/lib/rust/msports-rust/src/msports/msports.rs)
    # parallel.c  # replaced by Rust (sdk/lib/rust/msports-rust/src/msports/parallel.rs)
    # serial.c  # replaced by Rust (sdk/lib/rust/msports-rust/src/msports/serial.rs)
    rust_cohab_stub.c
    precomp.h
    ${CMAKE_CURRENT_BINARY_DIR}/msports_stubs.c)

# Every translation unit is Rust now (msports-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(msports_rust msports-rust "" TRUE)
add_library(msports MODULE
    ${SOURCE}
    msports.rc
    ${CMAKE_CURRENT_BINARY_DIR}/msports.def)
add_dependencies(msports rust_msports_rust)
target_link_libraries(msports ${msports_rust_LIB})

set_module_type(msports win32dll UNICODE)
target_link_libraries(msports wine)
add_importlibs(msports setupapi comctl32 user32 advapi32 msvcrt kernel32 ntdll)
add_pch(msports precomp.h SOURCE)
add_cd_file(TARGET msports DESTINATION reactos/system32 FOR all)
