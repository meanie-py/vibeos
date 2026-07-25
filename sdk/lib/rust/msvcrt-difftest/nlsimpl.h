/* nlsimpl.h — the codepage-aware NLS environment shared by both sides.
 *
 * See nlsimpl.c for why this is codepage-aware where refimpl.c's NLS is not.
 * Included by nlsimpl.c, refimpl.c (which delegates its LCID-only entry points here)
 * and subjectimpl.c (which installs the same fixture on the subject's side).
 */
#ifndef MSVCRT_DIFFTEST_NLSIMPL_H
#define MSVCRT_DIFFTEST_NLSIMPL_H

/* This header is included by TUs that have already established the Windows type model
 * (refstub/msvcrt.h) and by ones that have not (nlsimpl.c itself).  Spell the few types
 * needed here only if nobody has. */
#ifndef __WINE_MSVCRT_H
#define CDECL
#define WINAPI
typedef int            BOOL;
typedef unsigned char  BYTE;
typedef unsigned short WORD;
typedef unsigned int   DWORD;
typedef unsigned int   LCID;
#define wchar_t unsigned short
#ifndef TRUE
#define TRUE  1
#define FALSE 0
#endif
#endif /* __WINE_MSVCRT_H */

/* winnls.h's CPINFO, in the shape create_mbcinfo walks. */
#ifndef MAX_LEADBYTES
#define MAX_LEADBYTES   12
#define MAX_DEFAULTCHAR  2
#endif

#ifndef MSVCRT_DIFFTEST_HAVE_CPINFO
#define MSVCRT_DIFFTEST_HAVE_CPINFO
typedef struct _cpinfo {
    unsigned int MaxCharSize;
    BYTE         DefaultChar[MAX_DEFAULTCHAR];
    BYTE         LeadByte[MAX_LEADBYTES];
} CPINFO;
#endif

/* kernel32, as create_mbcinfo calls it. */
unsigned int WINAPI GetACP(void);
unsigned int WINAPI GetOEMCP(void);
BOOL         WINAPI GetCPInfo(unsigned int cp, CPINFO *info);
int          WINAPI MultiByteToWideChar(unsigned int cp, DWORD flags, const char *src,
                                        int srclen, wchar_t *dst, int dstlen);
unsigned int WINAPI LocaleNameToLCID(const wchar_t *name, DWORD flags);
BOOL                locale_to_sname(const char *locale, unsigned short *codepage,
                                    BOOL *sname_match, wchar_t *sname);

/* Which codepage the LCID-only entry points (CompareStringA, LCMapStringW) should
 * assume.  See the note in nlsimpl.c: several codepages share an LCID, so the installed
 * fixture is the answer rather than a reverse map. */
unsigned int nls_active_codepage(void);
void         nls_set_active_codepage(unsigned int cp);
int          nls_is_dbcs_codepage(unsigned int cp);
unsigned int nls_lcid_for_codepage(unsigned int cp);

/* The codepages a fixture may pin, for the harness to enumerate. */
int          nls_codepage_count(void);
unsigned int nls_codepage_at(int i);

/* Wide classification / case mapping, from the host's Unicode tables under a per-call
 * C.utf8 locale.  refimpl.c's GetStringTypeW and LCMapStringW delegate here for
 * anything a DBCS codepage can produce — which is most of what create_mbcinfo asks
 * about, since CP932's 0xA1-0xDF land at U+FF61-U+FF9F. */
unsigned int   nls_towlower(unsigned int wc);
unsigned int   nls_towupper(unsigned int wc);
unsigned short nls_ctype1(unsigned int wc);   /* C1_UPPER | C1_LOWER | … */
unsigned short nls_ctype3(unsigned int wc);   /* C3_SYMBOL */

int nls_wide_to_mb(unsigned int cp, const unsigned short *src, int srclen,
                   char *dst, int dstlen, const char *defchar, int *used);

/* ── the pinned MBCS fixture ────────────────────────────────────────────────
 *
 * Built by the REFERENCE's own create_mbcinfo (`ref_create_mbcinfo`), not by a
 * hand-written filler.  That is the whole trick: the state every graded function is
 * pinned against is produced by the very C those functions are graded against, so it
 * cannot drift from them, and the only thing this file has to get right is the NLS
 * underneath it.
 *
 * `mbcs_fixture_install` copies it into the reference's slot and the subject's slot —
 * two separate copies, see the get_mbcinfo note in refstub/msvcrt.h.  `mbcs_fixture_
 * check` reports whether either copy has drifted from the pristine one, which is how a
 * graded function that MUTATES the shared state gets caught instead of quietly
 * poisoning every later case.
 */
#include "locale_layout.h"

int  mbcs_fixture_install(unsigned int cp);
int  mbcs_fixture_check(void);          /* 0 = both copies still pristine */
const rl_threadmbcinfo *mbcs_fixture_pristine(void);

/* The two slots.  Defined in refimpl.c and subjectimpl.c respectively, because each
 * must be reachable from its own side's `get_mbcinfo`. */
extern rl_threadmbcinfo *mbcs_ref_slot(void);
extern rl_threadmbcinfo *mbcs_subj_slot(void);

#endif /* MSVCRT_DIFFTEST_NLSIMPL_H */
