//! The C++ header inlines from `sdk/include/reactos/shellutils.h` and `dll/win32/shell32/utils.h`.
//!
//! These are `static inline` / `inline` helpers that live in HEADERS, not in any `.c`/`.cpp`.
//! A translation unit that calls them therefore never carries their bodies: rustconv translates
//! the TU, and every one of these comes out as an unresolved name. They are shared by at least
//! `shldataobject.cpp`, `CShellItem.cpp`, `COpenWithMenu.cpp` and `wine/pidl.c`, so they belong
//! in one module rather than being re-derived per TU.
//!
//! Everything here is a line-by-line port of the header, checked against it symbol by symbol.
//! Where the C++ used a `DECLSPEC_SELECTANY` cached CLIPFORMAT global, the Rust keeps a `static
//! mut` cache with the same lazy-register-once semantics — including the same benign race the
//! C++ has (two threads may both call RegisterClipboardFormatW, which returns the same atom).

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
pub use crate::wine::shellole::SHAlloc;
// One declaration per external symbol: island.rs.
use crate::island::IUnknown_SetSite;

use core::ffi::c_char;

use crate::island::{
    lstrcpyW, lstrlenW, WideCharToMultiByte, CP_ACP, SEE_CMIC_COMMON_FLAGS, SEE_MASK_CLASSKEY,
    SEE_MASK_CLASSNAME, SEE_MASK_HASLINKNAME, SEE_MASK_HASTITLE,
    GetVersion, GlobalAlloc, GlobalFree, GlobalLock, GlobalSize, GlobalUnlock,
    RegisterClipboardFormatW, BOOL,
    CIDA, CLIPFORMAT, DWORD, FORMATETC, GHND, GUID, HRESULT, LONG, LPCITEMIDLIST, LPITEMIDLIST,
    PCUIDLIST_ABSOLUTE, PCUIDLIST_RELATIVE, POINT, SAFEARRAY, SHDESCRIPTIONID, SIZE_T, STGMEDIUM,
    UINT, ULONG, VARIANT, VARTYPE, WM_DRAWITEM, WM_INITMENUPOPUP, WM_MEASUREITEM,
    WM_MENUCHAR, WM_MENUSELECT,
};
use crate::V_VT;
use crate::prelude::{
    CopyMemory, ILClone, ILFindLastID, SHFree, DVASPECT_CONTENT, E_OUTOFMEMORY, E_UNEXPECTED, S_OK,
    TRUE, TYMED_HGLOBAL, TYMED_NULL,
};
use crate::shellcom::IDataObject;

/// `STG_E_INVALIDHANDLE` — winerror.h. Not in the island's include closure.
pub const STG_E_INVALIDHANDLE: HRESULT = 0x8003_0006u32 as HRESULT;

extern "system" {
    /// ole32. Not in the island's closure because shell32's TUs reach it through ole2.h.
    pub fn ReleaseStgMedium(pmedium: *mut STGMEDIUM);
}

/// `CFSTR_SHELLIDLISTW` = "Shell IDList Array" (shlobj.h CF_IDLIST).
pub const CFSTR_SHELLIDLISTW: [u16; 19] = [
    b'S' as u16, b'h' as u16, b'e' as u16, b'l' as u16, b'l' as u16, b' ' as u16, b'I' as u16,
    b'D' as u16, b'L' as u16, b'i' as u16, b's' as u16, b't' as u16, b' ' as u16, b'A' as u16,
    b'r' as u16, b'r' as u16, b'a' as u16, b'y' as u16, 0,
];

/// `CFSTR_SHELLIDLISTOFFSETW` = "Shell Object Offsets" (shlobj.h CF_OBJECTPOSITIONS).
const CFSTR_SHELLIDLISTOFFSETW: [u16; 21] = [
    b'S' as u16, b'h' as u16, b'e' as u16, b'l' as u16, b'l' as u16, b' ' as u16, b'O' as u16,
    b'b' as u16, b'j' as u16, b'e' as u16, b'c' as u16, b't' as u16, b' ' as u16, b'O' as u16,
    b'f' as u16, b'f' as u16, b's' as u16, b'e' as u16, b't' as u16, b's' as u16, 0,
];

/// `g_cfHIDA` — shellutils.h's DECLSPEC_SELECTANY cache of the registered HIDA format.
static mut G_CF_HIDA: CLIPFORMAT = 0;
/// `g_cfShellIdListOffsets` — the same, for the object-offsets format.
static mut G_CF_SHELLIDLIST_OFFSETS: CLIPFORMAT = 0;

/// The lazily-registered `CFSTR_SHELLIDLISTW` clipboard format.
///
/// The C++ inlines this as `if (g_cfHIDA == NULL) g_cfHIDA = RegisterClipboardFormatW(...)` at
/// each use site; the translation calls it as a helper, so it is one here.
pub unsafe fn hida_clipformat() -> CLIPFORMAT {
    if G_CF_HIDA == 0 {
        G_CF_HIDA = RegisterClipboardFormatW(CFSTR_SHELLIDLISTW.as_ptr()) as CLIPFORMAT;
    }
    G_CF_HIDA
}

/// The lazily-registered `CFSTR_SHELLIDLISTOFFSETW` clipboard format.
pub unsafe fn shellidlist_offsets_clipformat() -> CLIPFORMAT {
    if G_CF_SHELLIDLIST_OFFSETS == 0 {
        G_CF_SHELLIDLIST_OFFSETS =
            RegisterClipboardFormatW(CFSTR_SHELLIDLISTOFFSETW.as_ptr()) as CLIPFORMAT;
    }
    G_CF_SHELLIDLIST_OFFSETS
}

/// `ILIsSingle` — a PIDL with exactly one item ID.
pub unsafe fn ILIsSingle(pidl: LPCITEMIDLIST) -> BOOL {
    (pidl == ILFindLastID(pidl)) as BOOL
}

/// `HIDA_GetPIDLFolder(pida)` — the parent folder PIDL, at `aoffset[0]`.
pub unsafe fn HIDA_GetPIDLFolder(pida: *const CIDA) -> PCUIDLIST_ABSOLUTE {
    base_offset(pida, 0) as PCUIDLIST_ABSOLUTE
}

/// `HIDA_GetPIDLItem(pida, i)` — child item `i`, at `aoffset[i + 1]`.
pub unsafe fn HIDA_GetPIDLItem(pida: *const CIDA, i: SIZE_T) -> PCUIDLIST_RELATIVE {
    base_offset(pida, i as usize + 1) as PCUIDLIST_RELATIVE
}

/// `((LPBYTE)pida) + pida->aoffset[n]`, the shape both HIDA accessors share.
///
/// `aoffset` is a C flexible array member and `CIDA` is `#[repr(C, packed)]`, so this must walk
/// it through `addr_of!` + `.add(n)`. Indexing a bindgen `[UINT; 1]` with `[n]` would
/// bounds-check-panic for every n >= 1, and `.as_ptr()` on the field would take a reference to
/// a packed field, which is unaligned and rejected.
unsafe fn base_offset(pida: *const CIDA, n: usize) -> *const u8 {
    let aoffset = core::ptr::addr_of!((*pida).aoffset) as *const UINT;
    (pida as *const u8).add(core::ptr::read_unaligned(aoffset.add(n)) as usize)
}

/// `CDataObjectHIDA::DestroyCIDA` — unlock the HGLOBAL if it was locked, then release the medium.
pub unsafe fn DestroyCIDA(pcida: *const CIDA, medium: *mut STGMEDIUM) {
    if !pcida.is_null() {
        GlobalUnlock((*medium).__bindgen_anon_1.hGlobal);
    }
    ReleaseStgMedium(medium);
}

/// `DataObject_GetData` — read a whole clipboard-format blob out of an IDataObject.
///
/// Faithful to the header, including that a blob LARGER than the caller's buffer is an
/// `E_OUTOFMEMORY` rather than a truncating copy.
pub unsafe fn DataObject_GetData(
    pDataObject: *mut IDataObject,
    clipformat: CLIPFORMAT,
    pBuffer: *mut c_void,
    dwBufferSize: SIZE_T,
) -> HRESULT {
    let mut fmt = FORMATETC {
        cfFormat: clipformat,
        ptd: core::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT,
        lindex: -1,
        tymed: TYMED_HGLOBAL,
    };
    let mut medium: STGMEDIUM = core::mem::zeroed();
    medium.tymed = TYMED_NULL;

    let mut hr = ((*(*pDataObject).lpVtbl).GetData)(
        pDataObject,
        (&mut fmt as *mut FORMATETC).cast(),
        (&mut medium as *mut STGMEDIUM).cast(),
    );
    if hr >= 0 {
        let blob = GlobalLock(medium.__bindgen_anon_1.hGlobal);
        if !blob.is_null() {
            let size = GlobalSize(medium.__bindgen_anon_1.hGlobal);
            if size <= dwBufferSize {
                CopyMemory(pBuffer, blob, size as usize);
                hr = S_OK;
            } else {
                hr = E_OUTOFMEMORY;
            }
            GlobalUnlock(medium.__bindgen_anon_1.hGlobal);
        } else {
            hr = STG_E_INVALIDHANDLE;
        }
        ReleaseStgMedium(&mut medium);
    }
    hr
}

/// `DataObject_SetData` — hand a blob to an IDataObject as an HGLOBAL medium.
pub unsafe fn DataObject_SetData(
    pDataObject: *mut IDataObject,
    clipformat: CLIPFORMAT,
    pBuffer: *mut c_void,
    dwBufferSize: SIZE_T,
) -> HRESULT {
    let mut medium: STGMEDIUM = core::mem::zeroed();
    medium.tymed = TYMED_HGLOBAL;
    medium.__bindgen_anon_1.hGlobal = GlobalAlloc(GHND, dwBufferSize);
    if medium.__bindgen_anon_1.hGlobal.is_null() {
        return E_OUTOFMEMORY;
    }

    let mut hr = E_UNEXPECTED;
    let blob = GlobalLock(medium.__bindgen_anon_1.hGlobal);
    if !blob.is_null() {
        CopyMemory(blob, pBuffer, dwBufferSize as usize);
        GlobalUnlock(medium.__bindgen_anon_1.hGlobal);

        let mut etc = FORMATETC {
            cfFormat: clipformat,
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        hr = ((*(*pDataObject).lpVtbl).SetData)(
            pDataObject,
            (&mut etc as *mut FORMATETC).cast(),
            (&mut medium as *mut STGMEDIUM).cast(),
            TRUE,
        );
    }

    if hr < 0 {
        GlobalFree(medium.__bindgen_anon_1.hGlobal);
    }
    hr
}

/// `DataObject_GetOffset` — the CF_OBJECTPOSITIONS point, zeroed first as the header does.
pub unsafe fn DataObject_GetOffset(pDataObject: *mut IDataObject, point: *mut POINT) -> HRESULT {
    let cf = shellidlist_offsets_clipformat();
    (*point).x = 0;
    (*point).y = 0;
    DataObject_GetData(
        pDataObject,
        cf,
        point as *mut c_void,
        core::mem::size_of::<POINT>() as SIZE_T,
    )
}

/// `DataObject_SetOffset` — publish the CF_OBJECTPOSITIONS point.
pub unsafe fn DataObject_SetOffset(pDataObject: *mut IDataObject, point: *mut POINT) -> HRESULT {
    let cf = shellidlist_offsets_clipformat();
    DataObject_SetData(
        pDataObject,
        cf,
        point as *mut c_void,
        core::mem::size_of::<POINT>() as SIZE_T,
    )
}

/// `CCidaChildArrayHelper` (utils.h) — a SHAlloc'd array of borrowed child PIDLs.
///
/// The items point INTO the CIDA and share its lifetime; the C++ header says so explicitly.
/// Only the array itself is owned, and the destructor frees only that.
pub struct CCidaChildArrayHelper {
    pub m_hr: HRESULT,
    pub m_array: *mut LPITEMIDLIST,
}

impl CCidaChildArrayHelper {
    pub unsafe fn new(pCida: *const CIDA) -> Self {
        let cidl = (*pCida).cidl;
        let array = SHAlloc(
            (cidl as SIZE_T).wrapping_mul(core::mem::size_of::<LPITEMIDLIST>() as SIZE_T),
        ) as *mut LPITEMIDLIST;
        if array.is_null() {
            return Self { m_hr: E_OUTOFMEMORY, m_array: core::ptr::null_mut() };
        }
        let mut i: UINT = 0;
        while i < cidl {
            *array.add(i as usize) = HIDA_GetPIDLItem(pCida, i as SIZE_T) as LPITEMIDLIST;
            i += 1;
        }
        Self { m_hr: S_OK, m_array: array }
    }

    pub fn hr(&self) -> HRESULT {
        self.m_hr
    }

    pub fn GetItems(&self) -> *mut LPITEMIDLIST {
        self.m_array
    }
}

impl Drop for CCidaChildArrayHelper {
    fn drop(&mut self) {
        unsafe { SHFree(self.m_array as *mut _) };
        self.m_array = core::ptr::null_mut();
    }
}

// -------------------------------------------------------------------------------------------
// `dll/win32/shell32/wine/shell32_main.h` — the property-system inlines.
//
// Same class as everything above (a `static inline` in a header reaches no translation unit),
// but these had been getting away with it: shlfolder.cpp inlined them into ITSELF and exported
// `SHELL_Create*_RustBridge` wrappers, so translated Rust reached them through a C++ hop.
// Flipping shlfolder.cpp took that only definition with it, and the i386 link failed on
// `_SHELL_CreateVariantBuffer@8` / `_SHELL_CreateSHDESCRIPTIONID@12`. Ported here instead of
// re-homing the wrappers in another `.cpp`, so the next flip cannot repeat it.
//
// `#[no_mangle] extern "system"` to match the WINAPI symbols the existing declarations in
// `win32.rs` and the translated folder classes already reference. There is no collision with
// the C++: `static inline` at namespace scope has INTERNAL linkage, so each C++ TU's copy is
// file-local.

/// wtypes.h `VARENUM` — the two the property-system helpers use.
pub const VT_UI1: VARTYPE = 17;
pub const VT_ARRAY: VARTYPE = 0x2000;

extern "system" {
    fn SafeArrayCreateVector(vt: VARTYPE, lLbound: LONG, cElements: ULONG) -> *mut SAFEARRAY;
}

/// `V_ARRAY(pVar)`, over com-sys's anonymous-union chain (`V_VT` is `.n1.n2.vt`, one level up).
#[inline]
unsafe fn v_array(pVar: *mut VARIANT) -> *mut *mut SAFEARRAY {
    core::ptr::addr_of_mut!((*pVar).n1.n2.n3.parray)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_CreateVariantBufferEx(
    pVar: *mut VARIANT,
    cb: UINT,
    vt: VARTYPE,
) -> HRESULT {
    let pSA = SafeArrayCreateVector(vt, 0, cb);
    if !pSA.is_null() {
        V_VT!(pVar) = VT_ARRAY | vt;
        *v_array(pVar) = pSA;
        return S_OK;
    }
    E_OUTOFMEMORY
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_CreateVariantBuffer(pVar: *mut VARIANT, cb: UINT) -> HRESULT {
    SHELL_CreateVariantBufferEx(pVar, cb, VT_UI1)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_CreateSHDESCRIPTIONID(
    pVar: *mut VARIANT,
    Id: DWORD,
    pCLSID: *const GUID,
) -> HRESULT {
    let hr = SHELL_CreateVariantBuffer(pVar, core::mem::size_of::<SHDESCRIPTIONID>() as UINT);
    if hr >= 0 {
        let pDID = (**v_array(pVar)).pvData as *mut SHDESCRIPTIONID;
        (*pDID).dwDescriptionId = Id;
        (*pDID).clsid = *pCLSID.cast();
    }
    hr
}

/// `wine/shell32_main.h:193` — `static __inline BOOL SHELL_OsIsUnicode(void)`. A header inline
/// has INTERNAL linkage, so every C TU that uses it holds its own copy and there is no symbol
/// to import: a Rust caller needs the body ported, and must NOT `#[no_mangle]` it (nothing
/// links to this name, and exporting it would invent a symbol the C side never had).
#[inline]
pub unsafe fn SHELL_OsIsUnicode() -> BOOL {
    // if high-bit of version is 0, we are emulating NT
    (GetVersion() & 0x80000000 == 0) as BOOL
}

/// `shellutils.h:697` — `static inline BOOL SHELL_IsContextMenuMsg(UINT uMsg)`. Header inline,
/// so not `#[no_mangle]`; see `SHELL_OsIsUnicode` above.
#[inline]
pub fn SHELL_IsContextMenuMsg(uMsg: UINT) -> BOOL {
    (uMsg == WM_MEASUREITEM
        || uMsg == WM_DRAWITEM
        || uMsg == WM_INITMENUPOPUP
        || uMsg == WM_MENUSELECT
        || uMsg == WM_MENUCHAR) as BOOL
}

/// `shellutils.h:462` — `template<class P, class R> static HRESULT SHILClone(P, R*)`. A C++
/// TEMPLATE is in no translation unit for the same reason an inline is not, and its two type
/// parameters exist only to launder PIDL type-safety: every instantiation erases to the same
/// pointer, so one monomorphic port covers them all.
#[inline]
pub unsafe fn SHILClone(pidl: LPCITEMIDLIST, ppOut: *mut LPITEMIDLIST) -> HRESULT {
    let r = ILClone(pidl);
    *ppOut = r;
    if r.is_null() { E_OUTOFMEMORY } else { S_OK }
}

/// `compat_undoc.h:5` — the shim block the loader hangs off the PEB.
#[repr(C)]
struct ReactOS_ShimData {
    dwReserved1: [DWORD; 130],
    dwSize: DWORD,
    dwMagic: DWORD,
    dwReserved2: [DWORD; 242],
    dwRosProcessCompatVersion: DWORD,
}

const REACTOS_SHIMDATA_MAGIC: DWORD = 0xAC0D_EDAB;
const REACTOS_COMPATVERSION_UNINITIALIZED: DWORD = 0xffff_fffe;

/// `compat_undoc.h:22` — `static inline DWORD RosGetProcessCompatVersion(VOID)`. Its cache is a
/// FUNCTION-LOCAL static with a constant initialiser, i.e. a hidden global with no guard
/// variable, so a module `static mut` is exactly equivalent.
static mut g_CompatVersion: DWORD = REACTOS_COMPATVERSION_UNINITIALIZED;

#[inline]
pub unsafe fn RosGetProcessCompatVersion() -> DWORD {
    if g_CompatVersion == REACTOS_COMPATVERSION_UNINITIALIZED {
        let pShimData = (*ntdll_sys::nt::NtCurrentPeb()).pShimData as *const ReactOS_ShimData;
        if !pShimData.is_null()
            && (*pShimData).dwMagic == REACTOS_SHIMDATA_MAGIC
            && (*pShimData).dwSize as usize == core::mem::size_of::<ReactOS_ShimData>()
        {
            g_CompatVersion = (*pShimData).dwRosProcessCompatVersion;
        }
    }
    if g_CompatVersion < REACTOS_COMPATVERSION_UNINITIALIZED { g_CompatVersion } else { 0 }
}

/// `compat_undoc.h:39` — the shimmed version if one is set, else the PEB's OS version.
#[inline]
pub unsafe fn RosGetProcessEffectiveVersion() -> UINT {
    let shimVer = RosGetProcessCompatVersion();
    if shimVer != 0 {
        return shimVer;
    }
    let peb = ntdll_sys::nt::NtCurrentPeb();
    (((*peb).OSMajorVersion << 8) | (*peb).OSMinorVersion) as UINT
}

// ── CCoInitBase (shellutils.h:611) ───────────────────────────────────────────────────────
// A C++ RAII *template*, so like the inlines above it exists in no TU and rustconv never sees
// a body — but unlike them it has no name to leave unresolved either: bindgen renders it as
// the opaque `pub type COleInit = u8;`, against which the natural translation of
// `COleInit OleInit;` is `let _ole_init: COleInit;`. That compiles, and runs NEITHER half of
// the RAII pair — no OleInitialize on the way in, no OleUninitialize on the way out. An
// uninitialised Rust binding is never dropped. Port the guard for real and let Drop do it.
//
// `prelude` re-exports this name explicitly so it shadows the island's `u8` alias, the same
// way `CCidaChildArrayHelper` is handled.
unsafe extern "system" {
    fn OleInitialize(pvReserved: *mut c_void) -> HRESULT;
    fn OleUninitialize();
    fn CoInitialize(pvReserved: *mut c_void) -> HRESULT;
    fn CoUninitialize();
}

/// `CCoInitBase<OleInitialize, OleUninitialize>` — ctor initialises OLE, dtor uninitialises it
/// iff the init succeeded. Construct it with `COleInit::new()`; binding it to `_` would drop it
/// immediately, so keep a named binding for the scope you want COM alive in.
pub struct COleInit {
    pub hr: HRESULT,
}

impl COleInit {
    #[inline]
    pub unsafe fn new() -> Self {
        Self { hr: OleInitialize(core::ptr::null_mut()) }
    }
}

impl Drop for COleInit {
    fn drop(&mut self) {
        if self.hr >= 0 {
            unsafe { OleUninitialize() };
        }
    }
}

/// `CCoInitBase<CoInitialize, CoUninitialize>` (shellutils.h:622) — the other instantiation of
/// the same template, and it carries the identical trap: the island's opaque `pub type CCoInit
/// = u8` swallows `CCoInit init;` into a binding that initialises nothing and drops nothing.
pub struct CCoInit {
    pub hr: HRESULT,
}

impl CCoInit {
    #[inline]
    pub unsafe fn new() -> Self {
        Self { hr: CoInitialize(core::ptr::null_mut()) }
    }
}

impl Drop for CCoInit {
    fn drop(&mut self) {
        if self.hr >= 0 {
            unsafe { CoUninitialize() };
        }
    }
}

/// `shellutils.h:689` — narrows a `SEE_MASK_*` set to the `CMIC_MASK_*` bits an
/// `IContextMenu::InvokeCommand` accepts. A header inline, so it reaches no TU.
#[inline]
pub fn SeeFlagsToCmicFlags(mut flags: UINT) -> UINT {
    if (flags & (SEE_MASK_CLASSNAME | SEE_MASK_CLASSKEY)) == SEE_MASK_CLASSNAME {
        flags &= !(SEE_MASK_HASLINKNAME | SEE_MASK_HASTITLE);
    }
    flags & SEE_CMIC_COMMON_FLAGS
}

// ── wine/shell32_main.h inlines ──────────────────────────────────────────────────────────
// `static __inline` in a Wine header: in no TU, so rustconv never sees a body. Both allocate
// with SHAlloc (the COM task allocator), and the caller frees with SHFree.

/// `wine/shell32_main.h:210` — SHAlloc a CP_ACP copy of a wide string into `*target`.
#[inline]
pub unsafe fn __SHCloneStrWtoA(target: *mut *mut c_char, source: *const u16) {
    let len = WideCharToMultiByte(CP_ACP, 0, source, -1, core::ptr::null_mut(), 0,
                                  core::ptr::null(), core::ptr::null_mut());
    *target = SHAlloc(len as SIZE_T) as *mut c_char;
    WideCharToMultiByte(CP_ACP, 0, source, -1, *target, len,
                        core::ptr::null(), core::ptr::null_mut());
}

/// `wine/shell32_main.h:217` — SHAlloc a copy of a wide string into `*target`.
#[inline]
pub unsafe fn __SHCloneStrW(target: *mut *mut u16, source: *const u16) {
    let cch = lstrlenW(source) + 1;
    *target = SHAlloc(cch as SIZE_T * core::mem::size_of::<u16>() as SIZE_T) as *mut u16;
    lstrcpyW(*target, source);
}

/// `CScopedSetObjectWithSite` — shellutils.h:657.
///
/// Sets a site on an object for a scope and clears it on the way out. The C++ is a
/// three-line RAII struct, and the one detail that matters is what it does with a NULL
/// site: it does NOT call `IUnknown_SetSite` and it stores NULL in `m_pObj`, so the
/// destructor's unconditional `IUnknown_SetSite(m_pObj, NULL)` is a no-op rather than
/// clearing a site the scope never set. Storing `pObj` regardless would clear a site
/// somebody else installed.
///
/// `drop_target_helper::simulate_drop_with_site` inlines these same semantics for the one
/// call it needed; this is the reusable form, for translated bodies that construct the
/// C++ object by name.
pub struct CScopedSetObjectWithSite {
    m_pObj: *mut crate::shellcom::IUnknown,
}

impl CScopedSetObjectWithSite {
    pub unsafe fn new(
        pObj: *mut crate::shellcom::IUnknown,
        pSite: *mut crate::shellcom::IUnknown,
    ) -> Self {
        if pSite.is_null() {
            return Self { m_pObj: core::ptr::null_mut() };
        }
        IUnknown_SetSite((pObj).cast(), (pSite).cast());
        Self { m_pObj: pObj }
    }
}

impl Drop for CScopedSetObjectWithSite {
    fn drop(&mut self) {
        unsafe {
            IUnknown_SetSite((self.m_pObj).cast(), core::ptr::null_mut());
        }
    }
}

use crate::shell_link::CMINVOKECOMMANDINFOEX;

/// `shellutils.h:722` — `IsUnicode(const CMINVOKECOMMANDINFOEX &ici)`.
///
/// Both halves matter: the caller may hand us a plain CMINVOKECOMMANDINFO, so the flag alone
/// is not enough — `cbSize` must also reach `ptInvoke`, the first field past the ANSI-only
/// prefix, before `lpVerbW` is real storage rather than someone else's memory.
#[inline]
pub unsafe fn IsUnicode(ici: *const CMINVOKECOMMANDINFOEX) -> bool {
    let minsize = core::mem::offset_of!(CMINVOKECOMMANDINFOEX, ptInvoke) as UINT;
    ((*ici).fMask & crate::shell32::utils_support::CMIC_MASK_UNICODE) != 0 && (*ici).cbSize >= minsize
}
