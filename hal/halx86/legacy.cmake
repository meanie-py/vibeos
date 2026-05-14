
list(APPEND HAL_LEGACY_SOURCE
    # legacy/bus/bushndlr.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/bushndlr.rs)
    # legacy/bus/cmosbus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/cmosbus.rs)
    # legacy/bus/isabus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/isabus.rs)
    legacy/bus/pcibus.c    # Rust replacement at sdk/lib/rust/hal-rust/src/x86/pcibus.rs
                            # is compile-clean and link-clean (cohabits via
                            # --allow-multiple-definition); final swap blocked
                            # on porting the 8 file-scope statics from pcibus.c
                            # to pcibus.rs.  See docs/engineering/handoff-pcibus-integration.md.
    ${CMAKE_CURRENT_BINARY_DIR}/pci_classes.c
    ${CMAKE_CURRENT_BINARY_DIR}/pci_vendors.c
    # legacy/bus/sysbus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/sysbus.rs)
    # legacy/bussupp.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/bussupp.rs)
    # legacy/halpnpdd.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/legacy_halpnpdd.rs)
    # legacy/halpcat.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/halpcat.rs)
    )

add_library(lib_hal_legacy OBJECT ${HAL_LEGACY_SOURCE})
add_dependencies(lib_hal_legacy bugcodes xdk)
#add_pch(lib_hal_legacy include/hal.h)
