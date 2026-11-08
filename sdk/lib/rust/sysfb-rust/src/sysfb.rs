//! sysfb.sys — standalone GOP/linear-framebuffer video device driver.
//! Rust translation of `drivers/video/sysfb/sysfb.c`.
//!
//! See `lib.rs` for the cohabitation/role overview. One C file → one
//! module. Built for all three arches (live UEFI framebuffer driver).

use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    IRP, PIRP, PDEVICE_OBJECT, PDRIVER_OBJECT, PUNICODE_STRING, PDRIVER_DISPATCH,
    PHYSICAL_ADDRESS, _UNICODE_STRING,
    OBJECT_ATTRIBUTES,
    CM_FULL_RESOURCE_DESCRIPTOR, PCM_FULL_RESOURCE_DESCRIPTOR,
    CM_PARTIAL_RESOURCE_LIST, PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    PKEY_VALUE_PARTIAL_INFORMATION,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation,
    IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink,
    IofCompleteRequest,
    ZwCreateKey, ZwOpenKey, ZwClose, ZwSetValueKey, ZwQueryValueKey,
    ExAllocatePoolWithTag, ExFreePoolWithTag,
    MmMapIoSpace, MmUnmapIoSpace,
    RtlInitUnicodeString, RtlFreeUnicodeString,
    // PnP
    GUID, PCM_RESOURCE_LIST, CM_PARTIAL_RESOURCE_DESCRIPTOR,
    IoAttachDeviceToDeviceStack, IoDetachDevice, IoForwardIrpSynchronously,
    IoReportDetectedDevice, IoRegisterDeviceInterface, IoSetDeviceInterfaceState,
    ZwDeleteValueKey,
    _INTERFACE_TYPE_InterfaceTypeUndefined as InterfaceTypeUndefined,
    SYSTEM_POWER_STATE,
    _POWER_STATE_TYPE_SystemPowerState as SystemPowerState,
    _SYSTEM_POWER_STATE_PowerSystemSleeping1 as PowerSystemSleeping1,
    _SYSTEM_POWER_STATE_PowerSystemHibernate as PowerSystemHibernate,
};
use ntkrnl_sys::driver_ddk::{
    IoCallDriver, PoCallDriver, PoStartNextPowerIrp,
    IRP_MJ_PNP, IRP_MJ_POWER,
    IRP_MN_START_DEVICE, IRP_MN_QUERY_REMOVE_DEVICE, IRP_MN_REMOVE_DEVICE,
    IRP_MN_CANCEL_REMOVE_DEVICE, IRP_MN_STOP_DEVICE, IRP_MN_QUERY_STOP_DEVICE,
    IRP_MN_CANCEL_STOP_DEVICE, IRP_MN_SURPRISE_REMOVAL, IRP_MN_QUERY_POWER,
    DO_DEVICE_INITIALIZING, DO_POWER_PAGABLE, FILE_DEVICE_SECURE_OPEN,
};
// FORCEINLINE DDK helpers — bindgen surfaces these as extern decls with no
// exported symbol; use ntkrnl-sys's real inline impls (AGENTS.md gotcha).
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation};
use ntkrnl_sys::inline::common::InitializeObjectAttributes;
use core::ffi::c_void;
use core::sync::atomic::{AtomicI32, Ordering};

// ---- integration shims (mechanical) ----
extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
}

/// `DPRINT1(...)` — the C used `#define DPRINT1 DbgPrint`. We keep the call
/// sites readable while terminating each format string with NUL.
macro_rules! DPRINT1 {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        DbgPrint(concat!($fmt, "\0").as_ptr() as *const i8 $(, $arg)*)
    };
}

/// C `L"..."` → NUL-terminated UTF-16, evaluated at compile time. These
/// strings are pure ASCII so a per-byte widen is exact and lossless.
/// (Same idiom as arm64input-rust; bindgen's RtlInitUnicodeString takes
/// `PCWSTR` = `*const u16`, so a `b".."` byte string would be misread.)
macro_rules! wsz {
    ($s:literal) => {{
        const B: &[u8] = $s;
        const N: usize = B.len();
        const W: [u16; N + 1] = {
            let mut w = [0u16; N + 1];
            let mut i = 0;
            while i < N {
                w[i] = B[i] as u16;
                i += 1;
            }
            w
        };
        W
    }};
}

/// IoCompleteRequest is a C macro aliasing IofCompleteRequest.
#[inline]
unsafe fn IoCompleteRequest(irp: PIRP, boost: i8) {
    IofCompleteRequest(irp, boost as _);
}

/// `CTL_CODE(t,f,m,a)` from winioctl.h, evaluated at compile time so the
/// IOCTL constants are provably the same values the C header produced
/// (avoids the wrong-magic-constant class of bug — see memory
/// `feedback_rustconv_wrong_constant_value`). All video IOCTLs use
/// METHOD_BUFFERED(0) + FILE_ANY_ACCESS(0).
const fn ctl_code(devtype: u32, func: u32, method: u32, access: u32) -> u32 {
    (devtype << 16) | (access << 14) | (func << 2) | method
}
const FILE_DEVICE_VIDEO: u32 = 0x00000023;

// IOCTL_VIDEO_* — function numbers taken verbatim from sdk/include/psdk/ntddvdeo.h.
const IOCTL_VIDEO_INIT_WIN32K_CALLBACKS: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x07, 0, 0);
const IOCTL_VIDEO_QUERY_AVAIL_MODES: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x100, 0, 0);
const IOCTL_VIDEO_QUERY_NUM_AVAIL_MODES: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x101, 0, 0);
const IOCTL_VIDEO_QUERY_CURRENT_MODE: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x102, 0, 0);
const IOCTL_VIDEO_SET_CURRENT_MODE: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x103, 0, 0);
const IOCTL_VIDEO_RESET_DEVICE: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x104, 0, 0);
const IOCTL_VIDEO_SET_COLOR_REGISTERS: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x107, 0, 0);
const IOCTL_VIDEO_MAP_VIDEO_MEMORY: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x116, 0, 0);
const IOCTL_VIDEO_UNMAP_VIDEO_MEMORY: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x117, 0, 0);
const IOCTL_VIDEO_QUERY_COLOR_CAPABILITIES: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x11a, 0, 0);

// VIDEO mode attribute flags (sdk/include/psdk/ntddvdeo.h)
const VIDEO_MODE_COLOR: u32 = 0x0001;
const VIDEO_MODE_GRAPHICS: u32 = 0x0002;
const VIDEO_MODE_LINEAR: u32 = 0x0100;
const VIDEO_DEVICE_COLOR: u32 = 0x00000001;

const FILE_DEVICE_VIDEO_TYPE: u32 = 0x00000023; // FILE_DEVICE_VIDEO for IoCreateDevice
const TAG_SYSFB: u32 = u32::from_le_bytes(*b"SyFb"); // 'bFyS' multi-char → LE bytes

// CM_RESOURCE_TYPE values (sdk/include/xdk/cmtypes.h). Authoritative — NOT
// guessed: DeviceSpecific=5 is the exact constant whose wrong value (0x81)
// once broke this driver's framebuffer handoff (BUGCHECK 0xB4, see memory
// feedback_rustconv_wrong_constant_value). The freeldr writer side
// (freeldr-rust/src/pchw.rs) stamps these same bindgen values. CM_PARTIAL_
// RESOURCE_DESCRIPTOR.Type is a UCHAR, so the reader compares against u8.
const CmResourceTypeMemory: u8 = 3;
const CmResourceTypeDeviceSpecific: u8 = 5;
// A BAR past 4 GiB: same descriptor layout as Memory, with Length scaled by the flag.
const CmResourceTypeMemoryLarge: u8 = 7;
const CM_RESOURCE_MEMORY_LARGE_40: u16 = 0x0200;
const CM_RESOURCE_MEMORY_LARGE_48: u16 = 0x0400;
const CM_RESOURCE_MEMORY_LARGE_64: u16 = 0x0800;

// ntstatus.h values the shared constants do not carry.
const STATUS_DEVICE_BUSY: NTSTATUS = 0x80000011_u32 as i32;
const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000E_u32 as i32;
const STATUS_DEVICE_CONFIGURATION_ERROR: NTSTATUS = 0xC0000182_u32 as i32;

/// GUID_DEVINTERFACE_DISPLAY_ADAPTER, {5b45201d-f2f2-4f3b-85bb-30ff1f953599} (ntddvdeo.h).
static GUID_DEVINTERFACE_DISPLAY_ADAPTER: GUID = GUID {
    Data1: 0x5b45201d,
    Data2: 0xf2f2,
    Data3: 0x4f3b,
    Data4: [0x85, 0xbb, 0x30, 0xff, 0x1f, 0x95, 0x35, 0x99],
};

// Registry / IRP constants absent from ntkrnl-sys constants.rs (present only
// in hal_bindgen, which isn't a dependency here). Values from xdk headers.
const KEY_ALL_ACCESS: u32 = 0x000F003F; // 983103
const REG_OPTION_VOLATILE: u32 = 1;
const IRP_MJ_DEVICE_CONTROL: usize = 0x0E; // 14

/// Driver dispatch routine ABI. The bindgen `PDRIVER_DISPATCH` is nominally
/// `extern "stdcall"` on i386 / `extern "C"` on amd64+arm64; `extern
/// "system"` resolves to the same per-arch calling convention but the type
/// checker won't unify the literal ABI strings. So the dispatch fns are
/// `extern "system"` and transmuted to `PDRIVER_DISPATCH` at the
/// MajorFunction store (identical codegen, established pattern in this tree).
type DispatchFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS;
type AddDeviceFn = unsafe extern "system" fn(PDRIVER_OBJECT, PDEVICE_OBJECT) -> NTSTATUS;

// ---- in-module video DDK structs (ntddvdeo.h types absent from bindgen) ----

#[repr(C)]
pub struct VIDEO_MODE_INFORMATION {
    pub Length: ULONG,
    pub ModeIndex: ULONG,
    pub VisScreenWidth: ULONG,
    pub VisScreenHeight: ULONG,
    pub ScreenStride: ULONG,
    pub NumberOfPlanes: ULONG,
    pub BitsPerPlane: ULONG,
    pub Frequency: ULONG,
    pub XMillimeter: ULONG,
    pub YMillimeter: ULONG,
    pub NumberRedBits: ULONG,
    pub NumberGreenBits: ULONG,
    pub NumberBlueBits: ULONG,
    pub RedMask: ULONG,
    pub GreenMask: ULONG,
    pub BlueMask: ULONG,
    pub AttributeFlags: ULONG,
    pub VideoMemoryBitmapWidth: ULONG,
    pub VideoMemoryBitmapHeight: ULONG,
    pub DriverSpecificAttributeFlags: ULONG,
}

#[repr(C)]
pub struct VIDEO_NUM_MODES {
    pub NumModes: ULONG,
    pub ModeInformationLength: ULONG,
}

#[repr(C)]
pub struct VIDEO_MEMORY {
    pub RequestedVirtualAddress: PVOID,
}

#[repr(C)]
pub struct VIDEO_MEMORY_INFORMATION {
    pub VideoRamBase: PVOID,
    pub VideoRamLength: ULONG,
    pub FrameBufferBase: PVOID,
    pub FrameBufferLength: ULONG,
}

/// `PVIDEO_WIN32K_CALLOUT` — `VOID (NTAPI *)(PVOID Params)`. We only store
/// it opaquely (never invoke it), so an opaque pointer is sufficient.
pub type PVIDEO_WIN32K_CALLOUT = *mut c_void;

#[repr(C)]
pub struct VIDEO_WIN32K_CALLBACKS {
    pub PhysDisp: PVOID,
    pub Callout: PVIDEO_WIN32K_CALLOUT,
    pub bACPI: ULONG,
    pub pPhysDeviceObject: HANDLE,
    pub DualviewFlags: ULONG,
}

#[repr(C)]
pub struct VIDEO_COLOR_CAPABILITIES {
    pub Length: ULONG,
    pub AttributeFlags: ULONG,
    pub RedPhosphoreDecay: LONG,
    pub GreenPhosphoreDecay: LONG,
    pub BluePhosphoreDecay: LONG,
    pub WhiteChromaticity_x: LONG,
    pub WhiteChromaticity_y: LONG,
    pub WhiteChromaticity_Y: LONG,
    pub RedChromaticity_x: LONG,
    pub RedChromaticity_y: LONG,
    pub GreenChromaticity_x: LONG,
    pub GreenChromaticity_y: LONG,
    pub BlueChromaticity_x: LONG,
    pub BlueChromaticity_y: LONG,
    pub WhiteGamma: LONG,
    pub RedGamma: LONG,
    pub GreenGamma: LONG,
    pub BlueGamma: LONG,
}

/// `CM_FRAMEBUF_DEVICE_DATA` from reactos/drivers/bootvid/framebuf.h —
/// CM_VIDEO_DEVICE_DATA header (Version/Revision/VideoClock) followed by
/// the framebuffer geometry, appended as a CmResourceTypeDeviceSpecific
/// descriptor's trailing data. Not in the shared bindgen.
#[repr(C)]
pub struct CM_PIXEL_MASKS {
    pub RedMask: ULONG,
    pub GreenMask: ULONG,
    pub BlueMask: ULONG,
    pub ReservedMask: ULONG,
}

#[repr(C)]
pub struct CM_FRAMEBUF_DEVICE_DATA {
    // CM_VIDEO_DEVICE_DATA (anonymous base in C)
    pub Version: USHORT,
    pub Revision: USHORT,
    pub VideoClock: ULONG,
    // CM_FRAMEBUF_DEVICE_DATA extension
    pub FrameBufferOffset: ULONG,
    pub ScreenWidth: ULONG,
    pub ScreenHeight: ULONG,
    pub PixelsPerScanLine: ULONG,
    pub BitsPerPixel: ULONG,
    pub PixelMasks: CM_PIXEL_MASKS,
}
pub type PCM_FRAMEBUF_DEVICE_DATA = *mut CM_FRAMEBUF_DEVICE_DATA;

// ---- device extension ----

#[repr(C)]
pub struct SYSFB_DEVICE_EXTENSION {
    pub FrameBufferPhysical: PHYSICAL_ADDRESS,
    pub FrameBufferSize: ULONG,
    pub ScreenWidth: ULONG,
    pub ScreenHeight: ULONG,
    pub PixelsPerScanLine: ULONG,
    pub BitsPerPixel: ULONG,
    pub ScreenStride: ULONG,
    pub RedMask: ULONG,
    pub GreenMask: ULONG,
    pub BlueMask: ULONG,
    pub FrameBufferMapped: PVOID,
    pub FrameBufferMappedSize: ULONG,
    pub PhysDisp: PVOID,
    pub Callout: PVOID,
    /// No framebuffer from firmware: FrameBufferMapped is nonpaged pool standing in
    /// for one (see SysFbHeadless), never an I/O mapping.
    pub Headless: BOOLEAN,
}
pub type PSYSFB_DEVICE_EXTENSION = *mut SYSFB_DEVICE_EXTENSION;

// ---- registry helpers ----

unsafe fn RegSetSz(KeyHandle: HANDLE, ValueName: *const u16, Data: *const u16) -> NTSTATUS {
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut Name, ValueName);
    // length = (wcslen(Data)+1) * sizeof(WCHAR)
    let mut len: usize = 0;
    while *Data.add(len) != 0 {
        len += 1;
    }
    ZwSetValueKey(
        KeyHandle,
        &mut Name,
        0,
        REG_SZ,
        Data as PVOID,
        ((len + 1) * core::mem::size_of::<u16>()) as ULONG,
    )
}

unsafe fn RegSetMultiSz(
    KeyHandle: HANDLE,
    ValueName: *const u16,
    Data: *const u16,
    DataSize: ULONG,
) -> NTSTATUS {
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut Name, ValueName);
    ZwSetValueKey(KeyHandle, &mut Name, 0, REG_MULTI_SZ, Data as PVOID, DataSize)
}

unsafe fn RegSetDword(KeyHandle: HANDLE, ValueName: *const u16, Value: ULONG) -> NTSTATUS {
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut Name, ValueName);
    let mut v = Value;
    ZwSetValueKey(
        KeyHandle,
        &mut Name,
        0,
        REG_DWORD,
        core::ptr::addr_of_mut!(v) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    )
}

unsafe fn RegCreateKey(
    ParentKey: HANDLE,
    SubKeyName: *const u16,
    OutHandle: *mut HANDLE,
) -> NTSTATUS {
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut Disposition: ULONG = 0;

    RtlInitUnicodeString(&mut Name, SubKeyName);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut Name,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        ParentKey,
        core::ptr::null_mut(),
    );
    ZwCreateKey(
        OutHandle,
        KEY_ALL_ACCESS,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        &mut Disposition,
    )
}

// ---- PnP: two ways in, one display ----
//
// sysfb drives the framebuffer the firmware handed over, and binds to the device it can
// show owns it. There are two kinds of FDO:
//
//  * The ROOT instance. DriverEntry reports it itself (IoReportDetectedDevice, as the HAL
//    does). A reported device counts as started at once and gets no START_DEVICE, so it
//    claims the display inside DriverEntry, where the legacy driver used to create
//    \Device\Video0. win32k's InitVideo therefore cannot run ahead of it. On a machine with
//    no PCI display (arm64 virt's ramfb, a Pi, a headless server) it stays the owner.
//  * A PCI display controller: class 03/00 or 03/80, bound through the
//    CriticalDeviceDatabase. At START_DEVICE it takes the display over if, and only if,
//    the firmware framebuffer lies inside one of its memory BARs. Otherwise it fails start,
//    because a second card is not the one the firmware drew on.
//
// The display itself is a CONTROL device, \Device\Video0. It is created on the first claim
// and attached to no stack, so ownership can move between instances without renaming the
// display: win32k names it \\.\DISPLAY1 after Video0 and caches that name. When the owning
// PCI instance stops or goes away, the display falls back to the root instance.
//
// Claims and releases happen only in DriverEntry and in PnP IRPs, and the PnP manager
// delivers those one at a time, so the ownership statics need no lock. Only the open count
// is touched from arbitrary threads.

/// The FDO extension of either instance.
#[repr(C)]
pub struct SYSFB_FDO_EXTENSION {
    pub Pdo: PDEVICE_OBJECT,
    pub Lower: PDEVICE_OBJECT,
    /// The instance DriverEntry reported (the fallback), as opposed to a PCI one.
    pub IsRoot: BOOLEAN,
    /// GUID_DEVINTERFACE_DISPLAY_ADAPTER on this PDO; enabled only while it owns the display.
    pub InterfaceName: _UNICODE_STRING,
}
pub type PSYSFB_FDO_EXTENSION = *mut SYSFB_FDO_EXTENSION;

static mut DRIVER: PDRIVER_OBJECT = core::ptr::null_mut();
/// The framebuffer (or headless frame) DriverEntry found. One per boot.
static mut DISPLAY: SYSFB_DEVICE_EXTENSION = unsafe { core::mem::zeroed() };
static mut HAVE_DISPLAY: bool = false;
/// \Device\Video0, the control device win32k opens.
static mut VIDEO_DEVICE: PDEVICE_OBJECT = core::ptr::null_mut();
/// The FDO the display is attributed to, null if none.
static mut OWNER: PDEVICE_OBJECT = core::ptr::null_mut();
static mut ROOT_PDO: PDEVICE_OBJECT = core::ptr::null_mut();
static mut ROOT_FDO: PDEVICE_OBJECT = core::ptr::null_mut();
/// Handles open on the display, through Video0 or the owner's interface.
static OPENS: AtomicI32 = AtomicI32::new(0);

#[inline]
unsafe fn fdo_ext(Fdo: PDEVICE_OBJECT) -> PSYSFB_FDO_EXTENSION {
    (*Fdo).DeviceExtension as PSYSFB_FDO_EXTENSION
}

#[inline]
unsafe fn instance_kind(Fdo: PDEVICE_OBJECT) -> *const i8 {
    if (*fdo_ext(Fdo)).IsRoot != 0 {
        b"root\0".as_ptr() as *const i8
    } else {
        b"PCI\0".as_ptr() as *const i8
    }
}

// ---- DriverEntry ----

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let _ = RegistryPath;
    let mut Status: NTSTATUS;

    DPRINT1!("[sysfb] DriverEntry\n");
    DRIVER = DriverObject;

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(SysFbCreateClose);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(SysFbCreateClose);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(SysFbDeviceControl);
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(SysFbPnp);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(SysFbPower);
    (*(*DriverObject).DriverExtension).AddDevice =
        core::mem::transmute::<AddDeviceFn, _>(SysFbAddDevice);

    let DevExt = core::ptr::addr_of_mut!(DISPLAY);
    Status = SysFbDiscoverFramebuffer(DevExt);
    if !NT_SUCCESS(Status) {
        // A partial read must not leave a physical base behind for the PCI check to match.
        core::ptr::write_bytes(DevExt as *mut u8, 0, core::mem::size_of::<SYSFB_DEVICE_EXTENSION>());
        Status = SysFbHeadless(DevExt);
    }
    if !NT_SUCCESS(Status) {
        // Not even a frame in memory: there is nothing to hand win32k, which
        // bugchecks 0xB4 without a display device.
        DPRINT1!(
            "[sysfb] No ARC framebuffer and no headless frame: 0x%08lx - bailing\n",
            Status as u32
        );
        return Status;
    }
    HAVE_DISPLAY = true;

    DPRINT1!(
        "[sysfb] Framebuffer: %ldx%ld %ld BPP, size=0x%lx, stride=%ld\n",
        (*DevExt).ScreenWidth,
        (*DevExt).ScreenHeight,
        (*DevExt).BitsPerPixel,
        (*DevExt).FrameBufferSize,
        (*DevExt).ScreenStride
    );

    // The root instance, reported and started here so the display exists before win32k.
    let mut Pdo: PDEVICE_OBJECT = core::ptr::null_mut();
    Status = IoReportDetectedDevice(
        DriverObject,
        InterfaceTypeUndefined,
        !0u32,
        !0u32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        FALSE as u8,
        &mut Pdo,
    );
    if NT_SUCCESS(Status) {
        (*Pdo).Flags &= !DO_DEVICE_INITIALIZING;
        ROOT_PDO = Pdo;
        let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
        Status = SysFbCreateFdo(DriverObject, Pdo, &mut Fdo);
        if NT_SUCCESS(Status) {
            ROOT_FDO = Fdo;
        } else {
            DPRINT1!("[sysfb] root instance: AddDevice failed: 0x%08lx\n", Status as u32);
        }
    } else {
        DPRINT1!("[sysfb] IoReportDetectedDevice failed: 0x%08lx\n", Status as u32);
    }
    if ROOT_FDO.is_null() {
        // Publish the display anyway, owned by no device node. Without it win32k
        // bugchecks 0xB4, and a failed report is a PnP manager bug: chase it from
        // the line above, not from a black screen.
        DPRINT1!("[sysfb] *** no root instance: the display has no device node ***\n");
    }

    Status = SysFbClaim(ROOT_FDO);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    DPRINT1!("[sysfb] DriverEntry SUCCESS\n");
    STATUS_SUCCESS
}

// ---- AddDevice and the instances ----

unsafe extern "system" fn SysFbAddDevice(DriverObject: PDRIVER_OBJECT, Pdo: PDEVICE_OBJECT) -> NTSTATUS {
    let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
    SysFbCreateFdo(DriverObject, Pdo, &mut Fdo)
}

unsafe fn SysFbCreateFdo(
    DriverObject: PDRIVER_OBJECT,
    Pdo: PDEVICE_OBJECT,
    OutFdo: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
    let Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<SYSFB_FDO_EXTENSION>() as ULONG,
        core::ptr::null_mut(),
        FILE_DEVICE_VIDEO_TYPE,
        FILE_DEVICE_SECURE_OPEN,
        FALSE as u8,
        &mut Fdo,
    );
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] AddDevice: IoCreateDevice failed: 0x%08lx\n", Status as u32);
        return Status;
    }
    let Ext = fdo_ext(Fdo);
    core::ptr::write_bytes(Ext as *mut u8, 0, core::mem::size_of::<SYSFB_FDO_EXTENSION>());
    (*Ext).Pdo = Pdo;
    (*Ext).IsRoot = (Pdo == ROOT_PDO) as BOOLEAN;
    (*Ext).Lower = IoAttachDeviceToDeviceStack(Fdo, Pdo);
    if (*Ext).Lower.is_null() {
        DPRINT1!("[sysfb] AddDevice: cannot attach to the PDO\n");
        IoDeleteDevice(Fdo);
        return STATUS_NO_SUCH_DEVICE;
    }

    let IfStatus = IoRegisterDeviceInterface(
        Pdo,
        &GUID_DEVINTERFACE_DISPLAY_ADAPTER,
        core::ptr::null_mut(),
        &mut (*Ext).InterfaceName,
    );
    if !NT_SUCCESS(IfStatus) {
        // Nothing enumerates displays by interface yet; the device map is what win32k reads.
        DPRINT1!("[sysfb] display adapter interface not registered: 0x%08lx\n", IfStatus as u32);
        (*Ext).InterfaceName = core::mem::zeroed();
    }

    (*Fdo).Flags |= DO_POWER_PAGABLE;
    (*Fdo).Flags &= !DO_DEVICE_INITIALIZING;
    if (*Ext).IsRoot != 0 {
        // Here, not only in DriverEntry: a root instance that was disabled and enabled
        // again comes back through PnP's AddDevice, and the fallback must find it.
        ROOT_FDO = Fdo;
    }
    DPRINT1!("[sysfb] AddDevice: %s instance\n", instance_kind(Fdo));
    *OutFdo = Fdo;
    STATUS_SUCCESS
}

unsafe fn SysFbSetInterface(Fdo: PDEVICE_OBJECT, Enable: bool) {
    let Ext = fdo_ext(Fdo);
    if (*Ext).InterfaceName.Buffer.is_null() {
        return;
    }
    let Status = IoSetDeviceInterfaceState(&mut (*Ext).InterfaceName, Enable as BOOLEAN);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] IoSetDeviceInterfaceState(%lu) failed: 0x%08lx\n", Enable as u32, Status as u32);
    }
}

/// Make `Fdo` (null: no device node) the owner of the display, creating \Device\Video0 on
/// the first claim. A later claim moves the ownership and keeps the device.
unsafe fn SysFbClaim(Fdo: PDEVICE_OBJECT) -> NTSTATUS {
    if VIDEO_DEVICE.is_null() {
        let Status = SysFbCreateVideoDevice();
        if !NT_SUCCESS(Status) {
            return Status;
        }
    } else if OWNER == Fdo {
        return STATUS_SUCCESS;
    }
    let Previous = OWNER;
    if !Previous.is_null() {
        SysFbSetInterface(Previous, false);
    }
    OWNER = Fdo;
    if !Fdo.is_null() {
        SysFbSetInterface(Fdo, true);
        DPRINT1!("[sysfb] display owned by the %s instance\n", instance_kind(Fdo));
    }
    STATUS_SUCCESS
}

/// `Fdo` is going away or stopping. If it owns the display, the root instance takes the
/// display back. With no root instance, Video0 goes too, unless handles are still open on
/// it.
unsafe fn SysFbRelease(Fdo: PDEVICE_OBJECT) {
    if Fdo.is_null() || OWNER != Fdo {
        return;
    }
    SysFbSetInterface(Fdo, false);
    if !ROOT_FDO.is_null() && ROOT_FDO != Fdo {
        OWNER = ROOT_FDO;
        SysFbSetInterface(ROOT_FDO, true);
        DPRINT1!("[sysfb] display falls back to the root instance\n");
        return;
    }
    OWNER = core::ptr::null_mut();
    let Opens = OPENS.load(Ordering::SeqCst);
    if Opens != 0 {
        DPRINT1!("[sysfb] display owner gone with %ld handle(s) open; \\Device\\Video0 stays\n", Opens);
        return;
    }
    SysFbDeleteVideoDevice();
}

unsafe fn SysFbCreateVideoDevice() -> NTSTATUS {
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut SymlinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut Device: PDEVICE_OBJECT = core::ptr::null_mut();
    let dev_name = wsz!(b"\\Device\\Video0");
    let sym_name = wsz!(b"\\??\\DISPLAY1");

    RtlInitUnicodeString(&mut DeviceName, dev_name.as_ptr());
    let mut Status = IoCreateDevice(
        DRIVER,
        0,
        &mut DeviceName,
        FILE_DEVICE_VIDEO_TYPE,
        0,
        FALSE as u8,
        &mut Device,
    );
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] IoCreateDevice failed: 0x%08lx\n", Status as u32);
        return Status;
    }

    RtlInitUnicodeString(&mut SymlinkName, sym_name.as_ptr());
    Status = IoCreateSymbolicLink(&mut SymlinkName, &mut DeviceName);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] IoCreateSymbolicLink failed: 0x%08lx\n", Status as u32);
        // Non-fatal — continue
    }
    // Created outside DriverEntry on a PCI take-over, where the I/O manager does not clear it.
    (*Device).Flags &= !DO_DEVICE_INITIALIZING;
    VIDEO_DEVICE = Device;

    Status = SysFbWriteDeviceMap();
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] Failed to write DEVICEMAP\\VIDEO: 0x%08lx\n", Status as u32);
        // Non-fatal
    }

    Status = SysFbWriteServiceConfig(core::ptr::addr_of_mut!(DISPLAY));
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] Failed to write service config: 0x%08lx\n", Status as u32);
    }
    STATUS_SUCCESS
}

unsafe fn SysFbDeleteVideoDevice() {
    let DevExt = core::ptr::addr_of_mut!(DISPLAY);
    if !(*DevExt).FrameBufferMapped.is_null() && (*DevExt).Headless == 0 {
        MmUnmapIoSpace((*DevExt).FrameBufferMapped, (*DevExt).FrameBufferMappedSize as _);
        (*DevExt).FrameBufferMapped = core::ptr::null_mut();
        (*DevExt).FrameBufferMappedSize = 0;
    }
    SysFbDeleteDeviceMap();
    let mut SymlinkName: _UNICODE_STRING = core::mem::zeroed();
    let sym_name = wsz!(b"\\??\\DISPLAY1");
    RtlInitUnicodeString(&mut SymlinkName, sym_name.as_ptr());
    IoDeleteSymbolicLink(&mut SymlinkName);
    IoDeleteDevice(VIDEO_DEVICE);
    VIDEO_DEVICE = core::ptr::null_mut();
    DPRINT1!("[sysfb] \\Device\\Video0 removed\n");
}

// ---- IRP_MJ_PNP / IRP_MJ_POWER ----

unsafe fn SysFbPassDown(Fdo: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    IoSkipCurrentIrpStackLocation(Irp);
    IoCallDriver((*fdo_ext(Fdo)).Lower, Irp)
}

unsafe fn SysFbComplete(Irp: PIRP, Status: NTSTATUS) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    Status
}

unsafe extern "system" fn SysFbPnp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    if DeviceObject == VIDEO_DEVICE {
        // The control device sits on no PnP stack.
        return SysFbComplete(Irp, (*Irp).IoStatus.__bindgen_anon_1.Status);
    }
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    let Ext = fdo_ext(DeviceObject);

    match (*IrpSp).MinorFunction {
        IRP_MN_START_DEVICE => {
            // The bus starts first: the BARs decode only once the PCI PDO is started.
            let mut Status = if IoForwardIrpSynchronously((*Ext).Lower, Irp) != 0 {
                (*Irp).IoStatus.__bindgen_anon_1.Status
            } else {
                STATUS_UNSUCCESSFUL
            };
            if NT_SUCCESS(Status) {
                if (*Ext).IsRoot == 0 {
                    Status = SysFbStartPci(
                        DeviceObject,
                        (*IrpSp).Parameters.StartDevice.AllocatedResourcesTranslated,
                    );
                } else if OWNER.is_null() && HAVE_DISPLAY {
                    // The reported instance is started in DriverEntry and gets no START then;
                    // this START is a re-enable. It takes the display only if nobody has it.
                    Status = SysFbClaim(DeviceObject);
                }
            }
            SysFbComplete(Irp, Status)
        }
        IRP_MN_QUERY_STOP_DEVICE | IRP_MN_QUERY_REMOVE_DEVICE => {
            if OWNER == DeviceObject && OPENS.load(Ordering::SeqCst) > 0 {
                // Something (win32k) is drawing through us, and the frame goes dark with the
                // device. A vendor driver takes over only after the session lets go.
                DPRINT1!("[sysfb] %s instance owns an open display: refusing stop/remove\n",
                    instance_kind(DeviceObject));
                return SysFbComplete(Irp, STATUS_DEVICE_BUSY);
            }
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            SysFbPassDown(DeviceObject, Irp)
        }
        IRP_MN_CANCEL_STOP_DEVICE | IRP_MN_CANCEL_REMOVE_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            SysFbPassDown(DeviceObject, Irp)
        }
        IRP_MN_STOP_DEVICE | IRP_MN_SURPRISE_REMOVAL => {
            SysFbRelease(DeviceObject);
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            SysFbPassDown(DeviceObject, Irp)
        }
        IRP_MN_REMOVE_DEVICE => {
            SysFbRelease(DeviceObject);
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            let Lower = (*Ext).Lower;
            let Status = SysFbPassDown(DeviceObject, Irp);
            IoDetachDevice(Lower);
            if !(*Ext).InterfaceName.Buffer.is_null() {
                RtlFreeUnicodeString(&mut (*Ext).InterfaceName);
            }
            if ROOT_FDO == DeviceObject {
                ROOT_FDO = core::ptr::null_mut();
            }
            IoDeleteDevice(DeviceObject);
            Status
        }
        _ => SysFbPassDown(DeviceObject, Irp),
    }
}

/// A PCI display controller is ours if the firmware framebuffer sits in one of its BARs.
unsafe fn SysFbStartPci(Fdo: PDEVICE_OBJECT, Resources: PCM_RESOURCE_LIST) -> NTSTATUS {
    let DevExt = core::ptr::addr_of!(DISPLAY);
    if !HAVE_DISPLAY || (*DevExt).Headless != 0 {
        DPRINT1!("[sysfb] PCI display controller, but the firmware handed over no framebuffer\n");
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }
    let Base = (*DevExt).FrameBufferPhysical.QuadPart as u64;
    if !SysFbResourcesContain(Resources, Base) {
        DPRINT1!("[sysfb] PCI display controller does not decode the framebuffer at 0x%08lx%08lx\n",
            (Base >> 32) as u32, Base as u32);
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }
    SysFbClaim(Fdo)
}

/// Does a memory range in `List` contain `Address`? Walks the variable-length list:
/// full descriptors follow one another, and a device-specific partial carries DataSize
/// trailing bytes.
unsafe fn SysFbResourcesContain(List: PCM_RESOURCE_LIST, Address: u64) -> bool {
    if List.is_null() {
        return false;
    }
    let mut Full = core::ptr::addr_of!((*List).List) as *const u8;
    for _ in 0..(*List).Count {
        let FullDesc = Full as *const CM_FULL_RESOURCE_DESCRIPTOR;
        let Count = (*FullDesc).PartialResourceList.Count;
        let mut Partial = core::ptr::addr_of!((*FullDesc).PartialResourceList.PartialDescriptors) as *const u8;
        for _ in 0..Count {
            let Desc = Partial as *const CM_PARTIAL_RESOURCE_DESCRIPTOR;
            let Type = (*Desc).Type;
            if Type == CmResourceTypeMemory || Type == CmResourceTypeMemoryLarge {
                let Start = (*Desc).u.Memory.Start.QuadPart as u64;
                let Length = memory_length(Type, (*Desc).Flags, (*Desc).u.Memory.Length);
                if Address >= Start && Address - Start < Length {
                    return true;
                }
            }
            Partial = Partial.add(core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>());
            if Type == CmResourceTypeDeviceSpecific {
                Partial = Partial.add((*Desc).u.DeviceSpecificData.DataSize as usize);
            }
        }
        Full = Partial;
    }
    false
}

/// The byte length of a memory descriptor: a large one stores it scaled down by its flag.
fn memory_length(Type: u8, Flags: u16, Length: ULONG) -> u64 {
    if Type != CmResourceTypeMemoryLarge {
        return Length as u64;
    }
    match Flags & (CM_RESOURCE_MEMORY_LARGE_40 | CM_RESOURCE_MEMORY_LARGE_48 | CM_RESOURCE_MEMORY_LARGE_64) {
        CM_RESOURCE_MEMORY_LARGE_40 => (Length as u64) << 8,
        CM_RESOURCE_MEMORY_LARGE_48 => (Length as u64) << 16,
        CM_RESOURCE_MEMORY_LARGE_64 => (Length as u64) << 32,
        _ => Length as u64,
    }
}

/// Would entering `State` lose the display? S1-S3 power the GPU down and S4 cuts power
/// altogether; GOP set the mode before ExitBootServices and nothing can set it again. Shutdown
/// (S5) is queried too and must pass: the shutdown path ignores answers, and it ends the
/// session anyway.
fn sleep_loses_display(State: SYSTEM_POWER_STATE) -> bool {
    State >= PowerSystemSleeping1 && State <= PowerSystemHibernate
}

// Power policy is the bus's, and device D-states are no-ops for a framebuffer. The one
// decision sysfb makes is to refuse a system sleep while it is the display in use: there is
// no hardware state to save, and nothing could restore the GOP mode on resume. A vendor
// driver that can set modes lifts the restriction by replacing sysfb.
unsafe extern "system" fn SysFbPower(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    PoStartNextPowerIrp(Irp);
    if DeviceObject == VIDEO_DEVICE {
        return SysFbComplete(Irp, (*Irp).IoStatus.__bindgen_anon_1.Status);
    }
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    if (*IrpSp).MinorFunction == IRP_MN_QUERY_POWER
        && (*IrpSp).Parameters.Power.Type == SystemPowerState
        && sleep_loses_display((*IrpSp).Parameters.Power.State.SystemState)
        && OWNER == DeviceObject
        && OPENS.load(Ordering::SeqCst) > 0
    {
        DPRINT1!("[sysfb] refusing S%lu: the firmware display mode cannot be restored on resume\n",
            ((*IrpSp).Parameters.Power.State.SystemState - 1) as u32);
        return SysFbComplete(Irp, STATUS_NOT_SUPPORTED);
    }
    IoSkipCurrentIrpStackLocation(Irp);
    PoCallDriver((*fdo_ext(DeviceObject)).Lower, Irp)
}

/// No firmware framebuffer: a server (EC2 Graviton has no display, nor does an x86 box
/// booted without GOP). No arch has a VGA driver to fall back to any more (the miniports
/// went in bdea86df186), and win32k bugchecks (0xB4, VIDEO_DRIVER_INIT_FAILURE) without
/// a display device, so the desktop runs on a frame in memory that nothing scans out.
/// Everything above the driver works the same; the session is reached over the network
/// or watched over serial.
unsafe fn SysFbHeadless(DevExt: PSYSFB_DEVICE_EXTENSION) -> NTSTATUS {
    const WIDTH: ULONG = 1024;
    const HEIGHT: ULONG = 768;
    let size = WIDTH * HEIGHT * 4;
    let frame = ExAllocatePoolWithTag(NonPagedPool, size as _, TAG_SYSFB);
    if frame.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(frame as *mut u8, 0, size as usize);
    (*DevExt).ScreenWidth = WIDTH;
    (*DevExt).ScreenHeight = HEIGHT;
    (*DevExt).PixelsPerScanLine = WIDTH;
    (*DevExt).BitsPerPixel = 32;
    (*DevExt).ScreenStride = WIDTH * 4;
    (*DevExt).RedMask = 0x00FF_0000;
    (*DevExt).GreenMask = 0x0000_FF00;
    (*DevExt).BlueMask = 0x0000_00FF;
    (*DevExt).FrameBufferSize = size;
    (*DevExt).FrameBufferMapped = frame;
    (*DevExt).FrameBufferMappedSize = size;
    (*DevExt).Headless = 1;
    DPRINT1!("[sysfb] no firmware framebuffer: headless, %lux%lu in memory\n", WIDTH, HEIGHT);
    STATUS_SUCCESS
}

/// Video0 and the owning instance's interface are the display; any other FDO is idle.
#[inline]
unsafe fn is_display(DeviceObject: PDEVICE_OBJECT) -> bool {
    !DeviceObject.is_null() && (DeviceObject == VIDEO_DEVICE || DeviceObject == OWNER)
}

// IRP_MJ_CREATE / IRP_MJ_CLOSE — count the opens on the display; refuse an idle instance.
unsafe extern "system" fn SysFbCreateClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    let mut Status = STATUS_SUCCESS;
    if (*IrpSp).MajorFunction == IRP_MJ_CREATE as u8 {
        if is_display(DeviceObject) {
            OPENS.fetch_add(1, Ordering::SeqCst);
        } else {
            Status = STATUS_NO_SUCH_DEVICE;
        }
    } else {
        // A close follows only a create that succeeded, so it is always one of ours.
        OPENS.fetch_sub(1, Ordering::SeqCst);
    }
    (*Irp).IoStatus.Information = 0;
    SysFbComplete(Irp, Status)
}

// IRP_MJ_DEVICE_CONTROL — handle video IOCTLs
unsafe extern "system" fn SysFbDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    if !is_display(DeviceObject) {
        (*Irp).IoStatus.Information = 0;
        return SysFbComplete(Irp, STATUS_INVALID_DEVICE_REQUEST);
    }
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    let DevExt = core::ptr::addr_of_mut!(DISPLAY);
    let IoControlCode = (*IrpSp).Parameters.DeviceIoControl.IoControlCode;
    let InputBuffer = (*Irp).AssociatedIrp.SystemBuffer;
    let OutputBuffer = (*Irp).AssociatedIrp.SystemBuffer;
    let InputLength = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;
    let OutputLength = (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Information: ULONG = 0;

    match IoControlCode {
        IOCTL_VIDEO_INIT_WIN32K_CALLBACKS => {
            DPRINT1!("[sysfb] IOCTL_VIDEO_INIT_WIN32K_CALLBACKS\n");
            if (InputLength as usize) < core::mem::size_of::<VIDEO_WIN32K_CALLBACKS>()
                || (OutputLength as usize) < core::mem::size_of::<VIDEO_WIN32K_CALLBACKS>()
            {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let pCallbacks = InputBuffer as *mut VIDEO_WIN32K_CALLBACKS;
                (*DevExt).PhysDisp = (*pCallbacks).PhysDisp;
                (*DevExt).Callout = (*pCallbacks).Callout as PVOID;
                (*pCallbacks).bACPI = FALSE as ULONG;
                // The device node behind the display, which win32k queries for the
                // adapter's IDs and description. Video0 itself only with no node at all.
                (*pCallbacks).pPhysDeviceObject = if OWNER.is_null() {
                    DeviceObject as HANDLE
                } else {
                    (*fdo_ext(OWNER)).Pdo as HANDLE
                };
                (*pCallbacks).DualviewFlags = 0;
                Information = core::mem::size_of::<VIDEO_WIN32K_CALLBACKS>() as ULONG;
            }
        }

        IOCTL_VIDEO_QUERY_NUM_AVAIL_MODES => {
            DPRINT1!("[sysfb] IOCTL_VIDEO_QUERY_NUM_AVAIL_MODES\n");
            if (OutputLength as usize) < core::mem::size_of::<VIDEO_NUM_MODES>() {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let pModes = OutputBuffer as *mut VIDEO_NUM_MODES;
                (*pModes).NumModes = 1;
                (*pModes).ModeInformationLength = core::mem::size_of::<VIDEO_MODE_INFORMATION>() as ULONG;
                Information = core::mem::size_of::<VIDEO_NUM_MODES>() as ULONG;
            }
        }

        IOCTL_VIDEO_QUERY_AVAIL_MODES | IOCTL_VIDEO_QUERY_CURRENT_MODE => {
            DPRINT1!("[sysfb] IOCTL_VIDEO_QUERY_AVAIL/CURRENT_MODE(S)\n");
            if (OutputLength as usize) < core::mem::size_of::<VIDEO_MODE_INFORMATION>() {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let pModeInfo = OutputBuffer as *mut VIDEO_MODE_INFORMATION;
                core::ptr::write_bytes(pModeInfo as *mut u8, 0, core::mem::size_of::<VIDEO_MODE_INFORMATION>());
                (*pModeInfo).Length = core::mem::size_of::<VIDEO_MODE_INFORMATION>() as ULONG;
                (*pModeInfo).ModeIndex = 0;
                (*pModeInfo).VisScreenWidth = (*DevExt).ScreenWidth;
                (*pModeInfo).VisScreenHeight = (*DevExt).ScreenHeight;
                (*pModeInfo).ScreenStride = (*DevExt).ScreenStride;
                (*pModeInfo).NumberOfPlanes = 1;
                (*pModeInfo).BitsPerPlane = (*DevExt).BitsPerPixel;
                (*pModeInfo).Frequency = 60;
                (*pModeInfo).XMillimeter = 320;
                (*pModeInfo).YMillimeter = 240;
                // The channel widths are the masks' bit counts: a 16bpp 5-6-5 framebuffer
                // is 5/6/5, not 8/8/8. A mask the loader left zero keeps the old 8.
                (*pModeInfo).NumberRedBits = mask_bits((*DevExt).RedMask);
                (*pModeInfo).NumberGreenBits = mask_bits((*DevExt).GreenMask);
                (*pModeInfo).NumberBlueBits = mask_bits((*DevExt).BlueMask);
                (*pModeInfo).RedMask = (*DevExt).RedMask;
                (*pModeInfo).GreenMask = (*DevExt).GreenMask;
                (*pModeInfo).BlueMask = (*DevExt).BlueMask;
                (*pModeInfo).AttributeFlags = VIDEO_MODE_COLOR | VIDEO_MODE_GRAPHICS | VIDEO_MODE_LINEAR;
                (*pModeInfo).VideoMemoryBitmapWidth = (*DevExt).ScreenWidth;
                (*pModeInfo).VideoMemoryBitmapHeight = (*DevExt).ScreenHeight;
                (*pModeInfo).DriverSpecificAttributeFlags = 0;
                Information = core::mem::size_of::<VIDEO_MODE_INFORMATION>() as ULONG;
            }
        }

        IOCTL_VIDEO_SET_CURRENT_MODE => {
            DPRINT1!("[sysfb] IOCTL_VIDEO_SET_CURRENT_MODE\n");
            // Mode already set by UEFI GOP — nothing to do
            Status = STATUS_SUCCESS;
        }

        IOCTL_VIDEO_MAP_VIDEO_MEMORY => {
            DPRINT1!("[sysfb] IOCTL_VIDEO_MAP_VIDEO_MEMORY\n");
            if (InputLength as usize) < core::mem::size_of::<VIDEO_MEMORY>()
                || (OutputLength as usize) < core::mem::size_of::<VIDEO_MEMORY_INFORMATION>()
            {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let pVideoMemoryInfo = OutputBuffer as *mut VIDEO_MEMORY_INFORMATION;
                if (*DevExt).FrameBufferPhysical.QuadPart == 0 && (*DevExt).Headless == 0 {
                    DPRINT1!("[sysfb] No framebuffer physical address!\n");
                    Status = STATUS_UNSUCCESSFUL;
                } else {
                    if (*DevExt).FrameBufferMapped.is_null() {
                        (*DevExt).FrameBufferMappedSize = (*DevExt).FrameBufferSize;
                        (*DevExt).FrameBufferMapped = MmMapIoSpace(
                            (*DevExt).FrameBufferPhysical,
                            (*DevExt).FrameBufferMappedSize as _,
                            // Write-combined, not uncached: GDI streams whole spans
                            // into the frame, and UC makes every store a bus cycle
                            // on x86. On arm64 MmNonCached I/O is Device memory,
                            // where the unaligned stores compilers emit for merged
                            // byte writes alignment-fault on real silicon.
                            MmWriteCombined,
                        );
                        if (*DevExt).FrameBufferMapped.is_null() {
                            DPRINT1!("[sysfb] MmMapIoSpace failed\n");
                            Status = STATUS_UNSUCCESSFUL;
                        } else {
                            DPRINT1!("[sysfb] Framebuffer mapped at %p (size 0x%lx)\n",
                                (*DevExt).FrameBufferMapped, (*DevExt).FrameBufferMappedSize);
                        }
                    }
                    if NT_SUCCESS(Status) {
                        (*pVideoMemoryInfo).VideoRamBase = (*DevExt).FrameBufferMapped;
                        (*pVideoMemoryInfo).VideoRamLength = (*DevExt).FrameBufferMappedSize;
                        (*pVideoMemoryInfo).FrameBufferBase = (*DevExt).FrameBufferMapped;
                        (*pVideoMemoryInfo).FrameBufferLength = (*DevExt).FrameBufferMappedSize;
                        Information = core::mem::size_of::<VIDEO_MEMORY_INFORMATION>() as ULONG;
                    }
                }
            }
        }

        IOCTL_VIDEO_UNMAP_VIDEO_MEMORY => {
            DPRINT1!("[sysfb] IOCTL_VIDEO_UNMAP_VIDEO_MEMORY\n");
            // A headless frame is pool, kept for the next MAP.
            if !(*DevExt).FrameBufferMapped.is_null() && (*DevExt).Headless == 0 {
                MmUnmapIoSpace((*DevExt).FrameBufferMapped, (*DevExt).FrameBufferMappedSize as _);
                (*DevExt).FrameBufferMapped = core::ptr::null_mut();
                (*DevExt).FrameBufferMappedSize = 0;
            }
        }

        IOCTL_VIDEO_RESET_DEVICE => {
            DPRINT1!("[sysfb] IOCTL_VIDEO_RESET_DEVICE\n");
            // No-op
        }

        IOCTL_VIDEO_SET_COLOR_REGISTERS => {
            DPRINT1!("[sysfb] IOCTL_VIDEO_SET_COLOR_REGISTERS (no-op for 32bpp)\n");
            // No-op for 32bpp direct color
        }

        IOCTL_VIDEO_QUERY_COLOR_CAPABILITIES => {
            DPRINT1!("[sysfb] IOCTL_VIDEO_QUERY_COLOR_CAPABILITIES\n");
            if (OutputLength as usize) < core::mem::size_of::<VIDEO_COLOR_CAPABILITIES>() {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let pColorCaps = OutputBuffer as *mut VIDEO_COLOR_CAPABILITIES;
                core::ptr::write_bytes(pColorCaps as *mut u8, 0, core::mem::size_of::<VIDEO_COLOR_CAPABILITIES>());
                (*pColorCaps).Length = core::mem::size_of::<VIDEO_COLOR_CAPABILITIES>() as ULONG;
                (*pColorCaps).AttributeFlags = VIDEO_DEVICE_COLOR;
                (*pColorCaps).WhiteChromaticity_x = 3127;
                (*pColorCaps).WhiteChromaticity_y = 3290;
                (*pColorCaps).RedChromaticity_x = 6700;
                (*pColorCaps).RedChromaticity_y = 3300;
                (*pColorCaps).GreenChromaticity_x = 2100;
                (*pColorCaps).GreenChromaticity_y = 7100;
                (*pColorCaps).BlueChromaticity_x = 1400;
                (*pColorCaps).BlueChromaticity_y = 800;
                (*pColorCaps).WhiteGamma = 20000;
                (*pColorCaps).RedGamma = 20000;
                (*pColorCaps).GreenGamma = 20000;
                (*pColorCaps).BlueGamma = 20000;
                Information = core::mem::size_of::<VIDEO_COLOR_CAPABILITIES>() as ULONG;
            }
        }

        _ => {
            DPRINT1!("[sysfb] Unhandled IOCTL 0x%08lx\n", IoControlCode);
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = Information as _;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    Status
}

// ---- Discover framebuffer from ARC config tree ----

/// Append `value` in decimal at `at`; returns the new end.
fn push_decimal(buf: &mut [u16], mut at: usize, value: u32) -> usize {
    let mut digits = [0u16; 10];
    let mut v = value;
    let mut k = 0;
    loop {
        digits[k] = b'0' as u16 + (v % 10) as u16;
        k += 1;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    while k > 0 {
        k -= 1;
        buf[at] = digits[k];
        at += 1;
    }
    at
}

unsafe fn SysFbDiscoverFramebuffer(DevExt: PSYSFB_DEVICE_EXTENSION) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hDisplay: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ResultLength: ULONG = 0;
    let mut Found = false;

    // ARC config tree: HARDWARE\DESCRIPTION\System\MultifunctionAdapter
    //   \<N>\DisplayController\<M>. The loader hangs the display off its ISA
    // (BIOS) or UEFI adapter, and N counts every adapter written before it: the
    // PCI BIOS key plus ONE PER PCI BUS, then ACPI. A board with a few bridges
    // (the OptiPlex 9020's root ports) puts it at 4 or later, past the 0..3 this
    // used to try, and the boot ended in BUGCHECK 0xB4 with a working VBE
    // framebuffer described in the tree. Walk every index the loader could use.
    // (No RtlStringCchPrintfW in ntkrnl-sys: the path is assembled by hand.)
    let prefix = wsz!(b"\\Registry\\Machine\\HARDWARE\\DESCRIPTION\\System\\MultifunctionAdapter\\");
    let middle = wsz!(b"\\DisplayController\\");
    let mut path: [u16; 128] = [0; 128];

    'outer: for AdapterIdx in 0u32..64 {
        for InstanceIdx in 0u32..2 {
            let mut n = 0usize;
            for &c in &prefix[..prefix.len() - 1] {
                path[n] = c;
                n += 1;
            }
            n = push_decimal(&mut path, n, AdapterIdx);
            for &c in &middle[..middle.len() - 1] {
                path[n] = c;
                n += 1;
            }
            n = push_decimal(&mut path, n, InstanceIdx);
            path[n] = 0;

            RtlInitUnicodeString(&mut KeyName, path.as_ptr());
            InitializeObjectAttributes(
                &mut ObjectAttributes,
                &mut KeyName,
                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            Status = ZwOpenKey(&mut hDisplay, KEY_READ, &mut ObjectAttributes);
            if NT_SUCCESS(Status) {
                DPRINT1!("[sysfb] Found DisplayController at adapter %lu instance %lu\n",
                    AdapterIdx, InstanceIdx);
                Found = true;
                break 'outer;
            }
        }
    }

    if !Found {
        DPRINT1!("[sysfb] DisplayController not found in ARC config tree\n");
        return STATUS_NOT_FOUND;
    }

    // Read "Configuration Data" value
    let cfgdata = wsz!(b"Configuration Data");
    RtlInitUnicodeString(&mut ValueName, cfgdata.as_ptr());
    Status = ZwQueryValueKey(
        hDisplay,
        &mut ValueName,
        KeyValuePartialInformation,
        core::ptr::null_mut(),
        0,
        &mut ResultLength,
    );
    if Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_BUFFER_OVERFLOW {
        DPRINT1!("[sysfb] No Configuration Data value: 0x%08lx\n", Status as u32);
        ZwClose(hDisplay);
        return STATUS_NOT_FOUND;
    }

    let BufferSize = ResultLength;
    let Buffer = ExAllocatePoolWithTag(PagedPool, BufferSize as _, TAG_SYSFB) as *mut u8;
    if Buffer.is_null() {
        ZwClose(hDisplay);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = ZwQueryValueKey(
        hDisplay,
        &mut ValueName,
        KeyValuePartialInformation,
        Buffer as PVOID,
        BufferSize,
        &mut ResultLength,
    );
    ZwClose(hDisplay);

    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] Failed to read Configuration Data: 0x%08lx\n", Status as u32);
        ExFreePoolWithTag(Buffer as PVOID, TAG_SYSFB);
        return Status;
    }

    let ValueInfo = Buffer as PKEY_VALUE_PARTIAL_INFORMATION;
    if ((*ValueInfo).DataLength as usize) < core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>() {
        DPRINT1!("[sysfb] Configuration Data too small: %lu bytes\n", (*ValueInfo).DataLength);
        ExFreePoolWithTag(Buffer as PVOID, TAG_SYSFB);
        return STATUS_INVALID_PARAMETER;
    }

    let FullResDesc = core::ptr::addr_of!((*ValueInfo).Data) as PCM_FULL_RESOURCE_DESCRIPTOR;
    let PartialList = core::ptr::addr_of!((*FullResDesc).PartialResourceList) as *const CM_PARTIAL_RESOURCE_LIST;
    let count = (*PartialList).Count;

    DPRINT1!("[sysfb] Configuration Data: %lu partial descriptors\n", count);

    let mut FrameBufferOffset: ULONG = 0;
    let mut i: u32 = 0;
    while i < count {
        // PartialDescriptors is a flexible array of CM_PARTIAL_RESOURCE_DESCRIPTOR.
        let PartialDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
            core::ptr::addr_of!((*PartialList).PartialDescriptors) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
        let PartialDesc = PartialDesc.add(i as usize);

        if (*PartialDesc).Type == CmResourceTypeMemory as u8 {
            (*DevExt).FrameBufferPhysical.QuadPart = (*PartialDesc).u.Memory.Start.QuadPart;
            (*DevExt).FrameBufferSize = (*PartialDesc).u.Memory.Length;
            DPRINT1!("[sysfb] Memory resource: size=0x%lx\n", (*DevExt).FrameBufferSize);
        } else if (*PartialDesc).Type == CmResourceTypeDeviceSpecific as u8 {
            // The CM_FRAMEBUF_DEVICE_DATA follows immediately after this descriptor.
            let FramebufData = PartialDesc.add(1) as PCM_FRAMEBUF_DEVICE_DATA;
            FrameBufferOffset = (*FramebufData).FrameBufferOffset;
            (*DevExt).ScreenWidth = (*FramebufData).ScreenWidth;
            (*DevExt).ScreenHeight = (*FramebufData).ScreenHeight;
            (*DevExt).PixelsPerScanLine = (*FramebufData).PixelsPerScanLine;
            (*DevExt).BitsPerPixel = (*FramebufData).BitsPerPixel;
            (*DevExt).RedMask = (*FramebufData).PixelMasks.RedMask;
            (*DevExt).GreenMask = (*FramebufData).PixelMasks.GreenMask;
            (*DevExt).BlueMask = (*FramebufData).PixelMasks.BlueMask;
            (*DevExt).ScreenStride = (*DevExt).PixelsPerScanLine * ((*DevExt).BitsPerPixel / 8);
            DPRINT1!("[sysfb] Display: %lux%lu, %lu BPP, stride=%lu\n",
                (*DevExt).ScreenWidth, (*DevExt).ScreenHeight, (*DevExt).BitsPerPixel, (*DevExt).ScreenStride);
        }
        i += 1;
    }

    ExFreePoolWithTag(Buffer as PVOID, TAG_SYSFB);

    if (*DevExt).FrameBufferPhysical.QuadPart == 0 || (*DevExt).ScreenWidth == 0 {
        DPRINT1!("[sysfb] Incomplete framebuffer info from ARC tree\n");
        return STATUS_NOT_FOUND;
    }

    // The memory resource is the whole video RAM; the visible frame starts
    // FrameBufferOffset into it (framebuf.h, and how bootvid maps it).
    if FrameBufferOffset != 0 {
        if (*DevExt).FrameBufferSize != 0 && FrameBufferOffset >= (*DevExt).FrameBufferSize {
            DPRINT1!("[sysfb] FrameBufferOffset 0x%lx is past the 0x%lx-byte memory resource\n",
                FrameBufferOffset, (*DevExt).FrameBufferSize);
            return STATUS_INVALID_PARAMETER;
        }
        (*DevExt).FrameBufferPhysical.QuadPart += FrameBufferOffset as i64;
        if (*DevExt).FrameBufferSize != 0 {
            (*DevExt).FrameBufferSize -= FrameBufferOffset;
        }
        DPRINT1!("[sysfb] Visible frame at offset 0x%lx into the memory resource\n", FrameBufferOffset);
    }

    if (*DevExt).FrameBufferSize == 0 {
        (*DevExt).FrameBufferSize = (*DevExt).ScreenStride * (*DevExt).ScreenHeight;
    }

    STATUS_SUCCESS
}

/// The width of one colour channel: the number of bits set in its mask, or 8 when the
/// loader supplied no mask (the value every mode reported before masks were read).
fn mask_bits(mask: ULONG) -> ULONG {
    if mask == 0 {
        8
    } else {
        mask.count_ones()
    }
}

// ---- Write HARDWARE\DEVICEMAP\VIDEO ----

unsafe fn SysFbWriteDeviceMap() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hDeviceMap: HANDLE = core::ptr::null_mut();
    let mut hVideo: HANDLE = core::ptr::null_mut();

    let devicemap = wsz!(b"\\Registry\\Machine\\HARDWARE\\DEVICEMAP");
    let video = wsz!(b"VIDEO");
    let dev0_name = wsz!(b"\\Device\\Video0");
    let dev0_path = wsz!(b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\sysfb\\Device0");
    let max_obj = wsz!(b"MaxObjectNumber");

    Status = RegCreateKey(core::ptr::null_mut(), devicemap.as_ptr(), &mut hDeviceMap);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] Cannot create DEVICEMAP key: 0x%08lx\n", Status as u32);
        return Status;
    }

    Status = RegCreateKey(hDeviceMap, video.as_ptr(), &mut hVideo);
    ZwClose(hDeviceMap);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] Cannot create VIDEO key: 0x%08lx\n", Status as u32);
        return Status;
    }

    Status = RegSetSz(hVideo, dev0_name.as_ptr(), dev0_path.as_ptr());
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] Failed to write \\Device\\Video0 value: 0x%08lx\n", Status as u32);
    }

    Status = RegSetDword(hVideo, max_obj.as_ptr(), 0);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] Failed to write MaxObjectNumber: 0x%08lx\n", Status as u32);
    }

    ZwClose(hVideo);
    DPRINT1!("[sysfb] DEVICEMAP\\VIDEO written\n");
    STATUS_SUCCESS
}

/// Take \Device\Video0 back out of DEVICEMAP\VIDEO when the display goes away. Leaving
/// MaxObjectNumber alone is harmless: win32k skips a missing value.
unsafe fn SysFbDeleteDeviceMap() {
    let mut hVideo: HANDLE = core::ptr::null_mut();
    let video = wsz!(b"\\Registry\\Machine\\HARDWARE\\DEVICEMAP\\VIDEO");
    let dev0_name = wsz!(b"\\Device\\Video0");
    if !NT_SUCCESS(RegCreateKey(core::ptr::null_mut(), video.as_ptr(), &mut hVideo)) {
        return;
    }
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut Name, dev0_name.as_ptr());
    let Status = ZwDeleteValueKey(hVideo, &mut Name);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] Failed to delete the \\Device\\Video0 value: 0x%08lx\n", Status as u32);
    }
    ZwClose(hVideo);
}

// ---- Write driver service config ----

unsafe fn SysFbWriteServiceConfig(DevExt: PSYSFB_DEVICE_EXTENSION) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hService: HANDLE = core::ptr::null_mut();
    let mut hDevice0: HANDLE = core::ptr::null_mut();

    let svc = wsz!(b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\sysfb");
    let device0 = wsz!(b"Device0");
    let installed = wsz!(b"InstalledDisplayDrivers");
    let descr_name = wsz!(b"Device Description");
    let descr_val = wsz!(b"QEMU Framebuffer");
    // "framebuf\0\0" — double-NUL-terminated MULTI_SZ. wsz! adds the final
    // NUL; the inner one terminates the single string, the wsz NUL is the
    // list terminator.
    let multisz = wsz!(b"framebuf\0");
    let bpp_name = wsz!(b"DefaultSettings.BitsPerPel");
    let xres_name = wsz!(b"DefaultSettings.XResolution");
    let yres_name = wsz!(b"DefaultSettings.YResolution");
    let vref_name = wsz!(b"DefaultSettings.VRefresh");
    let flags_name = wsz!(b"DefaultSettings.Flags");
    let xpan_name = wsz!(b"DefaultSettings.XPanning");
    let ypan_name = wsz!(b"DefaultSettings.YPanning");

    Status = RegCreateKey(core::ptr::null_mut(), svc.as_ptr(), &mut hService);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] Cannot open/create sysfb service key: 0x%08lx\n", Status as u32);
        return Status;
    }

    Status = RegCreateKey(hService, device0.as_ptr(), &mut hDevice0);
    ZwClose(hService);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[sysfb] Cannot create Device0 key: 0x%08lx\n", Status as u32);
        return Status;
    }

    RegSetMultiSz(
        hDevice0,
        installed.as_ptr(),
        multisz.as_ptr(),
        (multisz.len() * core::mem::size_of::<u16>()) as ULONG,
    );
    RegSetSz(hDevice0, descr_name.as_ptr(), descr_val.as_ptr());
    RegSetDword(hDevice0, bpp_name.as_ptr(), (*DevExt).BitsPerPixel);
    RegSetDword(hDevice0, xres_name.as_ptr(), (*DevExt).ScreenWidth);
    RegSetDword(hDevice0, yres_name.as_ptr(), (*DevExt).ScreenHeight);
    RegSetDword(hDevice0, vref_name.as_ptr(), 60);
    RegSetDword(hDevice0, flags_name.as_ptr(), 0);
    RegSetDword(hDevice0, xpan_name.as_ptr(), 0);
    RegSetDword(hDevice0, ypan_name.as_ptr(), 0);

    ZwClose(hDevice0);
    DPRINT1!("[sysfb] Service config written\n");
    STATUS_SUCCESS
}
