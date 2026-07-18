use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn FreeLibraryAndExitThread(hLibModule: HMODULE, dwExitCode: DWORD);
    fn GetModuleHandleA(lpModuleName: LPCSTR) -> HMODULE;
    fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
    fn CreateThread( lpThreadAttributes: LPVOID, dwStackSize: SIZE_T, lpStartAddress: LPVOID, lpParameter: LPVOID, dwCreationFlags: DWORD, lpThreadId: *mut DWORD, ) -> HANDLE;
    fn ResumeThread(hThread: HANDLE) -> DWORD;
    fn QueueUserAPC(pfnAPC: PAPCFUNC, hThread: HANDLE, dwData: ULONG_PTR) -> DWORD;
    fn DeviceIoControl( hDevice: HANDLE, dwIoControlCode: DWORD, lpInBuffer: LPVOID, nInBufferSize: DWORD, lpOutBuffer: LPVOID, nOutBufferSize: DWORD, lpBytesReturned: *mut DWORD, lpOverlapped: *mut OVERLAPPED, ) -> BOOL;
    fn GetLastError() -> DWORD;
    fn OpenSCManagerA(lpMachineName: LPCSTR, lpDatabaseName: LPCSTR, dwDesiredAccess: DWORD) -> SC_HANDLE;
    fn OpenServiceA(hSCManager: SC_HANDLE, lpServiceName: LPCSTR, dwDesiredAccess: DWORD) -> SC_HANDLE;
    fn ChangeServiceConfigA( hService: SC_HANDLE, dwServiceType: DWORD, dwStartType: DWORD, dwErrorControl: DWORD, lpBinaryPathName: LPCSTR, lpLoadOrderGroup: LPCSTR, lpdwTagId: LPDWORD, lpDependencies: LPCSTR, lpServiceStartName: LPCSTR, lpPassword: LPCSTR, lpDisplayName: LPCSTR, ) -> BOOL;
    fn CloseServiceHandle(hSCObject: SC_HANDLE) -> BOOL;
}

// GlobalHeap is defined in dllmain (the module that owns the process-heap handle).
use crate::ws2help::dllmain::GlobalHeap;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const SERVICES_ACTIVE_DATABASEA: [::core::ffi::c_char; 15] =
    unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"ServicesActive\0") };

pub const SERVICES_ACTIVE_DATABASE: [::core::ffi::c_char; 15] = SERVICES_ACTIVE_DATABASEA;

pub static mut pWSASend: LPFN_WSASEND = None;

pub static mut pWSARecv: LPFN_WSARECV = None;

pub static mut pWSASendTo: LPFN_WSASENDTO = None;

pub static mut pWSARecvFrom: LPFN_WSARECVFROM = None;

pub static mut pWSAGetLastError: LPFN_WSAGETLASTERROR = None;

pub static mut pWSACancelBlockingCall: LPFN_WSACANCELBLOCKINGCALL = None;

pub static mut pWSASetBlockingHook: LPFN_WSASETBLOCKINGHOOK = None;

pub static mut pSelect: LPFN_SELECT = None;

pub static mut pWSAStartup: LPFN_WSASTARTUP = None;

pub static mut pWSACleanup: LPFN_WSACLEANUP = None;

pub static mut pGetSockOpt: LPFN_GETSOCKOPT = None;

pub static mut pWSAIoctl: LPFN_WSAIOCTL = None;

#[repr(C)]
pub struct _WSH_HELPER_CONTEXT {
    pub FileHandle: HANDLE,
    pub ThreadHandle: HANDLE,
    pub DllHandle: HANDLE,
}

pub type WSH_HELPER_CONTEXT = _WSH_HELPER_CONTEXT;
pub type PWAH_HELPER_CONTEXT = *mut _WSH_HELPER_CONTEXT;

pub const APCH: HANDLE = 0x534F5220 as HANDLE;
#[no_mangle]
pub unsafe extern "system" fn ApcThread(Context: ULONG_PTR) {

}

#[no_mangle]
pub unsafe extern "system" fn DoSocketCancel(
    Context1: PVOID,
    Context2: PVOID,
    Context3: PVOID,
) -> NTSTATUS {
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DoSocketRequest(
    Context1: PVOID,
    Context2: PVOID,
    Context3: PVOID,
) -> NTSTATUS {
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn ExitThreadApc(Context: ULONG_PTR) {

    let HelperContext = Context as PWAH_HELPER_CONTEXT;
    let DllHandle = (*HelperContext).DllHandle as HMODULE;

    CloseHandle((*HelperContext).FileHandle);

    HeapFree(GlobalHeap, 0, HelperContext as LPVOID);

    FreeLibraryAndExitThread(DllHandle, ERROR_SUCCESS);
}

#[no_mangle]
pub unsafe extern "system" fn WahCloseHandleHelper(HelperHandle: HANDLE) -> DWORD {
    let mut ErrorCode: DWORD;
    let Context: PWAH_HELPER_CONTEXT = HelperHandle as PWAH_HELPER_CONTEXT;

    ErrorCode = WS2HELP_PROLOG();
    if ErrorCode != ERROR_SUCCESS {
        return ErrorCode;
    }

    if Context.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if QueueUserAPC(
        // ExitThreadApc is `extern "system"`; PAPCFUNC is the ABI-identical `stdcall`/cdecl
        // fn-ptr typedef — transmute across the nominal ABI difference.
        core::mem::transmute::<_, PAPCFUNC>(Some(
            ExitThreadApc as unsafe extern "system" fn(ULONG_PTR),
        )),
        (*Context).ThreadHandle,
        Context as ULONG_PTR,
    ) != 0
    {
        return ERROR_SUCCESS;
    } else {
        return ERROR_GEN_FAILURE;
    }
}

#[no_mangle]
pub unsafe extern "system" fn WahCloseSocketHandle(
    HelperHandle: HANDLE,
    Socket: SOCKET,
) -> DWORD {

    let mut ErrorCode: DWORD;
    let Context: PWAH_HELPER_CONTEXT = HelperHandle as PWAH_HELPER_CONTEXT;

    ErrorCode = WS2HELP_PROLOG();
    if ErrorCode != ERROR_SUCCESS {
        return ErrorCode;
    }

    if Context.is_null() || Socket == INVALID_SOCKET || Socket == 0 {
        return ERROR_INVALID_PARAMETER;
    }

    CloseHandle(Socket as HANDLE);
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WahCreateSocketHandle(
    HelperHandle: HANDLE,
    Socket: *mut SOCKET,
) -> DWORD {
    let mut ErrorCode: DWORD;
    let mut OpenType: INT = 0;
    let mut Size: DWORD = core::mem::size_of::<INT>() as DWORD;
    let mut CreateOptions: DWORD = 0;
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut Ea: PFILE_FULL_EA_INFORMATION;
    let mut EaData: PWAH_EA_DATA;
    let mut EaBuffer: [CHAR; 28] = [0; 28];
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Context: PWAH_HELPER_CONTEXT = HelperHandle as PWAH_HELPER_CONTEXT;

    ErrorCode = WS2HELP_PROLOG();
    if ErrorCode != ERROR_SUCCESS {
        return ErrorCode;
    }

    if Context.is_null() || Socket.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    Ea = EaBuffer.as_mut_ptr() as PFILE_FULL_EA_INFORMATION;

    if let Some(pGetSockOptFn) = pGetSockOpt {
        if pGetSockOptFn(
            INVALID_SOCKET,
            SOL_SOCKET as INT,
            SO_OPENTYPE as INT,
            core::ptr::addr_of_mut!(OpenType) as PCHAR,
            core::ptr::addr_of_mut!(Size) as *mut INT,
        ) == ERROR_SUCCESS as INT
            && OpenType != 0
        {
            CreateOptions = FILE_SYNCHRONOUS_IO_NONALERT;
        }
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(Name),
        b"\\Device\\WS2IFSL\\NifsSct\0".as_ptr() as *const u16,
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(Name);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    (*Ea).NextEntryOffset = 0;
    (*Ea).Flags = 0;
    (*Ea).EaNameLength = core::mem::size_of::<[u8; 8]>() as UCHAR;
    (*Ea).EaValueLength = core::mem::size_of::<_WSH_EA_DATA>() as USHORT;
    RtlCopyMemory(
        core::ptr::addr_of_mut!((*Ea).EaName) as PVOID,
        b"NifsSct\0".as_ptr() as PVOID,
        (*Ea).EaNameLength as usize,
    );

    EaData = Ea.offset(1) as PWAH_EA_DATA;

    (*EaData).FileHandle = (*Context).FileHandle;
    (*EaData).Context = core::ptr::null_mut();

    Status = NtCreateFile(
        Socket as PHANDLE,
        FILE_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        0,
        FILE_OPEN_IF,
        CreateOptions,
        Ea as PVOID,
        (core::mem::size_of::<_FILE_FULL_EA_INFORMATION>() + core::mem::size_of::<_WSH_EA_DATA>()) as ULONG,
    );

    if NT_SUCCESS(Status) {
        (*EaData).Context = *Socket as HANDLE;

        if DeviceIoControl(
            *Socket as HANDLE,
            IOCTL_WS2IFSL_SET_HANDLE,
            core::ptr::addr_of_mut!(EaData) as LPVOID,
            core::mem::size_of::<WSH_EA_DATA>() as DWORD,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(Size),
            core::ptr::null_mut(),
        ) != 0
        {
            ErrorCode = NO_ERROR;
        } else {
            ErrorCode = GetLastError();
            CloseHandle(*Socket as HANDLE);
            *Socket = 0;
        }
    } else {
        ErrorCode = RtlNtStatusToDosError(Status);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn WahDisableNonIFSHandleSupport() -> INT {
    let mut ErrorCode: DWORD;
    let mut ServiceMgrHandle: SC_HANDLE;
    let mut Ws2IfsHandle: SC_HANDLE;

    ErrorCode = WS2HELP_PROLOG();
    if ErrorCode != ERROR_SUCCESS {
        return ErrorCode as INT;
    }

    ServiceMgrHandle = OpenSCManagerA(
        core::ptr::null(),
        b"ServicesActive\0".as_ptr() as LPCSTR,
        SC_MANAGER_CREATE_SERVICE,
    );
    if ServiceMgrHandle.is_null() {
        return GetLastError() as INT;
    }

    Ws2IfsHandle = OpenServiceA(
        ServiceMgrHandle,
        b"WS2IFSL\0".as_ptr() as LPCSTR,
        SERVICE_ALL_ACCESS,
    );

    ChangeServiceConfigA(
        Ws2IfsHandle,
        SERVICE_NO_CHANGE,
        SERVICE_DISABLED,
        SERVICE_NO_CHANGE,
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null(),
    );

    CloseServiceHandle(ServiceMgrHandle);
    CloseServiceHandle(Ws2IfsHandle);
    return ERROR_SUCCESS as INT;
}

type SC_HANDLE = *mut core::ffi::c_void;
type LPCSTR = *const CHAR;
type CHAR = i8;
type BOOL = i32;
type LPDWORD = *mut DWORD;
type INT = i32;
type DWORD = u32;

const SC_MANAGER_CREATE_SERVICE: DWORD = 0x0002;
const SERVICE_ALL_ACCESS: DWORD = 0xF01FF;
const SERVICE_NO_CHANGE: DWORD = 0xFFFFFFFF;

#[no_mangle]
pub unsafe extern "system" fn WahEnableNonIFSHandleSupport() -> INT {
    return 0 as INT;
}

#[no_mangle]
pub unsafe extern "system" fn WahOpenHandleHelper(HelperHandle: PHANDLE) -> DWORD {
    let mut ErrorCode: DWORD;
    let mut hWs2_32: HINSTANCE;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Ea: PFILE_FULL_EA_INFORMATION;
    let mut EaData: PWAH_EA_DATA2;
    let mut EaBuffer: [CHAR; 52] = [0; 52];
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Tid: DWORD = 0;
    let mut Context: PWAH_HELPER_CONTEXT;

    ErrorCode = WS2HELP_PROLOG();
    if ErrorCode != ERROR_SUCCESS {
        return ErrorCode;
    }

    if HelperHandle.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    hWs2_32 = GetModuleHandleA(b"WS2_32.DLL\0".as_ptr() as LPCSTR);
    if hWs2_32.is_null() {
        return WSASYSCALLFAILURE;
    }

    pGetSockOpt = core::mem::transmute(GetProcAddress(hWs2_32, b"getsockopt\0".as_ptr() as LPCSTR));
    if pGetSockOpt.is_none() {
        return WSASYSCALLFAILURE;
    }
    pSelect = core::mem::transmute(GetProcAddress(hWs2_32, b"select\0".as_ptr() as LPCSTR));
    if pSelect.is_none() {
        return WSASYSCALLFAILURE;
    }
    pWSACancelBlockingCall = core::mem::transmute(GetProcAddress(hWs2_32, b"WSACancelBlockingCall\0".as_ptr() as LPCSTR));
    if pWSACancelBlockingCall.is_none() {
        return WSASYSCALLFAILURE;
    }
    pWSACleanup = core::mem::transmute(GetProcAddress(hWs2_32, b"WSACleanup\0".as_ptr() as LPCSTR));
    if pWSACleanup.is_none() {
        return WSASYSCALLFAILURE;
    }
    pWSAGetLastError = core::mem::transmute(GetProcAddress(hWs2_32, b"WSAGetLastError\0".as_ptr() as LPCSTR));
    if pWSAGetLastError.is_none() {
        return WSASYSCALLFAILURE;
    }
    pWSASetBlockingHook = core::mem::transmute(GetProcAddress(hWs2_32, b"WSASetBlockingHook\0".as_ptr() as LPCSTR));
    if pWSASetBlockingHook.is_none() {
        return WSASYSCALLFAILURE;
    }
    pWSARecv = core::mem::transmute(GetProcAddress(hWs2_32, b"WSARecv\0".as_ptr() as LPCSTR));
    if pWSARecv.is_none() {
        return WSASYSCALLFAILURE;
    }
    pWSASend = core::mem::transmute(GetProcAddress(hWs2_32, b"WSASend\0".as_ptr() as LPCSTR));
    if pWSASend.is_none() {
        return WSASYSCALLFAILURE;
    }
    pWSASendTo = core::mem::transmute(GetProcAddress(hWs2_32, b"WSASendTo\0".as_ptr() as LPCSTR));
    if pWSASendTo.is_none() {
        return WSASYSCALLFAILURE;
    }
    pWSAStartup = core::mem::transmute(GetProcAddress(hWs2_32, b"WSAStartup\0".as_ptr() as LPCSTR));
    if pWSAStartup.is_none() {
        return WSASYSCALLFAILURE;
    }
    pWSARecvFrom = core::mem::transmute(GetProcAddress(hWs2_32, b"WSARecvFrom\0".as_ptr() as LPCSTR));
    if pWSARecvFrom.is_none() {
        return WSASYSCALLFAILURE;
    }
    pWSAIoctl = core::mem::transmute(GetProcAddress(hWs2_32, b"WSAIoctl\0".as_ptr() as LPCSTR));
    if pWSAIoctl.is_none() {
        return WSASYSCALLFAILURE;
    }

    Ea = EaBuffer.as_mut_ptr() as PFILE_FULL_EA_INFORMATION;

    Context = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<WSH_HELPER_CONTEXT>() as SIZE_T) as PWAH_HELPER_CONTEXT;
    if !Context.is_null() {
        (*Context).ThreadHandle = CreateThread(
            core::ptr::null_mut(),
            0,
            core::mem::transmute(ApcThread as *const ()),
            Context as PVOID,
            CREATE_SUSPENDED,
            core::ptr::addr_of_mut!(Tid)
        );
        if !(*Context).ThreadHandle.is_null() {
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(Name),
                b"\\Device\\WS2IFSL\\NifsPvd\0".as_ptr() as *const u16
            );
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.Attributes = 0;
            ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(Name);
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            (*Ea).NextEntryOffset = 0;
            (*Ea).Flags = 0;
            (*Ea).EaNameLength = core::mem::size_of::<[i8; 8]>() as u8;
            (*Ea).EaValueLength = core::mem::size_of::<_WAH_EA_DATA2>() as u16;
            RtlCopyMemory(
                core::ptr::addr_of_mut!((*Ea).EaName) as PVOID,
                b"NifsPvd\0".as_ptr() as PVOID,
                (*Ea).EaNameLength as usize
            );

            EaData = Ea.offset(1) as PWAH_EA_DATA2;

            (*EaData).ThreadHandle = (*Context).ThreadHandle;
            (*EaData).RequestRoutine = core::mem::transmute(DoSocketRequest as *const ());
            (*EaData).CancelRoutine = core::mem::transmute(DoSocketCancel as *const ());
            (*EaData).ApcContext = Context as PVOID;
            (*EaData).Reserved = 0;

            Status = NtCreateFile(
                core::ptr::addr_of_mut!((*Context).FileHandle),
                FILE_ALL_ACCESS,
                core::ptr::addr_of_mut!(ObjectAttributes),
                core::ptr::addr_of_mut!(IoStatusBlock),
                core::ptr::null_mut(),
                FILE_ATTRIBUTE_NORMAL,
                0,
                FILE_OPEN_IF,
                0,
                Ea as PVOID,
                (core::mem::size_of::<_FILE_FULL_EA_INFORMATION>() + core::mem::size_of::<_WAH_EA_DATA2>()) as ULONG
            );

            if NT_SUCCESS(Status) {
                ResumeThread((*Context).ThreadHandle);
                *HelperHandle = Context as HANDLE;
                return ERROR_SUCCESS;
            } else {
                ErrorCode = RtlNtStatusToDosError(Status);
            }

            (*Context).FileHandle = core::ptr::null_mut();
            ResumeThread((*Context).ThreadHandle);
        } else {
            ErrorCode = GetLastError();
        }

        HeapFree(GlobalHeap, 0, Context as PVOID);
    } else {
        ErrorCode = GetLastError();
    }

    return ErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn WahCompleteRequest(
    _HelperHandle: HANDLE,
    _Socket: SOCKET,
    _lpOverlapped: LPWSAOVERLAPPED,
    _ErrorCode: DWORD,
    _BytesTransferred: DWORD,
) -> DWORD {
    ERROR_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 DLL
//  * FILE:        dll/win32/ws2help/handle.c
//  * PURPOSE:     WinSock 2 DLL header
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "precomp.h"
//
// #include <ndk/iofuncs.h>
//
// #include "wshdrv.h"
//
// /* DATA **********************************************************************/
//
// typedef struct _WSH_HELPER_CONTEXT
// {
//     HANDLE FileHandle;
//     HANDLE ThreadHandle;
//     HANDLE DllHandle;
// } WSH_HELPER_CONTEXT, *PWAH_HELPER_CONTEXT;
//
// LPFN_WSASEND pWSASend;
// LPFN_WSARECV pWSARecv;
// LPFN_WSASENDTO pWSASendTo;
// LPFN_WSARECVFROM pWSARecvFrom;
// LPFN_WSAGETLASTERROR pWSAGetLastError;
// LPFN_WSACANCELBLOCKINGCALL pWSACancelBlockingCall;
// LPFN_WSASETBLOCKINGHOOK pWSASetBlockingHook;
// LPFN_SELECT pSelect;
// LPFN_WSASTARTUP pWSAStartup;
// LPFN_WSACLEANUP pWSACleanup;
// LPFN_GETSOCKOPT pGetSockOpt;
// LPFN_WSAIOCTL pWSAIoctl;
//
// #define APCH (HANDLE)'SOR '
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// __stdcall
// ApcThread(ULONG_PTR Context)
// {
//
// }
//
// NTSTATUS
// WINAPI
// DoSocketCancel(PVOID Context1,
//                PVOID Context2,
//                PVOID Context3)
// {
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// WINAPI
// DoSocketRequest(PVOID Context1,
//                 PVOID Context2,
//                 PVOID Context3)
// {
//     return STATUS_SUCCESS;
// }
//
// VOID
// __stdcall
// ExitThreadApc(ULONG_PTR Context)
// {
//     PWAH_HELPER_CONTEXT HelperContext = (PWAH_HELPER_CONTEXT)Context;
//     HMODULE DllHandle = HelperContext->DllHandle;
//
//     /* Close the file handle */
//     CloseHandle(HelperContext->FileHandle);
//
//     /* Free the context */
//     HeapFree(GlobalHeap, 0, HelperContext);
//
//     /* Exit the thread and library */
//     FreeLibraryAndExitThread(DllHandle, ERROR_SUCCESS);
// }
//
// DWORD
// WINAPI
// WahCloseHandleHelper(IN HANDLE HelperHandle)
// {
//     DWORD ErrorCode;
//     PWAH_HELPER_CONTEXT Context = HelperHandle;
//
//     /* Enter the prolog, make sure we're initialized */
//     ErrorCode = WS2HELP_PROLOG();
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//
//     /* Validate the handle */
//     if (!Context) return ERROR_INVALID_PARAMETER;
//
//     /* Queue an APC to exit the thread */
//     if (QueueUserAPC(ExitThreadApc, Context->ThreadHandle, (ULONG_PTR)Context))
//     {
//         /* Done */
//         return ERROR_SUCCESS;
//     }
//     else
//     {
//         /* We failed somewhere */
//         return ERROR_GEN_FAILURE;
//     }
// }
//
// DWORD
// WINAPI
// WahCloseSocketHandle(IN HANDLE HelperHandle,
//                      IN SOCKET Socket)
// {
//     DWORD ErrorCode;
//     PWAH_HELPER_CONTEXT Context = HelperHandle;
//
//     /* Enter the prolog, make sure we're initialized */
//     ErrorCode = WS2HELP_PROLOG();
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//
//     /* Validate the handle */
//     if (!(Context) || (Socket == INVALID_SOCKET) || !(Socket))
//     {
//         /* Invalid handle and/or socket */
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Just close the handle and return */
//     CloseHandle((HANDLE)Socket);
//     return ERROR_SUCCESS;
// }
//
// DWORD
// WINAPI
// WahCreateSocketHandle(IN HANDLE HelperHandle,
//                       OUT SOCKET *Socket)
// {
//     DWORD ErrorCode;
//     INT OpenType;
//     DWORD Size = sizeof(OpenType);
//     DWORD CreateOptions = 0;
//     UNICODE_STRING Name;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PFILE_FULL_EA_INFORMATION Ea;
//     PWAH_EA_DATA EaData;
//     CHAR EaBuffer[sizeof(*Ea) + sizeof(*EaData)];
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
// 	PWAH_HELPER_CONTEXT	Context = (PWAH_HELPER_CONTEXT)HelperHandle;
//
//     /* Enter the prolog, make sure we're initialized */
//     ErrorCode = WS2HELP_PROLOG();
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//
//     /* Validate the handle and pointer */
//     if (!(Context) || !(Socket))
//     {
//         /* Invalid handle and/or socket */
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Set pointer to EA */
//     Ea = (PFILE_FULL_EA_INFORMATION)EaBuffer;
//
//     /* Get the open type to determine the create options */
//     if ((pGetSockOpt(INVALID_SOCKET,
//                      SOL_SOCKET,
//                      SO_OPENTYPE,
//                      (PCHAR)&OpenType,
//                      (INT *)&Size) == ERROR_SUCCESS) && (OpenType))
//     {
//         /* This is a sync open */
//         CreateOptions = FILE_SYNCHRONOUS_IO_NONALERT;
//     }
//
//     /* Initialize the attributes for the driver */
//     RtlInitUnicodeString(&Name, L"\\Device\\WS2IFSL\\NifsSct");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Name,
//                                0,
//                                NULL,
//                                NULL);
//
//     /* Set up the EA */
//     Ea->NextEntryOffset = 0;
//     Ea->Flags = 0;
//     Ea->EaNameLength = sizeof("NifsSct");
//     Ea->EaValueLength = sizeof(*EaData);
//     RtlCopyMemory(Ea->EaName, "NifsSct", Ea->EaNameLength);
//
//     /* Get our EA data */
//     EaData = (PWAH_EA_DATA)(Ea + 1);
//
//     /* Write the EA Data */
//     EaData->FileHandle = Context->FileHandle;
//     EaData->Context = NULL;
//
//     /* Call the driver */
//     Status = NtCreateFile((PHANDLE)Socket,
//                            FILE_ALL_ACCESS,
//                            &ObjectAttributes,
//                            &IoStatusBlock,
//                            NULL,
//                            FILE_ATTRIBUTE_NORMAL,
//                            0,
//                            FILE_OPEN_IF,
//                            CreateOptions,
//                            Ea,
//                            sizeof(*Ea) + sizeof(*EaData));
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Write the socket handle */
//         EaData->Context =(HANDLE)*Socket;
//
//         /* Tell the driver about it */
//         if (DeviceIoControl((HANDLE)*Socket,
//                             IOCTL_WS2IFSL_SET_HANDLE,
//                             &EaData,
//                             sizeof(WSH_EA_DATA),
//                             NULL,
//                             0,
//                             &Size,
//                             NULL))
//         {
//             /* Set success */
//             ErrorCode = NO_ERROR;
//         }
//         else
//         {
//             /* We failed. Get the error and close the socket */
//             ErrorCode = GetLastError();
//             CloseHandle((HANDLE)*Socket);
//             *Socket = 0;
//         }
//     }
//     else
//     {
//         /* Create file failed, conver error code */
//         ErrorCode = RtlNtStatusToDosError(Status);
//     }
//
//     /* Return to caller */
//     return ErrorCode;
// }
//
// INT
// WINAPI
// WahDisableNonIFSHandleSupport(VOID)
// {
//     DWORD ErrorCode;
//     SC_HANDLE ServiceMgrHandle, Ws2IfsHandle;
//
//     /* Enter the prolog, make sure we're initialized */
//     ErrorCode = WS2HELP_PROLOG();
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//
//     /* Open the service DB */
//     ServiceMgrHandle = OpenSCManager(NULL,
//                                      SERVICES_ACTIVE_DATABASE,
//                                      SC_MANAGER_CREATE_SERVICE);
//     if (!ServiceMgrHandle) return GetLastError();
//
//     /* Open the service */
//     Ws2IfsHandle = OpenService(ServiceMgrHandle, "WS2IFSL", SERVICE_ALL_ACCESS);
//
//     /* Disable the service */
//     ChangeServiceConfig(Ws2IfsHandle,
//                         SERVICE_NO_CHANGE,
//                         SERVICE_DISABLED,
//                         SERVICE_NO_CHANGE,
//                         NULL,
//                         NULL,
//                         NULL,
//                         NULL,
//                         NULL,
//                         NULL,
//                         NULL);
//
//     /* Close the handles and return */
//     CloseServiceHandle(ServiceMgrHandle);
//     CloseServiceHandle(Ws2IfsHandle);
//     return ERROR_SUCCESS;
// }
//
// INT
// WINAPI
// WahEnableNonIFSHandleSupport(VOID)
// {
//     return 0;
// }
//
// DWORD
// WINAPI
// WahOpenHandleHelper(OUT PHANDLE HelperHandle)
// {
//     DWORD ErrorCode;
//     HINSTANCE hWs2_32;
//     UNICODE_STRING Name;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PFILE_FULL_EA_INFORMATION Ea;
//     PWAH_EA_DATA2 EaData;
//     CHAR EaBuffer[sizeof(*Ea) + sizeof(*EaData)];
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     DWORD Tid;
//     PWAH_HELPER_CONTEXT Context;
//
//     /* Enter the prolog, make sure we're initialized */
//     ErrorCode = WS2HELP_PROLOG();
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//
//     /* Validate the pointer */
//     if (!HelperHandle)
//     {
//         /* Invalid handle and/or socket */
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Get ws2_32.dll's handle. We don't load it: it MUST be here */
//     hWs2_32 = GetModuleHandle ("WS2_32.DLL");
//     if (!hWs2_32) return WSASYSCALLFAILURE;
//
//     /* Dynamically load all its APIs */
//     if (!((pGetSockOpt = (LPFN_GETSOCKOPT)GetProcAddress(hWs2_32, "getsockopt"))) ||
//         !((pSelect = (LPFN_SELECT)GetProcAddress(hWs2_32, "select"))) ||
//         !((pWSACancelBlockingCall = (LPFN_WSACANCELBLOCKINGCALL)GetProcAddress(hWs2_32, "WSACancelBlockingCall"))) ||
//         !((pWSACleanup = (LPFN_WSACLEANUP)GetProcAddress(hWs2_32, "WSACleanup"))) ||
//         !((pWSAGetLastError = (LPFN_WSAGETLASTERROR)GetProcAddress(hWs2_32, "WSAGetLastError"))) ||
//         !((pWSASetBlockingHook = (LPFN_WSASETBLOCKINGHOOK)GetProcAddress(hWs2_32, "WSASetBlockingHook"))) ||
//         !((pWSARecv = (LPFN_WSARECV)GetProcAddress(hWs2_32, "WSARecv"))) ||
//         !((pWSASend = (LPFN_WSASEND)GetProcAddress(hWs2_32, "WSASend"))) ||
//         !((pWSASendTo = (LPFN_WSASENDTO)GetProcAddress(hWs2_32, "WSASendTo"))) ||
//         !((pWSAStartup = (LPFN_WSASTARTUP)GetProcAddress(hWs2_32, "WSAStartup"))) ||
//         !((pWSARecvFrom = (LPFN_WSARECVFROM)GetProcAddress(hWs2_32, "WSARecvFrom"))) ||
//         !((pWSAIoctl = (LPFN_WSAIOCTL)GetProcAddress(hWs2_32, "WSAIoctl"))))
//     {
//         /* Uh, guess we failed somewhere */
//         return WSASYSCALLFAILURE;
//     }
//
//     /* Set pointer EA structure */
//     Ea = (PFILE_FULL_EA_INFORMATION)EaBuffer;
//
//     /* Create the helper context */
//     Context = HeapAlloc(GlobalHeap, 0, sizeof(WSH_HELPER_CONTEXT));
//     if (Context)
//     {
//         /* Create the special request thread */
// 		Context->ThreadHandle = CreateThread(NULL,
//                                              0,
//                                              (PVOID)ApcThread,
//                                              Context,
//                                              CREATE_SUSPENDED,
//                                              &Tid);
// 		if (Context->ThreadHandle)
//         {
//             /* Create the attributes for the driver open */
// 			RtlInitUnicodeString(&Name,  L"\\Device\\WS2IFSL\\NifsPvd");
// 			InitializeObjectAttributes(&ObjectAttributes,
// 								       &Name,
//                                        0,
//                                        NULL,
//                                        NULL);
//
//             /* Setup the EA */
// 			Ea->NextEntryOffset = 0;
// 			Ea->Flags = 0;
// 			Ea->EaNameLength = sizeof("NifsPvd");
// 			Ea->EaValueLength = sizeof(*EaData);
// 			RtlCopyMemory(Ea->EaName, "NifsPvd", Ea->EaNameLength);
//
//             /* Get our EA data */
//             EaData = (PWAH_EA_DATA2)(Ea + 1);
//
//             /* Fill out the EA Data */
// 			EaData->ThreadHandle = Context->ThreadHandle;
// 			EaData->RequestRoutine = DoSocketRequest;
// 			EaData->CancelRoutine = DoSocketCancel;
// 			EaData->ApcContext = Context;
// 			EaData->Reserved = 0;
//
//             /* Call the driver */
// 			Status = NtCreateFile(&Context->FileHandle,
// 								  FILE_ALL_ACCESS,
// 								  &ObjectAttributes,
// 								  &IoStatusBlock,
// 								  NULL,
// 								  FILE_ATTRIBUTE_NORMAL,
// 								  0,
// 								  FILE_OPEN_IF,
// 								  0,
// 								  Ea,
// 								  sizeof(*Ea) + sizeof(*EaData));
//
//             /* Check for success */
//             if (NT_SUCCESS(Status))
//             {
//                 /* Resume the thread and return a handle to the context */
//                 ResumeThread(Context->ThreadHandle);
// 				*HelperHandle = (HANDLE)Context;
// 				return ERROR_SUCCESS;
// 			}
// 		    else
//             {
//                 /* Get the error code */
// 			    ErrorCode = RtlNtStatusToDosError(Status);
//             }
//
//             /* We failed, mark us as such */
//             Context->FileHandle = NULL;
//             ResumeThread(Context->ThreadHandle);
//         }
// 		else
//         {
//             /* Get the error code */
// 			ErrorCode = GetLastError();
// 		}
//
//         /* If we got here, we failed, so free the context */
// 		HeapFree(GlobalHeap, 0, Context);
//     }
//     else
//     {
//         /* Get the error code */
//         ErrorCode = GetLastError();
//     }
//
//     /* Return to caller */
//     return ErrorCode;
// }
//
// DWORD
// WINAPI
// WahCompleteRequest(IN HANDLE HelperHandle,
//                    IN SOCKET Socket,
//                    IN LPWSAOVERLAPPED lpOverlapped,
//                    IN DWORD ErrorCode,
//                    IN DWORD BytesTransferred)
// {
//     UNREFERENCED_PARAMETER(HelperHandle);
//     UNREFERENCED_PARAMETER(Socket);
//     UNREFERENCED_PARAMETER(lpOverlapped);
//     UNREFERENCED_PARAMETER(ErrorCode);
//     UNREFERENCED_PARAMETER(BytesTransferred);
//     return ERROR_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
