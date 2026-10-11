
add_definitions(
    -Dstrcasecmp=_stricmp
)

list(APPEND SOURCE
    # callback.c  # replaced by Rust (sdk/lib/rust/winhlp32-rust/src/winhlp32/callback.rs)
    # hlpfile.c  # replaced by Rust (sdk/lib/rust/winhlp32-rust/src/winhlp32/hlpfile.rs)
    # macro.c  # replaced by Rust (sdk/lib/rust/winhlp32-rust/src/winhlp32/macro.rs)
    # winhelp.c  # replaced by Rust (sdk/lib/rust/winhlp32-rust/src/winhlp32/winhelp.rs)
    )

FLEX_TARGET(macro_scanner macro.lex.l ${CMAKE_CURRENT_BINARY_DIR}/macro.lex.yy.c)

list(APPEND PCH_SKIP_SOURCE
    ${FLEX_macro_scanner_OUTPUTS}
    # string.c  # replaced by Rust (sdk/lib/rust/winhlp32-rust/src/winhlp32/string/mod.rs, generated: port data)
    rust_cohab_stub.c
    )

add_rc_deps(rsrc.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/winhelp.ico)

# Every translation unit is Rust now (winhlp32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(winhlp32_rust winhlp32-rust "" TRUE)
add_executable(winhlp32
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    rsrc.rc)
add_dependencies(winhlp32 rust_winhlp32_rust)
target_link_libraries(winhlp32 ${winhlp32_rust_LIB})

set_module_type(winhlp32 win32gui)
target_link_libraries(winhlp32 wine oldnames)
add_importlibs(winhlp32 user32 gdi32 shell32 comctl32 comdlg32 msvcrt kernel32 ntdll)
add_pch(winhlp32 precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET winhlp32 DESTINATION reactos FOR all)
set_wine_module_FIXME(winhlp32) # CORE-5743: No ARRAY_SIZE macro
