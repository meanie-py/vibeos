
list(APPEND SOURCE
    # audio_format.cpp  # replaced by Rust (sdk/lib/rust/sndrec32-rust/src/sndrec32/cpp_classes.rs: its globals)
    # audio_membuffer.cpp  # replaced by Rust (sdk/lib/rust/sndrec32-rust/src/sndrec32/audio_membuffer_cpp_h.rs)
    #audio_producer.cpp
    #audio_receiver.cpp
    # audio_resampler_acm.cpp  # replaced by Rust (sdk/lib/rust/sndrec32-rust/src/sndrec32/audio_resampler_acm_cpp_h.rs)
    # audio_wavein.cpp  # replaced by Rust (sdk/lib/rust/sndrec32-rust/src/sndrec32/audio_wavein_cpp_h.rs)
    # audio_waveout.cpp  # replaced by Rust (sdk/lib/rust/sndrec32-rust/src/sndrec32/audio_waveout_cpp_h.rs)
    # sndrec32.cpp  # replaced by Rust (sdk/lib/rust/sndrec32-rust/src/sndrec32/sndrec32_cpp_h.rs)
    rust_cohab_stub.c
    stdafx.h)

file(GLOB sndrec32_rc_deps resources/*.*)
add_rc_deps(rsrc.rc ${sndrec32_rc_deps})
# Every translation unit is Rust now (sndrec32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
# sndrec32-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(SNDREC32_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(SNDREC32_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
rust_staticlib_variant(sndrec32_rust sndrec32-rust "" TRUE ${SNDREC32_RUST_ENV})
add_executable(sndrec32 ${SOURCE} rsrc.rc)
add_dependencies(sndrec32 rust_sndrec32_rust)
target_link_libraries(sndrec32 ${sndrec32_rust_LIB})
set_module_type(sndrec32 win32gui UNICODE)
add_importlibs(sndrec32 winmm user32 msacm32 comctl32 comdlg32 gdi32 shell32 msvcrt kernel32)
add_pch(sndrec32 stdafx.h SOURCE)
add_cd_file(TARGET sndrec32 DESTINATION reactos/system32 FOR all)
