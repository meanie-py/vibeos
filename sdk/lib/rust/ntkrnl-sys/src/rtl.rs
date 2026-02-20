//! RTL (Run-Time Library) extern declarations.

use crate::base::*;
use crate::mm::PRTL_BITMAP;

extern "system" {
    pub fn RtlRaiseStatus(Status: NTSTATUS) -> !;

    pub fn RtlZeroMemory(Destination: PVOID, Length: SIZE_T);

    pub fn RtlInitializeBitMap(
        BitMapHeader: PRTL_BITMAP,
        BitMapBuffer: *mut ULONG,
        SizeOfBitMap: ULONG,
    );

    pub fn RtlFindClearBitsAndSet(
        BitMapHeader: PRTL_BITMAP,
        NumberToFind: ULONG,
        HintIndex: ULONG,
    ) -> ULONG;

    pub fn RtlSetBit(BitMapHeader: PRTL_BITMAP, BitNumber: ULONG);

    pub fn RtlClearBit(BitMapHeader: PRTL_BITMAP, BitNumber: ULONG);

    pub fn RtlClearBits(
        BitMapHeader: PRTL_BITMAP,
        StartingIndex: ULONG,
        NumberToClear: ULONG,
    );

    pub fn RtlTestBit(BitMapHeader: PRTL_BITMAP, BitNumber: ULONG) -> BOOLEAN;

    pub fn RtlFindSetBitsAndClear(
        BitMapHeader: PRTL_BITMAP,
        NumberToFind: ULONG,
        HintIndex: ULONG,
    ) -> ULONG;

    pub fn RtlSetAllBits(BitMapHeader: PRTL_BITMAP);

    pub fn RtlClearAllBits(BitMapHeader: PRTL_BITMAP);
}

/// RtlCopyMemory — macro in C (`#define RtlCopyMemory memcpy`).
/// Inline implementation using core::ptr::copy_nonoverlapping.
#[inline]
pub unsafe fn RtlCopyMemory(destination: PVOID, source: PCVOID, length: SIZE_T) {
    core::ptr::copy_nonoverlapping(source as *const u8, destination as *mut u8, length);
}

/// InitializeSListHead — zero out SLIST_HEADER.
#[inline]
pub unsafe fn InitializeSListHead(header: *mut crate::base::SLIST_HEADER) {
    core::ptr::write_bytes(header as *mut u8, 0, core::mem::size_of::<crate::base::SLIST_HEADER>());
}
