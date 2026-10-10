
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
add_rpc_files(server ${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl/winspool.idl)

spec2def(spoolsv.exe spoolsv.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # forms.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/forms.rs)
    # init.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/init.rs)
    # jobs.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/jobs.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/main.rs)
    # monitors.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/monitors.rs)
    # notifications.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/notifications.rs)
    # ports.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/ports.rs)
    precomp.h
    # printerdata.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/printerdata.rs)
    # printerdrivers.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/printerdrivers.rs)
    # printers.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/printers.rs)
    # printprocessors.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/printprocessors.rs)
    # printproviders.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/printproviders.rs)
    # rpcserver.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/rpcserver.rs)
    # rpcstubs.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/rpcstubs.rs)
    # spoolfile.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/spoolfile.rs)
    # xcv.c  # replaced by Rust (sdk/lib/rust/spoolsv-rust/src/spoolsv/xcv.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/winspool_s.c
    ${CMAKE_CURRENT_BINARY_DIR}/spoolsv_stubs.c)

# Every translation unit is Rust now (spoolsv-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(spoolsv_rust spoolsv-rust "" TRUE)
add_executable(spoolsv
    ${SOURCE}
    spoolsv.rc
    ${CMAKE_CURRENT_BINARY_DIR}/spoolsv.def)
add_dependencies(spoolsv rust_spoolsv_rust)
target_link_libraries(spoolsv ${spoolsv_rust_LIB})

set_target_properties(spoolsv
    PROPERTIES
    ENABLE_EXPORTS TRUE
    DEFINE_SYMBOL "")

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(spoolsv PRIVATE "-Wno-cast-calling-convention")
endif()

set_module_type(spoolsv win32cui UNICODE)
target_link_libraries(spoolsv wine)
add_delay_importlibs(spoolsv spoolss)
add_importlibs(spoolsv advapi32 rpcrt4 msvcrt kernel32 ntdll)
add_pch(spoolsv precomp.h SOURCE)
add_cd_file(TARGET spoolsv DESTINATION reactos/system32 FOR all)
