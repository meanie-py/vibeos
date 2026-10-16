use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _EPROCESS, _ETHREAD, _OBJECT_HEADER, _OBJECT_TYPE,
    _UNICODE_STRING, _OBJECT_ATTRIBUTES, OBJECT_NAME_INFORMATION,
    PEPROCESS, PETHREAD, POBJECT_HEADER, POBJECT_TYPE,
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE,
    GUID, LPCGUID, LPGUID,
    PDEVICE_OBJECT, PEXTENDED_DEVOBJ_EXTENSION,
    PHANDLE, PUNICODE_STRING, PCUNICODE_STRING, POBJECT_NAME_INFORMATION,
    PKEY_BASIC_INFORMATION, PKEY_VALUE_FULL_INFORMATION, PKEY_VALUE_PARTIAL_INFORMATION,
    ObQueryNameString, RtlGUIDFromString, RtlStringFromGUID,
    RtlDuplicateUnicodeString, RtlEqualUnicodeString, RtlPrefixUnicodeString,
    RtlAppendUnicodeStringToString, RtlAppendUnicodeToString,
    RtlInitEmptyUnicodeString, RtlInitUnicodeString, RtlFreeUnicodeString,
    RtlCompareMemory,
    ExAllocatePool, ExFreePool,
    ZwClose, ZwCreateKey, ZwDeleteKey, ZwEnumerateKey, ZwOpenKey, ZwQueryValueKey, ZwSetValueKey,
    IopCreateRegistryKeyEx, IopGetRegistryValue, IopOpenRegistryKeyEx,
    IopQueueDeviceChangeEvent,
    IoCreateSymbolicLink, IoDeleteSymbolicLink,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const TAG_IO: ULONG = 0x20206F49; // ' oI '
const REG_OPTION_NON_VOLATILE: ULONG = 0x00000000;
const REG_OPTION_VOLATILE: ULONG = 0x00000001;
const KEY_WRITE: ULONG = STANDARD_RIGHTS_WRITE | KEY_SET_VALUE | KEY_CREATE_SUB_KEY;
const MAXUSHORT: ULONG = 0xFFFF;
const DEVICE_INTERFACE_INCLUDE_NONACTIVE: ULONG = 0x00000001;

extern "C" {
    fn InitializeObjectAttributes( p: *mut _OBJECT_ATTRIBUTES, n: *mut _UNICODE_STRING, a: ULONG, r: HANDLE, s: PVOID, );
    fn IopGetDeviceObjectFromDeviceInstance(DeviceInstance: PUNICODE_STRING) -> PDEVICE_OBJECT;
    fn PiNotifyDeviceInterfaceChange( EventGuid: LPCGUID, InterfaceClassGuid: LPCGUID, SymbolicLinkName: PUNICODE_STRING, );
    static mut GUID_DEVICE_INTERFACE_ARRIVAL: GUID;
    static mut GUID_DEVICE_INTERFACE_REMOVAL: GUID;
}

extern "system" {
    fn PnpRegSzToString(Buffer: PVOID, DataLength: ULONG, Length: *mut USHORT);
    #[link_name = "RtlUnicodeStringPrintf_4wZ_shim"]
    fn RtlUnicodeStringPrintf_4wZ( DestinationString: PUNICODE_STRING, Arg1: PUNICODE_STRING, Arg2: PUNICODE_STRING, Arg3: PUNICODE_STRING, Arg4: PUNICODE_STRING ) -> NTSTATUS;
}

// "System\\CurrentControlSet\\Control\\DeviceClasses" as UTF-16
static REGSTR_PATH_DEVICE_CLASSES: [u16; 50] = [
    b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
    b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
    b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16,
    b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
    b'\\' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16,
    b'o' as u16, b'l' as u16, b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16,
    b'i' as u16, b'c' as u16, b'e' as u16, b'C' as u16, b'l' as u16, b'a' as u16,
    b's' as u16, b's' as u16, b'e' as u16, b's' as u16, 0, 0, 0, 0,
];

const GUID_STRING_CHARS: usize = 38;
const GUID_STRING_BYTES: usize = GUID_STRING_CHARS * core::mem::size_of::<u16>();

static BASE_KEY_STRING: &[u16] = &[
    b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16,
    b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16,
    b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'\\' as u16,
    b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16,
    b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16,
    b'S' as u16, b'e' as u16, b't' as u16, b'\\' as u16,
    b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'\\' as u16,
    b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
    b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16, b'e' as u16, b's' as u16, b'\\' as u16,
    0
];

// L"\\??\\" — 4 wide chars, no null needed (Length == MaximumLength)
static KERNEL_MODE_PREFIX_BUF: [u16; 4] = [0x5C, 0x3F, 0x3F, 0x5C]; // \??\

static mut KERNEL_MODE_PREFIX: _UNICODE_STRING = _UNICODE_STRING {
    Length: 8,
    MaximumLength: 8,
    Buffer: KERNEL_MODE_PREFIX_BUF.as_ptr() as *mut u16,
};

// L"\\\\?\\" — 4 wide chars
static USER_MODE_PREFIX_BUF: [u16; 4] = [0x5C, 0x5C, 0x3F, 0x5C]; // \\?\

static mut USER_MODE_PREFIX: _UNICODE_STRING = _UNICODE_STRING {
    Length: 8,
    MaximumLength: 8,
    Buffer: USER_MODE_PREFIX_BUF.as_ptr() as *mut u16,
};

// L"\\"
static PATH_SEP_BUF: [u16; 1] = [0x5C]; // backslash

static mut PATH_SEP: _UNICODE_STRING = _UNICODE_STRING {
    Length: 2,
    MaximumLength: 2,
    Buffer: PATH_SEP_BUF.as_ptr() as *mut u16,
};

unsafe fn IopBuildSymbolicLink(
    DeviceString: PCUNICODE_STRING,
    GuidString: PCUNICODE_STRING,
    ReferenceString: PCUNICODE_STRING,
    UserModePrefixFormat: BOOLEAN,
    SymbolicLinkName: PUNICODE_STRING,
) -> NTSTATUS {
    let ReferenceStringReal = if ReferenceString.is_null() {
        core::ptr::addr_of!(PATH_SEP)
    } else {
        ReferenceString
    };

    let mut MungedDeviceString: _UNICODE_STRING = core::mem::zeroed();
    let Status = RtlDuplicateUnicodeString(0, DeviceString, &mut MungedDeviceString);
    if Status < 0 {
        return Status;
    }

    for i in 0..(MungedDeviceString.Length / 2) as usize {
        if *MungedDeviceString.Buffer.add(i) == b'\\' as u16 {
            *MungedDeviceString.Buffer.add(i) = b'#' as u16;
        }
    }

    let Length = (*core::ptr::addr_of!(KERNEL_MODE_PREFIX)).Length
        + MungedDeviceString.Length
        + 4  // sizeof(L"#") = 4 bytes (2 for '#' + 2 for null)
        + (*GuidString).Length
        + (*ReferenceStringReal).Length;

    let mut SymbolicLink: _UNICODE_STRING = core::mem::zeroed();
    SymbolicLink.Length = 0;
    SymbolicLink.MaximumLength = Length;
    SymbolicLink.Buffer = ExAllocatePoolWithTag(PagedPool, SymbolicLink.MaximumLength as _, TAG_IO) as *mut u16;
    if SymbolicLink.Buffer.is_null() {
        RtlFreeUnicodeString(&mut MungedDeviceString);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let prefix = if UserModePrefixFormat != 0 {
        &raw const USER_MODE_PREFIX
    } else {
        &raw const KERNEL_MODE_PREFIX
    };

    let Status = RtlUnicodeStringPrintf_4wZ(
        &mut SymbolicLink,
        prefix as *const _ as PUNICODE_STRING,
        &MungedDeviceString as *const _ as PUNICODE_STRING,
        GuidString as PUNICODE_STRING,
        ReferenceStringReal as PUNICODE_STRING,
    );

    RtlFreeUnicodeString(&mut MungedDeviceString);

    if Status < 0 {
        if !SymbolicLink.Buffer.is_null() {
            ExFreePoolWithTag(SymbolicLink.Buffer as PVOID, TAG_IO);
        }
        return Status;
    }

    *SymbolicLinkName = SymbolicLink;
    STATUS_SUCCESS
}

unsafe fn IopSeparateSymbolicLink(
    SymbolicLinkName: PCUNICODE_STRING,
    PrefixString: PUNICODE_STRING,
    MungedString: PUNICODE_STRING,
    GuidString: PUNICODE_STRING,
    ReferenceString: PUNICODE_STRING,
    ReferenceStringPresent: PBOOLEAN,
    InterfaceClassGuid: LPGUID,
) -> NTSTATUS {
    if RtlPrefixUnicodeString(&raw const KERNEL_MODE_PREFIX, SymbolicLinkName, 0u8) == 0 &&
       RtlPrefixUnicodeString(&raw const USER_MODE_PREFIX, SymbolicLinkName, 0u8) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    let mut LinkNameNoPrefix: _UNICODE_STRING = core::mem::zeroed();
    LinkNameNoPrefix.Buffer = (*SymbolicLinkName).Buffer.add((KERNEL_MODE_PREFIX.Length / 2) as usize);
    LinkNameNoPrefix.Length = (*SymbolicLinkName).Length - KERNEL_MODE_PREFIX.Length;
    LinkNameNoPrefix.MaximumLength = LinkNameNoPrefix.Length;

    let mut ReferenceStringOffset = 0u16;
    for i in 0..(LinkNameNoPrefix.Length / 2) {
        if *LinkNameNoPrefix.Buffer.add(i as usize) == b'\\' as u16 {
            ReferenceStringOffset = i * 2;
            break;
        }
    }
    if ReferenceStringOffset == 0 {
        ReferenceStringOffset = LinkNameNoPrefix.Length;
    }

    if ReferenceStringOffset < (GUID_STRING_BYTES + 2) as u16 {
        return STATUS_INVALID_PARAMETER;
    }

    let mut ReferenceStringReal: _UNICODE_STRING = core::mem::zeroed();
    ReferenceStringReal.Buffer = LinkNameNoPrefix.Buffer.add((ReferenceStringOffset / 2) as usize);
    ReferenceStringReal.Length = LinkNameNoPrefix.Length - ReferenceStringOffset;
    ReferenceStringReal.MaximumLength = ReferenceStringReal.Length;

    let mut GuidStringReal: _UNICODE_STRING = core::mem::zeroed();
    GuidStringReal.Buffer = LinkNameNoPrefix.Buffer.add(((ReferenceStringOffset - GUID_STRING_BYTES as u16) / 2) as usize);
    GuidStringReal.Length = GUID_STRING_BYTES as u16;
    GuidStringReal.MaximumLength = GuidStringReal.Length;

    if !PrefixString.is_null() {
        (*PrefixString).Buffer = (*SymbolicLinkName).Buffer;
        (*PrefixString).Length = KERNEL_MODE_PREFIX.Length;
        (*PrefixString).MaximumLength = (*PrefixString).Length;
    }

    if !MungedString.is_null() {
        let mut MungedStringReal: _UNICODE_STRING = core::mem::zeroed();
        MungedStringReal.Buffer = LinkNameNoPrefix.Buffer;
        MungedStringReal.Length = LinkNameNoPrefix.Length - ReferenceStringReal.Length - GUID_STRING_BYTES as u16 - 2;
        MungedStringReal.MaximumLength = MungedStringReal.Length;
        *MungedString = MungedStringReal;
    }

    if !GuidString.is_null() {
        *GuidString = GuidStringReal;
    }

    if !ReferenceString.is_null() {
        if ReferenceStringReal.Length > 2 {
            *ReferenceString = ReferenceStringReal;
        } else {
            RtlInitEmptyUnicodeString(ReferenceString, core::ptr::null_mut(), 0);
        }
    }

    if !ReferenceStringPresent.is_null() {
        *ReferenceStringPresent = if ReferenceStringReal.Length > 2 { 1 } else { 0 };
    }

    if !InterfaceClassGuid.is_null() {
        let Status = RtlGUIDFromString(&mut GuidStringReal, InterfaceClassGuid);
        if Status < 0 {
            return Status;
        }
    }

    STATUS_SUCCESS
}

unsafe fn IopOpenOrCreateSymbolicLinkSubKeys(
    DeviceHandle: PHANDLE,
    DeviceDisposition: *mut ULONG,
    InstanceHandle: PHANDLE,
    InstanceDisposition: *mut ULONG,
    ClassHandle: HANDLE,
    SymbolicLinkName: PCUNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    Create: BOOLEAN,
) -> NTSTATUS {
    let mut ReferenceString: _UNICODE_STRING = core::mem::zeroed();
    let mut SymbolicLink: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceKeyHandle = core::ptr::null_mut();
    let mut InstanceKeyHandle = core::ptr::null_mut();
    let mut DeviceKeyDisposition = 0u32;
    let mut InstanceKeyDisposition = 0u32;
    let mut ReferenceStringPresent = 0u8;

    let mut Status = RtlDuplicateUnicodeString(0, SymbolicLinkName, &mut SymbolicLink);
    if Status < 0 {
        return Status;
    }

    Status = IopSeparateSymbolicLink(
        &SymbolicLink,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut ReferenceString,
        &mut ReferenceStringPresent,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        RtlFreeUnicodeString(&mut SymbolicLink);
        return Status;
    }

    if ReferenceStringPresent != 0 {
        SymbolicLink.MaximumLength = SymbolicLink.Length - ReferenceString.Length;
        SymbolicLink.Length = SymbolicLink.MaximumLength;
        *ReferenceString.Buffer = b'#' as u16;
    } else {
        RtlInitUnicodeString(&mut ReferenceString, [0x23u16, 0x00].as_ptr()); // L"#"
    }

    for i in 0..(SymbolicLink.Length / 2) as usize {
        if *SymbolicLink.Buffer.add(i) == b'\\' as u16 {
            *SymbolicLink.Buffer.add(i) = b'#' as u16;
        }
    }

    *SymbolicLink.Buffer.add(1) = b'#' as u16;

    if Create != 0 {
        Status = IopCreateRegistryKeyEx(
            &mut DeviceKeyHandle,
            ClassHandle,
            &mut SymbolicLink,
            DesiredAccess | KEY_ENUMERATE_SUB_KEYS,
            REG_OPTION_NON_VOLATILE,
            &mut DeviceKeyDisposition,
        );
    } else {
        Status = IopOpenRegistryKeyEx(
            &mut DeviceKeyHandle,
            ClassHandle,
            &mut SymbolicLink,
            DesiredAccess | KEY_ENUMERATE_SUB_KEYS,
        );
    }

    if Status < 0 {
        RtlFreeUnicodeString(&mut SymbolicLink);
        return Status;
    }

    if Create != 0 {
        Status = IopCreateRegistryKeyEx(
            &mut InstanceKeyHandle,
            DeviceKeyHandle,
            &mut ReferenceString,
            DesiredAccess,
            REG_OPTION_NON_VOLATILE,
            &mut InstanceKeyDisposition,
        );
    } else {
        Status = IopOpenRegistryKeyEx(
            &mut InstanceKeyHandle,
            DeviceKeyHandle,
            &mut ReferenceString,
            DesiredAccess,
        );
    }

    if Status < 0 {
        if !DeviceKeyHandle.is_null() {
            ZwClose(DeviceKeyHandle);
        }
        RtlFreeUnicodeString(&mut SymbolicLink);
        return Status;
    }

    if !DeviceHandle.is_null() {
        *DeviceHandle = DeviceKeyHandle;
    } else {
        ZwClose(DeviceKeyHandle);
    }

    if !DeviceDisposition.is_null() {
        *DeviceDisposition = DeviceKeyDisposition;
    }

    if !InstanceHandle.is_null() {
        *InstanceHandle = InstanceKeyHandle;
    } else {
        ZwClose(InstanceKeyHandle);
    }

    if !InstanceDisposition.is_null() {
        *InstanceDisposition = InstanceKeyDisposition;
    }

    RtlFreeUnicodeString(&mut SymbolicLink);
    STATUS_SUCCESS
}

/// The device and instance keys of a device interface, for the property store (pnpprop,
/// upstream 3cc9aa99292).
#[no_mangle]
pub unsafe extern "C" fn IopOpenDeviceInterfaceKeys(
    SymbolicLinkName: PCUNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    DeviceKey: PHANDLE,
    InstanceKey: PHANDLE,
) -> NTSTATUS {
    OpenRegistryHandlesFromSymbolicLink(
        SymbolicLinkName,
        DesiredAccess,
        core::ptr::null_mut(),
        DeviceKey,
        InstanceKey,
    )
}

unsafe fn OpenRegistryHandlesFromSymbolicLink(
    SymbolicLinkName: PCUNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    GuidKey: PHANDLE,
    DeviceKey: PHANDLE,
    InstanceKey: PHANDLE,
) -> NTSTATUS {
    let mut BaseKeyU: _UNICODE_STRING = core::mem::zeroed();
    let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
    let mut ClassesKey = core::ptr::null_mut();
    let mut GuidKeyReal = core::ptr::null_mut();
    let mut DeviceKeyReal = core::ptr::null_mut();
    let mut InstanceKeyReal = core::ptr::null_mut();

    RtlInitUnicodeString(&mut BaseKeyU, BASE_KEY_STRING.as_ptr());

    let mut Status = IopSeparateSymbolicLink(
        SymbolicLinkName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut GuidString,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    Status = IopOpenRegistryKeyEx(
        &mut ClassesKey,
        core::ptr::null_mut(),
        &mut BaseKeyU,
        DesiredAccess | KEY_ENUMERATE_SUB_KEYS,
    );
    if Status < 0 {
        return Status;
    }

    Status = IopOpenRegistryKeyEx(
        &mut GuidKeyReal,
        ClassesKey,
        &mut GuidString,
        DesiredAccess | KEY_ENUMERATE_SUB_KEYS,
    );
    if Status < 0 {
        ZwClose(ClassesKey);
        return Status;
    }

    Status = IopOpenOrCreateSymbolicLinkSubKeys(
        &mut DeviceKeyReal,
        core::ptr::null_mut(),
        &mut InstanceKeyReal,
        core::ptr::null_mut(),
        GuidKeyReal,
        SymbolicLinkName,
        DesiredAccess,
        0u8,
    );
    if Status < 0 {
        ZwClose(GuidKeyReal);
        ZwClose(ClassesKey);
        return Status;
    }

    if !GuidKey.is_null() {
        *GuidKey = GuidKeyReal;
    } else {
        ZwClose(GuidKeyReal);
    }

    if !DeviceKey.is_null() {
        *DeviceKey = DeviceKeyReal;
    } else {
        ZwClose(DeviceKeyReal);
    }

    if !InstanceKey.is_null() {
        *InstanceKey = InstanceKeyReal;
    } else {
        ZwClose(InstanceKeyReal);
    }

    ZwClose(ClassesKey);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoOpenDeviceInterfaceRegistryKey(
    SymbolicLinkName: PUNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    DeviceInterfaceKey: PHANDLE,
) -> NTSTATUS {
    let SymbolicLinkName = opaque_ptr(SymbolicLinkName);
    let DeviceInterfaceKey = opaque_ptr(DeviceInterfaceKey);
    let mut InstanceKey = core::ptr::null_mut();
    let mut DeviceParametersKey = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DeviceParametersU: _UNICODE_STRING = core::mem::zeroed();

    // L"Device Parameters"
    const DEVICE_PARAMETERS_W: [u16; 18] = [
        b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
        b' ' as u16, b'P' as u16, b'a' as u16, b'r' as u16, b'a' as u16, b'm' as u16,
        b'e' as u16, b't' as u16, b'e' as u16, b'r' as u16, b's' as u16, 0,
    ];
    RtlInitUnicodeString(&mut DeviceParametersU, DEVICE_PARAMETERS_W.as_ptr());

    let Status = OpenRegistryHandlesFromSymbolicLink(
        SymbolicLinkName,
        KEY_CREATE_SUB_KEY,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut InstanceKey,
    );
    if Status < 0 {
        return Status;
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut DeviceParametersU,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE | OBJ_OPENIF,
        InstanceKey,
        core::ptr::null_mut(),
    );

    let Status = ZwCreateKey(
        &mut DeviceParametersKey,
        DesiredAccess,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        core::ptr::null_mut(),
    );
    ZwClose(InstanceKey);

    if Status >= 0 {
        *DeviceInterfaceKey = DeviceParametersKey;
    }

    Status
}

unsafe fn IopOpenInterfaceKey(
    InterfaceClassGuid: LPCGUID,
    DesiredAccess: ACCESS_MASK,
    pInterfaceKey: PHANDLE,
) -> NTSTATUS {
    // L"\\Registry\\Machine\\" — 18 wide chars, Length = 18*2 = 36
    const LOCAL_MACHINE_W: [u16; 19] = [
        0x5C, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16,
        b't' as u16, b'r' as u16, b'y' as u16, 0x5C, b'M' as u16, b'a' as u16,
        b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, 0x5C, 0,
    ];
    let mut LocalMachine: _UNICODE_STRING = _UNICODE_STRING {
        Length: 36,
        MaximumLength: 38, // includes null terminator
        Buffer: LOCAL_MACHINE_W.as_ptr() as *mut u16,
    };
    let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut InterfaceKey = core::ptr::null_mut();

    let mut Status = RtlStringFromGUID(InterfaceClassGuid, &mut GuidString);
    if Status < 0 {
        return Status;
    }

    let regstr_len = {
        let mut len = 0;
        while *(core::ptr::addr_of!(REGSTR_PATH_DEVICE_CLASSES) as *const u16).add(len) != 0 {
            len += 1;
        }
        len
    };

    KeyName.Length = 0;
    KeyName.MaximumLength = LocalMachine.Length + ((regstr_len + 1) * 2) as u16 + GuidString.Length;
    KeyName.Buffer = ExAllocatePool(PagedPool, KeyName.MaximumLength as _) as *mut u16;
    if KeyName.Buffer.is_null() {
        RtlFreeUnicodeString(&mut GuidString);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = RtlAppendUnicodeStringToString(&mut KeyName, &LocalMachine);
    if Status < 0 {
        ExFreePool(KeyName.Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return Status;
    }

    Status = RtlAppendUnicodeToString(&mut KeyName, core::ptr::addr_of!(REGSTR_PATH_DEVICE_CLASSES) as PCWSTR);
    if Status < 0 {
        ExFreePool(KeyName.Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return Status;
    }

    Status = RtlAppendUnicodeToString(&mut KeyName, [0x5Cu16, 0x00].as_ptr()); // L"\\"
    if Status < 0 {
        ExFreePool(KeyName.Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return Status;
    }

    Status = RtlAppendUnicodeStringToString(&mut KeyName, &GuidString);
    if Status < 0 {
        ExFreePool(KeyName.Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return Status;
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = ZwOpenKey(&mut InterfaceKey, DesiredAccess, &mut ObjectAttributes);

    ExFreePool(KeyName.Buffer as PVOID);
    RtlFreeUnicodeString(&mut GuidString);

    if Status < 0 {
        return Status;
    }

    *pInterfaceKey = InterfaceKey;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoGetDeviceInterfaceAlias(
    SymbolicLinkName: PUNICODE_STRING,
    AliasInterfaceClassGuid: LPCGUID,
    AliasSymbolicLinkName: PUNICODE_STRING,
) -> NTSTATUS {
    let SymbolicLinkName = opaque_ptr(SymbolicLinkName);
    let AliasSymbolicLinkName = opaque_ptr(AliasSymbolicLinkName);
    let mut AliasSymbolicLink: _UNICODE_STRING = core::mem::zeroed();
    let mut AliasGuidString: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceString: _UNICODE_STRING = core::mem::zeroed();
    let mut ReferenceString: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceKey = core::ptr::null_mut();
    let mut AliasInstanceKey = core::ptr::null_mut();
    let mut ReferenceStringPresent = 0u8;
    let mut kvInfo: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();

    if SymbolicLinkName.is_null() || AliasInterfaceClassGuid.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let mut Status = RtlStringFromGUID(AliasInterfaceClassGuid, &mut AliasGuidString);
    if Status < 0 {
        return Status;
    }

    Status = OpenRegistryHandlesFromSymbolicLink(
        SymbolicLinkName,
        KEY_QUERY_VALUE,
        core::ptr::null_mut(),
        &mut DeviceKey,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        RtlFreeUnicodeString(&mut AliasGuidString);
        return Status;
    }

    // L"DeviceInstance"
    const DEVICE_INSTANCE_W: [u16; 15] = [
        b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
        b'I' as u16, b'n' as u16, b's' as u16, b't' as u16, b'a' as u16, b'n' as u16,
        b'c' as u16, b'e' as u16, 0,
    ];
    Status = IopGetRegistryValue(DeviceKey, DEVICE_INSTANCE_W.as_ptr() as PWSTR, &mut kvInfo);
    ZwClose(DeviceKey);
    if Status < 0 {
        RtlFreeUnicodeString(&mut AliasGuidString);
        return Status;
    }

    if (*kvInfo).Type != REG_SZ || (*kvInfo).DataLength == 0 || (*kvInfo).DataLength > MAXUSHORT as u32 {
        RtlFreeUnicodeString(&mut AliasGuidString);
        return STATUS_INVALID_PARAMETER;
    }

    let Buffer = (kvInfo as ULONG_PTR + (*kvInfo).DataOffset as ULONG_PTR) as PVOID;
    PnpRegSzToString(Buffer, (*kvInfo).DataLength, &mut DeviceString.Length);
    DeviceString.MaximumLength = DeviceString.Length;
    DeviceString.Buffer = Buffer as *mut u16;

    Status = IopSeparateSymbolicLink(
        SymbolicLinkName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut ReferenceString,
        &mut ReferenceStringPresent,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        RtlFreeUnicodeString(&mut AliasGuidString);
        return Status;
    }

    let UserModePrefixFormat = RtlPrefixUnicodeString(&raw const USER_MODE_PREFIX, SymbolicLinkName, 0u8);

    Status = IopBuildSymbolicLink(
        &DeviceString,
        &AliasGuidString,
        if ReferenceStringPresent != 0 { &ReferenceString } else { core::ptr::null() },
        UserModePrefixFormat,
        &mut AliasSymbolicLink,
    );
    if Status < 0 {
        RtlFreeUnicodeString(&mut AliasGuidString);
        return Status;
    }

    Status = OpenRegistryHandlesFromSymbolicLink(
        &AliasSymbolicLink,
        KEY_READ,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut AliasInstanceKey,
    );
    if Status < 0 {
        RtlFreeUnicodeString(&mut AliasSymbolicLink);
        RtlFreeUnicodeString(&mut AliasGuidString);
        return Status;
    }
    ZwClose(AliasInstanceKey);

    *AliasSymbolicLinkName = AliasSymbolicLink;
    RtlFreeUnicodeString(&mut AliasGuidString);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoGetDeviceInterfaces(
    InterfaceClassGuid: LPCGUID,
    PhysicalDeviceObject: PDEVICE_OBJECT,
    Flags: ULONG,
    SymbolicLinkList: *mut PWSTR,
) -> NTSTATUS {
    let PhysicalDeviceObject = opaque_ptr(PhysicalDeviceObject);
    let SymbolicLinkList = opaque_ptr(SymbolicLinkList);
    // L"Control" — 7 chars, Length = 14
    const CONTROL_W: [u16; 8] = [
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16,
        b'r' as u16, b'o' as u16, b'l' as u16, 0,
    ];
    let mut Control: _UNICODE_STRING = _UNICODE_STRING {
        Length: 14,
        MaximumLength: 16,
        Buffer: CONTROL_W.as_ptr() as *mut u16,
    };
    // L"SymbolicLink" — 12 chars, Length = 24
    const SYMBOLIC_LINK_W: [u16; 13] = [
        b'S' as u16, b'y' as u16, b'm' as u16, b'b' as u16, b'o' as u16, b'l' as u16,
        b'i' as u16, b'c' as u16, b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16, 0,
    ];
    let mut SymbolicLink: _UNICODE_STRING = _UNICODE_STRING {
        Length: 24,
        MaximumLength: 26,
        Buffer: SYMBOLIC_LINK_W.as_ptr() as *mut u16,
    };
    let mut InterfaceKey = core::ptr::null_mut();
    let mut DeviceKey = core::ptr::null_mut();
    let mut ReferenceKey = core::ptr::null_mut();
    let mut ControlKey = core::ptr::null_mut();
    let mut DeviceBi: PKEY_BASIC_INFORMATION = core::ptr::null_mut();
    let mut ReferenceBi: PKEY_BASIC_INFORMATION = core::ptr::null_mut();
    let mut bip: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut InstanceDevicePath: PUNICODE_STRING = core::ptr::null_mut();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FoundRightPDO = 0u8;
    let mut i = 0u32;
    let mut ReturnBuffer: _UNICODE_STRING = core::mem::zeroed();

    if !PhysicalDeviceObject.is_null() {
        let DeviceObjectExtension = (*PhysicalDeviceObject).DeviceObjectExtension as PEXTENDED_DEVOBJ_EXTENSION;

        if (*DeviceObjectExtension).DeviceNode.is_null() {
            return STATUS_INVALID_DEVICE_REQUEST;
        }

        if (*(*DeviceObjectExtension).DeviceNode).InstancePath.Length == 0 {
            return STATUS_INVALID_DEVICE_REQUEST;
        }

        InstanceDevicePath = core::ptr::addr_of_mut!((*(*DeviceObjectExtension).DeviceNode).InstancePath);
    }

    let mut Status = IopOpenInterfaceKey(InterfaceClassGuid, KEY_ENUMERATE_SUB_KEYS, &mut InterfaceKey);
    if Status < 0 {
        return Status;
    }

    loop {
        let mut Size = 0u32;
        Status = ZwEnumerateKey(InterfaceKey, i, 0, core::ptr::null_mut(), 0, &mut Size);
        if Status == STATUS_NO_MORE_ENTRIES {
            break;
        } else if Status < 0 && Status != STATUS_BUFFER_TOO_SMALL {
            break;
        }

        DeviceBi = ExAllocatePool(PagedPool, Size as _) as PKEY_BASIC_INFORMATION;
        if DeviceBi.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        Status = ZwEnumerateKey(InterfaceKey, i, 0, DeviceBi as PVOID, Size, &mut Size);
        i += 1;
        if Status < 0 {
            break;
        }

        KeyName.Length = (*DeviceBi).NameLength as u16;
        KeyName.MaximumLength = KeyName.Length;
        KeyName.Buffer = (*DeviceBi).Name.as_mut_ptr();

        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut KeyName,
            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            InterfaceKey,
            core::ptr::null_mut(),
        );

        Status = ZwOpenKey(&mut DeviceKey, KEY_ENUMERATE_SUB_KEYS, &mut ObjectAttributes);
        if Status < 0 {
            break;
        }

        if !PhysicalDeviceObject.is_null() {
            let mut DeviceInstanceName: _UNICODE_STRING = core::mem::zeroed();
            // L"DeviceInstance"
            const DEV_INST_W: [u16; 15] = [
                b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
                b'I' as u16, b'n' as u16, b's' as u16, b't' as u16, b'a' as u16, b'n' as u16,
                b'c' as u16, b'e' as u16, 0,
            ];
            RtlInitUnicodeString(&mut DeviceInstanceName, DEV_INST_W.as_ptr());

            let mut NeededLength = 0u32;
            Status = ZwQueryValueKey(DeviceKey, &mut DeviceInstanceName, 2, core::ptr::null_mut(), 0, &mut NeededLength);

            if Status == STATUS_BUFFER_TOO_SMALL {
                let PartialInfo = ExAllocatePool(NonPagedPool, NeededLength as _) as PKEY_VALUE_PARTIAL_INFORMATION;
                if PartialInfo.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break;
                }

                Status = ZwQueryValueKey(DeviceKey, &mut DeviceInstanceName, 2, PartialInfo as PVOID, NeededLength, &mut NeededLength);
                if Status < 0 {
                    ExFreePool(PartialInfo as PVOID);
                    break;
                }

                if (*PartialInfo).DataLength == (*InstanceDevicePath).Length as u32 {
                    if RtlCompareMemory(
                        (*PartialInfo).Data.as_ptr() as *const _,
                        (*InstanceDevicePath).Buffer as *const _,
                        (*InstanceDevicePath).Length as _,
                    ) == (*InstanceDevicePath).Length as _
                    {
                        FoundRightPDO = 1;
                    }
                }
                ExFreePool(PartialInfo as PVOID);

                if FoundRightPDO == 0 {
                    continue;
                }
            } else {
                break;
            }
        }

        let mut j = 0u32;
        loop {
            let mut RefSize = 0u32;
            Status = ZwEnumerateKey(DeviceKey, j, 0, core::ptr::null_mut(), 0, &mut RefSize);
            if Status == STATUS_NO_MORE_ENTRIES {
                break;
            } else if Status < 0 && Status != STATUS_BUFFER_TOO_SMALL {
                break;
            }

            ReferenceBi = ExAllocatePool(PagedPool, RefSize as _) as PKEY_BASIC_INFORMATION;
            if ReferenceBi.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break;
            }

            Status = ZwEnumerateKey(DeviceKey, j, 0, ReferenceBi as PVOID, RefSize, &mut RefSize);
            j += 1;
            if Status < 0 {
                break;
            }

            KeyName.Length = (*ReferenceBi).NameLength as u16;
            KeyName.MaximumLength = KeyName.Length;
            KeyName.Buffer = (*ReferenceBi).Name.as_mut_ptr();

            if RtlEqualUnicodeString(&KeyName, &Control, 1u8) != 0 {
                ExFreePool(ReferenceBi as PVOID);
                ReferenceBi = core::ptr::null_mut();
                continue;
            }

            InitializeObjectAttributes(
                &mut ObjectAttributes,
                &mut KeyName,
                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                DeviceKey,
                core::ptr::null_mut(),
            );

            Status = ZwOpenKey(&mut ReferenceKey, KEY_QUERY_VALUE, &mut ObjectAttributes);
            if Status < 0 {
                break;
            }

            if (Flags & DEVICE_INTERFACE_INCLUDE_NONACTIVE) == 0 {
                InitializeObjectAttributes(
                    &mut ObjectAttributes,
                    &mut Control,
                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                    ReferenceKey,
                    core::ptr::null_mut(),
                );

                Status = ZwOpenKey(&mut ControlKey, KEY_QUERY_VALUE, &mut ObjectAttributes);
                if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                    if !ReferenceKey.is_null() {
                        ZwClose(ReferenceKey);
                        ReferenceKey = core::ptr::null_mut();
                    }
                    if !ReferenceBi.is_null() {
                        ExFreePool(ReferenceBi as PVOID);
                        ReferenceBi = core::ptr::null_mut();
                    }
                    continue;
                } else if Status < 0 {
                    break;
                }

                let mut LinkedName: _UNICODE_STRING = core::mem::zeroed();
                // L"Linked"
                const LINKED_W: [u16; 7] = [
                    b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16,
                    b'e' as u16, b'd' as u16, 0,
                ];
                RtlInitUnicodeString(&mut LinkedName, LINKED_W.as_ptr());

                let mut NeededLength = 0u32;
                Status = ZwQueryValueKey(ControlKey, &mut LinkedName, 2, core::ptr::null_mut(), 0, &mut NeededLength);

                if Status == STATUS_BUFFER_TOO_SMALL {
                    let PartialInfo = ExAllocatePool(NonPagedPool, NeededLength as _) as PKEY_VALUE_PARTIAL_INFORMATION;
                    if PartialInfo.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break;
                    }

                    Status = ZwQueryValueKey(ControlKey, &mut LinkedName, 2, PartialInfo as PVOID, NeededLength, &mut NeededLength);
                    if Status < 0 {
                        ExFreePool(PartialInfo as PVOID);
                        break;
                    }

                    if (*PartialInfo).Type != REG_DWORD || (*PartialInfo).DataLength != 4 {
                        ExFreePool(PartialInfo as PVOID);
                        break;
                    }

                    let mut LinkedValue = 0u32;
                    core::ptr::copy_nonoverlapping((*PartialInfo).Data.as_ptr(), &mut LinkedValue as *mut u32 as *mut u8, 4);
                    ExFreePool(PartialInfo as PVOID);

                    if LinkedValue == 0 {
                        if !ControlKey.is_null() {
                            ZwClose(ControlKey);
                            ControlKey = core::ptr::null_mut();
                        }
                        if !ReferenceKey.is_null() {
                            ZwClose(ReferenceKey);
                            ReferenceKey = core::ptr::null_mut();
                        }
                        if !ReferenceBi.is_null() {
                            ExFreePool(ReferenceBi as PVOID);
                            ReferenceBi = core::ptr::null_mut();
                        }
                        continue;
                    }
                } else {
                    break;
                }
            }

            let mut BipSize = 0u32;
            Status = ZwQueryValueKey(ReferenceKey, &mut SymbolicLink, 2, core::ptr::null_mut(), 0, &mut BipSize);
            if Status < 0 && Status != STATUS_BUFFER_TOO_SMALL {
                break;
            }

            bip = ExAllocatePool(PagedPool, BipSize as _) as PKEY_VALUE_PARTIAL_INFORMATION;
            if bip.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break;
            }

            Status = ZwQueryValueKey(ReferenceKey, &mut SymbolicLink, 2, bip as PVOID, BipSize, &mut BipSize);
            if Status < 0 {
                break;
            } else if (*bip).Type != REG_SZ {
                Status = STATUS_UNSUCCESSFUL;
                break;
            } else if (*bip).DataLength < 10 {
                Status = STATUS_UNSUCCESSFUL;
                break;
            }

            KeyName.Length = (*bip).DataLength as u16;
            KeyName.MaximumLength = KeyName.Length;
            KeyName.Buffer = (*bip).Data.as_ptr() as *mut u16;

            // Copy L"\\??\\" (4 wide chars) into KeyName.Buffer
            const DOSDEV_PREFIX: [u16; 4] = [0x5C, 0x3F, 0x3F, 0x5C];
            core::ptr::copy_nonoverlapping(DOSDEV_PREFIX.as_ptr(), KeyName.Buffer, 4);

            if ReturnBuffer.Length as usize + KeyName.Length as usize + 2 > ReturnBuffer.MaximumLength as usize {
                let NewMax = core::cmp::max(
                    2 * ReturnBuffer.MaximumLength as usize,
                    ReturnBuffer.Length as usize + KeyName.Length as usize + 4,
                );
                let NewBuffer = ExAllocatePool(PagedPool, NewMax as _) as *mut u16;
                if NewBuffer.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break;
                }

                if !ReturnBuffer.Buffer.is_null() {
                    core::ptr::copy_nonoverlapping(
                        ReturnBuffer.Buffer as *const u8,
                        NewBuffer as *mut u8,
                        ReturnBuffer.Length as usize,
                    );
                    ExFreePool(ReturnBuffer.Buffer as PVOID);
                }
                ReturnBuffer.Buffer = NewBuffer;
                ReturnBuffer.MaximumLength = NewMax as u16;
            }

            Status = RtlAppendUnicodeStringToString(&mut ReturnBuffer, &KeyName);
            if Status < 0 {
                break;
            }

            ReturnBuffer.Length += 2;

            if !ReferenceBi.is_null() {
                ExFreePool(ReferenceBi as PVOID);
                ReferenceBi = core::ptr::null_mut();
            }
            if !bip.is_null() {
                ExFreePool(bip as PVOID);
                bip = core::ptr::null_mut();
            }
            if !ReferenceKey.is_null() {
                ZwClose(ReferenceKey);
                ReferenceKey = core::ptr::null_mut();
            }
            if !ControlKey.is_null() {
                ZwClose(ControlKey);
                ControlKey = core::ptr::null_mut();
            }
        }

        if FoundRightPDO != 0 {
            break;
        }

        if !DeviceBi.is_null() {
            ExFreePool(DeviceBi as PVOID);
            DeviceBi = core::ptr::null_mut();
        }
        if !DeviceKey.is_null() {
            ZwClose(DeviceKey);
            DeviceKey = core::ptr::null_mut();
        }
    }

    if ReturnBuffer.Length >= ReturnBuffer.MaximumLength {
        let NewBuffer = ExAllocatePool(PagedPool, (ReturnBuffer.MaximumLength as usize + 2) as _) as *mut u16;
        if NewBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            if !ReturnBuffer.Buffer.is_null() {
                core::ptr::copy_nonoverlapping(
                    ReturnBuffer.Buffer as *const u8,
                    NewBuffer as *mut u8,
                    ReturnBuffer.Length as usize,
                );
                ExFreePool(ReturnBuffer.Buffer as PVOID);
            }
            ReturnBuffer.Buffer = NewBuffer;
            ReturnBuffer.MaximumLength += 2;
        }
    }

    if !ReturnBuffer.Buffer.is_null() {
        *ReturnBuffer.Buffer.add((ReturnBuffer.Length / 2) as usize) = 0;
        *SymbolicLinkList = ReturnBuffer.Buffer;
        Status = STATUS_SUCCESS;
    }

    if Status < 0 && !ReturnBuffer.Buffer.is_null() {
        ExFreePool(ReturnBuffer.Buffer as PVOID);
    }
    if !InterfaceKey.is_null() {
        ZwClose(InterfaceKey);
    }
    if !DeviceKey.is_null() {
        ZwClose(DeviceKey);
    }
    if !ReferenceKey.is_null() {
        ZwClose(ReferenceKey);
    }
    if !ControlKey.is_null() {
        ZwClose(ControlKey);
    }
    if !DeviceBi.is_null() {
        ExFreePool(DeviceBi as PVOID);
    }
    if !ReferenceBi.is_null() {
        ExFreePool(ReferenceBi as PVOID);
    }
    if !bip.is_null() {
        ExFreePool(bip as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoRegisterDeviceInterface(
    PhysicalDeviceObject: PDEVICE_OBJECT,
    InterfaceClassGuid: LPCGUID,
    ReferenceString: PUNICODE_STRING,
    SymbolicLinkName: PUNICODE_STRING,
) -> NTSTATUS {
    let PhysicalDeviceObject = opaque_ptr(PhysicalDeviceObject);
    let ReferenceString = opaque_ptr(ReferenceString);
    let SymbolicLinkName = opaque_ptr(SymbolicLinkName);
    let mut InstancePath: PUNICODE_STRING;
    let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
    let mut InterfaceKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut BaseKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut PdoNameInfoBuffer: [u8; 256 * 2 + core::mem::size_of::<OBJECT_NAME_INFORMATION>()] = [0; 256 * 2 + core::mem::size_of::<OBJECT_NAME_INFORMATION>()];
    let PdoNameInfo = PdoNameInfoBuffer.as_mut_ptr() as POBJECT_NAME_INFORMATION;
    // L"DeviceInstance" — 14 chars, Length = 28
    const DEV_INST_REG_W: [u16; 15] = [
        b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
        b'I' as u16, b'n' as u16, b's' as u16, b't' as u16, b'a' as u16, b'n' as u16,
        b'c' as u16, b'e' as u16, 0,
    ];
    let mut DeviceInstance: _UNICODE_STRING = _UNICODE_STRING {
        Length: 28,
        MaximumLength: 30,
        Buffer: DEV_INST_REG_W.as_ptr() as *mut u16,
    };
    // L"SymbolicLink" — 12 chars, Length = 24
    const SYM_LINK_REG_W: [u16; 13] = [
        b'S' as u16, b'y' as u16, b'm' as u16, b'b' as u16, b'o' as u16, b'l' as u16,
        b'i' as u16, b'c' as u16, b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16, 0,
    ];
    let mut SymbolicLinkVal: _UNICODE_STRING = _UNICODE_STRING {
        Length: 24,
        MaximumLength: 26,
        Buffer: SYM_LINK_REG_W.as_ptr() as *mut u16,
    };
    let mut ClassKey = core::ptr::null_mut();
    let mut InterfaceKey = core::ptr::null_mut();
    let mut SubKey = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    let DeviceObjectExtension = (*PhysicalDeviceObject).DeviceObjectExtension as PEXTENDED_DEVOBJ_EXTENSION;

    if (*DeviceObjectExtension).DeviceNode.is_null() {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    if (*(*DeviceObjectExtension).DeviceNode).InstancePath.Length == 0 {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    if !ReferenceString.is_null() {
        for i in 0..((*ReferenceString).Length / 2) as usize {
            let ch = *(*ReferenceString).Buffer.add(i);
            if ch == b'\\' as u16 || ch == b'/' as u16 {
                return STATUS_INVALID_DEVICE_REQUEST;
            }
        }
    }

    let mut Status = RtlStringFromGUID(InterfaceClassGuid, &mut GuidString);
    if Status < 0 {
        return Status;
    }

    let mut RetLen = 0u32;
    Status = ObQueryNameString(
        PhysicalDeviceObject as PVOID,
        PdoNameInfo,
        PdoNameInfoBuffer.len() as u32,
        &mut RetLen,
    );
    if Status < 0 {
        RtlFreeUnicodeString(&mut GuidString);
        return Status;
    }

    InstancePath = core::ptr::addr_of_mut!((*(*DeviceObjectExtension).DeviceNode).InstancePath);

    let base_key_len = {
        let mut len = 0;
        while BASE_KEY_STRING[len] != 0 {
            len += 1;
        }
        len * 2
    };

    BaseKeyName.Length = base_key_len as u16;
    BaseKeyName.MaximumLength = BaseKeyName.Length + GuidString.Length;
    BaseKeyName.Buffer = ExAllocatePool(PagedPool, BaseKeyName.MaximumLength as _) as *mut u16;
    if BaseKeyName.Buffer.is_null() {
        RtlFreeUnicodeString(&mut GuidString);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(BASE_KEY_STRING.as_ptr(), BaseKeyName.Buffer, base_key_len / 2);
    RtlAppendUnicodeStringToString(&mut BaseKeyName, &GuidString);

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut BaseKeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE | OBJ_OPENIF,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = ZwCreateKey(
        &mut ClassKey,
        KEY_WRITE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        core::ptr::null_mut(),
    );

    if Status < 0 {
        ExFreePool(BaseKeyName.Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return Status;
    }

    InterfaceKeyName.Length = 0;
    InterfaceKeyName.MaximumLength = 8 + (*InstancePath).Length + 2 + GuidString.Length;
    InterfaceKeyName.Buffer = ExAllocatePool(PagedPool, InterfaceKeyName.MaximumLength as _) as *mut u16;
    if InterfaceKeyName.Buffer.is_null() {
        ZwClose(ClassKey);
        ExFreePool(BaseKeyName.Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // L"##?#"
    const MUNGED_PREFIX_W: [u16; 5] = [0x23, 0x23, 0x3F, 0x23, 0]; // ##?#
    RtlAppendUnicodeToString(&mut InterfaceKeyName, MUNGED_PREFIX_W.as_ptr());
    let StartIndex = InterfaceKeyName.Length / 2;
    RtlAppendUnicodeStringToString(&mut InterfaceKeyName, InstancePath);

    for i in 0..((*InstancePath).Length / 2) as usize {
        if *InterfaceKeyName.Buffer.add(StartIndex as usize + i) == b'\\' as u16 {
            *InterfaceKeyName.Buffer.add(StartIndex as usize + i) = b'#' as u16;
        }
    }

    RtlAppendUnicodeToString(&mut InterfaceKeyName, [0x23u16, 0x00].as_ptr()); // L"#"
    RtlAppendUnicodeStringToString(&mut InterfaceKeyName, &GuidString);

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut InterfaceKeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE | OBJ_OPENIF,
        ClassKey,
        core::ptr::null_mut(),
    );

    Status = ZwCreateKey(
        &mut InterfaceKey,
        KEY_WRITE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        core::ptr::null_mut(),
    );

    if Status < 0 {
        ZwClose(ClassKey);
        ExFreePool(InterfaceKeyName.Buffer as PVOID);
        ExFreePool(BaseKeyName.Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return Status;
    }

    Status = ZwSetValueKey(
        InterfaceKey,
        &mut DeviceInstance,
        0,
        REG_SZ,
        (*InstancePath).Buffer as PVOID,
        (*InstancePath).Length as u32,
    );

    if Status < 0 {
        ZwClose(InterfaceKey);
        ZwClose(ClassKey);
        ExFreePool(InterfaceKeyName.Buffer as PVOID);
        ExFreePool(BaseKeyName.Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return Status;
    }

    let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();
    SubKeyName.Length = 0;
    SubKeyName.MaximumLength = 2;
    if !ReferenceString.is_null() && (*ReferenceString).Length > 0 {
        SubKeyName.MaximumLength += (*ReferenceString).Length;
    }

    SubKeyName.Buffer = ExAllocatePool(PagedPool, SubKeyName.MaximumLength as _) as *mut u16;
    if SubKeyName.Buffer.is_null() {
        ZwClose(InterfaceKey);
        ZwClose(ClassKey);
        ExFreePool(InterfaceKeyName.Buffer as PVOID);
        ExFreePool(BaseKeyName.Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlAppendUnicodeToString(&mut SubKeyName, [0x23u16, 0x00].as_ptr()); // L"#"
    if !ReferenceString.is_null() && (*ReferenceString).Length > 0 {
        RtlAppendUnicodeStringToString(&mut SubKeyName, ReferenceString);
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut SubKeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        InterfaceKey,
        core::ptr::null_mut(),
    );

    Status = ZwCreateKey(
        &mut SubKey,
        KEY_WRITE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        core::ptr::null_mut(),
    );

    if Status < 0 {
        ZwClose(InterfaceKey);
        ZwClose(ClassKey);
        ExFreePool(SubKeyName.Buffer as PVOID);
        ExFreePool(InterfaceKeyName.Buffer as PVOID);
        ExFreePool(BaseKeyName.Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return Status;
    }

    (*SymbolicLinkName).Length = 0;
    (*SymbolicLinkName).MaximumLength = 8 + (*InstancePath).Length + 2 + GuidString.Length + 2;
    if !ReferenceString.is_null() && (*ReferenceString).Length > 0 {
        (*SymbolicLinkName).MaximumLength += 2 + (*ReferenceString).Length;
    }

    (*SymbolicLinkName).Buffer = ExAllocatePool(PagedPool, (*SymbolicLinkName).MaximumLength as _) as *mut u16;
    if (*SymbolicLinkName).Buffer.is_null() {
        ZwClose(SubKey);
        ZwClose(InterfaceKey);
        ZwClose(ClassKey);
        ExFreePool(SubKeyName.Buffer as PVOID);
        ExFreePool(InterfaceKeyName.Buffer as PVOID);
        ExFreePool(BaseKeyName.Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // L"\\??\\"
    const DOSDEV_PREFIX2: [u16; 5] = [0x5C, 0x3F, 0x3F, 0x5C, 0];
    RtlAppendUnicodeToString(SymbolicLinkName, DOSDEV_PREFIX2.as_ptr());
    let StartIndexSym = (*SymbolicLinkName).Length / 2;
    RtlAppendUnicodeStringToString(SymbolicLinkName, InstancePath);

    for i in 0..((*InstancePath).Length / 2) as usize {
        if *(*SymbolicLinkName).Buffer.add(StartIndexSym as usize + i) == b'\\' as u16 {
            *(*SymbolicLinkName).Buffer.add(StartIndexSym as usize + i) = b'#' as u16;
        }
    }

    RtlAppendUnicodeToString(SymbolicLinkName, [0x23u16, 0x00].as_ptr()); // L"#"
    RtlAppendUnicodeStringToString(SymbolicLinkName, &GuidString);
    *(*SymbolicLinkName).Buffer.add(((*SymbolicLinkName).Length / 2) as usize) = 0;

    let SymLinkStatus = IoCreateSymbolicLink(SymbolicLinkName, &mut (*PdoNameInfo).Name);

    let SymLinkStatus = if SymLinkStatus == STATUS_OBJECT_NAME_COLLISION {
        IoDeleteSymbolicLink(SymbolicLinkName);
        IoCreateSymbolicLink(SymbolicLinkName, &mut (*PdoNameInfo).Name);
        STATUS_OBJECT_NAME_EXISTS
    } else {
        SymLinkStatus
    };

    if SymLinkStatus < 0 {
        ZwClose(SubKey);
        ZwClose(InterfaceKey);
        ZwClose(ClassKey);
        ExFreePool(SubKeyName.Buffer as PVOID);
        ExFreePool(InterfaceKeyName.Buffer as PVOID);
        ExFreePool(BaseKeyName.Buffer as PVOID);
        ExFreePool((*SymbolicLinkName).Buffer as PVOID);
        RtlFreeUnicodeString(&mut GuidString);
        return SymLinkStatus;
    }

    if !ReferenceString.is_null() && (*ReferenceString).Length > 0 {
        RtlAppendUnicodeToString(SymbolicLinkName, [0x5Cu16, 0x00].as_ptr()); // L"\\"
        RtlAppendUnicodeStringToString(SymbolicLinkName, ReferenceString);
    }
    *(*SymbolicLinkName).Buffer.add(((*SymbolicLinkName).Length / 2) as usize) = 0;

    *(*SymbolicLinkName).Buffer.add(1) = b'\\' as u16;
    Status = ZwSetValueKey(
        SubKey,
        &mut SymbolicLinkVal,
        0,
        REG_SZ,
        (*SymbolicLinkName).Buffer as PVOID,
        (*SymbolicLinkName).Length as u32,
    );

    if Status >= 0 {
        *(*SymbolicLinkName).Buffer.add(1) = b'?' as u16;
    } else {
        ExFreePool((*SymbolicLinkName).Buffer as PVOID);
    }

    ZwClose(SubKey);
    ZwClose(InterfaceKey);
    ZwClose(ClassKey);
    ExFreePool(SubKeyName.Buffer as PVOID);
    ExFreePool(InterfaceKeyName.Buffer as PVOID);
    ExFreePool(BaseKeyName.Buffer as PVOID);
    RtlFreeUnicodeString(&mut GuidString);

    if Status >= 0 {
        SymLinkStatus
    } else {
        Status
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoSetDeviceInterfaceState(
    SymbolicLinkName: PUNICODE_STRING,
    Enable: BOOLEAN,
) -> NTSTATUS {
    let SymbolicLinkName = opaque_ptr(SymbolicLinkName);
    let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
    let mut InstanceHandle = core::ptr::null_mut();
    let mut ControlHandle = core::ptr::null_mut();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceInstance: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DeviceGuid: GUID = core::mem::zeroed();
    // L"\\??\\" — 4 wide chars, Length = 8
    const DOSDEV1_W: [u16; 5] = [0x5C, 0x3F, 0x3F, 0x5C, 0];
    let mut DosDevicesPrefix1: _UNICODE_STRING = _UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: DOSDEV1_W.as_ptr() as *mut u16,
    };
    // L"\\\\?\\" — 4 wide chars, Length = 8
    const DOSDEV2_W: [u16; 5] = [0x5C, 0x5C, 0x3F, 0x5C, 0];
    let mut DosDevicesPrefix2: _UNICODE_STRING = _UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: DOSDEV2_W.as_ptr() as *mut u16,
    };

    if SymbolicLinkName.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if RtlPrefixUnicodeString(&DosDevicesPrefix1, SymbolicLinkName, 0u8) == 0 &&
       RtlPrefixUnicodeString(&DosDevicesPrefix2, SymbolicLinkName, 0u8) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    let mut LinkNameNoPrefix: _UNICODE_STRING = core::mem::zeroed();
    LinkNameNoPrefix.Buffer = (*SymbolicLinkName).Buffer.add((DosDevicesPrefix1.Length / 2) as usize);
    LinkNameNoPrefix.Length = (*SymbolicLinkName).Length - DosDevicesPrefix1.Length;
    LinkNameNoPrefix.MaximumLength = LinkNameNoPrefix.Length;

    let mut ReferenceStringOffset = 0u16;
    for i in 0..(LinkNameNoPrefix.Length / 2) {
        if *LinkNameNoPrefix.Buffer.add(i as usize) == b'\\' as u16 {
            ReferenceStringOffset = i * 2;
            break;
        }
    }
    if ReferenceStringOffset == 0 {
        ReferenceStringOffset = LinkNameNoPrefix.Length;
    }

    if ReferenceStringOffset < (GUID_STRING_BYTES + 2) as u16 {
        return STATUS_INVALID_PARAMETER;
    }

    GuidString.Buffer = LinkNameNoPrefix.Buffer.add(((ReferenceStringOffset - GUID_STRING_BYTES as u16) / 2) as usize);
    GuidString.Length = GUID_STRING_BYTES as u16;
    GuidString.MaximumLength = GuidString.Length;

    let mut Status = RtlGUIDFromString(&mut GuidString, &mut DeviceGuid);
    if Status < 0 {
        return Status;
    }

    Status = OpenRegistryHandlesFromSymbolicLink(
        SymbolicLinkName,
        KEY_CREATE_SUB_KEY,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut InstanceHandle,
    );
    if Status < 0 {
        return Status;
    }

    // L"Control"
    const CONTROL_KEY_W: [u16; 8] = [
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16,
        b'r' as u16, b'o' as u16, b'l' as u16, 0,
    ];
    RtlInitUnicodeString(&mut KeyName, CONTROL_KEY_W.as_ptr());
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        InstanceHandle,
        core::ptr::null_mut(),
    );

    Status = ZwCreateKey(
        &mut ControlHandle,
        KEY_SET_VALUE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        core::ptr::null_mut(),
    );
    ZwClose(InstanceHandle);
    if Status < 0 {
        return Status;
    }

    let LinkedValue = if Enable != 0 { 1u32 } else { 0u32 };

    // L"Linked"
    const LINKED_KEY_W: [u16; 7] = [
        b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16,
        b'e' as u16, b'd' as u16, 0,
    ];
    RtlInitUnicodeString(&mut KeyName, LINKED_KEY_W.as_ptr());
    Status = ZwSetValueKey(
        ControlHandle,
        &mut KeyName,
        0,
        REG_DWORD,
        core::ptr::addr_of!(LinkedValue) as PVOID,
        4,
    );
    ZwClose(ControlHandle);
    if Status < 0 {
        return Status;
    }

    DeviceInstance.Length = ((GuidString.Buffer as usize - LinkNameNoPrefix.Buffer as usize) / 2 - 1) as u16 * 2;
    if DeviceInstance.Length == 0 {
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    DeviceInstance.MaximumLength = DeviceInstance.Length;
    DeviceInstance.Buffer = ExAllocatePoolWithTag(PagedPool, DeviceInstance.MaximumLength as _, TAG_IO) as *mut u16;
    if DeviceInstance.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(
        LinkNameNoPrefix.Buffer as *const u8,
        DeviceInstance.Buffer as *mut u8,
        DeviceInstance.Length as usize,
    );

    for i in 0..(DeviceInstance.Length / 2) as usize {
        if *DeviceInstance.Buffer.add(i) == b'#' as u16 {
            *DeviceInstance.Buffer.add(i) = b'\\' as u16;
        }
    }

    let PhysicalDeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);

    if PhysicalDeviceObject.is_null() {
        ExFreePoolWithTag(DeviceInstance.Buffer as PVOID, TAG_IO);
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    ExFreePoolWithTag(DeviceInstance.Buffer as PVOID, TAG_IO);

    let EventGuid = if Enable != 0 {
        core::ptr::addr_of!(GUID_DEVICE_INTERFACE_ARRIVAL)
    } else {
        core::ptr::addr_of!(GUID_DEVICE_INTERFACE_REMOVAL)
    };

    PiNotifyDeviceInterfaceChange(EventGuid, &DeviceGuid, SymbolicLinkName);
    IopQueueDeviceChangeEvent(EventGuid, &DeviceGuid, SymbolicLinkName);

    ObDereferenceObject(PhysicalDeviceObject as PVOID);
    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/deviface.c
//  * PURPOSE:         Device interface functions
//  *
//  * PROGRAMMERS:     Filip Navara (xnavara@volny.cz)
//  *                  Matthew Brace (ismarc@austin.rr.com)
//  *                  Hervé Poussineau (hpoussin@reactos.org)
//  *                  Oleg Dubinskiy (oleg.dubinskiy@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FIXME: This should be somewhere global instead of having 20 different versions */
// #define GUID_STRING_CHARS 38
// #define GUID_STRING_BYTES (GUID_STRING_CHARS * sizeof(WCHAR))
// C_ASSERT(sizeof(L"{01234567-89ab-cdef-0123-456789abcdef}") == GUID_STRING_BYTES + sizeof(UNICODE_NULL));
//
// /* FUNCTIONS *****************************************************************/
//
// PDEVICE_OBJECT
// IopGetDeviceObjectFromDeviceInstance(PUNICODE_STRING DeviceInstance);
//
// static PCWSTR BaseKeyString = L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\DeviceClasses\\";
//
// /**
//  * @brief
//  * Creates a new symbolic link from the specified format of the prefix, device string,
//  * class GUID and reference string (if any).
//  *
//  * @param[in] DeviceString
//  * Device string, placed after prefix and before GUID, for example ACPI#PNP0501#1#.
//  *
//  * @param[in] GuidString
//  * Device interface class GUID represented by a string. Placed in curly brackets {},
//  * after device string, should always be 38 characters long. For example,
//  * {01234567-89ab-cdef-0123-456789abcdef}.
//  *
//  * @param[in] ReferenceString
//  * Optional reference string, if any. Placed after GUID, at the end of symbolic link.
//  * Usually contains human-readable subdevice name or class GUID.
//  *
//  * @param[in] UserModePrefixFormat
//  * Specifies whether a new symbolic link should have either a kernel mode or user mode prefix.
//  * TRUE for user mode prefix, FALSE for kernel mode.
//  *
//  * @param[out] SymbolicLinkName
//  * Pointer to unicode string which receives created symbolic link.
//  *
//  * @return
//  * STATUS_SUCCESS in case of success, or an NTSTATUS error code otherwise.
//  **/
// static
// NTSTATUS
// IopBuildSymbolicLink(
//     _In_ PCUNICODE_STRING DeviceString,
//     _In_ PCUNICODE_STRING GuidString,
//     _In_opt_ PCUNICODE_STRING ReferenceString,
//     _In_ BOOLEAN UserModePrefixFormat,
//     _Out_ PUNICODE_STRING SymbolicLinkName)
// {
//     static const UNICODE_STRING KernelModePrefix = RTL_CONSTANT_STRING(L"\\??\\");
//     static const UNICODE_STRING UserModePrefix = RTL_CONSTANT_STRING(L"\\\\?\\");
//     static const UNICODE_STRING PathSep = RTL_CONSTANT_STRING(L"\\");
//     UNICODE_STRING MungedDeviceString, SymbolicLink;
//     NTSTATUS Status;
//     ULONG Length;
//     USHORT i;
//
//     /* Use a backslash if reference string is not specified */
//     if (!ReferenceString)
//         ReferenceString = &PathSep;
//
//     /* Duplicate the device string (to "munge" it) */
//     Status = RtlDuplicateUnicodeString(0, DeviceString, &MungedDeviceString);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlDuplicateUnicodeString() failed, Status 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Replace all '\' by '#' in device string */
//     for (i = 0; i < MungedDeviceString.Length / sizeof(WCHAR); i++)
//     {
//         if (MungedDeviceString.Buffer[i] == L'\\')
//             MungedDeviceString.Buffer[i] = L'#';
//     }
//
//     /* Calculate total length */
//     Length = KernelModePrefix.Length // Same as UserModePrefix.Length
//            + MungedDeviceString.Length
//            + sizeof(L"#") + GuidString->Length
//            + ReferenceString->Length;
//     ASSERT(Length <= MAXUSHORT);
//
//     /* Build up new symbolic link */
//     SymbolicLink.Length = 0;
//     SymbolicLink.MaximumLength = Length;
//     SymbolicLink.Buffer = ExAllocatePoolWithTag(PagedPool, SymbolicLink.MaximumLength, TAG_IO);
//     if (!SymbolicLink.Buffer)
//     {
//         DPRINT1("ExAllocatePoolWithTag() failed\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = RtlUnicodeStringPrintf(&SymbolicLink,
//                                     L"%wZ%wZ#%wZ%wZ",
//                                     UserModePrefixFormat ?
//                                     &UserModePrefix : &KernelModePrefix,
//                                     &MungedDeviceString,
//                                     GuidString,
//                                     ReferenceString);
//     NT_VERIFY(NT_SUCCESS(Status));
//
//     DPRINT("New symbolic link is %wZ\n", &SymbolicLink);
//
//     *SymbolicLinkName = SymbolicLink;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Parses the specified symbolic link onto the 4 parts: prefix, device string,
//  * class GUID and reference string.
//  *
//  * @param[in] SymbolicLinkName
//  * Pointer to a symbolic link string to parse.
//  *
//  * @param[out] PrefixString
//  * Receives prefix of symbolic link. Can be '\??\' for Kernel mode or '\\?\' for User mode.
//  *
//  * @param[out] MungedString
//  * Receives device string. For example, ##?#ACPI#PNP0501#1#.
//  *
//  * @param[out] GuidString
//  * Receives device interface class GUID string represented by device interface.
//  * For example, {01234567-89ab-cdef-0123-456789abcdef}.
//  *
//  * @param[out] ReferenceString
//  * Receives reference string, if any. Usually contains a human-readable
//  * subdevice name or class GUID.
//  *
//  * @param[out] ReferenceStringPresent
//  * Pointer to variable that indicates whether the reference string exists in symbolic link.
//  * TRUE if it does, FALSE otherwise.
//  *
//  * @param[out] InterfaceClassGuid
//  * Receives the interface class GUID to which specified symbolic link belongs to.
//  *
//  * @return
//  * STATUS_SUCCESS in case of success, or an NTSTATUS error code otherwise.
//  **/
// static
// NTSTATUS
// IopSeparateSymbolicLink(
//     _In_ PCUNICODE_STRING SymbolicLinkName,
//     _Out_opt_ PUNICODE_STRING PrefixString,
//     _Out_opt_ PUNICODE_STRING MungedString,
//     _Out_opt_ PUNICODE_STRING GuidString,
//     _Out_opt_ PUNICODE_STRING ReferenceString,
//     _Out_opt_ PBOOLEAN ReferenceStringPresent,
//     _Out_opt_ LPGUID InterfaceClassGuid)
// {
//     static const UNICODE_STRING KernelModePrefix = RTL_CONSTANT_STRING(L"\\??\\");
//     static const UNICODE_STRING UserModePrefix = RTL_CONSTANT_STRING(L"\\\\?\\");
//     UNICODE_STRING MungedStringReal, GuidStringReal, ReferenceStringReal;
//     UNICODE_STRING LinkNameNoPrefix;
//     USHORT i, ReferenceStringOffset;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DPRINT("Symbolic link is %wZ\n", SymbolicLinkName);
//
//     /* The symbolic link name looks like \??\ACPI#PNP0501#1#{GUID}\ReferenceString
//      * Make sure it starts with the expected prefix. */
//     if (!RtlPrefixUnicodeString(&KernelModePrefix, SymbolicLinkName, FALSE) &&
//         !RtlPrefixUnicodeString(&UserModePrefix, SymbolicLinkName, FALSE))
//     {
//         DPRINT1("Invalid link name %wZ\n", SymbolicLinkName);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Sanity checks */
//     ASSERT(KernelModePrefix.Length == UserModePrefix.Length);
//     ASSERT(SymbolicLinkName->Length >= KernelModePrefix.Length);
//
//     /* Make a version without the prefix for further processing */
//     LinkNameNoPrefix.Buffer = SymbolicLinkName->Buffer + KernelModePrefix.Length / sizeof(WCHAR);
//     LinkNameNoPrefix.Length = SymbolicLinkName->Length - KernelModePrefix.Length;
//     LinkNameNoPrefix.MaximumLength = LinkNameNoPrefix.Length;
//
//     DPRINT("Symbolic link without prefix is %wZ\n", &LinkNameNoPrefix);
//
//     /* Find the reference string, if any */
//     for (i = 0; i < LinkNameNoPrefix.Length / sizeof(WCHAR); i++)
//     {
//         if (LinkNameNoPrefix.Buffer[i] == L'\\')
//             break;
//     }
//     ReferenceStringOffset = i * sizeof(WCHAR);
//
//     /* The GUID is before the reference string or at the end */
//     ASSERT(LinkNameNoPrefix.Length >= ReferenceStringOffset);
//     if (ReferenceStringOffset < GUID_STRING_BYTES + sizeof(WCHAR))
//     {
//         DPRINT1("Invalid link name %wZ\n", SymbolicLinkName);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Get reference string (starts with \ after {GUID}) from link without prefix */
//     ReferenceStringReal.Buffer = LinkNameNoPrefix.Buffer + ReferenceStringOffset / sizeof(WCHAR);
//     ReferenceStringReal.Length = LinkNameNoPrefix.Length - ReferenceStringOffset;
//     ReferenceStringReal.MaximumLength = ReferenceStringReal.Length;
//
//     DPRINT("Reference string is %wZ\n", &ReferenceStringReal);
//
//     /* Get GUID string (device class GUID in {} brackets) */
//     GuidStringReal.Buffer = LinkNameNoPrefix.Buffer + (ReferenceStringOffset - GUID_STRING_BYTES) / sizeof(WCHAR);
//     GuidStringReal.Length = GUID_STRING_BYTES;
//     GuidStringReal.MaximumLength = GuidStringReal.Length;
//
//     DPRINT("GUID string is %wZ\n", &GuidStringReal);
//
//     /* Validate GUID string for:
//      * 1) {} brackets at the start and the end;
//      * 2) - separators in the appropriate places. */
//     ASSERT(GuidStringReal.Buffer[0] == L'{');
//     ASSERT(GuidStringReal.Buffer[GUID_STRING_CHARS - 1] == L'}');
//     ASSERT(GuidStringReal.Buffer[9] == L'-');
//     ASSERT(GuidStringReal.Buffer[14] == L'-');
//     ASSERT(GuidStringReal.Buffer[19] == L'-');
//     ASSERT(GuidStringReal.Buffer[24] == L'-');
//
//     if (MungedString)
//     {
//         /* Create a munged path string (looks like ACPI#PNP0501#1#) */
//         MungedStringReal.Buffer = LinkNameNoPrefix.Buffer;
//         MungedStringReal.Length = LinkNameNoPrefix.Length - ReferenceStringReal.Length - GUID_STRING_BYTES - sizeof(WCHAR);
//         MungedStringReal.MaximumLength = MungedStringReal.Length;
//
//         DPRINT("Munged string is %wZ\n", &MungedStringReal);
//     }
//
//     /* Store received parts if the parameters are not null */
//     if (PrefixString)
//     {
//         PrefixString->Buffer = SymbolicLinkName->Buffer;
//         PrefixString->Length = KernelModePrefix.Length; // Same as UserModePrefix.Length
//         PrefixString->MaximumLength = PrefixString->Length;
//
//         DPRINT("Prefix string is %wZ\n", PrefixString);
//     }
//
//     if (MungedString)
//         *MungedString = MungedStringReal;
//
//     if (GuidString)
//         *GuidString = GuidStringReal;
//
//     if (ReferenceString)
//     {
//         if (ReferenceStringReal.Length > sizeof(WCHAR))
//             *ReferenceString = ReferenceStringReal;
//         else
//             RtlInitEmptyUnicodeString(ReferenceString, NULL, 0);
//     }
//
//     if (ReferenceStringPresent)
//         *ReferenceStringPresent = ReferenceStringReal.Length > sizeof(WCHAR);
//
//     if (InterfaceClassGuid)
//     {
//         /* Convert GUID string into a GUID and store it also */
//         Status = RtlGUIDFromString(&GuidStringReal, InterfaceClassGuid);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("RtlGUIDFromString() failed, Status 0x%08lx\n", Status);
//         }
//     }
//
//     /* We're done */
//     return Status;
// }
//
// /**
//  * @brief
//  * Retrieves a handles to the device and instance registry keys
//  * for the previously opened registry key handle of the specified symbolic link.
//  **/
// static
// NTSTATUS
// IopOpenOrCreateSymbolicLinkSubKeys(
//     _Out_opt_ PHANDLE DeviceHandle,
//     _Out_opt_ PULONG DeviceDisposition,
//     _Out_opt_ PHANDLE InstanceHandle,
//     _Out_opt_ PULONG InstanceDisposition,
//     _In_ HANDLE ClassHandle,
//     _In_ PCUNICODE_STRING SymbolicLinkName,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ BOOLEAN Create)
// {
//     UNICODE_STRING ReferenceString = {0};
//     UNICODE_STRING SymbolicLink = {0};
//     HANDLE DeviceKeyHandle, InstanceKeyHandle;
//     ULONG DeviceKeyDisposition, InstanceKeyDisposition;
//     BOOLEAN ReferenceStringPresent = FALSE; /* Assuming no ref string by default */
//     NTSTATUS Status;
//     USHORT i;
//
//     DeviceKeyHandle = InstanceKeyHandle = NULL;
//
//     /* Duplicate the symbolic link (we'll modify it later) */
//     Status = RtlDuplicateUnicodeString(0, SymbolicLinkName, &SymbolicLink);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlDuplicateUnicodeString() failed, Status 0x%08lx\n", Status);
//         goto Quit;
//     }
//
//     /* Separate it into its constituents */
//     Status = IopSeparateSymbolicLink(&SymbolicLink,
//                                      NULL,
//                                      NULL,
//                                      NULL,
//                                      &ReferenceString,
//                                      &ReferenceStringPresent,
//                                      NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to separate symbolic link %wZ, Status 0x%08lx\n", &SymbolicLink, Status);
//         goto Quit;
//     }
//
//     /* Did we got a ref string? */
//     if (ReferenceStringPresent)
//     {
//         /* Remove it from our symbolic link */
//         SymbolicLink.MaximumLength = SymbolicLink.Length -= ReferenceString.Length;
//
//         /* Replace the 1st backslash `\` character by '#' pound */
//         ReferenceString.Buffer[0] = L'#';
//     }
//     else
//     {
//         /* No ref string, initialize it with a single pound character '#' */
//         RtlInitUnicodeString(&ReferenceString, L"#");
//     }
//
//     /* Replace all '\' by '#' in symbolic link */
//     for (i = 0; i < SymbolicLink.Length / sizeof(WCHAR); i++)
//     {
//         if (SymbolicLink.Buffer[i] == L'\\')
//             SymbolicLink.Buffer[i] = L'#';
//     }
//
//     /* Fix prefix: '#??#' -> '##?#' */
//     SymbolicLink.Buffer[1] = L'#';
//
//     DPRINT("Munged symbolic link is %wZ\n", &SymbolicLink);
//
//     /* Try to open or create device interface keys */
//     if (Create)
//     {
//         Status = IopCreateRegistryKeyEx(&DeviceKeyHandle,
//                                         ClassHandle,
//                                         &SymbolicLink,
//                                         DesiredAccess | KEY_ENUMERATE_SUB_KEYS,
//                                         REG_OPTION_NON_VOLATILE,
//                                         &DeviceKeyDisposition);
//     }
//     else
//     {
//         Status = IopOpenRegistryKeyEx(&DeviceKeyHandle,
//                                       ClassHandle,
//                                       &SymbolicLink,
//                                       DesiredAccess | KEY_ENUMERATE_SUB_KEYS);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create or open %wZ, Status 0x%08lx\n", &SymbolicLink, Status);
//         goto Quit;
//     }
//
//     DPRINT("Munged reference string is %wZ\n", &ReferenceString);
//
//     /* Try to open or create instance subkeys */
//     if (Create)
//     {
//         Status = IopCreateRegistryKeyEx(&InstanceKeyHandle,
//                                         DeviceKeyHandle,
//                                         &ReferenceString,
//                                         DesiredAccess,
//                                         REG_OPTION_NON_VOLATILE,
//                                         &InstanceKeyDisposition);
//     }
//     else
//     {
//         Status = IopOpenRegistryKeyEx(&InstanceKeyHandle,
//                                       DeviceKeyHandle,
//                                       &ReferenceString,
//                                       DesiredAccess);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create or open %wZ, Status 0x%08lx\n", &ReferenceString, Status);
//         goto Quit;
//     }
//
//     Status = STATUS_SUCCESS;
//
// Quit:
//     if (NT_SUCCESS(Status))
//     {
//         if (DeviceHandle)
//             *DeviceHandle = DeviceKeyHandle;
//         else
//             ZwClose(DeviceKeyHandle);
//
//         if (DeviceDisposition)
//             *DeviceDisposition = DeviceKeyDisposition;
//
//         if (InstanceHandle)
//             *InstanceHandle = InstanceKeyHandle;
//         else
//             ZwClose(InstanceKeyHandle);
//
//         if (InstanceDisposition)
//             *InstanceDisposition = InstanceKeyDisposition;
//     }
//     else
//     {
//         if (InstanceKeyHandle)
//             ZwClose(InstanceKeyHandle);
//
//         if (Create)
//             ZwDeleteKey(DeviceKeyHandle);
//
//         if (DeviceKeyHandle)
//             ZwClose(DeviceKeyHandle);
//     }
//
//     if (SymbolicLink.Buffer)
//         RtlFreeUnicodeString(&SymbolicLink);
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Retrieves a handles to the GUID, device and instance registry keys
//  * for the specified symbolic link.
//  **/
// static
// NTSTATUS
// OpenRegistryHandlesFromSymbolicLink(
//     _In_ PCUNICODE_STRING SymbolicLinkName,
//     _In_ ACCESS_MASK DesiredAccess,
//     _Out_opt_ PHANDLE GuidKey,
//     _Out_opt_ PHANDLE DeviceKey,
//     _Out_opt_ PHANDLE InstanceKey)
// {
//     UNICODE_STRING BaseKeyU;
//     UNICODE_STRING GuidString;
//     HANDLE ClassesKey;
//     HANDLE GuidKeyReal, DeviceKeyReal, InstanceKeyReal;
//     NTSTATUS Status;
//
//     ClassesKey = GuidKeyReal = DeviceKeyReal = InstanceKeyReal = NULL;
//
//     RtlInitUnicodeString(&BaseKeyU, BaseKeyString);
//
//     /* Separate symbolic link onto the parts */
//     Status = IopSeparateSymbolicLink(SymbolicLinkName,
//                                      NULL,
//                                      NULL,
//                                      &GuidString,
//                                      NULL,
//                                      NULL,
//                                      NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to parse symbolic link %wZ, Status 0x%08lx\n",
//                 SymbolicLinkName, Status);
//         goto Quit;
//     }
//
//     /* Open the DeviceClasses key */
//     Status = IopOpenRegistryKeyEx(&ClassesKey,
//                                   NULL,
//                                   &BaseKeyU,
//                                   DesiredAccess | KEY_ENUMERATE_SUB_KEYS);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open %wZ, Status 0x%08lx\n", &BaseKeyU, Status);
//         goto Quit;
//     }
//
//     /* Open the GUID subkey */
//     Status = IopOpenRegistryKeyEx(&GuidKeyReal,
//                                   ClassesKey,
//                                   &GuidString,
//                                   DesiredAccess | KEY_ENUMERATE_SUB_KEYS);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open %wZ%wZ, Status 0x%08lx\n", &BaseKeyU, &GuidString, Status);
//         goto Quit;
//     }
//
//     /* Open the device and instance subkeys */
//     Status = IopOpenOrCreateSymbolicLinkSubKeys(&DeviceKeyReal,
//                                                 NULL,
//                                                 &InstanceKeyReal,
//                                                 NULL,
//                                                 GuidKeyReal,
//                                                 SymbolicLinkName,
//                                                 DesiredAccess,
//                                                 FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open %wZ%wZ, Status 0x%08lx\n", &BaseKeyU, &GuidString, Status);
//         goto Quit;
//     }
//
//     Status = STATUS_SUCCESS;
//
// Quit:
//     if (NT_SUCCESS(Status))
//     {
//         if (GuidKey)
//             *GuidKey = GuidKeyReal;
//         else
//             ZwClose(GuidKeyReal);
//
//         if (DeviceKey)
//             *DeviceKey = DeviceKeyReal;
//         else
//             ZwClose(DeviceKeyReal);
//
//         if (InstanceKey)
//             *InstanceKey = InstanceKeyReal;
//         else
//             ZwClose(InstanceKeyReal);
//     }
//     else
//     {
//         if (GuidKeyReal)
//             ZwClose(GuidKeyReal);
//
//         if (DeviceKeyReal)
//             ZwClose(DeviceKeyReal);
//
//         if (InstanceKeyReal)
//             ZwClose(InstanceKeyReal);
//     }
//
//     if (ClassesKey)
//         ZwClose(ClassesKey);
//
//     return Status;
// }
//
// /*++
//  * @name IoOpenDeviceInterfaceRegistryKey
//  * @unimplemented
//  *
//  * Provides a handle to the device's interface instance registry key.
//  * Documented in WDK.
//  *
//  * @param SymbolicLinkName
//  *        Pointer to a string which identifies the device interface instance
//  *
//  * @param DesiredAccess
//  *        Desired ACCESS_MASK used to access the key (like KEY_READ,
//  *        KEY_WRITE, etc)
//  *
//  * @param DeviceInterfaceKey
//  *        If a call has been succesfull, a handle to the registry key
//  *        will be stored there
//  *
//  * @return Three different NTSTATUS values in case of errors, and STATUS_SUCCESS
//  *         otherwise (see WDK for details)
//  *
//  * @remarks Must be called at IRQL = PASSIVE_LEVEL in the context of a system thread
//  *
//  *--*/
// NTSTATUS
// NTAPI
// IoOpenDeviceInterfaceRegistryKey(IN PUNICODE_STRING SymbolicLinkName,
//                                  IN ACCESS_MASK DesiredAccess,
//                                  OUT PHANDLE DeviceInterfaceKey)
// {
//     HANDLE InstanceKey, DeviceParametersKey;
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING DeviceParametersU = RTL_CONSTANT_STRING(L"Device Parameters");
//
//     Status = OpenRegistryHandlesFromSymbolicLink(SymbolicLinkName,
//                                                  KEY_CREATE_SUB_KEY,
//                                                  NULL,
//                                                  NULL,
//                                                  &InstanceKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DeviceParametersU,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE | OBJ_OPENIF,
//                                InstanceKey,
//                                NULL);
//     Status = ZwCreateKey(&DeviceParametersKey,
//                          DesiredAccess,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_NON_VOLATILE,
//                          NULL);
//     ZwClose(InstanceKey);
//
//     if (NT_SUCCESS(Status))
//         *DeviceInterfaceKey = DeviceParametersKey;
//
//     return Status;
// }
//
// /*++
//  * @name IopOpenInterfaceKey
//  *
//  * Returns the alias device interface of the specified device interface
//  *
//  * @param InterfaceClassGuid
//  *        FILLME
//  *
//  * @param DesiredAccess
//  *        FILLME
//  *
//  * @param pInterfaceKey
//  *        FILLME
//  *
//  * @return Usual NTSTATUS
//  *
//  * @remarks None
//  *
//  *--*/
// static NTSTATUS
// IopOpenInterfaceKey(IN CONST GUID *InterfaceClassGuid,
//                     IN ACCESS_MASK DesiredAccess,
//                     OUT HANDLE *pInterfaceKey)
// {
//     UNICODE_STRING LocalMachine = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\");
//     UNICODE_STRING GuidString;
//     UNICODE_STRING KeyName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE InterfaceKey = NULL;
//     NTSTATUS Status;
//
//     GuidString.Buffer = KeyName.Buffer = NULL;
//
//     Status = RtlStringFromGUID(InterfaceClassGuid, &GuidString);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("RtlStringFromGUID() failed with status 0x%08lx\n", Status);
//         goto cleanup;
//     }
//
//     KeyName.Length = 0;
//     KeyName.MaximumLength = LocalMachine.Length + ((USHORT)wcslen(REGSTR_PATH_DEVICE_CLASSES) + 1) * sizeof(WCHAR) + GuidString.Length;
//     KeyName.Buffer = ExAllocatePool(PagedPool, KeyName.MaximumLength);
//     if (!KeyName.Buffer)
//     {
//         DPRINT("ExAllocatePool() failed\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto cleanup;
//     }
//
//     Status = RtlAppendUnicodeStringToString(&KeyName, &LocalMachine);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("RtlAppendUnicodeStringToString() failed with status 0x%08lx\n", Status);
//         goto cleanup;
//     }
//     Status = RtlAppendUnicodeToString(&KeyName, REGSTR_PATH_DEVICE_CLASSES);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("RtlAppendUnicodeToString() failed with status 0x%08lx\n", Status);
//         goto cleanup;
//     }
//     Status = RtlAppendUnicodeToString(&KeyName, L"\\");
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("RtlAppendUnicodeToString() failed with status 0x%08lx\n", Status);
//         goto cleanup;
//     }
//     Status = RtlAppendUnicodeStringToString(&KeyName, &GuidString);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("RtlAppendUnicodeStringToString() failed with status 0x%08lx\n", Status);
//         goto cleanup;
//     }
//
//     InitializeObjectAttributes(
//         &ObjectAttributes,
//         &KeyName,
//         OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//         NULL,
//         NULL);
//     Status = ZwOpenKey(
//         &InterfaceKey,
//         DesiredAccess,
//         &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ZwOpenKey() failed with status 0x%08lx\n", Status);
//         goto cleanup;
//     }
//
//     *pInterfaceKey = InterfaceKey;
//     Status = STATUS_SUCCESS;
//
// cleanup:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InterfaceKey != NULL)
//             ZwClose(InterfaceKey);
//     }
//     RtlFreeUnicodeString(&GuidString);
//     RtlFreeUnicodeString(&KeyName);
//     return Status;
// }
//
// /**
//  * @brief
//  * Returns the alias device interface of the specified device interface
//  * instance, if the alias exists.
//  *
//  * @param[in] SymbolicLinkName
//  * Pointer to a symbolic link string which identifies the device interface instance.
//  *
//  * @param[in] AliasInterfaceClassGuid
//  * Pointer to a device interface class GUID.
//  *
//  * @param[out] AliasSymbolicLinkName
//  * Pointer to unicode string which receives the alias symbolic link upon success.
//  * Must be freed with RtlFreeUnicodeString after using.
//  *
//  * @return NTSTATUS values in case of errors, STATUS_SUCCESS otherwise.
//  *
//  * @remarks Must be called at IRQL = PASSIVE_LEVEL in the context of a system thread
//  **/
// NTSTATUS
// NTAPI
// IoGetDeviceInterfaceAlias(
//     _In_ PUNICODE_STRING SymbolicLinkName,
//     _In_ CONST GUID *AliasInterfaceClassGuid,
//     _Out_ PUNICODE_STRING AliasSymbolicLinkName)
// {
//     static const UNICODE_STRING UserModePrefix = RTL_CONSTANT_STRING(L"\\\\?\\");
//     UNICODE_STRING AliasSymbolicLink = {0};
//     UNICODE_STRING AliasGuidString = {0};
//     UNICODE_STRING DeviceString = {0};
//     UNICODE_STRING ReferenceString = {0};
//     PKEY_VALUE_FULL_INFORMATION kvInfo;
//     HANDLE DeviceKey, AliasInstanceKey;
//     BOOLEAN UserModePrefixFormat;
//     BOOLEAN ReferenceStringPresent = FALSE; /* Assuming no ref string by default */
//     PVOID Buffer;
//     NTSTATUS Status;
//
//     DPRINT("IoGetDeviceInterfaceAlias(%wZ, 0x%p)\n", SymbolicLinkName, AliasInterfaceClassGuid);
//
//     /* Sanity check */
//     if (!SymbolicLinkName || !AliasInterfaceClassGuid)
//     {
//         DPRINT1("IoGetDeviceInterfaceAlias() invalid symbolic link or alias class GUID\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Convert alias GUID to a string */
//     Status = RtlStringFromGUID(AliasInterfaceClassGuid, &AliasGuidString);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlStringFromGUID() failed, Status 0x%08lx\n", Status);
//         goto Quit;
//     }
//
//     DPRINT("Alias GUID is %wZ\n", &AliasGuidString);
//
//     /* Get the device instance string of existing symbolic link */
//     Status = OpenRegistryHandlesFromSymbolicLink(SymbolicLinkName,
//                                                  KEY_QUERY_VALUE,
//                                                  NULL,
//                                                  &DeviceKey,
//                                                  NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open device instance key for %wZ, Status 0x%08lx\n", SymbolicLinkName, Status);
//         goto Quit;
//     }
//
//     Status = IopGetRegistryValue(DeviceKey, L"DeviceInstance", &kvInfo);
//     ZwClose(DeviceKey);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed get device instance value, Status 0x%08lx\n", Status);
//         goto Quit;
//     }
//
//     if (kvInfo->Type != REG_SZ || kvInfo->DataLength == 0 || kvInfo->DataLength > MAXUSHORT)
//     {
//         DPRINT1("Wrong or empty instance value\n");
//         Status = STATUS_INVALID_PARAMETER;
//         goto Quit;
//     }
//
//     /* Convert received data to unicode string */
//     Buffer = (PVOID)((ULONG_PTR)kvInfo + kvInfo->DataOffset);
//     PnpRegSzToString(Buffer, kvInfo->DataLength, &DeviceString.Length);
//     DeviceString.MaximumLength = DeviceString.Length;
//     DeviceString.Buffer = Buffer;
//
//     /*
//      * Separate symbolic link into 4 parts:
//      * 1) prefix string (\??\ for kernel mode or \\?\ for user mode),
//      * 2) munged path string (like ##?#ACPI#PNP0501#1#{GUID}),
//      * 3) GUID string (the current GUID),
//      * 4) reference string (goes after GUID, starts with '\').
//      *
//      * We need only reference string.
//      */
//     Status = IopSeparateSymbolicLink(SymbolicLinkName,
//                                      NULL,
//                                      NULL,
//                                      NULL,
//                                      &ReferenceString,
//                                      &ReferenceStringPresent,
//                                      NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to separate symbolic link %wZ, Status 0x%08lx\n", SymbolicLinkName, Status);
//         goto Quit;
//     }
//
//     DPRINT("Device string is '%wZ'\n", &DeviceString);
//
//     /* Does symbolic link have kernel mode "\??\" or user mode "\\?\" prefix format? */
//     UserModePrefixFormat = RtlPrefixUnicodeString(&UserModePrefix, SymbolicLinkName, FALSE);
//
//     /* Build up new symbolic link with alias GUID */
//     Status = IopBuildSymbolicLink(&DeviceString,
//                                   &AliasGuidString,
//                                   ReferenceStringPresent ? &ReferenceString : NULL,
//                                   UserModePrefixFormat,
//                                   &AliasSymbolicLink);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to build alias symbolic link, Status 0x%08lx\n", Status);
//         goto Quit;
//     }
//
//     /* Make sure that alias symbolic link key exists in registry */
//     Status = OpenRegistryHandlesFromSymbolicLink(&AliasSymbolicLink,
//                                                  KEY_READ,
//                                                  NULL,
//                                                  NULL,
//                                                  &AliasInstanceKey);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open alias symbolic link key, Status 0x%08lx\n", Status);
//         goto Quit;
//     }
//     ZwClose(AliasInstanceKey);
//
//     /* We're done */
//     DPRINT("IoGetDeviceInterfaceAlias(): alias symbolic link %wZ\n", &AliasSymbolicLink);
//     *AliasSymbolicLinkName = AliasSymbolicLink;
//     Status = STATUS_SUCCESS;
//
// Quit:
//     if (!NT_SUCCESS(Status))
//     {
//         if (AliasSymbolicLink.Buffer)
//             RtlFreeUnicodeString(&AliasSymbolicLink);
//     }
//
//     if (AliasGuidString.Buffer)
//         RtlFreeUnicodeString(&AliasGuidString);
//
//     return Status;
// }
//
// /*++
//  * @name IoGetDeviceInterfaces
//  * @implemented
//  *
//  * Returns a list of device interfaces of a particular device interface class.
//  * Documented in WDK
//  *
//  * @param InterfaceClassGuid
//  *        Points to a class GUID specifying the device interface class
//  *
//  * @param PhysicalDeviceObject
//  *        Points to an optional PDO that narrows the search to only the
//  *        device interfaces of the device represented by the PDO
//  *
//  * @param Flags
//  *        Specifies flags that modify the search for device interfaces. The
//  *        DEVICE_INTERFACE_INCLUDE_NONACTIVE flag specifies that the list of
//  *        returned symbolic links should contain also disabled device
//  *        interfaces in addition to the enabled ones.
//  *
//  * @param SymbolicLinkList
//  *        Points to a character pointer that is filled in on successful return
//  *        with a list of unicode strings identifying the device interfaces
//  *        that match the search criteria. The newly allocated buffer contains
//  *        a list of symbolic link names. Each unicode string in the list is
//  *        null-terminated; the end of the whole list is marked by an additional
//  *        NULL. The caller is responsible for freeing the buffer (ExFreePool)
//  *        when it is no longer needed.
//  *        If no device interfaces match the search criteria, this routine
//  *        returns STATUS_SUCCESS and the string contains a single NULL
//  *        character.
//  *
//  * @return Usual NTSTATUS
//  *
//  * @remarks None
//  *
//  *--*/
// NTSTATUS
// NTAPI
// IoGetDeviceInterfaces(IN CONST GUID *InterfaceClassGuid,
//                       IN PDEVICE_OBJECT PhysicalDeviceObject OPTIONAL,
//                       IN ULONG Flags,
//                       OUT PWSTR *SymbolicLinkList)
// {
//     UNICODE_STRING Control = RTL_CONSTANT_STRING(L"Control");
//     UNICODE_STRING SymbolicLink = RTL_CONSTANT_STRING(L"SymbolicLink");
//     HANDLE InterfaceKey = NULL;
//     HANDLE DeviceKey = NULL;
//     HANDLE ReferenceKey = NULL;
//     HANDLE ControlKey = NULL;
//     PKEY_BASIC_INFORMATION DeviceBi = NULL;
//     PKEY_BASIC_INFORMATION ReferenceBi = NULL;
//     PKEY_VALUE_PARTIAL_INFORMATION bip = NULL;
//     PKEY_VALUE_PARTIAL_INFORMATION PartialInfo;
//     PEXTENDED_DEVOBJ_EXTENSION DeviceObjectExtension;
//     PUNICODE_STRING InstanceDevicePath = NULL;
//     UNICODE_STRING KeyName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     BOOLEAN FoundRightPDO = FALSE;
//     ULONG i = 0, j, Size, NeededLength, ActualLength, LinkedValue;
//     UNICODE_STRING ReturnBuffer = { 0, 0, NULL };
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     if (PhysicalDeviceObject != NULL)
//     {
//         /* Parameters must pass three border of checks */
//         DeviceObjectExtension = (PEXTENDED_DEVOBJ_EXTENSION)PhysicalDeviceObject->DeviceObjectExtension;
//
//         /* 1st level: Presence of a Device Node */
//         if (DeviceObjectExtension->DeviceNode == NULL)
//         {
//             DPRINT("PhysicalDeviceObject 0x%p doesn't have a DeviceNode\n", PhysicalDeviceObject);
//             return STATUS_INVALID_DEVICE_REQUEST;
//         }
//
//         /* 2nd level: Presence of an non-zero length InstancePath */
//         if (DeviceObjectExtension->DeviceNode->InstancePath.Length == 0)
//         {
//             DPRINT("PhysicalDeviceObject 0x%p's DOE has zero-length InstancePath\n", PhysicalDeviceObject);
//             return STATUS_INVALID_DEVICE_REQUEST;
//         }
//
//         InstanceDevicePath = &DeviceObjectExtension->DeviceNode->InstancePath;
//     }
//
//
//     Status = IopOpenInterfaceKey(InterfaceClassGuid, KEY_ENUMERATE_SUB_KEYS, &InterfaceKey);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("IopOpenInterfaceKey() failed with status 0x%08lx\n", Status);
//         goto cleanup;
//     }
//
//     /* Enumerate subkeys (i.e. the different device objects) */
//     while (TRUE)
//     {
//         Status = ZwEnumerateKey(
//             InterfaceKey,
//             i,
//             KeyBasicInformation,
//             NULL,
//             0,
//             &Size);
//         if (Status == STATUS_NO_MORE_ENTRIES)
//         {
//             break;
//         }
//         else if (!NT_SUCCESS(Status) && Status != STATUS_BUFFER_TOO_SMALL)
//         {
//             DPRINT("ZwEnumerateKey() failed with status 0x%08lx\n", Status);
//             goto cleanup;
//         }
//
//         DeviceBi = ExAllocatePool(PagedPool, Size);
//         if (!DeviceBi)
//         {
//             DPRINT("ExAllocatePool() failed\n");
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto cleanup;
//         }
//         Status = ZwEnumerateKey(
//             InterfaceKey,
//             i++,
//             KeyBasicInformation,
//             DeviceBi,
//             Size,
//             &Size);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("ZwEnumerateKey() failed with status 0x%08lx\n", Status);
//             goto cleanup;
//         }
//
//         /* Open device key */
//         KeyName.Length = KeyName.MaximumLength = (USHORT)DeviceBi->NameLength;
//         KeyName.Buffer = DeviceBi->Name;
//         InitializeObjectAttributes(
//             &ObjectAttributes,
//             &KeyName,
//             OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//             InterfaceKey,
//             NULL);
//         Status = ZwOpenKey(
//             &DeviceKey,
//             KEY_ENUMERATE_SUB_KEYS,
//             &ObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("ZwOpenKey() failed with status 0x%08lx\n", Status);
//             goto cleanup;
//         }
//
//         if (PhysicalDeviceObject)
//         {
//             /* Check if we are on the right physical device object,
//             * by reading the DeviceInstance string
//             */
//             RtlInitUnicodeString(&KeyName, L"DeviceInstance");
//             Status = ZwQueryValueKey(DeviceKey, &KeyName, KeyValuePartialInformation, NULL, 0, &NeededLength);
//             if (Status == STATUS_BUFFER_TOO_SMALL)
//             {
//                 ActualLength = NeededLength;
//                 PartialInfo = ExAllocatePool(NonPagedPool, ActualLength);
//                 if (!PartialInfo)
//                 {
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     goto cleanup;
//                 }
//
//                 Status = ZwQueryValueKey(DeviceKey, &KeyName, KeyValuePartialInformation, PartialInfo, ActualLength, &NeededLength);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     DPRINT1("ZwQueryValueKey #2 failed (%x)\n", Status);
//                     ExFreePool(PartialInfo);
//                     goto cleanup;
//                 }
//                 if (PartialInfo->DataLength == InstanceDevicePath->Length)
//                 {
//                     if (RtlCompareMemory(PartialInfo->Data, InstanceDevicePath->Buffer, InstanceDevicePath->Length) == InstanceDevicePath->Length)
//                     {
//                         /* found right pdo */
//                         FoundRightPDO = TRUE;
//                     }
//                 }
//                 ExFreePool(PartialInfo);
//                 PartialInfo = NULL;
//                 if (!FoundRightPDO)
//                 {
//                     /* not yet found */
//                     continue;
//                 }
//             }
//             else
//             {
//                 /* error */
//                 break;
//             }
//         }
//
//         /* Enumerate subkeys (ie the different reference strings) */
//         j = 0;
//         while (TRUE)
//         {
//             Status = ZwEnumerateKey(
//                 DeviceKey,
//                 j,
//                 KeyBasicInformation,
//                 NULL,
//                 0,
//                 &Size);
//             if (Status == STATUS_NO_MORE_ENTRIES)
//             {
//                 break;
//             }
//             else if (!NT_SUCCESS(Status) && Status != STATUS_BUFFER_TOO_SMALL)
//             {
//                 DPRINT("ZwEnumerateKey() failed with status 0x%08lx\n", Status);
//                 goto cleanup;
//             }
//
//             ReferenceBi = ExAllocatePool(PagedPool, Size);
//             if (!ReferenceBi)
//             {
//                 DPRINT("ExAllocatePool() failed\n");
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto cleanup;
//             }
//             Status = ZwEnumerateKey(
//                 DeviceKey,
//                 j++,
//                 KeyBasicInformation,
//                 ReferenceBi,
//                 Size,
//                 &Size);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT("ZwEnumerateKey() failed with status 0x%08lx\n", Status);
//                 goto cleanup;
//             }
//
//             KeyName.Length = KeyName.MaximumLength = (USHORT)ReferenceBi->NameLength;
//             KeyName.Buffer = ReferenceBi->Name;
//             if (RtlEqualUnicodeString(&KeyName, &Control, TRUE))
//             {
//                 /* Skip Control subkey */
//                 goto NextReferenceString;
//             }
//
//             /* Open reference key */
//             InitializeObjectAttributes(
//                 &ObjectAttributes,
//                 &KeyName,
//                 OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                 DeviceKey,
//                 NULL);
//             Status = ZwOpenKey(
//                 &ReferenceKey,
//                 KEY_QUERY_VALUE,
//                 &ObjectAttributes);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT("ZwOpenKey() failed with status 0x%08lx\n", Status);
//                 goto cleanup;
//             }
//
//             if (!(Flags & DEVICE_INTERFACE_INCLUDE_NONACTIVE))
//             {
//                 /* We have to check if the interface is enabled, by
//                 * reading the Linked value in the Control subkey
//                 */
//                 InitializeObjectAttributes(
//                     &ObjectAttributes,
//                     &Control,
//                     OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                     ReferenceKey,
//                     NULL);
//                 Status = ZwOpenKey(
//                     &ControlKey,
//                     KEY_QUERY_VALUE,
//                     &ObjectAttributes);
//                 if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//                 {
//                     /* That's OK. The key doesn't exist (yet) because
//                     * the interface is not activated.
//                     */
//                     goto NextReferenceString;
//                 }
//                 else if (!NT_SUCCESS(Status))
//                 {
//                     DPRINT1("ZwOpenKey() failed with status 0x%08lx\n", Status);
//                     goto cleanup;
//                 }
//
//                 RtlInitUnicodeString(&KeyName, L"Linked");
//                 Status = ZwQueryValueKey(ControlKey,
//                                          &KeyName,
//                                          KeyValuePartialInformation,
//                                          NULL,
//                                          0,
//                                          &NeededLength);
//                 if (Status == STATUS_BUFFER_TOO_SMALL)
//                 {
//                     ActualLength = NeededLength;
//                     PartialInfo = ExAllocatePool(NonPagedPool, ActualLength);
//                     if (!PartialInfo)
//                     {
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//                         goto cleanup;
//                     }
//
//                     Status = ZwQueryValueKey(ControlKey,
//                                              &KeyName,
//                                              KeyValuePartialInformation,
//                                              PartialInfo,
//                                              ActualLength,
//                                              &NeededLength);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         DPRINT1("ZwQueryValueKey #2 failed (%x)\n", Status);
//                         ExFreePool(PartialInfo);
//                         goto cleanup;
//                     }
//
//                     if (PartialInfo->Type != REG_DWORD || PartialInfo->DataLength != sizeof(ULONG))
//                     {
//                         DPRINT1("Bad registry read\n");
//                         ExFreePool(PartialInfo);
//                         goto cleanup;
//                     }
//
//                     RtlCopyMemory(&LinkedValue,
//                                   PartialInfo->Data,
//                                   PartialInfo->DataLength);
//
//                     ExFreePool(PartialInfo);
//                     if (LinkedValue == 0)
//                     {
//                         /* This interface isn't active */
//                         goto NextReferenceString;
//                     }
//                 }
//                 else
//                 {
//                     DPRINT1("ZwQueryValueKey #1 failed (%x)\n", Status);
//                     goto cleanup;
//                 }
//             }
//
//             /* Read the SymbolicLink string and add it into SymbolicLinkList */
//             Status = ZwQueryValueKey(
//                 ReferenceKey,
//                 &SymbolicLink,
//                 KeyValuePartialInformation,
//                 NULL,
//                 0,
//                 &Size);
//             if (!NT_SUCCESS(Status) && Status != STATUS_BUFFER_TOO_SMALL)
//             {
//                 DPRINT("ZwQueryValueKey() failed with status 0x%08lx\n", Status);
//                 goto cleanup;
//             }
//             bip = ExAllocatePool(PagedPool, Size);
//             if (!bip)
//             {
//                 DPRINT("ExAllocatePool() failed\n");
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto cleanup;
//             }
//             Status = ZwQueryValueKey(
//                 ReferenceKey,
//                 &SymbolicLink,
//                 KeyValuePartialInformation,
//                 bip,
//                 Size,
//                 &Size);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT("ZwQueryValueKey() failed with status 0x%08lx\n", Status);
//                 goto cleanup;
//             }
//             else if (bip->Type != REG_SZ)
//             {
//                 DPRINT("Unexpected registry type 0x%lx (expected 0x%lx)\n", bip->Type, REG_SZ);
//                 Status = STATUS_UNSUCCESSFUL;
//                 goto cleanup;
//             }
//             else if (bip->DataLength < 5 * sizeof(WCHAR))
//             {
//                 DPRINT("Registry string too short (length %lu, expected %lu at least)\n", bip->DataLength, 5 * sizeof(WCHAR));
//                 Status = STATUS_UNSUCCESSFUL;
//                 goto cleanup;
//             }
//             KeyName.Length = KeyName.MaximumLength = (USHORT)bip->DataLength;
//             KeyName.Buffer = (PWSTR)bip->Data;
//
//             /* Fixup the prefix (from "\\?\") */
//             RtlCopyMemory(KeyName.Buffer, L"\\??\\", 4 * sizeof(WCHAR));
//
//             /* Add new symbolic link to symbolic link list */
//             if (ReturnBuffer.Length + KeyName.Length + sizeof(WCHAR) > ReturnBuffer.MaximumLength)
//             {
//                 PWSTR NewBuffer;
//                 ReturnBuffer.MaximumLength = (USHORT)max(2 * ReturnBuffer.MaximumLength,
//                                                          (USHORT)(ReturnBuffer.Length +
//                                                          KeyName.Length +
//                                                          2 * sizeof(WCHAR)));
//                 NewBuffer = ExAllocatePool(PagedPool, ReturnBuffer.MaximumLength);
//                 if (!NewBuffer)
//                 {
//                     DPRINT("ExAllocatePool() failed\n");
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     goto cleanup;
//                 }
//                 if (ReturnBuffer.Buffer)
//                 {
//                     RtlCopyMemory(NewBuffer, ReturnBuffer.Buffer, ReturnBuffer.Length);
//                     ExFreePool(ReturnBuffer.Buffer);
//                 }
//                 ReturnBuffer.Buffer = NewBuffer;
//             }
//             DPRINT("Adding symbolic link %wZ\n", &KeyName);
//             Status = RtlAppendUnicodeStringToString(&ReturnBuffer, &KeyName);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT("RtlAppendUnicodeStringToString() failed with status 0x%08lx\n", Status);
//                 goto cleanup;
//             }
//             /* RtlAppendUnicodeStringToString added a NULL at the end of the
//              * destination string, but didn't increase the Length field.
//              * Do it for it.
//              */
//             ReturnBuffer.Length += sizeof(WCHAR);
//
// NextReferenceString:
//             ExFreePool(ReferenceBi);
//             ReferenceBi = NULL;
//             if (bip)
//                 ExFreePool(bip);
//             bip = NULL;
//             if (ReferenceKey != NULL)
//             {
//                 ZwClose(ReferenceKey);
//                 ReferenceKey = NULL;
//             }
//             if (ControlKey != NULL)
//             {
//                 ZwClose(ControlKey);
//                 ControlKey = NULL;
//             }
//         }
//         if (FoundRightPDO)
//         {
//             /* No need to go further, as we already have found what we searched */
//             break;
//         }
//
//         ExFreePool(DeviceBi);
//         DeviceBi = NULL;
//         ZwClose(DeviceKey);
//         DeviceKey = NULL;
//     }
//
//     /* Add final NULL to ReturnBuffer */
//     ASSERT(ReturnBuffer.Length <= ReturnBuffer.MaximumLength);
//     if (ReturnBuffer.Length >= ReturnBuffer.MaximumLength)
//     {
//         PWSTR NewBuffer;
//         ReturnBuffer.MaximumLength += sizeof(WCHAR);
//         NewBuffer = ExAllocatePool(PagedPool, ReturnBuffer.MaximumLength);
//         if (!NewBuffer)
//         {
//             DPRINT("ExAllocatePool() failed\n");
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto cleanup;
//         }
//         if (ReturnBuffer.Buffer)
//         {
//             RtlCopyMemory(NewBuffer, ReturnBuffer.Buffer, ReturnBuffer.Length);
//             ExFreePool(ReturnBuffer.Buffer);
//         }
//         ReturnBuffer.Buffer = NewBuffer;
//     }
//     ReturnBuffer.Buffer[ReturnBuffer.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     *SymbolicLinkList = ReturnBuffer.Buffer;
//     Status = STATUS_SUCCESS;
//
// cleanup:
//     if (!NT_SUCCESS(Status) && ReturnBuffer.Buffer)
//         ExFreePool(ReturnBuffer.Buffer);
//     if (InterfaceKey != NULL)
//         ZwClose(InterfaceKey);
//     if (DeviceKey != NULL)
//         ZwClose(DeviceKey);
//     if (ReferenceKey != NULL)
//         ZwClose(ReferenceKey);
//     if (ControlKey != NULL)
//         ZwClose(ControlKey);
//     if (DeviceBi)
//         ExFreePool(DeviceBi);
//     if (ReferenceBi)
//         ExFreePool(ReferenceBi);
//     if (bip)
//         ExFreePool(bip);
//     return Status;
// }
//
// /*++
//  * @name IoRegisterDeviceInterface
//  * @implemented
//  *
//  * Registers a device interface class, if it has not been previously registered,
//  * and creates a new instance of the interface class, which a driver can
//  * subsequently enable for use by applications or other system components.
//  * Documented in WDK.
//  *
//  * @param PhysicalDeviceObject
//  *        Points to an optional PDO that narrows the search to only the
//  *        device interfaces of the device represented by the PDO
//  *
//  * @param InterfaceClassGuid
//  *        Points to a class GUID specifying the device interface class
//  *
//  * @param ReferenceString
//  *        Optional parameter, pointing to a unicode string. For a full
//  *        description of this rather rarely used param (usually drivers
//  *        pass NULL here) see WDK
//  *
//  * @param SymbolicLinkName
//  *        Pointer to the resulting unicode string
//  *
//  * @return Usual NTSTATUS
//  *
//  * @remarks Must be called at IRQL = PASSIVE_LEVEL in the context of a
//  *          system thread
//  *
//  *--*/
// NTSTATUS
// NTAPI
// IoRegisterDeviceInterface(IN PDEVICE_OBJECT PhysicalDeviceObject,
//                           IN CONST GUID *InterfaceClassGuid,
//                           IN PUNICODE_STRING ReferenceString OPTIONAL,
//                           OUT PUNICODE_STRING SymbolicLinkName)
// {
//     PUNICODE_STRING InstancePath;
//     UNICODE_STRING GuidString;
//     UNICODE_STRING SubKeyName;
//     UNICODE_STRING InterfaceKeyName;
//     UNICODE_STRING BaseKeyName;
//     UCHAR PdoNameInfoBuffer[sizeof(OBJECT_NAME_INFORMATION) + (256 * sizeof(WCHAR))];
//     POBJECT_NAME_INFORMATION PdoNameInfo = (POBJECT_NAME_INFORMATION)PdoNameInfoBuffer;
//     UNICODE_STRING DeviceInstance = RTL_CONSTANT_STRING(L"DeviceInstance");
//     UNICODE_STRING SymbolicLink = RTL_CONSTANT_STRING(L"SymbolicLink");
//     HANDLE ClassKey;
//     HANDLE InterfaceKey;
//     HANDLE SubKey;
//     ULONG StartIndex;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     ULONG i;
//     NTSTATUS Status, SymLinkStatus;
//     PEXTENDED_DEVOBJ_EXTENSION DeviceObjectExtension;
//
//     ASSERT_IRQL_EQUAL(PASSIVE_LEVEL);
//
//     DPRINT("IoRegisterDeviceInterface(): PDO %p, RefString: %wZ\n",
//         PhysicalDeviceObject, ReferenceString);
//
//     /* Parameters must pass three border of checks */
//     DeviceObjectExtension = (PEXTENDED_DEVOBJ_EXTENSION)PhysicalDeviceObject->DeviceObjectExtension;
//
//     /* 1st level: Presence of a Device Node */
//     if (DeviceObjectExtension->DeviceNode == NULL)
//     {
//         DPRINT("PhysicalDeviceObject 0x%p doesn't have a DeviceNode\n", PhysicalDeviceObject);
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     /* 2nd level: Presence of an non-zero length InstancePath */
//     if (DeviceObjectExtension->DeviceNode->InstancePath.Length == 0)
//     {
//         DPRINT("PhysicalDeviceObject 0x%p's DOE has zero-length InstancePath\n", PhysicalDeviceObject);
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     /* 3rd level: Optional, based on WDK documentation */
//     if (ReferenceString != NULL)
//     {
//         /* Reference string must not contain path-separator symbols */
//         for (i = 0; i < ReferenceString->Length / sizeof(WCHAR); i++)
//         {
//             if ((ReferenceString->Buffer[i] == '\\') ||
//                 (ReferenceString->Buffer[i] == '/'))
//                 return STATUS_INVALID_DEVICE_REQUEST;
//         }
//     }
//
//     Status = RtlStringFromGUID(InterfaceClassGuid, &GuidString);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("RtlStringFromGUID() failed with status 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Create Pdo name: \Device\xxxxxxxx (unnamed device) */
//     Status = ObQueryNameString(
//         PhysicalDeviceObject,
//         PdoNameInfo,
//         sizeof(PdoNameInfoBuffer),
//         &i);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ObQueryNameString() failed with status 0x%08lx\n", Status);
//         return Status;
//     }
//     ASSERT(PdoNameInfo->Name.Length);
//
//     /* Create base key name for this interface: HKLM\SYSTEM\CurrentControlSet\Control\DeviceClasses\{GUID} */
//     ASSERT(((PEXTENDED_DEVOBJ_EXTENSION)PhysicalDeviceObject->DeviceObjectExtension)->DeviceNode);
//     InstancePath = &((PEXTENDED_DEVOBJ_EXTENSION)PhysicalDeviceObject->DeviceObjectExtension)->DeviceNode->InstancePath;
//     BaseKeyName.Length = (USHORT)wcslen(BaseKeyString) * sizeof(WCHAR);
//     BaseKeyName.MaximumLength = BaseKeyName.Length
//         + GuidString.Length;
//     BaseKeyName.Buffer = ExAllocatePool(
//         PagedPool,
//         BaseKeyName.MaximumLength);
//     if (!BaseKeyName.Buffer)
//     {
//         DPRINT("ExAllocatePool() failed\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     wcscpy(BaseKeyName.Buffer, BaseKeyString);
//     RtlAppendUnicodeStringToString(&BaseKeyName, &GuidString);
//
//     /* Create BaseKeyName key in registry */
//     InitializeObjectAttributes(
//         &ObjectAttributes,
//         &BaseKeyName,
//         OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE | OBJ_OPENIF,
//         NULL, /* RootDirectory */
//         NULL); /* SecurityDescriptor */
//
//     Status = ZwCreateKey(
//         &ClassKey,
//         KEY_WRITE,
//         &ObjectAttributes,
//         0, /* TileIndex */
//         NULL, /* Class */
//         REG_OPTION_NON_VOLATILE,
//         NULL); /* Disposition */
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ZwCreateKey() failed with status 0x%08lx\n", Status);
//         ExFreePool(BaseKeyName.Buffer);
//         return Status;
//     }
//
//     /* Create key name for this interface: ##?#ACPI#PNP0501#1#{GUID} */
//     InterfaceKeyName.Length = 0;
//     InterfaceKeyName.MaximumLength =
//         4 * sizeof(WCHAR) + /* 4  = size of ##?# */
//         InstancePath->Length +
//         sizeof(WCHAR) +     /* 1  = size of # */
//         GuidString.Length;
//     InterfaceKeyName.Buffer = ExAllocatePool(
//         PagedPool,
//         InterfaceKeyName.MaximumLength);
//     if (!InterfaceKeyName.Buffer)
//     {
//         DPRINT("ExAllocatePool() failed\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlAppendUnicodeToString(&InterfaceKeyName, L"##?#");
//     StartIndex = InterfaceKeyName.Length / sizeof(WCHAR);
//     RtlAppendUnicodeStringToString(&InterfaceKeyName, InstancePath);
//     for (i = 0; i < InstancePath->Length / sizeof(WCHAR); i++)
//     {
//         if (InterfaceKeyName.Buffer[StartIndex + i] == '\\')
//             InterfaceKeyName.Buffer[StartIndex + i] = '#';
//     }
//     RtlAppendUnicodeToString(&InterfaceKeyName, L"#");
//     RtlAppendUnicodeStringToString(&InterfaceKeyName, &GuidString);
//
//     /* Create the interface key in registry */
//     InitializeObjectAttributes(
//         &ObjectAttributes,
//         &InterfaceKeyName,
//         OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE | OBJ_OPENIF,
//         ClassKey,
//         NULL); /* SecurityDescriptor */
//
//     Status = ZwCreateKey(
//         &InterfaceKey,
//         KEY_WRITE,
//         &ObjectAttributes,
//         0, /* TileIndex */
//         NULL, /* Class */
//         REG_OPTION_NON_VOLATILE,
//         NULL); /* Disposition */
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ZwCreateKey() failed with status 0x%08lx\n", Status);
//         ZwClose(ClassKey);
//         ExFreePool(BaseKeyName.Buffer);
//         return Status;
//     }
//
//     /* Write DeviceInstance entry. Value is InstancePath */
//     Status = ZwSetValueKey(
//         InterfaceKey,
//         &DeviceInstance,
//         0, /* TileIndex */
//         REG_SZ,
//         InstancePath->Buffer,
//         InstancePath->Length);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ZwSetValueKey() failed with status 0x%08lx\n", Status);
//         ZwClose(InterfaceKey);
//         ZwClose(ClassKey);
//         ExFreePool(InterfaceKeyName.Buffer);
//         ExFreePool(BaseKeyName.Buffer);
//         return Status;
//     }
//
//     /* Create subkey. Name is #ReferenceString */
//     SubKeyName.Length = 0;
//     SubKeyName.MaximumLength = sizeof(WCHAR);
//     if (ReferenceString && ReferenceString->Length)
//         SubKeyName.MaximumLength += ReferenceString->Length;
//     SubKeyName.Buffer = ExAllocatePool(
//         PagedPool,
//         SubKeyName.MaximumLength);
//     if (!SubKeyName.Buffer)
//     {
//         DPRINT("ExAllocatePool() failed\n");
//         ZwClose(InterfaceKey);
//         ZwClose(ClassKey);
//         ExFreePool(InterfaceKeyName.Buffer);
//         ExFreePool(BaseKeyName.Buffer);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     RtlAppendUnicodeToString(&SubKeyName, L"#");
//     if (ReferenceString && ReferenceString->Length)
//         RtlAppendUnicodeStringToString(&SubKeyName, ReferenceString);
//
//     /* Create SubKeyName key in registry */
//     InitializeObjectAttributes(
//         &ObjectAttributes,
//         &SubKeyName,
//         OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//         InterfaceKey, /* RootDirectory */
//         NULL); /* SecurityDescriptor */
//
//     Status = ZwCreateKey(
//         &SubKey,
//         KEY_WRITE,
//         &ObjectAttributes,
//         0, /* TileIndex */
//         NULL, /* Class */
//         REG_OPTION_NON_VOLATILE,
//         NULL); /* Disposition */
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ZwCreateKey() failed with status 0x%08lx\n", Status);
//         ZwClose(InterfaceKey);
//         ZwClose(ClassKey);
//         ExFreePool(InterfaceKeyName.Buffer);
//         ExFreePool(BaseKeyName.Buffer);
//         return Status;
//     }
//
//     /* Create symbolic link name: \??\ACPI#PNP0501#1#{GUID}\ReferenceString */
//     SymbolicLinkName->Length = 0;
//     SymbolicLinkName->MaximumLength = SymbolicLinkName->Length
//         + 4 * sizeof(WCHAR) /* 4 = size of \??\ */
//         + InstancePath->Length
//         + sizeof(WCHAR)     /* 1  = size of # */
//         + GuidString.Length
//         + sizeof(WCHAR);    /* final NULL */
//     if (ReferenceString && ReferenceString->Length)
//         SymbolicLinkName->MaximumLength += sizeof(WCHAR) + ReferenceString->Length;
//     SymbolicLinkName->Buffer = ExAllocatePool(
//         PagedPool,
//         SymbolicLinkName->MaximumLength);
//     if (!SymbolicLinkName->Buffer)
//     {
//         DPRINT("ExAllocatePool() failed\n");
//         ZwClose(SubKey);
//         ZwClose(InterfaceKey);
//         ZwClose(ClassKey);
//         ExFreePool(InterfaceKeyName.Buffer);
//         ExFreePool(SubKeyName.Buffer);
//         ExFreePool(BaseKeyName.Buffer);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     RtlAppendUnicodeToString(SymbolicLinkName, L"\\??\\");
//     StartIndex = SymbolicLinkName->Length / sizeof(WCHAR);
//     RtlAppendUnicodeStringToString(SymbolicLinkName, InstancePath);
//     for (i = 0; i < InstancePath->Length / sizeof(WCHAR); i++)
//     {
//         if (SymbolicLinkName->Buffer[StartIndex + i] == '\\')
//             SymbolicLinkName->Buffer[StartIndex + i] = '#';
//     }
//     RtlAppendUnicodeToString(SymbolicLinkName, L"#");
//     RtlAppendUnicodeStringToString(SymbolicLinkName, &GuidString);
//     SymbolicLinkName->Buffer[SymbolicLinkName->Length/sizeof(WCHAR)] = L'\0';
//
//     /* Create symbolic link */
//     DPRINT("IoRegisterDeviceInterface(): creating symbolic link %wZ -> %wZ\n", SymbolicLinkName, &PdoNameInfo->Name);
//     SymLinkStatus = IoCreateSymbolicLink(SymbolicLinkName, &PdoNameInfo->Name);
//
//     /* If the symbolic link already exists, return an informational success status */
//     if (SymLinkStatus == STATUS_OBJECT_NAME_COLLISION)
//     {
//         /* HACK: Delete the existing symbolic link and update it to the new PDO name */
//         IoDeleteSymbolicLink(SymbolicLinkName);
//         IoCreateSymbolicLink(SymbolicLinkName, &PdoNameInfo->Name);
//         SymLinkStatus = STATUS_OBJECT_NAME_EXISTS;
//     }
//
//     if (!NT_SUCCESS(SymLinkStatus))
//     {
//         DPRINT1("IoCreateSymbolicLink() failed with status 0x%08lx\n", SymLinkStatus);
//         ZwClose(SubKey);
//         ZwClose(InterfaceKey);
//         ZwClose(ClassKey);
//         ExFreePool(SubKeyName.Buffer);
//         ExFreePool(InterfaceKeyName.Buffer);
//         ExFreePool(BaseKeyName.Buffer);
//         ExFreePool(SymbolicLinkName->Buffer);
//         return SymLinkStatus;
//     }
//
//     if (ReferenceString && ReferenceString->Length)
//     {
//         RtlAppendUnicodeToString(SymbolicLinkName, L"\\");
//         RtlAppendUnicodeStringToString(SymbolicLinkName, ReferenceString);
//     }
//     SymbolicLinkName->Buffer[SymbolicLinkName->Length/sizeof(WCHAR)] = L'\0';
//
//     /* Write symbolic link name in registry */
//     SymbolicLinkName->Buffer[1] = '\\';
//     Status = ZwSetValueKey(
//         SubKey,
//         &SymbolicLink,
//         0, /* TileIndex */
//         REG_SZ,
//         SymbolicLinkName->Buffer,
//         SymbolicLinkName->Length);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwSetValueKey() failed with status 0x%08lx\n", Status);
//         ExFreePool(SymbolicLinkName->Buffer);
//     }
//     else
//     {
//         SymbolicLinkName->Buffer[1] = '?';
//     }
//
//     ZwClose(SubKey);
//     ZwClose(InterfaceKey);
//     ZwClose(ClassKey);
//     ExFreePool(SubKeyName.Buffer);
//     ExFreePool(InterfaceKeyName.Buffer);
//     ExFreePool(BaseKeyName.Buffer);
//
//     return NT_SUCCESS(Status) ? SymLinkStatus : Status;
// }
//
// /*++
//  * @name IoSetDeviceInterfaceState
//  * @implemented
//  *
//  * Enables or disables an instance of a previously registered device
//  * interface class.
//  * Documented in WDK.
//  *
//  * @param SymbolicLinkName
//  *        Pointer to the string identifying instance to enable or disable
//  *
//  * @param Enable
//  *        TRUE = enable, FALSE = disable
//  *
//  * @return Usual NTSTATUS
//  *
//  * @remarks Must be called at IRQL = PASSIVE_LEVEL in the context of a
//  *          system thread
//  *
//  *--*/
// NTSTATUS
// NTAPI
// IoSetDeviceInterfaceState(IN PUNICODE_STRING SymbolicLinkName,
//                           IN BOOLEAN Enable)
// {
//     PDEVICE_OBJECT PhysicalDeviceObject;
//     UNICODE_STRING GuidString;
//     NTSTATUS Status;
//     LPCGUID EventGuid;
//     HANDLE InstanceHandle, ControlHandle;
//     UNICODE_STRING KeyName, DeviceInstance;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     ULONG LinkedValue, Index;
//     GUID DeviceGuid;
//     UNICODE_STRING DosDevicesPrefix1 = RTL_CONSTANT_STRING(L"\\??\\");
//     UNICODE_STRING DosDevicesPrefix2 = RTL_CONSTANT_STRING(L"\\\\?\\");
//     UNICODE_STRING LinkNameNoPrefix;
//     USHORT i;
//     USHORT ReferenceStringOffset;
//
//     if (SymbolicLinkName == NULL)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     DPRINT("IoSetDeviceInterfaceState('%wZ', %u)\n", SymbolicLinkName, Enable);
//
//     /* Symbolic link name is \??\ACPI#PNP0501#1#{GUID}\ReferenceString */
//     /* Make sure it starts with the expected prefix */
//     if (!RtlPrefixUnicodeString(&DosDevicesPrefix1, SymbolicLinkName, FALSE) &&
//         !RtlPrefixUnicodeString(&DosDevicesPrefix2, SymbolicLinkName, FALSE))
//     {
//         DPRINT1("IoSetDeviceInterfaceState() invalid link name '%wZ'\n", SymbolicLinkName);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Make a version without the prefix for further processing */
//     ASSERT(DosDevicesPrefix1.Length == DosDevicesPrefix2.Length);
//     ASSERT(SymbolicLinkName->Length >= DosDevicesPrefix1.Length);
//     LinkNameNoPrefix.Buffer = SymbolicLinkName->Buffer + DosDevicesPrefix1.Length / sizeof(WCHAR);
//     LinkNameNoPrefix.Length = SymbolicLinkName->Length - DosDevicesPrefix1.Length;
//     LinkNameNoPrefix.MaximumLength = LinkNameNoPrefix.Length;
//
//     /* Find the reference string, if any */
//     for (i = 0; i < LinkNameNoPrefix.Length / sizeof(WCHAR); i++)
//     {
//         if (LinkNameNoPrefix.Buffer[i] == L'\\')
//         {
//             break;
//         }
//     }
//     ReferenceStringOffset = i * sizeof(WCHAR);
//
//     /* The GUID is before the reference string or at the end */
//     ASSERT(LinkNameNoPrefix.Length >= ReferenceStringOffset);
//     if (ReferenceStringOffset < GUID_STRING_BYTES + sizeof(WCHAR))
//     {
//         DPRINT1("IoSetDeviceInterfaceState() invalid link name '%wZ'\n", SymbolicLinkName);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     GuidString.Buffer = LinkNameNoPrefix.Buffer + (ReferenceStringOffset - GUID_STRING_BYTES) / sizeof(WCHAR);
//     GuidString.Length = GUID_STRING_BYTES;
//     GuidString.MaximumLength = GuidString.Length;
//     Status = RtlGUIDFromString(&GuidString, &DeviceGuid);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlGUIDFromString() invalid GUID '%wZ' in link name '%wZ'\n", &GuidString, SymbolicLinkName);
//         return Status;
//     }
//
//     /* Open registry keys */
//     Status = OpenRegistryHandlesFromSymbolicLink(SymbolicLinkName,
//                                                  KEY_CREATE_SUB_KEY,
//                                                  NULL,
//                                                  NULL,
//                                                  &InstanceHandle);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     RtlInitUnicodeString(&KeyName, L"Control");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                InstanceHandle,
//                                NULL);
//     Status = ZwCreateKey(&ControlHandle,
//                          KEY_SET_VALUE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          NULL);
//     ZwClose(InstanceHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create the Control subkey\n");
//         return Status;
//     }
//
//     LinkedValue = (Enable ? 1 : 0);
//
//     RtlInitUnicodeString(&KeyName, L"Linked");
//     Status = ZwSetValueKey(ControlHandle,
//                            &KeyName,
//                            0,
//                            REG_DWORD,
//                            &LinkedValue,
//                            sizeof(ULONG));
//     ZwClose(ControlHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to write the Linked value\n");
//         return Status;
//     }
//
//     ASSERT(GuidString.Buffer >= LinkNameNoPrefix.Buffer + 1);
//     DeviceInstance.Length = (USHORT)((GuidString.Buffer - LinkNameNoPrefix.Buffer - 1) * sizeof(WCHAR));
//     if (DeviceInstance.Length == 0)
//     {
//         DPRINT1("No device instance in link name '%wZ'\n", SymbolicLinkName);
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//     DeviceInstance.MaximumLength = DeviceInstance.Length;
//     DeviceInstance.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                   DeviceInstance.MaximumLength,
//                                                   TAG_IO);
//     if (DeviceInstance.Buffer == NULL)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlCopyMemory(DeviceInstance.Buffer,
//                   LinkNameNoPrefix.Buffer,
//                   DeviceInstance.Length);
//
//     for (Index = 0; Index < DeviceInstance.Length / sizeof(WCHAR); Index++)
//     {
//         if (DeviceInstance.Buffer[Index] == L'#')
//         {
//             DeviceInstance.Buffer[Index] = L'\\';
//         }
//     }
//
//     PhysicalDeviceObject = IopGetDeviceObjectFromDeviceInstance(&DeviceInstance);
//
//     if (!PhysicalDeviceObject)
//     {
//         DPRINT1("IopGetDeviceObjectFromDeviceInstance failed to find device object for %wZ\n", &DeviceInstance);
//         ExFreePoolWithTag(DeviceInstance.Buffer, TAG_IO);
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
//     ExFreePoolWithTag(DeviceInstance.Buffer, TAG_IO);
//
//     EventGuid = Enable ? &GUID_DEVICE_INTERFACE_ARRIVAL : &GUID_DEVICE_INTERFACE_REMOVAL;
//
//     PiNotifyDeviceInterfaceChange(EventGuid, &DeviceGuid, SymbolicLinkName);
//     IopQueueDeviceChangeEvent(EventGuid, &DeviceGuid, SymbolicLinkName);
//
//     ObDereferenceObject(PhysicalDeviceObject);
//     DPRINT("Status %x\n", Status);
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
