use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{MmCreateSection, MmMapViewInSessionSpace, MmMapViewInSystemSpace, MmMapViewOfSection, MmUnmapViewInSessionSpace, MmUnmapViewInSystemSpace, MmUnmapViewOfSection, ObReferenceObjectByHandle, RtlInitUnicodeString, ZwClose, ZwCreateFile, ZwMapViewOfSection, ZwQueryInformationFile, ZwUnmapViewOfSection};
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::macros::NtCurrentProcess;
use ntkrnl_sys::ps::PsGetCurrentProcess;

// ---- file-scope preamble (translated) ----
pub static mut ghSystem32Directory: HANDLE = core::ptr::null_mut();
pub static mut ghRootDirectory: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub unsafe extern "system" fn EngMapSectionView(
    hSection: HANDLE,
    cjSize: SIZE_T,
    cjOffset: ULONG,
    phSecure: PHANDLE,
) -> PVOID {
    let mut liSectionOffset: ntkrnl_sys::ddk::_LARGE_INTEGER = core::mem::zeroed();
    let mut pvBaseAddress: PVOID = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut cjSize: ntkrnl_sys::ddk::SIZE_T = cjSize as _;

    if cjSize > ULONG_MAX as ntkrnl_sys::ddk::SIZE_T {
        return core::ptr::null_mut();
    }

    liSectionOffset.QuadPart = (cjOffset & !(MM_ALLOCATION_GRANULARITY - 1)) as i64;
    cjSize += (cjOffset & (MM_ALLOCATION_GRANULARITY - 1)) as ntkrnl_sys::ddk::SIZE_T;

    Status = ZwMapViewOfSection(
        hSection,
        NtCurrentProcess(),
        &mut pvBaseAddress,
        0,
        cjSize,
        &mut liSectionOffset,
        &mut cjSize,
        ViewShare as ntkrnl_sys::ddk::SECTION_INHERIT,
        0,
        PAGE_READWRITE,
    );
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    *phSecure = EngSecureMem(pvBaseAddress, cjSize as ULONG);
    if (*phSecure).is_null() {
        ZwUnmapViewOfSection(NtCurrentProcess(), pvBaseAddress);
        return core::ptr::null_mut();
    }

    (pvBaseAddress as PUCHAR).add((cjOffset & (MM_ALLOCATION_GRANULARITY - 1)) as usize) as PVOID
}
#[no_mangle]
pub unsafe extern "system" fn EngUnmapSectionView(
    pvBits: PVOID,
    cjOffset: ULONG,
    hSecure: HANDLE,
) {
    let mut Status: NTSTATUS;

    EngUnsecureMem(hSecure);

    let pvBits = (pvBits as PUCHAR).offset(-((cjOffset & (MM_ALLOCATION_GRANULARITY - 1)) as isize)) as PVOID;

    Status = MmUnmapViewOfSection(PsGetCurrentProcess() as *mut _, pvBits);
}

pub unsafe extern "system" fn EngCreateSection(
    fl: u32,
    cjSize: usize,
    ulTag: u32,
) -> *mut core::ffi::c_void {
    let mut Status: i32;
    let mut pSection: PENGSECTION;
    let mut pvSectionObject: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut liSize: ntkrnl_sys::ddk::_LARGE_INTEGER;

    pSection = EngAllocMem(0, core::mem::size_of::<ENGSECTION>() as ULONG, 0x73747355) as PENGSECTION;
    if pSection.is_null() {
        return core::ptr::null_mut();
    }

    liSize = core::mem::zeroed();
    liSize.QuadPart = cjSize as i64;
    Status = MmCreateSection(
        &mut pvSectionObject,
        0x000F001F,
        core::ptr::null_mut(),
        &mut liSize,
        PAGE_READWRITE,
        SEC_COMMIT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        EngFreeMem(pSection as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*pSection).ulTag = ulTag;
    (*pSection).pvSectionObject = pvSectionObject;
    (*pSection).pvMappedBase = core::ptr::null_mut();
    (*pSection).cjViewSize = cjSize as SIZE_T;

    pSection as *mut core::ffi::c_void
}

unsafe fn EngCreateSectionHack(
    fl: u32,
    cjSize: usize,
    ulTag: u32,
) -> *mut core::ffi::c_void {
    let mut Status: i32;
    let mut pSection: PENGSECTION;
    let mut pvSectionObject: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut liSize: ntkrnl_sys::ddk::_LARGE_INTEGER;

    pSection = EngAllocMem(0, core::mem::size_of::<ENGSECTION>() as u32, 0x73747355) as PENGSECTION;
    if pSection.is_null() {
        return core::ptr::null_mut();
    }

    liSize = core::mem::zeroed();
    liSize.QuadPart = cjSize as i64;
    Status = MmCreateSection(
        &mut pvSectionObject,
        0x000F001F,
        core::ptr::null_mut(),
        &mut liSize,
        PAGE_READWRITE,
        SEC_COMMIT | 1,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        EngFreeMem(pSection as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*pSection).ulTag = ulTag;
    (*pSection).pvSectionObject = pvSectionObject;
    (*pSection).pvMappedBase = core::ptr::null_mut();
    (*pSection).cjViewSize = cjSize as SIZE_T;

    pSection as *mut core::ffi::c_void
}
#[no_mangle]
pub unsafe extern "system" fn EngMapSection(
    pvSection: PVOID,
    bMap: BOOL,
    hProcess: HANDLE,
    pvBaseAddress: *mut PVOID,
) -> BOOL {
    let mut Status: NTSTATUS;
    let pSection = pvSection as PENGSECTION;
    let mut pepProcess: ntkrnl_sys::ddk::PEPROCESS = core::ptr::null_mut();
    // Cast view size through ntkrnl_sys's SIZE_T (layout-identical).
    let mut viewSize: ntkrnl_sys::ddk::SIZE_T = (*pSection).cjViewSize as _;

    Status = ObReferenceObjectByHandle(
        hProcess,
        PROCESS_VM_OPERATION,
        core::ptr::null_mut(),
        0i32 as i8,
        &mut pepProcess as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        ObDereferenceObject(pepProcess as PVOID);
        *pvBaseAddress = (*pSection).pvMappedBase;
        return 0i32;
    }

    if bMap != 0 {
        Status = MmMapViewOfSection(
            (*pSection).pvSectionObject,
            pepProcess,
            &mut (*pSection).pvMappedBase,
            0,
            viewSize,
            core::ptr::null_mut(),
            &mut viewSize,
            ViewUnmap as ntkrnl_sys::ddk::SECTION_INHERIT,
            0,
            PAGE_READWRITE,
        );
        (*pSection).cjViewSize = viewSize as SIZE_T;
    } else {
        Status = MmUnmapViewOfSection(pepProcess, (*pSection).pvMappedBase);
        if NT_SUCCESS(Status) {
            (*pSection).pvMappedBase = core::ptr::null_mut();
        }
    }

    ObDereferenceObject(pepProcess as PVOID);

    *pvBaseAddress = (*pSection).pvMappedBase;
    if NT_SUCCESS(Status) { 1i32 } else { 0i32 }
}
#[no_mangle]
pub unsafe extern "system" fn EngFreeSectionMem(
    pvSection: *mut core::ffi::c_void,
    pvMappedBase: *mut core::ffi::c_void,
) -> u32 {
    let pSection = pvSection as PENGSECTION;
    let mut bResult: u32 = 1;

    if !pvMappedBase.is_null() {
        let Status = MmUnmapViewInSessionSpace(pvMappedBase);
        if !NT_SUCCESS(Status) {
            bResult = 0;
        }
    }

    if !pSection.is_null() {
        ObDereferenceObject((*pSection).pvSectionObject as *mut core::ffi::c_void);
        EngFreeMem(pSection as *mut core::ffi::c_void);
    }

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn EngAllocSectionMem(
    ppvSection: *mut *mut core::ffi::c_void,
    fl: u32,
    cjSize: usize,
    ulTag: u32,
) -> *mut core::ffi::c_void {
    let mut Status: i32;
    let mut pSection: PENGSECTION;

    if cjSize == 0 {
        return core::ptr::null_mut();
    }

    pSection = EngCreateSectionHack(fl, cjSize, ulTag) as PENGSECTION;
    if pSection.is_null() {
        *ppvSection = core::ptr::null_mut();
        return core::ptr::null_mut();
    }

    Status = MmMapViewInSessionSpace(
        (*pSection).pvSectionObject,
        core::ptr::addr_of_mut!((*pSection).pvMappedBase),
        core::ptr::addr_of_mut!((*pSection).cjViewSize),
    );
    if !NT_SUCCESS(Status) {
        *ppvSection = core::ptr::null_mut();
        EngFreeSectionMem(pSection as *mut core::ffi::c_void, core::ptr::null_mut());
        return core::ptr::null_mut();
    }

    if (fl & FL_ZERO_MEMORY) != 0 {
        core::ptr::write_bytes((*pSection).pvMappedBase as *mut u8, 0, cjSize);
    }

    *ppvSection = pSection as *mut core::ffi::c_void;
    (*pSection).pvMappedBase
}
#[no_mangle]
pub unsafe extern "system" fn EngLoadModuleEx(
    pwsz: LPWSTR,
    cjSizeOfModule: u32,
    fl: FLONG,
) -> PFILEVIEW {
    let mut pFileView: PFILEVIEW = core::ptr::null_mut();
    let mut ObjectAttributes: ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let hRootDir: *mut core::ffi::c_void;
    let mut ustrFileName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: ntkrnl_sys::ddk::_IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileInformation: ntkrnl_sys::ddk::_FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut hFile: *mut core::ffi::c_void = core::ptr::null_mut();
    let Status: i32;
    let mut liSize: ntkrnl_sys::ddk::_LARGE_INTEGER = core::mem::zeroed();

    if (fl & FVF_FONTFILE) != 0 {
        pFileView = EngAllocMem(0, core::mem::size_of::<FONTFILEVIEW>() as u32, 0x76666647) as PFILEVIEW;
    } else {
        pFileView = EngAllocMem(0, core::mem::size_of::<FILEVIEW>() as u32, 0x6c496667) as PFILEVIEW;
    }

    if pFileView.is_null() {
        return core::ptr::null_mut();
    }

    if (fl & FVF_SYSTEMROOT) != 0 {
        hRootDir = ghSystem32Directory;
    } else {
        hRootDir = ghRootDirectory;
    }

    RtlInitUnicodeString(core::ptr::addr_of_mut!(ustrFileName), pwsz);

    ObjectAttributes.Length = core::mem::size_of::<ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = hRootDir;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(ustrFileName);
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let Status = ZwCreateFile(
        core::ptr::addr_of_mut!(hFile),
        FILE_READ_DATA,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        0,
        FILE_OPEN,
        FILE_NON_DIRECTORY_FILE,
        core::ptr::null_mut(),
        0,
    );

    if !NT_SUCCESS(Status) {
        EngFreeMem(pFileView as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    let Status = ZwQueryInformationFile(
        hFile,
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::addr_of_mut!(FileInformation) as *mut core::ffi::c_void,
        core::mem::size_of::<ntkrnl_sys::ddk::_FILE_BASIC_INFORMATION>() as u32,
        FileBasicInformation as ntkrnl_sys::ddk::FILE_INFORMATION_CLASS,
    );

    if NT_SUCCESS(Status) {
        // LastWriteTime: win32k_sys::LARGE_INTEGER and ntkrnl_sys::LARGE_INTEGER are
        // layout-identical bindgen unions; reinterpret bytes through transmute_copy.
        (*pFileView).LastWriteTime = core::mem::transmute_copy(&FileInformation.LastWriteTime);
    }

    liSize.QuadPart = cjSizeOfModule as i64;
    let Status = MmCreateSection(
        core::ptr::addr_of_mut!((*pFileView).pSection),
        0x000F0000 | 0x000F0001 | 0x0002 | 0x0004 | 0x0008 | 0x0010 | 0x0001,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(liSize),
        if (fl & FVF_READONLY) != 0 { PAGE_EXECUTE_READ } else { PAGE_EXECUTE_READWRITE },
        SEC_COMMIT,
        hFile,
        core::ptr::null_mut(),
    );

    ZwClose(hFile);

    if !NT_SUCCESS(Status) {
        EngFreeMem(pFileView as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*pFileView).pvKView = core::ptr::null_mut();
    (*pFileView).pvViewFD = core::ptr::null_mut();
    (*pFileView).cjView = 0;

    pFileView
}
#[no_mangle]
pub unsafe extern "system" fn EngLoadModule(
    pwsz: LPWSTR,
) -> *mut core::ffi::c_void {
    EngLoadModuleEx(pwsz, 0, FVF_READONLY | FVF_SYSTEMROOT) as *mut core::ffi::c_void
}
#[no_mangle]
pub unsafe extern "system" fn EngLoadModuleForWrite(
    pwsz: LPWSTR,
    cjSizeOfModule: u32,
) -> *mut core::ffi::c_void {
    EngLoadModuleEx(pwsz, cjSizeOfModule, FVF_SYSTEMROOT) as *mut core::ffi::c_void
}
#[no_mangle]
pub unsafe extern "system" fn EngMapModule(
    h: HANDLE,
    pulSize: PULONG,
) -> PVOID {
    let pFileView = h as PFILEVIEW;
    let Status: NTSTATUS;

    (*pFileView).cjView = 0;

    Status = MmMapViewInSystemSpace(
        (*pFileView).pSection,
        core::ptr::addr_of_mut!((*pFileView).pvKView),
        core::ptr::addr_of_mut!((*pFileView).cjView),
    );
    if !NT_SUCCESS(Status) {
        *pulSize = 0;
        return core::ptr::null_mut();
    }

    *pulSize = (*pFileView).cjView as ULONG;
    (*pFileView).pvKView
}
#[no_mangle]
pub unsafe extern "system" fn EngFreeModule(
    h: HANDLE,
) {
    let pFileView = h as PFILEVIEW;
    let Status: NTSTATUS;

    Status = MmUnmapViewInSystemSpace((*pFileView).pvKView);
    if !NT_SUCCESS(Status) {
        // DPRINT removed
    }

    ObDereferenceObject((*pFileView).pSection as PVOID);

    EngFreeMem(pFileView as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn EngMapFile(
    pwsz: LPWSTR,
    cjSize: u32,
    piFile: *mut usize,
) -> *mut core::ffi::c_void {
    let mut hModule: *mut core::ffi::c_void;
    let pvBase: *mut core::ffi::c_void;
    let mut cjSizeLocal = cjSize;

    hModule = EngLoadModuleEx(pwsz, 0, 0) as *mut core::ffi::c_void;
    if hModule.is_null() {
        *piFile = 0;
        return core::ptr::null_mut();
    }

    pvBase = EngMapModule(hModule, core::ptr::addr_of_mut!(cjSizeLocal));
    if pvBase.is_null() {
        EngFreeModule(hModule);
        hModule = core::ptr::null_mut();
    }

    *piFile = hModule as usize;
    return pvBase;
}
#[no_mangle]
pub unsafe extern "system" fn EngUnmapFile(
    iFile: ULONG_PTR,
) -> BOOL {
    let hModule = iFile as HANDLE;

    EngFreeModule(hModule);

    return 1i32;
}
#[no_mangle]
pub unsafe extern "system" fn EngUnmapFontFileFD(
    iFile: ULONG_PTR,
) {
}
#[no_mangle]
pub unsafe extern "system" fn EngMapFontFileFD(
    _iFile: ULONG_PTR,
    _ppjBuf: *mut PULONG,
    _pcjBuf: PULONG,
) -> BOOL {
    // @unimplemented stub
    0
}

#[no_mangle]
pub unsafe extern "system" fn EngMapFontFile(
    iFile: ULONG_PTR,
    ppjBuf: *mut PULONG,
    pcjBuf: PULONG,
) -> BOOL {
    return EngMapFontFileFD(iFile, ppjBuf, pcjBuf);
}
#[no_mangle]
pub unsafe extern "system" fn EngUnmapFontFile(
    iFile: ULONG_PTR,
) {
    EngUnmapFontFileFD(iFile);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           Functions for mapping files and sections
//  * FILE:              win32ss/gdi/eng/mapping.c
//  * PROGRAMER:         Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// HANDLE ghSystem32Directory;
// HANDLE ghRootDirectory;
//
// PVOID
// NTAPI
// EngMapSectionView(
//     _In_ HANDLE hSection,
//     _In_ SIZE_T cjSize,
//     _In_ ULONG cjOffset,
//     _Out_ PHANDLE phSecure)
// {
//     LARGE_INTEGER liSectionOffset;
//     PVOID pvBaseAddress;
//     NTSTATUS Status;
//
//     /* Check if the size is ok (for 64 bit) */
//     if (cjSize > ULONG_MAX)
//     {
//         DPRINT1("chSize out of range: 0x%Id\n", cjSize);
//         return NULL;
//     }
//
//     /* Align the offset at allocation granularity and compensate for the size */
//     liSectionOffset.QuadPart = cjOffset & ~(MM_ALLOCATION_GRANULARITY - 1);
//     cjSize += cjOffset & (MM_ALLOCATION_GRANULARITY - 1);
//
//     /* Map the section */
//     Status = ZwMapViewOfSection(hSection,
//                                 NtCurrentProcess(),
//                                 &pvBaseAddress,
//                                 0,
//                                 cjSize,
//                                 &liSectionOffset,
//                                 &cjSize,
//                                 ViewShare,
//                                 0,
//                                 PAGE_READWRITE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwMapViewOfSection failed (0x%lx)\n", Status);
//         return NULL;
//     }
//
//     /* Secure the section memory */
//     *phSecure = EngSecureMem(pvBaseAddress, (ULONG)cjSize);
//     if (!*phSecure)
//     {
//         ZwUnmapViewOfSection(NtCurrentProcess(), pvBaseAddress);
//         return NULL;
//     }
//
//     /* Return the address where the requested data starts */
//     return (PUCHAR)pvBaseAddress + (cjOffset & (MM_ALLOCATION_GRANULARITY - 1));
// }
//
// VOID
// NTAPI
// EngUnmapSectionView(
//     _In_ PVOID pvBits,
//     _In_ ULONG cjOffset,
//     _In_ HANDLE hSecure)
// {
//     NTSTATUS Status;
//
//     /* Unsecure the memory */
//     EngUnsecureMem(hSecure);
//
//     /* Calculate the real start of the section view */
//     pvBits = (PUCHAR)pvBits - (cjOffset & (MM_ALLOCATION_GRANULARITY - 1));
//
//     /* Unmap the section view */
//     Status = MmUnmapViewOfSection(PsGetCurrentProcess() as *mut _, pvBits);
//     ASSERT(NT_SUCCESS(Status));
// }
//
// PVOID
// NTAPI
// EngCreateSection(
//     _In_ ULONG fl,
//     _In_ SIZE_T cjSize,
//     _In_ ULONG ulTag)
// {
//     NTSTATUS Status;
//     PENGSECTION pSection;
//     PVOID pvSectionObject;
//     LARGE_INTEGER liSize;
//
//     /* Allocate a section object */
//     pSection = EngAllocMem(0, sizeof(ENGSECTION), 'stsU');
//     if (!pSection) return NULL;
//
//     liSize.QuadPart = cjSize;
//     Status = MmCreateSection(&pvSectionObject,
//                              SECTION_ALL_ACCESS,
//                              NULL,
//                              &liSize,
//                              PAGE_READWRITE,
//                              SEC_COMMIT,
//                              NULL,
//                              NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create a section Status=0x%x\n", Status);
//         EngFreeMem(pSection);
//         return NULL;
//     }
//
//     /* Set the fields of the section */
//     pSection->ulTag = ulTag;
//     pSection->pvSectionObject = pvSectionObject;
//     pSection->pvMappedBase = NULL;
//     pSection->cjViewSize = cjSize;
//
//     return pSection;
// }
//
// PVOID
// NTAPI
// EngCreateSectionHack(
//     _In_ ULONG fl,
//     _In_ SIZE_T cjSize,
//     _In_ ULONG ulTag)
// {
//     NTSTATUS Status;
//     PENGSECTION pSection;
//     PVOID pvSectionObject;
//     LARGE_INTEGER liSize;
//
//     /* Allocate a section object */
//     pSection = EngAllocMem(0, sizeof(ENGSECTION), 'stsU');
//     if (!pSection) return NULL;
//
//     liSize.QuadPart = cjSize;
//     Status = MmCreateSection(&pvSectionObject,
//                              SECTION_ALL_ACCESS,
//                              NULL,
//                              &liSize,
//                              PAGE_READWRITE,
//                              SEC_COMMIT | 1,
//                              NULL,
//                              NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create a section Status=0x%x\n", Status);
//         EngFreeMem(pSection);
//         return NULL;
//     }
//
//     /* Set the fields of the section */
//     pSection->ulTag = ulTag;
//     pSection->pvSectionObject = pvSectionObject;
//     pSection->pvMappedBase = NULL;
//     pSection->cjViewSize = cjSize;
//
//     return pSection;
// }
//
// _Success_(return!=FALSE)
// BOOL
// APIENTRY
// EngMapSection(
//     _In_ PVOID pvSection,
//     _In_ BOOL bMap,
//     _In_ HANDLE hProcess,
//     _When_(bMap, _Outptr_) PVOID* pvBaseAddress)
// {
//     NTSTATUS Status;
//     PENGSECTION pSection = pvSection;
//     PEPROCESS pepProcess;
//
//     /* Get a pointer to the process */
//     Status = ObReferenceObjectByHandle(hProcess,
//                                        PROCESS_VM_OPERATION,
//                                        NULL,
//                                        KernelMode,
//                                        (PVOID*)&pepProcess,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Could not access process %p, Status=0x%lx\n", hProcess, Status);
//         return FALSE;
//     }
//
//     if (bMap)
//     {
//         /* Make sure the section isn't already mapped */
//         ASSERT(pSection->pvMappedBase == NULL);
//
//         /* Map the section into the process address space */
//         Status = MmMapViewOfSection(pSection->pvSectionObject,
//                                     pepProcess,
//                                     &pSection->pvMappedBase,
//                                     0,
//                                     pSection->cjViewSize,
//                                     NULL,
//                                     &pSection->cjViewSize,
//                                     ViewUnmap,
//                                     0,
//                                     PAGE_READWRITE);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to map a section Status=0x%x\n", Status);
//         }
//     }
//     else
//     {
//         /* Make sure the section is mapped */
//         ASSERT(pSection->pvMappedBase);
//
//         /* Unmap the section from the process address space */
//         Status = MmUnmapViewOfSection(pepProcess, pSection->pvMappedBase);
//         if (NT_SUCCESS(Status))
//         {
//             pSection->pvMappedBase = NULL;
//         }
//         else
//         {
//             DPRINT1("Failed to unmap a section @ %p Status=0x%x\n",
//                     pSection->pvMappedBase, Status);
//         }
//     }
//
//     /* Dereference the process */
//     ObDereferenceObject(pepProcess);
//
//     /* Set the new mapping base and return bool status */
//     *pvBaseAddress = pSection->pvMappedBase;
//     return NT_SUCCESS(Status);
// }
//
// BOOL
// APIENTRY
// EngFreeSectionMem(
//     _In_opt_ PVOID pvSection,
//     _In_opt_ PVOID pvMappedBase)
// {
//     NTSTATUS Status;
//     PENGSECTION pSection = pvSection;
//     BOOL bResult = TRUE;
//
//     /* Did the caller give us a mapping base? */
//     if (pvMappedBase)
//     {
//         Status = MmUnmapViewInSessionSpace(pvMappedBase);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("MmUnmapViewInSessionSpace failed: 0x%lx\n", Status);
//             bResult = FALSE;
//         }
//     }
//
//     /* Check if we should free the section as well */
//     if (pSection)
//     {
//         /* Dereference the kernel section */
//         ObDereferenceObject(pSection->pvSectionObject);
//
//         /* Finally free the section memory itself */
//         EngFreeMem(pSection);
//     }
//
//     return bResult;
// }
//
// _Check_return_
// _Success_(return!=NULL)
// __drv_allocatesMem(Mem)
// _Post_writable_byte_size_(cjSize)
// PVOID
// APIENTRY
// EngAllocSectionMem(
//     _Outptr_ PVOID *ppvSection,
//     _In_ ULONG fl,
//     _In_ SIZE_T cjSize,
//     _In_ ULONG ulTag)
// {
//     NTSTATUS Status;
//     PENGSECTION pSection;
//
//     /* Check parameter */
//     if (cjSize == 0) return NULL;
//
//     /* Allocate a section object */
//     pSection = EngCreateSectionHack(fl, cjSize, ulTag);
//     if (!pSection)
//     {
//         *ppvSection = NULL;
//         return NULL;
//     }
//
//     /* Map the section in session space */
//     Status = MmMapViewInSessionSpace(pSection->pvSectionObject,
//                                      &pSection->pvMappedBase,
//                                      &pSection->cjViewSize);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to map a section Status=0x%x\n", Status);
//         *ppvSection = NULL;
//         EngFreeSectionMem(pSection, NULL);
//         return NULL;
//     }
//
//     if (fl & FL_ZERO_MEMORY)
//     {
//         RtlZeroMemory(pSection->pvMappedBase, cjSize);
//     }
//
//     /* Set section pointer and return base address */
//     *ppvSection = pSection;
//     return pSection->pvMappedBase;
// }
//
// _Check_return_
// PFILEVIEW
// NTAPI
// EngLoadModuleEx(
//     _In_z_ LPWSTR pwsz,
//     _In_ ULONG cjSizeOfModule,
//     _In_ FLONG fl)
// {
//     PFILEVIEW pFileView = NULL;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE hRootDir;
//     UNICODE_STRING ustrFileName;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_BASIC_INFORMATION FileInformation;
//     HANDLE hFile;
//     NTSTATUS Status;
//     LARGE_INTEGER liSize;
//
//     if (fl & FVF_FONTFILE)
//     {
//         pFileView = EngAllocMem(0, sizeof(FONTFILEVIEW), 'vffG');
//     }
//     else
//     {
//         pFileView = EngAllocMem(0, sizeof(FILEVIEW), 'liFg');
//     }
//
//     /* Check for success */
//     if (!pFileView) return NULL;
//
//     /* Check if the file is relative to system32 */
//     if (fl & FVF_SYSTEMROOT)
//     {
//         hRootDir = ghSystem32Directory;
//     }
//     else
//     {
//         hRootDir = ghRootDirectory;
//     }
//
//     /* Initialize unicode string and object attributes */
//     RtlInitUnicodeString(&ustrFileName, pwsz);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &ustrFileName,
//                                OBJ_CASE_INSENSITIVE|OBJ_KERNEL_HANDLE,
//                                hRootDir,
//                                NULL);
//
//     /* Now open the file */
//     Status = ZwCreateFile(&hFile,
//                           FILE_READ_DATA,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           FILE_ATTRIBUTE_NORMAL,
//                           0,
//                           FILE_OPEN,
//                           FILE_NON_DIRECTORY_FILE,
//                           NULL,
//                           0);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open file '%S' Status=0x%x\n", pwsz, Status);
//         EngFreeMem(pFileView);
//         return NULL;
//     }
//
//     Status = ZwQueryInformationFile(hFile,
//                                     &IoStatusBlock,
//                                     &FileInformation,
//                                     sizeof(FILE_BASIC_INFORMATION),
//                                     FileBasicInformation);
//     if (NT_SUCCESS(Status))
//     {
//         pFileView->LastWriteTime = FileInformation.LastWriteTime;
//     }
//
//     /* Create a section from the file */
//     liSize.QuadPart = cjSizeOfModule;
//     Status = MmCreateSection(&pFileView->pSection,
//                              SECTION_ALL_ACCESS,
//                              NULL,
//                              &liSize,
//                              fl & FVF_READONLY ? PAGE_EXECUTE_READ : PAGE_EXECUTE_READWRITE,
//                              SEC_COMMIT,
//                              hFile,
//                              NULL);
//
//     /* Close the file handle */
//     ZwClose(hFile);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create a section Status=0x%x\n", Status);
//         EngFreeMem(pFileView);
//         return NULL;
//     }
//
//
//     pFileView->pvKView = NULL;
//     pFileView->pvViewFD = NULL;
//     pFileView->cjView = 0;
//
//     return pFileView;
// }
//
// HANDLE
// APIENTRY
// EngLoadModule(
//     _In_ LPWSTR pwsz)
// {
//     /* Forward to EngLoadModuleEx */
//     return (HANDLE)EngLoadModuleEx(pwsz, 0, FVF_READONLY | FVF_SYSTEMROOT);
// }
//
// HANDLE
// APIENTRY
// EngLoadModuleForWrite(
//     _In_ LPWSTR pwsz,
//     _In_ ULONG  cjSizeOfModule)
// {
//     /* Forward to EngLoadModuleEx */
//     return (HANDLE)EngLoadModuleEx(pwsz, cjSizeOfModule, FVF_SYSTEMROOT);
// }
//
// _Check_return_
// _Success_(return!=NULL)
// _Post_writable_byte_size_(*pulSize)
// PVOID
// APIENTRY
// EngMapModule(
//     _In_  HANDLE h,
//     _Out_ PULONG pulSize)
// {
//     PFILEVIEW pFileView = (PFILEVIEW)h;
//     NTSTATUS Status;
//
//     pFileView->cjView = 0;
//
//     /* FIXME: Use system space because ARM3 doesn't support executable sections yet */
//     Status = MmMapViewInSystemSpace(pFileView->pSection,
//                                     &pFileView->pvKView,
//                                     &pFileView->cjView);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to map a section Status=0x%x\n", Status);
//         *pulSize = 0;
//         return NULL;
//     }
//
//     *pulSize = (ULONG)pFileView->cjView;
//     return pFileView->pvKView;
// }
//
// VOID
// APIENTRY
// EngFreeModule(
//     _In_ _Post_invalid_ HANDLE h)
// {
//     PFILEVIEW pFileView = (PFILEVIEW)h;
//     NTSTATUS Status;
//
//     /* FIXME: Use system space because ARM3 doesn't support executable sections yet */
//     Status = MmUnmapViewInSystemSpace(pFileView->pvKView);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("MmUnmapViewInSessionSpace failed: 0x%lx\n", Status);
//         ASSERT(FALSE);
//     }
//
//     /* Dereference the section */
//     ObDereferenceObject(pFileView->pSection);
//
//     /* Free the file view memory */
//     EngFreeMem(pFileView);
// }
//
// _Success_(return != 0)
// _When_(cjSize != 0, _At_(return, _Out_writes_bytes_(cjSize)))
// PVOID
// APIENTRY
// EngMapFile(
//     _In_ LPWSTR pwsz,
//     _In_ ULONG cjSize,
//     _Out_ ULONG_PTR *piFile)
// {
//     HANDLE hModule;
//     PVOID pvBase;
//
//     /* Load the file */
//     hModule = EngLoadModuleEx(pwsz, 0, 0);
//     if (!hModule)
//     {
//         *piFile = 0;
//         return NULL;
//     }
//
//     /* Map the file */
//     pvBase = EngMapModule(hModule, &cjSize);
//     if (!pvBase)
//     {
//         EngFreeModule(hModule);
//         hModule = NULL;
//     }
//
//     /* Set iFile and return mapped base */
//     *piFile = (ULONG_PTR)hModule;
//     return pvBase;
// }
//
// BOOL
// APIENTRY
// EngUnmapFile(
//     _In_ ULONG_PTR iFile)
// {
//     HANDLE hModule = (HANDLE)iFile;
//
//     EngFreeModule(hModule);
//
//     return TRUE;
// }
//
// _Check_return_
// _Success_(return!=FALSE)
// BOOL
// APIENTRY
// EngMapFontFileFD(
// 	_In_ ULONG_PTR iFile,
// 	_Outptr_result_bytebuffer_(*pcjBuf) PULONG *ppjBuf,
// 	_Out_ ULONG *pcjBuf)
// {
//     // www.osr.com/ddk/graphics/gdifncs_0co7.htm
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// VOID
// APIENTRY
// EngUnmapFontFileFD(
//     _In_ ULONG_PTR iFile)
// {
//     // http://www.osr.com/ddk/graphics/gdifncs_6wbr.htm
//     UNIMPLEMENTED;
// }
//
// __drv_preferredFunction("EngMapFontFileFD", "Obsolete")
// _Check_return_
// _Success_(return!=FALSE)
// BOOL
// APIENTRY
// EngMapFontFile(
//     _In_ ULONG_PTR iFile,
//     _Outptr_result_bytebuffer_(*pcjBuf) PULONG *ppjBuf,
//     _Out_ ULONG *pcjBuf)
// {
//     // www.osr.com/ddk/graphics/gdifncs_3up3.htm
//     return EngMapFontFileFD(iFile, ppjBuf, pcjBuf);
// }
//
// VOID
// APIENTRY
// EngUnmapFontFile(
//     _In_ ULONG_PTR iFile)
// {
//     // www.osr.com/ddk/graphics/gdifncs_09wn.htm
//     EngUnmapFontFileFD(iFile);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
