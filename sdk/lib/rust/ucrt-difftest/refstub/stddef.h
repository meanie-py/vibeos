/* Shadows ucrt/stddef.h, which is on the include path and would
   otherwise win over the host's. Defines from compiler builtins rather than
   chaining, because include_next would find the UCRT one, not the system one. */
#pragma once
typedef __SIZE_TYPE__    size_t;
typedef __PTRDIFF_TYPE__ ptrdiff_t;
#ifndef NULL
#define NULL 0
#endif
#define offsetof(t, m) __builtin_offsetof(t, m)
typedef long double max_align_t;
