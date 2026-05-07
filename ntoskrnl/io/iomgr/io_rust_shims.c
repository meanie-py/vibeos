/*
 * io_rust_shims.c — surviving C-side helpers for the Rust IO subsystem.
 *
 * Most of what used to live here has been promoted to Rust impls in
 * `ntoskrnl-rust` or to the sibling `io-inlines` rlib (see
 * `docs/engineering/shim-elimination.md`); the per-trampoline removal
 * commits live in `git log --oneline -- ntoskrnl/io/iomgr/io_rust_shims.c`.
 *
 * What's left:
 *   - `RtlUnicodeStringPrintf_4wZ_shim` — variadic wrapper for the fixed
 *     "%wZ%wZ#%wZ%wZ" format used by deviface.rs (Rust can't safely
 *     cross the `va_arg` boundary on stable).
 *   - `ProbeForWriteHandle` / `ProbeForReadLargeInteger` — wrap probe.h
 *     macros and provide the manual range-check + volatile copy. Kept
 *     here until the corresponding `common-inlines` impls grow the
 *     same `MmUserProbeAddress` bound check (see TODO in that crate).
 */

#include <ntoskrnl.h>
#include <ntstrsafe.h>
#define NDEBUG
#include <debug.h>
#include "internal/io_i.h"

/*
 * Probe helper macros from probe.h — Rust extern "system" needs function wrappers.
 * The C body adds an explicit `MmUserProbeAddress` overflow/upper-bound check
 * that the `common-inlines` impls currently elide; on i386 the NTAPI symbol
 * decoration (`_X@N`) is distinct from the cdecl version so both coexist.
 */
#undef ProbeForWriteHandle
VOID
NTAPI
ProbeForWriteHandle(IN PHANDLE Ptr)
{
    ProbeForWrite(Ptr, sizeof(HANDLE), sizeof(HANDLE));
}

#undef ProbeForReadLargeInteger
LARGE_INTEGER
NTAPI
ProbeForReadLargeInteger(IN const LARGE_INTEGER *Ptr)
{
    LARGE_INTEGER Result;
    if ((ULONG_PTR)Ptr + sizeof(LARGE_INTEGER) < (ULONG_PTR)Ptr ||
        (ULONG_PTR)Ptr + sizeof(LARGE_INTEGER) > (ULONG_PTR)MmUserProbeAddress) {
        ExRaiseAccessViolation();
    }
    Result = *(volatile const LARGE_INTEGER *)Ptr;
    return Result;
}

/*
 * RtlUnicodeStringPrintf_4wZ — wrapper for RtlUnicodeStringPrintf with
 * format "%wZ%wZ#%wZ%wZ" and 4 PUNICODE_STRING args.
 * RtlUnicodeStringPrintf is FORCEINLINE from ntstrsafe.h (variadic).
 */
NTSTATUS
NTAPI
RtlUnicodeStringPrintf_4wZ_shim(
    PUNICODE_STRING DestinationString,
    PUNICODE_STRING Arg1,
    PUNICODE_STRING Arg2,
    PUNICODE_STRING Arg3,
    PUNICODE_STRING Arg4)
{
    return RtlUnicodeStringPrintf(DestinationString,
                                  L"%wZ%wZ#%wZ%wZ",
                                  Arg1, Arg2, Arg3, Arg4);
}
