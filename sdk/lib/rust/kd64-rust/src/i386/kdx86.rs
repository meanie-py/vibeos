//! Translated from `ntoskrnl/kd64/i386/kdx86.c` — KD support routines for x86.

use ntkrnl_sys::ddk::*;

// ke globals referenced across the crate boundary as link symbols
// (all #[no_mangle] in ntoskrnl-rust ke/*). crate-split Phase 1b.
extern "C" {
    static mut KeNumberProcessors: core::ffi::c_char;
    static mut KiProcessorBlock: [ntkrnl_sys::ke::PKPRCB; ntkrnl_sys::constants::MAXIMUM_PROCESSORS as usize];
}

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001_u32 as NTSTATUS;
const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000E_u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000D_u32 as NTSTATUS;
const STATUS_DATATYPE_MISALIGNMENT: NTSTATUS = 0x80000002_u32 as NTSTATUS;
const STATUS_ACCESS_DENIED: NTSTATUS = 0xC0000022_u32 as NTSTATUS;

const EFLAGS_TF: ULONG = 0x100;
const REPORT_INCLUDES_SEGS: USHORT = 0x0001;
const REPORT_STANDARD_CS: USHORT = 0x0002;
const KGDT_R0_CODE: USHORT = 0x08;

// mm.h's encoding, the one MmDbgCopyMemory reads (see kdapi.rs).
use ntkrnl_sys::mm::{MMDBG_COPY_UNSAFE, MMDBG_COPY_WRITE};

const Isa: INTERFACE_TYPE = _INTERFACE_TYPE_Isa as INTERFACE_TYPE;

extern "system" {
    fn HalGetBusDataByOffset(
        BusDataType: BUS_DATA_TYPE,
        BusNumber: ULONG,
        SlotNumber: ULONG,
        Buffer: PVOID,
        Offset: ULONG,
        Length: ULONG,
    ) -> ULONG;
    fn HalSetBusDataByOffset(
        BusDataType: BUS_DATA_TYPE,
        BusNumber: ULONG,
        SlotNumber: ULONG,
        Buffer: PVOID,
        Offset: ULONG,
        Length: ULONG,
    ) -> ULONG;
    fn KdpCopyMemoryChunks(
        Address: u64,
        Buffer: PVOID,
        Size: ULONG,
        ChunkSize: ULONG,
        Flags: ULONG,
        ActualSize: *mut ULONG,
    ) -> NTSTATUS;
}

extern "C" {
    static mut KdpCurrentSymbolStart: ULONG_PTR;
    static mut KdpCurrentSymbolEnd: ULONG_PTR;
}

#[inline]
unsafe fn read_msr(msr: ULONG) -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") lo,
        out("edx") hi,
        options(nomem, nostack, preserves_flags),
    );
    ((hi as u64) << 32) | (lo as u64)
}

#[inline]
unsafe fn write_msr(msr: ULONG, value: u64) {
    let lo = value as u32;
    let hi = (value >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") lo,
        in("edx") hi,
        options(nomem, nostack, preserves_flags),
    );
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
    if report.SegCs == KGDT_R0_CODE {
        report.ReportFlags |= REPORT_STANDARD_CS;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysReadMsr(Msr: ULONG, MsrValue: *mut u64) -> NTSTATUS {
    // SEH wrapper in C is dead under DUMMY_PSEH; #GP from a bad MSR will trap.
    // Only invoked by the host debugger via WinDbg packet, never during boot.
    *MsrValue = read_msr(Msr);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysWriteMsr(Msr: ULONG, MsrValue: *const u64) -> NTSTATUS {
    write_msr(Msr, *MsrValue);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysReadBusData(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Offset: ULONG,
    Buffer: PVOID,
    Length: ULONG,
    ActualLength: *mut ULONG,
) -> NTSTATUS {
    *ActualLength = HalGetBusDataByOffset(BusDataType, BusNumber, SlotNumber, Buffer, Offset, Length);
    if *ActualLength != 0 { STATUS_SUCCESS } else { STATUS_UNSUCCESSFUL }
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysWriteBusData(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Offset: ULONG,
    Buffer: PVOID,
    Length: ULONG,
    ActualLength: *mut ULONG,
) -> NTSTATUS {
    *ActualLength = HalSetBusDataByOffset(BusDataType, BusNumber, SlotNumber, Buffer, Offset, Length);
    if *ActualLength != 0 { STATUS_SUCCESS } else { STATUS_UNSUCCESSFUL }
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysReadControlSpace(
    Processor: ULONG,
    BaseAddress: ULONG64,
    Buffer: PVOID,
    Length: ULONG,
    ActualLength: *mut ULONG,
) -> NTSTATUS {
    let proc_state_size = core::mem::size_of::<KPROCESSOR_STATE>() as ULONG64;
    if BaseAddress < proc_state_size && Processor < (KeNumberProcessors as ULONG) {
        let real_length = (proc_state_size - BaseAddress) as ULONG;
        let length = if real_length < Length { real_length } else { Length };

        let prcb = *(*(&raw const KiProcessorBlock)).as_ptr().add(Processor as usize);
        let control_start = (BaseAddress as usize
            + core::ptr::addr_of_mut!((*prcb).ProcessorState) as usize) as PVOID;

        KdpCopyMemoryChunks(
            Buffer as u64,
            control_start,
            length,
            0,
            MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
            ActualLength,
        )
    } else {
        *ActualLength = 0;
        STATUS_UNSUCCESSFUL
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysWriteControlSpace(
    Processor: ULONG,
    BaseAddress: ULONG64,
    Buffer: PVOID,
    Length: ULONG,
    ActualLength: *mut ULONG,
) -> NTSTATUS {
    let proc_state_size = core::mem::size_of::<KPROCESSOR_STATE>() as ULONG64;
    if (BaseAddress + Length as ULONG64) <= proc_state_size
        && Processor < (KeNumberProcessors as ULONG)
    {
        let prcb = *(*(&raw const KiProcessorBlock)).as_ptr().add(Processor as usize);
        let control_start = (BaseAddress as usize
            + core::ptr::addr_of_mut!((*prcb).ProcessorState) as usize) as PVOID;

        KdpCopyMemoryChunks(
            Buffer as u64,
            control_start,
            Length,
            0,
            MMDBG_COPY_UNSAFE,
            ActualLength,
        )
    } else {
        *ActualLength = 0;
        STATUS_UNSUCCESSFUL
    }
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
        return STATUS_UNSUCCESSFUL;
    }

    let port = IoAddress as USHORT;
    match DataSize {
        1 => {
            *(DataValue as *mut UCHAR) = read_port_uchar(port);
            *ActualDataSize = 1;
            STATUS_SUCCESS
        }
        2 => {
            if (IoAddress & 1) != 0 {
                *ActualDataSize = 0;
                return STATUS_DATATYPE_MISALIGNMENT;
            }
            *(DataValue as *mut USHORT) = read_port_ushort(port);
            *ActualDataSize = 2;
            STATUS_SUCCESS
        }
        4 => {
            if (IoAddress & 3) != 0 {
                *ActualDataSize = 0;
                return STATUS_DATATYPE_MISALIGNMENT;
            }
            *(DataValue as *mut ULONG) = read_port_ulong(port);
            *ActualDataSize = 4;
            STATUS_SUCCESS
        }
        _ => {
            *ActualDataSize = 0;
            STATUS_INVALID_PARAMETER
        }
    }
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
        return STATUS_UNSUCCESSFUL;
    }

    let port = IoAddress as USHORT;
    match DataSize {
        1 => {
            write_port_uchar(port, *(DataValue as *const UCHAR));
            *ActualDataSize = 1;
            STATUS_SUCCESS
        }
        2 => {
            if (IoAddress & 1) != 0 {
                *ActualDataSize = 0;
                return STATUS_DATATYPE_MISALIGNMENT;
            }
            write_port_ushort(port, *(DataValue as *const USHORT));
            *ActualDataSize = 2;
            STATUS_SUCCESS
        }
        4 => {
            if (IoAddress & 3) != 0 {
                *ActualDataSize = 0;
                return STATUS_DATATYPE_MISALIGNMENT;
            }
            write_port_ulong(port, *(DataValue as *const ULONG));
            *ActualDataSize = 4;
            STATUS_SUCCESS
        }
        _ => {
            *ActualDataSize = 0;
            STATUS_INVALID_PARAMETER
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpSysCheckLowMemory(_Flags: ULONG) -> NTSTATUS {
    // PAE not supported.
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

#[allow(dead_code)]
const _UNUSED_PLACEHOLDER: NTSTATUS = STATUS_NO_SUCH_DEVICE;
