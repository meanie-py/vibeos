
include_directories(include ${REACTOS_SOURCE_DIR}/drivers/bus/acpi/acpica/include)

list(APPEND HAL_ACPI_SOURCE
    acpi/halacpi.c
    acpi/halpnpdd.c
    # acpi/busemul.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/busemul.rs)
    legacy/bus/pcibus.c)

# acpi/madt.c — replaced by Rust for amd64 only
# (sdk/lib/rust/hal-rust/src/x86/madt.rs is cfg-gated to x86_64).
# i386 keeps the C version because halacpi.dll links the C side and PIC
# hal.dll would otherwise pull madt.rs's HalpParseApicTables into the
# same CGU as commonly-used symbols, leaving HalAcpiGetTable unresolved.
if(ARCH STREQUAL "i386")
    list(APPEND HAL_ACPI_SOURCE acpi/madt.c)
endif()

# Needed to compile while using ACPICA
if(ARCH STREQUAL "amd64")
    add_definitions(-DWIN64)
endif()

add_library(lib_hal_acpi OBJECT ${HAL_ACPI_SOURCE})
add_pch(lib_hal_acpi include/hal.h ${HAL_ACPI_SOURCE})
add_dependencies(lib_hal_acpi bugcodes xdk)
