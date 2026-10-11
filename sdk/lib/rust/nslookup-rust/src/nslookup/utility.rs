use crate::nslookup::nslookup::RequestID;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static TypeA: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"A\0") };

pub static TypeAny: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"ANY\0") };

pub static TypeCNAME: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"CNAME\0") };

pub static TypeMX: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"MX\0") };

pub static TypeNS: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"NS\0") };

pub static TypePTR: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"PTR\0") };

pub static TypeSOA: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"SOA\0") };

pub static TypeSRV: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"SRV\0") };

pub static ClassIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"IN\0") };

pub static ClassAny: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"ANY\0") };

pub static OpcodeQuery: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"QUERY\0") };

pub static OpcodeIQuery: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"IQUERY\0") };

pub static OpcodeStatus: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"STATUS\0") };

pub static OpcodeReserved: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"RESERVED\0") };

pub static RCodeNOERROR: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"NOERROR\0") };

pub static RCodeFORMERR: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"FORMERR\0") };

pub static RCodeFAILURE: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"FAILURE\0") };

pub static RCodeNXDOMAIN: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"NXDOMAIN\0") };

pub static RCodeNOTIMP: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"NOTIMP\0") };

pub static RCodeREFUSED: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"REFUSED\0") };

pub static RCodeReserved: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"RESERVED\0") };
#[no_mangle]
pub unsafe extern "C" fn SendRequest(mut pInBuffer: PCHAR, mut InBufferLength: ULONG, mut pOutBuffer: PCHAR, mut pOutBufferLength: PULONG) -> BOOL {
    let mut j: ::core::ffi::c_int = 0;
    let mut RequestID_local: USHORT = 0;
    let mut ResponseID: USHORT = 0;
    let mut bWait: BOOL = 0;
    let mut s: SOCKET = 0;
    let mut RecAddr: SOCKADDR_IN = core::mem::zeroed();
    let mut RecAddr2: SOCKADDR_IN = core::mem::zeroed();
    let mut SendAddr: SOCKADDR_IN = core::mem::zeroed();
    let mut SendAddrLen: ::core::ffi::c_int =
        ::core::mem::size_of::<SOCKADDR_IN>() as ::core::ffi::c_int;

    RtlZeroMemory!(
        core::ptr::addr_of_mut!(RecAddr) as *mut ::core::ffi::c_void,
        (core::mem::size_of::<SOCKADDR_IN>()) as usize,
    );
    RtlZeroMemory!(
        core::ptr::addr_of_mut!(RecAddr2) as *mut ::core::ffi::c_void,
        (core::mem::size_of::<SOCKADDR_IN>()) as usize,
    );
    RtlZeroMemory!(
        core::ptr::addr_of_mut!(SendAddr) as *mut ::core::ffi::c_void,
        (core::mem::size_of::<SOCKADDR_IN>()) as usize,
    );

    /* Pull the request ID from the buffer. */
    RequestID_local = ntohs(
        (*(pInBuffer.add(0 as usize) as PSHORT)
            .add(0 as usize)) as u16,
    );

    /* If D2 flags is enabled, then display D2 information. */
    if State.d2 != 0 {
        PrintD2(pInBuffer, InBufferLength);
    }

    /* Create the sockets for both send and receive. */
    s = socket(
        (AF_INET as i32),
        (SOCK_DGRAM as i32),
        IPPROTO_UDP,
    );

    if s == INVALID_SOCKET {
        return (FALSE) as BOOL;
    }

    /* Set up the structure to tell it where we are going. */
    RecAddr.sin_family = (AF_INET as i16);
    RecAddr.sin_port = htons(State.port);
    RecAddr.sin_addr.S_un.S_addr = inet_addr((State.DefaultServerAddress).as_ptr());

    /* Set up the structure to tell it what port to listen on. */
    RecAddr2.sin_family = (AF_INET as i16);
    RecAddr2.sin_port = htons(State.port);
    RecAddr2.sin_addr.S_un.S_addr = htonl(INADDR_ANY);

    /* Bind the receive socket. */
    bind(
        s,
        core::ptr::addr_of_mut!(RecAddr2) as *mut SOCKADDR,
        ::core::mem::size_of::<SOCKADDR_IN>() as ::core::ffi::c_int,
    );

    /* Send the datagram to the DNS server. */
    j = sendto(
        s,
        pInBuffer as *const ::core::ffi::c_char,
        InBufferLength as ::core::ffi::c_int,
        0,
        core::ptr::addr_of_mut!(RecAddr) as *mut SOCKADDR,
        ::core::mem::size_of::<SOCKADDR_IN>() as ::core::ffi::c_int,
    );
    if j == SOCKET_ERROR {
        match WSAGetLastError() {
            WSANOTINITIALISED => {
                _tprintf(_T!(b"sendto() failed with WSANOTINITIALIZED\n\0".as_ptr() as *const i8));
            }
            WSAENETDOWN => {
                _tprintf(_T!(b"sendto() failed with WSAENETDOWN\n\0".as_ptr() as *const i8));
            }
            WSAEACCES => {
                _tprintf(_T!(b"sendto() failed with WSAEACCES\n\0".as_ptr() as *const i8));
            }
            WSAEINVAL => {
                _tprintf(_T!(b"sendto() failed with WSAEINVAL\n\0".as_ptr() as *const i8));
            }
            WSAEINTR => {
                _tprintf(_T!(b"sendto() failed with WSAEINTR\n\0".as_ptr() as *const i8));
            }
            WSAEINPROGRESS => {
                _tprintf(_T!(b"sendto() failed with WSAEINPROGRESS\n\0".as_ptr() as *const i8));
            }
            WSAEFAULT => {
                _tprintf(_T!(b"sendto() failed with WSAEFAULT\n\0".as_ptr() as *const i8));
            }
            WSAENETRESET => {
                _tprintf(_T!(b"sendto() failed with WSAENETRESET\n\0".as_ptr() as *const i8));
            }
            WSAENOBUFS => {
                _tprintf(_T!(b"sendto() failed with WSAENOBUFS\n\0".as_ptr() as *const i8));
            }
            WSAENOTCONN => {
                _tprintf(_T!(b"sendto() failed with WSAENOTCONN\n\0".as_ptr() as *const i8));
            }
            WSAENOTSOCK => {
                _tprintf(_T!(b"sendto() failed with WSAENOTSOCK\n\0".as_ptr() as *const i8));
            }
            WSAEOPNOTSUPP => {
                _tprintf(_T!(b"sendto() failed with WSAEOPNOTSUPP\n\0".as_ptr() as *const i8));
            }
            WSAESHUTDOWN => {
                _tprintf(_T!(b"sendto() failed with WSAESHUTDOWN\n\0".as_ptr() as *const i8));
            }
            WSAEWOULDBLOCK => {
                _tprintf(_T!(b"sendto() failed with WSAEWOULDBLOCK\n\0".as_ptr() as *const i8));
            }
            WSAEMSGSIZE => {
                _tprintf(_T!(b"sendto() failed with WSAEMSGSIZE\n\0".as_ptr() as *const i8));
            }
            WSAEHOSTUNREACH => {
                _tprintf(_T!(b"sendto() failed with WSAEHOSTUNREACH\n\0".as_ptr() as *const i8));
            }
            WSAECONNABORTED => {
                _tprintf(_T!(b"sendto() failed with WSAECONNABORTED\n\0".as_ptr() as *const i8));
            }
            WSAECONNRESET => {
                _tprintf(_T!(b"sendto() failed with WSAECONNRESET\n\0".as_ptr() as *const i8));
            }
            WSAEADDRNOTAVAIL => {
                _tprintf(_T!(b"sendto() failed with WSAEADDRNOTAVAIL\n\0".as_ptr() as *const i8));
            }
            WSAEAFNOSUPPORT => {
                _tprintf(_T!(b"sendto() failed with WSAEAFNOSUPPORT\n\0".as_ptr() as *const i8));
            }
            WSAEDESTADDRREQ => {
                _tprintf(_T!(b"sendto() failed with WSAEDESTADDRREQ\n\0".as_ptr() as *const i8));
            }
            WSAENETUNREACH => {
                _tprintf(_T!(b"sendto() failed with WSAENETUNREACH\n\0".as_ptr() as *const i8));
            }
            WSAETIMEDOUT => {
                _tprintf(_T!(b"sendto() failed with WSAETIMEDOUT\n\0".as_ptr() as *const i8));
            }
            _ => {
                _tprintf(_T!(b"sendto() failed with unknown error\n\0".as_ptr() as *const i8));
            }
        }

        closesocket(s);
        return (FALSE) as BOOL;
    }

    bWait = (TRUE as i32);

    while bWait != 0 {
        /* Wait for the DNS reply. */
        j = recvfrom(
            s,
            pOutBuffer as *mut ::core::ffi::c_char,
            *pOutBufferLength as ::core::ffi::c_int,
            0,
            core::ptr::addr_of_mut!(SendAddr) as *mut sockaddr,
            core::ptr::addr_of_mut!(SendAddrLen),
        );
        if j == SOCKET_ERROR {
            match WSAGetLastError() {
                WSANOTINITIALISED => {
                    _tprintf(_T!(b"recvfrom() failed with WSANOTINITIALIZED\n\0".as_ptr() as *const i8));
                }
                WSAENETDOWN => {
                    _tprintf(_T!(b"recvfrom() failed with WSAENETDOWN\n\0".as_ptr() as *const i8));
                }
                WSAEACCES => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEACCES\n\0".as_ptr() as *const i8));
                }
                WSAEINVAL => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEINVAL\n\0".as_ptr() as *const i8));
                }
                WSAEINTR => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEINTR\n\0".as_ptr() as *const i8));
                }
                WSAEINPROGRESS => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEINPROGRESS\n\0".as_ptr() as *const i8));
                }
                WSAEFAULT => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEFAULT\n\0".as_ptr() as *const i8));
                }
                WSAENETRESET => {
                    _tprintf(_T!(b"recvfrom() failed with WSAENETRESET\n\0".as_ptr() as *const i8));
                }
                WSAENOBUFS => {
                    _tprintf(_T!(b"recvfrom() failed with WSAENOBUFS\n\0".as_ptr() as *const i8));
                }
                WSAENOTCONN => {
                    _tprintf(_T!(b"recvfrom() failed with WSAENOTCONN\n\0".as_ptr() as *const i8));
                }
                WSAENOTSOCK => {
                    _tprintf(_T!(b"recvfrom() failed with WSAENOTSOCK\n\0".as_ptr() as *const i8));
                }
                WSAEOPNOTSUPP => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEOPNOTSUPP\n\0".as_ptr() as *const i8));
                }
                WSAESHUTDOWN => {
                    _tprintf(_T!(b"recvfrom() failed with WSAESHUTDOWN\n\0".as_ptr() as *const i8));
                }
                WSAEWOULDBLOCK => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEWOULDBLOCK\n\0".as_ptr() as *const i8));
                }
                WSAEMSGSIZE => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEMSGSIZE\n\0".as_ptr() as *const i8));
                }
                WSAEHOSTUNREACH => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEHOSTUNREACH\n\0".as_ptr() as *const i8));
                }
                WSAECONNABORTED => {
                    _tprintf(_T!(b"recvfrom() failed with WSAECONNABORTED\n\0".as_ptr() as *const i8));
                }
                WSAECONNRESET => {
                    _tprintf(_T!(b"recvfrom() failed with WSAECONNRESET\n\0".as_ptr() as *const i8));
                }
                WSAEADDRNOTAVAIL => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEADDRNOTAVAIL\n\0".as_ptr() as *const i8));
                }
                WSAEAFNOSUPPORT => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEAFNOSUPPORT\n\0".as_ptr() as *const i8));
                }
                WSAEDESTADDRREQ => {
                    _tprintf(_T!(b"recvfrom() failed with WSAEDESTADDRREQ\n\0".as_ptr() as *const i8));
                }
                WSAENETUNREACH => {
                    _tprintf(_T!(b"recvfrom() failed with WSAENETUNREACH\n\0".as_ptr() as *const i8));
                }
                WSAETIMEDOUT => {
                    _tprintf(_T!(b"recvfrom() failed with WSAETIMEDOUT\n\0".as_ptr() as *const i8));
                }
                _ => {
                    _tprintf(_T!(b"recvfrom() failed with unknown error\n\0".as_ptr() as *const i8));
                }
            }

            closesocket(s);
            return (FALSE) as BOOL;
        }

        ResponseID = ntohs(
            (*(pOutBuffer.add(0 as usize) as PSHORT)
                .add(0 as usize)) as u16,
        );

        if ResponseID as ::core::ffi::c_int == RequestID_local as ::core::ffi::c_int {
            bWait = (FALSE as i32);
        }
    }

    /* We don't need the sockets anymore. */
    closesocket(s);

    /* If debug information then display debug information. */
    if State.debug != 0 {
        PrintDebug(pOutBuffer, j as DWORD);
    }

    /* Return the real output buffer length. */
    *pOutBufferLength = j as ULONG;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ReverseIP(pIP: PCHAR, pReturn: PCHAR) {
    let mut i: i32;
    let mut j: i32;
    let mut k: i32 = 0;

    assert!(pReturn != pIP);

    j = strlen(pIP as *const i8) as i32 - 1;
    i = j;

    /* We have A.B.C.D
       We will turn this into D.C.B.A and stick it in pReturn */

    /* A */
    while i > 0 {
        if '.' as i32 == *pIP.offset(i as isize) as i32 {
            break;
        }
        i -= 1;
    }

    if j != i {
        strncpy(
            (pReturn.offset(k as isize) as *mut i8).cast(),
            pIP.offset((i + 1) as isize) as *const u8,
            (j - i) as usize,
        );
    }
    k += j - i;

    *pReturn.offset(k as isize) = '.' as i32 as CHAR;
    k += 1;

    i -= 1;
    j = i;

    /* B */
    while i > 0 {
        if '.' as i32 == *pIP.offset(i as isize) as i32 {
            break;
        }
        i -= 1;
    }

    if j != i {
        strncpy(
            (pReturn.offset(k as isize) as *mut i8).cast(),
            pIP.offset((i + 1) as isize) as *const u8,
            (j - i) as usize,
        );
    }
    k += j - i;

    *pReturn.offset(k as isize) = '.' as i32 as CHAR;
    k += 1;

    i -= 1;
    j = i;

    /* C */
    while i > 0 {
        if '.' as i32 == *pIP.offset(i as isize) as i32 {
            break;
        }
        i -= 1;
    }

    if j != i {
        strncpy(
            (pReturn.offset(k as isize) as *mut i8).cast(),
            pIP.offset((i + 1) as isize) as *const u8,
            (j - i) as usize,
        );
    }
    k += j - i;

    *pReturn.offset(k as isize) = '.' as i32 as CHAR;
    k += 1;

    i -= 1;
    j = i;

    /* D */
    while i > 0 {
        i -= 1;
    }

    strncpy(
        (pReturn.offset(k as isize) as *mut i8).cast(),
        pIP.offset(i as isize) as *const u8,
        ((j - i) + 1) as usize,
    );
    k += (j - i) + 1;

    *pReturn.offset(k as isize) = '\0' as i32 as CHAR;
}
#[no_mangle]
pub unsafe extern "C" fn IsValidIP(pInput: PCHAR) -> BOOL {
    let mut i: i32 = 0;
    let mut l: i32 = 0;
    let mut b: i32 = 0;
    let mut c: i32 = 1;

    /* Max length of an IP, e.g. 255.255.255.255, is 15 characters. */
    l = strlen(pInput as *const i8) as i32;
    if l > 15 {
        return (FALSE) as BOOL;
    }

    /* 'b' is the count of the current segment. It gets reset after seeing a
       '.'. */
    while i < l {
        if '.' as i32 == *pInput.offset(i as isize) as i32 {
            if b == 0 {
                return (FALSE) as BOOL;
            }
            if b > 3 {
                return (FALSE) as BOOL;
            }

            b = 0;
            c += 1;
        } else {
            b += 1;

            if (*pInput.offset(i as isize) as i32 ) < '0' as i32
                || (*pInput.offset(i as isize) as i32) > '9' as i32
            {
                return (FALSE) as BOOL;
            }
        }
        i += 1;
    }

    if b > 3 {
        return (FALSE) as BOOL;
    }

    /* 'c' is the number of segments seen. If it's less than 4, then it's not
       a valid IP. */
    if c < 4 {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ExtractName(mut pBuffer: PCHAR, mut pOutput: PCHAR, mut Offset: USHORT, mut Limit: UCHAR) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut d: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut k: ::core::ffi::c_int = 0;
    let mut l: ::core::ffi::c_int = 0;
    let mut m: ::core::ffi::c_int = 0;
    i = Offset as ::core::ffi::c_int;
    d = Limit as ::core::ffi::c_int;
    if 0 == Limit as ::core::ffi::c_int {
        d = 255;
    }
    while d > 0 {
        l = *pBuffer.offset(i as isize) as ::core::ffi::c_int & 0xFF;
        i += 1;
        if m == 0 {
            c += 1;
        }
        if 0xC0 == l {
            if m == 0 {
                c += 1;
            }
            m = 1;
            d += 255 - Limit as ::core::ffi::c_int;
            i = *pBuffer.offset(i as isize) as ::core::ffi::c_int;
        } else {
            j = 0;
            while j < l {
                *pOutput.offset(k as isize) = *pBuffer.offset(i as isize);
                i += 1;
                if m == 0 {
                    c += 1;
                }
                k += 1;
                d -= 1;
                j += 1;
            }
            d -= 1;
            if *pBuffer.offset(i as isize) == 0 || d < 1 {
                break;
            }
            *pOutput.offset(k as isize) = '.' as i32 as CHAR;
            k += 1;
        }
    }
    if m == 0 {
        if Limit == 0 {
            c += 1;
        }
    }
    *pOutput.offset(k as isize) = '\0' as i32 as CHAR;
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn ExtractIP(pBuffer: PCHAR, pOutput: PCHAR, Offset: USHORT) -> ::core::ffi::c_int {
    let mut c: i32 = 0;
    let mut l: i32 = 0;
    let mut i: i32 = 0;
    let mut v: i32 = 0;

    i = Offset as i32;

    v = *pBuffer.offset(i as isize) as UCHAR as i32;
    l += 1;
    i += 1;

    sprintf(
        pOutput.offset(c as isize) as *mut i8,
        b"%d.\0".as_ptr() as *const i8,
        v,
    );
    c += strlen(pOutput.offset(c as isize) as *mut CHAR) as i32;

    v = *pBuffer.offset(i as isize) as UCHAR as i32;
    l += 1;
    i += 1;

    sprintf(
        pOutput.offset(c as isize) as *mut i8,
        b"%d.\0".as_ptr() as *const i8,
        v,
    );
    c += strlen(pOutput.offset(c as isize) as *mut CHAR) as i32;

    v = *pBuffer.offset(i as isize) as UCHAR as i32;
    l += 1;
    i += 1;

    sprintf(
        pOutput.offset(c as isize) as *mut i8,
        b"%d.\0".as_ptr() as *const i8,
        v,
    );
    c += strlen(pOutput.offset(c as isize) as *mut CHAR) as i32;

    v = *pBuffer.offset(i as isize) as UCHAR as i32;
    l += 1;
    i += 1;

    sprintf(
        pOutput.offset(c as isize) as *mut i8,
        b"%d\0".as_ptr() as *const i8,
        v,
    );
    c += strlen(pOutput.offset(c as isize) as *mut CHAR) as i32;

    *pOutput.offset(c as isize) = b'\0' as CHAR;

    return l;
}
#[no_mangle]
pub unsafe extern "C" fn PrintD2(mut pBuffer: PCHAR, mut BufferLength: DWORD) {
    let mut RequestID_local: USHORT = 0;
    let mut Header1: UCHAR = 0;
    let mut Header2: UCHAR = 0;
    let mut NumQuestions: USHORT = 0;
    let mut NumAnswers: USHORT = 0;
    let mut NumAuthority: USHORT = 0;
    let mut NumAdditional: USHORT = 0;
    let mut Type: USHORT = 0;
    let mut Class: USHORT = 0;
    let mut pName: [CHAR; 255] = [0; 255];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

    RequestID_local = ntohs(
        *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize) as u_short,
    ) as USHORT;
    i += 2 as ::core::ffi::c_int;

    Header1 = *pBuffer.offset(i as isize) as UCHAR;
    i += 1 as ::core::ffi::c_int;

    Header2 = *pBuffer.offset(i as isize) as UCHAR;
    i += 1 as ::core::ffi::c_int;

    NumQuestions = ntohs(
        *(pBuffer.offset(i as isize) as PSHORT).offset(0 as isize) as u_short,
    ) as USHORT;
    i += 2 as ::core::ffi::c_int;

    NumAnswers = ntohs(
        *(pBuffer.offset(i as isize) as PSHORT).offset(0 as isize) as u_short,
    ) as USHORT;
    i += 2 as ::core::ffi::c_int;

    NumAuthority = ntohs(
        *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize) as u_short,
    ) as USHORT;
    i += 2 as ::core::ffi::c_int;

    NumAdditional = ntohs(
        *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize) as u_short,
    ) as USHORT;
    i += 2 as ::core::ffi::c_int;

    _tprintf(b"------------\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"SendRequest(), len %d\n\0".as_ptr() as *const ::core::ffi::c_char, BufferLength as ::core::ffi::c_int);
    _tprintf(b"    HEADER:\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(
        b"        opcode = %s, id = %d, rcode = %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        OpcodeIDtoOpcodeName(
            ((Header1 as ::core::ffi::c_int & 0x78 as ::core::ffi::c_int)
                >> 3 as ::core::ffi::c_int) as UCHAR,
        ),
        RequestID_local as ::core::ffi::c_int,
        RCodeIDtoRCodeName(
            (Header2 as ::core::ffi::c_int & 0x0F as ::core::ffi::c_int) as UCHAR,
        ),
    );

    _tprintf(b"        header flags:  query\0".as_ptr() as *const ::core::ffi::c_char);
    if Header1 as ::core::ffi::c_int & 0x01 as ::core::ffi::c_int != 0 {
        _tprintf(b", want recursion\0".as_ptr() as *const ::core::ffi::c_char);
    }
    _tprintf(b"\n\0".as_ptr() as *const ::core::ffi::c_char);

    _tprintf(
        b"        questions = %d,  answers = %d,  authority records = %d,  additional = %d\n\n\0".as_ptr() as *const ::core::ffi::c_char,
        NumQuestions as ::core::ffi::c_int,
        NumAnswers as ::core::ffi::c_int,
        NumAuthority as ::core::ffi::c_int,
        NumAdditional as ::core::ffi::c_int,
    );

    if NumQuestions != 0 {
        _tprintf(b"    QUESTIONS:\n\0".as_ptr() as *const ::core::ffi::c_char);

        k = 0 as ::core::ffi::c_int;
        while k < NumQuestions as ::core::ffi::c_int {
            i += ExtractName(
                pBuffer,
                core::ptr::addr_of_mut!(pName) as PCHAR,
                i as USHORT,
                0 as UCHAR,
            );

            _tprintf(b"        %s\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(pName) as *mut CHAR);

            Type = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize) as u_short,
            ) as USHORT;
            i += 2 as ::core::ffi::c_int;

            Class = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize) as u_short,
            ) as USHORT;
            i += 2 as ::core::ffi::c_int;

            _tprintf(
                b", type = %s, class = %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                TypeIDtoTypeName(Type),
                ClassIDtoClassName(Class),
            );

            k += 1 as ::core::ffi::c_int;
        }
    }

    _tprintf(b"\n------------\n\0".as_ptr() as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn PrintDebug(pBuffer: PCHAR, BufferLength: DWORD) {
    let mut ResponseID: USHORT = 0;
    let mut Header1: UCHAR = 0;
    let mut Header2: UCHAR = 0;
    let mut NumQuestions: USHORT = 0;
    let mut NumAnswers: USHORT = 0;
    let mut NumAuthority: USHORT = 0;
    let mut NumAdditional: USHORT = 0;
    let mut Type: USHORT = 0;
    let mut Class: USHORT = 0;
    let mut TTL: ULONG = 0;
    let mut pName: [CHAR; 255] = [0; 255];
    let mut d: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut k: ::core::ffi::c_int = 0;

    ResponseID = ntohs(
        *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
    );
    i += 2;

    Header1 = *pBuffer.offset(i as isize) as UCHAR;
    i += 1;

    Header2 = *pBuffer.offset(i as isize) as UCHAR;
    i += 1;

    NumQuestions = ntohs(
        (*(pBuffer.offset(i as isize) as PSHORT).offset(0 as isize)) as u16,
    );
    i += 2;

    NumAnswers = ntohs(
        (*(pBuffer.offset(i as isize) as PSHORT).offset(0 as isize)) as u16,
    );
    i += 2;

    NumAuthority = ntohs(
        *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
    );
    i += 2;

    NumAdditional = ntohs(
        *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
    );
    i += 2;

    _tprintf(b"------------\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(b"Got answer (%d bytes):\n\0".as_ptr() as *const ::core::ffi::c_char, BufferLength as ::core::ffi::c_int);
    _tprintf(b"    HEADER:\n\0".as_ptr() as *const ::core::ffi::c_char);
    _tprintf(
        b"        opcode = %s, id = %d, rcode = %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        OpcodeIDtoOpcodeName(((Header1 & 0x78) >> 3) as UCHAR),
        ResponseID as ::core::ffi::c_int,
        RCodeIDtoRCodeName((Header2 & 0x0F) as UCHAR),
    );

    _tprintf(b"        header flags:  response\0".as_ptr() as *const ::core::ffi::c_char);
    if (Header1 & 0x01) != 0 {
        _tprintf(b", want recursion\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if (Header2 & 0x80) != 0 {
        _tprintf(b", recursion avail.\0".as_ptr() as *const ::core::ffi::c_char);
    }
    _tprintf(b"\n\0".as_ptr() as *const ::core::ffi::c_char);

    _tprintf(
        b"        questions = %d,  answers = %d,  authority records = %d,  additional = %d\n\n\0".as_ptr() as *const ::core::ffi::c_char,
        NumQuestions as ::core::ffi::c_int,
        NumAnswers as ::core::ffi::c_int,
        NumAuthority as ::core::ffi::c_int,
        NumAdditional as ::core::ffi::c_int,
    );

    if NumQuestions != 0 {
        _tprintf(b"    QUESTIONS:\n\0".as_ptr() as *const ::core::ffi::c_char);

        k = 0;
        while k < NumQuestions as ::core::ffi::c_int {
            i += ExtractName(
                pBuffer,
                core::ptr::addr_of_mut!(pName) as PCHAR,
                i as USHORT,
                0 as UCHAR,
            );

            _tprintf(b"        %s\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(pName) as *mut CHAR);

            Type = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
            );
            i += 2;

            Class = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
            );
            i += 2;

            _tprintf(
                b", type = %s, class = %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                TypeIDtoTypeName(Type),
                ClassIDtoClassName(Class),
            );

            k += 1;
        }
    }

    if NumAnswers != 0 {
        _tprintf(b"    ANSWERS:\n\0".as_ptr() as *const ::core::ffi::c_char);

        k = 0;
        while k < NumAnswers as ::core::ffi::c_int {
            _tprintf(b"    ->  \0".as_ptr() as *const ::core::ffi::c_char);

            /* Print out the name. */
            i += ExtractName(
                pBuffer,
                core::ptr::addr_of_mut!(pName) as PCHAR,
                i as USHORT,
                0 as UCHAR,
            );

            _tprintf(b"%s\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(pName) as *mut CHAR);

            /* Print out the type, class and data length. */
            Type = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
            );
            i += 2;

            Class = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
            );
            i += 2;

            TTL = ntohl(
                *(pBuffer.offset(i as isize) as PULONG).offset(0 as isize),
            );
            i += 4;

            d = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
            ) as ::core::ffi::c_int;
            i += 2;

            _tprintf(
                b"        type = %s, class = %s, dlen = %d\n\0".as_ptr() as *const ::core::ffi::c_char,
                TypeIDtoTypeName(Type),
                ClassIDtoClassName(Class),
                d,
            );

            /* Print out the answer. */
            if TYPE_A == (Type as ::core::ffi::c_int) as u32 {
                i += ExtractIP(pBuffer, core::ptr::addr_of_mut!(pName) as PCHAR, i as USHORT);

                _tprintf(
                    b"        internet address = %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    core::ptr::addr_of_mut!(pName) as *mut CHAR,
                );
            } else {
                i += ExtractName(
                    pBuffer,
                    core::ptr::addr_of_mut!(pName) as PCHAR,
                    i as USHORT,
                    d as UCHAR,
                );

                _tprintf(b"        name = %s\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(pName) as *mut CHAR);
            }

            _tprintf(b"        ttl = %d ()\n\0".as_ptr() as *const ::core::ffi::c_char, TTL as ::core::ffi::c_int);

            k += 1;
        }
    }

    if NumAuthority != 0 {
        _tprintf(b"    AUTHORITY RECORDS:\n\0".as_ptr() as *const ::core::ffi::c_char);

        k = 0;
        while k < NumAuthority as ::core::ffi::c_int {
            /* Print out the zone name. */
            i += ExtractName(
                pBuffer,
                core::ptr::addr_of_mut!(pName) as PCHAR,
                i as USHORT,
                0 as UCHAR,
            );

            _tprintf(b"    ->  %s\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(pName) as *mut CHAR);

            /* Print out the type, class, data length and TTL. */
            Type = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
            );
            i += 2;

            Class = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
            );
            i += 2;

            TTL = ntohl(
                *(pBuffer.offset(i as isize) as PULONG).offset(0 as isize),
            );
            i += 4;

            d = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
            ) as ::core::ffi::c_int;
            i += 2;

            _tprintf(
                b"        type = %s, class = %s, dlen = %d\n\0".as_ptr() as *const ::core::ffi::c_char,
                TypeIDtoTypeName(Type),
                ClassIDtoClassName(Class),
                d,
            );

            /* TODO: There might be more types? */
            if TYPE_NS == (Type as ::core::ffi::c_int) as u32 {
                /* Print out the NS. */
                i += ExtractName(
                    pBuffer,
                    core::ptr::addr_of_mut!(pName) as PCHAR,
                    i as USHORT,
                    d as UCHAR,
                );

                _tprintf(b"        nameserver = %s\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(pName) as *mut CHAR);

                _tprintf(b"        ttl = %d ()\n\0".as_ptr() as *const ::core::ffi::c_char, TTL as ::core::ffi::c_int);
            } else if TYPE_SOA == (Type as ::core::ffi::c_int) as u32 {
                _tprintf(b"        ttl = %d ()\n\0".as_ptr() as *const ::core::ffi::c_char, TTL as ::core::ffi::c_int);

                /* Print out the primary NS. */
                i += ExtractName(
                    pBuffer,
                    core::ptr::addr_of_mut!(pName) as PCHAR,
                    i as USHORT,
                    0 as UCHAR,
                );

                _tprintf(
                    b"        primary name server = %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    core::ptr::addr_of_mut!(pName) as *mut CHAR,
                );

                /* Print out the responsible mailbox. */
                i += ExtractName(
                    pBuffer,
                    core::ptr::addr_of_mut!(pName) as PCHAR,
                    i as USHORT,
                    0 as UCHAR,
                );

                _tprintf(
                    b"        responsible mail addr = %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    core::ptr::addr_of_mut!(pName) as *mut CHAR,
                );

                /* Print out the serial, refresh, retry, expire and default TTL. */
                _tprintf(b"        serial = ()\n\0".as_ptr() as *const ::core::ffi::c_char);
                _tprintf(b"        refresh = ()\n\0".as_ptr() as *const ::core::ffi::c_char);
                _tprintf(b"        retry = ()\n\0".as_ptr() as *const ::core::ffi::c_char);
                _tprintf(b"        expire = ()\n\0".as_ptr() as *const ::core::ffi::c_char);
                _tprintf(b"        default TTL = ()\n\0".as_ptr() as *const ::core::ffi::c_char);
                i += 20;
            }

            k += 1;
        }
    }

    if NumAdditional != 0 {
        _tprintf(b"    ADDITIONAL:\n\0".as_ptr() as *const ::core::ffi::c_char);

        k = 0;
        while k < NumAdditional as ::core::ffi::c_int {
            /* Print the name. */
            i += ExtractName(
                pBuffer,
                core::ptr::addr_of_mut!(pName) as PCHAR,
                i as USHORT,
                0 as UCHAR,
            );

            _tprintf(b"    ->  %s\n\0".as_ptr() as *const ::core::ffi::c_char, core::ptr::addr_of_mut!(pName) as *mut CHAR);

            /* Print out the type, class, data length and TTL. */
            Type = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
            );
            i += 2;

            Class = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
            );
            i += 2;

            TTL = ntohl(
                *(pBuffer.offset(i as isize) as PULONG).offset(0 as isize),
            );
            i += 4;

            d = ntohs(
                *(pBuffer.offset(i as isize) as PUSHORT).offset(0 as isize),
            ) as ::core::ffi::c_int;
            i += 2;

            _tprintf(
                b"        type = %s, class = %s, dlen = %d\n\0".as_ptr() as *const ::core::ffi::c_char,
                TypeIDtoTypeName(Type),
                ClassIDtoClassName(Class),
                d,
            );

            /* TODO: There might be more types? */
            if TYPE_A == (Type as ::core::ffi::c_int) as u32 {
                /* Print out the NS. */
                i += ExtractIP(pBuffer, core::ptr::addr_of_mut!(pName) as PCHAR, i as USHORT);

                _tprintf(
                    b"        internet address = %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    core::ptr::addr_of_mut!(pName) as *mut CHAR,
                );

                _tprintf(b"        ttl = %d ()\n\0".as_ptr() as *const ::core::ffi::c_char, TTL as ::core::ffi::c_int);
            }

            k += 1;
        }
    }

    _tprintf(b"\n------------\n\0".as_ptr() as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn OpcodeIDtoOpcodeName(mut Opcode: UCHAR) -> PCHAR {
    match ((Opcode as ::core::ffi::c_int) & 0x0F) as u32 {
        OPCODE_QUERY => {
            return OpcodeQuery.as_ptr() as PCHAR;
        }
        OPCODE_IQUERY => {
            return OpcodeIQuery.as_ptr() as PCHAR;
        }
        OPCODE_STATUS => {
            return OpcodeStatus.as_ptr() as PCHAR;
        }
        _ => {
            return OpcodeReserved.as_ptr() as PCHAR;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn RCodeIDtoRCodeName(RCode: UCHAR) -> PCHAR {
    match (RCode as ::core::ffi::c_int & 0x0F as ::core::ffi::c_int) as u32 {
        RCODE_NOERROR => {
            return RCodeNOERROR.as_ptr() as PCHAR;
        }
        RCODE_FORMERR => {
            return RCodeFORMERR.as_ptr() as PCHAR;
        }
        RCODE_FAILURE => {
            return RCodeFAILURE.as_ptr() as PCHAR;
        }
        RCODE_NXDOMAIN => {
            return RCodeNXDOMAIN.as_ptr() as PCHAR;
        }
        RCODE_NOTIMP => {
            return RCodeNOTIMP.as_ptr() as PCHAR;
        }
        RCODE_REFUSED => {
            return RCodeREFUSED.as_ptr() as PCHAR;
        }
        _ => {
            return RCodeReserved.as_ptr() as PCHAR;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn TypeIDtoTypeName(TypeID: USHORT) -> PCHAR {
    if TypeID == (TYPE_A as u16) {
        return TypeA.as_ptr() as PCHAR;
    }
    if TypeID == (TYPE_NS as u16) {
        return TypeNS.as_ptr() as PCHAR;
    }
    if TypeID == (TYPE_CNAME as u16) {
        return TypeCNAME.as_ptr() as PCHAR;
    }
    if TypeID == (TYPE_SOA as u16) {
        return TypeSOA.as_ptr() as PCHAR;
    }
    if TypeID == (TYPE_WKS as u16) {
        return TypeSRV.as_ptr() as PCHAR;
    }
    if TypeID == (TYPE_PTR as u16) {
        return TypePTR.as_ptr() as PCHAR;
    }
    if TypeID == (TYPE_MX as u16) {
        return TypeMX.as_ptr() as PCHAR;
    }
    if TypeID == (TYPE_ANY as u16) {
        return TypeAny.as_ptr() as PCHAR;
    }
    return b"Unknown\0" as *const u8 as *const ::core::ffi::c_char as PCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn TypeNametoTypeID(TypeName: PCHAR) -> USHORT {
    if strncmp(TypeName as *const ::core::ffi::c_char, TypeA.as_ptr(), strlen(TypeA.as_ptr())) == 0 { return TYPE_A as USHORT; }
    if strncmp(TypeName as *const ::core::ffi::c_char, TypeNS.as_ptr(), strlen(TypeNS.as_ptr())) == 0 { return TYPE_NS as USHORT; }
    if strncmp(TypeName as *const ::core::ffi::c_char, TypeCNAME.as_ptr(), strlen(TypeCNAME.as_ptr())) == 0 { return TYPE_CNAME as USHORT; }
    if strncmp(TypeName as *const ::core::ffi::c_char, TypeSOA.as_ptr(), strlen(TypeSOA.as_ptr())) == 0 { return TYPE_SOA as USHORT; }
    if strncmp(TypeName as *const ::core::ffi::c_char, TypeSRV.as_ptr(), strlen(TypeSRV.as_ptr())) == 0 { return TYPE_WKS as USHORT; }
    if strncmp(TypeName as *const ::core::ffi::c_char, TypePTR.as_ptr(), strlen(TypePTR.as_ptr())) == 0 { return TYPE_PTR as USHORT; }
    if strncmp(TypeName as *const ::core::ffi::c_char, TypeMX.as_ptr(), strlen(TypeMX.as_ptr())) == 0 { return TYPE_MX as USHORT; }
    if strncmp(TypeName as *const ::core::ffi::c_char, TypeAny.as_ptr(), strlen(TypeAny.as_ptr())) == 0 { return TYPE_ANY as USHORT; }

    return 0 as USHORT;
}
#[no_mangle]
pub unsafe extern "C" fn ClassIDtoClassName(ClassID: USHORT) -> PCHAR {
    match (ClassID) as u32 {
        CLASS_IN => return ClassIN.as_ptr() as PCHAR,
        CLASS_ANY => return ClassAny.as_ptr() as PCHAR,
        _ => return b"Unknown\0".as_ptr() as PCHAR,
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClassNametoClassID(mut ClassName: PCHAR) -> USHORT {
    if strncmp(
        ClassName as *const ::core::ffi::c_char,
        ClassIN.as_ptr(),
        strlen(ClassIN.as_ptr()),
    ) == 0
    {
        return (CLASS_IN) as USHORT;
    }
    if strncmp(
        ClassName as *const ::core::ffi::c_char,
        ClassAny.as_ptr(),
        strlen(ClassAny.as_ptr()),
    ) == 0
    {
        return (CLASS_ANY) as USHORT;
    }

    return 0;
}
