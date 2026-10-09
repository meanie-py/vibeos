//! Translation of sdk/lib/rtl/amd64/unwind.c — the x64 unwinding machinery
//! (RtlVirtualUnwind / RtlUnwindEx / RtlWalkFrameChain / RtlRestoreContext).
//!
//! The unwind-op state machine and the epilog instruction scanner are
//! byte-for-byte ports; all instruction-stream reads use read_unaligned.
//! Asm externs: RtlpExecuteHandlerForUnwind + RtlpRestoreContextInternal
//! (amd64/except_asm.S, amd64/unwind-asm.s) stay assembly.

use crate::types::*;
use ntkrnl_sys::ddk::*;
#[allow(unused_imports)]
use crate::types::{DISPATCHER_CONTEXT, PDISPATCHER_CONTEXT, EXCEPTION_DISPOSITION, RtlRaiseStatus};

const UWOP_PUSH_NONVOL: u8 = 0;
const UWOP_ALLOC_LARGE: u8 = 1;
const UWOP_ALLOC_SMALL: u8 = 2;
const UWOP_SET_FPREG: u8 = 3;
const UWOP_SAVE_NONVOL: u8 = 4;
const UWOP_SAVE_NONVOL_FAR: u8 = 5;
const UWOP_EPILOG: u8 = 6; // (UWOP_SAVE_XMM deprecated)
const UWOP_SPARE_CODE: u8 = 7; // (UWOP_SAVE_XMM_FAR deprecated)
const UWOP_SAVE_XMM128: u8 = 8;
const UWOP_SAVE_XMM128_FAR: u8 = 9;
const UWOP_PUSH_MACHFRAME: u8 = 10;

/// UNWIND_CODE — union { struct { UBYTE CodeOffset; UBYTE UnwindOp:4; UBYTE OpInfo:4; };
/// USHORT FrameOffset; } modeled as two raw bytes + accessors.
#[repr(C)]
#[derive(Copy, Clone)]
struct UNWIND_CODE {
    b0: u8,
    b1: u8,
}

impl UNWIND_CODE {
    #[inline]
    fn CodeOffset(self) -> u8 {
        self.b0
    }
    #[inline]
    fn UnwindOp(self) -> u8 {
        self.b1 & 0x0F
    }
    #[inline]
    fn OpInfo(self) -> u8 {
        self.b1 >> 4
    }
    #[inline]
    fn FrameOffset(self) -> u16 {
        u16::from_le_bytes([self.b0, self.b1])
    }
}

/// UNWIND_INFO — bitfield header (Version:3 Flags:5 | SizeOfProlog |
/// CountOfCodes | FrameRegister:4 FrameOffset:4) + flexible UNWIND_CODE array.
#[repr(C)]
struct UNWIND_INFO {
    VersionFlags: u8,
    SizeOfProlog: u8,
    CountOfCodes: u8,
    FrameRegisterOffset: u8,
    UnwindCode: [UNWIND_CODE; 1],
}

impl UNWIND_INFO {
    #[inline]
    fn Flags(&self) -> u8 {
        self.VersionFlags >> 3
    }
    #[inline]
    fn FrameRegister(&self) -> u8 {
        self.FrameRegisterOffset & 0x0F
    }
    #[inline]
    fn FrameOffset(&self) -> u8 {
        self.FrameRegisterOffset >> 4
    }
}

/// `UnwindInfo->UnwindCode[i]` on the flexible array.
#[inline]
unsafe fn unwind_code_at(info: *const UNWIND_INFO, i: usize) -> UNWIND_CODE {
    (core::ptr::addr_of!((*info).UnwindCode) as *const UNWIND_CODE)
        .add(i)
        .read_unaligned()
}

#[inline]
unsafe fn unwind_code_ptr(info: *const UNWIND_INFO, i: usize) -> *const UNWIND_CODE {
    (core::ptr::addr_of!((*info).UnwindCode) as *const UNWIND_CODE).add(i)
}

#[inline]
unsafe fn rva<T>(base: ULONG64, rva: ULONG) -> *mut T {
    (base as usize + rva as usize) as *mut T
}

extern "system" {
    /// amd64/except_asm.S
    fn RtlpExecuteHandlerForUnwind(
        ExceptionRecord: PEXCEPTION_RECORD,
        EstablisherFrame: PVOID,
        ContextRecord: PCONTEXT,
        DispatcherContext: PDISPATCHER_CONTEXT,
    ) -> EXCEPTION_DISPOSITION;

    /// amd64/unwind-asm.s
    fn RtlpRestoreContextInternal(ContextRecord: PCONTEXT);
}

/* RtlLookupFunctionTable
 * \brief Locates the table of RUNTIME_FUNCTION entries for a code address. */
#[no_mangle]
pub unsafe extern "system" fn RtlLookupFunctionTable(
    ControlPc: DWORD64,
    ImageBase: PDWORD64,
    Length: PULONG,
) -> PRUNTIME_FUNCTION {
    let mut Size: ULONG = 0;

    /* Find corresponding file header from code address */
    if RtlPcToFileHeader(ControlPc as PVOID, ImageBase as *mut PVOID).is_null() {
        /* Nothing found */
        return core::ptr::null_mut();
    }

    /* Locate the exception directory */
    let Table = RtlImageDirectoryEntryToData(
        *ImageBase as PVOID,
        1, /* TRUE */
        IMAGE_DIRECTORY_ENTRY_EXCEPTION,
        &mut Size,
    );

    /* Return the number of entries */
    *Length = Size / core::mem::size_of::<RUNTIME_FUNCTION>() as ULONG;

    /* Return the address of the table */
    Table as PRUNTIME_FUNCTION
}

/* RtlLookupFunctionEntry
 * \brief Locates the RUNTIME_FUNCTION entry corresponding to a code address.
 * \todo Implement HistoryTable */
#[no_mangle]
pub unsafe extern "system" fn RtlLookupFunctionEntry(
    ControlPc: DWORD64,
    ImageBase: PDWORD64,
    HistoryTable: *mut UNWIND_HISTORY_TABLE,
) -> PRUNTIME_FUNCTION {
    let mut TableLength: ULONG = 0;

    /* Find the corresponding table */
    let FunctionTable = RtlLookupFunctionTable(ControlPc, ImageBase, &mut TableLength);

    /* If no table is found, try dynamic function tables */
    if FunctionTable.is_null() {
        return crate::amd64::dynfntbl::RtlpLookupDynamicFunctionEntry(
            ControlPc,
            ImageBase,
            HistoryTable,
        );
    }

    /* Use relative virtual address */
    let ControlPc = ControlPc - *ImageBase;

    /* Do a binary search */
    let mut IndexLo: ULONG = 0;
    let mut IndexHi: ULONG = TableLength;
    while IndexHi > IndexLo {
        let IndexMid = (IndexLo + IndexHi) / 2;
        let FunctionEntry = FunctionTable.add(IndexMid as usize);

        if ControlPc < (*FunctionEntry).BeginAddress as DWORD64 {
            /* Continue search in lower half */
            IndexHi = IndexMid;
        } else if ControlPc >= (*FunctionEntry).EndAddress as DWORD64 {
            /* Continue search in upper half */
            IndexLo = IndexMid + 1;
        } else {
            /* ControlPc is within limits, return entry */
            return FunctionEntry;
        }
    }

    /* Nothing found, return NULL */
    core::ptr::null_mut()
}

#[inline]
fn UnwindOpSlots(UnwindCode: UNWIND_CODE) -> ULONG {
    // 16 entries (4-bit UnwindOp) so the index can never bounds-panic — the C
    // table has 12 and reads OOB garbage for ops 12..15, which never occur in
    // valid unwind info; entries 12..15 here are 0 (one slot).
    static UNWIND_OP_EXTRA_SLOT_TABLE: [u8; 16] = [
        0, // UWOP_PUSH_NONVOL
        1, // UWOP_ALLOC_LARGE (or 3, special cased in lookup code)
        0, // UWOP_ALLOC_SMALL
        0, // UWOP_SET_FPREG
        1, // UWOP_SAVE_NONVOL
        2, // UWOP_SAVE_NONVOL_FAR
        1, // UWOP_EPILOG // previously UWOP_SAVE_XMM
        2, // UWOP_SPARE_CODE // previously UWOP_SAVE_XMM_FAR
        1, // UWOP_SAVE_XMM128
        2, // UWOP_SAVE_XMM128_FAR
        0, // UWOP_PUSH_MACHFRAME
        2, // UWOP_SET_FPREG_LARGE
        0, 0, 0, 0, // ops 12..15: invalid, never emitted
    ];

    if (UnwindCode.UnwindOp() == UWOP_ALLOC_LARGE) && (UnwindCode.OpInfo() != 0) {
        3
    } else {
        UNWIND_OP_EXTRA_SLOT_TABLE[UnwindCode.UnwindOp() as usize] as ULONG + 1
    }
}

/* Integer register file: Rax..R15 are contiguous in CONTEXT — indexed access
 * mirrors C's ((DWORD64*)(&Context->Rax))[Reg]. */
#[inline]
unsafe fn SetReg(Context: PCONTEXT, Reg: u8, Value: DWORD64) {
    (core::ptr::addr_of_mut!((*Context).Rax) as *mut DWORD64)
        .add(Reg as usize)
        .write(Value);
}

#[inline]
unsafe fn SetRegFromStackValue(
    Context: PCONTEXT,
    ContextPointers: PKNONVOLATILE_CONTEXT_POINTERS,
    Reg: u8,
    ValuePointer: PDWORD64,
) {
    SetReg(Context, Reg, *ValuePointer);
    if !ContextPointers.is_null() {
        (*ContextPointers).__bindgen_anon_2.IntegerContext[Reg as usize] = ValuePointer;
    }
}

#[inline]
unsafe fn GetReg(Context: PCONTEXT, Reg: u8) -> DWORD64 {
    (core::ptr::addr_of!((*Context).Rax) as *const DWORD64)
        .add(Reg as usize)
        .read()
}

#[inline]
unsafe fn PopReg(Context: PCONTEXT, ContextPointers: PKNONVOLATILE_CONTEXT_POINTERS, Reg: u8) {
    SetRegFromStackValue(Context, ContextPointers, Reg, (*Context).Rsp as PDWORD64);
    (*Context).Rsp += core::mem::size_of::<DWORD64>() as u64;
}

/* Xmm register file: Xmm0..Xmm15 contiguous — C's ((M128A*)(&Context->Xmm0))[Reg]. */
#[inline]
unsafe fn xmm_base(Context: PCONTEXT) -> *mut M128A {
    core::ptr::addr_of_mut!((*Context).__bindgen_anon_1.__bindgen_anon_1.Xmm0)
}

#[inline]
unsafe fn SetXmmReg(Context: PCONTEXT, Reg: u8, Value: M128A) {
    xmm_base(Context).add(Reg as usize).write(Value);
}

#[inline]
unsafe fn SetXmmRegFromStackValue(
    Context: PCONTEXT,
    ContextPointers: PKNONVOLATILE_CONTEXT_POINTERS,
    Reg: u8,
    ValuePointer: *mut M128A,
) {
    SetXmmReg(Context, Reg, *ValuePointer);
    if !ContextPointers.is_null() {
        (*ContextPointers).__bindgen_anon_1.FloatingContext[Reg as usize] = ValuePointer;
    }
}

#[inline]
unsafe fn GetXmmReg(Context: PCONTEXT, Reg: u8) -> M128A {
    xmm_base(Context).add(Reg as usize).read()
}

/* RtlpTryToUnwindEpilog
 * \brief Helper function that tries to unwind epilog instructions.
 * \return TRUE if we have been in an epilog and it could be unwound.
 *         FALSE if the instructions were not allowed for an epilog. */
// Out of line on purpose (the C says `__inline`): this holds a whole CONTEXT
// copy, and inlined that 0x4d0 lands in RtlVirtualUnwind's frame on EVERY
// call, epilog or not. RtlVirtualUnwind runs nested inside RtlpUnwindInternal
// twice over during a dispatch (search pass, then RtlUnwindEx's unwind pass),
// and a STACK_OVERFLOW is dispatched on the one guaranteed page -- the bloat
// is what made amd64 RtlException's overflow undeliverable (exit 128).
#[inline(never)]
unsafe fn RtlpTryToUnwindEpilog(
    Context: PCONTEXT,
    ControlPc: ULONG64,
    ContextPointers: PKNONVOLATILE_CONTEXT_POINTERS,
    ImageBase: ULONG64,
    FunctionEntry: PRUNTIME_FUNCTION,
) -> bool {
    /* Make a local copy of the context */
    let mut LocalContext: CONTEXT = *Context;

    let mut InstrPtr = ControlPc as *const u8;

    /* Check if first instruction of epilog is "add rsp, x" */
    let Instr = (InstrPtr as *const u32).read_unaligned();
    if (Instr & 0x00fffdff) == 0x00c48148 {
        if (Instr & 0x0000ff00) == 0x8300 {
            /* This is "add rsp, 0x??" */
            LocalContext.Rsp += (Instr >> 24) as u64;
            InstrPtr = InstrPtr.add(4);
        } else {
            /* This is "add rsp, 0x???????? */
            LocalContext.Rsp += (InstrPtr.add(3) as *const u32).read_unaligned() as u64;
            InstrPtr = InstrPtr.add(7);
        }
    }
    /* Check if first instruction of epilog is "lea rsp, ..." */
    else if (Instr & 0x38fffe) == 0x208d48 {
        /* Get the register */
        let mut Reg = ((Instr >> 16) & 0x7) as u8;

        /* REX.R */
        Reg += ((Instr & 1) * 8) as u8;

        LocalContext.Rsp = GetReg(&mut LocalContext, Reg);

        /* Get addressing mode */
        let Mod = (Instr >> 22) & 0x3;
        if Mod == 0 {
            /* No displacement */
            InstrPtr = InstrPtr.add(3);
        } else if Mod == 1 {
            /* 1 byte displacement */
            LocalContext.Rsp = LocalContext
                .Rsp
                .wrapping_add(((Instr >> 24) as u8 as i8) as i64 as u64);
            InstrPtr = InstrPtr.add(4);
        } else if Mod == 2 {
            /* 4 bytes displacement */
            LocalContext.Rsp = LocalContext
                .Rsp
                .wrapping_add((InstrPtr.add(3) as *const i32).read_unaligned() as i64 as u64);
            InstrPtr = InstrPtr.add(7);
        }
    }

    /* Loop the following instructions before the ret */
    let EndAddress = (*FunctionEntry).EndAddress as u64 + ImageBase - 1;
    while (InstrPtr as DWORD64) < EndAddress {
        let Instr = (InstrPtr as *const u32).read_unaligned();

        /* Check for a simple pop */
        if (Instr & 0xf8) == 0x58 {
            /* Opcode pops a basic register from stack */
            let Reg = (Instr & 0x7) as u8;
            PopReg(&mut LocalContext, ContextPointers, Reg);
            InstrPtr = InstrPtr.add(1);
            continue;
        }

        /* Check for REX + pop */
        if (Instr & 0xf8fb) == 0x5841 {
            /* Opcode is pop r8 .. r15 */
            let Reg = (((Instr >> 8) & 0x7) + 8) as u8;
            PopReg(&mut LocalContext, ContextPointers, Reg);
            InstrPtr = InstrPtr.add(2);
            continue;
        }

        /* Opcode not allowed for Epilog */
        return false;
    }

    // check for popfq
    // also allow end with jmp imm, jmp [target], iretq

    /* Check if we are at the ret instruction */
    if (InstrPtr as DWORD64) != EndAddress {
        /* If we went past the end of the function, something is broken!
           (ASSERT elided — NDEBUG) */
        return false;
    }

    /* Make sure this is really a ret instruction */
    if *InstrPtr != 0xc3 {
        return false;
    }

    /* Unwind is finished, pop new Rip from Stack */
    LocalContext.Rip = (LocalContext.Rsp as *const DWORD64).read();
    LocalContext.Rsp += core::mem::size_of::<DWORD64>() as u64;

    *Context = LocalContext;
    true
}

/* \ref https://docs.microsoft.com/en-us/cpp/build/unwind-data-definitions-in-c */
unsafe fn GetEstablisherFrame(
    Context: PCONTEXT,
    UnwindInfo: *const UNWIND_INFO,
    CodeOffset: ULONG_PTR,
) -> ULONG64 {
    /* Check if we have a frame register */
    if (*UnwindInfo).FrameRegister() == 0 {
        /* No frame register means we use Rsp */
        return (*Context).Rsp;
    }

    if (CodeOffset >= (*UnwindInfo).SizeOfProlog as ULONG_PTR)
        || (((*UnwindInfo).Flags() as ULONG & UNW_FLAG_CHAININFO) != 0)
    {
        return GetReg(Context, (*UnwindInfo).FrameRegister())
            - (*UnwindInfo).FrameOffset() as u64 * 16;
    }

    /* Loop all unwind ops */
    let mut i: usize = 0;
    while i < (*UnwindInfo).CountOfCodes as usize {
        let code = unwind_code_at(UnwindInfo, i);

        /* Skip codes past our code offset */
        if (code.CodeOffset() as ULONG_PTR) <= CodeOffset {
            /* Check for SET_FPREG */
            if code.UnwindOp() == UWOP_SET_FPREG {
                return GetReg(Context, (*UnwindInfo).FrameRegister())
                    - (*UnwindInfo).FrameOffset() as u64 * 16;
            }
        }

        i += UnwindOpSlots(code) as usize;
    }

    (*Context).Rsp
}

#[no_mangle]
pub unsafe extern "system" fn RtlVirtualUnwind(
    HandlerType: ULONG,
    ImageBase: ULONG64,
    ControlPc: ULONG64,
    FunctionEntry: PRUNTIME_FUNCTION,
    Context: PCONTEXT,
    HandlerData: *mut PVOID,
    EstablisherFrame: PULONG64,
    ContextPointers: PKNONVOLATILE_CONTEXT_POINTERS,
) -> PEXCEPTION_ROUTINE {
    let mut FunctionEntry = FunctionEntry;

    /* Get relative virtual address */
    let ControlRva: ULONG_PTR = (ControlPc - ImageBase) as ULONG_PTR;

    /* Sanity checks */
    if (ControlRva < (*FunctionEntry).BeginAddress as ULONG_PTR)
        || (ControlRva >= (*FunctionEntry).EndAddress as ULONG_PTR)
    {
        return core::mem::transmute::<usize, PEXCEPTION_ROUTINE>(0);
    }

    /* Get a pointer to the unwind info */
    let mut UnwindInfo: *const UNWIND_INFO = rva(ImageBase, (*FunctionEntry).UnwindData);

    /* The language specific handler data follows the unwind info
       (ALIGN_UP_POINTER_BY(&UnwindInfo->UnwindCode[CountOfCodes], sizeof(ULONG))) */
    let LanguageHandler: *const ULONG = (((unwind_code_ptr(
        UnwindInfo,
        (*UnwindInfo).CountOfCodes as usize,
    ) as usize)
        + 3)
        & !3usize) as *const ULONG;

    /* Calculate relative offset to function start */
    let CodeOffset: ULONG_PTR = ControlRva - (*FunctionEntry).BeginAddress as ULONG_PTR;

    *EstablisherFrame = GetEstablisherFrame(Context, UnwindInfo, CodeOffset);

    /* Check if we are in the function epilog and try to finish it */
    if (CodeOffset > (*UnwindInfo).SizeOfProlog as ULONG_PTR) && ((*UnwindInfo).CountOfCodes > 0)
    {
        if RtlpTryToUnwindEpilog(Context, ControlPc, ContextPointers, ImageBase, FunctionEntry) {
            /* There's no exception routine */
            return core::mem::transmute::<usize, PEXCEPTION_ROUTINE>(0);
        }
    }

    /* Skip all Ops with an offset greater than the current Offset */
    let mut i: usize = 0;
    while (i < (*UnwindInfo).CountOfCodes as usize)
        && (unwind_code_at(UnwindInfo, i).CodeOffset() as ULONG_PTR > CodeOffset)
    {
        i += UnwindOpSlots(unwind_code_at(UnwindInfo, i)) as usize;
    }

    let mut machframe_exit = false;

    'RepeatChainedInfo: loop {
        /* Process the remaining unwind ops */
        while i < (*UnwindInfo).CountOfCodes as usize {
            let UnwindCode = unwind_code_at(UnwindInfo, i);
            match UnwindCode.UnwindOp() {
                UWOP_PUSH_NONVOL => {
                    let Reg = UnwindCode.OpInfo();
                    PopReg(Context, ContextPointers, Reg);
                    i += 1;
                }

                UWOP_ALLOC_LARGE => {
                    if UnwindCode.OpInfo() != 0 {
                        let Offset =
                            (unwind_code_ptr(UnwindInfo, i + 1) as *const ULONG).read_unaligned();
                        (*Context).Rsp += Offset as u64;
                        i += 3;
                    } else {
                        let Offset = unwind_code_at(UnwindInfo, i + 1).FrameOffset();
                        (*Context).Rsp += Offset as u64 * 8;
                        i += 2;
                    }
                }

                UWOP_ALLOC_SMALL => {
                    (*Context).Rsp += (UnwindCode.OpInfo() as u64 + 1) * 8;
                    i += 1;
                }

                UWOP_SET_FPREG => {
                    let Reg = (*UnwindInfo).FrameRegister();
                    (*Context).Rsp =
                        GetReg(Context, Reg) - (*UnwindInfo).FrameOffset() as u64 * 16;
                    i += 1;
                }

                UWOP_SAVE_NONVOL => {
                    let Reg = UnwindCode.OpInfo();
                    let Offset = unwind_code_at(UnwindInfo, i + 1).FrameOffset();
                    SetRegFromStackValue(
                        Context,
                        ContextPointers,
                        Reg,
                        ((*Context).Rsp as *mut DWORD64).add(Offset as usize),
                    );
                    i += 2;
                }

                UWOP_SAVE_NONVOL_FAR => {
                    let Reg = UnwindCode.OpInfo();
                    let Offset =
                        (unwind_code_ptr(UnwindInfo, i + 1) as *const ULONG).read_unaligned();
                    SetRegFromStackValue(
                        Context,
                        ContextPointers,
                        Reg,
                        ((*Context).Rsp + Offset as DWORD64) as *mut DWORD64,
                    );
                    i += 3;
                }

                UWOP_EPILOG => {
                    // 2 slots, matching UNWIND_OP_EXTRA_SLOT_TABLE above (1 extra + 1).
                    // Advancing by 1 desynced this loop from UnwindOpSlots(), which the
                    // scan/skip paths use, so the next opcode was read from the wrong
                    // offset. The file disagreed with itself.
                    i += 2;
                }

                UWOP_SPARE_CODE => {
                    /* ASSERT(FALSE) elided — NDEBUG */
                    // 3 slots, likewise matching the table (2 extra + 1).
                    i += 3;
                }

                UWOP_SAVE_XMM128 => {
                    let Reg = UnwindCode.OpInfo();
                    let Offset = unwind_code_at(UnwindInfo, i + 1).FrameOffset();
                    SetXmmRegFromStackValue(
                        Context,
                        ContextPointers,
                        Reg,
                        ((*Context).Rsp as *mut M128A).add(Offset as usize),
                    );
                    i += 2;
                }

                UWOP_SAVE_XMM128_FAR => {
                    let Reg = UnwindCode.OpInfo();
                    let Offset =
                        (unwind_code_ptr(UnwindInfo, i + 1) as *const ULONG).read_unaligned();
                    SetXmmRegFromStackValue(
                        Context,
                        ContextPointers,
                        Reg,
                        ((*Context).Rsp + Offset as DWORD64) as *mut M128A,
                    );
                    i += 3;
                }

                UWOP_PUSH_MACHFRAME => {
                    /* OpInfo is 1, when an error code was pushed, otherwise 0. */
                    (*Context).Rsp +=
                        UnwindCode.OpInfo() as u64 * core::mem::size_of::<DWORD64>() as u64;

                    /* Now pop the MACHINE_FRAME (RIP/RSP only. And yes, "magic
                       numbers", deal with it) */
                    (*Context).Rip = (((*Context).Rsp + 0x00) as PDWORD64).read();
                    (*Context).Rsp = (((*Context).Rsp + 0x18) as PDWORD64).read();
                    /* ASSERT((i + 1) == CountOfCodes) elided — NDEBUG */
                    machframe_exit = true; // goto Exit
                    break;
                }

                _ => {
                    /* Unknown op — C switch falls through silently (no default);
                       no slot-count either, which in C means an infinite loop.
                       Be faithful to "no progress" being impossible: the C only
                       reaches here for ops 11..15 which do not occur in valid
                       unwind info. Advance one slot to avoid hanging. */
                    i += 1;
                }
            }
        }

        if machframe_exit {
            break 'RepeatChainedInfo;
        }

        /* Check for chained info */
        if ((*UnwindInfo).Flags() as ULONG & UNW_FLAG_CHAININFO) != 0 {
            /* See https://docs.microsoft.com/en-us/cpp/build/exception-handling-x64#chained-unwind-info-structures */
            FunctionEntry = unwind_code_ptr(
                UnwindInfo,
                (((*UnwindInfo).CountOfCodes as usize) + 1) & !1usize,
            ) as PRUNTIME_FUNCTION;
            UnwindInfo = rva(ImageBase, (*FunctionEntry).UnwindData);
            i = 0;
            continue 'RepeatChainedInfo;
        }

        /* Unwind is finished, pop new Rip from Stack */
        if (*Context).Rsp != 0 {
            (*Context).Rip = ((*Context).Rsp as *const DWORD64).read();
            (*Context).Rsp += core::mem::size_of::<DWORD64>() as u64;
        }
        break 'RepeatChainedInfo;
    }

    /* Exit: check if we have a handler and return it */
    if ((*UnwindInfo).Flags() as ULONG & (HandlerType & (UNW_FLAG_EHANDLER | UNW_FLAG_UHANDLER)))
        != 0
    {
        *HandlerData = LanguageHandler.add(1) as PVOID;
        return core::mem::transmute::<*mut core::ffi::c_void, PEXCEPTION_ROUTINE>(rva::<
            core::ffi::c_void,
        >(
            ImageBase,
            LanguageHandler.read_unaligned(),
        ));
    }

    core::mem::transmute::<usize, PEXCEPTION_ROUTINE>(0)
}

#[inline]
fn RtlpIsStackPointerValid(StackPointer: ULONG64, LowLimit: ULONG64, HighLimit: ULONG64) -> bool {
    (StackPointer >= LowLimit) && (StackPointer < HighLimit) && ((StackPointer & 7) == 0)
}

/// Personality routine installed by the RtlpExecuteHandlerForUnwind asm thunk
/// (amd64/except_asm.S references this symbol) — must keep its exact name.
#[no_mangle]
pub unsafe extern "system" fn RtlpExecuteHandlerForUnwindHandler(
    ExceptionRecord: PEXCEPTION_RECORD,
    EstablisherFrame: PVOID,
    _ContextRecord: PCONTEXT,
    DispatcherContext: PDISPATCHER_CONTEXT,
) -> EXCEPTION_DISPOSITION {
    /* Get a pointer to the register home space for RtlpExecuteHandlerForUnwind */
    let HomeSpace = (EstablisherFrame as PULONG64).add(6);

    /* Get the ExceptionFlags value, which was saved in the home space */
    let ExceptionFlags = HomeSpace.read() as ULONG;

    /* Get the DispatcherContext, which was saved in the home space */
    let PreviousDispatcherContext = HomeSpace.add(3).read() as PDISPATCHER_CONTEXT;

    /* Check if the original call to RtlpExecuteHandlerForUnwind was an unwind */
    if IS_UNWINDING(ExceptionFlags) {
        /* Check if the current call to this function is due to unwinding */
        if IS_UNWINDING((*ExceptionRecord).ExceptionFlags) {
            /* We are unwinding over the call to a termination handler. This
               could be due to an exception or longjmp. We need to make sure
               to not run this termination handler again. To achieve that,
               we copy the contents of the original dispatcher context back
               over the current dispatcher context and return
               ExceptionCollidedUnwind. RtlUnwindInternal will take the
               original context, and continue unwing there. */
            *DispatcherContext = *PreviousDispatcherContext;
            return ExceptionCollidedUnwind;
        }
    }

    // TODO: properly handle nested exceptions

    ExceptionContinueSearch
}

/*
    \remark The implementation is based on the description in this blog:
            http://www.nynaeve.net/?p=106
*/
#[no_mangle]
pub unsafe extern "system" fn RtlpUnwindInternal(
    TargetFrame: PVOID,
    TargetIp: PVOID,
    ExceptionRecord: PEXCEPTION_RECORD,
    ReturnValue: PVOID,
    ContextRecord: PCONTEXT,
    HistoryTable: *mut UNWIND_HISTORY_TABLE,
    HandlerType: ULONG,
) -> BOOLEAN {
    let mut DispatcherContext: DISPATCHER_CONTEXT = core::mem::zeroed();
    let mut StackLow: ULONG_PTR = 0;
    let mut StackHigh: ULONG_PTR = 0;
    let mut ImageBase: ULONG64 = 0;
    let mut EstablisherFrame: ULONG64 = 0;

    /* Get the current stack limits */
    RtlpGetStackLimits(&mut StackLow, &mut StackHigh);

    /* If we have a target frame, then this is our high limit */
    if !TargetFrame.is_null() {
        StackHigh = TargetFrame as ULONG64 + 1;
    }

    /* Copy the context */
    let mut UnwindContext: CONTEXT = *ContextRecord;

    /* Set up the constant fields of the dispatcher context */
    DispatcherContext.ContextRecord = if HandlerType == UNW_FLAG_UHANDLER {
        ContextRecord
    } else {
        &mut UnwindContext
    };
    DispatcherContext.HistoryTable = HistoryTable;
    DispatcherContext.TargetIp = TargetIp as ULONG64;

    /* Start looping */
    loop {
        if !RtlpIsStackPointerValid(UnwindContext.Rsp, StackLow, StackHigh) {
            return 0;
        }

        /* Lookup the FunctionEntry for the current RIP */
        let FunctionEntry = RtlLookupFunctionEntry(
            UnwindContext.Rip,
            &mut ImageBase,
            core::ptr::null_mut(),
        );
        if FunctionEntry.is_null() {
            /* No function entry, so this must be a leaf function. Pop the
               return address from the stack.
               Note: this can happen after the first frame as the result of
               an exception */
            UnwindContext.Rip = (UnwindContext.Rsp as *const DWORD64).read();
            UnwindContext.Rsp += core::mem::size_of::<DWORD64>() as u64;

            if HandlerType == UNW_FLAG_UHANDLER {
                /* Copy the context back for the next iteration */
                *ContextRecord = UnwindContext;
            }
            continue;
        }

        /* Save Rip before the virtual unwind */
        DispatcherContext.ControlPc = UnwindContext.Rip;

        /* Do a virtual unwind to get the next frame */
        let ExceptionRoutine = RtlVirtualUnwind(
            HandlerType,
            ImageBase,
            UnwindContext.Rip,
            FunctionEntry,
            &mut UnwindContext,
            &mut DispatcherContext.HandlerData,
            &mut EstablisherFrame,
            core::ptr::null_mut(),
        );

        /* Check, if we are still within the stack boundaries */
        if (EstablisherFrame < StackLow as ULONG64)
            || (EstablisherFrame >= StackHigh as ULONG64)
            || ((EstablisherFrame & 7) != 0)
        {
            // TODO: Handle DPC stack

            /* If we are handling an exception, we are done here. */
            if HandlerType == UNW_FLAG_EHANDLER {
                (*ExceptionRecord).ExceptionFlags |= EXCEPTION_STACK_INVALID;
                return 0;
            }

            debugbreak();
            RtlRaiseStatus(STATUS_BAD_STACK);
        }

        /* Check if we have an exception routine */
        if ExceptionRoutine.is_some() {
            /* Check if this is the target frame */
            if EstablisherFrame == TargetFrame as ULONG64 {
                /* Set flag to inform the language handler */
                (*ExceptionRecord).ExceptionFlags |= EXCEPTION_TARGET_UNWIND;
            }

            /* Log the exception if it's enabled */
            RtlpCheckLogException(
                ExceptionRecord,
                &mut UnwindContext,
                &mut DispatcherContext as *mut _ as PVOID,
                core::mem::size_of::<DISPATCHER_CONTEXT>() as ULONG,
            );

            /* Set up the variable fields of the dispatcher context */
            DispatcherContext.ImageBase = ImageBase;
            DispatcherContext.FunctionEntry = FunctionEntry;
            DispatcherContext.LanguageHandler = ExceptionRoutine;
            DispatcherContext.EstablisherFrame = EstablisherFrame;
            DispatcherContext.ScopeIndex = 0;

            /* Store the return value in the unwind context */
            UnwindContext.Rax = ReturnValue as ULONG64;

            /* Loop all nested handlers */
            loop {
                /* Call the language specific handler */
                let Disposition = RtlpExecuteHandlerForUnwind(
                    ExceptionRecord,
                    EstablisherFrame as PVOID,
                    ContextRecord,
                    &mut DispatcherContext,
                );

                /* Clear exception flags for the next iteration */
                (*ExceptionRecord).ExceptionFlags &=
                    !(EXCEPTION_TARGET_UNWIND | EXCEPTION_COLLIDED_UNWIND);

                /* Check if we do exception handling */
                if HandlerType == UNW_FLAG_EHANDLER {
                    if Disposition == ExceptionContinueExecution {
                        /* Check if it was non-continuable */
                        if ((*ExceptionRecord).ExceptionFlags & EXCEPTION_NONCONTINUABLE) != 0 {
                            debugbreak();
                            RtlRaiseStatus(STATUS_NONCONTINUABLE_EXCEPTION);
                        }

                        /* Execution continues */
                        return 1;
                    } else if Disposition == ExceptionNestedException {
                        // TODO
                        debugbreak();
                    }
                }

                if Disposition == ExceptionCollidedUnwind {
                    /* We collided with another unwind, so we need to continue
                       "after" the handler, skipping the termination handler
                       that resulted in this unwind. The installed handler has
                       already copied the original dispatcher context, we now
                       need to copy back the original context. */
                    // Copy through the pointers, never via a CONTEXT temporary:
                    // a `let` here is a second 0x4d0 slot in this frame for its
                    // whole lifetime, and this frame is live twice over on a
                    // stack-overflow dispatch (see RtlpTryToUnwindEpilog). The
                    // source may alias ContextRecord, hence copy(), not
                    // copy_nonoverlapping().
                    core::ptr::copy(DispatcherContext.ContextRecord as *const CONTEXT, ContextRecord, 1);
                    core::ptr::copy_nonoverlapping(ContextRecord as *const CONTEXT, &mut UnwindContext, 1);

                    /* The original context was from "before" the unwind, so we
                       need to do an additional virtual unwind to restore the
                       unwind contxt. */
                    RtlVirtualUnwind(
                        HandlerType,
                        DispatcherContext.ImageBase,
                        UnwindContext.Rip,
                        DispatcherContext.FunctionEntry,
                        &mut UnwindContext,
                        &mut DispatcherContext.HandlerData,
                        &mut EstablisherFrame,
                        core::ptr::null_mut(),
                    );

                    /* Restore the context pointer and establisher frame. */
                    DispatcherContext.ContextRecord = &mut UnwindContext;
                    EstablisherFrame = DispatcherContext.EstablisherFrame;

                    /* Set the exception flags to indicate that we collided
                       with an unwind and continue the handler loop, which
                       will run any additional handlers from the previous
                       unwind. */
                    (*ExceptionRecord).ExceptionFlags |= EXCEPTION_COLLIDED_UNWIND;
                    continue;
                }
                /* This must be ExceptionContinueSearch now */
                else if Disposition != ExceptionContinueSearch {
                    debugbreak();
                    RtlRaiseStatus(STATUS_INVALID_DISPOSITION);
                }

                if ((*ExceptionRecord).ExceptionFlags & EXCEPTION_COLLIDED_UNWIND) == 0 {
                    break;
                }
            }
        }

        /* Check, if we have left our stack (8.) */
        if (EstablisherFrame < StackLow as ULONG64)
            || (EstablisherFrame > StackHigh as ULONG64)
            || ((EstablisherFrame & 7) != 0)
        {
            // TODO: Check for DPC stack
            debugbreak();

            if UnwindContext.Rip == (*ContextRecord).Rip {
                RtlRaiseStatus(STATUS_BAD_FUNCTION_TABLE);
            } else {
                ZwRaiseException(ExceptionRecord, ContextRecord, 0);
            }
        }

        if EstablisherFrame == TargetFrame as ULONG64 {
            break;
        }

        if HandlerType == UNW_FLAG_UHANDLER {
            /* We have successfully unwound a frame. Copy the unwind context back. */
            *ContextRecord = UnwindContext;
        }
    }

    if (*ExceptionRecord).ExceptionCode != STATUS_UNWIND_CONSOLIDATE {
        (*ContextRecord).Rip = TargetIp as ULONG64;
    }

    /* Set the return value */
    (*ContextRecord).Rax = ReturnValue as ULONG64;

    /* Restore the context */
    RtlRestoreContext(ContextRecord, ExceptionRecord);

    /* Should never get here! (ASSERT elided — NDEBUG) */
    #[allow(unreachable_code)]
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlUnwindEx(
    TargetFrame: PVOID,
    TargetIp: PVOID,
    ExceptionRecord: PEXCEPTION_RECORD,
    ReturnValue: PVOID,
    ContextRecord: PCONTEXT,
    HistoryTable: *mut UNWIND_HISTORY_TABLE,
) {
    let mut LocalExceptionRecord: EXCEPTION_RECORD = core::mem::zeroed();
    let mut ExceptionRecord = ExceptionRecord;

    /* Capture the current context */
    RtlCaptureContext(ContextRecord);

    /* Check if we have an exception record */
    if ExceptionRecord.is_null() {
        /* No exception record was passed, so set up a local one */
        LocalExceptionRecord.ExceptionCode = STATUS_UNWIND;
        LocalExceptionRecord.ExceptionAddress = (*ContextRecord).Rip as PVOID;
        LocalExceptionRecord.ExceptionRecord = core::ptr::null_mut();
        LocalExceptionRecord.NumberParameters = 0;
        ExceptionRecord = &mut LocalExceptionRecord;
    }

    /* Set unwind flags */
    (*ExceptionRecord).ExceptionFlags = EXCEPTION_UNWINDING;
    if TargetFrame.is_null() {
        (*ExceptionRecord).ExceptionFlags |= EXCEPTION_EXIT_UNWIND;
    }

    /* Call the internal function */
    RtlpUnwindInternal(
        TargetFrame,
        TargetIp,
        ExceptionRecord,
        ReturnValue,
        ContextRecord,
        HistoryTable,
        UNW_FLAG_UHANDLER,
    );
}

#[no_mangle]
pub unsafe extern "system" fn RtlUnwind(
    TargetFrame: PVOID,
    TargetIp: PVOID,
    ExceptionRecord: PEXCEPTION_RECORD,
    ReturnValue: PVOID,
) {
    let mut Context: CONTEXT = core::mem::zeroed();

    RtlUnwindEx(
        TargetFrame,
        TargetIp,
        ExceptionRecord,
        ReturnValue,
        &mut Context,
        core::ptr::null_mut(),
    );
}

/// NtCurrentTeb — gs:[0x30] on amd64 (same pattern as rtl-sync/src/shared.rs).
#[inline]
unsafe fn NtCurrentTeb() -> PTEB {
    let teb: usize;
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, pure, readonly));
    teb as PTEB
}

#[no_mangle]
pub unsafe extern "system" fn RtlWalkFrameChain(
    Callers: *mut PVOID,
    Count: ULONG,
    Flags: ULONG,
) -> ULONG {
    // RtlWalkFrameChain(NULL, ...) writes no frames and must return 0. The C reference
    // wrapped the walk in _SEH2_TRY/_SEH2_EXCEPT{ i = 0 } so the fault from the NULL
    // output store (`Callers.add(..).write(..)` below) returned 0; that __except is DEAD
    // under DUMMY_PSEH (see the "translated directly" note below), so a NULL Callers AV'd
    // at RtlWalkFrameChain+0x2b0 (the RtlStackKM/RtlStack quarantine). Guard it explicitly.
    if Callers.is_null() {
        return 0;
    }
    let mut Context: CONTEXT = core::mem::zeroed();
    let mut ImageBase: ULONG64 = 0;
    let mut EstablisherFrame: ULONG64 = 0;
    let mut StackLow: ULONG_PTR = 0;
    let mut StackHigh: ULONG_PTR = 0;
    let mut HandlerData: PVOID = core::ptr::null_mut();
    let mut CurrentMode: KPROCESSOR_MODE = RtlpGetMode();

    /* The upper bits in Flags define how many frames to skip */
    let FramesToSkip = Flags >> 8;

    /* Capture the current Context */
    RtlCaptureContext(&mut Context);
    let mut ControlPc = Context.Rip;

    /* Get the stack limits */
    RtlpGetStackLimits(&mut StackLow, &mut StackHigh);
    let mut StackLow = StackLow as ULONG64;
    let mut StackHigh = StackHigh as ULONG64;

    /* (_SEH2_TRY is dead under DUMMY_PSEH — body translated directly) */
    let mut i: ULONG = 0;
    while i < FramesToSkip + Count {
        /* Lookup the FunctionEntry for the current ControlPc */
        let FunctionEntry =
            RtlLookupFunctionEntry(ControlPc, &mut ImageBase, core::ptr::null_mut());

        /* Is this a leaf function? */
        if FunctionEntry.is_null() {
            Context.Rip = (Context.Rsp as *const DWORD64).read();
            Context.Rsp += core::mem::size_of::<DWORD64>() as u64;
        } else {
            RtlVirtualUnwind(
                UNW_FLAG_NHANDLER,
                ImageBase,
                ControlPc,
                FunctionEntry,
                &mut Context,
                &mut HandlerData,
                &mut EstablisherFrame,
                core::ptr::null_mut(),
            );
        }

        /* Check if we are in kernel mode */
        if CurrentMode == KernelMode {
            /* Check if we left the kernel range */
            if Context.Rip < 0xFFFF800000000000 {
                /* Bail out, unless user mode was requested */
                if (Flags & 1) == 0 {
                    break;
                }

                /* We are in user mode now, get UM stack bounds */
                CurrentMode = UserMode;
                let teb = NtCurrentTeb();
                StackLow = (*teb).NtTib.StackLimit as ULONG64;
                StackHigh = (*teb).NtTib.StackBase as ULONG64;
            }
        }

        /* Check (again) if we are in user mode now */
        if CurrentMode == UserMode {
            /* Check if we left the user range */
            if (Context.Rip < 0x10000) || (Context.Rip > 0x000007FFFFFEFFFF) {
                break;
            }
        }

        /* Check, if we have left our stack */
        if (Context.Rsp <= StackLow) || (Context.Rsp >= StackHigh) {
            break;
        }

        /* Continue with new Rip */
        ControlPc = Context.Rip;

        /* Save value, if we are past the frames to skip */
        if i >= FramesToSkip {
            Callers.add((i - FramesToSkip) as usize).write(ControlPc as PVOID);
        }

        i += 1;
    }

    i
}

/* RtlGetCallersAddress
 * \ref http://undocumented.ntinternals.net/UserMode/Undocumented%20Functions/Debug/RtlGetCallersAddress.html */
#[no_mangle]
pub unsafe extern "system" fn RtlGetCallersAddress(
    CallersAddress: *mut PVOID,
    CallersCaller: *mut PVOID,
) {
    let mut Callers: [PVOID; 4] = [core::ptr::null_mut(); 4];

    /* Get callers:
     * RtlWalkFrameChain -> RtlGetCallersAddress -> x -> y */
    let Number = RtlWalkFrameChain(Callers.as_mut_ptr(), 4, 0);

    *CallersAddress = if Number >= 3 {
        Callers[2]
    } else {
        core::ptr::null_mut()
    };
    *CallersCaller = if Number == 4 {
        Callers[3]
    } else {
        core::ptr::null_mut()
    };
}

unsafe fn RtlpCaptureNonVolatileContextPointers(
    NonvolatileContextPointers: PKNONVOLATILE_CONTEXT_POINTERS,
    _TargetFrame: ULONG64,
) {
    let mut Context: CONTEXT = core::mem::zeroed();
    let mut ImageBase: ULONG64 = 0;
    let mut HandlerData: PVOID = core::ptr::null_mut();
    let mut EstablisherFrame: ULONG64 = 0;

    /* Zero out the nonvolatile context pointers */
    core::ptr::write_bytes(NonvolatileContextPointers, 0, 1);

    /* Capture the current context */
    RtlCaptureContext(&mut Context);

    loop {
        /* (kernel-mode sanity ASSERTs elided — NDEBUG) */

        /* Look up the function entry */
        let FunctionEntry =
            RtlLookupFunctionEntry(Context.Rip, &mut ImageBase, core::ptr::null_mut());
        if !FunctionEntry.is_null() {
            /* Do a virtual unwind to the caller and capture saved non-volatiles */
            RtlVirtualUnwind(
                UNW_FLAG_EHANDLER,
                ImageBase,
                Context.Rip,
                FunctionEntry,
                &mut Context,
                &mut HandlerData,
                &mut EstablisherFrame,
                NonvolatileContextPointers,
            );
        } else {
            Context.Rip = (Context.Rsp as *const ULONG64).read();
            Context.Rsp += 8;
        }

        /* Continue until we reach user mode */
        if (Context.Rip as i64) >= 0 {
            break;
        }
    }

    /* ASSERT(EstablisherFrame >= TargetFrame) elided — NDEBUG */
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetUnwindContext(Context: PCONTEXT, TargetFrame: DWORD64) {
    let mut ContextPointers: KNONVOLATILE_CONTEXT_POINTERS = core::mem::zeroed();
    let ContextFlags = (*Context).ContextFlags & !CONTEXT_AMD64;

    /* Capture pointers to the non-volatiles up to the target frame */
    RtlpCaptureNonVolatileContextPointers(&mut ContextPointers, TargetFrame);

    /* Copy the nonvolatiles from the captured locations */
    if (ContextFlags & CONTEXT_INTEGER) != 0 {
        let ip = &ContextPointers.__bindgen_anon_2.__bindgen_anon_1;
        (*Context).Rbx = *ip.Rbx;
        (*Context).Rsi = *ip.Rsi;
        (*Context).Rdi = *ip.Rdi;
        (*Context).R12 = *ip.R12;
        (*Context).R13 = *ip.R13;
        (*Context).R14 = *ip.R14;
        (*Context).R15 = *ip.R15;
    }
    if (ContextFlags & CONTEXT_FLOATING_POINT) != 0 {
        let fp = &ContextPointers.__bindgen_anon_1.__bindgen_anon_1;
        let xmm = &mut (*Context).__bindgen_anon_1.__bindgen_anon_1;
        xmm.Xmm6 = *fp.Xmm6;
        xmm.Xmm7 = *fp.Xmm7;
        xmm.Xmm8 = *fp.Xmm8;
        xmm.Xmm9 = *fp.Xmm9;
        xmm.Xmm10 = *fp.Xmm10;
        xmm.Xmm11 = *fp.Xmm11;
        xmm.Xmm12 = *fp.Xmm12;
        xmm.Xmm13 = *fp.Xmm13;
        xmm.Xmm14 = *fp.Xmm14;
        xmm.Xmm15 = *fp.Xmm15;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetUnwindContext(Context: PCONTEXT, TargetFrame: DWORD64) {
    let mut ContextPointers: KNONVOLATILE_CONTEXT_POINTERS = core::mem::zeroed();
    let ContextFlags = (*Context).ContextFlags & !CONTEXT_AMD64;

    /* Capture pointers to the non-volatiles up to the target frame */
    RtlpCaptureNonVolatileContextPointers(&mut ContextPointers, TargetFrame);

    /* Copy the nonvolatiles to the captured locations */
    if (ContextFlags & CONTEXT_INTEGER) != 0 {
        let ip = &ContextPointers.__bindgen_anon_2.__bindgen_anon_1;
        *ip.Rbx = (*Context).Rbx;
        *ip.Rsi = (*Context).Rsi;
        *ip.Rdi = (*Context).Rdi;
        *ip.R12 = (*Context).R12;
        *ip.R13 = (*Context).R13;
        *ip.R14 = (*Context).R14;
        *ip.R15 = (*Context).R15;
    }
    if (ContextFlags & CONTEXT_FLOATING_POINT) != 0 {
        let fp = &ContextPointers.__bindgen_anon_1.__bindgen_anon_1;
        let xmm = &(*Context).__bindgen_anon_1.__bindgen_anon_1;
        *fp.Xmm6 = xmm.Xmm6;
        *fp.Xmm7 = xmm.Xmm7;
        *fp.Xmm8 = xmm.Xmm8;
        *fp.Xmm9 = xmm.Xmm9;
        *fp.Xmm10 = xmm.Xmm10;
        *fp.Xmm11 = xmm.Xmm11;
        *fp.Xmm12 = xmm.Xmm12;
        *fp.Xmm13 = xmm.Xmm13;
        *fp.Xmm14 = xmm.Xmm14;
        *fp.Xmm15 = xmm.Xmm15;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlRestoreContext(
    ContextRecord: PCONTEXT,
    ExceptionRecord: PEXCEPTION_RECORD,
) {
    if !ExceptionRecord.is_null() {
        if ((*ExceptionRecord).ExceptionCode == STATUS_UNWIND_CONSOLIDATE)
            && ((*ExceptionRecord).NumberParameters >= 1)
        {
            let Consolidate: unsafe extern "system" fn(PEXCEPTION_RECORD) -> PVOID =
                core::mem::transmute((*ExceptionRecord).ExceptionInformation[0]);
            // FIXME: This should be called through an asm wrapper to allow
            // handling recursive unwinding
            (*ContextRecord).Rip = Consolidate(ExceptionRecord) as ULONG64;
        } else if ((*ExceptionRecord).ExceptionCode == STATUS_LONGJUMP)
            && ((*ExceptionRecord).NumberParameters >= 1)
        {
            let JumpBuffer =
                (*ExceptionRecord).ExceptionInformation[0] as *const _JUMP_BUFFER;
            (*ContextRecord).Rbx = (*JumpBuffer).Rbx;
            (*ContextRecord).Rsp = (*JumpBuffer).Rsp;
            (*ContextRecord).Rbp = (*JumpBuffer).Rbp;
            (*ContextRecord).Rsi = (*JumpBuffer).Rsi;
            (*ContextRecord).Rdi = (*JumpBuffer).Rdi;
            (*ContextRecord).R12 = (*JumpBuffer).R12;
            (*ContextRecord).R13 = (*JumpBuffer).R13;
            (*ContextRecord).R14 = (*JumpBuffer).R14;
            (*ContextRecord).R15 = (*JumpBuffer).R15;
            (*ContextRecord).Rip = (*JumpBuffer).Rip;
            (*ContextRecord).MxCsr = (*JumpBuffer).MxCsr;
            (*ContextRecord).__bindgen_anon_1.FltSave.MxCsr = (*JumpBuffer).MxCsr;
            (*ContextRecord).__bindgen_anon_1.FltSave.ControlWord = (*JumpBuffer).FpCsr;
            /* *(M128A*)&JumpBuffer->XmmN */
            let xmm = &mut (*ContextRecord).__bindgen_anon_1.__bindgen_anon_1;
            xmm.Xmm6 = (core::ptr::addr_of!((*JumpBuffer).Xmm6) as *const M128A).read();
            xmm.Xmm7 = (core::ptr::addr_of!((*JumpBuffer).Xmm7) as *const M128A).read();
            xmm.Xmm8 = (core::ptr::addr_of!((*JumpBuffer).Xmm8) as *const M128A).read();
            xmm.Xmm9 = (core::ptr::addr_of!((*JumpBuffer).Xmm9) as *const M128A).read();
            xmm.Xmm10 = (core::ptr::addr_of!((*JumpBuffer).Xmm10) as *const M128A).read();
            xmm.Xmm11 = (core::ptr::addr_of!((*JumpBuffer).Xmm11) as *const M128A).read();
            xmm.Xmm12 = (core::ptr::addr_of!((*JumpBuffer).Xmm12) as *const M128A).read();
            xmm.Xmm13 = (core::ptr::addr_of!((*JumpBuffer).Xmm13) as *const M128A).read();
            xmm.Xmm14 = (core::ptr::addr_of!((*JumpBuffer).Xmm14) as *const M128A).read();
            xmm.Xmm15 = (core::ptr::addr_of!((*JumpBuffer).Xmm15) as *const M128A).read();
        }
    }

    RtlpRestoreContextInternal(ContextRecord);
}
