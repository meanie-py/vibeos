#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_FILESYSTEM; // pxe.c: DBG_DEFAULT_CHANNEL(FILESYSTEM)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, FILEINFORMATION, LARGE_INTEGER, PCSTR, UINT16};
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_ARC_CODES_EACCES as EACCES;
pub use ntkrnl_sys::ddk::_ARC_CODES_EBADF as EBADF;
pub use ntkrnl_sys::ddk::_ARC_CODES_EINVAL as EINVAL;
pub use ntkrnl_sys::ddk::_ARC_CODES_EIO as EIO;
pub use ntkrnl_sys::ddk::_ARC_CODES_ENOENT as ENOENT;
pub use ntkrnl_sys::ddk::_ARC_CODES_ESUCCESS as ESUCCESS;
pub use ntkrnl_sys::ddk::_KTHREAD_STATE_Initialized as Initialized;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_NetworkPeripheral as NetworkPeripheral;
pub use ntkrnl_sys::ddk::_OPENMODE_OpenReadOnly as OpenReadOnly;
pub use ntkrnl_sys::ddk::_FILEATTRIBUTES_ReadOnlyFile as ReadOnlyFile;
pub use ntkrnl_sys::ddk::_SEEKMODE_SeekAbsolute as SeekAbsolute;
// (rustconv spuriously imported the SECURITY_LOGON_TYPE `Service` enum variant;
// pxe.c never uses it, and as a unit const it turned the `Service` parameter of
// CallPxe into a constant-pattern (u32) instead of a binding. Not dropped content
// — erroneous translator-added import; removed to restore the parameter binding.)
// pub use ntkrnl_sys::ddk::_SECURITY_LOGON_TYPE_Service as Service;

extern "C" {
    fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> i32;
    fn StallExecutionProcessor(microseconds: ULONG);
    fn strlen(s: *const i8) -> usize;
    fn tolower(c: i32) -> i32;
    fn FsRegisterDevice(name: PCHAR, vtbl: PVOID);
}

// PxeCallApi: per-binary-owner split. BIOS freeldr: defined by
// arch/i386/entry.S (`_PxeCallApi`, realmode thunk, __cdecl) — extern.
// uefildr: no realmode / entry.S, so PXE network boot is unavailable;
// fs.rs's FileSystems table still references PxeMount/PxeInit so the
// symbols must exist — provide an inert UEFI stub that fails every PXE
// call (same per-binary-owner pattern as FrldrBootDrive). __cdecl.
#[cfg(not(feature = "uefi"))]
extern "C" {
    fn PxeCallApi(segment: UINT16, offset: UINT16, service: UINT16, parameter: PVOID) -> PXENV_EXIT;
}
#[cfg(feature = "uefi")]
unsafe extern "C" fn PxeCallApi(
    _segment: UINT16,
    _offset: UINT16,
    _service: UINT16,
    _parameter: PVOID,
) -> PXENV_EXIT {
    PXENV_EXIT_FAILURE
}

// ---- rustconv dropped pxe.h types + pxe.c file-local content, restored faithfully ----

pub type OFF16 = u16;
pub type PXENV_EXIT = u16;
pub type PXENV_STATUS = u16;
pub type SEGSEL = u16;
pub type UDP_PORT = u16;
pub type ADDR32 = u32;

#[repr(C)]
#[derive(Clone, Copy)]
pub union IP4 {
    pub num: u32,
    pub array: [u8; 4],
}

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct SEGDESC {
    pub segment_address: u16,
    pub physical_address: u32,
    pub seg_size: u16,
}

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct SEGOFF16 {
    pub offset: OFF16,
    pub segment: SEGSEL,
}

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct PXE {
    pub Signature: [u8; 4],
    pub StructLength: u8,
    pub StructCksum: u8,
    pub StructRev: u8,
    pub reserved1: u8,
    pub UNDIROMID: SEGOFF16,
    pub BaseROMID: SEGOFF16,
    pub EntryPointSP: SEGOFF16,
    pub EntryPointESP: SEGOFF16,
    pub StatusCallout: SEGOFF16,
    pub reserved2: u8,
    pub SegDescCnt: u8,
    pub FirstSelector: SEGSEL,
    pub Stack: SEGDESC,
    pub UNDIData: SEGDESC,
    pub UNDICode: SEGDESC,
    pub UNDICodeWrite: SEGDESC,
    pub BC_Data: SEGDESC,
    pub BC_Code: SEGDESC,
    pub BC_CodeWrite: SEGDESC,
}
pub type PPXE = *mut PXE;

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct t_PXENV_TFTP_OPEN {
    pub Status: PXENV_STATUS,
    pub ServerIPAddress: IP4,
    pub GatewayIPAddress: IP4,
    pub FileName: [u8; 128],
    pub TFTPPort: UDP_PORT,
    pub PacketSize: u16,
}

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct t_PXENV_TFTP_CLOSE {
    pub Status: PXENV_STATUS,
}

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct t_PXENV_TFTP_READ {
    pub Status: PXENV_STATUS,
    pub PacketNumber: u16,
    pub BufferSize: u16,
    pub Buffer: SEGOFF16,
}

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct t_PXENV_TFTP_GET_FSIZE {
    pub Status: PXENV_STATUS,
    pub ServerIPAddress: IP4,
    pub GatewayIPAddress: IP4,
    pub FileName: [u8; 128],
    pub FileSize: u32,
}

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct t_PXENV_GET_CACHED_INFO {
    pub Status: PXENV_STATUS,
    pub PacketType: u16,
    pub BufferSize: u16,
    pub Buffer: SEGOFF16,
    pub BufferLimit: u16,
}

pub const PXENV_EXIT_SUCCESS: PXENV_EXIT = 0x0000;
pub const PXENV_EXIT_FAILURE: PXENV_EXIT = 0x0001;
pub const PXENV_STATUS_SUCCESS: PXENV_STATUS = 0x00;
pub const PXENV_PACKET_TYPE_CACHED_REPLY: u16 = 3;
pub const PXENV_TFTP_OPEN: u16 = 0x20;
pub const PXENV_TFTP_CLOSE: u16 = 0x21;
pub const PXENV_TFTP_READ: u16 = 0x22;
pub const PXENV_TFTP_GET_FSIZE: u16 = 0x25;
pub const PXENV_GET_CACHED_INFO: u16 = 0x71;

// pxe.c file-local: #define NO_FILE ((ULONG)-1); #define TAG_PXE_FILE 'FexP'
pub const NO_FILE: ULONG = ULONG::MAX;
pub const TAG_PXE_FILE: ULONG = u32::from_be_bytes(*b"FexP");

static mut _ServerIP: IP4 = IP4 { num: 0 };
static mut _OpenFile: ULONG = NO_FILE;
static mut _OpenFileName: [CHAR; 128] = [0; 128];
static mut _FileSize: ULONG = 0;
static mut _FilePosition: ULONG = 0;
static mut _PacketPosition: ULONG = 0;
static mut _Packet: [UCHAR; 1024] = [0; 1024];
static mut _CachedLength: ULONG = 0;

unsafe fn FindPxeStructure() -> PPXE {

    let mut Ptr: PPXE = 0xA0000 as PPXE;
    while (Ptr as ULONG_PTR) > 0x10000 {
        Ptr = ((Ptr as ULONG_PTR) - 0x10) as PPXE;

        if memcmp(Ptr as *const core::ffi::c_void, b"!PXE".as_ptr() as *const core::ffi::c_void, 4) != 0 {
            continue;
        }

        if (*Ptr).StructLength != core::mem::size_of::<PXE>() as u8 {
            continue;
        }

        let mut Checksum: UCHAR = 0;
        let mut i: UCHAR = 0;
        while i < (*Ptr).StructLength {
            Checksum = Checksum.wrapping_add(*((Ptr as PUCHAR).add(i as usize)));
            i += 1;
        }
        if Checksum != 0 {
            continue;
        }

        TRACE!("!PXE structure found at %p\n", (Ptr) as usize);
        return Ptr;
    }

    core::ptr::null_mut()
}

unsafe fn GetPxeStructure() -> PPXE {
    static mut pPxe: PPXE = core::ptr::null_mut();
    static mut bPxeSearched: BOOLEAN = 0u8;
    if bPxeSearched == 0u8 {
        pPxe = FindPxeStructure();
        bPxeSearched = 1u8;
    }
    pPxe
}

unsafe fn CallPxe(Service: UINT16, Parameter: PVOID) -> BOOLEAN {

    let pxe = GetPxeStructure();
    if pxe.is_null() {
        return 0u8;
    }

    if Service != PXENV_TFTP_READ {
        StallExecutionProcessor(100 * 1000);
        TRACE!("PxeCall(0x%x, %p)\n", (Service) as usize, (Parameter) as usize);
    }

    let exit = PxeCallApi((*pxe).EntryPointSP.segment, (*pxe).EntryPointSP.offset, (Service) as u16 , Parameter);
    if exit != PXENV_EXIT_SUCCESS {
        ERR!("PxeCall(0x%x, %p) failed with exit=%d status=0x%x\n", (Service) as usize, (Parameter) as usize, (exit) as usize, (*(Parameter as *const PXENV_STATUS)) as usize);
        return 0u8;
    }
    if *(Parameter as *const PXENV_STATUS) != PXENV_STATUS_SUCCESS {
        ERR!("PxeCall(0x%x, %p) succeeded, but returned error status 0x%x\n", (Service) as usize, (Parameter) as usize, (*(Parameter as *const PXENV_STATUS)) as usize);
        return 0u8;
    }
    1u8
}

unsafe extern "C" fn PxeClose(FileId: ULONG) -> ARC_STATUS {
    let mut closeData: t_PXENV_TFTP_CLOSE = core::mem::zeroed();

    if _OpenFile == NO_FILE || FileId != _OpenFile {
        return EBADF;
    }

    core::ptr::write_bytes(&mut closeData as *mut _ as *mut u8, 0, core::mem::size_of::<t_PXENV_TFTP_CLOSE>());
    if CallPxe(PXENV_TFTP_CLOSE, &mut closeData as *mut _ as PVOID) == 0 {
        return EIO;
    }

    _OpenFile = NO_FILE;
    return ESUCCESS;
}

unsafe extern "C" fn PxeGetFileInformation(FileId: u32, Information: *mut FILEINFORMATION) -> ARC_STATUS {
    let mut FileName: PCSTR;

    if _OpenFile == NO_FILE || FileId != _OpenFile {
        return EBADF;
    }

    core::ptr::write_bytes(Information, 0, 1);
    (*Information).EndingAddress.__bindgen_anon_1.LowPart = _FileSize;
    (*Information).CurrentAddress.__bindgen_anon_1.LowPart = _FilePosition;

    (*Information).Type = NetworkPeripheral;

    (*Information).Attributes = (ReadOnlyFile) as u8 ;

    FileName = strrchr(_OpenFileName.as_ptr(), b'/' as i32);
    if !FileName.is_null() {
        FileName = FileName.add(1);
    } else {
        FileName = _OpenFileName.as_ptr();
    }

    (*Information).FileNameLength = strlen(FileName) as u32;
    (*Information).FileNameLength = min((*Information).FileNameLength, (core::mem::size_of_val(&(*Information).FileName) - 1) as u32);
    core::ptr::copy_nonoverlapping(FileName as *const u8, (*Information).FileName.as_mut_ptr() as *mut u8 , (*Information).FileNameLength as usize);
    *core::ptr::addr_of_mut!((*Information).FileName).cast::<u8>().add((*Information).FileNameLength as usize) = 0;

    TRACE!("PxeGetFileInformation(%lu) -> FileSize = %lu, FilePointer = 0x%lx\n", (FileId) as usize, ((*Information).EndingAddress.__bindgen_anon_1.LowPart) as usize, ((*Information).CurrentAddress.__bindgen_anon_1.LowPart) as usize);

    ESUCCESS
}

unsafe extern "C" fn PxeOpen(Path: PCHAR, OpenMode: OPENMODE, FileId: *mut ULONG) -> ARC_STATUS {

    let mut sizeData: t_PXENV_TFTP_GET_FSIZE = core::mem::zeroed();
    let mut openData: t_PXENV_TFTP_OPEN = core::mem::zeroed();
    let PathLen: usize;
    let mut i: usize;

    if _OpenFile != NO_FILE {
        return EIO;
    }
    if OpenMode != OpenReadOnly {
        return EACCES;
    }

    let mut Path = Path;
    if *Path == b'\\' as i8 || *Path == b'/' as i8 {
        Path = Path.add(1);
    }

    PathLen = strlen(Path as *const i8);
    let PathLen = min(PathLen, core::mem::size_of_val(&_OpenFileName) - 1);

    i = 0;
    while i < PathLen {
        if *Path.add(i) == b'\\' as i8 {
            _OpenFileName[i] = b'/' as i8;
        } else {
            _OpenFileName[i] = tolower(*Path.add(i) as i32) as i8;
        }
        i += 1;
    }

    core::ptr::write_bytes( // nolint: silent-failure (byte array — count==bytes)
        _OpenFileName.as_mut_ptr().add(PathLen),
        0,
        core::mem::size_of_val(&_OpenFileName) - PathLen,
    );

    core::ptr::write_bytes(&mut sizeData as *mut _ as *mut u8, 0, core::mem::size_of::<t_PXENV_TFTP_GET_FSIZE>());
    sizeData.ServerIPAddress = _ServerIP;
    core::ptr::copy_nonoverlapping(
        _OpenFileName.as_ptr(),
        sizeData.FileName.as_mut_ptr() as *mut i8 ,
        core::mem::size_of_val(&_OpenFileName),
    );
    if CallPxe(PXENV_TFTP_GET_FSIZE, &mut sizeData as *mut _ as PVOID) == 0 {
        ERR!("Failed to get '%s' size\n", (Path) as usize);
        return EIO;
    }

    _FileSize = sizeData.FileSize;
    _CachedLength = 0;

    core::ptr::write_bytes(&mut openData as *mut _ as *mut u8, 0, core::mem::size_of::<t_PXENV_TFTP_OPEN>());
    openData.ServerIPAddress = _ServerIP;
    core::ptr::copy_nonoverlapping(
        _OpenFileName.as_ptr(),
        openData.FileName.as_mut_ptr() as *mut i8 ,
        core::mem::size_of_val(&_OpenFileName),
    );
    openData.PacketSize = core::mem::size_of_val(&_Packet) as u16;

    if CallPxe(PXENV_TFTP_OPEN, &mut openData as *mut _ as PVOID) == 0 {
        return ENOENT;
    }

    _FilePosition = 0;
    _PacketPosition = 0;

    _OpenFile = *FileId;
    ESUCCESS
}

unsafe extern "C" fn PxeRead(FileId: ULONG, Buffer: PVOID, N: ULONG, Count: *mut ULONG) -> ARC_STATUS {
    let mut readData: t_PXENV_TFTP_READ = core::mem::zeroed();
    let mut i: ULONG;

    *Count = 0;

    if _OpenFile == NO_FILE || FileId != _OpenFile {
        return EBADF;
    }

    RtlZeroMemory(
        core::ptr::addr_of_mut!(readData) as PVOID,
        core::mem::size_of::<t_PXENV_TFTP_READ>(),
    );
    readData.Buffer.segment = ((_Packet.as_ptr() as ULONG_PTR & 0xf0000) / 16) as UINT16;
    readData.Buffer.offset = (_Packet.as_ptr() as ULONG_PTR & 0xffff) as UINT16;

    let mut N = N;
    let mut Buffer = Buffer;
    while N > 0 {
        if N < _CachedLength - _FilePosition {
            i = N;
        } else {
            i = _CachedLength - _FilePosition;
        }
        RtlCopyMemory(
            Buffer,
            _Packet.as_ptr().offset((_FilePosition - _PacketPosition) as isize) as PVOID,
            i as usize,
        );
        _FilePosition += i;
        Buffer = (Buffer as *mut UCHAR).offset(i as isize) as PVOID;
        *Count += i;
        N -= i;
        if N == 0 {
            break;
        }

        if CallPxe(PXENV_TFTP_READ, core::ptr::addr_of_mut!(readData) as PVOID) == 0 {
            return EIO;
        }
        _PacketPosition = _CachedLength;
        _CachedLength += readData.BufferSize as ULONG;
    }

    ESUCCESS
}

unsafe extern "C" fn PxeSeek(mut FileId: ULONG, Position: *mut LARGE_INTEGER, SeekMode: SEEKMODE) -> ARC_STATUS {
    let mut readData: t_PXENV_TFTP_READ = core::mem::zeroed();

    if _OpenFile == NO_FILE || FileId != _OpenFile {
        return EBADF;
    }

    if (*Position).__bindgen_anon_1.HighPart != 0 || SeekMode != SeekAbsolute {
        return EINVAL;
    }

    if (*Position).__bindgen_anon_1.LowPart < _FilePosition {
        if PxeClose(FileId) != ESUCCESS {
            return EIO;
        }
        if PxeOpen(_OpenFileName.as_mut_ptr(), OpenReadOnly, &mut FileId as *mut ULONG) != ESUCCESS {
            return EIO;
        }
    }

    core::ptr::write_bytes(&mut readData as *mut _ as *mut u8, 0, core::mem::size_of::<t_PXENV_TFTP_READ>());
    readData.Buffer.segment = ((_Packet.as_ptr() as ULONG_PTR & 0xf0000) / 16) as UINT16;
    readData.Buffer.offset = (_Packet.as_ptr() as ULONG_PTR & 0xffff) as UINT16;

    while (*Position).__bindgen_anon_1.LowPart > _CachedLength {
        if CallPxe(PXENV_TFTP_READ, &mut readData as *mut _ as PVOID) == 0 {
            return EIO;
        }
        _PacketPosition = _CachedLength;
        _CachedLength += readData.BufferSize as ULONG;
    }

    _FilePosition = (*Position).__bindgen_anon_1.LowPart;
    return ESUCCESS;
}

// pxe.c: static const DEVVTBL PxeVtbl = { PxeClose, ... }; (5-field init, no ServiceName)
static PxeVtbl: DEVVTBL = DEVVTBL {
    Close: Some(PxeClose),
    GetFileInformation: Some(PxeGetFileInformation),
    Open: Some(PxeOpen),
    Read: Some(PxeRead),
    Seek: Some(PxeSeek),
    ServiceName: core::ptr::null(),
};

#[no_mangle]
pub unsafe extern "C" fn PxeMount(_DeviceId: ULONG) -> *const DEVVTBL {
    if GetPxeStructure() == core::ptr::null_mut() {
        return core::ptr::null();
    }
    &PxeVtbl
}

unsafe extern "C" fn PxeDiskClose(_FileId: ULONG) -> ARC_STATUS {
    ESUCCESS
}

unsafe extern "C" fn PxeDiskGetFileInformation(_FileId: ULONG, _Information: *mut FILEINFORMATION) -> ARC_STATUS {
    EINVAL
}

unsafe extern "C" fn PxeDiskOpen(_Path: PCHAR, _OpenMode: OPENMODE, _FileId: *mut ULONG) -> ARC_STATUS {
    ESUCCESS
}

unsafe extern "C" fn PxeDiskRead(_FileId: ULONG, _Buffer: *mut VOID, _N: ULONG, _Count: *mut ULONG) -> ARC_STATUS {
    EINVAL
}

unsafe extern "C" fn PxeDiskSeek(_FileId: ULONG, _Position: *mut LARGE_INTEGER, _SeekMode: SEEKMODE) -> ARC_STATUS {
    EINVAL
}

// pxe.c: static const DEVVTBL PxeDiskVtbl = { PxeDiskClose, ... };
static PxeDiskVtbl: DEVVTBL = DEVVTBL {
    Close: Some(PxeDiskClose),
    GetFileInformation: Some(PxeDiskGetFileInformation),
    Open: Some(PxeDiskOpen),
    Read: Some(PxeDiskRead),
    Seek: Some(PxeDiskSeek),
    ServiceName: core::ptr::null(),
};

unsafe fn GetCachedInfo() -> BOOLEAN {
    let mut Data: t_PXENV_GET_CACHED_INFO = core::mem::zeroed();
    Data.PacketType = PXENV_PACKET_TYPE_CACHED_REPLY;

    let res = CallPxe(PXENV_GET_CACHED_INFO, &mut Data as *mut _ as PVOID);
    if res == 0u8 {
        return 0u8;
    }
    if Data.BufferSize < 36 {
        return 0u8;
    }
    let Packet = ((Data.Buffer.segment as ULONG_PTR) << 4).wrapping_add(Data.Buffer.offset as ULONG_PTR) as *mut UCHAR;
    core::ptr::copy_nonoverlapping(Packet.add(20), &mut _ServerIP as *mut _ as *mut u8, core::mem::size_of::<IP4>());
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn PxeInit() -> BOOLEAN {
    static mut Initialized: BOOLEAN = 0u8;
    static mut Success: BOOLEAN = 0u8;

    if Initialized != 0u8 {
        return Success;
    }
    Initialized = 1u8;

    if !GetPxeStructure().is_null() && GetCachedInfo() != 0u8 {
        FsRegisterDevice(b"net(0)\0".as_ptr() as PCHAR, &PxeDiskVtbl as *const _ as PVOID);
        Success = 1u8;
    }

    Success
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 2011  Hervé Poussineau
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
//  *  You should have received a copy of the GNU General Public License
//  *  along with this program; if not, write to the Free Software
//  *  Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.
//  */
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(FILESYSTEM);
//
// #define TAG_PXE_FILE 'FexP'
// #define NO_FILE ((ULONG)-1)
//
// static IP4 _ServerIP = { 0, };
// static ULONG _OpenFile = NO_FILE;
// static CHAR _OpenFileName[128];
// static ULONG _FileSize = 0;
// static ULONG _FilePosition = 0;
// static ULONG _PacketPosition = 0;
// static UCHAR _Packet[1024]; // Should be a value which can be transferred well in one packet over the network
// static ULONG _CachedLength = 0;
//
// static PPXE
// FindPxeStructure(VOID)
// {
//     PPXE Ptr;
//     UCHAR Checksum;
//     UCHAR i;
//
//     /* Find the '!PXE' structure */
//     Ptr = (PPXE)0xA0000;
//     while ((ULONG_PTR)Ptr > 0x10000)
//     {
//         Ptr = (PPXE)((ULONG_PTR)Ptr - 0x10);
//
//         /* Look for signature */
//         if (memcmp(Ptr, "!PXE", 4) != 0)
//             continue;
//
//         /* Check size */
//         if (Ptr->StructLength != sizeof(PXE))
//             continue;
//
//         /* Check checksum */
//         Checksum = 0;
//         for (i = 0; i < Ptr->StructLength; i++)
//             Checksum += *((PUCHAR)Ptr + i);
//         if (Checksum != 0)
//             continue;
//
//         TRACE("!PXE structure found at %p\n", (Ptr) as usize);
//         return Ptr;
//     }
//
//     return NULL;
// }
//
// static PPXE GetPxeStructure(VOID)
// {
//     static PPXE pPxe = NULL;
//     static BOOLEAN bPxeSearched = FALSE;
//     if (!bPxeSearched)
//     {
//         pPxe = FindPxeStructure();
//         bPxeSearched = TRUE;
//     }
//     return pPxe;
// }
//
// extern PXENV_EXIT __cdecl PxeCallApi(UINT16 Segment, UINT16 Offset, UINT16 Service, VOID *Parameter);
// BOOLEAN CallPxe(UINT16 Service, PVOID Parameter)
// {
//     PPXE pxe;
//     PXENV_EXIT exit;
//
//     pxe = GetPxeStructure();
//     if (!pxe)
//         return FALSE;
//
//     if (Service != PXENV_TFTP_READ)
//     {
//         // HACK: this delay shouldn't be necessary
//         StallExecutionProcessor(100 * 1000); // 100 ms
//         TRACE("PxeCall(0x%x, %p)\n", (Service) as usize, (Parameter) as usize);
//     }
//
//     exit = PxeCallApi(pxe->EntryPointSP.segment, pxe->EntryPointSP.offset, Service, Parameter);
//     if (exit != PXENV_EXIT_SUCCESS)
//     {
//         ERR("PxeCall(0x%x, %p) failed with exit=%d status=0x%x\n", (//                 Service) as usize, (Parameter) as usize, (exit) as usize, (*(PXENV_STATUS*)Parameter) as usize);
//         return FALSE;
//     }
//     if (*(PXENV_STATUS*)Parameter != PXENV_STATUS_SUCCESS)
//     {
//         ERR("PxeCall(0x%x, %p) succeeded, but returned error status 0x%x\n", (//                 Service) as usize, (Parameter) as usize, (*(PXENV_STATUS*)Parameter) as usize);
//         return FALSE;
//     }
//     return TRUE;
// }
//
// static ARC_STATUS PxeClose(ULONG FileId)
// {
//     t_PXENV_TFTP_CLOSE closeData;
//
//     if (_OpenFile == NO_FILE || FileId != _OpenFile)
//         return EBADF;
//
//     RtlZeroMemory(&closeData, sizeof(closeData));
//     if (!CallPxe(PXENV_TFTP_CLOSE, &closeData))
//         return EIO;
//
//     _OpenFile = NO_FILE;
//     return ESUCCESS;
// }
//
// static ARC_STATUS PxeGetFileInformation(ULONG FileId, FILEINFORMATION* Information)
// {
//     PCSTR FileName;
//
//     if (_OpenFile == NO_FILE || FileId != _OpenFile)
//         return EBADF;
//
//     RtlZeroMemory(Information, sizeof(*Information));
//     Information->EndingAddress.LowPart = _FileSize;
//     Information->CurrentAddress.LowPart = _FilePosition;
//
//     Information->Type = NetworkPeripheral;
//
//     /* Set the ARC file attributes */
//     Information->Attributes = ReadOnlyFile;
//
//     /* Search for the last path separator. Slashes are used as separators,
//      * for supporting TFTP servers on POSIX systems (see PxeOpen()) */
//     FileName = strrchr(_OpenFileName, '/');
//     if (FileName)
//         ++FileName; // Go past it.
//     else
//         FileName = _OpenFileName; // No separator: file name without directory.
//
//     /* Copy the file name, perhaps truncated, and NUL-terminated */
//     Information->FileNameLength = (ULONG)strlen(FileName);
//     Information->FileNameLength = min(Information->FileNameLength, sizeof(Information->FileName) - 1);
//     RtlCopyMemory(Information->FileName, FileName, Information->FileNameLength);
//     Information->FileName[Information->FileNameLength] = ANSI_NULL;
//
//     TRACE("PxeGetFileInformation(%lu) -> FileSize = %lu, FilePointer = 0x%lx\n", (//           FileId) as usize, (Information->EndingAddress.LowPart) as usize, (Information->CurrentAddress.LowPart) as usize);
//
//     return ESUCCESS;
// }
//
// static ARC_STATUS PxeOpen(CHAR* Path, OPENMODE OpenMode, ULONG* FileId)
// {
//     t_PXENV_TFTP_GET_FSIZE sizeData;
//     t_PXENV_TFTP_OPEN openData;
//     SIZE_T PathLen, i;
//
//     if (_OpenFile != NO_FILE)
//         return EIO;
//     if (OpenMode != OpenReadOnly)
//         return EACCES;
//
//     /* Skip leading path separator, if any, so as to ensure that
//      * we always lookup the file at the root of the TFTP server's
//      * file space ("virtual root"), even if the server doesn't
//      * support this, and NOT from the root of the file system. */
//     if (*Path == '\\' || *Path == '/')
//         ++Path;
//
//     /* Retrieve the path length without NULL terminator */
//     PathLen = strlen(Path);
//     PathLen = min(PathLen, sizeof(_OpenFileName) - 1);
//
//     /* Lowercase the path and always use slashes as separators,
//      * for supporting TFTP servers on POSIX systems */
//     for (i = 0; i < PathLen; i++)
//     {
//         if (Path[i] == '\\')
//             _OpenFileName[i] = '/';
//         else
//             _OpenFileName[i] = tolower(Path[i]);
//     }
//
//     /* Zero out rest of the file name */
//     RtlZeroMemory(_OpenFileName + PathLen, sizeof(_OpenFileName) - PathLen);
//
//     RtlZeroMemory(&sizeData, sizeof(sizeData));
//     sizeData.ServerIPAddress = _ServerIP;
//     RtlCopyMemory(sizeData.FileName, _OpenFileName, sizeof(_OpenFileName));
//     if (!CallPxe(PXENV_TFTP_GET_FSIZE, &sizeData))
//     {
//         ERR("Failed to get '%s' size\n", (Path) as usize);
//         return EIO;
//     }
//
//     _FileSize = sizeData.FileSize;
//     _CachedLength = 0;
//
//     RtlZeroMemory(&openData, sizeof(openData));
//     openData.ServerIPAddress = _ServerIP;
//     RtlCopyMemory(openData.FileName, _OpenFileName, sizeof(_OpenFileName));
//     openData.PacketSize = sizeof(_Packet);
//
//     if (!CallPxe(PXENV_TFTP_OPEN, &openData))
//         return ENOENT;
//
//     _FilePosition = 0;
//     _PacketPosition = 0;
//
//     _OpenFile = *FileId;
//     return ESUCCESS;
// }
//
// static ARC_STATUS PxeRead(ULONG FileId, VOID* Buffer, ULONG N, ULONG* Count)
// {
//     t_PXENV_TFTP_READ readData;
//     ULONG i;
//
//     *Count = 0;
//
//     if (_OpenFile == NO_FILE || FileId != _OpenFile)
//         return EBADF;
//
//     RtlZeroMemory(&readData, sizeof(readData));
//     readData.Buffer.segment = ((ULONG_PTR)_Packet & 0xf0000) / 16;
//     readData.Buffer.offset = (ULONG_PTR)_Packet & 0xffff;
//
//     // Get new packets as required
//     while (N > 0)
//     {
//         if (N < _CachedLength - _FilePosition)
//             i = N;
//         else
//             i = _CachedLength - _FilePosition;
//         RtlCopyMemory(Buffer, _Packet + _FilePosition - _PacketPosition, i);
//         _FilePosition += i;
//         Buffer = (UCHAR*)Buffer + i;
//         *Count += i;
//         N -= i;
//         if (N == 0)
//             break;
//
//         if (!CallPxe(PXENV_TFTP_READ, &readData))
//             return EIO;
//         _PacketPosition = _CachedLength;
//         _CachedLength += readData.BufferSize;
//     }
//
//     return ESUCCESS;
// }
//
// static ARC_STATUS PxeSeek(ULONG FileId, LARGE_INTEGER* Position, SEEKMODE SeekMode)
// {
//     t_PXENV_TFTP_READ readData;
//
//     if (_OpenFile == NO_FILE || FileId != _OpenFile)
//         return EBADF;
//
//     if (Position->HighPart != 0 || SeekMode != SeekAbsolute)
//         return EINVAL;
//
//     if (Position->LowPart < _FilePosition)
//     {
//         // Close and reopen the file to go to position 0
//         if (PxeClose(FileId) != ESUCCESS)
//             return EIO;
//         if (PxeOpen(_OpenFileName.as_mut_ptr(), OpenReadOnly, &FileId) != ESUCCESS)
//             return EIO;
//     }
//
//     RtlZeroMemory(&readData, sizeof(readData));
//     readData.Buffer.segment = ((ULONG_PTR)_Packet & 0xf0000) / 16;
//     readData.Buffer.offset = (ULONG_PTR)_Packet & 0xffff;
//
//     // Get new packets as required
//     while (Position->LowPart > _CachedLength)
//     {
//         if (!CallPxe(PXENV_TFTP_READ, &readData))
//             return EIO;
//         _PacketPosition = _CachedLength;
//         _CachedLength += readData.BufferSize;
//     }
//
//     _FilePosition = Position->LowPart;
//     return ESUCCESS;
// }
//
// static const DEVVTBL PxeVtbl = {
//     PxeClose,
//     PxeGetFileInformation,
//     PxeOpen,
//     PxeRead,
//     PxeSeek,
// };
//
// const DEVVTBL* PxeMount(ULONG DeviceId)
// {
//     if (GetPxeStructure() == NULL)
//         return NULL;
//     return &PxeVtbl;
// }
//
// static ARC_STATUS PxeDiskClose(ULONG FileId)
// {
//     // Nothing to do
//     return ESUCCESS;
// }
//
// static ARC_STATUS PxeDiskGetFileInformation(ULONG FileId, FILEINFORMATION* Information)
// {
//     // No disk access in PXE mode
//     return EINVAL;
// }
//
// static ARC_STATUS PxeDiskOpen(CHAR* Path, OPENMODE OpenMode, ULONG* FileId)
// {
//     // Nothing to do
//     return ESUCCESS;
// }
//
// static ARC_STATUS PxeDiskRead(ULONG FileId, VOID* Buffer, ULONG N, ULONG* Count)
// {
//     // No disk access in PXE mode
//     return EINVAL;
// }
//
// static ARC_STATUS PxeDiskSeek(ULONG FileId, LARGE_INTEGER* Position, SEEKMODE SeekMode)
// {
//     // No disk access in PXE mode
//     return EINVAL;
// }
//
// static const DEVVTBL PxeDiskVtbl = {
//     PxeDiskClose,
//     PxeDiskGetFileInformation,
//     PxeDiskOpen,
//     PxeDiskRead,
//     PxeDiskSeek,
// };
//
// static BOOLEAN GetCachedInfo(VOID)
// {
//     t_PXENV_GET_CACHED_INFO Data;
//     BOOLEAN res;
//     UCHAR* Packet;
//
//     RtlZeroMemory(&Data, sizeof(Data));
//     Data.PacketType = PXENV_PACKET_TYPE_CACHED_REPLY;
//
//     res = CallPxe(PXENV_GET_CACHED_INFO, &Data);
//     if (!res)
//         return FALSE;
//     if (Data.BufferSize < 36)
//         return FALSE;
//     Packet = (UCHAR*)((ULONG_PTR)(Data.Buffer.segment << 4) + Data.Buffer.offset);
//     RtlCopyMemory(&_ServerIP, Packet + 20, sizeof(IP4));
//     return TRUE;
// }
//
// BOOLEAN PxeInit(VOID)
// {
//     static BOOLEAN Initialized = FALSE;
//     static BOOLEAN Success = FALSE;
//
//     // Do initialization only once
//     if (Initialized)
//         return Success;
//     Initialized = TRUE;
//
//     // Check if PXE is available
//     if (GetPxeStructure() && GetCachedInfo())
//     {
//         FsRegisterDevice("net(0)", &PxeDiskVtbl);
//         Success = TRUE;
//     }
//
//     return Success;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
