
# Every translation unit is Rust now (whois-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(whois_rust whois-rust "" TRUE)
add_executable(whois
    # whois.c  # replaced by Rust (sdk/lib/rust/whois-rust/src/whois/whois.rs)
    rust_cohab_stub.c
    whois.rc)
add_dependencies(whois rust_whois_rust)
target_link_libraries(whois ${whois_rust_LIB})
set_module_type(whois win32cui)
add_importlibs(whois ws2_32 msvcrt kernel32)
add_cd_file(TARGET whois DESTINATION reactos/system32 FOR all)
