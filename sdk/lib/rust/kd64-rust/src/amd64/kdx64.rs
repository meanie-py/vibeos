//! Translated from `ntoskrnl/kd64/amd64/kdx64.c` — KD support routines for AMD64.

use ntkrnl_sys::ddk::*;

// ke globals referenced across the crate boundary as link symbols
// (all #[no_mangle] in ntoskrnl-rust ke/*). crate-split Phase 1b.
extern "C" {
    static mut KeNumberProcessors: core::ffi::c_char;
    static mut KiProcessorBlock: [ntkrnl_sys::ke::PKPRCB; ntkrnl_sys::constants::MAXIMUM_PROCESSORS as usize];
}

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001_u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000D_u32 as NTSTATUS;
const STATUS_DATATYPE_MISALIGNMENT: NTSTATUS = 0x80000002_u32 as NTSTATUS;
const STATUS_ACCESS_DENIED: NTSTATUS = 0xC0000022_u32 as NTSTATUS;

const EFLAGS_TF: ULONG = 0x100;
const REPORT_INCLUDES_SEGS: USHORT = 0x0001;
const REPORT_STANDARD_CS: USHORT = 0x0002;
const KGDT64_R0_CODE: USHORT = 0x0010;

// AMD64-specific control space classes (windbgkd.h).
const AMD64_DEBUG_CONTROL_SPACE_KPCR: ULONG64 = 0;
const AMD64_DEBUG_CONTROL_SPACE_KPRCB: ULONG64 = 1;
const AMD64_DEBUG_CONTROL_SPACE_KSPECIAL: ULONG64 = 2;
const AMD64_DEBUG_CONTROL_SPACE_KTHREAD: ULONG64 = 3;

const Isa: INTERFACE_TYPE = _INTERFACE_TYPE_Isa as INTERFACE_TYPE;

extern "C" {
    static mut KdpCurrentSymbolStart: ULONG_PTR;
    static mut KdpCurrentSymbolEnd: ULONG_PTR;
    fn __readmsr(msr: ULONG) -> u64;
    fn __writemsr(msr: ULONG, value: u64);
}

#[inline]
unsafe fn read_port_uchar(port: USHORT) -> UCHAR {
    let value: UCHAR;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nomem, nostack, preserves_flags)
    );
    value
}

#[inline]
unsafe fn read_port_ushort(port: USHORT) -> USHORT {
    let value: USHORT;
    core::arch::asm!(
        "in ax, dx",
        in("dx") port,
        out("ax") value,
        options(nomem, nostack, preserves_flags)
    );
    value
}

#[inline]
unsafe fn read_port_ulong(port: USHORT) -> ULONG {
    let value: ULONG;
    core::arch::asm!(
        "in eax, dx",
        in("dx") port,
        out("eax") value,
        options(nomem, nostack, preserves_flags)
    );
    value
}

#[inline]
unsafe fn write_port_uchar(port: USHORT, value: UCHAR) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline]
unsafe fn write_port_ushort(port: USHORT, value: USHORT) {
    core::arch::asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") value,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline]
unsafe fn write_port_ulong(port: USHORT, value: ULONG) {
    core::arch::asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") value,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline]
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

#[no_mangle]
pub unsafe extern "system" fn KdpGetStateChange(
    State: PDBGKD_MANIPULATE_STATE64,
    Context: PCONTEXT,
) {
    let continue2 = &mut (*State).u.Continue2;
    let control = &mut continue2.__bindgen_anon_1.ControlSet;

    if nt_success(continue2.ContinueStatus) {
        if control.TraceFlag != 0 {
            (*Context).EFlags |= EFLAGS_TF;
        } else {
            (*Context).EFlags &= !EFLAGS_TF;
        }

        for i in 0..(KeNumberProcessors as usize) {
            let prcb = *(*(&raw const KiProcessorBlock)).as_ptr().add(i);
            if !prcb.is_null() {
                (*prcb).ProcessorState.SpecialRegisters.KernelDr7 = control.Dr7;
                (*prcb).ProcessorState.SpecialRegisters.KernelDr6 = 0;
            }
        }

        if control.CurrentSymbolStart != 1 {
            KdpCurrentSymbolStart = control.CurrentSymbolStart as ULONG_PTR;
            KdpCurrentSymbolEnd = control.CurrentSymbolEnd as ULONG_PTR;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpSetContextState(
    WaitStateChange: PDBGKD_ANY_WAIT_STATE_CHANGE,
    Context: PCONTEXT,
) {
    let prcb = ntkrnl_sys::inline::ke::KeGetCurrentPrcb();
    let report = &mut (*WaitStateChange).__bindgen_anon_1.ControlReport;

    report.Dr6 = (*prcb).ProcessorState.SpecialRegisters.KernelDr6;
    report.Dr7 = (*prcb).ProcessorState.SpecialRegisters.KernelDr7;

    report.SegCs = (*Context).SegCs as USHORT;
    report.SegDs = (*Context).SegDs as USHORT;
    report.SegEs = (*Context).SegEs as USHORT;
    report.SegFs = (*Context).SegFs as USHORT;

    report.EFlags = (*Context).EFlags;

    report.ReportFlags = REPORT_INCLUDES_SEGS;
    if report.SegCs == KGDT64_R0_CODE {
        report.ReportFlags |= REPORT_STANDARD_CS;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysReadMsr(Msr: ULONG, MsrValue: *mut u64) -> NTSTATUS {
    // SEH wrapper in C is dead under DUMMY_PSEH; #GP from a bad MSR will trap.
    *MsrValue = __readmsr(Msr);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysWriteMsr(Msr: ULONG, MsrValue: *const u64) -> NTSTATUS {
    __writemsr(Msr, *MsrValue);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysReadBusData(
    _BusDataType: BUS_DATA_TYPE,
    _BusNumber: ULONG,
    _SlotNumber: ULONG,
    _Offset: ULONG,
    _Buffer: PVOID,
    _Length: ULONG,
    _ActualLength: *mut ULONG,
) -> NTSTATUS {
    // UNIMPLEMENTED — matches C version stub.
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysWriteBusData(
    _BusDataType: BUS_DATA_TYPE,
    _BusNumber: ULONG,
    _SlotNumber: ULONG,
    _Offset: ULONG,
    _Buffer: PVOID,
    _Length: ULONG,
    _ActualLength: *mut ULONG,
) -> NTSTATUS {
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysReadControlSpace(
    Processor: ULONG,
    BaseAddress: ULONG64,
    Buffer: PVOID,
    Length: ULONG,
    ActualLength: *mut ULONG,
) -> NTSTATUS {
    let prcb = *(*(&raw const KiProcessorBlock)).as_ptr().add(Processor as usize);
    // CONTAINING_RECORD(Prcb, KIPCR, Prcb): subtract offset of KIPCR.Prcb to recover the KIPCR pointer.
    let pcr_offset = core::mem::offset_of!(_KIPCR, Prcb);
    let pcr = ((prcb as usize).wrapping_sub(pcr_offset)) as *mut _KIPCR;

    // Locals hold the values we expose; ControlStart points to the appropriate one.
    let pcr_local: *mut _KIPCR = pcr;
    let prcb_local: PKPRCB = prcb;
    let thread_local: PKTHREAD = (*prcb).CurrentThread;

    let control_start: PVOID;
    let copy_len: ULONG;

    match BaseAddress {
        AMD64_DEBUG_CONTROL_SPACE_KPCR => {
            control_start = core::ptr::addr_of!(pcr_local) as PVOID;
            copy_len = core::mem::size_of::<PVOID>() as ULONG;
        }
        AMD64_DEBUG_CONTROL_SPACE_KPRCB => {
            control_start = core::ptr::addr_of!(prcb_local) as PVOID;
            copy_len = core::mem::size_of::<PVOID>() as ULONG;
        }
        AMD64_DEBUG_CONTROL_SPACE_KSPECIAL => {
            control_start = core::ptr::addr_of_mut!((*prcb).ProcessorState.SpecialRegisters) as PVOID;
            copy_len = core::mem::size_of::<KSPECIAL_REGISTERS>() as ULONG;
        }
        AMD64_DEBUG_CONTROL_SPACE_KTHREAD => {
            control_start = core::ptr::addr_of!(thread_local) as PVOID;
            copy_len = core::mem::size_of::<PVOID>() as ULONG;
        }
        _ => {
            *ActualLength = 0;
            return STATUS_UNSUCCESSFUL;
        }
    }
    *ActualLength = copy_len;

    let n = if Length < copy_len { Length } else { copy_len };
    core::ptr::copy_nonoverlapping(control_start as *const u8, Buffer as *mut u8, n as usize);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysWriteControlSpace(
    Processor: ULONG,
    BaseAddress: ULONG64,
    Buffer: PVOID,
    Length: ULONG,
    ActualLength: *mut ULONG,
) -> NTSTATUS {
    let prcb = *(*(&raw const KiProcessorBlock)).as_ptr().add(Processor as usize);

    let control_start: PVOID;
    let copy_len: ULONG;

    match BaseAddress {
        AMD64_DEBUG_CONTROL_SPACE_KSPECIAL => {
            control_start = core::ptr::addr_of_mut!((*prcb).ProcessorState.SpecialRegisters) as PVOID;
            copy_len = core::mem::size_of::<KSPECIAL_REGISTERS>() as ULONG;
        }
        _ => {
            *ActualLength = 0;
            return STATUS_UNSUCCESSFUL;
        }
    }
    *ActualLength = copy_len;

    let n = if Length < copy_len { Length } else { copy_len };
    core::ptr::copy_nonoverlapping(Buffer as *const u8, control_start as *mut u8, n as usize);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysReadIoSpace(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
    AddressSpace: ULONG,
    IoAddress: ULONG64,
    DataValue: PVOID,
    DataSize: ULONG,
    ActualDataSize: *mut ULONG,
) -> NTSTATUS {
    if InterfaceType != Isa || BusNumber != 0 || AddressSpace != 1 {
        *ActualDataSize = 0;
        return STATUS_INVALID_PARAMETER;
    }

    if (IoAddress & ((DataSize as ULONG64) - 1)) != 0 {
        *ActualDataSize = 0;
        return STATUS_DATATYPE_MISALIGNMENT;
    }

    let port = IoAddress as USHORT;
    match DataSize {
        1 => *(DataValue as *mut UCHAR) = read_port_uchar(port),
        2 => *(DataValue as *mut USHORT) = read_port_ushort(port),
        4 => *(DataValue as *mut ULONG) = read_port_ulong(port),
        _ => {
            *ActualDataSize = 0;
            return STATUS_INVALID_PARAMETER;
        }
    }

    *ActualDataSize = DataSize;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysWriteIoSpace(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
    AddressSpace: ULONG,
    IoAddress: ULONG64,
    DataValue: PVOID,
    DataSize: ULONG,
    ActualDataSize: *mut ULONG,
) -> NTSTATUS {
    if InterfaceType != Isa || BusNumber != 0 || AddressSpace != 1 {
        *ActualDataSize = 0;
        return STATUS_INVALID_PARAMETER;
    }

    if (IoAddress & ((DataSize as ULONG64) - 1)) != 0 {
        *ActualDataSize = 0;
        return STATUS_DATATYPE_MISALIGNMENT;
    }

    let port = IoAddress as USHORT;
    match DataSize {
        1 => write_port_uchar(port, *(DataValue as *const UCHAR)),
        2 => write_port_ushort(port, *(DataValue as *const USHORT)),
        4 => write_port_ulong(port, *(DataValue as *const ULONG)),
        _ => {
            *ActualDataSize = 0;
            return STATUS_INVALID_PARAMETER;
        }
    }

    *ActualDataSize = DataSize;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysCheckLowMemory(_Flags: ULONG) -> NTSTATUS {
    // UNIMPLEMENTED — matches C version stub.
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn KdpAllowDisable() -> NTSTATUS {
    for i in 0..(KeNumberProcessors as usize) {
        let prcb = *(*(&raw const KiProcessorBlock)).as_ptr().add(i);
        if !prcb.is_null() {
            let dr7 = (*prcb).ProcessorState.SpecialRegisters.KernelDr7;
            if (dr7 & 0xFF) != 0 {
                return STATUS_ACCESS_DENIED;
            }
        }
    }
    STATUS_SUCCESS
}
