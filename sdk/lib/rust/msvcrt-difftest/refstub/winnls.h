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
#define LOCALE_NAME_MAX_LENGTH 85

int WINAPI CompareStringA(LCID lcid, DWORD flags, const char *str1, int len1,
                          const char *str2, int len2);
int WINAPI LCMapStringA(LCID lcid, DWORD flags, const char *src, int srclen,
                        char *dst, int dstlen);

#endif /* __WINE_WINNLS_H */
