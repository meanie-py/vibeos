
include_directories(${SHIMLIB_DIR})

spec2def(acgenral.dll genral.spec)

list(APPEND SOURCE
    # ignoredbgout.c  # replaced by Rust (sdk/lib/rust/acgenral-rust/src/acgenral/ignoredbgout.rs)
    # ignorefreelib.c  # replaced by Rust (sdk/lib/rust/acgenral-rust/src/acgenral/ignorefreelib.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/acgenral-rust/src/acgenral/main.rs)
    # shimtest.c  # replaced by Rust (sdk/lib/rust/acgenral-rust/src/acgenral/shimtest.rs)
    # themes.c  # replaced by Rust (sdk/lib/rust/acgenral-rust/src/acgenral/themes.rs)
    rust_cohab_stub.c
    genral.spec)

if(ARCH STREQUAL "amd64")
    list(APPEND SOURCE msys2.c)
endif()

# Every translation unit is Rust now (acgenral-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(acgenral_rust acgenral-rust "" TRUE)
add_library(acgenral MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/acgenral.def)
add_dependencies(acgenral rust_acgenral_rust)
target_link_libraries(acgenral ${acgenral_rust_LIB})

set_module_type(acgenral win32dll ENTRYPOINT DllMain 12)
target_link_libraries(acgenral shimlib)
add_importlibs(acgenral uxtheme kernel32 ntdll)
add_cd_file(TARGET acgenral DESTINATION reactos/AppPatch FOR all)
