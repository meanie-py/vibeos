//! CompositeMoniker implementation — translation of dll/win32/ole32/compositemoniker.c
//!
//! A generic composite moniker (IMoniker + IROTData + IMarshal) over a growable table of
//! component monikers, plus the EnumMonikerImpl (IEnumMoniker) that walks it in either
//! direction. Every table slot holds a REFERENCE, so the AddRef/Release pairing against
//! the C is load-bearing.
use crate::prelude::*;
use core::ffi::c_void;

// ---------------------------------------------------------------------------
// Local bits the shared surface does not (yet) carry.
//
// OleLoadFromStream / OleSaveToStream are ole32's own exports, still implemented in C
// (storage32.c) — they belong in crate::ole32_exports' "still C" block. Declared
// privately here (WINAPI ⇒ `extern "system"`, never `extern "C"`) so they cannot
// collide with a future shared declaration.
// ---------------------------------------------------------------------------
extern "system" {
    fn OleLoadFromStream(pStm: *mut IStream, iidInterface: *const IID, ppvObj: *mut LPVOID)
        -> HRESULT;
    fn OleSaveToStream(pPStm: *mut IPersistStream, pStm: *mut IStream) -> HRESULT;
}

/// compositemoniker.c: `#define BLOCK_TAB_SIZE 5` — the first table size and its
/// increment block size.
const BLOCK_TAB_SIZE: ULONG = 5;

// ---------------------------------------------------------------------------
// structs
// ---------------------------------------------------------------------------

/// CompositeMoniker data structure (`ref` → `ref_`, a Rust keyword).
#[repr(C)]
pub struct CompositeMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub IMarshal_iface: IMarshal,
    pub ref_: LONG,
    /// dynamic table containing all components (monikers) of this composite moniker
    pub tabMoniker: *mut *mut IMoniker,
    /// size of tabMoniker
    pub tabSize: ULONG,
    /// first free index in tabMoniker
    pub tabLastIndex: ULONG,
}

/// EnumMoniker data structure
#[repr(C)]
pub struct EnumMonikerImpl {
    pub IEnumMoniker_iface: IEnumMoniker,
    pub ref_: LONG,
    /// dynamic table containing the enumerated monikers
    pub tabMoniker: *mut *mut IMoniker,
    /// size of tabMoniker
    pub tabSize: ULONG,
    /// index pointer on the current moniker
    pub currentPos: ULONG,
}

#[inline]
unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut CompositeMonikerImpl {
    CONTAINING_RECORD!(iface, CompositeMonikerImpl, IMoniker_iface)
}

#[inline]
unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut CompositeMonikerImpl {
    CONTAINING_RECORD!(iface, CompositeMonikerImpl, IROTData_iface)
}

#[inline]
unsafe fn impl_from_IMarshal(iface: *mut IMarshal) -> *mut CompositeMonikerImpl {
    CONTAINING_RECORD!(iface, CompositeMonikerImpl, IMarshal_iface)
}

#[inline]
unsafe fn impl_from_IEnumMoniker(iface: *mut IEnumMoniker) -> *mut EnumMonikerImpl {
    CONTAINING_RECORD!(iface, EnumMonikerImpl, IEnumMoniker_iface)
}

// ---------------------------------------------------------------------------
// IMoniker
// ---------------------------------------------------------------------------

stdmethod! {
fn CompositeMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const c_char, This,
           debugstr_guid(riid as *const c_void), ppvObject);

    /* Perform a sanity check on the parameters.*/
    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    /* Initialize the return parameter */
    *ppvObject = core::ptr::null_mut();

    /* Compare the riid with the interface IDs implemented by this object.*/
    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid)
    {
        *ppvObject = iface as *mut c_void;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IROTData), riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IROTData_iface) as *mut c_void;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IMarshal), riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IMarshal_iface) as *mut c_void;
    }

    /* Check that we obtained an interface.*/
    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    /* QueryInterface always increases the reference count by one when it is successful */
    IMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn CompositeMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

unsafe fn CompositeMonikerImpl_ReleaseMonikersInTable(This: *mut CompositeMonikerImpl) {
    let mut i: ULONG = 0;

    while i < (*This).tabLastIndex {
        IMoniker_Release(*(*This).tabMoniker.add(i as usize));
        i += 1;
    }

    (*This).tabLastIndex = 0;
}

stdmethod! {
fn CompositeMonikerImpl_Release(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, This);

    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    /* destroy the object if there are no more references to it */
    if ref_ == 0 {
        /* release all the components before destroying this object */
        CompositeMonikerImpl_ReleaseMonikersInTable(This);

        HeapFree(GetProcessHeap(), 0, (*This).tabMoniker as LPVOID);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_
}
}

stdmethod! {
fn CompositeMonikerImpl_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_CompositeMoniker;

    S_OK
}
}

stdmethod! {
/// The OLE-provided IPersistStream::IsDirty implementations always return S_FALSE
/// because their internal state never changes.
fn CompositeMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    S_FALSE
}
}

stdmethod! {
fn CompositeMonikerImpl_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;
    let mut moniker_count: DWORD = 0;
    let mut i: DWORD;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pStm);

    /* this function calls OleLoadFromStream for each moniker within this object */

    res = IStream_Read(pStm, core::ptr::addr_of_mut!(moniker_count) as *mut c_void,
                       core::mem::size_of::<DWORD>() as ULONG, core::ptr::null_mut());
    if res != S_OK {
        ERR!(b"couldn't reading moniker count from stream\n\0".as_ptr() as *const c_char);
        return E_FAIL;
    }

    CompositeMonikerImpl_ReleaseMonikersInTable(This);

    i = 0;
    while i < moniker_count {
        res = OleLoadFromStream(
            pStm,
            core::ptr::addr_of!(IID_IMoniker),
            (*This).tabMoniker.add((*This).tabLastIndex as usize) as *mut LPVOID,
        );
        if FAILED(res) {
            ERR!(b"couldn't load moniker from stream, res = 0x%08x\n\0".as_ptr() as *const c_char,
                 res);
            break;
        }

        /* resize the table if needed */
        (*This).tabLastIndex += 1;
        if (*This).tabLastIndex == (*This).tabSize {
            (*This).tabSize += BLOCK_TAB_SIZE;
            (*This).tabMoniker = HeapReAlloc(
                GetProcessHeap(),
                0,
                (*This).tabMoniker as LPVOID,
                ((*This).tabSize as usize * core::mem::size_of::<*mut IMoniker>()) as SIZE_T,
            ) as *mut *mut IMoniker;

            if (*This).tabMoniker.is_null() {
                return E_OUTOFMEMORY;
            }
        }

        i += 1;
    }

    res
}
}

stdmethod! {
fn CompositeMonikerImpl_Save(iface: *mut IMoniker, pStm: *mut IStream, fClearDirty: BOOL)
    -> HRESULT
{
    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;
    let mut enumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmk: *mut IMoniker = core::ptr::null_mut();
    let moniker_count: DWORD = (*This).tabLastIndex;

    TRACE!(b"(%p,%p,%d)\n\0".as_ptr() as *const c_char, iface, pStm, fClearDirty);

    /* This function calls OleSaveToStream for each moniker within this object. */
    res = IStream_Write(pStm, core::ptr::addr_of!(moniker_count) as *const c_void,
                        core::mem::size_of::<DWORD>() as ULONG, core::ptr::null_mut());
    if FAILED(res) {
        return res;
    }

    IMoniker_Enum(iface, 1 /* TRUE */, &mut enumMk);

    while IEnumMoniker_Next(enumMk, 1, &mut pmk, core::ptr::null_mut()) == S_OK {
        res = OleSaveToStream(pmk as *mut IPersistStream, pStm);

        IMoniker_Release(pmk);

        if FAILED(res) {
            IEnumMoniker_Release(enumMk);
            return res;
        }
    }

    IEnumMoniker_Release(enumMk);

    S_OK
}
}

stdmethod! {
/// The sizeMax of this object is calculated by calling GetSizeMax on each moniker within
/// this object then summing all returned values.
fn CompositeMonikerImpl_GetSizeMax(iface: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER)
    -> HRESULT
{
    let mut enumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmk: *mut IMoniker = core::ptr::null_mut();
    let mut ptmpSize: ULARGE_INTEGER = core::mem::zeroed();

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pcbSize);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    (*pcbSize).QuadPart = core::mem::size_of::<DWORD>() as ULONGLONG;

    IMoniker_Enum(iface, 1 /* TRUE */, &mut enumMk);

    while IEnumMoniker_Next(enumMk, 1, &mut pmk, core::ptr::null_mut()) == S_OK {
        IMoniker_GetSizeMax(pmk, &mut ptmpSize);

        IMoniker_Release(pmk);

        (*pcbSize).QuadPart = ptmpSize.QuadPart + core::mem::size_of::<CLSID>() as ULONGLONG;
    }

    IEnumMoniker_Release(enumMk);

    S_OK
}
}

stdmethod! {
fn CompositeMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut *mut c_void,
) -> HRESULT {
    let mut res: HRESULT = E_FAIL;
    let mut prot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut antiMk: *mut IMoniker = core::ptr::null_mut();
    let mut rightMostMk: *mut IMoniker = core::ptr::null_mut();
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%p,%s,%p)\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           debugstr_guid(riid as *const c_void), ppvResult);

    if ppvResult.is_null() {
        return E_POINTER;
    }

    *ppvResult = core::ptr::null_mut();

    /* If pmkToLeft is NULL, this method looks for the moniker in the ROT, and if found,
     * queries the retrieved object for the requested interface pointer. */
    if pmkToLeft.is_null() {
        res = IBindCtx_GetRunningObjectTable(pbc, &mut prot);

        if SUCCEEDED(res) {
            /* if the requested class was loaded before ! we don't need to reload it */
            res = IRunningObjectTable_GetObject(prot, iface,
                                                ppvResult as *mut *mut IUnknown);

            if res == S_OK {
                return res;
            }
        }
    } else {
        /* If pmkToLeft is not NULL, the method recursively calls IMoniker::BindToObject on
         * the rightmost component of the composite, passing the rest of the composite as
         * the pmkToLeft parameter for that call */
        IMoniker_Enum(iface, 0 /* FALSE */, &mut enumMoniker);
        IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
        IEnumMoniker_Release(enumMoniker);

        res = crate::ring::antimoniker::CreateAntiMoniker(&mut antiMk);
        res = IMoniker_ComposeWith(iface, antiMk, 0, &mut tempMk);
        IMoniker_Release(antiMk);

        res = IMoniker_BindToObject(rightMostMk, pbc, tempMk, riid, ppvResult);

        IMoniker_Release(tempMk);
        IMoniker_Release(rightMostMk);
    }

    res
}
}

stdmethod! {
fn CompositeMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut *mut c_void,
) -> HRESULT {
    let mut res: HRESULT;
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut antiMk: *mut IMoniker = core::ptr::null_mut();
    let mut rightMostMk: *mut IMoniker = core::ptr::null_mut();
    let leftMk: *mut IMoniker;
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%p,%s,%p)\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           debugstr_guid(riid as *const c_void), ppvResult);

    *ppvResult = core::ptr::null_mut();

    /* This method recursively calls BindToStorage on the rightmost component of the
     * composite, passing the rest of the composite as the pmkToLeft parameter. */

    if !pmkToLeft.is_null() {
        let mut left: *mut IMoniker = core::ptr::null_mut();
        res = IMoniker_ComposeWith(pmkToLeft, iface, 0 /* FALSE */, &mut left);
        if FAILED(res) {
            return res;
        }
        leftMk = left;
    } else {
        leftMk = iface;
    }

    IMoniker_Enum(iface, 0 /* FALSE */, &mut enumMoniker);
    IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
    IEnumMoniker_Release(enumMoniker);

    res = crate::ring::antimoniker::CreateAntiMoniker(&mut antiMk);
    if FAILED(res) {
        return res;
    }
    res = IMoniker_ComposeWith(leftMk, antiMk, 0, &mut tempMk);
    if FAILED(res) {
        return res;
    }
    IMoniker_Release(antiMk);

    res = IMoniker_BindToStorage(rightMostMk, pbc, tempMk, riid, ppvResult);

    IMoniker_Release(tempMk);

    IMoniker_Release(rightMostMk);

    if !pmkToLeft.is_null() {
        IMoniker_Release(leftMk);
    }

    res
}
}

stdmethod! {
fn CompositeMonikerImpl_Reduce(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    dwReduceHowFar: DWORD,
    ppmkToLeft: *mut *mut IMoniker,
    ppmkReduced: *mut *mut IMoniker,
) -> HRESULT {
    let res: HRESULT;
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut antiMk: *mut IMoniker = core::ptr::null_mut();
    let mut rightMostMk: *mut IMoniker = core::ptr::null_mut();
    let mut leftReducedComposedMk: *mut IMoniker = core::ptr::null_mut();
    let mut rightMostReducedMk: *mut IMoniker = core::ptr::null_mut();
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%d,%p,%p)\n\0".as_ptr() as *const c_char, iface, pbc, dwReduceHowFar,
           ppmkToLeft, ppmkReduced);

    if ppmkReduced.is_null() {
        return E_POINTER;
    }

    /* This method recursively calls Reduce for each of its component monikers. */

    if ppmkToLeft.is_null() {
        IMoniker_Enum(iface, 0 /* FALSE */, &mut enumMoniker);
        IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
        IEnumMoniker_Release(enumMoniker);

        crate::ring::antimoniker::CreateAntiMoniker(&mut antiMk);
        IMoniker_ComposeWith(iface, antiMk, 0, &mut tempMk);
        IMoniker_Release(antiMk);

        res = IMoniker_Reduce(rightMostMk, pbc, dwReduceHowFar, &mut tempMk, ppmkReduced);
        IMoniker_Release(tempMk);
        IMoniker_Release(rightMostMk);

        res
    } else if (*ppmkToLeft).is_null() {
        IMoniker_Reduce(iface, pbc, dwReduceHowFar, core::ptr::null_mut(), ppmkReduced)
    } else {
        /* separate the composite moniker in to left and right moniker */
        IMoniker_Enum(iface, 0 /* FALSE */, &mut enumMoniker);
        IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
        IEnumMoniker_Release(enumMoniker);

        crate::ring::antimoniker::CreateAntiMoniker(&mut antiMk);
        IMoniker_ComposeWith(iface, antiMk, 0, &mut tempMk);
        IMoniker_Release(antiMk);

        /* If any of the components reduces itself, the method returns S_OK and passes back
         * a composite of the reduced components.
         * (The C tests the HRESULTs for C truthiness — i.e. NON-ZERO, so S_OK(0) is
         * "false" here. Kept verbatim.) */
        if IMoniker_Reduce(rightMostMk, pbc, dwReduceHowFar, core::ptr::null_mut(),
                           &mut rightMostReducedMk) != 0
            && IMoniker_Reduce(rightMostMk, pbc, dwReduceHowFar, &mut tempMk,
                               &mut leftReducedComposedMk) != 0
        {
            IMoniker_Release(tempMk);
            IMoniker_Release(rightMostMk);

            CreateGenericComposite(leftReducedComposedMk, rightMostReducedMk, ppmkReduced)
        } else {
            /* If no reduction occurred, the method passes back the same moniker and returns
             * MK_S_REDUCED_TO_SELF. */
            IMoniker_Release(tempMk);
            IMoniker_Release(rightMostMk);

            IMoniker_AddRef(iface);

            *ppmkReduced = iface;

            MK_S_REDUCED_TO_SELF
        }
    }
}
}

stdmethod! {
fn CompositeMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    pmkRight: *mut IMoniker,
    fOnlyIfNotGeneric: BOOL,
    ppmkComposite: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%p,%d,%p)\n\0".as_ptr() as *const c_char, iface, pmkRight,
           fOnlyIfNotGeneric, ppmkComposite);

    if ppmkComposite.is_null() || pmkRight.is_null() {
        return E_POINTER;
    }

    *ppmkComposite = core::ptr::null_mut();

    /* If fOnlyIfNotGeneric is TRUE, this method sets *pmkComposite to NULL and returns
     * MK_E_NEEDGENERIC; otherwise, the method returns the result of combining the two
     * monikers by calling the CreateGenericComposite function */

    if fOnlyIfNotGeneric != 0 {
        return MK_E_NEEDGENERIC;
    }

    CreateGenericComposite(iface, pmkRight, ppmkComposite)
}
}

stdmethod! {
fn CompositeMonikerImpl_Enum(
    iface: *mut IMoniker,
    fForward: BOOL,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%d,%p)\n\0".as_ptr() as *const c_char, iface, fForward, ppenumMoniker);

    if ppenumMoniker.is_null() {
        return E_POINTER;
    }

    EnumMonikerImpl_CreateEnumMoniker((*This).tabMoniker, (*This).tabLastIndex, 0, fForward,
                                      ppenumMoniker)
}
}

stdmethod! {
fn CompositeMonikerImpl_IsEqual(iface: *mut IMoniker, pmkOtherMoniker: *mut IMoniker)
    -> HRESULT
{
    let mut enumMoniker1: *mut IEnumMoniker = core::ptr::null_mut();
    let mut enumMoniker2: *mut IEnumMoniker = core::ptr::null_mut();
    let mut tempMk1: *mut IMoniker = core::ptr::null_mut();
    let mut tempMk2: *mut IMoniker = core::ptr::null_mut();
    let mut res1: HRESULT;
    let mut res2: HRESULT;
    let mut res: HRESULT;
    let mut done: BOOL;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pmkOtherMoniker);

    if pmkOtherMoniker.is_null() {
        return S_FALSE;
    }

    /* This method returns S_OK if the components of both monikers are equal when compared
     * in the left-to-right order. */
    IMoniker_Enum(pmkOtherMoniker, 1 /* TRUE */, &mut enumMoniker1);

    if enumMoniker1.is_null() {
        return S_FALSE;
    }

    IMoniker_Enum(iface, 1 /* TRUE */, &mut enumMoniker2);

    loop {
        res1 = IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());
        res2 = IEnumMoniker_Next(enumMoniker2, 1, &mut tempMk2, core::ptr::null_mut());

        if (res1 == S_OK) && (res2 == S_OK) {
            res = IMoniker_IsEqual(tempMk1, tempMk2);
            done = (res == S_FALSE) as BOOL;
        } else {
            /* The C assigns a BOOL to the HRESULT here:
             *     res = (res1==S_FALSE) && (res2==S_FALSE);
             * i.e. 1 (== S_FALSE) when BOTH enumerators are exhausted, else 0 (== S_OK).
             * Kept verbatim — do not "fix" the polarity, the oracle is the C. */
            res = ((res1 == S_FALSE) && (res2 == S_FALSE)) as HRESULT;
            done = 1;
        }

        if res1 == S_OK {
            IMoniker_Release(tempMk1);
        }

        if res2 == S_OK {
            IMoniker_Release(tempMk2);
        }

        if done != 0 {
            break;
        }
    }

    IEnumMoniker_Release(enumMoniker1);
    IEnumMoniker_Release(enumMoniker2);

    res
}
}

stdmethod! {
fn CompositeMonikerImpl_Hash(iface: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT {
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut res: HRESULT;
    let mut tempHash: DWORD = 0;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pdwHash);

    if pdwHash.is_null() {
        return E_POINTER;
    }

    res = IMoniker_Enum(iface, 1 /* TRUE */, &mut enumMoniker);
    if FAILED(res) {
        return res;
    }

    *pdwHash = 0;

    while IEnumMoniker_Next(enumMoniker, 1, &mut tempMk, core::ptr::null_mut()) == S_OK {
        res = IMoniker_Hash(tempMk, &mut tempHash);
        if FAILED(res) {
            /* NOTE: the C breaks out here WITHOUT releasing tempMk. Kept verbatim. */
            break;
        }
        *pdwHash ^= tempHash;

        IMoniker_Release(tempMk);
    }

    IEnumMoniker_Release(enumMoniker);

    res
}
}

stdmethod! {
fn CompositeMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut res: HRESULT;
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut antiMk: *mut IMoniker = core::ptr::null_mut();
    let mut rightMostMk: *mut IMoniker = core::ptr::null_mut();
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           pmkNewlyRunning);

    /* If pmkToLeft is non-NULL, this method composes pmkToLeft with this moniker and calls
     * IsRunning on the result. */
    if !pmkToLeft.is_null() {
        CreateGenericComposite(pmkToLeft, iface, &mut tempMk);

        res = IMoniker_IsRunning(tempMk, pbc, core::ptr::null_mut(), pmkNewlyRunning);

        IMoniker_Release(tempMk);

        res
    } else if !pmkNewlyRunning.is_null() {
        /* If pmkToLeft is NULL, this method returns S_OK if pmkNewlyRunning is non-NULL and
         * is equal to this moniker */
        if IMoniker_IsEqual(iface, pmkNewlyRunning) == S_OK {
            S_OK
        } else {
            S_FALSE
        }
    } else {
        if pbc.is_null() {
            return E_INVALIDARG;
        }

        /* If pmkToLeft and pmkNewlyRunning are both NULL, this method checks the ROT to see
         * whether the moniker is running. If so, the method returns S_OK; otherwise, it
         * recursively calls IMoniker::IsRunning on the rightmost component of the composite,
         * passing the remainder of the composite as the pmkToLeft parameter for that call. */
        res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);

        if FAILED(res) {
            return res;
        }

        res = IRunningObjectTable_IsRunning(rot, iface);
        IRunningObjectTable_Release(rot);

        if res == S_OK {
            S_OK
        } else {
            IMoniker_Enum(iface, 0 /* FALSE */, &mut enumMoniker);
            IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
            IEnumMoniker_Release(enumMoniker);

            res = crate::ring::antimoniker::CreateAntiMoniker(&mut antiMk);
            res = IMoniker_ComposeWith(iface, antiMk, 0, &mut tempMk);
            IMoniker_Release(antiMk);

            res = IMoniker_IsRunning(rightMostMk, pbc, tempMk, pmkNewlyRunning);

            IMoniker_Release(tempMk);
            IMoniker_Release(rightMostMk);

            res
        }
    }
}
}

stdmethod! {
fn CompositeMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pCompositeTime: *mut FILETIME,
) -> HRESULT {
    let mut res: HRESULT;
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut antiMk: *mut IMoniker = core::ptr::null_mut();
    let mut rightMostMk: *mut IMoniker = core::ptr::null_mut();
    let leftMk: *mut IMoniker;
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           pCompositeTime);

    if pCompositeTime.is_null() {
        return E_INVALIDARG;
    }

    /* This method creates a composite of pmkToLeft (if non-NULL) and this moniker and uses
     * the ROT to retrieve the time of last change. If the object is not in the ROT, the
     * method recursively calls IMoniker::GetTimeOfLastChange on the rightmost component of
     * the composite, passing the remainder of the composite as pmkToLeft. */
    if !pmkToLeft.is_null() {
        let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
        let mut left: *mut IMoniker = core::ptr::null_mut();

        res = IMoniker_ComposeWith(pmkToLeft, iface, 0 /* FALSE */, &mut left);
        if FAILED(res) {
            return res;
        }
        leftMk = left;

        res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);
        if FAILED(res) {
            IMoniker_Release(leftMk);
            return res;
        }

        if IRunningObjectTable_GetTimeOfLastChange(rot, leftMk, pCompositeTime) == S_OK {
            IMoniker_Release(leftMk);
            return res;
        }
    } else {
        leftMk = iface;
    }

    IMoniker_Enum(iface, 0 /* FALSE */, &mut enumMoniker);
    IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
    IEnumMoniker_Release(enumMoniker);

    res = crate::ring::antimoniker::CreateAntiMoniker(&mut antiMk);
    res = IMoniker_ComposeWith(leftMk, antiMk, 0, &mut tempMk);
    IMoniker_Release(antiMk);

    res = IMoniker_GetTimeOfLastChange(rightMostMk, pbc, tempMk, pCompositeTime);

    IMoniker_Release(tempMk);
    IMoniker_Release(rightMostMk);

    if !pmkToLeft.is_null() {
        IMoniker_Release(leftMk);
    }

    res
}
}

stdmethod! {
fn CompositeMonikerImpl_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {
    let res: HRESULT;
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut antiMk: *mut IMoniker = core::ptr::null_mut();
    let mut rightMostMk: *mut IMoniker = core::ptr::null_mut();
    let mut tempInvMk: *mut IMoniker = core::ptr::null_mut();
    let mut rightMostInvMk: *mut IMoniker = core::ptr::null_mut();
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, ppmk);

    if ppmk.is_null() {
        return E_POINTER;
    }

    /* This method returns a composite moniker that consists of the inverses of each of the
     * components of the original composite, stored in reverse order */

    *ppmk = core::ptr::null_mut();

    let mut hr = crate::ring::antimoniker::CreateAntiMoniker(&mut antiMk);
    if FAILED(hr) {
        return hr;
    }

    hr = IMoniker_ComposeWith(iface, antiMk, 0 /* FALSE */, &mut tempMk);
    IMoniker_Release(antiMk);
    if FAILED(hr) {
        return hr;
    }

    if tempMk.is_null() {
        IMoniker_Inverse(iface, ppmk)
    } else {
        IMoniker_Enum(iface, 0 /* FALSE */, &mut enumMoniker);
        IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
        IEnumMoniker_Release(enumMoniker);

        IMoniker_Inverse(rightMostMk, &mut rightMostInvMk);
        CompositeMonikerImpl_Inverse(tempMk, &mut tempInvMk);

        res = CreateGenericComposite(rightMostInvMk, tempInvMk, ppmk);

        IMoniker_Release(tempMk);
        IMoniker_Release(rightMostMk);
        IMoniker_Release(tempInvMk);
        IMoniker_Release(rightMostInvMk);

        res
    }
}
}

stdmethod! {
fn CompositeMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkPrefix: *mut *mut IMoniker,
) -> HRESULT {
    let mut mkSys: DWORD = 0;
    let mut res1: HRESULT;
    let mut res2: HRESULT;
    let mut tempMk1: *mut IMoniker = core::ptr::null_mut();
    let mut tempMk2: *mut IMoniker = core::ptr::null_mut();
    let mut mostLeftMk1: *mut IMoniker = core::ptr::null_mut();
    let mut mostLeftMk2: *mut IMoniker = core::ptr::null_mut();
    let mut enumMoniker1: *mut IEnumMoniker = core::ptr::null_mut();
    let mut enumMoniker2: *mut IEnumMoniker = core::ptr::null_mut();
    let mut i: ULONG;
    let mut nbCommonMk: ULONG = 0;

    /* If the other moniker is a composite, this method compares the components of each
     * composite from left to right. The returned common prefix moniker might also be a
     * composite moniker, depending on how many of the leftmost components were common to
     * both monikers. */

    if ppmkPrefix.is_null() {
        return E_POINTER;
    }

    *ppmkPrefix = core::ptr::null_mut();

    if pmkOther.is_null() {
        return MK_E_NOPREFIX;
    }

    IMoniker_IsSystemMoniker(pmkOther, &mut mkSys);

    if mkSys == MKSYS_GENERICCOMPOSITE {
        IMoniker_Enum(iface, 1 /* TRUE */, &mut enumMoniker1);
        IMoniker_Enum(pmkOther, 1 /* TRUE */, &mut enumMoniker2);

        loop {
            res1 = IEnumMoniker_Next(enumMoniker1, 1, &mut mostLeftMk1, core::ptr::null_mut());
            res2 = IEnumMoniker_Next(enumMoniker2, 1, &mut mostLeftMk2, core::ptr::null_mut());

            if (res1 == S_FALSE) && (res2 == S_FALSE) {
                /* If the monikers are equal, the method returns MK_S_US and sets ppmkPrefix
                 * to this moniker. */
                *ppmkPrefix = iface;
                IMoniker_AddRef(iface);
                return MK_S_US;
            } else if (res1 == S_OK) && (res2 == S_OK) {
                if IMoniker_IsEqual(mostLeftMk1, mostLeftMk2) == S_OK {
                    nbCommonMk += 1;
                } else {
                    break;
                }
            } else if res1 == S_OK {
                /* If the other moniker is a prefix of this moniker, the method returns
                 * MK_S_HIM and sets ppmkPrefix to the other moniker. */
                *ppmkPrefix = pmkOther;
                return MK_S_HIM;
            } else {
                /* If this moniker is a prefix of the other, this method returns MK_S_ME and
                 * sets ppmkPrefix to this moniker. */
                *ppmkPrefix = iface;
                return MK_S_ME;
            }
        }

        IEnumMoniker_Release(enumMoniker1);
        IEnumMoniker_Release(enumMoniker2);

        /* If there is no common prefix, this method returns MK_E_NOPREFIX and sets
         * ppmkPrefix to NULL. */
        if nbCommonMk == 0 {
            return MK_E_NOPREFIX;
        }

        IEnumMoniker_Reset(enumMoniker1);

        IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());

        /* if we have more than one common moniker the result will be a composite moniker */
        if nbCommonMk > 1 {
            /* initialize the common prefix moniker with the composite of the two first
             * monikers (from the left) */
            IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk2, core::ptr::null_mut());
            CreateGenericComposite(tempMk1, tempMk2, ppmkPrefix);
            IMoniker_Release(tempMk1);
            IMoniker_Release(tempMk2);

            /* compose all common monikers in a composite moniker */
            i = 0;
            while i < nbCommonMk {
                IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());

                CreateGenericComposite(*ppmkPrefix, tempMk1, &mut tempMk2);

                IMoniker_Release(*ppmkPrefix);

                IMoniker_Release(tempMk1);

                *ppmkPrefix = tempMk2;
                i += 1;
            }

            S_OK
        } else {
            /* if we have only one common moniker the result will be a simple moniker which
             * is the most-left one */
            *ppmkPrefix = tempMk1;

            S_OK
        }
    } else {
        /* If the other moniker is not a composite, the method simply compares it to the
         * leftmost component of this moniker. */
        IMoniker_Enum(iface, 1 /* TRUE */, &mut enumMoniker1);

        IEnumMoniker_Next(enumMoniker1, 1, &mut mostLeftMk1, core::ptr::null_mut());

        if IMoniker_IsEqual(pmkOther, mostLeftMk1) == S_OK {
            *ppmkPrefix = pmkOther;

            MK_S_HIM
        } else {
            MK_E_NOPREFIX
        }
    }
}
}

/// GetAfterCommonPrefix (local function)
/// This function returns a moniker that consists of the remainder when the common prefix
/// is removed.
unsafe fn GetAfterCommonPrefix(
    pGenMk: *mut IMoniker,
    commonMk: *mut IMoniker,
    restMk: *mut *mut IMoniker,
) {
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut tempMk1: *mut IMoniker = core::ptr::null_mut();
    let mut tempMk2: *mut IMoniker = core::ptr::null_mut();
    let mut enumMoniker1: *mut IEnumMoniker = core::ptr::null_mut();
    let mut enumMoniker2: *mut IEnumMoniker = core::ptr::null_mut();
    let mut enumMoniker3: *mut IEnumMoniker = core::ptr::null_mut();
    let mut nbRestMk: ULONG = 0;
    let mut mkSys: DWORD = 0;
    let mut res1: HRESULT;
    let mut res2: HRESULT;

    *restMk = core::ptr::null_mut();

    /* to create an enumerator for pGenMk with current position pointed on the first element
     * after the common prefix: enum the two monikers (left-right) then compare these
     * enumerations (left-right) and stop on the first difference. */
    IMoniker_Enum(pGenMk, 1 /* TRUE */, &mut enumMoniker1);

    IMoniker_IsSystemMoniker(commonMk, &mut mkSys);

    if mkSys == MKSYS_GENERICCOMPOSITE {
        IMoniker_Enum(commonMk, 1 /* TRUE */, &mut enumMoniker2);
        loop {
            res1 = IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());
            res2 = IEnumMoniker_Next(enumMoniker2, 1, &mut tempMk2, core::ptr::null_mut());

            if (res1 == S_FALSE) || (res2 == S_FALSE) {
                if res1 == S_OK {
                    nbRestMk += 1;
                }

                IMoniker_Release(tempMk1);
                IMoniker_Release(tempMk2);

                break;
            }
            IMoniker_Release(tempMk1);
            IMoniker_Release(tempMk2);
        }
    } else {
        IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());
        IMoniker_Release(tempMk1);
    }

    /* count the number of elements in the enumerator after the common prefix */
    IEnumMoniker_Clone(enumMoniker1, &mut enumMoniker3);

    while IEnumMoniker_Next(enumMoniker3, 1, &mut tempMk, core::ptr::null_mut()) == S_OK {
        IMoniker_Release(tempMk);
        nbRestMk += 1;
    }

    if nbRestMk == 0 {
        return;
    }

    /* create a generic composite moniker with monikers located after the common prefix */
    IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut());

    if nbRestMk == 1 {
        *restMk = tempMk1;
    } else {
        IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk2, core::ptr::null_mut());

        CreateGenericComposite(tempMk1, tempMk2, restMk);

        IMoniker_Release(tempMk1);

        IMoniker_Release(tempMk2);

        while IEnumMoniker_Next(enumMoniker1, 1, &mut tempMk1, core::ptr::null_mut()) == S_OK {
            CreateGenericComposite(*restMk, tempMk1, &mut tempMk2);

            IMoniker_Release(tempMk1);

            IMoniker_Release(*restMk);

            *restMk = tempMk2;
        }
    }
}

stdmethod! {
fn CompositeMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker,
) -> HRESULT {
    let res: HRESULT;
    let mut restOtherMk: *mut IMoniker = core::ptr::null_mut();
    let mut restThisMk: *mut IMoniker = core::ptr::null_mut();
    let mut invRestThisMk: *mut IMoniker = core::ptr::null_mut();
    let mut commonMk: *mut IMoniker = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%p)\n\0".as_ptr() as *const c_char, iface, pmkOther, ppmkRelPath);

    if ppmkRelPath.is_null() {
        return E_POINTER;
    }

    *ppmkRelPath = core::ptr::null_mut();

    /* This method finds the common prefix of the two monikers and creates two monikers that
     * consist of the remainder when the common prefix is removed. Then it creates the
     * inverse for the remainder of this moniker and composes the remainder of the other
     * moniker on the right of it. */

    /* finds the common prefix of the two monikers */
    res = IMoniker_CommonPrefixWith(iface, pmkOther, &mut commonMk);

    /* if there's no common prefix or the two monikers are equal the relative is the other
     * moniker */
    if (res == MK_E_NOPREFIX) || (res == MK_S_US) {
        *ppmkRelPath = pmkOther;
        IMoniker_AddRef(pmkOther);
        return MK_S_HIM;
    }

    GetAfterCommonPrefix(iface, commonMk, &mut restThisMk);
    GetAfterCommonPrefix(pmkOther, commonMk, &mut restOtherMk);

    /* if other is a prefix of this moniker the relative path is the inverse of the remainder
     * path of this moniker when the common prefix is removed */
    if res == MK_S_HIM {
        IMoniker_Inverse(restThisMk, ppmkRelPath);
        IMoniker_Release(restThisMk);
    }
    /* if this moniker is a prefix of the other moniker the relative path is the remainder
     * path of the other moniker when the common prefix is removed */
    else if res == MK_S_ME {
        *ppmkRelPath = restOtherMk;
        IMoniker_AddRef(restOtherMk);
    }
    /* the relative path is the inverse for the remainder of this moniker and the remainder
     * of the other moniker on the right of it. */
    else if res == S_OK {
        IMoniker_Inverse(restThisMk, &mut invRestThisMk);
        IMoniker_Release(restThisMk);
        CreateGenericComposite(invRestThisMk, restOtherMk, ppmkRelPath);
        IMoniker_Release(invRestThisMk);
        IMoniker_Release(restOtherMk);
    }

    S_OK
}
}

stdmethod! {
fn CompositeMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPOLESTR,
) -> HRESULT {
    let mut lengthStr: ULONG = 1;
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut tempStr: LPOLESTR = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           ppszDisplayName);

    if ppszDisplayName.is_null() {
        return E_POINTER;
    }

    *ppszDisplayName = CoTaskMemAlloc(core::mem::size_of::<WCHAR>() as SIZE_T) as LPOLESTR;

    if (*ppszDisplayName).is_null() {
        return E_OUTOFMEMORY;
    }

    /* This method returns the concatenation of the display names returned by each component
     * moniker of the composite */

    **ppszDisplayName = 0;

    IMoniker_Enum(iface, 1 /* TRUE */, &mut enumMoniker);

    while IEnumMoniker_Next(enumMoniker, 1, &mut tempMk, core::ptr::null_mut()) == S_OK {
        IMoniker_GetDisplayName(tempMk, pbc, core::ptr::null_mut(), &mut tempStr);

        lengthStr += lstrlenW(tempStr) as ULONG;

        *ppszDisplayName = CoTaskMemRealloc(
            *ppszDisplayName as LPVOID,
            (lengthStr as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPOLESTR;

        if (*ppszDisplayName).is_null() {
            return E_OUTOFMEMORY;
        }

        lstrcatW(*ppszDisplayName, tempStr);

        CoTaskMemFree(tempStr as LPVOID);
        IMoniker_Release(tempMk);
    }

    IEnumMoniker_Release(enumMoniker);

    S_OK
}
}

stdmethod! {
fn CompositeMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let mut rightMostMk: *mut IMoniker = core::ptr::null_mut();
    let mut antiMk: *mut IMoniker = core::ptr::null_mut();

    /* This method recursively calls IMoniker::ParseDisplayName on the rightmost component
     * of the composite, passing everything else as the pmkToLeft parameter for that call. */

    /* get the most right moniker */
    IMoniker_Enum(iface, 0 /* FALSE */, &mut enumMoniker);
    IEnumMoniker_Next(enumMoniker, 1, &mut rightMostMk, core::ptr::null_mut());
    IEnumMoniker_Release(enumMoniker);

    /* get the left moniker */
    crate::ring::antimoniker::CreateAntiMoniker(&mut antiMk);
    IMoniker_ComposeWith(iface, antiMk, 0, &mut tempMk);
    IMoniker_Release(antiMk);

    IMoniker_ParseDisplayName(rightMostMk, pbc, tempMk, pszDisplayName, pchEaten, ppmkOut)
}
}

stdmethod! {
fn CompositeMonikerImpl_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: *mut DWORD)
    -> HRESULT
{
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_GENERICCOMPOSITE;

    S_OK
}
}

// ---------------------------------------------------------------------------
// IROTData
// ---------------------------------------------------------------------------

stdmethod! {
fn CompositeMonikerROTDataImpl_QueryInterface(
    iface: *mut IROTData,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const c_char, iface,
           debugstr_guid(riid as *const c_void), ppvObject);

    CompositeMonikerImpl_QueryInterface(core::ptr::addr_of_mut!((*This).IMoniker_iface), riid,
                                        ppvObject)
}
}

stdmethod! {
fn CompositeMonikerROTDataImpl_AddRef(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    IMoniker_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn CompositeMonikerROTDataImpl_Release(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    IMoniker_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn CompositeMonikerROTDataImpl_GetComparisonData(
    iface: *mut IROTData,
    pbData: *mut byte,
    cbMax: ULONG,
    pcbData: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IROTData(iface);
    let mut pEnumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmk: *mut IMoniker = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(b"(%p, %u, %p)\n\0".as_ptr() as *const c_char, pbData, cbMax, pcbData);

    *pcbData = core::mem::size_of::<CLSID>() as ULONG;

    hr = IMoniker_Enum(core::ptr::addr_of_mut!((*This).IMoniker_iface), 1 /* TRUE */,
                       &mut pEnumMk);
    if FAILED(hr) {
        return hr;
    }

    while IEnumMoniker_Next(pEnumMk, 1, &mut pmk, core::ptr::null_mut()) == S_OK {
        let mut pROTData: *mut IROTData = core::ptr::null_mut();
        hr = IMoniker_QueryInterface(
            pmk,
            core::ptr::addr_of!(IID_IROTData),
            &mut pROTData as *mut *mut IROTData as *mut *mut c_void,
        );
        if FAILED(hr) {
            ERR!(b"moniker doesn't support IROTData interface\n\0".as_ptr() as *const c_char);
        }

        if SUCCEEDED(hr) {
            let mut cbData: ULONG = 0;
            hr = IROTData_GetComparisonData(pROTData, core::ptr::null_mut(), 0, &mut cbData);
            IROTData_Release(pROTData);
            if SUCCEEDED(hr) || (hr == E_OUTOFMEMORY) {
                *pcbData += cbData;
                hr = S_OK;
            } else {
                ERR!(b"IROTData_GetComparisonData failed with error 0x%08x\n\0".as_ptr()
                     as *const c_char, hr);
            }
        }

        IMoniker_Release(pmk);

        if FAILED(hr) {
            IEnumMoniker_Release(pEnumMk);
            return hr;
        }
    }
    if cbMax < *pcbData {
        return E_OUTOFMEMORY;
    }

    IEnumMoniker_Reset(pEnumMk);

    memcpy(
        pbData as *mut c_void,
        core::ptr::addr_of!(CLSID_CompositeMoniker) as *const c_void,
        core::mem::size_of::<CLSID>(),
    );
    let mut pbData = pbData.add(core::mem::size_of::<CLSID>());
    let mut cbMax = cbMax - core::mem::size_of::<CLSID>() as ULONG;

    while IEnumMoniker_Next(pEnumMk, 1, &mut pmk, core::ptr::null_mut()) == S_OK {
        let mut pROTData: *mut IROTData = core::ptr::null_mut();
        hr = IMoniker_QueryInterface(
            pmk,
            core::ptr::addr_of!(IID_IROTData),
            &mut pROTData as *mut *mut IROTData as *mut *mut c_void,
        );
        if FAILED(hr) {
            ERR!(b"moniker doesn't support IROTData interface\n\0".as_ptr() as *const c_char);
        }

        if SUCCEEDED(hr) {
            let mut cbData: ULONG = 0;
            hr = IROTData_GetComparisonData(pROTData, pbData, cbMax, &mut cbData);
            IROTData_Release(pROTData);
            if SUCCEEDED(hr) {
                pbData = pbData.add(cbData as usize);
                cbMax -= cbData;
            } else {
                ERR!(b"IROTData_GetComparisonData failed with error 0x%08x\n\0".as_ptr()
                     as *const c_char, hr);
            }
        }

        IMoniker_Release(pmk);

        if FAILED(hr) {
            IEnumMoniker_Release(pEnumMk);
            return hr;
        }
    }

    IEnumMoniker_Release(pEnumMk);

    S_OK
}
}

// ---------------------------------------------------------------------------
// IMarshal
// ---------------------------------------------------------------------------

stdmethod! {
fn CompositeMonikerMarshalImpl_QueryInterface(
    iface: *mut IMarshal,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const c_char, iface,
           debugstr_guid(riid as *const c_void), ppv);

    CompositeMonikerImpl_QueryInterface(core::ptr::addr_of_mut!((*This).IMoniker_iface), riid,
                                        ppv)
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_AddRef(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_IMarshal(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    CompositeMonikerImpl_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_Release(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_IMarshal(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    CompositeMonikerImpl_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_GetUnmarshalClass(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pCid: *mut CLSID,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);

    TRACE!(b"(%s, %p, %x, %p, %x, %p)\n\0".as_ptr() as *const c_char,
           debugstr_guid(riid as *const c_void), pv, dwDestContext, pvDestContext, mshlflags,
           pCid);

    IMoniker_GetClassID(core::ptr::addr_of_mut!((*This).IMoniker_iface), pCid)
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_GetMarshalSizeMax(
    iface: *mut IMarshal,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pSize: *mut DWORD,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    let mut pEnumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmk: *mut IMoniker = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut size: ULARGE_INTEGER = core::mem::zeroed();

    TRACE!(b"(%s, %p, %x, %p, %x, %p)\n\0".as_ptr() as *const c_char,
           debugstr_guid(riid as *const c_void), pv, dwDestContext, pvDestContext, mshlflags,
           pSize);

    *pSize = 0x10; /* to match native */

    hr = IMoniker_Enum(core::ptr::addr_of_mut!((*This).IMoniker_iface), 1 /* TRUE */,
                       &mut pEnumMk);
    if FAILED(hr) {
        return hr;
    }

    hr = IMoniker_GetSizeMax(core::ptr::addr_of_mut!((*This).IMoniker_iface), &mut size);

    while IEnumMoniker_Next(pEnumMk, 1, &mut pmk, core::ptr::null_mut()) == S_OK {
        let mut size: ULONG = 0;

        hr = CoGetMarshalSizeMax(&mut size, core::ptr::addr_of!(IID_IMoniker),
                                 pmk as *mut IUnknown, dwDestContext, pvDestContext,
                                 mshlflags);
        if SUCCEEDED(hr) {
            *pSize += size;
        }

        IMoniker_Release(pmk);

        if FAILED(hr) {
            IEnumMoniker_Release(pEnumMk);
            return hr;
        }
    }

    IEnumMoniker_Release(pEnumMk);

    S_OK
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_MarshalInterface(
    iface: *mut IMarshal,
    pStm: *mut IStream,
    riid: *const IID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    let mut pEnumMk: *mut IEnumMoniker = core::ptr::null_mut();
    let mut pmk: *mut IMoniker = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut i: ULONG = 0;

    TRACE!(b"(%p, %s, %p, %x, %p, %x)\n\0".as_ptr() as *const c_char, pStm,
           debugstr_guid(riid as *const c_void), pv, dwDestContext, pvDestContext, mshlflags);

    hr = IMoniker_Enum(core::ptr::addr_of_mut!((*This).IMoniker_iface), 1 /* TRUE */,
                       &mut pEnumMk);
    if FAILED(hr) {
        return hr;
    }

    while IEnumMoniker_Next(pEnumMk, 1, &mut pmk, core::ptr::null_mut()) == S_OK {
        hr = CoMarshalInterface(pStm, core::ptr::addr_of!(IID_IMoniker), pmk as *mut IUnknown,
                                dwDestContext, pvDestContext, mshlflags);

        IMoniker_Release(pmk);

        if FAILED(hr) {
            IEnumMoniker_Release(pEnumMk);
            return hr;
        }
        i += 1;
    }

    if i != 2 {
        FIXME!(b"moniker count of %d not supported\n\0".as_ptr() as *const c_char, i);
    }

    IEnumMoniker_Release(pEnumMk);

    S_OK
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_UnmarshalInterface(
    iface: *mut IMarshal,
    pStm: *mut IStream,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    let mut hr: HRESULT;

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr() as *const c_char, pStm,
           debugstr_guid(riid as *const c_void), ppv);

    CompositeMonikerImpl_ReleaseMonikersInTable(This);

    /* resize the table if needed */
    if (*This).tabLastIndex + 2 > (*This).tabSize {
        /* C: This->tabSize += max(BLOCK_TAB_SIZE, 2); */
        (*This).tabSize += if BLOCK_TAB_SIZE > 2 { BLOCK_TAB_SIZE } else { 2 };
        (*This).tabMoniker = HeapReAlloc(
            GetProcessHeap(),
            0,
            (*This).tabMoniker as LPVOID,
            ((*This).tabSize as usize * core::mem::size_of::<*mut IMoniker>()) as SIZE_T,
        ) as *mut *mut IMoniker;

        if (*This).tabMoniker.is_null() {
            return E_OUTOFMEMORY;
        }
    }

    hr = CoUnmarshalInterface(
        pStm,
        core::ptr::addr_of!(IID_IMoniker),
        (*This).tabMoniker.add((*This).tabLastIndex as usize) as *mut LPVOID,
    );
    if FAILED(hr) {
        ERR!(b"couldn't unmarshal moniker, hr = 0x%08x\n\0".as_ptr() as *const c_char, hr);
        return hr;
    }
    (*This).tabLastIndex += 1;
    hr = CoUnmarshalInterface(
        pStm,
        core::ptr::addr_of!(IID_IMoniker),
        (*This).tabMoniker.add((*This).tabLastIndex as usize) as *mut LPVOID,
    );
    if FAILED(hr) {
        ERR!(b"couldn't unmarshal moniker, hr = 0x%08x\n\0".as_ptr() as *const c_char, hr);
        return hr;
    }
    (*This).tabLastIndex += 1;

    IMoniker_QueryInterface(core::ptr::addr_of_mut!((*This).IMoniker_iface), riid, ppv)
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_ReleaseMarshalData(iface: *mut IMarshal, pStm: *mut IStream)
    -> HRESULT
{
    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, pStm);
    let _ = iface;
    /* can't release a state-based marshal as nothing on server side to release */
    S_OK
}
}

stdmethod! {
fn CompositeMonikerMarshalImpl_DisconnectObject(iface: *mut IMarshal, dwReserved: DWORD)
    -> HRESULT
{
    TRACE!(b"(0x%x)\n\0".as_ptr() as *const c_char, dwReserved);
    let _ = iface;
    /* can't disconnect a state-based marshal as nothing on server side to disconnect from */
    S_OK
}
}

// ---------------------------------------------------------------------------
// IEnumMoniker
// ---------------------------------------------------------------------------

stdmethod! {
fn EnumMonikerImpl_QueryInterface(
    iface: *mut IEnumMoniker,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let _This = impl_from_IEnumMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const c_char, This,
           debugstr_guid(riid as *const c_void), ppvObject);

    /* Perform a sanity check on the parameters.*/
    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    /* Initialize the return parameter */
    *ppvObject = core::ptr::null_mut();

    /* Compare the riid with the interface IDs implemented by this object.*/
    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IEnumMoniker), riid)
    {
        *ppvObject = iface as *mut c_void;
    }

    /* Check that we obtained an interface.*/
    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    /* QueryInterface always increases the reference count by one when it is successful */
    IEnumMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn EnumMonikerImpl_AddRef(iface: *mut IEnumMoniker) -> ULONG {
    let This = impl_from_IEnumMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, This);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn EnumMonikerImpl_Release(iface: *mut IEnumMoniker) -> ULONG {
    let This = impl_from_IEnumMoniker(iface);
    let mut i: ULONG;

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, This);

    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    /* destroy the object if there are no more references to it */
    if ref_ == 0 {
        i = 0;
        while i < (*This).tabSize {
            IMoniker_Release(*(*This).tabMoniker.add(i as usize));
            i += 1;
        }

        HeapFree(GetProcessHeap(), 0, (*This).tabMoniker as LPVOID);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_
}
}

stdmethod! {
fn EnumMonikerImpl_Next(
    iface: *mut IEnumMoniker,
    celt: ULONG,
    rgelt: *mut *mut IMoniker,
    pceltFethed: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);
    let mut i: ULONG = 0;

    // compositemoniker.c: `if (!m) return E_INVALIDARG; *m = NULL;`
    if rgelt.is_null() {
        return E_INVALIDARG;
    }
    *rgelt = core::ptr::null_mut();

    /* retrieve the requested number of monikers from the current position */
    while ((*This).currentPos < (*This).tabSize) && (i < celt) {
        *rgelt.add(i as usize) = *(*This).tabMoniker.add((*This).currentPos as usize);
        (*This).currentPos += 1;
        IMoniker_AddRef(*rgelt.add(i as usize));
        i += 1;
    }

    if !pceltFethed.is_null() {
        *pceltFethed = i;
    }

    if i == celt {
        S_OK
    } else {
        S_FALSE
    }
}
}

stdmethod! {
fn EnumMonikerImpl_Skip(iface: *mut IEnumMoniker, celt: ULONG) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    if ((*This).currentPos + celt) >= (*This).tabSize {
        return S_FALSE;
    }

    (*This).currentPos += celt;

    S_OK
}
}

stdmethod! {
fn EnumMonikerImpl_Reset(iface: *mut IEnumMoniker) -> HRESULT {
    let This = impl_from_IEnumMoniker(iface);

    (*This).currentPos = 0;

    S_OK
}
}

stdmethod! {
fn EnumMonikerImpl_Clone(iface: *mut IEnumMoniker, ppenum: *mut *mut IEnumMoniker)
    -> HRESULT
{
    let This = impl_from_IEnumMoniker(iface);

    EnumMonikerImpl_CreateEnumMoniker((*This).tabMoniker, (*This).tabSize, (*This).currentPos,
                                      1 /* TRUE */, ppenum)
}
}

unsafe fn EnumMonikerImpl_CreateEnumMoniker(
    tabMoniker: *mut *mut IMoniker,
    tabSize: ULONG,
    currentPos: ULONG,
    leftToRight: BOOL,
    ppmk: *mut *mut IEnumMoniker,
) -> HRESULT {
    let newEnumMoniker: *mut EnumMonikerImpl;
    let mut i: ULONG;

    if currentPos > tabSize {
        return E_INVALIDARG;
    }

    newEnumMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<EnumMonikerImpl>() as SIZE_T,
    ) as *mut EnumMonikerImpl;

    if newEnumMoniker.is_null() {
        return STG_E_INSUFFICIENTMEMORY;
    }

    /* Initialize the virtual function table. */
    (*newEnumMoniker).IEnumMoniker_iface.lpVtbl =
        core::ptr::addr_of!(VT_EnumMonikerImpl) as *mut IEnumMonikerVtbl;
    (*newEnumMoniker).ref_ = 1;

    (*newEnumMoniker).tabSize = tabSize;
    (*newEnumMoniker).currentPos = currentPos;

    (*newEnumMoniker).tabMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        (tabSize as usize * core::mem::size_of::<*mut IMoniker>()) as SIZE_T,
    ) as *mut *mut IMoniker;

    if (*newEnumMoniker).tabMoniker.is_null() {
        HeapFree(GetProcessHeap(), 0, newEnumMoniker as LPVOID);
        return E_OUTOFMEMORY;
    }

    if leftToRight != 0 {
        i = 0;
        while i < tabSize {
            *(*newEnumMoniker).tabMoniker.add(i as usize) = *tabMoniker.add(i as usize);
            IMoniker_AddRef(*tabMoniker.add(i as usize));
            i += 1;
        }
    } else {
        i = tabSize;
        while i > 0 {
            *(*newEnumMoniker).tabMoniker.add((tabSize - i) as usize) =
                *tabMoniker.add((i - 1) as usize);
            IMoniker_AddRef(*tabMoniker.add((i - 1) as usize));
            i -= 1;
        }
    }

    *ppmk = core::ptr::addr_of_mut!((*newEnumMoniker).IEnumMoniker_iface);

    S_OK
}

// ---------------------------------------------------------------------------
// vtbls
// ---------------------------------------------------------------------------

/// Virtual function table for the IEnumMoniker class
pub static VT_EnumMonikerImpl: IEnumMonikerVtbl = IEnumMonikerVtbl {
    QueryInterface: Some(EnumMonikerImpl_QueryInterface),
    AddRef: Some(EnumMonikerImpl_AddRef),
    Release: Some(EnumMonikerImpl_Release),
    Next: Some(EnumMonikerImpl_Next),
    Skip: Some(EnumMonikerImpl_Skip),
    Reset: Some(EnumMonikerImpl_Reset),
    Clone: Some(EnumMonikerImpl_Clone),
};

/// Virtual function table for the CompositeMonikerImpl class which includes IPersist,
/// IPersistStream and IMoniker functions.
pub static VT_CompositeMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: Some(CompositeMonikerImpl_QueryInterface),
    AddRef: Some(CompositeMonikerImpl_AddRef),
    Release: Some(CompositeMonikerImpl_Release),
    GetClassID: Some(CompositeMonikerImpl_GetClassID),
    IsDirty: Some(CompositeMonikerImpl_IsDirty),
    Load: Some(CompositeMonikerImpl_Load),
    Save: Some(CompositeMonikerImpl_Save),
    GetSizeMax: Some(CompositeMonikerImpl_GetSizeMax),
    BindToObject: Some(CompositeMonikerImpl_BindToObject),
    BindToStorage: Some(CompositeMonikerImpl_BindToStorage),
    Reduce: Some(CompositeMonikerImpl_Reduce),
    ComposeWith: Some(CompositeMonikerImpl_ComposeWith),
    Enum: Some(CompositeMonikerImpl_Enum),
    IsEqual: Some(CompositeMonikerImpl_IsEqual),
    Hash: Some(CompositeMonikerImpl_Hash),
    IsRunning: Some(CompositeMonikerImpl_IsRunning),
    GetTimeOfLastChange: Some(CompositeMonikerImpl_GetTimeOfLastChange),
    Inverse: Some(CompositeMonikerImpl_Inverse),
    CommonPrefixWith: Some(CompositeMonikerImpl_CommonPrefixWith),
    RelativePathTo: Some(CompositeMonikerImpl_RelativePathTo),
    GetDisplayName: Some(CompositeMonikerImpl_GetDisplayName),
    ParseDisplayName: Some(CompositeMonikerImpl_ParseDisplayName),
    IsSystemMoniker: Some(CompositeMonikerImpl_IsSystemMoniker),
};

/// Virtual function table for the IROTData class.
pub static VT_ROTDataImpl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: Some(CompositeMonikerROTDataImpl_QueryInterface),
    AddRef: Some(CompositeMonikerROTDataImpl_AddRef),
    Release: Some(CompositeMonikerROTDataImpl_Release),
    GetComparisonData: Some(CompositeMonikerROTDataImpl_GetComparisonData),
};

pub static VT_MarshalImpl: IMarshalVtbl = IMarshalVtbl {
    QueryInterface: Some(CompositeMonikerMarshalImpl_QueryInterface),
    AddRef: Some(CompositeMonikerMarshalImpl_AddRef),
    Release: Some(CompositeMonikerMarshalImpl_Release),
    GetUnmarshalClass: Some(CompositeMonikerMarshalImpl_GetUnmarshalClass),
    GetMarshalSizeMax: Some(CompositeMonikerMarshalImpl_GetMarshalSizeMax),
    MarshalInterface: Some(CompositeMonikerMarshalImpl_MarshalInterface),
    UnmarshalInterface: Some(CompositeMonikerMarshalImpl_UnmarshalInterface),
    ReleaseMarshalData: Some(CompositeMonikerMarshalImpl_ReleaseMarshalData),
    DisconnectObject: Some(CompositeMonikerMarshalImpl_DisconnectObject),
};

// ---------------------------------------------------------------------------
// construction / exports
// ---------------------------------------------------------------------------

/// Composite-Moniker_Construct (local function)
unsafe fn CompositeMonikerImpl_Construct(
    ppMoniker: *mut *mut IMoniker,
    pmkFirst: *mut IMoniker,
    pmkRest: *mut IMoniker,
) -> HRESULT {
    let mut mkSys: DWORD = 0;
    let mut enumMoniker: *mut IEnumMoniker = core::ptr::null_mut();
    let mut tempMk: *mut IMoniker = core::ptr::null_mut();
    let res: HRESULT;
    let This: *mut CompositeMonikerImpl;
    let mut i: c_int;

    This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<CompositeMonikerImpl>() as SIZE_T,
    ) as *mut CompositeMonikerImpl;

    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    TRACE!(b"(%p,%p,%p)\n\0".as_ptr() as *const c_char, This, pmkFirst, pmkRest);

    /* Initialize the virtual function table. */
    (*This).IMoniker_iface.lpVtbl =
        core::ptr::addr_of!(VT_CompositeMonikerImpl) as *mut IMonikerVtbl;
    (*This).IROTData_iface.lpVtbl = core::ptr::addr_of!(VT_ROTDataImpl) as *mut IROTDataVtbl;
    (*This).IMarshal_iface.lpVtbl = core::ptr::addr_of!(VT_MarshalImpl) as *mut IMarshalVtbl;
    (*This).ref_ = 1;

    (*This).tabSize = BLOCK_TAB_SIZE;
    (*This).tabLastIndex = 0;

    (*This).tabMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        ((*This).tabSize as usize * core::mem::size_of::<*mut IMoniker>()) as SIZE_T,
    ) as *mut *mut IMoniker;
    if (*This).tabMoniker.is_null() {
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
        return E_OUTOFMEMORY;
    }

    if pmkFirst.is_null() && pmkRest.is_null() {
        *ppMoniker = core::ptr::addr_of_mut!((*This).IMoniker_iface);
        return S_OK;
    }

    IMoniker_IsSystemMoniker(pmkFirst, &mut mkSys);

    /* put the first moniker contents in the beginning of the table */
    if mkSys != MKSYS_GENERICCOMPOSITE {
        *(*This).tabMoniker.add((*This).tabLastIndex as usize) = pmkFirst;
        (*This).tabLastIndex += 1;
        IMoniker_AddRef(pmkFirst);
    } else {
        IMoniker_Enum(pmkFirst, 1 /* TRUE */, &mut enumMoniker);

        while IEnumMoniker_Next(
            enumMoniker,
            1,
            (*This).tabMoniker.add((*This).tabLastIndex as usize),
            core::ptr::null_mut(),
        ) == S_OK
        {
            (*This).tabLastIndex += 1;
            if (*This).tabLastIndex == (*This).tabSize {
                let tab_moniker = (*This).tabMoniker;

                (*This).tabSize += BLOCK_TAB_SIZE;
                (*This).tabMoniker = HeapReAlloc(
                    GetProcessHeap(),
                    0,
                    (*This).tabMoniker as LPVOID,
                    ((*This).tabSize as usize * core::mem::size_of::<*mut IMoniker>()) as SIZE_T,
                ) as *mut *mut IMoniker;

                if (*This).tabMoniker.is_null() {
                    i = 0;
                    while i < (*This).tabLastIndex as c_int {
                        IMoniker_Release(*tab_moniker.offset(i as isize));
                        i += 1;
                    }
                    HeapFree(GetProcessHeap(), 0, tab_moniker as LPVOID);
                    HeapFree(GetProcessHeap(), 0, This as LPVOID);
                    return E_OUTOFMEMORY;
                }
            }
        }

        IEnumMoniker_Release(enumMoniker);
    }

    /* put the rest moniker contents after the first one and make simplification if needed */

    IMoniker_IsSystemMoniker(pmkRest, &mut mkSys);

    if mkSys != MKSYS_GENERICCOMPOSITE {
        /* add a simple moniker to the moniker table */
        res = IMoniker_ComposeWith(
            *(*This).tabMoniker.add(((*This).tabLastIndex - 1) as usize),
            pmkRest,
            1, /* TRUE */
            &mut tempMk,
        );

        if res == MK_E_NEEDGENERIC {
            /* there's no simplification in this case */
            *(*This).tabMoniker.add((*This).tabLastIndex as usize) = pmkRest;

            (*This).tabLastIndex += 1;

            IMoniker_AddRef(pmkRest);
        } else if tempMk.is_null() {
            /* we have an antimoniker after a simple moniker so we can make a simplification
             * in this case */
            IMoniker_Release(*(*This).tabMoniker.add(((*This).tabLastIndex - 1) as usize));

            (*This).tabLastIndex -= 1;
        } else if SUCCEEDED(res) {
            /* the non-generic composition was successful so we can make a simplification in
             * this case */
            IMoniker_Release(*(*This).tabMoniker.add(((*This).tabLastIndex - 1) as usize));

            *(*This).tabMoniker.add(((*This).tabLastIndex - 1) as usize) = tempMk;
        } else {
            i = 0;
            while i < (*This).tabLastIndex as c_int {
                IMoniker_Release(*(*This).tabMoniker.offset(i as isize));
                i += 1;
            }
            HeapFree(GetProcessHeap(), 0, (*This).tabMoniker as LPVOID);
            HeapFree(GetProcessHeap(), 0, This as LPVOID);
            return res;
        }

        /* resize tabMoniker if needed */
        if (*This).tabLastIndex == (*This).tabSize {
            let tab_moniker = (*This).tabMoniker;

            (*This).tabSize += BLOCK_TAB_SIZE;

            /* NOTE: the C sizes this one with sizeof(IMoniker) (not the element type) —
             * same value in practice (IMoniker is a single vtbl pointer). Kept verbatim. */
            (*This).tabMoniker = HeapReAlloc(
                GetProcessHeap(),
                0,
                (*This).tabMoniker as LPVOID,
                ((*This).tabSize as usize * core::mem::size_of::<IMoniker>()) as SIZE_T,
            ) as *mut *mut IMoniker;

            if (*This).tabMoniker.is_null() {
                i = 0;
                while i < (*This).tabLastIndex as c_int {
                    IMoniker_Release(*tab_moniker.offset(i as isize));
                    i += 1;
                }
                HeapFree(GetProcessHeap(), 0, tab_moniker as LPVOID);
                HeapFree(GetProcessHeap(), 0, This as LPVOID);
                return E_OUTOFMEMORY;
            }
        }
    } else {
        /* add a composite moniker to the moniker table (do the same thing for each moniker
         * within the composite moniker as a simple moniker (see above)) */
        IMoniker_Enum(pmkRest, 1 /* TRUE */, &mut enumMoniker);

        while IEnumMoniker_Next(
            enumMoniker,
            1,
            (*This).tabMoniker.add((*This).tabLastIndex as usize),
            core::ptr::null_mut(),
        ) == S_OK
        {
            let res2 = IMoniker_ComposeWith(
                *(*This).tabMoniker.add(((*This).tabLastIndex - 1) as usize),
                *(*This).tabMoniker.add((*This).tabLastIndex as usize),
                1, /* TRUE */
                &mut tempMk,
            );

            if res2 == MK_E_NEEDGENERIC {
                (*This).tabLastIndex += 1;
            } else if tempMk.is_null() {
                IMoniker_Release(*(*This).tabMoniker.add(((*This).tabLastIndex - 1) as usize));
                IMoniker_Release(*(*This).tabMoniker.add((*This).tabLastIndex as usize));
                (*This).tabLastIndex -= 1;
            } else {
                IMoniker_Release(*(*This).tabMoniker.add(((*This).tabLastIndex - 1) as usize));

                *(*This).tabMoniker.add(((*This).tabLastIndex - 1) as usize) = tempMk;
            }

            if (*This).tabLastIndex == (*This).tabSize {
                let tab_moniker = (*This).tabMoniker;

                (*This).tabSize += BLOCK_TAB_SIZE;

                (*This).tabMoniker = HeapReAlloc(
                    GetProcessHeap(),
                    0,
                    (*This).tabMoniker as LPVOID,
                    ((*This).tabSize as usize * core::mem::size_of::<*mut IMoniker>()) as SIZE_T,
                ) as *mut *mut IMoniker;

                if (*This).tabMoniker.is_null() {
                    i = 0;
                    while i < (*This).tabLastIndex as c_int {
                        IMoniker_Release(*tab_moniker.offset(i as isize));
                        i += 1;
                    }
                    HeapFree(GetProcessHeap(), 0, tab_moniker as LPVOID);
                    HeapFree(GetProcessHeap(), 0, This as LPVOID);
                    return E_OUTOFMEMORY;
                }
            }
        }

        IEnumMoniker_Release(enumMoniker);
    }

    /* only one moniker, then just return it */
    if (*This).tabLastIndex == 1 {
        *ppMoniker = *(*This).tabMoniker.add(0);
        IMoniker_AddRef(*ppMoniker);
        IMoniker_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface));
    } else {
        *ppMoniker = core::ptr::addr_of_mut!((*This).IMoniker_iface);
    }

    S_OK
}

/// CreateGenericComposite [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn CreateGenericComposite(
    pmkFirst: *mut IMoniker,
    pmkRest: *mut IMoniker,
    ppmkComposite: *mut *mut IMoniker,
) -> HRESULT {
    let mut moniker: *mut IMoniker = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(b"(%p,%p,%p)\n\0".as_ptr() as *const c_char, pmkFirst, pmkRest, ppmkComposite);

    if ppmkComposite.is_null() {
        return E_POINTER;
    }

    *ppmkComposite = core::ptr::null_mut();

    if pmkFirst.is_null() && !pmkRest.is_null() {
        *ppmkComposite = pmkRest;
        IMoniker_AddRef(pmkRest);
        return S_OK;
    } else if !pmkFirst.is_null() && pmkRest.is_null() {
        *ppmkComposite = pmkFirst;
        IMoniker_AddRef(pmkFirst);
        return S_OK;
    } else if pmkFirst.is_null() && pmkRest.is_null() {
        return S_OK;
    }

    hr = CompositeMonikerImpl_Construct(&mut moniker, pmkFirst, pmkRest);

    if FAILED(hr) {
        return hr;
    }

    hr = IMoniker_QueryInterface(
        moniker,
        core::ptr::addr_of!(IID_IMoniker),
        ppmkComposite as *mut *mut c_void,
    );
    IMoniker_Release(moniker);

    hr
}

/// MonikerCommonPrefixWith [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn MonikerCommonPrefixWith(
    pmkThis: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkCommon: *mut *mut IMoniker,
) -> HRESULT {
    FIXME!(b"(),stub!\n\0".as_ptr() as *const c_char);
    let _ = (pmkThis, pmkOther, ppmkCommon);

    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn CompositeMoniker_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let mut pMoniker: *mut IMoniker = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr() as *const c_char, pUnk,
           debugstr_guid(riid as *const c_void), ppv);

    let _ = iface;

    *ppv = core::ptr::null_mut();

    if !pUnk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    hr = CompositeMonikerImpl_Construct(&mut pMoniker, core::ptr::null_mut(),
                                        core::ptr::null_mut());

    if SUCCEEDED(hr) {
        hr = IMoniker_QueryInterface(pMoniker, riid, ppv);
        IMoniker_Release(pMoniker);
    }

    hr
}
