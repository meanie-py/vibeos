
# Every translation unit is Rust now (route-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(route_rust route-rust "" TRUE)
add_executable(route
    # route.c  # replaced by Rust (sdk/lib/rust/route-rust/src/route/route.rs)
    rust_cohab_stub.c
    route.rc)
add_dependencies(route rust_route_rust)
target_link_libraries(route ${route_rust_LIB})
set_module_type(route win32cui UNICODE)
target_compile_definitions(route PRIVATE _CRT_NON_CONFORMING_SWPRINTFS)
target_link_libraries(route conutils)
add_importlibs(route ws2_32 iphlpapi advapi32 msvcrt kernel32 ntdll) ## ntdll for RtlIpv*
add_cd_file(TARGET route DESTINATION reactos/system32 FOR all)
