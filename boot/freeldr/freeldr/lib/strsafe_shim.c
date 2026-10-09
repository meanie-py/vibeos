/*
 * FreeLoader: Rust → C string-safety shim
 *
 * The Rust freeldr-rust crate calls RtlStringCbPrintfA / RtlStringCbCopyA
 * across many sites (winldr, tui, wlregistry, machpc, hwdisk, pchw, etc.).
 * ntstrsafe.h emits these as `static __inline` (NTSTRSAFEVAPI), so there
 * is no out-of-line C symbol in non-MSVC builds, and Rust stable cannot
 * define a variadic C function (no stable c_variadic on the def side).
 *
 * rustconv's degraded stub in ffi.rs::RtlStringCbPrintfA was just
 * `RtlStringCbCopyA(dest, cb, fmt)` — the format string was copied
 * verbatim with %u / %s / %x left unformatted. Boot path strings like
 * "multi(0)disk(0)cdrom(%u)" reached ArcOpen with the %u literal,
 * preventing rosload.exe from loading.
 *
 * Fix: emit the real out-of-line `RtlStringCbPrintfA` symbol. Implementation
 * delegates to libcntpr's `_vsnprintf` and mirrors ntstrsafe semantics
 * (NUL-terminate, truncation = STATUS_BUFFER_OVERFLOW). Same pattern as
 * sdk/lib/rtl/strsafe.c in tree.
 */

/* IMPORTANT: do NOT include freeldr.h or ntstrsafe.h here — they define
 * `static __inline NTSTATUS RtlStringCbPrintfA(...)` (NTSTRSAFEVAPI),
 * which conflicts with the out-of-line definition below. Pull in only
 * the minimal headers we need. */
typedef long NTSTATUS;
typedef unsigned int size_t_compat;
#define NULL ((void *)0)
#define STATUS_SUCCESS              ((NTSTATUS)0x00000000L)
#define STATUS_INVALID_PARAMETER    ((NTSTATUS)0xC000000DL)
#define STATUS_BUFFER_OVERFLOW      ((NTSTATUS)0x80000005L)

typedef __builtin_va_list va_list;
#define va_start(ap, fmt) __builtin_va_start(ap, fmt)
#define va_end(ap)        __builtin_va_end(ap)

/* libcntpr provides _vsnprintf (sdk/lib/crt/printf/_vsnprintf.c). */
int __cdecl _vsnprintf(char *str, unsigned int size, const char *fmt, va_list ap);

NTSTATUS __cdecl
RtlStringCbPrintfA(
    char *pszDest,
    unsigned int cbDest,
    const char *pszFormat,
    ...)
{
    va_list args;
    int n;

    if (pszDest == NULL || cbDest == 0 || pszFormat == NULL)
        return STATUS_INVALID_PARAMETER;

    va_start(args, pszFormat);
    n = _vsnprintf(pszDest, cbDest, pszFormat, args);
    va_end(args);

    /* Always NUL-terminate. */
    pszDest[cbDest - 1] = '\0';

    if (n < 0 || (unsigned int)n >= cbDest)
        return STATUS_BUFFER_OVERFLOW;
    return STATUS_SUCCESS;
}

/* libcntpr provides _vsnwprintf (sdk/lib/crt/printf/_vsnwprintf.c). Its count is in
 * characters; wchar_t is 16-bit on every PE target. */
int __cdecl _vsnwprintf(unsigned short *str, __SIZE_TYPE__ count, const unsigned short *fmt,
                        va_list ap);

/* The wide twin. setupldr formats the boot drivers' names and paths with it. */
NTSTATUS __cdecl
RtlStringCbPrintfW(
    unsigned short *pszDest,
    __SIZE_TYPE__ cbDest,
    const unsigned short *pszFormat,
    ...)
{
    va_list args;
    __SIZE_TYPE__ cchDest = cbDest / sizeof(unsigned short);
    int n;

    if (pszDest == NULL || cchDest == 0 || pszFormat == NULL)
        return STATUS_INVALID_PARAMETER;

    va_start(args, pszFormat);
    n = _vsnwprintf(pszDest, cchDest, pszFormat, args);
    va_end(args);

    pszDest[cchDest - 1] = 0;

    if (n < 0 || (__SIZE_TYPE__)n >= cchDest)
        return STATUS_BUFFER_OVERFLOW;
    return STATUS_SUCCESS;
}

/*
 * RtlZeroMemory: on i386, sdk/lib/rtl/i386/rtlmem.s in libblrtl.a provides
 * the stdcall `_RtlZeroMemory@8`. amd64/arm64 have no blrtl-side
 * RtlZeroMemory implementation, yet freeldr-rust references it (NTAPI).
 * Provide a portable C definition for those arches. The Rust extern decl
 * is `extern "system"` which is cdecl on non-i386, so this matches.
 */
#if !defined(__i386__) && !defined(_M_IX86)
void RtlZeroMemory(void *dst, unsigned long len)
{
    unsigned char *p = (unsigned char *)dst;
    while (len--)
        *p++ = 0;
}
#endif
