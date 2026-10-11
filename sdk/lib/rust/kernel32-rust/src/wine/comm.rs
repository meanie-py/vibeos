use winedebug_rust::{ERR, TRACE, WARN};
use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn wcsncmp( str1: *const WCHAR, str2: *const WCHAR, n: usize, ) -> i32;
}

// ---- cohab glue (kernel32 Rust port: SERIAL/IOCTL FFI + intra-DLL imports) ----
use crate::string::nls_subst::{HeapAlloc, HeapFree};


// BOOL is i32; nt::* glob exports FALSE/TRUE as u32 — shadow with BOOL-typed consts.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// winbase.h DCB DTR/RTS control values
pub const DTR_CONTROL_DISABLE: DWORD = 0x00;
pub const DTR_CONTROL_ENABLE: DWORD = 0x01;
pub const DTR_CONTROL_HANDSHAKE: DWORD = 0x02;
pub const RTS_CONTROL_DISABLE: DWORD = 0x00;
pub const RTS_CONTROL_ENABLE: DWORD = 0x01;
pub const RTS_CONTROL_HANDSHAKE: DWORD = 0x02;
pub const RTS_CONTROL_TOGGLE: DWORD = 0x03;

// IOCTL_SERIAL_* (ntddser.h): CTL_CODE(FILE_DEVICE_SERIAL_PORT=0x1b, fn, METHOD_BUFFERED=0, FILE_ANY_ACCESS=0)
const fn serial_ioctl(func: ULONG) -> ULONG { (0x1b << 16) | (func << 2) }
pub const IOCTL_SERIAL_SET_BAUD_RATE: ULONG = serial_ioctl(1);
pub const IOCTL_SERIAL_SET_QUEUE_SIZE: ULONG = serial_ioctl(2);
pub const IOCTL_SERIAL_SET_LINE_CONTROL: ULONG = serial_ioctl(3);
pub const IOCTL_SERIAL_SET_BREAK_ON: ULONG = serial_ioctl(4);
pub const IOCTL_SERIAL_SET_BREAK_OFF: ULONG = serial_ioctl(5);
pub const IOCTL_SERIAL_IMMEDIATE_CHAR: ULONG = serial_ioctl(6);
pub const IOCTL_SERIAL_SET_TIMEOUTS: ULONG = serial_ioctl(7);
pub const IOCTL_SERIAL_GET_TIMEOUTS: ULONG = serial_ioctl(8);
pub const IOCTL_SERIAL_SET_DTR: ULONG = serial_ioctl(9);
pub const IOCTL_SERIAL_CLR_DTR: ULONG = serial_ioctl(10);
pub const IOCTL_SERIAL_RESET_DEVICE: ULONG = serial_ioctl(11);
pub const IOCTL_SERIAL_SET_RTS: ULONG = serial_ioctl(12);
pub const IOCTL_SERIAL_CLR_RTS: ULONG = serial_ioctl(13);
pub const IOCTL_SERIAL_SET_XOFF: ULONG = serial_ioctl(14);
pub const IOCTL_SERIAL_SET_XON: ULONG = serial_ioctl(15);
pub const IOCTL_SERIAL_GET_WAIT_MASK: ULONG = serial_ioctl(16);
pub const IOCTL_SERIAL_SET_WAIT_MASK: ULONG = serial_ioctl(17);
pub const IOCTL_SERIAL_WAIT_ON_MASK: ULONG = serial_ioctl(18);
pub const IOCTL_SERIAL_PURGE: ULONG = serial_ioctl(19);
pub const IOCTL_SERIAL_GET_BAUD_RATE: ULONG = serial_ioctl(20);
pub const IOCTL_SERIAL_GET_LINE_CONTROL: ULONG = serial_ioctl(21);
pub const IOCTL_SERIAL_GET_CHARS: ULONG = serial_ioctl(22);
pub const IOCTL_SERIAL_SET_CHARS: ULONG = serial_ioctl(23);
pub const IOCTL_SERIAL_GET_HANDFLOW: ULONG = serial_ioctl(24);
pub const IOCTL_SERIAL_SET_HANDFLOW: ULONG = serial_ioctl(25);
pub const IOCTL_SERIAL_GET_MODEMSTATUS: ULONG = serial_ioctl(26);
pub const IOCTL_SERIAL_GET_COMMSTATUS: ULONG = serial_ioctl(27);
pub const IOCTL_SERIAL_LSRMST_INSERT: ULONG = serial_ioctl(31);

// SERIAL_* flag values (ntddser.h)
pub const SERIAL_DTR_CONTROL: ULONG = 0x00000001;
pub const SERIAL_DTR_HANDSHAKE: ULONG = 0x00000002;
pub const SERIAL_CTS_HANDSHAKE: ULONG = 0x00000008;
pub const SERIAL_DSR_HANDSHAKE: ULONG = 0x00000010;
pub const SERIAL_DSR_SENSITIVITY: ULONG = 0x00000040;
pub const SERIAL_ERROR_ABORT: ULONG = 0x80000000;
pub const SERIAL_AUTO_TRANSMIT: ULONG = 0x00000001;
pub const SERIAL_AUTO_RECEIVE: ULONG = 0x00000002;
pub const SERIAL_ERROR_CHAR: ULONG = 0x00000004;
pub const SERIAL_NULL_STRIPPING: ULONG = 0x00000008;
pub const SERIAL_RTS_CONTROL: ULONG = 0x00000040;
pub const SERIAL_RTS_HANDSHAKE: ULONG = 0x00000080;
pub const SERIAL_XOFF_CONTINUE: ULONG = 0x80000000;
pub const SERIAL_IOC_MCR_DTR: ULONG = 0x00000001;
pub const SERIAL_IOC_MCR_RTS: ULONG = 0x00000002;
pub const SERIAL_IOC_MCR_LOOP: ULONG = 0x00000010;
pub const SERIAL_IOC_MCR_OUT: ULONG = 0x0000000C;
pub const SERIAL_IOC_FCR_FIFO_ENABLE: ULONG = 0x00000001;
pub const SERIAL_IOC_FCR_RCVR_RESET: ULONG = 0x00000002;
pub const SERIAL_IOC_FCR_XMIT_RESET: ULONG = 0x00000004;
pub const SERIAL_IOC_FCR_DMA_MODE: ULONG = 0x00000008;
pub const SERIAL_IOC_FCR_RES: ULONG = 0x00000030;
pub const SERIAL_IOC_FCR_RCVR_TRIGGER_LSB: ULONG = 0x00000040;
pub const SERIAL_IOC_FCR_RCVR_TRIGGER_MSB: ULONG = 0x00000080;
pub const SERIAL_LSRMST_ESCAPE: ULONG = 0x00;
pub const SERIAL_LSRMST_LSR_DATA: ULONG = 0x01;
pub const SERIAL_LSRMST_LSR_NODATA: ULONG = 0x02;
pub const SERIAL_LSRMST_MST: ULONG = 0x03;
pub const SERIAL_ERROR_BREAK: ULONG = 0x00000001;
pub const SERIAL_ERROR_FRAMING: ULONG = 0x00000002;
pub const SERIAL_ERROR_OVERRUN: ULONG = 0x00000004;
pub const SERIAL_ERROR_QUEUEOVERRUN: ULONG = 0x00000008;
pub const SERIAL_ERROR_PARITY: ULONG = 0x00000010;
pub const SERIAL_TX_WAITING_FOR_CTS: ULONG = 0x00000001;
pub const SERIAL_TX_WAITING_FOR_DSR: ULONG = 0x00000002;
pub const SERIAL_TX_WAITING_FOR_DCD: ULONG = 0x00000004;
pub const SERIAL_TX_WAITING_FOR_XON: ULONG = 0x00000008;
pub const SERIAL_TX_WAITING_XOFF_SENT: ULONG = 0x00000010;

// ntddser.h serial control structures
#[repr(C)]
pub struct _SERIAL_BAUD_RATE { pub BaudRate: ULONG }
pub type SERIAL_BAUD_RATE = _SERIAL_BAUD_RATE;
pub type PSERIAL_BAUD_RATE = *mut _SERIAL_BAUD_RATE;

#[repr(C)]
pub struct _SERIAL_CHARS {
    pub EofChar: UCHAR, pub ErrorChar: UCHAR, pub BreakChar: UCHAR,
    pub EventChar: UCHAR, pub XonChar: UCHAR, pub XoffChar: UCHAR,
}
pub type SERIAL_CHARS = _SERIAL_CHARS;
pub type PSERIAL_CHARS = *mut _SERIAL_CHARS;

#[repr(C)]
pub struct _SERIAL_HANDFLOW {
    pub ControlHandShake: ULONG, pub FlowReplace: ULONG,
    pub XonLimit: LONG, pub XoffLimit: LONG,
}
pub type SERIAL_HANDFLOW = _SERIAL_HANDFLOW;
pub type PSERIAL_HANDFLOW = *mut _SERIAL_HANDFLOW;

#[repr(C)]
pub struct _SERIAL_LINE_CONTROL {
    pub StopBits: UCHAR, pub Parity: UCHAR, pub WordLength: UCHAR,
}
pub type SERIAL_LINE_CONTROL = _SERIAL_LINE_CONTROL;
pub type PSERIAL_LINE_CONTROL = *mut _SERIAL_LINE_CONTROL;

#[repr(C)]
pub struct _SERIAL_QUEUE_SIZE { pub InSize: ULONG, pub OutSize: ULONG }
pub type SERIAL_QUEUE_SIZE = _SERIAL_QUEUE_SIZE;
pub type PSERIAL_QUEUE_SIZE = *mut _SERIAL_QUEUE_SIZE;

#[repr(C)]
pub struct _SERIAL_STATUS {
    pub Errors: ULONG, pub HoldReasons: ULONG,
    pub AmountInInQueue: ULONG, pub AmountInOutQueue: ULONG,
    pub EofReceived: BOOLEAN, pub WaitForImmediate: BOOLEAN,
}
pub type SERIAL_STATUS = _SERIAL_STATUS;
pub type PSERIAL_STATUS = *mut _SERIAL_STATUS;

#[repr(C)]
pub struct _SERIAL_TIMEOUTS {
    pub ReadIntervalTimeout: ULONG,
    pub ReadTotalTimeoutMultiplier: ULONG,
    pub ReadTotalTimeoutConstant: ULONG,
    pub WriteTotalTimeoutMultiplier: ULONG,
    pub WriteTotalTimeoutConstant: ULONG,
}
pub type SERIAL_TIMEOUTS = _SERIAL_TIMEOUTS;
pub type PSERIAL_TIMEOUTS = *mut _SERIAL_TIMEOUTS;

// intra-DLL kernel32 exports (resolve to sibling #[no_mangle] symbols at link)
extern "system" {
    fn DeviceIoControl(
        hDevice: HANDLE, dwIoControlCode: DWORD, lpInBuffer: LPVOID, nInBufferSize: DWORD,
        lpOutBuffer: LPVOID, nOutBufferSize: DWORD, lpBytesReturned: LPDWORD, lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;
    fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HINSTANCE;
    fn FreeLibrary(hLibModule: HINSTANCE) -> BOOL;
    fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
    fn GetProcessHeap() -> HANDLE;
    fn MultiByteToWideChar(
        CodePage: UINT, Flags: DWORD, MultiByteString: LPCSTR, MultiByteCount: INT,
        WideCharString: LPWSTR, WideCharCount: INT,
    ) -> INT;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gDebugChannel: ULONG = 205 as ULONG;

static mut lpszSerialUI: [WCHAR; 13] = [
    's' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    '.' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

pub type PHYSICAL_ADDRESS = LARGE_INTEGER;
pub type PPHYSICAL_ADDRESS = *mut LARGE_INTEGER;

pub const NDEBUG: u32 = 0;
unsafe fn COMM_ParseStart(mut ptr: LPCWSTR) -> LPCWSTR {
    static mut comW: [WCHAR; 4] = ['C' as WCHAR, 'O' as WCHAR, 'M' as WCHAR, 0 as WCHAR];

    if strncmpiW(ptr, (*(&raw const comW)).as_ptr(), 3) == 0 {
        ptr = ptr.offset(3);

        if (*ptr as i32) < ('1' as i32) || (*ptr as i32) > ('9' as i32) {
            return core::ptr::null();
        }

        while (*ptr as i32) >= ('0' as i32) && (*ptr as i32) <= ('9' as i32) {
            ptr = ptr.offset(1);
        }

        if (*ptr as i32) != (':' as i32) && (*ptr as i32) != (' ' as i32) {
            return core::ptr::null();
        }

        while (*ptr as i32) == (' ' as i32) {
            ptr = ptr.offset(1);
        }

        if (*ptr as i32) == (':' as i32) {
            ptr = ptr.offset(1);
            while (*ptr as i32) == (' ' as i32) {
                ptr = ptr.offset(1);
            }
        }
    } else if (*ptr as i32) == (' ' as i32) {
        return core::ptr::null();
    }

    ptr
}
unsafe fn COMM_ParseNumber(mut ptr: LPCWSTR, mut lpnumber: LPDWORD) -> LPCWSTR {
    if (*ptr as i32) < '0' as i32 || (*ptr as i32) > '9' as i32 {
        return core::ptr::null();
    }
    *lpnumber = strtoulW(ptr, core::ptr::null_mut(), 10);
    while (*ptr as i32) >= '0' as i32 && (*ptr as i32) <= '9' as i32 {
        ptr = ptr.offset(1);
    }
    ptr
}
unsafe fn COMM_ParseParity(mut ptr: LPCWSTR, mut lpparity: LPBYTE) -> LPCWSTR {
    let fresh0 = ptr;
    ptr = ptr.offset(1);
    match toupperW(*fresh0) as i32 {
        69 => {
            *lpparity = EVENPARITY as BYTE;
        }
        77 => {
            *lpparity = MARKPARITY as BYTE;
        }
        78 => {
            *lpparity = NOPARITY as BYTE;
        }
        79 => {
            *lpparity = ODDPARITY as BYTE;
        }
        83 => {
            *lpparity = SPACEPARITY as BYTE;
        }
        _ => return core::ptr::null::<WCHAR>(),
    }
    return ptr;
}
unsafe fn COMM_ParseByteSize(mut ptr: LPCWSTR, mut lpbytesize: LPBYTE) -> LPCWSTR {
    let mut temp: DWORD = 0;
    ptr = COMM_ParseNumber(ptr, &raw mut temp);
    if ptr.is_null() {
        return core::ptr::null::<WCHAR>();
    }
    if temp >= 5 && temp <= 8 {
        *lpbytesize = temp as BYTE;
        return ptr;
    } else {
        return core::ptr::null::<WCHAR>();
    };
}
unsafe fn COMM_ParseStopBits(mut ptr: LPCWSTR, mut lpstopbits: LPBYTE) -> LPCWSTR {
    let mut temp: DWORD = 0;
    static mut stopbits15W: [WCHAR; 4] = [
        '1' as WCHAR,
        '.' as WCHAR,
        '5' as WCHAR,
        0 as WCHAR,
    ];
    if wcsncmp(
        &raw const stopbits15W as *const WCHAR,
        ptr as *const WCHAR,
        3,
    ) == 0
    {
        ptr = ptr.offset(3isize);
        *lpstopbits = ONE5STOPBITS as BYTE;
    } else {
        ptr = COMM_ParseNumber(ptr, &raw mut temp);
        if ptr.is_null() {
            return core::ptr::null::<WCHAR>();
        }
        if temp == 1 {
            *lpstopbits = ONESTOPBIT as BYTE;
        } else if temp == 2 {
            *lpstopbits = TWOSTOPBITS as BYTE;
        } else {
            return core::ptr::null::<WCHAR>();
        }
    }
    return ptr;
}
unsafe fn COMM_ParseOnOff(mut ptr: LPCWSTR, mut lponoff: LPDWORD) -> LPCWSTR {
    static mut onW: [WCHAR; 3] = ['o' as WCHAR, 'n' as WCHAR, 0 as WCHAR];
    static mut offW: [WCHAR; 4] = ['o' as WCHAR, 'f' as WCHAR, 'f' as WCHAR, 0 as WCHAR];

    if strncmpiW((*(&raw const onW)).as_ptr(), ptr, 2) == 0 {
        ptr = ptr.offset(2);
        *lponoff = 1;
    } else if strncmpiW((*(&raw const offW)).as_ptr(), ptr, 3) == 0 {
        ptr = ptr.offset(3);
        *lponoff = 0;
    } else {
        return core::ptr::null();
    }

    return ptr;
}
unsafe fn COMM_BuildOldCommDCB(mut device: LPCWSTR, mut lpdcb: LPDCB) -> BOOL {
    let mut last: WCHAR = 0;
    device = COMM_ParseNumber(device, &raw mut (*lpdcb).BaudRate);
    if device.is_null() {
        return FALSE;
    }
    match (*lpdcb).BaudRate {
        11 | 30 | 60 => {
            (*lpdcb).BaudRate = ((*lpdcb).BaudRate as DWORD).wrapping_mul(10) as DWORD;
        }
        12 | 24 | 48 | 96 => {
            (*lpdcb).BaudRate = ((*lpdcb).BaudRate as DWORD).wrapping_mul(100) as DWORD;
        }
        19 => {
            (*lpdcb).BaudRate = 19200;
        }
        _ => {}
    }
    while *device as i32 == ' ' as i32 {
        device = device.offset(1);
    }
    let fresh1 = device;
    device = device.offset(1);
    if *fresh1 as i32 != ',' as i32 {
        return FALSE;
    }
    while *device as i32 == ' ' as i32 {
        device = device.offset(1);
    }
    device = COMM_ParseParity(device, &raw mut (*lpdcb).Parity);
    if device.is_null() {
        return FALSE;
    }
    while *device as i32 == ' ' as i32 {
        device = device.offset(1);
    }
    let fresh2 = device;
    device = device.offset(1);
    if *fresh2 as i32 != ',' as i32 {
        return FALSE;
    }
    while *device as i32 == ' ' as i32 {
        device = device.offset(1);
    }
    device = COMM_ParseByteSize(device, &raw mut (*lpdcb).ByteSize);
    if device.is_null() {
        return FALSE;
    }
    while *device as i32 == ' ' as i32 {
        device = device.offset(1);
    }
    let fresh3 = device;
    device = device.offset(1);
    if *fresh3 as i32 != ',' as i32 {
        return FALSE;
    }
    while *device as i32 == ' ' as i32 {
        device = device.offset(1);
    }
    device = COMM_ParseStopBits(device, &raw mut (*lpdcb).StopBits);
    if device.is_null() {
        return FALSE;
    }
    while *device as i32 == ' ' as i32 {
        device = device.offset(1);
    }
    if *device as i32 == ',' as i32 {
        device = device.offset(1);
        while *device as i32 == ' ' as i32 {
            device = device.offset(1);
        }
        if *device != 0 {
            let fresh4 = device;
            device = device.offset(1);
            last = toupperW(*fresh4);
        }
        while *device as i32 == ' ' as i32 {
            device = device.offset(1);
        }
    }
    match last as i32 {
        0 => {
            (*lpdcb).set_fInX((FALSE) as DWORD);
            (*lpdcb).set_fOutX((FALSE) as DWORD);
            (*lpdcb).set_fOutxCtsFlow((FALSE) as DWORD);
            (*lpdcb).set_fOutxDsrFlow((FALSE) as DWORD);
            (*lpdcb).set_fDtrControl((DTR_CONTROL_ENABLE) as DWORD);
            (*lpdcb).set_fRtsControl((RTS_CONTROL_ENABLE) as DWORD);
        }
        88 => {
            (*lpdcb).set_fInX((TRUE) as DWORD);
            (*lpdcb).set_fOutX((TRUE) as DWORD);
            (*lpdcb).set_fOutxCtsFlow((FALSE) as DWORD);
            (*lpdcb).set_fOutxDsrFlow((FALSE) as DWORD);
            (*lpdcb).set_fDtrControl((DTR_CONTROL_ENABLE) as DWORD);
            (*lpdcb).set_fRtsControl((RTS_CONTROL_ENABLE) as DWORD);
        }
        80 => {
            (*lpdcb).set_fInX((FALSE) as DWORD);
            (*lpdcb).set_fOutX((FALSE) as DWORD);
            (*lpdcb).set_fOutxCtsFlow((TRUE) as DWORD);
            (*lpdcb).set_fOutxDsrFlow((TRUE) as DWORD);
            (*lpdcb).set_fDtrControl((DTR_CONTROL_HANDSHAKE) as DWORD);
            (*lpdcb).set_fRtsControl((RTS_CONTROL_HANDSHAKE) as DWORD);
        }
        _ => return FALSE,
    }
    if *device != 0 {
        return FALSE;
    }
    TRUE
}
unsafe fn COMM_BuildNewCommDCB(
    mut device: LPCWSTR,
    mut lpdcb: LPDCB,
    mut lptimeouts: LPCOMMTIMEOUTS,
) -> BOOL {
    let mut temp: DWORD = 0;
    let mut baud: BOOL = FALSE;
    let mut stop: BOOL = FALSE;
    static mut baudW: [WCHAR; 6] = [
        'b' as WCHAR, 'a' as WCHAR, 'u' as WCHAR, 'd' as WCHAR, '=' as WCHAR, 0 as WCHAR,
    ];
    static mut parityW: [WCHAR; 8] = [
        'p' as WCHAR, 'a' as WCHAR, 'r' as WCHAR, 'i' as WCHAR, 't' as WCHAR, 'y' as WCHAR,
        '=' as WCHAR, 0 as WCHAR,
    ];
    static mut dataW: [WCHAR; 6] = [
        'd' as WCHAR, 'a' as WCHAR, 't' as WCHAR, 'a' as WCHAR, '=' as WCHAR, 0 as WCHAR,
    ];
    static mut stopW: [WCHAR; 6] = [
        's' as WCHAR, 't' as WCHAR, 'o' as WCHAR, 'p' as WCHAR, '=' as WCHAR, 0 as WCHAR,
    ];
    static mut toW: [WCHAR; 4] = [
        't' as WCHAR, 'o' as WCHAR, '=' as WCHAR, 0 as WCHAR,
    ];
    static mut xonW: [WCHAR; 5] = [
        'x' as WCHAR, 'o' as WCHAR, 'n' as WCHAR, '=' as WCHAR, 0 as WCHAR,
    ];
    static mut odsrW: [WCHAR; 6] = [
        'o' as WCHAR, 'd' as WCHAR, 's' as WCHAR, 'r' as WCHAR, '=' as WCHAR, 0 as WCHAR,
    ];
    static mut octsW: [WCHAR; 6] = [
        'o' as WCHAR, 'c' as WCHAR, 't' as WCHAR, 's' as WCHAR, '=' as WCHAR, 0 as WCHAR,
    ];
    static mut dtrW: [WCHAR; 5] = [
        'd' as WCHAR, 't' as WCHAR, 'r' as WCHAR, '=' as WCHAR, 0 as WCHAR,
    ];
    static mut rtsW: [WCHAR; 5] = [
        'r' as WCHAR, 't' as WCHAR, 's' as WCHAR, '=' as WCHAR, 0 as WCHAR,
    ];
    static mut idsrW: [WCHAR; 6] = [
        'i' as WCHAR, 'd' as WCHAR, 's' as WCHAR, 'r' as WCHAR, '=' as WCHAR, 0 as WCHAR,
    ];

    while *device != 0 {
        while *device as i32 == ' ' as i32 {
            device = device.offset(1);
        }

        if strncmpiW(
            (*(&raw const baudW)).as_ptr() as LPCWSTR,
            device,
            5,
        ) == 0
        {
            baud = TRUE;
            device = COMM_ParseNumber(
                device.offset(5),
                &mut (*lpdcb).BaudRate as LPDWORD,
            );
            if device.is_null() {
                return FALSE;
            }
        } else if strncmpiW(
            (*(&raw const parityW)).as_ptr() as LPCWSTR,
            device,
            7,
        ) == 0
        {
            device = COMM_ParseParity(
                device.offset(7),
                &mut (*lpdcb).Parity as LPBYTE,
            );
            if device.is_null() {
                return FALSE;
            }
        } else if strncmpiW(
            (*(&raw const dataW)).as_ptr() as LPCWSTR,
            device,
            5,
        ) == 0
        {
            device = COMM_ParseByteSize(
                device.offset(5),
                &mut (*lpdcb).ByteSize as LPBYTE,
            );
            if device.is_null() {
                return FALSE;
            }
        } else if strncmpiW(
            (*(&raw const stopW)).as_ptr() as LPCWSTR,
            device,
            5,
        ) == 0
        {
            stop = TRUE;
            device = COMM_ParseStopBits(
                device.offset(5),
                &mut (*lpdcb).StopBits as LPBYTE,
            );
            if device.is_null() {
                return FALSE;
            }
        } else if strncmpiW(
            (*(&raw const toW)).as_ptr() as LPCWSTR,
            device,
            3,
        ) == 0
        {
            device = COMM_ParseOnOff(
                device.offset(3),
                &mut temp as LPDWORD,
            );
            if device.is_null() {
                return FALSE;
            }
            (*lptimeouts).ReadIntervalTimeout = 0;
            (*lptimeouts).ReadTotalTimeoutMultiplier = 0;
            (*lptimeouts).ReadTotalTimeoutConstant = 0;
            (*lptimeouts).WriteTotalTimeoutMultiplier = 0;
            (*lptimeouts).WriteTotalTimeoutConstant = if temp != 0 { 60000 } else { 0 };
        } else if strncmpiW(
            (*(&raw const xonW)).as_ptr() as LPCWSTR,
            device,
            4,
        ) == 0
        {
            device = COMM_ParseOnOff(
                device.offset(4),
                &mut temp as LPDWORD,
            );
            if device.is_null() {
                return FALSE;
            }
            (*lpdcb).set_fOutX((temp) as DWORD);
            (*lpdcb).set_fInX((temp) as DWORD);
        } else if strncmpiW(
            (*(&raw const odsrW)).as_ptr() as LPCWSTR,
            device,
            5,
        ) == 0
        {
            device = COMM_ParseOnOff(
                device.offset(5),
                &mut temp as LPDWORD,
            );
            if device.is_null() {
                return FALSE;
            }
            (*lpdcb).set_fOutxDsrFlow((temp) as DWORD);
        } else if strncmpiW(
            (*(&raw const octsW)).as_ptr() as LPCWSTR,
            device,
            5,
        ) == 0
        {
            device = COMM_ParseOnOff(
                device.offset(5),
                &mut temp as LPDWORD,
            );
            if device.is_null() {
                return FALSE;
            }
            (*lpdcb).set_fOutxCtsFlow((temp) as DWORD);
        } else if strncmpiW(
            (*(&raw const dtrW)).as_ptr() as LPCWSTR,
            device,
            4,
        ) == 0
        {
            device = COMM_ParseOnOff(
                device.offset(4),
                &mut temp as LPDWORD,
            );
            if device.is_null() {
                return FALSE;
            }
            (*lpdcb).set_fDtrControl((temp) as DWORD);
        } else if strncmpiW(
            (*(&raw const rtsW)).as_ptr() as LPCWSTR,
            device,
            4,
        ) == 0
        {
            device = COMM_ParseOnOff(
                device.offset(4),
                &mut temp as LPDWORD,
            );
            if device.is_null() {
                return FALSE;
            }
            (*lpdcb).set_fRtsControl((temp) as DWORD);
        } else if strncmpiW(
            (*(&raw const idsrW)).as_ptr() as LPCWSTR,
            device,
            5,
        ) == 0
        {
            device = COMM_ParseOnOff(
                device.offset(5),
                &mut temp as LPDWORD,
            );
            if device.is_null() {
                return FALSE;
            }
            (*lpdcb).set_fDsrSensitivity((temp) as DWORD);
        } else {
            return FALSE;
        }

        if *device != 0 && *device as i32 != ' ' as i32 {
            return FALSE;
        }
    }

    if stop == 0 {
        if baud != 0 && (*lpdcb).BaudRate == 110 {
            (*lpdcb).StopBits = TWOSTOPBITS as BYTE;
        } else {
            (*lpdcb).StopBits = ONESTOPBIT as BYTE;
        }
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "system" fn BuildCommDCBA(
    device: LPCSTR,
    lpdcb: LPDCB,
) -> BOOL {
    return BuildCommDCBAndTimeoutsA(device, lpdcb, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "system" fn BuildCommDCBAndTimeoutsA(
    device: LPCSTR,
    lpdcb: LPDCB,
    lptimeouts: LPCOMMTIMEOUTS,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut deviceW: _UNICODE_STRING = core::mem::zeroed();

    TRACE!(
        b"(%s,%p,%p)\n\0".as_ptr() as LPCSTR,
        device,
        lpdcb,
        lptimeouts,
    );

    if !device.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut deviceW, device);
    } else {
        deviceW.Buffer = core::ptr::null_mut();
    }

    if !deviceW.Buffer.is_null() {
        ret = BuildCommDCBAndTimeoutsW(deviceW.Buffer, lpdcb, lptimeouts);
    }

    RtlFreeUnicodeString(&raw mut deviceW);
    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn BuildCommDCBAndTimeoutsW(
    devid: LPCWSTR,
    lpdcb: LPDCB,
    lptimeouts: LPCOMMTIMEOUTS,
) -> BOOL {
    let mut dcb: DCB = core::mem::zeroed();
    let mut timeouts: COMMTIMEOUTS = core::mem::zeroed();
    let mut result: BOOL = 0;
    let mut ptr: LPCWSTR = devid;

    TRACE!(
        b"(%s,%p,%p)\n\0".as_ptr() as LPCSTR,
        debugstr_w(devid),
        lpdcb,
        lptimeouts,
    );

    memset(
        &mut timeouts as *mut COMMTIMEOUTS as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<COMMTIMEOUTS>(),
    );

    (*lpdcb).DCBlength = core::mem::size_of::<DCB>() as DWORD;

    dcb = *lpdcb;
    if !lptimeouts.is_null() {
        timeouts = *lptimeouts;
    }

    ptr = COMM_ParseStart(ptr);

    if ptr.is_null() {
        result = FALSE;
    } else if !strchrW(ptr, ',' as i32 as u16).is_null() {
        result = COMM_BuildOldCommDCB(ptr, &mut dcb);
    } else {
        result = COMM_BuildNewCommDCB(ptr, &mut dcb, &mut timeouts);
    }

    if result != 0 {
        *lpdcb = dcb;
        if !lptimeouts.is_null() {
            *lptimeouts = timeouts;
        }
        return TRUE;
    } else {
        WARN!(
            b"Invalid device control string: %s\n\0".as_ptr() as LPCSTR,
            debugstr_w(devid),
        );
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
}
#[no_mangle]
pub unsafe extern "system" fn BuildCommDCBW(
    devid: LPCWSTR,
    lpdcb: LPDCB,
) -> BOOL {
    return BuildCommDCBAndTimeoutsW(devid, lpdcb, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SetCommBreak(handle: HANDLE) -> BOOL {
    let mut dwBytesReturned: DWORD = 0;
    return DeviceIoControl(
        handle,
        IOCTL_SERIAL_SET_BREAK_ON as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        &raw mut dwBytesReturned,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn ClearCommBreak(handle: HANDLE) -> BOOL {
    let mut dwBytesReturned: DWORD = 0;
    return DeviceIoControl(
        handle,
        IOCTL_SERIAL_SET_BREAK_OFF as DWORD,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        &mut dwBytesReturned as *mut DWORD,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn EscapeCommFunction(
    handle: HANDLE,
    func: DWORD,
) -> BOOL {
    let mut ioc: DWORD = 0;
    let mut dwBytesReturned: DWORD = 0;

    match func {
        CLRDTR => {
            ioc = IOCTL_SERIAL_CLR_DTR;
        }
        CLRRTS => {
            ioc = IOCTL_SERIAL_CLR_RTS;
        }
        SETDTR => {
            ioc = IOCTL_SERIAL_SET_DTR;
        }
        SETRTS => {
            ioc = IOCTL_SERIAL_SET_RTS;
        }
        SETXOFF => {
            ioc = IOCTL_SERIAL_SET_XOFF;
        }
        SETXON => {
            ioc = IOCTL_SERIAL_SET_XON;
        }
        SETBREAK => {
            ioc = IOCTL_SERIAL_SET_BREAK_ON;
        }
        CLRBREAK => {
            ioc = IOCTL_SERIAL_SET_BREAK_OFF;
        }
        RESETDEV => {
            ioc = IOCTL_SERIAL_RESET_DEVICE;
        }
        _ => {
            ERR!(
                "Unknown function code (%u)\n",
                func,
            );
            SetLastError(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
    }

    return DeviceIoControl(
        handle,
        ioc,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        &mut dwBytesReturned as *mut DWORD,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn PurgeComm(
    handle: HANDLE,
    flags: DWORD,
) -> BOOL {
    let mut dwBytesReturned: DWORD = 0;
    return DeviceIoControl(
        handle,
        IOCTL_SERIAL_PURGE,
        core::ptr::addr_of!(flags) as LPVOID,
        core::mem::size_of::<DWORD>() as DWORD,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn ClearCommError(
    handle: HANDLE,
    errors: LPDWORD,
    lpStat: LPCOMSTAT,
) -> BOOL {
    let mut ss: SERIAL_STATUS = core::mem::zeroed();
    let mut dwBytesReturned: DWORD = 0;

    if DeviceIoControl(
        handle,
        IOCTL_SERIAL_GET_COMMSTATUS,
        core::ptr::null_mut(),
        0,
        &mut ss as *mut _ as LPVOID,
        core::mem::size_of::<SERIAL_STATUS>() as DWORD,
        &mut dwBytesReturned,
        core::ptr::null_mut(),
    ) == 0
    {
        return 0;
    }

    if !errors.is_null() {
        *errors = 0;
        if ss.Errors & SERIAL_ERROR_BREAK != 0 {
            *errors |= CE_BREAK;
        }
        if ss.Errors & SERIAL_ERROR_FRAMING != 0 {
            *errors |= CE_FRAME;
        }
        if ss.Errors & SERIAL_ERROR_OVERRUN != 0 {
            *errors |= CE_OVERRUN;
        }
        if ss.Errors & SERIAL_ERROR_QUEUEOVERRUN != 0 {
            *errors |= CE_RXOVER;
        }
        if ss.Errors & SERIAL_ERROR_PARITY != 0 {
            *errors |= CE_RXPARITY;
        }
    }

    if !lpStat.is_null() {
        core::ptr::write_bytes(lpStat as *mut u8, 0, core::mem::size_of::<COMSTAT>());

        if ss.HoldReasons & SERIAL_TX_WAITING_FOR_CTS != 0 {
            (*lpStat).set_fCtsHold((1) as DWORD);
        }
        if ss.HoldReasons & SERIAL_TX_WAITING_FOR_DSR != 0 {
            (*lpStat).set_fDsrHold((1) as DWORD);
        }
        if ss.HoldReasons & SERIAL_TX_WAITING_FOR_DCD != 0 {
            (*lpStat).set_fRlsdHold((1) as DWORD);
        }
        if ss.HoldReasons & SERIAL_TX_WAITING_FOR_XON != 0 {
            (*lpStat).set_fXoffHold((1) as DWORD);
        }
        if ss.HoldReasons & SERIAL_TX_WAITING_XOFF_SENT != 0 {
            (*lpStat).set_fXoffSent((1) as DWORD);
        }
        if ss.EofReceived != 0 {
            (*lpStat).set_fEof((1) as DWORD);
        }
        if ss.WaitForImmediate != 0 {
            (*lpStat).set_fTxim((1) as DWORD);
        }
        (*lpStat).cbInQue = ss.AmountInInQueue;
        (*lpStat).cbOutQue = ss.AmountInOutQueue;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetupComm(
    handle: HANDLE,
    insize: DWORD,
    outsize: DWORD,
) -> BOOL {
    let mut sqs: SERIAL_QUEUE_SIZE = SERIAL_QUEUE_SIZE {
        InSize: 0,
        OutSize: 0,
    };
    let mut dwBytesReturned: DWORD = 0;
    sqs.InSize = insize;
    sqs.OutSize = outsize;
    return DeviceIoControl(
        handle,
        IOCTL_SERIAL_SET_QUEUE_SIZE,
        &mut sqs as *mut SERIAL_QUEUE_SIZE as LPVOID,
        core::mem::size_of::<SERIAL_QUEUE_SIZE>() as DWORD,
        core::ptr::null_mut(),
        0,
        &mut dwBytesReturned as LPDWORD,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetCommMask(
    handle: HANDLE,
    evtmask: LPDWORD,
) -> BOOL {
    let mut dwBytesReturned: DWORD = 0;
    TRACE!(
        b"handle %p, mask %p\n\0".as_ptr() as LPCSTR,
        handle,
        evtmask,
    );
    return DeviceIoControl(
        handle,
        IOCTL_SERIAL_GET_WAIT_MASK,
        core::ptr::null_mut(),
        0,
        evtmask as LPVOID,
        core::mem::size_of::<DWORD>() as DWORD,
        &mut dwBytesReturned as LPDWORD,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetCommMask(
    handle: HANDLE,
    mut evtmask: DWORD,
) -> BOOL {
    let mut dwBytesReturned: DWORD = 0;
    TRACE!(
        b"handle %p, mask %x\n\0".as_ptr() as LPCSTR,
        handle,
        evtmask,
    );
    return DeviceIoControl(
        handle,
        IOCTL_SERIAL_SET_WAIT_MASK,
        &raw mut evtmask as LPVOID,
        core::mem::size_of::<DWORD>() as DWORD,
        core::ptr::null_mut(),
        0,
        &raw mut dwBytesReturned,
        core::ptr::null_mut(),
    );
}
unsafe fn dump_dcb(lpdcb: *const DCB) {
    TRACE!(
        b"bytesize=%d baudrate=%d fParity=%d Parity=%d stopbits=%d\n\0".as_ptr() as *const i8,
        (*lpdcb).ByteSize as i32,
        (*lpdcb).BaudRate as i32,
        (*lpdcb).fParity() as i32,
        (*lpdcb).Parity as i32,
        if (*lpdcb).StopBits as i32 == ONESTOPBIT as i32 {
            1i32
        } else if (*lpdcb).StopBits as i32 == TWOSTOPBITS as i32 {
            2i32
        } else {
            0i32
        },
    );
    TRACE!(
        b"%sIXON %sIXOFF\n\0".as_ptr() as *const i8,
        if (*lpdcb).fOutX() != 0 {
            b"\0".as_ptr() as *const i8
        } else {
            b"~\0".as_ptr() as *const i8
        },
        if (*lpdcb).fInX() != 0 {
            b"\0".as_ptr() as *const i8
        } else {
            b"~\0".as_ptr() as *const i8
        },
    );
    TRACE!(
        b"fOutxCtsFlow=%d fRtsControl=%d\n\0".as_ptr() as *const i8,
        (*lpdcb).fOutxCtsFlow() as i32,
        (*lpdcb).fRtsControl() as i32,
    );
    TRACE!(
        b"fOutxDsrFlow=%d fDtrControl=%d\n\0".as_ptr() as *const i8,
        (*lpdcb).fOutxDsrFlow() as i32,
        (*lpdcb).fDtrControl() as i32,
    );
    if (*lpdcb).fOutxCtsFlow() != 0 || (*lpdcb).fRtsControl() == RTS_CONTROL_HANDSHAKE {
        TRACE!(b"CRTSCTS\n\0".as_ptr() as *const i8);
    } else {
        TRACE!(b"~CRTSCTS\n\0".as_ptr() as *const i8);
    }
}

#[no_mangle]
pub unsafe extern "system" fn SetCommState(
    handle: HANDLE,
    lpdcb: LPDCB,
) -> BOOL {
    let mut sbr: SERIAL_BAUD_RATE = core::mem::zeroed();
    let mut slc: SERIAL_LINE_CONTROL = core::mem::zeroed();
    let mut shf: SERIAL_HANDFLOW = core::mem::zeroed();
    let mut sc: SERIAL_CHARS = core::mem::zeroed();
    let mut dwBytesReturned: DWORD = 0;

    TRACE!(b"handle %p, ptr %p\n\0".as_ptr() as LPCSTR, handle, lpdcb);

    if lpdcb.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    dump_dcb(lpdcb as *const DCB);

    sbr.BaudRate = (*lpdcb).BaudRate;

    slc.StopBits = (*lpdcb).StopBits;
    slc.Parity = (*lpdcb).Parity;
    slc.WordLength = (*lpdcb).ByteSize;

    shf.ControlHandShake = 0;
    shf.FlowReplace = 0;
    if (*lpdcb).fOutxCtsFlow() != 0 {
        shf.ControlHandShake |= SERIAL_CTS_HANDSHAKE;
    }
    if (*lpdcb).fOutxDsrFlow() != 0 {
        shf.ControlHandShake |= SERIAL_DSR_HANDSHAKE;
    }
    match (*lpdcb).fDtrControl() {
        DTR_CONTROL_DISABLE => {}
        DTR_CONTROL_ENABLE => {
            shf.ControlHandShake |= SERIAL_DTR_CONTROL;
        }
        DTR_CONTROL_HANDSHAKE => {
            shf.ControlHandShake |= SERIAL_DTR_HANDSHAKE;
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
    }
    match (*lpdcb).fRtsControl() {
        RTS_CONTROL_DISABLE => {}
        RTS_CONTROL_ENABLE => {
            shf.FlowReplace |= SERIAL_RTS_CONTROL;
        }
        RTS_CONTROL_HANDSHAKE => {
            shf.FlowReplace |= SERIAL_RTS_HANDSHAKE;
        }
        RTS_CONTROL_TOGGLE => {
            shf.FlowReplace |= SERIAL_RTS_CONTROL | SERIAL_RTS_HANDSHAKE;
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
    }
    if (*lpdcb).fDsrSensitivity() != 0 {
        shf.ControlHandShake |= SERIAL_DSR_SENSITIVITY;
    }
    if (*lpdcb).fAbortOnError() != 0 {
        shf.ControlHandShake |= SERIAL_ERROR_ABORT;
    }
    if (*lpdcb).fErrorChar() != 0 {
        shf.FlowReplace |= SERIAL_ERROR_CHAR;
    }
    if (*lpdcb).fNull() != 0 {
        shf.FlowReplace |= SERIAL_NULL_STRIPPING;
    }
    if (*lpdcb).fTXContinueOnXoff() != 0 {
        shf.FlowReplace |= SERIAL_XOFF_CONTINUE;
    }
    if (*lpdcb).fOutX() != 0 {
        shf.FlowReplace |= SERIAL_AUTO_TRANSMIT;
    }
    if (*lpdcb).fInX() != 0 {
        shf.FlowReplace |= SERIAL_AUTO_RECEIVE;
    }

    shf.XonLimit = (*lpdcb).XonLim as LONG;
    shf.XoffLimit = (*lpdcb).XoffLim as LONG;

    sc.EofChar = (*lpdcb).EofChar as UCHAR;
    sc.ErrorChar = (*lpdcb).ErrorChar as UCHAR;
    sc.BreakChar = 0;
    sc.EventChar = (*lpdcb).EvtChar as UCHAR;
    sc.XonChar = (*lpdcb).XonChar as UCHAR;
    sc.XoffChar = (*lpdcb).XoffChar as UCHAR;

    (DeviceIoControl(
        handle,
        IOCTL_SERIAL_SET_BAUD_RATE,
        &mut sbr as *mut _ as LPVOID,
        core::mem::size_of::<SERIAL_BAUD_RATE>() as DWORD,
        core::ptr::null_mut(),
        0,
        &mut dwBytesReturned,
        core::ptr::null_mut(),
    ) != 0
        && DeviceIoControl(
            handle,
            IOCTL_SERIAL_SET_LINE_CONTROL,
            &mut slc as *mut _ as LPVOID,
            core::mem::size_of::<SERIAL_LINE_CONTROL>() as DWORD,
            core::ptr::null_mut(),
            0,
            &mut dwBytesReturned,
            core::ptr::null_mut(),
        ) != 0
        && DeviceIoControl(
            handle,
            IOCTL_SERIAL_SET_HANDFLOW,
            &mut shf as *mut _ as LPVOID,
            core::mem::size_of::<SERIAL_HANDFLOW>() as DWORD,
            core::ptr::null_mut(),
            0,
            &mut dwBytesReturned,
            core::ptr::null_mut(),
        ) != 0
        && DeviceIoControl(
            handle,
            IOCTL_SERIAL_SET_CHARS,
            &mut sc as *mut _ as LPVOID,
            core::mem::size_of::<SERIAL_CHARS>() as DWORD,
            core::ptr::null_mut(),
            0,
            &mut dwBytesReturned,
            core::ptr::null_mut(),
        ) != 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetCommState(handle: HANDLE, lpdcb: LPDCB) -> BOOL {
    let mut sbr: SERIAL_BAUD_RATE = core::mem::zeroed();
    let mut slc: SERIAL_LINE_CONTROL = core::mem::zeroed();
    let mut shf: SERIAL_HANDFLOW = core::mem::zeroed();
    let mut sc: SERIAL_CHARS = core::mem::zeroed();
    let mut dwBytesReturned: DWORD = 0;

    TRACE!(b"handle %p, ptr %p\n\0".as_ptr() as LPCSTR, handle, lpdcb);

    if lpdcb.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    if DeviceIoControl(
        handle,
        IOCTL_SERIAL_GET_BAUD_RATE,
        core::ptr::null_mut(),
        0,
        &raw mut sbr as LPVOID,
        core::mem::size_of::<SERIAL_BAUD_RATE>() as DWORD,
        &raw mut dwBytesReturned,
        core::ptr::null_mut(),
    ) == 0
        || DeviceIoControl(
            handle,
            IOCTL_SERIAL_GET_LINE_CONTROL,
            core::ptr::null_mut(),
            0,
            &raw mut slc as LPVOID,
            core::mem::size_of::<SERIAL_LINE_CONTROL>() as DWORD,
            &raw mut dwBytesReturned,
            core::ptr::null_mut(),
        ) == 0
        || DeviceIoControl(
            handle,
            IOCTL_SERIAL_GET_HANDFLOW,
            core::ptr::null_mut(),
            0,
            &raw mut shf as LPVOID,
            core::mem::size_of::<SERIAL_HANDFLOW>() as DWORD,
            &raw mut dwBytesReturned,
            core::ptr::null_mut(),
        ) == 0
        || DeviceIoControl(
            handle,
            IOCTL_SERIAL_GET_CHARS,
            core::ptr::null_mut(),
            0,
            &raw mut sc as LPVOID,
            core::mem::size_of::<SERIAL_CHARS>() as DWORD,
            &raw mut dwBytesReturned,
            core::ptr::null_mut(),
        ) == 0
    {
        return FALSE;
    }

    core::ptr::write_bytes(lpdcb as *mut u8, 0, core::mem::size_of::<DCB>());
    (*lpdcb).DCBlength = core::mem::size_of::<DCB>() as DWORD;

    (*lpdcb).set_fBinary((1) as DWORD);
    (*lpdcb).set_fParity((0) as DWORD);

    (*lpdcb).BaudRate = sbr.BaudRate;

    (*lpdcb).StopBits = slc.StopBits;
    (*lpdcb).Parity = slc.Parity;
    (*lpdcb).ByteSize = slc.WordLength;

    if shf.ControlHandShake & SERIAL_CTS_HANDSHAKE != 0 {
        (*lpdcb).set_fOutxCtsFlow((1) as DWORD);
    }
    if shf.ControlHandShake & SERIAL_DSR_HANDSHAKE != 0 {
        (*lpdcb).set_fOutxDsrFlow((1) as DWORD);
    }
    match shf.ControlHandShake & (SERIAL_DTR_CONTROL | SERIAL_DTR_HANDSHAKE) {
        0 => (*lpdcb).set_fDtrControl(DTR_CONTROL_DISABLE as DWORD),
        1 => (*lpdcb).set_fDtrControl(DTR_CONTROL_ENABLE as DWORD),
        2 => (*lpdcb).set_fDtrControl(DTR_CONTROL_HANDSHAKE as DWORD),
        _ => {}
    }
    match shf.FlowReplace & (SERIAL_RTS_CONTROL | SERIAL_RTS_HANDSHAKE) {
        0 => (*lpdcb).set_fRtsControl(RTS_CONTROL_DISABLE as DWORD),
        64 => (*lpdcb).set_fRtsControl(RTS_CONTROL_ENABLE as DWORD),
        128 => (*lpdcb).set_fRtsControl(RTS_CONTROL_HANDSHAKE as DWORD),
        192 => (*lpdcb).set_fRtsControl(RTS_CONTROL_TOGGLE as DWORD),
        _ => {}
    }
    if shf.ControlHandShake & SERIAL_DSR_SENSITIVITY != 0 {
        (*lpdcb).set_fDsrSensitivity(1 as DWORD);
    }
    if shf.ControlHandShake & SERIAL_ERROR_ABORT != 0 {
        (*lpdcb).set_fAbortOnError((1) as DWORD);
    }
    if shf.FlowReplace & SERIAL_ERROR_CHAR != 0 {
        (*lpdcb).set_fErrorChar((1) as DWORD);
    }
    if shf.FlowReplace & SERIAL_NULL_STRIPPING != 0 {
        (*lpdcb).set_fNull((1) as DWORD);
    }
    if shf.FlowReplace & SERIAL_XOFF_CONTINUE != 0 {
        (*lpdcb).set_fTXContinueOnXoff((1) as DWORD);
    }
    (*lpdcb).XonLim = shf.XonLimit as WORD;
    (*lpdcb).XoffLim = shf.XoffLimit as WORD;

    if shf.FlowReplace & SERIAL_AUTO_TRANSMIT != 0 {
        (*lpdcb).set_fOutX((1) as DWORD);
    }
    if shf.FlowReplace & SERIAL_AUTO_RECEIVE != 0 {
        (*lpdcb).set_fInX((1) as DWORD);
    }

    (*lpdcb).EofChar = sc.EofChar as CHAR;
    (*lpdcb).ErrorChar = sc.ErrorChar as CHAR;
    (*lpdcb).EvtChar = sc.EventChar as CHAR;
    (*lpdcb).XonChar = sc.XonChar as CHAR;
    (*lpdcb).XoffChar = sc.XoffChar as CHAR;

    TRACE!(b"OK\n\0".as_ptr() as LPCSTR);
    dump_dcb(lpdcb);

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn TransmitCommChar(
    hComm: HANDLE,
    mut chTransmit: CHAR,
) -> BOOL {
    let mut dwBytesReturned: DWORD = 0;
    return DeviceIoControl(
        hComm,
        IOCTL_SERIAL_IMMEDIATE_CHAR as DWORD,
        core::ptr::addr_of_mut!(chTransmit) as LPVOID,
        core::mem::size_of::<CHAR>() as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!(dwBytesReturned),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetCommTimeouts(
    hComm: HANDLE,
    lptimeouts: LPCOMMTIMEOUTS,
) -> BOOL {
    let mut st: SERIAL_TIMEOUTS = core::mem::zeroed();
    let mut dwBytesReturned: DWORD = 0;

    TRACE!(b"(%p, %p)\n\0".as_ptr() as LPCSTR, hComm, lptimeouts);
    if lptimeouts.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    if DeviceIoControl(
        hComm,
        IOCTL_SERIAL_GET_TIMEOUTS,
        core::ptr::null_mut(),
        0,
        &mut st as *mut SERIAL_TIMEOUTS as LPVOID,
        core::mem::size_of::<SERIAL_TIMEOUTS>() as DWORD,
        &mut dwBytesReturned as LPDWORD,
        core::ptr::null_mut(),
    ) == 0
    {
        return FALSE;
    }
    (*lptimeouts).ReadIntervalTimeout = st.ReadIntervalTimeout;
    (*lptimeouts).ReadTotalTimeoutMultiplier = st.ReadTotalTimeoutMultiplier;
    (*lptimeouts).ReadTotalTimeoutConstant = st.ReadTotalTimeoutConstant;
    (*lptimeouts).WriteTotalTimeoutMultiplier = st.WriteTotalTimeoutMultiplier;
    (*lptimeouts).WriteTotalTimeoutConstant = st.WriteTotalTimeoutConstant;
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn SetCommTimeouts(
    hComm: HANDLE,
    lptimeouts: LPCOMMTIMEOUTS,
) -> BOOL {
    let mut st: SERIAL_TIMEOUTS = SERIAL_TIMEOUTS {
        ReadIntervalTimeout: 0,
        ReadTotalTimeoutMultiplier: 0,
        ReadTotalTimeoutConstant: 0,
        WriteTotalTimeoutMultiplier: 0,
        WriteTotalTimeoutConstant: 0,
    };
    let mut dwBytesReturned: DWORD = 0;

    TRACE!(
        b"(%p, %p)\n\0".as_ptr() as LPCSTR,
        hComm,
        lptimeouts,
    );

    if lptimeouts.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    st.ReadIntervalTimeout = (*lptimeouts).ReadIntervalTimeout;
    st.ReadTotalTimeoutMultiplier = (*lptimeouts).ReadTotalTimeoutMultiplier;
    st.ReadTotalTimeoutConstant = (*lptimeouts).ReadTotalTimeoutConstant;
    st.WriteTotalTimeoutMultiplier = (*lptimeouts).WriteTotalTimeoutMultiplier;
    st.WriteTotalTimeoutConstant = (*lptimeouts).WriteTotalTimeoutConstant;

    return DeviceIoControl(
        hComm,
        IOCTL_SERIAL_SET_TIMEOUTS,
        &raw mut st as LPVOID,
        core::mem::size_of::<SERIAL_TIMEOUTS>() as DWORD,
        core::ptr::null_mut(),
        0,
        &raw mut dwBytesReturned,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetCommModemStatus(
    hFile: HANDLE,
    lpModemStat: LPDWORD,
) -> BOOL {
    let mut dwBytesReturned: DWORD = 0;
    return DeviceIoControl(
        hFile,
        IOCTL_SERIAL_GET_MODEMSTATUS as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        lpModemStat as LPVOID,
        core::mem::size_of::<DWORD>() as DWORD,
        &mut dwBytesReturned as *mut DWORD as LPDWORD,
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "system" fn WaitCommEvent(
    hFile: HANDLE,
    lpdwEvents: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    return DeviceIoControl(
        hFile,
        IOCTL_SERIAL_WAIT_ON_MASK as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        lpdwEvents as LPVOID,
        core::mem::size_of::<DWORD>() as DWORD,
        core::ptr::null_mut::<DWORD>(),
        lpOverlapped,
    );
}
#[no_mangle]
pub unsafe extern "system" fn GetCommProperties(
    hFile: HANDLE,
    lpCommProp: LPCOMMPROP,
) -> BOOL {
    TRACE!(
        b"(%p %p)\0".as_ptr() as LPCSTR,
        hFile,
        lpCommProp,
    );
    if lpCommProp.is_null() {
        return FALSE;
    }
    memset(
        lpCommProp as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<COMMPROP>(),
    );
    (*lpCommProp).wPacketLength = 1;
    (*lpCommProp).wPacketVersion = 1;
    (*lpCommProp).dwServiceMask = SP_SERIALCOMM;
    (*lpCommProp).dwMaxTxQueue = 4096;
    (*lpCommProp).dwMaxRxQueue = 4096;
    (*lpCommProp).dwMaxBaud = BAUD_115200;
    (*lpCommProp).dwProvSubType = PST_RS232;
    (*lpCommProp).dwProvCapabilities = PCF_DTRDSR | PCF_PARITY_CHECK | PCF_RTSCTS | PCF_TOTALTIMEOUTS;
    (*lpCommProp).dwSettableParams = SP_BAUD | SP_DATABITS | SP_HANDSHAKING | SP_PARITY | SP_PARITY_CHECK | SP_STOPBITS;
    (*lpCommProp).dwSettableBaud = BAUD_075 | BAUD_110 | BAUD_134_5 | BAUD_150 | BAUD_300 | BAUD_600 | BAUD_1200 | BAUD_1800 | BAUD_2400 | BAUD_4800 | BAUD_9600 | BAUD_19200 | BAUD_38400 | BAUD_57600 | BAUD_115200;
    (*lpCommProp).wSettableData = (DATABITS_5 | DATABITS_6 | DATABITS_7 | DATABITS_8) as WORD;
    (*lpCommProp).wSettableStopParity = (STOPBITS_10 | STOPBITS_15 | STOPBITS_20 | PARITY_NONE | PARITY_ODD | PARITY_EVEN | PARITY_MARK | PARITY_SPACE) as WORD;
    (*lpCommProp).dwCurrentTxQueue = (*lpCommProp).dwMaxTxQueue;
    (*lpCommProp).dwCurrentRxQueue = (*lpCommProp).dwMaxRxQueue;
    TRUE
}
#[no_mangle]
pub unsafe extern "system" fn CommConfigDialogA(
    lpszDevice: LPCSTR,
    hWnd: HWND,
    lpCommConfig: LPCOMMCONFIG,
) -> BOOL {
    let mut lpDeviceW: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let mut r: BOOL = 0;

    TRACE!(
        "(%s, %p, %p)\n",
        debugstr_a(lpszDevice),
        hWnd,
        lpCommConfig,
    );

    if !lpszDevice.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            lpszDevice,
            -1,
            core::ptr::null_mut(),
            0,
        ) as DWORD;
        lpDeviceW = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as LPWSTR;
        MultiByteToWideChar(
            CP_ACP,
            0,
            lpszDevice,
            -1,
            lpDeviceW,
            len as i32,
        );
    }

    r = CommConfigDialogW(lpDeviceW as LPCWSTR, hWnd, lpCommConfig);
    HeapFree(GetProcessHeap(), 0, lpDeviceW as *mut core::ffi::c_void);
    return r;
}
#[no_mangle]
pub unsafe extern "system" fn CommConfigDialogW(
    lpszDevice: LPCWSTR,
    hWnd: HWND,
    lpCommConfig: LPCOMMCONFIG,
) -> BOOL {
    let mut pCommConfigDialog: Option<unsafe extern "system" fn(LPCWSTR, HWND, LPCOMMCONFIG) -> DWORD>;
    let mut hConfigModule: HMODULE;
    let mut res: DWORD = ERROR_INVALID_PARAMETER;

    TRACE!(
        b"(%s, %p, %p)\n\0".as_ptr() as LPCSTR,
        debugstr_w(lpszDevice),
        hWnd,
        lpCommConfig,
    );
    hConfigModule = LoadLibraryW((*(&raw const lpszSerialUI)).as_ptr());
    if !hConfigModule.is_null() {
        pCommConfigDialog = core::mem::transmute::<FARPROC, Option<unsafe extern "system" fn(LPCWSTR, HWND, LPCOMMCONFIG) -> DWORD>>(
            GetProcAddress(hConfigModule, b"drvCommConfigDialogW\0".as_ptr() as LPCSTR),
        );
        if pCommConfigDialog.is_some() {
            res = pCommConfigDialog.unwrap()(lpszDevice, hWnd, lpCommConfig);
        }
        FreeLibrary(hConfigModule);
    }
    if res != 0 {
        SetLastError(res);
    }
    return (res == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn GetCommConfig(
    hFile: HANDLE,
    lpCommConfig: LPCOMMCONFIG,
    lpdwSize: LPDWORD,
) -> BOOL {
    let mut r: BOOL;

    TRACE!(
        b"(%p, %p, %p) *lpdwSize: %u\n\0".as_ptr() as LPCSTR,
        hFile,
        lpCommConfig,
        lpdwSize,
        if !lpdwSize.is_null() { *lpdwSize } else { 0 },
    );

    if lpCommConfig.is_null() {
        return FALSE;
    }

    r = ((*lpdwSize as usize) < core::mem::size_of::<COMMCONFIG>() as usize) as BOOL;
    *lpdwSize = core::mem::size_of::<COMMCONFIG>() as DWORD;
    if r != 0 {
        return FALSE;
    }

    (*lpCommConfig).dwSize = core::mem::size_of::<COMMCONFIG>() as DWORD;
    (*lpCommConfig).wVersion = 1 as WORD;
    (*lpCommConfig).wReserved = 0 as WORD;
    r = GetCommState(hFile, &raw mut (*lpCommConfig).dcb);
    (*lpCommConfig).dwProviderSubType = PST_RS232 as DWORD;
    (*lpCommConfig).dwProviderOffset = 0 as DWORD;
    (*lpCommConfig).dwProviderSize = 0 as DWORD;

    return r;
}
#[no_mangle]
pub unsafe extern "system" fn SetCommConfig(
    hFile: HANDLE,
    lpCommConfig: LPCOMMCONFIG,
    dwSize: DWORD,
) -> BOOL {
    TRACE!("(%p, %p, %u)\n", hFile, lpCommConfig, dwSize);
    return SetCommState(hFile, &raw mut (*lpCommConfig).dcb);
}

#[no_mangle]
pub unsafe extern "system" fn SetDefaultCommConfigW(
    lpszDevice: LPCWSTR,
    lpCommConfig: LPCOMMCONFIG,
    dwSize: DWORD,
) -> BOOL {
    let mut lpfnSetDefaultCommConfig: Option<
        unsafe extern "system" fn(LPCWSTR, LPCOMMCONFIG, DWORD) -> BOOL,
    > = None;
    let mut hConfigModule: HMODULE = core::ptr::null_mut();
    let mut r: BOOL = 0;

    TRACE!(
        b"(%s, %p, %u)\n\0".as_ptr() as LPCSTR,
        debugstr_w(lpszDevice),
        lpCommConfig,
        dwSize,
    );

    hConfigModule = LoadLibraryW((*(&raw const lpszSerialUI)).as_ptr() as LPCWSTR);
    if hConfigModule.is_null() {
        return r;
    }

    lpfnSetDefaultCommConfig = core::mem::transmute::<
        FARPROC,
        Option<unsafe extern "system" fn(LPCWSTR, LPCOMMCONFIG, DWORD) -> BOOL>,
    >(
        GetProcAddress(
            hConfigModule,
            b"drvSetDefaultCommConfigW\0".as_ptr() as LPCSTR,
        ),
    );

    if lpfnSetDefaultCommConfig.is_some() {
        r = lpfnSetDefaultCommConfig.expect("non-null function pointer")(
            lpszDevice,
            lpCommConfig,
            dwSize,
        );
    }

    FreeLibrary(hConfigModule);

    r
}

#[no_mangle]
pub unsafe extern "system" fn SetDefaultCommConfigA(
    lpszDevice: LPCSTR,
    lpCommConfig: LPCOMMCONFIG,
    dwSize: DWORD,
) -> BOOL {
    let mut r: BOOL = 0;
    let mut lpDeviceW: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;

    TRACE!(
        "(%s, %p, %u)\n\0".as_ptr() as LPCSTR,
        debugstr_a(lpszDevice),
        lpCommConfig,
        dwSize,
    );

    if !lpszDevice.is_null() {
        len = MultiByteToWideChar(
            CP_ACP as UINT,
            0,
            lpszDevice,
            -1,
            core::ptr::null_mut(),
            0,
        ) as DWORD;
        lpDeviceW = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as LPWSTR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0,
            lpszDevice,
            -1,
            lpDeviceW,
            len as i32,
        );
    }
    r = SetDefaultCommConfigW(lpDeviceW as LPCWSTR, lpCommConfig, dwSize);
    HeapFree(GetProcessHeap(), 0, lpDeviceW as *mut core::ffi::c_void);
    return r;
}
#[no_mangle]
pub unsafe extern "system" fn GetDefaultCommConfigW(
    lpszName: LPCWSTR,
    lpCC: LPCOMMCONFIG,
    lpdwSize: LPDWORD,
) -> BOOL {
    let pGetDefaultCommConfig: Option<
        unsafe extern "system" fn(LPCWSTR, LPCOMMCONFIG, LPDWORD) -> DWORD,
    >;
    let hConfigModule: HMODULE;
    let mut res: DWORD = ERROR_INVALID_PARAMETER;

    TRACE!(
        b"(%s, %p, %p)  *lpdwSize: %u\n\0".as_ptr() as LPCSTR,
        debugstr_w(lpszName),
        lpCC,
        lpdwSize,
        if !lpdwSize.is_null() { *lpdwSize } else { 0 },
    );

    hConfigModule = LoadLibraryW((*(&raw const lpszSerialUI)).as_ptr() as LPCWSTR);

    if !hConfigModule.is_null() {
        pGetDefaultCommConfig = core::mem::transmute::<
            FARPROC,
            Option<unsafe extern "system" fn(LPCWSTR, LPCOMMCONFIG, LPDWORD) -> DWORD>,
        >(GetProcAddress(
            hConfigModule,
            b"drvGetDefaultCommConfigW\0".as_ptr() as LPCSTR,
        ));

        if let Some(func) = pGetDefaultCommConfig {
            res = func(lpszName, lpCC, lpdwSize);
        }

        FreeLibrary(hConfigModule);
    }

    if res != 0 {
        SetLastError(res);
    }

    return (res == ERROR_SUCCESS) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn GetDefaultCommConfigA(
    lpszName: LPCSTR,
    lpCC: LPCOMMCONFIG,
    lpdwSize: LPDWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut lpszNameW: _UNICODE_STRING = core::mem::zeroed();

    if !lpszName.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut lpszNameW, lpszName);
    } else {
        lpszNameW.Buffer = core::ptr::null_mut();
    }

    ret = GetDefaultCommConfigW(lpszNameW.Buffer, lpCC, lpdwSize);

    RtlFreeUnicodeString(&mut lpszNameW);
    return ret;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * DEC 93 Erik Bos <erik@xs4all.nl>
//  *
//  * Copyright 1996 Marcus Meissner
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <k32.h>
//
// typedef LARGE_INTEGER PHYSICAL_ADDRESS, *PPHYSICAL_ADDRESS;
// #undef SERIAL_LSRMST_ESCAPE
// #undef SERIAL_LSRMST_LSR_DATA
// #undef SERIAL_LSRMST_LSR_NODATA
// #undef SERIAL_LSRMST_MST
// #undef SERIAL_IOC_FCR_FIFO_ENABLE
// #undef SERIAL_IOC_FCR_RCVR_RESET
// #undef SERIAL_IOC_FCR_XMIT_RESET
// #undef SERIAL_IOC_FCR_DMA_MODE
// #undef SERIAL_IOC_FCR_RES1
// #undef SERIAL_IOC_FCR_RES2
// #undef SERIAL_IOC_FCR_RCVR_TRIGGER_LSB
// #undef SERIAL_IOC_FCR_RCVR_TRIGGER_MSB
// #undef SERIAL_IOC_MCR_DTR
// #undef SERIAL_IOC_MCR_RTS
// #undef SERIAL_IOC_MCR_OUT1
// #undef SERIAL_IOC_MCR_OUT2
// #undef SERIAL_IOC_MCR_LOOP
// #undef IOCTL_SERIAL_LSRMST_INSERT
// #include <ntddser.h>
//
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(comm);
//
// /***********************************************************************
//  *           COMM_Parse*   (Internal)
//  *
//  *  The following COMM_Parse* functions are used by the BuildCommDCB
//  *  functions to help parse the various parts of the device control string.
//  */
// static LPCWSTR COMM_ParseStart(LPCWSTR ptr)
// {
// 	static const WCHAR comW[] = {'C','O','M',0};
//
// 	/* The device control string may optionally start with "COMx" followed
// 	   by an optional ':' and spaces. */
// 	if(!strncmpiW(ptr, comW, 3))
// 	{
// 		ptr += 3;
//
// 		/* Allow any com port above 0 as Win 9x does (NT only allows
// 		   values for com ports which are actually present) */
// 		if(*ptr < '1' || *ptr > '9')
// 			return NULL;
//
// 		/* Advance pointer past port number */
// 		while(*ptr >= '0' && *ptr <= '9') ptr++;
//
// 		/* The com port number must be followed by a ':' or ' ' */
// 		if(*ptr != ':' && *ptr != ' ')
// 			return NULL;
//
// 		/* Advance pointer to beginning of next parameter */
// 		while(*ptr == ' ') ptr++;
// 		if(*ptr == ':')
// 		{
// 			ptr++;
// 			while(*ptr == ' ') ptr++;
// 		}
// 	}
// 	/* The device control string must not start with a space. */
// 	else if(*ptr == ' ')
// 		return NULL;
//
// 	return ptr;
// }
//
// static LPCWSTR COMM_ParseNumber(LPCWSTR ptr, LPDWORD lpnumber)
// {
// 	if(*ptr < '0' || *ptr > '9') return NULL;
// 	*lpnumber = strtoulW(ptr, NULL, 10);
// 	while(*ptr >= '0' && *ptr <= '9') ptr++;
// 	return ptr;
// }
//
// static LPCWSTR COMM_ParseParity(LPCWSTR ptr, LPBYTE lpparity)
// {
// 	/* Contrary to what you might expect, Windows only sets the Parity
// 	   member of DCB and not fParity even when parity is specified in the
// 	   device control string */
//
// 	switch(toupperW(*ptr++))
// 	{
// 	case 'E':
// 		*lpparity = EVENPARITY;
// 		break;
// 	case 'M':
// 		*lpparity = MARKPARITY;
// 		break;
// 	case 'N':
// 		*lpparity = NOPARITY;
// 		break;
// 	case 'O':
// 		*lpparity = ODDPARITY;
// 		break;
// 	case 'S':
// 		*lpparity = SPACEPARITY;
// 		break;
// 	default:
// 		return NULL;
// 	}
//
// 	return ptr;
// }
//
// static LPCWSTR COMM_ParseByteSize(LPCWSTR ptr, LPBYTE lpbytesize)
// {
// 	DWORD temp;
//
// 	if(!(ptr = COMM_ParseNumber(ptr, &temp)))
// 		return NULL;
//
// 	if(temp >= 5 && temp <= 8)
// 	{
// 		*lpbytesize = temp;
// 		return ptr;
// 	}
// 	else
// 		return NULL;
// }
//
// static LPCWSTR COMM_ParseStopBits(LPCWSTR ptr, LPBYTE lpstopbits)
// {
// 	DWORD temp;
// 	static const WCHAR stopbits15W[] = {'1','.','5',0};
//
// 	if(!strncmpW(stopbits15W, ptr, 3))
// 	{
// 		ptr += 3;
// 		*lpstopbits = ONE5STOPBITS;
// 	}
// 	else
// 	{
// 		if(!(ptr = COMM_ParseNumber(ptr, &temp)))
// 			return NULL;
//
// 		if(temp == 1)
// 			*lpstopbits = ONESTOPBIT;
// 		else if(temp == 2)
// 			*lpstopbits = TWOSTOPBITS;
// 		else
// 			return NULL;
// 	}
//
// 	return ptr;
// }
//
// static LPCWSTR COMM_ParseOnOff(LPCWSTR ptr, LPDWORD lponoff)
// {
// 	static const WCHAR onW[] = {'o','n',0};
// 	static const WCHAR offW[] = {'o','f','f',0};
//
// 	if(!strncmpiW(onW, ptr, 2))
// 	{
// 		ptr += 2;
// 		*lponoff = 1;
// 	}
// 	else if(!strncmpiW(offW, ptr, 3))
// 	{
// 		ptr += 3;
// 		*lponoff = 0;
// 	}
// 	else
// 		return NULL;
//
// 	return ptr;
// }
//
// /***********************************************************************
//  *           COMM_BuildOldCommDCB   (Internal)
//  *
//  *  Build a DCB using the old style settings string eg: "96,n,8,1"
//  */
// static BOOL COMM_BuildOldCommDCB(LPCWSTR device, LPDCB lpdcb)
// {
// 	WCHAR last = 0;
//
// 	if(!(device = COMM_ParseNumber(device, &lpdcb->BaudRate)))
// 		return FALSE;
//
// 	switch(lpdcb->BaudRate)
// 	{
// 	case 11:
// 	case 30:
// 	case 60:
// 		lpdcb->BaudRate *= 10;
// 		break;
// 	case 12:
// 	case 24:
// 	case 48:
// 	case 96:
// 		lpdcb->BaudRate *= 100;
// 		break;
// 	case 19:
// 		lpdcb->BaudRate = 19200;
// 		break;
// 	}
//
// 	while(*device == ' ') device++;
// 	if(*device++ != ',') return FALSE;
// 	while(*device == ' ') device++;
//
// 	if(!(device = COMM_ParseParity(device, &lpdcb->Parity)))
// 		return FALSE;
//
// 	while(*device == ' ') device++;
// 	if(*device++ != ',') return FALSE;
// 	while(*device == ' ') device++;
//
// 	if(!(device = COMM_ParseByteSize(device, &lpdcb->ByteSize)))
// 		return FALSE;
//
// 	while(*device == ' ') device++;
// 	if(*device++ != ',') return FALSE;
// 	while(*device == ' ') device++;
//
// 	if(!(device = COMM_ParseStopBits(device, &lpdcb->StopBits)))
// 		return FALSE;
//
// 	/* The last parameter for flow control is optional. */
// 	while(*device == ' ') device++;
// 	if(*device == ',')
// 	{
// 		device++;
// 		while(*device == ' ') device++;
// 		if(*device) last = toupperW(*device++);
// 		while(*device == ' ') device++;
// 	}
//
// 	/* Win NT sets the flow control members based on (or lack of) the last
// 	   parameter.  Win 9x does not set these members. */
// 	switch(last)
// 	{
// 	case 0:
// 		lpdcb->fInX = FALSE;
// 		lpdcb->fOutX = FALSE;
// 		lpdcb->fOutxCtsFlow = FALSE;
// 		lpdcb->fOutxDsrFlow = FALSE;
// 		lpdcb->fDtrControl = DTR_CONTROL_ENABLE;
// 		lpdcb->fRtsControl = RTS_CONTROL_ENABLE;
// 		break;
// 	case 'X':
// 		lpdcb->fInX = TRUE;
// 		lpdcb->fOutX = TRUE;
// 		lpdcb->fOutxCtsFlow = FALSE;
// 		lpdcb->fOutxDsrFlow = FALSE;
// 		lpdcb->fDtrControl = DTR_CONTROL_ENABLE;
// 		lpdcb->fRtsControl = RTS_CONTROL_ENABLE;
// 		break;
// 	case 'P':
// 		lpdcb->fInX = FALSE;
// 		lpdcb->fOutX = FALSE;
// 		lpdcb->fOutxCtsFlow = TRUE;
// 		lpdcb->fOutxDsrFlow = TRUE;
// 		lpdcb->fDtrControl = DTR_CONTROL_HANDSHAKE;
// 		lpdcb->fRtsControl = RTS_CONTROL_HANDSHAKE;
// 		break;
// 	default:
// 		return FALSE;
// 	}
//
// 	/* This should be the end of the string. */
// 	if(*device) return FALSE;
//
// 	return TRUE;
// }
//
// /***********************************************************************
//  *           COMM_BuildNewCommDCB   (Internal)
//  *
//  *  Build a DCB using the new style settings string.
//  *   eg: "baud=9600 parity=n data=8 stop=1 xon=on to=on"
//  */
// static BOOL COMM_BuildNewCommDCB(LPCWSTR device, LPDCB lpdcb, LPCOMMTIMEOUTS lptimeouts)
// {
// 	DWORD temp;
// 	BOOL baud = FALSE, stop = FALSE;
// 	static const WCHAR baudW[] = {'b','a','u','d','=',0};
// 	static const WCHAR parityW[] = {'p','a','r','i','t','y','=',0};
// 	static const WCHAR dataW[] = {'d','a','t','a','=',0};
// 	static const WCHAR stopW[] = {'s','t','o','p','=',0};
// 	static const WCHAR toW[] = {'t','o','=',0};
// 	static const WCHAR xonW[] = {'x','o','n','=',0};
// 	static const WCHAR odsrW[] = {'o','d','s','r','=',0};
// 	static const WCHAR octsW[] = {'o','c','t','s','=',0};
// 	static const WCHAR dtrW[] = {'d','t','r','=',0};
// 	static const WCHAR rtsW[] = {'r','t','s','=',0};
// 	static const WCHAR idsrW[] = {'i','d','s','r','=',0};
//
// 	while(*device)
// 	{
// 		while(*device == ' ') device++;
//
// 		if(!strncmpiW(baudW, device, 5))
// 		{
// 			baud = TRUE;
//
// 			if(!(device = COMM_ParseNumber(device + 5, &lpdcb->BaudRate)))
// 				return FALSE;
// 		}
// 		else if(!strncmpiW(parityW, device, 7))
// 		{
// 			if(!(device = COMM_ParseParity(device + 7, &lpdcb->Parity)))
// 				return FALSE;
// 		}
// 		else if(!strncmpiW(dataW, device, 5))
// 		{
// 			if(!(device = COMM_ParseByteSize(device + 5, &lpdcb->ByteSize)))
// 				return FALSE;
// 		}
// 		else if(!strncmpiW(stopW, device, 5))
// 		{
// 			stop = TRUE;
//
// 			if(!(device = COMM_ParseStopBits(device + 5, &lpdcb->StopBits)))
// 				return FALSE;
// 		}
// 		else if(!strncmpiW(toW, device, 3))
// 		{
// 			if(!(device = COMM_ParseOnOff(device + 3, &temp)))
// 				return FALSE;
//
// 			lptimeouts->ReadIntervalTimeout = 0;
// 			lptimeouts->ReadTotalTimeoutMultiplier = 0;
// 			lptimeouts->ReadTotalTimeoutConstant = 0;
// 			lptimeouts->WriteTotalTimeoutMultiplier = 0;
// 			lptimeouts->WriteTotalTimeoutConstant = temp ? 60000 : 0;
// 		}
// 		else if(!strncmpiW(xonW, device, 4))
// 		{
// 			if(!(device = COMM_ParseOnOff(device + 4, &temp)))
// 				return FALSE;
//
// 			lpdcb->fOutX = temp;
// 			lpdcb->fInX = temp;
// 		}
// 		else if(!strncmpiW(odsrW, device, 5))
// 		{
// 			if(!(device = COMM_ParseOnOff(device + 5, &temp)))
// 				return FALSE;
//
// 			lpdcb->fOutxDsrFlow = temp;
// 		}
// 		else if(!strncmpiW(octsW, device, 5))
// 		{
// 			if(!(device = COMM_ParseOnOff(device + 5, &temp)))
// 				return FALSE;
//
// 			lpdcb->fOutxCtsFlow = temp;
// 		}
// 		else if(!strncmpiW(dtrW, device, 4))
// 		{
// 			if(!(device = COMM_ParseOnOff(device + 4, &temp)))
// 				return FALSE;
//
// 			lpdcb->fDtrControl = temp;
// 		}
// 		else if(!strncmpiW(rtsW, device, 4))
// 		{
// 			if(!(device = COMM_ParseOnOff(device + 4, &temp)))
// 				return FALSE;
//
// 			lpdcb->fRtsControl = temp;
// 		}
// 		else if(!strncmpiW(idsrW, device, 5))
// 		{
// 			if(!(device = COMM_ParseOnOff(device + 5, &temp)))
// 				return FALSE;
//
// 			/* Win NT sets the fDsrSensitivity member based on the
// 			   idsr parameter.  Win 9x sets fOutxDsrFlow instead. */
// 			lpdcb->fDsrSensitivity = temp;
// 		}
// 		else
// 			return FALSE;
//
// 		/* After the above parsing, the next character (if not the end of
// 		   the string) should be a space */
// 		if(*device && *device != ' ')
// 			return FALSE;
// 	}
//
// 	/* If stop bits were not specified, a default is always supplied. */
// 	if(!stop)
// 	{
// 		if(baud && lpdcb->BaudRate == 110)
// 			lpdcb->StopBits = TWOSTOPBITS;
// 		else
// 			lpdcb->StopBits = ONESTOPBIT;
// 	}
//
// 	return TRUE;
// }
//
// /**************************************************************************
//  *         BuildCommDCBA		(KERNEL32.@)
//  *
//  *  Updates a device control block data structure with values from an
//  *  ascii device control string.  The device control string has two forms
//  *  normal and extended, it must be exclusively in one or the other form.
//  *
//  * RETURNS
//  *
//  *  True on success, false on a malformed control string.
//  */
// BOOL WINAPI BuildCommDCBA(
//     LPCSTR device, /* [in] The ascii device control string used to update the DCB. */
//     LPDCB  lpdcb)  /* [out] The device control block to be updated. */
// {
// 	return BuildCommDCBAndTimeoutsA(device,lpdcb,NULL);
// }
//
// /**************************************************************************
//  *         BuildCommDCBAndTimeoutsA		(KERNEL32.@)
//  *
//  *  Updates a device control block data structure with values from an
//  *  ascii device control string.  Taking timeout values from a timeouts
//  *  struct if desired by the control string.
//  *
//  * RETURNS
//  *
//  *  True on success, false bad handles etc.
//  */
// BOOL WINAPI BuildCommDCBAndTimeoutsA(
//     LPCSTR         device,     /* [in] The ascii device control string. */
//     LPDCB          lpdcb,      /* [out] The device control block to be updated. */
//     LPCOMMTIMEOUTS lptimeouts) /* [in] The COMMTIMEOUTS structure to be updated. */
// {
// 	BOOL ret = FALSE;
// 	UNICODE_STRING deviceW;
//
// 	TRACE!("(%s,%p,%p)\n",device,lpdcb,lptimeouts);
// 	if(device) RtlCreateUnicodeStringFromAsciiz(&deviceW,device);
// 	else deviceW.Buffer = NULL;
//
// 	if(deviceW.Buffer) ret = BuildCommDCBAndTimeoutsW(deviceW.Buffer,lpdcb,lptimeouts);
//
// 	RtlFreeUnicodeString(&deviceW);
// 	return ret;
// }
//
// /**************************************************************************
//  *         BuildCommDCBAndTimeoutsW	(KERNEL32.@)
//  *
//  *  Updates a device control block data structure with values from a
//  *  unicode device control string.  Taking timeout values from a timeouts
//  *  struct if desired by the control string.
//  *
//  * RETURNS
//  *
//  *  True on success, false bad handles etc
//  */
// BOOL WINAPI BuildCommDCBAndTimeoutsW(
//     LPCWSTR        devid,      /* [in] The unicode device control string. */
//     LPDCB          lpdcb,      /* [out] The device control block to be updated. */
//     LPCOMMTIMEOUTS lptimeouts) /* [in] The COMMTIMEOUTS structure to be updated. */
// {
// 	DCB dcb;
// 	COMMTIMEOUTS timeouts;
// 	BOOL result;
// 	LPCWSTR ptr = devid;
//
// 	TRACE!("(%s,%p,%p)\n",debugstr_w(devid),lpdcb,lptimeouts);
//
// 	memset(&timeouts, 0, sizeof timeouts);
//
// 	/* Set DCBlength. (Windows NT does not do this, but 9x does) */
// 	lpdcb->DCBlength = sizeof(DCB);
//
// 	/* Make a copy of the original data structures to work with since if
// 	   if there is an error in the device control string the originals
// 	   should not be modified (except possibly DCBlength) */
// 	dcb = *lpdcb;
// 	if(lptimeouts) timeouts = *lptimeouts;
//
// 	ptr = COMM_ParseStart(ptr);
//
// 	if(ptr == NULL)
// 		result = FALSE;
// 	else if(strchrW(ptr, ','))
// 		result = COMM_BuildOldCommDCB(ptr, &dcb);
// 	else
// 		result = COMM_BuildNewCommDCB(ptr, &dcb, &timeouts);
//
// 	if(result)
// 	{
// 		*lpdcb = dcb;
// 		if(lptimeouts) *lptimeouts = timeouts;
// 		return TRUE;
// 	}
// 	else
// 	{
// 		WARN!("Invalid device control string: %s\n", debugstr_w(devid));
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
// }
//
// /**************************************************************************
//  *         BuildCommDCBW		(KERNEL32.@)
//  *
//  *  Updates a device control block structure with values from an
//  *  unicode device control string.  The device control string has two forms
//  *  normal and extended, it must be exclusively in one or the other form.
//  *
//  * RETURNS
//  *
//  *  True on success, false on a malformed control string.
//  */
// BOOL WINAPI BuildCommDCBW(
//     LPCWSTR devid, /* [in] The unicode device control string. */
//     LPDCB   lpdcb) /* [out] The device control block to be updated. */
// {
// 	return BuildCommDCBAndTimeoutsW(devid,lpdcb,NULL);
// }
//
// /*****************************************************************************
//  *	SetCommBreak		(KERNEL32.@)
//  *
//  *  Halts the transmission of characters to a communications device.
//  *
//  * PARAMS
//  *      handle  [in] The communications device to suspend
//  *
//  * RETURNS
//  *
//  *  True on success, and false if the communications device could not be found,
//  *  the control is not supported.
//  *
//  * BUGS
//  *
//  *  Only TIOCSBRK and TIOCCBRK are supported.
//  */
// BOOL WINAPI SetCommBreak(HANDLE handle)
// {
//     DWORD dwBytesReturned;
//     return DeviceIoControl(handle, IOCTL_SERIAL_SET_BREAK_ON, NULL, 0, NULL, 0, &dwBytesReturned, NULL);
// }
//
// /*****************************************************************************
//  *	ClearCommBreak		(KERNEL32.@)
//  *
//  *  Resumes character transmission from a communication device.
//  *
//  * PARAMS
//  *
//  *      handle [in] The halted communication device whose character transmission is to be resumed
//  *
//  * RETURNS
//  *
//  *  True on success and false if the communications device could not be found.
//  *
//  * BUGS
//  *
//  *  Only TIOCSBRK and TIOCCBRK are supported.
//  */
// BOOL WINAPI ClearCommBreak(HANDLE handle)
// {
//     DWORD dwBytesReturned;
//     return DeviceIoControl(handle, IOCTL_SERIAL_SET_BREAK_OFF, NULL, 0, NULL, 0, &dwBytesReturned, NULL);
// }
//
// /*****************************************************************************
//  *	EscapeCommFunction	(KERNEL32.@)
//  *
//  *  Directs a communication device to perform an extended function.
//  *
//  * PARAMS
//  *
//  *      handle          [in] The communication device to perform the extended function
//  *      nFunction       [in] The extended function to be performed
//  *
//  * RETURNS
//  *
//  *  True or requested data on successful completion of the command,
//  *  false if the device is not present cannot execute the command
//  *  or the command failed.
//  */
// BOOL WINAPI EscapeCommFunction(HANDLE handle, DWORD func)
// {
//     DWORD       ioc;
//     DWORD dwBytesReturned;
//
//     switch (func)
//     {
//     case CLRDTR:        ioc = IOCTL_SERIAL_CLR_DTR;             break;
//     case CLRRTS:        ioc = IOCTL_SERIAL_CLR_RTS;             break;
//     case SETDTR:        ioc = IOCTL_SERIAL_SET_DTR;             break;
//     case SETRTS:        ioc = IOCTL_SERIAL_SET_RTS;             break;
//     case SETXOFF:       ioc = IOCTL_SERIAL_SET_XOFF;            break;
//     case SETXON:        ioc = IOCTL_SERIAL_SET_XON;             break;
//     case SETBREAK:      ioc = IOCTL_SERIAL_SET_BREAK_ON;        break;
//     case CLRBREAK:      ioc = IOCTL_SERIAL_SET_BREAK_OFF;       break;
//     case RESETDEV:      ioc = IOCTL_SERIAL_RESET_DEVICE;        break;
//     default:
//         ERR!("Unknown function code (%u)\n", func);
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//     return DeviceIoControl(handle, ioc, NULL, 0, NULL, 0, &dwBytesReturned, NULL);
// }
//
// /********************************************************************
//  *      PurgeComm        (KERNEL32.@)
//  *
//  *  Terminates pending operations and/or discards buffers on a
//  *  communication resource.
//  *
//  * PARAMS
//  *
//  *      handle  [in] The communication resource to be purged
//  *      flags   [in] Flags for clear pending/buffer on input/output
//  *
//  * RETURNS
//  *
//  *  True on success and false if the communications handle is bad.
//  */
// BOOL WINAPI PurgeComm(HANDLE handle, DWORD flags)
// {
//     DWORD dwBytesReturned;
//     return DeviceIoControl(handle, IOCTL_SERIAL_PURGE, &flags, sizeof(flags),
//                            NULL, 0, &dwBytesReturned, NULL);
// }
//
// /*****************************************************************************
//  *	ClearCommError	(KERNEL32.@)
//  *
//  *  Enables further I/O operations on a communications resource after
//  *  supplying error and current status information.
//  *
//  * PARAMS
//  *
//  *      handle  [in]    The communication resource with the error
//  *      errors  [out]   Flags indicating error the resource experienced
//  *      lpStat  [out] The status of the communication resource
//  * RETURNS
//  *
//  *  True on success, false if the communication resource handle is bad.
//  */
// BOOL WINAPI ClearCommError(HANDLE handle, LPDWORD errors, LPCOMSTAT lpStat)
// {
//     SERIAL_STATUS       ss;
//     DWORD dwBytesReturned;
//
//     if (!DeviceIoControl(handle, IOCTL_SERIAL_GET_COMMSTATUS, NULL, 0,
//                          &ss, sizeof(ss), &dwBytesReturned, NULL))
//         return FALSE;
//
//     if (errors)
//     {
//         *errors = 0;
//         if (ss.Errors & SERIAL_ERROR_BREAK)             *errors |= CE_BREAK;
//         if (ss.Errors & SERIAL_ERROR_FRAMING)           *errors |= CE_FRAME;
//         if (ss.Errors & SERIAL_ERROR_OVERRUN)           *errors |= CE_OVERRUN;
//         if (ss.Errors & SERIAL_ERROR_QUEUEOVERRUN)      *errors |= CE_RXOVER;
//         if (ss.Errors & SERIAL_ERROR_PARITY)            *errors |= CE_RXPARITY;
//     }
//
//     if (lpStat)
//     {
//         memset(lpStat, 0, sizeof(*lpStat));
//
//         if (ss.HoldReasons & SERIAL_TX_WAITING_FOR_CTS)         lpStat->fCtsHold = TRUE;
//         if (ss.HoldReasons & SERIAL_TX_WAITING_FOR_DSR)         lpStat->fDsrHold = TRUE;
//         if (ss.HoldReasons & SERIAL_TX_WAITING_FOR_DCD)         lpStat->fRlsdHold = TRUE;
//         if (ss.HoldReasons & SERIAL_TX_WAITING_FOR_XON)         lpStat->fXoffHold = TRUE;
//         if (ss.HoldReasons & SERIAL_TX_WAITING_XOFF_SENT)       lpStat->fXoffSent = TRUE;
//         if (ss.EofReceived)                                     lpStat->fEof = TRUE;
//         if (ss.WaitForImmediate)                                lpStat->fTxim = TRUE;
//         lpStat->cbInQue = ss.AmountInInQueue;
//         lpStat->cbOutQue = ss.AmountInOutQueue;
//     }
//     return TRUE;
// }
//
// /*****************************************************************************
//  *      SetupComm       (KERNEL32.@)
//  *
//  *  Called after CreateFile to hint to the communication resource to use
//  *  specified sizes for input and output buffers rather than the default values.
//  *
//  * PARAMS
//  *      handle  [in]    The just created communication resource handle
//  *      insize  [in]    The suggested size of the communication resources input buffer in bytes
//  *      outsize [in]    The suggested size of the communication resources output buffer in bytes
//  *
//  * RETURNS
//  *
//  *  True if successful, false if the communications resource handle is bad.
//  *
//  * BUGS
//  *
//  *  Stub.
//  */
// BOOL WINAPI SetupComm(HANDLE handle, DWORD insize, DWORD outsize)
// {
//     SERIAL_QUEUE_SIZE   sqs;
//     DWORD dwBytesReturned;
//
//     sqs.InSize = insize;
//     sqs.OutSize = outsize;
//     return DeviceIoControl(handle, IOCTL_SERIAL_SET_QUEUE_SIZE,
//                            &sqs, sizeof(sqs), NULL, 0, &dwBytesReturned, NULL);
// }
//
// /*****************************************************************************
//  *	GetCommMask	(KERNEL32.@)
//  *
//  *  Obtain the events associated with a communication device that will cause
//  *  a call WaitCommEvent to return.
//  *
//  *  PARAMS
//  *
//  *      handle  [in]    The communications device
//  *      evtmask [out]   The events which cause WaitCommEvent to return
//  *
//  *  RETURNS
//  *
//  *   True on success, fail on bad device handle etc.
//  */
// BOOL WINAPI GetCommMask(HANDLE handle, LPDWORD evtmask)
// {
//     DWORD dwBytesReturned;
//     TRACE!("handle %p, mask %p\n", handle, evtmask);
//     return DeviceIoControl(handle, IOCTL_SERIAL_GET_WAIT_MASK,
//                            NULL, 0, evtmask, sizeof(*evtmask), &dwBytesReturned, NULL);
// }
//
// /*****************************************************************************
//  *	SetCommMask	(KERNEL32.@)
//  *
//  *  There be some things we need to hear about yon there communications device.
//  *  (Set which events associated with a communication device should cause
//  *  a call WaitCommEvent to return.)
//  *
//  * PARAMS
//  *
//  *      handle  [in]    The communications device
//  *      evtmask [in]    The events that are to be monitored
//  *
//  * RETURNS
//  *
//  *  True on success, false on bad handle etc.
//  */
// BOOL WINAPI SetCommMask(HANDLE handle, DWORD evtmask)
// {
//     DWORD dwBytesReturned;
//     TRACE!("handle %p, mask %x\n", handle, evtmask);
//     return DeviceIoControl(handle, IOCTL_SERIAL_SET_WAIT_MASK,
//                            &evtmask, sizeof(evtmask), NULL, 0, &dwBytesReturned, NULL);
// }
//
// static void dump_dcb(const DCB* lpdcb)
// {
//     TRACE!("bytesize=%d baudrate=%d fParity=%d Parity=%d stopbits=%d\n",
//           lpdcb->ByteSize, lpdcb->BaudRate, lpdcb->fParity, lpdcb->Parity,
//           (lpdcb->StopBits == ONESTOPBIT) ? 1 :
//           (lpdcb->StopBits == TWOSTOPBITS) ? 2 : 0);
//     TRACE!("%sIXON %sIXOFF\n", (lpdcb->fOutX) ? "" : "~", (lpdcb->fInX) ? "" : "~");
//     TRACE!("fOutxCtsFlow=%d fRtsControl=%d\n", lpdcb->fOutxCtsFlow, lpdcb->fRtsControl);
//     TRACE!("fOutxDsrFlow=%d fDtrControl=%d\n", lpdcb->fOutxDsrFlow, lpdcb->fDtrControl);
//     if (lpdcb->fOutxCtsFlow || lpdcb->fRtsControl == RTS_CONTROL_HANDSHAKE)
//         TRACE!("CRTSCTS\n");
//     else
//         TRACE!("~CRTSCTS\n");
// }
//
// /*****************************************************************************
//  *	SetCommState    (KERNEL32.@)
//  *
//  *  Re-initializes all hardware and control settings of a communications device,
//  *  with values from a device control block without affecting the input and output
//  *  queues.
//  *
//  * PARAMS
//  *
//  *      handle  [in]    The communications device
//  *      lpdcb   [out]   The device control block
//  *
//  * RETURNS
//  *
//  *  True on success, false on failure, e.g., if the XonChar is equal to the XoffChar.
//  */
// BOOL WINAPI SetCommState( HANDLE handle, LPDCB lpdcb)
// {
//     SERIAL_BAUD_RATE           sbr;
//     SERIAL_LINE_CONTROL        slc;
//     SERIAL_HANDFLOW            shf;
//     SERIAL_CHARS               sc;
//     DWORD dwBytesReturned;
//
//     TRACE!("handle %p, ptr %p\n", handle, lpdcb);
//
//     if (lpdcb == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//     dump_dcb(lpdcb);
//
//     sbr.BaudRate = lpdcb->BaudRate;
//
//     slc.StopBits = lpdcb->StopBits;
//     slc.Parity = lpdcb->Parity;
//     slc.WordLength = lpdcb->ByteSize;
//
//     shf.ControlHandShake = 0;
//     shf.FlowReplace = 0;
//     if (lpdcb->fOutxCtsFlow)      shf.ControlHandShake |= SERIAL_CTS_HANDSHAKE;
//     if (lpdcb->fOutxDsrFlow)      shf.ControlHandShake |= SERIAL_DSR_HANDSHAKE;
//     switch (lpdcb->fDtrControl)
//     {
//     case DTR_CONTROL_DISABLE:                                                  break;
//     case DTR_CONTROL_ENABLE:      shf.ControlHandShake |= SERIAL_DTR_CONTROL;  break;
//     case DTR_CONTROL_HANDSHAKE:   shf.ControlHandShake |= SERIAL_DTR_HANDSHAKE;break;
//     default:
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//     switch (lpdcb->fRtsControl)
//     {
//     case RTS_CONTROL_DISABLE:                                                  break;
//     case RTS_CONTROL_ENABLE:      shf.FlowReplace |= SERIAL_RTS_CONTROL;       break;
//     case RTS_CONTROL_HANDSHAKE:   shf.FlowReplace |= SERIAL_RTS_HANDSHAKE;     break;
//     case RTS_CONTROL_TOGGLE:      shf.FlowReplace |= SERIAL_RTS_CONTROL |
//                                                      SERIAL_RTS_HANDSHAKE;     break;
//     default:
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//     if (lpdcb->fDsrSensitivity)   shf.ControlHandShake |= SERIAL_DSR_SENSITIVITY;
//     if (lpdcb->fAbortOnError)     shf.ControlHandShake |= SERIAL_ERROR_ABORT;
//
//     if (lpdcb->fErrorChar)        shf.FlowReplace |= SERIAL_ERROR_CHAR;
//     if (lpdcb->fNull)             shf.FlowReplace |= SERIAL_NULL_STRIPPING;
//     if (lpdcb->fTXContinueOnXoff) shf.FlowReplace |= SERIAL_XOFF_CONTINUE;
//     if (lpdcb->fOutX)             shf.FlowReplace |= SERIAL_AUTO_TRANSMIT;
//     if (lpdcb->fInX)              shf.FlowReplace |= SERIAL_AUTO_RECEIVE;
//
//     shf.XonLimit = lpdcb->XonLim;
//     shf.XoffLimit = lpdcb->XoffLim;
//
//     sc.EofChar = lpdcb->EofChar;
//     sc.ErrorChar = lpdcb->ErrorChar;
//     sc.BreakChar = 0;
//     sc.EventChar = lpdcb->EvtChar;
//     sc.XonChar = lpdcb->XonChar;
//     sc.XoffChar = lpdcb->XoffChar;
//
//     /* note: change DTR/RTS lines after setting the comm attributes,
//      * so flow control does not interfere.
//      */
//     return (DeviceIoControl(handle, IOCTL_SERIAL_SET_BAUD_RATE,
//                             &sbr, sizeof(sbr), NULL, 0, &dwBytesReturned, NULL) &&
//             DeviceIoControl(handle, IOCTL_SERIAL_SET_LINE_CONTROL,
//                             &slc, sizeof(slc), NULL, 0, &dwBytesReturned, NULL) &&
//             DeviceIoControl(handle, IOCTL_SERIAL_SET_HANDFLOW,
//                             &shf, sizeof(shf), NULL, 0, &dwBytesReturned, NULL) &&
//             DeviceIoControl(handle, IOCTL_SERIAL_SET_CHARS,
//                             &sc, sizeof(sc), NULL, 0, &dwBytesReturned, NULL));
// }
//
//
// /*****************************************************************************
//  *	GetCommState	(KERNEL32.@)
//  *
//  *  Fills in a device control block with information from a communications device.
//  *
//  * PARAMS
//  *      handle          [in]    The communications device
//  *      lpdcb           [out]   The device control block
//  *
//  * RETURNS
//  *
//  *  True on success, false if the communication device handle is bad etc
//  *
//  * BUGS
//  *
//  *  XonChar and XoffChar are not set.
//  */
// BOOL WINAPI GetCommState(HANDLE handle, LPDCB lpdcb)
// {
//     SERIAL_BAUD_RATE    sbr;
//     SERIAL_LINE_CONTROL slc;
//     SERIAL_HANDFLOW     shf;
//     SERIAL_CHARS        sc;
//     DWORD dwBytesReturned;
//
//     TRACE!("handle %p, ptr %p\n", handle, lpdcb);
//
//     if (!lpdcb)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (!DeviceIoControl(handle, IOCTL_SERIAL_GET_BAUD_RATE,
//                          NULL, 0, &sbr, sizeof(sbr), &dwBytesReturned, NULL) ||
//         !DeviceIoControl(handle, IOCTL_SERIAL_GET_LINE_CONTROL,
//                          NULL, 0, &slc, sizeof(slc), &dwBytesReturned, NULL) ||
//         !DeviceIoControl(handle, IOCTL_SERIAL_GET_HANDFLOW,
//                          NULL, 0, &shf, sizeof(shf), &dwBytesReturned, NULL) ||
//         !DeviceIoControl(handle, IOCTL_SERIAL_GET_CHARS,
//                          NULL, 0, &sc, sizeof(sc), &dwBytesReturned, NULL))
//         return FALSE;
//
//     memset(lpdcb, 0, sizeof(*lpdcb));
//     lpdcb->DCBlength = sizeof(*lpdcb);
//
//     /* yes, they seem no never be (re)set on NT */
//     lpdcb->fBinary = 1;
//     lpdcb->fParity = 0;
//
//     lpdcb->BaudRate = sbr.BaudRate;
//
//     lpdcb->StopBits = slc.StopBits;
//     lpdcb->Parity = slc.Parity;
//     lpdcb->ByteSize = slc.WordLength;
//
//     if (shf.ControlHandShake & SERIAL_CTS_HANDSHAKE)    lpdcb->fOutxCtsFlow = 1;
//     if (shf.ControlHandShake & SERIAL_DSR_HANDSHAKE)    lpdcb->fOutxDsrFlow = 1;
//     switch (shf.ControlHandShake & (SERIAL_DTR_CONTROL | SERIAL_DTR_HANDSHAKE))
//     {
//     case 0:                     lpdcb->fDtrControl = DTR_CONTROL_DISABLE; break;
//     case SERIAL_DTR_CONTROL:    lpdcb->fDtrControl = DTR_CONTROL_ENABLE; break;
//     case SERIAL_DTR_HANDSHAKE:  lpdcb->fDtrControl = DTR_CONTROL_HANDSHAKE; break;
//     }
//     switch (shf.FlowReplace & (SERIAL_RTS_CONTROL | SERIAL_RTS_HANDSHAKE))
//     {
//     case 0:                     lpdcb->fRtsControl = RTS_CONTROL_DISABLE; break;
//     case SERIAL_RTS_CONTROL:    lpdcb->fRtsControl = RTS_CONTROL_ENABLE; break;
//     case SERIAL_RTS_HANDSHAKE:  lpdcb->fRtsControl = RTS_CONTROL_HANDSHAKE; break;
//     case SERIAL_RTS_CONTROL | SERIAL_RTS_HANDSHAKE:
//                                 lpdcb->fRtsControl = RTS_CONTROL_TOGGLE; break;
//     }
//     if (shf.ControlHandShake & SERIAL_DSR_SENSITIVITY)  lpdcb->fDsrSensitivity = 1;
//     if (shf.ControlHandShake & SERIAL_ERROR_ABORT)      lpdcb->fAbortOnError = 1;
//     if (shf.FlowReplace & SERIAL_ERROR_CHAR)            lpdcb->fErrorChar = 1;
//     if (shf.FlowReplace & SERIAL_NULL_STRIPPING)        lpdcb->fNull = 1;
//     if (shf.FlowReplace & SERIAL_XOFF_CONTINUE)         lpdcb->fTXContinueOnXoff = 1;
//     lpdcb->XonLim = shf.XonLimit;
//     lpdcb->XoffLim = shf.XoffLimit;
//
//     if (shf.FlowReplace & SERIAL_AUTO_TRANSMIT) lpdcb->fOutX = 1;
//     if (shf.FlowReplace & SERIAL_AUTO_RECEIVE)  lpdcb->fInX = 1;
//
//     lpdcb->EofChar = sc.EofChar;
//     lpdcb->ErrorChar = sc.ErrorChar;
//     lpdcb->EvtChar = sc.EventChar;
//     lpdcb->XonChar = sc.XonChar;
//     lpdcb->XoffChar = sc.XoffChar;
//
//     TRACE!("OK\n");
//     dump_dcb(lpdcb);
//
//     return TRUE;
// }
//
// /*****************************************************************************
//  *	TransmitCommChar	(KERNEL32.@)
//  *
//  *  Transmits a single character in front of any pending characters in the
//  *  output buffer.  Usually used to send an interrupt character to a host.
//  *
//  * PARAMS
//  *      hComm           [in]    The communication device in need of a command character
//  *      chTransmit      [in]    The character to transmit
//  *
//  * RETURNS
//  *
//  *  True if the call succeeded, false if the previous command character to the
//  *  same device has not been sent yet the handle is bad etc.
//  *
//  */
// BOOL WINAPI TransmitCommChar(HANDLE hComm, CHAR chTransmit)
// {
//     DWORD dwBytesReturned;
//     return DeviceIoControl(hComm, IOCTL_SERIAL_IMMEDIATE_CHAR,
//                            &chTransmit, sizeof(chTransmit), NULL, 0, &dwBytesReturned, NULL);
// }
//
//
// /*****************************************************************************
//  *	GetCommTimeouts		(KERNEL32.@)
//  *
//  *  Obtains the request timeout values for the communications device.
//  *
//  * PARAMS
//  *      hComm           [in]    The communications device
//  *      lptimeouts      [out]   The struct of request timeouts
//  *
//  * RETURNS
//  *
//  *  True on success, false if communications device handle is bad
//  *  or the target structure is null.
//  */
// BOOL WINAPI GetCommTimeouts(HANDLE hComm, LPCOMMTIMEOUTS lptimeouts)
// {
//     SERIAL_TIMEOUTS     st;
//     DWORD dwBytesReturned;
//
//     TRACE!("(%p, %p)\n", hComm, lptimeouts);
//     if (!lptimeouts)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//     if (!DeviceIoControl(hComm, IOCTL_SERIAL_GET_TIMEOUTS,
//                          NULL, 0, &st, sizeof(st), &dwBytesReturned, NULL))
//         return FALSE;
//     lptimeouts->ReadIntervalTimeout         = st.ReadIntervalTimeout;
//     lptimeouts->ReadTotalTimeoutMultiplier  = st.ReadTotalTimeoutMultiplier;
//     lptimeouts->ReadTotalTimeoutConstant    = st.ReadTotalTimeoutConstant;
//     lptimeouts->WriteTotalTimeoutMultiplier = st.WriteTotalTimeoutMultiplier;
//     lptimeouts->WriteTotalTimeoutConstant   = st.WriteTotalTimeoutConstant;
//     return TRUE;
// }
//
// /*****************************************************************************
//  *	SetCommTimeouts		(KERNEL32.@)
//  *
//  * Sets the timeouts used when reading and writing data to/from COMM ports.
//  *
//  * PARAMS
//  *      hComm           [in]    handle of COMM device
//  *      lptimeouts      [in]    pointer to COMMTIMEOUTS structure
//  *
//  * ReadIntervalTimeout
//  *     - converted and passes to linux kernel as c_cc[VTIME]
//  * ReadTotalTimeoutMultiplier, ReadTotalTimeoutConstant
//  *     - used in ReadFile to calculate GetOverlappedResult's timeout
//  * WriteTotalTimeoutMultiplier, WriteTotalTimeoutConstant
//  *     - used in WriteFile to calculate GetOverlappedResult's timeout
//  *
//  * RETURNS
//  *
//  *  True if the timeouts were set, false otherwise.
//  */
// BOOL WINAPI SetCommTimeouts(HANDLE hComm, LPCOMMTIMEOUTS lptimeouts)
// {
//     SERIAL_TIMEOUTS     st;
//     DWORD dwBytesReturned;
//
//     TRACE!("(%p, %p)\n", hComm, lptimeouts);
//
//     if (lptimeouts == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//     st.ReadIntervalTimeout         = lptimeouts->ReadIntervalTimeout;
//     st.ReadTotalTimeoutMultiplier  = lptimeouts->ReadTotalTimeoutMultiplier;
//     st.ReadTotalTimeoutConstant    = lptimeouts->ReadTotalTimeoutConstant;
//     st.WriteTotalTimeoutMultiplier = lptimeouts->WriteTotalTimeoutMultiplier;
//     st.WriteTotalTimeoutConstant   = lptimeouts->WriteTotalTimeoutConstant;
//
//     return DeviceIoControl(hComm, IOCTL_SERIAL_SET_TIMEOUTS,
//                            &st, sizeof(st), NULL, 0, &dwBytesReturned, NULL);
// }
//
// /***********************************************************************
//  *           GetCommModemStatus   (KERNEL32.@)
//  *
//  *  Obtains the four control register bits if supported by the hardware.
//  *
//  * PARAMS
//  *
//  *      hFile           [in]    The communications device
//  *      lpModemStat     [out]   The control register bits
//  *
//  * RETURNS
//  *
//  *  True if the communications handle was good and for hardware that
//  *  control register access, false otherwise.
//  */
// BOOL WINAPI GetCommModemStatus(HANDLE hFile, LPDWORD lpModemStat)
// {
//     DWORD dwBytesReturned;
//     return DeviceIoControl(hFile, IOCTL_SERIAL_GET_MODEMSTATUS,
//                            NULL, 0, lpModemStat, sizeof(DWORD), &dwBytesReturned, NULL);
// }
//
// /***********************************************************************
//  *           WaitCommEvent   (KERNEL32.@)
//  *
//  * Wait until something interesting happens on a COMM port.
//  * Interesting things (events) are set by calling SetCommMask before
//  * this function is called.
//  *
//  * RETURNS
//  *   TRUE if successful
//  *   FALSE if failure
//  *
//  *   The set of detected events will be written to *lpdwEventMask
//  *   ERROR_IO_PENDING will be returned the overlapped structure was passed
//  *
//  * BUGS:
//  *  Only supports EV_RXCHAR and EV_TXEMPTY
//  */
// BOOL WINAPI WaitCommEvent(
//     HANDLE hFile,              /* [in] handle of comm port to wait for */
//     LPDWORD lpdwEvents,        /* [out] event(s) that were detected */
//     LPOVERLAPPED lpOverlapped) /* [in/out] for Asynchronous waiting */
// {
//     return DeviceIoControl(hFile, IOCTL_SERIAL_WAIT_ON_MASK, NULL, 0,
//                            lpdwEvents, sizeof(DWORD), NULL, lpOverlapped);
// }
//
// /***********************************************************************
//  *           GetCommProperties   (KERNEL32.@)
//  *
//  * This function fills in a structure with the capabilities of the
//  * communications port driver.
//  *
//  * RETURNS
//  *
//  *  TRUE on success, FALSE on failure
//  *  If successful, the lpCommProp structure be filled in with
//  *  properties of the comm port.
//  */
// BOOL WINAPI GetCommProperties(
//     HANDLE hFile,          /* [in] handle of the comm port */
//     LPCOMMPROP lpCommProp) /* [out] pointer to struct to be filled */
// {
//     TRACE!("(%p %p)\n",hFile,lpCommProp);
//     if(!lpCommProp)
//         return FALSE;
//
//     /*
//      * These values should be valid for LINUX's serial driver
//      * FIXME: Perhaps they deserve an #ifdef LINUX
//      */
//     memset(lpCommProp,0,sizeof(COMMPROP));
//     lpCommProp->wPacketLength       = 1;
//     lpCommProp->wPacketVersion      = 1;
//     lpCommProp->dwServiceMask       = SP_SERIALCOMM;
//     lpCommProp->dwMaxTxQueue        = 4096;
//     lpCommProp->dwMaxRxQueue        = 4096;
//     lpCommProp->dwMaxBaud           = BAUD_115200;
//     lpCommProp->dwProvSubType       = PST_RS232;
//     lpCommProp->dwProvCapabilities  = PCF_DTRDSR | PCF_PARITY_CHECK | PCF_RTSCTS | PCF_TOTALTIMEOUTS;
//     lpCommProp->dwSettableParams    = SP_BAUD | SP_DATABITS | SP_HANDSHAKING |
//                                       SP_PARITY | SP_PARITY_CHECK | SP_STOPBITS ;
//     lpCommProp->dwSettableBaud      = BAUD_075 | BAUD_110 | BAUD_134_5 | BAUD_150 |
//                 BAUD_300 | BAUD_600 | BAUD_1200 | BAUD_1800 | BAUD_2400 | BAUD_4800 |
//                 BAUD_9600 | BAUD_19200 | BAUD_38400 | BAUD_57600 | BAUD_115200 ;
//     lpCommProp->wSettableData       = DATABITS_5 | DATABITS_6 | DATABITS_7 | DATABITS_8 ;
//     lpCommProp->wSettableStopParity = STOPBITS_10 | STOPBITS_15 | STOPBITS_20 |
//                 PARITY_NONE | PARITY_ODD |PARITY_EVEN | PARITY_MARK | PARITY_SPACE;
//     lpCommProp->dwCurrentTxQueue    = lpCommProp->dwMaxTxQueue;
//     lpCommProp->dwCurrentRxQueue    = lpCommProp->dwMaxRxQueue;
//
//     return TRUE;
// }
//
// /***********************************************************************
//  * FIXME:
//  * The functionality of CommConfigDialogA, GetDefaultCommConfig and
//  * SetDefaultCommConfig is implemented in a DLL (usually SERIALUI.DLL).
//  * This is dependent on the type of COMM port, but since it is doubtful
//  * anybody will get around to implementing support for fancy serial
//  * ports in WINE, this is hardcoded for the time being.  The name of
//  * this DLL should be stored in and read from the system registry in
//  * the hive HKEY_LOCAL_MACHINE, key
//  * System\\CurrentControlSet\\Services\\Class\\Ports\\????
//  * where ???? is the port number... that is determined by PNP
//  * The DLL should be loaded when the COMM port is opened, and closed
//  * when the COMM port is closed. - MJM 20 June 2000
//  ***********************************************************************/
// static const WCHAR lpszSerialUI[] = {
//    's','e','r','i','a','l','u','i','.','d','l','l',0 };
//
//
// /***********************************************************************
//  *           CommConfigDialogA   (KERNEL32.@)
//  *
//  * Raises a dialog that allows the user to configure a comm port.
//  * Fills the COMMCONFIG struct with information specified by the user.
//  * This function should call a similar routine in the COMM driver...
//  *
//  * RETURNS
//  *
//  *  TRUE on success, FALSE on failure
//  *  If successful, the lpCommConfig structure will contain a new
//  *  configuration for the comm port, as specified by the user.
//  *
//  * BUGS
//  *  The library with the CommConfigDialog code is never unloaded.
//  * Perhaps this should be done when the comm port is closed?
//  */
// BOOL WINAPI CommConfigDialogA(
//     LPCSTR lpszDevice,         /* [in] name of communications device */
//     HWND hWnd,                 /* [in] parent window for the dialog */
//     LPCOMMCONFIG lpCommConfig) /* [out] pointer to struct to fill */
// {
//     LPWSTR  lpDeviceW = NULL;
//     DWORD   len;
//     BOOL    r;
//
//     TRACE!("(%s, %p, %p)\n", debugstr_a(lpszDevice), hWnd, lpCommConfig);
//
//     if (lpszDevice)
//     {
//         len = MultiByteToWideChar( CP_ACP, 0, lpszDevice, -1, NULL, 0 );
//         lpDeviceW = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) );
//         MultiByteToWideChar( CP_ACP, 0, lpszDevice, -1, lpDeviceW, len );
//     }
//     r = CommConfigDialogW(lpDeviceW, hWnd, lpCommConfig);
//     HeapFree( GetProcessHeap(), 0, lpDeviceW );
//     return r;
// }
//
// /***********************************************************************
//  *           CommConfigDialogW   (KERNEL32.@)
//  *
//  * See CommConfigDialogA.
//  */
// BOOL WINAPI CommConfigDialogW(
//     LPCWSTR lpszDevice,        /* [in] name of communications device */
//     HWND hWnd,                 /* [in] parent window for the dialog */
//     LPCOMMCONFIG lpCommConfig) /* [out] pointer to struct to fill */
// {
//     DWORD (WINAPI *pCommConfigDialog)(LPCWSTR, HWND, LPCOMMCONFIG);
//     HMODULE hConfigModule;
//     DWORD   res = ERROR_INVALID_PARAMETER;
//
//     TRACE!("(%s, %p, %p)\n", debugstr_w(lpszDevice), hWnd, lpCommConfig);
//     hConfigModule = LoadLibraryW(lpszSerialUI);
//
//     if (hConfigModule) {
//         pCommConfigDialog = (void *)GetProcAddress(hConfigModule, "drvCommConfigDialogW");
//         if (pCommConfigDialog) {
//             res = pCommConfigDialog(lpszDevice, hWnd, lpCommConfig);
//         }
//         FreeLibrary(hConfigModule);
//     }
//
//     if (res) SetLastError(res);
//     return (res == ERROR_SUCCESS);
// }
//
// /***********************************************************************
//  *           GetCommConfig     (KERNEL32.@)
//  *
//  * Fill in the COMMCONFIG structure for the comm port hFile
//  *
//  * RETURNS
//  *
//  *  TRUE on success, FALSE on failure
//  *  If successful, lpCommConfig contains the comm port configuration.
//  *
//  * BUGS
//  *
//  */
// BOOL WINAPI GetCommConfig(
//     HANDLE       hFile,        /* [in] The communications device. */
//     LPCOMMCONFIG lpCommConfig, /* [out] The communications configuration of the device (if it fits). */
//     LPDWORD      lpdwSize)     /* [in/out] Initially the size of the configuration buffer/structure,
//                                   afterwards the number of bytes copied to the buffer or
//                                   the needed size of the buffer. */
// {
//     BOOL r;
//
//     TRACE!("(%p, %p, %p) *lpdwSize: %u\n", hFile, lpCommConfig, lpdwSize, lpdwSize ? *lpdwSize : 0 );
//
//     if(lpCommConfig == NULL)
//         return FALSE;
//     r = *lpdwSize < sizeof(COMMCONFIG); /* TRUE if not enough space */
//     *lpdwSize = sizeof(COMMCONFIG);
//     if(r)
//         return FALSE;
//
//     lpCommConfig->dwSize = sizeof(COMMCONFIG);
//     lpCommConfig->wVersion = 1;
//     lpCommConfig->wReserved = 0;
//     r = GetCommState(hFile,&lpCommConfig->dcb);
//     lpCommConfig->dwProviderSubType = PST_RS232;
//     lpCommConfig->dwProviderOffset = 0;
//     lpCommConfig->dwProviderSize = 0;
//
//     return r;
// }
//
// /***********************************************************************
//  *           SetCommConfig     (KERNEL32.@)
//  *
//  *  Sets the configuration of the communications device.
//  *
//  * RETURNS
//  *
//  *  True on success, false if the handle was bad is not a communications device.
//  */
// BOOL WINAPI SetCommConfig(
//     HANDLE       hFile,		/* [in] The communications device. */
//     LPCOMMCONFIG lpCommConfig,	/* [in] The desired configuration. */
//     DWORD dwSize) 		/* [in] size of the lpCommConfig struct */
// {
//     TRACE!("(%p, %p, %u)\n", hFile, lpCommConfig, dwSize);
//     return SetCommState(hFile,&lpCommConfig->dcb);
// }
//
// /***********************************************************************
//  *           SetDefaultCommConfigW  (KERNEL32.@)
//  *
//  * Initializes the default configuration for a communication device.
//  *
//  * PARAMS
//  *  lpszDevice   [I] Name of the device targeted for configuration
//  *  lpCommConfig [I] PTR to a buffer with the configuration for the device
//  *  dwSize       [I] Number of bytes in the buffer
//  *
//  * RETURNS
//  *  Failure: FALSE
//  *  Success: TRUE, and default configuration saved
//  *
//  */
// BOOL WINAPI SetDefaultCommConfigW(LPCWSTR lpszDevice, LPCOMMCONFIG lpCommConfig, DWORD dwSize)
// {
//     BOOL (WINAPI *lpfnSetDefaultCommConfig)(LPCWSTR, LPCOMMCONFIG, DWORD);
//     HMODULE hConfigModule;
//     BOOL r = FALSE;
//
//     TRACE!("(%s, %p, %u)\n", debugstr_w(lpszDevice), lpCommConfig, dwSize);
//
//     hConfigModule = LoadLibraryW(lpszSerialUI);
//     if(!hConfigModule)
//         return r;
//
//     lpfnSetDefaultCommConfig = (void *)GetProcAddress(hConfigModule, "drvSetDefaultCommConfigW");
//     if (lpfnSetDefaultCommConfig)
//         r = lpfnSetDefaultCommConfig(lpszDevice, lpCommConfig, dwSize);
//
//     FreeLibrary(hConfigModule);
//
//     return r;
// }
//
//
// /***********************************************************************
//  *           SetDefaultCommConfigA     (KERNEL32.@)
//  *
//  * Initializes the default configuration for a communication device.
//  *
//  * See SetDefaultCommConfigW.
//  *
//  */
// BOOL WINAPI SetDefaultCommConfigA(LPCSTR lpszDevice, LPCOMMCONFIG lpCommConfig, DWORD dwSize)
// {
//     BOOL r;
//     LPWSTR lpDeviceW = NULL;
//     DWORD len;
//
//     TRACE!("(%s, %p, %u)\n", debugstr_a(lpszDevice), lpCommConfig, dwSize);
//
//     if (lpszDevice)
//     {
//         len = MultiByteToWideChar( CP_ACP, 0, lpszDevice, -1, NULL, 0 );
//         lpDeviceW = HeapAlloc( GetProcessHeap(), 0, len*sizeof(WCHAR) );
//         MultiByteToWideChar( CP_ACP, 0, lpszDevice, -1, lpDeviceW, len );
//     }
//     r = SetDefaultCommConfigW(lpDeviceW,lpCommConfig,dwSize);
//     HeapFree( GetProcessHeap(), 0, lpDeviceW );
//     return r;
// }
//
//
// /***********************************************************************
//  *           GetDefaultCommConfigW   (KERNEL32.@)
//  *
//  *   Acquires the default configuration of the specified communication device. (unicode)
//  *
//  *  RETURNS
//  *
//  *   True on successful reading of the default configuration,
//  *   if the device is not found or the buffer is too small.
//  */
// BOOL WINAPI GetDefaultCommConfigW(
//     LPCWSTR      lpszName, /* [in] The unicode name of the device targeted for configuration. */
//     LPCOMMCONFIG lpCC,     /* [out] The default configuration for the device. */
//     LPDWORD      lpdwSize) /* [in/out] Initially the size of the default configuration buffer,
//                               afterwards the number of bytes copied to the buffer or
//                               the needed size of the buffer. */
// {
//     DWORD (WINAPI *pGetDefaultCommConfig)(LPCWSTR, LPCOMMCONFIG, LPDWORD);
//     HMODULE hConfigModule;
//     DWORD   res = ERROR_INVALID_PARAMETER;
//
//     TRACE!("(%s, %p, %p)  *lpdwSize: %u\n", debugstr_w(lpszName), lpCC, lpdwSize, lpdwSize ? *lpdwSize : 0 );
//     hConfigModule = LoadLibraryW(lpszSerialUI);
//
//     if (hConfigModule) {
//         pGetDefaultCommConfig = (void *)GetProcAddress(hConfigModule, "drvGetDefaultCommConfigW");
//         if (pGetDefaultCommConfig) {
//             res = pGetDefaultCommConfig(lpszName, lpCC, lpdwSize);
//         }
//         FreeLibrary(hConfigModule);
//     }
//
//     if (res) SetLastError(res);
//     return (res == ERROR_SUCCESS);
// }
//
// /**************************************************************************
//  *         GetDefaultCommConfigA		(KERNEL32.@)
//  *
//  *   Acquires the default configuration of the specified communication device. (ascii)
//  *
//  *  RETURNS
//  *
//  *   True on successful reading of the default configuration,
//  *   if the device is not found or the buffer is too small.
//  */
// BOOL WINAPI GetDefaultCommConfigA(
//     LPCSTR       lpszName, /* [in] The ascii name of the device targeted for configuration. */
//     LPCOMMCONFIG lpCC,     /* [out] The default configuration for the device. */
//     LPDWORD      lpdwSize) /* [in/out] Initially the size of the default configuration buffer,
// 			      afterwards the number of bytes copied to the buffer or
//                               the needed size of the buffer. */
// {
// 	BOOL ret = FALSE;
// 	UNICODE_STRING lpszNameW;
//
// 	TRACE!("(%s, %p, %p)  *lpdwSize: %u\n", debugstr_a(lpszName), lpCC, lpdwSize, lpdwSize ? *lpdwSize : 0 );
// 	if(lpszName) RtlCreateUnicodeStringFromAsciiz(&lpszNameW,lpszName);
// 	else lpszNameW.Buffer = NULL;
//
// 	ret = GetDefaultCommConfigW(lpszNameW.Buffer,lpCC,lpdwSize);
//
// 	RtlFreeUnicodeString(&lpszNameW);
// 	return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
