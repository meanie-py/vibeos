
# The delay-load runtime is Rust (sdk/lib/rust/delayimp-rust): __delayLoadHelper2 and the
# __pfnDliNotifyHook2/__pfnDliFailureHook2 hook variables, from delayimp.c,
# pfnDliNotifyHook2.c and pfnDliFailureHook2.c (kept on disk as the record). `delayimp` stays
# the target gcc.cmake's add_delay_importlibs links, as an INTERFACE over the archive; every
# consumer is user mode. A module that defines its own hook does so in its own object, which
# precedes this archive on the link line and wins.
rust_staticlib_variant(delayimp_rust delayimp-rust "" TRUE)
add_library(delayimp INTERFACE)
add_dependencies(delayimp rust_delayimp_rust)
target_link_libraries(delayimp INTERFACE ${delayimp_rust_LIB} libkernel32)
