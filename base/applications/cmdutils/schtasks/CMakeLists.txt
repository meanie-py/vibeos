
# Every translation unit is Rust now (schtasks-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(schtasks_rust schtasks-rust "" TRUE)
add_executable(schtasks
    # schtasks.c  # replaced by Rust (sdk/lib/rust/schtasks-rust/src/schtasks/schtasks.rs)
    rust_cohab_stub.c
    )
add_dependencies(schtasks rust_schtasks_rust)
target_link_libraries(schtasks ${schtasks_rust_LIB})
target_link_libraries(schtasks wine)
set_module_type(schtasks win32cui UNICODE)
add_importlibs(schtasks oleaut32 ole32 msvcrt kernel32 ntdll)
add_cd_file(TARGET schtasks DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
