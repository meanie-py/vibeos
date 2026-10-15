use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::addrconv::ntohs;
use crate::src::r#async::WsAsyncAllocateBlock;
use crate::src::r#async::WsAsyncCancelRequest;
use crate::src::r#async::WsAsyncCheckAndInitThread;
use crate::src::r#async::WsAsyncQueueRequest;
use crate::src::dthread::WsThreadBlobToHostent;
use crate::src::dthread::WsThreadBlobToServent;
use crate::src::rasdial::WSNoteSuccessfulHostentLookup;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut HostnameGuid: GUID = _GUID {
    Data1: 0x2a800 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut AddressGuid: GUID = _GUID {
    Data1: 0x2a801 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut HostAddrByNameGuid: GUID = _GUID {
    Data1: 0x2a803 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IANAGuid: GUID = _GUID {
    Data1: 0x2a802 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut InetHostName: GUID = _GUID {
    Data1: 0x2a803 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut Ipv6Guid: GUID = _GUID {
    Data1: ((0x9 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | 53 as ::core::ffi::c_int)
        as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x1c as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut afp: [AFPROTOCOLS; 2] = [
    _AFPROTOCOLS {
        iAddressFamily: (AF_INET as i32),
        iProtocol: IPPROTO_UDP as ::core::ffi::c_int as INT,
    },
    _AFPROTOCOLS {
        iAddressFamily: (AF_INET as i32),
        iProtocol: IPPROTO_TCP as ::core::ffi::c_int as INT,
    },
];
#[no_mangle]
pub unsafe extern "system" fn FixList(
    List: *mut *mut PCHAR,
    Base: ULONG_PTR,
) {
    if !(*List).is_null() {
        let mut Addr: *mut PCHAR;

        *List = ((*List as ULONG_PTR).wrapping_add(Base)) as *mut PCHAR;
        Addr = *List;

        while !(*Addr).is_null() {
            *Addr = ((*Addr as ULONG_PTR).wrapping_add(Base)) as PCHAR;
            Addr = Addr.offset(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn UnpackServEnt(Servent: PSERVENT) {
    let ServentPtr: ULONG_PTR = Servent as ULONG_PTR;

    FixList(core::ptr::addr_of_mut!((*Servent).s_aliases), ServentPtr);

    (*Servent).s_name = (*Servent).s_name.offset(ServentPtr as isize);
    (*Servent).s_proto = (*Servent).s_proto.offset(ServentPtr as isize);
}
#[no_mangle]
pub unsafe extern "system" fn UnpackHostEnt(Hostent: PHOSTENT) {
    let HostentPtr: ULONG_PTR = Hostent as ULONG_PTR;

    if !(*Hostent).h_name.is_null() {
        (*Hostent).h_name = ((*Hostent).h_name as ULONG_PTR + HostentPtr) as PCHAR;
    }

    FixList(core::ptr::addr_of_mut!((*Hostent).h_aliases), HostentPtr);
    FixList(core::ptr::addr_of_mut!((*Hostent).h_addr_list), HostentPtr);
}
#[no_mangle]
pub unsafe extern "system" fn Local_Ip4AddresstoString(AddressBuffer: PCHAR, Address: PCHAR) {

    sprintf(
        AddressBuffer as *mut i8,
        b"%u.%u.%u.%u\0".as_ptr() as *const i8,
        (*Address.offset(0) as u32) & 0xff,
        (*Address.offset(1) as u32) & 0xff,
        (*Address.offset(2) as u32) & 0xff,
        (*Address.offset(3) as u32) & 0xff,
    );
}
#[no_mangle]
pub unsafe extern "system" fn Local_Ip6AddresstoString(
    AddressBuffer: PCHAR,
    Address: PCHAR,
) {
    let mut i: DWORD = 0;

    i = 0;
    while i < 8 {
        sprintf(
            AddressBuffer,
            b"%x:\0".as_ptr() as *const i8,
            (*Address.offset(0) as u32) & 0xff,
        );
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "system" fn getxyDataEnt(
    Results: *mut PCHAR,
    Length: DWORD,
    Name: LPSTR,
    Type: LPCGUID,
    NewName: *mut LPSTR,
) -> LPBLOB {
    let mut WsaQuery: PWSAQUERYSETA = (*Results) as PWSAQUERYSETA;
    let mut ErrorCode: INT;
    let mut NewLength: DWORD = Length;
    let mut RnRHandle: HANDLE = core::ptr::null_mut();
    let mut Blob: LPBLOB = core::ptr::null_mut();
    let mut NewResults: PVOID = core::ptr::null_mut();
    let mut dwControlFlags: DWORD = LUP_RETURN_NAME;

    if !NewName.is_null() {
        *NewName = core::ptr::null_mut();
    }

    RtlZeroMemory(
        WsaQuery as *mut core::ffi::c_void,
        core::mem::size_of::<_WSAQuerySetA>(),
    );
    (*WsaQuery).dwSize = core::mem::size_of::<_WSAQuerySetA>() as DWORD;
    (*WsaQuery).lpszServiceInstanceName = Name;
    (*WsaQuery).lpServiceClassId = Type as LPGUID;
    (*WsaQuery).dwNameSpace = NS_ALL;
    (*WsaQuery).dwNumberOfProtocols = (core::mem::size_of::<[AFPROTOCOLS; 2]>()
        / core::mem::size_of::<AFPROTOCOLS>()) as DWORD;
    (*WsaQuery).lpafpProtocols = core::ptr::addr_of_mut!(afp) as *mut AFPROTOCOLS;

    // C: `if (!IsEqualGUID(Type, &HostnameGuid))`. This read `!!IsEqualGUID(..)`, a
    // double negation, so the condition was inverted: gethostbyname never asked for
    // the hostent blob it then requires, and every lookup failed WSANO_DATA with the
    // address already resolved.
    if !IsEqualGUID(Type, core::ptr::addr_of!(HostnameGuid)) {
        dwControlFlags |= LUP_RETURN_BLOB;
    }

    ErrorCode = WSALookupServiceBeginA(WsaQuery, dwControlFlags, core::ptr::addr_of_mut!(RnRHandle));

    if ErrorCode == ERROR_SUCCESS as i32 {
        loop {
            ErrorCode = WSALookupServiceNextA(
                RnRHandle,
                0,
                core::ptr::addr_of_mut!(NewLength),
                WsaQuery,
            );

            if ErrorCode == ERROR_SUCCESS as i32 {
                Blob = (*WsaQuery).lpBlob;
                if !Blob.is_null() {
                    if !NewName.is_null() {
                        *NewName = (*WsaQuery).lpszServiceInstanceName;
                    }
                } else if IsEqualGUID(Type, core::ptr::addr_of!(HostnameGuid)) {
                    if !NewName.is_null() {
                        *NewName = (*WsaQuery).lpszServiceInstanceName;
                    }
                } else {
                    ErrorCode = WSANO_DATA as INT;
                }
            } else {
                ErrorCode = GetLastError() as INT;

                if ErrorCode == WSAEFAULT as INT && Length > RNR_BUFFER_SIZE {
                    NewResults = HeapAlloc(WsSockHeap, 0, ((Length as usize as u64)) as SIZE_T);
                    if !NewResults.is_null() {
                        *Results = NewResults as PCHAR;
                        WsaQuery = NewResults as PWSAQUERYSETA;
                        continue;
                    } else {
                        ErrorCode = WSA_NOT_ENOUGH_MEMORY as INT;
                    }
                }
            }

            WSALookupServiceEnd(RnRHandle);

            if ErrorCode != ERROR_SUCCESS as i32 {
                SetLastError(ErrorCode as DWORD);
            }

            break;
        }
    }

    Blob
}

#[no_mangle]
pub unsafe extern "system" fn gethostbyname(name: *const i8) -> PHOSTENT {
    let mut Hostent: PHOSTENT = core::ptr::null_mut();
    let mut Blob: LPBLOB;
    let mut ErrorCode: i32;
    let mut ResultsBuffer: [i8; 512] = [0; 512];
    let mut Results: *mut i8 = ResultsBuffer.as_mut_ptr();
    let mut szLocalName: [i8; 128] = [0; 128];
    let mut pszName: *mut i8;
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();

    ErrorCode = WsApiProlog(
        core::ptr::addr_of_mut!(Process),
        core::ptr::addr_of_mut!(Thread),
    );

    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as u32);
        return core::ptr::null_mut();
    }

    if name.is_null() || *name == 0 {
        if gethostname(szLocalName.as_mut_ptr(), (MAX_HOSTNAME_LEN as i32)) != (NO_ERROR as i32) {
            return core::ptr::null_mut();
        }
        pszName = szLocalName.as_mut_ptr();
    } else {
        pszName = name as *mut i8;
    }

    Blob = getxyDataEnt(
        core::ptr::addr_of_mut!(Results),
        RNR_BUFFER_SIZE as u32,
        pszName,
        core::ptr::addr_of!(HostAddrByNameGuid),
        core::ptr::null_mut(),
    );

    if Blob.is_null() && (name.is_null() || *name == 0) {
        Blob = getxyDataEnt(
            core::ptr::addr_of_mut!(Results),
            RNR_BUFFER_SIZE as u32,
            core::ptr::null_mut(),
            core::ptr::addr_of!(HostAddrByNameGuid),
            core::ptr::null_mut(),
        );
    }

    if !Blob.is_null() {
        Hostent = WsThreadBlobToHostent(Thread, Blob);
        if !Hostent.is_null() {
            UnpackHostEnt(Hostent);
        }
    } else {
        Hostent = core::ptr::null_mut();
        if GetLastError() == WSASERVICE_NOT_FOUND as u32 {
            SetLastError(WSAHOST_NOT_FOUND as u32);
        }
    }

    if Results != ResultsBuffer.as_mut_ptr() {
        HeapFree(WsSockHeap, 0, Results as PVOID);
    }

    if !Hostent.is_null() {
        WSNoteSuccessfulHostentLookup(
            name,
            **((*Hostent).h_addr_list.offset(0)) as u32,
        );
    }

    Hostent
}

#[no_mangle]
pub unsafe extern "system" fn gethostbyaddr(
    addr: *const i8,
    len: i32,
    type_0: i32,
) -> PHOSTENT {
    let mut AddressBuffer: [CHAR; 100] = [0; 100];
    let mut Hostent: PHOSTENT;
    let mut Blob: LPBLOB;
    let mut ResultsBuffer: [CHAR; RNR_BUFFER_SIZE as usize] = [0; RNR_BUFFER_SIZE as usize];
    let mut Results: PCHAR = ResultsBuffer.as_mut_ptr();
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let ErrorCode: INT;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as DWORD);
        return core::ptr::null_mut();
    }

    if addr.is_null() {
        SetLastError(WSAEINVAL as DWORD);
        return core::ptr::null_mut();
    }

    if type_0 == (AF_INET as i32) {
        Local_Ip4AddresstoString(AddressBuffer.as_mut_ptr(), addr as PCHAR);
    } else if type_0 == (AF_INET6 as i32) {
        Local_Ip6AddresstoString(AddressBuffer.as_mut_ptr(), addr as PCHAR);
    } else {
        SetLastError(WSAEINVAL as DWORD);
        return core::ptr::null_mut();
    }

    Blob = getxyDataEnt(
        core::ptr::addr_of_mut!(Results),
        RNR_BUFFER_SIZE,
        AddressBuffer.as_mut_ptr(),
        core::ptr::addr_of!(AddressGuid),
        core::ptr::null_mut(),
    );

    if !Blob.is_null() {
        Hostent = WsThreadBlobToHostent(Thread, Blob);
        if !Hostent.is_null() {
            UnpackHostEnt(Hostent);
        }
    } else {
        Hostent = core::ptr::null_mut();
        if GetLastError() == WSASERVICE_NOT_FOUND as DWORD {
            SetLastError(WSAHOST_NOT_FOUND as DWORD);
        }
    }

    if Results != ResultsBuffer.as_mut_ptr() {
        HeapFree(WsSockHeap, 0, Results as LPVOID);
    }

    Hostent
}

#[no_mangle]
pub unsafe extern "system" fn gethostname(
    name: *mut i8,
    namelen: i32,
) -> i32 {
    let mut Name: *mut i8 = core::ptr::null_mut();
    let mut ResultsBuffer: [i8; 512] = [0; 512];
    let mut Results: *mut i8 = ResultsBuffer.as_mut_ptr();

    if name.is_null() || namelen < 1 {
        SetLastError(WSAEFAULT);
        return SOCKET_ERROR;
    }

    getxyDataEnt(
        &mut Results,
        RNR_BUFFER_SIZE as u32,
        core::ptr::null_mut(),
        &HostnameGuid,
        &mut Name,
    );

    if !Name.is_null() {
        strncpy(name, Name, (namelen - 1) as usize);
    }

    if Results != ResultsBuffer.as_mut_ptr() {
        HeapFree(WsSockHeap, 0, Results as *mut core::ffi::c_void);
    }

    (ERROR_SUCCESS as i32)
}

#[no_mangle]
pub unsafe extern "system" fn getservbyport(
    port: i32,
    proto: *const i8,
) -> PSERVENT {
    let mut Servent: PSERVENT;
    let mut Blob: LPBLOB;
    let mut ResultsBuffer: [i8; 512] = [0; 512];
    let mut Results: *mut i8 = ResultsBuffer.as_mut_ptr();
    let mut PortName: *mut i8;
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let ErrorCode: i32;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as u32);
        return core::ptr::null_mut();
    }

    let proto = if proto.is_null() {
        b"\0".as_ptr() as *const i8
    } else {
        proto
    };

    PortName = HeapAlloc(
        WsSockHeap,
        0,
        (((strlen(proto) as usize)
            .wrapping_add(1)
            .wrapping_add(1)
            .wrapping_add(5) as u64)) as SIZE_T,
    ) as *mut i8;
    if PortName.is_null() {
        SetLastError(WSA_NOT_ENOUGH_MEMORY as u32);
        return core::ptr::null_mut();
    }

    sprintf(
        PortName,
        b"%d/%s\0".as_ptr() as *const i8,
        (ntohs(port as u16) as i32) & 0xffff,
        proto,
    );

    Blob = getxyDataEnt(
        core::ptr::addr_of_mut!(Results),
        RNR_BUFFER_SIZE as u32,
        PortName as LPSTR,
        core::ptr::addr_of!(IANAGuid),
        core::ptr::null_mut(),
    );

    HeapFree(WsSockHeap, 0, PortName as *mut core::ffi::c_void);

    if !Blob.is_null() {
        Servent = WsThreadBlobToServent(Thread, Blob);
        if !Servent.is_null() {
            UnpackServEnt(Servent);
        }
    } else {
        Servent = core::ptr::null_mut();
    }

    if Results != ResultsBuffer.as_mut_ptr() {
        HeapFree(WsSockHeap, 0, Results as *mut core::ffi::c_void);
    }

    Servent
}

#[no_mangle]
pub unsafe extern "system" fn getservbyname(
    name: *const i8,
    proto: *const i8,
) -> PSERVENT {
    let mut Servent: PSERVENT;
    let mut Blob: LPBLOB;
    let mut ResultsBuffer: [i8; 512] = [0; 512];
    let mut Results: *mut i8 = ResultsBuffer.as_mut_ptr();
    let mut PortName: *mut i8;
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let ErrorCode: i32;

    ErrorCode = WsApiProlog(
        core::ptr::addr_of_mut!(Process),
        core::ptr::addr_of_mut!(Thread),
    );
    if ErrorCode != ERROR_SUCCESS as i32 {
        SetLastError(ErrorCode as u32);
        return core::ptr::null_mut();
    }

    let proto = if proto.is_null() {
        b"\0".as_ptr() as *const i8
    } else {
        proto
    };

    PortName = HeapAlloc(
        WsSockHeap,
        0,
        ((strlen(proto)
            .wrapping_add(1)
            .wrapping_add(strlen(name))
            .wrapping_add(1) as u64)) as SIZE_T,
    ) as *mut i8;
    if PortName.is_null() {
        SetLastError(WSA_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    sprintf(PortName, b"%s/%s\0".as_ptr() as *const i8, name, proto);

    Blob = getxyDataEnt(
        core::ptr::addr_of_mut!(Results),
        RNR_BUFFER_SIZE,
        PortName,
        core::ptr::addr_of!(IANAGuid),
        core::ptr::null_mut(),
    );

    HeapFree(WsSockHeap, 0, PortName as *mut core::ffi::c_void);

    if !Blob.is_null() {
        Servent = WsThreadBlobToServent(Thread, Blob);
        if !Servent.is_null() {
            UnpackServEnt(Servent);
        }
    } else {
        Servent = core::ptr::null_mut();
    }

    if Results != ResultsBuffer.as_mut_ptr() {
        HeapFree(WsSockHeap, 0, Results as *mut core::ffi::c_void);
    }

    Servent
}

#[no_mangle]
pub unsafe extern "system" fn WSAAsyncGetHostByAddr(
    hWnd: HWND,
    wMsg: u32,
    Address: *const i8,
    Length: i32,
    Type: i32,
    Buffer: *mut i8,
    BufferLength: i32,
) -> *mut core::ffi::c_void {
    let mut TaskHandle: *mut core::ffi::c_void;
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut AddressCopy: *mut core::ffi::c_void;
    let mut AsyncBlock: PWSASYNCBLOCK;
    let mut ErrorCode: i32;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != ERROR_SUCCESS as i32 {
        SetLastError(ErrorCode as u32);
        return core::ptr::null_mut();
    }

    if WsAsyncCheckAndInitThread() == 0 {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    AsyncBlock = WsAsyncAllocateBlock(Length as usize);
    if AsyncBlock.is_null() {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    AddressCopy = AsyncBlock.offset(1) as *mut core::ffi::c_void;
    core::ptr::copy_nonoverlapping(Address as *const u8, AddressCopy as *mut u8, Length as usize);

    (*AsyncBlock).Operation = WsAsyncGetHostByAddr;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().hWnd = hWnd;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().wMsg = wMsg;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().ByWhat = AddressCopy as *mut i8;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().Length = Length as u32;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().Type = Type as u32;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().Buffer = Buffer as *mut core::ffi::c_void;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().BufferLength = BufferLength as u32;

    TaskHandle = (*AsyncBlock).TaskHandle;
    WsAsyncQueueRequest(AsyncBlock);

    TaskHandle
}

#[no_mangle]
pub unsafe extern "system" fn WSAAsyncGetHostByName(
    hWnd: HWND,
    wMsg: u32,
    Name: *const i8,
    Buffer: *mut i8,
    BufferLength: i32,
) -> *mut core::ffi::c_void {

    let mut TaskHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut AsyncBlock: PWSASYNCBLOCK = core::ptr::null_mut();
    let mut ErrorCode: i32;
    let mut NameCopy: *mut core::ffi::c_void;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != ERROR_SUCCESS as i32 {
        SetLastError(ErrorCode as u32);
        return core::ptr::null_mut();
    }

    if WsAsyncCheckAndInitThread() == 0 {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    AsyncBlock = WsAsyncAllocateBlock(strlen(Name).wrapping_add(core::mem::size_of::<i8>()));
    if AsyncBlock.is_null() {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    NameCopy = AsyncBlock.offset(1) as *mut core::ffi::c_void;
    strcpy(NameCopy as *mut i8, (Name as *mut i8));

    (*AsyncBlock).Operation = WsAsyncGetHostByName;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().hWnd = hWnd;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().wMsg = wMsg;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().ByWhat = NameCopy as *mut i8;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().Buffer = Buffer as *mut core::ffi::c_void;
    (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().BufferLength = BufferLength as u32;

    TaskHandle = (*AsyncBlock).TaskHandle;
    WsAsyncQueueRequest(AsyncBlock);

    TaskHandle
}

#[no_mangle]
pub unsafe extern "system" fn WSAAsyncGetProtoByName(
    hWnd: HWND,
    wMsg: u32,
    Name: *const i8,
    Buffer: *mut i8,
    BufferLength: i32,
) -> *mut core::ffi::c_void {

    let mut TaskHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut AsyncBlock: PWSASYNCBLOCK = core::ptr::null_mut();
    let mut ErrorCode: i32;
    let mut NameCopy: *mut core::ffi::c_void;

    ErrorCode = WsApiProlog(
        core::ptr::addr_of_mut!(Process),
        core::ptr::addr_of_mut!(Thread),
    );
    if ErrorCode != ERROR_SUCCESS as i32 {
        SetLastError(ErrorCode as u32);
        return core::ptr::null_mut();
    }

    if WsAsyncCheckAndInitThread() == 0 {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    AsyncBlock = WsAsyncAllocateBlock(
        strlen(Name).wrapping_add(core::mem::size_of::<i8>()),
    );
    if AsyncBlock.is_null() {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    NameCopy = AsyncBlock.offset(1) as *mut core::ffi::c_void;
    strcpy(NameCopy as *mut i8, (Name as *mut i8));

    (*AsyncBlock).Operation = WsAsyncGetProtoByName;
    (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().hWnd = hWnd;
    (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().wMsg = wMsg;
    (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().ByWhat = NameCopy as *mut i8;
    (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().Buffer = Buffer as *mut core::ffi::c_void;
    (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().BufferLength = BufferLength as u32;

    TaskHandle = (*AsyncBlock).TaskHandle;
    WsAsyncQueueRequest(AsyncBlock);

    TaskHandle
}

#[no_mangle]
pub unsafe extern "system" fn WSAAsyncGetProtoByNumber(
    hWnd: HWND,
    wMsg: u32,
    Number: i32,
    Buffer: *mut i8,
    BufferLength: i32,
) -> *mut core::ffi::c_void {
    let mut TaskHandle: *mut core::ffi::c_void;
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut AsyncBlock: PWSASYNCBLOCK;
    let mut ErrorCode: i32;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as u32);
        return core::ptr::null_mut();
    }

    if WsAsyncCheckAndInitThread() == 0 {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    AsyncBlock = WsAsyncAllocateBlock(0);
    if AsyncBlock.is_null() {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    (*AsyncBlock).Operation = WsAsyncGetProtoByNumber;
    (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().hWnd = hWnd;
    (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().wMsg = wMsg;
    (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().ByWhat = UlongToPtr(Number as u32) as *mut i8;
    (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().Buffer = Buffer as *mut core::ffi::c_void;
    (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().BufferLength = BufferLength as u32;

    TaskHandle = (*AsyncBlock).TaskHandle;
    WsAsyncQueueRequest(AsyncBlock);

    TaskHandle
}

#[no_mangle]
pub unsafe extern "system" fn WSAAsyncGetServByName(
    hWnd: HWND,
    wMsg: u32,
    Name: *const i8,
    Protocol: *const i8,
    Buffer: *mut i8,
    BufferLength: i32,
) -> *mut core::ffi::c_void {
    let mut TaskHandle: *mut core::ffi::c_void;
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut AsyncBlock: PWSASYNCBLOCK;
    let mut ErrorCode: i32;
    let mut NameCopy: *mut core::ffi::c_void;

    ErrorCode = WsApiProlog(
        core::ptr::addr_of_mut!(Process),
        core::ptr::addr_of_mut!(Thread),
    );
    if ErrorCode != ERROR_SUCCESS as i32 {
        SetLastError(ErrorCode as u32);
        return core::ptr::null_mut();
    }

    if WsAsyncCheckAndInitThread() == 0 {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    AsyncBlock = WsAsyncAllocateBlock(
        strlen((Name as *const u8 as *const i8)) + core::mem::size_of::<i8>(),
    );
    if AsyncBlock.is_null() {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    NameCopy = AsyncBlock.offset(1) as *mut core::ffi::c_void;
    strcpy(NameCopy as *mut core::ffi::c_char, Name as *const core::ffi::c_char);

    (*AsyncBlock).Operation = WsAsyncGetServByPort;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().hWnd = hWnd;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().wMsg = wMsg;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().ByWhat = NameCopy as *mut i8;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().Protocol = Protocol as *mut i8;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().Buffer = Buffer as *mut core::ffi::c_void;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().BufferLength = BufferLength as u32;

    TaskHandle = (*AsyncBlock).TaskHandle;
    WsAsyncQueueRequest(AsyncBlock);

    TaskHandle
}

#[no_mangle]
pub unsafe extern "system" fn WSAAsyncGetServByPort(
    hWnd: HWND,
    wMsg: u32,
    Port: i32,
    Protocol: *const i8,
    Buffer: *mut i8,
    BufferLength: i32,
) -> *mut core::ffi::c_void {
    let mut TaskHandle: *mut core::ffi::c_void;
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut AsyncBlock: PWSASYNCBLOCK;
    let ErrorCode: i32;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as u32);
        return core::ptr::null_mut();
    }

    if WsAsyncCheckAndInitThread() == 0 {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    AsyncBlock = WsAsyncAllocateBlock(0);
    if AsyncBlock.is_null() {
        SetLastError(WSAENOBUFS as u32);
        return core::ptr::null_mut();
    }

    (*AsyncBlock).Operation = WsAsyncGetServByPort;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().hWnd = hWnd;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().wMsg = wMsg;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().ByWhat = UlongToPtr(Port as u32) as *mut i8;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().Protocol = Protocol as *mut i8;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().Buffer = Buffer as *mut core::ffi::c_void;
    (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().BufferLength = BufferLength as u32;

    TaskHandle = (*AsyncBlock).TaskHandle;
    WsAsyncQueueRequest(AsyncBlock);

    TaskHandle
}

#[no_mangle]
pub unsafe extern "system" fn WSACancelAsyncRequest(hAsyncTaskHandle: HANDLE) -> ::core::ffi::c_int {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode == (ERROR_SUCCESS as i32) {
        ErrorCode = WsAsyncCancelRequest(hAsyncTaskHandle);
        if ErrorCode == (ERROR_SUCCESS as i32) {
            return (ERROR_SUCCESS as i32);
        }
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}
// enforce_import_abi: moved 5 import decl(s) into the extern block their
// exporting DLL's .spec requires: GetLastError->system, HeapAlloc->system, HeapFree->system, SetLastError->system, WSNoteSuccessfulHostentLookup->system.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/getxbyxx.c
//  * PURPOSE:     Get X by Y Functions for Name Resolution.
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* DATA **********************************************************************/
//
// AFPROTOCOLS afp[2] = {{AF_INET, IPPROTO_UDP}, {AF_INET, IPPROTO_TCP}};
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// __stdcall
// FixList(PCHAR **List,
//         ULONG_PTR Base)
// {
//     /* Make sure it's valid */
//     if (*List)
//     {
//         PCHAR *Addr;
//
//         /* Get the right base */
//         Addr = *List = (PCHAR*)(((ULONG_PTR)*List + Base));
//
//         /* Loop the pointers */
//         while (*Addr)
//         {
//             /* Rebase them too */
//             *Addr = (PCHAR)(((ULONG_PTR)*Addr + Base));
//             Addr++;
//         }
//     }
// }
//
// VOID
// __stdcall
// UnpackServEnt(PSERVENT Servent)
// {
//     ULONG_PTR ServentPtr = (ULONG_PTR)Servent;
//
//     /* Convert all the List Offsets to Pointers */
//     FixList(&Servent->s_aliases, ServentPtr);
//
//     /* Convert the Name and Protocol Offsets to Pointers */
//     Servent->s_name = (PCHAR)(Servent->s_name + ServentPtr);
//     Servent->s_proto = (PCHAR)(Servent->s_proto + ServentPtr);
// }
//
// VOID
// __stdcall
// UnpackHostEnt(PHOSTENT Hostent)
// {
//     ULONG_PTR HostentPtr = (ULONG_PTR)Hostent;
//
//     /* Convert the Name Offset to a Pointer */
//     if (Hostent->h_name) Hostent->h_name = (PCHAR)(Hostent->h_name + HostentPtr);
//
//     /* Convert all the List Offsets to Pointers */
//     FixList(&Hostent->h_aliases, HostentPtr);
//     FixList(&Hostent->h_addr_list, HostentPtr);
// }
//
// VOID
// __stdcall
// Local_Ip4AddresstoString(IN PCHAR AddressBuffer,
//                          IN PCHAR Address)
// {
//     /* Convert the address into IPv4 format */
//     sprintf(AddressBuffer, "%u.%u.%u.%u",
//             ((unsigned)Address[0] & 0xff),
//             ((unsigned)Address[1] & 0xff),
//             ((unsigned)Address[2] & 0xff),
//             ((unsigned)Address[3] & 0xff));
// }
//
// VOID
// __stdcall
// Local_Ip6AddresstoString(IN PCHAR AddressBuffer,
//                          IN PCHAR Address)
// {
//     DWORD i;
//
//     /* Convert the address into IPv6 format */
//     for (i = 0; i < 8; i++)
//     {
//         sprintf(AddressBuffer, "%x:",
//                 ((unsigned)Address[0] & 0xff));
//     }
// }
//
// LPBLOB
// __stdcall
// getxyDataEnt(IN OUT PCHAR *Results,
//              IN DWORD Length,
//              IN LPSTR Name,
//              IN LPCGUID Type,
//              IN LPSTR *NewName)
// {
//     PWSAQUERYSETA WsaQuery = (PWSAQUERYSETA)*Results;
//     INT ErrorCode;
//     DWORD NewLength = Length;
//     HANDLE RnRHandle;
//     LPBLOB Blob = NULL;
//     PVOID NewResults = NULL;
//     DWORD dwControlFlags = LUP_RETURN_NAME;
//
//     /* Assume empty return name */
//     if (NewName) *NewName = NULL;
//
//     /* Set up the Winsock Service Query */
//     RtlZeroMemory(WsaQuery, sizeof(*WsaQuery));
//     WsaQuery->dwSize = sizeof(*WsaQuery);
//     WsaQuery->lpszServiceInstanceName = Name;
//     WsaQuery->lpServiceClassId = (LPGUID)Type;
//     WsaQuery->dwNameSpace = NS_ALL;
//     WsaQuery->dwNumberOfProtocols = sizeof(afp)/sizeof(afp[0]);
//     WsaQuery->lpafpProtocols = afp;
//
//     if (!IsEqualGUID(Type, &HostnameGuid))
//         dwControlFlags |= LUP_RETURN_BLOB;
//
//     /* Send the Query Request to find a Service */
//     ErrorCode = WSALookupServiceBeginA(WsaQuery,
//                                        dwControlFlags,
//                                        &RnRHandle);
//
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         while (TRUE)
//         {
//             /* Service was found, send the real query */
//             ErrorCode = WSALookupServiceNextA(RnRHandle,
//                                               0,
//                                               &NewLength,
//                                               WsaQuery);
//
//             /* Return the information requested */
//             if (ErrorCode == ERROR_SUCCESS)
//             {
//                 /* Get the Blob and check if we have one */
//                 Blob = WsaQuery->lpBlob;
//                 if (Blob)
//                 {
//                     /* Did they want the name back? */
//                     if (NewName) *NewName = WsaQuery->lpszServiceInstanceName;
//                 }
//                 else
//                 {
//                     /* Check if this was a Hostname lookup */
//                     if (IsEqualGUID(Type, &HostnameGuid))
//                     {
//                         /* Return the name anyways */
//                         if (NewName) *NewName = WsaQuery->lpszServiceInstanceName;
//                     }
//                     else
//                     {
//                         /* We don't have a blob, sorry */
//                         ErrorCode = WSANO_DATA;
//                     }
//                 }
//             }
//             else
//             {
//                 /* WSALookupServiceEnd will set its own error, so save ours */
//                 ErrorCode = GetLastError();
//
//                 /* Check if we failed because of missing buffer space */
//                 if ((ErrorCode == WSAEFAULT) && (Length > RNR_BUFFER_SIZE))
//                 {
//                     /* Allocate a new buffer */
//                     NewResults = HeapAlloc(WsSockHeap, 0, Length);
//                     if (NewResults)
//                     {
//                         /* Tell the caller his new buffer */
//                         *Results = NewResults;
//
//                         /* Update the WSA Query's location */
//                         WsaQuery = (PWSAQUERYSETA)NewResults;
//
//                         /* Loop again */
//                         continue;
//                     }
//                     else
//                     {
//                         /* No memory to allocate the new buffer */
//                         ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//                     }
//                 }
//             }
//
//             /* Finish the Query Request */
//             WSALookupServiceEnd(RnRHandle);
//
//             /* Now set the Last Error */
//             if (ErrorCode != ERROR_SUCCESS) SetLastError(ErrorCode);
//
//             /* Leave the loop */
//             break;
//         }
//     }
//
//     /* Return the blob */
//     return Blob;
// }
//
// /*
//  * @implemented
//  */
// PHOSTENT
// __stdcall
// gethostbyname(IN const char  * name)
// {
//     PHOSTENT Hostent;
//     LPBLOB Blob;
//     INT ErrorCode;
//     CHAR ResultsBuffer[RNR_BUFFER_SIZE];
//     PCHAR Results = ResultsBuffer;
//     CHAR szLocalName[MAX_HOSTNAME_LEN];
//     PCHAR pszName;
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     DPRINT("gethostbyname: %s\n", name);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* Check if no name was given */
//     if (!name || !*name)
//     {
//         /* This means we should do a local lookup first */
//         if (gethostname(szLocalName, MAX_HOSTNAME_LEN) != NO_ERROR) return(NULL);
//         pszName = szLocalName;
//     }
//     else
//     {
//         /* Use the name tha twas given to us */
//         pszName = (PCHAR)name;
//     }
//
//     /* Get the Hostname in a Blob Structure */
//     Blob = getxyDataEnt(&Results,
//                         RNR_BUFFER_SIZE,
//                         pszName,
//                         &HostAddrByNameGuid,
//                         0);
//
//     /* Check if we didn't get a blob, or if we got an empty name */
//     if (!(Blob) && (!(name) || !(*name)))
//     {
//         /* Try a new query */
//         Blob = getxyDataEnt(&Results,
//                             RNR_BUFFER_SIZE,
//                             NULL,
//                             &HostAddrByNameGuid,
//                             0);
//     }
//
//     /* Check if we got a blob */
//     if (Blob)
//     {
//         /* Copy the blob to our buffer and convert it */
//         Hostent = WsThreadBlobToHostent(Thread, Blob);
//
//         /* Unpack the hostent */
//         if (Hostent) UnpackHostEnt(Hostent);
//     }
//     else
//     {
//         /* We failed, so zero it out */
//         Hostent = NULL;
//
//         /* Normalize the error message */
//         if (GetLastError() == WSASERVICE_NOT_FOUND)
//         {
//             SetLastError(WSAHOST_NOT_FOUND);
//         }
//     }
//
//     /* Check if we received a newly allocated buffer; free it. */
//     if (Results != ResultsBuffer) HeapFree(WsSockHeap, 0, Results);
//
//     /* Notify RAS Auto-dial helper */
//     if (Hostent) WSNoteSuccessfulHostentLookup(name, *Hostent->h_addr);
//
//     /* Return the hostent */
//     return Hostent;
// }
//
// /*
//  * @implemented
//  */
// PHOSTENT
// __stdcall
// gethostbyaddr(IN const char  * addr,
//               IN int len,
//               IN int type)
// {
//     CHAR AddressBuffer[100];
//     PHOSTENT Hostent;
//     LPBLOB Blob;
//     CHAR ResultsBuffer[RNR_BUFFER_SIZE];
//     PCHAR Results = ResultsBuffer;
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     DPRINT("gethostbyaddr: %s\n", addr);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* Check for valid address pointer */
//     if (!addr)
//     {
//         /* Fail */
//         SetLastError(WSAEINVAL);
//         return NULL;
//     }
//
//     /* Check which type it is */
//     if (type == AF_INET)
//     {
//         /* Use IPV4 Address to String */
//         Local_Ip4AddresstoString(AddressBuffer, (PCHAR)addr);
//     }
//     else if (type == AF_INET6)
//     {
//         /* Use IPV6 Address to String */
//         Local_Ip6AddresstoString(AddressBuffer, (PCHAR)addr);
//     }
//     else
//     {
//         /* Invalid address type; fail */
//         SetLastError(WSAEINVAL);
//         return NULL;
//     }
//
//     /* Get the Hostname in a Blob Structure */
//     Blob = getxyDataEnt(&Results,
//                         RNR_BUFFER_SIZE,
//                         AddressBuffer,
//                         &AddressGuid,
//                         0);
//
//     /* Check if we got a blob */
//     if (Blob)
//     {
//         /* Copy the blob to our buffer and convert it */
//         Hostent = WsThreadBlobToHostent(Thread, Blob);
//
//         /* Unpack the hostent */
//         if (Hostent) UnpackHostEnt(Hostent);
//     }
//     else
//     {
//         /* We failed, so zero it out */
//         Hostent = NULL;
//
//         /* Normalize the error message */
//         if (GetLastError() == WSASERVICE_NOT_FOUND)
//         {
//             SetLastError(WSAHOST_NOT_FOUND);
//         }
//     }
//
//     /* Check if we received a newly allocated buffer; free it. */
//     if (Results != ResultsBuffer) HeapFree(WsSockHeap, 0, Results);
//
//     /* Return the hostent */
//     return Hostent;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// gethostname(OUT char  * name,
//             IN INT namelen)
// {
//     PCHAR Name = NULL;
//     CHAR ResultsBuffer[RNR_BUFFER_SIZE];
//     PCHAR Results = ResultsBuffer;
//     DPRINT("gethostname: %p\n", name);
//
//     if (!name || namelen < 1)
//     {
//         SetLastError(WSAEFAULT);
//         return SOCKET_ERROR;
//     }
//     /* Get the Hostname in a String */
//     /* getxyDataEnt does not return blob for HostnameGuid */
//     getxyDataEnt(&Results, RNR_BUFFER_SIZE, NULL, &HostnameGuid, &Name);
//     if (Name)
//     {
//         /* Copy it */
//         strncpy(name, Name, namelen-1);
//     }
//
//     /* Check if we received a newly allocated buffer; free it. */
//     if (Results != ResultsBuffer) HeapFree(WsSockHeap, 0, Results);
//
//     /* Return success */
//     return ERROR_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// PSERVENT
// __stdcall
// getservbyport(IN int port,
//               IN const char  * proto)
// {
//     PSERVENT Servent;
//     LPBLOB Blob;
//     CHAR ResultsBuffer[RNR_BUFFER_SIZE];
//     PCHAR Results = ResultsBuffer;
//     PCHAR PortName;
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     DPRINT("getservbyport: %s\n", proto);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* No protocol specified */
//     if (!proto) proto = "";
//
//     /* Allocate memory for the port name */
//     PortName = HeapAlloc(WsSockHeap, 0, strlen(proto) + 1 + 1 + 5);
//     if (!PortName)
//     {
//         /* Fail */
//         SetLastError(WSA_NOT_ENOUGH_MEMORY);
//         return NULL;
//     }
//
//     /* Put it into the right syntax */
//     sprintf(PortName, "%d/%s", (ntohs(port) & 0xffff), proto);
//
//     /* Get the Service in a Blob */
//     Blob = getxyDataEnt(&Results, RNR_BUFFER_SIZE, PortName, &IANAGuid, 0);
//
//     /* Free the string we sent */
//     HeapFree(WsSockHeap, 0, PortName);
//
//     /* Check if we got a blob */
//     if (Blob)
//     {
//         /* Copy the blob to our buffer and convert it */
//         Servent = WsThreadBlobToServent(Thread, Blob);
//
//         /* Unpack the hostent */
//         if (Servent) UnpackServEnt(Servent);
//     }
//     else
//     {
//         /* We failed, so zero it out */
//         Servent = 0;
//     }
//
//     /* Check if we received a newly allocated buffer; free it. */
//     if (Results != ResultsBuffer) HeapFree(WsSockHeap, 0, Results);
//
//     /* Return the hostent */
//     return Servent;
// }
//
// /*
//  * @implemented
//  */
// PSERVENT
// __stdcall
// getservbyname(IN const char  * name,
//               IN const char  * proto)
// {
//     PSERVENT Servent;
//     LPBLOB Blob;
//     CHAR ResultsBuffer[RNR_BUFFER_SIZE];
//     PCHAR Results = ResultsBuffer;
//     PCHAR PortName;
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     DPRINT("getservbyname: %s\n", name);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* No protocol specified */
//     if (!proto) proto = "";
//
//     /* Allocate buffer for it */
//     PortName = HeapAlloc(WsSockHeap, 0, strlen(proto) + 1 + strlen(name) + 1);
//     if (!PortName)
//     {
//         /* Fail */
//         SetLastError(WSA_NOT_ENOUGH_MEMORY);
//         return NULL;
//     }
//
//     /* Put it into the right syntax */
//     sprintf(PortName, "%s/%s", name, proto);
//
//     /* Get the Service in a Blob */
//     Blob = getxyDataEnt(&Results, RNR_BUFFER_SIZE, PortName, &IANAGuid, 0);
//
//     /* Free the string we sent */
//     HeapFree(WsSockHeap, 0, PortName);
//
//     /* Check if we got a blob */
//     if (Blob)
//     {
//         /* Copy the blob to our buffer and convert it */
//         Servent = WsThreadBlobToServent(Thread, Blob);
//
//         /* Unpack the hostent */
//         if (Servent) UnpackServEnt(Servent);
//     }
//     else
//     {
//         /* We failed, so zero it out */
//         Servent = 0;
//     }
//
//     /* Check if we received a newly allocated buffer; free it. */
//     if (Results != ResultsBuffer) HeapFree(WsSockHeap, 0, Results);
//
//     /* Return the hostent */
//     return Servent;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// __stdcall
// WSAAsyncGetHostByAddr(IN HWND hWnd,
//                       IN UINT wMsg,
//                       IN CONST CHAR  *Address,
//                       IN INT Length,
//                       IN INT Type,
//                       OUT CHAR  *Buffer,
//                       IN INT BufferLength)
// {
//     HANDLE TaskHandle;
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PVOID AddressCopy;
//     PWSASYNCBLOCK AsyncBlock;
//     INT ErrorCode;
//     DPRINT("WSAAsyncGetHostByAddr: %lx, %lx, %s\n", hWnd, wMsg, Address);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* Initialize the Async Thread */
//     if (!WsAsyncCheckAndInitThread())
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Allocate an async block */
//     if (!(AsyncBlock = WsAsyncAllocateBlock(Length)))
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Make a copy of the address */
//     AddressCopy = AsyncBlock + 1;
//     RtlMoveMemory(AddressCopy, Address, Length);
//
//     /* Initialize the Async Block */
//     AsyncBlock->Operation = WsAsyncGetHostByAddr;
//     AsyncBlock->GetHost.hWnd = hWnd;
//     AsyncBlock->GetHost.wMsg = wMsg;
//     AsyncBlock->GetHost.ByWhat = AddressCopy;
//     AsyncBlock->GetHost.Length = Length;
//     AsyncBlock->GetHost.Type = Type;
//     AsyncBlock->GetHost.Buffer = Buffer;
//     AsyncBlock->GetHost.BufferLength = BufferLength;
//
//     /* Save the task handle and queue the request */
//     TaskHandle = AsyncBlock->TaskHandle;
//     WsAsyncQueueRequest(AsyncBlock);
//
//     /* Return the task handle */
//     return TaskHandle;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// __stdcall
// WSAAsyncGetHostByName(IN HWND hWnd,
//                       IN UINT wMsg,
//                       IN CONST CHAR  *Name,
//                       OUT CHAR  *Buffer,
//                       IN INT BufferLength)
// {
//     HANDLE TaskHandle;
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSASYNCBLOCK AsyncBlock;
//     INT ErrorCode;
//     PVOID NameCopy;
//     DPRINT("WSAAsyncGetProtoByNumber: %lx, %lx, %s\n", hWnd, wMsg, Name);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* Initialize the Async Thread */
//     if (!WsAsyncCheckAndInitThread())
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Allocate an async block */
//     if (!(AsyncBlock = WsAsyncAllocateBlock(strlen(Name) + sizeof(CHAR))))
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Make a copy of the address */
//     NameCopy = AsyncBlock + 1;
//     strcpy(NameCopy, Name);
//
//     /* Initialize the Async Block */
//     AsyncBlock->Operation = WsAsyncGetHostByName;
//     AsyncBlock->GetHost.hWnd = hWnd;
//     AsyncBlock->GetHost.wMsg = wMsg;
//     AsyncBlock->GetHost.ByWhat = NameCopy;
//     AsyncBlock->GetHost.Buffer = Buffer;
//     AsyncBlock->GetHost.BufferLength = BufferLength;
//
//     /* Save the task handle and queue the request */
//     TaskHandle = AsyncBlock->TaskHandle;
//     WsAsyncQueueRequest(AsyncBlock);
//
//     /* Return the task handle */
//     return TaskHandle;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// __stdcall
// WSAAsyncGetProtoByName(IN HWND hWnd,
//                        IN UINT wMsg,
//                        IN CONST CHAR  *Name,
//                        OUT CHAR  *Buffer,
//                        IN INT BufferLength)
// {
//     HANDLE TaskHandle;
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSASYNCBLOCK AsyncBlock;
//     INT ErrorCode;
//     PVOID NameCopy;
//     DPRINT("WSAAsyncGetProtoByName: %lx, %lx, %s\n", hWnd, wMsg, Name);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* Initialize the Async Thread */
//     if (!WsAsyncCheckAndInitThread())
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Allocate an async block */
//     if (!(AsyncBlock = WsAsyncAllocateBlock(strlen(Name) + sizeof(CHAR))))
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Make a copy of the address */
//     NameCopy = AsyncBlock + 1;
//     strcpy(NameCopy, Name);
//
//     /* Initialize the Async Block */
//     AsyncBlock->Operation = WsAsyncGetProtoByName;
//     AsyncBlock->GetProto.hWnd = hWnd;
//     AsyncBlock->GetProto.wMsg = wMsg;
//     AsyncBlock->GetProto.ByWhat = NameCopy;
//     AsyncBlock->GetProto.Buffer = Buffer;
//     AsyncBlock->GetProto.BufferLength = BufferLength;
//
//     /* Save the task handle and queue the request */
//     TaskHandle = AsyncBlock->TaskHandle;
//     WsAsyncQueueRequest(AsyncBlock);
//
//     /* Return the task handle */
//     return TaskHandle;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// __stdcall
// WSAAsyncGetProtoByNumber(IN HWND hWnd,
//                          IN UINT wMsg,
//                          IN INT Number,
//                          OUT CHAR * Buffer,
//                          IN INT BufferLength)
// {
//     HANDLE TaskHandle;
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSASYNCBLOCK AsyncBlock;
//     INT ErrorCode;
//     DPRINT("WSAAsyncGetProtoByNumber: %lx, %lx, %lx\n", hWnd, wMsg, Number);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* Initialize the Async Thread */
//     if (!WsAsyncCheckAndInitThread())
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Allocate an async block */
//     if (!(AsyncBlock = WsAsyncAllocateBlock(0)))
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Initialize the Async Block */
//     AsyncBlock->Operation = WsAsyncGetProtoByNumber;
//     AsyncBlock->GetProto.hWnd = hWnd;
//     AsyncBlock->GetProto.wMsg = wMsg;
//     AsyncBlock->GetProto.ByWhat = UlongToPtr(Number);
//     AsyncBlock->GetProto.Buffer = Buffer;
//     AsyncBlock->GetProto.BufferLength = BufferLength;
//
//     /* Save the task handle and queue the request */
//     TaskHandle = AsyncBlock->TaskHandle;
//     WsAsyncQueueRequest(AsyncBlock);
//
//     /* Return the task handle */
//     return TaskHandle;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// __stdcall
// WSAAsyncGetServByName(IN HWND hWnd,
//                       IN UINT wMsg,
//                       IN CONST CHAR  *Name,
//                       IN CONST CHAR  *Protocol,
//                       OUT CHAR  *Buffer,
//                       IN INT BufferLength)
// {
//     HANDLE TaskHandle;
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSASYNCBLOCK AsyncBlock;
//     INT ErrorCode;
//     PVOID NameCopy;
//     DPRINT("WSAAsyncGetProtoByNumber: %lx, %lx, %s\n", hWnd, wMsg, Name);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* Initialize the Async Thread */
//     if (!WsAsyncCheckAndInitThread())
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Allocate an async block */
//     if (!(AsyncBlock = WsAsyncAllocateBlock(strlen(Name) + sizeof(CHAR))))
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Make a copy of the address */
//     NameCopy = AsyncBlock + 1;
//     strcpy(NameCopy, Name);
//
//     /* Initialize the Async Block */
//     AsyncBlock->Operation = WsAsyncGetProtoByName;
//     AsyncBlock->GetServ.hWnd = hWnd;
//     AsyncBlock->GetServ.wMsg = wMsg;
//     AsyncBlock->GetServ.ByWhat = NameCopy;
//     AsyncBlock->GetServ.Protocol = (PCHAR)Protocol;
//     AsyncBlock->GetServ.Buffer = Buffer;
//     AsyncBlock->GetServ.BufferLength = BufferLength;
//
//     /* Save the task handle and queue the request */
//     TaskHandle = AsyncBlock->TaskHandle;
//     WsAsyncQueueRequest(AsyncBlock);
//
//     /* Return the task handle */
//     return TaskHandle;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// __stdcall
// WSAAsyncGetServByPort(IN HWND hWnd,
//                       IN UINT wMsg,
//                       IN INT Port,
//                       IN CONST CHAR  *Protocol,
//                       OUT CHAR  *Buffer,
//                       IN INT BufferLength)
// {
//     HANDLE TaskHandle;
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSASYNCBLOCK AsyncBlock;
//     INT ErrorCode;
//     DPRINT("WSAAsyncGetProtoByNumber: %lx, %lx, %lx\n", hWnd, wMsg, Port);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* Initialize the Async Thread */
//     if (!WsAsyncCheckAndInitThread())
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Allocate an async block */
//     if (!(AsyncBlock = WsAsyncAllocateBlock(0)))
//     {
//         /* Fail */
//         SetLastError(WSAENOBUFS);
//         return NULL;
//     }
//
//     /* Initialize the Async Block */
//     AsyncBlock->Operation = WsAsyncGetServByPort;
//     AsyncBlock->GetServ.hWnd = hWnd;
//     AsyncBlock->GetServ.wMsg = wMsg;
//     AsyncBlock->GetServ.ByWhat = UlongToPtr(Port);
//     AsyncBlock->GetServ.Protocol = (PCHAR)Protocol;
//     AsyncBlock->GetServ.Buffer = Buffer;
//     AsyncBlock->GetServ.BufferLength = BufferLength;
//
//     /* Save the task handle and queue the request */
//     TaskHandle = AsyncBlock->TaskHandle;
//     WsAsyncQueueRequest(AsyncBlock);
//
//     /* Return the task handle */
//     return TaskHandle;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSACancelAsyncRequest(IN HANDLE hAsyncTaskHandle)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     DPRINT("WSACancelAsyncRequest: %lx\n", hAsyncTaskHandle);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) == ERROR_SUCCESS)
//     {
//         /* Call the Async code */
//         ErrorCode = WsAsyncCancelRequest(hAsyncTaskHandle);
//
//         /* Return */
//         if (ErrorCode == ERROR_SUCCESS) return ERROR_SUCCESS;
//     }
//
//     /* Fail */
//     SetLastError(ErrorCode);
//     return SOCKET_ERROR;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

/// Wine protocol.c GetHostNameW: fetch the ANSI name via gethostname, convert with CP_ACP;
/// WSAEFAULT if `name` is NULL or the converted string (incl. NUL) exceeds `namelen` WCHARs.
#[no_mangle]
pub unsafe extern "system" fn GetHostNameW(name: *mut u16, namelen: i32) -> i32 {
    let mut buf: [i8; 256] = [0; 256]; // [255] stays 0: gethostname copies at most 255 bytes
    if name.is_null() {
        SetLastError(WSAEFAULT);
        return SOCKET_ERROR;
    }
    if gethostname(buf.as_mut_ptr(), buf.len() as i32) != 0 {
        return SOCKET_ERROR; // gethostname already set the last error
    }
    if MultiByteToWideChar(0 /* CP_ACP */, 0, buf.as_ptr(), -1, core::ptr::null_mut(), 0) > namelen {
        SetLastError(WSAEFAULT);
        return SOCKET_ERROR;
    }
    MultiByteToWideChar(0, 0, buf.as_ptr(), -1, name, namelen);
    0
}
