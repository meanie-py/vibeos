/*
	Compatibility <intrin.h> header for GCC/Clang -- GCC equivalents of intrinsic
	Microsoft Visual C++ functions. ARM64/AArch64 version.

	Originally developed for the ReactOS (<https://reactos.org/>) project.

	This file is placed in the public domain.
*/

#ifndef KJK_INTRIN_ARM64_H_
#define KJK_INTRIN_ARM64_H_

#if !defined(__GNUC__) && !defined(__clang__)
#error Unsupported compiler
#endif

/* Most ARM64 intrinsics are already provided by Clang as builtins.
   However, we need to declare them so the compiler knows their signatures
   when called through macros. */

/* Bit scanning - Clang has these as builtins, just declare them */
unsigned char _BitScanForward(unsigned long *_Index, unsigned long _Mask);
unsigned char _BitScanReverse(unsigned long *_Index, unsigned long _Mask);
unsigned char _BitScanForward64(unsigned long *_Index, unsigned long long _Mask);
unsigned char _BitScanReverse64(unsigned long *_Index, unsigned long long _Mask);

/* Bit testing */
unsigned char _bittest(const long *_BitBase, long _BitPos);
unsigned char _bittestandset(long *_BitBase, long _BitPos);
unsigned char _bittestandreset(long *_BitBase, long _BitPos);
unsigned char _bittestandcomplement(long *_BitBase, long _BitPos);
unsigned char _bittest64(const long long *_BitBase, long long _BitPos);
unsigned char _bittestandset64(long long *_BitBase, long long _BitPos);
unsigned char _bittestandreset64(long long *_BitBase, long long _BitPos);

/* Interlocked operations */
long _InterlockedExchange(volatile long *_Target, long _Value);
long _InterlockedExchangeAdd(volatile long *_Addend, long _Value);
long _InterlockedCompareExchange(volatile long *_Destination, long _ExChange, long _Comperand);
long _InterlockedIncrement(volatile long *_Addend);
long _InterlockedDecrement(volatile long *_Addend);
long _InterlockedAnd(volatile long *_Destination, long _Value);
long _InterlockedOr(volatile long *_Destination, long _Value);
long _InterlockedXor(volatile long *_Destination, long _Value);
long long _InterlockedAnd64(volatile long long *_Destination, long long _Value);
long long _InterlockedOr64(volatile long long *_Destination, long long _Value);
long long _InterlockedXor64(volatile long long *_Destination, long long _Value);

long long _InterlockedExchange64(volatile long long *_Target, long long _Value);
long long _InterlockedExchangeAdd64(volatile long long *_Addend, long long _Value);
long long _InterlockedCompareExchange64(volatile long long *_Destination, long long _ExChange, long long _Comperand);
long long _InterlockedIncrement64(volatile long long *_Addend);
long long _InterlockedDecrement64(volatile long long *_Addend);

void *_InterlockedExchangePointer(void *volatile *_Target, void *_Value);
void *_InterlockedCompareExchangePointer(void *volatile *_Destination, void *_Exchange, void *_Comparand);

/* Count leading zeros */
unsigned int _CountLeadingZeros(unsigned long _Mask);
unsigned int _CountLeadingZeros64(unsigned long long _Mask);

/* Return address */
#ifndef _ReturnAddress
#define _ReturnAddress() (__builtin_return_address(0))
#endif

/* Debug break - Clang provides this as a builtin */
#ifndef __debugbreak
#define __debugbreak() __break(0xf000)
#endif

/* Memory barriers */
#ifndef _ReadWriteBarrier
#define _ReadWriteBarrier() __sync_synchronize()
#endif

#ifndef _mm_mfence
#define _mm_mfence() __sync_synchronize()
#endif

#ifndef _mm_lfence
#define _mm_lfence() __sync_synchronize()
#endif

#ifndef _mm_sfence
#define _mm_sfence() __sync_synchronize()
#endif

/* Byte swap - these may or may not be builtins depending on clang version */
#ifndef _byteswap_ushort
__INTRIN_INLINE unsigned short _byteswap_ushort(unsigned short value)
{
    return __builtin_bswap16(value);
}
#endif

#ifndef _byteswap_ulong
__INTRIN_INLINE unsigned long _byteswap_ulong(unsigned long value)
{
    return __builtin_bswap32(value);
}
#endif

#ifndef _byteswap_uint64
__INTRIN_INLINE unsigned long long _byteswap_uint64(unsigned long long value)
{
    return __builtin_bswap64(value);
}
#endif

/* Interrupt enable/disable */
__INTRIN_INLINE void __cdecl _disable(void)
{
    __asm__ __volatile__("msr daifset, #2" : : : "memory");
}

__INTRIN_INLINE void __cdecl _enable(void)
{
    __asm__ __volatile__("msr daifclr, #2" : : : "memory");
}

/* Special register access */
#ifndef __readx18
__INTRIN_INLINE unsigned long long __readx18(void)
{
    unsigned long long value;
    __asm__ __volatile__("mrs %0, tpidr_el0" : "=r" (value));
    return value;
}
#endif

/* 128-bit compare exchange for ARM64
 * Clang provides this as a builtin, we just need to declare it. */
unsigned char _InterlockedCompareExchange128(
    volatile __int64 *_Destination,
    __int64 _ExchangeHigh,
    __int64 _ExchangeLow,
    __int64 *_ComparandResult);

#endif /* KJK_INTRIN_ARM64_H_ */
