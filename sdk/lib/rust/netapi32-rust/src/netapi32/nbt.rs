use crate::prelude::*;
use core::ffi::c_int;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const TRANSPORT_NBT: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"MNBT\0") };

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netbios as *mut __wine_debug_channel };

static mut __wine_dbch_netbios: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netbios\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut gTransportID: ULONG = 0;

static mut gEnableDNS: BOOL = 0;

static mut gBCastQueries: DWORD = 0;

static mut gBCastQueryTimeout: DWORD = 0;

static mut gWINSQueries: DWORD = 0;

static mut gWINSQueryTimeout: DWORD = 0;

static mut gWINSServers: [DWORD; 2] = [0; 2];

static mut gNumWINSServers: ::core::ffi::c_int = 0;

static mut gScopeID: [::core::ffi::c_char; 256] = [0; 256];

static mut gCacheTimeout: DWORD = 0;

static mut gNameCache: *mut NBNameCache = ::core::ptr::null::<NBNameCache>() as *mut NBNameCache;

static mut VxD_MSTCPW: [WCHAR; 44] = [
    'S' as i32 as WCHAR,
    'Y' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'V' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

static mut NetBT_ParametersW: [WCHAR; 51] = [
    'S' as i32 as WCHAR,
    'Y' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'B' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

static mut EnableDNSW: [WCHAR; 10] = [
    'E' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

static mut BcastNameQueryCountW: [WCHAR; 20] = [
    'B' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'Q' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

static mut BcastNameQueryTimeoutW: [WCHAR; 22] = [
    'B' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'Q' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

static mut NameSrvQueryCountW: [WCHAR; 18] = [
    'N' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'Q' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

static mut NameSrvQueryTimeoutW: [WCHAR; 20] = [
    'N' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'Q' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

static mut ScopeIDW: [WCHAR; 8] = [
    'S' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

static mut CacheTimeoutW: [WCHAR; 13] = [
    'C' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

static mut Config_NetworkW: [WCHAR; 22] = [
    'S' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

pub const PORT_NBNS: u16 = 137;
pub const PORT_NBDG: u16 = 138;
pub const PORT_NBSS: u16 = 139;
pub const INADDR_NONE: u32 = 0xFFFFFFFFu32;
pub const MIN_QUERIES: u32 = 1;
pub const MAX_QUERIES: u32 = 0xffff;
pub const MIN_QUERY_TIMEOUT: u32 = 100;
pub const MAX_QUERY_TIMEOUT: u32 = 0xffffffff;
pub const BCAST_QUERIES: u32 = 3;
pub const BCAST_QUERY_TIMEOUT: u32 = 750;
pub const WINS_QUERIES: u32 = 3;
pub const WINS_QUERY_TIMEOUT: u32 = 750;
pub const MAX_WINS_SERVERS: usize = 2;
pub const MIN_CACHE_TIMEOUT: u32 = 60000;
pub const CACHE_TIMEOUT: u32 = 360000;
pub const MAX_NBT_NAME_SZ: u32 = 255;
pub const SIMPLE_NAME_QUERY_PKT_SIZE: u32 = 271;
pub const NBNS_TYPE_NB: u16 = 0x0020;
pub const NBNS_TYPE_NBSTAT: u16 = 0x0021;
pub const NBNS_CLASS_INTERNET: u16 = 0x0001;
pub const NBNS_HEADER_SIZE: usize = 12;
pub const NBNS_RESPONSE_AND_OPCODE: u16 = 0xf800;
pub const NBNS_RESPONSE_AND_QUERY: u16 = 0x8000;
pub const NBNS_REPLYCODE: u8 = 0x0f;
pub const NBSS_HDRSIZE: u32 = 4;
pub const NBSS_MSG: u8 = 0x00;
pub const NBSS_REQ: u8 = 0x81;
pub const NBSS_ACK: u8 = 0x82;
pub const NBSS_NACK: u8 = 0x83;
pub const NBSS_RETARGET: u8 = 0x84;
pub const NBSS_KEEPALIVE: u8 = 0x85;
pub const NBSS_ERR_NOT_LISTENING_ON_NAME: u8 = 0x80;
pub const NBSS_ERR_NOT_LISTENING_FOR_CALLER: u8 = 0x81;
pub const NBSS_ERR_BAD_NAME: u8 = 0x82;
pub const NBSS_ERR_INSUFFICIENT_RESOURCES: u8 = 0x83;
pub const NBSS_EXTENSION: u8 = 0x01;

#[repr(C)]
pub struct _NetBTSession {
    pub cs: CRITICAL_SECTION,
    pub fd: SOCKET,
    pub bytesPending: DWORD,
}
pub type NetBTSession = _NetBTSession;

#[repr(C)]
pub struct _NetBTAdapter {
    pub ipr: MIB_IPADDRROW,
    pub nameQueryXID: WORD,
    pub nameCache: *mut NBNameCache,
    pub xmit_success: DWORD,
    pub recv_success: DWORD,
}
pub type NetBTAdapter = _NetBTAdapter;

pub type NetBTAnswerCallback = Option<unsafe extern "system" fn(data: *mut core::ffi::c_void, answerCount: WORD, answerIndex: WORD, rData: PUCHAR, rdLength: WORD) -> BOOL>;

#[repr(C)]
pub struct _NetBTNameQueryData {
    pub cacheEntry: *mut NBNameCacheEntry,
    pub ret: UCHAR,
}
pub type NetBTNameQueryData = _NetBTNameQueryData;

#[repr(C)]
pub struct _NetBTNodeQueryData {
    pub gotResponse: BOOL,
    pub astat: PADAPTER_STATUS,
    pub astatLen: WORD,
}
pub type NetBTNodeQueryData = _NetBTNodeQueryData;
unsafe extern "C" fn NetBTNameEncode(p: *const u8, buffer: *mut u8) -> i32 {
    let mut len: i32 = 0;
    if p.is_null() || buffer.is_null() {
        return 0;
    }
    *buffer.add(0) = (NCBNAMSZ * 2) as u8;
    len = 1;
    let mut i: i32 = 0;
    while i < NCBNAMSZ as i32 && *p.add(i as usize) != 0 {
        let byte = *p.add(i as usize);
        *buffer.add(len as usize) = ((byte & 0xf0) >> 4) + b'A';
        len += 1;
        *buffer.add(len as usize) = (byte & 0x0f) + b'A';
        len += 1;
        i += 1;
    }
    while (len as usize) < (NCBNAMSZ * 2) as usize {
        *buffer.add(len as usize) = b'C';
        len += 1;
        *buffer.add(len as usize) = b'A';
        len += 1;
    }
    if *gScopeID.as_ptr() != 0 {
        let scope_id_ptr = gScopeID.as_ptr() as *const ::core::ffi::c_char;
        let scope_id_len = strlen(scope_id_ptr) as i32;
        memcpy(
            buffer.add(len as usize) as *mut ::core::ffi::c_void,
            scope_id_ptr as *const ::core::ffi::c_void,
            scope_id_len as usize,
        );
        len += scope_id_len;
    }
    *buffer.add(len as usize) = 0;
    len += 1;
    len
}
unsafe extern "C" fn NetBTNameReq(
    name: *const UCHAR,
    xid: WORD,
    qtype: WORD,
    broadcast: BOOL,
    buffer: *mut UCHAR,
    len: i32,
) -> DWORD {
    let mut i: i32 = 0;

    if len < 16 + MAX_NBT_NAME_SZ as i32 {
        return 0u32;
    }

    *buffer.add(i as usize).cast::<WORD>() = htons(xid);
    i += 2;

    if broadcast != 0 {
        *buffer.add(i as usize).cast::<WORD>() = htons(0x0110u16);
        i += 2;
    } else {
        *buffer.add(i as usize).cast::<WORD>() = htons(0x0100u16);
        i += 2;
    }

    *buffer.add(i as usize).cast::<WORD>() = htons(0x0001u16);
    i += 2;
    *buffer.add(i as usize).cast::<WORD>() = htons(0x0000u16);
    i += 2;
    *buffer.add(i as usize).cast::<WORD>() = htons(0x0000u16);
    i += 2;
    *buffer.add(i as usize).cast::<WORD>() = htons(0x0000u16);
    i += 2;

    i += NetBTNameEncode(name, buffer.add(i as usize));

    *buffer.add(i as usize).cast::<WORD>() = htons(qtype);
    i += 2;
    *buffer.add(i as usize).cast::<WORD>() = htons(NBNS_CLASS_INTERNET as u16);
    i += 2;

    i as u32
}
unsafe extern "C" fn NetBTSendNameQuery(
    fd: SOCKET,
    name: *const UCHAR,
    xid: WORD,
    qtype: WORD,
    destAddr: DWORD,
    broadcast: BOOL,
) -> i32 {
    let mut ret: i32 = 0;
    let on: i32 = 1;
    let mut addr: in_addr = core::mem::zeroed();
    addr.S_un.S_addr = destAddr;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            dbcl,
            dbch,
            b"dll/win32/netapi32/nbt.c\0"
                .as_ptr() as *const i8,
            b"NetBTSendNameQuery\0".as_ptr() as *const i8,
            234i32,
            b"name %s, dest addr %s\n\0".as_ptr() as *const i8,
            name,
            inet_ntoa(addr),
        );
    }

    if broadcast != 0 {
        ret = setsockopt(
            fd,
            SOL_SOCKET as c_int,
            SO_BROADCAST as c_int,
            core::ptr::addr_of!(on) as *const i8,
            core::mem::size_of::<i32>() as i32,
        );
    }

    if ret == 0 {
        let mut wsaBuf: WSABUF = WSABUF {
            len: 0,
            buf: core::ptr::null_mut(),
        };
        let mut buf: [UCHAR; SIMPLE_NAME_QUERY_PKT_SIZE as usize] =
            [0; SIMPLE_NAME_QUERY_PKT_SIZE as usize];
        let mut sin: sockaddr_in = core::mem::zeroed();
        memset(
            core::ptr::addr_of_mut!(sin) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<sockaddr_in>(),
        );
        sin.sin_addr.S_un.S_addr = destAddr;
        sin.sin_family = AF_INET as i16;
        sin.sin_port = htons(PORT_NBNS as u16) as u16;

        wsaBuf.buf = buf.as_mut_ptr() as *mut CHAR;
        wsaBuf.len = NetBTNameReq(
            name,
            xid,
            qtype,
            broadcast,
            buf.as_mut_ptr(),
            core::mem::size_of::<[UCHAR; SIMPLE_NAME_QUERY_PKT_SIZE as usize]>() as i32,
        ) as u32;

        if wsaBuf.len > 0u32 {
            let mut bytesSent: DWORD = 0;
            ret = WSASendTo(
                fd,
                core::ptr::addr_of_mut!(wsaBuf) as *mut c_void,
                1u32,
                core::ptr::addr_of_mut!(bytesSent),
                0u32,
                core::ptr::addr_of_mut!(sin) as *const c_void,
                core::mem::size_of::<sockaddr_in>() as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if ret < 0 || bytesSent < wsaBuf.len {
                ret = -1i32;
            } else {
                ret = 0i32;
            }
        } else {
            ret = -1i32;
        }
    }

    ret
}
unsafe extern "C" fn NetBTWaitForNameResponse(
    adapter: *const NetBTAdapter,
    fd: SOCKET,
    waitUntil: DWORD,
    answerCallback: NetBTAnswerCallback,
    data: *mut core::ffi::c_void,
) -> UCHAR {

    let mut found: BOOL = FALSE;
    let mut now: DWORD;
    let mut ret: UCHAR = NRC_GOODRET;

    if adapter.is_null() || fd == INVALID_SOCKET || answerCallback.is_none() {
        return NRC_BADDR;
    }

    while found == 0 && ret == NRC_GOODRET && {
        now = GetTickCount();
        (now.wrapping_sub(waitUntil) as c_int) < 0
    } {
        let msToWait = waitUntil - now;
        let mut fds: fd_set = core::mem::zeroed();
        let mut timeout = timeval {
            tv_sec: (msToWait / 1000) as c_int,
            tv_usec: ((msToWait % 1000) * 1000) as c_int, // original was msToWait % 1000 as microseconds
        };

        // we assume FD_ZERO/FD_SET as external functions
        FD_ZERO(&raw mut fds);
        FD_SET(fd, &raw mut fds);

        let r = select(
            (fd as c_int) + 1,
            &raw mut fds,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut timeout as *const c_void,
        );

        if r < 0 {
            ret = NRC_SYSTEM;
        } else if r == 1 {
            let mut buffer: [UCHAR; 256] = [0; 256];
            let buf_ptr = buffer.as_mut_ptr();
            let mut fromaddr: sockaddr_in = core::mem::zeroed();
            let mut fromsize = core::mem::size_of::<sockaddr_in>() as c_int;

            let mut wsaBuf = WSABUF {
                len: core::mem::size_of::<[UCHAR; 256]>() as ULONG,
                buf: buf_ptr as *mut CHAR,
            };
            let mut bytesReceived: DWORD = 0;
            let mut recvFlags: DWORD = 0;

            let r2 = WSARecvFrom(
                fd,
                &raw mut wsaBuf as *mut c_void,
                1,
                &raw mut bytesReceived,
                &raw mut recvFlags,
                &raw mut fromaddr as *mut c_void,
                &raw mut fromsize,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if r2 < 0 {
                ret = NRC_SYSTEM;
                break;
            }

            if (bytesReceived as usize) < NBNS_HEADER_SIZE {
                continue;
            }

            let respXID = NBR_GETWORD(buf_ptr as *const _);
            if respXID != (*adapter).nameQueryXID {
                continue;
            }

            let flags = NBR_GETWORD(buf_ptr.add(2) as *const _);
            let queryCount = NBR_GETWORD(buf_ptr.add(4) as *const _);
            let answerCount = NBR_GETWORD(buf_ptr.add(6) as *const _);

            if queryCount > 0 {
                continue;
            }

            if (flags & NBNS_RESPONSE_AND_OPCODE) == NBNS_RESPONSE_AND_QUERY {
                if (flags & NBNS_REPLYCODE as u16) != 0 {
                    ret = NRC_NAMERR;
                } else if answerCount > 0 {
                    let mut ptr = buf_ptr.offset(NBNS_HEADER_SIZE as isize);
                    let mut shouldContinue = TRUE;
                    let mut answerIndex: WORD = 0;

                    found = TRUE;
                    while ret == NRC_GOODRET
                        && answerIndex < answerCount
                        && (ptr.offset_from(buf_ptr) as usize) < bytesReceived as usize
                        && shouldContinue != 0
                    {
                        // scan past name
                        while *ptr != 0
                            && (ptr.offset_from(buf_ptr) as usize) < bytesReceived as usize
                        {
                            ptr = ptr.offset((*ptr + 1) as isize);
                        }
                        ptr = ptr.offset(1); // null terminator
                        ptr = ptr.offset(2); // skip type

                        if (ptr.offset_from(buf_ptr) as usize) < bytesReceived as usize
                            && ret == NRC_GOODRET
                            && NBR_GETWORD(ptr as *const _) as i32 == NBNS_CLASS_INTERNET as i32
                        {
                            ptr = ptr.offset(core::mem::size_of::<WORD>() as isize);
                        } else {
                            ret = NRC_SYSTEM;
                        }

                        ptr = ptr.offset(core::mem::size_of::<DWORD>() as isize); // TTL

                        let mut rLen = NBR_GETWORD(ptr as *const _);
                        let remaining = (bytesReceived as usize)
                            .wrapping_sub(ptr.offset_from(buf_ptr) as usize);
                        rLen = min(rLen as c_ulonglong, remaining as c_ulonglong) as WORD;
                        ptr = ptr.offset(core::mem::size_of::<WORD>() as isize);

                        if let Some(cb) = answerCallback {
                            shouldContinue = cb(data, answerCount, answerIndex, ptr, rLen);
                        }
                        ptr = ptr.offset(rLen as isize);
                        answerIndex += 1;
                    }
                }
            }
        }
    }

    // debug trace as in the anchor
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0"
                as *const u8
                as *const core::ffi::c_char,
            b"NetBTWaitForNameResponse\0" as *const u8 as *const core::ffi::c_char,
            378,
            b"returning 0x%02x\n\0" as *const u8 as *const core::ffi::c_char,
            ret as i32,
        );
    }

    ret
}
unsafe extern "system" fn NetBTFindNameAnswerCallback(
    pVoid: *mut core::ffi::c_void,
    answerCount: WORD,
    _answerIndex: WORD,
    rData: PUCHAR,
    rLen: WORD,
) -> BOOL {
    let queryData = pVoid as *mut NetBTNameQueryData;
    let mut ret: BOOL = 0;
    if !queryData.is_null() {
        if (*queryData).cacheEntry.is_null() {
            (*queryData).cacheEntry = HeapAlloc(
                GetProcessHeap(),
                0,
                // FIELD_OFFSET(NBNameCacheEntry, addresses[answerCount]) — offset of the array
                // plus answerCount elements (addresses is DWORD[1], element size = sizeof(DWORD)).
                (core::mem::offset_of!(NBNameCacheEntry, addresses)
                    + (answerCount as usize) * core::mem::size_of::<DWORD>())
                    as SIZE_T,
            ) as *mut NBNameCacheEntry;
            if !(*queryData).cacheEntry.is_null() {
                (*(*queryData).cacheEntry).numAddresses = 0;
            } else {
                (*queryData).ret = NRC_OSRESNOTAV as UCHAR;
            }
        }
        if rLen == 6
            && !(*queryData).cacheEntry.is_null()
            && (*(*queryData).cacheEntry).numAddresses < answerCount as u32
        {
            let entry = &mut *(*queryData).cacheEntry;
            let idx = entry.numAddresses;
            entry.numAddresses += 1;
            let dest = core::ptr::addr_of_mut!(entry.addresses) as *mut DWORD;
            let src = (rData.add(2usize)) as *const DWORD;
            *dest.add(idx as usize) = *src;
            ret = (entry.numAddresses < answerCount as u32) as BOOL;
        } else {
            ret = 0;
        }
    } else {
        ret = 0;
    }
    ret
}
unsafe extern "C" fn NetBTNameWaitLoop(
    adapter: *const NetBTAdapter,
    fd: SOCKET,
    ncb: *const NCB,
    sendTo: DWORD,
    broadcast: BOOL,
    timeout: DWORD,
    maxQueries: DWORD,
    cacheEntry: *mut *mut NBNameCacheEntry,
) -> UCHAR {
    if adapter.is_null() {
        return NRC_BADDR as UCHAR;
    }
    if fd == INVALID_SOCKET {
        return NRC_BADDR as UCHAR;
    }
    if ncb.is_null() {
        return NRC_BADDR as UCHAR;
    }
    if cacheEntry.is_null() {
        return NRC_BADDR as UCHAR;
    }

    let mut queries: DWORD = 0;
    let mut queryData = NetBTNameQueryData {
        cacheEntry: core::ptr::null_mut::<NBNameCacheEntry>(),
        ret: NRC_GOODRET as UCHAR,
    };

    while queryData.cacheEntry.is_null() && queries < maxQueries {
        if *(core::ptr::addr_of!((*ncb).ncb_reserve) as *const BOOL) == 0 {
            let r = NetBTSendNameQuery(
                fd,
                core::ptr::addr_of!((*ncb).ncb_callname) as *const UCHAR,
                (*adapter).nameQueryXID,
                NBNS_TYPE_NB as WORD,
                sendTo,
                broadcast,
            );
            if r == 0 {
                queryData.ret = NetBTWaitForNameResponse(
                    adapter,
                    fd,
                    GetTickCount().wrapping_add(timeout),
                    Some(NetBTFindNameAnswerCallback),
                    &raw mut queryData as *mut core::ffi::c_void,
                );
            } else {
                queryData.ret = NRC_SYSTEM as UCHAR;
            }
        } else {
            queryData.ret = NRC_CMDCAN as UCHAR;
        }
        queries = queries.wrapping_add(1);
    }

    if !queryData.cacheEntry.is_null() {
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*ncb).ncb_callname) as *const u8,
            core::ptr::addr_of_mut!((*queryData.cacheEntry).name) as *mut u8,
            NCBNAMSZ as usize,
        );
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*ncb).ncb_callname) as *const u8,
            core::ptr::addr_of_mut!((*queryData.cacheEntry).nbname) as *mut u8,
            NCBNAMSZ as usize,
        );
    }

    *cacheEntry = queryData.cacheEntry;
    queryData.ret
}
unsafe extern "C" fn NetBTStoreCacheEntry(
    nameCache: *mut *mut NBNameCache,
    cacheEntry: *mut NBNameCacheEntry,
) -> UCHAR {
    let mut ret: UCHAR = 0;

    if nameCache.is_null() {
        return NRC_BADDR as UCHAR;
    }
    if cacheEntry.is_null() {
        return NRC_BADDR as UCHAR;
    }

    if (*nameCache).is_null() {
        *nameCache = NBNameCacheCreate(GetProcessHeap(), gCacheTimeout) as *mut NBNameCache;
    }

    if !(*nameCache).is_null() {
        ret = if NBNameCacheAddEntry(*nameCache as *mut crate::netapi32::nbnamecache::NBNameCache, cacheEntry) != 0 {
            NRC_GOODRET as UCHAR
        } else {
            NRC_OSRESNOTAV as UCHAR
        };
    } else {
        HeapFree(GetProcessHeap(), 0u32, cacheEntry as *mut core::ffi::c_void);
        ret = NRC_OSRESNOTAV as UCHAR;
    }

    ret
}
unsafe extern "C" fn NetBTinetResolve(name: *const UCHAR, cacheEntry: *mut *mut NBNameCacheEntry) -> UCHAR {
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & (1 << (__WINE_DBCL_TRACE as core::ffi::c_int)))
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0".as_ptr() as *const core::ffi::c_char,
            b"NetBTinetResolve\0".as_ptr() as *const core::ffi::c_char,
            514 as core::ffi::c_int,
            b"name %s, cacheEntry %p\n\0".as_ptr() as *const core::ffi::c_char,
            name as *const core::ffi::c_char,
            cacheEntry as *mut core::ffi::c_void,
        );
    }

    let mut ret: UCHAR = NRC_GOODRET;

    if name.is_null() {
        return NRC_BADDR;
    }
    if cacheEntry.is_null() {
        return NRC_BADDR;
    }

    if isalnum(*name.offset(0) as core::ffi::c_int) != 0
        && (*name.offset((NCBNAMSZ - 1) as isize) as core::ffi::c_int == 0
            || *name.offset((NCBNAMSZ - 1) as isize) as core::ffi::c_int == 0x20)
    {
        let mut toLookup: [i8; NCBNAMSZ as usize] = [0; NCBNAMSZ as usize];
        let mut i: core::ffi::c_uint = 0;

        while (i as usize) < (NCBNAMSZ as usize - 1)
            && *name.offset(i as isize) as core::ffi::c_int != 0
            && *name.offset(i as isize) as core::ffi::c_int != ' ' as i32
        {
            toLookup[i as usize] = *name.offset(i as isize) as i8;
            i = i.wrapping_add(1);
        }
        toLookup[i as usize] = b'\0' as i8;

        if isdigit(toLookup[0] as core::ffi::c_int) != 0 {
            let addr: core::ffi::c_ulong = inet_addr(toLookup.as_mut_ptr());
            if addr != INADDR_NONE as core::ffi::c_ulong {
                let alloc_size = (core::mem::offset_of!(NBNameCacheEntry, addresses)
                    + 1 * core::mem::size_of::<DWORD>()) as usize;
                let allocated = HeapAlloc(
                    GetProcessHeap(),
                    0u32,
                    alloc_size as SIZE_T,
                ) as *mut NBNameCacheEntry;
                *cacheEntry = allocated;
                if !(*cacheEntry).is_null() {
                    memcpy(
                        core::ptr::addr_of_mut!((**cacheEntry).name) as *mut core::ffi::c_void,
                        name as *const core::ffi::c_void,
                        NCBNAMSZ as usize,
                    );
                    memset(
                        core::ptr::addr_of_mut!((**cacheEntry).nbname) as *mut core::ffi::c_void,
                        0i32,
                        NCBNAMSZ as usize,
                    );
                    (**cacheEntry).nbname[0] = b'*';
                    (**cacheEntry).numAddresses = 1;
                    *(core::ptr::addr_of_mut!((**cacheEntry).addresses) as *mut DWORD).offset(0) = addr as DWORD;
                } else {
                    ret = NRC_OSRESNOTAV;
                }
            }
        }

        if gEnableDNS != 0 && ret == NRC_GOODRET && (*cacheEntry).is_null() {
            let host: *mut hostent = gethostbyname(toLookup.as_mut_ptr());
            if !host.is_null() {
                i = 0;
                while ret as core::ffi::c_int == NRC_GOODRET as core::ffi::c_int
                    && !(*host).h_addr_list.is_null()
                    && !(*(*host).h_addr_list.offset(i as isize)).is_null()
                {
                    i = i.wrapping_add(1);
                }
                if !(*host).h_addr_list.is_null()
                    && !(*(*host).h_addr_list.offset(0)).is_null()
                {
                    let alloc_size = (core::mem::offset_of!(NBNameCacheEntry, addresses)
                        + (i as usize) * core::mem::size_of::<DWORD>()) as usize;
                    let allocated = HeapAlloc(
                        GetProcessHeap(),
                        0u32,
                        alloc_size as SIZE_T,
                    ) as *mut NBNameCacheEntry;
                    *cacheEntry = allocated;
                    if !(*cacheEntry).is_null() {
                        memcpy(
                            core::ptr::addr_of_mut!((**cacheEntry).name) as *mut core::ffi::c_void,
                            name as *const core::ffi::c_void,
                            NCBNAMSZ as usize,
                        );
                        memset(
                            core::ptr::addr_of_mut!((**cacheEntry).nbname) as *mut core::ffi::c_void,
                            0i32,
                            NCBNAMSZ as usize,
                        );
                        (**cacheEntry).nbname[0] = b'*';
                        (**cacheEntry).numAddresses = i as DWORD;
                        let mut j: core::ffi::c_uint = 0;
                        while (j as usize) < (**cacheEntry).numAddresses as usize {
                            *(core::ptr::addr_of_mut!((**cacheEntry).addresses) as *mut DWORD).offset(j as isize) =
                                *(*(*host).h_addr_list.offset(j as isize) as *mut DWORD);
                            j = j.wrapping_add(1);
                        }
                    } else {
                        ret = NRC_OSRESNOTAV;
                    }
                }
            }
        }
    }

    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & (1 << (__WINE_DBCL_TRACE as core::ffi::c_int)))
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0".as_ptr() as *const core::ffi::c_char,
            b"NetBTinetResolve\0".as_ptr() as *const core::ffi::c_char,
            579 as core::ffi::c_int,
            b"returning 0x%02x\n\0".as_ptr() as *const core::ffi::c_char,
            ret as core::ffi::c_int,
        );
    }

    ret
}
unsafe extern "C" fn NetBTInternalFindName(
    adapter: *mut NetBTAdapter,
    ncb: PNCB,
    cacheEntry: *mut *const NBNameCacheEntry,
) -> UCHAR {
    let mut ret: UCHAR = NRC_GOODRET as UCHAR;
    if cacheEntry.is_null() {
        return NRC_BADDR as UCHAR;
    }
    *cacheEntry = core::ptr::null();
    if adapter.is_null() || ncb.is_null() {
        return NRC_BADDR as UCHAR;
    }
    if (*ncb).ncb_callname[0] as i32 == b'*' as i32 {
        ret = NRC_NOWILD as UCHAR;
    } else {
        *cacheEntry = NBNameCacheFindEntry(
            gNameCache as *mut crate::netapi32::nbnamecache::NBNameCache,
            core::ptr::addr_of!((*ncb).ncb_callname) as *const UCHAR,
        );
        if (*cacheEntry).is_null() {
            *cacheEntry = NBNameCacheFindEntry(
                (*adapter).nameCache as *mut crate::netapi32::nbnamecache::NBNameCache,
                core::ptr::addr_of!((*ncb).ncb_callname) as *const UCHAR,
            );
        }
        if (*cacheEntry).is_null() {
            let mut newEntry: *mut NBNameCacheEntry = core::ptr::null_mut();
            ret = NetBTinetResolve(
                core::ptr::addr_of!((*ncb).ncb_callname) as *const UCHAR,
                &raw mut newEntry,
            );
            if ret == NRC_GOODRET as UCHAR && !newEntry.is_null() {
                ret = NetBTStoreCacheEntry(&raw mut gNameCache, newEntry);
                if ret != NRC_GOODRET as UCHAR {
                    newEntry = core::ptr::null_mut();
                }
            } else {
                let fd: SOCKET = WSASocketA(
                    PF_INET as c_int,
                    SOCK_DGRAM as c_int,
                    IPPROTO_UDP as i32,
                    core::ptr::null_mut(),
                    0,
                    WSA_FLAG_OVERLAPPED as DWORD,
                );
                if fd == INVALID_SOCKET {
                    ret = NRC_OSRESNOTAV as UCHAR;
                } else {
                    (*adapter).nameQueryXID = (*adapter).nameQueryXID.wrapping_add(1);
                    let mut winsNdx: i32 = 0;
                    while ret == NRC_GOODRET as UCHAR
                        && (*cacheEntry).is_null()
                        && winsNdx < gNumWINSServers
                    {
                        ret = NetBTNameWaitLoop(
                            adapter,
                            fd,
                            ncb as *const NCB,
                            gWINSServers[winsNdx as usize],
                            FALSE,
                            gWINSQueryTimeout,
                            gWINSQueries,
                            &raw mut newEntry,
                        );
                        winsNdx += 1;
                    }
                    if ret == NRC_GOODRET as UCHAR && !newEntry.is_null() {
                        ret = NetBTStoreCacheEntry(&raw mut gNameCache, newEntry);
                        if ret != NRC_GOODRET as UCHAR {
                            newEntry = core::ptr::null_mut();
                        }
                    }
                    if ret == NRC_GOODRET as UCHAR && (*cacheEntry).is_null() {
                        let mut bcastAddr: DWORD =
                            (*adapter).ipr.dwAddr & (*adapter).ipr.dwMask;
                        if (*adapter).ipr.dwBCastAddr != 0 {
                            bcastAddr |= !(*adapter).ipr.dwMask;
                        }
                        ret = NetBTNameWaitLoop(
                            adapter,
                            fd,
                            ncb as *const NCB,
                            bcastAddr,
                            TRUE,
                            gBCastQueryTimeout,
                            gBCastQueries,
                            &raw mut newEntry,
                        );
                        if ret == NRC_GOODRET as UCHAR && !newEntry.is_null() {
                            ret = NetBTStoreCacheEntry(
                                &raw mut (*adapter).nameCache,
                                newEntry,
                            );
                            if ret != NRC_GOODRET as UCHAR {
                                newEntry = core::ptr::null_mut();
                            }
                        }
                    }
                    closesocket(fd);
                }
            }
            *cacheEntry = newEntry;
        }
    }
    ret
}
unsafe extern "system" fn NetBTNodeStatusAnswerCallback(
    pVoid: *mut core::ffi::c_void,
    _answerCount: u16,
    _answerIndex: u16,
    rData: *mut u8,
    rLen: u16,
) -> i32 {
    let data: *mut NetBTNodeQueryData = pVoid as *mut NetBTNodeQueryData;
    if !data.is_null()
        && (*data).gotResponse == 0
        && !rData.is_null()
        && rLen as i32 >= 1i32
    {
        if rLen as i32 >= (*rData as i32) * (NCBNAMSZ as i32 + 2i32) {
            let mut i: u16 = 0u16;
            let mut src: *mut u8 = rData.add(1usize);
            let mut dst: *mut NAME_BUFFER =
                ((*data).astat as *mut u8).add(core::mem::size_of::<ADAPTER_STATUS>()) as *mut NAME_BUFFER;

            (*data).gotResponse = 1;
            (*(*data).astat).name_count = *rData as u16;

            while (i as i32) < (*(*data).astat).name_count as i32
                && (src.offset_from(rData) as isize) < (rLen as isize)
                && ((dst as *mut u8).offset_from((*data).astat as *mut u8) as isize)
                    < ((*data).astatLen as isize)
            {
                let flags: u8 = *src.add(NCBNAMSZ as usize);
                core::ptr::copy_nonoverlapping(
                    src,
                    &raw mut (*dst).name as *mut u8,
                    NCBNAMSZ as usize,
                );
                (*dst).name_flags = REGISTERING as u8;
                if flags & 0x80u8 != 0u8 {
                    (*dst).name_flags |= GROUP_NAME as u8;
                }
                if flags & 0x10u8 != 0u8 {
                    (*dst).name_flags |= DEREGISTERED as u8;
                }
                if flags & 0x08u8 != 0u8 {
                    (*dst).name_flags |= DUPLICATE as u8;
                }
                if (*dst).name_flags as i32 == REGISTERING as i32 {
                    (*dst).name_flags = REGISTERED as u8;
                }

                i = i.wrapping_add(1);
                dst = dst.add(1usize);
                src = src.add(NCBNAMSZ as usize + 2);
            }

            (*(*data).astat).adapter_type = 0xfeu8;

            if (src.offset_from(rData) as isize) < rLen as isize {
                let remaining = (rLen as isize) - (src.offset_from(rData) as isize);
                let bytes = min(
                    if remaining < 6isize { remaining } else { 6isize } as usize,
                    6usize,
                );
                core::ptr::copy_nonoverlapping(
                    src,
                    &raw mut (*(*data).astat).adapter_address as *mut u8,
                    bytes,
                );
            }
        }
    }
    0i32
}
unsafe extern "C" fn NetBTAstatRemote(adapter: *mut _NetBTAdapter, ncb: PNCB) -> u8 {
    let mut ret: u8 = NRC_GOODRET as u8;
    let mut cacheEntry: *const _NBNameCacheEntry = core::ptr::null();

    if adapter.is_null() {
        return NRC_BADDR as u8;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as u8;
    }

    ret = NetBTInternalFindName(adapter, ncb, &raw mut cacheEntry);
    if ret == (NRC_GOODRET as u8) && !cacheEntry.is_null() {
        if (*cacheEntry).numAddresses > 0 {
            let fd = WSASocketA(
                2,                // PF_INET
                2,                // SOCK_DGRAM
                17,               // IPPROTO_UDP
                core::ptr::null_mut(),
                0,
                0x1,              // WSA_FLAG_OVERLAPPED
            );
            if fd == INVALID_SOCKET {
                ret = NRC_OSRESNOTAV as u8;
            } else {
                let mut queryData: NetBTNodeQueryData = NetBTNodeQueryData {
                    gotResponse: 0, // FALSE
                    astat: core::ptr::null_mut(),
                    astatLen: 0,
                };
                let astat = (*ncb).ncb_buffer as *mut _ADAPTER_STATUS;

                (*adapter).nameQueryXID = (*adapter).nameQueryXID.wrapping_add(1);
                (*astat).name_count = 0;
                queryData.gotResponse = 0; // FALSE
                queryData.astat = astat;
                queryData.astatLen = (*ncb).ncb_length;

                let mut queries: u32 = 0;
                while queryData.gotResponse == 0 && queries < gWINSQueries {
                    let cancelled = *(core::ptr::addr_of_mut!((*ncb).ncb_reserve) as *const BOOL) != 0;
                    if !cancelled {
                        let r = NetBTSendNameQuery(
                            fd,
                            (*ncb).ncb_callname.as_ptr(),
                            (*adapter).nameQueryXID,
                            NBNS_TYPE_NBSTAT as u16,
                            (*cacheEntry).addresses[0],
                            0, // FALSE
                        );
                        if r == 0 {
                            ret = NetBTWaitForNameResponse(
                                adapter,
                                fd,
                                GetTickCount().wrapping_add(gWINSQueryTimeout),
                                Some(NetBTNodeStatusAnswerCallback),
                                &raw mut queryData as *mut core::ffi::c_void,
                            );
                        } else {
                            ret = NRC_SYSTEM as u8;
                        }
                    } else {
                        ret = NRC_CMDCAN as u8;
                    }
                    queries = queries.wrapping_add(1);
                }
                closesocket(fd);
            }
        } else {
            ret = NRC_CMDTMO as u8;
        }
    } else if ret != (NRC_CMDCAN as u8) {
        ret = NRC_CMDTMO as u8;
    }

    ret
}
unsafe extern "system" fn NetBTAstat(adapt: *mut core::ffi::c_void, ncb: PNCB) -> UCHAR {
    let adapter: *mut NetBTAdapter = adapt as *mut NetBTAdapter;
    let mut ret: UCHAR = 0;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/nbt.c\0"
                .as_ptr() as *const i8,
            b"NetBTAstat\0".as_ptr() as *const i8,
            807i32,
            b"adapt %p, NCB %p\n\0".as_ptr() as *const i8,
            adapt,
            ncb,
        );
    }

    if adapter.is_null() {
        return NRC_ENVNOTDEF as UCHAR;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as UCHAR;
    }
    if (*ncb).ncb_buffer.is_null() {
        return NRC_BADDR as UCHAR;
    }
    if ((*ncb).ncb_length as usize) < core::mem::size_of::<ADAPTER_STATUS>() {
        return NRC_BUFLEN as UCHAR;
    }

    if (*ncb).ncb_callname[0] as i32 == '*' as i32 {
        let mut physAddrLen: DWORD = 0;
        let mut ifRow: MIB_IFROW = MIB_IFROW {
            wszName: [0u16; 256],
            dwIndex: 0,
            dwType: 0,
            dwMtu: 0,
            dwSpeed: 0,
            dwPhysAddrLen: 0,
            bPhysAddr: [0u8; 8],
            dwAdminStatus: 0,
            dwOperStatus: IF_OPER_STATUS_NON_OPERATIONAL,
            dwLastChange: 0,
            dwInOctets: 0,
            dwInUcastPkts: 0,
            dwInNUcastPkts: 0,
            dwInDiscards: 0,
            dwInErrors: 0,
            dwInUnknownProtos: 0,
            dwOutOctets: 0,
            dwOutUcastPkts: 0,
            dwOutNUcastPkts: 0,
            dwOutDiscards: 0,
            dwOutErrors: 0,
            dwOutQLen: 0,
            dwDescrLen: 0,
            bDescr: [0u8; 256],
        };
        let astat: PADAPTER_STATUS = (*ncb).ncb_buffer as PADAPTER_STATUS;
        memset(
            astat as *mut core::ffi::c_void,
            0i32,
            core::mem::size_of::<ADAPTER_STATUS>() as usize,
        );
        (*astat).rev_major = 3u8;
        ifRow.dwIndex = (*adapter).ipr.dwIndex as DWORD;
        if GetIfEntry(&raw mut ifRow) != NO_ERROR as u32 {
            ret = NRC_BRIDGE as UCHAR;
        } else {
            physAddrLen = if ifRow.dwPhysAddrLen < 6u32 {
                ifRow.dwPhysAddrLen
            } else {
                6u32
            };
            if physAddrLen > 0u32 {
                memcpy(
                    core::ptr::addr_of_mut!((*astat).adapter_address) as *mut core::ffi::c_void,
                    core::ptr::addr_of!(ifRow.bPhysAddr) as *const core::ffi::c_void,
                    physAddrLen as usize,
                );
            }
            if ifRow.dwType == MIB_IF_TYPE_TOKENRING as u32 {
                (*astat).adapter_type = 0xffu8;
            } else {
                (*astat).adapter_type = 0xfeu8;
            }
            (*astat).max_sess_pkt_size = 0xffffu16;
            (*astat).xmit_success = (*adapter).xmit_success;
            (*astat).recv_success = (*adapter).recv_success;
            ret = NRC_GOODRET as UCHAR;
        }
    } else {
        ret = NetBTAstatRemote(adapter, ncb);
    }

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/netapi32/nbt.c\0"
                .as_ptr() as *const i8,
            b"NetBTAstat\0".as_ptr() as *const i8,
            843i32,
            b"returning 0x%02x\n\0".as_ptr() as *const i8,
            ret as i32,
        );
    }

    ret
}
unsafe extern "system" fn NetBTFindName(adapt: *mut core::ffi::c_void, ncb: PNCB) -> UCHAR {
    let adapter: *mut NetBTAdapter = adapt as *mut NetBTAdapter;
    let mut ret: UCHAR = 0;
    let mut cacheEntry: *const NBNameCacheEntry = core::ptr::null();
    let mut foundName: PFIND_NAME_HEADER = core::ptr::null_mut();

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"NetBTFindName\0".as_ptr() as *const ::core::ffi::c_char,
            854,
            b"adapt %p, NCB %p\n\0".as_ptr() as *const ::core::ffi::c_char,
            adapt,
            ncb,
        );
    }

    if adapter.is_null() {
        return NRC_ENVNOTDEF as UCHAR;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as UCHAR;
    }
    if (*ncb).ncb_buffer.is_null() {
        return NRC_BADDR as UCHAR;
    }
    if ((*ncb).ncb_length as usize) < core::mem::size_of::<FIND_NAME_HEADER>() {
        return NRC_BUFLEN as UCHAR;
    }

    foundName = (*ncb).ncb_buffer as PFIND_NAME_HEADER;
    core::ptr::write_bytes(
        foundName as *mut u8,
        0,
        core::mem::size_of::<FIND_NAME_HEADER>(),
    );

    ret = NetBTInternalFindName(adapter, ncb, &raw mut cacheEntry);

    if ret as i32 == NRC_GOODRET as i32 {
        if !cacheEntry.is_null() {
            let spaceFor: DWORD = min(
                ((*ncb).ncb_length as usize - core::mem::size_of::<FIND_NAME_HEADER>())
                    / core::mem::size_of::<FIND_NAME_BUFFER>(),
                (*cacheEntry).numAddresses as usize,
            ) as DWORD;
            let mut ndx: DWORD = 0;
            while ndx < spaceFor {
                let findNameBuffer: PFIND_NAME_BUFFER = (foundName as PUCHAR)
                    .offset(core::mem::size_of::<FIND_NAME_HEADER>() as isize)
                    .offset(
                        ((*foundName).node_count as usize
                            * core::mem::size_of::<FIND_NAME_BUFFER>())
                            as isize,
                    )
                    as PFIND_NAME_BUFFER;
                core::ptr::write_bytes(
                    &raw mut (*findNameBuffer).destination_addr as *mut u8,
                    0,
                    2,
                );
                core::ptr::copy_nonoverlapping(
                    &raw const (*adapter).ipr.dwAddr as *const u8,
                    (&raw mut (*findNameBuffer).destination_addr as *mut u8).offset(2),
                    core::mem::size_of::<DWORD>(),
                );
                core::ptr::write_bytes(
                    &raw mut (*findNameBuffer).source_addr as *mut u8,
                    0,
                    2,
                );
                core::ptr::copy_nonoverlapping(
                    (&raw const (*cacheEntry).addresses as *const DWORD)
                        .offset(ndx as isize) as *const u8,
                    (&raw mut (*findNameBuffer).source_addr as *mut u8).offset(2),
                    core::mem::size_of::<DWORD>(),
                );
                (*foundName).node_count = (*foundName).node_count.wrapping_add(1);
                ndx = ndx.wrapping_add(1);
            }
            if spaceFor < (*cacheEntry).numAddresses {
                ret = NRC_BUFLEN as UCHAR;
            }
        } else {
            ret = NRC_CMDTMO as UCHAR;
        }
    }

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"NetBTFindName\0".as_ptr() as *const ::core::ffi::c_char,
            895,
            b"returning 0x%02x\n\0".as_ptr() as *const ::core::ffi::c_char,
            ret as i32,
        );
    }

    ret
}
unsafe extern "C" fn NetBTSessionReq(
    fd: SOCKET,
    calledName: *const UCHAR,
    callingName: *const UCHAR,
) -> UCHAR {
    let mut buffer: [UCHAR; (NBSS_HDRSIZE + MAX_DOMAIN_NAME_LEN * 2) as usize] = [0; (NBSS_HDRSIZE + MAX_DOMAIN_NAME_LEN * 2) as usize];
    let mut ret: UCHAR;
    let mut r: i32;
    let mut len: u32 = 0;
    let mut bytesSent: DWORD = 0;
    let mut bytesReceived: DWORD = 0;
    let mut recvFlags: DWORD = 0;
    let mut wsaBuf: WSABUF = core::mem::zeroed();

    buffer[0] = NBSS_REQ as UCHAR;
    buffer[1] = 0;

    len = len.wrapping_add(
        NetBTNameEncode(calledName, buffer.as_mut_ptr().add(NBSS_HDRSIZE as usize) as *mut UCHAR)
            as u32,
    );
    len = len.wrapping_add(
        NetBTNameEncode(
            callingName,
            buffer.as_mut_ptr().add((NBSS_HDRSIZE as u32).wrapping_add(len) as usize) as *mut UCHAR,
        ) as u32,
    );

    ((buffer.as_mut_ptr() as *mut UCHAR).add(2) as *mut WORD).write_unaligned(htons(len as u16) as WORD);

    wsaBuf.len = len.wrapping_add(NBSS_HDRSIZE as u32);
    wsaBuf.buf = buffer.as_mut_ptr() as *mut CHAR;

    r = WSASend(
        fd,
        &raw mut wsaBuf as *mut c_void,
        1,
        &raw mut bytesSent,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if r < 0 || bytesSent < len.wrapping_add(NBSS_HDRSIZE as u32) {
        if (*(__wine_dbch___default)).flags as i32
            & ((1i32) << __WINE_DBCL_ERR as i32)
            != 0
        {
            let dbch = __wine_dbch___default;
            ros_dbg_log(
                __WINE_DBCL_ERR,
                dbch,
                b"dll/win32/netapi32/nbt.c\0"
                    .as_ptr() as *const i8,
                b"NetBTSessionReq\0".as_ptr() as *const i8,
                922,
                b"send failed\n\0".as_ptr() as *const i8,
            );
        }
        return NRC_SABORT as UCHAR;
    }

    wsaBuf.len = (NBSS_HDRSIZE + 1) as u32;
    r = WSARecv(
        fd,
        &raw mut wsaBuf as *mut c_void,
        1,
        &raw mut bytesReceived,
        &raw mut recvFlags,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if r < 0 || bytesReceived < NBSS_HDRSIZE as u32 {
        ret = NRC_SABORT as UCHAR;
    } else if buffer[0] as i32 == NBSS_NACK as i32 {
        if r == (NBSS_HDRSIZE + 1) as i32 {
            match buffer[NBSS_HDRSIZE as usize] {
                NBSS_ERR_INSUFFICIENT_RESOURCES => {
                    ret = NRC_REMTFUL as UCHAR;
                }
                _ => {
                    ret = NRC_NOCALL as UCHAR;
                }
            }
        } else {
            ret = NRC_NOCALL as UCHAR;
        }
    } else if buffer[0] as i32 == NBSS_RETARGET as i32 {
        if (*(__wine_dbch___default)).flags as i32
            & ((1i32) << __WINE_DBCL_FIXME as i32)
            != 0
        {
            let dbch_0 = __wine_dbch___default;
            ros_dbg_log(
                __WINE_DBCL_FIXME,
                dbch_0,
                b"dll/win32/netapi32/nbt.c\0"
                    .as_ptr() as *const i8,
                b"NetBTSessionReq\0".as_ptr() as *const i8,
                952,
                b"Got a session retarget, can't deal\n\0".as_ptr() as *const i8,
            );
        }
        ret = NRC_NOCALL as UCHAR;
    } else if buffer[0] as i32 == NBSS_ACK as i32 {
        ret = NRC_GOODRET as UCHAR;
    } else {
        ret = NRC_SYSTEM as UCHAR;
    }

    if (*(__wine_dbch___default)).flags as i32
        & ((1i32) << __WINE_DBCL_TRACE as i32)
        != 0
    {
        let dbch_1 = __wine_dbch___default;
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            dbch_1,
            b"dll/win32/netapi32/nbt.c\0"
                .as_ptr() as *const i8,
            b"NetBTSessionReq\0".as_ptr() as *const i8,
            960,
            b"returning 0x%02x\n\0".as_ptr() as *const i8,
            ret as i32,
        );
    }
    ret
}
unsafe extern "system" fn NetBTCall(
    adapt: *mut core::ffi::c_void,
    ncb: PNCB,
    sess: *mut *mut core::ffi::c_void,
) -> u8 {
    let adapter = adapt as *mut NetBTAdapter;
    let mut ret: u8;
    let mut cacheEntry: *const NBNameCacheEntry = core::ptr::null();

    if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetBTCall\0".as_ptr() as *const core::ffi::c_char,
            970,
            b"adapt %p, ncb %p\n\0".as_ptr() as *const core::ffi::c_char,
            adapt,
            ncb,
        );
    }
    if adapter.is_null() {
        return NRC_ENVNOTDEF as u8;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as u8;
    }
    if sess.is_null() {
        return NRC_BADDR as u8;
    }

    ret = NetBTInternalFindName(adapter, ncb, &mut cacheEntry);
    if ret as i32 == NRC_GOODRET as i32 {
        if !cacheEntry.is_null() && unsafe { (*cacheEntry).numAddresses } > 0 {
            let fd = unsafe {
                WSASocketA(
                    PF_INET as c_int,
                    SOCK_STREAM as c_int,
                    IPPROTO_TCP as i32,
                    core::ptr::null_mut(),
                    0,
                    WSA_FLAG_OVERLAPPED,
                )
            };
            if fd != INVALID_SOCKET {
                let mut timeout: u32;
                let mut sin = unsafe { core::mem::zeroed::<sockaddr_in>() };

                unsafe {
                    if (*ncb).ncb_rto as i32 > 0 {
                        timeout = ((*ncb).ncb_rto as i32 * 500) as u32;
                        setsockopt(
                            fd,
                            SOL_SOCKET as c_int,
                            SO_RCVTIMEO as c_int,
                            &mut timeout as *mut u32 as *mut core::ffi::c_char,
                            core::mem::size_of::<u32>() as i32,
                        );
                    }
                    if (*ncb).ncb_sto as i32 > 0 {
                        timeout = ((*ncb).ncb_sto as i32 * 500) as u32;
                        setsockopt(
                            fd,
                            SOL_SOCKET as c_int,
                            SO_SNDTIMEO as c_int,
                            &mut timeout as *mut u32 as *mut core::ffi::c_char,
                            core::mem::size_of::<u32>() as i32,
                        );
                    }
                }

                core::ptr::write_bytes(&mut sin, 0, 1);
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!((*cacheEntry).addresses[0]) as *const u32 as *const core::ffi::c_void,
                    &raw mut sin.sin_addr as *mut core::ffi::c_void,
                    core::mem::size_of::<IN_ADDR>(),
                );
                sin.sin_family = AF_INET as i16;
                sin.sin_port = (unsafe { htons(PORT_NBSS as u16) }) as u16;

                if unsafe {
                    connect(
                        fd,
                        &raw mut sin as *const c_void,
                        core::mem::size_of::<sockaddr_in>() as i32,
                    )
                } == SOCKET_ERROR
                {
                    ret = NRC_CMDTMO as u8;
                } else {
                    static mut fakedCalledName: [u8; 11] = *b"*SMBSERVER\0";
                    let calledParty: *const u8 = if unsafe { (*cacheEntry).nbname[0] } as i32
                        == '*' as i32
                    {
                        &raw const fakedCalledName as *const u8
                    } else {
                        core::ptr::addr_of!((*cacheEntry).nbname) as *const u8
                    };
                    ret =
                        NetBTSessionReq(fd, calledParty, core::ptr::addr_of_mut!((*ncb).ncb_name) as *mut u8);
                    if ret as i32 != NRC_GOODRET as i32
                        && (unsafe { *calledParty.offset(0) } as i32) == '*' as i32
                    {
                        if ((*__wine_dbch___default).flags as i32
                            & (1 << __WINE_DBCL_FIXME))
                            != 0
                        {
                            ros_dbg_log(
                                __WINE_DBCL_FIXME,
                                __wine_dbch___default,
                                b"dll/win32/netapi32/nbt.c\0"
                                    .as_ptr() as *const core::ffi::c_char,
                                b"NetBTCall\0".as_ptr() as *const core::ffi::c_char,
                                1023,
                                b"NBT session to \"*SMBSERVER\" refused,\n\0".as_ptr()
                                    as *const core::ffi::c_char,
                            );
                        }
                        if ((*__wine_dbch___default).flags as i32
                            & (1 << __WINE_DBCL_FIXME))
                            != 0
                        {
                            ros_dbg_log(
                                __WINE_DBCL_FIXME,
                                __wine_dbch___default,
                                b"dll/win32/netapi32/nbt.c\0"
                                    .as_ptr() as *const core::ffi::c_char,
                                b"NetBTCall\0".as_ptr() as *const core::ffi::c_char,
                                1024,
                                b"should try finding name using ASTAT\n\0".as_ptr()
                                    as *const core::ffi::c_char,
                            );
                        }
                    }
                }
                if ret as i32 != NRC_GOODRET as i32 {
                    unsafe { closesocket(fd) };
                } else {
                    let session = unsafe {
                        HeapAlloc(
                            GetProcessHeap(),
                            HEAP_ZERO_MEMORY,
                            core::mem::size_of::<NetBTSession>() as SIZE_T,
                        ) as *mut NetBTSession
                    };
                    if !session.is_null() {
                        unsafe {
                            (*session).fd = fd;
                            InitializeCriticalSection(&raw mut (*session).cs);
                            (*(*session).cs.DebugInfo)
                                .__bindgen_anon_1
                                .Spare[0] = b"dll/win32/netapi32/nbt.c: \
                                             NetBTSession.cs\0"
                                .as_ptr() as DWORD_PTR;
                            *sess = session as *mut core::ffi::c_void;
                        }
                    } else {
                        ret = NRC_OSRESNOTAV as u8;
                        unsafe { closesocket(fd) };
                    }
                }
            } else {
                ret = NRC_OSRESNOTAV as u8;
            }
        } else {
            ret = NRC_NAMERR as u8;
        }
    }

    if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetBTCall\0".as_ptr() as *const core::ffi::c_char,
            1054,
            b"returning 0x%02x\n\0".as_ptr() as *const core::ffi::c_char,
            ret as i32,
        );
    }
    ret
}
unsafe extern "system" fn NetBTSend(adapt: *mut core::ffi::c_void, sess: *mut core::ffi::c_void, ncb: PNCB) -> UCHAR {
    let adapter = adapt as *mut NetBTAdapter;
    let session = sess as *mut NetBTSession;
    let mut buffer: [UCHAR; 4] = [0; 4];
    let mut ret: UCHAR;
    let r: i32;
    let mut wsaBufs: [WSABUF; 2] = [
        WSABUF { len: 0, buf: core::ptr::null_mut::<CHAR>() },
        WSABUF { len: 0, buf: core::ptr::null_mut::<CHAR>() },
    ];
    let mut bytesSent: DWORD = 0;

    if !adapter.is_null() && (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0" as *const u8 as *const i8,
            b"NetBTSend\0" as *const u8 as *const i8,
            1074,
            b"adapt %p, session %p, NCB %p\n\0" as *const u8 as *const i8,
            adapt,
            session,
            ncb,
        );
    }

    if adapter.is_null() { return NRC_ENVNOTDEF; }
    if ncb.is_null() { return NRC_INVADDRESS; }
    if (*ncb).ncb_buffer.is_null() { return NRC_BADDR; }
    if session.is_null() { return NRC_SNUMOUT; }
    if (*session).fd == INVALID_SOCKET { return NRC_SNUMOUT; }

    buffer[0] = NBSS_MSG as UCHAR;
    buffer[1] = 0;
    *((&raw mut buffer[2] as *mut UCHAR as *mut u16)) = htons((*ncb).ncb_length as u16);

    wsaBufs[0].len = NBSS_HDRSIZE as u32;
    wsaBufs[0].buf = buffer.as_mut_ptr() as *mut CHAR;
    wsaBufs[1].len = (*ncb).ncb_length as u32;
    wsaBufs[1].buf = (*ncb).ncb_buffer as *mut CHAR;

    r = WSASend(
        (*session).fd,
        wsaBufs.as_mut_ptr() as *mut c_void,
        (core::mem::size_of::<[WSABUF; 2]>() / core::mem::size_of::<WSABUF>()) as DWORD,
        &raw mut bytesSent,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if r == SOCKET_ERROR {
        NetBIOSHangupSession(ncb);
        ret = NRC_SABORT;
    } else if bytesSent < (NBSS_HDRSIZE + (*ncb).ncb_length as u32) {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_FIXME,
                __wine_dbch___default,
                b"dll/win32/netapi32/nbt.c\0" as *const u8 as *const i8,
                b"NetBTSend\0" as *const u8 as *const i8,
                1101,
                b"Only sent %d bytes (of %d), hanging up session\n\0" as *const u8 as *const i8,
                bytesSent,
                4i32 + (*ncb).ncb_length as i32,
            );
        }
        NetBIOSHangupSession(ncb);
        ret = NRC_SABORT;
    } else {
        ret = NRC_GOODRET;
        (*adapter).xmit_success = (*adapter).xmit_success.wrapping_add(1);
    }

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0" as *const u8 as *const i8,
            b"NetBTSend\0" as *const u8 as *const i8,
            1110,
            b"returning 0x%02x\n\0" as *const u8 as *const i8,
            ret as i32,
        );
    }
    ret
}
unsafe extern "system" fn NetBTRecv(
    adapt: *mut core::ffi::c_void,
    sess: *mut core::ffi::c_void,
    ncb: PNCB,
) -> UCHAR {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0"
                .as_ptr() as *const i8,
            b"NetBTRecv\0".as_ptr() as *const i8,
            1123i32,
            b"adapt %p, session %p, NCB %p\n\0".as_ptr() as *const i8,
            adapt,
            sess,
            ncb,
        );
    }
    let adapter = adapt as *mut NetBTAdapter;
    let session = sess as *mut NetBTSession;
    let mut buffer: [UCHAR; 4] = [0; 4];
    let mut wsaBufs: [WSABUF; 2] = [
        WSABUF { len: 0, buf: core::ptr::null_mut::<CHAR>() },
        WSABUF { len: 0, buf: core::ptr::null_mut::<CHAR>() },
    ];
    let mut bufferCount: DWORD = 0;
    let mut bytesReceived: DWORD = 0;
    let mut flags: DWORD = 0;

    if adapter.is_null() {
        return NRC_ENVNOTDEF as UCHAR;
    }
    if ncb.is_null() {
        return NRC_BADDR as UCHAR;
    }
    if (*ncb).ncb_buffer.is_null() {
        return NRC_BADDR as UCHAR;
    }
    if session.is_null() {
        return NRC_SNUMOUT as UCHAR;
    }
    if (*session).fd == INVALID_SOCKET {
        return NRC_SNUMOUT as UCHAR;
    }

    EnterCriticalSection(&raw mut (*session).cs);

    bufferCount = 0;
    if (*session).bytesPending == 0 {
        bufferCount = bufferCount.wrapping_add(1);
        wsaBufs[0].len = NBSS_HDRSIZE as u32;
        wsaBufs[0].buf = buffer.as_mut_ptr() as *mut CHAR;
    }
    wsaBufs[bufferCount as usize].len = (*ncb).ncb_length as u32;
    wsaBufs[bufferCount as usize].buf = (*ncb).ncb_buffer as *mut CHAR;
    bufferCount = bufferCount.wrapping_add(1);

    flags = 0;
    let r = WSARecv(
        (*session).fd,
        wsaBufs.as_mut_ptr() as *mut c_void,
        bufferCount,
        &mut bytesReceived,
        &mut flags,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    let ret = 'error: {
        if r == SOCKET_ERROR && WSAGetLastError() as i32 != WSAEWOULDBLOCK as i32 {
            LeaveCriticalSection(&raw mut (*session).cs);
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_ERR as u32)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/nbt.c\0"
                        .as_ptr() as *const i8,
                    b"NetBTRecv\0".as_ptr() as *const i8,
                    1150i32,
                    b"Receive error, WSAGetLastError() returns %d\n\0".as_ptr() as *const i8,
                    WSAGetLastError(),
                );
            }
            NetBIOSHangupSession(ncb);
            break 'error NRC_SABORT as UCHAR;
        }
        if (*(&raw mut (*ncb).ncb_reserve as *mut UCHAR as *const BOOL)) != 0 {
            LeaveCriticalSection(&raw mut (*session).cs);
            break 'error NRC_CMDCAN as UCHAR;
        }
        if bufferCount == 2 {
            if buffer[0] as i32 == NBSS_KEEPALIVE as i32 {
                LeaveCriticalSection(&raw mut (*session).cs);
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_FIXME,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/nbt.c\0"
                            .as_ptr() as *const i8,
                        b"NetBTRecv\0".as_ptr() as *const i8,
                        1166i32,
                        b"Oops, received a session keepalive and lost my place\n\0".as_ptr()
                            as *const i8,
                    );
                }
                NetBIOSHangupSession(ncb);
                break 'error NRC_SABORT as UCHAR;
            }
            if buffer[0] as i32 != NBSS_MSG as i32 {
                LeaveCriticalSection(&raw mut (*session).cs);
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_FIXME,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/nbt.c\0"
                            .as_ptr() as *const i8,
                        b"NetBTRecv\0".as_ptr() as *const i8,
                        1176i32,
                        b"Received unexpected session msg type %d\n\0".as_ptr() as *const i8,
                        buffer[0] as i32,
                    );
                }
                NetBIOSHangupSession(ncb);
                break 'error NRC_SABORT as UCHAR;
            }
            if buffer[1] as i32 & NBSS_EXTENSION as i32 != 0 {
                LeaveCriticalSection(&raw mut (*session).cs);
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_FIXME,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/nbt.c\0"
                            .as_ptr() as *const i8,
                        b"NetBTRecv\0".as_ptr() as *const i8,
                        1186i32,
                        b"Received a message that's too long for my taste\n\0".as_ptr()
                            as *const i8,
                    );
                }
                NetBIOSHangupSession(ncb);
                break 'error NRC_SABORT as UCHAR;
            }
            (*session).bytesPending = (NBSS_HDRSIZE
                + ntohs((buffer.as_ptr().add(2usize) as *const WORD).read_unaligned()) as u32
                - bytesReceived) as DWORD;
            (*ncb).ncb_length = (bytesReceived.wrapping_sub(NBSS_HDRSIZE)) as WORD;
            LeaveCriticalSection(&raw mut (*session).cs);
        } else {
            if bytesReceived < (*session).bytesPending {
                (*session).bytesPending = ((*session).bytesPending as u32)
                    .wrapping_sub(bytesReceived) as DWORD;
            } else {
                (*session).bytesPending = 0;
            }
            LeaveCriticalSection(&raw mut (*session).cs);
            (*ncb).ncb_length = bytesReceived as WORD;
        }
        if (*session).bytesPending > 0 {
            NRC_INCOMP as UCHAR
        } else {
            (*adapter).recv_success = (*adapter).recv_success.wrapping_add(1);
            NRC_GOODRET as UCHAR
        }
    };

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0"
                .as_ptr() as *const i8,
            b"NetBTRecv\0".as_ptr() as *const i8,
            1218i32,
            b"returning 0x%02x\n\0".as_ptr() as *const i8,
            ret as i32,
        );
    }
    ret
}
unsafe extern "system" fn NetBTHangup(adapt: *mut ::core::ffi::c_void, sess: *mut ::core::ffi::c_void) -> UCHAR {
    let session: *mut NetBTSession = sess as *mut NetBTSession;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/nbt.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"NetBTHangup\0".as_ptr() as *const ::core::ffi::c_char,
            1226 as ::core::ffi::c_int,
            b"adapt %p, session %p\n\0".as_ptr() as *const ::core::ffi::c_char,
            adapt,
            session,
        );
    }

    if session.is_null() {
        return NRC_SNUMOUT as UCHAR;
    }

    closesocket((*session).fd);
    (*session).fd = INVALID_SOCKET;
    (*session).bytesPending = 0 as DWORD;
    (*(*session).cs.DebugInfo).__bindgen_anon_1.Spare[0] = 0 as DWORD_PTR;
    DeleteCriticalSection(core::ptr::addr_of_mut!((*session).cs));
    HeapFree(GetProcessHeap(), 0 as DWORD, session as PVOID);
    return NRC_GOODRET as UCHAR;
}
unsafe extern "system" fn NetBTCleanupAdapter(adapt: *mut core::ffi::c_void) {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0".as_ptr() as *const core::ffi::c_char,
            b"NetBTCleanupAdapter\0".as_ptr() as *const core::ffi::c_char,
            1245i32,
            b"adapt %p\n\0".as_ptr() as *const core::ffi::c_char,
            adapt,
        );
    }
    if !adapt.is_null() {
        let adapter: *mut NetBTAdapter = adapt as *mut NetBTAdapter;
        if !(*adapter).nameCache.is_null() {
            NBNameCacheDestroy((*adapter).nameCache as *mut crate::netapi32::nbnamecache::NBNameCache);
        }
        HeapFree(GetProcessHeap(), 0u32, adapt as *mut core::ffi::c_void);
    }
}
unsafe extern "system" fn NetBTCleanup() {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0".as_ptr() as *const core::ffi::c_char,
            b"NetBTCleanup\0".as_ptr() as *const core::ffi::c_char,
            1258 as core::ffi::c_int,
            b"\n\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    if !gNameCache.is_null() {
        NBNameCacheDestroy(gNameCache as *mut crate::netapi32::nbnamecache::NBNameCache);
        gNameCache = core::ptr::null_mut::<NBNameCache>();
    }
}
unsafe extern "C" fn NetBTRegisterAdapter(ipRow: *const MIB_IPADDRROW) -> UCHAR {
    let mut ret: UCHAR;
    if ipRow.is_null() {
        return NRC_BADDR as UCHAR;
    }
    let adapter = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as u32,
        core::mem::size_of::<NetBTAdapter>() as SIZE_T,
    ) as *mut NetBTAdapter;
    if !adapter.is_null() {
        (*adapter).ipr = core::ptr::read(ipRow);
        if NetBIOSRegisterAdapter(
            gTransportID,
            (*ipRow).dwIndex as u32,
            adapter as *mut core::ffi::c_void,
        ) == 0
        {
            NetBTCleanupAdapter(adapter as *mut core::ffi::c_void);
            ret = NRC_SYSTEM as UCHAR;
        } else {
            ret = NRC_GOODRET as UCHAR;
        }
    } else {
        ret = NRC_OSRESNOTAV as UCHAR;
    }
    ret
}

unsafe extern "system" fn NetBTEnumCallback(
    totalLANAs: UCHAR,
    lanaIndex: UCHAR,
    transport: ULONG,
    data: *const NetBIOSAdapterImpl,
    closure: *mut core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL = 0;
    let table: PMIB_IPADDRTABLE = closure as PMIB_IPADDRTABLE;
    if !table.is_null() && !data.is_null() {
        let mut ndx: DWORD = 0;
        ret = 0;
        ndx = 0;
        while ret == 0 && ndx < (*table).dwNumEntries {
            let adapter: *const NetBTAdapter = (*data).data as *const NetBTAdapter;
            let row = &mut *((*table).table.as_mut_ptr() as *mut MIB_IPADDRROW).offset(ndx as isize);
            if row.dwIndex == (*adapter).ipr.dwIndex {
                NetBIOSEnableAdapter((*data).lana);
                row.dwAddr = INADDR_LOOPBACK as DWORD;
                ret = 1;
            }
            ndx = ndx.wrapping_add(1);
        }
    } else {
        ret = 0;
    }
    ret
}
unsafe extern "system" fn NetBTEnum() -> UCHAR {
    let mut ret: UCHAR;
    let mut size: DWORD = 0;

    if GetIpAddrTable(
        core::ptr::null_mut::<MIB_IPADDRTABLE>(),
        &mut size,
        FALSE,
    ) == ERROR_INSUFFICIENT_BUFFER
    {
        let mut ipAddrs: PMIB_IPADDRTABLE = core::ptr::null_mut();
        let mut coalesceTable: PMIB_IPADDRTABLE = core::ptr::null_mut();
        let numIPAddrs: DWORD = (((size as usize).wrapping_sub(core::mem::size_of::<MIB_IPADDRTABLE>())
            / core::mem::size_of::<MIB_IPADDRROW>())
            + 1) as DWORD;

        ipAddrs = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            size as SIZE_T,
        ) as PMIB_IPADDRTABLE;

        if !ipAddrs.is_null() {
            let table_size = core::mem::size_of::<MIB_IPADDRTABLE>()
                + (::core::cmp::min(numIPAddrs as usize, (MAX_LANA + 1) as usize) - 1)
                    * core::mem::size_of::<MIB_IPADDRROW>();
            coalesceTable = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                table_size as SIZE_T,
            ) as PMIB_IPADDRTABLE;
        }

        if !ipAddrs.is_null() && !coalesceTable.is_null() {
            if GetIpAddrTable(ipAddrs, &mut size, FALSE) == ERROR_SUCCESS {
                let mut ndx: DWORD = 0;
                while ndx < (*ipAddrs).dwNumEntries {
                    // MIB_IPADDRTABLE.table is a C flexible-array member (`[_; 1]` stub,
                    // heap-allocated for dwNumEntries rows). Index by pointer arithmetic —
                    // the `[..]` operator bounds-checks against the length-1 stub and panics.
                    let addr_entry = &*(*ipAddrs).table.as_ptr().add(ndx as usize);
                    if addr_entry.dwAddr & addr_entry.dwMask
                        != htonl((INADDR_LOOPBACK & IN_CLASSA_NET) as u_long)
                    {
                        let mut newNetwork: BOOL = TRUE;
                        let mut innerIndex: DWORD = 0;
                        while newNetwork != 0 && innerIndex < (*coalesceTable).dwNumEntries {
                            let cached = &*(*coalesceTable).table.as_ptr().add(innerIndex as usize);
                            if addr_entry.dwAddr & addr_entry.dwMask == cached.dwAddr & cached.dwMask {
                                newNetwork = FALSE;
                            }
                            innerIndex += 1;
                        }
                        if newNetwork != 0 {
                            let idx = (*coalesceTable).dwNumEntries;
                            (*coalesceTable).dwNumEntries += 1;
                            core::ptr::copy_nonoverlapping(
                                addr_entry as *const MIB_IPADDRROW,
                                (*coalesceTable).table.as_mut_ptr().add(idx as usize),
                                1,
                            );
                        }
                    }
                    ndx += 1;
                }

                NetBIOSEnumAdapters(
                    gTransportID,
                    Some(NetBTEnumCallback),
                    coalesceTable as *mut core::ffi::c_void,
                );
                ret = NRC_GOODRET as UCHAR;

                ndx = 0;
                while ret as i32 == NRC_GOODRET as i32 && ndx < (*coalesceTable).dwNumEntries {
                    let cache_entry = &*(*coalesceTable).table.as_ptr().add(ndx as usize);
                    if cache_entry.dwAddr != INADDR_LOOPBACK {
                        ret = NetBTRegisterAdapter(cache_entry as *const MIB_IPADDRROW as *mut MIB_IPADDRROW);
                    }
                    ndx += 1;
                }
            } else {
                ret = NRC_SYSTEM as UCHAR;
            }

            HeapFree(GetProcessHeap(), 0, ipAddrs as PVOID);
            HeapFree(GetProcessHeap(), 0, coalesceTable as PVOID);
        } else {
            ret = NRC_OSRESNOTAV as UCHAR;
        }
    } else {
        ret = NRC_SYSTEM as UCHAR;
    }

    ret
}
#[no_mangle]
pub unsafe extern "C" fn NetBTInit() {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut transport: NetBIOSTransport = unsafe { core::mem::zeroed() };
    let mut ret: LONG;

    if unsafe { (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 } {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbt.c\0".as_ptr() as *const core::ffi::c_char,
            b"NetBTInit\0".as_ptr() as *const core::ffi::c_char,
            1438,
            b"\n\0".as_ptr() as *const core::ffi::c_char,
        );
    }

    gEnableDNS = 1;
    gBCastQueries = BCAST_QUERIES as DWORD;
    gBCastQueryTimeout = BCAST_QUERY_TIMEOUT as DWORD;
    gWINSQueries = WINS_QUERIES as DWORD;
    gWINSQueryTimeout = WINS_QUERY_TIMEOUT as DWORD;
    gNumWINSServers = 0i32;
    memset(
        core::ptr::addr_of_mut!(gWINSServers) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[DWORD; 2]>() as size_t,
    );
    gScopeID[0] = b'\0' as core::ffi::c_char;
    gCacheTimeout = CACHE_TIMEOUT as DWORD;

    ret = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        core::ptr::addr_of!(VxD_MSTCPW) as LPCWSTR,
        0,
        KEY_READ as REGSAM,
        &raw mut hKey,
    ) as LONG;
    if ret != ERROR_SUCCESS as i32 {
        ret = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            core::ptr::addr_of!(NetBT_ParametersW) as LPCWSTR,
            0,
            KEY_READ as REGSAM,
            &raw mut hKey,
        ) as LONG;
    }

    if ret == ERROR_SUCCESS as i32 {
        let mut dword: DWORD = 0;
        let mut size: DWORD;

        size = core::mem::size_of::<DWORD>() as DWORD;
        if RegQueryValueExW(
            hKey,
            core::ptr::addr_of!(EnableDNSW) as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut dword as LPBYTE,
            &raw mut size,
        ) == ERROR_SUCCESS as i32
        {
            gEnableDNS = dword as BOOL;
        }

        size = core::mem::size_of::<DWORD>() as DWORD;
        if RegQueryValueExW(
            hKey,
            core::ptr::addr_of!(BcastNameQueryCountW) as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut dword as LPBYTE,
            &raw mut size,
        ) == ERROR_SUCCESS as i32
            && dword >= MIN_QUERIES as DWORD
            && dword <= MAX_QUERIES as DWORD
        {
            gBCastQueries = dword;
        }

        size = core::mem::size_of::<DWORD>() as DWORD;
        if RegQueryValueExW(
            hKey,
            core::ptr::addr_of!(BcastNameQueryTimeoutW) as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut dword as LPBYTE,
            &raw mut size,
        ) == ERROR_SUCCESS as i32
            && dword >= MIN_QUERY_TIMEOUT as DWORD
        {
            gBCastQueryTimeout = dword;
        }

        size = core::mem::size_of::<DWORD>() as DWORD;
        if RegQueryValueExW(
            hKey,
            core::ptr::addr_of!(NameSrvQueryCountW) as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut dword as LPBYTE,
            &raw mut size,
        ) == ERROR_SUCCESS as i32
            && dword >= MIN_QUERIES as DWORD
            && dword <= MAX_QUERIES as DWORD
        {
            gWINSQueries = dword;
        }

        size = core::mem::size_of::<DWORD>() as DWORD;
        if RegQueryValueExW(
            hKey,
            core::ptr::addr_of!(NameSrvQueryTimeoutW) as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut dword as LPBYTE,
            &raw mut size,
        ) == ERROR_SUCCESS as i32
            && dword >= MIN_QUERY_TIMEOUT as DWORD
        {
            gWINSQueryTimeout = dword;
        }

        size = (core::mem::size_of::<[::core::ffi::c_char; 256]>() - 1) as DWORD;
        if RegQueryValueExW(
            hKey,
            core::ptr::addr_of!(ScopeIDW) as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (core::ptr::addr_of_mut!(gScopeID) as LPBYTE).add(1),
            &raw mut size,
        ) == ERROR_SUCCESS as i32
        {
            let mut ptr: *mut core::ffi::c_char;
            let mut lenPtr: *mut core::ffi::c_char;

            ptr = gScopeID.as_mut_ptr().add(1);
            lenPtr = gScopeID.as_mut_ptr();

            while (ptr.offset_from(gScopeID.as_mut_ptr()) as usize)
                < core::mem::size_of::<[::core::ffi::c_char; 256]>()
                && *ptr as core::ffi::c_int != 0
            {
                if *ptr as core::ffi::c_int == '.' as i32 {
                    lenPtr = ptr;
                    *lenPtr = 0 as core::ffi::c_char;
                } else {
                    *lenPtr += 1;
                }
                ptr = ptr.add(1);
            }
        }

        if RegQueryValueExW(
            hKey,
            core::ptr::addr_of!(CacheTimeoutW) as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut dword as LPBYTE,
            &raw mut size,
        ) == ERROR_SUCCESS as i32
            && dword >= MIN_CACHE_TIMEOUT as DWORD
        {
            gCacheTimeout = dword;
        }

        RegCloseKey(hKey);
    }

    if RegOpenKeyW(
        HKEY_CURRENT_USER,
        core::ptr::addr_of!(Config_NetworkW) as LPCWSTR,
        &raw mut hKey,
    ) == ERROR_SUCCESS as i32
    {
        const nsValueNames: [*const core::ffi::c_char; 2] = [
            b"WinsServer\0".as_ptr() as *const core::ffi::c_char,
            b"BackupWinsServer\0".as_ptr() as *const core::ffi::c_char,
        ];
        let mut nsString: [core::ffi::c_char; 16] = unsafe { core::mem::zeroed() };
        let mut size_0: DWORD;
        let mut ndx: usize;

        for ndx in 0..nsValueNames.len() {
            size_0 = (core::mem::size_of::<[core::ffi::c_char; 16]>() / core::mem::size_of::<core::ffi::c_char>()) as DWORD;
            if RegQueryValueExA(
                hKey,
                nsValueNames[ndx],
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                nsString.as_mut_ptr() as LPBYTE,
                &raw mut size_0,
            ) == ERROR_SUCCESS as i32
            {
                let addr: core::ffi::c_ulong = inet_addr(nsString.as_mut_ptr());
                if addr != INADDR_NONE as core::ffi::c_ulong
                    && gNumWINSServers < MAX_WINS_SERVERS as i32
                {
                    gWINSServers[gNumWINSServers as usize] = addr as DWORD;
                    gNumWINSServers += 1;
                }
            }
        }
        RegCloseKey(hKey);
    }

    transport.enumerate = Some(NetBTEnum);
    transport.astat = Some(NetBTAstat);
    transport.findName = Some(NetBTFindName);
    transport.call = Some(NetBTCall);
    transport.send = Some(NetBTSend);
    transport.recv = Some(NetBTRecv);
    transport.hangup = Some(NetBTHangup);
    transport.cleanupAdapter = Some(NetBTCleanupAdapter);
    transport.cleanup = Some(NetBTCleanup);

    memcpy(
        &raw mut gTransportID as *mut core::ffi::c_void,
        TRANSPORT_NBT.as_ptr() as *const core::ffi::c_void,
        core::mem::size_of::<ULONG>() as size_t,
    );

    NetBIOSRegisterTransport(gTransportID, &raw mut transport);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /* Copyright (c) 2003 Juan Lang
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
//  *
//  * I am heavily indebted to Chris Hertel's excellent Implementing CIFS,
//  * http://ubiqx.org/cifs/ , for whatever understanding I have of NBT.
//  * I also stole from Mike McCormack's smb.c and netapi32.c, although little of
//  * that code remains.
//  * Lack of understanding and bugs are my fault.
//  *
//  * FIXME:
//  * - Of the NetBIOS session functions, only client functions are supported, and
//  *   it's likely they'll be the only functions supported.  NBT requires session
//  *   servers to listen on TCP/139.  This requires root privilege, and Samba is
//  *   likely to be listening here already.  This further restricts NetBIOS
//  *   applications, both explicit users and implicit ones:  CreateNamedPipe
//  *   won't actually create a listening pipe, for example, so applications can't
//  *   act as RPC servers using a named pipe protocol binding, DCOM won't be able
//  *   to support callbacks or servers over the named pipe protocol, etc.
//  *
//  * - Datagram support is omitted for the same reason.  To send a NetBIOS
//  *   datagram, you must include the NetBIOS name by which your application is
//  *   known.  This requires you to have registered the name previously, and be
//  *   able to act as a NetBIOS datagram server (listening on UDP/138).
//  *
//  * - Name registration functions are omitted for the same reason--registering a
//  *   name requires you to be able to defend it, and this means listening on
//  *   UDP/137.
//  *   Win98 requires you either use your computer's NetBIOS name (with the NULL
//  *   suffix byte) as the calling name when creating a session, or to register
//  *   a new name before creating one:  it disallows '*' as the calling name.
//  *   Win2K initially starts with an empty name table, and doesn't allow you to
//  *   use the machine's NetBIOS name (with the NULL suffix byte) as the calling
//  *   name.  Although it allows sessions to be created with '*' as the calling
//  *   name, doing so results in timeouts for all receives, because the
//  *   application never gets them.
//  *   So, a well-behaved NetBIOS application will typically want to register a
//  *   name.  I should probably support a do-nothing name list that allows
//  *   NCBADDNAME to add to it, but doesn't actually register the name, or does
//  *   attempt to register it without being able to defend it.
//  *
//  * - Name lookups may not behave quite as you'd expect/like if you have
//  *   multiple LANAs.  If a name is resolvable through DNS, or if you're using
//  *   WINS, it'll resolve on _any_ LANA.  So, a Call will succeed on any LANA as
//  *   well.
//  *   I'm not sure how Windows behaves in this case.  I could try to force
//  *   lookups to the correct adapter by using one of the GetPreferred*
//  *   functions, but with the possibility of multiple adapters in the same
//  *   same subnet, there's no guarantee that what IpHlpApi thinks is the
//  *   preferred adapter will actually be a LANA.  (It's highly probable because
//  *   this is an unusual configuration, but not guaranteed.)
//  *
//  * See also other FIXMEs in the code.
//  */
//
// #include "netapi32.h"
//
// #include <winsock2.h>
// #include <winreg.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(netbios);
//
// #define PORT_NBNS 137
// #define PORT_NBDG 138
// #define PORT_NBSS 139
//
// #ifndef INADDR_NONE
// #define INADDR_NONE ~0UL
// #endif
//
// #define NBR_ADDWORD(p,word) (*(WORD *)(p)) = htons(word)
// #define NBR_GETWORD(p) ntohs(*(WORD *)(p))
//
// #define MIN_QUERIES         1
// #define MAX_QUERIES         0xffff
// #define MIN_QUERY_TIMEOUT   100
// #define MAX_QUERY_TIMEOUT   0xffffffff
// #define BCAST_QUERIES       3
// #define BCAST_QUERY_TIMEOUT 750
// #define WINS_QUERIES        3
// #define WINS_QUERY_TIMEOUT  750
// #define MAX_WINS_SERVERS    2
// #define MIN_CACHE_TIMEOUT   60000
// #define CACHE_TIMEOUT       360000
//
// #define MAX_NBT_NAME_SZ            255
// #define SIMPLE_NAME_QUERY_PKT_SIZE 16 + MAX_NBT_NAME_SZ
//
// #define NBNS_TYPE_NB             0x0020
// #define NBNS_TYPE_NBSTAT         0x0021
// #define NBNS_CLASS_INTERNET      0x00001
// #define NBNS_HEADER_SIZE         (sizeof(WORD) * 6)
// #define NBNS_RESPONSE_AND_OPCODE 0xf800
// #define NBNS_RESPONSE_AND_QUERY  0x8000
// #define NBNS_REPLYCODE           0x0f
//
// #define NBSS_HDRSIZE 4
//
// #define NBSS_MSG       0x00
// #define NBSS_REQ       0x81
// #define NBSS_ACK       0x82
// #define NBSS_NACK      0x83
// #define NBSS_RETARGET  0x84
// #define NBSS_KEEPALIVE 0x85
//
// #define NBSS_ERR_NOT_LISTENING_ON_NAME    0x80
// #define NBSS_ERR_NOT_LISTENING_FOR_CALLER 0x81
// #define NBSS_ERR_BAD_NAME                 0x82
// #define NBSS_ERR_INSUFFICIENT_RESOURCES   0x83
//
// #define NBSS_EXTENSION 0x01
//
// typedef struct _NetBTSession
// {
//     CRITICAL_SECTION cs;
//     SOCKET           fd;
//     DWORD            bytesPending;
// } NetBTSession;
//
// typedef struct _NetBTAdapter
// {
//     MIB_IPADDRROW       ipr;
//     WORD                nameQueryXID;
//     struct NBNameCache *nameCache;
//     DWORD               xmit_success;
//     DWORD               recv_success;
// } NetBTAdapter;
//
// static ULONG gTransportID;
// static BOOL  gEnableDNS;
// static DWORD gBCastQueries;
// static DWORD gBCastQueryTimeout;
// static DWORD gWINSQueries;
// static DWORD gWINSQueryTimeout;
// static DWORD gWINSServers[MAX_WINS_SERVERS];
// static int   gNumWINSServers;
// static char  gScopeID[MAX_SCOPE_ID_LEN];
// static DWORD gCacheTimeout;
// static struct NBNameCache *gNameCache;
//
// /* Converts from a NetBIOS name into a Second Level Encoding-formatted name.
//  * Assumes p is not NULL and is either NULL terminated or has at most NCBNAMSZ
//  * bytes, and buffer has at least MAX_NBT_NAME_SZ bytes.  Pads with space bytes
//  * if p is NULL-terminated.  Returns the number of bytes stored in buffer.
//  */
// static int NetBTNameEncode(const UCHAR *p, UCHAR *buffer)
// {
//     int i,len=0;
//
//     if (!p) return 0;
//     if (!buffer) return 0;
//
//     buffer[len++] = NCBNAMSZ * 2;
//     for (i = 0; i < NCBNAMSZ && p[i]; i++)
//     {
//         buffer[len++] = ((p[i] & 0xf0) >> 4) + 'A';
//         buffer[len++] =  (p[i] & 0x0f) + 'A';
//     }
//     while (len < NCBNAMSZ * 2)
//     {
//         buffer[len++] = 'C';
//         buffer[len++] = 'A';
//     }
//     if (*gScopeID)
//     {
//         int scopeIDLen = strlen(gScopeID);
//
//         memcpy(buffer + len, gScopeID, scopeIDLen);
//         len += scopeIDLen;
//     }
//     buffer[len++] = 0;     /* add second terminator */
//     return len;
// }
//
// /* Creates a NBT name request packet for name in buffer.  If broadcast is true,
//  * creates a broadcast request, otherwise creates a unicast request.
//  * Returns the number of bytes stored in buffer.
//  */
// static DWORD NetBTNameReq(const UCHAR name[NCBNAMSZ], WORD xid, WORD qtype,
//  BOOL broadcast, UCHAR *buffer, int len)
// {
//     int i = 0;
//
//     if (len < SIMPLE_NAME_QUERY_PKT_SIZE) return 0;
//
//     NBR_ADDWORD(&buffer[i],xid);    i+=2; /* transaction */
//     if (broadcast)
//     {
//         NBR_ADDWORD(&buffer[i],0x0110); /* flags: r=req,op=query,rd=1,b=1 */
//         i+=2;
//     }
//     else
//     {
//         NBR_ADDWORD(&buffer[i],0x0100); /* flags: r=req,op=query,rd=1,b=0 */
//         i+=2;
//     }
//     NBR_ADDWORD(&buffer[i],0x0001); i+=2; /* one name query */
//     NBR_ADDWORD(&buffer[i],0x0000); i+=2; /* zero answers */
//     NBR_ADDWORD(&buffer[i],0x0000); i+=2; /* zero authorities */
//     NBR_ADDWORD(&buffer[i],0x0000); i+=2; /* zero additional */
//
//     i += NetBTNameEncode(name, &buffer[i]);
//
//     NBR_ADDWORD(&buffer[i],qtype); i+=2;
//     NBR_ADDWORD(&buffer[i],NBNS_CLASS_INTERNET); i+=2;
//
//     return i;
// }
//
// /* Sends a name query request for name on fd to destAddr.  Sets SO_BROADCAST on
//  * fd if broadcast is TRUE.  Assumes fd is not INVALID_SOCKET, and name is not
//  * NULL.
//  * Returns 0 on success, -1 on failure.
//  */
// static int NetBTSendNameQuery(SOCKET fd, const UCHAR name[NCBNAMSZ], WORD xid,
//  WORD qtype, DWORD destAddr, BOOL broadcast)
// {
//     int ret = 0, on = 1;
//     struct in_addr addr;
//
//     addr.s_addr = destAddr;
//     TRACE("name %s, dest addr %s\n", name, inet_ntoa(addr));
//
//     if (broadcast)
//         ret = setsockopt(fd, SOL_SOCKET, SO_BROADCAST, (const char*)&on, sizeof(on));
//     if(ret == 0)
//     {
//         WSABUF wsaBuf;
//         UCHAR buf[SIMPLE_NAME_QUERY_PKT_SIZE];
//         struct sockaddr_in sin;
//
//         memset(&sin, 0, sizeof(sin));
//         sin.sin_addr.s_addr = destAddr;
//         sin.sin_family      = AF_INET;
//         sin.sin_port        = htons(PORT_NBNS);
//
//         wsaBuf.buf = (CHAR*)buf;
//         wsaBuf.len = NetBTNameReq(name, xid, qtype, broadcast, buf,
//          sizeof(buf));
//         if (wsaBuf.len > 0)
//         {
//             DWORD bytesSent;
//
//             ret = WSASendTo(fd, &wsaBuf, 1, &bytesSent, 0,
//              (struct sockaddr*)&sin, sizeof(sin), NULL, NULL);
//             if (ret < 0 || bytesSent < wsaBuf.len)
//                 ret = -1;
//             else
//                 ret = 0;
//         }
//         else
//             ret = -1;
//     }
//     return ret;
// }
//
// typedef BOOL (*NetBTAnswerCallback)(void *data, WORD answerCount,
//  WORD answerIndex, PUCHAR rData, WORD rdLength);
//
// /* Waits on fd until GetTickCount() returns a value greater than or equal to
//  * waitUntil for a name service response.  If a name response matching xid
//  * is received, calls answerCallback once for each answer resource record in
//  * the response.  (The callback's answerCount will be the total number of
//  * answers to expect, and answerIndex will be the 0-based index that's being
//  * sent this time.)  Quits parsing if answerCallback returns FALSE.
//  * Returns NRC_GOODRET on timeout or a valid response received, something else
//  * on error.
//  */
// static UCHAR NetBTWaitForNameResponse(const NetBTAdapter *adapter, SOCKET fd,
//  DWORD waitUntil, NetBTAnswerCallback answerCallback, void *data)
// {
//     BOOL found = FALSE;
//     DWORD now;
//     UCHAR ret = NRC_GOODRET;
//
//     if (!adapter) return NRC_BADDR;
//     if (fd == INVALID_SOCKET) return NRC_BADDR;
//     if (!answerCallback) return NRC_BADDR;
//
//     while (!found && ret == NRC_GOODRET && (int)((now = GetTickCount()) - waitUntil) < 0)
//     {
//         DWORD msToWait = waitUntil - now;
//         struct fd_set fds;
//         struct timeval timeout = { msToWait / 1000, msToWait % 1000 };
//         int r;
//
//         FD_ZERO(&fds);
//         FD_SET(fd, &fds);
//         r = select(fd + 1, &fds, NULL, NULL, &timeout);
//         if (r < 0)
//             ret = NRC_SYSTEM;
//         else if (r == 1)
//         {
//             /* FIXME: magic #, is this always enough? */
//             UCHAR buffer[256];
//             int fromsize;
//             struct sockaddr_in fromaddr;
//             WORD respXID, flags, queryCount, answerCount;
//             WSABUF wsaBuf = { sizeof(buffer), (CHAR*)buffer };
//             DWORD bytesReceived, recvFlags = 0;
//
//             fromsize = sizeof(fromaddr);
//             r = WSARecvFrom(fd, &wsaBuf, 1, &bytesReceived, &recvFlags,
//              (struct sockaddr*)&fromaddr, &fromsize, NULL, NULL);
//             if(r < 0)
//             {
//                 ret = NRC_SYSTEM;
//                 break;
//             }
//
//             if (bytesReceived < NBNS_HEADER_SIZE)
//                 continue;
//
//             respXID = NBR_GETWORD(buffer);
//             if (adapter->nameQueryXID != respXID)
//                 continue;
//
//             flags = NBR_GETWORD(buffer + 2);
//             queryCount = NBR_GETWORD(buffer + 4);
//             answerCount = NBR_GETWORD(buffer + 6);
//
//             /* a reply shouldn't contain a query, ignore bad packet */
//             if (queryCount > 0)
//                 continue;
//
//             if ((flags & NBNS_RESPONSE_AND_OPCODE) == NBNS_RESPONSE_AND_QUERY)
//             {
//                 if ((flags & NBNS_REPLYCODE) != 0)
//                     ret = NRC_NAMERR;
//                 else if ((flags & NBNS_REPLYCODE) == 0 && answerCount > 0)
//                 {
//                     PUCHAR ptr = buffer + NBNS_HEADER_SIZE;
//                     BOOL shouldContinue = TRUE;
//                     WORD answerIndex = 0;
//
//                     found = TRUE;
//                     /* decode one answer at a time */
//                     while (ret == NRC_GOODRET && answerIndex < answerCount &&
//                      ptr - buffer < bytesReceived && shouldContinue)
//                     {
//                         WORD rLen;
//
//                         /* scan past name */
//                         for (; ptr[0] && ptr - buffer < bytesReceived; )
//                             ptr += ptr[0] + 1;
//                         ptr++;
//                         ptr += 2; /* scan past type */
//                         if (ptr - buffer < bytesReceived && ret == NRC_GOODRET
//                          && NBR_GETWORD(ptr) == NBNS_CLASS_INTERNET)
//                             ptr += sizeof(WORD);
//                         else
//                             ret = NRC_SYSTEM; /* parse error */
//                         ptr += sizeof(DWORD); /* TTL */
//                         rLen = NBR_GETWORD(ptr);
//                         rLen = min(rLen, bytesReceived - (ptr - buffer));
//                         ptr += sizeof(WORD);
//                         shouldContinue = answerCallback(data, answerCount,
//                          answerIndex, ptr, rLen);
//                         ptr += rLen;
//                         answerIndex++;
//                     }
//                 }
//             }
//         }
//     }
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// typedef struct _NetBTNameQueryData {
//     NBNameCacheEntry *cacheEntry;
//     UCHAR ret;
// } NetBTNameQueryData;
//
// /* Name query callback function for NetBTWaitForNameResponse, creates a cache
//  * entry on the first answer, adds each address as it's called again (as long
//  * as there's space).  If there's an error that should be propagated as the
//  * NetBIOS error, modifies queryData's ret member to the proper return code.
//  */
// static BOOL NetBTFindNameAnswerCallback(void *pVoid, WORD answerCount,
//  WORD answerIndex, PUCHAR rData, WORD rLen)
// {
//     NetBTNameQueryData *queryData = pVoid;
//     BOOL ret;
//
//     if (queryData)
//     {
//         if (queryData->cacheEntry == NULL)
//         {
//             queryData->cacheEntry = HeapAlloc(GetProcessHeap(), 0,
//              FIELD_OFFSET(NBNameCacheEntry, addresses[answerCount]));
//             if (queryData->cacheEntry)
//                 queryData->cacheEntry->numAddresses = 0;
//             else
//                 queryData->ret = NRC_OSRESNOTAV;
//         }
//         if (rLen == 6 && queryData->cacheEntry &&
//          queryData->cacheEntry->numAddresses < answerCount)
//         {
//             queryData->cacheEntry->addresses[queryData->cacheEntry->
//              numAddresses++] = *(const DWORD *)(rData + 2);
//             ret = queryData->cacheEntry->numAddresses < answerCount;
//         }
//         else
//             ret = FALSE;
//     }
//     else
//         ret = FALSE;
//     return ret;
// }
//
// /* Workhorse NetBT name lookup function.  Sends a name lookup query for
//  * ncb->ncb_callname to sendTo, as a broadcast if broadcast is TRUE, using
//  * adapter->nameQueryXID as the transaction ID.  Waits up to timeout
//  * milliseconds, and retries up to maxQueries times, waiting for a reply.
//  * If a valid response is received, stores the looked up addresses as a
//  * NBNameCacheEntry in *cacheEntry.
//  * Returns NRC_GOODRET on success, though this may not mean the name was
//  * resolved--check whether *cacheEntry is NULL.
//  */
// static UCHAR NetBTNameWaitLoop(const NetBTAdapter *adapter, SOCKET fd, const NCB *ncb,
//  DWORD sendTo, BOOL broadcast, DWORD timeout, DWORD maxQueries,
//  NBNameCacheEntry **cacheEntry)
// {
//     unsigned int queries;
//     NetBTNameQueryData queryData;
//
//     if (!adapter) return NRC_BADDR;
//     if (fd == INVALID_SOCKET) return NRC_BADDR;
//     if (!ncb) return NRC_BADDR;
//     if (!cacheEntry) return NRC_BADDR;
//
//     queryData.cacheEntry = NULL;
//     queryData.ret = NRC_GOODRET;
//     for (queries = 0; queryData.cacheEntry == NULL && queries < maxQueries;
//      queries++)
//     {
//         if (!NCB_CANCELLED(ncb))
//         {
//             int r = NetBTSendNameQuery(fd, ncb->ncb_callname,
//              adapter->nameQueryXID, NBNS_TYPE_NB, sendTo, broadcast);
//
//             if (r == 0)
//                 queryData.ret = NetBTWaitForNameResponse(adapter, fd,
//                  GetTickCount() + timeout, NetBTFindNameAnswerCallback,
//                  &queryData);
//             else
//                 queryData.ret = NRC_SYSTEM;
//         }
//         else
//             queryData.ret = NRC_CMDCAN;
//     }
//     if (queryData.cacheEntry)
//     {
//         memcpy(queryData.cacheEntry->name, ncb->ncb_callname, NCBNAMSZ);
//         memcpy(queryData.cacheEntry->nbname, ncb->ncb_callname, NCBNAMSZ);
//     }
//     *cacheEntry = queryData.cacheEntry;
//     return queryData.ret;
// }
//
// /* Attempts to add cacheEntry to the name cache in *nameCache; if *nameCache
//  * has not yet been created, creates it, using gCacheTimeout as the cache
//  * entry timeout.  If memory allocation fails, or if NBNameCacheAddEntry fails,
//  * frees cacheEntry.
//  * Returns NRC_GOODRET on success, and something else on failure.
//  */
// static UCHAR NetBTStoreCacheEntry(struct NBNameCache **nameCache,
//  NBNameCacheEntry *cacheEntry)
// {
//     UCHAR ret;
//
//     if (!nameCache) return NRC_BADDR;
//     if (!cacheEntry) return NRC_BADDR;
//
//     if (!*nameCache)
//         *nameCache = NBNameCacheCreate(GetProcessHeap(), gCacheTimeout);
//     if (*nameCache)
//         ret = NBNameCacheAddEntry(*nameCache, cacheEntry)
//          ?  NRC_GOODRET : NRC_OSRESNOTAV;
//     else
//     {
//         HeapFree(GetProcessHeap(), 0, cacheEntry);
//         ret = NRC_OSRESNOTAV;
//     }
//     return ret;
// }
//
// /* Attempts to resolve name using inet_addr(), then gethostbyname() if
//  * gEnableDNS is TRUE, if the suffix byte is either <00> or <20>.  If the name
//  * can be looked up, returns 0 and stores the looked up addresses as a
//  * NBNameCacheEntry in *cacheEntry.
//  * Returns NRC_GOODRET on success, though this may not mean the name was
//  * resolved--check whether *cacheEntry is NULL.  Returns something else on
//  * error.
//  */
// static UCHAR NetBTinetResolve(const UCHAR name[NCBNAMSZ],
//  NBNameCacheEntry **cacheEntry)
// {
//     UCHAR ret = NRC_GOODRET;
//
//     TRACE("name %s, cacheEntry %p\n", name, cacheEntry);
//
//     if (!name) return NRC_BADDR;
//     if (!cacheEntry) return NRC_BADDR;
//
//     if (isalnum(name[0]) && (name[NCBNAMSZ - 1] == 0 ||
//      name[NCBNAMSZ - 1] == 0x20))
//     {
//         CHAR toLookup[NCBNAMSZ];
//         unsigned int i;
//
//         for (i = 0; i < NCBNAMSZ - 1 && name[i] && name[i] != ' '; i++)
//             toLookup[i] = name[i];
//         toLookup[i] = '\0';
//
//         if (isdigit(toLookup[0]))
//         {
//             unsigned long addr = inet_addr(toLookup);
//
//             if (addr != INADDR_NONE)
//             {
//                 *cacheEntry = HeapAlloc(GetProcessHeap(), 0,
//                  FIELD_OFFSET(NBNameCacheEntry, addresses[1]));
//                 if (*cacheEntry)
//                 {
//                     memcpy((*cacheEntry)->name, name, NCBNAMSZ);
//                     memset((*cacheEntry)->nbname, 0, NCBNAMSZ);
//                     (*cacheEntry)->nbname[0] = '*';
//                     (*cacheEntry)->numAddresses = 1;
//                     (*cacheEntry)->addresses[0] = addr;
//                 }
//                 else
//                     ret = NRC_OSRESNOTAV;
//             }
//         }
//         if (gEnableDNS && ret == NRC_GOODRET && !*cacheEntry)
//         {
//             struct hostent *host;
//
//             if ((host = gethostbyname(toLookup)) != NULL)
//             {
//                 for (i = 0; ret == NRC_GOODRET && host->h_addr_list &&
//                  host->h_addr_list[i]; i++)
//                     ;
//                 if (host->h_addr_list && host->h_addr_list[0])
//                 {
//                     *cacheEntry = HeapAlloc(GetProcessHeap(), 0,
//                      FIELD_OFFSET(NBNameCacheEntry, addresses[i]));
//                     if (*cacheEntry)
//                     {
//                         memcpy((*cacheEntry)->name, name, NCBNAMSZ);
//                         memset((*cacheEntry)->nbname, 0, NCBNAMSZ);
//                         (*cacheEntry)->nbname[0] = '*';
//                         (*cacheEntry)->numAddresses = i;
//                         for (i = 0; i < (*cacheEntry)->numAddresses; i++)
//                             (*cacheEntry)->addresses[i] =
//                              *(DWORD*)host->h_addr_list[i];
//                     }
//                     else
//                         ret = NRC_OSRESNOTAV;
//                 }
//             }
//         }
//     }
//
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// /* Looks up the name in ncb->ncb_callname, first in the name caches (global
//  * and this adapter's), then using gethostbyname(), next by WINS if configured,
//  * and finally using broadcast NetBT name resolution.  In NBT parlance, this
//  * makes this an "H-node".  Stores an entry in the appropriate name cache for a
//  * found node, and returns it as *cacheEntry.
//  * Assumes data, ncb, and cacheEntry are not NULL.
//  * Returns NRC_GOODRET on success--which doesn't mean the name was resolved,
//  * just that all name lookup operations completed successfully--and something
//  * else on failure.  *cacheEntry will be NULL if the name was not found.
//  */
// static UCHAR NetBTInternalFindName(NetBTAdapter *adapter, PNCB ncb,
//  const NBNameCacheEntry **cacheEntry)
// {
//     UCHAR ret = NRC_GOODRET;
//
//     TRACE("adapter %p, ncb %p, cacheEntry %p\n", adapter, ncb, cacheEntry);
//
//     if (!cacheEntry) return NRC_BADDR;
//     *cacheEntry = NULL;
//
//     if (!adapter) return NRC_BADDR;
//     if (!ncb) return NRC_BADDR;
//
//     if (ncb->ncb_callname[0] == '*')
//         ret = NRC_NOWILD;
//     else
//     {
//         *cacheEntry = NBNameCacheFindEntry(gNameCache, ncb->ncb_callname);
//         if (!*cacheEntry)
//             *cacheEntry = NBNameCacheFindEntry(adapter->nameCache,
//              ncb->ncb_callname);
//         if (!*cacheEntry)
//         {
//             NBNameCacheEntry *newEntry = NULL;
//
//             ret = NetBTinetResolve(ncb->ncb_callname, &newEntry);
//             if (ret == NRC_GOODRET && newEntry)
//             {
//                 ret = NetBTStoreCacheEntry(&gNameCache, newEntry);
//                 if (ret != NRC_GOODRET)
//                     newEntry = NULL;
//             }
//             else
//             {
//                 SOCKET fd = WSASocketA(PF_INET, SOCK_DGRAM, IPPROTO_UDP, NULL,
//                  0, WSA_FLAG_OVERLAPPED);
//
//                 if(fd == INVALID_SOCKET)
//                     ret = NRC_OSRESNOTAV;
//                 else
//                 {
//                     int winsNdx;
//
//                     adapter->nameQueryXID++;
//                     for (winsNdx = 0; ret == NRC_GOODRET && *cacheEntry == NULL
//                      && winsNdx < gNumWINSServers; winsNdx++)
//                         ret = NetBTNameWaitLoop(adapter, fd, ncb,
//                          gWINSServers[winsNdx], FALSE, gWINSQueryTimeout,
//                          gWINSQueries, &newEntry);
//                     if (ret == NRC_GOODRET && newEntry)
//                     {
//                         ret = NetBTStoreCacheEntry(&gNameCache, newEntry);
//                         if (ret != NRC_GOODRET)
//                             newEntry = NULL;
//                     }
//                     if (ret == NRC_GOODRET && *cacheEntry == NULL)
//                     {
//                         DWORD bcastAddr =
//                          adapter->ipr.dwAddr & adapter->ipr.dwMask;
//
//                         if (adapter->ipr.dwBCastAddr)
//                             bcastAddr |= ~adapter->ipr.dwMask;
//                         ret = NetBTNameWaitLoop(adapter, fd, ncb, bcastAddr,
//                          TRUE, gBCastQueryTimeout, gBCastQueries, &newEntry);
//                         if (ret == NRC_GOODRET && newEntry)
//                         {
//                             ret = NetBTStoreCacheEntry(&adapter->nameCache,
//                              newEntry);
//                             if (ret != NRC_GOODRET)
//                                 newEntry = NULL;
//                         }
//                     }
//                     closesocket(fd);
//                 }
//             }
//             *cacheEntry = newEntry;
//         }
//     }
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// typedef struct _NetBTNodeQueryData
// {
//     BOOL gotResponse;
//     PADAPTER_STATUS astat;
//     WORD astatLen;
// } NetBTNodeQueryData;
//
// /* Callback function for NetBTAstatRemote, parses the rData for the node
//  * status and name list of the remote node.  Always returns FALSE, since
//  * there's never more than one answer we care about in a node status response.
//  */
// static BOOL NetBTNodeStatusAnswerCallback(void *pVoid, WORD answerCount,
//  WORD answerIndex, PUCHAR rData, WORD rLen)
// {
//     NetBTNodeQueryData *data = pVoid;
//
//     if (data && !data->gotResponse && rData && rLen >= 1)
//     {
//         /* num names is first byte; each name is NCBNAMSZ + 2 bytes */
//         if (rLen >= rData[0] * (NCBNAMSZ + 2))
//         {
//             WORD i;
//             PUCHAR src;
//             PNAME_BUFFER dst;
//
//             data->gotResponse = TRUE;
//             data->astat->name_count = rData[0];
//             for (i = 0, src = rData + 1,
//              dst = (PNAME_BUFFER)((PUCHAR)data->astat +
//               sizeof(ADAPTER_STATUS));
//              i < data->astat->name_count && src - rData < rLen &&
//              (PUCHAR)dst - (PUCHAR)data->astat < data->astatLen;
//              i++, dst++, src += NCBNAMSZ + 2)
//             {
//                 UCHAR flags = *(src + NCBNAMSZ);
//
//                 memcpy(dst->name, src, NCBNAMSZ);
//                 /* we won't actually see a registering name in the returned
//                  * response.  It's useful to see if no other flags are set; if
//                  * none are, then the name is registered. */
//                 dst->name_flags = REGISTERING;
//                 if (flags & 0x80)
//                     dst->name_flags |= GROUP_NAME;
//                 if (flags & 0x10)
//                     dst->name_flags |= DEREGISTERED;
//                 if (flags & 0x08)
//                     dst->name_flags |= DUPLICATE;
//                 if (dst->name_flags == REGISTERING)
//                     dst->name_flags = REGISTERED;
//             }
//             /* arbitrarily set HW type to Ethernet */
//             data->astat->adapter_type = 0xfe;
//             if (src - rData < rLen)
//                 memcpy(data->astat->adapter_address, src,
//                  min(rLen - (src - rData), 6));
//         }
//     }
//     return FALSE;
// }
//
// /* This uses the WINS timeout and query values, as they're the
//  * UCAST_REQ_RETRY_TIMEOUT and UCAST_REQ_RETRY_COUNT according to the RFCs.
//  */
// static UCHAR NetBTAstatRemote(NetBTAdapter *adapter, PNCB ncb)
// {
//     UCHAR ret = NRC_GOODRET;
//     const NBNameCacheEntry *cacheEntry = NULL;
//
//     TRACE("adapter %p, NCB %p\n", adapter, ncb);
//
//     if (!adapter) return NRC_BADDR;
//     if (!ncb) return NRC_INVADDRESS;
//
//     ret = NetBTInternalFindName(adapter, ncb, &cacheEntry);
//     if (ret == NRC_GOODRET && cacheEntry)
//     {
//         if (cacheEntry->numAddresses > 0)
//         {
//             SOCKET fd = WSASocketA(PF_INET, SOCK_DGRAM, IPPROTO_UDP, NULL, 0,
//              WSA_FLAG_OVERLAPPED);
//
//             if(fd == INVALID_SOCKET)
//                 ret = NRC_OSRESNOTAV;
//             else
//             {
//                 NetBTNodeQueryData queryData;
//                 DWORD queries;
//                 PADAPTER_STATUS astat = (PADAPTER_STATUS)ncb->ncb_buffer;
//
//                 adapter->nameQueryXID++;
//                 astat->name_count = 0;
//                 queryData.gotResponse = FALSE;
//                 queryData.astat = astat;
//                 queryData.astatLen = ncb->ncb_length;
//                 for (queries = 0; !queryData.gotResponse &&
//                  queries < gWINSQueries; queries++)
//                 {
//                     if (!NCB_CANCELLED(ncb))
//                     {
//                         int r = NetBTSendNameQuery(fd, ncb->ncb_callname,
//                          adapter->nameQueryXID, NBNS_TYPE_NBSTAT,
//                          cacheEntry->addresses[0], FALSE);
//
//                         if (r == 0)
//                             ret = NetBTWaitForNameResponse(adapter, fd,
//                              GetTickCount() + gWINSQueryTimeout,
//                              NetBTNodeStatusAnswerCallback, &queryData);
//                         else
//                             ret = NRC_SYSTEM;
//                     }
//                     else
//                         ret = NRC_CMDCAN;
//                 }
//                 closesocket(fd);
//             }
//         }
//         else
//             ret = NRC_CMDTMO;
//     }
//     else if (ret == NRC_CMDCAN)
//         ; /* do nothing, we were cancelled */
//     else
//         ret = NRC_CMDTMO;
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static UCHAR NetBTAstat(void *adapt, PNCB ncb)
// {
//     NetBTAdapter *adapter = adapt;
//     UCHAR ret;
//
//     TRACE("adapt %p, NCB %p\n", adapt, ncb);
//
//     if (!adapter) return NRC_ENVNOTDEF;
//     if (!ncb) return NRC_INVADDRESS;
//     if (!ncb->ncb_buffer) return NRC_BADDR;
//     if (ncb->ncb_length < sizeof(ADAPTER_STATUS)) return NRC_BUFLEN;
//
//     if (ncb->ncb_callname[0] == '*')
//     {
//         DWORD physAddrLen;
//         MIB_IFROW ifRow;
//         PADAPTER_STATUS astat = (PADAPTER_STATUS)ncb->ncb_buffer;
//   
//         memset(astat, 0, sizeof(ADAPTER_STATUS));
//         astat->rev_major = 3;
//         ifRow.dwIndex = adapter->ipr.dwIndex;
//         if (GetIfEntry(&ifRow) != NO_ERROR)
//             ret = NRC_BRIDGE;
//         else
//         {
//             physAddrLen = min(ifRow.dwPhysAddrLen, 6);
//             if (physAddrLen > 0)
//                 memcpy(astat->adapter_address, ifRow.bPhysAddr, physAddrLen);
//             /* doubt anyone cares, but why not.. */
//             if (ifRow.dwType == MIB_IF_TYPE_TOKENRING)
//                 astat->adapter_type = 0xff;
//             else
//                 astat->adapter_type = 0xfe; /* for Ethernet */
//             astat->max_sess_pkt_size = 0xffff;
//             astat->xmit_success = adapter->xmit_success;
//             astat->recv_success = adapter->recv_success;
//             ret = NRC_GOODRET;
//         }
//     }
//     else
//         ret = NetBTAstatRemote(adapter, ncb);
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static UCHAR NetBTFindName(void *adapt, PNCB ncb)
// {
//     NetBTAdapter *adapter = adapt;
//     UCHAR ret;
//     const NBNameCacheEntry *cacheEntry = NULL;
//     PFIND_NAME_HEADER foundName;
//
//     TRACE("adapt %p, NCB %p\n", adapt, ncb);
//
//     if (!adapter) return NRC_ENVNOTDEF;
//     if (!ncb) return NRC_INVADDRESS;
//     if (!ncb->ncb_buffer) return NRC_BADDR;
//     if (ncb->ncb_length < sizeof(FIND_NAME_HEADER)) return NRC_BUFLEN;
//
//     foundName = (PFIND_NAME_HEADER)ncb->ncb_buffer;
//     memset(foundName, 0, sizeof(FIND_NAME_HEADER));
//
//     ret = NetBTInternalFindName(adapter, ncb, &cacheEntry);
//     if (ret == NRC_GOODRET)
//     {
//         if (cacheEntry)
//         {
//             DWORD spaceFor = min((ncb->ncb_length - sizeof(FIND_NAME_HEADER)) /
//              sizeof(FIND_NAME_BUFFER), cacheEntry->numAddresses);
//             DWORD ndx;
//
//             for (ndx = 0; ndx < spaceFor; ndx++)
//             {
//                 PFIND_NAME_BUFFER findNameBuffer;
//
//                 findNameBuffer =
//                  (PFIND_NAME_BUFFER)((PUCHAR)foundName +
//                  sizeof(FIND_NAME_HEADER) + foundName->node_count *
//                  sizeof(FIND_NAME_BUFFER));
//                 memset(findNameBuffer->destination_addr, 0, 2);
//                 memcpy(findNameBuffer->destination_addr + 2,
//                  &adapter->ipr.dwAddr, sizeof(DWORD));
//                 memset(findNameBuffer->source_addr, 0, 2);
//                 memcpy(findNameBuffer->source_addr + 2,
//                  &cacheEntry->addresses[ndx], sizeof(DWORD));
//                 foundName->node_count++;
//             }
//             if (spaceFor < cacheEntry->numAddresses)
//                 ret = NRC_BUFLEN;
//         }
//         else
//             ret = NRC_CMDTMO;
//     }
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static UCHAR NetBTSessionReq(SOCKET fd, const UCHAR *calledName,
//  const UCHAR *callingName)
// {
//     UCHAR buffer[NBSS_HDRSIZE + MAX_DOMAIN_NAME_LEN * 2], ret;
//     int r;
//     unsigned int len = 0;
//     DWORD bytesSent, bytesReceived, recvFlags = 0;
//     WSABUF wsaBuf;
//
//     buffer[0] = NBSS_REQ;
//     buffer[1] = 0;
//
//     len += NetBTNameEncode(calledName, &buffer[NBSS_HDRSIZE]);
//     len += NetBTNameEncode(callingName, &buffer[NBSS_HDRSIZE + len]);
//
//     NBR_ADDWORD(&buffer[2], len);
//
//     wsaBuf.len = len + NBSS_HDRSIZE;
//     wsaBuf.buf = (char*)buffer;
//
//     r = WSASend(fd, &wsaBuf, 1, &bytesSent, 0, NULL, NULL);
//     if(r < 0 || bytesSent < len + NBSS_HDRSIZE)
//     {
//         ERR("send failed\n");
//         return NRC_SABORT;
//     }
//
//     /* I've already set the recv timeout on this socket (if it supports it), so
//      * just block.  Hopefully we'll always receive the session acknowledgement
//      * within one timeout.
//      */
//     wsaBuf.len = NBSS_HDRSIZE + 1;
//     r = WSARecv(fd, &wsaBuf, 1, &bytesReceived, &recvFlags, NULL, NULL);
//     if (r < 0 || bytesReceived < NBSS_HDRSIZE)
//         ret = NRC_SABORT;
//     else if (buffer[0] == NBSS_NACK)
//     {
//         if (r == NBSS_HDRSIZE + 1)
//         {
//             switch (buffer[NBSS_HDRSIZE])
//             {
//                 case NBSS_ERR_INSUFFICIENT_RESOURCES:
//                     ret = NRC_REMTFUL;
//                     break;
//                 default:
//                     ret = NRC_NOCALL;
//             }
//         }
//         else
//             ret = NRC_NOCALL;
//     }
//     else if (buffer[0] == NBSS_RETARGET)
//     {
//         FIXME("Got a session retarget, can't deal\n");
//         ret = NRC_NOCALL;
//     }
//     else if (buffer[0] == NBSS_ACK)
//         ret = NRC_GOODRET;
//     else
//         ret = NRC_SYSTEM;
//
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static UCHAR NetBTCall(void *adapt, PNCB ncb, void **sess)
// {
//     NetBTAdapter *adapter = adapt;
//     UCHAR ret;
//     const NBNameCacheEntry *cacheEntry = NULL;
//
//     TRACE("adapt %p, ncb %p\n", adapt, ncb);
//
//     if (!adapter) return NRC_ENVNOTDEF;
//     if (!ncb) return NRC_INVADDRESS;
//     if (!sess) return NRC_BADDR;
//
//     ret = NetBTInternalFindName(adapter, ncb, &cacheEntry);
//     if (ret == NRC_GOODRET)
//     {
//         if (cacheEntry && cacheEntry->numAddresses > 0)
//         {
//             SOCKET fd;
//
//             fd = WSASocketA(PF_INET, SOCK_STREAM, IPPROTO_TCP, NULL, 0,
//              WSA_FLAG_OVERLAPPED);
//             if (fd != INVALID_SOCKET)
//             {
//                 DWORD timeout;
//                 struct sockaddr_in sin;
//
//                 if (ncb->ncb_rto > 0)
//                 {
//                     timeout = ncb->ncb_rto * 500;
//                     setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, (char*)&timeout,
//                      sizeof(timeout));
//                 }
//                 if (ncb->ncb_sto > 0)
//                 {
//                     timeout = ncb->ncb_sto * 500;
//                     setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, (char*)&timeout,
//                      sizeof(timeout));
//                 }
//
//                 memset(&sin, 0, sizeof(sin));
//                 memcpy(&sin.sin_addr, &cacheEntry->addresses[0],
//                  sizeof(sin.sin_addr));
//                 sin.sin_family = AF_INET;
//                 sin.sin_port   = htons(PORT_NBSS);
//                 /* FIXME: use nonblocking mode for the socket, check the
//                  * cancel flag periodically
//                  */
//                 if (connect(fd, (struct sockaddr *)&sin, sizeof(sin))
//                  == SOCKET_ERROR)
//                     ret = NRC_CMDTMO;
//                 else
//                 {
//                     static const UCHAR fakedCalledName[] = "*SMBSERVER";
//                     const UCHAR *calledParty = cacheEntry->nbname[0] == '*'
//                      ? fakedCalledName : cacheEntry->nbname;
//
//                     ret = NetBTSessionReq(fd, calledParty, ncb->ncb_name);
//                     if (ret != NRC_GOODRET && calledParty[0] == '*')
//                     {
//                         FIXME("NBT session to \"*SMBSERVER\" refused,\n");
//                         FIXME("should try finding name using ASTAT\n");
//                     }
//                 }
//                 if (ret != NRC_GOODRET)
//                     closesocket(fd);
//                 else
//                 {
//                     NetBTSession *session = HeapAlloc(
//                      GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(NetBTSession));
//
//                     if (session)
//                     {
//                         session->fd = fd;
//                         InitializeCriticalSection(&session->cs);
//                         session->cs.DebugInfo->Spare[0] = (DWORD_PTR)(__FILE__ ": NetBTSession.cs");
//                         *sess = session;
//                     }
//                     else
//                     {
//                         ret = NRC_OSRESNOTAV;
//                         closesocket(fd);
//                     }
//                 }
//             }
//             else
//                 ret = NRC_OSRESNOTAV;
//         }
//         else
//             ret = NRC_NAMERR;
//     }
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// /* Notice that I don't protect against multiple thread access to NetBTSend.
//  * This is because I don't update any data in the adapter, and I only make a
//  * single call to WSASend, which I assume to act atomically (not interleaving
//  * data from other threads).
//  * I don't lock, because I only depend on the fd being valid, and this won't be
//  * true until a session setup is completed.
//  */
// static UCHAR NetBTSend(void *adapt, void *sess, PNCB ncb)
// {
//     NetBTAdapter *adapter = adapt;
//     NetBTSession *session = sess;
//     UCHAR buffer[NBSS_HDRSIZE], ret;
//     int r;
//     WSABUF wsaBufs[2];
//     DWORD bytesSent;
//
//     TRACE("adapt %p, session %p, NCB %p\n", adapt, session, ncb);
//
//     if (!adapter) return NRC_ENVNOTDEF;
//     if (!ncb) return NRC_INVADDRESS;
//     if (!ncb->ncb_buffer) return NRC_BADDR;
//     if (!session) return NRC_SNUMOUT;
//     if (session->fd == INVALID_SOCKET) return NRC_SNUMOUT;
//
//     buffer[0] = NBSS_MSG;
//     buffer[1] = 0;
//     NBR_ADDWORD(&buffer[2], ncb->ncb_length);
//
//     wsaBufs[0].len = NBSS_HDRSIZE;
//     wsaBufs[0].buf = (char*)buffer;
//     wsaBufs[1].len = ncb->ncb_length;
//     wsaBufs[1].buf = (char*)ncb->ncb_buffer;
//
//     r = WSASend(session->fd, wsaBufs, sizeof(wsaBufs) / sizeof(wsaBufs[0]),
//      &bytesSent, 0, NULL, NULL);
//     if (r == SOCKET_ERROR)
//     {
//         NetBIOSHangupSession(ncb);
//         ret = NRC_SABORT;
//     }
//     else if (bytesSent < NBSS_HDRSIZE + ncb->ncb_length)
//     {
//         FIXME("Only sent %d bytes (of %d), hanging up session\n", bytesSent,
//          NBSS_HDRSIZE + ncb->ncb_length);
//         NetBIOSHangupSession(ncb);
//         ret = NRC_SABORT;
//     }
//     else
//     {
//         ret = NRC_GOODRET;
//         adapter->xmit_success++;
//     }
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static UCHAR NetBTRecv(void *adapt, void *sess, PNCB ncb)
// {
//     NetBTAdapter *adapter = adapt;
//     NetBTSession *session = sess;
//     UCHAR buffer[NBSS_HDRSIZE], ret;
//     int r;
//     WSABUF wsaBufs[2];
//     DWORD bufferCount, bytesReceived, flags;
//
//     TRACE("adapt %p, session %p, NCB %p\n", adapt, session, ncb);
//
//     if (!adapter) return NRC_ENVNOTDEF;
//     if (!ncb) return NRC_BADDR;
//     if (!ncb->ncb_buffer) return NRC_BADDR;
//     if (!session) return NRC_SNUMOUT;
//     if (session->fd == INVALID_SOCKET) return NRC_SNUMOUT;
//
//     EnterCriticalSection(&session->cs);
//     bufferCount = 0;
//     if (session->bytesPending == 0)
//     {
//         bufferCount++;
//         wsaBufs[0].len = NBSS_HDRSIZE;
//         wsaBufs[0].buf = (char*)buffer;
//     }
//     wsaBufs[bufferCount].len = ncb->ncb_length;
//     wsaBufs[bufferCount].buf = (char*)ncb->ncb_buffer;
//     bufferCount++;
//
//     flags = 0;
//     /* FIXME: should poll a bit so I can check the cancel flag */
//     r = WSARecv(session->fd, wsaBufs, bufferCount, &bytesReceived, &flags,
//      NULL, NULL);
//     if (r == SOCKET_ERROR && WSAGetLastError() != WSAEWOULDBLOCK)
//     {
//         LeaveCriticalSection(&session->cs);
//         ERR("Receive error, WSAGetLastError() returns %d\n", WSAGetLastError());
//         NetBIOSHangupSession(ncb);
//         ret = NRC_SABORT;
//     }
//     else if (NCB_CANCELLED(ncb))
//     {
//         LeaveCriticalSection(&session->cs);
//         ret = NRC_CMDCAN;
//     }
//     else
//     {
//         if (bufferCount == 2)
//         {
//             if (buffer[0] == NBSS_KEEPALIVE)
//             {
//                 LeaveCriticalSection(&session->cs);
//                 FIXME("Oops, received a session keepalive and lost my place\n");
//                 /* need to read another session header until we get a session
//                  * message header. */
//                 NetBIOSHangupSession(ncb);
//                 ret = NRC_SABORT;
//                 goto error;
//             }
//             else if (buffer[0] != NBSS_MSG)
//             {
//                 LeaveCriticalSection(&session->cs);
//                 FIXME("Received unexpected session msg type %d\n", buffer[0]);
//                 NetBIOSHangupSession(ncb);
//                 ret = NRC_SABORT;
//                 goto error;
//             }
//             else
//             {
//                 if (buffer[1] & NBSS_EXTENSION)
//                 {
//                     LeaveCriticalSection(&session->cs);
//                     FIXME("Received a message that's too long for my taste\n");
//                     NetBIOSHangupSession(ncb);
//                     ret = NRC_SABORT;
//                     goto error;
//                 }
//                 else
//                 {
//                     session->bytesPending = NBSS_HDRSIZE
//                      + NBR_GETWORD(&buffer[2]) - bytesReceived;
//                     ncb->ncb_length = bytesReceived - NBSS_HDRSIZE;
//                     LeaveCriticalSection(&session->cs);
//                 }
//             }
//         }
//         else
//         {
//             if (bytesReceived < session->bytesPending)
//                 session->bytesPending -= bytesReceived;
//             else
//                 session->bytesPending = 0;
//             LeaveCriticalSection(&session->cs);
//             ncb->ncb_length = bytesReceived;
//         }
//         if (session->bytesPending > 0)
//             ret = NRC_INCOMP;
//         else
//         {
//             ret = NRC_GOODRET;
//             adapter->recv_success++;
//         }
//     }
// error:
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static UCHAR NetBTHangup(void *adapt, void *sess)
// {
//     NetBTSession *session = sess;
//
//     TRACE("adapt %p, session %p\n", adapt, session);
//
//     if (!session) return NRC_SNUMOUT;
//
//     /* I don't lock the session, because NetBTRecv knows not to decrement
//      * past 0, so if a receive completes after this it should still deal.
//      */
//     closesocket(session->fd);
//     session->fd = INVALID_SOCKET;
//     session->bytesPending = 0;
//     session->cs.DebugInfo->Spare[0] = 0;
//     DeleteCriticalSection(&session->cs);
//     HeapFree(GetProcessHeap(), 0, session);
//
//     return NRC_GOODRET;
// }
//
// static void NetBTCleanupAdapter(void *adapt)
// {
//     TRACE("adapt %p\n", adapt);
//     if (adapt)
//     {
//         NetBTAdapter *adapter = adapt;
//
//         if (adapter->nameCache)
//             NBNameCacheDestroy(adapter->nameCache);
//         HeapFree(GetProcessHeap(), 0, adapt);
//     }
// }
//
// static void NetBTCleanup(void)
// {
//     TRACE("\n");
//     if (gNameCache)
//     {
//         NBNameCacheDestroy(gNameCache);
//         gNameCache = NULL;
//     }
// }
//
// static UCHAR NetBTRegisterAdapter(const MIB_IPADDRROW *ipRow)
// {
//     UCHAR ret;
//     NetBTAdapter *adapter;
//
//     if (!ipRow) return NRC_BADDR;
//
//     adapter = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(NetBTAdapter));
//     if (adapter)
//     {
//         adapter->ipr = *ipRow;
//         if (!NetBIOSRegisterAdapter(gTransportID, ipRow->dwIndex, adapter))
//         {
//             NetBTCleanupAdapter(adapter);
//             ret = NRC_SYSTEM;
//         }
//         else
//             ret = NRC_GOODRET;
//     }
//     else
//         ret = NRC_OSRESNOTAV;
//     return ret;
// }
//
// /* Callback for NetBIOS adapter enumeration.  Assumes closure is a pointer to
//  * a MIB_IPADDRTABLE containing all the IP adapters needed to be added to the
//  * NetBIOS adapter table.  For each callback, checks if the passed-in adapt
//  * has an entry in the table; if so, this adapter was enumerated previously,
//  * and it's enabled.  As a flag, the table's dwAddr entry is changed to
//  * INADDR_LOOPBACK, since this is an invalid address for a NetBT adapter.
//  * The NetBTEnum function will add any remaining adapters from the
//  * MIB_IPADDRTABLE to the NetBIOS adapter table.
//  */
// static BOOL NetBTEnumCallback(UCHAR totalLANAs, UCHAR lanaIndex,
//  ULONG transport, const NetBIOSAdapterImpl *data, void *closure)
// {
//     BOOL ret;
//     PMIB_IPADDRTABLE table = closure;
//
//     if (table && data)
//     {
//         DWORD ndx;
//
//         ret = FALSE;
//         for (ndx = 0; !ret && ndx < table->dwNumEntries; ndx++)
//         {
//             const NetBTAdapter *adapter = data->data;
//
//             if (table->table[ndx].dwIndex == adapter->ipr.dwIndex)
//             {
//                 NetBIOSEnableAdapter(data->lana);
//                 table->table[ndx].dwAddr = INADDR_LOOPBACK;
//                 ret = TRUE;
//             }
//         }
//     }
//     else
//         ret = FALSE;
//     return ret;
// }
//
// /* Enumerates adapters by:
//  * - retrieving the IP address table for the local machine
//  * - eliminating loopback addresses from the table
//  * - eliminating redundant addresses, that is, multiple addresses on the same
//  *   subnet
//  * Calls NetBIOSEnumAdapters, passing the resulting table as the callback
//  * data.  The callback reenables each adapter that's already in the NetBIOS
//  * table.  After NetBIOSEnumAdapters returns, this function adds any remaining
//  * adapters to the NetBIOS table.
//  */
// static UCHAR NetBTEnum(void)
// {
//     UCHAR ret;
//     DWORD size = 0;
//
//     TRACE("\n");
//
//     if (GetIpAddrTable(NULL, &size, FALSE) == ERROR_INSUFFICIENT_BUFFER)
//     {
//         PMIB_IPADDRTABLE ipAddrs, coalesceTable = NULL;
//         DWORD numIPAddrs = (size - sizeof(MIB_IPADDRTABLE)) /
//          sizeof(MIB_IPADDRROW) + 1;
//
//         ipAddrs = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size);
//         if (ipAddrs)
//             coalesceTable = HeapAlloc(GetProcessHeap(),
//              HEAP_ZERO_MEMORY, sizeof(MIB_IPADDRTABLE) +
//              (min(numIPAddrs, MAX_LANA + 1) - 1) * sizeof(MIB_IPADDRROW));
//         if (ipAddrs && coalesceTable)
//         {
//             if (GetIpAddrTable(ipAddrs, &size, FALSE) == ERROR_SUCCESS)
//             {
//                 DWORD ndx;
//
//                 for (ndx = 0; ndx < ipAddrs->dwNumEntries; ndx++)
//                 {
//                     if ((ipAddrs->table[ndx].dwAddr &
//                      ipAddrs->table[ndx].dwMask) !=
//                      htonl((INADDR_LOOPBACK & IN_CLASSA_NET)))
//                     {
//                         BOOL newNetwork = TRUE;
//                         DWORD innerIndex;
//
//                         /* make sure we don't have more than one entry
//                          * for a subnet */
//                         for (innerIndex = 0; newNetwork &&
//                          innerIndex < coalesceTable->dwNumEntries; innerIndex++)
//                             if ((ipAddrs->table[ndx].dwAddr &
//                              ipAddrs->table[ndx].dwMask) ==
//                              (coalesceTable->table[innerIndex].dwAddr
//                              & coalesceTable->table[innerIndex].dwMask))
//                                 newNetwork = FALSE;
//
//                         if (newNetwork)
//                             memcpy(&coalesceTable->table[
//                              coalesceTable->dwNumEntries++],
//                              &ipAddrs->table[ndx], sizeof(MIB_IPADDRROW));
//                     }
//                 }
//
//                 NetBIOSEnumAdapters(gTransportID, NetBTEnumCallback,
//                  coalesceTable);
//                 ret = NRC_GOODRET;
//                 for (ndx = 0; ret == NRC_GOODRET &&
//                  ndx < coalesceTable->dwNumEntries; ndx++)
//                     if (coalesceTable->table[ndx].dwAddr != INADDR_LOOPBACK)
//                         ret = NetBTRegisterAdapter(&coalesceTable->table[ndx]);
//             }
//             else
//                 ret = NRC_SYSTEM;
//             HeapFree(GetProcessHeap(), 0, ipAddrs);
//             HeapFree(GetProcessHeap(), 0, coalesceTable);
//         }
//         else
//             ret = NRC_OSRESNOTAV;
//     }
//     else
//         ret = NRC_SYSTEM;
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static const WCHAR VxD_MSTCPW[] = { 'S','Y','S','T','E','M','\\','C','u','r',
//  'r','e','n','t','C','o','n','t','r','o','l','S','e','t','\\','S','e','r','v',
//  'i','c','e','s','\\','V','x','D','\\','M','S','T','C','P','\0' };
// static const WCHAR NetBT_ParametersW[] = { 'S','Y','S','T','E','M','\\','C','u',
//  'r','r','e','n','t','C','o','n','t','r','o','l','S','e','t','\\','S','e','r',
//  'v','i','c','e','s','\\','N','e','t','B','T','\\','P','a','r','a','m','e','t',
//  'e','r','s','\0' };
// static const WCHAR EnableDNSW[] = { 'E','n','a','b','l','e','D','N','S','\0' };
// static const WCHAR BcastNameQueryCountW[] = { 'B','c','a','s','t','N','a','m',
//  'e','Q','u','e','r','y','C','o','u','n','t','\0' };
// static const WCHAR BcastNameQueryTimeoutW[] = { 'B','c','a','s','t','N','a','m',
//  'e','Q','u','e','r','y','T','i','m','e','o','u','t','\0' };
// static const WCHAR NameSrvQueryCountW[] = { 'N','a','m','e','S','r','v',
//  'Q','u','e','r','y','C','o','u','n','t','\0' };
// static const WCHAR NameSrvQueryTimeoutW[] = { 'N','a','m','e','S','r','v',
//  'Q','u','e','r','y','T','i','m','e','o','u','t','\0' };
// static const WCHAR ScopeIDW[] = { 'S','c','o','p','e','I','D','\0' };
// static const WCHAR CacheTimeoutW[] = { 'C','a','c','h','e','T','i','m','e','o',
//  'u','t','\0' };
// static const WCHAR Config_NetworkW[] = { 'S','o','f','t','w','a','r','e','\\',
//                                          'W','i','n','e','\\','N','e','t','w','o','r','k','\0' };
//
// /* Initializes global variables and registers the NetBT transport */
// void NetBTInit(void)
// {
//     HKEY hKey;
//     NetBIOSTransport transport;
//     LONG ret;
//
//     TRACE("\n");
//
//     gEnableDNS = TRUE;
//     gBCastQueries = BCAST_QUERIES;
//     gBCastQueryTimeout = BCAST_QUERY_TIMEOUT;
//     gWINSQueries = WINS_QUERIES;
//     gWINSQueryTimeout = WINS_QUERY_TIMEOUT;
//     gNumWINSServers = 0;
//     memset(gWINSServers, 0, sizeof(gWINSServers));
//     gScopeID[0] = '\0';
//     gCacheTimeout = CACHE_TIMEOUT;
//
//     /* Try to open the Win9x NetBT configuration key */
//     ret = RegOpenKeyExW(HKEY_LOCAL_MACHINE, VxD_MSTCPW, 0, KEY_READ, &hKey);
//     /* If that fails, try the WinNT NetBT configuration key */
//     if (ret != ERROR_SUCCESS)
//         ret = RegOpenKeyExW(HKEY_LOCAL_MACHINE, NetBT_ParametersW, 0, KEY_READ,
//          &hKey);
//     if (ret == ERROR_SUCCESS)
//     {
//         DWORD dword, size;
//
//         size = sizeof(dword);
//         if (RegQueryValueExW(hKey, EnableDNSW, NULL, NULL,
//          (LPBYTE)&dword, &size) == ERROR_SUCCESS)
//             gEnableDNS = dword;
//         size = sizeof(dword);
//         if (RegQueryValueExW(hKey, BcastNameQueryCountW, NULL, NULL,
//          (LPBYTE)&dword, &size) == ERROR_SUCCESS && dword >= MIN_QUERIES
//          && dword <= MAX_QUERIES)
//             gBCastQueries = dword;
//         size = sizeof(dword);
//         if (RegQueryValueExW(hKey, BcastNameQueryTimeoutW, NULL, NULL,
//          (LPBYTE)&dword, &size) == ERROR_SUCCESS && dword >= MIN_QUERY_TIMEOUT)
//             gBCastQueryTimeout = dword;
//         size = sizeof(dword);
//         if (RegQueryValueExW(hKey, NameSrvQueryCountW, NULL, NULL,
//          (LPBYTE)&dword, &size) == ERROR_SUCCESS && dword >= MIN_QUERIES
//          && dword <= MAX_QUERIES)
//             gWINSQueries = dword;
//         size = sizeof(dword);
//         if (RegQueryValueExW(hKey, NameSrvQueryTimeoutW, NULL, NULL,
//          (LPBYTE)&dword, &size) == ERROR_SUCCESS && dword >= MIN_QUERY_TIMEOUT)
//             gWINSQueryTimeout = dword;
//         size = sizeof(gScopeID) - 1;
//         if (RegQueryValueExW(hKey, ScopeIDW, NULL, NULL, (LPBYTE)gScopeID + 1, &size)
//          == ERROR_SUCCESS)
//         {
//             /* convert into L2-encoded version, suitable for use by
//                NetBTNameEncode */
//             char *ptr, *lenPtr;
//
//             for (ptr = gScopeID + 1, lenPtr = gScopeID; ptr - gScopeID < sizeof(gScopeID) && *ptr; ++ptr)
//             {
//                 if (*ptr == '.')
//                 {
//                     lenPtr = ptr;
//                     *lenPtr = 0;
//                 }
//                 else
//                 {
//                     ++*lenPtr;
//                 }
//             }
//         }
//         if (RegQueryValueExW(hKey, CacheTimeoutW, NULL, NULL,
//          (LPBYTE)&dword, &size) == ERROR_SUCCESS && dword >= MIN_CACHE_TIMEOUT)
//             gCacheTimeout = dword;
//         RegCloseKey(hKey);
//     }
//     /* WINE-specific NetBT registry settings.  Because our adapter naming is
//      * different than MS', we can't do per-adapter WINS configuration in the
//      * same place.  Just do a global WINS configuration instead.
//      */
//     /* @@ Wine registry key: HKCU\Software\Wine\Network */
//     if (RegOpenKeyW(HKEY_CURRENT_USER, Config_NetworkW, &hKey) == ERROR_SUCCESS)
//     {
//         static const char *nsValueNames[] = { "WinsServer", "BackupWinsServer" };
//         char nsString[16];
//         DWORD size, ndx;
//
//         for (ndx = 0; ndx < sizeof(nsValueNames) / sizeof(nsValueNames[0]);
//          ndx++)
//         {
//             size = sizeof(nsString) / sizeof(char);
//             if (RegQueryValueExA(hKey, nsValueNames[ndx], NULL, NULL,
//              (LPBYTE)nsString, &size) == ERROR_SUCCESS)
//             {
//                 unsigned long addr = inet_addr(nsString);
//
//                 if (addr != INADDR_NONE && gNumWINSServers < MAX_WINS_SERVERS)
//                     gWINSServers[gNumWINSServers++] = addr;
//             }
//         }
//         RegCloseKey(hKey);
//     }
//
//     transport.enumerate      = NetBTEnum;
//     transport.astat          = NetBTAstat;
//     transport.findName       = NetBTFindName;
//     transport.call           = NetBTCall;
//     transport.send           = NetBTSend;
//     transport.recv           = NetBTRecv;
//     transport.hangup         = NetBTHangup;
//     transport.cleanupAdapter = NetBTCleanupAdapter;
//     transport.cleanup        = NetBTCleanup;
//     memcpy(&gTransportID, TRANSPORT_NBT, sizeof(ULONG));
//     NetBIOSRegisterTransport(gTransportID, &transport);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
