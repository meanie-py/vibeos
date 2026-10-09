
spec2def(powercfg.cpl powercfg.spec)

list(APPEND SOURCE
    # powercfg.c  # replaced by Rust (sdk/lib/rust/powercfg-rust/src/powercfg/powercfg.rs)
    # powershemes.c  # replaced by Rust (sdk/lib/rust/powercfg-rust/src/powercfg/powershemes.rs)
    # alarms.c  # replaced by Rust (sdk/lib/rust/powercfg-rust/src/powercfg/alarms.rs)
    # advanced.c  # replaced by Rust (sdk/lib/rust/powercfg-rust/src/powercfg/advanced.rs)
    # hibernate.c  # replaced by Rust (sdk/lib/rust/powercfg-rust/src/powercfg/hibernate.rs)
    # powermeter.c  # replaced by Rust (sdk/lib/rust/powercfg-rust/src/powercfg/powermeter.rs)
    rust_cohab_stub.c
    powercfg.h)

file(GLOB powercfg_rc_deps resources/*.*)
add_rc_deps(powercfg.rc ${powercfg_rc_deps})

# Every translation unit is Rust now (powercfg-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(powercfg_rust powercfg-rust "" TRUE)
add_library(powercfg MODULE
    ${SOURCE}
    powercfg.rc
    ${CMAKE_CURRENT_BINARY_DIR}/powercfg.def)
add_dependencies(powercfg rust_powercfg_rust)
target_link_libraries(powercfg ${powercfg_rust_LIB})

set_module_type(powercfg cpl UNICODE)
add_importlibs(powercfg shlwapi user32 powrprof comctl32 shell32 advapi32 msvcrt setupapi kernel32 ntdll)
add_pch(powercfg powercfg.h SOURCE)
add_cd_file(TARGET powercfg DESTINATION reactos/system32 FOR all)
