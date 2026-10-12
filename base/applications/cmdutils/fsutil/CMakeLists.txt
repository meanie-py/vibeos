list(APPEND SOURCE
    # common.c  # replaced by Rust (sdk/lib/rust/fsutil-rust/src/fsutil/common.rs)
    # dirty.c  # replaced by Rust (sdk/lib/rust/fsutil-rust/src/fsutil/dirty.rs)
    # fsinfo.c  # replaced by Rust (sdk/lib/rust/fsutil-rust/src/fsutil/fsinfo.rs)
    # fsutil.c  # replaced by Rust (sdk/lib/rust/fsutil-rust/src/fsutil/fsutil.rs)
    # hardlink.c  # replaced by Rust (sdk/lib/rust/fsutil-rust/src/fsutil/hardlink.rs)
    # volume.c  # replaced by Rust (sdk/lib/rust/fsutil-rust/src/fsutil/volume.rs)
    rust_cohab_stub.c
    fsutil.h)
# Every translation unit is Rust now (fsutil-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(fsutil_rust fsutil-rust "" TRUE)
add_executable(fsutil ${SOURCE} fsutil.rc)
add_dependencies(fsutil rust_fsutil_rust)
target_link_libraries(fsutil ${fsutil_rust_LIB})
set_module_type(fsutil win32cui UNICODE)
add_importlibs(fsutil msvcrt kernel32 ntdll)
add_pch(fsutil fsutil.h SOURCE)
add_cd_file(TARGET fsutil DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
