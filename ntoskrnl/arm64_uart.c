/*
 * arm64_uart.c — ARM64-only UART debug output helpers.
 *
 * `DbgPrint` doesn't reach serial on ARM64 because the kernel debug
 * transport isn't wired up at the bringup-path callsites (PsCreateProcess,
 * config init, etc.). These two functions poke `Arm64UartKernelBase`
 * directly so any Rust caller can emit progress over UART.
 *
 * On i386/amd64 the bodies are no-ops — the symbols still need to exist
 * so the cross-arch Rust callers (cminit.rs, kill.rs, process.rs, psmgr.rs)
 * can declare them unconditionally.
 *
 * Headers: `<ntdef.h>` for VOID / NTAPI / PCSTR / PVOID / ULONG_PTR / ULONG.
 */

#include <ntdef.h>

#ifdef _M_ARM64
extern volatile PVOID Arm64UartKernelBase;
#endif

VOID NTAPI ARM64_UART_TRACE(PCSTR msg)
{
#ifdef _M_ARM64
    volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
    if (u) { while (*msg) *u = (ULONG)*msg++; }
#else
    (void)msg;
#endif
}

VOID NTAPI ARM64_UART_TRACE_HEX(PCSTR prefix, ULONG_PTR val)
{
#ifdef _M_ARM64
    static const char hex[] = "0123456789ABCDEF";
    int i;
    volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
    if (!u) return;
    while (*prefix) *u = (ULONG)*prefix++;
    *u = '0'; *u = 'x';
    for (i = (sizeof(val)*2 - 1); i >= 0; i--)
        *u = hex[(val >> (i*4)) & 0xF];
#else
    (void)prefix;
    (void)val;
#endif
}
