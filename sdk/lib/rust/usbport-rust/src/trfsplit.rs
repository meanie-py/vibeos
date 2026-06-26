use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExFreePool, LIST_ENTRY, PDEVICE_OBJECT};
use core::ffi::c_void;

extern "C" {
    fn USBPORT_DoneTransfer(Transfer: PUSBPORT_TRANSFER);
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG (skipped: no value, used for conditional compilation)

#[no_mangle]
pub unsafe extern "system" fn USBPORT_MakeSplitTransfer(
    FdoDevice: PDEVICE_OBJECT,
    Transfer: PUSBPORT_TRANSFER,
    SplitTransfer: PUSBPORT_TRANSFER,
    MaxTransferSize: ULONG,
    SgIdx: PULONG,
    SgOffset: PULONG,
    mut TransferRemainLen: ULONG,
    TransferOffset: ULONG,
) -> ULONG {
    let SplitSgList: PUSBPORT_SCATTER_GATHER_LIST;
    let Element0: PUSBPORT_SCATTER_GATHER_ELEMENT;
    let Element1: PUSBPORT_SCATTER_GATHER_ELEMENT;
    let SgLength: SIZE_T;
    let mut SgRemainLen: SIZE_T;

    SplitSgList = core::ptr::addr_of_mut!((*SplitTransfer).SgList) as PUSBPORT_SCATTER_GATHER_LIST;
    Element0 = (core::ptr::addr_of_mut!((*SplitSgList).SgElement) as *mut USBPORT_SCATTER_GATHER_ELEMENT)
        .offset(0);

    SgLength = ((*Transfer).SgList.SgElement[*SgIdx as usize].SgTransferLength as u32)
        .wrapping_sub(*SgOffset) as SIZE_T;

    if SgLength > MaxTransferSize as SIZE_T {
        (*SplitTransfer).SgList.SgElementCount = 1 as ULONG;

        (*Element0).SgOffset = 0 as ULONG;
        (*Element0).SgTransferLength = MaxTransferSize;
        (*Element0).SgPhysicalAddress.c2rust_unnamed.__bindgen_anon_1.LowPart =
            ((*Transfer).SgList.SgElement[*SgIdx as usize].SgPhysicalAddress.c2rust_unnamed.__bindgen_anon_1.LowPart as u32)
                .wrapping_add(*SgOffset) as ULONG;

        (*SplitTransfer).TransferParameters.IsTransferSplited = 1 as BOOL;
        (*SplitTransfer).TransferParameters.TransferBufferLength = MaxTransferSize;

        (*SplitTransfer).SgList.CurrentVa =
            (((*Transfer).SgList.CurrentVa as u64).wrapping_add(TransferOffset as u64) as ULONG_PTR as u32);
        (*SplitTransfer).SgList.MappedSystemVa =
            ((*Transfer).SgList.MappedSystemVa as u64).wrapping_add(TransferOffset as u64) as PVOID;

        (*SplitTransfer).Flags |= TRANSFER_FLAG_SPLITED as u32;

        *SgOffset = (*SgOffset).wrapping_add(MaxTransferSize);
        TransferRemainLen = TransferRemainLen.wrapping_sub(MaxTransferSize);
        return TransferRemainLen;
    }

    (*SplitTransfer).SgList.SgElementCount = 1 as ULONG;
    TransferRemainLen = (TransferRemainLen as u64).wrapping_sub(SgLength as u64) as ULONG;

    (*Element0).SgOffset = 0 as ULONG;
    (*Element0).SgTransferLength = SgLength as ULONG;
    (*Element0).SgPhysicalAddress.c2rust_unnamed.__bindgen_anon_1.LowPart =
        ((*Transfer).SgList.SgElement[*SgIdx as usize].SgPhysicalAddress.c2rust_unnamed.__bindgen_anon_1.LowPart as u32)
            .wrapping_add(*SgOffset) as ULONG;

    (*SplitTransfer).TransferParameters.TransferBufferLength = SgLength as ULONG;
    (*SplitTransfer).TransferParameters.IsTransferSplited = 1 as BOOL;

    (*SplitTransfer).SgList.CurrentVa =
        (((*Transfer).SgList.CurrentVa as u64).wrapping_add(TransferOffset as u64) as ULONG_PTR as u32);
    (*SplitTransfer).SgList.MappedSystemVa =
        ((*Transfer).SgList.MappedSystemVa as u64).wrapping_add(TransferOffset as u64) as PVOID;

    (*SplitTransfer).Flags |= TRANSFER_FLAG_SPLITED as u32;

    *SgOffset = (*SgOffset as u64).wrapping_add(SgLength as u64) as u32;

    SgRemainLen = (MaxTransferSize as SIZE_T).wrapping_sub(SgLength);

    if SgRemainLen > TransferRemainLen as SIZE_T {
        SgRemainLen = TransferRemainLen as SIZE_T;
    }

    if SgRemainLen == 0 {
        *SgIdx = (*SgIdx).wrapping_add(1);
        *SgOffset = 0 as u32;
        return TransferRemainLen;
    }

    *SgIdx = (*SgIdx).wrapping_add(1);

    *SgOffset = 0 as u32;
    (*SplitTransfer).SgList.SgElementCount = (*SplitTransfer).SgList.SgElementCount.wrapping_add(1);

    Element1 = (core::ptr::addr_of_mut!((*SplitSgList).SgElement) as *mut USBPORT_SCATTER_GATHER_ELEMENT)
        .offset(1);

    (*Element1).SgOffset = SgRemainLen as ULONG;
    (*Element1).SgTransferLength = (*Element0).SgTransferLength;
    (*Element1).SgPhysicalAddress.c2rust_unnamed.__bindgen_anon_1.LowPart =
        ((*Transfer).SgList.SgElement[*SgIdx as usize].SgPhysicalAddress.c2rust_unnamed.__bindgen_anon_1.LowPart as u32)
            .wrapping_add(*SgOffset) as ULONG;

    (*SplitTransfer).TransferParameters.TransferBufferLength =
        ((*SplitTransfer).TransferParameters.TransferBufferLength as u64).wrapping_add(SgRemainLen as u64)
            as ULONG;

    *SgOffset = (*SgOffset as u64).wrapping_add(SgRemainLen as u64) as u32;
    TransferRemainLen = (TransferRemainLen as u64).wrapping_sub(SgRemainLen as u64) as ULONG;

    return TransferRemainLen;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SplitBulkInterruptTransfer(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
    Transfer: PUSBPORT_TRANSFER,
    List: PLIST_ENTRY,
) {
    let mut current_block: u64;
    let mut SplitTransfer: PUSBPORT_TRANSFER = core::ptr::null_mut();
    let mut tmplist: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut NeedSplits: ULONG;
    let TransferBufferLength: SIZE_T;
    let MaxTransferSize: SIZE_T;
    let mut TransferOffset: SIZE_T = 0 as SIZE_T;
    let mut RemainLength: SIZE_T;
    let mut ix: ULONG;
    let mut SgIdx: ULONG = 0 as ULONG;
    let mut SgOffset: ULONG = 0 as ULONG;

    MaxTransferSize = ((*Endpoint).EndpointProperties.TotalMaxPacketSize as u32)
        .wrapping_mul(
            ((*Endpoint).EndpointProperties.MaxTransferSize as u32).wrapping_div(
                (*Endpoint).EndpointProperties.TotalMaxPacketSize as u32,
            ),
        ) as SIZE_T;

    if (*Endpoint).EndpointProperties.MaxTransferSize > PAGE_SIZE as u32 {
        KeBugCheckEx(
            BUGCODE_USB_DRIVER,
            1 as ULONG_PTR,
            0 as ULONG_PTR,
            0 as ULONG_PTR,
            0 as ULONG_PTR,
        );
    }

    TransferBufferLength = (*Transfer).TransferParameters.TransferBufferLength as SIZE_T;
    (*Transfer).Flags |= TRANSFER_FLAG_PARENT as u32;

    NeedSplits = (TransferBufferLength as u64)
        .wrapping_div(MaxTransferSize as u64)
        .wrapping_add(1 as u64) as ULONG;

    InitializeListHead(core::ptr::addr_of_mut!(tmplist));

    if NeedSplits == 0 {
        DbgBreakPoint();
    } else {
        ix = 0 as ULONG;
        loop {
            if !(ix < NeedSplits) {
                current_block = 1118134448028020070;
                break;
            }
            SplitTransfer = ExAllocatePoolWithTag(
                NonPagedPool,
                (*Transfer).FullTransferLength as usize,
                USB_PORT_TAG as ULONG,
            ) as PUSBPORT_TRANSFER;
            if SplitTransfer.is_null() {
                DbgBreakPoint();
                current_block = 1710853546084824164;
                break;
            } else {
                RtlCopyMemory(
                    SplitTransfer as *mut core::ffi::c_void,
                    Transfer as *const core::ffi::c_void,
                    (*Transfer).FullTransferLength as usize,
                );
                (*SplitTransfer).MiniportTransfer = (SplitTransfer as SIZE_T)
                    .wrapping_add(((*SplitTransfer).PortTransferLength as usize))
                    as PVOID;
                InsertTailList(
                    core::ptr::addr_of_mut!(tmplist),
                    core::ptr::addr_of_mut!((*SplitTransfer).TransferLink),
                );
                ix = ix.wrapping_add(1);
            }
        }
        match current_block {
            1710853546084824164 => {}
            _ => {
                if !((*Transfer).TransferParameters.TransferBufferLength == 0 as u32) {
                    RemainLength = (*Transfer).TransferParameters.TransferBufferLength as SIZE_T;
                    loop {
                        SplitTransfer = (tmplist.Flink as ULONG_PTR).wrapping_sub(
                            core::mem::offset_of!(USBPORT_TRANSFER, TransferLink) as ULONG_PTR,
                        ) as *mut USBPORT_TRANSFER as PUSBPORT_TRANSFER;
                        RemoveHeadList(core::ptr::addr_of_mut!(tmplist));
                        RemainLength = USBPORT_MakeSplitTransfer(
                            FdoDevice,
                            Transfer,
                            SplitTransfer,
                            MaxTransferSize as ULONG,
                            core::ptr::addr_of_mut!(SgIdx),
                            core::ptr::addr_of_mut!(SgOffset),
                            RemainLength as ULONG,
                            TransferOffset as ULONG,
                        ) as SIZE_T;
                        TransferOffset = (TransferOffset as u64).wrapping_add(
                            (*SplitTransfer).TransferParameters.TransferBufferLength as u64,
                        ) as SIZE_T;
                        InsertTailList(
                            List,
                            core::ptr::addr_of_mut!((*SplitTransfer).TransferLink),
                        );
                        InsertTailList(
                            core::ptr::addr_of_mut!((*Transfer).SplitTransfersList),
                            core::ptr::addr_of_mut!((*SplitTransfer).SplitLink),
                        );
                        if !(RemainLength != (0 as u64 as usize)) {
                            break;
                        }
                    }
                }
            }
        }
    }

    while IsListEmpty(core::ptr::addr_of_mut!(tmplist)) == 0 {
        SplitTransfer = (tmplist.Flink as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(USBPORT_TRANSFER, TransferLink) as ULONG_PTR,
        ) as *mut USBPORT_TRANSFER as PUSBPORT_TRANSFER;
        RemoveHeadList(core::ptr::addr_of_mut!(tmplist));
        ExFreePoolWithTag(SplitTransfer as PVOID, USB_PORT_TAG as ULONG);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SplitTransfer(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
    Transfer: PUSBPORT_TRANSFER,
    List: PLIST_ENTRY,
) {
    const USBD_STATUS_SUCCESS: u32 = 0;
    const USBPORT_TRANSFER_TYPE_BULK: u32 = 3;
    const USBPORT_TRANSFER_TYPE_INTERRUPT: u32 = 1;
    const USBPORT_TRANSFER_TYPE_ISOCHRONOUS: u32 = 0;
    const USBPORT_TRANSFER_TYPE_CONTROL: u32 = 2;
    const BUGCODE_USB_DRIVER: u32 = 0xFE;

    let mut TransferType: ULONG;

    InitializeListHead(List);
    InitializeListHead(core::ptr::addr_of_mut!((*Transfer).SplitTransfersList));

    (*Transfer).USBDStatus = (USBD_STATUS_SUCCESS as i32);

    if (*Transfer).TransferParameters.TransferBufferLength
        > (*Endpoint).EndpointProperties.MaxTransferSize
    {
        TransferType = (*Endpoint).EndpointProperties.TransferType;

        if TransferType == USBPORT_TRANSFER_TYPE_BULK
            || TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT
        {
            USBPORT_SplitBulkInterruptTransfer(FdoDevice, Endpoint, Transfer, List);
        } else if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS
            || TransferType == USBPORT_TRANSFER_TYPE_CONTROL
        {
            KeBugCheckEx(
                BUGCODE_USB_DRIVER,
                1 as ULONG_PTR,
                0 as ULONG_PTR,
                0 as ULONG_PTR,
                0 as ULONG_PTR,
            );
        } else if DbgPrint(
            b"(%s:%d) USBPORT_SplitTransfer: Unknown TransferType - %x\n\0".as_ptr() as PCSTR,
            b"trfsplit.c\0".as_ptr() as *const core::ffi::c_char,
            267 as core::ffi::c_int,
            TransferType,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"trfsplit.c\0".as_ptr() as *const core::ffi::c_char,
                267 as core::ffi::c_int,
            );
        }
    } else {
        InsertTailList(List, core::ptr::addr_of_mut!((*Transfer).TransferLink));
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DoneSplitTransfer(SplitTransfer: PUSBPORT_TRANSFER) {

    let mut ParentTransfer: PUSBPORT_TRANSFER;
    let mut OldIrql: KIRQL = 0;

    ParentTransfer = (*SplitTransfer).ParentTransfer as PUSBPORT_TRANSFER;
    (*ParentTransfer).CompletedTransferLen = ((*ParentTransfer).CompletedTransferLen as ULONG)
        .wrapping_add((*SplitTransfer).CompletedTransferLen as ULONG) as ULONG;

    if (*SplitTransfer).USBDStatus != USBD_STATUS_SUCCESS {
        (*ParentTransfer).USBDStatus = (*SplitTransfer).USBDStatus;
    }

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*ParentTransfer).TransferSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    RemoveEntryList(core::ptr::addr_of_mut!((*SplitTransfer).SplitLink));
    ExFreePoolWithTag(SplitTransfer as PVOID, USB_PORT_TAG as ULONG);

    if IsListEmpty(core::ptr::addr_of_mut!((*ParentTransfer).SplitTransfersList)) != 0 {
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*ParentTransfer).TransferSpinLock) as *mut usize),
            OldIrql,
        );
        USBPORT_DoneTransfer(ParentTransfer);
    } else {
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*ParentTransfer).TransferSpinLock) as *mut usize),
            OldIrql,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CancelSplitTransfer(mut SplitTransfer: PUSBPORT_TRANSFER) {
    let mut ParentTransfer: PUSBPORT_TRANSFER = core::ptr::null_mut();
    let mut Endpoint: PUSBPORT_ENDPOINT = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;

    Endpoint = (*SplitTransfer).Endpoint;
    ParentTransfer = (*SplitTransfer).ParentTransfer as PUSBPORT_TRANSFER;
    (*ParentTransfer).CompletedTransferLen = ((*ParentTransfer).CompletedTransferLen as ULONG)
        .wrapping_add((*SplitTransfer).CompletedTransferLen as ULONG) as ULONG;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*ParentTransfer).TransferSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );
    RemoveEntryList(core::ptr::addr_of_mut!((*SplitTransfer).SplitLink));
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*ParentTransfer).TransferSpinLock) as *mut usize),
        OldIrql,
    );

    ExFreePool(SplitTransfer as PVOID);

    if IsListEmpty(core::ptr::addr_of_mut!((*ParentTransfer).SplitTransfersList)) != 0 {
        InsertTailList(
            core::ptr::addr_of_mut!((*Endpoint).CancelList),
            core::ptr::addr_of_mut!((*ParentTransfer).TransferLink),
        );
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort split transfer functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// ULONG
// NTAPI
// USBPORT_MakeSplitTransfer(IN PDEVICE_OBJECT FdoDevice,
//                           IN PUSBPORT_TRANSFER Transfer,
//                           IN PUSBPORT_TRANSFER SplitTransfer,
//                           IN ULONG MaxTransferSize,
//                           IN PULONG SgIdx,
//                           IN PULONG SgOffset,
//                           IN ULONG TransferRemainLen,
//                           IN ULONG TransferOffset)
// {
//     PUSBPORT_SCATTER_GATHER_LIST SplitSgList;
//     PUSBPORT_SCATTER_GATHER_ELEMENT Element0;
//     PUSBPORT_SCATTER_GATHER_ELEMENT Element1;
//     SIZE_T SgLength;
//     SIZE_T SgRemainLen;
//
//     DPRINT("USBPORT_MakeSplitTransfer: ... \n");
//
//     SplitSgList = &SplitTransfer->SgList;
//     Element0 = &SplitSgList->SgElement[0];
//
//     SgLength = Transfer->SgList.SgElement[*SgIdx].SgTransferLength - *SgOffset;
//
//     if (SgLength > MaxTransferSize)
//     {
//         /* SgLength > MaxTransferSize */
//         SplitTransfer->SgList.SgElementCount = 1;
//
//         Element0->SgOffset = 0;
//         Element0->SgTransferLength = MaxTransferSize;
//         Element0->SgPhysicalAddress.LowPart = Transfer->SgList.SgElement[*SgIdx].SgPhysicalAddress.LowPart + *SgOffset;
//
//         SplitTransfer->TransferParameters.IsTransferSplited = TRUE;
//         SplitTransfer->TransferParameters.TransferBufferLength = MaxTransferSize;
//
//         SplitTransfer->SgList.CurrentVa = Transfer->SgList.CurrentVa + TransferOffset;
//         SplitTransfer->SgList.MappedSystemVa = (PVOID)((ULONG_PTR)Transfer->SgList.MappedSystemVa + TransferOffset);
//
//         SplitTransfer->Flags |= TRANSFER_FLAG_SPLITED;
//
//         *SgOffset += MaxTransferSize;
//         TransferRemainLen -= MaxTransferSize;
//         return TransferRemainLen;
//     }
//
//     /* SgLength <= MaxTransferSize */
//     SplitTransfer->SgList.SgElementCount = 1;
//     TransferRemainLen -= SgLength;
//
//     Element0->SgOffset = 0;
//     Element0->SgTransferLength = SgLength;
//     Element0->SgPhysicalAddress.LowPart = Transfer->SgList.SgElement[*SgIdx].SgPhysicalAddress.LowPart + *SgOffset;
//
//     SplitTransfer->TransferParameters.TransferBufferLength = SgLength;
//     SplitTransfer->TransferParameters.IsTransferSplited = TRUE;
//
//     SplitTransfer->SgList.CurrentVa = Transfer->SgList.CurrentVa + TransferOffset;
//     SplitTransfer->SgList.MappedSystemVa = (PVOID)((ULONG_PTR)Transfer->SgList.MappedSystemVa + TransferOffset);
//
//     SplitTransfer->Flags |= TRANSFER_FLAG_SPLITED;
//
//     *SgOffset += SgLength;
//
//     SgRemainLen = MaxTransferSize - SgLength;
//
//     if (SgRemainLen > TransferRemainLen)
//     {
//         SgRemainLen = TransferRemainLen;
//     }
//
//     if (!SgRemainLen)
//     {
//         /* SgLength == MaxTransferSize */
//         ++*SgIdx;
//         *SgOffset = 0;
//         return TransferRemainLen;
//     }
//
//     /* SgLength < MaxTransferSize */
//
//     DPRINT1("MakeSplitTransfer: SgRemainLen - %x\n", SgRemainLen);
//     DPRINT1("MakeSplitTransfer: SgIdx - %x\n", *SgIdx);
//     ++*SgIdx;
//
//     *SgOffset = 0;
//     SplitTransfer->SgList.SgElementCount++;
//
//     Element1 = &SplitSgList->SgElement[1];
//
//     Element1->SgOffset = SgRemainLen;
//     Element1->SgTransferLength = Element0->SgTransferLength;
//     Element1->SgPhysicalAddress.LowPart = Transfer->SgList.SgElement[*SgIdx].SgPhysicalAddress.LowPart + *SgOffset;
//
//     SplitTransfer->TransferParameters.TransferBufferLength += SgRemainLen;
//
//     *SgOffset += SgRemainLen;
//     TransferRemainLen -= SgRemainLen;
//
//     return TransferRemainLen;
// }
//
// VOID
// NTAPI
// USBPORT_SplitBulkInterruptTransfer(IN PDEVICE_OBJECT FdoDevice,
//                                    IN PUSBPORT_ENDPOINT Endpoint,
//                                    IN PUSBPORT_TRANSFER Transfer,
//                                    IN PLIST_ENTRY List)
// {
//     PUSBPORT_TRANSFER SplitTransfer;
//     LIST_ENTRY tmplist;
//     ULONG NeedSplits;
//     SIZE_T TransferBufferLength;
//     SIZE_T MaxTransferSize;
//     SIZE_T TransferOffset = 0;
//     SIZE_T RemainLength;
//     ULONG ix;
//     ULONG SgIdx = 0;
//     ULONG SgOffset = 0;
//
//     DPRINT("USBPORT_SplitBulkInterruptTransfer: ... \n");
//
//     MaxTransferSize = Endpoint->EndpointProperties.TotalMaxPacketSize *
//                       (Endpoint->EndpointProperties.MaxTransferSize /
//                        Endpoint->EndpointProperties.TotalMaxPacketSize);
//
//     if (Endpoint->EndpointProperties.MaxTransferSize > PAGE_SIZE)
//     {
//         KeBugCheckEx(BUGCODE_USB_DRIVER, 1, 0, 0, 0);
//     }
//
//     TransferBufferLength = Transfer->TransferParameters.TransferBufferLength;
//     Transfer->Flags |= TRANSFER_FLAG_PARENT;
//
//     NeedSplits = TransferBufferLength / MaxTransferSize + 1;
//
//     InitializeListHead(&tmplist);
//
//     DPRINT("USBPORT_SplitBulkInterruptTransfer: TransferBufferLength - %x, NeedSplits - %x\n",
//            TransferBufferLength, NeedSplits);
//
//     if (!NeedSplits)
//     {
//         DPRINT1("USBPORT_SplitBulkInterruptTransfer: DbgBreakPoint \n");
//         DbgBreakPoint();
//         goto Exit;
//     }
//
//     for (ix = 0; ix < NeedSplits; ++ix)
//     {
//         SplitTransfer = ExAllocatePoolWithTag(NonPagedPool,
//                                               Transfer->FullTransferLength,
//                                               USB_PORT_TAG);
//
//         if (!SplitTransfer)
//         {
//             DPRINT1("USBPORT_SplitBulkInterruptTransfer: DbgBreakPoint \n");
//             DbgBreakPoint();
//             goto Exit;
//         }
//
//         RtlCopyMemory(SplitTransfer, Transfer, Transfer->FullTransferLength);
//
//         SplitTransfer->MiniportTransfer = (PVOID)((ULONG_PTR)SplitTransfer +
//                                           SplitTransfer->PortTransferLength);
//
//         InsertTailList(&tmplist, &SplitTransfer->TransferLink);
//     }
//
//     if (Transfer->TransferParameters.TransferBufferLength == 0)
//     {
//         goto Exit;
//     }
//
//     RemainLength = Transfer->TransferParameters.TransferBufferLength;
//
//     do
//     {
//         SplitTransfer = CONTAINING_RECORD(tmplist.Flink,
//                                           USBPORT_TRANSFER,
//                                           TransferLink);
//
//         RemoveHeadList(&tmplist);
//
//         RemainLength = USBPORT_MakeSplitTransfer(FdoDevice,
//                                                  Transfer,
//                                                  SplitTransfer,
//                                                  MaxTransferSize,
//                                                  &SgIdx,
//                                                  &SgOffset,
//                                                  RemainLength,
//                                                  TransferOffset);
//
//         TransferOffset += SplitTransfer->TransferParameters.TransferBufferLength;
//
//         InsertTailList(List, &SplitTransfer->TransferLink);
//         InsertTailList(&Transfer->SplitTransfersList,&SplitTransfer->SplitLink);
//     }
//     while (RemainLength != 0);
//
// Exit:
//
//     while (!IsListEmpty(&tmplist))
//     {
//         DPRINT("USBPORT_SplitBulkInterruptTransfer: ... \n");
//
//         SplitTransfer = CONTAINING_RECORD(tmplist.Flink,
//                                           USBPORT_TRANSFER,
//                                           TransferLink);
//         RemoveHeadList(&tmplist);
//
//         ExFreePoolWithTag(SplitTransfer, USB_PORT_TAG);
//     }
//
//     return;
// }
//
// VOID
// NTAPI
// USBPORT_SplitTransfer(IN PDEVICE_OBJECT FdoDevice,
//                       IN PUSBPORT_ENDPOINT Endpoint,
//                       IN PUSBPORT_TRANSFER Transfer,
//                       IN PLIST_ENTRY List)
// {
//     ULONG TransferType;
//
//     DPRINT("USBPORT_SplitTransfer ... \n");
//
//     InitializeListHead(List);
//     InitializeListHead(&Transfer->SplitTransfersList);
//
//     Transfer->USBDStatus = USBD_STATUS_SUCCESS;
//
//     if (Transfer->TransferParameters.TransferBufferLength >
//         Endpoint->EndpointProperties.MaxTransferSize)
//     {
//         TransferType = Endpoint->EndpointProperties.TransferType;
//
//         if (TransferType == USBPORT_TRANSFER_TYPE_BULK ||
//             TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//         {
//             USBPORT_SplitBulkInterruptTransfer(FdoDevice,
//                                                Endpoint,
//                                                Transfer,
//                                                List);
//         }
//         else if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS ||
//                  TransferType == USBPORT_TRANSFER_TYPE_CONTROL)
//         {
//             KeBugCheckEx(BUGCODE_USB_DRIVER, 1, 0, 0, 0);
//         }
//         else
//         {
//             DPRINT1("USBPORT_SplitTransfer: Unknown TransferType - %x\n",
//                     TransferType);
//         }
//     }
//     else
//     {
//         InsertTailList(List, &Transfer->TransferLink);
//     }
// }
//
// VOID
// NTAPI
// USBPORT_DoneSplitTransfer(IN PUSBPORT_TRANSFER SplitTransfer)
// {
//     PUSBPORT_TRANSFER ParentTransfer;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_DoneSplitTransfer: ... \n");
//
//     ParentTransfer = SplitTransfer->ParentTransfer;
//     ParentTransfer->CompletedTransferLen += SplitTransfer->CompletedTransferLen;
//
//     if (SplitTransfer->USBDStatus != USBD_STATUS_SUCCESS)
//     {
//         DPRINT1("USBPORT_DoneSplitTransfer: SplitTransfer->USBDStatus - %X\n",
//                 SplitTransfer->USBDStatus);
//
//         ParentTransfer->USBDStatus = SplitTransfer->USBDStatus;
//     }
//
//     KeAcquireSpinLock(&ParentTransfer->TransferSpinLock, &OldIrql);
//
//     RemoveEntryList(&SplitTransfer->SplitLink);
//     ExFreePoolWithTag(SplitTransfer, USB_PORT_TAG);
//
//     if (IsListEmpty(&ParentTransfer->SplitTransfersList))
//     {
//         KeReleaseSpinLock(&ParentTransfer->TransferSpinLock, OldIrql);
//         USBPORT_DoneTransfer(ParentTransfer);
//     }
//     else
//     {
//         KeReleaseSpinLock(&ParentTransfer->TransferSpinLock, OldIrql);
//     }
// }
//
// VOID
// NTAPI
// USBPORT_CancelSplitTransfer(IN PUSBPORT_TRANSFER SplitTransfer)
// {
//     PUSBPORT_TRANSFER ParentTransfer;
//     PUSBPORT_ENDPOINT Endpoint;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_CancelSplitTransfer \n");
//
//     Endpoint = SplitTransfer->Endpoint;
//     ParentTransfer = SplitTransfer->ParentTransfer;
//     ParentTransfer->CompletedTransferLen += SplitTransfer->CompletedTransferLen;
//
//     KeAcquireSpinLock(&ParentTransfer->TransferSpinLock, &OldIrql);
//     RemoveEntryList(&SplitTransfer->SplitLink);
//     KeReleaseSpinLock(&ParentTransfer->TransferSpinLock, OldIrql);
//
//     ExFreePool(SplitTransfer);
//
//     if (IsListEmpty(&ParentTransfer->SplitTransfersList))
//     {
//         InsertTailList(&Endpoint->CancelList, &ParentTransfer->TransferLink);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
