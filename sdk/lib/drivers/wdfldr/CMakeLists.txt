spec2def(wdfldr.sys wdfldr.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # class.c  # replaced by Rust (sdk/lib/rust/wdfldr-rust/src/wdfldr/class.rs)
    # common.c  # replaced by Rust (sdk/lib/rust/wdfldr-rust/src/wdfldr/common.rs)
    # library.c  # replaced by Rust (sdk/lib/rust/wdfldr-rust/src/wdfldr/library.rs)
    # registry.c  # replaced by Rust (sdk/lib/rust/wdfldr-rust/src/wdfldr/registry.rs)
    # wdfldr.c  # replaced by Rust (sdk/lib/rust/wdfldr-rust/src/wdfldr/wdfldr.rs)
    rust_cohab_stub.c
    )

# Every translation unit is Rust now (wdfldr-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wdfldr_rust wdfldr-rust "" TRUE)
add_library(wdfldr MODULE
    ${SOURCE}
    wdfldr.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wdfldr.def)
add_dependencies(wdfldr rust_wdfldr_rust)
target_link_libraries(wdfldr ${wdfldr_rust_LIB})

target_include_directories(wdfldr
    PRIVATE ${REACTOS_SOURCE_DIR}/vendor/wdf/shared/inc/private/common
    ${REACTOS_SOURCE_DIR}/vendor/wdf/kmdf/inc/private
    ${REACTOS_SOURCE_DIR}/sdk/lib/drivers/wdf_reactos/include
)

set_module_type(wdfldr kernelmodedriver ENTRYPOINT 0)
target_link_libraries(wdfldr aux_klib libcntpr ${PSEH_LIB})
add_importlibs(wdfldr ntoskrnl hal)
add_pch(wdfldr wdfloader.h SOURCE)
add_cd_file(TARGET wdfldr DESTINATION reactos/system32/drivers NO_CAB FOR all)
add_registry_inf(wdfldr_reg.inf)
