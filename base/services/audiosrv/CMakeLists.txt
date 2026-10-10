
list(APPEND SOURCE
    # main.c  # replaced by Rust (sdk/lib/rust/audiosrv-rust/src/audiosrv/main.rs)
    # pnp_list_manager.c  # replaced by Rust (sdk/lib/rust/audiosrv-rust/src/audiosrv/pnp_list_manager.rs)
    # pnp_list_lock.c  # replaced by Rust (sdk/lib/rust/audiosrv-rust/src/audiosrv/pnp_list_lock.rs)
    # pnp.c  # replaced by Rust (sdk/lib/rust/audiosrv-rust/src/audiosrv/pnp.rs)
    # services.c  # replaced by Rust (sdk/lib/rust/audiosrv-rust/src/audiosrv/services.rs)
    rust_cohab_stub.c
    audiosrv.h)

# Every translation unit is Rust now (audiosrv-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(audiosrv_rust audiosrv-rust "" TRUE)
add_executable(audiosrv ${SOURCE} audiosrv.rc)
add_dependencies(audiosrv rust_audiosrv_rust)
target_link_libraries(audiosrv ${audiosrv_rust_LIB})
set_module_type(audiosrv win32cui UNICODE)
add_importlibs(audiosrv advapi32 user32 setupapi msvcrt kernel32 ntdll)
add_pch(audiosrv audiosrv.h SOURCE)
add_cd_file(TARGET audiosrv DESTINATION reactos/system32 FOR all)
