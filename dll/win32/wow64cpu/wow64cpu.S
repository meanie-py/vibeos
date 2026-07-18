/*
 * PROJECT:     vibeos wow64cpu.dll
 * FILE:        dll/win32/wow64cpu/wow64cpu.S
 * PURPOSE:     amd64 heaven's-gate mode-switch trampolines (Phase E).
 *
 * Three routines share one CPU-area WOW64_CONTEXT (installed by the host at
 * TEB->TlsSlots[WOW64_TLS_CPURESERVED] + 16):
 *
 *   BTCpuSimulate     (exported) 64-bit entry from the host; never returns.
 *                     Stashes the 64-bit dispatcher RSP and enters the guest.
 *   EnterGuest        internal; builds an iretq frame from the context and
 *                     drops into 32-bit compatibility mode (CS 0x23).
 *   Wow64CpuSyscall64 internal; the 64-bit landing pad the below-2GB
 *                     trampoline far-jumps to on a guest `call [Wow64Transition]`.
 *                     Saves guest GPRs into the context, calls Wow64CpuDispatch
 *                     (C), then re-enters the guest.
 *
 * The trick that makes this cheap: r15 pins the CPU-area context across guest
 * execution. 32-bit (compat-mode) code cannot touch r15 or the upper halves of
 * the legacy registers, so r15 survives the round trip and we never need to
 * re-derive the TEB through gs after the mode switch (gs is deliberately left
 * untouched so its base stays TEB64 for the 64-bit dispatcher and C code).
 */

#include <asm.inc>

/* Selectors (amd64 boot GDT — freeldr Amd64SetupGdt). */
#define SEL_R3_DATA   HEX(2B)   /* KGDT64_R3_DATA  | RPL 3  (ds/es/ss/gs) */
#define SEL_R3_CMCODE HEX(23)   /* KGDT64_R3_CMCODE| RPL 3  (32-bit guest CS) */
#define SEL_R3_CMTEB  HEX(53)   /* KGDT64_R3_CMTEB | RPL 3  (32-bit guest FS) */
#define SEL_R3_CODE64 HEX(33)   /* KGDT64_R3_CODE  | RPL 3  (64-bit CS) */

/* TEB (amd64): Self @ 0x30, TlsSlots @ 0x1480; WOW64_TLS_CPURESERVED == 1. */
#define TEB_SELF      HEX(30)
#define TEB_CPUAREA   HEX(1488) /* TlsSlots[1] */

/* i386 CONTEXT field offsets inside the CPU-area WOW64_CONTEXT (== CpuArea+16).
 * Kept in sync with wow64cpu.c (CTX_*). */
#define WCTX_Edi      HEX(9C)
#define WCTX_Esi      HEX(A0)
#define WCTX_Ebx      HEX(A4)
#define WCTX_Edx      HEX(A8)
#define WCTX_Ecx      HEX(AC)
#define WCTX_Eax      HEX(B0)
#define WCTX_Ebp      HEX(B4)
#define WCTX_Eip      HEX(B8)
#define WCTX_EFlags   HEX(C0)
#define WCTX_Esp      HEX(C4)

EXTERN Wow64CpuDispatch:PROC

.code

/*
 * VOID WINAPI BTCpuSimulate(VOID);
 *
 * Entered in 64-bit mode with gs base = TEB64 (the host just called us). Stash
 * our 64-bit RSP for the syscall dispatcher to reuse, then enter the guest from
 * the CPU-area context. Never returns (the guest exits via NtTerminateProcess).
 */
PUBLIC BTCpuSimulate
.PROC BTCpuSimulate
    .endprolog
    mov rax, gs:[TEB_SELF]              /* TEB64 self */
    mov rax, [rax + TEB_CPUAREA]        /* CpuArea = TlsSlots[1] */
    mov [rax + 8], rsp                  /* stash dispatcher RSP (CpuArea+8) */
    lea rcx, [rax + 16]                 /* rcx = ctx (WOW64_CONTEXT*) */
    jmp EnterGuest
.ENDP

/*
 * EnterGuest(rcx = ctx) — build an iretq frame and drop into compat mode.
 * Pins ctx in r15 for the syscall path. Leaves gs untouched (base stays TEB64).
 */
.PROC EnterGuest
    .endprolog
    mov r15, rcx                        /* pin ctx across guest execution */

    /* Clear the REPLACED flag (CpuArea+4 == ctx-12): a fresh guest entry
     * always resumes THIS exact context, so any pending "skip fixup" signal
     * from a context-replacing service has already been consumed by the
     * dispatch that set it. Leaving it set would poison the next syscall. */
    mov dword ptr [r15 - 12], 0

    /* Guest data segments (gs deliberately left alone). */
    mov ax, SEL_R3_DATA
    mov ds, ax
    mov es, ax
    mov ax, SEL_R3_CMTEB
    mov fs, ax

    /* iretq frame (popped as RIP, CS, RFLAGS, RSP, SS): push in reverse. */
    push SEL_R3_DATA                    /* SS */
    mov eax, [r15 + WCTX_Esp]
    push rax                            /* RSP (zero-extended) */
    mov eax, [r15 + WCTX_EFlags]
    push rax                            /* RFLAGS */
    push SEL_R3_CMCODE                  /* CS (32-bit code) */
    mov eax, [r15 + WCTX_Eip]
    push rax                            /* RIP (zero-extended) */

    /* Load guest GPRs (rax/ecx read from ctx last — r15 is the ctx base). */
    mov ebx, [r15 + WCTX_Ebx]
    mov edx, [r15 + WCTX_Edx]
    mov esi, [r15 + WCTX_Esi]
    mov edi, [r15 + WCTX_Edi]
    mov ebp, [r15 + WCTX_Ebp]
    mov ecx, [r15 + WCTX_Ecx]
    mov eax, [r15 + WCTX_Eax]
    iretq
.ENDP

/*
 * Wow64CpuSyscall64 — the 64-bit landing pad. Reached from the below-2GB
 * trampoline's `jmp rax` after a guest `call [Wow64Transition]`:
 *   r10d = guest EAX (syscall number)   rsp = guest ESP (32-bit stack)
 *   r15  = CPU-area context (pinned)    rbx/ecx/edx/esi/edi/ebp = guest GPRs
 */
PUBLIC Wow64CpuSyscall64
.PROC Wow64CpuSyscall64
    .endprolog
    mov r11, rsp                        /* r11 = guest ESP (low 32 valid) */

    /* Save the live guest GPRs into the context before clobbering them. */
    mov [r15 + WCTX_Ebx], ebx
    mov [r15 + WCTX_Edx], edx
    mov [r15 + WCTX_Ecx], ecx
    mov [r15 + WCTX_Esi], esi
    mov [r15 + WCTX_Edi], edi
    mov [r15 + WCTX_Ebp], ebp
    mov [r15 + WCTX_Eax], r10d          /* guest EAX (syscall number) */

    /* Save the LIVE Esp/Eip too. The fixup only writes these when the
     * dispatch RETURNS — so while a service is in flight the context held
     * the PREVIOUS syscall's values, and a kernel->user callback fired from
     * inside win32k derived the guest-side dispatcher stack from that stale
     * Esp: the nested frame landed ON TOP of everything the guest pushed
     * since, smashing live locals (doom died in RtlInitUnicodeString on a
     * clobbered string pointer exactly this way). FEX tracks the live guest
     * state internally on arm64, which is why only amd64 needed this. */
    mov [r15 + WCTX_Esp], r11d          /* true guest ESP (ret-addr slot) */
    mov eax, [r11]                      /* return address the stub pushed */
    mov [r15 + WCTX_Eip], eax           /* = where the guest will resume */

    /* Switch to the 64-bit dispatcher stack (BTCpuSimulate stashed it at
     * CpuArea+8; ctx == CpuArea+16, so that slot is r15-8). */
    mov rsp, [r15 - 8]
    and rsp, -16                        /* 16-align for the Win64 call */

    /* Wow64CpuDispatch(ctx, guest_esp) */
    mov rcx, r15
    mov edx, r11d                       /* zero-extend guest ESP */
    sub rsp, 32                         /* shadow space */
    call Wow64CpuDispatch

    /* ctx now holds Eax=result, Eip=return-into-stub, Esp=guest_esp+4. */
    mov rcx, r15
    jmp EnterGuest
.ENDP

END
