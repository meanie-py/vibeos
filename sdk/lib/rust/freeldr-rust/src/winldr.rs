use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{ARC_DISK_SIGNATURE, ARC_STATUS, BOOT_DRIVER_LIST_ENTRY, BOOT_DRIVER_NODE, FILEINFORMATION, HEADLESS_LOADER_BLOCK, KiSystemStartup, LOADER_PARAMETER_EXTENSION, MEMORY_ALLOCATION_DESCRIPTOR, PARC_DISK_SIGNATURE, PBOOT_DRIVER_LIST_ENTRY, PBOOT_DRIVER_NODE, PCCH, PCSTR, PLDR_DATA_TABLE_ENTRY, PLOADER_PARAMETER_BLOCK, PLOADER_PARAMETER_EXTENSION, PMEMORY_ALLOCATION_DESCRIPTOR, PSETUP_LOADER_BLOCK, PSTR, PUNICODE_STRING, _BOOT_DRIVER_LIST_ENTRY, _LDR_DATA_TABLE_ENTRY};
// freeldr_ddk_imports dropped (not in ddk): File, PaeEnabled
use core::ffi::c_void;

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

pub use ntkrnl_sys::ddk::_ARC_CODES_EINVAL as EINVAL;
pub use ntkrnl_sys::ddk::_ARC_CODES_ENOEXEC as ENOEXEC;
pub use ntkrnl_sys::ddk::_ARC_CODES_ESUCCESS as ESUCCESS;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderBootDriver as LoaderBootDriver;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderHalCode as LoaderHalCode;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderRegistryData as LoaderRegistryData;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderSystemBlock as LoaderSystemBlock;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderSystemCode as LoaderSystemCode;
pub use ntkrnl_sys::ddk::_OPENMODE_OpenReadOnly as OpenReadOnly;

// registry.h: typedef HANDLE HKEY, *PHKEY;
pub type HKEY = ntkrnl_sys::ddk::HANDLE;
pub type PHKEY = *mut HKEY;
// winerror.h
pub const ERROR_SUCCESS: i32 = 0;
// sdkddkver.h
pub const _WIN32_WINNT_NT4: u16 = 0x0400;
pub const _WIN32_WINNT_WS03: u16 = 0x0502;
pub const _WIN32_WINNT_VISTA: u16 = 0x0600;
// KI_USER_SHARED_DATA comes from the keystone (crate::frldr::*).

// winldr.c file-scope globals. SosEnabled/PaeModeOn are externed by
// ntldr/winldr.h (referenced by setupldr.c etc.) → external linkage, so
// this TU is the #[no_mangle] definer. WinLdrSystemBlock/BootFileSystem
// are also defined here (winldr.c lines 27-28). The remaining flags have
// internal linkage in C but keep #[no_mangle] for any straggler C TU.
#[no_mangle]
pub static mut WinLdrSystemBlock: PLOADER_SYSTEM_BLOCK = core::ptr::null_mut();
#[no_mangle]
pub static mut BootFileSystem: PCSTR = core::ptr::null_mut();
#[no_mangle]
pub static mut VirtualBias: BOOLEAN = 0;
#[no_mangle]
pub static mut SosEnabled: BOOLEAN = 0;
#[no_mangle]
pub static mut SafeBoot: BOOLEAN = 0;
#[no_mangle]
pub static mut BootLogo: BOOLEAN = 0;
#[cfg(target_arch = "x86")]
#[no_mangle]
pub static mut PaeModeOn: BOOLEAN = 0;
#[no_mangle]
pub static mut NoExecuteEnabled: BOOLEAN = 0;

extern "C" {
    fn RtlStringCbPrintfA(pszDest: *mut u8, cbDest: usize, pszFormat: PCSTR, ...) -> i32;
    fn TRACE(fmt: *const i8, ...);
    fn FrLdrIsUefiBoot() -> u8; // the first-stage loader (freeldr.spec)
    fn UiDrawStatusText(text: PCSTR);
    fn printf(fmt: *const i8, ...) -> i32;
    // Returns the option's PCSTR (ntldropts.c). Read as a byte, an option found at an
    // address whose low byte is 0 reads "absent"; only the pointer width is safe.
    fn NtLdrGetOption(BootOptions: PCSTR, Option: PCSTR) -> usize;
    fn strrchr(s: *const i8, c: i32) -> *mut i8;
    fn PeLdrCheckForLoadedDll( LoadOrderListHead: PLIST_ENTRY, DllName: PCSTR, DriverDTE: *mut PLDR_DATA_TABLE_ENTRY, ) -> u8;
    fn PeLdrLoadImage( FullPath: PCSTR, LoaderType: u32, DriverBase: *mut *mut core::ffi::c_void, ) -> u8;
    fn ERR(fmt: *const i8, ...);
    fn PaToVa(pa: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn PeLdrAllocateDataTableEntry( LoadOrderListHead: PLIST_ENTRY, BaseDllName: PCSTR, FullDllName: PCSTR, BaseAddress: *mut core::ffi::c_void, DriverDTE: *mut PLDR_DATA_TABLE_ENTRY, ) -> u8;
    fn MmFreeMemory(ptr: *mut core::ffi::c_void);
    fn PeLdrInitSecurityCookie(DriverDTE: PLDR_DATA_TABLE_ENTRY);
    fn PeLdrScanImportDescriptorTable( LoadOrderListHead: PLIST_ENTRY, FullPath: PCSTR, DriverDTE: PLDR_DATA_TABLE_ENTRY, ) -> u8;
    fn PeLdrFreeDataTableEntry(DriverDTE: PLDR_DATA_TABLE_ENTRY);
    fn ArcClose(FileId: u32) -> ARC_STATUS;
    fn ArcGetFileInformation(FileId: u32, FileInfo: *mut FILEINFORMATION) -> ARC_STATUS;
    fn ArcOpen(Path: PSTR, OpenMode: u32, FileId: *mut u32) -> ARC_STATUS;
    fn ArcRead(FileId: u32, Buffer: *mut core::ffi::c_void, Length: u32, Count: *mut u32) -> ARC_STATUS;
    // mm.c canonical takes SIZE_T (usize). u32-decl truncated arg on 64-bit.
    fn MmAllocateMemoryWithType(MemorySize: usize, MemoryType: u32) -> *mut core::ffi::c_void;
    fn WARN(fmt: *const i8, ...);
    fn UiUpdateProgressBar(Percentage: u32, ProgressString: PCCH);
    fn RtlStringCbCatA(pszDest: *mut u8, cbDest: usize, pszSrc: PCCH) -> i32;
    fn UiIndicateProgress();
    fn UiMessageBox(format: PCSTR, ...);
    fn ArcGetDiskCount() -> u32;
    fn ArcGetDiskInfo(i: u32) -> *mut core::ffi::c_void;
    fn ConvertConfigToVA(ConfigurationRoot: *mut core::ffi::c_void);
    fn FrLdrHeapAlloc(Size: usize, Tag: u32) -> *mut core::ffi::c_void;
    fn IsAcpiPresent() -> u8;
    fn List_PaToVa(ListHead: *mut core::ffi::c_void);
    fn RtlStringCbCopyNA(psz_dest: *mut u8, cb_dest: usize, psz_src: *const u8, cb_to_copy: usize) -> i32;
    fn strstr(haystack: *const u8, needle: *const u8) -> *const u8;
    static mut WinLdrTerminalConnected: u8; // headless.c: BOOLEAN -- a u32 read took 3 bytes of whatever follows
    static mut LoaderRedirectionInformation: HEADLESS_LOADER_BLOCK;
    fn RegCloseKey(hKey: HKEY) -> i32;
    fn RegOpenKey(hKey: HKEY, lpSubKey: *const u16, phkResult: *mut HKEY) -> i32;
    fn RegQueryValue(hKey: HKEY, lpSubKey: *const u16, lpType: *mut u32, lpData: *mut u8, lpcbData: *mut u32) -> i32;
    fn FIXME(fmt: *const i8, ...);
    fn NtLdrGetOptionEx(BootOptions: PCSTR, Option: PCSTR, OptionLength: *mut u32) -> PCSTR;
    fn _strlwr(s: *mut u8) -> *mut u8;
    fn _strnicmp(s1: *const u8, s2: *const u8, n: usize) -> i32;
    fn strcspn(s: *const u8, reject: *const u8) -> usize;
    fn IniCleanup();
    fn MachHwDetect(BootOptions: PCSTR) -> *mut core::ffi::c_void;
    fn MachPrepareForReactOS();
    fn MmGetLoaderPagesSpanned() -> u32;
    fn TuiPrintf(fmt: PCSTR, ...);
    fn UiSetProgressBarSubset(min: u32, max: u32);
    fn UiSetProgressBarText(text: PCSTR);
    fn WinLdrSetProcessorContext(OperatingSystemVersion: u16);
    fn WinLdrSetupEms(BootOptions: PCSTR);
    fn WinLdrSetupMachineDependent(LoaderBlock: PLOADER_PARAMETER_BLOCK);
    fn WinLdrSetupMemoryLayout(LoaderBlock: PLOADER_PARAMETER_BLOCK);
    static mut PeLdrImportDllLoadCallback: *mut core::ffi::c_void;
    static mut CurrentControlSetKey: HKEY;
    fn GetArgumentValue(Argc: u32, Argv: *mut PCHAR, ArgumentName: PCSTR) -> PCSTR;
    fn UiDrawBackdrop(bottom: u32);
    fn UiGetScreenHeight() -> u32;
    fn UiDrawProgressBarCenter(text: PCSTR);
    fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32;
    fn strlen(s: PCSTR) -> usize;
    fn AppendBootTimeOptions(Options: PCHAR);
    fn RamDiskInitialize(bReportError: u8, LoadOptions: PCSTR, SystemPartition: PCSTR) -> ARC_STATUS;
    fn UiResetForSOS();
    fn DbgPrint(fmt: *const i8, ...) -> i32;
    fn WinLdrInitSystemHive(LoaderBlock: PLOADER_PARAMETER_BLOCK, BootPath: PCSTR, bUpdate: u8) -> u8;
    fn WinLdrScanSystemHive(LoaderBlock: PLOADER_PARAMETER_BLOCK, SystemRoot: PCSTR) -> u8;
}

// C declaration in peloader.h is `VOID (NTAPI *)(PCSTR)` → `extern "system"`
// (stdcall on i386). Was `extern "C"` (cdecl) — see commentary on
// `PELDR_IMPORTDLL_LOAD_CALLBACK` in ffi.rs for the i386 stack-corruption
// chain it caused before the fix.
unsafe extern "system" fn NtLdrImportDllLoadCallback(
    FileName: PCSTR)
{
    NtLdrOutputLoadMsg(FileName, core::ptr::null());
}

#[no_mangle]
pub unsafe extern "C" fn NtLdrOutputLoadMsg(
    FileName: PCSTR,
    Description: PCSTR)
{

    // Boot-phase timing stamp on the freeldr debug port (livecd.ini
    // [FREELOADER] Debug=): the framebuffer prints below are invisible on
    // serial, which made the whole load phase one silent multi-second gap.
    // tools/boot_phase_report.py turns these into per-file load timing.
    // Assembled from three raw prints: DbgPrint is non-variadic on the
    // stable channel (see debug.rs) and rosload imports it from freeldr.sys's
    // export table — no formatting, no new export needed.
    DbgPrint(b"[FLDR] load \0".as_ptr() as *const i8);
    DbgPrint(FileName as *const i8);
    DbgPrint(b"\n\0".as_ptr() as *const i8);

    if SosEnabled != 0 {
        printf(b"  %s\n\0".as_ptr() as PCSTR, FileName);
        TRACE(b"Loading: %s\n\0".as_ptr() as PCSTR, (FileName) as usize);
    } else {
        let mut ProgressString: [u8; 256] = [0; 256];

        RtlStringCbPrintfA(ProgressString.as_mut_ptr(), 
                          core::mem::size_of_val(&ProgressString),
                          b"Loading %s...\0".as_ptr() as PCSTR,
                          if !Description.is_null() { Description } else { FileName });
        UiDrawStatusText(ProgressString.as_ptr() as PCSTR);
    }
}

#[no_mangle]
pub unsafe extern "C" fn AllocateAndInitLPB(
    VersionToBoot: u16,
    OutLoaderBlock: *mut PLOADER_PARAMETER_BLOCK,
) {
    let LoaderBlock: PLOADER_PARAMETER_BLOCK;
    let Extension: PLOADER_PARAMETER_EXTENSION;

    WinLdrSystemBlock = MmAllocateMemoryWithType(
        core::mem::size_of::<LOADER_SYSTEM_BLOCK>(),
        LoaderSystemBlock,
    ) as *mut LOADER_SYSTEM_BLOCK;
    
    if WinLdrSystemBlock.is_null() {
        UiMessageBox(b"Failed to allocate memory for system block!\0".as_ptr() as *const i8);
        return;
    }

    RtlZeroMemory(
        WinLdrSystemBlock as *mut core::ffi::c_void,
        core::mem::size_of::<LOADER_SYSTEM_BLOCK>(),
    );

    LoaderBlock = core::ptr::addr_of_mut!((*WinLdrSystemBlock).LoaderBlock);
    (*LoaderBlock).NlsData = core::ptr::addr_of_mut!((*WinLdrSystemBlock).NlsDataBlock);

    Extension = core::ptr::addr_of_mut!((*WinLdrSystemBlock).Extension);
    (*LoaderBlock).Extension = Extension;
    (*Extension).Size = core::mem::size_of::<LOADER_PARAMETER_EXTENSION>() as u32;
    (*Extension).MajorVersion = ((VersionToBoot & 0xFF00) >> 8) as u32 ;
    (*Extension).MinorVersion = (VersionToBoot & 0xFF) as u32;

    // C: `#ifdef UEFIBOOT Extension->BootViaEFI = 1;`, which never fires: this code is
    // rosload, one binary for both loaders, and nothing defines UEFIBOOT for it. Ask the
    // first-stage loader instead. The bit is the only record past the loader of which
    // firmware booted us (setup installs a boot sector or a UEFI loader file by it).
    (*Extension).set_BootViaEFI(FrLdrIsUefiBoot() as u32);

    InitializeListHead(core::ptr::addr_of_mut!((*LoaderBlock).LoadOrderListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*LoaderBlock).MemoryDescriptorListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*LoaderBlock).BootDriverListHead));

    *OutLoaderBlock = LoaderBlock;
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrInitializePhase1(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    Options: PCSTR,
    SystemRoot: PCSTR,
    BootPath: PCSTR,
    VersionToBoot: u16,
) {

    let mut LoadOptions: PSTR;
    let mut NewLoadOptions: PSTR;
    let HalPath: [u8; 2] = *b"\\\0";
    let mut ArcBoot: [u8; 260 + 1] = [0; 261];
    let mut MiscFiles: [u8; 260 + 1] = [0; 261];
    let _i: u32;
    let PathSeparator: usize;
    let _Extension: PLOADER_PARAMETER_EXTENSION;

    PathSeparator = strstr(BootPath as *const u8 , b"\\\0".as_ptr()).offset_from(BootPath as *const u8 ) as usize;
    RtlStringCbCopyNA(ArcBoot.as_mut_ptr(), core::mem::size_of_val(&ArcBoot), BootPath as *const u8 , PathSeparator);

    TRACE(b"ArcBoot: '%s'\\n\0".as_ptr() as PCSTR, (ArcBoot.as_ptr()) as usize);
    TRACE(b"SystemRoot: '%s'\\n\0".as_ptr() as PCSTR, (SystemRoot) as usize);
    TRACE(b"Options: '%s'\\n\0".as_ptr() as PCSTR, (Options) as usize);

    (*LoaderBlock).ArcBootDeviceName = (*WinLdrSystemBlock).ArcBootDeviceName.as_mut_ptr();
    RtlStringCbCopyA((*LoaderBlock).ArcBootDeviceName, core::mem::size_of_val(&(*WinLdrSystemBlock).ArcBootDeviceName), ArcBoot.as_ptr() as *const i8 );
    (*LoaderBlock).ArcBootDeviceName = PaToVa((*LoaderBlock).ArcBootDeviceName as *mut core::ffi::c_void) as PSTR;

    if !(*LoaderBlock).SetupLdrBlock.is_null() {
        let SetupBlock: PSETUP_LOADER_BLOCK = (*LoaderBlock).SetupLdrBlock;

        (*SetupBlock).ArcSetupDeviceName = (*WinLdrSystemBlock).ArcBootDeviceName.as_mut_ptr();
        (*SetupBlock).ArcSetupDeviceName = PaToVa((*SetupBlock).ArcSetupDeviceName as *mut core::ffi::c_void) as PSTR;

        (*LoaderBlock).SetupLdrBlock = PaToVa((*LoaderBlock).SetupLdrBlock as *mut core::ffi::c_void) as PSETUP_LOADER_BLOCK;
    }

    (*LoaderBlock).ArcHalDeviceName = (*WinLdrSystemBlock).ArcBootDeviceName.as_mut_ptr();
    (*LoaderBlock).ArcHalDeviceName = PaToVa((*LoaderBlock).ArcHalDeviceName as *mut core::ffi::c_void) as PSTR;

    (*LoaderBlock).NtBootPathName = (*WinLdrSystemBlock).NtBootPathName.as_mut_ptr();
    RtlStringCbCopyA((*LoaderBlock).NtBootPathName, core::mem::size_of_val(&(*WinLdrSystemBlock).NtBootPathName), SystemRoot);
    (*LoaderBlock).NtBootPathName = PaToVa((*LoaderBlock).NtBootPathName as *mut core::ffi::c_void) as PSTR;

    (*LoaderBlock).NtHalPathName = (*WinLdrSystemBlock).NtHalPathName.as_mut_ptr();
    RtlStringCbCopyA((*LoaderBlock).NtHalPathName, core::mem::size_of_val(&(*WinLdrSystemBlock).NtHalPathName), HalPath.as_ptr() as *const i8 );
    (*LoaderBlock).NtHalPathName = PaToVa((*LoaderBlock).NtHalPathName as *mut core::ffi::c_void) as PSTR;

    NewLoadOptions = (*WinLdrSystemBlock).LoadOptions.as_mut_ptr();
    LoadOptions = NewLoadOptions;
    (*LoaderBlock).LoadOptions = LoadOptions;
    RtlStringCbCopyA((*LoaderBlock).LoadOptions, core::mem::size_of_val(&(*WinLdrSystemBlock).LoadOptions), Options);
    // If the loader's debug channel moved to an ACPI SPCR UART, KD must follow.
    // (rosload has no debug module; the BIOS second stage never takes SPCR.)
    #[cfg(not(feature = "rosload"))]
    crate::debug::SpcrRewriteDebugPort((*LoaderBlock).LoadOptions, core::mem::size_of_val(&(*WinLdrSystemBlock).LoadOptions));

    loop {
        while *LoadOptions == ((b'/') as i8) {
            LoadOptions = LoadOptions.offset(1);
        }

        *NewLoadOptions = *LoadOptions;
        NewLoadOptions = NewLoadOptions.offset(1);

        if *LoadOptions == 0 {
            break;
        }
        LoadOptions = LoadOptions.offset(1);
    }

    (*LoaderBlock).LoadOptions = PaToVa((*LoaderBlock).LoadOptions as *mut core::ffi::c_void) as PSTR;

    (*LoaderBlock).ArcDiskInformation = core::ptr::addr_of_mut!((*WinLdrSystemBlock).ArcDiskInformation);
    InitializeListHead(core::ptr::addr_of_mut!((*(*LoaderBlock).ArcDiskInformation).DiskSignatureListHead));

    let DiscCount = ArcGetDiskCount();
    let mut i = 0u32;
    while i < DiscCount {
        // ARC_DISK_SIGNATURE_EX = { ARC_DISK_SIGNATURE DiskSignature; CHAR ArcName[MAX_PATH]; }
        let ArcDiskSig = FrLdrHeapAlloc(
            core::mem::size_of::<ntkrnl_sys::ddk::_ARC_DISK_SIGNATURE>() + 260,
            u32::from_be_bytes(*b"giSD"),
        ) as *mut ntkrnl_sys::ddk::_ARC_DISK_SIGNATURE;
        if ArcDiskSig.is_null() {
            ERR(b"Failed to allocate ARC structure! Ignoring remaining ARC disks. (i = %lu, DiskCount = %lu)\\n\0".as_ptr() as PCSTR, (i) as usize, (DiscCount) as usize);
            break;
        }

        RtlCopyMemory(
            ArcDiskSig as *mut core::ffi::c_void,
            ArcGetDiskInfo(i),
            core::mem::size_of::<ntkrnl_sys::ddk::_ARC_DISK_SIGNATURE>() + 260,
        );

        // ArcName[] trails the ARC_DISK_SIGNATURE in ARC_DISK_SIGNATURE_EX
        let ArcNamePtr = (ArcDiskSig as *mut u8)
            .add(core::mem::size_of::<ntkrnl_sys::ddk::_ARC_DISK_SIGNATURE>());
        (*ArcDiskSig).ArcName = PaToVa(ArcNamePtr as *mut core::ffi::c_void) as PCHAR;

        InsertTailList(
            core::ptr::addr_of_mut!((*(*LoaderBlock).ArcDiskInformation).DiskSignatureListHead),
            core::ptr::addr_of_mut!((*ArcDiskSig).ListEntry),
        );

        i += 1;
    }

    List_PaToVa(core::ptr::addr_of_mut!((*(*LoaderBlock).ArcDiskInformation).DiskSignatureListHead) as *mut _);
    (*LoaderBlock).ArcDiskInformation = PaToVa((*LoaderBlock).ArcDiskInformation as *mut core::ffi::c_void) as *mut ntkrnl_sys::ddk::_ARC_DISK_INFORMATION;

    ConvertConfigToVA((*LoaderBlock).ConfigurationRoot as *mut core::ffi::c_void);
    (*LoaderBlock).ConfigurationRoot = PaToVa((*LoaderBlock).ConfigurationRoot as *mut core::ffi::c_void) as _;

    List_PaToVa(core::ptr::addr_of_mut!((*LoaderBlock).LoadOrderListHead) as *mut _);

    List_PaToVa(core::ptr::addr_of_mut!((*LoaderBlock).BootDriverListHead) as *mut _);

    let Extension = (*LoaderBlock).Extension;

    (*Extension).Profile.Status = 2;

    if IsAcpiPresent() != 0 {
        (*Extension).AcpiTable = 1 as *mut core::ffi::c_void;
    }

    if VersionToBoot >= _WIN32_WINNT_VISTA {
        (*Extension).set_BootViaWinload(1);
        (*Extension).LoaderPerformanceData = PaToVa(core::ptr::addr_of_mut!((*WinLdrSystemBlock).LoaderPerformanceData) as *mut core::ffi::c_void) as ntkrnl_sys::ddk::PLOADER_PERFORMANCE_DATA;

        InitializeListHead(core::ptr::addr_of_mut!((*Extension).BootApplicationPersistentData));
        List_PaToVa(core::ptr::addr_of_mut!((*Extension).BootApplicationPersistentData) as *mut _);
    }

    #[cfg(target_arch = "x86")]
    {
        if WinLdrTerminalConnected != 0 {
            (*Extension).HeadlessLoaderBlock = core::ptr::addr_of_mut!((*WinLdrSystemBlock).HeadlessLoaderBlock) as ntkrnl_sys::ddk::PHEADLESS_LOADER_BLOCK;
            RtlCopyMemory(
                (*Extension).HeadlessLoaderBlock as *mut core::ffi::c_void,
                &LoaderRedirectionInformation as *const _ as *const core::ffi::c_void,
                core::mem::size_of::<HEADLESS_LOADER_BLOCK>(),
            );
            (*Extension).HeadlessLoaderBlock = PaToVa((*Extension).HeadlessLoaderBlock as *mut core::ffi::c_void) as ntkrnl_sys::ddk::PHEADLESS_LOADER_BLOCK;
        }
    }

    RtlStringCbCopyA(MiscFiles.as_mut_ptr() as *mut i8 , core::mem::size_of_val(&MiscFiles), BootPath);
    RtlStringCbCatA(MiscFiles.as_mut_ptr(), core::mem::size_of_val(&MiscFiles), b"AppPatch\\drvmain.sdb\0".as_ptr() as *const i8 );
    (*Extension).DrvDBImage = PaToVa(WinLdrLoadModule(
        MiscFiles.as_ptr() as PCSTR,
        core::ptr::addr_of_mut!((*Extension).DrvDBSize),
        LoaderRegistryData,
    ));

    (*LoaderBlock).Extension = PaToVa((*LoaderBlock).Extension as *mut core::ffi::c_void) as PLOADER_PARAMETER_EXTENSION;

    TRACE(b"WinLdrInitializePhase1() completed\\n\0".as_ptr() as PCSTR);
}

unsafe fn WinLdrLoadDeviceDriver(
    LoadOrderListHead: PLIST_ENTRY,
    BootPath: PCSTR,
    FilePath: PUNICODE_STRING,
    Flags: u32,
    DriverDTE: *mut PLDR_DATA_TABLE_ENTRY,
) -> u8 {

    let mut FullPath: [u8; 1024] = [0; 1024];
    let mut DriverPath: [u8; 1024] = [0; 1024];
    let mut DllName: [u8; 1024] = [0; 1024];
    let DriverNamePos: *mut i8;
    let mut Success: u8;
    let mut DriverBase: *mut core::ffi::c_void = core::ptr::null_mut();

    RtlStringCbPrintfA(
        DriverPath.as_mut_ptr(),
        core::mem::size_of_val(&DriverPath),
        b"%wZ\0".as_ptr() as PCSTR,
        FilePath,
    );
    DriverNamePos = strrchr(DriverPath.as_ptr() as *const i8, '\\' as i32);
    if DriverNamePos != core::ptr::null_mut() {
        RtlStringCbCopyA(
            DllName.as_mut_ptr() as *mut i8 ,
            core::mem::size_of_val(&DllName),
            DriverNamePos.offset(1) as *const u8 as *const i8 ,
        );

        *DriverNamePos.offset(1) = 0;
    } else {
        RtlStringCbCopyA(
            DllName.as_mut_ptr() as *mut i8 ,
            core::mem::size_of_val(&DllName),
            DriverPath.as_ptr() as *const i8 ,
        );
        *DriverPath.as_mut_ptr() = 0;
    }

    TRACE(
        b"DriverPath: '%s', DllName: '%s', LPB\n\0".as_ptr() as PCSTR, (DriverPath.as_ptr()) as usize, (DllName.as_ptr()) as usize,
    );

    Success = PeLdrCheckForLoadedDll(LoadOrderListHead, DllName.as_ptr() as PCSTR, DriverDTE);
    if Success != 0 {
        return 1u8;
    }

    RtlStringCbPrintfA(
        FullPath.as_mut_ptr(),
        core::mem::size_of_val(&FullPath),
        b"%s%wZ\0".as_ptr() as PCSTR,
        BootPath,
        FilePath,
    );

    NtLdrOutputLoadMsg(FullPath.as_ptr() as PCSTR, core::ptr::null());
    Success = PeLdrLoadImage(
        FullPath.as_ptr() as PCSTR,
        LoaderBootDriver,
        core::ptr::addr_of_mut!(DriverBase),
    );
    if Success == 0 {
        ERR(
            b"PeLdrLoadImage('%s') failed\n\0".as_ptr() as PCSTR, (DllName.as_ptr()) as usize,
        );
        return 0u8;
    }

    Success = PeLdrAllocateDataTableEntry(
        LoadOrderListHead,
        DllName.as_ptr() as PCSTR,
        DllName.as_ptr() as PCSTR,
        PaToVa(DriverBase),
        DriverDTE,
    );
    if Success == 0 {
        ERR(
            b"PeLdrAllocateDataTableEntry('%s') failed\n\0".as_ptr() as PCSTR, (DllName.as_ptr()) as usize,
        );
        MmFreeMemory(DriverBase);
        return 0u8;
    }

    PeLdrInitSecurityCookie(*DriverDTE);

    (*(*DriverDTE)).Flags |= Flags;

    RtlStringCbPrintfA(
        FullPath.as_mut_ptr(),
        core::mem::size_of_val(&FullPath),
        b"%s%s\0".as_ptr() as PCSTR,
        BootPath,
        DriverPath.as_ptr(),
    );
    Success = PeLdrScanImportDescriptorTable(
        LoadOrderListHead,
        FullPath.as_ptr() as PCSTR,
        *DriverDTE,
    );
    if Success == 0 {
        ERR(
            b"PeLdrScanImportDescriptorTable('%s') failed\n\0".as_ptr() as PCSTR, (FullPath.as_ptr()) as usize,
        );
        PeLdrFreeDataTableEntry(*DriverDTE);
        MmFreeMemory(DriverBase);
        return 0u8;
    }

    return 1u8;
}

unsafe fn WinLdrLoadBootDrivers(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    BootPath: PCSTR,
) -> u8 {

    let mut NextBd: PLIST_ENTRY;
    let mut DriverNode: PBOOT_DRIVER_NODE;
    let mut BootDriver: PBOOT_DRIVER_LIST_ENTRY;
    let mut Success: u8;
    let mut ret: u8 = 1u8;

    NextBd = (*LoaderBlock).BootDriverListHead.Flink;
    while NextBd != core::ptr::addr_of_mut!((*LoaderBlock).BootDriverListHead) {
        DriverNode = (NextBd as *mut u8).offset(
            -(core::mem::offset_of!(BOOT_DRIVER_NODE, ListEntry) as isize)
            -(core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link) as isize)
        ) as PBOOT_DRIVER_NODE;
        BootDriver = core::ptr::addr_of_mut!((*DriverNode).ListEntry);

        NextBd = (*BootDriver).Link.Flink;

        UiIndicateProgress();
        Success = WinLdrLoadDeviceDriver(
            core::ptr::addr_of_mut!((*LoaderBlock).LoadOrderListHead),
            BootPath,
            core::ptr::addr_of_mut!((*BootDriver).FilePath),
            0,
            core::ptr::addr_of_mut!((*BootDriver).LdrEntry),
        );
        if Success != 0 {
            (*BootDriver).RegistryPath.Buffer = PaToVa((*BootDriver).RegistryPath.Buffer as *mut core::ffi::c_void) as *mut u16;
            (*BootDriver).FilePath.Buffer = PaToVa((*BootDriver).FilePath.Buffer as *mut core::ffi::c_void) as *mut u16;
            (*BootDriver).LdrEntry = PaToVa((*BootDriver).LdrEntry as *mut core::ffi::c_void) as *mut _LDR_DATA_TABLE_ENTRY;

            if !(*DriverNode).Group.Buffer.is_null() {
                (*DriverNode).Group.Buffer = PaToVa((*DriverNode).Group.Buffer as *mut core::ffi::c_void) as *mut u16;
            }
            (*DriverNode).Name.Buffer = PaToVa((*DriverNode).Name.Buffer as *mut core::ffi::c_void) as *mut u16;
        } else {
            ERR(
                b"Cannot load boot driver '%wZ'!\n\0".as_ptr() as PCSTR, (core::ptr::addr_of_mut!((*BootDriver).FilePath)) as usize,
            );
            // ERR is a trace channel nobody sees on UEFI; the kernel then never
            // hears of the driver, and a device that needs it just sits there.
            // (DbgPrint takes no arguments here, so narrow the path by hand.)
            let path = &(*BootDriver).FilePath;
            let mut name = [0u8; 128];
            let n = ((path.Length / 2) as usize).min(name.len() - 1);
            for i in 0..n {
                name[i] = (*path.Buffer.add(i) & 0x7F) as u8;
            }
            DbgPrint(b"[FLDR] boot driver not loaded, dropped from the boot driver list: \0".as_ptr() as *const i8);
            DbgPrint(name.as_ptr() as *const i8);
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            #[cfg(not(feature = "uefi"))]
            UiMessageBox(
                b"Cannot load boot driver '%wZ'!\0".as_ptr() as PCSTR,
                core::ptr::addr_of_mut!((*BootDriver).FilePath),
            );
            ret = 0u8;

            RemoveEntryList(core::ptr::addr_of_mut!((*BootDriver).Link));
        }
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrLoadModule(
    ModuleName: PCSTR,
    Size: *mut u32,
    MemoryType: u32) -> *mut core::ffi::c_void
{

    let mut FileId: u32 = 0;
    let PhysicalBase: *mut core::ffi::c_void;
    let mut FileInfo: FILEINFORMATION = core::mem::zeroed();
    let FileSize: u32;
    let mut Status: ARC_STATUS;
    let mut BytesRead: u32 = 0;

    *Size = 0;

    NtLdrOutputLoadMsg(ModuleName, core::ptr::null());
    Status = ArcOpen(ModuleName as PSTR, OpenReadOnly, &mut FileId);
    if Status != ESUCCESS {
        WARN(b"Error while opening '%s', Status: %u\n\0".as_ptr() as PCSTR, (ModuleName) as usize, (Status) as usize);
        return core::ptr::null_mut();
    }

    Status = ArcGetFileInformation(FileId, &mut FileInfo);
    if Status != ESUCCESS {
        ArcClose(FileId);
        return core::ptr::null_mut();
    }
    FileSize = FileInfo.EndingAddress.__bindgen_anon_1.LowPart;
    *Size = FileSize;

    PhysicalBase = MmAllocateMemoryWithType(FileSize as usize, MemoryType);
    if PhysicalBase.is_null() {
        ERR(b"Could not allocate memory for '%s'\n\0".as_ptr() as PCSTR, (ModuleName) as usize);
        ArcClose(FileId);
        return core::ptr::null_mut();
    }

    Status = ArcRead(FileId, PhysicalBase, FileSize, &mut BytesRead);
    ArcClose(FileId);
    if Status != ESUCCESS {
        WARN(b"Error while reading '%s', Status: %u\n\0".as_ptr() as PCSTR, (ModuleName) as usize, (Status) as usize);
        return core::ptr::null_mut();
    }

    TRACE(b"Loaded %s at 0x%x with size 0x%x\n\0".as_ptr() as PCSTR, (ModuleName) as usize, (PhysicalBase) as usize, (FileSize) as usize);

    return PhysicalBase;
}

unsafe fn WinLdrDetectVersion() -> USHORT {
    let mut hKey: HKEY = core::ptr::null_mut();
    
    let rc = RegOpenKey(
        CurrentControlSetKey,
        w!("Control\\Terminal Server").as_ptr() as *const u16,
        &mut hKey
    );
    
    if rc != ERROR_SUCCESS {
        return _WIN32_WINNT_NT4;
    }
    
    RegCloseKey(hKey);
    
    _WIN32_WINNT_WS03
}

unsafe fn LoadModule(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    Path: PCCH,
    File: PCCH,
    ImportName: PCCH,
    MemoryType: u32,
    Dte: *mut PLDR_DATA_TABLE_ENTRY,
    Percentage: u32,
) -> *mut core::ffi::c_void {

    let mut FullFileName: [u8; 260] = [0; 260];
    let mut ProgressString: [u8; 256] = [0; 256];
    let mut BaseAddress: *mut core::ffi::c_void = core::ptr::null_mut();

    RtlStringCbPrintfA(
        ProgressString.as_mut_ptr(),
        core::mem::size_of_val(&ProgressString),
        b"Loading %s...\0".as_ptr() as PCCH,
        File,
    );
    UiUpdateProgressBar(Percentage, ProgressString.as_ptr() as PCCH);

    RtlStringCbCopyA(
        FullFileName.as_mut_ptr() as *mut i8 ,
        core::mem::size_of_val(&FullFileName),
        Path,
    );
    RtlStringCbCatA(
        FullFileName.as_mut_ptr(),
        core::mem::size_of_val(&FullFileName),
        File,
    );

    NtLdrOutputLoadMsg(FullFileName.as_ptr() as PCCH, core::ptr::null());
    let Success = PeLdrLoadImage(FullFileName.as_ptr() as PCCH, MemoryType, &mut BaseAddress);
    if Success == 0 {
        ERR(b"PeLdrLoadImage('%s') failed\n\0".as_ptr() as PCCH, (File) as usize);
        return core::ptr::null_mut();
    }
    TRACE(
        b"%s loaded successfully at %p\n\0".as_ptr() as PCCH, (File) as usize, (BaseAddress) as usize,
    );

    let Success = PeLdrAllocateDataTableEntry(
        core::ptr::addr_of_mut!((*LoaderBlock).LoadOrderListHead),
        ImportName,
        FullFileName.as_ptr() as PCCH,
        PaToVa(BaseAddress),
        Dte,
    );
    if Success == 0 {
        ERR(
            b"PeLdrAllocateDataTableEntry('%s') failed\n\0".as_ptr() as PCCH, (FullFileName.as_ptr()) as usize,
        );
        MmFreeMemory(BaseAddress);
        return core::ptr::null_mut();
    }

    PeLdrInitSecurityCookie(*Dte);

    BaseAddress
}

unsafe fn WinLdrIsPaeSupported(
    OperatingSystemVersion: u16,
    _LoaderBlock: PLOADER_PARAMETER_BLOCK,
    BootOptions: PCSTR,
    _HalFileName: PCSTR,
    _KernelFileName: PSTR,
    _KernelFileNameSize: usize,
) -> u8 {

    let mut PaeEnabled: u8 = 0u8;
    let mut PaeDisabled: u8 = 0u8;
    let mut Result: u8;

    if (OperatingSystemVersion > 0x0400) &&
        (NtLdrGetOption(BootOptions, b"PAE\0".as_ptr() as PCSTR) != 0)
    {
        PaeEnabled = 1u8;
    }

    Result = PaeEnabled;

    if (OperatingSystemVersion > 0x0500) &&
        (NtLdrGetOption(BootOptions, b"NOPAE\0".as_ptr() as PCSTR) != 0)
    {
        PaeDisabled = 1u8;
    }

    if SafeBoot != 0 {
        PaeDisabled = 1u8;
    }

    TRACE(b"PaeEnabled %X, PaeDisabled %X\n\0".as_ptr() as PCSTR, (PaeEnabled) as usize, (PaeDisabled) as usize);

    if PaeDisabled != 0 {
        Result = 0u8;
    }

    if NoExecuteEnabled != 0 {
        Result = 1u8;
    }

    if Result != 0 {
        panic!("UNIMPLEMENTED");
    }

    Result
}

unsafe fn LoadWindowsCore(
    OperatingSystemVersion: u16,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    BootOptions: PCSTR,
    BootPath: PCSTR,
    KernelDTE: *mut PLDR_DATA_TABLE_ENTRY,
) -> u8 {

    let mut Success: u8;
    let mut Option: PCSTR;
    let mut OptionLength: u32 = 0;
    let KernelBase: *mut core::ffi::c_void;
    let HalBase: *mut core::ffi::c_void;
    let mut KdDllBase: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut HalDTE: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut KdDllDTE: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut DirPath: [u8; 260] = [0; 260];
    let mut HalFileName: [u8; 260] = [0; 260];
    let mut KernelFileName: [u8; 260] = [0; 260];
    let mut KdDllName: [u8; 260] = [0; 260];

    if KernelDTE.is_null() {
        return 0u8;
    }

    RtlStringCbCopyA(DirPath.as_mut_ptr() as *mut i8 , 260, BootPath);
    RtlStringCbCatA(DirPath.as_mut_ptr(), 260, b"system32\\\0".as_ptr() as *const i8 );

    TRACE(b"LoadWindowsCore: BootOptions '%s'\n\0".as_ptr() as *const i8 , (BootOptions) as usize);

    #[cfg(target_arch = "x86")]
    {
        if NtLdrGetOption(BootOptions, (b"3GB\0".as_ptr() as *const i8)) != 0 {
            FIXME((b"LoadWindowsCore: 3GB - TRUE (not implemented)\n\0".as_ptr() as *const i8));
            VirtualBias = 1u8;
        }
    }

    if (OperatingSystemVersion > 0x0400) &&
        (NtLdrGetOption(BootOptions, b"SAFEBOOT\0".as_ptr() as *const i8 ) != 0 ||
         NtLdrGetOption(BootOptions, b"SAFEBOOT:\0".as_ptr() as *const i8 ) != 0)
    {
        FIXME(b"LoadWindowsCore: SAFEBOOT - TRUE (not implemented)\n\0".as_ptr() as *const i8 );
        SafeBoot = 1u8;
    }

    if (OperatingSystemVersion > 0x0500) &&
        NtLdrGetOption(BootOptions, b"BOOTLOGO\0".as_ptr() as *const i8 ) != 0
    {
        FIXME(b"LoadWindowsCore: BOOTLOGO - TRUE (not implemented)\n\0".as_ptr() as *const i8 );
        BootLogo = 1u8;
    }

    if (OperatingSystemVersion > 0x0500) &&
        (*LoaderBlock).SetupLdrBlock.is_null()
    {
        #[cfg(target_arch = "x86")]
        {
            NoExecuteEnabled = 0u8;
        }
        #[cfg(not(target_arch = "x86"))]
        {
            NoExecuteEnabled = 1u8;
        }

        #[cfg(target_arch = "x86")]
        {
            if NtLdrGetOption(BootOptions, (b"NOEXECUTE=OPTIN\0".as_ptr() as *const i8)) != 0 ||
                NtLdrGetOption(BootOptions, (b"NOEXECUTE=OPTOUT\0".as_ptr() as *const i8)) != 0 ||
                NtLdrGetOption(BootOptions, (b"NOEXECUTE=ALWAYSON\0".as_ptr() as *const i8)) != 0
            {
                NoExecuteEnabled = 1u8;
            } else if NtLdrGetOption(BootOptions, (b"NOEXECUTE=ALWAYSOFF\0".as_ptr() as *const i8)) != 0 {
                NoExecuteEnabled = 0u8;
            } else if NtLdrGetOption(BootOptions, (b"NOEXECUTE\0".as_ptr() as *const i8)) != 0 {
                NoExecuteEnabled = 1u8;
            } else if NtLdrGetOption(BootOptions, (b"EXECUTE\0".as_ptr() as *const i8)) != 0 {
                NoExecuteEnabled = 0u8;
            }

            if SafeBoot != 0 {
                NoExecuteEnabled = 0u8;
            }
        }
        #[cfg(not(target_arch = "x86"))]
        {
            if NtLdrGetOption(BootOptions, b"NOEXECUTE\0".as_ptr() as PCSTR) != 0 {
                NoExecuteEnabled = 1u8;
            } else if NtLdrGetOption(BootOptions, b"EXECUTE\0".as_ptr() as PCSTR) != 0 {
                NoExecuteEnabled = 0u8;
            }
        }
    }
    TRACE(b"NoExecuteEnabled %X\n\0".as_ptr() as *const i8 , (NoExecuteEnabled) as usize);

    RtlStringCbCopyA(HalFileName.as_mut_ptr() as *mut i8 , 260, b"hal.dll\0".as_ptr() as *const i8 );
    RtlStringCbCopyA(KernelFileName.as_mut_ptr() as *mut i8 , 260, b"ntoskrnl.exe\0".as_ptr() as *const i8 );

    Option = NtLdrGetOptionEx(BootOptions, b"HAL=\0".as_ptr() as *const i8 , core::ptr::addr_of_mut!(OptionLength));
    if !Option.is_null() && (OptionLength > 4) {
        Option = Option.add(4);
        OptionLength -= 4;
        RtlStringCbCopyNA(HalFileName.as_mut_ptr(), 260, Option as *const u8 , OptionLength as usize);
        _strlwr(HalFileName.as_mut_ptr());
    }

    Option = NtLdrGetOptionEx(BootOptions, b"KERNEL=\0".as_ptr() as *const i8 , core::ptr::addr_of_mut!(OptionLength));
    if !Option.is_null() && (OptionLength > 7) {
        Option = Option.add(7);
        OptionLength -= 7;
        RtlStringCbCopyNA(KernelFileName.as_mut_ptr(), 260, Option as *const u8 , OptionLength as usize);
        _strlwr(KernelFileName.as_mut_ptr());
    }

    #[cfg(target_arch = "x86")]
    {
        PaeModeOn = WinLdrIsPaeSupported(
            OperatingSystemVersion,
            LoaderBlock,
            BootOptions,
            (HalFileName.as_ptr() as *const i8),
            (KernelFileName.as_mut_ptr() as *mut i8),
            260,
        );
        if PaeModeOn != 0 {
            FIXME((b"WinLdrIsPaeSupported: PaeModeOn\n\0".as_ptr() as *const i8));
        }
    }

    TRACE(b"HAL file = '%s' ; Kernel file = '%s'\n\0".as_ptr() as *const i8 , (HalFileName.as_ptr()) as usize, (KernelFileName.as_ptr()) as usize);

    KernelBase = LoadModule(
        LoaderBlock,
        DirPath.as_ptr() as *const i8 ,
        KernelFileName.as_ptr() as *const i8 ,
        b"ntoskrnl.exe\0".as_ptr() as *const i8 ,
        LoaderSystemCode,
        KernelDTE,
        30,
    );
    if KernelBase.is_null() {
        ERR(b"LoadModule('%s') failed\n\0".as_ptr() as *const i8 , (KernelFileName.as_ptr()) as usize);
        UiMessageBox(b"Could not load %s\0".as_ptr() as *const i8 , KernelFileName.as_ptr());
        return 0u8;
    }

    HalBase = LoadModule(
        LoaderBlock,
        DirPath.as_ptr() as *const i8 ,
        HalFileName.as_ptr() as *const i8 ,
        b"hal.dll\0".as_ptr() as *const i8 ,
        LoaderHalCode,
        core::ptr::addr_of_mut!(HalDTE),
        35,
    );
    if HalBase.is_null() {
        ERR(b"LoadModule('%s') failed\n\0".as_ptr() as *const i8 , (HalFileName.as_ptr()) as usize);
        UiMessageBox(b"Could not load %s\0".as_ptr() as *const i8 , HalFileName.as_ptr());
        PeLdrFreeDataTableEntry(*KernelDTE);
        MmFreeMemory(KernelBase);
        return 0u8;
    }

    if OperatingSystemVersion > 0x0500 {
        let mut IsCustomKdDll: u8 = 0u8;

        Option = NtLdrGetOptionEx(BootOptions, b"DEBUGPORT=\0".as_ptr() as *const i8 , core::ptr::addr_of_mut!(OptionLength));
        if !Option.is_null() && (OptionLength > 10) {
            Option = Option.add(10);
            OptionLength -= 10;

            OptionLength = strcspn(Option as *const u8 , b" \t:\0".as_ptr()) as u32;

            if (OptionLength == 0) ||
                ((OptionLength >= 3) && (_strnicmp(Option as *const u8 , b"COM\0".as_ptr(), 3) == 0) &&
                 ((OptionLength == 3) || (*Option.add(3) >= ((b'0') as i8) && *Option.add(3) <= ((b'9') as i8))))
            {
            } else {
                IsCustomKdDll = 1u8;
            }
        }
        if IsCustomKdDll == 0 {
            Option = b"COM\0".as_ptr() as *const i8 ;
            OptionLength = 3;
        }

        RtlStringCbPrintfA(
            KdDllName.as_mut_ptr(),
            260,
            b"kd%.*s.dll\0".as_ptr() as *const i8 ,
            OptionLength,
            Option,
        );
        _strlwr(KdDllName.as_mut_ptr());

        KdDllBase = LoadModule(
            LoaderBlock,
            DirPath.as_ptr() as *const i8 ,
            KdDllName.as_ptr() as *const i8 ,
            b"kdcom.dll\0".as_ptr() as *const i8 ,
            LoaderSystemCode,
            core::ptr::addr_of_mut!(KdDllDTE),
            40,
        );
        if KdDllBase.is_null() {
            if IsCustomKdDll != 0 {
                WARN(b"LoadModule('%s') failed\n\0".as_ptr() as *const i8 , (KdDllName.as_ptr()) as usize);

                IsCustomKdDll = 0u8;
                RtlStringCbCopyA(KdDllName.as_mut_ptr() as *mut i8 , 260, b"kdcom.dll\0".as_ptr() as *const i8 );

                KdDllBase = LoadModule(
                    LoaderBlock,
                    DirPath.as_ptr() as *const i8 ,
                    KdDllName.as_ptr() as *const i8 ,
                    b"kdcom.dll\0".as_ptr() as *const i8 ,
                    LoaderSystemCode,
                    core::ptr::addr_of_mut!(KdDllDTE),
                    40,
                );
            }

            if KdDllBase.is_null() {
                ERR(b"LoadModule('%s') failed\n\0".as_ptr() as *const i8 , (KdDllName.as_ptr()) as usize);
            }
        }
    }

    Success = PeLdrScanImportDescriptorTable(
        core::ptr::addr_of_mut!((*LoaderBlock).LoadOrderListHead),
        DirPath.as_ptr() as *const i8 ,
        *KernelDTE,
    );
    if Success == 0 {
        UiMessageBox(b"Could not load %s\0".as_ptr() as *const i8 , KernelFileName.as_ptr());
    } else {
        Success = PeLdrScanImportDescriptorTable(
            core::ptr::addr_of_mut!((*LoaderBlock).LoadOrderListHead),
            DirPath.as_ptr() as *const i8 ,
            HalDTE,
        );
        if Success == 0 {
            UiMessageBox(b"Could not load %s\0".as_ptr() as *const i8 , HalFileName.as_ptr());
        } else if !KdDllDTE.is_null() {
            Success = PeLdrScanImportDescriptorTable(
                core::ptr::addr_of_mut!((*LoaderBlock).LoadOrderListHead),
                DirPath.as_ptr() as *const i8 ,
                KdDllDTE,
            );
            if Success == 0 {
                UiMessageBox(b"Could not load %s\0".as_ptr() as *const i8 , KdDllName.as_ptr());
            }
        }
    }

    if Success == 0 {
        if !KdDllDTE.is_null() {
            PeLdrFreeDataTableEntry(KdDllDTE);
        }
        if !KdDllBase.is_null() {
            MmFreeMemory(KdDllBase);
        }

        PeLdrFreeDataTableEntry(HalDTE);
        MmFreeMemory(HalBase);

        PeLdrFreeDataTableEntry(*KernelDTE);
        MmFreeMemory(KernelBase);
    }

    Success
}

unsafe fn WinLdrInitErrataInf(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    OperatingSystemVersion: u16,
    SystemRoot: PCSTR) -> u8
{

    let mut rc: i32;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut BufferSize: u32;
    let mut FileSize: u32 = 0;
    let PhysicalBase: *mut core::ffi::c_void;
    let mut szFileName: [u16; 80] = [0; 80];
    let mut ErrataFilePath: [u8; 260] = [0; 260];

    if OperatingSystemVersion >= 0x0600 {
        rc = RegOpenKey(CurrentControlSetKey, 
                       w!("Control\\Errata").as_ptr() as *const u16,
                       &mut hKey);
    } else {
        rc = RegOpenKey(CurrentControlSetKey,
                       w!("Control\\BiosInfo").as_ptr() as *const u16,
                       &mut hKey);
    }
    
    if rc != 0 {
        WARN(b"Could not open the BiosInfo/Errata registry key (Error %u)\n\0".as_ptr() as PCSTR, (rc as i32) as usize);
        return 0u8;
    }

    BufferSize = core::mem::size_of::<[u16; 80]>() as u32;
    rc = RegQueryValue(hKey,
                      w!("InfName").as_ptr() as *const u16,
                      core::ptr::null_mut(),
                      szFileName.as_mut_ptr() as *mut u8,
                      &mut BufferSize);
    if rc != 0 {
        WARN(b"Could not retrieve the InfName value (Error %u)\n\0".as_ptr() as PCSTR, (rc as i32) as usize);
        RegCloseKey(hKey);
        return 0u8;
    }

    RegCloseKey(hKey);

    RtlStringCbPrintfA(ErrataFilePath.as_mut_ptr() as PSTR as *mut u8 ,
                      core::mem::size_of::<[u8; 260]>(),
                      b"%s%s%S\0".as_ptr() as PCSTR,
                      SystemRoot,
                      b"inf\\\0".as_ptr() as PCSTR,
                      szFileName.as_ptr());

    PhysicalBase = WinLdrLoadModule(ErrataFilePath.as_ptr() as PCSTR,
                                    &mut FileSize,
                                    LoaderRegistryData);
    if PhysicalBase.is_null() {
        WARN(b"Could not load '%s'\n\0".as_ptr() as PCSTR, (ErrataFilePath.as_ptr() as PCSTR) as usize);
        return 0u8;
    }

    (*(*LoaderBlock).Extension).EmInfFileImage = PaToVa(PhysicalBase);
    (*(*LoaderBlock).Extension).EmInfFileSize = FileSize;

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn LoadAndBootWindows(
    Argc: u32,
    Argv: *mut PCHAR,
    _Envp: *mut PCHAR,
) -> ARC_STATUS {

    let Status: ARC_STATUS;
    let mut ArgValue: PCSTR;
    let SystemPartition: PCSTR;
    let mut FileName: PCSTR;
    let mut FileNameLength: u32 = 0;
    let mut Success: u8;
    let mut OperatingSystemVersion: u16;
    let mut LoaderBlock: PLOADER_PARAMETER_BLOCK = core::ptr::null_mut();
    let mut BootPath: [i8; 260] = [0; 260];
    let mut FilePath: [i8; 260] = [0; 260];
    let mut BootOptions: [i8; 256] = [0; 256];

    ArgValue = GetArgumentValue(Argc, Argv, b"BootType\0".as_ptr() as PCSTR);
    if ArgValue.is_null() || *ArgValue == 0 {
        ERR(b"No 'BootType' value, aborting!\n\0".as_ptr() as PCSTR);
        return EINVAL;
    }

    if _stricmp(ArgValue, b"Windows\0".as_ptr() as PCSTR) == 0 ||
        _stricmp(ArgValue, b"Windows2003\0".as_ptr() as PCSTR) == 0 {
        OperatingSystemVersion = _WIN32_WINNT_WS03;
    } else if _stricmp(ArgValue, b"WindowsNT40\0".as_ptr() as PCSTR) == 0 {
        OperatingSystemVersion = _WIN32_WINNT_NT4;
    } else if _stricmp(ArgValue, b"WindowsVista\0".as_ptr() as PCSTR) == 0 {
        OperatingSystemVersion = _WIN32_WINNT_VISTA;
    } else {
        ERR(b"Unknown 'BootType' value '%s', aborting!\n\0".as_ptr() as PCSTR, (ArgValue) as usize);
        return EINVAL;
    }

    SystemPartition = GetArgumentValue(Argc, Argv, b"SystemPartition\0".as_ptr() as PCSTR);
    if SystemPartition.is_null() || *SystemPartition == 0 {
        ERR(b"No 'SystemPartition' specified, aborting!\n\0".as_ptr() as PCSTR);
        return EINVAL;
    }

    UiDrawBackdrop(UiGetScreenHeight());
    UiDrawStatusText(b"Loading...\0".as_ptr() as PCSTR);
    UiDrawProgressBarCenter(b"Loading NT...\0".as_ptr() as PCSTR);

    *BootPath.as_mut_ptr() = 0;
    ArgValue = GetArgumentValue(Argc, Argv, b"SystemPath\0".as_ptr() as PCSTR);
    if !ArgValue.is_null() {
        RtlStringCbCopyA(BootPath.as_mut_ptr() as PCHAR, core::mem::size_of_val(&BootPath), ArgValue);
    }

    if strrchr(BootPath.as_ptr() as PCSTR, ')' as i32).is_null() {
        RtlStringCbCopyA(FilePath.as_mut_ptr() as PCHAR, core::mem::size_of_val(&FilePath), BootPath.as_ptr() as PCSTR);

        RtlStringCbCopyA(BootPath.as_mut_ptr() as PCHAR, core::mem::size_of_val(&BootPath), SystemPartition);

        if *FilePath.as_ptr() != b'\\' as i8 && *FilePath.as_ptr() != b'/' as i8 {
            RtlStringCbCatA(BootPath.as_mut_ptr() as PCHAR as *mut u8 , core::mem::size_of_val(&BootPath), b"\\\0".as_ptr() as PCSTR);
        }

        RtlStringCbCatA(BootPath.as_mut_ptr() as PCHAR as *mut u8 , core::mem::size_of_val(&BootPath), FilePath.as_ptr() as PCSTR);
    }

    if *BootPath.as_ptr() == 0 || BootPath[strlen(BootPath.as_ptr() as PCSTR) - 1] != b'\\' as i8 {
        RtlStringCbCatA(BootPath.as_mut_ptr() as PCHAR as *mut u8 , core::mem::size_of_val(&BootPath), b"\\\0".as_ptr() as PCSTR);
    }

    TRACE(b"BootPath: '%s'\n\0".as_ptr() as PCSTR, (BootPath.as_ptr()) as usize);

    *BootOptions.as_mut_ptr() = 0;
    ArgValue = GetArgumentValue(Argc, Argv, b"Options\0".as_ptr() as PCSTR);
    if !ArgValue.is_null() && *ArgValue != 0 {
        RtlStringCbCopyA(BootOptions.as_mut_ptr() as PCHAR, core::mem::size_of_val(&BootOptions), ArgValue);
    }

    AppendBootTimeOptions(BootOptions.as_mut_ptr() as PCHAR);

    if NtLdrGetOption(BootOptions.as_ptr() as PCSTR, b"HAL=\0".as_ptr() as PCSTR) == 0 {
        ArgValue = GetArgumentValue(Argc, Argv, b"Hal\0".as_ptr() as PCSTR);
        if !ArgValue.is_null() && *ArgValue != 0 {
            RtlStringCbCatA(BootOptions.as_mut_ptr() as PCHAR as *mut u8 , core::mem::size_of_val(&BootOptions), b" /HAL=\0".as_ptr() as PCSTR);
            RtlStringCbCatA(BootOptions.as_mut_ptr() as PCHAR as *mut u8 , core::mem::size_of_val(&BootOptions), ArgValue);
        }
    }
    if NtLdrGetOption(BootOptions.as_ptr() as PCSTR, b"KERNEL=\0".as_ptr() as PCSTR) == 0 {
        ArgValue = GetArgumentValue(Argc, Argv, b"Kernel\0".as_ptr() as PCSTR);
        if !ArgValue.is_null() && *ArgValue != 0 {
            RtlStringCbCatA(BootOptions.as_mut_ptr() as PCHAR as *mut u8 , core::mem::size_of_val(&BootOptions), b" /KERNEL=\0".as_ptr() as PCSTR);
            RtlStringCbCatA(BootOptions.as_mut_ptr() as PCHAR as *mut u8 , core::mem::size_of_val(&BootOptions), ArgValue);
        }
    }

    TRACE(b"BootOptions: '%s'\n\0".as_ptr() as PCSTR, (BootOptions.as_ptr()) as usize);

    FileName = NtLdrGetOptionEx(BootOptions.as_ptr() as PCSTR, b"RDPATH=\0".as_ptr() as PCSTR, &mut FileNameLength);
    if !FileName.is_null() && FileNameLength >= 7 {
        Status = RamDiskInitialize(0u8, BootOptions.as_ptr() as PCSTR, SystemPartition);
        if Status != ESUCCESS {
            FileName = FileName.offset(7);
            FileNameLength -= 7;
            UiMessageBox(b"Failed to load RAM disk file '%.*s'\0".as_ptr() as PCSTR, FileNameLength, FileName);
            return Status;
        }
    }

    SosEnabled = if NtLdrGetOption(BootOptions.as_ptr() as PCSTR, b"SOS\0".as_ptr() as PCSTR) != 0 { 1u8 } else { 0u8 };
    if SosEnabled != 0 {
        UiResetForSOS();
    }

    AllocateAndInitLPB(OperatingSystemVersion, &mut LoaderBlock);

    DbgPrint(b"[FLDR] phase: system hive load begin\n\0".as_ptr() as *const i8);
    UiUpdateProgressBar(15, b"Loading system hive...\0".as_ptr() as PCSTR);
    Success = WinLdrInitSystemHive(LoaderBlock, BootPath.as_ptr() as PCSTR, 0u8);
    TRACE(b"SYSTEM hive %s\n\0".as_ptr() as PCSTR, (if Success != 0 { b"loaded\0".as_ptr() } else { b"not loaded\0".as_ptr() }) as usize);
    if Success == 0 {
        return ENOEXEC;
    }

    if OperatingSystemVersion == 0 {
        OperatingSystemVersion = WinLdrDetectVersion();
    }
    (*(*LoaderBlock).Extension).MajorVersion = ((OperatingSystemVersion & 0xFF00) >> 8) as u32 ;
    (*(*LoaderBlock).Extension).MinorVersion = (OperatingSystemVersion & 0xFF) as u32;

    DbgPrint(b"[FLDR] phase: system hive loaded\n\0".as_ptr() as *const i8);
    Success = WinLdrScanSystemHive(LoaderBlock, BootPath.as_ptr() as PCSTR);
    TRACE(b"SYSTEM hive %s\n\0".as_ptr() as PCSTR, (if Success != 0 { b"scanned\0".as_ptr() } else { b"not scanned\0".as_ptr() }) as usize);
    if Success == 0 {
        return ENOEXEC;
    }

    Success = WinLdrInitErrataInf(LoaderBlock, OperatingSystemVersion, BootPath.as_ptr() as PCSTR);
    TRACE(b"Firmware Errata file %s\n\0".as_ptr() as PCSTR, (if Success != 0 { b"loaded\0".as_ptr() } else { b"not loaded\0".as_ptr() }) as usize);

    return LoadAndBootWindowsCommon(OperatingSystemVersion, LoaderBlock, BootOptions.as_ptr() as PCSTR, BootPath.as_ptr() as PCSTR);
}

#[no_mangle]
pub unsafe extern "C" fn LoadAndBootWindowsCommon(
    OperatingSystemVersion: u16,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    BootOptions: PCSTR,
    BootPath: PCSTR,
) -> ARC_STATUS {

    let LoaderBlockVA: PLOADER_PARAMETER_BLOCK;
    let mut Success: u8;
    let mut KernelDTE: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let KiSystemStartup: unsafe extern "C" fn(PLOADER_PARAMETER_BLOCK);
    let SystemRoot: PCSTR;

    #[cfg(target_arch = "x86")]
    {
        WinLdrSetupEms(BootOptions);
    }

    SystemRoot = strstr(BootPath as *const u8 , b"\\\0".as_ptr() as PCSTR as *const u8 ) as *const i8 ;

    DbgPrint(b"[FLDR] phase: hw-detect begin\n\0".as_ptr() as *const i8);
    UiUpdateProgressBar(20, b"Detecting hardware...\0".as_ptr() as PCSTR);
    (*LoaderBlock).ConfigurationRoot = MachHwDetect(BootOptions) as _;

    PeLdrImportDllLoadCallback = NtLdrImportDllLoadCallback as *mut core::ffi::c_void;

    DbgPrint(b"[FLDR] phase: core (kernel/HAL/kd) load begin\n\0".as_ptr() as *const i8);
    Success = LoadWindowsCore(
        OperatingSystemVersion,
        LoaderBlock,
        BootOptions,
        BootPath,
        core::ptr::addr_of_mut!(KernelDTE),
    );
    if Success == 0 {
        PeLdrImportDllLoadCallback = core::ptr::null_mut();

        UiMessageBox(b"Error loading NTOS core.\0".as_ptr() as PCSTR);
        return ENOEXEC;
    }

    IniCleanup();

    UiSetProgressBarSubset(40, 90);

    DbgPrint(b"[FLDR] phase: boot drivers load begin\n\0".as_ptr() as *const i8);
    UiSetProgressBarText(b"Loading boot drivers...\0".as_ptr() as PCSTR);
    Success = WinLdrLoadBootDrivers(LoaderBlock, BootPath);

    UiSetProgressBarSubset(0, 100);

    PeLdrImportDllLoadCallback = core::ptr::null_mut();

    DbgPrint(b"[FLDR] phase: boot drivers loaded\n\0".as_ptr() as *const i8);
    WinLdrInitializePhase1(
        LoaderBlock,
        BootOptions,
        SystemRoot,
        BootPath,
        OperatingSystemVersion,
    );

    UiUpdateProgressBar(100, core::ptr::null_mut());

    KiSystemStartup = core::mem::transmute((*KernelDTE).EntryPoint);
    LoaderBlockVA = PaToVa(LoaderBlock as *mut core::ffi::c_void) as PLOADER_PARAMETER_BLOCK;

    MachPrepareForReactOS();

    if NtLdrGetOption(BootOptions, b"CRASHDEBUG\0".as_ptr() as PCSTR) == 0 &&
        NtLdrGetOption(BootOptions, b"NODEBUG\0".as_ptr() as PCSTR) == 0 &&
        NtLdrGetOption(BootOptions, b"DEBUG\0".as_ptr() as PCSTR) != 0
    {
        let mut DebugPortLength: u32 = 0;
        let mut DebugPort: PCSTR = NtLdrGetOptionEx(BootOptions, b"DEBUGPORT=\0".as_ptr() as PCSTR, core::ptr::addr_of_mut!(DebugPortLength));
        if !DebugPort.is_null() && DebugPortLength > 10 {
            DebugPort = DebugPort.offset(10);
            DebugPortLength -= 10;
        } else {
            DebugPort = b"COM\0".as_ptr() as PCSTR;
            DebugPortLength = 3;
        }

        #[cfg(feature = "uefi")]
        {
            ERR(
                b"Debug mode: connect debugger on port %.*s\n\0".as_ptr() as PCSTR, (DebugPortLength) as usize, (DebugPort) as usize,
            );
        }
        #[cfg(not(feature = "uefi"))]
        {
            TuiPrintf(
                b"You need to connect a debugger on port %.*s\nFor more information, visit https://reactos.org/wiki/Debugging.\n\0".as_ptr() as PCSTR,
                DebugPortLength,
                DebugPort,
            );
        }
    }

    WinLdrSetupMachineDependent(LoaderBlock);

    #[cfg(target_arch = "aarch64")]
    {
        ERR(b">>> WinLdrSetupMemoryLayout...\n\0".as_ptr() as PCSTR);
    }

    WinLdrSetupMemoryLayout(LoaderBlock);

    #[cfg(target_arch = "aarch64")]
    {
        ERR(b">>> WinLdrSetProcessorContext...\n\0".as_ptr() as PCSTR);
    }

    WinLdrSetProcessorContext(OperatingSystemVersion);

    #[cfg(target_arch = "aarch64")]
    {
        ERR(
            b">>> Page tables active! Extension=%p KiSystemStartup=%p\n\0".as_ptr() as PCSTR, ((*LoaderBlock).Extension) as usize, (KiSystemStartup as *const core::ffi::c_void) as usize,
        );
        ERR(b">>> Writing LoaderPagesSpanned...\n\0".as_ptr() as PCSTR);
    }

    // On real ARM64 hardware this store was the first thing to fault after the
    // page-table switch (Pi 4B, 2026-09-20): the Extension is reached through a
    // PaToVa kernel VA, so it needs a TTBR1 mapping that the QEMU virt layout
    // happens to get and this board did not. dbgcon, not ERR, because ERR is
    // `#if DBG` and vanishes from the release build that ships to hardware.
    #[cfg(target_arch = "aarch64")]
    {
        crate::frldr::dbgcon_str(b"[FLDR] pre-spanned: LB=0x");
        crate::frldr::dbgcon_hex64(LoaderBlock as u64);
        crate::frldr::dbgcon_str(b" Ext=0x");
        crate::frldr::dbgcon_hex64((*LoaderBlock).Extension as u64);
        crate::frldr::dbgcon_str(b" spanned=0x");
        crate::frldr::dbgcon_hex64(MmGetLoaderPagesSpanned() as u64);
        crate::frldr::dbgcon_str(b" mapfail=0x");
        crate::frldr::dbgcon_hex32(crate::winldr_arm64::Arm64MapFailures);
        crate::frldr::dbgcon_str(b"\r\n");
        // NOT a page-table walk here any more. Arm64ReportMapping reads the
        // tables through their physical addresses, which the firmware's identity
        // map made safe; once the loader's own tables are live only what we
        // mapped is reachable, and walking them from here faulted on a table
        // page that is not itself mapped. mapfail above is the cheap check that
        // survives; walk before WinLdrSetProcessorContext if a walk is needed.
    }

    // LoaderPagesSpanned is ULONG_PTR (u32 on i386, u64 on amd64); the local
    // MmGetLoaderPagesSpanned() extern is declared -> u32. `as _` coerces to
    // the field's per-arch width (identity on i386, widen on amd64).
    (*(*LoaderBlock).Extension).LoaderPagesSpanned = MmGetLoaderPagesSpanned() as _;

    #[cfg(target_arch = "aarch64")]
    crate::frldr::dbgcon_str(b"[FLDR] spanned stored\r\n");

    #[cfg(target_arch = "aarch64")]
    {
        ERR(
            b">>> Zeroing KI_USER_SHARED_DATA at %p...\n\0".as_ptr() as PCSTR, (KI_USER_SHARED_DATA as *const core::ffi::c_void) as usize,
        );
    }

    RtlZeroMemory(KI_USER_SHARED_DATA as *mut core::ffi::c_void, 4096);

    #[cfg(target_arch = "aarch64")]
    {
        ERR(
            b">>> Calling KiSystemStartup(%p)...\n\0".as_ptr() as PCSTR, (LoaderBlockVA) as usize,
        );
    }

    DbgPrint(b"[FLDR] phase: handoff to kernel\n\0".as_ptr() as *const i8);
    WinLdrpDumpMemoryDescriptors(LoaderBlockVA);
    WinLdrpDumpBootDriver(LoaderBlockVA);
    #[cfg(not(target_arch = "x86_64"))]
    {
        WinLdrpDumpArcDisks(LoaderBlockVA);
    }

    (KiSystemStartup)(LoaderBlockVA);

    core::hint::unreachable_unchecked()
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrpDumpMemoryDescriptors(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let mut NextMd: PLIST_ENTRY = (*LoaderBlock).MemoryDescriptorListHead.Flink;
    
    while NextMd != core::ptr::addr_of_mut!((*LoaderBlock).MemoryDescriptorListHead) {
        let MemoryDescriptor: PMEMORY_ALLOCATION_DESCRIPTOR = 
            (NextMd as *mut u8).offset(
                -(core::mem::offset_of!(MEMORY_ALLOCATION_DESCRIPTOR, ListEntry) as isize)
            ) as PMEMORY_ALLOCATION_DESCRIPTOR;
        
        
        TRACE(
            b"BP %08X PC %04X MT %d\n\0".as_ptr() as *const i8, ((*MemoryDescriptor).BasePage) as usize, ((*MemoryDescriptor).PageCount) as usize, ((*MemoryDescriptor).MemoryType) as usize);
        
        NextMd = (*MemoryDescriptor).ListEntry.Flink;
    }
}

pub unsafe fn WinLdrpDumpBootDriver(LoaderBlock: PLOADER_PARAMETER_BLOCK) {

    let mut NextBd: PLIST_ENTRY = (*LoaderBlock).BootDriverListHead.Flink;

    while NextBd != core::ptr::addr_of_mut!((*LoaderBlock).BootDriverListHead) {
        let BootDriver: PBOOT_DRIVER_LIST_ENTRY = (NextBd as *mut u8)
            .offset(-((core::mem::offset_of!(BOOT_DRIVER_LIST_ENTRY, Link)) as isize))
            as PBOOT_DRIVER_LIST_ENTRY;

        TRACE(
            b"BootDriver %wZ DTE %08X RegPath: %wZ\n\0".as_ptr() as *const i8 , (core::ptr::addr_of_mut!((*BootDriver).FilePath)) as usize, ((*BootDriver).LdrEntry) as usize, (core::ptr::addr_of_mut!((*BootDriver).RegistryPath)) as usize,
        );

        NextBd = (*BootDriver).Link.Flink;
    }
}

unsafe fn WinLdrpDumpArcDisks(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let mut NextBd: PLIST_ENTRY;
    let mut ArcDisk: PARC_DISK_SIGNATURE;

    NextBd = (*(*LoaderBlock).ArcDiskInformation).DiskSignatureListHead.Flink;

    while NextBd != core::ptr::addr_of_mut!((*(*LoaderBlock).ArcDiskInformation).DiskSignatureListHead) {
        ArcDisk = (NextBd as *mut u8).offset(-(core::mem::offset_of!(ARC_DISK_SIGNATURE, ListEntry) as isize)) as PARC_DISK_SIGNATURE;

        TRACE(
            b"ArcDisk %s checksum: 0x%X, signature: 0x%X\n\0".as_ptr() as *const i8 , ((*ArcDisk).ArcName) as usize, ((*ArcDisk).CheckSum) as usize, ((*ArcDisk).Signature) as usize);

        NextBd = (*ArcDisk).ListEntry.Flink;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Windows-compatible NT OS Loader.
//  * COPYRIGHT:   Copyright 2006-2019 Aleksey Bragin <aleksey@reactos.org>
//  */
//
// #include <freeldr.h>
// #include <ndk/ldrtypes.h>
// #include "winldr.h"
// #include "ntldropts.h"
// #include "registry.h"
// #include <internal/cmboot.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WINDOWS);
//
// ULONG ArcGetDiskCount(VOID);
// PARC_DISK_SIGNATURE_EX ArcGetDiskInfo(ULONG Index);
//
// BOOLEAN IsAcpiPresent(VOID);
//
// extern HEADLESS_LOADER_BLOCK LoaderRedirectionInformation;
// extern BOOLEAN WinLdrTerminalConnected;
// extern VOID WinLdrSetupEms(IN PCSTR BootOptions);
//
// PLOADER_SYSTEM_BLOCK WinLdrSystemBlock;
// /**/PCWSTR BootFileSystem = NULL;/**/
//
// BOOLEAN VirtualBias = FALSE;
// BOOLEAN SosEnabled = FALSE;
// BOOLEAN SafeBoot = FALSE;
// BOOLEAN BootLogo = FALSE;
// #ifdef _M_IX86
// BOOLEAN PaeModeOn = FALSE;
// #endif
// BOOLEAN NoExecuteEnabled = FALSE;
//
// // debug stuff
// VOID DumpMemoryAllocMap(VOID);
//
// /* PE loader import-DLL loading callback */
// static VOID
// NTAPI
// NtLdrImportDllLoadCallback(
//     _In_ PCSTR FileName)
// {
//     NtLdrOutputLoadMsg(FileName, NULL);
// }
//
// VOID
// NtLdrOutputLoadMsg(
//     _In_ PCSTR FileName,
//     _In_opt_ PCSTR Description)
// {
//     if (SosEnabled)
//     {
//         printf("  %s\n", FileName);
//         TRACE("Loading: %s\n", (FileName) as usize);
//     }
//     else
//     {
//         /* Inform the user we load a file */
//         CHAR ProgressString[256];
//
//         RtlStringCbPrintfA(ProgressString, sizeof(ProgressString),
//                            "Loading %s...",
//                            (Description ? Description : FileName));
//         // UiSetProgressBarText(ProgressString);
//         // UiIndicateProgress();
//         UiDrawStatusText(ProgressString);
//     }
// }
//
// // Init "phase 0"
// VOID
// AllocateAndInitLPB(
//     IN USHORT VersionToBoot,
//     OUT PLOADER_PARAMETER_BLOCK* OutLoaderBlock)
// {
//     PLOADER_PARAMETER_BLOCK LoaderBlock;
//     PLOADER_PARAMETER_EXTENSION Extension;
//
//     /* Allocate and zero-init the Loader Parameter Block */
//     WinLdrSystemBlock = MmAllocateMemoryWithType(sizeof(LOADER_SYSTEM_BLOCK),
//                                                  LoaderSystemBlock);
//     if (WinLdrSystemBlock == NULL)
//     {
//         UiMessageBox("Failed to allocate memory for system block!");
//         return;
//     }
//
//     RtlZeroMemory(WinLdrSystemBlock, sizeof(LOADER_SYSTEM_BLOCK));
//
//     LoaderBlock = &WinLdrSystemBlock->LoaderBlock;
//     LoaderBlock->NlsData = &WinLdrSystemBlock->NlsDataBlock;
//
//     /* Initialize the Loader Block Extension */
//     Extension = &WinLdrSystemBlock->Extension;
//     LoaderBlock->Extension = Extension;
//     Extension->Size = sizeof(LOADER_PARAMETER_EXTENSION);
//     Extension->MajorVersion = (VersionToBoot & 0xFF00) >> 8;
//     Extension->MinorVersion = (VersionToBoot & 0xFF);
//
//     /* Init three critical lists, used right away */
//     InitializeListHead(&LoaderBlock->LoadOrderListHead);
//     InitializeListHead(&LoaderBlock->MemoryDescriptorListHead);
//     InitializeListHead(&LoaderBlock->BootDriverListHead);
//
//     *OutLoaderBlock = LoaderBlock;
// }
//
// // Init "phase 1"
// VOID
// WinLdrInitializePhase1(PLOADER_PARAMETER_BLOCK LoaderBlock,
//                        PCSTR Options,
//                        PCSTR SystemRoot,
//                        PCSTR BootPath,
//                        USHORT VersionToBoot)
// {
//     /*
//      * Examples of correct options and paths:
//      * CHAR Options[] = "/DEBUGPORT=COM1 /BAUDRATE=115200";
//      * CHAR Options[] = "/NODEBUG";
//      * CHAR SystemRoot[] = "\\WINNT\\";
//      * CHAR ArcBoot[] = "multi(0)disk(0)rdisk(0)partition(1)";
//      */
//
//     PSTR  LoadOptions, NewLoadOptions;
//     CHAR  HalPath[] = "\\";
//     CHAR  ArcBoot[MAX_PATH+1];
//     CHAR  MiscFiles[MAX_PATH+1];
//     ULONG i;
//     ULONG_PTR PathSeparator;
//     PLOADER_PARAMETER_EXTENSION Extension;
//
//     /* Construct SystemRoot and ArcBoot from SystemPath */
//     PathSeparator = strstr(BootPath, "\\") - BootPath;
//     RtlStringCbCopyNA(ArcBoot, sizeof(ArcBoot), BootPath, PathSeparator);
//
//     TRACE("ArcBoot: '%s'\n", (ArcBoot) as usize);
//     TRACE("SystemRoot: '%s'\n", (SystemRoot) as usize);
//     TRACE("Options: '%s'\n", (Options) as usize);
//
//     /* Fill ARC BootDevice */
//     LoaderBlock->ArcBootDeviceName = WinLdrSystemBlock->ArcBootDeviceName;
//     RtlStringCbCopyA(LoaderBlock->ArcBootDeviceName, sizeof(WinLdrSystemBlock->ArcBootDeviceName), ArcBoot);
//     LoaderBlock->ArcBootDeviceName = PaToVa(LoaderBlock->ArcBootDeviceName);
//
// //
// // IMPROVE!!
// // SetupBlock->ArcSetupDeviceName must be the path to the setup **SOURCE**,
// // and not the setup boot path. Indeed they may differ!!
// //
//     if (LoaderBlock->SetupLdrBlock)
//     {
//         PSETUP_LOADER_BLOCK SetupBlock = LoaderBlock->SetupLdrBlock;
//
//         /* Adjust the ARC path in the setup block - Matches ArcBoot path */
//         SetupBlock->ArcSetupDeviceName = WinLdrSystemBlock->ArcBootDeviceName;
//         SetupBlock->ArcSetupDeviceName = PaToVa(SetupBlock->ArcSetupDeviceName);
//
//         /* Convert the setup block pointer */
//         LoaderBlock->SetupLdrBlock = PaToVa(LoaderBlock->SetupLdrBlock);
//     }
//
//     /* Fill ARC HalDevice, it matches ArcBoot path */
//     LoaderBlock->ArcHalDeviceName = WinLdrSystemBlock->ArcBootDeviceName;
//     LoaderBlock->ArcHalDeviceName = PaToVa(LoaderBlock->ArcHalDeviceName);
//
//     /* Fill SystemRoot */
//     LoaderBlock->NtBootPathName = WinLdrSystemBlock->NtBootPathName;
//     RtlStringCbCopyA(LoaderBlock->NtBootPathName, sizeof(WinLdrSystemBlock->NtBootPathName), SystemRoot);
//     LoaderBlock->NtBootPathName = PaToVa(LoaderBlock->NtBootPathName);
//
//     /* Fill NtHalPathName */
//     LoaderBlock->NtHalPathName = WinLdrSystemBlock->NtHalPathName;
//     RtlStringCbCopyA(LoaderBlock->NtHalPathName, sizeof(WinLdrSystemBlock->NtHalPathName), HalPath);
//     LoaderBlock->NtHalPathName = PaToVa(LoaderBlock->NtHalPathName);
//
//     /* Fill LoadOptions and strip the '/' switch symbol in front of each option */
//     NewLoadOptions = LoadOptions = LoaderBlock->LoadOptions = WinLdrSystemBlock->LoadOptions;
//     RtlStringCbCopyA(LoaderBlock->LoadOptions, sizeof(WinLdrSystemBlock->LoadOptions), Options);
//
//     do
//     {
//         while (*LoadOptions == '/')
//             ++LoadOptions;
//
//         *NewLoadOptions++ = *LoadOptions;
//     } while (*LoadOptions++);
//
//     LoaderBlock->LoadOptions = PaToVa(LoaderBlock->LoadOptions);
//
//     /* ARC devices */
//     LoaderBlock->ArcDiskInformation = &WinLdrSystemBlock->ArcDiskInformation;
//     InitializeListHead(&LoaderBlock->ArcDiskInformation->DiskSignatureListHead);
//
//     /* Convert ARC disk information from freeldr to a correct format */
//     ULONG DiscCount = ArcGetDiskCount();
//     for (i = 0; i < DiscCount; i++)
//     {
//         PARC_DISK_SIGNATURE_EX ArcDiskSig;
//
//         /* Allocate the ARC structure */
//         ArcDiskSig = FrLdrHeapAlloc(sizeof(ARC_DISK_SIGNATURE_EX), 'giSD');
//         if (!ArcDiskSig)
//         {
//             ERR("Failed to allocate ARC structure! Ignoring remaining ARC disks. (i = %lu, DiskCount = %lu)\n", (//                 i) as usize, (DiscCount) as usize);
//             break;
//         }
//
//         /* Copy the data over */
//         RtlCopyMemory(ArcDiskSig, ArcGetDiskInfo(i), sizeof(ARC_DISK_SIGNATURE_EX));
//
//         /* Set the ARC Name pointer */
//         ArcDiskSig->DiskSignature.ArcName = PaToVa(ArcDiskSig->ArcName);
//
//         /* Insert into the list */
//         InsertTailList(&LoaderBlock->ArcDiskInformation->DiskSignatureListHead,
//                        &ArcDiskSig->DiskSignature.ListEntry);
//     }
//
//     /* Convert all lists to Virtual address */
//
//     /* Convert the ArcDisks list to virtual address */
//     List_PaToVa(&LoaderBlock->ArcDiskInformation->DiskSignatureListHead);
//     LoaderBlock->ArcDiskInformation = PaToVa(LoaderBlock->ArcDiskInformation);
//
//     /* Convert configuration entries to VA */
//     ConvertConfigToVA(LoaderBlock->ConfigurationRoot);
//     LoaderBlock->ConfigurationRoot = PaToVa(LoaderBlock->ConfigurationRoot);
//
//     /* Convert all DTE into virtual addresses */
//     List_PaToVa(&LoaderBlock->LoadOrderListHead);
//
//     /* This one will be converted right before switching to virtual paging mode */
//     //List_PaToVa(&LoaderBlock->MemoryDescriptorListHead);
//
//     /* Convert list of boot drivers */
//     List_PaToVa(&LoaderBlock->BootDriverListHead);
//
//     Extension = LoaderBlock->Extension;
//
//     /* FIXME! HACK value for docking profile */
//     Extension->Profile.Status = 2;
//
//     /* Check if FreeLdr detected a ACPI table */
//     if (IsAcpiPresent())
//     {
//         /* Set the pointer to something for compatibility */
//         Extension->AcpiTable = (PVOID)1;
//         // FIXME: Extension->AcpiTableSize;
//     }
//
//     if (VersionToBoot >= _WIN32_WINNT_VISTA)
//     {
//         Extension->BootViaWinload = 1;
//         Extension->LoaderPerformanceData = PaToVa(&WinLdrSystemBlock->LoaderPerformanceData);
//
//         InitializeListHead(&Extension->BootApplicationPersistentData);
//         List_PaToVa(&Extension->BootApplicationPersistentData);
//     }
//
// #ifdef _M_IX86
//     /* Set headless block pointer */
//     if (WinLdrTerminalConnected)
//     {
//         Extension->HeadlessLoaderBlock = &WinLdrSystemBlock->HeadlessLoaderBlock;
//         RtlCopyMemory(Extension->HeadlessLoaderBlock,
//                       &LoaderRedirectionInformation,
//                       sizeof(HEADLESS_LOADER_BLOCK));
//         Extension->HeadlessLoaderBlock = PaToVa(Extension->HeadlessLoaderBlock);
//     }
// #endif
//     /* Load drivers database */
//     RtlStringCbCopyA(MiscFiles, sizeof(MiscFiles), BootPath);
//     RtlStringCbCatA(MiscFiles, sizeof(MiscFiles), "AppPatch\\drvmain.sdb");
//     Extension->DrvDBImage = PaToVa(WinLdrLoadModule(MiscFiles,
//                                                     &Extension->DrvDBSize,
//                                                     LoaderRegistryData));
//
//     /* Convert the extension block pointer */
//     LoaderBlock->Extension = PaToVa(LoaderBlock->Extension);
//
//     TRACE("WinLdrInitializePhase1() completed\n");
// }
//
// static BOOLEAN
// WinLdrLoadDeviceDriver(PLIST_ENTRY LoadOrderListHead,
//                        PCSTR BootPath,
//                        PUNICODE_STRING FilePath,
//                        ULONG Flags,
//                        PLDR_DATA_TABLE_ENTRY *DriverDTE)
// {
//     CHAR FullPath[1024];
//     CHAR DriverPath[1024];
//     CHAR DllName[1024];
//     PCHAR DriverNamePos;
//     BOOLEAN Success;
//     PVOID DriverBase = NULL;
//
//     // Separate the path to file name and directory path
//     RtlStringCbPrintfA(DriverPath, sizeof(DriverPath), "%wZ", FilePath);
//     DriverNamePos = strrchr(DriverPath, '\\');
//     if (DriverNamePos != NULL)
//     {
//         // Copy the name
//         RtlStringCbCopyA(DllName, sizeof(DllName), DriverNamePos+1);
//
//         // Cut out the name from the path
//         *(DriverNamePos+1) = ANSI_NULL;
//     }
//     else
//     {
//         // There is no directory in the path
//         RtlStringCbCopyA(DllName, sizeof(DllName), DriverPath);
//         *DriverPath = ANSI_NULL;
//     }
//
//     TRACE("DriverPath: '%s', DllName: '%s', LPB\n", (DriverPath) as usize, (DllName) as usize);
//
//     // Check if driver is already loaded
//     Success = PeLdrCheckForLoadedDll(LoadOrderListHead, DllName, DriverDTE);
//     if (Success)
//     {
//         // We've got the pointer to its DTE, just return success
//         return TRUE;
//     }
//
//     // It's not loaded, we have to load it
//     RtlStringCbPrintfA(FullPath, sizeof(FullPath), "%s%wZ", BootPath, FilePath);
//
//     NtLdrOutputLoadMsg(FullPath, NULL);
//     Success = PeLdrLoadImage(FullPath, LoaderBootDriver, &DriverBase);
//     if (!Success)
//     {
//         ERR("PeLdrLoadImage('%s') failed\n", (DllName) as usize);
//         return FALSE;
//     }
//
//     // Allocate a DTE for it
//     Success = PeLdrAllocateDataTableEntry(LoadOrderListHead,
//                                           DllName,
//                                           DllName,
//                                           PaToVa(DriverBase),
//                                           DriverDTE);
//     if (!Success)
//     {
//         /* Cleanup and bail out */
//         ERR("PeLdrAllocateDataTableEntry('%s') failed\n", (DllName) as usize);
//         MmFreeMemory(DriverBase);
//         return FALSE;
//     }
//
//     /* Init security cookie */
//     PeLdrInitSecurityCookie(*DriverDTE);
//
//     // Modify any flags, if needed
//     (*DriverDTE)->Flags |= Flags;
//
//     // Look for any dependencies it may have, and load them too
//     RtlStringCbPrintfA(FullPath, sizeof(FullPath), "%s%s", BootPath, DriverPath);
//     Success = PeLdrScanImportDescriptorTable(LoadOrderListHead, FullPath, *DriverDTE);
//     if (!Success)
//     {
//         /* Cleanup and bail out */
//         ERR("PeLdrScanImportDescriptorTable('%s') failed\n", (FullPath) as usize);
//         PeLdrFreeDataTableEntry(*DriverDTE);
//         MmFreeMemory(DriverBase);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// BOOLEAN
// WinLdrLoadBootDrivers(PLOADER_PARAMETER_BLOCK LoaderBlock,
//                       PCSTR BootPath)
// {
//     PLIST_ENTRY NextBd;
//     PBOOT_DRIVER_NODE DriverNode;
//     PBOOT_DRIVER_LIST_ENTRY BootDriver;
//     BOOLEAN Success;
//     BOOLEAN ret = TRUE;
//
//     /* Walk through the boot drivers list */
//     NextBd = LoaderBlock->BootDriverListHead.Flink;
//     while (NextBd != &LoaderBlock->BootDriverListHead)
//     {
//         DriverNode = CONTAINING_RECORD(NextBd,
//                                        BOOT_DRIVER_NODE,
//                                        ListEntry.Link);
//         BootDriver = &DriverNode->ListEntry;
//
//         /* Get the next list entry as we may remove the current one on failure */
//         NextBd = BootDriver->Link.Flink;
//
//         // Paths are relative (FIXME: Are they always relative?)
//
//         /* Load it */
//         UiIndicateProgress();
//         Success = WinLdrLoadDeviceDriver(&LoaderBlock->LoadOrderListHead,
//                                          BootPath,
//                                          &BootDriver->FilePath,
//                                          0,
//                                          &BootDriver->LdrEntry);
//         if (Success)
//         {
//             /* Convert the addresses to VA since we are not going to use them anymore */
//             BootDriver->RegistryPath.Buffer = PaToVa(BootDriver->RegistryPath.Buffer);
//             BootDriver->FilePath.Buffer = PaToVa(BootDriver->FilePath.Buffer);
//             BootDriver->LdrEntry = PaToVa(BootDriver->LdrEntry);
//
//             if (DriverNode->Group.Buffer)
//                 DriverNode->Group.Buffer = PaToVa(DriverNode->Group.Buffer);
//             DriverNode->Name.Buffer = PaToVa(DriverNode->Name.Buffer);
//         }
//         else
//         {
//             /* Loading failed: cry loudly */
//             ERR("Cannot load boot driver '%wZ'!\n", (&BootDriver->FilePath) as usize);
// #ifndef UEFIBOOT
//             UiMessageBox("Cannot load boot driver '%wZ'!", &BootDriver->FilePath);
// #endif
//             ret = FALSE;
//
//             /* Remove it from the list and try to continue */
//             RemoveEntryList(&BootDriver->Link);
//         }
//     }
//
//     return ret;
// }
//
// PVOID
// WinLdrLoadModule(PCSTR ModuleName,
//                  PULONG Size,
//                  TYPE_OF_MEMORY MemoryType)
// {
//     ULONG FileId;
//     PVOID PhysicalBase;
//     FILEINFORMATION FileInfo;
//     ULONG FileSize;
//     ARC_STATUS Status;
//     ULONG BytesRead;
//
//     *Size = 0;
//
//     /* Open the image file */
//     NtLdrOutputLoadMsg(ModuleName, NULL);
//     Status = ArcOpen((PSTR)ModuleName, OpenReadOnly, &FileId);
//     if (Status != ESUCCESS)
//     {
//         /* In case of errors, we just return, without complaining to the user */
//         WARN("Error while opening '%s', Status: %u\n", (ModuleName) as usize, (Status) as usize);
//         return NULL;
//     }
//
//     /* Retrieve its size */
//     Status = ArcGetFileInformation(FileId, &FileInfo);
//     if (Status != ESUCCESS)
//     {
//         ArcClose(FileId);
//         return NULL;
//     }
//     FileSize = FileInfo.EndingAddress.LowPart;
//     *Size = FileSize;
//
//     /* Allocate memory */
//     PhysicalBase = MmAllocateMemoryWithType(FileSize, MemoryType);
//     if (PhysicalBase == NULL)
//     {
//         ERR("Could not allocate memory for '%s'\n", (ModuleName) as usize);
//         ArcClose(FileId);
//         return NULL;
//     }
//
//     /* Load the whole file */
//     Status = ArcRead(FileId, PhysicalBase, FileSize, &BytesRead);
//     ArcClose(FileId);
//     if (Status != ESUCCESS)
//     {
//         WARN("Error while reading '%s', Status: %u\n", (ModuleName) as usize, (Status) as usize);
//         return NULL;
//     }
//
//     TRACE("Loaded %s at 0x%x with size 0x%x\n", (ModuleName) as usize, (PhysicalBase) as usize, (FileSize) as usize);
//
//     return PhysicalBase;
// }
//
// USHORT
// WinLdrDetectVersion(VOID)
// {
//     LONG rc;
//     HKEY hKey;
//
//     rc = RegOpenKey(CurrentControlSetKey, L"Control\\Terminal Server", &hKey);
//     if (rc != ERROR_SUCCESS)
//     {
//         /* Key doesn't exist; assume NT 4.0 */
//         return _WIN32_WINNT_NT4;
//     }
//     RegCloseKey(hKey);
//
//     /* We may here want to read the value of ProductVersion */
//     return _WIN32_WINNT_WS03;
// }
//
// static
// PVOID
// LoadModule(
//     IN OUT PLOADER_PARAMETER_BLOCK LoaderBlock,
//     IN PCCH Path,
//     IN PCCH File,
//     IN PCCH ImportName, // BaseDllName
//     IN TYPE_OF_MEMORY MemoryType,
//     OUT PLDR_DATA_TABLE_ENTRY *Dte,
//     IN ULONG Percentage)
// {
//     BOOLEAN Success;
//     CHAR FullFileName[MAX_PATH];
//     CHAR ProgressString[256];
//     PVOID BaseAddress;
//
//     RtlStringCbPrintfA(ProgressString, sizeof(ProgressString), "Loading %s...", File);
//     UiUpdateProgressBar(Percentage, ProgressString);
//
//     RtlStringCbCopyA(FullFileName, sizeof(FullFileName), Path);
//     RtlStringCbCatA(FullFileName, sizeof(FullFileName), File);
//
//     NtLdrOutputLoadMsg(FullFileName, NULL);
//     Success = PeLdrLoadImage(FullFileName, MemoryType, &BaseAddress);
//     if (!Success)
//     {
//         ERR("PeLdrLoadImage('%s') failed\n", (File) as usize);
//         return NULL;
//     }
//     TRACE("%s loaded successfully at %p\n", (File) as usize, (BaseAddress) as usize);
//
//     Success = PeLdrAllocateDataTableEntry(&LoaderBlock->LoadOrderListHead,
//                                           ImportName,
//                                           FullFileName,
//                                           PaToVa(BaseAddress),
//                                           Dte);
//     if (!Success)
//     {
//         /* Cleanup and bail out */
//         ERR("PeLdrAllocateDataTableEntry('%s') failed\n", (FullFileName) as usize);
//         MmFreeMemory(BaseAddress);
//         return NULL;
//     }
//
//     /* Init security cookie */
//     PeLdrInitSecurityCookie(*Dte);
//
//     return BaseAddress;
// }
//
// #ifdef _M_IX86
// static
// BOOLEAN
// WinLdrIsPaeSupported(
//     _In_ USHORT OperatingSystemVersion,
//     _In_ PLOADER_PARAMETER_BLOCK LoaderBlock,
//     _In_ PCSTR BootOptions,
//     _In_ PCSTR HalFileName,
//     _Inout_updates_bytes_(KernelFileNameSize) _Always_(_Post_z_)
//          PSTR KernelFileName,
//     _In_ SIZE_T KernelFileNameSize)
// {
//     BOOLEAN PaeEnabled = FALSE;
//     BOOLEAN PaeDisabled = FALSE;
//     BOOLEAN Result;
//
//     if ((OperatingSystemVersion > _WIN32_WINNT_NT4) &&
//         NtLdrGetOption(BootOptions, "PAE"))
//     {
//         /* We found the PAE option */
//         PaeEnabled = TRUE;
//     }
//
//     Result = PaeEnabled;
//
//     if ((OperatingSystemVersion > _WIN32_WINNT_WIN2K) &&
//         NtLdrGetOption(BootOptions, "NOPAE"))
//     {
//         PaeDisabled = TRUE;
//     }
//
//     if (SafeBoot)
//         PaeDisabled = TRUE;
//
//     TRACE("PaeEnabled %X, PaeDisabled %X\n", (PaeEnabled) as usize, (PaeDisabled) as usize);
//
//     if (PaeDisabled)
//         Result = FALSE;
//
//     /* Enable PAE if DEP is enabled */
//     if (NoExecuteEnabled)
//         Result = TRUE;
//
//     // TODO: checks for CPU support, hotplug memory support ... other tests
//     // TODO: select kernel name ("ntkrnlpa.exe" or "ntoskrnl.exe"), or,
//     // if KernelFileName is a user-specified kernel file, check whether it
//     // has, if PAE needs to be enabled, the IMAGE_FILE_LARGE_ADDRESS_AWARE
//     // Characteristics bit set, and that the HAL image has a similar support.
//
//     if (Result) UNIMPLEMENTED;
//
//     return Result;
// }
// #endif /* _M_IX86 */
//
// static
// BOOLEAN
// LoadWindowsCore(IN USHORT OperatingSystemVersion,
//                 IN OUT PLOADER_PARAMETER_BLOCK LoaderBlock,
//                 IN PCSTR BootOptions,
//                 IN PCSTR BootPath,
//                 IN OUT PLDR_DATA_TABLE_ENTRY* KernelDTE)
// {
//     BOOLEAN Success;
//     PCSTR Option;
//     ULONG OptionLength;
//     PVOID KernelBase, HalBase, KdDllBase = NULL;
//     PLDR_DATA_TABLE_ENTRY HalDTE, KdDllDTE = NULL;
//     CHAR DirPath[MAX_PATH];
//     CHAR HalFileName[MAX_PATH];
//     CHAR KernelFileName[MAX_PATH];
//     CHAR KdDllName[MAX_PATH];
//
//     if (!KernelDTE) return FALSE;
//
//     /* Initialize SystemRoot\System32 path */
//     RtlStringCbCopyA(DirPath, sizeof(DirPath), BootPath);
//     RtlStringCbCatA(DirPath, sizeof(DirPath), "system32\\");
//
//     /* Parse the boot options */
//     TRACE("LoadWindowsCore: BootOptions '%s'\n", (BootOptions) as usize);
//
// #ifdef _M_IX86
//     if (NtLdrGetOption(BootOptions, "3GB"))
//     {
//         /* We found the 3GB option. */
//         FIXME("LoadWindowsCore: 3GB - TRUE (not implemented)\n");
//         VirtualBias = TRUE;
//     }
//     // TODO: "USERVA=" for XP/2k3
// #endif
//
//     if ((OperatingSystemVersion > _WIN32_WINNT_NT4) &&
//         (NtLdrGetOption(BootOptions, "SAFEBOOT") ||
//          NtLdrGetOption(BootOptions, "SAFEBOOT:")))
//     {
//         /* We found the SAFEBOOT option. */
//         FIXME("LoadWindowsCore: SAFEBOOT - TRUE (not implemented)\n");
//         SafeBoot = TRUE;
//     }
//
//     if ((OperatingSystemVersion > _WIN32_WINNT_WIN2K) &&
//         NtLdrGetOption(BootOptions, "BOOTLOGO"))
//     {
//         /* We found the BOOTLOGO option. */
//         FIXME("LoadWindowsCore: BOOTLOGO - TRUE (not implemented)\n");
//         BootLogo = TRUE;
//     }
//
//     /* Check the (NO)EXECUTE options */
//     if ((OperatingSystemVersion > _WIN32_WINNT_WIN2K) &&
//         !LoaderBlock->SetupLdrBlock)
//     {
//         /* Disable NX by default on x86, otherwise enable it */
// #ifdef _M_IX86
//         NoExecuteEnabled = FALSE;
// #else
//         NoExecuteEnabled = TRUE;
// #endif
//
// #ifdef _M_IX86
//         /* Check the options in decreasing order of precedence */
//         if (NtLdrGetOption(BootOptions, "NOEXECUTE=OPTIN")  ||
//             NtLdrGetOption(BootOptions, "NOEXECUTE=OPTOUT") ||
//             NtLdrGetOption(BootOptions, "NOEXECUTE=ALWAYSON"))
//         {
//             NoExecuteEnabled = TRUE;
//         }
//         else if (NtLdrGetOption(BootOptions, "NOEXECUTE=ALWAYSOFF"))
//             NoExecuteEnabled = FALSE;
//         else
// #else
//         /* Only the following two options really apply for x64 and other platforms */
// #endif
//         if (NtLdrGetOption(BootOptions, "NOEXECUTE"))
//             NoExecuteEnabled = TRUE;
//         else if (NtLdrGetOption(BootOptions, "EXECUTE"))
//             NoExecuteEnabled = FALSE;
//
// #ifdef _M_IX86
//         /* Disable DEP in SafeBoot mode for x86 only */
//         if (SafeBoot)
//             NoExecuteEnabled = FALSE;
// #endif
//     }
//     TRACE("NoExecuteEnabled %X\n", (NoExecuteEnabled) as usize);
//
//     /*
//      * Select the HAL and KERNEL file names.
//      * Check for any "/HAL=" or "/KERNEL=" override option.
//      *
//      * See the following links to know how the file names are actually chosen:
//      * https://www.geoffchappell.com/notes/windows/boot/bcd/osloader/detecthal.htm
//      * https://www.geoffchappell.com/notes/windows/boot/bcd/osloader/hal.htm
//      * https://www.geoffchappell.com/notes/windows/boot/bcd/osloader/kernel.htm
//      */
//     /* Default HAL and KERNEL file names */
//     RtlStringCbCopyA(HalFileName   , sizeof(HalFileName)   , "hal.dll");
//     RtlStringCbCopyA(KernelFileName, sizeof(KernelFileName), "ntoskrnl.exe");
//
//     Option = NtLdrGetOptionEx(BootOptions, "HAL=", &OptionLength);
//     if (Option && (OptionLength > 4))
//     {
//         /* Retrieve the HAL file name */
//         Option += 4; OptionLength -= 4;
//         RtlStringCbCopyNA(HalFileName, sizeof(HalFileName), Option, OptionLength);
//         _strlwr(HalFileName);
//     }
//
//     Option = NtLdrGetOptionEx(BootOptions, "KERNEL=", &OptionLength);
//     if (Option && (OptionLength > 7))
//     {
//         /* Retrieve the KERNEL file name */
//         Option += 7; OptionLength -= 7;
//         RtlStringCbCopyNA(KernelFileName, sizeof(KernelFileName), Option, OptionLength);
//         _strlwr(KernelFileName);
//     }
//
// #ifdef _M_IX86
//     /* Check for PAE support and select the adequate kernel image */
//     PaeModeOn = WinLdrIsPaeSupported(OperatingSystemVersion,
//                                      LoaderBlock,
//                                      BootOptions,
//                                      HalFileName,
//                                      KernelFileName,
//                                      sizeof(KernelFileName));
//     if (PaeModeOn) FIXME("WinLdrIsPaeSupported: PaeModeOn\n");
// #endif
//
//     TRACE("HAL file = '%s' ; Kernel file = '%s'\n", (HalFileName) as usize, (KernelFileName) as usize);
//
//     /*
//      * Load the core NT files: Kernel, HAL and KD transport DLL.
//      * Cheat about their base DLL name so as to satisfy the imports/exports,
//      * even if the corresponding underlying files do not have the same names
//      * -- this happens e.g. with UP vs. MP kernel, standard vs. ACPI hal, or
//      * different KD transport DLLs.
//      */
//
//     /* Load the Kernel */
//     KernelBase = LoadModule(LoaderBlock, DirPath, KernelFileName,
//                             "ntoskrnl.exe", LoaderSystemCode, KernelDTE, 30);
//     if (!KernelBase)
//     {
//         ERR("LoadModule('%s') failed\n", (KernelFileName) as usize);
//         UiMessageBox("Could not load %s", KernelFileName);
//         return FALSE;
//     }
//
//     /* Load the HAL */
//     HalBase = LoadModule(LoaderBlock, DirPath, HalFileName,
//                          "hal.dll", LoaderHalCode, &HalDTE, 35);
//     if (!HalBase)
//     {
//         ERR("LoadModule('%s') failed\n", (HalFileName) as usize);
//         UiMessageBox("Could not load %s", HalFileName);
//         PeLdrFreeDataTableEntry(*KernelDTE);
//         MmFreeMemory(KernelBase);
//         return FALSE;
//     }
//
//     /* Load the Kernel Debugger Transport DLL */
//     if (OperatingSystemVersion > _WIN32_WINNT_WIN2K)
//     {
//         /*
//          * According to http://www.nynaeve.net/?p=173 :
//          * "[...] Another enhancement that could be done Microsoft-side would be
//          * a better interface for replacing KD transport modules. Right now, due
//          * to the fact that ntoskrnl is static linked to KDCOM.DLL, the OS loader
//          * has a hardcoded hack that interprets the KD type in the OS loader options,
//          * loads one of the (hardcoded filenames) "kdcom.dll", "kd1394.dll", or
//          * "kdusb2.dll" modules, and inserts them into the loaded module list under
//          * the name "kdcom.dll". [...]"
//          */
//
//         /*
//          * A Kernel Debugger Transport DLL is always loaded for Windows XP+ :
//          * either the standard KDCOM.DLL (by default): IsCustomKdDll == FALSE
//          * or an alternative user-provided one via the /DEBUGPORT= option:
//          * IsCustomKdDll == TRUE if it does not specify the default KDCOM.
//          */
//         BOOLEAN IsCustomKdDll = FALSE;
//
//         /* Check whether there is a DEBUGPORT option */
//         Option = NtLdrGetOptionEx(BootOptions, "DEBUGPORT=", &OptionLength);
//         if (Option && (OptionLength > 10))
//         {
//             /* Move to the debug port name */
//             Option += 10; OptionLength -= 10;
//
//             /*
//              * Parse the port name.
//              * Format: /DEBUGPORT=COM[0-9]
//              * or: /DEBUGPORT=FILE:\Device\HarddiskX\PartitionY\debug.log
//              * or: /DEBUGPORT=FOO
//              * If we only have /DEBUGPORT= (i.e. without any port name),
//              * default to "COM".
//              */
//
//             /* Get the actual length of the debug port
//              * until the next whitespace or colon. */
//             OptionLength = (ULONG)strcspn(Option, " \t:");
//
//             if ((OptionLength == 0) ||
//                 ( (OptionLength >= 3) && (_strnicmp(Option, "COM", 3) == 0) &&
//                  ((OptionLength == 3) || ('0' <= Option[3] && Option[3] <= '9')) ))
//             {
//                 /* The standard KDCOM.DLL is used */
//             }
//             else
//             {
//                 /* A custom KD DLL is used */
//                 IsCustomKdDll = TRUE;
//             }
//         }
//         if (!IsCustomKdDll)
//         {
//             Option = "COM"; OptionLength = 3;
//         }
//
//         RtlStringCbPrintfA(KdDllName, sizeof(KdDllName), "kd%.*s.dll",
//                            OptionLength, Option);
//         _strlwr(KdDllName);
//
//         /* Load the KD DLL. Override its base DLL name to the default "KDCOM.DLL". */
//         KdDllBase = LoadModule(LoaderBlock, DirPath, KdDllName,
//                                "kdcom.dll", LoaderSystemCode, &KdDllDTE, 40);
//         if (!KdDllBase)
//         {
//             /* If we failed to load a custom KD DLL, fall back to the standard one */
//             if (IsCustomKdDll)
//             {
//                 /* The custom KD DLL being optional, just ignore the failure */
//                 WARN("LoadModule('%s') failed\n", (KdDllName) as usize);
//
//                 IsCustomKdDll = FALSE;
//                 RtlStringCbCopyA(KdDllName, sizeof(KdDllName), "kdcom.dll");
//
//                 KdDllBase = LoadModule(LoaderBlock, DirPath, KdDllName,
//                                        "kdcom.dll", LoaderSystemCode, &KdDllDTE, 40);
//             }
//
//             if (!KdDllBase)
//             {
//                 /* Ignore the failure; we will fail later when scanning the
//                  * kernel import tables, if it really needs the KD DLL. */
//                 ERR("LoadModule('%s') failed\n", (KdDllName) as usize);
//             }
//         }
//     }
//
//     /* Load all referenced DLLs for Kernel, HAL and Kernel Debugger Transport DLL */
//     Success = PeLdrScanImportDescriptorTable(&LoaderBlock->LoadOrderListHead, DirPath, *KernelDTE);
//     if (!Success)
//     {
//         UiMessageBox("Could not load %s", KernelFileName);
//         goto Quit;
//     }
//     Success = PeLdrScanImportDescriptorTable(&LoaderBlock->LoadOrderListHead, DirPath, HalDTE);
//     if (!Success)
//     {
//         UiMessageBox("Could not load %s", HalFileName);
//         goto Quit;
//     }
//     if (KdDllDTE)
//     {
//         Success = PeLdrScanImportDescriptorTable(&LoaderBlock->LoadOrderListHead, DirPath, KdDllDTE);
//         if (!Success)
//         {
//             UiMessageBox("Could not load %s", KdDllName);
//             goto Quit;
//         }
//     }
//
// Quit:
//     if (!Success)
//     {
//         /* Cleanup and bail out */
//         if (KdDllDTE)
//             PeLdrFreeDataTableEntry(KdDllDTE);
//         if (KdDllBase) // Optional
//             MmFreeMemory(KdDllBase);
//
//         PeLdrFreeDataTableEntry(HalDTE);
//         MmFreeMemory(HalBase);
//
//         PeLdrFreeDataTableEntry(*KernelDTE);
//         MmFreeMemory(KernelBase);
//     }
//
//     return Success;
// }
//
// static
// BOOLEAN
// WinLdrInitErrataInf(
//     IN OUT PLOADER_PARAMETER_BLOCK LoaderBlock,
//     IN USHORT OperatingSystemVersion,
//     IN PCSTR SystemRoot)
// {
//     LONG rc;
//     HKEY hKey;
//     ULONG BufferSize;
//     ULONG FileSize;
//     PVOID PhysicalBase;
//     WCHAR szFileName[80];
//     CHAR ErrataFilePath[MAX_PATH];
//
//     /* Open either the 'BiosInfo' (Windows <= 2003) or the 'Errata' (Vista+) key */
//     if (OperatingSystemVersion >= _WIN32_WINNT_VISTA)
//     {
//         rc = RegOpenKey(CurrentControlSetKey, L"Control\\Errata", &hKey);
//     }
//     else // (OperatingSystemVersion <= _WIN32_WINNT_WS03)
//     {
//         rc = RegOpenKey(CurrentControlSetKey, L"Control\\BiosInfo", &hKey);
//     }
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("Could not open the BiosInfo/Errata registry key (Error %u)\n", ((int)rc) as usize);
//         return FALSE;
//     }
//
//     /* Retrieve the INF file name value */
//     BufferSize = sizeof(szFileName);
//     rc = RegQueryValue(hKey, L"InfName", NULL, (PUCHAR)szFileName, &BufferSize);
//     if (rc != ERROR_SUCCESS)
//     {
//         WARN("Could not retrieve the InfName value (Error %u)\n", ((int)rc) as usize);
//         RegCloseKey(hKey);
//         return FALSE;
//     }
//
//     // TODO: "SystemBiosDate"
//
//     RegCloseKey(hKey);
//
//     RtlStringCbPrintfA(ErrataFilePath, sizeof(ErrataFilePath), "%s%s%S",
//                        SystemRoot, "inf\\", szFileName);
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
// ARC_STATUS
// LoadAndBootWindows(
//     IN ULONG Argc,
//     IN PCHAR Argv[],
//     IN PCHAR Envp[])
// {
//     ARC_STATUS Status;
//     PCSTR ArgValue;
//     PCSTR SystemPartition;
//     PCSTR FileName;
//     ULONG FileNameLength;
//     BOOLEAN Success;
//     USHORT OperatingSystemVersion;
//     PLOADER_PARAMETER_BLOCK LoaderBlock;
//     CHAR BootPath[MAX_PATH];
//     CHAR FilePath[MAX_PATH];
//     CHAR BootOptions[256];
//
//     /* Retrieve the (mandatory) boot type */
//     ArgValue = GetArgumentValue(Argc, Argv, "BootType");
//     if (!ArgValue || !*ArgValue)
//     {
//         ERR("No 'BootType' value, aborting!\n");
//         return EINVAL;
//     }
//
//     /* Convert it to an OS version */
//     if (_stricmp(ArgValue, "Windows") == 0 ||
//         _stricmp(ArgValue, "Windows2003") == 0)
//     {
//         OperatingSystemVersion = _WIN32_WINNT_WS03;
//     }
//     else if (_stricmp(ArgValue, "WindowsNT40") == 0)
//     {
//         OperatingSystemVersion = _WIN32_WINNT_NT4;
//     }
//     else if (_stricmp(ArgValue, "WindowsVista") == 0)
//     {
//         OperatingSystemVersion = _WIN32_WINNT_VISTA;
//     }
//     else
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
//     UiDrawStatusText("Loading...");
//     UiDrawProgressBarCenter("Loading NT...");
//
//     /* Retrieve the system path */
//     *BootPath = ANSI_NULL;
//     ArgValue = GetArgumentValue(Argc, Argv, "SystemPath");
//     if (ArgValue)
//         RtlStringCbCopyA(BootPath, sizeof(BootPath), ArgValue);
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
//     /* Retrieve the boot options */
//     *BootOptions = ANSI_NULL;
//     ArgValue = GetArgumentValue(Argc, Argv, "Options");
//     if (ArgValue && *ArgValue)
//         RtlStringCbCopyA(BootOptions, sizeof(BootOptions), ArgValue);
//
//     /* Append boot-time options */
//     AppendBootTimeOptions(BootOptions);
//
//     /*
//      * Set the "/HAL=" and "/KERNEL=" options if needed.
//      * If already present on the standard "Options=" option line, they take
//      * precedence over those passed via the separate "Hal=" and "Kernel="
//      * options.
//      */
//     if (!NtLdrGetOption(BootOptions, "HAL="))
//     {
//         /*
//          * Not found in the options, try to retrieve the
//          * separate value and append it to the options.
//          */
//         ArgValue = GetArgumentValue(Argc, Argv, "Hal");
//         if (ArgValue && *ArgValue)
//         {
//             RtlStringCbCatA(BootOptions, sizeof(BootOptions), " /HAL=");
//             RtlStringCbCatA(BootOptions, sizeof(BootOptions), ArgValue);
//         }
//     }
//     if (!NtLdrGetOption(BootOptions, "KERNEL="))
//     {
//         /*
//          * Not found in the options, try to retrieve the
//          * separate value and append it to the options.
//          */
//         ArgValue = GetArgumentValue(Argc, Argv, "Kernel");
//         if (ArgValue && *ArgValue)
//         {
//             RtlStringCbCatA(BootOptions, sizeof(BootOptions), " /KERNEL=");
//             RtlStringCbCatA(BootOptions, sizeof(BootOptions), ArgValue);
//         }
//     }
//
//     TRACE("BootOptions: '%s'\n", (BootOptions) as usize);
//
//     /* Check if a RAM disk file was given */
//     FileName = NtLdrGetOptionEx(BootOptions, "RDPATH=", &FileNameLength);
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
//     /* Handle the SOS option */
//     SosEnabled = !!NtLdrGetOption(BootOptions, "SOS");
//     if (SosEnabled)
//         UiResetForSOS();
//
//     /* Allocate and minimally-initialize the Loader Parameter Block */
//     AllocateAndInitLPB(OperatingSystemVersion, &LoaderBlock);
//
//     /* Load the system hive */
//     UiUpdateProgressBar(15, "Loading system hive...");
//     Success = WinLdrInitSystemHive(LoaderBlock, BootPath, FALSE);
//     TRACE("SYSTEM hive %s\n", ((Success ? "loaded" : "not loaded")) as usize);
//     /* Bail out if failure */
//     if (!Success)
//         return ENOEXEC;
//
//     /* Fixup the version number using data from the registry */
//     if (OperatingSystemVersion == 0)
//         OperatingSystemVersion = WinLdrDetectVersion();
//     LoaderBlock->Extension->MajorVersion = (OperatingSystemVersion & 0xFF00) >> 8;
//     LoaderBlock->Extension->MinorVersion = (OperatingSystemVersion & 0xFF);
//
//     /* Load NLS data, OEM font, and prepare boot drivers list */
//     Success = WinLdrScanSystemHive(LoaderBlock, BootPath);
//     TRACE("SYSTEM hive %s\n", ((Success ? "scanned" : "not scanned")) as usize);
//     /* Bail out if failure */
//     if (!Success)
//         return ENOEXEC;
//
//     /* Load the Firmware Errata file */
//     Success = WinLdrInitErrataInf(LoaderBlock, OperatingSystemVersion, BootPath);
//     TRACE("Firmware Errata file %s\n", ((Success ? "loaded" : "not loaded")) as usize);
//     /* Not necessarily fatal if not found - carry on going */
//
//     /* Finish loading */
//     return LoadAndBootWindowsCommon(OperatingSystemVersion,
//                                     LoaderBlock,
//                                     BootOptions,
//                                     BootPath);
// }
//
// ARC_STATUS
// LoadAndBootWindowsCommon(
//     IN USHORT OperatingSystemVersion,
//     IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//     IN PCSTR BootOptions,
//     IN PCSTR BootPath)
// {
//     PLOADER_PARAMETER_BLOCK LoaderBlockVA;
//     BOOLEAN Success;
//     PLDR_DATA_TABLE_ENTRY KernelDTE;
//     KERNEL_ENTRY_POINT KiSystemStartup;
//     PCSTR SystemRoot;
//
//     ASSERT(OperatingSystemVersion != 0);
//
// #ifdef _M_IX86
//     /* Setup redirection support */
//     WinLdrSetupEms(BootOptions);
// #endif
//
//     /* Convert BootPath to SystemRoot */
//     SystemRoot = strstr(BootPath, "\\");
//
//     /* Detect hardware */
//     UiUpdateProgressBar(20, "Detecting hardware...");
//     LoaderBlock->ConfigurationRoot = MachHwDetect(BootOptions);
//
//     /* Initialize the PE loader import-DLL callback, so that we can obtain
//      * feedback (for example during SOS) on the PE images that get loaded. */
//     PeLdrImportDllLoadCallback = NtLdrImportDllLoadCallback;
//
//     /* Load the operating system core: the Kernel, the HAL and the Kernel Debugger Transport DLL */
//     Success = LoadWindowsCore(OperatingSystemVersion,
//                               LoaderBlock,
//                               BootOptions,
//                               BootPath,
//                               &KernelDTE);
//     if (!Success)
//     {
//         /* Reset the PE loader import-DLL callback */
//         PeLdrImportDllLoadCallback = NULL;
//
//         UiMessageBox("Error loading NTOS core.");
//         return ENOEXEC;
//     }
//
//     /* Cleanup INI file */
//     IniCleanup();
//
// /****
//  **** WE HAVE NOW REACHED THE POINT OF NO RETURN !!
//  ****/
//
//     UiSetProgressBarSubset(40, 90); // NTOS goes from 25 to 75%
//
//     /* Load boot drivers */
//     UiSetProgressBarText("Loading boot drivers...");
//     Success = WinLdrLoadBootDrivers(LoaderBlock, BootPath);
//
//     UiSetProgressBarSubset(0, 100);
//
//     /* Reset the PE loader import-DLL callback */
//     PeLdrImportDllLoadCallback = NULL;
//
//     /* Initialize Phase 1 - no drivers loading anymore */
//     WinLdrInitializePhase1(LoaderBlock,
//                            BootOptions,
//                            SystemRoot,
//                            BootPath,
//                            OperatingSystemVersion);
//
//     UiUpdateProgressBar(100, NULL);
//
//     /* Save entry-point pointer and Loader block VAs */
//     KiSystemStartup = (KERNEL_ENTRY_POINT)KernelDTE->EntryPoint;
//     LoaderBlockVA = PaToVa(LoaderBlock);
//
//     /* "Stop all motors", change videomode */
//     MachPrepareForReactOS();
//
//     /* Show the "debug mode" notice if needed */
//     /* Match KdInitSystem() conditions */
//     if (!NtLdrGetOption(BootOptions, "CRASHDEBUG") &&
//         !NtLdrGetOption(BootOptions, "NODEBUG") &&
//         !!NtLdrGetOption(BootOptions, "DEBUG"))
//     {
//         /* Check whether there is a DEBUGPORT option */
//         PCSTR DebugPort;
//         ULONG DebugPortLength = 0;
//         DebugPort = NtLdrGetOptionEx(BootOptions, "DEBUGPORT=", &DebugPortLength);
//         if (DebugPort != NULL && DebugPortLength > 10)
//         {
//             /* Move to the debug port name */
//             DebugPort += 10; DebugPortLength -= 10;
//         }
//         else
//         {
//             /* Default to COM */
//             DebugPort = "COM"; DebugPortLength = 3;
//         }
//
//         /* It is booting in debug mode, show the banner */
// #ifdef UEFIBOOT
//         /* On UEFI without video, TuiPrintf hangs (framebuffer).
//          * Use the debug channel instead. */
//         ERR("Debug mode: connect debugger on port %.*s\n", (//             DebugPortLength) as usize, (DebugPort) as usize);
// #else
//         TuiPrintf("You need to connect a debugger on port %.*s\n"
//                   "For more information, visit https://reactos.org/wiki/Debugging.\n",
//                   DebugPortLength, DebugPort);
// #endif
//     }
//
//     /* Debugging... */
//     //DumpMemoryAllocMap();
//
//     /* Do the machine specific initialization */
//     WinLdrSetupMachineDependent(LoaderBlock);
//
// #ifdef _M_ARM64
//     ERR(">>> WinLdrSetupMemoryLayout...\n");
// #endif
//     /* Map pages and create memory descriptors */
//     WinLdrSetupMemoryLayout(LoaderBlock);
//
// #ifdef _M_ARM64
//     ERR(">>> WinLdrSetProcessorContext...\n");
// #endif
//     /* Set processor context */
//     WinLdrSetProcessorContext(OperatingSystemVersion);
//
// #ifdef _M_ARM64
//     ERR(">>> Page tables active! Extension=%p KiSystemStartup=%p\n", (//         LoaderBlock->Extension) as usize, (KiSystemStartup) as usize);
//     ERR(">>> Writing LoaderPagesSpanned...\n");
// #endif
//     /* Save final value of LoaderPagesSpanned */
//     LoaderBlock->Extension->LoaderPagesSpanned = MmGetLoaderPagesSpanned();
//
// #ifdef _M_ARM64
//     ERR(">>> Zeroing KI_USER_SHARED_DATA at %p...\n", ((PVOID)KI_USER_SHARED_DATA) as usize);
// #endif
//     /* Zero KI_USER_SHARED_DATA page */
//     RtlZeroMemory((PVOID)KI_USER_SHARED_DATA, MM_PAGE_SIZE);
//
// #ifdef _M_ARM64
//     ERR(">>> Calling KiSystemStartup(%p)...\n", (LoaderBlockVA) as usize);
// #endif
//
//     WinLdrpDumpMemoryDescriptors(LoaderBlockVA);
//     WinLdrpDumpBootDriver(LoaderBlockVA);
// #ifndef _M_AMD64
//     WinLdrpDumpArcDisks(LoaderBlockVA);
// #endif
//
//     /* Pass control */
//     (*KiSystemStartup)(LoaderBlockVA);
//
//     UNREACHABLE; // return ESUCCESS;
// }
//
// VOID
// WinLdrpDumpMemoryDescriptors(PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLIST_ENTRY NextMd;
//     PMEMORY_ALLOCATION_DESCRIPTOR MemoryDescriptor;
//
//     NextMd = LoaderBlock->MemoryDescriptorListHead.Flink;
//
//     while (NextMd != &LoaderBlock->MemoryDescriptorListHead)
//     {
//         MemoryDescriptor = CONTAINING_RECORD(NextMd, MEMORY_ALLOCATION_DESCRIPTOR, ListEntry);
//
//         TRACE("BP %08X PC %04X MT %d\n", (MemoryDescriptor->BasePage) as usize, (//             MemoryDescriptor->PageCount) as usize, (MemoryDescriptor->MemoryType) as usize);
//
//         NextMd = MemoryDescriptor->ListEntry.Flink;
//     }
// }
//
// VOID
// WinLdrpDumpBootDriver(PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLIST_ENTRY NextBd;
//     PBOOT_DRIVER_LIST_ENTRY BootDriver;
//
//     NextBd = LoaderBlock->BootDriverListHead.Flink;
//
//     while (NextBd != &LoaderBlock->BootDriverListHead)
//     {
//         BootDriver = CONTAINING_RECORD(NextBd, BOOT_DRIVER_LIST_ENTRY, Link);
//
//         TRACE("BootDriver %wZ DTE %08X RegPath: %wZ\n", (&BootDriver->FilePath) as usize, (//             BootDriver->LdrEntry) as usize, (&BootDriver->RegistryPath) as usize);
//
//         NextBd = BootDriver->Link.Flink;
//     }
// }
//
// VOID
// WinLdrpDumpArcDisks(PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLIST_ENTRY NextBd;
//     PARC_DISK_SIGNATURE ArcDisk;
//
//     NextBd = LoaderBlock->ArcDiskInformation->DiskSignatureListHead.Flink;
//
//     while (NextBd != &LoaderBlock->ArcDiskInformation->DiskSignatureListHead)
//     {
//         ArcDisk = CONTAINING_RECORD(NextBd, ARC_DISK_SIGNATURE, ListEntry);
//
//         TRACE("ArcDisk %s checksum: 0x%X, signature: 0x%X\n", (//             ArcDisk->ArcName) as usize, (ArcDisk->CheckSum) as usize, (ArcDisk->Signature) as usize);
//
//         NextBd = ArcDisk->ListEntry.Flink;
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
