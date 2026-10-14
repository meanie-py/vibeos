
list(APPEND SOURCE
    # cmds.c  # replaced by Rust (sdk/lib/rust/ftp-rust/src/ftp/cmds.rs)
    # cmdtab.c  # replaced by Rust (sdk/lib/rust/ftp-rust/src/ftp/cmdtab.rs)
    # domacro.c  # replaced by Rust (sdk/lib/rust/ftp-rust/src/ftp/domacro.rs)
    # fake.c  # replaced by Rust (sdk/lib/rust/ftp-rust/src/ftp/fake.rs)
    # ftp.c  # replaced by Rust (sdk/lib/rust/ftp-rust/src/ftp/ftp.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/ftp-rust/src/ftp/main.rs)
    # ruserpass.c  # replaced by Rust (sdk/lib/rust/ftp-rust/src/ftp/ruserpass.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    precomp.h)

# Every translation unit is Rust now (ftp-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(ftp_rust ftp-rust "" TRUE)
add_executable(ftp ${SOURCE} ftp.rc)
add_dependencies(ftp rust_ftp_rust)
target_link_libraries(ftp ${ftp_rust_LIB})
target_compile_definitions(ftp PRIVATE lint)

set_module_type(ftp win32cui)
target_link_libraries(ftp oldnames ${PSEH_LIB})
add_importlibs(ftp ws2_32 iphlpapi msvcrt kernel32)
add_pch(ftp precomp.h SOURCE)

add_cd_file(TARGET ftp DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
