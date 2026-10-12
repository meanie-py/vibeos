
remove_definitions(-D_WIN32_IE=0x600)

list(APPEND SOURCE
    # olecallback.c  # replaced by Rust (sdk/lib/rust/wordpad-rust/src/wordpad/olecallback.rs)
    # print.c  # replaced by Rust (sdk/lib/rust/wordpad-rust/src/wordpad/print.rs)
    # registry.c  # replaced by Rust (sdk/lib/rust/wordpad-rust/src/wordpad/registry.rs)
    # wordpad.c  # replaced by Rust (sdk/lib/rust/wordpad-rust/src/wordpad/wordpad.rs)
    rust_cohab_stub.c
    )

file(GLOB wordpad_rc_deps res/*.*)
add_rc_deps(rsrc.rc ${wordpad_rc_deps})
# wordpad-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(WORDPAD_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(WORDPAD_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (wordpad-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wordpad_rust wordpad-rust "" TRUE ${WORDPAD_RUST_ENV})
add_executable(wordpad ${SOURCE} rsrc.rc)
add_dependencies(wordpad rust_wordpad_rust)
target_link_libraries(wordpad ${wordpad_rust_LIB})
set_module_type(wordpad win32gui)
target_link_libraries(wordpad wine uuid)
add_importlibs(wordpad comdlg32 ole32 shell32 user32 gdi32 advapi32 comctl32 msvcrt kernel32 ntdll)
add_cd_file(TARGET wordpad DESTINATION reactos/system32 FOR all)
