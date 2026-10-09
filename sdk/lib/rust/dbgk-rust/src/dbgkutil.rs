use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;

use ntkrnl_sys::ddk::{
    _EPROCESS, _ETHREAD, _UNICODE_STRING, _OBJECT_ATTRIBUTES, _IO_STATUS_BLOCK, IMAGE_INFO,
    PEPROCESS, PETHREAD, PTEB, POBJECT_NAME_INFORMATION, PIMAGE_NT_HEADERS, HANDLE,
    DBGKM_MSG, DBGKM_CREATE_PROCESS, DBGKM_CREATE_THREAD,
    DBGKM_EXIT_PROCESS, DBGKM_EXIT_THREAD, DBGKM_LOAD_DLL, DBGKM_UNLOAD_DLL,
    PDBGKM_CREATE_PROCESS, PDBGKM_CREATE_THREAD,
    PDBGKM_EXIT_PROCESS, PDBGKM_EXIT_THREAD,
    PDBGKM_LOAD_DLL, PDBGKM_UNLOAD_DLL,
    MmGetFileNameForSection, ZwOpenFile, ExFreePool,
    DbgkpSendApiMessage, ObCloseHandle, RtlImageNtHeader,
    KeFreezeAllThreads, KeThawAllThreads,
    PspRunLoadImageNotifyRoutines, RtlInitUnicodeString,
    ExGetPreviousMode, NtCurrentTeb,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn InitializeObjectAttributes(
        p: *mut _OBJECT_ATTRIBUTES,
        n: *mut _UNICODE_STRING,
        a: ULONG,
        r: HANDLE,
        s: PVOID,
    );

    static PspSystemDllBase: PVOID;
    static PsNtDllPathName: _UNICODE_STRING;
    static PsImageNotifyEnabled: BOOLEAN;
}
extern "system" {
    #[link_name = "IoGetCurrentProcess"]
    fn PsGetCurrentProcess() -> PEPROCESS;
    fn PsGetCurrentThread() -> PETHREAD;
}

const PSF_CREATE_REPORTED_BIT: ULONG = 0x00000001;
const PSF_IMAGE_NOTIFY_DONE_BIT: ULONG = 0x400000;

const IMAGE_ADDRESSING_MODE_32BIT: ULONG = 0x3;

const FILE_SHARE_READ: ULONG = 0x00000001;
const FILE_SHARE_WRITE: ULONG = 0x00000002;
const FILE_SHARE_DELETE: ULONG = 0x00000004;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x00000020;

const DbgKmCreateProcessApi: ULONG = 2; // header value; the hand copy said 0
const DbgKmLoadDllApi: ULONG = 5; // header value; the hand copy said 1
const DbgKmCreateThreadApi: ULONG = 1; // header value; the hand copy said 2
const DbgKmExitProcessApi: ULONG = 4; // header value; the hand copy said 3
const DbgKmExitThreadApi: ULONG = 3; // header value; the hand copy said 4
const DbgKmUnloadDllApi: ULONG = 6; // header value; the hand copy said 5

const LPC_DEBUG_EVENT: i16 = 0x08;

extern "C" {
    fn PspSetProcessFlag(Process: PEPROCESS, Flags: ULONG) -> ULONG;
}
extern "system" {
    fn KeQuerySystemTime(current_time: PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpSectionToFileHandle(Section: PVOID) -> HANDLE {
    let Section = opaque_ptr(Section);
    let mut file_name: POBJECT_NAME_INFORMATION = core::ptr::null_mut();
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut handle: HANDLE = core::ptr::null_mut();

    let status = MmGetFileNameForSection(Section, &mut file_name);
    if status < 0 {
        return core::ptr::null_mut();
    }

    InitializeObjectAttributes(
        &mut object_attributes,
        core::ptr::addr_of_mut!((*file_name).Name),
        OBJ_CASE_INSENSITIVE | OBJ_FORCE_ACCESS_CHECK | OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    let status = ZwOpenFile(
        &mut handle,
        GENERIC_READ | SYNCHRONIZE,
        &mut object_attributes,
        &mut io_status_block,
        FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );

    ExFreePool(file_name as PVOID);
    
    if status < 0 {
        return core::ptr::null_mut();
    }
    handle
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpSuspendProcess() -> BOOLEAN {
    let process = PsGetCurrentProcess() as *mut _EPROCESS;
    
    if (*process).__bindgen_anon_2.__bindgen_anon_1.ProcessDelete() == 0 {
        KeFreezeAllThreads();
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpResumeProcess() {
    KeThawAllThreads();
}

#[no_mangle]
#[allow(unused_variables)]
pub unsafe extern "system" fn DbgkCreateThread(Thread: PETHREAD, StartAddress: PVOID) {
    let Thread = opaque_ptr(Thread);
    let StartAddress = opaque_ptr(StartAddress);
    let process = PsGetCurrentProcess();
    let process_ptr = process as *mut _EPROCESS;
    let mut process_flags: ULONG;
    let mut image_info: IMAGE_INFO = core::mem::zeroed();
    let mut nt_header: PIMAGE_NT_HEADERS;
    let mut module_name: POBJECT_NAME_INFORMATION = core::ptr::null_mut();
    let mut ntdll_name: _UNICODE_STRING = core::mem::zeroed();
    let mut status: NTSTATUS;
    let debug_port: PVOID;
    let mut api_message: DBGKM_MSG = core::mem::zeroed();
    let create_thread: PDBGKM_CREATE_THREAD = core::ptr::addr_of_mut!(api_message.__bindgen_anon_1.CreateThread);
    let create_process: PDBGKM_CREATE_PROCESS = core::ptr::addr_of_mut!(api_message.__bindgen_anon_1.CreateProcess);
    let load_dll: PDBGKM_LOAD_DLL = core::ptr::addr_of_mut!(api_message.__bindgen_anon_1.LoadDll);
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let teb: PTEB;

    process_flags = PspSetProcessFlag(
        process,
        PSF_CREATE_REPORTED_BIT | PSF_IMAGE_NOTIFY_DONE_BIT,
    );

    if (process_flags & PSF_IMAGE_NOTIFY_DONE_BIT) == 0 && PsImageNotifyEnabled != 0 {
        image_info.__bindgen_anon_1.Properties = 0;
        image_info.__bindgen_anon_1.__bindgen_anon_1.set_ImageAddressingMode(IMAGE_ADDRESSING_MODE_32BIT);
        image_info.ImageBase = (*process_ptr).SectionBaseAddress;
        image_info.ImageSize = 0;
        image_info.ImageSelector = 0;
        image_info.ImageSectionNumber = 0;

        nt_header = RtlImageNtHeader((*process_ptr).SectionBaseAddress);
        if !nt_header.is_null() {
            image_info.ImageSize = (*nt_header).OptionalHeader.SizeOfImage as _;
        }

        status = MmGetFileNameForSection((*process_ptr).SectionObject, &mut module_name);
        if status >= 0 {
            PspRunLoadImageNotifyRoutines(
                core::ptr::addr_of_mut!((*module_name).Name),
                (*process_ptr).UniqueProcessId,
                &mut image_info,
            );
            ExFreePool(module_name as PVOID);
        } else {
            PspRunLoadImageNotifyRoutines(
                core::ptr::null_mut(),
                (*process_ptr).UniqueProcessId,
                &mut image_info,
            );
        }

        image_info.__bindgen_anon_1.Properties = 0;
        image_info.__bindgen_anon_1.__bindgen_anon_1.set_ImageAddressingMode(IMAGE_ADDRESSING_MODE_32BIT);
        image_info.ImageBase = PspSystemDllBase;
        image_info.ImageSize = 0;
        image_info.ImageSelector = 0;
        image_info.ImageSectionNumber = 0;

        nt_header = RtlImageNtHeader(PspSystemDllBase);
        if !nt_header.is_null() {
            image_info.ImageSize = (*nt_header).OptionalHeader.SizeOfImage as _;
        }

        // L"\\SystemRoot\\System32\\ntdll.dll" — UTF-16LE
        static NTDLL_PATH: [u16; 31] = [
            0x5C, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x52, // \SystemR
            0x6F, 0x6F, 0x74, 0x5C, 0x53, 0x79, 0x73, 0x74, // oot\Syst
            0x65, 0x6D, 0x33, 0x32, 0x5C, 0x6E, 0x74, 0x64, // em32\ntd
            0x6C, 0x6C, 0x2E, 0x64, 0x6C, 0x6C, 0x00,        // ll.dll\0
        ];
        RtlInitUnicodeString(
            &mut ntdll_name,
            NTDLL_PATH.as_ptr() as *mut u16,
        );
        PspRunLoadImageNotifyRoutines(
            &mut ntdll_name,
            (*process_ptr).UniqueProcessId,
            &mut image_info,
        );
    }

    debug_port = (*process_ptr).DebugPort;
    if debug_port.is_null() {
        return;
    }

    if (process_flags & PSF_CREATE_REPORTED_BIT) == 0 {
        (*create_process).InitialThread.SubSystemKey = 0;
        (*create_process).InitialThread.StartAddress = core::ptr::null_mut();

        (*create_process).SubSystemKey = 0;
        (*create_process).FileHandle = DbgkpSectionToFileHandle((*process_ptr).SectionObject);
        (*create_process).BaseOfImage = (*process_ptr).SectionBaseAddress;
        (*create_process).DebugInfoFileOffset = 0;
        (*create_process).DebugInfoSize = 0;

        nt_header = RtlImageNtHeader((*process_ptr).SectionBaseAddress);
        if !nt_header.is_null() {
            (*create_process).InitialThread.StartAddress = ((*nt_header).OptionalHeader.ImageBase as ULONG_PTR
                + (*nt_header).OptionalHeader.AddressOfEntryPoint as ULONG_PTR) as PVOID;
            (*create_process).DebugInfoFileOffset = (*nt_header).FileHeader.PointerToSymbolTable;
            (*create_process).DebugInfoSize = (*nt_header).FileHeader.NumberOfSymbols;
        }

        api_message.h.u1.Length = ((core::mem::size_of::<DBGKM_MSG>() << 16)
            | (8 + core::mem::size_of::<DBGKM_CREATE_PROCESS>())) as u32;
        api_message.h.u2.ZeroInit = 0;
        api_message.h.u2.s2.Type = LPC_DEBUG_EVENT;
        api_message.ApiNumber = DbgKmCreateProcessApi;

        DbgkpSendApiMessage(&mut api_message, 0u8);

        ObCloseHandle((*create_process).FileHandle, 0i8);

        (*load_dll).BaseOfDll = PspSystemDllBase;
        (*load_dll).DebugInfoFileOffset = 0;
        (*load_dll).DebugInfoSize = 0;
        (*load_dll).NamePointer = core::ptr::null_mut();

        nt_header = RtlImageNtHeader(PspSystemDllBase);
        if !nt_header.is_null() {
            (*load_dll).DebugInfoFileOffset = (*nt_header).FileHeader.PointerToSymbolTable;
            (*load_dll).DebugInfoSize = (*nt_header).FileHeader.NumberOfSymbols;
        }

        teb = (*(Thread as *mut _ETHREAD)).Tcb.Teb;
        if !teb.is_null() {
            // L"ntdll.dll" — UTF-16LE (10 u16s including null)
            static NTDLL_NAME: [u16; 10] = [
                0x6E, 0x74, 0x64, 0x6C, 0x6C, 0x2E, 0x64, 0x6C, 0x6C, 0x00,
            ];
            let src = NTDLL_NAME.as_ptr();
            let dst = (*teb).StaticUnicodeBuffer.as_mut_ptr();
            let max_len = core::mem::size_of_val(&(*teb).StaticUnicodeBuffer) / core::mem::size_of::<u16>();
            let src_len = 10;
            let copy_len = if src_len < max_len { src_len } else { max_len };
            core::ptr::copy_nonoverlapping(src, dst, copy_len);
            
            (*teb).NtTib.ArbitraryUserPointer = (*teb).StaticUnicodeBuffer.as_mut_ptr() as PVOID;
            (*load_dll).NamePointer = core::ptr::addr_of_mut!((*teb).NtTib.ArbitraryUserPointer) as PVOID;
        }

        InitializeObjectAttributes(
            &mut object_attributes,
            core::ptr::addr_of!(PsNtDllPathName) as *mut _UNICODE_STRING,
            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE | OBJ_FORCE_ACCESS_CHECK,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        
        status = ZwOpenFile(
            core::ptr::addr_of_mut!((*load_dll).FileHandle),
            GENERIC_READ | SYNCHRONIZE,
            &mut object_attributes,
            &mut io_status_block,
            FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_SYNCHRONOUS_IO_NONALERT,
        );
        
        if status >= 0 {
            api_message.h.u1.Length = ((core::mem::size_of::<DBGKM_MSG>() << 16)
                | (8 + core::mem::size_of::<DBGKM_LOAD_DLL>())) as u32;
            api_message.h.u2.ZeroInit = 0;
            api_message.h.u2.s2.Type = LPC_DEBUG_EVENT;
            api_message.ApiNumber = DbgKmLoadDllApi;

            DbgkpSendApiMessage(&mut api_message, 1u8);

            ObCloseHandle((*load_dll).FileHandle, 0i8);
        }
    } else {
        (*create_thread).SubSystemKey = 0;
        (*create_thread).StartAddress = StartAddress;

        api_message.h.u1.Length = ((core::mem::size_of::<DBGKM_MSG>() << 16)
            | (8 + core::mem::size_of::<DBGKM_CREATE_THREAD>())) as u32;
        api_message.h.u2.ZeroInit = 0;
        api_message.h.u2.s2.Type = LPC_DEBUG_EVENT;
        api_message.ApiNumber = DbgKmCreateThreadApi;

        DbgkpSendApiMessage(&mut api_message, 1u8);
    }
}

#[no_mangle]
pub unsafe extern "system" fn DbgkExitProcess(ExitStatus: NTSTATUS) {
    let mut api_message: DBGKM_MSG = core::mem::zeroed();
    let exit_process: PDBGKM_EXIT_PROCESS = core::ptr::addr_of_mut!(api_message.__bindgen_anon_1.ExitProcess);
    let process = PsGetCurrentProcess() as *mut _EPROCESS;
    let thread = PsGetCurrentThread() as *mut _ETHREAD;

    if (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.HideFromDebugger() != 0
        || (*process).DebugPort.is_null()
        || (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.DeadThread() != 0
    {
        return;
    }

    (*exit_process).ExitStatus = ExitStatus;

    api_message.h.u1.Length = ((core::mem::size_of::<DBGKM_MSG>() << 16)
        | (8 + core::mem::size_of::<DBGKM_EXIT_PROCESS>())) as u32;
    api_message.h.u2.ZeroInit = 0;
    api_message.h.u2.s2.Type = LPC_DEBUG_EVENT;
    api_message.ApiNumber = DbgKmExitProcessApi;

    KeQuerySystemTime(core::ptr::addr_of_mut!((*process).ExitTime) as PVOID);

    DbgkpSendApiMessage(&mut api_message, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn DbgkExitThread(ExitStatus: NTSTATUS) {
    let mut api_message: DBGKM_MSG = core::mem::zeroed();
    let exit_thread: PDBGKM_EXIT_THREAD = core::ptr::addr_of_mut!(api_message.__bindgen_anon_1.ExitThread);
    let process = PsGetCurrentProcess() as *mut _EPROCESS;
    let thread = PsGetCurrentThread() as *mut _ETHREAD;
    let suspended: BOOLEAN;

    if (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.HideFromDebugger() != 0
        || (*process).DebugPort.is_null()
        || (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.DeadThread() != 0
    {
        return;
    }

    (*exit_thread).ExitStatus = ExitStatus;

    api_message.h.u1.Length = ((core::mem::size_of::<DBGKM_MSG>() << 16)
        | (8 + core::mem::size_of::<DBGKM_EXIT_THREAD>())) as u32;
    api_message.h.u2.ZeroInit = 0;
    api_message.h.u2.s2.Type = LPC_DEBUG_EVENT;
    api_message.ApiNumber = DbgKmExitThreadApi;

    suspended = DbgkpSuspendProcess();

    DbgkpSendApiMessage(&mut api_message, 0u8);

    if suspended != 0 {
        DbgkpResumeProcess();
    }
}

#[no_mangle]
pub unsafe extern "system" fn DbgkMapViewOfSection(
    Section: PVOID,
    BaseAddress: PVOID,
    SectionOffset: ULONG,
    ViewSize: ULONG_PTR,
) {
    let Section = opaque_ptr(Section);
    let BaseAddress = opaque_ptr(BaseAddress);
    let mut api_message: DBGKM_MSG = core::mem::zeroed();
    let load_dll: PDBGKM_LOAD_DLL = core::ptr::addr_of_mut!(api_message.__bindgen_anon_1.LoadDll);
    let process = PsGetCurrentProcess() as *mut _EPROCESS;
    let thread = PsGetCurrentThread() as *mut _ETHREAD;
    let nt_header: PIMAGE_NT_HEADERS;

    if ExGetPreviousMode() == 0i8
        || (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.HideFromDebugger() != 0
        || (*process).DebugPort.is_null()
    {
        return;
    }

    (*load_dll).FileHandle = DbgkpSectionToFileHandle(Section);
    (*load_dll).BaseOfDll = BaseAddress;
    (*load_dll).DebugInfoFileOffset = 0;
    (*load_dll).DebugInfoSize = 0;
    (*load_dll).NamePointer = core::ptr::addr_of_mut!((*NtCurrentTeb()).NtTib.ArbitraryUserPointer) as PVOID;

    nt_header = RtlImageNtHeader(BaseAddress);
    if !nt_header.is_null() {
        (*load_dll).DebugInfoFileOffset = (*nt_header).FileHeader.PointerToSymbolTable;
        (*load_dll).DebugInfoSize = (*nt_header).FileHeader.NumberOfSymbols;
    }

    api_message.h.u1.Length = ((core::mem::size_of::<DBGKM_MSG>() << 16)
        | (8 + core::mem::size_of::<DBGKM_LOAD_DLL>())) as u32;
    api_message.h.u2.ZeroInit = 0;
    api_message.h.u2.s2.Type = LPC_DEBUG_EVENT;
    api_message.ApiNumber = DbgKmLoadDllApi;

    DbgkpSendApiMessage(&mut api_message, 1u8);

    ObCloseHandle((*load_dll).FileHandle, 0i8);
}

#[no_mangle]
pub unsafe extern "system" fn DbgkUnMapViewOfSection(BaseAddress: PVOID) {
    let BaseAddress = opaque_ptr(BaseAddress);
    let mut api_message: DBGKM_MSG = core::mem::zeroed();
    let unload_dll: PDBGKM_UNLOAD_DLL = core::ptr::addr_of_mut!(api_message.__bindgen_anon_1.UnloadDll);
    let process = PsGetCurrentProcess() as *mut _EPROCESS;
    let thread = PsGetCurrentThread() as *mut _ETHREAD;

    if ExGetPreviousMode() == 0i8
        || (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.HideFromDebugger() != 0
        || (*process).DebugPort.is_null()
    {
        return;
    }

    (*unload_dll).BaseAddress = BaseAddress;

    api_message.h.u1.Length = ((core::mem::size_of::<DBGKM_MSG>() << 16)
        | (8 + core::mem::size_of::<DBGKM_UNLOAD_DLL>())) as u32;
    api_message.h.u2.ZeroInit = 0;
    api_message.h.u2.s2.Type = LPC_DEBUG_EVENT;
    api_message.ApiNumber = DbgKmUnloadDllApi;

    DbgkpSendApiMessage(&mut api_message, 1u8);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/dbgk/dbgkutil.c
//  * PURPOSE:         User-Mode Debugging Support, Internal Debug Functions.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// HANDLE
// NTAPI
// DbgkpSectionToFileHandle(IN PVOID Section)
// {
//     NTSTATUS Status;
//     POBJECT_NAME_INFORMATION FileName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     HANDLE Handle;
//     PAGED_CODE();
//
//     /* Get the filename of the section */
//     Status = MmGetFileNameForSection(Section, &FileName);
//     if (!NT_SUCCESS(Status)) return NULL;
//
//     /* Initialize object attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FileName->Name,
//                                OBJ_CASE_INSENSITIVE |
//                                OBJ_FORCE_ACCESS_CHECK |
//                                OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Open the file */
//     Status = ZwOpenFile(&Handle,
//                         GENERIC_READ | SYNCHRONIZE,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_NONALERT);
//
//     /* Free the name and return the handle if we succeeded */
//     ExFreePool(FileName);
//     if (!NT_SUCCESS(Status)) return NULL;
//     return Handle;
// }
//
// BOOLEAN
// NTAPI
// DbgkpSuspendProcess(VOID)
// {
//     PAGED_CODE();
//
//     /* Make sure this isn't a deleted process */
//     if (!PsGetCurrentProcess()->ProcessDelete)
//     {
//         /* Freeze all the threads */
//         KeFreezeAllThreads();
//         return TRUE;
//     }
//     else
//     {
//         /* No suspend was done */
//         return FALSE;
//     }
// }
//
// VOID
// NTAPI
// DbgkpResumeProcess(VOID)
// {
//     PAGED_CODE();
//
//     /* Thaw all the threads */
//     KeThawAllThreads();
// }
//
// VOID
// NTAPI
// DbgkCreateThread(IN PETHREAD Thread,
//                  IN PVOID StartAddress)
// {
//     PEPROCESS Process = PsGetCurrentProcess();
//     ULONG ProcessFlags;
//     IMAGE_INFO ImageInfo;
//     PIMAGE_NT_HEADERS NtHeader;
//     POBJECT_NAME_INFORMATION ModuleName;
//     UNICODE_STRING NtDllName;
//     NTSTATUS Status;
//     PVOID DebugPort;
//     DBGKM_MSG ApiMessage;
//     PDBGKM_CREATE_THREAD CreateThread = &ApiMessage.CreateThread;
//     PDBGKM_CREATE_PROCESS CreateProcess = &ApiMessage.CreateProcess;
//     PDBGKM_LOAD_DLL LoadDll = &ApiMessage.LoadDll;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PTEB Teb;
//     PAGED_CODE();
//
//     /* Sanity check */
//     ASSERT(Thread == PsGetCurrentThread());
//
//     /* Try ORing in the create reported and image notify flags */
//     ProcessFlags = PspSetProcessFlag(Process,
//                                      PSF_CREATE_REPORTED_BIT |
//                                      PSF_IMAGE_NOTIFY_DONE_BIT);
//
//     /* Check if we were the first to set them or if another thread raced us */
//     if (!(ProcessFlags & PSF_IMAGE_NOTIFY_DONE_BIT) && (PsImageNotifyEnabled))
//     {
//         /* It hasn't.. set up the image info for the process */
//         ImageInfo.Properties = 0;
//         ImageInfo.ImageAddressingMode = IMAGE_ADDRESSING_MODE_32BIT;
//         ImageInfo.ImageBase = Process->SectionBaseAddress;
//         ImageInfo.ImageSize = 0;
//         ImageInfo.ImageSelector = 0;
//         ImageInfo.ImageSectionNumber = 0;
//
//         /* Get the NT Headers */
//         NtHeader = RtlImageNtHeader(Process->SectionBaseAddress);
//         if (NtHeader)
//         {
//             /* Set image size */
//             ImageInfo.ImageSize = NtHeader->OptionalHeader.SizeOfImage;
//         }
//
//         /* Get the image name */
//         Status = MmGetFileNameForSection(Process->SectionObject, &ModuleName);
//         if (NT_SUCCESS(Status))
//         {
//             /* Call the notify routines and free the name */
//             PspRunLoadImageNotifyRoutines(&ModuleName->Name,
//                                           Process->UniqueProcessId,
//                                           &ImageInfo);
//             ExFreePool(ModuleName);
//         }
//         else
//         {
//             /* Call the notify routines */
//             PspRunLoadImageNotifyRoutines(NULL,
//                                           Process->UniqueProcessId,
//                                           &ImageInfo);
//         }
//
//         /* Setup the info for ntdll.dll */
//         ImageInfo.Properties = 0;
//         ImageInfo.ImageAddressingMode = IMAGE_ADDRESSING_MODE_32BIT;
//         ImageInfo.ImageBase = PspSystemDllBase;
//         ImageInfo.ImageSize = 0;
//         ImageInfo.ImageSelector = 0;
//         ImageInfo.ImageSectionNumber = 0;
//
//         /* Get the NT Headers */
//         NtHeader = RtlImageNtHeader(PspSystemDllBase);
//         if (NtHeader)
//         {
//             /* Set image size */
//             ImageInfo.ImageSize = NtHeader->OptionalHeader.SizeOfImage;
//         }
//
//         /* Call the notify routines */
//         RtlInitUnicodeString(&NtDllName,
//                              L"\\SystemRoot\\System32\\ntdll.dll");
//         PspRunLoadImageNotifyRoutines(&NtDllName,
//                                       Process->UniqueProcessId,
//                                       &ImageInfo);
//     }
//
//     /* Fail if we have no port */
//     DebugPort = Process->DebugPort;
//     if (!DebugPort) return;
//
//     /* Check if create was not already reported */
//     if (!(ProcessFlags & PSF_CREATE_REPORTED_BIT))
//     {
//         /* Setup the information structure for the new thread */
//         CreateProcess->InitialThread.SubSystemKey = 0;
//         CreateProcess->InitialThread.StartAddress = NULL;
//
//         /* And for the new process */
//         CreateProcess->SubSystemKey = 0;
//         CreateProcess->FileHandle = DbgkpSectionToFileHandle(Process->
//                                                              SectionObject);
//         CreateProcess->BaseOfImage = Process->SectionBaseAddress;
//         CreateProcess->DebugInfoFileOffset = 0;
//         CreateProcess->DebugInfoSize = 0;
//
//         /* Get the NT Header */
//         NtHeader = RtlImageNtHeader(Process->SectionBaseAddress);
//         if (NtHeader)
//         {
//             /* Fill out data from the header */
//             CreateProcess->InitialThread.StartAddress =
//                 (PVOID)((ULONG_PTR)NtHeader->OptionalHeader.ImageBase +
//                         NtHeader->OptionalHeader.AddressOfEntryPoint);
//             CreateProcess->DebugInfoFileOffset = NtHeader->FileHeader.
//                                                  PointerToSymbolTable;
//             CreateProcess->DebugInfoSize = NtHeader->FileHeader.
//                                            NumberOfSymbols;
//         }
//
//         /* Setup the API Message */
//         ApiMessage.h.u1.Length = sizeof(DBGKM_MSG) << 16 |
//                                  (8 + sizeof(DBGKM_CREATE_PROCESS));
//         ApiMessage.h.u2.ZeroInit = 0;
//         ApiMessage.h.u2.s2.Type = LPC_DEBUG_EVENT;
//         ApiMessage.ApiNumber = DbgKmCreateProcessApi;
//
//         /* Send the message */
//         DbgkpSendApiMessage(&ApiMessage, FALSE);
//
//         /* Close the handle */
//         ObCloseHandle(CreateProcess->FileHandle, KernelMode);
//
//         /* Setup the parameters */
//         LoadDll->BaseOfDll = PspSystemDllBase;
//         LoadDll->DebugInfoFileOffset = 0;
//         LoadDll->DebugInfoSize = 0;
//         LoadDll->NamePointer = NULL;
//
//         /* Get the NT Headers */
//         NtHeader = RtlImageNtHeader(PspSystemDllBase);
//         if (NtHeader)
//         {
//             /* Fill out debug information */
//             LoadDll->DebugInfoFileOffset = NtHeader->
//                                            FileHeader.PointerToSymbolTable;
//             LoadDll->DebugInfoSize = NtHeader->FileHeader.NumberOfSymbols;
//         }
//
//         /* Get the TEB */
//         Teb = Thread->Tcb.Teb;
//         if (Teb)
//         {
//             /* Copy the system library name and link to it */
//             wcsncpy(Teb->StaticUnicodeBuffer,
//                     L"ntdll.dll",
//                     sizeof(Teb->StaticUnicodeBuffer) / sizeof(WCHAR));
//             Teb->NtTib.ArbitraryUserPointer = Teb->StaticUnicodeBuffer;
//
//             /* Return it in the debug event as well */
//             LoadDll->NamePointer = &Teb->NtTib.ArbitraryUserPointer;
//         }
//
//         /* Get a handle */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &PsNtDllPathName,
//                                    OBJ_CASE_INSENSITIVE |
//                                    OBJ_KERNEL_HANDLE |
//                                    OBJ_FORCE_ACCESS_CHECK,
//                                    NULL,
//                                    NULL);
//         Status = ZwOpenFile(&LoadDll->FileHandle,
//                             GENERIC_READ | SYNCHRONIZE,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_DELETE |
//                             FILE_SHARE_READ |
//                             FILE_SHARE_WRITE,
//                             FILE_SYNCHRONOUS_IO_NONALERT);
//         if (NT_SUCCESS(Status))
//         {
//             /* Setup the API Message */
//             ApiMessage.h.u1.Length = sizeof(DBGKM_MSG) << 16 |
//                                      (8 + sizeof(DBGKM_LOAD_DLL));
//             ApiMessage.h.u2.ZeroInit = 0;
//             ApiMessage.h.u2.s2.Type = LPC_DEBUG_EVENT;
//             ApiMessage.ApiNumber = DbgKmLoadDllApi;
//
//             /* Send the message */
//             DbgkpSendApiMessage(&ApiMessage, TRUE);
//
//             /* Close the handle */
//             ObCloseHandle(LoadDll->FileHandle, KernelMode);
//         }
//     }
//     else
//     {
//         /* Otherwise, do it just for the thread */
//         CreateThread->SubSystemKey = 0;
//         CreateThread->StartAddress = StartAddress;
//
//         /* Setup the API Message */
//         ApiMessage.h.u1.Length = sizeof(DBGKM_MSG) << 16 |
//                                  (8 + sizeof(DBGKM_CREATE_THREAD));
//         ApiMessage.h.u2.ZeroInit = 0;
//         ApiMessage.h.u2.s2.Type = LPC_DEBUG_EVENT;
//         ApiMessage.ApiNumber = DbgKmCreateThreadApi;
//
//         /* Send the message */
//         DbgkpSendApiMessage(&ApiMessage, TRUE);
//     }
// }
//
// VOID
// NTAPI
// DbgkExitProcess(IN NTSTATUS ExitStatus)
// {
//     DBGKM_MSG ApiMessage;
//     PDBGKM_EXIT_PROCESS ExitProcess = &ApiMessage.ExitProcess;
//     PEPROCESS Process = PsGetCurrentProcess();
//     PETHREAD Thread = PsGetCurrentThread();
//     PAGED_CODE();
//
//     /* Check if this thread is hidden, doesn't have a debug port, or died */
//     if ((Thread->HideFromDebugger) ||
//         !(Process->DebugPort) ||
//         (Thread->DeadThread))
//     {
//         /* Don't notify the debugger */
//         return;
//     }
//
//     /* Set the exit status */
//     ExitProcess->ExitStatus = ExitStatus;
//
//     /* Setup the API Message */
//     ApiMessage.h.u1.Length = sizeof(DBGKM_MSG) << 16 |
//                              (8 + sizeof(DBGKM_EXIT_PROCESS));
//     ApiMessage.h.u2.ZeroInit = 0;
//     ApiMessage.h.u2.s2.Type = LPC_DEBUG_EVENT;
//     ApiMessage.ApiNumber = DbgKmExitProcessApi;
//
//     /* Set the current exit time */
//     KeQuerySystemTime(&Process->ExitTime);
//
//     /* Send the message */
//     DbgkpSendApiMessage(&ApiMessage, FALSE);
// }
//
// VOID
// NTAPI
// DbgkExitThread(IN NTSTATUS ExitStatus)
// {
//     DBGKM_MSG ApiMessage;
//     PDBGKM_EXIT_THREAD ExitThread = &ApiMessage.ExitThread;
//     PEPROCESS Process = PsGetCurrentProcess();
//     PETHREAD Thread = PsGetCurrentThread();
//     BOOLEAN Suspended;
//     PAGED_CODE();
//
//     /* Check if this thread is hidden, doesn't have a debug port, or died */
//     if ((Thread->HideFromDebugger) ||
//         !(Process->DebugPort) ||
//         (Thread->DeadThread))
//     {
//         /* Don't notify the debugger */
//         return;
//     }
//
//     /* Set the exit status */
//     ExitThread->ExitStatus = ExitStatus;
//
//     /* Setup the API Message */
//     ApiMessage.h.u1.Length = sizeof(DBGKM_MSG) << 16 |
//                              (8 + sizeof(DBGKM_EXIT_THREAD));
//     ApiMessage.h.u2.ZeroInit = 0;
//     ApiMessage.h.u2.s2.Type = LPC_DEBUG_EVENT;
//     ApiMessage.ApiNumber = DbgKmExitThreadApi;
//
//     /* Suspend the process */
//     Suspended = DbgkpSuspendProcess();
//
//     /* Send the message */
//     DbgkpSendApiMessage(&ApiMessage, FALSE);
//
//     /* Resume the process if needed */
//     if (Suspended) DbgkpResumeProcess();
// }
//
// VOID
// NTAPI
// DbgkMapViewOfSection(IN PVOID Section,
//                      IN PVOID BaseAddress,
//                      IN ULONG SectionOffset,
//                      IN ULONG_PTR ViewSize)
// {
//     DBGKM_MSG ApiMessage;
//     PDBGKM_LOAD_DLL LoadDll = &ApiMessage.LoadDll;
//     PEPROCESS Process = PsGetCurrentProcess();
//     PETHREAD Thread = PsGetCurrentThread();
//     PIMAGE_NT_HEADERS NtHeader;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_PROCESS_DEBUG,
//               "Section: %p. Base: %p\n", Section, BaseAddress);
//
//     /* Check if this thread is kernel, hidden or doesn't have a debug port */
//     if ((ExGetPreviousMode() == KernelMode) ||
//         (Thread->HideFromDebugger) ||
//         !(Process->DebugPort))
//     {
//         /* Don't notify the debugger */
//         return;
//     }
//
//     /* Setup the parameters */
//     LoadDll->FileHandle = DbgkpSectionToFileHandle(Section);
//     LoadDll->BaseOfDll = BaseAddress;
//     LoadDll->DebugInfoFileOffset = 0;
//     LoadDll->DebugInfoSize = 0;
//     LoadDll->NamePointer = &NtCurrentTeb()->NtTib.ArbitraryUserPointer;
//
//     /* Get the NT Headers */
//     NtHeader = RtlImageNtHeader(BaseAddress);
//     if (NtHeader)
//     {
//         /* Fill out debug information */
//         LoadDll->DebugInfoFileOffset = NtHeader->FileHeader.
//                                        PointerToSymbolTable;
//         LoadDll->DebugInfoSize = NtHeader->FileHeader.NumberOfSymbols;
//     }
//
//     /* Setup the API Message */
//     ApiMessage.h.u1.Length = sizeof(DBGKM_MSG) << 16 |
//                              (8 + sizeof(DBGKM_LOAD_DLL));
//     ApiMessage.h.u2.ZeroInit = 0;
//     ApiMessage.h.u2.s2.Type = LPC_DEBUG_EVENT;
//     ApiMessage.ApiNumber = DbgKmLoadDllApi;
//
//     /* Send the message */
//     DbgkpSendApiMessage(&ApiMessage, TRUE);
//
//     /* Close the handle */
//     ObCloseHandle(LoadDll->FileHandle, KernelMode);
// }
//
// VOID
// NTAPI
// DbgkUnMapViewOfSection(IN PVOID BaseAddress)
// {
//     DBGKM_MSG ApiMessage;
//     PDBGKM_UNLOAD_DLL UnloadDll = &ApiMessage.UnloadDll;
//     PEPROCESS Process = PsGetCurrentProcess();
//     PETHREAD Thread = PsGetCurrentThread();
//     PAGED_CODE();
//
//     /* Check if this thread is kernel, hidden or doesn't have a debug port */
//     if ((ExGetPreviousMode() == KernelMode) ||
//         (Thread->HideFromDebugger) ||
//         !(Process->DebugPort))
//     {
//         /* Don't notify the debugger */
//         return;
//     }
//
//     /* Set the DLL Base */
//     UnloadDll->BaseAddress = BaseAddress;
//
//     /* Setup the API Message */
//     ApiMessage.h.u1.Length = sizeof(DBGKM_MSG) << 16 |
//                              (8 + sizeof(DBGKM_UNLOAD_DLL));
//     ApiMessage.h.u2.ZeroInit = 0;
//     ApiMessage.h.u2.s2.Type = LPC_DEBUG_EVENT;
//     ApiMessage.ApiNumber = DbgKmUnloadDllApi;
//
//     /* Send the message */
//     DbgkpSendApiMessage(&ApiMessage, TRUE);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
