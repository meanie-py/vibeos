
list(APPEND SOURCE
    # nslookup.c  # replaced by Rust (sdk/lib/rust/nslookup-rust/src/nslookup/nslookup.rs)
    # utility.c  # replaced by Rust (sdk/lib/rust/nslookup-rust/src/nslookup/utility.rs)
    rust_cohab_stub.c
    nslookup.h)

# Every translation unit is Rust now (nslookup-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(nslookup_rust nslookup-rust "" TRUE)
add_executable(nslookup ${SOURCE} nslookup.rc)
add_dependencies(nslookup rust_nslookup_rust)
target_link_libraries(nslookup ${nslookup_rust_LIB})
set_module_type(nslookup win32cui)
add_importlibs(nslookup user32 ws2_32 snmpapi iphlpapi msvcrt kernel32)
add_pch(nslookup nslookup.h SOURCE)
add_cd_file(TARGET nslookup DESTINATION reactos/system32 FOR all)
