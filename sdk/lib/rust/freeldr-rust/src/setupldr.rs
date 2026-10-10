#[allow(unused_imports)] use crate::{ERR, TRACE, WARN};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_WINDOWS; // setupldr.c: DBG_DEFAULT_CHANNEL(WINDOWS)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_STATUS, PCSTR, PLOADER_PARAMETER_BLOCK, PSETUP_LOADER_BLOCK, PSTR, RtlCreateUnicodeStringFromAsciiz, RtlFreeUnicodeString, RtlInitEmptyUnicodeString, UINT8, _UNICODE_STRING, PLIST_ENTRY, PCUNICODE_STRING};
// freeldr_ddk_imports dropped (not in ddk): PLOADER_SYSTEM_BLOCK
use core::ffi::c_void;

// INF cache types live in inffile.rs (rustconv dropped the cross-module use)
use crate::inffile::{HINF, PINFCONTEXT, INFCONTEXT};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// SERVICE_ERROR_CRITICAL: cmtypes.h #define SERVICE_ERROR_CRITICAL 0x00000003
pub const SERVICE_ERROR_CRITICAL: ULONG = 0x00000003;
// TAG_BOOT_OPTIONS 'pOtB' (C multi-char literal is big-endian)
pub const TAG_BOOT_OPTIONS: ULONG = u32::from_be_bytes(*b"pOtB");

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "system" fn UNREFERENCED_PARAMETER(_: PVOID) {}

// setupldr.c file-local: list of options and their higher-priority counterparts.
#[repr(C)]
pub struct HighPriorOptionsMapEntry {
    pub Options: PCSTR,
    pub ExtraOptions: PCSTR,
    pub HigherPriorOptions: PCSTR,
}
// read-only table of static string literal pointers; Sync-safe.
unsafe impl Sync for HighPriorOptionsMapEntry {}
#[allow(non_upper_case_globals)]
pub static HighPriorOptionsMap: [HighPriorOptionsMapEntry; 7] = [
    HighPriorOptionsMapEntry { Options: b"/DEBUG/DEBUG=\0".as_ptr() as PCSTR, ExtraOptions: core::ptr::null(), HigherPriorOptions: b"/NODEBUG\0".as_ptr() as PCSTR },
    HighPriorOptionsMapEntry { Options: b"/DEBUGPORT=SCREEN\0".as_ptr() as PCSTR, ExtraOptions: core::ptr::null(), HigherPriorOptions: b"/NOGUIBOOT\0".as_ptr() as PCSTR },
    HighPriorOptionsMapEntry { Options: b"/HAL=/KERNEL=\0".as_ptr() as PCSTR, ExtraOptions: core::ptr::null(), HigherPriorOptions: b"/DETECTHAL\0".as_ptr() as PCSTR },
    HighPriorOptionsMapEntry { Options: b"/PAE\0".as_ptr() as PCSTR, ExtraOptions: core::ptr::null(), HigherPriorOptions: b"/NOPAE\0".as_ptr() as PCSTR },
    HighPriorOptionsMapEntry { Options: b"/EXECUTE\0".as_ptr() as PCSTR, ExtraOptions: b"/NOEXECUTE=ALWAYSOFF\0".as_ptr() as PCSTR, HigherPriorOptions: b"/NOEXECUTE/NOEXECUTE=\0".as_ptr() as PCSTR },
    HighPriorOptionsMapEntry { Options: b"/NOEXECUTE/NOEXECUTE=\0".as_ptr() as PCSTR, ExtraOptions: core::ptr::null(), HigherPriorOptions: b"/NOEXECUTE/NOEXECUTE=\0".as_ptr() as PCSTR },
    HighPriorOptionsMapEntry { Options: b"/SAFEBOOT/SAFEBOOT:\0".as_ptr() as PCSTR, ExtraOptions: core::ptr::null(), HigherPriorOptions: b"/SAFEBOOT/SAFEBOOT:\0".as_ptr() as PCSTR },
];
pub use ntkrnl_sys::ddk::_ARC_CODES_EINVAL as EINVAL;
pub use ntkrnl_sys::ddk::_ARC_CODES_ENOENT as ENOENT;
pub use ntkrnl_sys::ddk::_ARC_CODES_ENOEXEC as ENOEXEC;
pub use ntkrnl_sys::ddk::_ARC_CODES_ESUCCESS as ESUCCESS;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderRegistryData as LoaderRegistryData;

extern "C" {
    fn InfFindFirstLine(InfHandle: HINF, Section: PCSTR, Key: PCSTR, InfContext: PINFCONTEXT) -> BOOLEAN;
    fn InfGetDataField(InfContext: PINFCONTEXT, FieldIndex: u32, Data: *mut PCSTR) -> BOOLEAN;
    fn PaToVa(PhysicalAddress: PVOID) -> PVOID;
    fn RtlStringCbCatA(pszDest: *mut u8, cbDest: usize, pszSrc: PCSTR) -> NTSTATUS;
    fn WinLdrLoadModule(ModuleName: PCSTR, Size: *mut u32, MemoryType: u32) -> PVOID;
    fn NtLdrAddOptions(LoadOptions: PSTR, BufferSize: u32, Append: u8, OptionsToAdd: PCSTR);
    fn NtLdrGetNextOption(Options: *mut PCSTR, OptionLength: *mut u32) -> PCSTR;
    fn _strnicmp(s1: PCSTR, s2: PCSTR, n: usize) -> i32;
    fn strlen(s: PCSTR) -> usize;
    fn FrLdrHeapAlloc(size: usize, tag: u32) -> *mut core::ffi::c_void;
    fn FrLdrHeapFree(ptr: *mut core::ffi::c_void, tag: u32);
    // Returns the option's PCSTR (ntldropts.c). Read as a byte, an option found at an
    // address whose low byte is 0 reads "absent"; only the pointer width is safe.
    fn NtLdrGetOptionExN(options: PCSTR, option: PCSTR, length: u32, OptionLength: *mut ULONG) -> usize;
    fn RTL_NUMBER_OF(arr: *const core::ffi::c_void) -> usize;
    fn AllocateAndInitLPB(version: u32, LoaderBlock: *mut PLOADER_PARAMETER_BLOCK);
    fn GetArgumentValue(Argc: u32, Argv: *mut PSTR, name: PCSTR) -> PCSTR;
    fn InfCloseFile(InfHandle: HINF);
    fn InfOpenFile(InfHandle: *mut HINF, FileName: PCSTR, ErrorLine: *mut u32) -> u8;
    fn LoadAndBootWindowsCommon(version: u32, LoaderBlock: PLOADER_PARAMETER_BLOCK, BootOptions: PCSTR, BootPath: PCSTR) -> ARC_STATUS;
    fn NtLdrGetOption(BootOptions: PCSTR, option: PCSTR) -> usize;
    fn NtLdrGetOptionEx(BootOptions: PCSTR, option: PCSTR, length: *mut u32) -> PSTR;
    fn RamDiskInitialize(arg1: u8, BootOptions: PCSTR, SystemPartition: PCSTR) -> ARC_STATUS;
    fn UiDrawBackdrop(height: u32);
    fn UiDrawProgressBarCenter(text: PCSTR);
    fn UiDrawStatusText(text: PCSTR);
    fn UiGetScreenHeight() -> u32;
    fn UiMessageBox(fmt: PCSTR, ...);
    fn UiResetForSOS();
    fn UiUpdateProgressBar(percent: u32, text: PCSTR);
    fn WinLdrInitSystemHive(LoaderBlock: PLOADER_PARAMETER_BLOCK, BootPath: PCSTR, setuphive: u8) -> u8;
    fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32;
    fn strrchr(s: PCSTR, c: i32) -> PSTR;
    fn strstr(haystack: PCSTR, needle: PCSTR) -> PSTR;
    static mut SosEnabled: u8;
    // winldr.h: extern PLOADER_SYSTEM_BLOCK WinLdrSystemBlock; (owned by winldr.rs)
    static mut WinLdrSystemBlock: PLOADER_SYSTEM_BLOCK;
    // winldr.c: PCWSTR BootFileSystem = NULL; (owned by winldr.rs, used here as PCWSTR)
    static mut BootFileSystem: PCWSTR;
    // cross-module fns (winldr.c / inffile.c) — canonical C-header sigs, FLDRAPI=cdecl
    fn WinLdrLoadNLSData(
        LoaderBlock: PLOADER_PARAMETER_BLOCK,
        DirectoryPath: PCSTR,
        AnsiFileName: PCUNICODE_STRING,
        OemFileName: PCUNICODE_STRING,
        LangFileName: PCUNICODE_STRING,
        OemHalFileName: PCUNICODE_STRING,
    ) -> BOOLEAN;
    fn WinLdrAddDriverToList(
        DriverListHead: PLIST_ENTRY,
        InsertAtHead: BOOLEAN,
        DriverName: PCWSTR,
        ImagePath: PCWSTR,
        GroupName: PCWSTR,
        ErrorControl: ULONG,
        Tag: ULONG,
    ) -> BOOLEAN;
    fn InfGetData(Context: PINFCONTEXT, Key: *mut PCSTR, Data: *mut PCSTR) -> BOOLEAN;
    fn InfFindNextLine(ContextIn: PINFCONTEXT, ContextOut: PINFCONTEXT) -> BOOLEAN;
    // ntstrsafe.h: NTSTRSAFEVAPI RtlStringCbPrintfW(PWSTR, size_t, PCWSTR, ...); cdecl
    fn RtlStringCbPrintfW(pszDest: *mut u16, cbDest: usize, pszFormat: *const u16, ...) -> NTSTATUS;
}

// cmtypes.h: #define SERVICE_ERROR_NORMAL 0x00000001
pub const SERVICE_ERROR_NORMAL: ULONG = 0x00000001;

// setupldr.c: "SourceDisksFiles." INF_ARCH
#[cfg(target_arch = "x86")]
const INF_FILES_SECTION: &[u8] = b"SourceDisksFiles.x86\0";
#[cfg(target_arch = "x86_64")]
const INF_FILES_SECTION: &[u8] = b"SourceDisksFiles.amd64\0";
#[cfg(target_arch = "arm")]
const INF_FILES_SECTION: &[u8] = b"SourceDisksFiles.arm\0";
#[cfg(target_arch = "aarch64")]
const INF_FILES_SECTION: &[u8] = b"SourceDisksFiles.arm64\0";

unsafe fn SetupLdrLoadNlsData(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    InfHandle: HINF,
    SearchPath: PCSTR,
) {
    let Success: u8;
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut AnsiData: PCSTR = core::ptr::null();
    let mut AnsiFileName: _UNICODE_STRING = core::mem::zeroed();
    let mut OemFileName: _UNICODE_STRING = core::mem::zeroed();
    let mut LangFileName: _UNICODE_STRING = core::mem::zeroed();
    let mut OemHalFileName: _UNICODE_STRING = core::mem::zeroed();

    if InfFindFirstLine(InfHandle, b"NLS\0".as_ptr() as PCSTR, b"AnsiCodepage\0".as_ptr() as PCSTR, &mut InfContext) == 0 ||
        InfGetDataField(&mut InfContext, 1, &mut AnsiData) == 0 ||
        RtlCreateUnicodeStringFromAsciiz(&mut AnsiFileName, AnsiData) == 0
    {
        ERR!("Failed to find or get 'NLS/AnsiCodepage'\n");
        return;
    }

    if InfFindFirstLine(InfHandle, b"NLS\0".as_ptr() as PCSTR, b"OemCodepage\0".as_ptr() as PCSTR, &mut InfContext) == 0 ||
        InfGetDataField(&mut InfContext, 1, &mut AnsiData) == 0 ||
        RtlCreateUnicodeStringFromAsciiz(&mut OemFileName, AnsiData) == 0
    {
        ERR!("Failed to find or get 'NLS/OemCodepage'\n");
        RtlFreeUnicodeString(&mut OemHalFileName);
        RtlFreeUnicodeString(&mut LangFileName);
        RtlFreeUnicodeString(&mut OemFileName);
        RtlFreeUnicodeString(&mut AnsiFileName);
        return;
    }

    if InfFindFirstLine(InfHandle, b"NLS\0".as_ptr() as PCSTR, b"UnicodeCasetable\0".as_ptr() as PCSTR, &mut InfContext) == 0 ||
        InfGetDataField(&mut InfContext, 1, &mut AnsiData) == 0 ||
        RtlCreateUnicodeStringFromAsciiz(&mut LangFileName, AnsiData) == 0
    {
        ERR!("Failed to find or get 'NLS/UnicodeCasetable'\n");
        RtlFreeUnicodeString(&mut OemHalFileName);
        RtlFreeUnicodeString(&mut LangFileName);
        RtlFreeUnicodeString(&mut OemFileName);
        RtlFreeUnicodeString(&mut AnsiFileName);
        return;
    }

    if InfFindFirstLine(InfHandle, b"NLS\0".as_ptr() as PCSTR, b"OemHalFont\0".as_ptr() as PCSTR, &mut InfContext) == 0 ||
        InfGetData(&mut InfContext, core::ptr::null_mut(), &mut AnsiData) == 0 ||
        RtlCreateUnicodeStringFromAsciiz(&mut OemHalFileName, AnsiData) == 0
    {
        WARN!("Failed to find or get 'NLS/OemHalFont'\n");
        RtlInitEmptyUnicodeString(&mut OemHalFileName, core::ptr::null_mut(), 0);
    }

    TRACE!("NLS data: '%wZ' '%wZ' '%wZ' '%wZ'\n", (&AnsiFileName) as *const _ as usize, (&OemFileName) as *const _ as usize, (&LangFileName) as *const _ as usize, (&OemHalFileName) as *const _ as usize);

    Success = WinLdrLoadNLSData(LoaderBlock,
                                SearchPath,
                                &mut AnsiFileName,
                                &mut OemFileName,
                                &mut LangFileName,
                                &mut OemHalFileName);
    TRACE!("NLS data loading %s\n", (if Success != 0 { b"successful\0".as_ptr() as PCSTR } else { b"failed\0".as_ptr() as PCSTR }) as usize);

    RtlFreeUnicodeString(&mut OemHalFileName);
    RtlFreeUnicodeString(&mut LangFileName);
    RtlFreeUnicodeString(&mut OemFileName);
    RtlFreeUnicodeString(&mut AnsiFileName);
}

unsafe fn SetupLdrInitErrataInf(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    InfHandle: HINF,
    SystemRoot: PCSTR,
) -> BOOLEAN {

    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut FileName: PCSTR = core::ptr::null_mut();
    let mut FileSize: u32 = 0;
    let PhysicalBase: PVOID;
    let mut ErrataFilePath: [u8; 260] = [0; 260];

    if InfFindFirstLine(InfHandle, b"BiosInfo\0".as_ptr() as PCSTR, b"InfName\0".as_ptr() as PCSTR, &mut InfContext) == 0 {
        WARN!("Failed to find 'BiosInfo/InfName'\n");
        return 0u8;
    }
    if InfGetDataField(&mut InfContext, 1, &mut FileName) == 0 {
        WARN!("Failed to read 'InfName' value\n");
        return 0u8;
    }

    RtlStringCbCopyA(ErrataFilePath.as_mut_ptr() as *mut i8 , core::mem::size_of_val(&ErrataFilePath), SystemRoot);
    RtlStringCbCatA(ErrataFilePath.as_mut_ptr(), core::mem::size_of_val(&ErrataFilePath), FileName);

    PhysicalBase = WinLdrLoadModule(ErrataFilePath.as_ptr() as PCSTR, &mut FileSize, LoaderRegistryData);
    if PhysicalBase.is_null() {
        WARN!("Could not load '%s'\n", (ErrataFilePath.as_ptr()) as usize);
        return 0u8;
    }

    (*(*LoaderBlock).Extension).EmInfFileImage = PaToVa(PhysicalBase);
    (*(*LoaderBlock).Extension).EmInfFileSize = FileSize;

    1u8
}

unsafe fn SetupLdrScanBootDrivers(
    BootDriverListHead: PLIST_ENTRY,
    InfHandle: HINF,
    SearchPath: PCSTR,
) {
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut dirContext: INFCONTEXT = core::mem::zeroed();
    let mut Media: PCSTR = core::ptr::null();
    let mut DriverName: PCSTR = core::ptr::null();
    let mut dirIndex: PCSTR = core::ptr::null();
    let mut ImagePath: PCSTR = core::ptr::null();
    let mut Success: BOOLEAN;
    let mut Pass: UINT8;
    let mut ImagePathW: [WCHAR; 260] = [0; 260];
    let mut DriverNameW: [WCHAR; 256] = [0; 256];

    UNREFERENCED_PARAMETER(SearchPath as PVOID);

    Pass = 0;
    while Pass <= 1 {
        // C: {"SourceDisksFiles." INF_ARCH, "SourceDisksFiles"}. The section is per
        // ARCH: reading `.x86` everywhere left amd64 setup without its own section's
        // boot drivers (acpi.sys is 'x' in `.amd64` and not in `.x86`).
        let pFilesSection: [PCSTR; 2] = [INF_FILES_SECTION.as_ptr() as PCSTR, b"SourceDisksFiles\0".as_ptr() as PCSTR];

        if InfFindFirstLine(InfHandle, pFilesSection[Pass as usize], core::ptr::null(), &mut InfContext) == 0 {
            Pass += 1;
            continue;
        }

        loop {
            if InfGetDataField(&mut InfContext, 7, &mut Media) != 0
                && InfGetDataField(&mut InfContext, 0, &mut DriverName) != 0
                && InfGetDataField(&mut InfContext, 13, &mut dirIndex) != 0
            {
                if strcmp(Media, b"x\0".as_ptr() as PCSTR) == 0
                    && InfFindFirstLine(InfHandle, b"Directories\0".as_ptr() as PCSTR, dirIndex, &mut dirContext) != 0
                    && InfGetDataField(&mut dirContext, 1, &mut ImagePath) != 0
                {
                    RtlStringCbPrintfW(
                        ImagePathW.as_mut_ptr(),
                        core::mem::size_of_val(&ImagePathW),
                        w!("%S\\%S").as_ptr() as *const u16,
                        ImagePath,
                        DriverName,
                    );

                    RtlStringCbPrintfW(
                        DriverNameW.as_mut_ptr(),
                        core::mem::size_of_val(&DriverNameW),
                        w!("%S").as_ptr() as *const u16,
                        DriverName,
                    );
                    // Strip ".sys". The C indexes len - 4 unchecked; a name shorter than
                    // its extension is an INF error, not a reason to panic the loader.
                    let NameLen = wcslen(DriverNameW.as_ptr());
                    if NameLen >= 4 {
                        DriverNameW[NameLen - 4] = 0;
                    }

                    Success = WinLdrAddDriverToList(
                        BootDriverListHead,
                        0u8,
                        DriverNameW.as_ptr(),
                        ImagePathW.as_ptr(),
                        core::ptr::null(),
                        SERVICE_ERROR_NORMAL,
                        (-1i32) as u32,
                    );
                    if Success == 0 {
                        ERR!("Could not add boot driver '%s'\n", (DriverName) as usize);
                    }
                }
            }

            if InfFindNextLine(&mut InfContext, &mut InfContext) == 0 {
                break;
            }
        }

        Pass += 1;
    }

    if !BootFileSystem.is_null() {
        TRACE!("Adding filesystem driver %S\n", (BootFileSystem) as usize);
        Success = WinLdrAddDriverToList(
            BootDriverListHead,
            0u8,
            BootFileSystem,
            core::ptr::null(),
            w!("Boot File System").as_ptr() as *const u16,
            SERVICE_ERROR_CRITICAL,
            (-1i32) as u32 ,
        );
        if Success == 0 {
            ERR!("Failed to add filesystem driver %S\n", (BootFileSystem) as usize);
        }
    } else {
        TRACE!("No required filesystem driver\n");
    }
}

#[no_mangle]
pub unsafe fn NtLdrUpdateLoadOptions(
    LoadOptions: PSTR,
    BufferSize: u32,
    Append: u8,
    OptionsToAdd: *const PCSTR,
    OptionsToRemove: *const PCSTR,
) {
    let mut NextOptions: PCSTR;
    let mut NextOpt: PCSTR;
    let mut Options: PSTR;
    let mut Option: PSTR;
    let mut NextOptLength: u32 = 0;
    let mut OptionLength: u32 = 0;

    if LoadOptions.is_null() || BufferSize == 0 {
        return;
    }

    let mut remove_ptr = OptionsToRemove;
    while !remove_ptr.is_null() && !(*remove_ptr).is_null() {
        NextOptions = *remove_ptr;
        loop {
            NextOpt = NtLdrGetNextOption(
                core::ptr::addr_of_mut!(NextOptions),
                core::ptr::addr_of_mut!(NextOptLength),
            );
            if NextOpt.is_null() {
                break;
            }

            Options = LoadOptions;
            loop {
                Option = NtLdrGetNextOption(
                    core::ptr::addr_of_mut!(Options) as *mut PCSTR,
                    core::ptr::addr_of_mut!(OptionLength),
                ) as PSTR;
                if Option.is_null() {
                    break;
                }

                if OptionLength >= NextOptLength
                    && _strnicmp(Option, NextOpt, NextOptLength as usize) == 0
                {
                    if OptionLength == NextOptLength
                        || *NextOpt.offset((NextOptLength - 1) as isize) == b'=' as i8
                        || *NextOpt.offset((NextOptLength - 1) as isize) == b':' as i8
                    {
                        while Option > LoadOptions
                            && (*Option.offset(-1) == b'/' as i8
                                || *Option.offset(-1) == b' ' as i8
                                || *Option.offset(-1) == b'\t' as i8)
                        {
                            Option = Option.offset(-1);
                        }

                        if Option > LoadOptions
                            && *Option.offset(-1) != b' ' as i8
                            && *Option.offset(-1) != b'\t' as i8
                            && *Options != 0
                        {
                            *Option = b' ' as i8;
                            Option = Option.offset(1);
                        }

                        core::ptr::copy(
                            Options,
                            Option,
                            (strlen(Options) + 1) as usize,
                        );

                        Options = Option;
                    }
                }
            }
        }
        remove_ptr = remove_ptr.offset(1);
    }

    let mut add_ptr = OptionsToAdd;
    while !add_ptr.is_null() && !(*add_ptr).is_null() {
        NtLdrAddOptions(LoadOptions, BufferSize, Append, *add_ptr);
        add_ptr = add_ptr.offset(1);
    }
}

unsafe fn NtLdrGetHigherPriorityOptions(
    BootOptions: PCSTR,
    ExtraOptions: *mut PSTR,
    HigherPriorityOptions: *mut PSTR,
) {

    let mut i: u32;
    let mut NextOptions: PCSTR;
    let mut NextOpt: PCSTR;
    let mut NextOptLength: u32 = 0;
    let mut ExtraOptsSize: usize = 0;
    let mut HighPriorOptsSize: usize = 0;

    let mut Masks: [u8; 64] = [0; 64];

    if ExtraOptions.is_null() && HigherPriorityOptions.is_null() {
        return;
    }

    if !ExtraOptions.is_null() {
        *ExtraOptions = core::ptr::null_mut();
    }
    if !HigherPriorityOptions.is_null() {
        *HigherPriorityOptions = core::ptr::null_mut();
    }

    if BootOptions.is_null() || *BootOptions == 0 {
        return;
    }

    core::ptr::write_bytes(Masks.as_mut_ptr() as *mut u8, 0, core::mem::size_of_val(&Masks));
    
    i = 0;
    while i < HighPriorOptionsMap.len() as u32 {
        NextOptions = (*HighPriorOptionsMap.as_ptr().add(i as usize)).Options;
        loop {
            NextOpt = NtLdrGetNextOption(core::ptr::addr_of_mut!(NextOptions), core::ptr::addr_of_mut!(NextOptLength));
            if NextOpt.is_null() {
                break;
            }
            
            if NtLdrGetOptionExN(BootOptions, NextOpt, NextOptLength, core::ptr::null_mut()) != 0 {
                Masks[i as usize] = 1u8;
                if !ExtraOptions.is_null() && !(*HighPriorOptionsMap.as_ptr().add(i as usize)).ExtraOptions.is_null() {
                    ExtraOptsSize += strlen((*HighPriorOptionsMap.as_ptr().add(i as usize)).ExtraOptions) * core::mem::size_of::<i8>();
                }
                if !HigherPriorityOptions.is_null() && !(*HighPriorOptionsMap.as_ptr().add(i as usize)).HigherPriorOptions.is_null() {
                    HighPriorOptsSize += strlen((*HighPriorOptionsMap.as_ptr().add(i as usize)).HigherPriorOptions) * core::mem::size_of::<i8>();
                }
                break;
            }
        }
        i += 1;
    }

    if !ExtraOptions.is_null() {
        ExtraOptsSize += core::mem::size_of::<i8>();
    }
    if !HigherPriorityOptions.is_null() {
        HighPriorOptsSize += core::mem::size_of::<i8>();
    }

    if !ExtraOptions.is_null() {
        *ExtraOptions = FrLdrHeapAlloc(ExtraOptsSize, TAG_BOOT_OPTIONS) as PSTR;
        if (*ExtraOptions).is_null() {
            return;
        }
    }
    if !HigherPriorityOptions.is_null() {
        *HigherPriorityOptions = FrLdrHeapAlloc(HighPriorOptsSize, TAG_BOOT_OPTIONS) as PSTR;
        if (*HigherPriorityOptions).is_null() {
            if !ExtraOptions.is_null() {
                FrLdrHeapFree(*ExtraOptions as *mut core::ffi::c_void, TAG_BOOT_OPTIONS);
                *ExtraOptions = core::ptr::null_mut();
            }
            return;
        }
    }

    if !ExtraOptions.is_null() {
        *(*ExtraOptions) = 0i8;
    }
    if !HigherPriorityOptions.is_null() {
        *(*HigherPriorityOptions) = 0i8;
    }

    i = 0;
    while i < HighPriorOptionsMap.len() as u32 {
        if Masks[i as usize] != 0 {
            if !ExtraOptions.is_null() && !(*HighPriorOptionsMap.as_ptr().add(i as usize)).ExtraOptions.is_null() {
                RtlStringCbCatA(
                    *ExtraOptions as *mut u8 ,
                    ExtraOptsSize,
                    (*HighPriorOptionsMap.as_ptr().add(i as usize)).ExtraOptions,
                );
            }
            if !HigherPriorityOptions.is_null() && !(*HighPriorOptionsMap.as_ptr().add(i as usize)).HigherPriorOptions.is_null() {
                RtlStringCbCatA(
                    *HigherPriorityOptions as *mut u8 ,
                    HighPriorOptsSize,
                    (*HighPriorOptionsMap.as_ptr().add(i as usize)).HigherPriorOptions,
                );
            }
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn LoadReactOSSetup(
    Argc: u32,
    Argv: *mut PSTR,
    _Envp: *mut PSTR,
) -> ARC_STATUS {

    let Status: ARC_STATUS;
    let mut ArgValue: PCSTR;
    let SystemPartition: PCSTR;
    let mut SystemPath: PCSTR;
    let mut FileName: PSTR;
    let mut FileNameLength: u32 = 0;
    let BootFromFloppy: u8;
    let _Success: u8;
    let mut InfHandle: HINF = core::ptr::null_mut();
    let mut InfContext: INFCONTEXT = core::mem::zeroed();
    let mut i: u32;
    let mut ErrorLine: u32 = 0;
    let mut LoaderBlock: PLOADER_PARAMETER_BLOCK = core::ptr::null_mut();
    let SetupBlock: PSETUP_LOADER_BLOCK;
    let mut BootPath: [u8; 260] = [0; 260];
    let mut FilePath: [u8; 260] = [0; 260];
    let mut UserBootOptions: [u8; 256] = [0; 256];
    let mut BootOptions: PCSTR;

    // C: { "", #if _M_IX86 "I386\\" #endif "reactos\\", NULL } — the I386\ entry is x86 only.
    #[repr(transparent)]
    struct SourcePathsT<const N: usize>([PCSTR; N]);
    unsafe impl<const N: usize> Sync for SourcePathsT<N> {}
    #[cfg(target_arch = "x86")]
    static SourcePaths: SourcePathsT<4> = SourcePathsT([
        b"\0".as_ptr() as PCSTR,
        b"I386\\\0".as_ptr() as PCSTR,
        b"reactos\\\0".as_ptr() as PCSTR,
        core::ptr::null(),
    ]);
    #[cfg(not(target_arch = "x86"))]
    static SourcePaths: SourcePathsT<3> = SourcePathsT([
        b"\0".as_ptr() as PCSTR,
        b"reactos\\\0".as_ptr() as PCSTR,
        core::ptr::null(),
    ]);

    ArgValue = GetArgumentValue(Argc, Argv, b"BootType\0".as_ptr() as PCSTR);
    if ArgValue.is_null() || *ArgValue == 0 {
        ERR!("No 'BootType' value, aborting!\n");
        return EINVAL;
    }
    if _stricmp(ArgValue, b"ReactOSSetup\0".as_ptr() as PCSTR) != 0 {
        ERR!("Unknown 'BootType' value '%s', aborting!\n", (ArgValue) as usize);
        return EINVAL;
    }

    SystemPartition = GetArgumentValue(Argc, Argv, b"SystemPartition\0".as_ptr() as PCSTR);
    if SystemPartition.is_null() || *SystemPartition == 0 {
        ERR!("No 'SystemPartition' specified, aborting!\n");
        return EINVAL;
    }

    UiDrawBackdrop(UiGetScreenHeight());
    UiDrawStatusText(b"Setup is loading...\0".as_ptr() as PCSTR);
    UiDrawProgressBarCenter(b"Loading ReactOS Setup...\0".as_ptr() as PCSTR);

    *BootPath.as_mut_ptr() = 0;
    ArgValue = GetArgumentValue(Argc, Argv, b"SystemPath\0".as_ptr() as PCSTR);
    if !ArgValue.is_null() {
        RtlStringCbCopyA(BootPath.as_mut_ptr() as PSTR, BootPath.len(), ArgValue);
    } else {
        *BootPath.as_mut_ptr() = 0;
    }

    if strrchr(BootPath.as_ptr() as PCSTR, ')' as i32).is_null() {
        RtlStringCbCopyA(FilePath.as_mut_ptr() as PSTR, FilePath.len(), BootPath.as_ptr() as PCSTR);
        RtlStringCbCopyA(BootPath.as_mut_ptr() as PSTR, BootPath.len(), SystemPartition);
        if *FilePath.as_ptr() != b'\\' && *FilePath.as_ptr() != b'/' {
            RtlStringCbCatA(BootPath.as_mut_ptr() as PSTR as *mut u8 , BootPath.len(), b"\\\0".as_ptr() as PCSTR);
        }
        RtlStringCbCatA(BootPath.as_mut_ptr() as PSTR as *mut u8 , BootPath.len(), FilePath.as_ptr() as PCSTR);
    }

    if *BootPath.as_ptr() == 0 || *BootPath.as_ptr().add(strlen(BootPath.as_ptr() as PCSTR) - 1) != b'\\' {
        RtlStringCbCatA(BootPath.as_mut_ptr() as PSTR as *mut u8 , BootPath.len(), b"\\\0".as_ptr() as PCSTR);
    }

    TRACE!("BootPath: '%s'\n", (BootPath.as_ptr()) as usize);

    BootOptions = GetArgumentValue(Argc, Argv, b"Options\0".as_ptr() as PCSTR);
    if BootOptions.is_null() {
        BootOptions = b"\0".as_ptr() as PCSTR;
    }

    TRACE!("BootOptions: '%s'\n", (BootOptions) as usize);

    FileName = NtLdrGetOptionEx(BootOptions, b"RDPATH=\0".as_ptr() as PCSTR, &mut FileNameLength);
    if !FileName.is_null() && FileNameLength >= 7 {
        Status = RamDiskInitialize(0u8, BootOptions, SystemPartition);
        if Status != ESUCCESS {
            FileName = FileName.add(7);
            FileNameLength -= 7;
            UiMessageBox(b"Failed to load RAM disk file '%.*s'\0".as_ptr() as PCSTR, FileNameLength, FileName);
            return Status;
        }
    }

    BootFromFloppy = if !strstr(BootPath.as_ptr() as PCSTR, b"fdisk\0".as_ptr() as PCSTR).is_null() { 1u8 } else { 0u8 };

    FileName = BootPath.as_mut_ptr().add(strlen(BootPath.as_ptr() as PCSTR)) as *mut i8 ;
    i = if BootFromFloppy != 0 { 0 } else { 1 };
    loop {
        SystemPath = SourcePaths.0[i as usize];
        if SystemPath.is_null() {
            UiMessageBox(b"Failed to open txtsetup.sif\0".as_ptr() as PCSTR);
            return ENOENT;
        }
        FileNameLength = (BootPath.len() - (FileName as usize - BootPath.as_ptr() as usize)) as u32;
        RtlStringCbCopyA(FileName, FileNameLength as usize, SystemPath);
        RtlStringCbCopyA(FilePath.as_mut_ptr() as PSTR, FilePath.len(), BootPath.as_ptr() as PCSTR);
        RtlStringCbCatA(FilePath.as_mut_ptr() as PSTR as *mut u8 , FilePath.len(), b"txtsetup.sif\0".as_ptr() as PCSTR);
        if InfOpenFile(&mut InfHandle, FilePath.as_ptr() as PCSTR, &mut ErrorLine) != 0 {
            break;
        }
        i += 1;
    }

    TRACE!("BootPath: '%s', SystemPath: '%s'\n", (BootPath.as_ptr()) as usize, (SystemPath) as usize);

    if NtLdrGetOption(BootOptions, b"SIFOPTIONSOVERRIDE\0".as_ptr() as PCSTR) != 0 {
        let OptionsToRemove: [PCSTR; 2] = [b"SIFOPTIONSOVERRIDE\0".as_ptr() as PCSTR, core::ptr::null()];
        RtlStringCbCopyA(UserBootOptions.as_mut_ptr() as PSTR, UserBootOptions.len(), BootOptions);
        NtLdrUpdateLoadOptions(UserBootOptions.as_mut_ptr() as PSTR, UserBootOptions.len() as u32, 0u8, core::ptr::null(), OptionsToRemove.as_ptr());
        BootOptions = UserBootOptions.as_ptr() as PCSTR;
    } else {
        let mut LoadOptions: PCSTR = core::ptr::null();
        let mut DbgLoadOptions: PCSTR = core::ptr::null();
        let mut ExtraOptions: PSTR = core::ptr::null_mut();
        let mut HigherPriorityOptions: PSTR = core::ptr::null_mut();
        let mut OptionsToAdd: [PSTR; 3] = [core::ptr::null_mut(); 3];
        let mut OptionsToRemove: [PSTR; 4] = [core::ptr::null_mut(); 4];

        if InfFindFirstLine(InfHandle, b"SetupData\0".as_ptr() as PCSTR, b"OsLoadOptions\0".as_ptr() as PCSTR, &mut InfContext) != 0 {
            if InfGetDataField(&mut InfContext, 1, &mut LoadOptions) == 0 {
                WARN!("Failed to get load options\n");
            }
        }

        #[cfg(not(debug_assertions))]
        {
            if NtLdrGetOption(BootOptions, b"DEBUG\0".as_ptr() as PCSTR) != 0 || NtLdrGetOption(BootOptions, b"DEBUG=\0".as_ptr() as PCSTR) != 0 {
                if InfFindFirstLine(InfHandle, b"SetupData\0".as_ptr() as PCSTR, b"SetupDebugOptions\0".as_ptr() as PCSTR, &mut InfContext) != 0 {
                    if InfGetDataField(&mut InfContext, 1, &mut DbgLoadOptions) == 0 {
                        WARN!("Failed to get debug load options\n");
                    }
                }
                if DbgLoadOptions.is_null() {
                    DbgLoadOptions = b"/DEBUG\0".as_ptr() as PCSTR;
                }
            }
        }

        #[cfg(debug_assertions)]
        {
            if InfFindFirstLine(InfHandle, b"SetupData\0".as_ptr() as PCSTR, b"SetupDebugOptions\0".as_ptr() as PCSTR, &mut InfContext) != 0 {
                if InfGetDataField(&mut InfContext, 1, &mut DbgLoadOptions) == 0 {
                    WARN!("Failed to get debug load options\n");
                }
            }
            if DbgLoadOptions.is_null() {
                DbgLoadOptions = b"/DEBUG\0".as_ptr() as PCSTR;
            }
        }

        *UserBootOptions.as_mut_ptr() = 0;
        if !LoadOptions.is_null() && *LoadOptions != 0 {
            RtlStringCbCopyA(UserBootOptions.as_mut_ptr() as PSTR, UserBootOptions.len(), LoadOptions);
        }

        if !DbgLoadOptions.is_null() {
            core::ptr::write_bytes(OptionsToAdd.as_mut_ptr(), 0, 3);
            core::ptr::write_bytes(OptionsToRemove.as_mut_ptr(), 0, 4);

            OptionsToRemove[0] = b"/NODEBUG\0".as_ptr() as PSTR;
            NtLdrGetHigherPriorityOptions(DbgLoadOptions, &mut ExtraOptions, &mut HigherPriorityOptions);
            OptionsToAdd[1] = if !ExtraOptions.is_null() { ExtraOptions } else { b"\0".as_ptr() as PSTR };
            OptionsToRemove[1] = if !HigherPriorityOptions.is_null() { HigherPriorityOptions } else { b"\0".as_ptr() as PSTR };

            OptionsToAdd[0] = DbgLoadOptions as PSTR;
            OptionsToRemove[2] = DbgLoadOptions as PSTR;
            NtLdrUpdateLoadOptions(UserBootOptions.as_mut_ptr() as PSTR, UserBootOptions.len() as u32, 0u8, OptionsToAdd.as_ptr() as *const PCSTR, OptionsToRemove.as_ptr() as *const PCSTR);

            if !ExtraOptions.is_null() {
                FrLdrHeapFree(ExtraOptions as *mut core::ffi::c_void, 0x544F4F42);
            }
            if !HigherPriorityOptions.is_null() {
                FrLdrHeapFree(HigherPriorityOptions as *mut core::ffi::c_void, 0x544F4F42);
            }
        }

        core::ptr::write_bytes(OptionsToAdd.as_mut_ptr(), 0, 3);
        core::ptr::write_bytes(OptionsToRemove.as_mut_ptr(), 0, 4);

        NtLdrGetHigherPriorityOptions(BootOptions, &mut ExtraOptions, &mut HigherPriorityOptions);
        OptionsToAdd[1] = if !ExtraOptions.is_null() { ExtraOptions } else { b"\0".as_ptr() as PSTR };
        OptionsToRemove[0] = if !HigherPriorityOptions.is_null() { HigherPriorityOptions } else { b"\0".as_ptr() as PSTR };

        OptionsToAdd[0] = BootOptions as PSTR;
        OptionsToRemove[1] = BootOptions as PSTR;
        NtLdrUpdateLoadOptions(UserBootOptions.as_mut_ptr() as PSTR, UserBootOptions.len() as u32, 0u8, OptionsToAdd.as_ptr() as *const PCSTR, OptionsToRemove.as_ptr() as *const PCSTR);

        if !ExtraOptions.is_null() {
            FrLdrHeapFree(ExtraOptions as *mut core::ffi::c_void, 0x544F4F42);
        }
        if !HigherPriorityOptions.is_null() {
            FrLdrHeapFree(HigherPriorityOptions as *mut core::ffi::c_void, 0x544F4F42);
        }

        BootOptions = UserBootOptions.as_ptr() as PCSTR;
    }

    TRACE!("BootOptions: '%s'\n", (BootOptions) as usize);

    SosEnabled = if NtLdrGetOption(BootOptions, b"SOS\0".as_ptr() as PCSTR) != 0 { 1u8 } else { 0u8 };
    if SosEnabled != 0 {
        UiResetForSOS();
    }

    AllocateAndInitLPB(0x0502, &mut LoaderBlock);

    SetupBlock = core::ptr::addr_of_mut!((*WinLdrSystemBlock).SetupBlock);
    (*LoaderBlock).SetupLdrBlock = SetupBlock;

    (*SetupBlock).Flags = 0x00000001;

    UiUpdateProgressBar(15, b"Loading setup system hive...\0".as_ptr() as PCSTR);
    let Success = WinLdrInitSystemHive(LoaderBlock, BootPath.as_ptr() as PCSTR, 1u8);
    TRACE!("Setup SYSTEM hive %s\n", (if Success != 0 { b"loaded\0".as_ptr() as PCSTR } else { b"not loaded\0".as_ptr() as PCSTR }) as usize);
    if Success == 0 {
        return ENOEXEC;
    }

    RtlStringCbCopyA(FilePath.as_mut_ptr() as PSTR, FilePath.len(), BootPath.as_ptr() as PCSTR);
    RtlStringCbCatA(FilePath.as_mut_ptr() as PSTR as *mut u8 , FilePath.len(), b"system32\\\0".as_ptr() as PCSTR);
    SetupLdrLoadNlsData(LoaderBlock, InfHandle, FilePath.as_ptr() as PCSTR);

    let Success = SetupLdrInitErrataInf(LoaderBlock, InfHandle, BootPath.as_ptr() as PCSTR);
    TRACE!("Firmware Errata file %s\n", (if Success != 0 { b"loaded\0".as_ptr() as PCSTR } else { b"not loaded\0".as_ptr() as PCSTR }) as usize);

    SetupLdrScanBootDrivers(core::ptr::addr_of_mut!((*LoaderBlock).BootDriverListHead), InfHandle, BootPath.as_ptr() as PCSTR);

    InfCloseFile(InfHandle);

    UiDrawStatusText(b"The Setup program is starting...\0".as_ptr() as PCSTR);

    return LoadAndBootWindowsCommon(0x0502, LoaderBlock, BootOptions, BootPath.as_ptr() as PCSTR);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Windows-compatible NT OS Setup Loader.
//  * COPYRIGHT:   Copyright 2009-2019 Aleksey Bragin <aleksey@reactos.org>
//  */
//
// #include <freeldr.h>
// #include <ndk/ldrtypes.h>
// #include <arc/setupblk.h>
// #include "winldr.h"
// #include "inffile.h"
// #include "ntldropts.h"
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WINDOWS);
//
// /* Architecture name suffixes for architecture-specific INF sections */
// #if defined(_M_IX86)
// #define INF_ARCH "x86"
// #elif defined(_M_AMD64)
// #define INF_ARCH "amd64"
// #elif defined(_M_ARM)
// #define INF_ARCH "arm"
// #elif defined(_M_ARM64)
// #define INF_ARCH "arm64"
// #endif
//
// // TODO: Move to .h
// VOID
// AllocateAndInitLPB(
//     IN USHORT VersionToBoot,
//     OUT PLOADER_PARAMETER_BLOCK* OutLoaderBlock);
//
// static VOID
// SetupLdrLoadNlsData(
//     _Inout_ PLOADER_PARAMETER_BLOCK LoaderBlock,
//     _In_ HINF InfHandle,
//     _In_ PCSTR SearchPath)
// {
//     BOOLEAN Success;
//     INFCONTEXT InfContext;
//     PCSTR AnsiData;
//     UNICODE_STRING AnsiFileName = {0};
//     UNICODE_STRING OemFileName = {0};
//     UNICODE_STRING LangFileName = {0}; // CaseTable
//     UNICODE_STRING OemHalFileName = {0};
//
//     /* Get ANSI codepage file */
//     if (!InfFindFirstLine(InfHandle, "NLS", "AnsiCodepage", &InfContext) ||
//         !InfGetDataField(&InfContext, 1, &AnsiData) ||
//         !RtlCreateUnicodeStringFromAsciiz(&AnsiFileName, AnsiData))
//     {
//         ERR("Failed to find or get 'NLS/AnsiCodepage'\n");
//         return;
//     }
//
//     /* Get OEM codepage file */
//     if (!InfFindFirstLine(InfHandle, "NLS", "OemCodepage", &InfContext) ||
//         !InfGetDataField(&InfContext, 1, &AnsiData) ||
//         !RtlCreateUnicodeStringFromAsciiz(&OemFileName, AnsiData))
//     {
//         ERR("Failed to find or get 'NLS/OemCodepage'\n");
//         goto Quit;
//     }
//
//     /* Get the Unicode case table file */
//     if (!InfFindFirstLine(InfHandle, "NLS", "UnicodeCasetable", &InfContext) ||
//         !InfGetDataField(&InfContext, 1, &AnsiData) ||
//         !RtlCreateUnicodeStringFromAsciiz(&LangFileName, AnsiData))
//     {
//         ERR("Failed to find or get 'NLS/UnicodeCasetable'\n");
//         goto Quit;
//     }
//
//     /* Get OEM HAL font file */
//     if (!InfFindFirstLine(InfHandle, "NLS", "OemHalFont", &InfContext) ||
//         !InfGetData(&InfContext, NULL, &AnsiData) ||
//         !RtlCreateUnicodeStringFromAsciiz(&OemHalFileName, AnsiData))
//     {
//         WARN("Failed to find or get 'NLS/OemHalFont'\n");
//         /* Ignore, this is an optional file */
//         RtlInitEmptyUnicodeString(&OemHalFileName, NULL, 0);
//     }
//
//     TRACE("NLS data: '%wZ' '%wZ' '%wZ' '%wZ'\n", (//           &AnsiFileName) as usize, (&OemFileName) as usize, (&LangFileName) as usize, (&OemHalFileName) as usize);
//
//     /* Load NLS data */
//     Success = WinLdrLoadNLSData(LoaderBlock,
//                                 SearchPath,
//                                 &AnsiFileName,
//                                 &OemFileName,
//                                 &LangFileName,
//                                 &OemHalFileName);
//     TRACE("NLS data loading %s\n", (Success ? "successful" : "failed") as usize);
//     (VOID)Success;
//
// Quit:
//     RtlFreeUnicodeString(&OemHalFileName);
//     RtlFreeUnicodeString(&LangFileName);
//     RtlFreeUnicodeString(&OemFileName);
//     RtlFreeUnicodeString(&AnsiFileName);
// }
//
// static
// BOOLEAN
// SetupLdrInitErrataInf(
//     IN OUT PLOADER_PARAMETER_BLOCK LoaderBlock,
//     IN HINF InfHandle,
//     IN PCSTR SystemRoot)
// {
//     INFCONTEXT InfContext;
//     PCSTR FileName;
//     ULONG FileSize;
//     PVOID PhysicalBase;
//     CHAR ErrataFilePath[MAX_PATH];
//
//     /* Retrieve the INF file name value */
//     if (!InfFindFirstLine(InfHandle, "BiosInfo", "InfName", &InfContext))
//     {
//         WARN("Failed to find 'BiosInfo/InfName'\n");
//         return FALSE;
//     }
//     if (!InfGetDataField(&InfContext, 1, &FileName))
//     {
//         WARN("Failed to read 'InfName' value\n");
//         return FALSE;
//     }
//
//     RtlStringCbCopyA(ErrataFilePath, sizeof(ErrataFilePath), SystemRoot);
//     RtlStringCbCatA(ErrataFilePath, sizeof(ErrataFilePath), FileName);
//
//     /* Load the INF file */
//     PhysicalBase = WinLdrLoadModule(ErrataFilePath, &FileSize, LoaderRegistryData);
//     if (!PhysicalBase)
//     {
//         WARN("Could not load '%s'\n", (ErrataFilePath) as usize);
//         return FALSE;
//     }
//
//     LoaderBlock->Extension->EmInfFileImage = PaToVa(PhysicalBase);
//     LoaderBlock->Extension->EmInfFileSize  = FileSize;
//
//     return TRUE;
// }
//
// static VOID
// SetupLdrScanBootDrivers(
//     _Inout_ PLIST_ENTRY BootDriverListHead,
//     _In_ HINF InfHandle,
//     _In_ PCSTR SearchPath)
// {
//     INFCONTEXT InfContext, dirContext;
//     PCSTR Media, DriverName, dirIndex, ImagePath;
//     BOOLEAN Success;
//     UINT8 Pass;
//     WCHAR ImagePathW[MAX_PATH];
//     WCHAR DriverNameW[256];
//
//     UNREFERENCED_PARAMETER(SearchPath);
//
//     /* Open the INF section, first the optional platform-specific one,
//      * then the generic section */
//     for (Pass = 0; Pass <= 1; ++Pass)
//     {
//     PCSTR pFilesSection[] = {"SourceDisksFiles." INF_ARCH, "SourceDisksFiles"};
//
//     if (!InfFindFirstLine(InfHandle, pFilesSection[Pass], NULL, &InfContext))
//         continue;
//
//     /* Load all listed boot drivers */
//     do
//     {
//         if (InfGetDataField(&InfContext, 7, &Media) &&
//             InfGetDataField(&InfContext, 0, &DriverName) &&
//             InfGetDataField(&InfContext, 13, &dirIndex))
//         {
//             if ((strcmp(Media, "x") == 0) && // HACK: ReactOS-specific
//                 InfFindFirstLine(InfHandle, "Directories", dirIndex, &dirContext) &&
//                 InfGetDataField(&dirContext, 1, &ImagePath))
//             {
//                 /* Prepare image path */
//                 RtlStringCbPrintfW(ImagePathW, sizeof(ImagePathW),
//                                    L"%S\\%S", ImagePath, DriverName);
//
//                 /* Convert name to Unicode and remove .sys extension */
//                 RtlStringCbPrintfW(DriverNameW, sizeof(DriverNameW),
//                                    L"%S", DriverName);
//                 DriverNameW[wcslen(DriverNameW) - 4] = UNICODE_NULL;
//
//                 /* Add it to the list */
//                 Success = WinLdrAddDriverToList(BootDriverListHead,
//                                                 FALSE,
//                                                 DriverNameW,
//                                                 ImagePathW,
//                                                 NULL,
//                                                 SERVICE_ERROR_NORMAL,
//                                                 -1);
//                 if (!Success)
//                 {
//                     ERR("Could not add boot driver '%s'\n", (DriverName) as usize);
//                     /* Ignore and continue adding other drivers */
//                 }
//             }
//         }
//     } while (InfFindNextLine(&InfContext, &InfContext));
//     } // for (Pass...)
//
//     /* Finally, add the boot filesystem driver to the list */
//     if (BootFileSystem)
//     {
//         TRACE("Adding filesystem driver %S\n", (BootFileSystem) as usize);
//         Success = WinLdrAddDriverToList(BootDriverListHead,
//                                         FALSE,
//                                         BootFileSystem,
//                                         NULL,
//                                         L"Boot File System",
//                                         SERVICE_ERROR_CRITICAL,
//                                         -1);
//         if (!Success)
//             ERR("Failed to add filesystem driver %S\n", (BootFileSystem) as usize);
//     }
//     else
//     {
//         TRACE("No required filesystem driver\n");
//     }
// }
//
//
// /* SETUP STARTER **************************************************************/
//
// /*
//  * Update the options in the buffer pointed by LoadOptions, of maximum size
//  * BufferSize, by first removing any specified options, and then adding any
//  * other ones.
//  *
//  * OptionsToAdd is a NULL-terminated array of string buffer pointers that
//  *    specify the options to be added into LoadOptions. Whether they are
//  *    prepended or appended to LoadOptions is controlled via the Append
//  *    parameter. The options are added in the order specified by the array.
//  *
//  * OptionsToRemove is a NULL-terminated array of string buffer pointers that
//  *    specify the options to remove from LoadOptions. Specifying also there
//  *    any options to add, has the effect of removing from LoadOptions any
//  *    duplicates of the options to be added, before adding them later into
//  *    LoadOptions. The options are removed in the order specified by the array.
//  *
//  * The options string buffers in the OptionsToRemove array have the format:
//  *    "/option1 /option2[=] ..."
//  *
//  * An option in the OptionsToRemove list with a trailing '=' or ':' designates
//  * an option in LoadOptions with user-specific data appended after the sign.
//  * When such an option is being removed from LoadOptions, all the appended
//  * data is also removed until the next option.
//  */
// VOID
// NtLdrUpdateLoadOptions(
//     IN OUT PSTR LoadOptions,
//     IN ULONG BufferSize,
//     IN BOOLEAN Append,
//     IN PCSTR OptionsToAdd[] OPTIONAL,
//     IN PCSTR OptionsToRemove[] OPTIONAL)
// {
//     PCSTR NextOptions, NextOpt;
//     PSTR Options, Option;
//     ULONG NextOptLength;
//     ULONG OptionLength;
//
//     if (!LoadOptions || (BufferSize == 0))
//         return;
//     // ASSERT(strlen(LoadOptions) + 1 <= BufferSize);
//
//     /* Loop over the options to remove */
//     for (; OptionsToRemove && *OptionsToRemove; ++OptionsToRemove)
//     {
//         NextOptions = *OptionsToRemove;
//         while ((NextOpt = NtLdrGetNextOption(&NextOptions, &NextOptLength)))
//         {
//             /* Scan the load options */
//             Options = LoadOptions;
//             while ((Option = (PSTR)NtLdrGetNextOption((PCSTR*)&Options, &OptionLength)))
//             {
//                 /*
//                  * Check whether the option to find exactly matches the current
//                  * load option, or is a prefix thereof if this is an option with
//                  * appended data.
//                  */
//                 if ((OptionLength >= NextOptLength) &&
//                     (_strnicmp(Option, NextOpt, NextOptLength) == 0))
//                 {
//                     if ((OptionLength == NextOptLength) ||
//                         (NextOpt[NextOptLength-1] == '=') ||
//                         (NextOpt[NextOptLength-1] == ':'))
//                     {
//                         /* Eat any skipped option or whitespace separators */
//                         while ((Option > LoadOptions) &&
//                                (Option[-1] == '/' ||
//                                 Option[-1] == ' ' ||
//                                 Option[-1] == '\t'))
//                         {
//                             --Option;
//                         }
//
//                         /* If the option was not preceded by a whitespace
//                          * separator, insert one and advance the pointer. */
//                         if ((Option > LoadOptions) &&
//                             (Option[-1] != ' ') &&
//                             (Option[-1] != '\t') &&
//                             (*Options != '\0') /* &&
//                             ** Not necessary since NtLdrGetNextOption() **
//                             ** stripped any leading separators.         **
//                             (*Options != ' ') &&
//                             (*Options != '\t') */)
//                         {
//                             *Option++ = ' ';
//                         }
//
//                         /* Move the remaining options back, erasing the current one */
//                         ASSERT(Option <= Options);
//                         RtlMoveMemory(Option,
//                                       Options,
//                                       (strlen(Options) + 1) * sizeof(CHAR));
//
//                         /* Reset the iterator */
//                         Options = Option;
//                     }
//                 }
//             }
//         }
//     }
//
//     /* Now loop over the options to add */
//     for (; OptionsToAdd && *OptionsToAdd; ++OptionsToAdd)
//     {
//         NtLdrAddOptions(LoadOptions,
//                         BufferSize,
//                         Append,
//                         *OptionsToAdd);
//     }
// }
//
//
// /*
//  * List of options and their corresponding higher priority ones,
//  * that are either checked before any other ones, or whose name
//  * includes another option name as a subset (e.g. NODEBUG vs. DEBUG).
//  * See also https://geoffchappell.com/notes/windows/boot/editoptions.htm
//  */
// static const struct
// {
//     PCSTR Options;
//     PCSTR ExtraOptions;
//     PCSTR HigherPriorOptions;
// } HighPriorOptionsMap[] =
// {
//     /* NODEBUG has a higher precedence than DEBUG */
//     {"/DEBUG/DEBUG=", NULL, "/NODEBUG"},
//
//     /* When using SCREEN debug port, we need boot video */
//     {"/DEBUGPORT=SCREEN", NULL, "/NOGUIBOOT"},
//
//     /* DETECTHAL has a higher precedence than HAL= or KERNEL= */
//     {"/HAL=/KERNEL=", NULL, "/DETECTHAL"},
//
//     /* NOPAE has a higher precedence than PAE */
//     {"/PAE", NULL, "/NOPAE"},
//
//     /* NOEXECUTE(=) has a higher precedence than EXECUTE */
//     {"/EXECUTE", "/NOEXECUTE=ALWAYSOFF", "/NOEXECUTE/NOEXECUTE="},
//     /* NOEXECUTE(=) options are self-excluding and
//      * some have higher precedence than others. */
//     {"/NOEXECUTE/NOEXECUTE=", NULL, "/NOEXECUTE/NOEXECUTE="},
//
//     /* SAFEBOOT(:) options are self-excluding */
//     {"/SAFEBOOT/SAFEBOOT:", NULL, "/SAFEBOOT/SAFEBOOT:"},
// };
//
// #define TAG_BOOT_OPTIONS 'pOtB'
//
// VOID
// NtLdrGetHigherPriorityOptions(
//     IN PCSTR BootOptions,
//     OUT PSTR* ExtraOptions,
//     OUT PSTR* HigherPriorityOptions)
// {
//     ULONG i;
//     PCSTR NextOptions, NextOpt;
//     ULONG NextOptLength;
//     SIZE_T ExtraOptsSize = 0;
//     SIZE_T HighPriorOptsSize = 0;
//
//     /* Masks specifying the presence (TRUE) or absence (FALSE) of the options */
//     BOOLEAN Masks[RTL_NUMBER_OF(HighPriorOptionsMap)];
//
//     /* Just return if we cannot return anything */
//     if (!ExtraOptions && !HigherPriorityOptions)
//         return;
//
//     if (ExtraOptions)
//         *ExtraOptions = NULL;
//     if (HigherPriorityOptions)
//         *HigherPriorityOptions = NULL;
//
//     /* Just return if no initial options were given */
//     if (!BootOptions || !*BootOptions)
//         return;
//
//     /* Determine the presence of the colliding options, and the
//      * maximum necessary sizes for the pointers to be allocated. */
//     RtlZeroMemory(Masks, sizeof(Masks));
//     for (i = 0; i < RTL_NUMBER_OF(HighPriorOptionsMap); ++i)
//     {
//         /* Loop over the given options to search for */
//         NextOptions = HighPriorOptionsMap[i].Options;
//         while ((NextOpt = NtLdrGetNextOption(&NextOptions, &NextOptLength)))
//         {
//             /* If any of these options are present... */
//             if (NtLdrGetOptionExN(BootOptions, NextOpt, NextOptLength, NULL))
//             {
//                 /* ... set the mask, retrieve the sizes and stop looking for these options */
//                 Masks[i] = TRUE;
//                 if (ExtraOptions && HighPriorOptionsMap[i].ExtraOptions)
//                 {
//                     ExtraOptsSize += strlen(HighPriorOptionsMap[i].ExtraOptions) * sizeof(CHAR);
//                 }
//                 if (HigherPriorityOptions && HighPriorOptionsMap[i].HigherPriorOptions)
//                 {
//                     HighPriorOptsSize += strlen(HighPriorOptionsMap[i].HigherPriorOptions) * sizeof(CHAR);
//                 }
//                 break;
//             }
//         }
//     }
//     /* Count the NULL-terminator */
//     if (ExtraOptions)
//         ExtraOptsSize += sizeof(ANSI_NULL);
//     if (HigherPriorityOptions)
//         HighPriorOptsSize += sizeof(ANSI_NULL);
//
//     /* Allocate the string pointers */
//     if (ExtraOptions)
//     {
//         *ExtraOptions = FrLdrHeapAlloc(ExtraOptsSize, TAG_BOOT_OPTIONS);
//         if (!*ExtraOptions)
//             return;
//     }
//     if (HigherPriorityOptions)
//     {
//         *HigherPriorityOptions = FrLdrHeapAlloc(HighPriorOptsSize, TAG_BOOT_OPTIONS);
//         if (!*HigherPriorityOptions)
//         {
//             if (ExtraOptions)
//             {
//                 FrLdrHeapFree(*ExtraOptions, TAG_BOOT_OPTIONS);
//                 *ExtraOptions = NULL;
//             }
//             return;
//         }
//     }
//
//     /* Initialize the strings */
//     if (ExtraOptions)
//         *(*ExtraOptions) = '\0';
//     if (HigherPriorityOptions)
//         *(*HigherPriorityOptions) = '\0';
//
//     /* Go through the masks that determine the options to check */
//     for (i = 0; i < RTL_NUMBER_OF(HighPriorOptionsMap); ++i)
//     {
//         if (Masks[i])
//         {
//             /* Retrieve the strings */
//             if (ExtraOptions && HighPriorOptionsMap[i].ExtraOptions)
//             {
//                 RtlStringCbCatA(*ExtraOptions,
//                                 ExtraOptsSize,
//                                 HighPriorOptionsMap[i].ExtraOptions);
//             }
//             if (HigherPriorityOptions && HighPriorOptionsMap[i].HigherPriorOptions)
//             {
//                 RtlStringCbCatA(*HigherPriorityOptions,
//                                 HighPriorOptsSize,
//                                 HighPriorOptionsMap[i].HigherPriorOptions);
//             }
//         }
//     }
// }
//
//
// ARC_STATUS
// LoadReactOSSetup(
//     IN ULONG Argc,
//     IN PCHAR Argv[],
//     IN PCHAR Envp[])
// {
//     ARC_STATUS Status;
//     PCSTR ArgValue;
//     PCSTR SystemPartition;
//     PCSTR SystemPath;
//     PSTR FileName;
//     ULONG FileNameLength;
//     BOOLEAN BootFromFloppy;
//     BOOLEAN Success;
//     HINF InfHandle;
//     INFCONTEXT InfContext;
//     ULONG i, ErrorLine;
//     PLOADER_PARAMETER_BLOCK LoaderBlock;
//     PSETUP_LOADER_BLOCK SetupBlock;
//     CHAR BootPath[MAX_PATH];
//     CHAR FilePath[MAX_PATH];
//     CHAR UserBootOptions[256];
//     PCSTR BootOptions;
//
//     static PCSTR SourcePaths[] =
//     {
//         "", /* Only for floppy boot */
// #if defined(_M_IX86)
//         "I386\\",
// #elif defined(_M_MPPC)
//         "PPC\\",
// #elif defined(_M_MRX000)
//         "MIPS\\",
// #endif
//         "reactos\\",
//         NULL
//     };
//
//     /* Retrieve the (mandatory) boot type */
//     ArgValue = GetArgumentValue(Argc, Argv, "BootType");
//     if (!ArgValue || !*ArgValue)
//     {
//         ERR("No 'BootType' value, aborting!\n");
//         return EINVAL;
//     }
//     if (_stricmp(ArgValue, "ReactOSSetup") != 0)
//     {
//         ERR("Unknown 'BootType' value '%s', aborting!\n", (ArgValue) as usize);
//         return EINVAL;
//     }
//
//     /* Retrieve the (mandatory) system partition */
//     SystemPartition = GetArgumentValue(Argc, Argv, "SystemPartition");
//     if (!SystemPartition || !*SystemPartition)
//     {
//         ERR("No 'SystemPartition' specified, aborting!\n");
//         return EINVAL;
//     }
//
//     /* Let the user know we started loading */
//     UiDrawBackdrop(UiGetScreenHeight());
//     UiDrawStatusText("Setup is loading...");
//     UiDrawProgressBarCenter("Loading ReactOS Setup...");
//
//     /* Retrieve the system path */
//     *BootPath = ANSI_NULL;
//     ArgValue = GetArgumentValue(Argc, Argv, "SystemPath");
//     if (ArgValue)
//     {
//         RtlStringCbCopyA(BootPath, sizeof(BootPath), ArgValue);
//     }
//     else
//     {
//         /*
//          * IMPROVE: I don't want to use the SystemPartition here as a
//          * default choice because I can do it after (see few lines below).
//          * Instead I reset BootPath here so that we can build the full path
//          * using the general code from below.
//          */
//         // RtlStringCbCopyA(BootPath, sizeof(BootPath), SystemPartition);
//         *BootPath = ANSI_NULL;
//     }
//
//     /*
//      * Check whether BootPath is a full path
//      * and if not, create a full boot path.
//      *
//      * See FsOpenFile for the technique used.
//      */
//     if (strrchr(BootPath, ')') == NULL)
//     {
//         /* Temporarily save the boot path */
//         RtlStringCbCopyA(FilePath, sizeof(FilePath), BootPath);
//
//         /* This is not a full path: prepend the SystemPartition */
//         RtlStringCbCopyA(BootPath, sizeof(BootPath), SystemPartition);
//
//         /* Append a path separator if needed */
//         if (*FilePath != '\\' && *FilePath != '/')
//             RtlStringCbCatA(BootPath, sizeof(BootPath), "\\");
//
//         /* Append the remaining path */
//         RtlStringCbCatA(BootPath, sizeof(BootPath), FilePath);
//     }
//
//     /* Append a path separator if needed */
//     if (!*BootPath || BootPath[strlen(BootPath) - 1] != '\\')
//         RtlStringCbCatA(BootPath, sizeof(BootPath), "\\");
//
//     TRACE("BootPath: '%s'\n", (BootPath) as usize);
//
//     /*
//      * Retrieve the boot options. Any options present here will supplement or
//      * override those that will be specified in TXTSETUP.SIF's OsLoadOptions.
//      */
//     BootOptions = GetArgumentValue(Argc, Argv, "Options");
//     if (!BootOptions)
//         BootOptions = "";
//
//     TRACE("BootOptions: '%s'\n", (BootOptions) as usize);
//
//     /* Check if a RAM disk file was given */
//     FileName = (PSTR)NtLdrGetOptionEx(BootOptions, "RDPATH=", &FileNameLength);
//     if (FileName && (FileNameLength >= 7))
//     {
//         /* Load the RAM disk */
//         Status = RamDiskInitialize(FALSE, BootOptions, SystemPartition);
//         if (Status != ESUCCESS)
//         {
//             FileName += 7; FileNameLength -= 7;
//             UiMessageBox("Failed to load RAM disk file '%.*s'",
//                          FileNameLength, FileName);
//             return Status;
//         }
//     }
//
//     /* Check if we booted from floppy */
//     BootFromFloppy = strstr(BootPath, "fdisk") != NULL;
//
//     /* Open 'txtsetup.sif' from any of the source paths */
//     FileName = BootPath + strlen(BootPath);
//     for (i = BootFromFloppy ? 0 : 1; ; i++)
//     {
//         SystemPath = SourcePaths[i];
//         if (!SystemPath)
//         {
//             UiMessageBox("Failed to open txtsetup.sif");
//             return ENOENT;
//         }
//         FileNameLength = (ULONG)(sizeof(BootPath) - (FileName - BootPath)*sizeof(CHAR));
//         RtlStringCbCopyA(FileName, FileNameLength, SystemPath);
//         RtlStringCbCopyA(FilePath, sizeof(FilePath), BootPath);
//         RtlStringCbCatA(FilePath, sizeof(FilePath), "txtsetup.sif");
//         if (InfOpenFile(&InfHandle, FilePath, &ErrorLine))
//         {
//             break;
//         }
//     }
//
//     TRACE("BootPath: '%s', SystemPath: '%s'\n", (BootPath) as usize, (SystemPath) as usize);
//
//     // UseLocalSif = NtLdrGetOption(BootOptions, "USELOCALSIF");
//
//     if (NtLdrGetOption(BootOptions, "SIFOPTIONSOVERRIDE"))
//     {
//         PCSTR OptionsToRemove[2] = {"SIFOPTIONSOVERRIDE", NULL};
//
//         /* Do not use any load options from TXTSETUP.SIF, but
//          * use instead those passed from the command line. */
//         RtlStringCbCopyA(UserBootOptions, sizeof(UserBootOptions), BootOptions);
//
//         /* Remove the private switch from the options */
//         NtLdrUpdateLoadOptions(UserBootOptions,
//                                sizeof(UserBootOptions),
//                                FALSE,
//                                NULL,
//                                OptionsToRemove);
//
//         BootOptions = UserBootOptions;
//     }
//     else // if (!*BootOptions || NtLdrGetOption(BootOptions, "SIFOPTIONSADD"))
//     {
//         PCSTR LoadOptions = NULL;
//         PCSTR DbgLoadOptions = NULL;
//         PSTR ExtraOptions, HigherPriorityOptions;
//         PSTR OptionsToAdd[3];
//         PSTR OptionsToRemove[4];
//
//         /* Load the options from TXTSETUP.SIF */
//         if (InfFindFirstLine(InfHandle, "SetupData", "OsLoadOptions", &InfContext))
//         {
//             if (!InfGetDataField(&InfContext, 1, &LoadOptions))
//                 WARN("Failed to get load options\n");
//         }
//
// #if !DBG
//         /* Non-debug mode: get the debug load options only if /DEBUG was specified
//          * in the Argv command-line options (was e.g. added to the options when
//          * the user selected "Debugging Mode" in the advanced boot menu). */
//         if (NtLdrGetOption(BootOptions, "DEBUG") ||
//             NtLdrGetOption(BootOptions, "DEBUG="))
//         {
// #else
//         /* Debug mode: always get the debug load options */
// #endif
//         if (InfFindFirstLine(InfHandle, "SetupData", "SetupDebugOptions", &InfContext))
//         {
//             if (!InfGetDataField(&InfContext, 1, &DbgLoadOptions))
//                 WARN("Failed to get debug load options\n");
//         }
//         /* If none was found, default to enabling debugging */
//         if (!DbgLoadOptions)
//             DbgLoadOptions = "/DEBUG";
// #if !DBG
//         }
// #endif
//
//         /* Initialize the load options with those from TXTSETUP.SIF */
//         *UserBootOptions = ANSI_NULL;
//         if (LoadOptions && *LoadOptions)
//             RtlStringCbCopyA(UserBootOptions, sizeof(UserBootOptions), LoadOptions);
//
//         /* Merge the debug load options if any */
//         if (DbgLoadOptions)
//         {
//             RtlZeroMemory(OptionsToAdd, sizeof(OptionsToAdd));
//             RtlZeroMemory(OptionsToRemove, sizeof(OptionsToRemove));
//
//             /*
//              * Retrieve any option patterns that we should remove from the
//              * SIF load options because they are of higher precedence than
//              * those specified in the debug load options to be added.
//              * Also always remove NODEBUG (even if the debug load options
//              * do not contain explicitly the DEBUG option), since we want
//              * to have debugging enabled if possible.
//              */
//             OptionsToRemove[0] = "/NODEBUG";
//             NtLdrGetHigherPriorityOptions(DbgLoadOptions,
//                                           &ExtraOptions,
//                                           &HigherPriorityOptions);
//             OptionsToAdd[1] = (ExtraOptions ? ExtraOptions : "");
//             OptionsToRemove[1] = (HigherPriorityOptions ? HigherPriorityOptions : "");
//
//             /*
//              * Prepend the debug load options, so that in case it contains
//              * redundant options with respect to the SIF load options, the
//              * former can take precedence over the latter.
//              */
//             OptionsToAdd[0] = (PSTR)DbgLoadOptions;
//             OptionsToRemove[2] = (PSTR)DbgLoadOptions;
//             NtLdrUpdateLoadOptions(UserBootOptions,
//                                    sizeof(UserBootOptions),
//                                    FALSE,
//                                    (PCSTR*)OptionsToAdd,
//                                    (PCSTR*)OptionsToRemove);
//
//             if (ExtraOptions)
//                 FrLdrHeapFree(ExtraOptions, TAG_BOOT_OPTIONS);
//             if (HigherPriorityOptions)
//                 FrLdrHeapFree(HigherPriorityOptions, TAG_BOOT_OPTIONS);
//         }
//
//         RtlZeroMemory(OptionsToAdd, sizeof(OptionsToAdd));
//         RtlZeroMemory(OptionsToRemove, sizeof(OptionsToRemove));
//
//         /*
//          * Retrieve any option patterns that we should remove from the
//          * SIF load options because they are of higher precedence than
//          * those specified in the options to be added.
//          */
//         NtLdrGetHigherPriorityOptions(BootOptions,
//                                       &ExtraOptions,
//                                       &HigherPriorityOptions);
//         OptionsToAdd[1] = (ExtraOptions ? ExtraOptions : "");
//         OptionsToRemove[0] = (HigherPriorityOptions ? HigherPriorityOptions : "");
//
//         /* Finally, prepend the user-specified options that
//          * take precedence over those from TXTSETUP.SIF. */
//         OptionsToAdd[0] = (PSTR)BootOptions;
//         OptionsToRemove[1] = (PSTR)BootOptions;
//         NtLdrUpdateLoadOptions(UserBootOptions,
//                                sizeof(UserBootOptions),
//                                FALSE,
//                                (PCSTR*)OptionsToAdd,
//                                (PCSTR*)OptionsToRemove);
//
//         if (ExtraOptions)
//             FrLdrHeapFree(ExtraOptions, TAG_BOOT_OPTIONS);
//         if (HigherPriorityOptions)
//             FrLdrHeapFree(HigherPriorityOptions, TAG_BOOT_OPTIONS);
//
//         BootOptions = UserBootOptions;
//     }
//
//     TRACE("BootOptions: '%s'\n", (BootOptions) as usize);
//
//     /* Handle the SOS option */
//     SosEnabled = !!NtLdrGetOption(BootOptions, "SOS");
//     if (SosEnabled)
//         UiResetForSOS();
//
//     /* Allocate and minimally-initialize the Loader Parameter Block */
//     AllocateAndInitLPB(_WIN32_WINNT_WS03, &LoaderBlock);
//
//     /* Allocate and initialize the setup loader block */
//     SetupBlock = &WinLdrSystemBlock->SetupBlock;
//     LoaderBlock->SetupLdrBlock = SetupBlock;
//
//     /* Set textmode setup flag */
//     SetupBlock->Flags = SETUPLDR_TEXT_MODE;
//
//     /* Load the "setupreg.hiv" setup system hive */
//     UiUpdateProgressBar(15, "Loading setup system hive...");
//     Success = WinLdrInitSystemHive(LoaderBlock, BootPath, TRUE);
//     TRACE("Setup SYSTEM hive %s\n", ((Success ? "loaded" : "not loaded")) as usize);
//     /* Bail out if failure */
//     if (!Success)
//         return ENOEXEC;
//
//     /* Load NLS data, they are in the System32 directory of the installation medium */
//     RtlStringCbCopyA(FilePath, sizeof(FilePath), BootPath);
//     RtlStringCbCatA(FilePath, sizeof(FilePath), "system32\\");
//     SetupLdrLoadNlsData(LoaderBlock, InfHandle, FilePath);
//
//     /* Load the Firmware Errata file from the installation medium */
//     Success = SetupLdrInitErrataInf(LoaderBlock, InfHandle, BootPath);
//     TRACE("Firmware Errata file %s\n", ((Success ? "loaded" : "not loaded")) as usize);
//     /* Not necessarily fatal if not found - carry on going */
//
//     // UiDrawStatusText("Press F6 if you need to install a 3rd-party SCSI or RAID driver...");
//
//     /* Get a list of boot drivers */
//     SetupLdrScanBootDrivers(&LoaderBlock->BootDriverListHead, InfHandle, BootPath);
//
//     /* Close the inf file */
//     InfCloseFile(InfHandle);
//
//     UiDrawStatusText("The Setup program is starting...");
//
//     /* Finish loading */
//     return LoadAndBootWindowsCommon(_WIN32_WINNT_WS03,
//                                     LoaderBlock,
//                                     BootOptions,
//                                     BootPath);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
