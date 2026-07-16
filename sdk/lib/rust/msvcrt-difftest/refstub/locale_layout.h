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
