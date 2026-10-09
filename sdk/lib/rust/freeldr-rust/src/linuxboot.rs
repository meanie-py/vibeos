// linuxboot.rs — Rust translation of boot/freeldr/freeldr/linuxboot.c
// BIOS-only (pcat.cmake); gated `#[cfg(not(feature = "uefi"))]` in lib.rs.
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, FILEINFORMATION, LARGE_INTEGER, PCSTR, PSTR};
use core::ffi::c_void;

extern "C" {
    fn TRACE(fmt: *const i8, ...);
    fn ERR(fmt: *const i8, ...);

    // UI surface (lib/ui/{ui,tui,...}.c).
    fn UiDrawBackdrop(BackdropTop: u32);
    fn UiGetScreenHeight() -> u32;
    fn UiDrawStatusText(StatusText: PCSTR);
    fn UiDrawProgressBarCenter(ProgressText: PCSTR);
    fn UiShowMessageBoxesInArgv(Argc: ULONG, Argv: *mut PCHAR);
    fn UiMessageBox(MessageText: PCSTR, ...);
    fn UiUnInitialize(BootText: PCSTR);
    fn UiUpdateProgressBar(Percentage: ULONG, ProgressText: PCSTR);

    // Loader argument / boot path helpers.
    fn GetArgumentValue(Argc: ULONG, Argv: *mut PCHAR, ArgumentName: PCSTR) -> PCHAR;
    fn ConstructArcPath(ArcPath: PSTR, RelativePath: PCSTR, DriveNumber: UCHAR, PartitionNumber: ULONG);
    fn DriveMapGetBiosDriveNumber(DeviceName: PCSTR) -> UCHAR;

    // FS / ARC IO.
    fn FsOpenFile(FileName: PCSTR, DefaultPath: PCSTR, OpenMode: u32, FileId: *mut ULONG) -> ARC_STATUS;
    fn ArcGetFileInformation(FileId: ULONG, Information: *mut FILEINFORMATION) -> ARC_STATUS;
    fn ArcClose(FileId: ULONG) -> ARC_STATUS;
    fn ArcSeek(FileId: ULONG, Position: *mut LARGE_INTEGER, SeekMode: u32) -> ARC_STATUS;
    fn ArcRead(FileId: ULONG, Buffer: *mut c_void, N: ULONG, Count: *mut ULONG) -> ARC_STATUS;

    // MM (the freeldr heap, not ntoskrnl MM).
    fn MmAllocateMemoryWithType(MemorySize: SIZE_T, MemoryType: TYPE_OF_MEMORY) -> PVOID;
    fn MmAllocateMemoryAtAddress(MemorySize: SIZE_T, VirtualAddress: PVOID, MemoryType: TYPE_OF_MEMORY) -> PVOID;
    fn MmAllocateHighestMemoryBelowAddress(MemorySize: SIZE_T, HighestAddress: PVOID, MemoryType: TYPE_OF_MEMORY) -> PVOID;
    fn MmFreeMemory(MemoryPointer: PVOID);

    // INI cleanup.
    fn IniCleanup();

    // RTL / CRT (strcpy/strrchr come from crate::frldr::*).
    fn RtlStringCbPrintfA(pszDest: *mut i8, cbDest: usize, pszFormat: *const i8, ...) -> i32;
    fn atoi(s: PCSTR) -> i32;
    fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32;
    fn strlen(s: PCSTR) -> usize;
    fn memmove(dst: *mut c_void, src: *const c_void, len: usize) -> *mut c_void;

    // linux.S — real-mode trampoline that boots the kernel.
    fn BootLinuxKernel(KernelSize: ULONG, KernelCurrentLoadAddress: PVOID, KernelTargetLoadAddress: PVOID) -> !;
}

// ---- linux.h constants/types (file-local; no header crate exposure) ----
pub const LINUX_LOADER_TYPE_FREELOADER: UCHAR = 0x81;
pub const LINUX_COMMAND_LINE_MAGIC: USHORT = 0xA33F;
pub const LINUX_SETUP_HEADER_ID: ULONG = 0x53726448; // 'HdrS'
pub const LINUX_BOOT_SECTOR_MAGIC: USHORT = 0xAA55;
pub const LINUX_KERNEL_LOAD_ADDRESS: usize = 0x100000;
pub const LINUX_FLAG_LOAD_HIGH: UCHAR = 0x01;
pub const LINUX_FLAG_CAN_USE_HEAP: UCHAR = 0x80;
pub const LINUX_MAX_INITRD_ADDRESS: usize = 0x38000000;

#[repr(C, packed)]
pub struct LINUX_BOOTSECTOR {
    pub BootCode1: [UCHAR; 0x20],
    pub CommandLineMagic: USHORT,
    pub CommandLineOffset: USHORT,
    pub BootCode2: [UCHAR; 0x1CD],
    pub SetupSectors: UCHAR,
    pub RootFlags: USHORT,
    pub SystemSize: USHORT,
    pub SwapDevice: USHORT,
    pub RamSize: USHORT,
    pub VideoMode: USHORT,
    pub RootDevice: USHORT,
    pub BootFlag: USHORT, // 0xAA55
}
pub type PLINUX_BOOTSECTOR = *mut LINUX_BOOTSECTOR;

#[repr(C, packed)]
pub struct LINUX_SETUPSECTOR {
    pub JumpInstruction: [UCHAR; 2],
    pub SetupHeaderSignature: ULONG,
    pub Version: USHORT,
    pub RealModeSwitch: USHORT,
    pub SetupSeg: USHORT,
    pub StartSystemSeg: USHORT,
    pub KernelVersion: USHORT,
    pub TypeOfLoader: UCHAR,
    pub LoadFlags: UCHAR,
    pub SetupMoveSize: USHORT,
    pub Code32Start: ULONG,
    pub RamdiskAddress: ULONG,
    pub RamdiskSize: ULONG,
    pub BootSectKludgeOffset: USHORT,
    pub BootSectKludgeSegment: USHORT,
    pub HeapEnd: USHORT,
    pub Pad1: USHORT,
    pub CommandLinePointer: ULONG,
    pub InitrdAddressMax: ULONG,
}
pub type PLINUX_SETUPSECTOR = *mut LINUX_SETUPSECTOR;

// ---- file-scope preamble (translated) ----
pub const LINUX_READ_CHUNK_SIZE: u32 = 0x20000;

// xdk intype.h `PtrToUlong(p) = ((ULONG)(ULONG_PTR)p)` — truncating cast,
// matches MSVC intrinsic. Local since freeldr's frldr keystone only exports
// UlongToPtr.
#[inline]
unsafe fn PtrToUlong(p: *const c_void) -> ULONG {
    (p as ULONG_PTR) as ULONG
}

// C_ASSERT(cond) is a compile-time check; collapse to const eval.
macro_rules! C_ASSERT { ($cond:expr) => { const _: () = assert!($cond); }; }

pub static mut LinuxBootSector: PLINUX_BOOTSECTOR = core::ptr::null_mut();
pub static mut LinuxSetupSector: PLINUX_SETUPSECTOR = core::ptr::null_mut();
pub static mut SetupSectorSize: ULONG = 0;
pub static mut NewStyleLinuxKernel: BOOLEAN = 0;
pub static mut LinuxKernelSize: ULONG = 0;
pub static mut LinuxInitrdSize: ULONG = 0;
pub static mut LinuxKernelName: PCSTR = core::ptr::null();
pub static mut LinuxInitrdName: PCSTR = core::ptr::null();
pub static mut LinuxCommandLine: PSTR = core::ptr::null_mut();
pub static mut LinuxCommandLineSize: ULONG = 0;
pub static mut LinuxKernelLoadAddress: PVOID = core::ptr::null_mut();
pub static mut LinuxInitrdLoadAddress: PVOID = core::ptr::null_mut();
pub static mut LinuxBootDescription: [CHAR; 80] = [0; 80];

unsafe fn RemoveQuotes(QuotedString: PSTR) {
    let mut p: PSTR;
    let Start: PSTR;
    let Size: usize;

    p = QuotedString;
    while *p == b' ' as i8 || *p == b'\t' as i8 || *p == b'"' as i8 {
        p = p.offset(1);
    }
    Start = p;

    while *p != 0 && *p != b'"' as i8 {
        p = p.offset(1);
    }
    *p = 0;

    Size = (strlen(Start) + 1) * core::mem::size_of::<i8>();
    memmove(QuotedString as *mut core::ffi::c_void, Start as *const core::ffi::c_void, Size);
}

#[no_mangle]
pub unsafe extern "C" fn LoadAndBootLinux(
    Argc: ULONG,
    Argv: *mut PCHAR,
    _Envp: *mut PCHAR,
) -> ARC_STATUS {
    let mut Status: ARC_STATUS;
    let Description: PCSTR;
    let mut ArgValue: PCSTR;
    let mut BootPath: PCSTR;
    let mut LinuxKernel: ULONG = 0;
    let mut LinuxInitrdFile: ULONG = 0;
    let mut FileInfo: FILEINFORMATION = core::mem::zeroed();
    let mut ArcPath: [CHAR; 260] = [0; 260];

    'main: {
        #[cfg(debug_assertions)]
        {
            ArgValue = GetArgumentValue(Argc, Argv, b"BootType\0".as_ptr() as PCSTR);
            if ArgValue.is_null() || *ArgValue == 0 || _stricmp(ArgValue, b"Linux\0".as_ptr() as PCSTR) != 0 {
                ERR(b"Unexpected boot type '%s', aborting\n\0".as_ptr() as *const i8,
                    if !ArgValue.is_null() { ArgValue } else { b"n/a\0".as_ptr() as PCSTR });
                return EINVAL;
            }
        }

        Description = GetArgumentValue(Argc, Argv, b"LoadIdentifier\0".as_ptr() as PCSTR);
        if !Description.is_null() && *Description != 0 {
            RtlStringCbPrintfA(
                LinuxBootDescription.as_mut_ptr(),
                core::mem::size_of_val(&LinuxBootDescription),
                b"Loading %s...\0".as_ptr() as *const i8,
                Description,
            );
        } else {
            strcpy(LinuxBootDescription.as_mut_ptr(), b"Loading Linux...\0".as_ptr() as PCSTR);
        }

        UiDrawBackdrop(UiGetScreenHeight());
        UiDrawStatusText(LinuxBootDescription.as_ptr());
        UiDrawProgressBarCenter(LinuxBootDescription.as_ptr());

        UiShowMessageBoxesInArgv(Argc, Argv);

        BootPath = GetArgumentValue(Argc, Argv, b"BootPath\0".as_ptr() as PCSTR);
        if BootPath.is_null() || *BootPath == 0 {
            ArgValue = GetArgumentValue(Argc, Argv, b"BootDrive\0".as_ptr() as PCSTR);
            if !ArgValue.is_null() && *ArgValue != 0 {
                let DriveNumber: UCHAR = DriveMapGetBiosDriveNumber(ArgValue);

                let mut PartitionNumber: ULONG = 0;
                ArgValue = GetArgumentValue(Argc, Argv, b"BootPartition\0".as_ptr() as PCSTR);
                if !ArgValue.is_null() && *ArgValue != 0 {
                    PartitionNumber = atoi(ArgValue) as ULONG;
                }
                if PartitionNumber == 0 {
                    UiMessageBox(b"Boot partition cannot be 0!\0".as_ptr() as PCSTR);
                    break 'main;
                }

                ConstructArcPath(ArcPath.as_mut_ptr(), b"\0".as_ptr() as PCSTR, DriveNumber, PartitionNumber);
                *strrchr(ArcPath.as_mut_ptr(), b'\\' as i32) = 0;

                BootPath = ArcPath.as_ptr();
            } else {
                BootPath = GetArgumentValue(Argc, Argv, b"SystemPartition\0".as_ptr() as PCSTR);
            }
        }

        LinuxKernelName = GetArgumentValue(Argc, Argv, b"Kernel\0".as_ptr() as PCSTR);
        if LinuxKernelName.is_null() || *LinuxKernelName == 0 {
            UiMessageBox(b"Linux kernel filename not specified for selected OS!\0".as_ptr() as PCSTR);
            break 'main;
        }

        LinuxInitrdName = GetArgumentValue(Argc, Argv, b"Initrd\0".as_ptr() as PCSTR);

        LinuxCommandLineSize = 0;
        LinuxCommandLine = GetArgumentValue(Argc, Argv, b"CommandLine\0".as_ptr() as PCSTR);
        if !LinuxCommandLine.is_null() && *LinuxCommandLine != 0 {
            RemoveQuotes(LinuxCommandLine);
            LinuxCommandLineSize = strlen(LinuxCommandLine) as ULONG + 1;
            LinuxCommandLineSize = min(LinuxCommandLineSize, 260);
        }

        Status = FsOpenFile(LinuxKernelName, BootPath, OpenReadOnly, &mut LinuxKernel);
        if Status != ESUCCESS {
            UiMessageBox(b"Linux kernel '%s' not found.\0".as_ptr() as PCSTR, LinuxKernelName);
            break 'main;
        }

        if !LinuxInitrdName.is_null() {
            Status = FsOpenFile(LinuxInitrdName, BootPath, OpenReadOnly, &mut LinuxInitrdFile);
            if Status != ESUCCESS {
                UiMessageBox(b"Linux initrd image '%s' not found.\0".as_ptr() as PCSTR, LinuxInitrdName);
                break 'main;
            }
        }

        if LinuxReadBootSector(LinuxKernel) == 0 {
            break 'main;
        }

        if LinuxReadSetupSector(LinuxKernel) == 0 {
            break 'main;
        }

        Status = ArcGetFileInformation(LinuxKernel, &mut FileInfo);
        if Status != ESUCCESS || FileInfo.EndingAddress.__bindgen_anon_1.HighPart != 0 {
            LinuxKernelSize = 0;
        } else {
            LinuxKernelSize = FileInfo.EndingAddress.__bindgen_anon_1.LowPart - (512 + SetupSectorSize);
        }

        LinuxInitrdSize = 0;
        if !LinuxInitrdName.is_null() {
            Status = ArcGetFileInformation(LinuxInitrdFile, &mut FileInfo);
            if Status != ESUCCESS || FileInfo.EndingAddress.__bindgen_anon_1.HighPart != 0 {
                LinuxInitrdSize = 0;
            } else {
                LinuxInitrdSize = FileInfo.EndingAddress.__bindgen_anon_1.LowPart;
            }
        }

        if LinuxReadKernel(LinuxKernel) == 0 {
            break 'main;
        }

        if !LinuxInitrdName.is_null() {
            if LinuxReadInitrd(LinuxInitrdFile) == 0 {
                break 'main;
            }
        }

        const NODEV: u16 = 0;
        const FLOPPY_MAJOR: u16 = 2;

        if (*LinuxBootSector).RootDevice == NODEV {
            (*LinuxBootSector).RootDevice = ((FLOPPY_MAJOR << 8) | 0) as u16;
        }

        if (*LinuxSetupSector).Version >= 0x0202 {
            (*LinuxSetupSector).CommandLinePointer = 0x99000;
        } else {
            (*LinuxBootSector).CommandLineMagic = LINUX_COMMAND_LINE_MAGIC;
            (*LinuxBootSector).CommandLineOffset = 0x9000;
        }

        if NewStyleLinuxKernel != 0 {
            (*LinuxSetupSector).TypeOfLoader = LINUX_LOADER_TYPE_FREELOADER;
        } else {
            (*LinuxSetupSector).LoadFlags = 0;
        }

        RtlCopyMemory(0x90000 as *mut c_void, LinuxBootSector as *const c_void, 512);
        RtlCopyMemory(0x90200 as *mut c_void, LinuxSetupSector as *const c_void, SetupSectorSize as usize);
        RtlCopyMemory(
            0x99000 as *mut c_void,
            if !LinuxCommandLine.is_null() { LinuxCommandLine as *const c_void } else { b"\0".as_ptr() as *const c_void },
            if !LinuxCommandLine.is_null() { LinuxCommandLineSize as usize } else { 1 },
        );

        UiUnInitialize(b"Booting Linux...\0".as_ptr() as PCSTR);
        IniCleanup();

        BootLinuxKernel(
            LinuxKernelSize,
            LinuxKernelLoadAddress,
            if ((*LinuxSetupSector).LoadFlags & LINUX_FLAG_LOAD_HIGH) != 0 {
                LINUX_KERNEL_LOAD_ADDRESS as PVOID
            } else {
                0x10000 as PVOID
            },
        );
        // BootLinuxKernel is `-> !`; unreachable below.
    }

    // LinuxBootFailed: cleanup + return ENOEXEC.
    if LinuxKernel != 0 {
        ArcClose(LinuxKernel);
    }

    if LinuxInitrdFile != 0 {
        ArcClose(LinuxInitrdFile);
    }

    if !LinuxBootSector.is_null() {
        MmFreeMemory(LinuxBootSector as PVOID);
    }

    if !LinuxSetupSector.is_null() {
        MmFreeMemory(LinuxSetupSector as PVOID);
    }

    if !LinuxKernelLoadAddress.is_null() {
        MmFreeMemory(LinuxKernelLoadAddress);
    }

    if !LinuxInitrdLoadAddress.is_null() {
        MmFreeMemory(LinuxInitrdLoadAddress);
    }

    LinuxBootSector = core::ptr::null_mut();
    LinuxSetupSector = core::ptr::null_mut();
    SetupSectorSize = 0;
    NewStyleLinuxKernel = 0u8;
    LinuxKernelSize = 0;
    LinuxInitrdSize = 0;
    LinuxKernelName = core::ptr::null();
    LinuxInitrdName = core::ptr::null();
    LinuxCommandLine = core::ptr::null_mut();
    LinuxCommandLineSize = 0;
    LinuxKernelLoadAddress = core::ptr::null_mut();
    LinuxInitrdLoadAddress = core::ptr::null_mut();
    *LinuxBootDescription.as_mut_ptr() = 0;

    ENOEXEC
}

unsafe fn LinuxReadBootSector(LinuxKernelFile: ULONG) -> BOOLEAN {
    let mut Position: LARGE_INTEGER = core::mem::zeroed();

    LinuxBootSector = MmAllocateMemoryWithType(512, LoaderSystemCode) as PLINUX_BOOTSECTOR;
    if LinuxBootSector.is_null() {
        return 0u8;
    }

    Position.QuadPart = 0;
    if ArcSeek(LinuxKernelFile, &mut Position, SeekAbsolute) != ESUCCESS {
        return 0u8;
    }
    if ArcRead(LinuxKernelFile, LinuxBootSector as *mut core::ffi::c_void, 512, core::ptr::null_mut()) != ESUCCESS {
        return 0u8;
    }

    if (*LinuxBootSector).BootFlag != LINUX_BOOT_SECTOR_MAGIC {
        UiMessageBox(b"Invalid boot sector magic (0xaa55)\0".as_ptr() as *const i8);
        return 0u8;
    }

    TRACE(b"SetupSectors: %d\n\0".as_ptr() as *const i8, (*LinuxBootSector).SetupSectors as u32);
    TRACE(b"RootFlags:    0x%x\n\0".as_ptr() as *const i8, (*LinuxBootSector).RootFlags as u32);
    TRACE(b"SystemSize:   0x%x\n\0".as_ptr() as *const i8, (*LinuxBootSector).SystemSize as u32);
    TRACE(b"SwapDevice:   0x%x\n\0".as_ptr() as *const i8, (*LinuxBootSector).SwapDevice as u32);
    TRACE(b"RamSize:      0x%x\n\0".as_ptr() as *const i8, (*LinuxBootSector).RamSize as u32);
    TRACE(b"VideoMode:    0x%x\n\0".as_ptr() as *const i8, (*LinuxBootSector).VideoMode as u32);
    TRACE(b"RootDevice:   0x%x\n\0".as_ptr() as *const i8, (*LinuxBootSector).RootDevice as u32);
    TRACE(b"BootFlag:     0x%x\n\0".as_ptr() as *const i8, (*LinuxBootSector).BootFlag as u32);

    1u8
}

unsafe fn LinuxReadSetupSector(LinuxKernelFile: ULONG) -> BOOLEAN {
    let mut Position: LARGE_INTEGER = core::mem::zeroed();
    let mut TempLinuxSetupSector: [UCHAR; 512] = [0; 512];

    Position.QuadPart = 512;
    if ArcSeek(LinuxKernelFile, &mut Position, SeekAbsolute) != ESUCCESS {
        return 0u8;
    }
    if ArcRead(LinuxKernelFile, TempLinuxSetupSector.as_mut_ptr() as PVOID, 512, core::ptr::null_mut()) != ESUCCESS {
        return 0u8;
    }

    LinuxSetupSector = TempLinuxSetupSector.as_ptr() as PLINUX_SETUPSECTOR;
    if LinuxCheckKernelVersion() == 0u8 {
        return 0u8;
    }

    if NewStyleLinuxKernel != 0 {
        SetupSectorSize = 512 * (*LinuxBootSector).SetupSectors as ULONG;
    } else {
        SetupSectorSize = 512 * 4;
    }

    LinuxSetupSector = MmAllocateMemoryWithType(SetupSectorSize as usize, LoaderSystemCode) as PLINUX_SETUPSECTOR;
    if LinuxSetupSector.is_null() {
        return 0u8;
    }

    RtlCopyMemory(LinuxSetupSector as *mut c_void, TempLinuxSetupSector.as_ptr() as *const c_void, 512);

    Position.QuadPart = 1024;
    if ArcSeek(LinuxKernelFile, &mut Position, SeekAbsolute) != ESUCCESS {
        return 0u8;
    }
    if ArcRead(LinuxKernelFile, (LinuxSetupSector as ULONG_PTR + 512) as PVOID, SetupSectorSize - 512, core::ptr::null_mut()) != ESUCCESS {
        return 0u8;
    }

    TRACE(b"SetupHeaderSignature: 0x%x (HdrS)\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).SetupHeaderSignature as u32);
    TRACE(b"Version:              0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).Version as u32);
    TRACE(b"RealModeSwitch: 0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).RealModeSwitch as u32);
    TRACE(b"SetupSeg:       0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).SetupSeg as u32);
    TRACE(b"StartSystemSeg: 0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).StartSystemSeg as u32);
    TRACE(b"KernelVersion:  0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).KernelVersion as u32);
    TRACE(b"TypeOfLoader:   0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).TypeOfLoader as u32);
    TRACE(b"LoadFlags:      0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).LoadFlags as u32);
    TRACE(b"SetupMoveSize:  0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).SetupMoveSize as u32);
    TRACE(b"Code32Start:    0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).Code32Start as u32);
    TRACE(b"RamdiskAddress: 0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).RamdiskAddress as u32);
    TRACE(b"RamdiskSize:    0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).RamdiskSize as u32);
    TRACE(b"BootSectKludgeOffset:  0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).BootSectKludgeOffset as u32);
    TRACE(b"BootSectKludgeSegment: 0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).BootSectKludgeSegment as u32);
    TRACE(b"HeapEnd: 0x%x\n\0".as_ptr() as PCHAR, (*LinuxSetupSector).HeapEnd as u32);

    1u8
}

unsafe fn LinuxReadKernel(LinuxKernelFile: ULONG) -> BOOLEAN {
    let mut LoadAddress: PVOID;
    let mut Position: LARGE_INTEGER = core::mem::zeroed();
    let mut BytesLoaded: ULONG;
    let mut StatusText: [CHAR; 260] = [0; 260];

    RtlStringCbPrintfA(
        StatusText.as_mut_ptr(),
        core::mem::size_of_val(&StatusText),
        b"Loading %s\0".as_ptr() as PCHAR,
        LinuxKernelName,
    );
    UiDrawStatusText(StatusText.as_ptr());

    LinuxKernelLoadAddress = MmAllocateMemoryAtAddress(
        LinuxKernelSize as usize,
        LINUX_KERNEL_LOAD_ADDRESS as PVOID,
        LoaderSystemCode,
    );
    if LinuxKernelLoadAddress != LINUX_KERNEL_LOAD_ADDRESS as PVOID {
        LinuxKernelLoadAddress = MmAllocateMemoryWithType(LinuxKernelSize as usize, LoaderSystemCode);
        if LinuxKernelLoadAddress == core::ptr::null_mut() {
            TRACE(
                b"Failed to allocate 0x%lx bytes for the kernel image.\n\0".as_ptr() as PCHAR,
                LinuxKernelSize,
            );
            return 0u8;
        }
    }

    LoadAddress = LinuxKernelLoadAddress;

    Position.QuadPart = (512 + SetupSectorSize) as i64;
    if ArcSeek(LinuxKernelFile, &mut Position, SeekAbsolute) != ESUCCESS {
        return 0u8;
    }
    BytesLoaded = 0;
    while BytesLoaded < LinuxKernelSize {
        if ArcRead(
            LinuxKernelFile,
            LoadAddress,
            LINUX_READ_CHUNK_SIZE,
            core::ptr::null_mut(),
        ) != ESUCCESS
        {
            return 0u8;
        }

        BytesLoaded += LINUX_READ_CHUNK_SIZE;
        LoadAddress = (LoadAddress as ULONG_PTR + LINUX_READ_CHUNK_SIZE as ULONG_PTR) as PVOID;

        UiUpdateProgressBar(
            BytesLoaded * 100 / (LinuxKernelSize + LinuxInitrdSize),
            core::ptr::null_mut(),
        );
    }

    1u8
}

unsafe fn LinuxCheckKernelVersion() -> BOOLEAN {
    NewStyleLinuxKernel = 0u8;

    if (*LinuxSetupSector).SetupHeaderSignature != LINUX_SETUP_HEADER_ID {
        NewStyleLinuxKernel = 0u8;
    } else if (*LinuxSetupSector).Version < 0x0200 {
        NewStyleLinuxKernel = 0u8;
    } else if (*LinuxSetupSector).Version == 0x0200 {
        NewStyleLinuxKernel = 1u8;
    } else if (*LinuxSetupSector).Version >= 0x0201 {
        NewStyleLinuxKernel = 1u8;
        (*LinuxSetupSector).HeapEnd = 0x9000;
        (*LinuxSetupSector).LoadFlags |= LINUX_FLAG_CAN_USE_HEAP;
    }

    if (NewStyleLinuxKernel == 0u8) && !LinuxInitrdName.is_null() {
        UiMessageBox(b"Error: Cannot load a ramdisk (initrd) with an old kernel image.\0".as_ptr() as PCHAR);
        return 0u8;
    }

    1u8
}

unsafe fn LinuxReadInitrd(LinuxInitrdFile: u32) -> u8 {
    let mut BytesLoaded: u32;
    let mut StatusText: [i8; 260] = [0; 260];

    RtlStringCbPrintfA(
        StatusText.as_mut_ptr(),
        core::mem::size_of_val(&StatusText),
        b"Loading %s\0".as_ptr() as *const i8,
        LinuxInitrdName
    );
    UiDrawStatusText(StatusText.as_ptr());

    if (*LinuxSetupSector).Version <= 0x0202 {
        #[cfg(target_arch = "x86_64")]
        {
            C_ASSERT!(LINUX_MAX_INITRD_ADDRESS < 0x100000000);
        }
        LinuxInitrdLoadAddress = MmAllocateHighestMemoryBelowAddress(
            LinuxInitrdSize as usize,
            LINUX_MAX_INITRD_ADDRESS as *mut core::ffi::c_void,
            LoaderSystemCode
        );
    } else {
        LinuxInitrdLoadAddress = MmAllocateHighestMemoryBelowAddress(
            LinuxInitrdSize as usize,
            UlongToPtr((*LinuxSetupSector).InitrdAddressMax),
            LoaderSystemCode
        );
    }
    if LinuxInitrdLoadAddress == core::ptr::null_mut() {
        return 0u8;
    }
    #[cfg(target_arch = "x86_64")]
    {
        ASSERT!((LinuxInitrdLoadAddress as usize) < 0x100000000);
    }

    (*LinuxSetupSector).RamdiskAddress = PtrToUlong(LinuxInitrdLoadAddress);
    (*LinuxSetupSector).RamdiskSize = LinuxInitrdSize;

    TRACE(
        b"RamdiskAddress: 0x%x\n\0".as_ptr() as *const i8,
        (*LinuxSetupSector).RamdiskAddress
    );
    TRACE(
        b"RamdiskSize:    0x%x\n\0".as_ptr() as *const i8,
        (*LinuxSetupSector).RamdiskSize
    );

    if (*LinuxSetupSector).Version >= 0x0203 {
        TRACE(
            b"InitrdAddressMax: 0x%x\n\0".as_ptr() as *const i8,
            (*LinuxSetupSector).InitrdAddressMax
        );
    }

    BytesLoaded = 0;
    while BytesLoaded < LinuxInitrdSize {
        if ArcRead(
            LinuxInitrdFile,
            LinuxInitrdLoadAddress,
            LINUX_READ_CHUNK_SIZE,
            core::ptr::null_mut()
        ) != ESUCCESS {
            return 0u8;
        }

        BytesLoaded += LINUX_READ_CHUNK_SIZE;
        LinuxInitrdLoadAddress = (LinuxInitrdLoadAddress as usize + LINUX_READ_CHUNK_SIZE as usize) as *mut core::ffi::c_void;

        UiUpdateProgressBar(
            (BytesLoaded + LinuxKernelSize) * 100 / (LinuxInitrdSize + LinuxKernelSize),
            core::ptr::null_mut()
        );
    }

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 1998-2003  Brian Palmer  <brianp@sginet.com>
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
// /*
//  * The x86 Linux Boot Protocol is explained at:
//  * https://www.kernel.org/doc/Documentation/x86/boot.txt
//  * https://www.linux.it/~rubini/docs/boot/boot.html
//  */
//
// #if defined(_M_IX86) || defined(_M_AMD64)
//
// /* INCLUDES *******************************************************************/
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(LINUX);
//
// /* GLOBALS ********************************************************************/
//
// #define LINUX_READ_CHUNK_SIZE   0x20000 // Read 128k at a time
//
// PLINUX_BOOTSECTOR  LinuxBootSector = NULL;
// PLINUX_SETUPSECTOR LinuxSetupSector = NULL;
// ULONG   SetupSectorSize = 0;
// BOOLEAN NewStyleLinuxKernel = FALSE;
// ULONG   LinuxKernelSize = 0;
// ULONG   LinuxInitrdSize = 0;
// PCSTR   LinuxKernelName = NULL;
// PCSTR   LinuxInitrdName = NULL;
// PSTR    LinuxCommandLine = NULL;
// ULONG   LinuxCommandLineSize = 0;
// PVOID   LinuxKernelLoadAddress = NULL;
// PVOID   LinuxInitrdLoadAddress = NULL;
// CHAR    LinuxBootDescription[80];
//
// /* FUNCTIONS ******************************************************************/
//
// static BOOLEAN LinuxReadBootSector(ULONG LinuxKernelFile);
// static BOOLEAN LinuxReadSetupSector(ULONG LinuxKernelFile);
// static BOOLEAN LinuxReadKernel(ULONG LinuxKernelFile);
// static BOOLEAN LinuxCheckKernelVersion(VOID);
// static BOOLEAN LinuxReadInitrd(ULONG LinuxInitrdFile);
//
// static VOID
// RemoveQuotes(
//     IN OUT PSTR QuotedString)
// {
//     PCHAR  p;
//     PSTR   Start;
//     SIZE_T Size;
//
//     /* Skip spaces up to " */
//     p = QuotedString;
//     while (*p == ' ' || *p == '\t' || *p == '"')
//         ++p;
//     Start = p;
//
//     /* Go up to next " */
//     while (*p != ANSI_NULL && *p != '"')
//         ++p;
//     /* NULL-terminate */
//     *p = ANSI_NULL;
//
//     /* Move the NULL-terminated string back into 'QuotedString' in place */
//     Size = (strlen(Start) + 1) * sizeof(CHAR);
//     memmove(QuotedString, Start, Size);
// }
//
// ARC_STATUS
// LoadAndBootLinux(
//     _In_ ULONG Argc,
//     _In_ PCHAR Argv[],
//     _In_ PCHAR Envp[])
// {
//     ARC_STATUS Status;
//     PCSTR Description;
//     PCSTR ArgValue;
//     PCSTR BootPath;
//     ULONG LinuxKernel = 0;
//     ULONG LinuxInitrdFile = 0;
//     FILEINFORMATION FileInfo;
//     CHAR ArcPath[MAX_PATH];
//
// #if DBG
//     /* Ensure the boot type is the one expected */
//     ArgValue = GetArgumentValue(Argc, Argv, "BootType");
//     if (!ArgValue || !*ArgValue || _stricmp(ArgValue, "Linux") != 0)
//     {
//         ERR("Unexpected boot type '%s', aborting\n", ArgValue ? ArgValue : "n/a");
//         return EINVAL;
//     }
// #endif
//
//     Description = GetArgumentValue(Argc, Argv, "LoadIdentifier");
//     if (Description && *Description)
//         RtlStringCbPrintfA(LinuxBootDescription, sizeof(LinuxBootDescription), "Loading %s...", Description);
//     else
//         strcpy(LinuxBootDescription, "Loading Linux...");
//
//     UiDrawBackdrop(UiGetScreenHeight());
//     UiDrawStatusText(LinuxBootDescription);
//     UiDrawProgressBarCenter(LinuxBootDescription);
//
//     /* Find all the message box settings and run them */
//     UiShowMessageBoxesInArgv(Argc, Argv);
//
//     /*
//      * Check whether we have a "BootPath" value (takes precedence
//      * over both "BootDrive" and "BootPartition").
//      */
//     BootPath = GetArgumentValue(Argc, Argv, "BootPath");
//     if (!BootPath || !*BootPath)
//     {
//         /* We don't have one, check whether we use "BootDrive" and "BootPartition" */
//
//         /* Retrieve the boot drive (optional, fall back to using default path otherwise) */
//         ArgValue = GetArgumentValue(Argc, Argv, "BootDrive");
//         if (ArgValue && *ArgValue)
//         {
//             UCHAR DriveNumber = DriveMapGetBiosDriveNumber(ArgValue);
//
//             /* Retrieve the boot partition (not optional and cannot be zero) */
//             ULONG PartitionNumber = 0;
//             ArgValue = GetArgumentValue(Argc, Argv, "BootPartition");
//             if (ArgValue && *ArgValue)
//                 PartitionNumber = atoi(ArgValue);
//             if (PartitionNumber == 0)
//             {
//                 UiMessageBox("Boot partition cannot be 0!");
//                 goto LinuxBootFailed;
//                 // return EINVAL;
//             }
//
//             /* Construct the corresponding ARC path */
//             ConstructArcPath(ArcPath, "", DriveNumber, PartitionNumber);
//             *strrchr(ArcPath, '\\') = ANSI_NULL; // Trim the trailing path separator.
//
//             BootPath = ArcPath;
//         }
//         else
//         {
//             /* Fall back to using the system partition as default path */
//             BootPath = GetArgumentValue(Argc, Argv, "SystemPartition");
//         }
//     }
//
//     /* Get the kernel name */
//     LinuxKernelName = GetArgumentValue(Argc, Argv, "Kernel");
//     if (!LinuxKernelName || !*LinuxKernelName)
//     {
//         UiMessageBox("Linux kernel filename not specified for selected OS!");
//         goto LinuxBootFailed;
//     }
//
//     /* Get the initrd name (optional) */
//     LinuxInitrdName = GetArgumentValue(Argc, Argv, "Initrd");
//
//     /* Get the command line (optional) */
//     LinuxCommandLineSize = 0;
//     LinuxCommandLine = GetArgumentValue(Argc, Argv, "CommandLine");
//     if (LinuxCommandLine && *LinuxCommandLine)
//     {
//         RemoveQuotes(LinuxCommandLine);
//         LinuxCommandLineSize = (ULONG)strlen(LinuxCommandLine) + 1;
//         LinuxCommandLineSize = min(LinuxCommandLineSize, 260);
//     }
//
//     /* Open the kernel */
//     Status = FsOpenFile(LinuxKernelName, BootPath, OpenReadOnly, &LinuxKernel);
//     if (Status != ESUCCESS)
//     {
//         UiMessageBox("Linux kernel '%s' not found.", LinuxKernelName);
//         goto LinuxBootFailed;
//     }
//
//     /* Open the initrd file image (if necessary) */
//     if (LinuxInitrdName)
//     {
//         Status = FsOpenFile(LinuxInitrdName, BootPath, OpenReadOnly, &LinuxInitrdFile);
//         if (Status != ESUCCESS)
//         {
//             UiMessageBox("Linux initrd image '%s' not found.", LinuxInitrdName);
//             goto LinuxBootFailed;
//         }
//     }
//
//     /* Load the boot sector */
//     if (!LinuxReadBootSector(LinuxKernel))
//         goto LinuxBootFailed;
//
//     /* Load the setup sector */
//     if (!LinuxReadSetupSector(LinuxKernel))
//         goto LinuxBootFailed;
//
//     /* Calc kernel size */
//     Status = ArcGetFileInformation(LinuxKernel, &FileInfo);
//     if (Status != ESUCCESS || FileInfo.EndingAddress.HighPart != 0)
//         LinuxKernelSize = 0;
//     else
//         LinuxKernelSize = FileInfo.EndingAddress.LowPart - (512 + SetupSectorSize);
//
//     /* Get the initrd file image (if necessary) */
//     LinuxInitrdSize = 0;
//     if (LinuxInitrdName)
//     {
//         Status = ArcGetFileInformation(LinuxInitrdFile, &FileInfo);
//         if (Status != ESUCCESS || FileInfo.EndingAddress.HighPart != 0)
//             LinuxInitrdSize = 0;
//         else
//             LinuxInitrdSize = FileInfo.EndingAddress.LowPart;
//     }
//
//     /* Load the kernel */
//     if (!LinuxReadKernel(LinuxKernel))
//         goto LinuxBootFailed;
//
//     /* Load the initrd (if necessary) */
//     if (LinuxInitrdName)
//     {
//         if (!LinuxReadInitrd(LinuxInitrdFile))
//             goto LinuxBootFailed;
//     }
//
//     /*
//      * If the default root device is set to FLOPPY (0000h), change to /dev/fd0 (0200h).
//      * For a list of majors, see
//      * https://github.com/torvalds/linux/blob/master/include/uapi/linux/major.h
//      * For some examples of (decoded) root devices values, see
//      * https://github.com/torvalds/linux/blob/cc89c63e2/include/linux/root_dev.h
//      *
//      * NOTE: The RootDevice field is deprecated since commits
//      * https://github.com/torvalds/linux/commit/079f85e62
//      * https://github.com/torvalds/linux/commit/7448e8e5d
//      */
// #define NODEV           0
// #define FLOPPY_MAJOR    2
// #define makedev(maj, min)   (((maj) << 8) | (min))
//     if (LinuxBootSector->RootDevice == NODEV)
//         LinuxBootSector->RootDevice = makedev(FLOPPY_MAJOR, 0);
//
//     if (LinuxSetupSector->Version >= 0x0202)
//     {
//         LinuxSetupSector->CommandLinePointer = 0x99000;
//     }
//     else
//     {
//         LinuxBootSector->CommandLineMagic = LINUX_COMMAND_LINE_MAGIC;
//         LinuxBootSector->CommandLineOffset = 0x9000;
//     }
//
//     if (NewStyleLinuxKernel)
//         LinuxSetupSector->TypeOfLoader = LINUX_LOADER_TYPE_FREELOADER;
//     else
//         LinuxSetupSector->LoadFlags = 0;
//
//     RtlCopyMemory((PVOID)0x90000, LinuxBootSector, 512);
//     RtlCopyMemory((PVOID)0x90200, LinuxSetupSector, SetupSectorSize);
//     RtlCopyMemory((PVOID)0x99000,
//                   LinuxCommandLine ? LinuxCommandLine : "",
//                   LinuxCommandLine ? LinuxCommandLineSize : sizeof(ANSI_NULL));
//
//     UiUnInitialize("Booting Linux...");
//     IniCleanup();
//
//     BootLinuxKernel(LinuxKernelSize, LinuxKernelLoadAddress,
//                     (LinuxSetupSector->LoadFlags & LINUX_FLAG_LOAD_HIGH)
//                         ? (PVOID)LINUX_KERNEL_LOAD_ADDRESS /* == 0x100000 */
//                         : (PVOID)0x10000);
//     /* Must not return! */
//     return ESUCCESS;
//
// LinuxBootFailed:
//
//     if (LinuxKernel)
//         ArcClose(LinuxKernel);
//
//     if (LinuxInitrdFile)
//         ArcClose(LinuxInitrdFile);
//
//     if (LinuxBootSector != NULL)
//         MmFreeMemory(LinuxBootSector);
//
//     if (LinuxSetupSector != NULL)
//         MmFreeMemory(LinuxSetupSector);
//
//     if (LinuxKernelLoadAddress != NULL)
//         MmFreeMemory(LinuxKernelLoadAddress);
//
//     if (LinuxInitrdLoadAddress != NULL)
//         MmFreeMemory(LinuxInitrdLoadAddress);
//
//     LinuxBootSector = NULL;
//     LinuxSetupSector = NULL;
//     SetupSectorSize = 0;
//     NewStyleLinuxKernel = FALSE;
//     LinuxKernelSize = 0;
//     LinuxInitrdSize = 0;
//     LinuxKernelName = NULL;
//     LinuxInitrdName = NULL;
//     LinuxCommandLine = NULL;
//     LinuxCommandLineSize = 0;
//     LinuxKernelLoadAddress = NULL;
//     LinuxInitrdLoadAddress = NULL;
//     *LinuxBootDescription = ANSI_NULL;
//
//     return ENOEXEC;
// }
//
// static BOOLEAN LinuxReadBootSector(ULONG LinuxKernelFile)
// {
//     LARGE_INTEGER Position;
//
//     /* Allocate memory for boot sector */
//     LinuxBootSector = MmAllocateMemoryWithType(512, LoaderSystemCode);
//     if (LinuxBootSector == NULL)
//         return FALSE;
//
//     /* Load the linux boot sector */
//     Position.QuadPart = 0;
//     if (ArcSeek(LinuxKernelFile, &Position, SeekAbsolute) != ESUCCESS)
//         return FALSE;
//     if (ArcRead(LinuxKernelFile, LinuxBootSector, 512, NULL) != ESUCCESS)
//         return FALSE;
//
//     /* Check for validity */
//     if (LinuxBootSector->BootFlag != LINUX_BOOT_SECTOR_MAGIC)
//     {
//         UiMessageBox("Invalid boot sector magic (0xaa55)");
//         return FALSE;
//     }
//
//     // DbgDumpBuffer(DPRINT_LINUX, LinuxBootSector, 512);
//
//     TRACE("SetupSectors: %d\n"  , LinuxBootSector->SetupSectors);
//     TRACE("RootFlags:    0x%x\n", LinuxBootSector->RootFlags);
//     TRACE("SystemSize:   0x%x\n", LinuxBootSector->SystemSize);
//     TRACE("SwapDevice:   0x%x\n", LinuxBootSector->SwapDevice);
//     TRACE("RamSize:      0x%x\n", LinuxBootSector->RamSize);
//     TRACE("VideoMode:    0x%x\n", LinuxBootSector->VideoMode);
//     TRACE("RootDevice:   0x%x\n", LinuxBootSector->RootDevice);
//     TRACE("BootFlag:     0x%x\n", LinuxBootSector->BootFlag);
//
//     return TRUE;
// }
//
// static BOOLEAN LinuxReadSetupSector(ULONG LinuxKernelFile)
// {
//     LARGE_INTEGER Position;
//     UCHAR TempLinuxSetupSector[512];
//
//     /* Load the first linux setup sector */
//     Position.QuadPart = 512;
//     if (ArcSeek(LinuxKernelFile, &Position, SeekAbsolute) != ESUCCESS)
//         return FALSE;
//     if (ArcRead(LinuxKernelFile, TempLinuxSetupSector, 512, NULL) != ESUCCESS)
//         return FALSE;
//
//     /* Check the kernel version */
//     LinuxSetupSector = (PLINUX_SETUPSECTOR)TempLinuxSetupSector;
//     if (!LinuxCheckKernelVersion())
//         return FALSE;
//
//     if (NewStyleLinuxKernel)
//         SetupSectorSize = 512 * LinuxBootSector->SetupSectors;
//     else
//         SetupSectorSize = 512 * 4; // Always 4 setup sectors
//
//     /* Allocate memory for setup sectors */
//     LinuxSetupSector = MmAllocateMemoryWithType(SetupSectorSize, LoaderSystemCode);
//     if (LinuxSetupSector == NULL)
//         return FALSE;
//
//     /* Copy over first setup sector */
//     RtlCopyMemory(LinuxSetupSector, TempLinuxSetupSector, 512);
//
//     /* Load the rest of the linux setup sectors */
//     Position.QuadPart = 1024;
//     if (ArcSeek(LinuxKernelFile, &Position, SeekAbsolute) != ESUCCESS)
//         return FALSE;
//     if (ArcRead(LinuxKernelFile, (PVOID)((ULONG_PTR)LinuxSetupSector + 512), SetupSectorSize - 512, NULL) != ESUCCESS)
//         return FALSE;
//
//     // DbgDumpBuffer(DPRINT_LINUX, LinuxSetupSector, SetupSectorSize);
//
//     TRACE("SetupHeaderSignature: 0x%x (HdrS)\n", LinuxSetupSector->SetupHeaderSignature);
//     TRACE("Version:              0x%x\n", LinuxSetupSector->Version);
//     TRACE("RealModeSwitch: 0x%x\n", LinuxSetupSector->RealModeSwitch);
//     TRACE("SetupSeg:       0x%x\n", LinuxSetupSector->SetupSeg);
//     TRACE("StartSystemSeg: 0x%x\n", LinuxSetupSector->StartSystemSeg);
//     TRACE("KernelVersion:  0x%x\n", LinuxSetupSector->KernelVersion);
//     TRACE("TypeOfLoader:   0x%x\n", LinuxSetupSector->TypeOfLoader);
//     TRACE("LoadFlags:      0x%x\n", LinuxSetupSector->LoadFlags);
//     TRACE("SetupMoveSize:  0x%x\n", LinuxSetupSector->SetupMoveSize);
//     TRACE("Code32Start:    0x%x\n", LinuxSetupSector->Code32Start);
//     TRACE("RamdiskAddress: 0x%x\n", LinuxSetupSector->RamdiskAddress);
//     TRACE("RamdiskSize:    0x%x\n", LinuxSetupSector->RamdiskSize);
//     TRACE("BootSectKludgeOffset:  0x%x\n", LinuxSetupSector->BootSectKludgeOffset);
//     TRACE("BootSectKludgeSegment: 0x%x\n", LinuxSetupSector->BootSectKludgeSegment);
//     TRACE("HeapEnd: 0x%x\n", LinuxSetupSector->HeapEnd);
//
//     return TRUE;
// }
//
// static BOOLEAN LinuxReadKernel(ULONG LinuxKernelFile)
// {
//     PVOID LoadAddress;
//     LARGE_INTEGER Position;
//     ULONG BytesLoaded;
//     CHAR  StatusText[260];
//
//     RtlStringCbPrintfA(StatusText, sizeof(StatusText), "Loading %s", LinuxKernelName);
//     UiDrawStatusText(StatusText);
//
//     /* Try to allocate memory for the Linux kernel; if it fails, allocate somewhere else */
//     LinuxKernelLoadAddress = MmAllocateMemoryAtAddress(LinuxKernelSize, (PVOID)LINUX_KERNEL_LOAD_ADDRESS, LoaderSystemCode);
//     if (LinuxKernelLoadAddress != (PVOID)LINUX_KERNEL_LOAD_ADDRESS)
//     {
//         /* It's OK, let's allocate again somewhere else */
//         LinuxKernelLoadAddress = MmAllocateMemoryWithType(LinuxKernelSize, LoaderSystemCode);
//         if (LinuxKernelLoadAddress == NULL)
//         {
//             TRACE("Failed to allocate 0x%lx bytes for the kernel image.\n", LinuxKernelSize);
//             return FALSE;
//         }
//     }
//
//     LoadAddress = LinuxKernelLoadAddress;
//
//     /* Load the linux kernel at 0x100000 (1mb) */
//     Position.QuadPart = 512 + SetupSectorSize;
//     if (ArcSeek(LinuxKernelFile, &Position, SeekAbsolute) != ESUCCESS)
//         return FALSE;
//     for (BytesLoaded = 0; BytesLoaded < LinuxKernelSize; )
//     {
//         if (ArcRead(LinuxKernelFile, LoadAddress, LINUX_READ_CHUNK_SIZE, NULL) != ESUCCESS)
//             return FALSE;
//
//         BytesLoaded += LINUX_READ_CHUNK_SIZE;
//         LoadAddress = (PVOID)((ULONG_PTR)LoadAddress + LINUX_READ_CHUNK_SIZE);
//
//         UiUpdateProgressBar(BytesLoaded * 100 / (LinuxKernelSize + LinuxInitrdSize), NULL);
//     }
//
//     return TRUE;
// }
//
// static BOOLEAN LinuxCheckKernelVersion(VOID)
// {
//     /* Just assume old kernel until we find otherwise */
//     NewStyleLinuxKernel = FALSE;
//
//     /* Check for new style setup header */
//     if (LinuxSetupSector->SetupHeaderSignature != LINUX_SETUP_HEADER_ID)
//     {
//         NewStyleLinuxKernel = FALSE;
//     }
//     /* Check for version below 2.0 */
//     else if (LinuxSetupSector->Version < 0x0200)
//     {
//         NewStyleLinuxKernel = FALSE;
//     }
//     /* Check for version 2.0 */
//     else if (LinuxSetupSector->Version == 0x0200)
//     {
//         NewStyleLinuxKernel = TRUE;
//     }
//     /* Check for version 2.01+ */
//     else if (LinuxSetupSector->Version >= 0x0201)
//     {
//         NewStyleLinuxKernel = TRUE;
//         LinuxSetupSector->HeapEnd = 0x9000;
//         LinuxSetupSector->LoadFlags |= LINUX_FLAG_CAN_USE_HEAP;
//     }
//
//     if ((NewStyleLinuxKernel == FALSE) && (LinuxInitrdName))
//     {
//         UiMessageBox("Error: Cannot load a ramdisk (initrd) with an old kernel image.");
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static BOOLEAN LinuxReadInitrd(ULONG LinuxInitrdFile)
// {
//     ULONG BytesLoaded;
//     CHAR  StatusText[260];
//
//     RtlStringCbPrintfA(StatusText, sizeof(StatusText), "Loading %s", LinuxInitrdName);
//     UiDrawStatusText(StatusText);
//
//     /* Allocate memory for the ramdisk, below 4GB */
//     // LinuxInitrdLoadAddress = MmAllocateMemory(LinuxInitrdSize);
//     /* Try to align it at the next MB boundary after the kernel */
//     // LinuxInitrdLoadAddress = MmAllocateMemoryAtAddress(LinuxInitrdSize, (PVOID)ROUND_UP((LINUX_KERNEL_LOAD_ADDRESS + LinuxKernelSize), 0x100000));
//     if (LinuxSetupSector->Version <= 0x0202)
//     {
// #ifdef _M_AMD64
//         C_ASSERT(LINUX_MAX_INITRD_ADDRESS < 0x100000000);
// #endif
//         LinuxInitrdLoadAddress = MmAllocateHighestMemoryBelowAddress(LinuxInitrdSize, (PVOID)LINUX_MAX_INITRD_ADDRESS, LoaderSystemCode);
//     }
//     else
//     {
//         LinuxInitrdLoadAddress = MmAllocateHighestMemoryBelowAddress(LinuxInitrdSize, UlongToPtr(LinuxSetupSector->InitrdAddressMax), LoaderSystemCode);
//     }
//     if (LinuxInitrdLoadAddress == NULL)
//     {
//         return FALSE;
//     }
// #ifdef _M_AMD64
//     ASSERT((ULONG_PTR)LinuxInitrdLoadAddress < 0x100000000);
// #endif
//
//     /* Set the information in the setup struct */
//     LinuxSetupSector->RamdiskAddress = PtrToUlong(LinuxInitrdLoadAddress);
//     LinuxSetupSector->RamdiskSize = LinuxInitrdSize;
//
//     TRACE("RamdiskAddress: 0x%x\n", LinuxSetupSector->RamdiskAddress);
//     TRACE("RamdiskSize:    0x%x\n", LinuxSetupSector->RamdiskSize);
//
//     if (LinuxSetupSector->Version >= 0x0203)
//     {
//         TRACE("InitrdAddressMax: 0x%x\n", LinuxSetupSector->InitrdAddressMax);
//     }
//
//     /* Load the ramdisk */
//     for (BytesLoaded = 0; BytesLoaded < LinuxInitrdSize; )
//     {
//         if (ArcRead(LinuxInitrdFile, LinuxInitrdLoadAddress, LINUX_READ_CHUNK_SIZE, NULL) != ESUCCESS)
//             return FALSE;
//
//         BytesLoaded += LINUX_READ_CHUNK_SIZE;
//         LinuxInitrdLoadAddress = (PVOID)((ULONG_PTR)LinuxInitrdLoadAddress + LINUX_READ_CHUNK_SIZE);
//
//         UiUpdateProgressBar((BytesLoaded + LinuxKernelSize) * 100 / (LinuxInitrdSize + LinuxKernelSize), NULL);
//     }
//
//     return TRUE;
// }
//
// #endif /* _M_IX86 || _M_AMD64 */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
