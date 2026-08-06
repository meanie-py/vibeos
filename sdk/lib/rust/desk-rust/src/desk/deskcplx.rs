//! `sdk/include/reactos/dll/desk/deskcplx.h`'s `static __inline` helpers.
//!
//! The header is the desk.cpl <-> display-driver shell-extension protocol: a shell extension
//! hands desk.cpl an `IDataObject` and desk.cpl pulls named clipboard formats out of it. Because
//! every helper is `static __inline`, no object file in the tree defines one — advmon.c inlined
//! its own copy, and flipping advmon.c took the last one with it. Ported here rather than
//! re-homed as a wrapper in a .c, so the next flip cannot repeat it (same call as syssetup's
//! strsafe and shell32's shellutils.h inlines).
//!
//! Only `QueryDeskCplString` is ported: it is the one helper the module itself calls. The other
//! two (`QueryDeskCplExtInterface`, `DeskCplExtDisplaySaveSettings`) exist for the shell
//! extensions on the far side of the protocol, which are separate C modules that still inline
//! their own copies from the header — porting them here would be dead code that drifts.
use crate::prelude::*;

/// deskcplx.h `QueryDeskCplString` — fetch one clipboard-format string out of the shell
/// extension's data object. Returns a `LocalAlloc`'d copy the caller frees, or NULL.
///
/// The `#ifdef UNICODE` branch is the live one (desk builds with -DUNICODE), so the copy is a
/// plain `wcscpy` and not a `WideCharToMultiByte` — the header's comment says it outright: "We
/// always receive the string in unicode!".
pub unsafe fn QueryDeskCplString(pdo: *mut IDataObject, cfFormat: UINT) -> LPTSTR {
    let mut fetc: FORMATETC = core::mem::zeroed();
    let mut medium: STGMEDIUM = core::mem::zeroed();
    let mut lpStr: LPTSTR = core::ptr::null_mut();

    fetc.cfFormat = cfFormat as CLIPFORMAT;
    fetc.ptd = core::ptr::null_mut();
    fetc.dwAspect = DVASPECT_CONTENT as DWORD;
    fetc.lindex = -1;
    fetc.tymed = TYMED_HGLOBAL as DWORD;

    if SUCCEEDED!(IDataObject_GetData(pdo, &mut fetc, &mut medium))
        && !medium.__bindgen_anon_1.hGlobal.is_null()
    {
        let lpRecvBuffer = GlobalLock(medium.__bindgen_anon_1.hGlobal) as LPWSTR;
        let BufLen: SIZE_T = (wcslen(lpRecvBuffer) + 1) as SIZE_T;
        lpStr = LocalAlloc(
            LMEM_FIXED,
            (BufLen as usize * core::mem::size_of::<TCHAR>()) as SIZE_T,
        ) as LPTSTR;
        if !lpStr.is_null() {
            wcscpy(lpStr, lpRecvBuffer);
        }
        GlobalUnlock(medium.__bindgen_anon_1.hGlobal);
        ReleaseStgMedium(&mut medium);
    }

    lpStr
}
