
# Every translation unit is Rust now (clip-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(clip_rust clip-rust "" TRUE)
add_executable(clip
    # clip.c  # replaced by Rust (sdk/lib/rust/clip-rust/src/clip/clip.rs)
    rust_cohab_stub.c
    clip.rc)
add_dependencies(clip rust_clip_rust)
target_link_libraries(clip ${clip_rust_LIB})
set_module_type(clip win32cui UNICODE)
target_link_libraries(clip conutils)
add_importlibs(clip advapi32 user32 msvcrt kernel32)
add_cd_file(TARGET clip DESTINATION reactos/system32 FOR all)
