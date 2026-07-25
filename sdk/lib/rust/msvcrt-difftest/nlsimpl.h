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

#endif /* MSVCRT_DIFFTEST_NLSIMPL_H */
