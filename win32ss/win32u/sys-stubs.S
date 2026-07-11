
#include <asm.inc>
#include <syscalls.inc>

.code

#ifdef _M_IX86
/* Per-module WoW64 transition slot (see syscalls.inc DEFINE_WOW64_TRANSITION). */
DEFINE_WOW64_TRANSITION
#endif

SyscallId = HEX(1000)
#define SVC_(name, argcount) STUB_U name, argcount

#ifdef _M_AMD64
#include <w32ksvc64.h>
#else
#include <w32ksvc32.h>
#endif

END
