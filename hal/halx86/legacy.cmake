
list(APPEND HAL_LEGACY_SOURCE
    # legacy/bus/bushndlr.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/bushndlr.rs)
    # legacy/bus/cmosbus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/cmosbus.rs)
    # legacy/bus/isabus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/isabus.rs)
    legacy/bus/pcibus.c
    ${CMAKE_CURRENT_BINARY_DIR}/pci_classes.c
    ${CMAKE_CURRENT_BINARY_DIR}/pci_vendors.c
    # legacy/bus/sysbus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/sysbus.rs)
    legacy/bussupp.c
    legacy/halpnpdd.c
    # legacy/halpcat.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/halpcat.rs)
    )

add_library(lib_hal_legacy OBJECT ${HAL_LEGACY_SOURCE})
add_dependencies(lib_hal_legacy bugcodes xdk)
#add_pch(lib_hal_legacy include/hal.h)
