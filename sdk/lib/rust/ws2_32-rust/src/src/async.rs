use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::bhook::WSACancelBlockingCall;
use crate::src::bhook::WSASetBlockingHook;
use crate::src::getproto::getprotobyname;
use crate::src::getproto::getprotobynumber;
use crate::src::getxbyxx::gethostbyaddr;
use crate::src::getxbyxx::gethostbyname;
use crate::src::getxbyxx::getservbyname;
use crate::src::getxbyxx::getservbyport;
use crate::src::startup::WSACleanup;


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

pub static mut WsAsyncThreadInitialized: BOOLEAN = 0;

pub static mut WsAsyncTaskHandle: LONG = 0;

pub static mut WsAsyncQueue: PLIST_ENTRY = ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY;

pub static mut WsAsyncCritSect: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

pub static mut WsAsyncEvent: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub static mut WsAsyncCurrentTaskHandle: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub static mut WsAsyncCancelledTaskHandle: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub static mut WsAsyncDllHandle: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub unsafe extern "system" fn WsAsyncGlobalInitialize() {
    InitializeCriticalSection((core::ptr::addr_of_mut!(WsAsyncCritSect) as *mut c_void));
}

#[no_mangle]
pub unsafe extern "system" fn WsAsyncGlobalTerminate() {
    DeleteCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
}
#[no_mangle]
pub unsafe extern "system" fn BytesInHostent(Hostent: PHOSTENT) -> usize {
    let mut Bytes: usize;
    let mut i: i32;

    Bytes = core::mem::size_of::<HOSTENT>()
        .wrapping_add(strlen((*Hostent).h_name) as usize)
        .wrapping_add(core::mem::size_of::<i8>());

    Bytes = Bytes.wrapping_add(2 * core::mem::size_of::<usize>());

    i = 0;
    while !(*(*Hostent).h_aliases.offset(i as isize)).is_null() {
        Bytes = Bytes.wrapping_add(
            (strlen(*(*Hostent).h_aliases.offset(i as isize)) as usize)
                .wrapping_add(core::mem::size_of::<i8>())
                .wrapping_add(core::mem::size_of::<usize>())
        );
        i += 1;
    }

    i = 0;
    while !(*(*Hostent).h_addr_list.offset(i as isize)).is_null() {
        Bytes = Bytes.wrapping_add(
            ((*Hostent).h_length as usize)
                .wrapping_add(core::mem::size_of::<usize>())
        );
        i += 1;
    }

    (Bytes.wrapping_add(7)) & !7
}
#[no_mangle]
pub unsafe extern "system" fn BytesInServent(Servent: PSERVENT) -> usize {
    let mut Bytes: usize;
    let mut i: i32;

    Bytes = core::mem::size_of::<SERVENT>()
        .wrapping_add(strlen((*Servent).s_name))
        .wrapping_add(core::mem::size_of::<i8>())
        .wrapping_add(strlen((*Servent).s_proto))
        .wrapping_add(core::mem::size_of::<i8>());

    Bytes = Bytes.wrapping_add(core::mem::size_of::<usize>());

    i = 0;
    while !(*(*Servent).s_aliases.offset(i as isize)).is_null() {
        Bytes = Bytes.wrapping_add(
            strlen(*(*Servent).s_aliases.offset(i as isize))
                .wrapping_add(core::mem::size_of::<i8>())
                .wrapping_add(core::mem::size_of::<usize>())
        );
        i += 1;
    }

    return Bytes;
}
#[no_mangle]
pub unsafe extern "system" fn BytesInProtoent(Protoent: PPROTOENT) -> usize {

    let mut Bytes: usize;
    let mut i: i32;

    Bytes = core::mem::size_of::<SERVENT>() + strlen((*Protoent).p_name) + core::mem::size_of::<i8>();

    Bytes += core::mem::size_of::<usize>();

    i = 0;
    while !(*(*Protoent).p_aliases.offset(i as isize)).is_null() {
        Bytes += strlen(*(*Protoent).p_aliases.offset(i as isize)) + core::mem::size_of::<i8>() + core::mem::size_of::<usize>();
        i += 1;
    }

    Bytes
}
#[no_mangle]
pub unsafe extern "system" fn CopyHostentToBuffer(
    Buffer: PCHAR,
    BufferLength: INT,
    Hostent: PHOSTENT,
) -> usize {

    let mut BufferSize: usize;
    let mut CurrentSize: usize;
    let mut NameSize: usize;
    let mut p: PCHAR = Buffer;
    let mut Aliases: u32 = 0;
    let mut Names: u32 = 0;
    let mut i: u32;
    let ReturnedHostent: PHOSTENT = Buffer as PHOSTENT;

    BufferSize = BytesInHostent(Hostent);

    if (BufferLength as u32) as u64 > BufferSize as u64 {
        core::ptr::write_bytes(Buffer, 0, BufferSize);
    } else {
        core::ptr::write_bytes(Buffer, 0, BufferLength as usize);
    }

    CurrentSize = core::mem::size_of::<HOSTENT>();

    if CurrentSize as u64 > (BufferLength as u32) as u64 {
        return BufferSize;
    }

    CopyMemory(
        p as *mut core::ffi::c_void,
        Hostent as *const core::ffi::c_void,
        core::mem::size_of::<HOSTENT>(),
    );
    p = Buffer.offset(CurrentSize as isize);
    (*ReturnedHostent).h_name = core::ptr::null_mut();
    (*ReturnedHostent).h_aliases = core::ptr::null_mut();
    (*ReturnedHostent).h_addr_list = core::ptr::null_mut();

    while !(*(*Hostent).h_aliases.offset(Aliases as isize)).is_null() {
        Aliases = Aliases.wrapping_add(1);
    }

    CurrentSize = (CurrentSize as u64).wrapping_add(
        ((Aliases.wrapping_add(1)) as usize)
            .wrapping_mul(core::mem::size_of::<usize>()) as u64,
    ) as usize;
    if CurrentSize as u64 > (BufferLength as u32) as u64 {
        (*Hostent).h_aliases = core::ptr::null_mut();
        return BufferSize;
    }

    (*ReturnedHostent).h_aliases = p as *mut PCHAR;
    p = Buffer.offset(CurrentSize as isize);

    while !(*(*Hostent).h_addr_list.offset(Names as isize)).is_null() {
        Names = Names.wrapping_add(1);
    }

    CurrentSize = (CurrentSize as u64).wrapping_add(
        ((Names.wrapping_add(1)) as usize)
            .wrapping_mul(core::mem::size_of::<usize>()) as u64,
    ) as usize;
    if CurrentSize as u64 > (BufferLength as u32) as u64 {
        (*Hostent).h_addr_list = core::ptr::null_mut();
        return BufferSize;
    }

    (*ReturnedHostent).h_addr_list = p as *mut PCHAR;
    p = Buffer.offset(CurrentSize as isize);

    i = 0;
    while i < Names {
        CurrentSize = (CurrentSize as u64)
            .wrapping_add((*Hostent).h_length as u64) as usize;
        if CurrentSize as u64 > (BufferLength as u32) as u64 {
            return BufferSize;
        }

        *(*ReturnedHostent).h_addr_list.offset(i as isize) = p;
        CopyMemory(
            p as *mut core::ffi::c_void,
            *(*Hostent).h_addr_list.offset(i as isize) as *const core::ffi::c_void,
            (*Hostent).h_length as usize,
        );

        p = Buffer.offset(CurrentSize as isize);
        i = i.wrapping_add(1);
    }

    *(*ReturnedHostent).h_addr_list.offset(i as isize) = core::ptr::null_mut();

    NameSize = strlen((*Hostent).h_name).wrapping_add(core::mem::size_of::<i8>());
    CurrentSize = (CurrentSize as u64).wrapping_add(NameSize as u64) as usize;
    if CurrentSize as u64 > (BufferLength as u32) as u64 {
        return BufferSize;
    }

    (*ReturnedHostent).h_name = p;
    CopyMemory(
        p as *mut core::ffi::c_void,
        (*Hostent).h_name as *const core::ffi::c_void,
        NameSize,
    );
    p = Buffer.offset(CurrentSize as isize);

    i = 0;
    while i < Aliases {
        NameSize = strlen(*(*Hostent).h_aliases.offset(i as isize))
            .wrapping_add(core::mem::size_of::<i8>());
        CurrentSize = (CurrentSize as u64).wrapping_add(NameSize as u64) as usize;
        if CurrentSize as u64 > (BufferLength as u32) as u64 {
            return BufferSize;
        }

        *(*ReturnedHostent).h_aliases.offset(i as isize) = p;
        CopyMemory(
            p as *mut core::ffi::c_void,
            *(*Hostent).h_aliases.offset(i as isize) as *const core::ffi::c_void,
            NameSize,
        );

        p = Buffer.offset(CurrentSize as isize);
        i = i.wrapping_add(1);
    }

    *(*ReturnedHostent).h_aliases.offset(i as isize) = core::ptr::null_mut();
    BufferSize
}
#[no_mangle]
pub unsafe extern "system" fn CopyServentToBuffer(
    Buffer: PCHAR,
    BufferLength: INT,
    Servent: PSERVENT,
) -> usize {

    let mut BufferSize: usize;
    let mut CurrentSize: usize;
    let mut NameSize: usize;
    let mut p: PCHAR = Buffer;
    let mut Aliases: u32 = 0;
    let mut i: u32;
    let ReturnedServent: PSERVENT = Buffer as PSERVENT;

    BufferSize = BytesInServent(Servent);

    if (BufferLength as u32 as u64) > BufferSize as u64 {
        core::ptr::write_bytes(Buffer as *mut u8, 0, BufferSize);
    } else {
        core::ptr::write_bytes(Buffer as *mut u8, 0, BufferLength as usize);
    }

    CurrentSize = core::mem::size_of::<SERVENT>();

    if CurrentSize > (BufferLength as u32 as u64 as usize) {
        return BufferSize;
    }

    core::ptr::copy_nonoverlapping(
        Servent as *const u8,
        p as *mut u8,
        core::mem::size_of::<SERVENT>(),
    );
    p = Buffer.offset(CurrentSize as isize);
    (*ReturnedServent).s_name = core::ptr::null_mut();
    (*ReturnedServent).s_aliases = core::ptr::null_mut();
    (*ReturnedServent).s_proto = core::ptr::null_mut();

    while !(*(*Servent).s_aliases.offset(Aliases as isize)).is_null() {
        Aliases = Aliases.wrapping_add(1);
    }

    CurrentSize = CurrentSize.wrapping_add(
        (Aliases.wrapping_add(1) as usize)
            .wrapping_mul(core::mem::size_of::<usize>()),
    );
    if CurrentSize > (BufferLength as u32 as u64 as usize) {
        (*Servent).s_aliases = core::ptr::null_mut();
        return BufferSize;
    }

    (*ReturnedServent).s_aliases = p as *mut *mut i8;
    p = Buffer.offset(CurrentSize as isize);

    NameSize = strlen((*Servent).s_name)
        .wrapping_add(core::mem::size_of::<i8>());
    CurrentSize = CurrentSize.wrapping_add(NameSize);
    if CurrentSize > (BufferLength as u32 as u64 as usize) {
        return BufferSize;
    }

    (*ReturnedServent).s_name = p;
    core::ptr::copy_nonoverlapping(
        (*Servent).s_name as *const u8,
        p as *mut u8,
        NameSize,
    );
    p = Buffer.offset(CurrentSize as isize);

    i = 0;
    while i < Aliases {
        NameSize = strlen(*(*Servent).s_aliases.offset(i as isize))
            .wrapping_add(core::mem::size_of::<i8>());
        CurrentSize = CurrentSize.wrapping_add(NameSize);
        if CurrentSize > (BufferLength as u32 as u64 as usize) {
            return BufferSize;
        }

        *(*ReturnedServent).s_aliases.offset(i as isize) = p;
        core::ptr::copy_nonoverlapping(
            *(*Servent).s_aliases.offset(i as isize) as *const u8,
            p as *mut u8,
            NameSize,
        );

        p = Buffer.offset(CurrentSize as isize);
        i = i.wrapping_add(1);
    }

    *(*ReturnedServent).s_aliases.offset(i as isize) = core::ptr::null_mut();
    BufferSize
}
#[no_mangle]
pub unsafe extern "system" fn CopyProtoentToBuffer(
    Buffer: PCHAR,
    BufferLength: i32,
    Protoent: PPROTOENT,
) -> usize {
    let mut BufferSize: usize;
    let mut CurrentSize: usize;
    let mut NameSize: usize;
    let mut p: PCHAR = Buffer;
    let mut Aliases: u32 = 0;
    let mut i: u32;
    let ReturnedProtoent: PPROTOENT = Buffer as PPROTOENT;

    BufferSize = BytesInProtoent(Protoent);

    if BufferLength as u32 as u64 > BufferSize as u64 {
        core::ptr::write_bytes(Buffer, 0, BufferSize);
    } else {
        core::ptr::write_bytes(Buffer, 0, BufferLength as usize);
    }

    CurrentSize = core::mem::size_of::<PROTOENT>();

    if CurrentSize > (BufferLength as u32 as u64 as usize) {
        return BufferSize;
    }

    core::ptr::copy_nonoverlapping(
        Protoent as *const u8,
        p as *mut u8,
        core::mem::size_of::<PROTOENT>(),
    );
    p = Buffer.offset(CurrentSize as isize);
    (*ReturnedProtoent).p_name = core::ptr::null_mut();
    (*ReturnedProtoent).p_aliases = core::ptr::null_mut();

    while !(*(*Protoent).p_aliases.offset(Aliases as isize)).is_null() {
        Aliases = Aliases.wrapping_add(1);
    }

    CurrentSize = CurrentSize.wrapping_add(
        (Aliases.wrapping_add(1) as usize)
            .wrapping_mul(core::mem::size_of::<usize>()),
    );
    if CurrentSize > (BufferLength as u32 as u64 as usize) {
        (*Protoent).p_aliases = core::ptr::null_mut();
        return BufferSize;
    }

    (*ReturnedProtoent).p_aliases = p as *mut PCHAR;
    p = Buffer.offset(CurrentSize as isize);

    NameSize = (strlen((*Protoent).p_name) as usize)
        .wrapping_add(core::mem::size_of::<i8>());
    CurrentSize = CurrentSize.wrapping_add(NameSize);
    if CurrentSize > (BufferLength as u32 as u64 as usize) {
        return BufferSize;
    }

    (*ReturnedProtoent).p_name = p;
    core::ptr::copy_nonoverlapping(
        (*Protoent).p_name as *const u8,
        p as *mut u8,
        NameSize,
    );
    p = Buffer.offset(CurrentSize as isize);

    i = 0;
    while i < Aliases {
        NameSize = (strlen(*(*Protoent).p_aliases.offset(i as isize)) as usize)
            .wrapping_add(core::mem::size_of::<i8>());
        CurrentSize = CurrentSize.wrapping_add(NameSize);
        if CurrentSize > (BufferLength as u32 as u64 as usize) {
            return BufferSize;
        }

        *(*ReturnedProtoent).p_aliases.offset(i as isize) = p;
        core::ptr::copy_nonoverlapping(
            *(*Protoent).p_aliases.offset(i as isize) as *const u8,
            p as *mut u8,
            NameSize,
        );

        p = Buffer.offset(CurrentSize as isize);
        i = i.wrapping_add(1);
    }

    *(*ReturnedProtoent).p_aliases.offset(i as isize) = core::ptr::null_mut();
    return BufferSize;
}
#[no_mangle]
pub unsafe extern "system" fn WsAsyncAllocateBlock(mut ExtraLength: usize) -> PWSASYNCBLOCK {
    let mut AsyncBlock: PWSASYNCBLOCK;

    ExtraLength += core::mem::size_of::<WSASYNCBLOCK>();

    AsyncBlock = HeapAlloc(WsSockHeap, 0, ((ExtraLength as u64)) as SIZE_T) as PWSASYNCBLOCK;

    (*AsyncBlock).TaskHandle = UlongToPtr(InterlockedIncrement(&mut WsAsyncTaskHandle) as u32);

    return AsyncBlock;
}

#[no_mangle]
pub unsafe extern "system" fn WsAsyncThreadBlockingHook() -> BOOL {
    if WsAsyncCurrentTaskHandle == WsAsyncCancelledTaskHandle {
        WSACancelBlockingCall();
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn WsAsyncFreeBlock(AsyncBlock: PWSASYNCBLOCK) {
    HeapFree(WsSockHeap, 0, AsyncBlock as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn WsAsyncGetServ(
    TaskHandle: HANDLE,
    Operation: DWORD,
    hWnd: HWND,
    wMsg: UINT,
    ByWhat: PCHAR,
    Protocol: PCHAR,
    Buffer: PCHAR,
    BufferLength: INT,
) {

    let mut Servent: PSERVENT;
    let mut BufferSize: usize = 0;
    let lParam: LPARAM;
    let mut ErrorCode: INT = 0;

    if Operation == WsAsyncGetServByName {
        Servent = getservbyname(ByWhat, Protocol);
    } else {
        Servent = getservbyport(PtrToUlong(ByWhat as *const core::ffi::c_void) as INT, Protocol);
    }

    if Servent.is_null() {
        ErrorCode = GetLastError() as INT;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

    if TaskHandle == WsAsyncCancelledTaskHandle {
        LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
        return;
    }

    if !Servent.is_null() {
        BufferSize = CopyServentToBuffer(Buffer, BufferLength, Servent);

        if BufferSize > BufferLength as usize {
            ErrorCode = (WSAENOBUFS as i32);
        } else {
            ErrorCode = (NO_ERROR as i32);
        }
    }

    WsAsyncCurrentTaskHandle = core::ptr::null_mut();

    LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

    lParam = ((BufferSize & 0xffff) as u16 as u32 | ((ErrorCode as u32 & 0xffff) as u16 as u32) << 16) as i32 as LPARAM;

    WPUPostMessage(hWnd, wMsg, TaskHandle as WPARAM, lParam);
}
#[no_mangle]
pub unsafe extern "system" fn WsAsyncGetProto(
    TaskHandle: HANDLE,
    Operation: DWORD,
    hWnd: HWND,
    wMsg: UINT,
    ByWhat: PCHAR,
    Buffer: PCHAR,
    BufferLength: INT,
) {

    let mut Protoent: PPROTOENT = core::ptr::null_mut();
    let mut BufferSize: usize = 0;
    let mut lParam: LPARAM;
    let mut ErrorCode: i32 = 0;

    if Operation == WsAsyncGetProtoByName {
        Protoent = getprotobyname(ByWhat);
    } else {
        Protoent = getprotobynumber(PtrToUlong(ByWhat as *const core::ffi::c_void) as i32);
    }

    if Protoent.is_null() {
        ErrorCode = GetLastError() as i32;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

    if TaskHandle == WsAsyncCancelledTaskHandle {
        LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
        return;
    }

    if !Protoent.is_null() {
        BufferSize = CopyProtoentToBuffer(Buffer, BufferLength, Protoent);

        if BufferSize > BufferLength as usize {
            ErrorCode = WSAENOBUFS as i32;
        } else {
            ErrorCode = NO_ERROR as i32;
        }
    }

    WsAsyncCurrentTaskHandle = core::ptr::null_mut();

    LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

    lParam = ((((BufferSize & 0xffff) as u16 as u32 | ((ErrorCode as u32 & 0xffff) as u16 as u32) << 16) as isize)) as LPARAM;

    WPUPostMessage(hWnd, wMsg, TaskHandle as WPARAM, lParam);
}
#[no_mangle]
pub unsafe extern "system" fn WsAsyncGetHost(
    TaskHandle: HANDLE,
    Operation: DWORD,
    hWnd: HWND,
    wMsg: UINT,
    ByWhat: PCHAR,
    Length: INT,
    Type: INT,
    Buffer: PCHAR,
    BufferLength: INT,
) {
    let mut Hostent: PHOSTENT = core::ptr::null_mut();
    let mut BufferSize: usize = 0;
    let mut lParam: LPARAM;
    let mut ErrorCode: INT = 0;

    if Operation == WsAsyncGetHostByAddr as DWORD {
        Hostent = gethostbyaddr(ByWhat, Length, Type);
    } else {
        Hostent = gethostbyname(ByWhat);
    }

    if Hostent.is_null() {
        ErrorCode = GetLastError() as INT;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

    if TaskHandle == WsAsyncCancelledTaskHandle {
        LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
        return;
    }

    if !Hostent.is_null() {
        BufferSize = CopyHostentToBuffer(Buffer, BufferLength, Hostent);

        if BufferSize > BufferLength as usize {
            ErrorCode = WSAENOBUFS as INT;
        } else {
            ErrorCode = NO_ERROR as INT;
        }
    }

    WsAsyncCurrentTaskHandle = core::ptr::null_mut();

    LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

    lParam = ((BufferSize & 0xffff) as u16 as u32
        | (((ErrorCode as usize) & 0xffff) as u16 as u32) << 16) as i32 as LPARAM;

    WPUPostMessage(hWnd, wMsg, TaskHandle as WPARAM, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn WsAsyncThread(ThreadContext: PVOID) -> DWORD {
    let Context: PWSASYNCCONTEXT = ThreadContext as PWSASYNCCONTEXT;
    let mut AsyncBlock: PWSASYNCBLOCK;
    let mut Entry: PLIST_ENTRY;
    let AsyncEvent: HANDLE = (*Context).AsyncEvent;
    let ListHead: PLIST_ENTRY = core::ptr::addr_of_mut!((*Context).AsyncQueue);

    WSASetBlockingHook(core::mem::transmute::<
        Option<unsafe extern "system" fn() -> BOOL>,
        FARPROC,
    >(Some(WsAsyncThreadBlockingHook as unsafe extern "system" fn() -> BOOL)));

    loop {
        WaitForSingleObject(AsyncEvent, INFINITE);

        EnterCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

        while !IsListEmpty(ListHead) {
            Entry = RemoveHeadList(ListHead);
            AsyncBlock = ((Entry as ULONG_PTR).wrapping_sub(
                core::ptr::addr_of_mut!((*core::ptr::null_mut::<WSASYNCBLOCK>()).AsyncQueue) as ULONG_PTR
            )) as PWSASYNCBLOCK;

            WsAsyncCurrentTaskHandle = (*AsyncBlock).TaskHandle;

            LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

            match (*AsyncBlock).Operation {
                0 | 1 => {
                    WsAsyncGetHost(
                        (*AsyncBlock).TaskHandle,
                        (*AsyncBlock).Operation as DWORD,
                        (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().hWnd,
                        (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().wMsg,
                        (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().ByWhat as PCHAR,
                        (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().Length as INT,
                        (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().Type as INT,
                        (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().Buffer as PCHAR,
                        (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().BufferLength as INT,
                    );
                }

                3 | 2 => {
                    WsAsyncGetProto(
                        (*AsyncBlock).TaskHandle,
                        (*AsyncBlock).Operation as DWORD,
                        (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().hWnd,
                        (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().wMsg,
                        (*AsyncBlock).__bindgen_anon_1.GetHost.as_mut().ByWhat as PCHAR,
                        (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().Buffer as PCHAR,
                        (*AsyncBlock).__bindgen_anon_1.GetProto.as_mut().BufferLength as INT,
                    );
                }

                5 | 4 => {
                    WsAsyncGetServ(
                        (*AsyncBlock).TaskHandle,
                        (*AsyncBlock).Operation as DWORD,
                        (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().hWnd,
                        (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().wMsg,
                        (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().ByWhat as PCHAR,
                        (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().Protocol as PCHAR,
                        (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().Buffer as PCHAR,
                        (*AsyncBlock).__bindgen_anon_1.GetServ.as_mut().BufferLength as INT,
                    );
                }

                6 => {
                    WSACleanup();

                    WsAsyncFreeBlock(AsyncBlock);

                    EnterCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

                    while !IsListEmpty(ListHead) {
                        Entry = RemoveHeadList(ListHead);
                        AsyncBlock = ((Entry as ULONG_PTR).wrapping_sub(
                            core::ptr::addr_of_mut!((*core::ptr::null_mut::<WSASYNCBLOCK>()).AsyncQueue) as ULONG_PTR
                        )) as PWSASYNCBLOCK;
                        WsAsyncFreeBlock(AsyncBlock);
                    }

                    LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

                    CloseHandle(AsyncEvent);
                    HeapFree(WsSockHeap, 0, Context as LPVOID);

                    FreeLibraryAndExitThread(WsAsyncDllHandle as HMODULE, 0);
                }

                _ => {}
            }

            WsAsyncCurrentTaskHandle = core::ptr::null_mut();

            WsAsyncFreeBlock(AsyncBlock);
            EnterCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
    }
}
#[no_mangle]
pub unsafe extern "system" fn WsAsyncCheckAndInitThread() -> BOOL {
    let mut ThreadHandle: HANDLE;
    let mut Tid: DWORD = 0;
    let mut Context: PWSASYNCCONTEXT = core::ptr::null_mut();
    let mut WsaData: WSADATA = core::mem::zeroed();

    if WsAsyncThreadInitialized != 0 {
        return (TRUE as i32);
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

    if WsAsyncThreadInitialized == 0 {
        Context = HeapAlloc(
            WsSockHeap,
            0,
            (core::mem::size_of::<_WSASYNCCONTEXT>() as SIZE_T),
        ) as PWSASYNCCONTEXT;

        if !Context.is_null() {
            WsAsyncQueue = core::ptr::addr_of_mut!((*Context).AsyncQueue);
            InitializeListHead(WsAsyncQueue);
            (*Context).AsyncEvent = CreateEventA(
                core::ptr::null_mut(),
                FALSE as BOOL,
                FALSE as BOOL,
                core::ptr::null(),
            );
            WsAsyncEvent = (*Context).AsyncEvent;

            if WSAStartup(MAKEWORD(2, 2), core::ptr::addr_of_mut!(WsaData)) != (ERROR_SUCCESS as i32) {
                if !(*Context).AsyncEvent.is_null() {
                    CloseHandle((*Context).AsyncEvent);
                }
                HeapFree(WsSockHeap, 0, Context as PVOID);
            } else {
                ThreadHandle = CreateThread(
                    core::ptr::null_mut(),
                    0,
                    Some(WsAsyncThread),
                    Context as PVOID,
                    0,
                    core::ptr::addr_of_mut!(Tid),
                );

                if ThreadHandle.is_null() {
                    WSACleanup();
                    if !(*Context).AsyncEvent.is_null() {
                        CloseHandle((*Context).AsyncEvent);
                    }
                    HeapFree(WsSockHeap, 0, Context as PVOID);
                } else {
                    CloseHandle(ThreadHandle);
                    WsAsyncThreadInitialized = (TRUE as u8);
                }
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
    return WsAsyncThreadInitialized as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WsAsyncTerminateThread() {
    let mut AsyncBlock: PWSASYNCBLOCK = core::ptr::null_mut();

    if WsAsyncThreadInitialized == 0 {
        return;
    }

    AsyncBlock = WsAsyncAllocateBlock(0);

    (*AsyncBlock).Operation = WsAsyncTerminate;

    WsAsyncQueueRequest(AsyncBlock);
    WsAsyncThreadInitialized = 0u8;
}

#[no_mangle]
pub unsafe extern "system" fn WsAsyncQueueRequest(AsyncBlock: PWSASYNCBLOCK) {
    EnterCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
    InsertTailList(WsAsyncQueue, core::ptr::addr_of_mut!((*AsyncBlock).AsyncQueue));
    SetEvent(WsAsyncEvent);
    LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
}

#[no_mangle]
pub unsafe extern "system" fn WsAsyncCancelRequest(TaskHandle: HANDLE) -> INT {
    let mut Entry: PLIST_ENTRY;
    let mut AsyncBlock: PWSASYNCBLOCK;

    if WsAsyncThreadInitialized == 0 {
        return WSAEINVAL as INT;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));

    if TaskHandle == WsAsyncCurrentTaskHandle {
        WsAsyncCancelledTaskHandle = TaskHandle;
        LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
        return (NO_ERROR as i32);
    }

    Entry = (*WsAsyncQueue).Flink;
    while Entry != WsAsyncQueue {
        AsyncBlock = (Entry as usize).wrapping_sub(
            core::ptr::addr_of_mut!((*core::ptr::null_mut::<WSASYNCBLOCK>()).AsyncQueue) as usize
        ) as PWSASYNCBLOCK;

        if TaskHandle == (*AsyncBlock).TaskHandle {
            RemoveEntryList(Entry);
            LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
            WsAsyncFreeBlock(AsyncBlock);
            return (NO_ERROR as i32);
        }

        Entry = (*Entry).Flink;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(WsAsyncCritSect));
    return WSAEINVAL as INT;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/async.c
//  * PURPOSE:     Async Block Object and Async Thread Management
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// /* DATA **********************************************************************/
//
// BOOLEAN WsAsyncThreadInitialized;
// LONG WsAsyncTaskHandle;
// PLIST_ENTRY WsAsyncQueue;
// CRITICAL_SECTION WsAsyncCritSect;
// HANDLE WsAsyncEvent;
// HANDLE WsAsyncCurrentTaskHandle;
// HANDLE WsAsyncCancelledTaskHandle;
// HINSTANCE WsAsyncDllHandle;
//
// #define WsAsyncLock()       EnterCriticalSection(&WsAsyncCritSect)
// #define WsAsyncUnlock()     LeaveCriticalSection(&WsAsyncCritSect)
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// __stdcall
// WsAsyncGlobalInitialize(VOID)
// {
//     /* Initialize the async lock */
//     InitializeCriticalSection(&WsAsyncCritSect);
// }
//
// VOID
// __stdcall
// WsAsyncGlobalTerminate(VOID)
// {
//     /* Destroy the async lock */
//     DeleteCriticalSection(&WsAsyncCritSect);
// }
//
//
// SIZE_T
// __stdcall
// BytesInHostent(PHOSTENT Hostent)
// {
//     SIZE_T Bytes;
//     INT i;
//
//     /* Start with the static stuff */
//     Bytes = sizeof(HOSTENT) + strlen(Hostent->h_name) + sizeof(CHAR);
//
//     /* Add 2 pointers for the list-terminators */
//     Bytes += 2 * sizeof(ULONG_PTR);
//
//     /* Now loop for the aliases */
//     for (i = 0; Hostent->h_aliases[i]; i++)
//     {
//         /* Add the alias size, plus the space its pointer takes */
//         Bytes += strlen(Hostent->h_aliases[i]) + sizeof(CHAR) + sizeof(ULONG_PTR);
//     }
//
//     /* Now loop for the hostnames  */
//     for (i = 0; Hostent->h_addr_list[i]; i++)
//     {
//         /* Add the alias size, plus the space its pointer takes */
//         Bytes += Hostent->h_length + sizeof(ULONG_PTR);
//     }
//
//     /* Align to 8 bytes */
//     return (Bytes + 7) & ~7;
// }
//
// SIZE_T
// __stdcall
// BytesInServent(PSERVENT Servent)
// {
//     SIZE_T Bytes;
//     INT i;
//
//     /* Start with the static stuff */
//     Bytes = sizeof(SERVENT) +
//             strlen(Servent->s_name) + sizeof(CHAR) +
//             strlen(Servent->s_proto) + sizeof(CHAR);
//
//     /* Add 1 pointers for the list terminator */
//     Bytes += sizeof(ULONG_PTR);
//
//     /* Now loop for the aliases */
//     for (i = 0; Servent->s_aliases[i]; i++)
//     {
//         /* Add the alias size, plus the space its pointer takes */
//         Bytes += strlen(Servent->s_aliases[i]) + sizeof(CHAR) + sizeof(ULONG_PTR);
//     }
//
//     /* return */
//     return Bytes;
// }
//
// SIZE_T
// __stdcall
// BytesInProtoent(PPROTOENT Protoent)
// {
//     SIZE_T Bytes;
//     INT i;
//
//     /* Start with the static stuff */
//     Bytes = sizeof(SERVENT) + strlen(Protoent->p_name) + sizeof(CHAR);
//
//     /* Add 1 pointers for the list terminator */
//     Bytes += sizeof(ULONG_PTR);
//
//     /* Now loop for the aliases */
//     for (i = 0; Protoent->p_aliases[i]; i++)
//     {
//         /* Add the alias size, plus the space its pointer takes */
//         Bytes += strlen(Protoent->p_aliases[i]) + sizeof(CHAR) + sizeof(ULONG_PTR);
//     }
//
//     /* return */
//     return Bytes;
// }
//
// SIZE_T
// __stdcall
// CopyHostentToBuffer(IN PCHAR Buffer,
//                     IN INT BufferLength,
//                     IN PHOSTENT Hostent)
// {
//     SIZE_T BufferSize, CurrentSize, NameSize;
//     PCHAR p = Buffer;
//     DWORD Aliases = 0, Names = 0;
//     DWORD i;
//     PHOSTENT ReturnedHostent = (PHOSTENT)Buffer;
//
//     /* Determine the buffer size required */
//     BufferSize = BytesInHostent(Hostent);
//
//     /* Check which size to use */
//     if ((DWORD)BufferLength > BufferSize)
//     {
//         /* Zero the buffer */
//         RtlZeroMemory(Buffer, BufferSize);
//     }
//     else
//     {
//         /* Zero the buffer */
//         RtlZeroMemory(Buffer, BufferLength);
//     }
//
//     /* Start with the raw Hostent */
//     CurrentSize = sizeof(HOSTENT);
//
//     /* Return the size needed now */
//     if (CurrentSize > (DWORD)BufferLength) return BufferSize;
//
//     /* Copy the Hostent and initialize it */
//     CopyMemory(p, Hostent, sizeof(HOSTENT));
//     p = Buffer + CurrentSize;
//     ReturnedHostent->h_name = NULL;
//     ReturnedHostent->h_aliases = NULL;
//     ReturnedHostent->h_addr_list = NULL;
//
//     /* Find out how many aliases there are */
//     while (Hostent->h_aliases[Aliases])
//     {
//         /* Increase the alias count */
//         Aliases++;
//     }
//
//     /* Add the aliases to the size, and validate it */
//     CurrentSize += (Aliases + 1) * sizeof(ULONG_PTR);
//     if (CurrentSize > (DWORD)BufferLength)
//     {
//         /* Clear the aliases and return */
//         Hostent->h_aliases = NULL;
//         return BufferSize;
//     }
//
//     /* Write the aliases, update the pointer */
//     ReturnedHostent->h_aliases = (PCHAR*)p;
//     p = Buffer + CurrentSize;
//
//     /* Find out how many names there are */
//     while (Hostent->h_addr_list[Names])
//     {
//         /* Increase the alias count */
//         Names++;
//     }
//
//     /* Add the names to the size, and validate it */
//     CurrentSize += (Names + 1) * sizeof(ULONG_PTR);
//     if (CurrentSize > (DWORD)BufferLength)
//     {
//         /* Clear the aliases and return */
//         Hostent->h_addr_list = NULL;
//         return BufferSize;
//     }
//
//     /* Write the names, update the pointer */
//     ReturnedHostent->h_addr_list = (PCHAR*)p;
//     p = Buffer + CurrentSize;
//
//     /* Now add the names */
//     for (i = 0; i < Names; i++)
//     {
//         /* Update size and validate */
//         CurrentSize += Hostent->h_length;
//         if (CurrentSize > (DWORD)BufferLength) return BufferSize;
//
//         /* Write pointer and copy */
//         ReturnedHostent->h_addr_list[i] = p;
//         CopyMemory(p, Hostent->h_addr_list[i], Hostent->h_length);
//
//         /* Update pointer */
//         p = Buffer + CurrentSize;
//     }
//
//     /* Finalize the list */
//     ReturnedHostent->h_addr_list[i] = NULL;
//
//     /* Add the service name to the size, and validate it */
//     NameSize = strlen(Hostent->h_name) + sizeof(CHAR);
//     CurrentSize += NameSize;
//     if (CurrentSize > (DWORD)BufferLength) return BufferSize;
//
//     /* Write the service name and update the pointer */
//     ReturnedHostent->h_name = p;
//     CopyMemory(p, Hostent->h_name, NameSize);
//     p = Buffer + CurrentSize;
//
//     /* Now add the aliases */
//     for (i = 0; i < Aliases; i++)
//     {
//         /* Update size and validate */
//         NameSize = strlen(Hostent->h_aliases[i]) + sizeof(CHAR);
//         CurrentSize += NameSize;
//         if (CurrentSize > (DWORD)BufferLength) return BufferSize;
//
//         /* Write pointer and copy */
//         ReturnedHostent->h_aliases[i] = p;
//         CopyMemory(p, Hostent->h_aliases[i], NameSize);
//
//         /* Update pointer */
//         p = Buffer + CurrentSize;
//     }
//
//     /* Finalize the list and return */
//     ReturnedHostent->h_aliases[i] = NULL;
//     return BufferSize;
// }
//
// SIZE_T
// __stdcall
// CopyServentToBuffer(IN PCHAR Buffer,
//                     IN INT BufferLength,
//                     IN PSERVENT Servent)
// {
//     SIZE_T BufferSize, CurrentSize, NameSize;
//     PCHAR p = Buffer;
//     DWORD Aliases = 0;
//     DWORD i;
//     PSERVENT ReturnedServent = (PSERVENT)Buffer;
//
//     /* Determine the buffer size required */
//     BufferSize = BytesInServent(Servent);
//
//     /* Check which size to use */
//     if ((DWORD)BufferLength > BufferSize)
//     {
//         /* Zero the buffer */
//         ZeroMemory(Buffer, BufferSize);
//     }
//     else
//     {
//         /* Zero the buffer */
//         ZeroMemory(Buffer, BufferLength);
//     }
//
//     /* Start with the raw servent */
//     CurrentSize = sizeof(SERVENT);
//
//     /* Return the size needed now */
//     if (CurrentSize > (DWORD)BufferLength) return BufferSize;
//
//     /* Copy the servent and initialize it */
//     CopyMemory(p, Servent, sizeof(SERVENT));
//     p = Buffer + CurrentSize;
//     ReturnedServent->s_name = NULL;
//     ReturnedServent->s_aliases = NULL;
//     ReturnedServent->s_proto = NULL;
//
//     /* Find out how many aliases there are */
//     while (Servent->s_aliases[Aliases])
//     {
//         /* Increase the alias count */
//         Aliases++;
//     }
//
//     /* Add the aliases to the size, and validate it */
//     CurrentSize += (Aliases + 1) * sizeof(ULONG_PTR);
//     if (CurrentSize > (DWORD)BufferLength)
//     {
//         /* Clear the aliases and return */
//         Servent->s_aliases = NULL;
//         return BufferSize;
//     }
//
//     /* Write the aliases, update the pointer */
//     ReturnedServent->s_aliases = (PCHAR*)p;
//     p = Buffer + CurrentSize;
//
//     /* Add the service name to the size, and validate it */
//     NameSize = strlen(Servent->s_name) + sizeof(CHAR);
//     CurrentSize += NameSize;
//     if (CurrentSize > (DWORD)BufferLength) return BufferSize;
//
//     /* Write the service name and update the pointer */
//     ReturnedServent->s_name = p;
//     CopyMemory(p, Servent->s_name, NameSize);
//     p = Buffer + CurrentSize;
//
//     /* Now add the aliases */
//     for (i = 0; i < Aliases; i++)
//     {
//         /* Update size and validate */
//         NameSize = strlen(Servent->s_aliases[i]) + sizeof(CHAR);
//         CurrentSize += NameSize;
//         if (CurrentSize > (DWORD)BufferLength) return BufferSize;
//
//         /* Write pointer and copy */
//         ReturnedServent->s_aliases[i] = p;
//         CopyMemory(p, Servent->s_aliases[i], NameSize);
//
//         /* Update pointer */
//         p = Buffer + CurrentSize;
//     }
//
//     /* Finalize the list and return */
//     ReturnedServent->s_aliases[i] = NULL;
//     return BufferSize;
// }
//
// SIZE_T
// __stdcall
// CopyProtoentToBuffer(IN PCHAR Buffer,
//                      IN INT BufferLength,
//                      IN PPROTOENT Protoent)
// {
//     SIZE_T BufferSize, CurrentSize, NameSize;
//     PCHAR p = Buffer;
//     DWORD Aliases = 0;
//     DWORD i;
//     PPROTOENT ReturnedProtoent = (PPROTOENT)Buffer;
//
//     /* Determine the buffer size required */
//     BufferSize = BytesInProtoent(Protoent);
//
//     /* Check which size to use */
//     if ((DWORD)BufferLength > BufferSize)
//     {
//         /* Zero the buffer */
//         ZeroMemory(Buffer, BufferSize);
//     }
//     else
//     {
//         /* Zero the buffer */
//         ZeroMemory(Buffer, BufferLength);
//     }
//
//     /* Start with the raw servent */
//     CurrentSize = sizeof(PROTOENT);
//
//     /* Return the size needed now */
//     if (CurrentSize > (DWORD)BufferLength) return BufferSize;
//
//     /* Copy the servent and initialize it */
//     CopyMemory(p, Protoent, sizeof(PROTOENT));
//     p = Buffer + CurrentSize;
//     ReturnedProtoent->p_name = NULL;
//     ReturnedProtoent->p_aliases = NULL;
//
//     /* Find out how many aliases there are */
//     while (Protoent->p_aliases[Aliases])
//     {
//         /* Increase the alias count */
//         Aliases++;
//     }
//
//     /* Add the aliases to the size, and validate it */
//     CurrentSize += (Aliases + 1) * sizeof(ULONG_PTR);
//     if (CurrentSize > (DWORD)BufferLength)
//     {
//         /* Clear the aliases and return */
//         Protoent->p_aliases = NULL;
//         return BufferSize;
//     }
//
//     /* Write the aliases, update the pointer */
//     ReturnedProtoent->p_aliases = (PCHAR*)p;
//     p = Buffer + CurrentSize;
//
//     /* Add the service name to the size, and validate it */
//     NameSize = strlen(Protoent->p_name) + sizeof(CHAR);
//     CurrentSize += NameSize;
//     if (CurrentSize > (DWORD)BufferLength) return BufferSize;
//
//     /* Write the service name and update the pointer */
//     ReturnedProtoent->p_name = p;
//     CopyMemory(p, Protoent->p_name, NameSize);
//     p = Buffer + CurrentSize;
//
//     /* Now add the aliases */
//     for (i = 0; i < Aliases; i++)
//     {
//         /* Update size and validate */
//         NameSize = strlen(Protoent->p_aliases[i]) + sizeof(CHAR);
//         CurrentSize += NameSize;
//         if (CurrentSize > (DWORD)BufferLength) return BufferSize;
//
//         /* Write pointer and copy */
//         ReturnedProtoent->p_aliases[i] = p;
//         CopyMemory(p, Protoent->p_aliases[i], NameSize);
//
//         /* Update pointer */
//         p = Buffer + CurrentSize;
//     }
//
//     /* Finalize the list and return */
//     ReturnedProtoent->p_aliases[i] = NULL;
//     return BufferSize;
// }
//
// PWSASYNCBLOCK
// __stdcall
// WsAsyncAllocateBlock(IN SIZE_T ExtraLength)
// {
//     PWSASYNCBLOCK AsyncBlock;
//
//     /* Add the size of the block */
//     ExtraLength += sizeof(WSASYNCBLOCK);
//
//     /* Allocate it */
//     AsyncBlock = HeapAlloc(WsSockHeap, 0, ExtraLength);
//
//     /* Get a handle to it */
//     AsyncBlock->TaskHandle = UlongToPtr(InterlockedIncrement(&WsAsyncTaskHandle));
//
//     /* Return it */
//     return AsyncBlock;
// }
//
// BOOL
// WINAPI
// WsAsyncThreadBlockingHook(VOID)
// {
//     /* Check if this task is being cancelled */
//     if (WsAsyncCurrentTaskHandle == WsAsyncCancelledTaskHandle)
//     {
//         /* Cancel the blocking call so we can get back */
//         WSACancelBlockingCall();
//     }
//
//     /* Return to system */
//     return FALSE;
// }
//
// VOID
// __stdcall
// WsAsyncFreeBlock(IN PWSASYNCBLOCK AsyncBlock)
// {
//     /* Free it */
//     HeapFree(WsSockHeap, 0, AsyncBlock);
// }
//
// VOID
// __stdcall
// WsAsyncGetServ(IN HANDLE TaskHandle,
//                IN DWORD Operation,
//                IN HWND hWnd,
//                IN UINT wMsg,
//                IN CHAR  *ByWhat,
//                IN CHAR  *Protocol,
//                IN CHAR  *Buffer,
//                IN INT BufferLength)
// {
//     PSERVENT Servent;
//     SIZE_T BufferSize = 0;
//     LPARAM lParam;
//     INT ErrorCode = 0;
//
//     /* Check the operation */
//     if (Operation == WsAsyncGetServByName)
//     {
//         /* Call the API */
//         Servent = getservbyname(ByWhat, Protocol);
//     }
//     else
//     {
//         /* Call the API */
//         Servent = getservbyport(PtrToUlong(ByWhat), Protocol);
//     }
//
//     /* Make sure we got one */
//     if (!Servent) ErrorCode = GetLastError();
//
//     /* Acquire the lock */
//     WsAsyncLock();
//
//     /* Check if this task got cancelled */
//     if (TaskHandle == WsAsyncCancelledTaskHandle)
//     {
//         /* Return */
//         WsAsyncUnlock();
//         return;
//     }
//
//     /* If we got a Servent back, copy it */
//     if (Servent)
//     {
//         /* Copy it into the buffer */
//         BufferSize = CopyServentToBuffer(Buffer, BufferLength, Servent);
//
//         /* Check if we had enough space */
//         if (BufferSize > (DWORD)BufferLength)
//         {
//             /* Not enough */
//             ErrorCode = WSAENOBUFS;
//         }
//         else
//         {
//             /* Perfect */
//             ErrorCode = NO_ERROR;
//         }
//     }
//
//     /* Not processing anymore */
//     WsAsyncCurrentTaskHandle = NULL;
//
//     /* Release the lock */
//     WsAsyncUnlock();
//
//     /* Make the messed-up lParam reply */
//     lParam = WSAMAKEASYNCREPLY(BufferSize, ErrorCode);
//
//     /* Sent it through the Upcall API */
//     WPUPostMessage(hWnd, wMsg, (WPARAM)TaskHandle, lParam);
// }
//
// VOID
// __stdcall
// WsAsyncGetProto(IN HANDLE TaskHandle,
//                 IN DWORD Operation,
//                 IN HWND hWnd,
//                 IN UINT wMsg,
//                 IN CHAR  *ByWhat,
//                 IN CHAR  *Buffer,
//                 IN INT BufferLength)
// {
//     PPROTOENT Protoent;
//     SIZE_T BufferSize = 0;
//     LPARAM lParam;
//     INT ErrorCode = 0;
//
//     /* Check the operation */
//     if (Operation == WsAsyncGetProtoByName)
//     {
//         /* Call the API */
//         Protoent = getprotobyname(ByWhat);
//     }
//     else
//     {
//         /* Call the API */
//         Protoent = getprotobynumber(PtrToUlong(ByWhat));
//     }
//
//     /* Make sure we got one */
//     if (!Protoent) ErrorCode = GetLastError();
//
//     /* Acquire the lock */
//     WsAsyncLock();
//
//     /* Check if this task got cancelled */
//     if (TaskHandle == WsAsyncCancelledTaskHandle)
//     {
//         /* Return */
//         WsAsyncUnlock();
//         return;
//     }
//
//     /* If we got a Servent back, copy it */
//     if (Protoent)
//     {
//         /* Copy it into the buffer */
//         BufferSize = CopyProtoentToBuffer(Buffer, BufferLength, Protoent);
//
//         /* Check if we had enough space */
//         if (BufferSize > (DWORD)BufferLength)
//         {
//             /* Not enough */
//             ErrorCode = WSAENOBUFS;
//         }
//         else
//         {
//             /* Perfect */
//             ErrorCode = NO_ERROR;
//         }
//     }
//
//     /* Not processing anymore */
//     WsAsyncCurrentTaskHandle = NULL;
//
//     /* Release the lock */
//     WsAsyncUnlock();
//
//     /* Make the messed-up lParam reply */
//     lParam = WSAMAKEASYNCREPLY(BufferSize, ErrorCode);
//
//     /* Sent it through the Upcall API */
//     WPUPostMessage(hWnd, wMsg, (WPARAM)TaskHandle, lParam);
// }
//
// VOID
// __stdcall
// WsAsyncGetHost(IN HANDLE TaskHandle,
//                IN DWORD Operation,
//                IN HWND hWnd,
//                IN UINT wMsg,
//                IN CHAR  *ByWhat,
//                IN INT Length,
//                IN INT Type,
//                IN CHAR  *Buffer,
//                IN INT BufferLength)
// {
//     PHOSTENT Hostent;
//     SIZE_T BufferSize = 0;
//     LPARAM lParam;
//     INT ErrorCode = 0;
//
//     /* Check the operation */
//     if (Operation == WsAsyncGetHostByAddr)
//     {
//         /* Call the API */
//         Hostent = gethostbyaddr(ByWhat, Length, Type);
//     }
//     else
//     {
//         /* Call the API */
//         Hostent = gethostbyname(ByWhat);
//     }
//
//     /* Make sure we got one */
//     if (!Hostent) ErrorCode = GetLastError();
//
//     /* Acquire the lock */
//     WsAsyncLock();
//
//     /* Check if this task got cancelled */
//     if (TaskHandle == WsAsyncCancelledTaskHandle)
//     {
//         /* Return */
//         WsAsyncUnlock();
//         return;
//     }
//
//     /* If we got a Servent back, copy it */
//     if (Hostent)
//     {
//         /* Copy it into the buffer */
//         BufferSize = CopyHostentToBuffer(Buffer, BufferLength, Hostent);
//
//         /* Check if we had enough space */
//         if (BufferSize > (DWORD)BufferLength)
//         {
//             /* Not enough */
//             ErrorCode = WSAENOBUFS;
//         }
//         else
//         {
//             /* Perfect */
//             ErrorCode = NO_ERROR;
//         }
//     }
//
//     /* Not processing anymore */
//     WsAsyncCurrentTaskHandle = NULL;
//
//     /* Release the lock */
//     WsAsyncUnlock();
//
//     /* Make the messed-up lParam reply */
//     lParam = WSAMAKEASYNCREPLY(BufferSize, ErrorCode);
//
//     /* Sent it through the Upcall API */
//     WPUPostMessage(hWnd, wMsg, (WPARAM)TaskHandle, lParam);
// }
//
// DWORD
// WINAPI
// WsAsyncThread(IN PVOID ThreadContext)
// {
//     PWSASYNCCONTEXT Context = ThreadContext;
//     PWSASYNCBLOCK AsyncBlock;
//     PLIST_ENTRY Entry;
//     HANDLE AsyncEvent = Context->AsyncEvent;
//     PLIST_ENTRY ListHead = &Context->AsyncQueue;
//
//     /* Set the blocking hook */
//     WSASetBlockingHook((FARPROC)WsAsyncThreadBlockingHook);
//
//     /* Loop */
//     while (TRUE)
//     {
//         /* Wait for the event */
//         WaitForSingleObject(AsyncEvent, INFINITE);
//
//         /* Get the lock */
//         WsAsyncLock();
//
//         /* Process the queue */
//         while (!IsListEmpty(ListHead))
//         {
//             /* Remove this entry and get the async block */
//             Entry = RemoveHeadList(ListHead);
//             AsyncBlock = CONTAINING_RECORD(Entry, WSASYNCBLOCK, AsyncQueue);
//
//             /* Save the current task handle */
//             WsAsyncCurrentTaskHandle = AsyncBlock->TaskHandle;
//
//             /* Release the lock */
//             WsAsyncUnlock();
//
//             /* Check which operation to do */
//             switch (AsyncBlock->Operation)
//             {
//                 /* Get Host by Y */
//                 case WsAsyncGetHostByAddr: case WsAsyncGetHostByName:
//
//                     /* Call the handler */
//                     WsAsyncGetHost(AsyncBlock->TaskHandle,
//                                    AsyncBlock->Operation,
//                                    AsyncBlock->GetHost.hWnd,
//                                    AsyncBlock->GetHost.wMsg,
//                                    AsyncBlock->GetHost.ByWhat,
//                                    AsyncBlock->GetHost.Length,
//                                    AsyncBlock->GetHost.Type,
//                                    AsyncBlock->GetHost.Buffer,
//                                    AsyncBlock->GetHost.BufferLength);
//                     break;
//
//                 /* Get Proto by Y */
//                 case WsAsyncGetProtoByNumber: case WsAsyncGetProtoByName:
//
//                     /* Call the handler */
//                     WsAsyncGetProto(AsyncBlock->TaskHandle,
//                                     AsyncBlock->Operation,
//                                     AsyncBlock->GetProto.hWnd,
//                                     AsyncBlock->GetProto.wMsg,
//                                     AsyncBlock->GetHost.ByWhat,
//                                     AsyncBlock->GetProto.Buffer,
//                                     AsyncBlock->GetProto.BufferLength);
//                     break;
//
//                 /* Get Serv by Y */
//                 case WsAsyncGetServByPort: case WsAsyncGetServByName:
//
//                     /* Call the handler */
//                     WsAsyncGetServ(AsyncBlock->TaskHandle,
//                                    AsyncBlock->Operation,
//                                    AsyncBlock->GetServ.hWnd,
//                                    AsyncBlock->GetServ.wMsg,
//                                    AsyncBlock->GetServ.ByWhat,
//                                    AsyncBlock->GetServ.Protocol,
//                                    AsyncBlock->GetServ.Buffer,
//                                    AsyncBlock->GetServ.BufferLength);
//                     break;
//
//                 /* Termination */
//                 case WsAsyncTerminate:
//
//                     /* Clean up the extra reference */
//                     WSACleanup();
//
//                     /* Free the context block */
//                     WsAsyncFreeBlock(AsyncBlock);
//
//                     /* Acquire the lock */
//                     WsAsyncLock();
//
//                     /* Loop the queue and flush it */
//                     while (!IsListEmpty(ListHead))
//                     {
//                         Entry = RemoveHeadList(ListHead);
//                         AsyncBlock = CONTAINING_RECORD(Entry,
//                                                        WSASYNCBLOCK,
//                                                        AsyncQueue);
//                         WsAsyncFreeBlock(AsyncBlock);
//                     }
//
//                     /* Release lock */
//                     WsAsyncUnlock();
//
//                     /* Close the event, free the Context */
//                     CloseHandle(AsyncEvent);
//                     HeapFree(WsSockHeap, 0, Context);
//
//                     /* Remove the extra DLL reference and kill us */
//                     FreeLibraryAndExitThread(WsAsyncDllHandle, 0);
//
//                 default:
//                     break;
//             }
//
//             /* Done processing */
//             WsAsyncCurrentTaskHandle = NULL;
//
//             /* Free this block, get lock and reloop */
//             WsAsyncFreeBlock(AsyncBlock);
//             WsAsyncLock();
//         }
//
//         /* Release the lock */
//         WsAsyncUnlock();
//     }
// }
//
// BOOL
// __stdcall
// WsAsyncCheckAndInitThread(VOID)
// {
//     HANDLE ThreadHandle;
//     DWORD Tid;
//     PWSASYNCCONTEXT Context = NULL;
//     WSADATA WsaData;
//
//     /* Make sure we're not initialized */
//     if (WsAsyncThreadInitialized) return TRUE;
//
//     /* Acquire the lock */
//     WsAsyncLock();
//
//     /* Make sure we're not initialized */
//     if (!WsAsyncThreadInitialized)
//     {
//         /* Initialize Thread Context */
//         Context = HeapAlloc(WsSockHeap, 0, sizeof(*Context));
//         if (!Context)
//             goto Exit;
//
//         /* Initialize the Queue and event */
//         WsAsyncQueue = &Context->AsyncQueue;
//         InitializeListHead(WsAsyncQueue);
//         Context->AsyncEvent = CreateEvent(NULL, FALSE, FALSE, NULL);
//         WsAsyncEvent = Context->AsyncEvent;
//
//         /* Prevent us from ever being killed while running */
//         if (WSAStartup(MAKEWORD(2,2), &WsaData) != ERROR_SUCCESS)
//             goto Fail;
//
//         /* Create the thread */
//         ThreadHandle = CreateThread(NULL,
//                                     0,
//                                     WsAsyncThread,
//                                     Context,
//                                     0,
//                                     &Tid);
//         if (ThreadHandle == NULL)
//         {
//             /* Cleanup and fail */
//             WSACleanup();
//             goto Fail;
//         }
//
//         /* Close the handle and set init */
//         CloseHandle(ThreadHandle);
//         WsAsyncThreadInitialized = TRUE;
//     }
//
// Exit:
//     /* Release the lock */
//     WsAsyncUnlock();
//     return WsAsyncThreadInitialized;
//
// Fail:
//     /* Close the event, free the Context */
//     if (Context->AsyncEvent)
//         CloseHandle(Context->AsyncEvent);
//     HeapFree(WsSockHeap, 0, Context);
//
//     /* Bail out */
//     goto Exit;
// }
//
// VOID
// __stdcall
// WsAsyncTerminateThread(VOID)
// {
//     PWSASYNCBLOCK AsyncBlock;
//
//     /* Make sure we're initialized */
//     if (!WsAsyncThreadInitialized) return;
//
//     /* Allocate a block */
//     AsyncBlock = WsAsyncAllocateBlock(0);
//
//     /* Initialize it for termination */
//     AsyncBlock->Operation = WsAsyncTerminate;
//
//     /* Queue the request and return */
//     WsAsyncQueueRequest(AsyncBlock);
//     WsAsyncThreadInitialized = FALSE;
// }
//
// VOID
// __stdcall
// WsAsyncQueueRequest(IN PWSASYNCBLOCK AsyncBlock)
// {
//     /* Get the lock */
//     WsAsyncLock();
//
//     /* Insert it into the queue */
//     InsertTailList(WsAsyncQueue, &AsyncBlock->AsyncQueue);
//
//     /* Wake up the thread */
//     SetEvent(WsAsyncEvent);
//
//     /* Release lock and return */
//     WsAsyncUnlock();
// }
//
// INT
// __stdcall
// WsAsyncCancelRequest(IN HANDLE TaskHandle)
// {
//     PLIST_ENTRY Entry;
//     PWSASYNCBLOCK AsyncBlock;
//
//     /* Make sure we're initialized */
//     if (!WsAsyncThreadInitialized) return WSAEINVAL;
//
//     /* Acquire the lock */
//     WsAsyncLock();
//
//     /* Check if we're cancelling the current task */
//     if (TaskHandle == WsAsyncCurrentTaskHandle)
//     {
//         /* Mark us as cancelled, the async thread will see this later */
//         WsAsyncCancelledTaskHandle = TaskHandle;
//
//         /* Release lock and return */
//         WsAsyncUnlock();
//         return NO_ERROR;
//     }
//
//     /* Loop the queue */
//     Entry = WsAsyncQueue->Flink;
//     while (Entry != WsAsyncQueue)
//     {
//         /* Get the Async Block */
//         AsyncBlock = CONTAINING_RECORD(Entry, WSASYNCBLOCK, AsyncQueue);
//
//         /* Check if this is the one */
//         if (TaskHandle == AsyncBlock->TaskHandle)
//         {
//             /* It is, remove it */
//             RemoveEntryList(Entry);
//
//             /* Release the lock, free the block, and return */
//             WsAsyncUnlock();
//             WsAsyncFreeBlock(AsyncBlock);
//             return NO_ERROR;
//         }
//
//         /* Move to the next entry */
//         Entry = Entry->Flink;
//     }
//
//     /* Nothing found, fail */
//     WsAsyncUnlock();
//     return WSAEINVAL;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
