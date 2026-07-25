/*
 * Minimal <winnls.h> for the msvcrt string.c reference build.
 *
 * string.c uses exactly four things from the NLS surface — and ONLY on
 * the non-"C"-locale branches (`if(!locinfo->lc_handle[LC_COLLATE])`
 * takes the plain-ASCII path first).  Our get_locinfo() models the
 * default C locale, so these are dead code at run time; they still have
 * to compile and link.  Implementations: refimpl.c.
 */
#ifndef __WINE_WINNLS_H
#define __WINE_WINNLS_H

#define CSTR_LESS_THAN      1
#define CSTR_EQUAL          2
#define CSTR_GREATER_THAN   3

#define NORM_IGNORECASE     0x00000001
#define NORM_IGNORENONSPACE 0x00000002
#define NORM_IGNORESYMBOLS  0x00000004
#define SORT_STRINGSORT     0x00001000

#define LCMAP_LOWERCASE     0x00000100
#define LCMAP_UPPERCASE     0x00000200
#define LCMAP_SORTKEY       0x00000400

#define CP_ACP              0
#define CP_UTF8             65001
#define LOCALE_NAME_MAX_LENGTH 85

#define CT_CTYPE1           0x0001
#define CT_CTYPE3           0x0004
#define WC_NO_BEST_FIT_CHARS 0x0400
#define ERROR_INSUFFICIENT_BUFFER 122

/* Character-type bits.  mbcs.c reads exactly three: create_mbcinfo switches on
 * C1_UPPER/C1_LOWER to fill mbctype's _SBUP/_SBLOW and mbcasemap, and
 * _ismbcsymbol_l tests C3_SYMBOL.  Values as in sdk/include/psdk/winnls.h. */
#define C1_UPPER            0x0001
#define C1_LOWER            0x0002
#define C1_DIGIT            0x0004
#define C1_SPACE            0x0008
#define C1_PUNCT            0x0010
#define C1_CNTRL            0x0020
#define C1_BLANK            0x0040
#define C1_XDIGIT           0x0080
#define C1_ALPHA            0x0100
#define C1_DEFINED          0x0200
#define C3_SYMBOL           0x0008

#define LOCALE_ALLOW_NEUTRAL_NAMES 0x08000000

/* create_mbcinfo walks LeadByte[] as {start,end} pairs terminated by {0,0}. */
#define MAX_LEADBYTES       12
#define MAX_DEFAULTCHAR      2
#define MSVCRT_DIFFTEST_HAVE_CPINFO
typedef struct _cpinfo {
    UINT MaxCharSize;
    BYTE DefaultChar[MAX_DEFAULTCHAR];
    BYTE LeadByte[MAX_LEADBYTES];
} CPINFO;

UINT WINAPI GetACP(void);
UINT WINAPI GetOEMCP(void);
BOOL WINAPI GetCPInfo(UINT cp, CPINFO *info);
int  WINAPI MultiByteToWideChar(UINT cp, DWORD flags, const char *src, int srclen,
                                wchar_t *dst, int dstlen);
LCID WINAPI LocaleNameToLCID(const wchar_t *name, DWORD flags);

int WINAPI CompareStringA(LCID lcid, DWORD flags, const char *str1, int len1,
                          const char *str2, int len2);
int WINAPI LCMapStringA(LCID lcid, DWORD flags, const char *src, int srclen,
                        char *dst, int dstlen);

/* The WIDE half, for the wcs.c reference TU.  Same story as the A versions:
 * dead under the C locale, but they must compile and link. */
int WINAPI CompareStringW(LCID lcid, DWORD flags, const wchar_t *str1, int len1,
                          const wchar_t *str2, int len2);
int WINAPI LCMapStringW(LCID lcid, DWORD flags, const wchar_t *src, int srclen,
                        wchar_t *dst, int dstlen);
BOOL WINAPI GetStringTypeW(DWORD type, const wchar_t *src, int count, WORD *chartype);
int WINAPI WideCharToMultiByte(UINT page, DWORD flags, const wchar_t *src, int srclen,
                               char *dst, int dstlen, const char *defchar, BOOL *used);

#endif /* __WINE_WINNLS_H */
