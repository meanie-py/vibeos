/* Minimal <wine/winternl.h> for the libcntpr scanf reference TU.
 * Provides the NT types + the RTL_BITMAP / heap surface scanf.h's
 * `%[` scanset uses. The bitmap ops are byte-faithful to
 * sdk/lib/rtl/bitmap.c (set/test `count` bits from `start`, in-range
 * checks) so the scanset range quirks are pinned exactly. The heap is
 * a deterministic calloc/free model — allocation identity is not part
 * of the value contract; only the bitmap CONTENTS are. */
#ifndef CNT_REFSTUB_WINE_WINTERNL_H
#define CNT_REFSTUB_WINE_WINTERNL_H

#include <stdlib.h> /* calloc/free */

#ifndef CNT_NT_INT_TYPES
#define CNT_NT_INT_TYPES
typedef unsigned int   ULONG;  /* Windows LLP64: ULONG/LONG are 32-bit */
typedef unsigned int  *PULONG;
typedef int            LONG;
typedef long long      LONGLONG;
#ifndef ULONGLONG
typedef unsigned long long ULONGLONG;
#endif
typedef void           VOID;
typedef void          *PVOID;
typedef int            BOOL;
#endif

#ifndef TRUE
#define TRUE 1
#endif
#ifndef FALSE
#define FALSE 0
#endif
#define HEAP_ZERO_MEMORY 0x00000008

/* ---- deterministic process-heap model ---------------------------- */
typedef struct _CNT_PEB { void *ProcessHeap; } CNT_PEB;
typedef struct _CNT_TEB { CNT_PEB *Peb; } CNT_TEB;
static CNT_PEB _cnt_peb = { (void *)1 };
static CNT_TEB _cnt_teb = { &_cnt_peb };
static __inline CNT_TEB *NtCurrentTeb(void) { return &_cnt_teb; }

static __inline void *RtlAllocateHeap(void *h, unsigned f, unsigned long n)
{
    (void)h;
    (void)f;
    return calloc(1, n ? n : 1); /* scanf always passes HEAP_ZERO_MEMORY */
}
static __inline int RtlFreeHeap(void *h, unsigned f, void *p)
{
    (void)h;
    (void)f;
    free(p);
    return 1;
}

/* ---- RTL_BITMAP (faithful to sdk/lib/rtl/bitmap.c) ---------------- */
typedef struct _RTL_BITMAP {
    ULONG SizeOfBitMap;
    PULONG Buffer;
} RTL_BITMAP, *PRTL_BITMAP;

static __inline void RtlInitializeBitMap(PRTL_BITMAP bm, PULONG buf, ULONG sz)
{
    bm->SizeOfBitMap = sz;
    bm->Buffer = buf;
}
static __inline void RtlSetBits(PRTL_BITMAP bm, ULONG start, ULONG count)
{
    ULONG i;
    if (count == 0 || start >= bm->SizeOfBitMap) return;
    if (start + count > bm->SizeOfBitMap) count = bm->SizeOfBitMap - start;
    for (i = 0; i < count; i++)
        bm->Buffer[(start + i) >> 5] |= (1UL << ((start + i) & 31));
}
static __inline int RtlAreBitsSet(PRTL_BITMAP bm, ULONG start, ULONG len)
{
    ULONG i;
    if (len == 0 || start >= bm->SizeOfBitMap ||
        start + len > bm->SizeOfBitMap)
        return FALSE;
    for (i = 0; i < len; i++)
        if (!(bm->Buffer[(start + i) >> 5] & (1UL << ((start + i) & 31))))
            return FALSE;
    return TRUE;
}
static __inline int RtlAreBitsClear(PRTL_BITMAP bm, ULONG start, ULONG len)
{
    ULONG i;
    if (len == 0 || start >= bm->SizeOfBitMap ||
        start + len > bm->SizeOfBitMap)
        return FALSE;
    for (i = 0; i < len; i++)
        if (bm->Buffer[(start + i) >> 5] & (1UL << ((start + i) & 31)))
            return FALSE;
    return TRUE;
}

#endif
