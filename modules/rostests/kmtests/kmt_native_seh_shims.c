/*
 * PROJECT:     ReactOS kernel-mode tests (vibeos conformance lane)
 * PURPOSE:     Out-of-line homes for FORCEINLINE SDK helpers under native SEH
 *
 * clang never inlines callees that sit lexically inside a __try region
 * (funclet EH restriction, any -O level). Our mingw __forceinline for C is
 * `extern inline __attribute__((always_inline, gnu_inline))` — GNU inline
 * semantics, which never emit an out-of-line body — so tests compiled with
 * _USE_NATIVE_SEH get undefined references to these helpers at their
 * non-inlined call sites. The gnu_inline model expects exactly this file:
 * one TU providing the real external definitions. It must NOT include the
 * SDK headers (same-TU redefinition of a gnu_inline function is an error),
 * so the bodies below replicate the trivial SDK inlines with bare types.
 * Test sources are untouched. Only wired into the build on amd64/arm64,
 * where the kmtest targets opt into native SEH (see CMakeLists.txt).
 */

#if defined(__x86_64__) || defined(__aarch64__)

/* Weak: some test TUs (RtlMemory.c, RtlStrSafe.c, ExInterlocked.c) DO emit an
 * out-of-line body for these, and then this copy must lose regardless of link
 * order. As strong definitions they were duplicates resolved by order, which
 * tools/link_guard.py refuses. */
#define KMT_FALLBACK __attribute__((weak))

typedef unsigned short KMT_USHORT;
typedef unsigned int KMT_ULONG;
typedef unsigned long long KMT_SIZE_T;
typedef long long KMT_LONG64;

/* matches UNICODE_STRING layout (Length, MaximumLength, [pad,] Buffer) */
typedef struct
{
    KMT_USHORT Length;
    KMT_USHORT MaximumLength;
    unsigned short *Buffer;
} KMT_UNICODE_STRING;

KMT_FALLBACK void *RtlSecureZeroMemory(void *Pointer, KMT_SIZE_T Size)
{
    volatile char *vptr = (volatile char *)Pointer;
    while (Size--)
        *vptr++ = 0;
    return Pointer;
}

KMT_FALLBACK void RtlInitEmptyUnicodeString(KMT_UNICODE_STRING *UnicodeString,
                               unsigned short *Buffer,
                               KMT_USHORT BufferSize)
{
    UnicodeString->Length = 0;
    UnicodeString->MaximumLength = BufferSize;
    UnicodeString->Buffer = Buffer;
}

#if defined(__x86_64__)
/* clang builtin on arm64; FORCEINLINE wrapper on amd64 (xdk/interlocked.h) */
KMT_FALLBACK KMT_LONG64 _InterlockedAdd64(volatile KMT_LONG64 *Target, KMT_LONG64 Value)
{
    return __atomic_add_fetch(Target, Value, __ATOMIC_SEQ_CST);
}

/* amd64-only FORCEINLINE in xdk/rtlfuncs.h; a real ntoskrnl export elsewhere */
KMT_FALLBACK void RtlFillMemoryUlong(void *Destination, KMT_SIZE_T Length, KMT_ULONG Pattern)
{
    KMT_ULONG *Address = (KMT_ULONG *)Destination;
    Length /= sizeof(KMT_ULONG);
    while (Length--)
        *Address++ = Pattern;
}
#endif /* __x86_64__ */

#endif /* __x86_64__ || __aarch64__ */

#if defined(__i386__)

/*
 * i386 under KMT_I386_NATIVE_SEH, which retargets to -windows-msvc.
 *
 * This is a DIFFERENT root cause from the block above, so it gets its own guard
 * rather than widening that one: i386 resolves every other helper fine, and
 * defining the rest here would only add redundant duplicates.
 *
 * `_InterlockedAddLargeStatistic` is provided by mingw as an __INTRIN_INLINE in
 * sdk/include/vcruntime/mingw32/intrin_x86.h, but that header is selected by the
 * *gnu* environment. Retargeted to -windows-msvc, the SDK expects MSVC to supply
 * the intrinsic and emits a plain external call instead — leaving exactly one
 * undefined symbol at ExInterlocked.c:326 (_TestInterlockedFunctional).
 *
 * The body replicates the mingw asm verbatim: a lock-prefixed 32-bit add to the
 * low half with carry propagated into the high half. Deliberately NOT written as
 * a 64-bit __atomic_fetch_add — on i386 that can lower to a libatomic call
 * (__atomic_fetch_add_8), trading one undefined symbol for another in a kernel
 * driver that links no such runtime. Two locked halves is also what the real
 * intrinsic does; it is a statistic, not a synchronisation primitive.
 */
long _InterlockedAddLargeStatistic(volatile long long *Addend, long Value)
{
    __asm__ __volatile__
    (
        "lock; addl %[Value], %[Lo32];"
        "jae 1f;"
        "lock; adcl $0, %[Hi32];"
        "1:;"
        : [Lo32] "+m" (*((volatile long *)(Addend) + 0)),
          [Hi32] "+m" (*((volatile long *)(Addend) + 1))
        : [Value] "ir" (Value)
        : "memory"
    );

    return Value;
}

#endif /* __i386__ */
