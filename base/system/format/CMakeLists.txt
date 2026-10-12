
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/fmifs)

# format-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(FORMAT_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(FORMAT_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (format-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(format_rust format-rust "" TRUE ${FORMAT_RUST_ENV})
add_executable(format
    # format.c  # replaced by Rust (sdk/lib/rust/format-rust/src/format/format.rs)
    rust_cohab_stub.c
    format.rc)
add_dependencies(format rust_format_rust)
target_link_libraries(format ${format_rust_LIB})
set_module_type(format win32cui UNICODE)
target_link_libraries(format conutils)
add_importlibs(format fmifs msvcrt kernel32)
set_target_properties(format PROPERTIES SUFFIX ".com")
add_cd_file(TARGET format DESTINATION reactos/system32 FOR all)
