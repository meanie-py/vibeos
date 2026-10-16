
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(evr.dll evr.spec ADD_IMPORTLIB)

add_idl_headers(evr_classes_header evr_classes.idl)

list(APPEND SOURCE
    # evr.c  # replaced by Rust (sdk/lib/rust/evr-rust/src/evr/evr.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/evr-rust/src/evr/main.rs)
    # mixer.c  # replaced by Rust (sdk/lib/rust/evr-rust/src/evr/mixer.rs)
    # presenter.c  # replaced by Rust (sdk/lib/rust/evr-rust/src/evr/presenter.rs)
    # sample.c  # replaced by Rust (sdk/lib/rust/evr-rust/src/evr/sample.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/evr_stubs.c)

# evr-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(EVR_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(EVR_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (evr-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(evr_rust evr-rust "" TRUE ${EVR_RUST_ENV})
add_library(evr MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/evr.def)
add_dependencies(evr rust_evr_rust)
target_link_libraries(evr ${evr_rust_LIB})

set_module_type(evr win32dll)
add_idl_reg_scripts(evr registry evr_classes.idl)
target_link_libraries(evr wine wine_dll_register wine_dll_canunload strmbase mfuuid strmiids uuid dxguid oldnames)
add_delay_importlibs(evr mfplat)
add_importlibs(evr ole32 oleaut32 user32 d3d9 dxva2 msvcrt kernel32 ntdll)
add_dependencies(evr psdk wineheaders evr_classes_header)
add_cd_file(TARGET evr DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(evr)
