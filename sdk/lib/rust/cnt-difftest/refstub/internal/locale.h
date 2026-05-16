/* Minimal <internal/locale.h>: the genuine MSVCRT_threadlocinfo
 * layout (so winesup.c's `threadlocinfo _LIBCNT_locinfo = {...}`
 * initializer is valid and `locinfo->lconv->decimal_point` lands).
 * struct lconv comes from the host <locale.h> (its first member is
 * `char *decimal_point` — the only field the engine reads). The
 * zero-initialized members are never dereferenced by the scanf
 * engine, so host-vs-target width of e.g. wchar_t is irrelevant. */
#ifndef CNT_REFSTUB_INTERNAL_LOCALE_H
#define CNT_REFSTUB_INTERNAL_LOCALE_H

#include <locale.h> /* host struct lconv (decimal_point first) */

#ifndef CNT_NT_INT_TYPES
#define CNT_NT_INT_TYPES
typedef unsigned int ULONG;  /* Windows LLP64: 32-bit */
typedef unsigned int *PULONG;
typedef int LONG;
typedef long long LONGLONG;
#ifndef ULONGLONG
typedef unsigned long long ULONGLONG;
#endif
#endif

typedef struct _CNT_LC_ID {
    unsigned short wLanguage;
    unsigned short wCountry;
    unsigned short wCodePage;
} LC_ID;

struct __lc_time_data;

typedef struct MSVCRT_threadlocaleinfostruct {
    LONG refcount;
    unsigned int lc_codepage;
    unsigned int lc_collate_cp;
    unsigned long lc_handle[6];
    LC_ID lc_id[6];
    struct {
        char *locale;
        wchar_t *wlocale;
        int *refcount;
        int *wrefcount;
    } lc_category[6];
    int lc_clike;
    int mb_cur_max;
    int *lconv_intl_refcount;
    int *lconv_num_refcount;
    int *lconv_mon_refcount;
    struct lconv *lconv;
    int *ctype1_refcount;
    unsigned short *ctype1;
    const unsigned short *pctype;
    unsigned char *pclmap;
    unsigned char *pcumap;
    struct __lc_time_data *lc_time_curr;
} MSVCRT_threadlocinfo;

typedef MSVCRT_threadlocinfo threadlocinfo;
typedef MSVCRT_threadlocinfo *pthreadlocinfo;

/* Completes the opaque `_locale_t` from refstub/precomp.h so scanf.h's
 * dead `locale->locinfo` branch (locale is always NULL here) compiles. */
struct _cnt_locale_s {
    pthreadlocinfo locinfo;
    void *mbcinfo;
};

#endif
