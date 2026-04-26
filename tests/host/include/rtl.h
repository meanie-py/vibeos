/*
 * rtl.h — Host shim replacing sdk/lib/rtl/rtl.h
 *
 * When compiling RTL source files for the host, this header is picked up
 * instead of the real rtl.h because tests/host/include/ is first in the
 * include path.
 */
#ifndef RTL_H
#define RTL_H
#include "nt_host.h"

/* RTL internal macros from rtlp.h */
#define ROUND_DOWN(n, align) \
    (((ULONG_PTR)(n)) & ~((align) - 1l))

#define ROUND_UP(n, align) \
    ROUND_DOWN(((ULONG_PTR)(n)) + (align) - 1, (align))

/* RTL internal allocation — maps to malloc/free on host */
static inline PVOID NTAPI RtlpAllocateMemory(SIZE_T Bytes, ULONG Tag) {
    (void)Tag;
    return malloc(Bytes);
}

static inline VOID NTAPI RtlpFreeMemory(PVOID Mem, ULONG Tag) {
    (void)Tag;
    free(Mem);
}

#endif
