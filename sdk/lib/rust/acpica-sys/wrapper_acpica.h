/*
 * acpica-sys bindgen wrapper — the public ACPICA interface used by the ReactOS
 * acpi driver glue (main/pnp/power/buspdo/eval/interface/acpienum + busmgr).
 *
 * acpi.h is ACPICA's master public include: it pulls actypes.h (ACPI_STATUS,
 * ACPI_HANDLE, ACPI_OBJECT, ACPI_BUFFER, the table/resource types), acpixf.h
 * (the outbound Acpi* API), acpiosxf.h (the AcpiOs* OSL callback prototypes +
 * the ACPI_OSD_* handler typedefs the Rust OSL impls must match), and
 * acexcep.h (AE_* status codes). On __REACTOS__ acpi.h first includes ntddk.h.
 */
#include <acpi.h>
