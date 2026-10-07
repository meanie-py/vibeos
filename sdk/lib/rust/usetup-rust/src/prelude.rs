//! usetup-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/DPRINT1/UNIMPLEMENTED — no-op MACROS, never out-of-line
// fns (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! DPRINT1 { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, DPRINT1, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { ((($s) as i32) >= 0) as i32 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ARRAYSIZE, ASSERT, C_ASSERT, CopyMemory, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, RGB, RTL_CONSTANT_STRING, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SUBLANGID, UNICODE_NULL, UNREFERENCED_PARAMETER, UlongToPtr, ZeroMemory, _countof, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const GB: ULONGLONG = 0x40000000; // clang, partlist.c
pub const IOCTL_CONSOLE_FILL_OUTPUT_ATTRIBUTE: i32 = 0x1CA040; // clang, console.c
pub const IOCTL_CONSOLE_FILL_OUTPUT_CHARACTER: i32 = 0x1CA080; // clang, console.c
pub const IOCTL_CONSOLE_GET_SCREEN_BUFFER_INFO: i32 = 0x1C6004; // clang, console.c
pub const IOCTL_CONSOLE_LOADFONT: i32 = 0x1CA101; // clang, console.c
pub const IOCTL_CONSOLE_RESET_SCREEN: i32 = 0x1CA000; // clang, console.c
pub const IOCTL_CONSOLE_SET_CURSOR_INFO: i32 = 0x1CA010; // clang, console.c
pub const IOCTL_CONSOLE_SET_SCREEN_BUFFER_INFO: i32 = 0x1CA008; // clang, console.c
pub const IOCTL_CONSOLE_SET_TEXT_ATTRIBUTE: i32 = 0x1CA04C; // clang, console.c
pub const IOCTL_CONSOLE_WRITE_OUTPUT_CHARACTER: i32 = 0x1CA089; // clang, console.c
pub const KB: ULONGLONG = 0x400; // clang, partlist.c
pub const MAX_WBITS: i32 = 0xF; // clang, cabinet.c
pub const MB: ULONGLONG = 0x100000; // clang, partlist.c
pub const STATUS_APP_INIT_FAILURE: NTSTATUS = 0xC0000145_u32 as NTSTATUS; // clang, usetup.c
pub const STATUS_PARTITION_FAILURE: NTSTATUS = 0xC0000172_u32 as NTSTATUS; // clang, usetup.c
pub const STD_INPUT_HANDLE: DWORD = 0xFFFFFFF6; // clang, console.c
pub const STD_OUTPUT_HANDLE: DWORD = 0xFFFFFFF5; // clang, cmdcons.c
// ==== end port facts ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ANSI_NULL: CHAR = 0; // freeldr-rust/src/ui_keystone.rs
pub const BACKGROUND_BLUE: u16 = 16; // winsrv-sys/src/bindgen_amd64.rs
pub const BOOTLOADER_INSTALL_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_BOOTLOADER_INSTALL_PAGE; // island enum
pub const BOOTLOADER_REMOVABLE_DISK_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_BOOTLOADER_REMOVABLE_DISK_PAGE; // island enum
pub const BOOTLOADER_SELECT_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_BOOTLOADER_SELECT_PAGE; // island enum
pub const CAPSLOCK_ON: u32 = 128; // winsrv-sys/src/bindgen_amd64.rs
pub const CHANGE_SYSTEM_PARTITION: _PAGE_NUMBER = _PAGE_NUMBER_CHANGE_SYSTEM_PARTITION; // island enum
pub const CHECK_FILE_SYSTEM_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_CHECK_FILE_SYSTEM_PAGE; // island enum
pub const COMPUTER_SETTINGS_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_COMPUTER_SETTINGS_PAGE; // island enum
pub const CONFIRM_DELETE_SYSTEM_PARTITION_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_CONFIRM_DELETE_SYSTEM_PARTITION_PAGE; // island enum
pub const CREATE_PARTITION_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_CREATE_PARTITION_PAGE; // island enum
pub const DELETE_PARTITION_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_DELETE_PARTITION_PAGE; // island enum
pub const DEVICE_SETTINGS_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_DEVICE_SETTINGS_PAGE; // island enum
pub const DISPLAY_SETTINGS_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_DISPLAY_SETTINGS_PAGE; // island enum
pub const ENHANCED_KEY: u32 = 256; // winsrv-sys/src/bindgen_amd64.rs
pub const ERROR_DIRECTORY_NAME: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_DIRECTORY_NAME; // island enum
pub const ERROR_DRIVE_INFORMATION: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_DRIVE_INFORMATION; // island enum
pub const ERROR_FORMATTING_PARTITION: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_FORMATTING_PARTITION; // island enum
pub const ERROR_INSTALL_BOOTCODE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_INSTALL_BOOTCODE; // island enum
pub const ERROR_INSUFFICIENT_PARTITION_SIZE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_INSUFFICIENT_PARTITION_SIZE; // island enum
pub const ERROR_LAST_ERROR_CODE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_LAST_ERROR_CODE; // island enum
pub const ERROR_LOAD_COMPUTER: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_LOAD_COMPUTER; // island enum
pub const ERROR_LOAD_DISPLAY: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_LOAD_DISPLAY; // island enum
pub const ERROR_LOAD_KBLAYOUT: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_LOAD_KBLAYOUT; // island enum
pub const ERROR_LOAD_KEYBOARD: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_LOAD_KEYBOARD; // island enum
pub const ERROR_NOT_INSTALLED: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_NOT_INSTALLED; // island enum
pub const ERROR_NO_BUILD_PATH: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_NO_BUILD_PATH; // island enum
pub const ERROR_NO_FLOPPY: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_NO_FLOPPY; // island enum
pub const ERROR_NO_HDD: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_NO_HDD; // island enum
pub const ERROR_SOURCE_DIR: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_SOURCE_DIR; // island enum
pub const ERROR_SOURCE_PATH: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_SOURCE_PATH; // island enum
pub const ERROR_WRITE_BOOT: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_WRITE_BOOT; // island enum
pub const ERROR_WRITE_PTABLE: _ERROR_NUMBER = _ERROR_NUMBER_ERROR_WRITE_PTABLE; // island enum
pub const FILE_ALL_ACCESS: u32 = 2032127; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const FILE_COPY_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_FILE_COPY_PAGE; // island enum
pub const FLUSH_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_FLUSH_PAGE; // island enum
pub const FMIFS_HARDDISK: FMIFS_MEDIA_FLAG = FMIFS_MEDIA_FLAG_FMIFS_HARDDISK; // island enum
pub const FOREGROUND_BLUE: u16 = 1; // winsrv-sys/src/bindgen_amd64.rs
pub const FOREGROUND_INTENSITY: u16 = 8; // winsrv-sys/src/bindgen_amd64.rs
pub const FOREGROUND_RED: u16 = 4; // winsrv-sys/src/bindgen_amd64.rs
pub const FORMAT_PARTITION_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_FORMAT_PARTITION_PAGE; // island enum
pub const FSVOL_ABORT: _FSVOL_OP = _FSVOL_OP_FSVOL_ABORT; // island enum
pub const FSVOL_DOIT: _FSVOL_OP = _FSVOL_OP_FSVOL_DOIT; // island enum
pub const FSVOL_FORMAT: _FSVOL_OP = _FSVOL_OP_FSVOL_FORMAT; // island enum
pub const FSVOL_RETRY: _FSVOL_OP = _FSVOL_OP_FSVOL_RETRY; // island enum
pub const FSVOL_SKIP: _FSVOL_OP = _FSVOL_OP_FSVOL_SKIP; // island enum
pub const INSTALL_DIRECTORY_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_INSTALL_DIRECTORY_PAGE; // island enum
pub const INSTALL_INTRO_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_INSTALL_INTRO_PAGE; // island enum
pub const IOCTL_DISK_GET_DRIVE_GEOMETRY: u32 = 0x00070000; // disk-rust/src/glue.rs
pub const IOCTL_DISK_GET_DRIVE_LAYOUT: i32 = 0x7400C; // setuplib-rust/src/prelude.rs
pub const IOCTL_KEYBOARD_SET_INDICATORS: u32 = 0x000B_0008; // i8042prt-rust/src/i8042prt_h.rs
pub const KEYBOARD_SETTINGS_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_KEYBOARD_SETTINGS_PAGE; // island enum
pub const KEY_EVENT: u32 = 1; // msvcrt-sys/src/bindgen_amd64.rs
pub const LANGUAGE_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_LANGUAGE_PAGE; // island enum
pub const LAYOUT_SETTINGS_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_LAYOUT_SETTINGS_PAGE; // island enum
pub const LEFT_ALT_PRESSED: u32 = 2; // msvcrt-sys/src/bindgen_amd64.rs
pub const LEFT_CTRL_PRESSED: u32 = 8; // msvcrt-sys/src/bindgen_amd64.rs
pub const LICENSE_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_LICENSE_PAGE; // island enum
pub const MUTANT_ALL_ACCESS: u32 = 2031617; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const NOT_AN_ERROR: _ERROR_NUMBER = _ERROR_NUMBER_NOT_AN_ERROR; // island enum
pub const NUMLOCK_ON: u32 = 32; // winsrv-sys/src/bindgen_amd64.rs
pub const PREPARE_COPY_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_PREPARE_COPY_PAGE; // island enum
pub const QUIT_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_QUIT_PAGE; // island enum
pub const REBOOT_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_REBOOT_PAGE; // island enum
pub const RECOVERY_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_RECOVERY_PAGE; // island enum
pub const REGISTRY_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_REGISTRY_PAGE; // island enum
pub const REPAIR_INTRO_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_REPAIR_INTRO_PAGE; // island enum
pub const RIGHT_ALT_PRESSED: u32 = 1; // msvcrt-sys/src/bindgen_amd64.rs
pub const RIGHT_CTRL_PRESSED: u32 = 4; // msvcrt-sys/src/bindgen_amd64.rs
pub const SCROLLLOCK_ON: u32 = 64; // winsrv-sys/src/bindgen_amd64.rs
pub const SELECT_FILE_SYSTEM_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_SELECT_FILE_SYSTEM_PAGE; // island enum
pub const SELECT_PARTITION_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_SELECT_PARTITION_PAGE; // island enum
pub const SETUP_INIT_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_SETUP_INIT_PAGE; // island enum
pub const SHIFT_PRESSED: u32 = 16; // msvcrt-sys/src/bindgen_amd64.rs
pub const START_PARTITION_OPERATIONS_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_START_PARTITION_OPERATIONS_PAGE; // island enum
pub const STATUS_DEVICE_NOT_READY: i32 = 0xC00000A3u32 as i32; // classpnp-rust/src/glue.rs
pub const STATUS_UNRECOGNIZED_VOLUME: i32 = 0xC000014Fu32 as i32; // fs_rec-rust/src/glue.rs
pub const STATUS_WAIT_0: NTSTATUS = 0x00000000; // kernel32-rust/src/client/synch.rs
pub const SUCCESS_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_SUCCESS_PAGE; // island enum
pub const UPGRADE_REPAIR_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_UPGRADE_REPAIR_PAGE; // island enum
pub const Unformatted: _FORMATSTATE = _FORMATSTATE_Unformatted; // island enum
pub const WELCOME_PAGE: _PAGE_NUMBER = _PAGE_NUMBER_WELCOME_PAGE; // island enum
extern "C" {
    pub fn CombinePaths(PathBuffer: PWSTR, cchPathSize: usize, NumberOfPathComponents: u32, ...) -> i32; // setuplib-rust/src/setuplib/filesup.rs
    pub fn ConcatPaths(PathBuffer: PWSTR, cchPathSize: usize, NumberOfPathComponents: u32, ...) -> i32; // setuplib-rust/src/setuplib/filesup.rs
    pub fn CreateComputerTypeList(InfFile: HINF) -> PGENERIC_LIST; // setuplib-rust/src/setuplib/settings.rs
    pub fn CreateDisplayDriverList(InfFile: HINF) -> PGENERIC_LIST; // setuplib-rust/src/setuplib/settings.rs
    pub fn CreateKeyboardDriverList(InfFile: HINF) -> PGENERIC_LIST; // setuplib-rust/src/setuplib/settings.rs
    pub fn CreateKeyboardLayoutList(InfFile: HINF, LanguageId: PCWSTR, DefaultKBLayout: PWSTR,) -> PGENERIC_LIST; // setuplib-rust/src/setuplib/settings.rs
    pub fn CreateLanguageList(InfFile: HINF, DefaultLanguage: PWSTR,) -> PGENERIC_LIST; // setuplib-rust/src/setuplib/settings.rs
    pub fn GetDefaultLanguageIndex() -> u32; // setuplib-rust/src/setuplib/settings.rs
    pub fn INF_GetDataField(Context: PINFCONTEXT, FieldIndex: ULONG, Data: *mut PCWSTR,) -> BOOLEAN; // setuplib-rust/src/setuplib/infsupp.rs
    pub fn InfSetHeap(Heap: PVOID); // inflib-rust/src/infrosgen.rs
    pub fn IsPartitionActive(PartEntry: PPARTENTRY) -> BOOLEAN; // setuplib-rust/src/setuplib/partlist.rs
    pub fn MUIDefaultKeyboardLayout(LanguageId: PCWSTR) -> KLID; // setuplib-rust/src/setuplib/mui.rs
    pub fn SelectPartition(List: PPARTLIST, DiskNumber: ULONG, PartitionNumber: ULONG,) -> PPARTENTRY; // setuplib-rust/src/setuplib/partlist.rs
    pub fn _atoi64(nptr: *const u8) -> i64; // cnt-rust/src/cabi/generated.rs
    pub fn _stricmp(s1: PCSTR, s2: PCSTR) -> i32; // freeldr-rust/src/bootmgr.rs
    pub fn _swprintf(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int; // rtl-actctx/src/shim.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // inflib-rust/src/infcore.rs
    pub fn _wcstoui64(nptr: *const u16, endptr: *mut *mut u16, base: i32) -> u64; // cnt-rust/src/cabi/generated.rs
    pub fn abs(x: i32) -> i32; // cmlib-rust/src/cmcheck.rs
    pub fn iscntrl(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn isprint(c: c_int) -> c_int; // dhcpcsvc-rust/src/prelude.rs
    pub fn isspace(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32; // freeldr-rust/src/hwdisk.rs
    pub fn strchr(s: *const i8, c: i32) -> *mut i8; // freeldr-rust/src/frldr.rs
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int; // browser-rust/src/prelude.rs
    pub fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8; // cabinet-rust/src/cabinet/cabinet_main.rs
    pub fn strlen(s: *const i8) -> usize; // advapi32-rust/src/prelude.rs
    pub fn strtoul(s: PCSTR, endptr: *mut PCSTR, base: i32) -> u32; // freeldr-rust/src/drivemap.rs
    pub fn tolower(c: c_int) -> c_int; // kernel32-rust/src/prelude.rs
    pub fn toupper(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // acpi-rust/src/buspdo.rs
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16; // kernel32-rust/src/prelude.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // inflib-rust/src/infget.rs
    pub fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32; // concfg-rust/src/font.rs
}
extern "system" {
    pub fn CheckUnattendedSetup(pSetupData: PUSETUP_DATA,) -> BOOLEAN; // setuplib-rust/src/setuplib/setuplib.rs
    pub fn CreateNTOSInstallationsList(PartList: *mut PARTLIST,) -> *mut GENERIC_LIST; // setuplib-rust/src/setuplib/osdetect.rs
    pub fn CreatePartition(List: PPARTLIST, PartEntry: PPARTENTRY, SizeBytes: ULONGLONG, PartitionInfo: ULONG_PTR,) -> BOOLEAN; // setuplib-rust/src/setuplib/partlist.rs
    pub fn CreatePartitionList() -> *mut PARTLIST; // setuplib-rust/src/setuplib/partlist.rs
    pub fn DeletePartition(List: PPARTLIST, PartEntry: PPARTENTRY, FreeRegion: *mut PPARTENTRY,) -> BOOLEAN; // setuplib-rust/src/setuplib/partlist.rs
    pub fn DestroyGenericList(List: PGENERIC_LIST, FreeData: BOOLEAN,); // setuplib-rust/src/setuplib/genlist.rs
    pub fn DestroyPartitionList(List: *mut PARTLIST); // setuplib-rust/src/setuplib/partlist.rs
    pub fn DoFileCopy(pSetupData: *mut USETUP_DATA, MsgHandler: PSP_FILE_CALLBACK_W, Context: *mut core::ffi::c_void,) -> i32; // setuplib-rust/src/setuplib/install.rs
    pub fn FinishSetup(pSetupData: PUSETUP_DATA); // setuplib-rust/src/setuplib/setuplib.rs
    pub fn FsVolCommitOpsQueue(PartitionList: PPARTLIST, SystemVolume: PVOLENTRY, InstallVolume: PVOLENTRY, FsVolCallback: PFSVOL_CALLBACK, Context: PVOID,) -> BOOLEAN; // setuplib-rust/src/setuplib/fsutil.rs
    pub fn GetFirstListEntry(List: PGENERIC_LIST) -> PGENERIC_LIST_ENTRY; // setuplib-rust/src/setuplib/genlist.rs
    pub fn GetListEntryData(Entry: PGENERIC_LIST_ENTRY) -> PVOID; // setuplib-rust/src/setuplib/genlist.rs
    pub fn GetNextListEntry(Entry: PGENERIC_LIST_ENTRY,) -> PGENERIC_LIST_ENTRY; // setuplib-rust/src/setuplib/genlist.rs
    pub fn InitDestinationPaths(pSetupData: *mut USETUP_DATA, InstallationDir: PCWSTR, Volume: *mut VOLENTRY,) -> NTSTATUS; // setuplib-rust/src/setuplib/setuplib.rs
    pub fn InitSystemPartition(PartitionList: PPARTLIST, InstallPartition: PPARTENTRY, pSystemPartition: *mut PPARTENTRY, FsVolCallback: PFSVOL_CALLBACK, Context: PVOID,) -> BOOLEAN; // setuplib-rust/src/setuplib/setuplib.rs
    pub fn InitializeSetup(pSetupData: PUSETUP_DATA, ErrorRoutine: PSETUP_ERROR_ROUTINE, pSpFileExports: PSPFILE_EXPORTS, pSpInfExports: PSPINF_EXPORTS,) -> ERROR_NUMBER; // setuplib-rust/src/setuplib/setuplib.rs
    pub fn InstallBootManagerAndBootEntries(ArchType: ARCHITECTURE_TYPE, SystemRootPath: PCUNICODE_STRING, SourceRootPath: PCUNICODE_STRING, DestinationArcPath: PCUNICODE_STRING, Options: usize,) -> i32; // setuplib-rust/src/setuplib/bootsup.rs
    pub fn InstallBootcodeToRemovable(ArchType: ARCHITECTURE_TYPE, RemovableRootPath: PCUNICODE_STRING, SourceRootPath: PCUNICODE_STRING, DestinationArcPath: PCUNICODE_STRING,) -> NTSTATUS; // setuplib-rust/src/setuplib/bootsup.rs
    pub fn InstallSetupInfFile(pSetupData: PUSETUP_DATA); // setuplib-rust/src/setuplib/setuplib.rs
    pub fn IsValidInstallDirectory(InstallDir: PCWSTR) -> i32; // setuplib-rust/src/setuplib/setuplib.rs
    pub fn PartitionCreateChecks(PartEntry: PPARTENTRY, SizeBytes: ULONGLONG, PartitionInfo: ULONG_PTR,) -> ERROR_NUMBER; // setuplib-rust/src/setuplib/partlist.rs
    pub fn PrepareFileCopy(pSetupData: PUSETUP_DATA, StatusRoutine: PFILE_COPY_STATUS_ROUTINE,) -> BOOLEAN; // setuplib-rust/src/setuplib/install.rs
    pub fn SetCurrentListEntry(List: PGENERIC_LIST, Entry: PGENERIC_LIST_ENTRY,); // setuplib-rust/src/setuplib/genlist.rs
    pub fn UpdateRegistry(pSetupData: *mut USETUP_DATA, RepairUpdateFlag: BOOLEAN, PartitionList: *mut PARTLIST, DestinationDriveLetter: WCHAR, SelectedLanguageId: PCWSTR, StatusRoutine: PREGISTRY_STATUS_ROUTINE, SubstSettings: *mut FONTSUBSTSETTINGS,) -> ERROR_NUMBER; // setuplib-rust/src/setuplib/setuplib.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// CAB_CODEC: c2rust, cabinet.c
pub type CAB_CODEC = _CAB_CODEC;
// CFHEADER: c2rust, cabinet.c
pub type CFHEADER = _CFHEADER;
// COMMAND: c2rust, cmdcons.c
pub type COMMAND = tagCOMMAND;
// CONSOLE_STATE: c2rust, cmdcons.c
pub type CONSOLE_STATE = _CONSOLE_STATE;
// PCABINET_CODEC_UNCOMPRESS: c2rust, cabinet.c
pub type PCABINET_CODEC_UNCOMPRESS =
    Option<unsafe extern "C" fn(*mut _CAB_CODEC, PVOID, PVOID, PLONG, PLONG) -> ULONG>;
// PCONSOLE_STATE: c2rust, cmdcons.c
pub type PCONSOLE_STATE = *mut _CONSOLE_STATE;
// SCANTOASCII: c2rust, keytrans.c
pub type SCANTOASCII = _SCANTOASCII;
// _CAB_CODEC: c2rust, cabinet.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CAB_CODEC {
    pub Uncompress: PCABINET_CODEC_UNCOMPRESS,
    pub ZStream: z_stream,
}
// _CFDATA: c2rust, cabinet.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CFDATA {
    pub Checksum: ULONG,
    pub CompSize: USHORT,
    pub UncompSize: USHORT,
}
// _CFFILE: c2rust, cabinet.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CFFILE {
    pub FileSize: ULONG,
    pub FileOffset: ULONG,
    pub FolderIndex: USHORT,
    pub FileDate: USHORT,
    pub FileTime: USHORT,
    pub Attributes: USHORT,
    pub FileName: [CHAR; 1],
}
// _CFFOLDER: c2rust, cabinet.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CFFOLDER {
    pub DataOffset: ULONG,
    pub DataBlockCount: USHORT,
    pub CompressionType: USHORT,
}
// _CFHEADER: c2rust, cabinet.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CFHEADER {
    pub Signature: ULONG,
    pub Reserved1: ULONG,
    pub CabinetSize: ULONG,
    pub Reserved2: ULONG,
    pub FileTableOffset: ULONG,
    pub Reserved3: ULONG,
    pub Version: USHORT,
    pub FolderCount: USHORT,
    pub FileCount: USHORT,
    pub Flags: USHORT,
    pub SetID: USHORT,
    pub CabinetNumber: USHORT,
}
// _CONSOLE_STATE: c2rust, cmdcons.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _CONSOLE_STATE {
    pub maxx: SHORT,
    pub maxy: SHORT,
    pub bInsert: BOOLEAN,
    pub bExit: BOOLEAN,
}
// _SCANTOASCII: c2rust, keytrans.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _SCANTOASCII {
    pub ScanCode: USHORT,
    pub Enhanced: USHORT,
    pub Normal: UCHAR,
    pub Shift: UCHAR,
    pub NumLock: UCHAR,
    pub bCAPS: UCHAR,
}
// tagCOMMAND: c2rust, cmdcons.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagCOMMAND {
    pub name: LPSTR,
    pub flags: INT,
    pub func: Option<unsafe extern "C" fn(PCONSOLE_STATE, LPSTR) -> INT>,
    pub help: Option<unsafe extern "C" fn() -> ()>,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn InfFreeContext(Context: PINFCONTEXT); // infsupp.c
    pub fn InfGetBinaryField(
        Context: PINFCONTEXT,
        FieldIndex: ULONG,
        ReturnBuffer: PUCHAR,
        ReturnBufferSize: ULONG,
        RequiredSize: PULONG,
    ) -> BOOLEAN; // infsupp.c
    pub fn InfGetFieldCount(Context: PINFCONTEXT) -> LONG; // infsupp.c
    pub fn InfGetIntField(Context: PINFCONTEXT, FieldIndex: ULONG, IntegerValue: *mut INT) -> BOOLEAN; // infsupp.c
    pub fn InfGetMultiSzField(
        Context: PINFCONTEXT,
        FieldIndex: ULONG,
        ReturnBuffer: PWSTR,
        ReturnBufferSize: ULONG,
        RequiredSize: PULONG,
    ) -> BOOLEAN; // infsupp.c
    pub fn InfGetStringField(
        Context: PINFCONTEXT,
        FieldIndex: ULONG,
        ReturnBuffer: PWSTR,
        ReturnBufferSize: ULONG,
        RequiredSize: PULONG,
    ) -> BOOLEAN; // infsupp.c
    pub fn InfOpenBufferedFile(
        InfHandle: PHINF,
        Buffer: PVOID,
        BufferSize: ULONG,
        LanguageId: LANGID,
        ErrorLine: PULONG,
    ) -> NTSTATUS; // infsupp.c
    pub fn MUIGetOEMCodePage(LanguageId: PCWSTR) -> UINT; // mui.c
    pub fn RoundingDivide(Dividend: ULONGLONG, Divisor: ULONGLONG) -> ULONGLONG; // partlist.c
    pub fn SetupCopyFile(
        SourceFileName: PCWSTR,
        DestinationFileName: PCWSTR,
        FailIfExists: BOOLEAN,
    ) -> NTSTATUS; // fileqsup.c
    pub fn SetupDeleteFile(FileName: PCWSTR, ForceDelete: BOOLEAN) -> NTSTATUS; // fileqsup.c
    pub fn SetupMoveFile(ExistingFileName: PCWSTR, NewFileName: PCWSTR, Flags: ULONG) -> NTSTATUS; // fileqsup.c
    pub fn inflate(strm: z_streamp, flush: ::core::ffi::c_int) -> ::core::ffi::c_int; // cabinet.c
    pub fn inflateEnd(strm: z_streamp) -> ::core::ffi::c_int; // cabinet.c
}
extern "system" {
    pub fn GetCurrentListEntry(List: PGENERIC_LIST) -> PGENERIC_LIST_ENTRY; // genlist.c
    pub fn GetNextPartition(List: PPARTLIST, CurrentPart: PPARTENTRY) -> PPARTENTRY; // partlist.c
    pub fn GetNumberOfListEntries(List: PGENERIC_LIST) -> ULONG; // genlist.c
    pub fn GetPrevPartition(List: PPARTLIST, CurrentPart: PPARTENTRY) -> PPARTENTRY; // partlist.c
    pub fn GetRegisteredFileSystems(Index: ULONG, FileSystemName: *mut PCWSTR) -> BOOLEAN; // fslist.c
    pub fn LookupPartitionTypeString(PartitionStyle: PARTITION_STYLE, PartitionType: PVOID) -> PCSTR; // partlist.c
}
pub use crate::prelude::_snprintf as snprintf; // sdk alias
// ==== end port externs ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::usetup::consup::CONSOLE_ConOutPrintf;
pub use crate::usetup::mui::CharBlock;
pub use crate::usetup::mui::CharDoubleHorizontalLine;
pub use crate::usetup::mui::CharDoubleLowerLeftCorner;
pub use crate::usetup::mui::CharDoubleLowerRightCorner;
pub use crate::usetup::mui::CharDoubleUpperLeftCorner;
pub use crate::usetup::mui::CharDoubleUpperRightCorner;
pub use crate::usetup::mui::CharDoubleVerticalLine;
pub use crate::usetup::mui::CharDownArrow;
pub use crate::usetup::mui::CharHalfBlock;
pub use crate::usetup::mui::CharHorizontalLine;
pub use crate::usetup::mui::CharLeftHorizLineAndVertLine;
pub use crate::usetup::mui::CharLowerLeftCorner;
pub use crate::usetup::mui::CharLowerRightCorner;
pub use crate::usetup::mui::CharUpArrow;
pub use crate::usetup::mui::CharUpperLeftCorner;
pub use crate::usetup::mui::CharUpperRightCorner;
pub use crate::usetup::mui::CharVertLineAndRightHorizLine;
pub use crate::usetup::mui::CharVerticalLine;
pub use crate::usetup::usetup::ProcessHeap;
pub use crate::usetup::usetup::SelectedLanguageId;
pub use crate::usetup::fileqsup::SpFileExports;
pub use crate::usetup::infsupp::SpInfExports;
pub use crate::usetup::consup::StdInput;
pub use crate::usetup::consup::StdOutput;
pub use crate::usetup::consup::xScreen;
pub use crate::usetup::consup::yScreen;
// ==== end cross-TU definitions ====

// ---- hand-ported header items (NOT generated: outside every `// ==== ... ====` block, so
// `port resolve` treats these names as provided; banked with `rustconv port hand`) ----

// sdk/lib/inflib/infros.h -- usetup links INFLIB. The tree-consensus declarations resolve
// picked came from freeldr's PCSTR INF parser (InfOpenFile returning a u8 where inflib
// returns an NTSTATUS, InfFindFirstLine writing the context in place instead of returning
// a PINFCONTEXT through a pointer), so the inflib prototypes are declared here.
extern "C" {
    pub fn InfCloseFile(InfHandle: HINF);
    pub fn InfFindFirstLine(InfHandle: HINF, Section: PCWSTR, Key: PCWSTR, Context: *mut PINFCONTEXT) -> BOOLEAN;
    pub fn InfFindNextLine(ContextIn: PINFCONTEXT, ContextOut: PINFCONTEXT) -> BOOLEAN;
    pub fn InfGetDataField(Context: PINFCONTEXT, FieldIndex: ULONG, Data: *mut PWCHAR) -> BOOLEAN;
    pub fn InfOpenFile(InfHandle: PHINF, FileName: PUNICODE_STRING, LanguageId: LANGID, ErrorLine: PULONG) -> NTSTATUS;
}
// stdio.h: `int vsprintf(char *buffer, const char *format, va_list argptr)` (msvcrt).
extern "C" {
    pub fn vsprintf(buffer: *mut c_char, format: *const c_char, argptr: va_list) -> c_int;
}

// zlib.h:1939 (no Z_PREFIX, no ZLIB_WINAPI: cdecl, as `inflate`/`inflateEnd` above).
// `#define inflateInit2(strm, windowBits) inflateInit2_((strm), (windowBits), ZLIB_VERSION, (int)sizeof(z_stream))`
extern "C" {
    pub fn inflateInit2_(
        strm: z_streamp,
        windowBits: c_int,
        version: *const c_char,
        stream_size: c_int,
    ) -> c_int;
}
#[inline(always)]
pub unsafe fn inflateInit2(strm: z_streamp, windowBits: c_int) -> c_int {
    inflateInit2_(
        strm,
        windowBits,
        ZLIB_VERSION.as_ptr() as *const c_char,
        core::mem::size_of::<z_stream>() as c_int,
    )
}

// ndk/rtlfuncs.h:2001 (NTOS_MODE_USER):
// `#define RtlUnicodeStringToOemSize(STRING) (NLS_MB_OEM_CODE_PAGE_TAG ?
//      RtlxUnicodeStringToOemSize(STRING) : ((STRING)->Length + sizeof(UNICODE_NULL)) / sizeof(WCHAR))`
// ndk/rtltypes.h:401: `extern BOOLEAN NTSYSAPI NLS_MB_OEM_CODE_PAGE_TAG;` -- an ntdll DATA
// import, so it is read through the import (as win32k-rust reads NlsMbCodePageTag).
extern "system" {
    pub fn RtlxUnicodeStringToOemSize(UnicodeString: PCUNICODE_STRING) -> ULONG;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntdll.dll", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    // Renamed in Rust only: ntdll-sys's glob already exports a (non-import) declaration of
    // this name, and a private item would shadow it.
    #[link_name = "NlsMbOemCodePageTag"]
    static NlsMbOemCodePageTag_dll: BOOLEAN;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_NlsMbOemCodePageTag"]
    static NlsMbOemCodePageTag_imp: *const BOOLEAN;
}
#[inline(always)]
pub unsafe fn NLS_MB_OEM_CODE_PAGE_TAG() -> BOOLEAN {
    #[cfg(target_arch = "x86_64")]
    {
        *NlsMbOemCodePageTag_imp
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        NlsMbOemCodePageTag_dll
    }
}
#[inline(always)]
pub unsafe fn RtlUnicodeStringToOemSize(STRING: PCUNICODE_STRING) -> ULONG {
    if NLS_MB_OEM_CODE_PAGE_TAG() != 0 {
        RtlxUnicodeStringToOemSize(STRING)
    } else {
        // size_t arithmetic, truncated to the ULONG the caller stores it in.
        (((*STRING).Length as usize + core::mem::size_of::<WCHAR>()) / core::mem::size_of::<WCHAR>())
            as ULONG
    }
}

// base/setup/lib/spapisup/infsupp.h:152: header-inline `INF_FreeData` (the `#if 0` arm is dead).
#[allow(dead_code)]
#[inline]
pub unsafe fn INF_FreeData(InfData: PCWSTR) {
    UNREFERENCED_PARAMETER!(InfData);
}
// base/setup/lib/spapisup/infsupp.h:140: #define SpInfFindFirstLine (SpInfExports.SpInfFindFirstLine)
// (usetup's own spapisup/infsupp.c fills SpInfExports, so the pointer is always set.)
#[allow(unused_macros)]
macro_rules! SpInfFindFirstLine {
    ($($arg:expr),* $(,)?) => {
        ((SpInfExports.SpInfFindFirstLine.unwrap_unchecked())($($arg),*))
    };
}
#[allow(unused_imports)]
pub(crate) use SpInfFindFirstLine;

// base/setup/lib/utils/partlist.h:255: #define GetPartEntrySizeInBytes(PartEntry) ((PartEntry)->SectorCount.QuadPart * (PartEntry)->DiskEntry->BytesPerSector)
#[allow(unused_macros)]
macro_rules! GetPartEntrySizeInBytes {
    ($PartEntry:expr $(,)?) => {
        (((*$PartEntry).SectorCount.QuadPart) * ((*(*$PartEntry).DiskEntry).BytesPerSector as ULONGLONG))
    };
}
#[allow(unused_imports)]
pub(crate) use GetPartEntrySizeInBytes;
// base/setup/lib/utils/partlist.h:258: #define GetDiskSizeInBytes(DiskEntry) ((DiskEntry)->SectorCount.QuadPart * (DiskEntry)->BytesPerSector)
#[allow(unused_macros)]
macro_rules! GetDiskSizeInBytes {
    ($DiskEntry:expr $(,)?) => {
        ((*$DiskEntry).SectorCount.QuadPart * ((*$DiskEntry).BytesPerSector as ULONGLONG))
    };
}
#[allow(unused_imports)]
pub(crate) use GetDiskSizeInBytes;

// base/setup/lib/utils/volutil.h:30-40:
// #define IS_RAWFS(fs) ((fs)[0] == 'R' && (fs)[1] == 'A' && (fs)[2] == 'W' && (fs)[3] == 0)
// #define IsUnknown(VolInfo) (!*(VolInfo)->FileSystem)
// #define IsUnformatted(VolInfo) IS_RAWFS((VolInfo)->FileSystem)
// #define IsFormatted(VolInfo) (!IsUnknown(VolInfo) && !IsUnformatted(VolInfo))
#[allow(unused_macros)]
macro_rules! IS_RAWFS {
    ($fs:expr $(,)?) => {
        (($fs[0] == b'R' as WCHAR) && ($fs[1] == b'A' as WCHAR) && ($fs[2] == b'W' as WCHAR) && ($fs[3] == 0))
    };
}
#[allow(unused_imports)]
pub(crate) use IS_RAWFS;
#[allow(unused_macros)]
macro_rules! IsUnknown {
    ($VolInfo:expr $(,)?) => {
        ((*$VolInfo).FileSystem[0] == 0)
    };
}
#[allow(unused_imports)]
pub(crate) use IsUnknown;
#[allow(unused_macros)]
macro_rules! IsUnformatted {
    ($VolInfo:expr $(,)?) => {
        (IS_RAWFS!((*$VolInfo).FileSystem))
    };
}
#[allow(unused_imports)]
pub(crate) use IsUnformatted;
#[allow(unused_macros)]
macro_rules! IsFormatted {
    ($VolInfo:expr $(,)?) => {
        (!IsUnknown!($VolInfo) && !IsUnformatted!($VolInfo))
    };
}
#[allow(unused_imports)]
pub(crate) use IsFormatted;

// base/setup/lib/setuplib.h:206:
// #define IS_VALID_INSTALL_PATH_CHAR(c) (isalnum(c) || (c) == L'.' || (c) == L'\\' || (c) == L'-' || (c) == L'_')
extern "C" {
    pub fn isalnum(c: c_int) -> c_int;
}
#[allow(unused_macros)]
macro_rules! IS_VALID_INSTALL_PATH_CHAR {
    ($c:expr $(,)?) => {
        // isalnum takes the promoted int; the rest compare WCHARs.
        ((isalnum(($c) as c_int) != 0) || (($c) as WCHAR == b'.' as WCHAR) || (($c) as WCHAR == b'\\' as WCHAR) || (($c) as WCHAR == b'-' as WCHAR) || (($c) as WCHAR == b'_' as WCHAR))
    };
}
#[allow(unused_imports)]
pub(crate) use IS_VALID_INSTALL_PATH_CHAR;

// msvcrt `_snprintf`: `_vsnprintf` with the arguments in place of the `va_list`, which is how
// the ntstrsafe A printf worker below is driven (strsafe-rust drives its W twin the same way).
extern "C" {
    pub fn _snprintf(buf: *mut c_char, count: usize, format: *const c_char, ...) -> c_int;
}

/// ddk/ntstrsafe.h:2894 `RtlStringVPrintfExWorkerA`, its one `_vsnprintf(pszDest, cchMax,
/// pszFormat, argList)` handed in as `print(pszDest, cchMax, pszFormat)`. Unlike the W worker
/// strsafe-rust ports, this A worker formats into `cchDest - 1` and forces the terminator.
#[allow(clippy::too_many_arguments)]
pub unsafe fn RtlStringVPrintfExWorkerA(
    pszDest: *mut c_char,
    cchDest: usize,
    cbDest: usize,
    ppszDestEnd: *mut *mut c_char,
    pcchRemaining: *mut usize,
    dwFlags: DWORD,
    mut pszFormat: *const c_char,
    print: impl FnOnce(*mut c_char, usize, *const c_char) -> c_int,
) -> NTSTATUS {
    use strsafe_rust::ntstrsafe::{STATUS_BUFFER_OVERFLOW, STATUS_INVALID_PARAMETER};
    use strsafe_rust::{
        STRSAFE_FILL_BEHIND_NULL, STRSAFE_FILL_ON_FAILURE, STRSAFE_IGNORE_NULLS,
        STRSAFE_NO_TRUNCATION, STRSAFE_NULL_ON_FAILURE, STRSAFE_VALID_FLAGS,
    };
    // ntstrsafe.h:45 `#define STRSAFE_GET_FILL_PATTERN(dwFlags) ((int)((dwFlags) & 0x000000FF))`
    let fill_pattern = (dwFlags & 0x0000_00FF) as c_int;
    let mut Status: NTSTATUS = 0; // STATUS_SUCCESS
    let mut pszDestEnd: *mut c_char = pszDest;
    let mut cchRemaining: usize = 0;

    if dwFlags & !STRSAFE_VALID_FLAGS != 0 {
        Status = STATUS_INVALID_PARAMETER;
    } else {
        if dwFlags & STRSAFE_IGNORE_NULLS != 0 {
            if pszDest.is_null() && (cchDest != 0 || cbDest != 0) {
                Status = STATUS_INVALID_PARAMETER;
            }
            if pszFormat.is_null() {
                pszFormat = b"\0".as_ptr() as *const c_char;
            }
        }
        if NT_SUCCESS(Status) {
            if cchDest == 0 {
                pszDestEnd = pszDest;
                cchRemaining = 0;
                if *pszFormat != 0 {
                    if pszDest.is_null() {
                        Status = STATUS_INVALID_PARAMETER;
                    } else {
                        Status = STATUS_BUFFER_OVERFLOW;
                    }
                }
            } else {
                let cchMax = cchDest - 1;
                let iRet = print(pszDest, cchMax, pszFormat);
                if iRet < 0 || (iRet as usize) > cchMax {
                    pszDestEnd = pszDest.add(cchMax);
                    cchRemaining = 1;
                    *pszDestEnd = 0;
                    Status = STATUS_BUFFER_OVERFLOW;
                } else if (iRet as usize) == cchMax {
                    pszDestEnd = pszDest.add(cchMax);
                    cchRemaining = 1;
                    *pszDestEnd = 0;
                } else if (iRet as usize) < cchMax {
                    pszDestEnd = pszDest.offset(iRet as isize);
                    cchRemaining = cchDest - iRet as usize;
                    if dwFlags & STRSAFE_FILL_BEHIND_NULL != 0 {
                        memset(
                            pszDestEnd.add(1) as *mut c_void,
                            fill_pattern,
                            ((cchRemaining - 1) * core::mem::size_of::<c_char>())
                                + (cbDest % core::mem::size_of::<c_char>()),
                        );
                    }
                }
            }
        }
    }

    if !NT_SUCCESS(Status) && !pszDest.is_null() {
        if dwFlags & STRSAFE_FILL_ON_FAILURE != 0 {
            memset(pszDest as *mut c_void, fill_pattern, cbDest);
            if fill_pattern == 0 {
                pszDestEnd = pszDest;
                cchRemaining = cchDest;
            } else if cchDest > 0 {
                pszDestEnd = pszDest.add(cchDest - 1);
                cchRemaining = 1;
                *pszDestEnd = 0;
            }
        }
        if dwFlags & (STRSAFE_NULL_ON_FAILURE | STRSAFE_NO_TRUNCATION) != 0 && cchDest > 0 {
            pszDestEnd = pszDest;
            cchRemaining = cchDest;
            *pszDestEnd = 0;
        }
    }

    if NT_SUCCESS(Status) || Status == STATUS_BUFFER_OVERFLOW {
        if !ppszDestEnd.is_null() {
            *ppszDestEnd = pszDestEnd;
        }
        if !pcchRemaining.is_null() {
            *pcchRemaining = cchRemaining;
        }
    }

    Status
}

/// ddk/ntstrsafe.h:1218 `RtlStringCchPrintfExA(pszDest, cchDest, ppszDestEnd, pcchRemaining,
/// dwFlags, pszFormat, ...)` -- a static inline over the A Ex worker (no ntdll export exists).
#[allow(unused_macros)]
macro_rules! RtlStringCchPrintfExA {
    ($pszDest:expr, $cchDest:expr, $ppszDestEnd:expr, $pcchRemaining:expr, $dwFlags:expr,
     $pszFormat:expr $(, $arg:expr)* $(,)?) => {{
        let pszDest: *mut c_char = $pszDest as _;
        let cchDest: usize = $cchDest as usize;
        if cchDest > strsafe_rust::ntstrsafe::NTSTRSAFE_MAX_CCH {
            if cchDest > 0 {
                *pszDest = 0;
            }
            strsafe_rust::ntstrsafe::STATUS_INVALID_PARAMETER
        } else {
            RtlStringVPrintfExWorkerA(
                pszDest,
                cchDest,
                cchDest,
                $ppszDestEnd as _,
                $pcchRemaining as _,
                $dwFlags as _,
                $pszFormat as _,
                |d, n, f| _snprintf(d, n, f $(, $arg)*),
            )
        }
    }};
}
#[allow(unused_imports)]
pub(crate) use RtlStringCchPrintfExA;
// The enum constants usetup's `switch`es name. bindgen prefixes them with the enum TAG, so
// without these plain names a `match` arm spelled `FSVOLNOTIFY_STARTQUEUE =>` is a BINDING
// that matches everything (field manual: a missing constant becomes a catch-all arm).
// base/setup/lib/fsutil.h `enum _FSVOLNOTIFY`:
pub const FSVOLNOTIFY_STARTQUEUE: FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTQUEUE;
pub const FSVOLNOTIFY_ENDQUEUE: FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDQUEUE;
pub const FSVOLNOTIFY_STARTSUBQUEUE: FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTSUBQUEUE;
pub const FSVOLNOTIFY_ENDSUBQUEUE: FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDSUBQUEUE;
pub const FSVOLNOTIFY_PARTITIONERROR: FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_PARTITIONERROR;
pub const FSVOLNOTIFY_STARTFORMAT: FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTFORMAT;
pub const FSVOLNOTIFY_ENDFORMAT: FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDFORMAT;
pub const FSVOLNOTIFY_FORMATERROR: FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_FORMATERROR;
pub const FSVOLNOTIFY_STARTCHECK: FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_STARTCHECK;
pub const FSVOLNOTIFY_ENDCHECK: FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_ENDCHECK;
pub const FSVOLNOTIFY_CHECKERROR: FSVOLNOTIFY = _FSVOLNOTIFY_FSVOLNOTIFY_CHECKERROR;
pub const ChangeSystemPartition: FSVOLNOTIFY = _FSVOLNOTIFY_ChangeSystemPartition;
// sdk/include/reactos/libs/fmifs/fmifs.h `enum CALLBACKCOMMAND` (fmtchk.c's FormatCallback):
pub const PROGRESS: CALLBACKCOMMAND = CALLBACKCOMMAND_PROGRESS;
pub const DONE: CALLBACKCOMMAND = CALLBACKCOMMAND_DONE;
// base/setup/lib/setuplib.h:188: #define ERROR_SYSTEM_PARTITION_NOT_FOUND (ERROR_LAST_ERROR_CODE + 1)
// (an int expression: enum constant + 1).
pub const ERROR_SYSTEM_PARTITION_NOT_FOUND: i32 = ERROR_LAST_ERROR_CODE as i32 + 1;
// ---- end hand-ported header items ----
