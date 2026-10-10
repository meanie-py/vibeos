//! arm64vid.sys — standalone GOP/linear-framebuffer video device driver.
//! Rust translation of `drivers/video/arm64vid/arm64vid.c`.
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
    RtlInitUnicodeString,
};
// FORCEINLINE DDK helpers — bindgen surfaces these as extern decls with no
// exported symbol; use ntkrnl-sys's real inline impls (AGENTS.md gotcha).
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;
use ntkrnl_sys::inline::common::InitializeObjectAttributes;
use core::ffi::c_void;

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
const TAG_AVID: u32 = u32::from_le_bytes(*b"AVid"); // 'diVA' multi-char → LE bytes

// CM_RESOURCE_TYPE values (sdk/include/xdk/cmtypes.h). Authoritative — NOT
// guessed: DeviceSpecific=5 is the exact constant whose wrong value (0x81)
// once broke this driver's framebuffer handoff (BUGCHECK 0xB4, see memory
// feedback_rustconv_wrong_constant_value). The freeldr writer side
// (freeldr-rust/src/pchw.rs) stamps these same bindgen values. CM_PARTIAL_
// RESOURCE_DESCRIPTOR.Type is a UCHAR, so the reader compares against u8.
const CmResourceTypeMemory: u8 = 3;
const CmResourceTypeDeviceSpecific: u8 = 5;

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
pub struct ARM64VID_DEVICE_EXTENSION {
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
    /// for one (see Arm64VidHeadless), never an I/O mapping.
    pub Headless: BOOLEAN,
}
pub type PARM64VID_DEVICE_EXTENSION = *mut ARM64VID_DEVICE_EXTENSION;

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

// ---- DriverEntry ----

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let _ = RegistryPath;
    let mut Status: NTSTATUS;
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut SymlinkName: _UNICODE_STRING = core::mem::zeroed();

    DPRINT1!("[arm64vid] DriverEntry\n");

    // Dispatch routines
    let dev_name = wsz!(b"\\Device\\Video0");
    let sym_name = wsz!(b"\\??\\DISPLAY1");

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(Arm64VidCreateClose);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(Arm64VidCreateClose);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(Arm64VidDeviceControl);

    RtlInitUnicodeString(&mut DeviceName, dev_name.as_ptr());
    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<ARM64VID_DEVICE_EXTENSION>() as ULONG,
        &mut DeviceName,
        FILE_DEVICE_VIDEO_TYPE,
        0,
        FALSE as u8,
        &mut DeviceObject,
    );
    if !NT_SUCCESS(Status) {
        DPRINT1!("[arm64vid] IoCreateDevice failed: 0x%08lx\n", Status as u32);
        return Status;
    }

    RtlInitUnicodeString(&mut SymlinkName, sym_name.as_ptr());
    Status = IoCreateSymbolicLink(&mut SymlinkName, &mut DeviceName);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[arm64vid] IoCreateSymbolicLink failed: 0x%08lx\n", Status as u32);
        // Non-fatal — continue
    }

    let DevExt = (*DeviceObject).DeviceExtension as PARM64VID_DEVICE_EXTENSION;
    core::ptr::write_bytes(DevExt as *mut u8, 0, core::mem::size_of::<ARM64VID_DEVICE_EXTENSION>());

    Status = Arm64VidDiscoverFramebuffer(DevExt);
    #[cfg(target_arch = "aarch64")]
    if !NT_SUCCESS(Status) {
        Status = Arm64VidHeadless(DevExt);
    }
    if !NT_SUCCESS(Status) {
        // No GOP/linear framebuffer in the ARC config tree — normal on a
        // legacy-BIOS boot where the VGA miniport owns the display. We MUST
        // bail rather than steal \Device\Video0 + point framebuf.dll at
        // invalid memory. On UEFI the framebuffer is always present.
        DPRINT1!(
            "[arm64vid] No ARC framebuffer (BIOS/no-GOP boot): 0x%08lx - bailing, leaving display to VGA\n",
            Status as u32
        );
        IoDeleteSymbolicLink(&mut SymlinkName);
        IoDeleteDevice(DeviceObject);
        return Status;
    }

    DPRINT1!(
        "[arm64vid] Framebuffer: %ldx%ld %ld BPP, size=0x%lx, stride=%ld\n",
        (*DevExt).ScreenWidth,
        (*DevExt).ScreenHeight,
        (*DevExt).BitsPerPixel,
        (*DevExt).FrameBufferSize,
        (*DevExt).ScreenStride
    );

    Status = Arm64VidWriteDeviceMap();
    if !NT_SUCCESS(Status) {
        DPRINT1!("[arm64vid] Failed to write DEVICEMAP\\VIDEO: 0x%08lx\n", Status as u32);
        // Non-fatal
    }

    Status = Arm64VidWriteServiceConfig(DevExt);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[arm64vid] Failed to write service config: 0x%08lx\n", Status as u32);
    }

    DPRINT1!("[arm64vid] DriverEntry SUCCESS\n");
    STATUS_SUCCESS
}

/// arm64 with no firmware framebuffer: a server (EC2 Graviton has no display). An
/// arm64 machine has no VGA to fall back to, and win32k bugchecks (0xB4,
/// VIDEO_DRIVER_INIT_FAILURE) without a display device, so the desktop runs on a
/// frame in memory that nothing scans out. Everything above the driver works the
/// same; the session is reached over the network or watched over serial.
#[cfg(target_arch = "aarch64")]
unsafe fn Arm64VidHeadless(DevExt: PARM64VID_DEVICE_EXTENSION) -> NTSTATUS {
    const WIDTH: ULONG = 1024;
    const HEIGHT: ULONG = 768;
    let size = WIDTH * HEIGHT * 4;
    let frame = ExAllocatePoolWithTag(NonPagedPool, size as _, TAG_AVID);
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
    DPRINT1!("[arm64vid] no firmware framebuffer: headless, %lux%lu in memory\n", WIDTH, HEIGHT);
    STATUS_SUCCESS
}

// IRP_MJ_CREATE / IRP_MJ_CLOSE — just succeed
unsafe extern "system" fn Arm64VidCreateClose(_DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

// IRP_MJ_DEVICE_CONTROL — handle video IOCTLs
unsafe extern "system" fn Arm64VidDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    let DevExt = (*DeviceObject).DeviceExtension as PARM64VID_DEVICE_EXTENSION;
    let IoControlCode = (*IrpSp).Parameters.DeviceIoControl.IoControlCode;
    let InputBuffer = (*Irp).AssociatedIrp.SystemBuffer;
    let OutputBuffer = (*Irp).AssociatedIrp.SystemBuffer;
    let InputLength = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;
    let OutputLength = (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Information: ULONG = 0;

    match IoControlCode {
        IOCTL_VIDEO_INIT_WIN32K_CALLBACKS => {
            DPRINT1!("[arm64vid] IOCTL_VIDEO_INIT_WIN32K_CALLBACKS\n");
            if (InputLength as usize) < core::mem::size_of::<VIDEO_WIN32K_CALLBACKS>()
                || (OutputLength as usize) < core::mem::size_of::<VIDEO_WIN32K_CALLBACKS>()
            {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let pCallbacks = InputBuffer as *mut VIDEO_WIN32K_CALLBACKS;
                (*DevExt).PhysDisp = (*pCallbacks).PhysDisp;
                (*DevExt).Callout = (*pCallbacks).Callout as PVOID;
                (*pCallbacks).bACPI = FALSE as ULONG;
                (*pCallbacks).pPhysDeviceObject = DeviceObject as HANDLE;
                (*pCallbacks).DualviewFlags = 0;
                Information = core::mem::size_of::<VIDEO_WIN32K_CALLBACKS>() as ULONG;
            }
        }

        IOCTL_VIDEO_QUERY_NUM_AVAIL_MODES => {
            DPRINT1!("[arm64vid] IOCTL_VIDEO_QUERY_NUM_AVAIL_MODES\n");
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
            DPRINT1!("[arm64vid] IOCTL_VIDEO_QUERY_AVAIL/CURRENT_MODE(S)\n");
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
                (*pModeInfo).NumberRedBits = 8;
                (*pModeInfo).NumberGreenBits = 8;
                (*pModeInfo).NumberBlueBits = 8;
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
            DPRINT1!("[arm64vid] IOCTL_VIDEO_SET_CURRENT_MODE\n");
            // Mode already set by UEFI GOP — nothing to do
            Status = STATUS_SUCCESS;
        }

        IOCTL_VIDEO_MAP_VIDEO_MEMORY => {
            DPRINT1!("[arm64vid] IOCTL_VIDEO_MAP_VIDEO_MEMORY\n");
            if (InputLength as usize) < core::mem::size_of::<VIDEO_MEMORY>()
                || (OutputLength as usize) < core::mem::size_of::<VIDEO_MEMORY_INFORMATION>()
            {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let pVideoMemoryInfo = OutputBuffer as *mut VIDEO_MEMORY_INFORMATION;
                if (*DevExt).FrameBufferPhysical.QuadPart == 0 && (*DevExt).Headless == 0 {
                    DPRINT1!("[arm64vid] No framebuffer physical address!\n");
                    Status = STATUS_UNSUCCESSFUL;
                } else {
                    if (*DevExt).FrameBufferMapped.is_null() {
                        (*DevExt).FrameBufferMappedSize = (*DevExt).FrameBufferSize;
                        (*DevExt).FrameBufferMapped = MmMapIoSpace(
                            (*DevExt).FrameBufferPhysical,
                            (*DevExt).FrameBufferMappedSize as _,
                            MmNonCached,
                        );
                        if (*DevExt).FrameBufferMapped.is_null() {
                            DPRINT1!("[arm64vid] MmMapIoSpace failed\n");
                            Status = STATUS_UNSUCCESSFUL;
                        } else {
                            DPRINT1!("[arm64vid] Framebuffer mapped at %p (size 0x%lx)\n",
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
            DPRINT1!("[arm64vid] IOCTL_VIDEO_UNMAP_VIDEO_MEMORY\n");
            // A headless frame is pool, kept for the next MAP.
            if !(*DevExt).FrameBufferMapped.is_null() && (*DevExt).Headless == 0 {
                MmUnmapIoSpace((*DevExt).FrameBufferMapped, (*DevExt).FrameBufferMappedSize as _);
                (*DevExt).FrameBufferMapped = core::ptr::null_mut();
                (*DevExt).FrameBufferMappedSize = 0;
            }
        }

        IOCTL_VIDEO_RESET_DEVICE => {
            DPRINT1!("[arm64vid] IOCTL_VIDEO_RESET_DEVICE\n");
            // No-op
        }

        IOCTL_VIDEO_SET_COLOR_REGISTERS => {
            DPRINT1!("[arm64vid] IOCTL_VIDEO_SET_COLOR_REGISTERS (no-op for 32bpp)\n");
            // No-op for 32bpp direct color
        }

        IOCTL_VIDEO_QUERY_COLOR_CAPABILITIES => {
            DPRINT1!("[arm64vid] IOCTL_VIDEO_QUERY_COLOR_CAPABILITIES\n");
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
            DPRINT1!("[arm64vid] Unhandled IOCTL 0x%08lx\n", IoControlCode);
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

unsafe fn Arm64VidDiscoverFramebuffer(DevExt: PARM64VID_DEVICE_EXTENSION) -> NTSTATUS {
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
                DPRINT1!("[arm64vid] Found DisplayController at adapter %lu instance %lu\n",
                    AdapterIdx, InstanceIdx);
                Found = true;
                break 'outer;
            }
        }
    }

    if !Found {
        DPRINT1!("[arm64vid] DisplayController not found in ARC config tree\n");
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
        DPRINT1!("[arm64vid] No Configuration Data value: 0x%08lx\n", Status as u32);
        ZwClose(hDisplay);
        return STATUS_NOT_FOUND;
    }

    let BufferSize = ResultLength;
    let Buffer = ExAllocatePoolWithTag(PagedPool, BufferSize as _, TAG_AVID) as *mut u8;
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
        DPRINT1!("[arm64vid] Failed to read Configuration Data: 0x%08lx\n", Status as u32);
        ExFreePoolWithTag(Buffer as PVOID, TAG_AVID);
        return Status;
    }

    let ValueInfo = Buffer as PKEY_VALUE_PARTIAL_INFORMATION;
    if ((*ValueInfo).DataLength as usize) < core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>() {
        DPRINT1!("[arm64vid] Configuration Data too small: %lu bytes\n", (*ValueInfo).DataLength);
        ExFreePoolWithTag(Buffer as PVOID, TAG_AVID);
        return STATUS_INVALID_PARAMETER;
    }

    let FullResDesc = core::ptr::addr_of!((*ValueInfo).Data) as PCM_FULL_RESOURCE_DESCRIPTOR;
    let PartialList = core::ptr::addr_of!((*FullResDesc).PartialResourceList) as *const CM_PARTIAL_RESOURCE_LIST;
    let count = (*PartialList).Count;

    DPRINT1!("[arm64vid] Configuration Data: %lu partial descriptors\n", count);

    let mut i: u32 = 0;
    while i < count {
        // PartialDescriptors is a flexible array of CM_PARTIAL_RESOURCE_DESCRIPTOR.
        let PartialDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
            core::ptr::addr_of!((*PartialList).PartialDescriptors) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
        let PartialDesc = PartialDesc.add(i as usize);

        if (*PartialDesc).Type == CmResourceTypeMemory as u8 {
            (*DevExt).FrameBufferPhysical.QuadPart = (*PartialDesc).u.Memory.Start.QuadPart;
            (*DevExt).FrameBufferSize = (*PartialDesc).u.Memory.Length;
            DPRINT1!("[arm64vid] Memory resource: size=0x%lx\n", (*DevExt).FrameBufferSize);
        } else if (*PartialDesc).Type == CmResourceTypeDeviceSpecific as u8 {
            // The CM_FRAMEBUF_DEVICE_DATA follows immediately after this descriptor.
            let FramebufData = PartialDesc.add(1) as PCM_FRAMEBUF_DEVICE_DATA;
            (*DevExt).ScreenWidth = (*FramebufData).ScreenWidth;
            (*DevExt).ScreenHeight = (*FramebufData).ScreenHeight;
            (*DevExt).PixelsPerScanLine = (*FramebufData).PixelsPerScanLine;
            (*DevExt).BitsPerPixel = (*FramebufData).BitsPerPixel;
            (*DevExt).RedMask = (*FramebufData).PixelMasks.RedMask;
            (*DevExt).GreenMask = (*FramebufData).PixelMasks.GreenMask;
            (*DevExt).BlueMask = (*FramebufData).PixelMasks.BlueMask;
            (*DevExt).ScreenStride = (*DevExt).PixelsPerScanLine * ((*DevExt).BitsPerPixel / 8);
            DPRINT1!("[arm64vid] Display: %lux%lu, %lu BPP, stride=%lu\n",
                (*DevExt).ScreenWidth, (*DevExt).ScreenHeight, (*DevExt).BitsPerPixel, (*DevExt).ScreenStride);
        }
        i += 1;
    }

    ExFreePoolWithTag(Buffer as PVOID, TAG_AVID);

    if (*DevExt).FrameBufferPhysical.QuadPart == 0 || (*DevExt).ScreenWidth == 0 {
        DPRINT1!("[arm64vid] Incomplete framebuffer info from ARC tree\n");
        return STATUS_NOT_FOUND;
    }

    if (*DevExt).FrameBufferSize == 0 {
        (*DevExt).FrameBufferSize = (*DevExt).ScreenStride * (*DevExt).ScreenHeight;
    }

    STATUS_SUCCESS
}

// ---- Write HARDWARE\DEVICEMAP\VIDEO ----

unsafe fn Arm64VidWriteDeviceMap() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hDeviceMap: HANDLE = core::ptr::null_mut();
    let mut hVideo: HANDLE = core::ptr::null_mut();

    let devicemap = wsz!(b"\\Registry\\Machine\\HARDWARE\\DEVICEMAP");
    let video = wsz!(b"VIDEO");
    let dev0_name = wsz!(b"\\Device\\Video0");
    let dev0_path = wsz!(b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\arm64vid\\Device0");
    let max_obj = wsz!(b"MaxObjectNumber");

    Status = RegCreateKey(core::ptr::null_mut(), devicemap.as_ptr(), &mut hDeviceMap);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[arm64vid] Cannot create DEVICEMAP key: 0x%08lx\n", Status as u32);
        return Status;
    }

    Status = RegCreateKey(hDeviceMap, video.as_ptr(), &mut hVideo);
    ZwClose(hDeviceMap);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[arm64vid] Cannot create VIDEO key: 0x%08lx\n", Status as u32);
        return Status;
    }

    Status = RegSetSz(hVideo, dev0_name.as_ptr(), dev0_path.as_ptr());
    if !NT_SUCCESS(Status) {
        DPRINT1!("[arm64vid] Failed to write \\Device\\Video0 value: 0x%08lx\n", Status as u32);
    }

    Status = RegSetDword(hVideo, max_obj.as_ptr(), 0);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[arm64vid] Failed to write MaxObjectNumber: 0x%08lx\n", Status as u32);
    }

    ZwClose(hVideo);
    DPRINT1!("[arm64vid] DEVICEMAP\\VIDEO written\n");
    STATUS_SUCCESS
}

// ---- Write driver service config ----

unsafe fn Arm64VidWriteServiceConfig(DevExt: PARM64VID_DEVICE_EXTENSION) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hArm64vid: HANDLE = core::ptr::null_mut();
    let mut hDevice0: HANDLE = core::ptr::null_mut();

    let svc = wsz!(b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\arm64vid");
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

    Status = RegCreateKey(core::ptr::null_mut(), svc.as_ptr(), &mut hArm64vid);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[arm64vid] Cannot open/create arm64vid service key: 0x%08lx\n", Status as u32);
        return Status;
    }

    Status = RegCreateKey(hArm64vid, device0.as_ptr(), &mut hDevice0);
    ZwClose(hArm64vid);
    if !NT_SUCCESS(Status) {
        DPRINT1!("[arm64vid] Cannot create Device0 key: 0x%08lx\n", Status as u32);
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
    DPRINT1!("[arm64vid] Service config written\n");
    STATUS_SUCCESS
}
