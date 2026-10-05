// bootmgr.rs — Rust translation of boot/freeldr/freeldr/bootmgr.c
// UEFI-only build (gated `#[cfg(feature = "uefi")]` in lib.rs); rosload's
// freeldr.lib link still uses bootmgr.c (doesn't link libfreeldr_rust.a).
//
// Note: UEFIBOOT is always defined for this Rust copy (uefildr.efi),
// so OSLoadingMethods omits BootSector/Linux on i386/amd64.
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCCHAR, PCSTR, PSTR};
use core::ffi::c_void;
use crate::oslist::OperatingSystemItem;
use crate::ffi::ARC_STATUS;

// ARC_ENTRY_POINT from arcsupp.h: ARC_STATUS (__cdecl *)(ULONG, PCHAR[], PCHAR[])
// EDIT_OS_ENTRY_PROC from bootmgr.c: VOID (*)(OperatingSystemItem*)
pub type ARC_ENTRY_POINT = Option<
    unsafe extern "C" fn(Argc: ULONG, Argv: *mut PSTR, Envp: *mut PSTR) -> ARC_STATUS,
>;
pub type EDIT_OS_ENTRY_PROC = Option<
    unsafe extern "C" fn(OperatingSystem: *mut OperatingSystemItem),
>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OS_LOADING_METHOD {
    pub BootType: PCSTR,
    pub EditOsEntry: EDIT_OS_ENTRY_PROC,
    pub OsLoader: ARC_ENTRY_POINT,
}
pub type POS_LOADING_METHOD = *mut OS_LOADING_METHOD;

// PCSTR (= *const i8) is !Sync; wrap the table.
#[repr(transparent)]
pub struct OsLoadingMethodsTable<const N: usize>(pub [OS_LOADING_METHOD; N]);
unsafe impl<const N: usize> Sync for OsLoadingMethodsTable<N> {}

extern "C" {
    // freeldr-internal — defined in custom.rs / setupldr.rs / winldr.rs
    pub fn EditCustomBootReactOS(OperatingSystem: *mut OperatingSystemItem, IsSetup: BOOLEAN);
    pub fn LoadReactOSSetup(Argc: ULONG, Argv: *mut PSTR, Envp: *mut PSTR) -> ARC_STATUS;
    pub fn LoadAndBootWindows(Argc: ULONG, Argv: *mut PSTR, Envp: *mut PSTR) -> ARC_STATUS;

    // freeldr-internal — ini/heap/path/section/settings/UI surfaces.
    fn FrLdrGetBootPath() -> PCCHAR;
    fn FrLdrHeapAlloc(size: usize, tag: ULONG) -> *mut c_void;
    fn FrLdrHeapFree(p: *mut c_void, tag: ULONG);
    fn IniGetNumSectionItems(SectionId: ULONG_PTR) -> ULONG;
    #[link_name = "IniGetSectionSettingNameSize"]
    fn IniGetSectionSettingNameSize__c(SectionId: ULONG_PTR, index: ULONG) -> ULONG;
    #[link_name = "IniGetSectionSettingValueSize"]
    fn IniGetSectionSettingValueSize__c(SectionId: ULONG_PTR, index: ULONG) -> ULONG;
    fn IniReadSettingByNumber(
        SectionId: ULONG_PTR, index: ULONG,
        name: PCHAR, name_size: usize,
        value: PCHAR, value_size: usize,
    ) -> BOOLEAN;
    fn IniReadSettingByName(SectionId: ULONG_PTR, SettingName: PCSTR, Buffer: PCHAR, BufferSize: ULONG) -> BOOLEAN;
    fn IniModifySettingValue(SectionId: ULONG_PTR, SettingName: PCSTR, SettingValue: PCSTR) -> BOOLEAN;
    fn IniFileInitialize() -> BOOLEAN;
    fn IniCleanup();

    fn strcat(dest: PCHAR, src: PCCHAR) -> PCHAR;
    fn strcpy(dest: PCHAR, src: PCCHAR) -> PCHAR;
    fn strlen(s: PCCHAR) -> usize;
    fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32;

    fn UiMessageBox(Format: PCSTR);
    fn UiMessageBoxCritical(MessageText: PCSTR);
    fn LoadBootDeviceDriver() -> LONG;
    fn LoadSettings(SectionName: PCSTR);
    fn DebugInit(DebugString: PCSTR);
    fn UiInitialize(ShowUi: BOOLEAN) -> BOOLEAN;
    fn UiUnInitialize(BootText: PCSTR);
    fn InitOperatingSystemList(
        OperatingSystemCount: *mut ULONG,
        DefaultOperatingSystem: *mut ULONG,
    ) -> *mut OperatingSystemItem;
    fn UiShowMessageBoxesInSection(SectionId: ULONG_PTR);
    fn UiDrawBackdrop(rows: ULONG);
    fn UiGetScreenHeight() -> ULONG;
    fn UiDisplayMenu(
        MenuHeader: PCSTR,
        MenuFooter: PCSTR,
        MenuItemList: *mut PCSTR,
        ItemCount: ULONG,
        DefaultMenuItem: ULONG,
        TimeOut: LONG,
        SelectedMenuItem: *mut ULONG,
        CanEscape: BOOLEAN,
        KeyPressFilter: Option<unsafe extern "C" fn(ULONG, ULONG, *mut c_void) -> BOOLEAN>,
        Context: *mut c_void,
    ) -> BOOLEAN;
    fn GetBootMgrInfo() -> *mut BOOTMGRINFO;

    // bootmgr.c → options.c (still C in rosload, Rust in uefi build).
    fn DoOptionsMenu(OperatingSystem: *mut OperatingSystemItem);
    fn DisplayBootTimeOptions();
}

// inifile.rs (and the C) return ULONG; a usize decl read 32 bits nobody wrote on 64-bit.
#[allow(non_snake_case)]
unsafe fn IniGetSectionSettingValueSize(SectionId: ULONG_PTR, index: ULONG) -> usize {
    IniGetSectionSettingValueSize__c(SectionId, index) as usize
}

// inifile.rs (and the C) return ULONG; a usize decl read 32 bits nobody wrote on 64-bit.
#[allow(non_snake_case)]
unsafe fn IniGetSectionSettingNameSize(SectionId: ULONG_PTR, index: ULONG) -> usize {
    IniGetSectionSettingNameSize__c(SectionId, index) as usize
}

// keycodes.h: KEY_F8 = 0x42, KEY_F10 = 0x44
const KEY_F8: ULONG = 0x42;
const KEY_F10: ULONG = 0x44;

// SystemPartition= / LoadIdentifier= argv buffer; TAG_STRING ('Stra' little-endian).
const TAG_STRING: ULONG = u32::from_be_bytes(*b" rtS"); // nolint: const-contradiction (' rtS' -- freeldr/include/oslist.h:22; win32k's TAG_STRING is ' RTS')
// FrLdrTempAlloc tag for OS display-name list ('OSNm').
const TAG_OSNAME_LIST: ULONG = u32::from_be_bytes(*b"OSNm");

// EditCustomBoot{Sector,Linux} / LoadAndBoot{Sector,Linux} only exist on
// non-UEFI x86/amd64. They are NOT referenced from this Rust file (the
// OSLoadingMethods table below has no UEFIBOOT branches for them), so we
// don't need to declare them.

unsafe extern "C" fn EditCustomBootReactOSSetup(OperatingSystem: *mut OperatingSystemItem) {
    EditCustomBootReactOS(OperatingSystem, 1 as BOOLEAN);
}

unsafe extern "C" fn EditCustomBootNTOS(OperatingSystem: *mut OperatingSystemItem) {
    EditCustomBootReactOS(OperatingSystem, 0 as BOOLEAN);
}

// UEFIBOOT path: BootSector/Linux entries are omitted.
// _M_IX86 → 5 entries (ReactOSSetup + WindowsNT40 + 3 NT-family)
#[cfg(target_arch = "x86")]
pub static OSLoadingMethods: OsLoadingMethodsTable<5> = OsLoadingMethodsTable([
    OS_LOADING_METHOD {
        BootType: b"ReactOSSetup\0".as_ptr() as PCSTR,
        EditOsEntry: Some(EditCustomBootReactOSSetup),
        OsLoader: Some(LoadReactOSSetup),
    },
    OS_LOADING_METHOD {
        BootType: b"WindowsNT40\0".as_ptr() as PCSTR,
        EditOsEntry: Some(EditCustomBootNTOS),
        OsLoader: Some(LoadAndBootWindows),
    },
    OS_LOADING_METHOD {
        BootType: b"Windows\0".as_ptr() as PCSTR,
        EditOsEntry: Some(EditCustomBootNTOS),
        OsLoader: Some(LoadAndBootWindows),
    },
    OS_LOADING_METHOD {
        BootType: b"Windows2003\0".as_ptr() as PCSTR,
        EditOsEntry: Some(EditCustomBootNTOS),
        OsLoader: Some(LoadAndBootWindows),
    },
    OS_LOADING_METHOD {
        BootType: b"WindowsVista\0".as_ptr() as PCSTR,
        EditOsEntry: Some(EditCustomBootNTOS),
        OsLoader: Some(LoadAndBootWindows),
    },
]);

// amd64 / arm64: no WindowsNT40 → 4 entries
#[cfg(not(target_arch = "x86"))]
pub static OSLoadingMethods: OsLoadingMethodsTable<4> = OsLoadingMethodsTable([
    OS_LOADING_METHOD {
        BootType: b"ReactOSSetup\0".as_ptr() as PCSTR,
        EditOsEntry: Some(EditCustomBootReactOSSetup),
        OsLoader: Some(LoadReactOSSetup),
    },
    OS_LOADING_METHOD {
        BootType: b"Windows\0".as_ptr() as PCSTR,
        EditOsEntry: Some(EditCustomBootNTOS),
        OsLoader: Some(LoadAndBootWindows),
    },
    OS_LOADING_METHOD {
        BootType: b"Windows2003\0".as_ptr() as PCSTR,
        EditOsEntry: Some(EditCustomBootNTOS),
        OsLoader: Some(LoadAndBootWindows),
    },
    OS_LOADING_METHOD {
        BootType: b"WindowsVista\0".as_ptr() as PCSTR,
        EditOsEntry: Some(EditCustomBootNTOS),
        OsLoader: Some(LoadAndBootWindows),
    },
]);

unsafe fn GetOSLoadingMethod(SectionId: ULONG_PTR) -> *const OS_LOADING_METHOD {
    let mut BootType: [i8; 80] = [0; 80];
    BootType[0] = 0;
    IniReadSettingByName(
        SectionId,
        b"BootType\0".as_ptr() as PCSTR,
        BootType.as_mut_ptr(),
        core::mem::size_of_val(&BootType) as ULONG,
    );

    // HAS_DEPRECATED_OPTIONS branch: rewrite Drive/Partition→BootSector.
    // The C variadic WarnDeprecated is inlined here (call site is unique)
    // to avoid Rust's c_variadic unstable feature for variadic defs.
    if _stricmp(BootType.as_ptr(), b"Drive\0".as_ptr() as PCSTR) == 0
        || _stricmp(BootType.as_ptr(), b"Partition\0".as_ptr() as PCSTR) == 0
    {
        // Only display the warning if the user already cancelled the timeout.
        if (*GetBootMgrInfo()).TimeOut < 0 {
            UiMessageBox(
                b"                           WARNING!\n\nThe boot configuration you are using ('Drive'/'Partition')\nis no longer supported and will be removed in future\nFreeLoader versions.\n\nPlease edit FREELDR.INI to replace 'BootType=Drive' or\n'BootType=Partition' with 'BootType=BootSector'.\n\nShould you need assistance, please contact ReactOS developers\non the official ReactOS Mattermost server <chat.reactos.org>.\0"
                    .as_ptr() as PCSTR,
            );
        }
        // Fixup the BootType.
        strcpy(BootType.as_mut_ptr(), b"BootSector\0".as_ptr() as PCCHAR);
        let _ = IniModifySettingValue(
            SectionId,
            b"BootType\0".as_ptr() as PCSTR,
            BootType.as_ptr() as PCSTR,
        );
    }

    let mut i: ULONG = 0;
    let table = &OSLoadingMethods.0;
    loop {
        if (i as usize) >= table.len() {
            UiMessageBox(b"Unknown boot entry type.\0".as_ptr() as PCSTR);
            return core::ptr::null();
        }
        if _stricmp(BootType.as_ptr(), table[i as usize].BootType) == 0 {
            return &table[i as usize] as *const OS_LOADING_METHOD;
        }
        i += 1;
    }
}

unsafe fn BuildArgvForOsLoader(
    LoadIdentifier_in: PCSTR,
    SectionId: ULONG_PTR,
    pArgc: *mut ULONG,
) -> *mut PCHAR {
    let mut Size: usize;
    let Count: ULONG;
    let mut i: ULONG;
    let Argc: ULONG;
    let Argv: *mut PCHAR;
    let mut Args: *mut PCHAR;
    let mut SettingName: PCHAR;
    let mut SettingValue: PCHAR;
    let BootPath: PCCHAR = FrLdrGetBootPath();

    *pArgc = 0;

    if SectionId == 0 {
        return core::ptr::null_mut();
    }

    // Normalize LoadIdentifier to make subsequent tests simpler.
    let LoadIdentifier: PCSTR =
        if !LoadIdentifier_in.is_null() && *LoadIdentifier_in == 0 {
            core::ptr::null()
        } else {
            LoadIdentifier_in
        };

    Count = IniGetNumSectionItems(SectionId);
    Argc = 2 + (if !LoadIdentifier.is_null() { 1 } else { 0 }) + Count;

    Size = 0;
    Size += (strlen(b"SystemPartition=\0".as_ptr() as PCCHAR) + strlen(BootPath) + 1)
        * core::mem::size_of::<CHAR>();
    if !LoadIdentifier.is_null() {
        Size += (strlen(b"LoadIdentifier=\0".as_ptr() as PCCHAR) + strlen(LoadIdentifier as PCCHAR)
            + 1)
            * core::mem::size_of::<CHAR>();
    }
    i = 0;
    while i < Count {
        Size += IniGetSectionSettingNameSize(SectionId, i);
        Size += IniGetSectionSettingValueSize(SectionId, i);
        i += 1;
    }
    Size += core::mem::size_of::<CHAR>();

    Argv = FrLdrHeapAlloc(
        ((Argc + 1) as usize * core::mem::size_of::<*mut PCHAR>()) + Size,
        TAG_STRING,
    ) as *mut PCHAR;
    if Argv.is_null() {
        return core::ptr::null_mut();
    }

    SettingName = (Argv as ULONG_PTR
        + ((Argc + 1) as ULONG_PTR * core::mem::size_of::<*mut PCHAR>() as ULONG_PTR))
        as PCHAR;
    Args = Argv;

    *Args = core::ptr::null_mut();
    Args = Args.add(1);

    strcpy(SettingName, b"SystemPartition=\0".as_ptr() as PCCHAR);
    strcat(SettingName, BootPath);
    *Args = SettingName;
    Args = Args.add(1);
    SettingName = SettingName.add(strlen(SettingName) + 1);

    if !LoadIdentifier.is_null() {
        strcpy(SettingName, b"LoadIdentifier=\0".as_ptr() as PCCHAR);
        strcat(SettingName, LoadIdentifier as PCCHAR);
        *Args = SettingName;
        Args = Args.add(1);
        SettingName = SettingName.add(strlen(SettingName) + 1);
    }

    i = 0;
    while i < Count {
        Size = IniGetSectionSettingNameSize(SectionId, i);
        SettingValue = SettingName.add(Size);
        IniReadSettingByNumber(
            SectionId,
            i,
            SettingName,
            Size,
            SettingValue,
            IniGetSectionSettingValueSize(SectionId, i),
        );
        *SettingName.add(Size - 1) = b'=' as CHAR;
        *Args = SettingName;
        Args = Args.add(1);
        SettingName = SettingName.add(strlen(SettingName) + 1);
        i += 1;
    }

    *Args = core::ptr::null_mut();

    *pArgc = Argc;
    Argv
}

#[no_mangle]
pub unsafe extern "C" fn LoadOperatingSystem(OperatingSystem: *mut OperatingSystemItem) {
    let SectionId: ULONG_PTR = (*OperatingSystem).SectionId;
    let OSLoadingMethod: *const OS_LOADING_METHOD;
    let mut Argc: ULONG = 0;
    let Argv: *mut PCHAR;

    OSLoadingMethod = GetOSLoadingMethod(SectionId);
    if OSLoadingMethod.is_null() {
        return;
    }

    Argv = BuildArgvForOsLoader(
        (*OperatingSystem).LoadIdentifier,
        SectionId,
        core::ptr::addr_of_mut!(Argc),
    );
    if Argv.is_null() {
        return;
    }

    // _M_IX86 + !UEFIBOOT branch (DriveMapMapDrivesInSection) is skipped:
    // this Rust file is built only when UEFIBOOT is in effect.
    if let Some(loader) = (*OSLoadingMethod).OsLoader {
        loader(Argc, Argv as *mut PSTR, core::ptr::null_mut());
    }
    FrLdrHeapFree(Argv as *mut c_void, TAG_STRING);
}

#[no_mangle]
pub unsafe extern "C" fn EditOperatingSystemEntry(OperatingSystem: *mut OperatingSystemItem) {
    let OSLoadingMethod: *const OS_LOADING_METHOD =
        GetOSLoadingMethod((*OperatingSystem).SectionId);
    if !OSLoadingMethod.is_null() {
        if let Some(edit) = (*OSLoadingMethod).EditOsEntry {
            edit(OperatingSystem);
        }
    }
}

unsafe extern "C" fn MainBootMenuKeyPressFilter(
    KeyPress: ULONG,
    SelectedMenuItem: ULONG,
    Context: *mut c_void,
) -> BOOLEAN {
    // Any key-press cancels the global timeout.
    (*GetBootMgrInfo()).TimeOut = -1i32;

    match KeyPress {
        KEY_F8 => {
            DoOptionsMenu(
                (Context as *mut OperatingSystemItem).add(SelectedMenuItem as usize),
            );
            DisplayBootTimeOptions();
            return 1 as BOOLEAN;
        }
        KEY_F10 => {
            EditOperatingSystemEntry(
                (Context as *mut OperatingSystemItem).add(SelectedMenuItem as usize),
            );
            return 1 as BOOLEAN;
        }
        _ => {
            return 0 as BOOLEAN;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn RunLoader() {
    let mut OperatingSystemList: *mut OperatingSystemItem;
    let mut OperatingSystemDisplayNames: *mut PCSTR;
    let mut OperatingSystemCount: ULONG = 0;
    let mut DefaultOperatingSystem: ULONG = 0;
    let mut SelectedOperatingSystem: ULONG = 0;
    let mut i: ULONG;

    // _M_IX86 + !UEFIBOOT branch (LoadBootDeviceDriver) is skipped.
    // This Rust file is built only when UEFIBOOT is in effect.
    let _ = LoadBootDeviceDriver as *const ();

    if IniFileInitialize() == 0 {
        UiMessageBoxCritical(b"Error initializing .ini file.\0".as_ptr() as PCSTR);
        return;
    }
    LoadSettings(core::ptr::null());

    DebugInit((*GetBootMgrInfo()).DebugString);

    if UiInitialize(1 as BOOLEAN) == 0 {
        UiMessageBoxCritical(b"Unable to initialize UI.\0".as_ptr() as PCSTR);
        return;
    }

    OperatingSystemList = InitOperatingSystemList(
        core::ptr::addr_of_mut!(OperatingSystemCount),
        core::ptr::addr_of_mut!(DefaultOperatingSystem),
    );
    if OperatingSystemList.is_null() {
        UiMessageBox(
            b"Unable to read operating systems section in freeldr.ini.\nPress ENTER to reboot.\0"
                .as_ptr() as PCSTR,
        );
        UiUnInitialize(b"Rebooting...\0".as_ptr() as PCSTR);
        IniCleanup();
        return;
    }
    if OperatingSystemCount == 0 {
        UiMessageBox(
            b"There were no operating systems listed in freeldr.ini.\nPress ENTER to reboot.\0"
                .as_ptr() as PCSTR,
        );
        UiUnInitialize(b"Rebooting...\0".as_ptr() as PCSTR);
        IniCleanup();
        return;
    }

    OperatingSystemDisplayNames = FrLdrTempAlloc(
        core::mem::size_of::<PCSTR>() * OperatingSystemCount as usize,
        TAG_OSNAME_LIST,
    ) as *mut PCSTR;
    if OperatingSystemDisplayNames.is_null() {
        UiUnInitialize(b"Rebooting...\0".as_ptr() as PCSTR);
        IniCleanup();
        return;
    }

    i = 0;
    while i < OperatingSystemCount {
        *OperatingSystemDisplayNames.add(i as usize) =
            (*OperatingSystemList.add(i as usize)).LoadIdentifier;
        i += 1;
    }

    UiShowMessageBoxesInSection((*GetBootMgrInfo()).FrLdrSection as ULONG_PTR);

    loop {
        UiDrawBackdrop(UiGetScreenHeight() - 2);

        if UiDisplayMenu(
            b"Please select the operating system to start:\0".as_ptr() as PCSTR,
            b"For troubleshooting and advanced startup options for ReactOS, press F8.\0".as_ptr()
                as PCSTR,
            OperatingSystemDisplayNames,
            OperatingSystemCount,
            DefaultOperatingSystem,
            (*GetBootMgrInfo()).TimeOut,
            core::ptr::addr_of_mut!(SelectedOperatingSystem),
            0 as BOOLEAN,
            Some(MainBootMenuKeyPressFilter),
            OperatingSystemList as *mut c_void,
        ) == 0
        {
            UiMessageBox(b"Press ENTER to reboot.\0".as_ptr() as PCSTR);
            UiUnInitialize(b"Rebooting...\0".as_ptr() as PCSTR);
            IniCleanup();
            return;
        }

        // Load the chosen operating system.
        LoadOperatingSystem(OperatingSystemList.add(SelectedOperatingSystem as usize));

        (*GetBootMgrInfo()).TimeOut = -1;

        // If we got here, the OS loader failed. Re-initialize the UI in case
        // it has been corrupted. (_M_ARM branch in C skips UnInit; we always
        // call it since this Rust file isn't built for arm.)
        UiUnInitialize(b"\0".as_ptr() as PCSTR);
        UiInitialize(1 as BOOLEAN);
    }
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
// (original C source body — see boot/freeldr/freeldr/bootmgr.c)
