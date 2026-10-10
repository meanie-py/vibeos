
include_directories(${SHIMLIB_DIR})

spec2def(aclayers.dll layer.spec)

list(APPEND SOURCE
    # dispmode.c  # replaced by Rust (sdk/lib/rust/aclayers-rust/src/aclayers/dispmode.rs)
    # forcedxsetupsuccess.c  # replaced by Rust (sdk/lib/rust/aclayers-rust/src/aclayers/forcedxsetupsuccess.rs)
    # globalmemorystatus.c  # replaced by Rust (sdk/lib/rust/aclayers-rust/src/aclayers/globalmemorystatus.rs)
    # ignoreloadlibrary.c  # replaced by Rust (sdk/lib/rust/aclayers-rust/src/aclayers/ignoreloadlibrary.rs)
    # versionlie.c  # replaced by Rust (sdk/lib/rust/aclayers-rust/src/aclayers/versionlie.rs)
    # vmhorizon.c  # replaced by Rust (sdk/lib/rust/aclayers-rust/src/aclayers/vmhorizon.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/aclayers-rust/src/aclayers/main.rs)
    rust_cohab_stub.c
    layer.spec)

# Every translation unit is Rust now (aclayers-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(aclayers_rust aclayers-rust "" TRUE)
add_library(aclayers MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/aclayers.def)
add_dependencies(aclayers rust_aclayers_rust)
target_link_libraries(aclayers ${aclayers_rust_LIB})

set_module_type(aclayers win32dll ENTRYPOINT DllMain 12)
target_link_libraries(aclayers shimlib)
add_importlibs(aclayers user32 kernel32 ntdll)
add_cd_file(TARGET aclayers DESTINATION reactos/AppPatch FOR all)
