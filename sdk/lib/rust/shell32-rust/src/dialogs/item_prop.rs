//! `dll/win32/shell32/dialogs/item_prop.cpp` — the filesystem property-sheet entry points.
//!
//! Nine free functions came from the paid run in
//! `.responses/dialogs__item_prop__factseal_v1__callseal_v1/`. The four
//! `ShellPropSheetDialog` statics are hand-written from the C: that `struct` is a namespace,
//! not a COM class — no vtable, no `BEGIN_COM_MAP`, no activation — so `--cpp-com` refuses it
//! by design and `--free-functions` skips it. They are spelled here as free functions under
//! the names rustconv itself listed them by.
//!
//! `crate::prelude` (island) world, because the spliced `CDataObjectHIDA` lowering reimplements
//! `CreateCIDA` against the island's `STGMEDIUM`/`FORMATETC`. `HKEY` is imported explicitly
//! from `crate::types` — an explicit `use` shadows the glob — so the key-array plumbing here
//! agrees with `crate::shfldr::CRegKeyHandleArray` and with `propsheet.rs`'s `SHOpenPropSheetW`.

use crate::atl::{CComHeapPtr, CComPtr};
// The crate's own functions: the definition is the declaration.
use crate::shell32::shldataobject::SHELL_DataObject_ILCloneFullItem;
use crate::wine::shellpath::SHGetSpecialFolderLocation;
// One declaration per external symbol: island.rs.
use crate::island::SHCreateThread;
use crate::prelude::*;
use crate::shfldr::CRegKeyHandleArray;
use crate::types::HKEY;
use core::ffi::c_void;

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_09FFC547AFB5: [WCHAR; 6] = [0x002C, 0x0020, 0x002E, 0x002E, 0x002E, 0x0000];
pub static CPP_COM_WSTR_32491EF24F87: [WCHAR; 6] = [0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0000];
pub static CPP_COM_WSTR_6E342AB1A0F1: [WCHAR; 7] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0000];

/// `"properties"` — the ANSI verb handed to `ShellExecuteExA`. Narrow on purpose: the C++ uses
/// `SHELLEXECUTEINFOA`, and widening it would change which export runs.
static VERB_PROPERTIES: [u8; 11] = *b"properties\0";

// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
// Hand-verified Rust lowering of the C++ RAII struct `CDataObjectHIDA`
// (sdk/include/reactos/shellutils.h:738). Consumed by
// `rustconv translate --free-functions --local-raii CDataObjectHIDA=<this file>`:
// spliced verbatim into the assembled module, and its API sealed into each
// affected worker prompt so the model maps C++ usage onto this fixed surface
// instead of inventing the drop semantics.
//
// It borrows the HIDA out of an IDataObject WITHOUT copying it, so the medium
// must stay locked for exactly the lifetime of the guard — which is the whole
// reason it is RAII and the whole reason a hand lowering is required.
//
// C++ semantics preserved exactly:
//   ctor        — CreateCIDA(pDataObject, &m_cida, m_medium); m_hr = result
//   dtor        — DestroyCIDA(m_cida, m_medium)                → impl Drop
//   hr()        — the stored ctor HRESULT
//   operator bool()        → is_valid()      (m_cida != nullptr)
//   operator const CIDA*() → as_ptr()        (the implicit conversion at a call
//                                             site such as `f(x, cida)`)
//   operator->()           → (*cida.as_ptr()).field
//
// Two details that look like bugs and are not, so do not "fix" them:
//   * DestroyCIDA calls ReleaseStgMedium unconditionally. On the failure paths
//     CreateCIDA has already set tymed = TYMED_NULL, which makes that call a
//     documented no-op — including after CreateCIDA itself released the medium.
//   * C++ leaves m_medium uninitialized and only assigns pUnkForRelease. Every
//     path assigns tymed before anything reads it (GetData on success,
//     TYMED_NULL otherwise). Zeroing here is strictly safer and changes no path.
struct CDataObjectHIDA {
    m_medium: STGMEDIUM,
    m_cida: *const CIDA,
    m_hr: HRESULT,
}

impl CDataObjectHIDA {
    unsafe fn new(pDataObject: *mut IDataObject) -> CDataObjectHIDA {
        let mut medium: STGMEDIUM = core::mem::zeroed();
        let mut cida: *const CIDA = core::ptr::null();
        let hr = CDataObjectHIDA::CreateCIDA(pDataObject, &mut cida, &mut medium);
        CDataObjectHIDA { m_medium: medium, m_cida: cida, m_hr: hr }
    }

    unsafe fn CreateCIDA(
        pDataObject: *mut IDataObject,
        ppcida: *mut *const CIDA,
        medium: *mut STGMEDIUM,
    ) -> HRESULT {
        *ppcida = core::ptr::null();
        (*medium).pUnkForRelease = core::ptr::null_mut();
        // g_cfHIDA in C++ is a DECLSPEC_SELECTANY cache of the registered format.
        let mut format = FORMATETC {
            cfFormat: hida_clipformat(),
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        let hr = ((*(*pDataObject).lpVtbl).GetData)(
            pDataObject,
            core::ptr::addr_of_mut!(format).cast(),
            medium.cast(),
        );
        if hr >= 0 {
            *ppcida = GlobalLock((*medium).__bindgen_anon_1.hGlobal) as *const CIDA;
            if !(*ppcida).is_null() {
                return S_OK;
            }
            ReleaseStgMedium(medium);
            (*medium).tymed = TYMED_NULL;
            return E_UNEXPECTED;
        }
        (*medium).tymed = TYMED_NULL;
        hr
    }

    /// Mirrors the C++ public static. It is not only the destructor's body:
    /// dll/shellext/cabview/folder.cpp calls CreateCIDA/DestroyCIDA directly to
    /// hold a CIDA across a scope the guard cannot span, so it must stay callable.
    unsafe fn DestroyCIDA(pcida: *const CIDA, medium: *mut STGMEDIUM) {
        if !pcida.is_null() {
            GlobalUnlock((*medium).__bindgen_anon_1.hGlobal);
        }
        ReleaseStgMedium(medium);
    }

    unsafe fn hr(&self) -> HRESULT {
        self.m_hr
    }

    unsafe fn as_ptr(&self) -> *const CIDA {
        self.m_cida
    }
}

impl Drop for CDataObjectHIDA {
    fn drop(&mut self) {
        unsafe { CDataObjectHIDA::DestroyCIDA(self.m_cida, &mut self.m_medium) }
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// Substrate this TU reaches that the prelude does not already carry
// ─────────────────────────────────────────────────────────────────────────────────────────

/// `objbase.h` `COWAIT_FLAGS` — only the default is used here.
const COWAIT_DEFAULT: DWORD = 0;

/// `shobjidl.idl:162` `SHGNO::SHGDN_INFOLDER`.
const SHGDN_INFOLDER: DWORD = 1;

// The island already declares these against the SDK, so they are IMPORTED rather than
// restated: a local `extern` block is a hand-written ABI claim, and the island's is generated
// from the headers. Only what the island genuinely lacks is declared below.
use crate::island::{
    CloseHandle, CoGetInterfaceAndReleaseStream, CoMarshalInterThreadInterfaceInStream,
    CoWaitForMultipleHandles, CreateEventW, GetLastError, SetEvent, SHStrDupW, StringFromGUID2,
};

extern "system" {
    /// `shlobj.h` `SHOpenPropSheetW` — shell32's own export, implemented in `crate::propsheet`,
    /// which is `crate::types` world; hence the explicit `HKEY` import at the top of this
    /// module. `pShellBrowser` is opaque because every call site in this TU passes NULL and
    /// `IShellBrowser` lives in the other world.
    fn SHOpenPropSheetW(
        pszCaption: LPCWSTR,
        ahKeys: *mut HKEY,
        cKeys: UINT,
        pclsidDefault: *const CLSID,
        pDataObject: *mut IDataObject,
        pShellBrowser: *mut c_void,
        pszStartPage: LPCWSTR,
    ) -> BOOL;

    /// `shlobj_undoc.h`. The island's declaration takes an `LPTHREAD_START_ROUTINE` from
    /// ntdll-sys, which has NOT been through `normalize_island_abi.py` — three spellings of one
    /// ABI but three distinct Rust fn-pointer types. Redeclared against `extern "system"` for
    /// the same reason `dialogs/drive.rs` does; the ABI is unchanged, only how it is written.

    fn ShellExecuteExA(sei: *mut SHELLEXECUTEINFOA) -> BOOL;
}

extern "C" {
    /// `precomp.h:326-329` — `EXTERN_C`, so cdecl. Implemented in `shell32/shldataobject.rs`.
    fn DataObject_GetHIDACount(pdo: *mut IDataObject) -> HRESULT;
    fn SHELL_CloneDataObject(pDO: *mut IDataObject, ppDO: *mut *mut IDataObject) -> HRESULT;

    /// `shfldr.h:185/187` — `EXTERN_C`, so cdecl. Implemented in `shell32/shlfolder.rs`.
    /// `HKEY` here is `crate::types::HKEY`, matching `CRegKeyHandleArray`.
    fn AddClassKeyToArray(szClass: LPCWSTR, array: *mut HKEY, cKeys: *mut UINT) -> LONG;
    fn AddFSClassKeysToArray(
        cidl: UINT,
        apidl: PCUITEMID_CHILD_ARRAY,
        array: *mut HKEY,
        cKeys: *mut UINT,
    );

    /// `shlguid_undoc.h` — the two default property-sheet extension CLSIDs.
    static CLSID_ShellDrvDefExt: CLSID;
    static CLSID_ShellFileDefExt: CLSID;
    static IID_IDataObject: IID;
}

/// `minwindef.h` `IS_INTRESOURCE` — true when the whole value fits in the low 16 bits, i.e. it
/// is a CSIDL rather than a real PIDL pointer.
#[inline]
fn IS_INTRESOURCE(value: LPCITEMIDLIST) -> bool {
    (value as usize) >> 16 == 0
}

/// `shellutils.h` `HResultFromWin32`.
#[inline]
fn HResultFromWin32(err: DWORD) -> HRESULT {
    HRESULT_FROM_WIN32(err as LONG)
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// ShellPropSheetDialog — a C++ `struct` used as a namespace: only static members, no state,
// no vtable. Hand-written from the C; see the module header for why it is not in the paid set.
// ─────────────────────────────────────────────────────────────────────────────────────────

/// `typedef void (CALLBACK *PFNINITIALIZE)(LPCWSTR, IDataObject*, HKEY*, UINT*)`.
///
/// `CALLBACK` is `__stdcall`. It MUST survive onto the two callbacks below: they are stored in
/// this slot and called back through it, and on i386 a Rust-convention `fn` and a stdcall `fn`
/// disagree about who pops the arguments.
pub type PFNINITIALIZE = Option<
    unsafe extern "system" fn(
        InitString: LPCWSTR,
        pDO: *mut IDataObject,
        hKeys: *mut HKEY,
        cKeys: *mut UINT,
    ),
>;

/// `ShellPropSheetDialog::DATA` — the heap block handed to the worker thread.
#[repr(C)]
pub struct ShellPropSheetDialog_DATA {
    pub InitFunc: PFNINITIALIZE,
    pub InitString: LPWSTR,
    pub ClsidDefault: CLSID,
    pub pClsidDefault: *const CLSID,
    pub pObjStream: *mut IStream,
    pub hEvent: HANDLE,
}

/// `ShellPropSheetDialog::Show`.
///
/// Two of C++'s conversion operators fire here, and BOTH are cases where one object supplies
/// two different argument types in a single call — `InitFunc(InitString, pDO, keys, keys)` is
/// `HKEY*` then `UINT*`, and `SHOpenPropSheetW(.., keys, keys, ..)` is `HKEY*` then `UINT`.
/// Written out, the two arguments cannot be the same expression.
unsafe fn ShellPropSheetDialog_Show(
    pClsidDefault: *const CLSID,
    pDO: *mut IDataObject,
    InitFunc: PFNINITIALIZE,
    InitString: LPCWSTR,
) -> HRESULT {
    // Declared before the SHOpenPropSheetW call that borrows it, so its Drop (CloseRegKeyArray)
    // runs after that call returns — exactly as the C++ scope exit did.
    let mut keys = CRegKeyHandleArray::default();
    if let Some(pfn) = InitFunc {
        pfn(InitString, pDO, keys.hKeys.as_mut_ptr(), &mut keys.cKeys);
    }
    let mut szCaption: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut pszCaption: LPCWSTR = core::ptr::null();
    if SHELL_GetCaptionFromDataObject(pDO, szCaption.as_mut_ptr(), MAX_PATH as UINT) >= 0 {
        pszCaption = szCaption.as_ptr();
    }
    if SHOpenPropSheetW(
        pszCaption,
        keys.hKeys.as_mut_ptr(),
        keys.cKeys,
        pClsidDefault,
        pDO,
        core::ptr::null_mut(),
        core::ptr::null(),
    ) != 0
    {
        S_OK
    } else {
        E_FAIL
    }
}

/// `ShellPropSheetDialog::FreeData`.
unsafe fn ShellPropSheetDialog_FreeData(pData: *mut ShellPropSheetDialog_DATA) {
    if !(*pData).InitString.is_null() {
        SHFree((*pData).InitString as LPVOID);
    }
    SHFree(pData as LPVOID);
}

/// `ShellPropSheetDialog::ShowAsync`.
unsafe fn ShellPropSheetDialog_ShowAsync(
    pClsidDefault: *const CLSID,
    pDO: *mut IDataObject,
    InitFunc: PFNINITIALIZE,
    InitString: LPCWSTR,
) -> HRESULT {
    let pData = SHAlloc(core::mem::size_of::<ShellPropSheetDialog_DATA>() as SIZE_T)
        as *mut ShellPropSheetDialog_DATA;
    if pData.is_null() {
        return E_OUTOFMEMORY;
    }
    // C `ZeroMemory(pData, sizeof(*pData))`. The count is ELEMENTS, not bytes: `write_bytes`
    // multiplies by size_of::<T>() itself.
    core::ptr::write_bytes(pData, 0, 1);
    (*pData).InitFunc = InitFunc;
    if !InitString.is_null()
        && SHStrDupW(InitString, core::ptr::addr_of_mut!((*pData).InitString)) < 0
    {
        ShellPropSheetDialog_FreeData(pData);
        return E_OUTOFMEMORY;
    }
    if !pClsidDefault.is_null() {
        (*pData).ClsidDefault = *pClsidDefault;
        (*pData).pClsidDefault = core::ptr::addr_of!((*pData).ClsidDefault);
    }
    // C++ keeps its own copy of the handle because pData is freed by the worker thread.
    let mut hEvent = CreateEventW(core::ptr::null_mut(), TRUE, FALSE, core::ptr::null());
    (*pData).hEvent = hEvent;

    let mut hr: HRESULT = S_OK;
    if !pDO.is_null() {
        // The island's ole32 declarations are typed against com-sys; `IID`/`IStream`/
        // `IUnknown` here resolve to the ntdll-sys aliases of the same layouts, so the three
        // pointers are cast rather than the declaration being restated.
        hr = CoMarshalInterThreadInterfaceInStream(
            core::ptr::addr_of!(IID_IDataObject).cast(),
            pDO.cast(),
            core::ptr::addr_of_mut!((*pData).pObjStream).cast(),
        );
    }

    let flags: DWORD = (CTF_COINIT | CTF_PROCESS_REF | CTF_INSIST) as DWORD;
    if hr >= 0
        && SHCreateThread(
            Some(ShellPropSheetDialog_ShowPropertiesThread),
            pData as *mut c_void,
            flags,
            None,
        ) == 0
    {
        if !(*pData).pObjStream.is_null() {
            let pStm = (*pData).pObjStream as *mut IUnknown;
            ((*(*pStm).lpVtbl).Release)(pStm);
        }
        hr = E_FAIL;
    }

    if hr >= 0 {
        if !hEvent.is_null() {
            let mut index: DWORD = 0;
            // Pump COM messages until the thread can create its own IDataObject (for CORE-19933).
            // SHOpenPropSheetW is modal and we cannot wait for it to complete.
            CoWaitForMultipleHandles(
                COWAIT_DEFAULT,
                INFINITE,
                1,
                core::ptr::addr_of_mut!(hEvent),
                &mut index,
            );
            CloseHandle(hEvent);
        }
    } else {
        ShellPropSheetDialog_FreeData(pData);
    }
    hr
}

/// `ShellPropSheetDialog::ShowPropertiesThread` — `DWORD CALLBACK`, handed to `SHCreateThread`.
unsafe extern "system" fn ShellPropSheetDialog_ShowPropertiesThread(Param: *mut c_void) -> DWORD {
    let pData = Param as *mut ShellPropSheetDialog_DATA;
    // C++ `CComPtr<IDataObject> pDO, pLocalDO;` — both Release on scope exit.
    let mut pDO: CComPtr<IDataObject> = CComPtr::new();
    let mut pLocalDO: CComPtr<IDataObject> = CComPtr::new();
    if !(*pData).pObjStream.is_null() {
        CoGetInterfaceAndReleaseStream(
            (*pData).pObjStream.cast(),
            core::ptr::addr_of!(IID_IDataObject).cast(),
            pDO.GetAddressOf().cast(),
        );
    }
    if !pDO.is_null() && SHELL_CloneDataObject(pDO.Get(), pLocalDO.GetAddressOf()) >= 0 {
        // C++ `pDO = pLocalDO;` is CComPtr::operator=, which AddRefs the incoming pointer and
        // THEN Releases the outgoing one. Both smart pointers end up holding a reference to the
        // clone and both release it at scope exit; Attach() alone would drop one of the two.
        let u = pLocalDO.Get() as *mut IUnknown;
        ((*(*u).lpVtbl).AddRef)(u);
        pDO.Attach(pLocalDO.Get());
    }
    if !(*pData).hEvent.is_null() {
        SetEvent((*pData).hEvent);
    }
    ShellPropSheetDialog_Show(
        (*pData).pClsidDefault,
        pDO.Get(),
        (*pData).InitFunc,
        (*pData).InitString,
    );
    ShellPropSheetDialog_FreeData(pData);
    0
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// Free functions (translated)
// ─────────────────────────────────────────────────────────────────────────────────────────

unsafe fn SHELL_GetCaptionFromDataObject(
    pDO: *mut IDataObject,
    Buf: LPWSTR,
    cchBuf: UINT,
) -> HRESULT {
    let mut hr: HRESULT = E_INVALIDARG;
    let pidl: PIDLIST_ABSOLUTE = SHELL_DataObject_ILCloneFullItem(pDO, 0);
    if !pidl.is_null() {
        hr = SHGetNameAndFlagsW(pidl, SHGDN_INFOLDER, Buf, cchBuf, core::ptr::null_mut());
        ILFree(pidl);

        if hr >= 0 && DataObject_GetHIDACount(pDO) > 1 {
            StringCchCatW(Buf, cchBuf as usize, CPP_COM_WSTR_09FFC547AFB5.as_ptr());
        }
    }
    hr
}

/// `static void CALLBACK` — stdcall, stored in a `PFNINITIALIZE` slot.
unsafe extern "system" fn FSFolderItemPropDialogInitCallback(
    InitString: LPCWSTR,
    pDO: *mut IDataObject,
    hKeys: *mut HKEY,
    cKeys: *mut UINT,
) {
    UNREFERENCED_PARAMETER!(InitString);
    let cida = CDataObjectHIDA::new(pDO);
    if cida.hr() >= 0 && (*cida.as_ptr()).cidl != 0 {
        // Note: Since we are only passing a single item to AddFSClassKeysToArray,
        // we don't need the rest of the array to be valid.
        let pidl: PCUITEMID_CHILD = HIDA_GetPIDLItem(cida.as_ptr(), 0);
        AddFSClassKeysToArray(1, &pidl, hKeys, cKeys); // Add file-type specific pages
    }
}

/// `static void CALLBACK` — stdcall, stored in a `PFNINITIALIZE` slot.
unsafe extern "system" fn ClassPropDialogInitCallback(
    InitString: LPCWSTR,
    pDO: *mut IDataObject,
    hKeys: *mut HKEY,
    cKeys: *mut UINT,
) {
    UNREFERENCED_PARAMETER!(pDO);
    // Add pages from HKCR\%ProgId% (with shellex\PropertySheetHandlers appended later)
    AddClassKeyToArray(InitString, hKeys, cKeys);
}

#[no_mangle]
pub unsafe extern "C" fn SHELL32_ShowFilesystemItemPropertiesDialogAsync(
    pDO: *mut IDataObject,
) -> HRESULT {
    let cida = CDataObjectHIDA::new(pDO);
    let hr = cida.hr();
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let pClsid: *const CLSID;
    let InitFunc: PFNINITIALIZE;
    let mut InitString: LPCWSTR = core::ptr::null();

    if (*cida.as_ptr()).cidl != 0 && _ILIsDrive(HIDA_GetPIDLItem(cida.as_ptr(), 0)) != 0 {
        pClsid = core::ptr::addr_of!(CLSID_ShellDrvDefExt);
        InitFunc = Some(ClassPropDialogInitCallback);
        InitString = CPP_COM_WSTR_32491EF24F87.as_ptr();
    } else {
        pClsid = core::ptr::addr_of!(CLSID_ShellFileDefExt);
        InitFunc = Some(FSFolderItemPropDialogInitCallback);
    }
    ShellPropSheetDialog_ShowAsync(pClsid, pDO, InitFunc, InitString)
}

#[no_mangle]
pub unsafe extern "C" fn SHELL32_ShowFilesystemItemsPropertiesDialogAsync(
    hOwner: HWND,
    pDO: *mut IDataObject,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(hOwner);
    if DataObject_GetHIDACount(pDO) == 1 {
        return SHELL32_ShowFilesystemItemPropertiesDialogAsync(pDO);
    }
    SHMultiFileProperties(pDO, 0)
}

#[no_mangle]
pub unsafe extern "C" fn SHELL32_ShowShellExtensionProperties(
    pClsid: *const CLSID,
    pDO: *mut IDataObject,
) -> HRESULT {
    // C++ `WCHAR ClassBuf[6 + 38 + 1] = L"CLSID\\";` — aggregate init copies the 6 characters
    // and ZERO-FILLS the rest, which is what leaves room for StringFromGUID2 to write at +6.
    let mut ClassBuf: [WCHAR; 6 + 38 + 1] = [0; 6 + 38 + 1];
    ClassBuf[..CPP_COM_WSTR_6E342AB1A0F1.len()].copy_from_slice(&CPP_COM_WSTR_6E342AB1A0F1);
    StringFromGUID2(pClsid, ClassBuf.as_mut_ptr().add(6), 38 + 1);
    ShellPropSheetDialog_ShowAsync(
        core::ptr::null(),
        pDO,
        Some(ClassPropDialogInitCallback),
        ClassBuf.as_ptr(),
    )
}

#[no_mangle]
pub unsafe extern "C" fn SHELL_ShowItemIDListProperties(mut pidl: LPCITEMIDLIST) -> HRESULT {
    debug_assert!(!pidl.is_null());

    // C++ `CComHeapPtr<ITEMIDLIST> alloc;` — frees through the COM task allocator on scope
    // exit. It only ever owns the PIDL that SHGetSpecialFolderLocation allocated; on the
    // ordinary path the caller still owns `pidl` and `alloc` stays empty.
    let mut alloc: CComHeapPtr<ITEMIDLIST> = CComHeapPtr::new();
    if IS_INTRESOURCE(pidl) {
        let hr = SHGetSpecialFolderLocation(
            core::ptr::null_mut(),
            LOWORD(pidl as usize as DWORD) as INT,
            core::ptr::addr_of_mut!(pidl) as *mut LPITEMIDLIST,
        );
        if hr < 0 {
            return hr;
        }
        alloc.Attach(pidl as LPITEMIDLIST);
    }
    // C++ aggregate init, positionally: cbSize, fMask, hwnd, lpVerb, lpFile, lpParameters,
    // lpDirectory, nShow, hInstApp, lpIDList. Written by name so the mapping is checkable.
    let mut sei: SHELLEXECUTEINFOA = core::mem::zeroed();
    sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOA>() as DWORD;
    sei.fMask = SEE_MASK_INVOKEIDLIST | SEE_MASK_ASYNCOK;
    sei.lpVerb = VERB_PROPERTIES.as_ptr() as LPCSTR;
    sei.nShow = SW_SHOWNORMAL as INT;
    sei.lpIDList = pidl as LPITEMIDLIST as LPVOID;
    if ShellExecuteExA(&mut sei) != 0 {
        S_OK
    } else {
        HResultFromWin32(GetLastError())
    }
}

/*
 * SHMultiFileProperties                [SHELL32.716]
 */
#[no_mangle]
pub unsafe extern "system" fn SHMultiFileProperties(
    pDataObject: *mut IDataObject,
    dwFlags: DWORD,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(dwFlags);
    if DataObject_GetHIDACount(pDataObject) == 1 {
        return SHELL32_ShowFilesystemItemPropertiesDialogAsync(pDataObject);
    }

    ShellPropSheetDialog_ShowAsync(
        core::ptr::addr_of!(CLSID_ShellFileDefExt),
        pDataObject,
        None,
        core::ptr::null(),
    )
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     shell32
//  * LICENSE:     LGPL-2.1+ (https://spdx.org/licenses/LGPL-2.1+)
//  * PURPOSE:     FileSystem PropertySheet implementation
//  * COPYRIGHT:   Copyright 2024 Whindmar Saksit <whindsaks@proton.me>
//  */
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// static HRESULT
// SHELL_GetCaptionFromDataObject(IDataObject *pDO, LPWSTR Buf, UINT cchBuf)
// {
//     HRESULT hr = E_INVALIDARG;
//     if (PIDLIST_ABSOLUTE pidl = SHELL_DataObject_ILCloneFullItem(pDO, 0))
//     {
//         hr = SHGetNameAndFlagsW(pidl, SHGDN_INFOLDER, Buf, cchBuf, NULL);
//         ILFree(pidl);
//
//         if (SUCCEEDED(hr) && DataObject_GetHIDACount(pDO) > 1)
//             StringCchCatW(Buf, cchBuf, L", ...");
//     }
//     return hr;
// }
//
// struct ShellPropSheetDialog
// {
//     typedef void (__stdcall*PFNINITIALIZE)(LPCWSTR InitString, IDataObject *pDO,
//                                           HKEY *hKeys, UINT *cKeys);
//
//     static HRESULT Show(const CLSID *pClsidDefault, IDataObject *pDO,
//                         PFNINITIALIZE InitFunc, LPCWSTR InitString)
//     {
//         HRESULT hr;
//         CRegKeyHandleArray keys;
//         if (InitFunc)
//             InitFunc(InitString, pDO, keys, keys);
//         WCHAR szCaption[MAX_PATH], *pszCaption = NULL;
//         if (SUCCEEDED(SHELL_GetCaptionFromDataObject(pDO, szCaption, _countof(szCaption))))
//             pszCaption = szCaption;
//         hr = SHOpenPropSheetW(pszCaption, keys, keys, pClsidDefault, pDO, NULL, NULL) ? S_OK : E_FAIL;
//         return hr;
//     }
//
//     struct DATA
//     {
//         PFNINITIALIZE InitFunc;
//         LPWSTR InitString;
//         CLSID ClsidDefault;
//         const CLSID *pClsidDefault;
//         IStream *pObjStream;
//         HANDLE hEvent;
//     };
//
//     static void FreeData(DATA *pData)
//     {
//         if (pData->InitString)
//             SHFree(pData->InitString);
//         SHFree(pData);
//     }
//
//     static HRESULT ShowAsync(const CLSID *pClsidDefault, IDataObject *pDO,
//                              PFNINITIALIZE InitFunc, LPCWSTR InitString)
//     {
//         DATA *pData = (DATA*)SHAlloc(sizeof(*pData));
//         if (!pData)
//             return E_OUTOFMEMORY;
//         ZeroMemory(pData, sizeof(*pData));
//         pData->InitFunc = InitFunc;
//         if (InitString && FAILED(SHStrDupW(InitString, &pData->InitString)))
//         {
//             FreeData(pData);
//             return E_OUTOFMEMORY;
//         }
//         if (pClsidDefault)
//         {
//             pData->ClsidDefault = *pClsidDefault;
//             pData->pClsidDefault = &pData->ClsidDefault;
//         }
//         HANDLE hEvent = pData->hEvent = CreateEventW(NULL, TRUE, FALSE, NULL);
//
//         HRESULT hr = S_OK;
//         if (pDO)
//             hr = CoMarshalInterThreadInterfaceInStream(IID_IDataObject, pDO, &pData->pObjStream);
//
//         const UINT flags = CTF_COINIT | CTF_PROCESS_REF | CTF_INSIST;
//         if (SUCCEEDED(hr) && !SHCreateThread(ShowPropertiesThread, pData, flags, NULL))
//         {
//             if (pData->pObjStream)
//                 pData->pObjStream->Release();
//             hr = E_FAIL;
//         }
//
//         if (SUCCEEDED(hr))
//         {
//             if (hEvent)
//             {
//                 DWORD index;
//                 // Pump COM messages until the thread can create its own IDataObject (for CORE-19933).
//                 // SHOpenPropSheetW is modal and we cannot wait for it to complete.
//                 CoWaitForMultipleHandles(COWAIT_DEFAULT, INFINITE, 1, &hEvent, &index);
//                 CloseHandle(hEvent);
//             }
//         }
//         else
//         {
//             FreeData(pData);
//         }
//         return hr;
//     }
//
//     static DWORD __stdcall ShowPropertiesThread(LPVOID Param)
//     {
//         DATA *pData = (DATA*)Param;
//         CComPtr<IDataObject> pDO, pLocalDO;
//         if (pData->pObjStream)
//             CoGetInterfaceAndReleaseStream(pData->pObjStream, IID_PPV_ARG(IDataObject, &pDO));
//         if (pDO && SUCCEEDED(SHELL_CloneDataObject(pDO, &pLocalDO)))
//             pDO = pLocalDO;
//         if (pData->hEvent)
//             SetEvent(pData->hEvent);
//         Show(pData->pClsidDefault, pDO, pData->InitFunc, pData->InitString);
//         FreeData(pData);
//         return 0;
//     }
// };
//
// static void __stdcall
// FSFolderItemPropDialogInitCallback(LPCWSTR InitString, IDataObject *pDO, HKEY *hKeys, UINT *cKeys)
// {
//     UNREFERENCED_PARAMETER(InitString);
//     CDataObjectHIDA cida(pDO);
//     if (SUCCEEDED(cida.hr()) && cida->cidl)
//     {
//         // Note: Since we are only passing a single item to AddFSClassKeysToArray,
//         // we don't need the rest of the array to be valid.
//         PCUITEMID_CHILD pidl = HIDA_GetPIDLItem(cida, 0);
//         AddFSClassKeysToArray(1, &pidl, hKeys, cKeys); // Add file-type specific pages
//     }
// }
//
// static void __stdcall
// ClassPropDialogInitCallback(LPCWSTR InitString, IDataObject *pDO, HKEY *hKeys, UINT *cKeys)
// {
//     UNREFERENCED_PARAMETER(pDO);
//     AddClassKeyToArray(InitString, hKeys, cKeys); // Add pages from HKCR\%ProgId% (with shellex\PropertySheetHandlers appended later)
// }
//
// HRESULT
// SHELL32_ShowFilesystemItemPropertiesDialogAsync(IDataObject *pDO)
// {
//     CDataObjectHIDA cida(pDO);
//     HRESULT hr = cida.hr();
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     const CLSID *pClsid = NULL;
//     ShellPropSheetDialog::PFNINITIALIZE InitFunc = NULL;
//     LPCWSTR InitString = NULL;
//
//     if (cida->cidl && _ILIsDrive(HIDA_GetPIDLItem(cida, 0)))
//     {
//         pClsid = &CLSID_ShellDrvDefExt;
//         InitFunc = ClassPropDialogInitCallback;
//         InitString = L"Drive";
//     }
//     else
//     {
//         pClsid = &CLSID_ShellFileDefExt;
//         InitFunc = FSFolderItemPropDialogInitCallback;
//     }
//     return ShellPropSheetDialog().ShowAsync(pClsid, pDO, InitFunc, InitString);
// }
//
// HRESULT
// SHELL32_ShowFilesystemItemsPropertiesDialogAsync(HWND hOwner, IDataObject *pDO)
// {
//     if (DataObject_GetHIDACount(pDO) == 1)
//         return SHELL32_ShowFilesystemItemPropertiesDialogAsync(pDO);
//     return SHMultiFileProperties(pDO, 0);
// }
//
// HRESULT
// SHELL32_ShowShellExtensionProperties(const CLSID *pClsid, IDataObject *pDO)
// {
//     WCHAR ClassBuf[6 + 38 + 1] = L"CLSID\\";
//     StringFromGUID2(*pClsid, ClassBuf + 6, 38 + 1);
//     return ShellPropSheetDialog().ShowAsync(NULL, pDO, ClassPropDialogInitCallback, ClassBuf);
// }
//
// HRESULT
// SHELL_ShowItemIDListProperties(LPCITEMIDLIST pidl)
// {
//     assert(pidl);
//
//     CComHeapPtr<ITEMIDLIST> alloc;
//     if (IS_INTRESOURCE(pidl))
//     {
//         HRESULT hr = SHGetSpecialFolderLocation(NULL, LOWORD(pidl), const_cast<LPITEMIDLIST*>(&pidl));
//         if (FAILED(hr))
//             return hr;
//         alloc.Attach(const_cast<LPITEMIDLIST>(pidl));
//     }
//     SHELLEXECUTEINFOA sei = {
//         sizeof(sei), SEE_MASK_INVOKEIDLIST | SEE_MASK_ASYNCOK, NULL, "properties",
//         NULL, NULL, NULL, SW_SHOWNORMAL, NULL, const_cast<LPITEMIDLIST>(pidl)
//     };
//     return ShellExecuteExA(&sei) ? S_OK : HResultFromWin32(GetLastError());
// }
//
// /*
//  * SHMultiFileProperties                [SHELL32.716]
//  */
// extern "C" HRESULT
// WINAPI
// SHMultiFileProperties(IDataObject *pDataObject, DWORD dwFlags)
// {
//     if (DataObject_GetHIDACount(pDataObject) == 1)
//         return SHELL32_ShowFilesystemItemPropertiesDialogAsync(pDataObject);
//
//     return ShellPropSheetDialog().ShowAsync(&CLSID_ShellFileDefExt, pDataObject, NULL, NULL);
// }
//
// // Rust bridge — shim over SHELL_ShowItemIDListProperties for the Rust-owned
// // CRecycleBin (shell32-rust recycle_bin).
// extern "C" HRESULT WINAPI
// SHELL_ShowItemIDListProperties_RustBridge(LPCITEMIDLIST pidl)
// {
//     return SHELL_ShowItemIDListProperties(pidl);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
