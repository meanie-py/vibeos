
## The message string templates are in ANSI to reduce binary size
add_message_headers(ANSI evtmsgstr.mc)

# Every translation unit is Rust now (eventcreate-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(eventcreate_rust eventcreate-rust "" TRUE)
add_executable(eventcreate
    # eventcreate.c  # replaced by Rust (sdk/lib/rust/eventcreate-rust/src/eventcreate/eventcreate.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    eventcreate.rc)
add_dependencies(eventcreate rust_eventcreate_rust)
target_link_libraries(eventcreate ${eventcreate_rust_LIB})
set_module_type(eventcreate win32cui UNICODE)
add_dependencies(eventcreate evtmsgstr)
target_link_libraries(eventcreate conutils)
add_importlibs(eventcreate advapi32 msvcrt kernel32)
add_cd_file(TARGET eventcreate DESTINATION reactos/system32 FOR all)
