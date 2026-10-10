
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(client ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/winspool.idl)
spec2def(winspool.drv winspool.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # ../marshalling.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/marshalling.rs)
    # devmode.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/devmode.rs)
    # forms.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/forms.rs)
    # jobs.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/jobs.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/main.rs)
    # monitors.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/monitors.rs)
    # ports.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/ports.rs)
    # printerdata.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/printerdata.rs)
    # printerdrivers.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/printerdrivers.rs)
    # printers.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/printers.rs)
    # printprocessors.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/printprocessors.rs)
    # printproviders.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/printproviders.rs)
    # spoolfile.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/spoolfile.rs)
    # utils.c  # replaced by Rust (sdk/lib/rust/winspool-rust/src/winspool/utils.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/winspool_c.c)

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/winspool_stubs.c)

# Every translation unit is Rust now (winspool-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(winspool_rust winspool-rust "" TRUE)
add_library(winspool MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    winspool.rc
    ${CMAKE_CURRENT_BINARY_DIR}/winspool.def)
add_dependencies(winspool rust_winspool_rust)
target_link_libraries(winspool ${winspool_rust_LIB})

set_target_properties(winspool PROPERTIES SUFFIX ".drv")
set_module_type(winspool win32dll UNICODE)
target_link_libraries(winspool wine ${PSEH_LIB})
add_importlibs(winspool advapi32 gdi32 user32 rpcrt4 msvcrt kernel32 ntdll)
add_pch(winspool precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET winspool DESTINATION reactos/system32 FOR all)
