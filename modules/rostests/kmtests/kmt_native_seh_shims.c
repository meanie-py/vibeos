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

void *RtlSecureZeroMemory(void *Pointer, KMT_SIZE_T Size)
{
    volatile char *vptr = (volatile char *)Pointer;
    while (Size--)
        *vptr++ = 0;
    return Pointer;
}

void RtlInitEmptyUnicodeString(KMT_UNICODE_STRING *UnicodeString,
                               unsigned short *Buffer,
                               KMT_USHORT BufferSize)
{
    UnicodeString->Length = 0;
    UnicodeString->MaximumLength = BufferSize;
    UnicodeString->Buffer = Buffer;
}

#if defined(__x86_64__)
/* clang builtin on arm64; FORCEINLINE wrapper on amd64 (xdk/interlocked.h) */
KMT_LONG64 _InterlockedAdd64(volatile KMT_LONG64 *Target, KMT_LONG64 Value)
{
    return __atomic_add_fetch(Target, Value, __ATOMIC_SEQ_CST);
}

/* amd64-only FORCEINLINE in xdk/rtlfuncs.h; a real ntoskrnl export elsewhere */
void RtlFillMemoryUlong(void *Destination, KMT_SIZE_T Length, KMT_ULONG Pattern)
{
    KMT_ULONG *Address = (KMT_ULONG *)Destination;
    Length /= sizeof(KMT_ULONG);
    while (Length--)
        *Address++ = Pattern;
}
#endif /* __x86_64__ */

#endif /* __x86_64__ || __aarch64__ */
