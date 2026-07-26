/* nlsimpl.c — the codepage-aware half of the harness's NLS environment.
 *
 * WHY THIS EXISTS SEPARATELY FROM refimpl.c
 *
 * refimpl.c already models CompareStringA / LCMapStringA / GetStringTypeW /
 * WideCharToMultiByte, and says of itself: "What the stub does NOT model is Windows'
 * actual NLS *data*."  That was a sound, bounded blind spot for string.c and wcs.c,
 * which reach NLS only when a test plants an lc_handle — the C locale never gets there.
 *
 * mbcs.c makes that blind spot load-bearing.  `create_mbcinfo()` builds the
 * `threadmbcinfo` — the lead/trail-byte table that IS the answer to every question all
 * 160 graded functions ask — out of GetCPInfo + MultiByteToWideChar + GetStringTypeW +
 * LCMapStringW.  Hand those a codepage-blind model and CP932 comes back with no lead
 * bytes at all: `ismbcodepage` is 0, every DBCS branch in every graded function is
 * unreachable, and the harness grades the single-byte paths only — then reports a
 * confident green over the half of mbcs.c that actually has the bugs.
 *
 * So the codepage has to be real.  It comes from the HOST's iconv, which implements
 * these exact Windows codepages (CP932/936/949/950/1361).  That is not the same as
 * Windows' NLS, and it does not need to be: both the reference and the subject call
 * through here, so what is being graded is mbcs.c's LOGIC around the conversion, never
 * the conversion itself.  What matters is that the model is (a) deterministic and
 * (b) rich enough to drive the real branches — which a genuine DBCS table is and a
 * latin-1 identity map is not.
 *
 * Nothing here is transcribed.  The lead-byte ranges are PROBED out of iconv rather
 * than typed in from a table, for the reason refimpl.c gives about the case-mapping
 * tables: a hand copy is invisible-wrong, because the reference and the subject share
 * one fixture and a mistyped byte makes them both wrong the same way.
 *
 * Not compiled with the rename prelude — like refimpl.c, this is the ENVIRONMENT, and
 * there is exactly ONE copy that both sides call.
 */

#define _GNU_SOURCE   /* newlocale / towlower_l / iswupper_l */

#include <errno.h>
#include <stdlib.h>  /* abort() — libc_fn fails loud */
#include <iconv.h>
#include <locale.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>
#include <wctype.h>

/* Host headers FIRST — nlsimpl.h remaps `wchar_t` to 16 bits for the Windows model,
 * and no host header may be parsed under that remap. */
#include "nlsimpl.h"

/* ------------------------------------------------------------------ *
 * iconv handles, cached.
 *
 * An exhaustive case runs 0..=0xFFFF across six codepages — 393,216 calls, several of
 * which reach a conversion.  iconv_open() per call would dominate the run time of the
 * thing being measured, so each (codepage, direction) pair is opened once.
 * ------------------------------------------------------------------ */

/* The codepages a fixture can pin.  ASCII (20127) is deliberately present and
 * deliberately has no iconv name: mbcs.c's create_mbcinfo has its own built-in
 * fallback for it ("if (newcp == 20127) ... MaxCharSize = 1, DefaultChar '?'"), reached
 * precisely when GetCPInfo FAILS.  Letting the probe fail is how that real path gets
 * exercised, so do not "fix" it by mapping 20127 to ANSI_X3.4-1968. */
static const struct { unsigned int cp; const char *iconv_name; unsigned int lcid; }
CP_TABLE[] = {
    {  932, "CP932",  0x0411 },  /* ja-JP  Shift-JIS   */
    {  936, "CP936",  0x0804 },  /* zh-CN  GBK         */
    {  949, "CP949",  0x0412 },  /* ko-KR  EUC-KR      */
    {  950, "CP950",  0x0404 },  /* zh-TW  Big5        */
    { 1361, "CP1361", 0x0412 },  /* ko-KR  Johab       */
    { 1252, "CP1252", 0x0409 },  /* en-US  latin-1     */
    { 20127, NULL,    0x0409 },  /* ASCII: no iconv name, see above */
};
#define CP_COUNT ((int)(sizeof(CP_TABLE) / sizeof(CP_TABLE[0])))

static int cp_index(unsigned int cp)
{
    int i;
    for (i = 0; i < CP_COUNT; i++)
        if (CP_TABLE[i].cp == cp) return i;
    return -1;
}

/* Windows wchar_t is 16-bit, so the wide side is always UTF-16LE. */
#define WIDE_ENC "UTF-16LE"

static iconv_t to_wide_cache[CP_COUNT];
static iconv_t to_mb_cache[CP_COUNT];

static iconv_t cp_to_wide(unsigned int cp)
{
    int i = cp_index(cp);
    if (i < 0 || !CP_TABLE[i].iconv_name) return (iconv_t)-1;
    if (!to_wide_cache[i])
        to_wide_cache[i] = iconv_open(WIDE_ENC, CP_TABLE[i].iconv_name);
    return to_wide_cache[i];
}

static iconv_t cp_to_mb(unsigned int cp)
{
    int i = cp_index(cp);
    if (i < 0 || !CP_TABLE[i].iconv_name) return (iconv_t)-1;
    if (!to_mb_cache[i])
        to_mb_cache[i] = iconv_open(CP_TABLE[i].iconv_name, WIDE_ENC);
    return to_mb_cache[i];
}

/* ------------------------------------------------------------------ *
 * Byte classification, probed.
 * ------------------------------------------------------------------ */

#define BYTE_SINGLE  0
#define BYTE_LEAD    1
#define BYTE_INVALID 2

/* Ask iconv what a lone byte is.
 *
 * EINVAL means "incomplete sequence at end of input" — the byte is a LEAD byte and
 * iconv is waiting for its trail.  EILSEQ means the byte is not valid at all.  Anything
 * converted means it stands alone.  This is a question about the codepage's structure,
 * so it is asked, not tabulated. */
static int classify_byte(unsigned int cp, unsigned char b)
{
    iconv_t cd = cp_to_wide(cp);
    char inbuf[1], outbuf[8];
    char *in = inbuf, *out = outbuf;
    size_t inleft = 1, outleft = sizeof(outbuf);
    size_t r;

    if (cd == (iconv_t)-1) return BYTE_SINGLE;
    inbuf[0] = (char)b;
    iconv(cd, NULL, NULL, NULL, NULL);            /* reset shift state */
    errno = 0;
    r = iconv(cd, &in, &inleft, &out, &outleft);
    if (r == (size_t)-1)
    {
        if (errno == EINVAL) return BYTE_LEAD;
        return BYTE_INVALID;
    }
    return BYTE_SINGLE;
}

/* ------------------------------------------------------------------ *
 * kernel32: the pieces create_mbcinfo needs that refimpl.c never had.
 * ------------------------------------------------------------------ */

unsigned int WINAPI GetACP(void)   { return 1252; }
unsigned int WINAPI GetOEMCP(void) { return 437; }

/* MaxCharSize + the lead-byte RANGES, in the CPINFO shape create_mbcinfo walks
 * (`while (bytes[0] || bytes[1])` over pairs).  Both are derived from the probe above,
 * so they move when iconv's tables do and can never disagree with the conversions that
 * every other function here performs. */
BOOL WINAPI GetCPInfo(unsigned int cp, CPINFO *info)
{
    int i, nlead = 0;
    int in_run = 0;

    if (!info) return FALSE;
    if (cp_index(cp) < 0 || !CP_TABLE[cp_index(cp)].iconv_name)
        return FALSE;             /* 20127 lands here — create_mbcinfo's own fallback */

    memset(info, 0, sizeof(*info));
    info->DefaultChar[0] = '?';
    info->MaxCharSize = 1;

    for (i = 0; i < 256; i++)
    {
        int lead = classify_byte(cp, (unsigned char)i) == BYTE_LEAD;
        if (lead)
        {
            info->MaxCharSize = 2;
            if (!in_run)
            {
                if (nlead + 2 >= MAX_LEADBYTES) break;   /* keep the {0,0} terminator */
                info->LeadByte[nlead++] = (BYTE)i;
                in_run = 1;
            }
            info->LeadByte[nlead] = (BYTE)i;             /* extend the run's end */
        }
        else if (in_run)
        {
            nlead++;
            in_run = 0;
        }
    }
    if (in_run) nlead++;
    return TRUE;
}

/* iconv, in MultiByteToWideChar's clothing.
 *
 * create_mbcinfo hands this every non-lead byte at once and checks the returned count
 * against what it passed; a short result makes it ERR and carry on with a partly-built
 * table.  So an unconvertible byte must still CONSUME one input and produce one output
 * (U+FFFD) rather than stopping — Windows with no MB_ERR_INVALID_CHARS behaves the same
 * way, and stopping would silently truncate the table for every codepage that has a
 * hole in it (CP932 has several). */
int WINAPI MultiByteToWideChar(unsigned int cp, DWORD flags, const char *src, int srclen,
                               wchar_t *dst, int dstlen)
{
    iconv_t cd = cp_to_wide(cp);
    int produced = 0;
    int consumed = 0;

    (void)flags;
    if (srclen < 0) srclen = (int)strlen(src) + 1;
    if (cd == (iconv_t)-1)
    {
        /* No iconv name (ASCII): 1:1 for < 128, U+FFFD above. */
        int i;
        if (!dstlen) return srclen;
        for (i = 0; i < srclen && i < dstlen; i++)
            dst[i] = (unsigned char)src[i] < 128 ? (wchar_t)(unsigned char)src[i]
                                                 : (wchar_t)0xFFFD;
        return i;
    }

    iconv(cd, NULL, NULL, NULL, NULL);
    while (consumed < srclen)
    {
        char *in = (char *)src + consumed;
        size_t inleft = (size_t)(srclen - consumed);
        wchar_t scratch[8];
        char *out = (char *)scratch;
        size_t outleft = sizeof(scratch);
        size_t before = inleft;
        size_t r;

        errno = 0;
        r = iconv(cd, &in, &inleft, &out, &outleft);
        {
            int got = (int)((sizeof(scratch) - outleft) / sizeof(wchar_t));
            int j;
            for (j = 0; j < got; j++)
            {
                if (dstlen && produced >= dstlen) return produced;
                if (dstlen) dst[produced] = scratch[j];
                produced++;
            }
            consumed += (int)(before - inleft);
        }
        if (r == (size_t)-1 && (errno == EILSEQ || errno == EINVAL))
        {
            /* Unconvertible or truncated: emit one replacement and step over one byte,
             * so the caller's 1-byte-in/1-wchar-out accounting still lines up. */
            if (dstlen && produced >= dstlen) return produced;
            if (dstlen) dst[produced] = (wchar_t)0xFFFD;
            produced++;
            consumed++;
            iconv(cd, NULL, NULL, NULL, NULL);
        }
        else if (r == (size_t)-1 && errno == E2BIG)
        {
            continue;    /* scratch full; loop refills it */
        }
    }
    return produced;
}

/* ------------------------------------------------------------------ *
 * Locale naming.  create_mbcinfo turns a codepage into an LCID by spelling it ".<cp>"
 * and round-tripping through locale_to_sname + LocaleNameToLCID.  We model that pair
 * as the CP_TABLE lookup it effectively is, keeping the ".<cp>" spelling so the C's
 * own sprintf/parse path still runs.
 * ------------------------------------------------------------------ */

BOOL locale_to_sname(const char *locale, unsigned short *codepage, BOOL *sname_match,
                     wchar_t *sname)
{
    unsigned int cp;
    int i;

    if (sname_match) *sname_match = FALSE;
    if (!locale || locale[0] != '.') return FALSE;
    if (sscanf(locale + 1, "%u", &cp) != 1) return FALSE;
    if (cp_index(cp) < 0) return FALSE;
    if (codepage) *codepage = (unsigned short)cp;

    /* Hand back the codepage's decimal digits as the "name"; LocaleNameToLCID below is
     * the only reader, and this keeps the two halves impossible to get out of step. */
    if (sname)
    {
        char buf[16];
        int n = snprintf(buf, sizeof(buf), "%u", cp);
        for (i = 0; i < n; i++) sname[i] = (wchar_t)(unsigned char)buf[i];
        sname[n] = 0;
    }
    return TRUE;
}

unsigned int WINAPI LocaleNameToLCID(const wchar_t *name, DWORD flags)
{
    unsigned int cp = 0;
    int i, idx;

    (void)flags;
    if (!name) return 0;
    for (i = 0; name[i]; i++)
    {
        if (name[i] < '0' || name[i] > '9') return 0;
        cp = cp * 10 + (unsigned int)(name[i] - '0');
    }
    idx = cp_index(cp);
    return idx < 0 ? 0 : CP_TABLE[idx].lcid;
}

/* ------------------------------------------------------------------ *
 * Which codepage the LCID-only entry points should use.
 *
 * CompareStringA and LCMapStringW get an LCID, not a codepage.  Rather than invent a
 * reverse map (several codepages share an LCID — 949 and 1361 are both ko-KR), the
 * currently installed fixture is the answer, and it is exactly the one mbcs.c would
 * have used: every call site reaches them through `mbcinfo->mblcid`.
 * ------------------------------------------------------------------ */
static unsigned int active_cp = 1252;

unsigned int nls_active_codepage(void)          { return active_cp; }
void         nls_set_active_codepage(unsigned int cp) { active_cp = cp; }

int nls_is_dbcs_codepage(unsigned int cp)
{
    int i = cp_index(cp);
    if (i < 0 || !CP_TABLE[i].iconv_name) return 0;
    return cp == 932 || cp == 936 || cp == 949 || cp == 950 || cp == 1361;
}

unsigned int nls_lcid_for_codepage(unsigned int cp)
{
    int i = cp_index(cp);
    return i < 0 ? 0 : CP_TABLE[i].lcid;
}

int nls_codepage_count(void) { return CP_COUNT; }

unsigned int nls_codepage_at(int i)
{
    return (i < 0 || i >= CP_COUNT) ? 0 : CP_TABLE[i].cp;
}

/* ------------------------------------------------------------------ *
 * The pinned fixture.
 *
 * Built by the reference's OWN create_mbcinfo.  A hand-written filler would be the
 * §9.73 trap in its purest form: data handed to both sides can never be caught by a
 * differential test, because a mistake makes both sides wrong the same way.  Letting
 * the reference C build it means the only thing left to get right is the NLS beneath
 * it — and that is what the probes above exist to check.
 * ------------------------------------------------------------------ */

/* mbcs.c's create_mbcinfo, after the prelude's rename.  Declared rather than included:
 * this TU is compiled WITHOUT the rename prelude (it is the environment), so it must
 * name the renamed symbol explicitly. */
extern rl_threadmbcinfo *ref_create_mbcinfo(int cp, unsigned int lcid,
                                            rl_threadmbcinfo *old);

static rl_threadmbcinfo pristine;
static int              pristine_valid;

const rl_threadmbcinfo *mbcs_fixture_pristine(void)
{
    return pristine_valid ? &pristine : (const rl_threadmbcinfo *)0;
}

int mbcs_fixture_install(unsigned int cp)
{
    rl_threadmbcinfo *built;

    /* The active codepage must be set BEFORE create_mbcinfo runs: it is what the
     * LCID-only entry points (CompareStringA, LCMapStringW) consult, and
     * create_mbcinfo calls LCMapStringW while building the case map. */
    nls_set_active_codepage(cp);

    built = ref_create_mbcinfo((int)cp, (unsigned int)-1, (rl_threadmbcinfo *)0);
    if (!built) return -1;

    pristine = *built;
    pristine_valid = 1;
    *mbcs_ref_slot()  = pristine;
    *mbcs_subj_slot() = pristine;
    return 0;
}

/* How many bytes `__p__mbctype` hands back.
 *
 * From `sizeof`, never typed out: the case that grades that accessor compares the bytes
 * behind the returned pointer, and a length constant transcribed into the harness is part
 * of the oracle — too small and the tail of the table is never compared, too large and the
 * comparison reads off the end of the struct.  This reads the mirror in locale_layout.h,
 * which ref_integrity already asserts field-by-field against msvcrt-sys, so the number
 * cannot drift away from the struct it describes. */
size_t mbcs_mbctype_len(void)
{
    return sizeof(((rl_threadmbcinfo *)0)->mbctype);
}

/* Did either side scribble on it?  The fixture is read-only state; a graded function
 * that writes to it is a translation bug, and without this check it would not surface
 * as a failure in that function — it would surface as a divergence in whichever
 * unrelated case happened to run next. */
int mbcs_fixture_check(void)
{
    if (!pristine_valid) return 0;
    if (memcmp(mbcs_ref_slot(),  &pristine, sizeof(pristine)) != 0) return 1;
    if (memcmp(mbcs_subj_slot(), &pristine, sizeof(pristine)) != 0) return 2;
    return 0;
}

/* ------------------------------------------------------------------ *
 * Odds and ends the mbcs.c reference TU imports.
 * ------------------------------------------------------------------ */

/* locale.c's, and it only ever has to not-crash here: the fixture slots are static
 * storage, never malloc'd, so nothing this frees was ever allocated by us.
 * create_mbcinfo's OWN return value IS malloc'd, and mbcs_fixture_install copies out of
 * it and lets it go — one small leak per codepage, once, deliberately: freeing it would
 * mean modelling msvcrt's refcount discipline for no gain in a process that exits. */
void free_mbcinfo(rl_threadmbcinfo *mbcinfo) { (void)mbcinfo; }

/* locale.c's global. mbcs.c reads it in _setmbcp (which is refused as a state mutator,
 * but must still link). NULL is what msvcrt has before setlocale(). */
void *MSVCRT_locale = 0;

/* mbcs.c's create_mbcinfo bumps a refcount with this. Single-threaded harness, so the
 * atomicity is not load-bearing — the RETURN VALUE is: it must be the INCREMENTED
 * value, which is what the caller compares against. */
int InterlockedIncrement(int volatile *addend) { return ++*addend; }

/* ------------------------------------------------------------------ *
 * Wide classification and case mapping.
 *
 * create_mbcinfo asks GetStringTypeW/LCMapStringW about the wide form of every
 * single-byte character, and for a DBCS codepage many of those land well above U+00FF
 * (CP932's 0xA1-0xDF are halfwidth katakana at U+FF61-U+FF9F).  refimpl.c's model
 * answers 0 for everything >= 256, which would leave `mbcasemap` empty and silently
 * un-grade every case-mapping path in the file.
 *
 * A PER-CALL locale, never setlocale(): the harness's own locale modelling is a
 * hand-built `threadlocinfo`, and the reference reads a host `struct lconv` through it
 * (see subj_off_lconv_decimalpoint).  Moving the global locale underneath that would
 * change what the float family sees, in a component whose job is to hold still.
 * ------------------------------------------------------------------ */
static locale_t wide_locale(void)
{
    static locale_t loc;
    static int tried;
    if (!tried)
    {
        tried = 1;
        loc = newlocale(LC_CTYPE_MASK, "C.utf8", (locale_t)0);
    }
    return loc;
}

/* ── reach the REAL libc, not whatever the subject exports ──────────────────
 *
 * THIS IS LOAD-BEARING, and it took about ten seconds to prove it.
 *
 * The finished binary links these objects together with msvcrt-crt-rust's
 * `#[no_mangle]` shims, and a static definition beats the shared libc. The Rust today
 * exports `iswalpha_l`, `towlower` and `towupper`. So a plain `iswalpha_l(c, loc)` call
 * here bound straight to the CODE UNDER TEST, which called `GetStringTypeW`, which
 * called back into `nls_ctype1` — infinite recursion, stack overflow on the first case.
 *
 * That is the same hazard refstub/msvcrt.h's namespace block exists for, but a `#define`
 * rename cannot fix it: the collision is at LINK time, and renaming would just ask glibc
 * for a name it does not have. And the shadow set GROWS — `iswupper_l` is glibc's today
 * only because nobody has translated it yet, so a fix that enumerates today's collisions
 * is a fix with an expiry date.
 *
 * So: resolve through the dynamic loader, by name, out of libc itself. Whatever the
 * subject defines, this reaches the genuine implementation.
 */
#include <dlfcn.h>

static void *libc_fn(const char *name)
{
    static void *libc;
    if (!libc)
    {
        /* NOLOAD: libc is already mapped; this just takes a handle to it. */
        libc = dlopen("libc.so.6", RTLD_LAZY | RTLD_NOLOAD);
        if (!libc) libc = RTLD_NEXT;
    }
    void *fn = dlsym(libc, name);
    /* Fail loudly. A NULL here would otherwise become "this character is in no class",
     * i.e. an empty mbcasemap and a green run that graded nothing. */
    if (!fn)
    {
        fprintf(stderr,
                "msvcrt-difftest: cannot resolve `%s` from libc — the NLS model cannot\n"
                "classify characters, and every case would be graded against zeros.\n",
                name);
        abort();
    }
    return fn;
}

typedef wint_t (*host_towctrans_fn)(wint_t, locale_t);
typedef int (*host_isw_fn)(wint_t, locale_t);

#define HOST_ISW(name)                                                     \
    ({                                                                     \
        static host_isw_fn _fn;                                            \
        if (!_fn) _fn = (host_isw_fn)libc_fn(#name);                       \
        _fn;                                                               \
    })

#define HOST_TOW(name)                                                     \
    ({                                                                     \
        static host_towctrans_fn _fn;                                      \
        if (!_fn) _fn = (host_towctrans_fn)libc_fn(#name);                 \
        _fn;                                                               \
    })

unsigned int nls_towlower(unsigned int wc)
{
    locale_t loc = wide_locale();
    return loc ? (unsigned int)HOST_TOW(towlower_l)((wint_t)wc, loc) : wc;
}

unsigned int nls_towupper(unsigned int wc)
{
    locale_t loc = wide_locale();
    return loc ? (unsigned int)HOST_TOW(towupper_l)((wint_t)wc, loc) : wc;
}

/* CT_CTYPE1.  Only C1_UPPER and C1_LOWER are load-bearing — they are what
 * create_mbcinfo switches on to fill `mbctype`'s _SBUP/_SBLOW bits and `mbcasemap` —
 * but the rest are cheap and keep the model honest for anything that reads them later. */
unsigned short nls_ctype1(unsigned int wc)
{
    locale_t loc = wide_locale();
    unsigned short r = 0;
    wint_t c = (wint_t)wc;

    if (!loc) return 0;
    if (HOST_ISW(iswupper_l)(c, loc))  r |= 0x0001;  /* C1_UPPER  */
    if (HOST_ISW(iswlower_l)(c, loc))  r |= 0x0002;  /* C1_LOWER  */
    if (HOST_ISW(iswdigit_l)(c, loc))  r |= 0x0004;  /* C1_DIGIT  */
    if (HOST_ISW(iswspace_l)(c, loc))  r |= 0x0008;  /* C1_SPACE  */
    if (HOST_ISW(iswpunct_l)(c, loc))  r |= 0x0010;  /* C1_PUNCT  */
    if (HOST_ISW(iswcntrl_l)(c, loc))  r |= 0x0020;  /* C1_CNTRL  */
    if (HOST_ISW(iswblank_l)(c, loc))  r |= 0x0040;  /* C1_BLANK  */
    if (HOST_ISW(iswxdigit_l)(c, loc)) r |= 0x0080;  /* C1_XDIGIT */
    if (HOST_ISW(iswalpha_l)(c, loc))  r |= 0x0100;  /* C1_ALPHA  */
    if (r) r |= 0x0200;                    /* C1_DEFINED */
    return r;
}

/* CT_CTYPE3.  mbcs.c reads exactly one bit of this — C3_SYMBOL, in `_ismbcsymbol_l`. */
unsigned short nls_ctype3(unsigned int wc)
{
    locale_t loc = wide_locale();
    if (!loc) return 0;
    /* Win32 C3_SYMBOL is the math/currency/modifier/other-symbol classes; glibc spells
     * that set `iswpunct && !iswalnum` minus the ASCII punctuation Windows counts as
     * punctuation rather than symbol.  Close enough is the right target here: both
     * sides read the same answer, so this grades _ismbcsymbol_l's LOGIC, not Windows'
     * symbol table (the same bounded scope refimpl.c documents for its NLS stubs). */
    return (unsigned short)(HOST_ISW(iswpunct_l)((wint_t)wc, loc)
                                    && !HOST_ISW(iswalnum_l)((wint_t)wc, loc)
                                ? 0x0008
                                : 0);
}

/* The reverse conversion, for create_mbcinfo's second pass (wide case-mapped forms back
 * to single bytes, which is how `mbcasemap` is filled). */
int nls_wide_to_mb(unsigned int cp, const unsigned short *src, int srclen,
                   char *dst, int dstlen, const char *defchar, int *used)
{
    iconv_t cd = cp_to_mb(cp);
    int produced = 0, consumed = 0;

    if (used) *used = 0;
    if (cd == (iconv_t)-1)
    {
        int i;
        for (i = 0; i < srclen && (!dstlen || i < dstlen); i++)
        {
            if (src[i] < 128) dst[i] = (char)src[i];
            else { dst[i] = defchar ? *defchar : '?'; if (used) *used = 1; }
        }
        return i;
    }

    iconv(cd, NULL, NULL, NULL, NULL);
    while (consumed < srclen)
    {
        char *in = (char *)(src + consumed);
        size_t inleft = (size_t)(srclen - consumed) * sizeof(unsigned short);
        char scratch[8];
        char *out = scratch;
        size_t outleft = sizeof(scratch);
        size_t before = inleft;
        size_t r;

        errno = 0;
        r = iconv(cd, &in, &inleft, &out, &outleft);
        {
            int got = (int)(sizeof(scratch) - outleft), j;
            for (j = 0; j < got; j++)
            {
                if (dstlen && produced >= dstlen) return produced;
                if (dstlen) dst[produced] = scratch[j];
                produced++;
            }
            consumed += (int)((before - inleft) / sizeof(unsigned short));
        }
        if (r == (size_t)-1 && (errno == EILSEQ || errno == EINVAL))
        {
            if (dstlen && produced >= dstlen) return produced;
            if (dstlen) dst[produced] = defchar ? *defchar : '?';
            if (used) *used = 1;
            produced++;
            consumed++;
            iconv(cd, NULL, NULL, NULL, NULL);
        }
    }
    return produced;
}
