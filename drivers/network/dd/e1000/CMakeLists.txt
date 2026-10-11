
add_definitions(
    -DNDIS50_MINIPORT
    -DNDIS_MINIPORT_DRIVER
    -DNDIS_LEGACY_MINIPORT=1)

list(APPEND SOURCE
    # ndis.c  # replaced by Rust (sdk/lib/rust/e1000-rust/src/e1000/ndis.rs)
    # hardware.c  # replaced by Rust (sdk/lib/rust/e1000-rust/src/e1000/hardware.rs)
    # info.c  # replaced by Rust (sdk/lib/rust/e1000-rust/src/e1000/info.rs)
    # interrupt.c  # replaced by Rust (sdk/lib/rust/e1000-rust/src/e1000/interrupt.rs)
    nic.h
    e1000hw.h
    # debug.c  # replaced by Rust (sdk/lib/rust/e1000-rust/src/e1000/debug.rs)
    debug.h
    # send.c  # replaced by Rust (sdk/lib/rust/e1000-rust/src/e1000/send.rs)
    rust_cohab_stub.c
    )

# Every translation unit is Rust now (e1000-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(e1000_rust e1000-rust "" TRUE)
add_library(e1000 MODULE ${SOURCE} e1000.rc)
add_dependencies(e1000 rust_e1000_rust)
target_link_libraries(e1000 ${e1000_rust_LIB})
add_pch(e1000 nic.h SOURCE)
set_module_type(e1000 kernelmodedriver)
add_importlibs(e1000 ndis ntoskrnl hal)
add_cd_file(TARGET e1000 DESTINATION reactos/system32/drivers FOR all)
add_driver_inf(e1000 nete1000.inf)
