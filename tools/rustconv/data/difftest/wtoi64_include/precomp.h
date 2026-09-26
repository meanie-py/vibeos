#include <stdint.h>
#include <wctype.h>

#define CDECL
#define __int64 long long
#define _locale_t void *
#define LONGLONG int64_t
#define BOOL int
#define TRUE 1
#define FALSE 0
#define _LIBCNT_ 1
#define wchar_t uint16_t
#define _SPACE 1
#define _I64_MAX INT64_MAX
#define _I64_MIN INT64_MIN
#define _UI64_MAX UINT64_MAX
#define MSVCRT_CHECK_PMT(condition) (condition)
#define towlower(ch) ((ch) >= 'A' && (ch) <= 'Z' ? (ch) + ('a' - 'A') : (ch))
#define iswctype(ch, kind) \
    ((kind) == _SPACE && ((ch) == 0x20 || ((ch) >= 0x09 && (ch) <= 0x0d)))
