project(MSPAINT)

add_definitions(-DINITGUID)

if(DBG)
    add_definitions(-D_DEBUG=1) # CORE-17505
endif()

list(APPEND SOURCE
    # canvas.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/canvas_cpp_h.rs)
    # dialogs.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/dialogs_cpp_h.rs)
    # dib.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/dib_cpp_h.rs)
    # drawing.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/drawing_cpp_h.rs)
    # fullscreen.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/fullscreen_cpp_h.rs)
    # history.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/history_cpp_h.rs)
    # main.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/main_cpp_h.rs)
    # miniature.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/miniature_cpp_h.rs)
    # mouse.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/mouse_cpp_h.rs)
    # palette.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/palette_cpp_h.rs)
    # palettemodel.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/palettemodel_cpp_h.rs)
    # registry.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/registry_cpp_h.rs)
    # selectionmodel.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/selectionmodel_cpp_h.rs)
    # sizebox.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/sizebox_cpp_h.rs)
    # textedit.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/textedit_cpp_h.rs)
    # toolbox.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/toolbox_cpp_h.rs)
    # toolsettings.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/toolsettings_cpp_h.rs)
    # toolsmodel.cpp  # replaced by Rust (sdk/lib/rust/mspaint-rust/src/mspaint/toolsmodel_cpp_h.rs)
    rust_varargs_shim.c
    rust_cohab_stub.c
    precomp.h)

file(GLOB mspaint_rc_deps img/*.*)
add_rc_deps(rsrc.rc ${mspaint_rc_deps})
# mspaint-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(MSPAINT_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(MSPAINT_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (mspaint-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mspaint_rust mspaint-rust "" TRUE ${MSPAINT_RUST_ENV})
add_executable(mspaint ${SOURCE} rsrc.rc)
add_dependencies(mspaint rust_mspaint_rust)
target_link_libraries(mspaint ${mspaint_rust_LIB})
set_module_type(mspaint win32gui UNICODE)
target_link_libraries(mspaint uuid cpprt atl_classes)
set_target_cpp_properties(mspaint WITH_EXCEPTIONS)
add_importlibs(mspaint comdlg32 shell32 user32 gdi32 advapi32 comctl32 msvcrt kernel32 rpcrt4 shlwapi ntdll)
add_pch(mspaint precomp.h SOURCE)
add_cd_file(TARGET mspaint DESTINATION reactos/system32 FOR all)
