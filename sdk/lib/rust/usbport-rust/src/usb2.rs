use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{LIST_ENTRY, PDEVICE_OBJECT};
use core::ffi::c_void;

extern "system" {
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut CMASKS: [UCHAR; 8] = [
    0x1c as ::core::ffi::c_int as UCHAR,
    0x38 as ::core::ffi::c_int as UCHAR,
    0x70 as ::core::ffi::c_int as UCHAR,
    0xe0 as ::core::ffi::c_int as UCHAR,
    0xc1 as ::core::ffi::c_int as UCHAR,
    0x83 as ::core::ffi::c_int as UCHAR,
    0x7 as ::core::ffi::c_int as UCHAR,
    0xe as ::core::ffi::c_int as UCHAR,
];

pub const NDEBUG: () = ();


const USBPORT_TRANSFER_TYPE_ISOCHRONOUS: ULONG = 0;
const USB2_FS_SOF_TIME: ULONG = 6;

#[no_mangle]
pub unsafe extern "system" fn USB2_AllocateCheck(
    OutTimeUsed: PULONG,
    CalcBusTime: ULONG,
    LimitAllocation: ULONG,
) -> BOOLEAN {
    let mut BusTime: ULONG;
    let mut Result: BOOLEAN = 1 as BOOLEAN;

    BusTime = (*OutTimeUsed).wrapping_add(CalcBusTime);
    *OutTimeUsed = (*OutTimeUsed).wrapping_add(CalcBusTime);

    if BusTime > LimitAllocation {
        Result = 0 as BOOLEAN;
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_AddDataBitStuff(DataTime: USHORT) -> USHORT {
    return (DataTime as i32 + DataTime as i32 / 16) as USHORT;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_IncMicroFrame(frame: PUCHAR, uframe: PUCHAR) {
    *uframe = (*uframe).wrapping_add(1);

    if *uframe as i32 > ((USB2_MICROFRAMES - 1) as i32) {
        *uframe = 0;
        *frame = ((*frame as i32 + 1) & ((USB2_FRAMES - 1) as i32)) as u8;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USB2_GetPrevMicroFrame(frame: PUCHAR, uframe: PUCHAR) {
    *uframe = (USB2_MICROFRAMES - 1) as UCHAR;

    if *frame != 0 {
        *frame = (*frame).wrapping_sub(1);
    } else {
        *frame = (USB2_FRAMES - 1) as UCHAR;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USB2_CheckTtEndpointInsert(
    nextTtEndpoint: PUSB2_TT_ENDPOINT,
    TtEndpoint: PUSB2_TT_ENDPOINT,
) -> BOOLEAN {
    let TransferType: ULONG;

    if TtEndpoint.is_null() {
        RtlAssert(
            b"TtEndpoint\0".as_ptr() as PVOID,
            b"usb2.c\0".as_ptr() as PVOID,
            83 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    if (*TtEndpoint).CalcBusTime as i32 >= (USB2_FS_MAX_PERIODIC_ALLOCATION / 2) as i32 {
        return FALSE as BOOLEAN;
    }

    if nextTtEndpoint.is_null() {
        return TRUE as BOOLEAN;
    }

    TransferType = (*TtEndpoint).TtEndpointParams.c2rust_unnamed.TransferType();

    if ((*nextTtEndpoint).ActualPeriod as i32) < (*TtEndpoint).ActualPeriod as i32
        && TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
    {
        return TRUE as BOOLEAN;
    }

    if (*nextTtEndpoint).ActualPeriod as i32 <= (*TtEndpoint).ActualPeriod as i32
        && TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG
        || nextTtEndpoint == TtEndpoint
    {
        return TRUE as BOOLEAN;
    }

    return FALSE as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_GetOverhead(TtEndpoint: PUSB2_TT_ENDPOINT) -> ULONG {
    let TransferType: ULONG;
    let Direction: ULONG;
    let DeviceSpeed: ULONG;
    let Overhead: ULONG;
    let HostDelay: ULONG;

    TransferType = (*TtEndpoint).TtEndpointParams.TransferType();
    Direction = (*TtEndpoint).TtEndpointParams.Direction();
    DeviceSpeed = (*TtEndpoint).TtEndpointParams.DeviceSpeed();

    HostDelay = (*(*(*TtEndpoint).Tt).HcExtension).HcDelayTime;

    if DeviceSpeed == UsbHighSpeed as ULONG {
        if Direction == USBPORT_TRANSFER_DIRECTION_OUT as ULONG {
            if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
                Overhead = HostDelay.wrapping_add(USB2_HS_ISOCHRONOUS_OUT_OVERHEAD as ULONG);
            } else {
                Overhead = HostDelay.wrapping_add(USB2_HS_INTERRUPT_OUT_OVERHEAD as ULONG);
            }
        } else {
            if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
                Overhead = HostDelay.wrapping_add(USB2_HS_ISOCHRONOUS_IN_OVERHEAD as ULONG);
            } else {
                Overhead = HostDelay.wrapping_add(USB2_HS_INTERRUPT_IN_OVERHEAD as ULONG);
            }
        }
    } else if DeviceSpeed == UsbFullSpeed as ULONG {
        if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
            Overhead = HostDelay.wrapping_add(USB2_FS_ISOCHRONOUS_OVERHEAD as ULONG);
        } else {
            Overhead = HostDelay.wrapping_add(USB2_FS_INTERRUPT_OVERHEAD as ULONG);
        }
    } else {
        Overhead = HostDelay.wrapping_add(USB2_LS_INTERRUPT_OVERHEAD as ULONG);
    }

    return Overhead;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_GetHsOverhead(
    TtEndpoint: PUSB2_TT_ENDPOINT,
    OverheadSS: PULONG,
    OverheadCS: PULONG,
) {
    let TransferType: ULONG;
    let Direction: ULONG;
    let HostDelay: ULONG;

    TransferType = (*TtEndpoint).TtEndpointParams.TransferType();
    Direction = (*TtEndpoint).TtEndpointParams.Direction();

    HostDelay = (*(*(*TtEndpoint).Tt).HcExtension).HcDelayTime;

    if Direction == USBPORT_TRANSFER_DIRECTION_OUT {
        if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS {
            *OverheadSS = HostDelay.wrapping_add(USB2_HS_SS_ISOCHRONOUS_OUT_OVERHEAD);
            *OverheadCS = 0;
        } else {
            *OverheadSS = HostDelay.wrapping_add(USB2_HS_SS_INTERRUPT_OUT_OVERHEAD);
            *OverheadCS = HostDelay.wrapping_add(USB2_HS_CS_INTERRUPT_OUT_OVERHEAD);
        }
    } else {
        if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS {
            *OverheadSS = HostDelay.wrapping_add(USB2_HS_SS_ISOCHRONOUS_IN_OVERHEAD);
            *OverheadCS = HostDelay.wrapping_add(USB2_HS_CS_ISOCHRONOUS_IN_OVERHEAD);
        } else {
            *OverheadSS = HostDelay.wrapping_add(USB2_HS_SS_INTERRUPT_IN_OVERHEAD);
            *OverheadCS = HostDelay.wrapping_add(USB2_HS_CS_INTERRUPT_IN_OVERHEAD);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn USB2_GetLastIsoTime(
    TtEndpoint: PUSB2_TT_ENDPOINT,
    Frame: ULONG,
) -> ULONG {
    let mut nextTtEndpoint: PUSB2_TT_ENDPOINT;
    let Result: ULONG;

    nextTtEndpoint = (*(*(*TtEndpoint).Tt).FrameBudget[Frame as usize].IsoEndpoint).NextTtEndpoint
        as PUSB2_TT_ENDPOINT;

    if !nextTtEndpoint.is_null() || {
        nextTtEndpoint = (*(*TtEndpoint).Tt).FrameBudget[Frame as usize].AltEndpoint;
        !nextTtEndpoint.is_null()
    } {
        Result = ((*nextTtEndpoint).StartTime as i32 + (*nextTtEndpoint).CalcBusTime as i32) as ULONG;
    } else {
        Result = USB2_FS_SOF_TIME as ULONG;
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_GetStartTime(
    nextTtEndpoint: PUSB2_TT_ENDPOINT,
    TtEndpoint: PUSB2_TT_ENDPOINT,
    prevTtEndpoint: PUSB2_TT_ENDPOINT,
    Frame: ULONG,
) -> ULONG {
    let mut ttEndpoint: PUSB2_TT_ENDPOINT;
    let mut TransferType: ULONG;

    TransferType = (*TtEndpoint).TtEndpointParams.TransferType();

    if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS {
        if !nextTtEndpoint.is_null() {
            return ((*nextTtEndpoint).StartTime as i32
                + (*nextTtEndpoint).CalcBusTime as i32) as ULONG;
        }

        ttEndpoint = (*(*TtEndpoint).Tt).FrameBudget[Frame as usize].AltEndpoint;

        if !ttEndpoint.is_null() {
            return ((*ttEndpoint).StartTime as i32
                + (*ttEndpoint).CalcBusTime as i32) as ULONG;
        } else {
            return USB2_FS_SOF_TIME;
        }
    } else {
        ttEndpoint = prevTtEndpoint;

        if ttEndpoint == (*(*TtEndpoint).Tt).FrameBudget[Frame as usize].IntEndpoint {
            return USB2_GetLastIsoTime(TtEndpoint, Frame);
        } else {
            return ((*ttEndpoint).StartTime as i32
                + (*ttEndpoint).CalcBusTime as i32) as ULONG;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn USB2_InitTtEndpoint(
    TtEndpoint: PUSB2_TT_ENDPOINT,
    TransferType: UCHAR,
    Direction: UCHAR,
    DeviceSpeed: UCHAR,
    Period: USHORT,
    MaxPacketSize: USHORT,
    Tt: PUSB2_TT,
) {
    (*TtEndpoint).TtEndpointParams.set_TransferType(TransferType as ULONG);
    (*TtEndpoint).TtEndpointParams.set_Direction(Direction as ULONG);
    (*TtEndpoint).TtEndpointParams.set_DeviceSpeed(DeviceSpeed as ULONG);

    (*TtEndpoint).Period = Period;
    (*TtEndpoint).MaxPacketSize = MaxPacketSize;
    (*TtEndpoint).Tt = Tt;

    (*TtEndpoint).CalcBusTime = 0;
    (*TtEndpoint).StartTime = 0;
    (*TtEndpoint).ActualPeriod = 0;
    (*TtEndpoint).StartFrame = 0;
    (*TtEndpoint).StartMicroframe = 0;

    (*TtEndpoint).Nums.AsULONG = 0;
    (*TtEndpoint).NextTtEndpoint = core::ptr::null_mut();
    (*TtEndpoint).Reserved2 = 0;
    (*TtEndpoint).PreviosPeriod = 0;
    (*TtEndpoint).IsPromoted = 0;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_AllocateHS(
    TtEndpoint: PUSB2_TT_ENDPOINT,
    Frame: LONG,
) -> BOOLEAN {
    let mut HcExtension: PUSB2_HC_EXTENSION;
    let mut Tt: PUSB2_TT;
    let mut TransferType: ULONG;
    let mut Direction: ULONG;
    let mut DataTime: ULONG;
    let mut DataSize: ULONG;
    let mut RemainDataTime: ULONG;
    let mut OverheadCS: ULONG = 0;
    let mut OverheadSS: ULONG = 0;
    let mut ix: ULONG;
    let mut PktSize: USHORT;
    let mut PktSizeBitStuff: USHORT;
    let mut frame: UCHAR;
    let mut uframe: UCHAR;
    let mut Result: BOOL = 1;

    Tt = (*TtEndpoint).Tt as PUSB2_TT;
    HcExtension = (*Tt).HcExtension as PUSB2_HC_EXTENSION;

    TransferType = (*TtEndpoint).TtEndpointParams.c2rust_unnamed.TransferType();
    Direction = (*TtEndpoint).TtEndpointParams.c2rust_unnamed.Direction();

    if Frame == 0 {
        (*TtEndpoint).StartMicroframe =
            ((*TtEndpoint).StartTime as i32 / (USB2_FS_RAW_BYTES_IN_MICROFRAME as i32) - 1) as UCHAR;
    }

    USB2_GetHsOverhead(
        TtEndpoint,
        core::ptr::addr_of_mut!(OverheadSS),
        core::ptr::addr_of_mut!(OverheadCS),
    );

    if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG {
        if Frame == 0 {
            (*TtEndpoint).Nums.c2rust_unnamed.set_NumStarts(1 as ULONG);

            if ((*TtEndpoint).StartMicroframe as CHAR as i32) < ((USB2_MICROFRAMES - 3) as i32) {
                (*TtEndpoint).Nums.c2rust_unnamed.set_NumCompletes(3 as ULONG);
            } else {
                (*TtEndpoint).Nums.c2rust_unnamed.set_NumCompletes(2 as ULONG);
            }
        }
    } else {
        if Direction == USBPORT_TRANSFER_DIRECTION_OUT as ULONG {
            // ISO UNIMPLEMENTED
            // ASSERT(FALSE);
        } else {
            // ISO UNIMPLEMENTED
            // ASSERT(FALSE);
        }
    }

    frame = ((*TtEndpoint).StartFrame as LONG + Frame) as UCHAR;
    uframe = (*TtEndpoint).StartMicroframe;

    if (*TtEndpoint).StartMicroframe as i32 == (USB2_PREV_MICROFRAME as i32) {
        USB2_GetPrevMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
    }

    ix = 0;
    while ix < (*TtEndpoint).Nums.c2rust_unnamed.NumStarts() {
        if USB2_AllocateCheck(
            (&raw mut *(core::ptr::addr_of_mut!((*HcExtension).TimeUsed) as *mut [ULONG; 8])
                .offset(frame as isize) as *mut ULONG)
                .offset(uframe as isize) as PULONG,
            OverheadSS,
            USB2_MAX_MICROFRAME_ALLOCATION as ULONG,
        ) == 0
        {
            Result = 0;
        }

        if (*Tt).NumStartSplits[frame as usize][uframe as usize]
            > (USB2_MAX_FS_LS_TRANSACTIONS_IN_UFRAME - 1) as ULONG
        {
            // DPRINT1: Num Start Splits
            // ASSERT(FALSE);
            Result = 0;
        }

        (*Tt).NumStartSplits[frame as usize][uframe as usize] =
            (*Tt).NumStartSplits[frame as usize][uframe as usize].wrapping_add(1);
        USB2_IncMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
        ix = ix.wrapping_add(1);
    }

    frame = ((*TtEndpoint).StartFrame as LONG + Frame) as UCHAR;
    uframe = ((*TtEndpoint).StartMicroframe as i32
        + (*TtEndpoint).Nums.c2rust_unnamed.NumStarts() as i32
        + 1) as UCHAR;

    ix = 0;
    while ix < (*TtEndpoint).Nums.c2rust_unnamed.NumCompletes() {
        if USB2_AllocateCheck(
            (&raw mut *(core::ptr::addr_of_mut!((*HcExtension).TimeUsed) as *mut [ULONG; 8])
                .offset(frame as isize) as *mut ULONG)
                .offset(uframe as isize) as PULONG,
            OverheadCS,
            USB2_MAX_MICROFRAME_ALLOCATION as ULONG,
        ) == 0
        {
            Result = 0;
        }

        USB2_IncMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
        ix = ix.wrapping_add(1);
    }

    PktSize = (*TtEndpoint).MaxPacketSize;
    PktSizeBitStuff = USB2_AddDataBitStuff(PktSize);

    if Direction == USBPORT_TRANSFER_DIRECTION_OUT as ULONG {
        frame = ((*TtEndpoint).StartFrame as LONG + Frame) as UCHAR;
        uframe = (*TtEndpoint).StartMicroframe;

        if uframe as i32 == (USB2_PREV_MICROFRAME as i32) {
            USB2_GetPrevMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
        }

        DataTime = 0;

        ix = 0;
        while ix < (*TtEndpoint).Nums.c2rust_unnamed.NumStarts() {
            DataSize = (PktSizeBitStuff as ULONG).wrapping_sub(DataTime);

            if DataSize <= USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG {
                DataTime = DataSize;
            } else {
                DataTime = USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG;
            }

            if USB2_AllocateCheck(
                (&raw mut *(core::ptr::addr_of_mut!((*HcExtension).TimeUsed) as *mut [ULONG; 8])
                    .offset(frame as isize) as *mut ULONG)
                    .offset(uframe as isize) as PULONG,
                DataTime,
                USB2_MAX_MICROFRAME_ALLOCATION as ULONG,
            ) == 0
            {
                Result = 0;
            }

            USB2_IncMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
            DataTime = DataTime.wrapping_add(USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG);
            ix = ix.wrapping_add(1);
        }
    } else {
        frame = ((*TtEndpoint).StartFrame as LONG + Frame) as UCHAR;
        uframe = ((*TtEndpoint).StartMicroframe as i32
            + (*TtEndpoint).Nums.c2rust_unnamed.NumStarts() as i32
            + 1) as UCHAR;

        ix = 0;
        while ix < (*TtEndpoint).Nums.c2rust_unnamed.NumCompletes() {
            if (*Tt).TimeCS[frame as usize][uframe as usize]
                < USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG
            {
                RemainDataTime = (USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG)
                    .wrapping_sub((*Tt).TimeCS[frame as usize][uframe as usize]);

                if RemainDataTime >= PktSizeBitStuff as ULONG {
                    DataTime = PktSizeBitStuff as ULONG;
                } else if RemainDataTime > 0 {
                    DataTime = RemainDataTime;
                } else {
                    DataTime = 0;
                }

                if USB2_AllocateCheck(
                    (&raw mut *(core::ptr::addr_of_mut!((*HcExtension).TimeUsed)
                        as *mut [ULONG; 8])
                        .offset(frame as isize) as *mut ULONG)
                        .offset(uframe as isize) as PULONG,
                    DataTime,
                    USB2_MAX_MICROFRAME_ALLOCATION as ULONG,
                ) == 0
                {
                    Result = 0;
                }
            }

            if (PktSizeBitStuff as i32) < (USB2_FS_RAW_BYTES_IN_MICROFRAME as i32) {
                (*Tt).TimeCS[frame as usize][uframe as usize] = (*Tt).TimeCS[frame as usize]
                    [uframe as usize]
                    .wrapping_add(PktSizeBitStuff as ULONG);
            } else {
                (*Tt).TimeCS[frame as usize][uframe as usize] = (*Tt).TimeCS[frame as usize]
                    [uframe as usize]
                    .wrapping_add(USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG);
            }

            USB2_IncMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
            ix = ix.wrapping_add(1);
        }
    }

    return Result as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_DeallocateHS(TtEndpoint: PUSB2_TT_ENDPOINT, Frame: ULONG) {
    let mut Tt: PUSB2_TT;
    let mut HcExtension: PUSB2_HC_EXTENSION;
    let mut OverheadCS: ULONG = 0;
    let mut OverheadSS: ULONG = 0;
    let mut Direction: ULONG;
    let mut ix: ULONG;
    let mut CurrentDataTime: ULONG;
    let mut RemainDataTime: ULONG;
    let mut DataTime: ULONG;
    let mut DataSize: ULONG;
    let mut PktSize: USHORT;
    let mut PktSizeBitStuff: USHORT;
    let mut uframe: UCHAR;
    let mut frame: UCHAR;

    Tt = (*TtEndpoint).Tt as PUSB2_TT;
    HcExtension = (*Tt).HcExtension as PUSB2_HC_EXTENSION;

    USB2_GetHsOverhead(
        TtEndpoint,
        core::ptr::addr_of_mut!(OverheadSS),
        core::ptr::addr_of_mut!(OverheadCS),
    );

    frame = ((*TtEndpoint).StartFrame as ULONG).wrapping_add(Frame) as UCHAR;
    uframe = (*TtEndpoint).StartMicroframe;

    if (*TtEndpoint).StartMicroframe as i32 == (USB2_PREV_MICROFRAME as i32) {
        USB2_GetPrevMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
    }

    ix = 0;
    while ix < (*TtEndpoint).Nums.NumStarts() {
        (*HcExtension).TimeUsed[frame as usize][uframe as usize] =
            ((*HcExtension).TimeUsed[frame as usize][uframe as usize] as ULONG)
                .wrapping_sub(OverheadSS) as ULONG;
        (*Tt).NumStartSplits[frame as usize][uframe as usize] =
            (*Tt).NumStartSplits[frame as usize][uframe as usize].wrapping_sub(1);
        USB2_IncMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
        ix = ix.wrapping_add(1);
    }

    frame = ((*TtEndpoint).StartFrame as ULONG).wrapping_add(Frame) as UCHAR;
    uframe = ((*TtEndpoint).StartMicroframe as i32
        + (*TtEndpoint).Nums.NumStarts() as i32
        + 1) as UCHAR;

    ix = 0;
    while ix < (*TtEndpoint).Nums.NumCompletes() {
        (*HcExtension).TimeUsed[frame as usize][uframe as usize] =
            ((*HcExtension).TimeUsed[frame as usize][uframe as usize] as ULONG)
                .wrapping_sub(OverheadCS) as ULONG;
        USB2_IncMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
        ix = ix.wrapping_add(1);
    }

    Direction = (*TtEndpoint).TtEndpointParams.Direction();
    PktSize = (*TtEndpoint).MaxPacketSize;
    PktSizeBitStuff = USB2_AddDataBitStuff(PktSize);

    if Direction == USBPORT_TRANSFER_DIRECTION_OUT as ULONG {
        frame = ((*TtEndpoint).StartFrame as ULONG).wrapping_add(Frame) as UCHAR;
        uframe = (*TtEndpoint).StartMicroframe;

        if (*TtEndpoint).StartMicroframe as i32 == (USB2_PREV_MICROFRAME as i32) {
            USB2_GetPrevMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
        }

        DataTime = 0;

        ix = 0;
        while ix < (*TtEndpoint).Nums.NumStarts() {
            DataSize = (PktSizeBitStuff as ULONG).wrapping_sub(DataTime);

            if DataSize <= USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG {
                CurrentDataTime = (PktSizeBitStuff as ULONG).wrapping_sub(DataTime);
            } else {
                CurrentDataTime = USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG;
            }

            (*HcExtension).TimeUsed[frame as usize][uframe as usize] =
                ((*HcExtension).TimeUsed[frame as usize][uframe as usize] as ULONG)
                    .wrapping_sub(CurrentDataTime) as ULONG;
            USB2_IncMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
            DataTime = (DataTime).wrapping_add(USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG) as ULONG;
            ix = ix.wrapping_add(1);
        }
    } else {
        frame = ((*TtEndpoint).StartFrame as ULONG).wrapping_add(Frame) as UCHAR;
        uframe = ((*TtEndpoint).StartMicroframe as i32
            + (*TtEndpoint).Nums.NumStarts() as i32
            + 1) as UCHAR;

        ix = 0;
        while ix < (*TtEndpoint).Nums.NumCompletes() {
            if PktSizeBitStuff as i32 >= (USB2_FS_RAW_BYTES_IN_MICROFRAME as i32) {
                CurrentDataTime = USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG;
            } else {
                CurrentDataTime = PktSizeBitStuff as ULONG;
            }

            (*Tt).TimeCS[frame as usize][uframe as usize] =
                ((*Tt).TimeCS[frame as usize][uframe as usize] as ULONG)
                    .wrapping_sub(CurrentDataTime) as ULONG;

            if (*Tt).TimeCS[frame as usize][uframe as usize] < USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG {
                RemainDataTime = (USB2_FS_RAW_BYTES_IN_MICROFRAME as ULONG)
                    .wrapping_sub((*Tt).TimeCS[frame as usize][uframe as usize]);

                if RemainDataTime >= PktSizeBitStuff as ULONG {
                    RemainDataTime = PktSizeBitStuff as ULONG;
                }

                (*HcExtension).TimeUsed[frame as usize][uframe as usize] =
                    ((*HcExtension).TimeUsed[frame as usize][uframe as usize] as ULONG)
                        .wrapping_sub(RemainDataTime) as ULONG;
            }

            USB2_IncMicroFrame(core::ptr::addr_of_mut!(frame), core::ptr::addr_of_mut!(uframe));
            ix = ix.wrapping_add(1);
        }
    }

    return;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_MoveTtEndpoint(
    TtEndpoint: PUSB2_TT_ENDPOINT,
    BusTime: LONG,
    Rebalance: PUSB2_REBALANCE,
    RebalanceListEntries: ULONG,
    mut OutResult: *mut BOOLEAN,
) -> BOOLEAN {
    let mut EndBusTime: ULONG;
    let TransferType: ULONG;
    let mut Num: ULONG;
    let mut ix: UCHAR;

    *OutResult = TRUE as BOOLEAN;

    Num = 0 as ULONG;
    while !(*Rebalance).RebalanceEndpoint[Num as usize].is_null() {
        if (*Rebalance).RebalanceEndpoint[Num as usize] == TtEndpoint {
            break;
        }
        Num = Num.wrapping_add(1);
    }

    TransferType = (*TtEndpoint).TtEndpointParams.c2rust_unnamed.TransferType();

    if !(*Rebalance).RebalanceEndpoint[Num as usize].is_null()
        && (*TtEndpoint).TtEndpointParams.c2rust_unnamed.EndpointMoved() as i32 != 0
        && (TransferType != USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG || BusTime >= 0)
    {
        return FALSE as BOOLEAN;
    }

    ix = 0 as UCHAR;
    while ((*TtEndpoint).StartFrame as i32 + ix as i32) < (USB2_FRAMES as i32) {
        USB2_DeallocateHS(TtEndpoint, ix as ULONG);
        ix = (ix as i32 + (*TtEndpoint).ActualPeriod as i32) as UCHAR;
    }

    (*TtEndpoint).StartTime =
        ((*TtEndpoint).StartTime as i32 + BusTime as i32) as USHORT;

    EndBusTime = ((*TtEndpoint).StartTime as i32 + (*TtEndpoint).CalcBusTime as i32) as ULONG;

    if EndBusTime > USB2_FS_MAX_PERIODIC_ALLOCATION as ULONG {
        *OutResult = FALSE as BOOLEAN;
    }

    (*TtEndpoint).TtEndpointParams.c2rust_unnamed.set_EndpointMoved(1 as ULONG);

    if (*Rebalance).RebalanceEndpoint[Num as usize].is_null() {
        if Num >= RebalanceListEntries {
            *OutResult = FALSE as BOOLEAN;
        } else {
            (*Rebalance).RebalanceEndpoint[Num as usize] = TtEndpoint;
            (*Rebalance).RebalanceEndpoint[(Num as ULONG).wrapping_add(1) as usize] =
                core::ptr::null_mut();
        }
    }

    ix = 0 as UCHAR;
    while ((*TtEndpoint).StartFrame as i32 + ix as i32) < (USB2_FRAMES as i32) {
        if USB2_AllocateHS(TtEndpoint, ix as LONG) == 0 {
            OutResult = core::ptr::null_mut();
        }
        ix = (ix as i32 + (*TtEndpoint).ActualPeriod as i32) as UCHAR;
    }

    return TRUE as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_CommonFrames(
    NextTtEndpoint: PUSB2_TT_ENDPOINT,
    TtEndpoint: PUSB2_TT_ENDPOINT,
) -> BOOLEAN {
    let mut Frame: UCHAR;

    if (*NextTtEndpoint).ActualPeriod as i32 == (ENDPOINT_INTERRUPT_1ms as i32)
        || (*TtEndpoint).ActualPeriod as i32 == (ENDPOINT_INTERRUPT_1ms as i32)
    {
        return TRUE as BOOLEAN;
    }

    if ((*NextTtEndpoint).ActualPeriod as i32) < (*TtEndpoint).ActualPeriod as i32 {
        Frame = ((*TtEndpoint).StartFrame as i32 % (*TtEndpoint).ActualPeriod as i32) as UCHAR;
    } else {
        Frame = ((*NextTtEndpoint).StartFrame as i32 % (*TtEndpoint).ActualPeriod as i32) as UCHAR;
    }

    return (Frame as i32 == (*TtEndpoint).StartFrame as i32) as i32 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_ConvertFrame(
    Frame: UCHAR,
    Microframe: UCHAR,
    HcFrame: PUCHAR,
    HcMicroframe: PUCHAR,
) {
    if Microframe as i32 == (USB2_PREV_MICROFRAME as i32) {
        *HcFrame = Frame as u8;
        *HcMicroframe = 0 as u8;
    }

    if Microframe as i32 <= (((USB2_MICROFRAMES - (2 as i32 as i32) as u32)) as i32) {
        *HcFrame = Frame as u8;
        *HcMicroframe = (Microframe as i32 + 1) as u8;
    }

    if Microframe as i32 == (((USB2_MICROFRAMES - (1 as i32 as i32) as u32)) as i32) {
        *HcFrame = (Frame as i32 + 1) as u8;
        *HcMicroframe = 0 as u8;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USB2_GetSMASK(TtEndpoint: PUSB2_TT_ENDPOINT) -> UCHAR {
    let mut ix: ULONG;
    let mut SMask: UCHAR = 0;
    let mut HcFrame: UCHAR = 0;
    let mut HcMicroFrame: UCHAR = 0;

    if (*TtEndpoint).TtEndpointParams.DeviceSpeed() as i32 == UsbHighSpeed as i32 {
        SMask = (1 << (*TtEndpoint).StartMicroframe as i32) as UCHAR;
    } else {
        USB2_ConvertFrame(
            (*TtEndpoint).StartFrame,
            (*TtEndpoint).StartMicroframe,
            core::ptr::addr_of_mut!(HcFrame),
            core::ptr::addr_of_mut!(HcMicroFrame),
        );

        ix = 0;
        while ix < (*TtEndpoint).Nums.NumStarts() {
            SMask = (SMask as i32 | (1 << HcMicroFrame as i32)) as UCHAR;
            HcMicroFrame = HcMicroFrame.wrapping_add(1);
            ix = ix.wrapping_add(1);
        }
    }

    return SMask;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_GetCMASK(TtEndpoint: PUSB2_TT_ENDPOINT) -> UCHAR {
    let mut NumCompletes: ULONG;
    let TransferType: ULONG;
    let DeviceSpeed: ULONG;
    let Direction: ULONG;
    let Result: UCHAR;
    let mut MicroFrameCS: UCHAR;
    let mut HcFrame: UCHAR = 0;
    let mut HcMicroFrame: UCHAR = 0;
    let mut MaskCS: UCHAR = 0;

    TransferType = (*TtEndpoint).TtEndpointParams.TransferType();
    DeviceSpeed = (*TtEndpoint).TtEndpointParams.DeviceSpeed();
    Direction = (*TtEndpoint).TtEndpointParams.Direction();

    if DeviceSpeed == UsbHighSpeed as ULONG {
        return 0;
    }

    if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG {
        USB2_ConvertFrame(
            (*TtEndpoint).StartFrame,
            (*TtEndpoint).StartMicroframe,
            core::ptr::addr_of_mut!(HcFrame),
            core::ptr::addr_of_mut!(HcMicroFrame),
        );

        Result = CMASKS[HcMicroFrame as usize];
    } else {
        if Direction == USBPORT_TRANSFER_DIRECTION_OUT as ULONG {
            return 0;
        }

        USB2_ConvertFrame(
            (*TtEndpoint).StartFrame,
            (*TtEndpoint).StartMicroframe,
            core::ptr::addr_of_mut!(HcFrame),
            core::ptr::addr_of_mut!(HcMicroFrame),
        );

        NumCompletes = (*TtEndpoint).Nums.NumCompletes();

        MicroFrameCS = (HcMicroFrame as i32 + 2) as UCHAR;
        while (MicroFrameCS as i32) < (USB2_MICROFRAMES as i32) {
            MaskCS = (MaskCS as i32 | (1i32 << MicroFrameCS as i32)) as UCHAR;
            NumCompletes = NumCompletes.wrapping_sub(1);

            if NumCompletes == 0 {
                return MaskCS;
            }

            MicroFrameCS = MicroFrameCS.wrapping_add(1);
        }

        while NumCompletes != 0 {
            MaskCS = (MaskCS as i32 | (1i32 << (MicroFrameCS as i32 - (USB2_MICROFRAMES as i32)))) as UCHAR;
            NumCompletes = NumCompletes.wrapping_sub(1);
        }

        Result = MaskCS;
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_RebalanceEndpoint(
    FdoDevice: PDEVICE_OBJECT,
    List: PLIST_ENTRY,
) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut Entry: PLIST_ENTRY;
    let mut AllocedBusTime: ULONG;
    let mut EndpointBandwidth: ULONG;
    let mut Factor: ULONG;
    let mut ScheduleOffset: ULONG;
    let mut Bandwidth: ULONG;
    let mut n: ULONG;
    let mut ix: ULONG;
    let mut OldIrql: KIRQL = 0;
    let mut NewPeriod: UCHAR;
    let mut SMask: UCHAR;
    let mut CMask: UCHAR;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    while !IsListEmpty(List) {
        Entry = RemoveHeadList(List);

        Endpoint = ((Entry as usize)
            - (core::mem::offset_of!(USBPORT_ENDPOINT, RebalanceLink.Flink))) as *mut USBPORT_ENDPOINT as PUSBPORT_ENDPOINT;

        (*Endpoint).RebalanceLink.Flink = core::ptr::null_mut();
        (*Endpoint).RebalanceLink.Blink = core::ptr::null_mut();

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
        );

        SMask = USB2_GetSMASK((*Endpoint).TtEndpoint as PUSB2_TT_ENDPOINT);
        CMask = USB2_GetCMASK((*Endpoint).TtEndpoint as PUSB2_TT_ENDPOINT);

        ScheduleOffset = (*(*Endpoint).TtEndpoint).StartFrame as ULONG;
        NewPeriod = (*(*Endpoint).TtEndpoint).ActualPeriod as UCHAR;

        AllocedBusTime = (*(*Endpoint).TtEndpoint).CalcBusTime as ULONG;
        EndpointBandwidth = (USB2_MICROFRAMES as ULONG).wrapping_mul(AllocedBusTime);

        (*Endpoint).EndpointProperties.InterruptScheduleMask = SMask;
        (*Endpoint).EndpointProperties.SplitCompletionMask = CMask;

        if (*Endpoint).EndpointProperties.Period as i32 != NewPeriod as i32 {
            Factor = (USB2_FRAMES / ((*Endpoint).EndpointProperties.Period as i32 as u32)) as ULONG;

            ix = 0;
            while ix < Factor {
                Bandwidth = (*Endpoint).EndpointProperties.UsbBandwidth;
                n = Factor.wrapping_mul((*Endpoint).EndpointProperties.ScheduleOffset);
                (*(*Endpoint).TtExtension).Bandwidth[n.wrapping_add(ix) as usize] =
                    (*(*Endpoint).TtExtension).Bandwidth[n.wrapping_add(ix) as usize]
                        .wrapping_add(Bandwidth);
                ix = ix.wrapping_add(1);
            }

            (*Endpoint).EndpointProperties.Period = NewPeriod;
            (*Endpoint).EndpointProperties.ScheduleOffset = ScheduleOffset;
            (*Endpoint).EndpointProperties.UsbBandwidth = EndpointBandwidth;

            Factor = (USB2_FRAMES / (NewPeriod as i32 as u32)) as ULONG;

            ix = 0;
            while ix < Factor {
                n = Factor.wrapping_mul(ScheduleOffset);
                (*(*Endpoint).TtExtension).Bandwidth[n.wrapping_add(ix) as usize] =
                    (*(*Endpoint).TtExtension).Bandwidth[n.wrapping_add(ix) as usize]
                        .wrapping_add(EndpointBandwidth);
                ix = ix.wrapping_add(1);
            }
        }

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );

        ((*Packet).RebalanceEndpoint.expect("non-null function pointer"))(
            (*FdoExtension).MiniPortExt,
            core::ptr::addr_of_mut!((*Endpoint).EndpointProperties),
            (Endpoint as usize).wrapping_add(core::mem::size_of::<USBPORT_ENDPOINT>()) as PVOID,
        );

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            OldIrql,
        );

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            (*Endpoint).EndpointOldIrql,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USB2_Rebalance(
    FdoDevice: PDEVICE_OBJECT,
    RebalanceList: PLIST_ENTRY,
) {
    const USBPORT_TRANSFER_TYPE_ISOCHRONOUS: ULONG = 0;
    const USBPORT_TRANSFER_TYPE_INTERRUPT: ULONG = 3;

    let mut EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES =
        core::ptr::null_mut();
    let mut Endpoint: PUSBPORT_ENDPOINT = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut BalanceListInt1: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut BalanceListInt2: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut TransferType: ULONG;
    let mut ScheduleOffset: ULONG;
    let mut SMask: UCHAR;
    let mut CMask: UCHAR;
    let mut ActualPeriod: UCHAR;

    InitializeListHead(core::ptr::addr_of_mut!(BalanceListInt1));
    InitializeListHead(core::ptr::addr_of_mut!(BalanceListInt2));

    while !IsListEmpty(RebalanceList) {
        Entry = (*RebalanceList).Flink;

        Endpoint = (Entry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(USBPORT_ENDPOINT, RebalanceLink.Flink) as ULONG_PTR,
        ) as *mut USBPORT_ENDPOINT;

        RemoveHeadList(RebalanceList);
        (*Entry).Flink = core::ptr::null_mut();
        (*Entry).Blink = core::ptr::null_mut();

        SMask = USB2_GetSMASK((*Endpoint).TtEndpoint);
        CMask = USB2_GetCMASK((*Endpoint).TtEndpoint);

        ScheduleOffset = (*(*Endpoint).TtEndpoint).StartFrame as ULONG;
        ActualPeriod = (*(*Endpoint).TtEndpoint).ActualPeriod as UCHAR;

        EndpointProperties = core::ptr::addr_of_mut!((*Endpoint).EndpointProperties);
        TransferType = (*EndpointProperties).TransferType;

        match TransferType {
            USBPORT_TRANSFER_TYPE_ISOCHRONOUS => {
                if 0 == 0 {
                    RtlAssert(
                        b"FALSE\0".as_ptr() as PVOID,
                        b"drivers/usb/usbport/usb2.c\0".as_ptr() as PVOID,
                        992 as ULONG,
                        core::ptr::null_mut(),
                    );
                }
            }
            USBPORT_TRANSFER_TYPE_INTERRUPT => {
                if SMask as i32 != (*EndpointProperties).InterruptScheduleMask as i32
                    || CMask as i32 != (*EndpointProperties).SplitCompletionMask as i32
                    || ScheduleOffset != (*EndpointProperties).ScheduleOffset
                    || ActualPeriod as i32 != (*EndpointProperties).Period as i32
                {
                    if ActualPeriod as i32 == (*EndpointProperties).Period as i32
                        && ScheduleOffset == (*EndpointProperties).ScheduleOffset
                    {
                        InsertTailList(core::ptr::addr_of_mut!(BalanceListInt1), Entry);
                    } else {
                        InsertTailList(core::ptr::addr_of_mut!(BalanceListInt2), Entry);
                    }
                }
            }
            _ => {
                if 0 == 0 {
                    RtlAssert(
                        b"FALSE\0".as_ptr() as PVOID,
                        b"drivers/usb/usbport/usb2.c\0".as_ptr() as PVOID,
                        1014 as ULONG,
                        core::ptr::null_mut(),
                    );
                }
            }
        }
    }

    USB2_RebalanceEndpoint(FdoDevice, core::ptr::addr_of_mut!(BalanceListInt2));
    USB2_RebalanceEndpoint(FdoDevice, core::ptr::addr_of_mut!(BalanceListInt1));
}

#[no_mangle]
pub unsafe extern "system" fn USB2_DeallocateEndpointBudget(
    TtEndpoint: PUSB2_TT_ENDPOINT,
    Rebalance: PUSB2_REBALANCE,
    RebalanceListEntries: PULONG,
    MaxFrames: ULONG,
) -> BOOLEAN {
    let mut Tt: PUSB2_TT;
    let mut HcExtension: PUSB2_HC_EXTENSION;
    let mut Speed: ULONG;
    let mut TransferType: ULONG;
    let mut Frame: ULONG;
    let mut StartMicroframe: ULONG;
    let mut ix: ULONG;
    let mut endpoint: PUSB2_TT_ENDPOINT;
    let mut nextEndpoint: PUSB2_TT_ENDPOINT;
    let mut lastEndpoint: PUSB2_TT_ENDPOINT;
    let mut tmpEndpoint: PUSB2_TT_ENDPOINT;
    let mut endTime: ULONG;
    let mut maxEndTime: ULONG;
    let mut lastEndTime: ULONG;
    let mut Factor: ULONG;
    let mut jx: ULONG;
    let mut frame: UCHAR;
    let mut uframe: UCHAR;
    let mut Period: USHORT;
    let mut IsMoved: BOOLEAN = 0;

    if (*TtEndpoint).CalcBusTime as i32 == 0 {
        return 0;
    }

    Tt = (*TtEndpoint).Tt as PUSB2_TT;
    HcExtension = (*Tt).HcExtension as PUSB2_HC_EXTENSION;

    Speed = (*TtEndpoint).TtEndpointParams.c2rust_unnamed.DeviceSpeed();

    StartMicroframe = ((*TtEndpoint).StartFrame as i32 * (USB2_MICROFRAMES as i32)
        + (*TtEndpoint).StartMicroframe as i32) as ULONG;

    if Speed == UsbHighSpeed as ULONG {
        ix = StartMicroframe;
        while ix < USB2_MAX_MICROFRAMES as ULONG {
            frame = (ix / USB2_MICROFRAMES as ULONG) as UCHAR;
            uframe = (ix % ((USB2_MICROFRAMES - 1) as ULONG)) as UCHAR;

            (*HcExtension).TimeUsed[frame as usize][uframe as usize] =
                ((*HcExtension).TimeUsed[frame as usize][uframe as usize] as ULONG)
                    .wrapping_sub((*TtEndpoint).CalcBusTime as ULONG) as ULONG;

            ix = ix.wrapping_add((*TtEndpoint).ActualPeriod as ULONG);
        }

        (*TtEndpoint).CalcBusTime = 0 as USHORT;

        return 1;
    }

    TransferType = (*TtEndpoint).TtEndpointParams.c2rust_unnamed.TransferType();

    ix = MaxFrames;
    Frame = (MaxFrames)
        .wrapping_sub(1)
        .wrapping_sub((*TtEndpoint).StartFrame as ULONG) as ULONG;
    while ix > 0 {
        frame = ((*TtEndpoint).StartFrame as ULONG).wrapping_add(Frame) as UCHAR;

        if (Frame).wrapping_rem((*TtEndpoint).ActualPeriod as ULONG) == 0 {
            USB2_DeallocateHS(TtEndpoint, Frame);
            (*Tt).FrameBudget[frame as usize].TimeUsed =
                ((*Tt).FrameBudget[frame as usize].TimeUsed as i32
                    - (*TtEndpoint).CalcBusTime as i32) as USHORT;
        }

        if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG {
            endpoint = (*Tt).FrameBudget[frame as usize].IntEndpoint;
        } else {
            endpoint = (*Tt).FrameBudget[frame as usize].IsoEndpoint;
        }

        nextEndpoint = (*endpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;

        if (*TtEndpoint).CalcBusTime as i32 > (((USB2_FS_MAX_PERIODIC_ALLOCATION / (2 as i32 as i32) as u32)) as i32) {
            while !nextEndpoint.is_null() {
                endpoint = nextEndpoint;
                nextEndpoint = (*nextEndpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;
            }

            nextEndpoint = TtEndpoint;
        } else {
            while !nextEndpoint.is_null()
                && USB2_CheckTtEndpointInsert(nextEndpoint, TtEndpoint) == 0
            {
                endpoint = nextEndpoint;
                nextEndpoint = (*nextEndpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;
            }

            if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG
                && !nextEndpoint.is_null()
            {
                DbgBreakPoint();
            }
        }

        if (Frame).wrapping_rem((*TtEndpoint).ActualPeriod as ULONG) == 0 {
            if (*TtEndpoint).CalcBusTime as i32 > (((USB2_FS_MAX_PERIODIC_ALLOCATION / (2 as i32 as i32) as u32)) as i32) {
                (*Tt).FrameBudget[frame as usize].AltEndpoint = core::ptr::null_mut();
            } else if !nextEndpoint.is_null() {
                nextEndpoint = (*nextEndpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;
                (*endpoint).NextTtEndpoint = nextEndpoint;
            }
        }

        if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG {
            if endpoint == (*Tt).FrameBudget[frame as usize].IntEndpoint {
                if !(*(*Tt).FrameBudget[frame as usize].IsoEndpoint)
                    .NextTtEndpoint
                    .is_null()
                {
                    endpoint = (*(*Tt).FrameBudget[frame as usize].IsoEndpoint).NextTtEndpoint
                        as PUSB2_TT_ENDPOINT;
                } else if !(*Tt).FrameBudget[frame as usize].AltEndpoint.is_null() {
                    endpoint = (*Tt).FrameBudget[frame as usize].AltEndpoint;
                }
            }
        } else {
            DbgBreakPoint();
        }

        Period = (*TtEndpoint).ActualPeriod;

        while !nextEndpoint.is_null() {
            endTime = ((*endpoint).StartTime as i32 + (*endpoint).CalcBusTime as i32) as ULONG;
            maxEndTime = endTime;

            if Period as i32 > (*nextEndpoint).ActualPeriod as i32
                || (*TtEndpoint).StartFrame as i32 != (*nextEndpoint).StartFrame as i32
            {
                if USB2_CommonFrames(nextEndpoint, TtEndpoint) != 0 {
                    Factor = (Period as i32 / (*nextEndpoint).ActualPeriod as i32) as ULONG;
                } else {
                    Factor = (USB2_FRAMES / ((*nextEndpoint).ActualPeriod as i32 as u32)) as ULONG;
                }

                maxEndTime = endTime;

                jx = 0;
                frame = (*nextEndpoint).StartFrame;
                while jx < Factor {
                    if (*nextEndpoint).StartFrame as i32 != (*TtEndpoint).StartFrame as i32 {
                        lastEndpoint = (*Tt).FrameBudget[frame as usize].IntEndpoint;

                        if !(*(*Tt).FrameBudget[frame as usize].IsoEndpoint)
                            .NextTtEndpoint
                            .is_null()
                        {
                            lastEndpoint = (*(*Tt).FrameBudget[frame as usize].IsoEndpoint)
                                .NextTtEndpoint
                                as PUSB2_TT_ENDPOINT;
                        } else if !(*Tt).FrameBudget[frame as usize].AltEndpoint.is_null() {
                            lastEndpoint = (*Tt).FrameBudget[frame as usize].AltEndpoint;
                        }

                        tmpEndpoint = (*(*Tt).FrameBudget[frame as usize].IntEndpoint)
                            .NextTtEndpoint
                            as PUSB2_TT_ENDPOINT;
                        while !tmpEndpoint.is_null() && tmpEndpoint != nextEndpoint {
                            lastEndpoint = tmpEndpoint;
                            tmpEndpoint = (*tmpEndpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;
                        }

                        lastEndTime = ((*lastEndpoint).StartTime as i32
                            + (*lastEndpoint).CalcBusTime as i32) as ULONG;

                        if endTime < (lastEndTime).wrapping_sub(1) {
                            maxEndTime = lastEndTime;
                            endTime = maxEndTime;

                            if (*nextEndpoint).StartTime as ULONG == maxEndTime {
                                break;
                            }
                        } else {
                            maxEndTime = endTime;
                        }
                    }
                    jx = jx.wrapping_add(1);
                    frame = (frame as i32 + (*nextEndpoint).ActualPeriod as i32) as UCHAR;
                }
            }

            if maxEndTime >= (*nextEndpoint).StartTime as ULONG {
                break;
            }

            if USB2_MoveTtEndpoint(
                nextEndpoint,
                (maxEndTime).wrapping_sub((*nextEndpoint).StartTime as ULONG) as LONG,
                Rebalance,
                *RebalanceListEntries,
                core::ptr::addr_of_mut!(IsMoved),
            ) == 0
            {
                if IsMoved == 0 {}

                break;
            } else {
                if Period as i32 > (*nextEndpoint).ActualPeriod as i32 {
                    Period = (*nextEndpoint).ActualPeriod;
                }
                endpoint = nextEndpoint;
                nextEndpoint = (*nextEndpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;
            }
        }

        ix = ix.wrapping_sub(1);
        Frame = Frame.wrapping_sub(1);
    }

    (*TtEndpoint).CalcBusTime = 0 as USHORT;

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_AllocateTimeForEndpoint(
    TtEndpoint: PUSB2_TT_ENDPOINT,
    Rebalance: PUSB2_REBALANCE,
    RebalanceListEntries: PULONG,
) -> BOOLEAN {
    let mut Tt: PUSB2_TT;
    let mut HcExtension: PUSB2_HC_EXTENSION;
    let mut Speed: ULONG;
    let mut TimeUsed: ULONG;
    let mut MinTimeUsed: ULONG;
    let mut ix: ULONG;
    let mut frame: ULONG;
    let mut uframe: ULONG;
    let mut Microframe: ULONG;
    let mut TransferType: ULONG;
    let mut Overhead: ULONG;
    let mut LatestStart: ULONG;
    let mut prevEndpoint: PUSB2_TT_ENDPOINT;
    let mut nextEndpoint: PUSB2_TT_ENDPOINT;
    let mut IntEndpoint: PUSB2_TT_ENDPOINT;
    let mut StartTime: ULONG;
    let mut calcBusTime: ULONG = 0;
    let mut Result: BOOLEAN = TRUE as BOOLEAN;

    Tt = (*TtEndpoint).Tt as PUSB2_TT;
    HcExtension = (*Tt).HcExtension as PUSB2_HC_EXTENSION;

    (*TtEndpoint).Nums.c2rust_unnamed.set_NumStarts(0);
    (*TtEndpoint).Nums.c2rust_unnamed.set_NumCompletes(0);

    (*TtEndpoint).StartFrame = 0 as UCHAR;
    (*TtEndpoint).StartMicroframe = 0 as UCHAR;

    if (*TtEndpoint).CalcBusTime != 0 {
        return FALSE as BOOLEAN;
    }

    Speed = (*TtEndpoint).TtEndpointParams.c2rust_unnamed.DeviceSpeed();

    if Speed == UsbHighSpeed as ULONG {
        if (*TtEndpoint).Period as i32 > (USB2_MAX_MICROFRAMES as i32) {
            (*TtEndpoint).ActualPeriod = USB2_MAX_MICROFRAMES as USHORT;
        } else {
            (*TtEndpoint).ActualPeriod = (*TtEndpoint).Period;
        }

        MinTimeUsed = (*HcExtension).TimeUsed[0][0];

        ix = 1 as ULONG;
        while ix < (*TtEndpoint).ActualPeriod as ULONG {
            frame = ix / USB2_MICROFRAMES as ULONG;
            uframe = ix % (USB2_MICROFRAMES - 1) as ULONG;

            TimeUsed = (*HcExtension).TimeUsed[frame as usize][uframe as usize];

            if TimeUsed < MinTimeUsed {
                MinTimeUsed = TimeUsed;
                (*TtEndpoint).StartFrame = frame as UCHAR;
                (*TtEndpoint).StartMicroframe = uframe as UCHAR;
            }
            ix = ix.wrapping_add(1);
        }

        (*TtEndpoint).CalcBusTime = (USB2_GetOverhead(TtEndpoint) as ULONG)
            .wrapping_add(USB2_AddDataBitStuff((*TtEndpoint).MaxPacketSize) as ULONG)
            as USHORT;

        Microframe = ((*TtEndpoint).StartFrame as i32 * (USB2_MICROFRAMES as i32)
            + (*TtEndpoint).StartMicroframe as i32) as ULONG;

        if Microframe >= USB2_MAX_MICROFRAMES as ULONG {
            return TRUE as BOOLEAN;
        }

        ix = Microframe;
        while ix < USB2_MAX_MICROFRAMES as ULONG {
            frame = ix / USB2_MICROFRAMES as ULONG;
            uframe = ix % (USB2_MICROFRAMES - 1) as ULONG;

            if USB2_AllocateCheck(
                (&raw mut *(&raw mut (*HcExtension).TimeUsed as *mut [ULONG; 8])
                    .offset(frame as isize) as *mut ULONG)
                    .offset(uframe as isize) as PULONG,
                (*TtEndpoint).CalcBusTime as ULONG,
                USB2_MAX_MICROFRAME_ALLOCATION as ULONG,
            ) == 0
            {
                Result = FALSE as BOOLEAN;
            }
            ix = ix.wrapping_add((*TtEndpoint).ActualPeriod as ULONG);
        }

        if Result == 0 {
            ix = Microframe;
            while ix < USB2_MAX_MICROFRAMES as ULONG {
                frame = ix / USB2_MICROFRAMES as ULONG;
                uframe = ix % (USB2_MICROFRAMES - 1) as ULONG;

                (*HcExtension).TimeUsed[frame as usize][uframe as usize] =
                    ((*HcExtension).TimeUsed[frame as usize][uframe as usize] as ULONG)
                        .wrapping_sub((*TtEndpoint).CalcBusTime as ULONG)
                        as ULONG;
                ix = ix.wrapping_add((*TtEndpoint).ActualPeriod as ULONG);
            }
        }

        return TRUE as BOOLEAN;
    }

    if (*TtEndpoint).Period as i32 > (USB2_FRAMES as i32) {
        (*TtEndpoint).ActualPeriod = USB2_FRAMES as USHORT;
    } else {
        (*TtEndpoint).ActualPeriod = (*TtEndpoint).Period;
    }

    MinTimeUsed = (*Tt).FrameBudget[0].TimeUsed as ULONG;

    ix = 1 as ULONG;
    while ix < (*TtEndpoint).ActualPeriod as ULONG {
        if ((*Tt).FrameBudget[ix as usize].TimeUsed as ULONG) < MinTimeUsed {
            MinTimeUsed = (*Tt).FrameBudget[ix as usize].TimeUsed as ULONG;
            (*TtEndpoint).StartFrame = ix as UCHAR;
        }
        ix = ix.wrapping_add(1);
    }

    TransferType = (*TtEndpoint).TtEndpointParams.c2rust_unnamed.TransferType();

    if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        if Speed == UsbFullSpeed as ULONG {
            Overhead = (USB2_FS_ISOCHRONOUS_OVERHEAD as ULONG).wrapping_add((*Tt).DelayTime);
        } else {
            return FALSE as BOOLEAN;
        }
    } else if Speed == UsbFullSpeed as ULONG {
        Overhead = (USB2_FS_INTERRUPT_OVERHEAD as ULONG).wrapping_add((*Tt).DelayTime);
    } else {
        Overhead = (USB2_LS_INTERRUPT_OVERHEAD as ULONG).wrapping_add((*Tt).DelayTime);
    }

    if Speed == UsbLowSpeed as ULONG {
        (*TtEndpoint).CalcBusTime =
            (((*TtEndpoint).MaxPacketSize as i32 * 8) as ULONG).wrapping_add(Overhead) as USHORT;
    } else {
        (*TtEndpoint).CalcBusTime =
            ((*TtEndpoint).MaxPacketSize as ULONG).wrapping_add(Overhead) as USHORT;
    }

    LatestStart = (USB2_HUB_DELAY + USB2_FS_SOF_TIME) as ULONG;

    ix = 0 as ULONG;
    while ((*TtEndpoint).StartFrame as ULONG).wrapping_add(ix) < USB2_FRAMES as ULONG {
        frame = ((*TtEndpoint).StartFrame as ULONG).wrapping_add(ix);

        if !(*Tt).FrameBudget[frame as usize].AltEndpoint.is_null()
            && (*TtEndpoint).CalcBusTime as i32 >= (((USB2_FS_MAX_PERIODIC_ALLOCATION / (2 as i32 as i32) as u32)) as i32)
        {
            return FALSE as BOOLEAN;
        }

        if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
            prevEndpoint = (*Tt).FrameBudget[frame as usize].IsoEndpoint;
        } else {
            prevEndpoint = (*Tt).FrameBudget[frame as usize].IntEndpoint;
        }

        nextEndpoint = (*prevEndpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;
        while !nextEndpoint.is_null() {
            if USB2_CheckTtEndpointInsert(nextEndpoint, TtEndpoint) != 0 {
                break;
            }
            prevEndpoint = nextEndpoint;
            nextEndpoint = (*nextEndpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;
        }

        StartTime = USB2_GetStartTime(nextEndpoint, TtEndpoint, prevEndpoint, frame);

        LatestStart = if LatestStart > StartTime {
            LatestStart
        } else {
            StartTime
        };
        ix = ix.wrapping_add((*TtEndpoint).ActualPeriod as ULONG);
    }

    (*TtEndpoint).StartTime = LatestStart as USHORT;

    if (LatestStart as ULONG).wrapping_add((*TtEndpoint).CalcBusTime as ULONG)
        > USB2_FS_MAX_PERIODIC_ALLOCATION as ULONG
    {
        (*TtEndpoint).CalcBusTime = 0 as USHORT;
        return FALSE as BOOLEAN;
    }

    ix = 0 as ULONG;
    frame = -((*TtEndpoint).StartFrame as i32) as ULONG;
    while ix < USB2_FRAMES as ULONG {
        if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
            // DPRINT1: Iso Ep UNIMPLEMENTED. FIXME
            // ASSERT(FALSE);
            DbgBreakPoint();
        } else {
            IntEndpoint = (*Tt).FrameBudget[ix as usize].IntEndpoint;
            nextEndpoint = (*IntEndpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;

            nextEndpoint = (*IntEndpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;
            while !nextEndpoint.is_null() {
                if USB2_CheckTtEndpointInsert(nextEndpoint, TtEndpoint) != 0 {
                    break;
                }
                IntEndpoint = nextEndpoint;
                nextEndpoint = (*nextEndpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;
            }

            if frame % (*TtEndpoint).ActualPeriod as ULONG == 0 {
                calcBusTime = 0 as ULONG;
            } else {
                if !nextEndpoint.is_null() {
                    calcBusTime = (LatestStart as ULONG)
                        .wrapping_add((*TtEndpoint).CalcBusTime as ULONG)
                        .wrapping_sub((*nextEndpoint).StartTime as ULONG)
                        as ULONG;
                } else {
                    calcBusTime = (*TtEndpoint).CalcBusTime as ULONG;
                }

                if calcBusTime > 0 {
                    TimeUsed = (*Tt).FrameBudget[ix as usize].TimeUsed as ULONG;

                    if USB2_AllocateCheck(
                        &raw mut TimeUsed,
                        calcBusTime,
                        USB2_FS_MAX_PERIODIC_ALLOCATION as ULONG,
                    ) == 0
                    {
                        Result = FALSE as BOOLEAN;
                    }
                }
            }

            if nextEndpoint != TtEndpoint {
                if frame % (*TtEndpoint).ActualPeriod as ULONG == 0 {
                    if frame == 0 {
                        (*TtEndpoint).NextTtEndpoint = nextEndpoint as *mut _USB2_TT_ENDPOINT;
                    }

                    (*IntEndpoint).NextTtEndpoint = TtEndpoint as *mut _USB2_TT_ENDPOINT;
                }

                if calcBusTime > 0 {
                    let mut IsMoved: BOOLEAN = 0;
                    let mut MoveResult: BOOLEAN;

                    while !nextEndpoint.is_null() {
                        MoveResult = USB2_MoveTtEndpoint(
                            nextEndpoint,
                            calcBusTime as LONG,
                            Rebalance,
                            *RebalanceListEntries,
                            &raw mut IsMoved,
                        );

                        if IsMoved == 0 {
                            Result = FALSE as BOOLEAN;
                        }

                        if MoveResult == 0 {
                            break;
                        }
                        nextEndpoint = (*nextEndpoint).NextTtEndpoint as PUSB2_TT_ENDPOINT;
                    }
                }
            }
        }

        if frame % (*TtEndpoint).ActualPeriod as ULONG == 0 {
            if USB2_AllocateHS(TtEndpoint, frame as LONG) == 0 {
                // DPRINT1: USB2_AllocateHS return FALSE
                Result = FALSE as BOOLEAN;
            }

            (*Tt).FrameBudget[ix as usize].TimeUsed =
                ((*Tt).FrameBudget[ix as usize].TimeUsed as i32
                    + (*TtEndpoint).CalcBusTime as i32) as USHORT;
        }

        if Result as i32 == (FALSE as i32) {
            USB2_DeallocateEndpointBudget(
                TtEndpoint,
                Rebalance,
                RebalanceListEntries,
                ix.wrapping_add(1 as ULONG),
            );

            return FALSE as BOOLEAN;
        }

        ix = ix.wrapping_add(1);
        frame = frame.wrapping_add(1);
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_ChangePeriod(
    TtEndpoint: PUSB2_TT_ENDPOINT,
    Rebalance: PUSB2_REBALANCE,
    RebalanceListEntries: PULONG,
) -> BOOLEAN {
    let mut Result: BOOLEAN;

    USB2_DeallocateEndpointBudget(
        TtEndpoint,
        Rebalance,
        RebalanceListEntries,
        USB2_FRAMES as ULONG,
    );

    (*TtEndpoint).PreviosPeriod = (*TtEndpoint).Period;
    (*TtEndpoint).Period = ENDPOINT_INTERRUPT_1ms as USHORT;

    Result = USB2_AllocateTimeForEndpoint(TtEndpoint, Rebalance, RebalanceListEntries);

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USB2_PromotePeriods(
    TtEndpoint: PUSB2_TT_ENDPOINT,
    Rebalance: PUSB2_REBALANCE,
    RebalanceListEntries: PULONG,
) -> BOOLEAN {
    let mut ttEndpoint: PUSB2_TT_ENDPOINT;
    let mut TransferType: ULONG;
    let mut ix: ULONG;

    TransferType = (*TtEndpoint).TtEndpointParams.c2rust_unnamed.TransferType();

    if (*TtEndpoint).ActualPeriod as i32 != (ENDPOINT_INTERRUPT_1ms as i32)
        && TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
        && (*TtEndpoint).StartMicroframe as CHAR as i32 > 2
        && USB2_ChangePeriod(TtEndpoint, Rebalance, RebalanceListEntries) == 0
    {
        return FALSE as BOOLEAN;
    }

    if (*Rebalance).RebalanceEndpoint[0].is_null() {
        return TRUE as BOOLEAN;
    }

    ix = 0;
    while !(*Rebalance).RebalanceEndpoint[ix as usize].is_null() {
        (*(*Rebalance).RebalanceEndpoint[ix as usize]).IsPromoted = FALSE as BOOL;
        ix = ix.wrapping_add(1);
    }

    ix = 0;
    loop {
        ttEndpoint = (*Rebalance).RebalanceEndpoint[ix as usize];
        TransferType = (*ttEndpoint).TtEndpointParams.c2rust_unnamed.TransferType();

        if (*ttEndpoint).ActualPeriod as i32 != (ENDPOINT_INTERRUPT_1ms as i32)
            && TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
            && (*ttEndpoint).StartMicroframe as CHAR as i32 > 2
        {
            USB2_DeallocateEndpointBudget(
                ttEndpoint,
                Rebalance,
                RebalanceListEntries,
                USB2_FRAMES as ULONG,
            );

            (*ttEndpoint).IsPromoted = TRUE as BOOL;
            (*ttEndpoint).PreviosPeriod = (*ttEndpoint).Period;
            (*ttEndpoint).Period = ENDPOINT_INTERRUPT_1ms as USHORT;

            if USB2_AllocateTimeForEndpoint(ttEndpoint, Rebalance, RebalanceListEntries) == 0 {
                break;
            }
        }

        if (*Rebalance).RebalanceEndpoint[(ix as ULONG).wrapping_add(1) as usize].is_null() {
            return TRUE as BOOLEAN;
        }

        ix = ix.wrapping_add(1);
    }

    USB2_DeallocateEndpointBudget(
        TtEndpoint,
        Rebalance,
        RebalanceListEntries,
        USB2_FRAMES as ULONG,
    );

    (*TtEndpoint).Period = (*TtEndpoint).PreviosPeriod;
    (*TtEndpoint).PreviosPeriod = 0 as USHORT;

    ix = 0;
    while !(*Rebalance).RebalanceEndpoint[ix as usize].is_null() {
        ttEndpoint = (*Rebalance).RebalanceEndpoint[ix as usize];

        if (*ttEndpoint).IsPromoted != 0 {
            if (*ttEndpoint).CalcBusTime != 0 {
                USB2_DeallocateEndpointBudget(
                    ttEndpoint,
                    Rebalance,
                    RebalanceListEntries,
                    USB2_FRAMES as ULONG,
                );
            }

            (*TtEndpoint).Period = (*TtEndpoint).PreviosPeriod;
            (*TtEndpoint).PreviosPeriod = 0 as USHORT;

            USB2_AllocateTimeForEndpoint(ttEndpoint, Rebalance, RebalanceListEntries);
        }

        ix = ix.wrapping_add(1);
    }

    return FALSE as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_UpdateAllocatedBwTt(TtExtension: PUSB2_TT_EXTENSION) {
    let mut BusBandwidth: ULONG;
    let mut NewBusBandwidth: ULONG;
    let mut MaxBusBandwidth: ULONG = 0;
    let mut MinBusBandwidth: ULONG;
    let mut ix: ULONG;

    BusBandwidth = (*TtExtension).BusBandwidth;
    MinBusBandwidth = BusBandwidth;

    ix = 0;
    while ix < USB2_FRAMES as ULONG {
        NewBusBandwidth = BusBandwidth.wrapping_sub((*TtExtension).Bandwidth[ix as usize]);

        MaxBusBandwidth = if MaxBusBandwidth > NewBusBandwidth {
            MaxBusBandwidth
        } else {
            NewBusBandwidth
        };
        MinBusBandwidth = if MinBusBandwidth < NewBusBandwidth {
            MinBusBandwidth
        } else {
            NewBusBandwidth
        };

        ix = ix.wrapping_add(1);
    }

    (*TtExtension).MaxBandwidth = MaxBusBandwidth;

    if MinBusBandwidth == BusBandwidth {
        (*TtExtension).MinBandwidth = 0;
    } else {
        (*TtExtension).MinBandwidth = MinBusBandwidth;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AllocateBandwidthUSB2(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) -> BOOLEAN {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES = core::ptr::null_mut();
    let mut TtExtension: PUSB2_TT_EXTENSION = core::ptr::null_mut();
    let mut TransferType: ULONG;
    let mut Rebalance: PUSB2_REBALANCE = core::ptr::null_mut();
    let mut RebalanceList: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut RebalanceListEntries: ULONG = 0;
    let mut TtEndpoint: PUSB2_TT_ENDPOINT = core::ptr::null_mut();
    let mut RebalanceTtEndpoint: PUSB2_TT_ENDPOINT;
    let mut Tt: PUSB2_TT = core::ptr::null_mut();
    let mut DeviceSpeed: USB_DEVICE_SPEED;
    let mut Period: ULONG = 0;
    let mut AllocedBusTime: ULONG;
    let mut EndpointBandwidth: ULONG;
    let mut ScheduleOffset: ULONG;
    let mut Factor: ULONG;
    let mut ix: ULONG;
    let mut n: ULONG;
    let mut Direction: BOOLEAN;
    let mut SMask: UCHAR;
    let mut CMask: UCHAR;
    let mut ActualPeriod: UCHAR;
    let mut Result: BOOLEAN = 0;

    EndpointProperties =
        core::ptr::addr_of_mut!((*Endpoint).EndpointProperties) as PUSBPORT_ENDPOINT_PROPERTIES;
    (*EndpointProperties).ScheduleOffset = 0;

    if (*Endpoint).Flags as u32 & ENDPOINT_FLAG_ROOTHUB_EP0 as u32 != 0 {
        return 1 as BOOLEAN;
    }

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    TransferType = (*EndpointProperties).TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_CONTROL as u32
        || TransferType == USBPORT_TRANSFER_TYPE_BULK as u32
    {
        return 1 as BOOLEAN;
    }

    // An xHCI controller budgets periodic bandwidth itself (Configure Endpoint fails
    // with a Bandwidth Error) and drives transaction translators itself, so this
    // USB2 budget does not apply to it. It would also crash: low- and full-speed
    // devices sit directly on xHCI root ports with no TtExtension, which never
    // happens under EHCI, where companion controllers take those devices.
    if (*(*FdoExtension).MiniPortInterface).Packet.MiniPortVersion == USB_MINIPORT_VERSION_XHCI {
        return 1 as BOOLEAN;
    }

    if !(*Endpoint).TtExtension.is_null() {
        TtExtension = (*Endpoint).TtExtension as PUSB2_TT_EXTENSION;
    } else {
        TtExtension = core::ptr::null_mut();
    }

    InitializeListHead(core::ptr::addr_of_mut!(RebalanceList));

    Rebalance = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USB2_REBALANCE>() as SIZE_T,
        USB_PORT_TAG as ULONG,
    ) as PUSB2_REBALANCE;

    if !Rebalance.is_null() {
        RtlZeroMemory(
            Rebalance as PVOID,
            core::mem::size_of::<USB2_REBALANCE>() as SIZE_T,
        );

        TtEndpoint = (*Endpoint).TtEndpoint as PUSB2_TT_ENDPOINT;
        (*TtEndpoint).Endpoint = Endpoint;

        Direction = ((*EndpointProperties).Direction == USBPORT_TRANSFER_DIRECTION_OUT as u32)
            as i32 as BOOLEAN;
        DeviceSpeed = (*EndpointProperties).DeviceSpeed;

        match DeviceSpeed as u32 {
            0 | 1 => {
                Tt = core::ptr::addr_of_mut!((*TtExtension).Tt) as PUSB2_TT;
                Period = USB2_FRAMES as ULONG;

                while Period > 0 && Period > (*EndpointProperties).Period as u32 {
                    Period >>= 1;
                }
            }
            2 => {
                Tt = core::ptr::addr_of_mut!((*(*FdoExtension).Usb2Extension).HcTt) as PUSB2_TT;
                Period = (*EndpointProperties).Period as ULONG;
            }
            _ => {
                DbgBreakPoint();

                Tt = core::ptr::addr_of_mut!((*TtExtension).Tt) as PUSB2_TT;
            }
        }

        USB2_InitTtEndpoint(
            TtEndpoint,
            TransferType as UCHAR,
            Direction as UCHAR,
            DeviceSpeed as UCHAR,
            Period as USHORT,
            (*EndpointProperties).MaxPacketSize as USHORT,
            Tt,
        );

        RebalanceListEntries = (USB2_FRAMES - 2) as ULONG;

        Result = USB2_AllocateTimeForEndpoint(
            TtEndpoint,
            Rebalance,
            core::ptr::addr_of_mut!(RebalanceListEntries),
        );

        if Result != 0 {
            Result = USB2_PromotePeriods(
                TtEndpoint,
                Rebalance,
                core::ptr::addr_of_mut!(RebalanceListEntries),
            );
        }

        RebalanceListEntries = 0;

        ix = 0;
        while !(*Rebalance).RebalanceEndpoint[ix as usize].is_null() {
            RebalanceListEntries = ix.wrapping_add(1);
            ix = ix.wrapping_add(1);
        }
    } else {
        RebalanceListEntries = 0;
        Result = 0 as BOOLEAN;
    }

    ix = 0;
    while ix < RebalanceListEntries {
        RebalanceTtEndpoint = (*Rebalance).RebalanceEndpoint[ix as usize];

        InsertTailList(
            core::ptr::addr_of_mut!(RebalanceList),
            core::ptr::addr_of_mut!((*(*RebalanceTtEndpoint).Endpoint).RebalanceLink),
        );

        ix = ix.wrapping_add(1);
    }

    if !Rebalance.is_null() {
        ExFreePoolWithTag(Rebalance as PVOID, USB_PORT_TAG as ULONG);
    }

    if Result != 0 {
        SMask = USB2_GetSMASK((*Endpoint).TtEndpoint as PUSB2_TT_ENDPOINT);
        (*EndpointProperties).InterruptScheduleMask = SMask;

        CMask = USB2_GetCMASK((*Endpoint).TtEndpoint as PUSB2_TT_ENDPOINT);
        (*EndpointProperties).SplitCompletionMask = CMask;

        AllocedBusTime = (*TtEndpoint).CalcBusTime as ULONG;

        EndpointBandwidth = (USB2_MICROFRAMES as ULONG).wrapping_mul(AllocedBusTime);
        (*EndpointProperties).UsbBandwidth = EndpointBandwidth;

        ActualPeriod = (*(*Endpoint).TtEndpoint).ActualPeriod as UCHAR;
        (*EndpointProperties).Period = ActualPeriod;

        ScheduleOffset = (*(*Endpoint).TtEndpoint).StartFrame as ULONG;
        (*EndpointProperties).ScheduleOffset = ScheduleOffset;

        Factor = (USB2_FRAMES / (ActualPeriod as i32 as u32)) as ULONG;
        n = ScheduleOffset.wrapping_mul(Factor);

        if !TtExtension.is_null() {
            ix = 0;
            while ix < Factor {
                (*TtExtension).Bandwidth[n.wrapping_add(ix) as usize] = ((*TtExtension).Bandwidth
                    [n.wrapping_add(ix) as usize]
                    as u32)
                    .wrapping_sub(EndpointBandwidth)
                    as ULONG;
                ix = ix.wrapping_add(1);
            }
        } else {
            ix = 1;
            while ix < Factor {
                (*FdoExtension).Bandwidth[n.wrapping_add(ix) as usize] = ((*FdoExtension).Bandwidth
                    [n.wrapping_add(ix) as usize]
                    as u32)
                    .wrapping_sub(EndpointBandwidth)
                    as ULONG;
                ix = ix.wrapping_add(1);
            }
        }

        USBPORT_DumpingEndpointProperties(EndpointProperties);
        USBPORT_DumpingTtEndpoint((*Endpoint).TtEndpoint as PUSB2_TT_ENDPOINT);

        if AllocedBusTime >= (USB2_FS_MAX_PERIODIC_ALLOCATION / 2) as u32 {
        }
    }

    USB2_Rebalance(FdoDevice, core::ptr::addr_of_mut!(RebalanceList));

    if TtExtension.is_null() {
        return Result;
    }

    ix = 0;
    while ix < USB2_FRAMES as u32 {
        (*FdoExtension).Bandwidth[ix as usize] = ((*FdoExtension).Bandwidth[ix as usize] as u32)
            .wrapping_add((*TtExtension).MaxBandwidth as u32)
            as ULONG;
        ix = ix.wrapping_add(1);
    }

    USBPORT_UpdateAllocatedBwTt(TtExtension);

    ix = 0;
    while ix < USB2_FRAMES as u32 {
        (*FdoExtension).Bandwidth[ix as usize] = ((*FdoExtension).Bandwidth[ix as usize] as u32)
            .wrapping_sub((*TtExtension).MaxBandwidth as u32)
            as ULONG;
        ix = ix.wrapping_add(1);
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FreeBandwidthUSB2(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Period: ULONG;
    let mut ScheduleOffset: ULONG;
    let mut EndpointBandwidth: ULONG;
    let mut RebalanceList: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut TransferType: ULONG;
    let mut Rebalance: PUSB2_REBALANCE;
    let mut RebalanceListEntries: ULONG;
    let mut Factor: ULONG;
    let mut ix: ULONG;
    let mut n: ULONG;
    let mut TtExtension: PUSB2_TT_EXTENSION;
    let mut RebalanceTtEndpoint: PUSB2_TT_ENDPOINT;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    Period = (*Endpoint).EndpointProperties.Period as ULONG;
    ScheduleOffset = (*Endpoint).EndpointProperties.ScheduleOffset;
    EndpointBandwidth = (*Endpoint).EndpointProperties.UsbBandwidth;

    InitializeListHead(core::ptr::addr_of_mut!(RebalanceList));

    TransferType = (*Endpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_BULK as ULONG
        || ((*Endpoint).Flags as ULONG & ENDPOINT_FLAG_ROOTHUB_EP0 as ULONG) != 0
    {
        return;
    }

    // Nothing was allocated for an xHCI endpoint (see USBPORT_AllocateBandwidthUSB2).
    if (*(*FdoExtension).MiniPortInterface).Packet.MiniPortVersion == USB_MINIPORT_VERSION_XHCI {
        return;
    }

    Rebalance = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USB2_REBALANCE>() as SIZE_T,
        USB_PORT_TAG as ULONG,
    ) as PUSB2_REBALANCE;

    if Rebalance.is_null() {
        return;
    }

    RtlZeroMemory(
        Rebalance as PVOID,
        core::mem::size_of::<USB2_REBALANCE>() as SIZE_T,
    );

    Factor = (USB2_FRAMES as ULONG).wrapping_div(Period);
    n = ScheduleOffset.wrapping_mul(Factor);

    TtExtension = (*Endpoint).TtExtension as PUSB2_TT_EXTENSION;

    if !TtExtension.is_null() {
        ix = 0;
        while ix < Factor {
            (*TtExtension).Bandwidth[n.wrapping_add(ix) as usize] = ((*TtExtension).Bandwidth
                [n.wrapping_add(ix) as usize] as ULONG)
                .wrapping_add(EndpointBandwidth) as ULONG;
            ix = ix.wrapping_add(1);
        }
    } else {
        ix = 1;
        while ix < Factor {
            (*FdoExtension).Bandwidth[n.wrapping_add(ix) as usize] = ((*FdoExtension).Bandwidth
                [n.wrapping_add(ix) as usize] as ULONG)
                .wrapping_add(EndpointBandwidth) as ULONG;
            ix = ix.wrapping_add(1);
        }
    }

    RebalanceListEntries = (USB2_FRAMES - 2) as ULONG;

    USB2_DeallocateEndpointBudget(
        (*Endpoint).TtEndpoint as PUSB2_TT_ENDPOINT,
        Rebalance,
        core::ptr::addr_of_mut!(RebalanceListEntries),
        USB2_FRAMES as ULONG,
    );

    RebalanceListEntries = 0;

    ix = 0;
    while !(*Rebalance).RebalanceEndpoint[ix as usize].is_null() {
        RebalanceListEntries = ix.wrapping_add(1);
        ix = ix.wrapping_add(1);
    }

    ix = 0;
    while ix < RebalanceListEntries {
        RebalanceTtEndpoint = (*Rebalance).RebalanceEndpoint[ix as usize];

        InsertTailList(
            core::ptr::addr_of_mut!(RebalanceList),
            core::ptr::addr_of_mut!((*(*RebalanceTtEndpoint).Endpoint).RebalanceLink),
        );
        ix = ix.wrapping_add(1);
    }

    ExFreePoolWithTag(Rebalance as PVOID, USB_PORT_TAG as ULONG);

    USB2_Rebalance(FdoDevice, core::ptr::addr_of_mut!(RebalanceList));

    if TtExtension.is_null() {
        return;
    }

    ix = 0;
    while ix < USB2_FRAMES as ULONG {
        (*FdoExtension).Bandwidth[ix as usize] = ((*FdoExtension).Bandwidth[ix as usize] as ULONG)
            .wrapping_add((*TtExtension).MaxBandwidth as ULONG) as ULONG;
        ix = ix.wrapping_add(1);
    }

    USBPORT_UpdateAllocatedBwTt(TtExtension);

    ix = 0;
    while ix < USB2_FRAMES as ULONG {
        (*FdoExtension).Bandwidth[ix as usize] = ((*FdoExtension).Bandwidth[ix as usize] as ULONG)
            .wrapping_sub((*TtExtension).MaxBandwidth as ULONG) as ULONG;
        ix = ix.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USB2_InitTT(
    HcExtension: PUSB2_HC_EXTENSION,
    Tt: PUSB2_TT,
) {
    let mut ix: ULONG;
    let mut jx: ULONG;

    (*Tt).HcExtension = HcExtension;
    (*Tt).DelayTime = 1 as ULONG;
    (*Tt).MaxTime = USB2_FS_MAX_PERIODIC_ALLOCATION as ULONG;

    ix = 0 as ULONG;
    while ix < USB2_FRAMES as ULONG {
        (*Tt).FrameBudget[ix as usize].TimeUsed = USB2_MAX_MICROFRAMES as USHORT;
        (*Tt).FrameBudget[ix as usize].AltEndpoint = core::ptr::null_mut();

        jx = 0 as ULONG;
        while jx < USB2_MICROFRAMES as ULONG {
            (*Tt).TimeCS[ix as usize][jx as usize] = 0 as ULONG;
            (*Tt).NumStartSplits[ix as usize][jx as usize] = 0 as ULONG;
            jx = jx.wrapping_add(1);
        }

        (*Tt).FrameBudget[ix as usize].IsoEndpoint =
            (&raw mut (*Tt).IsoEndpoint as *mut USB2_TT_ENDPOINT).offset(ix as isize)
                as PUSB2_TT_ENDPOINT;

        USB2_InitTtEndpoint(
            (&raw mut (*Tt).IsoEndpoint as *mut USB2_TT_ENDPOINT).offset(ix as isize)
                as PUSB2_TT_ENDPOINT,
            USBPORT_TRANSFER_TYPE_ISOCHRONOUS as UCHAR,
            USBPORT_TRANSFER_DIRECTION_OUT as UCHAR,
            UsbFullSpeed as UCHAR,
            USB2_FRAMES as USHORT,
            0 as USHORT,
            Tt,
        );

        (*Tt).IsoEndpoint[ix as usize].ActualPeriod = USB2_FRAMES as USHORT;
        (*Tt).IsoEndpoint[ix as usize].CalcBusTime = (USB2_FS_SOF_TIME + USB2_HUB_DELAY) as USHORT;
        (*Tt).IsoEndpoint[ix as usize].StartFrame = ix as UCHAR;
        (*Tt).IsoEndpoint[ix as usize].StartMicroframe = USB2_PREV_MICROFRAME as UCHAR;

        (*Tt).FrameBudget[ix as usize].IntEndpoint =
            (&raw mut (*Tt).IntEndpoint as *mut USB2_TT_ENDPOINT).offset(ix as isize)
                as PUSB2_TT_ENDPOINT;

        USB2_InitTtEndpoint(
            (&raw mut (*Tt).IntEndpoint as *mut USB2_TT_ENDPOINT).offset(ix as isize)
                as PUSB2_TT_ENDPOINT,
            USBPORT_TRANSFER_TYPE_INTERRUPT as UCHAR,
            USBPORT_TRANSFER_DIRECTION_OUT as UCHAR,
            UsbFullSpeed as UCHAR,
            USB2_FRAMES as USHORT,
            0 as USHORT,
            Tt,
        );

        (*Tt).IntEndpoint[ix as usize].ActualPeriod = USB2_FRAMES as USHORT;
        (*Tt).IntEndpoint[ix as usize].CalcBusTime = (USB2_FS_SOF_TIME + USB2_HUB_DELAY) as USHORT;
        (*Tt).IntEndpoint[ix as usize].StartFrame = ix as UCHAR;
        (*Tt).IntEndpoint[ix as usize].StartMicroframe = USB2_PREV_MICROFRAME as UCHAR;

        ix = ix.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USB2_InitController(HcExtension: PUSB2_HC_EXTENSION) {
    let mut ix: ULONG;
    let mut jx: ULONG;

    (*HcExtension).MaxHsBusAllocation = USB2_MAX_MICROFRAME_ALLOCATION as ULONG;

    ix = 0;
    while ix < USB2_FRAMES as ULONG {
        jx = 0;
        while jx < USB2_MICROFRAMES as ULONG {
            (*HcExtension).TimeUsed[ix as usize][jx as usize] = 0 as ULONG;
            jx = jx.wrapping_add(1);
        }
        ix = ix.wrapping_add(1);
    }

    (*HcExtension).HcDelayTime = USB2_CONTROLLER_DELAY as ULONG;

    USB2_InitTT(HcExtension, core::ptr::addr_of_mut!((*HcExtension).HcTt));
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort USB 2.0 functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// static const UCHAR CMASKS[USB2_MICROFRAMES] = {
//   0x1C, 0x38, 0x70, 0xE0, 0xC1, 0x83, 0x07, 0x0E
// };
//
// BOOLEAN
// NTAPI
// USB2_AllocateCheck(IN OUT PULONG OutTimeUsed,
//                    IN ULONG CalcBusTime,
//                    IN ULONG LimitAllocation)
// {
//     ULONG BusTime;
//     BOOLEAN Result = TRUE;
//
//     BusTime = *OutTimeUsed + CalcBusTime;
//     *OutTimeUsed += CalcBusTime;
//
//     if (BusTime > LimitAllocation)
//     {
//         DPRINT("USB2_AllocateCheck: BusTime > LimitAllocation\n");
//         Result = FALSE;
//     }
//
//     return Result;
// }
//
// USHORT
// NTAPI
// USB2_AddDataBitStuff(IN USHORT DataTime)
// {
//     return (DataTime + (DataTime / 16));
// }
//
// VOID
// NTAPI
// USB2_IncMicroFrame(OUT PUCHAR frame,
//                    OUT PUCHAR uframe)
// {
//     ++*uframe;
//
//     if (*uframe > (USB2_MICROFRAMES - 1))
//     {
//         *uframe = 0;
//         *frame = (*frame + 1) & (USB2_FRAMES - 1);
//     }
// }
//
// VOID
// NTAPI
// USB2_GetPrevMicroFrame(OUT PUCHAR frame,
//                        OUT PUCHAR uframe)
// {
//     *uframe = USB2_MICROFRAMES - 1;
//
//     if (*frame)
//         --*frame;
//     else
//         *frame = USB2_FRAMES - 1;
// }
//
// BOOLEAN
// NTAPI
// USB2_CheckTtEndpointInsert(IN PUSB2_TT_ENDPOINT nextTtEndpoint,
//                            IN PUSB2_TT_ENDPOINT TtEndpoint)
// {
//     ULONG TransferType;
//
//     DPRINT("USB2_CheckTtEndpointInsert: nextTtEndpoint - %p, TtEndpoint - %p\n",
//            nextTtEndpoint,
//            TtEndpoint);
//
//     ASSERT(TtEndpoint);
//
//     if (TtEndpoint->CalcBusTime >= (USB2_FS_MAX_PERIODIC_ALLOCATION / 2))
//     {
//         DPRINT1("USB2_CheckTtEndpointInsert: Result - FALSE\n");
//         return FALSE;
//     }
//
//     if (!nextTtEndpoint)
//     {
//         DPRINT("USB2_CheckTtEndpointInsert: Result - TRUE\n");
//         return TRUE;
//     }
//
//     TransferType = TtEndpoint->TtEndpointParams.TransferType;
//
//     if (nextTtEndpoint->ActualPeriod < TtEndpoint->ActualPeriod &&
//         TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         DPRINT("USB2_CheckTtEndpointInsert: Result - TRUE\n");
//         return TRUE;
//     }
//
//     if ((nextTtEndpoint->ActualPeriod <= TtEndpoint->ActualPeriod &&
//         TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS) ||
//         nextTtEndpoint == TtEndpoint)
//     {
//         DPRINT("USB2_CheckTtEndpointInsert: Result - TRUE\n");
//         return TRUE;
//     }
//
//     DPRINT("USB2_CheckTtEndpointInsert: Result - FALSE\n");
//     return FALSE;
// }
//
// ULONG
// NTAPI
// USB2_GetOverhead(IN PUSB2_TT_ENDPOINT TtEndpoint)
// {
//     ULONG TransferType;
//     ULONG Direction;
//     ULONG DeviceSpeed;
//     ULONG Overhead;
//     ULONG HostDelay;
//
//     TransferType = TtEndpoint->TtEndpointParams.TransferType;
//     Direction = TtEndpoint->TtEndpointParams.Direction;
//     DeviceSpeed = TtEndpoint->TtEndpointParams.DeviceSpeed;
//
//     HostDelay = TtEndpoint->Tt->HcExtension->HcDelayTime;
//
//     if (DeviceSpeed == UsbHighSpeed)
//     {
//         if (Direction == USBPORT_TRANSFER_DIRECTION_OUT)
//         {
//             if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//                 Overhead = HostDelay + USB2_HS_ISOCHRONOUS_OUT_OVERHEAD;
//             else
//                 Overhead = HostDelay + USB2_HS_INTERRUPT_OUT_OVERHEAD;
//         }
//         else
//         {
//             if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//                 Overhead = HostDelay + USB2_HS_ISOCHRONOUS_IN_OVERHEAD;
//             else
//                 Overhead = HostDelay + USB2_HS_INTERRUPT_IN_OVERHEAD;
//         }
//     }
//     else if (DeviceSpeed == UsbFullSpeed)
//     {
//         if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//             Overhead = HostDelay + USB2_FS_ISOCHRONOUS_OVERHEAD;
//         else
//             Overhead = HostDelay + USB2_FS_INTERRUPT_OVERHEAD;
//     }
//     else
//     {
//         Overhead = HostDelay + USB2_LS_INTERRUPT_OVERHEAD;
//     }
//
//     return Overhead;
// }
//
// VOID
// NTAPI
// USB2_GetHsOverhead(IN PUSB2_TT_ENDPOINT TtEndpoint,
//                    IN PULONG OverheadSS,
//                    IN PULONG OverheadCS)
// {
//     ULONG TransferType;
//     ULONG Direction;
//     ULONG HostDelay;
//
//     TransferType = TtEndpoint->TtEndpointParams.TransferType;
//     Direction = TtEndpoint->TtEndpointParams.Direction;
//
//     HostDelay = TtEndpoint->Tt->HcExtension->HcDelayTime;
//
//     if (Direction == USBPORT_TRANSFER_DIRECTION_OUT)
//     {
//         if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         {
//             *OverheadSS = HostDelay + USB2_HS_SS_ISOCHRONOUS_OUT_OVERHEAD;
//             *OverheadCS = 0;
//         }
//         else
//         {
//             *OverheadSS = HostDelay + USB2_HS_SS_INTERRUPT_OUT_OVERHEAD;
//             *OverheadCS = HostDelay + USB2_HS_CS_INTERRUPT_OUT_OVERHEAD;
//         }
//     }
//     else
//     {
//         if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         {
//             *OverheadSS = HostDelay + USB2_HS_SS_ISOCHRONOUS_IN_OVERHEAD;
//             *OverheadCS = HostDelay + USB2_HS_CS_ISOCHRONOUS_IN_OVERHEAD;
//         }
//         else
//         {
//             *OverheadSS = HostDelay + USB2_HS_SS_INTERRUPT_IN_OVERHEAD;
//             *OverheadCS = HostDelay + USB2_HS_CS_INTERRUPT_IN_OVERHEAD;
//         }
//     }
// }
//
// ULONG
// NTAPI
// USB2_GetLastIsoTime(IN PUSB2_TT_ENDPOINT TtEndpoint,
//                     IN ULONG Frame)
// {
//     PUSB2_TT_ENDPOINT nextTtEndpoint;
//     ULONG Result;
//
//     //DPRINT("USB2_GetLastIsoTime: TtEndpoint - %p, Frame - %X\n",
//     //       TtEndpoint,
//     //       Frame);
//
//     nextTtEndpoint = TtEndpoint->Tt->FrameBudget[Frame].IsoEndpoint->NextTtEndpoint;
//
//     if (nextTtEndpoint ||
//         (nextTtEndpoint = TtEndpoint->Tt->FrameBudget[Frame].AltEndpoint) != NULL)
//     {
//         Result = nextTtEndpoint->StartTime + nextTtEndpoint->CalcBusTime;
//     }
//     else
//     {
//         Result = USB2_FS_SOF_TIME;
//     }
//
//     return Result;
// }
//
// ULONG
// NTAPI
// USB2_GetStartTime(IN PUSB2_TT_ENDPOINT nextTtEndpoint,
//                   IN PUSB2_TT_ENDPOINT TtEndpoint,
//                   IN PUSB2_TT_ENDPOINT prevTtEndpoint,
//                   IN ULONG Frame)
// {
//     PUSB2_TT_ENDPOINT ttEndpoint;
//     ULONG TransferType;
//
//     DPRINT("USB2_GetStartTime: nextTtEndpoint - %p, TtEndpoint - %p, prevTtEndpoint - %p, Frame - %X\n",
//            nextTtEndpoint,
//            TtEndpoint,
//            prevTtEndpoint,
//            Frame);
//
//     TransferType = TtEndpoint->TtEndpointParams.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//     {
//         if (nextTtEndpoint)
//             return nextTtEndpoint->StartTime + nextTtEndpoint->CalcBusTime;
//
//         ttEndpoint = TtEndpoint->Tt->FrameBudget[Frame].AltEndpoint;
//
//         if (ttEndpoint)
//             return ttEndpoint->StartTime + ttEndpoint->CalcBusTime;
//         else
//             return USB2_FS_SOF_TIME;
//     }
//     else
//     {
//         ttEndpoint = prevTtEndpoint;
//
//         if (ttEndpoint == TtEndpoint->Tt->FrameBudget[Frame].IntEndpoint)
//             return USB2_GetLastIsoTime(TtEndpoint, Frame);
//         else
//             return ttEndpoint->StartTime + ttEndpoint->CalcBusTime;
//     }
// }
//
// VOID
// NTAPI
// USB2_InitTtEndpoint(IN PUSB2_TT_ENDPOINT TtEndpoint,
//                     IN UCHAR TransferType,
//                     IN UCHAR Direction,
//                     IN UCHAR DeviceSpeed,
//                     IN USHORT Period,
//                     IN USHORT MaxPacketSize,
//                     IN PUSB2_TT Tt)
// {
//     TtEndpoint->TtEndpointParams.TransferType = TransferType;
//     TtEndpoint->TtEndpointParams.Direction = Direction;
//     TtEndpoint->TtEndpointParams.DeviceSpeed = DeviceSpeed;
//
//     TtEndpoint->Period = Period;
//     TtEndpoint->MaxPacketSize = MaxPacketSize;
//     TtEndpoint->Tt = Tt;
//
//     TtEndpoint->CalcBusTime = 0;
//     TtEndpoint->StartTime = 0;
//     TtEndpoint->ActualPeriod = 0;
//     TtEndpoint->StartFrame = 0;
//     TtEndpoint->StartMicroframe = 0;
//
//     TtEndpoint->Nums.AsULONG = 0;
//     TtEndpoint->NextTtEndpoint = NULL;
//     TtEndpoint->Reserved2 = 0;
//     TtEndpoint->PreviosPeriod = 0;
//     TtEndpoint->IsPromoted = FALSE;
// }
//
// BOOLEAN
// NTAPI
// USB2_AllocateHS(IN PUSB2_TT_ENDPOINT TtEndpoint,
//                 IN LONG Frame)
// {
//     PUSB2_HC_EXTENSION HcExtension;
//     PUSB2_TT Tt;
//     ULONG TransferType;
//     ULONG Direction;
//     ULONG DataTime;
//     ULONG DataSize;
//     ULONG RemainDataTime;
//     ULONG OverheadCS;
//     ULONG OverheadSS;
//     ULONG ix;
//     USHORT PktSize;
//     USHORT PktSizeBitStuff;
//     UCHAR frame;
//     UCHAR uframe;
//     BOOL Result = TRUE;
//
//     DPRINT("USB2_AllocateHS: TtEndpoint - %p, Frame - %X, StartFrame - %X\n",
//            TtEndpoint,
//            Frame,
//            TtEndpoint->StartFrame);
//
//     Tt = TtEndpoint->Tt;
//     HcExtension = Tt->HcExtension;
//
//     TransferType = TtEndpoint->TtEndpointParams.TransferType;
//     Direction = TtEndpoint->TtEndpointParams.Direction;
//
//     if (Frame == 0)
//     {
//         TtEndpoint->StartMicroframe =
//         TtEndpoint->StartTime / USB2_FS_RAW_BYTES_IN_MICROFRAME - 1;
//
//         DPRINT("USB2_AllocateHS: TtEndpoint->StartMicroframe - %X\n",
//                TtEndpoint->StartMicroframe);
//     }
//
//     USB2_GetHsOverhead(TtEndpoint, &OverheadSS, &OverheadCS);
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         if (Frame == 0)
//         {
//             TtEndpoint->Nums.NumStarts = 1;
//
//             if ((CHAR)TtEndpoint->StartMicroframe < (USB2_MICROFRAMES - 3))
//                 TtEndpoint->Nums.NumCompletes = 3;
//             else
//                 TtEndpoint->Nums.NumCompletes = 2;
//         }
//     }
//     else
//     {
//         if (Direction == USBPORT_TRANSFER_DIRECTION_OUT)
//         {
//             DPRINT("USB2_AllocateHS: ISO UNIMPLEMENTED\n");
//             ASSERT(FALSE);
//         }
//         else
//         {
//             DPRINT("USB2_AllocateHS: ISO UNIMPLEMENTED\n");
//             ASSERT(FALSE);
//         }
//     }
//
//     frame = TtEndpoint->StartFrame + Frame;
//     uframe = TtEndpoint->StartMicroframe;
//
//     if (TtEndpoint->StartMicroframe == USB2_PREV_MICROFRAME)
//         USB2_GetPrevMicroFrame(&frame, &uframe);
//
//     for (ix = 0; ix < TtEndpoint->Nums.NumStarts; ix++)
//     {
//         if (!USB2_AllocateCheck(&HcExtension->TimeUsed[frame][uframe],
//                                 OverheadSS,
//                                 USB2_MAX_MICROFRAME_ALLOCATION))
//         {
//             Result = FALSE;
//         }
//
//         if (Tt->NumStartSplits[frame][uframe] >
//             (USB2_MAX_FS_LS_TRANSACTIONS_IN_UFRAME - 1))
//         {
//             DPRINT1("USB2_AllocateHS: Num Start Splits - %X\n",
//                     Tt->NumStartSplits[frame][uframe] + 1);
//
//             ASSERT(FALSE);
//             Result = FALSE;
//         }
//
//         ++Tt->NumStartSplits[frame][uframe];
//         USB2_IncMicroFrame(&frame, &uframe);
//     }
//
//     frame = TtEndpoint->StartFrame + Frame;
//     uframe = TtEndpoint->StartMicroframe + TtEndpoint->Nums.NumStarts + 1;
//
//     for (ix = 0; ix < TtEndpoint->Nums.NumCompletes; ix++)
//     {
//         if (!USB2_AllocateCheck(&HcExtension->TimeUsed[frame][uframe],
//                                 OverheadCS,
//                                 USB2_MAX_MICROFRAME_ALLOCATION))
//         {
//             Result = FALSE;
//         }
//
//         USB2_IncMicroFrame(&frame, &uframe);
//     }
//
//     PktSize = TtEndpoint->MaxPacketSize;
//     PktSizeBitStuff = USB2_AddDataBitStuff(PktSize);
//
//     if (Direction == USBPORT_TRANSFER_DIRECTION_OUT)
//     {
//         frame = TtEndpoint->StartFrame + Frame;
//         uframe = TtEndpoint->StartMicroframe;
//
//         if (uframe == USB2_PREV_MICROFRAME)
//             USB2_GetPrevMicroFrame(&frame, &uframe);
//
//         DataTime = 0;
//
//         for (ix = 0; ix < TtEndpoint->Nums.NumStarts; ix++)
//         {
//             DataSize = PktSizeBitStuff - DataTime;
//
//             if (DataSize <= USB2_FS_RAW_BYTES_IN_MICROFRAME)
//                 DataTime = DataSize;
//             else
//                 DataTime = USB2_FS_RAW_BYTES_IN_MICROFRAME;
//
//             DPRINT("USB2_AllocateHS: ix - %X, frame - %X, uframe - %X, TimeUsed - %X\n",
//                    ix,
//                    frame,
//                    uframe,
//                    HcExtension->TimeUsed[frame][uframe]);
//
//             if (!USB2_AllocateCheck(&HcExtension->TimeUsed[frame][uframe],
//                                     DataTime,
//                                     USB2_MAX_MICROFRAME_ALLOCATION))
//             {
//                 Result = FALSE;
//             }
//
//             USB2_IncMicroFrame(&frame, &uframe);
//             DataTime += USB2_FS_RAW_BYTES_IN_MICROFRAME;
//         }
//     }
//     else
//     {
//         frame = TtEndpoint->StartFrame + Frame;
//         uframe = TtEndpoint->StartMicroframe + TtEndpoint->Nums.NumStarts + 1;
//
//         for (ix = 0; ix < TtEndpoint->Nums.NumCompletes; ix++)
//         {
//             if (Tt->TimeCS[frame][uframe] < USB2_FS_RAW_BYTES_IN_MICROFRAME)
//             {
//                 RemainDataTime = USB2_FS_RAW_BYTES_IN_MICROFRAME -
//                                  Tt->TimeCS[frame][uframe];
//
//                 if (RemainDataTime >= PktSizeBitStuff)
//                 {
//                     DataTime = PktSizeBitStuff;
//                 }
//                 else if (RemainDataTime > 0)
//                 {
//                     DataTime = RemainDataTime;
//                 }
//                 else
//                 {
//                     DataTime = 0;
//                 }
//
//                 if (!USB2_AllocateCheck(&HcExtension->TimeUsed[frame][uframe],
//                                         DataTime,
//                                         USB2_MAX_MICROFRAME_ALLOCATION))
//                 {
//                     Result = FALSE;
//                 }
//             }
//
//             if (PktSizeBitStuff < USB2_FS_RAW_BYTES_IN_MICROFRAME)
//                 Tt->TimeCS[frame][uframe] += PktSizeBitStuff;
//             else
//                 Tt->TimeCS[frame][uframe] += USB2_FS_RAW_BYTES_IN_MICROFRAME;
//
//             USB2_IncMicroFrame(&frame, &uframe);
//         }
//     }
//
//     DPRINT("USB2_AllocateHS: Result - %X\n", Result);
//     return Result;
// }
//
// VOID
// NTAPI
// USB2_DeallocateHS(IN PUSB2_TT_ENDPOINT TtEndpoint,
//                   IN ULONG Frame)
// {
//     PUSB2_TT Tt;
//     PUSB2_HC_EXTENSION HcExtension;
//     ULONG OverheadCS;
//     ULONG OverheadSS;
//     ULONG Direction;
//     ULONG ix;
//     ULONG CurrentDataTime;
//     ULONG RemainDataTime;
//     ULONG DataTime;
//     ULONG DataSize;
//     USHORT PktSize;
//     USHORT PktSizeBitStuff;
//     UCHAR uframe;
//     UCHAR frame;
//
//     DPRINT("USB2_DeallocateHS: TtEndpoint - %p, Frame - %X\n",
//            TtEndpoint,
//            Frame);
//
//     Tt = TtEndpoint->Tt;
//     HcExtension = Tt->HcExtension;
//
//     USB2_GetHsOverhead(TtEndpoint, &OverheadSS, &OverheadCS);
//
//     frame = TtEndpoint->StartFrame + Frame;
//     uframe = TtEndpoint->StartMicroframe;
//
//     if (TtEndpoint->StartMicroframe == USB2_PREV_MICROFRAME)
//         USB2_GetPrevMicroFrame(&frame, &uframe);
//
//     for (ix = 0; ix < TtEndpoint->Nums.NumStarts; ix++)
//     {
//         HcExtension->TimeUsed[frame][uframe] -= OverheadSS;
//         --Tt->NumStartSplits[frame][uframe];
//         USB2_IncMicroFrame(&frame, &uframe);
//     }
//
//     frame = TtEndpoint->StartFrame + Frame;
//     uframe = TtEndpoint->StartMicroframe + TtEndpoint->Nums.NumStarts + 1;
//
//     for (ix = 0; ix < TtEndpoint->Nums.NumCompletes; ix++)
//     {
//         HcExtension->TimeUsed[frame][uframe] -= OverheadCS;
//         USB2_IncMicroFrame(&frame, &uframe);
//     }
//
//     Direction = TtEndpoint->TtEndpointParams.Direction;
//     PktSize = TtEndpoint->MaxPacketSize;
//     PktSizeBitStuff = USB2_AddDataBitStuff(PktSize);
//
//     if (Direction == USBPORT_TRANSFER_DIRECTION_OUT)
//     {
//         frame = TtEndpoint->StartFrame + Frame;
//         uframe = TtEndpoint->StartMicroframe;
//
//         if (TtEndpoint->StartMicroframe == USB2_PREV_MICROFRAME)
//             USB2_GetPrevMicroFrame(&frame, &uframe);
//
//         DataTime = 0;
//
//         for (ix = 0; ix < TtEndpoint->Nums.NumStarts; ix++)
//         {
//             DataSize = PktSizeBitStuff - DataTime;
//
//             if (DataSize <= USB2_FS_RAW_BYTES_IN_MICROFRAME)
//                 CurrentDataTime = PktSizeBitStuff - DataTime;
//             else
//                 CurrentDataTime = USB2_FS_RAW_BYTES_IN_MICROFRAME;
//
//             HcExtension->TimeUsed[frame][uframe] -= CurrentDataTime;
//             USB2_IncMicroFrame(&frame, &uframe);
//             DataTime += USB2_FS_RAW_BYTES_IN_MICROFRAME;
//         }
//     }
//     else
//     {
//         frame = TtEndpoint->StartFrame + Frame;
//         uframe = TtEndpoint->StartMicroframe + TtEndpoint->Nums.NumStarts + 1;
//
//         for (ix = 0; ix < TtEndpoint->Nums.NumCompletes; ix++)
//         {
//             if (PktSizeBitStuff >= USB2_FS_RAW_BYTES_IN_MICROFRAME)
//                 CurrentDataTime = USB2_FS_RAW_BYTES_IN_MICROFRAME;
//             else
//                 CurrentDataTime = PktSizeBitStuff;
//
//             Tt->TimeCS[frame][uframe] -= CurrentDataTime;
//
//             if (Tt->TimeCS[frame][uframe] < USB2_FS_RAW_BYTES_IN_MICROFRAME)
//             {
//                 RemainDataTime = USB2_FS_RAW_BYTES_IN_MICROFRAME -
//                                  Tt->TimeCS[frame][uframe];
//
//                 if (RemainDataTime >= PktSizeBitStuff)
//                     RemainDataTime = PktSizeBitStuff;
//
//                 HcExtension->TimeUsed[frame][uframe] -= RemainDataTime;
//             }
//
//             USB2_IncMicroFrame(&frame, &uframe);
//         }
//     }
//
//     return;
// }
//
// BOOLEAN
// NTAPI
// USB2_MoveTtEndpoint(IN PUSB2_TT_ENDPOINT TtEndpoint,
//                     IN LONG BusTime,
//                     IN PUSB2_REBALANCE Rebalance,
//                     IN ULONG RebalanceListEntries,
//                     OUT BOOLEAN * OutResult)
// {
//     ULONG EndBusTime;
//     ULONG TransferType;
//     ULONG Num;
//     UCHAR ix;
//
//     DPRINT("USB2_MoveTtEndpoint: TtEndpoint - %p, BusTime - %X\n",
//            TtEndpoint,
//            BusTime);
//
//     *OutResult = TRUE;
//
//     for (Num = 0; Rebalance->RebalanceEndpoint[Num]; Num++)
//     {
//         if (Rebalance->RebalanceEndpoint[Num] == TtEndpoint)
//             break;
//     }
//
//     DPRINT("USB2_MoveTtEndpoint: Num - %X\n", Num);
//
//     TransferType = TtEndpoint->TtEndpointParams.TransferType;
//
//     if (Rebalance->RebalanceEndpoint[Num] &&
//         TtEndpoint->TtEndpointParams.EndpointMoved &&
//         ((TransferType != USBPORT_TRANSFER_TYPE_INTERRUPT) || BusTime >= 0))
//     {
//         DPRINT("USB2_MoveTtEndpoint: result - FALSE\n");
//         return FALSE;
//     }
//
//     for (ix = 0;
//          (TtEndpoint->StartFrame + ix) < USB2_FRAMES;
//          ix += TtEndpoint->ActualPeriod)
//     {
//         USB2_DeallocateHS(TtEndpoint, ix);
//     }
//
//     TtEndpoint->StartTime += BusTime;
//
//     EndBusTime = TtEndpoint->StartTime + TtEndpoint->CalcBusTime;
//
//     if (EndBusTime > USB2_FS_MAX_PERIODIC_ALLOCATION)
//     {
//         DPRINT("USB2_MoveTtEndpoint: EndBusTime is too large!\n");
//         *OutResult = FALSE;
//     }
//
//     TtEndpoint->TtEndpointParams.EndpointMoved = 1;
//
//     if (Rebalance->RebalanceEndpoint[Num] == NULL)
//     {
//         if (Num >= RebalanceListEntries)
//         {
//             DPRINT("USB2_MoveTtEndpoint: Too many changes!\n");
//             *OutResult = FALSE;
//         }
//         else
//         {
//             Rebalance->RebalanceEndpoint[Num] = TtEndpoint;
//             Rebalance->RebalanceEndpoint[Num + 1] = NULL;
//         }
//     }
//
//     for (ix = 0;
//          (TtEndpoint->StartFrame + ix) < USB2_FRAMES;
//          ix += TtEndpoint->ActualPeriod)
//     {
//         if (!USB2_AllocateHS(TtEndpoint, ix))
//         {
//             DPRINT("USB2_MoveTtEndpoint: OutResult - FALSE\n");
//             OutResult = FALSE;
//         }
//     }
//
//     DPRINT("USB2_MoveTtEndpoint: result - TRUE\n");
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// USB2_CommonFrames(IN PUSB2_TT_ENDPOINT NextTtEndpoint,
//                   IN PUSB2_TT_ENDPOINT TtEndpoint)
// {
//     UCHAR Frame;
//
//     DPRINT("USB2_CommonFrames: \n");
//
//     if (NextTtEndpoint->ActualPeriod == ENDPOINT_INTERRUPT_1ms ||
//         TtEndpoint->ActualPeriod == ENDPOINT_INTERRUPT_1ms)
//     {
//         return TRUE;
//     }
//
//     if (NextTtEndpoint->ActualPeriod < TtEndpoint->ActualPeriod)
//         Frame = TtEndpoint->StartFrame % TtEndpoint->ActualPeriod;
//     else
//         Frame = NextTtEndpoint->StartFrame % TtEndpoint->ActualPeriod;
//
//     return (Frame == TtEndpoint->StartFrame);
// }
//
// VOID
// NTAPI
// USB2_ConvertFrame(IN UCHAR Frame,
//                   IN UCHAR Microframe,
//                   OUT PUCHAR HcFrame,
//                   OUT PUCHAR HcMicroframe)
// {
//     DPRINT("USB2_ConvertFrame: Frame - %x, Microframe - %x\n",
//            Frame,
//            Microframe);
//
//     if (Microframe == USB2_PREV_MICROFRAME)
//     {
//         *HcFrame = Frame;
//         *HcMicroframe = 0;
//     }
//
//     if (Microframe <= (USB2_MICROFRAMES - 2))
//     {
//         *HcFrame = Frame;
//         *HcMicroframe = Microframe + 1;
//     }
//
//     if (Microframe == (USB2_MICROFRAMES - 1))
//     {
//         *HcFrame = Frame + 1;
//         *HcMicroframe = 0;
//     }
// }
//
// UCHAR
// NTAPI
// USB2_GetSMASK(IN PUSB2_TT_ENDPOINT TtEndpoint)
// {
//     ULONG ix;
//     UCHAR SMask = 0;
//     UCHAR HcFrame;
//     UCHAR HcMicroFrame;
//
//     if (TtEndpoint->TtEndpointParams.DeviceSpeed == UsbHighSpeed)
//     {
//         SMask = (1 << TtEndpoint->StartMicroframe);
//     }
//     else
//     {
//         USB2_ConvertFrame(TtEndpoint->StartFrame,
//                           TtEndpoint->StartMicroframe,
//                           &HcFrame,
//                           &HcMicroFrame);
//
//         for (ix = 0; ix < TtEndpoint->Nums.NumStarts; ix++)
//         {
//             SMask |= (1 << HcMicroFrame);
//             HcMicroFrame++;
//         }
//     }
//
//     return SMask;
// }
//
// UCHAR
// NTAPI
// USB2_GetCMASK(IN PUSB2_TT_ENDPOINT TtEndpoint)
// {
//     ULONG NumCompletes;
//     ULONG TransferType;
//     ULONG DeviceSpeed;
//     ULONG Direction;
//     UCHAR Result;
//     UCHAR MicroFrameCS;
//     UCHAR HcFrame;
//     UCHAR HcMicroFrame;
//     UCHAR MaskCS = 0;
//
//     TransferType = TtEndpoint->TtEndpointParams.TransferType;
//     DeviceSpeed = TtEndpoint->TtEndpointParams.DeviceSpeed;
//     Direction = TtEndpoint->TtEndpointParams.Direction;
//
//     if (DeviceSpeed == UsbHighSpeed)
//         return 0;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         USB2_ConvertFrame(TtEndpoint->StartFrame,
//                           TtEndpoint->StartMicroframe,
//                           &HcFrame,
//                           &HcMicroFrame);
//
//         Result = CMASKS[HcMicroFrame];
//     }
//     else
//     {
//         if (Direction == USBPORT_TRANSFER_DIRECTION_OUT)
//             return 0;
//
//         USB2_ConvertFrame(TtEndpoint->StartFrame,
//                           TtEndpoint->StartMicroframe,
//                           &HcFrame,
//                           &HcMicroFrame);
//
//         NumCompletes = TtEndpoint->Nums.NumCompletes;
//
//         for (MicroFrameCS = HcMicroFrame + 2;
//              MicroFrameCS < USB2_MICROFRAMES;
//              MicroFrameCS++)
//         {
//             MaskCS |= (1 << MicroFrameCS);
//             NumCompletes--;
//
//             if (!NumCompletes)
//                 return MaskCS;
//         }
//
//         for (; NumCompletes; NumCompletes--)
//         {
//             MaskCS |= (1 << (MicroFrameCS - USB2_MICROFRAMES));
//         }
//
//         Result = MaskCS;
//     }
//
//     return Result;
// }
//
// VOID
// NTAPI
// USB2_RebalanceEndpoint(IN PDEVICE_OBJECT FdoDevice,
//                        IN PLIST_ENTRY List)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PUSBPORT_ENDPOINT Endpoint;
//     PLIST_ENTRY Entry;
//     ULONG AllocedBusTime;
//     ULONG EndpointBandwidth;
//     ULONG Factor;
//     ULONG ScheduleOffset;
//     ULONG Bandwidth;
//     ULONG n;
//     ULONG ix;
//     KIRQL OldIrql;
//     UCHAR NewPeriod;
//     UCHAR SMask;
//     UCHAR CMask;
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     while (!IsListEmpty(List))
//     {
//         Entry = RemoveHeadList(List);
//
//         Endpoint = CONTAINING_RECORD(Entry,
//                                      USBPORT_ENDPOINT,
//                                      RebalanceLink.Flink);
//
//         DPRINT("USB2_RebalanceEndpoint: Endpoint - %p\n", Endpoint);
//
//         Endpoint->RebalanceLink.Flink = NULL;
//         Endpoint->RebalanceLink.Blink = NULL;
//
//         KeAcquireSpinLock(&Endpoint->EndpointSpinLock,
//                           &Endpoint->EndpointOldIrql);
//
//         SMask = USB2_GetSMASK(Endpoint->TtEndpoint);
//         CMask = USB2_GetCMASK(Endpoint->TtEndpoint);
//
//         ScheduleOffset = Endpoint->TtEndpoint->StartFrame;
//         NewPeriod = Endpoint->TtEndpoint->ActualPeriod;
//
//         AllocedBusTime = Endpoint->TtEndpoint->CalcBusTime;
//         EndpointBandwidth = USB2_MICROFRAMES * AllocedBusTime;
//
//         Endpoint->EndpointProperties.InterruptScheduleMask = SMask;
//         Endpoint->EndpointProperties.SplitCompletionMask = CMask;
//
//         if (Endpoint->EndpointProperties.Period != NewPeriod)
//         {
//             ASSERT(Endpoint->EndpointProperties.Period);
//             Factor = USB2_FRAMES / Endpoint->EndpointProperties.Period;
//
//             for (ix = 0; ix < Factor; ix++)
//             {
//                 Bandwidth = Endpoint->EndpointProperties.UsbBandwidth;
//                 n = Factor * Endpoint->EndpointProperties.ScheduleOffset;
//                 Endpoint->TtExtension->Bandwidth[n + ix] += Bandwidth;
//             }
//
//             Endpoint->EndpointProperties.Period = NewPeriod;
//             Endpoint->EndpointProperties.ScheduleOffset = ScheduleOffset;
//             Endpoint->EndpointProperties.UsbBandwidth = EndpointBandwidth;
//
//             ASSERT(NewPeriod);
//             Factor = USB2_FRAMES / NewPeriod;
//
//             for (ix = 0; ix < Factor; ix++)
//             {
//                 n = Factor * ScheduleOffset;
//                 Endpoint->TtExtension->Bandwidth[n + ix] += EndpointBandwidth;
//             }
//         }
//
//         KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//         Packet->RebalanceEndpoint(FdoExtension->MiniPortExt,
//                                   &Endpoint->EndpointProperties,
//                                   (PVOID)((ULONG_PTR)Endpoint + sizeof(USBPORT_ENDPOINT)));
//
//         KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//         KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                           Endpoint->EndpointOldIrql);
//     }
// }
//
// VOID
// NTAPI
// USB2_Rebalance(IN PDEVICE_OBJECT FdoDevice,
//                IN PLIST_ENTRY RebalanceList)
// {
//     PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties;
//     PUSBPORT_ENDPOINT Endpoint;
//     PLIST_ENTRY Entry;
//     LIST_ENTRY BalanceListInt1;
//     LIST_ENTRY BalanceListInt2;
//     ULONG TransferType;
//     ULONG ScheduleOffset;
//     UCHAR SMask;
//     UCHAR CMask;
//     UCHAR ActualPeriod;
//
//     DPRINT("USB2_Rebalance: FdoDevice - %p, RebalanceList - %p\n",
//            FdoDevice,
//            RebalanceList);
//
//     InitializeListHead(&BalanceListInt1);
//     InitializeListHead(&BalanceListInt2);
//
//     while (!IsListEmpty(RebalanceList))
//     {
//         Entry = RebalanceList->Flink;
//
//         Endpoint = CONTAINING_RECORD(Entry,
//                                      USBPORT_ENDPOINT,
//                                      RebalanceLink.Flink);
//
//         DPRINT("USBPORT_Rebalance: Entry - %p, Endpoint - %p\n",
//                Entry,
//                Endpoint);
//
//         RemoveHeadList(RebalanceList);
//         Entry->Flink = NULL;
//         Entry->Blink = NULL;
//
//         SMask = USB2_GetSMASK(Endpoint->TtEndpoint);
//         CMask = USB2_GetCMASK(Endpoint->TtEndpoint);
//
//         ScheduleOffset = Endpoint->TtEndpoint->StartFrame;
//         ActualPeriod = Endpoint->TtEndpoint->ActualPeriod;
//
//         EndpointProperties = &Endpoint->EndpointProperties;
//         TransferType = EndpointProperties->TransferType;
//
//         switch (TransferType)
//         {
//             case USBPORT_TRANSFER_TYPE_ISOCHRONOUS:
//                 DPRINT("USBPORT_Rebalance: USBPORT_TRANSFER_TYPE_ISOCHRONOUS. FIXME\n");
//                 ASSERT(FALSE);
//                 break;
//
//             case USBPORT_TRANSFER_TYPE_INTERRUPT:
//                 if (SMask != EndpointProperties->InterruptScheduleMask ||
//                     CMask != EndpointProperties->SplitCompletionMask ||
//                     ScheduleOffset != EndpointProperties->ScheduleOffset ||
//                     ActualPeriod != EndpointProperties->Period)
//                 {
//                     if (ActualPeriod == EndpointProperties->Period &&
//                         ScheduleOffset == EndpointProperties->ScheduleOffset)
//                     {
//                         InsertTailList(&BalanceListInt1, Entry);
//                     }
//                     else
//                     {
//                         InsertTailList(&BalanceListInt2, Entry);
//                     }
//                 }
//                 break;
//
//             default:
//                 ASSERT(FALSE);
//                 break;
//         }
//     }
//
//     USB2_RebalanceEndpoint(FdoDevice, &BalanceListInt2);
//     USB2_RebalanceEndpoint(FdoDevice, &BalanceListInt1);
//     //USB2_RebalanceEndpoint(FdoDevice, &BalanceListIso);
// }
//
// BOOLEAN
// NTAPI
// USB2_DeallocateEndpointBudget(IN PUSB2_TT_ENDPOINT TtEndpoint,
//                               IN PUSB2_REBALANCE Rebalance,
//                               IN PULONG RebalanceListEntries,
//                               IN ULONG MaxFrames)
// {
//     PUSB2_TT Tt;
//     PUSB2_HC_EXTENSION HcExtension;
//     ULONG Speed;
//     ULONG TransferType;
//     ULONG Frame;
//     ULONG StartMicroframe;
//     ULONG ix;
//     PUSB2_TT_ENDPOINT endpoint;
//     PUSB2_TT_ENDPOINT nextEndpoint;
//     PUSB2_TT_ENDPOINT lastEndpoint;
//     PUSB2_TT_ENDPOINT tmpEndpoint;
//     ULONG endTime;
//     ULONG maxEndTime;
//     ULONG lastEndTime;
//     ULONG Factor;
//     ULONG jx;
//     UCHAR frame;
//     UCHAR uframe;
//     USHORT Period;
//     BOOLEAN IsMoved = FALSE;
//
//     DPRINT("USB2_DeallocateEndpointBudget: TtEndpoint - %p, MaxFrames - %X, CalcBusTime - %X\n",
//            TtEndpoint,
//            MaxFrames,
//            TtEndpoint->CalcBusTime);
//
//     if (TtEndpoint->CalcBusTime == 0)
//     {
//         DPRINT("USB2_DeallocateEndpointBudget: TtEndpoint not allocated!\n");
//         return FALSE;
//     }
//
//     Tt = TtEndpoint->Tt;
//     HcExtension = Tt->HcExtension;
//
//     Speed = TtEndpoint->TtEndpointParams.DeviceSpeed;
//     DPRINT("USB2_DeallocateEndpointBudget: DeviceSpeed - %X\n", Speed);
//
//     StartMicroframe = TtEndpoint->StartFrame * USB2_MICROFRAMES +
//                       TtEndpoint->StartMicroframe;
//
//     if (Speed == UsbHighSpeed)
//     {
//         for (ix = StartMicroframe;
//              ix < USB2_MAX_MICROFRAMES;
//              ix += TtEndpoint->ActualPeriod)
//         {
//             frame = ix / USB2_MICROFRAMES;
//             uframe = ix % (USB2_MICROFRAMES - 1);
//
//             HcExtension->TimeUsed[frame][uframe] -= TtEndpoint->CalcBusTime;
//         }
//
//         TtEndpoint->CalcBusTime = 0;
//
//         DPRINT("USB2_DeallocateEndpointBudget: return TRUE\n");
//         return TRUE;
//     }
//
//     /* Speed != UsbHighSpeed (FS/LS) */
//
//     TransferType = TtEndpoint->TtEndpointParams.TransferType;
//
//     for (ix = MaxFrames, Frame = (MaxFrames - 1) - TtEndpoint->StartFrame;
//          ix > 0;
//          ix--, Frame--)
//     {
//         frame = TtEndpoint->StartFrame + Frame;
//
//         DPRINT("USB2_DeallocateEndpointBudget: frame - %X, Frame - %X, StartFrame - %X\n",
//                frame,
//                Frame,
//                TtEndpoint->StartFrame);
//
//         if ((Frame % TtEndpoint->ActualPeriod) == 0)
//         {
//             USB2_DeallocateHS(TtEndpoint, Frame);
//             Tt->FrameBudget[frame].TimeUsed -= TtEndpoint->CalcBusTime;
//         }
//
//         if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//             endpoint = Tt->FrameBudget[frame].IntEndpoint;
//         else
//             endpoint = Tt->FrameBudget[frame].IsoEndpoint;
//
//         nextEndpoint = endpoint->NextTtEndpoint;
//
//         DPRINT("USB2_DeallocateEndpointBudget: TtEndpoint - %p, nextEndpoint - %p\n",
//                TtEndpoint,
//                nextEndpoint);
//
//         if (TtEndpoint->CalcBusTime > (USB2_FS_MAX_PERIODIC_ALLOCATION / 2))
//         {
//             while (nextEndpoint)
//             {
//                 endpoint = nextEndpoint;
//                 nextEndpoint = nextEndpoint->NextTtEndpoint;
//             }
//
//             nextEndpoint = TtEndpoint;
//
//             DPRINT("USB2_DeallocateEndpointBudget: endpoint - %p, nextEndpoint - %p\n",
//                    endpoint,
//                    nextEndpoint);
//         }
//         else
//         {
//             while (nextEndpoint &&
//                    !USB2_CheckTtEndpointInsert(nextEndpoint, TtEndpoint))
//             {
//                 endpoint = nextEndpoint;
//                 nextEndpoint = nextEndpoint->NextTtEndpoint;
//             }
//
//             if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS &&
//                 nextEndpoint)
//             {
//                 DPRINT1("USB2_DeallocateEndpointBudget: Iso Ep UNIMPLEMENTED. FIXME\n");
//                 ASSERT(FALSE);
//             }
//
//             DPRINT("USB2_DeallocateEndpointBudget: endpoint - %p, nextEndpoint - %p\n",
//                    endpoint,
//                    nextEndpoint);
//         }
//
//         if ((Frame % TtEndpoint->ActualPeriod) == 0)
//         {
//             if (TtEndpoint->CalcBusTime > (USB2_FS_MAX_PERIODIC_ALLOCATION / 2))
//             {
//                 Tt->FrameBudget[frame].AltEndpoint = NULL;
//             }
//             else if (nextEndpoint)
//             {
//                 nextEndpoint = nextEndpoint->NextTtEndpoint;
//                 endpoint->NextTtEndpoint = nextEndpoint;
//
//                 DPRINT("USB2_DeallocateEndpointBudget: endpoint - %p, nextEndpoint - %p\n",
//                        endpoint,
//                        nextEndpoint);
//             }
//         }
//
//         if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//         {
//             if (endpoint == Tt->FrameBudget[frame].IntEndpoint)
//             {
//                 if (Tt->FrameBudget[frame].IsoEndpoint->NextTtEndpoint)
//                 {
//                     endpoint = Tt->FrameBudget[frame].IsoEndpoint->NextTtEndpoint;
//                 }
//                 else if (Tt->FrameBudget[frame].AltEndpoint)
//                 {
//                     endpoint = Tt->FrameBudget[frame].AltEndpoint;
//                 }
//             }
//         }
//         else
//         {
//             DPRINT1("USB2_DeallocateEndpointBudget: Iso Ep UNIMPLEMENTED. FIXME\n");
//             ASSERT(FALSE);
//         }
//
//         Period = TtEndpoint->ActualPeriod;
//
//         for (;
//              nextEndpoint;
//              endpoint = nextEndpoint,
//              nextEndpoint = nextEndpoint->NextTtEndpoint)
//         {
//             DPRINT("USB2_DeallocateEndpointBudget: endpoint - %p, nextEndpoint - %p\n",
//                    endpoint,
//                    nextEndpoint);
//
//             endTime = endpoint->StartTime + endpoint->CalcBusTime;
//             maxEndTime = endTime;
//
//             if (Period > nextEndpoint->ActualPeriod ||
//                 TtEndpoint->StartFrame != nextEndpoint->StartFrame)
//             {
//                 if (USB2_CommonFrames(nextEndpoint, TtEndpoint))
//                     Factor = Period / nextEndpoint->ActualPeriod;
//                 else
//                     Factor = USB2_FRAMES / nextEndpoint->ActualPeriod;
//
//                 maxEndTime = endTime;
//
//                 for (jx = 0, frame = nextEndpoint->StartFrame;
//                      jx < Factor;
//                      jx++, frame += nextEndpoint->ActualPeriod)
//                 {
//                     if (nextEndpoint->StartFrame != TtEndpoint->StartFrame)
//                     {
//                         lastEndpoint = Tt->FrameBudget[frame].IntEndpoint;
//
//                         if (Tt->FrameBudget[frame].IsoEndpoint->NextTtEndpoint)
//                         {
//                             lastEndpoint = Tt->FrameBudget[frame].IsoEndpoint->NextTtEndpoint;
//                         }
//                         else if (Tt->FrameBudget[frame].AltEndpoint)
//                         {
//                             lastEndpoint = Tt->FrameBudget[frame].AltEndpoint;
//                         }
//
//                         for (tmpEndpoint = Tt->FrameBudget[frame].IntEndpoint->NextTtEndpoint;
//                              tmpEndpoint && tmpEndpoint != nextEndpoint;
//                              tmpEndpoint = tmpEndpoint->NextTtEndpoint)
//                         {
//                             lastEndpoint = tmpEndpoint;
//                         }
//
//                         lastEndTime = lastEndpoint->StartTime + lastEndpoint->CalcBusTime;
//
//                         if (endTime < (lastEndTime - 1))
//                         {
//                             maxEndTime = lastEndTime;
//                             endTime = maxEndTime;
//
//                             if (nextEndpoint->StartTime == maxEndTime)
//                                 break;
//                         }
//                         else
//                         {
//                             maxEndTime = endTime;
//                         }
//                     }
//                 }
//             }
//
//             if (maxEndTime >= nextEndpoint->StartTime)
//                 break;
//
//             if (!USB2_MoveTtEndpoint(nextEndpoint,
//                                      maxEndTime - nextEndpoint->StartTime,
//                                      Rebalance,
//                                      *RebalanceListEntries,
//                                      &IsMoved))
//             {
//                 if (!IsMoved)
//                 {
//                     DPRINT("USB2_DeallocateEndpointBudget: Not moved!\n");
//                 }
//
//                 break;
//             }
//
//             if (Period > nextEndpoint->ActualPeriod)
//                 Period = nextEndpoint->ActualPeriod;
//         }
//     }
//
//     TtEndpoint->CalcBusTime = 0;
//
//     DPRINT("USB2_DeallocateEndpointBudget: return TRUE\n");
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// USB2_AllocateTimeForEndpoint(IN PUSB2_TT_ENDPOINT TtEndpoint,
//                              IN PUSB2_REBALANCE Rebalance,
//                              IN PULONG RebalanceListEntries)
// {
//     PUSB2_TT Tt;
//     PUSB2_HC_EXTENSION HcExtension;
//     ULONG Speed;
//     ULONG TimeUsed;
//     ULONG MinTimeUsed;
//     ULONG ix;
//     ULONG frame;
//     ULONG uframe;
//     ULONG Microframe;
//     ULONG TransferType;
//     ULONG Overhead;
//     ULONG LatestStart;
//     PUSB2_TT_ENDPOINT prevEndpoint;
//     PUSB2_TT_ENDPOINT nextEndpoint;
//     PUSB2_TT_ENDPOINT IntEndpoint;
//     ULONG StartTime;
//     ULONG calcBusTime;
//     BOOLEAN Result = TRUE;
//
//     DPRINT("USB2_AllocateTimeForEndpoint: TtEndpoint - %p\n", TtEndpoint);
//
//     Tt = TtEndpoint->Tt;
//     HcExtension = Tt->HcExtension;
//
//     TtEndpoint->Nums.NumStarts = 0;
//     TtEndpoint->Nums.NumCompletes = 0;
//
//     TtEndpoint->StartFrame = 0;
//     TtEndpoint->StartMicroframe = 0;
//
//     if (TtEndpoint->CalcBusTime)
//     {
//         DPRINT("USB2_AllocateTimeForEndpoint: TtEndpoint already allocated!\n");
//         return FALSE;
//     }
//
//     Speed = TtEndpoint->TtEndpointParams.DeviceSpeed;
//
//     if (Speed == UsbHighSpeed)
//     {
//         if (TtEndpoint->Period > USB2_MAX_MICROFRAMES)
//             TtEndpoint->ActualPeriod = USB2_MAX_MICROFRAMES;
//         else
//             TtEndpoint->ActualPeriod = TtEndpoint->Period;
//
//         MinTimeUsed = HcExtension->TimeUsed[0][0];
//
//         for (ix = 1; ix < TtEndpoint->ActualPeriod; ix++)
//         {
//             frame = ix / USB2_MICROFRAMES;
//             uframe = ix % (USB2_MICROFRAMES - 1);
//
//             TimeUsed = HcExtension->TimeUsed[frame][uframe];
//
//             if (TimeUsed < MinTimeUsed)
//             {
//                 MinTimeUsed = TimeUsed;
//                 TtEndpoint->StartFrame = frame;
//                 TtEndpoint->StartMicroframe = uframe;
//             }
//         }
//
//         TtEndpoint->CalcBusTime = USB2_GetOverhead(TtEndpoint) +
//                                   USB2_AddDataBitStuff(TtEndpoint->MaxPacketSize);
//
//         DPRINT("USB2_AllocateTimeForEndpoint: StartFrame - %X, StartMicroframe - %X, CalcBusTime - %X\n",
//                TtEndpoint->StartFrame,
//                TtEndpoint->StartMicroframe,
//                TtEndpoint->CalcBusTime);
//
//         Microframe = TtEndpoint->StartFrame * USB2_MICROFRAMES +
//                      TtEndpoint->StartMicroframe;
//
//         if (Microframe >= USB2_MAX_MICROFRAMES)
//         {
//             DPRINT("USB2_AllocateTimeForEndpoint: Microframe >= 256. Result - TRUE\n");
//             return TRUE;
//         }
//
//         for (ix = Microframe;
//              ix < USB2_MAX_MICROFRAMES;
//              ix += TtEndpoint->ActualPeriod)
//         {
//             frame = ix / USB2_MICROFRAMES;
//             uframe = ix % (USB2_MICROFRAMES - 1);
//
//             DPRINT("USB2_AllocateTimeForEndpoint: frame - %X, uframe - %X, TimeUsed[f][uf] - %X\n",
//                    frame,
//                    uframe,
//                    HcExtension->TimeUsed[frame][uframe]);
//
//             if (!USB2_AllocateCheck(&HcExtension->TimeUsed[frame][uframe],
//                                     TtEndpoint->CalcBusTime,
//                                     USB2_MAX_MICROFRAME_ALLOCATION))
//             {
//                 DPRINT("USB2_AllocateTimeForEndpoint: Result = FALSE\n");
//                 Result = FALSE;
//             }
//         }
//
//         if (!Result)
//         {
//             for (ix = Microframe;
//                  ix < USB2_MAX_MICROFRAMES;
//                  ix += TtEndpoint->ActualPeriod)
//             {
//                 frame = ix / USB2_MICROFRAMES;
//                 uframe = ix % (USB2_MICROFRAMES - 1);
//
//                 HcExtension->TimeUsed[frame][uframe] -= TtEndpoint->CalcBusTime;
//             }
//         }
//
//         DPRINT("USB2_AllocateTimeForEndpoint: Result - TRUE\n");
//         return TRUE;
//     }
//
//     /* Speed != UsbHighSpeed (FS/LS) */
//
//     if (TtEndpoint->Period > USB2_FRAMES)
//         TtEndpoint->ActualPeriod = USB2_FRAMES;
//     else
//         TtEndpoint->ActualPeriod = TtEndpoint->Period;
//
//     MinTimeUsed = Tt->FrameBudget[0].TimeUsed;
//
//     for (ix = 1; ix < TtEndpoint->ActualPeriod; ix++)
//     {
//         if ((Tt->FrameBudget[ix].TimeUsed) < MinTimeUsed)
//         {
//             MinTimeUsed = Tt->FrameBudget[ix].TimeUsed;
//             TtEndpoint->StartFrame = ix;
//         }
//     }
//
//     TransferType = TtEndpoint->TtEndpointParams.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//     {
//         if (Speed == UsbFullSpeed)
//         {
//             Overhead = USB2_FS_ISOCHRONOUS_OVERHEAD + Tt->DelayTime;
//         }
//         else
//         {
//             DPRINT("USB2_AllocateTimeForEndpoint: ISO can not be on a LS bus!\n");
//             return FALSE;
//         }
//     }
//     else
//     {
//         if (Speed == UsbFullSpeed)
//             Overhead = USB2_FS_INTERRUPT_OVERHEAD + Tt->DelayTime;
//         else
//             Overhead = USB2_LS_INTERRUPT_OVERHEAD + Tt->DelayTime;
//     }
//
//     if (Speed == UsbLowSpeed)
//         TtEndpoint->CalcBusTime = TtEndpoint->MaxPacketSize * 8 + Overhead;
//     else
//         TtEndpoint->CalcBusTime = TtEndpoint->MaxPacketSize + Overhead;
//
//     LatestStart = USB2_HUB_DELAY + USB2_FS_SOF_TIME;
//
//     for (ix = 0;
//          (TtEndpoint->StartFrame + ix) < USB2_FRAMES;
//          ix += TtEndpoint->ActualPeriod)
//     {
//         frame = TtEndpoint->StartFrame + ix;
//
//         if (Tt->FrameBudget[frame].AltEndpoint &&
//             TtEndpoint->CalcBusTime >= (USB2_FS_MAX_PERIODIC_ALLOCATION / 2))
//         {
//             DPRINT("USB2_AllocateTimeForEndpoint: return FALSE\n");
//             return FALSE;
//         }
//
//         if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//             prevEndpoint = Tt->FrameBudget[frame].IsoEndpoint;
//         else
//             prevEndpoint = Tt->FrameBudget[frame].IntEndpoint;
//
//         for (nextEndpoint = prevEndpoint->NextTtEndpoint;
//              nextEndpoint;
//              nextEndpoint = nextEndpoint->NextTtEndpoint)
//         {
//             if (USB2_CheckTtEndpointInsert(nextEndpoint, TtEndpoint))
//                 break;
//
//             prevEndpoint = nextEndpoint;
//         }
//
//         StartTime = USB2_GetStartTime(nextEndpoint,
//                                       TtEndpoint,
//                                       prevEndpoint,
//                                       frame);
//
//         LatestStart = max(LatestStart, StartTime);
//     }
//
//     TtEndpoint->StartTime = LatestStart;
//
//     if ((LatestStart + TtEndpoint->CalcBusTime) > USB2_FS_MAX_PERIODIC_ALLOCATION)
//     {
//         TtEndpoint->CalcBusTime = 0;
//         DPRINT("USB2_AllocateTimeForEndpoint: return FALSE\n");
//         return FALSE;
//     }
//
//     for (ix = 0, frame = -TtEndpoint->StartFrame;
//          ix < USB2_FRAMES;
//          ix++, frame++)
//     {
//         DPRINT("USB2_AllocateTimeForEndpoint: ix - %X, frame - %X, StartFrame - %X\n",
//                ix,
//                frame,
//                TtEndpoint->StartFrame);
//
//         if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         {
//             DPRINT1("USB2_AllocateTimeForEndpoint: Iso Ep UNIMPLEMENTED. FIXME\n");
//             ASSERT(FALSE);
//         }
//         else
//         {
//             IntEndpoint = Tt->FrameBudget[ix].IntEndpoint;
//             nextEndpoint = IntEndpoint->NextTtEndpoint;
//
//             for (nextEndpoint = IntEndpoint->NextTtEndpoint;
//                  nextEndpoint;
//                  nextEndpoint = nextEndpoint->NextTtEndpoint)
//             {
//                 if (USB2_CheckTtEndpointInsert(nextEndpoint, TtEndpoint))
//                     break;
//                 IntEndpoint = nextEndpoint;
//             }
//
//             if ((frame % TtEndpoint->ActualPeriod) == 0)
//             {
//                 calcBusTime = 0;
//             }
//             else
//             {
//                 if (nextEndpoint)
//                 {
//                     calcBusTime = LatestStart + TtEndpoint->CalcBusTime -
//                                   nextEndpoint->StartTime;
//                 }
//                 else
//                 {
//                     calcBusTime = TtEndpoint->CalcBusTime;
//                 }
//
//                 if (calcBusTime > 0)
//                 {
//                     TimeUsed = Tt->FrameBudget[ix].TimeUsed;
//
//                     if (!USB2_AllocateCheck(&TimeUsed,
//                                             calcBusTime,
//                                             USB2_FS_MAX_PERIODIC_ALLOCATION))
//                     {
//                         DPRINT("USB2_AllocateTimeForEndpoint: Result = FALSE\n");
//                         Result = FALSE;
//                     }
//                 }
//             }
//
//             if (nextEndpoint != TtEndpoint)
//             {
//                 if ((frame % TtEndpoint->ActualPeriod) == 0)
//                 {
//                     if (frame == 0)
//                     {
//                         DPRINT("USB2_AllocateTimeForEndpoint: frame == 0\n");
//                         TtEndpoint->NextTtEndpoint = nextEndpoint;
//                     }
//
//                     IntEndpoint->NextTtEndpoint = TtEndpoint;
//
//                     DPRINT("USB2_AllocateTimeForEndpoint: TtEndpoint - %p, nextEndpoint - %p\n",
//                            TtEndpoint,
//                            nextEndpoint);
//                 }
//
//                 if (calcBusTime > 0)
//                 {
//                     BOOLEAN IsMoved;
//                     BOOLEAN MoveResult;
//
//                     DPRINT("USB2_AllocateTimeForEndpoint: nextEndpoint - %p, calcBusTime - %X\n",
//                            nextEndpoint,
//                            calcBusTime);
//
//                     for (;
//                          nextEndpoint;
//                          nextEndpoint = nextEndpoint->NextTtEndpoint)
//                     {
//                         MoveResult = USB2_MoveTtEndpoint(nextEndpoint,
//                                                          calcBusTime,
//                                                          Rebalance,
//                                                          *RebalanceListEntries,
//                                                          &IsMoved);
//
//                         if (!IsMoved)
//                         {
//                             DPRINT("USB2_AllocateTimeForEndpoint: Result = FALSE\n");
//                             Result = FALSE;
//                         }
//
//                         if (!MoveResult)
//                             break;
//                     }
//                 }
//             }
//         }
//
//         if ((frame % TtEndpoint->ActualPeriod) == 0)
//         {
//             if (!USB2_AllocateHS(TtEndpoint, frame))
//             {
//                 DPRINT1("USB2_AllocateTimeForEndpoint: USB2_AllocateHS return FALSE\n");
//                 Result = FALSE;
//             }
//
//             Tt->FrameBudget[ix].TimeUsed += TtEndpoint->CalcBusTime;
//         }
//
//         if (Result == FALSE)
//         {
//             USB2_DeallocateEndpointBudget(TtEndpoint,
//                                           Rebalance,
//                                           RebalanceListEntries,
//                                           ix + 1);
//
//             DPRINT("USB2_AllocateTimeForEndpoint: return FALSE\n");
//             return FALSE;
//         }
//     }
//
//     DPRINT("USB2_AllocateTimeForEndpoint: Result - %X\n", Result);
//     return Result;
// }
//
// BOOLEAN
// NTAPI
// USB2_ChangePeriod(IN PUSB2_TT_ENDPOINT TtEndpoint,
//                   IN PUSB2_REBALANCE Rebalance,
//                   IN PULONG RebalanceListEntries)
// {
//     BOOLEAN Result;
//
//     DPRINT("USB2_ChangePeriod: RebalanceListEntries - %X\n",
//            *RebalanceListEntries);
//
//     USB2_DeallocateEndpointBudget(TtEndpoint,
//                                   Rebalance,
//                                   RebalanceListEntries,
//                                   USB2_FRAMES);
//
//     TtEndpoint->PreviosPeriod = TtEndpoint->Period;
//     TtEndpoint->Period = ENDPOINT_INTERRUPT_1ms;
//
//     Result = USB2_AllocateTimeForEndpoint(TtEndpoint,
//                                           Rebalance,
//                                           RebalanceListEntries);
//
//     return Result;
// }
//
// BOOLEAN
// NTAPI
// USB2_PromotePeriods(IN PUSB2_TT_ENDPOINT TtEndpoint,
//                     IN PUSB2_REBALANCE Rebalance,
//                     IN PULONG RebalanceListEntries)
// {
//     PUSB2_TT_ENDPOINT ttEndpoint;
//     ULONG TransferType;
//     ULONG ix;
//
//     TransferType = TtEndpoint->TtEndpointParams.TransferType;
//
//     if (TtEndpoint->ActualPeriod != ENDPOINT_INTERRUPT_1ms &&
//         TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT &&
//         (CHAR)TtEndpoint->StartMicroframe > 2 &&
//         !USB2_ChangePeriod(TtEndpoint, Rebalance, RebalanceListEntries))
//     {
//         DPRINT("USB2_PromotePeriods: return FALSE\n");
//         return FALSE;
//     }
//
//     if (Rebalance->RebalanceEndpoint[0] == NULL)
//     {
//         DPRINT("USB2_PromotePeriods: return TRUE\n");
//         return TRUE;
//     }
//
//     DPRINT("USB2_PromotePeriods: RebalanceListEntries - %X\n",
//            *RebalanceListEntries);
//
//     for (ix = 0; Rebalance->RebalanceEndpoint[ix]; ix++)
//     {
//         Rebalance->RebalanceEndpoint[ix]->IsPromoted = FALSE;
//     }
//
//     for (ix = 0; ; ix++)
//     {
//         ttEndpoint = Rebalance->RebalanceEndpoint[ix];
//         TransferType = ttEndpoint->TtEndpointParams.TransferType;
//
//         if (ttEndpoint->ActualPeriod != ENDPOINT_INTERRUPT_1ms &&
//             TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT &&
//             (CHAR)ttEndpoint->StartMicroframe > 2)
//         {
//             USB2_DeallocateEndpointBudget(ttEndpoint,
//                                           Rebalance,
//                                           RebalanceListEntries,
//                                           USB2_FRAMES);
//
//             ttEndpoint->IsPromoted = TRUE;
//             ttEndpoint->PreviosPeriod = ttEndpoint->Period;
//             ttEndpoint->Period = ENDPOINT_INTERRUPT_1ms;
//
//             if (!USB2_AllocateTimeForEndpoint(ttEndpoint,
//                                               Rebalance,
//                                               RebalanceListEntries))
//             {
//                 break;
//             }
//         }
//
//         if (Rebalance->RebalanceEndpoint[ix + 1] == NULL)
//         {
//             DPRINT("USB2_PromotePeriods: return TRUE\n");
//             return TRUE;
//         }
//     }
//
//     USB2_DeallocateEndpointBudget(TtEndpoint,
//                                   Rebalance,
//                                   RebalanceListEntries,
//                                   USB2_FRAMES);
//
//     TtEndpoint->Period = TtEndpoint->PreviosPeriod;
//     TtEndpoint->PreviosPeriod = 0;
//
//     for (ix = 0; Rebalance->RebalanceEndpoint[ix]; ix++)
//     {
//         ttEndpoint = Rebalance->RebalanceEndpoint[ix];
//
//         if (ttEndpoint->IsPromoted)
//         {
//             if (ttEndpoint->CalcBusTime)
//             {
//                 USB2_DeallocateEndpointBudget(ttEndpoint,
//                                               Rebalance,
//                                               RebalanceListEntries,
//                                               USB2_FRAMES);
//             }
//
//             TtEndpoint->Period = TtEndpoint->PreviosPeriod;
//             TtEndpoint->PreviosPeriod = 0;
//
//             USB2_AllocateTimeForEndpoint(ttEndpoint,
//                                          Rebalance,
//                                          RebalanceListEntries);
//         }
//     }
//
//     DPRINT("USB2_PromotePeriods: return FALSE\n");
//     return FALSE;
// }
//
// VOID
// NTAPI
// USBPORT_UpdateAllocatedBwTt(IN PUSB2_TT_EXTENSION TtExtension)
// {
//     ULONG BusBandwidth;
//     ULONG NewBusBandwidth;
//     ULONG MaxBusBandwidth = 0;
//     ULONG MinBusBandwidth;
//     ULONG ix;
//
//     DPRINT("USBPORT_UpdateAllocatedBwTt: TtExtension - %p\n", TtExtension);
//
//     BusBandwidth = TtExtension->BusBandwidth;
//     MinBusBandwidth = BusBandwidth;
//
//     for (ix = 0; ix < USB2_FRAMES; ix++)
//     {
//         NewBusBandwidth = BusBandwidth - TtExtension->Bandwidth[ix];
//
//         MaxBusBandwidth = max(MaxBusBandwidth, NewBusBandwidth);
//         MinBusBandwidth = min(MinBusBandwidth, NewBusBandwidth);
//     }
//
//     TtExtension->MaxBandwidth = MaxBusBandwidth;
//
//     if (MinBusBandwidth == BusBandwidth)
//         TtExtension->MinBandwidth = 0;
//     else
//         TtExtension->MinBandwidth = MinBusBandwidth;
// }
//
// BOOLEAN
// NTAPI
// USBPORT_AllocateBandwidthUSB2(IN PDEVICE_OBJECT FdoDevice,
//                               IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties;
//     PUSB2_TT_EXTENSION TtExtension;
//     ULONG TransferType;
//     PUSB2_REBALANCE Rebalance;
//     LIST_ENTRY RebalanceList;
//     ULONG RebalanceListEntries;
//     PUSB2_TT_ENDPOINT TtEndpoint;
//     PUSB2_TT_ENDPOINT RebalanceTtEndpoint;
//     PUSB2_TT Tt;
//     USB_DEVICE_SPEED DeviceSpeed;
//     ULONG Period;
//     ULONG AllocedBusTime;
//     ULONG EndpointBandwidth;
//     ULONG ScheduleOffset;
//     ULONG Factor;
//     ULONG ix;
//     ULONG n;
//     BOOLEAN Direction;
//     UCHAR SMask;
//     UCHAR CMask;
//     UCHAR ActualPeriod;
//     BOOLEAN Result;
//
//     DPRINT("USBPORT_AllocateBandwidthUSB2: FdoDevice - %p, Endpoint - %p\n",
//            FdoDevice,
//            Endpoint);
//
//     EndpointProperties = &Endpoint->EndpointProperties;
//     EndpointProperties->ScheduleOffset = 0;
//
//     if (Endpoint->Flags & ENDPOINT_FLAG_ROOTHUB_EP0)
//     {
//         DPRINT("USBPORT_AllocateBandwidthUSB2: ENDPOINT_FLAG_ROOTHUB_EP0\n");
//         return TRUE;
//     }
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     TransferType = EndpointProperties->TransferType;
//     DPRINT("USBPORT_AllocateBandwidthUSB2: TransferType - %X\n", TransferType);
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL ||
//         TransferType == USBPORT_TRANSFER_TYPE_BULK)
//     {
//         return TRUE;
//     }
//
//     if (Endpoint->TtExtension)
//         TtExtension = Endpoint->TtExtension;
//     else
//         TtExtension = NULL;
//
//     InitializeListHead(&RebalanceList);
//
//     Rebalance = ExAllocatePoolWithTag(NonPagedPool,
//                                       sizeof(USB2_REBALANCE),
//                                       USB_PORT_TAG);
//
//     DPRINT("USBPORT_AllocateBandwidthUSB2: Rebalance - %p, TtExtension - %p\n",
//            Rebalance,
//            TtExtension);
//
//     if (Rebalance)
//     {
//         RtlZeroMemory(Rebalance, sizeof(USB2_REBALANCE));
//
//         TtEndpoint = Endpoint->TtEndpoint;
//         TtEndpoint->Endpoint = Endpoint;
//
//         Direction = EndpointProperties->Direction == USBPORT_TRANSFER_DIRECTION_OUT;
//         DeviceSpeed = EndpointProperties->DeviceSpeed;
//
//         switch (DeviceSpeed)
//         {
//             case UsbLowSpeed:
//             case UsbFullSpeed:
//             {
//                 Tt = &TtExtension->Tt;
//                 Period = USB2_FRAMES;
//
//                 while (Period > 0 && Period > EndpointProperties->Period)
//                 {
//                     Period >>= 1;
//                 }
//
//                 DPRINT("USBPORT_AllocateBandwidthUSB2: Period - %X\n", Period);
//                 break;
//             }
//
//             case UsbHighSpeed:
//             {
//                 Tt = &FdoExtension->Usb2Extension->HcTt;
//                 Period = EndpointProperties->Period;
//
//                 break;
//             }
//
//             default:
//             {
//                 DPRINT1("USBPORT_AllocateBandwidthUSB2: DeviceSpeed - %X!\n",
//                         DeviceSpeed);
//
//                 DbgBreakPoint();
//
//                 Tt = &TtExtension->Tt;
//                 break;
//             }
//         }
//
//         USB2_InitTtEndpoint(TtEndpoint,
//                             TransferType,
//                             Direction,
//                             DeviceSpeed,
//                             Period,
//                             EndpointProperties->MaxPacketSize,
//                             Tt);
//
//         RebalanceListEntries = USB2_FRAMES - 2;
//
//         Result = USB2_AllocateTimeForEndpoint(TtEndpoint,
//                                               Rebalance,
//                                               &RebalanceListEntries);
//
//         if (Result)
//         {
//             Result = USB2_PromotePeriods(TtEndpoint,
//                                          Rebalance,
//                                          &RebalanceListEntries);
//         }
//
//         RebalanceListEntries = 0;
//
//         for (ix = 0; Rebalance->RebalanceEndpoint[ix]; ix++)
//         {
//             RebalanceListEntries = ix + 1;
//         }
//     }
//     else
//     {
//         RebalanceListEntries = 0;
//         Result = FALSE;
//     }
//
//     DPRINT("USBPORT_AllocateBandwidthUSB2: RebalanceListEntries - %X, Result - %X\n",
//            RebalanceListEntries,
//            Result);
//
//     for (ix = 0; ix < RebalanceListEntries; ix++)
//     {
//         RebalanceTtEndpoint = Rebalance->RebalanceEndpoint[ix];
//
//         DPRINT("USBPORT_AllocateBandwidthUSB2: RebalanceTtEndpoint[%X] - %p, RebalanceTtEndpoint - %p, RebalanceLink - %p\n",
//                ix,
//                RebalanceTtEndpoint,
//                &RebalanceTtEndpoint->Endpoint->RebalanceLink);
//
//         InsertTailList(&RebalanceList,
//                        &RebalanceTtEndpoint->Endpoint->RebalanceLink);
//     }
//
//     if (Rebalance)
//         ExFreePoolWithTag(Rebalance, USB_PORT_TAG);
//
//     if (Result)
//     {
//         SMask = USB2_GetSMASK(Endpoint->TtEndpoint);
//         EndpointProperties->InterruptScheduleMask = SMask;
//
//         CMask = USB2_GetCMASK(Endpoint->TtEndpoint);
//         EndpointProperties->SplitCompletionMask = CMask;
//
//         AllocedBusTime = TtEndpoint->CalcBusTime;
//
//         EndpointBandwidth = USB2_MICROFRAMES * AllocedBusTime;
//         EndpointProperties->UsbBandwidth = EndpointBandwidth;
//
//         ActualPeriod = Endpoint->TtEndpoint->ActualPeriod;
//         EndpointProperties->Period = ActualPeriod;
//
//         ScheduleOffset = Endpoint->TtEndpoint->StartFrame;
//         EndpointProperties->ScheduleOffset = ScheduleOffset;
//
//         ASSERT(ActualPeriod);
//         Factor = USB2_FRAMES / ActualPeriod;
//         n = ScheduleOffset * Factor;
//
//         if (TtExtension)
//         {
//             for (ix = 0; ix < Factor; ix++)
//             {
//                 TtExtension->Bandwidth[n + ix] -= EndpointBandwidth;
//             }
//         }
//         else
//         {
//             for (ix = 1; ix < Factor; ix++)
//             {
//                 FdoExtension->Bandwidth[n + ix] -= EndpointBandwidth;
//             }
//         }
//
//         USBPORT_DumpingEndpointProperties(EndpointProperties);
//         USBPORT_DumpingTtEndpoint(Endpoint->TtEndpoint);
//
//         if (AllocedBusTime >= (USB2_FS_MAX_PERIODIC_ALLOCATION / 2))
//         {
//             DPRINT1("USBPORT_AllocateBandwidthUSB2: AllocedBusTime >= 0.5 * MAX_ALLOCATION \n");
//         }
//     }
//
//     USB2_Rebalance(FdoDevice, &RebalanceList);
//
//     if (!TtExtension)
//     {
//         DPRINT("USBPORT_AllocateBandwidthUSB2: Result - %X\n", Result);
//         return Result;
//     }
//
//     for (ix = 0; ix < USB2_FRAMES; ix++)
//     {
//         FdoExtension->Bandwidth[ix] += TtExtension->MaxBandwidth;
//     }
//
//     USBPORT_UpdateAllocatedBwTt(TtExtension);
//
//     for (ix = 0; ix < USB2_FRAMES; ix++)
//     {
//         FdoExtension->Bandwidth[ix] -= TtExtension->MaxBandwidth;
//     }
//
//     DPRINT("USBPORT_AllocateBandwidthUSB2: Result - %X\n", Result);
//
//     return Result;
// }
//
// VOID
// NTAPI
// USBPORT_FreeBandwidthUSB2(IN PDEVICE_OBJECT FdoDevice,
//                           IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     ULONG Period;
//     ULONG ScheduleOffset;
//     ULONG EndpointBandwidth;
//     LIST_ENTRY RebalanceList;
//     ULONG TransferType;
//     PUSB2_REBALANCE Rebalance;
//     ULONG RebalanceListEntries;
//     ULONG Factor;
//     ULONG ix;
//     ULONG n;
//     PUSB2_TT_EXTENSION TtExtension;
//     PUSB2_TT_ENDPOINT RebalanceTtEndpoint;
//
//     DPRINT("USBPORT_FreeBandwidthUSB2: Endpoint - %p\n", Endpoint);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     Period = Endpoint->EndpointProperties.Period;
//     ScheduleOffset = Endpoint->EndpointProperties.ScheduleOffset;
//     EndpointBandwidth = Endpoint->EndpointProperties.UsbBandwidth;
//
//     InitializeListHead(&RebalanceList);
//
//     TransferType = Endpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL ||
//         TransferType == USBPORT_TRANSFER_TYPE_BULK ||
//         (Endpoint->Flags & ENDPOINT_FLAG_ROOTHUB_EP0))
//     {
//         return;
//     }
//
//     Rebalance = ExAllocatePoolWithTag(NonPagedPool,
//                                       sizeof(USB2_REBALANCE),
//                                       USB_PORT_TAG);
//
//     if (!Rebalance)
//     {
//         DPRINT1("USBPORT_FreeBandwidthUSB2: Rebalance == NULL!\n");
//         return;
//     }
//
//     RtlZeroMemory(Rebalance, sizeof(USB2_REBALANCE));
//
//     ASSERT(Period != 0);
//     Factor = USB2_FRAMES / Period;
//     n = ScheduleOffset * Factor;
//
//     TtExtension = Endpoint->TtExtension;
//
//     if (TtExtension)
//     {
//         for (ix = 0; ix < Factor; ix++)
//         {
//             TtExtension->Bandwidth[n + ix] += EndpointBandwidth;
//         }
//     }
//     else
//     {
//         for (ix = 1; ix < Factor; ix++)
//         {
//             FdoExtension->Bandwidth[n + ix] += EndpointBandwidth;
//         }
//     }
//
//     RebalanceListEntries = USB2_FRAMES - 2;
//
//     USB2_DeallocateEndpointBudget(Endpoint->TtEndpoint,
//                                   Rebalance,
//                                   &RebalanceListEntries,
//                                   USB2_FRAMES);
//
//     RebalanceListEntries = 0;
//
//     for (ix = 0; Rebalance->RebalanceEndpoint[ix]; ix++)
//     {
//         RebalanceListEntries = ix + 1;
//     }
//
//     for (ix = 0; ix < RebalanceListEntries; ix++)
//     {
//         RebalanceTtEndpoint = Rebalance->RebalanceEndpoint[ix];
//
//         DPRINT("USBPORT_AllocateBandwidthUSB2: RebalanceTtEndpoint[%X] - %p, RebalanceTtEndpoint - %p, RebalanceLink - %p\n",
//                ix,
//                RebalanceTtEndpoint,
//                &RebalanceTtEndpoint->Endpoint->RebalanceLink);
//
//         InsertTailList(&RebalanceList,
//                        &RebalanceTtEndpoint->Endpoint->RebalanceLink);
//     }
//
//     ExFreePoolWithTag(Rebalance, USB_PORT_TAG);
//
//     USB2_Rebalance(FdoDevice, &RebalanceList);
//
//     if (!TtExtension)
//         return;
//
//     for (ix = 0; ix < USB2_FRAMES; ix++)
//     {
//         FdoExtension->Bandwidth[ix] += TtExtension->MaxBandwidth;
//     }
//
//     USBPORT_UpdateAllocatedBwTt(TtExtension);
//
//     for (ix = 0; ix < USB2_FRAMES; ix++)
//     {
//         FdoExtension->Bandwidth[ix] -= TtExtension->MaxBandwidth;
//     }
//
//     DPRINT1("USBPORT_FreeBandwidthUSB2: exit\n");
// }
//
// VOID
// NTAPI
// USB2_InitTT(IN PUSB2_HC_EXTENSION HcExtension,
//             IN PUSB2_TT Tt)
// {
//     ULONG ix;
//     ULONG jx;
//
//     DPRINT("USB2_InitTT: HcExtension - %p, Tt - %p\n", HcExtension, Tt);
//
//     Tt->HcExtension = HcExtension;
//     Tt->DelayTime = 1;
//     Tt->MaxTime = USB2_FS_MAX_PERIODIC_ALLOCATION;
//
//     for (ix = 0; ix < USB2_FRAMES; ix++)
//     {
//         Tt->FrameBudget[ix].TimeUsed = USB2_MAX_MICROFRAMES;
//         Tt->FrameBudget[ix].AltEndpoint = NULL;
//
//         for (jx = 0; jx < USB2_MICROFRAMES; jx++)
//         {
//             Tt->TimeCS[ix][jx] = 0;
//             Tt->NumStartSplits[ix][jx] = 0;
//         }
//
//         Tt->FrameBudget[ix].IsoEndpoint = &Tt->IsoEndpoint[ix];
//
//         USB2_InitTtEndpoint(&Tt->IsoEndpoint[ix],
//                             USBPORT_TRANSFER_TYPE_ISOCHRONOUS,
//                             USBPORT_TRANSFER_DIRECTION_OUT,
//                             UsbFullSpeed,
//                             USB2_FRAMES,
//                             0,
//                             Tt);
//
//         Tt->IsoEndpoint[ix].ActualPeriod = USB2_FRAMES;
//         Tt->IsoEndpoint[ix].CalcBusTime = USB2_FS_SOF_TIME + USB2_HUB_DELAY;
//         Tt->IsoEndpoint[ix].StartFrame = ix;
//         Tt->IsoEndpoint[ix].StartMicroframe = USB2_PREV_MICROFRAME;
//
//         Tt->FrameBudget[ix].IntEndpoint = &Tt->IntEndpoint[ix];
//
//         USB2_InitTtEndpoint(&Tt->IntEndpoint[ix],
//                             USBPORT_TRANSFER_TYPE_INTERRUPT,
//                             USBPORT_TRANSFER_DIRECTION_OUT,
//                             UsbFullSpeed,
//                             USB2_FRAMES,
//                             0,
//                             Tt);
//
//         Tt->IntEndpoint[ix].ActualPeriod = USB2_FRAMES;
//         Tt->IntEndpoint[ix].CalcBusTime = USB2_FS_SOF_TIME + USB2_HUB_DELAY;
//         Tt->IntEndpoint[ix].StartFrame = ix;
//         Tt->IntEndpoint[ix].StartMicroframe = USB2_PREV_MICROFRAME;
//     }
// }
//
// VOID
// NTAPI
// USB2_InitController(IN PUSB2_HC_EXTENSION HcExtension)
// {
//     ULONG ix;
//     ULONG jx;
//
//     DPRINT("USB2_InitController: HcExtension - %p\n", HcExtension);
//
//     HcExtension->MaxHsBusAllocation = USB2_MAX_MICROFRAME_ALLOCATION;
//
//     for (ix = 0; ix < USB2_FRAMES; ix++)
//     {
//         for (jx = 0; jx < USB2_MICROFRAMES; jx++)
//         {
//             HcExtension->TimeUsed[ix][jx] = 0;
//         }
//     }
//
//     HcExtension->HcDelayTime = USB2_CONTROLLER_DELAY;
//
//     USB2_InitTT(HcExtension, &HcExtension->HcTt);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
