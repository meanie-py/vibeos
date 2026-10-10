use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePool, ExFreePool, IoAllocateMdl, IoBuildPartialMdl, IoFreeMdl, MmBuildMdlForNonPagedPool};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The C preamble contains only `#undef` directives, which have no Rust declarations to emit. No constants, types, or tables are defined here.
#[no_mangle]
pub unsafe extern "C" fn SkipToOffset(
    mut Buffer: PNDIS_BUFFER,
    mut Offset: UINT,
    mut Data: *mut PUCHAR,
    mut Size: PUINT,
) -> ULONG {
    loop {
        if Buffer.is_null() {
            return 0xFFFFFFFF;
        }

        NdisQueryBuffer(Buffer, (Data as PVOID).cast(), Size);

        if Offset < *Size {
            *Data = (*Data as ULONG_PTR).wrapping_add(Offset as ULONG_PTR) as PUCHAR;
            *Size = (*Size).wrapping_sub(Offset);
            break;
        }

        Offset = Offset.wrapping_sub(*Size);

        NdisGetNextBuffer!(Buffer, core::ptr::addr_of_mut!(Buffer));
    }

    return Offset as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn CopyBufferToBufferChain(
    mut DstBuffer: PNDIS_BUFFER,
    mut DstOffset: UINT,
    mut SrcData: PUCHAR,
    mut Length: UINT,
) -> UINT {
    let mut BytesCopied: UINT = 0;
    let mut BytesToCopy: UINT = 0;
    let mut DstSize: UINT = 0;
    let mut DstData: PUCHAR = core::ptr::null_mut();

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"DstBuffer (0x%X)  DstOffset (0x%X)  SrcData (0x%X)  Length (%d)\n\0".as_ptr() as *const i8,
        DstBuffer,
        DstOffset,
        SrcData,
        Length,
    );

    /* Skip DstOffset bytes in the destination buffer chain */
    if SkipToOffset(
        DstBuffer,
        DstOffset,
        core::ptr::addr_of_mut!(DstData),
        core::ptr::addr_of_mut!(DstSize),
    ) == 0xFFFFFFFF
    {
        return 0;
    }

    /* Start copying the data */
    BytesCopied = 0;
    loop {
        BytesToCopy = MIN!(DstSize, Length);

        RtlCopyMemory!(DstData as PVOID, SrcData as PVOID, BytesToCopy);
        BytesCopied = BytesCopied.wrapping_add(BytesToCopy);
        SrcData = (SrcData as ULONG_PTR).wrapping_add(BytesToCopy as ULONG_PTR) as PUCHAR;

        Length = Length.wrapping_sub(BytesToCopy);
        if Length == 0 {
            break;
        }

        DstSize = DstSize.wrapping_sub(BytesToCopy);
        if DstSize == 0 {
            /* No more bytes in destination buffer. Proceed to
               the next buffer in the destination buffer chain */
            NdisGetNextBuffer!(DstBuffer, core::ptr::addr_of_mut!(DstBuffer));
            if DstBuffer.is_null() {
                break;
            }

            NdisQueryBuffer(
                DstBuffer,
                (core::ptr::addr_of_mut!(DstData) as PVOID).cast(),
                core::ptr::addr_of_mut!(DstSize),
            );
        }
    }

    BytesCopied
}
#[no_mangle]
pub unsafe extern "C" fn CopyBufferChainToBuffer(
    mut DstData: PUCHAR,
    mut SrcBuffer: PNDIS_BUFFER,
    mut SrcOffset: UINT,
    mut Length: UINT,
) -> UINT {
    let mut BytesCopied: UINT = 0;
    let mut BytesToCopy: UINT = 0;
    let mut SrcSize: UINT = 0;
    let mut SrcData: PUCHAR = core::ptr::null_mut();

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"DstData 0x%X  SrcBuffer 0x%X  SrcOffset 0x%X  Length %d\n\0".as_ptr() as PCSTR,
        DstData,
        SrcBuffer,
        SrcOffset,
        Length,
    );

    /* Skip SrcOffset bytes in the source buffer chain */
    if SkipToOffset(
        SrcBuffer,
        SrcOffset,
        core::ptr::addr_of_mut!(SrcData),
        core::ptr::addr_of_mut!(SrcSize),
    ) == 0xFFFFFFFF
    {
        return 0;
    }

    /* Start copying the data */
    BytesCopied = 0;
    loop {
        BytesToCopy = MIN!(SrcSize, Length);

        NDIS_DbgPrint!(
            MAX_TRACE,
            b"Copying (%d) bytes from 0x%X to 0x%X\n\0".as_ptr() as PCSTR,
            BytesToCopy,
            SrcData,
            DstData,
        );

        RtlCopyMemory!(DstData as PVOID, SrcData as PVOID, BytesToCopy as usize);
        BytesCopied = BytesCopied.wrapping_add(BytesToCopy);
        DstData = (DstData as ULONG_PTR).wrapping_add(BytesToCopy as ULONG_PTR) as PUCHAR;

        Length = Length.wrapping_sub(BytesToCopy);
        if Length == 0 {
            break;
        }

        SrcSize = SrcSize.wrapping_sub(BytesToCopy);
        if SrcSize == 0 {
            /* No more bytes in source buffer. Proceed to
               the next buffer in the source buffer chain */
            NdisGetNextBuffer!(SrcBuffer, core::ptr::addr_of_mut!(SrcBuffer));
            if SrcBuffer.is_null() {
                break;
            }

            NdisQueryBuffer(
                SrcBuffer,
                (core::ptr::addr_of_mut!(SrcData) as PVOID).cast(),
                core::ptr::addr_of_mut!(SrcSize),
            );
        }
    }

    BytesCopied
}
#[no_mangle]
pub unsafe extern "C" fn CopyPacketToBuffer(
    DstData: PUCHAR,
    SrcPacket: PNDIS_PACKET,
    SrcOffset: UINT,
    Length: UINT,
) -> UINT {
    let mut FirstBuffer: PNDIS_BUFFER = core::ptr::null_mut();
    let mut Address: PVOID = core::ptr::null_mut();
    let mut FirstLength: UINT = 0;
    let mut TotalLength: UINT = 0;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"DstData (0x%X)  SrcPacket (0x%X)  SrcOffset (0x%X)  Length (%d)\n\0".as_ptr() as PCSTR,
        DstData,
        SrcPacket,
        SrcOffset,
        Length,
    );

    NdisGetFirstBufferFromPacket(
        SrcPacket,
        core::ptr::addr_of_mut!(FirstBuffer),
        core::ptr::addr_of_mut!(Address),
        core::ptr::addr_of_mut!(FirstLength),
        core::ptr::addr_of_mut!(TotalLength),
    );

    return CopyBufferChainToBuffer(DstData, FirstBuffer, SrcOffset, Length);
}
#[no_mangle]
pub unsafe extern "C" fn CopyPacketToBufferChain(
    mut DstBuffer: PNDIS_BUFFER,
    mut DstOffset: UINT,
    mut SrcPacket: PNDIS_PACKET,
    mut SrcOffset: UINT,
    mut Length: UINT,
) -> UINT {
    let mut SrcBuffer: PNDIS_BUFFER = core::ptr::null_mut();
    let mut DstData: PUCHAR = core::ptr::null_mut();
    let mut SrcData: PUCHAR = core::ptr::null_mut();
    let mut DstSize: UINT = 0;
    let mut SrcSize: UINT = 0;
    let mut Count: UINT = 0;
    let mut Total: UINT = 0;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"DstBuffer (0x%X)  DstOffset (0x%X)  SrcPacket (0x%X)  SrcOffset (0x%X)  Length (%d)\n\0".as_ptr() as PCSTR,
        DstBuffer,
        DstOffset,
        SrcPacket,
        SrcOffset,
        Length,
    );

    /* Skip DstOffset bytes in the destination buffer chain */
    NdisQueryBuffer(DstBuffer, (core::ptr::addr_of_mut!(DstData) as PVOID).cast(), core::ptr::addr_of_mut!(DstSize));
    if SkipToOffset(DstBuffer, DstOffset, core::ptr::addr_of_mut!(DstData), core::ptr::addr_of_mut!(DstSize)) == 0xFFFFFFFF {
        return 0;
    }
    /* Skip SrcOffset bytes in the source packet */
    NdisGetFirstBufferFromPacket(
        SrcPacket,
        core::ptr::addr_of_mut!(SrcBuffer),
        core::ptr::addr_of_mut!(SrcData) as *mut PVOID,
        core::ptr::addr_of_mut!(SrcSize),
        core::ptr::addr_of_mut!(Total),
    );
    if SkipToOffset(SrcBuffer, SrcOffset, core::ptr::addr_of_mut!(SrcData), core::ptr::addr_of_mut!(SrcSize)) == 0xFFFFFFFF {
        return 0;
    }
    /* Copy the data */
    Total = 0;
    loop {
        /* Find out how many bytes we can copy at one time */
        if Length < SrcSize {
            Count = Length;
        } else {
            Count = SrcSize;
        }
        if DstSize < Count {
            Count = DstSize;
        }

        core::ptr::copy_nonoverlapping(SrcData as *const u8, DstData as *mut u8, Count as usize);

        Total = Total.wrapping_add(Count);
        Length = Length.wrapping_sub(Count);
        if Length == 0 {
            break;
        }

        DstSize = DstSize.wrapping_sub(Count);
        if DstSize == 0 {
            /* No more bytes in destination buffer. Proceed to
               the next buffer in the destination buffer chain */
            NdisGetNextBuffer!(DstBuffer, core::ptr::addr_of_mut!(DstBuffer));
            if DstBuffer.is_null() {
                break;
            }

            NdisQueryBuffer(DstBuffer, (core::ptr::addr_of_mut!(DstData) as PVOID).cast(), core::ptr::addr_of_mut!(DstSize));
        }

        SrcSize = SrcSize.wrapping_sub(Count);
        if SrcSize == 0 {
            /* No more bytes in source buffer. Proceed to
               the next buffer in the source buffer chain */
            NdisGetNextBuffer!(SrcBuffer, core::ptr::addr_of_mut!(SrcBuffer));
            if SrcBuffer.is_null() {
                break;
            }

            NdisQueryBuffer(SrcBuffer, (core::ptr::addr_of_mut!(SrcData) as PVOID).cast(), core::ptr::addr_of_mut!(SrcSize));
        }
    }

    return Total;
}
#[no_mangle]
pub unsafe extern "system" fn NdisAdjustBufferLength(Buffer: PNDIS_BUFFER, Length: UINT) {
    (*Buffer).ByteCount = Length as ULONG;
}
#[no_mangle]
pub unsafe extern "system" fn NDIS_BUFFER_TO_SPAN_PAGES(mut Buffer: PNDIS_BUFFER) -> ULONG {
    if MmGetMdlByteCount!(Buffer as PMDL) == 0 {
        return 1;
    }

    return (ADDRESS_AND_SIZE_TO_SPAN_PAGES(
        MmGetMdlVirtualAddress!(Buffer as PMDL),
        (MmGetMdlByteCount!(Buffer as PMDL)) as _,
    )) as ULONG;
}
#[no_mangle]
pub unsafe extern "system" fn NdisAllocateBuffer(
    Status: PNDIS_STATUS,
    Buffer: *mut PNDIS_BUFFER,
    PoolHandle: NDIS_HANDLE,
    VirtualAddress: PVOID,
    Length: UINT,
) {
    if VirtualAddress.is_null() {
        RtlAssert(
            b"VirtualAddress != NULL\0".as_ptr() as PVOID,
            b"buffer.c\0".as_ptr() as PVOID,
            352 as ULONG,
            core::ptr::null_mut(),
        );
    }
    if !(Length > 0) {
        RtlAssert(
            b"Length > 0\0".as_ptr() as PVOID,
            b"buffer.c\0".as_ptr() as PVOID,
            353 as ULONG,
            core::ptr::null_mut(),
        );
    }

    *Buffer = IoAllocateMdl(
        VirtualAddress,
        Length as ULONG,
        FALSE as BOOLEAN,
        FALSE as BOOLEAN,
        core::ptr::null_mut(),
    ) as PNDIS_BUFFER;
    if !(*Buffer).is_null() {
        MmBuildMdlForNonPagedPool(*Buffer);
        (**Buffer).Next = core::ptr::null_mut();
        *Status = NDIS_STATUS_SUCCESS;
    } else {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"IoAllocateMdl failed (%x, %lx)\n\0".as_ptr() as PCSTR,
            VirtualAddress,
            Length,
        );
        *Status = NDIS_STATUS_FAILURE;
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisAllocateBufferPool(
    Status: PNDIS_STATUS,
    PoolHandle: PNDIS_HANDLE,
    NumberOfDescriptors: UINT,
) {
    *Status = NDIS_STATUS_SUCCESS;
    *PoolHandle = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn NdisAllocatePacket(
    Status: PNDIS_STATUS,
    Packet: *mut PNDIS_PACKET,
    PoolHandle: NDIS_HANDLE,
) {
    let Pool: PNDISI_PACKET_POOL = PoolHandle as PNDISI_PACKET_POOL;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*Pool).SpinLock.SpinLock),
        core::ptr::addr_of_mut!((*Pool).SpinLock.OldIrql),
    );
    NdisDprAllocatePacketNonInterlocked(Status, Packet, PoolHandle);
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*Pool).SpinLock.SpinLock),
        (*Pool).SpinLock.OldIrql,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisAllocatePacketPool(
    Status: PNDIS_STATUS,
    PoolHandle: PNDIS_HANDLE,
    NumberOfDescriptors: UINT,
    ProtocolReservedLength: UINT,
) {
    NdisAllocatePacketPoolEx(
        Status,
        PoolHandle,
        NumberOfDescriptors,
        0,
        ProtocolReservedLength,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisAllocatePacketPoolEx(
    Status: PNDIS_STATUS,
    PoolHandle: PNDIS_HANDLE,
    mut NumberOfDescriptors: UINT,
    NumberOfOverflowDescriptors: UINT,
    ProtocolReservedLength: UINT,
) {
    let mut Pool: PNDISI_PACKET_POOL = core::ptr::null_mut();
    let mut Size: UINT = 0;
    let mut Length: UINT = 0;
    let mut i: UINT = 0;
    let mut Packet: PNDIS_PACKET = core::ptr::null_mut();
    let mut NextPacket: PNDIS_PACKET = core::ptr::null_mut();

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Status (0x%X)  PoolHandle (0x%X)  NumberOfDescriptors (%d)  ProtocolReservedLength (%d).\n\0".as_ptr() as PCSTR,
        Status,
        PoolHandle,
        NumberOfDescriptors,
        ProtocolReservedLength,
    );

    *PoolHandle = core::ptr::null_mut();

    if NumberOfDescriptors > 0xffff {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Invalid number of descriptors (%lx)\n\0".as_ptr() as PCSTR,
            NumberOfDescriptors,
        );
        *Status = NDIS_STATUS_RESOURCES;
    } else {
        NumberOfDescriptors = NumberOfDescriptors.wrapping_add(NumberOfOverflowDescriptors);
        if NumberOfDescriptors > 0xffff {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Total number of descriptors > 0xffff (%lx)\n\0".as_ptr() as PCSTR,
                NumberOfDescriptors,
            );
            NumberOfDescriptors = 0xffff;
        }

        Length = (core::mem::size_of::<NDIS_PACKET>() as usize)
            .wrapping_add(core::mem::size_of::<NDIS_PACKET_OOB_DATA>() as usize)
            .wrapping_add(core::mem::size_of::<NDIS_PACKET_EXTENSION>() as usize)
            .wrapping_add(ProtocolReservedLength as usize) as UINT;
        Size = (core::mem::size_of::<NDISI_PACKET_POOL>() as usize)
            .wrapping_add(Length.wrapping_mul(NumberOfDescriptors) as usize) as UINT;

        Pool = ExAllocatePool(NonPagedPool, Size as SIZE_T) as PNDISI_PACKET_POOL;
        if !Pool.is_null() {
            KeInitializeSpinLock(core::ptr::addr_of_mut!((*Pool).SpinLock.SpinLock));
            (*Pool).PacketLength = Length;

            if NumberOfDescriptors > 0 {
                Packet = core::ptr::addr_of_mut!((*Pool).Buffer) as PNDIS_PACKET;
                (*Pool).FreeList = Packet as *mut _NDIS_PACKET;

                NextPacket = (Packet as ULONG_PTR).wrapping_add(Length as ULONG_PTR) as PNDIS_PACKET;
                i = 1;
                while i < NumberOfDescriptors {
                    (*Packet).Reserved[0 as usize] = NextPacket as ULONG_PTR;
                    Packet = NextPacket;
                    NextPacket = (Packet as ULONG_PTR).wrapping_add(Length as ULONG_PTR) as PNDIS_PACKET;
                    i = i.wrapping_add(1);
                }
                (*Packet).Reserved[0 as usize] = 0;
            } else {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"Attempted to allocate a packet pool with 0 descriptors\n\0".as_ptr() as PCSTR,
                );
                (*Pool).FreeList = core::ptr::null_mut();
            }

            *Status = NDIS_STATUS_SUCCESS;
            *PoolHandle = Pool as PNDIS_HANDLE as PVOID;
        } else {
            *Status = NDIS_STATUS_RESOURCES;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisBufferLength(Buffer: PNDIS_BUFFER) -> ULONG {
    MmGetMdlByteCount!(Buffer)
}

#[no_mangle]
pub unsafe extern "system" fn NdisBufferVirtualAddress(Buffer: PNDIS_BUFFER) -> PVOID {
    MmGetSystemAddressForMdl!(Buffer)
}

unsafe extern "system" fn NdisCopyFromPacketToPacket_placeholder() {}
#[no_mangle]
pub unsafe extern "system" fn NdisCopyFromPacketToPacket(
    Destination: PNDIS_PACKET,
    DestinationOffset: UINT,
    mut BytesToCopy: UINT,
    Source: PNDIS_PACKET,
    SourceOffset: UINT,
    BytesCopied: PUINT,
) {
    let mut SrcBuffer: PNDIS_BUFFER = core::ptr::null_mut();
    let mut DstBuffer: PNDIS_BUFFER = core::ptr::null_mut();
    let mut DstData: PUCHAR = core::ptr::null_mut();
    let mut SrcData: PUCHAR = core::ptr::null_mut();
    let mut DstSize: UINT = 0;
    let mut SrcSize: UINT = 0;
    let mut Count: UINT = 0;
    let mut Total: UINT = 0;

    *BytesCopied = 0;

    /* Skip DestinationOffset bytes in the destination packet */
    NdisGetFirstBufferFromPacket(
        Destination,
        core::ptr::addr_of_mut!(DstBuffer),
        core::ptr::addr_of_mut!(DstData) as *mut PVOID,
        core::ptr::addr_of_mut!(DstSize),
        core::ptr::addr_of_mut!(Total),
    );
    if SkipToOffset(
        DstBuffer,
        DestinationOffset,
        core::ptr::addr_of_mut!(DstData),
        core::ptr::addr_of_mut!(DstSize),
    ) == 0xFFFFFFFF
    {
        return;
    }

    /* Skip SourceOffset bytes in the source packet */
    NdisGetFirstBufferFromPacket(
        Source,
        core::ptr::addr_of_mut!(SrcBuffer),
        core::ptr::addr_of_mut!(SrcData) as *mut PVOID,
        core::ptr::addr_of_mut!(SrcSize),
        core::ptr::addr_of_mut!(Total),
    );
    if SkipToOffset(
        SrcBuffer,
        SourceOffset,
        core::ptr::addr_of_mut!(SrcData),
        core::ptr::addr_of_mut!(SrcSize),
    ) == 0xFFFFFFFF
    {
        return;
    }

    /* Copy the data */
    Total = 0;
    loop {
        /* Find out how many bytes we can copy at one time */
        if BytesToCopy < SrcSize {
            Count = BytesToCopy;
        } else {
            Count = SrcSize;
        }
        if DstSize < Count {
            Count = DstSize;
        }

        RtlCopyMemory!(DstData as PVOID, SrcData as *const core::ffi::c_void, Count as SIZE_T);

        Total = Total.wrapping_add(Count);
        BytesToCopy = BytesToCopy.wrapping_sub(Count);
        if BytesToCopy == 0 {
            break;
        }

        DstSize = DstSize.wrapping_sub(Count);
        if DstSize == 0 {
            /* No more bytes in destination buffer. Proceed to
               the next buffer in the destination buffer chain */
            NdisGetNextBuffer!(DstBuffer, core::ptr::addr_of_mut!(DstBuffer));
            if DstBuffer.is_null() {
                break;
            }

            NdisQueryBuffer(
                DstBuffer,
                (core::ptr::addr_of_mut!(DstData) as PVOID).cast(),
                core::ptr::addr_of_mut!(DstSize),
            );
        }

        SrcSize = SrcSize.wrapping_sub(Count);
        if SrcSize == 0 {
            /* No more bytes in source buffer. Proceed to
               the next buffer in the source buffer chain */
            NdisGetNextBuffer!(SrcBuffer, core::ptr::addr_of_mut!(SrcBuffer));
            if SrcBuffer.is_null() {
                break;
            }

            NdisQueryBuffer(
                SrcBuffer,
                (core::ptr::addr_of_mut!(SrcData) as PVOID).cast(),
                core::ptr::addr_of_mut!(SrcSize),
            );
        }
    }

    *BytesCopied = Total;
}
#[no_mangle]
pub unsafe extern "system" fn NdisDprAllocatePacket(
    Status: PNDIS_STATUS,
    Packet: *mut PNDIS_PACKET,
    PoolHandle: NDIS_HANDLE,
) {
    let Pool: PNDISI_PACKET_POOL = PoolHandle as PNDISI_PACKET_POOL;
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*Pool).SpinLock.SpinLock));
    NdisDprAllocatePacketNonInterlocked(Status, Packet, PoolHandle);
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*Pool).SpinLock.SpinLock));
}
#[no_mangle]
pub unsafe extern "system" fn NdisDprAllocatePacketNonInterlocked(
    Status: PNDIS_STATUS,
    Packet: *mut PNDIS_PACKET,
    PoolHandle: NDIS_HANDLE,
) {
    let mut Temp: PNDIS_PACKET = core::ptr::null_mut();
    let Pool: PNDISI_PACKET_POOL = PoolHandle as PNDISI_PACKET_POOL;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Status (0x%X)  Packet (0x%X)  PoolHandle (0x%X).\n\0".as_ptr() as *const i8,
        Status,
        Packet,
        PoolHandle,
    );

    *Packet = core::ptr::null_mut();

    if Pool.is_null() {
        *Status = NDIS_STATUS_FAILURE;
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Called passed a bad pool handle\n\0".as_ptr() as *const i8,
        );
        return;
    }

    if !(*Pool).FreeList.is_null() {
        Temp = (*Pool).FreeList as PNDIS_PACKET;
        (*Pool).FreeList =
            (*Temp).Reserved[0 as usize] as PNDIS_PACKET as *mut _NDIS_PACKET;

        RtlZeroMemory!(Temp as *mut core::ffi::c_void, (*Pool).PacketLength as usize);
        (*Temp).Private.Pool = Pool as PNDIS_PACKET_POOL;
        (*Temp).Private.ValidCounts = TRUE;
        (*Temp).Private.NdisPacketFlags = fPACKET_ALLOCATED_BY_NDIS as UCHAR;
        (*Temp).Private.NdisPacketOobOffset = ((*Pool).PacketLength as usize).wrapping_sub(
            (core::mem::size_of::<NDIS_PACKET_OOB_DATA>() as usize)
                .wrapping_add(core::mem::size_of::<NDIS_PACKET_EXTENSION>() as usize),
        ) as USHORT;

        *Packet = Temp;
        *Status = NDIS_STATUS_SUCCESS;
    } else {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"No more free descriptors\n\0".as_ptr() as *const i8,
        );
        *Status = NDIS_STATUS_RESOURCES;
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisDprFreePacket(Packet: PNDIS_PACKET) {
    let Pool: PNDISI_PACKET_POOL = (*Packet).Private.Pool as PNDISI_PACKET_POOL;
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*Pool).SpinLock.SpinLock));
    NdisDprFreePacketNonInterlocked(Packet);
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*Pool).SpinLock.SpinLock));
}
#[no_mangle]
pub unsafe extern "system" fn NdisDprFreePacketNonInterlocked(Packet: PNDIS_PACKET) {
    NDIS_DbgPrint!(MAX_TRACE, b"Packet (0x%X).\n\0".as_ptr() as PCSTR, Packet);

    (*Packet).Reserved[0] =
        (*((*Packet).Private.Pool as *mut NDISI_PACKET_POOL)).FreeList as ULONG_PTR;
    let ref mut fresh16 = (*((*Packet).Private.Pool as *mut NDISI_PACKET_POOL)).FreeList;
    *fresh16 = Packet as *mut _NDIS_PACKET;
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeBufferPool(PoolHandle: NDIS_HANDLE) {}
#[no_mangle]
pub unsafe extern "system" fn NdisFreePacketPool(PoolHandle: NDIS_HANDLE) {
    ExFreePool(PoolHandle as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeBuffer(Buffer: PNDIS_BUFFER) {
    IoFreeMdl(Buffer);
}
#[no_mangle]
pub unsafe extern "system" fn NdisFreePacket(Packet: PNDIS_PACKET) {
    let Pool: PNDISI_PACKET_POOL = (*Packet).Private.Pool as PNDISI_PACKET_POOL;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*Pool).SpinLock.SpinLock),
        core::ptr::addr_of_mut!((*Pool).SpinLock.OldIrql),
    );
    NdisDprFreePacketNonInterlocked(Packet);
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*Pool).SpinLock.SpinLock),
        (*Pool).SpinLock.OldIrql,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisGetBufferPhysicalArraySize(
    mut Buffer: PNDIS_BUFFER,
    mut ArraySize: PUINT,
) {
    if !(KeGetCurrentIrql() as i32 <= 2) {
        RtlAssert(
            b"KeGetCurrentIrql() <= DISPATCH_LEVEL\0".as_ptr() as PVOID,
            b"drivers/network/ndis/ndis/buffer.c\0".as_ptr() as PVOID,
            860,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    if !(!Buffer.is_null() && !ArraySize.is_null()) {
        RtlAssert(
            b"Buffer && ArraySize\0".as_ptr() as PVOID,
            b"drivers/network/ndis/ndis/buffer.c\0".as_ptr() as PVOID,
            861,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    *ArraySize = NDIS_BUFFER_TO_SPAN_PAGES(Buffer) as UINT;
}
#[no_mangle]
pub unsafe extern "system" fn NdisGetFirstBufferFromPacket(
    _Packet: PNDIS_PACKET,
    _FirstBuffer: *mut PNDIS_BUFFER,
    _FirstBufferVA: *mut PVOID,
    _FirstBufferLength: PUINT,
    _TotalBufferLength: PUINT,
) {
    let mut Buffer: PNDIS_BUFFER = core::ptr::null_mut();
    Buffer = (*_Packet).Private.Head;
    *_FirstBuffer = Buffer;

    if !Buffer.is_null() {
        *_FirstBufferLength = MmGetMdlByteCount!(Buffer as PMDL);
        *_FirstBufferVA = MmGetSystemAddressForMdl!(Buffer as PMDL);
        Buffer = (*Buffer).Next;
    } else {
        NDIS_DbgPrint!(
            MID_TRACE,
            b"No buffers linked to this packet\n\0".as_ptr() as *const i8,
        );
        *_FirstBufferLength = 0;
        *_FirstBufferVA = core::ptr::null_mut();
    }

    *_TotalBufferLength = *_FirstBufferLength;

    while !Buffer.is_null() {
        *_TotalBufferLength = (*_TotalBufferLength).wrapping_add(MmGetMdlByteCount!(Buffer as PMDL));
        Buffer = (*Buffer).Next;
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisGetFirstBufferFromPacketSafe(
    _Packet: PNDIS_PACKET,
    _FirstBuffer: *mut PNDIS_BUFFER,
    _FirstBufferVA: *mut PVOID,
    _FirstBufferLength: PUINT,
    _TotalBufferLength: PUINT,
    Priority: MM_PAGE_PRIORITY,
) {
    let mut Buffer: PNDIS_BUFFER;

    Buffer = (*_Packet).Private.Head;
    *_FirstBuffer = Buffer;

    if !Buffer.is_null() {
        *_FirstBufferLength = MmGetMdlByteCount!(Buffer as PMDL);
        *_FirstBufferVA = MmGetSystemAddressForMdlSafe(Buffer as PMDL, Priority);
        Buffer = (*Buffer).Next;
    } else {
        NDIS_DbgPrint!(
            MID_TRACE,
            b"No buffers linked to this packet\n\0".as_ptr() as *const i8,
        );
        *_FirstBufferLength = 0;
        *_FirstBufferVA = core::ptr::null_mut();
    }

    *_TotalBufferLength = *_FirstBufferLength;

    while !Buffer.is_null() {
        *_TotalBufferLength = (*_TotalBufferLength).wrapping_add(MmGetMdlByteCount!(Buffer as PMDL));
        Buffer = (*Buffer).Next;
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisQueryBuffer(Buffer: PNDIS_BUFFER, VirtualAddress: *mut PVOID, Length: PUINT) {
    if !VirtualAddress.is_null() {
        *(VirtualAddress as *mut PVOID) = MmGetSystemAddressForMdl!(Buffer as PMDL);
    }

    *Length = MmGetMdlByteCount!(Buffer as PMDL);
}
#[no_mangle]
pub unsafe extern "system" fn NdisQueryBufferSafe(
    Buffer: PNDIS_BUFFER,
    VirtualAddress: *mut PVOID,
    Length: PUINT,
    Priority: UINT,
) {
    if !VirtualAddress.is_null() {
        *VirtualAddress = MmGetSystemAddressForMdlSafe(Buffer as PMDL, Priority as ULONG);
    }
    *Length = MmGetMdlByteCount!(Buffer as PMDL);
}
#[no_mangle]
pub unsafe extern "system" fn NdisQueryBufferOffset(
    Buffer: PNDIS_BUFFER,
    Offset: PUINT,
    Length: PUINT,
) {
    *((Offset) as PUINT) = MmGetMdlByteOffset!(Buffer);
    *((Length) as PUINT) = MmGetMdlByteCount!(Buffer);
}
#[no_mangle]
pub unsafe extern "system" fn NdisUnchainBufferAtBack(
    Packet: PNDIS_PACKET,
    Buffer: *mut PNDIS_BUFFER,
) {
    let mut NdisBuffer: PNDIS_BUFFER = core::ptr::null_mut();
    let mut Previous: PNDIS_BUFFER = core::ptr::null_mut();
    NdisQueryPacket(
        Packet,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(NdisBuffer),
        core::ptr::null_mut(),
    );
    if NdisBuffer.is_null() {
        NDIS_DbgPrint!(
            MID_TRACE,
            b"No buffer to unchain\n\0".as_ptr() as *const i8,
        );
        *Buffer = core::ptr::null_mut();
        return;
    }
    Previous = core::ptr::null_mut();
    while !(*NdisBuffer).Next.is_null() {
        Previous = NdisBuffer;
        NdisBuffer = (*NdisBuffer).Next as PNDIS_BUFFER;
    }
    if !Previous.is_null() {
        (*Previous).Next = core::ptr::null_mut();
        (*Packet).Private.Tail = Previous;
    } else {
        (*Packet).Private.Head = core::ptr::null_mut();
        (*Packet).Private.Tail = core::ptr::null_mut();
    }
    (*Packet).Private.ValidCounts = FALSE;
    *Buffer = NdisBuffer;
}
#[no_mangle]
pub unsafe extern "system" fn NdisUnchainBufferAtFront(Packet: PNDIS_PACKET, Buffer: *mut PNDIS_BUFFER) {
    let mut NdisBuffer: PNDIS_BUFFER = core::ptr::null_mut();
    NdisQueryPacket(
        Packet,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(NdisBuffer),
        core::ptr::null_mut(),
    );
    if NdisBuffer.is_null() {
        NDIS_DbgPrint!(MID_TRACE, b"No buffer to unchain\n\0".as_ptr() as *const i8);
        *Buffer = core::ptr::null_mut();
        return;
    }

    (*Packet).Private.Head = (*NdisBuffer).Next;

    if (*NdisBuffer).Next.is_null() {
        (*Packet).Private.Tail = core::ptr::null_mut();
    }

    (*NdisBuffer).Next = core::ptr::null_mut();

    (*Packet).Private.ValidCounts = FALSE;

    *Buffer = NdisBuffer;
}
#[no_mangle]
pub unsafe extern "system" fn NdisCopyBuffer(
    Status: PNDIS_STATUS,
    Buffer: *mut PNDIS_BUFFER,
    PoolHandle: NDIS_HANDLE,
    MemoryDescriptor: PVOID,
    Offset: UINT,
    Length: UINT,
) {
    let mut CurrentVa: PVOID = (MmGetMdlVirtualAddress!(MemoryDescriptor as PNDIS_BUFFER) as PUCHAR)
        .add(Offset as usize) as PVOID;

    NDIS_DbgPrint!(MAX_TRACE, b"Called\n\0".as_ptr() as PCSTR);

    *Buffer = IoAllocateMdl(
        CurrentVa,
        Length as ULONG,
        FALSE,
        FALSE,
        core::ptr::null_mut(),
    ) as PNDIS_BUFFER;
    if (*Buffer).is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"IoAllocateMdl failed (%x, %lx)\n\0".as_ptr() as PCSTR,
            CurrentVa,
            Length,
        );
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    IoBuildPartialMdl(
        MemoryDescriptor as PMDL,
        *Buffer as PMDL,
        CurrentVa,
        Length as ULONG,
    );

    (**Buffer).Next = core::ptr::null_mut();
    *Status = NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisGetPoolFromPacket(Packet: PNDIS_PACKET) -> NDIS_HANDLE {
    return (*Packet).Private.Pool as NDIS_HANDLE;
}
#[no_mangle]
pub unsafe extern "system" fn NdisPacketSize(ProtocolReservedSize: UINT) -> UINT {
    return (core::mem::size_of::<NDIS_PACKET>() as UINT)
        .wrapping_add(core::mem::size_of::<NDIS_PACKET_OOB_DATA>() as UINT)
        .wrapping_add(core::mem::size_of::<NDIS_PACKET_EXTENSION>() as UINT)
        .wrapping_add(ProtocolReservedSize);
}

#[no_mangle]
pub unsafe extern "system" fn NdisGetPacketCancelId(Packet: PNDIS_PACKET) -> PVOID {
    NDIS_GET_PACKET_CANCEL_ID!(Packet)
}

#[no_mangle]
pub unsafe extern "system" fn NdisSetPacketCancelId(Packet: PNDIS_PACKET, CancelId: PVOID) {
    NDIS_SET_PACKET_CANCEL_ID!(Packet, CancelId);
}
#[no_mangle]
pub unsafe extern "system" fn NdisCopyFromPacketToPacketSafe(
    Destination: PNDIS_PACKET,
    DestinationOffset: UINT,
    mut BytesToCopy: UINT,
    Source: PNDIS_PACKET,
    SourceOffset: UINT,
    BytesCopied: PUINT,
    Priority: MM_PAGE_PRIORITY,
) {
    let mut SrcBuffer: PNDIS_BUFFER = core::ptr::null_mut();
    let mut DstBuffer: PNDIS_BUFFER = core::ptr::null_mut();
    let mut DstData: PUCHAR = core::ptr::null_mut();
    let mut SrcData: PUCHAR = core::ptr::null_mut();
    let mut DstSize: UINT = 0;
    let mut SrcSize: UINT = 0;
    let mut Count: UINT = 0;
    let mut Total: UINT = 0;

    *BytesCopied = 0;

    /* Skip DestinationOffset bytes in the destination packet */
    NdisGetFirstBufferFromPacketSafe(
        Destination,
        &raw mut DstBuffer,
        &raw mut DstData as *mut PVOID,
        &raw mut DstSize,
        &raw mut Total,
        Priority,
    );
    if DstData.is_null()
        || SkipToOffset(
            DstBuffer,
            DestinationOffset,
            &raw mut DstData,
            &raw mut DstSize,
        ) == 0xFFFFFFFF
    {
        return;
    }

    /* Skip SourceOffset bytes in the source packet */
    NdisGetFirstBufferFromPacketSafe(
        Source,
        &raw mut SrcBuffer,
        &raw mut SrcData as *mut PVOID,
        &raw mut SrcSize,
        &raw mut Total,
        Priority,
    );
    if SrcData.is_null()
        || SkipToOffset(
            SrcBuffer,
            SourceOffset,
            &raw mut SrcData,
            &raw mut SrcSize,
        ) == 0xFFFFFFFF
    {
        return;
    }

    /* Copy the data */
    Total = 0;
    loop {
        /* Find out how many bytes we can copy at one time */
        if BytesToCopy < SrcSize {
            Count = BytesToCopy;
        } else {
            Count = SrcSize;
        }
        if DstSize < Count {
            Count = DstSize;
        }

        RtlCopyMemory!(
            DstData as *mut core::ffi::c_void,
            SrcData as *const core::ffi::c_void,
            Count as usize,
        );

        Total = Total.wrapping_add(Count);
        BytesToCopy = BytesToCopy.wrapping_sub(Count);
        if BytesToCopy == 0 {
            break;
        }

        DstSize = DstSize.wrapping_sub(Count);
        if DstSize == 0 {
            /* No more bytes in destination buffer. Proceed to
               the next buffer in the destination buffer chain */
            NdisGetNextBuffer!(DstBuffer, &raw mut DstBuffer);
            if DstBuffer.is_null() {
                break;
            }

            NdisQueryBufferSafe(
                DstBuffer,
                &raw mut DstData as PVOID as *mut PVOID,
                &raw mut DstSize,
                Priority,
            );
            if DstData.is_null() {
                break;
            }
        }

        SrcSize = SrcSize.wrapping_sub(Count);
        if SrcSize == 0 {
            /* No more bytes in source buffer. Proceed to
               the next buffer in the source buffer chain */
            NdisGetNextBuffer!(SrcBuffer, &raw mut SrcBuffer);
            if SrcBuffer.is_null() {
                break;
            }

            NdisQueryBufferSafe(
                SrcBuffer,
                &raw mut SrcData as PVOID as *mut PVOID,
                &raw mut SrcSize,
                Priority,
            );
            if SrcData.is_null() {
                break;
            }
        }
    }

    *BytesCopied = Total;
}
#[no_mangle]
pub unsafe extern "system" fn NdisIMCopySendCompletePerPacketInfo(
    mut DstPacket: PNDIS_PACKET,
    mut SrcPacket: PNDIS_PACKET,
) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    RtlCopyMemory!(
        NDIS_PACKET_EXTENSION_FROM_PACKET!(DstPacket) as *mut core::ffi::c_void,
        NDIS_PACKET_EXTENSION_FROM_PACKET!(SrcPacket) as *const core::ffi::c_void,
        core::mem::size_of::<NDIS_PACKET_EXTENSION>(),
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisIMCopySendPerPacketInfo(
    DstPacket: PNDIS_PACKET,
    SrcPacket: PNDIS_PACKET,
) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    RtlCopyMemory!(
        NDIS_PACKET_EXTENSION_FROM_PACKET!(DstPacket) as *mut core::ffi::c_void,
        NDIS_PACKET_EXTENSION_FROM_PACKET!(SrcPacket) as *const core::ffi::c_void,
        core::mem::size_of::<NDIS_PACKET_EXTENSION>(),
    );
}
