
add_definitions(-DDEBUG_NT4)
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/sound)
spec2def(sndblst.dll sndblst.spec)

list(APPEND SOURCE
    # sndblst.c  # replaced by Rust (sdk/lib/rust/sndblst-rust/src/sndblst/sndblst.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/sndblst.def)

# Every translation unit is Rust now (sndblst-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(sndblst_rust sndblst-rust "" TRUE)
add_library(sndblst MODULE ${SOURCE})
add_dependencies(sndblst rust_sndblst_rust)
target_link_libraries(sndblst ${sndblst_rust_LIB})
set_module_type(sndblst win32dll UNICODE)
target_link_libraries(sndblst mment4 mmebuddy)
add_importlibs(sndblst user32 winmm advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET sndblst DESTINATION reactos/system32 FOR all)
