// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_32434DC5B0F7: [u16; 2] = [0x002Cu16, 0x0000];
pub static RUSTCONV_WSTR_408607DCA0FF: [u16; 20] = [
    0x0025u16, 0x006C, 0x0075, 0x0020, 0x0025, 0x0033, 0x0073, 0x0020, 0x0025, 0x0033,
    0x0073, 0x0020, 0x0025, 0x0033, 0x0073, 0x0020, 0x0025, 0x0033, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_4D70F4A881C7: [u16; 2] = [0x003Au16, 0x0000];
pub static RUSTCONV_WSTR_7AF58FDC50E8: [u16; 51] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0050, 0x006F, 0x0072, 0x0074, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_C1D34563AF12: [u16; 9] = [0x0050u16, 0x006F, 0x0072, 0x0074, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_EC477EC46A3E: [u16; 13] = [
    0x0025u16, 0x006C, 0x0075, 0x002C, 0x0025, 0x0073, 0x002C, 0x0025, 0x0073, 0x002C,
    0x0025, 0x0073, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_3F56473D402A: [u16; 2] = [0x006Eu16, 0x0000];
pub static RUSTCONV_C2R_WSTR_52E9E3ADD38B: [u16; 2] = [0x0073u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_60864AAE2645: [u16; 2] = [0x0031u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_66A0B53312C1: [u16; 2] = [0x0078u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_6855B5C2B40B: [u16; 2] = [0x0036u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_6FEA016A651B: [u16; 2] = [0x0038u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_77F906B94309: [u16; 2] = [0x0034u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_8432386E2C3B: [u16; 2] = [0x0035u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_85DD751867E3: [u16; 2] = [0x0032u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_8FC6CB4A9816: [u16; 2] = [0x0065u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_9D04B389D66E: [u16; 4] = [0x0031u16, 0x002E, 0x0035, 0x0000];
pub static RUSTCONV_C2R_WSTR_CB4B9DBBEB77: [u16; 2] = [0x006Fu16, 0x0000];
pub static RUSTCONV_C2R_WSTR_D0FCC72CCEEA: [u16; 2] = [0x006Du16, 0x0000];
pub static RUSTCONV_C2R_WSTR_D3FE97979D0F: [u16; 2] = [0x0070u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_E2733B3DB9D9: [u16; 2] = [0x0037u16, 0x0000];

#[no_mangle]
pub static mut BaudRates: [DWORD; 18] = [
    75 as ::core::ffi::c_int as DWORD,
    110 as ::core::ffi::c_int as DWORD,
    134 as ::core::ffi::c_int as DWORD,
    150 as ::core::ffi::c_int as DWORD,
    300 as ::core::ffi::c_int as DWORD,
    600 as ::core::ffi::c_int as DWORD,
    1200 as ::core::ffi::c_int as DWORD,
    1800 as ::core::ffi::c_int as DWORD,
    2400 as ::core::ffi::c_int as DWORD,
    4800 as ::core::ffi::c_int as DWORD,
    7200 as ::core::ffi::c_int as DWORD,
    9600 as ::core::ffi::c_int as DWORD,
    14400 as ::core::ffi::c_int as DWORD,
    19200 as ::core::ffi::c_int as DWORD,
    38400 as ::core::ffi::c_int as DWORD,
    57600 as ::core::ffi::c_int as DWORD,
    115200 as ::core::ffi::c_int as DWORD,
    128000 as ::core::ffi::c_int as DWORD,
];

#[no_mangle]
pub static mut Paritys: [PWSTR; 5] = unsafe {
    [
        RUSTCONV_C2R_WSTR_8FC6CB4A9816.as_ptr()
            as PWSTR,
        RUSTCONV_C2R_WSTR_CB4B9DBBEB77.as_ptr()
            as PWSTR,
        RUSTCONV_C2R_WSTR_3F56473D402A.as_ptr()
            as PWSTR,
        RUSTCONV_C2R_WSTR_D0FCC72CCEEA.as_ptr()
            as PWSTR,
        RUSTCONV_C2R_WSTR_52E9E3ADD38B.as_ptr()
            as PWSTR,
    ]
};

#[no_mangle]
pub static mut DataBits: [PWSTR; 5] = unsafe {
    [
        RUSTCONV_C2R_WSTR_77F906B94309.as_ptr()
            as PWSTR,
        RUSTCONV_C2R_WSTR_8432386E2C3B.as_ptr()
            as PWSTR,
        RUSTCONV_C2R_WSTR_6855B5C2B40B.as_ptr()
            as PWSTR,
        RUSTCONV_C2R_WSTR_E2733B3DB9D9.as_ptr()
            as PWSTR,
        RUSTCONV_C2R_WSTR_6FEA016A651B.as_ptr()
            as PWSTR,
    ]
};

#[no_mangle]
pub static mut StopBits: [PWSTR; 3] = unsafe {
    [
        RUSTCONV_C2R_WSTR_60864AAE2645.as_ptr()
            as PWSTR,
        RUSTCONV_C2R_WSTR_9D04B389D66E.as_ptr()
            as PWSTR,
        RUSTCONV_C2R_WSTR_85DD751867E3.as_ptr()
            as PWSTR,
    ]
};

#[no_mangle]
pub static mut FlowControls: [PWSTR; 2] = unsafe {
    [
        RUSTCONV_C2R_WSTR_66A0B53312C1.as_ptr()
            as PWSTR,
        RUSTCONV_C2R_WSTR_D3FE97979D0F.as_ptr()
            as PWSTR,
    ]
};

#[repr(C)]
pub struct _PORT_DATA {
    pub DeviceInfoSet: HDEVINFO,
    pub DeviceInfoData: PSP_DEVINFO_DATA,
    pub szPortName: [WCHAR; 16],
    pub nBaudRateIndex: INT,
    pub nParityIndex: INT,
    pub nDataBitsIndex: INT,
    pub nStopBitsIndex: INT,
    pub nFlowControlIndex: INT,
    pub bChanged: BOOL,
}
pub type PORT_DATA = _PORT_DATA;
pub type PPORT_DATA = *mut _PORT_DATA;

pub const DEFAULT_BAUD_RATE_INDEX: i32 = 11;
pub const DEFAULT_DATA_BITS_INDEX: i32 = 4;
pub const DEFAULT_PARITY_INDEX: i32 = 2;
pub const DEFAULT_STOP_BITS_INDEX: i32 = 0;
pub const DEFAULT_FLOW_CONTROL_INDEX: i32 = 2;

unsafe extern "C" fn FillComboBox(hwnd: HWND, szBuffer: PWSTR) {
    let mut pStart: PWSTR;
    let mut pEnd: PWSTR;

    pStart = szBuffer;
    loop {
        pEnd = wcschr(pStart, 0x002Cu16) as PWSTR;
        if !pEnd.is_null() {
            *pEnd = UNICODE_NULL as WCHAR;
        }

        ComboBox_AddString!(hwnd, pStart);

        if pEnd.is_null() {
            break;
        }

        pStart = pEnd.offset(1 as isize);
    }
}

unsafe extern "C" fn ReadPortSettings(pPortData: PPORT_DATA) {
    let mut szPortData: [WCHAR; 32] = [0; 32];
    let mut szParity: [WCHAR; 4] = [0; 4];
    let mut szDataBits: [WCHAR; 4] = [0; 4];
    let mut szStopBits: [WCHAR; 4] = [0; 4];
    let mut szFlowControl: [WCHAR; 4] = [0; 4];
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut dwBaudRate: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut n: INT = 0;
    let mut i: INT = 0;
    let mut lError: LONG = 0;

    (*pPortData).nBaudRateIndex = DEFAULT_BAUD_RATE_INDEX;
    (*pPortData).nParityIndex = DEFAULT_PARITY_INDEX;
    (*pPortData).nDataBitsIndex = DEFAULT_DATA_BITS_INDEX;
    (*pPortData).nStopBitsIndex = DEFAULT_STOP_BITS_INDEX;
    (*pPortData).nFlowControlIndex = DEFAULT_FLOW_CONTROL_INDEX;
    (*pPortData).bChanged = (FALSE as i32);

    hKey = SetupDiOpenDevRegKey(
        (*pPortData).DeviceInfoSet,
        (*pPortData).DeviceInfoData,
        DICS_FLAG_GLOBAL,
        0,
        DIREG_DEV,
        KEY_READ,
    );
    if hKey == (INVALID_HANDLE_VALUE).cast() {
        ERR!("SetupDiOpenDevRegKey() failed\n");
        return;
    }

    dwSize = core::mem::size_of::<[WCHAR; 16]>() as DWORD;
    lError = RegQueryValueExW(
        hKey,
        RUSTCONV_WSTR_C1D34563AF12.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*pPortData).szPortName) as PBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );
    RegCloseKey(hKey);

    if lError != (ERROR_SUCCESS as i32) {
        ERR!("RegQueryValueExW failed (Error %lu)\n", lError);
        return;
    }

    wcscat(
        core::ptr::addr_of_mut!((*pPortData).szPortName) as *mut WCHAR,
        RUSTCONV_WSTR_4D70F4A881C7.as_ptr(),
    );

    lError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_7AF58FDC50E8.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    );
    if lError != (ERROR_SUCCESS as i32) {
        ERR!("RegOpenKeyExW failed (Error %lu)\n", lError);
        return;
    }

    dwSize = core::mem::size_of::<[WCHAR; 32]>() as DWORD;
    lError = RegQueryValueExW(
        hKey,
        core::ptr::addr_of_mut!((*pPortData).szPortName) as *mut WCHAR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(szPortData) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );
    RegCloseKey(hKey);

    if lError != (ERROR_SUCCESS as i32) {
        ERR!("RegQueryValueExW failed (Error %lu)\n", lError);
        return;
    }

    if (dwType != REG_SZ)
        || (dwSize > core::mem::size_of::<[WCHAR; 32]>() as DWORD)
    {
        ERR!("Wrong type or size\n");
        return;
    }

    /* Replace commas by spaces */
    i = 0;
    while (i as usize) < (wcslen(addr_of_mut!(szPortData) as *mut WCHAR) as usize) {
        if szPortData[i as usize] == 0x002Cu16 {
            szPortData[i as usize] = 0x0020u16;
        }
        i += 1;
    }

    /* Parse the port settings */
    n = swscanf(
        core::ptr::addr_of_mut!(szPortData) as *mut WCHAR,
        RUSTCONV_WSTR_408607DCA0FF.as_ptr(),
        core::ptr::addr_of_mut!(dwBaudRate),
        core::ptr::addr_of_mut!(szParity) as *mut WCHAR,
        core::ptr::addr_of_mut!(szDataBits) as *mut WCHAR,
        core::ptr::addr_of_mut!(szStopBits) as *mut WCHAR,
        core::ptr::addr_of_mut!(szFlowControl) as *mut WCHAR,
    ) as INT;

    if n > 0 {
        i = 0;
        while i < ARRAYSIZE!(BaudRates) as INT {
            if dwBaudRate == BaudRates[i as usize] {
                (*pPortData).nBaudRateIndex = i;
            }
            i += 1;
        }
    }

    if n > 1 {
        i = 0;
        while i < ARRAYSIZE!(Paritys) as INT {
            if _wcsicmp(
                core::ptr::addr_of_mut!(szParity) as *mut WCHAR,
                Paritys[i as usize],
            ) == 0
            {
                (*pPortData).nParityIndex = i;
            }
            i += 1;
        }
    }

    if n > 2 {
        i = 0;
        while i < ARRAYSIZE!(DataBits) as INT {
            if _wcsicmp(
                core::ptr::addr_of_mut!(szDataBits) as *mut WCHAR,
                DataBits[i as usize],
            ) == 0
            {
                (*pPortData).nDataBitsIndex = i;
            }
            i += 1;
        }
    }

    if n > 3 {
        i = 0;
        while i < ARRAYSIZE!(StopBits) as INT {
            if _wcsicmp(
                core::ptr::addr_of_mut!(szStopBits) as *mut WCHAR,
                StopBits[i as usize],
            ) == 0
            {
                (*pPortData).nStopBitsIndex = i;
            }
            i += 1;
        }
    }

    if n > 4 {
        i = 0;
        while i < ARRAYSIZE!(FlowControls) as INT {
            if _wcsicmp(
                core::ptr::addr_of_mut!(szFlowControl) as *mut WCHAR,
                FlowControls[i as usize],
            ) == 0
            {
                (*pPortData).nFlowControlIndex = i;
            }
            i += 1;
        }
    }
}

unsafe extern "C" fn WritePortSettings(hwnd: HWND, pPortData: PPORT_DATA) {
    let mut PropChangeParams: SP_PROPCHANGE_PARAMS = core::mem::zeroed();
    let mut szPortData: [WCHAR; 32] = [0; 32];
    let mut hwndControl: HWND = core::ptr::null_mut();
    let mut nBaudRateIndex: INT;
    let mut nDataBitsIndex: INT;
    let mut nParityIndex: INT;
    let mut nStopBitsIndex: INT;
    let mut nFlowControlIndex: INT;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG;

    if (*pPortData).bChanged == (FALSE as i32)
    {
        return;
    }

    nBaudRateIndex = (*pPortData).nBaudRateIndex;
    nDataBitsIndex = (*pPortData).nDataBitsIndex;
    nParityIndex = (*pPortData).nParityIndex;
    nStopBitsIndex = (*pPortData).nStopBitsIndex;
    nFlowControlIndex = (*pPortData).nFlowControlIndex;

    hwndControl = GetDlgItem(hwnd, (IDC_SERIAL_BITSPERSECOND as i32));
    if !hwndControl.is_null()
    {
        nBaudRateIndex = ComboBox_GetCurSel!(hwndControl);
    }

    hwndControl = GetDlgItem(hwnd, (IDC_SERIAL_DATABITS as i32));
    if !hwndControl.is_null()
    {
        nDataBitsIndex = ComboBox_GetCurSel!(hwndControl);
    }

    hwndControl = GetDlgItem(hwnd, (IDC_SERIAL_PARITY as i32));
    if !hwndControl.is_null()
    {
        nParityIndex = ComboBox_GetCurSel!(hwndControl);
    }

    hwndControl = GetDlgItem(hwnd, (IDC_SERIAL_STOPBITS as i32));
    if !hwndControl.is_null()
    {
        nStopBitsIndex = ComboBox_GetCurSel!(hwndControl);
    }

    hwndControl = GetDlgItem(hwnd, (IDC_SERIAL_FLOWCONTROL as i32));
    if !hwndControl.is_null()
    {
        nFlowControlIndex = ComboBox_GetCurSel!(hwndControl);
    }

    _swprintf(
        core::ptr::addr_of_mut!(szPortData) as *mut WCHAR,
        RUSTCONV_WSTR_EC477EC46A3E.as_ptr(),
        BaudRates[nBaudRateIndex as usize],
        Paritys[nParityIndex as usize],
        DataBits[nDataBitsIndex as usize],
        StopBits[nStopBitsIndex as usize]
    );
    if nFlowControlIndex < 2
    {
        wcscat(
            core::ptr::addr_of_mut!(szPortData) as *mut WCHAR,
            RUSTCONV_WSTR_32434DC5B0F7.as_ptr()
        );
        wcscat(
            core::ptr::addr_of_mut!(szPortData) as *mut WCHAR,
            FlowControls[nFlowControlIndex as usize]
        );
    }

    lError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_7AF58FDC50E8.as_ptr(),
        0,
        KEY_WRITE,
        core::ptr::addr_of_mut!(hKey)
    );
    if lError != (ERROR_SUCCESS as i32)
    {
        ERR!("RegOpenKeyExW failed (Error %lu)\n", lError);
        return;
    }

    lError = RegSetValueExW(
        hKey,
        core::ptr::addr_of_mut!((*pPortData).szPortName) as *mut WCHAR as LPCWSTR,
        0,
        REG_SZ,
        core::ptr::addr_of_mut!(szPortData) as *mut WCHAR as LPBYTE,
        ((wcslen(core::ptr::addr_of_mut!(szPortData) as *mut WCHAR) as usize
            + 1 as usize)
            * core::mem::size_of::<WCHAR>()) as DWORD
    );

    RegCloseKey(hKey);

    if lError != (ERROR_SUCCESS as i32)
    {
        ERR!("RegSetValueExW failed (Error %lu)\n", lError);
        return;
    }

    /* Notify the system */
    PostMessageW(
        HWND_BROADCAST,
        WM_WININICHANGE,
        0,
        core::ptr::addr_of_mut!((*pPortData).szPortName) as *mut WCHAR as LPARAM
    );

    /* Notify the installer (and device) */
    PropChangeParams.ClassInstallHeader.cbSize = (core::mem::size_of::<SP_CLASSINSTALL_HEADER>()) as _;
    PropChangeParams.ClassInstallHeader.InstallFunction = DIF_PROPERTYCHANGE;
    PropChangeParams.Scope = DICS_FLAG_GLOBAL;
    PropChangeParams.StateChange = DICS_PROPCHANGE;

    SetupDiSetClassInstallParams(
        (*pPortData).DeviceInfoSet,
        (*pPortData).DeviceInfoData,
        core::ptr::addr_of_mut!(PropChangeParams) as PSP_CLASSINSTALL_HEADER,
        (core::mem::size_of::<SP_PROPCHANGE_PARAMS>()) as _
    );

    SetupDiCallClassInstaller(
        DIF_PROPERTYCHANGE,
        (*pPortData).DeviceInfoSet,
        (*pPortData).DeviceInfoData
    );
}

unsafe extern "C" fn OnInitDialog(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) -> BOOL {
    let mut pPortData: PPORT_DATA;
    let mut szBuffer: [WCHAR; 256] = [0; 256];
    let mut i: UINT;
    let mut hwndControl: HWND;

    pPortData = (*(lParam as LPPROPSHEETPAGEW)).lParam as PPORT_DATA;
    if pPortData.is_null() {
        ERR!("pPortData is NULL\n");
        return (FALSE) as BOOL;
    }

    SetWindowLongPtr(hwnd, DWLP_USER as i32, pPortData as LONG_PTR);

    /* Read and parse the port settings */
    ReadPortSettings(pPortData);

    /* Fill the 'Bits per second' combobox */
    hwndControl = GetDlgItem(hwnd, IDC_SERIAL_BITSPERSECOND as i32);
    if !hwndControl.is_null() {
        i = 0 as UINT;
        while i < ARRAYSIZE!(BaudRates) as UINT {
            _ultow(
                BaudRates[i as usize] as _,
                szBuffer.as_mut_ptr(),
                10 as i32,
            );
            ComboBox_AddString!(hwndControl, szBuffer.as_mut_ptr());
            i += 1;
        }

        ComboBox_SetCurSel!(hwndControl, (*pPortData).nBaudRateIndex as WPARAM);
    }

    /* Fill the 'Data bits' combobox */
    hwndControl = GetDlgItem(hwnd, IDC_SERIAL_DATABITS as i32);
    if !hwndControl.is_null() {
        i = 0 as UINT;
        while i < ARRAYSIZE!(DataBits) as UINT {
            ComboBox_AddString!(hwndControl, DataBits[i as usize]);
            i += 1;
        }

        ComboBox_SetCurSel!(hwndControl, (*pPortData).nDataBitsIndex as WPARAM);
    }

    /* Fill the 'Parity' combobox */
    LoadStringW(
        hInstance,
        IDS_PARITY as UINT,
        szBuffer.as_mut_ptr(),
        ARRAYSIZE!(szBuffer) as i32,
    );

    hwndControl = GetDlgItem(hwnd, IDC_SERIAL_PARITY as i32);
    if !hwndControl.is_null() {
        FillComboBox(hwndControl, szBuffer.as_mut_ptr());
        ComboBox_SetCurSel!(hwndControl, (*pPortData).nParityIndex as WPARAM);
    }

    /* Fill the 'Stop bits' combobox */
    LoadStringW(
        hInstance,
        IDS_STOPBITS as UINT,
        szBuffer.as_mut_ptr(),
        ARRAYSIZE!(szBuffer) as i32,
    );

    hwndControl = GetDlgItem(hwnd, IDC_SERIAL_STOPBITS as i32);
    if !hwndControl.is_null() {
        FillComboBox(hwndControl, szBuffer.as_mut_ptr());
        ComboBox_SetCurSel!(hwndControl, (*pPortData).nStopBitsIndex as WPARAM);
    }

    /* Fill the 'Flow control' combobox */
    LoadStringW(
        hInstance,
        IDS_FLOWCONTROL as UINT,
        szBuffer.as_mut_ptr(),
        ARRAYSIZE!(szBuffer) as i32,
    );

    hwndControl = GetDlgItem(hwnd, IDC_SERIAL_FLOWCONTROL as i32);
    if !hwndControl.is_null() {
        FillComboBox(hwndControl, szBuffer.as_mut_ptr());
        ComboBox_SetCurSel!(hwndControl, (*pPortData).nFlowControlIndex as WPARAM);
    }

    /* Disable the 'Advanced' button */
    hwndControl = GetDlgItem(hwnd, IDC_SERIAL_ADVANCED as i32);
    if !hwndControl.is_null() {
        EnableWindow(hwndControl, FALSE as i32);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn RestoreDefaultValues(hwnd: HWND, pPortData: PPORT_DATA) {
    let mut hwndControl: HWND;

    /* Reset the 'Bits per second' combobox */
    hwndControl = GetDlgItem(hwnd, (IDC_SERIAL_BITSPERSECOND as i32));
    if !hwndControl.is_null() {
        ComboBox_SetCurSel!(hwndControl, DEFAULT_BAUD_RATE_INDEX);
    }

    /* Reset the 'Data bits' combobox */
    hwndControl = GetDlgItem(hwnd, (IDC_SERIAL_DATABITS as i32));
    if !hwndControl.is_null() {
        ComboBox_SetCurSel!(hwndControl, DEFAULT_DATA_BITS_INDEX);
    }

    /* Reset the 'Parity' combobox */
    hwndControl = GetDlgItem(hwnd, (IDC_SERIAL_PARITY as i32));
    if !hwndControl.is_null() {
        ComboBox_SetCurSel!(hwndControl, DEFAULT_PARITY_INDEX);
    }

    /* Reset the 'Stop bits' combobox */
    hwndControl = GetDlgItem(hwnd, (IDC_SERIAL_STOPBITS as i32));
    if !hwndControl.is_null() {
        ComboBox_SetCurSel!(hwndControl, DEFAULT_STOP_BITS_INDEX);
    }

    /* Reset the 'Flow control' combobox */
    hwndControl = GetDlgItem(hwnd, (IDC_SERIAL_FLOWCONTROL as i32));
    if !hwndControl.is_null() {
        ComboBox_SetCurSel!(hwndControl, DEFAULT_FLOW_CONTROL_INDEX);
    }

    (*pPortData).bChanged = (TRUE as i32);
}

unsafe extern "C" fn OnCommand(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) {
    let mut pPortData: PPORT_DATA;

    pPortData = GetWindowLongPtr(hwnd, DWLP_USER) as PPORT_DATA;
    if pPortData.is_null() {
        ERR!("pPortData is NULL\n");
        return;
    }

    match (LOWORD!(wParam) as i32) as u32 {
        IDC_SERIAL_BITSPERSECOND
        | IDC_SERIAL_DATABITS
        | IDC_SERIAL_PARITY
        | IDC_SERIAL_STOPBITS
        | IDC_SERIAL_FLOWCONTROL => {
            if HIWORD!(wParam) as i32 == (CBN_SELCHANGE as i32)
                || HIWORD!(wParam) as i32 == (CBN_EDITCHANGE as i32)
            {
                (*pPortData).bChanged = (TRUE as i32);
            }
        }

        IDC_SERIAL_RESTORE => {
            RestoreDefaultValues(hwnd, pPortData);
        }

        _ => {}
    }
}

unsafe extern "C" fn OnNotify(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) {
    let mut pPortData: PPORT_DATA;

    pPortData = GetWindowLongPtr(hwnd, DWLP_USER) as PPORT_DATA;
    if pPortData.is_null() {
        ERR!("pPortData is NULL\n");
        return;
    }

    if (*(lParam as LPNMHDR)).code == (PSN_APPLY as UINT) {
        WritePortSettings(hwnd, pPortData);
    }
}

unsafe extern "C" fn OnDestroy(hwnd: HWND) {
    let mut pPortData: PPORT_DATA;

    pPortData = GetWindowLongPtr(hwnd, DWLP_USER as i32) as PPORT_DATA;
    if pPortData.is_null() {
        ERR!("pPortData is NULL\n");
        return;
    }

    HeapFree(GetProcessHeap(), 0 as DWORD, pPortData as PVOID);
    SetWindowLongPtr(hwnd, DWLP_USER as i32, 0 as LONG_PTR);
}

unsafe extern "system" fn SerialSettingsDlgProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => return OnInitDialog(hwnd, wParam, lParam) as INT_PTR,
        WM_COMMAND => {
            OnCommand(hwnd, wParam, lParam);
        }
        WM_NOTIFY => {
            OnNotify(hwnd, wParam, lParam);
        }
        WM_DESTROY => {
            OnDestroy(hwnd);
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn SerialPortPropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    let mut PropSheetPage: PROPSHEETPAGEW = core::mem::zeroed();
    let mut hPropSheetPage: HPROPSHEETPAGE = core::ptr::null_mut();
    let mut pPortData: PPORT_DATA;

    pPortData = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<PORT_DATA>() as SIZE_T,
    ) as PPORT_DATA;
    if pPortData.is_null() {
        ERR!("Port data allocation failed!\n");
        return (FALSE) as BOOL;
    }

    (*pPortData).DeviceInfoSet = (*lpPropSheetPageRequest).DeviceInfoSet;
    (*pPortData).DeviceInfoData = (*lpPropSheetPageRequest).DeviceInfoData;

    if (*lpPropSheetPageRequest).PageRequested == SPPSR_ENUM_ADV_DEVICE_PROPERTIES as DWORD {
        PropSheetPage.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
        PropSheetPage.dwFlags = 0 as DWORD;
        PropSheetPage.hInstance = hInstance;
        PropSheetPage.__bindgen_anon_1.pszTemplate =
            MAKEINTRESOURCE!(IDD_SERIALSETTINGS) as LPCWSTR;
        PropSheetPage.pfnDlgProc = Some(SerialSettingsDlgProc);
        PropSheetPage.lParam = pPortData as LPARAM;
        PropSheetPage.pfnCallback = None;

        hPropSheetPage = CreatePropertySheetPageW(
            core::ptr::addr_of_mut!(PropSheetPage) as LPCPROPSHEETPAGEW,
        );
        if hPropSheetPage.is_null() {
            ERR!("CreatePropertySheetPageW() failed!\n");
            return (FALSE) as BOOL;
        }

        if (lpfnAddPropSheetPageProc.unwrap_unchecked())(hPropSheetPage, lParam)
            == 0
        {
            ERR!("lpfnAddPropSheetPageProc() failed!\n");
            DestroyPropertySheetPage(hPropSheetPage);
            return (FALSE) as BOOL;
        }
    }

    return (TRUE) as BOOL;
}
