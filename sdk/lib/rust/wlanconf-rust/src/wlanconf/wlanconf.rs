// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_22DCFAF0828D: [u16; 10] = [0x0053u16, 0x0053, 0x0049, 0x0044, 0x003A, 0x0020, 0x0025, 0x0053, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_4D70F4A881C7: [u16; 2] = [0x003Au16, 0x0000];
pub static RUSTCONV_WSTR_5D8BC069E4F5: [u16; 5] = [0x0025u16, 0x002E, 0x0032, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_6F0DA0ADD46E: [u16; 8] = [0x0042u16, 0x0053, 0x0053, 0x0049, 0x0044, 0x003A, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_845F9467E185: [u16; 4] = [0x0025u16, 0x0075, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_AB03050D95AC: [u16; 13] = [
    0x000Au16, 0x0053, 0x0053, 0x0049, 0x0044, 0x003A, 0x0020, 0x0025, 0x002E, 0x002A,
    0x0053, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_E5979252A780: [u16; 6] = [0x0025u16, 0x0075, 0x002E, 0x0035, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_FE0B1AAD8CFF: [u16; 14] = [
    0x0052u16, 0x0053, 0x0053, 0x0049, 0x003A, 0x0020, 0x0025, 0x0069, 0x0020, 0x0064,
    0x0042, 0x006D, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const NDISUIO_DEVICE_NAME: [::core::ffi::c_ushort; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_ushort; 13]>(
        *b"\\\0\\\0.\0\\\0\\\0N\0d\0i\0s\0u\0i\0o\0\0\0",
    )
};

#[no_mangle]
pub static mut bScan: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut bDisconnect: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut bConnect: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut bAdhoc: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut sSsid: PCWSTR = ::core::ptr::null::<WCHAR>();

#[no_mangle]
pub static mut sWepKey: PCWSTR = ::core::ptr::null::<WCHAR>();
#[no_mangle]
pub unsafe extern "C" fn DoFormatMessage(ErrorCode: DWORD) {
    if ErrorCode == ERROR_SUCCESS {
        return;
    }
    ConMsgPuts(
        StdErr,
        FORMAT_MESSAGE_FROM_SYSTEM,
        core::ptr::null_mut(),
        ErrorCode,
        LANG_USER_DEFAULT,
    );
}
#[no_mangle]
pub unsafe extern "C" fn OpenDriverHandle() -> HANDLE {
    let mut hDriver: HANDLE = core::ptr::null_mut();
    let mut dwBytesReturned: DWORD = 0;
    let mut bSuccess: BOOL = 0;

    /* Open a handle to the NDISUIO driver */
    hDriver = CreateFileW((NDISUIO_DEVICE_NAME).as_ptr(),
                          GENERIC_READ | GENERIC_WRITE,
                          0,
                          core::ptr::null_mut(),
                          OPEN_EXISTING,
                          FILE_ATTRIBUTE_NORMAL,
                          core::ptr::null_mut());
    if hDriver == INVALID_HANDLE_VALUE {
        return INVALID_HANDLE_VALUE;
    }

    /* Wait for binds */
    bSuccess = DeviceIoControl(hDriver,
                               (IOCTL_NDISUIO_BIND_WAIT as u32),
                               core::ptr::null_mut(),
                               0,
                               core::ptr::null_mut(),
                               0,
                               core::ptr::addr_of_mut!(dwBytesReturned),
                               core::ptr::null_mut());
    if bSuccess == 0 {
        CloseHandle(hDriver);
        return INVALID_HANDLE_VALUE;
    }

    return hDriver;
}
#[no_mangle]
pub unsafe extern "C" fn IsWlanAdapter(mut hAdapter: HANDLE) -> BOOL {
    let mut bSuccess: BOOL = 0;
    let mut dwBytesReturned: DWORD = 0;
    let mut QueryOid: NDISUIO_QUERY_OID = _NDISUIO_QUERY_OID {
        Oid: 0,
        Data: [0; 4],
    };
    /* WLAN drivers must support this OID */
    QueryOid.Oid = OID_GEN_PHYSICAL_MEDIUM as NDIS_OID;
    bSuccess = DeviceIoControl(
        hAdapter,
        IOCTL_NDISUIO_QUERY_OID_VALUE as DWORD,
        &raw mut QueryOid as LPVOID,
        ::core::mem::size_of::<NDISUIO_QUERY_OID>() as DWORD,
        &raw mut QueryOid as LPVOID,
        ::core::mem::size_of::<NDISUIO_QUERY_OID>() as DWORD,
        &raw mut dwBytesReturned,
        ::core::ptr::null_mut::<_OVERLAPPED>(),
    );
    if bSuccess == 0
        || *(&raw mut QueryOid.Data as *mut UCHAR as PULONG)
            != NdisPhysicalMediumWirelessLan as ::core::ffi::c_int as ULONG
    {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn OpenAdapterHandle(
    mut Index: DWORD,
    mut hAdapter: *mut HANDLE,
    mut IpInfo: *mut IP_ADAPTER_INDEX_MAP,
) -> BOOL {
    let mut hDriver: HANDLE = core::ptr::null_mut();
    let mut bSuccess: BOOL = 0;
    let mut dwBytesReturned: DWORD = 0;
    let QueryBindingSize: DWORD = (core::mem::size_of::<NDISUIO_QUERY_BINDING>()
        + (1024 as usize) * core::mem::size_of::<WCHAR>()) as DWORD;
    let mut QueryBinding: PNDISUIO_QUERY_BINDING = core::ptr::null_mut();
    let mut dwStatus: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut i: LONG = 0;
    let mut InterfaceInfo: PIP_INTERFACE_INFO = core::ptr::null_mut();

    /* Open the driver handle */
    hDriver = OpenDriverHandle();
    if hDriver == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }

    /* Allocate the binding struct */
    QueryBinding = HeapAlloc(
        GetProcessHeap(),
        0,
        QueryBindingSize as SIZE_T,
    ) as PNDISUIO_QUERY_BINDING;
    if QueryBinding.is_null() {
        CloseHandle(hDriver);
        return (FALSE) as BOOL;
    }

    /* Query the adapter binding information */
    (*QueryBinding).BindingIndex = Index as ULONG;
    bSuccess = DeviceIoControl(
        hDriver,
        IOCTL_NDISUIO_QUERY_BINDING as DWORD,
        QueryBinding as LPVOID,
        QueryBindingSize,
        QueryBinding as LPVOID,
        QueryBindingSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );

    if bSuccess == 0 {
        HeapFree(GetProcessHeap(), 0, QueryBinding as PVOID);
        CloseHandle(hDriver);
        return (FALSE) as BOOL;
    }

    /* Bind to the adapter */
    bSuccess = DeviceIoControl(
        hDriver,
        IOCTL_NDISUIO_OPEN_DEVICE as DWORD,
        (QueryBinding as PUCHAR).add((*QueryBinding).DeviceNameOffset as usize) as LPVOID,
        (*QueryBinding).DeviceNameLength as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );

    if bSuccess == 0 {
        HeapFree(GetProcessHeap(), 0, QueryBinding as PVOID);
        CloseHandle(hDriver);
        return (FALSE) as BOOL;
    }

    /* Get interface info from the IP helper */
    dwSize = core::mem::size_of::<IP_INTERFACE_INFO>() as DWORD;
    loop {
        if !InterfaceInfo.is_null() {
            HeapFree(GetProcessHeap(), 0, InterfaceInfo as PVOID);
        }
        InterfaceInfo = HeapAlloc(
            GetProcessHeap(),
            0,
            dwSize as SIZE_T,
        ) as PIP_INTERFACE_INFO;
        if InterfaceInfo.is_null() {
            HeapFree(GetProcessHeap(), 0, QueryBinding as PVOID);
            CloseHandle(hDriver);
            return (FALSE) as BOOL;
        }
        dwStatus = GetInterfaceInfo(InterfaceInfo, core::ptr::addr_of_mut!(dwSize));
        if !(dwStatus == ERROR_INSUFFICIENT_BUFFER) {
            break;
        }
    }

    if dwStatus != NO_ERROR {
        HeapFree(GetProcessHeap(), 0, QueryBinding as PVOID);
        HeapFree(GetProcessHeap(), 0, InterfaceInfo as PVOID);
        CloseHandle(hDriver);
        return (FALSE) as BOOL;
    }

    i = 0 as LONG;
    while i < (*InterfaceInfo).NumAdapters {
        let AdapterPtr: *mut IP_ADAPTER_INDEX_MAP =
            core::ptr::addr_of_mut!((*InterfaceInfo).Adapter) as *mut IP_ADAPTER_INDEX_MAP;
        let InterfaceGuid: PWCHAR = wcschr(
            core::ptr::addr_of_mut!((*AdapterPtr.offset(i as isize)).Name) as *mut WCHAR,
            0x007Bu16,
        ) as PWCHAR;

        if InterfaceGuid.is_null() {
            i += 1;
            continue;
        }

        if !wcsstr(
            (QueryBinding as PUCHAR).add((*QueryBinding).DeviceNameOffset as usize) as PWCHAR,
            InterfaceGuid,
        )
        .is_null()
        {
            *IpInfo = *AdapterPtr.offset(i as isize);
            *hAdapter = hDriver;

            HeapFree(GetProcessHeap(), 0, QueryBinding as PVOID);
            HeapFree(GetProcessHeap(), 0, InterfaceInfo as PVOID);

            return (TRUE) as BOOL;
        }
        i += 1;
    }

    HeapFree(GetProcessHeap(), 0, QueryBinding as PVOID);
    HeapFree(GetProcessHeap(), 0, InterfaceInfo as PVOID);
    CloseHandle(hDriver);

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn OpenWlanAdapter(hAdapter: *mut HANDLE, IpInfo: *mut IP_ADAPTER_INDEX_MAP) -> BOOL {
    let mut dwCurrentIndex: DWORD = 0;
    loop {
        if OpenAdapterHandle(dwCurrentIndex, hAdapter, IpInfo) == 0 {
            break;
        }

        if IsWlanAdapter(*hAdapter) != 0 {
            return (TRUE) as BOOL;
        } else {
            CloseHandle(*hAdapter);
        }
        dwCurrentIndex = dwCurrentIndex.wrapping_add(1);
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn WlanDisconnect(hAdapter: HANDLE, IpInfo: PIP_ADAPTER_INDEX_MAP) -> BOOL {
    let mut bSuccess: BOOL = 0;
    let mut dwBytesReturned: DWORD = 0;
    let mut SetOid: NDISUIO_SET_OID = core::mem::zeroed();

    /* Release this IP address */
    IpReleaseAddress(IpInfo);

    /* Disassociate from the AP */
    SetOid.Oid = OID_802_11_DISASSOCIATE as NDIS_OID;
    bSuccess = DeviceIoControl(
        hAdapter,
        IOCTL_NDISUIO_SET_OID_VALUE as DWORD,
        &raw mut SetOid as LPVOID,
        FIELD_OFFSET!(NDISUIO_SET_OID, Data) as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        &raw mut dwBytesReturned,
        core::ptr::null_mut(),
    );
    if bSuccess == 0 {
        return (FALSE) as BOOL;
    }

    ConResPuts(&raw mut csStdOut, IDS_SUCCESS as UINT);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn CharToHex(Char: WCHAR) -> UCHAR {
    if (Char >= 0x0030u16) && (Char <= 0x0039u16) {
        return (Char as i32 - 0x0030u16 as i32) as UCHAR;
    }

    if (Char >= 0x0061u16) && (Char <= 0x0066u16) {
        return (Char as i32 - 0x0061u16 as i32 + 10 as i32) as UCHAR;
    }

    if (Char >= 0x0041u16) && (Char <= 0x0046u16) {
        return (Char as i32 - 0x0041u16 as i32 + 10 as i32) as UCHAR;
    }

    return 0 as UCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn WlanPrintCurrentStatus(hAdapter: HANDLE) -> BOOL {
    let mut QueryOid: PNDISUIO_QUERY_OID = core::ptr::null_mut();
    let mut QueryOidSize: DWORD = 0;
    let mut bSuccess: BOOL = 0;
    let mut dwBytesReturned: DWORD = 0;
    let mut SsidInfo: PNDIS_802_11_SSID = core::ptr::null_mut();
    let mut SsidBuffer: [CHAR; (NDIS_802_11_LENGTH_SSID + 1) as usize] =
        [0; (NDIS_802_11_LENGTH_SSID + 1) as usize];
    let mut i: DWORD = 0;
    let mut szMsgBuf: [WCHAR; 128 as usize] =
        [0; 128 as usize];

    QueryOidSize = (core::mem::offset_of!(NDISUIO_QUERY_OID, Data)
        + core::mem::size_of::<NDIS_802_11_SSID>()) as DWORD;
    QueryOid = HeapAlloc(GetProcessHeap(), 0, QueryOidSize as SIZE_T)
        as PNDISUIO_QUERY_OID;
    if QueryOid.is_null() {
        return (FALSE) as BOOL;
    }

    (*QueryOid).Oid = OID_802_11_SSID;
    SsidInfo = core::ptr::addr_of_mut!((*QueryOid).Data) as *mut UCHAR as PNDIS_802_11_SSID;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_QUERY_OID_VALUE as u32),
        QueryOid as LPVOID,
        QueryOidSize,
        QueryOid as LPVOID,
        QueryOidSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if bSuccess == 0 {
        HeapFree(GetProcessHeap(), 0, QueryOid as PVOID);
        return (FALSE) as BOOL;
    }

    /* Copy the SSID to our internal buffer and terminate it */
    RtlCopyMemory!(
        core::ptr::addr_of_mut!(SsidBuffer) as *mut CHAR as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*SsidInfo).Ssid) as *const core::ffi::c_void,
        (*SsidInfo).SsidLength as usize,
    );
    SsidBuffer[(*SsidInfo).SsidLength as usize] = 0;

    HeapFree(GetProcessHeap(), 0, QueryOid as PVOID);
    QueryOidSize = (core::mem::offset_of!(NDISUIO_QUERY_OID, Data)
        + core::mem::size_of::<NDIS_802_11_MAC_ADDRESS>()) as DWORD;
    QueryOid = HeapAlloc(GetProcessHeap(), 0, QueryOidSize as SIZE_T)
        as PNDISUIO_QUERY_OID;
    if QueryOid.is_null() {
        return (FALSE) as BOOL;
    }

    (*QueryOid).Oid = OID_802_11_BSSID;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_QUERY_OID_VALUE as u32),
        QueryOid as LPVOID,
        QueryOidSize,
        QueryOid as LPVOID,
        QueryOidSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if (*SsidInfo).SsidLength == 0 || bSuccess == 0 {
        ConResPuts(StdOut, IDS_WLAN_DISCONNECT);
        HeapFree(GetProcessHeap(), 0, QueryOid as PVOID);
        return (TRUE) as BOOL;
    } else {
        ConResPuts(StdOut, IDS_MSG_CURRENT_WIRELESS);
    }

    ConPrintf(
        StdOut,
        RUSTCONV_WSTR_22DCFAF0828D.as_ptr(),
        core::ptr::addr_of_mut!(SsidBuffer) as *mut CHAR,
    );

    ConPuts(StdOut, RUSTCONV_WSTR_6F0DA0ADD46E.as_ptr());
    i = 0;
    while (i as usize) < core::mem::size_of::<NDIS_802_11_MAC_ADDRESS>() {
        let mut BssidData: UINT = (*QueryOid).Data[i as usize] as UINT;

        ConPrintf(StdOut, RUSTCONV_WSTR_5D8BC069E4F5.as_ptr(), BssidData);

        if (i as usize)
            != core::mem::size_of::<NDIS_802_11_MAC_ADDRESS>().wrapping_sub(1 as usize)
        {
            ConPuts(StdOut, RUSTCONV_WSTR_4D70F4A881C7.as_ptr());
        }
        i = i.wrapping_add(1);
    }
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    HeapFree(GetProcessHeap(), 0, QueryOid as PVOID);
    QueryOidSize = core::mem::size_of::<NDISUIO_QUERY_OID>() as DWORD;
    QueryOid = HeapAlloc(GetProcessHeap(), 0, QueryOidSize as SIZE_T)
        as PNDISUIO_QUERY_OID;
    if QueryOid.is_null() {
        return (FALSE) as BOOL;
    }

    (*QueryOid).Oid = OID_802_11_INFRASTRUCTURE_MODE;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_QUERY_OID_VALUE as u32),
        QueryOid as LPVOID,
        QueryOidSize,
        QueryOid as LPVOID,
        QueryOidSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if bSuccess == 0 {
        HeapFree(GetProcessHeap(), 0, QueryOid as PVOID);
        return (FALSE) as BOOL;
    }

    LoadStringW(
        core::ptr::null_mut(),
        (if *(core::ptr::addr_of_mut!((*QueryOid).Data) as *mut UCHAR as PUINT) == Ndis802_11IBSS {
            IDS_ADHOC
        } else {
            IDS_INFRASTRUCTURE
        }),
        core::ptr::addr_of_mut!(szMsgBuf) as *mut WCHAR,
        (ARRAYSIZE!(szMsgBuf)) as i32,
    );
    ConResPrintf(StdOut, IDS_MSG_NETWORK_MODE, core::ptr::addr_of_mut!(szMsgBuf) as *mut WCHAR);

    (*QueryOid).Oid = OID_802_11_WEP_STATUS;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_QUERY_OID_VALUE as u32),
        QueryOid as LPVOID,
        QueryOidSize,
        QueryOid as LPVOID,
        QueryOidSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if bSuccess == 0 {
        HeapFree(GetProcessHeap(), 0, QueryOid as PVOID);
        return (FALSE) as BOOL;
    }

    LoadStringW(
        core::ptr::null_mut(),
        (if *(core::ptr::addr_of_mut!((*QueryOid).Data) as *mut UCHAR as PUINT) == Ndis802_11WEPEnabled {
            IDS_YES
        } else {
            IDS_NO
        }),
        core::ptr::addr_of_mut!(szMsgBuf) as *mut WCHAR,
        (ARRAYSIZE!(szMsgBuf)) as i32,
    );
    ConResPrintf(StdOut, IDS_MSG_WEP_ENABLED, core::ptr::addr_of_mut!(szMsgBuf) as *mut WCHAR);

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    (*QueryOid).Oid = OID_802_11_RSSI;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_QUERY_OID_VALUE as u32),
        QueryOid as LPVOID,
        QueryOidSize,
        QueryOid as LPVOID,
        QueryOidSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if bSuccess != 0 {
        /* This OID is optional */
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_FE0B1AAD8CFF.as_ptr(),
            *(core::ptr::addr_of_mut!((*QueryOid).Data) as *mut UCHAR as PINT),
        );
    }

    (*QueryOid).Oid = OID_802_11_TX_POWER_LEVEL;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_QUERY_OID_VALUE as u32),
        QueryOid as LPVOID,
        QueryOidSize,
        QueryOid as LPVOID,
        QueryOidSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if bSuccess != 0 {
        /* This OID is optional */
        ConResPrintf(
            StdOut,
            IDS_MSG_TRANSMISSION_POWER,
            *(core::ptr::addr_of_mut!((*QueryOid).Data) as *mut UCHAR as PUINT),
        );
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    (*QueryOid).Oid = OID_802_11_NUMBER_OF_ANTENNAS;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_QUERY_OID_VALUE as u32),
        QueryOid as LPVOID,
        QueryOidSize,
        QueryOid as LPVOID,
        QueryOidSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if bSuccess != 0 {
        /* This OID is optional */
        ConResPrintf(
            StdOut,
            IDS_MSG_ANTENNA_COUNT,
            *(core::ptr::addr_of_mut!((*QueryOid).Data) as *mut UCHAR as PUINT),
        );
    }

    (*QueryOid).Oid = OID_802_11_TX_ANTENNA_SELECTED;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_QUERY_OID_VALUE as u32),
        QueryOid as LPVOID,
        QueryOidSize,
        QueryOid as LPVOID,
        QueryOidSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if bSuccess != 0 {
        let mut TransmitAntenna: UINT = *(core::ptr::addr_of_mut!((*QueryOid).Data) as *mut UCHAR as PUINT);

        if TransmitAntenna != 0xFFFFFFFF {
            ConResPrintf(StdOut, IDS_MSG_TRANSMIT_ANTENNA, TransmitAntenna);
        } else {
            ConResPuts(StdOut, IDS_MSG_TRANSMIT_ANTENNA_ANY);
        }
    }

    (*QueryOid).Oid = OID_802_11_RX_ANTENNA_SELECTED;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_QUERY_OID_VALUE as u32),
        QueryOid as LPVOID,
        QueryOidSize,
        QueryOid as LPVOID,
        QueryOidSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if bSuccess != 0 {
        let mut ReceiveAntenna: UINT = *(core::ptr::addr_of_mut!((*QueryOid).Data) as *mut UCHAR as PUINT);

        if ReceiveAntenna != 0xFFFFFFFF {
            ConResPrintf(StdOut, IDS_MSG_RECEIVE_ANTENNA, ReceiveAntenna);
        } else {
            ConResPuts(StdOut, IDS_MSG_RECEIVE_ANTENNA_ANY);
        }
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    (*QueryOid).Oid = OID_802_11_FRAGMENTATION_THRESHOLD;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_QUERY_OID_VALUE as u32),
        QueryOid as LPVOID,
        QueryOidSize,
        QueryOid as LPVOID,
        QueryOidSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if bSuccess != 0 {
        /* This OID is optional */
        ConResPrintf(
            StdOut,
            IDS_MSG_FRAGMENT_THRESHOLD,
            *(core::ptr::addr_of_mut!((*QueryOid).Data) as *mut UCHAR as PUINT),
        );
    }

    (*QueryOid).Oid = OID_802_11_RTS_THRESHOLD;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_QUERY_OID_VALUE as u32),
        QueryOid as LPVOID,
        QueryOidSize,
        QueryOid as LPVOID,
        QueryOidSize,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if bSuccess != 0 {
        /* This OID is optional */
        ConResPrintf(
            StdOut,
            IDS_MSG_RTS_THRESHOLD,
            *(core::ptr::addr_of_mut!((*QueryOid).Data) as *mut UCHAR as PUINT),
        );
    }

    HeapFree(GetProcessHeap(), 0, QueryOid as PVOID);

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn WlanConnect(hAdapter: HANDLE) -> BOOL {
    let mut bSuccess: BOOL;
    let mut dwBytesReturned: DWORD = core::mem::zeroed();
    let mut SetOidSize: DWORD;
    let mut SetOid: PNDISUIO_SET_OID;
    let mut Ssid: PNDIS_802_11_SSID;
    let mut StrLength: DWORD;
    let mut i: DWORD;
    let mut SsidBuffer: [CHAR; (NDIS_802_11_LENGTH_SSID + 1) as usize] =
        [0; (NDIS_802_11_LENGTH_SSID + 1) as usize];

    SetOidSize = core::mem::size_of::<NDISUIO_SET_OID>() as DWORD;
    SetOid = HeapAlloc(
        GetProcessHeap(),
        0,
        SetOidSize as SIZE_T,
    ) as PNDISUIO_SET_OID;
    if SetOid.is_null() {
        return (FALSE) as BOOL;
    }

    /* Set the network mode */
    (*SetOid).Oid = OID_802_11_INFRASTRUCTURE_MODE;
    *(&raw mut (*SetOid).Data as *mut UCHAR as PULONG) = (if bAdhoc != 0 {
        Ndis802_11IBSS
    } else {
        Ndis802_11Infrastructure
    }) as ULONG;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_SET_OID_VALUE as u32),
        SetOid as LPVOID,
        SetOidSize,
        core::ptr::null_mut(),
        0,
        &raw mut dwBytesReturned,
        core::ptr::null_mut(),
    );
    if bSuccess == 0 {
        HeapFree(GetProcessHeap(), 0, SetOid as PVOID);
        return (FALSE) as BOOL;
    }

    /* Set the authentication mode */
    (*SetOid).Oid = OID_802_11_AUTHENTICATION_MODE;
    *(&raw mut (*SetOid).Data as *mut UCHAR as PULONG) = (if !sWepKey.is_null() {
        Ndis802_11AuthModeShared
    } else {
        Ndis802_11AuthModeOpen
    }) as ULONG;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_SET_OID_VALUE as u32),
        SetOid as LPVOID,
        SetOidSize,
        core::ptr::null_mut(),
        0,
        &raw mut dwBytesReturned,
        core::ptr::null_mut(),
    );
    if bSuccess == 0 {
        HeapFree(GetProcessHeap(), 0, SetOid as PVOID);
        return (FALSE) as BOOL;
    }

    if !sWepKey.is_null() {
        let mut WepData: PNDIS_802_11_WEP;

        HeapFree(GetProcessHeap(), 0, SetOid as PVOID);

        StrLength = wcslen(sWepKey as *const wchar_t) as DWORD >> 1; // 2 string characters represent 1 hex byte.
        SetOidSize = core::mem::offset_of!(NDISUIO_SET_OID, Data) as DWORD
            + core::mem::offset_of!(NDIS_802_11_WEP, KeyMaterial) as DWORD
            + StrLength;
        SetOid = HeapAlloc(
            GetProcessHeap(),
            0,
            SetOidSize as SIZE_T,
        ) as PNDISUIO_SET_OID;
        if SetOid.is_null() {
            return (FALSE) as BOOL;
        }

        /* Add the WEP key */
        (*SetOid).Oid = OID_802_11_ADD_WEP;
        WepData = &raw mut (*SetOid).Data as *mut UCHAR as PNDIS_802_11_WEP;

        (*WepData).KeyIndex = 0x80000000;
        (*WepData).KeyLength = StrLength as ULONG;
        (*WepData).Length = (core::mem::offset_of!(NDIS_802_11_WEP, KeyMaterial) as DWORD + StrLength) as ULONG;

        /* Assemble the hex key */
        i = 0;
        while *sWepKey.offset((i << 1) as isize) != '\0' as WCHAR
        {
            *(&raw mut (*WepData).KeyMaterial as *mut UCHAR).offset(i as isize) =
                ((CharToHex(*sWepKey.offset((i << 1) as isize)) as i32)
                    << 4) as UCHAR;
            let pkm: *mut UCHAR = (&raw mut (*WepData).KeyMaterial as *mut UCHAR).offset(i as isize);
            *pkm = (*pkm as i32
                | CharToHex(
                    *sWepKey.offset(((i << 1) + 1) as isize),
                ) as i32) as UCHAR;
            i = i.wrapping_add(1);
        }

        bSuccess = DeviceIoControl(
            hAdapter,
            (IOCTL_NDISUIO_SET_OID_VALUE as u32),
            SetOid as LPVOID,
            SetOidSize,
            core::ptr::null_mut(),
            0,
            &raw mut dwBytesReturned,
            core::ptr::null_mut(),
        );
        if bSuccess == 0 {
            HeapFree(GetProcessHeap(), 0, SetOid as PVOID);
            return (FALSE) as BOOL;
        }
    }

    /* Set the encryption status */
    (*SetOid).Oid = OID_802_11_WEP_STATUS;
    *(&raw mut (*SetOid).Data as *mut UCHAR as PULONG) = (if !sWepKey.is_null() {
        Ndis802_11WEPEnabled
    } else {
        Ndis802_11WEPDisabled
    }) as ULONG;

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_SET_OID_VALUE as u32),
        SetOid as LPVOID,
        SetOidSize,
        core::ptr::null_mut(),
        0,
        &raw mut dwBytesReturned,
        core::ptr::null_mut(),
    );
    if bSuccess == 0 {
        HeapFree(GetProcessHeap(), 0, SetOid as PVOID);
        return (FALSE) as BOOL;
    }

    HeapFree(GetProcessHeap(), 0, SetOid as PVOID);
    SetOidSize = core::mem::offset_of!(NDISUIO_SET_OID, Data) as DWORD
        + core::mem::size_of::<NDIS_802_11_MAC_ADDRESS>() as DWORD;
    SetOid = HeapAlloc(
        GetProcessHeap(),
        0,
        SetOidSize as SIZE_T,
    ) as PNDISUIO_SET_OID;
    if SetOid.is_null() {
        return (FALSE) as BOOL;
    }

    /* Set the BSSID */
    (*SetOid).Oid = OID_802_11_BSSID;
    core::ptr::write_bytes(
        &raw mut (*SetOid).Data as *mut UCHAR,
        0xFF as u8,
        core::mem::size_of::<NDIS_802_11_MAC_ADDRESS>(),
    );

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_SET_OID_VALUE as u32),
        SetOid as LPVOID,
        SetOidSize,
        core::ptr::null_mut(),
        0,
        &raw mut dwBytesReturned,
        core::ptr::null_mut(),
    );
    if bSuccess == 0 {
        HeapFree(GetProcessHeap(), 0, SetOid as PVOID);
        return (FALSE) as BOOL;
    }

    HeapFree(GetProcessHeap(), 0, SetOid as PVOID);
    SetOidSize = core::mem::offset_of!(NDISUIO_SET_OID, Data) as DWORD
        + core::mem::size_of::<NDIS_802_11_SSID>() as DWORD;
    SetOid = HeapAlloc(
        GetProcessHeap(),
        0,
        SetOidSize as SIZE_T,
    ) as PNDISUIO_SET_OID;
    if SetOid.is_null() {
        return (FALSE) as BOOL;
    }

    /* Finally, set the SSID */
    (*SetOid).Oid = OID_802_11_SSID;
    Ssid = &raw mut (*SetOid).Data as *mut UCHAR as PNDIS_802_11_SSID;

    snprintf(
        &raw mut SsidBuffer as *mut CHAR,
        core::mem::size_of::<[CHAR; (NDIS_802_11_LENGTH_SSID + 1) as usize]>(),
        b"%S\0".as_ptr() as *const CHAR,
        sSsid,
    );
    StrLength = strlen(&raw mut SsidBuffer as *mut CHAR) as DWORD;
    (*Ssid).SsidLength = StrLength as ULONG;
    core::ptr::copy_nonoverlapping(
        &raw mut SsidBuffer as *mut CHAR as *const UCHAR,
        &raw mut (*Ssid).Ssid as *mut UCHAR,
        StrLength as usize,
    );

    bSuccess = DeviceIoControl(
        hAdapter,
        (IOCTL_NDISUIO_SET_OID_VALUE as u32),
        SetOid as LPVOID,
        SetOidSize,
        core::ptr::null_mut(),
        0,
        &raw mut dwBytesReturned,
        core::ptr::null_mut(),
    );

    HeapFree(GetProcessHeap(), 0, SetOid as PVOID);

    if bSuccess == 0 {
        return (FALSE) as BOOL;
    }

    ConResPuts(StdOut, IDS_SUCCESS);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn WlanScan(hAdapter: HANDLE) -> BOOL {
    let mut bSuccess: BOOL;
    let mut dwBytesReturned: DWORD = core::mem::zeroed();
    let mut SetOid: NDISUIO_SET_OID = core::mem::zeroed();
    let mut QueryOid: PNDISUIO_QUERY_OID;
    let mut QueryOidSize: DWORD;
    let mut BssidList: PNDIS_802_11_BSSID_LIST;
    let mut i: DWORD;
    let mut j: DWORD;
    let mut dwNetworkCount: DWORD;
    let mut szMsgBuf: [WCHAR; 128] = [0; 128];

    SetOid.Oid = OID_802_11_BSSID_LIST_SCAN as NDIS_OID;

    /* Send the scan OID */
    bSuccess = DeviceIoControl(
        hAdapter,
        IOCTL_NDISUIO_SET_OID_VALUE as DWORD,
        core::ptr::addr_of_mut!(SetOid) as LPVOID,
        FIELD_OFFSET!(NDISUIO_SET_OID, Data) as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
    if bSuccess == 0 {
        return (FALSE) as BOOL;
    }

    /* Wait 2 seconds for the scan to return some results */
    Sleep(2000 as DWORD);

    /* Allocate space for 10 networks to be returned initially */
    QueryOid = core::ptr::null_mut();
    dwNetworkCount = 10 as DWORD;
    loop {
        if !QueryOid.is_null() {
            HeapFree(GetProcessHeap(), 0 as DWORD, QueryOid as PVOID);
        }

        QueryOidSize = (core::mem::size_of::<NDISUIO_QUERY_OID>() as DWORD).wrapping_add(
            (core::mem::size_of::<NDIS_WLAN_BSSID>() as DWORD).wrapping_mul(dwNetworkCount),
        );
        QueryOid = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            QueryOidSize as SIZE_T,
        ) as PNDISUIO_QUERY_OID;
        if QueryOid.is_null() {
            return (FALSE) as BOOL;
        }

        (*QueryOid).Oid = OID_802_11_BSSID_LIST as NDIS_OID;
        BssidList = core::ptr::addr_of_mut!((*QueryOid).Data) as *mut UCHAR as PNDIS_802_11_BSSID_LIST;

        bSuccess = DeviceIoControl(
            hAdapter,
            IOCTL_NDISUIO_QUERY_OID_VALUE as DWORD,
            QueryOid as LPVOID,
            QueryOidSize,
            QueryOid as LPVOID,
            QueryOidSize,
            core::ptr::addr_of_mut!(dwBytesReturned),
            core::ptr::null_mut(),
        );
        if !(bSuccess == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER as DWORD) {
            break;
        }
        /* Try allocating space for 10 more networks */
        dwNetworkCount = dwNetworkCount.wrapping_add(10 as DWORD);
    }

    if bSuccess == 0 {
        HeapFree(GetProcessHeap(), 0 as DWORD, QueryOid as PVOID);
        return (FALSE) as BOOL;
    }

    if (*BssidList).NumberOfItems == 0 as ULONG {
        ConResPuts(StdOut, IDS_NO_NETWORK as UINT);
    } else {
        let mut BssidInfo: PNDIS_WLAN_BSSID = core::ptr::addr_of_mut!((*BssidList).Bssid) as PNDIS_WLAN_BSSID;
        i = 0 as DWORD;
        while i < (*BssidList).NumberOfItems {
            let mut Ssid: PNDIS_802_11_SSID = core::ptr::addr_of_mut!((*BssidInfo).Ssid);
            let mut Rssi: NDIS_802_11_RSSI = (*BssidInfo).Rssi;
            let mut NetworkType: NDIS_802_11_NETWORK_INFRASTRUCTURE = (*BssidInfo).InfrastructureMode;
            let mut Rate: UINT;

            /* The SSID member is a non-NUL terminated ASCII string */
            ConPrintf(
                StdOut,
                RUSTCONV_WSTR_AB03050D95AC.as_ptr(),
                (*Ssid).SsidLength,
                core::ptr::addr_of_mut!((*Ssid).Ssid) as *mut UCHAR,
            );

            ConPuts(StdOut, RUSTCONV_WSTR_6F0DA0ADD46E.as_ptr());
            j = 0 as DWORD;
            while (j as usize) < core::mem::size_of::<NDIS_802_11_MAC_ADDRESS>() {
                let mut BssidData: UINT = (*BssidInfo).MacAddress[j as usize] as UINT;

                ConPrintf(StdOut, RUSTCONV_WSTR_5D8BC069E4F5.as_ptr(), BssidData);

                if (j as usize) != (core::mem::size_of::<NDIS_802_11_MAC_ADDRESS>() - 1 as usize) {
                    ConPuts(StdOut, RUSTCONV_WSTR_4D70F4A881C7.as_ptr());
                }
                j = j.wrapping_add(1);
            }
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

            LoadStringW(
                core::ptr::null_mut(),
                if (*BssidInfo).Privacy == 0 as ULONG {
                    IDS_NO
                } else {
                    IDS_YES
                } as UINT,
                szMsgBuf.as_mut_ptr(),
                (ARRAYSIZE!(szMsgBuf)) as i32,
            );
            ConResPrintf(StdOut, IDS_MSG_ENCRYPTED as UINT, szMsgBuf.as_mut_ptr());
            LoadStringW(
                core::ptr::null_mut(),
                if NetworkType == Ndis802_11IBSS as NDIS_802_11_NETWORK_INFRASTRUCTURE {
                    IDS_ADHOC
                } else {
                    IDS_INFRASTRUCTURE
                } as UINT,
                szMsgBuf.as_mut_ptr(),
                (ARRAYSIZE!(szMsgBuf)) as i32,
            );
            ConResPrintf(StdOut, IDS_MSG_NETWORK_TYPE as UINT, szMsgBuf.as_mut_ptr());
            ConResPrintf(StdOut, IDS_MSG_RSSI as UINT, Rssi as core::ffi::c_int);
            ConResPuts(StdOut, IDS_MSG_SUPPORT_RATE as UINT);

            j = 0 as DWORD;
            while j < NDIS_802_11_LENGTH_RATES as DWORD {
                Rate = (*BssidInfo).SupportedRates[j as usize] as UINT;
                if Rate != 0 as UINT {
                    /* Bit 7 is the basic rates bit */
                    Rate = Rate & 0x7F as UINT;

                    /* SupportedRates are in units of .5 */
                    if Rate & 0x01 as UINT != 0 {
                        /* Bit 0 is set so we need to add 0.5 */
                        ConPrintf(StdOut, RUSTCONV_WSTR_E5979252A780.as_ptr(), Rate >> 1);
                    } else {
                        /* Bit 0 is clear so just print the conversion */
                        ConPrintf(StdOut, RUSTCONV_WSTR_845F9467E185.as_ptr(), Rate >> 1);
                    }
                }
                j = j.wrapping_add(1);
            }
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

            /* Move to the next entry */
            BssidInfo = (BssidInfo as PUCHAR).add((*BssidInfo).Length as usize) as PNDIS_WLAN_BSSID;
            i = i.wrapping_add(1);
        }
    }

    HeapFree(GetProcessHeap(), 0 as DWORD, QueryOid as PVOID);

    return bSuccess;
}
#[no_mangle]
pub unsafe extern "C" fn ParseCmdline(argc: ::core::ffi::c_int, argv: *mut *mut WCHAR) -> BOOL {
    let mut i: INT = 1;
    while i < argc {
        if *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Du16 {
            match *(*argv.offset(i as isize)).offset(1 as isize) {
                0x0073u16 => {
                    bScan = (TRUE as i32);
                }
                0x0064u16 => {
                    bDisconnect = (TRUE as i32);
                }
                0x0063u16 => {
                    if i == argc - 1 {
                        ConResPuts(StdOut, IDS_USAGE);
                        return (FALSE) as BOOL;
                    }
                    bConnect = (TRUE as i32);
                    i += 1;
                    sSsid = *argv.offset(i as isize) as PCWSTR;
                }
                0x0077u16 => {
                    if i == argc - 1 {
                        ConResPuts(StdOut, IDS_USAGE);
                        return (FALSE) as BOOL;
                    }
                    i += 1;
                    sWepKey = *argv.offset(i as isize) as PCWSTR;
                }
                0x0061u16 => {
                    bAdhoc = (TRUE as i32);
                }
                _ => {
                    ConResPuts(StdOut, IDS_USAGE);
                    return (FALSE) as BOOL;
                }
            }
        } else {
            ConResPuts(StdOut, IDS_USAGE);
            return (FALSE) as BOOL;
        }
        i += 1;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(mut argc: i32, mut argv: *mut *mut WCHAR) -> i32 {
    let mut hAdapter: HANDLE = core::ptr::null_mut();
    let mut IpInfo: IP_ADAPTER_INDEX_MAP = core::mem::zeroed();

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    if !(ParseCmdline(argc, argv) != 0) {
        return -1;
    }

    if !(OpenWlanAdapter(core::ptr::addr_of_mut!(hAdapter), core::ptr::addr_of_mut!(IpInfo)) != 0) {
        ConResPuts(StdOut, IDS_NO_WLAN_ADAPTER);
        return -1;
    }

    if bScan != 0 {
        if !(WlanScan(hAdapter) != 0) {
            DoFormatMessage(GetLastError());
            CloseHandle(hAdapter);
            return -1;
        }
    } else if bDisconnect != 0 {
        if !(WlanDisconnect(hAdapter, core::ptr::addr_of_mut!(IpInfo)) != 0) {
            DoFormatMessage(GetLastError());
            CloseHandle(hAdapter);
            return -1;
        }
    } else if bConnect != 0 {
        if !(WlanConnect(hAdapter) != 0) {
            DoFormatMessage(GetLastError());
            CloseHandle(hAdapter);
            return -1;
        }
    } else {
        if !(WlanPrintCurrentStatus(hAdapter) != 0) {
            DoFormatMessage(GetLastError());
            CloseHandle(hAdapter);
            return -1;
        }
    }

    CloseHandle(hAdapter);
    return 0;
}
