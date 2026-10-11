
include_directories(${SHIMLIB_DIR})

remove_definitions(-D_WIN32_WINNT=0x502 -DWINVER=0x502)
add_definitions(-D_WIN32_WINNT=0x601 -DWINVER=0x601)
#add_definitions(-DSDBAPI_DEBUG_ALLOC)

spec2def(apphelp.dll apphelp.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # apphelp.c  # replaced by Rust (sdk/lib/rust/apphelp-rust/src/apphelp/apphelp.rs)
    # dbgheap.c  # replaced by Rust (sdk/lib/rust/apphelp-rust/src/apphelp/dbgheap.rs)
    # hsdb.c  # replaced by Rust (sdk/lib/rust/apphelp-rust/src/apphelp/hsdb.rs)
    # layer.c  # replaced by Rust (sdk/lib/rust/apphelp-rust/src/apphelp/layer.rs)
    # sdbapi.c  # replaced by Rust (sdk/lib/rust/apphelp-rust/src/apphelp/sdbapi.rs)
    # sdbfileattr.c  # replaced by Rust (sdk/lib/rust/apphelp-rust/src/apphelp/sdbfileattr.rs)
    # sdbread.c  # replaced by Rust (sdk/lib/rust/apphelp-rust/src/apphelp/sdbread.rs)
    # sdbstringtable.c  # replaced by Rust (sdk/lib/rust/apphelp-rust/src/apphelp/sdbstringtable.rs)
    # sdbwrite.c  # replaced by Rust (sdk/lib/rust/apphelp-rust/src/apphelp/sdbwrite.rs)
    # shimeng.c  # replaced by Rust (sdk/lib/rust/apphelp-rust/src/apphelp/shimeng.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    apphelp.spec
    ${CMAKE_CURRENT_BINARY_DIR}/apphelp_stubs.c)

list(APPEND HEADERS
    apphelp.h
    sdbpapi.h
    sdbstringtable.h
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/appcompat/sdbtagid.h
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/appcompat/sdbtypes.h
    sdbwrite.h
    shimeng.h)

# Every translation unit is Rust now (apphelp-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(apphelp_rust apphelp-rust "" TRUE)
add_library(apphelp MODULE
    ${SOURCE}
    ${HEADERS}
    apphelp.rc
    ${CMAKE_CURRENT_BINARY_DIR}/apphelp.def)
add_dependencies(apphelp rust_apphelp_rust)
target_link_libraries(apphelp ${apphelp_rust_LIB})

set_module_type(apphelp win32dll UNICODE ENTRYPOINT DllMain 12)
add_delay_importlibs(apphelp version)
add_importlibs(apphelp kernel32 ntdll)
add_cd_file(TARGET apphelp DESTINATION reactos/system32 FOR all)
