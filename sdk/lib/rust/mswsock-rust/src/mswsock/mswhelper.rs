use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    pub fn strlen(s: *const core::ffi::c_char) -> usize;
    fn wcslen(s: *const WCHAR) -> usize;
}
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> PVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
    fn MultiByteToWideChar( CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: ::core::ffi::c_int, lpWideCharStr: LPWSTR, cchWideChar: ::core::ffi::c_int, ) -> ::core::ffi::c_int;
    fn WideCharToMultiByte( CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: i32, lpMultiByteStr: LPSTR, cbMultiByte: i32, lpDefaultChar: LPCSTR, lpUsedDefaultChar: LPBOOL, ) -> i32;
}

// ---- file-scope preamble (translated) ----
pub const MSW_BUFSIZE: usize = 512;
pub const MAX_ARRAY_SIZE: usize = 5;
#[no_mangle]
pub unsafe extern "C" fn mswBufferInit(
    mswBuf: PMSW_BUFFER,
    buffer: *mut BYTE,
    bufferSize: DWORD,
) {
    core::ptr::write_bytes(
        mswBuf as *mut u8,
        0,
        core::mem::size_of::<MSW_BUFFER>(),
    );
    core::ptr::write_bytes(
        buffer,
        0,
        bufferSize as usize,
    );
    (*mswBuf).bytesMax = bufferSize;
    (*mswBuf).buffer = buffer;
    (*mswBuf).bufendptr = buffer;
    (*mswBuf).bufok = TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferCheck(mswBuf: PMSW_BUFFER, count: DWORD) -> BOOL {
    if (*mswBuf).bytesUsed.wrapping_add(count) <= (*mswBuf).bytesMax {
        return TRUE as INT;
    }
    (*mswBuf).bufok = FALSE as BOOL;
    return FALSE as INT;
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferIncUsed(mswBuf: PMSW_BUFFER, count: DWORD) -> BOOL {
    if mswBufferCheck(mswBuf, count) == 0 {
        return 0;
    }
    (*mswBuf).bytesUsed = ((*mswBuf).bytesUsed as u32)
        .wrapping_add(count as u32) as DWORD as DWORD;
    (*mswBuf).bufendptr = (*mswBuf).bufendptr.offset(count as isize);
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferEndPtr(mswBuf: PMSW_BUFFER) -> *mut BYTE {
    return (*mswBuf).bufendptr;
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferAppend(
    mswBuf: PMSW_BUFFER,
    dataToAppend: *mut core::ffi::c_void,
    dataSize: DWORD,
) -> BOOL {
    if mswBufferCheck(mswBuf, dataSize) == 0 {
        return FALSE as INT;
    }

    core::ptr::copy_nonoverlapping(
        dataToAppend as *const u8,
        (*mswBuf).bufendptr,
        dataSize as usize,
    );
    (*mswBuf).bytesUsed = (*mswBuf).bytesUsed.wrapping_add(dataSize);
    (*mswBuf).bufendptr = (*mswBuf).bufendptr.offset(dataSize as isize);

    TRUE as INT
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferAppendStrA(
    mswBuf: PMSW_BUFFER,
    str: *mut core::ffi::c_char,
) -> BOOL {
    
    return mswBufferAppend(
        mswBuf,
        str as *mut core::ffi::c_void,
        (strlen(str) as usize).wrapping_add(core::mem::size_of::<core::ffi::c_char>() as usize) as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferAppendStrW(
    mswBuf: PMSW_BUFFER,
    str: *mut WCHAR,
) -> BOOL {
    
    let bytelen: i32 = (wcslen(str)
        .wrapping_add(1)
        .wrapping_mul(core::mem::size_of::<WCHAR>())) as i32;
    return mswBufferAppend(mswBuf, str as *mut core::ffi::c_void, bytelen as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferAppendPtr(
    mswBuf: PMSW_BUFFER,
    ptr: *mut core::ffi::c_void,
) -> BOOL {
    return mswBufferAppend(
        mswBuf,
        &ptr as *const *mut core::ffi::c_void as *mut core::ffi::c_void,
        core::mem::size_of::<*mut core::ffi::c_void>() as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferAppendLst(
    mswBuf: PMSW_BUFFER,
    lst: *mut *mut core::ffi::c_void,
    itemByteLength: DWORD,
    ptrofs: LONG_PTR,
) -> BOOL {
    let mut lstItemCount: DWORD;
    let mut lstByteSize: DWORD;
    let mut lstDataPos: DWORD_PTR;
    let mut i1: DWORD;
    let mut ptrSrcLstPos: *mut UINT_PTR;

    ptrSrcLstPos = lst as *mut UINT_PTR;
    lstItemCount = 0;
    while *ptrSrcLstPos != 0 as UINT_PTR {
        lstItemCount = lstItemCount.wrapping_add(1);
        ptrSrcLstPos = ptrSrcLstPos.offset(1);
    }

    lstByteSize = ((lstItemCount.wrapping_add(1) as usize)
        .wrapping_mul(core::mem::size_of::<UINT_PTR>())
        .wrapping_add(lstItemCount.wrapping_mul(itemByteLength) as usize)) as DWORD;

    if (*mswBuf).bytesUsed.wrapping_add(lstByteSize) > (*mswBuf).bytesMax {
        return 0;
    }

    lstDataPos = ((lstItemCount.wrapping_add(1) as usize)
        .wrapping_mul(core::mem::size_of::<UINT_PTR>())
        .wrapping_add(mswBufferEndPtr(mswBuf) as usize)) as DWORD_PTR;
    lstDataPos = lstDataPos.wrapping_add(ptrofs as DWORD_PTR);

    i1 = 0;
    while i1 < lstItemCount {
        if mswBufferAppendPtr(mswBuf, lstDataPos as *mut core::ffi::c_void) == 0 {
            return 0;
        }
        lstDataPos = lstDataPos.wrapping_add(core::mem::size_of::<UINT_PTR>() as DWORD_PTR);
        i1 = i1.wrapping_add(1);
    }

    if mswBufferAppendPtr(mswBuf, core::ptr::null_mut()) == 0 {
        return 0;
    }

    ptrSrcLstPos = lst as *mut UINT_PTR;
    i1 = 0;
    while i1 < lstItemCount {
        mswBufferAppend(
            mswBuf,
            *(ptrSrcLstPos as *mut *mut BYTE) as *mut core::ffi::c_void,
            itemByteLength,
        );
        ptrSrcLstPos = ptrSrcLstPos.offset(1);
        i1 = i1.wrapping_add(1);
    }
    return (*mswBuf).bufok;
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferAppendStrLstA(
    mswBuf: PMSW_BUFFER,
    lst: *mut *mut core::ffi::c_void,
    ptrofs: LONG_PTR,
) -> BOOL {
    let mut lstItemLen: [DWORD; MAX_ARRAY_SIZE as usize] = [0; MAX_ARRAY_SIZE as usize];
    let mut lstItemCount: DWORD;
    let mut lstByteSize: DWORD;
    let mut lstDataPos: DWORD_PTR;
    let mut lstDataSize: DWORD;
    let mut i1: DWORD;
    let mut ptrSrcLstPos: *mut UINT_PTR;

    ptrSrcLstPos = lst as *mut UINT_PTR;
    lstItemCount = 0;
    lstDataSize = 0;

    while *ptrSrcLstPos != 0 as UINT_PTR {
        if lstItemCount >= MAX_ARRAY_SIZE as DWORD {
            return FALSE as INT;
        }

        i1 = (strlen(*ptrSrcLstPos as *mut core::ffi::c_char) as usize
            + core::mem::size_of::<core::ffi::c_char>()) as DWORD;
        lstItemLen[lstItemCount as usize] = i1;
        lstItemCount = lstItemCount.wrapping_add(1);
        lstDataSize = lstDataSize.wrapping_add(i1);
        ptrSrcLstPos = ptrSrcLstPos.offset(1);
    }

    lstByteSize = ((lstItemCount.wrapping_add(1) as usize)
        * core::mem::size_of::<UINT_PTR>())
        .wrapping_add(lstDataSize as usize) as DWORD;

    if (*mswBuf).bytesUsed.wrapping_add(lstByteSize) > (*mswBuf).bytesMax {
        return FALSE as INT;
    }

    lstDataPos = ((lstItemCount.wrapping_add(1) as usize)
        * core::mem::size_of::<UINT_PTR>())
        .wrapping_add(mswBufferEndPtr(mswBuf) as usize) as DWORD_PTR;

    lstDataPos = lstDataPos.wrapping_add(ptrofs as DWORD_PTR);

    i1 = 0;
    while i1 < lstItemCount {
        if mswBufferAppendPtr(mswBuf, lstDataPos as *mut core::ffi::c_void) == 0 {
            return FALSE as INT;
        }

        lstDataPos = lstDataPos.wrapping_add(lstItemLen[i1 as usize] as DWORD_PTR);
        i1 = i1.wrapping_add(1);
    }

    if mswBufferAppendPtr(mswBuf, core::ptr::null_mut()) == 0 {
        return FALSE as INT;
    }

    ptrSrcLstPos = lst as *mut UINT_PTR;
    i1 = 0;
    while i1 < lstItemCount {
        if mswBufferAppendStrA(mswBuf, *(ptrSrcLstPos as *mut *mut core::ffi::c_char)) == 0 {
            return FALSE as INT;
        }

        ptrSrcLstPos = ptrSrcLstPos.offset(1);
        i1 = i1.wrapping_add(1);
    }

    return (*mswBuf).bufok;
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferAppendBlob_Hostent(
    mswBuf: PMSW_BUFFER,
    lpRes: LPWSAQUERYSETW,
    hostAliasesA: *mut *mut core::ffi::c_char,
    hostnameA: *mut core::ffi::c_char,
    mut ip4addr: DWORD,
) -> BOOL {
    let mut phe: PHOSTENT;
    let mut lst: [*mut core::ffi::c_void; 2] = [core::ptr::null_mut(); 2];
    let bytesOfs: *mut BYTE;

    (*lpRes).lpBlob = mswBufferEndPtr(mswBuf) as LPBLOB;

    if mswBufferIncUsed(mswBuf, core::mem::size_of::<_BLOB>() as DWORD) == 0 {
        return 0;
    }

    (*(*lpRes).lpBlob).cbSize = 0;
    (*(*lpRes).lpBlob).pBlobData = mswBufferEndPtr(mswBuf);

    phe = (*(*lpRes).lpBlob).pBlobData as PHOSTENT;
    bytesOfs = mswBufferEndPtr(mswBuf);

    if mswBufferIncUsed(mswBuf, core::mem::size_of::<hostent>() as DWORD) == 0 {
        return 0;
    }

    (*phe).h_addrtype = AF_INET as core::ffi::c_short;
    (*phe).h_length = 4;

    (*phe).h_aliases = mswBufferEndPtr(mswBuf).offset_from(bytesOfs) as LONG_PTR as *mut *mut core::ffi::c_char;

    if !hostAliasesA.is_null() {
        if mswBufferAppendStrLstA(
            mswBuf,
            hostAliasesA as *mut *mut core::ffi::c_void,
            -(bytesOfs as LONG_PTR),
        ) == 0
        {
            return 0;
        }
    } else {
        if mswBufferAppendPtr(mswBuf, core::ptr::null_mut()) == 0 {
            return 0;
        }
    }

    core::ptr::write_bytes(lst.as_mut_ptr(), 0, 1);

    lst[0] = core::ptr::addr_of_mut!(ip4addr) as *mut core::ffi::c_void;

    (*phe).h_addr_list = mswBufferEndPtr(mswBuf).offset_from(bytesOfs) as LONG_PTR as *mut *mut core::ffi::c_char;

    if mswBufferAppendLst(
        mswBuf,
        lst.as_mut_ptr(),
        4,
        -(bytesOfs as LONG_PTR),
    ) == 0
    {
        return 0;
    }

    (*phe).h_name = mswBufferEndPtr(mswBuf).offset_from(bytesOfs) as LONG_PTR as *mut core::ffi::c_char;

    if mswBufferAppendStrA(mswBuf, hostnameA) == 0 {
        return 0;
    }

    (*(*lpRes).lpBlob).cbSize = mswBufferEndPtr(mswBuf).offset_from(bytesOfs) as LONG_PTR as ULONG;
    return (*mswBuf).bufok;
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferAppendBlob_Servent(
    mswBuf: PMSW_BUFFER,
    lpRes: LPWSAQUERYSETW,
    serviceNameA: *mut core::ffi::c_char,
    serviceAliasesA: *mut *mut core::ffi::c_char,
    protocolNameA: *mut core::ffi::c_char,
    port: WORD,
) -> BOOL {
    let mut pse: PSERVENT;
    let mut bytesOfs: *mut BYTE;

    (*lpRes).lpBlob = mswBufferEndPtr(mswBuf) as LPBLOB;

    if mswBufferIncUsed(mswBuf, core::mem::size_of::<_BLOB>() as DWORD) == 0 {
        return 0;
    }

    (*(*lpRes).lpBlob).cbSize = 0;
    (*(*lpRes).lpBlob).pBlobData = mswBufferEndPtr(mswBuf);

    pse = (*(*lpRes).lpBlob).pBlobData as PSERVENT;
    bytesOfs = mswBufferEndPtr(mswBuf);

    if mswBufferIncUsed(mswBuf, core::mem::size_of::<servent>() as DWORD) == 0 {
        return 0;
    }

    (*pse).s_aliases = mswBufferEndPtr(mswBuf).offset_from(bytesOfs) as *mut *mut core::ffi::c_char;

    if !serviceAliasesA.is_null() {
        if mswBufferAppendStrLstA(
            mswBuf,
            serviceAliasesA as *mut *mut core::ffi::c_void,
            -(bytesOfs as LONG_PTR),
        ) == 0
        {
            return 0;
        }
    } else {
        if mswBufferAppendPtr(mswBuf, core::ptr::null_mut()) == 0 {
            return 0;
        }
    }

    (*pse).s_name = mswBufferEndPtr(mswBuf).offset_from(bytesOfs) as *mut core::ffi::c_char;

    if mswBufferAppendStrA(mswBuf, serviceNameA) == 0 {
        return 0;
    }

    (*pse).s_port = htons(port as u_short) as core::ffi::c_short;

    (*pse).s_proto = mswBufferEndPtr(mswBuf).offset_from(bytesOfs) as *mut core::ffi::c_char;

    if mswBufferAppendStrA(mswBuf, protocolNameA) == 0 {
        return 0;
    }

    (*(*lpRes).lpBlob).cbSize = mswBufferEndPtr(mswBuf).offset_from(bytesOfs) as DWORD;
    return (*mswBuf).bufok;
}
#[no_mangle]
pub unsafe extern "C" fn mswBufferAppendAddr_AddrInfoW(
    mswBuf: PMSW_BUFFER,
    lpRes: LPWSAQUERYSETW,
    ip4addr: DWORD,
) -> BOOL {
    let mut paddrinfo: LPCSADDR_INFO;
    let mut psa: LPSOCKADDR_IN;

    (*lpRes).dwNumberOfCsAddrs = 1;
    (*lpRes).lpcsaBuffer = mswBufferEndPtr(mswBuf) as LPCSADDR_INFO;

    paddrinfo = (*lpRes).lpcsaBuffer;

    if mswBufferIncUsed(mswBuf, core::mem::size_of::<_CSADDR_INFO>() as DWORD) == 0 {
        return 0;
    }

    (*paddrinfo).LocalAddr.lpSockaddr = mswBufferEndPtr(mswBuf) as LPSOCKADDR;

    if mswBufferIncUsed(mswBuf, core::mem::size_of::<sockaddr>() as DWORD) == 0 {
        return 0;
    }

    (*paddrinfo).RemoteAddr.lpSockaddr = mswBufferEndPtr(mswBuf) as LPSOCKADDR;

    if mswBufferIncUsed(mswBuf, core::mem::size_of::<sockaddr>() as DWORD) == 0 {
        return 0;
    }

    (*paddrinfo).iSocketType = SOCK_DGRAM as INT;
    (*paddrinfo).iProtocol = IPPROTO_UDP as INT;

    psa = (*paddrinfo).LocalAddr.lpSockaddr as LPSOCKADDR_IN;
    (*paddrinfo).LocalAddr.iSockaddrLength = core::mem::size_of::<sockaddr_in>() as INT;
    (*psa).sin_family = AF_INET as i16;
    (*psa).sin_port = 0;
    (*psa).sin_addr.S_un.S_addr = 0;
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*psa).sin_zero) as *mut u8,
        0,
        core::mem::size_of_val(&(*psa).sin_zero),
    );

    psa = (*paddrinfo).RemoteAddr.lpSockaddr as LPSOCKADDR_IN;
    (*paddrinfo).RemoteAddr.iSockaddrLength = core::mem::size_of::<sockaddr_in>() as INT;
    (*psa).sin_family = AF_INET as i16;
    (*psa).sin_port = 0;
    (*psa).sin_addr.S_un.S_addr = ip4addr;
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*psa).sin_zero) as *mut u8,
        0,
        core::mem::size_of_val(&(*psa).sin_zero),
    );

    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn StrA2WHeapAlloc(
    mut hHeap: HANDLE,
    mut aStr: *mut ::core::ffi::c_char,
) -> *mut WCHAR {
    let mut aStrByteLen: ::core::ffi::c_int;
    let mut wStrByteLen: ::core::ffi::c_int;
    let mut charLen: ::core::ffi::c_int;
    let mut ret: ::core::ffi::c_int;
    let mut wStr: *mut WCHAR;

    if aStr.is_null() {
        return core::ptr::null_mut();
    }

    charLen = (strlen(aStr) as ::core::ffi::c_ulonglong).wrapping_add(1) as ::core::ffi::c_int;

    aStrByteLen = (charLen as usize)
        .wrapping_mul(core::mem::size_of::<::core::ffi::c_char>())
        as ::core::ffi::c_int;
    wStrByteLen = (charLen as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>())
        as ::core::ffi::c_int;

    if hHeap.is_null() {
        hHeap = GetProcessHeap();
    }

    wStr = HeapAlloc(hHeap, 0, wStrByteLen as SIZE_T) as *mut WCHAR;
    if wStr.is_null() {
        HeapFree(hHeap, 0, wStr as PVOID);
        return core::ptr::null_mut();
    }

    ret = MultiByteToWideChar(
        CP_ACP,
        0,
        aStr as LPCSTR,
        aStrByteLen,
        wStr as LPWSTR,
        charLen,
    );

    if ret != charLen {
        HeapFree(hHeap, 0, wStr as PVOID);
        return core::ptr::null_mut();
    }
    return wStr;
}
#[no_mangle]
pub unsafe extern "C" fn StrW2AHeapAlloc(
    mut hHeap: HANDLE,
    wStr: *mut WCHAR,
) -> *mut i8 {
    let mut charLen: i32;
    let mut aStrByteLen: i32;
    let mut ret: i32;
    let mut aStr: *mut i8;

    if wStr.is_null() {
        return core::ptr::null_mut::<i8>();
    }

    charLen = (wcslen(wStr).wrapping_add(1)) as i32;

    aStrByteLen = (charLen as usize)
        .wrapping_mul(core::mem::size_of::<i8>()) as i32;

    if hHeap.is_null() {
        hHeap = GetProcessHeap();
    }

    aStr = HeapAlloc(hHeap, 0, aStrByteLen as SIZE_T) as *mut i8;
    if aStr.is_null() {
        HeapFree(hHeap, 0, aStr as LPVOID);
        return core::ptr::null_mut::<i8>();
    }

    ret = WideCharToMultiByte(
        CP_ACP,
        0,
        wStr as LPCWSTR,
        charLen,
        aStr as LPSTR,
        aStrByteLen,
        core::ptr::null::<i8>(),
        core::ptr::null_mut::<BOOL>(),
    );
    if ret != aStrByteLen {
        HeapFree(hHeap, 0, aStr as LPVOID);
        return core::ptr::null_mut::<i8>();
    }
    return aStr;
}
#[no_mangle]
pub unsafe extern "C" fn StrCpyHeapAllocW(mut hHeap: HANDLE, mut wStr: *mut WCHAR) -> *mut WCHAR {
    let mut chLen: usize;
    let mut bLen: usize;
    let mut resW: *mut WCHAR;

    if wStr.is_null() {
        return core::ptr::null_mut();
    }

    if hHeap.is_null() {
        hHeap = GetProcessHeap();
    }

    chLen = wcslen(wStr);

    bLen = (chLen + 1) * core::mem::size_of::<WCHAR>();

    resW = HeapAlloc(hHeap, 0, bLen as SIZE_T) as *mut WCHAR;
    RtlCopyMemory(resW as *mut core::ffi::c_void, wStr as *const core::ffi::c_void, bLen);
    return resW;
}
#[no_mangle]
pub unsafe extern "C" fn StrCpyHeapAllocA(
    mut hHeap: HANDLE,
    mut aStr: *mut i8,
) -> *mut i8 {
    let mut chLen: usize = 0;
    let mut bLen: usize = 0;
    let mut resA: *mut i8 = core::ptr::null_mut::<i8>();
    
    if aStr.is_null() {
        return core::ptr::null_mut::<i8>();
    }
    
    if hHeap.is_null() {
        hHeap = GetProcessHeap();
    }
    
    chLen = strlen(aStr);
    
    bLen = (chLen as usize)
        .wrapping_add(1 as usize)
        .wrapping_mul(core::mem::size_of::<i8>() as usize) as usize;
    
    resA = HeapAlloc(hHeap, 0 as DWORD, bLen as SIZE_T) as *mut i8;
    core::ptr::copy_nonoverlapping(aStr, resA, bLen);
    return resA;
}

// StrAryCpyHeapAllocA / StrAryCpyHeapAllocWToA — faithful translations of mswhelper.c:529-683.
// These two were DROPPED during the original batch translation (present only in the ORIGINAL C
// SOURCE block below); nsplookup.rs declares + calls them (extern "C"), so define them
// #[no_mangle] extern "C" here so the cross-module cdecl symbol resolves at link. They copy a
// NULL-terminated string ARRAY into one heap block laid out as `ptr1..ptrn NULL, data1..datan`.
#[no_mangle]
pub unsafe extern "C" fn StrAryCpyHeapAllocA(
    mut hHeap: HANDLE,
    aStrAry: *mut PCHAR,
) -> *mut PCHAR {
    let mut aStrByteLen: [DWORD; MAX_ARRAY_SIZE] = [0; MAX_ARRAY_SIZE];
    let mut bLen: usize = 0;
    let mut aCount: i32 = 0;

    if hHeap.is_null() {
        hHeap = GetProcessHeap();
    }

    // Calculating size of array ...
    let mut aSrcPtr: *mut PCHAR = aStrAry;
    while !(*aSrcPtr).is_null() {
        if aCount as usize >= MAX_ARRAY_SIZE {
            return core::ptr::null_mut();
        }
        let bItmLen = strlen(*aSrcPtr as *const i8) + 1;
        aStrByteLen[aCount as usize] = bItmLen as DWORD;
        bLen += core::mem::size_of::<PCHAR>() + bItmLen;
        aSrcPtr = aSrcPtr.add(1);
        aCount += 1;
    }
    // size for NULL-terminator
    bLen += core::mem::size_of::<PCHAR>();

    // get memory
    let resA = HeapAlloc(hHeap, 0 as DWORD, bLen as SIZE_T) as *mut PCHAR;

    // copy data
    aSrcPtr = aStrAry;
    let mut aDstPtr: *mut PCHAR = resA;
    // pos for the first string (past the aCount+1 pointer slots)
    let mut aDstNextStr: *mut i8 = resA.add(aCount as usize + 1) as *mut i8;
    let mut i1: i32 = 0;
    while i1 < aCount {
        let bItmLen = aStrByteLen[i1 as usize] as usize;
        *aDstPtr = aDstNextStr as PCHAR;
        RtlCopyMemory(*aDstPtr as *mut c_void, *aSrcPtr as *const c_void, bItmLen);
        aDstNextStr = (aDstNextStr as usize + bItmLen) as *mut i8;
        aDstPtr = aDstPtr.add(1);
        aSrcPtr = aSrcPtr.add(1);
        i1 += 1;
    }
    // terminate with NULL
    *aDstPtr = core::ptr::null_mut();
    return resA;
}

#[no_mangle]
pub unsafe extern "C" fn StrAryCpyHeapAllocWToA(
    mut hHeap: HANDLE,
    wStrAry: *mut *mut WCHAR,
) -> *mut PCHAR {
    let mut aStrByteLen: [DWORD; MAX_ARRAY_SIZE] = [0; MAX_ARRAY_SIZE];
    let mut bLen: usize = 0;
    let mut aCount: i32 = 0;

    if hHeap.is_null() {
        hHeap = GetProcessHeap();
    }

    // Calculating size of array ...
    let mut wSrcPtr: *mut *mut WCHAR = wStrAry;
    while !(*wSrcPtr).is_null() {
        if aCount as usize >= MAX_ARRAY_SIZE {
            return core::ptr::null_mut();
        }
        let bItmLen = wcslen(*wSrcPtr) + 1;
        aStrByteLen[aCount as usize] = bItmLen as DWORD;
        bLen += core::mem::size_of::<*mut WCHAR>() + bItmLen;
        wSrcPtr = wSrcPtr.add(1);
        aCount += 1;
    }
    // size for NULL-terminator
    bLen += core::mem::size_of::<*mut WCHAR>();

    // get memory
    let resA = HeapAlloc(hHeap, 0 as DWORD, bLen as SIZE_T) as *mut PCHAR;

    // copy data
    wSrcPtr = wStrAry;
    let mut aDstPtr: *mut PCHAR = resA;
    let mut aDstNextStr: *mut i8 = resA.add(aCount as usize + 1) as *mut i8;
    let mut i1: i32 = 0;
    while i1 < aCount {
        let bItmLen = aStrByteLen[i1 as usize] as usize;
        *aDstPtr = aDstNextStr as PCHAR;

        let aStr = HeapAlloc(hHeap, 0 as DWORD, bItmLen as SIZE_T) as *mut i8;
        if aStr.is_null() {
            HeapFree(hHeap, 0 as DWORD, aStr as PVOID);
            return core::ptr::null_mut();
        }
        let ret = WideCharToMultiByte(
            CP_ACP,
            0,
            *wSrcPtr as LPCWSTR,
            bItmLen as i32,
            aStr as LPSTR,
            bItmLen as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        if ret != bItmLen as i32 {
            HeapFree(hHeap, 0 as DWORD, aStr as PVOID);
            return core::ptr::null_mut();
        }
        RtlCopyMemory(*aDstPtr as *mut c_void, aStr as *const c_void, bItmLen);
        HeapFree(hHeap, 0 as DWORD, aStr as PVOID);

        aDstNextStr = (aDstNextStr as usize + bItmLen) as *mut i8;
        aDstPtr = aDstPtr.add(1);
        wSrcPtr = wSrcPtr.add(1);
        i1 += 1;
    }
    // terminate with NULL
    *aDstPtr = core::ptr::null_mut();
    return resA;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
//
// #include "precomp.h"
//
// #include <winuser.h>
// #include <winnls.h>
// #include <wchar.h>
// #include <sal.h>
//
// #include "mswhelper.h"
//
// #define MSW_BUFSIZE 512
// #define MAX_ARRAY_SIZE 5
//
// void
// mswBufferInit( PMSW_BUFFER mswBuf,
//                BYTE* buffer,
//                DWORD bufferSize)
// {
//     RtlZeroMemory(mswBuf, sizeof(*mswBuf));
//     RtlZeroMemory(buffer, bufferSize);
//     mswBuf->bytesMax = bufferSize;
//     mswBuf->buffer = buffer;
//     mswBuf->bufendptr = buffer;
//     mswBuf->bufok = TRUE;
// }
//
// BOOL
// mswBufferCheck( PMSW_BUFFER mswBuf,
//                 DWORD count)
// {
//     if (mswBuf->bytesUsed + count <= mswBuf->bytesMax)
//         return TRUE;
//
//     mswBuf->bufok = FALSE;
//     return FALSE;
// }
//
// BOOL
// mswBufferIncUsed( PMSW_BUFFER mswBuf,
//                   DWORD count)
// {
//     if (!mswBufferCheck(mswBuf, count))
//         return FALSE;
//
//     mswBuf->bytesUsed += count;
//     mswBuf->bufendptr += count;
//     return TRUE;
// }
//
// BYTE*
// mswBufferEndPtr( PMSW_BUFFER mswBuf)
// {
//     return mswBuf->bufendptr;
// }
//
// BOOL
// mswBufferAppend( PMSW_BUFFER mswBuf,
//                  void *dataToAppend,
//                  DWORD dataSize)
// {
//     if (!mswBufferCheck(mswBuf, dataSize))
//         return FALSE;
//
//     RtlCopyMemory(mswBuf->bufendptr, dataToAppend, dataSize);
//     mswBuf->bytesUsed += dataSize;
//     mswBuf->bufendptr += dataSize;
//
//     return TRUE;
// }
//
// BOOL mswBufferAppendStrA( PMSW_BUFFER mswBuf,
//                           char* str)
// {
//     return mswBufferAppend(mswBuf, str, strlen(str) + sizeof(char));
// }
//
// BOOL
// mswBufferAppendStrW( PMSW_BUFFER mswBuf,
//                      WCHAR* str)
// {
//     int bytelen = (wcslen(str) + 1) * sizeof(WCHAR);
//     return mswBufferAppend(mswBuf, str, bytelen);
// }
//
// BOOL
// mswBufferAppendPtr( PMSW_BUFFER mswBuf,
//                     void* ptr)
// {
//     return mswBufferAppend(mswBuf, &ptr, sizeof(ptr));
// }
//
// /* lst = pointer to pointer of items
//
//    *lst[0] = 1st item
//    *lst[1] = 2nd item
//    ...
//    lst[n] = NULL = End of List
//
//    itemLength = data in Bytes for each item.
//
//    ptrofs = delta relative to mswBuf.buffer
// */
// BOOL
// mswBufferAppendLst( PMSW_BUFFER mswBuf,
//                     void **lst,
//                     DWORD itemByteLength,
//                     LONG_PTR ptrofs)
// {
//     DWORD lstItemCount;
//     DWORD lstByteSize;
//     DWORD_PTR lstDataPos;
//     DWORD i1;
//     UINT_PTR *ptrSrcLstPos;
//
//     /* calculate size of list */
//     ptrSrcLstPos = (UINT_PTR*)lst;
//     lstItemCount = 0;
//     while (*ptrSrcLstPos != (UINT_PTR)NULL)
//     {
//         lstItemCount++;
//         ptrSrcLstPos++;
//     }
//
//     lstByteSize = ((lstItemCount + 1) * sizeof(UINT_PTR)) + /* item-pointer + null-ptr (for end) */
//                   (lstItemCount * itemByteLength); /* item-data */
//
//     if (mswBuf->bytesUsed + lstByteSize > mswBuf->bytesMax)
//         return FALSE;
//
//     /* calculate position for the data of the first item */
//     lstDataPos = ((lstItemCount + 1) * sizeof(UINT_PTR)) +
//                  (DWORD_PTR)mswBufferEndPtr(mswBuf);
//     /* add ptrofs */
//     lstDataPos += ptrofs;
//
//     /* write array of Pointer to data */
//     for (i1 = 0; i1 < lstItemCount; i1++)
//     {
//         if (!mswBufferAppendPtr(mswBuf, (void*)lstDataPos))
//             return FALSE;
//
//         lstDataPos += sizeof(UINT_PTR);
//     }
//
//     /* end of list */
//     if (!mswBufferAppendPtr(mswBuf, NULL))
//         return FALSE;
//
//     /* write data */
//     ptrSrcLstPos = (UINT_PTR*)lst;
//     for (i1 = 0; i1 < lstItemCount; i1++)
//     {
//         mswBufferAppend(mswBuf, *(BYTE**)ptrSrcLstPos, itemByteLength);
//         ptrSrcLstPos++;
//     }
//     return mswBuf->bufok;
// }
//
// BOOL
// mswBufferAppendStrLstA( PMSW_BUFFER mswBuf,
//                         void **lst,
//                         LONG_PTR ptrofs)
// {
//     DWORD lstItemLen[MAX_ARRAY_SIZE];
//     DWORD lstItemCount;
//     DWORD lstByteSize;
//     DWORD_PTR lstDataPos;
//     DWORD lstDataSize;
//     DWORD i1;
//     UINT_PTR *ptrSrcLstPos;
//
//     /* calculate size of list */
//     ptrSrcLstPos = (UINT_PTR*)lst;
//     lstItemCount = 0;
//     lstDataSize = 0;
//
//     while (*ptrSrcLstPos != (UINT_PTR)NULL)
//     {
//         if (lstItemCount >= MAX_ARRAY_SIZE)
//             return FALSE;
//
//         i1 = strlen((char*)*ptrSrcLstPos) + sizeof(char);
//         lstItemLen[lstItemCount] = i1;
//         lstItemCount++;
//         lstDataSize += i1;
//         ptrSrcLstPos++;
//     }
//
//     lstByteSize = ((lstItemCount + 1) * sizeof(UINT_PTR)) + /* item-pointer + null-ptr (for end) */
//                   lstDataSize; /* item-data */
//
//     if (mswBuf->bytesUsed + lstByteSize > mswBuf->bytesMax)
//         return FALSE;
//
//     /* calculate position for the data of the first item */
//     lstDataPos = ((lstItemCount + 1) * sizeof(UINT_PTR)) +
//                  (DWORD_PTR)mswBufferEndPtr(mswBuf);
//
//     /* add ptrofs */
//     lstDataPos += ptrofs;
//
//     for (i1 = 0; i1 < lstItemCount; i1++)
//     {
//         if (!mswBufferAppendPtr(mswBuf, (void*)lstDataPos))
//             return FALSE;
//
//         lstDataPos += lstItemLen[i1];
//     }
//
//     /* end of list */
//     if (!mswBufferAppendPtr(mswBuf, NULL))
//         return FALSE;
//
//     /* write data */
//     ptrSrcLstPos = (UINT_PTR*)lst;
//     for (i1 = 0; i1 < lstItemCount; i1++)
//     {
//         if (!mswBufferAppendStrA(mswBuf, *(char**)ptrSrcLstPos))
//             return FALSE;
//
//         ptrSrcLstPos++;
//     }
//     return mswBuf->bufok;
// }
//
// BOOL
// mswBufferAppendBlob_Hostent( PMSW_BUFFER mswBuf,
//                              LPWSAQUERYSETW lpRes,
//                              char** hostAliasesA,
//                              char* hostnameA,
//                              DWORD ip4addr)
// {
//     PHOSTENT phe;
//     void* lst[2];
//     BYTE* bytesOfs;
//
//     /* blob */
//     lpRes->lpBlob = (LPBLOB)mswBufferEndPtr(mswBuf);
//
//     if (!mswBufferIncUsed(mswBuf, sizeof(*lpRes->lpBlob)))
//         return FALSE;
//
//     /* cbSize will be set later */
//     lpRes->lpBlob->cbSize = 0;
//     lpRes->lpBlob->pBlobData = mswBufferEndPtr(mswBuf);
//
//     /* hostent */
//     phe = (PHOSTENT)lpRes->lpBlob->pBlobData;
//     bytesOfs = mswBufferEndPtr(mswBuf);
//
//     if (!mswBufferIncUsed(mswBuf, sizeof(*phe)))
//         return FALSE;
//
//     phe->h_addrtype = AF_INET;
//     phe->h_length = 4; /* 4 Byte (IPv4) */
//
//     /* aliases */
//     phe->h_aliases = (char**)(mswBufferEndPtr(mswBuf) - bytesOfs);
//
//     if (hostAliasesA)
//     {
//         if (!mswBufferAppendStrLstA(mswBuf,
//                                     (void**)hostAliasesA,
//                                     -(LONG_PTR)bytesOfs))
//             return FALSE;
//     }
//     else
//     {
//         if (!mswBufferAppendPtr(mswBuf, NULL))
//             return FALSE;
//     }
//
//     /* addr_list */
//     RtlZeroMemory(lst, sizeof(lst));
//
//     lst[0] = (void*)&ip4addr;
//
//     phe->h_addr_list = (char**)(mswBufferEndPtr(mswBuf) - bytesOfs);
//
//     if (!mswBufferAppendLst(mswBuf, lst, 4, -(LONG_PTR)bytesOfs))
//         return FALSE;
//
//     /* name */
//     phe->h_name = (char*)(mswBufferEndPtr(mswBuf) - bytesOfs);
//
//     if (!mswBufferAppendStrA(mswBuf, hostnameA))
//         return FALSE;
//
//     lpRes->lpBlob->cbSize = (DWORD)(mswBufferEndPtr(mswBuf) - bytesOfs);
//     return mswBuf->bufok;
// }
//
// BOOL
// mswBufferAppendBlob_Servent( PMSW_BUFFER mswBuf,
//                              LPWSAQUERYSETW lpRes,
//                              char* serviceNameA,
//                              char** serviceAliasesA,
//                              char* protocolNameA,
//                              WORD port)
// {
//     PSERVENT pse;
//     BYTE* bytesOfs;
//
//     /* blob */
//     lpRes->lpBlob = (LPBLOB)mswBufferEndPtr(mswBuf);
//
//     if (!mswBufferIncUsed(mswBuf, sizeof(*lpRes->lpBlob)))
//         return FALSE;
//
//     lpRes->lpBlob->cbSize = 0;//later
//     lpRes->lpBlob->pBlobData = mswBufferEndPtr(mswBuf);
//
//     /* servent */
//     pse = (LPSERVENT)lpRes->lpBlob->pBlobData;
//     bytesOfs = mswBufferEndPtr(mswBuf);
//
//     if (!mswBufferIncUsed(mswBuf, sizeof(*pse)))
//         return FALSE;
//
//     pse->s_aliases = (char**)(mswBufferEndPtr(mswBuf) - bytesOfs);
//
//     if (serviceAliasesA)
//     {
//         if (!mswBufferAppendStrLstA(mswBuf,
//                                     (void**)serviceAliasesA,
//                                     -(LONG_PTR)bytesOfs))
//             return FALSE;
//     }
//     else
//     {
//         if (!mswBufferAppendPtr(mswBuf, NULL))
//             return FALSE;
//     }
//
//     pse->s_name = (char*)(mswBufferEndPtr(mswBuf) - bytesOfs);
//
//     if (!mswBufferAppendStrA(mswBuf, serviceNameA))
//         return FALSE;
//
//     pse->s_port = htons(port);
//
//     pse->s_proto = (char*)(mswBufferEndPtr(mswBuf) - bytesOfs);
//
//     if (!mswBufferAppendStrA(mswBuf, protocolNameA))
//         return FALSE;
//
//     lpRes->lpBlob->cbSize = (DWORD)(mswBufferEndPtr(mswBuf) - bytesOfs);
//     return mswBuf->bufok;
// }
//
// BOOL
// mswBufferAppendAddr_AddrInfoW( PMSW_BUFFER mswBuf,
//                                LPWSAQUERYSETW lpRes,
//                                DWORD ip4addr)
// {
//     LPCSADDR_INFO paddrinfo;
//     LPSOCKADDR_IN psa;
//
//     lpRes->dwNumberOfCsAddrs = 1;
//     lpRes->lpcsaBuffer = (LPCSADDR_INFO)mswBufferEndPtr(mswBuf);
//
//     paddrinfo = lpRes->lpcsaBuffer;
//
//     if (!mswBufferIncUsed(mswBuf, sizeof(*paddrinfo)))
//         return FALSE;
//
//     paddrinfo->LocalAddr.lpSockaddr = (LPSOCKADDR)mswBufferEndPtr(mswBuf);
//
//     if (!mswBufferIncUsed(mswBuf, sizeof(*paddrinfo->LocalAddr.lpSockaddr)))
//         return FALSE;
//
//     paddrinfo->RemoteAddr.lpSockaddr = (LPSOCKADDR)mswBufferEndPtr(mswBuf);
//
//     if (!mswBufferIncUsed(mswBuf, sizeof(*paddrinfo->RemoteAddr.lpSockaddr)))
//         return FALSE;
//
//     paddrinfo->iSocketType = SOCK_DGRAM;
//     paddrinfo->iProtocol = IPPROTO_UDP;
//
//     psa = (LPSOCKADDR_IN)paddrinfo->LocalAddr.lpSockaddr;
//     paddrinfo->LocalAddr.iSockaddrLength = sizeof(*psa);
//     psa->sin_family = AF_INET;
//     psa->sin_port = 0;
//     psa->sin_addr.s_addr = 0;
//     RtlZeroMemory(psa->sin_zero, sizeof(psa->sin_zero));
//
//     psa = (LPSOCKADDR_IN)paddrinfo->RemoteAddr.lpSockaddr;
//     paddrinfo->RemoteAddr.iSockaddrLength = sizeof(*psa);
//     psa->sin_family = AF_INET;
//     psa->sin_port = 0;
//     psa->sin_addr.s_addr = ip4addr;
//     RtlZeroMemory(psa->sin_zero, sizeof(psa->sin_zero));
//
//     return TRUE;
// }
//
// /* ansicode <-> unicode */
// WCHAR*
// StrA2WHeapAlloc( HANDLE hHeap,
//                  char* aStr)
// {
//     int aStrByteLen;
//     int wStrByteLen;
//     int charLen;
//     int ret;
//     WCHAR* wStr;
//
//     if (aStr == NULL)
//         return NULL;
//
//     charLen = strlen(aStr) + 1;
//
//     aStrByteLen = (charLen * sizeof(char));
//     wStrByteLen = (charLen * sizeof(WCHAR));
//
//     if (hHeap == 0)
//         hHeap = GetProcessHeap();
//
//     wStr = HeapAlloc(hHeap, 0, wStrByteLen);
//     if (wStr == NULL)
//     {
//         HeapFree(hHeap, 0, wStr);
//         return NULL;
//     }
//
//     ret = MultiByteToWideChar(CP_ACP,
//                               0,
//                               aStr,
//                               aStrByteLen,
//                               wStr,
//                               charLen);
//
//     if (ret != charLen)
//     {
//         HeapFree(hHeap, 0, wStr);
//         return NULL;
//     }
//     return wStr;
// }
//
// char*
// StrW2AHeapAlloc( HANDLE hHeap,
//                  WCHAR* wStr)
// {
//     int charLen;
//     int aStrByteLen;
//     int ret;
//     char* aStr;
//
//     if (wStr == NULL)
//         return NULL;
//
//     charLen = wcslen(wStr) + 1;
//
//     aStrByteLen = (charLen * sizeof(char));
//
//     if (hHeap == 0)
//         hHeap = GetProcessHeap();
//
//     aStr = HeapAlloc(hHeap, 0, aStrByteLen);
//     if (aStr == NULL)
//     {
//         HeapFree(hHeap, 0, aStr);
//         return NULL;
//     }
//
//     ret = WideCharToMultiByte(CP_ACP,
//                               0,
//                               wStr,
//                               charLen,
//                               aStr,
//                               aStrByteLen,
//                               NULL,
//                               NULL);
//     if (ret != aStrByteLen)
//     {
//         HeapFree(hHeap, 0, aStr);
//         return NULL;
//     }
//     return aStr;
// }
//
// WCHAR*
// StrCpyHeapAllocW( HANDLE hHeap,
//                   WCHAR* wStr)
// {
//     size_t chLen;
//     size_t bLen;
//     WCHAR* resW;
//
//     if (wStr == NULL)
//         return NULL;
//
//     if (hHeap == 0)
//         hHeap = GetProcessHeap();
//
//     chLen = wcslen(wStr);
//
//     bLen = (chLen + 1) * sizeof(WCHAR);
//
//     resW = HeapAlloc(hHeap, 0, bLen);
//     RtlCopyMemory(resW, wStr, bLen);
//     return resW;
// }
//
// char*
// StrCpyHeapAllocA( HANDLE hHeap,
//                   char* aStr)
// {
//     size_t chLen;
//     size_t bLen;
//     char* resA;
//
//     if (aStr == NULL)
//         return NULL;
//
//     if (hHeap == 0)
//         hHeap = GetProcessHeap();
//
//     chLen = strlen(aStr);
//
//     bLen = (chLen + 1) * sizeof(char);
//
//     resA = HeapAlloc(hHeap, 0, bLen);
//     RtlCopyMemory(resA, aStr, bLen);
//     return resA;
// }
//
// char**
// StrAryCpyHeapAllocA( HANDLE hHeap,
//                      char** aStrAry)
// {
//     char** aSrcPtr;
//     char** aDstPtr;
//     char* aDstNextStr;
//     DWORD aStrByteLen[MAX_ARRAY_SIZE];
//     size_t bLen;
//     size_t bItmLen;
//     int aCount;
//     int i1;
//     char** resA;
//
//     if (hHeap == 0)
//         hHeap = GetProcessHeap();
//
//     /* Calculating size of array ... */
//     aSrcPtr = aStrAry;
//     bLen = 0;
//     aCount = 0;
//
//     while (*aSrcPtr != NULL)
//     {
//         if (aCount >= MAX_ARRAY_SIZE)
//             return NULL;
//
//         bItmLen = strlen(*aSrcPtr) + 1;
//         aStrByteLen[aCount] = bItmLen;
//
//         bLen += sizeof(*aSrcPtr) + bItmLen;
//
//         aSrcPtr++;
//         aCount++;
//     }
//
//     /* size for NULL-terminator */
//     bLen += sizeof(*aSrcPtr);
//
//     /* get memory */
//     resA = HeapAlloc(hHeap, 0, bLen);
//
//     /* copy data */
//     aSrcPtr = aStrAry;
//     aDstPtr = resA;
//
//     /* pos for the first string */
//     aDstNextStr = (char*)(resA + aCount + 1);
//     for (i1 = 0; i1 < aCount; i1++)
//     {
//         bItmLen = aStrByteLen[i1];
//
//         *aDstPtr = aDstNextStr;
//         RtlCopyMemory(*aDstPtr, *aSrcPtr, bItmLen);
//
//         aDstNextStr = (char*)((DWORD_PTR)aDstNextStr + (DWORD)bItmLen);
//         aDstPtr++;
//         aSrcPtr++;
//     }
//
//     /* terminate with NULL */
//     *aDstPtr = NULL;
//
//     return resA;
// }
//
// char**
// StrAryCpyHeapAllocWToA( HANDLE hHeap,
//      WCHAR** wStrAry)
// {
//     WCHAR** wSrcPtr;
//     char** aDstPtr;
//     char* aDstNextStr;
//     DWORD aStrByteLen[MAX_ARRAY_SIZE];
//     int bLen;
//     int bItmLen;
//     int aCount;
//     int i1;
//     char** resA;
//     int ret;
//     char* aStr;
//
//     if (hHeap == 0)
//         hHeap = GetProcessHeap();
//
//     /* Calculating size of array ... */
//     wSrcPtr = wStrAry;
//     bLen = 0;
//     aCount = 0;
//
//     while (*wSrcPtr != NULL)
//     {
//         if (aCount >= MAX_ARRAY_SIZE)
//             return NULL;
//
//         bItmLen = wcslen(*wSrcPtr) + 1;
//         aStrByteLen[aCount] = bItmLen;
//
//         bLen += sizeof(*wSrcPtr) + bItmLen;
//
//         wSrcPtr++;
//         aCount++;
//     }
//
//     /* size for NULL-terminator */
//     bLen += sizeof(*wSrcPtr);
//
//     /* get memory */
//     resA = HeapAlloc(hHeap, 0, bLen);
//
//     /* copy data */
//     wSrcPtr = wStrAry;
//     aDstPtr = resA;
//
//     /* pos for the first string */
//     aDstNextStr = (char*)(resA + aCount + 1);
//     for (i1 = 0; i1 < aCount; i1++)
//     {
//         bItmLen = aStrByteLen[i1];
//
//         *aDstPtr = aDstNextStr;
//
//         aStr = HeapAlloc(hHeap, 0, bItmLen);
//         if (aStr == NULL)
//         {
//             HeapFree(hHeap, 0, aStr);
//             return NULL;
//         }
//
//         ret = WideCharToMultiByte(CP_ACP,
//                                   0,
//                                   *wSrcPtr,
//                                   bItmLen,
//                                   aStr,
//                                   bItmLen,
//                                   NULL,
//                                   NULL);
//         if (ret != bItmLen)
//         {
//             HeapFree(hHeap, 0, aStr);
//             return NULL;
//         }
//         RtlCopyMemory(*aDstPtr, aStr, bItmLen);
//         HeapFree(hHeap, 0, aStr);
//
//         aDstNextStr = (char*)((DWORD_PTR)aDstNextStr + (DWORD)bItmLen);
//         aDstPtr++;
//         wSrcPtr++;
//     }
//
//     /* terminate with NULL */
//     *aDstPtr = NULL;
//
//     return resA;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
