use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::k32_externs::{BaseCreateStack, BaseInitializeContext, ExitThread, RtlZeroMemory};

extern "system" {
    fn GetProcessHeap() -> HANDLE;
}

// FIBER, PFIBER, RTL_FLS_DATA, PRTL_FLS_DATA, LIST_ENTRY, PLIST_ENTRY,
// PFLS_CALLBACK_FUNCTION, LPFIBER_START_ROUTINE, LPVOID, PTEB, PPEB,
// PACTIVATION_CONTEXT_STACK, and all the FLS_*/FIBER_*/CONTEXT_*/ERROR_*/HEAP_*/
// MEM_* constants come from ntdll_sys::nt::* (bindgen) — do not redefine.

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// EXCEPTION_CHAIN_END is `((PEXCEPTION_REGISTRATION_RECORD)-1)` (winnt.h) — not
// bindgen-evaluable.
const EXCEPTION_CHAIN_END: *mut _EXCEPTION_REGISTRATION_RECORD =
    !0usize as *mut _EXCEPTION_REGISTRATION_RECORD;

// PEB.FlsCallback is typed `*mut PVOID` by bindgen. Its slots are Windows' FLS_CALLBACK_INFO
// pairs { Callback, Unknown }, the layout ntdll's RtlFlsAlloc/RtlFlsFree use (ntdll-rust
// rtl/fls.rs, upstream fls.c). The C here indexes it as single pointers, so a process that
// reaches both families (Vista+ kernel32 routes FlsAlloc/FlsFree to ntdll while keeping
// this BaseRundownFls) would read the callback for slot n from slot n/2. One layout for both.
#[repr(C)]
struct FLS_CALLBACK_INFO {
    Callback: PFLS_CALLBACK_FUNCTION,
    Unknown: PVOID,
}

/// The callback of FLS slot `index`.
#[inline]
unsafe fn fls_callbacks(Peb: PPEB, index: u32) -> *mut PFLS_CALLBACK_FUNCTION {
    let slots = (*Peb).FlsCallback as *mut FLS_CALLBACK_INFO;
    core::ptr::addr_of_mut!((*slots.add(index as usize)).Callback)
}

#[no_mangle]
pub unsafe extern "system" fn BaseRundownFls(FlsData: PVOID) {
    let mut n: u32;
    let FlsHighIndex: u32;
    let pFlsData: PRTL_FLS_DATA;
    let mut lpCallback: PFLS_CALLBACK_FUNCTION;

    pFlsData = FlsData as PRTL_FLS_DATA;

    RtlAcquirePebLock();
    FlsHighIndex = (*NtCurrentPeb()).FlsHighIndex;
    RemoveEntryList(&mut (*pFlsData).ListEntry);
    RtlReleasePebLock();

    n = 1;
    while n <= FlsHighIndex {
        lpCallback = *fls_callbacks(NtCurrentPeb(), n);
        if lpCallback.is_some() && !(*core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(n as usize)).is_null() {
            (lpCallback.unwrap())(*core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(n as usize));
        }
        n += 1;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, FlsData);
}

#[no_mangle]
pub unsafe extern "system" fn ConvertFiberToThread() -> BOOL {
    let Teb: PTEB;
    let FiberData: PFIBER;

    Teb = NtCurrentTeb();
    if (*Teb).HasFiberData == 0 {
        SetLastError(ERROR_ALREADY_THREAD);
        return 0i32;
    }

    (*Teb).HasFiberData = FALSE as BOOLEAN;
    FiberData = (*Teb).NtTib.__bindgen_anon_1.FiberData as PFIBER;
    (*Teb).NtTib.__bindgen_anon_1.FiberData = core::ptr::null_mut();

    RtlFreeHeap(RtlGetProcessHeap(), 0, FiberData as LPVOID);

    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn ConvertThreadToFiberEx(
    lpParameter: LPVOID,
    dwFlags: DWORD,
) -> LPVOID {
    let Teb: PTEB;
    let Fiber: PFIBER;

    /* Check for invalid flags */
    if (dwFlags & !FIBER_FLAG_FLOAT_SWITCH) != 0 {
        /* Fail */
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    /* Are we already a fiber? */
    Teb = NtCurrentTeb();
    if (*Teb).HasFiberData != 0 {
        /* Fail */
        SetLastError(ERROR_ALREADY_FIBER);
        return core::ptr::null_mut();
    }

    /* Allocate the fiber */
    Fiber = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<FIBER>() as SIZE_T,
    ) as PFIBER;
    if Fiber.is_null() {
        /* Fail */
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    /* Copy some contextual data from the thread to the fiber */
    (*Fiber).FiberData = lpParameter;
    (*Fiber).ExceptionList = (*Teb).NtTib.ExceptionList;
    (*Fiber).StackBase = (*Teb).NtTib.StackBase;
    (*Fiber).StackLimit = (*Teb).NtTib.StackLimit;
    (*Fiber).DeallocationStack = (*Teb).DeallocationStack;
    (*Fiber).FlsData = (*Teb).FlsData;
    (*Fiber).GuaranteedStackBytes = (*Teb).GuaranteedStackBytes;
    (*Fiber).ActivationContextStackPointer = (*Teb).ActivationContextStackPointer;

    /* Save FPU State if requested, otherwise just the basic registers */
    (*Fiber).FiberContext.ContextFlags = if (dwFlags & FIBER_FLAG_FLOAT_SWITCH) != 0 {
        CONTEXT_FULL | CONTEXT_FLOATING_POINT
    } else {
        CONTEXT_FULL
    };

    /* Associate the fiber to the current thread */
    (*Teb).NtTib.__bindgen_anon_1.FiberData = Fiber as LPVOID;
    (*Teb).HasFiberData = TRUE as BOOLEAN;

    /* Return opaque fiber data */
    return Fiber as LPVOID;
}

#[no_mangle]
pub unsafe extern "system" fn ConvertThreadToFiber(
    lpParameter: LPVOID,
) -> LPVOID {
    return ConvertThreadToFiberEx(lpParameter, 0);
}

#[no_mangle]
pub unsafe extern "system" fn CreateFiber(
    dwStackSize: SIZE_T,
    lpStartAddress: LPFIBER_START_ROUTINE,
    lpParameter: LPVOID,
) -> LPVOID {
    return CreateFiberEx(dwStackSize, 0, 0, lpStartAddress, lpParameter);
}

#[no_mangle]
pub unsafe extern "system" fn CreateFiberEx(
    dwStackCommitSize: SIZE_T,
    dwStackReserveSize: SIZE_T,
    dwFlags: DWORD,
    lpStartAddress: LPFIBER_START_ROUTINE,
    lpParameter: LPVOID,
) -> LPVOID {
    let Fiber: PFIBER;
    let mut Status: NTSTATUS;
    let mut InitialTeb: INITIAL_TEB = core::mem::zeroed();
    let mut ActivationContextStackPointer: PACTIVATION_CONTEXT_STACK;

    /* Check for invalid flags */
    if (dwFlags & !FIBER_FLAG_FLOAT_SWITCH) != 0 {
        /* Fail */
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    /* Allocate the Activation Context Stack */
    ActivationContextStackPointer = core::ptr::null_mut();
    Status = RtlAllocateActivationContextStack(&mut ActivationContextStackPointer);
    if !NT_SUCCESS(Status) {
        /* Fail */
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    /* Allocate the fiber */
    Fiber = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<FIBER>() as SIZE_T,
    ) as PFIBER;
    if Fiber.is_null() {
        /* Free the activation context stack */
        RtlFreeActivationContextStack(ActivationContextStackPointer);

        /* Fail */
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    /* Create the stack for the fiber */
    Status = BaseCreateStack(
        NtCurrentProcess(),
        dwStackCommitSize,
        dwStackReserveSize,
        &mut InitialTeb,
    );
    if !NT_SUCCESS(Status) {
        /* Free the fiber */
        RtlFreeHeap(
            GetProcessHeap(),
            0,
            Fiber as LPVOID,
        );

        /* Free the activation context stack */
        RtlFreeActivationContextStack(ActivationContextStackPointer);

        /* Failure */
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    /* Clear the context */
    RtlZeroMemory(
        &mut (*Fiber).FiberContext as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<CONTEXT>() as SIZE_T,
    );

    /* Copy the data into the fiber */
    (*Fiber).StackBase = InitialTeb.StackBase;
    (*Fiber).StackLimit = InitialTeb.StackLimit;
    (*Fiber).DeallocationStack = InitialTeb.AllocatedStackBase;
    (*Fiber).FiberData = lpParameter;
    (*Fiber).ExceptionList = EXCEPTION_CHAIN_END;
    (*Fiber).GuaranteedStackBytes = 0;
    (*Fiber).FlsData = core::ptr::null_mut();
    (*Fiber).ActivationContextStackPointer = ActivationContextStackPointer;

    /* Save FPU State if requested, otherwise just the basic registers */
    (*Fiber).FiberContext.ContextFlags = if (dwFlags & FIBER_FLAG_FLOAT_SWITCH) != 0 {
        CONTEXT_FULL | CONTEXT_FLOATING_POINT
    } else {
        CONTEXT_FULL
    };

    /* Initialize the context for the fiber */
    BaseInitializeContext(
        &mut (*Fiber).FiberContext,
        lpParameter,
        lpStartAddress.map_or(core::ptr::null_mut(), |f| f as PVOID),
        InitialTeb.StackBase,
        2,
    );

    /* Return the Fiber */
    Fiber as LPVOID
}
#[no_mangle]
pub unsafe extern "system" fn DeleteFiber(lpFiber: LPVOID) {
    let mut Size: SIZE_T;
    let Fiber: PFIBER;
    let Teb: PTEB;

    Teb = NtCurrentTeb();
    Fiber = lpFiber as PFIBER;
    if (*Teb).HasFiberData != 0 && (*Teb).NtTib.__bindgen_anon_1.FiberData == Fiber as PVOID {
        ExitThread(1);
    }

    Size = 0;
    NtFreeVirtualMemory(
        NtCurrentProcess(),
        &mut (*Fiber).DeallocationStack as *mut _ as *mut *mut _,
        &mut Size as *mut _,
        MEM_RELEASE,
    );

    if !(*Fiber).FlsData.is_null() {
        BaseRundownFls((*Fiber).FlsData);
    }

    RtlFreeActivationContextStack((*Fiber).ActivationContextStackPointer);

    RtlFreeHeap(RtlGetProcessHeap(), 0, lpFiber);
}

#[no_mangle]
pub unsafe extern "system" fn IsThreadAFiber() -> BOOL {
    (*NtCurrentTeb()).HasFiberData as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn FlsAlloc(lpCallback: PFLS_CALLBACK_FUNCTION) -> DWORD {
    let mut dwFlsIndex: DWORD;
    let Peb: PPEB = NtCurrentPeb();
    let mut pFlsData: PRTL_FLS_DATA;

    RtlAcquirePebLock();

    pFlsData = (*NtCurrentTeb()).FlsData as PRTL_FLS_DATA;

    if (*Peb).FlsCallback == core::ptr::null_mut()
        && {
            (*Peb).FlsCallback = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                ((FLS_MAXIMUM_AVAILABLE as usize) * core::mem::size_of::<FLS_CALLBACK_INFO>()) as SIZE_T,
            ) as *mut _;
            (*Peb).FlsCallback.is_null()
        }
    {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        dwFlsIndex = FLS_OUT_OF_INDEXES;
    } else {
        dwFlsIndex = RtlFindClearBitsAndSet(((*Peb).FlsBitmap as PRTL_BITMAP), 1, 1);
        if dwFlsIndex != FLS_OUT_OF_INDEXES {
            if pFlsData.is_null()
                && {
                    pFlsData = RtlAllocateHeap(
                        RtlGetProcessHeap(),
                        HEAP_ZERO_MEMORY,
                        core::mem::size_of::<RTL_FLS_DATA>() as SIZE_T,
                    ) as PRTL_FLS_DATA;
                    pFlsData.is_null()
                }
            {
                RtlClearBits(((*Peb).FlsBitmap as PRTL_BITMAP), dwFlsIndex, 1);
                dwFlsIndex = FLS_OUT_OF_INDEXES;
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            } else {
                if (*NtCurrentTeb()).FlsData.is_null() {
                    (*NtCurrentTeb()).FlsData = pFlsData as PVOID;
                    InsertTailList(&mut (*Peb).FlsListHead, &mut (*pFlsData).ListEntry);
                }

                *core::ptr::addr_of_mut!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize) = core::ptr::null_mut();
                *fls_callbacks(Peb, dwFlsIndex) = lpCallback;

                if dwFlsIndex > (*Peb).FlsHighIndex {
                    (*Peb).FlsHighIndex = dwFlsIndex;
                }
            }
        } else {
            SetLastError(ERROR_NO_MORE_ITEMS);
        }
    }
    RtlReleasePebLock();
    return dwFlsIndex;
}

#[no_mangle]
pub unsafe extern "system" fn FlsFree(dwFlsIndex: DWORD) -> BOOL {
    let mut ret: BOOL;
    let Peb: PPEB = NtCurrentPeb();

    if dwFlsIndex >= FLS_MAXIMUM_AVAILABLE as DWORD {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    RtlAcquirePebLock();

    // _SEH2_TRY
    {
        ret = if RtlAreBitsSet(((*Peb).FlsBitmap as PRTL_BITMAP), dwFlsIndex, 1) != 0 { 1 } else { 0 };
        if ret != 0 {
            let mut Entry: PLIST_ENTRY;
            let lpCallback: PFLS_CALLBACK_FUNCTION;

            RtlClearBits(((*Peb).FlsBitmap as PRTL_BITMAP), dwFlsIndex, 1);
            lpCallback = *fls_callbacks(Peb, dwFlsIndex);

            Entry = (*Peb).FlsListHead.Flink;
            while Entry != &mut (*Peb).FlsListHead as *mut _ as PLIST_ENTRY {
                let pFlsData: PRTL_FLS_DATA = {
                    let _addr = Entry as *mut u8;
                    let _offset = core::mem::offset_of!(RTL_FLS_DATA, ListEntry) as isize;
                    _addr.offset(-_offset) as *mut RTL_FLS_DATA
                };
                if *core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize) != core::ptr::null_mut() {
                    if let Some(cb) = lpCallback {
                        cb(*core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize));
                    }
                    *core::ptr::addr_of_mut!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize) = core::ptr::null_mut();
                }
                Entry = (*Entry).Flink;
            }
            *fls_callbacks(Peb, dwFlsIndex) = None;
        } else {
            SetLastError(ERROR_INVALID_PARAMETER);
        }
    }

    // _SEH2_FINALLY
    {
        RtlReleasePebLock();
    }
    // _SEH2_END

    ret
}

#[no_mangle]
pub unsafe extern "system" fn FlsGetValue(dwFlsIndex: DWORD) -> LPVOID {
    let pFlsData: PRTL_FLS_DATA;
    pFlsData = (*NtCurrentTeb()).FlsData as PRTL_FLS_DATA;
    if dwFlsIndex == 0 || dwFlsIndex >= FLS_MAXIMUM_AVAILABLE || pFlsData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }
    SetLastError(ERROR_SUCCESS);
    return *core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize);
}

/// Windows 11's FlsGetValue2: FlsGetValue without the last-error traffic. It never sets
/// it, not for a freed or never-set slot either (both read NULL; winref
/// 20261031-223113-3h38le85). An index out of range reads NULL rather than past the array.
#[no_mangle]
pub unsafe extern "system" fn FlsGetValue2(dwFlsIndex: DWORD) -> LPVOID {
    let pFlsData = (*NtCurrentTeb()).FlsData as PRTL_FLS_DATA;
    if dwFlsIndex == 0 || dwFlsIndex >= FLS_MAXIMUM_AVAILABLE || pFlsData.is_null() {
        return core::ptr::null_mut();
    }
    *core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize)
}

#[no_mangle]
pub unsafe extern "system" fn FlsSetValue(
    dwFlsIndex: DWORD,
    lpFlsData: PVOID,
) -> BOOL {
    let mut pFlsData: PRTL_FLS_DATA;

    if dwFlsIndex == 0 || dwFlsIndex >= FLS_MAXIMUM_AVAILABLE {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    pFlsData = (*NtCurrentTeb()).FlsData as PRTL_FLS_DATA;

    if (*NtCurrentTeb()).FlsData.is_null()
        && {
            (*NtCurrentTeb()).FlsData = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<RTL_FLS_DATA>() as SIZE_T,
            );
            (*NtCurrentTeb()).FlsData.is_null()
        }
    {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    if pFlsData.is_null() {
        pFlsData = (*NtCurrentTeb()).FlsData as PRTL_FLS_DATA;
        RtlAcquirePebLock();
        InsertTailList(
            &mut (*NtCurrentPeb()).FlsListHead,
            &mut (*pFlsData).ListEntry,
        );
        RtlReleasePebLock();
    }
    *core::ptr::addr_of_mut!((*pFlsData).Data).cast::<PVOID>().add(dwFlsIndex as usize) = lpFlsData;
    return 1;
}


// ============================================================================
// SwitchToFiber / BaseFiberStartup on amd64 and arm64
// ============================================================================
//
// The C tree only has these for i386 (client/i386/fiber.S). client/amd64/fiber.S is an
// UNIMPLEMENTED stub whose own print faults, and client/arm64/fiber.S is `brk #0xf000`, in
// upstream as well, so every 64-bit program died on its first SwitchToFiber. These follow the
// i386 routine: save the callee-saved state into the current fiber's FiberContext, move the
// per-fiber TEB fields across, load the target, and continue it.
//
// - A fiber is resumed by restoring its stack pointer and jumping to its saved PC, which for a
//   suspended fiber is SwitchToFiber's return address, so SwitchToFiber "returns" in the
//   target. A new fiber's context comes from BaseInitializeContext (utils.rs): PC =
//   BaseFiberStartup, with the stack pointer set as a call would leave it (amd64:
//   StackBase - 40, i.e. a return slot plus 32 bytes of home space).
// - Only callee-saved registers survive a switch, so BaseFiberStartup reads the start
//   routine and parameter back out of the fiber's context (amd64 Rcx/Rdx, arm64 X0/X1, which
//   BaseInitializeContext stored) and tail-calls BaseThreadStartup, as the i386 one does with
//   Eax/Ebx.
// - The callee-saved set is the platform ABI's. amd64: rbx rbp rsi rdi r12-r15, xmm6-xmm15,
//   MXCSR and the x87 control word. arm64: x19-x28, fp, lr, sp, d8-d15 and FPCR. i386 saves
//   its FPU words only for FIBER_FLAG_FLOAT_SWITCH; on amd64 CONTEXT_FULL already includes
//   CONTEXT_FLOATING_POINT, so the flag is not distinguishable here, and MXCSR and the control
//   word are cheap and always switched, as Windows' x64 SwitchToFiber does.
// - Offsets come from the bindgen layouts (offset_of!), never literals.

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod switch64 {
    use core::mem::offset_of;
    use ntdll_sys::nt::{CONTEXT, FIBER, TEB};

    pub const TEB_FIBER: usize = offset_of!(TEB, NtTib.__bindgen_anon_1.FiberData);
    pub const TEB_STACK_BASE: usize = offset_of!(TEB, NtTib.StackBase);
    pub const TEB_STACK_LIMIT: usize = offset_of!(TEB, NtTib.StackLimit);
    pub const TEB_DEALLOC: usize = offset_of!(TEB, DeallocationStack);
    pub const TEB_GSB: usize = offset_of!(TEB, GuaranteedStackBytes);
    pub const TEB_ACTX: usize = offset_of!(TEB, ActivationContextStackPointer);
    pub const TEB_FLS: usize = offset_of!(TEB, FlsData);

    pub const F_STACK_BASE: usize = offset_of!(FIBER, StackBase);
    pub const F_STACK_LIMIT: usize = offset_of!(FIBER, StackLimit);
    pub const F_DEALLOC: usize = offset_of!(FIBER, DeallocationStack);
    pub const F_GSB: usize = offset_of!(FIBER, GuaranteedStackBytes);
    pub const F_ACTX: usize = offset_of!(FIBER, ActivationContextStackPointer);
    pub const F_FLS: usize = offset_of!(FIBER, FlsData);
    pub const F_CTX: usize = offset_of!(FIBER, FiberContext);

    // GuaranteedStackBytes is a ULONG in both; the asm moves it as 32 bits.
    const _: () = assert!(core::mem::size_of::<ntdll_sys::nt::ULONG>() == 4);
    // The fiber's FIELDS the asm reads at F_CTX + a CONTEXT offset.
    const _: () = assert!(F_CTX + core::mem::size_of::<CONTEXT>() <= core::mem::size_of::<FIBER>());

    #[cfg(target_arch = "x86_64")]
    pub mod ctx {
        use super::F_CTX;
        use core::mem::offset_of;
        use ntdll_sys::nt::{CONTEXT, NT_TIB, TEB};
        macro_rules! c { ($($f:tt)+) => { F_CTX + offset_of!(CONTEXT, $($f)+) } }
        pub const TEB_SELF: usize = offset_of!(TEB, NtTib) + offset_of!(NT_TIB, Self_);
        pub const RBX: usize = c!(Rbx);
        pub const RBP: usize = c!(Rbp);
        pub const RSI: usize = c!(Rsi);
        pub const RDI: usize = c!(Rdi);
        pub const R12: usize = c!(R12);
        pub const R13: usize = c!(R13);
        pub const R14: usize = c!(R14);
        pub const R15: usize = c!(R15);
        pub const RSP: usize = c!(Rsp);
        pub const RIP: usize = c!(Rip);
        pub const RCX: usize = c!(Rcx);
        pub const RDX: usize = c!(Rdx);
        pub const MXCSR: usize = c!(MxCsr);
        pub const FPCW: usize = c!(__bindgen_anon_1.FltSave.ControlWord);
        pub const XMM6: usize = c!(__bindgen_anon_1.__bindgen_anon_1.Xmm6);
        pub const XMM15: usize = c!(__bindgen_anon_1.__bindgen_anon_1.Xmm15);
        // xmm6..xmm15 are stored as one run of 16-byte slots.
        const _: () = assert!(XMM15 - XMM6 == 9 * 16);
    }

    #[cfg(target_arch = "aarch64")]
    pub mod ctx {
        use super::F_CTX;
        use core::mem::offset_of;
        use ntdll_sys::nt::CONTEXT;
        macro_rules! c { ($($f:tt)+) => { F_CTX + offset_of!(CONTEXT, $($f)+) } }
        pub const X0: usize = c!(__bindgen_anon_1.__bindgen_anon_1.X0);
        pub const X1: usize = c!(__bindgen_anon_1.__bindgen_anon_1.X1);
        pub const X19: usize = c!(__bindgen_anon_1.__bindgen_anon_1.X19);
        pub const X28: usize = c!(__bindgen_anon_1.__bindgen_anon_1.X28);
        pub const FP: usize = c!(__bindgen_anon_1.__bindgen_anon_1.Fp);
        pub const LR: usize = c!(__bindgen_anon_1.__bindgen_anon_1.Lr);
        pub const SP: usize = c!(Sp);
        pub const PC: usize = c!(Pc);
        /// V[8].Low; V[n] is a 16-byte NEON128, Low its first 8 bytes.
        pub const D8: usize = c!(V) + 8 * 16;
        pub const FPCR: usize = c!(Fpcr);
        // x19..x28, fp, lr are one run of 8-byte slots, so they go as stp/ldp pairs.
        const _: () = assert!(X28 - X19 == 9 * 8 && FP == X28 + 8 && LR == FP + 8);
        const _: () = assert!(core::mem::size_of::<ntdll_sys::nt::NEON128>() == 16);
    }
}

#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "system" fn SwitchToFiber(lpFiber: LPVOID) {
    core::arch::naked_asm!(
        // rdx = TEB, rax = the fiber we are leaving, rcx = the one we enter.
        "mov rdx, gs:[{teb_self}]",
        "mov rax, [rdx + {teb_fiber}]",
        "mov [rax + {rbx}], rbx",
        "mov [rax + {rbp}], rbp",
        "mov [rax + {rsi}], rsi",
        "mov [rax + {rdi}], rdi",
        "mov [rax + {r12}], r12",
        "mov [rax + {r13}], r13",
        "mov [rax + {r14}], r14",
        "mov [rax + {r15}], r15",
        "movups [rax + {xmm6}], xmm6",
        "movups [rax + {xmm6} + 16], xmm7",
        "movups [rax + {xmm6} + 32], xmm8",
        "movups [rax + {xmm6} + 48], xmm9",
        "movups [rax + {xmm6} + 64], xmm10",
        "movups [rax + {xmm6} + 80], xmm11",
        "movups [rax + {xmm6} + 96], xmm12",
        "movups [rax + {xmm6} + 112], xmm13",
        "movups [rax + {xmm6} + 128], xmm14",
        "movups [rax + {xmm6} + 144], xmm15",
        "stmxcsr [rax + {mxcsr}]",
        "fnstcw [rax + {fpcw}]",
        // Resume point: our return address, with the stack as the caller will see it.
        "mov r8, [rsp]",
        "mov [rax + {rip}], r8",
        "lea r8, [rsp + 8]",
        "mov [rax + {rsp}], r8",
        // Per-fiber TEB state the running fiber may have changed.
        "mov r8, [rdx + {teb_fls}]",
        "mov [rax + {f_fls}], r8",
        "mov r8, [rdx + {teb_actx}]",
        "mov [rax + {f_actx}], r8",
        "mov r8, [rdx + {teb_stack_limit}]",
        "mov [rax + {f_stack_limit}], r8",
        "mov r8d, [rdx + {teb_gsb}]",
        "mov [rax + {f_gsb}], r8d",
        // Enter the new fiber's TEB state.
        "mov [rdx + {teb_fiber}], rcx",
        "mov r8, [rcx + {f_stack_base}]",
        "mov [rdx + {teb_stack_base}], r8",
        "mov r8, [rcx + {f_stack_limit}]",
        "mov [rdx + {teb_stack_limit}], r8",
        "mov r8, [rcx + {f_dealloc}]",
        "mov [rdx + {teb_dealloc}], r8",
        "mov r8d, [rcx + {f_gsb}]",
        "mov [rdx + {teb_gsb}], r8d",
        "mov r8, [rcx + {f_actx}]",
        "mov [rdx + {teb_actx}], r8",
        "mov r8, [rcx + {f_fls}]",
        "mov [rdx + {teb_fls}], r8",
        // Its registers.
        "mov rbx, [rcx + {rbx}]",
        "mov rbp, [rcx + {rbp}]",
        "mov rsi, [rcx + {rsi}]",
        "mov rdi, [rcx + {rdi}]",
        "mov r12, [rcx + {r12}]",
        "mov r13, [rcx + {r13}]",
        "mov r14, [rcx + {r14}]",
        "mov r15, [rcx + {r15}]",
        "movups xmm6, [rcx + {xmm6}]",
        "movups xmm7, [rcx + {xmm6} + 16]",
        "movups xmm8, [rcx + {xmm6} + 32]",
        "movups xmm9, [rcx + {xmm6} + 48]",
        "movups xmm10, [rcx + {xmm6} + 64]",
        "movups xmm11, [rcx + {xmm6} + 80]",
        "movups xmm12, [rcx + {xmm6} + 96]",
        "movups xmm13, [rcx + {xmm6} + 112]",
        "movups xmm14, [rcx + {xmm6} + 128]",
        "movups xmm15, [rcx + {xmm6} + 144]",
        "ldmxcsr [rcx + {mxcsr}]",
        "fldcw [rcx + {fpcw}]",
        "mov rsp, [rcx + {rsp}]",
        "jmp qword ptr [rcx + {rip}]",
        teb_self = const switch64::ctx::TEB_SELF,
        teb_fiber = const switch64::TEB_FIBER,
        teb_stack_base = const switch64::TEB_STACK_BASE,
        teb_stack_limit = const switch64::TEB_STACK_LIMIT,
        teb_dealloc = const switch64::TEB_DEALLOC,
        teb_gsb = const switch64::TEB_GSB,
        teb_actx = const switch64::TEB_ACTX,
        teb_fls = const switch64::TEB_FLS,
        f_stack_base = const switch64::F_STACK_BASE,
        f_stack_limit = const switch64::F_STACK_LIMIT,
        f_dealloc = const switch64::F_DEALLOC,
        f_gsb = const switch64::F_GSB,
        f_actx = const switch64::F_ACTX,
        f_fls = const switch64::F_FLS,
        rbx = const switch64::ctx::RBX,
        rbp = const switch64::ctx::RBP,
        rsi = const switch64::ctx::RSI,
        rdi = const switch64::ctx::RDI,
        r12 = const switch64::ctx::R12,
        r13 = const switch64::ctx::R13,
        r14 = const switch64::ctx::R14,
        r15 = const switch64::ctx::R15,
        rsp = const switch64::ctx::RSP,
        rip = const switch64::ctx::RIP,
        mxcsr = const switch64::ctx::MXCSR,
        fpcw = const switch64::ctx::FPCW,
        xmm6 = const switch64::ctx::XMM6,
    );
}

/// A new fiber's first instruction (BaseInitializeContext sets its PC here).
#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "system" fn BaseFiberStartup() {
    core::arch::naked_asm!(
        "mov rax, gs:[{teb_self}]",
        "mov rax, [rax + {teb_fiber}]",
        "mov rcx, [rax + {rcx}]",
        "mov rdx, [rax + {rdx}]",
        "jmp {start}",
        teb_self = const switch64::ctx::TEB_SELF,
        teb_fiber = const switch64::TEB_FIBER,
        rcx = const switch64::ctx::RCX,
        rdx = const switch64::ctx::RDX,
        start = sym crate::client::thread::BaseThreadStartup,
    );
}

#[cfg(target_arch = "aarch64")]
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "system" fn SwitchToFiber(lpFiber: LPVOID) {
    core::arch::naked_asm!(
        // x18 = TEB, x9 = the fiber we are leaving, x0 = the one we enter.
        "ldr x9, [x18, #{teb_fiber}]",
        "add x10, x9, #{x19}",
        "stp x19, x20, [x10, #0]",
        "stp x21, x22, [x10, #16]",
        "stp x23, x24, [x10, #32]",
        "stp x25, x26, [x10, #48]",
        "stp x27, x28, [x10, #64]",
        "stp x29, x30, [x10, #80]",
        "add x10, x9, #{d8}",
        "str d8, [x10, #0]",
        "str d9, [x10, #16]",
        "str d10, [x10, #32]",
        "str d11, [x10, #48]",
        "str d12, [x10, #64]",
        "str d13, [x10, #80]",
        "str d14, [x10, #96]",
        "str d15, [x10, #112]",
        "mrs x11, fpcr",
        "str w11, [x9, #{fpcr}]",
        // Resume point: our return address, with our stack.
        "mov x11, sp",
        "str x11, [x9, #{sp}]",
        "str x30, [x9, #{pc}]",
        // Per-fiber TEB state the running fiber may have changed.
        "ldr x11, [x18, #{teb_fls}]",
        "str x11, [x9, #{f_fls}]",
        "ldr x11, [x18, #{teb_actx}]",
        "str x11, [x9, #{f_actx}]",
        "ldr x11, [x18, #{teb_stack_limit}]",
        "str x11, [x9, #{f_stack_limit}]",
        "ldr w11, [x18, #{teb_gsb}]",
        "str w11, [x9, #{f_gsb}]",
        // Enter the new fiber's TEB state.
        "str x0, [x18, #{teb_fiber}]",
        "ldr x11, [x0, #{f_stack_base}]",
        "str x11, [x18, #{teb_stack_base}]",
        "ldr x11, [x0, #{f_stack_limit}]",
        "str x11, [x18, #{teb_stack_limit}]",
        "ldr x11, [x0, #{f_dealloc}]",
        "str x11, [x18, #{teb_dealloc}]",
        "ldr w11, [x0, #{f_gsb}]",
        "str w11, [x18, #{teb_gsb}]",
        "ldr x11, [x0, #{f_actx}]",
        "str x11, [x18, #{teb_actx}]",
        "ldr x11, [x0, #{f_fls}]",
        "str x11, [x18, #{teb_fls}]",
        // Its registers.
        "add x10, x0, #{x19}",
        "ldp x19, x20, [x10, #0]",
        "ldp x21, x22, [x10, #16]",
        "ldp x23, x24, [x10, #32]",
        "ldp x25, x26, [x10, #48]",
        "ldp x27, x28, [x10, #64]",
        "ldp x29, x30, [x10, #80]",
        "add x10, x0, #{d8}",
        "ldr d8, [x10, #0]",
        "ldr d9, [x10, #16]",
        "ldr d10, [x10, #32]",
        "ldr d11, [x10, #48]",
        "ldr d12, [x10, #64]",
        "ldr d13, [x10, #80]",
        "ldr d14, [x10, #96]",
        "ldr d15, [x10, #112]",
        "ldr w11, [x0, #{fpcr}]",
        "msr fpcr, x11",
        "ldr x11, [x0, #{sp}]",
        "mov sp, x11",
        "ldr x11, [x0, #{pc}]",
        "br x11",
        teb_fiber = const switch64::TEB_FIBER,
        teb_stack_base = const switch64::TEB_STACK_BASE,
        teb_stack_limit = const switch64::TEB_STACK_LIMIT,
        teb_dealloc = const switch64::TEB_DEALLOC,
        teb_gsb = const switch64::TEB_GSB,
        teb_actx = const switch64::TEB_ACTX,
        teb_fls = const switch64::TEB_FLS,
        f_stack_base = const switch64::F_STACK_BASE,
        f_stack_limit = const switch64::F_STACK_LIMIT,
        f_dealloc = const switch64::F_DEALLOC,
        f_gsb = const switch64::F_GSB,
        f_actx = const switch64::F_ACTX,
        f_fls = const switch64::F_FLS,
        x19 = const switch64::ctx::X19,
        d8 = const switch64::ctx::D8,
        fpcr = const switch64::ctx::FPCR,
        sp = const switch64::ctx::SP,
        pc = const switch64::ctx::PC,
    );
}

/// A new fiber's first instruction (BaseInitializeContext sets its PC here).
#[cfg(target_arch = "aarch64")]
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "system" fn BaseFiberStartup() {
    core::arch::naked_asm!(
        "ldr x9, [x18, #{teb_fiber}]",
        "ldr x0, [x9, #{x0}]",
        "ldr x1, [x9, #{x1}]",
        "b {start}",
        teb_fiber = const switch64::TEB_FIBER,
        x0 = const switch64::ctx::X0,
        x1 = const switch64::ctx::X1,
        start = sym crate::client::thread::BaseThreadStartup,
    );
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS System Libraries
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Fiber Implementation
//  * COPYRIGHT:   Copyright 2005-2011 Alex Ionescu (alex@relsoft.net)
//  *              Copyright 2003-2008 KJK::Hyperion (noog@libero.it)
//  *              Copyright 2018 Mark Jansen (mark.jansen@reactos.org)
//  */
//
// #include <k32.h>
// #include <ndk/rtltypes.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #ifdef _M_IX86
// C_ASSERT(FIELD_OFFSET(FIBER, ExceptionList) == 0x04);
// C_ASSERT(FIELD_OFFSET(FIBER, StackBase) == 0x08);
// C_ASSERT(FIELD_OFFSET(FIBER, StackLimit) == 0x0C);
// C_ASSERT(FIELD_OFFSET(FIBER, DeallocationStack) == 0x10);
// C_ASSERT(FIELD_OFFSET(FIBER, FiberContext) == 0x14);
// C_ASSERT(FIELD_OFFSET(FIBER, GuaranteedStackBytes) == 0x2E0);
// C_ASSERT(FIELD_OFFSET(FIBER, FlsData) == 0x2E4);
// C_ASSERT(FIELD_OFFSET(FIBER, ActivationContextStackPointer) == 0x2E8);
// C_ASSERT(RTL_FLS_MAXIMUM_AVAILABLE == FLS_MAXIMUM_AVAILABLE);
// #endif // _M_IX86
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// VOID
// WINAPI
// BaseRundownFls( PVOID FlsData)
// {
//     ULONG n, FlsHighIndex;
//     PRTL_FLS_DATA pFlsData;
//     PFLS_CALLBACK_FUNCTION lpCallback;
//
//     pFlsData = FlsData;
//
//     RtlAcquirePebLock();
//     FlsHighIndex = NtCurrentPeb()->FlsHighIndex;
//     RemoveEntryList(&pFlsData->ListEntry);
//     RtlReleasePebLock();
//
//     for (n = 1; n <= FlsHighIndex; ++n)
//     {
//         lpCallback = NtCurrentPeb()->FlsCallback[n];
//         if (lpCallback && pFlsData->Data[n])
//         {
//             lpCallback(pFlsData->Data[n]);
//         }
//     }
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, FlsData);
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ConvertFiberToThread(VOID)
// {
//     PTEB Teb;
//     PFIBER FiberData;
//     DPRINT1("Converting Fiber to Thread\n");
//
//     /* Check if the thread is already not a fiber */
//     Teb = NtCurrentTeb();
//     if (!Teb->HasFiberData)
//     {
//         /* Fail */
//         SetLastError(ERROR_ALREADY_THREAD);
//         return FALSE;
//     }
//
//     /* This thread won't run a fiber anymore */
//     Teb->HasFiberData = FALSE;
//     FiberData = Teb->NtTib.FiberData;
//     Teb->NtTib.FiberData = NULL;
//
//     /* Free the fiber */
//     ASSERT(FiberData != NULL);
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 FiberData);
//
//     /* Success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// WINAPI
// ConvertThreadToFiberEx( LPVOID lpParameter,
//                         DWORD dwFlags)
// {
//     PTEB Teb;
//     PFIBER Fiber;
//     DPRINT1("Converting Thread to Fiber\n");
//
//     /* Check for invalid flags */
//     if (dwFlags & ~FIBER_FLAG_FLOAT_SWITCH)
//     {
//         /* Fail */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     /* Are we already a fiber? */
//     Teb = NtCurrentTeb();
//     if (Teb->HasFiberData)
//     {
//         /* Fail */
//         SetLastError(ERROR_ALREADY_FIBER);
//         return NULL;
//     }
//
//     /* Allocate the fiber */
//     Fiber = RtlAllocateHeap(RtlGetProcessHeap(),
//                             0,
//                             sizeof(FIBER));
//     if (!Fiber)
//     {
//         /* Fail */
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return NULL;
//     }
//
//     /* Copy some contextual data from the thread to the fiber */
//     Fiber->FiberData = lpParameter;
//     Fiber->ExceptionList = Teb->NtTib.ExceptionList;
//     Fiber->StackBase = Teb->NtTib.StackBase;
//     Fiber->StackLimit = Teb->NtTib.StackLimit;
//     Fiber->DeallocationStack = Teb->DeallocationStack;
//     Fiber->FlsData = Teb->FlsData;
//     Fiber->GuaranteedStackBytes = Teb->GuaranteedStackBytes;
//     Fiber->ActivationContextStackPointer = Teb->ActivationContextStackPointer;
//
//     /* Save FPU State if requested, otherwise just the basic registers */
//     Fiber->FiberContext.ContextFlags = (dwFlags & FIBER_FLAG_FLOAT_SWITCH) ?
//                                        (CONTEXT_FULL | CONTEXT_FLOATING_POINT) :
//                                        CONTEXT_FULL;
//
//     /* Associate the fiber to the current thread */
//     Teb->NtTib.FiberData = Fiber;
//     Teb->HasFiberData = TRUE;
//
//     /* Return opaque fiber data */
//     return (LPVOID)Fiber;
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// WINAPI
// ConvertThreadToFiber( LPVOID lpParameter)
// {
//     /* Call the newer function */
//     return ConvertThreadToFiberEx(lpParameter,
//                                   0);
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// WINAPI
// CreateFiber( SIZE_T dwStackSize,
//              LPFIBER_START_ROUTINE lpStartAddress,
//              LPVOID lpParameter)
// {
//     /* Call the Newer Function */
//     return CreateFiberEx(dwStackSize,
//                          0,
//                          0,
//                          lpStartAddress,
//                          lpParameter);
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// WINAPI
// CreateFiberEx( SIZE_T dwStackCommitSize,
//                SIZE_T dwStackReserveSize,
//                DWORD dwFlags,
//                LPFIBER_START_ROUTINE lpStartAddress,
//                LPVOID lpParameter)
// {
//     PFIBER Fiber;
//     NTSTATUS Status;
//     INITIAL_TEB InitialTeb;
//     PACTIVATION_CONTEXT_STACK ActivationContextStackPointer;
//     DPRINT("Creating Fiber\n");
//
//     /* Check for invalid flags */
//     if (dwFlags & ~FIBER_FLAG_FLOAT_SWITCH)
//     {
//         /* Fail */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     /* Allocate the Activation Context Stack */
//     ActivationContextStackPointer = NULL;
//     Status = RtlAllocateActivationContextStack(&ActivationContextStackPointer);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Allocate the fiber */
//     Fiber = RtlAllocateHeap(RtlGetProcessHeap(),
//                             0,
//                             sizeof(FIBER));
//     if (!Fiber)
//     {
//         /* Free the activation context stack */
//         RtlFreeActivationContextStack(ActivationContextStackPointer);
//
//         /* Fail */
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return NULL;
//     }
//
//     /* Create the stack for the fiber */
//     Status = BaseCreateStack(NtCurrentProcess(),
//                               dwStackCommitSize,
//                               dwStackReserveSize,
//                               &InitialTeb);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Free the fiber */
//         RtlFreeHeap(GetProcessHeap(),
//                     0,
//                     Fiber);
//
//         /* Free the activation context stack */
//         RtlFreeActivationContextStack(ActivationContextStackPointer);
//
//         /* Failure */
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Clear the context */
//     RtlZeroMemory(&Fiber->FiberContext,
//                   sizeof(CONTEXT));
//
//     /* Copy the data into the fiber */
//     Fiber->StackBase = InitialTeb.StackBase;
//     Fiber->StackLimit = InitialTeb.StackLimit;
//     Fiber->DeallocationStack = InitialTeb.AllocatedStackBase;
//     Fiber->FiberData = lpParameter;
//     Fiber->ExceptionList = EXCEPTION_CHAIN_END;
//     Fiber->GuaranteedStackBytes = 0;
//     Fiber->FlsData = NULL;
//     Fiber->ActivationContextStackPointer = ActivationContextStackPointer;
//
//     /* Save FPU State if requested, otherwise just the basic registers */
//     Fiber->FiberContext.ContextFlags = (dwFlags & FIBER_FLAG_FLOAT_SWITCH) ?
//                                        (CONTEXT_FULL | CONTEXT_FLOATING_POINT) :
//                                        CONTEXT_FULL;
//
//     /* Initialize the context for the fiber */
//     BaseInitializeContext(&Fiber->FiberContext,
//                           lpParameter,
//                           lpStartAddress,
//                           InitialTeb.StackBase,
//                           2);
//
//     /* Return the Fiber */
//     return Fiber;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// DeleteFiber( LPVOID lpFiber)
// {
//     SIZE_T Size;
//     PFIBER Fiber;
//     PTEB Teb;
//
//     /* Are we deleting ourselves? */
//     Teb = NtCurrentTeb();
//     Fiber = (PFIBER)lpFiber;
//     if ((Teb->HasFiberData) &&
//         (Teb->NtTib.FiberData == Fiber))
//     {
//         /* Just exit */
//         ExitThread(1);
//     }
//
//     /* Not ourselves, de-allocate the stack */
//     Size = 0 ;
//     NtFreeVirtualMemory(NtCurrentProcess(),
//                         &Fiber->DeallocationStack,
//                         &Size,
//                         MEM_RELEASE);
//
//     /* Get rid of FLS */
//     if (Fiber->FlsData) BaseRundownFls(Fiber->FlsData);
//
//     /* Get rid of the activation context stack */
//     RtlFreeActivationContextStack(Fiber->ActivationContextStackPointer);
//
//     /* Free the fiber data */
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 lpFiber);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsThreadAFiber(VOID)
// {
//     /* Return flag in the TEB */
//     return NtCurrentTeb()->HasFiberData;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// FlsAlloc(PFLS_CALLBACK_FUNCTION lpCallback)
// {
//     DWORD dwFlsIndex;
//     PPEB Peb = NtCurrentPeb();
//     PRTL_FLS_DATA pFlsData;
//
//     RtlAcquirePebLock();
//
//     pFlsData = NtCurrentTeb()->FlsData;
//
//     if (!Peb->FlsCallback &&
//         !(Peb->FlsCallback = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY,
//                                              FLS_MAXIMUM_AVAILABLE * sizeof(PVOID))))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         dwFlsIndex = FLS_OUT_OF_INDEXES;
//     }
//     else
//     {
//         dwFlsIndex = RtlFindClearBitsAndSet(Peb->FlsBitmap, 1, 1);
//         if (dwFlsIndex != FLS_OUT_OF_INDEXES)
//         {
//             if (!pFlsData &&
//                 !(pFlsData = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(RTL_FLS_DATA))))
//             {
//                 RtlClearBits(Peb->FlsBitmap, dwFlsIndex, 1);
//                 dwFlsIndex = FLS_OUT_OF_INDEXES;
//                 SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             }
//             else
//             {
//                 if (!NtCurrentTeb()->FlsData)
//                 {
//                     NtCurrentTeb()->FlsData = pFlsData;
//                     InsertTailList(&Peb->FlsListHead, &pFlsData->ListEntry);
//                 }
//
//                 pFlsData->Data[dwFlsIndex] = NULL; /* clear the value */
//                 Peb->FlsCallback[dwFlsIndex] = lpCallback;
//
//                 if (dwFlsIndex > Peb->FlsHighIndex)
//                     Peb->FlsHighIndex = dwFlsIndex;
//             }
//         }
//         else
//         {
//             SetLastError(ERROR_NO_MORE_ITEMS);
//         }
//     }
//     RtlReleasePebLock();
//     return dwFlsIndex;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FlsFree(DWORD dwFlsIndex)
// {
//     BOOL ret;
//     PPEB Peb = NtCurrentPeb();
//
//     if (dwFlsIndex >= FLS_MAXIMUM_AVAILABLE)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     RtlAcquirePebLock();
//
//     _SEH2_TRY
//     {
//         ret = RtlAreBitsSet(Peb->FlsBitmap, dwFlsIndex, 1);
//         if (ret)
//         {
//             PLIST_ENTRY Entry;
//             PFLS_CALLBACK_FUNCTION lpCallback;
//
//             RtlClearBits(Peb->FlsBitmap, dwFlsIndex, 1);
//             lpCallback = Peb->FlsCallback[dwFlsIndex];
//
//             for (Entry = Peb->FlsListHead.Flink; Entry != &Peb->FlsListHead; Entry = Entry->Flink)
//             {
//                 PRTL_FLS_DATA pFlsData;
//
//                 pFlsData = CONTAINING_RECORD(Entry, RTL_FLS_DATA, ListEntry);
//                 if (pFlsData->Data[dwFlsIndex])
//                 {
//                     if (lpCallback)
//                     {
//                         lpCallback(pFlsData->Data[dwFlsIndex]);
//                     }
//                     pFlsData->Data[dwFlsIndex] = NULL;
//                 }
//             }
//             Peb->FlsCallback[dwFlsIndex] = NULL;
//         }
//         else
//         {
//             SetLastError(ERROR_INVALID_PARAMETER);
//         }
//     }
//     _SEH2_FINALLY
//     {
//         RtlReleasePebLock();
//     }
//     _SEH2_END;
//
//     return ret;
// }
//
//
// /*
//  * @implemented
//  */
// PVOID
// WINAPI
// FlsGetValue(DWORD dwFlsIndex)
// {
//     PRTL_FLS_DATA pFlsData;
//
//     pFlsData = NtCurrentTeb()->FlsData;
//     if (!dwFlsIndex || dwFlsIndex >= FLS_MAXIMUM_AVAILABLE || !pFlsData)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     SetLastError(ERROR_SUCCESS);
//     return pFlsData->Data[dwFlsIndex];
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FlsSetValue(DWORD dwFlsIndex,
//             PVOID lpFlsData)
// {
//     PRTL_FLS_DATA pFlsData;
//
//     if (!dwFlsIndex || dwFlsIndex >= FLS_MAXIMUM_AVAILABLE)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     pFlsData = NtCurrentTeb()->FlsData;
//
//     if (!NtCurrentTeb()->FlsData &&
//         !(NtCurrentTeb()->FlsData = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY,
//                                                     sizeof(RTL_FLS_DATA))))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//     if (!pFlsData)
//     {
//         pFlsData = NtCurrentTeb()->FlsData;
//         RtlAcquirePebLock();
//         InsertTailList(&NtCurrentPeb()->FlsListHead, &pFlsData->ListEntry);
//         RtlReleasePebLock();
//     }
//     pFlsData->Data[dwFlsIndex] = lpFlsData;
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
