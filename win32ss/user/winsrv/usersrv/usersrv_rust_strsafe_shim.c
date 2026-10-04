/*
 * usersrv_rust_strsafe_shim.c
 *
 * Non-inline providers for the ntstrsafe / rtl FORCEINLINE helpers that the
 * Rust usersrv translation (sdk/lib/rust/winsrv-rust/src/usersrv/harderror.rs)
 * references.  These functions are inline-only in the C SDK headers
 * (ntstrsafe.h / ntddk.h), so they have no exported symbol — the Rust
 * cohabitation link has no provider for them.  We re-implement the small
 * subset that harderror.rs needs, on top of libcntpr's _vsnwprintf.
 *
 * Two of them (RtlStringCbPrintfW / RtlStringCbPrintfExW) are C-variadic and
 * therefore CANNOT be expressed as Rust function definitions on stable Rust
 * (see memory feedback_degraded_variadic_stubs) — a C shim is the canonical
 * fix.  The other two are kept here too so all four live in one place.
 *
 * Self-contained (no SDK headers) on purpose: including ntstrsafe.h / ntddk.h
 * would re-declare these names as `static __forceinline`, which collides with
 * the external definitions below.  The ABI types used are layout-stable.
 */

#include <stdarg.h>

typedef long            NTSTATUS;
typedef unsigned short  WCHAR;
typedef WCHAR          *PWCHAR;
typedef unsigned short  USHORT;
typedef unsigned long   ULONG;
typedef __SIZE_TYPE__   SIZE_T;   /* pointer-width size_t — MUST match Rust usize
                                   * (4 bytes on i386, 8 on amd64/arm64). Using a
                                   * fixed `unsigned int` here silently truncated
                                   * the byte-counts on 64-bit. */

typedef struct _UNICODE_STRING {
    USHORT Length;
    USHORT MaximumLength;
    PWCHAR Buffer;
} UNICODE_STRING, *PUNICODE_STRING;

#define STATUS_SUCCESS            ((NTSTATUS)0x00000000L)
#define STATUS_BUFFER_OVERFLOW    ((NTSTATUS)0x80000005L)
#define STATUS_INVALID_PARAMETER  ((NTSTATUS)0xC000000DL)

/* `_InterlockedIncrement` / `_InterlockedDecrement` / `IsListEmpty` for the
 * consrv Rust members: winsrv-rust consrv/shim.rs, on every arch. (A 64-bit C
 * copy lived here and won the link by order alone.) */

/* libcntpr — wide vsnprintf; writes at most `count` wchars (incl. NUL on
 * success), returns the number written, or a negative value on truncation. */
extern int __cdecl _vsnwprintf(WCHAR *buf, SIZE_T count, const WCHAR *fmt, va_list ap);

/* ntddk.h: RtlInitEmptyUnicodeString(DestinationString, Buffer, BufferSize) */
void __cdecl
RtlInitEmptyUnicodeString(PUNICODE_STRING DestinationString,
                          PWCHAR          Buffer,
                          USHORT          BufferSize)
{
    DestinationString->Length        = 0;
    DestinationString->MaximumLength  = BufferSize;
    DestinationString->Buffer         = Buffer;
}

/* ntstrsafe.h: RtlStringCbCopyNW — stdcall, byte counts.  Copies at most
 * cbSrc bytes from pszSrc into pszDest (which is cbDest bytes), stopping at
 * the source NUL, always NUL-terminating. */
NTSTATUS __stdcall
RtlStringCbCopyNW(PWCHAR pszDest, SIZE_T cbDest, const WCHAR *pszSrc, SIZE_T cbSrc)
{
    SIZE_T cchDest   = cbDest / sizeof(WCHAR);
    SIZE_T cchToCopy = cbSrc  / sizeof(WCHAR);
    SIZE_T i = 0;

    if (cchDest == 0)
        return STATUS_INVALID_PARAMETER;

    while (i < cchToCopy && i < (cchDest - 1) && pszSrc[i] != 0) {
        pszDest[i] = pszSrc[i];
        i++;
    }
    pszDest[i] = 0;

    /* Overflow iff there was more source to copy but no destination room. */
    if (i == (cchDest - 1) && i < cchToCopy && pszSrc[i] != 0)
        return STATUS_BUFFER_OVERFLOW;
    return STATUS_SUCCESS;
}

/* ntstrsafe.h: RtlStringCbPrintfW — cdecl variadic, byte count for dest. */
NTSTATUS __cdecl
RtlStringCbPrintfW(PWCHAR pszDest, SIZE_T cbDest, const WCHAR *pszFormat, ...)
{
    SIZE_T  cchDest = cbDest / sizeof(WCHAR);
    va_list ap;
    int     n;

    if (cchDest == 0)
        return STATUS_INVALID_PARAMETER;

    va_start(ap, pszFormat);
    n = _vsnwprintf(pszDest, cchDest, pszFormat, ap);
    va_end(ap);

    pszDest[cchDest - 1] = 0;
    if (n < 0 || (SIZE_T)n >= cchDest)
        return STATUS_BUFFER_OVERFLOW;
    return STATUS_SUCCESS;
}

/* ntstrsafe.h: RtlStringCbPrintfExW — cdecl variadic with end/remaining out
 * params and flags.  Flags are ignored (harderror.rs passes 0). */
NTSTATUS __cdecl
RtlStringCbPrintfExW(PWCHAR  pszDest,
                     SIZE_T  cbDest,
                     PWCHAR *ppszDestEnd,
                     SIZE_T *pcbRemaining,
                     ULONG   dwFlags,
                     const WCHAR *pszFormat, ...)
{
    SIZE_T  cchDest = cbDest / sizeof(WCHAR);
    va_list ap;
    int     n;
    SIZE_T  used;

    (void)dwFlags;

    if (cchDest == 0)
        return STATUS_INVALID_PARAMETER;

    va_start(ap, pszFormat);
    n = _vsnwprintf(pszDest, cchDest, pszFormat, ap);
    va_end(ap);

    pszDest[cchDest - 1] = 0;

    if (n < 0 || (SIZE_T)n >= cchDest) {
        used = cchDest - 1;
        if (ppszDestEnd)  *ppszDestEnd  = pszDest + used;
        if (pcbRemaining) *pcbRemaining = sizeof(WCHAR);   /* room for the NUL */
        return STATUS_BUFFER_OVERFLOW;
    }

    used = (SIZE_T)n;
    if (ppszDestEnd)  *ppszDestEnd  = pszDest + used;
    if (pcbRemaining) *pcbRemaining = (cchDest - used) * sizeof(WCHAR);
    return STATUS_SUCCESS;
}
