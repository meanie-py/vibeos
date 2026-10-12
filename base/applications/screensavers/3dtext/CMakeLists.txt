
list(APPEND SOURCE
    # 3dtext.c  # replaced by Rust (sdk/lib/rust/text3d-rust/src/text3d/3dtext.rs)
    # settings.c  # replaced by Rust (sdk/lib/rust/text3d-rust/src/text3d/settings.rs)
    rust_cohab_stub.c
    3dtext.h)

add_rc_deps(rsrc.rc ${CMAKE_CURRENT_SOURCE_DIR}/res/icon_3dtext.ico)
# text3d-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(TEXT3D_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(TEXT3D_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (text3d-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(text3d_rust text3d-rust "" TRUE ${TEXT3D_RUST_ENV})
add_executable(3dtext ${SOURCE} rsrc.rc)
add_dependencies(3dtext rust_text3d_rust)
target_link_libraries(3dtext ${text3d_rust_LIB})
set_module_type(3dtext win32gui UNICODE)
set_target_properties(3dtext PROPERTIES SUFFIX ".scr")
target_link_libraries(3dtext scrnsave)
add_importlibs(3dtext user32 gdi32 opengl32 glu32 advapi32 msvcrt kernel32)
add_pch(3dtext 3dtext.h SOURCE)
add_cd_file(TARGET 3dtext DESTINATION reactos/system32 FOR all)
