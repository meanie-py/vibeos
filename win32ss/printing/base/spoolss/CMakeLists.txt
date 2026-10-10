
spec2def(spoolss.dll spoolss.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # ../marshalling.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/marshalling.rs)
    # context.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/context.rs)
    # forms.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/forms.rs)
    # jobs.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/jobs.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/main.rs)
    # memory.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/memory.rs)
    # monitors.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/monitors.rs)
    # ports.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/ports.rs)
    # printerdata.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/printerdata.rs)
    # printerdrivers.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/printerdrivers.rs)
    # printers.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/printers.rs)
    # printprocessors.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/printprocessors.rs)
    # printproviders.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/printproviders.rs)
    # spoolfile.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/spoolfile.rs)
    # tools.c  # replaced by Rust (sdk/lib/rust/spoolss-rust/src/spoolss/tools.rs)
    rust_cohab_stub.c
    )

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/spoolss_stubs.c)

# Every translation unit is Rust now (spoolss-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(spoolss_rust spoolss-rust "" TRUE)
add_library(spoolss MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    spoolss.rc
    ${CMAKE_CURRENT_BINARY_DIR}/spoolss.def)
add_dependencies(spoolss rust_spoolss_rust)
target_link_libraries(spoolss ${spoolss_rust_LIB})

set_module_type(spoolss win32dll UNICODE)
target_link_libraries(spoolss wine)
add_importlibs(spoolss advapi32 msvcrt kernel32 ntdll)
add_pch(spoolss precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET spoolss DESTINATION reactos/system32 FOR all)
