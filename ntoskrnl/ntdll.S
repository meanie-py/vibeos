
#include <syscalls.inc>

/* This file only has the NT5 syscalls */
#define SYSFUNCS_NT5_ONLY

#ifdef _M_ARM

    TEXTAREA

#define SVC_(name, argcount) STUB_U name

#include <sysfuncs.h>

    END

#else

.code

#ifdef _M_IX86
/* The syscall dispatch slot every i386 user stub calls through (see
 * STUBCODE_U / DEFINE_WOW64_TRANSITION in syscalls.inc). Native boots take
 * the static fallback (SharedUserData->SystemCall, one jmp longer than
 * before). Under WoW64 the host wow64.dll writes the CPU layer's BOP bridge
 * address into the guest ntdll32 mapping's slot (its own COW data page), so
 * the exact same binary serves system32 and syswow64. Exported via
 * ntdll.spec. */
DEFINE_WOW64_TRANSITION
#endif

#define SVC_(name, argcount) STUB_U name, argcount

#include <sysfuncs.h>

END
#endif
