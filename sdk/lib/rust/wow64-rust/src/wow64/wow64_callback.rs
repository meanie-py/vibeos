use crate::wow64::wow64_cb_automarshal::LONG;
use crate::wow64::wow64_cb_automarshal::LONGLONG;
use crate::wow64::wow64_cb_automarshal::ULONG;
use crate::wow64::wow64_cb_automarshal::Wow64CbFindMarshal;
use crate::wow64::wow64::Wow64LastErrorToGuest;
use crate::wow64::wow64::Wow64LastErrorToHost;
use crate::wow64::wow64::Wow64pGuestModule32;
use crate::wow64::wow64::Wow64pKiUserApcDispatcher32;
use crate::wow64::wow64::Wow64pKiUserCallbackDispatcher32;
use crate::wow64::wow64_pfnmap::Wow64pMapClientProc;
use crate::wow64::wow64_cb_automarshal::WowCbDown__WINDOWPOS;
use crate::wow64::wow64_cb_automarshal::WowCbDown_tagCOPYDATASTRUCT;
use crate::wow64::wow64_cb_automarshal::WowCbDown_tagNCCALCSIZE_PARAMS;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0AC2808DA411: [u16; 11] = [
    0x0075u16, 0x0073, 0x0065, 0x0072, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C, 0x006C,
    0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

// ---- the 20-entry 64-bit KernelCallbackTable's stubs (hand: the C makes them by token
// pasting, `WOW64_CB_STUB(n)`, so translate never saw a function to buy) ----
macro_rules! wow64_cb_stub {
    ($($name:ident = $n:literal),* $(,)?) => {$(
        unsafe extern "C" fn $name(a: PVOID, l: ULONG) -> NTSTATUS {
            Wow64pUserCallbackCommon($n, a, l)
        }
    )*};
}
wow64_cb_stub!(
    Wow64pCb0 = 0, Wow64pCb1 = 1, Wow64pCb2 = 2, Wow64pCb3 = 3, Wow64pCb4 = 4,
    Wow64pCb5 = 5, Wow64pCb6 = 6, Wow64pCb7 = 7, Wow64pCb8 = 8, Wow64pCb9 = 9,
    Wow64pCb10 = 10, Wow64pCb11 = 11, Wow64pCb12 = 12, Wow64pCb13 = 13, Wow64pCb14 = 14,
    Wow64pCb15 = 15, Wow64pCb16 = 16, Wow64pCb17 = 17, Wow64pCb18 = 18, Wow64pCb19 = 19,
);

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut Wow64pUserCallbackTable: [PVOID; 20] = unsafe {
    [
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb0 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb1 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb2 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb3 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb4 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb5 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb6 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb7 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb8 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb9 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb10 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb11 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb12 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb13 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb14 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb15 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb16 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb17 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb18 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
        ::core::mem::transmute::<Option<unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS>, PVOID>(
            Some(Wow64pCb19 as unsafe extern "C" fn(PVOID, ULONG) -> NTSTATUS),
        ),
    ]
};

pub type NTSTATUS = LONG;
pub const WOW64_CURRENT_PROCESS: HANDLE = (-1isize) as HANDLE;
pub const WOW64_STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS;

// C declares these `extern` and wow64.c DEFINES them; a second Rust definition here was a
// private copy that stayed None forever (NtContinue returned STATUS_INVALID_PARAMETER and
// every i386 guest died after its loader). Share wow64.rs's.
use crate::wow64::wow64::{Wow64pBTCpuGetContext, Wow64pBTCpuSetContext, Wow64pBTCpuSimulate};

pub const WOW64_TEB_TLSSLOTS_OFFSET: usize = 0x1480;
pub const WOW64_TLS_CBFRAME: usize = 12; /* FEX WOW64 layer uses 15..18; wow64.c uses 1,10 */
pub const WOW64_TEB_PEB_OFFSET: usize = 0x60;
pub const WOW64_PEB_KERNELCALLBACKTABLE: usize = 0x58;

#[cfg(target_arch = "x86_64")]
pub const WOW64_TLS_CPURESERVED_SLOT: usize = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WOW64_CB_CONTEXT {
    pub ContextFlags: ULONG,
    pub Dr0: ULONG,
    pub Dr1: ULONG,
    pub Dr2: ULONG,
    pub Dr3: ULONG,
    pub Dr6: ULONG,
    pub Dr7: ULONG,
    pub FloatSave: [UCHAR; 112],
    pub SegGs: ULONG,
    pub SegFs: ULONG,
    pub SegEs: ULONG,
    pub SegDs: ULONG,
    pub Edi: ULONG,
    pub Esi: ULONG,
    pub Ebx: ULONG,
    pub Edx: ULONG,
    pub Ecx: ULONG,
    pub Eax: ULONG,
    pub Ebp: ULONG,
    pub Eip: ULONG,
    pub SegCs: ULONG,
    pub EFlags: ULONG,
    pub Esp: ULONG,
    pub SegSs: ULONG,
    pub ExtendedRegisters: [UCHAR; 512],
}

pub const WOW64_CB_CONTEXT_FULL: ULONG = 0x00010007;
pub const WOW64_CB_CONTEXT_EXTENDED: ULONG = 0x00010020;

#[repr(C)]
pub struct WOW64_DDECB64 {
    pub Type: INT,
    pub _pad0: ULONG,
    pub hwnd: ULONGLONG,
    pub message: ULONG,
    pub _pad1: ULONG,
    pub wParam: ULONGLONG,
    pub lParam: ULONGLONG,
    pub time: ULONG,
    pub x: LONG,
    pub y: LONG,
    pub _pad2: ULONG,
    pub size: INT,
    pub _pad3: ULONG,
    pub pvData: ULONGLONG,
}

#[repr(C)]
pub struct WOW64_DDECB32 {
    pub Type: INT,
    pub hwnd: ULONG,
    pub message: ULONG,
    pub wParam: ULONG,
    pub lParam: ULONG,
    pub time: ULONG,
    pub x: LONG,
    pub y: LONG,
    pub size: INT,
    pub pvData: ULONG,
}

pub const WOW64_DDECB64_HDR: usize = 72; /* offsetof(buffer), host */
pub const WOW64_DDECB32_HDR: usize = 40; /* offsetof(buffer), guest */

const _: () = assert!(core::mem::size_of::<WOW64_DDECB64>() == WOW64_DDECB64_HDR);
const _: () = assert!(core::mem::size_of::<WOW64_DDECB32>() == WOW64_DDECB32_HDR);

#[repr(C)]
pub struct _WOW64_CB_FRAME {
    pub Prev: *mut _WOW64_CB_FRAME,
    pub Index: ULONG,
    pub OrigArg: PVOID,           /* the kernel's 64-bit arg buffer (stays mapped) */
    pub ArgLen64: ULONG,          /* original 64-bit arg length (for result sizing) */
    pub Msg: ULONG,               /* WINDOWPROC message — the payload repack keys off it */
    pub Status: NTSTATUS,         /* filled by the NtCallbackReturn marshaller */
    pub RetBuf: PVOID,            /* 64-bit result to hand to the real NtCallbackReturn */
    pub RetLen: ULONG,
    pub SavedCtx: WOW64_CB_CONTEXT,
    pub SavedDispatchRsp: ULONGLONG, /* amd64: CpuArea+8, clobbered by nesting */
    /* 21 slots on arm64, 30 on amd64 (10 nonvol XMMs); size for the larger. */
    pub Jmp: [ULONGLONG; 32],
    pub Buf64: [UCHAR; 1024],       /* translated 64-bit result storage */
}
pub type WOW64_CB_FRAME = _WOW64_CB_FRAME;

#[repr(C)]
pub struct _WOW64_WINDOWPOS64 {
    pub hwnd: ULONGLONG,
    pub hwndInsertAfter: ULONGLONG,
    pub x: LONG,
    pub y: LONG,
    pub cx: LONG,
    pub cy: LONG,
    pub flags: ULONG,
    pub _pad: ULONG,
}
pub type WOW64_WINDOWPOS64 = _WOW64_WINDOWPOS64;

#[repr(C)]
pub struct _WOW64_WINDOWPOS32 {
    pub hwnd: ULONG,
    pub hwndInsertAfter: ULONG,
    pub x: LONG,
    pub y: LONG,
    pub cx: LONG,
    pub cy: LONG,
    pub flags: ULONG,
}
pub type WOW64_WINDOWPOS32 = _WOW64_WINDOWPOS32;

pub const WOW64_WM_GETTEXT: ULONG = 0x000D;
pub const WOW64_WM_COPYDATA: ULONG = 0x004A;
pub const WOW64_WM_GETMINMAXINFO: ULONG = 0x0024;
pub const WOW64_WM_WINDOWPOSCHANGING: ULONG = 0x0046;
pub const WOW64_WM_WINDOWPOSCHANGED: ULONG = 0x0047;
pub const WOW64_WM_STYLECHANGING: ULONG = 0x007C;
pub const WOW64_WM_NCCALCSIZE: ULONG = 0x0083;
pub const WOW64_WM_SIZING: ULONG = 0x0214;
pub const WOW64_WM_MOVING: ULONG = 0x0216;

#[repr(C)]
pub struct _WOW64_WNDPROC_CB64 {
    pub Proc: ULONGLONG,
    pub IsAnsiProc: ULONG,
    pub _pad0: ULONG,
    pub Wnd: ULONGLONG,
    pub Msg: ULONG,
    pub _pad1: ULONG,
    pub wParam: ULONGLONG,
    pub lParam: ULONGLONG,
    pub lParamBufferSize: LONG,
    pub _pad2: ULONG,
    pub Result: ULONGLONG,
}
pub type WOW64_WNDPROC_CB64 = _WOW64_WNDPROC_CB64;

#[repr(C)]
pub struct _WOW64_WNDPROC_CB32 {
    pub Proc: ULONG,
    pub IsAnsiProc: ULONG,
    pub Wnd: ULONG,
    pub Msg: ULONG,
    pub wParam: ULONG,
    pub lParam: ULONG,
    pub lParamBufferSize: LONG,
    pub Result: ULONG,
}
pub type WOW64_WNDPROC_CB32 = _WOW64_WNDPROC_CB32;

pub const WOW64_TLS_INCPUSIMULATION: usize = 2; /* wow64.c: set once the guest owns the thread */
pub const WOW64_TLS_APCLIST: usize = 7; /* Windows' slot name; the chain below */

#[repr(C)]
pub struct _WOW64_APC_FRAME {
    pub Prev: *mut _WOW64_APC_FRAME,
    pub GuestContext: ULONG,          /* the CONTEXT the guest dispatcher NtContinues to */
    pub Native: PVOID,                /* the CONTEXT the kernel delivered the APC with */
    pub Saved: WOW64_CB_CONTEXT,      /* the guest as the APC found it */
    pub SavedDispatchRsp: ULONGLONG,  /* amd64: CpuArea+8, clobbered by nesting */
}
pub type WOW64_APC_FRAME = _WOW64_APC_FRAME;
unsafe extern "C" fn Wow64pCbTlsSlot() -> *mut PVOID {
    return (NtCurrentTeb() as PBYTE).add(
        (WOW64_TEB_TLSSLOTS_OFFSET as usize)
            + (WOW64_TLS_CBFRAME as usize) * core::mem::size_of::<PVOID>(),
    ) as *mut PVOID;
}

#[cfg(target_arch = "x86_64")] // `#ifdef _M_AMD64` in the C
unsafe extern "C" fn Wow64pClearCpuResumeFlag() {
    let Tls: *mut PBYTE = (NtCurrentTeb() as PBYTE).offset(WOW64_TEB_TLSSLOTS_OFFSET as isize) as *mut PBYTE;
    let CpuArea: PBYTE = *Tls.offset(WOW64_TLS_CPURESERVED_SLOT as isize);
    if !CpuArea.is_null() {
        core::ptr::write_volatile(
            (CpuArea.offset(4 as isize)) as *mut ULONG,
            0 as ULONG,
        );
    }
}

#[cfg(target_arch = "x86_64")] // `#ifdef _M_AMD64` in the C
unsafe extern "C" fn Wow64pCpuDispatchRspSlot() -> *mut ULONGLONG {
    let Tls: *mut PBYTE = (NtCurrentTeb() as PBYTE).add(WOW64_TEB_TLSSLOTS_OFFSET) as *mut PBYTE;
    let CpuArea: PBYTE = *Tls.add(WOW64_TLS_CPURESERVED_SLOT);
    return if !CpuArea.is_null() {
        CpuArea.add(8) as *mut ULONGLONG
    } else {
        core::ptr::null_mut()
    };
}
// Minimal setjmp/longjmp (ntdll-only DLL: no CRT import). Hand-ported from the C's two
// `__attribute__((naked))` arms; the translation kept only the arm64 one, with no cfg.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn Wow64pSetJmp(Buf: *mut u64) -> i32 {
    core::arch::naked_asm!(
        "stp x19, x20, [x0, #0]",
        "stp x21, x22, [x0, #16]",
        "stp x23, x24, [x0, #32]",
        "stp x25, x26, [x0, #48]",
        "stp x27, x28, [x0, #64]",
        "stp x29, x30, [x0, #80]",
        "mov x1, sp",
        "str x1, [x0, #96]",
        "stp d8,  d9,  [x0, #104]",
        "stp d10, d11, [x0, #120]",
        "stp d12, d13, [x0, #136]",
        "stp d14, d15, [x0, #152]",
        "mov w0, #0",
        "ret",
    )
}
#[cfg(target_arch = "aarch64")]
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn Wow64pLongJmp(Buf: *mut ::core::ffi::c_ulonglong, Val: ::core::ffi::c_int) -> ! {
    ::core::arch::naked_asm!(
        "ldp x19, x20, [x0, #0]",
        "ldp x21, x22, [x0, #16]",
        "ldp x23, x24, [x0, #32]",
        "ldp x25, x26, [x0, #48]",
        "ldp x27, x28, [x0, #64]",
        "ldp x29, x30, [x0, #80]",
        "ldr x2, [x0, #96]",
        "mov sp, x2",
        "ldp d8,  d9,  [x0, #104]",
        "ldp d10, d11, [x0, #120]",
        "ldp d12, d13, [x0, #136]",
        "ldp d14, d15, [x0, #152]",
        "mov w0, w1",
        "cmp w0, #0",
        "csinc w0, w0, wzr, ne",
        "ret",
    )
}
// _M_AMD64, Windows x64 ABI: Buf in rcx, Val in rdx. Non-volatile GPRs at 0..63, caller
// rsp at 64, return address at 72, xmm6-xmm15 at 80..239 (movups: any alignment).
#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn Wow64pSetJmp(Buf: *mut u64) -> i32 {
    core::arch::naked_asm!(
        "movq %rbx, 0(%rcx)",
        "movq %rbp, 8(%rcx)",
        "movq %rdi, 16(%rcx)",
        "movq %rsi, 24(%rcx)",
        "movq %r12, 32(%rcx)",
        "movq %r13, 40(%rcx)",
        "movq %r14, 48(%rcx)",
        "movq %r15, 56(%rcx)",
        "leaq 8(%rsp), %rax", // caller's rsp (after the call's push)
        "movq %rax, 64(%rcx)",
        "movq (%rsp), %rax", // return address
        "movq %rax, 72(%rcx)",
        "movups %xmm6,  80(%rcx)",
        "movups %xmm7,  96(%rcx)",
        "movups %xmm8,  112(%rcx)",
        "movups %xmm9,  128(%rcx)",
        "movups %xmm10, 144(%rcx)",
        "movups %xmm11, 160(%rcx)",
        "movups %xmm12, 176(%rcx)",
        "movups %xmm13, 192(%rcx)",
        "movups %xmm14, 208(%rcx)",
        "movups %xmm15, 224(%rcx)",
        "xorl %eax, %eax",
        "ret",
        options(att_syntax)
    )
}
#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn Wow64pLongJmp(Buf: *mut ::core::ffi::c_ulonglong, Val: ::core::ffi::c_int) -> ! {
    ::core::arch::naked_asm!(
        "movq 0(%rcx), %rbx",
        "movq 8(%rcx), %rbp",
        "movq 16(%rcx), %rdi",
        "movq 24(%rcx), %rsi",
        "movq 32(%rcx), %r12",
        "movq 40(%rcx), %r13",
        "movq 48(%rcx), %r14",
        "movq 56(%rcx), %r15",
        "movups 80(%rcx),  %xmm6",
        "movups 96(%rcx),  %xmm7",
        "movups 112(%rcx), %xmm8",
        "movups 128(%rcx), %xmm9",
        "movups 144(%rcx), %xmm10",
        "movups 160(%rcx), %xmm11",
        "movups 176(%rcx), %xmm12",
        "movups 192(%rcx), %xmm13",
        "movups 208(%rcx), %xmm14",
        "movups 224(%rcx), %xmm15",
        "movl %edx, %eax", // return Val (or 1 if Val==0)
        "testl %eax, %eax",
        "jne 2f", // the C's label is `1:`; rustc rejects 0/1-only labels on x86
        "incl %eax",
        "2:",
        "movq 72(%rcx), %r8", // retaddr into a volatile scratch
        "movq 64(%rcx), %rsp", // switch back to the caller's stack
        "jmp *%r8",
        options(att_syntax)
    )
}

unsafe extern "C" fn Wow64pEnterGuestCallback(
    Frame: *mut WOW64_CB_FRAME,
    Index: ULONG,
    Arg32: *const ::core::ffi::c_void,
    Len32: ULONG,
) {
    let mut Ctx: WOW64_CB_CONTEXT = core::mem::zeroed();
    let mut Esp: ULONG = 0;
    let mut ArgGuest: ULONG = 0 as ULONG;

    Ctx = (*Frame).SavedCtx;
    Esp = Ctx.Esp;

    if !Arg32.is_null() && Len32 != 0 {
        Esp = Esp
            .wrapping_sub(Len32)
            .wrapping_sub(64 as ULONG)
            & !(15 as ULONG);
        ArgGuest = Esp;
        {
            let mut i: ULONG = 0;
            i = 0 as ULONG;
            while i < Len32 {
                *((Esp as ULONG_PTR) as PBYTE).offset(i as isize) =
                    *(Arg32 as *const BYTE).offset(i as isize);
                i = i.wrapping_add(1);
            }
        }
    } else {
        Esp = Esp.wrapping_sub(64 as ULONG)
            & !(15 as ULONG);
    }

    Esp = Esp.wrapping_sub(24 as ULONG);
    *((Esp.wrapping_add(0 as ULONG) as ULONG_PTR) as *mut ULONG) =
        0 as ULONG;
    *((Esp.wrapping_add(4 as ULONG) as ULONG_PTR) as *mut ULONG) = Index;
    *((Esp.wrapping_add(8 as ULONG) as ULONG_PTR) as *mut ULONG) = ArgGuest;
    *((Esp.wrapping_add(12 as ULONG) as ULONG_PTR) as *mut ULONG) = Len32;
    *((Esp.wrapping_add(16 as ULONG) as ULONG_PTR) as *mut ULONG) =
        0 as ULONG;
    *((Esp.wrapping_add(20 as ULONG) as ULONG_PTR) as *mut ULONG) =
        0 as ULONG;

    Ctx.Eip = Wow64pKiUserCallbackDispatcher32 as ULONG_PTR as ULONG;
    Ctx.Esp = Esp;
    Ctx.ContextFlags = (WOW64_CB_CONTEXT_FULL | WOW64_CB_CONTEXT_EXTENDED) as ULONG;

    (Wow64pBTCpuSetContext.unwrap_unchecked())(
        (-(2 as ::core::ffi::c_int) as LONG_PTR as HANDLE),
        WOW64_CURRENT_PROCESS,
        NULL,
        &raw mut Ctx as PVOID,
    );

    /* Control is crossing into the guest, so last-error has to cross with it:
     * from here until NtCallbackReturn, GetLastError() means TEB32. The
     * matching copy back up is at the longjmp landing in
     * Wow64pUserCallbackCommon. */
    Wow64LastErrorToGuest();

    (Wow64pBTCpuSimulate.unwrap_unchecked())(); /* exits via guest NtCallbackReturn -> longjmp */

    DbgPrint(
        b"[WOW64CB] FATAL: nested BTCpuSimulate returned (idx %lu)\n\0".as_ptr() as PCSTR,
        Index,
    );
    NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBADCB);
}

unsafe extern "C" fn Wow64pUserCallbackCommon(Index: ULONG, Arg: PVOID, Len: ULONG) -> NTSTATUS {
    let mut Frame: WOW64_CB_FRAME = core::mem::zeroed();
    let mut St: NTSTATUS = 0;

    if Wow64TraceSyscalls != 0 {
        let mut Depth: ULONG = 0;
        let mut W: *const WOW64_CB_FRAME = *Wow64pCbTlsSlot() as *const WOW64_CB_FRAME;
        while !W.is_null() && Depth < 10000 {
            Depth = Depth.wrapping_add(1);
            W = (*W).Prev as *const WOW64_CB_FRAME;
        }
        DbgPrint(
            b"[WOW64CB] -> callback %lu (arg=%p len=%lu sp=%p depth=%lu)\n\0" as *const u8 as PCSTR,
            Index,
            Arg,
            Len,
            core::ptr::addr_of_mut!(Frame) as PVOID,
            Depth,
        );
    }

    if Wow64pKiUserCallbackDispatcher32.is_null()
        || Wow64pBTCpuGetContext.is_none()
        || Wow64pBTCpuSetContext.is_none()
        || Wow64pBTCpuSimulate.is_none()
    {
        DbgPrint(
            b"[WOW64CB] callback %lu with no guest dispatcher \xE2\x80\x94 failing\n\0" as *const u8 as PCSTR,
            Index,
        );
        return NtCallbackReturn(core::ptr::null_mut(), 0, WOW64_STATUS_NOT_IMPLEMENTED);
    }

    Frame.Prev = *Wow64pCbTlsSlot() as *mut WOW64_CB_FRAME;
    Frame.Index = Index;
    Frame.OrigArg = Arg;
    Frame.ArgLen64 = Len;
    Frame.Status = WOW64_STATUS_NOT_IMPLEMENTED;
    Frame.RetBuf = core::ptr::null_mut();
    Frame.RetLen = 0;
    Frame.SavedCtx.ContextFlags = WOW64_CB_CONTEXT_FULL | WOW64_CB_CONTEXT_EXTENDED;
    (Wow64pBTCpuGetContext.unwrap_unchecked())(
        (-2 as LONG_PTR) as HANDLE,
        WOW64_CURRENT_PROCESS,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Frame.SavedCtx) as PVOID,
    );
    Frame.SavedDispatchRsp = 0;
    #[cfg(target_arch = "x86_64")]
    {
        let Slot: *mut ULONGLONG = Wow64pCpuDispatchRspSlot();
        if !Slot.is_null() {
            Frame.SavedDispatchRsp = *Slot;
        }
    }
    *Wow64pCbTlsSlot() = core::ptr::addr_of_mut!(Frame) as PVOID;

    if Wow64pSetJmp(core::ptr::addr_of_mut!(Frame.Jmp) as *mut u64) == 0 {
        'sw: {
            match Index {
                0 => {
                    let A64: *const WOW64_WNDPROC_CB64 =
                        Arg as *const WOW64_WNDPROC_CB64;
                    let Extra: ULONG = if Len as usize > core::mem::size_of::<WOW64_WNDPROC_CB64>() {
                        (Len as usize).wrapping_sub(core::mem::size_of::<WOW64_WNDPROC_CB64>()) as ULONG
                    } else {
                        0
                    };
                    if Wow64TraceSyscalls != 0 {
                        DbgPrint(
                            b"[WOW64CB] WINDOWPROC hwnd=%p msg=%04lx wp=%p lp=%p extra=%lu\n\0" as *const u8 as PCSTR,
                            (*A64).Wnd as ULONG_PTR as PVOID,
                            (*A64).Msg,
                            (*A64).wParam as ULONG_PTR as PVOID,
                            (*A64).lParam as ULONG_PTR as PVOID,
                            Extra,
                        );
                    }
                    let mut Buf: *mut UCHAR = core::ptr::addr_of_mut!(Frame.Buf64) as *mut UCHAR;
                    let A32: *mut WOW64_WNDPROC_CB32 = Buf as *mut WOW64_WNDPROC_CB32;
                    if Extra > 512 {
                        DbgPrint(
                            b"[WOW64CB] WINDOWPROC extra %lu too big\n\0" as *const u8 as PCSTR,
                            Extra,
                        );
                        break 'sw;
                    }
                    Frame.Msg = (*A64).Msg;
                    (*A32).Proc = Wow64pMapClientProc((*A64).Proc) as ULONG;
                    (*A32).IsAnsiProc = (*A64).IsAnsiProc;
                    (*A32).Wnd = (*A64).Wnd as ULONG;
                    (*A32).Msg = (*A64).Msg;
                    (*A32).wParam = (*A64).wParam as ULONG;
                    (*A32).lParam = (*A64).lParam as ULONG;
                    (*A32).lParamBufferSize = (*A64).lParamBufferSize;
                    (*A32).Result = 0 as ULONG;
                    if Extra >= 80 as ULONG
                        && ((*A64).Msg == 0x0081
                            || (*A64).Msg == 0x0001)
                    {
                        let C64: *const ULONGLONG = (Arg as *const BYTE)
                            .offset(core::mem::size_of::<WOW64_WNDPROC_CB64>() as isize)
                            as *const ULONGLONG;
                        let C64d: *const ULONG = C64 as *const ULONG;
                        let C32: *mut ULONG =
                            Buf.offset(core::mem::size_of::<WOW64_WNDPROC_CB32>() as isize) as *mut ULONG;
                        let mut Blob: ULONG = Extra.wrapping_sub(80 as ULONG);
                        let mut i: ULONG = 0;
                        *C32.offset(0 as isize) =
                            *C64.offset(0 as isize) as ULONG;
                        *C32.offset(1 as isize) =
                            *C64.offset(1 as isize) as ULONG;
                        *C32.offset(2 as isize) =
                            *C64.offset(2 as isize) as ULONG;
                        *C32.offset(3 as isize) =
                            *C64.offset(3 as isize) as ULONG;
                        *C32.offset(4 as isize) =
                            *C64d.offset(8 as isize);
                        *C32.offset(5 as isize) =
                            *C64d.offset(9 as isize);
                        *C32.offset(6 as isize) =
                            *C64d.offset(10 as isize);
                        *C32.offset(7 as isize) =
                            *C64d.offset(11 as isize);
                        *C32.offset(8 as isize) =
                            *C64d.offset(12 as isize);
                        *C32.offset(9 as isize) =
                            if *C64.offset(7 as isize) != 0 {
                                (*C64.offset(7 as isize) as ULONG)
                                    .wrapping_sub(32 as ULONG)
                            } else {
                                0 as ULONG
                            };
                        *C32.offset(10 as isize) =
                            if *C64.offset(8 as isize) != 0 {
                                (*C64.offset(8 as isize) as ULONG)
                                    .wrapping_sub(32 as ULONG)
                            } else {
                                0 as ULONG
                            };
                        *C32.offset(11 as isize) =
                            *C64d.offset(18 as isize);
                        i = 0 as ULONG;
                        while i < Blob {
                            *Buf.offset(
                                (48 as usize)
                                    .wrapping_add(core::mem::size_of::<WOW64_WNDPROC_CB32>())
                                    .wrapping_add(i as usize) as isize,
                            ) = *(Arg as *const BYTE).offset(
                                (80 as usize)
                                    .wrapping_add(core::mem::size_of::<WOW64_WNDPROC_CB64>())
                                    .wrapping_add(i as usize) as isize,
                            );
                            i = i.wrapping_add(1);
                        }
                        (*A32).lParamBufferSize =
                            (48 as ULONG).wrapping_add(Blob) as LONG;
                        if Wow64TraceSyscalls != 0 {
                            DbgPrint(
                                b"[WOW64CB] (NC)CREATE cs64 name=%08lx%08lx cls=%08lx%08lx blob=%lu -> cs32 name=%08lx cls=%08lx\n\0"
                                    as *const u8 as PCSTR,
                                ((*C64.offset(7 as isize))
                                    >> 32) as ULONG,
                                *C64.offset(7 as isize) as ULONG,
                                ((*C64.offset(8 as isize))
                                    >> 32) as ULONG,
                                *C64.offset(8 as isize) as ULONG,
                                Blob,
                                *C32.offset(9 as isize),
                                *C32.offset(10 as isize),
                            );
                        }
                        Wow64pEnterGuestCallback(
                            core::ptr::addr_of_mut!(Frame),
                            Index,
                            Buf as *const ::core::ffi::c_void,
                            (core::mem::size_of::<WOW64_WNDPROC_CB32>() as usize)
                                .wrapping_add(48 as usize)
                                .wrapping_add(Blob as usize) as ULONG,
                        );
                        break 'sw;
                    }
                    if ((*A64).Msg == WOW64_WM_WINDOWPOSCHANGING
                        || (*A64).Msg == WOW64_WM_WINDOWPOSCHANGED)
                        && Extra as usize >= core::mem::size_of::<WOW64_WINDOWPOS64>()
                    {
                        let P64: *const WOW64_WINDOWPOS64 = (Arg as *const BYTE)
                            .offset(core::mem::size_of::<WOW64_WNDPROC_CB64>() as isize)
                            as *const WOW64_WINDOWPOS64;
                        let P32: *mut WOW64_WINDOWPOS32 =
                            Buf.offset(core::mem::size_of::<WOW64_WNDPROC_CB32>() as isize)
                                as *mut WOW64_WINDOWPOS32;
                        (*P32).hwnd = (*P64).hwnd as ULONG;
                        (*P32).hwndInsertAfter = (*P64).hwndInsertAfter as ULONG;
                        (*P32).x = (*P64).x;
                        (*P32).y = (*P64).y;
                        (*P32).cx = (*P64).cx;
                        (*P32).cy = (*P64).cy;
                        (*P32).flags = (*P64).flags;
                        (*A32).lParamBufferSize =
                            core::mem::size_of::<WOW64_WINDOWPOS32>() as LONG;
                        Wow64pEnterGuestCallback(
                            core::ptr::addr_of_mut!(Frame),
                            Index,
                            Buf as *const ::core::ffi::c_void,
                            (core::mem::size_of::<WOW64_WNDPROC_CB32>()
                                + core::mem::size_of::<WOW64_WINDOWPOS32>()) as ULONG,
                        );
                        break 'sw;
                    }
                    /* M18: NCCALCSIZE_PARAMS + appended WINDOWPOS (wParam != 0;
                     * wParam == 0 is a bare width-identical RECT and falls through
                     * to the verbatim path). */
                    if (*A64).Msg == WOW64_WM_NCCALCSIZE
                        && (*A64).wParam != 0
                        && Extra >= (WOW64_CB_SIZE64_tagNCCALCSIZE_PARAMS + WOW64_CB_SIZE64__WINDOWPOS) as ULONG
                    {
                        let S: *const BYTE = (Arg as *const BYTE)
                            .offset(core::mem::size_of::<WOW64_WNDPROC_CB64>() as isize);
                        let D: *mut BYTE = Buf.offset(core::mem::size_of::<WOW64_WNDPROC_CB32>() as isize);
                        WowCbDown_tagNCCALCSIZE_PARAMS(S as *const ::core::ffi::c_uchar, D as *mut ::core::ffi::c_uchar);
                        WowCbDown__WINDOWPOS(
                            S.offset(WOW64_CB_SIZE64_tagNCCALCSIZE_PARAMS as isize),
                            D.offset(WOW64_CB_SIZE32_tagNCCALCSIZE_PARAMS as isize),
                        );
                        (*A32).lParamBufferSize =
                            (WOW64_CB_SIZE32_tagNCCALCSIZE_PARAMS + WOW64_CB_SIZE32__WINDOWPOS) as LONG;
                        Wow64pEnterGuestCallback(
                            core::ptr::addr_of_mut!(Frame),
                            Index,
                            Buf as *const ::core::ffi::c_void,
                            (core::mem::size_of::<WOW64_WNDPROC_CB32>() as usize)
                                .wrapping_add((*A32).lParamBufferSize as usize) as ULONG,
                        );
                        break 'sw;
                    }
                    /* M18: COPYDATASTRUCT header + trailing data blob (READ-only;
                     * guest recomputes lpData = (cds + 1) in MsgiKMToUMMessage). */
                    if (*A64).Msg == WOW64_WM_COPYDATA
                        && Extra >= WOW64_CB_SIZE64_tagCOPYDATASTRUCT as ULONG
                    {
                        let S: *const BYTE = (Arg as *const BYTE)
                            .offset(core::mem::size_of::<WOW64_WNDPROC_CB64>() as isize);
                        let D: *mut BYTE = Buf.offset(core::mem::size_of::<WOW64_WNDPROC_CB32>() as isize);
                        let Blob: ULONG = Extra.wrapping_sub(WOW64_CB_SIZE64_tagCOPYDATASTRUCT as ULONG);
                        let mut i: ULONG = 0;
                        WowCbDown_tagCOPYDATASTRUCT(S as *const ::core::ffi::c_uchar, D as *mut ::core::ffi::c_uchar);
                        i = 0 as ULONG;
                        while i < Blob {
                            *D.offset((WOW64_CB_SIZE32_tagCOPYDATASTRUCT as ULONG).wrapping_add(i) as isize) =
                                *S.offset((WOW64_CB_SIZE64_tagCOPYDATASTRUCT as ULONG).wrapping_add(i) as isize);
                            i = i.wrapping_add(1);
                        }
                        (*A32).lParamBufferSize =
                            (WOW64_CB_SIZE32_tagCOPYDATASTRUCT as ULONG).wrapping_add(Blob) as LONG;
                        Wow64pEnterGuestCallback(
                            core::ptr::addr_of_mut!(Frame),
                            Index,
                            Buf as *const ::core::ffi::c_void,
                            (core::mem::size_of::<WOW64_WNDPROC_CB32>() as usize)
                                .wrapping_add((*A32).lParamBufferSize as usize) as ULONG,
                        );
                        break 'sw;
                    }
                    /* M18: table-driven one-struct payloads (generated:
                     * DRAWITEM / MEASUREITEM / HELP / NEXTMENU today). */
                    {
                        let Cbm: *const WOW64_CB_MARSHAL =
                            Wow64CbFindMarshal((*A64).Msg as ::core::ffi::c_uint);
                        if !Cbm.is_null() && Extra >= (*Cbm).Size64 as ULONG {
                            (*Cbm).Down.unwrap_unchecked()(
                                (Arg as *const BYTE).offset(core::mem::size_of::<WOW64_WNDPROC_CB64>() as isize),
                                Buf.offset(core::mem::size_of::<WOW64_WNDPROC_CB32>() as isize),
                            );
                            (*A32).lParamBufferSize = (*Cbm).Size32 as LONG;
                            Wow64pEnterGuestCallback(
                                core::ptr::addr_of_mut!(Frame),
                                Index,
                                Buf as *const ::core::ffi::c_void,
                                (core::mem::size_of::<WOW64_WNDPROC_CB32>() as usize)
                                    .wrapping_add((*Cbm).Size32 as usize) as ULONG,
                            );
                            break 'sw;
                        }
                    }
                    if Extra != 0 {
                        /* Everything else copies verbatim. */
                        let mut i: ULONG = 0;
                        if Wow64TraceSyscalls != 0 {
                            DbgPrint(
                                b"[WOW64CB] WINDOWPROC msg %lx payload %lu verbatim (width-identical or unclassified)\n\0"
                                    as *const u8 as PCSTR,
                                (*A64).Msg,
                                Extra,
                            );
                        }
                        i = 0 as ULONG;
                        while i < Extra {
                            *Buf.offset(
                                (core::mem::size_of::<WOW64_WNDPROC_CB32>()).wrapping_add(i as usize) as isize,
                            ) = *(Arg as *const BYTE).offset(
                                (core::mem::size_of::<WOW64_WNDPROC_CB64>()).wrapping_add(i as usize) as isize,
                            );
                            i = i.wrapping_add(1);
                        }
                    }
                    Wow64pEnterGuestCallback(
                        core::ptr::addr_of_mut!(Frame),
                        Index,
                        Buf as *const ::core::ffi::c_void,
                        (core::mem::size_of::<WOW64_WNDPROC_CB32>() as usize).wrapping_add(Extra as usize) as ULONG,
                    );
                    break 'sw;
                }

                7 => {
                    /* CLIENTTHREADSTARTUP — no argument payload */
                    Wow64pEnterGuestCallback(
                        core::ptr::addr_of_mut!(Frame),
                        Index,
                        core::ptr::null_mut(),
                        0,
                    );
                    break 'sw;
                }

                8 => {
                    /* CLIENTLOADLIBRARY: 40 -> 24 + trailing string blob. */
                    // Block-scope C structs, transcribed field for field (the C_ASSERT/comment sizes pinned).
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    struct WOW64_CLL_US64 { Len: USHORT, Max: USHORT, _pad: ULONG, Buf: ULONGLONG }
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    struct WOW64_CLL_US32 { Len: USHORT, Max: USHORT, Buf: ULONG }
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    struct WOW64_CLL64 { Lib: WOW64_CLL_US64, Init: WOW64_CLL_US64, Unload: ULONG, ApiHook: ULONG }
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    struct WOW64_CLL32 { Lib: WOW64_CLL_US32, Init: WOW64_CLL_US32, Unload: ULONG, ApiHook: ULONG }
                    const _: () = assert!(
                        core::mem::size_of::<WOW64_CLL64>() == 40 && core::mem::size_of::<WOW64_CLL32>() == 24
                    );
                    let A64: *const WOW64_CLL64 = Arg as *const WOW64_CLL64;
                    let mut Buf: *mut UCHAR = core::ptr::addr_of_mut!(Frame.Buf64) as *mut UCHAR;
                    let A32: *mut WOW64_CLL32 = Buf as *mut WOW64_CLL32;
                    let Shrink: ULONG = (core::mem::size_of::<WOW64_CLL64>()
                        .wrapping_sub(core::mem::size_of::<WOW64_CLL32>())) as ULONG;
                    let mut Blob: ULONG = 0;
                    let mut i: ULONG = 0;

                    if Arg.is_null()
                        || (Len as usize) < core::mem::size_of::<WOW64_CLL64>()
                        || (Len as usize).wrapping_sub(Shrink as usize)
                            > core::mem::size_of_val(&Frame.Buf64)
                    {
                        DbgPrint(
                            b"[WOW64CB] CLIENTLOADLIBRARY arg=%p len=%lu unusable\n\0" as *const u8 as PCSTR,
                            Arg,
                            Len,
                        );
                        break 'sw;
                    }
                    Blob = (Len as usize).wrapping_sub(core::mem::size_of::<WOW64_CLL64>()) as ULONG;
                    (*A32).Lib.Len = (*A64).Lib.Len;
                    (*A32).Lib.Max = (*A64).Lib.Max;
                    (*A32).Lib.Buf = if (*A64).Lib.Buf != 0 {
                        ((*A64).Lib.Buf as ULONG).wrapping_sub(Shrink)
                    } else {
                        0 as ULONG
                    };
                    (*A32).Init.Len = (*A64).Init.Len;
                    (*A32).Init.Max = (*A64).Init.Max;
                    (*A32).Init.Buf = if (*A64).Init.Buf != 0 {
                        ((*A64).Init.Buf as ULONG).wrapping_sub(Shrink)
                    } else {
                        0 as ULONG
                    };
                    (*A32).Unload = (*A64).Unload;
                    (*A32).ApiHook = (*A64).ApiHook;
                    i = 0 as ULONG;
                    while i < Blob {
                        *Buf.offset(
                            (core::mem::size_of::<WOW64_CLL32>()).wrapping_add(i as usize) as isize,
                        ) = *(Arg as *const UCHAR).offset(
                            (core::mem::size_of::<WOW64_CLL64>()).wrapping_add(i as usize) as isize,
                        );
                        i = i.wrapping_add(1);
                    }
                    if Wow64TraceSyscalls != 0 {
                        DbgPrint(
                            b"[WOW64CB] CLIENTLOADLIBRARY lib=%.*S unload=%lu apihook=%lu\n\0" as *const u8 as PCSTR,
                            if (*A32).Lib.Buf != 0 {
                                ((*A32).Lib.Len as usize
                                    / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int
                            } else {
                                0
                            },
                            if (*A32).Lib.Buf != 0 {
                                Buf.offset((*A32).Lib.Buf as isize) as PCWSTR
                            } else {
                                RUSTCONV_WSTR_96A296D224F2.as_ptr()
                            },
                            (*A32).Unload,
                            (*A32).ApiHook,
                        );
                    }
                    Wow64pEnterGuestCallback(
                        core::ptr::addr_of_mut!(Frame),
                        Index,
                        Buf as *const ::core::ffi::c_void,
                        (core::mem::size_of::<WOW64_CLL32>() as usize).wrapping_add(Blob as usize) as ULONG,
                    );
                    break 'sw;
                }

                10 => {
                    /* COPYIMAGE: 24 -> 20 (the HANDLE narrows). */
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    struct WOW64_COPYIMAGE64 { hImage: ULONGLONG, uType: ULONG, cx: LONG, cy: LONG, fuFlags: ULONG }
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    struct WOW64_COPYIMAGE32 { hImage: ULONG, uType: ULONG, cx: LONG, cy: LONG, fuFlags: ULONG }
                    const _: () = assert!(
                        core::mem::size_of::<WOW64_COPYIMAGE64>() == 24
                            && core::mem::size_of::<WOW64_COPYIMAGE32>() == 20
                    );
                    let A64: *const WOW64_COPYIMAGE64 =
                        Arg as *const WOW64_COPYIMAGE64;
                    let mut A32: WOW64_COPYIMAGE32 = core::mem::zeroed();

                    if Arg.is_null() || (Len as usize) < core::mem::size_of::<WOW64_COPYIMAGE64>() {
                        DbgPrint(
                            b"[WOW64CB] COPYIMAGE arg=%p len=%lu unusable\n\0" as *const u8 as PCSTR,
                            Arg,
                            Len,
                        );
                        break 'sw;
                    }
                    A32.hImage = (*A64).hImage as ULONG;
                    A32.uType = (*A64).uType;
                    A32.cx = (*A64).cx;
                    A32.cy = (*A64).cy;
                    A32.fuFlags = (*A64).fuFlags;
                    Wow64pEnterGuestCallback(
                        core::ptr::addr_of_mut!(Frame),
                        Index,
                        core::ptr::addr_of_mut!(A32) as *const ::core::ffi::c_void,
                        core::mem::size_of_val(&A32) as ULONG,
                    );
                    break 'sw;
                }

                13 | 14 => {
                    /* DDEPOST / DDEGET: 80 -> 44 with the payload repacked. */
                    let A64: *const WOW64_DDECB64 = Arg as *const WOW64_DDECB64;
                    let mut Buf: *mut UCHAR = core::ptr::addr_of_mut!(Frame.Buf64) as *mut UCHAR;
                    let A32: *mut WOW64_DDECB32 = Buf as *mut WOW64_DDECB32;
                    let mut Tail: ULONG = 0;
                    let mut i: ULONG = 0;

                    if Arg.is_null()
                        || Len < WOW64_DDECB64_HDR as ULONG
                        || (Len as usize).wrapping_sub(WOW64_DDECB64_HDR as usize)
                            > core::mem::size_of_val(&Frame.Buf64).wrapping_sub(WOW64_DDECB32_HDR as usize)
                    {
                        DbgPrint(
                            b"[WOW64CB] DDE%s arg=%p len=%lu unusable (payload over %lu)\n\0" as *const u8 as PCSTR,
                            if Index == 13 {
                                b"POST\0" as *const u8 as *const ::core::ffi::c_char
                            } else {
                                b"GET\0" as *const u8 as *const ::core::ffi::c_char
                            },
                            Arg,
                            Len,
                            (core::mem::size_of_val(&Frame.Buf64).wrapping_sub(WOW64_DDECB32_HDR as usize)) as ULONG,
                        );
                        break 'sw;
                    }
                    Tail = Len.wrapping_sub(WOW64_DDECB64_HDR as ULONG);
                    if Wow64TraceSyscalls != 0 {
                        DbgPrint(
                            b"[WOW64CB] DDE%s msg %04lx hwnd %08lx lParam %08lx size %d tail %lu\n\0" as *const u8 as PCSTR,
                            if Index == 13 {
                                b"POST\0" as *const u8 as *const ::core::ffi::c_char
                            } else {
                                b"GET\0" as *const u8 as *const ::core::ffi::c_char
                            },
                            (*A64).message,
                            (*A64).hwnd as ULONG,
                            (*A64).lParam as ULONG,
                            (*A64).size,
                            Tail,
                        );
                    }
                    (*A32).Type = (*A64).Type;
                    (*A32).hwnd = (*A64).hwnd as ULONG;
                    (*A32).message = (*A64).message;
                    (*A32).wParam = (*A64).wParam as ULONG;
                    (*A32).lParam = (*A64).lParam as ULONG;
                    (*A32).time = (*A64).time;
                    (*A32).x = (*A64).x;
                    (*A32).y = (*A64).y;
                    (*A32).size = (*A64).size;
                    (*A32).pvData = (*A64).pvData as ULONG;
                    i = 0 as ULONG;
                    while i < Tail {
                        *Buf.offset((WOW64_DDECB32_HDR as usize).wrapping_add(i as usize) as isize) =
                            *(Arg as *const UCHAR)
                                .offset((WOW64_DDECB64_HDR as usize).wrapping_add(i as usize) as isize);
                        i = i.wrapping_add(1);
                    }
                    Wow64pEnterGuestCallback(
                        core::ptr::addr_of_mut!(Frame),
                        Index,
                        Buf as *const ::core::ffi::c_void,
                        (WOW64_DDECB32_HDR as ULONG).wrapping_add(Tail),
                    );
                    break 'sw;
                }

                6 => {
                    /* LOADMENU: the name starts at 16 on the host, 8 in the guest. */
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    struct WOW64_LOADMENU64 { hModule: ULONGLONG, InterSource: ULONGLONG }
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    struct WOW64_LOADMENU32 { hModule: ULONG, InterSource: ULONG }
                    const _: () = assert!(
                        core::mem::size_of::<WOW64_LOADMENU64>() == 16 && core::mem::size_of::<WOW64_LOADMENU32>() == 8
                    );
                    C_ASSERT!(
                        core::mem::size_of::<WOW64_LOADMENU64>() == 16
                            && core::mem::size_of::<WOW64_LOADMENU32>() == 8
                    );
                    let A64: *const WOW64_LOADMENU64 =
                        Arg as *const WOW64_LOADMENU64;
                    let mut Buf: *mut UCHAR = core::ptr::addr_of_mut!(Frame.Buf64) as *mut UCHAR;
                    let A32: *mut WOW64_LOADMENU32 = Buf as *mut WOW64_LOADMENU32;
                    let mut Tail: ULONG = 0;
                    let mut i: ULONG = 0;

                    if Arg.is_null()
                        || (Len as usize) < core::mem::size_of::<WOW64_LOADMENU64>()
                        || (Len as usize).wrapping_sub(core::mem::size_of::<WOW64_LOADMENU64>())
                            > core::mem::size_of_val(&Frame.Buf64)
                                .wrapping_sub(core::mem::size_of::<WOW64_LOADMENU32>())
                    {
                        DbgPrint(
                            b"[WOW64CB] LOADMENU arg=%p len=%lu unusable\n\0" as *const u8 as PCSTR,
                            Arg,
                            Len,
                        );
                        break 'sw;
                    }
                    Tail = (Len as usize).wrapping_sub(core::mem::size_of::<WOW64_LOADMENU64>()) as ULONG;
                    /* A module above 4 GB is the HOST user32; use the guest's own. */
                    if (*A64).hModule > 0xFFFFFFFF {
                        let mut ImgSize: SIZE_T = 0;
                        let User32: PVOID = Wow64pGuestModule32(
                            RUSTCONV_WSTR_0AC2808DA411.as_ptr(),
                            core::ptr::addr_of_mut!(ImgSize),
                        );
                        if User32.is_null() {
                            DbgPrint(
                                b"[WOW64CB] LOADMENU host module %I64x, no guest user32\n\0" as *const u8 as PCSTR,
                                (*A64).hModule,
                            );
                            break 'sw;
                        }
                        (*A32).hModule = User32 as ULONG_PTR as ULONG;
                    } else {
                        (*A32).hModule = (*A64).hModule as ULONG;
                    }
                    (*A32).InterSource = (*A64).InterSource as ULONG;
                    i = 0 as ULONG;
                    while i < Tail {
                        *Buf.offset(
                            (core::mem::size_of::<WOW64_LOADMENU32>()).wrapping_add(i as usize) as isize,
                        ) = *(Arg as *const UCHAR).offset(
                            (core::mem::size_of::<WOW64_LOADMENU64>()).wrapping_add(i as usize) as isize,
                        );
                        i = i.wrapping_add(1);
                    }
                    Wow64pEnterGuestCallback(
                        core::ptr::addr_of_mut!(Frame),
                        Index,
                        Buf as *const ::core::ffi::c_void,
                        (core::mem::size_of::<WOW64_LOADMENU32>() as usize).wrapping_add(Tail as usize) as ULONG,
                    );
                    break 'sw;
                }

                16 => {
                    /* LPK: 56 -> 44, then the string. */
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    struct WOW64_LPK64 {
                        lpString: ULONGLONG, hdc: ULONGLONG, x: LONG, y: LONG,
                        flags: ULONG, rect: RECT, count: ULONG, bRect: LONG,
                    }
                    #[repr(C)]
                    #[derive(Copy, Clone)]
                    struct WOW64_LPK32 {
                        lpString: ULONG, hdc: ULONG, x: LONG, y: LONG,
                        flags: ULONG, rect: RECT, count: ULONG, bRect: LONG,
                    }
                    const _: () = assert!(
                        core::mem::size_of::<WOW64_LPK64>() == 56 && core::mem::size_of::<WOW64_LPK32>() == 44
                    );
                    C_ASSERT!(
                        core::mem::size_of::<WOW64_LPK64>() == 56
                            && core::mem::size_of::<WOW64_LPK32>() == 44
                    );
                    let A64: *const WOW64_LPK64 = Arg as *const WOW64_LPK64;
                    let mut Buf: *mut UCHAR = core::ptr::addr_of_mut!(Frame.Buf64) as *mut UCHAR;
                    let A32: *mut WOW64_LPK32 = Buf as *mut WOW64_LPK32;
                    let mut Tail: ULONG = 0;
                    let mut i: ULONG = 0;

                    if Arg.is_null()
                        || (Len as usize) < core::mem::size_of::<WOW64_LPK64>()
                        || (*A64).lpString as usize != core::mem::size_of::<WOW64_LPK64>()
                        || (Len as usize).wrapping_sub(core::mem::size_of::<WOW64_LPK64>())
                            > core::mem::size_of_val(&Frame.Buf64)
                                .wrapping_sub(core::mem::size_of::<WOW64_LPK32>())
                    {
                        DbgPrint(
                            b"[WOW64CB] LPK arg=%p len=%lu unusable\n\0" as *const u8 as PCSTR,
                            Arg,
                            Len,
                        );
                        break 'sw;
                    }
                    Tail = (Len as usize).wrapping_sub(core::mem::size_of::<WOW64_LPK64>()) as ULONG;
                    (*A32).lpString = core::mem::size_of::<WOW64_LPK32>() as ULONG;
                    (*A32).hdc = (*A64).hdc as ULONG;
                    (*A32).x = (*A64).x;
                    (*A32).y = (*A64).y;
                    (*A32).flags = (*A64).flags;
                    (*A32).rect = (*A64).rect;
                    (*A32).count = (*A64).count;
                    (*A32).bRect = (*A64).bRect;
                    i = 0 as ULONG;
                    while i < Tail {
                        *Buf.offset(
                            (core::mem::size_of::<WOW64_LPK32>()).wrapping_add(i as usize) as isize,
                        ) = *(Arg as *const UCHAR).offset(
                            (core::mem::size_of::<WOW64_LPK64>()).wrapping_add(i as usize) as isize,
                        );
                        i = i.wrapping_add(1);
                    }
                    Wow64pEnterGuestCallback(
                        core::ptr::addr_of_mut!(Frame),
                        Index,
                        Buf as *const ::core::ffi::c_void,
                        (core::mem::size_of::<WOW64_LPK32>() as usize).wrapping_add(Tail as usize) as ULONG,
                    );
                    break 'sw;
                }

                3 | 11 | 12 => {
                    /* LOADDEFAULTCURSORS / SETWNDICONS / DELIVERUSERAPC: flat 4-byte payload. */
                    let mut Flat: ULONG = if !Arg.is_null() {
                        *(Arg as *mut ULONG)
                    } else {
                        0 as ULONG
                    };
                    Wow64pEnterGuestCallback(
                        core::ptr::addr_of_mut!(Frame),
                        Index,
                        if !Arg.is_null() {
                            core::ptr::addr_of_mut!(Flat) as *const ::core::ffi::c_void
                        } else {
                            core::ptr::null()
                        },
                        if !Arg.is_null() {
                            core::mem::size_of::<ULONG>() as ULONG
                        } else {
                            0 as ULONG
                        },
                    );
                    break 'sw;
                }

                _ => {
                    DbgPrint(
                        b"[WOW64CB] callback %lu NEEDS 64->32 MARSHALLER (len %lu) \xE2\x80\x94 failing loud\n\0"
                            as *const u8 as PCSTR,
                        Index,
                        Len,
                    );
                    break 'sw;
                }
            }
        }

        /* Marshalling failed / unimplemented: unwind our frame and tell the
         * kernel the callback failed (loud, non-fatal for optional paths). */
        *Wow64pCbTlsSlot() = Frame.Prev as PVOID;
        #[cfg(target_arch = "x86_64")]
        {
            let Slot: *mut ULONGLONG = Wow64pCpuDispatchRspSlot();
            if !Slot.is_null() && Frame.SavedDispatchRsp != 0 {
                *Slot = Frame.SavedDispatchRsp;
            }
        }
        return NtCallbackReturn(core::ptr::null_mut(), 0, WOW64_STATUS_NOT_IMPLEMENTED);
    }

    /* longjmp landing: guest completed via NtCallbackReturn marshaller. */
    *Wow64pCbTlsSlot() = Frame.Prev as PVOID;
    /* ...and control is crossing back into the kernel, so last-error comes with it. */
    Wow64LastErrorToHost();
    Frame.SavedCtx.ContextFlags = WOW64_CB_CONTEXT_FULL | WOW64_CB_CONTEXT_EXTENDED;
    (Wow64pBTCpuSetContext.unwrap_unchecked())(
        (-2 as LONG_PTR) as HANDLE,
        WOW64_CURRENT_PROCESS,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Frame.SavedCtx) as PVOID,
    );
    #[cfg(target_arch = "x86_64")]
    {
        /* Clear the CPU-area REPLACED flag so the outer syscall does its normal fixup. */
        Wow64pClearCpuResumeFlag();
        let Slot: *mut ULONGLONG = Wow64pCpuDispatchRspSlot();
        if !Slot.is_null() && Frame.SavedDispatchRsp != 0 {
            *Slot = Frame.SavedDispatchRsp;
        }
    }
    St = NtCallbackReturn(Frame.RetBuf, Frame.RetLen, Frame.Status);
    DbgPrint(
        b"[WOW64CB] FATAL: NtCallbackReturn(%p,%lu) => %08lx fell through\n\0" as *const u8 as PCSTR,
        Frame.RetBuf,
        Frame.RetLen,
        St,
    );
    NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBADCC);
    return St;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pCompleteUserCallback(Result32: ULONG, Len32: ULONG, Status: NTSTATUS) -> ::core::ffi::c_int {
    let Frame: *mut WOW64_CB_FRAME = *Wow64pCbTlsSlot() as *mut WOW64_CB_FRAME;

    if Frame.is_null() {
        return 0;
    }

    (*Frame).Status = Status;
    (*Frame).RetBuf = core::ptr::null_mut();
    (*Frame).RetLen = 0;

    if Result32 != 0 && Len32 != 0 {
        match (*Frame).Index {
            0 => {
                let A32: *const WOW64_WNDPROC_CB32 =
                    Result32 as ULONG_PTR as *const WOW64_WNDPROC_CB32;
                let A64: *mut WOW64_WNDPROC_CB64 =
                    &raw mut (*Frame).Buf64 as *mut UCHAR as *mut WOW64_WNDPROC_CB64;
                let mut N: ULONG = (*Frame).ArgLen64;
                let mut i: ULONG = 0;
                let mut Extra: ULONG = if N as usize > core::mem::size_of::<WOW64_WNDPROC_CB64>() {
                    (N as usize - core::mem::size_of::<WOW64_WNDPROC_CB64>()) as ULONG
                } else {
                    0 as ULONG
                };
                let P32: *const BYTE = (A32 as *const BYTE)
                    .offset(core::mem::size_of::<WOW64_WNDPROC_CB32>() as isize);
                let P64: *mut BYTE = (&raw mut (*Frame).Buf64 as *mut UCHAR)
                    .offset(core::mem::size_of::<WOW64_WNDPROC_CB64>() as isize);

                if N as usize > core::mem::size_of::<[UCHAR; 1024]>() {
                    N = core::mem::size_of::<[UCHAR; 1024]>() as ULONG;
                }
                i = 0;
                while i < N {
                    (*Frame).Buf64[i as usize] =
                        *((*Frame).OrigArg as *const BYTE).offset(i as isize) as UCHAR;
                    i = i.wrapping_add(1);
                }
                (*A64).Result = (*A32).Result as ULONGLONG;

                match (*Frame).Msg {
                    WOW64_WM_GETMINMAXINFO
                    | WOW64_WM_SIZING
                    | WOW64_WM_MOVING
                    | WOW64_WM_STYLECHANGING
                    | WOW64_WM_GETTEXT => {
                        if Extra != 0
                            && Len32 as usize
                                >= core::mem::size_of::<WOW64_WNDPROC_CB32>() + Extra as usize
                        {
                            i = 0;
                            while i < Extra {
                                *P64.offset(i as isize) =
                                    *P32.offset(i as isize);
                                i = i.wrapping_add(1);
                            }
                            if Wow64TraceSyscalls != 0 {
                                DbgPrint(
                                    b"[WOW64CB] msg 0x%lx OUT copied back (%lub)\n\0".as_ptr() as PCSTR,
                                    (*Frame).Msg,
                                    Extra,
                                );
                            }
                        }
                    }

                    WOW64_WM_NCCALCSIZE => {
                        let mut cb: ULONG = if Extra >= 48 as ULONG {
                            48 as ULONG
                        } else {
                            Extra
                        };
                        if cb != 0
                            && Len32 as usize
                                >= core::mem::size_of::<WOW64_WNDPROC_CB32>() + cb as usize
                        {
                            i = 0;
                            while i < cb {
                                *P64.offset(i as isize) =
                                    *P32.offset(i as isize);
                                i = i.wrapping_add(1);
                            }
                            if Wow64TraceSyscalls != 0 {
                                DbgPrint(
                                    b"[WOW64CB] NCCALCSIZE OUT copied back (%lub rects)\n\0".as_ptr() as PCSTR,
                                    cb,
                                );
                            }
                        }
                    }

                    WOW64_WM_WINDOWPOSCHANGING => {
                        if Len32 as usize
                            >= core::mem::size_of::<WOW64_WNDPROC_CB32>()
                                + core::mem::size_of::<WOW64_WINDOWPOS32>()
                            && Extra as usize >= core::mem::size_of::<WOW64_WINDOWPOS64>()
                        {
                            let G32: *const WOW64_WINDOWPOS32 =
                                P32 as *const WOW64_WINDOWPOS32;
                            let G64: *mut WOW64_WINDOWPOS64 =
                                P64 as *mut WOW64_WINDOWPOS64;
                            (*G64).x = (*G32).x;
                            (*G64).y = (*G32).y;
                            (*G64).cx = (*G32).cx;
                            (*G64).cy = (*G32).cy;
                            (*G64).flags = (*G32).flags;
                            if Wow64TraceSyscalls != 0 {
                                DbgPrint(
                                    b"[WOW64CB] WINDOWPOSCHANGING OUT copied back (cx=%ld cy=%ld fl=0x%lx)\n\0"
                                        .as_ptr() as PCSTR,
                                    (*G64).cx,
                                    (*G64).cy,
                                    (*G64).flags,
                                );
                            }
                        }
                    }

                    _ => {
                        let Cbm: *const WOW64_CB_MARSHAL =
                            Wow64CbFindMarshal((*Frame).Msg as ::core::ffi::c_uint);
                        if !Cbm.is_null()
                            && Extra >= (*Cbm).Size64 as ULONG
                            && Len32 as usize
                                >= core::mem::size_of::<WOW64_WNDPROC_CB32>()
                                    + (*Cbm).Size32 as usize
                        {
                            (*Cbm).Up.unwrap_unchecked()(
                                P32 as *const ::core::ffi::c_uchar,
                                P64 as *mut ::core::ffi::c_uchar,
                            );
                            if Wow64TraceSyscalls != 0 {
                                DbgPrint(
                                    b"[WOW64CB] msg 0x%lx OUT marshalled back (gen)\n\0".as_ptr() as PCSTR,
                                    (*Frame).Msg,
                                );
                            }
                        }
                    }
                }

                if Wow64TraceSyscalls != 0 {
                    DbgPrint(
                        b"[WOW64CB] <- WINDOWPROC msg 0x%lx result=%08lx%08lx retlen=%lu (guest ret32: proc=%08lx ansi=%lu wnd=%08lx msg=%04lx lpbs=%ld len32=%lu)\n\0"
                            .as_ptr() as PCSTR,
                        (*Frame).Msg,
                        ((*A64).Result >> 32) as ULONG,
                        (*A64).Result as ULONG,
                        N,
                        (*A32).Proc,
                        (*A32).IsAnsiProc,
                        (*A32).Wnd,
                        (*A32).Msg,
                        (*A32).lParamBufferSize,
                        Len32,
                    );
                }
                (*Frame).RetBuf = A64 as PVOID;
                (*Frame).RetLen = N;
            }

            13 | 14 => {
                let R32: *const WOW64_DDECB32 =
                    Result32 as ULONG_PTR as *const WOW64_DDECB32;
                let R64: *mut WOW64_DDECB64 =
                    &raw mut (*Frame).Buf64 as *mut UCHAR as *mut WOW64_DDECB64;
                let mut N: ULONG = (*Frame).ArgLen64;
                let mut i: ULONG = 0;

                if N as usize > core::mem::size_of::<[UCHAR; 1024]>() {
                    N = core::mem::size_of::<[UCHAR; 1024]>() as ULONG;
                }
                if (*Frame).OrigArg.is_null()
                    || N < WOW64_DDECB64_HDR as ULONG
                    || Len32 < WOW64_DDECB32_HDR as ULONG
                {
                    DbgPrint(
                        b"[WOW64CB] DDE%s result unusable (len32 %lu, len64 %lu, status %08lx, first dword %08lx)\n\0"
                            .as_ptr() as PCSTR,
                        if (*Frame).Index == 13 as ULONG {
                            b"POST\0".as_ptr() as *const ::core::ffi::c_char
                        } else {
                            b"GET\0".as_ptr() as *const ::core::ffi::c_char
                        },
                        Len32,
                        N,
                        Status as ULONG,
                        if Len32 >= 4 as ULONG {
                            *(Result32 as ULONG_PTR as *const ULONG)
                        } else {
                            0 as ULONG
                        },
                    );
                    (*Frame).Status = WOW64_STATUS_NOT_IMPLEMENTED;
                } else {
                    i = 0;
                    while i < N {
                        (*Frame).Buf64[i as usize] =
                            *((*Frame).OrigArg as *const BYTE).offset(i as isize) as UCHAR;
                        i = i.wrapping_add(1);
                    }
                    (*R64).lParam =
                        (*R32).lParam as LONG as LONGLONG as ULONGLONG;
                    (*R64).size = (*R32).size;
                    (*R64).pvData =
                        (*R32).pvData as ULONGLONG;
                    (*Frame).RetBuf = &raw mut (*Frame).Buf64 as *mut UCHAR as PVOID;
                    (*Frame).RetLen = N;
                }
            }

            8 | 10 | 6 => {
                let R: ULONGLONG = (if Len32 as usize >= core::mem::size_of::<ULONG>() {
                    *(Result32 as ULONG_PTR as *const ULONG)
                } else {
                    0 as ULONG
                }) as ULONGLONG;
                let mut i: ULONG = 0;
                i = 0;
                while (i as usize) < core::mem::size_of_val(&R) {
                    (*Frame).Buf64[i as usize] =
                        *(&raw const R as *const BYTE).offset(i as isize) as UCHAR;
                    i = i.wrapping_add(1);
                }
                (*Frame).RetBuf = &raw mut (*Frame).Buf64 as *mut UCHAR as PVOID;
                (*Frame).RetLen = core::mem::size_of::<ULONGLONG>() as ULONG;
            }

            _ => {
                let mut N: ULONG = Len32;
                let mut i: ULONG = 0;
                if N as usize > core::mem::size_of::<[UCHAR; 1024]>() {
                    N = core::mem::size_of::<[UCHAR; 1024]>() as ULONG;
                }
                i = 0;
                while i < N {
                    (*Frame).Buf64[i as usize] =
                        *(Result32 as ULONG_PTR as *const BYTE).offset(i as isize) as UCHAR;
                    i = i.wrapping_add(1);
                }
                (*Frame).RetBuf = &raw mut (*Frame).Buf64 as *mut UCHAR as PVOID;
                (*Frame).RetLen = N;
            }
        }
    }

    Wow64pLongJmp(
        &raw mut (*Frame).Jmp as *mut ULONGLONG,
        1,
    );
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pInstallUserCallbackTable() {
    let Teb: PBYTE = NtCurrentTeb() as PBYTE;
    let Peb: PBYTE = *(Teb.add(WOW64_TEB_PEB_OFFSET) as *mut PBYTE);
    *(Peb.add(WOW64_PEB_KERNELCALLBACKTABLE) as *mut PVOID) = core::ptr::addr_of_mut!(Wow64pUserCallbackTable) as PVOID;
    DbgPrint(
        b"[WOW64CB] PEB64 KernelCallbackTable -> wow64 bridge (%p), guest dispatcher %p\n\0".as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(Wow64pUserCallbackTable) as PVOID,
        Wow64pKiUserCallbackDispatcher32,
    );
}

unsafe extern "C" fn Wow64pTlsSlot(Index: ULONG) -> *mut PVOID {
    return (NtCurrentTeb() as PBYTE)
        .add(WOW64_TEB_TLSSLOTS_OFFSET as usize)
        .add((Index as usize).wrapping_mul(core::mem::size_of::<PVOID>()))
        as *mut PVOID;
}
pub unsafe extern "system" fn Wow64ApcRoutine(Packed: ULONG_PTR, Arg1: ULONG_PTR, Arg2: ULONG_PTR, Native: PVOID) {
    let mut Routine: ULONG = (Packed >> 32) as ULONG;
    let mut Context: ULONG = Packed as ULONG;
    let mut CAS: ULONG = ((core::mem::size_of::<WOW64_CB_CONTEXT>() + 3) & !(3)) as ULONG;
    let mut E: ULONG = 0;
    let mut i: ULONG = 0;
    let mut Frame: WOW64_APC_FRAME = core::mem::zeroed();
    let mut Run: WOW64_CB_CONTEXT = core::mem::zeroed();

    if (*Wow64pTlsSlot((WOW64_TLS_INCPUSIMULATION) as ULONG)).is_null() {
        /* A thread that has not entered the guest yet: an APC queued to a new
         * suspended thread, delivered by the native loader's thread-init
         * NtTestAlert. Queue it again and resume WITHOUT testing for alerts
         * (returning would NtContinue with TestAlert and deliver it straight
         * back here); the guest loader's own NtTestAlert, once this thread has
         * guest state, delivers it where it can run. */
        let St: NTSTATUS = NtQueueApcThread(
            (-(2) as LONG_PTR) as HANDLE,
            core::mem::transmute::<*mut c_void, _>(Wow64ApcRoutine as usize as PVOID),
            Packed as PVOID,
            Arg1 as PVOID,
            Arg2 as PVOID,
        );
        if St < 0 {
            DbgPrint(
                b"[WOW64] guest APC %08lx before guest entry: requeue failed %08lx\n\0".as_ptr() as PCSTR,
                Routine,
                St,
            );
        }
        NtContinue((Native).cast(), (FALSE) as BOOLEAN);
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBADA1);
    }
    if Wow64pKiUserApcDispatcher32.is_null()
        || Wow64pBTCpuGetContext.is_none()
        || Wow64pBTCpuSetContext.is_none()
        || Wow64pBTCpuSimulate.is_none()
    {
        DbgPrint(
            b"[WOW64] guest APC %08lx with no guest dispatcher - dropped\n\0".as_ptr() as PCSTR,
            Routine,
        );
        return;
    }

    Frame.Saved.ContextFlags = WOW64_CB_CONTEXT_FULL | WOW64_CB_CONTEXT_EXTENDED;
    (Wow64pBTCpuGetContext.unwrap_unchecked())(
        (-(2) as LONG_PTR) as HANDLE,
        WOW64_CURRENT_PROCESS,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Frame.Saved) as PVOID,
    );
    #[cfg(target_arch = "x86_64")]
    {
        let Slot: *mut ULONGLONG = Wow64pCpuDispatchRspSlot();
        Frame.SavedDispatchRsp = if !Slot.is_null() { *Slot } else { 0 };
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        Frame.SavedDispatchRsp = 0;
    }

    E = Frame.Saved.Esp.wrapping_sub(16 + CAS + 8) & !(15);
    *((E.wrapping_add(0) as ULONG_PTR) as *mut ULONG) = Routine;
    *((E.wrapping_add(4) as ULONG_PTR) as *mut ULONG) = Context;
    *((E.wrapping_add(8) as ULONG_PTR) as *mut ULONG) = Arg1 as ULONG;
    *((E.wrapping_add(12) as ULONG_PTR) as *mut ULONG) = Arg2 as ULONG;
    i = 0;
    while (i as usize) < core::mem::size_of::<WOW64_CB_CONTEXT>() {
        *((E.wrapping_add(16).wrapping_add(i) as ULONG_PTR) as PBYTE) =
            *((core::ptr::addr_of_mut!(Frame.Saved) as *const BYTE).offset(i as isize));
        i = i.wrapping_add(1);
    }
    *((E.wrapping_add(16).wrapping_add(CAS) as ULONG_PTR) as *mut ULONG) = 0; /* SEH scratch (dispatcher writes) */
    *((E.wrapping_add(16).wrapping_add(CAS).wrapping_add(4) as ULONG_PTR) as *mut ULONG) = 0;

    Frame.GuestContext = E.wrapping_add(16);
    Frame.Native = Native;
    Frame.Prev = *Wow64pTlsSlot((WOW64_TLS_APCLIST) as ULONG) as *mut WOW64_APC_FRAME;
    *Wow64pTlsSlot(7 as ULONG /* WOW64_TLS_APCLIST */) = core::ptr::addr_of_mut!(Frame) as PVOID;

    if Wow64TraceSyscalls != 0 {
        DbgPrint(
            b"[WOW64] guest APC %08lx(%08lx, %08lx, %08lx) frame %08lx\n\0".as_ptr() as PCSTR,
            Routine,
            Context,
            Arg1 as ULONG,
            Arg2 as ULONG,
            E,
        );
    }

    Run = Frame.Saved;
    Run.ContextFlags = WOW64_CB_CONTEXT_FULL | WOW64_CB_CONTEXT_EXTENDED;
    Run.Eip = Wow64pKiUserApcDispatcher32 as usize as ULONG;
    Run.Esp = E;
    (Wow64pBTCpuSetContext.unwrap_unchecked())(
        (-(2) as LONG_PTR) as HANDLE,
        WOW64_CURRENT_PROCESS,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Run) as PVOID,
    );
    (Wow64pBTCpuSimulate.unwrap_unchecked())(); /* exits via the guest's NtContinue -> Wow64pCompleteGuestApc */

    DbgPrint(
        b"[WOW64] FATAL: guest APC %08lx returned without NtContinue\n\0".as_ptr() as PCSTR,
        Routine,
    );
    NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBADA2);
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pCompleteGuestApc(Context32: ULONG, TestAlert: BOOLEAN) -> ::core::ffi::c_int {
    let mut Frame: *mut WOW64_APC_FRAME = (*Wow64pTlsSlot((WOW64_TLS_APCLIST) as ULONG)) as *mut WOW64_APC_FRAME;
    let mut St: NTSTATUS = 0;

    if Frame.is_null() || (*Frame).GuestContext != Context32 {
        return 0;
    }
    *Wow64pTlsSlot(WOW64_TLS_APCLIST as ULONG) = (*Frame).Prev as PVOID;

    /* Back to where the APC found the guest -- not to the frame's CONTEXT -- so
     * the next APC of the same drain reads the interrupted state. */
    (*Frame).Saved.ContextFlags = (WOW64_CB_CONTEXT_FULL | WOW64_CB_CONTEXT_EXTENDED) as ULONG;
    (Wow64pBTCpuSetContext.unwrap_unchecked())(
        -(2 as LONG_PTR) as HANDLE,
        WOW64_CURRENT_PROCESS,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*Frame).Saved) as PVOID,
    );
    /* As for a callback: the interrupted syscall still owes its normal return
     * fixup, and the nesting moved the dispatcher RSP. (`#ifdef _M_AMD64` in the C;
     * the translation had dropped the guard.) */
    #[cfg(target_arch = "x86_64")]
    {
        Wow64pClearCpuResumeFlag();
        let Slot: *mut ULONGLONG = Wow64pCpuDispatchRspSlot();
        if !Slot.is_null() && (*Frame).SavedDispatchRsp != 0 {
            *Slot = (*Frame).SavedDispatchRsp;
        }
    }
    St = NtContinue(((*Frame).Native).cast(), TestAlert);
    DbgPrint(b"[WOW64] FATAL: NtContinue out of a guest APC => %08lx\n\0".as_ptr() as PCSTR, St);
    NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBADA3 as NTSTATUS);
    return 1;
}
