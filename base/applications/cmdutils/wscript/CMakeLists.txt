
list(APPEND SOURCE
    # arguments.c  # replaced by Rust (sdk/lib/rust/wscript-rust/src/wscript/arguments.rs)
    # host.c  # replaced by Rust (sdk/lib/rust/wscript-rust/src/wscript/host.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/wscript-rust/src/wscript/main.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    precomp.h)

# Every translation unit is Rust now (wscript-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wscript_rust wscript-rust "" TRUE)
add_executable(wscript ${SOURCE} rsrc.rc)
add_dependencies(wscript rust_wscript_rust)
target_link_libraries(wscript ${wscript_rust_LIB})
add_idl_headers(wscript_idlheader ihost.idl)
add_typelib(ihost.idl)
set_source_files_properties(rsrc.rc PROPERTIES OBJECT_DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/ihost.tlb)
target_link_libraries(wscript uuid wine oldnames)
set_module_type(wscript win32gui UNICODE)
add_delay_importlibs(wscript shlwapi)
add_importlibs(wscript shell32 oleaut32 ole32 user32 advapi32 msvcrt kernel32 ntdll)
add_dependencies(wscript stdole2 wscript_idlheader)
add_pch(wscript precomp.h SOURCE)
add_cd_file(TARGET wscript DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(wscript) # CORE-5743: No ARRAY_SIZE macro
