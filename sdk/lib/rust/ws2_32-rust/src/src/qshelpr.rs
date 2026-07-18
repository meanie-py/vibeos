use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;

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

#[no_mangle]
pub unsafe extern "system" fn AnsiDupFromUnicode(UnicodeString: LPCWSTR) -> LPSTR {
    let mut Length: i32 = 0;
    let mut GoOn: i32 = 1;
    let mut DuplicatedString: LPSTR = core::ptr::null_mut();
    let mut ReturnValue: i32;

    while GoOn != 0 {
        ReturnValue = WideCharToMultiByte(
            CP_ACP,
            0,
            (UnicodeString as *mut u16),
            -1,
            DuplicatedString,
            Length,
            core::ptr::null(),
            core::ptr::null_mut(),
        );

        if ReturnValue > Length {
            Length = ReturnValue;
            DuplicatedString = HeapAlloc(WsSockHeap, 0, ((Length as usize as u64)) as SIZE_T) as LPSTR;
            if DuplicatedString.is_null() {
                GoOn = 0;
            }
        } else if ReturnValue > 0 {
            GoOn = 0;
        } else {
            if !DuplicatedString.is_null() {
                HeapFree(WsSockHeap, 0, DuplicatedString as *mut core::ffi::c_void);
            }
            DuplicatedString = core::ptr::null_mut();
            GoOn = 0;
        }
    }

    DuplicatedString
}
#[no_mangle]
pub unsafe extern "system" fn UnicodeDupFromAnsi(AnsiString: LPCSTR) -> LPWSTR {
    let mut Length: i32 = 0;
    let mut GoOn: i32 = 1;
    let mut DuplicatedString: LPWSTR = core::ptr::null_mut();
    let mut ReturnValue: i32;

    while GoOn != 0 {
        ReturnValue = MultiByteToWideChar(
            CP_ACP,
            0,
            AnsiString,
            -1,
            DuplicatedString,
            Length,
        );
        if ReturnValue > Length {
            Length = ReturnValue;
            DuplicatedString = HeapAlloc(
                WsSockHeap,
                0,
                (((Length as usize).wrapping_mul(core::mem::size_of::<u16>()) as u64)) as SIZE_T,
            ) as LPWSTR;
            if DuplicatedString.is_null() {
                GoOn = 0;
            }
        } else if ReturnValue > 0 {
            GoOn = 0;
        } else {
            if !DuplicatedString.is_null() {
                HeapFree(WsSockHeap, 0, DuplicatedString as *mut core::ffi::c_void);
            }
            DuplicatedString = core::ptr::null_mut();
            GoOn = 0;
        }
    }

    DuplicatedString
}

const CP_ACP: u32 = 0;

#[no_mangle]
pub unsafe extern "system" fn ComputeStringSize(
    String: LPSTR,
    IsUnicode: u8,
) -> usize {
    if IsUnicode != 0 {
        (wcslen(String as LPWSTR) + 1) * core::mem::size_of::<u16>()
    } else {
        strlen(String as *const i8) + core::mem::size_of::<i8>()
    }
}
#[no_mangle]
pub unsafe extern "system" fn ComputeQuerySetSize(
    AnsiSet: LPWSAQUERYSETA,
    IsUnicode: u8,
) -> SIZE_T {
    let mut Size: SIZE_T = core::mem::size_of::<WSAQUERYSETA>() as SIZE_T;
    let mut i: u32;

    if !(*AnsiSet).lpszServiceInstanceName.is_null() {
        Size = Size.wrapping_add(((ComputeStringSize(
            (*AnsiSet).lpszServiceInstanceName,
            IsUnicode,
        ) as u64)) as SIZE_T);
    }

    if !(*AnsiSet).lpServiceClassId.is_null() {
        Size = (Size.wrapping_add(3)) & !3;
        Size = Size.wrapping_add(core::mem::size_of::<GUID>() as SIZE_T);
    }

    if !(*AnsiSet).lpVersion.is_null() {
        Size = (Size.wrapping_add(3)) & !3;
        Size = Size.wrapping_add(core::mem::size_of::<WSAVERSION>() as SIZE_T);
    }

    if !(*AnsiSet).lpszComment.is_null() {
        Size = (Size.wrapping_add(1)) & !1;
        Size = Size.wrapping_add(((ComputeStringSize((*AnsiSet).lpszComment, IsUnicode) as u64)) as SIZE_T);
    }

    if !(*AnsiSet).lpNSProviderId.is_null() {
        Size = (Size.wrapping_add(3)) & !3;
        Size = Size.wrapping_add(core::mem::size_of::<GUID>() as SIZE_T);
    }

    if !(*AnsiSet).lpszContext.is_null() {
        Size = (Size.wrapping_add(1)) & !1;
        Size = Size.wrapping_add(((ComputeStringSize((*AnsiSet).lpszContext, IsUnicode) as u64)) as SIZE_T);
    }

    if !(*AnsiSet).lpszQueryString.is_null() {
        Size = (Size.wrapping_add(1)) & !1;
        Size = Size.wrapping_add(((ComputeStringSize(
            (*AnsiSet).lpszQueryString,
            IsUnicode,
        ) as u64)) as SIZE_T);
    }

    if !(*AnsiSet).lpafpProtocols.is_null() {
        Size = (Size.wrapping_add(3)) & !3;
        Size = Size.wrapping_add(
            (core::mem::size_of::<AFPROTOCOLS>() as SIZE_T)
                .wrapping_mul((*AnsiSet).dwNumberOfProtocols as SIZE_T),
        );
    }

    if !(*AnsiSet).lpcsaBuffer.is_null() {
        Size = (Size.wrapping_add(3)) & !3;

        i = 0;
        while i < (*AnsiSet).dwNumberOfCsAddrs {
            if !(*(*AnsiSet).lpcsaBuffer.offset(i as isize))
                .LocalAddr
                .lpSockaddr
                .is_null()
            {
                Size = (Size.wrapping_add(3)) & !3;
                Size = Size.wrapping_add(
                    (*(*AnsiSet).lpcsaBuffer.offset(i as isize))
                        .LocalAddr
                        .iSockaddrLength as SIZE_T,
                );
            }
            if !(*(*AnsiSet).lpcsaBuffer.offset(i as isize))
                .RemoteAddr
                .lpSockaddr
                .is_null()
            {
                Size = (Size.wrapping_add(3)) & !3;
                Size = Size.wrapping_add(
                    (*(*AnsiSet).lpcsaBuffer.offset(i as isize))
                        .RemoteAddr
                        .iSockaddrLength as SIZE_T,
                );
            }

            Size = Size.wrapping_add(core::mem::size_of::<CSADDR_INFO>() as SIZE_T);
            i = i.wrapping_add(1);
        }
    }

    if !(*AnsiSet).lpBlob.is_null() {
        Size = (Size.wrapping_add(3)) & !3;
        Size = Size.wrapping_add(core::mem::size_of::<BLOB>() as SIZE_T);

        if !(*AnsiSet).lpBlob.is_null() {
            Size = Size.wrapping_add((*(*AnsiSet).lpBlob).cbSize as SIZE_T);
        }
    }

    Size
}

#[no_mangle]
pub unsafe extern "system" fn WSAComputeQuerySetSizeA(
    AnsiSet: LPWSAQUERYSETA,
) -> SIZE_T {
    ComputeQuerySetSize(AnsiSet, 0u8)
}

#[no_mangle]
pub unsafe extern "system" fn WSAComputeQuerySetSizeW(UnicodeSet: LPWSAQUERYSETW) -> SIZE_T {
    ComputeQuerySetSize(UnicodeSet as LPWSAQUERYSETA, 1u8)
}
#[no_mangle]
pub unsafe extern "system" fn WsBufferAllocate(
    Buffer: PWS_BUFFER,
    Size: usize,
    Align: u32,
) -> *mut core::ffi::c_void {
    let mut NewPosition: *mut core::ffi::c_void;

    (*Buffer).BytesUsed = (((((*Buffer).BytesUsed as u64)
        .wrapping_add(Align as u64)
        .wrapping_sub(1u64)
        & !(Align.wrapping_sub(1u32) as u64)) as usize as u64)) as SIZE_T;

    NewPosition = ((*Buffer).Position as usize).wrapping_add(((*Buffer).BytesUsed as usize)) as *mut core::ffi::c_void;

    (*Buffer).BytesUsed = ((((*Buffer).BytesUsed as u64)
        .wrapping_add(Size as u64) as usize as u64)) as SIZE_T;

    NewPosition
}
#[no_mangle]
pub unsafe extern "system" fn CopyBlobIndirect(
    Buffer: PWS_BUFFER,
    RelativeBlob: LPBLOB,
    Blob: LPBLOB,
) {
    if !(*Blob).pBlobData.is_null() && (*Blob).cbSize != 0 {
        (*RelativeBlob).pBlobData = WsBufferAllocate(
            Buffer,
            (*Blob).cbSize as usize,
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as *mut u8;
        core::ptr::copy_nonoverlapping(
            (*Blob).pBlobData,
            (*RelativeBlob).pBlobData,
            (*Blob).cbSize as usize,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn CopyAddrInfoArrayIndirect(
    Buffer: PWS_BUFFER,
    RelativeCsaddr: LPCSADDR_INFO,
    Addresses: u32,
    Csaddr: LPCSADDR_INFO,
) {
    let mut i: u32 = 0;

    while i < Addresses {
        if !(*Csaddr.offset(i as isize)).LocalAddr.lpSockaddr.is_null()
            && (*Csaddr.offset(i as isize)).LocalAddr.iSockaddrLength != 0
        {
            (*RelativeCsaddr.offset(i as isize)).LocalAddr.lpSockaddr =
                (WsBufferAllocate(
                    Buffer,
                    (*Csaddr.offset(i as isize)).LocalAddr.iSockaddrLength as usize,
                    core::mem::size_of::<*mut core::ffi::c_void>() as u32,
                ) as *mut core::ffi::c_void as *mut sockaddr);
            core::ptr::copy_nonoverlapping(
                (*Csaddr.offset(i as isize)).LocalAddr.lpSockaddr as *const u8,
                (*RelativeCsaddr.offset(i as isize)).LocalAddr.lpSockaddr as *mut u8,
                (*Csaddr.offset(i as isize)).LocalAddr.iSockaddrLength as usize,
            );
        } else {
            (*Csaddr.offset(i as isize)).LocalAddr.lpSockaddr = core::ptr::null_mut();
            (*Csaddr.offset(i as isize)).LocalAddr.iSockaddrLength = 0;
        }

        if !(*Csaddr.offset(i as isize)).RemoteAddr.lpSockaddr.is_null()
            && (*Csaddr.offset(i as isize)).RemoteAddr.iSockaddrLength != 0
        {
            (*RelativeCsaddr.offset(i as isize)).RemoteAddr.lpSockaddr =
                (WsBufferAllocate(
                    Buffer,
                    (*Csaddr.offset(i as isize)).RemoteAddr.iSockaddrLength as usize,
                    core::mem::size_of::<*mut core::ffi::c_void>() as u32,
                ) as *mut core::ffi::c_void as *mut sockaddr);
            core::ptr::copy_nonoverlapping(
                (*Csaddr.offset(i as isize)).RemoteAddr.lpSockaddr as *const u8,
                (*RelativeCsaddr.offset(i as isize)).RemoteAddr.lpSockaddr as *mut u8,
                (*Csaddr.offset(i as isize)).RemoteAddr.iSockaddrLength as usize,
            );
        } else {
            (*Csaddr.offset(i as isize)).RemoteAddr.lpSockaddr = core::ptr::null_mut();
            (*Csaddr.offset(i as isize)).RemoteAddr.iSockaddrLength = 0;
        }

        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "system" fn CopyQuerySetIndirectA(
    Buffer: PWS_BUFFER,
    RelativeSet: LPWSAQUERYSETA,
    AnsiSet: LPWSAQUERYSETA,
) {

    let mut AnsiString: LPSTR;

    AnsiString = (*AnsiSet).lpszServiceInstanceName;
    if !AnsiString.is_null() {
        (*RelativeSet).lpszServiceInstanceName = WsBufferAllocate(
            Buffer,
            strlen(AnsiString as *const core::ffi::c_char).wrapping_add(1),
            core::mem::size_of::<i8>() as u32,
        ) as LPSTR;
        strcpy(
            (*RelativeSet).lpszServiceInstanceName as *mut core::ffi::c_char,
            (AnsiString as *const core::ffi::c_char as *mut i8),
        );
    } else {
        (*RelativeSet).lpszServiceInstanceName = core::ptr::null_mut();
    }

    if !(*AnsiSet).lpServiceClassId.is_null() {
        (*RelativeSet).lpServiceClassId = WsBufferAllocate(
            Buffer,
            core::mem::size_of::<GUID>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as *mut GUID;
        *(*RelativeSet).lpServiceClassId = *(*AnsiSet).lpServiceClassId;
    } else {
        (*RelativeSet).lpServiceClassId = core::ptr::null_mut();
    }

    if !(*AnsiSet).lpVersion.is_null() {
        (*RelativeSet).lpVersion = WsBufferAllocate(
            Buffer,
            core::mem::size_of::<WSAVERSION>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as *mut WSAVERSION;
        core::ptr::copy_nonoverlapping((*AnsiSet).lpVersion, (*RelativeSet).lpVersion, 1);
    } else {
        (*RelativeSet).lpVersion = core::ptr::null_mut();
    }

    AnsiString = (*AnsiSet).lpszComment;
    if !AnsiString.is_null() {
        (*RelativeSet).lpszComment = WsBufferAllocate(
            Buffer,
            strlen(AnsiString as *const core::ffi::c_char).wrapping_add(1),
            core::mem::size_of::<i8>() as u32,
        ) as LPSTR;
        strcpy(
            (*RelativeSet).lpszComment as *mut core::ffi::c_char,
            (AnsiString as *const core::ffi::c_char as *mut i8),
        );
    } else {
        (*RelativeSet).lpszComment = core::ptr::null_mut();
    }

    if !(*AnsiSet).lpNSProviderId.is_null() {
        (*RelativeSet).lpNSProviderId = WsBufferAllocate(
            Buffer,
            core::mem::size_of::<GUID>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as *mut GUID;
        *(*RelativeSet).lpNSProviderId = *(*AnsiSet).lpNSProviderId;
    } else {
        (*RelativeSet).lpNSProviderId = core::ptr::null_mut();
    }

    AnsiString = (*AnsiSet).lpszContext;
    if !AnsiString.is_null() {
        (*RelativeSet).lpszContext = WsBufferAllocate(
            Buffer,
            strlen(AnsiString as *const core::ffi::c_char).wrapping_add(1),
            core::mem::size_of::<i8>() as u32,
        ) as LPSTR;
        strcpy(
            (*RelativeSet).lpszContext as *mut core::ffi::c_char,
            (AnsiString as *const core::ffi::c_char as *mut i8),
        );
    } else {
        (*RelativeSet).lpszContext = core::ptr::null_mut();
    }

    AnsiString = (*AnsiSet).lpszQueryString;
    if !AnsiString.is_null() {
        (*RelativeSet).lpszQueryString = WsBufferAllocate(
            Buffer,
            strlen(AnsiString as *const core::ffi::c_char).wrapping_add(1),
            core::mem::size_of::<i8>() as u32,
        ) as LPSTR;
        strcpy(
            (*RelativeSet).lpszQueryString as *mut core::ffi::c_char,
            (AnsiString as *const core::ffi::c_char as *mut i8),
        );
    } else {
        (*RelativeSet).lpszQueryString = core::ptr::null_mut();
    }

    if !(*AnsiSet).lpafpProtocols.is_null() && (*AnsiSet).dwNumberOfProtocols != 0 {
        (*RelativeSet).lpafpProtocols = WsBufferAllocate(
            Buffer,
            ((*AnsiSet).dwNumberOfProtocols as usize)
                .wrapping_mul(core::mem::size_of::<AFPROTOCOLS>()),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as *mut AFPROTOCOLS;
        RtlCopyMemory(
            (*RelativeSet).lpafpProtocols as *mut core::ffi::c_void,
            (*AnsiSet).lpafpProtocols as *const core::ffi::c_void,
            ((*AnsiSet).dwNumberOfProtocols as usize)
                .wrapping_mul(core::mem::size_of::<AFPROTOCOLS>()),
        );
    } else {
        (*RelativeSet).lpafpProtocols = core::ptr::null_mut();
        (*RelativeSet).dwNumberOfProtocols = 0;
    }

    if !(*AnsiSet).lpcsaBuffer.is_null() && (*AnsiSet).dwNumberOfCsAddrs != 0 {
        (*RelativeSet).lpcsaBuffer = WsBufferAllocate(
            Buffer,
            ((*AnsiSet).dwNumberOfCsAddrs as usize)
                .wrapping_mul(core::mem::size_of::<CSADDR_INFO>()),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as LPCSADDR_INFO;
        RtlCopyMemory(
            (*RelativeSet).lpcsaBuffer as *mut core::ffi::c_void,
            (*AnsiSet).lpcsaBuffer as *const core::ffi::c_void,
            ((*AnsiSet).dwNumberOfCsAddrs as usize)
                .wrapping_mul(core::mem::size_of::<CSADDR_INFO>()),
        );
        CopyAddrInfoArrayIndirect(
            Buffer,
            (*RelativeSet).lpcsaBuffer,
            (*AnsiSet).dwNumberOfCsAddrs,
            (*AnsiSet).lpcsaBuffer,
        );
    } else {
        (*RelativeSet).lpcsaBuffer = core::ptr::null_mut();
        (*RelativeSet).dwNumberOfCsAddrs = 0;
    }

    if !(*AnsiSet).lpBlob.is_null() {
        (*RelativeSet).lpBlob = WsBufferAllocate(
            Buffer,
            core::mem::size_of::<BLOB>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as LPBLOB;
        core::ptr::copy_nonoverlapping((*AnsiSet).lpBlob, (*RelativeSet).lpBlob, 1);
        CopyBlobIndirect(Buffer, (*RelativeSet).lpBlob, (*AnsiSet).lpBlob);
    } else {
        (*RelativeSet).lpBlob = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "system" fn CopyQuerySetIndirectW(
    Buffer: PWS_BUFFER,
    RelativeSet: LPWSAQUERYSETW,
    UnicodeSet: LPWSAQUERYSETW,
) {
    let mut UnicodeString: LPWSTR;

    UnicodeString = (*UnicodeSet).lpszServiceInstanceName;
    if !UnicodeString.is_null() {
        (*RelativeSet).lpszServiceInstanceName = WsBufferAllocate(
            Buffer,
            (wcslen(UnicodeString) + 1) * core::mem::size_of::<u16>(),
            core::mem::size_of::<i8>() as u32,
        ) as LPWSTR;
        wcscpy((*RelativeSet).lpszServiceInstanceName, UnicodeString);
    } else {
        (*RelativeSet).lpszServiceInstanceName = core::ptr::null_mut();
    }

    if !(*UnicodeSet).lpServiceClassId.is_null() {
        (*RelativeSet).lpServiceClassId = WsBufferAllocate(
            Buffer,
            core::mem::size_of::<GUID>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as *mut GUID;
        *(*RelativeSet).lpServiceClassId = *(*UnicodeSet).lpServiceClassId;
    } else {
        (*RelativeSet).lpServiceClassId = core::ptr::null_mut();
    }

    if !(*UnicodeSet).lpVersion.is_null() {
        (*RelativeSet).lpVersion = WsBufferAllocate(
            Buffer,
            core::mem::size_of::<WSAVERSION>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as *mut WSAVERSION;
        core::ptr::copy_nonoverlapping((*UnicodeSet).lpVersion, (*RelativeSet).lpVersion, 1);
    } else {
        (*RelativeSet).lpVersion = core::ptr::null_mut();
    }

    UnicodeString = (*UnicodeSet).lpszComment;
    if !UnicodeString.is_null() {
        (*RelativeSet).lpszComment = WsBufferAllocate(
            Buffer,
            (wcslen(UnicodeString) + 1) * core::mem::size_of::<u16>(),
            core::mem::size_of::<i8>() as u32,
        ) as LPWSTR;
        wcscpy((*RelativeSet).lpszComment, UnicodeString);
    } else {
        (*RelativeSet).lpszComment = core::ptr::null_mut();
    }

    if !(*UnicodeSet).lpNSProviderId.is_null() {
        (*RelativeSet).lpNSProviderId = WsBufferAllocate(
            Buffer,
            core::mem::size_of::<GUID>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as *mut GUID;
        *(*RelativeSet).lpNSProviderId = *(*UnicodeSet).lpNSProviderId;
    } else {
        (*RelativeSet).lpNSProviderId = core::ptr::null_mut();
    }

    UnicodeString = (*UnicodeSet).lpszContext;
    if !UnicodeString.is_null() {
        (*RelativeSet).lpszContext = WsBufferAllocate(
            Buffer,
            (wcslen(UnicodeString) + 1) * core::mem::size_of::<u16>(),
            core::mem::size_of::<i8>() as u32,
        ) as LPWSTR;
        wcscpy((*RelativeSet).lpszContext, UnicodeString);
    } else {
        (*RelativeSet).lpszContext = core::ptr::null_mut();
    }

    UnicodeString = (*UnicodeSet).lpszQueryString;
    if !UnicodeString.is_null() {
        (*RelativeSet).lpszQueryString = WsBufferAllocate(
            Buffer,
            (wcslen(UnicodeString) + 1) * core::mem::size_of::<u16>(),
            core::mem::size_of::<i8>() as u32,
        ) as LPWSTR;
        wcscpy((*RelativeSet).lpszQueryString, UnicodeString);
    } else {
        (*RelativeSet).lpszQueryString = core::ptr::null_mut();
    }

    if !(*UnicodeSet).lpafpProtocols.is_null() && (*UnicodeSet).dwNumberOfProtocols != 0 {
        (*RelativeSet).lpafpProtocols = WsBufferAllocate(
            Buffer,
            (*UnicodeSet).dwNumberOfProtocols as usize * core::mem::size_of::<AFPROTOCOLS>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as *mut AFPROTOCOLS;
        RtlCopyMemory(
            (*RelativeSet).lpafpProtocols as *mut core::ffi::c_void,
            (*UnicodeSet).lpafpProtocols as *const core::ffi::c_void,
            (*UnicodeSet).dwNumberOfProtocols as usize * core::mem::size_of::<AFPROTOCOLS>(),
        );
    } else {
        (*RelativeSet).lpafpProtocols = core::ptr::null_mut();
        (*RelativeSet).dwNumberOfProtocols = 0;
    }

    if !(*UnicodeSet).lpcsaBuffer.is_null() && (*UnicodeSet).dwNumberOfCsAddrs != 0 {
        (*RelativeSet).lpcsaBuffer = WsBufferAllocate(
            Buffer,
            (*UnicodeSet).dwNumberOfCsAddrs as usize * core::mem::size_of::<CSADDR_INFO>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as LPCSADDR_INFO;
        RtlCopyMemory(
            (*RelativeSet).lpcsaBuffer as *mut core::ffi::c_void,
            (*UnicodeSet).lpcsaBuffer as *const core::ffi::c_void,
            (*UnicodeSet).dwNumberOfCsAddrs as usize * core::mem::size_of::<CSADDR_INFO>(),
        );
        CopyAddrInfoArrayIndirect(
            Buffer,
            (*RelativeSet).lpcsaBuffer,
            (*UnicodeSet).dwNumberOfCsAddrs,
            (*UnicodeSet).lpcsaBuffer,
        );
    } else {
        (*RelativeSet).lpcsaBuffer = core::ptr::null_mut();
        (*RelativeSet).dwNumberOfCsAddrs = 0;
    }

    if !(*UnicodeSet).lpBlob.is_null() {
        (*RelativeSet).lpBlob = WsBufferAllocate(
            Buffer,
            core::mem::size_of::<BLOB>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as LPBLOB;
        core::ptr::copy_nonoverlapping((*UnicodeSet).lpBlob, (*RelativeSet).lpBlob, 1);
        CopyBlobIndirect(Buffer, (*RelativeSet).lpBlob, (*UnicodeSet).lpBlob);
    } else {
        (*RelativeSet).lpBlob = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn WSABuildQuerySetBufferA(
    AnsiSet: LPWSAQUERYSETA,
    BufferSize: usize,
    RelativeSet: LPWSAQUERYSETA,
) -> i32 {
    let mut ErrorCode: i32 = ERROR_SUCCESS as i32;
    let mut SetSize: usize;
    let mut Buffer: WS_BUFFER = WS_BUFFER {
        Position: 0,
        MaxSize: 0,
        BytesUsed: 0,
    };
    let mut NewSet: LPWSAQUERYSETA;

    SetSize = (WSAComputeQuerySetSizeA(AnsiSet) as usize);
    if SetSize <= BufferSize {
        Buffer.Position = ((RelativeSet as usize as u64)) as SIZE_T;
        Buffer.MaxSize = ((SetSize as u64)) as SIZE_T;
        Buffer.BytesUsed = 0;

        NewSet = WsBufferAllocate(
            core::ptr::addr_of_mut!(Buffer),
            core::mem::size_of::<WSAQUERYSETA>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as LPWSAQUERYSETA;
        core::ptr::copy_nonoverlapping(AnsiSet, NewSet, 1);

        CopyQuerySetIndirectA(core::ptr::addr_of_mut!(Buffer), NewSet, AnsiSet);
    } else {
        ErrorCode = SOCKET_ERROR;
    }

    return ErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn WSABuildQuerySetBufferW(
    UnicodeSet: LPWSAQUERYSETW,
    BufferSize: usize,
    RelativeSet: LPWSAQUERYSETW,
) -> i32 {
    let mut ErrorCode: i32 = ERROR_SUCCESS as i32;
    let mut SetSize: usize;
    let mut Buffer: WS_BUFFER = WS_BUFFER {
        Position: 0,
        MaxSize: 0,
        BytesUsed: 0,
    };
    let mut NewSet: LPWSAQUERYSETW;

    SetSize = (WSAComputeQuerySetSizeW(UnicodeSet) as usize);
    if SetSize <= BufferSize {
        Buffer.Position = ((RelativeSet as usize as u64)) as SIZE_T;
        Buffer.MaxSize = ((SetSize as u64)) as SIZE_T;
        Buffer.BytesUsed = 0;

        NewSet = WsBufferAllocate(
            core::ptr::addr_of_mut!(Buffer),
            core::mem::size_of::<WSAQUERYSETW>(),
            core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        ) as LPWSAQUERYSETW;
        core::ptr::copy_nonoverlapping(UnicodeSet, NewSet, 1);

        CopyQuerySetIndirectW(core::ptr::addr_of_mut!(Buffer), NewSet, UnicodeSet);
    } else {
        ErrorCode = SOCKET_ERROR;
    }

    return ErrorCode;
}
#[no_mangle]
pub unsafe extern "system" fn MapAnsiQuerySetToUnicode(
    AnsiSet: LPWSAQUERYSETA,
    SetSize: PSIZE_T,
    UnicodeSet: LPWSAQUERYSETW,
) -> i32 {
    let mut ErrorCode: i32 = (ERROR_SUCCESS as i32);
    let mut AnsiSize: usize;
    let mut UnicodeSize: usize;
    let mut AnsiCopy: LPWSAQUERYSETA = core::ptr::null_mut();
    let mut UnicodeCopy: LPWSAQUERYSETW;
    let mut ServiceCopy: LPWSTR = core::ptr::null_mut();
    let mut CommentCopy: LPWSTR = core::ptr::null_mut();
    let mut ContextCopy: LPWSTR = core::ptr::null_mut();
    let mut QueryCopy: LPWSTR = core::ptr::null_mut();

    'exit: {
        AnsiSize = (WSAComputeQuerySetSizeA(AnsiSet) as usize);
        AnsiCopy = HeapAlloc(WsSockHeap, 0, ((AnsiSize as u64)) as SIZE_T) as LPWSAQUERYSETA;
        if AnsiCopy.is_null() {
            ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
            break 'exit;
        }

        ErrorCode = WSABuildQuerySetBufferA(AnsiSet, AnsiSize, AnsiCopy);
        if ErrorCode != (ERROR_SUCCESS as i32) {
            break 'exit;
        }

        UnicodeCopy = AnsiCopy as LPWSAQUERYSETW;

        if !(*AnsiCopy).lpszServiceInstanceName.is_null() {
            ServiceCopy = UnicodeDupFromAnsi((*AnsiCopy).lpszServiceInstanceName);
            if ServiceCopy.is_null() {
                ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
                break 'exit;
            }
            (*UnicodeCopy).lpszServiceInstanceName = ServiceCopy;
        }

        if !(*AnsiCopy).lpszContext.is_null() {
            ContextCopy = UnicodeDupFromAnsi((*AnsiCopy).lpszContext);
            if ContextCopy.is_null() {
                ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
                break 'exit;
            }
            (*UnicodeCopy).lpszContext = ContextCopy;
        }

        if !(*AnsiCopy).lpszComment.is_null() {
            CommentCopy = UnicodeDupFromAnsi((*AnsiCopy).lpszComment);
            if CommentCopy.is_null() {
                ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
                break 'exit;
            }
            (*UnicodeCopy).lpszComment = CommentCopy;
        }

        if !(*AnsiCopy).lpszQueryString.is_null() {
            QueryCopy = UnicodeDupFromAnsi((*AnsiCopy).lpszQueryString);
            if QueryCopy.is_null() {
                ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
                break 'exit;
            }
            (*UnicodeCopy).lpszQueryString = QueryCopy;
        }

        UnicodeSize = (WSAComputeQuerySetSizeW(UnicodeCopy) as usize);
        if UnicodeSize > (*SetSize as usize) {
            *SetSize = ((UnicodeSize as u64)) as SIZE_T;
            ErrorCode = (WSAEFAULT as i32);
            break 'exit;
        }

        ErrorCode = WSABuildQuerySetBufferW(UnicodeCopy, (*SetSize as usize), UnicodeSet);
    }

    if !AnsiCopy.is_null() {
        HeapFree(WsSockHeap, 0, AnsiCopy as PVOID);
    }
    if !ServiceCopy.is_null() {
        HeapFree(WsSockHeap, 0, ServiceCopy as PVOID);
    }
    if !CommentCopy.is_null() {
        HeapFree(WsSockHeap, 0, CommentCopy as PVOID);
    }
    if !ContextCopy.is_null() {
        HeapFree(WsSockHeap, 0, ContextCopy as PVOID);
    }
    if !QueryCopy.is_null() {
        HeapFree(WsSockHeap, 0, QueryCopy as PVOID);
    }

    ErrorCode
}
#[no_mangle]
pub unsafe extern "system" fn MapUnicodeQuerySetToAnsi(
    UnicodeSet: LPWSAQUERYSETW,
    SetSize: PSIZE_T,
    AnsiSet: LPWSAQUERYSETA,
) -> i32 {
    let mut ErrorCode: i32 = (ERROR_SUCCESS as i32);
    let mut UnicodeSize: usize;
    let mut AnsiSize: usize;
    let mut UnicodeCopy: LPWSAQUERYSETW = core::ptr::null_mut();
    let mut AnsiCopy: LPWSAQUERYSETA;
    let mut ServiceCopy: LPSTR = core::ptr::null_mut();
    let mut CommentCopy: LPSTR = core::ptr::null_mut();
    let mut ContextCopy: LPSTR = core::ptr::null_mut();
    let mut QueryCopy: LPSTR = core::ptr::null_mut();

    'exit: {
        UnicodeSize = (WSAComputeQuerySetSizeW(UnicodeSet) as usize);
        UnicodeCopy = HeapAlloc(WsSockHeap, 0, ((UnicodeSize as u64)) as SIZE_T) as LPWSAQUERYSETW;
        if UnicodeCopy.is_null() {
            ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
            break 'exit;
        }

        ErrorCode = WSABuildQuerySetBufferW(UnicodeSet, UnicodeSize, UnicodeCopy);
        if ErrorCode != (ERROR_SUCCESS as i32) {
            break 'exit;
        }

        AnsiCopy = UnicodeCopy as LPWSAQUERYSETA;

        if !(*UnicodeCopy).lpszServiceInstanceName.is_null() {
            ServiceCopy = AnsiDupFromUnicode((*UnicodeCopy).lpszServiceInstanceName);
            if ServiceCopy.is_null() {
                ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
                break 'exit;
            }
            (*AnsiCopy).lpszServiceInstanceName = ServiceCopy;
        }

        if !(*UnicodeCopy).lpszContext.is_null() {
            ContextCopy = AnsiDupFromUnicode((*UnicodeCopy).lpszContext);
            if ContextCopy.is_null() {
                ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
                break 'exit;
            }
            (*AnsiCopy).lpszContext = ContextCopy;
        }

        if !(*UnicodeCopy).lpszComment.is_null() {
            CommentCopy = AnsiDupFromUnicode((*UnicodeCopy).lpszComment);
            if CommentCopy.is_null() {
                ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
                break 'exit;
            }
            (*AnsiCopy).lpszComment = CommentCopy;
        }

        if !(*UnicodeCopy).lpszQueryString.is_null() {
            QueryCopy = AnsiDupFromUnicode((*UnicodeCopy).lpszQueryString);
            if QueryCopy.is_null() {
                ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
                break 'exit;
            }
            (*AnsiCopy).lpszQueryString = QueryCopy;
        }

        AnsiSize = (WSAComputeQuerySetSizeA(AnsiCopy) as usize);
        if AnsiSize > (*SetSize as usize) {
            *SetSize = ((AnsiSize as u64)) as SIZE_T;
            ErrorCode = (WSAEFAULT as i32);
            break 'exit;
        }

        ErrorCode = WSABuildQuerySetBufferA(AnsiCopy, (*SetSize as usize), AnsiSet);
    }

    if !UnicodeCopy.is_null() {
        HeapFree(WsSockHeap, 0, UnicodeCopy as *mut core::ffi::c_void);
    }
    if !ServiceCopy.is_null() {
        HeapFree(WsSockHeap, 0, ServiceCopy as *mut core::ffi::c_void);
    }
    if !CommentCopy.is_null() {
        HeapFree(WsSockHeap, 0, CommentCopy as *mut core::ffi::c_void);
    }
    if !ContextCopy.is_null() {
        HeapFree(WsSockHeap, 0, ContextCopy as *mut core::ffi::c_void);
    }
    if !QueryCopy.is_null() {
        HeapFree(WsSockHeap, 0, QueryCopy as *mut core::ffi::c_void);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn CopyQuerySetW(
    UnicodeSet: LPWSAQUERYSETW,
    UnicodeCopy: *mut LPWSAQUERYSETW,
) -> i32 {
    let SetSize: usize;

    SetSize = (WSAComputeQuerySetSizeW(UnicodeSet) as usize);

    *UnicodeCopy = HeapAlloc(WsSockHeap, 0, ((SetSize as u64)) as SIZE_T) as LPWSAQUERYSETW;
    if (*UnicodeCopy).is_null() {
        return (WSA_NOT_ENOUGH_MEMORY as i32);
    }

    return WSABuildQuerySetBufferW(UnicodeSet, SetSize, *UnicodeCopy);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/qshelpr.c
//  * PURPOSE:     Query Set Conversion/Packing Helpers
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
// /* FUNCTIONS *****************************************************************/
//
// LPSTR
// __stdcall
// AnsiDupFromUnicode(IN LPCWSTR UnicodeString)
// {
//     INT Length = 0;
//     BOOL GoOn = TRUE;
//     LPSTR DuplicatedString = NULL;
//     INT ReturnValue;
//
//     /* Start a loop (which should only run twice) */
//     while (GoOn)
//     {
//         /* Call the conversion function */
//         ReturnValue = WideCharToMultiByte(CP_ACP,
//                                           0,
//                                           UnicodeString,
//                                           -1,
//                                           DuplicatedString,
//                                           Length,
//                                           NULL,
//                                           NULL);
//         if (ReturnValue > Length)
//         {
//             /* This is the first loop, and we have the real size now */
//             Length = ReturnValue;
//
//             /* Allocate buffer for it */
//             DuplicatedString = HeapAlloc(WsSockHeap, 0, Length);
//             if (!DuplicatedString) GoOn = FALSE;
//         }
//         else if (ReturnValue > 0)
//         {
//             /* The second loop was successful and we have the string */
//             GoOn = FALSE;
//         }
//         else
//         {
//             /* Some weird error happened */
//             if (DuplicatedString) HeapFree(WsSockHeap, 0, DuplicatedString);
//             DuplicatedString = NULL;
//             GoOn = FALSE;
//         }
//     }
//
//     /* Return the duplicate */
//     return DuplicatedString;
// }
//
// LPWSTR
// __stdcall
// UnicodeDupFromAnsi(IN LPCSTR AnsiString)
// {
//     INT Length = 0;
//     BOOL GoOn = TRUE;
//     LPWSTR DuplicatedString = NULL;
//     INT ReturnValue;
//
//     /* Start a loop (which should only run twice) */
//     while (GoOn)
//     {
//         /* Call the conversion function */
//         ReturnValue = MultiByteToWideChar(CP_ACP,
//                                           0,
//                                           AnsiString,
//                                           -1,
//                                           DuplicatedString,
//                                           Length);
//         if (ReturnValue > Length)
//         {
//             /* This is the first loop, and we have the real size now */
//             Length = ReturnValue;
//
//             /* Allocate buffer for it */
//             DuplicatedString = HeapAlloc(WsSockHeap, 0, Length * sizeof(WCHAR));
//             if (!DuplicatedString) GoOn = FALSE;
//         }
//         else if (ReturnValue > 0)
//         {
//             /* The second loop was successful and we have the string */
//             GoOn = FALSE;
//         }
//         else
//         {
//             /* Some weird error happened */
//             if (DuplicatedString) HeapFree(WsSockHeap, 0, DuplicatedString);
//             DuplicatedString = NULL;
//             GoOn = FALSE;
//         }
//     }
//
//     /* Return the duplicate */
//     return DuplicatedString;
// }
//
// SIZE_T
// __stdcall
// ComputeStringSize(IN LPSTR String,
//                   IN BOOLEAN IsUnicode)
// {
//     /* Return the size of the string, in bytes, including null-char */
//     return (IsUnicode) ? (wcslen((LPWSTR)String) + 1 ) * sizeof(WCHAR) :
//                           strlen(String) + sizeof(CHAR);
// }
//
// SIZE_T
// __stdcall
// ComputeQuerySetSize(IN LPWSAQUERYSETA AnsiSet,
//                     IN BOOLEAN IsUnicode)
// {
//     SIZE_T Size = sizeof(WSAQUERYSETA);
//     //LPWSAQUERYSETW UnicodeSet = (LPWSAQUERYSETW)AnsiSet;
//     DWORD i;
//
//     /* Check for instance name */
//     if (AnsiSet->lpszServiceInstanceName)
//     {
//         /* Add its size */
//         Size += ComputeStringSize(AnsiSet->lpszServiceInstanceName, IsUnicode);
//     }
//
//     /* Check for Service Class ID */
//     if (AnsiSet->lpServiceClassId)
//     {
//         /* Align the current size and add GUID size */
//         Size = (Size + 3) & ~3;
//         Size += sizeof(GUID);
//     }
//
//     /* Check for version data */
//     if (AnsiSet->lpVersion)
//     {
//         /* Align the current size and add GUID size */
//         Size = (Size + 3) & ~3;
//         Size += sizeof(WSAVERSION);
//     }
//
//     /* Check for comment */
//     if (AnsiSet->lpszComment)
//     {
//         /* Align the current size and add string size */
//         Size = (Size + 1) & ~1;
//         Size += ComputeStringSize(AnsiSet->lpszComment, IsUnicode);
//     }
//
//     /* Check for Provider ID */
//     if (AnsiSet->lpNSProviderId)
//     {
//         /* Align the current size and add GUID size */
//         Size = (Size + 3) & ~3;
//         Size += sizeof(GUID);
//     }
//
//     /* Check for context */
//     if (AnsiSet->lpszContext)
//     {
//         /* Align the current size and add string size */
//         Size = (Size + 1) & ~1;
//         Size += ComputeStringSize(AnsiSet->lpszContext, IsUnicode);
//     }
//
//     /* Check for query string */
//     if (AnsiSet->lpszQueryString)
//     {
//         /* Align the current size and add string size */
//         Size = (Size + 1) & ~1;
//         Size += ComputeStringSize(AnsiSet->lpszQueryString, IsUnicode);
//     }
//
//     /* Check for AF Protocol data */
//     if (AnsiSet->lpafpProtocols)
//     {
//         /* Align the current size and add AFP size */
//         Size = (Size + 3) & ~3;
//         Size += sizeof(AFPROTOCOLS) * AnsiSet->dwNumberOfProtocols;
//     }
//
//     /* Check for CSADDR buffer */
//     if (AnsiSet->lpcsaBuffer)
//     {
//         /* Align the current size */
//         Size = (Size + 3) & ~3;
//
//         /* Loop all the addresses in the array */
//         for (i = 0; i < AnsiSet->dwNumberOfCsAddrs; i++)
//         {
//             /* Check for local sockaddr */
//             if (AnsiSet->lpcsaBuffer[i].LocalAddr.lpSockaddr)
//             {
//                 /* Align the current size and add the sockaddr's length */
//                 Size = (Size + 3) & ~3;
//                 Size += AnsiSet->lpcsaBuffer[i].LocalAddr.iSockaddrLength;
//             }
//             /* Check for remote sockaddr */
//             if (AnsiSet->lpcsaBuffer[i].RemoteAddr.lpSockaddr)
//             {
//                 /* Align the current size and add the sockaddr's length */
//                 Size = (Size + 3) & ~3;
//                 Size += AnsiSet->lpcsaBuffer[i].RemoteAddr.iSockaddrLength;
//             }
//
//             /* Add the sockaddr size itself */
//             Size += sizeof(CSADDR_INFO);
//         }
//     }
//
//     /* Check for blob data */
//     if (AnsiSet->lpBlob)
//     {
//         /* Align the current size and add blob size */
//         Size = (Size + 3) & ~3;
//         Size += sizeof(BLOB);
//
//         /* Also add the actual blob data size, if it exists */
//         if (AnsiSet->lpBlob) Size += AnsiSet->lpBlob->cbSize;
//     }
//
//     /* Return the total size */
//     return Size;
// }
//
// SIZE_T
// __stdcall
// WSAComputeQuerySetSizeA(IN LPWSAQUERYSETA AnsiSet)
// {
//     /* Call the generic helper */
//     return ComputeQuerySetSize(AnsiSet, FALSE);
// }
//
// SIZE_T
// __stdcall
// WSAComputeQuerySetSizeW(IN LPWSAQUERYSETW UnicodeSet)
// {
//     /* Call the generic helper */
//     return ComputeQuerySetSize((LPWSAQUERYSETA)UnicodeSet, TRUE);
// }
//
// PVOID
// __stdcall
// WsBufferAllocate(IN PWS_BUFFER Buffer,
//                  IN SIZE_T Size,
//                  IN DWORD Align)
// {
//     PVOID NewPosition;
//
//     /* Align the current usage */
//     Buffer->BytesUsed = (Buffer->BytesUsed + Align - 1) & ~(Align - 1);
//
//     /* Update our location */
//     NewPosition = (PVOID)(Buffer->Position + Buffer->BytesUsed);
//
//     /* Update the usage */
//     Buffer->BytesUsed += Size;
//
//     /* Return new location */
//     return NewPosition;
// }
//
// VOID
// __stdcall
// CopyBlobIndirect(IN PWS_BUFFER Buffer,
//                  IN OUT LPBLOB RelativeBlob,
//                  IN LPBLOB Blob)
// {
//     /* Make sure we have blob data */
//     if ((Blob->pBlobData) && (Blob->cbSize))
//     {
//         /* Allocate and copy the blob data */
//         RelativeBlob->pBlobData = WsBufferAllocate(Buffer,
//                                                    Blob->cbSize,
//                                                    sizeof(PVOID));
//         RtlCopyMemory(RelativeBlob->pBlobData,
//                       Blob->pBlobData,
//                       Blob->cbSize);
//     }
// }
//
// VOID
// __stdcall
// CopyAddrInfoArrayIndirect(IN PWS_BUFFER Buffer,
//                           IN OUT LPCSADDR_INFO RelativeCsaddr,
//                           IN DWORD Addresses,
//                           IN LPCSADDR_INFO Csaddr)
// {
//     DWORD i;
//
//     /* Loop for every address inside */
//     for (i = 0; i < Addresses; i++)
//     {
//         /* Check for a local address */
//         if ((Csaddr[i].LocalAddr.lpSockaddr) &&
//             (Csaddr[i].LocalAddr.iSockaddrLength))
//         {
//             /* Allocate and copy the address */
//             RelativeCsaddr[i].LocalAddr.lpSockaddr =
//                 WsBufferAllocate(Buffer,
//                                  Csaddr[i].LocalAddr.iSockaddrLength,
//                                  sizeof(PVOID));
//             RtlCopyMemory(RelativeCsaddr[i].LocalAddr.lpSockaddr,
//                           Csaddr[i].LocalAddr.lpSockaddr,
//                           Csaddr[i].LocalAddr.iSockaddrLength);
//         }
//         else
//         {
//             /* Nothing in this address */
//             Csaddr[i].LocalAddr.lpSockaddr = NULL;
//             Csaddr[i].LocalAddr.iSockaddrLength = 0;
//         }
//
//         /* Check for a remote address */
//         if ((Csaddr[i].RemoteAddr.lpSockaddr) &&
//             (Csaddr[i].RemoteAddr.iSockaddrLength))
//         {
//             /* Allocate and copy the address */
//             RelativeCsaddr[i].RemoteAddr.lpSockaddr =
//                 WsBufferAllocate(Buffer,
//                                  Csaddr[i].RemoteAddr.iSockaddrLength,
//                                  sizeof(PVOID));
//             RtlCopyMemory(RelativeCsaddr[i].RemoteAddr.lpSockaddr,
//                           Csaddr[i].RemoteAddr.lpSockaddr,
//                           Csaddr[i].RemoteAddr.iSockaddrLength);
//         }
//         else
//         {
//             /* Nothing in this address */
//             Csaddr[i].RemoteAddr.lpSockaddr = NULL;
//             Csaddr[i].RemoteAddr.iSockaddrLength = 0;
//         }
//     }
// }
//
// VOID
// __stdcall
// CopyQuerySetIndirectA(IN PWS_BUFFER Buffer,
//                       IN OUT LPWSAQUERYSETA RelativeSet,
//                       IN LPWSAQUERYSETA AnsiSet)
// {
//     LPSTR AnsiString;
//
//     /* Get the service name */
//     AnsiString = AnsiSet->lpszServiceInstanceName;
//     if (AnsiString)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpszServiceInstanceName = WsBufferAllocate(Buffer,
//                                                                 strlen(AnsiString) + 1,
//                                                                 sizeof(CHAR));
//         /* Copy it into the buffer */
//         strcpy(RelativeSet->lpszServiceInstanceName, AnsiString);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpszServiceInstanceName = NULL;
//     }
//
//     /* Check for the service class ID */
//     if (AnsiSet->lpServiceClassId)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpServiceClassId = WsBufferAllocate(Buffer,
//                                                          sizeof(GUID),
//                                                          sizeof(PVOID));
//         /* Copy it into the buffer */
//         *(RelativeSet->lpServiceClassId) = *(AnsiSet->lpServiceClassId);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpServiceClassId = NULL;
//     }
//
//     /* Get the version data */
//     if (AnsiSet->lpVersion)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpVersion = WsBufferAllocate(Buffer,
//                                                   sizeof(WSAVERSION),
//                                                   sizeof(PVOID));
//         /* Copy it into the buffer */
//         *(RelativeSet->lpVersion) = *(AnsiSet->lpVersion);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpVersion = NULL;
//     }
//
//     /* Get the comment */
//     AnsiString = AnsiSet->lpszComment;
//     if (AnsiString)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpszComment = WsBufferAllocate(Buffer,
//                                                     strlen(AnsiString) + 1,
//                                                     sizeof(CHAR));
//         /* Copy it into the buffer */
//         strcpy(RelativeSet->lpszComment, AnsiString);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpszComment = NULL;
//     }
//
//     /* Get the NS Provider ID */
//     if (AnsiSet->lpNSProviderId)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpNSProviderId = WsBufferAllocate(Buffer,
//                                                        sizeof(GUID),
//                                                        sizeof(PVOID));
//         /* Copy it into the buffer */
//         *(RelativeSet->lpNSProviderId) = *(AnsiSet->lpNSProviderId);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpNSProviderId = NULL;
//     }
//
//     /* Get the context */
//     AnsiString = AnsiSet->lpszContext;
//     if (AnsiString)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpszContext = WsBufferAllocate(Buffer,
//                                                     strlen(AnsiString) + 1,
//                                                     sizeof(CHAR));
//         /* Copy it into the buffer */
//         strcpy(RelativeSet->lpszContext, AnsiString);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpszContext = NULL;
//     }
//
//     /* Get the query string */
//     AnsiString = AnsiSet->lpszQueryString;
//     if (AnsiString)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpszQueryString = WsBufferAllocate(Buffer,
//                                                         strlen(AnsiString) + 1,
//                                                         sizeof(CHAR));
//         /* Copy it into the buffer */
//         strcpy(RelativeSet->lpszQueryString, AnsiString);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpszQueryString = NULL;
//     }
//
//     /* Check for a protocol structure with non-zero protocols */
//     if ((AnsiSet->lpafpProtocols) && (AnsiSet->dwNumberOfProtocols))
//     {
//         /* One exists, allocate space for it */
//         RelativeSet->lpafpProtocols = WsBufferAllocate(Buffer,
//                                                        AnsiSet->dwNumberOfProtocols *
//                                                        sizeof(AFPROTOCOLS),
//                                                        sizeof(PVOID));
//         /* Copy it into the buffer */
//         RtlCopyMemory(RelativeSet->lpafpProtocols,
//                       AnsiSet->lpafpProtocols,
//                       AnsiSet->dwNumberOfProtocols * sizeof(AFPROTOCOLS));
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpafpProtocols = NULL;
//         RelativeSet->dwNumberOfProtocols = 0;
//     }
//
//     /* Check if we have a CSADDR with addresses inside */
//     if ((AnsiSet->lpcsaBuffer) && (AnsiSet->dwNumberOfCsAddrs))
//     {
//         /* Allocate and copy the CSADDR structure itself */
//         RelativeSet->lpcsaBuffer = WsBufferAllocate(Buffer,
//                                                     AnsiSet->dwNumberOfCsAddrs *
//                                                     sizeof(CSADDR_INFO),
//                                                     sizeof(PVOID));
//
//         /* Copy it into the buffer */
//         RtlCopyMemory(RelativeSet->lpcsaBuffer,
//                       AnsiSet->lpcsaBuffer,
//                       AnsiSet->dwNumberOfCsAddrs * sizeof(CSADDR_INFO));
//
//         /* Copy the addresses inside the CSADDR */
//         CopyAddrInfoArrayIndirect(Buffer,
//                                   RelativeSet->lpcsaBuffer,
//                                   AnsiSet->dwNumberOfCsAddrs,
//                                   AnsiSet->lpcsaBuffer);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpcsaBuffer = NULL;
//         RelativeSet->dwNumberOfCsAddrs = 0;
//     }
//
//     /* Check for blob data */
//     if (AnsiSet->lpBlob)
//     {
//         /* Allocate and copy the blob itself */
//         RelativeSet->lpBlob = WsBufferAllocate(Buffer,
//                                                sizeof(BLOB),
//                                                sizeof(PVOID));
//         *(RelativeSet->lpBlob) = *(AnsiSet->lpBlob);
//
//         /* Copy the data inside the blob */
//         CopyBlobIndirect(Buffer, RelativeSet->lpBlob, AnsiSet->lpBlob);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpBlob = NULL;
//     }
// }
//
// VOID
// __stdcall
// CopyQuerySetIndirectW(IN PWS_BUFFER Buffer,
//                       IN OUT LPWSAQUERYSETW RelativeSet,
//                       IN LPWSAQUERYSETW UnicodeSet)
// {
//     LPWSTR UnicodeString;
//
//     /* Get the service name */
//     UnicodeString = UnicodeSet->lpszServiceInstanceName;
//     if (UnicodeString)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpszServiceInstanceName = WsBufferAllocate(Buffer,
//                                                                 (wcslen(UnicodeString) + 1) *
//                                                                 sizeof(WCHAR),
//                                                                 sizeof(CHAR));
//         /* Copy it into the buffer */
//         wcscpy(RelativeSet->lpszServiceInstanceName, UnicodeString);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpszServiceInstanceName = NULL;
//     }
//
//     /* Check for the service class ID */
//     if (UnicodeSet->lpServiceClassId)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpServiceClassId = WsBufferAllocate(Buffer,
//                                                          sizeof(GUID),
//                                                          sizeof(PVOID));
//         /* Copy it into the buffer */
//         *(RelativeSet->lpServiceClassId) = *(UnicodeSet->lpServiceClassId);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpServiceClassId = NULL;
//     }
//
//     /* Get the version data */
//     if (UnicodeSet->lpVersion)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpVersion = WsBufferAllocate(Buffer,
//                                                   sizeof(WSAVERSION),
//                                                   sizeof(PVOID));
//         /* Copy it into the buffer */
//         *(RelativeSet->lpVersion) = *(UnicodeSet->lpVersion);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpVersion = NULL;
//     }
//
//     /* Get the comment */
//     UnicodeString = UnicodeSet->lpszComment;
//     if (UnicodeString)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpszComment = WsBufferAllocate(Buffer,
//                                                     (wcslen(UnicodeString) + 1) *
//                                                     sizeof(WCHAR),
//                                                     sizeof(CHAR));
//         /* Copy it into the buffer */
//         wcscpy(RelativeSet->lpszComment, UnicodeString);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpszComment = NULL;
//     }
//
//     /* Get the NS Provider ID */
//     if (UnicodeSet->lpNSProviderId)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpNSProviderId = WsBufferAllocate(Buffer,
//                                                        sizeof(GUID),
//                                                        sizeof(PVOID));
//         /* Copy it into the buffer */
//         *(RelativeSet->lpNSProviderId) = *(UnicodeSet->lpNSProviderId);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpNSProviderId = NULL;
//     }
//
//     /* Get the context */
//     UnicodeString = UnicodeSet->lpszContext;
//     if (UnicodeString)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpszContext = WsBufferAllocate(Buffer,
//                                                     (wcslen(UnicodeString) + 1) *
//                                                     sizeof(WCHAR),
//                                                     sizeof(CHAR));
//         /* Copy it into the buffer */
//         wcscpy(RelativeSet->lpszContext, UnicodeString);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpszContext = NULL;
//     }
//
//     /* Get the query string */
//     UnicodeString = UnicodeSet->lpszQueryString;
//     if (UnicodeString)
//     {
//         /* One exists, allocate a space in the buffer for it */
//         RelativeSet->lpszQueryString = WsBufferAllocate(Buffer,
//                                                         (wcslen(UnicodeString) + 1) *
//                                                         sizeof(WCHAR),
//                                                         sizeof(CHAR));
//         /* Copy it into the buffer */
//         wcscpy(RelativeSet->lpszQueryString, UnicodeString);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpszQueryString = NULL;
//     }
//
//     /* Check for a protocol structure with non-zero protocols */
//     if ((UnicodeSet->lpafpProtocols) && (UnicodeSet->dwNumberOfProtocols))
//     {
//         /* One exists, allocate space for it */
//         RelativeSet->lpafpProtocols = WsBufferAllocate(Buffer,
//                                                        UnicodeSet->dwNumberOfProtocols *
//                                                        sizeof(AFPROTOCOLS),
//                                                        sizeof(PVOID));
//         /* Copy it into the buffer */
//         RtlCopyMemory(RelativeSet->lpafpProtocols,
//                       UnicodeSet->lpafpProtocols,
//                       UnicodeSet->dwNumberOfProtocols * sizeof(AFPROTOCOLS));
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpafpProtocols = NULL;
//         RelativeSet->dwNumberOfProtocols = 0;
//     }
//
//     /* Check if we have a CSADDR with addresses inside */
//     if ((UnicodeSet->lpcsaBuffer) && (UnicodeSet->dwNumberOfCsAddrs))
//     {
//         /* Allocate and copy the CSADDR structure itself */
//         RelativeSet->lpcsaBuffer = WsBufferAllocate(Buffer,
//                                                     UnicodeSet->dwNumberOfCsAddrs *
//                                                     sizeof(CSADDR_INFO),
//                                                     sizeof(PVOID));
//
//         /* Copy it into the buffer */
//         RtlCopyMemory(RelativeSet->lpcsaBuffer,
//                       UnicodeSet->lpcsaBuffer,
//                       UnicodeSet->dwNumberOfCsAddrs * sizeof(CSADDR_INFO));
//
//         /* Copy the addresses inside the CSADDR */
//         CopyAddrInfoArrayIndirect(Buffer,
//                                   RelativeSet->lpcsaBuffer,
//                                   UnicodeSet->dwNumberOfCsAddrs,
//                                   UnicodeSet->lpcsaBuffer);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpcsaBuffer = NULL;
//         RelativeSet->dwNumberOfCsAddrs = 0;
//     }
//
//     /* Check for blob data */
//     if (UnicodeSet->lpBlob)
//     {
//         /* Allocate and copy the blob itself */
//         RelativeSet->lpBlob = WsBufferAllocate(Buffer,
//                                                sizeof(BLOB),
//                                                sizeof(PVOID));
//         *(RelativeSet->lpBlob) = *(UnicodeSet->lpBlob);
//
//         /* Copy the data inside the blob */
//         CopyBlobIndirect(Buffer, RelativeSet->lpBlob, UnicodeSet->lpBlob);
//     }
//     else
//     {
//         /* Nothing in the buffer */
//         RelativeSet->lpBlob = NULL;
//     }
// }
//
// INT
// __stdcall
// WSABuildQuerySetBufferA(IN LPWSAQUERYSETA AnsiSet,
//                         IN SIZE_T BufferSize,
//                         OUT LPWSAQUERYSETA RelativeSet)
// {
//     INT ErrorCode = ERROR_SUCCESS;
//     SIZE_T SetSize;
//     WS_BUFFER Buffer;
//     LPWSAQUERYSETA NewSet;
//
//     /* Find out how big the set really is */
//     SetSize = WSAComputeQuerySetSizeA(AnsiSet);
//     if (SetSize <= BufferSize)
//     {
//         /* Configure the buffer */
//         Buffer.Position = (ULONG_PTR)RelativeSet;
//         Buffer.MaxSize = SetSize;
//         Buffer.BytesUsed = 0;
//
//         /* Copy the set itself into the buffer */
//         NewSet = WsBufferAllocate(&Buffer, sizeof(*AnsiSet), sizeof(PVOID));
//         *NewSet = *AnsiSet;
//
//         /* Now copy the data inside */
//         CopyQuerySetIndirectA(&Buffer, NewSet, AnsiSet);
//     }
//     else
//     {
//         /* We failed */
//         ErrorCode = SOCKET_ERROR;
//     }
//
//     /* Return to caller */
//     return ErrorCode;
// }
//
// INT
// __stdcall
// WSABuildQuerySetBufferW(IN LPWSAQUERYSETW UnicodeSet,
//                         IN SIZE_T BufferSize,
//                         OUT LPWSAQUERYSETW RelativeSet)
// {
//     INT ErrorCode = ERROR_SUCCESS;
//     SIZE_T SetSize;
//     WS_BUFFER Buffer;
//     LPWSAQUERYSETW NewSet;
//
//     /* Find out how big the set really is */
//     SetSize = WSAComputeQuerySetSizeW(UnicodeSet);
//     if (SetSize <= BufferSize)
//     {
//         /* Configure the buffer */
//         Buffer.Position = (ULONG_PTR)RelativeSet;
//         Buffer.MaxSize = SetSize;
//         Buffer.BytesUsed = 0;
//
//         /* Copy the set itself into the buffer */
//         NewSet = WsBufferAllocate(&Buffer, sizeof(*UnicodeSet), sizeof(PVOID));
//         *NewSet = *UnicodeSet;
//
//         /* Now copy the data inside */
//         CopyQuerySetIndirectW(&Buffer, NewSet, UnicodeSet);
//     }
//     else
//     {
//         /* We failed */
//         ErrorCode = SOCKET_ERROR;
//     }
//
//     /* Return to caller */
//     return ErrorCode;
// }
//
// INT
// __stdcall
// MapAnsiQuerySetToUnicode(IN LPWSAQUERYSETA AnsiSet,
//                          IN OUT PSIZE_T SetSize,
//                          OUT LPWSAQUERYSETW UnicodeSet)
// {
//     INT ErrorCode = ERROR_SUCCESS;
//     SIZE_T AnsiSize, UnicodeSize;
//     LPWSAQUERYSETA AnsiCopy = NULL;
//     LPWSAQUERYSETW UnicodeCopy;
//     LPWSTR ServiceCopy = NULL, CommentCopy = NULL;
//     LPWSTR ContextCopy = NULL, QueryCopy = NULL;
//
//     /* Calculate the size of the Ansi version and allocate space for a copy */
//     AnsiSize = WSAComputeQuerySetSizeA(AnsiSet);
//     AnsiCopy = HeapAlloc(WsSockHeap, 0, AnsiSize);
//     if (!AnsiCopy)
//     {
//         /* Fail, couldn't allocate memory */
//         ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//         goto Exit;
//     }
//
//     /* Build the relative buffer version */
//     ErrorCode = WSABuildQuerySetBufferA(AnsiSet, AnsiSize, AnsiCopy);
//     if (ErrorCode != ERROR_SUCCESS) goto Exit;
//
//     /* Re-use the ANSI version since the fields match */
//     UnicodeCopy = (LPWSAQUERYSETW)AnsiCopy;
//
//     /* Check if we have a service instance name */
//     if (AnsiCopy->lpszServiceInstanceName)
//     {
//         /* Duplicate it into unicode form */
//         ServiceCopy = UnicodeDupFromAnsi(AnsiCopy->lpszServiceInstanceName);
//         if (!ServiceCopy)
//         {
//             /* Fail */
//             ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//
//         /* Set the new string pointer */
//         UnicodeCopy->lpszServiceInstanceName = ServiceCopy;
//     }
//
//     /* Check if we have a service instance name */
//     if (AnsiCopy->lpszContext)
//     {
//         /* Duplicate it into unicode form */
//         ContextCopy = UnicodeDupFromAnsi(AnsiCopy->lpszContext);
//         if (!ContextCopy)
//         {
//             /* Fail */
//             ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//
//         /* Set the new string pointer */
//         UnicodeCopy->lpszContext = ContextCopy;
//     }
//
//     /* Check if we have a service instance name */
//     if (AnsiCopy->lpszComment)
//     {
//         /* Duplicate it into unicode form */
//         CommentCopy = UnicodeDupFromAnsi(AnsiCopy->lpszComment);
//         if (!CommentCopy)
//         {
//             /* Fail */
//             ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//
//         /* Set the new string pointer */
//         UnicodeCopy->lpszComment = CommentCopy;
//     }
//
//     /* Check if we have a query name */
//     if (AnsiCopy->lpszQueryString)
//     {
//         /* Duplicate it into unicode form */
//         QueryCopy = UnicodeDupFromAnsi(AnsiCopy->lpszQueryString);
//         if (!QueryCopy)
//         {
//             /* Fail */
//             ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//
//         /* Set the new string pointer */
//         UnicodeCopy->lpszQueryString = QueryCopy;
//     }
//
//     /* Now that we have the absolute unicode buffer, calculate its size */
//     UnicodeSize = WSAComputeQuerySetSizeW(UnicodeCopy);
//     if (UnicodeSize > *SetSize)
//     {
//         /* The buffer wasn't large enough; return how much we need */
//         *SetSize = UnicodeSize;
//         ErrorCode = WSAEFAULT;
//         goto Exit;
//     }
//
//     /* Build the relative unicode buffer */
//     ErrorCode = WSABuildQuerySetBufferW(UnicodeCopy, *SetSize, UnicodeSet);
//
// Exit:
//     /* Free the Ansi copy if we had one */
//     if (AnsiCopy) HeapFree(WsSockHeap, 0, AnsiCopy);
//
//     /* Free all the strings */
//     if (ServiceCopy) HeapFree(WsSockHeap, 0, ServiceCopy);
//     if (CommentCopy) HeapFree(WsSockHeap, 0, CommentCopy);
//     if (ContextCopy) HeapFree(WsSockHeap, 0, ContextCopy);
//     if (QueryCopy) HeapFree(WsSockHeap, 0, QueryCopy);
//
//     /* Return error code */
//     return ErrorCode;
// }
//
// INT
// __stdcall
// MapUnicodeQuerySetToAnsi(IN LPWSAQUERYSETW UnicodeSet,
//                          IN OUT PSIZE_T SetSize,
//                          OUT LPWSAQUERYSETA AnsiSet)
// {
//     INT ErrorCode = ERROR_SUCCESS;
//     SIZE_T UnicodeSize, AnsiSize;
//     LPWSAQUERYSETW UnicodeCopy = NULL;
//     LPWSAQUERYSETA AnsiCopy;
//     LPSTR ServiceCopy = NULL, CommentCopy = NULL;
//     LPSTR ContextCopy = NULL, QueryCopy = NULL;
//
//     /* Calculate the size of the Ansi version and allocate space for a copy */
//     UnicodeSize = WSAComputeQuerySetSizeW(UnicodeSet);
//     UnicodeCopy = HeapAlloc(WsSockHeap, 0, UnicodeSize);
//     if (!UnicodeCopy)
//     {
//         /* Fail, couldn't allocate memory */
//         ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//         goto Exit;
//     }
//
//     /* Build the relative buffer version */
//     ErrorCode = WSABuildQuerySetBufferW(UnicodeSet, UnicodeSize, UnicodeCopy);
//     if (ErrorCode != ERROR_SUCCESS) goto Exit;
//
//     /* Re-use the Unicode version since the fields match */
//     AnsiCopy = (LPWSAQUERYSETA)UnicodeCopy;
//
//     /* Check if we have a service instance name */
//     if (UnicodeCopy->lpszServiceInstanceName)
//     {
//         /* Duplicate it into unicode form */
//         ServiceCopy = AnsiDupFromUnicode(UnicodeCopy->lpszServiceInstanceName);
//         if (!ServiceCopy)
//         {
//             /* Fail */
//             ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//
//         /* Set the new string pointer */
//         AnsiCopy->lpszServiceInstanceName = ServiceCopy;
//     }
//
//     /* Check if we have a service instance name */
//     if (UnicodeCopy->lpszContext)
//     {
//         /* Duplicate it into unicode form */
//         ContextCopy = AnsiDupFromUnicode(UnicodeCopy->lpszContext);
//         if (!ContextCopy)
//         {
//             /* Fail */
//             ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//
//         /* Set the new string pointer */
//         AnsiCopy->lpszContext = ContextCopy;
//     }
//
//     /* Check if we have a service instance name */
//     if (UnicodeCopy->lpszComment)
//     {
//         /* Duplicate it into unicode form */
//         CommentCopy = AnsiDupFromUnicode(UnicodeCopy->lpszComment);
//         if (!CommentCopy)
//         {
//             /* Fail */
//             ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//
//         /* Set the new string pointer */
//         AnsiCopy->lpszComment = CommentCopy;
//     }
//
//     /* Check if we have a query name */
//     if (UnicodeCopy->lpszQueryString)
//     {
//         /* Duplicate it into unicode form */
//         QueryCopy = AnsiDupFromUnicode(UnicodeCopy->lpszQueryString);
//         if (!QueryCopy)
//         {
//             /* Fail */
//             ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//
//         /* Set the new string pointer */
//         AnsiCopy->lpszQueryString = QueryCopy;
//     }
//
//     /* Now that we have the absolute unicode buffer, calculate its size */
//     AnsiSize = WSAComputeQuerySetSizeA(AnsiCopy);
//     if (AnsiSize > *SetSize)
//     {
//         /* The buffer wasn't large enough; return how much we need */
//         *SetSize = AnsiSize;
//         ErrorCode = WSAEFAULT;
//         goto Exit;
//     }
//
//     /* Build the relative unicode buffer */
//     ErrorCode = WSABuildQuerySetBufferA(AnsiCopy, *SetSize, AnsiSet);
//
// Exit:
//     /* Free the Ansi copy if we had one */
//     if (UnicodeCopy) HeapFree(WsSockHeap, 0, UnicodeCopy);
//
//     /* Free all the strings */
//     if (ServiceCopy) HeapFree(WsSockHeap, 0, ServiceCopy);
//     if (CommentCopy) HeapFree(WsSockHeap, 0, CommentCopy);
//     if (ContextCopy) HeapFree(WsSockHeap, 0, ContextCopy);
//     if (QueryCopy) HeapFree(WsSockHeap, 0, QueryCopy);
//
//     /* Return error code */
//     return ErrorCode;
// }
//
// INT
// __stdcall
// CopyQuerySetW(IN LPWSAQUERYSETW UnicodeSet,
//               OUT LPWSAQUERYSETW *UnicodeCopy)
// {
//     SIZE_T SetSize;
//
//     /* Get the size */
//     SetSize = WSAComputeQuerySetSizeW(UnicodeSet);
//
//     /* Allocate memory for copy */
//     *UnicodeCopy = HeapAlloc(WsSockHeap, 0, SetSize);
//     if (!(*UnicodeCopy)) return WSA_NOT_ENOUGH_MEMORY;
//
//     /* Build a copy and return */
//     return WSABuildQuerySetBufferW(UnicodeSet, SetSize, *UnicodeCopy);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
