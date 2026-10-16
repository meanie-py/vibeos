
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(mf.dll mf.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # clock.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/clock.rs)
    # copier.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/copier.rs)
    # evr.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/evr.rs)
    # main.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/main.rs)
    # quality.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/quality.rs)
    # sac.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/sac.rs)
    # samplegrabber.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/samplegrabber.rs)
    # sar.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/sar.rs)
    # scheme_handler.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/scheme_handler.rs)
    # session.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/session.rs)
    # topology.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/topology.rs)
    # topology_loader.c  # replaced by Rust (sdk/lib/rust/mf-rust/src/mf/topology_loader.rs)
    rust_cohab_stub.c
    ${CMAKE_CURRENT_BINARY_DIR}/mf_stubs.c)

# mf-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(MF_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(MF_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (mf-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mf_rust mf-rust "" TRUE ${MF_RUST_ENV})
add_library(mf MODULE
    ${SOURCE}
    mf.rc
    ${CMAKE_CURRENT_BINARY_DIR}/mf.def)
add_dependencies(mf rust_mf_rust)
target_link_libraries(mf ${mf_rust_LIB})

set_module_type(mf win32dll)
add_idl_reg_scripts(mf registry mf.idl)
target_link_libraries(mf wine wine_dll_register wine_dll_canunload uuid mfuuid strmiids dxguid oldnames)
add_delay_importlibs(mf evr urlmon user32)
add_importlibs(mf advapi32 mfplat ole32 oleaut32 shlwapi kernelbase msvcrt kernel32 kernel32_vista ntdll)
add_dependencies(mf psdk wineheaders)
add_cd_file(TARGET mf DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(mf)
