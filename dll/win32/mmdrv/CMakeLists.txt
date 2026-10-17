
spec2def(mmdrv.dll mmdrv.spec)

list(APPEND SOURCE
    # entry.c  # replaced by Rust (sdk/lib/rust/mmdrv-rust/src/mmdrv/entry.rs)
    # mme.c  # replaced by Rust (sdk/lib/rust/mmdrv-rust/src/mmdrv/mme.rs)
    # kernel.c  # replaced by Rust (sdk/lib/rust/mmdrv-rust/src/mmdrv/kernel.rs)
    # session.c  # replaced by Rust (sdk/lib/rust/mmdrv-rust/src/mmdrv/session.rs)
    # common.c  # replaced by Rust (sdk/lib/rust/mmdrv-rust/src/mmdrv/common.rs)
    # wave.c  # replaced by Rust (sdk/lib/rust/mmdrv-rust/src/mmdrv/wave.rs)
    # wave_io.c  # replaced by Rust (sdk/lib/rust/mmdrv-rust/src/mmdrv/wave_io.rs)
    rust_cohab_stub.c
    mmdrv.h)

# Every translation unit is Rust now (mmdrv-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mmdrv_rust mmdrv-rust "" TRUE)
add_library(mmdrv MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/mmdrv.def)
add_dependencies(mmdrv rust_mmdrv_rust)
target_link_libraries(mmdrv ${mmdrv_rust_LIB})

set_module_type(mmdrv win32dll UNICODE)
add_importlibs(mmdrv winmm user32 msvcrt kernel32 ntdll)
add_pch(mmdrv mmdrv.h SOURCE)
add_cd_file(TARGET mmdrv DESTINATION reactos/system32 FOR all)
