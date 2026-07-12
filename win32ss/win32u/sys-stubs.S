
#include <asm.inc>
#include <syscalls.inc>

.code

#ifdef _M_IX86
/* Per-module WoW64 transition slot (see syscalls.inc DEFINE_WOW64_TRANSITION). */
DEFINE_WOW64_TRANSITION
#endif

SyscallId = HEX(1000)
#define SVC_(name, argcount) STUB_U name, argcount

/* The service-number list must match the KERNEL table order (napi.h picks
 * w32ksvc64.h under _WIN64, which includes arm64) — selecting by _M_AMD64
 * alone made arm64 win32u number its stubs from the 32-bit list, so every
 * diverging index called the WRONG win32k routine (NtGdiInit@219-in-32 hit
 * NtGdiModifyWorldTransform@219-in-64; real GdiInit is 481). Native
 * gdi32/user32 were immune (win32ss/sys-stubs.S has a correct _M_ARM64
 * branch); only win32u callers — the wow64 host — saw it. */
#ifdef _M_IX86
#include <w32ksvc32.h>
#else
#include <w32ksvc64.h>
#endif

END
