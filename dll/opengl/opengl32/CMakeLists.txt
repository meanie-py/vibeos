
spec2def(opengl32.dll opengl32.spec ADD_IMPORTLIB)

add_definitions(
    -D_GDI32_ # prevent gl* being declared __declspec(dllimport) in MS headers
    -DBUILD_GL32 # declare gl* as __declspec(dllexport) in Mesa headers
    -D_GLAPI_NO_EXPORTS # prevent _glapi_* from being declared __declspec(dllimport)
)

if(OPENGL32_USE_TLS)
    add_definitions(-DOPENGL32_USE_TLS)
endif()

list(APPEND SOURCE
    # apistubs.c  # replaced by Rust (sdk/lib/rust/opengl32-rust/src/opengl32/apistubs.rs)
    # dllmain.c  # replaced by Rust (sdk/lib/rust/opengl32-rust/src/opengl32/dllmain.rs)
    # icdload.c  # replaced by Rust (sdk/lib/rust/opengl32-rust/src/opengl32/icdload.rs)
    # swimpl.c  # replaced by Rust (sdk/lib/rust/opengl32-rust/src/opengl32/swimpl.rs)
    # wgl.c  # replaced by Rust (sdk/lib/rust/opengl32-rust/src/opengl32/wgl.rs)
    # wgl_font.c  # replaced by Rust (sdk/lib/rust/opengl32-rust/src/opengl32/wgl_font.rs)
    rust_cohab_stub.c
    opengl32.h
    ${CMAKE_CURRENT_BINARY_DIR}/opengl32_stubs.c)

# set_source_files_properties(gcrt0.o libgmon.a PROPERTIES EXTERNAL_OBJECT TRUE)

if(ARCH STREQUAL "i386")
    # Optimisation: use asm trampolines to ICD provided functions
    add_asm_files(opengl32_asm
        glapi_x86.s
    )
endif()

# Every translation unit is Rust now (opengl32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(opengl32_rust opengl32-rust "" TRUE)
add_library(opengl32 MODULE
    ${SOURCE}
    ${opengl32_asm}
    ${CMAKE_CURRENT_BINARY_DIR}/opengl32.def)
add_dependencies(opengl32 rust_opengl32_rust)
target_link_libraries(opengl32 ${opengl32_rust_LIB})
add_pch(opengl32 opengl32.h SOURCE)

target_link_libraries(opengl32 mesa wine ${PSEH_LIB})

set_module_type(opengl32 win32dll)
add_importlibs(opengl32 gdi32 user32 advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET opengl32 DESTINATION reactos/system32 FOR all)
