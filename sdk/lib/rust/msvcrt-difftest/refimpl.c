/*
 * refimpl.c — the externals of the genuine dll/win32/msvcrt/string.c.
 *
 * The real string.c.obj (i386) references exactly 19 undefined symbols:
 *
 *   floor, log10, malloc, memmove, _vsnprintf, _alloca, __divdi3, __udivdi3
 *       → the host libc / the host compiler provide these (_alloca and the
 *         64-bit divides are i386 compiler helpers x86_64 doesn't need;
 *         _vsnprintf is what ReactOS's <stdio.h> lowers `sprintf` to — on
 *         the host that stays a plain glibc `sprintf` call).
 *
 *   _errno, _invalid_parameter, get_locinfo, msvcrt_get_thread_data,
 *   _isspace_l, _tolower_l, _toupper_l, CompareStringA, LCMapStringA,
 *   ros_dbg_log, wine_dbgstr_an
 *       → this file.
 *
 * Compiled WITHOUT the ref_* rename prelude, but WITH refstub/msvcrt.h — so
 * the `msvcrt_ref_*` import namespace (see that header) is in force here
 * too, and the names below land in it.
 *
 * NOTHING HERE MAY CALL A FUNCTION cnt-rust EXPORTS.  cnt-rust ships
 * `#[no_mangle]` `strlen`, `memmove`, `tolower`, … and a static definition
 * beats libc at link time, so a casual `strlen()` in this file would route
 * the *reference* through *the code under test*.  Every helper below is
 * therefore hand-rolled from first principles.  None of them is one of the
 * 106 functions under test, so none of msvcrt's real logic is displaced.
 */
#include <msvcrt.h>
#include <winnls.h>

/* ------------------------------------------------------------------ *
 * Hijack-proof primitives (see the header comment).
 * ------------------------------------------------------------------ */

/* msvcrt's own memmove lives OUTSIDE string.c (it is an undefined symbol
 * in the real object too — the SSE2 one is `#ifndef __REACTOS__`), so this
 * displaces no reference logic.  `ref_memcpy` tail-calls it, and ref_memcpy
 * IS under test, hence it must not reach cnt-rust's memmove. */
void *__cdecl memmove(void *dst, const void *src, size_t n)
{
    unsigned char *d = dst;
    const unsigned char *s = src;

    if (d == s || !n) return dst;
    if (d < s)
        while (n--) *d++ = *s++;
    else
        while (n--) d[n] = s[n];
    return dst;
}

static size_t ref_len(const char *s)
{
    size_t n = 0;
    while (s[n]) n++;
    return n;
}

/* The C locale's ASCII classification/folding, spelled out.  This is
 * exactly what the host <ctype.h> does in the "C" locale (the process never
 * calls setlocale) AND exactly what msvcrt's `_pctype` table does before a
 * locale is set — but written out so it cannot bind to cnt-rust's exported
 * `tolower`. Bytes outside 0..127 pass through unchanged in the C locale. */
static int ref_ascii_lower(int c) { return (c >= 'A' && c <= 'Z') ? c + ('a' - 'A') : c; }
static int ref_ascii_upper(int c) { return (c >= 'a' && c <= 'z') ? c - ('a' - 'A') : c; }
static int ref_ascii_space(int c)
{
    return c == ' ' || c == '\t' || c == '\n' || c == '\v' || c == '\f' || c == '\r';
}

/* ------------------------------------------------------------------ *
 * errno
 * ------------------------------------------------------------------ */
static __thread int ref_errno_value;

int * __cdecl _errno(void)
{
    return &ref_errno_value;
}

/* Read/clear window for the tests: EINVAL / ERANGE / EILSEQ / STRUNCATE are
 * part of several msvcrt contracts, so a parity check should assert on them.
 * These two are exported un-namespaced (they are harness API, not msvcrt). */
int  ref_errno_get(void)      { return ref_errno_value; }
void ref_errno_set(int value) { ref_errno_value = value; }

/* ------------------------------------------------------------------ *
 * invalid-parameter handler
 *
 * MSVCRT_INVALID_PMT() sets errno, then calls this.  Real msvcrt dispatches
 * to a user handler; the default is "none → return".  A no-op IS the default
 * behaviour, and the errno store — the part that IS a value contract — has
 * already happened inside the macro.
 * ------------------------------------------------------------------ */
void __cdecl _invalid_parameter(const wchar_t *expr, const wchar_t *func,
                                const wchar_t *file, unsigned int line,
                                uintptr_t arg)
{
    (void)expr; (void)func; (void)file; (void)line; (void)arg;
}

/* ------------------------------------------------------------------ *
 * Locale — the DEFAULT "C" locale.
 *
 * This is the load-bearing modelling decision of the whole harness.  In the
 * C locale every lc_handle[] slot is 0, and string.c *branches on that*,
 * taking its genuine plain-ASCII paths:
 *
 *   _strcoll_l   : !lc_handle[LC_COLLATE] → strcmp()
 *   _stricoll_l  : !lc_handle[LC_COLLATE] → _stricmp()
 *   _strncoll_l  : !lc_handle[LC_COLLATE] → strncmp()
 *   _strxfrm_l   : !lc_handle[LC_COLLATE] → strncpy() + strlen()
 *   _strlwr_s_l  : !lc_handle[LC_CTYPE]   → 'A'..'Z' fold
 *   _strupr_s_l  : !lc_handle[LC_CTYPE]   → 'a'..'z' fold
 *
 * So CompareStringA / LCMapStringA below are unreachable at run time; they
 * only have to link.  A test that plants a non-zero lc_handle leaves the
 * modelled world and starts grading those two stubs instead of msvcrt.
 *
 * lconv->decimal_point is the one locale field on the hot path (fpnum_parse
 * / strtod).  It is a fixed table, not the host's — so a stray setlocale()
 * anywhere in the test process cannot move the reference.
 * ------------------------------------------------------------------ */
static char ref_decimal_point[] = ".";
static struct lconv ref_lconv;
static threadlocinfo ref_locinfo;

pthreadlocinfo CDECL get_locinfo(void)
{
    if (!ref_locinfo.lconv)
    {
        ref_lconv.decimal_point = ref_decimal_point;
        /* lc_handle[] stays all-zero == the "C" locale. */
        ref_locinfo.lconv       = &ref_lconv;
        ref_locinfo.mb_cur_max  = 1;
        ref_locinfo.lc_codepage = 0;
    }
    return &ref_locinfo;
}

/* ------------------------------------------------------------------ *
 * Per-thread CRT data.  string.c touches exactly one field (->strtok_next,
 * from strtok).  Real msvcrt keeps it in TLS; so do we — which also means
 * ref_strtok and the shim under test keep independent cursors, so a parity
 * test must drive both with the same call sequence.
 * ------------------------------------------------------------------ */
static __thread thread_data_t ref_thread_data;

thread_data_t * CDECL msvcrt_get_thread_data(void)
{
    return &ref_thread_data;
}

/* ------------------------------------------------------------------ *
 * _l ctype — the C locale (see ref_ascii_* above).
 *
 * Domain-guarded: string.c hands _tolower_l a RAW `char` in places (e.g.
 * _strtoi64_l's "0x" sniff), which can be negative.  In the C locale msvcrt
 * passes anything outside the ASCII range through unchanged, which is what
 * the helpers do — and unlike a table lookup, they can't fault on it.
 * ------------------------------------------------------------------ */
int __cdecl _isspace_l(int c, _locale_t locale) { (void)locale; return ref_ascii_space(c); }
int __cdecl _tolower_l(int c, _locale_t locale) { (void)locale; return ref_ascii_lower(c); }
int __cdecl _toupper_l(int c, _locale_t locale) { (void)locale; return ref_ascii_upper(c); }

/* ------------------------------------------------------------------ *
 * NLS.  Dead code under the C locale (see get_locinfo above).  Minimal
 * byte-wise ASCII implementations so a test that *does* plant an lc_handle
 * gets something sane rather than a link error.
 * ------------------------------------------------------------------ */
static int ref_nls_len(const char *s, int len)
{
    return len >= 0 ? len : (int)ref_len(s);
}

int WINAPI CompareStringA(LCID lcid, DWORD flags, const char *str1, int len1,
                          const char *str2, int len2)
{
    int fold = (flags & NORM_IGNORECASE) != 0;
    int i, n1 = ref_nls_len(str1, len1), n2 = ref_nls_len(str2, len2);
    int n = n1 < n2 ? n1 : n2;

    (void)lcid;
    for (i = 0; i < n; i++)
    {
        int c1 = (unsigned char)str1[i];
        int c2 = (unsigned char)str2[i];
        if (fold) { c1 = ref_ascii_upper(c1); c2 = ref_ascii_upper(c2); }
        if (c1 != c2) return c1 < c2 ? CSTR_LESS_THAN : CSTR_GREATER_THAN;
    }
    if (n1 == n2) return CSTR_EQUAL;
    return n1 < n2 ? CSTR_LESS_THAN : CSTR_GREATER_THAN;
}

int WINAPI LCMapStringA(LCID lcid, DWORD flags, const char *src, int srclen,
                        char *dst, int dstlen)
{
    int i, n = ref_nls_len(src, srclen);

    (void)lcid;
    if (flags & LCMAP_SORTKEY)
    {
        /* A byte-order-preserving key: the bytes themselves, NUL-terminated.
         * The returned length INCLUDES the terminator — that is what
         * _strxfrm_l's `ret - 1` accounting expects. */
        if (!dstlen) return n + 1;
        if (dstlen < n + 1) return 0;
        for (i = 0; i < n; i++) dst[i] = src[i];
        dst[n] = 0;
        return n + 1;
    }

    if (!dstlen) return n;
    if (dstlen < n) return 0;
    for (i = 0; i < n; i++)
    {
        int c = (unsigned char)src[i];
        if (flags & LCMAP_LOWERCASE)      c = ref_ascii_lower(c);
        else if (flags & LCMAP_UPPERCASE) c = ref_ascii_upper(c);
        dst[i] = (char)c;
    }
    return n;
}

/* ------------------------------------------------------------------ *
 * Debug output.  Unreferenced in practice (refstub/wine/debug.h no-ops
 * TRACE/FIXME, so nothing reaches ros_dbg_log/wine_dbgstr_an); defined
 * only so the external inventory of the real string.c.obj is complete.
 * Namespaced like the rest, so they can never collide with the real
 * ReactOS ones a future msvcrt-rust test binary might pull in.
 * ------------------------------------------------------------------ */
void msvcrt_ref_ros_dbg_log(unsigned int type, const char *file, unsigned int line,
                            const char *func, const char *format, ...)
{
    (void)type; (void)file; (void)line; (void)func; (void)format;
}

const char * msvcrt_ref_wine_dbgstr_an(const char *s, int n)
{
    (void)n;
    return s;
}
