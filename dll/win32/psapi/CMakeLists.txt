
spec2def(psapi.dll psapi.spec ADD_IMPORTLIB)

# psapi.c is Rust now (psapi-rust, `rustconv port`); the C stays on disk. psapi_static keeps
# a stub TU and links the Rust archive -- kernel32 and kernelbase link it too.
list(APPEND PSAPI_STATIC_SOURCE
    # psapi.c  # replaced by Rust (sdk/lib/rust/psapi-rust/src/psapi/psapi.rs)
    rust_cohab_stub.c)
rust_staticlib_variant(psapi_rust psapi-rust "" TRUE)
add_library(psapi_static ${PSAPI_STATIC_SOURCE})
add_dependencies(psapi_static psdk rust_psapi_rust)
target_link_libraries(psapi_static PRIVATE ${psapi_rust_LIB})

list(APPEND SOURCE
    main.c
    psapi.rc
    ${CMAKE_CURRENT_BINARY_DIR}/psapi.def)

add_library(psapi MODULE ${SOURCE})
set_module_type(psapi win32dll ENTRYPOINT DllMain 12)
target_link_libraries(psapi psapi_static ${PSEH_LIB})
add_importlibs(psapi kernel32 ntdll)
add_cd_file(TARGET psapi DESTINATION reactos/system32 FOR all)
