//! Translation of sdk/lib/rtl/arm64/except.c — user-mode exception support for
//! ARM64 (RtlLookupFunctionEntry / RtlVirtualUnwind / RtlDispatchException /
//! __C_specific_handler / RtlUnwindEx / RtlRestoreContext + dynamic-table stubs).
//!
//! Faithful 1:1 port, including the C's deliberate simplifications (the .xdata
//! path does a leaf unwind `Pc = Lr` rather than decoding unwind codes, and the
//! packed-format handler lookup returns no handler for Flag==2). NDEBUG build:
//! ASSERT/DPRINT are no-ops and are kept as comments.
//!
//! CONTEXT register access goes through the bindgen union: `Context->X[i]` ==
//! `(*Context).__bindgen_anon_1.X[i]` (Fp = X[29], Lr = X[30] — they alias the
//! named fields; X[] indexing is used consistently throughout).

use crate::types::*;
use ntkrnl_sys::ddk::*;

/* Both the bindgen layer and types.rs declare RtlRaiseException; pin the
 * types.rs extern (rtl-diag exception.rs) to resolve the glob ambiguity —
 * the linked symbol is the same either way. */
use crate::types::RtlRaiseException;

use core::ptr::{null_mut, write_volatile};

/* ARM64 exception filter/handler types (file-local typedefs in the C) */
type PARM64_EXCEPTION_FILTER = unsafe extern "C" fn(
    ExceptionPointers: *mut EXCEPTION_POINTERS,
    EstablisherFrame: PVOID,
) -> LONG;

type PARM64_TERMINATION_HANDLER =
    unsafe extern "C" fn(AbnormalTermination: BOOLEAN, EstablisherFrame: PVOID);

/// `RVA(base, rva)` — convert RVA to pointer (the C file's local macro,
/// overriding rtlp.h's swapped-arg version).
#[inline]
fn RVA(base: ULONG64, rva: ULONG) -> PVOID {
    (base as usize + rva as usize) as PVOID
}

/* PRIVATE FUNCTIONS ********************************************************/

/*
 * RtlpLookupFunctionTable - find the .pdata exception table for a code address.
 */
unsafe fn RtlpLookupFunctionTable(
    ControlPc: ULONG64,
    ImageBase: *mut ULONG64,
    Length: *mut ULONG,
) -> PRUNTIME_FUNCTION {
    let mut Size: ULONG = 0;

    /* Find the PE image header for this code address */
    if RtlPcToFileHeader(ControlPc as usize as PVOID, ImageBase as *mut PVOID).is_null() {
        return null_mut();
    }

    /* Locate the exception directory (.pdata) */
    let Table = RtlImageDirectoryEntryToData(
        *ImageBase as usize as PVOID,
        1, /* TRUE */
        IMAGE_DIRECTORY_ENTRY_EXCEPTION,
        &mut Size,
    );
    if Table.is_null() {
        return null_mut();
    }

    write_volatile(
        Length,
        Size / core::mem::size_of::<RUNTIME_FUNCTION>() as ULONG,
    );
    Table as PRUNTIME_FUNCTION
}

/*
 * RtlpGetFunctionEndAddress - compute function end address from RUNTIME_FUNCTION.
 * ARM64 RUNTIME_FUNCTION has no EndAddress field; it must be derived.
 */
unsafe fn RtlpGetFunctionEndAddress(FunctionEntry: PRUNTIME_FUNCTION, ImageBase: ULONG64) -> ULONG {
    let Packed = &(*FunctionEntry).__bindgen_anon_1.__bindgen_anon_1;
    if Packed.Flag() != 0 {
        /* Packed format: FunctionLength is in 4-byte units */
        (*FunctionEntry).BeginAddress + Packed.FunctionLength() * 4
    } else {
        /* .xdata format: first DWORD has FunctionLength in bits [17:0] (in 4-byte units) */
        let XData = RVA(ImageBase, (*FunctionEntry).__bindgen_anon_1.UnwindData) as *const ULONG;
        let FunctionLength = (*XData) & 0x3FFFF;
        (*FunctionEntry).BeginAddress + FunctionLength * 4
    }
}

/* PUBLIC FUNCTIONS **********************************************************/

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeContext(
    _ProcessHandle: HANDLE,
    ThreadContext: PCONTEXT,
    ThreadStartParam: PVOID,
    ThreadStartAddress: PTHREAD_START_ROUTINE,
    StackBase: PINITIAL_TEB,
) {
    core::ptr::write_bytes(ThreadContext, 0, 1); /* RtlZeroMemory */

    (*ThreadContext).Pc = core::mem::transmute::<PTHREAD_START_ROUTINE, ULONG64>(ThreadStartAddress);
    (*ThreadContext).Sp = StackBase as ULONG64;
    (*ThreadContext).Sp &= !15u64;
    (*ThreadContext).__bindgen_anon_1.X[0] = ThreadStartParam as ULONG64;
    (*ThreadContext).Cpsr = 0; /* EL0t */
    (*ThreadContext).ContextFlags = CONTEXT_CONTROL | CONTEXT_INTEGER;
}

/*
 * RtlLookupFunctionEntry - binary search of .pdata for a code address.
 */
#[no_mangle]
pub unsafe extern "system" fn RtlLookupFunctionEntry(
    ControlPc: ULONG64,
    ImageBase: *mut ULONG64,
    _HistoryTable: PVOID,
) -> PRUNTIME_FUNCTION {
    let mut TableLength: ULONG = 0;

    let FunctionTable = RtlpLookupFunctionTable(ControlPc, ImageBase, &mut TableLength);
    if FunctionTable.is_null() {
        write_volatile(ImageBase, 0);
        return null_mut();
    }

    let ControlRva = (ControlPc - *ImageBase) as ULONG;

    /* Binary search */
    let mut IndexLo: ULONG = 0;
    let mut IndexHi: ULONG = TableLength;
    while IndexHi > IndexLo {
        let IndexMid = (IndexLo + IndexHi) / 2;
        let FunctionEntry = FunctionTable.add(IndexMid as usize);

        if ControlRva < (*FunctionEntry).BeginAddress {
            IndexHi = IndexMid;
        } else if ControlRva >= RtlpGetFunctionEndAddress(FunctionEntry, *ImageBase) {
            IndexLo = IndexMid + 1;
        } else {
            return FunctionEntry;
        }
    }

    null_mut()
}

/*
 * RtlVirtualUnwind - unwind one stack frame using ARM64 packed unwind data.
 *
 * For packed entries (Flag != 0), the prologue follows a canonical pattern:
 *   - If CR==3: stp x29, x30 saved (frame pointer + LR)
 *   - Integer registers x19-x(18+RegI) saved
 *   - FP registers d8-d(7+RegF*2) saved (if RegF > 0)
 *   - Total allocation: FrameSize * 16 bytes
 *
 * Save area layout from FP (when CR==3):
 *   [FP+0]:  saved x29, [FP+8]: saved x30
 *   [FP+16]: x19, [FP+24]: x20, ... [FP+16+(i-1)*8]: x(18+i)
 *
 * For CR==0/1 (no frame pointer): saves at top of allocation.
 */
#[no_mangle]
#[allow(unused_assignments)] /* the C's IntSaveSize compute-then-overwrite is kept faithfully */
pub unsafe extern "system" fn RtlVirtualUnwind(
    HandlerType: ULONG,
    ImageBase: ULONG64,
    _ControlPc: ULONG64,
    FunctionEntry: PRUNTIME_FUNCTION,
    Context: PCONTEXT,
    HandlerData: *mut PVOID,
    EstablisherFrame: *mut ULONG64,
    ContextPointers: PVOID,
) -> PEXCEPTION_ROUTINE {
    let NvContext = ContextPointers as PKNONVOLATILE_CONTEXT_POINTERS;
    let SaveBase: ULONG64;
    let mut Offset: ULONG;
    let mut IntSaveSize: ULONG;
    let mut HandlerRoutine: PEXCEPTION_ROUTINE = None;

    write_volatile(HandlerData, null_mut());

    let Packed = (*FunctionEntry).__bindgen_anon_1.__bindgen_anon_1;
    if Packed.Flag() != 0 {
        /* ===== Packed unwind format ===== */
        let Flag = Packed.Flag();
        let RegI = Packed.RegI();
        let RegF = Packed.RegF();
        let H = Packed.H();
        let CR = Packed.CR();
        let FrameSize = Packed.FrameSize();

        /* Compute save area sizes */
        let CrSaveSize: ULONG = if CR == 3 { 16 } else if CR == 1 { 8 } else { 0 };
        /* Integer regs saved: RegI registers, stored in pairs (last may be unpaired) */
        IntSaveSize = ((RegI + 1) / 2) * 16;
        if RegI % 2 != 0 && CR != 3 {
            IntSaveSize -= 8; /* unpaired without CR=3 rounding */
        }
        /* For simplicity, use RegI * 8 and round up to 16 for pairing */
        IntSaveSize = ((RegI + 1) / 2) * 16;
        /* FP regs: if RegF > 0, RegF pairs of d-registers */
        let FpSaveSize: ULONG = if RegF > 0 { RegF * 16 } else { 0 };
        /* Parameter homes */
        let SaveAreaSize: ULONG =
            CrSaveSize + IntSaveSize + FpSaveSize + if H != 0 { 64 } else { 0 };

        /* Establisher frame is the function's SP */
        if CR == 3 {
            /* Frame pointer available: FP points to the FP/LR save area */
            write_volatile(EstablisherFrame, (*Context).__bindgen_anon_1.X[29]); /* FP register */
            SaveBase = (*Context).__bindgen_anon_1.X[29]; /* saves start at FP */
        } else {
            /* No frame pointer: saves at top of frame */
            write_volatile(EstablisherFrame, (*Context).Sp);
            SaveBase = (*Context).Sp + (FrameSize as ULONG64) * 16 - SaveAreaSize as ULONG64;
        }

        /* Restore registers from save area */
        Offset = 0;

        /* CR==3: restore FP and LR from [SaveBase+0] and [SaveBase+8] */
        if CR == 3 {
            let SavedFp = *((SaveBase + 0) as PULONG64);
            let SavedLr = *((SaveBase + 8) as PULONG64);
            (*Context).__bindgen_anon_1.X[29] = SavedFp;
            (*Context).__bindgen_anon_1.X[30] = SavedLr;
            if !NvContext.is_null() {
                (*NvContext).Fp = (SaveBase + 0) as PULONG64;
                (*NvContext).Lr = (SaveBase + 8) as PULONG64;
            }
            Offset = 16;
        }

        /* Restore integer registers x19-x(18+RegI) */
        {
            let NvRegs: *mut PULONG64 = if !NvContext.is_null() {
                &raw mut (*NvContext).X19
            } else {
                null_mut()
            };
            for i in 0..RegI {
                let Slot = (SaveBase + Offset as ULONG64 + i as ULONG64 * 8) as PULONG64;
                /* raw-pointer write — C's Context->X[19 + i] is unchecked; avoid
                   a bounds-check panic path on malformed RegI (12..15) */
                *(*Context).__bindgen_anon_1.X.as_mut_ptr().add(19 + i as usize) = *Slot;
                if !NvRegs.is_null() {
                    *NvRegs.add(i as usize) = Slot;
                }
            }
            Offset += IntSaveSize;
        }

        /* Skip FP registers for now (RegF handling) */
        Offset += FpSaveSize;
        let _ = Offset; /* (final value unused, as in the C) */

        /* Restore SP: caller's SP = current SP + FrameSize*16 */
        (*Context).Sp += (FrameSize as ULONG64) * 16;

        /* Restore PC from LR */
        if CR == 3 {
            /* LR was saved and restored above */
            (*Context).Pc = (*Context).__bindgen_anon_1.X[30];
        } else {
            /* LR register holds return address (leaf or LR not saved) */
            (*Context).Pc = (*Context).__bindgen_anon_1.X[30]; /* Context->Lr */
        }

        /* Check for exception/unwind handler */
        if Flag == 1 {
            /* Packed short: no handler */
            HandlerRoutine = None;
        } else if Flag == 2 {
            /* Packed with .xdata containing handler info */
            /* The handler data follows the RUNTIME_FUNCTION in .xdata */
            /* For now, no handler */
            HandlerRoutine = None;
        }
    } else {
        /* ===== .xdata unwind format ===== */
        let XData = RVA(ImageBase, (*FunctionEntry).__bindgen_anon_1.UnwindData) as *mut ULONG;
        let XDataHeader = *XData;
        let _FuncLen = XDataHeader & 0x3FFFF; /* (void)FuncLen */
        let HasHandler = (XDataHeader >> 20) & 1; /* X bit */
        let SingleEpilog = (XDataHeader >> 21) & 1; /* E bit */
        let EpilogCount = (XDataHeader >> 22) & 0x1F;
        let CodeWords = (XDataHeader >> 27) & 0x1F;

        /* Establisher frame = current SP */
        write_volatile(EstablisherFrame, (*Context).Sp);

        /* For .xdata, we can't easily decode the unwind codes in this initial
         * implementation. Do a simple leaf-function unwind: PC = LR */
        (*Context).Pc = (*Context).__bindgen_anon_1.X[30]; /* Context->Lr */

        /* If handler exists, find it after epilog scopes and unwind codes */
        if HasHandler != 0 {
            let EpilogScopeCount = if SingleEpilog != 0 { 0 } else { EpilogCount };
            let HandlerRva = XData.add(1 + EpilogScopeCount as usize + CodeWords as usize);

            if HandlerType & (UNW_FLAG_EHANDLER | UNW_FLAG_UHANDLER) != 0 {
                HandlerRoutine = core::mem::transmute::<PVOID, PEXCEPTION_ROUTINE>(RVA(
                    ImageBase,
                    *HandlerRva,
                ));
                write_volatile(HandlerData, HandlerRva.add(1) as PVOID);
            }
        }
    }

    HandlerRoutine
}

/*
 * RtlDispatchException - walk the call stack dispatching to exception handlers.
 */
#[no_mangle]
pub unsafe extern "system" fn RtlDispatchException(
    ExceptionRecord: PEXCEPTION_RECORD,
    Context: PCONTEXT,
) -> BOOLEAN {
    let mut StackLow: ULONG_PTR = 0;
    let mut StackHigh: ULONG_PTR = 0;
    let mut ImageBase: ULONG64 = 0;
    let mut EstablisherFrame: ULONG64 = 0;
    let mut Iterations: ULONG = 0;

    RtlpGetStackLimits(&mut StackLow, &mut StackHigh);

    /* Copy context for unwinding */
    let mut UnwindContext: CONTEXT = *Context;

    /* Initialize dispatcher context (RtlZeroMemory) */
    let mut DispatcherContext: DISPATCHER_CONTEXT = core::mem::zeroed();
    DispatcherContext.ContextRecord = Context;

    /* Walk frames */
    while {
        let i = Iterations;
        Iterations += 1;
        i < 200
    } {
        /* Look up the function entry for current PC */
        let FunctionEntry = RtlLookupFunctionEntry(UnwindContext.Pc, &mut ImageBase, null_mut());
        if FunctionEntry.is_null() {
            /* Leaf function: PC = LR, done if LR is 0 */
            if UnwindContext.__bindgen_anon_1.X[30] == 0 {
                break;
            }
            UnwindContext.Pc = UnwindContext.__bindgen_anon_1.X[30];
            UnwindContext.__bindgen_anon_1.X[30] = 0;
            continue;
        }

        /* Virtual unwind to find handler */
        let ExceptionRoutine = RtlVirtualUnwind(
            UNW_FLAG_EHANDLER,
            ImageBase,
            UnwindContext.Pc,
            FunctionEntry,
            &mut UnwindContext,
            &mut DispatcherContext.HandlerData,
            &mut EstablisherFrame,
            null_mut(),
        );

        /* Validate establisher frame */
        if EstablisherFrame < StackLow as ULONG64
            || EstablisherFrame >= StackHigh as ULONG64
            || (EstablisherFrame & 0x7) != 0
        {
            (*ExceptionRecord).ExceptionFlags |= EXCEPTION_STACK_INVALID;
            return 0; /* FALSE */
        }

        /* If there's a handler, call it */
        if let Some(Routine) = ExceptionRoutine {
            DispatcherContext.ControlPc = (*Context).Pc as ULONG_PTR;
            DispatcherContext.ImageBase = ImageBase as ULONG_PTR;
            DispatcherContext.FunctionEntry = FunctionEntry;
            DispatcherContext.EstablisherFrame = EstablisherFrame as ULONG_PTR;
            DispatcherContext.LanguageHandler = ExceptionRoutine;
            DispatcherContext.ScopeIndex = 0;

            let Disposition = Routine(
                ExceptionRecord,
                EstablisherFrame as usize as PVOID,
                Context,
                &mut DispatcherContext as *mut DISPATCHER_CONTEXT as PVOID,
            );

            match Disposition {
                ExceptionContinueExecution => {
                    if (*ExceptionRecord).ExceptionFlags & EXCEPTION_NONCONTINUABLE != 0 {
                        let mut NestedRecord: EXCEPTION_RECORD = core::mem::zeroed();
                        NestedRecord.ExceptionCode = STATUS_NONCONTINUABLE_EXCEPTION;
                        NestedRecord.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
                        NestedRecord.ExceptionRecord = ExceptionRecord;
                        NestedRecord.NumberParameters = 0;
                        RtlRaiseException(&mut NestedRecord);
                    }
                    return 1; /* TRUE */
                }

                ExceptionContinueSearch => {}

                ExceptionNestedException => {
                    (*ExceptionRecord).ExceptionFlags |= EXCEPTION_NESTED_CALL;
                }

                _ => {
                    let mut NestedRecord: EXCEPTION_RECORD = core::mem::zeroed();
                    NestedRecord.ExceptionCode = STATUS_INVALID_DISPOSITION;
                    NestedRecord.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
                    NestedRecord.ExceptionRecord = ExceptionRecord;
                    NestedRecord.NumberParameters = 0;
                    RtlRaiseException(&mut NestedRecord);
                }
            }
        }

        /* If PC is 0, we've reached the end of the stack */
        if UnwindContext.Pc == 0 {
            break;
        }
    }

    0 /* FALSE */
}

/*
 * __C_specific_handler - implements __try/__except/__finally for ARM64.
 */
#[no_mangle]
pub unsafe extern "C" fn __C_specific_handler(
    ExceptionRecord: PEXCEPTION_RECORD,
    EstablisherFrame: PVOID,
    ContextRecord: PCONTEXT,
    DispatcherContext: PDISPATCHER_CONTEXT,
) -> crate::types::EXCEPTION_DISPOSITION {
    let ImageBase: ULONG64 = (*DispatcherContext).ImageBase as ULONG64;
    let ScopeTable = (*DispatcherContext).HandlerData as PSCOPE_TABLE;
    let ControlRva = ((*DispatcherContext).ControlPc as ULONG64 - ImageBase) as ULONG;
    let StartIndex = (*DispatcherContext).ScopeIndex;

    /* Flexible-array access: &ScopeTable->ScopeRecord[0] */
    let ScopeRecord = (&raw const (*ScopeTable).ScopeRecord) as *const SCOPE_RECORD;

    if (*ExceptionRecord).ExceptionFlags & EXCEPTION_UNWIND != 0 {
        /* ===== Unwind phase: call __finally handlers ===== */
        let mut i = StartIndex;
        while i < (*ScopeTable).Count {
            let Scope = ScopeRecord.add(i as usize);
            if ControlRva < (*Scope).BeginAddress || ControlRva >= (*Scope).EndAddress {
                i += 1;
                continue;
            }

            /* JumpTarget == 0 means __finally handler */
            if (*Scope).JumpTarget == 0 {
                let TermHandler: PARM64_TERMINATION_HANDLER = core::mem::transmute::<
                    PVOID,
                    PARM64_TERMINATION_HANDLER,
                >(RVA(ImageBase, (*Scope).HandlerAddress));

                /* Set scope index for potential nested exceptions */
                (*DispatcherContext).ScopeIndex = i + 1;

                /* Call __finally handler: AbnormalTermination = TRUE */
                TermHandler(1 /* TRUE */, EstablisherFrame);
            }
            i += 1;
        }

        ExceptionContinueSearch
    } else {
        /* ===== Exception dispatch phase: find matching __except handler ===== */
        let mut i = StartIndex;
        while i < (*ScopeTable).Count {
            let Scope = ScopeRecord.add(i as usize);
            if ControlRva < (*Scope).BeginAddress || ControlRva >= (*Scope).EndAddress {
                i += 1;
                continue;
            }

            /* JumpTarget != 0 means __except handler */
            if (*Scope).JumpTarget != 0 {
                let FilterResult: LONG;
                if (*Scope).HandlerAddress == 1 {
                    /* EXCEPTION_EXECUTE_HANDLER (constant filter = 1) */
                    FilterResult = EXCEPTION_EXECUTE_HANDLER;
                } else {
                    /* Call the filter function */
                    let mut ExceptionPointers = EXCEPTION_POINTERS {
                        ExceptionRecord,
                        ContextRecord,
                    };

                    let Filter: PARM64_EXCEPTION_FILTER = core::mem::transmute::<
                        PVOID,
                        PARM64_EXCEPTION_FILTER,
                    >(RVA(ImageBase, (*Scope).HandlerAddress));

                    FilterResult = Filter(&mut ExceptionPointers, EstablisherFrame);
                }

                if FilterResult == EXCEPTION_CONTINUE_EXECUTION {
                    return ExceptionContinueExecution;
                } else if FilterResult == EXCEPTION_EXECUTE_HANDLER {
                    let JumpTarget = RVA(ImageBase, (*Scope).JumpTarget);

                    /* Call __finally handlers for unwinding */
                    (*DispatcherContext).ScopeIndex = i + 1;

                    /* Unwind to the __except block */
                    RtlUnwindEx(
                        EstablisherFrame,
                        JumpTarget,
                        ExceptionRecord,
                        (*ExceptionRecord).ExceptionCode as u32 as usize as PVOID, /* UlongToPtr */
                        ContextRecord,
                        (*DispatcherContext).HistoryTable as PVOID,
                    );

                    /* Should not return */
                    /* ASSERT(FALSE) — NDEBUG no-op; C falls through */
                }
                /* EXCEPTION_CONTINUE_SEARCH: try next scope */
            }
            i += 1;
        }

        ExceptionContinueSearch
    }
}

/*
 * RtlUnwindEx - unwind the stack to a target frame, calling unwind handlers.
 */
#[no_mangle]
pub unsafe extern "system" fn RtlUnwindEx(
    TargetFrame: PVOID,
    TargetIp: PVOID,
    ExceptionRecord: PEXCEPTION_RECORD,
    ReturnValue: PVOID,
    ContextRecord: PCONTEXT,
    HistoryTable: PVOID,
) {
    let mut ExceptionRecord = ExceptionRecord;
    let mut LocalRecord: EXCEPTION_RECORD = core::mem::zeroed();
    let mut StackLow: ULONG_PTR = 0;
    let mut StackHigh: ULONG_PTR = 0;
    let mut ImageBase: ULONG64 = 0;
    let mut EstablisherFrame: ULONG64 = 0;
    let mut Iterations: ULONG = 0;

    RtlpGetStackLimits(&mut StackLow, &mut StackHigh);

    /* Create exception record if not provided */
    if ExceptionRecord.is_null() {
        LocalRecord.ExceptionCode = STATUS_UNWIND;
        LocalRecord.ExceptionFlags = 0;
        LocalRecord.ExceptionRecord = null_mut();
        LocalRecord.ExceptionAddress = (*ContextRecord).Pc as usize as PVOID;
        LocalRecord.NumberParameters = 0;
        ExceptionRecord = &mut LocalRecord;
    }

    /* Set unwind flags */
    (*ExceptionRecord).ExceptionFlags |= EXCEPTION_UNWINDING;
    if TargetFrame.is_null() {
        (*ExceptionRecord).ExceptionFlags |= EXCEPTION_EXIT_UNWIND;
    }

    /* Capture current context if not provided.
     * (The capture is immediately overwritten by *ContextRecord — kept
     * faithfully from the C; it is dead but harmless.) */
    let mut UnwindContext: CONTEXT = core::mem::zeroed();
    RtlCaptureContext(&mut UnwindContext);
    UnwindContext = *ContextRecord;

    let mut DispatcherContext: DISPATCHER_CONTEXT = core::mem::zeroed(); /* RtlZeroMemory */
    DispatcherContext.ContextRecord = &mut UnwindContext;
    DispatcherContext.HistoryTable = HistoryTable as *mut _UNWIND_HISTORY_TABLE;
    DispatcherContext.TargetPc = TargetIp as ULONG_PTR;

    /* Walk frames calling unwind handlers */
    while {
        let i = Iterations;
        Iterations += 1;
        i < 200
    } {
        let FunctionEntry = RtlLookupFunctionEntry(UnwindContext.Pc, &mut ImageBase, null_mut());
        if FunctionEntry.is_null() {
            /* Leaf function */
            if UnwindContext.__bindgen_anon_1.X[30] == 0 {
                break;
            }
            UnwindContext.Pc = UnwindContext.__bindgen_anon_1.X[30];
            UnwindContext.__bindgen_anon_1.X[30] = 0;
            continue;
        }

        DispatcherContext.ControlPc = UnwindContext.Pc as ULONG_PTR;

        let ExceptionRoutine = RtlVirtualUnwind(
            UNW_FLAG_UHANDLER,
            ImageBase,
            UnwindContext.Pc,
            FunctionEntry,
            &mut UnwindContext,
            &mut DispatcherContext.HandlerData,
            &mut EstablisherFrame,
            null_mut(),
        );

        /* Validate frame */
        if EstablisherFrame < StackLow as ULONG64
            || EstablisherFrame >= StackHigh as ULONG64
            || (EstablisherFrame & 0x7) != 0
        {
            break;
        }

        if let Some(Routine) = ExceptionRoutine {
            /* Mark target unwind if we reached the target frame */
            if EstablisherFrame == TargetFrame as ULONG64 {
                (*ExceptionRecord).ExceptionFlags |= EXCEPTION_TARGET_UNWIND;
            }

            DispatcherContext.ImageBase = ImageBase as ULONG_PTR;
            DispatcherContext.FunctionEntry = FunctionEntry;
            DispatcherContext.EstablisherFrame = EstablisherFrame as ULONG_PTR;
            DispatcherContext.LanguageHandler = ExceptionRoutine;
            DispatcherContext.ScopeIndex = 0;

            let Disposition = Routine(
                ExceptionRecord,
                EstablisherFrame as usize as PVOID,
                ContextRecord,
                &mut DispatcherContext as *mut DISPATCHER_CONTEXT as PVOID,
            );

            (*ExceptionRecord).ExceptionFlags &=
                !(EXCEPTION_TARGET_UNWIND | EXCEPTION_COLLIDED_UNWIND);

            if Disposition == ExceptionContinueSearch {
                /* Continue to next frame */
            } else if Disposition == ExceptionCollidedUnwind {
                /* Collided unwind: restore and continue */
                UnwindContext = *DispatcherContext.ContextRecord;
            } else {
                break;
            }
        }

        /* Check if we reached the target frame */
        if EstablisherFrame == TargetFrame as ULONG64 {
            break;
        }
    }

    /* Set return value and target IP */
    (*ContextRecord).__bindgen_anon_1.X[0] = ReturnValue as ULONG64;
    if (*ExceptionRecord).ExceptionCode != STATUS_UNWIND_CONSOLIDATE {
        (*ContextRecord).Pc = TargetIp as ULONG64;
    }

    /* Restore context (does not return) */
    RtlRestoreContext(ContextRecord, ExceptionRecord);
}

/*
 * RtlUnwind - simplified unwind (no context record parameter).
 */
#[no_mangle]
pub unsafe extern "system" fn RtlUnwind(
    TargetFrame: PVOID,
    TargetIp: PVOID,
    ExceptionRecord: PEXCEPTION_RECORD,
    ReturnValue: PVOID,
) {
    let mut Context: CONTEXT = core::mem::zeroed();
    RtlCaptureContext(&mut Context);
    RtlUnwindEx(
        TargetFrame,
        TargetIp,
        ExceptionRecord,
        ReturnValue,
        &mut Context,
        null_mut(),
    );
}

/*
 * RtlRestoreContext - restore execution from a CONTEXT.
 */
#[no_mangle]
pub unsafe extern "system" fn RtlRestoreContext(
    ContextRecord: PCONTEXT,
    ExceptionRecord: PEXCEPTION_RECORD,
) {
    if !ExceptionRecord.is_null()
        && (*ExceptionRecord).ExceptionCode == STATUS_UNWIND_CONSOLIDATE
        && (*ExceptionRecord).NumberParameters >= 1
    {
        let Consolidate: unsafe extern "C" fn(PEXCEPTION_RECORD) -> PVOID =
            core::mem::transmute::<ULONG_PTR, unsafe extern "C" fn(PEXCEPTION_RECORD) -> PVOID>(
                (*ExceptionRecord).ExceptionInformation[0],
            );
        (*ContextRecord).Pc = Consolidate(ExceptionRecord) as ULONG64;
    }

    /* Use NtContinue to restore the context */
    NtContinue(ContextRecord, 0 /* FALSE */);
}

/* Dynamic function table stubs (not needed for initial boot) */

#[no_mangle]
pub unsafe extern "system" fn RtlAddFunctionTable(
    _FunctionTable: PRUNTIME_FUNCTION,
    _EntryCount: ULONG,
    _BaseAddress: ULONG64,
) -> BOOLEAN {
    1 /* TRUE */
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteFunctionTable(
    _FunctionTable: PRUNTIME_FUNCTION,
) -> BOOLEAN {
    1 /* TRUE */
}

#[no_mangle]
pub unsafe extern "system" fn RtlInstallFunctionTableCallback(
    _TableIdentifier: ULONG64,
    _BaseAddress: ULONG64,
    _Length: ULONG,
    _Callback: PVOID,
    _Context: PVOID,
    _OutOfProcessCallbackDll: PCWSTR,
) -> BOOLEAN {
    1 /* TRUE */
}

/* EOF */
