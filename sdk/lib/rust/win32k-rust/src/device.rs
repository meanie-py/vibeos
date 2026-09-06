use win32k_sys::win32k::*;
// Several imports below are unused because their using functions are currently DEFERRED-STUBs;
// keep them so resurrection from the inline C source needs zero re-discovery.
#[allow(unused_imports)]
use ntkrnl_sys::ddk::{ExAllocatePoolZero, ExFreePool, IoBuildDeviceIoControlRequest, IoBuildSynchronousFsdRequest, IoGetDeviceObjectPointer, IoGetNextIrpStackLocation, IoGetRelatedDeviceObject, IoOpenDeviceRegistryKey, IoSynchronousInvalidateDeviceRelations, KeInitializeEvent, KeWaitForSingleObject, RtlEqualUnicodeString, RtlInitUnicodeString, RtlQueryRegistryValues, ZwClose};
// Bring in NT_SUCCESS, STATUS_*, ExAllocatePoolWithTag, ExFreePoolWithTag, RtlCopyMemory,
// ObDereferenceObject from ntkrnl_sys. These are not in `win32k_sys::win32k::*` but the
// translation needs them constantly.
use ntkrnl_sys::base::NT_SUCCESS;
#[allow(unused_imports)]
use ntkrnl_sys::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NOT_IMPLEMENTED};
#[allow(unused_imports)]
use ntkrnl_sys::constants::{
    STATUS_UNSUCCESSFUL, STATUS_PENDING, STATUS_NOT_SUPPORTED, STATUS_INVALID_PARAMETER,
    STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_HANDLE,
};
#[allow(unused_imports)]
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
#[allow(unused_imports)]
use ntkrnl_sys::rtl::RtlCopyMemory;
#[allow(unused_imports)]
use ntkrnl_sys::ob::ObDereferenceObject;
#[allow(unused_imports)]
use core::ffi::c_void;

// STATUS_DEVICE_DOES_NOT_EXIST not in ntkrnl_sys::constants — define locally
// (matches winerror.h / ntstatus.h).
const STATUS_DEVICE_DOES_NOT_EXIST: NTSTATUS = 0xC00000C0_u32 as i32;

// IoCallDriver is a C macro `#define IoCallDriver(a,b) IofCallDriver(a,b)`. Forward to the
// ntkrnl_sys IofCallDriver but accept the win32k-sys flavoured pointers (layouts are
// identical — both crates bindgen the same C struct).
unsafe fn IoCallDriver(device: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    ntkrnl_sys::ddk::IofCallDriver(
        device as *mut ntkrnl_sys::ddk::_DEVICE_OBJECT,
        irp as *mut ntkrnl_sys::ddk::_IRP,
    )
}

extern "C" {
    // libcnt string helpers (UCRT shape; libcntpr exports these undecorated).
    fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    fn swprintf(buf: *mut u16, fmt: *const u16, ...) -> i32;

    // ScreenDeviceContext is a real global in user/ntuser/desktop.c — plain extern.
    static ScreenDeviceContext: HDC;
}

// win32ss/user/ntuser/misc/registry.c — NTAPI = stdcall on i386, cdecl elsewhere.
extern "system" {
    fn RegOpenKey(KeyName: *const u16, phKey: *mut HKEY) -> i32;
    fn RegQueryValue(hKey: HKEY, ValueName: *const u16, Type: u32, Data: *mut core::ffi::c_void, DataSize: *mut u32) -> i32;
}

// win32ss/user/ntuser/misc/registry.c:147 — NTAPI = stdcall on i386, cdecl elsewhere.
extern "system" {
    fn RegReadDWORD(hkey: HKEY, pwszValue: *const u16, pdwData: *mut DWORD) -> BOOL;
}

// win32ss/user/ntuser/display.c:67 — NTAPI = stdcall on i386, cdecl elsewhere.
// Called from EngpUpdateGraphicsDeviceList for each \Device\VideoN registry entry.
extern "system" {
    fn InitDisplayDriver(pwszDeviceName: *mut u16, pwszRegKey: *mut u16) -> PGRAPHICS_DEVICE;
}

// RtlStringCchCopyW is NTSTRSAFEAPI (`static __inline NTSTATUS NTAPI`) — no extern
// symbol exists, so we inline a Rust equivalent. Copies up to cch_dest-1 wide chars
// from src then null-terminates. Returns STATUS_SUCCESS on success.
unsafe fn RtlStringCchCopyW(dest: *mut u16, cch_dest: usize, src: *const u16) -> NTSTATUS {
    if cch_dest == 0 {
        return STATUS_INVALID_PARAMETER as NTSTATUS;
    }
    let mut i: usize = 0;
    while i < cch_dest - 1 {
        let c = *src.add(i);
        *dest.add(i) = c;
        if c == 0 {
            return STATUS_SUCCESS;
        }
        i += 1;
    }
    // Truncated: terminate and report buffer overflow.
    *dest.add(cch_dest - 1) = 0;
    STATUS_BUFFER_OVERFLOW as NTSTATUS
}

// Mirror of RtlStringCbCopyNW (NTSTRSAFEAPI, byte-counted Unicode copy). Used by
// EngpRegisterGraphicsDevice to copy the NT device name into the GRAPHICS_DEVICE.
unsafe fn RtlStringCbCopyNW(dest: *mut u16, cb_dest: usize, src: *const u16, cb_src: usize) -> NTSTATUS {
    let cch_dest = cb_dest / core::mem::size_of::<u16>();
    let cch_src = cb_src / core::mem::size_of::<u16>();
    if cch_dest == 0 {
        return STATUS_INVALID_PARAMETER as NTSTATUS;
    }
    let copy = cch_src.min(cch_dest - 1);
    for i in 0..copy {
        *dest.add(i) = *src.add(i);
    }
    *dest.add(copy) = 0;
    if cch_src >= cch_dest {
        STATUS_BUFFER_OVERFLOW as NTSTATUS
    } else {
        STATUS_SUCCESS
    }
}

// These are C `#define` macros, NOT symbols. Earlier rustconv emitted them as
// `extern static ULONG` which produces undefined-symbol link errors AND, on
// arches that auto-link them somewhere, reads random memory at runtime.
// Convert to `const` with the actual values.
//
// Pool tags: C `'vdgG'` (multi-char literal, big-endian) = u32::from_le_bytes(b"Ggdv")
// per AGENTS.md convention.
const GDITAG_GDEVICE: ULONG = u32::from_le_bytes(*b"Ggdv");   // 'vdgG'
const GDITAG_DRVSUP:  ULONG = u32::from_le_bytes(*b"Gdrs");   // 'srdG'

// IOCTLs from sdk/include/psdk/ntddvdeo.h:
//   #define IOCTL_VIDEO_ENUM_MONITOR_PDO        CTL_CODE(FILE_DEVICE_VIDEO=0x23, 0x06, METHOD_BUFFERED=0, FILE_ANY_ACCESS=0)
//   #define IOCTL_VIDEO_INIT_WIN32K_CALLBACKS   CTL_CODE(0x23, 0x07, 0, 0)
// CTL_CODE = (Device << 16) | (Access << 14) | (Function << 2) | Method
const IOCTL_VIDEO_ENUM_MONITOR_PDO:      ULONG = 0x00230018;  // (0x23<<16) | (0x06<<2)
const IOCTL_VIDEO_INIT_WIN32K_CALLBACKS: ULONG = 0x0023001C;  // (0x23<<16) | (0x07<<2)

// ---- file-scope preamble (translated) ----
static mut gpPrimaryGraphicsDevice: PGRAPHICS_DEVICE = core::ptr::null_mut();
static mut gpVgaGraphicsDevice: PGRAPHICS_DEVICE = core::ptr::null_mut();
static mut gpGraphicsDeviceFirst: PGRAPHICS_DEVICE = core::ptr::null_mut();
static mut gpGraphicsDeviceLast: PGRAPHICS_DEVICE = core::ptr::null_mut();
static mut ghsemGraphicsDeviceList: HSEMAPHORE = core::ptr::null_mut();
static mut giDevNum: ULONG = 1;

#[inline(always)]
unsafe fn InitDeviceImpl_impl() -> NTSTATUS {
    ghsemGraphicsDeviceList = EngCreateSemaphore();
    if ghsemGraphicsDeviceList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn InitDeviceImpl() -> NTSTATUS {
    InitDeviceImpl_impl()
}

// Translated from device.c:36-99 (2026-05-28 resurrection).
// Reads HARDWARE\DEVICEMAP\VIDEO, finds the device key, opens it, reads its
// Service name, returns TRUE iff the service is "VGASave" (case-insensitive).
unsafe fn EngpHasVgaDriver(pGraphicsDevice: PGRAPHICS_DEVICE) -> u8 {
    // L"\Registry\Machine\HARDWARE\DEVICEMAP\VIDEO" + L"Service" + L"VGASave" wide literals.
    static KEY_VIDEO: &[u16] = &[
        0x005c, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079,
        0x005c, 0x004d, 0x0061, 0x0063, 0x0068, 0x0069, 0x006e, 0x0065,
        0x005c, 0x0048, 0x0041, 0x0052, 0x0044, 0x0057, 0x0041, 0x0052, 0x0045,
        0x005c, 0x0044, 0x0045, 0x0056, 0x0049, 0x0043, 0x0045, 0x004d, 0x0041, 0x0050,
        0x005c, 0x0056, 0x0049, 0x0044, 0x0045, 0x004f, 0x0000,
    ];
    static SERVICE: &[u16] = &[0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0000];
    static VGASAVE: &[u16] = &[0x0056, 0x0047, 0x0041, 0x0053, 0x0061, 0x0076, 0x0065, 0x0000];
    static VIDEO: &[u16] = &[0x0056, 0x0069, 0x0064, 0x0065, 0x006f, 0x0000];

    let mut awcDeviceKey: [u16; 256] = [0; 256];
    let mut awcServiceName: [u16; 100] = [0; 100];
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut cbValue: ULONG;

    // Open the key for the adapters
    let mut Status = RegOpenKey(KEY_VIDEO.as_ptr(), &mut hkey);
    if !NT_SUCCESS(Status) {
        ERR!("EngpHasVgaDriver: open DEVICEMAP\\VIDEO failed 0x%08x\n", Status);
        return 0;
    }

    // Read the name of the device key
    cbValue = core::mem::size_of_val(&awcDeviceKey) as ULONG;
    Status = RegQueryValue(
        hkey,
        (*pGraphicsDevice).szNtDeviceName.as_ptr(),
        REG_SZ,
        awcDeviceKey.as_mut_ptr() as *mut core::ffi::c_void,
        &mut cbValue,
    );
    ZwClose(hkey as *mut core::ffi::c_void);
    if !NT_SUCCESS(Status) {
        return 0;
    }

    // Replace last "...\DeviceN" component with "...\Video"
    let lastBkSlash = wcsrchr(awcDeviceKey.as_ptr(), 0x005c /* '\\' */);
    if lastBkSlash.is_null() {
        return 0;
    }
    // Offset from start of awcDeviceKey to char after '\'.
    let prefix_chars = lastBkSlash.offset_from(awcDeviceKey.as_ptr()) as usize + 1;
    let remaining = awcDeviceKey.len() - prefix_chars;
    if !NT_SUCCESS(RtlStringCchCopyW(
        awcDeviceKey.as_mut_ptr().add(prefix_chars),
        remaining,
        VIDEO.as_ptr(),
    )) {
        return 0;
    }

    // Open device key
    Status = RegOpenKey(awcDeviceKey.as_ptr(), &mut hkey);
    if !NT_SUCCESS(Status) {
        return 0;
    }

    // Read service name
    cbValue = core::mem::size_of_val(&awcServiceName) as ULONG;
    Status = RegQueryValue(
        hkey,
        SERVICE.as_ptr(),
        REG_SZ,
        awcServiceName.as_mut_ptr() as *mut core::ffi::c_void,
        &mut cbValue,
    );
    ZwClose(hkey as *mut core::ffi::c_void);
    if !NT_SUCCESS(Status) {
        return 0;
    }

    if _wcsicmp(awcServiceName.as_ptr(), VGASAVE.as_ptr()) == 0 { 1 } else { 0 }
}

unsafe fn EngpLinkGraphicsDevice(
    pToAdd: PGRAPHICS_DEVICE)
{
    let mut pGraphicsDevice: PGRAPHICS_DEVICE;

    TRACE!(b"EngLinkGraphicsDevice(%p)\n\0".as_ptr() as PCHAR, pToAdd);

    pGraphicsDevice = gpGraphicsDeviceFirst;
    while !pGraphicsDevice.is_null() {
        if pGraphicsDevice == pToAdd {
            return;
        }
        pGraphicsDevice = (*pGraphicsDevice).pNextGraphicsDevice;
    }

    (*pToAdd).pNextGraphicsDevice = core::ptr::null_mut();
    if !gpGraphicsDeviceLast.is_null() {
        (*gpGraphicsDeviceLast).pNextGraphicsDevice = pToAdd;
    }
    gpGraphicsDeviceLast = pToAdd;
    if gpGraphicsDeviceFirst.is_null() {
        gpGraphicsDeviceFirst = pToAdd;
    }
}

unsafe fn EngpUnlinkGraphicsDevice(
    pToDelete: PGRAPHICS_DEVICE)
{
    let mut pPrevGraphicsDevice: PGRAPHICS_DEVICE = core::ptr::null_mut();
    let mut pGraphicsDevice: PGRAPHICS_DEVICE = gpGraphicsDeviceFirst;

    TRACE!(b"EngpUnlinkGraphicsDevice('%S')\n\0".as_ptr() as PCHAR, (*pToDelete).szNtDeviceName.as_ptr());

    while !pGraphicsDevice.is_null()
    {
        if pGraphicsDevice != pToDelete
        {
            pPrevGraphicsDevice = pGraphicsDevice;
            pGraphicsDevice = (*pGraphicsDevice).pNextGraphicsDevice;
        }
        else
        {
            if !(*pGraphicsDevice).pVgaDevice.is_null() {
                EngpLinkGraphicsDevice((*pGraphicsDevice).pVgaDevice);
            }

            pGraphicsDevice = (*pGraphicsDevice).pNextGraphicsDevice;

            if pPrevGraphicsDevice.is_null() {
                gpGraphicsDeviceFirst = (*pToDelete).pNextGraphicsDevice;
            } else {
                (*pPrevGraphicsDevice).pNextGraphicsDevice = (*pToDelete).pNextGraphicsDevice;
            }
            if gpGraphicsDeviceLast == pToDelete {
                gpGraphicsDeviceLast = pPrevGraphicsDevice;
            }
        }
    }
}

// Translated from device.c:182-338 (2026-05-28 resurrection).
// Walks HARDWARE\DEVICEMAP\VIDEO, calls InitDisplayDriver (user/display.c)
// for each new \Device\VideoN, then classifies VGA / primary devices and
// links the VGA device under the primary when both exist.
#[no_mangle]
pub unsafe extern "C" fn EngpUpdateGraphicsDeviceList() -> NTSTATUS {
    // L"\Registry\Machine\HARDWARE\DEVICEMAP\VIDEO"
    static KEY_VIDEO: &[u16] = &[
        0x005c, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079,
        0x005c, 0x004d, 0x0061, 0x0063, 0x0068, 0x0069, 0x006e, 0x0065,
        0x005c, 0x0048, 0x0041, 0x0052, 0x0044, 0x0057, 0x0041, 0x0052, 0x0045,
        0x005c, 0x0044, 0x0045, 0x0056, 0x0049, 0x0043, 0x0045, 0x004d, 0x0041, 0x0050,
        0x005c, 0x0056, 0x0049, 0x0044, 0x0045, 0x004f, 0x0000,
    ];
    // L"MaxObjectNumber"
    static MAX_OBJ_NUMBER: &[u16] = &[
        0x004d, 0x0061, 0x0078, 0x004f, 0x0062, 0x006a, 0x0065, 0x0063, 0x0074,
        0x004e, 0x0075, 0x006d, 0x0062, 0x0065, 0x0072, 0x0000,
    ];
    // L"\\Device\\Video%lu"
    static FMT_DEVICE_VIDEO: &[u16] = &[
        0x005c, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
        0x005c, 0x0056, 0x0069, 0x0064, 0x0065, 0x006f,
        0x0025, 0x006c, 0x0075, 0x0000,
    ];
    // L"\\\\.\\DISPLAY%lu"
    static FMT_DISPLAY: &[u16] = &[
        0x005c, 0x005c, 0x002e, 0x005c,
        0x0044, 0x0049, 0x0053, 0x0050, 0x004c, 0x0041, 0x0059,
        0x0025, 0x006c, 0x0075, 0x0000,
    ];

    let mut ulMaxObjectNumber: ULONG = 0;
    let mut awcDeviceName: [u16; 20] = [0; 20];
    let mut awcWinDeviceName: [u16; 20] = [0; 20];
    let mut ustrDeviceName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut awcBuffer: [u16; 256] = [0; 256];
    let mut hkey: HKEY = core::ptr::null_mut();

    // Open the key for the adapters
    let mut Status = RegOpenKey(KEY_VIDEO.as_ptr(), &mut hkey);
    if !NT_SUCCESS(Status) {
        ERR!("EngpUpdateGraphicsDeviceList: open DEVICEMAP\\VIDEO failed 0x%lx\n", Status);
        return Status;
    }

    // Get the maximum number of adapters
    if RegReadDWORD(hkey, MAX_OBJ_NUMBER.as_ptr(), &mut ulMaxObjectNumber as *mut ULONG as *mut DWORD) == 0 {
        ERR!("Could not read MaxObjectNumber, defaulting to 0.\n");
    }

    // Loop through all adapters, to detect new ones
    let mut iDevNum: ULONG = 0;
    while iDevNum <= ulMaxObjectNumber {
        // Build adapter key name
        swprintf(awcDeviceName.as_mut_ptr(), FMT_DEVICE_VIDEO.as_ptr(), iDevNum);
        // Build Win32 display device name
        swprintf(awcWinDeviceName.as_mut_ptr(), FMT_DISPLAY.as_ptr(), iDevNum + 1);
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ustrDeviceName), awcWinDeviceName.as_ptr());

        // Already registered?
        let pGraphicsDevice = EngpFindGraphicsDevice(
            core::ptr::addr_of_mut!(ustrDeviceName) as PUNICODE_STRING,
            iDevNum,
        );
        if !pGraphicsDevice.is_null() {
            iDevNum += 1;
            continue;
        }

        // Read the reg key name
        let mut cbValue: ULONG = core::mem::size_of_val(&awcBuffer) as ULONG;
        Status = RegQueryValue(
            hkey,
            awcDeviceName.as_ptr(),
            REG_SZ,
            awcBuffer.as_mut_ptr() as *mut core::ffi::c_void,
            &mut cbValue,
        );
        if !NT_SUCCESS(Status) {
            iDevNum += 1;
            continue;
        }

        // Initialize the driver for this device
        let pGraphicsDevice = InitDisplayDriver(awcDeviceName.as_mut_ptr(), awcBuffer.as_mut_ptr());
        if pGraphicsDevice.is_null() {
            iDevNum += 1;
            continue;
        }
        iDevNum += 1;
    }

    // Close the device map registry key
    ZwClose(hkey as *mut core::ffi::c_void);

    // Choose a VGA device — DISPLAY_DEVICE_VGA_COMPATIBLE + VGASave service.
    let mut pGraphicsDevice = gpGraphicsDeviceFirst;
    while !pGraphicsDevice.is_null() {
        if pGraphicsDevice == gpVgaGraphicsDevice {
            pGraphicsDevice = (*pGraphicsDevice).pNextGraphicsDevice;
            continue;
        }
        if ((*pGraphicsDevice).StateFlags & DISPLAY_DEVICE_VGA_COMPATIBLE) != 0
            && EngpHasVgaDriver(pGraphicsDevice) != 0
        {
            gpVgaGraphicsDevice = pGraphicsDevice;
            break;
        }
        pGraphicsDevice = (*pGraphicsDevice).pNextGraphicsDevice;
    }

    // Handle gbBaseVideo — remove every non-base-video device from the global list.
    if gbBaseVideo != 0 {
        EngAcquireSemaphore(ghsemGraphicsDeviceList);
        let mut pGraphicsDevice = gpGraphicsDeviceFirst;
        while !pGraphicsDevice.is_null() {
            if EngpHasVgaDriver(pGraphicsDevice) == 0 {
                let pToDelete = pGraphicsDevice;
                EngpUnlinkGraphicsDevice(pGraphicsDevice);
                pGraphicsDevice = (*pGraphicsDevice).pNextGraphicsDevice;
                ExFreePoolWithTag((*pToDelete).pDiplayDrivers as PVOID, GDITAG_DRVSUP);
                ExFreePoolWithTag(pToDelete as PVOID, GDITAG_GDEVICE);
            } else {
                pGraphicsDevice = (*pGraphicsDevice).pNextGraphicsDevice;
            }
        }
        EngReleaseSemaphore(ghsemGraphicsDeviceList);
    }

    // Choose a primary device if none yet — prefer non-VGA-Save, fall back to first.
    if gpPrimaryGraphicsDevice.is_null() {
        let mut pNewPrimaryGraphicsDevice: PGRAPHICS_DEVICE = core::ptr::null_mut();
        let mut pGraphicsDevice = gpGraphicsDeviceFirst;
        while !pGraphicsDevice.is_null() {
            if EngpHasVgaDriver(pGraphicsDevice) == 0 {
                pNewPrimaryGraphicsDevice = pGraphicsDevice;
                break;
            }
            pGraphicsDevice = (*pGraphicsDevice).pNextGraphicsDevice;
        }
        if pNewPrimaryGraphicsDevice.is_null() {
            pNewPrimaryGraphicsDevice = gpGraphicsDeviceFirst;
        }
        if !pNewPrimaryGraphicsDevice.is_null() {
            (*pNewPrimaryGraphicsDevice).StateFlags |= DISPLAY_DEVICE_PRIMARY_DEVICE;
            gpPrimaryGraphicsDevice = pNewPrimaryGraphicsDevice;
        }
    }

    // If both primary and VGA exist and aren't already linked, splice VGA under primary.
    if !gpPrimaryGraphicsDevice.is_null()
        && !gpVgaGraphicsDevice.is_null()
        && gpPrimaryGraphicsDevice != gpVgaGraphicsDevice
        && (*gpPrimaryGraphicsDevice).pVgaDevice.is_null()
    {
        EngAcquireSemaphore(ghsemGraphicsDeviceList);
        EngpUnlinkGraphicsDevice(gpVgaGraphicsDevice);
        (*gpPrimaryGraphicsDevice).pVgaDevice = gpVgaGraphicsDevice;
        EngReleaseSemaphore(ghsemGraphicsDeviceList);
    }

    STATUS_SUCCESS
}

unsafe fn EngpGetRegistryHandleFromDeviceMap(
    pGraphicsDevice: PGRAPHICS_DEVICE
) -> HKEY {
    static KEY_VIDEO: &[u16] = &[
        92, 82, 101, 103, 105, 115, 116, 114, 121, 92, 77, 97, 99, 104, 105, 110, 101, 92,
        72, 65, 82, 68, 87, 65, 82, 69, 92, 68, 69, 86, 73, 67, 69, 77, 65, 80, 92, 86, 73, 68, 69, 79, 0
    ];
    
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szDeviceKey: [u16; 256] = [0; 256];
    let mut cbSize: u32;
    let mut Status: i32;

    Status = RegOpenKey(KEY_VIDEO.as_ptr(), &mut hKey);
    if !NT_SUCCESS(Status) {
        ERR!("Could not open HARDWARE\\DEVICEMAP\\VIDEO registry key: status 0x%08x\n", Status);
        return core::ptr::null_mut();
    }

    cbSize = core::mem::size_of_val(&szDeviceKey) as u32;
    RegQueryValue(hKey,
                  (*pGraphicsDevice).szNtDeviceName.as_ptr(),
                  REG_SZ,
                  szDeviceKey.as_mut_ptr() as *mut core::ffi::c_void,
                  &mut cbSize);
    ZwClose(hKey as *mut core::ffi::c_void);

    Status = RegOpenKey(szDeviceKey.as_ptr(), &mut hKey);
    if !NT_SUCCESS(Status) {
        ERR!("Could not open registry key '%S': status 0x%08x\n", szDeviceKey.as_ptr(), Status);
        return core::ptr::null_mut();
    }

    hKey
}

// Plain cdecl in C (device.h:47, device.c:381).
#[no_mangle]
pub unsafe extern "C" fn EngpGetDisplayDriverParameters(
    pGraphicsDevice: PGRAPHICS_DEVICE,
    pdm: PDEVMODEW
) -> i32 {
    const RTL_REGISTRY_HANDLE_LOCAL: ULONG = 0x40000000;
    const RTL_QUERY_REGISTRY_DIRECT_LOCAL: ULONG = 0x20;
    const REG_NONE_LOCAL: ULONG = 0;

    static DEFAULT_BITS_PER_PEL: &[u16] = &[
        68, 101, 102, 97, 117, 108, 116, 83, 101, 116, 116, 105, 110, 103, 115, 46, 66, 105,
        116, 115, 80, 101, 114, 80, 101, 108, 0,
    ];
    static DEFAULT_X_RESOLUTION: &[u16] = &[
        68, 101, 102, 97, 117, 108, 116, 83, 101, 116, 116, 105, 110, 103, 115, 46, 88, 82,
        101, 115, 111, 108, 117, 116, 105, 111, 110, 0,
    ];
    static DEFAULT_Y_RESOLUTION: &[u16] = &[
        68, 101, 102, 97, 117, 108, 116, 83, 101, 116, 116, 105, 110, 103, 115, 46, 89, 82,
        101, 115, 111, 108, 117, 116, 105, 111, 110, 0,
    ];
    static DEFAULT_FLAGS: &[u16] = &[
        68, 101, 102, 97, 117, 108, 116, 83, 101, 116, 116, 105, 110, 103, 115, 46, 70, 108,
        97, 103, 115, 0,
    ];
    static DEFAULT_V_REFRESH: &[u16] = &[
        68, 101, 102, 97, 117, 108, 116, 83, 101, 116, 116, 105, 110, 103, 115, 46, 86, 82,
        101, 102, 114, 101, 115, 104, 0,
    ];
    static DEFAULT_X_PANNING: &[u16] = &[
        68, 101, 102, 97, 117, 108, 116, 83, 101, 116, 116, 105, 110, 103, 115, 46, 88, 80,
        97, 110, 110, 105, 110, 103, 0,
    ];
    static DEFAULT_Y_PANNING: &[u16] = &[
        68, 101, 102, 97, 117, 108, 116, 83, 101, 116, 116, 105, 110, 103, 115, 46, 89, 80,
        97, 110, 110, 105, 110, 103, 0,
    ];
    static DEFAULT_ORIENTATION: &[u16] = &[
        68, 101, 102, 97, 117, 108, 116, 83, 101, 116, 116, 105, 110, 103, 115, 46, 79, 114,
        105, 101, 110, 116, 97, 116, 105, 111, 110, 0,
    ];
    static DEFAULT_FIXED_OUTPUT: &[u16] = &[
        68, 101, 102, 97, 117, 108, 116, 83, 101, 116, 116, 105, 110, 103, 115, 46, 70, 105,
        120, 101, 100, 79, 117, 116, 112, 117, 116, 0,
    ];
    static ATTACH_RELATIVE_X: &[u16] = &[
        65, 116, 116, 97, 99, 104, 46, 82, 101, 108, 97, 116, 105, 118, 101, 88, 0,
    ];
    static ATTACH_RELATIVE_Y: &[u16] = &[
        65, 116, 116, 97, 99, 104, 46, 82, 101, 108, 97, 116, 105, 118, 101, 89, 0,
    ];

    if pdm.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let mut table: [ntkrnl_sys::ddk::_RTL_QUERY_REGISTRY_TABLE; 12] = core::mem::zeroed();

    table[0].Flags = RTL_QUERY_REGISTRY_DIRECT_LOCAL;
    table[0].Name = DEFAULT_BITS_PER_PEL.as_ptr();
    table[0].EntryContext = core::ptr::addr_of_mut!((*pdm).dmBitsPerPel) as PVOID;
    table[0].DefaultType = REG_NONE_LOCAL;

    table[1].Flags = RTL_QUERY_REGISTRY_DIRECT_LOCAL;
    table[1].Name = DEFAULT_X_RESOLUTION.as_ptr();
    table[1].EntryContext = core::ptr::addr_of_mut!((*pdm).dmPelsWidth) as PVOID;
    table[1].DefaultType = REG_NONE_LOCAL;

    table[2].Flags = RTL_QUERY_REGISTRY_DIRECT_LOCAL;
    table[2].Name = DEFAULT_Y_RESOLUTION.as_ptr();
    table[2].EntryContext = core::ptr::addr_of_mut!((*pdm).dmPelsHeight) as PVOID;
    table[2].DefaultType = REG_NONE_LOCAL;

    table[3].Flags = RTL_QUERY_REGISTRY_DIRECT_LOCAL;
    table[3].Name = DEFAULT_FLAGS.as_ptr();
    table[3].EntryContext = core::ptr::addr_of_mut!((*pdm).__bindgen_anon_2.dmDisplayFlags) as PVOID;
    table[3].DefaultType = REG_NONE_LOCAL;

    table[4].Flags = RTL_QUERY_REGISTRY_DIRECT_LOCAL;
    table[4].Name = DEFAULT_V_REFRESH.as_ptr();
    table[4].EntryContext = core::ptr::addr_of_mut!((*pdm).dmDisplayFrequency) as PVOID;
    table[4].DefaultType = REG_NONE_LOCAL;

    table[5].Flags = RTL_QUERY_REGISTRY_DIRECT_LOCAL;
    table[5].Name = DEFAULT_X_PANNING.as_ptr();
    table[5].EntryContext = core::ptr::addr_of_mut!((*pdm).dmPanningWidth) as PVOID;
    table[5].DefaultType = REG_NONE_LOCAL;

    table[6].Flags = RTL_QUERY_REGISTRY_DIRECT_LOCAL;
    table[6].Name = DEFAULT_Y_PANNING.as_ptr();
    table[6].EntryContext = core::ptr::addr_of_mut!((*pdm).dmPanningHeight) as PVOID;
    table[6].DefaultType = REG_NONE_LOCAL;

    table[7].Flags = RTL_QUERY_REGISTRY_DIRECT_LOCAL;
    table[7].Name = DEFAULT_ORIENTATION.as_ptr();
    table[7].EntryContext = core::ptr::addr_of_mut!((*pdm).__bindgen_anon_1.__bindgen_anon_2.dmDisplayOrientation) as PVOID;
    table[7].DefaultType = REG_NONE_LOCAL;

    table[8].Flags = RTL_QUERY_REGISTRY_DIRECT_LOCAL;
    table[8].Name = DEFAULT_FIXED_OUTPUT.as_ptr();
    table[8].EntryContext = core::ptr::addr_of_mut!((*pdm).__bindgen_anon_1.__bindgen_anon_2.dmDisplayFixedOutput) as PVOID;
    table[8].DefaultType = REG_NONE_LOCAL;

    table[9].Flags = RTL_QUERY_REGISTRY_DIRECT_LOCAL;
    table[9].Name = ATTACH_RELATIVE_X.as_ptr();
    table[9].EntryContext = core::ptr::addr_of_mut!((*pdm).__bindgen_anon_1.__bindgen_anon_2.dmPosition.x) as PVOID;
    table[9].DefaultType = REG_NONE_LOCAL;

    table[10].Flags = RTL_QUERY_REGISTRY_DIRECT_LOCAL;
    table[10].Name = ATTACH_RELATIVE_Y.as_ptr();
    table[10].EntryContext = core::ptr::addr_of_mut!((*pdm).__bindgen_anon_1.__bindgen_anon_2.dmPosition.y) as PVOID;
    table[10].DefaultType = REG_NONE_LOCAL;

    let hKey = EngpGetRegistryHandleFromDeviceMap(pGraphicsDevice);
    if hKey.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    let status = RtlQueryRegistryValues(
        RTL_REGISTRY_HANDLE_LOCAL,
        hKey as ntkrnl_sys::ddk::PCWSTR,
        table.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    ZwClose(hKey as *mut core::ffi::c_void);
    status
}

// Plain cdecl in C (device.h:53, device.c:427).
#[no_mangle]
pub unsafe extern "C" fn EngpGetDisplayDriverAccelerationLevel(
    pGraphicsDevice: PGRAPHICS_DEVICE
) -> u32 {
    // DEFERRED-STUB (win32k-translation): translation has cross-crate type / missing-decl issues.
    // TODO: resurrect from the original C in the bottom-of-file source block.
    let _ = pGraphicsDevice;
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn VideoPortCallout(
    Params: PVOID,
) {
    // DEFERRED-STUB (win32k-translation): translation has cross-crate type / missing-decl issues.
    // TODO: resurrect from the original C in the bottom-of-file source block.
    let _ = Params;
}

unsafe fn EngpPnPTargetRelationRequest(
    pDeviceObject: PDEVICE_OBJECT,
    pDeviceRelations: *mut PDEVICE_RELATIONS) -> NTSTATUS
{
    // DEFERRED-STUB (win32k-translation): translation has cross-crate type / missing-decl issues.
    // TODO: resurrect from the original C in the bottom-of-file source block.
    let _ = (pDeviceObject, pDeviceRelations);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn EngpUpdateMonitorDevices(
    pGraphicsDevice: PGRAPHICS_DEVICE) -> NTSTATUS
{
    // DEFERRED-STUB (win32k-translation): translation has cross-crate type / missing-decl issues.
    // TODO: resurrect from the original C in the bottom-of-file source block.
    let _ = pGraphicsDevice;
    STATUS_SUCCESS
}

// Translated from device.c:667-798 (2026-05-28 resurrection).
// Allocates a GRAPHICS_DEVICE, opens the device, sets up Win32K callbacks via
// IOCTL_VIDEO_INIT_WIN32K_CALLBACKS, copies the NT/Win32 names + driver list,
// then links into the global graphics device list. Returns NULL on failure.
#[no_mangle]
pub unsafe extern "system" fn EngpRegisterGraphicsDevice(
    pustrDeviceName: PUNICODE_STRING,
    pustrDiplayDrivers: PUNICODE_STRING,
    pustrDescription: PUNICODE_STRING,
) -> PGRAPHICS_DEVICE {
    // L"\\\\.\\DISPLAY%d"
    static FMT_DISPLAY: &[u16] = &[
        0x005c, 0x005c, 0x002e, 0x005c,
        0x0044, 0x0049, 0x0053, 0x0050, 0x004c, 0x0041, 0x0059,
        0x0025, 0x0064, 0x0000,
    ];
    // L"DISPLAY" for the wallpaper-paint HACK at the end.
    static DRIVER_DISPLAY: &[u16] = &[0x0044, 0x0049, 0x0053, 0x0050, 0x004c, 0x0041, 0x0059, 0x0000];

    // Allocate a GRAPHICS_DEVICE structure (zero-init).
    // ExAllocatePoolZero is FORCEINLINE in exfuncs.h (= AllocPoolWithTag + ZeroMemory),
    // so no exported symbol — inline the same pattern.
    let size = core::mem::size_of::<GRAPHICS_DEVICE>();
    let pGraphicsDevice = ExAllocatePoolWithTag(PagedPool, size as _, GDITAG_GDEVICE) as PGRAPHICS_DEVICE;
    if pGraphicsDevice.is_null() {
        ERR!("ExAllocatePoolWithTag failed\n");
        return core::ptr::null_mut();
    }
    core::ptr::write_bytes(pGraphicsDevice as *mut u8, 0, size);

    // Try to open and enable the device.
    let mut pFileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut pDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let Status = IoGetDeviceObjectPointer(
        pustrDeviceName as *mut ntkrnl_sys::ddk::_UNICODE_STRING,
        FILE_READ_DATA | FILE_WRITE_DATA,
        &mut pFileObject as *mut _ as *mut *mut ntkrnl_sys::ddk::_FILE_OBJECT,
        &mut pDeviceObject as *mut _ as *mut *mut ntkrnl_sys::ddk::_DEVICE_OBJECT,
    );
    if !NT_SUCCESS(Status) {
        ERR!("Could not open device, 0x%lx\n", Status);
        ExFreePoolWithTag(pGraphicsDevice as PVOID, GDITAG_GDEVICE);
        return core::ptr::null_mut();
    }

    // Copy the device and file object pointers.
    (*pGraphicsDevice).DeviceObject = pDeviceObject;
    (*pGraphicsDevice).FileObject = pFileObject;

    // Initialize and register the device with videoprt for Win32k callbacks.
    let mut Win32kCallbacks: VIDEO_WIN32K_CALLBACKS = core::mem::zeroed();
    Win32kCallbacks.PhysDisp = pGraphicsDevice as PVOID;
    // PVIDEO_WIN32K_CALLOUT is `extern "stdcall" fn` on i386 / `extern "C" fn`
    // elsewhere — VideoPortCallout uses `extern "system"` (= stdcall on i386,
    // cdecl elsewhere), which is identical at the ABI level but a distinct
    // type to rustc. Transmute via raw fn-pointer pun.
    Win32kCallbacks.Callout = Some(core::mem::transmute::<usize, _>(VideoPortCallout as usize));
    // bACPI/pPhysDeviceObject/DualviewFlags already zeroed.
    let mut ulReturn: ULONG = 0;
    let ioctl_status = EngDeviceIoControl(
        pDeviceObject as HANDLE,
        IOCTL_VIDEO_INIT_WIN32K_CALLBACKS,
        core::ptr::addr_of_mut!(Win32kCallbacks) as LPVOID,
        core::mem::size_of_val(&Win32kCallbacks) as DWORD,
        core::ptr::addr_of_mut!(Win32kCallbacks) as LPVOID,
        core::mem::size_of_val(&Win32kCallbacks) as DWORD,
        &mut ulReturn as *mut ULONG as LPDWORD,
    );
    if ioctl_status != ERROR_SUCCESS {
        ERR!("EngDeviceIoControl(IOCTL_VIDEO_INIT_WIN32K_CALLBACKS) failed 0x%lx\n", ioctl_status);
    }
    (*pGraphicsDevice).PhysDeviceHandle = Win32kCallbacks.pPhysDeviceObject as PDEVICE_OBJECT;

    // Copy the device name.
    RtlStringCbCopyNW(
        (*pGraphicsDevice).szNtDeviceName.as_mut_ptr(),
        core::mem::size_of_val(&(*pGraphicsDevice).szNtDeviceName),
        (*pustrDeviceName).Buffer,
        (*pustrDeviceName).Length as usize,
    );

    // Create a Win32 device name (FIXME: virtual devices!)
    // C uses RtlStringCbPrintfW, which is FORCEINLINE in ntstrsafe.h — no
    // exported symbol. Use libcntpr swprintf instead (output max ~14 chars
    // for DISPLAY%d, well within the 16-wchar szWinDeviceName buffer).
    swprintf(
        (*pGraphicsDevice).szWinDeviceName.as_mut_ptr(),
        FMT_DISPLAY.as_ptr(),
        giDevNum as i32,
    );

    // Allocate a buffer for the strings (drivers || description || trailing NUL).
    let cj = (*pustrDiplayDrivers).Length as usize
        + (*pustrDescription).Length as usize
        + core::mem::size_of::<u16>();
    let pwsz = ExAllocatePoolWithTag(PagedPool, cj as _, GDITAG_DRVSUP) as PWSTR;
    if pwsz.is_null() {
        ERR!("Could not allocate string buffer\n");
        ExFreePoolWithTag(pGraphicsDevice as PVOID, GDITAG_GDEVICE);
        return core::ptr::null_mut();
    }

    // Copy the display driver names (REG_MULTI_SZ).
    (*pGraphicsDevice).pDiplayDrivers = pwsz;
    RtlCopyMemory(
        (*pGraphicsDevice).pDiplayDrivers as *mut core::ffi::c_void,
        (*pustrDiplayDrivers).Buffer as *const core::ffi::c_void,
        (*pustrDiplayDrivers).Length as usize,
    );

    // Copy the description right after.
    let desc_offset_chars = (*pustrDiplayDrivers).Length as usize / core::mem::size_of::<u16>();
    (*pGraphicsDevice).pwszDescription = pwsz.add(desc_offset_chars);
    RtlCopyMemory(
        (*pGraphicsDevice).pwszDescription as *mut core::ffi::c_void,
        (*pustrDescription).Buffer as *const core::ffi::c_void,
        (*pustrDescription).Length as usize,
    );
    let desc_chars = (*pustrDescription).Length as usize / core::mem::size_of::<u16>();
    *(*pGraphicsDevice).pwszDescription.add(desc_chars) = 0;

    // Update list of connected monitors (stub for now — STATUS_SUCCESS no-op).
    EngpUpdateMonitorDevices(pGraphicsDevice);

    // Insert the device into the global list under the loader semaphore.
    EngAcquireSemaphore(ghsemGraphicsDeviceList);
    EngpLinkGraphicsDevice(pGraphicsDevice);
    giDevNum += 1;
    EngReleaseSemaphore(ghsemGraphicsDeviceList);

    // HACK: already in graphic mode → repaint desktop on this new display.
    if !ScreenDeviceContext.is_null() {
        let mut DriverName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
        let mut DisplayName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString(core::ptr::addr_of_mut!(DriverName), DRIVER_DISPLAY.as_ptr());
        RtlInitUnicodeString(core::ptr::addr_of_mut!(DisplayName), (*pGraphicsDevice).szWinDeviceName.as_ptr());
        let hdc = IntGdiCreateDC(
            core::ptr::addr_of_mut!(DriverName) as PUNICODE_STRING,
            core::ptr::addr_of_mut!(DisplayName) as PUNICODE_STRING,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
        );
        IntPaintDesktop(hdc);
    }

    pGraphicsDevice
}

// NTAPI in C — stdcall on i386, cdecl elsewhere.
#[no_mangle]
pub unsafe extern "system" fn EngpFindGraphicsDevice(
    pustrDevice: PUNICODE_STRING,
    iDevNum: u32,
) -> PGRAPHICS_DEVICE {
    EngpFindGraphicsDevice_impl(pustrDevice, iDevNum)
}

unsafe fn EngpFindGraphicsDevice_impl(
    pustrDevice: PUNICODE_STRING,
    iDevNum: u32,
) -> PGRAPHICS_DEVICE {
    // Use ntkrnl_sys flavour of _UNICODE_STRING for compatibility with RtlInit/RtlEqual.
    let mut ustrCurrent: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut pGraphicsDevice: PGRAPHICS_DEVICE;
    let mut i: u32;

    EngAcquireSemaphore(ghsemGraphicsDeviceList);

    if !pustrDevice.is_null() && !(*pustrDevice).Buffer.is_null() {
        pGraphicsDevice = gpGraphicsDeviceFirst;
        while !pGraphicsDevice.is_null() {
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(ustrCurrent),
                (*pGraphicsDevice).szWinDeviceName.as_ptr(),
            );
            if RtlEqualUnicodeString(
                core::ptr::addr_of_mut!(ustrCurrent),
                pustrDevice as *mut ntkrnl_sys::ddk::_UNICODE_STRING,
                0u8,
            ) != 0
            {
                break;
            }
            pGraphicsDevice = (*pGraphicsDevice).pNextGraphicsDevice;
        }
    } else {
        pGraphicsDevice = gpGraphicsDeviceFirst;
        i = 0;
        while !pGraphicsDevice.is_null() && i < iDevNum {
            pGraphicsDevice = (*pGraphicsDevice).pNextGraphicsDevice;
            i += 1;
        }
    }

    EngReleaseSemaphore(ghsemGraphicsDeviceList);

    pGraphicsDevice
}

unsafe fn EngpFileIoRequest(
    pFileObject: PFILE_OBJECT,
    ulMajorFunction: ULONG,
    lpBuffer: PVOID,
    nBufferSize: SIZE_T,
    ullStartOffset: ULONGLONG,
    lpInformation: PULONG_PTR,
) -> NTSTATUS {
    // DEFERRED-STUB (win32k-translation): translation has cross-crate type / missing-decl issues.
    // TODO: resurrect from the original C in the bottom-of-file source block.
    let _ = (pFileObject, ulMajorFunction, lpBuffer, nBufferSize, ullStartOffset, lpInformation);
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn EngFileWrite(
    pFileObject: PFILE_OBJECT,
    lpBuffer: PVOID,
    nLength: SIZE_T,
    lpBytesWritten: PSIZE_T,
) {
    let status: NTSTATUS;

    status = EngpFileIoRequest(
        pFileObject,
        IRP_MJ_WRITE,
        lpBuffer,
        nLength,
        0,
        lpBytesWritten,
    );
    if !NT_SUCCESS(status) {
        *lpBytesWritten = 0;
    }
}
#[no_mangle]
pub unsafe extern "system" fn EngFileIoControl(
    pFileObject: PFILE_OBJECT,
    dwIoControlCode: u32,
    lpInBuffer: *mut core::ffi::c_void,
    nInBufferSize: usize,
    lpOutBuffer: *mut core::ffi::c_void,
    nOutBufferSize: usize,
    lpInformation: PULONG_PTR,
) -> i32 {
    // DEFERRED-STUB (win32k-translation): translation has cross-crate type / missing-decl issues.
    // TODO: resurrect from the original C in the bottom-of-file source block.
    let _ = (pFileObject, dwIoControlCode, lpInBuffer, nInBufferSize, lpOutBuffer, nOutBufferSize, lpInformation);
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn EngDeviceIoControl(
    hDevice: HANDLE,
    dwIoControlCode: DWORD,
    lpInBuffer: LPVOID,
    cjInBufferSize: DWORD,
    lpOutBuffer: LPVOID,
    cjOutBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
) -> DWORD {
    EngDeviceIoControl_impl(
        hDevice,
        dwIoControlCode,
        lpInBuffer,
        cjInBufferSize,
        lpOutBuffer,
        cjOutBufferSize,
        lpBytesReturned,
    )
}

unsafe fn EngDeviceIoControl_impl(
    hDevice: HANDLE,
    dwIoControlCode: DWORD,
    lpInBuffer: LPVOID,
    cjInBufferSize: DWORD,
    lpOutBuffer: LPVOID,
    cjOutBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
) -> DWORD {
    const ERROR_INVALID_FUNCTION_LOCAL: DWORD = 1;
    const ERROR_INVALID_HANDLE_LOCAL: DWORD = 6;
    const ERROR_NOT_ENOUGH_MEMORY_LOCAL: DWORD = 8;
    const ERROR_DEV_NOT_EXIST_LOCAL: DWORD = 55;
    const ERROR_INVALID_PARAMETER_LOCAL: DWORD = 87;
    const ERROR_INSUFFICIENT_BUFFER_LOCAL: DWORD = 122;
    const ERROR_MORE_DATA_LOCAL: DWORD = 234;
    const ERROR_IO_PENDING_LOCAL: DWORD = 997;

    if hDevice.is_null() {
        return ERROR_INVALID_HANDLE_LOCAL;
    }

    let mut event: ntkrnl_sys::ddk::_KEVENT = core::mem::zeroed();
    let mut iosb: ntkrnl_sys::ddk::_IO_STATUS_BLOCK = core::mem::zeroed();
    let device_object = hDevice as ntkrnl_sys::ddk::PDEVICE_OBJECT;

    KeInitializeEvent(
        core::ptr::addr_of_mut!(event),
        ntkrnl_sys::ddk::_EVENT_TYPE_SynchronizationEvent,
        0,
    );

    let irp = IoBuildDeviceIoControlRequest(
        dwIoControlCode,
        device_object,
        lpInBuffer,
        cjInBufferSize,
        lpOutBuffer,
        cjOutBufferSize,
        0,
        core::ptr::addr_of_mut!(event),
        core::ptr::addr_of_mut!(iosb),
    );
    if irp.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY_LOCAL;
    }

    let mut status = ntkrnl_sys::ddk::IofCallDriver(device_object, irp);
    if status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(event) as *mut core::ffi::c_void,
            ntkrnl_sys::ddk::_KWAIT_REASON_Executive,
            ntkrnl_sys::ddk::_MODE_KernelMode as ntkrnl_sys::ddk::KPROCESSOR_MODE,
            1,
            core::ptr::null_mut(),
        );
        status = iosb.__bindgen_anon_1.Status;
    }

    if !lpBytesReturned.is_null() {
        *lpBytesReturned = iosb.Information as DWORD;
    }

    match status {
        STATUS_INSUFFICIENT_RESOURCES => ERROR_NOT_ENOUGH_MEMORY_LOCAL,
        STATUS_BUFFER_OVERFLOW => ERROR_MORE_DATA_LOCAL,
        STATUS_NOT_IMPLEMENTED => ERROR_INVALID_FUNCTION_LOCAL,
        STATUS_INVALID_PARAMETER => ERROR_INVALID_PARAMETER_LOCAL,
        STATUS_BUFFER_TOO_SMALL => ERROR_INSUFFICIENT_BUFFER_LOCAL,
        STATUS_DEVICE_DOES_NOT_EXIST => ERROR_DEV_NOT_EXIST_LOCAL,
        STATUS_PENDING => ERROR_IO_PENDING_LOCAL,
        _ => status as DWORD,
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           GDI Driver Device Functions
//  * FILE:              win32ss/gdi/eng/device.c
//  * PROGRAMER:         Jason Filby
//  *                    Timo Kreuzer
//  */
//
// #include <win32k.h>
// #include <ntddvdeo.h>
//
// DBG_DEFAULT_CHANNEL(EngDev);
//
// static PGRAPHICS_DEVICE gpPrimaryGraphicsDevice;
// static PGRAPHICS_DEVICE gpVgaGraphicsDevice;
//
// static PGRAPHICS_DEVICE gpGraphicsDeviceFirst = NULL;
// static PGRAPHICS_DEVICE gpGraphicsDeviceLast = NULL;
// static HSEMAPHORE ghsemGraphicsDeviceList;
// static ULONG giDevNum = 1;
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// InitDeviceImpl(VOID)
// {
//     ghsemGraphicsDeviceList = EngCreateSemaphore();
//     if (!ghsemGraphicsDeviceList)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     return STATUS_SUCCESS;
// }
//
// static
// BOOLEAN
// EngpHasVgaDriver(
//     _In_ PGRAPHICS_DEVICE pGraphicsDevice)
// {
//     WCHAR awcDeviceKey[256], awcServiceName[100];
//     PWSTR lastBkSlash;
//     NTSTATUS Status;
//     ULONG cbValue;
//     HKEY hkey;
//
//     /* Open the key for the adapters */
//     Status = RegOpenKey(L"\\Registry\\Machine\\HARDWARE\\DEVICEMAP\\VIDEO", &hkey);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Could not open HARDWARE\\DEVICEMAP\\VIDEO registry key: 0x%08lx\n", Status);
//         return FALSE;
//     }
//
//     /* Read the name of the device key */
//     cbValue = sizeof(awcDeviceKey);
//     Status = RegQueryValue(hkey, pGraphicsDevice->szNtDeviceName, REG_SZ, awcDeviceKey, &cbValue);
//     ZwClose(hkey);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Could not read '%S' registry value: 0x%08lx\n", Status);
//         return FALSE;
//     }
//
//     /* Replace 'DeviceN' by 'Video' */
//     lastBkSlash = wcsrchr(awcDeviceKey, L'\\');
//     if (!lastBkSlash)
//     {
//         ERR("Invalid registry key '%S'\n", lastBkSlash);
//         return FALSE;
//     }
//     if (!NT_SUCCESS(RtlStringCchCopyW(lastBkSlash + 1,
//                                       ARRAYSIZE(awcDeviceKey) - (lastBkSlash + 1 - awcDeviceKey),
//                                       L"Video")))
//     {
//         ERR("Failed to add 'Video' to registry key '%S'\n", awcDeviceKey);
//         return FALSE;
//     }
//
//     /* Open device key */
//     Status = RegOpenKey(awcDeviceKey, &hkey);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Could not open %S registry key: 0x%08lx\n", awcDeviceKey, Status);
//         return FALSE;
//     }
//
//     /* Read service name */
//     cbValue = sizeof(awcServiceName);
//     Status = RegQueryValue(hkey, L"Service", REG_SZ, awcServiceName, &cbValue);
//     ZwClose(hkey);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Could not read Service registry value in %S: 0x%08lx\n", awcDeviceKey, Status);
//         return FALSE;
//     }
//
//     /* Device is using VGA driver if service name is 'VGASave' (case insensitive) */
//     return (_wcsicmp(awcServiceName, L"VGASave") == 0);
// }
//
// /*
//  * Add a device to gpGraphicsDeviceFirst/gpGraphicsDeviceLast list (if not already present).
//  */
// _Requires_lock_held_(ghsemGraphicsDeviceList)
// static
// VOID
// EngpLinkGraphicsDevice(
//     _In_ PGRAPHICS_DEVICE pToAdd)
// {
//     PGRAPHICS_DEVICE pGraphicsDevice;
//
//     TRACE("EngLinkGraphicsDevice(%p)\n", pToAdd);
//
//     /* Search if device is not already linked */
//     for (pGraphicsDevice = gpGraphicsDeviceFirst;
//          pGraphicsDevice;
//          pGraphicsDevice = pGraphicsDevice->pNextGraphicsDevice)
//     {
//         if (pGraphicsDevice == pToAdd)
//             return;
//     }
//
//     pToAdd->pNextGraphicsDevice = NULL;
//     if (gpGraphicsDeviceLast)
//         gpGraphicsDeviceLast->pNextGraphicsDevice = pToAdd;
//     gpGraphicsDeviceLast = pToAdd;
//     if (!gpGraphicsDeviceFirst)
//         gpGraphicsDeviceFirst = pToAdd;
// }
//
// /*
//  * Remove a device from gpGraphicsDeviceFirst/gpGraphicsDeviceLast list.
//  */
// _Requires_lock_held_(ghsemGraphicsDeviceList)
// static
// VOID
// EngpUnlinkGraphicsDevice(
//     _In_ PGRAPHICS_DEVICE pToDelete)
// {
//     PGRAPHICS_DEVICE pPrevGraphicsDevice = NULL;
//     PGRAPHICS_DEVICE pGraphicsDevice = gpGraphicsDeviceFirst;
//
//     TRACE("EngpUnlinkGraphicsDevice('%S')\n", pToDelete->szNtDeviceName);
//
//     while (pGraphicsDevice)
//     {
//         if (pGraphicsDevice != pToDelete)
//         {
//             /* Keep current device */
//             pPrevGraphicsDevice = pGraphicsDevice;
//             pGraphicsDevice = pGraphicsDevice->pNextGraphicsDevice;
//         }
//         else
//         {
//             /* At first, link again associated VGA Device */
//             if (pGraphicsDevice->pVgaDevice)
//                 EngpLinkGraphicsDevice(pGraphicsDevice->pVgaDevice);
//
//             /* We need to remove current device */
//             pGraphicsDevice = pGraphicsDevice->pNextGraphicsDevice;
//
//             /* Unlink chain */
//             if (!pPrevGraphicsDevice)
//                 gpGraphicsDeviceFirst = pToDelete->pNextGraphicsDevice;
//             else
//                 pPrevGraphicsDevice->pNextGraphicsDevice = pToDelete->pNextGraphicsDevice;
//             if (gpGraphicsDeviceLast == pToDelete)
//                 gpGraphicsDeviceLast = pPrevGraphicsDevice;
//         }
//     }
// }
//
// /* Goal of this function is to:
//  * - detect new graphic devices (from registry) and initialize them
//  * - link primary device and VGA device (if available) using pVgaDevice field
//  * - handle gbBaseVideo global flag
//  * - set DISPLAY_DEVICE_PRIMARY_DEVICE on at least one device
//  * - set gpPrimaryGraphicsDevice
//  * - set gpVgaGraphicsDevice
//  */
// NTSTATUS
// EngpUpdateGraphicsDeviceList(VOID)
// {
//     ULONG iDevNum, ulMaxObjectNumber = 0;
//     WCHAR awcDeviceName[20], awcWinDeviceName[20];
//     UNICODE_STRING ustrDeviceName;
//     WCHAR awcBuffer[256];
//     NTSTATUS Status;
//     PGRAPHICS_DEVICE pGraphicsDevice, pNewPrimaryGraphicsDevice = NULL;
//     ULONG cbValue;
//     HKEY hkey;
//
//     /* Open the key for the adapters */
//     Status = RegOpenKey(L"\\Registry\\Machine\\HARDWARE\\DEVICEMAP\\VIDEO", &hkey);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Could not open HARDWARE\\DEVICEMAP\\VIDEO registry key:0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Get the maximum number of adapters */
//     if (!RegReadDWORD(hkey, L"MaxObjectNumber", &ulMaxObjectNumber))
//     {
//         ERR("Could not read MaxObjectNumber, defaulting to 0.\n");
//     }
//
//     ERR("EngpUpdateGraphicsDeviceList: Found %lu devices\n", ulMaxObjectNumber + 1);
//
//     /* Loop through all adapters, to detect new ones */
//     for (iDevNum = 0; iDevNum <= ulMaxObjectNumber; iDevNum++)
//     {
//         /* Create the adapter's key name */
//         swprintf(awcDeviceName, L"\\Device\\Video%lu", iDevNum);
//
//         /* Create the display device name */
//         swprintf(awcWinDeviceName, L"\\\\.\\DISPLAY%lu", iDevNum + 1);
//         RtlInitUnicodeString(&ustrDeviceName, awcWinDeviceName);
//
//         /* Check if the device exists already */
//         pGraphicsDevice = EngpFindGraphicsDevice(&ustrDeviceName, iDevNum);
//         if (pGraphicsDevice != NULL)
//         {
//             continue;
//         }
//
//         /* Read the reg key name */
//         cbValue = sizeof(awcBuffer);
//         Status = RegQueryValue(hkey, awcDeviceName, REG_SZ, awcBuffer, &cbValue);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("EngpUpdateGraphicsDeviceList: failed to query %S: 0x%lx\n", awcDeviceName, Status);
//             continue;
//         }
//
//         ERR("EngpUpdateGraphicsDeviceList: dev %lu = %S -> %S\n", iDevNum, awcDeviceName, awcBuffer);
//
//         /* Initialize the driver for this device */
//         pGraphicsDevice = InitDisplayDriver(awcDeviceName, awcBuffer);
//         if (!pGraphicsDevice)
//         {
//             ERR("EngpUpdateGraphicsDeviceList: InitDisplayDriver failed for %S\n", awcDeviceName);
//             continue;
//         }
//         ERR("EngpUpdateGraphicsDeviceList: registered device %S\n", pGraphicsDevice->szWinDeviceName);
//     }
//
//     /* Close the device map registry key */
//     ZwClose(hkey);
//
//     /* Choose a VGA device */
//     /* Try a device with DISPLAY_DEVICE_VGA_COMPATIBLE flag. If not found,
//      * fall back to current VGA device */
//     for (pGraphicsDevice = gpGraphicsDeviceFirst;
//          pGraphicsDevice;
//          pGraphicsDevice = pGraphicsDevice->pNextGraphicsDevice)
//     {
//         if (pGraphicsDevice == gpVgaGraphicsDevice)
//             continue;
//         if (pGraphicsDevice->StateFlags & DISPLAY_DEVICE_VGA_COMPATIBLE && EngpHasVgaDriver(pGraphicsDevice))
//         {
//             gpVgaGraphicsDevice = pGraphicsDevice;
//             break;
//         }
//     }
//
//     /* Handle gbBaseVideo */
//     if (gbBaseVideo)
//     {
//         PGRAPHICS_DEVICE pToDelete;
//
//         /* Lock list */
//         EngAcquireSemaphore(ghsemGraphicsDeviceList);
//
//         /* Remove every device from linked list, except base-video one */
//         pGraphicsDevice = gpGraphicsDeviceFirst;
//         while (pGraphicsDevice)
//         {
//             if (!EngpHasVgaDriver(pGraphicsDevice))
//             {
//                 /* Not base-video device. Remove it */
//                 pToDelete = pGraphicsDevice;
//                 TRACE("Removing non-base-video device %S (%S)\n", pToDelete->szWinDeviceName, pToDelete->szNtDeviceName);
//
//                 EngpUnlinkGraphicsDevice(pGraphicsDevice);
//                 pGraphicsDevice = pGraphicsDevice->pNextGraphicsDevice;
//
//                 /* Free memory */
//                 ExFreePoolWithTag(pToDelete->pDiplayDrivers, GDITAG_DRVSUP);
//                 ExFreePoolWithTag(pToDelete, GDITAG_GDEVICE);
//             }
//             else
//             {
//                 pGraphicsDevice = pGraphicsDevice->pNextGraphicsDevice;
//             }
//         }
//
//         /* Unlock list */
//         EngReleaseSemaphore(ghsemGraphicsDeviceList);
//     }
//
//     /* Choose a primary device (if none already exists) */
//     if (!gpPrimaryGraphicsDevice)
//     {
//         for (pGraphicsDevice = gpGraphicsDeviceFirst;
//              pGraphicsDevice;
//              pGraphicsDevice = pGraphicsDevice->pNextGraphicsDevice)
//         {
//             if (!EngpHasVgaDriver(pGraphicsDevice))
//             {
//                 pNewPrimaryGraphicsDevice = pGraphicsDevice;
//                 break;
//             }
//         }
//         if (!pNewPrimaryGraphicsDevice)
//             pNewPrimaryGraphicsDevice = gpGraphicsDeviceFirst;
//         if (pNewPrimaryGraphicsDevice)
//         {
//             pNewPrimaryGraphicsDevice->StateFlags |= DISPLAY_DEVICE_PRIMARY_DEVICE;
//             gpPrimaryGraphicsDevice = pNewPrimaryGraphicsDevice;
//         }
//     }
//
//     /* Can we link VGA device to primary device? */
//     if (gpPrimaryGraphicsDevice &&
//         gpVgaGraphicsDevice &&
//         gpPrimaryGraphicsDevice != gpVgaGraphicsDevice &&
//         !gpPrimaryGraphicsDevice->pVgaDevice)
//     {
//         /* Yes. Remove VGA device from global list, and attach it to primary device */
//         TRACE("Linking VGA device %S to primary device %S\n", gpVgaGraphicsDevice->szNtDeviceName, gpPrimaryGraphicsDevice->szNtDeviceName);
//         EngAcquireSemaphore(ghsemGraphicsDeviceList);
//         EngpUnlinkGraphicsDevice(gpVgaGraphicsDevice);
//         gpPrimaryGraphicsDevice->pVgaDevice = gpVgaGraphicsDevice;
//         EngReleaseSemaphore(ghsemGraphicsDeviceList);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /* Open display settings registry key
//  * Returns NULL in case of error. */
// static HKEY
// EngpGetRegistryHandleFromDeviceMap(
//     _In_ PGRAPHICS_DEVICE pGraphicsDevice)
// {
//     static const PWCHAR KEY_VIDEO = L"\\Registry\\Machine\\HARDWARE\\DEVICEMAP\\VIDEO";
//     HKEY hKey;
//     WCHAR szDeviceKey[256];
//     ULONG cbSize;
//     NTSTATUS Status;
//
//     /* Open the device map registry key */
//     Status = RegOpenKey(KEY_VIDEO, &hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Could not open HARDWARE\\DEVICEMAP\\VIDEO registry key: status 0x%08x\n", Status);
//         return NULL;
//     }
//
//     /* Query the registry path */
//     cbSize = sizeof(szDeviceKey);
//     RegQueryValue(hKey,
//                   pGraphicsDevice->szNtDeviceName,
//                   REG_SZ,
//                   szDeviceKey,
//                   &cbSize);
//     ZwClose(hKey);
//
//     /* Open the registry key */
//     Status = RegOpenKey(szDeviceKey, &hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Could not open registry key '%S': status 0x%08x\n", szDeviceKey, Status);
//         return NULL;
//     }
//
//     return hKey;
// }
//
// NTSTATUS
// EngpGetDisplayDriverParameters(
//     _In_ PGRAPHICS_DEVICE pGraphicsDevice,
//     _Out_ PDEVMODEW pdm)
// {
//     HKEY hKey;
//     NTSTATUS Status;
//     RTL_QUERY_REGISTRY_TABLE DisplaySettingsTable[] =
//     {
// #define READ(field, str) \
//         { \
//             NULL, \
//             RTL_QUERY_REGISTRY_DIRECT, \
//             L ##str, \
//             &pdm->field, \
//             REG_NONE, NULL, 0 \
//         },
//     READ(dmBitsPerPel, "DefaultSettings.BitsPerPel")
//     READ(dmPelsWidth, "DefaultSettings.XResolution")
//     READ(dmPelsHeight, "DefaultSettings.YResolution")
//     READ(dmDisplayFlags, "DefaultSettings.Flags")
//     READ(dmDisplayFrequency, "DefaultSettings.VRefresh")
//     READ(dmPanningWidth, "DefaultSettings.XPanning")
//     READ(dmPanningHeight, "DefaultSettings.YPanning")
//     READ(dmDisplayOrientation, "DefaultSettings.Orientation")
//     READ(dmDisplayFixedOutput, "DefaultSettings.FixedOutput")
//     READ(dmPosition.x, "Attach.RelativeX")
//     READ(dmPosition.y, "Attach.RelativeY")
// #undef READ
//         {0}
//     };
//
//     hKey = EngpGetRegistryHandleFromDeviceMap(pGraphicsDevice);
//     if (!hKey)
//         return STATUS_UNSUCCESSFUL;
//
//     Status = RtlQueryRegistryValues(RTL_REGISTRY_HANDLE,
//                                     (PWSTR)hKey,
//                                     DisplaySettingsTable,
//                                     NULL,
//                                     NULL);
//
//     ZwClose(hKey);
//     return Status;
// }
//
// DWORD
// EngpGetDisplayDriverAccelerationLevel(
//     _In_ PGRAPHICS_DEVICE pGraphicsDevice)
// {
//     HKEY hKey;
//     DWORD dwAccelerationLevel = 0;
//     RTL_QUERY_REGISTRY_TABLE DisplaySettingsTable[] =
//     {
//         {
//             NULL,
//             RTL_QUERY_REGISTRY_DIRECT,
//             L"Acceleration.Level",
//             &dwAccelerationLevel,
//             REG_NONE, NULL, 0
//         },
//         {0}
//     };
//
//     hKey = EngpGetRegistryHandleFromDeviceMap(pGraphicsDevice);
//     if (!hKey)
//         return 0;
//
//     RtlQueryRegistryValues(RTL_REGISTRY_HANDLE,
//                            (PWSTR)hKey,
//                            DisplaySettingsTable,
//                            NULL,
//                            NULL);
//     ZwClose(hKey);
//
//     return dwAccelerationLevel;
// }
//
// extern VOID
// UserRefreshDisplay(IN PPDEVOBJ ppdev);
//
// // PVIDEO_WIN32K_CALLOUT
// VOID
// NTAPI
// VideoPortCallout(
//     _In_ PVOID Params)
// {
// /*
//  * IMPORTANT NOTICE!! On Windows XP/2003 this function triggers the creation of
//  * a specific VideoPortCalloutThread() system thread using the same mechanism
//  * as the RIT/desktop/Ghost system threads.
//  */
//
//     PVIDEO_WIN32K_CALLBACKS_PARAMS CallbackParams = (PVIDEO_WIN32K_CALLBACKS_PARAMS)Params;
//
//     TRACE("VideoPortCallout(0x%p, 0x%x)\n",
//           CallbackParams, CallbackParams ? CallbackParams->CalloutType : -1);
//
//     if (!CallbackParams)
//         return;
//
//     switch (CallbackParams->CalloutType)
//     {
//         case VideoFindAdapterCallout:
//         {
//             TRACE("VideoPortCallout: VideoFindAdapterCallout called - Param = %s\n",
//                   CallbackParams->Param ? "TRUE" : "FALSE");
//             if (CallbackParams->Param == TRUE)
//             {
//                 /* Re-enable the display */
//                 UserRefreshDisplay(gpmdev->ppdevGlobal);
//             }
//             else
//             {
//                 /* Disable the display */
//                 NOTHING; // Nothing to do for the moment...
//             }
//
//             CallbackParams->Status = STATUS_SUCCESS;
//             break;
//         }
//
//         case VideoPowerNotifyCallout:
//         case VideoDisplaySwitchCallout:
//         case VideoEnumChildPdoNotifyCallout:
//         case VideoWakeupCallout:
//         case VideoChangeDisplaySettingsCallout:
//         case VideoPnpNotifyCallout:
//         case VideoDxgkDisplaySwitchCallout:
//         case VideoDxgkMonitorEventCallout:
//         case VideoDxgkFindAdapterTdrCallout:
//             ERR("VideoPortCallout: CalloutType 0x%x is UNIMPLEMENTED!\n", CallbackParams->CalloutType);
//             CallbackParams->Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         default:
//             ERR("VideoPortCallout: Unknown CalloutType 0x%x\n", CallbackParams->CalloutType);
//             CallbackParams->Status = STATUS_UNSUCCESSFUL;
//             break;
//     }
// }
//
// /* Sends a TargetDeviceRelation request to PDO
//  * On success, caller needs to free pDeviceRelations with ExFreePool()
//  */
// static
// NTSTATUS
// EngpPnPTargetRelationRequest(
//     _In_ PDEVICE_OBJECT pDeviceObject,
//     _Out_ PDEVICE_RELATIONS *pDeviceRelations)
// {
//     PIO_STACK_LOCATION IrpSp;
//     KEVENT Event;
//     PIRP pIrp;
//     IO_STATUS_BLOCK Iosb;
//     NTSTATUS Status;
//
//     /* Initialize an event */
//     KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//
//     /* Build IRP */
//     pIrp = IoBuildSynchronousFsdRequest(IRP_MJ_PNP,
//                                         pDeviceObject,
//                                         NULL, 0,
//                                         NULL,
//                                         &Event,
//                                         &Iosb);
//     if (!pIrp)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Initialize IRP */
//     pIrp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//     IrpSp = IoGetNextIrpStackLocation(pIrp);
//     IrpSp->MinorFunction = IRP_MN_QUERY_DEVICE_RELATIONS;
//     IrpSp->Parameters.QueryDeviceRelations.Type = TargetDeviceRelation;
//
//     /* Call the driver */
//     Status = IoCallDriver(pDeviceObject, pIrp);
//
//     /* Wait if neccessary */
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, 0);
//         Status = Iosb.Status;
//     }
//
//     /* Return information to the caller about the operation. */
//     if (NT_SUCCESS(Status))
//         *pDeviceRelations = (PDEVICE_RELATIONS)Iosb.Information;
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// EngpUpdateMonitorDevices(
//     _In_ PGRAPHICS_DEVICE pGraphicsDevice)
// {
//     PDEVICE_RELATIONS pDeviceRelations;
//     PVIDEO_MONITOR_DEVICE pMonitorDevices;
//     ULONG i, bytesWritten, monitorCount;
//     NTSTATUS Status;
//     HANDLE hkRegistry;
//
//     /* Request right PDO for device relations */
//     Status = EngpPnPTargetRelationRequest(pGraphicsDevice->DeviceObject, &pDeviceRelations);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("EngpPnPTargetRelationRequest() failed with status 0x%08x\n", Status);
//         return Status;
//     }
//     ASSERT(pDeviceRelations->Count == 1);
//
//     /* Invalidate relations, so that videoprt reenumerates its monitors.
//      * Only do this for valid PDOs - check by trying to open registry key. */
//     Status = IoOpenDeviceRegistryKey(pDeviceRelations->Objects[0],
//                                      PLUGPLAY_REGKEY_DRIVER,
//                                      KEY_READ,
//                                      &hkRegistry);
//     if (NT_SUCCESS(Status))
//     {
//         ZwClose(hkRegistry);
//         IoSynchronousInvalidateDeviceRelations(pDeviceRelations->Objects[0], BusRelations);
//     }
//     else
//     {
//         /* Legacy device without valid PDO - skip invalidation.
//          * This is expected for non-PnP devices like VGA. */
//     }
//
//     /* Free returned structure */
//     for (i = 0; i < pDeviceRelations->Count; i++)
//         ObDereferenceObject(pDeviceRelations->Objects[i]);
//     ExFreePool(pDeviceRelations);
//
//     /* Now, get list of monitor PDOs */
//     Status = EngDeviceIoControl(pGraphicsDevice->DeviceObject,
//                                 IOCTL_VIDEO_ENUM_MONITOR_PDO,
//                                 NULL, 0,
//                                 &pMonitorDevices, sizeof(pMonitorDevices),
//                                 &bytesWritten);
//     if (Status != ERROR_SUCCESS)
//     {
//         ERR("EngDeviceIoControl(IOCTL_VIDEO_ENUM_MONITOR_PDO) failed with status 0x%08x\n", Status);
//         return Status;
//     }
//     ASSERT(bytesWritten == sizeof(pMonitorDevices));
//
//     /* Count number of available monitors */
//     for (monitorCount = 0; pMonitorDevices[monitorCount].pdo; ++monitorCount)
//         ;
//
//     if (pGraphicsDevice->pvMonDev)
//     {
//         /* Erase everything */
//         for (i = 0; i < pGraphicsDevice->dwMonCnt; i++)
//             ObDereferenceObject(pGraphicsDevice->pvMonDev[i].pdo);
//         ExFreePoolWithTag(pGraphicsDevice->pvMonDev, GDITAG_GDEVICE);
//         pGraphicsDevice->pvMonDev = NULL;
//         pGraphicsDevice->dwMonCnt = 0;
//     }
//
//     if (monitorCount > 0)
//     {
//         pGraphicsDevice->pvMonDev = ExAllocatePoolZero(PagedPool,
//                                                        monitorCount * sizeof(VIDEO_MONITOR_DEVICE),
//                                                        GDITAG_GDEVICE);
//         if (!pGraphicsDevice->pvMonDev)
//         {
//             for (i = 0; pMonitorDevices[i].pdo; ++i)
//                 ObDereferenceObject(pMonitorDevices[i].pdo);
//             ExFreePool(pMonitorDevices);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Copy data */
//         for (i = 0; i < monitorCount; i++)
//         {
//             TRACE("%S\\Monitor%u: PDO %p HwID %u\n", pGraphicsDevice->szWinDeviceName, i, pMonitorDevices[i].pdo, pMonitorDevices[i].HwID);
//             pGraphicsDevice->pvMonDev[pGraphicsDevice->dwMonCnt++] = pMonitorDevices[i];
//         }
//     }
//
//     ExFreePool(pMonitorDevices);
//     return STATUS_SUCCESS;
// }
//
// PGRAPHICS_DEVICE
// NTAPI
// EngpRegisterGraphicsDevice(
//     _In_ PUNICODE_STRING pustrDeviceName,
//     _In_ PUNICODE_STRING pustrDiplayDrivers,
//     _In_ PUNICODE_STRING pustrDescription)
// {
//     PGRAPHICS_DEVICE pGraphicsDevice;
//     PDEVICE_OBJECT pDeviceObject;
//     PFILE_OBJECT pFileObject;
//     NTSTATUS Status;
//     VIDEO_WIN32K_CALLBACKS Win32kCallbacks;
//     ULONG ulReturn;
//     PWSTR pwsz;
//     ULONG cj;
//
//     TRACE("EngpRegisterGraphicsDevice(%wZ)\n", pustrDeviceName);
//
//     /* Allocate a GRAPHICS_DEVICE structure */
//     pGraphicsDevice = ExAllocatePoolZero(PagedPool,
//                                          sizeof(GRAPHICS_DEVICE),
//                                          GDITAG_GDEVICE);
//     if (!pGraphicsDevice)
//     {
//         ERR("ExAllocatePoolWithTag failed\n");
//         return NULL;
//     }
//
//     /* Try to open and enable the device */
//     Status = IoGetDeviceObjectPointer(pustrDeviceName,
//                                       FILE_READ_DATA | FILE_WRITE_DATA,
//                                       &pFileObject,
//                                       &pDeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Could not open device %wZ, 0x%lx\n", pustrDeviceName, Status);
//         ExFreePoolWithTag(pGraphicsDevice, GDITAG_GDEVICE);
//         return NULL;
//     }
//
//     /* Copy the device and file object pointers */
//     pGraphicsDevice->DeviceObject = pDeviceObject;
//     pGraphicsDevice->FileObject = pFileObject;
//
//     /* Initialize and register the device with videoprt for Win32k callbacks */
//     Win32kCallbacks.PhysDisp = pGraphicsDevice;
//     Win32kCallbacks.Callout = VideoPortCallout;
//     // Reset the data being returned prior to the call.
//     Win32kCallbacks.bACPI = FALSE;
//     Win32kCallbacks.pPhysDeviceObject = NULL;
//     Win32kCallbacks.DualviewFlags = 0;
//     Status = (NTSTATUS)EngDeviceIoControl((HANDLE)pDeviceObject,
//                                           IOCTL_VIDEO_INIT_WIN32K_CALLBACKS,
//                                           &Win32kCallbacks,
//                                           sizeof(Win32kCallbacks),
//                                           &Win32kCallbacks,
//                                           sizeof(Win32kCallbacks),
//                                           &ulReturn);
//     if (Status != ERROR_SUCCESS)
//     {
//         ERR("EngDeviceIoControl(0x%p, IOCTL_VIDEO_INIT_WIN32K_CALLBACKS) failed, Status 0x%lx\n",
//             pDeviceObject, Status);
//     }
//     // TODO: Set flags according to the results.
//     // if (Win32kCallbacks.bACPI)
//     // if (Win32kCallbacks.DualviewFlags & ???)
//     pGraphicsDevice->PhysDeviceHandle = Win32kCallbacks.pPhysDeviceObject;
//
//     /* Copy the device name */
//     RtlStringCbCopyNW(pGraphicsDevice->szNtDeviceName,
//                       sizeof(pGraphicsDevice->szNtDeviceName),
//                       pustrDeviceName->Buffer,
//                       pustrDeviceName->Length);
//
//     /* Create a Win32 device name (FIXME: virtual devices!) */
//     RtlStringCbPrintfW(pGraphicsDevice->szWinDeviceName,
//                        sizeof(pGraphicsDevice->szWinDeviceName),
//                        L"\\\\.\\DISPLAY%d",
//                        (int)giDevNum);
//
//     /* Allocate a buffer for the strings */
//     cj = pustrDiplayDrivers->Length + pustrDescription->Length + sizeof(WCHAR);
//     pwsz = ExAllocatePoolWithTag(PagedPool, cj, GDITAG_DRVSUP);
//     if (!pwsz)
//     {
//         ERR("Could not allocate string buffer\n");
//         ASSERT(FALSE); // FIXME
//         ExFreePoolWithTag(pGraphicsDevice, GDITAG_GDEVICE);
//         return NULL;
//     }
//
//     /* Copy the display driver names */
//     pGraphicsDevice->pDiplayDrivers = pwsz;
//     RtlCopyMemory(pGraphicsDevice->pDiplayDrivers,
//                   pustrDiplayDrivers->Buffer,
//                   pustrDiplayDrivers->Length);
//
//     /* Copy the description */
//     pGraphicsDevice->pwszDescription = pwsz + pustrDiplayDrivers->Length / sizeof(WCHAR);
//     RtlCopyMemory(pGraphicsDevice->pwszDescription,
//                   pustrDescription->Buffer,
//                   pustrDescription->Length);
//     pGraphicsDevice->pwszDescription[pustrDescription->Length/sizeof(WCHAR)] = 0;
//
//     /* Update list of connected monitors */
//     EngpUpdateMonitorDevices(pGraphicsDevice);
//
//     /* Lock loader */
//     EngAcquireSemaphore(ghsemGraphicsDeviceList);
//
//     /* Insert the device into the global list */
//     EngpLinkGraphicsDevice(pGraphicsDevice);
//
//     /* Increment the device number */
//     giDevNum++;
//
//     /* Unlock loader */
//     EngReleaseSemaphore(ghsemGraphicsDeviceList);
//
//     /* HACK: already in graphic mode; display wallpaper on this new display */
//     if (ScreenDeviceContext)
//     {
//         UNICODE_STRING DriverName = RTL_CONSTANT_STRING(L"DISPLAY");
//         UNICODE_STRING DisplayName;
//         HDC hdc;
//         RtlInitUnicodeString(&DisplayName, pGraphicsDevice->szWinDeviceName);
//         hdc = IntGdiCreateDC(&DriverName, &DisplayName, NULL, NULL, FALSE);
//         IntPaintDesktop(hdc);
//     }
//
//     return pGraphicsDevice;
// }
//
// PGRAPHICS_DEVICE
// NTAPI
// EngpFindGraphicsDevice(
//     _In_opt_ PUNICODE_STRING pustrDevice,
//     _In_ ULONG iDevNum)
// {
//     UNICODE_STRING ustrCurrent;
//     PGRAPHICS_DEVICE pGraphicsDevice;
//     ULONG i;
//     TRACE("EngpFindGraphicsDevice('%wZ', %lu)\n",
//            pustrDevice, iDevNum);
//
//     /* Lock list */
//     EngAcquireSemaphore(ghsemGraphicsDeviceList);
//
//     if (pustrDevice && pustrDevice->Buffer)
//     {
//         /* Find specified video adapter by name */
//         for (pGraphicsDevice = gpGraphicsDeviceFirst;
//              pGraphicsDevice;
//              pGraphicsDevice = pGraphicsDevice->pNextGraphicsDevice)
//         {
//             /* Compare the device name */
//             RtlInitUnicodeString(&ustrCurrent, pGraphicsDevice->szWinDeviceName);
//             if (RtlEqualUnicodeString(&ustrCurrent, pustrDevice, FALSE))
//             {
//                 break;
//             }
//         }
//
//         if (pGraphicsDevice)
//         {
//             /* Validate selected monitor number */
// #if 0
//             if (iDevNum >= pGraphicsDevice->dwMonCnt)
//                 pGraphicsDevice = NULL;
// #else
//             /* FIXME: dwMonCnt not initialized, see EngpRegisterGraphicsDevice */
// #endif
//         }
//     }
//     else
//     {
//         /* Select video adapter by device number */
//         for (pGraphicsDevice = gpGraphicsDeviceFirst, i = 0;
//              pGraphicsDevice && i < iDevNum;
//              pGraphicsDevice = pGraphicsDevice->pNextGraphicsDevice, i++);
//     }
//
//     /* Unlock list */
//     EngReleaseSemaphore(ghsemGraphicsDeviceList);
//
//     return pGraphicsDevice;
// }
//
// static
// NTSTATUS
// EngpFileIoRequest(
//     _In_ PFILE_OBJECT pFileObject,
//     _In_ ULONG ulMajorFunction,
//     _In_reads_(nBufferSize) PVOID lpBuffer,
//     _In_ SIZE_T nBufferSize,
//     _In_ ULONGLONG ullStartOffset,
//     _Out_ PULONG_PTR lpInformation)
// {
//     PDEVICE_OBJECT pDeviceObject;
//     KEVENT Event;
//     PIRP pIrp;
//     IO_STATUS_BLOCK Iosb;
//     NTSTATUS Status;
//     LARGE_INTEGER liStartOffset;
//
//     /* Get corresponding device object */
//     pDeviceObject = IoGetRelatedDeviceObject(pFileObject);
//     if (!pDeviceObject)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Initialize an event */
//     KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//
//     /* Build IRP */
//     liStartOffset.QuadPart = ullStartOffset;
//     pIrp = IoBuildSynchronousFsdRequest(ulMajorFunction,
//                                         pDeviceObject,
//                                         lpBuffer,
//                                         (ULONG)nBufferSize,
//                                         &liStartOffset,
//                                         &Event,
//                                         &Iosb);
//     if (!pIrp)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Call the driver */
//     Status = IoCallDriver(pDeviceObject, pIrp);
//
//     /* Wait if neccessary */
//     if (STATUS_PENDING == Status)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, TRUE, 0);
//         Status = Iosb.Status;
//     }
//
//     /* Return information to the caller about the operation. */
//     *lpInformation = Iosb.Information;
//
//     /* Return NTSTATUS */
//     return Status;
// }
//
// VOID
// APIENTRY
// EngFileWrite(
//     _In_ PFILE_OBJECT pFileObject,
//     _In_reads_(nLength) PVOID lpBuffer,
//     _In_ SIZE_T nLength,
//     _Out_ PSIZE_T lpBytesWritten)
// {
//     NTSTATUS status;
//
//     status = EngpFileIoRequest(pFileObject,
//                                IRP_MJ_WRITE,
//                                lpBuffer,
//                                nLength,
//                                0,
//                                lpBytesWritten);
//     if (!NT_SUCCESS(status))
//     {
//         *lpBytesWritten = 0;
//     }
// }
//
// _Success_(return>=0)
// NTSTATUS
// APIENTRY
// EngFileIoControl(
//     _In_ PFILE_OBJECT pFileObject,
//     _In_ DWORD dwIoControlCode,
//     _In_reads_(nInBufferSize) PVOID lpInBuffer,
//     _In_ SIZE_T nInBufferSize,
//     _Out_writes_(nOutBufferSize) PVOID lpOutBuffer,
//     _In_ SIZE_T nOutBufferSize,
//     _Out_ PULONG_PTR lpInformation)
// {
//     PDEVICE_OBJECT pDeviceObject;
//     KEVENT Event;
//     PIRP pIrp;
//     IO_STATUS_BLOCK Iosb;
//     NTSTATUS Status;
//
//     /* Get corresponding device object */
//     pDeviceObject = IoGetRelatedDeviceObject(pFileObject);
//     if (!pDeviceObject)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Initialize an event */
//     KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//
//     /* Build IO control IRP */
//     pIrp = IoBuildDeviceIoControlRequest(dwIoControlCode,
//                                          pDeviceObject,
//                                          lpInBuffer,
//                                          (ULONG)nInBufferSize,
//                                          lpOutBuffer,
//                                          (ULONG)nOutBufferSize,
//                                          FALSE,
//                                          &Event,
//                                          &Iosb);
//     if (!pIrp)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Call the driver */
//     Status = IoCallDriver(pDeviceObject, pIrp);
//
//     /* Wait if neccessary */
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, TRUE, 0);
//         Status = Iosb.Status;
//     }
//
//     /* Return information to the caller about the operation. */
//     *lpInformation = Iosb.Information;
//
//     /* This function returns NTSTATUS */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// _Success_(return==0)
// DWORD
// APIENTRY
// EngDeviceIoControl(
//     _In_ HANDLE hDevice,
//     _In_ DWORD dwIoControlCode,
//     _In_reads_bytes_opt_(cjInBufferSize) LPVOID lpInBuffer,
//     _In_ DWORD cjInBufferSize,
//     _Out_writes_bytes_opt_(cjOutBufferSize) LPVOID lpOutBuffer,
//     _In_ DWORD cjOutBufferSize,
//     _Out_ LPDWORD lpBytesReturned)
// {
//     PIRP Irp;
//     NTSTATUS Status;
//     KEVENT Event;
//     IO_STATUS_BLOCK Iosb;
//     PDEVICE_OBJECT DeviceObject;
//
//     TRACE("EngDeviceIoControl() called\n");
//
//     if (!hDevice)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//
//     DeviceObject = (PDEVICE_OBJECT) hDevice;
//
//     Irp = IoBuildDeviceIoControlRequest(dwIoControlCode,
//                                         DeviceObject,
//                                         lpInBuffer,
//                                         cjInBufferSize,
//                                         lpOutBuffer,
//                                         cjOutBufferSize,
//                                         FALSE,
//                                         &Event,
//                                         &Iosb);
//     if (!Irp) return ERROR_NOT_ENOUGH_MEMORY;
//
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         (VOID)KeWaitForSingleObject(&Event, Executive, KernelMode, TRUE, 0);
//         Status = Iosb.Status;
//     }
//
//     TRACE("EngDeviceIoControl(): Returning %X/%X\n", Iosb.Status,
//            Iosb.Information);
//
//     /* Return information to the caller about the operation. */
//     *lpBytesReturned = (DWORD)Iosb.Information;
//
//     /* Convert NT status values to win32 error codes. */
//     switch (Status)
//     {
//         case STATUS_INSUFFICIENT_RESOURCES:
//             return ERROR_NOT_ENOUGH_MEMORY;
//
//         case STATUS_BUFFER_OVERFLOW:
//             return ERROR_MORE_DATA;
//
//         case STATUS_NOT_IMPLEMENTED:
//             return ERROR_INVALID_FUNCTION;
//
//         case STATUS_INVALID_PARAMETER:
//             return ERROR_INVALID_PARAMETER;
//
//         case STATUS_BUFFER_TOO_SMALL:
//             return ERROR_INSUFFICIENT_BUFFER;
//
//         case STATUS_DEVICE_DOES_NOT_EXIST:
//             return ERROR_DEV_NOT_EXIST;
//
//         case STATUS_PENDING:
//             return ERROR_IO_PENDING;
//     }
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
