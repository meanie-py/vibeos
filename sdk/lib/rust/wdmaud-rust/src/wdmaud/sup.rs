use crate::wdmaud::control::KSPROPSETID_Sysaudio;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_004CA9D808C3: [u16; 5] = [0x0025u16, 0x0030, 0x0034, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_1680B17394C4: [u16; 96] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
    0x005C, 0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x005C, 0x007B, 0x0034, 0x0044,
    0x0033, 0x0036, 0x0045, 0x0039, 0x0036, 0x0043, 0x002D, 0x0045, 0x0033, 0x0032,
    0x0035, 0x002D, 0x0031, 0x0031, 0x0043, 0x0045, 0x002D, 0x0042, 0x0046, 0x0043,
    0x0031, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0045, 0x0031,
    0x0030, 0x0033, 0x0031, 0x0038, 0x007D, 0x0000,
];
pub static RUSTCONV_WSTR_9EF518E24D40: [u16; 17] = [
    0x004Du16, 0x0061, 0x0074, 0x0063, 0x0068, 0x0069, 0x006E, 0x0067, 0x0044, 0x0065,
    0x0076, 0x0069, 0x0063, 0x0065, 0x0049, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_BF4CD342C46C: [u16; 11] = [
    0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0044, 0x0065, 0x0073, 0x0063,
    0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{LPWSTR, _OBJECT_ATTRIBUTES, ObReferenceObjectByHandle, PDEVICE_OBJECT, PFILE_OBJECT, PHANDLE, PIRP, PKEY_FULL_INFORMATION, PKEY_VALUE_PARTIAL_INFORMATION, PUNICODE_STRING, RtlInitUnicodeString, ZwClose, ZwOpenKey, ZwQueryKey, ZwQueryValueKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const TAG_WDMAUD: u32 = 0x444d4457 as u32; // 'DMDW' (multi-char constant: 'D'=0x44 'M'=0x4D 'D'=0x44 'W'=0x57, packed big-endian)
#[no_mangle]
pub unsafe extern "C" fn AllocateItem(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T) -> PVOID {
    return ExAllocatePoolZero(PoolType, (NumberOfBytes) as _, TAG_WDMAUD);
}
#[no_mangle]
pub unsafe extern "C" fn FreeItem(Item: PVOID) {
    ExFreePoolWithTag(Item, TAG_WDMAUD as ULONG);
}
#[no_mangle]
pub unsafe extern "C" fn GetSysAudioDeviceCount(mut DeviceObject: PDEVICE_OBJECT) -> ULONG {
    let mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION;
    let mut Pin: KSPROPERTY = core::mem::zeroed();
    let mut Count: ULONG = 0;
    let mut BytesReturned: ULONG = 0;
    let mut Status: NTSTATUS;

    /* setup the query request */
    Pin.Set = KSPROPSETID_Sysaudio;
    Pin.Id = KSPROPERTY_SYSAUDIO_DEVICE_COUNT as ULONG;
    Pin.Flags = KSPROPERTY_TYPE_GET as ULONG;

    DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;

    /* query sysaudio for the device count */
    Status = KsSynchronousIoControlDevice(
        (*DeviceExtension).FileObject,
        0i32 as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY as ULONG,
        &raw mut Pin as PVOID,
        core::mem::size_of::<KSPROPERTY>() as ULONG,
        &raw mut Count as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        &raw mut BytesReturned,
    );
    if !NT_SUCCESS!(Status) {
        return 0 as ULONG;
    }

    return Count;
}
#[no_mangle]
pub unsafe extern "C" fn SetIrpIoStatus(
    mut Irp: PIRP,
    mut Status: NTSTATUS,
    mut Length: ULONG,
) -> NTSTATUS {
    (*Irp).IoStatus.Information = Length as ULONG_PTR;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn ClosePin(
    ClientInfo: PWDMAUD_CLIENT,
    FilterId: ULONG,
    PinId: ULONG,
    DeviceType: SOUND_DEVICE_TYPE,
) -> ULONG {
    let mut Index: ULONG = 0;
    Index = 0 as ULONG;
    while Index < (*ClientInfo).NumPins {
        if (*(*ClientInfo).hPins.offset(Index as isize)).FilterId == FilterId
            && (*(*ClientInfo).hPins.offset(Index as isize)).PinId == PinId
            && !(*(*ClientInfo).hPins.offset(Index as isize))
                .Handle
                .is_null()
            && (*(*ClientInfo).hPins.offset(Index as isize)).Type as ::core::ffi::c_uint
                == DeviceType as ::core::ffi::c_uint
        {
            if (*(*ClientInfo).hPins.offset(Index as isize)).Type as ::core::ffi::c_uint
                != MIXER_DEVICE_TYPE as ::core::ffi::c_uint
            {
                ZwClose((*(*ClientInfo).hPins.offset(Index as isize)).Handle);
            }
            (*(*ClientInfo).hPins.offset(Index as isize)).Handle = core::ptr::null_mut();
            return Index;
        }
        Index = Index.wrapping_add(1);
    }
    return MAXULONG;
}
#[no_mangle]
pub unsafe extern "C" fn InsertPinHandle(
    ClientInfo: PWDMAUD_CLIENT,
    FilterId: ULONG,
    PinId: ULONG,
    DeviceType: SOUND_DEVICE_TYPE,
    PinHandle: HANDLE,
    FreeIndex: ULONG,
) -> NTSTATUS {
    let mut Handles: PWDMAUD_HANDLE = core::ptr::null_mut();

    if FreeIndex != MAXULONG {
        /* re-use a free index */
        (*(*ClientInfo).hPins.add(FreeIndex as usize)).Handle = PinHandle;
        (*(*ClientInfo).hPins.add(FreeIndex as usize)).FilterId = FilterId;
        (*(*ClientInfo).hPins.add(FreeIndex as usize)).PinId = PinId;
        (*(*ClientInfo).hPins.add(FreeIndex as usize)).Type = DeviceType;

        return STATUS_SUCCESS;
    }

    Handles = AllocateItem(
        NonPagedPool,
        (core::mem::size_of::<WDMAUD_HANDLE>() as SIZE_T)
            .wrapping_mul((*ClientInfo).NumPins.wrapping_add(1 as ULONG) as SIZE_T),
    ) as PWDMAUD_HANDLE;

    if Handles.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if (*ClientInfo).NumPins != 0 {
        core::ptr::copy(
            (*ClientInfo).hPins as *const u8,
            Handles as *mut u8,
            core::mem::size_of::<WDMAUD_HANDLE>() * (*ClientInfo).NumPins as usize,
        );
        FreeItem((*ClientInfo).hPins as PVOID);
    }

    (*ClientInfo).hPins = Handles;
    (*(*ClientInfo).hPins.add((*ClientInfo).NumPins as usize)).Handle = PinHandle;
    (*(*ClientInfo).hPins.add((*ClientInfo).NumPins as usize)).Type = DeviceType;
    (*(*ClientInfo).hPins.add((*ClientInfo).NumPins as usize)).FilterId = FilterId;
    (*(*ClientInfo).hPins.add((*ClientInfo).NumPins as usize)).PinId = PinId;
    (*ClientInfo).NumPins += 1;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn ReadKeyValue(
    mut hSubKey: HANDLE,
    mut KeyName: PUNICODE_STRING,
) -> PKEY_VALUE_PARTIAL_INFORMATION {
    let mut Status: NTSTATUS;
    let mut Length: ULONG = core::mem::zeroed();
    let mut PartialInformation: PKEY_VALUE_PARTIAL_INFORMATION;

    /* now query MatchingDeviceId key */
    Status = ZwQueryValueKey(hSubKey, KeyName, KeyValuePartialInformation, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(Length));

    /* check for success */
    if Status != STATUS_BUFFER_TOO_SMALL {
        return core::ptr::null_mut();
    }

    /* allocate a buffer for key data */
    PartialInformation = AllocateItem(NonPagedPool, Length as SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;

    if PartialInformation.is_null() {
        return core::ptr::null_mut();
    }

    /* now query MatchingDeviceId key */
    Status = ZwQueryValueKey(hSubKey, KeyName, KeyValuePartialInformation, PartialInformation as PVOID, Length, core::ptr::addr_of_mut!(Length));

    /* check for success */
    if !NT_SUCCESS!(Status) {
        FreeItem(PartialInformation as PVOID);
        return core::ptr::null_mut();
    }

    if (*PartialInformation).Type != REG_SZ {
        /* invalid key type */
        FreeItem(PartialInformation as PVOID);
        return core::ptr::null_mut();
    }

    return PartialInformation;
}
#[no_mangle]
pub unsafe extern "C" fn CompareProductName(
    hSubKey: HANDLE,
    PnpName: LPWSTR,
    ProductNameSize: ULONG,
    ProductName: LPWSTR,
) -> NTSTATUS {
    let mut PartialInformation: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut DriverDescName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_BF4CD342C46C.as_ptr());
    let mut MatchingDeviceIdName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_9EF518E24D40.as_ptr());
    let mut Length: ULONG = 0;
    let mut DeviceName: LPWSTR = core::ptr::null_mut();

    /* read MatchingDeviceId value */
    PartialInformation = ReadKeyValue(hSubKey, &raw mut MatchingDeviceIdName);

    if PartialInformation.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    /* extract last '&' */
    DeviceName = wcsrchr(
        &raw mut (*PartialInformation).Data as *mut UCHAR as LPWSTR as *const wchar_t,
        (0x0026u16 as i32) as u16,
    ) as LPWSTR;
    ASSERT!(!DeviceName.is_null());
    /* terminate it */
    *DeviceName.offset(0 as isize) = 0x0000u16;

    Length = wcslen(
        &raw mut (*PartialInformation).Data as *mut UCHAR as LPWSTR as *const wchar_t,
    ) as ULONG;

    DPRINT!(
        b"DeviceName %S PnpName %S Length %u\n\0".as_ptr() as *const i8,
        &raw mut (*PartialInformation).Data as *mut UCHAR as LPWSTR,
        PnpName,
        Length
    );

    if _wcsnicmp(
        &raw mut (*PartialInformation).Data as *mut UCHAR as LPWSTR as *const wchar_t,
        PnpName.offset(4 as isize) as *const wchar_t,
        Length as usize,
    ) != 0
    {
        FreeItem(PartialInformation as PVOID);
        return STATUS_NO_MATCH;
    }

    /* free buffer */
    FreeItem(PartialInformation as PVOID);

    /* read DriverDescName value */
    PartialInformation = ReadKeyValue(hSubKey, &raw mut DriverDescName);

    if PartialInformation.is_null() {
        /* failed to read driver desc key */
        return STATUS_UNSUCCESSFUL;
    }

    /* copy key name */
    Length = min(
        ProductNameSize * core::mem::size_of::<WCHAR>() as ULONG,
        (*PartialInformation).DataLength,
    );
    core::ptr::copy(
        &raw mut (*PartialInformation).Data as *mut UCHAR as PVOID as *const u8,
        ProductName as *mut u8,
        Length as usize,
    );

    /* zero terminate it */
    *ProductName.offset(ProductNameSize.wrapping_sub(1) as isize) =
        0x0000u16;

    /* free buffer */
    FreeItem(PartialInformation as PVOID);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn FindProductName(
    mut PnpName: LPWSTR,
    mut ProductNameSize: ULONG,
    mut ProductName: LPWSTR,
) -> NTSTATUS {
    let mut KeyName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_1680B17394C4.as_ptr());

    let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut SubKey: [WCHAR; 20] = [0; 20];
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut hSubKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Length: ULONG;
    let mut Index: ULONG;
    let mut KeyInformation: PKEY_FULL_INFORMATION;

    Index = 0 as ULONG;
    while (Index as usize) < wcslen(PnpName as *const u16) {
        if *PnpName.add(Index as usize) == '#' as u16 {
            *PnpName.add(Index as usize) = 0x005Cu16;
        }
        Index = Index.wrapping_add(1);
    }

    /* initialize key attributes */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut() as HANDLE;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName) as PUNICODE_STRING;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE | OBJ_OPENIF) as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut() as PVOID;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut() as PVOID;

    /* open the key */
    Status = ZwOpenKey(
        core::ptr::addr_of_mut!(hKey),
        GENERIC_READ,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );

    /* check for success */
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    /* query num of subkeys */
    Length = 0;
    Status = ZwQueryKey(
        hKey,
        KeyFullInformation,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(Length),
    );

    if Status != STATUS_BUFFER_TOO_SMALL {
        DPRINT1!("ZwQueryKey failed with %x\n", Status);
        /* failed */
        ZwClose(hKey);
        return Status;
    }

    /* allocate key information struct */
    KeyInformation = AllocateItem(NonPagedPool, Length as SIZE_T) as PKEY_FULL_INFORMATION;
    if KeyInformation.is_null() {
        /* no memory */
        ZwClose(hKey);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* query num of subkeys */
    Status = ZwQueryKey(
        hKey,
        KeyFullInformation,
        KeyInformation as PVOID,
        Length,
        core::ptr::addr_of_mut!(Length),
    );

    if !NT_SUCCESS!(Status) {
        DPRINT1!("ZwQueryKey failed with %x\n", Status);
        FreeItem(KeyInformation as PVOID);
        ZwClose(hKey);
        return Status;
    }

    /* now iterate through all subkeys */
    Index = 0 as ULONG;
    while Index < (*KeyInformation).SubKeys {
        /* subkeys are always in the format 0000-XXXX */
        _swprintf(SubKey.as_mut_ptr(), RUSTCONV_WSTR_004CA9D808C3.as_ptr(), Index);

        /* initialize subkey name */
        RtlInitUnicodeString(core::ptr::addr_of_mut!(SubKeyName), SubKey.as_mut_ptr());

        /* initialize key attributes */
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = hKey;
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(SubKeyName) as PUNICODE_STRING;
        ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE | OBJ_OPENIF) as ULONG;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut() as PVOID;
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut() as PVOID;

        /* open the sub key */
        Status = ZwOpenKey(
            core::ptr::addr_of_mut!(hSubKey),
            GENERIC_READ,
            core::ptr::addr_of_mut!(ObjectAttributes),
        );

        /* check for success */
        if NT_SUCCESS!(Status) {
            /* compare product name */
            Status = CompareProductName(hSubKey, PnpName, ProductNameSize, ProductName);

            /* close subkey */
            ZwClose(hSubKey);

            if NT_SUCCESS!(Status) {
                break;
            }
        }
        Index = Index.wrapping_add(1);
    }

    /* free buffer */
    FreeItem(KeyInformation as PVOID);

    /* close key */
    ZwClose(hKey);

    /* no matching key found */
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn GetSysAudioDevicePnpName(
    DeviceObject: PDEVICE_OBJECT,
    DeviceIndex: ULONG,
    Device: *mut LPWSTR,
) -> NTSTATUS {
    let mut BytesReturned: ULONG = 0;
    let mut Pin: KSP_PIN = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut DeviceExtension: PWDMAUD_DEVICE_EXTENSION = core::ptr::null_mut();

    /* first check if the device index is within bounds */
    if DeviceIndex >= GetSysAudioDeviceCount(DeviceObject) {
        return STATUS_INVALID_PARAMETER;
    }

    /* setup the query request */
    Pin.Property.Set = KSPROPSETID_Sysaudio;
    Pin.Property.Id = KSPROPERTY_SYSAUDIO_DEVICE_INTERFACE_NAME as ULONG;
    Pin.Property.Flags = KSPROPERTY_TYPE_GET as ULONG;
    Pin.PinId = DeviceIndex;

    DeviceExtension = (*DeviceObject).DeviceExtension as PWDMAUD_DEVICE_EXTENSION;

    /* query sysaudio for the device path */
    Status = KsSynchronousIoControlDevice(
        (*DeviceExtension).FileObject,
        0i32 as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY as ULONG,
        &raw mut Pin as PVOID,
        (core::mem::size_of::<KSPROPERTY>() + core::mem::size_of::<ULONG>()) as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
        &raw mut BytesReturned,
    );

    /* check if the request failed */
    if Status != STATUS_BUFFER_TOO_SMALL || BytesReturned == 0 as ULONG {
        return STATUS_UNSUCCESSFUL;
    }

    /* allocate buffer for the device */
    *Device = AllocateItem(NonPagedPool as POOL_TYPE, BytesReturned as SIZE_T) as LPWSTR;
    if Device.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* query sysaudio again for the device path */
    Status = KsSynchronousIoControlDevice(
        (*DeviceExtension).FileObject,
        0i32 as KPROCESSOR_MODE,
        IOCTL_KS_PROPERTY as ULONG,
        &raw mut Pin as PVOID,
        (core::mem::size_of::<KSPROPERTY>() + core::mem::size_of::<ULONG>()) as ULONG,
        *Device as PVOID,
        BytesReturned,
        &raw mut BytesReturned,
    );

    if !NT_SUCCESS!(Status) {
        /* failed */
        FreeItem(*Device as PVOID);
        return Status;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn OpenDevice(
    Device: LPWSTR,
    DeviceHandle: PHANDLE,
    FileObject: *mut PFILE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hDevice: HANDLE = core::ptr::null_mut();

    /* now open the device */
    Status = WdmAudOpenSysAudioDevice(Device, core::ptr::addr_of_mut!(hDevice));

    if !NT_SUCCESS!(Status) {
        return Status;
    }

    *DeviceHandle = hDevice;

    if !FileObject.is_null() {
        Status = ObReferenceObjectByHandle(
            hDevice,
            (FILE_READ_DATA | FILE_WRITE_DATA) as ACCESS_MASK,
            *IoFileObjectType,
            0i32 as KPROCESSOR_MODE,
            FileObject as *mut PVOID,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS!(Status) {
            ZwClose(hDevice);
        }
    }

    return Status;
}
