
list(APPEND SOURCE
    # eventvwr.c  # replaced by Rust (sdk/lib/rust/eventvwr-rust/src/eventvwr/eventvwr.rs)
    # evtdetctl.c  # replaced by Rust (sdk/lib/rust/eventvwr-rust/src/eventvwr/evtdetctl.rs)
    rust_cohab_stub.c
    eventvwr.h)

file(GLOB eventvwr_rc_deps res/*.*)
add_rc_deps(eventvwr.rc ${eventvwr_rc_deps})
# Every translation unit is Rust now (eventvwr-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(eventvwr_rust eventvwr-rust "" TRUE)
add_executable(eventvwr ${SOURCE} eventvwr.rc)
add_dependencies(eventvwr rust_eventvwr_rust)
target_link_libraries(eventvwr ${eventvwr_rust_LIB})
add_pch(eventvwr eventvwr.h SOURCE)
set_module_type(eventvwr win32gui UNICODE)
target_link_libraries(eventvwr ${PSEH_LIB})
add_importlibs(eventvwr user32 gdi32 comctl32 comdlg32 advapi32 shell32 shlwapi msvcrt kernel32 ntdll)

add_cd_file(TARGET eventvwr DESTINATION reactos/system32 FOR all)
