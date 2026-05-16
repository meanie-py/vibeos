/*
 * PROJECT:     ReactOS Win32k
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Win32k C-side pre-translation: SEH-free user-buffer validation.
 *
 * Pass 1 probe-site lowering shim. See safe_user_io.h and
 * docs/engineering/win32k-pretranslation-audit.md.
 */

#include <win32k.h>
#include <safe_user_io.h>

/*
 * Validate that [Address, Address+Length) lies entirely below
 * MmUserProbeAddress and satisfies the requested alignment, without
 * raising and without touching the page. Identical range/alignment
 * arithmetic to the DDK ProbeForRead/ProbeForWrite macros and to
 * ntkrnl-sys::probe::probe_user_buffer, minus the exception/touch.
 */
/* NB: FORCEINLINE expands to `extern inline __attribute__((gnu_inline))`
 * with this toolchain, which cannot combine with `static` (invalid C).
 * Both callers are in this TU, so gnu_inline alone is sufficient. */
NTSTATUS
FORCEINLINE
SafepValidateUserRange(
    const volatile VOID *Address,
    SIZE_T Length,
    ULONG Alignment)
{
    ULONG_PTR Start, Last;

    /* DDK ProbeFor* is a no-op for zero length */
    if (Length == 0)
        return STATUS_SUCCESS;

    /* Alignment is a byte count and must be a power of two (DDK contract) */
    if (Alignment > 1 && ((ULONG_PTR)Address & (ULONG_PTR)(Alignment - 1)) != 0)
        return STATUS_DATATYPE_MISALIGNMENT;

    Start = (ULONG_PTR)Address;
    Last  = Start + Length - 1;

    /* Address-space wrap, or any byte at/above the user probe boundary */
    if (Last < Start || Last >= (ULONG_PTR)MmUserProbeAddress)
        return STATUS_ACCESS_VIOLATION;

    return STATUS_SUCCESS;
}

NTSTATUS
NTAPI
SafeProbeForRead(
    const volatile VOID *Address,
    SIZE_T Length,
    ULONG Alignment)
{
    return SafepValidateUserRange(Address, Length, Alignment);
}

NTSTATUS
NTAPI
SafeProbeForWrite(
    volatile VOID *Address,
    SIZE_T Length,
    ULONG Alignment)
{
    return SafepValidateUserRange(Address, Length, Alignment);
}
