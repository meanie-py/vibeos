
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/sound
    ${REACTOS_SOURCE_DIR}/drivers/wdm/audio/legacy/wdmaud)

spec2def(wdmaud.drv wdmaud.spec)

list(APPEND SOURCE
    # wdmaud.c  # replaced by Rust (sdk/lib/rust/wdmaud_drv-rust/src/wdmaud_drv/wdmaud.rs)
    # mixer.c  # replaced by Rust (sdk/lib/rust/wdmaud_drv-rust/src/wdmaud_drv/mixer.rs)
    # mmixer.c  # replaced by Rust (sdk/lib/rust/wdmaud_drv-rust/src/wdmaud_drv/mmixer.rs)
    # legacy.c  # replaced by Rust (sdk/lib/rust/wdmaud_drv-rust/src/wdmaud_drv/legacy.rs)
    rust_cohab_stub.c
    wdmaud.h)

# wdmaud_drv-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(WDMAUD_DRV_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(WDMAUD_DRV_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (wdmaud_drv-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wdmaud_drv_rust wdmaud_drv-rust "" TRUE ${WDMAUD_DRV_RUST_ENV})
add_library(wdmaud.drv MODULE
    ${SOURCE}
    wdmaud.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wdmaud.def)
add_dependencies(wdmaud.drv rust_wdmaud_drv_rust)
target_link_libraries(wdmaud.drv ${wdmaud_drv_rust_LIB})
# wdmaud.h includes mmixer.h, whose directory the C mmixer target used to export.
target_include_directories(wdmaud.drv PRIVATE ${REACTOS_SOURCE_DIR}/sdk/lib/drivers/sound/mmixer)
add_pch(wdmaud.drv wdmaud.h SOURCE)

set_module_type(wdmaud.drv win32dll UNICODE)
set_target_properties(wdmaud.drv PROPERTIES SUFFIX "")
# The mixer library is mmixer-rust, built for user mode (its `user` panic personality);
# the mmixer target links the kernel build for wdmaud.sys.
rust_staticlib_variant(wdmaud_drv_mmixer_rust mmixer-rust "user" FALSE)
add_dependencies(wdmaud.drv rust_wdmaud_drv_mmixer_rust)
target_link_libraries(wdmaud.drv mmebuddy libsamplerate ${wdmaud_drv_mmixer_rust_LIB})
add_importlibs(wdmaud.drv user32 winmm advapi32 msvcrt setupapi ksuser kernel32 ntdll)
add_cd_file(TARGET wdmaud.drv DESTINATION reactos/system32 FOR all)
