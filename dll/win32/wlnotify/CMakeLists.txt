
spec2def(wlnotify.dll wlnotify.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # scard.c  # replaced by Rust (sdk/lib/rust/wlnotify-rust/src/wlnotify/scard.rs)
    # schedsvc.c  # replaced by Rust (sdk/lib/rust/wlnotify-rust/src/wlnotify/schedsvc.rs)
    # senssvc.c  # replaced by Rust (sdk/lib/rust/wlnotify-rust/src/wlnotify/senssvc.rs)
    # termserv.c  # replaced by Rust (sdk/lib/rust/wlnotify-rust/src/wlnotify/termserv.rs)
    # wlballoon.c  # replaced by Rust (sdk/lib/rust/wlnotify-rust/src/wlnotify/wlballoon.rs)
    # wlnotify.c  # replaced by Rust (sdk/lib/rust/wlnotify-rust/src/wlnotify/wlnotify.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (wlnotify-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wlnotify_rust wlnotify-rust "" TRUE)
add_library(wlnotify MODULE
    ${SOURCE}
    wlnotify.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wlnotify.def)
add_dependencies(wlnotify rust_wlnotify_rust)
target_link_libraries(wlnotify ${wlnotify_rust_LIB})

set_module_type(wlnotify win32dll UNICODE ENTRYPOINT DllMain 12)
target_link_libraries(wlnotify wine)
add_importlibs(wlnotify advapi32 kernel32 ntdll)
add_pch(wlnotify precomp.h SOURCE)
add_cd_file(TARGET wlnotify DESTINATION reactos/system32 FOR all)
