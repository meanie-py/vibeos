
include_directories(include ${REACTOS_SOURCE_DIR}/vendor/acpica/include)

list(APPEND HAL_ACPI_SOURCE
    # acpi/halacpi.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/halacpi.rs)
    # acpi/halpnpdd.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/halpnpdd.rs)
    # acpi/busemul.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/busemul.rs)
    # legacy/bus/pcibus.c — replaced by Rust (sdk/lib/rust/hal-rust/src/x86/pcibus.rs)
    )

# Needed to compile while using ACPICA
if(ARCH STREQUAL "amd64")
    add_definitions(-DWIN64)
endif()

# All HAL ACPI sources are now in Rust (sdk/lib/rust/hal-rust). The
# `acpi` component is intentionally a no-op OBJECT lib — keeping the
# target alive lets add_hal(COMPONENTS ... acpi ...) keep working
# without having to plumb a conditional through CMakeLists.txt. We
# create the library with a tiny placeholder .c so add_pch has a
# source to attach to.
set(HAL_ACPI_STUB ${CMAKE_CURRENT_BINARY_DIR}/lib_hal_acpi_empty.c)
file(WRITE ${HAL_ACPI_STUB} "/* placeholder: HAL ACPI sources are in hal-rust */\n")
list(APPEND HAL_ACPI_SOURCE ${HAL_ACPI_STUB})

add_library(lib_hal_acpi OBJECT ${HAL_ACPI_SOURCE})
add_pch(lib_hal_acpi include/hal.h ${HAL_ACPI_SOURCE})
add_dependencies(lib_hal_acpi bugcodes xdk)
