use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PIMAGE_NT_HEADERS};
use core::ffi::c_void;

// libsupp.c:23 `PVOID MmHighestUserAddress = (PVOID)MI_HIGHEST_USER_ADDRESS;`
// — C global data (libsupp.c is the canonical definer); rustconv dropped
// the non-fn definition. ndk/<arch>/mmtypes.h MI_HIGHEST_USER_ADDRESS:
//   i386 0x7FFEFFFF ; amd64 0x000007FFFFFEFFFF ; arm64 0x00007FFFFFFFF000
#[cfg(target_arch = "x86")]
#[no_mangle]
pub static mut MmHighestUserAddress: PVOID = 0x7FFE_FFFF as PVOID;
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub static mut MmHighestUserAddress: PVOID = 0x0000_07FF_FFFE_FFFF as PVOID;
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut MmHighestUserAddress: PVOID = 0x0000_7FFF_FFFF_F000 as PVOID;

extern "C" {
    fn FrLdrHeapAlloc(Bytes: ULONG, Tag: ULONG) -> PVOID;
}
extern "system" {
    fn FrLdrHeapFree(Mem: PVOID, Tag: ULONG);
    fn RtlpImageNtHeaderEx( Flags: ULONG, Base: PVOID, Size: ULONG64, OutHeaders: *mut PIMAGE_NT_HEADERS, ) -> NTSTATUS;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn CHECK_PAGED_CODE_RTL(file: *mut i8, line: i32) {
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn CHECK_PAGED_CODE_RTL(file: *mut i8, line: i32) {
}

#[no_mangle]
pub unsafe extern "C" fn RtlpAllocateMemory(
    Bytes: ULONG,
    Tag: ULONG,
) -> PVOID {
    FrLdrHeapAlloc(Bytes, Tag)
}

#[no_mangle]
pub unsafe extern "C" fn RtlpFreeMemory(
    Mem: PVOID,
    Tag: ULONG,
) {
    FrLdrHeapFree(Mem, Tag);
}

#[no_mangle]
pub unsafe extern "system" fn RtlImageNtHeaderEx(
    Flags: ULONG,
    Base: PVOID,
    Size: ULONG64,
    OutHeaders: *mut PIMAGE_NT_HEADERS,
) -> NTSTATUS {
    RtlpImageNtHeaderEx(Flags, Base, Size, OutHeaders)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 1998-2003 Brian Palmer   <brianp@sginet.com>
//  *  Copyright (C) 2006      Aleksey Bragin <aleksey@reactos.org>
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
//
// PVOID MmHighestUserAddress = (PVOID)MI_HIGHEST_USER_ADDRESS;
//
// #if DBG
// VOID FASTCALL
// CHECK_PAGED_CODE_RTL(char *file, int line)
// {
//     // boot-code is always ok
// }
// #endif
//
// PVOID
// NTAPI
// RtlpAllocateMemory(ULONG Bytes,
//                    ULONG Tag)
// {
//     return FrLdrHeapAlloc(Bytes, Tag);
// }
//
//
// VOID
// NTAPI
// RtlpFreeMemory(PVOID Mem,
//                ULONG Tag)
// {
//     FrLdrHeapFree(Mem, Tag);
// }
//
// NTSTATUS
// NTAPI
// RtlpSafeCopyMemory(
//    _Out_writes_bytes_all_(Length) VOID UNALIGNED *Destination,
//    _In_reads_bytes_(Length) CONST VOID UNALIGNED *Source,
//    _In_ SIZE_T Length)
// {
//     RtlCopyMemory(Destination, Source, Length);
//     return STATUS_SUCCESS;
// }
//
// /* Ldr access to IMAGE_NT_HEADERS without SEH */
//
// /* Rtl SEH-Free version of this */
// NTSTATUS
// NTAPI
// RtlpImageNtHeaderEx(
//     _In_ ULONG Flags,
//     _In_ PVOID Base,
//     _In_ ULONG64 Size,
//     _Out_ PIMAGE_NT_HEADERS *OutHeaders);
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// RtlImageNtHeaderEx(
//     _In_ ULONG Flags,
//     _In_ PVOID Base,
//     _In_ ULONG64 Size,
//     _Out_ PIMAGE_NT_HEADERS *OutHeaders)
// {
//     return RtlpImageNtHeaderEx(Flags, Base, Size, OutHeaders);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
