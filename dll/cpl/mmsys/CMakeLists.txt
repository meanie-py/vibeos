
spec2def(mmsys.cpl mmsys.spec)

list(APPEND SOURCE
    # mmsys.c  # replaced by Rust (sdk/lib/rust/mmsys-rust/src/mmsys/mmsys.rs)
    # sounds.c  # replaced by Rust (sdk/lib/rust/mmsys-rust/src/mmsys/sounds.rs)
    # speakervolume.c  # replaced by Rust (sdk/lib/rust/mmsys-rust/src/mmsys/speakervolume.rs)
    # volume.c  # replaced by Rust (sdk/lib/rust/mmsys-rust/src/mmsys/volume.rs)
    # audio.c  # replaced by Rust (sdk/lib/rust/mmsys-rust/src/mmsys/audio.rs)
    # voice.c  # replaced by Rust (sdk/lib/rust/mmsys-rust/src/mmsys/voice.rs)
    rust_cohab_stub.c
    mmsys.h)

file(GLOB mmsys_rc_deps resources/*.*)
add_rc_deps(mmsys.rc ${mmsys_rc_deps})

# Every translation unit is Rust now (mmsys-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mmsys_rust mmsys-rust "" TRUE)
add_library(mmsys MODULE
    ${SOURCE}
    mmsys.rc
    ${CMAKE_CURRENT_BINARY_DIR}/mmsys.def)
add_dependencies(mmsys rust_mmsys_rust)
target_link_libraries(mmsys ${mmsys_rust_LIB})

set_module_type(mmsys cpl UNICODE)
add_importlibs(mmsys user32 comctl32 devmgr gdi32 winmm advapi32 shell32 setupapi shlwapi comdlg32 msvcrt kernel32 ntdll)
add_pch(mmsys mmsys.h SOURCE)
add_cd_file(TARGET mmsys DESTINATION reactos/system32 FOR all)
