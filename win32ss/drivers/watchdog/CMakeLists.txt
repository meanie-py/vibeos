
spec2def(watchdog.sys watchdog.spec)

# Every translation unit is Rust now (watchdog-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(watchdog_rust watchdog-rust "" TRUE)
add_library(watchdog MODULE
    # watchdog.c  # replaced by Rust (sdk/lib/rust/watchdog-rust/src/watchdog/watchdog.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/watchdog.def)
add_dependencies(watchdog rust_watchdog_rust)
target_link_libraries(watchdog ${watchdog_rust_LIB})

set_module_type(watchdog kernelmodedriver)
add_importlibs(watchdog ntoskrnl)
add_cd_file(TARGET watchdog DESTINATION reactos/system32/drivers FOR all)
