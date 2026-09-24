#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeDelayExecutionThread, KeWaitForSingleObject, ObCloseHandle, ObCreateObject, ObInsertObject, ObOpenObjectByName, ObReferenceObjectByHandle, PsGetCurrentProcessId, PsGetCurrentProcessWin32Process, PsGetCurrentThreadWin32Thread, PsGetProcessWin32WindowStation, PsSetProcessWindowStation, RtlAreAllAccessesGranted, RtlCreateAtomTable, RtlCreateUnicodeString, RtlDestroyAtomTable, RtlInitUnicodeString, ZwAllocateVirtualMemory, ZwClose, ZwCreateDirectoryObject, ZwDuplicateObject, ZwFreeVirtualMemory, ZwOpenDirectoryObject, ZwQueryDirectoryObject};
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::ob::{ObDereferenceObject, ObReferenceObject};
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_NO_MEMORY, STATUS_INSUFFICIENT_RESOURCES, STATUS_NOT_IMPLEMENTED};
use ntkrnl_sys::constants::{STATUS_ACCESS_DENIED, STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_HANDLE, STATUS_INVALID_PARAMETER, STATUS_NO_MORE_ENTRIES, STATUS_OBJECT_PATH_INVALID, STATUS_OBJECT_TYPE_MISMATCH, STATUS_UNSUCCESSFUL};
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


// Not present in the ntkrnl-sys constant tables — define locally.
const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5_u32 as i32;
const STATUS_NAME_TOO_LONG: NTSTATUS = 0xC0000106_u32 as i32;

// Cross-file win32k functions — decoration verified via llvm-nm on win32ss objs.
// cdecl (`_Name` / `__Name`):
extern "C" {
    fn EngpUpdateGraphicsDeviceList() -> NTSTATUS;
    fn MenuInit() -> u32;
    fn InitCursorImpl() -> u32;
    fn GetW32ProcessInfo() -> PPROCESSINFO;
    fn GetProcessLuid(Thread: *mut core::ffi::c_void, Process: PEPROCESS, Luid: *mut LUID) -> NTSTATUS;
    // MmCopyToCaller(x,y,z) is a macro for `_MmCopyToCaller((PCHAR)x,(PCHAR)y,(UINT)z)`
    // (mmcopy.h). The exported worker is `_MmCopyToCaller` (cdecl).
    #[link_name = "_MmCopyToCaller"]
    fn _MmCopyToCaller(Target: PVOID, Source: PVOID, Bytes: UINT) -> NTSTATUS;
    fn wcslen(s: *const u16) -> usize;
    fn wcschr(s: *const u16, c: u16) -> *mut u16;
    fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32;
    fn _scwprintf(format: *const u16, ...) -> i32;
    // ntoskrnl exports `_snwprintf` (cdecl); supports the %wZ UNICODE_STRING
    // conversion. The strsafe RtlStringCbPrintfW (FORCEINLINE in ntstrsafe.h)
    // sites are rewritten to forward to it.
    fn _snwprintf(buffer: *mut u16, count: usize, format: *const u16, ...) -> i32;
}

// stdcall (`_Name@N`) — extern "system" = stdcall on i386, cdecl elsewhere:
extern "system" {
    fn co_IntLoadDefaultCursors() -> u32;
    fn EngpFindGraphicsDevice(arg1: PUNICODE_STRING, arg2: ULONG) -> PGRAPHICS_DEVICE;
    fn InitMetrics() -> BOOL;
}

// fastcall (`@Name@N`) — extern "fastcall" on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn co_IntSetWndIcons() -> u32;
    fn co_IntShowDesktop(pdesk: PDESKTOP, cx: LONG, cy: LONG, fShow: u32) -> u32;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn co_IntSetWndIcons() -> u32;
    fn co_IntShowDesktop(pdesk: PDESKTOP, cx: LONG, cy: LONG, fShow: u32) -> u32;
}

// MmCopyToCaller(x,y,z) macro wrapper — forwards to the cdecl `_MmCopyToCaller`.
#[inline(always)]
unsafe fn MmCopyToCaller(dst: PVOID, src: *const core::ffi::c_void, len: usize) -> NTSTATUS {
    _MmCopyToCaller(dst, src as PVOID, len as UINT)
}

// NtCurrentTeb() / NtCurrentPeb() — FORCEINLINE intrinsics (no exported symbol).
// i386: TEB self-pointer at fs:[0x18]; amd64: gs:[0x30]. PEB = TEB->ProcessEnvironmentBlock.
#[inline(always)]
unsafe fn NtCurrentTeb() -> *mut TEB {
    let teb: *mut TEB;
    #[cfg(target_arch = "x86")]
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, pure, readonly));
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, pure, readonly));
    #[cfg(target_arch = "aarch64")]
    // Windows ARM64 carries the current TEB pointer in x18 (the platform
    // register); TPIDR_EL0 is not the kernel's source of truth. Matches
    // ntkrnl_sys::inline::ke::NtCurrentTeb. NtCurrentTeb is a FORCEINLINE
    // intrinsic with no exported symbol, so an extern call won't link.
    core::arch::asm!("mov {}, x18", out(reg) teb, options(nomem, nostack, preserves_flags));
    teb
}
#[inline(always)]
unsafe fn NtCurrentPeb() -> *mut PEB {
    (*NtCurrentTeb()).ProcessEnvironmentBlock as *mut PEB
}

// SafeProbeForRead/Write — NTAPI (extern "system" = stdcall i386 / cdecl else).
// Length = SIZE_T, Alignment = ULONG (canonical, per icm.rs/console.rs).
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// ExWindowStationObjectType / ExDesktopObjectType are ntoskrnl DATA exports
// (ntoskrnl.spec `@ extern`). Per-arch cross-DLL raw-dylib `__imp_` pattern
// (same shape as desktop.rs / console.rs). Accessor returns the typed POBJECT_TYPE.
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut ExWindowStationObjectType: *mut _OBJECT_TYPE;
    static mut ExDesktopObjectType: *mut _OBJECT_TYPE;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_ExWindowStationObjectType"]
    static mut ExWindowStationObjectType_imp: *mut *mut _OBJECT_TYPE;
    #[link_name = "__imp_ExDesktopObjectType"]
    static mut ExDesktopObjectType_imp: *mut *mut _OBJECT_TYPE;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn ex_windowstation_object_type() -> *mut _OBJECT_TYPE { ExWindowStationObjectType }
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ex_windowstation_object_type() -> *mut _OBJECT_TYPE { *ExWindowStationObjectType_imp }
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn ex_desktop_object_type() -> *mut _OBJECT_TYPE { ExDesktopObjectType }
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ex_desktop_object_type() -> *mut _OBJECT_TYPE { *ExDesktopObjectType_imp }

// ---- macro / static-inline reimplementations (no kernel symbol) ----
#[allow(unused_imports)] use {ERR as _ERR, FIXME as _FIXME};

// Pool tags (C multichar literal is big-endian; reverse for from_le_bytes).
// TAG_WINSTA = 'ATSW'; USERTAG_SECURITY = 'essU'.
const TAG_WINSTA: u32 = u32::from_le_bytes(*b"WSTA");
const USERTAG_SECURITY: u32 = u32::from_le_bytes(*b"Usse");

const OBJ_NAME_PATH_SEPARATOR: u16 = b'\\' as u16;

// L"%wZ\Service-0x%x-%x$" wide format literal (must be [u16] for WCHAR alignment).
static WINSTA_SVC_NAME_FMT: [u16; 21] = [
    b'%' as u16, b'w' as u16, b'Z' as u16, b'\\' as u16, b'S' as u16, b'e' as u16,
    b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'-' as u16,
    b'0' as u16, b'x' as u16, b'%' as u16, b'x' as u16, b'-' as u16, b'%' as u16,
    b'x' as u16, b'$' as u16, 0,
];

// ALIGN_UP(size, type) = ALIGN_UP_BY(size, sizeof(type)) rounding helper.
#[inline(always)]
fn ALIGN_UP(size: usize, align: usize) -> usize { (size + align - 1) & !(align - 1) }

// InitializeListHead FORCEINLINE.
#[inline(always)]
unsafe fn InitializeListHead(h: PLIST_ENTRY) { (*h).Flink = h; (*h).Blink = h; }

// SERVERINFO inherits PERUSERSERVERINFO inline (C `PERUSERSERVERINFO;` member).
// bindgen flattened that trailing embedded struct into `__bindgen_padding_0`, so
// the per-user fields (aiSysMet, ptCursor, Planes, ...) are accessed via a typed
// pointer at that offset.
#[inline(always)]
unsafe fn gpsi_pusi(psi: PSERVERINFO) -> *mut _PERUSERSERVERINFO {
    ((psi as usize) + core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0))
        as *mut _PERUSERSERVERINFO
}

// PROCESSINFO inherits W32PROCESS at offset 0 (C `W32PROCESS;` first member).
// bindgen flattened the W32PROCESS head into the leading `__bindgen_padding_0`, so
// peProcess (offset 0) / W32PF_flags are reached via the embedded-struct pointer.
#[inline(always)]
unsafe fn ppi_w32(ppi: PPROCESSINFO) -> *mut _W32PROCESS {
    ppi as *mut _W32PROCESS
}

// OBJECT_HEADER_TO_NAME_INFO(oh) — FORCEINLINE macro (this OBJECT_HEADER flavor
// uses a NameInfoOffset byte, not an InfoMask bitfield): when nonzero, the name
// info block precedes the header by NameInfoOffset bytes; otherwise NULL.
#[inline(always)]
unsafe fn OBJECT_HEADER_TO_NAME_INFO(oh: POBJECT_HEADER) -> POBJECT_HEADER_NAME_INFO {
    if (*oh).NameInfoOffset == 0 {
        return core::ptr::null_mut();
    }
    ((oh as usize) - (*oh).NameInfoOffset as usize) as POBJECT_HEADER_NAME_INFO
}

// RtlInitEmptyUnicodeString FORCEINLINE.
#[inline(always)]
unsafe fn RtlInitEmptyUnicodeString(dst: *mut _UNICODE_STRING, buf: *mut u16, len: u16) {
    (*dst).Length = 0;
    (*dst).MaximumLength = len;
    (*dst).Buffer = buf;
}

// snwprintf rc (count, or -1 on overflow) -> NTSTATUS (RtlStringCb* contract).
#[inline(always)]
fn snwprintf_status(rc: i32) -> NTSTATUS {
    if rc < 0 { STATUS_BUFFER_OVERFLOW } else { STATUS_SUCCESS }
}

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut InputWindowStation: PWINSTATION_OBJECT = core::ptr::null_mut();
#[no_mangle]
pub static mut hwndSAS: HWND = core::ptr::null_mut();
#[no_mangle]
pub static mut gbDesktopCursorsAndIconsLoaded: BOOLEAN = 0;
#[no_mangle]
pub static mut gustrWindowStationsDir: _UNICODE_STRING = unsafe { core::mem::zeroed() };

// bindgen lowered the C `#define WINSTA_OBJ_DIR L"..."` / `SESSION_DIR L"..."`
// wide-string defines to NARROW `&[u8]` consts (the L prefix is dropped). Casting
// their byte ptr to *const u16 reinterprets ASCII pairs as wide chars → garbage
// path → STATUS_OBJECT_PATH_SYNTAX_BAD when RtlCreateUnicodeString copies it.
// Build real UTF-16 twins and use these everywhere instead.
const fn ascii_to_u16<const N: usize>(s: &[u8; N]) -> [u16; N] {
    let mut out = [0u16; N];
    let mut i = 0;
    while i < N {
        out[i] = s[i] as u16;
        i += 1;
    }
    out
}
static WINSTA_OBJ_DIR_W: [u16; 24] = ascii_to_u16(b"\\Windows\\WindowStations\0");
static SESSION_DIR_W: [u16; 10] = ascii_to_u16(b"\\Sessions\0");

#[no_mangle]
pub unsafe extern "system" fn InitWindowStationImpl() -> NTSTATUS {
    let IntWindowStationMapping = _GENERIC_MAPPING {
        GenericRead: WINSTA_READ,
        GenericWrite: WINSTA_WRITE,
        GenericExecute: WINSTA_EXECUTE,
        GenericAll: WINSTA_ACCESS_ALL,
    };

    (*ex_windowstation_object_type()).TypeInfo.DefaultNonPagedPoolCharge =
        core::mem::size_of::<WINSTATION_OBJECT>() as u32;
    (*ex_windowstation_object_type()).TypeInfo.GenericMapping = IntWindowStationMapping;
    (*ex_windowstation_object_type()).TypeInfo.ValidAccessMask = WINSTA_ACCESS_ALL;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn UserCreateWinstaDirectory() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let Peb: PPEB;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hWinstaDir: HANDLE = core::ptr::null_mut();
    let mut wstrWindowStationsDir: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    Peb = NtCurrentPeb();
    if (*Peb).SessionId == 0 {
        if RtlCreateUnicodeString(
            core::ptr::addr_of_mut!(gustrWindowStationsDir) as _,
            WINSTA_OBJ_DIR_W.as_ptr(),
        ) == 0
        {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    } else {
        Status = snwprintf_status(_snwprintf(
            wstrWindowStationsDir.as_mut_ptr(),
            core::mem::size_of_val(&wstrWindowStationsDir) / core::mem::size_of::<u16>(),
            w!("%ws\\%lu%ws").as_ptr() as *const u16,
            SESSION_DIR_W.as_ptr(),
            (*Peb).SessionId,
            WINSTA_OBJ_DIR_W.as_ptr(),
        ));
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if RtlCreateUnicodeString(
            core::ptr::addr_of_mut!(gustrWindowStationsDir) as _,
            wstrWindowStationsDir.as_ptr(),
        ) == 0
        {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(gustrWindowStationsDir);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateDirectoryObject(
        &mut hWinstaDir,
        DIRECTORY_CREATE_OBJECT,
        &mut ObjectAttributes as *mut _ as _,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn IntWinStaObjectDelete(Parameters: PVOID) -> NTSTATUS {
    let DeleteParameters: PWIN32_DELETEMETHOD_PARAMETERS = Parameters as PWIN32_DELETEMETHOD_PARAMETERS;
    let WinSta: PWINSTATION_OBJECT = (*DeleteParameters).Object as PWINSTATION_OBJECT;

    if WinSta == InputWindowStation {
        InputWindowStation = core::ptr::null_mut();
    }

    (*WinSta).Flags |= WSS_DYING;

    UserEmptyClipboardData(WinSta);

    RtlDestroyAtomTable((*WinSta).AtomTable as _);

    UserAssignmentUnlock(core::ptr::addr_of_mut!((*WinSta).spklList) as *mut PVOID);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IntWinStaObjectParse(Parameters: PVOID) -> NTSTATUS {
    let ParseParameters: PWIN32_PARSEMETHOD_PARAMETERS = Parameters as PWIN32_PARSEMETHOD_PARAMETERS;
    let RemainingName: PUNICODE_STRING = (*ParseParameters).RemainingName;

    /* Assume we don't find anything */
    *(*ParseParameters).Object = core::ptr::null_mut();

    /* Check for an empty name */
    if (*RemainingName).Length == 0 {
        /* Make sure this is a window station, can't parse a desktop now */
        if (*ParseParameters).ObjectType != ex_windowstation_object_type() as PVOID {
            /* Fail */
            return STATUS_OBJECT_TYPE_MISMATCH;
        }

        /* Reference the window station and return */
        ObReferenceObject((*ParseParameters).ParseObject);
        *(*ParseParameters).Object = (*ParseParameters).ParseObject;
        return STATUS_SUCCESS;
    }

    /* Check for leading slash */
    if *(*RemainingName).Buffer == OBJ_NAME_PATH_SEPARATOR {
        /* Skip it */
        (*RemainingName).Buffer = (*RemainingName).Buffer.add(1);
        (*RemainingName).Length -= core::mem::size_of::<u16>() as u16;
        (*RemainingName).MaximumLength -= core::mem::size_of::<u16>() as u16;
    }

    /* Check if there is still a slash */
    if !wcschr((*RemainingName).Buffer, OBJ_NAME_PATH_SEPARATOR).is_null() {
        /* In this case, fail */
        return STATUS_OBJECT_PATH_INVALID;
    }

    /*
     * Check if we are parsing a desktop.
     */
    if (*ParseParameters).ObjectType == ex_desktop_object_type() as PVOID {
        /* Then call the desktop parse routine */
        return IntDesktopObjectParse(
            (*ParseParameters).ParseObject,
            (*ParseParameters).ObjectType,
            (*ParseParameters).AccessState,
            (*ParseParameters).AccessMode,
            (*ParseParameters).Attributes,
            (*ParseParameters).CompleteName,
            RemainingName,
            (*ParseParameters).Context,
            (*ParseParameters).SecurityQos,
            (*ParseParameters).Object,
        );
    }

    /* Should hopefully never get here */
    STATUS_OBJECT_TYPE_MISMATCH
}
#[no_mangle]
pub unsafe extern "system" fn IntWinStaOkToClose(Parameters: PVOID) -> NTSTATUS {
    let OkToCloseParameters = Parameters as PWIN32_OKAYTOCLOSEMETHOD_PARAMETERS;
    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

    if !ppi.is_null() && ((*OkToCloseParameters).Handle == (*ppi).hwinsta as *mut c_void) {
        return STATUS_ACCESS_DENIED;
    }

    STATUS_SUCCESS
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IntValidateWindowStationHandle(
    WindowStation: HWINSTA,
    AccessMode: KPROCESSOR_MODE,
    DesiredAccess: ACCESS_MASK,
    Object: *mut PWINSTATION_OBJECT,
    pObjectHandleInfo: POBJECT_HANDLE_INFORMATION,
) -> NTSTATUS {
    IntValidateWindowStationHandleImpl(WindowStation, AccessMode, DesiredAccess, Object, pObjectHandleInfo)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IntValidateWindowStationHandle(
    WindowStation: HWINSTA,
    AccessMode: KPROCESSOR_MODE,
    DesiredAccess: ACCESS_MASK,
    Object: *mut PWINSTATION_OBJECT,
    pObjectHandleInfo: POBJECT_HANDLE_INFORMATION,
) -> NTSTATUS {
    IntValidateWindowStationHandleImpl(WindowStation, AccessMode, DesiredAccess, Object, pObjectHandleInfo)
}

unsafe fn IntValidateWindowStationHandleImpl(
    WindowStation: HWINSTA,
    AccessMode: KPROCESSOR_MODE,
    DesiredAccess: ACCESS_MASK,
    Object: *mut PWINSTATION_OBJECT,
    pObjectHandleInfo: POBJECT_HANDLE_INFORMATION,
) -> NTSTATUS {

    let Status: NTSTATUS;

    if WindowStation.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return STATUS_INVALID_HANDLE;
    }

    Status = ObReferenceObjectByHandle(
        WindowStation as *mut core::ffi::c_void,
        DesiredAccess,
        ex_windowstation_object_type() as _,
        AccessMode,
        Object as *mut *mut core::ffi::c_void,
        pObjectHandleInfo as _,
    );

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
    }

    Status
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntDeferredCursorsAndIcons() {
    co_IntDeferredCursorsAndIcons_impl();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntDeferredCursorsAndIcons() {
    co_IntDeferredCursorsAndIcons_impl();
}

unsafe fn co_IntDeferredCursorsAndIcons_impl() {

    if gbDesktopCursorsAndIconsLoaded != 0 {
        return;
    }

    if PsGetCurrentThreadWin32Thread() == gptiDesktopThread as _ {
        return;
    }

    let Process: PEPROCESS = PsGetCurrentProcess() as PEPROCESS;
    if Process.is_null()
        || (*Process).Peb.is_null()
        || (*(*Process).Peb).KernelCallbackTable.is_null()
    {
        return;
    }

    co_IntLoadDefaultCursors();
    co_IntSetWndIcons();
    gbDesktopCursorsAndIconsLoaded = 1;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntInitializeDesktopGraphics() -> BOOL {
    co_IntInitializeDesktopGraphics_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntInitializeDesktopGraphics() -> BOOL {
    co_IntInitializeDesktopGraphics_impl()
}

unsafe fn co_IntInitializeDesktopGraphics_impl() -> BOOL {
    let lRet: LONG;
    let mut DriverName: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(core::ptr::addr_of_mut!(DriverName) as _, w!("DISPLAY").as_ptr() as *const u16);
    let pdesk: PDESKTOP;

    if !gpmdev.is_null() {
        /* PDEV already exists */
    } else {
        let mut lRetMut = PDEVOBJ_lChangeDisplaySettings(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(gpmdev),
            1,
        );
        if lRetMut != DISP_CHANGE_SUCCESSFUL as LONG && gbBaseVideo == 0 {
            gbBaseVideo = 1;
            EngpUpdateGraphicsDeviceList();
            lRetMut = PDEVOBJ_lChangeDisplaySettings(
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(gpmdev),
                1,
            );
            gbBaseVideo = 0;
            EngpUpdateGraphicsDeviceList();
        }
        if lRetMut != DISP_CHANGE_SUCCESSFUL as LONG {
            return 0;
        }
        lRet = lRetMut;
        let _ = lRet;
    }

    let mut tmw: TEXTMETRICW = core::mem::zeroed();

    ScreenDeviceContext = IntGdiCreateDC(
        &mut DriverName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    );
    if ScreenDeviceContext.is_null() {
        IntDestroyPrimarySurface();
        return 0;
    }
    GreSetDCOwner(ScreenDeviceContext, GDI_OBJ_HMGR_PUBLIC);

    if IntCreatePrimarySurface() == 0 {
        return 0;
    }

    hSystemBM = NtGdiCreateCompatibleDC(ScreenDeviceContext);

    NtGdiSelectFont(hSystemBM, NtGdiGetStockObject(SYSTEM_FONT as i32) as HFONT);
    GreSetDCOwner(hSystemBM, GDI_OBJ_HMGR_PUBLIC);

    InitMetrics();

    UserUpdateMonitorSize((*gpmdev).ppdevGlobal as HDEV);

    (*gpsi_pusi(gpsi)).aiSysMet[SM_CXSCREEN as usize] = (*(*gpmdev).ppdevGlobal).gdiinfo.ulHorzRes as LONG;
    (*gpsi_pusi(gpsi)).aiSysMet[SM_CYSCREEN as usize] = (*(*gpmdev).ppdevGlobal).gdiinfo.ulVertRes as LONG;
    (*gpsi_pusi(gpsi)).Planes = NtGdiGetDeviceCaps(ScreenDeviceContext, PLANES as i32) as _;
    (*gpsi_pusi(gpsi)).BitsPixel = NtGdiGetDeviceCaps(ScreenDeviceContext, BITSPIXEL as i32) as _;
    (*gpsi_pusi(gpsi)).BitCount =
        (*gpsi_pusi(gpsi)).Planes as USHORT * (*gpsi_pusi(gpsi)).BitsPixel as USHORT;
    (*gpsi_pusi(gpsi)).dmLogPixels = NtGdiGetDeviceCaps(ScreenDeviceContext, LOGPIXELSY as i32) as _;
    if NtGdiGetDeviceCaps(ScreenDeviceContext, RASTERCAPS as i32) as u32 & RC_PALETTE != 0 {
        (*gpsi_pusi(gpsi)).PUSIFlags |= PUSIF_PALETTEDISPLAY;
    } else {
        (*gpsi_pusi(gpsi)).PUSIFlags &= !PUSIF_PALETTEDISPLAY;
    }
    (*gpsi_pusi(gpsi)).cxSysFontChar = IntGetCharDimensions(
        hSystemBM,
        &mut tmw,
        core::ptr::addr_of_mut!((*gpsi_pusi(gpsi)).cySysFontChar) as *mut u32,
    ) as INT;
    (*gpsi_pusi(gpsi)).tmSysFont = tmw;

    (*gpsi_pusi(gpsi)).ptCursor.x = (*gpsi_pusi(gpsi)).aiSysMet[SM_CXSCREEN as usize] / 2;
    (*gpsi_pusi(gpsi)).ptCursor.y = (*gpsi_pusi(gpsi)).aiSysMet[SM_CYSCREEN as usize] / 2;

    UserAttachMonitor((*gpmdev).ppdevGlobal as HDEV);

    // Setup the cursor and icons here, as upstream does (winsta.c, unconditionally).
    // They were skipped from 2026-03 under a comment claiming the C compiled this
    // out under _USE_DUMMY_PSEH. It never did: the guard was ours. The skip was
    // blamed on "no user callback table on the desktop thread" and an i386
    // "Sending to Suspended Thread" hang. That predicate was a padded KAPC_STATE
    // copy (fixed 2026-09-23). The load needs a KeUserModeCallback, so it still
    // requires a callback table. Without one, co_IntDeferredCursorsAndIcons()
    // does it later on the first GUI thread.
    {
        extern "C" { fn DbgPrint(Format: *const core::ffi::c_char, ...) -> u32; }
        let Process: PEPROCESS = PsGetCurrentProcess() as PEPROCESS;
        if !Process.is_null()
            && !(*Process).Peb.is_null()
            && !(*(*Process).Peb).KernelCallbackTable.is_null()
        {
            co_IntLoadDefaultCursors();
            co_IntSetWndIcons();
            gbDesktopCursorsAndIconsLoaded = 1;
            DbgPrint(b"[WIN32K] cursors/icons loaded at desktop-graphics init\n\0".as_ptr() as *const _);
        } else {
            DbgPrint(b"[WIN32K] cursors/icons deferred (no callback table on this thread)\n\0".as_ptr() as *const _);
        }
    }

    MenuInit();

    pdesk = IntGetActiveDesktop();
    if !pdesk.is_null() {
        co_IntShowDesktop(
            pdesk,
            (*gpsi_pusi(gpsi)).aiSysMet[SM_CXSCREEN as usize],
            (*gpsi_pusi(gpsi)).aiSysMet[SM_CYSCREEN as usize],
            (1u8) as _,
        );
    }

    {
        let mut DriverName2: _UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString(core::ptr::addr_of_mut!(DriverName2) as _, w!("DISPLAY").as_ptr() as *const u16);
        let mut DisplayName: _UNICODE_STRING = core::mem::zeroed();
        let mut hdc: HDC;
        let mut iDevNum: ULONG = 1;
        loop {
            let pGraphicsDevice: PGRAPHICS_DEVICE =
                EngpFindGraphicsDevice(core::ptr::null_mut(), iDevNum);
            if pGraphicsDevice.is_null() {
                break;
            }
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(DisplayName) as _,
                core::ptr::addr_of_mut!((*pGraphicsDevice).szWinDeviceName) as *const u16,
            );
            hdc = IntGdiCreateDC(
                &mut DriverName2,
                &mut DisplayName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
            );
            IntPaintDesktop(hdc);
            iDevNum += 1;
        }
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntEndDesktopGraphics() {
    IntEndDesktopGraphics_impl();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEndDesktopGraphics() {
    IntEndDesktopGraphics_impl();
}

unsafe fn IntEndDesktopGraphics_impl() {
    if !ScreenDeviceContext.is_null() {
        GreSetDCOwner(ScreenDeviceContext, GDI_OBJ_HMGR_POWNED);
        GreDeleteObject(ScreenDeviceContext as HGDIOBJ);
        ScreenDeviceContext = core::ptr::null_mut();
    }
    IntHideDesktop(IntGetActiveDesktop());
    IntDestroyPrimarySurface();
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetScreenDC() -> HDC {
    ScreenDeviceContext
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetScreenDC() -> HDC {
    ScreenDeviceContext
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CheckWinstaAttributeAccess(DesiredAccess: ACCESS_MASK) -> BOOL {
    CheckWinstaAttributeAccess_impl(DesiredAccess)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CheckWinstaAttributeAccess(DesiredAccess: ACCESS_MASK) -> BOOL {
    CheckWinstaAttributeAccess_impl(DesiredAccess)
}

unsafe fn CheckWinstaAttributeAccess_impl(DesiredAccess: ACCESS_MASK) -> BOOL {
    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    if gpidLogon != PsGetCurrentProcessId() {
        if ((*ppi_w32(ppi)).W32PF_flags & W32PF_IOWINSTA) == 0 {
            EngSetLastError(ERROR_REQUIRES_INTERACTIVE_WINDOWSTATION as ULONG);
            return 0;
        }
        if RtlAreAllAccessesGranted((*ppi).amwinsta, DesiredAccess) == 0 {
            EngSetLastError(ERROR_ACCESS_DENIED as ULONG);
            return 0;
        }
    }
    1
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IntGetProcessWindowStation(
    phWinSta: *mut HWINSTA,
) -> PWINSTATION_OBJECT {
    IntGetProcessWindowStationImpl(phWinSta)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IntGetProcessWindowStation(
    phWinSta: *mut HWINSTA,
) -> PWINSTATION_OBJECT {
    IntGetProcessWindowStationImpl(phWinSta)
}

unsafe fn IntGetProcessWindowStationImpl(
    phWinSta: *mut HWINSTA,
) -> PWINSTATION_OBJECT {
    let mut pWinSta: PWINSTATION_OBJECT = core::ptr::null_mut();
    let ppi: PPROCESSINFO = GetW32ProcessInfo();
    let hWinSta: HWINSTA = (*ppi).hwinsta;
    if !phWinSta.is_null() {
        *phWinSta = hWinSta;
    }
    IntValidateWindowStationHandle(
        hWinSta,
        1i32 as KPROCESSOR_MODE,
        0,
        core::ptr::addr_of_mut!(pWinSta),
        core::ptr::null_mut(),
    );
    pWinSta
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCreateWindowStation(
    phWinSta: *mut HWINSTA,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    AccessMode: KPROCESSOR_MODE,
    OwnerMode: KPROCESSOR_MODE,
    dwDesiredAccess: ACCESS_MASK,
    Unknown2: u32,
    Unknown3: u32,
    Unknown4: u32,
    Unknown5: u32,
    Unknown6: u32,
) -> i32 {
    IntCreateWindowStation_impl(
        phWinSta,
        ObjectAttributes,
        AccessMode,
        OwnerMode,
        dwDesiredAccess,
        Unknown2,
        Unknown3,
        Unknown4,
        Unknown5,
        Unknown6,
    )
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCreateWindowStation(
    phWinSta: *mut HWINSTA,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    AccessMode: KPROCESSOR_MODE,
    OwnerMode: KPROCESSOR_MODE,
    dwDesiredAccess: ACCESS_MASK,
    Unknown2: u32,
    Unknown3: u32,
    Unknown4: u32,
    Unknown5: u32,
    Unknown6: u32,
) -> i32 {
    IntCreateWindowStation_impl(
        phWinSta,
        ObjectAttributes,
        AccessMode,
        OwnerMode,
        dwDesiredAccess,
        Unknown2,
        Unknown3,
        Unknown4,
        Unknown5,
        Unknown6,
    )
}

unsafe fn IntCreateWindowStation_impl(
    phWinSta: *mut HWINSTA,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    AccessMode: KPROCESSOR_MODE,
    OwnerMode: KPROCESSOR_MODE,
    dwDesiredAccess: ACCESS_MASK,
    _Unknown2: u32,
    _Unknown3: u32,
    _Unknown4: u32,
    _Unknown5: u32,
    _Unknown6: u32,
) -> i32 {
    let mut Status: i32;
    let mut hWinSta: HWINSTA = core::ptr::null_mut();
    let WindowStation: PWINSTATION_OBJECT;

    *phWinSta = core::ptr::null_mut();

    Status = ObOpenObjectByName(
        ObjectAttributes as _,
        ex_windowstation_object_type() as _,
        AccessMode,
        core::ptr::null_mut(),
        dwDesiredAccess,
        core::ptr::null_mut(),
        &mut hWinSta as *mut HWINSTA as *mut *mut core::ffi::c_void,
    );
    if NT_SUCCESS(Status) {
        *phWinSta = hWinSta;
        return Status;
    }

    let mut WindowStationPtr: *mut core::ffi::c_void = core::ptr::null_mut();
    Status = ObCreateObject(
        AccessMode,
        ex_windowstation_object_type() as _,
        ObjectAttributes as _,
        OwnerMode,
        core::ptr::null_mut(),
        core::mem::size_of::<WINSTATION_OBJECT>() as u32,
        0,
        0,
        &mut WindowStationPtr,
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return Status;
    }

    WindowStation = WindowStationPtr as PWINSTATION_OBJECT;

    RtlZeroMemory(
        WindowStation as *mut core::ffi::c_void,
        core::mem::size_of::<WINSTATION_OBJECT>(),
    );

    InitializeListHead(core::ptr::addr_of_mut!((*WindowStation).DesktopListHead));
    (*WindowStation).dwSessionId = (*NtCurrentPeb()).SessionId;
    Status = RtlCreateAtomTable(37, core::ptr::addr_of_mut!((*WindowStation).AtomTable) as _);
    if !NT_SUCCESS(Status) {
        ObDereferenceObject(WindowStation as *mut core::ffi::c_void);
        SetLastNtError(Status);
        return Status;
    }

    Status = ObInsertObject(
        WindowStation as *mut core::ffi::c_void,
        core::ptr::null_mut(),
        dwDesiredAccess,
        0,
        core::ptr::null_mut(),
        &mut hWinSta as *mut HWINSTA as *mut *mut core::ffi::c_void,
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return Status;
    }

    if InputWindowStation.is_null() {
        InputWindowStation = WindowStation;
        (*WindowStation).Flags &= !(WSS_NOIO as u32);

        InitCursorImpl();

        UserCreateSystemThread(ST_DESKTOP_THREAD);
        UserCreateSystemThread(ST_RIT);

        UserLeave(); // UserLeaveCo
        KeWaitForSingleObject(
            gpDesktopThreadStartedEvent as *mut c_void,
            _KWAIT_REASON_UserRequest as KWAIT_REASON,
            0i32 as i8, // C: KernelMode (not UserMode=1)
            0u8,
            core::ptr::null_mut(),
        );
        UserEnterExclusive(); // UserEnterCo
    } else {
        (*WindowStation).Flags |= WSS_NOIO as u32;
    }

    *phWinSta = hWinSta;
    EngSetLastError(ERROR_SUCCESS);

    STATUS_SUCCESS
}

unsafe fn FreeUserModeWindowStationName(
    mut WindowStationName: PUNICODE_STRING,
    TebStaticUnicodeString: PUNICODE_STRING,
    UserModeObjectAttributes: POBJECT_ATTRIBUTES,
    LocalObjectAttributes: POBJECT_ATTRIBUTES,
) {
    let mut MemSize: SIZE_T = 0;

    if !UserModeObjectAttributes.is_null() && !LocalObjectAttributes.is_null() {
        if NT_SUCCESS(SafeProbeForWrite(
            UserModeObjectAttributes as *mut _,
            core::mem::size_of::<_OBJECT_ATTRIBUTES>() as SIZE_T,
            core::mem::size_of::<ULONG>() as ULONG,
        )) {
            *UserModeObjectAttributes = *LocalObjectAttributes;
        }
    }

    if !WindowStationName.is_null() && WindowStationName != TebStaticUnicodeString {
        ZwFreeVirtualMemory(
            -1isize as HANDLE,
            core::ptr::addr_of_mut!(WindowStationName) as *mut PVOID,
            core::ptr::addr_of_mut!(MemSize),
            MEM_RELEASE,
        );
    }
}

unsafe fn BuildUserModeWindowStationName(
    UserModeObjectAttributes: POBJECT_ATTRIBUTES,
    LocalObjectAttributes: POBJECT_ATTRIBUTES,
    WindowStationName: *mut PUNICODE_STRING,
    TebStaticUnicodeString: *mut PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut MemSize: usize;

    let mut CallerLuid: LUID = core::mem::zeroed();
    let Teb: PTEB;
    let StrSize: u16;

    *WindowStationName = core::ptr::null_mut();
    *TebStaticUnicodeString = core::ptr::null_mut();

    /* Retrieve the current process LUID */
    Status = GetProcessLuid(core::ptr::null_mut(), core::ptr::null_mut(), &mut CallerLuid);
    if !NT_SUCCESS(Status) {
        ERR!("Failed to retrieve the caller LUID, Status 0x%08lx\n", Status);
        return Status;
    }

    /* Compute the needed string size */
    MemSize = _scwprintf(
        WINSTA_SVC_NAME_FMT.as_ptr(),
        core::ptr::addr_of_mut!(gustrWindowStationsDir),
        CallerLuid.HighPart,
        CallerLuid.LowPart,
    ) as usize;
    MemSize = MemSize * core::mem::size_of::<u16>() + core::mem::size_of::<u16>();
    if MemSize > MAXUSHORT as usize {
        ERR!("Window station name length is too long.\n");
        return STATUS_NAME_TOO_LONG;
    }
    StrSize = MemSize as u16;

    /*
     * Check whether it's short enough so that we can use the static buffer
     * in the TEB. Otherwise continue with virtual memory allocation.
     */
    Teb = NtCurrentTeb() as PTEB;
    if !Teb.is_null()
        && (StrSize as usize <= core::mem::size_of_val(&(*Teb).StaticUnicodeBuffer))
    {
        /* Remember the TEB's static unicode string address for later */
        *TebStaticUnicodeString = core::ptr::addr_of_mut!((*Teb).StaticUnicodeString);

        *WindowStationName = *TebStaticUnicodeString;
        (**WindowStationName).Length = 0;
    } else {
        /* The TEB's static unicode string is too small, allocate some user-mode virtual memory */
        MemSize += ALIGN_UP(
            core::mem::size_of::<_UNICODE_STRING>(),
            core::mem::size_of::<*mut core::ffi::c_void>(),
        );

        /* Allocate the memory in user-mode */
        Status = ZwAllocateVirtualMemory(
            (-1isize) as *mut core::ffi::c_void,
            WindowStationName as *mut *mut core::ffi::c_void,
            0,
            core::ptr::addr_of_mut!(MemSize) as *mut SIZE_T,
            MEM_COMMIT,
            PAGE_READWRITE,
        );
        if !NT_SUCCESS(Status) {
            ERR!("ZwAllocateVirtualMemory() failed, Status 0x%08lx\n", Status);
            return Status;
        }

        RtlInitEmptyUnicodeString(
            *WindowStationName,
            ((*WindowStationName as usize)
                + ALIGN_UP(
                    core::mem::size_of::<_UNICODE_STRING>(),
                    core::mem::size_of::<*mut core::ffi::c_void>(),
                )) as *mut u16,
            StrSize,
        );
    }

    Status = 'quit: {
        /* Build a valid window station name from the LUID */
        Status = snwprintf_status(_snwprintf(
            (**WindowStationName).Buffer,
            (**WindowStationName).MaximumLength as usize / core::mem::size_of::<u16>(),
            WINSTA_SVC_NAME_FMT.as_ptr(),
            core::ptr::addr_of_mut!(gustrWindowStationsDir),
            CallerLuid.HighPart,
            CallerLuid.LowPart,
        ));
        if !NT_SUCCESS(Status) {
            ERR!("Impossible to build a valid window station name, Status 0x%08lx\n", Status);
            break 'quit Status;
        }
        (**WindowStationName).Length =
            (wcslen((**WindowStationName).Buffer) * core::mem::size_of::<u16>()) as u16;

        /* Try to update the user's UserModeObjectAttributes */
        Status = SafeProbeForWrite(
            UserModeObjectAttributes as *mut core::ffi::c_void,
            core::mem::size_of::<_OBJECT_ATTRIBUTES>() as SIZE_T,
            core::mem::size_of::<u32>() as ULONG,
        );
        if NT_SUCCESS(Status) {
            *LocalObjectAttributes = *UserModeObjectAttributes;

            (*UserModeObjectAttributes).ObjectName = *WindowStationName;
            (*UserModeObjectAttributes).RootDirectory = core::ptr::null_mut();

            Status = STATUS_SUCCESS;
        }

        Status
    };

    if !NT_SUCCESS(Status) {
        /* Release the window station name */
        FreeUserModeWindowStationName(
            *WindowStationName,
            *TebStaticUnicodeString,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCreateWindowStation(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    dwDesiredAccess: ACCESS_MASK,
    Unknown2: u32,
    Unknown3: u32,
    Unknown4: u32,
    Unknown5: u32,
    Unknown6: u32,
) -> HWINSTA {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut hWinSta: HWINSTA = core::ptr::null_mut();
    let mut LocalObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut WindowStationName: PUNICODE_STRING = core::ptr::null_mut();
    let mut TebStaticUnicodeString: PUNICODE_STRING = core::ptr::null_mut();
    let mut OwnerMode: KPROCESSOR_MODE = 1i8;

    'CaptureDone: {
        /* Capture the object attributes and the window station name */
        Status = SafeProbeForRead(
            ObjectAttributes as *mut core::ffi::c_void,
            core::mem::size_of::<_OBJECT_ATTRIBUTES>() as SIZE_T,
            core::mem::size_of::<u32>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'CaptureDone;
        }
        LocalObjectAttributes = *ObjectAttributes;
        if LocalObjectAttributes.Length as usize != core::mem::size_of::<_OBJECT_ATTRIBUTES>() {
            Status = STATUS_INVALID_PARAMETER;
            break 'CaptureDone;
        }

        if LocalObjectAttributes.ObjectName.is_null()
            || (*LocalObjectAttributes.ObjectName).Buffer.is_null()
            || (*LocalObjectAttributes.ObjectName).Length == 0
            || *(*LocalObjectAttributes.ObjectName).Buffer == 0
        {
            Status = BuildUserModeWindowStationName(
                ObjectAttributes,
                &mut LocalObjectAttributes,
                &mut WindowStationName,
                &mut TebStaticUnicodeString,
            );
            if !NT_SUCCESS(Status) {
                break 'CaptureDone;
            }
            OwnerMode = 0i8;
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    UserEnterExclusive();

    /* Create the window station */
    Status = IntCreateWindowStation(
        &mut hWinSta,
        ObjectAttributes,
        1i8,
        OwnerMode,
        dwDesiredAccess,
        Unknown2,
        Unknown3,
        Unknown4,
        Unknown5,
        Unknown6,
    );
    UserLeave();

    /* Try to restore the user's ObjectAttributes and release the window station name */
    FreeUserModeWindowStationName(
        WindowStationName,
        TebStaticUnicodeString,
        if OwnerMode == 0i8 {
            ObjectAttributes
        } else {
            core::ptr::null_mut()
        },
        &mut LocalObjectAttributes,
    );

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
    }

    hWinSta
}
#[no_mangle]
pub unsafe extern "system" fn NtUserOpenWindowStation(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    dwDesiredAccess: ACCESS_MASK,
) -> HWINSTA {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut hWinSta: HWINSTA = core::ptr::null_mut();
    let mut LocalObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut WindowStationName: PUNICODE_STRING = core::ptr::null_mut();
    let mut TebStaticUnicodeString: PUNICODE_STRING = core::ptr::null_mut();
    let mut OwnerMode: KPROCESSOR_MODE = 1i32 as KPROCESSOR_MODE;

    'CaptureDone: {
        Status = SafeProbeForRead(
            ObjectAttributes as *mut core::ffi::c_void,
            core::mem::size_of::<_OBJECT_ATTRIBUTES>() as SIZE_T,
            core::mem::size_of::<u32>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'CaptureDone;
        }

        LocalObjectAttributes = *ObjectAttributes;
        if LocalObjectAttributes.Length as usize != core::mem::size_of::<_OBJECT_ATTRIBUTES>() {
            Status = STATUS_INVALID_PARAMETER;
            break 'CaptureDone;
        }

        if LocalObjectAttributes.ObjectName.is_null()
            || (*LocalObjectAttributes.ObjectName).Buffer.is_null()
            || (*LocalObjectAttributes.ObjectName).Length == 0
            || *(*LocalObjectAttributes.ObjectName).Buffer == 0
        {
            LocalObjectAttributes.ObjectName = core::ptr::null_mut();
        }

        const SERVICE_NAME: &[u16; 29] = &[
            b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
            b'e' as u16, b'-' as u16, b'0' as u16, b'x' as u16, b'0' as u16, b'0' as u16,
            b'0' as u16, b'0' as u16, b'0' as u16, b'0' as u16, b'0' as u16, b'0' as u16,
            b'-' as u16, b'0' as u16, b'0' as u16, b'0' as u16, b'0' as u16, b'0' as u16,
            b'0' as u16, b'0' as u16, b'0' as u16, b'$' as u16, 0u16,
        ];
        let service_len = (core::mem::size_of_val(SERVICE_NAME) - core::mem::size_of::<u16>()) as u16;

        if !LocalObjectAttributes.ObjectName.is_null()
            && (*LocalObjectAttributes.ObjectName).Length == service_len
            && _wcsnicmp(
                (*LocalObjectAttributes.ObjectName).Buffer,
                SERVICE_NAME.as_ptr(),
                ((*LocalObjectAttributes.ObjectName).Length as usize)
                    / core::mem::size_of::<u16>(),
            ) == 0
        {
            LocalObjectAttributes.ObjectName = core::ptr::null_mut();
        }

        if LocalObjectAttributes.ObjectName.is_null() {
            Status = BuildUserModeWindowStationName(
                ObjectAttributes,
                &mut LocalObjectAttributes,
                &mut WindowStationName,
                &mut TebStaticUnicodeString,
            );
            if !NT_SUCCESS(Status) {
                break 'CaptureDone;
            }
            OwnerMode = 0i32 as KPROCESSOR_MODE;
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    Status = ObOpenObjectByName(
        ObjectAttributes as _,
        ex_windowstation_object_type() as _,
        1i32 as KPROCESSOR_MODE,
        core::ptr::null_mut(),
        dwDesiredAccess,
        core::ptr::null_mut(),
        &mut hWinSta as *mut HWINSTA as *mut *mut core::ffi::c_void,
    );

    FreeUserModeWindowStationName(
        WindowStationName,
        TebStaticUnicodeString,
        if OwnerMode == 0i32 as KPROCESSOR_MODE {
            ObjectAttributes
        } else {
            core::ptr::null_mut()
        },
        &mut LocalObjectAttributes,
    );

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
    }

    hWinSta
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCloseWindowStation(hWinSta: HWINSTA) -> BOOL {
    let mut Object: PWINSTATION_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if hWinSta == UserGetProcessWindowStation() {
        return 0;
    }

    Status = IntValidateWindowStationHandle(
        hWinSta,
        1i32 as KPROCESSOR_MODE,
        0,
        &mut Object,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return 0;
    }

    ObDereferenceObject(Object as *mut _);

    Status = ObCloseHandle(hWinSta as HANDLE, 1i32 as KPROCESSOR_MODE);
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }

    1
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetObjectInformation(
    hObject: HANDLE,
    nIndex: DWORD,
    pvInformation: PVOID,
    nLength: DWORD,
    nLengthNeeded: PDWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut WinStaObject: PWINSTATION_OBJECT = core::ptr::null_mut();
    let mut DesktopObject: PDESKTOP = core::ptr::null_mut();
    let mut ObjectHeader: POBJECT_HEADER;
    let NameInfo: POBJECT_HEADER_NAME_INFO;
    let mut HandleInfo: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let mut ObjectFlags: USEROBJECTFLAGS = core::mem::zeroed();
    let mut pStrNameU: PUNICODE_STRING = core::ptr::null_mut();
    let mut pvData: PVOID = core::ptr::null_mut();
    let mut nDataSize: SIZE_T = 0;

    if !nLengthNeeded.is_null() {
        Status = SafeProbeForWrite(
            nLengthNeeded as PVOID,
            core::mem::size_of::<DWORD>() as SIZE_T,
            1,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            return 0;
        }
    }
    Status = SafeProbeForWrite(pvInformation, nLength as SIZE_T, 1);
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }

    Status = ObReferenceObjectByHandle(
        hObject,
        0,
        ex_windowstation_object_type() as _,
        1i32 as i8,
        &mut WinStaObject as *mut _ as *mut PVOID,
        &mut HandleInfo as *mut _ as _,
    );

    if Status == STATUS_OBJECT_TYPE_MISMATCH {
        WinStaObject = core::ptr::null_mut();
        Status = IntValidateDesktopHandle(hObject as HDESK, 1i32 as i8, 0, &mut DesktopObject);
    }

    'exit: {
        if !NT_SUCCESS(Status) {
            break 'exit;
        }

        match nIndex {
            x if x == UOI_FLAGS => {
                ObjectFlags.fReserved = 0;
                ObjectFlags.fInherit =
                    if (HandleInfo.HandleAttributes & OBJ_INHERIT as u32) != 0 { 1 } else { 0 };

                ObjectFlags.dwFlags = 0;
                if !WinStaObject.is_null() {
                    if ((*WinStaObject).Flags & WSS_NOIO as u32) == 0 {
                        ObjectFlags.dwFlags |= WSF_VISIBLE as u32;
                    }
                } else if !DesktopObject.is_null() {
                    FIXME!(b"Setting DF_ALLOWOTHERACCOUNTHOOK is unimplemented.\n\0".as_ptr());
                } else {
                    ERR!("No associated WinStaObject nor DesktopObject!\n");
                }

                pvData = &mut ObjectFlags as *mut _ as PVOID;
                nDataSize = core::mem::size_of::<USEROBJECTFLAGS>() as SIZE_T;
                Status = STATUS_SUCCESS;
            }

            x if x == UOI_NAME => {
                if !WinStaObject.is_null() {
                    ObjectHeader = ((WinStaObject as usize)
                        - core::mem::offset_of!(_OBJECT_HEADER, Body))
                        as POBJECT_HEADER;
                    NameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);

                    if !NameInfo.is_null() && (*NameInfo).Name.Length > 0 {
                        pStrNameU = core::ptr::addr_of_mut!((*NameInfo).Name);
                        nDataSize =
                            (*pStrNameU).Length as SIZE_T + core::mem::size_of::<u16>() as SIZE_T;
                    } else {
                        pStrNameU = core::ptr::null_mut();
                        nDataSize = core::mem::size_of::<u16>() as SIZE_T;
                    }
                    Status = STATUS_SUCCESS;
                } else if !DesktopObject.is_null() {
                    pvData = (*(*DesktopObject).pDeskInfo).szDesktopName.as_mut_ptr() as PVOID;
                    nDataSize = ((wcslen((*(*DesktopObject).pDeskInfo).szDesktopName.as_ptr()) + 1)
                        * core::mem::size_of::<u16>()) as SIZE_T;
                    Status = STATUS_SUCCESS;
                } else {
                    Status = STATUS_INVALID_PARAMETER;
                }
            }

            x if x == UOI_TYPE => {
                if !WinStaObject.is_null() {
                    ObjectHeader = ((WinStaObject as usize)
                        - core::mem::offset_of!(_OBJECT_HEADER, Body))
                        as POBJECT_HEADER;
                    pStrNameU = core::ptr::addr_of_mut!((*(*ObjectHeader).Type).Name);
                    nDataSize =
                        (*pStrNameU).Length as SIZE_T + core::mem::size_of::<u16>() as SIZE_T;
                    Status = STATUS_SUCCESS;
                } else if !DesktopObject.is_null() {
                    ObjectHeader = ((DesktopObject as usize)
                        - core::mem::offset_of!(_OBJECT_HEADER, Body))
                        as POBJECT_HEADER;
                    pStrNameU = core::ptr::addr_of_mut!((*(*ObjectHeader).Type).Name);
                    nDataSize =
                        (*pStrNameU).Length as SIZE_T + core::mem::size_of::<u16>() as SIZE_T;
                    Status = STATUS_SUCCESS;
                } else {
                    Status = STATUS_INVALID_PARAMETER;
                }
            }

            x if x == UOI_USER_SID => {
                Status = STATUS_NOT_IMPLEMENTED;
                ERR!("UOI_USER_SID unimplemented!\n");
            }

            _ => {
                Status = STATUS_INVALID_PARAMETER;
            }
        }
    }

    if Status == STATUS_SUCCESS && (nLength as SIZE_T) < nDataSize {
        Status = STATUS_BUFFER_TOO_SMALL;
    }

    if !nLengthNeeded.is_null() {
        core::ptr::write_volatile(nLengthNeeded, nDataSize as DWORD);
    }

    if Status == STATUS_SUCCESS && nDataSize > 0 {
        if !pvData.is_null() {
            RtlCopyMemory(pvInformation, pvData, nDataSize as _);
        } else if !pStrNameU.is_null() {
            RtlCopyMemory(
                pvInformation,
                (*pStrNameU).Buffer as PVOID,
                (*pStrNameU).Length as _,
            );
            *(pvInformation as PWCHAR)
                .add((*pStrNameU).Length as usize / core::mem::size_of::<u16>()) = 0;
        } else {
            RtlZeroMemory(pvInformation, nDataSize as _);
        }
    }

    if !DesktopObject.is_null() {
        ObDereferenceObject(DesktopObject as PVOID);
    }
    if !WinStaObject.is_null() {
        ObDereferenceObject(WinStaObject as PVOID);
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }

    1
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetObjectInformation(
    hObject: HANDLE,
    nIndex: DWORD,
    pvInformation: PVOID,
    nLength: DWORD,
) -> BOOL {
    SetLastNtError(STATUS_UNSUCCESSFUL);
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetProcessWindowStation() -> HWINSTA {
    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    (*ppi).hwinsta
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetProcessWindowStation() -> HWINSTA {
    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    (*ppi).hwinsta
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetProcessWindowStation() -> HWINSTA {
    UserGetProcessWindowStation()
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn UserSetProcessWindowStation(hWindowStation: HWINSTA) -> BOOL {
    UserSetProcessWindowStationImpl(hWindowStation)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn UserSetProcessWindowStation(hWindowStation: HWINSTA) -> BOOL {
    UserSetProcessWindowStationImpl(hWindowStation)
}

unsafe fn UserSetProcessWindowStationImpl(hWindowStation: HWINSTA) -> BOOL {

    let Status: NTSTATUS;
    let ppi: PPROCESSINFO;
    let mut ObjectHandleInfo: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let mut NewWinSta: PWINSTATION_OBJECT = core::ptr::null_mut();
    let mut OldWinSta: PWINSTATION_OBJECT;
    let mut hCacheWinSta: HWINSTA;

    ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

    /* Reference the new window station */
    if hWindowStation != core::ptr::null_mut() {
        Status = IntValidateWindowStationHandle(
            hWindowStation,
            1i32 as KPROCESSOR_MODE,
            0,
            core::ptr::addr_of_mut!(NewWinSta),
            core::ptr::addr_of_mut!(ObjectHandleInfo),
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            return 0;
        }
    }

    OldWinSta = (*ppi).prpwinsta;
    hCacheWinSta = PsGetProcessWin32WindowStation((*ppi_w32(ppi)).peProcess as _) as HWINSTA;

    /* Dereference the previous window station */
    if OldWinSta != core::ptr::null_mut() {
        ObDereferenceObject(OldWinSta as *mut _);
    }

    /* Close the cached _EPROCESS window station handle if needed */
    if hCacheWinSta != core::ptr::null_mut() {
        let RefStatus = ObReferenceObjectByHandle(
            hCacheWinSta as *mut _,
            0,
            ex_windowstation_object_type() as _,
            1i32 as i8,
            core::ptr::addr_of_mut!(OldWinSta) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(RefStatus) {
            hCacheWinSta = core::ptr::null_mut();
            PsSetProcessWindowStation((*ppi_w32(ppi)).peProcess as _, hCacheWinSta as *mut _);
        } else {
            if NewWinSta != OldWinSta {
                ObCloseHandle(hCacheWinSta as *mut _, 1i32 as i8);
                hCacheWinSta = core::ptr::null_mut();
                PsSetProcessWindowStation((*ppi_w32(ppi)).peProcess as _, hCacheWinSta as *mut _);
            }

            ObDereferenceObject(OldWinSta as *mut _);
        }
    }

    /* Duplicate and save a new cached _EPROCESS window station handle */
    if hCacheWinSta == core::ptr::null_mut() && hWindowStation != core::ptr::null_mut() {
        let DupStatus = ZwDuplicateObject(
            -1isize as HANDLE,
            hWindowStation as *mut _,
            -1isize as HANDLE,
            core::ptr::addr_of_mut!(hCacheWinSta) as PHANDLE,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        );
        if NT_SUCCESS(DupStatus) {
            PsSetProcessWindowStation((*ppi_w32(ppi)).peProcess as _, hCacheWinSta as *mut _);
        }
    }

    (*ppi).prpwinsta = NewWinSta;
    (*ppi).hwinsta = hWindowStation;
    (*ppi).amwinsta = if hWindowStation != core::ptr::null_mut() {
        ObjectHandleInfo.GrantedAccess
    } else {
        0
    };

    if RtlAreAllAccessesGranted((*ppi).amwinsta, WINSTA_READSCREEN) != 0 {
        (*ppi_w32(ppi)).W32PF_flags |= W32PF_READSCREENACCESSGRANTED;
    } else {
        (*ppi_w32(ppi)).W32PF_flags &= !W32PF_READSCREENACCESSGRANTED;
    }

    if !NewWinSta.is_null() && ((*NewWinSta).Flags & WSS_NOIO) == 0 {
        (*ppi_w32(ppi)).W32PF_flags |= W32PF_IOWINSTA;
    } else {
        (*ppi_w32(ppi)).W32PF_flags &= !W32PF_IOWINSTA;
    }

    1
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetProcessWindowStation(hWindowStation: HWINSTA) -> BOOL {
    let ret: BOOL;

    UserEnterExclusive();

    ret = UserSetProcessWindowStation(hWindowStation);

    UserLeave();

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserLockWindowStation(hWindowStation: HWINSTA) -> BOOL {
    let mut Object: PWINSTATION_OBJECT = core::ptr::null_mut();
    let Status: NTSTATUS;

    if gpidLogon != PsGetCurrentProcessId() {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return 0;
    }

    Status = IntValidateWindowStationHandle(
        hWindowStation,
        1i32 as KPROCESSOR_MODE,
        0,
        &mut Object,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }

    (*Object).Flags |= WSS_LOCKED;

    ObDereferenceObject(Object as *mut core::ffi::c_void);
    1
}
#[no_mangle]
pub unsafe extern "system" fn NtUserUnlockWindowStation(hWindowStation: HWINSTA) -> BOOL {
    let mut Object: PWINSTATION_OBJECT = core::ptr::null_mut();
    let Status: NTSTATUS;
    let Ret: BOOL;

    if gpidLogon != PsGetCurrentProcessId() {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return 0;
    }

    Status = IntValidateWindowStationHandle(
        hWindowStation,
        1i32 as KPROCESSOR_MODE,
        0,
        &mut Object,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }

    Ret = (((*Object).Flags & WSS_LOCKED) == WSS_LOCKED) as BOOL;
    (*Object).Flags &= !WSS_LOCKED;

    ObDereferenceObject(Object as *mut _);
    Ret
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn BuildWindowStationNameList(
    dwSize: ULONG,
    lpBuffer: PVOID,
    pRequiredSize: PULONG,
) -> NTSTATUS {
    BuildWindowStationNameList_impl(dwSize, lpBuffer, pRequiredSize)
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn BuildWindowStationNameList(
    dwSize: ULONG,
    lpBuffer: PVOID,
    pRequiredSize: PULONG,
) -> NTSTATUS {
    BuildWindowStationNameList_impl(dwSize, lpBuffer, pRequiredSize)
}

unsafe fn BuildWindowStationNameList_impl(
    dwSize: ULONG,
    mut lpBuffer: PVOID,
    pRequiredSize: PULONG,
) -> NTSTATUS {

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut DirectoryHandle: HANDLE = core::ptr::null_mut();
    let mut InitialBuffer: [core::ffi::c_char; 256] = [0; 256];
    let mut Buffer: *mut core::ffi::c_char;
    let mut Context: ULONG;
    let mut ReturnLength: ULONG = 0;
    let mut BufferSize: ULONG;
    let mut EntryCount: u32;
    let mut DirEntry: POBJECT_DIRECTORY_INFORMATION;
    let NullWchar: u16;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(gustrWindowStationsDir);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenDirectoryObject(
        &mut DirectoryHandle,
        DIRECTORY_QUERY,
        &mut ObjectAttributes as *mut _ as _,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    Context = 0;
    Buffer = core::ptr::null_mut();
    Status = ZwQueryDirectoryObject(
        DirectoryHandle,
        InitialBuffer.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&InitialBuffer) as ULONG,
        0u8,
        1u8,
        &mut Context,
        &mut ReturnLength,
    );
    if NT_SUCCESS(Status) {
        if STATUS_NO_MORE_ENTRIES
            == ZwQueryDirectoryObject(
                DirectoryHandle,
                core::ptr::null_mut(),
                0,
                0u8,
                0u8,
                &mut Context,
                core::ptr::null_mut(),
            )
        {
            Buffer = InitialBuffer.as_mut_ptr();
        }
    }

    if Buffer.is_null() {
        Status = ZwQueryDirectoryObject(
            DirectoryHandle,
            core::ptr::null_mut(),
            0,
            0u8,
            1u8,
            &mut Context,
            &mut ReturnLength,
        );
        if !NT_SUCCESS(Status) {
            ZwClose(DirectoryHandle);
            return Status;
        }

        BufferSize = ReturnLength;
        Buffer = ExAllocatePoolWithTag(PagedPool, BufferSize as _, TAG_WINSTA) as *mut core::ffi::c_char;
        if Buffer.is_null() {
            ZwClose(DirectoryHandle);
            return STATUS_NO_MEMORY;
        }

        Context = 0;
        Status = ZwQueryDirectoryObject(
            DirectoryHandle,
            Buffer as PVOID,
            BufferSize,
            0u8,
            1u8,
            &mut Context,
            &mut ReturnLength,
        );
        if !NT_SUCCESS(Status)
            || STATUS_NO_MORE_ENTRIES
                != ZwQueryDirectoryObject(
                    DirectoryHandle,
                    core::ptr::null_mut(),
                    0,
                    0u8,
                    0u8,
                    &mut Context,
                    core::ptr::null_mut(),
                )
        {
            ExFreePoolWithTag(Buffer as PVOID, TAG_WINSTA);
            ZwClose(DirectoryHandle);
            return if NT_SUCCESS(Status) { STATUS_INTERNAL_ERROR } else { Status };
        }
    }

    ZwClose(DirectoryHandle);

    ReturnLength = core::mem::size_of::<u32>() as ULONG;
    EntryCount = 0;
    DirEntry = Buffer as POBJECT_DIRECTORY_INFORMATION;
    while 0 != (*DirEntry).Name.Length {
        ReturnLength += (*DirEntry).Name.Length as ULONG + core::mem::size_of::<u16>() as ULONG;
        EntryCount += 1;
        DirEntry = DirEntry.add(1);
    }

    if !pRequiredSize.is_null() {
        Status = MmCopyToCaller(
            pRequiredSize as PVOID,
            &ReturnLength as *const ULONG as *const core::ffi::c_void,
            core::mem::size_of::<ULONG>(),
        );
        if !NT_SUCCESS(Status) {
            if Buffer != InitialBuffer.as_mut_ptr() {
                ExFreePoolWithTag(Buffer as PVOID, TAG_WINSTA);
            }
            return STATUS_BUFFER_TOO_SMALL;
        }
    }

    if dwSize < ReturnLength {
        if Buffer != InitialBuffer.as_mut_ptr() {
            ExFreePoolWithTag(Buffer as PVOID, TAG_WINSTA);
        }
        return STATUS_BUFFER_TOO_SMALL;
    }

    Status = MmCopyToCaller(
        lpBuffer,
        &EntryCount as *const u32 as *const core::ffi::c_void,
        core::mem::size_of::<u32>(),
    );
    if !NT_SUCCESS(Status) {
        if Buffer != InitialBuffer.as_mut_ptr() {
            ExFreePoolWithTag(Buffer as PVOID, TAG_WINSTA);
        }
        return Status;
    }
    lpBuffer = (lpBuffer as *mut core::ffi::c_char).add(core::mem::size_of::<u32>()) as PVOID;

    NullWchar = 0u16;
    DirEntry = Buffer as POBJECT_DIRECTORY_INFORMATION;
    while 0 != (*DirEntry).Name.Length {
        Status = MmCopyToCaller(
            lpBuffer,
            (*DirEntry).Name.Buffer as *const core::ffi::c_void,
            (*DirEntry).Name.Length as usize,
        );
        if !NT_SUCCESS(Status) {
            if Buffer != InitialBuffer.as_mut_ptr() {
                ExFreePoolWithTag(Buffer as PVOID, TAG_WINSTA);
            }
            return Status;
        }
        lpBuffer = (lpBuffer as *mut core::ffi::c_char).add((*DirEntry).Name.Length as usize) as PVOID;
        Status = MmCopyToCaller(
            lpBuffer,
            &NullWchar as *const u16 as *const core::ffi::c_void,
            core::mem::size_of::<u16>(),
        );
        if !NT_SUCCESS(Status) {
            if Buffer != InitialBuffer.as_mut_ptr() {
                ExFreePoolWithTag(Buffer as PVOID, TAG_WINSTA);
            }
            return Status;
        }
        lpBuffer = (lpBuffer as *mut core::ffi::c_char).add(core::mem::size_of::<u16>()) as PVOID;
        DirEntry = DirEntry.add(1);
    }

    if Buffer != InitialBuffer.as_mut_ptr() {
        ExFreePoolWithTag(Buffer as PVOID, TAG_WINSTA);
    }

    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn BuildDesktopNameList(
    hWindowStation: HWINSTA,
    dwSize: ULONG,
    lpBuffer: PVOID,
    pRequiredSize: PULONG,
) -> NTSTATUS {
    BuildDesktopNameListImpl(hWindowStation, dwSize, lpBuffer, pRequiredSize)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn BuildDesktopNameList(
    hWindowStation: HWINSTA,
    dwSize: ULONG,
    lpBuffer: PVOID,
    pRequiredSize: PULONG,
) -> NTSTATUS {
    BuildDesktopNameListImpl(hWindowStation, dwSize, lpBuffer, pRequiredSize)
}

unsafe fn BuildDesktopNameListImpl(
    hWindowStation: HWINSTA,
    dwSize: ULONG,
    mut lpBuffer: PVOID,
    pRequiredSize: PULONG,
) -> NTSTATUS {

    let mut Status: NTSTATUS;
    let mut WindowStation: PWINSTATION_OBJECT = core::ptr::null_mut();
    let mut DesktopEntry: PLIST_ENTRY;
    let mut DesktopObject: PDESKTOP;
    let mut EntryCount: u32;
    let mut ReturnLength: ULONG;
    let mut NullWchar: u16;
    let mut DesktopName: _UNICODE_STRING = core::mem::zeroed();

    Status = IntValidateWindowStationHandle(
        hWindowStation,
        1i32 as i8,
        0,
        &mut WindowStation,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ReturnLength = core::mem::size_of::<u32>() as ULONG;
    EntryCount = 0;
    DesktopEntry = (*WindowStation).DesktopListHead.Flink;
    while DesktopEntry != core::ptr::addr_of_mut!((*WindowStation).DesktopListHead) {
        DesktopObject = (DesktopEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_DESKTOP, ListEntry) as isize))
            as PDESKTOP;
        RtlInitUnicodeString(core::ptr::addr_of_mut!(DesktopName) as _, (*(*DesktopObject).pDeskInfo).szDesktopName.as_ptr());
        ReturnLength += DesktopName.Length as ULONG + core::mem::size_of::<u16>() as ULONG;
        EntryCount += 1;
        DesktopEntry = (*DesktopEntry).Flink;
    }

    if !pRequiredSize.is_null() {
        Status = MmCopyToCaller(
            pRequiredSize as PVOID,
            core::ptr::addr_of_mut!(ReturnLength) as PVOID,
            core::mem::size_of::<ULONG>(),
        );
        if !NT_SUCCESS(Status) {
            ObDereferenceObject(WindowStation as PVOID);
            return STATUS_BUFFER_TOO_SMALL;
        }
    }

    if dwSize < ReturnLength {
        ObDereferenceObject(WindowStation as PVOID);
        return STATUS_BUFFER_TOO_SMALL;
    }

    Status = MmCopyToCaller(
        lpBuffer,
        core::ptr::addr_of_mut!(EntryCount) as PVOID,
        core::mem::size_of::<u32>(),
    );
    if !NT_SUCCESS(Status) {
        ObDereferenceObject(WindowStation as PVOID);
        return Status;
    }
    lpBuffer = (lpBuffer as *mut u8).add(core::mem::size_of::<u32>()) as PVOID;

    NullWchar = 0;
    DesktopEntry = (*WindowStation).DesktopListHead.Flink;
    while DesktopEntry != core::ptr::addr_of_mut!((*WindowStation).DesktopListHead) {
        DesktopObject = (DesktopEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_DESKTOP, ListEntry) as isize))
            as PDESKTOP;
        RtlInitUnicodeString(core::ptr::addr_of_mut!(DesktopName) as _, (*(*DesktopObject).pDeskInfo).szDesktopName.as_ptr());
        Status = MmCopyToCaller(lpBuffer, DesktopName.Buffer as PVOID, DesktopName.Length as usize);
        if !NT_SUCCESS(Status) {
            ObDereferenceObject(WindowStation as PVOID);
            return Status;
        }
        lpBuffer = (lpBuffer as *mut u8).add(DesktopName.Length as usize) as PVOID;
        Status = MmCopyToCaller(
            lpBuffer,
            core::ptr::addr_of_mut!(NullWchar) as PVOID,
            core::mem::size_of::<u16>(),
        );
        if !NT_SUCCESS(Status) {
            ObDereferenceObject(WindowStation as PVOID);
            return Status;
        }
        lpBuffer = (lpBuffer as *mut u8).add(core::mem::size_of::<u16>()) as PVOID;
        DesktopEntry = (*DesktopEntry).Flink;
    }

    ObDereferenceObject(WindowStation as PVOID);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn NtUserBuildNameList(
    hWindowStation: HWINSTA,
    dwSize: ULONG,
    lpBuffer: PVOID,
    pRequiredSize: PULONG,
) -> NTSTATUS {
    if hWindowStation.is_null() {
        BuildWindowStationNameList(dwSize, lpBuffer, pRequiredSize)
    } else {
        BuildDesktopNameList(hWindowStation, dwSize, lpBuffer, pRequiredSize)
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetLogonNotifyWindow(hWnd: HWND) -> BOOL {
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'leave: {
        if IntIsWindow(hWnd) == 0 {
            break 'leave;
        }

        if gpidLogon != PsGetCurrentProcessId() {
            EngSetLastError(ERROR_ACCESS_DENIED);
            break 'leave;
        }

        hwndSAS = hWnd;
        Ret = 1;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserLockWorkStation() -> BOOL {
    let ret: BOOL;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    UserEnterExclusive();

    if (*pti).rpdesk == IntGetActiveDesktop() {
        ret = UserPostMessage(
            hwndSAS,
            WM_LOGONNOTIFY,
            LN_LOCK_WORKSTATION as _,
            0,
        );
    } else {
        ret = 0;
    }

    UserLeave();

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetWindowStationUser(
    hWindowStation: HWINSTA,
    pluid: PLUID,
    psid: PSID,
    size: DWORD,
) -> BOOL {
    let mut Ret: BOOL = 0;
    let mut Status: NTSTATUS;
    let mut WindowStation: PWINSTATION_OBJECT = core::ptr::null_mut();
    let mut luidUser: LUID = core::mem::zeroed();

    UserEnterExclusive();

    'leave: {
        if gpidLogon != PsGetCurrentProcessId() {
            EngSetLastError(ERROR_ACCESS_DENIED);
            break 'leave;
        }

        Status = IntValidateWindowStationHandle(
            hWindowStation,
            1i32 as KPROCESSOR_MODE,
            0,
            core::ptr::addr_of_mut!(WindowStation),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'leave;
        }

        Status = SafeProbeForRead(pluid as *mut _, core::mem::size_of::<LUID>() as SIZE_T, 1);
        if !NT_SUCCESS(Status) {
            break 'leave;
        }
        luidUser = *pluid;

        RtlZeroMemory(
            core::ptr::addr_of_mut!((*WindowStation).luidUser) as *mut _,
            core::mem::size_of::<LUID>(),
        );

        if !(*WindowStation).psidUser.is_null() {
            ExFreePoolWithTag((*WindowStation).psidUser, USERTAG_SECURITY);
            (*WindowStation).psidUser = core::ptr::null_mut();
        }

        if !psid.is_null() {
            (*WindowStation).psidUser =
                ExAllocatePoolWithTag(PagedPool, size as _, USERTAG_SECURITY);
            if (*WindowStation).psidUser.is_null() {
                EngSetLastError(ERROR_OUTOFMEMORY);
                break 'leave;
            }

            Status = STATUS_SUCCESS;
            Status = SafeProbeForRead(psid as *mut _, size as SIZE_T, 1);
            if NT_SUCCESS(Status) {
                RtlCopyMemory((*WindowStation).psidUser, psid, size as usize);
            }

            if !NT_SUCCESS(Status) {
                ExFreePoolWithTag((*WindowStation).psidUser, USERTAG_SECURITY);
                (*WindowStation).psidUser = core::ptr::null_mut();
                break 'leave;
            }
        }

        (*WindowStation).luidUser = luidUser;

        Ret = 1;
    }

    if !WindowStation.is_null() {
        ObDereferenceObject(WindowStation as *mut _);
    }

    UserLeave();
    Ret
}
