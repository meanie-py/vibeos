
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(mfplat.dll mfplat.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # buffer.c  # replaced by Rust (sdk/lib/rust/mfplat-rust/src/mfplat/buffer.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/mfplat-rust/src/mfplat/main.rs)
    # mediatype.c  # replaced by Rust (sdk/lib/rust/mfplat-rust/src/mfplat/mediatype.rs)
    # queue.c  # replaced by Rust (sdk/lib/rust/mfplat-rust/src/mfplat/queue.rs)
    # sample.c  # replaced by Rust (sdk/lib/rust/mfplat-rust/src/mfplat/sample.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/mfplat_stubs.c)

# mfplat-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(MFPLAT_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(MFPLAT_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (mfplat-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mfplat_rust mfplat-rust "" TRUE ${MFPLAT_RUST_ENV})
add_library(mfplat MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/mfplat.def)
add_dependencies(mfplat rust_mfplat_rust)
target_link_libraries(mfplat ${mfplat_rust_LIB})

set_module_type(mfplat win32dll)
target_link_libraries(mfplat wine uuid dmoguids mfuuid strmiids dxguid oldnames)
add_delay_importlibs(mfplat bcrypt)
add_importlibs(mfplat advapi32 ole32 oleaut32 propsys rtworkq user32 shlwapi kernelbase msvcrt kernel32 kernel32_vista ntdll)
add_dependencies(mfplat psdk wineheaders)
add_cd_file(TARGET mfplat DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(mfplat)
