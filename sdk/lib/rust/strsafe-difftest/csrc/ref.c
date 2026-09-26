/* The genuine strsafe.h, host-compiled. Its printf entry points are variadic `static inline`s,
 * so each is wrapped at the fixed arity the harness drives: (int, wide string). */
#include <stddef.h>
#include <stdarg.h>
#include <string.h>

#define _SIZE_T_DEFINED
#define _SSIZE_T_DEFINED
#define _WCHAR_T_DEFINED
#define _HRESULT_DEFINED
typedef int HRESULT;              /* 32-bit, as on every Windows target (long is 64 here) */
typedef unsigned int ULONG;       /* likewise */
#define WEOF ((unsigned short)0xFFFF) /* only the unused StringGets* workers read it */
#define WINAPI
#define __CRT_INLINE static inline
#define __MINGW_EXTENSION
int _vsnwprintf(wchar_t *buf, size_t count, const wchar_t *fmt, va_list ap);
#include STRSAFE_H_PATH

HRESULT ref_StringCchPrintfW(wchar_t *d, size_t cch, const wchar_t *f, int a, const wchar_t *s)
{ return StringCchPrintfW(d, cch, f, a, s); }

HRESULT ref_StringCbPrintfW(wchar_t *d, size_t cb, const wchar_t *f, int a, const wchar_t *s)
{ return StringCbPrintfW(d, cb, f, a, s); }

HRESULT ref_StringCbPrintfExW(wchar_t *d, size_t cb, wchar_t **end, size_t *rem,
                              unsigned flags, const wchar_t *f, int a, const wchar_t *s)
{ return StringCbPrintfExW(d, cb, end, rem, flags, f, a, s); }
