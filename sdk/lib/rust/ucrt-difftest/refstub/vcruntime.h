/* Shadow for sdk/include/vcruntime/vcruntime.h, reusing its include guard
   (_VCRUNTIME_H) so the real header expands to nothing.  Same trick
   msvcrt-difftest plays on msvcrt.h via __WINE_MSVCRT_H. */
#ifndef _VCRUNTIME_H
#define _VCRUNTIME_H

#include <stddef.h>
#include <stdint.h>

/* The tree's own SAL neutraliser -- exactly what it is for. Without it every
   _In_opt_z_ / _Field_range_ in the UCRT headers is an undeclared identifier. */
#include <no_sal2.h>
#include <sal_extra.h>

/* Calling conventions: SysV host has one. Defined HERE rather than in the
   prelude because the mingw headers redefine __cdecl to an attribute g++
   rejects, and a prelude cannot win that race. */
#define __cdecl
#define __fastcall
#define __stdcall
#define __thiscall
#define __clrcall
#define _VCRTIMP
#define _ACRTIMP
#define _ACRTIMP_ALT
#define _CRTIMP
#define __CRTDECL
#define _UNALIGNED
/* Defined HERE, not in the prelude: the psdk/mingw headers redefine it and
   the prelude loses that race. */
#define __declspec(x)
#define __attribute__(x)
#define _CRT_BEGIN_C_HEADER extern "C" {
#define _CRT_END_C_HEADER }
#define _CRT_INSECURE_DEPRECATE(x)
#define _CRT_INSECURE_DEPRECATE_MEMORY(x)
#define _CRT_OBSOLETE(x)
#define _CRT_STRINGIZE_(v) #v
#define _CRT_STRINGIZE(v) _CRT_STRINGIZE_(v)
#define _CRT_WIDE_(s) L ## s
#define _CRT_WIDE(s) _CRT_WIDE_(s)
#define _CRT_CONCATENATE_(a, b) a ## b
#define _CRT_CONCATENATE(a, b) _CRT_CONCATENATE_(a, b)
#define __crt_countof(a) (sizeof(a) / sizeof((a)[0]))
#define _CRT_PACKING 8
#define _CRT_HAS_CXX17 0
#define _NODISCARD
#define _HAS_NODISCARD 0
#define _CRT_DEPRECATE_TEXT(x)

/* MSVC fixed-width spellings the UCRT headers use in declarations. */
#define __int8  char
#define __int16 short
#define __int32 int
#define __int64 long long
#define __forceinline inline

typedef __builtin_va_list va_list;
#ifndef _VA_LIST_DEFINED
#define _VA_LIST_DEFINED
#endif

typedef int errno_t;
typedef unsigned short wint_t;
typedef unsigned short wctype_t;
typedef long __time32_t;
typedef long long __time64_t;
typedef __SIZE_TYPE__ rsize_t;
typedef __INTPTR_TYPE__ ssize_t;

extern uintptr_t __security_cookie;
void __security_init_cookie(void);

#endif /* _VCRUNTIME_H */
