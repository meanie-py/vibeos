/* Minimal <wine/unicode.h>: unicode.c uses only `strchrW` (a u16
 * wide strchr). Provided here over our WCHAR==unsigned short. */
#ifndef RTL_REFSTUB_WINE_UNICODE_H
#define RTL_REFSTUB_WINE_UNICODE_H
static __inline WCHAR *strchrW(const WCHAR *s, WCHAR c) {
    do {
        if (*s == c) return (WCHAR *)s;
    } while (*s++);
    return 0;
}
#endif
