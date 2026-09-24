/*
 * PROJECT:     vibeos wow64.dll
 * PURPOSE:     Kernel->user callback bridge for WoW64 (M8). win32k delivers
 *              user-mode callbacks through the 64-bit ntdll's
 *              KiUserCallbackDispatcher -> PEB64->KernelCallbackTable. In a
 *              WoW64 process no native user32 is loaded, so WE own that
 *              table: each entry marshals the argument struct 64->32, pushes
 *              an i386 KiUserCallbackDispatcher frame on the guest stack and
 *              re-enters BTCpuSimulate (nested — FEX supports this; it is
 *              exactly wine's Wow64KiUserCallbackDispatcher shape, see FEX
 *              Module.cpp CachedCallRetSp). The guest's 32-bit user32 handles
 *              the callback and exits via ZwCallbackReturn -> BOP -> our
 *              manual NtCallbackReturn marshaller, which longjmps back here;
 *              we restore the saved guest context, translate the 32-bit
 *              result buffer up to 64-bit, and issue the REAL NtCallbackReturn
 *              from this (still-live) host frame. The kernel then resumes the
 *              original syscall; this frame is abandoned by design.
 *
 *              Callback indices/arg structs: win32ss/include/callback.h +
 *              user32's apfnDispatch order (misc/dllmain.rs). Only the
 *              callbacks the M8 window path needs marshal args for real;
 *              everything else fails LOUD with STATUS_NOT_IMPLEMENTED so
 *              serial names the next gap.
 */

#define WIN32_NO_STATUS
#include <windef.h>

typedef LONG NTSTATUS;

ULONG WINAPI DbgPrint(_In_z_ _Printf_format_string_ PCSTR Format, ...);
NTSTATUS WINAPI NtCallbackReturn(PVOID Result, ULONG ResultLength, NTSTATUS Status);
NTSTATUS WINAPI NtTerminateProcess(HANDLE Process, NTSTATUS ExitStatus);

#define WOW64_CURRENT_PROCESS ((HANDLE)(LONG_PTR)-1)
#define WOW64_STATUS_NOT_IMPLEMENTED ((NTSTATUS)0xC0000002L)

/* ---- shared with wow64.c ------------------------------------------------ */
extern ULONG Wow64TraceSyscalls;
extern LONG (WINAPI *Wow64pBTCpuSetContext)(HANDLE, HANDLE, PVOID, PVOID);
extern LONG (WINAPI *Wow64pBTCpuGetContext)(HANDLE, HANDLE, PVOID, PVOID);
extern VOID (WINAPI *Wow64pBTCpuSimulate)(VOID);
extern PVOID Wow64pKiUserCallbackDispatcher32; /* guest ntdll32 export */

/* wow64.c — the guest/host last-error mirror. This file owns its 64->32
 * boundary; wow64_win32k.c owns the 32->64 one. */
ULONG Wow64LastErrorToHost(VOID);
VOID Wow64LastErrorToGuest(VOID);

#define WOW64_TEB_TLSSLOTS_OFFSET 0x1480
#define WOW64_TLS_CBFRAME 12   /* FEX WOW64 layer uses 15..18; wow64.c uses 1,10 */
#define WOW64_TEB_PEB_OFFSET 0x60
#define WOW64_PEB_KERNELCALLBACKTABLE 0x58

static PVOID *Wow64pCbTlsSlot(VOID)
{
    return (PVOID *)((PBYTE)NtCurrentTeb() + WOW64_TEB_TLSSLOTS_OFFSET
                     + WOW64_TLS_CBFRAME * sizeof(PVOID)) ;
}

#ifdef _M_AMD64
/* Native wow64cpu contract: the CPU-area (TlsSlots[WOW64_TLS_CPURESERVED]) has
 * a "resume replaced" flag at +4 that BTCpuSetContext sets and the syscall
 * dispatcher consumes to skip the return-to-stub fixup after a context-
 * replacing service (NtContinue). The callback-return path below calls
 * BTCpuSetContext to RESTORE the pre-callback guest state, which sets that flag
 * too — but here the OUTER syscall (the one the callback interrupted) must
 * still complete with its normal fixup. Clear the flag so the outer dispatch
 * does not mistake this restore for an NtContinue. (No-op-costed on arm64,
 * where FEX owns the resume model — hence the _M_AMD64 guard.) */
#define WOW64_TLS_CPURESERVED_SLOT 1
static void Wow64pClearCpuResumeFlag(void)
{
    PBYTE *Tls = (PBYTE *)((PBYTE)NtCurrentTeb() + WOW64_TEB_TLSSLOTS_OFFSET);
    PBYTE CpuArea = Tls[WOW64_TLS_CPURESERVED_SLOT];
    if (CpuArea)
        *(volatile ULONG *)(CpuArea + 4) = 0;
}

/* CpuArea+8 is where BTCpuSimulate stashes the 64-bit dispatcher RSP, and it
 * is the ONLY thing the 32->64 syscall transition has to find its native
 * stack with (`mov rsp, [r15 - 8]` in wow64cpu.S). A user callback runs the
 * guest through a NESTED BTCpuSimulate, which overwrites that slot with the
 * deeper RSP -- and the callback never returns through the asm, so nothing
 * puts the outer value back. Every subsequent guest syscall then switched to
 * the stale, lower stack, so each callback permanently consumed one
 * dispatcher frame of the thread's 1MB native stack: taskmgr died at ~950
 * callbacks, and the death was a genuine stack exhaustion with no runaway
 * recursion behind it. Save around the nesting, restore on the way out. */
static ULONGLONG *Wow64pCpuDispatchRspSlot(void)
{
    PBYTE *Tls = (PBYTE *)((PBYTE)NtCurrentTeb() + WOW64_TEB_TLSSLOTS_OFFSET);
    PBYTE CpuArea = Tls[WOW64_TLS_CPURESERVED_SLOT];
    return CpuArea ? (ULONGLONG *)(CpuArea + 8) : NULL;
}
#endif

/* ---- minimal setjmp/longjmp (ntdll-only DLL: no CRT import) --------------
 * Saves callee-saved state + the caller's SP/return address; exactly what a
 * longjmp from a deeper frame on the SAME host stack needs. Two arch flavors:
 * arm64 (x19-x28, fp, lr, sp, d8-d15 -> 21 slots) and amd64 (rbx/rbp/rdi/rsi/
 * r12-r15, caller-rsp, retaddr, xmm6-xmm15 -> 30 slots). WOW64_CB_JMP_SLOTS
 * sizes the buffer for the larger. */
#ifdef _M_ARM64
__attribute__((naked)) int Wow64pSetJmp(unsigned long long *Buf)
{
    __asm__ volatile(
        "stp x19, x20, [x0, #0]\n"
        "stp x21, x22, [x0, #16]\n"
        "stp x23, x24, [x0, #32]\n"
        "stp x25, x26, [x0, #48]\n"
        "stp x27, x28, [x0, #64]\n"
        "stp x29, x30, [x0, #80]\n"
        "mov x1, sp\n"
        "str x1, [x0, #96]\n"
        "stp d8,  d9,  [x0, #104]\n"
        "stp d10, d11, [x0, #120]\n"
        "stp d12, d13, [x0, #136]\n"
        "stp d14, d15, [x0, #152]\n"
        "mov w0, #0\n"
        "ret\n");
}

__attribute__((naked, noreturn)) void Wow64pLongJmp(unsigned long long *Buf, int Val)
{
    __asm__ volatile(
        "ldp x19, x20, [x0, #0]\n"
        "ldp x21, x22, [x0, #16]\n"
        "ldp x23, x24, [x0, #32]\n"
        "ldp x25, x26, [x0, #48]\n"
        "ldp x27, x28, [x0, #64]\n"
        "ldp x29, x30, [x0, #80]\n"
        "ldr x2, [x0, #96]\n"
        "mov sp, x2\n"
        "ldp d8,  d9,  [x0, #104]\n"
        "ldp d10, d11, [x0, #120]\n"
        "ldp d12, d13, [x0, #136]\n"
        "ldp d14, d15, [x0, #152]\n"
        "mov w0, w1\n"
        "cmp w0, #0\n"
        "csinc w0, w0, wzr, ne\n"
        "ret\n");
}
#else /* _M_AMD64 — Windows x64 ABI: Buf in rcx, Val in rdx. Non-volatile GPRs
       * at 0..63, caller rsp at 64, return address at 72, xmm6-xmm15 at
       * 80..239 (movups: buffer need not be 16-aligned). Saving nonvol XMMs is
       * belt-and-suspenders — the marshalling code is integer-only, but a
       * compiler SSE spill into xmm6-15 across the setjmp would otherwise leak
       * past the longjmp. */
__attribute__((naked)) int Wow64pSetJmp(unsigned long long *Buf)
{
    __asm__ volatile(
        "movq %rbx, 0(%rcx)\n"
        "movq %rbp, 8(%rcx)\n"
        "movq %rdi, 16(%rcx)\n"
        "movq %rsi, 24(%rcx)\n"
        "movq %r12, 32(%rcx)\n"
        "movq %r13, 40(%rcx)\n"
        "movq %r14, 48(%rcx)\n"
        "movq %r15, 56(%rcx)\n"
        "leaq 8(%rsp), %rax\n"      /* caller's rsp (after the call's push)   */
        "movq %rax, 64(%rcx)\n"
        "movq (%rsp), %rax\n"       /* return address                        */
        "movq %rax, 72(%rcx)\n"
        "movups %xmm6,  80(%rcx)\n"
        "movups %xmm7,  96(%rcx)\n"
        "movups %xmm8,  112(%rcx)\n"
        "movups %xmm9,  128(%rcx)\n"
        "movups %xmm10, 144(%rcx)\n"
        "movups %xmm11, 160(%rcx)\n"
        "movups %xmm12, 176(%rcx)\n"
        "movups %xmm13, 192(%rcx)\n"
        "movups %xmm14, 208(%rcx)\n"
        "movups %xmm15, 224(%rcx)\n"
        "xorl %eax, %eax\n"
        "ret\n");
}

__attribute__((naked, noreturn)) void Wow64pLongJmp(unsigned long long *Buf, int Val)
{
    __asm__ volatile(
        "movq 0(%rcx), %rbx\n"
        "movq 8(%rcx), %rbp\n"
        "movq 16(%rcx), %rdi\n"
        "movq 24(%rcx), %rsi\n"
        "movq 32(%rcx), %r12\n"
        "movq 40(%rcx), %r13\n"
        "movq 48(%rcx), %r14\n"
        "movq 56(%rcx), %r15\n"
        "movups 80(%rcx),  %xmm6\n"
        "movups 96(%rcx),  %xmm7\n"
        "movups 112(%rcx), %xmm8\n"
        "movups 128(%rcx), %xmm9\n"
        "movups 144(%rcx), %xmm10\n"
        "movups 160(%rcx), %xmm11\n"
        "movups 176(%rcx), %xmm12\n"
        "movups 192(%rcx), %xmm13\n"
        "movups 208(%rcx), %xmm14\n"
        "movups 224(%rcx), %xmm15\n"
        "movl %edx, %eax\n"         /* return Val (or 1 if Val==0)           */
        "testl %eax, %eax\n"
        "jne 1f\n"
        "incl %eax\n"
        "1:\n"
        "movq 72(%rcx), %r8\n"      /* retaddr into a volatile scratch       */
        "movq 64(%rcx), %rsp\n"     /* switch back to the caller's stack     */
        "jmp *%r8\n");
}
#endif

/* ---- WOW64_CONTEXT (i386 CONTEXT, matches wow64.c usage) ----------------- */
typedef struct _WOW64_CB_CONTEXT {
    ULONG ContextFlags;
    ULONG Dr0, Dr1, Dr2, Dr3, Dr6, Dr7;
    UCHAR FloatSave[112];
    ULONG SegGs, SegFs, SegEs, SegDs;
    ULONG Edi, Esi, Ebx, Edx, Ecx, Eax;
    ULONG Ebp, Eip, SegCs, EFlags, Esp, SegSs;
    UCHAR ExtendedRegisters[512];
} WOW64_CB_CONTEXT;

#define WOW64_CB_CONTEXT_FULL 0x00010007
#define WOW64_CB_CONTEXT_EXTENDED 0x00010020

/* ---- in-flight callback frame -------------------------------------------- */
typedef struct _WOW64_CB_FRAME {
    struct _WOW64_CB_FRAME *Prev;
    ULONG Index;
    PVOID OrigArg;           /* the kernel's 64-bit arg buffer (stays mapped) */
    ULONG ArgLen64;          /* original 64-bit arg length (for result sizing) */
    ULONG Msg;               /* WINDOWPROC message — the payload repack keys off it */
    NTSTATUS Status;         /* filled by the NtCallbackReturn marshaller */
    PVOID RetBuf;            /* 64-bit result to hand to the real NtCallbackReturn */
    ULONG RetLen;
    WOW64_CB_CONTEXT SavedCtx;
    ULONGLONG SavedDispatchRsp; /* amd64: CpuArea+8, clobbered by nesting */
    /* 21 slots on arm64, 30 on amd64 (10 nonvol XMMs); size for the larger. */
    unsigned long long Jmp[32];
    UCHAR Buf64[1024];       /* translated 64-bit result storage */
} WOW64_CB_FRAME;

/* WINDOWPOS: two HWNDs then five 32-bit fields. 40 bytes on the host (8-byte
 * handles + tail padding), 28 in the guest. WM_WINDOWPOSCHANGING/CHANGED pass
 * one by pointer AND the window proc is allowed to MODIFY it (that is the whole
 * point of WM_WINDOWPOSCHANGING), so it must be repacked in BOTH directions. */
typedef struct _WOW64_WINDOWPOS64 {
    ULONGLONG hwnd;
    ULONGLONG hwndInsertAfter;
    LONG x, y, cx, cy;
    ULONG flags;
    ULONG _pad;
} WOW64_WINDOWPOS64; /* 40 */

typedef struct _WOW64_WINDOWPOS32 {
    ULONG hwnd;
    ULONG hwndInsertAfter;
    LONG x, y, cx, cy;
    ULONG flags;
} WOW64_WINDOWPOS32; /* 28 */

/* Messages whose lParam payload the window proc writes back into. For these the
 * RAW copy-down is only half the job: the guest's edits must be copied back up,
 * or they are silently dropped (the pre-M11 behaviour — the return path restored
 * the payload from the kernel's original buffer unconditionally). */
#include "wow64_cb_automarshal.h"   /* M18: generated payload marshallers */
#include "wow64_pfnmap.h"

#define WOW64_WM_COPYDATA         0x004A
#define WOW64_WM_GETMINMAXINFO    0x0024
#define WOW64_WM_WINDOWPOSCHANGING 0x0046
#define WOW64_WM_WINDOWPOSCHANGED  0x0047
#define WOW64_WM_STYLECHANGING    0x007C
#define WOW64_WM_NCCALCSIZE       0x0083
#define WOW64_WM_SIZING           0x0214
#define WOW64_WM_MOVING           0x0216

/* ---- 32/64 mirrors of callback.h structs --------------------------------- */
/* WINDOWPROC_CALLBACK_ARGUMENTS (index 0) */
typedef struct _WOW64_WNDPROC_CB64 {
    ULONGLONG Proc;
    ULONG IsAnsiProc; ULONG _pad0;
    ULONGLONG Wnd;
    ULONG Msg; ULONG _pad1;
    ULONGLONG wParam;
    ULONGLONG lParam;
    LONG lParamBufferSize; ULONG _pad2;
    ULONGLONG Result;
} WOW64_WNDPROC_CB64; /* 64 bytes; trailing lParam buffer follows */

typedef struct _WOW64_WNDPROC_CB32 {
    ULONG Proc;
    ULONG IsAnsiProc;
    ULONG Wnd;
    ULONG Msg;
    ULONG wParam;
    ULONG lParam;
    LONG lParamBufferSize;
    ULONG Result;
} WOW64_WNDPROC_CB32; /* 32 bytes; trailing lParam buffer follows */

/*
 * Wow64pEnterGuestCallback — push an i386 KiUserCallbackDispatcher frame
 * (see dll/ntdll/dispatch/i386/dispatch.S: [ret][Index][Arg][Len][seh][seh])
 * below the guest's parked Esp and simulate. Never returns normally: the
 * guest's ZwCallbackReturn longjmps back into the caller via the frame.
 */
static void Wow64pEnterGuestCallback(WOW64_CB_FRAME *Frame, ULONG Index,
                                     const void *Arg32, ULONG Len32)
{
    WOW64_CB_CONTEXT Ctx;
    ULONG Esp, ArgGuest = 0;

    Ctx = Frame->SavedCtx;
    Esp = Ctx.Esp;

    if (Arg32 && Len32)
    {
        Esp = (Esp - Len32 - 64) & ~15u;
        ArgGuest = Esp;
        {
            ULONG i;
            for (i = 0; i < Len32; i++)
                ((PBYTE)(ULONG_PTR)Esp)[i] = ((const BYTE *)Arg32)[i];
        }
    }
    else
    {
        Esp = (Esp - 64) & ~15u;
    }

    Esp -= 24; /* [ret][Index][Arg][Len][seh1][seh2] */
    *(ULONG *)(ULONG_PTR)(Esp + 0)  = 0;        /* fake return address (popped) */
    *(ULONG *)(ULONG_PTR)(Esp + 4)  = Index;
    *(ULONG *)(ULONG_PTR)(Esp + 8)  = ArgGuest;
    *(ULONG *)(ULONG_PTR)(Esp + 12) = Len32;
    *(ULONG *)(ULONG_PTR)(Esp + 16) = 0;        /* SEH scratch (dispatcher writes) */
    *(ULONG *)(ULONG_PTR)(Esp + 20) = 0;

    Ctx.Eip = (ULONG)(ULONG_PTR)Wow64pKiUserCallbackDispatcher32;
    Ctx.Esp = Esp;
    Ctx.ContextFlags = WOW64_CB_CONTEXT_FULL | WOW64_CB_CONTEXT_EXTENDED;
    Wow64pBTCpuSetContext((HANDLE)(LONG_PTR)-2, WOW64_CURRENT_PROCESS, NULL, &Ctx);

    /* Control is crossing into the guest, so last-error has to cross with it:
     * from here until NtCallbackReturn, GetLastError() means TEB32. The
     * matching copy back up is at the longjmp landing in
     * Wow64pUserCallbackCommon. Without this pair, the guest's own win32k
     * syscalls (which copy TEB32 up on entry) would overwrite an error the
     * OUTER call had already set — co_IntSendMessageTimeout sets ERROR_TIMEOUT
     * and BroadcastSystemMessage reads it back after the send. */
    Wow64LastErrorToGuest();

    Wow64pBTCpuSimulate(); /* exits via guest NtCallbackReturn -> longjmp */

    DbgPrint("[WOW64CB] FATAL: nested BTCpuSimulate returned (idx %lu)\n", Index);
    NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBADCB);
}

/*
 * Common 64-bit KernelCallbackTable entry. Marshals per-index, runs the guest,
 * then (after the longjmp back) restores the guest context and completes the
 * kernel callback with the translated result. Never returns.
 */
static NTSTATUS Wow64pUserCallbackCommon(ULONG Index, PVOID Arg, ULONG Len)
{
    WOW64_CB_FRAME Frame;
    NTSTATUS St;

    if (Wow64TraceSyscalls)
    {
        /* &Frame is the native 64-bit stack pointer at this nesting level, and
         * the Prev chain is the nesting depth. Printing both turns "the thread
         * ran out of its 1MB native stack" into a measurement: a monotonically
         * falling sp at constant depth is a LEAK, a falling sp tracking depth
         * is recursion. Inferring depth from the -> / <- trace does not work,
         * because the unmarshalled return path never prints a <-. */
        ULONG Depth = 0;
        const WOW64_CB_FRAME *W = (const WOW64_CB_FRAME *)*Wow64pCbTlsSlot();
        while (W && Depth < 10000) { Depth++; W = W->Prev; }
        DbgPrint("[WOW64CB] -> callback %lu (arg=%p len=%lu sp=%p depth=%lu)\n",
                 Index, Arg, Len, (PVOID)&Frame, Depth);
    }

    if (!Wow64pKiUserCallbackDispatcher32 || !Wow64pBTCpuGetContext ||
        !Wow64pBTCpuSetContext || !Wow64pBTCpuSimulate)
    {
        DbgPrint("[WOW64CB] callback %lu with no guest dispatcher — failing\n", Index);
        return NtCallbackReturn(NULL, 0, WOW64_STATUS_NOT_IMPLEMENTED);
    }

    Frame.Prev = (WOW64_CB_FRAME *)*Wow64pCbTlsSlot();
    Frame.Index = Index;
    Frame.OrigArg = Arg;
    Frame.ArgLen64 = Len;
    Frame.Status = WOW64_STATUS_NOT_IMPLEMENTED;
    Frame.RetBuf = NULL;
    Frame.RetLen = 0;
    Frame.SavedCtx.ContextFlags = WOW64_CB_CONTEXT_FULL | WOW64_CB_CONTEXT_EXTENDED;
    Wow64pBTCpuGetContext((HANDLE)(LONG_PTR)-2, WOW64_CURRENT_PROCESS, NULL,
                          &Frame.SavedCtx);
    Frame.SavedDispatchRsp = 0;
#ifdef _M_AMD64
    {
        ULONGLONG *Slot = Wow64pCpuDispatchRspSlot();
        if (Slot)
            Frame.SavedDispatchRsp = *Slot;
    }
#endif
    *Wow64pCbTlsSlot() = &Frame;

    if (Wow64pSetJmp(Frame.Jmp) == 0)
    {
        switch (Index)
        {
            case 0: /* WINDOWPROC */
            {
                const WOW64_WNDPROC_CB64 *A64 = (const WOW64_WNDPROC_CB64 *)Arg;
                ULONG Extra = (Len > sizeof(*A64)) ? Len - sizeof(*A64) : 0;
                if (Wow64TraceSyscalls)
                    DbgPrint("[WOW64CB] WINDOWPROC hwnd=%p msg=%04lx wp=%p lp=%p extra=%lu\n",
                             (PVOID)(ULONG_PTR)A64->Wnd, A64->Msg,
                             (PVOID)(ULONG_PTR)A64->wParam, (PVOID)(ULONG_PTR)A64->lParam, Extra);
                /* Frame.Buf64 doubles as 32-bit marshal scratch: it is copied
                 * onto the guest stack before any nesting, and only reused
                 * for the RESULT after the guest completes. */
                UCHAR *Buf = Frame.Buf64;
                WOW64_WNDPROC_CB32 *A32 = (WOW64_WNDPROC_CB32 *)Buf;
                if (Extra > 512)
                {
                    DbgPrint("[WOW64CB] WINDOWPROC extra %lu too big\n", Extra);
                    break;
                }
                Frame.Msg = A64->Msg; /* the return path repacks OUT payloads by msg */
                A32->Proc = Wow64pMapClientProc(A64->Proc);
                A32->IsAnsiProc = A64->IsAnsiProc;
                A32->Wnd = (ULONG)A64->Wnd;
                A32->Msg = A64->Msg;
                A32->wParam = (ULONG)A64->wParam;
                A32->lParam = (ULONG)A64->lParam;
                A32->lParamBufferSize = A64->lParamBufferSize;
                A32->Result = 0;
                if (Extra >= 80 && (A64->Msg == 0x0081 /* WM_NCCREATE */ ||
                                    A64->Msg == 0x0001 /* WM_CREATE  */))
                {
                    /* Trailing payload = packed CREATESTRUCTW (64-bit, 80
                     * bytes) + string blob; lpszName/lpszClass hold OFFSETS
                     * from the struct base (MsgiKMToUMMessage converts them
                     * in place guest-side). Repack as the 48-byte 32-bit
                     * CREATESTRUCT: pointers truncate, offsets shift by the
                     * 32-byte header shrink. */
                    const ULONGLONG *C64 = (const ULONGLONG *)((const BYTE *)Arg + sizeof(*A64));
                    const ULONG *C64d = (const ULONG *)C64;
                    ULONG *C32 = (ULONG *)(Buf + sizeof(*A32));
                    ULONG Blob = Extra - 80, i;
                    C32[0] = (ULONG)C64[0];            /* lpCreateParams */
                    C32[1] = (ULONG)C64[1];            /* hInstance      */
                    C32[2] = (ULONG)C64[2];            /* hMenu          */
                    C32[3] = (ULONG)C64[3];            /* hwndParent     */
                    C32[4] = C64d[8];                  /* cy             */
                    C32[5] = C64d[9];                  /* cx             */
                    C32[6] = C64d[10];                 /* y              */
                    C32[7] = C64d[11];                 /* x              */
                    C32[8] = C64d[12];                 /* style          */
                    C32[9] = C64[7] ? (ULONG)C64[7] - 32 : 0;  /* lpszName offset */
                    C32[10] = C64[8] ? (ULONG)C64[8] - 32 : 0; /* lpszClass offset */
                    C32[11] = C64d[18];                /* dwExStyle      */
                    for (i = 0; i < Blob; i++)
                        Buf[sizeof(*A32) + 48 + i] = ((const BYTE *)Arg)[sizeof(*A64) + 80 + i];
                    A32->lParamBufferSize = 48 + Blob;
                    if (Wow64TraceSyscalls)
                        DbgPrint("[WOW64CB] (NC)CREATE cs64 name=%08lx%08lx cls=%08lx%08lx blob=%lu -> cs32 name=%08lx cls=%08lx\n",
                                 (ULONG)(C64[7] >> 32), (ULONG)C64[7],
                                 (ULONG)(C64[8] >> 32), (ULONG)C64[8],
                                 Blob, C32[9], C32[10]);
                    Wow64pEnterGuestCallback(&Frame, Index, Buf,
                                             sizeof(*A32) + 48 + Blob);
                    break;
                }
                if ((A64->Msg == WOW64_WM_WINDOWPOSCHANGING ||
                     A64->Msg == WOW64_WM_WINDOWPOSCHANGED) &&
                    Extra >= sizeof(WOW64_WINDOWPOS64))
                {
                    /* WINDOWPOS 40 -> 28. Copied back up after the guest runs. */
                    const WOW64_WINDOWPOS64 *P64 =
                        (const WOW64_WINDOWPOS64 *)((const BYTE *)Arg + sizeof(*A64));
                    WOW64_WINDOWPOS32 *P32 = (WOW64_WINDOWPOS32 *)(Buf + sizeof(*A32));
                    P32->hwnd = (ULONG)P64->hwnd;
                    P32->hwndInsertAfter = (ULONG)P64->hwndInsertAfter;
                    P32->x = P64->x;
                    P32->y = P64->y;
                    P32->cx = P64->cx;
                    P32->cy = P64->cy;
                    P32->flags = P64->flags;
                    A32->lParamBufferSize = sizeof(*P32);
                    Wow64pEnterGuestCallback(&Frame, Index, Buf,
                                             sizeof(*A32) + sizeof(*P32));
                    break;
                }
                /* M18: NCCALCSIZE_PARAMS + appended WINDOWPOS (wParam != 0;
                 * wParam == 0 is a bare width-identical RECT and falls through
                 * to the verbatim path). Guest user32 recomputes lppos from the
                 * appended tail (message.c: Params->lppos = (PWINDOWPOS)(Params+1)),
                 * so only the LAYOUT must be right. Generated marshallers. */
                if (A64->Msg == WOW64_WM_NCCALCSIZE && A64->wParam &&
                    Extra >= WOW64_CB_SIZE64_tagNCCALCSIZE_PARAMS +
                             WOW64_CB_SIZE64__WINDOWPOS)
                {
                    const BYTE *S = (const BYTE *)Arg + sizeof(*A64);
                    BYTE *D = Buf + sizeof(*A32);
                    WowCbDown_tagNCCALCSIZE_PARAMS(S, D);
                    WowCbDown__WINDOWPOS(S + WOW64_CB_SIZE64_tagNCCALCSIZE_PARAMS,
                                           D + WOW64_CB_SIZE32_tagNCCALCSIZE_PARAMS);
                    A32->lParamBufferSize = WOW64_CB_SIZE32_tagNCCALCSIZE_PARAMS +
                                            WOW64_CB_SIZE32__WINDOWPOS;
                    Wow64pEnterGuestCallback(&Frame, Index, Buf,
                                             sizeof(*A32) + A32->lParamBufferSize);
                    break;
                }
                /* M18: COPYDATASTRUCT header + trailing data blob (READ-only;
                 * guest recomputes lpData = (cds + 1) in MsgiKMToUMMessage). */
                if (A64->Msg == WOW64_WM_COPYDATA &&
                    Extra >= WOW64_CB_SIZE64_tagCOPYDATASTRUCT)
                {
                    const BYTE *S = (const BYTE *)Arg + sizeof(*A64);
                    BYTE *D = Buf + sizeof(*A32);
                    ULONG Blob = Extra - WOW64_CB_SIZE64_tagCOPYDATASTRUCT;
                    ULONG i;
                    WowCbDown_tagCOPYDATASTRUCT(S, D);
                    for (i = 0; i < Blob; i++)
                        D[WOW64_CB_SIZE32_tagCOPYDATASTRUCT + i] =
                            S[WOW64_CB_SIZE64_tagCOPYDATASTRUCT + i];
                    A32->lParamBufferSize = WOW64_CB_SIZE32_tagCOPYDATASTRUCT + Blob;
                    Wow64pEnterGuestCallback(&Frame, Index, Buf,
                                             sizeof(*A32) + A32->lParamBufferSize);
                    break;
                }
                /* M18: table-driven one-struct payloads (generated:
                 * DRAWITEM / MEASUREITEM / HELP / NEXTMENU today). */
                {
                    const WOW64_CB_MARSHAL *Cbm = Wow64CbFindMarshal(A64->Msg);
                    if (Cbm && Extra >= Cbm->Size64)
                    {
                        Cbm->Down((const BYTE *)Arg + sizeof(*A64),
                                  Buf + sizeof(*A32));
                        A32->lParamBufferSize = Cbm->Size32;
                        Wow64pEnterGuestCallback(&Frame, Index, Buf,
                                                 sizeof(*A32) + Cbm->Size32);
                        break;
                    }
                }
                if (Extra)
                {
                    /* Everything else copies verbatim. That is CORRECT only for
                     * width-identical payloads — MINMAXINFO (5 POINTs) and a bare
                     * RECT are; a payload with a handle or pointer in it is not.
                     * Keep the trace: it is how the next divergent message will
                     * announce itself. */
                    ULONG i;
                    if (Wow64TraceSyscalls)
                        DbgPrint("[WOW64CB] WINDOWPROC msg %lx payload %lu verbatim "
                                 "(width-identical or unclassified)\n",
                                 A64->Msg, Extra);
                    for (i = 0; i < Extra; i++)
                        Buf[sizeof(*A32) + i] = ((const BYTE *)Arg)[sizeof(*A64) + i];
                }
                Wow64pEnterGuestCallback(&Frame, Index, Buf, sizeof(*A32) + Extra);
                break;
            }

            case 7: /* CLIENTTHREADSTARTUP — no argument payload */
                Wow64pEnterGuestCallback(&Frame, Index, NULL, 0);
                break;

            case 8: /* CLIENTLOADLIBRARY: CLIENT_LOAD_LIBRARY_ARGUMENTS 40->24
                     * + trailing string blob. Both UNICODE_STRING Buffers are
                     * SELF-RELATIVE offsets from the struct base (win32k
                     * builds them that way; guest user32 re-bases them at
                     * +Argument), so the squeeze is: narrow both strings,
                     * shift the offsets by the 16-byte header shrink, copy
                     * the blob verbatim. */
            {
                typedef struct { USHORT Len; USHORT Max; ULONG _pad;
                                 ULONGLONG Buf; } WOW64_CLL_US64;
                typedef struct { USHORT Len; USHORT Max; ULONG Buf; } WOW64_CLL_US32;
                typedef struct { WOW64_CLL_US64 Lib, Init;
                                 ULONG Unload, ApiHook; } WOW64_CLL64; /* 40 */
                typedef struct { WOW64_CLL_US32 Lib, Init;
                                 ULONG Unload, ApiHook; } WOW64_CLL32; /* 24 */
                const WOW64_CLL64 *A64 = (const WOW64_CLL64 *)Arg;
                UCHAR *Buf = Frame.Buf64;
                WOW64_CLL32 *A32 = (WOW64_CLL32 *)Buf;
                const ULONG Shrink = sizeof(*A64) - sizeof(*A32);
                ULONG Blob, i;

                if (!Arg || Len < sizeof(*A64) ||
                    Len - Shrink > sizeof(Frame.Buf64))
                {
                    DbgPrint("[WOW64CB] CLIENTLOADLIBRARY arg=%p len=%lu unusable\n",
                             Arg, Len);
                    break;
                }
                Blob = Len - sizeof(*A64);
                A32->Lib.Len = A64->Lib.Len;
                A32->Lib.Max = A64->Lib.Max;
                A32->Lib.Buf = A64->Lib.Buf ? (ULONG)A64->Lib.Buf - Shrink : 0;
                A32->Init.Len = A64->Init.Len;
                A32->Init.Max = A64->Init.Max;
                A32->Init.Buf = A64->Init.Buf ? (ULONG)A64->Init.Buf - Shrink : 0;
                A32->Unload = A64->Unload;
                A32->ApiHook = A64->ApiHook;
                for (i = 0; i < Blob; i++)
                    Buf[sizeof(*A32) + i] = ((const UCHAR *)Arg)[sizeof(*A64) + i];
                if (Wow64TraceSyscalls)
                    DbgPrint("[WOW64CB] CLIENTLOADLIBRARY lib=%.*S unload=%lu apihook=%lu\n",
                             A32->Lib.Buf ? (int)(A32->Lib.Len / sizeof(WCHAR)) : 0,
                             A32->Lib.Buf ? (PCWSTR)(Buf + A32->Lib.Buf) : L"",
                             A32->Unload, A32->ApiHook);
                Wow64pEnterGuestCallback(&Frame, Index, Buf, sizeof(*A32) + Blob);
                break;
            }

            case 3:  /* LOADDEFAULTCURSORS: arg = BOOL* (flat 4 bytes ok) */
            case 11: /* SETWNDICONS: no meaningful payload divergence */
            case 12: /* DELIVERUSERAPC */
            {
                ULONG Flat = Arg ? *(ULONG *)Arg : 0;
                Wow64pEnterGuestCallback(&Frame, Index, Arg ? &Flat : NULL,
                                         Arg ? sizeof(Flat) : 0);
                break;
            }

            default:
                DbgPrint("[WOW64CB] callback %lu NEEDS 64->32 MARSHALLER (len %lu) — failing loud\n",
                         Index, Len);
                break;
        }
        /* Marshalling failed / unimplemented: unwind our frame and tell the
         * kernel the callback failed (loud, non-fatal for optional paths). */
        *Wow64pCbTlsSlot() = Frame.Prev;
#ifdef _M_AMD64
        {
            ULONGLONG *Slot = Wow64pCpuDispatchRspSlot();
            if (Slot && Frame.SavedDispatchRsp)
                *Slot = Frame.SavedDispatchRsp;
        }
#endif
        return NtCallbackReturn(NULL, 0, WOW64_STATUS_NOT_IMPLEMENTED);
    }

    /* longjmp landing: guest completed via NtCallbackReturn marshaller. */
    *Wow64pCbTlsSlot() = Frame.Prev;
    /* ...and control is crossing back into the kernel, so last-error comes
     * with it (pairs with the copy-down in Wow64pEnterGuestCallback). */
    Wow64LastErrorToHost();
    Frame.SavedCtx.ContextFlags = WOW64_CB_CONTEXT_FULL | WOW64_CB_CONTEXT_EXTENDED;
    Wow64pBTCpuSetContext((HANDLE)(LONG_PTR)-2, WOW64_CURRENT_PROCESS, NULL,
                          &Frame.SavedCtx);
#ifdef _M_AMD64
    /* The restore above set the native CPU-area REPLACED flag; the outer
     * syscall that this callback interrupted must still do its normal
     * return-to-stub fixup, so clear it before we hand control back. */
    Wow64pClearCpuResumeFlag();
    {
        ULONGLONG *Slot = Wow64pCpuDispatchRspSlot();
        if (Slot && Frame.SavedDispatchRsp)
            *Slot = Frame.SavedDispatchRsp;
    }
#endif
    St = NtCallbackReturn(Frame.RetBuf, Frame.RetLen, Frame.Status);
    DbgPrint("[WOW64CB] FATAL: NtCallbackReturn(%p,%lu) => %08lx fell through\n",
             Frame.RetBuf, Frame.RetLen, St);
    NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBADCC);
    return St;
}

/*
 * Guest-side completion, called from the manual NtCallbackReturn syscall
 * marshaller (wow64_manual.c) with the guest's 32-bit result buffer. Runs on
 * the nested-Simulate host stack; translates 32->64 into the frame and jumps
 * back to Wow64pUserCallbackCommon. Returns only if no callback is in flight.
 */
int Wow64pCompleteUserCallback(ULONG Result32, ULONG Len32, NTSTATUS Status)
{
    WOW64_CB_FRAME *Frame = (WOW64_CB_FRAME *)*Wow64pCbTlsSlot();

    if (!Frame)
        return 0;

    Frame->Status = Status;
    Frame->RetBuf = NULL;
    Frame->RetLen = 0;

    if (Result32 && Len32)
    {
        switch (Frame->Index)
        {
            case 0: /* WINDOWPROC: header back up to 64-bit; payload restored
                     * from the ORIGINAL kernel buffer (the guest converts the
                     * packed offsets to 32-bit pointers in place, so its copy
                     * is not translatable back — and for the M8 message set
                     * (NCCREATE/CREATE) the payload is effectively IN-only).
                     *
                     * But several messages pass the payload as an OUT parameter:
                     * the window proc WRITES into it and the kernel reads the
                     * result. For those, restoring the kernel's original bytes
                     * silently discards everything the guest just did. Repack
                     * them back up, keyed on the message. */
            {
                const WOW64_WNDPROC_CB32 *A32 =
                    (const WOW64_WNDPROC_CB32 *)(ULONG_PTR)Result32;
                WOW64_WNDPROC_CB64 *A64 = (WOW64_WNDPROC_CB64 *)Frame->Buf64;
                ULONG N = Frame->ArgLen64, i;
                ULONG Extra = (N > sizeof(*A64)) ? N - sizeof(*A64) : 0;
                const BYTE *P32 = (const BYTE *)A32 + sizeof(*A32);
                BYTE *P64 = Frame->Buf64 + sizeof(*A64);

                if (N > sizeof(Frame->Buf64))
                    N = sizeof(Frame->Buf64);
                for (i = 0; i < N; i++)
                    Frame->Buf64[i] = ((const BYTE *)Frame->OrigArg)[i];
                A64->Result = A32->Result;

                /* OUT-payload copy-back (M15a). Several window messages pass
                 * their lParam struct as an OUT parameter: the window proc
                 * WRITES into it and win32k reads the result back
                 * (co_IntCallWindowProc copies lParamBufferSize bytes at
                 * Args+sizeof(ARGS_64) into the 64-bit lParam — callback.rs).
                 * The baseline restore above (from OrigArg) discards the guest's
                 * edits, so for OUT messages we must translate the guest's
                 * 32-bit modified payload back into the 64-bit slot P64.
                 *
                 * Re-enabled INCREMENTALLY, per message, oracle-gated on
                 * win32.exe staying 0x600D. Only width-identical / pointer-free
                 * OUT fields are copied.
                 *
                 * WM_GETMINMAXINFO (40b), WM_NCCALCSIZE (rgrc rects), and
                 * WM_WINDOWPOSCHANGING (28<->40, field-translated) are all
                 * enabled. WINDOWPOSCHANGING was long dropped because a naive
                 * copy-back collapsed the window: the guest's DefWindowProc
                 * returned SWP_NOSIZE-cleared cx=cy=0. That turned out NOT to be
                 * this copy-back's fault — it was the unmarshalled
                 * NtUserMessageCall(FNID_DEFWINDOWPROC) syscall handing win32k the
                 * 28-byte WINDOWPOS to read as 40 (M16 Fix A, wow64_manual.c).
                 * With Fix A the guest returns sane geometry, so this copy-back
                 * is now correct + safe (win32.exe stays 0x600D). */
                switch (Frame->Msg)
                {
                    case WOW64_WM_GETMINMAXINFO: /* MINMAXINFO: 5 POINTs = 40b */
                    case WOW64_WM_SIZING:        /* RECT = 16b */
                    case WOW64_WM_MOVING:        /* RECT = 16b */
                    case WOW64_WM_STYLECHANGING: /* STYLESTRUCT = 8b */
                        /* Width-identical, pointer-free OUT payloads: the guest's
                         * edited struct copies back verbatim (Extra == the 64-bit
                         * payload size == the 32-bit payload size for these). */
                        if (Extra && Len32 >= sizeof(*A32) + Extra)
                        {
                            for (i = 0; i < Extra; i++) P64[i] = P32[i];
                            if (Wow64TraceSyscalls)
                                DbgPrint("[WOW64CB] msg 0x%lx OUT copied back (%lub)\n",
                                         (unsigned long)Frame->Msg,
                                         (unsigned long)Extra);
                        }
                        break;

                    case WOW64_WM_NCCALCSIZE:
                        /* NCCALCSIZE_PARAMS: rgrc[3] (48b, RECTs, width-identical)
                         * then PWINDOWPOS lppos (8b host / 4b guest). The proc
                         * computes the client rect into rgrc[0]; win32k reads it
                         * back to derive the non-client inset. Copy back ONLY the
                         * rects (pointer-free) and PRESERVE the 64-bit lppos from
                         * the baseline restore. wParam==FALSE is a lone RECT
                         * (16b) — Extra distinguishes. */
                    {
                        ULONG cb = (Extra >= 48) ? 48 : Extra;
                        if (cb && Len32 >= sizeof(*A32) + cb)
                        {
                            for (i = 0; i < cb; i++) P64[i] = P32[i];
                            if (Wow64TraceSyscalls)
                                DbgPrint("[WOW64CB] NCCALCSIZE OUT copied back (%lub rects)\n",
                                         (unsigned long)cb);
                        }
                        break;
                    }

                    case WOW64_WM_WINDOWPOSCHANGING:
                        /* WINDOWPOS 28->40, field-translated. Safe now that M16
                         * Fix A (NtUserMessageCall marshalling) makes the guest's
                         * DefWindowProc return SANE geometry — before Fix A it
                         * returned SWP_NOSIZE-cleared cx=cy=0 and copying that
                         * back collapsed the window. Copy geometry + flags;
                         * PRESERVE the 64-bit hwnd/hwndInsertAfter from the
                         * baseline restore (z-order handle sentinels don't
                         * zero-extend; the common proc never repoints them). */
                        if (Len32 >= sizeof(*A32) + sizeof(WOW64_WINDOWPOS32) &&
                            Extra >= sizeof(WOW64_WINDOWPOS64))
                        {
                            const WOW64_WINDOWPOS32 *G32 = (const WOW64_WINDOWPOS32 *)P32;
                            WOW64_WINDOWPOS64 *G64 = (WOW64_WINDOWPOS64 *)P64;
                            G64->x = G32->x;
                            G64->y = G32->y;
                            G64->cx = G32->cx;
                            G64->cy = G32->cy;
                            G64->flags = G32->flags;
                            if (Wow64TraceSyscalls)
                                DbgPrint("[WOW64CB] WINDOWPOSCHANGING OUT copied back "
                                         "(cx=%ld cy=%ld fl=0x%lx)\n",
                                         (long)G64->cx, (long)G64->cy,
                                         (unsigned long)G64->flags);
                        }
                        break;

                    default:
                    {
                        /* M18: table-driven copy-back (generated Up marshallers:
                         * geometry/handles/uintptr values copy back sign-extended;
                         * dereferenceable kernel pointers are preserved). */
                        const WOW64_CB_MARSHAL *Cbm = Wow64CbFindMarshal(Frame->Msg);
                        if (Cbm && Extra >= Cbm->Size64 &&
                            Len32 >= sizeof(*A32) + Cbm->Size32)
                        {
                            Cbm->Up(P32, P64);
                            if (Wow64TraceSyscalls)
                                DbgPrint("[WOW64CB] msg 0x%lx OUT marshalled back (gen)\n",
                                         (unsigned long)Frame->Msg);
                        }
                        break;
                    }
                }

                if (Wow64TraceSyscalls)
                    DbgPrint("[WOW64CB] <- WINDOWPROC msg 0x%lx result=%08lx%08lx retlen=%lu "
                             "(guest ret32: proc=%08lx ansi=%lu wnd=%08lx msg=%04lx lpbs=%ld len32=%lu)\n",
                             (unsigned long)Frame->Msg,
                             (ULONG)(A64->Result >> 32), (ULONG)A64->Result, N,
                             A32->Proc, A32->IsAnsiProc, A32->Wnd, A32->Msg,
                             (long)A32->lParamBufferSize, Len32);
                Frame->RetBuf = A64;
                Frame->RetLen = N;
                break;
            }

            case 8: /* CLIENTLOADLIBRARY: guest user32 returns 4 bytes
                     * (BOOL bResult declared sizeof(HINSTANCE)); win32k reads
                     * ResultPointer as an 8-byte HMODULE — widen zero-extended
                     * so the high half is never stale scratch. */
            {
                ULONGLONG R = (Len32 >= sizeof(ULONG))
                                  ? *(const ULONG *)(ULONG_PTR)Result32 : 0;
                ULONG i;
                for (i = 0; i < sizeof(R); i++)
                    Frame->Buf64[i] = ((const BYTE *)&R)[i];
                Frame->RetBuf = Frame->Buf64;
                Frame->RetLen = sizeof(R);
                break;
            }

            default: /* flat pass-through (only safe for pointer-free blobs) */
            {
                ULONG N = Len32;
                ULONG i;
                if (N > sizeof(Frame->Buf64))
                    N = sizeof(Frame->Buf64);
                for (i = 0; i < N; i++)
                    Frame->Buf64[i] = ((const BYTE *)(ULONG_PTR)Result32)[i];
                Frame->RetBuf = Frame->Buf64;
                Frame->RetLen = N;
                break;
            }
        }
    }

    Wow64pLongJmp(Frame->Jmp, 1);
    return 1; /* unreachable */
}

/* ---- the 20-entry 64-bit KernelCallbackTable ------------------------------ */
#define WOW64_CB_STUB(n) \
    static NTSTATUS Wow64pCb##n(PVOID a, ULONG l) { return Wow64pUserCallbackCommon(n, a, l); }
WOW64_CB_STUB(0)  WOW64_CB_STUB(1)  WOW64_CB_STUB(2)  WOW64_CB_STUB(3)
WOW64_CB_STUB(4)  WOW64_CB_STUB(5)  WOW64_CB_STUB(6)  WOW64_CB_STUB(7)
WOW64_CB_STUB(8)  WOW64_CB_STUB(9)  WOW64_CB_STUB(10) WOW64_CB_STUB(11)
WOW64_CB_STUB(12) WOW64_CB_STUB(13) WOW64_CB_STUB(14) WOW64_CB_STUB(15)
WOW64_CB_STUB(16) WOW64_CB_STUB(17) WOW64_CB_STUB(18) WOW64_CB_STUB(19)
#undef WOW64_CB_STUB

static PVOID Wow64pUserCallbackTable[20] = {
    Wow64pCb0,  Wow64pCb1,  Wow64pCb2,  Wow64pCb3,  Wow64pCb4,
    Wow64pCb5,  Wow64pCb6,  Wow64pCb7,  Wow64pCb8,  Wow64pCb9,
    Wow64pCb10, Wow64pCb11, Wow64pCb12, Wow64pCb13, Wow64pCb14,
    Wow64pCb15, Wow64pCb16, Wow64pCb17, Wow64pCb18, Wow64pCb19,
};

/* Called from Wow64LdrpInitialize once ntdll32 is mapped + CPU layer up. */
void Wow64pInstallUserCallbackTable(void)
{
    PBYTE Teb = (PBYTE)NtCurrentTeb();
    PBYTE Peb = *(PBYTE *)(Teb + WOW64_TEB_PEB_OFFSET);
    *(PVOID *)(Peb + WOW64_PEB_KERNELCALLBACKTABLE) = Wow64pUserCallbackTable;
    DbgPrint("[WOW64CB] PEB64 KernelCallbackTable -> wow64 bridge (%p), guest dispatcher %p\n",
             (PVOID)Wow64pUserCallbackTable, Wow64pKiUserCallbackDispatcher32);
}
