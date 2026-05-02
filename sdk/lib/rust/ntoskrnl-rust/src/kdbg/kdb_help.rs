//! Translated from `ntoskrnl/kdbg/{i386,amd64}/kdb_help.S` —
//! `KdbpStackSwitchAndCall` stack-switching trampoline.
//!
//! Saves the current stack pointer, swaps in `NewStack`, calls `Procedure`,
//! then restores the original stack. KDB uses this to give its interactive
//! prompt its own large stack so command parsing doesn't trample the trapped
//! thread's stack.

#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".intel_syntax noprefix",
    // i386 stdcall: VOID NTAPI KdbpStackSwitchAndCall(PVOID NewStack, VOID (*Procedure)(VOID))
    // Two PVOID args = 8 bytes; mangled name is _KdbpStackSwitchAndCall@8.
    ".globl _KdbpStackSwitchAndCall@8",
    "_KdbpStackSwitchAndCall@8:",
    "    push  ebp",
    "    mov   ebp, esp",
    "    mov   eax, [esp + 8]",   // NewStack
    "    mov   ecx, [esp + 12]",  // Procedure
    "    mov   edx, esp",         // save old stack in edx
    "    mov   esp, eax",         // switch
    "    push  edx",              // stash old stack on new stack
    "    call  ecx",
    "    pop   esp",              // restore old stack
    "    pop   ebp",
    "    ret   8",
);

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".intel_syntax noprefix",
    // amd64 Win64: VOID KdbpStackSwitchAndCall(PVOID NewStack, VOID (*Procedure)(VOID))
    // 1st arg = rcx (NewStack), 2nd arg = rdx (Procedure). No name decoration.
    ".globl KdbpStackSwitchAndCall",
    "KdbpStackSwitchAndCall:",
    "    mov   rax, rsp",   // save old stack
    "    mov   rsp, rcx",   // switch
    "    push  rax",        // stash old stack on new stack
    "    call  rdx",
    "    pop   rax",        // restore
    "    mov   rsp, rax",
    "    ret",
);
