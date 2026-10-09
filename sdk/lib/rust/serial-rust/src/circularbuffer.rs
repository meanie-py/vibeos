use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use core::ffi::c_void;

extern "C" {
    fn DbgPrintEx( ComponentId: ULONG, Level: ULONG, Format: PCSTR, ... ) -> ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn InitializeCircularBuffer(
    mut pBuffer: PCIRCULAR_BUFFER,
    mut BufferSize: ULONG,
) -> NTSTATUS {
    const DPFLTR_SERIAL_ID: u32 = 39;
    const DPFLTR_TRACE_LEVEL: u32 = 2;
    const SERIAL_TAG: u32 = 0x6c726553;
    DbgPrintEx(
        DPFLTR_SERIAL_ID as ::core::ffi::c_int as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) InitializeCircularBuffer(pBuffer %p, BufferSize %lu)\n\0" as *const u8
            as PCSTR,
        b"drivers/serial/serial/circularbuffer.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        19 as ::core::ffi::c_int,
        pBuffer,
        BufferSize,
    );
    if pBuffer.is_null() {
        RtlAssert(
            b"pBuffer\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/serial/serial/circularbuffer.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            20 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    (*pBuffer).Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        (BufferSize as SIZE_T).wrapping_mul(::core::mem::size_of::<UCHAR>() as SIZE_T),
        SERIAL_TAG as ULONG,
    ) as PUCHAR;
    if (*pBuffer).Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*pBuffer).Length = BufferSize;
    (*pBuffer).WritePosition = 0 as ULONG;
    (*pBuffer).ReadPosition = (*pBuffer).WritePosition;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn FreeCircularBuffer(pBuffer: PCIRCULAR_BUFFER) -> NTSTATUS {
    const DPFLTR_SERIAL_ID: u32 = 39; // SERIAL component id
    const DPFLTR_TRACE_LEVEL: u32 = 2;
    const SERIAL_TAG: u32 = 0x6c726553; // 'Serl'

    DbgPrintEx(
        DPFLTR_SERIAL_ID as core::ffi::c_int as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) FreeCircularBuffer(pBuffer %p)\n\0" as *const u8 as PCSTR,
        b"drivers/serial/serial/circularbuffer.c\0"
            as *const u8 as *const core::ffi::c_char,
        33 as core::ffi::c_int,
        pBuffer,
    );
    if pBuffer.is_null() {
        RtlAssert(
            b"pBuffer\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/serial/serial/circularbuffer.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            34 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if !(*pBuffer).Buffer.is_null() {
        ExFreePoolWithTag((*pBuffer).Buffer as PVOID, SERIAL_TAG as ULONG);
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn IsCircularBufferEmpty(mut pBuffer: PCIRCULAR_BUFFER) -> BOOLEAN {
    const DPFLTR_SERIAL_ID: u32 = 39;
    const DPFLTR_TRACE_LEVEL: u32 = 2;
    DbgPrintEx(
        DPFLTR_SERIAL_ID as ::core::ffi::c_int as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) IsCircularBufferEmpty(pBuffer %p)\n\0" as *const u8 as PCSTR,
        b"drivers/serial/serial/circularbuffer.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        44 as ::core::ffi::c_int,
        pBuffer,
    );
    if pBuffer.is_null() {
        RtlAssert(
            b"pBuffer\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/serial/serial/circularbuffer.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            45 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return ((*pBuffer).ReadPosition == (*pBuffer).WritePosition) as ::core::ffi::c_int as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "C" fn GetNumberOfElementsInCircularBuffer(
    mut pBuffer: PCIRCULAR_BUFFER,
) -> ULONG {
    DbgPrintEx(
        DPFLTR_SERIAL_ID as ::core::ffi::c_int as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) GetNumberOfElementsInCircularBuffer(pBuffer %p)\n\0" as *const u8
            as PCSTR,
        b"drivers/serial/serial/circularbuffer.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        53 as ::core::ffi::c_int,
        pBuffer,
    );
    if pBuffer.is_null() {
        RtlAssert(
            b"pBuffer\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/serial/serial/circularbuffer.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            54 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return (*pBuffer)
        .WritePosition
        .wrapping_add((*pBuffer).Length)
        .wrapping_sub((*pBuffer).ReadPosition)
        .wrapping_rem((*pBuffer).Length);
}

#[no_mangle]
pub unsafe extern "C" fn PushCircularBufferEntry(
    mut pBuffer: PCIRCULAR_BUFFER,
    mut Entry: UCHAR,
) -> NTSTATUS {
    const DPFLTR_SERIAL_ID: u32 = 39;
    const DPFLTR_TRACE_LEVEL: u32 = 2;

    let mut NextPosition: ULONG = 0;
    DbgPrintEx(
        DPFLTR_SERIAL_ID as ::core::ffi::c_int as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) PushCircularBufferEntry(pBuffer %p, Entry 0x%x)\n\0" as *const u8
            as PCSTR,
        b"drivers/serial/serial/circularbuffer.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        64 as ::core::ffi::c_int,
        pBuffer,
        Entry as ::core::ffi::c_int,
    );
    if pBuffer.is_null() {
        RtlAssert(
            b"pBuffer\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/serial/serial/circularbuffer.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            65 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if (*pBuffer).Length == 0 {
        RtlAssert(
            b"pBuffer->Length\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/serial/serial/circularbuffer.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            66 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    NextPosition = (*pBuffer)
        .WritePosition
        .wrapping_add(1 as ULONG)
        .wrapping_rem((*pBuffer).Length);
    if NextPosition == (*pBuffer).ReadPosition {
        return STATUS_BUFFER_TOO_SMALL;
    }
    *(*pBuffer).Buffer.offset((*pBuffer).WritePosition as isize) = Entry as ::core::ffi::c_uchar;
    (*pBuffer).WritePosition = NextPosition;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PopCircularBufferEntry(
    mut pBuffer: PCIRCULAR_BUFFER,
    mut Entry: PUCHAR,
) -> NTSTATUS {
    const DPFLTR_SERIAL_ID: u32 = 39;
    const DPFLTR_TRACE_LEVEL: u32 = 2;
    const STATUS_ARRAY_BOUNDS_EXCEEDED: NTSTATUS = 0xC000008Cu32 as NTSTATUS;
    DbgPrintEx(
        DPFLTR_SERIAL_ID as core::ffi::c_int as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) PopCircularBufferEntry(pBuffer %p)\n\0" as *const u8 as PCSTR,
        b"circularbuffer.c\0" as *const u8 as *const core::ffi::c_char,
        80 as core::ffi::c_int,
        pBuffer,
    );
    if pBuffer.is_null() {
        RtlAssert(
            b"pBuffer\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"circularbuffer.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
            81 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if (*pBuffer).Length == 0 {
        RtlAssert(
            b"pBuffer->Length\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"circularbuffer.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
            82 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if IsCircularBufferEmpty(pBuffer) != 0 {
        return STATUS_ARRAY_BOUNDS_EXCEEDED;
    }
    *Entry = *(*pBuffer).Buffer.offset((*pBuffer).ReadPosition as isize);
    (*pBuffer).ReadPosition = (*pBuffer)
        .ReadPosition
        .wrapping_add(1 as ULONG)
        .wrapping_rem((*pBuffer).Length);
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn IncreaseCircularBufferSize(
    pBuffer: PCIRCULAR_BUFFER,
    NewBufferSize: ULONG,
) -> NTSTATUS {
    const DPFLTR_SERIAL_ID: i32 = 39;
    const DPFLTR_TRACE_LEVEL: u32 = 2;
    const SERIAL_TAG: u32 = 0x6c726553; // 'Serl'

    let mut NewBuffer: PUCHAR = core::ptr::null_mut::<core::ffi::c_uchar>();

    DbgPrintEx(
        DPFLTR_SERIAL_ID as core::ffi::c_int as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) IncreaseCircularBufferSize(pBuffer %p, NewBufferSize %lu)\n\0" as *const u8 as PCSTR,
        b"circularbuffer.c\0" as *const u8 as *const core::ffi::c_char,
        97 as core::ffi::c_int,
        pBuffer,
        NewBufferSize,
    );

    if pBuffer.is_null() {
        RtlAssert(
            b"pBuffer\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"circularbuffer.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
            98 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if (*pBuffer).Length == 0 {
        RtlAssert(
            b"pBuffer->Length\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"circularbuffer.c\0" as *const u8 as *const core::ffi::c_char as PVOID,
            99 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    if (*pBuffer).Length > NewBufferSize {
        return STATUS_INVALID_PARAMETER;
    } else if (*pBuffer).Length == NewBufferSize {
        return STATUS_SUCCESS;
    }

    NewBuffer = ExAllocatePoolWithTag(
        NonPagedPool,
        (NewBufferSize as SIZE_T).wrapping_mul(core::mem::size_of::<UCHAR>() as SIZE_T),
        SERIAL_TAG as ULONG,
    ) as PUCHAR;
    if NewBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyMemory(
        NewBuffer as *mut core::ffi::c_void,
        (*pBuffer).Buffer as *const core::ffi::c_void,
        ((*pBuffer).Length as SIZE_T).wrapping_mul(core::mem::size_of::<UCHAR>() as SIZE_T),
    );
    ExFreePoolWithTag((*pBuffer).Buffer as PVOID, SERIAL_TAG as ULONG);
    (*pBuffer).Buffer = NewBuffer;
    (*pBuffer).Length = NewBufferSize;
    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Serial port driver
//  * FILE:            drivers/dd/serial/circularbuffer.c
//  * PURPOSE:         Operations on a circular buffer
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "serial.h"
//
// #include <debug.h>
//
// NTSTATUS
// InitializeCircularBuffer(
// 	IN PCIRCULAR_BUFFER pBuffer,
// 	IN ULONG BufferSize)
// {
// 	TRACE_!(SERIAL, "InitializeCircularBuffer(pBuffer %p, BufferSize %lu)\n", pBuffer, BufferSize);
// 	ASSERT(pBuffer);
// 	pBuffer->Buffer = (PUCHAR)ExAllocatePoolWithTag(NonPagedPool, BufferSize * sizeof(UCHAR), SERIAL_TAG);
// 	if (!pBuffer->Buffer)
// 		return STATUS_INSUFFICIENT_RESOURCES;
// 	pBuffer->Length = BufferSize;
// 	pBuffer->ReadPosition = pBuffer->WritePosition = 0;
// 	return STATUS_SUCCESS;
// }
//
// NTSTATUS
// FreeCircularBuffer(
// 	IN PCIRCULAR_BUFFER pBuffer)
// {
// 	TRACE_!(SERIAL, "FreeCircularBuffer(pBuffer %p)\n", pBuffer);
// 	ASSERT(pBuffer);
// 	if (pBuffer->Buffer != NULL)
// 		ExFreePoolWithTag(pBuffer->Buffer, SERIAL_TAG);
// 	return STATUS_SUCCESS;
// }
//
// BOOLEAN
// IsCircularBufferEmpty(
// 	IN PCIRCULAR_BUFFER pBuffer)
// {
// 	TRACE_!(SERIAL, "IsCircularBufferEmpty(pBuffer %p)\n", pBuffer);
// 	ASSERT(pBuffer);
// 	return (pBuffer->ReadPosition == pBuffer->WritePosition);
// }
//
// ULONG
// GetNumberOfElementsInCircularBuffer(
// 	IN PCIRCULAR_BUFFER pBuffer)
// {
// 	TRACE_!(SERIAL, "GetNumberOfElementsInCircularBuffer(pBuffer %p)\n", pBuffer);
// 	ASSERT(pBuffer);
// 	return (pBuffer->WritePosition + pBuffer->Length - pBuffer->ReadPosition) % pBuffer->Length;
// }
//
// NTSTATUS
// PushCircularBufferEntry(
// 	IN PCIRCULAR_BUFFER pBuffer,
// 	IN UCHAR Entry)
// {
// 	ULONG NextPosition;
// 	TRACE_!(SERIAL, "PushCircularBufferEntry(pBuffer %p, Entry 0x%x)\n", pBuffer, Entry);
// 	ASSERT(pBuffer);
// 	ASSERT(pBuffer->Length);
// 	NextPosition = (pBuffer->WritePosition + 1) % pBuffer->Length;
// 	if (NextPosition == pBuffer->ReadPosition)
// 		return STATUS_BUFFER_TOO_SMALL;
// 	pBuffer->Buffer[pBuffer->WritePosition] = Entry;
// 	pBuffer->WritePosition = NextPosition;
// 	return STATUS_SUCCESS;
// }
//
// NTSTATUS
// PopCircularBufferEntry(
// 	IN PCIRCULAR_BUFFER pBuffer,
// 	OUT PUCHAR Entry)
// {
// 	TRACE_!(SERIAL, "PopCircularBufferEntry(pBuffer %p)\n", pBuffer);
// 	ASSERT(pBuffer);
// 	ASSERT(pBuffer->Length);
// 	if (IsCircularBufferEmpty(pBuffer))
// 		return STATUS_ARRAY_BOUNDS_EXCEEDED;
// 	*Entry = pBuffer->Buffer[pBuffer->ReadPosition];
// 	pBuffer->ReadPosition = (pBuffer->ReadPosition + 1) % pBuffer->Length;
// 	return STATUS_SUCCESS;
// }
//
// NTSTATUS
// IncreaseCircularBufferSize(
// 	IN PCIRCULAR_BUFFER pBuffer,
// 	IN ULONG NewBufferSize)
// {
// 	PUCHAR NewBuffer;
//
// 	TRACE_!(SERIAL, "IncreaseCircularBufferSize(pBuffer %p, NewBufferSize %lu)\n", pBuffer, NewBufferSize);
// 	ASSERT(pBuffer);
// 	ASSERT(pBuffer->Length);
// 	if (pBuffer->Length > NewBufferSize)
// 		return STATUS_INVALID_PARAMETER;
// 	else if (pBuffer->Length == NewBufferSize)
// 		return STATUS_SUCCESS;
//
// 	NewBuffer = (PUCHAR)ExAllocatePoolWithTag(NonPagedPool, NewBufferSize * sizeof(UCHAR), SERIAL_TAG);
// 	if (!NewBuffer)
// 		return STATUS_INSUFFICIENT_RESOURCES;
// 	RtlCopyMemory(NewBuffer, pBuffer->Buffer, pBuffer->Length * sizeof(UCHAR));
// 	ExFreePoolWithTag(pBuffer->Buffer, SERIAL_TAG);
// 	pBuffer->Buffer = NewBuffer;
// 	pBuffer->Length = NewBufferSize;
// 	return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
