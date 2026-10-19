
project(SHELL)

spec2def(stobject.dll stobject.spec)

file(GLOB_RECURSE stobject_rc_deps resources/*.*)
add_rc_deps(stobject.rc ${stobject_rc_deps})

list(APPEND SOURCE
    # csystray.cpp  # replaced by Rust (sdk/lib/rust/stobject-rust/src/stobject/csystray_cpp_h.rs)
    # stobject.cpp  # replaced by Rust (sdk/lib/rust/stobject-rust/src/stobject/stobject_cpp_h.rs)
    # hotplug.cpp  # replaced by Rust (sdk/lib/rust/stobject-rust/src/stobject/hotplug_cpp_h.rs)
    # mouse.cpp  # replaced by Rust (sdk/lib/rust/stobject-rust/src/stobject/mouse_cpp_h.rs)
    # power.cpp  # replaced by Rust (sdk/lib/rust/stobject-rust/src/stobject/power_cpp_h.rs)
    # volume.cpp  # replaced by Rust (sdk/lib/rust/stobject-rust/src/stobject/volume_cpp_h.rs)
    rust_cohab_stub.cpp
    precomp.h)

# Every translation unit is Rust now (stobject-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(stobject_rust stobject-rust "" TRUE)
add_library(stobject MODULE
    ${SOURCE}
    stobject.rc
    ${CMAKE_CURRENT_BINARY_DIR}/stobject.def)
add_dependencies(stobject rust_stobject_rust)
target_link_libraries(stobject ${stobject_rust_LIB})

set_module_type(stobject win32dll UNICODE)
target_link_libraries(stobject uuid wine cpprt atl_classes)

add_importlibs(stobject
    setupapi
    advapi32
    winmm
    ole32
    oleaut32
    powrprof
    shlwapi
    shell32
    comctl32
    msvcrt
    gdi32
    user32
    kernel32
    ntdll)

add_pch(stobject precomp.h SOURCE)
add_cd_file(TARGET stobject DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
