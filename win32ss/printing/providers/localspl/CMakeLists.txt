remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

spec2def(localspl.dll localspl.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # forms.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/forms.rs)
    # jobs.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/jobs.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/main.rs)
    # monitors.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/monitors.rs)
    # ports.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/ports.rs)
    precomp.h
    # printerdata.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/printerdata.rs)
    # printerdrivers.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/printerdrivers.rs)
    # printers.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/printers.rs)
    # printingthread.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/printingthread.rs)
    # printprocessors.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/printprocessors.rs)
    # spoolfile.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/spoolfile.rs)
    # tools.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/tools.rs)
    # xcv.c  # replaced by Rust (sdk/lib/rust/localspl-rust/src/localspl/xcv.rs)
    rust_cohab_stub.c
    )

# Every translation unit is Rust now (localspl-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(localspl_rust localspl-rust "" TRUE)
add_library(localspl MODULE
    ${SOURCE}
    localspl.rc
    ${CMAKE_CURRENT_BINARY_DIR}/localspl_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/localspl.def)
add_dependencies(localspl rust_localspl_rust)
target_link_libraries(localspl ${localspl_rust_LIB})
add_pch(localspl precomp.h SOURCE)

set_module_type(localspl win32dll UNICODE)
target_link_libraries(localspl skiplist16 wine)
add_importlibs(localspl user32 advapi32 advapi32_vista netapi32 rpcrt4 secur32 spoolss msvcrt kernel32 ntdll)
add_cd_file(TARGET localspl DESTINATION reactos/system32 FOR all)
