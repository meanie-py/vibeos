// rustconv-assembly: whole-tu
//! ntoskrnl/io/pnpmgr/pnpprop.c — the DEVPROPKEY property store for devices and device
//! interfaces (upstream ReactOS 3cc9aa99292): IoGet/SetDevicePropertyData,
//! IoGet/SetDeviceInterfacePropertyData, PiPropReleaseDevice and the private PiProp* helpers.
//!
//! Bodies are a sealed translation; the file-scope preamble (defines, typedefs, tables) was
//! rebuilt by hand from the C, entry for entry. Header constants the kernel bindgen island does
//! not carry (devpropdef.h, devpkey.h, regstr.h, iotypes.h DNF_*, a few ntstatus.h codes) are
//! copied here verbatim from those headers; a local item shadows the glob imports below, so a
//! later island regeneration that adds them cannot conflict.
use ntkrnl_sys::base::{APC_LEVEL, FALSE, NT_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_SUCCESS, TRUE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::inline::common::{
    InsertTailList, RemoveEntryList, RtlEqualMemory, RtlInitEmptyUnicodeString,
};
use ntkrnl_sys::inline::ex::{
    ExAcquirePushLockExclusive, ExAcquirePushLockShared, ExReleasePushLockExclusive,
    ExReleasePushLockShared,
};
use ntkrnl_sys::inline::ke::{KeEnterCriticalRegion, KeLeaveCriticalRegion};
use ntkrnl_sys::bugcodes::PNP_DETECTED_FATAL_ERROR;
use ntkrnl_sys::ke::KeGetCurrentIrql;
use ntkrnl_sys::mm::ALIGN_UP_BY;
use ntkrnl_sys::w;

// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_04BEA12A5F1F: [u16; 5] = [0x005Cu16, 0x005C, 0x003F, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_05ECCD5C2679: [u16; 8] = [0x0043u16, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_2BE316E67F49: [u16; 48] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0045, 0x006E, 0x0075, 0x006D, 0x0000,
];
pub static RUSTCONV_WSTR_36EBA83186E3: [u16; 7] = [0x004Cu16, 0x0069, 0x006E, 0x006B, 0x0065, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_665AC3C75B1B: [u16; 52] = [
    0x007Bu16, 0x0025, 0x0030, 0x0038, 0x006C, 0x0078, 0x002D, 0x0025, 0x0030, 0x0034,
    0x0078, 0x002D, 0x0025, 0x0030, 0x0034, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032,
    0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078,
    0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030,
    0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078,
    0x007D, 0x0000,
];
pub static RUSTCONV_WSTR_8768144DD79E: [u16; 11] = [
    0x0025u16, 0x0073, 0x005C, 0x0025, 0x0077, 0x005A, 0x005C, 0x0025, 0x0077, 0x005A,
    0x0000,
];
pub static RUSTCONV_WSTR_C554B73F8939: [u16; 5] = [0x005Cu16, 0x003F, 0x003F, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_CD183236F6BC: [u16; 6] = [0x0052u16, 0x004F, 0x004F, 0x0054, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_DE4DB6B4EC92: [u16; 6] = [0x0025u16, 0x0030, 0x0034, 0x006C, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_E9A256AB2F78: [u16; 15] = [
    0x0044u16, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0049, 0x006E, 0x0073, 0x0074,
    0x0061, 0x006E, 0x0063, 0x0065, 0x0000,
];

// ---- debug macros (sdk/include/reactos/debug.h, xdk/kefuncs.h) ----

/// debug.h `DPRINT1` under DBG: `DbgPrint("(%s:%d) " fmt, __RELFILE__, __LINE__, ...)`, and a
/// second print when the first fails. NDEBUG silences only DPRINT, never DPRINT1. A macro, never
/// a fn — an out-of-line no-op is an optimization barrier. `line!()` is this file's line.
macro_rules! DPRINT1 {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{
        if crate::dbg_print!(
            concat!("(%s:%d) ", $fmt).as_bytes(),
            b"ntoskrnl/io/pnpmgr/pnpprop.c\0".as_ptr(),
            line!()
            $(, $arg)*
        ) != 0
        {
            crate::dbg_print!(
                b"(%s:%d) DbgPrint() failed!\n",
                b"ntoskrnl/io/pnpmgr/pnpprop.c\0".as_ptr(),
                line!()
            );
        }
    }};
}

/// xdk/kefuncs.h `PAGED_CODE()` under DBG: complain and assert when entered above APC_LEVEL.
macro_rules! PAGED_CODE {
    () => {{
        if KeGetCurrentIrql() > APC_LEVEL as KIRQL {
            crate::dbg_print!(
                b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n",
                KeGetCurrentIrql() as i32
            );
            ASSERT!(false);
        }
    }};
}

/// ntdef.h `RTL_NUMBER_OF` — element count of an array expression.
macro_rules! RTL_NUMBER_OF {
    ($a:expr) => {
        ::core::mem::size_of_val(&$a) / ::core::mem::size_of_val(&$a[0])
    };
}

/// ntdef.h `RTL_CONSTANT_STRING(s)` = `{ sizeof(s) - sizeof(s[0]), sizeof(s), s }` for a
/// NUL-terminated wide literal.
const fn RTL_CONSTANT_STRING(s: &'static [u16]) -> UNICODE_STRING {
    UNICODE_STRING {
        Length: ((s.len() - 1) * ::core::mem::size_of::<WCHAR>()) as USHORT,
        MaximumLength: (s.len() * ::core::mem::size_of::<WCHAR>()) as USHORT,
        Buffer: s.as_ptr() as PWCH,
    }
}

// ---- ntstrsafe.h printf entry points ----
//
// The C calls the ntstrsafe.h FORCEINLINE wrappers, which bottom out in _vsnwprintf. Here the
// format is evaluated by ntprintf_w (the kernel crate's wide printf), with the exact ntstrsafe
// validation and termination rules restated below.

/// Wide sink for `RtlpArrayVPrintfWorkerW`: `_vsnwprintf(pszDest, cchDest, ...)` writes at most
/// `cchDest` characters and adds no terminator once the buffer is full.
struct PiPropWideSink {
    Dest: *mut WCHAR,
    Cap: usize,
    Pos: usize,
    Total: usize,
}

impl ntprintf_w::WriterW for PiPropWideSink {
    fn write_chars(&mut self, chars: &[u16]) {
        self.Total += chars.len();
        for &c in chars {
            if self.Pos >= self.Cap {
                break;
            }
            unsafe { *self.Dest.add(self.Pos) = c };
            self.Pos += 1;
        }
    }
    fn written(&self) -> usize {
        self.Total
    }
}

fn PiPropStripNul(Format: &[u16]) -> &[u16] {
    match Format.split_last() {
        Some((&0, Rest)) => Rest,
        _ => Format,
    }
}

/// ntstrsafe.h `NTSTRSAFE_MAX_CCH`.
const NTSTRSAFE_MAX_CCH: usize = 2147483647;
/// ntstrsafe.h `NTSTRSAFEUNICODE_STRING_MAX_CCH`.
const NTSTRSAFEUNICODE_STRING_MAX_CCH: usize = 0xffff / ::core::mem::size_of::<WCHAR>();

/// ntstrsafe.h `RtlStringCchPrintfW`: validate `cchDest` (`RtlStringValidateDestW`), then
/// `RtlpStringVPrintfWorkerW` — format into `cchDest - 1` characters and always terminate;
/// truncation is STATUS_BUFFER_OVERFLOW.
unsafe fn RtlStringCchPrintfW(
    pszDest: *mut WCHAR,
    cchDest: usize,
    pszFormat: &[u16],
    Args: &[u64],
) -> NTSTATUS {
    if cchDest == 0 || cchDest > NTSTRSAFE_MAX_CCH {
        return STATUS_INVALID_PARAMETER;
    }
    let cchMax = cchDest - 1;
    let mut Sink = PiPropWideSink { Dest: pszDest, Cap: cchMax, Pos: 0, Total: 0 };
    let iRet = ntprintf_w::format_into(&mut Sink, PiPropStripNul(pszFormat), Args);
    if iRet > cchMax {
        *pszDest.add(cchMax) = UNICODE_NULL;
        STATUS_BUFFER_OVERFLOW
    } else {
        // iRet == cchMax: _vsnwprintf left no room, terminate at the end; otherwise it did.
        *pszDest.add(iRet) = UNICODE_NULL;
        STATUS_SUCCESS
    }
}

/// ntstrsafe.h `RtlUnicodeStringPrintf`: validate the destination (`RtlUnicodeStringValidate`),
/// format into all `MaximumLength / sizeof(WCHAR)` characters without a terminator
/// (`RtlpArrayVPrintfWorkerW`), and set `Length` on success or on overflow.
unsafe fn RtlUnicodeStringPrintf(
    DestinationString: PUNICODE_STRING,
    pszFormat: &[u16],
    Args: &[u64],
) -> NTSTATUS {
    let W = ::core::mem::size_of::<WCHAR>();
    if DestinationString.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let Dest = &mut *DestinationString;
    if (Dest.Length as usize) % W != 0
        || (Dest.MaximumLength as usize) % W != 0
        || Dest.Length > Dest.MaximumLength
        || (Dest.MaximumLength as usize) > NTSTRSAFEUNICODE_STRING_MAX_CCH * W
        || (Dest.Buffer.is_null() && (Dest.Length != 0 || Dest.MaximumLength != 0))
    {
        return STATUS_INVALID_PARAMETER;
    }
    let cchDest = Dest.MaximumLength as usize / W;
    let mut Sink = PiPropWideSink { Dest: Dest.Buffer, Cap: cchDest, Pos: 0, Total: 0 };
    let iRet = ntprintf_w::format_into(&mut Sink, PiPropStripNul(pszFormat), Args);
    let (Status, cchFinalLength) = if iRet > cchDest {
        (STATUS_BUFFER_OVERFLOW, cchDest)
    } else {
        (STATUS_SUCCESS, iRet)
    };
    Dest.Length = (cchFinalLength * W) as USHORT;
    Status
}

// ---- header constants not in the kernel island ----

// psdk/devpropdef.h
const DEVPROP_TYPEMOD_ARRAY: DEVPROPTYPE = 0x00001000;
const DEVPROP_TYPEMOD_LIST: DEVPROPTYPE = 0x00002000;
const DEVPROP_TYPE_EMPTY: DEVPROPTYPE = 0x00000000;
const DEVPROP_TYPE_NULL: DEVPROPTYPE = 0x00000001;
const DEVPROP_TYPE_UINT32: DEVPROPTYPE = 0x00000007;
const DEVPROP_TYPE_GUID: DEVPROPTYPE = 0x0000000D;
const DEVPROP_TYPE_BOOLEAN: DEVPROPTYPE = 0x00000011;
const DEVPROP_TYPE_STRING: DEVPROPTYPE = 0x00000012;
const DEVPROP_TYPE_STRING_LIST: DEVPROPTYPE = DEVPROP_TYPE_STRING | DEVPROP_TYPEMOD_LIST;
const DEVPROP_TYPE_STRING_INDIRECT: DEVPROPTYPE = 0x00000019;
const MAX_DEVPROP_TYPE: DEVPROPTYPE = 0x00000019;
const DEVPROP_MASK_TYPE: DEVPROPTYPE = 0x00000FFF;
const DEVPROP_TRUE: DEVPROP_BOOLEAN = -1;
const DEVPROP_FALSE: DEVPROP_BOOLEAN = 0;

// psdk/regstr.h
const REGSTR_VAL_CLASS: &[u16] = w!("Class");
const REGSTR_VAL_DEVDESC: &[u16] = w!("DeviceDesc");
const REGSTR_VAL_COMPATIBLEIDS: &[u16] = w!("CompatibleIDs");
const REGSTR_VAL_MFG: &[u16] = w!("Mfg");
const REGSTR_VAL_DRIVER: &[u16] = w!("Driver");
const REGSTR_VAL_CONFIGFLAGS: &[u16] = w!("ConfigFlags");
const REGSTR_VAL_CAPABILITIES: &[u16] = w!("Capabilities");
const REGSTR_VAL_CLASSGUID: &[u16] = w!("ClassGUID");
const REGSTR_VAL_LOCATION_INFORMATION: &[u16] = w!("LocationInformation");
const REGSTR_VAL_LOWERFILTERS: &[u16] = w!("LowerFilters");
const REGSTR_VAL_SERVICE: &[u16] = w!("Service");
const REGSTR_VAL_UI_NUMBER: &[u16] = w!("UINumber");
const REGSTR_VAL_UPPERFILTERS: &[u16] = w!("UpperFilters");
const REGSTR_VAL_FRIENDLYNAME: &[u16] = w!("FriendlyName");
const REGSTR_VAL_HARDWAREID: &[u16] = w!("HardwareID");
const REGSTR_KEY_DEVICE_PROPERTIES: &[u16] = w!("Properties");

// ndk/iotypes.h
const DNF_HAS_PROBLEM: ULONG = 0x00002000;
const DNF_DEVICE_GONE: ULONG = 0x00010000;
const DNF_LEGACY_RESOURCE_DEVICENODE: ULONG = 0x00020000;

// psdk/ntstatus.h
const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5_u32 as NTSTATUS;
const STATUS_CHILD_MUST_BE_VOLATILE: NTSTATUS = 0xC0000181_u32 as NTSTATUS;

// xdk/ntbasedef.h, psdk/winnt.h
const UNICODE_NULL: WCHAR = 0;
const UNICODE_STRING_MAX_BYTES: USHORT = 65534;
const LOCALE_NAME_MAX_LENGTH: usize = 85;
/// MAKELCID(MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL), SORT_DEFAULT)
const LOCALE_NEUTRAL: LCID = 0;
const REG_OPTION_NON_VOLATILE: ULONG = 0x00000000;
const REG_OPTION_VOLATILE: ULONG = 0x00000001;
const REG_CREATED_NEW_KEY: ULONG = 0x00000001;

// Enum members the C spells bare; bindgen emits them only under their enum's prefix.
use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyAddress as DevicePropertyAddress;
use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyBusNumber as DevicePropertyBusNumber;
use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyBusTypeGuid as DevicePropertyBusTypeGuid;
use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyEnumeratorName as DevicePropertyEnumeratorName;
use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyLegacyBusType as DevicePropertyLegacyBusType;
use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyPhysicalDeviceObjectName as DevicePropertyPhysicalDeviceObjectName;
use ntkrnl_sys::ddk::DEVICE_REGISTRY_PROPERTY_DevicePropertyRemovalPolicy as DevicePropertyRemovalPolicy;
use ntkrnl_sys::ddk::_KEY_INFORMATION_CLASS_KeyFullInformation as KeyFullInformation;
use ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation;
use ntkrnl_sys::ddk::_PNP_DEVNODE_STATE_DeviceNodeInitialized as DeviceNodeInitialized;
use ntkrnl_sys::ddk::_PNP_DEVNODE_STATE_DeviceNodeUninitialized as DeviceNodeUninitialized;
use ntkrnl_sys::ddk::_PNP_DEVNODE_STATE_DeviceNodeUnspecified as DeviceNodeUnspecified;
use ntkrnl_sys::ddk::_POOL_TYPE_PagedPool as PagedPool;

// ---- globals defined elsewhere in the kernel ----

extern "C" {
    /// ntoskrnl/io/pnpmgr/pnpmgr.c (pnpmgr-rust)
    static mut PpRegistryDeviceResource: ERESOURCE;
    /// ntoskrnl/io/pnpmgr/devnode.c (pnpmgr-rust)
    static mut IopRootDeviceNode: PDEVICE_NODE;
}

// ---- functions defined outside pnpprop.c (exact prototypes; io.h / rtlfuncs.h) ----

extern "C" {
    /// io.h: `NTSTATUS IopOpenDeviceInterfaceKeys(PCUNICODE_STRING, ACCESS_MASK, PHANDLE, PHANDLE)`
    fn IopOpenDeviceInterfaceKeys(
        SymbolicLinkName: PCUNICODE_STRING,
        DesiredAccess: ACCESS_MASK,
        DeviceKey: PHANDLE,
        InstanceKey: PHANDLE,
    ) -> NTSTATUS;
    /// io.h: `NTSTATUS IopQueueDevicePropertyChangeEvent(PCUNICODE_STRING)`
    fn IopQueueDevicePropertyChangeEvent(DeviceId: PCUNICODE_STRING) -> NTSTATUS;
    /// io.h: `ULONG IopGetDeviceNodeStatus(PDEVICE_NODE)`
    fn IopGetDeviceNodeStatus(DeviceNode: PDEVICE_NODE) -> ULONG;
}

// io.h: `PDEVICE_NODE FASTCALL IopGetDeviceNode(PDEVICE_OBJECT)`
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IopGetDeviceNode(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_NODE;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IopGetDeviceNode(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_NODE;
}

extern "system" {
    /// io.h: `NTSTATUS NTAPI IopOpenRegistryKeyEx(PHANDLE, HANDLE, PUNICODE_STRING, ACCESS_MASK)`
    fn IopOpenRegistryKeyEx(
        KeyHandle: PHANDLE,
        ParentKey: HANDLE,
        Name: PUNICODE_STRING,
        DesiredAccess: ACCESS_MASK,
    ) -> NTSTATUS;
    /// io.h: `NTSTATUS NTAPI IopCreateRegistryKeyEx(PHANDLE, HANDLE, PUNICODE_STRING, ACCESS_MASK, ULONG, PULONG)`
    fn IopCreateRegistryKeyEx(
        Handle: PHANDLE,
        BaseHandle: HANDLE,
        KeyName: PUNICODE_STRING,
        DesiredAccess: ACCESS_MASK,
        CreateOptions: ULONG,
        Disposition: PULONG,
    ) -> NTSTATUS;
    /// io.h: `NTSTATUS NTAPI IopGetRegistryValue(HANDLE, PWSTR, PKEY_VALUE_FULL_INFORMATION *)`
    fn IopGetRegistryValue(
        Handle: HANDLE,
        ValueName: PWSTR,
        Information: *mut PKEY_VALUE_FULL_INFORMATION,
    ) -> NTSTATUS;
    /// ndk/rtlfuncs.h: `BOOLEAN NTAPI RtlLCIDToCultureName(LCID, PUNICODE_STRING)`
    fn RtlLCIDToCultureName(Lcid: LCID, String: PUNICODE_STRING) -> BOOLEAN;
}

// ---- psdk/devpkey.h keys used here (values copied from the header, one per DEFINE_DEVPROPKEY) ----

const fn devpropkey(l: u32, w1: u16, w2: u16, b: [u8; 8], pid: DEVPROPID) -> DEVPROPKEY {
    DEVPROPKEY {
        fmtid: GUID { Data1: l as _, Data2: w1, Data3: w2, Data4: b },
        pid,
    }
}

static DEVPKEY_NAME: DEVPROPKEY = devpropkey(0xb725f130, 0x47ef, 0x101a, [0xa5, 0xf1, 0x02, 0x60, 0x8c, 0x9e, 0xeb, 0xac], 10);
static DEVPKEY_Device_DeviceDesc: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 2);
static DEVPKEY_Device_HardwareIds: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 3);
static DEVPKEY_Device_CompatibleIds: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 4);
static DEVPKEY_Device_Service: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 6);
static DEVPKEY_Device_Class: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 9);
static DEVPKEY_Device_ClassGuid: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 10);
static DEVPKEY_Device_Driver: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 11);
static DEVPKEY_Device_ConfigFlags: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 12);
static DEVPKEY_Device_Manufacturer: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 13);
static DEVPKEY_Device_FriendlyName: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 14);
static DEVPKEY_Device_LocationInfo: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 15);
static DEVPKEY_Device_Capabilities: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 17);
static DEVPKEY_Device_UINumber: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 18);
static DEVPKEY_Device_UpperFilters: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 19);
static DEVPKEY_Device_LowerFilters: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 20);
static DEVPKEY_Device_PDOName: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 16);
static DEVPKEY_Device_BusTypeGuid: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 21);
static DEVPKEY_Device_LegacyBusType: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 22);
static DEVPKEY_Device_BusNumber: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 23);
static DEVPKEY_Device_EnumeratorName: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 24);
static DEVPKEY_Device_Address: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 30);
static DEVPKEY_Device_RemovalPolicy: DEVPROPKEY = devpropkey(0xa45c254e, 0xdf1c, 0x4efd, [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0], 33);
static DEVPKEY_Device_InstanceId: DEVPROPKEY = devpropkey(0x78c34fc8, 0x104a, 0x4aca, [0x9e, 0xa4, 0x52, 0x4d, 0x52, 0x99, 0x6e, 0x57], 256);
static DEVPKEY_Device_Parent: DEVPROPKEY = devpropkey(0x4340a6c5, 0x93fa, 0x4706, [0x97, 0x2c, 0x7b, 0x64, 0x80, 0x08, 0xa5, 0xa7], 8);
static DEVPKEY_Device_DevNodeStatus: DEVPROPKEY = devpropkey(0x4340a6c5, 0x93fa, 0x4706, [0x97, 0x2c, 0x7b, 0x64, 0x80, 0x08, 0xa5, 0xa7], 2);
static DEVPKEY_Device_ProblemCode: DEVPROPKEY = devpropkey(0x4340a6c5, 0x93fa, 0x4706, [0x97, 0x2c, 0x7b, 0x64, 0x80, 0x08, 0xa5, 0xa7], 3);
static DEVPKEY_Device_HasProblem: DEVPROPKEY = devpropkey(0x540b947e, 0x8b40, 0x45bc, [0xa8, 0xa2, 0x6a, 0x0b, 0x89, 0x4c, 0xbd, 0xa2], 6);
static DEVPKEY_Device_IsPresent: DEVPROPKEY = devpropkey(0x540b947e, 0x8b40, 0x45bc, [0xa8, 0xa2, 0x6a, 0x0b, 0x89, 0x4c, 0xbd, 0xa2], 5);
static DEVPKEY_DeviceInterface_ClassGuid: DEVPROPKEY = devpropkey(0x026e516e, 0xb814, 0x414b, [0x83, 0xcd, 0x85, 0x6d, 0x6f, 0xef, 0x48, 0x22], 4);
static DEVPKEY_DeviceInterface_Enabled: DEVPROPKEY = devpropkey(0x026e516e, 0xb814, 0x414b, [0x83, 0xcd, 0x85, 0x6d, 0x6f, 0xef, 0x48, 0x22], 3);
static DEVPKEY_DeviceInterface_ReferenceString: DEVPROPKEY = devpropkey(0x026e516e, 0xb814, 0x414b, [0x83, 0xcd, 0x85, 0x6d, 0x6f, 0xef, 0x48, 0x22], 5);

// ---- DEFINES ----

/// 'rPpP'
const TAG_PNP_PROPERTY: ULONG = 0x72507050;

/// Store values: <ObjectKey>\Properties\{FMTID}\<PID>, type PIP_PROP_REG_TYPE_MARK | DEVPROPTYPE
const PIP_PROP_REG_TYPE_MARK: ULONG = 0xFFFF0000;

const PIP_PROP_FMTID_CHARS: usize = 38 + 1;
const PIP_PROP_PID_CHARS: usize = 8 + 1;
const PIP_PROP_PATH_CHARS: usize = 10 + 1 + 38 + 1 + 8 + 1;
const PIP_PROP_MAX_QUERY_SIZE: ULONG = 0x10000;

const PIP_PROP_READ_ONLY: ULONG = 0x00000001;
const PIP_PROP_ROOT_ENUM_WRITE: ULONG = 0x00000002;

const PIP_PROP_OBJECT_PROTECTED: ULONG = 0x00000001;

// ---- TYPES ----

type PIP_PROP_OBJECT_KIND = ::core::ffi::c_int;
const PipPropObjectDevice: PIP_PROP_OBJECT_KIND = 0;
const PipPropObjectInterface: PIP_PROP_OBJECT_KIND = 1;

#[repr(C)]
#[derive(Copy, Clone)]
struct PIP_PROP_OBJECT {
    Kind: PIP_PROP_OBJECT_KIND,
    Flags: ULONG,
    Pdo: PDEVICE_OBJECT,
    DeviceNode: PDEVICE_NODE,
    Name: PCUNICODE_STRING,
    ClassGuid: GUID,
}
type PPIP_PROP_OBJECT = *mut PIP_PROP_OBJECT;

#[repr(C)]
#[derive(Copy, Clone)]
struct PIP_PROP_NAME {
    Key: *const DEVPROPKEY,
    Locale: UNICODE_STRING,
    Fmtid: UNICODE_STRING,
    Pid: UNICODE_STRING,
    LocaleBuffer: [WCHAR; LOCALE_NAME_MAX_LENGTH],
    FmtidBuffer: [WCHAR; PIP_PROP_FMTID_CHARS],
    PidBuffer: [WCHAR; PIP_PROP_PID_CHARS],
}
type PPIP_PROP_NAME = *mut PIP_PROP_NAME;

type PIP_PROP_SOURCE = ::core::ffi::c_int;
const PipSourceKeyValue: PIP_PROP_SOURCE = 0;
const PipSourceRegistryProperty: PIP_PROP_SOURCE = 1;
const PipSourceDeviceNode: PIP_PROP_SOURCE = 2;
const PipSourceInterface: PIP_PROP_SOURCE = 3;

type PIP_PROP_FIELD = ::core::ffi::c_int;
const PipFieldName: PIP_PROP_FIELD = 0;
const PipFieldInstanceId: PIP_PROP_FIELD = 1;
const PipFieldParent: PIP_PROP_FIELD = 2;
const PipFieldStatus: PIP_PROP_FIELD = 3;
const PipFieldProblemCode: PIP_PROP_FIELD = 4;
const PipFieldHasProblem: PIP_PROP_FIELD = 5;
const PipFieldIsPresent: PIP_PROP_FIELD = 6;
const PipFieldInterfaceClass: PIP_PROP_FIELD = 7;
const PipFieldInterfaceEnabled: PIP_PROP_FIELD = 8;
const PipFieldInterfaceDevice: PIP_PROP_FIELD = 9;
const PipFieldInterfaceReference: PIP_PROP_FIELD = 10;

#[repr(C)]
struct PIP_BUILTIN_PROPERTY {
    Key: *const DEVPROPKEY,
    Type: DEVPROPTYPE,
    Source: PIP_PROP_SOURCE,
    Flags: ULONG,
    ValueName: PCWSTR,
    /// REG_xxx, DEVICE_REGISTRY_PROPERTY or PIP_PROP_FIELD
    Selector: ULONG,
}
// `static const` tables of pointers to immutable statics.
unsafe impl Sync for PIP_BUILTIN_PROPERTY {}

const fn Builtin(
    Key: &'static DEVPROPKEY,
    Type: DEVPROPTYPE,
    Source: PIP_PROP_SOURCE,
    Flags: ULONG,
    ValueName: Option<&'static [u16]>,
    Selector: ULONG,
) -> PIP_BUILTIN_PROPERTY {
    PIP_BUILTIN_PROPERTY {
        Key,
        Type,
        Source,
        Flags,
        ValueName: match ValueName {
            Some(Name) => Name.as_ptr(),
            None => ::core::ptr::null(),
        },
        Selector,
    }
}

// ---- GLOBALS ----

#[rustfmt::skip]
static PipDeviceBuiltins: [PIP_BUILTIN_PROPERTY; 29] = [
    Builtin(&DEVPKEY_Device_DeviceDesc,     DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        Some(REGSTR_VAL_DEVDESC),              REG_SZ),
    Builtin(&DEVPKEY_Device_HardwareIds,    DEVPROP_TYPE_STRING_LIST, PipSourceKeyValue,         PIP_PROP_ROOT_ENUM_WRITE, Some(REGSTR_VAL_HARDWAREID),           REG_MULTI_SZ),
    Builtin(&DEVPKEY_Device_CompatibleIds,  DEVPROP_TYPE_STRING_LIST, PipSourceKeyValue,         PIP_PROP_ROOT_ENUM_WRITE, Some(REGSTR_VAL_COMPATIBLEIDS),        REG_MULTI_SZ),
    Builtin(&DEVPKEY_Device_Service,        DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        Some(REGSTR_VAL_SERVICE),              REG_SZ),
    Builtin(&DEVPKEY_Device_Class,          DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        Some(REGSTR_VAL_CLASS),                REG_SZ),
    Builtin(&DEVPKEY_Device_ClassGuid,      DEVPROP_TYPE_GUID,        PipSourceKeyValue,         0,                        Some(REGSTR_VAL_CLASSGUID),            REG_SZ),
    Builtin(&DEVPKEY_Device_Driver,         DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        Some(REGSTR_VAL_DRIVER),               REG_SZ),
    Builtin(&DEVPKEY_Device_ConfigFlags,    DEVPROP_TYPE_UINT32,      PipSourceKeyValue,         0,                        Some(REGSTR_VAL_CONFIGFLAGS),          REG_DWORD),
    Builtin(&DEVPKEY_Device_Manufacturer,   DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        Some(REGSTR_VAL_MFG),                  REG_SZ),
    Builtin(&DEVPKEY_Device_FriendlyName,   DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        Some(REGSTR_VAL_FRIENDLYNAME),         REG_SZ),
    Builtin(&DEVPKEY_Device_LocationInfo,   DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        Some(REGSTR_VAL_LOCATION_INFORMATION), REG_SZ),
    Builtin(&DEVPKEY_Device_Capabilities,   DEVPROP_TYPE_UINT32,      PipSourceKeyValue,         PIP_PROP_READ_ONLY,       Some(REGSTR_VAL_CAPABILITIES),         REG_DWORD),
    Builtin(&DEVPKEY_Device_UINumber,       DEVPROP_TYPE_UINT32,      PipSourceKeyValue,         PIP_PROP_READ_ONLY,       Some(REGSTR_VAL_UI_NUMBER),            REG_DWORD),
    Builtin(&DEVPKEY_Device_UpperFilters,   DEVPROP_TYPE_STRING_LIST, PipSourceKeyValue,         0,                        Some(REGSTR_VAL_UPPERFILTERS),         REG_MULTI_SZ),
    Builtin(&DEVPKEY_Device_LowerFilters,   DEVPROP_TYPE_STRING_LIST, PipSourceKeyValue,         0,                        Some(REGSTR_VAL_LOWERFILTERS),         REG_MULTI_SZ),

    Builtin(&DEVPKEY_Device_PDOName,        DEVPROP_TYPE_STRING,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       None, DevicePropertyPhysicalDeviceObjectName as ULONG),
    Builtin(&DEVPKEY_Device_BusTypeGuid,    DEVPROP_TYPE_GUID,        PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       None, DevicePropertyBusTypeGuid as ULONG),
    Builtin(&DEVPKEY_Device_LegacyBusType,  DEVPROP_TYPE_UINT32,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       None, DevicePropertyLegacyBusType as ULONG),
    Builtin(&DEVPKEY_Device_BusNumber,      DEVPROP_TYPE_UINT32,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       None, DevicePropertyBusNumber as ULONG),
    Builtin(&DEVPKEY_Device_EnumeratorName, DEVPROP_TYPE_STRING,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       None, DevicePropertyEnumeratorName as ULONG),
    Builtin(&DEVPKEY_Device_Address,        DEVPROP_TYPE_UINT32,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       None, DevicePropertyAddress as ULONG),
    Builtin(&DEVPKEY_Device_RemovalPolicy,  DEVPROP_TYPE_UINT32,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       None, DevicePropertyRemovalPolicy as ULONG),

    Builtin(&DEVPKEY_NAME,                  DEVPROP_TYPE_STRING,      PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       None, PipFieldName as ULONG),
    Builtin(&DEVPKEY_Device_InstanceId,     DEVPROP_TYPE_STRING,      PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       None, PipFieldInstanceId as ULONG),
    Builtin(&DEVPKEY_Device_Parent,         DEVPROP_TYPE_STRING,      PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       None, PipFieldParent as ULONG),
    Builtin(&DEVPKEY_Device_DevNodeStatus,  DEVPROP_TYPE_UINT32,      PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       None, PipFieldStatus as ULONG),
    Builtin(&DEVPKEY_Device_ProblemCode,    DEVPROP_TYPE_UINT32,      PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       None, PipFieldProblemCode as ULONG),
    Builtin(&DEVPKEY_Device_HasProblem,     DEVPROP_TYPE_BOOLEAN,     PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       None, PipFieldHasProblem as ULONG),
    Builtin(&DEVPKEY_Device_IsPresent,      DEVPROP_TYPE_BOOLEAN,     PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       None, PipFieldIsPresent as ULONG),
];

#[rustfmt::skip]
static PipInterfaceBuiltins: [PIP_BUILTIN_PROPERTY; 4] = [
    Builtin(&DEVPKEY_DeviceInterface_ClassGuid,       DEVPROP_TYPE_GUID,    PipSourceInterface,        PIP_PROP_READ_ONLY,       None, PipFieldInterfaceClass as ULONG),
    Builtin(&DEVPKEY_DeviceInterface_Enabled,         DEVPROP_TYPE_BOOLEAN, PipSourceInterface,        PIP_PROP_READ_ONLY,       None, PipFieldInterfaceEnabled as ULONG),
    Builtin(&DEVPKEY_Device_InstanceId,               DEVPROP_TYPE_STRING,  PipSourceInterface,        PIP_PROP_READ_ONLY,       None, PipFieldInterfaceDevice as ULONG),
    Builtin(&DEVPKEY_DeviceInterface_ReferenceString, DEVPROP_TYPE_STRING,  PipSourceInterface,        PIP_PROP_READ_ONLY,       None, PipFieldInterfaceReference as ULONG),
];

static PiPropInterruptKey: DEVPROPKEY =
    devpropkey(0xF0E20F09, 0xD97A, 0x49A9, [0x80, 0x46, 0xBB, 0x6E, 0x22, 0xE6, 0xBB, 0x2E], 2);

type PIP_PROP_CHECK = ::core::ffi::c_int;
const PipCheckNone: PIP_PROP_CHECK = 0;
const PipCheckNoData: PIP_PROP_CHECK = 1;
const PipCheckBoolean: PIP_PROP_CHECK = 2;
const PipCheckFileTime: PIP_PROP_CHECK = 3;
const PipCheckString: PIP_PROP_CHECK = 4;
const PipCheckSecurity: PIP_PROP_CHECK = 5;

#[repr(C)]
struct PIP_PROP_TYPE_RULE {
    Width: UCHAR,
    ListAllowed: BOOLEAN,
    Check: UCHAR,
}

const fn Rule(Width: usize, ListAllowed: BOOLEAN, Check: PIP_PROP_CHECK) -> PIP_PROP_TYPE_RULE {
    PIP_PROP_TYPE_RULE { Width: Width as UCHAR, ListAllowed, Check: Check as UCHAR }
}

use ::core::mem::size_of;

/// Indexed by base DEVPROPTYPE, a zero width means variable length
#[rustfmt::skip]
static PiPropTypeRules: [PIP_PROP_TYPE_RULE; MAX_DEVPROP_TYPE as usize + 1] = [
    Rule(0,                          FALSE, PipCheckNoData),     /* EMPTY */
    Rule(0,                          FALSE, PipCheckNoData),     /* NULL */
    Rule(size_of::<CHAR>(),          FALSE, PipCheckNone),       /* SBYTE */
    Rule(size_of::<UCHAR>(),         FALSE, PipCheckNone),       /* BYTE */
    Rule(size_of::<SHORT>(),         FALSE, PipCheckNone),       /* INT16 */
    Rule(size_of::<USHORT>(),        FALSE, PipCheckNone),       /* UINT16 */
    Rule(size_of::<LONG>(),          FALSE, PipCheckNone),       /* INT32 */
    Rule(size_of::<ULONG>(),         FALSE, PipCheckNone),       /* UINT32 */
    Rule(size_of::<LONGLONG>(),      FALSE, PipCheckNone),       /* INT64 */
    Rule(size_of::<ULONGLONG>(),     FALSE, PipCheckNone),       /* UINT64 */
    Rule(size_of::<ULONG>(),         FALSE, PipCheckNone),       /* FLOAT */
    Rule(size_of::<ULONGLONG>(),     FALSE, PipCheckNone),       /* DOUBLE */
    Rule(16,                         FALSE, PipCheckNone),       /* DECIMAL */
    Rule(size_of::<GUID>(),          FALSE, PipCheckNone),       /* GUID */
    Rule(size_of::<LONGLONG>(),      FALSE, PipCheckNone),       /* CURRENCY */
    Rule(size_of::<ULONGLONG>(),     FALSE, PipCheckNone),       /* DATE */
    Rule(size_of::<LARGE_INTEGER>(), FALSE, PipCheckFileTime),   /* FILETIME */
    Rule(size_of::<DEVPROP_BOOLEAN>(), FALSE, PipCheckBoolean),  /* BOOLEAN */
    Rule(0,                          TRUE,  PipCheckString),     /* STRING */
    Rule(0,                          FALSE, PipCheckSecurity),   /* SECURITY_DESCRIPTOR */
    Rule(0,                          TRUE,  PipCheckString),     /* SECURITY_DESCRIPTOR_STRING */
    Rule(size_of::<DEVPROPKEY>(),    FALSE, PipCheckNone),       /* DEVPROPKEY */
    Rule(size_of::<DEVPROPTYPE>(),   FALSE, PipCheckNone),       /* DEVPROPTYPE */
    Rule(size_of::<ULONG>(),         FALSE, PipCheckNone),       /* ERROR */
    Rule(size_of::<NTSTATUS>(),      FALSE, PipCheckNone),       /* NTSTATUS */
    Rule(0,                          FALSE, PipCheckString),     /* STRING_INDIRECT */
];

// ---- Saved interrupt data ----

#[repr(C)]
struct PIP_PROP_SAVED_VALUE {
    Link: LIST_ENTRY,
    Pdo: PDEVICE_OBJECT,
    Type: DEVPROPTYPE,
    Size: ULONG,
    Data: [UCHAR; 1], // ANYSIZE_ARRAY
}
type PPIP_PROP_SAVED_VALUE = *mut PIP_PROP_SAVED_VALUE;

/// RTL_STATIC_LIST_HEAD(PiPropSavedValues)
static mut PiPropSavedValues: LIST_ENTRY = LIST_ENTRY {
    Flink: &raw mut PiPropSavedValues,
    Blink: &raw mut PiPropSavedValues,
};
static mut PiPropSavedLock: EX_PUSH_LOCK = unsafe { ::core::mem::zeroed() };

// ---- Reading ----

#[repr(C)]
struct PIP_PROP_OUTPUT {
    Data: PVOID,
    Size: ULONG,
    Required: ULONG,
    Type: DEVPROPTYPE,
}
type PPIP_PROP_OUTPUT = *mut PIP_PROP_OUTPUT;

/// Returns TRUE when the source has the final answer in *Status
type PIP_PROP_READER = Option<
    unsafe extern "C" fn(
        Object: PPIP_PROP_OBJECT,
        Name: PPIP_PROP_NAME,
        Out: PPIP_PROP_OUTPUT,
        Status: PNTSTATUS,
    ) -> BOOLEAN,
>;

static PiPropReaders: [PIP_PROP_READER; 3] = [
    Some(PiPropReadFromBuiltin),
    Some(PiPropReadFromStore),
    Some(PiPropReadFromSaved),
];

// ---- Writing ----

type PIP_PROP_TARGET = ::core::ffi::c_int;
const PipTargetDenied: PIP_PROP_TARGET = 0;
const PipTargetBuiltin: PIP_PROP_TARGET = 1;
const PipTargetStore: PIP_PROP_TARGET = 2;

// ---- PRIVATE FUNCTIONS ----

unsafe extern "C" fn PiPropIsSameKey(Key1: *const DEVPROPKEY, Key2: *const DEVPROPKEY) -> BOOLEAN {
    ((*Key1).pid == (*Key2).pid
        && RtlEqualMemory(
            core::ptr::addr_of!((*Key1).fmtid) as *const core::ffi::c_void,
            core::ptr::addr_of!((*Key2).fmtid) as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        )) as BOOLEAN
}
unsafe extern "C" fn PiPropCheckElements(
    mut Check: UCHAR,
    mut Data: PUCHAR,
    mut Width: ULONG,
    mut Count: ULONG,
) -> BOOLEAN {
    let mut Time: LONGLONG = 0;
    while Count != 0 {
        if Check as ::core::ffi::c_int == PipCheckBoolean as ::core::ffi::c_int {
            if *(Data as PDEVPROP_BOOLEAN) != DEVPROP_FALSE
                && *(Data as PDEVPROP_BOOLEAN) != DEVPROP_TRUE
            {
                return FALSE;
            }
        } else if Check as ::core::ffi::c_int == PipCheckFileTime as ::core::ffi::c_int {
            core::ptr::copy_nonoverlapping(
                Data as *const u8,
                &raw mut Time as *mut u8,
                ::core::mem::size_of::<LONGLONG>(),
            );
            if Time < 0 {
                return FALSE;
            }
        }
        Count = Count.wrapping_sub(1);
        Data = Data.offset(Width as isize);
    }
    return TRUE;
}
unsafe extern "C" fn PiPropIsValidString(String: PCWSTR, Size: ULONG) -> BOOLEAN {
    let mut Chars: ULONG;
    let mut Index: ULONG;

    if (Size < core::mem::size_of::<WCHAR>() as ULONG)
        || (Size % core::mem::size_of::<WCHAR>() as ULONG) != 0
        || (Size > UNICODE_STRING_MAX_BYTES as ULONG)
    {
        return FALSE;
    }

    Chars = Size / core::mem::size_of::<WCHAR>() as ULONG;
    Index = 0;
    while Index < Chars - 1 {
        if *String.offset(Index as isize) == UNICODE_NULL {
            return FALSE;
        }
        Index += 1;
    }

    (*String.offset((Chars - 1) as isize) == UNICODE_NULL) as BOOLEAN
}
unsafe extern "C" fn PiPropIsValidStringList(List: PCWSTR, Size: ULONG) -> BOOLEAN {
    let mut Chars: ULONG;
    let mut Index: ULONG;
    let mut EntryStart: ULONG;

    if (Size as usize) < core::mem::size_of::<WCHAR>()
        || (Size as usize) % core::mem::size_of::<WCHAR>() != 0
    {
        return FALSE;
    }

    Chars = ((Size as usize) / core::mem::size_of::<WCHAR>()) as ULONG;
    EntryStart = 0;

    Index = 0;
    while Index < Chars {
        if *List.add(Index as usize) != UNICODE_NULL {
            Index = Index.wrapping_add(1);
            continue;
        }

        if Index == EntryStart {
            return (Index == Chars.wrapping_sub(1)) as BOOLEAN;
        }

        if ((Index.wrapping_sub(EntryStart).wrapping_add(1)) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>())
            > (UNICODE_STRING_MAX_BYTES) as usize
        {
            return FALSE;
        }

        EntryStart = Index.wrapping_add(1);
        Index = Index.wrapping_add(1);
    }

    FALSE
}
unsafe extern "C" fn PiPropCheckPayload(
    mut Type: DEVPROPTYPE,
    mut Data: PVOID,
    mut Size: ULONG,
) -> NTSTATUS {
    let mut BaseType: DEVPROPTYPE = Type & DEVPROP_MASK_TYPE as DEVPROPTYPE;
    let mut Rule: *const PIP_PROP_TYPE_RULE = core::ptr::null::<PIP_PROP_TYPE_RULE>();
    let mut Valid: BOOLEAN = 0;

    if BaseType > MAX_DEVPROP_TYPE as DEVPROPTYPE || (Size != 0 && Data.is_null()) {
        return STATUS_INVALID_PARAMETER;
    }

    Rule = (&raw const PiPropTypeRules as *const PIP_PROP_TYPE_RULE).offset(BaseType as isize);

    match Type.wrapping_sub(BaseType) {
        0 => {
            if (*Rule).Width != 0 {
                Valid = (Size == (*Rule).Width as ULONG) as BOOLEAN;
            } else if (*Rule).Check as core::ffi::c_int == PipCheckNoData as core::ffi::c_int {
                Valid = (Size == 0 as ULONG) as BOOLEAN;
            } else if (*Rule).Check as core::ffi::c_int == PipCheckString as core::ffi::c_int {
                Valid = PiPropIsValidString(Data as PCWSTR, Size);
            } else {
                Valid = (Size != 0 as ULONG
                    && RtlValidRelativeSecurityDescriptor(
                        Data as PSECURITY_DESCRIPTOR,
                        Size,
                        0 as SECURITY_INFORMATION,
                    ) != 0
                    && RtlLengthSecurityDescriptor(Data as PSECURITY_DESCRIPTOR) == Size) as BOOLEAN;
            }
        }
        DEVPROP_TYPEMOD_ARRAY => {
            Valid = ((*Rule).Width as ULONG != 0 as ULONG
                && Size != 0 as ULONG
                && Size % ((*Rule).Width as ULONG) == 0) as BOOLEAN;
        }
        DEVPROP_TYPEMOD_LIST => {
            Valid = ((*Rule).ListAllowed != 0
                && PiPropIsValidStringList(Data as PCWSTR, Size) != 0) as BOOLEAN;
        }
        _ => {
            Valid = FALSE as BOOLEAN;
        }
    }

    if Valid != 0 && (*Rule).Width != 0 && (*Rule).Check as core::ffi::c_int != PipCheckNone as core::ffi::c_int {
        Valid = PiPropCheckElements(
            (*Rule).Check,
            Data as PUCHAR,
            (*Rule).Width as ULONG,
            Size / ((*Rule).Width as ULONG),
        );
    }

    if Valid != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_INVALID_PARAMETER
    }
}
unsafe extern "C" fn PiPropInitLocale(Name: PPIP_PROP_NAME, Lcid: LCID) -> NTSTATUS {
    RtlInitEmptyUnicodeString(
        (&raw mut (*Name).Locale).cast(),
        (*Name).LocaleBuffer.as_mut_ptr(),
        core::mem::size_of_val(&(*Name).LocaleBuffer) as USHORT,
    );

    if (Lcid != LOCALE_NEUTRAL) && RtlLCIDToCultureName(Lcid, (&raw mut (*Name).Locale).cast()) == 0 {
        DPRINT1!("Cannot resolve LCID 0x%lx\n", Lcid);
        return STATUS_UNSUCCESSFUL;
    }

    STATUS_SUCCESS
}
unsafe extern "C" fn PiPropInitName(Name: PPIP_PROP_NAME, Key: *const DEVPROPKEY, Lcid: LCID) -> NTSTATUS {
    let Fmtid: *const GUID = core::ptr::addr_of!((*Key).fmtid);
    let mut Status: NTSTATUS;

    (*Name).Key = Key;

    Status = PiPropInitLocale(Name, Lcid);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // Variadic promotion: USHORT / UCHAR arguments are passed as int, Data1 as ULONG.
    Status = RtlStringCchPrintfW(
        core::ptr::addr_of_mut!((*Name).FmtidBuffer) as PWSTR,
        RTL_NUMBER_OF!((*Name).FmtidBuffer),
        &RUSTCONV_WSTR_665AC3C75B1B,
        &[
            (*Fmtid).Data1 as u64,
            (*Fmtid).Data2 as u64,
            (*Fmtid).Data3 as u64,
            (*Fmtid).Data4[0] as u64,
            (*Fmtid).Data4[1] as u64,
            (*Fmtid).Data4[2] as u64,
            (*Fmtid).Data4[3] as u64,
            (*Fmtid).Data4[4] as u64,
            (*Fmtid).Data4[5] as u64,
            (*Fmtid).Data4[6] as u64,
            (*Fmtid).Data4[7] as u64,
        ],
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = RtlStringCchPrintfW(
        core::ptr::addr_of_mut!((*Name).PidBuffer) as PWSTR,
        RTL_NUMBER_OF!((*Name).PidBuffer),
        &RUSTCONV_WSTR_DE4DB6B4EC92,
        &[(*Key).pid as u64],
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitUnicodeString(
        (core::ptr::addr_of_mut!((*Name).Fmtid)).cast(),
        core::ptr::addr_of_mut!((*Name).FmtidBuffer) as *mut WCHAR as PCWSTR,
    );
    RtlInitUnicodeString(
        (core::ptr::addr_of_mut!((*Name).Pid)).cast(),
        core::ptr::addr_of_mut!((*Name).PidBuffer) as *mut WCHAR as PCWSTR,
    );
    return STATUS_SUCCESS;
}
unsafe extern "C" fn PiPropIsInterruptKey(Object: PPIP_PROP_OBJECT, Name: PPIP_PROP_NAME) -> BOOLEAN {
    if (*Object).Pdo != core::ptr::null_mut()
        && (*Name).Locale.Length == 0
        && PiPropIsSameKey((*Name).Key, &raw const PiPropInterruptKey) != 0u8
    {
        1u8
    } else {
        0u8
    }
}
unsafe extern "C" fn PiPropNormalizeStatus(Status: NTSTATUS) -> NTSTATUS {
    if (Status == STATUS_NOT_FOUND) ||
        (Status == STATUS_OBJECT_PATH_NOT_FOUND) ||
        (Status == STATUS_KEY_DELETED)
    {
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    return Status;
}
unsafe extern "C" fn PiPropCopyOut(
    SourceType: DEVPROPTYPE,
    Source: PVOID,
    SourceSize: ULONG,
    Type: PDEVPROPTYPE,
    Data: PVOID,
    Size: ULONG,
    RequiredSize: PULONG,
) -> NTSTATUS {
    *Type = SourceType;
    *RequiredSize = SourceSize;

    if (Size < SourceSize) || (Size == 0 && SourceType != DEVPROP_TYPE_NULL) {
        return STATUS_BUFFER_TOO_SMALL;
    }

    if SourceSize != 0 {
        core::ptr::copy_nonoverlapping(Source as *const u8, Data as *mut u8, SourceSize as usize);
    }

    return STATUS_SUCCESS;
}
unsafe extern "C" fn PiPropParseInterfaceName(
    LinkName: PCUNICODE_STRING,
    ClassGuid: LPGUID,
) -> BOOLEAN {
    let KernelPrefix: UNICODE_STRING = RTL_CONSTANT_STRING(&RUSTCONV_WSTR_C554B73F8939);
    let UserPrefix: UNICODE_STRING = RTL_CONSTANT_STRING(&RUSTCONV_WSTR_04BEA12A5F1F);
    let mut GuidString: UNICODE_STRING = UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Index: USHORT;
    let mut Chars: USHORT;

    if ((*LinkName).Length as usize) % core::mem::size_of::<WCHAR>() != 0 {
        return FALSE;
    }

    if (RtlPrefixUnicodeString(&KernelPrefix, LinkName, FALSE) == 0)
        && (RtlPrefixUnicodeString(&UserPrefix, LinkName, FALSE) == 0)
    {
        return FALSE;
    }

    Chars = ((*LinkName).Length as usize / core::mem::size_of::<WCHAR>()) as USHORT;
    Index = 4;
    while (Index < Chars) && (*(*LinkName).Buffer.add(Index as usize) != 0x005Cu16) {
        Index += 1;
    }

    if Index < 4 + 1 + 38 {
        return FALSE;
    }

    GuidString.Buffer = (*LinkName).Buffer.add(Index as usize - 38 as usize);
    GuidString.MaximumLength =
        (38 as usize * core::mem::size_of::<WCHAR>()) as USHORT;
    GuidString.Length = GuidString.MaximumLength;

    NT_SUCCESS(RtlGUIDFromString(&mut GuidString, ClassGuid)) as BOOLEAN
}
/// NTAPI
unsafe extern "system" fn PiPropGetInterfaceReference(
    LinkName: PCUNICODE_STRING,
    Reference: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Start: USHORT;
    let mut End: USHORT;
    let mut Chars: USHORT;

    Chars = ((*LinkName).Length as usize / core::mem::size_of::<WCHAR>()) as USHORT;
    Start = 4;
    while Start < Chars && *(*LinkName).Buffer.add(Start as usize) != 0x005Cu16 {
        Start += 1;
    }

    if Start >= Chars {
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    Start += 1;
    End = Start;
    while End < Chars {
        if *(*LinkName).Buffer.add(End as usize) == 0x005Cu16
            || *(*LinkName).Buffer.add(End as usize) == 0x002Fu16
        {
            return STATUS_OBJECT_PATH_INVALID;
        }
        End += 1;
    }

    (*Reference).Buffer = (*LinkName).Buffer.add(Start as usize);
    (*Reference).Length = ((End - Start) as usize * core::mem::size_of::<WCHAR>()) as USHORT;
    (*Reference).MaximumLength = (*Reference).Length;
    return STATUS_SUCCESS;
}
unsafe extern "C" fn PiPropOpenObjectKey(
    Object: PPIP_PROP_OBJECT,
    DesiredAccess: ACCESS_MASK,
    ObjectKey: PHANDLE,
) -> NTSTATUS {
    let mut EnumKeyName: UNICODE_STRING = RTL_CONSTANT_STRING(&RUSTCONV_WSTR_2BE316E67F49);
    let mut EnumKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    *ObjectKey = core::ptr::null_mut();

    match (*Object).Kind {
        PipPropObjectInterface => {
            return IopOpenDeviceInterfaceKeys((*Object).Name, DesiredAccess, core::ptr::null_mut(), ObjectKey);
        }
        _ => {}
    }

    Status = IopOpenRegistryKeyEx(&raw mut EnumKey, core::ptr::null_mut(), &raw mut EnumKeyName, KEY_READ);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = IopOpenRegistryKeyEx(
        ObjectKey,
        EnumKey,
        (*Object).Name as PUNICODE_STRING,
        DesiredAccess,
    );
    ZwClose(EnumKey);
    return Status;
}
unsafe extern "C" fn PiPropBuildValuePath(
    Name: PPIP_PROP_NAME,
    Path: PUNICODE_STRING,
    Buffer: PWCHAR,
) -> NTSTATUS {
    RtlInitEmptyUnicodeString(
        Path,
        Buffer as PWSTR,
        ((PIP_PROP_PATH_CHARS as usize) * (core::mem::size_of::<WCHAR>())) as USHORT,
    );
    return RtlUnicodeStringPrintf(
        Path,
        &RUSTCONV_WSTR_8768144DD79E,
        &[
            REGSTR_KEY_DEVICE_PROPERTIES.as_ptr() as usize as u64,
            core::ptr::addr_of_mut!((*Name).Fmtid) as usize as u64,
            core::ptr::addr_of_mut!((*Name).Pid) as usize as u64,
        ],
    );
}
unsafe extern "C" fn PiPropStoreQuery(
    ObjectKey: HANDLE,
    Name: PPIP_PROP_NAME,
    Type: PDEVPROPTYPE,
    Data: PVOID,
    Size: ULONG,
    RequiredSize: PULONG,
) -> NTSTATUS {
    #[repr(C)]
    struct C2RustUnnamed_196 {
        Info: KEY_VALUE_PARTIAL_INFORMATION,
        Inline: [UCHAR; 64],
    }
    let mut Small: C2RustUnnamed_196 = core::mem::zeroed();
    let mut Info: PKEY_VALUE_PARTIAL_INFORMATION = &raw mut Small.Info;
    let mut PathBuffer: [WCHAR; PIP_PROP_PATH_CHARS] = [0; PIP_PROP_PATH_CHARS];
    let mut Path: UNICODE_STRING = UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut PropertyKey: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut InfoSize: ULONG = 0;
    let mut Status: NTSTATUS;

    Status = PiPropBuildValuePath(Name, (&raw mut Path).cast(), &raw mut PathBuffer as PWCHAR);
    if !(Status >= 0 as NTSTATUS) {
        return Status;
    }

    Status = IopOpenRegistryKeyEx(
        &raw mut PropertyKey,
        ObjectKey,
        (&raw mut Path).cast(),
        KEY_QUERY_VALUE as ACCESS_MASK,
    );
    if !(Status >= 0 as NTSTATUS) {
        return Status;
    }

    InfoSize = core::mem::size_of::<C2RustUnnamed_196>() as ULONG;
    loop {
        Status = ZwQueryValueKey(
            PropertyKey,
            (&raw mut (*Name).Locale).cast(),
            KeyValuePartialInformation,
            Info as PVOID,
            InfoSize,
            &raw mut InfoSize,
        );
        if Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL {
            break;
        }

        if Info != &raw mut Small.Info {
            ExFreePoolWithTag(Info as PVOID, TAG_PNP_PROPERTY as ULONG);
        }

        Info = ExAllocatePoolWithTag(
            PagedPool,
            InfoSize as SIZE_T,
            TAG_PNP_PROPERTY as ULONG,
        ) as PKEY_VALUE_PARTIAL_INFORMATION;
        if !Info.is_null() {
            continue;
        }
        Status = STATUS_INSUFFICIENT_RESOURCES;
        break;
    }
    ZwClose(PropertyKey);

    if Status >= 0 as NTSTATUS {
        if ((*Info).Type & PIP_PROP_REG_TYPE_MARK) == PIP_PROP_REG_TYPE_MARK
        {
            Status = PiPropCopyOut(
                (*Info).Type & !PIP_PROP_REG_TYPE_MARK,
                &raw mut (*Info).Data as *mut UCHAR as PVOID,
                (*Info).DataLength,
                Type,
                Data,
                Size,
                RequiredSize,
            );
        } else {
            DPRINT1!(
                "Unexpected value type 0x%lx for %wZ\\%wZ\n",
                (*Info).Type,
                &raw mut (*Name).Fmtid,
                &raw mut (*Name).Pid
            );
            Status = STATUS_OBJECT_NAME_NOT_FOUND;
        }
    }

    if !Info.is_null() && Info != &raw mut Small.Info {
        ExFreePoolWithTag(Info as PVOID, TAG_PNP_PROPERTY as ULONG);
    }

    return Status;
}
unsafe extern "C" fn PiPropPutValue(ValueKey: HANDLE, Name: PPIP_PROP_NAME, Type: DEVPROPTYPE, Data: PVOID, Size: ULONG) -> NTSTATUS {
    ZwSetValueKey(ValueKey, (core::ptr::addr_of_mut!((*Name).Locale)).cast(), 0, PIP_PROP_REG_TYPE_MARK | (Type as ULONG), Data, Size)
}
unsafe extern "C" fn PiPropCreateBranch(Parent: HANDLE, Parts: *mut PCUNICODE_STRING, Depth: ULONG, Name: PPIP_PROP_NAME, Type: DEVPROPTYPE, Data: PVOID, Size: ULONG) -> NTSTATUS {
    let Access: ACCESS_MASK = (KEY_CREATE_SUB_KEY | KEY_SET_VALUE | DELETE) as ACCESS_MASK;
    let mut Disposition: ULONG = 0;
    let mut Child: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = IopCreateRegistryKeyEx(core::ptr::addr_of_mut!(Child), Parent, *Parts.offset(0 as isize) as PUNICODE_STRING, Access,
                                    REG_OPTION_NON_VOLATILE, core::ptr::addr_of_mut!(Disposition));
    if Status == STATUS_CHILD_MUST_BE_VOLATILE {
        Status = IopCreateRegistryKeyEx(core::ptr::addr_of_mut!(Child), Parent, *Parts.offset(0 as isize) as PUNICODE_STRING, Access,
                                        REG_OPTION_VOLATILE, core::ptr::addr_of_mut!(Disposition));
    }
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if Depth > 1 {
        Status = PiPropCreateBranch(Child, Parts.offset(1 as isize), Depth - 1, Name, Type, Data, Size);
    } else {
        Status = PiPropPutValue(Child, Name, Type, Data, Size);
    }

    if !NT_SUCCESS(Status) && (Disposition == REG_CREATED_NEW_KEY) {
        ZwDeleteKey(Child);
    }

    ZwClose(Child);
    return Status;
}
unsafe extern "C" fn PiPropStoreWrite(
    ObjectKey: HANDLE,
    Name: PPIP_PROP_NAME,
    Type: DEVPROPTYPE,
    Data: PVOID,
    Size: ULONG,
) -> NTSTATUS {
    let mut PropertiesName: UNICODE_STRING = RTL_CONSTANT_STRING(REGSTR_KEY_DEVICE_PROPERTIES);
    let mut Parts: [PCUNICODE_STRING; 3] = [
        &PropertiesName as *const UNICODE_STRING,
        &(*Name).Fmtid as *const UNICODE_STRING,
        &(*Name).Pid as *const UNICODE_STRING,
    ];
    let mut PathBuffer: [WCHAR; PIP_PROP_PATH_CHARS] = [0; PIP_PROP_PATH_CHARS];
    let mut Path: UNICODE_STRING = core::mem::zeroed();
    let mut ValueKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = PiPropBuildValuePath(Name, &mut Path, PathBuffer.as_mut_ptr());
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = IopOpenRegistryKeyEx(&mut ValueKey, ObjectKey, &mut Path, KEY_SET_VALUE);
    if !NT_SUCCESS(Status) {
        return PiPropCreateBranch(
            ObjectKey,
            Parts.as_mut_ptr(),
            RTL_NUMBER_OF!(Parts) as ULONG,
            Name,
            Type,
            Data,
            Size,
        );
    }

    Status = PiPropPutValue(ValueKey, Name, Type, Data, Size);
    ZwClose(ValueKey);
    return Status;
}
unsafe extern "C" fn PiPropIsKeyEmpty(KeyHandle: HANDLE) -> BOOLEAN {
    #[repr(C)]
    struct Buffer_t {
        Info: KEY_FULL_INFORMATION,
        ClassName: [u16; 32],
    }
    let mut Buffer: Buffer_t = core::mem::zeroed();
    let mut ResultLength: ULONG = 0;
    let mut Status: NTSTATUS;

    Status = ZwQueryKey(
        KeyHandle,
        KeyFullInformation,
        &mut Buffer as *mut Buffer_t as PVOID,
        core::mem::size_of::<Buffer_t>() as ULONG,
        &mut ResultLength,
    );

    (NT_SUCCESS(Status)
        && (Buffer.Info.SubKeys == 0)
        && (Buffer.Info.Values == 0)) as BOOLEAN
}
unsafe extern "C" fn PiPropStoreDelete(ObjectKey: HANDLE, Name: PPIP_PROP_NAME) -> NTSTATUS {
    let mut PropertiesName: UNICODE_STRING = RTL_CONSTANT_STRING(REGSTR_KEY_DEVICE_PROPERTIES);
    let mut Chain: [PUNICODE_STRING; 3] = [
        core::ptr::addr_of_mut!(PropertiesName),
        core::ptr::addr_of_mut!((*Name).Fmtid),
        core::ptr::addr_of_mut!((*Name).Pid),
    ];
    // HANDLE Keys[RTL_NUMBER_OF(Chain) + 1];
    let mut Keys: [HANDLE; 3 + 1] = [core::ptr::null_mut(); 3 + 1];
    let mut Depth: ULONG = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Keys[0 as usize] = ObjectKey;
    while Depth < RTL_NUMBER_OF!(Chain) as ULONG {
        Status = IopOpenRegistryKeyEx(
            core::ptr::addr_of_mut!(Keys[(Depth + 1) as usize]) as PHANDLE,
            Keys[Depth as usize],
            Chain[Depth as usize],
            (KEY_QUERY_VALUE | KEY_SET_VALUE | DELETE) as ACCESS_MASK,
        );
        if !NT_SUCCESS(Status) {
            break;
        }
        Depth += 1;
    }

    if NT_SUCCESS(Status) {
        Status = ZwDeleteValueKey(Keys[Depth as usize], (core::ptr::addr_of_mut!((*Name).Locale)).cast());
    }

    while Depth > 0 {
        if NT_SUCCESS(Status) && PiPropIsKeyEmpty(Keys[Depth as usize]) != 0 {
            ZwDeleteKey(Keys[Depth as usize]);
        }

        ZwClose(Keys[Depth as usize]);
        Depth -= 1;
    }

    Status
}
unsafe extern "C" fn PiPropFindSaved(Pdo: PDEVICE_OBJECT) -> PPIP_PROP_SAVED_VALUE {
    let mut Saved: PPIP_PROP_SAVED_VALUE;
    let mut Entry: PLIST_ENTRY;

    Entry = (*(&raw mut PiPropSavedValues)).Flink;
    while Entry != (&raw mut PiPropSavedValues) {
        Saved = (Entry as *mut u8).sub(core::mem::offset_of!(PIP_PROP_SAVED_VALUE, Link))
            as PPIP_PROP_SAVED_VALUE;
        if (*Saved).Pdo == Pdo {
            return Saved;
        }
        Entry = (*Entry).Flink;
    }

    core::ptr::null_mut()
}
unsafe extern "C" fn PiPropReplaceSaved(
    Pdo: PDEVICE_OBJECT,
    Type: DEVPROPTYPE,
    Data: PVOID,
    Size: ULONG,
    Replaced: PBOOLEAN,
) -> NTSTATUS {
    let mut Fresh: PPIP_PROP_SAVED_VALUE = core::ptr::null_mut();
    let mut Stale: PPIP_PROP_SAVED_VALUE;

    if Type != DEVPROP_TYPE_EMPTY {
        Fresh = ExAllocatePoolWithTag(
            PagedPool,
            (core::mem::offset_of!(PIP_PROP_SAVED_VALUE, Data) as SIZE_T) + (Size as SIZE_T),
            TAG_PNP_PROPERTY,
        ) as PPIP_PROP_SAVED_VALUE;
        if Fresh.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*Fresh).Pdo = Pdo;
        (*Fresh).Type = Type;
        (*Fresh).Size = Size;
        if Size != 0 {
            core::ptr::copy_nonoverlapping(
                Data as *const u8,
                core::ptr::addr_of_mut!((*Fresh).Data) as *mut u8,
                Size as usize,
            );
        }
    }

    KeEnterCriticalRegion();
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!(PiPropSavedLock));

    Stale = PiPropFindSaved(Pdo);
    if !Stale.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*Stale).Link));
    }
    if !Fresh.is_null() {
        InsertTailList(
            core::ptr::addr_of_mut!(PiPropSavedValues),
            core::ptr::addr_of_mut!((*Fresh).Link),
        );
    }

    ExReleasePushLockExclusive(core::ptr::addr_of_mut!(PiPropSavedLock));
    KeLeaveCriticalRegion();

    if !Replaced.is_null() {
        *Replaced = (!Stale.is_null()) as u8;
    }

    if !Stale.is_null() {
        ExFreePoolWithTag(Stale as PVOID, TAG_PNP_PROPERTY);
    }

    return STATUS_SUCCESS;
}
unsafe extern "C" fn PiPropBuiltinTable(
    Kind: PIP_PROP_OBJECT_KIND,
    Count: PULONG,
) -> *const PIP_BUILTIN_PROPERTY {
    match Kind {
        PipPropObjectDevice => {
            *Count = RTL_NUMBER_OF!(PipDeviceBuiltins) as ULONG;
            return PipDeviceBuiltins.as_ptr();
        }
        PipPropObjectInterface => {
            *Count = RTL_NUMBER_OF!(PipInterfaceBuiltins) as ULONG;
            return PipInterfaceBuiltins.as_ptr();
        }
        _ => {
            *Count = 0;
            return core::ptr::null();
        }
    }
}
unsafe extern "C" fn PiPropFindBuiltin(
    Object: PPIP_PROP_OBJECT,
    Key: *const DEVPROPKEY,
) -> *const PIP_BUILTIN_PROPERTY {
    let mut Table: *const PIP_BUILTIN_PROPERTY;
    let mut Count: ULONG = core::mem::zeroed();

    Table = PiPropBuiltinTable((*Object).Kind, &mut Count);
    while Count != 0 {
        if PiPropIsSameKey((*Table).Key, Key) != 0 {
            return Table;
        }
        Count = Count.wrapping_sub(1);
        Table = Table.add(1);
    }

    core::ptr::null()
}
unsafe extern "C" fn PiPropAllocateBlob(Source: PVOID, Size: ULONG, Blob: *mut PVOID) -> NTSTATUS {
    *Blob = core::ptr::null_mut();
    if Size == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    *Blob = ExAllocatePoolWithTag(PagedPool, Size as SIZE_T, TAG_PNP_PROPERTY as ULONG);
    if (*Blob).is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(Source as *const u8, *Blob as *mut u8, Size as usize);
    return STATUS_SUCCESS;
}
unsafe extern "C" fn PiPropCaptureString(
    Raw: PCWSTR,
    RawSize: ULONG,
    IsList: BOOLEAN,
    Blob: *mut PVOID,
    BlobSize: PULONG,
) -> NTSTATUS {
    let RawChars: ULONG = RawSize / (core::mem::size_of::<WCHAR>() as ULONG);
    let mut Index: ULONG = 0;
    let mut Out: ULONG = 0 as ULONG;
    let mut InEntry: BOOLEAN = FALSE as BOOLEAN;
    let String: PWCHAR = ExAllocatePoolWithTag(
        PagedPool,
        ((RawChars + 2 as ULONG) as SIZE_T)
            * (core::mem::size_of::<WCHAR>() as SIZE_T),
        TAG_PNP_PROPERTY,
    ) as PWCHAR;
    if String.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    *Blob = String as PVOID;

    Index = 0 as ULONG;
    while Index < RawChars {
        if *Raw.offset(Index as isize) != (UNICODE_NULL) as WCHAR {
            *String.offset(Out as isize) = *Raw.offset(Index as isize);
            Out += 1;
            InEntry = TRUE as BOOLEAN;
        } else {
            if IsList == 0 || InEntry == 0 {
                break;
            }
            *String.offset(Out as isize) = (UNICODE_NULL) as WCHAR;
            Out += 1;
            InEntry = FALSE as BOOLEAN;
        }
        Index += 1;
    }

    if IsList == 0 || InEntry != 0 {
        *String.offset(Out as isize) = (UNICODE_NULL) as WCHAR;
        Out += 1;
    }
    if IsList != 0 {
        *String.offset(Out as isize) = (UNICODE_NULL) as WCHAR;
        Out += 1;
    }

    *BlobSize = Out * (core::mem::size_of::<WCHAR>() as ULONG);
    return STATUS_SUCCESS;
}
unsafe extern "C" fn PiPropReadRegistryValue(
    KeyHandle: HANDLE,
    ValueName: PCWSTR,
    RegType: ULONG,
    Type: DEVPROPTYPE,
    Blob: *mut PVOID,
    BlobSize: PULONG,
) -> NTSTATUS {
    let mut Info: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut RawData: PVOID = core::ptr::null_mut();
    let mut RawSize: ULONG = 0;
    let mut Status: NTSTATUS;

    Status = IopGetRegistryValue(KeyHandle, ValueName as PWSTR, &raw mut Info);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RawData = (Info as PUCHAR).add((*Info).DataOffset as usize) as PVOID;
    RawSize = (*Info).DataLength;

    if ((*Info).Type != RegType)
        && !((RegType == REG_SZ) && ((*Info).Type == REG_EXPAND_SZ))
    {
        DPRINT1!("Value '%S' has type %lu, expected %lu\n", ValueName, (*Info).Type, RegType);
        ExFreePool(Info as PVOID);
        return STATUS_OBJECT_TYPE_MISMATCH;
    }

    match Type {
        DEVPROP_TYPE_STRING | DEVPROP_TYPE_STRING_LIST => {
            Status = PiPropCaptureString(
                RawData as PCWSTR,
                RawSize,
                (Type == DEVPROP_TYPE_STRING_LIST) as BOOLEAN,
                Blob,
                BlobSize,
            );
        }

        DEVPROP_TYPE_UINT32 => {
            if RawSize as usize != core::mem::size_of::<ULONG>() {
                Status = STATUS_OBJECT_TYPE_MISMATCH;
            } else {
                Status = PiPropAllocateBlob(RawData, RawSize, Blob);
                *BlobSize = RawSize;
            }
        }

        DEVPROP_TYPE_GUID => {
            let mut GuidString: UNICODE_STRING = UNICODE_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut(),
            };
            let mut Guid: GUID = GUID {
                Data1: 0,
                Data2: 0,
                Data3: 0,
                Data4: [0; 8],
            };

            Status = PiPropCaptureString(
                RawData as PCWSTR,
                RawSize,
                FALSE,
                Blob,
                BlobSize,
            );
            if !NT_SUCCESS(Status) {
                // break out of switch
            } else {
                RtlInitUnicodeString((&raw mut GuidString).cast(), *Blob as PCWSTR);
                Status = RtlGUIDFromString((&raw mut GuidString).cast(), &raw mut Guid);
                if NT_SUCCESS(Status) {
                    core::ptr::copy_nonoverlapping(
                        &raw const Guid as *const u8,
                        *Blob as *mut u8,
                        core::mem::size_of::<GUID>(),
                    );
                    *BlobSize = core::mem::size_of::<GUID>() as ULONG;
                } else {
                    ExFreePoolWithTag(*Blob, TAG_PNP_PROPERTY);
                    Status = STATUS_OBJECT_TYPE_MISMATCH;
                }
            }
        }

        _ => {
            ASSERT!(false);
            Status = STATUS_INTERNAL_ERROR;
        }
    }

    ExFreePool(Info as PVOID);
    return Status;
}
unsafe extern "C" fn PiPropReadKeyValue(
    Object: PPIP_PROP_OBJECT,
    ValueName: PCWSTR,
    RegType: ULONG,
    Type: DEVPROPTYPE,
    Blob: *mut PVOID,
    BlobSize: PULONG,
) -> NTSTATUS {
    let mut ObjectKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = PiPropOpenObjectKey(Object, KEY_QUERY_VALUE as ACCESS_MASK, core::ptr::addr_of_mut!(ObjectKey));
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = PiPropReadRegistryValue(ObjectKey, ValueName, RegType, Type, Blob, BlobSize);
    ZwClose(ObjectKey);
    return Status;
}
unsafe extern "C" fn PiPropReadDeviceRegistryProperty(
    Object: PPIP_PROP_OBJECT,
    Builtin: *const PIP_BUILTIN_PROPERTY,
    Blob: *mut PVOID,
    BlobSize: PULONG,
) -> NTSTATUS {
    let mut Length: ULONG = 128;
    let mut Needed: ULONG = 0;
    let mut Buffer: PVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    if (*Object).Pdo.is_null() {
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    loop {
        Buffer = ExAllocatePoolWithTag(PagedPool, Length as SIZE_T, TAG_PNP_PROPERTY as ULONG);
        if Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = IoGetDeviceProperty(
            (*Object).Pdo,
            (*Builtin).Selector as DEVICE_REGISTRY_PROPERTY,
            Length,
            Buffer,
            &raw mut Needed,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break;
        }

        ExFreePoolWithTag(Buffer, TAG_PNP_PROPERTY as ULONG);

        // ALIGN_UP_BY(max(Needed, Length * 2), sizeof(WCHAR))
        Length = ALIGN_UP_BY(
            core::cmp::max(Needed, Length.wrapping_mul(2)) as usize,
            ::core::mem::size_of::<WCHAR>(),
        ) as ULONG;
        if Length > PIP_PROP_MAX_QUERY_SIZE {
            return STATUS_UNSUCCESSFUL;
        }
    }

    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(Buffer, TAG_PNP_PROPERTY as ULONG);
        return Status;
    }

    if (*Builtin).Type == DEVPROP_TYPE_STRING {
        Status = PiPropCaptureString(Buffer as PCWSTR, Needed, FALSE as BOOLEAN, Blob, BlobSize);
        ExFreePoolWithTag(Buffer, TAG_PNP_PROPERTY as ULONG);
        return Status;
    }

    if Needed != PiPropTypeRules[(*Builtin).Type as usize].Width as ULONG {
        ExFreePoolWithTag(Buffer, TAG_PNP_PROPERTY as ULONG);
        return STATUS_OBJECT_TYPE_MISMATCH;
    }

    *Blob = Buffer;
    *BlobSize = Needed;
    return STATUS_SUCCESS;
}
unsafe extern "C" fn PiPropReadDeviceNodeField(
    Object: PPIP_PROP_OBJECT,
    Field: PIP_PROP_FIELD,
    Blob: *mut PVOID,
    BlobSize: PULONG,
) -> NTSTATUS {
    let DeviceNode: PDEVICE_NODE = (*Object).DeviceNode;
    let mut Boolean: DEVPROP_BOOLEAN = 0;
    let mut Value: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    match Field {
        PipFieldName => {
            Status = PiPropReadKeyValue(
                Object,
                (REGSTR_VAL_FRIENDLYNAME).as_ptr(),
                REG_SZ,
                DEVPROP_TYPE_STRING,
                Blob,
                BlobSize,
            );
            if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                Status = PiPropReadKeyValue(
                    Object,
                    (REGSTR_VAL_DEVDESC).as_ptr(),
                    REG_SZ,
                    DEVPROP_TYPE_STRING,
                    Blob,
                    BlobSize,
                );
            }
            return Status;
        }
        PipFieldInstanceId => {
            return PiPropCaptureString(
                (*(*Object).Name).Buffer as PCWSTR,
                (*(*Object).Name).Length as ULONG,
                FALSE,
                Blob,
                BlobSize,
            );
        }
        PipFieldIsPresent => {
            Boolean = (if !DeviceNode.is_null() && ((*DeviceNode).Flags & DNF_DEVICE_GONE) == 0 {
                DEVPROP_TRUE
            } else {
                DEVPROP_FALSE
            }) as DEVPROP_BOOLEAN;
            *BlobSize = ::core::mem::size_of::<DEVPROP_BOOLEAN>() as ULONG;
            return PiPropAllocateBlob(
                &raw mut Boolean as PVOID,
                ::core::mem::size_of::<DEVPROP_BOOLEAN>() as ULONG,
                Blob,
            );
        }
        _ => {}
    }

    if DeviceNode.is_null() {
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    match Field {
        PipFieldParent => {
            if (*DeviceNode).Parent.is_null() {
                return STATUS_OBJECT_NAME_NOT_FOUND;
            }
            return PiPropCaptureString(
                (*(*DeviceNode).Parent).InstancePath.Buffer as PCWSTR,
                (*(*DeviceNode).Parent).InstancePath.Length as ULONG,
                FALSE,
                Blob,
                BlobSize,
            );
        }
        PipFieldStatus => {
            Value = IopGetDeviceNodeStatus(DeviceNode);
        }
        PipFieldProblemCode => {
            Value = (*DeviceNode).Problem;
        }
        PipFieldHasProblem => {
            Boolean = (if ((*DeviceNode).Flags & DNF_HAS_PROBLEM) != 0 {
                DEVPROP_TRUE
            } else {
                DEVPROP_FALSE
            }) as DEVPROP_BOOLEAN;
            *BlobSize = ::core::mem::size_of::<DEVPROP_BOOLEAN>() as ULONG;
            return PiPropAllocateBlob(
                &raw mut Boolean as PVOID,
                ::core::mem::size_of::<DEVPROP_BOOLEAN>() as ULONG,
                Blob,
            );
        }
        _ => {
            ASSERT!(false);
            return STATUS_INTERNAL_ERROR;
        }
    }

    *BlobSize = ::core::mem::size_of::<ULONG>() as ULONG;
    return PiPropAllocateBlob(
        &raw mut Value as PVOID,
        ::core::mem::size_of::<ULONG>() as ULONG,
        Blob,
    );
}
unsafe extern "C" fn PiPropReadInterfaceField(
    Object: PPIP_PROP_OBJECT,
    Field: PIP_PROP_FIELD,
    Blob: *mut PVOID,
    BlobSize: PULONG,
) -> NTSTATUS {
    let mut ControlName: UNICODE_STRING = RTL_CONSTANT_STRING(&RUSTCONV_WSTR_05ECCD5C2679);
    let mut DeviceKey: HANDLE = core::ptr::null_mut();
    let mut InstanceKey: HANDLE = core::ptr::null_mut();
    let mut ControlKey: HANDLE = core::ptr::null_mut();
    let mut Info: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut Enabled: DEVPROP_BOOLEAN = 0;
    let mut Reference: UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = IopOpenDeviceInterfaceKeys(
        (*Object).Name,
        KEY_READ as ACCESS_MASK,
        &mut DeviceKey,
        &mut InstanceKey,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    match Field {
        PipFieldInterfaceClass => {
            *BlobSize = core::mem::size_of::<GUID>() as ULONG;
            Status = PiPropAllocateBlob(
                core::ptr::addr_of_mut!((*Object).ClassGuid) as PVOID,
                core::mem::size_of::<GUID>() as ULONG,
                Blob,
            );
        }

        PipFieldInterfaceEnabled => {
            Enabled = DEVPROP_FALSE;
            if NT_SUCCESS(IopOpenRegistryKeyEx(
                &mut ControlKey,
                InstanceKey,
                &mut ControlName,
                KEY_QUERY_VALUE as ACCESS_MASK,
            )) {
                if NT_SUCCESS(IopGetRegistryValue(
                    ControlKey,
                    RUSTCONV_WSTR_36EBA83186E3.as_ptr().cast_mut(),
                    &mut Info,
                )) {
                    if ((*Info).Type == REG_DWORD as ULONG)
                        && ((*Info).DataLength as usize == core::mem::size_of::<ULONG>() as usize)
                        && (*((Info as PUCHAR).add((*Info).DataOffset as usize) as PULONG)
                            != 0 as ULONG)
                    {
                        Enabled = DEVPROP_TRUE;
                    }
                    ExFreePool(Info as PVOID);
                }
                ZwClose(ControlKey);
            }

            *BlobSize = core::mem::size_of::<DEVPROP_BOOLEAN>() as ULONG;
            Status = PiPropAllocateBlob(
                core::ptr::addr_of_mut!(Enabled) as PVOID,
                core::mem::size_of::<DEVPROP_BOOLEAN>() as ULONG,
                Blob,
            );
        }

        PipFieldInterfaceDevice => {
            Status = PiPropReadRegistryValue(
                DeviceKey,
                RUSTCONV_WSTR_E9A256AB2F78.as_ptr(),
                REG_SZ as ULONG,
                DEVPROP_TYPE_STRING as DEVPROPTYPE,
                Blob,
                BlobSize,
            );
        }

        PipFieldInterfaceReference => {
            Status = PiPropGetInterfaceReference((*Object).Name, &mut Reference);
            if NT_SUCCESS(Status) {
                Status = PiPropCaptureString(
                    Reference.Buffer,
                    Reference.Length as ULONG,
                    FALSE as BOOLEAN,
                    Blob,
                    BlobSize,
                );
            }
        }

        _ => {
            ASSERT!(false);
            Status = STATUS_INTERNAL_ERROR;
        }
    }

    ZwClose(InstanceKey);
    ZwClose(DeviceKey);
    return Status;
}
unsafe extern "C" fn PiPropReadBuiltin(
    Object: PPIP_PROP_OBJECT,
    Builtin: *const PIP_BUILTIN_PROPERTY,
    Blob: *mut PVOID,
    BlobSize: PULONG,
) -> NTSTATUS {
    match (*Builtin).Source {
        PipSourceKeyValue => {
            return PiPropReadKeyValue(
                Object,
                (*Builtin).ValueName,
                (*Builtin).Selector,
                (*Builtin).Type,
                Blob,
                BlobSize,
            );
        }
        PipSourceRegistryProperty => return PiPropReadDeviceRegistryProperty(Object, Builtin, Blob, BlobSize),
        PipSourceDeviceNode => {
            return PiPropReadDeviceNodeField(
                Object,
                (*Builtin).Selector as PIP_PROP_FIELD,
                Blob,
                BlobSize,
            );
        }
        PipSourceInterface => {
            return PiPropReadInterfaceField(
                Object,
                (*Builtin).Selector as PIP_PROP_FIELD,
                Blob,
                BlobSize,
            );
        }
        _ => {
            ASSERT!(false);
            return STATUS_INTERNAL_ERROR;
        }
    }
}
unsafe extern "C" fn PiPropWriteBuiltin(
    Object: PPIP_PROP_OBJECT,
    Builtin: *const PIP_BUILTIN_PROPERTY,
    Type: DEVPROPTYPE,
    Data: PVOID,
    Size: ULONG,
) -> NTSTATUS {
    let RootEnumerator_local: UNICODE_STRING = RTL_CONSTANT_STRING(&RUSTCONV_WSTR_CD183236F6BC);
    let mut ValueName: UNICODE_STRING = core::mem::zeroed();
    let mut ObjectKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if ((*Builtin).Flags & PIP_PROP_READ_ONLY) != 0
        || (*Builtin).Source != PipSourceKeyValue
    {
        return STATUS_ACCESS_DENIED;
    }

    if ((*Builtin).Flags & PIP_PROP_ROOT_ENUM_WRITE) != 0
        && RtlPrefixUnicodeString(&RootEnumerator_local, (*Object).Name, TRUE) == 0
    {
        return STATUS_ACCESS_DENIED;
    }

    if Type != DEVPROP_TYPE_EMPTY && Type != (*Builtin).Type {
        return STATUS_INVALID_PARAMETER;
    }

    Status = PiPropOpenObjectKey(Object, KEY_SET_VALUE, &mut ObjectKey);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitUnicodeString(&mut ValueName, (*Builtin).ValueName);

    if Type == DEVPROP_TYPE_EMPTY {
        Status = ZwDeleteValueKey(ObjectKey, &mut ValueName);
    } else if Type == DEVPROP_TYPE_GUID {
        let mut GuidString: UNICODE_STRING = core::mem::zeroed();
        let mut Guid: GUID = core::mem::zeroed();

        core::ptr::copy_nonoverlapping(
            Data as *const u8,
            &mut Guid as *mut GUID as *mut u8,
            core::mem::size_of::<GUID>(),
        );
        Status = RtlStringFromGUID(&mut Guid, &mut GuidString);
        if NT_SUCCESS(Status) {
            Status = ZwSetValueKey(
                ObjectKey,
                &mut ValueName,
                0,
                REG_SZ,
                GuidString.Buffer as PVOID,
                (GuidString.Length as ULONG).wrapping_add(core::mem::size_of_val(&UNICODE_NULL) as ULONG),
            );
            RtlFreeUnicodeString(&mut GuidString);
        }
    } else {
        Status = ZwSetValueKey(
            ObjectKey,
            &mut ValueName,
            0,
            (*Builtin).Selector,
            Data,
            Size,
        );
    }

    ZwClose(ObjectKey);
    return Status;
}
unsafe extern "C" fn PiPropReadFromBuiltin(
    Object: PPIP_PROP_OBJECT,
    Name: PPIP_PROP_NAME,
    Out: PPIP_PROP_OUTPUT,
    Status: PNTSTATUS,
) -> BOOLEAN {
    let mut Builtin: *const PIP_BUILTIN_PROPERTY = core::ptr::null();
    let mut BlobSize: ULONG = 0;
    let mut Blob: PVOID = core::ptr::null_mut();

    if (*Name).Locale.Length != 0 {
        return FALSE;
    }

    Builtin = PiPropFindBuiltin(Object, (*Name).Key);
    if Builtin.is_null() {
        return FALSE;
    }

    *Status = PiPropReadBuiltin(
        Object,
        Builtin,
        core::ptr::addr_of_mut!(Blob),
        core::ptr::addr_of_mut!(BlobSize),
    );
    if NT_SUCCESS(*Status) {
        *Status = PiPropCopyOut(
            (*Builtin).Type,
            Blob,
            BlobSize,
            core::ptr::addr_of_mut!((*Out).Type),
            (*Out).Data,
            (*Out).Size,
            core::ptr::addr_of_mut!((*Out).Required),
        );
        ExFreePoolWithTag(Blob, TAG_PNP_PROPERTY);
    }

    return TRUE;
}
unsafe extern "C" fn PiPropReadFromStore(
    Object: PPIP_PROP_OBJECT,
    Name: PPIP_PROP_NAME,
    Out: PPIP_PROP_OUTPUT,
    Status: PNTSTATUS,
) -> BOOLEAN {
    let mut ObjectKey: HANDLE = core::ptr::null_mut();

    *Status = PiPropOpenObjectKey(Object, KEY_READ as ACCESS_MASK, &raw mut ObjectKey);
    if NT_SUCCESS(*Status) {
        *Status = PiPropStoreQuery(
            ObjectKey,
            Name,
            &raw mut (*Out).Type,
            (*Out).Data,
            (*Out).Size,
            &raw mut (*Out).Required,
        );
        ZwClose(ObjectKey);
    }

    (PiPropNormalizeStatus(*Status) != STATUS_OBJECT_NAME_NOT_FOUND) as BOOLEAN
}
unsafe extern "C" fn PiPropReadFromSaved(
    Object: PPIP_PROP_OBJECT,
    Name: PPIP_PROP_NAME,
    Out: PPIP_PROP_OUTPUT,
    Status: PNTSTATUS,
) -> BOOLEAN {
    let mut Saved: PPIP_PROP_SAVED_VALUE;

    if PiPropIsInterruptKey(Object, Name) == 0 {
        return FALSE;
    }

    KeEnterCriticalRegion();
    ExAcquirePushLockShared(core::ptr::addr_of_mut!(PiPropSavedLock));

    Saved = PiPropFindSaved((*Object).Pdo);
    if !Saved.is_null() {
        *Status = PiPropCopyOut(
            (*Saved).Type,
            core::ptr::addr_of_mut!((*Saved).Data) as PVOID,
            (*Saved).Size,
            core::ptr::addr_of_mut!((*Out).Type),
            (*Out).Data,
            (*Out).Size,
            core::ptr::addr_of_mut!((*Out).Required),
        );
    }

    ExReleasePushLockShared(core::ptr::addr_of_mut!(PiPropSavedLock));
    KeLeaveCriticalRegion();

    (!Saved.is_null()) as BOOLEAN
}
unsafe extern "C" fn PiPropGet(
    Object: PPIP_PROP_OBJECT,
    PropertyKey: *const DEVPROPKEY,
    Lcid: LCID,
    Size: ULONG,
    Data: PVOID,
    RequiredSize: PULONG,
    Type: PDEVPROPTYPE,
) -> NTSTATUS {
    let mut Out: PIP_PROP_OUTPUT = PIP_PROP_OUTPUT {
        Data: Data,
        Size: if !Data.is_null() { Size } else { 0 },
        Required: 0,
        Type: DEVPROP_TYPE_EMPTY as DEVPROPTYPE,
    };
    let mut Name: PIP_PROP_NAME = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Index: ULONG;

    PAGED_CODE!();

    if PropertyKey.is_null() || RequiredSize.is_null() || Type.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = PiPropInitName(&raw mut Name, PropertyKey, Lcid);
    if NT_SUCCESS(Status) {
        Status = STATUS_OBJECT_NAME_NOT_FOUND;
        Index = 0;
        while (Index as usize) < RTL_NUMBER_OF!(PiPropReaders) {
            if PiPropReaders[Index as usize].unwrap_unchecked()(
                Object,
                &raw mut Name,
                &raw mut Out,
                &raw mut Status,
            ) != 0
            {
                break;
            }
            Index = Index.wrapping_add(1);
        }
    }

    *RequiredSize = Out.Required;
    *Type = Out.Type;
    return PiPropNormalizeStatus(Status);
}
unsafe extern "C" fn PiPropPickTarget(
    Object: PPIP_PROP_OBJECT,
    Name: PPIP_PROP_NAME,
    Builtin: *mut *const PIP_BUILTIN_PROPERTY,
) -> PIP_PROP_TARGET {
    *Builtin = core::ptr::null();

    if ((*Object).Flags & PIP_PROP_OBJECT_PROTECTED) != 0 && (*Name).Locale.Length == 0 {
        return PipTargetDenied;
    }

    if (*Name).Locale.Length == 0 {
        *Builtin = PiPropFindBuiltin(Object, (*Name).Key);
    }

    if !(*Builtin).is_null() {
        PipTargetBuiltin
    } else {
        PipTargetStore
    }
}
unsafe extern "C" fn PiPropStoreApply(
    Object: PPIP_PROP_OBJECT,
    Name: PPIP_PROP_NAME,
    Type: DEVPROPTYPE,
    Data: PVOID,
    Size: ULONG,
) -> NTSTATUS {
    let mut Replaced: BOOLEAN = 0u8;
    let mut ObjectKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = PiPropOpenObjectKey(Object, KEY_READ | KEY_CREATE_SUB_KEY, &mut ObjectKey);
    if NT_SUCCESS(Status) {
        if Type == DEVPROP_TYPE_EMPTY {
            Status = PiPropStoreDelete(ObjectKey, Name);
        } else {
            Status = PiPropStoreWrite(ObjectKey, Name, Type, Data, Size);
        }

        ZwClose(ObjectKey);
    }

    if PiPropIsInterruptKey(Object, Name) == 0u8 {
        return Status;
    }

    if NT_SUCCESS(Status) {
        return PiPropReplaceSaved((*Object).Pdo, Type, Data, Size, core::ptr::null_mut());
    }

    if (Type == DEVPROP_TYPE_EMPTY)
        && (PiPropNormalizeStatus(Status) == STATUS_OBJECT_NAME_NOT_FOUND)
    {
        PiPropReplaceSaved((*Object).Pdo, Type, core::ptr::null_mut(), 0, &mut Replaced);
        if Replaced != 0u8 {
            Status = STATUS_SUCCESS;
        }
    }

    return Status;
}
unsafe extern "C" fn PiPropShouldAnnounce(DeviceNode: PDEVICE_NODE) -> BOOLEAN {
    match (*DeviceNode).State {
        DeviceNodeUnspecified
        | DeviceNodeUninitialized
        | DeviceNodeInitialized => return FALSE,
        _ => return TRUE,
    }
}
unsafe extern "C" fn PiPropSet(
    Object: PPIP_PROP_OBJECT,
    PropertyKey: *const DEVPROPKEY,
    Lcid: LCID,
    Type: DEVPROPTYPE,
    Size: ULONG,
    Data: PVOID,
) -> NTSTATUS {
    let mut Builtin: *const PIP_BUILTIN_PROPERTY = core::ptr::null();
    let mut Target: PIP_PROP_TARGET = PipTargetDenied;
    let mut Name: PIP_PROP_NAME = core::mem::zeroed();
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    if PropertyKey.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Status = PiPropCheckPayload(Type, Data, Size);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = PiPropInitName(&mut Name, PropertyKey, Lcid);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (Type == DEVPROP_TYPE_STRING_INDIRECT) && Name.Locale.Length != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Target = PiPropPickTarget(Object, &mut Name, &mut Builtin);
    if Target == PipTargetDenied {
        return STATUS_ACCESS_DENIED;
    }

    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(&raw mut PpRegistryDeviceResource, TRUE);

    if Target == PipTargetBuiltin {
        Status = PiPropWriteBuiltin(Object, Builtin, Type, Data, Size);
    } else {
        Status = PiPropStoreApply(Object, &mut Name, Type, Data, Size);
    }

    ExReleaseResourceLite(&raw mut PpRegistryDeviceResource);
    KeLeaveCriticalRegion();

    if !(*Object).DeviceNode.is_null() && PiPropShouldAnnounce((*Object).DeviceNode) != 0 {
        IopQueueDevicePropertyChangeEvent(&raw mut (*(*Object).DeviceNode).InstancePath as PCUNICODE_STRING);
    }

    return PiPropNormalizeStatus(Status);
}
unsafe extern "C" fn PiPropInitDeviceObject(Pdo: PDEVICE_OBJECT, Object: PPIP_PROP_OBJECT) -> NTSTATUS {
    let DeviceNode: PDEVICE_NODE = if !Pdo.is_null() {
        IopGetDeviceNode(Pdo)
    } else {
        core::ptr::null_mut()
    };

    if DeviceNode.is_null() || ((*DeviceNode).Flags & DNF_LEGACY_RESOURCE_DEVICENODE) != 0 {
        DPRINT1!("%p is not a PDO\n", Pdo);
        KeBugCheckEx(
            PNP_DETECTED_FATAL_ERROR,
            0x2,
            Pdo as ULONG_PTR,
            0,
            0,
        );
    }

    if (*DeviceNode).InstancePath.Length == 0 {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    core::ptr::write_bytes(Object as *mut u8, 0, core::mem::size_of::<PIP_PROP_OBJECT>());
    (*Object).Kind = PipPropObjectDevice;
    (*Object).Pdo = Pdo;
    (*Object).DeviceNode = DeviceNode;
    (*Object).Name = core::ptr::addr_of_mut!((*DeviceNode).InstancePath) as PCUNICODE_STRING;

    if DeviceNode == IopRootDeviceNode {
        (*Object).Flags |= PIP_PROP_OBJECT_PROTECTED;
    }

    STATUS_SUCCESS
}
unsafe extern "C" fn PiPropInitInterfaceObject(
    SymbolicLinkName: PCUNICODE_STRING,
    Object: PPIP_PROP_OBJECT,
) -> NTSTATUS {
    // RtlZeroMemory(Object, sizeof(*Object)): write_bytes counts ELEMENTS.
    core::ptr::write_bytes(Object, 0, 1);

    if SymbolicLinkName.is_null()
        || (*SymbolicLinkName).Buffer.is_null()
        || (*SymbolicLinkName).Length == 0
        || PiPropParseInterfaceName(SymbolicLinkName, core::ptr::addr_of_mut!((*Object).ClassGuid)) == 0u8
    {
        return STATUS_INVALID_PARAMETER;
    }

    (*Object).Kind = PipPropObjectInterface;
    (*Object).Name = SymbolicLinkName;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PiPropReleaseDevice(DeviceObject: PDEVICE_OBJECT) {
    PAGED_CODE!();

    PiPropReplaceSaved(DeviceObject, DEVPROP_TYPE_EMPTY as DEVPROPTYPE, core::ptr::null_mut(), 0 as ULONG, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "system" fn IoSetDevicePropertyData(
    Pdo: PDEVICE_OBJECT,
    PropertyKey: *const DEVPROPKEY,
    Lcid: LCID,
    Flags: ULONG,
    mut Type: DEVPROPTYPE,
    mut Size: ULONG,
    Data: PVOID,
) -> NTSTATUS {
    let mut Object: PIP_PROP_OBJECT = PIP_PROP_OBJECT {
        Kind: PipPropObjectDevice,
        Flags: 0,
        Pdo: core::ptr::null_mut(),
        DeviceNode: core::ptr::null_mut(),
        Name: core::ptr::null(),
        ClassGuid: GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        },
    };
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    let _ = Flags; // UNREFERENCED_PARAMETER

    Status = PiPropInitDeviceObject(Pdo, &mut Object);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* A NULL buffer deletes the property */
    if Data.is_null() {
        Type = DEVPROP_TYPE_EMPTY as DEVPROPTYPE;
        Size = 0 as ULONG;
    }

    return PiPropSet(&mut Object, PropertyKey, Lcid, Type, Size, Data);
}
#[no_mangle]
pub unsafe extern "system" fn IoGetDevicePropertyData(
    Pdo: PDEVICE_OBJECT,
    PropertyKey: *const DEVPROPKEY,
    Lcid: LCID,
    Flags: ULONG,
    Size: ULONG,
    Data: PVOID,
    RequiredSize: PULONG,
    Type: PDEVPROPTYPE,
) -> NTSTATUS {
    let mut Object: PIP_PROP_OBJECT = PIP_PROP_OBJECT {
        Kind: PipPropObjectDevice,
        Flags: 0,
        Pdo: core::ptr::null_mut(),
        DeviceNode: core::ptr::null_mut(),
        Name: core::ptr::null(),
        ClassGuid: GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        },
    };
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    let _ = Flags; // UNREFERENCED_PARAMETER

    Status = PiPropInitDeviceObject(Pdo, &raw mut Object);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    return PiPropGet(
        &raw mut Object,
        PropertyKey,
        Lcid,
        Size,
        Data,
        RequiredSize,
        Type,
    );
}
#[no_mangle]
pub unsafe extern "system" fn IoSetDeviceInterfacePropertyData(
    SymbolicLinkName: PUNICODE_STRING,
    PropertyKey: *const DEVPROPKEY,
    Lcid: LCID,
    Flags: ULONG,
    mut Type: DEVPROPTYPE,
    mut Size: ULONG,
    Data: PVOID,
) -> NTSTATUS {
    let mut Object: PIP_PROP_OBJECT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    let _ = Flags; // UNREFERENCED_PARAMETER

    Status = PiPropInitInterfaceObject(SymbolicLinkName as PCUNICODE_STRING, &raw mut Object);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if Data.is_null() {
        Type = DEVPROP_TYPE_EMPTY;
        Size = 0;
    }

    return PiPropSet(&raw mut Object, PropertyKey, Lcid, Type, Size, Data);
}

#[no_mangle]
pub unsafe extern "system" fn IoGetDeviceInterfacePropertyData(
    SymbolicLinkName: PUNICODE_STRING,
    PropertyKey: *const DEVPROPKEY,
    Lcid: LCID,
    Flags: ULONG,
    Size: ULONG,
    Data: PVOID,
    RequiredSize: PULONG,
    Type: PDEVPROPTYPE,
) -> NTSTATUS {
    let mut Object: PIP_PROP_OBJECT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    let _ = Flags; // UNREFERENCED_PARAMETER

    Status = PiPropInitInterfaceObject(SymbolicLinkName as PCUNICODE_STRING, &raw mut Object);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    return PiPropGet(
        &raw mut Object,
        PropertyKey,
        Lcid,
        Size,
        Data,
        RequiredSize,
        Type,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     DEVPROPKEY property store for devices and device interfaces
//  * COPYRIGHT:   Copyright 2026 Justin Miller <justin.miller@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include <devpkey.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* DEFINES *******************************************************************/
//
// #define TAG_PNP_PROPERTY            'rPpP'
//
// /* Store values: <ObjectKey>\Properties\{FMTID}\<PID>, type PIP_PROP_REG_TYPE_MARK | DEVPROPTYPE */
// #define PIP_PROP_REG_TYPE_MARK      0xFFFF0000UL
//
// #define PIP_PROP_FMTID_CHARS        (38 + 1)
// #define PIP_PROP_PID_CHARS          (8 + 1)
// #define PIP_PROP_PATH_CHARS         (10 + 1 + 38 + 1 + 8 + 1)
// #define PIP_PROP_MAX_QUERY_SIZE     0x10000
//
// #define PIP_PROP_READ_ONLY          0x00000001
// #define PIP_PROP_ROOT_ENUM_WRITE    0x00000002
//
// #define PIP_PROP_OBJECT_PROTECTED   0x00000001
//
// /* TYPES *********************************************************************/
//
// typedef enum _PIP_PROP_OBJECT_KIND
// {
//     PipPropObjectDevice,
//     PipPropObjectInterface
// } PIP_PROP_OBJECT_KIND;
//
// typedef struct _PIP_PROP_OBJECT
// {
//     PIP_PROP_OBJECT_KIND Kind;
//     ULONG Flags;
//     PDEVICE_OBJECT Pdo;
//     PDEVICE_NODE DeviceNode;
//     PCUNICODE_STRING Name;
//     GUID ClassGuid;
// } PIP_PROP_OBJECT, *PPIP_PROP_OBJECT;
//
// typedef struct _PIP_PROP_NAME
// {
//     const DEVPROPKEY *Key;
//     UNICODE_STRING Locale;
//     UNICODE_STRING Fmtid;
//     UNICODE_STRING Pid;
//     WCHAR LocaleBuffer[LOCALE_NAME_MAX_LENGTH];
//     WCHAR FmtidBuffer[PIP_PROP_FMTID_CHARS];
//     WCHAR PidBuffer[PIP_PROP_PID_CHARS];
// } PIP_PROP_NAME, *PPIP_PROP_NAME;
//
// typedef enum _PIP_PROP_SOURCE
// {
//     PipSourceKeyValue,
//     PipSourceRegistryProperty,
//     PipSourceDeviceNode,
//     PipSourceInterface
// } PIP_PROP_SOURCE;
//
// typedef enum _PIP_PROP_FIELD
// {
//     PipFieldName,
//     PipFieldInstanceId,
//     PipFieldParent,
//     PipFieldStatus,
//     PipFieldProblemCode,
//     PipFieldHasProblem,
//     PipFieldIsPresent,
//     PipFieldInterfaceClass,
//     PipFieldInterfaceEnabled,
//     PipFieldInterfaceDevice,
//     PipFieldInterfaceReference
// } PIP_PROP_FIELD;
//
// typedef struct _PIP_BUILTIN_PROPERTY
// {
//     const DEVPROPKEY *Key;
//     DEVPROPTYPE Type;
//     PIP_PROP_SOURCE Source;
//     ULONG Flags;
//     PCWSTR ValueName;
//     ULONG Selector; /* REG_xxx, DEVICE_REGISTRY_PROPERTY or PIP_PROP_FIELD */
// } PIP_BUILTIN_PROPERTY, *PPIP_BUILTIN_PROPERTY;
//
// /* GLOBALS *******************************************************************/
//
// static const PIP_BUILTIN_PROPERTY PipDeviceBuiltins[] =
// {
//     { &DEVPKEY_Device_DeviceDesc,     DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        REGSTR_VAL_DEVDESC,              REG_SZ },
//     { &DEVPKEY_Device_HardwareIds,    DEVPROP_TYPE_STRING_LIST, PipSourceKeyValue,         PIP_PROP_ROOT_ENUM_WRITE, REGSTR_VAL_HARDWAREID,           REG_MULTI_SZ },
//     { &DEVPKEY_Device_CompatibleIds,  DEVPROP_TYPE_STRING_LIST, PipSourceKeyValue,         PIP_PROP_ROOT_ENUM_WRITE, REGSTR_VAL_COMPATIBLEIDS,        REG_MULTI_SZ },
//     { &DEVPKEY_Device_Service,        DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        REGSTR_VAL_SERVICE,              REG_SZ },
//     { &DEVPKEY_Device_Class,          DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        REGSTR_VAL_CLASS,                REG_SZ },
//     { &DEVPKEY_Device_ClassGuid,      DEVPROP_TYPE_GUID,        PipSourceKeyValue,         0,                        REGSTR_VAL_CLASSGUID,            REG_SZ },
//     { &DEVPKEY_Device_Driver,         DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        REGSTR_VAL_DRIVER,               REG_SZ },
//     { &DEVPKEY_Device_ConfigFlags,    DEVPROP_TYPE_UINT32,      PipSourceKeyValue,         0,                        REGSTR_VAL_CONFIGFLAGS,          REG_DWORD },
//     { &DEVPKEY_Device_Manufacturer,   DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        REGSTR_VAL_MFG,                  REG_SZ },
//     { &DEVPKEY_Device_FriendlyName,   DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        REGSTR_VAL_FRIENDLYNAME,         REG_SZ },
//     { &DEVPKEY_Device_LocationInfo,   DEVPROP_TYPE_STRING,      PipSourceKeyValue,         0,                        REGSTR_VAL_LOCATION_INFORMATION, REG_SZ },
//     { &DEVPKEY_Device_Capabilities,   DEVPROP_TYPE_UINT32,      PipSourceKeyValue,         PIP_PROP_READ_ONLY,       REGSTR_VAL_CAPABILITIES,         REG_DWORD },
//     { &DEVPKEY_Device_UINumber,       DEVPROP_TYPE_UINT32,      PipSourceKeyValue,         PIP_PROP_READ_ONLY,       REGSTR_VAL_UI_NUMBER,            REG_DWORD },
//     { &DEVPKEY_Device_UpperFilters,   DEVPROP_TYPE_STRING_LIST, PipSourceKeyValue,         0,                        REGSTR_VAL_UPPERFILTERS,         REG_MULTI_SZ },
//     { &DEVPKEY_Device_LowerFilters,   DEVPROP_TYPE_STRING_LIST, PipSourceKeyValue,         0,                        REGSTR_VAL_LOWERFILTERS,         REG_MULTI_SZ },
//
//     { &DEVPKEY_Device_PDOName,        DEVPROP_TYPE_STRING,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       NULL, DevicePropertyPhysicalDeviceObjectName },
//     { &DEVPKEY_Device_BusTypeGuid,    DEVPROP_TYPE_GUID,        PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       NULL, DevicePropertyBusTypeGuid },
//     { &DEVPKEY_Device_LegacyBusType,  DEVPROP_TYPE_UINT32,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       NULL, DevicePropertyLegacyBusType },
//     { &DEVPKEY_Device_BusNumber,      DEVPROP_TYPE_UINT32,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       NULL, DevicePropertyBusNumber },
//     { &DEVPKEY_Device_EnumeratorName, DEVPROP_TYPE_STRING,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       NULL, DevicePropertyEnumeratorName },
//     { &DEVPKEY_Device_Address,        DEVPROP_TYPE_UINT32,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       NULL, DevicePropertyAddress },
//     { &DEVPKEY_Device_RemovalPolicy,  DEVPROP_TYPE_UINT32,      PipSourceRegistryProperty, PIP_PROP_READ_ONLY,       NULL, DevicePropertyRemovalPolicy },
//
//     { &DEVPKEY_NAME,                  DEVPROP_TYPE_STRING,      PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       NULL, PipFieldName },
//     { &DEVPKEY_Device_InstanceId,     DEVPROP_TYPE_STRING,      PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       NULL, PipFieldInstanceId },
//     { &DEVPKEY_Device_Parent,         DEVPROP_TYPE_STRING,      PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       NULL, PipFieldParent },
//     { &DEVPKEY_Device_DevNodeStatus,  DEVPROP_TYPE_UINT32,      PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       NULL, PipFieldStatus },
//     { &DEVPKEY_Device_ProblemCode,    DEVPROP_TYPE_UINT32,      PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       NULL, PipFieldProblemCode },
//     { &DEVPKEY_Device_HasProblem,     DEVPROP_TYPE_BOOLEAN,     PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       NULL, PipFieldHasProblem },
//     { &DEVPKEY_Device_IsPresent,      DEVPROP_TYPE_BOOLEAN,     PipSourceDeviceNode,       PIP_PROP_READ_ONLY,       NULL, PipFieldIsPresent },
// };
//
// static const PIP_BUILTIN_PROPERTY PipInterfaceBuiltins[] =
// {
//     { &DEVPKEY_DeviceInterface_ClassGuid,       DEVPROP_TYPE_GUID,    PipSourceInterface,        PIP_PROP_READ_ONLY,       NULL, PipFieldInterfaceClass },
//     { &DEVPKEY_DeviceInterface_Enabled,         DEVPROP_TYPE_BOOLEAN, PipSourceInterface,        PIP_PROP_READ_ONLY,       NULL, PipFieldInterfaceEnabled },
//     { &DEVPKEY_Device_InstanceId,               DEVPROP_TYPE_STRING,  PipSourceInterface,        PIP_PROP_READ_ONLY,       NULL, PipFieldInterfaceDevice },
//     { &DEVPKEY_DeviceInterface_ReferenceString, DEVPROP_TYPE_STRING,  PipSourceInterface,        PIP_PROP_READ_ONLY,       NULL, PipFieldInterfaceReference },
// };
//
// static const DEVPROPKEY PiPropInterruptKey =
// {
//     { 0xF0E20F09, 0xD97A, 0x49A9, { 0x80, 0x46, 0xBB, 0x6E, 0x22, 0xE6, 0xBB, 0x2E } },
//     2
// };
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// static inline
// BOOLEAN
// PiPropIsSameKey(
//          const DEVPROPKEY *Key1,
//          const DEVPROPKEY *Key2)
// {
//     return (Key1->pid == Key2->pid) &&
//            RtlEqualMemory(&Key1->fmtid, &Key2->fmtid, sizeof(GUID));
// }
//
// typedef enum _PIP_PROP_CHECK
// {
//     PipCheckNone,
//     PipCheckNoData,
//     PipCheckBoolean,
//     PipCheckFileTime,
//     PipCheckString,
//     PipCheckSecurity
// } PIP_PROP_CHECK;
//
// typedef struct _PIP_PROP_TYPE_RULE
// {
//     UCHAR Width;
//     BOOLEAN ListAllowed;
//     UCHAR Check;
// } PIP_PROP_TYPE_RULE;
//
// /* Indexed by base DEVPROPTYPE, a zero width means variable length */
// static const PIP_PROP_TYPE_RULE PiPropTypeRules[MAX_DEVPROP_TYPE + 1] =
// {
//     { 0,                       FALSE, PipCheckNoData },     /* EMPTY */
//     { 0,                       FALSE, PipCheckNoData },     /* NULL */
//     { sizeof(CHAR),            FALSE, PipCheckNone },       /* SBYTE */
//     { sizeof(UCHAR),           FALSE, PipCheckNone },       /* BYTE */
//     { sizeof(SHORT),           FALSE, PipCheckNone },       /* INT16 */
//     { sizeof(USHORT),          FALSE, PipCheckNone },       /* UINT16 */
//     { sizeof(LONG),            FALSE, PipCheckNone },       /* INT32 */
//     { sizeof(ULONG),           FALSE, PipCheckNone },       /* UINT32 */
//     { sizeof(LONGLONG),        FALSE, PipCheckNone },       /* INT64 */
//     { sizeof(ULONGLONG),       FALSE, PipCheckNone },       /* UINT64 */
//     { sizeof(ULONG),           FALSE, PipCheckNone },       /* FLOAT */
//     { sizeof(ULONGLONG),       FALSE, PipCheckNone },       /* DOUBLE */
//     { 16,                      FALSE, PipCheckNone },       /* DECIMAL */
//     { sizeof(GUID),            FALSE, PipCheckNone },       /* GUID */
//     { sizeof(LONGLONG),        FALSE, PipCheckNone },       /* CURRENCY */
//     { sizeof(ULONGLONG),       FALSE, PipCheckNone },       /* DATE */
//     { sizeof(LARGE_INTEGER),   FALSE, PipCheckFileTime },   /* FILETIME */
//     { sizeof(DEVPROP_BOOLEAN), FALSE, PipCheckBoolean },    /* BOOLEAN */
//     { 0,                       TRUE,  PipCheckString },     /* STRING */
//     { 0,                       FALSE, PipCheckSecurity },   /* SECURITY_DESCRIPTOR */
//     { 0,                       TRUE,  PipCheckString },     /* SECURITY_DESCRIPTOR_STRING */
//     { sizeof(DEVPROPKEY),      FALSE, PipCheckNone },       /* DEVPROPKEY */
//     { sizeof(DEVPROPTYPE),     FALSE, PipCheckNone },       /* DEVPROPTYPE */
//     { sizeof(ULONG),           FALSE, PipCheckNone },       /* ERROR */
//     { sizeof(NTSTATUS),        FALSE, PipCheckNone },       /* NTSTATUS */
//     { 0,                       FALSE, PipCheckString },     /* STRING_INDIRECT */
// };
//
// static
// BOOLEAN
// PiPropCheckElements(
//          UCHAR Check,
//                                     PUCHAR Data,
//          ULONG Width,
//          ULONG Count)
// {
//     LONGLONG Time;
//
//     for (; Count; Count--, Data += Width)
//     {
//         if (Check == PipCheckBoolean)
//         {
//             if ((*(PDEVPROP_BOOLEAN)Data != DEVPROP_FALSE) &&
//                 (*(PDEVPROP_BOOLEAN)Data != DEVPROP_TRUE))
//                 return FALSE;
//         }
//         else if (Check == PipCheckFileTime)
//         {
//             RtlCopyMemory(&Time, Data, sizeof(Time));
//             if (Time < 0)
//                 return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// static
// BOOLEAN
// PiPropIsValidString(
//                            PCWSTR String,
//          ULONG Size)
// {
//     ULONG Chars, Index;
//
//     if ((Size < sizeof(WCHAR)) || (Size % sizeof(WCHAR)) ||
//         (Size > UNICODE_STRING_MAX_BYTES))
//     {
//         return FALSE;
//     }
//
//     Chars = Size / sizeof(WCHAR);
//     for (Index = 0; Index < Chars - 1; Index++)
//     {
//         if (String[Index] == UNICODE_NULL)
//             return FALSE;
//     }
//
//     return (String[Chars - 1] == UNICODE_NULL);
// }
//
// static
// BOOLEAN
// PiPropIsValidStringList(
//                            PCWSTR List,
//          ULONG Size)
// {
//     ULONG Chars, Index, EntryStart;
//
//     if ((Size < sizeof(WCHAR)) || (Size % sizeof(WCHAR)))
//         return FALSE;
//
//     Chars = Size / sizeof(WCHAR);
//     EntryStart = 0;
//
//     for (Index = 0; Index < Chars; Index++)
//     {
//         if (List[Index] != UNICODE_NULL)
//             continue;
//
//         if (Index == EntryStart)
//             return (Index == Chars - 1);
//
//         if ((Index - EntryStart + 1) * sizeof(WCHAR) > UNICODE_STRING_MAX_BYTES)
//             return FALSE;
//
//         EntryStart = Index + 1;
//     }
//
//     return FALSE;
// }
//
// static
// NTSTATUS
// PiPropCheckPayload(
//          DEVPROPTYPE Type,
//                                PVOID Data,
//          ULONG Size)
// {
//     DEVPROPTYPE BaseType = Type & DEVPROP_MASK_TYPE;
//     const PIP_PROP_TYPE_RULE *Rule;
//     BOOLEAN Valid;
//
//     if ((BaseType > MAX_DEVPROP_TYPE) || (Size && !Data))
//         return STATUS_INVALID_PARAMETER;
//
//     Rule = &PiPropTypeRules[BaseType];
//
//     /* Everything above the base type must be a single known modifier */
//     switch (Type - BaseType)
//     {
//         case 0:
//             if (Rule->Width)
//                 Valid = (Size == Rule->Width);
//             else if (Rule->Check == PipCheckNoData)
//                 Valid = (Size == 0);
//             else if (Rule->Check == PipCheckString)
//                 Valid = PiPropIsValidString(Data, Size);
//             else
//                 Valid = (Size != 0) &&
//                         RtlValidRelativeSecurityDescriptor(Data, Size, 0) &&
//                         (RtlLengthSecurityDescriptor(Data) == Size);
//             break;
//
//         case DEVPROP_TYPEMOD_ARRAY:
//             Valid = (Rule->Width != 0) && (Size != 0) && !(Size % Rule->Width);
//             break;
//
//         case DEVPROP_TYPEMOD_LIST:
//             Valid = Rule->ListAllowed && PiPropIsValidStringList(Data, Size);
//             break;
//
//         default:
//             Valid = FALSE;
//             break;
//     }
//
//     if (Valid && Rule->Width && (Rule->Check != PipCheckNone))
//         Valid = PiPropCheckElements(Rule->Check, Data, Rule->Width, Size / Rule->Width);
//
//     return Valid ? STATUS_SUCCESS : STATUS_INVALID_PARAMETER;
// }
//
// static
// NTSTATUS
// PiPropInitLocale(
//           PPIP_PROP_NAME Name,
//          LCID Lcid)
// {
//     RtlInitEmptyUnicodeString(&Name->Locale,
//                               Name->LocaleBuffer,
//                               sizeof(Name->LocaleBuffer));
//
//     if ((Lcid != LOCALE_NEUTRAL) && !RtlLCIDToCultureName(Lcid, &Name->Locale))
//     {
//         DPRINT1("Cannot resolve LCID 0x%lx\n", Lcid);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// PiPropInitName(
//           PPIP_PROP_NAME Name,
//          const DEVPROPKEY *Key,
//          LCID Lcid)
// {
//     const GUID *Fmtid = &Key->fmtid;
//     NTSTATUS Status;
//
//     Name->Key = Key;
//
//     Status = PiPropInitLocale(Name, Lcid);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = RtlStringCchPrintfW(Name->FmtidBuffer,
//                                  RTL_NUMBER_OF(Name->FmtidBuffer),
//                                  L"{%08lx-%04x-%04x-%02x%02x-%02x%02x%02x%02x%02x%02x}",
//                                  Fmtid->Data1, Fmtid->Data2, Fmtid->Data3,
//                                  Fmtid->Data4[0], Fmtid->Data4[1],
//                                  Fmtid->Data4[2], Fmtid->Data4[3],
//                                  Fmtid->Data4[4], Fmtid->Data4[5],
//                                  Fmtid->Data4[6], Fmtid->Data4[7]);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = RtlStringCchPrintfW(Name->PidBuffer,
//                                  RTL_NUMBER_OF(Name->PidBuffer),
//                                  L"%04lX",
//                                  Key->pid);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     RtlInitUnicodeString(&Name->Fmtid, Name->FmtidBuffer);
//     RtlInitUnicodeString(&Name->Pid, Name->PidBuffer);
//     return STATUS_SUCCESS;
// }
//
// static
// BOOLEAN
// PiPropIsInterruptKey(
//          PPIP_PROP_OBJECT Object,
//          PPIP_PROP_NAME Name)
// {
//     return (Object->Pdo != NULL) &&
//            (Name->Locale.Length == 0) &&
//            PiPropIsSameKey(Name->Key, &PiPropInterruptKey);
// }
//
// static
// NTSTATUS
// PiPropNormalizeStatus(
//          NTSTATUS Status)
// {
//     if ((Status == STATUS_NOT_FOUND) ||
//         (Status == STATUS_OBJECT_PATH_NOT_FOUND) ||
//         (Status == STATUS_KEY_DELETED))
//     {
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     return Status;
// }
//
// static
// NTSTATUS
// PiPropCopyOut(
//          DEVPROPTYPE SourceType,
//                                      PVOID Source,
//          ULONG SourceSize,
//           PDEVPROPTYPE Type,
//                                                    PVOID Data,
//          ULONG Size,
//           PULONG RequiredSize)
// {
//     *Type = SourceType;
//     *RequiredSize = SourceSize;
//
//     /* An empty buffer only satisfies DEVPROP_TYPE_NULL */
//     if ((Size < SourceSize) || (!Size && (SourceType != DEVPROP_TYPE_NULL)))
//         return STATUS_BUFFER_TOO_SMALL;
//
//     if (SourceSize)
//         RtlCopyMemory(Data, Source, SourceSize);
//
//     return STATUS_SUCCESS;
// }
//
// /* \??\<Device>#{ClassGuid}[\RefString] */
// static
// BOOLEAN
// PiPropParseInterfaceName(
//          PCUNICODE_STRING LinkName,
//           LPGUID ClassGuid)
// {
//     static const UNICODE_STRING KernelPrefix = RTL_CONSTANT_STRING(L"\\??\\");
//     static const UNICODE_STRING UserPrefix = RTL_CONSTANT_STRING(L"\\\\?\\");
//     UNICODE_STRING GuidString;
//     USHORT Index, Chars;
//
//     if (LinkName->Length % sizeof(WCHAR))
//         return FALSE;
//
//     if (!RtlPrefixUnicodeString(&KernelPrefix, LinkName, FALSE) &&
//         !RtlPrefixUnicodeString(&UserPrefix, LinkName, FALSE))
//     {
//         return FALSE;
//     }
//
//     Chars = LinkName->Length / sizeof(WCHAR);
//     Index = 4;
//     while ((Index < Chars) && (LinkName->Buffer[Index] != L'\\'))
//         Index++;
//
//     if (Index < 4 + 1 + 38)
//         return FALSE;
//
//     GuidString.Buffer = &LinkName->Buffer[Index - 38];
//     GuidString.Length = GuidString.MaximumLength = 38 * sizeof(WCHAR);
//
//     return NT_SUCCESS(RtlGUIDFromString(&GuidString, ClassGuid));
// }
//
// /* Text after the backslash that follows {ClassGuid}, which may be empty */
// static
// NTSTATUS
// NTAPI
// PiPropGetInterfaceReference(
//          PCUNICODE_STRING LinkName,
//           PUNICODE_STRING Reference)
// {
//     USHORT Start, End, Chars;
//
//     /* Both link prefixes are four characters */
//     Chars = LinkName->Length / sizeof(WCHAR);
//     Start = 4;
//     while ((Start < Chars) && (LinkName->Buffer[Start] != L'\\'))
//         Start++;
//
//     if (Start >= Chars)
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//
//     Start++;
//     for (End = Start; End < Chars; End++)
//     {
//         if ((LinkName->Buffer[End] == L'\\') || (LinkName->Buffer[End] == L'/'))
//             return STATUS_OBJECT_PATH_INVALID;
//     }
//
//     Reference->Buffer = &LinkName->Buffer[Start];
//     Reference->Length = (End - Start) * sizeof(WCHAR);
//     Reference->MaximumLength = Reference->Length;
//     return STATUS_SUCCESS;
// }
//
// /* Property store ************************************************************/
//
// static
// NTSTATUS
// PiPropOpenObjectKey(
//          PPIP_PROP_OBJECT Object,
//          ACCESS_MASK DesiredAccess,
//           PHANDLE ObjectKey)
// {
//     UNICODE_STRING EnumKeyName =
//         RTL_CONSTANT_STRING(L"\\Registry\\Machine\\System\\CurrentControlSet\\Enum");
//     HANDLE EnumKey;
//     NTSTATUS Status;
//
//     *ObjectKey = NULL;
//
//     switch (Object->Kind)
//     {
//         case PipPropObjectInterface:
//             return IopOpenDeviceInterfaceKeys(Object->Name, DesiredAccess, NULL, ObjectKey);
//
//         default:
//             break;
//     }
//
//     Status = IopOpenRegistryKeyEx(&EnumKey, NULL, &EnumKeyName, KEY_READ);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = IopOpenRegistryKeyEx(ObjectKey,
//                                   EnumKey,
//                                   (PUNICODE_STRING)Object->Name,
//                                   DesiredAccess);
//     ZwClose(EnumKey);
//     return Status;
// }
//
// static
// NTSTATUS
// PiPropBuildValuePath(
//          PPIP_PROP_NAME Name,
//           PUNICODE_STRING Path,
//                                       PWCHAR Buffer)
// {
//     RtlInitEmptyUnicodeString(Path, Buffer, PIP_PROP_PATH_CHARS * sizeof(WCHAR));
//     return RtlUnicodeStringPrintf(Path,
//                                   L"%s\\%wZ\\%wZ",
//                                   REGSTR_KEY_DEVICE_PROPERTIES,
//                                   &Name->Fmtid,
//                                   &Name->Pid);
// }
//
// static
// NTSTATUS
// PiPropStoreQuery(
//          HANDLE ObjectKey,
//          PPIP_PROP_NAME Name,
//           PDEVPROPTYPE Type,
//                                                    PVOID Data,
//          ULONG Size,
//           PULONG RequiredSize)
// {
//     struct
//     {
//         KEY_VALUE_PARTIAL_INFORMATION Info;
//         UCHAR Inline[64];
//     } Small;
//     PKEY_VALUE_PARTIAL_INFORMATION Info = &Small.Info;
//     WCHAR PathBuffer[PIP_PROP_PATH_CHARS];
//     UNICODE_STRING Path;
//     HANDLE PropertyKey;
//     ULONG InfoSize;
//     NTSTATUS Status;
//
//     Status = PiPropBuildValuePath(Name, &Path, PathBuffer);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = IopOpenRegistryKeyEx(&PropertyKey, ObjectKey, &Path, KEY_QUERY_VALUE);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     InfoSize = sizeof(Small);
//     for (;;)
//     {
//         Status = ZwQueryValueKey(PropertyKey,
//                                  &Name->Locale,
//                                  KeyValuePartialInformation,
//                                  Info,
//                                  InfoSize,
//                                  &InfoSize);
//         if ((Status != STATUS_BUFFER_OVERFLOW) && (Status != STATUS_BUFFER_TOO_SMALL))
//             break;
//
//         if (Info != &Small.Info)
//             ExFreePoolWithTag(Info, TAG_PNP_PROPERTY);
//
//         Info = ExAllocatePoolWithTag(PagedPool, InfoSize, TAG_PNP_PROPERTY);
//         if (!Info)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             break;
//         }
//     }
//     ZwClose(PropertyKey);
//
//     if (NT_SUCCESS(Status))
//     {
//         if ((Info->Type & PIP_PROP_REG_TYPE_MARK) == PIP_PROP_REG_TYPE_MARK)
//         {
//             Status = PiPropCopyOut(Info->Type & ~PIP_PROP_REG_TYPE_MARK,
//                                    Info->Data,
//                                    Info->DataLength,
//                                    Type,
//                                    Data,
//                                    Size,
//                                    RequiredSize);
//         }
//         else
//         {
//             DPRINT1("Unexpected value type 0x%lx for %wZ\\%wZ\n",
//                     Info->Type, &Name->Fmtid, &Name->Pid);
//             Status = STATUS_OBJECT_NAME_NOT_FOUND;
//         }
//     }
//
//     if (Info && (Info != &Small.Info))
//         ExFreePoolWithTag(Info, TAG_PNP_PROPERTY);
//
//     return Status;
// }
//
// static
// NTSTATUS
// PiPropPutValue(
//          HANDLE ValueKey,
//          PPIP_PROP_NAME Name,
//          DEVPROPTYPE Type,
//                                PVOID Data,
//          ULONG Size)
// {
//     return ZwSetValueKey(ValueKey,
//                          &Name->Locale,
//                          0,
//                          PIP_PROP_REG_TYPE_MARK | Type,
//                          Data,
//                          Size);
// }
//
// static
// NTSTATUS
// PiPropCreateBranch(
//          HANDLE Parent,
//                       PCUNICODE_STRING *Parts,
//          ULONG Depth,
//          PPIP_PROP_NAME Name,
//          DEVPROPTYPE Type,
//                                PVOID Data,
//          ULONG Size)
// {
//     const ACCESS_MASK Access = KEY_CREATE_SUB_KEY | KEY_SET_VALUE | DELETE;
//     ULONG Disposition;
//     HANDLE Child;
//     NTSTATUS Status;
//
//     Status = IopCreateRegistryKeyEx(&Child, Parent, (PUNICODE_STRING)Parts[0], Access,
//                                     REG_OPTION_NON_VOLATILE, &Disposition);
//     if (Status == STATUS_CHILD_MUST_BE_VOLATILE)
//     {
//         Status = IopCreateRegistryKeyEx(&Child, Parent, (PUNICODE_STRING)Parts[0], Access,
//                                         REG_OPTION_VOLATILE, &Disposition);
//     }
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (Depth > 1)
//         Status = PiPropCreateBranch(Child, Parts + 1, Depth - 1, Name, Type, Data, Size);
//     else
//         Status = PiPropPutValue(Child, Name, Type, Data, Size);
//
//     if (!NT_SUCCESS(Status) && (Disposition == REG_CREATED_NEW_KEY))
//         ZwDeleteKey(Child);
//
//     ZwClose(Child);
//     return Status;
// }
//
// static
// NTSTATUS
// PiPropStoreWrite(
//          HANDLE ObjectKey,
//          PPIP_PROP_NAME Name,
//          DEVPROPTYPE Type,
//                                PVOID Data,
//          ULONG Size)
// {
//     UNICODE_STRING PropertiesName = RTL_CONSTANT_STRING(REGSTR_KEY_DEVICE_PROPERTIES);
//     PCUNICODE_STRING Parts[] = { &PropertiesName, &Name->Fmtid, &Name->Pid };
//     WCHAR PathBuffer[PIP_PROP_PATH_CHARS];
//     UNICODE_STRING Path;
//     HANDLE ValueKey;
//     NTSTATUS Status;
//
//     Status = PiPropBuildValuePath(Name, &Path, PathBuffer);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = IopOpenRegistryKeyEx(&ValueKey, ObjectKey, &Path, KEY_SET_VALUE);
//     if (!NT_SUCCESS(Status))
//         return PiPropCreateBranch(ObjectKey, Parts, RTL_NUMBER_OF(Parts), Name, Type, Data, Size);
//
//     Status = PiPropPutValue(ValueKey, Name, Type, Data, Size);
//     ZwClose(ValueKey);
//     return Status;
// }
//
// static
// BOOLEAN
// PiPropIsKeyEmpty(
//          HANDLE KeyHandle)
// {
//     struct
//     {
//         KEY_FULL_INFORMATION Info;
//         WCHAR ClassName[32];
//     } Buffer;
//     ULONG ResultLength;
//     NTSTATUS Status;
//
//     Status = ZwQueryKey(KeyHandle,
//                         KeyFullInformation,
//                         &Buffer,
//                         sizeof(Buffer),
//                         &ResultLength);
//
//     return NT_SUCCESS(Status) &&
//            (Buffer.Info.SubKeys == 0) &&
//            (Buffer.Info.Values == 0);
// }
//
// static
// NTSTATUS
// PiPropStoreDelete(
//          HANDLE ObjectKey,
//          PPIP_PROP_NAME Name)
// {
//     UNICODE_STRING PropertiesName = RTL_CONSTANT_STRING(REGSTR_KEY_DEVICE_PROPERTIES);
//     PUNICODE_STRING Chain[] = { &PropertiesName, &Name->Fmtid, &Name->Pid };
//     HANDLE Keys[RTL_NUMBER_OF(Chain) + 1];
//     ULONG Depth;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     Keys[0] = ObjectKey;
//     for (Depth = 0; Depth < RTL_NUMBER_OF(Chain); Depth++)
//     {
//         Status = IopOpenRegistryKeyEx(&Keys[Depth + 1],
//                                       Keys[Depth],
//                                       Chain[Depth],
//                                       KEY_QUERY_VALUE | KEY_SET_VALUE | DELETE);
//         if (!NT_SUCCESS(Status))
//             break;
//     }
//
//     if (NT_SUCCESS(Status))
//         Status = ZwDeleteValueKey(Keys[Depth], &Name->Locale);
//
//     for (; Depth > 0; Depth--)
//     {
//         if (NT_SUCCESS(Status) && PiPropIsKeyEmpty(Keys[Depth]))
//             ZwDeleteKey(Keys[Depth]);
//
//         ZwClose(Keys[Depth]);
//     }
//
//     return Status;
// }
//
// /* Saved interrupt data ******************************************************/
//
// typedef struct _PIP_PROP_SAVED_VALUE
// {
//     LIST_ENTRY Link;
//     PDEVICE_OBJECT Pdo;
//     DEVPROPTYPE Type;
//     ULONG Size;
//     UCHAR Data[ANYSIZE_ARRAY];
// } PIP_PROP_SAVED_VALUE, *PPIP_PROP_SAVED_VALUE;
//
// static RTL_STATIC_LIST_HEAD(PiPropSavedValues);
// static EX_PUSH_LOCK PiPropSavedLock;
//
// /* Caller holds PiPropSavedLock */
// static
// PPIP_PROP_SAVED_VALUE
// PiPropFindSaved(
//          PDEVICE_OBJECT Pdo)
// {
//     PPIP_PROP_SAVED_VALUE Saved;
//     PLIST_ENTRY Entry;
//
//     for (Entry = PiPropSavedValues.Flink; Entry != &PiPropSavedValues; Entry = Entry->Flink)
//     {
//         Saved = CONTAINING_RECORD(Entry, PIP_PROP_SAVED_VALUE, Link);
//         if (Saved->Pdo == Pdo)
//             return Saved;
//     }
//
//     return NULL;
// }
//
// /* DEVPROP_TYPE_EMPTY removes the value */
// static
// NTSTATUS
// PiPropReplaceSaved(
//          PDEVICE_OBJECT Pdo,
//          DEVPROPTYPE Type,
//                                PVOID Data,
//          ULONG Size,
//               PBOOLEAN Replaced)
// {
//     PPIP_PROP_SAVED_VALUE Fresh = NULL, Stale;
//
//     if (Type != DEVPROP_TYPE_EMPTY)
//     {
//         Fresh = ExAllocatePoolWithTag(PagedPool,
//                                       FIELD_OFFSET(PIP_PROP_SAVED_VALUE, Data) + Size,
//                                       TAG_PNP_PROPERTY);
//         if (!Fresh)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         Fresh->Pdo = Pdo;
//         Fresh->Type = Type;
//         Fresh->Size = Size;
//         if (Size)
//             RtlCopyMemory(Fresh->Data, Data, Size);
//     }
//
//     KeEnterCriticalRegion();
//     ExAcquirePushLockExclusive(&PiPropSavedLock);
//
//     Stale = PiPropFindSaved(Pdo);
//     if (Stale)
//         RemoveEntryList(&Stale->Link);
//     if (Fresh)
//         InsertTailList(&PiPropSavedValues, &Fresh->Link);
//
//     ExReleasePushLockExclusive(&PiPropSavedLock);
//     KeLeaveCriticalRegion();
//
//     if (Replaced)
//         *Replaced = (Stale != NULL);
//
//     if (Stale)
//         ExFreePoolWithTag(Stale, TAG_PNP_PROPERTY);
//
//     return STATUS_SUCCESS;
// }
//
// /* Builtin properties ********************************************************/
//
// static
// const PIP_BUILTIN_PROPERTY *
// PiPropBuiltinTable(
//          PIP_PROP_OBJECT_KIND Kind,
//           PULONG Count)
// {
//     switch (Kind)
//     {
//         case PipPropObjectDevice:
//             *Count = RTL_NUMBER_OF(PipDeviceBuiltins);
//             return PipDeviceBuiltins;
//
//         case PipPropObjectInterface:
//             *Count = RTL_NUMBER_OF(PipInterfaceBuiltins);
//             return PipInterfaceBuiltins;
//
//         default:
//             *Count = 0;
//             return NULL;
//     }
// }
//
// static
// const PIP_BUILTIN_PROPERTY *
// PiPropFindBuiltin(
//          PPIP_PROP_OBJECT Object,
//          const DEVPROPKEY *Key)
// {
//     const PIP_BUILTIN_PROPERTY *Table;
//     ULONG Count;
//
//     Table = PiPropBuiltinTable(Object->Kind, &Count);
//     for (; Count; Count--, Table++)
//     {
//         if (PiPropIsSameKey(Table->Key, Key))
//             return Table;
//     }
//
//     return NULL;
// }
//
// static
// NTSTATUS
// PiPropAllocateBlob(
//                            PVOID Source,
//          ULONG Size,
//                                                PVOID *Blob)
// {
//     *Blob = NULL;
//     if (Size == 0)
//         return STATUS_INVALID_PARAMETER;
//
//     *Blob = ExAllocatePoolWithTag(PagedPool, Size, TAG_PNP_PROPERTY);
//     if (!*Blob)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     RtlCopyMemory(*Blob, Source, Size);
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// PiPropCaptureString(
//                               PCWSTR Raw,
//          ULONG RawSize,
//          BOOLEAN IsList,
//                                                     PVOID *Blob,
//           PULONG BlobSize)
// {
//     ULONG RawChars = RawSize / sizeof(WCHAR);
//     ULONG Index, Out = 0;
//     BOOLEAN InEntry = FALSE;
//     PWCHAR String;
//
//     String = ExAllocatePoolWithTag(PagedPool, (RawChars + 2) * sizeof(WCHAR), TAG_PNP_PROPERTY);
//     if (!String)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     *Blob = String;
//
//     for (Index = 0; Index < RawChars; Index++)
//     {
//         if (Raw[Index] != UNICODE_NULL)
//         {
//             String[Out++] = Raw[Index];
//             InEntry = TRUE;
//             continue;
//         }
//
//         if (!IsList || !InEntry)
//             break;
//
//         String[Out++] = UNICODE_NULL;
//         InEntry = FALSE;
//     }
//
//     if (!IsList || InEntry)
//         String[Out++] = UNICODE_NULL;
//     if (IsList)
//         String[Out++] = UNICODE_NULL;
//
//     *BlobSize = Out * sizeof(WCHAR);
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// PiPropReadRegistryValue(
//          HANDLE KeyHandle,
//          PCWSTR ValueName,
//          ULONG RegType,
//          DEVPROPTYPE Type,
//                                                     PVOID *Blob,
//           PULONG BlobSize)
// {
//     PKEY_VALUE_FULL_INFORMATION Info;
//     PVOID RawData;
//     ULONG RawSize;
//     NTSTATUS Status;
//
//     Status = IopGetRegistryValue(KeyHandle, (PWSTR)ValueName, &Info);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     RawData = (PUCHAR)Info + Info->DataOffset;
//     RawSize = Info->DataLength;
//
//     /* REG_EXPAND_SZ is accepted where REG_SZ is expected */
//     if ((Info->Type != RegType) &&
//         !((RegType == REG_SZ) && (Info->Type == REG_EXPAND_SZ)))
//     {
//         DPRINT1("Value '%S' has type %lu, expected %lu\n", ValueName, Info->Type, RegType);
//         ExFreePool(Info);
//         return STATUS_OBJECT_TYPE_MISMATCH;
//     }
//
//     switch (Type)
//     {
//         case DEVPROP_TYPE_STRING:
//         case DEVPROP_TYPE_STRING_LIST:
//             Status = PiPropCaptureString(RawData,
//                                          RawSize,
//                                          (Type == DEVPROP_TYPE_STRING_LIST),
//                                          Blob,
//                                          BlobSize);
//             break;
//
//         case DEVPROP_TYPE_UINT32:
//             if (RawSize != sizeof(ULONG))
//             {
//                 Status = STATUS_OBJECT_TYPE_MISMATCH;
//                 break;
//             }
//             Status = PiPropAllocateBlob(RawData, RawSize, Blob);
//             *BlobSize = RawSize;
//             break;
//
//         case DEVPROP_TYPE_GUID:
//         {
//             UNICODE_STRING GuidString;
//             GUID Guid;
//
//             Status = PiPropCaptureString(RawData, RawSize, FALSE, Blob, BlobSize);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             RtlInitUnicodeString(&GuidString, *Blob);
//             Status = RtlGUIDFromString(&GuidString, &Guid);
//             if (NT_SUCCESS(Status))
//             {
//                 RtlCopyMemory(*Blob, &Guid, sizeof(Guid));
//                 *BlobSize = sizeof(Guid);
//             }
//             else
//             {
//                 ExFreePoolWithTag(*Blob, TAG_PNP_PROPERTY);
//                 Status = STATUS_OBJECT_TYPE_MISMATCH;
//             }
//             break;
//         }
//
//         default:
//             ASSERT(FALSE);
//             Status = STATUS_INTERNAL_ERROR;
//             break;
//     }
//
//     ExFreePool(Info);
//     return Status;
// }
//
// static
// NTSTATUS
// PiPropReadKeyValue(
//          PPIP_PROP_OBJECT Object,
//          PCWSTR ValueName,
//          ULONG RegType,
//          DEVPROPTYPE Type,
//                                                     PVOID *Blob,
//           PULONG BlobSize)
// {
//     HANDLE ObjectKey;
//     NTSTATUS Status;
//
//     Status = PiPropOpenObjectKey(Object, KEY_QUERY_VALUE, &ObjectKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = PiPropReadRegistryValue(ObjectKey, ValueName, RegType, Type, Blob, BlobSize);
//     ZwClose(ObjectKey);
//     return Status;
// }
//
// static
// NTSTATUS
// PiPropReadDeviceRegistryProperty(
//          PPIP_PROP_OBJECT Object,
//          const PIP_BUILTIN_PROPERTY *Builtin,
//                                                     PVOID *Blob,
//           PULONG BlobSize)
// {
//     ULONG Length = 128, Needed;
//     PVOID Buffer;
//     NTSTATUS Status;
//
//     if (!Object->Pdo)
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//
//     for (;;)
//     {
//         Buffer = ExAllocatePoolWithTag(PagedPool, Length, TAG_PNP_PROPERTY);
//         if (!Buffer)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         Status = IoGetDeviceProperty(Object->Pdo,
//                                      (DEVICE_REGISTRY_PROPERTY)Builtin->Selector,
//                                      Length,
//                                      Buffer,
//                                      &Needed);
//         if (Status != STATUS_BUFFER_TOO_SMALL)
//             break;
//
//         ExFreePoolWithTag(Buffer, TAG_PNP_PROPERTY);
//
//         Length = ALIGN_UP_BY(max(Needed, Length * 2), sizeof(WCHAR));
//         if (Length > PIP_PROP_MAX_QUERY_SIZE)
//             return STATUS_UNSUCCESSFUL;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(Buffer, TAG_PNP_PROPERTY);
//         return Status;
//     }
//
//     if (Builtin->Type == DEVPROP_TYPE_STRING)
//     {
//         Status = PiPropCaptureString(Buffer, Needed, FALSE, Blob, BlobSize);
//         ExFreePoolWithTag(Buffer, TAG_PNP_PROPERTY);
//         return Status;
//     }
//
//     if (Needed != PiPropTypeRules[Builtin->Type].Width)
//     {
//         ExFreePoolWithTag(Buffer, TAG_PNP_PROPERTY);
//         return STATUS_OBJECT_TYPE_MISMATCH;
//     }
//
//     *Blob = Buffer;
//     *BlobSize = Needed;
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// PiPropReadDeviceNodeField(
//          PPIP_PROP_OBJECT Object,
//          PIP_PROP_FIELD Field,
//                                                     PVOID *Blob,
//           PULONG BlobSize)
// {
//     PDEVICE_NODE DeviceNode = Object->DeviceNode;
//     DEVPROP_BOOLEAN Boolean;
//     ULONG Value;
//     NTSTATUS Status;
//
//     switch (Field)
//     {
//         case PipFieldName:
//             Status = PiPropReadKeyValue(Object, REGSTR_VAL_FRIENDLYNAME, REG_SZ,
//                                         DEVPROP_TYPE_STRING, Blob, BlobSize);
//             if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//             {
//                 Status = PiPropReadKeyValue(Object, REGSTR_VAL_DEVDESC, REG_SZ,
//                                             DEVPROP_TYPE_STRING, Blob, BlobSize);
//             }
//             return Status;
//
//         case PipFieldInstanceId:
//             return PiPropCaptureString(Object->Name->Buffer,
//                                        Object->Name->Length,
//                                        FALSE,
//                                        Blob,
//                                        BlobSize);
//
//         case PipFieldIsPresent:
//             Boolean = (DeviceNode && !(DeviceNode->Flags & DNF_DEVICE_GONE)) ? DEVPROP_TRUE : DEVPROP_FALSE;
//             *BlobSize = sizeof(Boolean);
//             return PiPropAllocateBlob(&Boolean, sizeof(Boolean), Blob);
//
//         default:
//             break;
//     }
//
//     if (!DeviceNode)
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//
//     switch (Field)
//     {
//         case PipFieldParent:
//             if (!DeviceNode->Parent)
//                 return STATUS_OBJECT_NAME_NOT_FOUND;
//
//             return PiPropCaptureString(DeviceNode->Parent->InstancePath.Buffer,
//                                        DeviceNode->Parent->InstancePath.Length,
//                                        FALSE,
//                                        Blob,
//                                        BlobSize);
//
//         case PipFieldStatus:
//             Value = IopGetDeviceNodeStatus(DeviceNode);
//             break;
//
//         case PipFieldProblemCode:
//             Value = DeviceNode->Problem;
//             break;
//
//         case PipFieldHasProblem:
//             Boolean = (DeviceNode->Flags & DNF_HAS_PROBLEM) ? DEVPROP_TRUE : DEVPROP_FALSE;
//             *BlobSize = sizeof(Boolean);
//             return PiPropAllocateBlob(&Boolean, sizeof(Boolean), Blob);
//
//         default:
//             ASSERT(FALSE);
//             return STATUS_INTERNAL_ERROR;
//     }
//
//     *BlobSize = sizeof(Value);
//     return PiPropAllocateBlob(&Value, sizeof(Value), Blob);
// }
//
// static
// NTSTATUS
// PiPropReadInterfaceField(
//          PPIP_PROP_OBJECT Object,
//          PIP_PROP_FIELD Field,
//                                                     PVOID *Blob,
//           PULONG BlobSize)
// {
//     UNICODE_STRING ControlName = RTL_CONSTANT_STRING(L"Control");
//     HANDLE DeviceKey, InstanceKey, ControlKey;
//     PKEY_VALUE_FULL_INFORMATION Info;
//     DEVPROP_BOOLEAN Enabled;
//     UNICODE_STRING Reference;
//     NTSTATUS Status;
//
//     Status = IopOpenDeviceInterfaceKeys(Object->Name, KEY_READ, &DeviceKey, &InstanceKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     switch (Field)
//     {
//         case PipFieldInterfaceClass:
//             *BlobSize = sizeof(GUID);
//             Status = PiPropAllocateBlob(&Object->ClassGuid, sizeof(GUID), Blob);
//             break;
//
//         case PipFieldInterfaceEnabled:
//             Enabled = DEVPROP_FALSE;
//             if (NT_SUCCESS(IopOpenRegistryKeyEx(&ControlKey, InstanceKey, &ControlName, KEY_QUERY_VALUE)))
//             {
//                 if (NT_SUCCESS(IopGetRegistryValue(ControlKey, L"Linked", &Info)))
//                 {
//                     if ((Info->Type == REG_DWORD) &&
//                         (Info->DataLength == sizeof(ULONG)) &&
//                         (*(PULONG)((PUCHAR)Info + Info->DataOffset) != 0))
//                     {
//                         Enabled = DEVPROP_TRUE;
//                     }
//                     ExFreePool(Info);
//                 }
//                 ZwClose(ControlKey);
//             }
//
//             *BlobSize = sizeof(Enabled);
//             Status = PiPropAllocateBlob(&Enabled, sizeof(Enabled), Blob);
//             break;
//
//         case PipFieldInterfaceDevice:
//             Status = PiPropReadRegistryValue(DeviceKey, L"DeviceInstance", REG_SZ,
//                                              DEVPROP_TYPE_STRING, Blob, BlobSize);
//             break;
//
//         case PipFieldInterfaceReference:
//             Status = PiPropGetInterfaceReference(Object->Name, &Reference);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             Status = PiPropCaptureString(Reference.Buffer, Reference.Length, FALSE, Blob, BlobSize);
//             break;
//
//         default:
//             ASSERT(FALSE);
//             Status = STATUS_INTERNAL_ERROR;
//             break;
//     }
//
//     ZwClose(InstanceKey);
//     ZwClose(DeviceKey);
//     return Status;
// }
//
// static
// NTSTATUS
// PiPropReadBuiltin(
//          PPIP_PROP_OBJECT Object,
//          const PIP_BUILTIN_PROPERTY *Builtin,
//                                                     PVOID *Blob,
//           PULONG BlobSize)
// {
//     switch (Builtin->Source)
//     {
//         case PipSourceKeyValue:
//             return PiPropReadKeyValue(Object,
//                                       Builtin->ValueName,
//                                       Builtin->Selector,
//                                       Builtin->Type,
//                                       Blob,
//                                       BlobSize);
//
//         case PipSourceRegistryProperty:
//             return PiPropReadDeviceRegistryProperty(Object, Builtin, Blob, BlobSize);
//
//         case PipSourceDeviceNode:
//             return PiPropReadDeviceNodeField(Object, Builtin->Selector, Blob, BlobSize);
//
//         case PipSourceInterface:
//             return PiPropReadInterfaceField(Object, Builtin->Selector, Blob, BlobSize);
//
//         default:
//             ASSERT(FALSE);
//             return STATUS_INTERNAL_ERROR;
//     }
// }
//
// static
// NTSTATUS
// PiPropWriteBuiltin(
//          PPIP_PROP_OBJECT Object,
//          const PIP_BUILTIN_PROPERTY *Builtin,
//          DEVPROPTYPE Type,
//                                PVOID Data,
//          ULONG Size)
// {
//     static const UNICODE_STRING RootEnumerator = RTL_CONSTANT_STRING(L"ROOT\\");
//     UNICODE_STRING ValueName;
//     HANDLE ObjectKey;
//     NTSTATUS Status;
//
//     if ((Builtin->Flags & PIP_PROP_READ_ONLY) ||
//         (Builtin->Source != PipSourceKeyValue))
//     {
//         return STATUS_ACCESS_DENIED;
//     }
//
//     if ((Builtin->Flags & PIP_PROP_ROOT_ENUM_WRITE) &&
//         !RtlPrefixUnicodeString(&RootEnumerator, Object->Name, TRUE))
//     {
//         return STATUS_ACCESS_DENIED;
//     }
//
//     if ((Type != DEVPROP_TYPE_EMPTY) && (Type != Builtin->Type))
//         return STATUS_INVALID_PARAMETER;
//
//     Status = PiPropOpenObjectKey(Object, KEY_SET_VALUE, &ObjectKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     RtlInitUnicodeString(&ValueName, Builtin->ValueName);
//
//     if (Type == DEVPROP_TYPE_EMPTY)
//     {
//         Status = ZwDeleteValueKey(ObjectKey, &ValueName);
//     }
//     else if (Type == DEVPROP_TYPE_GUID)
//     {
//         UNICODE_STRING GuidString;
//         GUID Guid;
//
//         RtlCopyMemory(&Guid, Data, sizeof(Guid));
//         Status = RtlStringFromGUID(&Guid, &GuidString);
//         if (NT_SUCCESS(Status))
//         {
//             Status = ZwSetValueKey(ObjectKey,
//                                    &ValueName,
//                                    0,
//                                    REG_SZ,
//                                    GuidString.Buffer,
//                                    GuidString.Length + sizeof(UNICODE_NULL));
//             RtlFreeUnicodeString(&GuidString);
//         }
//     }
//     else
//     {
//         Status = ZwSetValueKey(ObjectKey, &ValueName, 0, Builtin->Selector, Data, Size);
//     }
//
//     ZwClose(ObjectKey);
//     return Status;
// }
//
// /* Reading *******************************************************************/
//
// typedef struct _PIP_PROP_OUTPUT
// {
//     PVOID Data;
//     ULONG Size;
//     ULONG Required;
//     DEVPROPTYPE Type;
// } PIP_PROP_OUTPUT, *PPIP_PROP_OUTPUT;
//
// /* Returns TRUE when the source has the final answer in *Status */
// typedef BOOLEAN
// (*PIP_PROP_READER)(
//          PPIP_PROP_OBJECT Object,
//          PPIP_PROP_NAME Name,
//             PPIP_PROP_OUTPUT Out,
//           PNTSTATUS Status);
//
// static
// BOOLEAN
// PiPropReadFromBuiltin(
//          PPIP_PROP_OBJECT Object,
//          PPIP_PROP_NAME Name,
//             PPIP_PROP_OUTPUT Out,
//           PNTSTATUS Status)
// {
//     const PIP_BUILTIN_PROPERTY *Builtin;
//     ULONG BlobSize;
//     PVOID Blob;
//
//     if (Name->Locale.Length)
//         return FALSE;
//
//     Builtin = PiPropFindBuiltin(Object, Name->Key);
//     if (!Builtin)
//         return FALSE;
//
//     *Status = PiPropReadBuiltin(Object, Builtin, &Blob, &BlobSize);
//     if (NT_SUCCESS(*Status))
//     {
//         *Status = PiPropCopyOut(Builtin->Type, Blob, BlobSize,
//                                 &Out->Type, Out->Data, Out->Size, &Out->Required);
//         ExFreePoolWithTag(Blob, TAG_PNP_PROPERTY);
//     }
//
//     return TRUE;
// }
//
// static
// BOOLEAN
// PiPropReadFromStore(
//          PPIP_PROP_OBJECT Object,
//          PPIP_PROP_NAME Name,
//             PPIP_PROP_OUTPUT Out,
//           PNTSTATUS Status)
// {
//     HANDLE ObjectKey;
//
//     /* Writers change one value at a time, and a key deleted under us reads as not found */
//     *Status = PiPropOpenObjectKey(Object, KEY_READ, &ObjectKey);
//     if (NT_SUCCESS(*Status))
//     {
//         *Status = PiPropStoreQuery(ObjectKey, Name, &Out->Type,
//                                    Out->Data, Out->Size, &Out->Required);
//         ZwClose(ObjectKey);
//     }
//
//     return (PiPropNormalizeStatus(*Status) != STATUS_OBJECT_NAME_NOT_FOUND);
// }
//
// static
// BOOLEAN
// PiPropReadFromSaved(
//          PPIP_PROP_OBJECT Object,
//          PPIP_PROP_NAME Name,
//             PPIP_PROP_OUTPUT Out,
//           PNTSTATUS Status)
// {
//     PPIP_PROP_SAVED_VALUE Saved;
//
//     if (!PiPropIsInterruptKey(Object, Name))
//         return FALSE;
//
//     KeEnterCriticalRegion();
//     ExAcquirePushLockShared(&PiPropSavedLock);
//
//     Saved = PiPropFindSaved(Object->Pdo);
//     if (Saved)
//     {
//         *Status = PiPropCopyOut(Saved->Type, Saved->Data, Saved->Size,
//                                 &Out->Type, Out->Data, Out->Size, &Out->Required);
//     }
//
//     ExReleasePushLockShared(&PiPropSavedLock);
//     KeLeaveCriticalRegion();
//
//     return (Saved != NULL);
// }
//
// static const PIP_PROP_READER PiPropReaders[] =
// {
//     PiPropReadFromBuiltin,
//     PiPropReadFromStore,
//     PiPropReadFromSaved
// };
//
// static
// NTSTATUS
// PiPropGet(
//          PPIP_PROP_OBJECT Object,
//          const DEVPROPKEY *PropertyKey,
//          LCID Lcid,
//          ULONG Size,
//                                                    PVOID Data,
//           PULONG RequiredSize,
//           PDEVPROPTYPE Type)
// {
//     PIP_PROP_OUTPUT Out = { Data, Data ? Size : 0, 0, DEVPROP_TYPE_EMPTY };
//     PIP_PROP_NAME Name;
//     NTSTATUS Status;
//     ULONG Index;
//
//     PAGED_CODE!();
//
//     if (!PropertyKey || !RequiredSize || !Type)
//         return STATUS_INVALID_PARAMETER;
//
//     Status = PiPropInitName(&Name, PropertyKey, Lcid);
//     if (NT_SUCCESS(Status))
//     {
//         Status = STATUS_OBJECT_NAME_NOT_FOUND;
//         for (Index = 0; Index < RTL_NUMBER_OF(PiPropReaders); Index++)
//         {
//             if (PiPropReaders[Index](Object, &Name, &Out, &Status))
//                 break;
//         }
//     }
//
//     *RequiredSize = Out.Required;
//     *Type = Out.Type;
//     return PiPropNormalizeStatus(Status);
// }
//
// /* Writing *******************************************************************/
//
// typedef enum _PIP_PROP_TARGET
// {
//     PipTargetDenied,
//     PipTargetBuiltin,
//     PipTargetStore
// } PIP_PROP_TARGET;
//
// static
// PIP_PROP_TARGET
// PiPropPickTarget(
//          PPIP_PROP_OBJECT Object,
//          PPIP_PROP_NAME Name,
//           const PIP_BUILTIN_PROPERTY **Builtin)
// {
//     *Builtin = NULL;
//
//     /* Localized values are still allowed on a protected object */
//     if ((Object->Flags & PIP_PROP_OBJECT_PROTECTED) && !Name->Locale.Length)
//         return PipTargetDenied;
//
//     if (!Name->Locale.Length)
//         *Builtin = PiPropFindBuiltin(Object, Name->Key);
//
//     return *Builtin ? PipTargetBuiltin : PipTargetStore;
// }
//
// static
// NTSTATUS
// PiPropStoreApply(
//          PPIP_PROP_OBJECT Object,
//          PPIP_PROP_NAME Name,
//          DEVPROPTYPE Type,
//                                PVOID Data,
//          ULONG Size)
// {
//     BOOLEAN Replaced;
//     HANDLE ObjectKey;
//     NTSTATUS Status;
//
//     Status = PiPropOpenObjectKey(Object, KEY_READ | KEY_CREATE_SUB_KEY, &ObjectKey);
//     if (NT_SUCCESS(Status))
//     {
//         if (Type == DEVPROP_TYPE_EMPTY)
//             Status = PiPropStoreDelete(ObjectKey, Name);
//         else
//             Status = PiPropStoreWrite(ObjectKey, Name, Type, Data, Size);
//
//         ZwClose(ObjectKey);
//     }
//
//     if (!PiPropIsInterruptKey(Object, Name))
//         return Status;
//
//     if (NT_SUCCESS(Status))
//         return PiPropReplaceSaved(Object->Pdo, Type, Data, Size, NULL);
//
//     /* Deleting a value that only exists in the saved copy still succeeds */
//     if ((Type == DEVPROP_TYPE_EMPTY) &&
//         (PiPropNormalizeStatus(Status) == STATUS_OBJECT_NAME_NOT_FOUND))
//     {
//         PiPropReplaceSaved(Object->Pdo, Type, NULL, 0, &Replaced);
//         if (Replaced)
//             Status = STATUS_SUCCESS;
//     }
//
//     return Status;
// }
//
// static inline
// BOOLEAN
// PiPropShouldAnnounce(
//          PDEVICE_NODE DeviceNode)
// {
//     switch (DeviceNode->State)
//     {
//         case DeviceNodeUnspecified:
//         case DeviceNodeUninitialized:
//         case DeviceNodeInitialized:
//             return FALSE;
//
//         default:
//             return TRUE;
//     }
// }
//
// static
// NTSTATUS
// PiPropSet(
//          PPIP_PROP_OBJECT Object,
//          const DEVPROPKEY *PropertyKey,
//          LCID Lcid,
//          DEVPROPTYPE Type,
//          ULONG Size,
//                                PVOID Data)
// {
//     const PIP_BUILTIN_PROPERTY *Builtin;
//     PIP_PROP_TARGET Target;
//     PIP_PROP_NAME Name;
//     NTSTATUS Status;
//
//     PAGED_CODE!();
//
//     if (!PropertyKey)
//         return STATUS_INVALID_PARAMETER;
//
//     Status = PiPropCheckPayload(Type, Data, Size);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = PiPropInitName(&Name, PropertyKey, Lcid);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if ((Type == DEVPROP_TYPE_STRING_INDIRECT) && Name.Locale.Length)
//         return STATUS_INVALID_PARAMETER;
//
//     Target = PiPropPickTarget(Object, &Name, &Builtin);
//     if (Target == PipTargetDenied)
//         return STATUS_ACCESS_DENIED;
//
//     KeEnterCriticalRegion();
//     ExAcquireResourceExclusiveLite(&PpRegistryDeviceResource, TRUE);
//
//     if (Target == PipTargetBuiltin)
//         Status = PiPropWriteBuiltin(Object, Builtin, Type, Data, Size);
//     else
//         Status = PiPropStoreApply(Object, &Name, Type, Data, Size);
//
//     ExReleaseResourceLite(&PpRegistryDeviceResource);
//     KeLeaveCriticalRegion();
//
//     if (Object->DeviceNode && PiPropShouldAnnounce(Object->DeviceNode))
//         IopQueueDevicePropertyChangeEvent(&Object->DeviceNode->InstancePath);
//
//     return PiPropNormalizeStatus(Status);
// }
//
// /* Objects *******************************************************************/
//
// static
// NTSTATUS
// PiPropInitDeviceObject(
//          PDEVICE_OBJECT Pdo,
//           PPIP_PROP_OBJECT Object)
// {
//     PDEVICE_NODE DeviceNode = Pdo ? IopGetDeviceNode(Pdo) : NULL;
//
//     if (!DeviceNode || (DeviceNode->Flags & DNF_LEGACY_RESOURCE_DEVICENODE))
//     {
//         DPRINT1("%p is not a PDO\n", Pdo);
//         KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2, (ULONG_PTR)Pdo, 0, 0);
//     }
//
//     if (!DeviceNode->InstancePath.Length)
//         return STATUS_INVALID_DEVICE_REQUEST;
//
//     RtlZeroMemory(Object, sizeof(*Object));
//     Object->Kind = PipPropObjectDevice;
//     Object->Pdo = Pdo;
//     Object->DeviceNode = DeviceNode;
//     Object->Name = &DeviceNode->InstancePath;
//
//     if (DeviceNode == IopRootDeviceNode)
//         Object->Flags |= PIP_PROP_OBJECT_PROTECTED;
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// PiPropInitInterfaceObject(
//          PCUNICODE_STRING SymbolicLinkName,
//           PPIP_PROP_OBJECT Object)
// {
//     RtlZeroMemory(Object, sizeof(*Object));
//
//     if (!SymbolicLinkName || !SymbolicLinkName->Buffer || !SymbolicLinkName->Length ||
//         !PiPropParseInterfaceName(SymbolicLinkName, &Object->ClassGuid))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Object->Kind = PipPropObjectInterface;
//     Object->Name = SymbolicLinkName;
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("PAGE")
// VOID
// PiPropReleaseDevice(
//          PDEVICE_OBJECT DeviceObject)
// {
//     PAGED_CODE!();
//
//     PiPropReplaceSaved(DeviceObject, DEVPROP_TYPE_EMPTY, NULL, 0, NULL);
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// CODE_SEG("PAGE")
// _IRQL_requires_max_(PASSIVE_LEVEL)
//
// NTSTATUS
// NTAPI
// IoSetDevicePropertyData(
//          PDEVICE_OBJECT Pdo,
//          const DEVPROPKEY *PropertyKey,
//          LCID Lcid,
//          ULONG Flags,
//          DEVPROPTYPE Type,
//          ULONG Size,
//                                PVOID Data)
// {
//     PIP_PROP_OBJECT Object;
//     NTSTATUS Status;
//
//     PAGED_CODE!();
//
//     let _ = Flags; // UNREFERENCED_PARAMETER
//
//     Status = PiPropInitDeviceObject(Pdo, &Object);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* A NULL buffer deletes the property */
//     if (!Data)
//     {
//         Type = DEVPROP_TYPE_EMPTY;
//         Size = 0;
//     }
//
//     return PiPropSet(&Object, PropertyKey, Lcid, Type, Size, Data);
// }
//
// /*
//  * @implemented
//  */
// CODE_SEG("PAGE")
// _IRQL_requires_max_(PASSIVE_LEVEL)
//
// NTSTATUS
// NTAPI
// IoGetDevicePropertyData(
//          PDEVICE_OBJECT Pdo,
//          const DEVPROPKEY *PropertyKey,
//          LCID Lcid,
//                ULONG Flags,
//          ULONG Size,
//                                                    PVOID Data,
//           PULONG RequiredSize,
//           PDEVPROPTYPE Type)
// {
//     PIP_PROP_OBJECT Object;
//     NTSTATUS Status;
//
//     PAGED_CODE!();
//
//     let _ = Flags; // UNREFERENCED_PARAMETER
//
//     Status = PiPropInitDeviceObject(Pdo, &Object);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     return PiPropGet(&Object, PropertyKey, Lcid, Size, Data, RequiredSize, Type);
// }
//
// /*
//  * @implemented
//  */
// CODE_SEG("PAGE")
// _IRQL_requires_max_(PASSIVE_LEVEL)
//
// NTSTATUS
// NTAPI
// IoSetDeviceInterfacePropertyData(
//          PUNICODE_STRING SymbolicLinkName,
//          const DEVPROPKEY *PropertyKey,
//          LCID Lcid,
//          ULONG Flags,
//          DEVPROPTYPE Type,
//          ULONG Size,
//                                PVOID Data)
// {
//     PIP_PROP_OBJECT Object;
//     NTSTATUS Status;
//
//     PAGED_CODE!();
//
//     let _ = Flags; // UNREFERENCED_PARAMETER
//
//     Status = PiPropInitInterfaceObject(SymbolicLinkName, &Object);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (!Data)
//     {
//         Type = DEVPROP_TYPE_EMPTY;
//         Size = 0;
//     }
//
//     return PiPropSet(&Object, PropertyKey, Lcid, Type, Size, Data);
// }
//
// /*
//  * @implemented
//  */
// CODE_SEG("PAGE")
// _IRQL_requires_max_(PASSIVE_LEVEL)
//
// NTSTATUS
// NTAPI
// IoGetDeviceInterfacePropertyData(
//          PUNICODE_STRING SymbolicLinkName,
//          const DEVPROPKEY *PropertyKey,
//          LCID Lcid,
//                ULONG Flags,
//          ULONG Size,
//                                                    PVOID Data,
//           PULONG RequiredSize,
//           PDEVPROPTYPE Type)
// {
//     PIP_PROP_OBJECT Object;
//     NTSTATUS Status;
//
//     PAGED_CODE!();
//
//     let _ = Flags; // UNREFERENCED_PARAMETER
//
//     Status = PiPropInitInterfaceObject(SymbolicLinkName, &Object);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     return PiPropGet(&Object, PropertyKey, Lcid, Size, Data, RequiredSize, Type);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
