/* locale_layout.h — msvcrt's locale/thread state LAYOUT, shared by both sides.
 *
 * WHY THIS IS A SEPARATE HEADER FROM refstub/msvcrt.h
 *
 * refstub/msvcrt.h carries the anti-circularity NAMESPACE block (`#define get_locinfo
 * msvcrt_ref_get_locinfo`, …), which is what stops the reference from ever binding to
 * the code under test.  Only the reference side may see it.
 *
 * But both sides need the *layout* of `threadlocinfo` and `thread_data_t`:
 *
 *   * the reference (refimpl.c + ctype.c) allocates and fills one, and
 *   * the SUBJECT (subjectimpl.c) allocates and fills its own — separate storage, so
 *     classify can compare each side's errno / strtok cursor / pmt count independently.
 *
 * and, above all, the SHIPPING RUST reads these structs through `msvcrt-sys`, a bindgen
 * snapshot of msvcrt's REAL layout (`pcumap` @ 336 on amd64, @ 208 on i386).  The
 * harness links that same Rust and points it at the structs defined here.  If this
 * layout is a convenient cartoon and the Rust's is the real thing, the Rust reads
 * `pcumap` from past the end of the object — inside the very component whose job is to
 * be a trustworthy oracle.  A wrong offset there does not crash.  It makes the oracle
 * lie, quietly, forever.
 *
 * So this is a faithful transcription of msvcrt's structs (dll/win32/msvcrt/include/
 * corecrt.h + msvcrt.h, `_MSVCR_VER=0` branches resolved), and `ref_integrity` asserts
 * at run time that C's `offsetof` here equals Rust's `offset_of!` in msvcrt-sys, field
 * by field.  Do not "simplify" it.
 */
#ifndef MSVCRT_DIFFTEST_LOCALE_LAYOUT_H
#define MSVCRT_DIFFTEST_LOCALE_LAYOUT_H

/* msvcrt's wchar_t is 16-bit.  Only ever used through a POINTER below, so it cannot
 * affect the layout — but spell it explicitly so the two sides cannot disagree even
 * in principle (the host's is 32-bit). */
typedef unsigned short rl_wchar;

struct lconv;
struct __lc_time_data;

typedef struct tagLC_ID {
    unsigned short wLanguage;
    unsigned short wCountry;
    unsigned short wCodePage;
} RL_LC_ID;

/* threadlocaleinfostruct, verbatim.  `__msvcrt_ulong` is 32-bit on BOTH the LP64 host
 * (`unsigned int`) and LLP64 Windows (`unsigned long`) — corecrt.h goes out of its way
 * to guarantee that — so `unsigned int` here is exact on every target we build. */
typedef struct threadlocaleinfostruct {
    int             refcount;
    unsigned int    lc_codepage;
    unsigned int    lc_collate_cp;
    unsigned int    lc_handle[6];       /* __msvcrt_ulong */
    RL_LC_ID        lc_id[6];
    struct {
        char       *locale;
        rl_wchar   *wlocale;
        int        *refcount;
        int        *wrefcount;
    } lc_category[6];
    int             lc_clike;
    int             mb_cur_max;
    int            *lconv_intl_refcount;
    int            *lconv_num_refcount;
    int            *lconv_mon_refcount;
    struct lconv   *lconv;
    int            *ctype1_refcount;
    unsigned short *ctype1;
    unsigned short *pctype;             /* == MSVCRT__ctype + 1 -- NOTE THE +1 */
    const unsigned char *pclmap;        /* tolower table */
    const unsigned char *pcumap;        /* toupper table */
    struct __lc_time_data *lc_time_curr;
} rl_threadlocinfo;

/* threadmbcinfostruct, verbatim (dll/win32/msvcrt/msvcrt.h).
 *
 * This lives here, beside threadlocinfo, for exactly the reason at the top of this
 * file — and it is the sharper case of it.  refstub/msvcrt.h used to carry a
 * three-field version (refcount, mbcodepage, ismbcodepage), which was all string.c
 * and wcs.c ever touch.  That was safe only because refimpl.c sets
 * `locale->mbcinfo = NULL` and nothing dereferenced it.
 *
 * mbcs.c dereferences it on every call, and so does the shipping Rust — through
 * msvcrt-sys's bindgen of the REAL struct, where `mbctype` starts at offset 28 and
 * runs 257 bytes.  Against a 12-byte object that is a read 273 bytes past the end,
 * inside the component whose entire job is to be a trustworthy oracle.  It would not
 * crash; it would return whatever the allocator left there, differently on each run.
 *
 * `LONG` is 32-bit on Windows; `int` is exact on every target we build.
 * ref_integrity asserts this layout against msvcrt-sys field by field. */
typedef struct threadmbcinfostruct {
    int             refcount;           /* LONG */
    int             mbcodepage;
    int             ismbcodepage;
    int             mblcid;
    unsigned short  mbulinfo[6];
    unsigned char   mbctype[257];       /* NOTE 257, not 256: index is byte+1 */
    unsigned char   mbcasemap[256];
} rl_threadmbcinfo;

/* `_locale_t`'s target — msvcrt-sys calls it `localeinfo_struct`.  Shared for the same
 * reason as the two structs above: an explicit `_locale_t` argument is how all 80 of
 * mbcs.c's `_l` entry points reach their MBCS state, so both sides must agree where
 * `mbcinfo` sits.  The struct TAG stays `_locale_tstruct` because the reference TUs use
 * that name as a type directly (wcs.c's `_wcsicmp_l` family declares
 * `_locale_tstruct tmp = {0};` on the stack). */
typedef struct _locale_tstruct {
    struct threadlocaleinfostruct *locinfo;
    struct threadmbcinfostruct    *mbcinfo;
} rl_locale_t;

/* MSVC's locale categories.  The host <locale.h> numbers these completely differently
 * (glibc: LC_CTYPE=0 … LC_ALL=6 … LC_IDENTIFICATION=12), and with lc_handle[] at its
 * real size of 6 a glibc LC_ALL(=6) would index one PAST the end.  These are what
 * msvcrt's own code means. */
#define RL_LC_ALL      0
#define RL_LC_COLLATE  1
#define RL_LC_CTYPE    2
#define RL_LC_MONETARY 3
#define RL_LC_NUMERIC  4
#define RL_LC_TIME     5

/* dll/win32/msvcrt/include/ctype.h — the _pctype/_pwctype classification bits. */
#define RL_UPPER    0x0001
#define RL_LOWER    0x0002
#define RL_DIGIT    0x0004
#define RL_SPACE    0x0008
#define RL_PUNCT    0x0010
#define RL_CONTROL  0x0020
#define RL_BLANK    0x0040
#define RL_HEX      0x0080
#define RL_LEADBYTE 0x8000
#define RL_ALPHA   (0x0100 | RL_UPPER | RL_LOWER)

#endif /* MSVCRT_DIFFTEST_LOCALE_LAYOUT_H */
