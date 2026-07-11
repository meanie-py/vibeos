
#include <syscalls.inc>

#if defined(_M_ARM)

    TEXTAREA

#define SVC_(name, argcount) STUB_U name

#include "w32ksvc32.h"

    END

#elif defined(_M_ARM64)

    .text

SyscallId = 0x1000
#define SVC_(name, argcount) STUB_U name, argcount

#include "w32ksvc64.h"

    END

#else

.code

#ifdef _M_IX86
/* Per-module WoW64 transition slot for the win32k stubs (see syscalls.inc);
 * native boots use the fallback path, a future wow64win.dll patches it. */
DEFINE_WOW64_TRANSITION
#endif

SyscallId = HEX(1000)
#define SVC_(name, argcount) STUB_U name, argcount

#ifdef _WIN64
#include "w32ksvc64.h"
#else
#include "w32ksvc32.h"
#endif

    END
#endif

