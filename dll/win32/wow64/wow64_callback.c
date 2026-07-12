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

#define WOW64_TEB_TLSSLOTS_OFFSET 0x1480
#define WOW64_TLS_CBFRAME 12   /* FEX WOW64 layer uses 15..18; wow64.c uses 1,10 */
#define WOW64_TEB_PEB_OFFSET 0x60
#define WOW64_PEB_KERNELCALLBACKTABLE 0x58

static PVOID *Wow64pCbTlsSlot(VOID)
{
    return (PVOID *)((PBYTE)NtCurrentTeb() + WOW64_TEB_TLSSLOTS_OFFSET
                     + WOW64_TLS_CBFRAME * sizeof(PVOID)) ;
}

/* ---- minimal aarch64 setjmp/longjmp (ntdll-only DLL: no CRT import) ------
 * Saves x19-x28, fp, lr, sp, d8-d15 -> 21 slots. Non-volatile only; exactly
 * what a longjmp from a deeper frame on the SAME host stack needs. */
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
    NTSTATUS Status;         /* filled by the NtCallbackReturn marshaller */
    PVOID RetBuf;            /* 64-bit result to hand to the real NtCallbackReturn */
    ULONG RetLen;
    WOW64_CB_CONTEXT SavedCtx;
    unsigned long long Jmp[21];
    UCHAR Buf64[1024];       /* translated 64-bit result storage */
} WOW64_CB_FRAME;

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
        DbgPrint("[WOW64CB] -> callback %lu (arg=%p len=%lu)\n", Index, Arg, Len);

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
                A32->Proc = (ULONG)A64->Proc;
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
                    Wow64pEnterGuestCallback(&Frame, Index, Buf,
                                             sizeof(*A32) + 48 + Blob);
                    break;
                }
                if (Extra)
                {
                    ULONG i;
                    if (Wow64TraceSyscalls)
                        DbgPrint("[WOW64CB] WINDOWPROC msg %lx payload %lu RAW (no 64->32 repack)\n",
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
        return NtCallbackReturn(NULL, 0, WOW64_STATUS_NOT_IMPLEMENTED);
    }

    /* longjmp landing: guest completed via NtCallbackReturn marshaller. */
    *Wow64pCbTlsSlot() = Frame.Prev;
    Frame.SavedCtx.ContextFlags = WOW64_CB_CONTEXT_FULL | WOW64_CB_CONTEXT_EXTENDED;
    Wow64pBTCpuSetContext((HANDLE)(LONG_PTR)-2, WOW64_CURRENT_PROCESS, NULL,
                          &Frame.SavedCtx);
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
                     * (NCCREATE/CREATE) the payload is effectively IN-only). */
            {
                const WOW64_WNDPROC_CB32 *A32 =
                    (const WOW64_WNDPROC_CB32 *)(ULONG_PTR)Result32;
                WOW64_WNDPROC_CB64 *A64 = (WOW64_WNDPROC_CB64 *)Frame->Buf64;
                ULONG N = Frame->ArgLen64, i;
                if (N > sizeof(Frame->Buf64))
                    N = sizeof(Frame->Buf64);
                for (i = 0; i < N; i++)
                    Frame->Buf64[i] = ((const BYTE *)Frame->OrigArg)[i];
                A64->Result = A32->Result;
                Frame->RetBuf = A64;
                Frame->RetLen = N;
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
