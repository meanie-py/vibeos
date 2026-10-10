
list(APPEND SOURCE
    # system.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/system.rs)
    # display.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/display.rs)
    # sound.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/sound.rs)
    # music.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/music.rs)
    # input.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/input.rs)
    # network.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/network.rs)
    # help.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/help.rs)
    # dxdiag.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/dxdiag.rs)
    # ddtest.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/ddtest.rs)
    # d3dtest.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/d3dtest.rs)
    # d3dtest7.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/d3dtest7.rs)
    # d3dtest8.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/d3dtest8.rs)
    # d3dtest9.c  # replaced by Rust (sdk/lib/rust/dxdiag-rust/src/dxdiag/d3dtest9.rs)
    rust_cohab_stub.c
    precomp.h)

add_rc_deps(dxdiag.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/dxdiag.ico)
# Every translation unit is Rust now (dxdiag-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(dxdiag_rust dxdiag-rust "" TRUE)
add_executable(dxdiag ${SOURCE} dxdiag.rc)
add_dependencies(dxdiag rust_dxdiag_rust)
target_link_libraries(dxdiag ${dxdiag_rust_LIB})
add_pch(dxdiag precomp.h SOURCE)
set_module_type(dxdiag win32gui UNICODE)
target_link_libraries(dxdiag dxguid udmihelp)
add_importlibs(dxdiag user32 advapi32 comctl32 dinput8 setupapi dsound ddraw version gdi32 winmm d3d9 msvcrt kernel32 ntdll)
add_cd_file(TARGET dxdiag DESTINATION reactos/system32 FOR all)
