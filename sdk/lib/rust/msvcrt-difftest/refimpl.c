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
 *
 * But "did the handler fire AT ALL?" is itself an observable contract: an app
 * installs one with _set_invalid_parameter_handler() and it may abort(). So we
 * COUNT the calls, mirroring subjectimpl.c's counter, and classify compares the
 * two.  Without this the whole effect is invisible — and it WAS invisible: the
 * counter existed on the subject side from wave 5 but nothing ever read it, so
 * `_ultoa_s`-class contracts (which set errno WITHOUT firing the handler) could
 * not be distinguished from `_ltoa_s`-class ones (which fire it).  See §9.66.
 * ------------------------------------------------------------------ */
static __thread int ref_invalid_pmt_calls;

void __cdecl _invalid_parameter(const wchar_t *expr, const wchar_t *func,
                                const wchar_t *file, unsigned int line,
                                uintptr_t arg)
{
    (void)expr; (void)func; (void)file; (void)line; (void)arg;
    ref_invalid_pmt_calls++;
}

int  ref_invalid_pmt_count(void) { return ref_invalid_pmt_calls; }
void ref_invalid_pmt_reset(void) { ref_invalid_pmt_calls = 0; }

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

/* The C locale's lower/upper mapping tables, PARSED OUT OF locale.c by build.rs
 * (they are `static const` there, so they cannot be linked).  Not hand-copied: the
 * reference and the subject share ONE locinfo, so a mistyped byte would make them
 * both wrong the same way and classify would still say MATCH.  See build.rs. */
#include "cloc_tables.h"

/* ctype.c (reference TU #3) owns the narrow ctype table.  It is shared ENVIRONMENT,
 * not code under test — exactly like get_locinfo() itself: the reference and the
 * subject must agree on what the locale IS, or the comparison means nothing.  Hence
 * it is deliberately NOT in the ref_* rename lists.
 *
 * NOTE THE +1.  msvcrt sets `pctype = MSVCRT__ctype + 1`, so `pctype[-1]` is a real,
 * addressable slot (the EOF entry) — which is why _isctype_l's domain test is
 * `c >= -1`.  Any implementation that indexes pctype with an unsigned `c` reads at
 * 0xFFFFFFFF instead. */
extern WORD MSVCRT__ctype[257];

/* locale.c's exported `_pctype` pointer (ctype.c's __p__pctype returns its address).
 * locale.c is not a reference TU, so the harness owns this one. */
unsigned short *MSVCRT__pctype = NULL;

/* locale.c's "no setlocale() has been called yet" flag.  ctype.c BRANCHES on it:
 * `toupper`/`tolower` take a hardcoded-ASCII fast path when it is TRUE and only fall
 * through to `_toupper_l`/`_tolower_l` (the pcumap/pclmap lookup) when it is FALSE.
 * TRUE is the truthful default here — the harness never calls setlocale — and the
 * Rust flip must reproduce the same branch, or it is both slower and a different
 * function. */
BOOL initial_locale = TRUE;

pthreadlocinfo CDECL get_locinfo(void)
{
    if (!ref_locinfo.lconv)
    {
        ref_lconv.decimal_point = ref_decimal_point;
        /* lc_handle[] stays all-zero == the "C" locale. */
        ref_locinfo.lconv       = &ref_lconv;
        ref_locinfo.mb_cur_max  = 1;
        ref_locinfo.lc_codepage = 0;
        /* The three tables that make the ctype/case family gradeable at all.
         * Same assignment locale.c:654-656 makes for its static C locale. */
        ref_locinfo.pctype      = MSVCRT__ctype + 1;
        ref_locinfo.pclmap      = cloc_clmap;
        ref_locinfo.pcumap      = cloc_cumap;
        MSVCRT__pctype          = ref_locinfo.pctype;
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
 * _l ctype — GONE as of wave 10, and that is the upgrade.
 *
 * refimpl.c used to hand-stub `_isspace_l` / `_tolower_l` / `_toupper_l` as ASCII
 * approximations, because string.c and wcs.c call them and nothing else defined them.
 * That made them the ORACLE for those calls — an oracle we wrote ourselves.
 *
 * ctype.c is now reference TU #3, so it defines the GENUINE ones, and the rename
 * prelude routes string.c's internal calls to `ref__tolower_l` → msvcrt's real body →
 * `locinfo->pclmap[c]`.  The reference stopped grading our approximation and started
 * grading itself.
 *
 * On the subject side the same three names are now supplied by msvcrt-crt-rust's own
 * flips.  Keeping the stubs here would be a duplicate-symbol link error — and, worse,
 * whichever copy won would silently become the answer.
 * ------------------------------------------------------------------ */

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

const wchar_t * msvcrt_ref_wine_dbgstr_wn(const wchar_t *s, int n)
{
    (void)n;
    return s;
}

/* ================================================================== *
 * THE WIDE HALF — the externals of dll/win32/msvcrt/wcs.c, the second
 * reference TU.
 *
 * Same modelling contract as everything above: the DEFAULT "C" locale.  The
 * 14 pure wide-string functions under test (wcslen/wcscmp/wcschr/wcscpy/…)
 * touch NONE of what follows — they are pure.  These exist so wcs.c *links*:
 * its other 162 public symbols (the printf engine, the isw* ctype family, the
 * NLS collation paths) reference them, and those are all DEFERRED, not graded.
 *
 * They are still written faithfully rather than as zero-returning stubs.  A
 * stub that fabricates a plausible value is exactly how a differential harness
 * ends up grading Rust against a lie and reporting MATCH — so where a real
 * answer is cheap (the heap, the ASCII ctype table), give the real answer.
 * ================================================================== */

/* ------------------------------------------------------------------ *
 * Heap.  _wcsdup / the printf engine's scratch buffers.  A heap is a heap:
 * malloc IS the faithful model, so this is not a stub in any meaningful sense.
 * ------------------------------------------------------------------ */
static int ref_process_heap;

HANDLE WINAPI GetProcessHeap(void)
{
    return &ref_process_heap;
}

LPVOID WINAPI HeapAlloc(HANDLE heap, DWORD flags, size_t size)
{
    void *p;

    (void)heap;
    p = malloc(size ? size : 1);
    if (p && (flags & 0x8 /* HEAP_ZERO_MEMORY */))
    {
        unsigned char *b = p;
        size_t i;
        for (i = 0; i < size; i++) b[i] = 0;   /* not memset: see header comment */
    }
    return p;
}

BOOL WINAPI HeapFree(HANDLE heap, DWORD flags, LPVOID ptr)
{
    (void)heap; (void)flags;
    free(ptr);
    return TRUE;
}

static DWORD ref_last_error;

DWORD WINAPI GetLastError(void)
{
    return ref_last_error;
}

/* ------------------------------------------------------------------ *
 * The wide ctype table — GONE as of wave 10, and this is the upgrade.
 *
 * refimpl.c used to spell out a 256-entry `MSVCRT__pwctype` "from first principles",
 * because wcs.c reads that global and, with only string.c + wcs.c as reference TUs,
 * nothing defined it. So the reference graded itself against OUR MODEL of msvcrt.
 *
 * ctype.c is now reference TU #3, and it carries the genuine article:
 *
 *     WORD  MSVCRT__wctype[257];
 *     WORD *MSVCRT__pwctype = MSVCRT__wctype + 1;
 *
 * so this file's model is deleted and both sides read msvcrt's real table.  (The
 * subject's copy of the *pointer* lives in subjectimpl.c — `MSVCRT__pwctype` IS
 * namespaced (refstub/msvcrt.h), so the reference's is `msvcrt_ref__pwctype` and the
 * two sides keep independent pointers — but they are aimed at the SAME shared
 * `MSVCRT__wctype` table, which is NOT namespaced. That is the correct split: the
 * locale DATA is environment both sides must agree on; the pointers and the errno
 * are per-side state that classify compares.)
 * ------------------------------------------------------------------ */

/* ------------------------------------------------------------------ *
 * Locale plumbing.  get_current_locale_noalloc() fills a caller-provided
 * _locale_tstruct (the `_locale_tstruct tmp = {0};` on wcs.c's stack) and
 * hands it back; free_locale_noalloc() releases it.  Under the C locale there
 * is nothing to refcount, so "fill in the one locinfo we model" is exact.
 * ------------------------------------------------------------------ */
_locale_t CDECL get_current_locale_noalloc(_locale_t locale)
{
    locale->locinfo = get_locinfo();
    locale->mbcinfo = NULL;
    return locale;
}

void CDECL free_locale_noalloc(_locale_t locale)
{
    (void)locale;
}

/* printf's %e exponent width.  msvcrt's default is the 3-digit form; the
 * 2-digit form is opt-in via _set_output_format.  Return the default. */
unsigned int __cdecl _get_output_format(void)
{
    return 0;
}

/* ------------------------------------------------------------------ *
 * The _l wide-ctype / conversion externals wcs.c calls but does NOT define.
 * C locale, ASCII, spelled out (see ref_ascii_* above).
 * ------------------------------------------------------------------ */
int __cdecl _mbtowc_l(wchar_t *dst, const char *str, size_t n, _locale_t locale)
{
    (void)locale;
    if (!str || !n) return 0;
    if (!*str) { if (dst) *dst = 0; return 0; }
    /* C locale: single-byte, 1:1 onto U+0000..U+00FF. */
    if (dst) *dst = (wchar_t)(unsigned char)*str;
    return 1;
}

/* ------------------------------------------------------------------ *
 * MSVC intrinsic.  `_BitScanReverse(&i, mask)` → index of the highest set
 * bit; returns 0 (and leaves *index untouched) when mask == 0.
 * ------------------------------------------------------------------ */
unsigned char _BitScanReverse(unsigned int *index, unsigned int mask)
{
    unsigned int i;

    if (!mask) return 0;
    for (i = 31; ; i--)
    {
        if (mask & (1u << i)) { *index = i; return 1; }
        if (!i) break;
    }
    return 0;
}

/* ------------------------------------------------------------------ *
 * Wide NLS.  Dead under the C locale, exactly like CompareStringA /
 * LCMapStringA above (wcs.c takes `if(!locinfo->lc_handle[LC_COLLATE])` first).
 * Byte-wise ASCII so a test that DOES plant an lc_handle gets something sane
 * rather than a link error.
 * ------------------------------------------------------------------ */
static int ref_nls_wlen(const wchar_t *s, int len)
{
    int n = 0;
    if (len >= 0) return len;
    while (s[n]) n++;
    return n;
}

int WINAPI CompareStringW(LCID lcid, DWORD flags, const wchar_t *str1, int len1,
                          const wchar_t *str2, int len2)
{
    int fold = (flags & NORM_IGNORECASE) != 0;
    int i, n1 = ref_nls_wlen(str1, len1), n2 = ref_nls_wlen(str2, len2);
    int n = n1 < n2 ? n1 : n2;

    (void)lcid;
    for (i = 0; i < n; i++)
    {
        int c1 = str1[i], c2 = str2[i];
        if (fold) { c1 = ref_ascii_upper(c1); c2 = ref_ascii_upper(c2); }
        if (c1 != c2) return c1 < c2 ? CSTR_LESS_THAN : CSTR_GREATER_THAN;
    }
    if (n1 == n2) return CSTR_EQUAL;
    return n1 < n2 ? CSTR_LESS_THAN : CSTR_GREATER_THAN;
}

int WINAPI LCMapStringW(LCID lcid, DWORD flags, const wchar_t *src, int srclen,
                        wchar_t *dst, int dstlen)
{
    int i, n = ref_nls_wlen(src, srclen);

    (void)lcid;
    if (flags & LCMAP_SORTKEY)
    {
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
        int c = src[i];
        if (flags & LCMAP_LOWERCASE)      c = ref_ascii_lower(c);
        else if (flags & LCMAP_UPPERCASE) c = ref_ascii_upper(c);
        dst[i] = (wchar_t)c;
    }
    return n;
}

/* ------------------------------------------------------------------ *
 * The two NLS entry points ctype.c reaches for OUTSIDE the byte range.
 *
 * Neither is namespaced, so there is ONE definition and both sides call it.  That is
 * deliberate, and it is what makes the `c >= 256` arms gradeable at all:
 *
 *   _toupper_l(c) for c >= 256, even in the "C" locale, is NOT a table lookup — it
 *   builds a 1-2 byte string, sets errno=EILSEQ if the high byte is not a leadbyte,
 *   calls __crtLCMapStringA, and switches on the RETURN COUNT (0 -> c unchanged,
 *   1 -> ret[0], else ret[0] | ret[1]<<8).
 *
 * All of that is real logic the Rust must reproduce, and it is exactly what a shared
 * stub tests: both sides call the same function with the same arguments and must agree
 * on how to interpret what comes back.  What the stub does NOT model is Windows' actual
 * NLS *data* — so a divergence in kernel32's uppercase mapping is out of scope here
 * (correctly: it is kernel32's job, not msvcrt's).  Same bounded blind spot the
 * CompareStringA / LCMapStringA stubs already carry.
 * ------------------------------------------------------------------ */
BOOL WINAPI GetStringTypeExA(LCID lcid, DWORD type, const char *src, int len, WORD *out)
{
    int i, n = ref_nls_len(src, len);

    (void)lcid; (void)type;
    for (i = 0; i < n; i++)
        out[i] = MSVCRT__ctype[(unsigned char)src[i] + 1];  /* +1: pctype's bias */
    return TRUE;
}

int CDECL __crtLCMapStringA(LCID lcid, DWORD mapflags, const char *src, int srclen,
                            char *dst, int dstlen, unsigned int codepage, int xflag)
{
    int i, n = ref_nls_len(src, srclen);

    (void)lcid; (void)codepage; (void)xflag;
    if (n > dstlen) n = dstlen;
    for (i = 0; i < n; i++)
    {
        unsigned char c = (unsigned char)src[i];
        if (mapflags & LCMAP_UPPERCASE)      dst[i] = (char)cloc_cumap[c];
        else if (mapflags & LCMAP_LOWERCASE) dst[i] = (char)cloc_clmap[c];
        else                                 dst[i] = (char)c;
    }
    return n;
}

BOOL WINAPI GetStringTypeW(DWORD type, const wchar_t *src, int count, WORD *chartype)
{
    int i, n = ref_nls_wlen(src, count);

    (void)type;
    /* Reads msvcrt's REAL wide ctype table (ctype.c), not the model this file used to
     * carry — see "The wide ctype table — GONE as of wave 10" above.  MSVCRT__pwctype
     * is namespaced, so this is the reference's own pointer; it is aimed at the same
     * shared MSVCRT__wctype[] the subject's is. */
    for (i = 0; i < n; i++)
        chartype[i] = (WORD)(src[i] < 256 ? MSVCRT__pwctype[src[i]] : 0);
    return TRUE;
}

int WINAPI WideCharToMultiByte(UINT page, DWORD flags, const wchar_t *src, int srclen,
                               char *dst, int dstlen, const char *defchar, BOOL *used)
{
    int i, n = ref_nls_wlen(src, srclen);

    (void)page; (void)flags;
    if (used) *used = FALSE;
    if (!dstlen) return n;
    if (dstlen < n) { ref_last_error = ERROR_INSUFFICIENT_BUFFER; return 0; }
    for (i = 0; i < n; i++)
    {
        /* C locale: U+0000..U+00FF map 1:1; anything else is unrepresentable. */
        if (src[i] < 256)
            dst[i] = (char)src[i];
        else
        {
            dst[i] = defchar ? *defchar : '?';
            if (used) *used = TRUE;
        }
    }
    return n;
}
