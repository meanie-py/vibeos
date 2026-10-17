
spec2def(beepmidi.dll beepmidi.spec)

list(APPEND SOURCE
    # beepmidi.c  # replaced by Rust (sdk/lib/rust/beepmidi-rust/src/beepmidi/beepmidi.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    ${CMAKE_CURRENT_BINARY_DIR}/beepmidi.def)

# beepmidi-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(BEEPMIDI_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(BEEPMIDI_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (beepmidi-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(beepmidi_rust beepmidi-rust "" TRUE ${BEEPMIDI_RUST_ENV})
add_library(beepmidi MODULE ${SOURCE})
add_dependencies(beepmidi rust_beepmidi_rust)
target_link_libraries(beepmidi ${beepmidi_rust_LIB})
set_module_type(beepmidi win32dll UNICODE)
add_importlibs(beepmidi winmm msvcrt kernel32 ntdll)
add_cd_file(TARGET beepmidi DESTINATION reactos/system32 FOR all)
