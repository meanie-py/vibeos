
#ifdef _M_ARM

#include <ksarm.h>

    GBLA SyscallId

SyscallId SETA 0

	MACRO
	STUB_K $Name
	LCLS ZwFuncName
	LCLS ZwFuncEndName
ZwFuncName    SETS "Zw$Name"
ZwFuncEndName SETS "$ZwFuncName":CC:"_end"
	ALIGN 2
	EXPORT $ZwFuncName [FUNC]
$ZwFuncName
	ROUT
    mov r12, #SyscallId
    svc #1
    bx lr
$ZwFuncEndName
    MEND

	MACRO
	STUB_U $Name
	LCLS NtFuncName
	LCLS NtFuncEndName
	LCLS ZwFuncName
	LCLS ZwFuncEndName
NtFuncName    SETS "Nt$Name"
NtFuncEndName SETS "$NtFuncName":CC:"_end"
ZwFuncName    SETS "Zw$Name"
ZwFuncEndName SETS "$ZwFuncName":CC:"_end"
	ALIGN 2
	EXPORT $NtFuncName [FUNC]
$NtFuncName
	EXPORT $ZwFuncName [FUNC]
$ZwFuncName
	ROUT
    mov r12, #SyscallId
    svc #1
    bx lr
$NtFuncEndName
$ZwFuncEndName
    MEND

#else

#include <asm.inc>

SyscallId = 0

#ifdef _M_IX86
#define KUSER_SHARED_SYSCALL HEX(7ffe0300)
#define KGDT_R0_CODE 8
/* User stubs dispatch through the Wow64Transition slot (defined in ntdll.S):
 * statically -> KiIndirectSystemCall (jmp [SharedUserData->SystemCall], the
 * old direct path) for native i386 boots; under WoW64 the host wow64.dll
 * patches the slot in the guest ntdll32 mapping (per-process COW page) to the
 * CPU layer's BOP bridge — Win10-parity, one binary for system32 AND syswow64.
 * Stack shape at the transition target is unchanged (ret-to-stub at [esp],
 * ret-to-caller at [esp+4], args at [esp+8]). */
MACRO(STUBCODE_U, Name, SyscallId, ArgCount)
    StackBytes = 4 * ArgCount
    FPO 0, 0, 0, 0, 0, FRAME_FPO
    mov eax, SyscallId
    call dword ptr [_Wow64Transition]
    ret StackBytes
ENDM
/* Instantiate ONCE in every i386 TU that expands STUB_U (ntdll.S,
 * win32ss/sys-stubs.S): the slot the stubs call through plus its native
 * fallback. Win10 keeps one such slot per syscall-stub module too (ntdll32,
 * win32u) — the WoW64 host patches each module's slot separately. */
MACRO(DEFINE_WOW64_TRANSITION)
PUBLIC _KiIndirectSystemCall
_KiIndirectSystemCall:
    mov edx, KUSER_SHARED_SYSCALL
    jmp dword ptr [edx]
.data
.align 4
PUBLIC _Wow64Transition
_Wow64Transition:
    .long _KiIndirectSystemCall
.code
ENDM
MACRO(STUBCODE_K, Name, SyscallId, ArgCount)
    StackBytes = 4 * \ArgCount
    FPO 0, 0, 0, 0, 0, FRAME_FPO
    mov eax, SyscallId
    lea edx, [esp + 4]
    pushfd
    push KGDT_R0_CODE
    call _KiSystemService
    ret StackBytes
ENDM
#elif defined(_M_AMD64)
MACRO(STUBCODE_U, Name, SyscallId, ArgCount)
    .ENDPROLOG
    mov eax, SyscallId
    mov r10, rcx
    syscall
    ret
ENDM
MACRO(STUBCODE_K, Name, SyscallId, ArgCount)
    .ENDPROLOG
    EXTERN Nt\Name:PROC
    lea rax, Nt\Name\()[rip]
    mov r10, ArgCount * 8
    jmp KiZwSystemService
ENDM
#elif defined(_M_PPC)
MACRO(STUBCODE_U, Name, SyscallId, ArgCount)
    stwu 1,-16(1)
    mflr 0
    stw  0,0(1)
    li   0, SyscallId
    sc
    lwz 0,0(1)
    mtlr 0
    addi 1,1,16
    blr
ENDM
#define STUBCODE_K STUBCODE_U
#elif defined(_M_MIPS)
MACRO(STUBCODE_U, Name, SyscallId, ArgCount)
    li $8, KUSER_SHARED_SYSCALL
    lw $8,0($8)
    j $8
    nop
ENDM
MACRO(STUBCODE_K, Name, SyscallId, ArgCount)
    j KiSystemService
    nop
ENDM
#elif defined(_M_ARM64)
MACRO(STUBCODE_U, Name, SyscallId, ArgCount)
    mov x8, SyscallId
    svc #0
    ret
ENDM
MACRO(STUBCODE_K, Name, SyscallId, ArgCount)
    /* ARM64 kernel syscall stubs - TODO: implement properly */
    mov x8, SyscallId
    svc #0
    ret
ENDM
#else
#error unsupported architecture
#endif

#ifdef _M_IX86
MACRO(MAKE_LABEL, Name, StackBytes)
    PUBLIC _\Name\()@\StackBytes
    _\Name\()@\StackBytes\():
ENDM
MACRO(START_PROC, Name, StackBytes)
    PUBLIC _\Name\()@\StackBytes
    .PROC _\Name\()@\StackBytes
ENDM
#elif defined(_M_ARM64)
/* ARM64 uses GAS-style syntax with Clang */
.macro MAKE_LABEL Name, StackBytes
    .globl \Name
    \Name:
.endm
.macro START_PROC Name, StackBytes
    .globl \Name
    .p2align 2
    \Name:
.endm
.macro STUB_U Name, ArgCount
    MAKE_LABEL Zw\Name, 0
    START_PROC Nt\Name, 0
    mov x8, #SyscallId
    svc #0
    ret
    SyscallId = SyscallId + 1
.endm
.macro STUB_K Name, ArgCount
    START_PROC Zw\Name, 0
    /* ARM64 kernel-mode Zw* — direct dispatch, no SVC.
     * x0-x7 contain the original syscall arguments (untouched).
     * x8 = syscall ID (set below, used by KiArm64ZwDispatch).
     * KiArm64ZwDispatch is a C function that reads x8 via inline asm,
     * sets PreviousMode=KernelMode, dispatches Nt*, restores PreviousMode. */
    mov x8, #SyscallId
    b KiArm64ZwDispatch
    SyscallId = SyscallId + 1
.endm
#else
MACRO(MAKE_LABEL, Name, StackBytes)
    PUBLIC \Name
    \Name\():
ENDM
MACRO(START_PROC, Name, StackBytes)
    PUBLIC \Name
    .PROC \Name
ENDM
#endif

#ifndef _M_ARM64
MACRO(STUB_U, Name, ArgCount)
    MAKE_LABEL Zw\Name, %ArgCount * 4
    START_PROC Nt\Name, %ArgCount * 4
    STUBCODE_U Name, SyscallId, %ArgCount
    .ENDP
    SyscallId = SyscallId + 1
ENDM

MACRO(STUB_K, Name, ArgCount)
    START_PROC Zw\Name, %ArgCount * 4
    STUBCODE_K Name, SyscallId, %ArgCount
    .ENDP
    SyscallId = SyscallId + 1
ENDM
#endif

#endif
