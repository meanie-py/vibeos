use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::STATUS_UNSUCCESSFUL;
use ntkrnl_sys::ddk::KeStallExecutionProcessor;
use crate::i8042prt_h::*;

// ntstatus.h: STATUS_IO_TIMEOUT — not currently in ntkrnl-sys constants.
const STATUS_IO_TIMEOUT: NTSTATUS = 0xC00000B5u32 as NTSTATUS;

// READ_PORT_UCHAR / WRITE_PORT_UCHAR are HAL port-I/O intrinsics, not exported
// symbols: C lowers them to __inbyte/__outbyte (ddk/ioaccess.h). Provide them
// inline. On x86 they are real `in`/`out al, dx`; on arm64 (no port I/O) the C
// path is MMIO READ/WRITE_REGISTER_UCHAR, so fall back to a volatile byte
// access at the mapped address.
#[inline]
unsafe fn READ_PORT_UCHAR(Port: PUCHAR) -> UCHAR {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let v: UCHAR;
        core::arch::asm!("in al, dx", in("dx") Port as usize as u16, out("al") v,
            options(nostack, preserves_flags));
        v
    }
    #[cfg(target_arch = "aarch64")]
    {
        core::ptr::read_volatile(Port)
    }
}
#[inline]
unsafe fn WRITE_PORT_UCHAR(Port: PUCHAR, Value: UCHAR) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        core::arch::asm!("out dx, al", in("dx") Port as usize as u16, in("al") Value,
            options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "aarch64")]
    {
        core::ptr::write_volatile(Port, Value);
    }
}

// i8042ReadKeyboardData(De, Data) == i8042ReadData(De, KBD_OBF, Data) (macro in
// i8042prt.h). i8042ReadDataWait calls it; provide it locally so the macro
// semantics are preserved without an unresolved extern.
#[inline]
pub(crate) unsafe fn i8042ReadKeyboardData(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    Data: PUCHAR,
) -> NTSTATUS {
    i8042ReadData(DeviceExtension, KBD_OBF, Data)
}

// i8042ReadMouseData(De, Data) == i8042ReadData(De, MOU_OBF, Data) (macro in
// i8042prt.h). Mirror of i8042ReadKeyboardData above for the mouse OBF bit.
#[inline]
pub(crate) unsafe fn i8042ReadMouseData(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    Data: PUCHAR,
) -> NTSTATUS {
    i8042ReadData(DeviceExtension, MOU_OBF, Data)
}

pub(crate) unsafe fn i8042Flush(DeviceExtension: PPORT_DEVICE_EXTENSION) {
    let mut Ignore: UCHAR = 0;

    while NT_SUCCESS(i8042ReadData(DeviceExtension, KBD_OBF, &mut Ignore)) {
        KeStallExecutionProcessor(50);
    }

    while NT_SUCCESS(i8042ReadData(DeviceExtension, KBD_IBF, &mut Ignore)) {
        KeStallExecutionProcessor(50);
    }
}

pub(crate) unsafe fn i8042IsrWritePort(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    Value: UCHAR,
    SelectCmd: UCHAR,
) -> BOOLEAN {
    // Fail-loud on dropped controller writes (2026-07-06 PS/2 triage kept):
    // i8042Write drops writes SILENTLY on IBF timeout and every ISR-side
    // caller ignores the return — a dropped byte permanently stalls the mouse
    // reset state machine, so make it visible.
    if SelectCmd != 0 {
        if i8042Write(DeviceExtension, (*DeviceExtension).ControlPort, SelectCmd) == 0 {
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[I8042PRT] IsrWritePort sel=0x%x DROPPED (IBF stuck)\n\0".as_ptr()
                    as *const i8,
                SelectCmd as u32,
            );
            return 0u8;
        }
    }

    let ok = i8042Write(DeviceExtension, (*DeviceExtension).DataPort, Value);
    if ok == 0 {
        ntkrnl_sys::driver_ddk::DbgPrint(
            b"[I8042PRT] IsrWritePort val=0x%x DROPPED (IBF stuck)\n\0".as_ptr() as *const i8,
            Value as u32,
        );
    }
    ok
}

pub(crate) unsafe fn i8042ReadData(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    StatusFlags: UCHAR,
    Data: PUCHAR,
) -> NTSTATUS {
    let mut PortStatus: UCHAR = 0;
    let Status: NTSTATUS;

    Status = i8042ReadStatus(DeviceExtension, &mut PortStatus);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if PortStatus & StatusFlags != 0 {
        *Data = READ_PORT_UCHAR((*DeviceExtension).DataPort);

        if (PortStatus & KBD_PERR) == 0 {
            return STATUS_SUCCESS;
        }
    }
    STATUS_UNSUCCESSFUL
}

pub(crate) unsafe fn i8042ReadStatus(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    Status: PUCHAR,
) -> NTSTATUS {
    *Status = READ_PORT_UCHAR((*DeviceExtension).ControlPort);
    STATUS_SUCCESS
}

// C: `NTSTATUS i8042ReadDataWait(...)` — plain (non-NTAPI) function in
// i8042prt.h, called from sibling .c modules during cohabitation. Export as
// cdecl (`extern "C"`) so the i386 symbol is `_i8042ReadDataWait` (NTAPI would
// give the stdcall-decorated `_i8042ReadDataWait@8` that C callers don't expect).
#[no_mangle]
pub unsafe extern "C" fn i8042ReadDataWait(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    Data: PUCHAR,
) -> NTSTATUS {
    let mut Counter: ULONG;
    let mut Status: NTSTATUS;

    Counter = (*DeviceExtension).Settings.PollingIterations;

    while Counter != 0 {
        Counter -= 1;

        Status = i8042ReadKeyboardData(DeviceExtension, Data);

        if NT_SUCCESS(Status) {
            return Status;
        }

        KeStallExecutionProcessor(50);
    }

    /* Timed out */
    STATUS_IO_TIMEOUT
}

#[no_mangle]
pub unsafe extern "system" fn i8042SynchReadPort(
    Context: PVOID,
    Value: PUCHAR,
    WaitForAck: BOOLEAN,
) -> NTSTATUS {
    let DeviceExtension: PPORT_DEVICE_EXTENSION;

    let _ = WaitForAck;

    DeviceExtension = Context as PPORT_DEVICE_EXTENSION;

    i8042ReadDataWait(DeviceExtension, Value)
}

#[no_mangle]
pub unsafe extern "system" fn i8042SynchWritePort(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    Port: UCHAR,
    Value: UCHAR,
    WaitForAck: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Ack: UCHAR = 0;
    let mut ResendIterations: ULONG;

    ResendIterations = (*DeviceExtension).Settings.ResendIterations + 1;

    loop {
        if Port != 0 {
            if i8042Write(DeviceExtension, (*DeviceExtension).DataPort, Port) == 0 {
                return STATUS_IO_TIMEOUT;
            }
        }

        if i8042Write(DeviceExtension, (*DeviceExtension).DataPort, Value) == 0 {
            return STATUS_IO_TIMEOUT;
        }

        if WaitForAck != 0 {
            Status = i8042ReadDataWait(DeviceExtension, core::ptr::addr_of_mut!(Ack));
            if !NT_SUCCESS(Status) {
                return Status;
            }
            if Ack == KBD_ACK {
                return STATUS_SUCCESS;
            } else if Ack == KBD_RESEND {
            }
        } else {
            return STATUS_SUCCESS;
        }

        ResendIterations -= 1;
        if ResendIterations == 0 {
            break;
        }
    }

    STATUS_IO_TIMEOUT
}

pub(crate) unsafe fn i8042Write(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    addr: PUCHAR,
    data: UCHAR,
) -> BOOLEAN {
    let mut Counter: ULONG;

    Counter = (*DeviceExtension).Settings.PollingIterations;

    while (KBD_IBF & READ_PORT_UCHAR((*DeviceExtension).ControlPort)) != 0 && {
        let old = Counter;
        Counter = Counter.wrapping_sub(1);
        old != 0
    } {
        KeStallExecutionProcessor(50);
    }

    if Counter != 0 {
        WRITE_PORT_UCHAR(addr, data);
        return 1u8;
    }
    0u8
}
