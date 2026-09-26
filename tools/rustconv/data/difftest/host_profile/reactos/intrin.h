/* Host stand-in for vcruntime's <intrin.h>: the tree's own GCC implementations of the
   MSVC intrinsics (vcruntime/mingw32, found through the profile's -idirafter), which
   winnt.h's inline functions call. The two macros are what _mingw.h would provide. */
#pragma once
#ifndef __INTRIN_INLINE
#define __INTRIN_INLINE extern __inline__ __attribute__((__always_inline__, __gnu_inline__, artificial))
#endif
#ifndef HAS_BUILTIN
#define HAS_BUILTIN(x) __has_builtin(x)
#endif
#include <mingw32/intrin.h>
