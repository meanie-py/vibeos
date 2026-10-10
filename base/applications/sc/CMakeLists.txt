
add_definitions(-DDEFINE_GUID)

list(APPEND SOURCE
    # config.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/config.rs)
    # control.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/control.rs)
    # create.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/create.rs)
    # delete.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/delete.rs)
    # depend.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/depend.rs)
    # description.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/description.rs)
    # failure.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/failure.rs)
    # misc.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/misc.rs)
    # name.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/name.rs)
    # print.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/print.rs)
    # query.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/query.rs)
    # sc.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/sc.rs)
    # sdset.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/sdset.rs)
    # sdshow.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/sdshow.rs)
    # start.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/start.rs)
    # usage.c  # replaced by Rust (sdk/lib/rust/sc-rust/src/sc/usage.rs)
    rust_cohab_stub.c
    sc.h)

# Every translation unit is Rust now (sc-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(sc_rust sc-rust "" TRUE)
add_executable(sc ${SOURCE} sc.rc)
add_dependencies(sc rust_sc_rust)
target_link_libraries(sc ${sc_rust_LIB})
set_module_type(sc win32cui UNICODE)
add_importlibs(sc advapi32 msvcrt kernel32 ntdll)
add_pch(sc sc.h SOURCE)
add_cd_file(TARGET sc DESTINATION reactos/system32 FOR all)
