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
/* The codepage-aware half of the NLS environment. The five entry points below delegate
 * to it whenever a DBCS codepage is in play; see nlsimpl.c for why that is mandatory
 * rather than an improvement. */
#include "nlsimpl.h"

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
 *
 * `_errno` USED to be defined here, over a `static __thread int` this file
 * invented.  It isn't any more: errno.c is a reference TU (#7), so the
 * reference now gets msvcrt's GENUINE accessor, which returns
 * `&msvcrt_get_thread_data()->thread_errno` — the thread_data this very file
 * supplies below.  That is the point of making it a reference TU: a harness
 * that models the errno slot cannot grade code whose contract IS the errno
 * slot (§9.73), and file.c alone stores through it 54 times.
 *
 * The two accessors below keep their names and signatures precisely so this
 * swap costs nothing elsewhere — they have ~62 call sites across 8 files in
 * two crates, and every one of them still works because only the STORAGE
 * moved.  fileimpl.c's deferral banner expected this to be "a change to every
 * existing test's plumbing"; it is not, as long as the harness API is held
 * fixed and only the implementation moves underneath it.
 *
 * Read/clear window for the tests: EINVAL / ERANGE / EILSEQ / STRUNCATE are
 * part of several msvcrt contracts, so a parity check should assert on them.
 * These two are exported un-namespaced (they are harness API, not msvcrt).
 * ------------------------------------------------------------------ */
int  ref_errno_get(void)      { return *_errno(); }
void ref_errno_set(int value) { *_errno() = value; }

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
 *
 * Since errno.c became reference TU #7 this is a COUNTING WRAPPER rather than a
 * replacement: it tallies, then calls errno.c's genuine body (which the errno
 * prelude renames `ref__invalid_parameter`, out of the msvcrt_ref_* import
 * namespace this definition occupies).  Two reasons to compose rather than
 * choose.  The counter is harness API with call sites in 8 files, so it has to
 * survive; and the real body is what a future _MSVCR_VER bump would change, so
 * keeping it on the live path means the reference follows msvcrt instead of
 * following this file.
 *
 * Composing is free HERE and would not be everywhere: at _MSVCR_VER = 0 that
 * body is inert.  The `>= 140` thread-data handler is compiled out; the
 * file-static `invalid_parameter_handler` it falls back on is writable only by
 * `_set_invalid_parameter_handler`, which is itself `>= 80` and so does not
 * exist — the slot is permanently NULL; and the `RaiseException` arm is `>= 80`
 * too.  What remains is one `ERR(...)`, which refstub/wine/debug.h no-ops.  So
 * the delegation adds no side effect, no output, and no undefined symbol.  (An
 * earlier plan here was to install the counter AS a handler through the real
 * dispatch.  There is no reachable slot to install it in.)
 * ------------------------------------------------------------------ */
static __thread int ref_invalid_pmt_calls;

/* errno.c's own body, under the name the errno reference prelude gives it. */
void __cdecl ref__invalid_parameter(const wchar_t *expr, const wchar_t *func,
                                    const wchar_t *file, unsigned int line,
                                    uintptr_t arg);

void __cdecl _invalid_parameter(const wchar_t *expr, const wchar_t *func,
                                const wchar_t *file, unsigned int line,
                                uintptr_t arg)
{
    ref_invalid_pmt_calls++;
    ref__invalid_parameter(expr, func, file, line, arg);
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

/* ── the REFERENCE's copy of the pinned MBCS fixture ───────────────────────
 *
 * Separate storage from the subject's (subjectimpl.c), for the reason the get_mbcinfo
 * note in refstub/msvcrt.h gives: the fixture is read-only state, so a graded function
 * that writes to it is a translation bug, and one shared copy could not tell the
 * difference.  Filled by mbcs_fixture_install (nlsimpl.c) out of the reference's own
 * create_mbcinfo.
 *
 * This is `get_mbcinfo` in the source and `msvcrt_ref_get_mbcinfo` after the namespace
 * block above — the reference can never reach the subject's. */
static rl_threadmbcinfo ref_mbcinfo;

rl_threadmbcinfo *mbcs_ref_slot(void) { return &ref_mbcinfo; }

pthreadmbcinfo CDECL get_mbcinfo(void)
{
    return &ref_mbcinfo;
}

/* ------------------------------------------------------------------ *
 * Per-thread CRT data.  string.c touches exactly one field (->strtok_next,
 * from strtok).  Real msvcrt keeps it in TLS; so do we — which also means
 * ref_strtok and the shim under test keep independent cursors, so a parity
 * test must drive both with the same call sequence.
 * ------------------------------------------------------------------ */
/* ── the non-"C"-locale mode: a locale whose TABLES ARE NOT ASCII ──────────
 *
 * FIRST ATTEMPT AT THIS WAS A FALSE GREEN, and the negative control caught it. Planting a
 * non-zero lc_handle DOES make msvcrt take its path 2:
 *
 *     if(!locinfo->lc_handle[LC_CTYPE]) { ...inline ASCII fold... }   // path 1
 *     else { c1 = _tolower_l((unsigned char)*s1++, locale); ... }     // path 2
 *
 * ...but path 2 folds through `_tolower_l`, which reads `locinfo->pclmap[]` — and if that
 * is still the *C locale's* map, it is an ASCII fold too. The two paths then agree on every
 * input, a path-1-only implementation is indistinguishable from a correct one, and the
 * harness reports MATCH. Which it did: sabotaging `_strnicmp` to always take path 1 did not
 * move a single row.
 *
 * So the mode must also swap the TABLES. These are the C tables plus the Latin-1 case pairs
 * (0xC0-0xDE <-> 0xE0-0xFE, holding 0xD7/0xF7 — the multiplication/division signs, which are
 * not letters). Now path 1 leaves 0xE9 alone while path 2 maps it to 0xC9, the two paths are
 * observably different, and the negative control bites.
 *
 * This is §9.73 in miniature, again: it is not enough to exercise the branch — the branch has
 * to be able to PRODUCE A DIFFERENT ANSWER, or exercising it proves nothing. */
static unsigned char nls_clmap[256];
static unsigned char nls_cumap[256];

static void ref_init_nls_tables(void)
{
    int c;
    if (nls_cumap[0xE9]) return;             /* already built */
    for (c = 0; c < 256; c++)
    {
        nls_clmap[c] = cloc_clmap[c];
        nls_cumap[c] = cloc_cumap[c];
    }
    for (c = 0xC0; c <= 0xDE; c++)           /* Latin-1 uppercase -> lowercase */
        if (c != 0xD7) nls_clmap[c] = (unsigned char)(c + 0x20);
    for (c = 0xE0; c <= 0xFE; c++)           /* Latin-1 lowercase -> uppercase */
        if (c != 0xF7) nls_cumap[c] = (unsigned char)(c - 0x20);
}

/* Exposed so the SUBJECT's locinfo can be aimed at the SAME tables. They are shared DATA —
 * the environment both sides must agree on — exactly like MSVCRT__ctype. */
const unsigned char *ref_nls_clmap(void) { ref_init_nls_tables(); return nls_clmap; }
const unsigned char *ref_nls_cumap(void) { ref_init_nls_tables(); return nls_cumap; }

/* The reference's mirror of subj_set_nls(). Both sides must be switched together, or the
 * comparison is meaningless. */
void ref_set_nls(int on)
{
    (void)get_locinfo();
    ref_init_nls_tables();
    ref_locinfo.lc_handle[LC_CTYPE]   = on ? 0x0409u : 0u;
    ref_locinfo.lc_handle[LC_COLLATE] = on ? 0x0409u : 0u;
    ref_locinfo.pclmap = on ? nls_clmap : cloc_clmap;
    ref_locinfo.pcumap = on ? nls_cumap : cloc_cumap;
}

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

    /* mbcs.c's _mbscoll / _mbsicoll family arrives here with MBCS strings. Comparing
     * them byte-wise splits a double-byte character down the middle, and folding a
     * TRAIL byte through an ASCII map corrupts it — so the collation family would be
     * graded against a reference doing something no implementation does.
     *
     * The codepage comes from the installed fixture rather than the LCID: several
     * codepages share one (949 and 1361 are both ko-KR), and every call site here
     * reaches this through `mbcinfo->mblcid` anyway. */
    if (nls_is_dbcs_codepage(nls_active_codepage()))
    {
        unsigned int cp = nls_active_codepage();
        unsigned short w1[512], w2[512];
        int m1 = MultiByteToWideChar(cp, 0, str1, n1, w1, 512);
        int m2 = MultiByteToWideChar(cp, 0, str2, n2, w2, 512);
        int m = m1 < m2 ? m1 : m2;
        for (i = 0; i < m; i++)
        {
            unsigned int c1 = w1[i], c2 = w2[i];
            if (fold) { c1 = nls_towupper(c1); c2 = nls_towupper(c2); }
            if (c1 != c2) return c1 < c2 ? CSTR_LESS_THAN : CSTR_GREATER_THAN;
        }
        if (m1 == m2) return CSTR_EQUAL;
        return m1 < m2 ? CSTR_LESS_THAN : CSTR_GREATER_THAN;
    }

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

/* The writer, for fileimpl.c's POSIX-backed Win32 shims — which are the only
 * thing that RAISES a last error rather than reporting one.  Kept here so the
 * slot stays single-storage: see the long note below on why one writer and one
 * reader is not a simplification but a requirement. */
void msvcrt_ref_set_last_error(DWORD err)
{
    ref_last_error = err;
}

/* ...and the SAME storage under the plain name, for the SUBJECT.
 *
 * refstub/msvcrt.h:224 namespaces `GetLastError` so the reference can never reach into
 * the code under test — correct everywhere else, wrong here. `WideCharToMultiByte` is
 * deliberately NOT namespaced (kernel32 is shared environment, not code under test), so
 * ONE converter serves both sides and it reports failure by writing `ref_last_error`. A
 * private subject-side last-error would read 0 exactly where that shared converter had
 * just set ERROR_INSUFFICIENT_BUFFER, and `_wctomb_s_l`'s insufficient-buffer arm is
 * precisely that writer/reader pair. One writer, one reader — which is also what the real
 * DLL has, last-error being a single TEB slot.
 *
 * wave 16's `_wctomb_s_l` (msvcrt-crt-rust/src/wcs_rest.rs) is the first flipped code to
 * import it; before that nothing subject-side referenced it and the asymmetry was
 * invisible. */
#undef GetLastError
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

/* `_mbtowc_l` USED to live here — a C-locale ASCII stub, because wcs.c called it and
 * nothing defined it.  mbcs.c DEFINES it, so as of the mbcs TU it is a reference
 * EXPORT, not an import: the prelude renames it to `ref__mbtowc_l` like every other
 * one, and it left the msvcrt_ref_* namespace block above.
 *
 * This is the same transition `_isspace_l` / `_tolower_l` / `_toupper_l` made when
 * ctype.c became a TU (see the note in the namespace block), and it fails the same
 * loud way if only half of it is done: keeping the stub gave `duplicate symbol
 * msvcrt_ref__mbtowc_l`, which is how it was found.
 *
 * Keeping the stub would have been worse than a link error if it had linked — wcs.c
 * would have gone on folding every byte 1:1 onto U+0000..U+00FF while mbcs.c did real
 * DBCS decoding, with the two disagreeing inside one process. */

/* `_wctomb_l` USED to be forwarded from here to `ref__wctomb_l` — wave 14b, when the
 * Rust printf engine imported a plain `_wctomb_l` that nothing on the subject side
 * defined, and the C converter still owned that symbol inside msvcrt.dll.
 *
 * Wave 16 FLIPPED `_wctomb_l` (msvcrt-crt-rust/src/wcs_rest.rs), so this forwarder became
 * a `duplicate symbol: _wctomb_l` that took EVERY test in this crate down — classify,
 * printf_shells, perf, all of them.  It is the same transition `_mbtowc_l` made just
 * above, and it fails the same loud way if only half of it is done.
 *
 * Deleting it is also the correct grading change, not just the linkable one: the subject
 * now runs the SHIPPING Rust converter (which is what msvcrt.dll will run) while the
 * reference runs the genuine `ref__wctomb_l`.  The printf rows stop being handed a shared
 * converter — the §9.73 shape — and start grading it. */

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
    /* The wide twin of __crtLCMapStringA, and it must obey the SAME non-ASCII mapping —
     * otherwise `_towlower_l`'s path 2 (LCMapStringW) agrees with its path 1 (ASCII) on
     * every input and a path-1-only implementation is again indistinguishable. See
     * ref_set_nls(). Both sides call THIS function (it is un-namespaced as of wave 11:
     * kernel32 is the environment, not code under test), so the comparison isolates the
     * Rust's logic around the call, not our model of the NLS. */
    int i, n = ref_nls_wlen(src, srclen);

    (void)lcid;
    ref_init_nls_tables();
    if (!dstlen) return n;
    if (dstlen < n) return 0;
    for (i = 0; i < n; i++)
    {
        wchar_t c = src[i];
        if (c < 256)
        {
            if (flags & LCMAP_UPPERCASE)      c = nls_cumap[c];
            else if (flags & LCMAP_LOWERCASE) c = nls_clmap[c];
        }
        else
        {
            /* create_mbcinfo case-maps the wide form of every single byte, and for a
             * DBCS codepage most of those are above U+00FF (CP932's 0xA1-0xDF are
             * halfwidth katakana at U+FF61-U+FF9F).  Leaving them unmapped, as this
             * did, empties `mbcasemap` and un-grades every case path in mbcs.c. */
            if (flags & LCMAP_UPPERCASE)      c = (wchar_t)nls_towupper(c);
            else if (flags & LCMAP_LOWERCASE) c = (wchar_t)nls_towlower(c);
        }
        dst[i] = c;
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

    (void)lcid; (void)xflag;

    /* mbcs.c's _mbctolower_l / _mbctoupper_l reach this with a TWO-BYTE DBCS character
     * and switch on the returned count: 0 leaves the character unchanged, 1 means a
     * single byte came back, anything else means two.  The byte-wise cloc map below
     * cannot case-map a DBCS character at all, so both arms would return the input
     * unchanged and the whole DBCS case family would grade as a no-op that agrees.
     *
     * Round-trip through the wide form instead, which is what Windows does. */
    if (nls_is_dbcs_codepage(codepage))
    {
        unsigned short w[8];
        int wn = MultiByteToWideChar(codepage, 0, src, n, w, 8);
        int j;
        for (j = 0; j < wn; j++)
        {
            if (mapflags & LCMAP_UPPERCASE)      w[j] = (unsigned short)nls_towupper(w[j]);
            else if (mapflags & LCMAP_LOWERCASE) w[j] = (unsigned short)nls_towlower(w[j]);
        }
        return nls_wide_to_mb(codepage, w, wn, dst, dstlen, "?", 0);
    }

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

    /* Reads msvcrt's REAL wide ctype table (ctype.c), not the model this file used to
     * carry — see "The wide ctype table — GONE as of wave 10" above.  MSVCRT__pwctype
     * is namespaced, so this is the reference's own pointer; it is aimed at the same
     * shared MSVCRT__wctype[] the subject's is.
     *
     * Its RL_UPPER/RL_LOWER bits are 0x1/0x2, the same values as C1_UPPER/C1_LOWER, so
     * the < 256 path already answers CT_CTYPE1 correctly and is left alone — it is what
     * the existing ctype verdicts are graded against.  Above the byte range, and for
     * CT_CTYPE3, the host's Unicode tables answer instead of a flat 0. */
    for (i = 0; i < n; i++)
    {
        if (type == CT_CTYPE3)
            chartype[i] = nls_ctype3(src[i]);
        else if (src[i] < 256)
            chartype[i] = (WORD)MSVCRT__pwctype[src[i]];
        else
            chartype[i] = nls_ctype1(src[i]);
    }
    return TRUE;
}

int WINAPI WideCharToMultiByte(UINT page, DWORD flags, const wchar_t *src, int srclen,
                               char *dst, int dstlen, const char *defchar, BOOL *used)
{
    int i, n;

    /* Real kernel32 REJECTS a NULL source with ERROR_INVALID_PARAMETER and returns 0.
     * This shim used to dereference it, which turned a graceful failure into a SIGSEGV —
     * and not hypothetically: `_tempnam` hands `astrdupw_utf8` whatever `_wtempnam`
     * returned, with no guard between them (msvcrt's own C does the same), so a failed
     * `GetTempFileNameW` arrives here as NULL. On Windows that path returns NULL to the
     * caller; here it killed the process, which reads as a translation crash and is not
     * one. Same rule as fileimpl.c's header states: model the API's ERROR behaviour, not
     * just its success path. */
    if (!src)
    {
        msvcrt_ref_set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }

    n = ref_nls_wlen(src, srclen);

    /* srclen == -1 means "NUL-terminated", and real kernel32 then CONVERTS AND COUNTS
     * the terminator.  ref_nls_wlen excludes it — correct for CompareStringW and
     * LCMapStringW, which take a length and not a buffer size — so add it back here.
     *
     * Leaving it off made every measure-then-malloc-then-convert caller allocate one
     * byte short and emit an UNTERMINATED string, which is the shape of the whole
     * pattern: msvcrt.h's astrdupw_utf8, fileimpl.c's path_from_wide, locale.c's
     * lconv fields, process.c's argv flattening.  path_from_wide is where it showed:
     * `_open` created its file under the requested name plus whatever byte followed
     * the block, and then round-tripped through the SAME garbled name perfectly, so
     * only file_substrate.rs's read-back-through-the-host could see it.
     *
     * Nothing differential could have caught this either — WideCharToMultiByte is
     * SHARED environment, so both sides truncated identically and every row still
     * reported MATCH (§9.73).  Note MultiByteToWideChar in nlsimpl.c has always had
     * the matching `strlen(src) + 1`; this was the asymmetry. */
    if (srclen < 0) n++;

    (void)flags;
    if (used) *used = FALSE;

    /* A DBCS codepage has a real answer for most of the BMP, and create_mbcinfo needs
     * it: this is the pass that turns case-mapped wide forms back into the single bytes
     * that fill `mbcasemap`.  The 1:1 model below would map U+FF71 to '?' and leave the
     * table empty. */
    if (nls_is_dbcs_codepage(page))
    {
        int used_i = 0;
        int r;

        /* Real kernel32 NEVER truncates: when the conversion does not fit it writes
         * nothing, returns 0 and sets ERROR_INSUFFICIENT_BUFFER.  The model used to
         * return the truncated count instead, and that one difference made a whole arm of
         * msvcrt unreachable — `_wctomb_s_l`'s
         *
         *     if (!mblen && GetLastError() == ERROR_INSUFFICIENT_BUFFER) ... ERANGE
         *
         * needs a ZERO return for a single-character conversion, which a truncating model
         * can never produce.  wave 16's reachability probe measured it: 0 hits across
         * 3 codepages x 9363 characters x 4 sizes.  A branch a harness cannot reach is a
         * branch it is not grading, however green the row looks.
         *
         * Scoped to the single-character conversions (`n == 1`) that this affects, so the
         * multi-character passes create_mbcinfo makes keep their existing behaviour, and
         * measured into a real buffer rather than a NULL one (the `cd == -1` fallback
         * inside nls_wide_to_mb writes unconditionally when dstlen is 0). */
        if (dstlen > 0 && n == 1)
        {
            char probe[8];
            int need = nls_wide_to_mb(page, src, n, probe, (int)sizeof(probe), defchar, NULL);
            if (need > dstlen)
            {
                ref_last_error = ERROR_INSUFFICIENT_BUFFER;
                return 0;
            }
        }

        r = nls_wide_to_mb(page, src, n, dst, dstlen, defchar, &used_i);
        if (used) *used = used_i ? TRUE : FALSE;
        return r;
    }

    /* CP_UTF8 has no default character to fall back on, so kernel32 does not substitute
     * one — it REFUSES an unpaired surrogate, returning 0 with ERROR_NO_UNICODE_TRANSLATION.
     * (Every other codepage here maps the unrepresentable to defchar and reports *used.)
     *
     * That distinction is the whole EILSEQ signal for a UTF-8 caller, because wcs.c:641
     * hands `WideCharToMultiByte` a NULL lpUsedDefaultChar for exactly this codepage:
     *
     *     pused_default = (locinfo->lc_codepage != CP_UTF8 ? &used_default : NULL);
     *
     * so `wcsrtombs_l` under UTF-8 can only learn of a bad character from a ZERO return.
     * With the model substituting '?' and returning n, that arm was unreachable and
     * wave16's `wcsrtombs [CP_UTF8]` row was entering it only through the empty-string
     * miscount fixed above — which is what its reachability guard caught.
     *
     * NOTE what this does NOT fix: the 1:1 loop below still renders U+0100..U+FFFF as a
     * single '?' byte rather than a real UTF-8 sequence. Both sides share it, so no row
     * can diverge on it; it costs realism, not correctness, and the surrogate arm is what
     * the graded branches actually need. */
    if (page == CP_UTF8)
    {
        for (i = 0; i < n; i++)
        {
            /* UNPAIRED only — a well-formed high/low pair is a legal code point and
             * converts.  Rejecting every surrogate would make the row fail on valid
             * astral input, which is a different bug wearing the same green tick. */
            if (src[i] >= 0xD800 && src[i] <= 0xDBFF)
            {
                if (i + 1 < n && src[i + 1] >= 0xDC00 && src[i + 1] <= 0xDFFF) { i++; continue; }
            }
            else if (src[i] < 0xDC00 || src[i] > 0xDFFF)
                continue;
            ref_last_error = ERROR_NO_UNICODE_TRANSLATION;
            return 0;
        }
    }

    if (dstlen && dstlen < n) { ref_last_error = ERROR_INSUFFICIENT_BUFFER; return 0; }
    for (i = 0; i < n; i++)
    {
        /* C locale: U+0000..U+00FF map 1:1; anything else is unrepresentable. */
        char c;
        if (src[i] < 256)
            c = (char)src[i];
        else
        {
            c = defchar ? *defchar : '?';
            if (used) *used = TRUE;
        }
        /* The loop RUNS on a measure-only call (dstlen == 0) and only the STORE is
         * skipped.  Returning early instead — which is what this did — reports the
         * size without ever setting *used, and `used` is the entire EILSEQ signal for
         * the size-query callers: wcs.c:646 asks
         *
         *     tmp = WideCharToMultiByte(cp, WC_NO_BEST_FIT_CHARS, *wcstr, -1,
         *                               NULL, 0, NULL, pused_default);
         *     if (!tmp || (pused_default && *pused_default)) { errno = EILSEQ; ... }
         *
         * so with *used never written the arm could only be entered by `!tmp` — which,
         * before the `srclen < 0` fix above, an EMPTY string produced.  wave16's
         * `wcsrtombs [CP_UTF8]` row was reaching EILSEQ that way and its reachability
         * guard rightly went UNEXERCISED the moment the count was corrected: it had been
         * grading a spurious failure on `L""` rather than a real unconvertible character.
         * Real kernel32 converts internally on a size query and reports the flag. */
        if (dstlen) dst[i] = c;
    }
    return n;
}

/* ==================================================================
 * ntdll's bitmap — what `%[` builds its scanset in (scanf.h:622-689).
 *
 * The genuine RtlSetBits has NO range check outside an ASSERT that a release
 * ntdll compiles out, and msvcrt hands it `*format` — a SIGNED char for the
 * narrow engine.  `%[\x80]` therefore calls RtlSetBits(map, 0xFFFFFF80, 1) and
 * writes roughly 128 MB below a 32-byte heap block.  That is undefined
 * behaviour, not behaviour: there is nothing for a port to reproduce and
 * nothing for a harness to compare.  tests/scanf.rs excludes bytes >= 0x80 from
 * every NARROW scanset corpus and prints that exclusion.
 *
 * So this model CLAMPS instead of corrupting, and COUNTS every clamp.  The count
 * is not decoration — it is what stops the exclusion from rotting.  If a corpus
 * change ever starts feeding out-of-range indices, tests/scanf.rs asserts the
 * counter is zero and fails, instead of quietly grading a wild write against a
 * Rust that carefully avoids one.
 *
 * Everything else is the real algorithm, behaviourally verbatim from
 * sdk/lib/rust/bitmap/src/ops.rs (the ntdll actually under this DLL): bit k
 * lives in word k/32 at bit k%32, and BOTH range checks in RtlAreBitsSet /
 * RtlAreBitsClear fail CLOSED — so an out-of-range character matches neither an
 * ordinary nor an inverted scanset, and `are_bits_clear` is NOT the negation of
 * `are_bits_set`.
 *
 * Named msvcrt_ref_* because the scanf prelude puts them in the reference's
 * import namespace; the plain-named declarations in the generated winternl.h are
 * included here only for RTL_BITMAP's layout and are never defined.
 * ================================================================== */
#include "winternl.h"

static unsigned long ref_bitmap_oob_clamps;

unsigned long msvcrt_ref_bitmap_oob_count(void) { return ref_bitmap_oob_clamps; }
void          msvcrt_ref_bitmap_oob_reset(void) { ref_bitmap_oob_clamps = 0; }

void msvcrt_ref_RtlInitializeBitMap(PRTL_BITMAP bm, ULONG *buffer, ULONG size)
{
    bm->SizeOfBitMap = size;
    bm->Buffer = buffer;
}

void msvcrt_ref_RtlSetBits(PRTL_BITMAP bm, ULONG start, ULONG count)
{
    ULONG i;

    if (count == 0) return;
    if ((unsigned long long)start + count > (unsigned long long)bm->SizeOfBitMap)
    {
        ref_bitmap_oob_clamps++;   /* see the header comment — deliberately NOT silent */
        return;
    }
    for (i = 0; i < count; i++)
        bm->Buffer[(start + i) >> 5] |= 1u << ((start + i) & 31);
}

BOOL msvcrt_ref_RtlAreBitsSet(PRTL_BITMAP bm, ULONG start, ULONG length)
{
    ULONG i;

    if (start + length > bm->SizeOfBitMap || start + length <= start) return FALSE;
    for (i = 0; i < length; i++)
        if (!(bm->Buffer[(start + i) >> 5] & (1u << ((start + i) & 31)))) return FALSE;
    return TRUE;
}

BOOL msvcrt_ref_RtlAreBitsClear(PRTL_BITMAP bm, ULONG start, ULONG length)
{
    ULONG i;

    if (start + length > bm->SizeOfBitMap || start + length <= start) return FALSE;
    for (i = 0; i < length; i++)
        if (bm->Buffer[(start + i) >> 5] & (1u << ((start + i) & 31))) return FALSE;
    return TRUE;
}
