use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{KeSynchronizeExecution, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
// IoGetCurrentIrpStackLocation, IoMarkIrpPending, IoSkipCurrentIrpStackLocation
//   → crate::glue (ntkrnl_sys::inline::tier1 — FORCEINLINE, not exported on i386)
// IoAcquireRemoveLockEx, IoReleaseRemoveLockEx, IoAcquireRemoveLock,
//   IoReleaseRemoveLock, READ_PORT_UCHAR, WRITE_PORT_UCHAR, ULongToPtr,
//   RtlAssert → crate::glue inline fns
use core::ffi::c_void;

extern "C" {
    fn DbgPrintEx( ComponentId: ULONG, Level: ULONG, Format: PCSTR, ... ) -> ULONG;
}

// ---- file-scope preamble (translated) ----
// (the only declaration is a function-like macro, which is skipped)

unsafe fn SerialGetUserBuffers(
    mut Irp: PIRP,
    mut IoControlCode: ULONG,
    mut BufferIn: *mut PVOID,
    mut BufferOut: *mut PVOID,
) {
    if Irp.is_null() {
        RtlAssert(
            b"Irp\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/serial/serial/devctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            23 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if BufferIn.is_null() {
        RtlAssert(
            b"BufferIn\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/serial/serial/devctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            24 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if BufferOut.is_null() {
        RtlAssert(
            b"BufferOut\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/serial/serial/devctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            25 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    match IoControlCode & 0x3 {
        METHOD_BUFFERED => {
            *BufferOut = (*Irp).AssociatedIrp.SystemBuffer;
            *BufferIn = *BufferOut;
        }
        METHOD_IN_DIRECT | METHOD_OUT_DIRECT => {
            *BufferIn = (*Irp).AssociatedIrp.SystemBuffer;
            *BufferOut =
                MmGetSystemAddressForMdlSafe((*Irp).MdlAddress, NormalPagePriority) as PVOID;
        }
        METHOD_NEITHER => {
            *BufferIn = (*IoGetCurrentIrpStackLocation(Irp))
                .Parameters
                .DeviceIoControl
                .Type3InputBuffer;
            *BufferOut = (*Irp).UserBuffer;
        }
        _ => {
            *BufferIn = core::ptr::null_mut();
            *BufferOut = core::ptr::null_mut();
        }
    };
}

#[no_mangle]
pub unsafe extern "system" fn SerialSetBaudRate(
    DeviceExtension: PSERIAL_DEVICE_EXTENSION,
    NewBaudRate: ULONG,
) -> NTSTATUS {
    const BAUD_CLOCK: core::ffi::c_int = 1843200;
    const CLOCKS_PER_BIT: core::ffi::c_int = 16;
    const SR_LCR_DLAB: core::ffi::c_int = 0x80;
    const DPFLTR_SERIAL_ID: core::ffi::c_int = 39;
    const DPFLTR_TRACE_LEVEL: core::ffi::c_uint = 2;

    let mut BaudRate: ULONG;
    let mut divisor: USHORT;
    let ComPortBase: PUCHAR =
        ULongToPtr((*DeviceExtension).BaseAddress as core::ffi::c_ulong) as PUCHAR;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if NewBaudRate == 0 as core::ffi::c_ulong {
        return STATUS_INVALID_PARAMETER;
    }

    divisor = (BAUD_CLOCK as ULONG)
        .wrapping_div((CLOCKS_PER_BIT as ULONG).wrapping_mul(NewBaudRate)) as USHORT;
    BaudRate = (BAUD_CLOCK / (CLOCKS_PER_BIT * divisor as core::ffi::c_int)) as ULONG;

    Status = IoAcquireRemoveLockEx(
        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
        ULongToPtr((*DeviceExtension).ComPort as core::ffi::c_ulong) as PVOID,
        b"drivers/serial/serial/devctrl.c\0"
            as *const u8 as PCSTR,
        65 as ULONG,
        core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    );
    if Status >= 0 as core::ffi::c_long {
        let Lcr: UCHAR;
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_TRACE_LEVEL as ULONG,
            b"(%s:%d) SerialSetBaudRate(COM%lu, %lu Bauds)\n\0" as *const u8 as PCSTR,
            b"drivers/serial/serial/devctrl.c\0"
                as *const u8 as *const core::ffi::c_char,
            69 as core::ffi::c_int,
            (*DeviceExtension).ComPort,
            BaudRate,
        );
        Lcr = READ_PORT_UCHAR(ComPortBase.offset(3 as core::ffi::c_int as isize));
        WRITE_PORT_UCHAR(
            ComPortBase.offset(3 as core::ffi::c_int as isize),
            (Lcr as core::ffi::c_int | (SR_LCR_DLAB as i32)) as UCHAR,
        );
        WRITE_PORT_UCHAR(
            ComPortBase.offset(0 as core::ffi::c_int as isize),
            (divisor as core::ffi::c_int & 0xff as core::ffi::c_int) as UCHAR,
        );
        WRITE_PORT_UCHAR(
            ComPortBase.offset(1 as core::ffi::c_int as isize),
            (divisor as core::ffi::c_int >> 8 as core::ffi::c_int) as UCHAR,
        );
        WRITE_PORT_UCHAR(ComPortBase.offset(3 as core::ffi::c_int as isize), Lcr);

        IoReleaseRemoveLockEx(
            core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
            ULongToPtr((*DeviceExtension).ComPort as core::ffi::c_ulong) as PVOID,
            core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
        );
    }

    if Status >= 0 as core::ffi::c_long {
        (*DeviceExtension).BaudRate = BaudRate;
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn SerialSetLineControl(
    DeviceExtension: PSERIAL_DEVICE_EXTENSION,
    NewSettings: PSERIAL_LINE_CONTROL,
) -> NTSTATUS {

    let mut ComPortBase: PUCHAR = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    let mut Lcr: UCHAR = 0 as UCHAR;
    let mut Status: NTSTATUS = 0;
    if DeviceExtension.is_null() {
        RtlAssert(
            b"DeviceExtension\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/serial/serial/devctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            96 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if NewSettings.is_null() {
        RtlAssert(
            b"NewSettings\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/serial/serial/devctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            97 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    DbgPrintEx(
        DPFLTR_SERIAL_ID as ::core::ffi::c_int as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) SerialSetLineControl(COM%lu, Settings { %lu %lu %lu })\n\0" as *const u8 as PCSTR,
        b"drivers/serial/serial/devctrl.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        100 as ::core::ffi::c_int,
        (*DeviceExtension).ComPort,
        (*NewSettings).StopBits as ::core::ffi::c_int,
        (*NewSettings).Parity as ::core::ffi::c_int,
        (*NewSettings).WordLength as ::core::ffi::c_int,
    );
    match (*NewSettings).WordLength as ::core::ffi::c_int {
        5 => {
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_CS5 as i32)) as UCHAR;
        }
        6 => {
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_CS6 as i32)) as UCHAR;
        }
        7 => {
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_CS7 as i32)) as UCHAR;
        }
        8 => {
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_CS8 as i32)) as UCHAR;
        }
        _ => return STATUS_INVALID_PARAMETER,
    }
    if ((*NewSettings).WordLength as ::core::ffi::c_int) < 5 as ::core::ffi::c_int
        || (*NewSettings).WordLength as ::core::ffi::c_int > 8 as ::core::ffi::c_int
    {
        return STATUS_INVALID_PARAMETER;
    }
    match (*NewSettings).Parity as ::core::ffi::c_int {
        x if x == NO_PARITY as i32 => {
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_PNO as i32)) as UCHAR;
        }
        x if x == ODD_PARITY as i32 => {
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_POD as i32)) as UCHAR;
        }
        x if x == EVEN_PARITY as i32 => {
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_PEV as i32)) as UCHAR;
        }
        x if x == MARK_PARITY as i32 => {
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_PMK as i32)) as UCHAR;
        }
        x if x == SPACE_PARITY as i32 => {
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_PSP as i32)) as UCHAR;
        }
        _ => return STATUS_INVALID_PARAMETER,
    }
    match (*NewSettings).StopBits as ::core::ffi::c_int {
        x if x == STOP_BIT_1 as i32 => {
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_ST1 as i32)) as UCHAR;
        }
        x if x == STOP_BITS_1_5 as i32 => {
            if (*NewSettings).WordLength as ::core::ffi::c_int != 5 as ::core::ffi::c_int {
                return STATUS_INVALID_PARAMETER;
            }
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_ST2 as i32)) as UCHAR;
        }
        x if x == STOP_BITS_2 as i32 => {
            if ((*NewSettings).WordLength as ::core::ffi::c_int) < 6 as ::core::ffi::c_int
                || (*NewSettings).WordLength as ::core::ffi::c_int > 8 as ::core::ffi::c_int
            {
                return STATUS_INVALID_PARAMETER;
            }
            Lcr = (Lcr as ::core::ffi::c_int | (SR_LCR_ST2 as i32)) as UCHAR;
        }
        _ => return STATUS_INVALID_PARAMETER,
    }
    ComPortBase = ULongToPtr((*DeviceExtension).BaseAddress as ::core::ffi::c_ulong) as PUCHAR;
    Status = IoAcquireRemoveLockEx(
        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
        ULongToPtr((*DeviceExtension).ComPort as ::core::ffi::c_ulong) as PVOID,
        b"drivers/serial/serial/devctrl.c\0"
            as *const u8 as PCSTR,
        146 as ULONG,
        ::core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    );
    if !(Status >= 0 as ::core::ffi::c_long) {
        return Status;
    }
    WRITE_PORT_UCHAR(ComPortBase.offset(3 as ::core::ffi::c_int as isize), Lcr);
    READ_PORT_UCHAR(ComPortBase.offset(0 as ::core::ffi::c_int as isize));
    IoReleaseRemoveLockEx(
        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
        ULongToPtr((*DeviceExtension).ComPort as ::core::ffi::c_ulong) as PVOID,
        ::core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    );
    if Status >= 0 as ::core::ffi::c_long {
        core::ptr::write(core::ptr::addr_of_mut!((*DeviceExtension).SerialLineControl), core::ptr::read(NewSettings as *const SERIAL_LINE_CONTROL));
    }
    return Status;
}
unsafe extern "system" fn SerialClearPerfStats(mut SynchronizeContext: PVOID) -> BOOLEAN {
    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION =
        SynchronizeContext as PSERIAL_DEVICE_EXTENSION;
    if DeviceExtension.is_null() {
        RtlAssert(
            b"DeviceExtension\0".as_ptr() as *const core::ffi::c_char as PVOID,
            b"drivers/serial/serial/devctrl.c\0".as_ptr() as *const core::ffi::c_char as PVOID,
            167 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    RtlZeroMemory(
        core::ptr::addr_of_mut!((*DeviceExtension).SerialPerfStats) as *mut core::ffi::c_void,
        core::mem::size_of::<SERIALPERF_STATS>(),
    );
    (*DeviceExtension).BreakInterruptErrorCount = 0 as ULONG;
    return 1 as BOOLEAN;
}

unsafe extern "system" fn SerialGetPerfStats(SynchronizeContext: PVOID) -> BOOLEAN {
    let pIrp: PIRP = SynchronizeContext as PIRP;
    let mut pDeviceExtension: PSERIAL_DEVICE_EXTENSION = core::ptr::null_mut();

    if pIrp.is_null() {
        RtlAssert(
            b"pIrp\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/serial/serial/devctrl.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            181 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    pDeviceExtension = (*(*IoGetCurrentIrpStackLocation(pIrp)).DeviceObject).DeviceExtension
        as PSERIAL_DEVICE_EXTENSION;

    RtlCopyMemory(
        (*pIrp).AssociatedIrp.SystemBuffer as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*pDeviceExtension).SerialPerfStats) as *const core::ffi::c_void,
        core::mem::size_of::<SERIALPERF_STATS>(),
    );
    return 1 as BOOLEAN;
}
unsafe fn SerialGetCommProp(
    pCommProp: PSERIAL_COMMPROP,
    DeviceExtension: PSERIAL_DEVICE_EXTENSION,
) -> NTSTATUS {
    if pCommProp.is_null() {
        RtlAssert(
            b"pCommProp\0" as *const u8 as *const i8 as PVOID,
            b"drivers/serial/serial/devctrl.c\0"
                as *const u8 as *const i8 as PVOID,
            199 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    RtlZeroMemory(
        pCommProp as *mut core::ffi::c_void,
        core::mem::size_of::<SERIAL_COMMPROP>(),
    );
    if (*pCommProp).ProvSpec1 as core::ffi::c_ulong & COMMPROP_INITIALIZED as core::ffi::c_ulong == 0
    {
        (*pCommProp).PacketLength = core::mem::size_of::<SERIAL_COMMPROP>() as USHORT;
    }
    (*pCommProp).PacketVersion = 2 as USHORT;
    (*pCommProp).ServiceMask = SERIAL_SP_SERIALCOMM as ULONG;
    (*pCommProp).CurrentTxQueue = ((*DeviceExtension).OutputBuffer.Length as core::ffi::c_ulong)
        .wrapping_sub(1 as core::ffi::c_ulong) as ULONG;
    (*pCommProp).MaxTxQueue = (*pCommProp).CurrentTxQueue;
    (*pCommProp).CurrentRxQueue = ((*DeviceExtension).InputBuffer.Length as core::ffi::c_ulong)
        .wrapping_sub(1 as core::ffi::c_ulong) as ULONG;
    (*pCommProp).MaxRxQueue = (*pCommProp).CurrentRxQueue;
    (*pCommProp).ProvSubType = PST_RS232 as ULONG;
    (*pCommProp).ProvCapabilities = (SERIAL_PCF_DTRDSR
        | SERIAL_PCF_INTTIMEOUTS
        | SERIAL_PCF_PARITY_CHECK
        | SERIAL_PCF_RTSCTS
        | SERIAL_PCF_SETXCHAR
        | SERIAL_PCF_SPECIALCHARS
        | SERIAL_PCF_TOTALTIMEOUTS
        | SERIAL_PCF_XONXOFF) as ULONG;
    (*pCommProp).SettableParams = (SERIAL_SP_BAUD
        | SERIAL_SP_DATABITS
        | SERIAL_SP_HANDSHAKING
        | SERIAL_SP_PARITY
        | SERIAL_SP_PARITY_CHECK
        | SERIAL_SP_STOPBITS) as ULONG;
    (*pCommProp).SettableBaud = (SERIAL_BAUD_075
        | SERIAL_BAUD_110
        | SERIAL_BAUD_134_5
        | SERIAL_BAUD_150
        | SERIAL_BAUD_300
        | SERIAL_BAUD_600
        | SERIAL_BAUD_1200
        | SERIAL_BAUD_1800
        | SERIAL_BAUD_2400
        | SERIAL_BAUD_4800
        | SERIAL_BAUD_7200
        | SERIAL_BAUD_9600
        | SERIAL_BAUD_USER) as ULONG;
    (*pCommProp).MaxBaud = SERIAL_BAUD_USER as ULONG;
    if (*DeviceExtension).UartType as core::ffi::c_uint
        >= Uart16450 as core::ffi::c_int as core::ffi::c_uint
    {
        (*pCommProp).SettableBaud |=
            (SERIAL_BAUD_14400 | SERIAL_BAUD_19200 | SERIAL_BAUD_38400) as core::ffi::c_ulong;
    }
    if (*DeviceExtension).UartType as core::ffi::c_uint
        >= Uart16550 as core::ffi::c_int as core::ffi::c_uint
    {
        (*pCommProp).SettableBaud |=
            (SERIAL_BAUD_56K | SERIAL_BAUD_57600 | SERIAL_BAUD_115200 | SERIAL_BAUD_128K)
                as core::ffi::c_ulong;
    }
    (*pCommProp).SettableData =
        (SERIAL_DATABITS_5 | SERIAL_DATABITS_6 | SERIAL_DATABITS_7 | SERIAL_DATABITS_8) as USHORT;
    (*pCommProp).SettableStopParity = (SERIAL_STOPBITS_10
        | SERIAL_STOPBITS_15
        | SERIAL_STOPBITS_20
        | SERIAL_PARITY_NONE
        | SERIAL_PARITY_ODD
        | SERIAL_PARITY_EVEN
        | SERIAL_PARITY_MARK
        | SERIAL_PARITY_SPACE) as USHORT;
    (*pCommProp).ProvSpec2 = 0 as ULONG;
    return STATUS_SUCCESS;
}
unsafe fn SerialGetCommStatus(
    pSerialStatus: PSERIAL_STATUS,
    DeviceExtension: PSERIAL_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Irql: KIRQL = 0;

    RtlZeroMemory(
        pSerialStatus as *mut core::ffi::c_void,
        core::mem::size_of::<SERIAL_STATUS>(),
    );

    (*pSerialStatus).Errors = 0;
    if (*DeviceExtension).BreakInterruptErrorCount != 0 {
        (*pSerialStatus).Errors |= SERIAL_ERROR_BREAK;
    }
    if (*DeviceExtension).SerialPerfStats.FrameErrorCount != 0 {
        (*pSerialStatus).Errors |= SERIAL_ERROR_FRAMING;
    }
    if (*DeviceExtension).SerialPerfStats.SerialOverrunErrorCount != 0 {
        (*pSerialStatus).Errors |= SERIAL_ERROR_OVERRUN;
    }
    if (*DeviceExtension).SerialPerfStats.BufferOverrunErrorCount != 0 {
        (*pSerialStatus).Errors |= SERIAL_ERROR_QUEUEOVERRUN;
    }
    if (*DeviceExtension).SerialPerfStats.ParityErrorCount != 0 {
        (*pSerialStatus).Errors |= SERIAL_ERROR_PARITY;
    }

    (*pSerialStatus).HoldReasons = 0; /* FIXME */

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*DeviceExtension).InputBufferLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(Irql),
    );
    (*pSerialStatus).AmountInInQueue = ((*DeviceExtension)
        .InputBuffer
        .WritePosition
        .wrapping_add((*DeviceExtension).InputBuffer.Length)
        .wrapping_sub((*DeviceExtension).InputBuffer.ReadPosition))
    .wrapping_rem((*DeviceExtension).InputBuffer.Length);
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*DeviceExtension).InputBufferLock) as PKSPIN_LOCK,
        Irql,
    );

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(Irql),
    );
    (*pSerialStatus).AmountInOutQueue = ((*DeviceExtension)
        .OutputBuffer
        .WritePosition
        .wrapping_add((*DeviceExtension).OutputBuffer.Length)
        .wrapping_sub((*DeviceExtension).OutputBuffer.ReadPosition))
    .wrapping_rem((*DeviceExtension).OutputBuffer.Length);
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
        Irql,
    );

    (*pSerialStatus).EofReceived = 0; /* always FALSE */
    (*pSerialStatus).WaitForImmediate = 0; /* always FALSE */

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn SerialDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Stack: PIO_STACK_LOCATION;
    let mut IoControlCode: ULONG;
    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION;
    let mut LengthIn: ULONG;
    let mut LengthOut: ULONG;
    let mut Information: ULONG_PTR = 0 as ULONG_PTR;
    let mut BufferIn: PVOID = core::ptr::null_mut();
    let mut BufferOut: PVOID = core::ptr::null_mut();
    let mut ComPortBase: PUCHAR;
    let mut Status: NTSTATUS;

    TRACE_!(SERIAL, b"IRP_MJ_DEVICE_CONTROL dispatch\n\0".as_ptr() as *const i8);

    Stack = IoGetCurrentIrpStackLocation(Irp);
    LengthIn = (*Stack).Parameters.DeviceIoControl.InputBufferLength;
    LengthOut = (*Stack).Parameters.DeviceIoControl.OutputBufferLength;
    DeviceExtension = (*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION;
    ComPortBase = ULongToPtr((*DeviceExtension).BaseAddress) as PUCHAR;
    IoControlCode = (*Stack).Parameters.DeviceIoControl.IoControlCode;
    SerialGetUserBuffers(
        Irp,
        IoControlCode,
        core::ptr::addr_of_mut!(BufferIn),
        core::ptr::addr_of_mut!(BufferOut),
    );

    match IoControlCode {
        IOCTL_SERIAL_CLEAR_STATS => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_CLEAR_STATS\n\0".as_ptr() as *const i8);
            KeSynchronizeExecution(
                (*DeviceExtension).Interrupt,
                Some(SerialClearPerfStats),
                DeviceExtension as PVOID,
            );
            Status = STATUS_SUCCESS;
        }
        IOCTL_SERIAL_CLR_DTR => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_CLR_DTR\n\0".as_ptr() as *const i8);
            Status = IoAcquireRemoveLock(
                core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                ULongToPtr((*DeviceExtension).ComPort) as PVOID,
            );
            if NT_SUCCESS(Status) {
                (*DeviceExtension).MCR =
                    ((*DeviceExtension).MCR as i32 & !(SR_MCR_DTR as i32)) as UCHAR;
                WRITE_PORT_UCHAR(SER_MCR(ComPortBase), (*DeviceExtension).MCR);
                IoReleaseRemoveLock(
                    core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                    ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                );
            }
        }
        IOCTL_SERIAL_CLR_RTS => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_CLR_RTS\n\0".as_ptr() as *const i8);
            Status = IoAcquireRemoveLock(
                core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                ULongToPtr((*DeviceExtension).ComPort) as PVOID,
            );
            if NT_SUCCESS(Status) {
                (*DeviceExtension).MCR =
                    ((*DeviceExtension).MCR as i32 & !(SR_MCR_RTS as i32)) as UCHAR;
                WRITE_PORT_UCHAR(SER_MCR(ComPortBase), (*DeviceExtension).MCR);
                IoReleaseRemoveLock(
                    core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                    ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                );
            }
        }
        IOCTL_SERIAL_CONFIG_SIZE => {
            let pConfigSize: PULONG;
            TRACE_!(SERIAL, b"IOCTL_SERIAL_CONFIG_SIZE\n\0".as_ptr() as *const i8);
            if LengthOut as usize != core::mem::size_of::<ULONG>() || BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                pConfigSize = BufferOut as PULONG;
                *pConfigSize = 0;
                Information = core::mem::size_of::<ULONG>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_GET_BAUD_RATE => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_GET_BAUD_RATE\n\0".as_ptr() as *const i8);
            if (LengthOut as usize) < core::mem::size_of::<SERIAL_BAUD_RATE>() {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else if BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                (*(BufferOut as PSERIAL_BAUD_RATE)).BaudRate = (*DeviceExtension).BaudRate;
                Information = core::mem::size_of::<SERIAL_BAUD_RATE>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_GET_CHARS => {
            let pSerialChars: PSERIAL_CHARS;
            ERR_(SERIAL, b"IOCTL_SERIAL_GET_CHARS not implemented.\n\0".as_ptr() as *const i8);
            if (LengthOut as usize) < core::mem::size_of::<SERIAL_CHARS>() {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else if BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                pSerialChars = BufferOut as PSERIAL_CHARS;
                (*pSerialChars).EofChar = 0 as UCHAR;
                (*pSerialChars).ErrorChar = 0 as UCHAR;
                (*pSerialChars).BreakChar = 0 as UCHAR;
                (*pSerialChars).EventChar = 0 as UCHAR;
                (*pSerialChars).XonChar = 0 as UCHAR;
                (*pSerialChars).XoffChar = 0 as UCHAR;
                Information = core::mem::size_of::<SERIAL_CHARS>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_GET_COMMSTATUS => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_GET_COMMSTATUS\n\0".as_ptr() as *const i8);
            if (LengthOut as usize) < core::mem::size_of::<SERIAL_STATUS>() {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else if BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                Status = SerialGetCommStatus(BufferOut as PSERIAL_STATUS, DeviceExtension);
                Information = core::mem::size_of::<SERIAL_STATUS>() as ULONG_PTR;
            }
        }
        IOCTL_SERIAL_GET_DTRRTS => {
            let pDtrRts: PULONG;
            TRACE_!(SERIAL, b"IOCTL_SERIAL_GET_DTRRTS\n\0".as_ptr() as *const i8);
            if LengthOut as usize != core::mem::size_of::<ULONG>() || BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                pDtrRts = BufferOut as PULONG;
                *pDtrRts = 0;
                if (*DeviceExtension).MCR as i32 & (SR_MCR_DTR as i32) != 0 {
                    *pDtrRts |= SERIAL_DTR_STATE as ULONG;
                }
                if (*DeviceExtension).MCR as i32 & (SR_MCR_RTS as i32) != 0 {
                    *pDtrRts |= SERIAL_RTS_STATE as ULONG;
                }
                Information = core::mem::size_of::<ULONG>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_GET_HANDFLOW => {
            let pSerialHandflow: PSERIAL_HANDFLOW;
            ERR_(SERIAL, b"IOCTL_SERIAL_GET_HANDFLOW not implemented.\n\0".as_ptr() as *const i8);
            if (LengthOut as usize) < core::mem::size_of::<SERIAL_HANDFLOW>() {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else if BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                pSerialHandflow = BufferOut as PSERIAL_HANDFLOW;
                (*pSerialHandflow).ControlHandShake = 0 as ULONG;
                (*pSerialHandflow).FlowReplace = 0 as ULONG;
                (*pSerialHandflow).XonLimit = 0 as LONG;
                (*pSerialHandflow).XoffLimit = 0 as LONG;
                Information = core::mem::size_of::<SERIAL_HANDFLOW>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_GET_LINE_CONTROL => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_GET_LINE_CONTROL\n\0".as_ptr() as *const i8);
            if (LengthOut as usize) < core::mem::size_of::<SERIAL_LINE_CONTROL>() {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else if BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                core::ptr::write(BufferOut as PSERIAL_LINE_CONTROL, core::ptr::read(core::ptr::addr_of!((*DeviceExtension).SerialLineControl)));
                Information = core::mem::size_of::<SERIAL_LINE_CONTROL>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_GET_MODEM_CONTROL => {
            let pMCR: PULONG;
            TRACE_!(SERIAL, b"IOCTL_SERIAL_GET_MODEM_CONTROL\n\0".as_ptr() as *const i8);
            if LengthOut as usize != core::mem::size_of::<ULONG>() || BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                pMCR = BufferOut as PULONG;
                *pMCR = (*DeviceExtension).MCR as ULONG;
                Information = core::mem::size_of::<ULONG>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_GET_MODEMSTATUS => {
            let pMSR: PULONG;
            TRACE_!(SERIAL, b"IOCTL_SERIAL_GET_MODEMSTATUS\n\0".as_ptr() as *const i8);
            if LengthOut as usize != core::mem::size_of::<ULONG>() || BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                pMSR = BufferOut as PULONG;
                *pMSR = (*DeviceExtension).MSR as ULONG;
                Information = core::mem::size_of::<ULONG>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_GET_PROPERTIES => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_GET_PROPERTIES\n\0".as_ptr() as *const i8);
            if (LengthOut as usize) < core::mem::size_of::<SERIAL_COMMPROP>() {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else if BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                Status = SerialGetCommProp(BufferOut as PSERIAL_COMMPROP, DeviceExtension);
                Information = core::mem::size_of::<SERIAL_COMMPROP>() as ULONG_PTR;
            }
        }
        IOCTL_SERIAL_GET_STATS => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_GET_STATS\n\0".as_ptr() as *const i8);
            if (LengthOut as usize) < core::mem::size_of::<SERIALPERF_STATS>() {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else if BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                KeSynchronizeExecution(
                    (*DeviceExtension).Interrupt,
                    Some(SerialGetPerfStats),
                    Irp as PVOID,
                );
                Information = core::mem::size_of::<SERIALPERF_STATS>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_GET_TIMEOUTS => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_GET_TIMEOUTS\n\0".as_ptr() as *const i8);
            if LengthOut as usize != core::mem::size_of::<SERIAL_TIMEOUTS>() || BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                core::ptr::write(BufferOut as PSERIAL_TIMEOUTS, core::ptr::read(core::ptr::addr_of!((*DeviceExtension).SerialTimeOuts)));
                Information = core::mem::size_of::<SERIAL_TIMEOUTS>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_GET_WAIT_MASK => {
            let pWaitMask: PULONG;
            TRACE_!(SERIAL, b"IOCTL_SERIAL_GET_WAIT_MASK\n\0".as_ptr() as *const i8);
            if LengthOut as usize != core::mem::size_of::<ULONG>() || BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                pWaitMask = BufferOut as PULONG;
                *pWaitMask = (*DeviceExtension).WaitMask as ULONG;
                Information = core::mem::size_of::<ULONG>() as ULONG_PTR;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_IMMEDIATE_CHAR => {
            ERR_(SERIAL, b"IOCTL_SERIAL_IMMEDIATE_CHAR not implemented.\n\0".as_ptr() as *const i8);
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_SERIAL_LSRMST_INSERT => {
            ERR_(SERIAL, b"IOCTL_SERIAL_LSRMST_INSERT not implemented.\n\0".as_ptr() as *const i8);
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_SERIAL_PURGE => {
            let mut Irql: KIRQL = 0;
            TRACE_!(SERIAL, b"IOCTL_SERIAL_PURGE\n\0".as_ptr() as *const i8);
            if LengthIn as usize != core::mem::size_of::<ULONG>() || BufferIn.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                let PurgeMask: ULONG = *(BufferIn as PULONG);
                Status = STATUS_SUCCESS;
                if PurgeMask & SERIAL_PURGE_RXCLEAR as ULONG != 0 {
                    KeAcquireSpinLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).InputBufferLock) as PKSPIN_LOCK,
                        core::ptr::addr_of_mut!(Irql),
                    );
                    (*DeviceExtension).InputBuffer.WritePosition = 0 as ULONG;
                    (*DeviceExtension).InputBuffer.ReadPosition =
                        (*DeviceExtension).InputBuffer.WritePosition;
                    if (*DeviceExtension).UartType as u32 >= Uart16550A as u32 {
                        Status = IoAcquireRemoveLock(
                            core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                            ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                        );
                        if NT_SUCCESS(Status) {
                            WRITE_PORT_UCHAR(SER_FCR(ComPortBase), SR_FCR_CLEAR_RCVR as UCHAR);
                            IoReleaseRemoveLock(
                                core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                                ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                            );
                        }
                    }
                    KeReleaseSpinLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).InputBufferLock) as PKSPIN_LOCK,
                        Irql,
                    );
                }

                if PurgeMask & SERIAL_PURGE_TXCLEAR as ULONG != 0 {
                    KeAcquireSpinLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
                        core::ptr::addr_of_mut!(Irql),
                    );
                    (*DeviceExtension).OutputBuffer.WritePosition = 0 as ULONG;
                    (*DeviceExtension).OutputBuffer.ReadPosition =
                        (*DeviceExtension).OutputBuffer.WritePosition;
                    if (*DeviceExtension).UartType as u32 >= Uart16550A as u32 {
                        Status = IoAcquireRemoveLock(
                            core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                            ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                        );
                        if NT_SUCCESS(Status) {
                            WRITE_PORT_UCHAR(SER_FCR(ComPortBase), SR_FCR_CLEAR_XMIT as UCHAR);
                            IoReleaseRemoveLock(
                                core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                                ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                            );
                        }
                    }
                    KeReleaseSpinLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
                        Irql,
                    );
                }
            }
        }
        IOCTL_SERIAL_RESET_DEVICE => {
            ERR_(SERIAL, b"IOCTL_SERIAL_RESET_DEVICE not implemented.\n\0".as_ptr() as *const i8);
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_SERIAL_SET_BAUD_RATE => {
            let pNewBaudRate: PULONG;
            TRACE_!(SERIAL, b"IOCTL_SERIAL_SET_BAUD_RATE\n\0".as_ptr() as *const i8);
            if LengthIn as usize != core::mem::size_of::<ULONG>() || BufferIn.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                pNewBaudRate = BufferIn as PULONG;
                Status = SerialSetBaudRate(DeviceExtension, *pNewBaudRate);
            }
        }
        IOCTL_SERIAL_SET_BREAK_OFF => {
            ERR_(SERIAL, b"IOCTL_SERIAL_SET_BREAK_OFF not implemented.\n\0".as_ptr() as *const i8);
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_SERIAL_SET_BREAK_ON => {
            ERR_(SERIAL, b"IOCTL_SERIAL_SET_BREAK_ON not implemented.\n\0".as_ptr() as *const i8);
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_SERIAL_SET_CHARS => {
            ERR_(SERIAL, b"IOCTL_SERIAL_SET_CHARS not implemented.\n\0".as_ptr() as *const i8);
            Status = STATUS_SUCCESS;
        }
        IOCTL_SERIAL_SET_DTR => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_SET_DTR\n\0".as_ptr() as *const i8);
            if (*DeviceExtension).MCR as i32 & (SR_MCR_DTR as i32) == 0 {
                Status = IoAcquireRemoveLock(
                    core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                    ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                );
                if NT_SUCCESS(Status) {
                    (*DeviceExtension).MCR =
                        ((*DeviceExtension).MCR as i32 | (SR_MCR_DTR as i32)) as UCHAR;
                    WRITE_PORT_UCHAR(SER_MCR(ComPortBase), (*DeviceExtension).MCR);
                    IoReleaseRemoveLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                        ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                    );
                }
            } else {
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_SET_FIFO_CONTROL => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_SET_FIFO_CONTROL\n\0".as_ptr() as *const i8);
            if LengthIn as usize != core::mem::size_of::<ULONG>() || BufferIn.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                Status = IoAcquireRemoveLock(
                    core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                    ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                );
                if NT_SUCCESS(Status) {
                    WRITE_PORT_UCHAR(
                        SER_FCR(ComPortBase),
                        (*(BufferIn as PULONG) & 0xff) as UCHAR,
                    );
                    IoReleaseRemoveLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                        ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                    );
                }
            }
        }
        IOCTL_SERIAL_SET_HANDFLOW => {
            ERR_(SERIAL, b"IOCTL_SERIAL_SET_HANDFLOW not implemented.\n\0".as_ptr() as *const i8);
            Status = STATUS_SUCCESS;
        }
        IOCTL_SERIAL_SET_LINE_CONTROL => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_SET_LINE_CONTROL\n\0".as_ptr() as *const i8);
            if (LengthIn as usize) < core::mem::size_of::<SERIAL_LINE_CONTROL>() {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else if BufferIn.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                Status = SerialSetLineControl(DeviceExtension, BufferIn as PSERIAL_LINE_CONTROL);
            }
        }
        IOCTL_SERIAL_SET_MODEM_CONTROL => {
            let pMCR: PULONG;
            TRACE_!(SERIAL, b"IOCTL_SERIAL_SET_MODEM_CONTROL\n\0".as_ptr() as *const i8);
            if LengthIn as usize != core::mem::size_of::<ULONG>() || BufferIn.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                Status = IoAcquireRemoveLock(
                    core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                    ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                );
                if NT_SUCCESS(Status) {
                    pMCR = BufferIn as PULONG;
                    (*DeviceExtension).MCR = (*pMCR & 0xff) as UCHAR;
                    WRITE_PORT_UCHAR(SER_MCR(ComPortBase), (*DeviceExtension).MCR);
                    IoReleaseRemoveLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                        ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                    );
                }
            }
        }
        IOCTL_SERIAL_SET_QUEUE_SIZE => {
            if (LengthIn as usize) < core::mem::size_of::<SERIAL_QUEUE_SIZE>() {
                return STATUS_BUFFER_TOO_SMALL;
            } else if BufferIn.is_null() {
                return STATUS_INVALID_PARAMETER;
            } else {
                let mut Irql: KIRQL = 0;
                let NewQueueSize: PSERIAL_QUEUE_SIZE = BufferIn as PSERIAL_QUEUE_SIZE;
                Status = STATUS_SUCCESS;
                if (*NewQueueSize).InSize > (*DeviceExtension).InputBuffer.Length {
                    KeAcquireSpinLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).InputBufferLock) as PKSPIN_LOCK,
                        core::ptr::addr_of_mut!(Irql),
                    );
                    Status = IncreaseCircularBufferSize(
                        core::ptr::addr_of_mut!((*DeviceExtension).InputBuffer),
                        (*NewQueueSize).InSize,
                    );
                    KeReleaseSpinLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).InputBufferLock) as PKSPIN_LOCK,
                        Irql,
                    );
                }
                if NT_SUCCESS(Status)
                    && (*NewQueueSize).OutSize > (*DeviceExtension).OutputBuffer.Length
                {
                    KeAcquireSpinLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
                        core::ptr::addr_of_mut!(Irql),
                    );
                    Status = IncreaseCircularBufferSize(
                        core::ptr::addr_of_mut!((*DeviceExtension).OutputBuffer),
                        (*NewQueueSize).OutSize,
                    );
                    KeReleaseSpinLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
                        Irql,
                    );
                }
            }
        }
        IOCTL_SERIAL_SET_RTS => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_SET_RTS\n\0".as_ptr() as *const i8);
            if (*DeviceExtension).MCR as i32 & (SR_MCR_RTS as i32) == 0 {
                Status = IoAcquireRemoveLock(
                    core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                    ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                );
                if NT_SUCCESS(Status) {
                    (*DeviceExtension).MCR =
                        ((*DeviceExtension).MCR as i32 | (SR_MCR_RTS as i32)) as UCHAR;
                    WRITE_PORT_UCHAR(SER_MCR(ComPortBase), (*DeviceExtension).MCR);
                    IoReleaseRemoveLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                        ULongToPtr((*DeviceExtension).ComPort) as PVOID,
                    );
                }
            } else {
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_SET_TIMEOUTS => {
            TRACE_!(SERIAL, b"IOCTL_SERIAL_SET_TIMEOUTS\n\0".as_ptr() as *const i8);
            if LengthIn as usize != core::mem::size_of::<SERIAL_TIMEOUTS>() || BufferIn.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                core::ptr::write(core::ptr::addr_of_mut!((*DeviceExtension).SerialTimeOuts), core::ptr::read(BufferIn as PSERIAL_TIMEOUTS));
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_SET_WAIT_MASK => {
            let pWaitMask: PULONG = BufferIn as PULONG;
            TRACE_!(SERIAL, b"IOCTL_SERIAL_SET_WAIT_MASK\n\0".as_ptr() as *const i8);
            if LengthIn as usize != core::mem::size_of::<ULONG>() || BufferIn.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else if !(*DeviceExtension).WaitOnMaskIrp.is_null() {
                WARN_(SERIAL, b"An IRP is already currently processed\n\0".as_ptr() as *const i8);
                Status = STATUS_INVALID_PARAMETER;
            } else {
                (*DeviceExtension).WaitMask = *pWaitMask as ULONG;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_SERIAL_SET_XOFF => {
            ERR_(SERIAL, b"IOCTL_SERIAL_SET_XOFF not implemented.\n\0".as_ptr() as *const i8);
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_SERIAL_SET_XON => {
            ERR_(SERIAL, b"IOCTL_SERIAL_SET_XON not implemented.\n\0".as_ptr() as *const i8);
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_SERIAL_WAIT_ON_MASK => {
            let WaitingIrp: PIRP;
            TRACE_!(SERIAL, b"IOCTL_SERIAL_WAIT_ON_MASK\n\0".as_ptr() as *const i8);
            if LengthOut as usize != core::mem::size_of::<ULONG>() || BufferOut.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                IoMarkIrpPending(Irp);

                WaitingIrp = InterlockedCompareExchangePointer(
                    core::ptr::addr_of_mut!((*DeviceExtension).WaitOnMaskIrp) as *mut PVOID,
                    Irp as PVOID,
                    core::ptr::null_mut(),
                ) as PIRP;

                if !WaitingIrp.is_null() {
                    WARN_(
                        SERIAL,
                        b"Unable to pend a second IRP for IOCTL_SERIAL_WAIT_ON_MASK\n\0".as_ptr()
                            as *const i8,
                    );
                    (*Irp).IoStatus.Information = 0 as _;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
                }
                return STATUS_PENDING;
            }
        }
        IOCTL_SERIAL_XOFF_COUNTER => {
            ERR_(SERIAL, b"IOCTL_SERIAL_XOFF_COUNTER not implemented.\n\0".as_ptr() as *const i8);
            Status = STATUS_NOT_IMPLEMENTED;
        }
        _ => {
            // TRACE_!(SERIAL, "Unknown IOCTL code 0x%x\n", IoControlCode) → DbgPrintEx
            DbgPrintEx(
                DPFLTR_SERIAL_ID,
                DPFLTR_TRACE_LEVEL,
                b"Unknown IOCTL code 0x%x\n\0".as_ptr() as PCSTR,
                (*Stack).Parameters.DeviceIoControl.IoControlCode,
            );
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*DeviceExtension).LowerDevice, Irp);
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    if Status == STATUS_PENDING {
        IoMarkIrpPending(Irp);
    } else {
        (*Irp).IoStatus.Information = Information as _;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Serial port driver
//  * FILE:            drivers/dd/serial/devctrl.c
//  * PURPOSE:         Serial IRP_MJ_DEVICE_CONTROL operations
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "serial.h"
//
// #include <debug.h>
//
// #define IO_METHOD_FROM_CTL_CODE(ctlCode) (ctlCode&0x00000003)
//
// static VOID
// SerialGetUserBuffers(
// 	IN PIRP Irp,
// 	IN ULONG IoControlCode,
// 	OUT PVOID* BufferIn,
// 	OUT PVOID* BufferOut)
// {
// 	ASSERT(Irp);
// 	ASSERT(BufferIn);
// 	ASSERT(BufferOut);
//
// 	switch (IO_METHOD_FROM_CTL_CODE(IoControlCode))
// 	{
// 		case METHOD_BUFFERED:
// 			*BufferIn = *BufferOut = Irp->AssociatedIrp.SystemBuffer;
// 			break;
// 		case METHOD_IN_DIRECT:
// 		case METHOD_OUT_DIRECT:
// 			*BufferIn = Irp->AssociatedIrp.SystemBuffer;
// 			*BufferOut = MmGetSystemAddressForMdlSafe(Irp->MdlAddress, NormalPagePriority);
// 			break;
// 		case METHOD_NEITHER:
// 			*BufferIn = IoGetCurrentIrpStackLocation(Irp)->Parameters.DeviceIoControl.Type3InputBuffer;
// 			*BufferOut = Irp->UserBuffer;
// 			break;
// 		default:
// 			/* Should never happen */
// 			*BufferIn = NULL;
// 			*BufferOut = NULL;
// 			break;
// 	}
// }
//
// NTSTATUS NTAPI
// SerialSetBaudRate(
// 	IN PSERIAL_DEVICE_EXTENSION DeviceExtension,
// 	IN ULONG NewBaudRate)
// {
// 	ULONG BaudRate;
// 	USHORT divisor;
// 	PUCHAR ComPortBase = ULongToPtr(DeviceExtension->BaseAddress);
// 	NTSTATUS Status = STATUS_SUCCESS;
//
// 	if (NewBaudRate == 0)
// 		return STATUS_INVALID_PARAMETER;
//
// 	divisor = (USHORT)(BAUD_CLOCK / (CLOCKS_PER_BIT * NewBaudRate));
// 	BaudRate = BAUD_CLOCK / (CLOCKS_PER_BIT * divisor);
//
// 	Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 	if (NT_SUCCESS(Status))
// 	{
// 		UCHAR Lcr;
// 		TRACE_!(SERIAL, "SerialSetBaudRate(COM%lu, %lu Bauds)\n", DeviceExtension->ComPort, BaudRate);
// 		/* Set Bit 7 of LCR to expose baud registers */
// 		Lcr = READ_PORT_UCHAR(SER_LCR(ComPortBase));
// 		WRITE_PORT_UCHAR(SER_LCR(ComPortBase), Lcr | (SR_LCR_DLAB as i32));
// 		/* Write the baud rate */
// 		WRITE_PORT_UCHAR(SER_DLL(ComPortBase), divisor & 0xff);
// 		WRITE_PORT_UCHAR(SER_DLM(ComPortBase), divisor >> 8);
// 		/* Switch back to normal registers */
// 		WRITE_PORT_UCHAR(SER_LCR(ComPortBase), Lcr);
//
// 		IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 	}
//
// 	if (NT_SUCCESS(Status))
// 		DeviceExtension->BaudRate = BaudRate;
// 	return Status;
// }
//
// NTSTATUS NTAPI
// SerialSetLineControl(
// 	IN PSERIAL_DEVICE_EXTENSION DeviceExtension,
// 	IN PSERIAL_LINE_CONTROL NewSettings)
// {
// 	PUCHAR ComPortBase;
// 	UCHAR Lcr = 0;
// 	NTSTATUS Status;
//
// 	ASSERT(DeviceExtension);
// 	ASSERT(NewSettings);
//
// 	TRACE_!(SERIAL, "SerialSetLineControl(COM%lu, Settings { %lu %lu %lu })\n",
// 		DeviceExtension->ComPort, NewSettings->StopBits, NewSettings->Parity, NewSettings->WordLength);
//
// 	/* Verify parameters */
// 	switch (NewSettings->WordLength)
// 	{
// 		case 5: Lcr |= (SR_LCR_CS5 as i32); break;
// 		case 6: Lcr |= (SR_LCR_CS6 as i32); break;
// 		case 7: Lcr |= (SR_LCR_CS7 as i32); break;
// 		case 8: Lcr |= (SR_LCR_CS8 as i32); break;
// 		default: return STATUS_INVALID_PARAMETER;
// 	}
//
// 	if (NewSettings->WordLength < 5 || NewSettings->WordLength > 8)
// 		return STATUS_INVALID_PARAMETER;
//
// 	switch (NewSettings->Parity)
// 	{
// 		case NO_PARITY:    Lcr |= (SR_LCR_PNO as i32); break;
// 		case ODD_PARITY:   Lcr |= (SR_LCR_POD as i32); break;
// 		case EVEN_PARITY:  Lcr |= (SR_LCR_PEV as i32); break;
// 		case MARK_PARITY:  Lcr |= (SR_LCR_PMK as i32); break;
// 		case SPACE_PARITY: Lcr |= (SR_LCR_PSP as i32); break;
// 		default: return STATUS_INVALID_PARAMETER;
// 	}
//
// 	switch (NewSettings->StopBits)
// 	{
// 		case STOP_BIT_1:
// 			Lcr |= (SR_LCR_ST1 as i32);
// 			break;
// 		case STOP_BITS_1_5:
// 			if (NewSettings->WordLength != 5)
// 				return STATUS_INVALID_PARAMETER;
// 			Lcr |= (SR_LCR_ST2 as i32);
// 			break;
// 		case STOP_BITS_2:
// 			if (NewSettings->WordLength < 6 || NewSettings->WordLength > 8)
// 				return STATUS_INVALID_PARAMETER;
// 			Lcr |= (SR_LCR_ST2 as i32);
// 			break;
// 		default:
// 			return STATUS_INVALID_PARAMETER;
// 	}
//
// 	/* Update current parameters */
// 	ComPortBase = ULongToPtr(DeviceExtension->BaseAddress);
// 	Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 	if (!NT_SUCCESS(Status))
// 		return Status;
// 	WRITE_PORT_UCHAR(SER_LCR(ComPortBase), Lcr);
//
// 	/* Read junk out of RBR */
// 	READ_PORT_UCHAR(SER_RBR(ComPortBase));
// 	IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
//
// 	if (NT_SUCCESS(Status))
// 		DeviceExtension->SerialLineControl = *NewSettings;
//
// 	return Status;
// }
//
// static
// BOOLEAN
// NTAPI
// SerialClearPerfStats(IN PVOID SynchronizeContext)
// {
//     PSERIAL_DEVICE_EXTENSION DeviceExtension = SynchronizeContext;
//     ASSERT(DeviceExtension);
//     RtlZeroMemory(&DeviceExtension->SerialPerfStats, sizeof(SERIALPERF_STATS));
//     DeviceExtension->BreakInterruptErrorCount = 0;
//     return TRUE;
// }
//
// static
// BOOLEAN
// NTAPI
// SerialGetPerfStats(IN PVOID SynchronizeContext)
// {
//     PIRP pIrp = SynchronizeContext;
//     PSERIAL_DEVICE_EXTENSION pDeviceExtension;
//
//     ASSERT(pIrp);
//     pDeviceExtension = IoGetCurrentIrpStackLocation(pIrp)->DeviceObject->DeviceExtension;
//
//     /*
//     * we assume buffer is big enough to hold SerialPerfStats structure
//     * caller must verify this
//     */
//     RtlCopyMemory(pIrp->AssociatedIrp.SystemBuffer,
//                   &pDeviceExtension->SerialPerfStats,
//                   sizeof(SERIALPERF_STATS));
//     return TRUE;
// }
//
// static NTSTATUS
// SerialGetCommProp(
// 	OUT PSERIAL_COMMPROP pCommProp,
// 	IN PSERIAL_DEVICE_EXTENSION DeviceExtension)
// {
// 	ASSERT(pCommProp);
//
// 	RtlZeroMemory(pCommProp, sizeof(SERIAL_COMMPROP));
//
// 	if (!(pCommProp->ProvSpec1 & COMMPROP_INITIALIZED))
// 		pCommProp->PacketLength = sizeof(SERIAL_COMMPROP);
// 	pCommProp->PacketVersion = 2;
// 	pCommProp->ServiceMask = SERIAL_SP_SERIALCOMM;
// 	pCommProp->MaxTxQueue = pCommProp->CurrentTxQueue = DeviceExtension->OutputBuffer.Length - 1;
// 	pCommProp->MaxRxQueue = pCommProp->CurrentRxQueue = DeviceExtension->InputBuffer.Length - 1;
// 	pCommProp->ProvSubType = PST_RS232;
// 	pCommProp->ProvCapabilities = SERIAL_PCF_DTRDSR | SERIAL_PCF_INTTIMEOUTS | SERIAL_PCF_PARITY_CHECK
// 		| SERIAL_PCF_RTSCTS | SERIAL_PCF_SETXCHAR | SERIAL_PCF_SPECIALCHARS | SERIAL_PCF_TOTALTIMEOUTS
// 		| SERIAL_PCF_XONXOFF;
// 	pCommProp->SettableParams = SERIAL_SP_BAUD | SERIAL_SP_DATABITS | SERIAL_SP_HANDSHAKING
// 		| SERIAL_SP_PARITY | SERIAL_SP_PARITY_CHECK | SERIAL_SP_STOPBITS;
//
// 	/* SettableBaud is related to Uart type */
// 	pCommProp->SettableBaud = SERIAL_BAUD_075 | SERIAL_BAUD_110 | SERIAL_BAUD_134_5
// 		| SERIAL_BAUD_150 | SERIAL_BAUD_300 | SERIAL_BAUD_600 | SERIAL_BAUD_1200
// 		| SERIAL_BAUD_1800 | SERIAL_BAUD_2400 | SERIAL_BAUD_4800 | SERIAL_BAUD_7200
// 		| SERIAL_BAUD_9600 | SERIAL_BAUD_USER;
// 	pCommProp->MaxBaud = SERIAL_BAUD_USER;
// 	if (DeviceExtension->UartType >= Uart16450)
// 	{
// 		pCommProp->SettableBaud |= SERIAL_BAUD_14400 | SERIAL_BAUD_19200 | SERIAL_BAUD_38400;
// 	}
// 	if (DeviceExtension->UartType >= Uart16550)
// 	{
// 		pCommProp->SettableBaud |= SERIAL_BAUD_56K | SERIAL_BAUD_57600 | SERIAL_BAUD_115200 | SERIAL_BAUD_128K;
// 	}
//
// 	pCommProp->SettableData = SERIAL_DATABITS_5 | SERIAL_DATABITS_6 | SERIAL_DATABITS_7 | SERIAL_DATABITS_8;
// 	pCommProp->SettableStopParity = SERIAL_STOPBITS_10 | SERIAL_STOPBITS_15 | SERIAL_STOPBITS_20
// 		| SERIAL_PARITY_NONE | SERIAL_PARITY_ODD | SERIAL_PARITY_EVEN | SERIAL_PARITY_MARK | SERIAL_PARITY_SPACE;
//
// 	pCommProp->ProvSpec2 = 0; /* Size of provider-specific data */
//
// 	return STATUS_SUCCESS;
// }
//
// static NTSTATUS
// SerialGetCommStatus(
// 	OUT PSERIAL_STATUS pSerialStatus,
// 	IN PSERIAL_DEVICE_EXTENSION DeviceExtension)
// {
// 	KIRQL Irql;
//
// 	ASSERT(pSerialStatus);
// 	RtlZeroMemory(pSerialStatus, sizeof(SERIAL_STATUS));
//
// 	pSerialStatus->Errors = 0;
// 	if (DeviceExtension->BreakInterruptErrorCount)
// 		pSerialStatus->Errors |= SERIAL_ERROR_BREAK;
// 	if (DeviceExtension->SerialPerfStats.FrameErrorCount)
// 		pSerialStatus->Errors |= SERIAL_ERROR_FRAMING;
// 	if (DeviceExtension->SerialPerfStats.SerialOverrunErrorCount)
// 		pSerialStatus->Errors |= SERIAL_ERROR_OVERRUN;
// 	if (DeviceExtension->SerialPerfStats.BufferOverrunErrorCount)
// 		pSerialStatus->Errors |= SERIAL_ERROR_QUEUEOVERRUN;
// 	if (DeviceExtension->SerialPerfStats.ParityErrorCount)
// 		pSerialStatus->Errors |= SERIAL_ERROR_PARITY;
//
// 	pSerialStatus->HoldReasons = 0; /* FIXME */
//
// 	KeAcquireSpinLock(&DeviceExtension->InputBufferLock, &Irql);
// 	pSerialStatus->AmountInInQueue = (DeviceExtension->InputBuffer.WritePosition + DeviceExtension->InputBuffer.Length
// 		- DeviceExtension->InputBuffer.ReadPosition) % DeviceExtension->InputBuffer.Length;
// 	KeReleaseSpinLock(&DeviceExtension->InputBufferLock, Irql);
//
// 	KeAcquireSpinLock(&DeviceExtension->OutputBufferLock, &Irql);
// 	pSerialStatus->AmountInOutQueue = (DeviceExtension->OutputBuffer.WritePosition + DeviceExtension->OutputBuffer.Length
// 		- DeviceExtension->OutputBuffer.ReadPosition) % DeviceExtension->OutputBuffer.Length;
// 	KeReleaseSpinLock(&DeviceExtension->OutputBufferLock, Irql);
//
// 	pSerialStatus->EofReceived = FALSE; /* always FALSE */
// 	pSerialStatus->WaitForImmediate = FALSE; /* always FALSE */
//
// 	return STATUS_SUCCESS;
// }
//
// NTSTATUS NTAPI
// SerialDeviceControl(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PIO_STACK_LOCATION Stack;
// 	ULONG IoControlCode;
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension;
// 	ULONG LengthIn, LengthOut;
// 	ULONG_PTR Information = 0;
// 	PVOID BufferIn, BufferOut;
// 	PUCHAR ComPortBase;
// 	NTSTATUS Status;
//
// 	TRACE_!(SERIAL, "IRP_MJ_DEVICE_CONTROL dispatch\n");
//
// 	Stack = IoGetCurrentIrpStackLocation(Irp);
// 	LengthIn = Stack->Parameters.DeviceIoControl.InputBufferLength;
// 	LengthOut = Stack->Parameters.DeviceIoControl.OutputBufferLength;
// 	DeviceExtension = (PSERIAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
// 	ComPortBase = ULongToPtr(DeviceExtension->BaseAddress);
// 	IoControlCode = Stack->Parameters.DeviceIoControl.IoControlCode;
// 	SerialGetUserBuffers(Irp, IoControlCode, &BufferIn, &BufferOut);
//
// 	/* FIXME: need to probe buffers */
// 	/* FIXME: see https://web.archive.org/web/20101020230420/http://www.osronline.com/ddkx/serial/serref_61bm.htm */
// 	switch (IoControlCode)
// 	{
// 		case IOCTL_SERIAL_CLEAR_STATS:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_CLEAR_STATS\n");
// 			KeSynchronizeExecution(
// 				DeviceExtension->Interrupt,
// 				SerialClearPerfStats,
// 				DeviceExtension);
// 			Status = STATUS_SUCCESS;
// 			break;
// 		}
// 		case IOCTL_SERIAL_CLR_DTR:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_CLR_DTR\n");
// 			/* FIXME: If the handshake flow control of the device is configured to
// 			 * automatically use DTR, return STATUS_INVALID_PARAMETER */
// 			Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 			if (NT_SUCCESS(Status))
// 			{
// 				DeviceExtension->MCR &= ~(SR_MCR_DTR as i32);
// 				WRITE_PORT_UCHAR(SER_MCR(ComPortBase), DeviceExtension->MCR);
// 				IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_CLR_RTS:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_CLR_RTS\n");
// 			/* FIXME: If the handshake flow control of the device is configured to
// 			 * automatically use RTS, return STATUS_INVALID_PARAMETER */
// 			Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 			if (NT_SUCCESS(Status))
// 			{
// 				DeviceExtension->MCR &= ~(SR_MCR_RTS as i32);
// 				WRITE_PORT_UCHAR(SER_MCR(ComPortBase), DeviceExtension->MCR);
// 				IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_CONFIG_SIZE:
// 		{
// 			/* Obsolete on Microsoft Windows 2000+ */
// 			PULONG pConfigSize;
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_CONFIG_SIZE\n");
// 			if (LengthOut != sizeof(ULONG) || BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				pConfigSize = (PULONG)BufferOut;
// 				*pConfigSize = 0;
// 				Information = sizeof(ULONG);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_BAUD_RATE:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_GET_BAUD_RATE\n");
// 			if (LengthOut < sizeof(SERIAL_BAUD_RATE))
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 			else if (BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				((PSERIAL_BAUD_RATE)BufferOut)->BaudRate = DeviceExtension->BaudRate;
// 				Information = sizeof(SERIAL_BAUD_RATE);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_CHARS:
// 		{
// 			/* FIXME */
// 			PSERIAL_CHARS pSerialChars;
// 			ERR_(SERIAL, "IOCTL_SERIAL_GET_CHARS not implemented.\n");
// 			if (LengthOut < sizeof(SERIAL_CHARS))
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 			else if (BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				pSerialChars = (PSERIAL_CHARS)BufferOut;
// 				pSerialChars->EofChar = 0;
// 				pSerialChars->ErrorChar = 0;
// 				pSerialChars->BreakChar = 0;
// 				pSerialChars->EventChar = 0;
// 				pSerialChars->XonChar = 0;
// 				pSerialChars->XoffChar = 0;
// 				Information = sizeof(SERIAL_CHARS);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_COMMSTATUS:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_GET_COMMSTATUS\n");
// 			if (LengthOut < sizeof(SERIAL_STATUS))
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 			else if (BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				Status = SerialGetCommStatus((PSERIAL_STATUS)BufferOut, DeviceExtension);
// 				Information = sizeof(SERIAL_STATUS);
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_DTRRTS:
// 		{
// 			PULONG pDtrRts;
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_GET_DTRRTS\n");
// 			if (LengthOut != sizeof(ULONG) || BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				pDtrRts = (PULONG)BufferOut;
// 				*pDtrRts = 0;
// 				if (DeviceExtension->MCR & (SR_MCR_DTR as i32))
// 					*pDtrRts |= SERIAL_DTR_STATE;
// 				if (DeviceExtension->MCR & (SR_MCR_RTS as i32))
// 					*pDtrRts |= SERIAL_RTS_STATE;
// 				Information = sizeof(ULONG);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_HANDFLOW:
// 		{
// 			/* FIXME */
// 			PSERIAL_HANDFLOW pSerialHandflow;
// 			ERR_(SERIAL, "IOCTL_SERIAL_GET_HANDFLOW not implemented.\n");
// 			if (LengthOut < sizeof(SERIAL_HANDFLOW))
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 			else if (BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				pSerialHandflow = (PSERIAL_HANDFLOW)BufferOut;
// 				pSerialHandflow->ControlHandShake = 0;
// 				pSerialHandflow->FlowReplace = 0;
// 				pSerialHandflow->XonLimit = 0;
// 				pSerialHandflow->XoffLimit = 0;
// 				Information = sizeof(SERIAL_HANDFLOW);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_LINE_CONTROL:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_GET_LINE_CONTROL\n");
// 			if (LengthOut < sizeof(SERIAL_LINE_CONTROL))
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 			else if (BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				*((PSERIAL_LINE_CONTROL)BufferOut) = DeviceExtension->SerialLineControl;
// 				Information = sizeof(SERIAL_LINE_CONTROL);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_MODEM_CONTROL:
// 		{
// 			PULONG pMCR;
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_GET_MODEM_CONTROL\n");
// 			if (LengthOut != sizeof(ULONG) || BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				pMCR = (PULONG)BufferOut;
// 				*pMCR = DeviceExtension->MCR;
// 				Information = sizeof(ULONG);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_MODEMSTATUS:
// 		{
// 			PULONG pMSR;
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_GET_MODEMSTATUS\n");
// 			if (LengthOut != sizeof(ULONG) || BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				pMSR = (PULONG)BufferOut;
// 				*pMSR = DeviceExtension->MSR;
// 				Information = sizeof(ULONG);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_PROPERTIES:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_GET_PROPERTIES\n");
// 			if (LengthOut < sizeof(SERIAL_COMMPROP))
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 			else if (BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				Status = SerialGetCommProp((PSERIAL_COMMPROP)BufferOut, DeviceExtension);
// 				Information = sizeof(SERIAL_COMMPROP);
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_STATS:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_GET_STATS\n");
// 			if (LengthOut < sizeof(SERIALPERF_STATS))
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 			else if (BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				KeSynchronizeExecution(DeviceExtension->Interrupt,
// 					SerialGetPerfStats, Irp);
// 				Information = sizeof(SERIALPERF_STATS);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_TIMEOUTS:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_GET_TIMEOUTS\n");
// 			if (LengthOut != sizeof(SERIAL_TIMEOUTS) || BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				*(PSERIAL_TIMEOUTS)BufferOut = DeviceExtension->SerialTimeOuts;
// 				Information = sizeof(SERIAL_TIMEOUTS);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_GET_WAIT_MASK:
// 		{
// 			PULONG pWaitMask;
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_GET_WAIT_MASK\n");
// 			if (LengthOut != sizeof(ULONG) || BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				pWaitMask = (PULONG)BufferOut;
// 				*pWaitMask = DeviceExtension->WaitMask;
// 				Information = sizeof(ULONG);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_IMMEDIATE_CHAR:
// 		{
// 			/* FIXME */
// 			ERR_(SERIAL, "IOCTL_SERIAL_IMMEDIATE_CHAR not implemented.\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_SERIAL_LSRMST_INSERT:
// 		{
// 			/* FIXME */
// 			ERR_(SERIAL, "IOCTL_SERIAL_LSRMST_INSERT not implemented.\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_SERIAL_PURGE:
// 		{
// 			KIRQL Irql;
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_PURGE\n");
// 			/* FIXME: SERIAL_PURGE_RXABORT and SERIAL_PURGE_TXABORT
// 			 * should stop current request */
// 			if (LengthIn != sizeof(ULONG) || BufferIn == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				ULONG PurgeMask = *(PULONG)BufferIn;
//
// 				Status = STATUS_SUCCESS;
// 				/* FIXME: use SERIAL_PURGE_RXABORT and SERIAL_PURGE_TXABORT flags */
// 				if (PurgeMask & SERIAL_PURGE_RXCLEAR)
// 				{
// 					KeAcquireSpinLock(&DeviceExtension->InputBufferLock, &Irql);
// 					DeviceExtension->InputBuffer.ReadPosition = DeviceExtension->InputBuffer.WritePosition = 0;
// 					if (DeviceExtension->UartType >= Uart16550A)
// 					{
// 						/* Clear also Uart FIFO */
// 						Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 						if (NT_SUCCESS(Status))
// 						{
// 							WRITE_PORT_UCHAR(SER_FCR(ComPortBase), (SR_FCR_CLEAR_RCVR as i32));
// 							IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 						}
// 					}
// 					KeReleaseSpinLock(&DeviceExtension->InputBufferLock, Irql);
// 				}
//
// 				if (PurgeMask & SERIAL_PURGE_TXCLEAR)
// 				{
// 					KeAcquireSpinLock(&DeviceExtension->OutputBufferLock, &Irql);
// 					DeviceExtension->OutputBuffer.ReadPosition = DeviceExtension->OutputBuffer.WritePosition = 0;
// 					if (DeviceExtension->UartType >= Uart16550A)
// 					{
// 						/* Clear also Uart FIFO */
// 						Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 						if (NT_SUCCESS(Status))
// 						{
// 							WRITE_PORT_UCHAR(SER_FCR(ComPortBase), (SR_FCR_CLEAR_XMIT as i32));
// 							IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 						}
// 					}
// 					KeReleaseSpinLock(&DeviceExtension->OutputBufferLock, Irql);
// 				}
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_RESET_DEVICE:
// 		{
// 			/* FIXME */
// 			ERR_(SERIAL, "IOCTL_SERIAL_RESET_DEVICE not implemented.\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_BAUD_RATE:
// 		{
// 			PULONG pNewBaudRate;
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_SET_BAUD_RATE\n");
// 			if (LengthIn != sizeof(ULONG) || BufferIn == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				pNewBaudRate = (PULONG)BufferIn;
// 				Status = SerialSetBaudRate(DeviceExtension, *pNewBaudRate);
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_BREAK_OFF:
// 		{
// 			/* FIXME */
// 			ERR_(SERIAL, "IOCTL_SERIAL_SET_BREAK_OFF not implemented.\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_BREAK_ON:
// 		{
// 			/* FIXME */
// 			ERR_(SERIAL, "IOCTL_SERIAL_SET_BREAK_ON not implemented.\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_CHARS:
// 		{
// 			/* FIXME */
// 			ERR_(SERIAL, "IOCTL_SERIAL_SET_CHARS not implemented.\n");
// 			Status = STATUS_SUCCESS;
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_DTR:
// 		{
// 			/* FIXME: If the handshake flow control of the device is configured to
// 			 * automatically use DTR, return STATUS_INVALID_PARAMETER */
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_SET_DTR\n");
// 			if (!(DeviceExtension->MCR & (SR_MCR_DTR as i32)))
// 			{
// 				Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 				if (NT_SUCCESS(Status))
// 				{
// 					DeviceExtension->MCR |= (SR_MCR_DTR as i32);
// 					WRITE_PORT_UCHAR(SER_MCR(ComPortBase), DeviceExtension->MCR);
// 					IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 				}
// 			}
// 			else
// 				Status = STATUS_SUCCESS;
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_FIFO_CONTROL:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_SET_FIFO_CONTROL\n");
// 			if (LengthIn != sizeof(ULONG) || BufferIn == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 				if (NT_SUCCESS(Status))
// 				{
// 					WRITE_PORT_UCHAR(SER_FCR(ComPortBase), (UCHAR)((*(PULONG)BufferIn) & 0xff));
// 					IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 				}
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_HANDFLOW:
// 		{
// 			/* FIXME */
// 			ERR_(SERIAL, "IOCTL_SERIAL_SET_HANDFLOW not implemented.\n");
// 			Status = STATUS_SUCCESS;
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_LINE_CONTROL:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_SET_LINE_CONTROL\n");
// 			if (LengthIn < sizeof(SERIAL_LINE_CONTROL))
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 			else if (BufferIn == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 				Status = SerialSetLineControl(DeviceExtension, (PSERIAL_LINE_CONTROL)BufferIn);
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_MODEM_CONTROL:
// 		{
// 			PULONG pMCR;
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_SET_MODEM_CONTROL\n");
// 			if (LengthIn != sizeof(ULONG) || BufferIn == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 				if (NT_SUCCESS(Status))
// 				{
// 					pMCR = (PULONG)BufferIn;
// 					DeviceExtension->MCR = (UCHAR)(*pMCR & 0xff);
// 					WRITE_PORT_UCHAR(SER_MCR(ComPortBase), DeviceExtension->MCR);
// 					IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 				}
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_QUEUE_SIZE:
// 		{
// 			if (LengthIn < sizeof(SERIAL_QUEUE_SIZE ))
// 				return STATUS_BUFFER_TOO_SMALL;
// 			else if (BufferIn == NULL)
// 				return STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				KIRQL Irql;
// 				PSERIAL_QUEUE_SIZE NewQueueSize = (PSERIAL_QUEUE_SIZE)BufferIn;
// 				Status = STATUS_SUCCESS;
// 				if (NewQueueSize->InSize > DeviceExtension->InputBuffer.Length)
// 				{
// 					KeAcquireSpinLock(&DeviceExtension->InputBufferLock, &Irql);
// 					Status = IncreaseCircularBufferSize(&DeviceExtension->InputBuffer, NewQueueSize->InSize);
// 					KeReleaseSpinLock(&DeviceExtension->InputBufferLock, Irql);
// 				}
// 				if (NT_SUCCESS(Status) && NewQueueSize->OutSize > DeviceExtension->OutputBuffer.Length)
// 				{
// 					KeAcquireSpinLock(&DeviceExtension->OutputBufferLock, &Irql);
// 					Status = IncreaseCircularBufferSize(&DeviceExtension->OutputBuffer, NewQueueSize->OutSize);
// 					KeReleaseSpinLock(&DeviceExtension->OutputBufferLock, Irql);
// 				}
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_RTS:
// 		{
// 			/* FIXME: If the handshake flow control of the device is configured to
// 			 * automatically use DTR, return STATUS_INVALID_PARAMETER */
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_SET_RTS\n");
// 			if (!(DeviceExtension->MCR & (SR_MCR_RTS as i32)))
// 			{
// 				Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 				if (NT_SUCCESS(Status))
// 				{
// 					DeviceExtension->MCR |= (SR_MCR_RTS as i32);
// 					WRITE_PORT_UCHAR(SER_MCR(ComPortBase), DeviceExtension->MCR);
// 					IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 				}
// 			}
// 			else
// 				Status = STATUS_SUCCESS;
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_TIMEOUTS:
// 		{
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_SET_TIMEOUTS\n");
// 			if (LengthIn != sizeof(SERIAL_TIMEOUTS) || BufferIn == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				DeviceExtension->SerialTimeOuts = *(PSERIAL_TIMEOUTS)BufferIn;
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_WAIT_MASK:
// 		{
// 			PULONG pWaitMask = (PULONG)BufferIn;
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_SET_WAIT_MASK\n");
//
// 			if (LengthIn != sizeof(ULONG) || BufferIn == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else if (DeviceExtension->WaitOnMaskIrp) /* FIXME: Race condition ; field may be currently in modification */
// 			{
// 				WARN_(SERIAL, "An IRP is already currently processed\n");
// 				Status = STATUS_INVALID_PARAMETER;
// 			}
// 			else
// 			{
// 				DeviceExtension->WaitMask = *pWaitMask;
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_XOFF:
// 		{
// 			/* FIXME */
// 			ERR_(SERIAL, "IOCTL_SERIAL_SET_XOFF not implemented.\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_SERIAL_SET_XON:
// 		{
// 			/* FIXME */
// 			ERR_(SERIAL, "IOCTL_SERIAL_SET_XON not implemented.\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_SERIAL_WAIT_ON_MASK:
// 		{
// 			PIRP WaitingIrp;
// 			TRACE_!(SERIAL, "IOCTL_SERIAL_WAIT_ON_MASK\n");
//
// 			if (LengthOut != sizeof(ULONG) || BufferOut == NULL)
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				IoMarkIrpPending(Irp);
//
// 				WaitingIrp = InterlockedCompareExchangePointer(
// 					(PVOID*)&DeviceExtension->WaitOnMaskIrp,
// 					Irp,
// 					NULL);
//
// 				/* Check if an Irp is already pending */
// 				if (WaitingIrp != NULL)
// 				{
// 					/* Unable to have a 2nd pending IRP for this IOCTL */
// 					WARN_(SERIAL, "Unable to pend a second IRP for IOCTL_SERIAL_WAIT_ON_MASK\n");
// 					Irp->IoStatus.Information = 0;
// 					Irp->IoStatus.Status = STATUS_INVALID_PARAMETER;
// 					IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 				}
// 				return STATUS_PENDING;
// 			}
// 			break;
// 		}
// 		case IOCTL_SERIAL_XOFF_COUNTER:
// 		{
// 			/* FIXME */
// 			ERR_(SERIAL, "IOCTL_SERIAL_XOFF_COUNTER not implemented.\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		default:
// 		{
// 			/* Pass Irp to lower driver */
// 			TRACE_!(SERIAL, "Unknown IOCTL code 0x%x\n", Stack->Parameters.DeviceIoControl.IoControlCode);
// 			IoSkipCurrentIrpStackLocation(Irp);
// 			return IoCallDriver(DeviceExtension->LowerDevice, Irp);
// 		}
// 	}
//
// 	Irp->IoStatus.Status = Status;
// 	if (Status == STATUS_PENDING)
// 	{
// 		IoMarkIrpPending(Irp);
// 	}
// 	else
// 	{
// 		Irp->IoStatus.Information = Information;
// 		IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	}
// 	return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
