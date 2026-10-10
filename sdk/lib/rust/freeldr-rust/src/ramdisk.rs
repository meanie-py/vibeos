use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, FILEINFORMATION, LARGE_INTEGER, PCSTR};
use core::ffi::c_void;
extern "C" {
    fn ERR(fmt: *const i8, ...);
}

// Cross-module freeldr API ramdisk.c calls (real #[no_mangle] owners:
// fs.rs/arcsupp.rs/mm.rs/ui.rs). File-local extern block — NOT in the
// keystone (would shadow other globs and regress prev-green files).
extern "C" {
    fn FsOpenFile(FileName: PCSTR, DefaultPath: PCSTR, OpenMode: OPENMODE, FileId: *mut ULONG) -> ARC_STATUS;
    fn FsGetVolumeSize(DeviceId: ULONG, VolumeSize: *mut u64) -> ARC_STATUS;
    fn ArcGetFileInformation(FileId: ULONG, Information: *mut FILEINFORMATION) -> ARC_STATUS;
    fn ArcClose(FileId: ULONG) -> ARC_STATUS;
    fn ArcSeek(FileId: ULONG, Position: *mut LARGE_INTEGER, SeekMode: SEEKMODE) -> ARC_STATUS;
    fn ArcRead(FileId: ULONG, Buffer: *mut c_void, N: ULONG, Count: *mut ULONG) -> ARC_STATUS;
    fn MmAllocateMemoryWithType(MemorySize: SIZE_T, MemoryType: TYPE_OF_MEMORY) -> PVOID;
    fn UiDrawProgressBarCenter(ProgressText: PCSTR);
    fn UiUpdateProgressBar(Percentage: ULONG, ProgressText: PCSTR);
    fn UiMessageBox(MessageText: *const i8);
}

// --- ramdisk.c globals + file-local statics (rustconv drops non-fn
// content). gInitRamDiskBase/Size are exported C globals (other freeldr
// code references them) → #[no_mangle]; the rest are file-static. ---
#[no_mangle]
pub static mut gInitRamDiskBase: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut gInitRamDiskSize: ULONG = 0;

static mut RamDiskDeviceRegistered: BOOLEAN = 0;
static mut RamDiskBase: PVOID = core::ptr::null_mut();
static mut RamDiskFileSize: ULONGLONG = 0; // FIXME: limited to 4GB.
static mut RamDiskImageLength: ULONGLONG = 0;
static mut RamDiskImageOffset: ULONG = 0;
static mut RamDiskOffset: ULONGLONG = 0;

// static const DEVVTBL RamDiskVtbl (dropped by rustconv).
static RamDiskVtbl: DEVVTBL = DEVVTBL {
    Close: Some(RamDiskClose),
    GetFileInformation: Some(RamDiskGetFileInformation),
    Open: Some(RamDiskOpen),
    Read: Some(RamDiskRead),
    Seek: Some(RamDiskSeek),
    ServiceName: core::ptr::null(),
};

extern "C" {
    fn TRACE(fmt: *const i8, ...);
    fn MmFreeMemory(ptr: *mut core::ffi::c_void);
    fn NtLdrGetOption(LoadOptions: PCSTR, Option: PCSTR) -> PCSTR;
    fn NtLdrGetOptionEx(LoadOptions: PCSTR, Option: PCSTR, Length: *mut u32) -> PCSTR;
    fn RtlStringCbCopyNA(Destination: PCSTR, DestinationSize: usize, Source: PCSTR, Count: usize) -> i32;
    fn _atoi64(str: PCSTR) -> i64;
    fn atol(str: PCSTR) -> i32;
    fn FsRegisterDevice(DeviceName: PCSTR, DeviceVtbl: *const core::ffi::c_void);
}

unsafe extern "C" fn RamDiskClose(_FileId: ULONG) -> ARC_STATUS {
    ESUCCESS
}

unsafe extern "C" fn RamDiskGetFileInformation(_FileId: ULONG, Information: *mut FILEINFORMATION) -> ARC_STATUS {
    core::ptr::write_bytes(Information, 0, 1);
    (*Information).EndingAddress.QuadPart = (RamDiskImageLength) as i64 ;
    (*Information).CurrentAddress.QuadPart = (RamDiskOffset) as i64 ;
    (*Information).Type = DiskPeripheral;

    ESUCCESS
}

unsafe extern "C" fn RamDiskOpen(_Path: PCHAR, _OpenMode: OPENMODE, _FileId: *mut ULONG) -> ARC_STATUS {
    ESUCCESS
}

unsafe extern "C" fn RamDiskRead(_FileId: ULONG, Buffer: PVOID, N: ULONG, Count: *mut ULONG) -> ARC_STATUS {
    let StartAddress: PVOID;

    if RamDiskOffset >= RamDiskImageLength || RamDiskOffset + ((N) as u64) > RamDiskImageLength {
        *Count = 0;
        return EIO;
    }

    StartAddress = (RamDiskBase as ULONG_PTR + ((RamDiskImageOffset) as usize) + RamDiskOffset as ULONG_PTR) as PVOID;

    RtlCopyMemory(Buffer, StartAddress, N as SIZE_T);
    RamDiskOffset += (N) as u64 ;
    *Count = N;

    return ESUCCESS;
}

unsafe extern "C" fn RamDiskSeek(_FileId: ULONG, Position: *mut LARGE_INTEGER, SeekMode: SEEKMODE) -> ARC_STATUS {
    let mut NewPosition: LARGE_INTEGER = *Position;

    match SeekMode {
        SeekAbsolute => {},
        SeekRelative => {
            NewPosition.QuadPart += (RamDiskOffset) as i64 ;
        },
        _ => {
            return EINVAL;
        }
    }

    if NewPosition.QuadPart >= ((RamDiskImageLength) as i64) {
        return EINVAL;
    }

    RamDiskOffset = (NewPosition.QuadPart) as u64 ;
    ESUCCESS
}

unsafe fn RamDiskLoadVirtualFile(
    FileName: PCSTR,
    DefaultPath: PCSTR,
) -> ARC_STATUS
{
    let mut Status: ARC_STATUS;
    let mut RamFileId: u32 = 0;
    let mut ChunkSize: u32;
    let mut Count: u32 = 0;
    let mut TotalRead: u64;
    let PercentPerChunk: u32;
    let mut Percent: u32;
    let mut Information: FILEINFORMATION = core::mem::zeroed();
    let mut Position: LARGE_INTEGER = core::mem::zeroed();

    UiDrawProgressBarCenter(b"Loading RamDisk...\0".as_ptr() as PCSTR);

    TRACE(b"RamDiskLoadVirtualFile: Opening '%s', '%s'\n\0".as_ptr() as PCSTR, (FileName) as usize, (if !DefaultPath.is_null() { DefaultPath } else { b"n/a\0".as_ptr() as PCSTR }) as usize);
    Status = FsOpenFile(FileName, DefaultPath, OpenReadOnly, core::ptr::addr_of_mut!(RamFileId));
    if Status != ESUCCESS {
        return Status;
    }

    Status = ArcGetFileInformation(RamFileId, core::ptr::addr_of_mut!(Information));
    if Status != ESUCCESS {
        ArcClose(RamFileId);
        return Status;
    }

    Information.EndingAddress.QuadPart = Information.EndingAddress.QuadPart - Information.StartingAddress.QuadPart;
    Information.StartingAddress.QuadPart = (0u64) as i64 ;

    if Information.FileNameLength == 0 && *Information.FileName.as_ptr() == 0 &&
        (Information.Type == DiskPeripheral || Information.Type == FloppyDiskPeripheral)
    {
        let mut VolumeSize: u64 = 0;
        Status = FsGetVolumeSize(RamFileId, core::ptr::addr_of_mut!(VolumeSize));
        if Status != ESUCCESS {
            ERR(b"Couldn't retrieve volume size on device '%s', falling back to RAW size\n\0".as_ptr() as PCSTR, (FileName) as usize);
        } else {
            Information.EndingAddress.QuadPart = (VolumeSize) as i64 ;
        }
    }

    TRACE(b"RAMDISK size: %I64u (High: %lu ; Low: %lu)\n\0".as_ptr() as PCSTR, (Information.EndingAddress.QuadPart) as usize, (Information.EndingAddress.u.HighPart) as usize, (Information.EndingAddress.u.LowPart) as usize);

    if Information.EndingAddress.u.HighPart != 0 {
        ArcClose(RamFileId);
        UiMessageBox(b"RAM disk too big.\0".as_ptr() as PCSTR);
        return ENOMEM;
    }
    RamDiskFileSize = (Information.EndingAddress.QuadPart) as u64 ;

    ChunkSize = 8 * 1024 * 1024;
    if RamDiskFileSize < ChunkSize as u64 {
        PercentPerChunk = 0;
    } else {
        PercentPerChunk = 100 * ChunkSize / RamDiskFileSize as u32;
    }
    // i386: take the image from the TOP of memory. Allocated bottom-up (the C), a ~300MB
    // writable-root ramdisk pushed every boot driver loaded after it above it, so the
    // loader span -- the kernel's KSEG0 boot-image reservation -- grew to 456MB and the
    // boot died (0x7F/0x50). The image is reached through MmMapIoSpace of its physical base,
    // so where it sits does not matter; where the KSEG0-mapped images sit does.
    #[cfg(target_arch = "x86")]
    {
        RamDiskBase = crate::mm::MmAllocateHighestMemoryBelowAddress(
            RamDiskFileSize as usize,
            usize::MAX as PVOID,
            LoaderXIPRom,
        );
    }
    #[cfg(not(target_arch = "x86"))]
    {
        RamDiskBase = MmAllocateMemoryWithType((RamDiskFileSize) as usize , LoaderXIPRom);
    }
    if RamDiskBase.is_null() {
        RamDiskFileSize = 0;
        ArcClose(RamFileId);
        UiMessageBox(b"Failed to allocate memory for RAM disk.\0".as_ptr() as PCSTR);
        return ENOMEM;
    }

    Position.QuadPart = 0;
    Status = ArcSeek(RamFileId, core::ptr::addr_of_mut!(Position), SeekAbsolute);
    if Status != ESUCCESS {
        MmFreeMemory(RamDiskBase);
        RamDiskBase = core::ptr::null_mut();
        RamDiskFileSize = 0;
        ArcClose(RamFileId);
        UiMessageBox(b"Failed to read RAM disk.\0".as_ptr() as PCSTR);
        return Status;
    }

    TotalRead = 0;
    Percent = 0;
    while TotalRead < RamDiskFileSize {
        if (RamDiskFileSize - TotalRead) < ChunkSize as u64 {
            ChunkSize = (RamDiskFileSize - TotalRead) as u32;
        }

        UiUpdateProgressBar(Percent, core::ptr::null_mut());

        Status = ArcRead(RamFileId,
                        (RamDiskBase as usize + TotalRead as usize) as *mut core::ffi::c_void,
                        ChunkSize,
                        core::ptr::addr_of_mut!(Count));
        if (Status != ESUCCESS) || (Count != ChunkSize) {
            Status = if Status != ESUCCESS { Status } else { EIO };
            MmFreeMemory(RamDiskBase);
            RamDiskBase = core::ptr::null_mut();
            RamDiskFileSize = 0;
            ArcClose(RamFileId);
            UiMessageBox(b"Failed to read RAM disk.\0".as_ptr() as PCSTR);
            return Status;
        }

        TotalRead += ChunkSize as u64;
        Percent += PercentPerChunk;
    }
    UiUpdateProgressBar(100, core::ptr::null_mut());
    ArcClose(RamFileId);
    ESUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn RamDiskInitialize(
    InitRamDisk: BOOLEAN,
    LoadOptions: PCSTR,
    DefaultPath: PCSTR,
) -> ARC_STATUS
{

    TRACE(
        b"RamDiskInitialize(%s, '%s', '%s')\n\0".as_ptr() as PCSTR, (if InitRamDisk != 0 { b"INIT\0".as_ptr() as PCSTR } else { b"REGULAR\0".as_ptr() as PCSTR }) as usize, (if !LoadOptions.is_null() { LoadOptions } else { b"n/a\0".as_ptr() as PCSTR }) as usize, (if !DefaultPath.is_null() { DefaultPath } else { b"n/a\0".as_ptr() as PCSTR }) as usize);

    if (RamDiskBase != gInitRamDiskBase) &&
        (RamDiskFileSize != ((gInitRamDiskSize) as u64)) &&
        (gInitRamDiskSize != 0)
    {
        MmFreeMemory(RamDiskBase);
    }
    RamDiskBase = core::ptr::null_mut();
    RamDiskFileSize = 0;
    RamDiskImageLength = 0;
    RamDiskImageOffset = 0;
    RamDiskOffset = 0;

    if InitRamDisk != 0
    {
        if gInitRamDiskBase.is_null() || gInitRamDiskSize == 0 {
            return ENODEV;
        }

        RamDiskBase = gInitRamDiskBase;
        RamDiskFileSize = (gInitRamDiskSize) as u64 ;
    }
    else
    {
        let Status: ARC_STATUS;
        let mut FileName: [i8; 260] = [0; 260];

        if !LoadOptions.is_null()
        {
            let mut Option: PCSTR;
            let mut FileNameLength: u32 = 0;

            Option = NtLdrGetOptionEx(LoadOptions, b"RDPATH=\0".as_ptr() as PCSTR, &mut FileNameLength);
            if !Option.is_null() && (FileNameLength > 7)
            {
                Option = Option.add(7);
                FileNameLength -= 7;
                RtlStringCbCopyNA(FileName.as_mut_ptr() as PCSTR, core::mem::size_of_val(&FileName),
                                  Option, (FileNameLength as usize) * core::mem::size_of::<i8>());
            }

            Option = NtLdrGetOption(LoadOptions, b"RDIMAGELENGTH=\0".as_ptr() as PCSTR);
            if !Option.is_null()
            {
                RamDiskImageLength = _atoi64(Option.add(14)) as u64;
            }

            Option = NtLdrGetOption(LoadOptions, b"RDIMAGEOFFSET=\0".as_ptr() as PCSTR);
            if !Option.is_null()
            {
                RamDiskImageOffset = atol(Option.add(14)) as u32;
            }
        }

        if *FileName.as_ptr() != 0 {
            Status = RamDiskLoadVirtualFile(FileName.as_ptr() as PCSTR, DefaultPath);
        } else {
            Status = RamDiskLoadVirtualFile(DefaultPath, core::ptr::null());
        }
        if Status != ESUCCESS {
            return Status;
        }
    }

    if RamDiskImageLength == 0 || (RamDiskImageLength > RamDiskFileSize - (RamDiskImageOffset as u64)) {
        RamDiskImageLength = RamDiskFileSize - (RamDiskImageOffset as u64);
    }

    if RamDiskDeviceRegistered == 0
    {
        FsRegisterDevice(b"ramdisk(0)\0".as_ptr() as PCSTR, &RamDiskVtbl as *const _ as *const core::ffi::c_void);
        RamDiskDeviceRegistered = 1u8;
    }

    ESUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     BSD - See COPYING.ARM in the top level directory
//  * PURPOSE:     Implements routines to support booting from a RAM Disk.
//  * COPYRIGHT:   Copyright 2008 ReactOS Portable Systems Group
//  *              Copyright 2009 Hervé Poussineau
//  *              Copyright 2019 Hermes Belusca-Maito
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <freeldr.h>
// #include "../ntldr/ntldropts.h"
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(DISK);
//
// /* GLOBALS ********************************************************************/
//
// PVOID gInitRamDiskBase = NULL;
// ULONG gInitRamDiskSize = 0;
//
// static BOOLEAN   RamDiskDeviceRegistered = FALSE;
// static PVOID     RamDiskBase;
// static ULONGLONG RamDiskFileSize;    // FIXME: RAM disks currently limited to 4GB.
// static ULONGLONG RamDiskImageLength; // Size of valid data in the Ramdisk (usually == RamDiskFileSize - RamDiskImageOffset)
// static ULONG     RamDiskImageOffset; // Starting offset from the Ramdisk base.
// static ULONGLONG RamDiskOffset;      // Current position in the Ramdisk.
//
// /* FUNCTIONS ******************************************************************/
//
// static ARC_STATUS RamDiskClose(ULONG FileId)
// {
//     /* Nothing to do */
//     return ESUCCESS;
// }
//
// static ARC_STATUS RamDiskGetFileInformation(ULONG FileId, FILEINFORMATION* Information)
// {
//     RtlZeroMemory(Information, sizeof(*Information));
//     Information->EndingAddress.QuadPart = RamDiskImageLength;
//     Information->CurrentAddress.QuadPart = RamDiskOffset;
//     Information->Type = DiskPeripheral;
//
//     return ESUCCESS;
// }
//
// static ARC_STATUS RamDiskOpen(CHAR* Path, OPENMODE OpenMode, ULONG* FileId)
// {
//     /* Always return success, as contents are already in memory */
//     return ESUCCESS;
// }
//
// static ARC_STATUS RamDiskRead(ULONG FileId, VOID* Buffer, ULONG N, ULONG* Count)
// {
//     PVOID StartAddress;
//
//     /* Don't allow reads past our image */
//     if (RamDiskOffset >= RamDiskImageLength || RamDiskOffset + N > RamDiskImageLength)
//     {
//         *Count = 0;
//         return EIO;
//     }
//     // N = min(N, RamdiskImageLength - RamDiskOffset);
//
//     /* Get actual pointer */
//     StartAddress = (PVOID)((ULONG_PTR)RamDiskBase + RamDiskImageOffset + (ULONG_PTR)RamDiskOffset);
//
//     /* Do the read */
//     RtlCopyMemory(Buffer, StartAddress, N);
//     RamDiskOffset += N;
//     *Count = N;
//
//     return ESUCCESS;
// }
//
// static ARC_STATUS RamDiskSeek(ULONG FileId, LARGE_INTEGER* Position, SEEKMODE SeekMode)
// {
//     LARGE_INTEGER NewPosition = *Position;
//
//     switch (SeekMode)
//     {
//         case SeekAbsolute:
//             break;
//         case SeekRelative:
//             NewPosition.QuadPart += RamDiskOffset;
//             break;
//         default:
//             ASSERT(FALSE);
//             return EINVAL;
//     }
//
//     if (NewPosition.QuadPart >= RamDiskImageLength)
//         return EINVAL;
//
//     RamDiskOffset = NewPosition.QuadPart;
//     return ESUCCESS;
// }
//
// static const DEVVTBL RamDiskVtbl =
// {
//     RamDiskClose,
//     RamDiskGetFileInformation,
//     RamDiskOpen,
//     RamDiskRead,
//     RamDiskSeek,
// };
//
// static ARC_STATUS
// RamDiskLoadVirtualFile(
//     IN PCSTR FileName,
//     IN PCSTR DefaultPath OPTIONAL)
// {
//     ARC_STATUS Status;
//     ULONG RamFileId;
//     ULONG ChunkSize, Count;
//     ULONGLONG TotalRead;
//     ULONG PercentPerChunk, Percent;
//     FILEINFORMATION Information;
//     LARGE_INTEGER Position;
//
//     /* Display progress */
//     UiDrawProgressBarCenter("Loading RamDisk...");
//
//     /* Try opening the Ramdisk file */
//     TRACE("RamDiskLoadVirtualFile: Opening '%s', '%s'\n", (//           FileName) as usize, (DefaultPath ? DefaultPath : "n/a") as usize);
//     Status = FsOpenFile(FileName, DefaultPath, OpenReadOnly, &RamFileId);
//     if (Status != ESUCCESS)
//         return Status;
//
//     /* Get the file or device size */
//     Status = ArcGetFileInformation(RamFileId, &Information);
//     if (Status != ESUCCESS)
//     {
//         ArcClose(RamFileId);
//         return Status;
//     }
//     /* NOTE: For partitions, StartingAddress/EndingAddress are the start/end
//      * positions of the partition as byte offsets from the start of the disk */
//     Information.EndingAddress.QuadPart -= Information.StartingAddress.QuadPart;
//     Information.StartingAddress.QuadPart = 0ULL;
//
//     /* If we are actually opening a RAW device, retrieve instead its usable volume size */
//     if (Information.FileNameLength == 0 && Information.FileName[0] == ANSI_NULL &&
//         (Information.Type == DiskPeripheral || Information.Type == FloppyDiskPeripheral))
//     {
//         ULONGLONG VolumeSize;
//         Status = FsGetVolumeSize(RamFileId, &VolumeSize);
//         if (Status != ESUCCESS)
//             ERR("Couldn't retrieve volume size on device '%s', falling back to RAW size\n", (FileName) as usize);
//         else
//             Information.EndingAddress.QuadPart = VolumeSize;
//     }
//
//     TRACE("RAMDISK size: %I64u (High: %lu ; Low: %lu)\n", (//           Information.EndingAddress.QuadPart) as usize, (//           Information.EndingAddress.HighPart) as usize, (//           Information.EndingAddress.LowPart) as usize);
//
//     /* FIXME: For now, limit RAM disks to 4GB */
//     if (Information.EndingAddress.HighPart != 0) // (RamDiskFileSize >= 0x100000000ULL)
//     {
//         ArcClose(RamFileId);
//         UiMessageBox("RAM disk too big.");
//         return ENOMEM;
//     }
//     RamDiskFileSize = Information.EndingAddress.QuadPart;
//
//     /* Allocate memory for it */
//     ChunkSize = 8 * 1024 * 1024;
//     if (RamDiskFileSize < ChunkSize)
//         PercentPerChunk = 0;
//     else
//         PercentPerChunk = 100 * ChunkSize / RamDiskFileSize;
//     RamDiskBase = MmAllocateMemoryWithType(RamDiskFileSize, LoaderXIPRom);
//     if (!RamDiskBase)
//     {
//         RamDiskFileSize = 0;
//         ArcClose(RamFileId);
//         UiMessageBox("Failed to allocate memory for RAM disk.");
//         return ENOMEM;
//     }
//
//     /*
//      * Read it in chunks, starting at the beginning
//      */
//     Position.QuadPart = 0;
//     Status = ArcSeek(RamFileId, &Position, SeekAbsolute);
//     if (Status != ESUCCESS)
//         goto ReadFailure;
//
//     for (TotalRead = 0, Percent = 0;
//          TotalRead < RamDiskFileSize;
//          TotalRead += ChunkSize, Percent += PercentPerChunk)
//     {
//         /* If we are at the last chunk, read only what's remaining */
//         if ((RamDiskFileSize - TotalRead) < ChunkSize)
//             ChunkSize = (ULONG)(RamDiskFileSize - TotalRead);
//
//         /* Update progress */
//         UiUpdateProgressBar(Percent, NULL);
//
//         /* Copy the data */
//         Status = ArcRead(RamFileId,
//                          (PVOID)((ULONG_PTR)RamDiskBase + (ULONG_PTR)TotalRead),
//                          ChunkSize,
//                          &Count);
//         if ((Status != ESUCCESS) || (Count != ChunkSize))
//         {
//             Status = ((Status != ESUCCESS) ? Status : EIO);
//             goto ReadFailure;
//         }
//     }
//     UiUpdateProgressBar(100, NULL);
//     ArcClose(RamFileId);
//     return ESUCCESS;
//
// ReadFailure:
//     MmFreeMemory(RamDiskBase);
//     RamDiskBase = NULL;
//     RamDiskFileSize = 0;
//     ArcClose(RamFileId);
//     UiMessageBox("Failed to read RAM disk.");
//     return Status;
// }
//
// ARC_STATUS
// RamDiskInitialize(
//     IN BOOLEAN InitRamDisk,
//     IN PCSTR LoadOptions OPTIONAL,
//     IN PCSTR DefaultPath OPTIONAL)
// {
//     TRACE("RamDiskInitialize(%s, '%s', '%s')\n", (//           InitRamDisk ? "INIT" : "REGULAR") as usize, (//           LoadOptions ? LoadOptions : "n/a") as usize, (//           DefaultPath ? DefaultPath : "n/a") as usize);
//
//     /* Reset the RAMDISK device */
//     if ((RamDiskBase != gInitRamDiskBase) &&
//         (RamDiskFileSize != gInitRamDiskSize) &&
//         (gInitRamDiskSize != 0))
//     {
//         /* This is not the initial Ramdisk, so we can free the allocated memory */
//         MmFreeMemory(RamDiskBase);
//     }
//     RamDiskBase = NULL;
//     RamDiskFileSize = 0;
//     RamDiskImageLength = 0;
//     RamDiskImageOffset = 0;
//     RamDiskOffset = 0;
//
//     if (InitRamDisk)
//     {
//         /* We initialize the initial Ramdisk: it should be present in memory */
//         if (!gInitRamDiskBase || gInitRamDiskSize == 0)
//             return ENODEV;
//
//         // TODO: Handle SDI image.
//
//         RamDiskBase = gInitRamDiskBase;
//         RamDiskFileSize = gInitRamDiskSize;
//         ASSERT(RamDiskFileSize < 0x100000000); // See FIXME about 4GB support in RamDiskLoadVirtualFile().
//     }
//     else
//     {
//         /* We initialize the Ramdisk from the load options */
//         ARC_STATUS Status;
//         CHAR FileName[MAX_PATH] = "";
//
//         /* If we don't have any load options, initialize an empty Ramdisk */
//         if (LoadOptions)
//         {
//             PCSTR Option;
//             ULONG FileNameLength;
//
//             /* Ramdisk image file name */
//             Option = NtLdrGetOptionEx(LoadOptions, "RDPATH=", &FileNameLength);
//             if (Option && (FileNameLength > 7))
//             {
//                 /* Copy the file name */
//                 Option += 7; FileNameLength -= 7;
//                 RtlStringCbCopyNA(FileName, sizeof(FileName),
//                                   Option, FileNameLength * sizeof(CHAR));
//             }
//
//             /* Ramdisk image length */
//             Option = NtLdrGetOption(LoadOptions, "RDIMAGELENGTH=");
//             if (Option)
//             {
//                 RamDiskImageLength = _atoi64(Option + 14);
//             }
//
//             /* Ramdisk image offset */
//             Option = NtLdrGetOption(LoadOptions, "RDIMAGEOFFSET=");
//             if (Option)
//             {
//                 RamDiskImageOffset = atol(Option + 14);
//             }
//         }
//
//         if (*FileName)
//             Status = RamDiskLoadVirtualFile(FileName, DefaultPath);
//         else
//             Status = RamDiskLoadVirtualFile(DefaultPath, NULL);
//         if (Status != ESUCCESS)
//             return Status;
//     }
//
//     /* Adjust the Ramdisk image length if needed */
//     if (!RamDiskImageLength || (RamDiskImageLength > RamDiskFileSize - RamDiskImageOffset))
//         RamDiskImageLength = RamDiskFileSize - RamDiskImageOffset;
//
//     /* Register the RAMDISK device */
//     if (!RamDiskDeviceRegistered)
//     {
//         FsRegisterDevice("ramdisk(0)", &RamDiskVtbl);
//         RamDiskDeviceRegistered = TRUE;
//     }
//
//     return ESUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
