
add_definitions(
    -DNDIS50_MINIPORT
    -DNDIS_MINIPORT_DRIVER
    -DNDIS_LEGACY_MINIPORT=1)

list(APPEND SOURCE
    # ndis.c  # replaced by Rust (sdk/lib/rust/rtl8139-rust/src/rtl8139/ndis.rs)
    # hardware.c  # replaced by Rust (sdk/lib/rust/rtl8139-rust/src/rtl8139/hardware.rs)
    # info.c  # replaced by Rust (sdk/lib/rust/rtl8139-rust/src/rtl8139/info.rs)
    # interrupt.c  # replaced by Rust (sdk/lib/rust/rtl8139-rust/src/rtl8139/interrupt.rs)
    rust_cohab_stub.c
    nic.h)

# Every translation unit is Rust now (rtl8139-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(rtl8139_rust rtl8139-rust "" TRUE)
add_library(rtl8139 MODULE ${SOURCE} rtl8139.rc)
add_dependencies(rtl8139 rust_rtl8139_rust)
target_link_libraries(rtl8139 ${rtl8139_rust_LIB})
add_pch(rtl8139 nic.h SOURCE)
set_module_type(rtl8139 kernelmodedriver)
add_importlibs(rtl8139 ndis ntoskrnl hal)
add_cd_file(TARGET rtl8139 DESTINATION reactos/system32/drivers FOR all)
add_driver_inf(rtl8139 netrtl.inf)
