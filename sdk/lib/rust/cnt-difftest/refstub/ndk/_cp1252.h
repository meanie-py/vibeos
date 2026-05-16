/* Minimal NT type + NLS-Rtl* stub for isolating the libcntpr
 * multibyte wrappers (mbstowcs_nt.c / wcstombs_nt.c / toupper_nt_mb.c)
 * on the host. The Rtl* model here MUST mirror core_rs/mbcp.rs
 * exactly — default single-byte CP1252 — so the differential test
 * pins the genuine wrapper logic, deterministically. Rtl* are
 * `static` so each ref TU gets a private copy (no link clash; not
 * exported — the diff only calls the renamed ref_* wrappers). */
#ifndef CNT_CP1252_H
#define CNT_CP1252_H
#include <stddef.h>

typedef long           NTSTATUS;
typedef unsigned long  ULONG;
typedef unsigned long *PULONG;
typedef unsigned char  UCHAR;
typedef unsigned char *PUCHAR;
typedef unsigned short WCHAR;
typedef WCHAR         *PWCHAR;
typedef WCHAR         *PWCH;
typedef char           CHAR;
typedef char          *PCHAR;
typedef int            BOOL;
typedef unsigned char  BOOLEAN;
typedef void           VOID;

#ifndef NT_SUCCESS
#define NT_SUCCESS(x) ((NTSTATUS)(x) >= 0)
#endif
#ifndef min
#define min(a, b) ((a) < (b) ? (a) : (b))
#endif
/* SAL + calling-convention annotations the .c use */
#define _Check_return_
#define _In_
#define _In_z_
#define _In_opt_
#define _Out_
#ifndef __cdecl
#define __cdecl
#endif

/* ---- CP1252 model — keep byte-identical to core_rs/mbcp.rs ------- */
static const unsigned short cnt_cp1252_high[32] = {
    0x20AC, 0x0081, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021,
    0x02C6, 0x2030, 0x0160, 0x2039, 0x0152, 0x008D, 0x017D, 0x008F,
    0x0090, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014,
    0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0x009D, 0x017E, 0x0178,
};
static unsigned short cnt_dec(unsigned char b) {
    return (b >= 0x80 && b < 0xA0) ? cnt_cp1252_high[b - 0x80]
                                   : (unsigned short)b;
}
static int cnt_enc(unsigned short w) {
    int i;
    if (w < 0x80 || (w >= 0xA0 && w <= 0xFF))
        return (int)(unsigned char)w;
    for (i = 0; i < 32; i++)
        if (cnt_cp1252_high[i] == w)
            return 0x80 + i;
    return -1;
}
static unsigned short cnt_upcase(unsigned short w) {
    if (w >= 0x61 && w <= 0x7A) return w - 0x20;
    if ((w >= 0xE0 && w <= 0xF6) || (w >= 0xF8 && w <= 0xFE)) return w - 0x20;
    if (w == 0xFF) return 0x0178;
    if (w == 0xB5) return 0x039C;
    if (w == 0x0161) return 0x0160;
    if (w == 0x0153) return 0x0152;
    if (w == 0x017E) return 0x017D;
    return w;
}

static WCHAR RtlAnsiCharToUnicodeChar(PUCHAR *ps) {
    unsigned char b = **ps;
    *ps += 1; /* single-byte CP: no lead bytes */
    return cnt_dec(b);
}
static WCHAR RtlUpcaseUnicodeChar(WCHAR w) { return cnt_upcase(w); }
static NTSTATUS RtlMultiByteToUnicodeSize(PULONG sz, const char *s, ULONG n) {
    (void)s;
    *sz = n * 2;
    return 0;
}
static NTSTATUS RtlMultiByteToUnicodeN(WCHAR *d, ULONG db, PULONG ret,
                                       const char *s, ULONG sb) {
    ULONG cap = db / 2, c = sb < cap ? sb : cap, i;
    for (i = 0; i < c; i++)
        d[i] = cnt_dec((unsigned char)s[i]);
    if (ret) *ret = c * 2;
    return 0;
}
static NTSTATUS RtlUnicodeToMultiByteSize(PULONG sz, const WCHAR *s, ULONG sb) {
    (void)s;
    *sz = sb / 2;
    return 0;
}
static NTSTATUS RtlUnicodeToMultiByteN(char *d, ULONG db, PULONG ret,
                                       const WCHAR *s, ULONG sb) {
    ULONG sw = sb / 2, c = sw < db ? sw : db, i;
    for (i = 0; i < c; i++) {
        int e = cnt_enc(s[i]);
        d[i] = (char)(e < 0 ? '?' : e);
    }
    if (ret) *ret = c;
    return 0;
}
static size_t cnt_wcslen(const WCHAR *s) {
    size_t n = 0;
    while (s[n]) n++;
    return n;
}
#define wcslen cnt_wcslen
#endif /* CNT_CP1252_H */
