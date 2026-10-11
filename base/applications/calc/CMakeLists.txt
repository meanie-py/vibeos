
list(APPEND SOURCE
    # convert.c  # replaced by Rust (sdk/lib/rust/calc-rust/src/calc/convert.rs)
    # fun_ieee.c  # replaced by Rust (sdk/lib/rust/calc-rust/src/calc/fun_ieee.rs)
    # rpn_ieee.c  # replaced by Rust (sdk/lib/rust/calc-rust/src/calc/rpn_ieee.rs)
    # utl_ieee.c  # replaced by Rust (sdk/lib/rust/calc-rust/src/calc/utl_ieee.rs)
    # winmain.c  # replaced by Rust (sdk/lib/rust/calc-rust/src/calc/winmain.rs)
    # htmlhelp.c  # replaced by Rust (sdk/lib/rust/calc-rust/src/calc/htmlhelp.rs)
    # theme.c  # replaced by Rust (sdk/lib/rust/calc-rust/src/calc/theme.rs)
    rust_cohab_stub.c
    calc.h)

file(GLOB calc_rc_deps res/*.*)
add_rc_deps(resource.rc ${calc_rc_deps})
# calc-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(CALC_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(CALC_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (calc-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(calc_rust calc-rust "" TRUE ${CALC_RUST_ENV})
add_executable(calc ${SOURCE} resource.rc)
add_dependencies(calc rust_calc_rust)
target_link_libraries(calc ${calc_rust_LIB})
set_module_type(calc win32gui UNICODE)
add_importlibs(calc advapi32 comctl32 user32 shell32 gdi32 msvcrt kernel32)

add_pch(calc calc.h SOURCE)
add_cd_file(TARGET calc DESTINATION reactos/system32 FOR all)
