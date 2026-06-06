use core::ffi::c_void;

use ntkrnl_sys::base::{ULONG, ULONG_PTR, PVOID, PULONG};

// kdvm.c statics (same DLL, other module).
use crate::kdvm::{KdVmBufferPos, KdVmBufferPhysicalAddress, KdVmDataBuffer};
// Header-only constant.
use crate::kdvm_h::KDVM_BUFFER_SIZE;

// kdvbox_asm.S: VOID FASTCALL KdVmExchange(ULONG_PTR PhysicalAddress, SIZE_T).
// FASTCALL on i386, cdecl/Win64 on amd64 — see crate::KdVmExchange. The decl
// must match the definition's ABI: extern "fastcall" on i386, "C" on amd64.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KdVmExchange(PhysicalAddress: ULONG_PTR, BufferSize: ULONG_PTR);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KdVmExchange(PhysicalAddress: ULONG_PTR, BufferSize: ULONG_PTR);
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct KDVBOX_SEND_HEADER {
    pub SendSize: ULONG,
    pub BufferSize: ULONG,
}

pub type PKDVBOX_SEND_HEADER = *mut KDVBOX_SEND_HEADER;

#[repr(C)]
pub struct KDVBOX_RECEIVE_HEADER {
    pub ReceivedDataSize: ULONG,
}

pub type PKDVBOX_RECEIVE_HEADER = *mut KDVBOX_RECEIVE_HEADER;
#[no_mangle]
pub unsafe extern "system" fn KdVmPrepareBuffer() {
    KdVmBufferPos = core::mem::size_of::<KDVBOX_SEND_HEADER>() as u32;
}

#[no_mangle]
pub unsafe extern "system" fn KdVmKdVmExchangeData(
    ReceiveData: *mut PVOID,
    ReceiveDataSize: PULONG,
) {
    let SendHeader: PKDVBOX_SEND_HEADER;
    let ReceiveHeader: PKDVBOX_RECEIVE_HEADER;

    SendHeader = core::ptr::addr_of_mut!(KdVmDataBuffer) as PKDVBOX_SEND_HEADER;
    (*SendHeader).SendSize = KdVmBufferPos - core::mem::size_of::<KDVBOX_SEND_HEADER>() as ULONG;
    (*SendHeader).BufferSize = KDVM_BUFFER_SIZE;

    KdVmExchange(KdVmBufferPhysicalAddress.QuadPart as ULONG_PTR, 0);

    KdVmBufferPos = core::mem::size_of::<KDVBOX_SEND_HEADER>() as ULONG;

    ReceiveHeader = core::ptr::addr_of_mut!(KdVmDataBuffer) as PKDVBOX_RECEIVE_HEADER;
    *ReceiveData = (core::ptr::addr_of_mut!(KdVmDataBuffer) as *mut u8)
        .add(core::mem::size_of::<KDVBOX_RECEIVE_HEADER>()) as PVOID;
    *ReceiveDataSize = (*ReceiveHeader).ReceivedDataSize;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, see COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            drivers/base/kdvm/kdvbox.c
//  * PURPOSE:         VBOX data exchange function for kdvbox
//  * PROGRAMMER:      Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// #include "kdvm.h"
//
// typedef struct
// {
//     ULONG SendSize;
//     ULONG BufferSize;
// } KDVBOX_SEND_HEADER, *PKDVBOX_SEND_HEADER;
//
// typedef struct
// {
//     ULONG ReceivedDataSize;
// } KDVBOX_RECEIVE_HEADER, *PKDVBOX_RECEIVE_HEADER;
//
// VOID
// NTAPI
// KdVmPrepareBuffer(VOID)
// {
//     KdVmBufferPos = sizeof(KDVBOX_SEND_HEADER);
// }
//
// VOID
// NTAPI
// KdVmKdVmExchangeData(
//      PVOID* ReceiveData,
//      PULONG ReceiveDataSize)
// {
//     PKDVBOX_SEND_HEADER SendHeader;
//     PKDVBOX_RECEIVE_HEADER ReceiveHeader;
//
//     /* Setup the send-header */
//     SendHeader = (PKDVBOX_SEND_HEADER)KdVmDataBuffer;
//     SendHeader->SendSize = KdVmBufferPos - sizeof(KDVBOX_SEND_HEADER);
//     SendHeader->BufferSize = KDVM_BUFFER_SIZE;
//
//     //KdpDbgPrint("Sending buffer:\n");
//     //KdVmDbgDumpBuffer(KdVmDataBuffer, KdVmBufferPos);
//
//     /* Do the data exchange */
//     KdVmExchange((ULONG_PTR)KdVmBufferPhysicalAddress.QuadPart, 0);
//
//     /* Reset the buffer position */
//     KdVmBufferPos = sizeof(KDVBOX_SEND_HEADER);
//
//     /* Get the receive-header and return information about the received data */
//     ReceiveHeader = (PKDVBOX_RECEIVE_HEADER)KdVmDataBuffer;
//     *ReceiveData = KdVmDataBuffer + sizeof(KDVBOX_RECEIVE_HEADER);
//     *ReceiveDataSize = ReceiveHeader->ReceivedDataSize;
//
//     //KdpDbgPrint("got data:\n");
//     //KdVmDbgDumpBuffer(KdVmDataBuffer, *ReceiveDataSize + sizeof(*ReceiveHeader));
//
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
