// custom.rs — Rust translation of boot/freeldr/freeldr/custom.c
// UEFI-only build (gated `#[cfg(feature = "uefi")]` in lib.rs); rosload's
// freeldr.lib link still uses custom.c (doesn't link libfreeldr_rust.a).
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCSTR, PSTR, TIMEINFO};
use core::ffi::c_void;
use crate::oslist::OperatingSystemItem;

// Raw `*const i8` is !Sync; wrap so each pub static can sit in static storage.
#[repr(transparent)]
pub struct PcstrSync(pub PCSTR);
unsafe impl Sync for PcstrSync {}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub static BootSectorFilePrompt: PcstrSync = PcstrSync(
    b"Enter the boot sector file path.\nLeave blank for booting a disk or partition.\n\nExamples:\n\\BOOTSECT.DOS\n/boot/bootsect.dos\0".as_ptr() as PCSTR,
);
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub static LinuxKernelPrompt: PcstrSync = PcstrSync(
    b"Enter the Linux kernel image path.\n\nExamples:\n/vmlinuz\n/boot/vmlinuz-2.4.18\0".as_ptr() as PCSTR,
);
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub static LinuxInitrdPrompt: PcstrSync = PcstrSync(
    b"Enter the initrd image path.\nLeave blank for no initial ramdisk.\n\nExamples:\n/initrd.gz\n/boot/root.img.gz\0".as_ptr() as PCSTR,
);
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub static LinuxCommandLinePrompt: PcstrSync = PcstrSync(
    b"Enter the Linux kernel command line.\n\nExamples:\nroot=/dev/hda1\nroot=/dev/fd0 read-only\nroot=/dev/sdb1 init=/sbin/init\0".as_ptr() as PCSTR,
);

pub static BootDrivePrompt: PcstrSync = PcstrSync(
    b"Enter the boot drive.\n\nExamples:\nfd0 - first floppy drive\nhd0 - first hard drive\nhd1 - second hard drive\ncd0 - first CD-ROM drive.\n\nBIOS drive numbers may also be used:\n0 - first floppy drive\n0x80 - first hard drive\n0x81 - second hard drive\0".as_ptr() as PCSTR,
);
pub static BootPartitionPrompt: PcstrSync = PcstrSync(
    b"Enter the boot partition.\n\0".as_ptr() as PCSTR,
);
pub static ARCPathPrompt: PcstrSync = PcstrSync(
    b"Enter the boot ARC path.\n\nExamples:\nmulti(0)disk(0)rdisk(0)partition(1)\nmulti(0)disk(0)fdisk(0)\0".as_ptr() as PCSTR,
);
pub static ReactOSSystemPathPrompt: PcstrSync = PcstrSync(
    b"Enter the path to your ReactOS system directory.\n\nExamples:\n\\REACTOS\n\\ROS\0".as_ptr() as PCSTR,
);
pub static ReactOSOptionsPrompt: PcstrSync = PcstrSync(
    b"Enter the load options you want passed to the kernel.\n\nExamples:\n/DEBUG /DEBUGPORT=COM1 /BAUDRATE=115200\n/FASTDETECT /SOS /NOGUIBOOT\n/BASEVIDEO /MAXMEM=64\n/KERNEL=NTKRNLMP.EXE /HAL=HALMPS.DLL\0".as_ptr() as PCSTR,
);
pub static ReactOSSetupOptionsPrompt: PcstrSync = PcstrSync(
    b"Enter additional load options you want passed to the ReactOS Setup.\nThese options will supplement those obtained from the TXTSETUP.SIF\nfile, unless you also specify the /SIFOPTIONSOVERRIDE option switch.\n\nExample:\n/DEBUG /DEBUGPORT=COM1 /BAUDRATE=115200 /NOGUIBOOT\0".as_ptr() as PCSTR,
);
pub static CustomBootPrompt: PcstrSync = PcstrSync(
    b"Press ENTER to boot your custom boot setup.\0".as_ptr() as PCSTR,
);

extern "C" {
    fn IniReadSettingByName(SectionId: ULONG_PTR, SettingName: PCSTR, Buffer: PCHAR, BufferSize: ULONG) -> BOOLEAN;
    fn IniModifySettingValue(SectionId: ULONG_PTR, SettingName: PCSTR, SettingValue: PCSTR) -> BOOLEAN;
    fn IniAddSection(SectionName: PCSTR, SectionId: *mut ULONG_PTR) -> BOOLEAN;
    fn IniAddSettingValueToSection(SectionId: ULONG_PTR, SettingName: PCSTR, SettingValue: PCSTR) -> BOOLEAN;
    fn UiMessageBox(MessageText: PCSTR);
    fn UiEditBox(MessageText: PCSTR, EditTextBuffer: PCHAR, Length: ULONG) -> BOOLEAN;
    fn UiDisplayMenu(
        MenuHeader: PCSTR, MenuFooter: PCSTR,
        MenuItemList: *mut PCSTR, MenuItemCount: ULONG,
        DefaultMenuItem: ULONG, MenuTimeOut: LONG,
        SelectedMenuItem: *mut ULONG, CanEscape: BOOLEAN,
        KeyPressFilter: Option<unsafe extern "C" fn(ULONG, ULONG, *mut c_void) -> BOOLEAN>,
        Context: *mut c_void,
    ) -> BOOLEAN;
    fn ArcGetTime() -> *mut TIMEINFO;
    fn RtlStringCbPrintfA(pszDest: PCHAR, cbDest: usize, pszFormat: PCSTR, ...) -> i32;
    fn ConstructArcPath(Path: PSTR, FileName: PCSTR, DriveNumber: u8, PartitionNumber: ULONG);
    fn DriveMapGetBiosDriveNumber(Name: PCSTR) -> u8;
    fn atoi(s: PCSTR) -> i32;
    fn strstr(haystack: PCSTR, needle: PCSTR) -> PCHAR;
    fn Reboot() -> !;

    // bootmgr.rs (Rust) — needed by OptionMenuCustomBoot tail.
    fn LoadOperatingSystem(OperatingSystem: *mut OperatingSystemItem);
}

// ----------------------------------------------------------------------------
// OptionMenuCustomBoot — custom.c's HAS_OPTION_MENU_CUSTOM_BOOT entry.
// Was silently dropped by the rustconv translation; reconstructed from C.
// ----------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn OptionMenuCustomBoot() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let mut CustomBootMenuList: [PCSTR; 4] = [
        b"Boot Sector (Disk/Partition/File)\0".as_ptr() as PCSTR,
        b"Linux\0".as_ptr() as PCSTR,
        b"ReactOS\0".as_ptr() as PCSTR,
        b"ReactOS Setup\0".as_ptr() as PCSTR,
    ];
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    let mut CustomBootMenuList: [PCSTR; 2] = [
        b"ReactOS\0".as_ptr() as PCSTR,
        b"ReactOS Setup\0".as_ptr() as PCSTR,
    ];

    let mut SelectedMenuItem: ULONG = 0;
    let mut OperatingSystem: OperatingSystemItem = core::mem::zeroed();

    if UiDisplayMenu(
        b"Please choose a boot method:\0".as_ptr() as PCSTR,
        core::ptr::null(),
        CustomBootMenuList.as_mut_ptr(),
        CustomBootMenuList.len() as ULONG,
        0,
        -1,
        core::ptr::addr_of_mut!(SelectedMenuItem),
        1 as BOOLEAN,
        None,
        core::ptr::null_mut(),
    ) == 0
    {
        // The user pressed ESC.
        return;
    }

    OperatingSystem.SectionId = 0;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    match SelectedMenuItem {
        0 => EditCustomBootSector(&mut OperatingSystem),
        1 => EditCustomBootLinux(&mut OperatingSystem),
        2 => EditCustomBootReactOS(&mut OperatingSystem, 0 as BOOLEAN),
        3 => EditCustomBootReactOS(&mut OperatingSystem, 1 as BOOLEAN),
        _ => {}
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    match SelectedMenuItem {
        0 => EditCustomBootReactOS(&mut OperatingSystem, 0 as BOOLEAN),
        1 => EditCustomBootReactOS(&mut OperatingSystem, 1 as BOOLEAN),
        _ => {}
    }

    // And boot it.
    if OperatingSystem.SectionId != 0 {
        UiMessageBox(CustomBootPrompt.0);
        LoadOperatingSystem(&mut OperatingSystem);
    }
}

// ----------------------------------------------------------------------------
// _M_IX86 || _M_AMD64 only: EditCustomBootSector / EditCustomBootLinux
// ----------------------------------------------------------------------------

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn EditCustomBootSector(OperatingSystem: *mut OperatingSystemItem) {
    let TimeInfo: *mut TIMEINFO;
    let mut SectionId: ULONG_PTR = (*OperatingSystem).SectionId;
    let mut SectionName: [CHAR; 100] = [0; 100];

    // The C source uses one union { CHAR Drive[20]; CHAR Partition[20]; CHAR ArcPath[200] }
    // accessed as both individual fields and the combined ArcPath. We mirror
    // the layout via a #[repr(C)] union (300 bytes, sized for ArcPath).
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct BootStringsNamed {
        Guard1: CHAR,
        Drive: [CHAR; 20],
        Partition: [CHAR; 20],
        Guard2: CHAR,
    }
    #[repr(C)]
    union BootStringsUnion {
        named: BootStringsNamed,
        ArcPath: [CHAR; 200],
    }

    let mut BootStrings: BootStringsUnion = core::mem::zeroed();
    let mut BootSectorFile: [CHAR; 200] = [0; 200];

    if SectionId != 0 {
        BootStrings.ArcPath[0] = 0;
        IniReadSettingByName(
            SectionId,
            b"BootPath\0".as_ptr() as PCSTR,
            BootStrings.ArcPath.as_mut_ptr(),
            core::mem::size_of_val(&BootStrings.ArcPath) as ULONG,
        );
        if BootStrings.ArcPath[0] == 0 {
            IniReadSettingByName(
                SectionId,
                b"BootDrive\0".as_ptr() as PCSTR,
                BootStrings.named.Drive.as_mut_ptr(),
                core::mem::size_of_val(&BootStrings.named.Drive) as ULONG,
            );
            IniReadSettingByName(
                SectionId,
                b"BootPartition\0".as_ptr() as PCSTR,
                BootStrings.named.Partition.as_mut_ptr(),
                core::mem::size_of_val(&BootStrings.named.Partition) as ULONG,
            );
        }
        IniReadSettingByName(
            SectionId,
            b"BootSectorFile\0".as_ptr() as PCSTR,
            BootSectorFile.as_mut_ptr(),
            core::mem::size_of_val(&BootSectorFile) as ULONG,
        );
    }

    if BootStrings.ArcPath[0] == 0 {
        if UiEditBox(
            BootDrivePrompt.0,
            BootStrings.named.Drive.as_mut_ptr(),
            core::mem::size_of_val(&BootStrings.named.Drive) as ULONG,
        ) == 0
        {
            return;
        }
        if BootStrings.named.Drive[0] != 0 {
            if UiEditBox(
                BootPartitionPrompt.0,
                BootStrings.named.Partition.as_mut_ptr(),
                core::mem::size_of_val(&BootStrings.named.Partition) as ULONG,
            ) == 0
            {
                return;
            }
        }
    }
    if BootStrings.named.Drive[0] == 0 {
        if UiEditBox(
            ARCPathPrompt.0,
            BootStrings.ArcPath.as_mut_ptr(),
            core::mem::size_of_val(&BootStrings.ArcPath) as ULONG,
        ) == 0
        {
            return;
        }
    }

    if (BootStrings.ArcPath[0] == 0 && atoi(BootStrings.named.Partition.as_ptr() as PCSTR) != 0)
        || (BootStrings.ArcPath[0] != 0
            && strstr(BootStrings.ArcPath.as_ptr() as PCSTR, b")partition()\0".as_ptr() as PCSTR)
                .is_null()
            && strstr(
                BootStrings.ArcPath.as_ptr() as PCSTR,
                b")partition(0)\0".as_ptr() as PCSTR,
            )
            .is_null())
    {
        if UiEditBox(
            BootSectorFilePrompt.0,
            BootSectorFile.as_mut_ptr(),
            core::mem::size_of_val(&BootSectorFile) as ULONG,
        ) == 0
        {
            return;
        }
    }

    if SectionId != 0 {
        if BootStrings.ArcPath[0] != 0 {
            IniModifySettingValue(
                SectionId,
                b"BootPath\0".as_ptr() as PCSTR,
                BootStrings.ArcPath.as_ptr() as PCSTR,
            );
        } else if BootStrings.named.Drive[0] != 0 {
            IniModifySettingValue(
                SectionId,
                b"BootDrive\0".as_ptr() as PCSTR,
                BootStrings.named.Drive.as_ptr() as PCSTR,
            );
            IniModifySettingValue(
                SectionId,
                b"BootPartition\0".as_ptr() as PCSTR,
                BootStrings.named.Partition.as_ptr() as PCSTR,
            );
        } else {
            IniModifySettingValue(
                SectionId,
                b"BootPath\0".as_ptr() as PCSTR,
                b"\0".as_ptr() as PCSTR,
            );
            IniModifySettingValue(
                SectionId,
                b"BootDrive\0".as_ptr() as PCSTR,
                b"\0".as_ptr() as PCSTR,
            );
            IniModifySettingValue(
                SectionId,
                b"BootPartition\0".as_ptr() as PCSTR,
                b"\0".as_ptr() as PCSTR,
            );
        }
        IniModifySettingValue(
            SectionId,
            b"BootSectorFile\0".as_ptr() as PCSTR,
            BootSectorFile.as_ptr() as PCSTR,
        );
        return;
    }

    TimeInfo = ArcGetTime();
    RtlStringCbPrintfA(
        SectionName.as_mut_ptr(),
        core::mem::size_of_val(&SectionName),
        b"CustomBootSector%u%u%u%u%u%u\0".as_ptr() as PCSTR,
        (*TimeInfo).Year as u32,
        (*TimeInfo).Day as u32,
        (*TimeInfo).Month as u32,
        (*TimeInfo).Hour as u32,
        (*TimeInfo).Minute as u32,
        (*TimeInfo).Second as u32,
    );

    if IniAddSection(
        SectionName.as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(SectionId),
    ) == 0
    {
        return;
    }
    if IniAddSettingValueToSection(
        SectionId,
        b"BootType\0".as_ptr() as PCSTR,
        b"BootSector\0".as_ptr() as PCSTR,
    ) == 0
    {
        return;
    }

    if BootStrings.ArcPath[0] != 0 {
        if IniAddSettingValueToSection(
            SectionId,
            b"BootPath\0".as_ptr() as PCSTR,
            BootStrings.ArcPath.as_ptr() as PCSTR,
        ) == 0
        {
            return;
        }
    } else if BootStrings.named.Drive[0] != 0 {
        if IniAddSettingValueToSection(
            SectionId,
            b"BootDrive\0".as_ptr() as PCSTR,
            BootStrings.named.Drive.as_ptr() as PCSTR,
        ) == 0
        {
            return;
        }
        if IniAddSettingValueToSection(
            SectionId,
            b"BootPartition\0".as_ptr() as PCSTR,
            BootStrings.named.Partition.as_ptr() as PCSTR,
        ) == 0
        {
            return;
        }
    }

    if BootSectorFile[0] != 0 {
        if IniAddSettingValueToSection(
            SectionId,
            b"BootSectorFile\0".as_ptr() as PCSTR,
            BootSectorFile.as_ptr() as PCSTR,
        ) == 0
        {
            return;
        }
    }

    (*OperatingSystem).SectionId = SectionId;
    (*OperatingSystem).LoadIdentifier = core::ptr::null();
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn EditCustomBootLinux(OperatingSystem: *mut OperatingSystemItem) {
    let TimeInfo: *mut TIMEINFO;
    let mut SectionId: ULONG_PTR = (*OperatingSystem).SectionId;
    let mut SectionName: [CHAR; 100] = [0; 100];

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct BootStringsNamed {
        Guard1: CHAR,
        Drive: [CHAR; 20],
        Partition: [CHAR; 20],
        Guard2: CHAR,
    }
    #[repr(C)]
    union BootStringsUnion {
        named: BootStringsNamed,
        ArcPath: [CHAR; 200],
    }

    let mut BootStrings: BootStringsUnion = core::mem::zeroed();
    let mut LinuxKernelString: [CHAR; 200] = [0; 200];
    let mut LinuxInitrdString: [CHAR; 200] = [0; 200];
    let mut LinuxCommandLineString: [CHAR; 200] = [0; 200];

    if SectionId != 0 {
        BootStrings.ArcPath[0] = 0;
        IniReadSettingByName(
            SectionId,
            b"BootPath\0".as_ptr() as PCSTR,
            BootStrings.ArcPath.as_mut_ptr(),
            core::mem::size_of_val(&BootStrings.ArcPath) as ULONG,
        );
        if BootStrings.ArcPath[0] == 0 {
            IniReadSettingByName(
                SectionId,
                b"BootDrive\0".as_ptr() as PCSTR,
                BootStrings.named.Drive.as_mut_ptr(),
                core::mem::size_of_val(&BootStrings.named.Drive) as ULONG,
            );
            IniReadSettingByName(
                SectionId,
                b"BootPartition\0".as_ptr() as PCSTR,
                BootStrings.named.Partition.as_mut_ptr(),
                core::mem::size_of_val(&BootStrings.named.Partition) as ULONG,
            );
        }
        IniReadSettingByName(
            SectionId,
            b"Kernel\0".as_ptr() as PCSTR,
            LinuxKernelString.as_mut_ptr(),
            core::mem::size_of_val(&LinuxKernelString) as ULONG,
        );
        IniReadSettingByName(
            SectionId,
            b"Initrd\0".as_ptr() as PCSTR,
            LinuxInitrdString.as_mut_ptr(),
            core::mem::size_of_val(&LinuxInitrdString) as ULONG,
        );
        IniReadSettingByName(
            SectionId,
            b"CommandLine\0".as_ptr() as PCSTR,
            LinuxCommandLineString.as_mut_ptr(),
            core::mem::size_of_val(&LinuxCommandLineString) as ULONG,
        );
    }

    if BootStrings.ArcPath[0] == 0 {
        if UiEditBox(
            BootDrivePrompt.0,
            BootStrings.named.Drive.as_mut_ptr(),
            core::mem::size_of_val(&BootStrings.named.Drive) as ULONG,
        ) == 0
        {
            return;
        }
        if BootStrings.named.Drive[0] != 0 {
            if UiEditBox(
                BootPartitionPrompt.0,
                BootStrings.named.Partition.as_mut_ptr(),
                core::mem::size_of_val(&BootStrings.named.Partition) as ULONG,
            ) == 0
            {
                return;
            }
        }
    }
    if BootStrings.named.Drive[0] == 0 {
        if UiEditBox(
            ARCPathPrompt.0,
            BootStrings.ArcPath.as_mut_ptr(),
            core::mem::size_of_val(&BootStrings.ArcPath) as ULONG,
        ) == 0
        {
            return;
        }
    }

    if UiEditBox(
        LinuxKernelPrompt.0,
        LinuxKernelString.as_mut_ptr(),
        core::mem::size_of_val(&LinuxKernelString) as ULONG,
    ) == 0
    {
        return;
    }
    if UiEditBox(
        LinuxInitrdPrompt.0,
        LinuxInitrdString.as_mut_ptr(),
        core::mem::size_of_val(&LinuxInitrdString) as ULONG,
    ) == 0
    {
        return;
    }
    if UiEditBox(
        LinuxCommandLinePrompt.0,
        LinuxCommandLineString.as_mut_ptr(),
        core::mem::size_of_val(&LinuxCommandLineString) as ULONG,
    ) == 0
    {
        return;
    }

    if SectionId != 0 {
        if BootStrings.ArcPath[0] != 0 {
            IniModifySettingValue(
                SectionId,
                b"BootPath\0".as_ptr() as PCSTR,
                BootStrings.ArcPath.as_ptr() as PCSTR,
            );
        } else if BootStrings.named.Drive[0] != 0 {
            IniModifySettingValue(
                SectionId,
                b"BootDrive\0".as_ptr() as PCSTR,
                BootStrings.named.Drive.as_ptr() as PCSTR,
            );
            IniModifySettingValue(
                SectionId,
                b"BootPartition\0".as_ptr() as PCSTR,
                BootStrings.named.Partition.as_ptr() as PCSTR,
            );
        } else {
            IniModifySettingValue(
                SectionId,
                b"BootPath\0".as_ptr() as PCSTR,
                b"\0".as_ptr() as PCSTR,
            );
            IniModifySettingValue(
                SectionId,
                b"BootDrive\0".as_ptr() as PCSTR,
                b"\0".as_ptr() as PCSTR,
            );
            IniModifySettingValue(
                SectionId,
                b"BootPartition\0".as_ptr() as PCSTR,
                b"\0".as_ptr() as PCSTR,
            );
        }
        IniModifySettingValue(
            SectionId,
            b"Kernel\0".as_ptr() as PCSTR,
            LinuxKernelString.as_ptr() as PCSTR,
        );
        IniModifySettingValue(
            SectionId,
            b"Initrd\0".as_ptr() as PCSTR,
            LinuxInitrdString.as_ptr() as PCSTR,
        );
        IniModifySettingValue(
            SectionId,
            b"CommandLine\0".as_ptr() as PCSTR,
            LinuxCommandLineString.as_ptr() as PCSTR,
        );
        return;
    }

    TimeInfo = ArcGetTime();
    RtlStringCbPrintfA(
        SectionName.as_mut_ptr(),
        core::mem::size_of_val(&SectionName),
        b"CustomLinux%u%u%u%u%u%u\0".as_ptr() as PCSTR,
        (*TimeInfo).Year as u32,
        (*TimeInfo).Day as u32,
        (*TimeInfo).Month as u32,
        (*TimeInfo).Hour as u32,
        (*TimeInfo).Minute as u32,
        (*TimeInfo).Second as u32,
    );

    if IniAddSection(
        SectionName.as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(SectionId),
    ) == 0
    {
        return;
    }
    if IniAddSettingValueToSection(
        SectionId,
        b"BootType\0".as_ptr() as PCSTR,
        b"Linux\0".as_ptr() as PCSTR,
    ) == 0
    {
        return;
    }

    if BootStrings.ArcPath[0] != 0 {
        if IniAddSettingValueToSection(
            SectionId,
            b"BootPath\0".as_ptr() as PCSTR,
            BootStrings.ArcPath.as_ptr() as PCSTR,
        ) == 0
        {
            return;
        }
    } else if BootStrings.named.Drive[0] != 0 {
        if IniAddSettingValueToSection(
            SectionId,
            b"BootDrive\0".as_ptr() as PCSTR,
            BootStrings.named.Drive.as_ptr() as PCSTR,
        ) == 0
        {
            return;
        }
        if IniAddSettingValueToSection(
            SectionId,
            b"BootPartition\0".as_ptr() as PCSTR,
            BootStrings.named.Partition.as_ptr() as PCSTR,
        ) == 0
        {
            return;
        }
    }

    if IniAddSettingValueToSection(
        SectionId,
        b"Kernel\0".as_ptr() as PCSTR,
        LinuxKernelString.as_ptr() as PCSTR,
    ) == 0
    {
        return;
    }
    if LinuxInitrdString[0] != 0 {
        if IniAddSettingValueToSection(
            SectionId,
            b"Initrd\0".as_ptr() as PCSTR,
            LinuxInitrdString.as_ptr() as PCSTR,
        ) == 0
        {
            return;
        }
    }
    if IniAddSettingValueToSection(
        SectionId,
        b"CommandLine\0".as_ptr() as PCSTR,
        LinuxCommandLineString.as_ptr() as PCSTR,
    ) == 0
    {
        return;
    }

    (*OperatingSystem).SectionId = SectionId;
    (*OperatingSystem).LoadIdentifier = b"Custom Linux Setup\0".as_ptr() as PCSTR;
}

// ----------------------------------------------------------------------------
// EditCustomBootReactOS — all arches.
// ----------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn EditCustomBootReactOS(
    OperatingSystem: *mut OperatingSystemItem,
    IsSetup: BOOLEAN,
) {
    let TimeInfo: *mut TIMEINFO;
    let mut SectionId: ULONG_PTR = (*OperatingSystem).SectionId;
    let mut SectionName: [CHAR; 100] = [0; 100];
    let mut BootDriveString: [CHAR; 20] = [0; 20];
    let mut BootPartitionString: [CHAR; 20] = [0; 20];
    let mut ReactOSSystemPath: [CHAR; 200] = [0; 200];
    let mut ReactOSARCPath: [CHAR; 200] = [0; 200];
    let mut ReactOSOptions: [CHAR; 200] = [0; 200];

    if SectionId != 0 {
        IniReadSettingByName(
            SectionId,
            b"SystemPath\0".as_ptr() as PCSTR,
            ReactOSARCPath.as_mut_ptr(),
            core::mem::size_of_val(&ReactOSARCPath) as ULONG,
        );
        IniReadSettingByName(
            SectionId,
            b"Options\0".as_ptr() as PCSTR,
            ReactOSOptions.as_mut_ptr(),
            core::mem::size_of_val(&ReactOSOptions) as ULONG,
        );
    }

    if SectionId == 0 {
        if UiEditBox(
            BootDrivePrompt.0,
            BootDriveString.as_mut_ptr(),
            core::mem::size_of_val(&BootDriveString) as ULONG,
        ) == 0
        {
            return;
        }
        if UiEditBox(
            BootPartitionPrompt.0,
            BootPartitionString.as_mut_ptr(),
            core::mem::size_of_val(&BootPartitionString) as ULONG,
        ) == 0
        {
            return;
        }
        if UiEditBox(
            ReactOSSystemPathPrompt.0,
            ReactOSSystemPath.as_mut_ptr(),
            core::mem::size_of_val(&ReactOSSystemPath) as ULONG,
        ) == 0
        {
            return;
        }
    } else {
        if UiEditBox(
            ReactOSSystemPathPrompt.0,
            ReactOSARCPath.as_mut_ptr(),
            core::mem::size_of_val(&ReactOSARCPath) as ULONG,
        ) == 0
        {
            return;
        }
    }

    let OptionsPrompt = if IsSetup != 0 {
        ReactOSSetupOptionsPrompt.0
    } else {
        ReactOSOptionsPrompt.0
    };
    if UiEditBox(
        OptionsPrompt,
        ReactOSOptions.as_mut_ptr(),
        core::mem::size_of_val(&ReactOSOptions) as ULONG,
    ) == 0
    {
        return;
    }

    if SectionId != 0 {
        IniModifySettingValue(
            SectionId,
            b"SystemPath\0".as_ptr() as PCSTR,
            ReactOSARCPath.as_ptr() as PCSTR,
        );
        IniModifySettingValue(
            SectionId,
            b"Options\0".as_ptr() as PCSTR,
            ReactOSOptions.as_ptr() as PCSTR,
        );
        return;
    }

    TimeInfo = ArcGetTime();
    RtlStringCbPrintfA(
        SectionName.as_mut_ptr(),
        core::mem::size_of_val(&SectionName),
        b"CustomReactOS%u%u%u%u%u%u\0".as_ptr() as PCSTR,
        (*TimeInfo).Year as u32,
        (*TimeInfo).Day as u32,
        (*TimeInfo).Month as u32,
        (*TimeInfo).Hour as u32,
        (*TimeInfo).Minute as u32,
        (*TimeInfo).Second as u32,
    );

    if IniAddSection(
        SectionName.as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(SectionId),
    ) == 0
    {
        return;
    }
    let BootTypeValue: PCSTR = if IsSetup != 0 {
        b"ReactOSSetup\0".as_ptr() as PCSTR
    } else {
        b"Windows2003\0".as_ptr() as PCSTR
    };
    if IniAddSettingValueToSection(SectionId, b"BootType\0".as_ptr() as PCSTR, BootTypeValue) == 0
    {
        return;
    }

    ConstructArcPath(
        ReactOSARCPath.as_mut_ptr(),
        ReactOSSystemPath.as_ptr() as PCSTR,
        DriveMapGetBiosDriveNumber(BootDriveString.as_ptr() as PCSTR),
        atoi(BootPartitionString.as_ptr() as PCSTR) as ULONG,
    );

    if IniAddSettingValueToSection(
        SectionId,
        b"SystemPath\0".as_ptr() as PCSTR,
        ReactOSARCPath.as_ptr() as PCSTR,
    ) == 0
    {
        return;
    }
    if IniAddSettingValueToSection(
        SectionId,
        b"Options\0".as_ptr() as PCSTR,
        ReactOSOptions.as_ptr() as PCSTR,
    ) == 0
    {
        return;
    }

    (*OperatingSystem).SectionId = SectionId;
    (*OperatingSystem).LoadIdentifier = core::ptr::null();
}

#[no_mangle]
pub unsafe extern "C" fn OptionMenuReboot() {
    UiMessageBox(b"The system will now reboot.\0".as_ptr() as PCSTR);
    Reboot();
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// (original C source body — see boot/freeldr/freeldr/custom.c)
