/*
 * ntprintf_trampoline.c — shared C-side adapter between variadic shims
 * and the Rust ntprintf crate.
 *
 * Each variadic shim (KdbPrintf, KdbpPrint, KdpDprintf, RtlStringCbPrintfA/W)
 * collects its `va_list ap` and calls one of the helpers below. The helper:
 *   1. Calls ntprintf_scan_arg_types(fmt, types, MAX) to learn the slot type
 *      for each '%' specifier.
 *   2. Walks `ap` once, pulling each slot at the correct width via va_arg.
 *   3. Calls ntprintf_format_a(dst, cap, fmt, slots, n) for the actual
 *      byte emission.
 *
 * No `_vsnprintf`. No format-string parsing in C. The C side only knows how
 * to pull a va_arg of a given primitive width.
 *
 * Tag values in `types[]` (defined by sdk/lib/rust/ntprintf/src/scanner.rs):
 *   'd' = 32-bit signed     -> va_arg(ap, int)
 *   'u' = 32-bit unsigned   -> va_arg(ap, unsigned int)
 *   'D' = 64-bit signed     -> va_arg(ap, long long)
 *   'U' = 64-bit unsigned   -> va_arg(ap, unsigned long long)
 *   'p' = pointer-width     -> va_arg(ap, void *)
 *   '*' = width/prec arg    -> va_arg(ap, int)
 */

#include <ntdef.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

/* Rust FFI: see sdk/lib/rust/ntprintf/src/lib.rs */
extern int      ntprintf_scan_arg_types(const unsigned char *fmt,
                                        unsigned char *out_types,
                                        size_t max);
extern size_t   ntprintf_format_a(unsigned char *dst, size_t cap,
                                  const unsigned char *fmt,
                                  const unsigned long long *slots, size_t n);

/* Hard cap on slot count. Format strings in the kernel use at most ~7 args;
 * 32 is a comfortable safety margin and keeps the stack frame small. */
#define NTPRINTF_MAX_SLOTS 32

/* Pull `ap` into a typed slot array sized by ntprintf_scan_arg_types(fmt).
 * Returns the count emitted (matches `n` returned by the scanner). The
 * caller passes a stack-allocated `slots[NTPRINTF_MAX_SLOTS]`. */
static int
ntprintf_pull_va(const unsigned char *fmt,
                 unsigned long long *slots,
                 va_list ap)
{
    unsigned char types[NTPRINTF_MAX_SLOTS];
    int n;
    int i;

    n = ntprintf_scan_arg_types(fmt, types, NTPRINTF_MAX_SLOTS);
    if (n < 0) return 0;

    for (i = 0; i < n; i++)
    {
        switch (types[i])
        {
            case 'd':
                slots[i] = (unsigned long long)(long long)(int)va_arg(ap, int);
                break;
            case 'u':
                slots[i] = (unsigned long long)(unsigned int)va_arg(ap, unsigned int);
                break;
            case 'D':
                slots[i] = (unsigned long long)va_arg(ap, long long);
                break;
            case 'U':
                slots[i] = (unsigned long long)va_arg(ap, unsigned long long);
                break;
            case 'p':
                slots[i] = (unsigned long long)(uintptr_t)va_arg(ap, void *);
                break;
            case '*':
                slots[i] = (unsigned long long)(long long)(int)va_arg(ap, int);
                break;
            default:
                slots[i] = 0;
                break;
        }
    }
    return n;
}

/* Public helper: format `fmt` + va_list into `dst[..cap]`. Returns the byte
 * count that would have been written (matches _vsnprintf semantics, including
 * truncation). NUL-terminates if cap > 0. */
size_t
ntprintf_va_format_a(char *dst, size_t cap, const char *fmt, va_list ap)
{
    unsigned long long slots[NTPRINTF_MAX_SLOTS];
    int n = ntprintf_pull_va((const unsigned char *)fmt, slots, ap);
    return ntprintf_format_a((unsigned char *)dst, cap,
                             (const unsigned char *)fmt,
                             slots, (size_t)n);
}

/* ------------------------------------------------------------------------- */
/* Wide (UTF-16) variant. Same shape, different scanner + formatter.        */
/* The va_arg pull loop is identical because tag bytes don't depend on the   */
/* format-string char width — both `'%d'` in ANSI and `'%' '\0' 'd' '\0'`    */
/* in UTF-16 produce the same `'d'` tag.                                     */
/* ------------------------------------------------------------------------- */

/* Rust FFI: see sdk/lib/rust/ntprintf_w/src/lib.rs */
extern int      ntprintf_w_scan_arg_types(const unsigned short *fmt,
                                          unsigned char *out_types,
                                          size_t max);
extern size_t   ntprintf_w_format(unsigned short *dst, size_t cap_chars,
                                  const unsigned short *fmt,
                                  const unsigned long long *slots, size_t n);

static int
ntprintf_w_pull_va(const unsigned short *fmt,
                   unsigned long long *slots,
                   va_list ap)
{
    unsigned char types[NTPRINTF_MAX_SLOTS];
    int n;
    int i;

    n = ntprintf_w_scan_arg_types(fmt, types, NTPRINTF_MAX_SLOTS);
    if (n < 0) return 0;

    for (i = 0; i < n; i++)
    {
        switch (types[i])
        {
            case 'd':
                slots[i] = (unsigned long long)(long long)(int)va_arg(ap, int);
                break;
            case 'u':
                slots[i] = (unsigned long long)(unsigned int)va_arg(ap, unsigned int);
                break;
            case 'D':
                slots[i] = (unsigned long long)va_arg(ap, long long);
                break;
            case 'U':
                slots[i] = (unsigned long long)va_arg(ap, unsigned long long);
                break;
            case 'p':
                slots[i] = (unsigned long long)(uintptr_t)va_arg(ap, void *);
                break;
            case '*':
                slots[i] = (unsigned long long)(long long)(int)va_arg(ap, int);
                break;
            default:
                slots[i] = 0;
                break;
        }
    }
    return n;
}

/* Public helper for wide variadic trampolines (RtlStringCbPrintfW, swprintf).
 *
 * `cap_chars` is **wide-character count**, not bytes. Callers with a byte
 * size (`RtlStringCbPrintfW`'s `cb`) divide by `sizeof(WCHAR)` first.
 * Returns the wide-character count that would have been written, matching
 * `_vsnwprintf` semantics. NUL-terminates with one wchar if cap_chars > 0. */
size_t
ntprintf_va_format_w(unsigned short *dst, size_t cap_chars,
                     const unsigned short *fmt, va_list ap)
{
    unsigned long long slots[NTPRINTF_MAX_SLOTS];
    int n = ntprintf_w_pull_va(fmt, slots, ap);
    return ntprintf_w_format(dst, cap_chars, fmt, slots, (size_t)n);
}
