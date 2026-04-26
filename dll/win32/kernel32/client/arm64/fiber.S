/*
 * COPYRIGHT:   See COPYING in the top level directory
 * PROJECT:     ReactOS system libraries
 * FILE:        dll/win32/kernel32/client/arm64/fiber.S
 * PURPOSE:     Fiber context switch code for the arm64 architecture
 */

#include <asm.inc>

    TEXTAREA

    LEAF_ENTRY BaseFiberStartup
    /* FIXME: TODO */
    brk #0xf000
    LEAF_END BaseFiberStartup

    LEAF_ENTRY SwitchToFiber
    /* FIXME: TODO */
    brk #0xf000
    ret
    LEAF_END SwitchToFiber

    END
