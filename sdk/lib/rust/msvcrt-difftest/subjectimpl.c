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
 * The wide ctype table + GetStringTypeW — the SUBJECT's copies (wave 9).
 *
 * msvcrt's `_iswctype_l` does not consult the locale at all:
 *
 *     if (wc == WEOF)  return 0;
 *     if (wc < 256)    return MSVCRT__pwctype[wc] & type;
 *     if (!GetStringTypeW(CT_CTYPE1, &wc, 1, &ct)) return 0;
 *     return ct & type;
 *
 * so the Rust flip reads the same global and calls the same NLS entry point. In
 * the REAL DLL those are msvcrt's own table and kernel32's GetStringTypeW, which
 * is what makes the flip locale-CORRECT rather than merely C-locale-correct.
 *
 * In the harness they have to be supplied here, because refimpl's are renamed
 * into the private `msvcrt_ref_*` namespace (refstub/msvcrt.h) — the namespace
 * that stops the reference binding to the code under test. Same reasoning, and
 * same separate storage, as _errno above.
 *
 * The table is spelled out from first principles, identical to refimpl's model,
 * so a divergence here is a divergence in the Rust's MASKING LOGIC — which is the
 * only part under test, since in the DLL both sides read one shared table.
 * ------------------------------------------------------------------ */
#define S_UPPER   0x0001
#define S_LOWER   0x0002
#define S_DIGIT   0x0004
#define S_SPACE   0x0008
#define S_PUNCT   0x0010
#define S_CONTROL 0x0020
#define S_BLANK   0x0040
#define S_HEX     0x0080
#define S_ALPHA   (0x0100 | S_UPPER | S_LOWER)

typedef unsigned short subj_word;

static subj_word subj_wctype_table[256];
static subj_word *subj_pwctype_ptr;

static void subj_init_wctype(void)
{
    int c;

    if (subj_pwctype_ptr) return;
    for (c = 0; c < 256; c++)
    {
        subj_word f = 0;
        if (c >= 'A' && c <= 'Z')                    f |= S_UPPER;
        if (c >= 'a' && c <= 'z')                    f |= S_LOWER;
        if (c >= '0' && c <= '9')                    f |= S_DIGIT | S_HEX;
        if ((c >= 'a' && c <= 'f') ||
            (c >= 'A' && c <= 'F'))                  f |= S_HEX;
        if (c == ' ' || (c >= '\t' && c <= '\r'))    f |= S_SPACE;
        if (c == ' ' || c == '\t')                   f |= S_BLANK;
        if (c < 0x20 || c == 0x7f)                   f |= S_CONTROL;
        if ((c >= '!' && c <= '/') || (c >= ':' && c <= '@') ||
            (c >= '[' && c <= '`') || (c >= '{' && c <= '~')) f |= S_PUNCT;
        if (f & (S_UPPER | S_LOWER))                 f |= S_ALPHA;
        subj_wctype_table[c] = f;
    }
    subj_pwctype_ptr = subj_wctype_table;
}

/* msvcrt exports this as a POINTER to the table — the Rust reads it as one. */
subj_word *MSVCRT__pwctype;

__attribute__((constructor))
static void subj_wctype_ctor(void)
{
    subj_init_wctype();
    MSVCRT__pwctype = subj_pwctype_ptr;
}

/* The wc >= 256 path. Same model as refimpl's, so the comparison isolates the
 * Rust's masking rather than the NLS data.
 *
 * The harness is a HOST (linux-x86_64) binary, where WINAPI is a no-op, so plain
 * cdecl is right here. In msvcrt.dll itself this resolves to kernel32's stdcall
 * export and the Rust declares it `extern "system"` accordingly — see the flip. */
int GetStringTypeW(unsigned int type, const unsigned short *src, int count,
                   unsigned short *chartype)
{
    int i, n = count;

    (void)type;
    subj_init_wctype();
    if (n < 0)
    {
        n = 0;
        while (src[n]) n++;
        n++;
    }
    for (i = 0; i < n; i++)
        chartype[i] = (subj_word)(src[i] < 256 ? subj_wctype_table[src[i]] : 0);
    return 1;
}
