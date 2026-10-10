
add_subdirectory(ui)

spec2def(localmon.dll localmon.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # main.c  # replaced by Rust (sdk/lib/rust/localmon-rust/src/localmon/main.rs)
    # ports.c  # replaced by Rust (sdk/lib/rust/localmon-rust/src/localmon/ports.rs)
    precomp.h
    # tools.c  # replaced by Rust (sdk/lib/rust/localmon-rust/src/localmon/tools.rs)
    # xcv.c  # replaced by Rust (sdk/lib/rust/localmon-rust/src/localmon/xcv.rs)
    rust_cohab_stub.c
    )

# Every translation unit is Rust now (localmon-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(localmon_rust localmon-rust "" TRUE)
add_library(localmon MODULE
    ${SOURCE}
    localmon.rc
    ${CMAKE_CURRENT_BINARY_DIR}/localmon.def)
add_dependencies(localmon rust_localmon_rust)
target_link_libraries(localmon ${localmon_rust_LIB})

set_module_type(localmon win32dll UNICODE)
target_link_libraries(localmon wine)
add_importlibs(localmon advapi32 spoolss user32 msvcrt kernel32 ntdll)
add_pch(localmon precomp.h SOURCE)
add_cd_file(TARGET localmon DESTINATION reactos/system32 FOR all)
