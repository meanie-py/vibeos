
spec2def(user32_vista.dll user32_vista.spec ADD_IMPORTLIB)

include_directories(
    include
    ${REACTOS_SOURCE_DIR}/win32ss/include)

list(APPEND VISTA_SOURCE 
     # wine/input.c  # replaced by Rust (sdk/lib/rust/user32_optional_vista-rust/src/user32_optional_vista/input.rs)
     # wine/misc.c  # replaced by Rust (sdk/lib/rust/user32_optional_vista-rust/src/user32_optional_vista/misc.rs)
     # wine/sysparams.c  # replaced by Rust (sdk/lib/rust/user32_optional_vista-rust/src/user32_optional_vista/sysparams.rs)
     # wine/win.c  # replaced by Rust (sdk/lib/rust/user32_optional_vista-rust/src/user32_optional_vista/win.rs)
     rust_cohab_stub.c
     )
     

list(APPEND SOURCE
    # dpi.c  # replaced by Rust (sdk/lib/rust/user32_vista-rust/src/user32_vista/dpi.rs)
    rust_cohab_stub.c
    )

# These are just for Vista+ support
# Every translation unit is Rust now (user32_optional_vista-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(user32_optional_vista_rust user32_optional_vista-rust "" TRUE)
add_library(user32_optional_vista
            ${VISTA_SOURCE})
add_dependencies(user32_optional_vista rust_user32_optional_vista_rust)
target_link_libraries(user32_optional_vista ${user32_optional_vista_rust_LIB})
target_link_libraries(user32_optional_vista wine win32ksys)
add_dependencies(user32_optional_vista wine psdk)

# These are required for WINE imports
# Every translation unit is Rust now (user32_vista-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(user32_vista_rust user32_vista-rust "" TRUE)
add_library(user32_vista_static
            ${SOURCE})
add_dependencies(user32_vista_static rust_user32_vista_rust)
target_link_libraries(user32_vista_static ${user32_vista_rust_LIB})
target_link_libraries(user32_vista_static win32ksys)
add_dependencies(user32_vista_static psdk)

add_library(user32_vista MODULE
    ${SOURCE}
    user32_vista.rc
    ${CMAKE_CURRENT_BINARY_DIR}/user32_vista.def)

set_module_type(user32_vista win32dll UNICODE ENTRYPOINT 0)
add_importlibs(user32_vista user32 gdi32 ntdll)
target_link_libraries(user32_vista user32_vista_static win32ksys)
add_dependencies(user32_vista psdk)
add_cd_file(TARGET user32_vista DESTINATION reactos/system32 FOR all)
