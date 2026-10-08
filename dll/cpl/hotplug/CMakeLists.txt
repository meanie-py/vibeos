
spec2def(hotplug.dll hotplug.spec)

list(APPEND SOURCE
    # hotplug.c  # replaced by Rust (sdk/lib/rust/hotplug-rust/src/hotplug/hotplug.rs)
    # eject.c  # replaced by Rust (sdk/lib/rust/hotplug-rust/src/hotplug/eject.rs)
    # enum.c  # replaced by Rust (sdk/lib/rust/hotplug-rust/src/hotplug/enum.rs)
    rust_cohab_stub.c
    )

file(GLOB hotplug_rc_deps resources/*.*)
add_rc_deps(hotplug.rc ${hotplug_rc_deps})

# Every translation unit is Rust now (hotplug-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(hotplug_rust hotplug-rust "" TRUE)
add_library(hotplug MODULE
    ${SOURCE}
    hotplug.rc
    ${CMAKE_CURRENT_BINARY_DIR}/hotplug.def)
add_dependencies(hotplug rust_hotplug_rust)
target_link_libraries(hotplug ${hotplug_rust_LIB})

set_module_type(hotplug win32dll UNICODE)
add_importlibs(hotplug user32 gdi32 advapi32 setupapi comctl32 msvcrt kernel32 ntdll)
add_cd_file(TARGET hotplug DESTINATION reactos/system32 FOR all)
