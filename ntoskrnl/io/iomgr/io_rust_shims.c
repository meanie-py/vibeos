/*
 * io_rust_shims.c — surviving C-side helpers for the Rust IO subsystem.
 *
 * Most of what used to live here has been promoted to Rust impls in
 * `ntoskrnl-rust` or to the sibling `io-inlines` rlib (see
 * `docs/engineering/shim-elimination.md`); the per-trampoline removal
 * commits live in `git log --oneline -- ntoskrnl/io/iomgr/io_rust_shims.c`.
 *
 * What's left:
 *   - `RtlUnicodeStringPrintf_4wZ_shim` — fixed-arity wrapper for the
 *     hard-coded "%wZ%wZ#%wZ%wZ" format used by deviface.rs. Originally
 *     called the variadic `RtlUnicodeStringPrintf` from <ntstrsafe.h>;
 *     the format is fixed so we open-code the concat to drop the
 *     dependency. (Rust can't safely cross the `va_arg` boundary on
 *     stable, so the shim itself stays — the symbol is what matters.)
 *
 * Header-free since the cargo-driven build PoC: no `<ntoskrnl.h>`, no
 * `<ntstrsafe.h>`, no `<debug.h>`, no `<internal/io_i.h>`. Local
 * typedefs + extern decls of the few functions/globals we need cover
 * everything.
 */

#include <stddef.h>
#include <stdint.h>
#include <string.h>

/* --- Local NT type aliases (replaces <ntoskrnl.h>) ------------------ */
typedef void              VOID;
typedef void             *PVOID;
typedef long              LONG;
typedef unsigned long     ULONG;
typedef unsigned short    USHORT;
typedef unsigned char     UCHAR;
typedef wchar_t           WCHAR;
typedef WCHAR            *PWCHAR;
typedef LONG              NTSTATUS;
typedef uintptr_t         ULONG_PTR;
typedef void             *HANDLE;
typedef HANDLE           *PHANDLE;

#define NTAPI             __attribute__((stdcall))
#if defined(_M_AMD64) || defined(_M_ARM64)
# undef  NTAPI
# define NTAPI
#endif
#define IN

#define STATUS_SUCCESS              ((NTSTATUS)0x00000000L)
#define STATUS_INVALID_PARAMETER    ((NTSTATUS)0xC000000DL)
#define STATUS_BUFFER_OVERFLOW      ((NTSTATUS)0x80000005L)

typedef union _LARGE_INTEGER
{
    struct { ULONG LowPart; LONG HighPart; };
    struct { ULONG LowPart_; LONG HighPart_; } u;
    int64_t QuadPart;
} LARGE_INTEGER;

typedef struct _UNICODE_STRING
{
    USHORT Length;
    USHORT MaximumLength;
    PWCHAR Buffer;
} UNICODE_STRING, *PUNICODE_STRING;

/*
 * ProbeForWriteHandle / ProbeForReadLargeInteger are ntoskrnl-rust's
 * (common_inlines.rs over ntkrnl-sys inline::common, which carries probe.h's
 * MmUserProbeAddress check). Copies here were duplicate symbols that won the
 * amd64/arm64 link by order; tools/link_guard.py refuses those.
 */

/*
 * RtlUnicodeStringPrintf_4wZ — open-coded "%wZ%wZ#%wZ%wZ" concat for
 * deviface.rs. The original called `RtlUnicodeStringPrintf` from
 * <ntstrsafe.h>; we replicate just the buffer-bounded append behaviour
 * so the shim no longer needs the strsafe header chain.
 */
static int
io_append_unicode(PUNICODE_STRING dest, const UNICODE_STRING *src)
{
    USHORT need = dest->Length + src->Length;
    if (need < dest->Length || need > dest->MaximumLength)
        return 0; /* overflow */
    memcpy((char *)dest->Buffer + dest->Length, src->Buffer, src->Length);
    dest->Length = need;
    return 1;
}

static int
io_append_wchar(PUNICODE_STRING dest, WCHAR ch)
{
    USHORT need = dest->Length + (USHORT)sizeof(WCHAR);
    if (need < dest->Length || need > dest->MaximumLength)
        return 0;
    *(WCHAR *)((char *)dest->Buffer + dest->Length) = ch;
    dest->Length = need;
    return 1;
}

NTSTATUS
NTAPI
RtlUnicodeStringPrintf_4wZ_shim(
    PUNICODE_STRING DestinationString,
    PUNICODE_STRING Arg1,
    PUNICODE_STRING Arg2,
    PUNICODE_STRING Arg3,
    PUNICODE_STRING Arg4)
{
    if (DestinationString == NULL || Arg1 == NULL || Arg2 == NULL ||
        Arg3 == NULL || Arg4 == NULL)
        return STATUS_INVALID_PARAMETER;

    /* Reset Length so we always start from the buffer head, matching
     * RtlUnicodeStringPrintf semantics. */
    DestinationString->Length = 0;

    if (!io_append_unicode(DestinationString, Arg1) ||
        !io_append_unicode(DestinationString, Arg2) ||
        !io_append_wchar(DestinationString, L'#') ||
        !io_append_unicode(DestinationString, Arg3) ||
        !io_append_unicode(DestinationString, Arg4))
        return STATUS_BUFFER_OVERFLOW;

    return STATUS_SUCCESS;
}
