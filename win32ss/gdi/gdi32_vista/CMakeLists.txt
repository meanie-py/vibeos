
include_directories(
    include
    ${REACTOS_SOURCE_DIR}/win32ss/include)

spec2def(gdi32_vista.dll gdi32_vista.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # dc.c  # replaced by Rust (sdk/lib/rust/gdi32_vista-rust/src/gdi32_vista/dc.rs)
    # d3dkmt.c  # replaced by Rust (sdk/lib/rust/gdi32_vista-rust/src/gdi32_vista/d3dkmt.rs)
    rust_cohab_stub.c
    )

# Every translation unit is Rust now (gdi32_vista-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(gdi32_vista_rust gdi32_vista-rust "" TRUE)
add_library(gdi32_vista MODULE
    ${SOURCE}
    gdi32_vista.rc
    ${CMAKE_CURRENT_BINARY_DIR}/gdi32_vista.def)
add_dependencies(gdi32_vista rust_gdi32_vista_rust)
target_link_libraries(gdi32_vista ${gdi32_vista_rust_LIB})

set_module_type(gdi32_vista win32dll UNICODE ENTRYPOINT 0)

add_library(gdi32_vista_static ${SOURCE})
target_link_libraries(gdi32_vista_static win32ksys)
add_dependencies(gdi32_vista_static psdk)

target_link_libraries(gdi32_vista win32ksys)

add_importlibs(gdi32_vista user32 msvcrt kernel32 ntdll)
#add_pch(gdi32_vista include/precomp.h SOURCE)
add_dependencies(gdi32_vista psdk)
add_cd_file(TARGET gdi32_vista DESTINATION reactos/system32 FOR all)
