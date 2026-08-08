// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct RecycleBin5File {
    pub vtable: *const IRecycleBinFileVtbl,
    pub m_ref: LONG,
    pub m_recycleBin: *mut IRecycleBin5,
    pub m_deletedFile: DELETED_FILE_RECORD,
    pub m_FullName: LPWSTR,
}

#[repr(C)]
pub struct RecycleBin5Enum {
    pub vtable: *const IRecycleBinEnumListVtbl,
    pub m_ref: LONG,
    pub m_recycleBin: *mut IRecycleBin5,
    pub m_hInfo: HANDLE,
    pub m_pInfo: *mut INFO2_HEADER,
    pub m_dwCurrent: DWORD,
    pub m_pszPrefix: LPWSTR,
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5File_IsEqualIdentity(
    this: *mut RecycleBin5File,
    pFI: *const RECYCLEBINFILEIDENTITY,
) -> HRESULT {
    let self_value = RECYCLEBINFILEIDENTITY {
        DeletionTime: (*this).m_deletedFile.DeletionTime,
        FullName: (*this).m_FullName,
    };
    let is_equal = RecycleBinGeneric_IsEqualFileIdentity(pFI, &self_value);
    if is_equal != 0 {
        S_OK
    } else {
        S_FALSE
    }
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5File_QueryInterface(
    this: *mut RecycleBin5File,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppvObject.is_null() {
        return E_POINTER;
    }

    if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IRecycleBinFile) {
        *ppvObject = this as *mut core::ffi::c_void;
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    AddRef(this);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5File_AddRef(this: *mut RecycleBin5File) -> u32 {
    (InterlockedIncrement(core::ptr::addr_of_mut!((*this).m_ref))) as u32
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5File_dtor(this: *mut RecycleBin5File) {
    IRecycleBin5_Release((*this).m_recycleBin);
    //
    SHFree(((*this).m_FullName).cast());
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5File_Release(this: *mut RecycleBin5File) -> u32 {
    let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*this).m_ref)) as u32;
    if ref_count == 0 as u32 {
        RecycleBin5File_Destroy(this);
    }
    ref_count
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5File_GetInfo(this: *mut RecycleBin5File, pInfo: PDELETED_FILE_INFO) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut uli: ULARGE_INTEGER = core::mem::zeroed();
    C_ASSERT(true);
    if GetLastModificationTime(this, core::ptr::addr_of_mut!((*pInfo).LastModification)) < 0 {
        ZeroMemory!(core::ptr::addr_of_mut!((*pInfo).LastModification), core::mem::size_of::<FILETIME>());
    }
    (*pInfo).DeletionTime = (*this).m_deletedFile.DeletionTime;
    if GetFileSize(this, &raw mut uli) < 0 {
        (*pInfo).FileSize = INVALID_FILE_SIZE;
    } else {
        (*pInfo).FileSize = uli.__bindgen_anon_1.LowPart;
    }
    hr = GetAttributes(this, core::ptr::addr_of_mut!((*pInfo).Attributes));
    if hr < 0 {
        (*pInfo).Attributes = 0;
    }
    InitializeRecycleBinStringRef(core::ptr::addr_of_mut!((*pInfo).OriginalFullPath), (*this).m_deletedFile.FileNameW);
    InitializeRecycleBinStringRef(core::ptr::addr_of_mut!((*pInfo).RecycledFullPath), (*this).m_FullName);
    hr
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5File_GetLastModificationTime(
    this: *mut RecycleBin5File,
    pLastModificationTime: *mut FILETIME,
) -> HRESULT {
    let dwAttributes: DWORD = GetFileAttributesW((*this).m_FullName);
    if dwAttributes == INVALID_FILE_ATTRIBUTES {
        return HResultFromWin32(GetLastError());
    }
    let mut hFile: HANDLE;
    hFile = CreateFileW(
        (*this).m_FullName,
        GENERIC_READ,
        FILE_SHARE_READ
            | if (dwAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                FILE_SHARE_WRITE | FILE_SHARE_DELETE
            } else {
                0
            },
        core::ptr::null_mut(),
        OPEN_EXISTING,
        if (dwAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
            FILE_FLAG_BACKUP_SEMANTICS
        } else {
            0
        },
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        return HResultFromWin32(GetLastError());
    }
    let hr: HRESULT;
    if GetFileTime(
        hFile,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pLastModificationTime,
    ) != 0
    {
        hr = S_OK;
    } else {
        hr = HResultFromWin32(GetLastError());
    }
    CloseHandle(hFile);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5File_GetDeletionTime(
    this: *mut RecycleBin5File,
    pDeletionTime: *mut FILETIME,
) -> HRESULT {
    *pDeletionTime = (*this).m_deletedFile.DeletionTime;
    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5File_GetFileSize(
    this: *mut RecycleBin5File,
    pFileSize: *mut ULARGE_INTEGER,
) -> HRESULT {
    let dwAttributes = GetFileAttributesW((*this).m_FullName);
    if dwAttributes == INVALID_FILE_ATTRIBUTES {
        return HRESULT_FROM_WIN32((GetLastError()) as LONG);
    }
    if (dwAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
        (*pFileSize).QuadPart = 0;
        return S_OK;
    }
    let hFile = CreateFileW(
        (*this).m_FullName,
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        return HRESULT_FROM_WIN32((GetLastError()) as LONG);
    }
    (*pFileSize).u.__bindgen_anon_1.LowPart = GetFileSize(this,
        hFile,
        core::ptr::addr_of_mut!((*pFileSize).u.__bindgen_anon_1.HighPart),
    );
    let hr;
    if (*pFileSize).u.__bindgen_anon_1.LowPart != INVALID_FILE_SIZE {
        hr = S_OK;
    } else {
        hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
    }
    CloseHandle(hFile);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5File_GetPhysicalFileSize(
    this: *mut RecycleBin5File,
    pPhysicalFileSize: *mut ULARGE_INTEGER,
) -> HRESULT {
    (*pPhysicalFileSize).u.__bindgen_anon_1.HighPart = 0;
    (*pPhysicalFileSize).u.__bindgen_anon_1.LowPart = (*this).m_deletedFile.dwPhysicalFileSize;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5File_GetAttributes(
    this: *mut RecycleBin5File,
    pAttributes: *mut u32,
) -> i32 {
    let dwAttributes: u32 = GetFileAttributesW((*this).m_FullName as *const u16);
    if dwAttributes == INVALID_FILE_ATTRIBUTES {
        return HResultFromWin32(GetLastError());
    }
    *pAttributes = dwAttributes;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5File_GetFileName(
    this: *mut RecycleBin5File,
    BufferSize: usize,
    Buffer: *mut u16,
    RequiredSize: *mut usize,
) -> HRESULT {
    let file_name_ptr = (*this).m_deletedFile.FileNameW;
    let len = wcslen(file_name_ptr) + 1;
    let dw_required = len.wrapping_mul(core::mem::size_of::<u16>());

    if !RequiredSize.is_null() {
        *RequiredSize = dw_required;
    }

    if BufferSize == 0 && Buffer.is_null() {
        return S_OK;
    }

    if BufferSize < dw_required {
        return E_OUTOFMEMORY;
    }

    CopyMemory(
        Buffer as *mut core::ffi::c_void,
        file_name_ptr as *const core::ffi::c_void,
        dw_required,
    );

    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5File_Delete(this: *mut RecycleBin5File) -> HRESULT {
    ((*this).m_recycleBin).Delete(this,
        (*this).m_FullName,
        core::ptr::addr_of_mut!((*this).m_deletedFile),
    )
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5File_Restore(this: *mut RecycleBin5File) -> HRESULT {
    let m_recycleBin = (*this).m_recycleBin;
    let m_FullName = (*this).m_FullName;
    Restore(m_recycleBin, m_FullName, &mut (*this).m_deletedFile)
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5File_RemoveFromDatabase(this: *mut RecycleBin5File) -> HRESULT {
    RemoveFromDatabase((*this).m_recycleBin, (*this).m_FullName, core::ptr::addr_of_mut!((*this).m_deletedFile))
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5File_ctor(this: *mut RecycleBin5File) {
    (*this).m_ref = 1;
    (*this).m_recycleBin = core::ptr::null_mut();
    (*this).m_FullName = core::ptr::null_mut();
    ZeroMemory!(
        core::ptr::addr_of_mut!((*this).m_deletedFile),
        core::mem::size_of::<DELETED_FILE_RECORD>(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5File_Init(
    this: *mut RecycleBin5File,
    prb: *mut IRecycleBin5,
    Folder: LPCWSTR,
    pDeletedFile: *mut DELETED_FILE_RECORD,
) -> HRESULT {
    (*this).m_recycleBin = prb;
    (*(*this).m_recycleBin).AddRef();
    (*this).m_deletedFile = (*pDeletedFile);

    let mut szUniqueId: [WCHAR; 32] = [0; 32];
    StringCchPrintfW(
        szUniqueId.as_mut_ptr(),
        _countof!(szUniqueId),
        L"%lu".as_ptr() as LPCWSTR,
        (*pDeletedFile).dwRecordUniqueId,
    );

    let mut strFullName = CStringW::from_ptr(Folder);
    strFullName.Append(L"\\D".as_ptr());
    strFullName.AppendChar((0x0061u16 + (*pDeletedFile).dwDriveNumber as u16) as WCHAR);
    strFullName.Append(szUniqueId.as_ptr());
    strFullName.Append(PathFindExtensionW((*pDeletedFile).FileNameW));

    if GetFileAttributesW(strFullName.as_ptr()) == INVALID_FILE_ATTRIBUTES {
        return E_FAIL;
    }

    SHStrDup(
        strFullName.as_ptr() as LPCWSTR,
        core::ptr::addr_of_mut!((*this).m_FullName),
    )
}
unsafe extern "C" fn RecycleBin5File_Constructor(
    prb: *mut IRecycleBin5,
    Folder: LPCWSTR,
    pDeletedFile: *mut DELETED_FILE_RECORD,
    ppFile: *mut *mut IRecycleBinFile,
) -> HRESULT {
    if ppFile.is_null() {
        return E_POINTER;
    }
    *ppFile = core::ptr::null_mut();
    let pThis = RecycleBin5File() as *mut RecycleBin5File;
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }
    let hr = (*pThis).Init(prb, Folder, pDeletedFile);
    if hr < 0 {
        drop(pThis);
        return hr;
    }
    *ppFile = pThis as *mut IRecycleBinFile;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5Enum_QueryInterface(
    this: *mut RecycleBin5Enum,
    riid: *const GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppvObject.is_null() {
        return E_POINTER;
    }

    if IsEqualIID(riid, &IID_IUnknown) != 0 || IsEqualIID(riid, &IID_IRecycleBinEnumList) != 0 {
        *ppvObject = this as *mut core::ffi::c_void;
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    AddRef(this);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5Enum_AddRef(this: *mut RecycleBin5Enum) -> ULONG {
    (InterlockedIncrement(core::ptr::addr_of_mut!((*this).m_ref))) as u32
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5Enum_dtor(this: *mut RecycleBin5Enum) {
    let recycle_bin = (*this).m_recycleBin;
    if !recycle_bin.is_null() {
        let lpVtbl = (*(*recycle_bin).lpVtbl);
        (lpVtbl.OnClosing)(recycle_bin, this);
    }
    UnmapViewOfFile((*this).m_pInfo);
    if !recycle_bin.is_null() {
        let lpVtbl = (*(*recycle_bin).lpVtbl);
        (lpVtbl.Release)(recycle_bin);
    }
    SHFree(((*this).m_pszPrefix).cast());
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5Enum_Release(this: *mut RecycleBin5Enum) -> ULONG {
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*this).m_ref));
    if refCount == 0 {
        RecycleBin5Enum_delete(this);
    }
    (refCount) as u32
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5Enum_Skip(this: *mut RecycleBin5Enum, celt: u32) -> HRESULT {
    (*this).m_dwCurrent += celt;
    S_OK
}

#[no_mangle] pub unsafe extern "C" fn RecycleBin5Enum_Reset(this: *mut RecycleBin5Enum) -> HRESULT {
    (*this).m_dwCurrent = 0;
    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5Enum_ctor(this: *mut RecycleBin5Enum) {
    (*this).m_ref = 1;
    (*this).m_recycleBin = core::ptr::null_mut();
    (*this).m_hInfo = core::ptr::null_mut();
    (*this).m_pInfo = core::ptr::null_mut();
    (*this).m_dwCurrent = 0;
    (*this).m_pszPrefix = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5Enum_Init(
    this: *mut RecycleBin5Enum,
    prb: *mut IRecycleBin5,
    hInfo: *mut core::ffi::c_void,
    hInfoMapped: *mut core::ffi::c_void,
    pszPrefix: *const u16,
) -> HRESULT {
    (*this).m_recycleBin = prb;
    (*(*this).m_recycleBin).AddRef();
    let hr = SHStrDup(pszPrefix, core::ptr::addr_of_mut!((*this).m_pszPrefix));
    if hr < 0 {
        return hr;
    }
    (*this).m_hInfo = hInfo;
    (*this).m_pInfo =
        MapViewOfFile(hInfoMapped, FILE_MAP_READ, 0, 0, 0)
            as *mut INFO2_HEADER;
    if (*this).m_pInfo.is_null() {
        return HResultFromWin32(GetLastError());
    }
    if (*(*this).m_pInfo).dwVersion != 5
        || (*(*this).m_pInfo).dwRecordSize != core::mem::size_of::<DELETED_FILE_RECORD>() as u32
    {
        return E_FAIL;
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5Enum_Constructor(
    prb: *mut IRecycleBin5,
    hInfo: HANDLE,
    hInfoMapped: HANDLE,
    szPrefix: LPCWSTR,
    ppUnknown: *mut *mut IUnknown,
) -> HRESULT {
    if ppUnknown.is_null() {
        return E_POINTER;
    }
    *ppUnknown = core::ptr::null_mut();

    let pThis: *mut RecycleBin5Enum = RecycleBin5Enum();
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }

    let hr: HRESULT = RecycleBin5Enum_Init(pThis, prb, hInfo, hInfoMapped, szPrefix);
    if hr < 0 {
        RecycleBin5Enum_delete(pThis);
        return hr;
    }

    *ppUnknown = pThis as *mut IUnknown;
    S_OK
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Recycle bin management
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Enumerates contents of a MS Windows 2000/XP/2003 recyclebin
//  * COPYRIGHT:   Copyright 2006-2007 Hervé Poussineau (hpoussin@reactos.org)
//  *              Copyright 2024 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include "recyclebin_private.h"
// #include <atlstr.h>
// #include <shlwapi.h>
// #include <strsafe.h>
//
// class RecycleBin5File : public IRecycleBinFile
// {
// public:
//     RecycleBin5File();
//     virtual ~RecycleBin5File();
//
//     HRESULT Init(
//          IRecycleBin5 *prb,
//          LPCWSTR Folder,
//          PDELETED_FILE_RECORD pDeletedFile);
//
//     /* IUnknown methods */
//     HRESULT STDMETHODCALLTYPE QueryInterface(REFIID riid, void **ppvObject) override;
//     STDMETHODIMP_(ULONG) AddRef() override;
//     STDMETHODIMP_(ULONG) Release() override;
//
//     /* IRecycleBinFile methods */
//     HRESULT STDMETHODCALLTYPE IsEqualIdentity(const RECYCLEBINFILEIDENTITY *pFI) override
//     {
//         RECYCLEBINFILEIDENTITY self = { m_deletedFile.DeletionTime, m_FullName };
//         return RecycleBinGeneric_IsEqualFileIdentity(pFI, &self) ? S_OK : S_FALSE;
//     }
//     HRESULT STDMETHODCALLTYPE GetInfo(PDELETED_FILE_INFO pInfo) override;
//     HRESULT STDMETHODCALLTYPE GetLastModificationTime(FILETIME *pLastModificationTime) override;
//     HRESULT STDMETHODCALLTYPE GetDeletionTime(FILETIME *pDeletionTime) override;
//     HRESULT STDMETHODCALLTYPE GetFileSize(ULARGE_INTEGER *pFileSize) override;
//     HRESULT STDMETHODCALLTYPE GetPhysicalFileSize(ULARGE_INTEGER *pPhysicalFileSize) override;
//     HRESULT STDMETHODCALLTYPE GetAttributes(DWORD *pAttributes) override;
//     HRESULT STDMETHODCALLTYPE GetFileName(SIZE_T BufferSize, LPWSTR Buffer, SIZE_T *RequiredSize) override;
//     HRESULT STDMETHODCALLTYPE Delete() override;
//     HRESULT STDMETHODCALLTYPE Restore() override;
//     HRESULT STDMETHODCALLTYPE RemoveFromDatabase() override;
//
// protected:
//     LONG m_ref;
//     IRecycleBin5 *m_recycleBin;
//     DELETED_FILE_RECORD m_deletedFile;
//     LPWSTR m_FullName;
// };
//
// HRESULT STDMETHODCALLTYPE RecycleBin5File::QueryInterface(REFIID riid, void **ppvObject)
// {
//     TRACE("(%p, %s, %p)\n", this, debugstr_guid(&riid), ppvObject);
//
//     if (!ppvObject)
//         return E_POINTER;
//
//     if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_IRecycleBinFile))
//     {
//         *ppvObject = static_cast<IRecycleBinFile *>(this);
//     }
//     else
//     {
//         *ppvObject = NULL;
//         return E_NOINTERFACE;
//     }
//
//     AddRef();
//     return S_OK;
// }
//
// STDMETHODIMP_(ULONG) RecycleBin5File::AddRef()
// {
//     TRACE("(%p)\n", this);
//     return InterlockedIncrement(&m_ref);
// }
//
// RecycleBin5File::~RecycleBin5File()
// {
//     TRACE("(%p)\n", this);
//     m_recycleBin->Release();
//     SHFree(m_FullName);
// }
//
// STDMETHODIMP_(ULONG) RecycleBin5File::Release()
// {
//     TRACE("(%p)\n", this);
//     ULONG refCount = InterlockedDecrement(&m_ref);
//     if (refCount == 0)
//         delete this;
//     return refCount;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5File::GetInfo(PDELETED_FILE_INFO pInfo)
// {
//     HRESULT hr = S_OK;
//     ULARGE_INTEGER uli;
//     if (FAILED(GetLastModificationTime(&pInfo->LastModification)))
//         ZeroMemory(&pInfo->LastModification, sizeof(pInfo->LastModification));
//     pInfo->DeletionTime = m_deletedFile.DeletionTime;
//     C_ASSERT(sizeof(pInfo->FileSize) <= sizeof(UINT));
//     pInfo->FileSize = FAILED(GetFileSize(&uli)) ? INVALID_FILE_SIZE : uli.LowPart;
//     if (FAILED(hr = GetAttributes(&pInfo->Attributes)))
//         pInfo->Attributes = 0;
//     InitializeRecycleBinStringRef(&pInfo->OriginalFullPath, m_deletedFile.FileNameW);
//     InitializeRecycleBinStringRef(&pInfo->RecycledFullPath, m_FullName);
//     return hr;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5File::GetLastModificationTime(FILETIME *pLastModificationTime)
// {
//     TRACE("(%p, %p)\n", this, pLastModificationTime);
//
//     DWORD dwAttributes = ::GetFileAttributesW(m_FullName);
//     if (dwAttributes == INVALID_FILE_ATTRIBUTES)
//         return HResultFromWin32(GetLastError());
//
//     HANDLE hFile;
//     hFile = CreateFileW(m_FullName,
//                         GENERIC_READ,
//                         FILE_SHARE_READ |
//                             ((dwAttributes & FILE_ATTRIBUTE_DIRECTORY) ? (FILE_SHARE_WRITE | FILE_SHARE_DELETE) : 0),
//                         NULL,
//                         OPEN_EXISTING,
//                         (dwAttributes & FILE_ATTRIBUTE_DIRECTORY) ? FILE_FLAG_BACKUP_SEMANTICS : 0,
//                         NULL);
//     if (hFile == INVALID_HANDLE_VALUE)
//         return HResultFromWin32(GetLastError());
//
//     HRESULT hr;
//     if (GetFileTime(hFile, NULL, NULL, pLastModificationTime))
//         hr = S_OK;
//     else
//         hr = HResultFromWin32(GetLastError());
//     CloseHandle(hFile);
//     return hr;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5File::GetDeletionTime(FILETIME *pDeletionTime)
// {
//     TRACE("(%p, %p)\n", this, pDeletionTime);
//     *pDeletionTime = m_deletedFile.DeletionTime;
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5File::GetFileSize(ULARGE_INTEGER *pFileSize)
// {
//     TRACE("(%p, %p)\n", this, pFileSize);
//
//     DWORD dwAttributes = GetFileAttributesW(m_FullName);
//     if (dwAttributes == INVALID_FILE_ATTRIBUTES)
//         return HRESULT_FROM_WIN32(GetLastError());
//     if (dwAttributes & FILE_ATTRIBUTE_DIRECTORY)
//     {
//         pFileSize->QuadPart = 0;
//         return S_OK;
//     }
//
//     HANDLE hFile = CreateFileW(m_FullName, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
//     if (hFile == INVALID_HANDLE_VALUE)
//         return HRESULT_FROM_WIN32(GetLastError());
//     pFileSize->u.LowPart = ::GetFileSize(hFile, &pFileSize->u.HighPart);
//
//     HRESULT hr;
//     if (pFileSize->u.LowPart != INVALID_FILE_SIZE)
//         hr = S_OK;
//     else
//         hr = HRESULT_FROM_WIN32(GetLastError());
//
//     CloseHandle(hFile);
//     return hr;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5File::GetPhysicalFileSize(ULARGE_INTEGER *pPhysicalFileSize)
// {
//     TRACE("(%p, %p)\n", this, pPhysicalFileSize);
//     pPhysicalFileSize->u.HighPart = 0;
//     pPhysicalFileSize->u.LowPart = m_deletedFile.dwPhysicalFileSize;
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5File::GetAttributes(DWORD *pAttributes)
// {
//     DWORD dwAttributes;
//
//     TRACE("(%p, %p)\n", this, pAttributes);
//
//     dwAttributes = GetFileAttributesW(m_FullName);
//     if (dwAttributes == INVALID_FILE_ATTRIBUTES)
//         return HResultFromWin32(GetLastError());
//
//     *pAttributes = dwAttributes;
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5File::GetFileName(SIZE_T BufferSize, LPWSTR Buffer, SIZE_T *RequiredSize)
// {
//     DWORD dwRequired;
//
//     TRACE("(%p, %u, %p, %p)\n", this, BufferSize, Buffer, RequiredSize);
//
//     dwRequired = (DWORD)(wcslen(m_deletedFile.FileNameW) + 1) * sizeof(WCHAR);
//     if (RequiredSize)
//         *RequiredSize = dwRequired;
//
//     if (BufferSize == 0 && !Buffer)
//         return S_OK;
//
//     if (BufferSize < dwRequired)
//         return E_OUTOFMEMORY;
//     CopyMemory(Buffer, m_deletedFile.FileNameW, dwRequired);
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5File::Delete()
// {
//     TRACE("(%p)\n", this);
//     return m_recycleBin->Delete(m_FullName, &m_deletedFile);
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5File::Restore()
// {
//     TRACE("(%p)\n", this);
//     return m_recycleBin->Restore(m_FullName, &m_deletedFile);
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5File::RemoveFromDatabase()
// {
//     TRACE("(%p)\n", this);
//     return m_recycleBin->RemoveFromDatabase(m_FullName, &m_deletedFile);
// }
//
// RecycleBin5File::RecycleBin5File()
//     : m_ref(1)
//     , m_recycleBin(NULL)
//     , m_FullName(NULL)
// {
//     ZeroMemory(&m_deletedFile, sizeof(m_deletedFile));
// }
//
// HRESULT
// RecycleBin5File::Init(
//      IRecycleBin5 *prb,
//      LPCWSTR Folder,
//      PDELETED_FILE_RECORD pDeletedFile)
// {
//     m_recycleBin = prb;
//     m_recycleBin->AddRef();
//     m_deletedFile = *pDeletedFile;
//
//     WCHAR szUniqueId[32];
//     StringCchPrintfW(szUniqueId, _countof(szUniqueId), L"%lu", pDeletedFile->dwRecordUniqueId);
//
//     CStringW strFullName(Folder);
//     strFullName += L"\\D";
//     strFullName += (WCHAR)(L'a' + pDeletedFile->dwDriveNumber);
//     strFullName += szUniqueId;
//     strFullName += PathFindExtensionW(pDeletedFile->FileNameW);
//     if (GetFileAttributesW(strFullName) == INVALID_FILE_ATTRIBUTES)
//         return E_FAIL;
//
//     return SHStrDup(strFullName, &m_FullName);
// }
//
// static HRESULT
// RecycleBin5File_Constructor(
//      IRecycleBin5 *prb,
//      LPCWSTR Folder,
//      PDELETED_FILE_RECORD pDeletedFile,
//      IRecycleBinFile **ppFile)
// {
//     if (!ppFile)
//         return E_POINTER;
//
//     *ppFile = NULL;
//
//     RecycleBin5File *pThis = new RecycleBin5File();
//     if (!pThis)
//         return E_OUTOFMEMORY;
//
//     HRESULT hr = pThis->Init(prb, Folder, pDeletedFile);
//     if (FAILED(hr))
//     {
//         delete pThis;
//         return hr;
//     }
//
//     *ppFile = static_cast<IRecycleBinFile *>(pThis);
//     return S_OK;
// }
//
// class RecycleBin5Enum : public IRecycleBinEnumList
// {
// public:
//     RecycleBin5Enum();
//     virtual ~RecycleBin5Enum();
//
//     HRESULT Init(
//          IRecycleBin5 *prb,
//          HANDLE hInfo,
//          HANDLE hInfoMapped,
//          LPCWSTR pszPrefix);
//
//     /* IUnknown methods */
//     HRESULT STDMETHODCALLTYPE QueryInterface(REFIID riid, void **ppvObject) override;
//     STDMETHODIMP_(ULONG) AddRef() override;
//     STDMETHODIMP_(ULONG) Release() override;
//
//     /* IRecycleBinEnumList methods */
//     HRESULT STDMETHODCALLTYPE Next(DWORD celt, IRecycleBinFile **rgelt, DWORD *pceltFetched) override;
//     HRESULT STDMETHODCALLTYPE Skip(DWORD celt) override;
//     HRESULT STDMETHODCALLTYPE Reset() override;
//
// protected:
//     LONG m_ref;
//     IRecycleBin5 *m_recycleBin;
//     HANDLE m_hInfo;
//     INFO2_HEADER *m_pInfo;
//     DWORD m_dwCurrent;
//     LPWSTR m_pszPrefix;
// };
//
// HRESULT STDMETHODCALLTYPE RecycleBin5Enum::QueryInterface(REFIID riid, void **ppvObject)
// {
//     TRACE("(%p, %s, %p)\n", this, debugstr_guid(&riid), ppvObject);
//
//     if (!ppvObject)
//         return E_POINTER;
//
//     if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_IRecycleBinEnumList))
//         *ppvObject = static_cast<IRecycleBinEnumList *>(this);
//     else
//     {
//         *ppvObject = NULL;
//         return E_NOINTERFACE;
//     }
//
//     AddRef();
//     return S_OK;
// }
//
// STDMETHODIMP_(ULONG) RecycleBin5Enum::AddRef()
// {
//     TRACE("(%p)\n", this);
//     return InterlockedIncrement(&m_ref);
// }
//
// RecycleBin5Enum::~RecycleBin5Enum()
// {
//     TRACE("(%p)\n", this);
//
//     m_recycleBin->OnClosing(this);
//     UnmapViewOfFile(m_pInfo);
//     m_recycleBin->Release();
//     SHFree(m_pszPrefix);
// }
//
// STDMETHODIMP_(ULONG) RecycleBin5Enum::Release()
// {
//     TRACE("(%p)\n", this);
//
//     ULONG refCount = InterlockedDecrement(&m_ref);
//     if (refCount == 0)
//         delete this;
//     return refCount;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5Enum::Next(DWORD celt, IRecycleBinFile **rgelt, DWORD *pceltFetched)
// {
//     HRESULT hr;
//
//     TRACE("(%p, %u, %p, %p)\n", this, celt, rgelt, pceltFetched);
//
//     if (!rgelt)
//         return E_POINTER;
//     if (!pceltFetched && celt > 1)
//         return E_INVALIDARG;
//
//     ULARGE_INTEGER FileSize;
//     FileSize.u.LowPart = GetFileSize(m_hInfo, &FileSize.u.HighPart);
//     if (FileSize.u.LowPart == 0)
//         return HResultFromWin32(GetLastError());
//
//     DWORD dwEntries =
//         (DWORD)((FileSize.QuadPart - sizeof(INFO2_HEADER)) / sizeof(DELETED_FILE_RECORD));
//
//     DWORD iEntry = m_dwCurrent, fetched = 0;
//     PDELETED_FILE_RECORD pDeletedFile = (PDELETED_FILE_RECORD)(m_pInfo + 1) + iEntry;
//     for (; iEntry < dwEntries && fetched < celt; ++iEntry)
//     {
//         hr = RecycleBin5File_Constructor(m_recycleBin, m_pszPrefix, pDeletedFile, &rgelt[fetched]);
//         if (SUCCEEDED(hr))
//             fetched++;
//         pDeletedFile++;
//     }
//
//     m_dwCurrent = iEntry;
//     if (pceltFetched)
//         *pceltFetched = fetched;
//     if (fetched == celt)
//         return S_OK;
//     else
//         return S_FALSE;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5Enum::Skip(DWORD celt)
// {
//     TRACE("(%p, %u)\n", this, celt);
//     m_dwCurrent += celt;
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5Enum::Reset()
// {
//     TRACE("(%p)\n", this);
//     m_dwCurrent = 0;
//     return S_OK;
// }
//
// RecycleBin5Enum::RecycleBin5Enum()
//     : m_ref(1)
//     , m_recycleBin(NULL)
//     , m_hInfo(NULL)
//     , m_pInfo(NULL)
//     , m_dwCurrent(0)
//     , m_pszPrefix(NULL)
// {
// }
//
// HRESULT
// RecycleBin5Enum::Init(
//      IRecycleBin5 *prb,
//      HANDLE hInfo,
//      HANDLE hInfoMapped,
//      LPCWSTR pszPrefix)
// {
//     m_recycleBin = prb;
//     m_recycleBin->AddRef();
//
//     HRESULT hr = SHStrDup(pszPrefix, &m_pszPrefix);
//     if (FAILED(hr))
//         return hr;
//
//     m_hInfo = hInfo;
//     m_pInfo = (PINFO2_HEADER)MapViewOfFile(hInfoMapped, FILE_MAP_READ, 0, 0, 0);
//     if (!m_pInfo)
//         return HResultFromWin32(GetLastError());
//
//     if (m_pInfo->dwVersion != 5 || m_pInfo->dwRecordSize != sizeof(DELETED_FILE_RECORD))
//         return E_FAIL;
//
//     return S_OK;
// }
//
// extern "C"
// HRESULT
// RecycleBin5Enum_Constructor(
//      IRecycleBin5 *prb,
//      HANDLE hInfo,
//      HANDLE hInfoMapped,
//      LPCWSTR szPrefix,
//      IUnknown **ppUnknown)
// {
//     if (!ppUnknown)
//         return E_POINTER;
//
//     *ppUnknown = NULL;
//
//     RecycleBin5Enum *pThis = new RecycleBin5Enum();
//     if (!pThis)
//         return E_OUTOFMEMORY;
//
//     HRESULT hr = pThis->Init(prb, hInfo, hInfoMapped, szPrefix);
//     if (FAILED(hr))
//     {
//         delete pThis;
//         return hr;
//     }
//
//     *ppUnknown = static_cast<IRecycleBinEnumList *>(pThis);
//     return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
