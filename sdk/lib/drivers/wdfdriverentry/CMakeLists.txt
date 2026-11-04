remove_definitions(-D_WIN32_WINNT=0x502)

# Every translation unit is Rust now (wdfdriverentry-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wdfdriverentry_rust wdfdriverentry-rust "" TRUE)
add_library(wdfdriverentry STATIC
    # wdfdriverentry.c  # replaced by Rust (sdk/lib/rust/wdfdriverentry-rust/src/wdfdriverentry/wdfdriverentry.rs)
    rust_cohab_stub.c
    )
add_dependencies(wdfdriverentry rust_wdfdriverentry_rust)
target_link_libraries(wdfdriverentry ${wdfdriverentry_rust_LIB})
add_dependencies(wdfdriverentry xdk)

if(GCC)
    target_compile_options(wdfdriverentry PRIVATE -Wno-unknown-pragmas -Wno-switch)
endif()

target_compile_definitions(wdfdriverentry
    PUBLIC WDF_DEVICE_NO_WDMSEC_H _WIN32_WINNT=0x603 NTDDI_VERSION=0x06030000
    PRIVATE __WDF_MAJOR_VERSION=1 __WDF_MINOR_VERSION=17 __WDF_BUILD_NUMBER=7600
)

target_include_directories(wdfdriverentry
    PUBLIC ${REACTOS_SOURCE_DIR}/vendor/wdf/include/kmdf/1.17
    PRIVATE ${REACTOS_SOURCE_DIR}/vendor/wdf/shared/inc/private/common
    ${REACTOS_SOURCE_DIR}/vendor/wdf/kmdf/inc/private
    ${REACTOS_SOURCE_DIR}/sdk/lib/drivers/wdf_reactos/include
)
