use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn DeviceIoControl( hDevice: HANDLE, dwIoControlCode: DWORD, lpInBuffer: LPVOID, nInBufferSize: DWORD, lpOutBuffer: LPVOID, nOutBufferSize: DWORD, lpBytesReturned: LPDWORD, lpOverlapped: LPOVERLAPPED, ) -> BOOL;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hDllInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut HidClassGuid: GUID = _GUID {
    Data1: 0x4d1e55b2 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xf16f as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x30 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

// `#define NDEBUG` has no value: nothing to emit.

// The HIDAPI-qualified prototypes (BOOLEAN/VOID/ULONG return types) have no
// function names in the preamble text, so there is nothing to declare here.

#[no_mangle]
pub unsafe extern "system" fn AllocFunction(ItemSize: ULONG) -> PVOID {
    return LocalAlloc(LHND as UINT, ItemSize as SIZE_T) as PVOID;
}
#[no_mangle]
pub unsafe extern "system" fn FreeFunction(Item: PVOID) {
    LocalFree(Item as HLOCAL);
}

#[no_mangle]
pub unsafe extern "system" fn ZeroFunction(Item: PVOID, ItemSize: ULONG) {
    memset(Item, 0, (ItemSize) as _);
}

#[no_mangle]
pub unsafe extern "system" fn CopyFunction(Target: PVOID, Source: PVOID, Length: ULONG) {
    memcpy(Target, Source, (Length) as _);
}
#[no_mangle]
pub unsafe extern "C" fn DebugFunction_v(FormatStr: LPCSTR, ap_list: *mut ::core::ffi::c_void) {}



#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match (dwReason) as i32 {
        DLL_PROCESS_ATTACH => {
            hDllInstance = hinstDLL;
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_FlushQueue(HidDeviceObject: HANDLE) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(HidDeviceObject, IOCTL_HID_FLUSH_QUEUE as DWORD, core::ptr::null_mut(), 0 as DWORD, core::ptr::null_mut(), 0 as DWORD, core::ptr::addr_of_mut!(RetLen), core::ptr::null_mut()) != 0) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_FreePreparsedData(
    mut PreparsedData: PHIDP_PREPARSED_DATA,
) -> BOOLEAN {
    return (LocalFree(PreparsedData as HLOCAL) == core::ptr::null_mut()) as i32 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetAttributes(
    HidDeviceObject: HANDLE,
    Attributes: PHIDD_ATTRIBUTES,
) -> BOOLEAN {
    let mut hci: HID_COLLECTION_INFORMATION = core::mem::zeroed();
    let mut RetLen: DWORD = 0;

    if DeviceIoControl(
        HidDeviceObject,
        IOCTL_HID_GET_COLLECTION_INFORMATION as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        &raw mut hci as LPVOID,
        core::mem::size_of::<HID_COLLECTION_INFORMATION>() as DWORD,
        &raw mut RetLen,
        core::ptr::null_mut(),
    ) == 0
    {
        return FALSE as BOOLEAN;
    }

    /* copy the fields */
    (*Attributes).Size = core::mem::size_of::<HIDD_ATTRIBUTES>() as ULONG;
    (*Attributes).VendorID = hci.VendorID;
    (*Attributes).ProductID = hci.ProductID;
    (*Attributes).VersionNumber = hci.VersionNumber;

    return TRUE as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetFeature(
    HidDeviceObject: HANDLE,
    ReportBuffer: PVOID,
    ReportBufferLength: ULONG,
) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(
        HidDeviceObject,
        (IOCTL_HID_GET_FEATURE) as DWORD,
        core::ptr::null_mut(),
        0,
        ReportBuffer as LPVOID,
        ReportBufferLength as DWORD,
        core::ptr::addr_of_mut!(RetLen),
        core::ptr::null_mut(),
    ) != 0) as ::core::ffi::c_int as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetHidGuid(HidGuid: LPGUID) {
    *HidGuid = HidClassGuid;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetInputReport(
    HidDeviceObject: HANDLE,
    ReportBuffer: PVOID,
    ReportBufferLength: ULONG,
) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(
        HidDeviceObject,
        IOCTL_HID_GET_INPUT_REPORT as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        ReportBuffer as LPVOID,
        ReportBufferLength as DWORD,
        core::ptr::addr_of_mut!(RetLen),
        core::ptr::null_mut(),
    ) != 0) as i32 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetManufacturerString(
    HidDeviceObject: HANDLE,
    Buffer: PVOID,
    BufferLength: ULONG,
) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(
        HidDeviceObject,
        IOCTL_HID_GET_MANUFACTURER_STRING as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        Buffer as LPVOID,
        BufferLength as DWORD,
        core::ptr::addr_of_mut!(RetLen),
        core::ptr::null_mut(),
    ) != 0) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetNumInputBuffers(
    HidDeviceObject: HANDLE,
    NumberBuffers: PULONG,
) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(
        HidDeviceObject,
        IOCTL_GET_NUM_DEVICE_INPUT_BUFFERS as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        NumberBuffers as LPVOID,
        core::mem::size_of::<ULONG>() as DWORD,
        core::ptr::addr_of_mut!(RetLen),
        core::ptr::null_mut(),
    ) != 0) as i32 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetPhysicalDescriptor(
    HidDeviceObject: HANDLE,
    Buffer: PVOID,
    BufferLength: ULONG,
) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(
        HidDeviceObject,
        (IOCTL_GET_PHYSICAL_DESCRIPTOR) as DWORD,
        core::ptr::null_mut(),
        0,
        Buffer as LPVOID,
        BufferLength,
        core::ptr::addr_of_mut!(RetLen),
        core::ptr::null_mut(),
    ) != 0) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetPreparsedData(
    HidDeviceObject: HANDLE,
    PreparsedData: *mut PHIDP_PREPARSED_DATA,
) -> BOOLEAN {
    let mut hci: HID_COLLECTION_INFORMATION = core::mem::zeroed();
    let mut RetLen: DWORD = 0;
    let mut Ret: BOOLEAN = 0;

    if PreparsedData.is_null() {
        return (FALSE) as BOOLEAN;
    }

    if DeviceIoControl(
        HidDeviceObject,
        (IOCTL_HID_GET_COLLECTION_INFORMATION) as DWORD,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(hci) as LPVOID,
        core::mem::size_of::<HID_COLLECTION_INFORMATION>() as DWORD,
        core::ptr::addr_of_mut!(RetLen),
        core::ptr::null_mut(),
    ) == 0
    {
        return (FALSE) as BOOLEAN;
    }

    *PreparsedData = LocalAlloc(LHND, hci.DescriptorSize as SIZE_T) as PHIDP_PREPARSED_DATA;
    if (*PreparsedData).is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return (FALSE) as BOOLEAN;
    }

    Ret = (DeviceIoControl(
        HidDeviceObject,
        (IOCTL_HID_GET_COLLECTION_DESCRIPTOR) as DWORD,
        core::ptr::null_mut(),
        0,
        *PreparsedData as LPVOID,
        hci.DescriptorSize as DWORD,
        core::ptr::addr_of_mut!(RetLen),
        core::ptr::null_mut(),
    ) != 0) as BOOLEAN;

    if Ret == 0 {
        /* FIXME - Free the buffer in case we failed to get the descriptor? */
        LocalFree(*PreparsedData as HLOCAL);
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetProductString(
    HidDeviceObject: HANDLE,
    Buffer: PVOID,
    BufferLength: ULONG,
) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(
        HidDeviceObject,
        (IOCTL_HID_GET_PRODUCT_STRING) as DWORD,
        core::ptr::null_mut(),
        0,
        Buffer as LPVOID,
        BufferLength,
        core::ptr::addr_of_mut!(RetLen),
        core::ptr::null_mut(),
    ) != 0) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetSerialNumberString(
    HidDeviceObject: HANDLE,
    Buffer: PVOID,
    BufferLength: ULONG,
) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(
        HidDeviceObject,
        IOCTL_HID_GET_SERIALNUMBER_STRING as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        Buffer as LPVOID,
        BufferLength as DWORD,
        core::ptr::addr_of_mut!(RetLen),
        core::ptr::null_mut(),
    ) != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_Hello(Buffer: PCHAR, BufferLength: ULONG) -> ULONG {
    let HelloString: [CHAR; 7] = [
        b'H' as CHAR,
        b'e' as CHAR,
        b'l' as CHAR,
        b'l' as CHAR,
        b'o' as CHAR,
        b'\n' as CHAR,
        0 as CHAR,
    ];

    if BufferLength > 0 {
        memcpy(
            Buffer as *mut core::ffi::c_void,
            core::ptr::addr_of!(HelloString) as *const CHAR as *const core::ffi::c_void,
            min!(
                core::mem::size_of::<[CHAR; 7]>() as usize,
                BufferLength as usize,
            ) as usize,
        );
    }

    return core::mem::size_of::<[CHAR; 7]>() as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_SetFeature(
    HidDeviceObject: HANDLE,
    ReportBuffer: PVOID,
    ReportBufferLength: ULONG,
) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(
        HidDeviceObject,
        IOCTL_HID_SET_FEATURE as DWORD,
        ReportBuffer as LPVOID,
        ReportBufferLength as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!(RetLen),
        core::ptr::null_mut(),
    ) != 0) as i32 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_SetNumInputBuffers(HidDeviceObject: HANDLE, mut NumberBuffers: ULONG) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(HidDeviceObject, (IOCTL_SET_NUM_DEVICE_INPUT_BUFFERS) as DWORD, core::ptr::addr_of_mut!(NumberBuffers) as LPVOID, core::mem::size_of::<ULONG>() as DWORD, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(RetLen), core::ptr::null_mut()) != 0) as i32 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_SetOutputReport(
    HidDeviceObject: HANDLE,
    ReportBuffer: PVOID,
    ReportBufferLength: ULONG,
) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(
        HidDeviceObject,
        IOCTL_HID_SET_OUTPUT_REPORT as DWORD,
        ReportBuffer as LPVOID,
        ReportBufferLength as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        &mut RetLen as *mut DWORD as LPDWORD,
        core::ptr::null_mut(),
    ) != 0 as BOOL) as core::ffi::c_int as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetIndexedString(
    HidDeviceObject: HANDLE,
    StringIndex: ULONG,
    Buffer: PVOID,
    BufferLength: ULONG,
) -> BOOLEAN {
    let mut StringIndex = StringIndex;
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(
        HidDeviceObject,
        IOCTL_HID_GET_INDEXED_STRING as DWORD,
        &mut StringIndex as *mut ULONG as LPVOID,
        core::mem::size_of::<ULONG>() as DWORD,
        Buffer as LPVOID,
        BufferLength as DWORD,
        &mut RetLen as *mut DWORD,
        core::ptr::null_mut(),
    ) != 0 as _) as i32 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetMsGenreDescriptor(
    HidDeviceObject: HANDLE,
    Buffer: PVOID,
    BufferLength: ULONG,
) -> BOOLEAN {
    let mut RetLen: DWORD = 0;
    return (DeviceIoControl(HidDeviceObject, (IOCTL_HID_GET_MS_GENRE_DESCRIPTOR) as DWORD,
                         core::ptr::null_mut(), 0,
                         Buffer, BufferLength,
                         core::ptr::addr_of_mut!(RetLen), core::ptr::null_mut()) != 0) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_GetConfiguration(
    HidDeviceObject: HANDLE,
    Configuration: PHIDD_CONFIGURATION,
    ConfigurationLength: ULONG,
) -> BOOLEAN {
    // magic cookie
    (*Configuration).cookie = HidD_GetConfiguration as *const () as PVOID;

    return (DeviceIoControl(
        HidDeviceObject,
        IOCTL_HID_GET_DRIVER_CONFIG as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!((*Configuration).size) as LPVOID,
        ConfigurationLength.wrapping_sub(core::mem::size_of::<ULONG>() as ULONG) as DWORD,
        core::ptr::addr_of_mut!((*Configuration).cookie) as PULONG,
        core::ptr::null_mut(),
    ) != 0 as i32) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HidD_SetConfiguration(
    HidDeviceObject: HANDLE,
    Configuration: PHIDD_CONFIGURATION,
    ConfigurationLength: ULONG,
) -> BOOLEAN {
    let mut Ret: BOOLEAN = (FALSE as u8);

    if (*Configuration).cookie == (HidD_GetConfiguration as *const () as PVOID) {
        Ret = (DeviceIoControl(
            HidDeviceObject,
            (IOCTL_HID_SET_DRIVER_CONFIG) as DWORD,
            core::ptr::null_mut(),
            0,
            &raw mut (*Configuration).size as PVOID,
            ConfigurationLength.wrapping_sub(core::mem::size_of::<ULONG>() as ULONG),
            &raw mut (*Configuration).cookie as PULONG,
            core::ptr::null_mut(),
        ) != 0) as core::ffi::c_int as BOOLEAN;
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
    }

    return Ret;
}
