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

#include <stddef.h>
#include <stdint.h>

/* ------------------------------------------------------------------ *
 * errno — the SUBJECT's, independent of the reference's.
 * ------------------------------------------------------------------ */
static __thread int subj_errno_value;

int *_errno(void)
{
    return &subj_errno_value;
}

/* Harness API (un-namespaced on purpose — these are read by classify.rs). */
int  subj_errno_get(void)      { return subj_errno_value; }
void subj_errno_set(int value) { subj_errno_value = value; }

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

/* ------------------------------------------------------------------ *
 * GetStringTypeW — the NLS entry point behind the wc >= 256 path.
 *
 * msvcrt's `_iswctype_l` (wcs.c) is:
 *
 *     if (wc == WEOF)  return 0;
 *     if (wc < 256)    return MSVCRT__pwctype[wc] & type;
 *     if (!GetStringTypeW(CT_CTYPE1, &wc, 1, &ct)) return 0;
 *     return ct & type;
 *
 * WAVE 10 DELETED A FAKE HERE.  Wave 9 hand-wrote a 256-entry `MSVCRT__pwctype`
 * table in this file, spelled out from first principles, because ctype.c was not
 * yet a reference TU and nothing else defined the symbol.  That made the harness
 * grade the Rust against *our own model of msvcrt* rather than against msvcrt.
 *
 * ctype.c is now reference TU #3, and it owns the real thing:
 *
 *     WORD  MSVCRT__wctype[257];
 *     WORD *MSVCRT__pwctype = MSVCRT__wctype + 1;   <-- note the +1, as with pctype
 *
 * so both sides now read msvcrt's genuine wide ctype table and the model is gone.
 * (The Rust's *indexing* was already right — `MSVCRT__pwctype[wc]`, matching the C.
 * It is the DATA that was ours, and data we supply to both sides can never be
 * caught by a differential test: a mistyped byte makes both sides wrong the same
 * way and classify still reports MATCH.  Delete fakes; do not grade them.)
 *
 * The harness is a HOST (linux-x86_64) binary, where WINAPI is a no-op, so plain
 * cdecl is right here. In msvcrt.dll itself this resolves to kernel32's stdcall
 * export and the Rust declares it `extern "system"` accordingly — see the flip.
 * ------------------------------------------------------------------ */
/* Defined below, aimed at ctype.c's shared MSVCRT__wctype[] by a constructor. */
extern unsigned short *MSVCRT__pwctype;

int GetStringTypeW(unsigned int type, const unsigned short *src, int count,
                   unsigned short *chartype)
{
    int i, n = count;

    (void)type;
    if (n < 0)
    {
        n = 0;
        while (src[n]) n++;
        n++;
    }
    for (i = 0; i < n; i++)
        chartype[i] = (unsigned short)(src[i] < 256 ? MSVCRT__pwctype[src[i]] : 0);
    return 1;
}

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

/* The subject's own "C" locale — same tables as the reference's, per the note above.
 * `lconv` is left NULL: no flipped function reads it (it is the fpnum/strtod path, and
 * the float family is NOT flipped — see the wave-5 probe). */
struct threadlocaleinfostruct *get_locinfo(void)
{
    if (!subj_locinfo.pcumap)
    {
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
