use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const TIMEOUT: i32 = 4000; // 4 second timeout

#[repr(C)]
pub struct INFO {
    pub Sock: SOCKET,
    pub myAddr: SOCKADDR_IN,
    pub ntpAddr: SOCKADDR_IN,
    pub SendPacket: NTPPACKET,
    pub RecvPacket: NTPPACKET,
}

pub type PINFO = *mut INFO;

unsafe extern "C" fn InitConnection(pInfo: PINFO, lpAddress: LPSTR) -> BOOL {
    let mut wsaData: WSADATA = core::mem::zeroed();
    let mut he: *mut HOSTENT = core::ptr::null_mut();
    let mut Ret: INT;

    Ret = WSAStartup(MAKEWORD!(2, 2),
                     core::ptr::addr_of_mut!(wsaData));
    if Ret != 0 {
        return FALSE as BOOL;
    }

    (*pInfo).Sock = socket(AF_INET as INT,
                         SOCK_DGRAM as INT,
                         0);
    if (*pInfo).Sock == INVALID_SOCKET {
        return FALSE as BOOL;
    }

    /* Setup server info */
    he = gethostbyname(lpAddress) as *mut HOSTENT;
    if !he.is_null() {
        /* Setup server socket info */
        ZeroMemory!(core::ptr::addr_of_mut!((*pInfo).ntpAddr) as *mut core::ffi::c_void,
                   core::mem::size_of::<SOCKADDR_IN>());
        (*pInfo).ntpAddr.sin_family = AF_INET as i16; // he->h_addrtype;
        (*pInfo).ntpAddr.sin_port = htons(NTPPORT as u_short) as USHORT;
        (*pInfo).ntpAddr.sin_addr = *(*(*he).h_addr_list.offset(0) as *mut in_addr) as IN_ADDR;
    } else {
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn DestroyConnection() {
    WSACleanup();
}

unsafe extern "C" fn GetTransmitTime(ptp: PTIMEPACKET) -> BOOL {
    TRUE as BOOL
}

unsafe extern "C" fn SendData(pInfo: PINFO) -> BOOL {
    let mut tp: TIMEPACKET = _TIMEPACKET {
        dwInteger: 0 as DWORD,
        dwFractional: 0 as DWORD,
    };
    let mut Ret: INT = 0;

    ZeroMemory!(
        core::ptr::addr_of_mut!((*pInfo).SendPacket) as *mut core::ffi::c_void,
        core::mem::size_of::<NTPPACKET>(),
    );
    (*pInfo).SendPacket.LiVnMode = 0x1b as BYTE;
    if GetTransmitTime(core::ptr::addr_of_mut!(tp)) == 0 {
        return FALSE as BOOL;
    }
    (*pInfo).SendPacket.TransmitTimestamp = tp;

    Ret = sendto(
        (*pInfo).Sock,
        core::ptr::addr_of_mut!((*pInfo).SendPacket) as *mut core::ffi::c_char,
        core::mem::size_of::<NTPPACKET>() as core::ffi::c_int,
        0 as core::ffi::c_int,
        core::ptr::addr_of_mut!((*pInfo).ntpAddr) as *mut SOCKADDR,
        core::mem::size_of::<SOCKADDR_IN>() as core::ffi::c_int,
    ) as INT;

    if Ret == SOCKET_ERROR {
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn ReceiveData(pInfo: PINFO) -> ULONG {
    let mut timeVal: TIMEVAL = core::mem::zeroed();
    let mut readFDS: FD_SET = core::mem::zeroed();
    let mut Ret: INT;
    let mut ulTime: ULONG = 0;

    /* Monitor socket for incoming connections */
    FD_ZERO!(&raw mut readFDS);
    FD_SET!((*pInfo).Sock, &raw mut readFDS);

    /* Set timeout values */
    timeVal.tv_sec = (TIMEOUT / 1000) as LONG;
    timeVal.tv_usec = (TIMEOUT % 1000) as LONG;

    /* Check for data on the socket for TIMEOUT millisecs */
    Ret = select(
        0,
        &raw mut readFDS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &raw mut timeVal,
    ) as INT;

    if (Ret != SOCKET_ERROR) && (Ret != 0) {
        Ret = recvfrom(
            (*pInfo).Sock,
            &raw mut (*pInfo).RecvPacket as *mut i8,
            core::mem::size_of::<NTPPACKET>() as INT,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as INT;

        if Ret != SOCKET_ERROR {
            ulTime = ntohl((*pInfo).RecvPacket.TransmitTimestamp.dwInteger as u_long) as ULONG;
        }
    }

    return ulTime;
}
#[no_mangle]
pub unsafe extern "C" fn GetServerTime(mut lpAddress: LPWSTR) -> ULONG {
    let mut pInfo: PINFO = core::ptr::null_mut();
    let mut lpAddr: LPSTR = core::ptr::null_mut();
    let mut dwSize: DWORD = (wcslen(lpAddress as *const u16) as DWORD) + (1 as DWORD);
    let mut ulTime: ULONG = 0 as ULONG;

    pInfo = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<INFO>() as SIZE_T,
    ) as PINFO;
    lpAddr = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        dwSize as SIZE_T,
    ) as LPSTR;

    if !pInfo.is_null() && !lpAddr.is_null() {
        if WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            lpAddress as LPCWSTR,
            -(1 as ::core::ffi::c_int),
            lpAddr,
            dwSize as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) != 0
        {
            if InitConnection(pInfo, lpAddr) != 0 {
                if SendData(pInfo) != 0 {
                    ulTime = ReceiveData(pInfo);
                }
            }

            DestroyConnection();
        }
    }

    if !pInfo.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pInfo as PVOID);
    }
    if !lpAddr.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, lpAddr as PVOID);
    }

    return ulTime;
}
