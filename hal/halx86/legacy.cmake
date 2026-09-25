
list(APPEND HAL_LEGACY_SOURCE
    # legacy/bus/bushndlr.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/bushndlr.rs)
    # legacy/bus/cmosbus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/cmosbus.rs)
    # legacy/bus/isabus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/isabus.rs)
    # legacy/bus/pcibus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/pcibus.rs)
    # pci_classes.c / pci_vendors.c (native-bin2c BINSTR of the .ids files)
    #   — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/pcidata.rs, which
    #   include_bytes! the .ids directly; pcidata.cmake/bin2c no longer used)
    # legacy/bus/sysbus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/sysbus.rs)
    # legacy/bussupp.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/bussupp.rs)
    # legacy/halpnpdd.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/legacy_halpnpdd.rs)
    # legacy/halpcat.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/halpcat.rs)
    )

# All HAL legacy sources are now in Rust (sdk/lib/rust/hal-rust). Keep the
# `legacy` component alive as a no-op OBJECT lib so add_hal(COMPONENTS ...
# legacy ...) keeps working without a conditional through CMakeLists.txt
# (same pattern as acpi.cmake / generic.cmake).
set(HAL_LEGACY_STUB ${CMAKE_CURRENT_BINARY_DIR}/lib_hal_legacy_empty.c)
file(WRITE ${HAL_LEGACY_STUB} "/* placeholder: HAL legacy sources are in hal-rust */\n")
list(APPEND HAL_LEGACY_SOURCE ${HAL_LEGACY_STUB})

add_library(lib_hal_legacy OBJECT ${HAL_LEGACY_SOURCE})
#add_pch(lib_hal_legacy include/hal.h)
add_dependencies(lib_hal_legacy bugcodes xdk)
