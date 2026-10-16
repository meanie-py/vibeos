use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn AsyncInetDownload(lpszAgent: LPCWSTR, dwAccessType: DWORD, lpszProxy: LPCWSTR, lpszProxyBypass: LPCWSTR, lpszUrl: LPCWSTR, bAllowCache: BOOL, Callback: ASYNCINET_CALLBACK, Extension: *mut core::ffi::c_void) -> pASYNCINET {
    let mut AsyncInet: pASYNCINET = core::ptr::null_mut();
    let mut bSuccess: BOOL = (FALSE as i32);
    let mut InetOpenUrlFlag: DWORD = 0;
    let mut OldCallbackFunc: INTERNET_STATUS_CALLBACK;

    AsyncInet = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<ASYNCINET>()) as SIZE_T) as pASYNCINET;
    if AsyncInet.is_null()
    {
        OutputDebugStringA(concat!("At File: ", file!(), " HeapAlloc returned 0\n", "\0").as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    (*AsyncInet).bCancelled = (FALSE as i32);

    (*AsyncInet).hInternet = core::ptr::null_mut();
    (*AsyncInet).hInetFile = core::ptr::null_mut();

    (*AsyncInet).Callback = Callback;
    (*AsyncInet).Extension = Extension;

    (*AsyncInet).hEventHandleCreated = CreateEvent(core::ptr::null_mut(), (FALSE as i32), (FALSE as i32), core::ptr::null_mut());

    InitializeCriticalSection(core::ptr::addr_of_mut!((*AsyncInet).CriticalSection));
    (*AsyncInet).ReferenceCnt = 1; // 1 for callee itself
    (*AsyncInet).hEventHandleClose = CreateEvent(core::ptr::null_mut(), (FALSE as i32), (FALSE as i32), core::ptr::null_mut());

    if !(*AsyncInet).hEventHandleCreated.is_null() && !(*AsyncInet).hEventHandleClose.is_null()
    {
        (*AsyncInet).hInternet = InternetOpenW(lpszAgent, dwAccessType, lpszProxy, lpszProxyBypass, INTERNET_FLAG_ASYNC);

        if !(*AsyncInet).hInternet.is_null()
        {
            OldCallbackFunc = InternetSetStatusCallbackW((*AsyncInet).hInternet, Some(AsyncInetStatusCallback));
            if OldCallbackFunc.map_or(0usize, |f| f as usize) != usize::MAX
            {
                InetOpenUrlFlag |= INTERNET_FLAG_PASSIVE | INTERNET_FLAG_RESYNCHRONIZE;
                if bAllowCache == 0
                {
                    InetOpenUrlFlag |= INTERNET_FLAG_DONT_CACHE | INTERNET_FLAG_PRAGMA_NOCACHE | INTERNET_FLAG_RELOAD;
                }

                (*AsyncInet).hInetFile = InternetOpenUrlW((*AsyncInet).hInternet, lpszUrl, core::ptr::null(), 0, InetOpenUrlFlag, AsyncInet as DWORD_PTR);

                if !(*AsyncInet).hInetFile.is_null()
                {
                    // operate complete synchronously
                    bSuccess = (TRUE as i32);
                    AsyncInetReadFileLoop(AsyncInet);
                }
                else
                {
                    if GetLastError() == ERROR_IO_PENDING
                    {
                        // everything fine. waiting for handle created
                        match WaitForSingleObject((*AsyncInet).hEventHandleCreated, INFINITE)
                        {
                            WAIT_OBJECT_0 => {
                                if !(*AsyncInet).hInetFile.is_null()
                                {
                                    bSuccess = (TRUE as i32);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    if bSuccess == 0
    {
        AsyncInetFree(AsyncInet);
        AsyncInet = core::ptr::null_mut();
    }

    if !AsyncInet.is_null()
    {
        // add reference count for caller.
        // the caller is responsible for call AsyncInetRelease when no longer using it.
        AsyncInetAcquire(AsyncInet);
    }
    return AsyncInet;
}
#[no_mangle]
pub unsafe extern "C" fn AsyncInetCancel(AsyncInet: pASYNCINET) -> BOOL {
    if !AsyncInet.is_null() {
        let hInetFile: HINTERNET;
        EnterCriticalSection(core::ptr::addr_of_mut!((*AsyncInet).CriticalSection));
        (*AsyncInet).bCancelled = (TRUE as i32);
        hInetFile = (*AsyncInet).hInetFile;
        (*AsyncInet).hInetFile = core::ptr::null_mut();
        LeaveCriticalSection(core::ptr::addr_of_mut!((*AsyncInet).CriticalSection));

        if !hInetFile.is_null() {
            InternetCloseHandle(hInetFile);
            return (TRUE) as BOOL;
        }
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn AsyncInetIsCanceled(AsyncInet: pASYNCINET) -> BOOL {
    if !AsyncInet.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!((*AsyncInet).CriticalSection));
        if (*AsyncInet).bCancelled != 0 {
            LeaveCriticalSection(core::ptr::addr_of_mut!((*AsyncInet).CriticalSection));
            return (TRUE) as BOOL;
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*AsyncInet).CriticalSection));
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn AsyncInetAcquire(AsyncInet: pASYNCINET) -> BOOL {
    let mut bResult: BOOL = (FALSE as i32);
    if !AsyncInet.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!((*AsyncInet).CriticalSection));
        ATLASSERT!((*AsyncInet).ReferenceCnt > 0);
        if AsyncInetIsCanceled(AsyncInet) == 0 {
            (*AsyncInet).ReferenceCnt += 1;
            bResult = (TRUE as i32);
        }
        // otherwise (AsyncInet->bCleanUp == TRUE)
        // AsyncInetAcquire will return FALSE.
        // In this case, any thread should no longer use this AsyncInet

        LeaveCriticalSection(core::ptr::addr_of_mut!((*AsyncInet).CriticalSection));
    }

    return bResult;
}
#[no_mangle]
pub unsafe extern "C" fn AsyncInetRelease(AsyncInet: pASYNCINET) {
    let mut bCleanUp: BOOL = (FALSE as i32);
    if !AsyncInet.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!((*AsyncInet).CriticalSection));

        ATLASSERT!((*AsyncInet).ReferenceCnt);
        (*AsyncInet).ReferenceCnt -= 1;
        if (*AsyncInet).ReferenceCnt == 0 {
            bCleanUp = (TRUE as i32);
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!((*AsyncInet).CriticalSection));

        if bCleanUp != 0 {
            AsyncInetCleanUp(AsyncInet);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn AsyncInetPerformCallback(AsyncInet: pASYNCINET, Event: ASYNC_EVENT, wParam: WPARAM, lParam: LPARAM) -> i32 {
    if !AsyncInet.is_null() && (*AsyncInet).Callback.is_some() {
        return (*AsyncInet).Callback.unwrap_unchecked()(AsyncInet, Event, wParam, lParam, (*AsyncInet).Extension);
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn AsyncInetStatusCallback(hInternet: HINTERNET, dwContext: DWORD_PTR, dwInternetStatus: DWORD, lpvStatusInformation: LPVOID, dwStatusInformationLength: DWORD) {
    let AsyncInet: pASYNCINET = dwContext as pASYNCINET;
    match dwInternetStatus {
        INTERNET_STATUS_HANDLE_CREATED => {
            'case0: {
                // retrieve handle created by InternetOpenUrlW
                (*AsyncInet).hInetFile = *(lpvStatusInformation as LPHINTERNET);
                SetEvent((*AsyncInet).hEventHandleCreated);
                break 'case0;
            }
        }
        INTERNET_STATUS_HANDLE_CLOSING => {
            'case1: {
                if AsyncInetIsCanceled(AsyncInet) != 0 {
                    AsyncInetPerformCallback(AsyncInet, ASYNCINET_CANCELLED, 0, 0);
                }
                SetEvent((*AsyncInet).hEventHandleClose);
                break 'case1;
            }
        }
        INTERNET_STATUS_REQUEST_COMPLETE => {
            'case5: {
                let AsyncResult: *mut INTERNET_ASYNC_RESULT = lpvStatusInformation as *mut INTERNET_ASYNC_RESULT;

                if (*AsyncInet).hInetFile.is_null() {
                    if AsyncInetIsCanceled(AsyncInet) == 0 {
                        // some error occurs during InternetOpenUrl
                        // and INTERNET_STATUS_HANDLE_CREATED is skipped
                        SetEvent((*AsyncInet).hEventHandleCreated);
                        break 'case5;
                    }
                }

                match (*AsyncResult).dwError {
                    ERROR_SUCCESS => {
                        // there are two possibilities:
                        // 1: InternetOpenUrlW (async mode) complete. (this should be only for the first time)
                        // 2: InternetReadFile (async mode) complete.
                        // so I use bIsOpenUrlComplete field in ASYNCINET to indicate this.

                        if (*AsyncInet).bIsOpenUrlComplete == 0 {
                            // InternetOpenUrlW completed
                            (*AsyncInet).bIsOpenUrlComplete = (TRUE as i32);
                        } else {
                            // asynchronous InternetReadFile complete
                            AsyncInetPerformCallback(AsyncInet, ASYNCINET_DATA, ((*AsyncInet).ReadBuffer).as_ptr() as WPARAM, (*AsyncInet).BytesRead as LPARAM);
                        }

                        AsyncInetReadFileLoop(AsyncInet);
                    }
                    ERROR_INVALID_HANDLE | ERROR_INTERNET_OPERATION_CANCELLED | ERROR_CANCELLED => {
                        if AsyncInetIsCanceled(AsyncInet) != 0 {
                            AsyncInetRelease(AsyncInet);
                            break 'case5;
                        }

                        // fall down
                        // default:
                        // something went wrong
                        if AsyncInetIsCanceled(AsyncInet) != 0 {
                            // sending both ASYNCINET_ERROR and ASYNCINET_CANCELLED may lead to unpredictable behavior
                            // TODO: log the error
                            AsyncInetRelease(AsyncInet);
                        } else {
                            AsyncInetPerformCallback(AsyncInet, ASYNCINET_ERROR, 0, (*AsyncResult).dwError as LPARAM);
                            AsyncInetRelease(AsyncInet);
                        }
                    }
                    _ => {
                        // something went wrong
                        if AsyncInetIsCanceled(AsyncInet) != 0 {
                            // sending both ASYNCINET_ERROR and ASYNCINET_CANCELLED may lead to unpredictable behavior
                            // TODO: log the error
                            AsyncInetRelease(AsyncInet);
                        } else {
                            AsyncInetPerformCallback(AsyncInet, ASYNCINET_ERROR, 0, (*AsyncResult).dwError as LPARAM);
                            AsyncInetRelease(AsyncInet);
                        }
                    }
                }
                break 'case5;
            }
        }
        _ => {}
    }
    return;
}

#[no_mangle]
pub unsafe extern "C" fn AsyncInetReadFileLoop(AsyncInet: pASYNCINET) {
    if AsyncInet.is_null() || (*AsyncInet).hInetFile.is_null() {
        return;
    }

    while 1 != 0 {
        if AsyncInetIsCanceled(AsyncInet) != 0 {
            // abort now.
            AsyncInetRelease(AsyncInet);
            break;
        }

        let bRet: i32 = InternetReadFile(
            (*AsyncInet).hInetFile,
            (*AsyncInet).ReadBuffer.as_mut_ptr() as LPVOID,
            _countof!((*AsyncInet).ReadBuffer) as DWORD,
            core::ptr::addr_of_mut!((*AsyncInet).BytesRead),
        );

        if bRet != 0 {
            if (*AsyncInet).BytesRead == 0 {
                // everything read. now complete.
                AsyncInetPerformCallback(AsyncInet, ASYNCINET_COMPLETE, 0, 0);
                AsyncInetRelease(AsyncInet);
                break;
            } else {
                // read completed immediately.
                AsyncInetPerformCallback(
                    AsyncInet,
                    ASYNCINET_DATA,
                    (*AsyncInet).ReadBuffer.as_mut_ptr() as WPARAM,
                    (*AsyncInet).BytesRead as LPARAM,
                );
            }
        } else {
            let mut dwError: DWORD;
            dwError = GetLastError();
            if dwError == ERROR_IO_PENDING {
                // performing asynchronous IO, everything OK.
                break;
            } else {
                if dwError == ERROR_INVALID_HANDLE
                    || dwError == ERROR_INTERNET_OPERATION_CANCELLED
                    || dwError == ERROR_CANCELLED
                {
                    if AsyncInetIsCanceled(AsyncInet) != 0 {
                        // not an error. just normally cancelling
                        AsyncInetRelease(AsyncInet);
                        break;
                    }
                }

                if AsyncInetIsCanceled(AsyncInet) == 0 {
                    // can not send both ASYNCINET_ERROR and ASYNCINET_CANCELLED
                    AsyncInetPerformCallback(AsyncInet, ASYNCINET_ERROR, 0, dwError as WPARAM as LPARAM);
                } else {
                    // TODO: log the error
                }
                AsyncInetRelease(AsyncInet);
                break;
            }
        }
    }
    return;
}
#[no_mangle]
pub unsafe extern "C" fn AsyncInetCleanUp(AsyncInet: pASYNCINET) -> BOOL {
    if !AsyncInet.is_null() {
        ATLASSERT!((*AsyncInet).ReferenceCnt == 0);
        // close the handle, waiting for all pending request cancelled.

        if (*AsyncInet).bCancelled != 0 {
            // already closed
            (*AsyncInet).hInetFile = core::ptr::null_mut();
        } else if !(*AsyncInet).hInetFile.is_null() {
            InternetCloseHandle((*AsyncInet).hInetFile);
            (*AsyncInet).hInetFile = core::ptr::null_mut();
        }

        // only cleanup when handle closed notification received
        match WaitForSingleObject((*AsyncInet).hEventHandleClose, INFINITE) {
            v if v == WAIT_OBJECT_0 => {
                AsyncInetFree(AsyncInet); // now safe to free the structure
                return (TRUE) as BOOL;
            }
            _ => {
                ATLASSERT!(FALSE);
                AsyncInetFree(AsyncInet);
                return (FALSE) as BOOL;
            }
        }
    } else {
        return (FALSE) as BOOL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn AsyncInetFree(AsyncInet: pASYNCINET) {
    if !AsyncInet.is_null() {
        DeleteCriticalSection(core::ptr::addr_of_mut!((*AsyncInet).CriticalSection));

        if !(*AsyncInet).hEventHandleCreated.is_null() {
            CloseHandle((*AsyncInet).hEventHandleCreated);
            (*AsyncInet).hEventHandleCreated = core::ptr::null_mut();
        }
        if !(*AsyncInet).hEventHandleClose.is_null() {
            CloseHandle((*AsyncInet).hEventHandleClose);
            (*AsyncInet).hEventHandleClose = core::ptr::null_mut();
        }
        if !(*AsyncInet).hInternet.is_null() {
            InternetCloseHandle((*AsyncInet).hInternet);
            (*AsyncInet).hInternet = core::ptr::null_mut();
        }
        if !(*AsyncInet).hInetFile.is_null() {
            InternetCloseHandle((*AsyncInet).hInetFile);
            (*AsyncInet).hInetFile = core::ptr::null_mut();
        }
        HeapFree(GetProcessHeap(), 0, AsyncInet as LPVOID);
    }
}
