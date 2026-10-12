remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x602)

add_definitions(-DUNICODE -D_UNICODE -D_NTOSKRNL_ -D_NTSYSTEM_)

include_directories(${REACTOS_SOURCE_DIR}/sdk/lib/rtl)

# Every translation unit is Rust now (ntoskrnl_vista-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archives. utf8.c is rtl's: it is the
# `utf8` crate rtl_vista already links, not a second translation.
list(APPEND SOURCE
    # etw.c  # replaced by Rust (sdk/lib/rust/ntoskrnl_vista-rust/src/ntoskrnl_vista/etw.rs)
    # fsrtl.c  # replaced by Rust (sdk/lib/rust/ntoskrnl_vista-rust/src/ntoskrnl_vista/fsrtl.rs)
    # io.c  # replaced by Rust (sdk/lib/rust/ntoskrnl_vista-rust/src/ntoskrnl_vista/io.rs)
    # po.c  # replaced by Rust (sdk/lib/rust/ntoskrnl_vista-rust/src/ntoskrnl_vista/po.rs)
    # pofx.c  # replaced by Rust (sdk/lib/rust/ntoskrnl_vista-rust/src/ntoskrnl_vista/pofx.rs)
    # ke.c  # replaced by Rust (sdk/lib/rust/ntoskrnl_vista-rust/src/ntoskrnl_vista/ke.rs)
    # ${REACTOS_SOURCE_DIR}/sdk/lib/rtl/utf8.c  # replaced by Rust (sdk/lib/rust/utf8)
    rust_cohab_stub.c)

rust_staticlib_variant(ntoskrnl_vista_rust ntoskrnl_vista-rust "" TRUE)
rust_staticlib_variant(ntoskrnl_vista_utf8 utf8 "" TRUE)
add_library(ntoskrnl_vista ${SOURCE})
add_dependencies(ntoskrnl_vista rust_ntoskrnl_vista_rust rust_ntoskrnl_vista_utf8)
target_link_libraries(ntoskrnl_vista PRIVATE ${ntoskrnl_vista_rust_LIB} ${ntoskrnl_vista_utf8_LIB})
target_link_libraries(ntoskrnl_vista PRIVATE pseh) # rtl_vista)
add_dependencies(ntoskrnl_vista bugcodes xdk)

target_compile_definitions(ntoskrnl_vista PUBLIC NTKRNLVISTA)
