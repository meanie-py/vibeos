use crate::ke::ke_defs::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::{STATUS_ACCESS_VIOLATION, STATUS_BREAKPOINT};
use ntkrnl_sys::ke::*;
use ntkrnl_sys::macros::NtCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;

use ntkrnl_sys::ddk::{
    CONTEXT,
    EXCEPTION_RECORD,
    KPROCESSOR_MODE,
    PCONTEXT,
    PEXCEPTION_RECORD,
    PKEXCEPTION_FRAME,
    PKTRAP_FRAME,
    PTEB,
    PLDR_DATA_TABLE_ENTRY,
    PPEB_LDR_DATA,
    LDR_DATA_TABLE_ENTRY,
    _EPROCESS,
    PEPROCESS,
    DbgkForwardException,
    KeBugCheckWithTf,
    KeContextToTrapFrame,
    KeGetPcr,
    KeTrapFrameToContext,
    KiApcInterrupt,
    RtlDispatchException,
    RtlImageDirectoryEntryToData,
    MmAccessFault,
    MmIsAddressValid,
    ZwTerminateProcess,
};

extern "system" {
    static mut KiUnexpectedRange: [KI_INTERRUPT_DISPATCH_ENTRY; 256];
    static mut KiDebugRoutine: unsafe extern "system" fn( PKTRAP_FRAME, PKEXCEPTION_FRAME, PEXCEPTION_RECORD, PCONTEXT, KPROCESSOR_MODE, BOOLEAN, ) -> BOOLEAN;
    static mut KdIgnoreUmExceptions: BOOLEAN;
    // NtCurrentProcess is a C macro: ((HANDLE)(LONG_PTR)-1), not an extern symbol
    // static mut NtCurrentProcess: PVOID;
    static mut KeUserExceptionDispatcher: PVOID;
    fn KiDivideErrorFault();
    fn KiDebugTrapOrFault();
    fn KiNmiInterrupt();
    fn KiBreakpointTrap();
    fn KiOverflowTrap();
    fn KiBoundFault();
    fn KiInvalidOpcodeFault();
    fn KiNpxNotAvailableFault();
    fn KiDoubleFaultAbort();
    fn KiNpxSegmentOverrunAbort();
    fn KiInvalidTssFault();
    fn KiSegmentNotPresentFault();
    fn KiStackFault();
    fn KiGeneralProtectionFault();
    fn KiPageFault();
    fn KiFloatingErrorFault();
    fn KiAlignmentFault();
    fn KiMcheckAbort();
    fn KiXmmException();
    fn KiRaiseAssertion();
    fn KiDebugServiceTrap();
    fn KiDpcInterrupt();
    fn KiIpiInterrupt();
    fn KdIsThisAKdTrap( ExceptionRecord: PEXCEPTION_RECORD, Context: PCONTEXT, Mode: KPROCESSOR_MODE, ) -> BOOLEAN;
    fn _enable();
    fn _disable();
}
extern "C" {
    fn __lidt(descriptor: *const u16);
}

const KGDT64_R0_CODE: u16 = 0x10;
const KGDT64_R3_CMCODE: u16 = 0x20;
const KGDT64_R3_DATA: u16 = 0x28;
const KGDT64_R3_CODE: u16 = 0x30;
const KGDT64_R3_CMTEB: u16 = 0x50;
const RPL_MASK: u16 = 3;

const CONTEXT_ALL: ULONG = 0x10001f;
const KI_EXCEPTION_ACCESS_VIOLATION: NTSTATUS = 0x10000004u32 as NTSTATUS;
const KMODE_EXCEPTION_NOT_HANDLED: ULONG = 0x1E;
const UNEXPECTED_KERNEL_MODE_TRAP: ULONG = 0x7F;
const TRAP_CAUSE_UNKNOWN: ULONG = 0x12;
const IMAGE_DIRECTORY_ENTRY_DEBUG: USHORT = 6;
const IMAGE_DIRECTORY_ENTRY_EXCEPTION: USHORT = 3;

const _MM_EXCEPT_INVALID: ULONG = 0x0001;
const _MM_EXCEPT_DENORM: ULONG = 0x0002;
const _MM_EXCEPT_DIV_ZERO: ULONG = 0x0004;
const _MM_EXCEPT_OVERFLOW: ULONG = 0x0008;
const _MM_EXCEPT_UNDERFLOW: ULONG = 0x0010;
const _MM_EXCEPT_INEXACT: ULONG = 0x0020;
const _MM_MASK_INVALID: ULONG = 0x0080;
const _MM_MASK_DENORM: ULONG = 0x0100;
const _MM_MASK_DIV_ZERO: ULONG = 0x0200;
const _MM_MASK_OVERFLOW: ULONG = 0x0400;
const _MM_MASK_UNDERFLOW: ULONG = 0x0800;
const _MM_MASK_INEXACT: ULONG = 0x1000;

#[repr(C)]
struct KIDT_INIT {
    InterruptId: UCHAR,
    Dpl: UCHAR,
    IstIndex: UCHAR,
    ServiceRoutine: PVOID,
}

/// AMD64 IDT gate descriptor — must be exactly 16 bytes to match hardware layout.
/// The C original packs IstIndex/Type/Dpl/Present into a single USHORT bitfield.
#[repr(C, packed)]
#[derive(Copy, Clone)]
struct KIDTENTRY64 {
    OffsetLow: USHORT,      // offset 0
    Selector: USHORT,       // offset 2
    Bits: USHORT,           // offset 4: IstIndex[2:0] Reserved0[7:3] Type[12:8] Dpl[14:13] Present[15]
    OffsetMiddle: USHORT,   // offset 6
    OffsetHigh: ULONG,      // offset 8
    Reserved1: ULONG,       // offset 12
}
// Total: 16 bytes — matches hardware IDT entry size

impl KIDTENTRY64 {
    fn set_ist_index(&mut self, val: u8)  { self.Bits = (self.Bits & !0x0007) | ((val as u16) & 0x07); }
    fn set_type(&mut self, val: u8)       { self.Bits = (self.Bits & !0x1F00) | (((val as u16) & 0x1F) << 8); }
    fn set_dpl(&mut self, val: u8)        { self.Bits = (self.Bits & !0x6000) | (((val as u16) & 0x03) << 13); }
    fn set_present(&mut self, val: u8)    { self.Bits = (self.Bits & !0x8000) | (((val as u16) & 0x01) << 15); }
}

#[repr(C)]
struct KDESCRIPTOR {
    Pad: [USHORT; 3],
    Limit: USHORT,
    Base: *mut KIDTENTRY64,
}

#[repr(C, packed)]
struct KI_INTERRUPT_DISPATCH_ENTRY {
    _Op_nop: UCHAR,
    _Op_push: UCHAR,
    _Vector: UCHAR,
    _Op_jmp: UCHAR,
    RelativeAddress: ULONG,
}
const _: () = assert!(core::mem::size_of::<KI_INTERRUPT_DISPATCH_ENTRY>() == 8);

#[repr(C)]
struct MACHINE_FRAME {
    Rip: u64,
    SegCs: USHORT,
    Fill1: [USHORT; 3],
    EFlags: ULONG,
    Fill2: ULONG,
    Rsp: u64,
    SegSs: USHORT,
    Fill3: [USHORT; 3],
}

#[repr(C)]
struct KUSER_EXCEPTION_STACK {
    Context: CONTEXT,
    ExceptionRecord: EXCEPTION_RECORD,
    Alignment: u64,
    MachineFrame: MACHINE_FRAME,
}

type PKUSER_EXCEPTION_STACK = *mut KUSER_EXCEPTION_STACK;

static mut KiInterruptInitTable: [KIDT_INIT; 25] = unsafe {
    [
        KIDT_INIT { InterruptId: 0x00, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiDivideErrorFault as PVOID },
        KIDT_INIT { InterruptId: 0x01, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiDebugTrapOrFault as PVOID },
        KIDT_INIT { InterruptId: 0x02, Dpl: 0x00, IstIndex: 0x03, ServiceRoutine: KiNmiInterrupt as PVOID },
        KIDT_INIT { InterruptId: 0x03, Dpl: 0x03, IstIndex: 0x00, ServiceRoutine: KiBreakpointTrap as PVOID },
        KIDT_INIT { InterruptId: 0x04, Dpl: 0x03, IstIndex: 0x00, ServiceRoutine: KiOverflowTrap as PVOID },
        KIDT_INIT { InterruptId: 0x05, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiBoundFault as PVOID },
        KIDT_INIT { InterruptId: 0x06, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiInvalidOpcodeFault as PVOID },
        KIDT_INIT { InterruptId: 0x07, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiNpxNotAvailableFault as PVOID },
        KIDT_INIT { InterruptId: 0x08, Dpl: 0x00, IstIndex: 0x01, ServiceRoutine: KiDoubleFaultAbort as PVOID },
        KIDT_INIT { InterruptId: 0x09, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiNpxSegmentOverrunAbort as PVOID },
        KIDT_INIT { InterruptId: 0x0A, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiInvalidTssFault as PVOID },
        KIDT_INIT { InterruptId: 0x0B, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiSegmentNotPresentFault as PVOID },
        KIDT_INIT { InterruptId: 0x0C, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiStackFault as PVOID },
        KIDT_INIT { InterruptId: 0x0D, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiGeneralProtectionFault as PVOID },
        KIDT_INIT { InterruptId: 0x0E, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiPageFault as PVOID },
        KIDT_INIT { InterruptId: 0x10, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiFloatingErrorFault as PVOID },
        KIDT_INIT { InterruptId: 0x11, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiAlignmentFault as PVOID },
        KIDT_INIT { InterruptId: 0x12, Dpl: 0x00, IstIndex: 0x02, ServiceRoutine: KiMcheckAbort as PVOID },
        KIDT_INIT { InterruptId: 0x13, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiXmmException as PVOID },
        KIDT_INIT { InterruptId: 0x1F, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiApcInterrupt as PVOID },
        KIDT_INIT { InterruptId: 0x2C, Dpl: 0x03, IstIndex: 0x00, ServiceRoutine: KiRaiseAssertion as PVOID },
        KIDT_INIT { InterruptId: 0x2D, Dpl: 0x03, IstIndex: 0x00, ServiceRoutine: KiDebugServiceTrap as PVOID },
        KIDT_INIT { InterruptId: 0x2F, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiDpcInterrupt as PVOID },
        KIDT_INIT { InterruptId: 0xE1, Dpl: 0x00, IstIndex: 0x00, ServiceRoutine: KiIpiInterrupt as PVOID },
        KIDT_INIT { InterruptId: 0, Dpl: 0, IstIndex: 0, ServiceRoutine: core::ptr::null_mut() },
    ]
};

static mut KiIdt: [KIDTENTRY64; 256] = unsafe { core::mem::zeroed() };

static mut KiIdtDescriptor: KDESCRIPTOR = KDESCRIPTOR {
    Pad: [0; 3],
    Limit: (core::mem::size_of::<[KIDTENTRY64; 256]>() - 1) as USHORT,
    Base: unsafe { core::ptr::addr_of_mut!(KiIdt) as *mut KIDTENTRY64 },
};

#[no_mangle]
pub unsafe extern "system" fn KeInitExceptions() {
    let mut j = 0usize;
    
    for i in 0..256 {
        let offset: u64;
        let dpl: u8;
        let ist: u8;

        if KiInterruptInitTable[j].InterruptId as usize == i {
            offset = KiInterruptInitTable[j].ServiceRoutine as u64;
            dpl = KiInterruptInitTable[j].Dpl;
            ist = KiInterruptInitTable[j].IstIndex;
            j += 1;
        } else {
            offset = core::ptr::addr_of!(KiUnexpectedRange[i]._Op_push) as u64;
            dpl = 0;
            ist = 0;
        }

        KiIdt[i].OffsetLow = (offset & 0xffff) as USHORT;
        KiIdt[i].Selector = KGDT64_R0_CODE;
        // Pack bitfield: IstIndex[2:0] | Type[12:8]=0xE (interrupt gate) | Dpl[14:13] | Present[15]=1
        KiIdt[i].Bits = ((ist as u16) & 0x07)
            | (0x0Eu16 << 8)
            | (((dpl as u16) & 0x03) << 13)
            | (1u16 << 15);
        KiIdt[i].OffsetMiddle = ((offset >> 16) & 0xffff) as USHORT;
        KiIdt[i].OffsetHigh = (offset >> 32) as ULONG;
        KiIdt[i].Reserved1 = 0;
    }
    
    (*KeGetPcr()).IdtBase = core::ptr::addr_of_mut!(KiIdt) as *mut _;
    __lidt(core::ptr::addr_of!(KiIdtDescriptor.Limit));
}

unsafe fn KiDispatchExceptionToUser(
    TrapFrame: PKTRAP_FRAME,
    Context: PCONTEXT,
    ExceptionRecord: PEXCEPTION_RECORD,
) -> BOOLEAN {
    let mut LocalExceptRecord: EXCEPTION_RECORD = core::mem::zeroed();
    let mut UserRsp: u64;
    let UserStack: PKUSER_EXCEPTION_STACK;
    let mut ExceptionRecordPtr = ExceptionRecord;
    
    if (*TrapFrame).SegSs != (KGDT64_R3_DATA | RPL_MASK) {
        LocalExceptRecord.ExceptionCode = STATUS_ACCESS_VIOLATION;
        LocalExceptRecord.ExceptionFlags = 0;
        LocalExceptRecord.NumberParameters = 0;
        ExceptionRecordPtr = core::ptr::addr_of_mut!(LocalExceptRecord);
    }
    
    UserRsp = ((*Context).Rsp - core::mem::size_of::<KUSER_EXCEPTION_STACK>() as u64) & !15u64;

    // A STACK OVERFLOW cannot be reported on the stack that just overflowed,
    // and until this clamp it structurally never was.
    //
    // MiCheckForUserStackOverflow commits the GUARANTEED page at
    // DeallocationStack + PAGE_SIZE, points NtTib.StackLimit at it and raises
    // STATUS_STACK_OVERFLOW. The last page, [DeallocationStack, StackLimit),
    // is the hard floor and is never committed. But the record goes BELOW the
    // faulting Rsp -- which by definition is already at the bottom -- so it
    // lands in that uncommitted floor every time. The pre-fault below then
    // fails with 0xc0000005, dispatch aborts, and csrss terminates the process
    // with 0x80, laundering the cause. MEASURED: rsp=0x60920, limit=0x61000,
    // dealloc=0x60000 -- the record wanted 0x60360, one page under the lowest
    // committed byte.
    //
    // So place it at StackLimit when it would fall below: that IS the page the
    // guarantee exists to provide, the thread has no live data down there (it
    // faulted trying to get there), and a handler that runs on the guaranteed
    // page is the entire point of guaranteeing one. Only when even that does
    // not fit is the exception genuinely undeliverable.
    {
        let teb = (*PsGetCurrentThread()).Teb as PTEB;
        if !teb.is_null() {
            let limit = (*teb).NtTib.StackLimit as u64;
            let base = (*teb).NtTib.StackBase as u64;
            let size = core::mem::size_of::<KUSER_EXCEPTION_STACK>() as u64;
            // The `limit + size <= Rsp` half is LOAD-BEARING and its absence
            // was a livelock. Without it the clamp also fires when Rsp is
            // already AT StackLimit -- there is then no committed room below,
            // so the frame is placed back at the address that just faulted,
            // the handler re-overflows and re-raises, forever. Measured at
            // 273,430 iterations in one boot (rsp=limit=0x61000), which read
            // as "no failed dispatches" because dispatch never got to fail.
            // Clamp only where doing so yields real room.
            if limit != 0
                && UserRsp < limit
                && limit.saturating_add(size) <= base
                && limit.saturating_add(size) <= (*Context).Rsp
            {
                UserRsp = (limit + 15) & !15u64;
            }
        }
    }

    UserStack = UserRsp as PKUSER_EXCEPTION_STACK;
    
    _enable();
    
    let result = 'seh: {
        if let Err(e) = (|| -> Result<(), NTSTATUS> {
            use ntkrnl_sys::probe::*;
            probe_user_buffer(
                UserStack as *const core::ffi::c_void,
                core::mem::size_of::<KUSER_EXCEPTION_STACK>(),
                core::mem::align_of::<KUSER_EXCEPTION_STACK>(),
                1i8,
            )?;
            Ok(())
        })() {
            // Fail-loud: a rejected dispatch here IS the process death the
            // second-chance block below reports as a bare [EXIT-THR] 0x80.
            crate::dbg_print!(
                b"[KI-EXC] user dispatch FAILED: exception-stack probe %p size 0x%lx status 0x%lx\n\0",
                UserStack as usize,
                core::mem::size_of::<KUSER_EXCEPTION_STACK>() as u32,
                e as u32,
            );
            break 'seh 0u8;
        }

        // DUMMY_PSEH: SEH can't catch page faults from writing to the user
        // stack. Pre-fault each page via MmAccessFault (simulating a write
        // fault in user mode). If any page can't be committed, bail out.
        let stack_size = core::mem::size_of::<KUSER_EXCEPTION_STACK>();
        {
            let start = UserStack as usize;
            let end = start + stack_size;
            let mut page = start & !0xFFF;
            while page < end {
                // FaultCode=2 (write, not present), Mode=UserMode(1)
                let status = MmAccessFault(2, page as PVOID, 1i8, core::ptr::null_mut());
                if status < 0 {
                    // Fail-loud: see the probe-failure print above. The bounds
                    // are part of the evidence, not decoration: this same line
                    // is printed both when the thread has genuinely run out of
                    // stack and when MiCheckForUserStackOverflow was handed
                    // bounds it could not honour, and without StackLimit /
                    // DeallocationStack next to the faulting page the two are
                    // indistinguishable -- which is how a whole boot's worth of
                    // guests died at one address with no way to tell why.
                    let teb = (*PsGetCurrentThread()).Teb as PTEB;
                    let (sb, sl, ds) = if teb.is_null() {
                        (0usize, 0usize, 0usize)
                    } else {
                        (
                            (*teb).NtTib.StackBase as usize,
                            (*teb).NtTib.StackLimit as usize,
                            (*teb).DeallocationStack as usize,
                        )
                    };
                    if (*ExceptionRecordPtr).ExceptionCode == STATUS_STACK_OVERFLOW
                        || (sb != 0 && (*Context).Rsp <= sl as u64)
                    {
                        // Name it. Otherwise this is the anonymous 0x80 that
                        // made every stack bug in this system unattributable:
                        // the thread is out of stack AND out of the room the
                        // guarantee was supposed to leave for reporting it.
                        crate::dbg_print!(
                            b"[KI-EXC] STACK EXHAUSTED and undeliverable: no committed room for the exception record (rsp=%p limit=%p dealloc=%p)\n\0",
                            (*Context).Rsp as usize,
                            sl,
                            ds,
                        );
                    }
                    crate::dbg_print!(
                        b"[KI-EXC] user dispatch FAILED: pre-fault of stack page %p = 0x%lx (rsp=%p base=%p limit=%p dealloc=%p)\n\0",
                        page,
                        status as u32,
                        (*Context).Rsp as usize,
                        sb,
                        sl,
                        ds,
                    );
                    break 'seh 0u8;
                }
                page += 0x1000;
            }
        }

        core::ptr::write_bytes(UserStack as *mut u8, 0, stack_size);

        (*UserStack).Context = *Context;
        (*UserStack).ExceptionRecord = *ExceptionRecordPtr;

        (*UserStack).MachineFrame.Rip = (*Context).Rip;
        (*UserStack).MachineFrame.SegCs = (*Context).SegCs;
        (*UserStack).MachineFrame.EFlags = (*Context).EFlags;
        (*UserStack).MachineFrame.Rsp = (*Context).Rsp;
        (*UserStack).MachineFrame.SegSs = (*Context).SegSs;

        1u8
    };
    
    if result == 0 {
        _disable();
        return 0u8;
    }
    
    (*TrapFrame).Rcx = core::ptr::addr_of_mut!((*UserStack).Context) as u64;
    (*TrapFrame).Rdx = core::ptr::addr_of_mut!((*UserStack).ExceptionRecord) as u64;
    
    (*TrapFrame).Rsp = UserRsp;
    
    (*TrapFrame).SegCs = KGDT64_R3_CODE | RPL_MASK;
    (*TrapFrame).SegDs = KGDT64_R3_DATA | RPL_MASK;
    (*TrapFrame).SegEs = KGDT64_R3_DATA | RPL_MASK;
    (*TrapFrame).SegFs = KGDT64_R3_CMTEB | RPL_MASK;
    (*TrapFrame).SegGs = KGDT64_R3_DATA | RPL_MASK;
    (*TrapFrame).SegSs = KGDT64_R3_DATA | RPL_MASK;
    
    (*TrapFrame).Rip = KeUserExceptionDispatcher as u64;
    
    _disable();
    
    1u8
}

unsafe fn KiPageInDirectory(ImageBase: PVOID, Directory: USHORT) {
    // DUMMY_PSEH: validate ImageBase before passing to RtlImageDirectoryEntryToData
    // since SEH can't catch page faults
    if ImageBase.is_null() || MmIsAddressValid(ImageBase) == 0 {
        return;
    }

    let mut Size: ULONG = 0;

    let Pointer = RtlImageDirectoryEntryToData(ImageBase, 1, Directory, core::ptr::addr_of_mut!(Size));
    if Pointer.is_null() {
        return;
    }

    let mut ptr = Pointer as *const u8;
    let mut size = Size as LONG;

    while size > 0 {
        // DUMMY_PSEH: check page is accessible before touching
        if MmIsAddressValid(ptr as PVOID) != 0 {
            let _touch = *ptr;
        }
        ptr = ptr.add(PAGE_SIZE);
        size -= PAGE_SIZE as LONG;
    }
}

unsafe fn KiPrepareUserDebugData() {
    let Teb = (*PsGetCurrentThread()).Teb;
    if Teb.is_null() {
        return;
    }

    _enable();

    // DUMMY_PSEH: The C code wraps this in __try/__except, but with DUMMY_PSEH
    // the handler is dead code. Validate every user-mode pointer with
    // MmIsAddressValid before dereferencing to avoid kernel-mode ACCESS_VIOLATION.
    'done: {
        if MmIsAddressValid(Teb as PVOID) == 0 {
            break 'done;
        }
        let Peb = (*Teb).ProcessEnvironmentBlock;
        if Peb.is_null() || MmIsAddressValid(Peb as PVOID) == 0 {
            break 'done;
        }
        let PebLdr = (*Peb).Ldr;
        if PebLdr.is_null() || MmIsAddressValid(PebLdr as PVOID) == 0 {
            break 'done;
        }

        let ListHead = core::ptr::addr_of_mut!((*PebLdr).InLoadOrderModuleList) as PLIST_ENTRY;
        let mut ListEntry = (*PebLdr).InLoadOrderModuleList.Flink;

        while ListEntry != ListHead {
            if MmIsAddressValid(ListEntry as PVOID) == 0 {
                break;
            }
            let LdrEntry = ((ListEntry as usize) - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)) as PLDR_DATA_TABLE_ENTRY;

            if MmIsAddressValid(LdrEntry as PVOID) != 0 {
                KiPageInDirectory((*LdrEntry).DllBase, IMAGE_DIRECTORY_ENTRY_DEBUG);
                KiPageInDirectory((*LdrEntry).DllBase, IMAGE_DIRECTORY_ENTRY_EXCEPTION);
            }

            ListEntry = (*ListEntry).Flink;
        }
    }

    _disable();
}

#[no_mangle]
pub unsafe extern "system" fn KiDispatchException(
    ExceptionRecord: PEXCEPTION_RECORD,
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
    PreviousMode: KPROCESSOR_MODE,
    FirstChance: BOOLEAN,
) {
    let mut Context: CONTEXT = core::mem::zeroed();
    
    (*KeGetCurrentPrcb()).KeExceptionDispatchCount += 1;
    
    core::ptr::write_bytes(core::ptr::addr_of_mut!(Context) as *mut u8, 0, core::mem::size_of::<CONTEXT>());
    
    Context.ContextFlags = CONTEXT_ALL;
    
    KeTrapFrameToContext(TrapFrame, ExceptionFrame, core::ptr::addr_of_mut!(Context));
    
    match (*ExceptionRecord).ExceptionCode {
        STATUS_BREAKPOINT => {
            Context.Rip -= 1;
        }
        KI_EXCEPTION_ACCESS_VIOLATION => {
            (*ExceptionRecord).ExceptionCode = STATUS_ACCESS_VIOLATION;
            if PreviousMode == 1i8 {
                // FIXME: Handle no execute
            }
        }
        _ => {}
    }
    
    if PreviousMode == 0i8 {
        if FirstChance != 0 {
            if KiDebugRoutine(
                TrapFrame,
                ExceptionFrame,
                ExceptionRecord,
                core::ptr::addr_of_mut!(Context),
                PreviousMode,
                0u8,
            ) != 0
            {
                KeContextToTrapFrame(
                    core::ptr::addr_of_mut!(Context),
                    ExceptionFrame,
                    TrapFrame,
                    Context.ContextFlags,
                    PreviousMode,
                );
                return;
            }
            
            if RtlDispatchException(ExceptionRecord, core::ptr::addr_of_mut!(Context)) != 0 {
                KeContextToTrapFrame(
                    core::ptr::addr_of_mut!(Context),
                    ExceptionFrame,
                    TrapFrame,
                    Context.ContextFlags,
                    PreviousMode,
                );
                return;
            }
        }
        
        if KiDebugRoutine(
            TrapFrame,
            ExceptionFrame,
            ExceptionRecord,
            core::ptr::addr_of_mut!(Context),
            PreviousMode,
            1u8,
        ) != 0
        {
            KeContextToTrapFrame(
                core::ptr::addr_of_mut!(Context),
                ExceptionFrame,
                TrapFrame,
                Context.ContextFlags,
                PreviousMode,
            );
            return;
        }
        
        // DUMMY_PSEH: SEH handlers are dead code, so kernel-mode exceptions
        // that would normally be caught by __try/__except go unhandled.
        // For STATUS_BREAKPOINT (INT 3 from DbgBreakPoint/RtlAssert): skip
        // the breakpoint and continue.  Rip was decremented above; restore
        // it past the INT 3 byte so execution continues after the assert.
        if (*ExceptionRecord).ExceptionCode == STATUS_BREAKPOINT {
            Context.Rip += 1;
            KeContextToTrapFrame(
                core::ptr::addr_of_mut!(Context),
                ExceptionFrame,
                TrapFrame,
                Context.ContextFlags,
                PreviousMode,
            );
            return;
        }

        KeBugCheckEx(
            KMODE_EXCEPTION_NOT_HANDLED,
            (*ExceptionRecord).ExceptionCode as ULONG_PTR,
            (*ExceptionRecord).ExceptionAddress as ULONG_PTR,
            TrapFrame as ULONG_PTR,
            0,
        );
    } else {
        if FirstChance != 0 {
            if ((*((PsGetCurrentProcess() as PEPROCESS) as *mut _EPROCESS)).DebugPort.is_null()
                && KdIgnoreUmExceptions == 0)
                || KdIsThisAKdTrap(ExceptionRecord, core::ptr::addr_of_mut!(Context), PreviousMode) != 0
            {
                KiPrepareUserDebugData();

                if KiDebugRoutine(
                    TrapFrame,
                    ExceptionFrame,
                    ExceptionRecord,
                    core::ptr::addr_of_mut!(Context),
                    PreviousMode,
                    0u8,
                ) != 0
                {
                    KeContextToTrapFrame(
                        core::ptr::addr_of_mut!(Context),
                        ExceptionFrame,
                        TrapFrame,
                        Context.ContextFlags,
                        PreviousMode,
                    );
                    return;
                }
            }

            if DbgkForwardException(ExceptionRecord, 1u8, 0u8) != 0 {
                return;
            }

            if KiDispatchExceptionToUser(TrapFrame, core::ptr::addr_of_mut!(Context), ExceptionRecord) != 0 {
                return;
            }
        }
        
        // Permanent tripwire (field manual §9.41): a user-mode exception
        // reaching SECOND CHANCE is a process death — csrss's exception port
        // usually "handles" it by TerminateProcess(client, 0x80), so without
        // this print the smoke serial shows only 4x [EXIT-THR] 0x80 with no
        // cause (the port forward below returns nonzero and the terminate
        // path here is never reached). Print the face + the two CLIENTINFO
        // fields the §9.41 stray write stomps (TEB+0x800 is CLIENTINFO;
        // pDeskInfo at +0x20, ulClientDelta at +0x28) so the faulting RIP is
        // diagnosable from serial alone.
        {
            let proc_ = PsGetCurrentProcess() as *mut _EPROCESS;
            let img = (*proc_).ImageFileName.as_ptr();
            let teb = (*PsGetCurrentThread()).Teb as *const u8;
            let mut pdesk: u64 = 0;
            let mut delta: u64 = 0;
            if !teb.is_null() && MmIsAddressValid(teb as _) != 0 {
                pdesk = core::ptr::read_unaligned(teb.add(0x820) as *const u64); // nolint: magic-offset (CLIENTINFO.pDeskInfo @ TEB+0x820 — §9.41 diagnostic tripwire)
                delta = core::ptr::read_unaligned(teb.add(0x828) as *const u64); // nolint: magic-offset (CLIENTINFO.ulClientDelta @ TEB+0x828 — §9.41 diagnostic tripwire)
            }
            crate::dbg_print!(
                b"[KI-EXC] %s teb=%p code=0x%lx addr=%p rip=%p pci.pDeskInfo=%p pci.ulClientDelta=%p\n\0",
                img as *const i8,
                teb as usize,
                (*ExceptionRecord).ExceptionCode as u32,
                (*ExceptionRecord).ExceptionAddress,
                (*TrapFrame).Rip as usize,
                pdesk as usize,
                delta as usize,
            );
            // The stack words: when RIP is not code at all (a call through a garbage pointer),
            // the caller is the return address at [RSP], and nothing else on this line names
            // it. Print RSP and the first qwords above it that look like user code addresses.
            // The bytes the CPU actually fetched, which CPU it was, and the physical page
            // behind them. csrss died at -smp 2 with 0xC000001D on an address whose file
            // bytes are int3 padding -- illegal only if memory differs from the image. With
            // these, one sample says whether the code page is wrong (bytes differ from the
            // file) or execution arrived through a wrong target (bytes match), and whether
            // it tracks one processor.
            let rip = (*TrapFrame).Rip as usize;
            let line = (rip & !0xf).wrapping_sub(0x10);
            let phys = if MmIsAddressValid(rip as _) != 0 {
                ntkrnl_sys::ddk::MmGetPhysicalAddress(rip as _).QuadPart as usize
            } else {
                0
            };
            crate::dbg_print!(
                b"[KI-EXC]   cpu=%lu rip.phys=%p\n\0",
                ntkrnl_sys::ddk::KeGetCurrentProcessorNumber() as u32,
                phys,
            );
            let mut q = 0usize;
            while q < 4 {
                let a = line + q * 8;
                if MmIsAddressValid(a as _) == 0 {
                    break;
                }
                crate::dbg_print!(
                    b"[KI-EXC]   code[%p]=%p\n\0",
                    a,
                    core::ptr::read_unaligned(a as *const u64).swap_bytes() as usize,
                );
                q += 1;
            }
            // The data address an access violation touched (P1) and the volatile registers
            // the trap frame holds: with only RIP, a fault inside a string routine says
            // nothing about which caller handed it the bad pointer.
            crate::dbg_print!(
                b"[KI-EXC]   P0=%p P1=%p rax=%p rcx=%p rdx=%p r8=%p r9=%p\n\0",
                (*ExceptionRecord).ExceptionInformation[0] as usize,
                (*ExceptionRecord).ExceptionInformation[1] as usize,
                (*TrapFrame).Rax as usize,
                (*TrapFrame).Rcx as usize,
                (*TrapFrame).Rdx as usize,
                (*TrapFrame).R8 as usize,
                (*TrapFrame).R9 as usize,
            );
            let rsp = (*TrapFrame).Rsp as usize;
            crate::dbg_print!(b"[KI-EXC]   rsp=%p\n\0", rsp);
            let mut shown = 0;
            let mut off = 0usize;
            while off < 0x200 && shown < 10 {
                let a = rsp + off;
                if MmIsAddressValid(a as _) == 0 {
                    break;
                }
                let w = core::ptr::read_unaligned(a as *const u64);
                // Above 4 GiB: EXEs load at 0x1_4000_0000 and DLLs at 0x7ff..., while stacks
                // and heaps sit low. `>= 0x7ff` alone hid every EXE return address, which is
                // why usetup's crash in ntdll's _wcsicmp showed no usetup frame at all.
                if w < 0x0000_8000_0000_0000 && (w >> 32) >= 1 {
                    crate::dbg_print!(b"[KI-EXC]   [rsp+0x%lx]=%p\n\0", off as u32, w as usize);
                    shown += 1;
                }
                off += 8;
            }
        }

        if DbgkForwardException(ExceptionRecord, 1u8, 1u8) != 0 {
            return;
        } else if DbgkForwardException(ExceptionRecord, 0u8, 1u8) != 0 {
            return;
        }

        ZwTerminateProcess(NtCurrentProcess(), (*ExceptionRecord).ExceptionCode);
        KeBugCheckEx(
            KMODE_EXCEPTION_NOT_HANDLED,
            (*ExceptionRecord).ExceptionCode as ULONG_PTR,
            (*ExceptionRecord).ExceptionAddress as ULONG_PTR,
            TrapFrame as ULONG_PTR,
            0,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeRaiseUserException(ExceptionCode: NTSTATUS) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn KiSystemFatalException(
    ExceptionCode: ULONG,
    TrapFrame: PKTRAP_FRAME,
) -> ! {
    KeBugCheckWithTf(UNEXPECTED_KERNEL_MODE_TRAP, ExceptionCode as u64, 0, 0, 0, TrapFrame);
}

#[no_mangle]
pub unsafe extern "system" fn KiNpxNotAvailableFaultHandler(TrapFrame: PKTRAP_FRAME) -> NTSTATUS {
    KeBugCheckWithTf(TRAP_CAUSE_UNKNOWN, 13, 0, 0, 1, TrapFrame);
    -1i32
}

unsafe fn KiIsPrivilegedInstruction(Ip: *const UCHAR, Wow64: BOOLEAN) -> BOOLEAN {
    let mut i = 0usize;
    let mut current_ip = Ip;
    
    while i < 15 {
        if Wow64 == 0 {
            if *current_ip >= 0x40 && *current_ip <= 0x4F {
                current_ip = current_ip.add(1);
                i += 1;
                continue;
            }
        }
        
        match *current_ip {
            0x26 | 0x2E | 0x36 | 0x3E | 0x64 | 0x65 | 0x66 | 0x67 | 0xF0 | 0xF2 | 0xF3 => {
                current_ip = current_ip.add(1);
                i += 1;
                continue;
            }
            _ => break,
        }
    }
    
    if i == 15 {
        return 0u8;
    }
    
    match *current_ip {
        0xF4 | 0xFA | 0xFB => return 1u8,
        0x0F => {
            match *current_ip.add(1) {
                0x06 | 0x07 | 0x08 | 0x09 | 0x20 | 0x21 | 0x22 | 0x23 | 0x30 | 0x32 | 0x33 | 0x35 | 0x78 | 0x79 => {
                    return 1u8;
                }
                0x00 => {
                    let reg = (*current_ip.add(2) >> 3) & 0x7;
                    if reg == 2 || reg == 3 {
                        return 1u8;
                    }
                }
                0x01 => {
                    match *current_ip.add(2) {
                        0xC1 | 0xC2 | 0xC3 | 0xC4 | 0xC8 | 0xC9 | 0xD1 | 0xF8 => return 1u8,
                        _ => {}
                    }
                    let reg = (*current_ip.add(2) >> 3) & 0x7;
                    if reg == 2 || reg == 3 || reg == 6 || reg == 7 {
                        return 1u8;
                    }
                }
                0x38 => {
                    match *current_ip.add(2) {
                        0x80 | 0x81 => return 1u8,
                        _ => {}
                    }
                }
                0xC7 => {
                    let reg = (*current_ip.add(2) >> 3) & 0x7;
                    if reg == 0x06 || reg == 0x07 {
                        return 1u8;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    
    0u8
}

unsafe fn KiGeneralProtectionFaultUserMode(TrapFrame: PKTRAP_FRAME) -> NTSTATUS {
    let Wow64 = if (*TrapFrame).SegCs == KGDT64_R3_CMCODE { 1u8 } else { 0u8 };
    let InstructionPointer = (*TrapFrame).Rip as *const UCHAR;
    
    let Status = 'seh: {
        if let Err(e) = (|| -> Result<(), NTSTATUS> {
            use ntkrnl_sys::probe::*;
            probe_user_buffer(
                InstructionPointer as *const core::ffi::c_void,
                64,
                1,
                1i8,
            )?;
            Ok(())
        })() {
            break 'seh e;
        }
        
        if KiIsPrivilegedInstruction(InstructionPointer, Wow64) != 0 {
            STATUS_PRIVILEGED_INSTRUCTION
        } else {
            STATUS_ACCESS_VIOLATION
        }
    };
    
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KiGeneralProtectionFaultHandler(TrapFrame: PKTRAP_FRAME) -> NTSTATUS {
    if ((*TrapFrame).SegCs & 3) != 0 {
        return KiGeneralProtectionFaultUserMode(TrapFrame);
    }
    
    if (*TrapFrame).SegDs != (KGDT64_R3_DATA | RPL_MASK) {
        (*TrapFrame).SegDs = KGDT64_R3_DATA | RPL_MASK;
        return STATUS_SUCCESS;
    } else if (*TrapFrame).SegEs != (KGDT64_R3_DATA | RPL_MASK) {
        (*TrapFrame).SegEs = KGDT64_R3_DATA | RPL_MASK;
        return STATUS_SUCCESS;
    }
    
    let Instructions = (*TrapFrame).Rip as *const UCHAR;
    
    if *Instructions == 0x48 && *Instructions.add(1) == 0xCF {
        core::unimplemented!();
    }
    
    if *Instructions == 0x0F && (*Instructions.add(1) == 0x30 || *Instructions.add(1) == 0x32) {
        return STATUS_ACCESS_VIOLATION;
    }
    
    if *Instructions == 0xE4 || *Instructions == 0xE5 || *Instructions == 0xE6 || *Instructions == 0xE7
        || *Instructions == 0xEC || *Instructions == 0xED || *Instructions == 0xEE || *Instructions == 0xEF
    {
        return STATUS_ACCESS_VIOLATION;
    }

    STATUS_ACCESS_VIOLATION
}

#[no_mangle]
pub unsafe extern "system" fn KiXmmExceptionHandler(TrapFrame: PKTRAP_FRAME) -> NTSTATUS {
    let ExceptionCode: ULONG;
    
    if ((*TrapFrame).MxCsr & _MM_EXCEPT_INVALID) != 0 && ((*TrapFrame).MxCsr & _MM_MASK_INVALID) == 0 {
        ExceptionCode = STATUS_FLOAT_INVALID_OPERATION as u32;
    } else if ((*TrapFrame).MxCsr & _MM_EXCEPT_DENORM) != 0 && ((*TrapFrame).MxCsr & _MM_MASK_DENORM) == 0 {
        ExceptionCode = STATUS_FLOAT_INVALID_OPERATION as u32;
    } else if ((*TrapFrame).MxCsr & _MM_EXCEPT_DIV_ZERO) != 0 && ((*TrapFrame).MxCsr & _MM_MASK_DIV_ZERO) == 0 {
        ExceptionCode = STATUS_FLOAT_DIVIDE_BY_ZERO as u32;
    } else if ((*TrapFrame).MxCsr & _MM_EXCEPT_OVERFLOW) != 0 && ((*TrapFrame).MxCsr & _MM_MASK_OVERFLOW) == 0 {
        ExceptionCode = STATUS_FLOAT_OVERFLOW as u32;
    } else if ((*TrapFrame).MxCsr & _MM_EXCEPT_UNDERFLOW) != 0 && ((*TrapFrame).MxCsr & _MM_MASK_UNDERFLOW) == 0 {
        ExceptionCode = STATUS_FLOAT_UNDERFLOW as u32;
    } else if ((*TrapFrame).MxCsr & _MM_EXCEPT_INEXACT) != 0 && ((*TrapFrame).MxCsr & _MM_MASK_INEXACT) == 0 {
        ExceptionCode = STATUS_FLOAT_INEXACT_RESULT as u32;
    } else {
        ExceptionCode = STATUS_SUCCESS as u32;
    }
    
    ExceptionCode as NTSTATUS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/amd64/except.c
//  * PURPOSE:         Exception Dispatching for amd64
//  * PROGRAMMER:      Timo Kreuzer (timo.kreuzer@reactos.org)
//  *                  Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// extern KI_INTERRUPT_DISPATCH_ENTRY KiUnexpectedRange[256];
//
// /* GLOBALS *******************************************************************/
//
// KIDT_INIT KiInterruptInitTable[] =
// {
//   /* Id,   Dpl,  IST,  ServiceRoutine */
//     {0x00, 0x00, 0x00, KiDivideErrorFault},
//     {0x01, 0x00, 0x00, KiDebugTrapOrFault},
//     {0x02, 0x00, 0x03, KiNmiInterrupt},
//     {0x03, 0x03, 0x00, KiBreakpointTrap},
//     {0x04, 0x03, 0x00, KiOverflowTrap},
//     {0x05, 0x00, 0x00, KiBoundFault},
//     {0x06, 0x00, 0x00, KiInvalidOpcodeFault},
//     {0x07, 0x00, 0x00, KiNpxNotAvailableFault},
//     {0x08, 0x00, 0x01, KiDoubleFaultAbort},
//     {0x09, 0x00, 0x00, KiNpxSegmentOverrunAbort},
//     {0x0A, 0x00, 0x00, KiInvalidTssFault},
//     {0x0B, 0x00, 0x00, KiSegmentNotPresentFault},
//     {0x0C, 0x00, 0x00, KiStackFault},
//     {0x0D, 0x00, 0x00, KiGeneralProtectionFault},
//     {0x0E, 0x00, 0x00, KiPageFault},
//     {0x10, 0x00, 0x00, KiFloatingErrorFault},
//     {0x11, 0x00, 0x00, KiAlignmentFault},
//     {0x12, 0x00, 0x02, KiMcheckAbort},
//     {0x13, 0x00, 0x00, KiXmmException},
//     {0x1F, 0x00, 0x00, KiApcInterrupt},
//     {0x2C, 0x03, 0x00, KiRaiseAssertion},
//     {0x2D, 0x03, 0x00, KiDebugServiceTrap},
//     {0x2F, 0x00, 0x00, KiDpcInterrupt},
//     {0xE1, 0x00, 0x00, KiIpiInterrupt},
//     {0, 0, 0, 0}
// };
//
// KIDTENTRY64 KiIdt[256];
// KDESCRIPTOR KiIdtDescriptor = {{0}, sizeof(KiIdt) - 1, KiIdt};
//
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KeInitExceptions(VOID)
// {
//     int i, j;
//
//     /* Initialize the Idt */
//     for (j = i = 0; i < 256; i++)
//     {
//         u64 Offset;
//
//         if (KiInterruptInitTable[j].InterruptId == i)
//         {
//             Offset = (u64)KiInterruptInitTable[j].ServiceRoutine;
//             KiIdt[i].Dpl = KiInterruptInitTable[j].Dpl;
//             KiIdt[i].IstIndex = KiInterruptInitTable[j].IstIndex;
//             j++;
//         }
//         else
//         {
//             Offset = (u64)&KiUnexpectedRange[i]._Op_push;
//             KiIdt[i].Dpl = 0;
//             KiIdt[i].IstIndex = 0;
//         }
//         KiIdt[i].OffsetLow = Offset & 0xffff;
//         KiIdt[i].Selector = KGDT64_R0_CODE;
//         KiIdt[i].Type = 0x0e;
//         KiIdt[i].Reserved0 = 0;
//         KiIdt[i].Present = 1;
//         KiIdt[i].OffsetMiddle = (Offset >> 16) & 0xffff;
//         KiIdt[i].OffsetHigh = (Offset >> 32);
//         KiIdt[i].Reserved1 = 0;
//     }
//
//     KeGetPcr()->IdtBase = KiIdt;
//     __lidt(&KiIdtDescriptor.Limit);
// }
//
// static
// BOOLEAN
// KiDispatchExceptionToUser(
//     IN PKTRAP_FRAME TrapFrame,
//     IN PCONTEXT Context,
//     IN PEXCEPTION_RECORD ExceptionRecord)
// {
//     EXCEPTION_RECORD LocalExceptRecord;
//     u64 UserRsp;
//     PKUSER_EXCEPTION_STACK UserStack;
//
//     /* Make sure we have a valid SS */
//     if (TrapFrame->SegSs != (KGDT64_R3_DATA | RPL_MASK))
//     {
//         /* Raise an access violation instead */
//         LocalExceptRecord.ExceptionCode = STATUS_ACCESS_VIOLATION;
//         LocalExceptRecord.ExceptionFlags = 0;
//         LocalExceptRecord.NumberParameters = 0;
//         ExceptionRecord = &LocalExceptRecord;
//     }
//
//     /* Get new stack pointer and align it to 16 bytes */
//     UserRsp = (Context->Rsp - sizeof(KUSER_EXCEPTION_STACK)) & ~15;
//
//     /* Get pointer to the usermode context, exception record and machine frame */
//     UserStack = (PKUSER_EXCEPTION_STACK)UserRsp;
//
//     /* Enable interrupts */
//     _enable();
//
//     /* Set up the user-stack */
//     _SEH2_TRY
//     {
//         /* Probe the user stack frame and zero it out */
//         ProbeForWrite(UserStack, sizeof(*UserStack), TYPE_ALIGNMENT(KUSER_EXCEPTION_STACK));
//         RtlZeroMemory(UserStack, sizeof(*UserStack));
//
//         /* Copy Context and ExceptionFrame */
//         UserStack->Context = *Context;
//         UserStack->ExceptionRecord = *ExceptionRecord;
//
//         /* Setup the machine frame */
//         UserStack->MachineFrame.Rip = Context->Rip;
//         UserStack->MachineFrame.SegCs = Context->SegCs;
//         UserStack->MachineFrame.EFlags = Context->EFlags;
//         UserStack->MachineFrame.Rsp = Context->Rsp;
//         UserStack->MachineFrame.SegSs = Context->SegSs;
//     }
//     _SEH2_EXCEPT((LocalExceptRecord = *_SEH2_GetExceptionInformation()->ExceptionRecord),
//                  EXCEPTION_EXECUTE_HANDLER)
//     {
//         // FIXME: handle stack overflow
//
//         /* Nothing we can do here */
//         _disable();
//         return FALSE;
//     }
//     _SEH2_END;
//
//     /* Now set the two params for the user-mode dispatcher */
//     TrapFrame->Rcx = (u64)&UserStack->ExceptionRecord;
//     TrapFrame->Rdx = (u64)&UserStack->Context;
//
//     /* Set new Stack Pointer */
//     TrapFrame->Rsp = UserRsp;
//
//     /* Force correct segments */
//     TrapFrame->SegCs = KGDT64_R3_CODE | RPL_MASK;
//     TrapFrame->SegDs = KGDT64_R3_DATA | RPL_MASK;
//     TrapFrame->SegEs = KGDT64_R3_DATA | RPL_MASK;
//     TrapFrame->SegFs = KGDT64_R3_CMTEB | RPL_MASK;
//     TrapFrame->SegGs = KGDT64_R3_DATA | RPL_MASK;
//     TrapFrame->SegSs = KGDT64_R3_DATA | RPL_MASK;
//
//     /* Set RIP to the User-mode Dispatcher */
//     TrapFrame->Rip = (u64)KeUserExceptionDispatcher;
//
//     _disable();
//
//     /* Exit to usermode */
//     return TRUE;
// }
//
// static
// VOID
// KiPageInDirectory(PVOID ImageBase, USHORT Directory)
// {
//     volatile CHAR *Pointer;
//     ULONG Size;
//
//    /* Get a pointer to the debug directory */
//     Pointer = RtlImageDirectoryEntryToData(ImageBase, 1, Directory, &Size);
//     if (!Pointer) return;
//
//     /* Loop all pages */
//     while ((LONG)Size > 0)
//     {
//         /* Touch it, to page it in */
//         (void)*Pointer;
//         Pointer += PAGE_SIZE;
//         Size -= PAGE_SIZE;
//     }
// }
//
// VOID
// KiPrepareUserDebugData(void)
// {
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PPEB_LDR_DATA PebLdr;
//     PLIST_ENTRY ListEntry;
//     PTEB Teb;
//
//     /* Get the Teb for this process */
//     Teb = PsGetCurrentThread()->Teb;
//     if (!Teb) return;
//
//     /* Enable interrupts */
//     _enable();
//
//     _SEH2_TRY
//     {
//         /* Get a pointer to the loader data */
//         PebLdr = Teb->ProcessEnvironmentBlock->Ldr;
//         if (!PebLdr) _SEH2_LEAVE;
//
//         /* Now loop all entries in the module list */
//         for (ListEntry = PebLdr->InLoadOrderModuleList.Flink;
//              ListEntry != &PebLdr->InLoadOrderModuleList;
//              ListEntry = ListEntry->Flink)
//         {
//             /* Get the loader entry */
//             LdrEntry = CONTAINING_RECORD(ListEntry,
//                                          LDR_DATA_TABLE_ENTRY,
//                                          InLoadOrderLinks);
//
//             KiPageInDirectory((PVOID)LdrEntry->DllBase,
//                               IMAGE_DIRECTORY_ENTRY_DEBUG);
//
//             KiPageInDirectory((PVOID)LdrEntry->DllBase,
//                               IMAGE_DIRECTORY_ENTRY_EXCEPTION);
//         }
//
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     _disable();
// }
//
// VOID
// NTAPI
// KiDispatchException(IN PEXCEPTION_RECORD ExceptionRecord,
//                     IN PKEXCEPTION_FRAME ExceptionFrame,
//                     IN PKTRAP_FRAME TrapFrame,
//                     IN KPROCESSOR_MODE PreviousMode,
//                     IN BOOLEAN FirstChance)
// {
//     CONTEXT Context;
//
//     /* Increase number of Exception Dispatches */
//     KeGetCurrentPrcb()->KeExceptionDispatchCount++;
//
//     /* Zero out the context to avoid leaking kernel stack memor to user mode */
//     RtlZeroMemory(&Context, sizeof(Context));
//
//     /* Set the context flags */
//     Context.ContextFlags = CONTEXT_ALL;
//
//     /* Get the Context from the trap and exception frame */
//     KeTrapFrameToContext(TrapFrame, ExceptionFrame, &Context);
//
//     /* Look at our exception code */
//     switch (ExceptionRecord->ExceptionCode)
//     {
//         /* Breakpoint */
//         case STATUS_BREAKPOINT:
//
//             /* Decrement RIP by one */
//             Context.Rip--;
//             break;
//
//         /* Internal exception */
//         case KI_EXCEPTION_ACCESS_VIOLATION:
//
//             /* Set correct code */
//             ExceptionRecord->ExceptionCode = STATUS_ACCESS_VIOLATION;
//             if (PreviousMode == UserMode)
//             {
//                 /* FIXME: Handle no execute */
//             }
//             break;
//     }
//
//     /* Handle kernel-mode first, it's simpler */
//     if (PreviousMode == KernelMode)
//     {
//         /* Check if this is a first-chance exception */
//         if (FirstChance)
//         {
//             /* Break into the debugger for the first time */
//             if (KiDebugRoutine(TrapFrame,
//                                ExceptionFrame,
//                                ExceptionRecord,
//                                &Context,
//                                PreviousMode,
//                                FALSE))
//             {
//                 /* Exception was handled */
//                 goto Handled;
//             }
//
//             /* If the Debugger couldn't handle it, dispatch the exception */
//             if (RtlDispatchException(ExceptionRecord, &Context)) goto Handled;
//         }
//
//         /* This is a second-chance exception, only for the debugger */
//         if (KiDebugRoutine(TrapFrame,
//                            ExceptionFrame,
//                            ExceptionRecord,
//                            &Context,
//                            PreviousMode,
//                            TRUE))
//         {
//             /* Exception was handled */
//             goto Handled;
//         }
//
//         /* Third strike; you're out */
//         KeBugCheckEx(KMODE_EXCEPTION_NOT_HANDLED,
//                      ExceptionRecord->ExceptionCode,
//                      (ULONG_PTR)ExceptionRecord->ExceptionAddress,
//                      (ULONG_PTR)TrapFrame,
//                      0);
//     }
//     else
//     {
//         /* User mode exception, was it first-chance? */
//         if (FirstChance)
//         {
//             /*
//              * Break into the kernel debugger unless a user mode debugger
//              * is present or user mode exceptions are ignored, except if this
//              * is a debug service which we must always pass to KD
//              */
//             if ((!(PsGetCurrentProcess()->DebugPort) &&
//                  !(KdIgnoreUmExceptions)) ||
//                  (KdIsThisAKdTrap(ExceptionRecord, &Context, PreviousMode)))
//             {
//                 /* Make sure the debugger can access debug directories */
//                 KiPrepareUserDebugData();
//
//                 /* Call the kernel debugger */
//                 if (KiDebugRoutine(TrapFrame,
//                                    ExceptionFrame,
//                                    ExceptionRecord,
//                                    &Context,
//                                    PreviousMode,
//                                    FALSE))
//                 {
//                     /* Exception was handled */
//                     goto Handled;
//                 }
//             }
//
//             /* Forward exception to user mode debugger */
//             if (DbgkForwardException(ExceptionRecord, TRUE, FALSE)) return;
//
//             /* Forward exception to user mode */
//             if (KiDispatchExceptionToUser(TrapFrame, &Context, ExceptionRecord))
//             {
//                 /* Success, the exception will be handled by KiUserExceptionDispatcher */
//                 return;
//             }
//
//             /* Failed to dispatch, fall through for second chance handling */
//         }
//
//         /* Try second chance */
//         if (DbgkForwardException(ExceptionRecord, TRUE, TRUE))
//         {
//             /* Handled, get out */
//             return;
//         }
//         else if (DbgkForwardException(ExceptionRecord, FALSE, TRUE))
//         {
//             /* Handled, get out */
//             return;
//         }
//
//         /* 3rd strike, kill the process */
//         DPRINT1("Kill %.16s, ExceptionCode: %lx, ExceptionAddress: %lx, BaseAddress: %lx\n",
//                 PsGetCurrentProcess()->ImageFileName,
//                 ExceptionRecord->ExceptionCode,
//                 ExceptionRecord->ExceptionAddress,
//                 PsGetCurrentProcess()->SectionBaseAddress);
//
//         ZwTerminateProcess(NtCurrentProcess(), ExceptionRecord->ExceptionCode);
//         KeBugCheckEx(KMODE_EXCEPTION_NOT_HANDLED,
//                      ExceptionRecord->ExceptionCode,
//                      (ULONG_PTR)ExceptionRecord->ExceptionAddress,
//                      (ULONG_PTR)TrapFrame,
//                      0);
//     }
//
// Handled:
//     /* Convert the context back into Trap/Exception Frames */
//     KeContextToTrapFrame(&Context,
//                          ExceptionFrame,
//                          TrapFrame,
//                          Context.ContextFlags,
//                          PreviousMode);
//     return;
// }
//
// NTSTATUS
// NTAPI
// KeRaiseUserException(IN NTSTATUS ExceptionCode)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
//
// VOID
// DECLSPEC_NORETURN
// KiSystemFatalException(IN ULONG ExceptionCode,
//                        IN PKTRAP_FRAME TrapFrame)
// {
//     /* Bugcheck the system */
//     KeBugCheckWithTf(UNEXPECTED_KERNEL_MODE_TRAP,
//                      ExceptionCode,
//                      0,
//                      0,
//                      0,
//                      TrapFrame);
// }
//
// NTSTATUS
// NTAPI
// KiNpxNotAvailableFaultHandler(
//     IN PKTRAP_FRAME TrapFrame)
// {
//     UNIMPLEMENTED;
//     KeBugCheckWithTf(TRAP_CAUSE_UNKNOWN, 13, 0, 0, 1, TrapFrame);
//     return -1;
// }
//
// static
// BOOLEAN
// KiIsPrivilegedInstruction(PUCHAR Ip, BOOLEAN Wow64)
// {
//     ULONG i;
//
//     /* Handle prefixes */
//     for (i = 0; i < 15; i++)
//     {
//         if (!Wow64)
//         {
//             /* Check for REX prefix */
//             if ((Ip[0] >= 0x40) && (Ip[0] <= 0x4F))
//             {
//                 Ip++;
//                 continue;
//             }
//         }
//
//         switch (Ip[0])
//         {
//             /* Check prefixes */
//             case 0x26: // ES
//             case 0x2E: // CS / null
//             case 0x36: // SS
//             case 0x3E: // DS
//             case 0x64: // FS
//             case 0x65: // GS
//             case 0x66: // OP
//             case 0x67: // ADDR
//             case 0xF0: // LOCK
//             case 0xF2: // REP
//             case 0xF3: // REP INS/OUTS
//                 Ip++;
//                 continue;
//         }
//
//         break;
//     }
//
//     if (i == 15)
//     {
//         /* Too many prefixes. Should only happen, when the code was concurrently modified. */
//          return FALSE;
//     }
//
//     switch (Ip[0])
//     {
//         case 0xF4: // HLT
//         case 0xFA: // CLI
//         case 0xFB: // STI
//             return TRUE;
//
//         case 0x0F:
//         {
//             switch (Ip[1])
//             {
//                 case 0x06: // CLTS
//                 case 0x07: // SYSRET
//                 case 0x08: // INVD
//                 case 0x09: // WBINVD
//                 case 0x20: // MOV CR, XXX
//                 case 0x21: // MOV DR, XXX
//                 case 0x22: // MOV XXX, CR
//                 case 0x23: // MOV YYY, DR
//                 case 0x30: // WRMSR
//                 case 0x32: // RDMSR
//                 case 0x33: // RDPMC
//                 case 0x35: // SYSEXIT
//                 case 0x78: // VMREAD
//                 case 0x79: // VMWRITE
//                     return TRUE;
//
//                 case 0x00:
//                 {
//                     /* Check MODRM Reg field */
//                     switch ((Ip[2] >> 3) & 0x7)
//                     {
//                         case 2: // LLDT
//                         case 3: // LTR
//                             return TRUE;
//                     }
//                     break;
//                 }
//
//                 case 0x01:
//                 {
//                     switch (Ip[2])
//                     {
//                         case 0xC1: // VMCALL
//                         case 0xC2: // VMLAUNCH
//                         case 0xC3: // VMRESUME
//                         case 0xC4: // VMXOFF
//                         case 0xC8: // MONITOR
//                         case 0xC9: // MWAIT
//                         case 0xD1: // XSETBV
//                         case 0xF8: // SWAPGS
//                             return TRUE;
//                     }
//
//                     /* Check MODRM Reg field */
//                     switch ((Ip[2] >> 3) & 0x7)
//                     {
//                         case 2: // LGDT
//                         case 3: // LIDT
//                         case 6: // LMSW
//                         case 7: // INVLPG / SWAPGS / RDTSCP
//                             return TRUE;
//                     }
//                     break;
//                 }
//
//                 case 0x38:
//                 {
//                     switch (Ip[2])
//                     {
//                         case 0x80: // INVEPT
//                         case 0x81: // INVVPID
//                             return TRUE;
//                     }
//                     break;
//                 }
//
//                 case 0xC7:
//                 {
//                     /* Check MODRM Reg field */
//                     switch ((Ip[2] >> 3) & 0x7)
//                     {
//                         case 0x06: // VMPTRLD, VMCLEAR, VMXON
//                         case 0x07: // VMPTRST
//                             return TRUE;
//                     }
//                     break;
//                 }
//             }
//
//             break;
//         }
//     }
//
//     return FALSE;
// }
//
// static
// NTSTATUS
// KiGeneralProtectionFaultUserMode(
//     _In_ PKTRAP_FRAME TrapFrame)
// {
//     BOOLEAN Wow64 = TrapFrame->SegCs == KGDT64_R3_CMCODE;
//     PUCHAR InstructionPointer;
//     NTSTATUS Status;
//
//     /* We need to decode the instruction at RIP */
//     InstructionPointer = (PUCHAR)TrapFrame->Rip;
//
//     _SEH2_TRY
//     {
//         /* Probe the instruction address */
//         ProbeForRead(InstructionPointer, 64, 1);
//
//         /* Check if it's a privileged instruction */
//         if (KiIsPrivilegedInstruction(InstructionPointer, Wow64))
//         {
//             Status = STATUS_PRIVILEGED_INSTRUCTION;
//         }
//         else
//         {
//             Status = STATUS_ACCESS_VIOLATION;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// KiGeneralProtectionFaultHandler(
//     IN PKTRAP_FRAME TrapFrame)
// {
//     PUCHAR Instructions;
//
//     /* Check for user-mode GPF */
//     if (TrapFrame->SegCs & 3)
//     {
//         return KiGeneralProtectionFaultUserMode(TrapFrame);
//     }
//
//     /* Check for lazy segment load */
//     if (TrapFrame->SegDs != (KGDT64_R3_DATA | RPL_MASK))
//     {
//         /* Fix it */
//         TrapFrame->SegDs = (KGDT64_R3_DATA | RPL_MASK);
//         return STATUS_SUCCESS;
//     }
//     else if (TrapFrame->SegEs != (KGDT64_R3_DATA | RPL_MASK))
//     {
//         /* Fix it */
//         TrapFrame->SegEs = (KGDT64_R3_DATA | RPL_MASK);
//         return STATUS_SUCCESS;
//     }
//
//     /* Get Instruction Pointer */
//     Instructions = (PUCHAR)TrapFrame->Rip;
//
//     /* Check for IRET */
//     if (Instructions[0] == 0x48 && Instructions[1] == 0xCF)
//     {
//         /* Not implemented */
//         UNIMPLEMENTED;
//         ASSERT(FALSE);
//     }
//
//     /* Check for RDMSR/WRMSR */
//     if ((Instructions[0] == 0xF) &&            // 2-byte opcode
//         ((Instructions[1] == 0x30) ||        // RDMSR
//          (Instructions[1] == 0x32)))         // WRMSR
//     {
//         /* Unknown CPU MSR, so raise an access violation */
//         return STATUS_ACCESS_VIOLATION;
//     }
//
//     /* Check for IN/OUT instructions (I/O port access) */
//     if (Instructions[0] == 0xE4 || Instructions[0] == 0xE5 ||  // IN  imm8
//         Instructions[0] == 0xE6 || Instructions[0] == 0xE7 ||  // OUT imm8
//         Instructions[0] == 0xEC || Instructions[0] == 0xED ||  // IN  DX
//         Instructions[0] == 0xEE || Instructions[0] == 0xEF)    // OUT DX
//     {
//         /* I/O port access fault */
//         return STATUS_ACCESS_VIOLATION;
//     }
//
//     /* Unrecognized kernel-mode GPF — log and dispatch as access violation */
//     DPRINT1("KiGeneralProtectionFaultHandler: Unhandled kernel GPF at RIP=%p "
//             "Instr: %02x %02x %02x %02x\n",
//             (PVOID)TrapFrame->Rip,
//             Instructions[0], Instructions[1], Instructions[2], Instructions[3]);
//     return STATUS_ACCESS_VIOLATION;
// }
//
// NTSTATUS
// NTAPI
// KiXmmExceptionHandler(
//     IN PKTRAP_FRAME TrapFrame)
// {
//     ULONG ExceptionCode;
//
//     if ((TrapFrame->MxCsr & _MM_EXCEPT_INVALID) &&
//         !(TrapFrame->MxCsr & _MM_MASK_INVALID))
//     {
//         /* Invalid operation */
//         ExceptionCode = STATUS_FLOAT_INVALID_OPERATION;
//     }
//     else if ((TrapFrame->MxCsr & _MM_EXCEPT_DENORM) &&
//              !(TrapFrame->MxCsr & _MM_MASK_DENORM))
//     {
//         /* Denormalized operand. Yes, this is what Windows returns. */
//         ExceptionCode = STATUS_FLOAT_INVALID_OPERATION;
//     }
//     else if ((TrapFrame->MxCsr & _MM_EXCEPT_DIV_ZERO) &&
//              !(TrapFrame->MxCsr & _MM_MASK_DIV_ZERO))
//     {
//         /* Divide by zero */
//         ExceptionCode = STATUS_FLOAT_DIVIDE_BY_ZERO;
//     }
//     else if ((TrapFrame->MxCsr & _MM_EXCEPT_OVERFLOW) &&
//              !(TrapFrame->MxCsr & _MM_MASK_OVERFLOW))
//     {
//         /* Overflow */
//         ExceptionCode = STATUS_FLOAT_OVERFLOW;
//     }
//     else if ((TrapFrame->MxCsr & _MM_EXCEPT_UNDERFLOW) &&
//              !(TrapFrame->MxCsr & _MM_MASK_UNDERFLOW))
//     {
//         /* Underflow */
//         ExceptionCode = STATUS_FLOAT_UNDERFLOW;
//     }
//     else if ((TrapFrame->MxCsr & _MM_EXCEPT_INEXACT) &&
//              !(TrapFrame->MxCsr & _MM_MASK_INEXACT))
//     {
//         /* Precision */
//         ExceptionCode = STATUS_FLOAT_INEXACT_RESULT;
//     }
//     else
//     {
//         /* Should not happen */
//         ASSERT(FALSE);
//     }
//     
//     return ExceptionCode;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
