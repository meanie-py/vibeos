/*
 * subjectimpl.c — the msvcrt internals that the CRATE UNDER TEST imports.
 *
 * `msvcrt-crt-rust` is the code that links into msvcrt.dll. For the `_s`
 * ("secure") family its contract is not just the destination buffer — it is
 * *errno* plus the *invalid-parameter handler*:
 *
 *     #define MSVCRT_INVALID_PMT(x,err) (*_errno() = (err), \
 *                                        _invalid_parameter(NULL,NULL,NULL,0,0))
 *
 * Inside the real DLL those two resolve to msvcrt's own C. In this harness they
 * must be supplied here.
 *
 * WHY NOT REUSE refimpl.c's? Because refimpl's are deliberately renamed into the
 * private `msvcrt_ref_*` namespace (refstub/msvcrt.h) — that namespace exists
 * precisely so the REFERENCE can never bind to the CODE UNDER TEST. Reusing them
 * would defeat it, AND it would give the subject and the reference a SHARED
 * errno — so a test could not tell which side set it.
 *
 * Separate storage is the point: `ref_strcpy_s` writes msvcrt_ref_errno_value,
 * the Rust `strcpy_s` writes subj_errno_value, and classify asserts BOTH agree.
 * An errno contract you cannot observe independently is an errno contract you are
 * not testing.
 *
 * Compiled WITHOUT the rename prelude and WITHOUT refstub/msvcrt.h, so these land
 * under their plain names — exactly what msvcrt-crt-rust's `extern "C"` block
 * declares.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

/* ------------------------------------------------------------------ *
 * errno — the SUBJECT's, independent of the reference's.
 *
 * The three definitions that used to live here (`_errno`, `__doserrno` and a
 * hand-written `msvcrt_set_errno`) are GONE: msvcrt-crt-rust's errno.rs is the
 * translation of errno.c and exports all three, so a copy here is a duplicate
 * symbol — and, worse, a second store that the code under test would not be using.
 *
 * The Rust reads them out of `msvcrt_get_thread_data()`, which this file already
 * provides (`subj_thread_data`, below), so the subject's errno now lives in the same
 * per-thread struct msvcrt really keeps it in. These accessors follow it there; they
 * stay un-namespaced on purpose, because classify.rs and the file/printf suites read
 * them directly.
 * ------------------------------------------------------------------ */
extern void *msvcrt_get_thread_data(void);
static int  *subj_errno_slot(void);
static unsigned int *subj_doserrno_slot(void);

int  subj_errno_get(void)      { return *subj_errno_slot(); }
void subj_errno_set(int value) { *subj_errno_slot() = value; }

/* `unsigned int` rather than msvcrt's `__msvcrt_ulong`, which is a 32-bit spelling this
 * TU has no header for — and 64-bit here would let a write past bit 31 go unnoticed. */
unsigned int subj_doserrno_get(void)           { return *subj_doserrno_slot(); }
void         subj_doserrno_set(unsigned int v) { *subj_doserrno_slot() = v; }

/* ------------------------------------------------------------------ *
 * _wcsdup — RETIRED (wave 32), exactly as this stub's own note said it would be.
 *
 * It stood in because `file.rs`'s `tmpnam_helper` calls `_wcsdup` and
 * msvcrt-crt-rust did not export it, so the subject side had no Rust copy to
 * link against while the reference's stayed renamed `ref__wcsdup`.  Wave 32 put
 * the real one in `wcs_rest.rs`, which is the retirement the note described —
 * "by making wcs.c's owner a reference TU whose subject half is Rust".  Keeping
 * the stub now is a duplicate symbol AND a worse test: it graded a hand-written
 * stand-in where the shipping artifact is available.
 *
 * The 16-bit-wchar_t warning it carried is preserved where it now applies: the
 * Rust body takes `*const wchar_t` from the msvcrt-sys island, which is
 * `c_ushort`, so the host's 32-bit `wchar_t` never enters the picture.
 * ------------------------------------------------------------------ */

/* ------------------------------------------------------------------ *
 * The invalid-parameter handler.
 *
 * Real msvcrt dispatches to a user-installed handler; the DEFAULT is "none
 * installed -> just return", and the errno store (the part that IS a value
 * contract) has already happened inside MSVCRT_INVALID_PMT before we get here.
 * So a no-op is the faithful default — but we also COUNT the calls, because
 * "did the invalid-parameter path fire at all?" is itself part of the `_s`
 * contract and classify asserts on it.
 * ------------------------------------------------------------------ */
static __thread int subj_invalid_pmt_calls;

void _invalid_parameter(const void *expr, const void *func, const void *file,
                        unsigned int line, uintptr_t arg)
{
    (void)expr; (void)func; (void)file; (void)line; (void)arg;
    subj_invalid_pmt_calls++;
}

int  subj_invalid_pmt_count(void) { return subj_invalid_pmt_calls; }
void subj_invalid_pmt_reset(void) { subj_invalid_pmt_calls = 0; }

/* GetStringTypeW lives in refimpl.c and is SHARED as of wave 11 (it is kernel32 — the
 * environment, never code under test). Modelling it separately here would have been the
 * §9.73 trap: behaviour handed to both sides can never be caught by a differential test.
 *
 * Wave 10 already deleted this file's hand-written 256-entry MSVCRT__pwctype table for
 * the same reason; ctype.c owns the real one. */

/* ------------------------------------------------------------------ *
 * The SUBJECT's locale + per-thread state (wave 10).
 *
 * `get_locinfo`, `msvcrt_get_thread_data` and `MSVCRT__pwctype` are all in the
 * anti-circularity namespace (refstub/msvcrt.h), so the reference's copies are
 * `msvcrt_ref_*` and are unreachable from here.  The subject needs its own — and
 * SHOULD have its own: separate storage is what lets classify compare each side's
 * errno, invalid-parameter count and strtok cursor independently.
 *
 * But the two sides must agree on what the LOCALE IS, or the comparison grades data
 * instead of code.  So the state is per-side while the DATA is shared:
 *
 *   MSVCRT__ctype[257] / MSVCRT__wctype[257]   ctype.c, NOT namespaced -> one copy,
 *                                              msvcrt's genuine tables, both sides.
 *   cloc_clmap / cloc_cumap                    lifted from locale.c by build.rs ->
 *                                              identical bytes on both sides.
 *
 * That split is the point.  Data we supply to BOTH sides can never be caught by a
 * differential test — a mistyped byte makes both sides wrong the same way and
 * classify still reports MATCH.  So we supply as little of it as possible, and take
 * what we do supply from msvcrt's own source.
 * ------------------------------------------------------------------ */
#include "locale_layout.h"
#include "cloc_tables.h"   /* generated: cloc_clmap[256], cloc_cumap[256] */

extern unsigned short MSVCRT__ctype[257];    /* ctype.c — shared, un-namespaced */
extern unsigned short MSVCRT__wctype[257];   /* ctype.c — shared, un-namespaced */

/* msvcrt exports these as POINTERS INTO the tables.  NOTE THE +1 on both: msvcrt sets
 * `pctype = MSVCRT__ctype + 1`, which is why `_isctype_l`'s domain test is `c >= -1`
 * (pctype[-1] is the real, addressable EOF slot).  Any implementation that indexes
 * pctype with an *unsigned* c reads at 0xFFFFFFFF instead. */
unsigned short *MSVCRT__pwctype;
/* MSVCRT__pctype is NOT namespaced, so refimpl.c's definition is the one and only —
 * do not define it here (it would be a duplicate symbol). */
extern unsigned short *MSVCRT__pctype;

static rl_threadlocinfo subj_locinfo;

/* ── the get_locinfo() CALL COUNTER ────────────────────────────────
 *
 * Not instrumentation for its own sake — this is the sixth instrument, and it exists
 * because the other five could not see a bug that turned the gate red.
 *
 * In the REAL msvcrt, `get_locinfo()` is NOT cheap:
 *
 *     pthreadlocinfo get_locinfo(void) {
 *         thread_data_t *data = msvcrt_get_thread_data();   // GetLastError() +
 *         update_thread_locale(data);                       // TlsGetValue()  +
 *         ...                                               // SetLastError()
 *
 * — three Win32 calls. Call it once per CALL and nobody notices. Call it once per
 * CHARACTER and `_wcsicmp` (registry key comparisons, on the boot's hot path) drags the
 * whole boot from 39s to 128s and times out the kernel TDI self-test. Which is exactly
 * what wave 11 shipped.
 *
 * Why no existing instrument caught it:
 *   classify   — the ANSWERS were right. It grades values, not cost.
 *   oob_guard  — in bounds.
 *   perf       — 0.91x, PASSING. And it cannot ever catch this, because the harness
 *                stubs get_locinfo() as a cheap static return for BOTH sides: they both
 *                pay the same trivial cost, so the RATIO stays healthy while the real
 *                DLL burns three syscalls a character. A stub in the harness hid a real
 *                cost (§9.73, third variation).
 *
 * A ratio cannot express this. A COUNT can, exactly: `_wcsicmp` on an n-character string
 * must call get_locinfo() O(1) times, not O(n). That is deterministic, immune to machine
 * noise, and it is precisely the invariant the C's own structure encodes —
 * `_wcsicmp_l` resolves the locale ONCE (`get_current_locale_noalloc(&tmp)`) and passes
 * it down. See tests/hot_path.rs.
 * ------------------------------------------------------------------ */
static int subj_locinfo_calls;

int  subj_locinfo_calls_get(void)   { return subj_locinfo_calls; }
void subj_locinfo_calls_reset(void) { subj_locinfo_calls = 0; }

/* The subject's own "C" locale — same tables as the reference's, per the note above.
 *
 * WAVE 13 ADDED `lconv`.  It used to be left NULL, and the comment here used to say
 * "no flipped function reads it (it is the fpnum/strtod path, and the float family is
 * NOT flipped)".  That is exactly the kind of load-bearing assumption a harness quietly
 * accumulates — and the float family IS flipped now, so the subject's `fpnum_parse_bnum`
 * dereferences `locinfo->lconv->decimal_point` on every call.  A NULL here is not a
 * subtle divergence; it is a segfault in the test binary.
 *
 * Note the radix character must MATCH the reference's, for the usual reason: it is shared
 * ENVIRONMENT, not code under test.  If the two sides disagreed about what "." means, the
 * comparison would be measuring the harness rather than the Rust.
 *
 * `struct lconv` comes from the HOST's <locale.h> — which is also where the reference gets
 * it (refstub/msvcrt.h:57), so both sides read the same object through the same layout.
 * The subject, though, reaches it through BINDGEN's *Windows* layout, and the two only
 * agree because `decimal_point` is the first member of both.  That is a true fact about
 * C99 §7.11.1.1 and about corecrt, but it is not a layout GUARANTEE, so it is asserted at
 * run time rather than believed: see subj_off_lconv_decimal_point() and its ref_integrity
 * row. */
#include <locale.h>
static char subj_decimal_point[] = ".";
static struct lconv subj_lconv;

struct threadlocaleinfostruct *get_locinfo(void)
{
    subj_locinfo_calls++;
    if (!subj_locinfo.pcumap)
    {
        subj_lconv.decimal_point = subj_decimal_point;
        subj_locinfo.lconv       = &subj_lconv;
        subj_locinfo.mb_cur_max  = 1;
        subj_locinfo.lc_codepage = 0;
        /* lc_handle[] stays all-zero == the "C" locale. */
        subj_locinfo.pctype      = MSVCRT__ctype + 1;
        subj_locinfo.pclmap      = cloc_clmap;
        subj_locinfo.pcumap      = cloc_cumap;
    }
    return (struct threadlocaleinfostruct *)&subj_locinfo;
}

/* Per-thread CRT data.  The subject's strtok/wcstok cursors live here, independent of
 * the reference's — so a parity test must drive BOTH with the same call sequence.
 * Only the fields the flipped code touches are modelled; the Rust reaches them via
 * msvcrt-sys's bindgen offsets, which ref_integrity cross-checks against this C. */
typedef struct __thread_data {
    unsigned int   tid;
    void          *handle;
    int            thread_errno;
    unsigned int   thread_doserrno;
    int            unk1;
    unsigned int   random_seed;
    char          *strtok_next;
    unsigned short *wcstok_next;
    unsigned char *mbstok_next;
    /* …the real struct continues; we never touch past here, and ref_integrity proves
     * the offsets of the fields we DO touch match msvcrt-sys. */
    char           _tail[512];
} subj_thread_data_t;

static __thread subj_thread_data_t subj_thread_data;

void *msvcrt_get_thread_data(void)
{
    return &subj_thread_data;
}

/* The two errno slots the harness accessors at the top of this file read and write.
 * Defined HERE, beside the struct, rather than up there: errno.rs reaches the same two
 * fields through msvcrt-sys's bindgen offsets, and ref_integrity cross-checks those
 * against this declaration — so the harness and the code under test are provably looking
 * at the same bytes, which is the whole reason the storage moved out of two file-static
 * `__thread` variables. */
static int *subj_errno_slot(void)
{
    return &subj_thread_data.thread_errno;
}

static unsigned int *subj_doserrno_slot(void)
{
    return &subj_thread_data.thread_doserrno;
}

/* ── the SUBJECT's copy of the pinned MBCS fixture ─────────────────────────
 *
 * The Rust imports `get_mbcinfo` through msvcrt-sys, unmangled, so this is the
 * definition it binds to.  The reference's copy lives in refimpl.c behind
 * msvcrt_ref_get_mbcinfo and the two can never reach each other — which is the point:
 * a graded function that mutates its mbcinfo shows up as a divergence between the two
 * slots (mbcs_fixture_check) instead of silently changing what every later case is
 * graded against. */
static rl_threadmbcinfo subj_mbcinfo;

rl_threadmbcinfo *mbcs_subj_slot(void) { return &subj_mbcinfo; }

void *get_mbcinfo(void)
{
    return &subj_mbcinfo;
}

/* locale.c's "setlocale() has never been called" flag — NOT namespaced, so refimpl.c
 * owns the single definition and both sides branch on the same value. */
extern int initial_locale;

/* Constructor: wcs.c and the Rust both read MSVCRT__pwctype directly, without ever
 * calling an initialiser. */
__attribute__((constructor))
static void subj_locale_ctor(void)
{
    MSVCRT__pwctype = MSVCRT__wctype + 1;
    (void)get_locinfo();
}

/* ── the NON-"C"-LOCALE mode (wave 11) ─────────────────────────────
 *
 * The lane's oldest documented blind spot (§9.60): the harness pins the "C" locale, and a
 * whole family of msvcrt functions BRANCH on whether one is set —
 *
 *     if(!locinfo->lc_handle[LC_CTYPE]) { ...inline ASCII fold... }   // path 1: "C"
 *     else                              { ..._tolower_l / LCMapStringW... }   // path 2
 *
 * With lc_handle[] pinned to zero, path 2 was NEVER EXERCISED — so `_stricmp`, `_strnicmp`,
 * `_wcsicmp`, `_wcsnicmp`, `_strlwr_s`, `_strupr_s` and the wide twins could ship a
 * path-1-only implementation and the harness would report MATCH forever. It did, and they
 * did (waves 7-8).
 *
 * Planting a non-zero lc_handle makes path 2 live. What that grades is the Rust's LOGIC
 * around the NLS call — does it call the same function, with the same arguments, and
 * interpret the result the same way — because both sides now reach the SAME shared stub
 * (refimpl.c's). What it does NOT grade is Windows' real NLS *data*, which is correct:
 * that is kernel32's job, not msvcrt's, and it is the same bounded scope the
 * __crtLCMapStringA arms of _toupper_l already have.
 *
 * A green from an unexercised branch is not a green. This is how the branch gets exercised. */
extern const unsigned char *ref_nls_clmap(void);   /* refimpl.c — SHARED tables */
extern const unsigned char *ref_nls_cumap(void);

void subj_set_nls(int on)
{
    (void)get_locinfo();
    /* mb_cur_max stays 1: this is a single-byte NLS locale (en-US), not a multibyte one.
     * That keeps _isctype_l's table path live and isolates the change to exactly the
     * lc_handle[LC_CTYPE] branches.
     *
     * The TABLES are swapped too, and that is load-bearing — see ref_set_nls(). Planting
     * lc_handle alone makes msvcrt take path 2, but path 2 folds through pclmap[], and if
     * that is still the C locale's ASCII map then both paths give the same answer and a
     * path-1-only implementation is INDISTINGUISHABLE. The first version of this mode was
     * exactly that false green; the negative control caught it.
     *
     * The tables come from refimpl.c so BOTH sides fold with the same bytes. Data supplied
     * to both sides can never be caught differentially (§9.73), so we supply one copy. */
    subj_locinfo.lc_handle[RL_LC_CTYPE]   = on ? 0x0409u : 0u;
    subj_locinfo.lc_handle[RL_LC_COLLATE] = on ? 0x0409u : 0u;
    subj_locinfo.pclmap = on ? ref_nls_clmap() : cloc_clmap;
    subj_locinfo.pcumap = on ? ref_nls_cumap() : cloc_cumap;
}

/* ── the ref_integrity cross-check hooks ───────────────────────────
 * Rust's view of these structs comes from msvcrt-sys (bindgen). C's view is the two
 * definitions above. If they disagree, every locale-aware flip reads garbage and the
 * harness reports MATCH anyway (both sides would be reading the same garbage). So
 * ref_integrity asserts, field by field, that C's offsetof == Rust's offset_of!. */
#include <stddef.h>
size_t subj_off_locinfo_pctype(void) { return offsetof(rl_threadlocinfo, pctype); }
size_t subj_off_locinfo_pclmap(void) { return offsetof(rl_threadlocinfo, pclmap); }
size_t subj_off_locinfo_pcumap(void) { return offsetof(rl_threadlocinfo, pcumap); }
size_t subj_off_locinfo_mbcurmax(void) { return offsetof(rl_threadlocinfo, mb_cur_max); }
size_t subj_off_locinfo_lchandle(void) { return offsetof(rl_threadlocinfo, lc_handle); }
size_t subj_off_td_strtok(void)   { return offsetof(subj_thread_data_t, strtok_next); }
size_t subj_off_td_wcstok(void)   { return offsetof(subj_thread_data_t, wcstok_next); }
size_t subj_off_td_errno(void)    { return offsetof(subj_thread_data_t, thread_errno); }

/* wave 13 — the float parser's two-hop reach: locinfo -> lconv -> decimal_point.
 *
 * `lconv` here is the HOST's (glibc's) struct, because that is the one the reference
 * reads through; the Rust reaches the SAME object through bindgen's Windows layout.  They
 * agree only because `decimal_point` is the first member on both sides.  True today, and
 * asserted so it stays true — a silent disagreement would make every strtod row compare a
 * radix character against garbage, and the whole float family would go DIVERGE with a
 * completely misleading cause. */
size_t subj_off_locinfo_lconv(void)      { return offsetof(rl_threadlocinfo, lconv); }
size_t subj_off_lconv_decimalpoint(void) { return offsetof(struct lconv, decimal_point); }

/* ── mbcs: the MBCS state every mbcs.c entry point dereferences ─────────────
 *
 * These matter more than the locinfo offsets above, not less.  A `threadmbcinfo` is
 * almost entirely two byte arrays, and `mbctype[]` IS the answer to "is this a lead
 * byte" for all 160 graded functions — so a wrong offset here does not produce a
 * plausible-but-wrong verdict, it produces a *stable* one: both sides read the same
 * bytes past the end of the object and agree with each other perfectly, forever.
 *
 * The size check is not redundant with the offsets.  `mbcasemap` is last, so its
 * offset is right even if the array is short; only sizeof catches a truncated tail. */
size_t subj_off_mbcinfo_mbcodepage(void)   { return offsetof(rl_threadmbcinfo, mbcodepage); }
size_t subj_off_mbcinfo_ismbcodepage(void) { return offsetof(rl_threadmbcinfo, ismbcodepage); }
size_t subj_off_mbcinfo_mblcid(void)       { return offsetof(rl_threadmbcinfo, mblcid); }
size_t subj_off_mbcinfo_mbulinfo(void)     { return offsetof(rl_threadmbcinfo, mbulinfo); }
size_t subj_off_mbcinfo_mbctype(void)      { return offsetof(rl_threadmbcinfo, mbctype); }
size_t subj_off_mbcinfo_mbcasemap(void)    { return offsetof(rl_threadmbcinfo, mbcasemap); }
size_t subj_sizeof_mbcinfo(void)           { return sizeof(rl_threadmbcinfo); }
size_t subj_off_locale_mbcinfo(void)       { return offsetof(rl_locale_t, mbcinfo); }

/* ── wave 14b: the Rust printf ENGINE's environment ─────────────────────────
 *
 * msvcrt.dll's printf engine imports these from kernel32 / its own C.  The
 * reference's copies live in the msvcrt_ref_* namespace (refstub/msvcrt.h) —
 * deliberately unreachable — so the SUBJECT gets its own here.  A heap is a
 * heap (malloc IS the faithful model, same argument as refimpl's), and the
 * engine only ever compares heap CONTENTS, never identities. */
#include <stdlib.h>

static int subj_process_heap;

void *GetProcessHeap(void)
{
    return &subj_process_heap;
}

void *HeapAlloc(void *heap, unsigned int flags, size_t size)
{
    void *p;

    (void)heap;
    p = malloc(size ? size : 1);
    if (p && (flags & 0x8 /* HEAP_ZERO_MEMORY */))
    {
        unsigned char *b = p;
        size_t i;
        for (i = 0; i < size; i++) b[i] = 0;
    }
    return p;
}

int HeapFree(void *heap, unsigned int flags, void *ptr)
{
    (void)heap; (void)flags;
    free(ptr);
    return 1;
}

/* printf's %e exponent width used to be stubbed here, returning 0.  It is now supplied by
 * the crate under test — `msvcrt/misc.rs`'s real `_get_output_format`, reading the real
 * `output_format` static that `_set_output_format` writes — so a stub would be a duplicate
 * symbol at link.  The default is still 0 (3-digit form), so refimpl's `msvcrt_ref_` twin
 * continues to agree and no %e row moves; the difference is that the agreement is now a
 * property of the shipping code rather than of two hand-kept-in-sync stubs. */

/* `_mbtowc_l` USED to be a stub here — a C-locale 1:1 twin of the one in refimpl.c,
 * required to stay body-identical to it or every wide-conversion printf row would
 * false-DIVERGE.  Both stubs are gone.
 *
 * The SHIPPING RUST defines `_mbtowc_l` (msvcrt-crt-rust's mbcs.rs), and as of the mbcs
 * TU the reference has the genuine C one as `ref__mbtowc_l`.  They are still twins —
 * but now they are the REAL implementations rather than two matched fakes, so printf's
 * wide conversion is graded against actual mb->wc behaviour instead of against an
 * agreement we wrote ourselves.
 *
 * Keeping this would have been a duplicate-symbol link error the moment the Rust mbcs
 * objects entered the link, which is how it was found. */

/* ── `_snprintf`, for the SUBJECT ────────────────────────────────────────────
 *
 * time.rs's `asctime_buf` calls `_snprintf`. In the shipping DLL that name is a
 * `va_start` trampoline in dll/win32/msvcrt/rust_varargs_shim.c, compiled into the
 * msvcrt MODULE and forwarding a `va_list` into the Rust engine. That TU cannot come
 * into this harness: the Rust `_vsnprintf` walks a `*mut u8` over 8-byte slots — the
 * Windows va_list — and the host's `va_start` produces a SysV register-save-area
 * struct. Feeding one to the other is the exact wall printf_probe hit, and it is why
 * the reference TUs get refstub_printf's flat <stdarg.h>.
 *
 * So the harness answers instead, over the host's vsnprintf. Be precise about what
 * that costs: `asctime`/`_wasctime`/`ctime` rows grade the ASSEMBLY of the string —
 * the field order, the range checks, the `'1' + (year+900)/1000` century arithmetic —
 * and NOT msvcrt's formatting of it. That is a real limit, and a narrow one: the only
 * format asctime uses is `%s %s %02d %02d:%02d:%02d %c%03d`, on which glibc and msvcrt
 * agree exactly. msvcrt's printf engine itself is graded by printf_port.rs against
 * `ref_pf_printf_a`, which is the genuine C — the coverage lives there, not here.
 *
 * The TRUNCATION semantics are msvcrt's, not C99's, because those genuinely differ:
 * msvcrt writes no terminator and returns -1 when the result does not fit, where C99
 * NUL-terminates and returns the length it wanted. Modelling C99 here would make a
 * short-buffer row pass on the reference and fail in the DLL.
 */
int _snprintf(char *str, size_t count, const char *format, ...)
{
    va_list ap;
    int n;

    va_start(ap, format);
    n = vsnprintf(str, count + 1, format, ap);   /* +1: ask glibc for the NUL slot */
    va_end(ap);

    if (n < 0) return -1;
    if ((size_t)n > count) {
        /* Did not fit. msvcrt leaves the buffer unterminated and reports -1. */
        return -1;
    }
    return n;
}

/* ------------------------------------------------------------------------
 * math.c's classification entry points, for the host harness.
 *
 * `math.rs` reaches these through the header's `fpclassify` macro, so they are
 * external calls in the object even though the C source never spells one. On the
 * host they must be MAPPED, not forwarded: ReactOS's msvcrt spells the results
 *
 *     FP_ZERO 0   FP_INFINITE 1   FP_NAN 2   FP_NORMAL -1   FP_SUBNORMAL -2
 *
 * (dll/win32/msvcrt/include/math.h:277-281), and glibc uses a completely
 * different, all-positive set. Returning the host's numbers would make every
 * classification comparison in `_fpclass`, `sqrt_validate` and `isnan` silently
 * wrong in a way that still typechecks — the exact class this harness exists to
 * catch, so the mapping is spelled out rather than assumed.
 * ------------------------------------------------------------------------ */
#include <math.h>
#include <stdarg.h>
#include <stdio.h>

#define MSVCRT_FP_INFINITE 1
#define MSVCRT_FP_NAN 2
#define MSVCRT_FP_NORMAL (-1)
#define MSVCRT_FP_SUBNORMAL (-2)
#define MSVCRT_FP_ZERO 0

static short msvcrt_classify(int host_class)
{
    switch (host_class)
    {
    case FP_NAN:       return MSVCRT_FP_NAN;
    case FP_INFINITE:  return MSVCRT_FP_INFINITE;
    case FP_ZERO:      return MSVCRT_FP_ZERO;
    case FP_SUBNORMAL: return MSVCRT_FP_SUBNORMAL;
    default:           return MSVCRT_FP_NORMAL;
    }
}

short _dclass(double x)
{
    return msvcrt_classify(fpclassify(x));
}

short _fdclass(float x)
{
    return msvcrt_classify(fpclassify(x));
}

/* `_scprintf` is "how long would this printf be" — vsnprintf into nothing. */
int _scprintf(const char *format, ...)
{
    va_list ap;
    int n;

    va_start(ap, format);
    n = vsnprintf(NULL, 0, format, ap);
    va_end(ap);
    return n;
}
