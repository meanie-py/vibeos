
spec2def(fmifs.dll fmifs.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # chkdsk.c  # replaced by Rust (sdk/lib/rust/fmifs-rust/src/fmifs/chkdsk.rs)
    # compress.c  # replaced by Rust (sdk/lib/rust/fmifs-rust/src/fmifs/compress.rs)
    # diskcopy.c  # replaced by Rust (sdk/lib/rust/fmifs-rust/src/fmifs/diskcopy.rs)
    # extend.c  # replaced by Rust (sdk/lib/rust/fmifs-rust/src/fmifs/extend.rs)
    # format.c  # replaced by Rust (sdk/lib/rust/fmifs-rust/src/fmifs/format.rs)
    # init.c  # replaced by Rust (sdk/lib/rust/fmifs-rust/src/fmifs/init.rs)
    # media.c  # replaced by Rust (sdk/lib/rust/fmifs-rust/src/fmifs/media.rs)
    # query.c  # replaced by Rust (sdk/lib/rust/fmifs-rust/src/fmifs/query.rs)
    rust_cohab_stub.c
    )

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/fmifs_stubs.c)

# Every translation unit is Rust now (fmifs-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(fmifs_rust fmifs-rust "" TRUE)
add_library(fmifs MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    fmifs.rc
    ${CMAKE_CURRENT_BINARY_DIR}/fmifs.def)
add_dependencies(fmifs rust_fmifs_rust)
target_link_libraries(fmifs ${fmifs_rust_LIB})

set_module_type(fmifs win32dll UNICODE ENTRYPOINT InitializeFmIfs 12)
add_importlibs(fmifs kernel32 ntdll)
add_pch(fmifs precomp.h "${PCH_SKIP_SOURCE}")
add_dependencies(fmifs psdk)
add_cd_file(TARGET fmifs DESTINATION reactos/system32 FOR all)
