/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/ke/arm64/boot.S
 * PURPOSE:         ARM64 Boot Stack Switch Trampoline
 */

.text

/*
 * VOID NTAPI KiSwitchToBootStack(ULONG_PTR InitialStack)
 *
 * Switches to the kernel boot stack and calls KiSystemStartupBootStack.
 * x0 = new stack pointer (top of stack, 16-byte aligned)
 */
.globl KiSwitchToBootStack
KiSwitchToBootStack:
    mov sp, x0
    b KiSystemStartupBootStack
