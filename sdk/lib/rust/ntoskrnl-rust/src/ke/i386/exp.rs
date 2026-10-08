use crate::ke::ke_defs::*;
// Explicit import to disambiguate KeGetCurrentThread from ke_defs vs mm globs
use ntkrnl_sys::mm::KeGetCurrentThread;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::HANDLE;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::macros::NtCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _EPROCESS,
    _ETHREAD,
    _KTHREAD,
    _KTRAP_FRAME,
    _KEXCEPTION_FRAME,
    _CONTEXT,
    _EXCEPTION_RECORD,
    _LDR_DATA_TABLE_ENTRY,
    _PEB,
    _PEB_LDR_DATA,
    _TEB,
    PCONTEXT,
    PEXCEPTION_RECORD,
    PKEXCEPTION_FRAME,
    PKTHREAD,
    PKTRAP_FRAME,
    PLDR_DATA_TABLE_ENTRY,
    PPEB,
    PPEB_LDR_DATA,
    PTEB,
    KPROCESSOR_MODE,
    CONTEXT,
    EXCEPTION_RECORD,
    KTRAP_FRAME,
    DbgkForwardException,
    ZwTerminateProcess,
    MmIsAddressValid,
    PsGetCurrentProcessId,
    RtlDispatchException,
    KeBugCheckWithTf,
};

// KiDebugRoutine is a global function pointer variable (PKDEBUG_ROUTINE) in kddata.c,
// NOT a function itself. Must be extern "C" (no stdcall decoration).
type PKDEBUG_ROUTINE = unsafe extern "system" fn(
    PKTRAP_FRAME, PKEXCEPTION_FRAME, PEXCEPTION_RECORD,
    PCONTEXT, KPROCESSOR_MODE, BOOLEAN,
) -> BOOLEAN;
extern "C" {
    static mut KiDebugRoutine: PKDEBUG_ROUTINE;
}

extern "system" {
    static mut KiIdt: [IDT_ENTRY; 256];
    static mut KeI386FxsrPresent: BOOLEAN;
    static mut KiMXCsrMask: ULONG;
    static mut MmHighestUserAddress: PVOID;
    static mut KdIgnoreUmExceptions: BOOLEAN;
    static mut KeUserExceptionDispatcher: PVOID;
    static mut KeRaiseUserExceptionDispatcher: PVOID;
    fn Ke386SanitizeFlags(Flags: ULONG, Mode: KPROCESSOR_MODE) -> ULONG;
    fn Ke386SanitizeSeg(Seg: ULONG, Mode: KPROCESSOR_MODE) -> ULONG;
    fn KiFlushNPXState(SaveArea: *mut FLOATING_SAVE_AREA);
    fn KiUserTrap(TrapFrame: PKTRAP_FRAME) -> KPROCESSOR_MODE;
    fn KdIsThisAKdTrap( ExceptionRecord: PEXCEPTION_RECORD, Context: PCONTEXT, PreviousMode: KPROCESSOR_MODE, ) -> BOOLEAN;
    fn __readeflags() -> ULONG;
    fn __writeeflags(Value: ULONG);
    fn _disable();
    fn ExpInterlockedPopEntrySListFault();
}
extern "C" {
    fn KeGetPcr() -> *mut KPCR;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiEoiHelper(TrapFrame: PKTRAP_FRAME) -> !;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiEoiHelper(TrapFrame: PKTRAP_FRAME) -> !;
}

#[repr(C)]
struct IDT_ENTRY {
    Offset: USHORT,
    Selector: USHORT,
    Access: UCHAR,
    Reserved: UCHAR,
    ExtendedOffset: USHORT,
}

#[repr(C)]
struct KPCR {
    TSS: *mut KTSS,
    KdVersionBlock: PVOID,
}

#[repr(C)]
struct KTSS {
    Esp0: ULONG,
}

#[repr(C)]
struct FX_SAVE_AREA {
    U: FX_SAVE_AREA_U,
    /// NpxSavedCpu@+0x208, Cr0NpxState@+0x20C — correct once FXSAVE_FORMAT is the
    /// full 520-byte C layout (see FXSAVE_FORMAT below). A short union U silently
    /// slides both fields low and corrupts NPX save/restore.
    NpxSavedCpu: ULONG,
    Cr0NpxState: ULONG,
}

#[repr(C)]
union FX_SAVE_AREA_U {
    FxArea: FXSAVE_FORMAT,
    FnArea: FNSAVE_FORMAT,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FXSAVE_FORMAT {
    ControlWord: USHORT,
    StatusWord: USHORT,
    TagWord: USHORT,
    ErrorOpcode: USHORT,
    ErrorOffset: ULONG,
    ErrorSelector: USHORT,
    Reserved1: USHORT,
    DataOffset: ULONG,
    DataSelector: USHORT,
    Reserved2: USHORT,
    MXCsr: ULONG,
    Reserved3: ULONG,
    RegisterArea: [UCHAR; 128],
    // C FXSAVE_FORMAT (ndk/i386/ketypes.h) is 520 bytes: Reserved3[128],
    // Reserved4[224], Align16Byte[8] follow RegisterArea[128]. Truncating the
    // tail to 128 made this struct 288 bytes, so U was 288 and Cr0NpxState landed
    // at +0x124 instead of +0x20C — KeContextToTrapFrame's Cr0NpxState sanitize
    // and KeTrapFrameToContext both hit the wrong dword. Match bindgen (size 520).
    Reserved4: [UCHAR; 128],
    Reserved5: [UCHAR; 224],
    Align16Byte: [UCHAR; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FNSAVE_FORMAT {
    ControlWord: ULONG,
    StatusWord: ULONG,
    TagWord: ULONG,
    ErrorOffset: ULONG,
    ErrorSelector: ULONG,
    DataOffset: ULONG,
    DataSelector: ULONG,
    RegisterArea: [UCHAR; 80],
}

#[repr(C)]
struct FLOATING_SAVE_AREA {
    ControlWord: ULONG,
    StatusWord: ULONG,
    TagWord: ULONG,
    ErrorOffset: ULONG,
    ErrorSelector: ULONG,
    DataOffset: ULONG,
    DataSelector: ULONG,
    RegisterArea: [UCHAR; 80],
    Cr0NpxState: ULONG,
}

type PFX_SAVE_AREA = *mut FX_SAVE_AREA;

const MAXIMUM_IDTVECTOR: usize = 255;
const MAXIMUM_SUPPORTED_EXTENSION: usize = 512;
const SIZE_OF_80387_REGISTERS: usize = 80;
const SIZE_OF_FX_REGISTERS: usize = 128;

const DR_MASK_0: ULONG = 1 << 0;
const DR_MASK_1: ULONG = 1 << 1;
const DR_MASK_2: ULONG = 1 << 2;
const DR_MASK_3: ULONG = 1 << 3;
const DR_MASK_6: ULONG = 1 << 6;
const DR_MASK_7: ULONG = 1 << 7;
const DR_MASK_DR7_OVERRIDE_V: ULONG = 1 << 8;
const DR_REG_MASK: ULONG = DR_MASK_0 | DR_MASK_1 | DR_MASK_2 | DR_MASK_3 | DR_MASK_6 | DR_MASK_7;

const DR7_RESERVED_MASK: ULONG = 0x0000DC00;
const DR7_OVERRIDE_MASK: ULONG = 0x000F0000;
const DR6_LEGAL: ULONG = 0x0000E00F;
const DR7_LEGAL: ULONG = 0xFFFF0155;

const EFLAGS_V86_MASK: ULONG = 0x00020000;
const EFLAGS_IOPL: ULONG = 0x00003000;
const FRAME_EDITED: ULONG = 0x0000FFF8;
const RPL_MASK: ULONG = 0x00000003;

const KGDT_R0_CODE: ULONG = 0x00000008;
const KGDT_R0_DATA: ULONG = 0x00000010;
const KGDT_R0_PCR: ULONG = 0x00000030;
const KGDT_R3_CODE: ULONG = 0x00000018;
const KGDT_R3_DATA: ULONG = 0x00000020;
const KGDT_R3_TEB: ULONG = 0x00000038;

const CR0_EM: ULONG = 0x00000004;
const CR0_MP: ULONG = 0x00000002;
const CR0_TS: ULONG = 0x00000008;

// CONTEXT_i386 (0x10000) | flag bits -- winnt_old.h:1291-1303. Dropping the arch
// base leaves a mask that still matches well-formed same-arch ContextFlags by luck
// (the discriminating bits are the low ones), but accepts a context record carrying
// a DIFFERENT arch base, which is exactly what WoW64 hands the 64-bit side.
const CONTEXT_CONTROL: ULONG = 0x00010001;
const CONTEXT_INTEGER: ULONG = 0x00010002;
const CONTEXT_SEGMENTS: ULONG = 0x00010004;
const CONTEXT_FLOATING_POINT: ULONG = 0x00010008;
const CONTEXT_DEBUG_REGISTERS: ULONG = 0x00010010;
const CONTEXT_EXTENDED_REGISTERS: ULONG = 0x00010020;
const CONTEXT_FULL: ULONG = CONTEXT_CONTROL | CONTEXT_INTEGER | CONTEXT_SEGMENTS;

const STATUS_BREAKPOINT: NTSTATUS = 0x80000003u32 as NTSTATUS;
const KI_EXCEPTION_ACCESS_VIOLATION: NTSTATUS = 0x10000004_u32 as NTSTATUS;
const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005u32 as NTSTATUS;
const STATUS_STACK_OVERFLOW: NTSTATUS = 0xC00000FDu32 as NTSTATUS;
const SET_OF_INVALID_CONTEXT: ULONG = 0x30;
const KMODE_EXCEPTION_NOT_HANDLED: ULONG = 0x0000001E;
const UNEXPECTED_KERNEL_MODE_TRAP: ULONG = 0x0000007F;

fn DR_MASK(n: usize) -> ULONG {
    1 << n
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KeInitExceptions() {
    let mut i = 0;
    while i <= MAXIMUM_IDTVECTOR {
        let flipped_selector = (*core::ptr::addr_of!(KiIdt[i])).Selector;
        (*core::ptr::addr_of_mut!(KiIdt[i])).Selector = (*core::ptr::addr_of!(KiIdt[i])).ExtendedOffset;
        (*core::ptr::addr_of_mut!(KiIdt[i])).ExtendedOffset = flipped_selector;
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "fastcall" fn KiUpdateDr7(Dr7: ULONG) -> ULONG {
    let debug_mask = (*(KeGetCurrentThread() as *mut _KTHREAD)).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.__bindgen_anon_1.DebugActive as ULONG;

    if (debug_mask & DR_MASK(8)) != 0 {
        ntkrnl_sys::ASSERT!((debug_mask & DR_REG_MASK) != 0);
        ntkrnl_sys::ASSERT!((Dr7 & !DR7_RESERVED_MASK) == DR7_OVERRIDE_MASK);
        return 0;
    }

    Dr7
}

#[no_mangle]
pub unsafe extern "fastcall" fn KiRecordDr7(Dr7Ptr: *mut ULONG, DrMask: *mut ULONG) -> BOOLEAN {
    let Dr7Ptr = opaque_ptr(Dr7Ptr);
    let DrMask = opaque_ptr(DrMask);
    let mask = if DrMask.is_null() {
        (*(KeGetCurrentThread() as *mut _KTHREAD)).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.__bindgen_anon_1.DebugActive as ULONG
    } else {
        *DrMask
    };

    ntkrnl_sys::ASSERT!((*Dr7Ptr & DR7_RESERVED_MASK) == 0);

    let mut new_mask = mask;
    let result: BOOLEAN;

    if *Dr7Ptr == 0 {
        result = 0u8;

        new_mask &= !DR_MASK(7);
        if (new_mask & DR_REG_MASK) != 0 {
            new_mask |= DR_MASK(8);
            *Dr7Ptr |= DR7_OVERRIDE_MASK;
        } else {
            ntkrnl_sys::ASSERT!(new_mask == 0);
        }
    } else {
        result = if new_mask != 0 { 1u8 } else { 0u8 };
        new_mask &= !DR_MASK(8);
        new_mask |= DR_MASK(7);
    }

    if !DrMask.is_null() {
        *DrMask = new_mask;
    } else {
        if mask != new_mask {
            (*(KeGetCurrentThread() as *mut _KTHREAD)).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.__bindgen_anon_1.DebugActive = new_mask as UCHAR;
        }
    }

    result
}

unsafe fn KiEspFromTrapFrame(TrapFrame: PKTRAP_FRAME) -> ULONG {
    if KiUserTrap(TrapFrame) != 0 || ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 {
        (*TrapFrame).HardwareEsp
    } else {
        if ((*TrapFrame).SegCs & FRAME_EDITED) == 0 {
            (*TrapFrame).TempEsp
        } else {
            core::ptr::addr_of!((*TrapFrame).HardwareEsp) as ULONG
        }
    }
}

unsafe fn KiEspToTrapFrame(TrapFrame: PKTRAP_FRAME, Esp: ULONG) {
    let old_irql = KeGetCurrentIrql();
    let mut raised_irql = old_irql;
    if old_irql < APC_LEVEL {
        KeRaiseIrql(APC_LEVEL, core::ptr::addr_of_mut!(raised_irql));
    }

    let previous = KiEspFromTrapFrame(TrapFrame);

    if KiUserTrap(TrapFrame) != 0 || ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 {
        (*TrapFrame).HardwareEsp = Esp;
    } else {
        if Esp < previous {
            KeBugCheckEx(
                SET_OF_INVALID_CONTEXT,
                Esp as ULONG_PTR,
                previous as ULONG_PTR,
                TrapFrame as ULONG_PTR,
                0,
            );
        }

        if ((*TrapFrame).SegCs & FRAME_EDITED) == 0 {
            (*TrapFrame).TempEsp = Esp;
        } else {
            if previous != Esp {
                (*TrapFrame).TempSegCs = (*TrapFrame).SegCs;
                (*TrapFrame).SegCs &= !FRAME_EDITED;
                (*TrapFrame).TempEsp = Esp;
            }
        }
    }

    if old_irql < APC_LEVEL {
        KeLowerIrql(old_irql);
    }
}

unsafe fn KiSsFromTrapFrame(TrapFrame: PKTRAP_FRAME) -> ULONG {
    if ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 {
        (*TrapFrame).HardwareSegSs
    } else if KiUserTrap(TrapFrame) != 0 {
        (*TrapFrame).HardwareSegSs | RPL_MASK
    } else {
        KGDT_R0_DATA
    }
}

unsafe fn KiSsToTrapFrame(TrapFrame: PKTRAP_FRAME, mut Ss: ULONG) {
    Ss &= 0xFFFF;

    if ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 {
        (*TrapFrame).HardwareSegSs = Ss;
    } else if KiUserTrap(TrapFrame) != 0 {
        (*TrapFrame).HardwareSegSs = Ss | RPL_MASK;
    }
}

unsafe fn KiTagWordFnsaveToFxsave(TagWord: USHORT) -> USHORT {
    let mut fx_tag_word = !TagWord as i32;

    fx_tag_word = (fx_tag_word | (fx_tag_word >> 1)) & 0x5555;
    fx_tag_word = (fx_tag_word | (fx_tag_word >> 1)) & 0x3333;
    fx_tag_word = (fx_tag_word | (fx_tag_word >> 2)) & 0x0f0f;
    fx_tag_word = (fx_tag_word | (fx_tag_word >> 4)) & 0x00ff;
    fx_tag_word as USHORT
}

#[no_mangle]
pub unsafe extern "system" fn Ki386AdjustEsp0(TrapFrame: PKTRAP_FRAME) {
    let thread = KeGetCurrentThread() as *mut _KTHREAD;
    let mut stack = (*thread).InitialStack as ULONG_PTR;

    if ((*TrapFrame).EFlags & EFLAGS_V86_MASK) == 0 {
        stack -= core::mem::size_of::<KTRAP_FRAME>() - 
                 (core::mem::offset_of!(KTRAP_FRAME, V86Es));
    }

    stack -= core::mem::size_of::<FX_SAVE_AREA>();

    let eflags = __readeflags();
    _disable();

    (*(*KeGetPcr()).TSS).Esp0 = stack as ULONG;

    __writeeflags(eflags);
}

#[no_mangle]
pub unsafe extern "system" fn KeContextToTrapFrame(
    Context: PCONTEXT,
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
    ContextFlags: ULONG,
    PreviousMode: KPROCESSOR_MODE,
) {
    let mut v86_switch = 0u8;
    let old_irql = KeGetCurrentIrql();
    let mut raised_irql = old_irql;
    if old_irql < APC_LEVEL {
        KeRaiseIrql(APC_LEVEL, core::ptr::addr_of_mut!(raised_irql));
    }

    let mut dr_mask: ULONG = 0;

    if (ContextFlags & CONTEXT_CONTROL) == CONTEXT_CONTROL {
        if ((*Context).EFlags & EFLAGS_V86_MASK) != ((*TrapFrame).EFlags & EFLAGS_V86_MASK) {
            v86_switch = 1u8;
        }

        (*TrapFrame).EFlags = Ke386SanitizeFlags((*Context).EFlags, PreviousMode);
        (*TrapFrame).Ebp = (*Context).Ebp;
        (*TrapFrame).Eip = (*Context).Eip;

        if ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 {
            (*TrapFrame).SegCs = (*Context).SegCs;
        } else {
            (*TrapFrame).SegCs = Ke386SanitizeSeg((*Context).SegCs, PreviousMode);

            if PreviousMode != 0 && (*TrapFrame).SegCs < 8 {
                (*TrapFrame).SegCs = KGDT_R3_CODE | RPL_MASK;
            }
        }

        KiSsToTrapFrame(TrapFrame, (*Context).SegSs);
        KiEspToTrapFrame(TrapFrame, (*Context).Esp);

        if v86_switch != 0 {
            Ki386AdjustEsp0(TrapFrame);
        }
    }

    if (ContextFlags & CONTEXT_INTEGER) == CONTEXT_INTEGER {
        (*TrapFrame).Eax = (*Context).Eax;
        (*TrapFrame).Ebx = (*Context).Ebx;
        (*TrapFrame).Ecx = (*Context).Ecx;
        (*TrapFrame).Edx = (*Context).Edx;
        (*TrapFrame).Esi = (*Context).Esi;
        (*TrapFrame).Edi = (*Context).Edi;
    }

    if (ContextFlags & CONTEXT_SEGMENTS) == CONTEXT_SEGMENTS {
        if ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 {
            (*TrapFrame).V86Ds = (*Context).SegDs;
            (*TrapFrame).V86Es = (*Context).SegEs;
            (*TrapFrame).V86Fs = (*Context).SegFs;
            (*TrapFrame).V86Gs = (*Context).SegGs;
        } else if KiUserTrap(TrapFrame) == 0 {
            (*TrapFrame).SegDs = KGDT_R3_DATA | RPL_MASK;
            (*TrapFrame).SegEs = KGDT_R3_DATA | RPL_MASK;
            (*TrapFrame).SegFs = Ke386SanitizeSeg((*Context).SegFs, PreviousMode);
            (*TrapFrame).SegGs = 0;
        } else {
            (*TrapFrame).SegDs = (*Context).SegDs;
            (*TrapFrame).SegEs = (*Context).SegEs;
            (*TrapFrame).SegFs = (*Context).SegFs;

            if (*TrapFrame).SegCs == (KGDT_R3_CODE | RPL_MASK) {
                (*TrapFrame).SegGs = 0;
            } else {
                (*TrapFrame).SegGs = (*Context).SegGs;
            }
        }
    }

    if ((ContextFlags & CONTEXT_EXTENDED_REGISTERS) == CONTEXT_EXTENDED_REGISTERS) && KiUserTrap(TrapFrame) != 0 {
        let fx_save_area = (TrapFrame as usize + core::mem::size_of::<KTRAP_FRAME>()) as PFX_SAVE_AREA;

        KiFlushNPXState(core::ptr::null_mut());

        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*Context).ExtendedRegisters[0]) as *const u8,
            core::ptr::addr_of_mut!((*fx_save_area).U.FxArea) as *mut u8,
            MAXIMUM_SUPPORTED_EXTENSION,
        );

        (*fx_save_area).U.FxArea.MXCsr &= KiMXCsrMask;
        (*fx_save_area).Cr0NpxState &= !(CR0_EM | CR0_MP | CR0_TS);

        if !(*(PsGetCurrentProcess() as *mut _EPROCESS)).VdmObjects.is_null() {
            (*fx_save_area).Cr0NpxState |= (*Context).FloatSave.Cr0NpxState & (CR0_EM | CR0_MP);
        }
    }

    if ((ContextFlags & CONTEXT_FLOATING_POINT) == CONTEXT_FLOATING_POINT) && KiUserTrap(TrapFrame) != 0 {
        let fx_save_area = (TrapFrame as usize + core::mem::size_of::<KTRAP_FRAME>()) as PFX_SAVE_AREA;

        KiFlushNPXState(core::ptr::null_mut());

        if KeI386FxsrPresent != 0 {
            (*fx_save_area).U.FxArea.ControlWord = (*Context).FloatSave.ControlWord as USHORT;
            (*fx_save_area).U.FxArea.StatusWord = (*Context).FloatSave.StatusWord as USHORT;
            (*fx_save_area).U.FxArea.TagWord = KiTagWordFnsaveToFxsave((*Context).FloatSave.TagWord as USHORT);
            (*fx_save_area).U.FxArea.ErrorOpcode = (((*Context).FloatSave.ErrorSelector >> 16) & 0xFFFF) as USHORT;
            (*fx_save_area).U.FxArea.ErrorOffset = (*Context).FloatSave.ErrorOffset;
            (*fx_save_area).U.FxArea.ErrorSelector = ((*Context).FloatSave.ErrorSelector & 0xFFFF) as USHORT;
            (*fx_save_area).U.FxArea.DataOffset = (*Context).FloatSave.DataOffset;
            (*fx_save_area).U.FxArea.DataSelector = (*Context).FloatSave.DataSelector as USHORT;

            core::ptr::write_bytes(
                core::ptr::addr_of_mut!((*fx_save_area).U.FxArea.RegisterArea[0]) as *mut u8,
                0,
                SIZE_OF_FX_REGISTERS,
            );

            for i in 0..8 {
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!((*Context).FloatSave.RegisterArea[i * 10]) as *const u8,
                    (core::ptr::addr_of_mut!((*fx_save_area).U.FxArea.RegisterArea[i * 16]) as *mut u8),
                    10,
                );
            }
        } else {
            (*fx_save_area).U.FnArea.ControlWord = (*Context).FloatSave.ControlWord;
            (*fx_save_area).U.FnArea.StatusWord = (*Context).FloatSave.StatusWord;
            (*fx_save_area).U.FnArea.TagWord = (*Context).FloatSave.TagWord;
            (*fx_save_area).U.FnArea.ErrorOffset = (*Context).FloatSave.ErrorOffset;
            (*fx_save_area).U.FnArea.ErrorSelector = (*Context).FloatSave.ErrorSelector;
            (*fx_save_area).U.FnArea.DataOffset = (*Context).FloatSave.DataOffset;
            (*fx_save_area).U.FnArea.DataSelector = (*Context).FloatSave.DataSelector;

            for i in 0..SIZE_OF_80387_REGISTERS {
                (*fx_save_area).U.FnArea.RegisterArea[i] = (*Context).FloatSave.RegisterArea[i];
            }
        }

        (*fx_save_area).Cr0NpxState &= !(CR0_EM | CR0_MP | CR0_TS);

        if !(*(PsGetCurrentProcess() as *mut _EPROCESS)).VdmObjects.is_null() {
            (*fx_save_area).Cr0NpxState |= (*Context).FloatSave.Cr0NpxState & (CR0_EM | CR0_MP);
        }
    }

    if (ContextFlags & CONTEXT_DEBUG_REGISTERS) == CONTEXT_DEBUG_REGISTERS {
        (*TrapFrame).Dr0 = (*Context).Dr0;
        (*TrapFrame).Dr1 = (*Context).Dr1;
        (*TrapFrame).Dr2 = (*Context).Dr2;
        (*TrapFrame).Dr3 = (*Context).Dr3;

        if PreviousMode != 0 {
            if (*Context).Dr0 > MmHighestUserAddress as ULONG {
                (*TrapFrame).Dr0 = 0;
            }
            if (*Context).Dr1 > MmHighestUserAddress as ULONG {
                (*TrapFrame).Dr1 = 0;
            }
            if (*Context).Dr2 > MmHighestUserAddress as ULONG {
                (*TrapFrame).Dr2 = 0;
            }
            if (*Context).Dr3 > MmHighestUserAddress as ULONG {
                (*TrapFrame).Dr3 = 0;
            }
        }

        (*TrapFrame).Dr6 = (*Context).Dr6 & DR6_LEGAL;

        if (*TrapFrame).Dr0 != 0 {
            dr_mask |= DR_MASK_0;
        }
        if (*TrapFrame).Dr1 != 0 {
            dr_mask |= DR_MASK_1;
        }
        if (*TrapFrame).Dr2 != 0 {
            dr_mask |= DR_MASK_2;
        }
        if (*TrapFrame).Dr3 != 0 {
            dr_mask |= DR_MASK_3;
        }
        if (*TrapFrame).Dr6 != 0 {
            dr_mask |= DR_MASK_6;
        }

        (*TrapFrame).Dr7 = (*Context).Dr7 & DR7_LEGAL;
        KiRecordDr7(core::ptr::addr_of_mut!((*TrapFrame).Dr7), core::ptr::addr_of_mut!(dr_mask));

        if PreviousMode != 0 {
            (*(KeGetCurrentThread() as *mut _KTHREAD)).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.__bindgen_anon_1.DebugActive = dr_mask as UCHAR;
        }
    }

    if (*(KeGetCurrentThread() as *mut _KTHREAD)).Iopl != 0 {
        (*TrapFrame).EFlags |= EFLAGS_IOPL;
    }

    if old_irql < APC_LEVEL {
        KeLowerIrql(old_irql);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeTrapFrameToContext(
    TrapFrame: PKTRAP_FRAME,
    ExceptionFrame: PKEXCEPTION_FRAME,
    Context: PCONTEXT,
) {
    #[repr(C, align(16))]
    struct AlignedFloatSaveBuffer {
        _hack: [UCHAR; 15],
        unaligned_area: FLOATING_SAVE_AREA,
    }

    let mut float_save_buffer = core::mem::zeroed::<AlignedFloatSaveBuffer>();
    let float_save_area: *mut FLOATING_SAVE_AREA;

    let old_irql = KeGetCurrentIrql();
    let mut raised_irql = old_irql;
    if old_irql < APC_LEVEL {
        KeRaiseIrql(APC_LEVEL, core::ptr::addr_of_mut!(raised_irql));
    }

    if ((*Context).ContextFlags & CONTEXT_CONTROL) == CONTEXT_CONTROL {
        (*Context).Ebp = (*TrapFrame).Ebp;
        (*Context).Eip = (*TrapFrame).Eip;
        (*Context).EFlags = (*TrapFrame).EFlags;

        if ((*TrapFrame).SegCs & FRAME_EDITED) == 0 && ((*TrapFrame).EFlags & EFLAGS_V86_MASK) == 0 {
            (*Context).SegCs = (*TrapFrame).TempSegCs & 0xFFFF;
        } else {
            (*Context).SegCs = (*TrapFrame).SegCs & 0xFFFF;
        }

        (*Context).SegSs = KiSsFromTrapFrame(TrapFrame);
        (*Context).Esp = KiEspFromTrapFrame(TrapFrame);
    }

    if ((*Context).ContextFlags & CONTEXT_SEGMENTS) == CONTEXT_SEGMENTS {
        if ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 {
            (*Context).SegGs = (*TrapFrame).V86Gs & 0xFFFF;
            (*Context).SegFs = (*TrapFrame).V86Fs & 0xFFFF;
            (*Context).SegEs = (*TrapFrame).V86Es & 0xFFFF;
            (*Context).SegDs = (*TrapFrame).V86Ds & 0xFFFF;
        } else {
            if (*TrapFrame).SegCs == KGDT_R0_CODE {
                (*TrapFrame).SegGs = 0;
                (*TrapFrame).SegFs = KGDT_R0_PCR;
                (*TrapFrame).SegEs = KGDT_R3_DATA | RPL_MASK;
                (*TrapFrame).SegDs = KGDT_R3_DATA | RPL_MASK;
            }

            (*Context).SegGs = (*TrapFrame).SegGs & 0xFFFF;
            (*Context).SegFs = (*TrapFrame).SegFs & 0xFFFF;
            (*Context).SegEs = (*TrapFrame).SegEs & 0xFFFF;
            (*Context).SegDs = (*TrapFrame).SegDs & 0xFFFF;
        }
    }

    if ((*Context).ContextFlags & CONTEXT_INTEGER) == CONTEXT_INTEGER {
        (*Context).Eax = (*TrapFrame).Eax;
        (*Context).Ebx = (*TrapFrame).Ebx;
        (*Context).Ecx = (*TrapFrame).Ecx;
        (*Context).Edx = (*TrapFrame).Edx;
        (*Context).Esi = (*TrapFrame).Esi;
        (*Context).Edi = (*TrapFrame).Edi;
    }

    if (((*Context).ContextFlags & CONTEXT_EXTENDED_REGISTERS) == CONTEXT_EXTENDED_REGISTERS) && KiUserTrap(TrapFrame) != 0 {
        let fx_save_area = (TrapFrame as usize + core::mem::size_of::<KTRAP_FRAME>()) as PFX_SAVE_AREA;

        KiFlushNPXState(core::ptr::null_mut());

        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*fx_save_area).U.FxArea) as *const u8,
            core::ptr::addr_of_mut!((*Context).ExtendedRegisters[0]) as *mut u8,
            MAXIMUM_SUPPORTED_EXTENSION,
        );
    }

    if (((*Context).ContextFlags & CONTEXT_FLOATING_POINT) == CONTEXT_FLOATING_POINT) && KiUserTrap(TrapFrame) != 0 {
        let fx_save_area = (TrapFrame as usize + core::mem::size_of::<KTRAP_FRAME>()) as PFX_SAVE_AREA;

        if KeI386FxsrPresent != 0 {
            float_save_area = (core::ptr::addr_of_mut!(float_save_buffer.unaligned_area) as ULONG_PTR & !0xF) as *mut FLOATING_SAVE_AREA;
            KiFlushNPXState(float_save_area);
        } else {
            float_save_area = core::ptr::addr_of_mut!((*fx_save_area).U.FnArea) as *mut FLOATING_SAVE_AREA;
            KiFlushNPXState(core::ptr::null_mut());
        }

        (*Context).FloatSave.ControlWord = (*float_save_area).ControlWord;
        (*Context).FloatSave.StatusWord = (*float_save_area).StatusWord;
        (*Context).FloatSave.TagWord = (*float_save_area).TagWord;
        (*Context).FloatSave.ErrorOffset = (*float_save_area).ErrorOffset;
        (*Context).FloatSave.ErrorSelector = (*float_save_area).ErrorSelector;
        (*Context).FloatSave.DataOffset = (*float_save_area).DataOffset;
        (*Context).FloatSave.DataSelector = (*float_save_area).DataSelector;
        (*Context).FloatSave.Cr0NpxState = (*fx_save_area).Cr0NpxState;

        for i in 0..SIZE_OF_80387_REGISTERS {
            (*Context).FloatSave.RegisterArea[i] = (*float_save_area).RegisterArea[i];
        }
    }

    if ((*Context).ContextFlags & CONTEXT_DEBUG_REGISTERS) == CONTEXT_DEBUG_REGISTERS {
        if ((*TrapFrame).Dr7 & !DR7_RESERVED_MASK) != 0 {
            (*Context).Dr0 = (*TrapFrame).Dr0;
            (*Context).Dr1 = (*TrapFrame).Dr1;
            (*Context).Dr2 = (*TrapFrame).Dr2;
            (*Context).Dr3 = (*TrapFrame).Dr3;
            (*Context).Dr6 = (*TrapFrame).Dr6;
            (*Context).Dr7 = KiUpdateDr7((*TrapFrame).Dr7);
        } else {
            (*Context).Dr0 = 0;
            (*Context).Dr1 = 0;
            (*Context).Dr2 = 0;
            (*Context).Dr3 = 0;
            (*Context).Dr6 = 0;
            (*Context).Dr7 = 0;
        }
    }

    if old_irql < APC_LEVEL {
        KeLowerIrql(old_irql);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeInvalidAccessAllowed(TrapInformation: PVOID) -> BOOLEAN {
    let TrapInformation = opaque_ptr(TrapInformation);
    if TrapInformation.is_null() {
        return 0u8;
    }

    let trap_frame = TrapInformation as PKTRAP_FRAME;
    let eip = match (*trap_frame).SegCs {
        KGDT_R0_CODE => ExpInterlockedPopEntrySListFault as ULONG,
        _ if (*trap_frame).SegCs == (KGDT_R3_CODE | RPL_MASK) => 0,
        _ => 0,
    };

    if (*trap_frame).Eip == eip { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn KiDispatchException(
    ExceptionRecord: PEXCEPTION_RECORD,
    ExceptionFrame: PKEXCEPTION_FRAME,
    TrapFrame: PKTRAP_FRAME,
    PreviousMode: KPROCESSOR_MODE,
    FirstChance: BOOLEAN,
) {
    let mut context = core::mem::zeroed::<CONTEXT>();
    let mut local_except_record = core::mem::zeroed::<EXCEPTION_RECORD>();

    (*KeGetCurrentPrcb()).KeExceptionDispatchCount += 1;

    context.ContextFlags = CONTEXT_FULL | CONTEXT_DEBUG_REGISTERS;

    if PreviousMode == 1i8 || !(*KeGetPcr()).KdVersionBlock.is_null() {
        context.ContextFlags |= CONTEXT_FLOATING_POINT;

        if KeI386FxsrPresent != 0 {
            context.ContextFlags |= CONTEXT_EXTENDED_REGISTERS;
        }
    }

    KeTrapFrameToContext(TrapFrame, ExceptionFrame, core::ptr::addr_of_mut!(context));

    match (*ExceptionRecord).ExceptionCode as u32 {
        0x80000003 => {
            context.Eip = context.Eip.wrapping_sub(1);
        }
        0xC0000005 => {
            (*ExceptionRecord).ExceptionCode = STATUS_ACCESS_VIOLATION;
            if PreviousMode == 1i8 {
                // FIXME: Handle no execute
            }
        }
        _ => {}
    }

    ntkrnl_sys::ASSERT!(!((PreviousMode == 0i8) && (context.EFlags & EFLAGS_V86_MASK) != 0));

    if PreviousMode == 0i8 {
        if FirstChance != 0 {
            if KiDebugRoutine(
                TrapFrame,
                ExceptionFrame,
                ExceptionRecord,
                core::ptr::addr_of_mut!(context),
                PreviousMode,
                0u8,
            ) != 0
            {
                KeContextToTrapFrame(
                    core::ptr::addr_of_mut!(context),
                    ExceptionFrame,
                    TrapFrame,
                    context.ContextFlags,
                    PreviousMode,
                );
                return;
            }

            if RtlDispatchException(ExceptionRecord, core::ptr::addr_of_mut!(context)) != 0 {
                KeContextToTrapFrame(
                    core::ptr::addr_of_mut!(context),
                    ExceptionFrame,
                    TrapFrame,
                    context.ContextFlags,
                    PreviousMode,
                );
                return;
            }
        }

        if KiDebugRoutine(
            TrapFrame,
            ExceptionFrame,
            ExceptionRecord,
            core::ptr::addr_of_mut!(context),
            PreviousMode,
            1u8,
        ) != 0
        {
            KeContextToTrapFrame(
                core::ptr::addr_of_mut!(context),
                ExceptionFrame,
                TrapFrame,
                context.ContextFlags,
                PreviousMode,
            );
            return;
        }

        // For STATUS_BREAKPOINT (INT 3 from DbgBreakPoint/RtlAssert): skip
        // the breakpoint and continue.
        if (*ExceptionRecord).ExceptionCode as u32 == 0x80000003 {
            context.Eip = context.Eip.wrapping_add(1);
            KeContextToTrapFrame(
                core::ptr::addr_of_mut!(context),
                ExceptionFrame,
                TrapFrame,
                context.ContextFlags,
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
        if (*ExceptionRecord).ExceptionCode as u32 != 0x80000003 {
        }

        if FirstChance != 0 {
            if ((*(PsGetCurrentProcess() as *mut _EPROCESS)).DebugPort.is_null() && KdIgnoreUmExceptions == 0)
                || KdIsThisAKdTrap(ExceptionRecord, core::ptr::addr_of_mut!(context), PreviousMode) != 0
            {
                if KiDebugRoutine(
                    TrapFrame,
                    ExceptionFrame,
                    ExceptionRecord,
                    core::ptr::addr_of_mut!(context),
                    PreviousMode,
                    0u8,
                ) != 0
                {
                    KeContextToTrapFrame(
                        core::ptr::addr_of_mut!(context),
                        ExceptionFrame,
                        TrapFrame,
                        context.ContextFlags,
                        PreviousMode,
                    );
                    return;
                }
            }

            if DbgkForwardException(ExceptionRecord, 1u8, 0u8) != 0 {
                return;
            }

            loop {
                // C: this block is a _SEH2_TRY whose filter copies the raised
                // exception record into LocalExceptRecord; the handler retries on
                // STATUS_STACK_OVERFLOW and otherwise falls through to the second
                // chance. No SEH here, so each raise/fault site records the code
                // the C filter would have captured and breaks with nonzero.
                macro_rules! seh_fail {
                    ($lbl:lifetime, $status:expr) => {{
                        local_except_record.ExceptionCode = $status;
                        local_except_record.ExceptionFlags = 1; // EXCEPTION_NONCONTINUABLE (ExRaiseStatus)
                        local_except_record.NumberParameters = 0;
                        break $lbl 1;
                    }};
                }
                let dispatch_result = 'dispatch: {
                    let size = (core::mem::size_of::<CONTEXT>() + 3) & !3;
                    let stack = (context.Esp & !3).wrapping_sub(size as u32);

                    if ((*TrapFrame).HardwareSegSs != (KGDT_R3_DATA | RPL_MASK))
                        || ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0
                    {
                        // C: RtlRaiseException(&LocalExceptRecord) caught by this
                        // same handler — the filter copies that record back.
                        local_except_record.ExceptionCode = STATUS_ACCESS_VIOLATION;
                        local_except_record.ExceptionFlags = 0;
                        local_except_record.NumberParameters = 0;
                        break 'dispatch 1;
                    }

                    // C ProbeForWrite checks unconditionally: UserMode (1).
                    if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
                        stack as PVOID,
                        size,
                        core::mem::size_of::<ULONG>(),
                        1,
                    ) {
                        seh_fail!('dispatch, status);
                    }
                    if let Err(status) = ntkrnl_sys::probe::safe_copy_to_user(
                        stack as PVOID,
                        core::ptr::addr_of!(context) as *const core::ffi::c_void,
                        core::mem::size_of::<CONTEXT>(),
                        1,
                        1,
                    ) {
                        seh_fail!('dispatch, status);
                    }

                    let except_size = (core::mem::size_of::<EXCEPTION_RECORD>()
                        - (15 - (*ExceptionRecord).NumberParameters as usize) * core::mem::size_of::<ULONG>()
                        + 3)
                        & !3;
                    let new_stack = stack.wrapping_sub(except_size as u32);

                    if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
                        new_stack.wrapping_sub(2 * core::mem::size_of::<ULONG_PTR>() as u32) as PVOID,
                        except_size + 2 * core::mem::size_of::<ULONG_PTR>(),
                        core::mem::size_of::<ULONG>(),
                        1,
                    ) {
                        seh_fail!('dispatch, status);
                    }
                    if let Err(status) = ntkrnl_sys::probe::safe_copy_to_user(
                        new_stack as PVOID,
                        ExceptionRecord as *const core::ffi::c_void,
                        except_size,
                        1,
                        1,
                    ) {
                        seh_fail!('dispatch, status);
                    }

                    if let Err(status) = ntkrnl_sys::probe::write_user(
                        new_stack.wrapping_sub(1 * core::mem::size_of::<ULONG_PTR>() as u32) as *mut ULONG_PTR,
                        stack as ULONG_PTR,
                        1,
                    ) {
                        seh_fail!('dispatch, status);
                    }
                    if let Err(status) = ntkrnl_sys::probe::write_user(
                        new_stack.wrapping_sub(2 * core::mem::size_of::<ULONG_PTR>() as u32) as *mut ULONG_PTR,
                        new_stack as ULONG_PTR,
                        1,
                    ) {
                        seh_fail!('dispatch, status);
                    }

                    KiSsToTrapFrame(TrapFrame, KGDT_R3_DATA);
                    KiEspToTrapFrame(TrapFrame, new_stack.wrapping_sub(2 * core::mem::size_of::<ULONG_PTR>() as u32));

                    (*TrapFrame).SegCs = Ke386SanitizeSeg(KGDT_R3_CODE, PreviousMode);
                    (*TrapFrame).SegDs = Ke386SanitizeSeg(KGDT_R3_DATA, PreviousMode);
                    (*TrapFrame).SegEs = Ke386SanitizeSeg(KGDT_R3_DATA, PreviousMode);
                    (*TrapFrame).SegFs = Ke386SanitizeSeg(KGDT_R3_TEB, PreviousMode);
                    (*TrapFrame).SegGs = 0;

                    (*TrapFrame).Eip = KeUserExceptionDispatcher as ULONG;
                    break 'dispatch 0;
                };

                if dispatch_result == 0 {
                    return;
                }

                if (local_except_record.ExceptionCode as u32) == 0xC00000FD {
                    local_except_record.ExceptionAddress = (*ExceptionRecord).ExceptionAddress;
                    core::ptr::copy_nonoverlapping(
                        core::ptr::addr_of!(local_except_record) as *const u8,
                        ExceptionRecord as *mut u8,
                        core::mem::size_of::<EXCEPTION_RECORD>(),
                    );
                    continue;
                }
                break;
            }
        }

        // Second chance is a process death, and it must be named HERE: csrss's
        // exception port usually "handles" it by TerminateProcess(client, 0x80),
        // the forward below returns nonzero, and the "Kill" print further down is
        // never reached. amd64 has had this tripwire for months; i386 did not, so
        // the native control of the WoW64 winetest A/B saw its user32:scroll
        // child die as an anonymous exit 0x80 -- which winetest then reported as
        // "128 failures in child process".
        {
            extern "system" {
                fn PsGetCurrentProcess() -> *mut core::ffi::c_void;
                fn PsGetProcessImageFileName(p: *mut core::ffi::c_void) -> *const u8;
            }
            crate::dbg_print!(
                b"[KI-EXC] %.16s second chance: code=0x%lx addr=%p eip=%p P0=%lx P1=%lx\n\0",
                PsGetProcessImageFileName(PsGetCurrentProcess()),
                (*ExceptionRecord).ExceptionCode as u32,
                (*ExceptionRecord).ExceptionAddress,
                (*TrapFrame).Eip as *const u8,
                (*ExceptionRecord).ExceptionInformation[0] as u32,
                (*ExceptionRecord).ExceptionInformation[1] as u32,
            );
        }
        // Bounded EBP-chain walk of the faulting user stack (we are in that process's
        // context), so the report names callers, not just the PC. It lived on the "Kill"
        // path below, which csrss's TerminateProcess(client, 0x80) means is almost never
        // reached -- ucrtbase_apitest's 0xC0000008 from RtlpUnWaitCriticalSection showed
        // up with no caller at all.
        {
            let mut ebp = (*TrapFrame).Ebp as usize;
            let mut n = 0;
            while n < 12
                && ebp != 0
                && ebp < 0x7FFF_0000
                && crate::ARM3::mmsup::MmIsAddressValid(ebp as _) != 0
                && crate::ARM3::mmsup::MmIsAddressValid((ebp + 4) as _) != 0
            {
                let ret = *((ebp + 4) as *const usize);
                crate::dbg_print!(b"[KI-EXC]   caller ret=%p\n\0", ret as *const u8);
                let next = *(ebp as *const usize);
                if next <= ebp {
                    break;
                }
                ebp = next;
                n += 1;
            }
        }
        if DbgkForwardException(ExceptionRecord, 1u8, 1u8) != 0 {
            return;
        } else if DbgkForwardException(ExceptionRecord, 0u8, 1u8) != 0 {
            return;
        }

        // 3rd strike, kill the process. This is the C's DPRINT1, which the
        // translation dropped. Without it an unhandled user exception in csrss
        // surfaced only as an anonymous BUGCHECK 0xF4 two frames later.
        {
            extern "system" {
                fn PsGetCurrentProcess() -> *mut core::ffi::c_void;
                fn PsGetProcessImageFileName(p: *mut core::ffi::c_void) -> *const u8;
                fn PsGetProcessSectionBaseAddress(p: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
            }
            crate::dbg_print!(
                b"Kill %.16s, ExceptionCode: %lx, ExceptionAddress: %p, BaseAddress: %p, P0: %lx, P1: %lx\n\0",
                PsGetProcessImageFileName(PsGetCurrentProcess()),
                (*ExceptionRecord).ExceptionCode as u32,
                (*ExceptionRecord).ExceptionAddress,
                PsGetProcessSectionBaseAddress(PsGetCurrentProcess()),
                (*ExceptionRecord).ExceptionInformation[0] as u32,
                (*ExceptionRecord).ExceptionInformation[1] as u32,
            );
            // The caller walk is printed at second chance above, which always precedes this.
        }
        ZwTerminateProcess(NtCurrentProcess(), (*ExceptionRecord).ExceptionCode as NTSTATUS);
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
pub unsafe extern "system" fn KiDispatchExceptionFromTrapFrame(
    Code: NTSTATUS,
    Flags: ULONG,
    Address: ULONG_PTR,
    ParameterCount: ULONG,
    Parameter1: ULONG_PTR,
    Parameter2: ULONG_PTR,
    Parameter3: ULONG_PTR,
    TrapFrame: PKTRAP_FRAME,
) -> ! {
    let mut exception_record = core::mem::zeroed::<EXCEPTION_RECORD>();

    exception_record.ExceptionCode = Code;
    exception_record.ExceptionFlags = Flags;
    exception_record.ExceptionRecord = core::ptr::null_mut();
    exception_record.ExceptionAddress = Address as PVOID;
    exception_record.NumberParameters = ParameterCount;
    if ParameterCount != 0 {
        exception_record.ExceptionInformation[0] = Parameter1 as u32;
        exception_record.ExceptionInformation[1] = Parameter2 as u32;
        exception_record.ExceptionInformation[2] = Parameter3 as u32;
    }

    KiDispatchException(
        core::ptr::addr_of_mut!(exception_record),
        core::ptr::null_mut(),
        TrapFrame,
        if ((*TrapFrame).EFlags & EFLAGS_V86_MASK) != 0 {
            -1i8
        } else {
            KiUserTrap(TrapFrame)
        },
        1u8,
    );

    KiEoiHelper(TrapFrame);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiSystemFatalException(ExceptionCode: ULONG, TrapFrame: PKTRAP_FRAME) -> ! {
    KeBugCheckWithTf(UNEXPECTED_KERNEL_MODE_TRAP, ExceptionCode as ULONG_PTR, 0, 0, 0, TrapFrame);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiSystemFatalException(ExceptionCode: ULONG, TrapFrame: PKTRAP_FRAME) -> ! {
    KeBugCheckWithTf(UNEXPECTED_KERNEL_MODE_TRAP, ExceptionCode as u32, 0, 0, 0, TrapFrame);
}

#[no_mangle]
pub unsafe extern "system" fn KeRaiseUserException(ExceptionCode: NTSTATUS) -> NTSTATUS {
    let teb = (*(KeGetCurrentThread() as *mut _ETHREAD)).Tcb.Teb as PTEB;
    let trap_frame = (*(KeGetCurrentThread() as *mut _ETHREAD)).Tcb.TrapFrame as PKTRAP_FRAME;

    let set_result = 'set_exception: {
        if !teb.is_null() {
            (*teb).ExceptionCode = ExceptionCode;
            break 'set_exception 0;
        }
        break 'set_exception 1;
    };

    if set_result != 0 {
        return STATUS_ACCESS_VIOLATION;
    }

    let old_eip = (*trap_frame).Eip;

    (*trap_frame).Eip = KeRaiseUserExceptionDispatcher as ULONG;

    old_eip as NTSTATUS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/i386/exp.c
//  * PURPOSE:         Exception Dispatching and Context<->Trap Frame Conversion
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Gregor Anich
//  *                  Skywing (skywing@valhallalegends.com)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KeInitExceptions(VOID)
// {
//     ULONG i;
//     USHORT FlippedSelector;
//
//     /* Loop the IDT */
//     for (i = 0; i <= MAXIMUM_IDTVECTOR; i++)
//     {
//         /* Save the current Selector */
//         FlippedSelector = KiIdt[i].Selector;
//
//         /* Flip Selector and Extended Offset */
//         KiIdt[i].Selector = KiIdt[i].ExtendedOffset;
//         KiIdt[i].ExtendedOffset = FlippedSelector;
//     }
// }
//
// ULONG
// FASTCALL
// KiUpdateDr7(IN ULONG Dr7)
// {
//     ULONG DebugMask = KeGetCurrentThread()->Header.DebugActive;
//
//     /* Check if debugging is enabled */
//     if (DebugMask & DR_MASK(DR7_OVERRIDE_V))
//     {
//         /* Sanity checks */
//         ASSERT((DebugMask & DR_REG_MASK) != 0);
//         ASSERT((Dr7 & ~DR7_RESERVED_MASK) == DR7_OVERRIDE_MASK);
//         return 0;
//     }
//
//     /* Return DR7 itself */
//     return Dr7;
// }
//
// BOOLEAN
// FASTCALL
// KiRecordDr7(OUT PULONG Dr7Ptr,
//             OUT PULONG DrMask)
// {
//     ULONG NewMask, Mask;
//     UCHAR Result;
//
//     /* Check if the caller gave us a mask */
//     if (!DrMask)
//     {
//         /* He didn't, use the one from the thread */
//         Mask = KeGetCurrentThread()->Header.DebugActive;
//     }
//     else
//     {
//         /* He did, read it */
//         Mask = *DrMask;
//     }
//
//     /* Sanity check */
//     ASSERT((*Dr7Ptr & DR7_RESERVED_MASK) == 0);
//
//     /* Check if DR7 is empty */
//     NewMask = Mask;
//     if (!(*Dr7Ptr))
//     {
//         /* Assume failure */
//         Result = FALSE;
//
//         /* Check the DR mask */
//         NewMask &= ~(DR_MASK(7));
//         if (NewMask & DR_REG_MASK)
//         {
//             /* Set the active mask */
//             NewMask |= DR_MASK(DR7_OVERRIDE_V);
//
//             /* Set DR7 override */
//             *Dr7Ptr |= DR7_OVERRIDE_MASK;
//         }
//         else
//         {
//             /* Sanity check */
//             ASSERT(NewMask == 0);
//         }
//     }
//     else
//     {
//         /* Check if we have a mask or not */
//         Result = NewMask ? TRUE: FALSE;
//
//         /* Update the mask to disable debugging */
//         NewMask &= ~(DR_MASK(DR7_OVERRIDE_V));
//         NewMask |= DR_MASK(7);
//     }
//
//     /* Check if caller wants the new mask */
//     if (DrMask)
//     {
//         /* Update it */
//         *DrMask = NewMask;
//     }
//     else
//     {
//         /* Check if the mask changed */
//         if (Mask != NewMask)
//         {
//             /* Update it */
//             KeGetCurrentThread()->Header.DebugActive = (UCHAR)NewMask;
//         }
//     }
//
//     /* Return the result */
//     return Result;
// }
//
// ULONG
// NTAPI
// KiEspFromTrapFrame(IN PKTRAP_FRAME TrapFrame)
// {
//     /* Check if this is user-mode or V86 */
//     if (KiUserTrap(TrapFrame) ||
//         (TrapFrame->EFlags & EFLAGS_V86_MASK))
//     {
//         /* Return it directly */
//         return TrapFrame->HardwareEsp;
//     }
//     else
//     {
//         /* Edited frame */
//         if (!(TrapFrame->SegCs & FRAME_EDITED))
//         {
//             /* Return edited value */
//             return TrapFrame->TempEsp;
//         }
//         else
//         {
//             /* Virgin frame, calculate */
//             return (ULONG)&TrapFrame->HardwareEsp;
//         }
//     }
// }
//
// VOID
// NTAPI
// KiEspToTrapFrame(IN PKTRAP_FRAME TrapFrame,
//                  IN ULONG Esp)
// {
//     KIRQL OldIrql;
//     ULONG Previous;
//
//     /* Raise to APC_LEVEL if needed */
//     OldIrql = KeGetCurrentIrql();
//     if (OldIrql < APC_LEVEL) KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Get the old ESP */
//     Previous = KiEspFromTrapFrame(TrapFrame);
//
//     /* Check if this is user-mode or V86 */
//     if (KiUserTrap(TrapFrame) ||
//         (TrapFrame->EFlags & EFLAGS_V86_MASK))
//     {
//         /* Write it directly */
//         TrapFrame->HardwareEsp = Esp;
//     }
//     else
//     {
//         /* Don't allow ESP to be lowered, this is illegal */
//         if (Esp < Previous) KeBugCheckEx(SET_OF_INVALID_CONTEXT,
//                                          Esp,
//                                          Previous,
//                                          (ULONG_PTR)TrapFrame,
//                                          0);
//
//         /* Create an edit frame, check if it was alrady */
//         if (!(TrapFrame->SegCs & FRAME_EDITED))
//         {
//             /* Update the value */
//             TrapFrame->TempEsp = Esp;
//         }
//         else
//         {
//             /* Check if ESP changed */
//             if (Previous != Esp)
//             {
//                 /* Save CS */
//                 TrapFrame->TempSegCs = TrapFrame->SegCs;
//                 TrapFrame->SegCs &= ~FRAME_EDITED;
//
//                 /* Save ESP */
//                 TrapFrame->TempEsp = Esp;
//             }
//         }
//     }
//
//     /* Restore IRQL */
//     if (OldIrql < APC_LEVEL) KeLowerIrql(OldIrql);
// }
//
// ULONG
// NTAPI
// KiSsFromTrapFrame(IN PKTRAP_FRAME TrapFrame)
// {
//     /* Check if this was V86 Mode */
//     if (TrapFrame->EFlags & EFLAGS_V86_MASK)
//     {
//         /* Just return it */
//         return TrapFrame->HardwareSegSs;
//     }
//     else if (KiUserTrap(TrapFrame))
//     {
//         /* User mode, return the User SS */
//         return TrapFrame->HardwareSegSs | RPL_MASK;
//     }
//     else
//     {
//         /* Kernel mode */
//         return KGDT_R0_DATA;
//     }
// }
//
// VOID
// NTAPI
// KiSsToTrapFrame(IN PKTRAP_FRAME TrapFrame,
//                 IN ULONG Ss)
// {
//     /* Remove the high-bits */
//     Ss &= 0xFFFF;
//
//     /* If this was V86 Mode */
//     if (TrapFrame->EFlags & EFLAGS_V86_MASK)
//     {
//         /* Just write it */
//         TrapFrame->HardwareSegSs = Ss;
//     }
//     else if (KiUserTrap(TrapFrame))
//     {
//         /* Usermode, save the User SS */
//         TrapFrame->HardwareSegSs = Ss | RPL_MASK;
//     }
// }
//
// USHORT
// NTAPI
// KiTagWordFnsaveToFxsave(USHORT TagWord)
// {
//     INT FxTagWord = ~TagWord;
//
//     /*
//      * Empty is now 00, any 2 bits containing 1 mean valid
//      * Now convert the rest (11->0 and the rest to 1)
//      */
//     FxTagWord = (FxTagWord | (FxTagWord >> 1)) & 0x5555; /* 0V0V0V0V0V0V0V0V */
//     FxTagWord = (FxTagWord | (FxTagWord >> 1)) & 0x3333; /* 00VV00VV00VV00VV */
//     FxTagWord = (FxTagWord | (FxTagWord >> 2)) & 0x0f0f; /* 0000VVVV0000VVVV */
//     FxTagWord = (FxTagWord | (FxTagWord >> 4)) & 0x00ff; /* 00000000VVVVVVVV */
//     return FxTagWord;
// }
//
// VOID
// NTAPI
// Ki386AdjustEsp0(IN PKTRAP_FRAME TrapFrame)
// {
//     PKTHREAD Thread;
//     ULONG_PTR Stack;
//     ULONG EFlags;
//
//     /* Get the current thread's stack */
//     Thread = KeGetCurrentThread();
//     Stack = (ULONG_PTR)Thread->InitialStack;
//
//     /* Check if we are in V8086 mode */
//     if (!(TrapFrame->EFlags & EFLAGS_V86_MASK))
//     {
//         /* Bias the stack for the V86 segments */
//         Stack -= sizeof(KTRAP_FRAME) -
//                  FIELD_OFFSET(KTRAP_FRAME, V86Es);
//     }
//
//     /* Bias the stack for the FPU area */
//     Stack -= sizeof(FX_SAVE_AREA);
//
//     /* Disable interrupts */
//     EFlags = __readeflags();
//     _disable();
//
//     /* Set new ESP0 value in the TSS */
//     KeGetPcr()->TSS->Esp0 = Stack;
//
//     /* Restore old interrupt state */
//     __writeeflags(EFlags);
// }
//
// VOID
// NTAPI
// KeContextToTrapFrame(IN PCONTEXT Context,
//                      IN OUT PKEXCEPTION_FRAME ExceptionFrame,
//                      IN OUT PKTRAP_FRAME TrapFrame,
//                      IN ULONG ContextFlags,
//                      IN KPROCESSOR_MODE PreviousMode)
// {
//     PFX_SAVE_AREA FxSaveArea;
//     ULONG i;
//     BOOLEAN V86Switch = FALSE;
//     KIRQL OldIrql;
//     ULONG DrMask = 0;
//
//     /* Do this at APC_LEVEL */
//     OldIrql = KeGetCurrentIrql();
//     if (OldIrql < APC_LEVEL) KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Start with the basic Registers */
//     if ((ContextFlags & CONTEXT_CONTROL) == CONTEXT_CONTROL)
//     {
//         /* Check if we went through a V86 switch */
//         if ((Context->EFlags & EFLAGS_V86_MASK) !=
//             (TrapFrame->EFlags & EFLAGS_V86_MASK))
//         {
//             /* We did, remember this for later */
//             V86Switch = TRUE;
//         }
//
//         /* Copy EFLAGS and sanitize them*/
//         TrapFrame->EFlags = Ke386SanitizeFlags(Context->EFlags, PreviousMode);
//
//         /* Copy EBP and EIP */
//         TrapFrame->Ebp = Context->Ebp;
//         TrapFrame->Eip = Context->Eip;
//
//         /* Check if we were in V86 Mode */
//         if (TrapFrame->EFlags & EFLAGS_V86_MASK)
//         {
//             /* Simply copy the CS value */
//             TrapFrame->SegCs = Context->SegCs;
//         }
//         else
//         {
//             /* We weren't in V86, so sanitize the CS */
//             TrapFrame->SegCs = Ke386SanitizeSeg(Context->SegCs, PreviousMode);
//
//             /* Don't let it under 8, that's invalid */
//             if ((PreviousMode != KernelMode) && (TrapFrame->SegCs < 8))
//             {
//                 /* Force it to User CS */
//                 TrapFrame->SegCs = KGDT_R3_CODE | RPL_MASK;
//             }
//         }
//
//         /* Handle SS Specially for validation */
//         KiSsToTrapFrame(TrapFrame, Context->SegSs);
//
//         /* Write ESP back; take into account Edited Trap Frames */
//         KiEspToTrapFrame(TrapFrame, Context->Esp);
//
//         /* Handle our V86 Bias if we went through a switch */
//         if (V86Switch) Ki386AdjustEsp0(TrapFrame);
//     }
//
//     /* Process the Integer Registers */
//     if ((ContextFlags & CONTEXT_INTEGER) == CONTEXT_INTEGER)
//     {
//         /* Copy them manually */
//         TrapFrame->Eax = Context->Eax;
//         TrapFrame->Ebx = Context->Ebx;
//         TrapFrame->Ecx = Context->Ecx;
//         TrapFrame->Edx = Context->Edx;
//         TrapFrame->Esi = Context->Esi;
//         TrapFrame->Edi = Context->Edi;
//     }
//
//     /* Process the Context Segments */
//     if ((ContextFlags & CONTEXT_SEGMENTS) == CONTEXT_SEGMENTS)
//     {
//         /* Check if we were in V86 Mode */
//         if (TrapFrame->EFlags & EFLAGS_V86_MASK)
//         {
//             /* Copy the V86 Segments directly */
//             TrapFrame->V86Ds = Context->SegDs;
//             TrapFrame->V86Es = Context->SegEs;
//             TrapFrame->V86Fs = Context->SegFs;
//             TrapFrame->V86Gs = Context->SegGs;
//         }
//         else if (!KiUserTrap(TrapFrame))
//         {
//             /* For kernel mode, write the standard values */
//             TrapFrame->SegDs = KGDT_R3_DATA | RPL_MASK;
//             TrapFrame->SegEs = KGDT_R3_DATA | RPL_MASK;
//             TrapFrame->SegFs = Ke386SanitizeSeg(Context->SegFs, PreviousMode);
//             TrapFrame->SegGs = 0;
//         }
//         else
//         {
//             /* For user mode, return the values directly */
//             TrapFrame->SegDs = Context->SegDs;
//             TrapFrame->SegEs = Context->SegEs;
//             TrapFrame->SegFs = Context->SegFs;
//
//             /* Handle GS specially */
//             if (TrapFrame->SegCs == (KGDT_R3_CODE | RPL_MASK))
//             {
//                 /* Don't use it, if user */
//                 TrapFrame->SegGs = 0;
//             }
//             else
//             {
//                 /* Copy it if kernel */
//                 TrapFrame->SegGs = Context->SegGs;
//             }
//         }
//     }
//
//     /* Handle the extended registers */
//     if (((ContextFlags & CONTEXT_EXTENDED_REGISTERS) ==
//         CONTEXT_EXTENDED_REGISTERS) && KiUserTrap(TrapFrame))
//     {
//         /* Get the FX Area */
//         FxSaveArea = (PFX_SAVE_AREA)(TrapFrame + 1);
//
//         /* Flush the NPX State */
//         KiFlushNPXState(NULL);
//
//         /* Copy the FX State */
//         RtlCopyMemory(&FxSaveArea->U.FxArea,
//                       &Context->ExtendedRegisters[0],
//                       MAXIMUM_SUPPORTED_EXTENSION);
//
//         /* Remove reserved bits from MXCSR */
//         FxSaveArea->U.FxArea.MXCsr &= KiMXCsrMask;
//
//         /* Mask out any invalid flags */
//         FxSaveArea->Cr0NpxState &= ~(CR0_EM | CR0_MP | CR0_TS);
//
//         /* Check if this is a VDM app */
//         if (PsGetCurrentProcess()->VdmObjects)
//         {
//             /* Allow the EM flag */
//             FxSaveArea->Cr0NpxState |= Context->FloatSave.Cr0NpxState & (CR0_EM | CR0_MP);
//         }
//     }
//
//     /* Handle the floating point state */
//     if (((ContextFlags & CONTEXT_FLOATING_POINT) ==
//         CONTEXT_FLOATING_POINT) && KiUserTrap(TrapFrame))
//     {
//         /* Get the FX Area */
//         FxSaveArea = (PFX_SAVE_AREA)(TrapFrame + 1);
//
//         /* Flush the NPX State */
//         KiFlushNPXState(NULL);
//
//         /* Check if we have Fxsr support */
//         if (KeI386FxsrPresent)
//         {
//             /* Convert the Fn Floating Point state to Fx */
//             FxSaveArea->U.FxArea.ControlWord = (USHORT)Context->FloatSave.ControlWord;
//             FxSaveArea->U.FxArea.StatusWord = (USHORT)Context->FloatSave.StatusWord;
//             FxSaveArea->U.FxArea.TagWord =
//                 KiTagWordFnsaveToFxsave((USHORT)Context->FloatSave.TagWord);
//             FxSaveArea->U.FxArea.ErrorOpcode =
//                 (USHORT)((Context->FloatSave.ErrorSelector >> 16) & 0xFFFF);
//             FxSaveArea->U.FxArea.ErrorOffset = Context->FloatSave.ErrorOffset;
//             FxSaveArea->U.FxArea.ErrorSelector = Context->FloatSave.ErrorSelector & 0xFFFF;
//             FxSaveArea->U.FxArea.DataOffset = Context->FloatSave.DataOffset;
//             FxSaveArea->U.FxArea.DataSelector = Context->FloatSave.DataSelector;
//
//             /* Clear out the Register Area */
//             RtlZeroMemory(&FxSaveArea->U.FxArea.RegisterArea[0], SIZE_OF_FX_REGISTERS);
//
//             /* Loop the 8 floating point registers */
//             for (i = 0; i < 8; i++)
//             {
//                 /* Copy from Fn to Fx */
//                 RtlCopyMemory(FxSaveArea->U.FxArea.RegisterArea + (i * 16),
//                               Context->FloatSave.RegisterArea + (i * 10),
//                               10);
//             }
//         }
//         else
//         {
//             /* Copy the structure */
//             FxSaveArea->U.FnArea.ControlWord = Context->FloatSave.ControlWord;
//             FxSaveArea->U.FnArea.StatusWord = Context->FloatSave.StatusWord;
//             FxSaveArea->U.FnArea.TagWord = Context->FloatSave.TagWord;
//             FxSaveArea->U.FnArea.ErrorOffset = Context->FloatSave.ErrorOffset;
//             FxSaveArea->U.FnArea.ErrorSelector = Context->FloatSave.ErrorSelector;
//             FxSaveArea->U.FnArea.DataOffset = Context->FloatSave.DataOffset;
//             FxSaveArea->U.FnArea.DataSelector = Context->FloatSave.DataSelector;
//
//             /* Loop registers */
//             for (i = 0; i < SIZE_OF_80387_REGISTERS; i++)
//             {
//                 /* Copy registers */
//                 FxSaveArea->U.FnArea.RegisterArea[i] = Context->FloatSave.RegisterArea[i];
//             }
//         }
//
//         /* Mask out any invalid flags */
//         FxSaveArea->Cr0NpxState &= ~(CR0_EM | CR0_MP | CR0_TS);
//
//         /* Check if this is a VDM app */
//         if (PsGetCurrentProcess()->VdmObjects)
//         {
//             /* Allow the EM flag */
//             FxSaveArea->Cr0NpxState |= Context->FloatSave.Cr0NpxState & (CR0_EM | CR0_MP);
//         }
//     }
//
//     /* Handle the Debug Registers */
//     if ((ContextFlags & CONTEXT_DEBUG_REGISTERS) == CONTEXT_DEBUG_REGISTERS)
//     {
//         /* Copy Dr0 - Dr4 */
//         TrapFrame->Dr0 = Context->Dr0;
//         TrapFrame->Dr1 = Context->Dr1;
//         TrapFrame->Dr2 = Context->Dr2;
//         TrapFrame->Dr3 = Context->Dr3;
//
//         /* If we're in user-mode */
//         if (PreviousMode != KernelMode)
//         {
//             /* Make sure, no Dr address is above user space */
//             if (Context->Dr0 > (ULONG)MmHighestUserAddress) TrapFrame->Dr0 = 0;
//             if (Context->Dr1 > (ULONG)MmHighestUserAddress) TrapFrame->Dr1 = 0;
//             if (Context->Dr2 > (ULONG)MmHighestUserAddress) TrapFrame->Dr2 = 0;
//             if (Context->Dr3 > (ULONG)MmHighestUserAddress) TrapFrame->Dr3 = 0;
//         }
//
//         /* Now sanitize and save DR6 */
//         TrapFrame->Dr6 = Context->Dr6 & DR6_LEGAL;
//
//         /* Update the Dr active mask */
//         if (TrapFrame->Dr0) DrMask |= DR_MASK(0);
//         if (TrapFrame->Dr1) DrMask |= DR_MASK(1);
//         if (TrapFrame->Dr2) DrMask |= DR_MASK(2);
//         if (TrapFrame->Dr3) DrMask |= DR_MASK(3);
//         if (TrapFrame->Dr6) DrMask |= DR_MASK(6);
//
//         /* Sanitize and save DR7 */
//         TrapFrame->Dr7 = Context->Dr7 & DR7_LEGAL;
//         KiRecordDr7(&TrapFrame->Dr7, &DrMask);
//
//         /* If we're in user-mode */
//         if (PreviousMode != KernelMode)
//         {
//             /* Save the mask */
//             KeGetCurrentThread()->Header.DebugActive = (UCHAR)DrMask;
//         }
//     }
//
//     /* Check if thread has IOPL and force it enabled if so */
//     if (KeGetCurrentThread()->Iopl) TrapFrame->EFlags |= EFLAGS_IOPL;
//
//     /* Restore IRQL */
//     if (OldIrql < APC_LEVEL) KeLowerIrql(OldIrql);
// }
//
// VOID
// NTAPI
// KeTrapFrameToContext(IN PKTRAP_FRAME TrapFrame,
//                      IN PKEXCEPTION_FRAME ExceptionFrame,
//                      IN OUT PCONTEXT Context)
// {
//     PFX_SAVE_AREA FxSaveArea;
//     struct _AlignHack
//     {
//         UCHAR Hack[15];
//         FLOATING_SAVE_AREA UnalignedArea;
//     } FloatSaveBuffer;
//     FLOATING_SAVE_AREA *FloatSaveArea;
//     KIRQL OldIrql;
//     ULONG i;
//
//     /* Do this at APC_LEVEL */
//     OldIrql = KeGetCurrentIrql();
//     if (OldIrql < APC_LEVEL) KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//     /* Start with the Control flags */
//     if ((Context->ContextFlags & CONTEXT_CONTROL) == CONTEXT_CONTROL)
//     {
//         /* EBP, EIP and EFLAGS */
//         Context->Ebp = TrapFrame->Ebp;
//         Context->Eip = TrapFrame->Eip;
//         Context->EFlags = TrapFrame->EFlags;
//
//         /* Return the correct CS */
//         if (!(TrapFrame->SegCs & FRAME_EDITED) &&
//             !(TrapFrame->EFlags & EFLAGS_V86_MASK))
//         {
//             /* Get it from the Temp location */
//             Context->SegCs = TrapFrame->TempSegCs & 0xFFFF;
//         }
//         else
//         {
//             /* Return it directly */
//             Context->SegCs = TrapFrame->SegCs & 0xFFFF;
//         }
//
//         /* Get the Ss and ESP */
//         Context->SegSs = KiSsFromTrapFrame(TrapFrame);
//         Context->Esp = KiEspFromTrapFrame(TrapFrame);
//     }
//
//     /* Handle the Segments */
//     if ((Context->ContextFlags & CONTEXT_SEGMENTS) == CONTEXT_SEGMENTS)
//     {
//         /* Do V86 Mode first */
//         if (TrapFrame->EFlags & EFLAGS_V86_MASK)
//         {
//             /* Return from the V86 location */
//             Context->SegGs = TrapFrame->V86Gs & 0xFFFF;
//             Context->SegFs = TrapFrame->V86Fs & 0xFFFF;
//             Context->SegEs = TrapFrame->V86Es & 0xFFFF;
//             Context->SegDs = TrapFrame->V86Ds & 0xFFFF;
//         }
//         else
//         {
//             /* Check if this was a Kernel Trap */
//             if (TrapFrame->SegCs == KGDT_R0_CODE)
//             {
//                 /* Set valid selectors */
//                 TrapFrame->SegGs = 0;
//                 TrapFrame->SegFs = KGDT_R0_PCR;
//                 TrapFrame->SegEs = KGDT_R3_DATA | RPL_MASK;
//                 TrapFrame->SegDs = KGDT_R3_DATA | RPL_MASK;
//             }
//
//             /* Return the segments */
//             Context->SegGs = TrapFrame->SegGs & 0xFFFF;
//             Context->SegFs = TrapFrame->SegFs & 0xFFFF;
//             Context->SegEs = TrapFrame->SegEs & 0xFFFF;
//             Context->SegDs = TrapFrame->SegDs & 0xFFFF;
//         }
//     }
//
//     /* Handle the simple registers */
//     if ((Context->ContextFlags & CONTEXT_INTEGER) == CONTEXT_INTEGER)
//     {
//         /* Return them directly */
//         Context->Eax = TrapFrame->Eax;
//         Context->Ebx = TrapFrame->Ebx;
//         Context->Ecx = TrapFrame->Ecx;
//         Context->Edx = TrapFrame->Edx;
//         Context->Esi = TrapFrame->Esi;
//         Context->Edi = TrapFrame->Edi;
//     }
//
//     /* Handle extended registers */
//     if (((Context->ContextFlags & CONTEXT_EXTENDED_REGISTERS) ==
//         CONTEXT_EXTENDED_REGISTERS) && KiUserTrap(TrapFrame))
//     {
//         /* Get the FX Save Area */
//         FxSaveArea = (PFX_SAVE_AREA)(TrapFrame + 1);
//
//         /* Flush the NPX State */
//         KiFlushNPXState(NULL);
//
//         /* Copy the registers */
//         RtlCopyMemory(&Context->ExtendedRegisters[0],
//                       &FxSaveArea->U.FxArea,
//                       MAXIMUM_SUPPORTED_EXTENSION);
//     }
//
//     /* Handle Floating Point */
//     if (((Context->ContextFlags & CONTEXT_FLOATING_POINT) ==
//         CONTEXT_FLOATING_POINT) && KiUserTrap(TrapFrame))
//     {
//         /* Get the FX Save Area */
//         FxSaveArea = (PFX_SAVE_AREA)(TrapFrame + 1);
//
//         /* Check if we have Fxsr support */
//         if (KeI386FxsrPresent)
//         {
//             /* Align the floating area to 16-bytes */
//             FloatSaveArea = (FLOATING_SAVE_AREA*)((ULONG_PTR)&FloatSaveBuffer.UnalignedArea &~ 0xF);
//
//             /* Get the State */
//             KiFlushNPXState(FloatSaveArea);
//         }
//         else
//         {
//             /* We don't, use the FN area and flush the NPX State */
//             FloatSaveArea = (FLOATING_SAVE_AREA*)&FxSaveArea->U.FnArea;
//             KiFlushNPXState(NULL);
//         }
//
//         /* Copy structure */
//         Context->FloatSave.ControlWord = FloatSaveArea->ControlWord;
//         Context->FloatSave.StatusWord = FloatSaveArea->StatusWord;
//         Context->FloatSave.TagWord = FloatSaveArea->TagWord;
//         Context->FloatSave.ErrorOffset = FloatSaveArea->ErrorOffset;
//         Context->FloatSave.ErrorSelector = FloatSaveArea->ErrorSelector;
//         Context->FloatSave.DataOffset = FloatSaveArea->DataOffset;
//         Context->FloatSave.DataSelector = FloatSaveArea->DataSelector;
//         Context->FloatSave.Cr0NpxState = FxSaveArea->Cr0NpxState;
//
//         /* Loop registers */
//         for (i = 0; i < SIZE_OF_80387_REGISTERS; i++)
//         {
//             /* Copy them */
//             Context->FloatSave.RegisterArea[i] = FloatSaveArea->RegisterArea[i];
//         }
//     }
//
//     /* Handle debug registers */
//     if ((Context->ContextFlags & CONTEXT_DEBUG_REGISTERS) ==
//         CONTEXT_DEBUG_REGISTERS)
//     {
//         /* Make sure DR7 is valid */
//         if (TrapFrame->Dr7 & ~DR7_RESERVED_MASK)
//         {
//             /* Copy the debug registers */
//             Context->Dr0 = TrapFrame->Dr0;
//             Context->Dr1 = TrapFrame->Dr1;
//             Context->Dr2 = TrapFrame->Dr2;
//             Context->Dr3 = TrapFrame->Dr3;
//             Context->Dr6 = TrapFrame->Dr6;
//
//             /* Update DR7 */
//             Context->Dr7 = KiUpdateDr7(TrapFrame->Dr7);
//         }
//         else
//         {
//             /* Otherwise clear DR registers */
//             Context->Dr0 =
//             Context->Dr1 =
//             Context->Dr2 =
//             Context->Dr3 =
//             Context->Dr6 =
//             Context->Dr7 = 0;
//         }
//     }
//
//     /* Restore IRQL */
//     if (OldIrql < APC_LEVEL) KeLowerIrql(OldIrql);
// }
//
// BOOLEAN
// FASTCALL
// KeInvalidAccessAllowed(IN PVOID TrapInformation OPTIONAL)
// {
//     ULONG Eip;
//     PKTRAP_FRAME TrapFrame = TrapInformation;
//     VOID NTAPI ExpInterlockedPopEntrySListFault(VOID);
//
//     /* Don't do anything if we didn't get a trap frame */
//     if (!TrapInformation) return FALSE;
//
//     /* Check where we came from */
//     switch (TrapFrame->SegCs)
//     {
//         /* Kernel mode */
//         case KGDT_R0_CODE:
//
//             /* Allow S-LIST Routine to fail */
//             Eip = (ULONG)&ExpInterlockedPopEntrySListFault;
//             break;
//
//         /* User code */
//         case KGDT_R3_CODE | RPL_MASK:
//
//             /* Allow S-LIST Routine to fail */
//             //Eip = (ULONG)KeUserPopEntrySListFault;
//             Eip = 0;
//             break;
//
//         default:
//
//             /* Anything else gets a bugcheck */
//             Eip = 0;
//     }
//
//     /* Return TRUE if we want to keep the system up */
//     return (TrapFrame->Eip == Eip) ? TRUE : FALSE;
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
//     EXCEPTION_RECORD LocalExceptRecord;
//
//     /* Increase number of Exception Dispatches */
//     KeGetCurrentPrcb()->KeExceptionDispatchCount++;
//
//     /* Set the context flags */
//     Context.ContextFlags = CONTEXT_FULL | CONTEXT_DEBUG_REGISTERS;
//
//     /* Check if User Mode or if the kernel debugger is enabled */
//     if ((PreviousMode == UserMode) || (KeGetPcr()->KdVersionBlock))
//     {
//         /* Add the FPU Flag */
//         Context.ContextFlags |= CONTEXT_FLOATING_POINT;
//
//         /* Check for NPX Support */
//         if (KeI386FxsrPresent)
//         {
//             /* Save those too */
//             Context.ContextFlags |= CONTEXT_EXTENDED_REGISTERS;
//         }
//     }
//
//     /* Get a Context */
//     KeTrapFrameToContext(TrapFrame, ExceptionFrame, &Context);
//
//     /* Look at our exception code */
//     switch (ExceptionRecord->ExceptionCode)
//     {
//         /* Breakpoint */
//         case STATUS_BREAKPOINT:
//
//             /* Decrement EIP by one */
//             Context.Eip--;
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
//     /* Sanity check */
//     ASSERT(!((PreviousMode == KernelMode) &&
//              (Context.EFlags & EFLAGS_V86_MASK)));
//
//     /* Handle kernel-mode first, it's simpler */
//     if (PreviousMode == KernelMode)
//     {
//         /* Check if this is a first-chance exception */
//         if (FirstChance != FALSE)
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
//         if (ExceptionRecord->ExceptionCode != STATUS_BREAKPOINT)
//         {
//             DbgPrint("[UEXCEPT] PID=%lu Code=0x%lX Addr=%p EIP=0x%lX ESP=0x%lX P0=0x%lX P1=0x%lX\n",
//                      (ULONG)(ULONG_PTR)PsGetCurrentProcessId(),
//                      ExceptionRecord->ExceptionCode,
//                      ExceptionRecord->ExceptionAddress,
//                      Context.Eip,
//                      Context.Esp,
//                      ExceptionRecord->NumberParameters >= 1 ? ExceptionRecord->ExceptionInformation[0] : 0,
//                      ExceptionRecord->NumberParameters >= 2 ? ExceptionRecord->ExceptionInformation[1] : 0);
//             /* Print stack frame to help diagnose call chain */
//             if (MmIsAddressValid((PVOID)Context.Esp))
//             {
//                 PULONG Stack = (PULONG)Context.Esp;
//                 DbgPrint("[UEXCEPT] Stack: [ESP]=%08lX [ESP+4]=%08lX [ESP+8]=%08lX [ESP+C]=%08lX EBP=%08lX\n",
//                          MmIsAddressValid(&Stack[0]) ? Stack[0] : 0,
//                          MmIsAddressValid(&Stack[1]) ? Stack[1] : 0,
//                          MmIsAddressValid(&Stack[2]) ? Stack[2] : 0,
//                          MmIsAddressValid(&Stack[3]) ? Stack[3] : 0,
//                          Context.Ebp);
//             }
//             /* Try to identify the module containing the faulting address */
//             {
//                 PPEB Peb = PsGetCurrentProcess()->Peb;
//                 if (Peb && MmIsAddressValid(Peb) && Peb->Ldr && MmIsAddressValid(Peb->Ldr))
//                 {
//                     PPEB_LDR_DATA Ldr = Peb->Ldr;
//                     PLIST_ENTRY Head = &Ldr->InMemoryOrderModuleList;
//                     PLIST_ENTRY Entry;
//                     if (MmIsAddressValid(Head) && MmIsAddressValid(Head->Flink))
//                     {
//                         for (Entry = Head->Flink; Entry != Head && MmIsAddressValid(Entry); Entry = Entry->Flink)
//                         {
//                             PLDR_DATA_TABLE_ENTRY LdrEntry = CONTAINING_RECORD(Entry, LDR_DATA_TABLE_ENTRY, InMemoryOrderLinks);
//                             if (MmIsAddressValid(LdrEntry) &&
//                                 (ULONG_PTR)ExceptionRecord->ExceptionAddress >= (ULONG_PTR)LdrEntry->DllBase &&
//                                 (ULONG_PTR)ExceptionRecord->ExceptionAddress < (ULONG_PTR)LdrEntry->DllBase + LdrEntry->SizeOfImage)
//                             {
//                                 DbgPrint("[UEXCEPT] Module: %wZ Base=%p Size=0x%lX\n",
//                                          &LdrEntry->BaseDllName, LdrEntry->DllBase, LdrEntry->SizeOfImage);
//                                 break;
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//
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
//                  (KdIsThisAKdTrap(ExceptionRecord,
//                                   &Context,
//                                   PreviousMode)))
//             {
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
//             /* Set up the user-stack */
// DispatchToUser:
//             _SEH2_TRY
//             {
//                 ULONG Size;
//                 ULONG_PTR Stack, NewStack;
//
//                 /* Make sure we have a valid SS and that this isn't V86 mode */
//                 if ((TrapFrame->HardwareSegSs != (KGDT_R3_DATA | RPL_MASK)) ||
//                     (TrapFrame->EFlags & EFLAGS_V86_MASK))
//                 {
//                     /* Raise an exception instead */
//                     LocalExceptRecord.ExceptionCode = STATUS_ACCESS_VIOLATION;
//                     LocalExceptRecord.ExceptionFlags = 0;
//                     LocalExceptRecord.NumberParameters = 0;
//                     RtlRaiseException(&LocalExceptRecord);
//                 }
//
//                 /* Align context size and get stack pointer */
//                 Size = (sizeof(CONTEXT) + 3) & ~3;
//                 Stack = (Context.Esp & ~3) - Size;
//
//                 /* Probe stack and copy Context */
//                 ProbeForWrite((PVOID)Stack, Size, sizeof(ULONG));
//                 RtlCopyMemory((PVOID)Stack, &Context, sizeof(CONTEXT));
//
//                 /* Align exception record size and get stack pointer */
//                 Size = (sizeof(EXCEPTION_RECORD) -
//                        (EXCEPTION_MAXIMUM_PARAMETERS -
//                         ExceptionRecord->NumberParameters) *
//                        sizeof(ULONG) + 3) & ~3;
//                 NewStack = Stack - Size;
//
//                 /* Probe stack and copy exception record */
//                 ProbeForWrite((PVOID)(NewStack - 2 * sizeof(ULONG_PTR)),
//                               Size +  2 * sizeof(ULONG_PTR),
//                               sizeof(ULONG));
//                 RtlCopyMemory((PVOID)NewStack, ExceptionRecord, Size);
//
//                 /* Now write the two params for the user-mode dispatcher */
//                 *(PULONG_PTR)(NewStack - 1 * sizeof(ULONG_PTR)) = Stack;
//                 *(PULONG_PTR)(NewStack - 2 * sizeof(ULONG_PTR)) = NewStack;
//
//                 /* Set new Stack Pointer */
//                 KiSsToTrapFrame(TrapFrame, KGDT_R3_DATA);
//                 KiEspToTrapFrame(TrapFrame, NewStack - 2 * sizeof(ULONG_PTR));
//
//                 /* Force correct segments */
//                 TrapFrame->SegCs = Ke386SanitizeSeg(KGDT_R3_CODE, PreviousMode);
//                 TrapFrame->SegDs = Ke386SanitizeSeg(KGDT_R3_DATA, PreviousMode);
//                 TrapFrame->SegEs = Ke386SanitizeSeg(KGDT_R3_DATA, PreviousMode);
//                 TrapFrame->SegFs = Ke386SanitizeSeg(KGDT_R3_TEB,  PreviousMode);
//                 TrapFrame->SegGs = 0;
//
//                 /* Set EIP to the User-mode Dispatcher */
//                 TrapFrame->Eip = (ULONG)KeUserExceptionDispatcher;
//
//                 /* Dispatch exception to user-mode */
//                 _SEH2_YIELD(return);
//             }
//             _SEH2_EXCEPT((RtlCopyMemory(&LocalExceptRecord, _SEH2_GetExceptionInformation()->ExceptionRecord, sizeof(EXCEPTION_RECORD)), EXCEPTION_EXECUTE_HANDLER))
//             {
//                 /* Check if we got a stack overflow and raise that instead */
//                 if ((NTSTATUS)LocalExceptRecord.ExceptionCode ==
//                     STATUS_STACK_OVERFLOW)
//                 {
//                     /* Copy the exception address and record */
//                     LocalExceptRecord.ExceptionAddress =
//                         ExceptionRecord->ExceptionAddress;
//                     RtlCopyMemory(ExceptionRecord,
//                                   (PVOID)&LocalExceptRecord,
//                                   sizeof(EXCEPTION_RECORD));
//
//                     /* Do the exception again */
//                     _SEH2_YIELD(goto DispatchToUser);
//                 }
//             }
//             _SEH2_END;
//
//             DPRINT("First chance exception in %.16s, ExceptionCode: %lx, ExceptionAddress: %p, P0: %lx, P1: %lx\n",
//                    PsGetCurrentProcess()->ImageFileName,
//                    ExceptionRecord->ExceptionCode,
//                    ExceptionRecord->ExceptionAddress,
//                    ExceptionRecord->ExceptionInformation[0],
//                    ExceptionRecord->ExceptionInformation[1]);
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
//         DPRINT1("Kill %.16s, ExceptionCode: %lx, ExceptionAddress: %p, BaseAddress: %p, P0: %lx, P1: %lx\n",
//                 PsGetCurrentProcess()->ImageFileName,
//                 ExceptionRecord->ExceptionCode,
//                 ExceptionRecord->ExceptionAddress,
//                 PsGetCurrentProcess()->SectionBaseAddress,
//                 ExceptionRecord->ExceptionInformation[0],
//                 ExceptionRecord->ExceptionInformation[1]);
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
// DECLSPEC_NORETURN
// VOID
// NTAPI
// KiDispatchExceptionFromTrapFrame(IN NTSTATUS Code,
//                                  IN ULONG Flags,
//                                  IN ULONG_PTR Address,
//                                  IN ULONG ParameterCount,
//                                  IN ULONG_PTR Parameter1,
//                                  IN ULONG_PTR Parameter2,
//                                  IN ULONG_PTR Parameter3,
//                                  IN PKTRAP_FRAME TrapFrame)
// {
//     EXCEPTION_RECORD ExceptionRecord;
//
//     /* Build the exception record */
//     ExceptionRecord.ExceptionCode = Code;
//     ExceptionRecord.ExceptionFlags = Flags;
//     ExceptionRecord.ExceptionRecord = NULL;
//     ExceptionRecord.ExceptionAddress = (PVOID)Address;
//     ExceptionRecord.NumberParameters = ParameterCount;
//     if (ParameterCount)
//     {
//         /* Copy extra parameters */
//         ExceptionRecord.ExceptionInformation[0] = Parameter1;
//         ExceptionRecord.ExceptionInformation[1] = Parameter2;
//         ExceptionRecord.ExceptionInformation[2] = Parameter3;
//     }
//
//     /* Now go dispatch the exception */
//     KiDispatchException(&ExceptionRecord,
//                         NULL,
//                         TrapFrame,
//                         TrapFrame->EFlags & EFLAGS_V86_MASK ?
//                         -1 : KiUserTrap(TrapFrame),
//                         TRUE);
//
//     /* Return from this trap */
//     KiEoiHelper(TrapFrame);
// }
//
// DECLSPEC_NORETURN
// VOID
// FASTCALL
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
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// KeRaiseUserException(IN NTSTATUS ExceptionCode)
// {
//     ULONG OldEip;
//     PTEB Teb = KeGetCurrentThread()->Teb;
//     PKTRAP_FRAME TrapFrame = KeGetCurrentThread()->TrapFrame;
//
//     /* Make sure we can access the TEB */
//     _SEH2_TRY
//     {
//         /* Set the exception code */
//         Teb->ExceptionCode = ExceptionCode;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Return the exception code */
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Get the old EIP */
//     OldEip = TrapFrame->Eip;
//
//     /* Change it to the user-mode dispatcher */
//     TrapFrame->Eip = (ULONG_PTR)KeRaiseUserExceptionDispatcher;
//
//     /* Return the old EIP */
//     return (NTSTATUS)OldEip;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
