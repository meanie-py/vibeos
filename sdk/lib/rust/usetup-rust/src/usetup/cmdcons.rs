use crate::usetup::consup::CONSOLE_ClearScreen;
use crate::usetup::consup::CONSOLE_ConInKey;
use crate::usetup::consup::CONSOLE_ConOutChar;
use crate::usetup::consup::CONSOLE_ConOutPuts;
use crate::usetup::consup::CONSOLE_GetCursorX;
use crate::usetup::consup::CONSOLE_GetCursorXY;
use crate::usetup::consup::CONSOLE_GetCursorY;
use crate::usetup::consup::CONSOLE_SetCursorType;
use crate::usetup::consup::CONSOLE_SetCursorXY;
use crate::usetup::console::GetConsoleScreenBufferInfo;
use crate::usetup::console::GetStdHandle;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_27B3B5E195F7: [u16; 31] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0048, 0x0061,
    0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0025, 0x006C, 0x0075, 0x005C,
    0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0030,
    0x0000,
];

use core::ffi::c_void;

extern "C" {
    fn DbgPrint(format: *const i8, ...) -> i32;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut Commands: [COMMAND; 6] = unsafe {
    [
        tagCOMMAND {
            name: b"cls\0" as *const u8 as *const ::core::ffi::c_char as LPSTR,
            flags: 0 as INT,
            func: Some(CommandCls as unsafe extern "C" fn(PCONSOLE_STATE, LPSTR) -> INT),
            help: Some(HelpCls as unsafe extern "C" fn() -> ()),
        },
        tagCOMMAND {
            name: b"dumpsector\0" as *const u8 as *const ::core::ffi::c_char as LPSTR,
            flags: 0 as INT,
            func: Some(CommandDumpSector as unsafe extern "C" fn(PCONSOLE_STATE, LPSTR) -> INT),
            help: Some(HelpDumpSector as unsafe extern "C" fn() -> ()),
        },
        tagCOMMAND {
            name: b"exit\0" as *const u8 as *const ::core::ffi::c_char as LPSTR,
            flags: 0 as INT,
            func: Some(CommandExit as unsafe extern "C" fn(PCONSOLE_STATE, LPSTR) -> INT),
            help: Some(HelpExit as unsafe extern "C" fn() -> ()),
        },
        tagCOMMAND {
            name: b"help\0" as *const u8 as *const ::core::ffi::c_char as LPSTR,
            flags: 0 as INT,
            func: Some(CommandHelp as unsafe extern "C" fn(PCONSOLE_STATE, LPSTR) -> INT),
            help: Some(HelpHelp as unsafe extern "C" fn() -> ()),
        },
        tagCOMMAND {
            name: b"partinfo\0" as *const u8 as *const ::core::ffi::c_char as LPSTR,
            flags: 0 as INT,
            func: Some(CommandPartInfo as unsafe extern "C" fn(PCONSOLE_STATE, LPSTR) -> INT),
            help: Some(HelpPartInfo as unsafe extern "C" fn() -> ()),
        },
        tagCOMMAND {
            name: ::core::ptr::null::<CHAR>() as *mut CHAR,
            flags: 0 as INT,
            func: None,
            help: None,
        },
    ]
};


pub type LPCOMMAND = *mut tagCOMMAND;

unsafe extern "C" fn freep(mut p: *mut LPSTR) {
    let mut q: *mut LPSTR;

    if p.is_null() {
        return;
    }

    q = p;
    while !(*q).is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*q).cast());
        q = q.offset(1);
    }

    RtlFreeHeap(ProcessHeap, 0, (p).cast());
}

unsafe extern "C" fn StripQuotes(mut in_0: LPSTR) {
    let mut out: LPSTR = in_0;
    while *in_0 != 0 {
        if *in_0 as i32 != '"' as i32 {
            let fresh34 = out;
            out = out.offset(1);
            *fresh34 = *in_0;
        }
        in_0 = in_0.offset(1);
    }
    *out = '\0' as i32 as i8;
}
#[no_mangle]
pub unsafe extern "C" fn add_entry(
    ac: LPINT,
    arg: *mut *mut LPSTR,
    entry: LPCSTR,
) -> i32 {
    let mut q: LPSTR;
    let mut oldarg: *mut LPSTR;

    q = (RtlAllocateHeap(
        ProcessHeap,
        0,
        (strlen(entry) + 1) as SIZE_T,
    )).cast();
    if q.is_null() {
        return (FALSE) as i32;
    }

    strcpy(q, entry);
    oldarg = *arg;
    *arg = (RtlReAllocateHeap(
        ProcessHeap,
        0,
        (oldarg).cast(),
        ((*ac + 2) * (core::mem::size_of::<LPSTR>()) as i32) as SIZE_T,
    )).cast();
    if (*arg).is_null() {
        RtlFreeHeap(ProcessHeap, 0, (q).cast());
        *arg = oldarg;
        return (FALSE) as i32;
    }

    *(*arg).offset(*ac as isize) = q;
    *ac += 1;
    *(*arg).offset(*ac as isize) = core::ptr::null_mut();

    return (TRUE) as i32;
}

unsafe extern "C" fn split(mut s: LPSTR, mut args: LPINT) -> *mut LPSTR {
    let mut arg: *mut LPSTR = core::ptr::null_mut::<LPSTR>();
    let mut start: LPSTR = (core::ptr::null_mut::<u8>()).cast();
    let mut q: LPSTR = (core::ptr::null_mut::<u8>()).cast();
    let mut ac: i32 = 0;
    let mut len: INT_PTR = 0;
    let mut bQuoted: i32 = 0;

    arg = RtlAllocateHeap(ProcessHeap, 0, (core::mem::size_of::<LPTSTR>()) as SIZE_T) as *mut LPSTR;
    if arg.is_null() {
        return core::ptr::null_mut::<LPSTR>();
    }

    *arg = (core::ptr::null_mut::<u8>()).cast();

    ac = 0;
    while *s != 0 {
        bQuoted = (FALSE) as i32;

        while *s != 0 && (isspace(*s as i32) != 0 || iscntrl(*s as i32) != 0) {
            s = s.offset(1);
        }

        start = s;

        if *s as i32 == '/' as i32 {
            s = s.offset(1);
        }

        while isprint(*s as i32) != 0 {
            bQuoted ^= (*s as i32 == '"' as i32) as i32;

            if bQuoted == 0 {
                if isspace(*s as i32) != 0 || *s as i32 == '/' as i32 {
                    if s == start {
                        s = s.offset(1);
                    }
                    break;
                }
            }

            s = s.offset(1);
        }

        if s != start {
            len = s.offset_from(start) as INT_PTR;
            q = RtlAllocateHeap(ProcessHeap, 0, (len + 1) as SIZE_T) as LPSTR;
            if q.is_null() {
                freep(arg);
                return core::ptr::null_mut::<LPSTR>();
            }

            memcpy(q as *mut core::ffi::c_void, start as *const core::ffi::c_void, len as usize);
            *q.offset(len as isize) = ('\0' as i32 as u8) as i8;

            StripQuotes(q);

            if add_entry(&raw mut ac, &raw mut arg, q as LPCSTR) == 0 {
                RtlFreeHeap(ProcessHeap, 0, q as *mut core::ffi::c_void);
                freep(arg);
                return core::ptr::null_mut::<LPSTR>();
            }

            RtlFreeHeap(ProcessHeap, 0, q as *mut core::ffi::c_void);
        }
    }

    *args = ac;

    return arg;
}

unsafe extern "C" fn HelpCls() {
    CONSOLE_ConOutPrintf(b"CLS\n\nClears the screen.\n\n\0".as_ptr() as *const i8);
}

unsafe extern "C" fn CommandCls(
    State: PCONSOLE_STATE,
    param: LPSTR,
) -> INT {
    if strcmp(param, b"/?\0".as_ptr() as *const i8) == 0 {
        HelpCls();
        return 0;
    }

    CONSOLE_ClearScreen();
    CONSOLE_SetCursorXY(0, 0);

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn HexDump(buffer: *mut u8, size: u32) {
    let mut offset: u32 = 0;
    let mut ptr: *mut u8;

    while offset < (size & !15) {
        ptr = (buffer as usize + offset as usize) as *mut u8;
        CONSOLE_ConOutPrintf(
            b"%04lx  %02hx %02hx %02hx %02hx %02hx %02hx %02hx %02hx %02hx %02hx %02hx %02hx %02hx %02hx %02hx %02hx\n\0".as_ptr() as *const i8,
            offset,
            *ptr.add(0) as i32,
            *ptr.add(1) as i32,
            *ptr.add(2) as i32,
            *ptr.add(3) as i32,
            *ptr.add(4) as i32,
            *ptr.add(5) as i32,
            *ptr.add(6) as i32,
            *ptr.add(7) as i32,
            *ptr.add(8) as i32,
            *ptr.add(9) as i32,
            *ptr.add(10) as i32,
            *ptr.add(11) as i32,
            *ptr.add(12) as i32,
            *ptr.add(13) as i32,
            *ptr.add(14) as i32,
            *ptr.add(15) as i32,
        );
        offset += 16;
    }

    if offset < size {
        ptr = (buffer as usize + offset as usize) as *mut u8;
        CONSOLE_ConOutPrintf(
            b"%04lx \0".as_ptr() as *const i8,
            offset,
        );
        while offset < size {
            CONSOLE_ConOutPrintf(
                b" %02hx\0".as_ptr() as *const i8,
                *ptr as i32,
            );
            offset += 1;
            ptr = ptr.add(1);
        }

        CONSOLE_ConOutPrintf(
            b"\n\0".as_ptr() as *const i8,
        );
    }

    CONSOLE_ConOutPrintf(
        b"\n\0".as_ptr() as *const i8,
    );
}

unsafe extern "C" fn HelpDumpSector() {
    CONSOLE_ConOutPrintf(
        b"DUMPSECT DiskNumber Sector\n\nDumps a disk sector to the screen.\n\n\0".as_ptr() as LPCSTR,
    );
}

unsafe extern "C" fn CommandDumpSector(
    State: *mut CONSOLE_STATE,
    param: *mut i8,
) -> i32 {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut PathName: _UNICODE_STRING = core::mem::zeroed();
    let mut hDisk: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut DiskGeometry: DISK_GEOMETRY = core::mem::zeroed();
    let mut Status: i32 = 0;
    let mut argv: *mut *mut i8 = core::ptr::null_mut();
    let mut argc: i32 = 0;
    let mut DriveName: [u16; 40] = [0; 40];
    let mut ulDrive: u32 = 0;
    let mut Sector: LARGE_INTEGER = core::mem::zeroed();
    let mut SectorCount: LARGE_INTEGER = core::mem::zeroed();
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut Buffer: *mut u8 = core::ptr::null_mut();

    DPRINT1!(b"param: %s\n\0".as_ptr() as *const i8, param);

    if ((strcmp(param as *const i8, b"/?\0".as_ptr() as *const i8)) == 0) {
        HelpDumpSector();
        return 0;
    }

    argv = split(param, &mut argc) as *mut *mut i8;

    DPRINT1!(b"argc: %d\n\0".as_ptr() as *const i8, argc);
    DPRINT1!(b"argv: %p\n\0".as_ptr() as *const i8, argv);

    'done: {
        if argc != 2 {
            break 'done;
        }

        DPRINT1!(b"Device: %s\n\0".as_ptr() as *const i8, *argv.offset(0 as isize));
        DPRINT1!(b"Sector: %s\n\0".as_ptr() as *const i8, *argv.offset(1 as isize));

        ulDrive = strtoul(
            *argv.offset(0 as isize) as *const i8,
            core::ptr::null_mut::<*const i8>(),
            0,
        ) as u32;

        Sector.QuadPart = _atoi64(
            (*argv.offset(1 as isize) as *const i8).cast(),
        ) as i64;

        _swprintf(DriveName.as_mut_ptr(), RUSTCONV_WSTR_27B3B5E195F7.as_ptr(), ulDrive);

        RtlInitUnicodeString(&mut PathName, DriveName.as_ptr());

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>();
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_INHERIT;
        ObjectAttributes.ObjectName = &mut PathName;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>();

        Status = NtOpenFile(
            &mut hDisk,
            GENERIC_READ | SYNCHRONIZE,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            FILE_SHARE_READ,
            FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE | FILE_RANDOM_ACCESS,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(b"NtCreateFile failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        Status = NtDeviceIoControlFile(
            hDisk,
            core::ptr::null_mut::<core::ffi::c_void>(),
            None,
            core::ptr::null_mut::<core::ffi::c_void>(),
            &mut IoStatusBlock,
            IOCTL_DISK_GET_DRIVE_GEOMETRY,
            core::ptr::null_mut::<core::ffi::c_void>(),
            0,
            &mut DiskGeometry as *mut DISK_GEOMETRY as *mut core::ffi::c_void,
            core::mem::size_of::<DISK_GEOMETRY>() as u32,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(b"NtDeviceIoControlFile failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        DPRINT1!(b"Drive number: %lu\n\0".as_ptr() as *const i8, ulDrive);
        DPRINT1!(
            b"Cylinders: %I64u\nMediaType: %x\nTracksPerCylinder: %lu\nSectorsPerTrack: %lu\nBytesPerSector: %lu\n\n\0".as_ptr() as *const i8,
            DiskGeometry.Cylinders.QuadPart,
            DiskGeometry.MediaType as u32,
            DiskGeometry.TracksPerCylinder,
            DiskGeometry.SectorsPerTrack,
            DiskGeometry.BytesPerSector,
        );

        DPRINT1!(b"Sector: %I64u\n\0".as_ptr() as *const i8, Sector.QuadPart);

        SectorCount.QuadPart = (DiskGeometry.Cylinders.QuadPart as u64)
            .wrapping_mul(DiskGeometry.TracksPerCylinder as u64)
            .wrapping_mul(DiskGeometry.SectorsPerTrack as u64) as i64;

        if Sector.QuadPart >= SectorCount.QuadPart {
            CONSOLE_ConOutPrintf(
                b"Invalid sector number! Valid range: [0 - %I64u]\n\0".as_ptr() as *const i8,
                SectorCount.QuadPart - (1 as i64),
            );
            break 'done;
        }

        Buffer = RtlAllocateHeap(
            ProcessHeap as *mut core::ffi::c_void,
            0,
            (DiskGeometry.BytesPerSector) as SIZE_T,
        ) as *mut u8;

        if Buffer.is_null() {
            DPRINT1!(b"Buffer allocation failed\n\0".as_ptr() as *const i8);
            break 'done;
        }

        Offset.QuadPart = Sector.QuadPart * (DiskGeometry.BytesPerSector as i64);
        DPRINT1!(b"Offset: %I64u\n\0".as_ptr() as *const i8, Offset.QuadPart);

        Status = NtReadFile(
            hDisk,
            core::ptr::null_mut::<core::ffi::c_void>(),
            None,
            core::ptr::null_mut::<core::ffi::c_void>(),
            &mut IoStatusBlock,
            Buffer as *mut core::ffi::c_void,
            DiskGeometry.BytesPerSector as u32,
            &mut Offset,
            core::ptr::null_mut::<u32>(),
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(b"NtReadFile failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        HexDump(Buffer, DiskGeometry.BytesPerSector as u32);
    }

    if !Buffer.is_null() {
        RtlFreeHeap(
            ProcessHeap as *mut core::ffi::c_void,
            0,
            Buffer as *mut core::ffi::c_void,
        );
    }
    if !hDisk.is_null() {
        NtClose(hDisk);
    }
    freep(argv);

    return 0;
}

unsafe extern "C" fn HelpExit() {
    CONSOLE_ConOutPrintf(b"EXIT\n\nExits the repair console.\n\n\0".as_ptr() as LPCSTR);
}

unsafe extern "C" fn CommandExit(State: PCONSOLE_STATE, param: LPSTR) -> INT {
    if strcmp(param as *const i8, b"/?\0".as_ptr() as *const i8) == 0 {
        HelpExit();
        return 0;
    }

    (*State).bExit = (TRUE) as u8;

    return 0;
}

unsafe extern "C" fn HelpHelp() {
    CONSOLE_ConOutPrintf(
        b"HELP [Command]\n\nShows help on repair console commands.\n\n\0" as *const u8 as LPCSTR,
    );
}

unsafe extern "C" fn CommandHelp(
    State: PCONSOLE_STATE,
    param: LPSTR,
) -> INT {
    let mut cmdptr: LPCOMMAND;

    DPRINT1!(
        b"param: %p %u '%s'\n\0".as_ptr() as *const i8,
        param,
        strlen(param),
        param,
    );

    if ((strcmp(param, b"/?\0".as_ptr() as *const i8)) == 0) {
        HelpHelp();
        return 0;
    }

    if param != core::ptr::null_mut() && strlen(param) > 0 {
        cmdptr = Commands.as_mut_ptr();
        while (*cmdptr).name != core::ptr::null_mut() {
            if ((_stricmp(param, (*cmdptr).name)) == 0) {
                if (*cmdptr).help != None {
                    (((*cmdptr).help).unwrap_unchecked())();
                    return 0;
                }
            }
            cmdptr = cmdptr.offset(1);
        }
    }

    CONSOLE_ConOutPrintf(b"CLS\n\0".as_ptr() as *const i8);
    CONSOLE_ConOutPrintf(b"DUMPSECTOR\n\0".as_ptr() as *const i8);
    CONSOLE_ConOutPrintf(b"EXIT\n\0".as_ptr() as *const i8);
    CONSOLE_ConOutPrintf(b"HELP\n\0".as_ptr() as *const i8);
    CONSOLE_ConOutPrintf(b"\n\0".as_ptr() as *const i8);

    return 0;
}

unsafe extern "C" fn HelpPartInfo() {
    CONSOLE_ConOutPrintf(
        b"PARTINFO DiskNumber\n\nDumps a partition table to the screen.\n\n\0".as_ptr() as LPCSTR,
    );
}

unsafe extern "C" fn CommandPartInfo(
    State: *mut CONSOLE_STATE,
    param: LPSTR,
) -> i32 {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut PathName: _UNICODE_STRING = core::mem::zeroed();
    let mut hDisk: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut DiskGeometry: DISK_GEOMETRY = core::mem::zeroed();
    let mut Status: i32 = 0;
    let mut argv: *mut LPSTR = core::ptr::null_mut();
    let mut argc: i32 = 0;
    let mut DriveName: [u16; 40] = [0; 40];
    let mut ulDrive: u32 = 0;
    let mut i: u32 = 0;
    let mut LayoutBuffer: *mut DRIVE_LAYOUT_INFORMATION = core::ptr::null_mut();
    let mut PartitionInfo: *mut PARTITION_INFORMATION = core::ptr::null_mut();

    if DbgPrint(
        b"(%s:%d) param: %s\n\0".as_ptr() as *const i8,
        b"base/setup/usetup/cmdcons.c\0"
            .as_ptr() as *const i8,
        583 as i32,
        param,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"base/setup/usetup/cmdcons.c\0"
                .as_ptr() as *const i8,
            583 as i32,
        );
    }

    if strcmp(param as *const i8, b"/?\0".as_ptr() as *const i8) == 0 {
        HelpPartInfo();
        return 0;
    }

    argv = split(param, core::ptr::addr_of_mut!(argc));

    if DbgPrint(
        b"(%s:%d) argc: %d\n\0".as_ptr() as *const i8,
        b"base/setup/usetup/cmdcons.c\0"
            .as_ptr() as *const i8,
        593 as i32,
        argc,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"base/setup/usetup/cmdcons.c\0"
                .as_ptr() as *const i8,
            593 as i32,
        );
    }

    if DbgPrint(
        b"(%s:%d) argv: %p\n\0".as_ptr() as *const i8,
        b"base/setup/usetup/cmdcons.c\0"
            .as_ptr() as *const i8,
        594 as i32,
        argv,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"base/setup/usetup/cmdcons.c\0"
                .as_ptr() as *const i8,
            594 as i32,
        );
    }

    'done: {
        if argc != 1 {
            break 'done;
        }

        if DbgPrint(
            b"(%s:%d) Device: %s\n\0".as_ptr() as *const i8,
            b"base/setup/usetup/cmdcons.c\0"
                .as_ptr() as *const i8,
            601 as i32,
            *argv.offset(0 as isize) as *const i8,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"base/setup/usetup/cmdcons.c\0"
                    .as_ptr() as *const i8,
                601 as i32,
            );
        }

        ulDrive = strtoul(
            *argv.offset(0 as isize) as *const i8,
            core::ptr::null_mut(),
            0,
        ) as u32;

        _swprintf(
            DriveName.as_mut_ptr(),
            RUSTCONV_WSTR_27B3B5E195F7.as_ptr(),
            ulDrive,
        );

        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(PathName),
            DriveName.as_ptr() as *const u16,
        );

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes =
            (OBJ_CASE_INSENSITIVE | OBJ_INHERIT) as u32;
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(PathName);
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenFile(
            core::ptr::addr_of_mut!(hDisk),
            (GENERIC_READ | SYNCHRONIZE) as u32,
            core::ptr::addr_of_mut!(ObjectAttributes),
            core::ptr::addr_of_mut!(IoStatusBlock),
            FILE_SHARE_READ as u32,
            (FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE | FILE_RANDOM_ACCESS)
                as u32,
        );

        if !(Status >= 0) {
            if DbgPrint(
                b"(%s:%d) NtCreateFile failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                b"base/setup/usetup/cmdcons.c\0"
                    .as_ptr() as *const i8,
                625 as i32,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"base/setup/usetup/cmdcons.c\0"
                        .as_ptr() as *const i8,
                    625 as i32,
                );
            }
            break 'done;
        }

        Status = NtDeviceIoControlFile(
            hDisk,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(IoStatusBlock),
            IOCTL_DISK_GET_DRIVE_GEOMETRY as u32,
            core::ptr::null_mut(),
            0 as u32,
            core::ptr::addr_of_mut!(DiskGeometry) as *mut core::ffi::c_void,
            core::mem::size_of::<DISK_GEOMETRY>() as u32,
        );

        if !(Status >= 0) {
            if DbgPrint(
                b"(%s:%d) NtDeviceIoControlFile failed (Status 0x%08lx)\n\0"
                    .as_ptr() as *const i8,
                b"base/setup/usetup/cmdcons.c\0"
                    .as_ptr() as *const i8,
                641 as i32,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"base/setup/usetup/cmdcons.c\0"
                        .as_ptr() as *const i8,
                    641 as i32,
                );
            }
            break 'done;
        }

        CONSOLE_ConOutPrintf(
            b"Drive number: %lu\n\0".as_ptr() as *const i8,
            ulDrive,
        );

        CONSOLE_ConOutPrintf(
            b"Cylinders: %I64u\nMediaType: %x\nTracksPerCylinder: %lu\nSectorsPerTrack: %lu\nBytesPerSector: %lu\n\n\0"
                .as_ptr() as *const i8,
            DiskGeometry.Cylinders.QuadPart,
            DiskGeometry.MediaType as u32,
            DiskGeometry.TracksPerCylinder,
            DiskGeometry.SectorsPerTrack,
            DiskGeometry.BytesPerSector,
        );

        LayoutBuffer = RtlAllocateHeap(
            ProcessHeap as *mut core::ffi::c_void,
            HEAP_ZERO_MEMORY as u32,
            (8192) as SIZE_T,
        ) as *mut DRIVE_LAYOUT_INFORMATION;

        if LayoutBuffer.is_null() {
            if DbgPrint(
                b"(%s:%d) LayoutBuffer allocation failed\n\0".as_ptr() as *const i8,
                b"base/setup/usetup/cmdcons.c\0"
                    .as_ptr() as *const i8,
                659 as i32,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"base/setup/usetup/cmdcons.c\0"
                        .as_ptr() as *const i8,
                    659 as i32,
                );
            }
            break 'done;
        }

        Status = NtDeviceIoControlFile(
            hDisk,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(IoStatusBlock),
            IOCTL_DISK_GET_DRIVE_LAYOUT as u32,
            core::ptr::null_mut(),
            0 as u32,
            LayoutBuffer as *mut core::ffi::c_void,
            8192 as u32,
        );

        if !(Status >= 0) {
            if DbgPrint(
                b"(%s:%d) NtDeviceIoControlFile(IOCTL_DISK_GET_DRIVE_LAYOUT) failed (Status 0x%08lx)\n\0"
                    .as_ptr() as *const i8,
                b"base/setup/usetup/cmdcons.c\0"
                    .as_ptr() as *const i8,
                675 as i32,
                Status,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"base/setup/usetup/cmdcons.c\0"
                        .as_ptr() as *const i8,
                    675 as i32,
                );
            }
            break 'done;
        }

        CONSOLE_ConOutPrintf(
            b"Partitions: %lu  Signature: %lx\n\n\0".as_ptr() as *const i8,
            (*LayoutBuffer).PartitionCount,
            (*LayoutBuffer).Signature,
        );

        CONSOLE_ConOutPrintf(
            b" #            Start             Size        Hidden  Nr  Type  Boot\n\0"
                .as_ptr() as *const i8,
        );

        CONSOLE_ConOutPrintf(
            b"--  ---------------  ---------------  ------------  --  ----  ----\n\0"
                .as_ptr() as *const i8,
        );

        i = 0;
        while i < (*LayoutBuffer).PartitionCount {
            PartitionInfo = core::ptr::addr_of_mut!((*LayoutBuffer).PartitionEntry)
                .cast::<PARTITION_INFORMATION>()
                .offset(i as isize);

            CONSOLE_ConOutPrintf(
                b"%2lu  %15I64u  %15I64u  %12lu  %2lu    %2x    %c\n\0"
                    .as_ptr() as *const i8,
                i,
                (*PartitionInfo).StartingOffset.QuadPart
                    / DiskGeometry.BytesPerSector as i64,
                (*PartitionInfo).PartitionLength.QuadPart
                    / DiskGeometry.BytesPerSector as i64,
                (*PartitionInfo).HiddenSectors,
                (*PartitionInfo).PartitionNumber,
                (*PartitionInfo).PartitionType as u32,
                if (*PartitionInfo).BootIndicator != 0 {
                    b'*' as i32
                } else {
                    b' ' as i32
                },
            );

            i = i.wrapping_add(1);
        }

        CONSOLE_ConOutPrintf(b"\n\0".as_ptr() as *const i8);
    }

    if !LayoutBuffer.is_null() {
        RtlFreeHeap(
            ProcessHeap as *mut core::ffi::c_void,
            0 as u32,
            LayoutBuffer as *mut core::ffi::c_void,
        );
    }

    if !hDisk.is_null() {
        NtClose(hDisk);
    }

    freep(argv);

    return 0;
}

unsafe extern "C" fn ClearCommandLine(
    mut str: LPSTR,
    mut maxlen: INT,
    mut orgx: SHORT,
    mut orgy: SHORT,
) {
    let mut count: INT = 0;
    CONSOLE_SetCursorXY(orgx, orgy);
    count = 0;
    while count < strlen(str) as INT {
        CONSOLE_ConOutChar((' ') as CHAR);
        count += 1;
    }
    memset(str as *mut core::ffi::c_void, 0, maxlen as usize);
    CONSOLE_SetCursorXY(orgx, orgy);
}

unsafe extern "C" fn ReadCommand(
    mut State: PCONSOLE_STATE,
    mut str: LPSTR,
    mut maxlen: INT,
) -> BOOL {
    let mut orgx: SHORT = 0;
    let mut orgy: SHORT = 0;
    let mut curx: SHORT = 0;
    let mut cury: SHORT = 0;
    let mut tempscreen: SHORT = 0;
    let mut count: INT = 0;
    let mut current: INT = 0;
    let mut charcount: INT = 0;
    let mut ir: INPUT_RECORD = INPUT_RECORD {
        EventType: 0,
        Event: _INPUT_RECORD__bindgen_ty_1 {
            KeyEvent: _KEY_EVENT_RECORD {
                bKeyDown: 0,
                wRepeatCount: 0,
                wVirtualKeyCode: 0,
                wVirtualScanCode: 0,
                uChar: _KEY_EVENT_RECORD__bindgen_ty_1 { UnicodeChar: 0 },
                dwControlKeyState: 0,
            },
        },
    };
    let mut ch: CHAR = 0;
    let mut bReturn: BOOL = (FALSE) as i32;
    let mut bCharInput: BOOL = 0;

    CONSOLE_GetCursorXY(&mut orgx, &mut orgy);
    curx = orgx;
    cury = orgy;

    memset(
        str as *mut ::core::ffi::c_void,
        0,
        (maxlen as usize).wrapping_mul(::core::mem::size_of::<CHAR>() as usize),
    );

    CONSOLE_SetCursorType((*State).bInsert as BOOL, (TRUE) as BOOL);

    loop {
        bReturn = (FALSE) as i32;
        CONSOLE_ConInKey(&mut ir);

        if ir.Event.KeyEvent.dwControlKeyState
            & (RIGHT_ALT_PRESSED
                | LEFT_ALT_PRESSED
                | RIGHT_CTRL_PRESSED
                | LEFT_CTRL_PRESSED) as DWORD
            != 0
        {
            match ir.Event.KeyEvent.wVirtualKeyCode as ::core::ffi::c_int {
                _ => {}
            }
        }

        bCharInput = (FALSE) as i32;

        let mut current_block: u64;
        match (ir.Event.KeyEvent.wVirtualKeyCode as ::core::ffi::c_int) as u32 {
            VK_BACK => {
                if current > 0 && charcount > 0 {
                    if current == charcount {
                        *str.offset((current - 1) as isize) =
                            0x0000u16 as CHAR;
                        if CONSOLE_GetCursorX() != 0 {
                            CONSOLE_ConOutPrintf(b"\x08 \x08\0" as *const u8 as LPCSTR);
                            curx -= 1;
                        } else {
                            CONSOLE_SetCursorXY(
                                ((*State).maxx as INT - 1) as SHORT,
                                (CONSOLE_GetCursorY() - 1) as SHORT,
                            );
                            CONSOLE_ConOutChar(' ' as CHAR);
                            CONSOLE_SetCursorXY(
                                ((*State).maxx as INT - 1) as SHORT,
                                (CONSOLE_GetCursorY() - 1) as SHORT,
                            );
                            cury -= 1;
                            curx = ((*State).maxx as INT - 1) as SHORT;
                        }
                    } else {
                        count = current - 1;
                        while count < charcount {
                            *str.offset(count as isize) =
                                *str.offset((count + 1) as isize);
                            count += 1;
                        }
                        if CONSOLE_GetCursorX() != 0 {
                            CONSOLE_SetCursorXY(
                                (CONSOLE_GetCursorX() - 1) as SHORT,
                                CONSOLE_GetCursorY(),
                            );
                            curx -= 1;
                        } else {
                            CONSOLE_SetCursorXY(
                                ((*State).maxx as INT - 1) as SHORT,
                                (CONSOLE_GetCursorY() - 1) as SHORT,
                            );
                            cury -= 1;
                            curx = ((*State).maxx as INT - 1) as SHORT;
                        }
                        CONSOLE_GetCursorXY(&mut curx, &mut cury);
                        CONSOLE_ConOutPrintf(
                            b"%s \0" as *const u8 as LPCSTR,
                            str.offset((current - 1) as isize) as *mut CHAR,
                        );
                        CONSOLE_SetCursorXY(curx, cury);
                    }
                    charcount -= 1;
                    current -= 1;
                }
                current_block = 1209030638129645089;
            }
            VK_INSERT => {
                (*State).bInsert = ((*State).bInsert as INT ^ (TRUE) as i32) as BOOLEAN;
                CONSOLE_SetCursorType((*State).bInsert as BOOL, (TRUE) as BOOL);
                current_block = 1209030638129645089;
            }
            VK_DELETE => {
                if current != charcount && charcount > 0 {
                    count = current;
                    while count < charcount {
                        *str.offset(count as isize) =
                            *str.offset((count + 1) as isize);
                        count += 1;
                    }
                    charcount -= 1;
                    CONSOLE_GetCursorXY(&mut curx, &mut cury);
                    CONSOLE_ConOutPrintf(
                        b"%s \0" as *const u8 as LPCSTR,
                        str.offset(current as isize) as *mut CHAR,
                    );
                    CONSOLE_SetCursorXY(curx, cury);
                }
                current_block = 1209030638129645089;
            }
            VK_HOME => {
                if current != 0 {
                    CONSOLE_SetCursorXY(orgx, orgy);
                    curx = orgx;
                    cury = orgy;
                    current = 0;
                }
                current_block = 1209030638129645089;
            }
            VK_END => {
                if current != charcount {
                    CONSOLE_SetCursorXY(orgx, orgy);
                    CONSOLE_ConOutPrintf(b"%s\0" as *const u8 as LPCSTR, str);
                    CONSOLE_GetCursorXY(&mut curx, &mut cury);
                    current = charcount;
                }
                current_block = 1209030638129645089;
            }
            77 | 67 => {
                bCharInput = (TRUE) as i32;
                if ir.Event.KeyEvent.dwControlKeyState
                    & (RIGHT_CTRL_PRESSED | LEFT_CTRL_PRESSED) as DWORD
                    == 0
                {
                    current_block = 1209030638129645089;
                } else {
                    current_block = 2820783365403562632;
                }
            }
            VK_RETURN => {
                current_block = 2820783365403562632;
            }
            VK_ESCAPE => {
                ClearCommandLine(str, maxlen, orgx, orgy);
                curx = orgx;
                cury = orgy;
                charcount = 0;
                current = charcount;
                current_block = 1209030638129645089;
            }
            VK_UP => {
                current_block = 1209030638129645089;
            }
            VK_DOWN => {
                current_block = 1209030638129645089;
            }
            VK_LEFT => {
                if current > 0 {
                    current -= 1;
                    if CONSOLE_GetCursorX() == 0 {
                        CONSOLE_SetCursorXY(
                            ((*State).maxx as INT - 1) as SHORT,
                            (CONSOLE_GetCursorY() - 1) as SHORT,
                        );
                        curx = ((*State).maxx as INT - 1) as SHORT;
                        cury -= 1;
                    } else {
                        CONSOLE_SetCursorXY(
                            (CONSOLE_GetCursorX() - 1) as SHORT,
                            CONSOLE_GetCursorY(),
                        );
                        curx -= 1;
                    }
                }
                current_block = 1209030638129645089;
            }
            VK_RIGHT => {
                if current != charcount {
                    current += 1;
                    if (CONSOLE_GetCursorX()) as i32 == (*State).maxx as INT - 1 {
                        CONSOLE_SetCursorXY(
                            0,
                            (CONSOLE_GetCursorY() + 1) as SHORT,
                        );
                        curx = 0;
                        cury += 1;
                    } else {
                        CONSOLE_SetCursorXY(
                            (CONSOLE_GetCursorX() + 1) as SHORT,
                            CONSOLE_GetCursorY(),
                        );
                        curx += 1;
                    }
                }
                current_block = 1209030638129645089;
            }
            _ => {
                bCharInput = (TRUE) as i32;
                current_block = 1209030638129645089;
            }
        }

        match current_block {
            2820783365403562632 => {
                *str.offset(charcount as isize) = '\0' as CHAR;
                CONSOLE_ConOutChar('\n' as CHAR);
                bReturn = (TRUE) as i32;
            }
            _ => {}
        }

        ch = ir.Event.KeyEvent.uChar.UnicodeChar as CHAR;
        if ch as INT >= 32
            && charcount != maxlen - 2
            && bCharInput != 0
        {
            if (*State).bInsert as INT != 0 && current != charcount {
                tempscreen = (strlen(str.offset(current as isize) as *const i8)
                    as USHORT as INT
                    + curx as INT) as SHORT;
                if tempscreen as INT % (*State).maxx as INT
                    == (*State).maxx as INT - 1
                    && tempscreen as INT / (*State).maxx as INT + cury as INT
                        == (*State).maxy as INT - 1
                {
                    orgy -= 1;
                    cury -= 1;
                }
                count = charcount;
                while count > current {
                    *str.offset(count as isize) =
                        *str.offset((count - 1) as isize);
                    count -= 1;
                }
                let fresh = current;
                current = current + 1;
                *str.offset(fresh as isize) = ch;
                if curx as INT == (*State).maxx as INT - 1 {
                    curx = 0;
                    cury += 1;
                } else {
                    curx += 1;
                }
                CONSOLE_ConOutPrintf(
                    b"%s\0" as *const u8 as LPCSTR,
                    str.offset((current - 1) as isize) as *mut CHAR,
                );
                CONSOLE_SetCursorXY(curx, cury);
                charcount += 1;
            } else {
                if current == charcount {
                    charcount += 1;
                }
                let fresh = current;
                current = current + 1;
                *str.offset(fresh as isize) = ch;
                if (CONSOLE_GetCursorX()) as i32 == (*State).maxx as INT - 1
                    && (CONSOLE_GetCursorY()) as i32 == (*State).maxy as INT - 1
                {
                    orgy -= 1;
                    cury -= 1;
                }
                if (CONSOLE_GetCursorX()) as i32 == (*State).maxx as INT - 1 {
                    curx = 0;
                    cury += 1;
                } else {
                    curx += 1;
                }
                CONSOLE_ConOutChar(ch);
            }
        }

        if bReturn != 0 {
            break;
        }
    }

    CONSOLE_SetCursorType((*State).bInsert as BOOL, (TRUE) as BOOL);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn IsDelimiter(mut c: CHAR) -> BOOL {
    return (c as ::core::ffi::c_int == '/' as i32
        || c as ::core::ffi::c_int == '=' as i32
        || c as ::core::ffi::c_int == '\0' as i32
        || isspace(c as ::core::ffi::c_int) != 0) as ::core::ffi::c_int;
}

unsafe extern "C" fn DoCommand(
    State: PCONSOLE_STATE,
    line: LPSTR,
) {
    let mut com: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut cp: LPSTR = com.as_mut_ptr();
    let mut rest: LPSTR = line;
    let mut cmdptr: LPCOMMAND;

    DPRINT1!(b"DoCommand: ('%s')\n\0".as_ptr() as LPCSTR, line);

    while isspace(*rest as i32) != 0 {
        rest = rest.offset(1);
    }

    if *rest != 0 {
        while IsDelimiter(*rest) == 0 {
            let fresh_rest = rest;
            rest = rest.offset(1);
            let fresh_cp = cp;
            cp = cp.offset(1);
            *fresh_cp = tolower(*fresh_rest as i32) as CHAR;
        }

        *cp = 0;

        while isspace(*rest as i32) != 0 {
            rest = rest.offset(1);
        }

        cmdptr = Commands.as_mut_ptr();
        loop {
            if (*cmdptr).name.is_null() {
                CONSOLE_ConOutPuts(
                    b"Unknown command. Enter HELP to get a list of commands.\0".as_ptr() as LPCSTR,
                );
                break;
            }

            if _stricmp(
                com.as_mut_ptr(),
                (*cmdptr).name as *const i8,
            ) == 0
            {
                (((*cmdptr).func).unwrap_unchecked())(State, rest);
                break;
            }

            cmdptr = cmdptr.offset(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn RecoveryConsole() {
    let mut szInputBuffer: [i8; 256] = [0; 256];
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let mut State: CONSOLE_STATE = core::mem::zeroed();

    GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &mut csbi);

    State.maxx = csbi.dwSize.X;
    State.maxy = csbi.dwSize.Y;
    State.bInsert = (TRUE) as u8;
    State.bExit = (FALSE) as u8;

    CONSOLE_ClearScreen();
    CONSOLE_SetCursorXY(0, 0);

    CONSOLE_ConOutPrintf(b"ReactOS Recovery Console\n\nEnter HELP to get a list of commands.\n\n\0".as_ptr() as *const i8);

    while State.bExit == 0 {
        CONSOLE_ConOutPrintf(b">\0".as_ptr() as *const i8);

        ReadCommand(&mut State, szInputBuffer.as_mut_ptr(), 256);
        DPRINT1!(b"%s\n\0".as_ptr() as *const i8, szInputBuffer.as_ptr());

        DoCommand(&mut State, szInputBuffer.as_mut_ptr());
    }
}
