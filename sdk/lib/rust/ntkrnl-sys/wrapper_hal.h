/* Wrapper header for bindgen — HAL-private types + ACPI tables.
 *
 * Two header trees:
 *   1. hal/halx86/include/hal.h  — HAL precompiled header.  Transitively
 *      pulls in bus.h, halirq.h, haldma.h, halhw.h, halp.h, mps.h, halacpi.h
 *      plus the kernel internal headers it needs (ntifs, NDK, internal/i386
 *      or internal/amd64 MM/KE).
 *   2. vendor/acpica/include/acpi.h — ACPICA umbrella.  Pulls in
 *      actypes, actbl, actbl1, actbl2, actbl3 with all the spec table
 *      layouts (FADT, MADT, RSDT, XSDT, SRAT, etc.).
 *
 * Arch defines come from generate_hal_bindings.sh per-arch invocation.
 * Only x86 arches (i386, amd64) are bindgen'd here — arm64 HAL is C-stub
 * only and uses standard NDK types already in wrapper_ndk.h coverage.
 */
#define _NTHAL_
#define _NTSYSTEM_
#define _USE_DUMMY_PSEH 1
#define __REACTOS__

/* Skip intrin.h — compiler intrinsics not available in libclang */
#define _INTRIN_H_
#define _INTRIN_X86_H_

/* __int64 needed by various headers */
#ifndef __int64
#define __int64 long long
#endif

/* Prevent FORCEINLINE body expansion — bindgen can't handle inline bodies */
#undef FORCEINLINE
#define FORCEINLINE

/* HAL precompiled header — pulls in ntifs, NDK, hal/halx86/include/*.h
 * and the per-arch internal/{i386,amd64}/{ke,mm}.h. */
#include "hal/halx86/include/hal.h"

/* ACPICA umbrella — table layouts (FADT, MADT, RSDT, XSDT, etc.).
 * Pulls in actypes, actbl, actbl1/2/3, plus the OSL/AML interpreter
 * external interface declarations (acpiosxf, acpixf) — those externs
 * generate FFI we don't use, but that's harmless. */
#include "vendor/acpica/include/acpi.h"
