
add_definitions(${I18N_DEFS})

include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers)

list(APPEND SOURCE
    # spapisup/cabinet.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/cabinet.rs)
    # spapisup/fileqsup.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/fileqsup.rs)
    # spapisup/infsupp.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/infsupp.rs)
    # cmdcons.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/cmdcons.rs)
    # console.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/console.rs)
    # consup.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/consup.rs)
    # devinst.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/devinst.rs)
    # fmtchk.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/fmtchk.rs)
    # fslist.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/fslist.rs)
    # genlist.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/genlist.rs)
    # keytrans.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/keytrans.rs)
    # mui.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/mui.rs)
    # partlist.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/partlist.rs)
    # progress.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/progress.rs)
    # usetup.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/usetup.rs)
    rust_cohab_stub.c
    # mui_tables.c  # replaced by Rust (sdk/lib/rust/usetup-rust/src/usetup/mui_tables/mod.rs, generated: port data)
    usetup.h)

# Every translation unit is Rust now (usetup-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(usetup_rust usetup-rust "" TRUE)
add_executable(usetup ${SOURCE} usetup.rc)
add_dependencies(usetup rust_usetup_rust)
target_link_libraries(usetup ${usetup_rust_LIB})
add_pch(usetup usetup.h SOURCE)

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(usetup PRIVATE "-Wno-invalid-source-encoding")
endif()

set_module_type(usetup nativecui)
target_link_libraries(usetup inflib zlib_solo chkstk)
add_importlibs(usetup setuplib ntdll)
add_cd_file(TARGET usetup DESTINATION reactos/system32 NO_CAB NAME_ON_CD smss.exe FOR bootcd regtest)
