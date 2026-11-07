remove_definitions(-D_WIN32_WINNT=0x502)

# Every translation unit is Rust now (wdfdriverentry-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(wdfdriverentry_rust wdfdriverentry-rust "" TRUE)
# wdfdriverentry.c  # replaced by Rust (sdk/lib/rust/wdfdriverentry-rust/src/wdfdriverentry/wdfdriverentry.rs)
add_library(wdfdriverentry INTERFACE)
add_dependencies(wdfdriverentry rust_wdfdriverentry_rust)
target_link_libraries(wdfdriverentry INTERFACE ${wdfdriverentry_rust_LIB})
add_dependencies(wdfdriverentry xdk)

target_compile_definitions(wdfdriverentry INTERFACE WDF_DEVICE_NO_WDMSEC_H _WIN32_WINNT=0x603 NTDDI_VERSION=0x06030000)

target_include_directories(wdfdriverentry INTERFACE ${REACTOS_SOURCE_DIR}/vendor/wdf/include/kmdf/1.17)
