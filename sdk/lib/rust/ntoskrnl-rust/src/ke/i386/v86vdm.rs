use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ps::*;

use ntkrnl_sys::ddk::{
    _CONTEXT, _EPROCESS, _ETHREAD, _KPCR, _KPRCB, _KTRAP_FRAME,
    _Vdm_Tib, _VDM_PROCESS_OBJECTS,
    CONTEXT, PEPROCESS, PETHREAD, PKPCR, PKPRCB, PKTRAP_FRAME,
    PVDM_TIB, PVDM_PROCESS_OBJECTS, PTEB,
    KAFFINITY, PCONTEXT,
    KeContextToTrapFrame, KeTrapFrameToContext,
    KeRevertToUserAffinityThread, KeSetSystemAffinityThread,
    NtVdmControl,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn RtlMoveMemory(dst: PVOID, src: PVOID, len: SIZE_T);
    fn Ki386AdjustEsp0(TrapFrame: PKTRAP_FRAME);
    fn VdmDispatchBop(TrapFrame: PKTRAP_FRAME);
    fn KiGetThreadNpxArea(Thread: PETHREAD) -> PVOID;
    fn KiSetTebBase(Pcr: PKPCR, Teb: PVOID);
    fn KiFillTrapFrameDebug(TrapFrame: PKTRAP_FRAME);
    fn Ki386SetupAndExitToV86Mode(VdmTeb: PTEB);
    fn KiComputeIopmOffset(MapNumber: ULONG) -> USHORT;
    static mut VdmState: PULONG;
    fn _enable();
    fn _disable();
    fn __readeflags() -> ULONG;
    fn _AddressOfReturnAddress() -> *const PVOID;
}
extern "C" {
    fn KeGetTrapFrame(Thread: PETHREAD) -> PKTRAP_FRAME;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiEoiHelper(TrapFrame: PKTRAP_FRAME);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiEoiHelper(TrapFrame: PKTRAP_FRAME);
}

const EFLAGS_USER_SANITIZE: ULONG = 0x003F4DD7;
const EFLAGS_INTERRUPT_MASK: ULONG = 0x200;
const EFLAGS_V86_MASK: ULONG = 0x20000;
const EFLAGS_ALIGN_CHECK: ULONG = 0x40000;
const EFLAGS_NESTED_TASK: ULONG = 0x4000;
const EFLAGS_IOPL: ULONG = 0x3000;
const EFLAGS_VIF: ULONG = 0x80000;
const EFLAGS_VIP: ULONG = 0x100000;
const EFLAGS_TF: ULONG = 0x100;

const PFX_FLAG_ES: ULONG = 0x100;
const PFX_FLAG_CS: ULONG = 0x200;
const PFX_FLAG_SS: ULONG = 0x400;
const PFX_FLAG_DS: ULONG = 0x800;
const PFX_FLAG_FS: ULONG = 0x1000;
const PFX_FLAG_GS: ULONG = 0x2000;
const PFX_FLAG_OPER32: ULONG = 0x4000;
const PFX_FLAG_ADDR32: ULONG = 0x8000;
const PFX_FLAG_LOCK: ULONG = 0x10000;
const PFX_FLAG_REPNE: ULONG = 0x20000;
const PFX_FLAG_REP: ULONG = 0x40000;

const FIXED_NTVDMSTATE_LINEAR_PC_AT: ULONG = 0x00000714;
const TRAMPOLINE_BASE: ULONG = 0x10000;
const TRAMPOLINE_TEB: ULONG = 0x11000;
const TRAMPOLINE_TIB: ULONG = 0x11000;
const TRAMPOLINE_BOP: ULONG = 0xC4C4C4C4;
const EXCEPTION_CHAIN_END: ULONG = 0xFFFFFFFF;
const KGDT_R0_CODE: USHORT = 0x08;
const KGDT_R3_CODE: USHORT = 0x18;
const KGDT_R3_DATA: USHORT = 0x20;
const KGDT_TSS: USHORT = 0x28;
const RPL_MASK: USHORT = 0x03;
const IOPM_OFFSET: ULONG = 0x2000;
const IOPM_SIZE: ULONG = 8192;
const IOPM_COUNT: ULONG = 0x00000001;
const IO_ACCESS_MAP_NONE: ULONG = 0;
const TAG_KERNEL: ULONG = 0x2020654B;
const NPX_FRAME_LENGTH: ULONG = 528;
const CONTEXT_CONTROL: ULONG = 0x00010001;
const CONTEXT_FULL: ULONG = 0x00010007;
const MUTANT_INCREMENT: LONG = 1;

#[inline]
fn KiVdmGetInstructionSize(x: ULONG) -> ULONG {
    x & 0xFF
}

#[inline]
fn KiVdmGetPrefixFlags(x: ULONG) -> ULONG {
    x & 0xFFFFFF00
}

macro_rules! KiVdmClearVdmEFlags {
    ($mask:expr) => {
        unsafe { *VdmState &= !($mask); }
    };
}

macro_rules! KiVdmSetVdmEFlags {
    ($flags:expr) => {
        unsafe { *VdmState |= ($flags); }
    };
}

macro_rules! KiCallVdmHandler {
    ($handler:ident, $tf:expr, $fl:expr) => {
        $handler($tf, $fl)
    };
}

macro_rules! KiCallVdmPrefixHandler {
    ($flag:expr, $tf:expr, $fl:expr) => {
        KiVdmOpcodePrefix($tf, $fl | ($flag))
    };
}

macro_rules! KiVdmUnhandledOpcode {
    ($func_name:ident) => {
        unsafe extern "fastcall" fn $func_name(
            _TrapFrame: PKTRAP_FRAME,
            _Flags: ULONG,
        ) -> BOOLEAN {
            0u8
        }
    };
}

#[repr(C)]
struct KIO_ACCESS_MAP {
    IoMap: [UCHAR; IOPM_SIZE as usize],
}
type PKIO_ACCESS_MAP = *mut KIO_ACCESS_MAP;

#[repr(C)]
struct KTSS {
    _pad0: [u8; 0x66],
    IoMapBase: USHORT,
    _pad1: [u8; IOPM_OFFSET as usize - 0x68],
    IoMaps: [KIO_ACCESS_MAP; IOPM_COUNT as usize],
}
type PKTSS = *mut KTSS;

#[repr(C)]
struct KV86_FRAME {
    ThreadStack: PVOID,
    ThreadTeb: PTEB,
    PcrTeb: PVOID,
}
type PKV86_FRAME = *mut KV86_FRAME;

#[repr(C, align(16))]
struct FX_SAVE_AREA {
    // C sizeof(FX_SAVE_AREA) is 528 (520-byte union U + NpxSavedCpu + Cr0NpxState),
    // not 512 — FXSAVE_FORMAT carries a trailing Align16Byte[8]. A 512-byte model
    // short-copies sizeof(FX_SAVE_AREA) RtlCopyMemory by 16 bytes.
    _data: [u8; 528],
}
type PFX_SAVE_AREA = *mut FX_SAVE_AREA;

#[repr(C, align(16))]
struct KV8086_STACK_FRAME {
    _align_pad: [u8; 0],
    NpxArea: FX_SAVE_AREA,
    V86Frame: KV86_FRAME,
    TrapFrame: _KTRAP_FRAME,
}
type PKV8086_STACK_FRAME = *mut KV8086_STACK_FRAME;

#[no_mangle]
pub static mut KeI386EFlagsAndMaskV86: ULONG = EFLAGS_USER_SANITIZE;

#[no_mangle]
pub static mut KeI386EFlagsOrMaskV86: ULONG = EFLAGS_INTERRUPT_MASK;

#[no_mangle]
pub static mut Ki386IopmSaveArea: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut KeI386VirtualIntExtensions: BOOLEAN = 0u8;

const KiNtVdmState: PULONG = FIXED_NTVDMSTATE_LINEAR_PC_AT as PULONG;

KiVdmUnhandledOpcode!(KiVdmOpcodeF);
KiVdmUnhandledOpcode!(KiVdmOpcodeOUTSW);
KiVdmUnhandledOpcode!(KiVdmOpcodeOUTSB);
KiVdmUnhandledOpcode!(KiVdmOpcodeINSB);
KiVdmUnhandledOpcode!(KiVdmOpcodeINSW);
KiVdmUnhandledOpcode!(KiVdmOpcodeNPX);
KiVdmUnhandledOpcode!(KiVdmOpcodeINBimm);
KiVdmUnhandledOpcode!(KiVdmOpcodeINWimm);
KiVdmUnhandledOpcode!(KiVdmOpcodeOUTBimm);
KiVdmUnhandledOpcode!(KiVdmOpcodeOUTWimm);
KiVdmUnhandledOpcode!(KiVdmOpcodeINB);
KiVdmUnhandledOpcode!(KiVdmOpcodeINW);
KiVdmUnhandledOpcode!(KiVdmOpcodeOUTB);
KiVdmUnhandledOpcode!(KiVdmOpcodeOUTW);
KiVdmUnhandledOpcode!(KiVdmOpcodeHLT);
KiVdmUnhandledOpcode!(KiVdmOpcodeINTO);
KiVdmUnhandledOpcode!(KiVdmOpcodeINV);

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn KiVdmOpcodePUSHF(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    let mut Esp: ULONG;
    let mut V86EFlags: ULONG;
    let TrapEFlags: ULONG;

    V86EFlags = *KiNtVdmState;
    V86EFlags &= !EFLAGS_INTERRUPT_MASK;

    TrapEFlags = (*TrapFrame).EFlags;
    let mut AdjustedTrapEFlags = TrapEFlags;
    if KeI386VirtualIntExtensions != 0 {
        AdjustedTrapEFlags &= !EFLAGS_INTERRUPT_MASK;
        if TrapEFlags & EFLAGS_VIF != 0 {
            AdjustedTrapEFlags |= EFLAGS_INTERRUPT_MASK;
        }
    }
    AdjustedTrapEFlags &= EFLAGS_ALIGN_CHECK | EFLAGS_NESTED_TASK | EFLAGS_INTERRUPT_MASK;

    V86EFlags |= AdjustedTrapEFlags;
    V86EFlags |= EFLAGS_IOPL;

    Esp = ((*TrapFrame).HardwareSegSs << 4) + ((*TrapFrame).HardwareEsp as USHORT) as ULONG;

    if KiVdmGetPrefixFlags(Flags) & PFX_FLAG_OPER32 != 0 {
        Esp -= 4;
        *(Esp as *mut ULONG) = V86EFlags;
    } else {
        Esp -= 2;
        *(Esp as *mut USHORT) = V86EFlags as USHORT;
    }

    (*TrapFrame).HardwareEsp = Esp - ((*TrapFrame).HardwareSegSs << 4);
    (*TrapFrame).Eip += KiVdmGetInstructionSize(Flags);

    1u8
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiVdmOpcodePUSHF(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    0u8
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn KiVdmOpcodePOPF(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    let mut Esp: ULONG;
    let mut V86EFlags: ULONG;
    let mut EFlags: ULONG;
    let TrapEFlags: ULONG;

    Esp = ((*TrapFrame).HardwareSegSs << 4) + ((*TrapFrame).HardwareEsp as USHORT) as ULONG;

    if KiVdmGetPrefixFlags(Flags) & PFX_FLAG_OPER32 != 0 {
        EFlags = *(Esp as *const ULONG);
        Esp += 4;
    } else {
        EFlags = *(Esp as *const USHORT) as ULONG;
        Esp += 2;
    }

    (*TrapFrame).HardwareEsp = Esp - ((*TrapFrame).HardwareSegSs << 4);

    EFlags &= !EFLAGS_IOPL;

    V86EFlags = EFlags & !EFLAGS_NESTED_TASK;

    EFlags &= EFLAGS_ALIGN_CHECK | EFLAGS_NESTED_TASK | EFLAGS_INTERRUPT_MASK;

    TrapEFlags = (*TrapFrame).EFlags;

    if KeI386VirtualIntExtensions != 0 {
        let mut AdjustedV86EFlags = V86EFlags & !EFLAGS_VIF;
        if V86EFlags & EFLAGS_INTERRUPT_MASK != 0 {
            AdjustedV86EFlags |= EFLAGS_VIF;
            V86EFlags &= !EFLAGS_INTERRUPT_MASK;
        }
        V86EFlags = AdjustedV86EFlags;
    }

    V86EFlags |= EFLAGS_V86_MASK;

    (*TrapFrame).EFlags |= V86EFlags;

    if TrapEFlags & EFLAGS_V86_MASK != 0 {
        Ki386AdjustEsp0(TrapFrame);
    }

    KiVdmClearVdmEFlags!(EFLAGS_ALIGN_CHECK | EFLAGS_NESTED_TASK | EFLAGS_INTERRUPT_MASK);
    KiVdmSetVdmEFlags!(EFlags);

    (*TrapFrame).Eip += KiVdmGetInstructionSize(Flags);

    1u8
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiVdmOpcodePOPF(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    0u8
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn KiVdmOpcodeINTnn(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    let mut Esp: ULONG;
    let mut V86EFlags: ULONG;
    let TrapEFlags: ULONG;
    let mut Eip: ULONG;
    let Interrupt: ULONG;

    TrapEFlags = (*TrapFrame).EFlags;

    V86EFlags = *KiNtVdmState;
    KiVdmClearVdmEFlags!(EFLAGS_INTERRUPT_MASK);

    V86EFlags &= EFLAGS_ALIGN_CHECK | EFLAGS_INTERRUPT_MASK;

    if KeI386VirtualIntExtensions != 0 {
        V86EFlags &= !EFLAGS_INTERRUPT_MASK;
        if TrapEFlags & EFLAGS_VIF != 0 {
            V86EFlags |= EFLAGS_INTERRUPT_MASK;
        }
    }

    V86EFlags |= TrapEFlags & !EFLAGS_INTERRUPT_MASK;

    (*TrapFrame).EFlags = TrapEFlags & !(EFLAGS_VIF | EFLAGS_NESTED_TASK | EFLAGS_TF);

    V86EFlags |= EFLAGS_IOPL;

    Esp = ((*TrapFrame).HardwareSegSs << 4) + (*TrapFrame).HardwareEsp;

    Esp -= 2;
    *(Esp as *mut USHORT) = V86EFlags as USHORT;

    Esp -= 2;
    *(Esp as *mut USHORT) = (*TrapFrame).SegCs as USHORT;

    Esp -= 2;
    *(Esp as *mut USHORT) = ((*TrapFrame).Eip + KiVdmGetInstructionSize(Flags) + 1) as USHORT;

    (*TrapFrame).HardwareEsp = Esp as USHORT as ULONG;

    Eip = ((*TrapFrame).SegCs << 4) + (*TrapFrame).Eip;

    Eip += KiVdmGetInstructionSize(Flags);

    let InterruptNum = *(Eip as *const UCHAR);

    let InterruptVec = *((InterruptNum as ULONG * 4) as *const ULONG);
    (*TrapFrame).Eip = InterruptVec as USHORT as ULONG;

    let mut InterruptCs = (InterruptVec >> 16) as USHORT;

    if (*TrapFrame).EFlags & EFLAGS_V86_MASK == 0 {
        InterruptCs |= RPL_MASK;
        if (*TrapFrame).SegCs as USHORT == KGDT_R0_CODE {
            (*TrapFrame).SegCs = InterruptCs as ULONG;
        } else {
            (*TrapFrame).SegCs = (KGDT_R3_CODE | RPL_MASK) as ULONG;
        }
    } else {
        (*TrapFrame).SegCs = InterruptCs as ULONG;
    }

    1u8
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiVdmOpcodeINTnn(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    0u8
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn KiVdmOpcodeIRET(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    let mut Esp: ULONG;
    let mut V86EFlags: ULONG;
    let mut EFlags: ULONG;
    let TrapEFlags: ULONG;
    let Eip: ULONG;

    Esp = ((*TrapFrame).HardwareSegSs << 4) + (*TrapFrame).HardwareEsp;

    if KiVdmGetPrefixFlags(Flags) & PFX_FLAG_OPER32 != 0 {
        (*TrapFrame).Eip = *(Esp as *const ULONG);
        (*TrapFrame).SegCs = *(Esp.wrapping_add(4) as *const USHORT) as ULONG;

        (*TrapFrame).HardwareEsp += 12;

        EFlags = *(Esp.wrapping_add(8) as *const ULONG);
    } else {
        (*TrapFrame).Eip = *(Esp as *const USHORT) as ULONG;
        (*TrapFrame).SegCs = *(Esp.wrapping_add(2) as *const USHORT) as ULONG;

        (*TrapFrame).HardwareEsp += 6;

        EFlags = *(Esp.wrapping_add(4) as *const USHORT) as ULONG;
    }

    EFlags &= !(EFLAGS_VIP | EFLAGS_VIF | EFLAGS_NESTED_TASK | EFLAGS_IOPL);
    V86EFlags = EFlags;

    if KeI386VirtualIntExtensions != 0 {
        if EFlags & EFLAGS_INTERRUPT_MASK != 0 {
            EFlags |= EFLAGS_VIF;
        }
    }

    EFlags |= EFLAGS_V86_MASK | EFLAGS_INTERRUPT_MASK;

    TrapEFlags = (*TrapFrame).EFlags;
    (*TrapFrame).EFlags = ((*TrapFrame).EFlags & EFLAGS_VIP) | EFlags;

    if TrapEFlags & EFLAGS_V86_MASK == 0 {
        Ki386AdjustEsp0(TrapFrame);
    }

    KiVdmClearVdmEFlags!(EFLAGS_INTERRUPT_MASK);
    KiVdmSetVdmEFlags!(V86EFlags);

    Eip = ((*TrapFrame).SegCs << 4) + (*TrapFrame).Eip;
    if *(Eip as *const USHORT) == 0xC4C4 {
        VdmDispatchBop(TrapFrame);
    }

    1u8
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiVdmOpcodeIRET(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    0u8
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn KiVdmOpcodeCLI(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    KiVdmClearVdmEFlags!(EFLAGS_INTERRUPT_MASK);

    (*TrapFrame).Eip += KiVdmGetInstructionSize(Flags);

    1u8
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiVdmOpcodeCLI(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    0u8
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn KiVdmOpcodeSTI(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    KiVdmSetVdmEFlags!(EFLAGS_INTERRUPT_MASK);

    (*TrapFrame).Eip += KiVdmGetInstructionSize(Flags);

    1u8
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiVdmOpcodeSTI(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    0u8
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn KiVdmHandleOpcode(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    let mut Eip: ULONG;

    Eip = ((*TrapFrame).SegCs << 4) + (*TrapFrame).Eip;
    Eip += KiVdmGetInstructionSize(Flags) - 1;

    match *(Eip as *const UCHAR) {
        0xF => KiCallVdmHandler!(KiVdmOpcodeF, TrapFrame, Flags),
        0x26 => KiCallVdmPrefixHandler!(PFX_FLAG_ES, TrapFrame, Flags),
        0x2E => KiCallVdmPrefixHandler!(PFX_FLAG_CS, TrapFrame, Flags),
        0x36 => KiCallVdmPrefixHandler!(PFX_FLAG_SS, TrapFrame, Flags),
        0x3E => KiCallVdmPrefixHandler!(PFX_FLAG_DS, TrapFrame, Flags),
        0x64 => KiCallVdmPrefixHandler!(PFX_FLAG_FS, TrapFrame, Flags),
        0x65 => KiCallVdmPrefixHandler!(PFX_FLAG_GS, TrapFrame, Flags),
        0x66 => KiCallVdmPrefixHandler!(PFX_FLAG_OPER32, TrapFrame, Flags),
        0x67 => KiCallVdmPrefixHandler!(PFX_FLAG_ADDR32, TrapFrame, Flags),
        0xF0 => KiCallVdmPrefixHandler!(PFX_FLAG_LOCK, TrapFrame, Flags),
        0xF2 => KiCallVdmPrefixHandler!(PFX_FLAG_REPNE, TrapFrame, Flags),
        0xF3 => KiCallVdmPrefixHandler!(PFX_FLAG_REP, TrapFrame, Flags),
        0x6C => KiCallVdmHandler!(KiVdmOpcodeINSB, TrapFrame, Flags),
        0x6D => KiCallVdmHandler!(KiVdmOpcodeINSW, TrapFrame, Flags),
        0x6E => KiCallVdmHandler!(KiVdmOpcodeOUTSB, TrapFrame, Flags),
        0x6F => KiCallVdmHandler!(KiVdmOpcodeOUTSW, TrapFrame, Flags),
        0x98 => KiCallVdmHandler!(KiVdmOpcodeNPX, TrapFrame, Flags),
        0xD8 => KiCallVdmHandler!(KiVdmOpcodeNPX, TrapFrame, Flags),
        0xD9 => KiCallVdmHandler!(KiVdmOpcodeNPX, TrapFrame, Flags),
        0xDA => KiCallVdmHandler!(KiVdmOpcodeNPX, TrapFrame, Flags),
        0xDB => KiCallVdmHandler!(KiVdmOpcodeNPX, TrapFrame, Flags),
        0xDC => KiCallVdmHandler!(KiVdmOpcodeNPX, TrapFrame, Flags),
        0xDD => KiCallVdmHandler!(KiVdmOpcodeNPX, TrapFrame, Flags),
        0xDE => KiCallVdmHandler!(KiVdmOpcodeNPX, TrapFrame, Flags),
        0xDF => KiCallVdmHandler!(KiVdmOpcodeNPX, TrapFrame, Flags),
        0x9C => KiCallVdmHandler!(KiVdmOpcodePUSHF, TrapFrame, Flags),
        0x9D => KiCallVdmHandler!(KiVdmOpcodePOPF, TrapFrame, Flags),
        0xCD => KiCallVdmHandler!(KiVdmOpcodeINTnn, TrapFrame, Flags),
        0xCE => KiCallVdmHandler!(KiVdmOpcodeINTO, TrapFrame, Flags),
        0xCF => KiCallVdmHandler!(KiVdmOpcodeIRET, TrapFrame, Flags),
        0xE4 => KiCallVdmHandler!(KiVdmOpcodeINBimm, TrapFrame, Flags),
        0xE5 => KiCallVdmHandler!(KiVdmOpcodeINWimm, TrapFrame, Flags),
        0xE6 => KiCallVdmHandler!(KiVdmOpcodeOUTBimm, TrapFrame, Flags),
        0xE7 => KiCallVdmHandler!(KiVdmOpcodeOUTWimm, TrapFrame, Flags),
        0xEC => KiCallVdmHandler!(KiVdmOpcodeINB, TrapFrame, Flags),
        0xED => KiCallVdmHandler!(KiVdmOpcodeINW, TrapFrame, Flags),
        0xEE => KiCallVdmHandler!(KiVdmOpcodeOUTB, TrapFrame, Flags),
        0xEF => KiCallVdmHandler!(KiVdmOpcodeOUTW, TrapFrame, Flags),
        0xF4 => KiCallVdmHandler!(KiVdmOpcodeHLT, TrapFrame, Flags),
        0xFA => KiCallVdmHandler!(KiVdmOpcodeCLI, TrapFrame, Flags),
        0xFB => KiCallVdmHandler!(KiVdmOpcodeSTI, TrapFrame, Flags),
        _ => KiCallVdmHandler!(KiVdmOpcodeINV, TrapFrame, Flags),
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiVdmHandleOpcode(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    0u8
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn KiVdmOpcodePrefix(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    let NewFlags = Flags + 1;
    KiVdmHandleOpcode(TrapFrame, NewFlags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KiVdmOpcodePrefix(
    TrapFrame: PKTRAP_FRAME,
    Flags: ULONG,
) -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "fastcall" fn Ki386HandleOpcodeV86(TrapFrame: PKTRAP_FRAME) -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "fastcall" fn KiExitV86Mode(TrapFrame: PKTRAP_FRAME) -> ULONG_PTR {
    0
}

#[no_mangle]
pub unsafe extern "fastcall" fn KiEnterV86Mode(StackFrameUnaligned: ULONG_PTR) {
}

#[no_mangle]
pub unsafe extern "system" fn Ke386SetIOPL() {
}

#[no_mangle]
pub unsafe extern "system" fn Ke386CallBios(
    Int: ULONG,
    Context: PCONTEXT,
) -> NTSTATUS {
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn Ke386IoSetAccessProcess(
    Process: *mut _EPROCESS,
    MapNumber: ULONG,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn Ke386SetIoAccessMap(
    MapNumber: ULONG,
    IopmBuffer: PKIO_ACCESS_MAP,
) -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn Ke386QueryIoAccessMap(
    MapNumber: ULONG,
    IopmBuffer: PKIO_ACCESS_MAP,
) -> BOOLEAN {
    0u8
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/ke/i386/v86vdm.c
//  * PURPOSE:         V8086 and VDM Trap Emulation
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  *                  Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define KiVdmGetInstructionSize(x) ((x) & 0xFF)
// #define KiVdmGetPrefixFlags(x)     ((x) & 0xFFFFFF00)
//
// /* GLOBALS ********************************************************************/
//
// ULONG KeI386EFlagsAndMaskV86 = EFLAGS_USER_SANITIZE;
// ULONG KeI386EFlagsOrMaskV86 = EFLAGS_INTERRUPT_MASK;
// PVOID Ki386IopmSaveArea;
// BOOLEAN KeI386VirtualIntExtensions = FALSE;
// const PULONG KiNtVdmState = (PULONG)FIXED_NTVDMSTATE_LINEAR_PC_AT;
//
// /* UNHANDLED OPCODES **********************************************************/
//
// KiVdmUnhandledOpcode(F);
// KiVdmUnhandledOpcode(OUTSW);
// KiVdmUnhandledOpcode(OUTSB);
// KiVdmUnhandledOpcode(INSB);
// KiVdmUnhandledOpcode(INSW);
// KiVdmUnhandledOpcode(NPX);
// KiVdmUnhandledOpcode(INBimm);
// KiVdmUnhandledOpcode(INWimm);
// KiVdmUnhandledOpcode(OUTBimm);
// KiVdmUnhandledOpcode(OUTWimm);
// KiVdmUnhandledOpcode(INB);
// KiVdmUnhandledOpcode(INW);
// KiVdmUnhandledOpcode(OUTB);
// KiVdmUnhandledOpcode(OUTW);
// KiVdmUnhandledOpcode(HLT);
// KiVdmUnhandledOpcode(INTO);
// KiVdmUnhandledOpcode(INV);
//
// /* OPCODE HANDLERS ************************************************************/
//
// BOOLEAN
// FASTCALL
// KiVdmOpcodePUSHF(IN PKTRAP_FRAME TrapFrame,
//                  IN ULONG Flags)
// {
//     ULONG Esp, V86EFlags, TrapEFlags;
//
//     /* Get current V8086 flags and mask out interrupt flag */
//     V86EFlags = *KiNtVdmState;
//     V86EFlags &= ~EFLAGS_INTERRUPT_MASK;
//
//     /* Get trap frame EFLags */
//     TrapEFlags = TrapFrame->EFlags;
//     /* Check for VME support */
//     if(KeI386VirtualIntExtensions)
//     {
//         /* Copy the virtual interrupt flag to the interrupt flag */
//         TrapEFlags &= ~EFLAGS_INTERRUPT_MASK;
//         if(TrapEFlags & EFLAGS_VIF)
//             TrapEFlags |= EFLAGS_INTERRUPT_MASK;
//     }
//     /* Leave only align, nested task and interrupt */
//     TrapEFlags &= (EFLAGS_ALIGN_CHECK | EFLAGS_NESTED_TASK | EFLAGS_INTERRUPT_MASK);
//
//     /* Add in those flags if they exist, and add in the IOPL flag */
//     V86EFlags |= TrapEFlags;
//     V86EFlags |= EFLAGS_IOPL;
//
//     /* Build flat ESP */
//     Esp = (TrapFrame->HardwareSegSs << 4) + (USHORT)TrapFrame->HardwareEsp;
//
//     /* Check for OPER32 */
//     if (KiVdmGetPrefixFlags(Flags) & PFX_FLAG_OPER32)
//     {
//         /* Save EFlags */
//         Esp -= 4;
//         *(PULONG)Esp = V86EFlags;
//     }
//     else
//     {
//         /* Save EFLags */
//         Esp -= 2;
//         *(PUSHORT)Esp = (USHORT)V86EFlags;
//     }
//
//     /* Set new ESP and EIP */
//     TrapFrame->HardwareEsp = Esp - (TrapFrame->HardwareSegSs << 4);
//     TrapFrame->Eip += KiVdmGetInstructionSize(Flags);
//
//     /* We're done */
//     return TRUE;
// }
//
// BOOLEAN
// FASTCALL
// KiVdmOpcodePOPF(IN PKTRAP_FRAME TrapFrame,
//                 IN ULONG Flags)
// {
//     ULONG Esp, V86EFlags, EFlags, TrapEFlags;
//
//     /* Build flat ESP */
//     Esp = (TrapFrame->HardwareSegSs << 4) + (USHORT)TrapFrame->HardwareEsp;
//
//     /* Check for OPER32 */
//     if (KiVdmGetPrefixFlags(Flags) & PFX_FLAG_OPER32)
//     {
//         /* Read EFlags */
//         EFlags = *(PULONG)Esp;
//         Esp += 4;
//     }
//     else
//     {
//         /* Read EFlags */
//         EFlags = *(PUSHORT)Esp;
//         Esp += 2;
//     }
//
//     /* Set new ESP */
//     TrapFrame->HardwareEsp = Esp - (TrapFrame->HardwareSegSs << 4);
//
//     /* Mask out IOPL from the flags */
//     EFlags &= ~EFLAGS_IOPL;
//
//     /* Save the V86 flags, but mask out the nested task flag */
//     V86EFlags = EFlags & ~EFLAGS_NESTED_TASK;
//
//     /* Now leave only alignment, nested task and interrupt flag */
//     EFlags &= (EFLAGS_ALIGN_CHECK | EFLAGS_NESTED_TASK | EFLAGS_INTERRUPT_MASK);
//
//     /* Get trap EFlags */
//     TrapEFlags = TrapFrame->EFlags;
//
//     /* Check for VME support */
//     if(KeI386VirtualIntExtensions)
//     {
//         /* Copy the IF flag into the VIF one */
//         V86EFlags &= ~EFLAGS_VIF;
//         if(V86EFlags & EFLAGS_INTERRUPT_MASK)
//         {
//             V86EFlags |= EFLAGS_VIF;
//             /* Don't set the interrupt flag */
//             V86EFlags &= ~EFLAGS_INTERRUPT_MASK;
//         }
//     }
//
//     /* Add V86 flag */
//     V86EFlags |= EFLAGS_V86_MASK;
//
//     /* Update EFlags in trap frame */
//     TrapFrame->EFlags |= V86EFlags;
//
//     /* Check if ESP0 needs to be fixed up */
//     if (TrapEFlags & EFLAGS_V86_MASK) Ki386AdjustEsp0(TrapFrame);
//
//     /* Update the V8086 EFlags state */
//     KiVdmClearVdmEFlags(EFLAGS_ALIGN_CHECK | EFLAGS_NESTED_TASK | EFLAGS_INTERRUPT_MASK);
//     KiVdmSetVdmEFlags(EFlags);
//
//     /* FIXME: Check for VDM interrupts */
//
//     /* Update EIP */
//     TrapFrame->Eip += KiVdmGetInstructionSize(Flags);
//
//     /* We're done */
//     return TRUE;
// }
//
// BOOLEAN
// FASTCALL
// KiVdmOpcodeINTnn(IN PKTRAP_FRAME TrapFrame,
//                  IN ULONG Flags)
// {
//     ULONG Esp, V86EFlags, TrapEFlags, Eip, Interrupt;
//
//     /* Read trap frame EFlags */
//     TrapEFlags = TrapFrame->EFlags;
//
//     /* Remove interrupt flag from V8086 EFlags */
//     V86EFlags = *KiNtVdmState;
//     KiVdmClearVdmEFlags(EFLAGS_INTERRUPT_MASK);
//
//     /* Keep only alignment and interrupt flag from the V8086 state */
//     V86EFlags &= (EFLAGS_ALIGN_CHECK | EFLAGS_INTERRUPT_MASK);
//
//     /* Check for VME support */
//     if (KeI386VirtualIntExtensions)
//     {
//         /* Set IF based on VIF */
//         V86EFlags &= ~EFLAGS_INTERRUPT_MASK;
//         if (TrapEFlags & EFLAGS_VIF)
//         {
//             V86EFlags |= EFLAGS_INTERRUPT_MASK;
//         }
//     }
//
//     /* Mask in the relevant V86 EFlags into the trap flags */
//     V86EFlags |= (TrapEFlags & ~EFLAGS_INTERRUPT_MASK);
//
//     /* And mask out the VIF, nested task and TF flag from the trap flags */
//     TrapFrame->EFlags = TrapEFlags & ~(EFLAGS_VIF | EFLAGS_NESTED_TASK | EFLAGS_TF);
//
//     /* Add the IOPL flag to the local trap flags */
//     V86EFlags |= EFLAGS_IOPL;
//
//     /* Build flat ESP */
//     Esp = (TrapFrame->HardwareSegSs << 4) + TrapFrame->HardwareEsp;
//
//     /* Push EFlags */
//     Esp -= 2;
//     *(PUSHORT)(Esp) = (USHORT)V86EFlags;
//
//     /* Push CS */
//     Esp -= 2;
//     *(PUSHORT)(Esp) = (USHORT)TrapFrame->SegCs;
//
//     /* Push IP */
//     Esp -= 2;
//     *(PUSHORT)(Esp) = (USHORT)TrapFrame->Eip + KiVdmGetInstructionSize(Flags) + 1;
//
//     /* Update ESP */
//     TrapFrame->HardwareEsp = (USHORT)Esp;
//
//     /* Get flat EIP */
//     Eip = (TrapFrame->SegCs << 4) + TrapFrame->Eip;
//
//     /* Now get the *next* EIP address (current is original + the count - 1) */
//     Eip += KiVdmGetInstructionSize(Flags);
//
//     /* Now read the interrupt number */
//     Interrupt = *(PUCHAR)Eip;
//
//     /* Read the EIP from its IVT entry */
//     Interrupt = *(PULONG)(Interrupt * 4);
//     TrapFrame->Eip = (USHORT)Interrupt;
//
//     /* Now get the CS segment */
//     Interrupt = (USHORT)(Interrupt >> 16);
//
//     /* Check if the trap was not V8086 trap */
//     if (!(TrapFrame->EFlags & EFLAGS_V86_MASK))
//     {
//         /* Was it a kernel CS? */
//         Interrupt |= RPL_MASK;
//         if (TrapFrame->SegCs == KGDT_R0_CODE)
//         {
//             /* Add the RPL mask */
//             TrapFrame->SegCs = Interrupt;
//         }
//         else
//         {
//             /* Set user CS */
//             TrapFrame->SegCs = KGDT_R3_CODE | RPL_MASK;
//         }
//     }
//     else
//     {
//         /* Set IVT CS */
//         TrapFrame->SegCs = Interrupt;
//     }
//
//     /* We're done */
//     return TRUE;
// }
//
// BOOLEAN
// FASTCALL
// KiVdmOpcodeIRET(IN PKTRAP_FRAME TrapFrame,
//                 IN ULONG Flags)
// {
//     ULONG Esp, V86EFlags, EFlags, TrapEFlags, Eip;
//
//     /* Build flat ESP */
//     Esp = (TrapFrame->HardwareSegSs << 4) + TrapFrame->HardwareEsp;
//
//     /* Check for OPER32 */
//     if (KiVdmGetPrefixFlags(Flags) & PFX_FLAG_OPER32)
//     {
//         /* Build segmented EIP */
//         TrapFrame->Eip = *(PULONG)Esp;
//         TrapFrame->SegCs = *(PUSHORT)(Esp + 4);
//
//         /* Set new ESP */
//         TrapFrame->HardwareEsp += 12;
//
//         /* Get EFLAGS */
//         EFlags = *(PULONG)(Esp + 8);
//     }
//     else
//     {
//         /* Build segmented EIP */
//         TrapFrame->Eip = *(PUSHORT)Esp;
//         TrapFrame->SegCs = *(PUSHORT)(Esp + 2);
//
//         /* Set new ESP */
//         TrapFrame->HardwareEsp += 6;
//
//         /* Get EFLAGS */
//         EFlags = *(PUSHORT)(Esp + 4);
//     }
//
//     /* Mask out EFlags */
//     EFlags &= ~(EFLAGS_VIP | EFLAGS_VIF | EFLAGS_NESTED_TASK | EFLAGS_IOPL);
//     V86EFlags = EFlags;
//
//     /* Check for VME support */
//     if (KeI386VirtualIntExtensions)
//     {
//         if (EFlags & EFLAGS_INTERRUPT_MASK)
//         {
//             EFlags |= EFLAGS_VIF;
//         }
//     }
//
//     /* Add V86 and Interrupt flag */
//     EFlags |= EFLAGS_V86_MASK | EFLAGS_INTERRUPT_MASK;
//
//     /* Update EFlags in trap frame */
//     TrapEFlags = TrapFrame->EFlags;
//     TrapFrame->EFlags = (TrapFrame->EFlags & EFLAGS_VIP) | EFlags;
//
//     /* Check if ESP0 needs to be fixed up */
//     if (!(TrapEFlags & EFLAGS_V86_MASK)) Ki386AdjustEsp0(TrapFrame);
//
//     /* Update the V8086 EFlags state */
//     KiVdmClearVdmEFlags(EFLAGS_INTERRUPT_MASK);
//     KiVdmSetVdmEFlags(V86EFlags);
//
//     /* Build flat EIP and check if this is the BOP instruction */
//     Eip = (TrapFrame->SegCs << 4) + TrapFrame->Eip;
//     if (*(PUSHORT)Eip == 0xC4C4)
//     {
//         /* Dispatch the BOP */
//         VdmDispatchBop(TrapFrame);
//     }
//     else
//     {
//         /* FIXME: Check for VDM interrupts */
//        DPRINT("FIXME: Check for VDM interrupts\n");
//     }
//
//     /* We're done */
//     return TRUE;
// }
//
// BOOLEAN
// FASTCALL
// KiVdmOpcodeCLI(IN PKTRAP_FRAME TrapFrame,
//                IN ULONG Flags)
// {
//     /* Check for VME support */
//     ASSERT(KeI386VirtualIntExtensions == FALSE);
//
//     /* Disable interrupts */
//     KiVdmClearVdmEFlags(EFLAGS_INTERRUPT_MASK);
//
//     /* Skip instruction */
//     TrapFrame->Eip += KiVdmGetInstructionSize(Flags);
//
//     /* Done */
//     return TRUE;
// }
//
// BOOLEAN
// FASTCALL
// KiVdmOpcodeSTI(IN PKTRAP_FRAME TrapFrame,
//                IN ULONG Flags)
// {
//     /* Check for VME support */
//     ASSERT(KeI386VirtualIntExtensions == FALSE);
//
//     /* Enable interrupts */
//     KiVdmSetVdmEFlags(EFLAGS_INTERRUPT_MASK);
//
//     /* Skip instruction */
//     TrapFrame->Eip += KiVdmGetInstructionSize(Flags);
//
//     /* Done */
//     return TRUE;
// }
//
// /* MASTER OPCODE HANDLER ******************************************************/
//
// BOOLEAN
// FASTCALL
// KiVdmHandleOpcode(IN PKTRAP_FRAME TrapFrame,
//                   IN ULONG Flags)
// {
//     ULONG Eip;
//
//     /* Get flat EIP of the *current* instruction (not the original EIP) */
//     Eip = (TrapFrame->SegCs << 4) + TrapFrame->Eip;
//     Eip += KiVdmGetInstructionSize(Flags) - 1;
//
//     /* Read the opcode entry */
//     switch (*(PUCHAR)Eip)
//     {
//         case 0xF:               return KiCallVdmHandler(F);
//         case 0x26:              return KiCallVdmPrefixHandler(PFX_FLAG_ES);
//         case 0x2E:              return KiCallVdmPrefixHandler(PFX_FLAG_CS);
//         case 0x36:              return KiCallVdmPrefixHandler(PFX_FLAG_SS);
//         case 0x3E:              return KiCallVdmPrefixHandler(PFX_FLAG_DS);
//         case 0x64:              return KiCallVdmPrefixHandler(PFX_FLAG_FS);
//         case 0x65:              return KiCallVdmPrefixHandler(PFX_FLAG_GS);
//         case 0x66:              return KiCallVdmPrefixHandler(PFX_FLAG_OPER32);
//         case 0x67:              return KiCallVdmPrefixHandler(PFX_FLAG_ADDR32);
//         case 0xF0:              return KiCallVdmPrefixHandler(PFX_FLAG_LOCK);
//         case 0xF2:              return KiCallVdmPrefixHandler(PFX_FLAG_REPNE);
//         case 0xF3:              return KiCallVdmPrefixHandler(PFX_FLAG_REP);
//         case 0x6C:              return KiCallVdmHandler(INSB);
//         case 0x6D:              return KiCallVdmHandler(INSW);
//         case 0x6E:              return KiCallVdmHandler(OUTSB);
//         case 0x6F:              return KiCallVdmHandler(OUTSW);
//         case 0x98:              return KiCallVdmHandler(NPX);
//         case 0xD8:              return KiCallVdmHandler(NPX);
//         case 0xD9:              return KiCallVdmHandler(NPX);
//         case 0xDA:              return KiCallVdmHandler(NPX);
//         case 0xDB:              return KiCallVdmHandler(NPX);
//         case 0xDC:              return KiCallVdmHandler(NPX);
//         case 0xDD:              return KiCallVdmHandler(NPX);
//         case 0xDE:              return KiCallVdmHandler(NPX);
//         case 0xDF:              return KiCallVdmHandler(NPX);
//         case 0x9C:              return KiCallVdmHandler(PUSHF);
//         case 0x9D:              return KiCallVdmHandler(POPF);
//         case 0xCD:              return KiCallVdmHandler(INTnn);
//         case 0xCE:              return KiCallVdmHandler(INTO);
//         case 0xCF:              return KiCallVdmHandler(IRET);
//         case 0xE4:              return KiCallVdmHandler(INBimm);
//         case 0xE5:              return KiCallVdmHandler(INWimm);
//         case 0xE6:              return KiCallVdmHandler(OUTBimm);
//         case 0xE7:              return KiCallVdmHandler(OUTWimm);
//         case 0xEC:              return KiCallVdmHandler(INB);
//         case 0xED:              return KiCallVdmHandler(INW);
//         case 0xEE:              return KiCallVdmHandler(OUTB);
//         case 0xEF:              return KiCallVdmHandler(OUTW);
//         case 0xF4:              return KiCallVdmHandler(HLT);
//         case 0xFA:              return KiCallVdmHandler(CLI);
//         case 0xFB:              return KiCallVdmHandler(STI);
//         default:
//             DPRINT1("Unhandled instruction: 0x%02x.\n", *(PUCHAR)Eip);
//             return KiCallVdmHandler(INV);
//     }
// }
//
// /* PREFIX HANDLER *************************************************************/
//
// BOOLEAN
// FASTCALL
// KiVdmOpcodePrefix(IN PKTRAP_FRAME TrapFrame,
//                   IN ULONG Flags)
// {
//     /* Increase instruction size */
//     Flags++;
//
//     /* Handle the next opcode */
//     return KiVdmHandleOpcode(TrapFrame, Flags);
// }
//
// /* TRAP HANDLER ***************************************************************/
//
// BOOLEAN
// FASTCALL
// Ki386HandleOpcodeV86(IN PKTRAP_FRAME TrapFrame)
// {
//     /* Clean up */
//     TrapFrame->Eip &= 0xFFFF;
//     TrapFrame->HardwareEsp &= 0xFFFF;
//
//     /* We start with only 1 byte per instruction */
//     return KiVdmHandleOpcode(TrapFrame, 1);
// }
//
// ULONG_PTR
// FASTCALL
// KiExitV86Mode(IN PKTRAP_FRAME TrapFrame)
// {
//     PKPCR Pcr = KeGetPcr();
//     ULONG_PTR StackFrameUnaligned;
//     PKV8086_STACK_FRAME StackFrame;
//     PKTHREAD Thread;
//     PKV86_FRAME V86Frame;
//     PFX_SAVE_AREA NpxFrame;
//
//     /* Get the stack frame back */
//     StackFrameUnaligned = TrapFrame->Esi;
//     StackFrame = (PKV8086_STACK_FRAME)(ROUND_UP(StackFrameUnaligned - 4, 16) + 4);
//     V86Frame = &StackFrame->V86Frame;
//     NpxFrame = &StackFrame->NpxArea;
//     ASSERT((ULONG_PTR)NpxFrame % 16 == 0);
//
//     /* Copy the FPU frame back */
//     Thread = KeGetCurrentThread();
//     RtlCopyMemory(KiGetThreadNpxArea(Thread), NpxFrame, sizeof(FX_SAVE_AREA));
//
//     /* Set initial stack back */
//     Thread->InitialStack = (PVOID)((ULONG_PTR)V86Frame->ThreadStack + sizeof(FX_SAVE_AREA));
//
//     /* Set ESP0 back in the KTSS */
//     Pcr->TSS->Esp0 = (ULONG_PTR)Thread->InitialStack;
//     Pcr->TSS->Esp0 -= sizeof(KTRAP_FRAME) - FIELD_OFFSET(KTRAP_FRAME, V86Es);
//     Pcr->TSS->Esp0 -= NPX_FRAME_LENGTH;
//
//     /* Restore TEB addresses */
//     Thread->Teb = V86Frame->ThreadTeb;
//     KiSetTebBase(KeGetPcr(), V86Frame->ThreadTeb);
//
//     /* Enable interrupts and return a pointer to the trap frame */
//     _enable();
//     return StackFrameUnaligned;
// }
//
// VOID
// FASTCALL
// KiEnterV86Mode(IN ULONG_PTR StackFrameUnaligned)
// {
//     PKTHREAD Thread;
//     PKV8086_STACK_FRAME StackFrame = (PKV8086_STACK_FRAME)(ROUND_UP(StackFrameUnaligned - 4, 16) + 4);
//     PKTRAP_FRAME TrapFrame = &StackFrame->TrapFrame;
//     PKV86_FRAME V86Frame = &StackFrame->V86Frame;
//     PFX_SAVE_AREA NpxFrame = &StackFrame->NpxArea;
//
//     ASSERT((ULONG_PTR)NpxFrame % 16 == 0);
//
//     /* Build fake user-mode trap frame */
//     TrapFrame->SegCs = KGDT_R0_CODE | RPL_MASK;
//     TrapFrame->SegEs = TrapFrame->SegDs = TrapFrame->SegFs = TrapFrame->SegGs = 0;
//     TrapFrame->ErrCode = 0;
//
//     /* Get the current thread's initial stack */
//     Thread = KeGetCurrentThread();
//     V86Frame->ThreadStack = KiGetThreadNpxArea(Thread);
//
//     /* Save TEB addresses */
//     V86Frame->ThreadTeb = Thread->Teb;
//     V86Frame->PcrTeb = KeGetPcr()->NtTib.Self;
//
//     /* Save return EIP */
//     TrapFrame->Eip = (ULONG_PTR)Ki386BiosCallReturnAddress;
//
//     /* Save our stack (after the frames) */
//     TrapFrame->Esi = StackFrameUnaligned;
//     TrapFrame->Edi = (ULONG_PTR)_AddressOfReturnAddress() + 4;
//
//     /* Sanitize EFlags and enable interrupts */
//     TrapFrame->EFlags = __readeflags() & 0x60DD7;
//     TrapFrame->EFlags |= EFLAGS_INTERRUPT_MASK;
//
//     /* Fill out the rest of the frame */
//     TrapFrame->HardwareSegSs = KGDT_R3_DATA | RPL_MASK;
//     TrapFrame->HardwareEsp = 0x11FFE;
//     TrapFrame->ExceptionList = EXCEPTION_CHAIN_END;
//     TrapFrame->Dr7 = 0;
//
//     /* Set some debug fields if trap debugging is enabled */
//     KiFillTrapFrameDebug(TrapFrame);
//
//     /* Disable interrupts */
//     _disable();
//
//     /* Copy the thread's NPX frame */
//     RtlCopyMemory(NpxFrame, V86Frame->ThreadStack, sizeof(FX_SAVE_AREA));
//
//     /* Clear exception list */
//     KeGetPcr()->NtTib.ExceptionList = EXCEPTION_CHAIN_END;
//
//     /* Set new ESP0 */
//     KeGetPcr()->TSS->Esp0 = (ULONG_PTR)&TrapFrame->V86Es;
//
//     /* Set new initial stack */
//     Thread->InitialStack = V86Frame;
//
//     /* Set VDM TEB */
//     Thread->Teb = (PTEB)TRAMPOLINE_TEB;
//     KiSetTebBase(KeGetPcr(), (PVOID)TRAMPOLINE_TEB);
//
//     /* Enable interrupts */
//     _enable();
//
//     /* Start VDM execution */
//     NtVdmControl(VdmStartExecution, NULL);
//
//     /* Exit to V86 mode */
//     KiEoiHelper(TrapFrame);
// }
//
// VOID
// NTAPI
// Ke386SetIOPL(VOID)
// {
//
//     PKTHREAD Thread = KeGetCurrentThread();
//     PKPROCESS Process = Thread->ApcState.Process;
//     PKTRAP_FRAME TrapFrame;
//     CONTEXT Context;
//
//     /* IOPL was enabled for this process/thread */
//     Process->Iopl = TRUE;
//     Thread->Iopl = TRUE;
//
//     /* Get the trap frame on exit */
//     TrapFrame = KeGetTrapFrame(Thread);
//
//     /* Convert to a context */
//     Context.ContextFlags = CONTEXT_CONTROL;
//     KeTrapFrameToContext(TrapFrame, NULL, &Context);
//
//     /* Set the IOPL flag */
//     Context.EFlags |= EFLAGS_IOPL;
//
//     /* Convert back to a trap frame */
//     KeContextToTrapFrame(&Context, NULL, TrapFrame, CONTEXT_CONTROL, UserMode);
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// Ke386CallBios(IN ULONG Int,
//               OUT PCONTEXT Context)
// {
//     PUCHAR Trampoline = (PUCHAR)TRAMPOLINE_BASE;
//     PTEB VdmTeb = (PTEB)TRAMPOLINE_TEB;
//     PVDM_TIB VdmTib = (PVDM_TIB)TRAMPOLINE_TIB;
//     ULONG ContextSize = FIELD_OFFSET(CONTEXT, ExtendedRegisters);
//     PKTHREAD Thread = KeGetCurrentThread();
//     PKTSS Tss = KeGetPcr()->TSS;
//     PKPROCESS Process = Thread->ApcState.Process;
//     PVDM_PROCESS_OBJECTS VdmProcessObjects;
//     USHORT OldOffset, OldBase;
//
//     /* Start with a clean TEB */
//     RtlZeroMemory(VdmTeb, sizeof(TEB));
//
//     /* Write the interrupt and bop */
//     *Trampoline++ = 0xCD;
//     *Trampoline++ = (UCHAR)Int;
//     *(PULONG)Trampoline = TRAMPOLINE_BOP;
//
//     /* Setup the VDM TEB and TIB */
//     VdmTeb->Vdm = (PVOID)TRAMPOLINE_TIB;
//     RtlZeroMemory(VdmTib, sizeof(VDM_TIB));
//     VdmTib->Size = sizeof(VDM_TIB);
//
//     /* Set a blank VDM state */
//     *VdmState = 0;
//
//     /* Copy the context */
//     RtlCopyMemory(&VdmTib->VdmContext, Context, ContextSize);
//     VdmTib->VdmContext.SegCs = (ULONG_PTR)Trampoline >> 4;
//     VdmTib->VdmContext.SegSs = (ULONG_PTR)Trampoline >> 4;
//     VdmTib->VdmContext.Eip = 0;
//     VdmTib->VdmContext.Esp = 2 * PAGE_SIZE - sizeof(ULONG_PTR);
//     VdmTib->VdmContext.EFlags |= EFLAGS_V86_MASK | EFLAGS_INTERRUPT_MASK;
//     VdmTib->VdmContext.ContextFlags = CONTEXT_FULL;
//
//     /* This can't be a real VDM process */
//     ASSERT(PsGetCurrentProcess()->VdmObjects == NULL);
//
//     /* Allocate VDM structure */
//     VdmProcessObjects = ExAllocatePoolWithTag(NonPagedPool,
//                                               sizeof(VDM_PROCESS_OBJECTS),
//                                               TAG_KERNEL);
//     if (!VdmProcessObjects) return STATUS_NO_MEMORY;
//
//     /* Set it up */
//     RtlZeroMemory(VdmProcessObjects, sizeof(VDM_PROCESS_OBJECTS));
//     VdmProcessObjects->VdmTib = VdmTib;
//     PsGetCurrentProcess()->VdmObjects = VdmProcessObjects;
//
//     /* Set the system affinity for the current thread */
//     KeSetSystemAffinityThread(1);
//
//     /* Make sure there's space for two IOPMs, then copy & clear the current */
//     ASSERT(((PKIPCR)KeGetPcr())->GDT[KGDT_TSS / 8].LimitLow >=
//             (0x2000 + IOPM_OFFSET - 1));
//     RtlCopyMemory(Ki386IopmSaveArea, &Tss->IoMaps[0].IoMap, IOPM_SIZE);
//     RtlZeroMemory(&Tss->IoMaps[0].IoMap, IOPM_SIZE);
//
//     /* Save the old offset and base, and set the new ones */
//     OldOffset = Process->IopmOffset;
//     OldBase = Tss->IoMapBase;
//     Process->IopmOffset = (USHORT)IOPM_OFFSET;
//     Tss->IoMapBase = (USHORT)IOPM_OFFSET;
//
//     /* Switch stacks and work the magic */
//     Ki386SetupAndExitToV86Mode(VdmTeb);
//
//     /* Restore IOPM */
//     RtlCopyMemory(&Tss->IoMaps[0].IoMap, Ki386IopmSaveArea, IOPM_SIZE);
//     Process->IopmOffset = OldOffset;
//     Tss->IoMapBase = OldBase;
//
//     /* Restore affinity */
//     KeRevertToUserAffinityThread();
//
//     /* Restore context */
//     RtlCopyMemory(Context, &VdmTib->VdmContext, ContextSize);
//     Context->ContextFlags = CONTEXT_FULL;
//
//     /* Free VDM objects */
//     ExFreePoolWithTag(PsGetCurrentProcess()->VdmObjects, TAG_KERNEL);
//     PsGetCurrentProcess()->VdmObjects = NULL;
//
//     /* Return status */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// Ke386IoSetAccessProcess(IN PKPROCESS Process,
//                         IN ULONG MapNumber)
// {
//     USHORT MapOffset;
//     PKPRCB Prcb;
//     KAFFINITY TargetProcessors;
//
//     if(MapNumber > IOPM_COUNT)
//         return FALSE;
//
//     MapOffset = KiComputeIopmOffset(MapNumber);
//
//     Process->IopmOffset = MapOffset;
//
//     TargetProcessors = Process->ActiveProcessors;
//     Prcb = KeGetCurrentPrcb();
//     if (TargetProcessors & Prcb->SetMember)
//         KeGetPcr()->TSS->IoMapBase = MapOffset;
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// Ke386SetIoAccessMap(IN ULONG MapNumber,
//                     IN PKIO_ACCESS_MAP IopmBuffer)
// {
//     PKPROCESS CurrentProcess;
//     PKPRCB Prcb;
//     PVOID pt;
//
//     if ((MapNumber > IOPM_COUNT) || (MapNumber == IO_ACCESS_MAP_NONE))
//         return FALSE;
//
//     Prcb = KeGetCurrentPrcb();
//
//     // Copy the IOP map and load the map for the current process.
//     pt = &(KeGetPcr()->TSS->IoMaps[MapNumber-1].IoMap);
//     RtlMoveMemory(pt, (PVOID)IopmBuffer, IOPM_SIZE);
//     CurrentProcess = Prcb->CurrentThread->ApcState.Process;
//     KeGetPcr()->TSS->IoMapBase = CurrentProcess->IopmOffset;
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// Ke386QueryIoAccessMap(IN ULONG MapNumber,
//                       IN PKIO_ACCESS_MAP IopmBuffer)
// {
//     ULONG i;
//     PVOID Map;
//     PUCHAR p;
//
//     if (MapNumber > IOPM_COUNT)
//         return FALSE;
//
//     if (MapNumber == IO_ACCESS_MAP_NONE)
//     {
//         // no access, simply return a map of all 1s
//         p = (PUCHAR)IopmBuffer;
//         for (i = 0; i < IOPM_SIZE; i++) {
//             p[i] = (UCHAR)-1;
//         }
//     }
//     else
//     {
//         // copy the bits
//         Map = (PVOID)&(KeGetPcr()->TSS->IoMaps[MapNumber-1].IoMap);
//         RtlMoveMemory((PVOID)IopmBuffer, Map, IOPM_SIZE);
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
