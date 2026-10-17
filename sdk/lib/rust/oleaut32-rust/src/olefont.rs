use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::olepropframe::CreateFontIndirectW;

extern "system" {
    fn EnumFontFamiliesExW( hdc: HDC, lpLogfont: *mut LOGFONTW, lpProc: Option<unsafe extern "system" fn(*const LOGFONTW, *const TEXTMETRICW, DWORD, LPARAM) -> i32>, lParam: LPARAM, dwFlags: DWORD, ) -> i32;
    fn GetTextFaceW(hdc: HDC, c: i32, lpName: LPWSTR) -> i32;
    fn GetTextMetricsW(hdc: HDC, lptm: *mut TEXTMETRICW) -> BOOL;
    fn MulDiv(nNumber: i32, nNumerator: i32, nDenominator: i32) -> i32;
    fn lstrcmpiW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> i32;
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: i32) -> LPWSTR;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_ole: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"ole\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_ole as *mut __wine_debug_channel };

static mut olefont_hdc: HDC = ::core::ptr::null::<HDC__>() as *mut HDC__;

static mut OLEFontImpl_hFontList: list = unsafe {
    list {
        next: &raw const OLEFontImpl_hFontList as *mut list,
        prev: &raw const OLEFontImpl_hFontList as *mut list,
    }
};

static mut ifont_cnt: LONG = 0 as LONG;

// olefont.c:85 —
//   static CRITICAL_SECTION_DEBUG OLEFontImpl_csHFONTLIST_debug =
//   { 0, 0, &OLEFontImpl_csHFONTLIST,
//     { &OLEFontImpl_csHFONTLIST_debug.ProcessLocksList,
//       &OLEFontImpl_csHFONTLIST_debug.ProcessLocksList },
//     0, 0, { (DWORD_PTR)(__FILE__ ": OLEFontImpl_csHFONTLIST") } };
// The C back-links the debug record to its critical section and SELF-LINKS
// ProcessLocksList (that is what makes it a valid, empty list head). The machine
// translation had nulled both; restored here. Same shape as variant.rs's critsect_debug.
static mut OLEFontImpl_csHFONTLIST_debug: CRITICAL_SECTION_DEBUG = CRITICAL_SECTION_DEBUG {
    Type: 0,
    CreatorBackTraceIndex: 0,
    CriticalSection: unsafe { &raw mut OLEFontImpl_csHFONTLIST },
    ProcessLocksList: LIST_ENTRY {
        Flink: unsafe { &raw mut OLEFontImpl_csHFONTLIST_debug.ProcessLocksList },
        Blink: unsafe { &raw mut OLEFontImpl_csHFONTLIST_debug.ProcessLocksList },
    },
    EntryCount: 0,
    ContentionCount: 0,
    // The trailing member is a union (ndk/rtltypes.h:1424-1443); the C initializes the
    // wine-debug-string arm. We have no `__FILE__ ": name"` literal address to hand here,
    // so zero it through that same arm (covers every alternative — it is pointer-sized).
    c2rust_unnamed: C2RustUnnamed_3 { WineDebugString: 0 },
};

static mut OLEFontImpl_csHFONTLIST: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw mut OLEFontImpl_csHFONTLIST_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as LONG),
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0,
    }
};

static mut STDFONT_CF: IClassFactoryImpl = unsafe {
    IClassFactoryImpl {
        IClassFactory_iface: IClassFactory {
            // bindgen spells the C's `CONST_VTBL IClassFactoryVtbl *` as `*mut` (it drops
            // the const). COM never writes through a vtbl pointer, so bridge with a cast
            // rather than making the vtbl static mut.
            lpVtbl: &raw const SFCF_Vtbl as *mut IClassFactoryVtbl,
        },
        ref_: 1 as LONG,
    }
};

// (c2rust emits an `INIT_ARRAY` + `run_static_initializers` pair when it thinks a
// static needs runtime init. Verified unnecessary here: every static in this module
// carries a complete const initializer — the self-referential list head uses
// `&raw const` — and the array had no `#[link_section]`/`#[used]` and was referenced
// nowhere, so it could never have run even if it compiled. Dropped, not stubbed.)

pub const FONTPERSIST_ITALIC: u32 = 0x02;
pub const FONTPERSIST_UNDERLINE: u32 = 0x04;
pub const FONTPERSIST_STRIKETHROUGH: u32 = 0x08;

#[repr(C)]
pub struct _HFONTItem {
    pub entry: list,
    pub int_refs: LONG,
    pub total_refs: LONG,
    pub gdiFont: HFONT,
}
pub type HFONTItem = _HFONTItem;
pub type PHFONTItem = *mut HFONTItem;

#[repr(C)]
pub struct OLEFontImpl {
    pub IFont_iface: IFont,
    pub IDispatch_iface: IDispatch,
    pub IPersistStream_iface: IPersistStream,
    pub IConnectionPointContainer_iface: IConnectionPointContainer,
    pub IPersistPropertyBag_iface: IPersistPropertyBag,
    pub IPersistStreamInit_iface: IPersistStreamInit,
    pub ref_: LONG,
    pub description: FONTDESC,
    pub gdiFont: HFONT,
    pub dirty: BOOL,
    pub cyLogical: LONG,
    pub cyHimetric: LONG,
    pub nRealHeight: LONG,
    pub pPropertyNotifyCP: *mut IConnectionPoint,
    pub pFontEventsCP: *mut IConnectionPoint,
}

#[repr(C)]
pub struct enum_data {
    pub orig_cs: i16,
    pub avail_cs: i16,
}

#[repr(C)]
pub struct IClassFactoryImpl {
    pub IClassFactory_iface: IClassFactory,
    pub ref_: LONG,
}
unsafe extern "C" fn get_dc() -> HDC {
    let hdc: HDC;
    EnterCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));
    if olefont_hdc.is_null() {
        olefont_hdc = CreateCompatibleDC(core::ptr::null_mut());
    }
    hdc = olefont_hdc;
    LeaveCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));
    return hdc;
}
unsafe extern "C" fn delete_dc() {
    EnterCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));
    if !olefont_hdc.is_null() {
        DeleteDC(olefont_hdc);
        olefont_hdc = core::ptr::null_mut();
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));
}
unsafe extern "C" fn HFONTItem_Delete(item: PHFONTItem) {

    DeleteObject((*item).gdiFont as HGDIOBJ);
    list_remove(core::ptr::addr_of_mut!((*item).entry));
    HeapFree(GetProcessHeap(), 0, item as LPVOID);
}
unsafe extern "C" fn find_hfontitem(hfont: HFONT) -> *mut HFONTItem {
    let mut item: *mut HFONTItem;
    
    item = (OLEFontImpl_hFontList.next as *mut u8).offset(
        -(core::mem::offset_of!(HFONTItem, entry) as isize)
    ) as *mut HFONTItem;
    
    while core::ptr::addr_of_mut!((*item).entry) != core::ptr::addr_of_mut!(OLEFontImpl_hFontList) {
        if (*item).gdiFont == hfont {
            return item;
        }
        item = ((*item).entry.next as *mut u8).offset(
            -(core::mem::offset_of!(HFONTItem, entry) as isize)
        ) as *mut HFONTItem;
    }
    
    core::ptr::null_mut()
}
unsafe extern "C" fn add_hfontitem(hfont: HFONT) -> HRESULT {
    let new_item = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<HFONTItem>() as SIZE_T,
    ) as *mut HFONTItem;
    
    if new_item.is_null() {
        return E_OUTOFMEMORY;
    }
    
    (*new_item).int_refs = 1;
    (*new_item).total_refs = 1;
    (*new_item).gdiFont = hfont;
    EnterCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));
    list_add_tail(core::ptr::addr_of_mut!(OLEFontImpl_hFontList), core::ptr::addr_of_mut!((*new_item).entry));
    LeaveCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));
    S_OK
}
unsafe extern "C" fn inc_int_ref(hfont: HFONT) -> HRESULT {
    let item: *mut HFONTItem;
    let mut hr: HRESULT = S_FALSE;

    EnterCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));
    item = find_hfontitem(hfont);

    if !item.is_null() {
        (*item).int_refs += 1;
        (*item).total_refs += 1;
        hr = S_OK;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));

    return hr;
}
unsafe extern "C" fn dec_int_ref(hfont: HFONT) -> HRESULT {
    let item: *mut HFONTItem;
    let mut hr: HRESULT = S_FALSE;

    EnterCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));
    item = find_hfontitem(hfont);

    if !item.is_null() {
        (*item).int_refs -= 1;
        (*item).total_refs -= 1;
        if (*item).int_refs == 0 && (*item).total_refs == 0 {
            HFONTItem_Delete(item);
        }
        hr = S_OK;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));

    return hr;
}
unsafe extern "C" fn inc_ext_ref(hfont: HFONT) -> HRESULT {
    let item: *mut HFONTItem;
    let mut hr: HRESULT = S_FALSE;
    EnterCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));
    item = find_hfontitem(hfont);
    if !item.is_null() {
        (*item).total_refs += 1;
        hr = S_OK;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));
    return hr;
}
unsafe extern "C" fn dec_ext_ref(hfont: HFONT) -> HRESULT {
    let item: *mut HFONTItem;
    let mut hr: HRESULT = S_FALSE;

    EnterCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));

    item = find_hfontitem(hfont);
    if !item.is_null() {
        (*item).total_refs -= 1;
        if (*item).total_refs >= 0 {
            hr = S_OK;
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(OLEFontImpl_csHFONTLIST));

    hr
}
unsafe extern "C" fn strdupW(str: *const WCHAR) -> *mut WCHAR {
    let ret: *mut WCHAR;
    let size: DWORD = ((lstrlenW(str as LPCWSTR) + 1) as usize * core::mem::size_of::<WCHAR>()) as DWORD;

    ret = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut WCHAR;
    if !ret.is_null() {
        memcpy(
            ret as *mut core::ffi::c_void,
            str as *const core::ffi::c_void,
            size as size_t,
        );
    }
    ret
}
unsafe extern "C" fn impl_from_IFont(iface: *mut IFont) -> *mut OLEFontImpl {
    (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(OLEFontImpl, IFont_iface) as ULONG_PTR
    ) as *mut OLEFontImpl
}
unsafe extern "C" fn impl_from_IDispatch(iface: *mut IDispatch) -> *mut OLEFontImpl {
    (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(OLEFontImpl, IDispatch_iface) as ULONG_PTR
    ) as *mut OLEFontImpl
}
unsafe extern "C" fn impl_from_IPersistStream(iface: *mut IPersistStream) -> *mut OLEFontImpl {
    (iface as DWORD_PTR).wrapping_sub(
        core::mem::offset_of!(OLEFontImpl, IPersistStream_iface) as DWORD_PTR
    ) as *mut OLEFontImpl
}
unsafe extern "C" fn impl_from_IConnectionPointContainer(
    iface: *mut IConnectionPointContainer,
) -> *mut OLEFontImpl {
    (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(OLEFontImpl, IConnectionPointContainer_iface) as ULONG_PTR,
    ) as *mut OLEFontImpl
}
unsafe extern "C" fn impl_from_IPersistPropertyBag(
    iface: *mut IPersistPropertyBag,
) -> *mut OLEFontImpl {
    let offset = core::mem::offset_of!(OLEFontImpl, IPersistPropertyBag_iface);
    (iface as *mut u8).sub(offset) as *mut OLEFontImpl
}
unsafe extern "C" fn impl_from_IPersistStreamInit(
    iface: *mut IPersistStreamInit,
) -> *mut OLEFontImpl {
    (iface as *mut u8).offset(
        -(core::mem::offset_of!(OLEFontImpl, IPersistStreamInit_iface) as isize)
    ) as *mut OLEFontImpl
}

#[no_mangle]
pub unsafe extern "system" fn OleCreateFontIndirect(
    mut lpFontDesc: LPFONTDESC,
    riid: *const IID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let newFont: *mut OLEFontImpl;
    let hr: HRESULT;
    let mut fd: FONTDESC = core::mem::zeroed();

    TRACE!(
        b"(%p, %s, %p)\n\0".as_ptr() as *const i8,
        lpFontDesc,
        debugstr_guid(riid),
        ppvObj,
    );

    if ppvObj.is_null() {
        return E_POINTER;
    }

    *ppvObj = core::ptr::null_mut();

    if lpFontDesc.is_null() {
        static mut fname: [u16; 7] = [
            b'S' as u16,
            b'y' as u16,
            b's' as u16,
            b't' as u16,
            b'e' as u16,
            b'm' as u16,
            0,
        ];

        fd.cbSizeofstruct = core::mem::size_of::<FONTDESC>() as u32;
        fd.lpstrName = (&raw mut fname).cast::<u16>();
        // C: fd.cySize.s.Lo / .Hi — `s` is CY's named struct arm (NONAMELESSUNION);
        // bindgen names the anonymous union member `__bindgen_anon_1`.
        fd.cySize.__bindgen_anon_1.Lo = 80000;
        fd.cySize.__bindgen_anon_1.Hi = 0;
        fd.sWeight = 0;
        fd.sCharset = 0;
        fd.fItalic = 0;
        fd.fUnderline = 0;
        fd.fStrikethrough = 0;
        lpFontDesc = core::ptr::addr_of_mut!(fd);
    } else if (*lpFontDesc).lpstrName.is_null() {
        // Wine 10.0, as Windows: a FONTDESC with no face name is rejected.
        // Before, OLEFontImpl_Construct strdupW'd the NULL and faulted.
        return CTL_E_INVALIDPROPERTYVALUE;
    }

    newFont = OLEFontImpl_Construct(lpFontDesc);
    if newFont.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = IFont_QueryInterface(
        core::ptr::addr_of_mut!((*newFont).IFont_iface),
        riid,
        ppvObj as *mut *mut core::ffi::c_void,
    );
    IFont_Release(core::ptr::addr_of_mut!((*newFont).IFont_iface));

    hr
}
unsafe extern "C" fn OLEFont_SendNotify(this: *mut OLEFontImpl, dispID: DISPID) {
    static wszName: [WCHAR; 5] = [
        'N' as WCHAR,
        'a' as WCHAR,
        'm' as WCHAR,
        'e' as WCHAR,
        0,
    ];
    static wszSize: [WCHAR; 5] = [
        'S' as WCHAR,
        'i' as WCHAR,
        'z' as WCHAR,
        'e' as WCHAR,
        0,
    ];
    static wszBold: [WCHAR; 5] = [
        'B' as WCHAR,
        'o' as WCHAR,
        'l' as WCHAR,
        'd' as WCHAR,
        0,
    ];
    static wszItalic: [WCHAR; 7] = [
        'I' as WCHAR,
        't' as WCHAR,
        'a' as WCHAR,
        'l' as WCHAR,
        'i' as WCHAR,
        'c' as WCHAR,
        0,
    ];
    static wszUnder: [WCHAR; 10] = [
        'U' as WCHAR,
        'n' as WCHAR,
        'd' as WCHAR,
        'e' as WCHAR,
        'r' as WCHAR,
        'l' as WCHAR,
        'i' as WCHAR,
        'n' as WCHAR,
        'e' as WCHAR,
        0,
    ];
    static wszStrike: [WCHAR; 14] = [
        'S' as WCHAR,
        't' as WCHAR,
        'r' as WCHAR,
        'i' as WCHAR,
        'k' as WCHAR,
        'e' as WCHAR,
        't' as WCHAR,
        'h' as WCHAR,
        'r' as WCHAR,
        'o' as WCHAR,
        'u' as WCHAR,
        'g' as WCHAR,
        'h' as WCHAR,
        0,
    ];
    static wszWeight: [WCHAR; 7] = [
        'W' as WCHAR,
        'e' as WCHAR,
        'i' as WCHAR,
        'g' as WCHAR,
        'h' as WCHAR,
        't' as WCHAR,
        0,
    ];
    static wszCharset: [WCHAR; 8] = [
        'C' as WCHAR,
        'h' as WCHAR,
        'a' as WCHAR,
        'r' as WCHAR,
        's' as WCHAR,
        'e' as WCHAR,
        't' as WCHAR,
        0,
    ];
    // C: `static const LPCWSTR dispid_mapping[] = {...}`. A `static` of raw pointers is not
    // Sync, so it cannot be a Rust item-static; a function-local `let` of the same read-only
    // pointer table is semantically identical (the pointees are still the statics above).
    let dispid_mapping: [LPCWSTR; 9] = [
        wszName.as_ptr(),
        core::ptr::null(),
        wszSize.as_ptr(),
        wszBold.as_ptr(),
        wszItalic.as_ptr(),
        wszUnder.as_ptr(),
        wszStrike.as_ptr(),
        wszWeight.as_ptr(),
        wszCharset.as_ptr(),
    ];

    let mut pEnum: *mut IEnumConnections = core::ptr::null_mut();
    let mut CD: CONNECTDATA = core::mem::zeroed();
    let mut hres: HRESULT;

    (*this).dirty = TRUE as BOOL;

    hres = IConnectionPoint_EnumConnections((*this).pPropertyNotifyCP, &mut pEnum);
    if SUCCEEDED(hres) {
        while IEnumConnections_Next(pEnum, 1, &mut CD, core::ptr::null_mut()) == S_OK {
            let mut sink: *mut IPropertyNotifySink = core::ptr::null_mut();

            IUnknown_QueryInterface(CD.pUnk, &IID_IPropertyNotifySink, &mut sink as *mut _ as *mut LPVOID);
            IPropertyNotifySink_OnChanged(sink, dispID);
            IPropertyNotifySink_Release(sink);
            IUnknown_Release(CD.pUnk);
        }
        IEnumConnections_Release(pEnum);
    }

    hres = IConnectionPoint_EnumConnections((*this).pFontEventsCP, &mut pEnum);
    if SUCCEEDED(hres) {
        let mut dispparams: DISPPARAMS = core::mem::zeroed();
        let mut vararg: VARIANTARG = core::mem::zeroed();

        VariantInit(&mut vararg);
        // C's V_VT/V_BSTR are lvalue macros; Rust spells the lvalue form as the `_ref` accessor.
        *V_VT_ref(&mut vararg) = VT_BSTR as VARTYPE;
        *V_BSTR_ref(&mut vararg) = SysAllocString(dispid_mapping[dispID as usize]);

        dispparams.cArgs = 1;
        dispparams.cNamedArgs = 0;
        dispparams.rgdispidNamedArgs = core::ptr::null_mut();
        dispparams.rgvarg = &mut vararg;

        while IEnumConnections_Next(pEnum, 1, &mut CD, core::ptr::null_mut()) == S_OK {
            let mut disp: *mut IFontEventsDisp = core::ptr::null_mut();

            IUnknown_QueryInterface(CD.pUnk, &IID_IFontEventsDisp, &mut disp as *mut _ as *mut LPVOID);
            IFontEventsDisp_Invoke(
                disp,
                DISPID_FONT_CHANGED,
                &IID_NULL,
                LOCALE_NEUTRAL,
                // INVOKE_FUNC is a ULONG; the vtbl slot's wFlags is a WORD. The C narrowed
                // implicitly at the call — make it explicit.
                INVOKE_FUNC as WORD,
                &mut dispparams,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            IFontEventsDisp_Release(disp);
            IUnknown_Release(CD.pUnk);
        }
        VariantClear(&mut vararg);
        IEnumConnections_Release(pEnum);
    }
}
stdmethod! {
fn OLEFontImpl_QueryInterface(
    iface: *mut IFont,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);

    TRACE!(
        b"(%p)->(%s, %p)\n\0".as_ptr() as *const i8,
        this,
        debugstr_guid(riid),
        ppvObject,
    );

    *ppvObject = core::ptr::null_mut();

    // IsEqualGUID returns `bool` here (winemacros.rs:194), not C's BOOL.
    if IsEqualGUID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualGUID(core::ptr::addr_of!(IID_IFont), riid)
    {
        *ppvObject = this as *mut core::ffi::c_void;
    } else if IsEqualGUID(core::ptr::addr_of!(IID_IDispatch), riid)
        || IsEqualGUID(core::ptr::addr_of!(IID_IFontDisp), riid)
    {
        *ppvObject = core::ptr::addr_of_mut!((*this).IDispatch_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID(core::ptr::addr_of!(IID_IPersist), riid)
        || IsEqualGUID(core::ptr::addr_of!(IID_IPersistStream), riid)
    {
        *ppvObject = core::ptr::addr_of_mut!((*this).IPersistStream_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID(core::ptr::addr_of!(IID_IConnectionPointContainer), riid)
    {
        *ppvObject = core::ptr::addr_of_mut!((*this).IConnectionPointContainer_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID(core::ptr::addr_of!(IID_IPersistPropertyBag), riid)
    {
        *ppvObject = core::ptr::addr_of_mut!((*this).IPersistPropertyBag_iface) as *mut core::ffi::c_void;
    }
    // Wine 10.0, as Windows: the font object does not expose
    // IPersistStreamInit (Wine's olefont test checks it is ABSENT). The
    // interface's vtable stays in the struct, unreachable.

    if (*ppvObject).is_null() {
        FIXME!(
            b"() : asking for unsupported interface %s\n\0".as_ptr() as *const i8,
            debugstr_guid(riid),
        );
        return E_NOINTERFACE;
    }

    IFont_AddRef(iface);

    return S_OK;
}
}
stdmethod! {
fn OLEFontImpl_AddRef(iface: *mut IFont) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(b"(%p)->(ref=%d)\n\0".as_ptr() as *const i8, this, (*this).ref_);
    return InterlockedIncrement(&mut (*this).ref_) as ULONG;
}
}
stdmethod! {
fn OLEFontImpl_Release(iface: *mut IFont) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    let ref_0: ULONG;

    TRACE!(
        b"(%p)->(ref=%d)\n\0".as_ptr() as *const i8,
        this,
        (*this).ref_,
    );

    ref_0 = InterlockedDecrement(&mut (*this).ref_) as ULONG;

    if ref_0 == 0 {
        let fontlist_refs: ULONG = InterlockedDecrement(&raw mut ifont_cnt) as ULONG;

        if fontlist_refs == 0 {
            let mut item: *mut HFONTItem;
            let mut cursor2: *mut HFONTItem;

            EnterCriticalSection(&raw mut OLEFontImpl_csHFONTLIST);
            
            item = (OLEFontImpl_hFontList.next as *mut u8).offset(
                -(core::mem::offset_of!(HFONTItem, entry) as isize)
            ) as *mut HFONTItem;
            cursor2 = ((*item).entry.next as *mut u8).offset(
                -(core::mem::offset_of!(HFONTItem, entry) as isize)
            ) as *mut HFONTItem;
            
            while core::ptr::addr_of_mut!((*item).entry) != core::ptr::addr_of_mut!(OLEFontImpl_hFontList) {
                HFONTItem_Delete(item as PHFONTItem);
                item = cursor2;
                cursor2 = ((*item).entry.next as *mut u8).offset(
                    -(core::mem::offset_of!(HFONTItem, entry) as isize)
                ) as *mut HFONTItem;
            }
            
            LeaveCriticalSection(&raw mut OLEFontImpl_csHFONTLIST);
            delete_dc();
        } else {
            dec_int_ref((*this).gdiFont);
        }
        OLEFontImpl_Destroy(this);
    }

    return ref_0;
}
}
// olefont.c:564 `static int CALLBACK font_enum_proc(...)`. This is a GDI FONTENUMPROCW
// callback, NOT a COM vtbl method, so it does not belong in `stdmethod!`. `extern "system"`
// is what CALLBACK means on every target (stdcall on i386, C on amd64/arm64) — ABI-identical
// to what stdmethod! expanded to, and it is the type EnumFontFamiliesExW's lpProc slot wants
// (rustc will not unify `extern "stdcall"` with `extern "system"` even where they coincide).
unsafe extern "system" fn font_enum_proc(
    elf: *const LOGFONTW,
    ntm: *const TEXTMETRICW,
    type_0: DWORD,
    lp: LPARAM,
) -> i32 {
    let _ = ntm;
    let _ = type_0;
    let data = lp as *mut enum_data;

    if (*elf).lfCharSet as i32 == (*data).orig_cs as i32 {
        (*data).avail_cs = (*data).orig_cs;
        return 0;
    }
    if (*data).avail_cs as i32 == -1 {
        (*data).avail_cs = (*elf).lfCharSet as i16;
    }
    return 1;
}
unsafe extern "C" fn realize_font(This: *mut OLEFontImpl) {
    let mut logFont: LOGFONTW = core::mem::zeroed();
    let fontHeight: INT;
    let mut text_face: [WCHAR; 32] = [0; 32];
    let hdc: HDC = get_dc();
    let mut old_font: HFONT;
    let mut tm: TEXTMETRICW = core::mem::zeroed();

    if (*This).dirty == 0 {
        return;
    }

    text_face[0] = 0;

    if !(*This).gdiFont.is_null() {
        old_font = SelectObject(hdc, (*This).gdiFont as HGDIOBJ) as HFONT;
        GetTextFaceW(
            hdc,
            (core::mem::size_of::<[WCHAR; 32]>() / core::mem::size_of::<WCHAR>()) as i32,
            text_face.as_mut_ptr(),
        );
        SelectObject(hdc, old_font as HGDIOBJ);
        dec_int_ref((*This).gdiFont);
        (*This).gdiFont = core::ptr::null_mut();
    }

    core::ptr::write_bytes(&mut logFont as *mut LOGFONTW, 0, 1);

    lstrcpynW(
        logFont.lfFaceName.as_mut_ptr(),
        (*This).description.lpstrName,
        LF_FACESIZE,
    );
    logFont.lfCharSet = (*This).description.sCharset as BYTE;

    if text_face[0] != 0 && lstrcmpiW(text_face.as_ptr(), (*This).description.lpstrName) != 0 {
        let mut data: enum_data = enum_data {
            orig_cs: (*This).description.sCharset,
            avail_cs: -1,
        };
        logFont.lfCharSet = DEFAULT_CHARSET as BYTE;
        EnumFontFamiliesExW(
            get_dc(),
            &mut logFont,
            Some(font_enum_proc),
            &mut data as *mut enum_data as LPARAM,
            0,
        );
        if data.avail_cs != -1 {
            logFont.lfCharSet = data.avail_cs as BYTE;
        }
    }

    fontHeight = MulDiv(
        (*This).description.cySize.__bindgen_anon_1.Lo as i32,
        (*This).cyLogical as i32 * 635,
        (*This).cyHimetric as i32 * 18,
    );

    logFont.lfHeight = if (fontHeight as i32 % 10000) > 5000 {
        (-fontHeight / 10000) - 1
    } else {
        -fontHeight / 10000
    };
    logFont.lfItalic = (*This).description.fItalic as BYTE;
    logFont.lfUnderline = (*This).description.fUnderline as BYTE;
    logFont.lfStrikeOut = (*This).description.fStrikethrough as BYTE;
    logFont.lfWeight = (*This).description.sWeight as LONG;
    logFont.lfOutPrecision = OUT_CHARACTER_PRECIS as BYTE;
    logFont.lfClipPrecision = CLIP_DEFAULT_PRECIS as BYTE;
    logFont.lfQuality = DEFAULT_QUALITY as BYTE;
    logFont.lfPitchAndFamily = DEFAULT_PITCH as BYTE;

    (*This).gdiFont = CreateFontIndirectW(&mut logFont);
    (*This).dirty = 0;

    add_hfontitem((*This).gdiFont);

    old_font = SelectObject(get_dc(), (*This).gdiFont as HGDIOBJ) as HFONT;
    GetTextFaceW(
        hdc,
        (core::mem::size_of::<[WCHAR; 32]>() / core::mem::size_of::<WCHAR>()) as i32,
        text_face.as_mut_ptr(),
    );
    if lstrcmpiW(text_face.as_ptr(), (*This).description.lpstrName) != 0 {
        HeapFree(
            GetProcessHeap(),
            0,
            (*This).description.lpstrName as LPVOID,
        );
        (*This).description.lpstrName = strdupW(text_face.as_ptr());
    }
    GetTextMetricsW(hdc, &mut tm);
    (*This).description.sCharset = tm.tmCharSet as SHORT;
    (*This).nRealHeight = tm.tmHeight - tm.tmInternalLeading;
    SelectObject(hdc, old_font as HGDIOBJ);
}
stdmethod! {
fn OLEFontImpl_get_Name(
    iface: *mut IFont,
    pname: *mut BSTR
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as PCHAR, this, pname);

    if pname.is_null() {
        return E_POINTER;
    }

    realize_font(this);

    if !(*this).description.lpstrName.is_null() {
        *pname = SysAllocString((*this).description.lpstrName);
    } else {
        *pname = core::ptr::null_mut();
    }

    S_OK
}
}
stdmethod! {
fn OLEFontImpl_put_Name(
    iface: *mut IFont,
    name: BSTR
) -> HRESULT {
    let This: *mut OLEFontImpl = impl_from_IFont(iface);
    
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                as *const u8 as *const i8,
            b"OLEFontImpl_put_Name\0" as *const u8 as *const i8,
            693i32,
            b"(%p)->(%p)\n\0" as *const u8 as *const i8,
            This,
            name,
        );
    }
    
    if name.is_null() {
        return CTL_E_INVALIDPROPERTYVALUE;
    }
    
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        (*This).description.lpstrName as PVOID,
    );
    
    (*This).description.lpstrName = strdupW(name as *const WCHAR) as LPOLESTR;
    
    if (*This).description.lpstrName.is_null() {
        return E_OUTOFMEMORY;
    }
    
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/oleaut32/olefont.c\0"
                as *const u8 as *const i8,
            b"OLEFontImpl_put_Name\0" as *const u8 as *const i8,
            702i32,
            b"new name %s\n\0" as *const u8 as *const i8,
            debugstr_w((*This).description.lpstrName as *const core::ffi::c_void),
        );
    }
    
    OLEFont_SendNotify(This, DISPID_FONT_NAME);
    S_OK
}
}
stdmethod! {
fn OLEFontImpl_get_Size(
    iface: *mut IFont,
    psize: *mut CY,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, this, psize);
    
    if psize.is_null() {
        return E_POINTER;
    }
    
    realize_font(this);
    
    (*psize).__bindgen_anon_1.Lo = MulDiv(
        (*this).nRealHeight,
        (*this).cyHimetric * 72 * 10000,
        (*this).cyLogical * 2540,
    ) as ULONG;
    (*psize).__bindgen_anon_1.Hi = 0;
    
    S_OK
}
}
stdmethod! {
fn OLEFontImpl_put_Size(
    iface: *mut IFont,
    size: CY
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(b"(%p)->(%d)\n\0".as_ptr() as PCHAR, this, size.__bindgen_anon_1.Lo);
    (*this).description.cySize.__bindgen_anon_1.Hi = 0;
    (*this).description.cySize.__bindgen_anon_1.Lo = size.__bindgen_anon_1.Lo;
    OLEFont_SendNotify(this, DISPID_FONT_SIZE);
    S_OK
}
}
stdmethod! {
fn OLEFontImpl_get_Bold(
    iface: *mut IFont,
    pbold: *mut BOOL,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, this, pbold);

    if pbold.is_null() {
        return E_POINTER;
    }

    realize_font(this);

    *pbold = if (*this).description.sWeight as i32 > 550 { 1 } else { 0 };

    S_OK
}
}
stdmethod! {
fn OLEFontImpl_put_Bold(
    iface: *mut IFont,
    bold: BOOL
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    
    if (*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_TRACE as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                as *const u8 as *const i8,
            b"OLEFontImpl_put_Bold\0" as *const u8 as *const i8,
            778i32,
            b"(%p)->(%d)\n\0" as *const u8 as *const i8,
            this,
            bold,
        );
    }
    
    (*this).description.sWeight = (if bold != 0 { FW_BOLD } else { FW_NORMAL }) as SHORT;
    OLEFont_SendNotify(this, DISPID_FONT_BOLD);
    
    S_OK
}
}
stdmethod! {
fn OLEFontImpl_get_Italic(
    iface: *mut IFont,
    pitalic: *mut BOOL,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, this, pitalic);

    if pitalic.is_null() {
        return E_POINTER;
    }

    realize_font(this);

    *pitalic = (*this).description.fItalic;

    return S_OK;
}
}
stdmethod! {
fn OLEFontImpl_put_Italic(
    iface: *mut IFont,
    italic: BOOL
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0".as_ptr() as *const i8,
            b"OLEFontImpl_put_Italic\0".as_ptr() as *const i8,
            813i32,
            b"(%p)->(%d)\n\0".as_ptr() as *const i8,
            this,
            italic,
        );
    }
    
    (*this).description.fItalic = italic;
    OLEFont_SendNotify(this, DISPID_FONT_ITALIC);
    
    S_OK
}
}
stdmethod! {
fn OLEFontImpl_get_Underline(
    iface: *mut IFont,
    punderline: *mut BOOL,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, this, punderline);

    if punderline.is_null() {
        return E_POINTER;
    }

    realize_font(this);

    *punderline = (*this).description.fUnderline;

    return S_OK;
}
}
stdmethod! {
fn OLEFontImpl_put_Underline(
    iface: *mut IFont,
    underline: BOOL,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    
    (*this).description.fUnderline = underline;
    OLEFont_SendNotify(this, DISPID_FONT_UNDER);
    
    S_OK
}
}
stdmethod! {
fn OLEFontImpl_get_Strikethrough(
    iface: *mut IFont,
    pstrikethrough: *mut BOOL,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, this, pstrikethrough);

    if pstrikethrough.is_null() {
        return E_POINTER;
    }

    realize_font(this);

    *pstrikethrough = (*this).description.fStrikethrough;

    return S_OK;
}
}
stdmethod! {
fn OLEFontImpl_put_Strikethrough(
    iface: *mut IFont,
    strikethrough: BOOL,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(b"(%p)->(%d)\n\0".as_ptr() as *const i8, this, strikethrough);

    (*this).description.fStrikethrough = strikethrough;
    OLEFont_SendNotify(this, DISPID_FONT_STRIKE);

    S_OK
}
}
stdmethod! {
fn OLEFontImpl_get_Weight(
    iface: *mut IFont,
    pweight: *mut i16,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(
        b"(%p)->(%p)\n\0".as_ptr() as *const i8,
        this,
        pweight,
    );

    if pweight.is_null() {
        return E_POINTER;
    }

    realize_font(this);

    *pweight = (*this).description.sWeight;

    return S_OK;
}
}
stdmethod! {
fn OLEFontImpl_put_Weight(
    iface: *mut IFont,
    weight: ::core::ffi::c_short,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"OLEFontImpl_put_Weight\0" as *const u8 as *const ::core::ffi::c_char,
            921 as ::core::ffi::c_int,
            b"(%p)->(%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            this,
            weight as ::core::ffi::c_int,
        );
    }
    
    (*this).description.sWeight = weight as SHORT;
    
    OLEFont_SendNotify(this, DISPID_FONT_WEIGHT);
    
    S_OK
}
}
stdmethod! {
fn OLEFontImpl_get_Charset(
    iface: *mut IFont,
    pcharset: *mut ::core::ffi::c_short,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as PCHAR, this, pcharset);

    if pcharset.is_null() {
        return E_POINTER;
    }

    realize_font(this);

    *pcharset = (*this).description.sCharset as ::core::ffi::c_short;

    return S_OK;
}
}
stdmethod! {
fn OLEFontImpl_put_Charset(
    iface: *mut IFont,
    charset: i16,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    
    TRACE!(b"(%p)->(%d)\n\0".as_ptr() as *const i8, this, charset as i32);
    
    (*this).description.sCharset = charset as SHORT;
    OLEFont_SendNotify(this, DISPID_FONT_CHARSET);
    
    S_OK
}
}
stdmethod! {
fn OLEFontImpl_get_hFont(
    iface: *mut IFont,
    phfont: *mut HFONT,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, this, phfont);
    
    if phfont.is_null() {
        return E_POINTER;
    }

    realize_font(this);

    *phfont = (*this).gdiFont;
    TRACE!(b"Returning %p\n\0".as_ptr() as *const i8, *phfont);
    
    S_OK
}
}
stdmethod! {
fn OLEFontImpl_Clone(
    iface: *mut IFont,
    ppfont: *mut *mut IFont,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    let newObject: *mut OLEFontImpl;

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, this, ppfont);

    if ppfont.is_null() {
        return E_POINTER;
    }

    *ppfont = core::ptr::null_mut();

    newObject = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<OLEFontImpl>() as SIZE_T,
    ) as *mut OLEFontImpl;
    if newObject.is_null() {
        return E_OUTOFMEMORY;
    }

    // C: `*newObject = *this;` — a bitwise struct copy. OLEFontImpl is not Copy in Rust
    // (it holds raw pointers/handles), so spell the memcpy explicitly. The shallow-copied
    // lpstrName is immediately replaced with its own buffer below, exactly as in the C.
    core::ptr::copy_nonoverlapping(this as *const OLEFontImpl, newObject, 1);
    (*newObject).description.lpstrName = strdupW((*this).description.lpstrName);

    if !(*newObject).gdiFont.is_null() {
        inc_int_ref((*newObject).gdiFont);
    }

    InterlockedIncrement(&raw mut ifont_cnt);

    (*newObject).pPropertyNotifyCP = core::ptr::null_mut();
    (*newObject).pFontEventsCP = core::ptr::null_mut();
    CreateConnectionPoint(
        &raw mut (*newObject).IFont_iface as *mut IUnknown,
        &raw const IID_IPropertyNotifySink,
        &raw mut (*newObject).pPropertyNotifyCP,
    );
    CreateConnectionPoint(
        &raw mut (*newObject).IFont_iface as *mut IUnknown,
        &raw const IID_IFontEventsDisp,
        &raw mut (*newObject).pFontEventsCP,
    );

    if (*newObject).pPropertyNotifyCP.is_null() || (*newObject).pFontEventsCP.is_null() {
        OLEFontImpl_Destroy(newObject);
        return E_OUTOFMEMORY;
    }

    (*newObject).ref_ = 1;

    *ppfont = &raw mut (*newObject).IFont_iface;

    S_OK
}
}
stdmethod! {
fn OLEFontImpl_IsEqual(
    iface: *mut IFont,
    pFontOther: *mut IFont,
) -> HRESULT {
    let left: *mut OLEFontImpl = impl_from_IFont(iface);
    let right: *mut OLEFontImpl = impl_from_IFont(pFontOther);
    let ret: INT;
    let left_len: INT;
    let right_len: INT;

    if pFontOther.is_null() {
        return E_POINTER;
    } else if (*left).description.cySize.__bindgen_anon_1.Lo
        != (*right).description.cySize.__bindgen_anon_1.Lo
    {
        return S_FALSE;
    } else if (*left).description.cySize.__bindgen_anon_1.Hi
        != (*right).description.cySize.__bindgen_anon_1.Hi
    {
        return S_FALSE;
    } else if (*left).description.sWeight as i32 != (*right).description.sWeight as i32 {
        return S_FALSE;
    } else if (*left).description.sCharset as i32 != (*right).description.sCharset as i32 {
        return S_FALSE;
    } else if (*left).description.fItalic != (*right).description.fItalic {
        return S_FALSE;
    } else if (*left).description.fUnderline != (*right).description.fUnderline {
        return S_FALSE;
    } else if (*left).description.fStrikethrough != (*right).description.fStrikethrough {
        return S_FALSE;
    }

    left_len = lstrlenW((*left).description.lpstrName as LPCWSTR) as INT;
    right_len = lstrlenW((*right).description.lpstrName as LPCWSTR) as INT;
    ret = CompareStringW(
        0 as LCID,
        0 as DWORD,
        (*left).description.lpstrName as LPCWSTR,
        left_len as i32,
        (*right).description.lpstrName as LPCWSTR,
        right_len as i32,
    ) as INT;
    
    if ret != CSTR_EQUAL {
        return S_FALSE;
    }

    S_OK
}
}
stdmethod! {
fn OLEFontImpl_SetRatio(
    iface: *mut IFont,
    cyLogical: LONG,
    cyHimetric: LONG,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    TRACE!(
        b"(%p)->(%d, %d)\n\0".as_ptr() as *const i8,
        this,
        cyLogical,
        cyHimetric,
    );

    if cyLogical == 0 || cyHimetric == 0 {
        return E_FAIL;
    }

    if cyLogical == 1 && cyHimetric == 1 {
        return S_OK;
    }

    (*this).cyLogical = cyLogical;
    (*this).cyHimetric = cyHimetric;
    (*this).dirty = TRUE as BOOL;

    S_OK
}
}
stdmethod! {
fn OLEFontImpl_QueryTextMetrics(
    iface: *mut IFont,
    ptm: *mut TEXTMETRICOLE,
) -> HRESULT {
    let hdcRef: HDC;
    let hOldFont: HFONT;
    let mut hNewFont: HFONT = core::ptr::null_mut();

    hdcRef = GetDC(core::ptr::null_mut());
    IFont_get_hFont(iface, &mut hNewFont);
    hOldFont = SelectObject(hdcRef, hNewFont as HGDIOBJ) as HFONT;
    GetTextMetricsW(hdcRef, ptm as LPTEXTMETRICW);
    SelectObject(hdcRef, hOldFont as HGDIOBJ);
    ReleaseDC(core::ptr::null_mut(), hdcRef);
    S_OK
}
}
stdmethod! {
fn OLEFontImpl_AddRefHfont(
    iface: *mut IFont,
    hfont: HFONT
) -> HRESULT {
    let _this: *mut OLEFontImpl = impl_from_IFont(iface);

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, this, hfont);

    if hfont.is_null() {
        return E_INVALIDARG;
    }

    return inc_ext_ref(hfont);
}
}
stdmethod! {
fn OLEFontImpl_ReleaseHfont(
    iface: *mut IFont,
    hfont: HFONT
) -> HRESULT {
    let _this: *mut OLEFontImpl = impl_from_IFont(iface);

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, this, hfont);

    if hfont.is_null() {
        return E_INVALIDARG;
    }

    return dec_ext_ref(hfont);
}
}
stdmethod! {
fn OLEFontImpl_SetHdc(
    iface: *mut IFont,
    hdc: HDC
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IFont(iface);
    if (*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_FIXME as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                as *const u8 as *const i8,
            b"OLEFontImpl_SetHdc\0" as *const u8 as *const i8,
            1160i32,
            b"(%p)->(%p): Stub\n\0" as *const u8 as *const i8,
            this,
            hdc,
        );
    }
    E_NOTIMPL
}
}
stdmethod! {
fn OLEFontImpl_IDispatch_QueryInterface(
    iface: *mut IDispatch,
    riid: *const IID,
    // C: `VOID** ppvoid` — one level of indirection, matching the IDispatchVtbl
    // QueryInterface slot (`ppvObject: *mut *mut c_void`). The translation had an extra `*`.
    ppvoid: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IDispatch(iface);
    return IFont_QueryInterface(core::ptr::addr_of_mut!((*this).IFont_iface), riid, ppvoid);
}
}
stdmethod! {
fn OLEFontImpl_IDispatch_Release(
    iface: *mut IDispatch
) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IDispatch(iface);
    IFont_Release(core::ptr::addr_of_mut!((*this).IFont_iface))
}
}
stdmethod! {
fn OLEFontImpl_IDispatch_AddRef(iface: *mut IDispatch) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IDispatch(iface);
    return IFont_AddRef(core::ptr::addr_of_mut!((*this).IFont_iface));
}
}
stdmethod! {
fn OLEFontImpl_GetTypeInfoCount(
    iface: *mut IDispatch,
    pctinfo: *mut ::core::ffi::c_uint,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IDispatch(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"OLEFontImpl_GetTypeInfoCount\0" as *const u8 as *const ::core::ffi::c_char,
            1235 as ::core::ffi::c_int,
            b"(%p)->(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            this,
            pctinfo,
        );
    }
    *pctinfo = 1 as ::core::ffi::c_uint;
    return S_OK;
}
}
stdmethod! {
fn OLEFontImpl_GetTypeInfo(
    iface: *mut IDispatch,
    iTInfo: UINT,
    lcid: LCID,
    ppTInfo: *mut *mut ITypeInfo,
) -> HRESULT {
    static mut stdole2tlb: [WCHAR; 12] = [
        's' as u16,
        't' as u16,
        'd' as u16,
        'o' as u16,
        'l' as u16,
        'e' as u16,
        '2' as u16,
        '.' as u16,
        't' as u16,
        'l' as u16,
        'b' as u16,
        0 as u16,
    ];
    let mut tl: *mut ITypeLib = core::ptr::null_mut();
    let mut hres: HRESULT;

    let _this = impl_from_IDispatch(iface);
    TRACE!(
        b"(%p, iTInfo=%d, lcid=%04x, %p)\n\0".as_ptr() as *const i8,
        this,
        iTInfo,
        lcid as i32,
        ppTInfo,
    );

    if iTInfo != 0 {
        return E_FAIL;
    }

    // The crate carries two ABI-identical `ITypeLib` COM shims (com_sys vs olesys):
    // LoadTypeLib is typed against olesys', while `tl` and the ITypeLib_* accessors are
    // com_sys'. Bridge with a raw-pointer cast — same COM object, translation-surface split
    // only. Same treatment as oleaut.rs:746.
    hres = LoadTypeLib(
        core::ptr::addr_of!(stdole2tlb) as LPCWSTR,
        core::ptr::addr_of_mut!(tl) as *mut *mut crate::olesys::ITypeLib,
    );
    if FAILED(hres) {
        ERR!(b"Could not load the stdole2.tlb?\n\0".as_ptr() as *const i8);
        return hres;
    }

    hres = ITypeLib_GetTypeInfoOfGuid(tl, core::ptr::addr_of!(IID_IFontDisp), ppTInfo);
    ITypeLib_Release(tl);

    if FAILED(hres) {
        FIXME!(
            b"Did not IDispatch typeinfo from typelib, hres %x\n\0".as_ptr() as *const i8,
            hres,
        );
    }

    hres
}
}
stdmethod! {
fn OLEFontImpl_GetIDsOfNames(
    iface: *mut IDispatch,
    riid: *const IID,
    rgszNames: *mut LPOLESTR,
    cNames: UINT,
    lcid: LCID,
    rgDispId: *mut DISPID,
) -> HRESULT {
    let mut pTInfo: *mut ITypeInfo = core::ptr::null_mut();
    let mut hres: HRESULT;

    let _this = impl_from_IDispatch(iface);

    TRACE!(
        b"(%p,%s,%p,cNames=%d,lcid=%04x,%p)\n\0".as_ptr() as *const i8,
        this,
        debugstr_guid(riid),
        rgszNames,
        cNames,
        lcid as i32,
        rgDispId,
    );

    if cNames == 0 {
        return E_INVALIDARG;
    }

    hres = IDispatch_GetTypeInfo(iface, 0, lcid, &mut pTInfo);
    if FAILED(hres) {
        ERR!(b"GetTypeInfo failed.\n\0".as_ptr() as *const i8);
        return hres;
    }

    // Same com_sys/olesys ITypeInfo surface split as GetTypeInfo above.
    hres = DispGetIDsOfNames(
        pTInfo as *mut crate::olesys::ITypeInfo,
        rgszNames,
        cNames,
        rgDispId,
    );
    ITypeInfo_Release(pTInfo);

    return hres;
}
}
stdmethod! {
fn OLEFontImpl_Invoke(
    iface: *mut IDispatch,
    dispIdMember: DISPID,
    riid: *const IID,
    lcid: LCID,
    wFlags: WORD,
    pDispParams: *mut DISPPARAMS,
    pVarResult: *mut VARIANT,
    pExepInfo: *mut EXCEPINFO,
    puArgErr: *mut UINT,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IDispatch(iface);
    let mut hr: HRESULT;

    TRACE!(
        b"%p->(%d,%s,0x%x,0x%x,%p,%p,%p,%p)\n\0".as_ptr() as *const ::core::ffi::c_char,
        this,
        dispIdMember,
        debugstr_guid(riid as *const CLSID),
        lcid,
        wFlags as ::core::ffi::c_int,
        pDispParams,
        pVarResult,
        pExepInfo,
        puArgErr,
    );

    if !IsEqualIID(riid, &GUID_NULL as *const IID) {
        ERR!(
            b"riid was %s instead of IID_NULL\n\0".as_ptr() as *const ::core::ffi::c_char,
            debugstr_guid(riid as *const CLSID),
        );
        return DISP_E_UNKNOWNINTERFACE;
    }

    if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
        if pVarResult.is_null() {
            ERR!(
                b"null pVarResult not allowed when DISPATCH_PROPERTYGET specified\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            return DISP_E_PARAMNOTOPTIONAL;
        }
    // NB: DISPATCH_PROPERTYGET is a `u32` (consts.rs:15) but DISPATCH_PROPERTYPUT is a
    // `c_int` (tierd_consts.rs:171) — hence the differing casts. See report.
    } else if wFlags as ::core::ffi::c_int & DISPATCH_PROPERTYPUT != 0 {
        if pDispParams.is_null() {
            ERR!(
                b"null pDispParams not allowed when DISPATCH_PROPERTYPUT specified\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            return DISP_E_PARAMNOTOPTIONAL;
        }
        if (*pDispParams).cArgs != 1 {
            ERR!(
                b"param count for DISPATCH_PROPERTYPUT was %d instead of 1\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
                (*pDispParams).cArgs,
            );
            return DISP_E_BADPARAMCOUNT;
        }
    } else {
        ERR!(
            b"one of DISPATCH_PROPERTYGET or DISPATCH_PROPERTYPUT must be specified\n\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return DISP_E_MEMBERNOTFOUND;
    }

    match dispIdMember {
        DISPID_FONT_NAME => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                // C's `&V_BSTR(pVarResult)` takes the address of an lvalue macro; the Rust
                // equivalent is the `_ref` accessor, not addr_of! on an rvalue temporary.
                *V_VT_ref(pVarResult) = VT_BSTR as ::core::ffi::c_int as VARTYPE;
                return IFont_get_Name(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_BSTR_ref(pVarResult),
                );
            } else {
                let mut vararg: VARIANT = core::mem::zeroed();
                VariantInit(core::ptr::addr_of_mut!(vararg));
                hr = VariantChangeTypeEx(
                    core::ptr::addr_of_mut!(vararg),
                    (*pDispParams).rgvarg.offset(0),
                    lcid,
                    0,
                    VT_BSTR as ::core::ffi::c_int as VARTYPE,
                );
                if FAILED(hr) {
                    return hr;
                }
                hr = IFont_put_Name(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_BSTR(core::ptr::addr_of_mut!(vararg)),
                );
                VariantClear(core::ptr::addr_of_mut!(vararg));
                return hr;
            }
        }
        DISPID_FONT_BOLD => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                let mut value: BOOL = 0;
                hr = IFont_get_Bold(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    core::ptr::addr_of_mut!(value),
                );
                *V_VT_ref(pVarResult) = VT_BOOL as ::core::ffi::c_int as VARTYPE;
                *V_BOOL_ref(pVarResult) = if value != 0 {
                    VARIANT_TRUE as ::core::ffi::c_int
                } else {
                    VARIANT_FALSE as ::core::ffi::c_int
                } as VARIANT_BOOL;
                return hr;
            } else {
                let mut vararg: VARIANT = core::mem::zeroed();
                VariantInit(core::ptr::addr_of_mut!(vararg));
                hr = VariantChangeTypeEx(
                    core::ptr::addr_of_mut!(vararg),
                    (*pDispParams).rgvarg.offset(0),
                    lcid,
                    0,
                    VT_BOOL as ::core::ffi::c_int as VARTYPE,
                );
                if FAILED(hr) {
                    return hr;
                }
                hr = IFont_put_Bold(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_BOOL(core::ptr::addr_of_mut!(vararg)) as BOOL,
                );
                VariantClear(core::ptr::addr_of_mut!(vararg));
                return hr;
            }
        }
        DISPID_FONT_ITALIC => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                let mut value: BOOL = 0;
                hr = IFont_get_Italic(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    core::ptr::addr_of_mut!(value),
                );
                *V_VT_ref(pVarResult) = VT_BOOL as ::core::ffi::c_int as VARTYPE;
                *V_BOOL_ref(pVarResult) = if value != 0 {
                    VARIANT_TRUE as ::core::ffi::c_int
                } else {
                    VARIANT_FALSE as ::core::ffi::c_int
                } as VARIANT_BOOL;
                return hr;
            } else {
                let mut vararg: VARIANT = core::mem::zeroed();
                VariantInit(core::ptr::addr_of_mut!(vararg));
                hr = VariantChangeTypeEx(
                    core::ptr::addr_of_mut!(vararg),
                    (*pDispParams).rgvarg.offset(0),
                    lcid,
                    0,
                    VT_BOOL as ::core::ffi::c_int as VARTYPE,
                );
                if FAILED(hr) {
                    return hr;
                }
                hr = IFont_put_Italic(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_BOOL(core::ptr::addr_of_mut!(vararg)) as BOOL,
                );
                VariantClear(core::ptr::addr_of_mut!(vararg));
                return hr;
            }
        }
        DISPID_FONT_UNDER => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                let mut value: BOOL = 0;
                hr = IFont_get_Underline(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    core::ptr::addr_of_mut!(value),
                );
                *V_VT_ref(pVarResult) = VT_BOOL as ::core::ffi::c_int as VARTYPE;
                *V_BOOL_ref(pVarResult) = if value != 0 {
                    VARIANT_TRUE as ::core::ffi::c_int
                } else {
                    VARIANT_FALSE as ::core::ffi::c_int
                } as VARIANT_BOOL;
                return hr;
            } else {
                let mut vararg: VARIANT = core::mem::zeroed();
                VariantInit(core::ptr::addr_of_mut!(vararg));
                hr = VariantChangeTypeEx(
                    core::ptr::addr_of_mut!(vararg),
                    (*pDispParams).rgvarg.offset(0),
                    lcid,
                    0,
                    VT_BOOL as ::core::ffi::c_int as VARTYPE,
                );
                if FAILED(hr) {
                    return hr;
                }
                hr = IFont_put_Underline(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_BOOL(core::ptr::addr_of_mut!(vararg)) as BOOL,
                );
                VariantClear(core::ptr::addr_of_mut!(vararg));
                return hr;
            }
        }
        DISPID_FONT_STRIKE => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                let mut value: BOOL = 0;
                hr = IFont_get_Strikethrough(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    core::ptr::addr_of_mut!(value),
                );
                *V_VT_ref(pVarResult) = VT_BOOL as ::core::ffi::c_int as VARTYPE;
                *V_BOOL_ref(pVarResult) = if value != 0 {
                    VARIANT_TRUE as ::core::ffi::c_int
                } else {
                    VARIANT_FALSE as ::core::ffi::c_int
                } as VARIANT_BOOL;
                return hr;
            } else {
                let mut vararg: VARIANT = core::mem::zeroed();
                VariantInit(core::ptr::addr_of_mut!(vararg));
                hr = VariantChangeTypeEx(
                    core::ptr::addr_of_mut!(vararg),
                    (*pDispParams).rgvarg.offset(0),
                    lcid,
                    0,
                    VT_BOOL as ::core::ffi::c_int as VARTYPE,
                );
                if FAILED(hr) {
                    return hr;
                }
                hr = IFont_put_Strikethrough(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_BOOL(core::ptr::addr_of_mut!(vararg)) as BOOL,
                );
                VariantClear(core::ptr::addr_of_mut!(vararg));
                return hr;
            }
        }
        DISPID_FONT_SIZE => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                *V_VT_ref(pVarResult) = VT_CY as ::core::ffi::c_int as VARTYPE;
                return IFont_get_Size(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_CY_ref(pVarResult),
                );
            } else {
                let mut vararg: VARIANT = core::mem::zeroed();
                VariantInit(core::ptr::addr_of_mut!(vararg));
                hr = VariantChangeTypeEx(
                    core::ptr::addr_of_mut!(vararg),
                    (*pDispParams).rgvarg.offset(0),
                    lcid,
                    0,
                    VT_CY as ::core::ffi::c_int as VARTYPE,
                );
                if FAILED(hr) {
                    return hr;
                }
                hr = IFont_put_Size(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_CY(core::ptr::addr_of_mut!(vararg)),
                );
                VariantClear(core::ptr::addr_of_mut!(vararg));
                return hr;
            }
        }
        DISPID_FONT_WEIGHT => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                *V_VT_ref(pVarResult) = VT_I2 as ::core::ffi::c_int as VARTYPE;
                return IFont_get_Weight(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_I2_ref(pVarResult),
                );
            } else {
                let mut vararg: VARIANT = core::mem::zeroed();
                VariantInit(core::ptr::addr_of_mut!(vararg));
                hr = VariantChangeTypeEx(
                    core::ptr::addr_of_mut!(vararg),
                    (*pDispParams).rgvarg.offset(0),
                    lcid,
                    0,
                    VT_I2 as ::core::ffi::c_int as VARTYPE,
                );
                if FAILED(hr) {
                    return hr;
                }
                hr = IFont_put_Weight(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_I2(core::ptr::addr_of_mut!(vararg)),
                );
                VariantClear(core::ptr::addr_of_mut!(vararg));
                return hr;
            }
        }
        DISPID_FONT_CHARSET => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                *V_VT_ref(pVarResult) = VT_I2 as ::core::ffi::c_int as VARTYPE;
                return OLEFontImpl_get_Charset(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_I2_ref(pVarResult),
                );
            } else {
                let mut vararg: VARIANT = core::mem::zeroed();
                VariantInit(core::ptr::addr_of_mut!(vararg));
                hr = VariantChangeTypeEx(
                    core::ptr::addr_of_mut!(vararg),
                    (*pDispParams).rgvarg.offset(0),
                    lcid,
                    0,
                    VT_I2 as ::core::ffi::c_int as VARTYPE,
                );
                if FAILED(hr) {
                    return hr;
                }
                hr = IFont_put_Charset(
                    core::ptr::addr_of_mut!((*this).IFont_iface),
                    V_I2(core::ptr::addr_of_mut!(vararg)),
                );
                VariantClear(core::ptr::addr_of_mut!(vararg));
                return hr;
            }
        }
        _ => {
            ERR!(
                b"member not found for dispid 0x%x\n\0".as_ptr() as *const ::core::ffi::c_char,
                dispIdMember,
            );
            return DISP_E_MEMBERNOTFOUND;
        }
    }
}
}
stdmethod! {
fn OLEFontImpl_IPersistStream_QueryInterface(
    iface: *mut IPersistStream,
    riid: *const IID,
    ppvoid: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IPersistStream(iface);
    return IFont_QueryInterface(core::ptr::addr_of_mut!((*this).IFont_iface), riid, ppvoid);
}
}
stdmethod! {
fn OLEFontImpl_IPersistStream_Release(iface: *mut IPersistStream) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IPersistStream(iface);
    return IFont_Release(core::ptr::addr_of_mut!((*this).IFont_iface));
}
}
stdmethod! {
fn OLEFontImpl_IPersistStream_AddRef(iface: *mut IPersistStream) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IPersistStream(iface);
    return IFont_AddRef(core::ptr::addr_of_mut!((*this).IFont_iface));
}
}
stdmethod! {
fn OLEFontImpl_GetClassID(
    iface: *mut IPersistStream,
    pClassID: *mut CLSID,
) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pClassID);
    
    if pClassID.is_null() {
        return E_POINTER;
    }
    
    *pClassID = CLSID_StdFont;
    
    return S_OK;
}
}
stdmethod! {
fn OLEFontImpl_IsDirty(
    iface: *mut IPersistStream
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                as *const u8 as *const i8,
            b"OLEFontImpl_IsDirty\0" as *const u8 as *const i8,
            1596i32,
            b"(%p)\n\0" as *const u8 as *const i8,
            iface,
        );
    }
    return S_OK;
}
}
stdmethod! {
fn OLEFontImpl_Load(
    iface: *mut IPersistStream,
    pLoadStream: *mut IStream,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IPersistStream(iface);
    let mut version: u8 = 0;
    let mut attributes: u8 = 0;
    let mut string_size: u8 = 0;
    let mut readBuffer: [i8; 256] = [0; 256];
    let mut cbRead: ULONG = 0;
    let len: i32;

    IStream_Read(
        pLoadStream,
        core::ptr::addr_of_mut!(version) as LPVOID,
        core::mem::size_of::<u8>() as ULONG,
        core::ptr::addr_of_mut!(cbRead),
    );
    if cbRead as usize != core::mem::size_of::<u8>() || version != 0x01 {
        return E_FAIL;
    }

    IStream_Read(
        pLoadStream,
        core::ptr::addr_of_mut!((*this).description.sCharset) as LPVOID,
        core::mem::size_of::<u16>() as ULONG,
        core::ptr::addr_of_mut!(cbRead),
    );
    if cbRead as usize != core::mem::size_of::<u16>() {
        return E_FAIL;
    }

    IStream_Read(
        pLoadStream,
        core::ptr::addr_of_mut!(attributes) as LPVOID,
        core::mem::size_of::<u8>() as ULONG,
        core::ptr::addr_of_mut!(cbRead),
    );
    if cbRead as usize != core::mem::size_of::<u8>() {
        return E_FAIL;
    }

    // FONTPERSIST_* are u32 consts; `attributes` is a BYTE. C promoted both to int.
    (*this).description.fItalic = if (attributes as u32 & FONTPERSIST_ITALIC) != 0 { 1 } else { 0 };
    (*this).description.fStrikethrough = if (attributes as u32 & FONTPERSIST_STRIKETHROUGH) != 0 { 1 } else { 0 };
    (*this).description.fUnderline = if (attributes as u32 & FONTPERSIST_UNDERLINE) != 0 { 1 } else { 0 };

    IStream_Read(
        pLoadStream,
        core::ptr::addr_of_mut!((*this).description.sWeight) as LPVOID,
        core::mem::size_of::<u16>() as ULONG,
        core::ptr::addr_of_mut!(cbRead),
    );
    if cbRead as usize != core::mem::size_of::<u16>() {
        return E_FAIL;
    }

    IStream_Read(
        pLoadStream,
        core::ptr::addr_of_mut!((*this).description.cySize.__bindgen_anon_1.Lo) as LPVOID,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::addr_of_mut!(cbRead),
    );
    if cbRead as usize != core::mem::size_of::<DWORD>() {
        return E_FAIL;
    }

    (*this).description.cySize.__bindgen_anon_1.Hi = 0;

    IStream_Read(
        pLoadStream,
        core::ptr::addr_of_mut!(string_size) as LPVOID,
        core::mem::size_of::<u8>() as ULONG,
        core::ptr::addr_of_mut!(cbRead),
    );
    if cbRead as usize != core::mem::size_of::<u8>() {
        return E_FAIL;
    }

    IStream_Read(
        pLoadStream,
        readBuffer.as_mut_ptr() as LPVOID,
        string_size as ULONG,
        core::ptr::addr_of_mut!(cbRead),
    );
    if cbRead != string_size as u32 {
        return E_FAIL;
    }

    HeapFree(
        GetProcessHeap(),
        0,
        (*this).description.lpstrName as LPVOID,
    );

    len = MultiByteToWideChar(
        CP_ACP,
        0,
        readBuffer.as_ptr(),
        string_size as i32,
        core::ptr::null_mut(),
        0,
    );
    (*this).description.lpstrName = HeapAlloc(
        GetProcessHeap(),
        0,
        (((len + 1) as usize) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPOLESTR;
    MultiByteToWideChar(
        CP_ACP,
        0,
        readBuffer.as_ptr(),
        string_size as i32,
        (*this).description.lpstrName,
        len,
    );
    *(*this).description.lpstrName.offset(len as isize) = 0;

    dec_int_ref((*this).gdiFont);
    (*this).gdiFont = core::ptr::null_mut();
    (*this).dirty = 1;

    S_OK
}
}
stdmethod! {
fn OLEFontImpl_Save(
    iface: *mut IPersistStream,
    pOutStream: *mut IStream,
    fClearDirty: BOOL,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IPersistStream(iface);
    let mut attributes: BYTE;
    let string_size: BYTE;
    let version: BYTE = 0x01;
    let writeBuffer: *mut i8;
    let mut written: ULONG = 0;

    TRACE!(
        b"(%p)->(%p %d)\n\0".as_ptr() as *const i8,
        this,
        pOutStream,
        fClearDirty,
    );

    IStream_Write(
        pOutStream,
        &version as *const BYTE as *const core::ffi::c_void,
        core::mem::size_of::<BYTE>() as ULONG,
        &mut written,
    );
    if written as usize != core::mem::size_of::<BYTE>() {
        return E_FAIL;
    }

    IStream_Write(
        pOutStream,
        // description.sCharset is a SHORT (i16), not a u16 — take its address at its real
        // type, then erase to void* like the C did.
        core::ptr::addr_of!((*this).description.sCharset) as *const core::ffi::c_void,
        core::mem::size_of::<WORD>() as ULONG,
        &mut written,
    );
    if written as usize != core::mem::size_of::<WORD>() {
        return E_FAIL;
    }

    attributes = 0;

    if (*this).description.fItalic != 0 {
        attributes |= FONTPERSIST_ITALIC as BYTE;
    }

    if (*this).description.fStrikethrough != 0 {
        attributes |= FONTPERSIST_STRIKETHROUGH as BYTE;
    }

    if (*this).description.fUnderline != 0 {
        attributes |= FONTPERSIST_UNDERLINE as BYTE;
    }

    IStream_Write(
        pOutStream,
        &attributes as *const BYTE as *const core::ffi::c_void,
        core::mem::size_of::<BYTE>() as ULONG,
        &mut written,
    );
    if written as usize != core::mem::size_of::<BYTE>() {
        return E_FAIL;
    }

    IStream_Write(
        pOutStream,
        &(*this).description.sWeight as *const i16 as *const core::ffi::c_void,
        core::mem::size_of::<WORD>() as ULONG,
        &mut written,
    );
    if written as usize != core::mem::size_of::<WORD>() {
        return E_FAIL;
    }

    IStream_Write(
        pOutStream,
        core::ptr::addr_of!((*this).description.cySize.__bindgen_anon_1.Lo)
            as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        &mut written,
    );
    if written as usize != core::mem::size_of::<DWORD>() {
        return E_FAIL;
    }

    if !(*this).description.lpstrName.is_null() {
        string_size = WideCharToMultiByte(
            CP_ACP,
            0,
            (*this).description.lpstrName as LPCWSTR,
            lstrlenW((*this).description.lpstrName as LPCWSTR),
            core::ptr::null_mut::<i8>(),
            0,
            core::ptr::null::<i8>(),
            core::ptr::null_mut::<i32>(),
        ) as BYTE;
    } else {
        string_size = 0;
    }

    IStream_Write(
        pOutStream,
        &string_size as *const BYTE as *const core::ffi::c_void,
        core::mem::size_of::<BYTE>() as ULONG,
        &mut written,
    );
    if written as usize != core::mem::size_of::<BYTE>() {
        return E_FAIL;
    }

    if string_size != 0 {
        writeBuffer = HeapAlloc(GetProcessHeap(), 0, string_size as SIZE_T) as *mut i8;
        if writeBuffer.is_null() {
            return E_OUTOFMEMORY;
        }
        WideCharToMultiByte(
            CP_ACP,
            0,
            (*this).description.lpstrName as LPCWSTR,
            lstrlenW((*this).description.lpstrName as LPCWSTR),
            writeBuffer,
            string_size as i32,
            core::ptr::null::<i8>(),
            core::ptr::null_mut::<i32>(),
        );

        IStream_Write(
            pOutStream,
            writeBuffer as *const core::ffi::c_void,
            string_size as ULONG,
            &mut written,
        );
        HeapFree(GetProcessHeap(), 0, writeBuffer as LPVOID);

        if written != string_size as u32 {
            return E_FAIL;
        }
    }

    S_OK
}
}
stdmethod! {
fn OLEFontImpl_GetSizeMax(
    iface: *mut IPersistStream,
    pcbSize: *mut ULARGE_INTEGER,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IPersistStream(iface);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    // C: `pcbSize->u.HighPart` / `pcbSize->u.LowPart`. `u` is ULARGE_INTEGER's *named* struct
    // arm (NONAMELESSUNION) and already carries LowPart/HighPart directly — the translation
    // had inserted a spurious extra `__bindgen_anon_1` level.
    (*pcbSize).u.HighPart = 0;
    (*pcbSize).u.LowPart = 0;

    (*pcbSize).u.LowPart = ((*pcbSize).u.LowPart as u64)
        .wrapping_add(core::mem::size_of::<BYTE>() as u64) as DWORD;
    (*pcbSize).u.LowPart = ((*pcbSize).u.LowPart as u64)
        .wrapping_add(core::mem::size_of::<WORD>() as u64) as DWORD;
    (*pcbSize).u.LowPart = ((*pcbSize).u.LowPart as u64)
        .wrapping_add(core::mem::size_of::<BYTE>() as u64) as DWORD;
    (*pcbSize).u.LowPart = ((*pcbSize).u.LowPart as u64)
        .wrapping_add(core::mem::size_of::<WORD>() as u64) as DWORD;
    (*pcbSize).u.LowPart = ((*pcbSize).u.LowPart as u64)
        .wrapping_add(core::mem::size_of::<DWORD>() as u64) as DWORD;
    (*pcbSize).u.LowPart = ((*pcbSize).u.LowPart as u64)
        .wrapping_add(core::mem::size_of::<BYTE>() as u64) as DWORD;

    if !(*this).description.lpstrName.is_null() {

        (*pcbSize).u.LowPart = ((*pcbSize).u.LowPart as u32).wrapping_add(
            WideCharToMultiByte(
                CP_ACP,
                0,
                (*this).description.lpstrName,
                lstrlenW((*this).description.lpstrName),
                core::ptr::null_mut(),
                0,
                core::ptr::null(),
                core::ptr::null_mut(),
            ) as u32,
        ) as DWORD;
    }

    S_OK
}
}
stdmethod! {
fn OLEFontImpl_IConnectionPointContainer_QueryInterface(
    iface: *mut IConnectionPointContainer,
    riid: REFIID,
    ppvoid: *mut LPVOID,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IConnectionPointContainer(iface);
    return IFont_QueryInterface(core::ptr::addr_of_mut!((*this).IFont_iface), riid, ppvoid);
}
}
stdmethod! {
fn OLEFontImpl_IConnectionPointContainer_Release(
    iface: *mut IConnectionPointContainer,
) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IConnectionPointContainer(iface);
    return IFont_Release(core::ptr::addr_of_mut!((*this).IFont_iface));
}
}
stdmethod! {
fn OLEFontImpl_IConnectionPointContainer_AddRef(
    iface: *mut IConnectionPointContainer,
) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IConnectionPointContainer(iface);
    return IFont_AddRef(core::ptr::addr_of_mut!((*this).IFont_iface));
}
}
stdmethod! {
fn OLEFontImpl_EnumConnectionPoints(
    iface: *mut IConnectionPointContainer,
    ppEnum: *mut *mut IEnumConnectionPoints,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IConnectionPointContainer(iface);
    
    if (*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_FIXME as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                as *const u8 as *const i8,
            b"OLEFontImpl_EnumConnectionPoints\0" as *const u8 as *const i8,
            1838i32,
            b"(%p)->(%p): stub\n\0" as *const u8 as *const i8,
            this,
            ppEnum,
        );
    }
    
    E_NOTIMPL
}
}
stdmethod! {
fn OLEFontImpl_FindConnectionPoint(
    iface: *mut IConnectionPointContainer,
    riid: *const IID,
    ppCp: *mut *mut IConnectionPoint,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IConnectionPointContainer(iface);
    TRACE!(
        b"(%p)->(%s, %p)\n\0".as_ptr() as *const i8,
        this,
        debugstr_guid(riid as *const CLSID),
        ppCp,
    );

    if IsEqualIID(riid, core::ptr::addr_of!(IID_IPropertyNotifySink)) {
        return IConnectionPoint_QueryInterface(
            (*this).pPropertyNotifyCP,
            core::ptr::addr_of!(IID_IConnectionPoint),
            ppCp as *mut LPVOID,
        );
    } else if IsEqualIID(riid, core::ptr::addr_of!(IID_IFontEventsDisp)) {
        return IConnectionPoint_QueryInterface(
            (*this).pFontEventsCP,
            core::ptr::addr_of!(IID_IConnectionPoint),
            ppCp as *mut LPVOID,
        );
    } else {
        FIXME!(
            b"no connection point for %s\n\0".as_ptr() as *const i8,
            debugstr_guid(riid as *const CLSID),
        );
        return CONNECT_E_NOCONNECTION;
    }
}
}
stdmethod! {
fn OLEFontImpl_IPersistPropertyBag_QueryInterface(
    iface: *mut IPersistPropertyBag,
    riid: *const IID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IPersistPropertyBag(iface);
    return IFont_QueryInterface(
        core::ptr::addr_of_mut!((*this).IFont_iface),
        riid,
        ppvObj as *mut *mut core::ffi::c_void,
    );
}
}
stdmethod! {
fn OLEFontImpl_IPersistPropertyBag_AddRef(
    iface: *mut IPersistPropertyBag,
) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IPersistPropertyBag(iface);
    return IFont_AddRef(core::ptr::addr_of_mut!((*this).IFont_iface));
}
}
stdmethod! {
fn OLEFontImpl_IPersistPropertyBag_Release(
    iface: *mut IPersistPropertyBag,
) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IPersistPropertyBag(iface);
    return IFont_Release(core::ptr::addr_of_mut!((*this).IFont_iface));
}
}
stdmethod! {
fn OLEFontImpl_IPersistPropertyBag_GetClassID(
    iface: *mut IPersistPropertyBag,
    classid: *mut CLSID,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_FIXME as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                .as_ptr() as *const i8,
            b"OLEFontImpl_IPersistPropertyBag_GetClassID\0"
                .as_ptr() as *const i8,
            1902i32,
            b"(%p,%p), stub!\n\0".as_ptr() as *const i8,
            iface,
            classid,
        );
    }
    return E_FAIL;
}
}
stdmethod! {
fn OLEFontImpl_IPersistPropertyBag_InitNew(
    iface: *mut IPersistPropertyBag,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_FIXME as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                .as_ptr() as *const i8,
            b"OLEFontImpl_IPersistPropertyBag_InitNew\0".as_ptr() as *const i8,
            1909 as i32,
            b"(%p), stub!\n\0".as_ptr() as *const i8,
            iface,
        );
    }
    return S_OK;
}
}
stdmethod! {
fn OLEFontImpl_IPersistPropertyBag_Load(
    iface: *mut IPersistPropertyBag,
    pPropBag: *mut IPropertyBag,
    pErrorLog: *mut IErrorLog,
) -> HRESULT {
    static sAttrName: [u16; 5] = [
        'N' as u16,
        'a' as u16,
        'm' as u16,
        'e' as u16,
        0,
    ];
    static sAttrSize: [u16; 5] = [
        'S' as u16,
        'i' as u16,
        'z' as u16,
        'e' as u16,
        0,
    ];
    static sAttrCharset: [u16; 8] = [
        'C' as u16,
        'h' as u16,
        'a' as u16,
        'r' as u16,
        's' as u16,
        'e' as u16,
        't' as u16,
        0,
    ];
    static sAttrWeight: [u16; 7] = [
        'W' as u16,
        'e' as u16,
        'i' as u16,
        'g' as u16,
        'h' as u16,
        't' as u16,
        0,
    ];
    static sAttrUnderline: [u16; 10] = [
        'U' as u16,
        'n' as u16,
        'd' as u16,
        'e' as u16,
        'r' as u16,
        'l' as u16,
        'i' as u16,
        'n' as u16,
        'e' as u16,
        0,
    ];
    static sAttrItalic: [u16; 7] = [
        'I' as u16,
        't' as u16,
        'a' as u16,
        'l' as u16,
        'i' as u16,
        'c' as u16,
        0,
    ];
    static sAttrStrikethrough: [u16; 14] = [
        'S' as u16,
        't' as u16,
        'r' as u16,
        'i' as u16,
        'k' as u16,
        'e' as u16,
        't' as u16,
        'h' as u16,
        'r' as u16,
        'o' as u16,
        'u' as u16,
        'g' as u16,
        'h' as u16,
        0,
    ];

    let this: *mut OLEFontImpl = impl_from_IPersistPropertyBag(iface);
    let mut value: VARIANT = core::mem::zeroed();
    let mut iRes: HRESULT;

    VariantInit(core::ptr::addr_of_mut!(value));

    iRes = IPropertyBag_Read(
        pPropBag,
        sAttrName.as_ptr() as LPCWSTR,
        core::ptr::addr_of_mut!(value),
        pErrorLog,
    );
    if iRes == S_OK {
        iRes = VariantChangeType(
            core::ptr::addr_of_mut!(value),
            core::ptr::addr_of_mut!(value),
            0,
            VT_BSTR as u16,
        );
        if iRes == S_OK {
            iRes = IFont_put_Name(core::ptr::addr_of_mut!((*this).IFont_iface), V_BSTR(core::ptr::addr_of_mut!(value)));
        }
    } else if iRes == E_INVALIDARG {
        iRes = S_OK;
    }

    VariantClear(core::ptr::addr_of_mut!(value));

    if iRes == S_OK {
        iRes = IPropertyBag_Read(
            pPropBag,
            sAttrSize.as_ptr() as LPCWSTR,
            core::ptr::addr_of_mut!(value),
            pErrorLog,
        );
        if iRes == S_OK {
            iRes = VariantChangeType(
                core::ptr::addr_of_mut!(value),
                core::ptr::addr_of_mut!(value),
                0,
                VT_CY as u16,
            );
            if iRes == S_OK {
                iRes = IFont_put_Size(core::ptr::addr_of_mut!((*this).IFont_iface), V_CY(core::ptr::addr_of_mut!(value)));
            }
        } else if iRes == E_INVALIDARG {
            iRes = S_OK;
        }

        VariantClear(core::ptr::addr_of_mut!(value));
    }

    if iRes == S_OK {
        iRes = IPropertyBag_Read(
            pPropBag,
            sAttrCharset.as_ptr() as LPCWSTR,
            core::ptr::addr_of_mut!(value),
            pErrorLog,
        );
        if iRes == S_OK {
            iRes = VariantChangeType(
                core::ptr::addr_of_mut!(value),
                core::ptr::addr_of_mut!(value),
                0,
                VT_I2 as u16,
            );
            if iRes == S_OK {
                iRes = IFont_put_Charset(core::ptr::addr_of_mut!((*this).IFont_iface), V_I2(core::ptr::addr_of_mut!(value)));
            }
        } else if iRes == E_INVALIDARG {
            iRes = S_OK;
        }

        VariantClear(core::ptr::addr_of_mut!(value));
    }

    if iRes == S_OK {
        iRes = IPropertyBag_Read(
            pPropBag,
            sAttrWeight.as_ptr() as LPCWSTR,
            core::ptr::addr_of_mut!(value),
            pErrorLog,
        );
        if iRes == S_OK {
            iRes = VariantChangeType(
                core::ptr::addr_of_mut!(value),
                core::ptr::addr_of_mut!(value),
                0,
                VT_I2 as u16,
            );
            if iRes == S_OK {
                iRes = IFont_put_Weight(core::ptr::addr_of_mut!((*this).IFont_iface), V_I2(core::ptr::addr_of_mut!(value)));
            }
        } else if iRes == E_INVALIDARG {
            iRes = S_OK;
        }

        VariantClear(core::ptr::addr_of_mut!(value));
    }

    if iRes == S_OK {
        iRes = IPropertyBag_Read(
            pPropBag,
            sAttrUnderline.as_ptr() as LPCWSTR,
            core::ptr::addr_of_mut!(value),
            pErrorLog,
        );
        if iRes == S_OK {
            iRes = VariantChangeType(
                core::ptr::addr_of_mut!(value),
                core::ptr::addr_of_mut!(value),
                0,
                VT_BOOL as u16,
            );
            if iRes == S_OK {
                // V_BOOL is a VARIANT_BOOL (short); IFont_put_* takes a BOOL (int). The C
                // widened implicitly.
                iRes = IFont_put_Underline(core::ptr::addr_of_mut!((*this).IFont_iface), V_BOOL(core::ptr::addr_of_mut!(value)) as BOOL);
            }
        } else if iRes == E_INVALIDARG {
            iRes = S_OK;
        }

        VariantClear(core::ptr::addr_of_mut!(value));
    }

    if iRes == S_OK {
        iRes = IPropertyBag_Read(
            pPropBag,
            sAttrItalic.as_ptr() as LPCWSTR,
            core::ptr::addr_of_mut!(value),
            pErrorLog,
        );
        if iRes == S_OK {
            iRes = VariantChangeType(
                core::ptr::addr_of_mut!(value),
                core::ptr::addr_of_mut!(value),
                0,
                VT_BOOL as u16,
            );
            if iRes == S_OK {
                iRes = IFont_put_Italic(core::ptr::addr_of_mut!((*this).IFont_iface), V_BOOL(core::ptr::addr_of_mut!(value)) as BOOL);
            }
        } else if iRes == E_INVALIDARG {
            iRes = S_OK;
        }

        VariantClear(core::ptr::addr_of_mut!(value));
    }

    if iRes == S_OK {
        iRes = IPropertyBag_Read(
            pPropBag,
            sAttrStrikethrough.as_ptr() as LPCWSTR,
            core::ptr::addr_of_mut!(value),
            pErrorLog,
        );
        if iRes == S_OK {
            iRes = VariantChangeType(
                core::ptr::addr_of_mut!(value),
                core::ptr::addr_of_mut!(value),
                0,
                VT_BOOL as u16,
            );
            if iRes == S_OK {
                IFont_put_Strikethrough(core::ptr::addr_of_mut!((*this).IFont_iface), V_BOOL(core::ptr::addr_of_mut!(value)) as BOOL);
            }
        } else if iRes == E_INVALIDARG {
            iRes = S_OK;
        }

        VariantClear(core::ptr::addr_of_mut!(value));
    }

    if FAILED(iRes) {
        WARN!(b"-- 0x%08x\n\0".as_ptr() as *const i8, iRes);
    }
    iRes
}
}
stdmethod! {
fn OLEFontImpl_IPersistPropertyBag_Save(
    iface: *mut IPersistPropertyBag,
    pPropBag: *mut IPropertyBag,
    fClearDirty: BOOL,
    fSaveAllProperties: BOOL,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_FIXME as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                .as_ptr() as *const i8,
            b"OLEFontImpl_IPersistPropertyBag_Save\0".as_ptr() as *const i8,
            2043i32,
            b"(%p,%p,%d,%d), stub!\n\0".as_ptr() as *const i8,
            iface,
            pPropBag,
            fClearDirty,
            fSaveAllProperties,
        );
    }
    return E_FAIL;
}
}
stdmethod! {
fn OLEFontImpl_IPersistStreamInit_QueryInterface(
    iface: *mut IPersistStreamInit,
    riid: *const IID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let this: *mut OLEFontImpl = impl_from_IPersistStreamInit(iface);
    return IFont_QueryInterface(
        core::ptr::addr_of_mut!((*this).IFont_iface),
        riid,
        ppvObj as *mut *mut core::ffi::c_void,
    );
}
}
stdmethod! {
fn OLEFontImpl_IPersistStreamInit_AddRef(
    iface: *mut IPersistStreamInit,
) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IPersistStreamInit(iface);
    return IFont_AddRef(core::ptr::addr_of_mut!((*this).IFont_iface));
}
}
stdmethod! {
fn OLEFontImpl_IPersistStreamInit_Release(
    iface: *mut IPersistStreamInit,
) -> ULONG {
    let this: *mut OLEFontImpl = impl_from_IPersistStreamInit(iface);
    return IFont_Release(core::ptr::addr_of_mut!((*this).IFont_iface));
}
}
stdmethod! {
fn OLEFontImpl_IPersistStreamInit_GetClassID(
    iface: *mut IPersistStreamInit,
    classid: *mut CLSID,
) -> HRESULT {
    FIXME!(b"(%p,%p), stub!\n\0".as_ptr() as *const i8, iface, classid);
    E_FAIL
}
}
stdmethod! {
fn OLEFontImpl_IPersistStreamInit_IsDirty(
    iface: *mut IPersistStreamInit,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_FIXME as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                as *const u8 as *const i8,
            b"OLEFontImpl_IPersistStreamInit_IsDirty\0" as *const u8 as *const i8,
            2093i32,
            b"(%p), stub!\n\0" as *const u8 as *const i8,
            iface,
        );
    }
    return E_FAIL;
}
}
stdmethod! {
fn OLEFontImpl_IPersistStreamInit_Load(
    iface: *mut IPersistStreamInit,
    pStm: LPSTREAM,
) -> HRESULT {
    FIXME!(b"(%p,%p), stub!\n\0".as_ptr() as *const i8, iface, pStm);
    E_FAIL
}
}
stdmethod! {
fn OLEFontImpl_IPersistStreamInit_Save(
    iface: *mut IPersistStreamInit,
    pStm: LPSTREAM,
    fClearDirty: BOOL,
) -> HRESULT {
    FIXME!(
        b"(%p,%p,%d), stub!\n\0".as_ptr() as *const i8,
        iface,
        pStm,
        fClearDirty,
    );
    E_FAIL
}
}
stdmethod! {
fn OLEFontImpl_IPersistStreamInit_GetSizeMax(
    iface: *mut IPersistStreamInit,
    pcbSize: *mut ULARGE_INTEGER,
) -> HRESULT {
    FIXME!(b"(%p,%p), stub!\n\0".as_ptr() as *const i8, iface, pcbSize);
    E_FAIL
}
}
stdmethod! {
fn OLEFontImpl_IPersistStreamInit_InitNew(
    iface: *mut IPersistStreamInit,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_FIXME as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olefont.c\0"
                .as_ptr() as *const i8,
            b"OLEFontImpl_IPersistStreamInit_InitNew\0".as_ptr() as *const i8,
            2121i32,
            b"(%p), stub!\n\0".as_ptr() as *const i8,
            iface,
        );
    }
    return S_OK;
}
}
unsafe extern "C" fn OLEFontImpl_Construct(fontDesc: *const FONTDESC) -> *mut OLEFontImpl {
    let newObject: *mut OLEFontImpl = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<OLEFontImpl>() as SIZE_T,
    ) as *mut OLEFontImpl;

    if newObject.is_null() {
        return newObject;
    }

    // bindgen drops the `CONST_VTBL` from the C's `const IFontVtbl *lpVtbl`, so each slot is
    // typed `*mut`. Cast rather than making the vtbl statics `static mut` — COM never writes
    // through a vtbl pointer.
    (*newObject).IFont_iface.lpVtbl = &raw const OLEFontImpl_VTable as *mut IFontVtbl;
    (*newObject).IDispatch_iface.lpVtbl =
        &raw const OLEFontImpl_IDispatch_VTable as *mut IDispatchVtbl;
    (*newObject).IPersistStream_iface.lpVtbl =
        &raw const OLEFontImpl_IPersistStream_VTable as *mut IPersistStreamVtbl;
    (*newObject).IConnectionPointContainer_iface.lpVtbl =
        &raw const OLEFontImpl_IConnectionPointContainer_VTable
            as *mut IConnectionPointContainerVtbl;
    (*newObject).IPersistPropertyBag_iface.lpVtbl =
        &raw const OLEFontImpl_IPersistPropertyBag_VTable as *mut IPersistPropertyBagVtbl;
    (*newObject).IPersistStreamInit_iface.lpVtbl =
        &raw const OLEFontImpl_IPersistStreamInit_VTable as *mut IPersistStreamInitVtbl;

    (*newObject).ref_ = 1;

    (*newObject).description.cbSizeofstruct = core::mem::size_of::<FONTDESC>() as UINT;
    (*newObject).description.lpstrName = strdupW((*fontDesc).lpstrName);
    (*newObject).description.cySize = (*fontDesc).cySize;
    (*newObject).description.sWeight = (*fontDesc).sWeight;
    (*newObject).description.sCharset = (*fontDesc).sCharset;
    (*newObject).description.fItalic = (*fontDesc).fItalic;
    (*newObject).description.fUnderline = (*fontDesc).fUnderline;
    (*newObject).description.fStrikethrough = (*fontDesc).fStrikethrough;

    (*newObject).gdiFont = core::ptr::null_mut();
    (*newObject).dirty = 1;
    (*newObject).cyLogical = GetDeviceCaps(get_dc(), LOGPIXELSY);
    (*newObject).cyHimetric = 2540;
    (*newObject).pPropertyNotifyCP = core::ptr::null_mut();
    (*newObject).pFontEventsCP = core::ptr::null_mut();

    CreateConnectionPoint(
        &mut (*newObject).IFont_iface as *mut IFont as *mut IUnknown,
        &IID_IPropertyNotifySink,
        &mut (*newObject).pPropertyNotifyCP,
    );
    CreateConnectionPoint(
        &mut (*newObject).IFont_iface as *mut IFont as *mut IUnknown,
        &IID_IFontEventsDisp,
        &mut (*newObject).pFontEventsCP,
    );

    if (*newObject).pPropertyNotifyCP.is_null() || (*newObject).pFontEventsCP.is_null() {
        OLEFontImpl_Destroy(newObject);
        return core::ptr::null_mut();
    }

    InterlockedIncrement(&raw mut ifont_cnt);

    TRACE!(b"returning %p\n\0".as_ptr() as *const i8, newObject);
    newObject
}
unsafe extern "C" fn OLEFontImpl_Destroy(fontDesc: *mut OLEFontImpl) {
    TRACE!(
        b"(%p)\n\0".as_ptr() as *const i8,
        fontDesc,
    );

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        (*fontDesc).description.lpstrName as LPVOID,
    );

    if !(*fontDesc).pPropertyNotifyCP.is_null() {
        IConnectionPoint_Release((*fontDesc).pPropertyNotifyCP);
    }
    if !(*fontDesc).pFontEventsCP.is_null() {
        IConnectionPoint_Release((*fontDesc).pFontEventsCP);
    }

    HeapFree(GetProcessHeap(), 0 as DWORD, fontDesc as LPVOID);
}
unsafe extern "C" fn impl_from_IClassFactory(
    iface: *mut IClassFactory,
) -> *mut IClassFactoryImpl {
    (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(IClassFactoryImpl, IClassFactory_iface) as ULONG_PTR,
    ) as *mut IClassFactoryImpl
}
stdmethod! {
fn SFCF_QueryInterface(
    iface: *mut IClassFactory,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let _This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface);

    TRACE!(
        b"(%p)->(%s, %p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_guid(riid as *const CLSID),
        obj,
    );

    *obj = core::ptr::null_mut();

    if IsEqualIID(
        core::ptr::addr_of!(IID_IClassFactory) as *const IID,
        riid,
    )
        || IsEqualIID(
            core::ptr::addr_of!(IID_IUnknown) as *const IID,
            riid,
        )
    {
        *obj = iface as *mut core::ffi::c_void;
        IClassFactory_AddRef(iface);
        return S_OK;
    }

    return E_NOINTERFACE;
}
}
stdmethod! {
fn SFCF_AddRef(iface: LPCLASSFACTORY) -> ULONG {
    let This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    InterlockedIncrement(&mut (*This).ref_ as *mut LONG) as ULONG
}
}
stdmethod! {
fn SFCF_Release(iface: LPCLASSFACTORY) -> ULONG {
    let This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    return InterlockedDecrement(&mut (*This).ref_ as *mut LONG) as ULONG;
}
}
stdmethod! {
fn SFCF_CreateInstance(
    iface: LPCLASSFACTORY,
    pOuter: LPUNKNOWN,
    riid: *const IID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    return OleCreateFontIndirect(core::ptr::null_mut(), riid, ppobj);
}
}
stdmethod! {
fn SFCF_LockServer(iface: LPCLASSFACTORY, dolock: BOOL) -> HRESULT {
    let This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    FIXME!(b"(%p)->(%d),stub!\n\0".as_ptr() as *const i8, This, dolock);
    return S_OK;
}
}
#[no_mangle]
pub unsafe extern "C" fn _get_STDFONT_CF(ppv: *mut LPVOID) {
    *ppv = core::ptr::addr_of_mut!(STDFONT_CF) as LPVOID;
}

// COM vtable tables — generated from olefont.c by vtbl_gen.py
/// olefont.c:1164 `static const IFontVtbl OLEFontImpl_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static OLEFontImpl_VTable: IFontVtbl = IFontVtbl {
    QueryInterface: Some(OLEFontImpl_QueryInterface),
    AddRef: Some(OLEFontImpl_AddRef),
    Release: Some(OLEFontImpl_Release),
    get_Name: Some(OLEFontImpl_get_Name),
    put_Name: Some(OLEFontImpl_put_Name),
    get_Size: Some(OLEFontImpl_get_Size),
    put_Size: Some(OLEFontImpl_put_Size),
    get_Bold: Some(OLEFontImpl_get_Bold),
    put_Bold: Some(OLEFontImpl_put_Bold),
    get_Italic: Some(OLEFontImpl_get_Italic),
    put_Italic: Some(OLEFontImpl_put_Italic),
    get_Underline: Some(OLEFontImpl_get_Underline),
    put_Underline: Some(OLEFontImpl_put_Underline),
    get_Strikethrough: Some(OLEFontImpl_get_Strikethrough),
    put_Strikethrough: Some(OLEFontImpl_put_Strikethrough),
    get_Weight: Some(OLEFontImpl_get_Weight),
    put_Weight: Some(OLEFontImpl_put_Weight),
    get_Charset: Some(OLEFontImpl_get_Charset),
    put_Charset: Some(OLEFontImpl_put_Charset),
    get_hFont: Some(OLEFontImpl_get_hFont),
    Clone: Some(OLEFontImpl_Clone),
    IsEqual: Some(OLEFontImpl_IsEqual),
    SetRatio: Some(OLEFontImpl_SetRatio),
    QueryTextMetrics: Some(OLEFontImpl_QueryTextMetrics),
    AddRefHfont: Some(OLEFontImpl_AddRefHfont),
    ReleaseHfont: Some(OLEFontImpl_ReleaseHfont),
    SetHdc: Some(OLEFontImpl_SetHdc),
};

/// olefont.c:1526 `static const IDispatchVtbl OLEFontImpl_IDispatch_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static OLEFontImpl_IDispatch_VTable: IDispatchVtbl = IDispatchVtbl {
    QueryInterface: Some(OLEFontImpl_IDispatch_QueryInterface),
    AddRef: Some(OLEFontImpl_IDispatch_AddRef),
    Release: Some(OLEFontImpl_IDispatch_Release),
    GetTypeInfoCount: Some(OLEFontImpl_GetTypeInfoCount),
    GetTypeInfo: Some(OLEFontImpl_GetTypeInfo),
    GetIDsOfNames: Some(OLEFontImpl_GetIDsOfNames),
    Invoke: Some(OLEFontImpl_Invoke),
};

/// olefont.c:1782 `static const IPersistStreamVtbl OLEFontImpl_IPersistStream_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static OLEFontImpl_IPersistStream_VTable: IPersistStreamVtbl = IPersistStreamVtbl {
    QueryInterface: Some(OLEFontImpl_IPersistStream_QueryInterface),
    AddRef: Some(OLEFontImpl_IPersistStream_AddRef),
    Release: Some(OLEFontImpl_IPersistStream_Release),
    GetClassID: Some(OLEFontImpl_GetClassID),
    IsDirty: Some(OLEFontImpl_IsDirty),
    Load: Some(OLEFontImpl_Load),
    Save: Some(OLEFontImpl_Save),
    GetSizeMax: Some(OLEFontImpl_GetSizeMax),
};

/// olefont.c:1865 `static const IConnectionPointContainerVtbl OLEFontImpl_IConnectionPointContainer_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static OLEFontImpl_IConnectionPointContainer_VTable: IConnectionPointContainerVtbl = IConnectionPointContainerVtbl {
    QueryInterface: Some(OLEFontImpl_IConnectionPointContainer_QueryInterface),
    AddRef: Some(OLEFontImpl_IConnectionPointContainer_AddRef),
    Release: Some(OLEFontImpl_IConnectionPointContainer_Release),
    EnumConnectionPoints: Some(OLEFontImpl_EnumConnectionPoints),
    FindConnectionPoint: Some(OLEFontImpl_FindConnectionPoint),
};

/// olefont.c:2047 `static const IPersistPropertyBagVtbl OLEFontImpl_IPersistPropertyBag_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static OLEFontImpl_IPersistPropertyBag_VTable: IPersistPropertyBagVtbl = IPersistPropertyBagVtbl {
    QueryInterface: Some(OLEFontImpl_IPersistPropertyBag_QueryInterface),
    AddRef: Some(OLEFontImpl_IPersistPropertyBag_AddRef),
    Release: Some(OLEFontImpl_IPersistPropertyBag_Release),
    GetClassID: Some(OLEFontImpl_IPersistPropertyBag_GetClassID),
    InitNew: Some(OLEFontImpl_IPersistPropertyBag_InitNew),
    Load: Some(OLEFontImpl_IPersistPropertyBag_Load),
    Save: Some(OLEFontImpl_IPersistPropertyBag_Save),
};

/// olefont.c:2125 `static const IPersistStreamInitVtbl OLEFontImpl_IPersistStreamInit_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static OLEFontImpl_IPersistStreamInit_VTable: IPersistStreamInitVtbl = IPersistStreamInitVtbl {
    QueryInterface: Some(OLEFontImpl_IPersistStreamInit_QueryInterface),
    AddRef: Some(OLEFontImpl_IPersistStreamInit_AddRef),
    Release: Some(OLEFontImpl_IPersistStreamInit_Release),
    GetClassID: Some(OLEFontImpl_IPersistStreamInit_GetClassID),
    IsDirty: Some(OLEFontImpl_IPersistStreamInit_IsDirty),
    Load: Some(OLEFontImpl_IPersistStreamInit_Load),
    Save: Some(OLEFontImpl_IPersistStreamInit_Save),
    GetSizeMax: Some(OLEFontImpl_IPersistStreamInit_GetSizeMax),
    InitNew: Some(OLEFontImpl_IPersistStreamInit_InitNew),
};

/// olefont.c:2276 `static const IClassFactoryVtbl SFCF_Vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static SFCF_Vtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(SFCF_QueryInterface),
    AddRef: Some(SFCF_AddRef),
    Release: Some(SFCF_Release),
    CreateInstance: Some(SFCF_CreateInstance),
    LockServer: Some(SFCF_LockServer),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * OLE Font encapsulation implementation
//  *
//  * This file contains an implementation of the IFont
//  * interface and the OleCreateFontIndirect API call.
//  *
//  * Copyright 1999 Francis Beaudet
//  * Copyright 2006 (Google) Benjamin Arai
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
// #include <assert.h>
// #include <stdarg.h>
// #include <string.h>
//
// #define COBJMACROS
// #define NONAMELESSUNION
// #define NONAMELESSSTRUCT
//
// #include "winerror.h"
// #include "windef.h"
// #include "winbase.h"
// #include "wingdi.h"
// #include "winuser.h"
// #include "wine/list.h"
// #include "objbase.h"
// #include "oleauto.h"    /* for SysAllocString(....) */
// #include "ole2.h"
// #include "olectl.h"
// #include "wine/debug.h"
// #include "connpt.h" /* for CreateConnectionPoint */
// #include "oaidl.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(ole);
//
// /***********************************************************************
//  * Declaration of constants used when serializing the font object.
//  */
// #define FONTPERSIST_ITALIC        0x02
// #define FONTPERSIST_UNDERLINE     0x04
// #define FONTPERSIST_STRIKETHROUGH 0x08
//
// static HDC olefont_hdc;
//
// /***********************************************************************
//  * List of the HFONTs it has given out, with each one having a separate
//  * ref count.
//  */
// typedef struct _HFONTItem
// {
//   struct list entry;
//
//   /* Reference count of any IFont objects that own this hfont */
//   LONG int_refs;
//
//   /* Total reference count of any refs held by the application obtained by AddRefHfont plus any internal refs */
//   LONG total_refs;
//
//   /* The font associated with this object. */
//   HFONT gdiFont;
//
// } HFONTItem, *PHFONTItem;
//
// static struct list OLEFontImpl_hFontList = LIST_INIT(OLEFontImpl_hFontList);
//
// /* Counts how many fonts contain at least one lock */
// static LONG ifont_cnt = 0;
//
// /***********************************************************************
//  * Critical section for OLEFontImpl_hFontList
//  */
// static CRITICAL_SECTION OLEFontImpl_csHFONTLIST;
// static CRITICAL_SECTION_DEBUG OLEFontImpl_csHFONTLIST_debug =
// {
//   0, 0, &OLEFontImpl_csHFONTLIST,
//   { &OLEFontImpl_csHFONTLIST_debug.ProcessLocksList,
//     &OLEFontImpl_csHFONTLIST_debug.ProcessLocksList },
//     0, 0, { (DWORD_PTR)(__FILE__ ": OLEFontImpl_csHFONTLIST") }
// };
// static CRITICAL_SECTION OLEFontImpl_csHFONTLIST = { &OLEFontImpl_csHFONTLIST_debug, -1, 0, 0, 0, 0 };
//
// static HDC get_dc(void)
// {
//     HDC hdc;
//     EnterCriticalSection(&OLEFontImpl_csHFONTLIST);
//     if(!olefont_hdc)
//         olefont_hdc = CreateCompatibleDC(NULL);
//     hdc = olefont_hdc;
//     LeaveCriticalSection(&OLEFontImpl_csHFONTLIST);
//     return hdc;
// }
//
// static void delete_dc(void)
// {
//     EnterCriticalSection(&OLEFontImpl_csHFONTLIST);
//     if(olefont_hdc)
//     {
//         DeleteDC(olefont_hdc);
//         olefont_hdc = NULL;
//     }
//     LeaveCriticalSection(&OLEFontImpl_csHFONTLIST);
// }
//
// static void HFONTItem_Delete(PHFONTItem item)
// {
//   DeleteObject(item->gdiFont);
//   list_remove(&item->entry);
//   HeapFree(GetProcessHeap(), 0, item);
// }
//
// /* Find hfont item entry in the list.  Should be called while holding the crit sect */
// static HFONTItem *find_hfontitem(HFONT hfont)
// {
//     HFONTItem *item;
//
//     LIST_FOR_EACH_ENTRY(item, &OLEFontImpl_hFontList, HFONTItem, entry)
//     {
//         if (item->gdiFont == hfont)
//             return item;
//     }
//     return NULL;
// }
//
// /* Add an item to the list with one internal reference */
// static HRESULT add_hfontitem(HFONT hfont)
// {
//     HFONTItem *new_item = HeapAlloc(GetProcessHeap(), 0, sizeof(*new_item));
//
//     if(!new_item) return E_OUTOFMEMORY;
//
//     new_item->int_refs = 1;
//     new_item->total_refs = 1;
//     new_item->gdiFont = hfont;
//     EnterCriticalSection(&OLEFontImpl_csHFONTLIST);
//     list_add_tail(&OLEFontImpl_hFontList,&new_item->entry);
//     LeaveCriticalSection(&OLEFontImpl_csHFONTLIST);
//     return S_OK;
// }
//
// static HRESULT inc_int_ref(HFONT hfont)
// {
//     HFONTItem *item;
//     HRESULT hr = S_FALSE;
//
//     EnterCriticalSection(&OLEFontImpl_csHFONTLIST);
//     item = find_hfontitem(hfont);
//
//     if(item)
//     {
//         item->int_refs++;
//         item->total_refs++;
//         hr = S_OK;
//     }
//     LeaveCriticalSection(&OLEFontImpl_csHFONTLIST);
//
//     return hr;
// }
//
// /* decrements the internal ref of a hfont item.  If both refs are zero it'll
//    remove the item from the list and delete the hfont */
// static HRESULT dec_int_ref(HFONT hfont)
// {
//     HFONTItem *item;
//     HRESULT hr = S_FALSE;
//
//     EnterCriticalSection(&OLEFontImpl_csHFONTLIST);
//     item = find_hfontitem(hfont);
//
//     if(item)
//     {
//         item->int_refs--;
//         item->total_refs--;
//         if(item->int_refs == 0 && item->total_refs == 0)
//             HFONTItem_Delete(item);
//         hr = S_OK;
//     }
//     LeaveCriticalSection(&OLEFontImpl_csHFONTLIST);
//
//     return hr;
// }
//
// static HRESULT inc_ext_ref(HFONT hfont)
// {
//     HFONTItem *item;
//     HRESULT hr = S_FALSE;
//
//     EnterCriticalSection(&OLEFontImpl_csHFONTLIST);
//
//     item = find_hfontitem(hfont);
//     if(item)
//     {
//         item->total_refs++;
//         hr = S_OK;
//     }
//     LeaveCriticalSection(&OLEFontImpl_csHFONTLIST);
//
//     return hr;
// }
//
// static HRESULT dec_ext_ref(HFONT hfont)
// {
//     HFONTItem *item;
//     HRESULT hr = S_FALSE;
//
//     EnterCriticalSection(&OLEFontImpl_csHFONTLIST);
//
//     item = find_hfontitem(hfont);
//     if(item)
//     {
//         if(--item->total_refs >= 0) hr = S_OK;
//     }
//     LeaveCriticalSection(&OLEFontImpl_csHFONTLIST);
//
//     return hr;
// }
//
// static WCHAR *strdupW(const WCHAR* str)
// {
//     WCHAR *ret;
//     DWORD size = (lstrlenW(str) + 1) * sizeof(WCHAR);
//
//     ret = HeapAlloc(GetProcessHeap(), 0, size);
//     if(ret)
//         memcpy(ret, str, size);
//     return ret;
// }
//
// /***********************************************************************
//  * Declaration of the implementation class for the IFont interface
//  */
// typedef struct OLEFontImpl OLEFontImpl;
//
// struct OLEFontImpl
// {
//   /*
//    * This class supports many interfaces. IUnknown, IFont,
//    * IDispatch, IDispFont IPersistStream and IConnectionPointContainer.
//    * The first two are supported by the first vtable, the next two are
//    * supported by the second table and the last two have their own.
//    */
//   IFont                       IFont_iface;
//   IDispatch                   IDispatch_iface;
//   IPersistStream              IPersistStream_iface;
//   IConnectionPointContainer   IConnectionPointContainer_iface;
//   IPersistPropertyBag         IPersistPropertyBag_iface;
//   IPersistStreamInit          IPersistStreamInit_iface;
//   /*
//    * Reference count for that instance of the class.
//    */
//   LONG ref;
//
//   /*
//    * This structure contains the description of the class.
//    */
//   FONTDESC description;
//
//   /*
//    * Contain the font associated with this object.
//    */
//   HFONT gdiFont;
//   BOOL dirty;
//   /*
//    * Size ratio
//    */
//   LONG cyLogical;
//   LONG cyHimetric;
//
//   /*
//    * Stash realized height (pixels) from TEXTMETRIC - used in get_Size()
//    */
//   LONG nRealHeight;
//
//   IConnectionPoint *pPropertyNotifyCP;
//   IConnectionPoint *pFontEventsCP;
// };
//
// static inline OLEFontImpl *impl_from_IFont(IFont *iface)
// {
//     return CONTAINING_RECORD(iface, OLEFontImpl, IFont_iface);
// }
//
// static inline OLEFontImpl *impl_from_IDispatch( IDispatch *iface )
// {
//     return CONTAINING_RECORD(iface, OLEFontImpl, IDispatch_iface);
// }
//
// static inline OLEFontImpl *impl_from_IPersistStream( IPersistStream *iface )
// {
//     return CONTAINING_RECORD(iface, OLEFontImpl, IPersistStream_iface);
// }
//
// static inline OLEFontImpl *impl_from_IConnectionPointContainer( IConnectionPointContainer *iface )
// {
//     return CONTAINING_RECORD(iface, OLEFontImpl, IConnectionPointContainer_iface);
// }
//
// static inline OLEFontImpl *impl_from_IPersistPropertyBag( IPersistPropertyBag *iface )
// {
//     return CONTAINING_RECORD(iface, OLEFontImpl, IPersistPropertyBag_iface);
// }
//
// static inline OLEFontImpl *impl_from_IPersistStreamInit( IPersistStreamInit *iface )
// {
//     return CONTAINING_RECORD(iface, OLEFontImpl, IPersistStreamInit_iface);
// }
//
//
// /***********************************************************************
//  * Prototypes for the implementation functions for the IFont
//  * interface
//  */
// static OLEFontImpl* OLEFontImpl_Construct(const FONTDESC *fontDesc);
// static void         OLEFontImpl_Destroy(OLEFontImpl* fontDesc);
// static ULONG        WINAPI OLEFontImpl_AddRef(IFont* iface);
//
// /******************************************************************************
//  *		OleCreateFontIndirect	[OLEAUT32.420]
//  */
// HRESULT WINAPI OleCreateFontIndirect(
//   LPFONTDESC lpFontDesc,
//   REFIID     riid,
//   LPVOID*     ppvObj)
// {
//   OLEFontImpl* newFont;
//   HRESULT      hr;
//   FONTDESC     fd;
//
//   TRACE("(%p, %s, %p)\n", lpFontDesc, debugstr_guid(riid), ppvObj);
//
//   if (!ppvObj) return E_POINTER;
//
//   *ppvObj = 0;
//
//   if (!lpFontDesc) {
//     static WCHAR fname[] = { 'S','y','s','t','e','m',0 };
//
//     fd.cbSizeofstruct = sizeof(fd);
//     fd.lpstrName      = fname;
//     fd.cySize.s.Lo    = 80000;
//     fd.cySize.s.Hi    = 0;
//     fd.sWeight 	      = 0;
//     fd.sCharset       = 0;
//     fd.fItalic        = FALSE;
//     fd.fUnderline     = FALSE;
//     fd.fStrikethrough = FALSE;
//     lpFontDesc = &fd;
//   }
//
//   newFont = OLEFontImpl_Construct(lpFontDesc);
//   if (!newFont) return E_OUTOFMEMORY;
//
//   hr = IFont_QueryInterface(&newFont->IFont_iface, riid, ppvObj);
//   IFont_Release(&newFont->IFont_iface);
//
//   return hr;
// }
//
//
// /***********************************************************************
//  * Implementation of the OLEFontImpl class.
//  */
//
// /***********************************************************************
//  *    OLEFont_SendNotify (internal)
//  *
//  * Sends notification messages of changed properties to any interested
//  * connections.
//  */
// static void OLEFont_SendNotify(OLEFontImpl* this, DISPID dispID)
// {
//   static const WCHAR wszName[] = {'N','a','m','e',0};
//   static const WCHAR wszSize[] = {'S','i','z','e',0};
//   static const WCHAR wszBold[] = {'B','o','l','d',0};
//   static const WCHAR wszItalic[] = {'I','t','a','l','i','c',0};
//   static const WCHAR wszUnder[] = {'U','n','d','e','r','l','i','n','e',0};
//   static const WCHAR wszStrike[] = {'S','t','r','i','k','e','t','h','r','o','u','g','h',0};
//   static const WCHAR wszWeight[] = {'W','e','i','g','h','t',0};
//   static const WCHAR wszCharset[] = {'C','h','a','r','s','e','t',0};
//   static const LPCWSTR dispid_mapping[] =
//   {
//     wszName,
//     NULL,
//     wszSize,
//     wszBold,
//     wszItalic,
//     wszUnder,
//     wszStrike,
//     wszWeight,
//     wszCharset
//   };
//
//   IEnumConnections *pEnum;
//   CONNECTDATA CD;
//   HRESULT hres;
//
//   this->dirty = TRUE;
//
//   hres = IConnectionPoint_EnumConnections(this->pPropertyNotifyCP, &pEnum);
//   if (SUCCEEDED(hres))
//   {
//     while(IEnumConnections_Next(pEnum, 1, &CD, NULL) == S_OK) {
//       IPropertyNotifySink *sink;
//
//       IUnknown_QueryInterface(CD.pUnk, &IID_IPropertyNotifySink, (void**)&sink);
//       IPropertyNotifySink_OnChanged(sink, dispID);
//       IPropertyNotifySink_Release(sink);
//       IUnknown_Release(CD.pUnk);
//     }
//     IEnumConnections_Release(pEnum);
//   }
//
//   hres = IConnectionPoint_EnumConnections(this->pFontEventsCP, &pEnum);
//   if (SUCCEEDED(hres))
//   {
//     DISPPARAMS dispparams;
//     VARIANTARG vararg;
//
//     VariantInit(&vararg);
//     V_VT(&vararg) = VT_BSTR;
//     V_BSTR(&vararg) = SysAllocString(dispid_mapping[dispID]);
//
//     dispparams.cArgs = 1;
//     dispparams.cNamedArgs = 0;
//     dispparams.rgdispidNamedArgs = NULL;
//     dispparams.rgvarg = &vararg;
//
//     while(IEnumConnections_Next(pEnum, 1, &CD, NULL) == S_OK) {
//         IFontEventsDisp *disp;
//
//         IUnknown_QueryInterface(CD.pUnk, &IID_IFontEventsDisp, (void**)&disp);
//         IFontEventsDisp_Invoke(disp, DISPID_FONT_CHANGED, &IID_NULL,
//                                LOCALE_NEUTRAL, INVOKE_FUNC, &dispparams, NULL,
//                                NULL, NULL);
//
//         IFontEventsDisp_Release(disp);
//         IUnknown_Release(CD.pUnk);
//     }
//     VariantClear(&vararg);
//     IEnumConnections_Release(pEnum);
//   }
// }
//
// /************************************************************************
//  * OLEFontImpl_QueryInterface (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static HRESULT WINAPI OLEFontImpl_QueryInterface(
//   IFont*  iface,
//   REFIID  riid,
//   void**  ppvObject)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//
//   TRACE("(%p)->(%s, %p)\n", this, debugstr_guid(riid), ppvObject);
//
//   *ppvObject = 0;
//
//   if (IsEqualGUID(&IID_IUnknown, riid) ||
//       IsEqualGUID(&IID_IFont, riid))
//   {
//     *ppvObject = this;
//   }
//   else if (IsEqualGUID(&IID_IDispatch, riid) ||
//            IsEqualGUID(&IID_IFontDisp, riid))
//   {
//     *ppvObject = &this->IDispatch_iface;
//   }
//   else if (IsEqualGUID(&IID_IPersist, riid) ||
//            IsEqualGUID(&IID_IPersistStream, riid))
//   {
//     *ppvObject = &this->IPersistStream_iface;
//   }
//   else if (IsEqualGUID(&IID_IConnectionPointContainer, riid))
//   {
//     *ppvObject = &this->IConnectionPointContainer_iface;
//   }
//   else if (IsEqualGUID(&IID_IPersistPropertyBag, riid))
//   {
//     *ppvObject = &this->IPersistPropertyBag_iface;
//   }
//   else if (IsEqualGUID(&IID_IPersistStreamInit, riid))
//   {
//     *ppvObject = &this->IPersistStreamInit_iface;
//   }
//
//   if (!*ppvObject)
//   {
//     FIXME("() : asking for unsupported interface %s\n", debugstr_guid(riid));
//     return E_NOINTERFACE;
//   }
//
//   IFont_AddRef(iface);
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_AddRef (IUnknown)
//  */
// static ULONG WINAPI OLEFontImpl_AddRef(
//   IFont* iface)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(ref=%d)\n", this, this->ref);
//   return InterlockedIncrement(&this->ref);
// }
//
// /************************************************************************
//  * OLEFontImpl_Release (IUnknown)
//  */
// static ULONG WINAPI OLEFontImpl_Release(IFont* iface)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   ULONG ref;
//
//   TRACE("(%p)->(ref=%d)\n", this, this->ref);
//
//   ref = InterlockedDecrement(&this->ref);
//
//   if (ref == 0)
//   {
//     ULONG fontlist_refs = InterlockedDecrement(&ifont_cnt);
//
//     /* Final IFont object so destroy font cache */
//     if (fontlist_refs == 0)
//     {
//       HFONTItem *item, *cursor2;
//
//       EnterCriticalSection(&OLEFontImpl_csHFONTLIST);
//       LIST_FOR_EACH_ENTRY_SAFE(item, cursor2, &OLEFontImpl_hFontList, HFONTItem, entry)
//         HFONTItem_Delete(item);
//       LeaveCriticalSection(&OLEFontImpl_csHFONTLIST);
//       delete_dc();
//     }
//     else
//     {
//       dec_int_ref(this->gdiFont);
//     }
//     OLEFontImpl_Destroy(this);
//   }
//
//   return ref;
// }
//
// typedef struct
// {
//     short orig_cs;
//     short avail_cs;
// } enum_data;
//
// static int __stdcall font_enum_proc(const LOGFONTW *elf, const TEXTMETRICW *ntm, DWORD type, LPARAM lp)
// {
//     enum_data *data = (enum_data*)lp;
//
//     if(elf->lfCharSet == data->orig_cs)
//     {
//         data->avail_cs = data->orig_cs;
//         return 0;
//     }
//     if(data->avail_cs == -1) data->avail_cs = elf->lfCharSet;
//     return 1;
// }
//
// static void realize_font(OLEFontImpl *This)
// {
//     LOGFONTW logFont;
//     INT fontHeight;
//     WCHAR text_face[LF_FACESIZE];
//     HDC hdc = get_dc();
//     HFONT old_font;
//     TEXTMETRICW tm;
//
//     if (!This->dirty) return;
//
//     text_face[0] = 0;
//
//     if(This->gdiFont)
//     {
//         old_font = SelectObject(hdc, This->gdiFont);
//         GetTextFaceW(hdc, ARRAY_SIZE(text_face), text_face);
//         SelectObject(hdc, old_font);
//         dec_int_ref(This->gdiFont);
//         This->gdiFont = 0;
//     }
//
//     memset(&logFont, 0, sizeof(LOGFONTW));
//
//     lstrcpynW(logFont.lfFaceName, This->description.lpstrName, LF_FACESIZE);
//     logFont.lfCharSet = This->description.sCharset;
//
//     /* If the font name has been changed then enumerate all charsets
//        and pick one that'll result in the font specified being selected */
//     if(text_face[0] && lstrcmpiW(text_face, This->description.lpstrName))
//     {
//         enum_data data;
//         data.orig_cs = This->description.sCharset;
//         data.avail_cs = -1;
//         logFont.lfCharSet = DEFAULT_CHARSET;
//         EnumFontFamiliesExW(get_dc(), &logFont, font_enum_proc, (LPARAM)&data, 0);
//         if(data.avail_cs != -1) logFont.lfCharSet = data.avail_cs;
//     }
//
//     /*
//      * The height of the font returned by the get_Size property is the
//      * height of the font in points multiplied by 10000... Using some
//      * simple conversions and the ratio given by the application, it can
//      * be converted to a height in pixels.
//      *
//      * Standard ratio is 72 / 2540, or 18 / 635 in lowest terms.
//      * Ratio is applied here relative to the standard.
//      */
//
//     fontHeight = MulDiv( This->description.cySize.s.Lo, This->cyLogical*635, This->cyHimetric*18 );
//
//     logFont.lfHeight          = ((fontHeight%10000L)>5000L) ? (-fontHeight/10000L) - 1 :
//                                                                   (-fontHeight/10000L);
//     logFont.lfItalic          = This->description.fItalic;
//     logFont.lfUnderline       = This->description.fUnderline;
//     logFont.lfStrikeOut       = This->description.fStrikethrough;
//     logFont.lfWeight          = This->description.sWeight;
//     logFont.lfOutPrecision    = OUT_CHARACTER_PRECIS;
//     logFont.lfClipPrecision   = CLIP_DEFAULT_PRECIS;
//     logFont.lfQuality         = DEFAULT_QUALITY;
//     logFont.lfPitchAndFamily  = DEFAULT_PITCH;
//
//     This->gdiFont = CreateFontIndirectW(&logFont);
//     This->dirty = FALSE;
//
//     add_hfontitem(This->gdiFont);
//
//     /* Fixup the name and charset properties so that they match the
//        selected font */
//     old_font = SelectObject(get_dc(), This->gdiFont);
//     GetTextFaceW(hdc, ARRAY_SIZE(text_face), text_face);
//     if(lstrcmpiW(text_face, This->description.lpstrName))
//     {
//         HeapFree(GetProcessHeap(), 0, This->description.lpstrName);
//         This->description.lpstrName = strdupW(text_face);
//     }
//     GetTextMetricsW(hdc, &tm);
//     This->description.sCharset = tm.tmCharSet;
//     /* While we have it handy, stash the realized font height for use by get_Size() */
//     This->nRealHeight = tm.tmHeight - tm.tmInternalLeading; /* corresponds to LOGFONT lfHeight */
//     SelectObject(hdc, old_font);
// }
//
// /************************************************************************
//  * OLEFontImpl_get_Name (IFont)
//  *
//  * See Windows documentation for more details on IFont methods.
//  */
// static HRESULT WINAPI OLEFontImpl_get_Name(
//   IFont*  iface,
//   BSTR* pname)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%p)\n", this, pname);
//
//   if (pname==0)
//     return E_POINTER;
//
//   realize_font(this);
//
//   if (this->description.lpstrName!=0)
//     *pname = SysAllocString(this->description.lpstrName);
//   else
//     *pname = 0;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_put_Name (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_put_Name(
//   IFont* iface,
//   BSTR name)
// {
//   OLEFontImpl *This = impl_from_IFont(iface);
//   TRACE("(%p)->(%p)\n", This, name);
//
//   if (!name)
//     return CTL_E_INVALIDPROPERTYVALUE;
//
//   HeapFree(GetProcessHeap(), 0, This->description.lpstrName);
//   This->description.lpstrName = strdupW(name);
//   if (!This->description.lpstrName) return E_OUTOFMEMORY;
//
//   TRACE("new name %s\n", debugstr_w(This->description.lpstrName));
//   OLEFont_SendNotify(This, DISPID_FONT_NAME);
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_get_Size (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_get_Size(
//   IFont* iface,
//   CY*    psize)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%p)\n", this, psize);
//
//   if (!psize) return E_POINTER;
//
//   realize_font(this);
//
//   /*
//    * Convert realized font height in pixels to points descaled by current
//    * scaling ratio then scaled up by 10000.
//    */
//   psize->s.Lo = MulDiv(this->nRealHeight,
//                        this->cyHimetric * 72 * 10000,
//                        this->cyLogical * 2540);
//   psize->s.Hi = 0;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_put_Size (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_put_Size(
//   IFont* iface,
//   CY     size)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%d)\n", this, size.s.Lo);
//   this->description.cySize.s.Hi = 0;
//   this->description.cySize.s.Lo = size.s.Lo;
//   OLEFont_SendNotify(this, DISPID_FONT_SIZE);
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_get_Bold (IFont)
//  *
//  * See Windows documentation for more details on IFont methods.
//  */
// static HRESULT WINAPI OLEFontImpl_get_Bold(
//   IFont*  iface,
//   BOOL* pbold)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%p)\n", this, pbold);
//
//   if (!pbold) return E_POINTER;
//
//   realize_font(this);
//
//   *pbold = this->description.sWeight > 550;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_put_Bold (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_put_Bold(
//   IFont* iface,
//   BOOL bold)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%d)\n", this, bold);
//   this->description.sWeight = bold ? FW_BOLD : FW_NORMAL;
//   OLEFont_SendNotify(this, DISPID_FONT_BOLD);
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_get_Italic (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_get_Italic(
//   IFont*  iface,
//   BOOL* pitalic)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%p)\n", this, pitalic);
//
//   if (pitalic==0)
//     return E_POINTER;
//
//   realize_font(this);
//
//   *pitalic = this->description.fItalic;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_put_Italic (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_put_Italic(
//   IFont* iface,
//   BOOL italic)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%d)\n", this, italic);
//
//   this->description.fItalic = italic;
//
//   OLEFont_SendNotify(this, DISPID_FONT_ITALIC);
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_get_Underline (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_get_Underline(
//   IFont*  iface,
//   BOOL* punderline)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%p)\n", this, punderline);
//
//   if (punderline==0)
//     return E_POINTER;
//
//   realize_font(this);
//
//   *punderline = this->description.fUnderline;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_put_Underline (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_put_Underline(
//   IFont* iface,
//   BOOL underline)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%d)\n", this, underline);
//
//   this->description.fUnderline = underline;
//
//   OLEFont_SendNotify(this, DISPID_FONT_UNDER);
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_get_Strikethrough (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_get_Strikethrough(
//   IFont*  iface,
//   BOOL* pstrikethrough)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%p)\n", this, pstrikethrough);
//
//   if (pstrikethrough==0)
//     return E_POINTER;
//
//   realize_font(this);
//
//   *pstrikethrough = this->description.fStrikethrough;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_put_Strikethrough (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_put_Strikethrough(
//  IFont* iface,
//  BOOL strikethrough)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%d)\n", this, strikethrough);
//
//   this->description.fStrikethrough = strikethrough;
//   OLEFont_SendNotify(this, DISPID_FONT_STRIKE);
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_get_Weight (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_get_Weight(
//   IFont* iface,
//   short* pweight)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%p)\n", this, pweight);
//
//   if (pweight==0)
//     return E_POINTER;
//
//   realize_font(this);
//
//   *pweight = this->description.sWeight;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_put_Weight (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_put_Weight(
//   IFont* iface,
//   short  weight)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%d)\n", this, weight);
//
//   this->description.sWeight = weight;
//
//   OLEFont_SendNotify(this, DISPID_FONT_WEIGHT);
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_get_Charset (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_get_Charset(
//   IFont* iface,
//   short* pcharset)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%p)\n", this, pcharset);
//
//   if (pcharset==0)
//     return E_POINTER;
//
//   realize_font(this);
//
//   *pcharset = this->description.sCharset;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_put_Charset (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_put_Charset(
//   IFont* iface,
//   short charset)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%d)\n", this, charset);
//
//   this->description.sCharset = charset;
//   OLEFont_SendNotify(this, DISPID_FONT_CHARSET);
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_get_hFont (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_get_hFont(
//   IFont*   iface,
//   HFONT* phfont)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%p)\n", this, phfont);
//   if (phfont==NULL)
//     return E_POINTER;
//
//   realize_font(this);
//
//   *phfont = this->gdiFont;
//   TRACE("Returning %p\n", *phfont);
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_Clone (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_Clone(
//   IFont*  iface,
//   IFont** ppfont)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   OLEFontImpl* newObject;
//
//   TRACE("(%p)->(%p)\n", this, ppfont);
//
//   if (ppfont == NULL)
//     return E_POINTER;
//
//   *ppfont = NULL;
//
//   newObject = HeapAlloc(GetProcessHeap(), 0, sizeof(OLEFontImpl));
//   if (newObject==NULL)
//     return E_OUTOFMEMORY;
//
//   *newObject = *this;
//   /* allocate separate buffer */
//   newObject->description.lpstrName = strdupW(this->description.lpstrName);
//
//   /* Increment internal ref in hfont item list */
//   if(newObject->gdiFont) inc_int_ref(newObject->gdiFont);
//
//   InterlockedIncrement(&ifont_cnt);
//
//   newObject->pPropertyNotifyCP = NULL;
//   newObject->pFontEventsCP = NULL;
//   CreateConnectionPoint((IUnknown*)&newObject->IFont_iface, &IID_IPropertyNotifySink,
//                          &newObject->pPropertyNotifyCP);
//   CreateConnectionPoint((IUnknown*)&newObject->IFont_iface, &IID_IFontEventsDisp,
//                          &newObject->pFontEventsCP);
//
//   if (!newObject->pPropertyNotifyCP || !newObject->pFontEventsCP)
//   {
//     OLEFontImpl_Destroy(newObject);
//     return E_OUTOFMEMORY;
//   }
//
//   /* The cloned object starts with a reference count of 1 */
//   newObject->ref = 1;
//
//   *ppfont = &newObject->IFont_iface;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_IsEqual (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_IsEqual(
//   IFont* iface,
//   IFont* pFontOther)
// {
//   OLEFontImpl *left = impl_from_IFont(iface);
//   OLEFontImpl *right = impl_from_IFont(pFontOther);
//   INT ret;
//   INT left_len,right_len;
//
//   if(pFontOther == NULL)
//     return E_POINTER;
//   else if (left->description.cySize.s.Lo != right->description.cySize.s.Lo)
//     return S_FALSE;
//   else if (left->description.cySize.s.Hi != right->description.cySize.s.Hi)
//     return S_FALSE;
//   else if (left->description.sWeight != right->description.sWeight)
//     return S_FALSE;
//   else if (left->description.sCharset != right->description.sCharset)
//     return S_FALSE;
//   else if (left->description.fItalic != right->description.fItalic)
//     return S_FALSE;
//   else if (left->description.fUnderline != right->description.fUnderline)
//     return S_FALSE;
//   else if (left->description.fStrikethrough != right->description.fStrikethrough)
//     return S_FALSE;
//
//   /* Check from string */
//   left_len = lstrlenW(left->description.lpstrName);
//   right_len = lstrlenW(right->description.lpstrName);
//   ret = CompareStringW(0,0,left->description.lpstrName, left_len,
//     right->description.lpstrName, right_len);
//   if (ret != CSTR_EQUAL)
//     return S_FALSE;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_SetRatio (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_SetRatio(
//   IFont* iface,
//   LONG   cyLogical,
//   LONG   cyHimetric)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   TRACE("(%p)->(%d, %d)\n", this, cyLogical, cyHimetric);
//
//   if(cyLogical == 0 || cyHimetric == 0)
//     return E_FAIL;
//
//   /* cyLogical and cyHimetric both set to 1 is a special case that
//      does not change the scaling but also does not fail */
//   if(cyLogical == 1 && cyHimetric == 1)
//     return S_OK;
//
//   this->cyLogical  = cyLogical;
//   this->cyHimetric = cyHimetric;
//   this->dirty = TRUE;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_QueryTextMetrics (IFont)
//  */
// static HRESULT      WINAPI OLEFontImpl_QueryTextMetrics(
//   IFont*         iface,
//   TEXTMETRICOLE* ptm)
// {
//   HDC hdcRef;
//   HFONT hOldFont, hNewFont;
//
//   hdcRef = GetDC(0);
//   IFont_get_hFont(iface, &hNewFont);
//   hOldFont = SelectObject(hdcRef, hNewFont);
//   GetTextMetricsW(hdcRef, ptm);
//   SelectObject(hdcRef, hOldFont);
//   ReleaseDC(0, hdcRef);
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_AddRefHfont (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_AddRefHfont(
//   IFont*  iface,
//   HFONT hfont)
// {
//     OLEFontImpl *this = impl_from_IFont(iface);
//
//     TRACE("(%p)->(%p)\n", this, hfont);
//
//     if (!hfont) return E_INVALIDARG;
//
//     return inc_ext_ref(hfont);
// }
//
// /************************************************************************
//  * OLEFontImpl_ReleaseHfont (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_ReleaseHfont(
//   IFont*  iface,
//   HFONT hfont)
// {
//     OLEFontImpl *this = impl_from_IFont(iface);
//
//     TRACE("(%p)->(%p)\n", this, hfont);
//
//     if (!hfont) return E_INVALIDARG;
//
//     return dec_ext_ref(hfont);
// }
//
// /************************************************************************
//  * OLEFontImpl_SetHdc (IFont)
//  */
// static HRESULT WINAPI OLEFontImpl_SetHdc(
//   IFont* iface,
//   HDC  hdc)
// {
//   OLEFontImpl *this = impl_from_IFont(iface);
//   FIXME("(%p)->(%p): Stub\n", this, hdc);
//   return E_NOTIMPL;
// }
//
// static const IFontVtbl OLEFontImpl_VTable =
// {
//   OLEFontImpl_QueryInterface,
//   OLEFontImpl_AddRef,
//   OLEFontImpl_Release,
//   OLEFontImpl_get_Name,
//   OLEFontImpl_put_Name,
//   OLEFontImpl_get_Size,
//   OLEFontImpl_put_Size,
//   OLEFontImpl_get_Bold,
//   OLEFontImpl_put_Bold,
//   OLEFontImpl_get_Italic,
//   OLEFontImpl_put_Italic,
//   OLEFontImpl_get_Underline,
//   OLEFontImpl_put_Underline,
//   OLEFontImpl_get_Strikethrough,
//   OLEFontImpl_put_Strikethrough,
//   OLEFontImpl_get_Weight,
//   OLEFontImpl_put_Weight,
//   OLEFontImpl_get_Charset,
//   OLEFontImpl_put_Charset,
//   OLEFontImpl_get_hFont,
//   OLEFontImpl_Clone,
//   OLEFontImpl_IsEqual,
//   OLEFontImpl_SetRatio,
//   OLEFontImpl_QueryTextMetrics,
//   OLEFontImpl_AddRefHfont,
//   OLEFontImpl_ReleaseHfont,
//   OLEFontImpl_SetHdc
// };
//
// /************************************************************************
//  * OLEFontImpl_IDispatch_QueryInterface (IUnknown)
//  */
// static HRESULT WINAPI OLEFontImpl_IDispatch_QueryInterface(
//   IDispatch* iface,
//   REFIID     riid,
//   VOID**     ppvoid)
// {
//   OLEFontImpl *this = impl_from_IDispatch(iface);
//   return IFont_QueryInterface(&this->IFont_iface, riid, ppvoid);
// }
//
// /************************************************************************
//  * OLEFontImpl_IDispatch_Release (IUnknown)
//  */
// static ULONG WINAPI OLEFontImpl_IDispatch_Release(
//   IDispatch* iface)
// {
//   OLEFontImpl *this = impl_from_IDispatch(iface);
//   return IFont_Release(&this->IFont_iface);
// }
//
// /************************************************************************
//  * OLEFontImpl_IDispatch_AddRef (IUnknown)
//  */
// static ULONG WINAPI OLEFontImpl_IDispatch_AddRef(
//   IDispatch* iface)
// {
//   OLEFontImpl *this = impl_from_IDispatch(iface);
//   return IFont_AddRef(&this->IFont_iface);
// }
//
// /************************************************************************
//  * OLEFontImpl_GetTypeInfoCount (IDispatch)
//  */
// static HRESULT WINAPI OLEFontImpl_GetTypeInfoCount(
//   IDispatch*    iface,
//   unsigned int* pctinfo)
// {
//   OLEFontImpl *this = impl_from_IDispatch(iface);
//   TRACE("(%p)->(%p)\n", this, pctinfo);
//   *pctinfo = 1;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_GetTypeInfo (IDispatch)
//  */
// static HRESULT WINAPI OLEFontImpl_GetTypeInfo(
//   IDispatch*  iface,
//   UINT      iTInfo,
//   LCID        lcid,
//   ITypeInfo** ppTInfo)
// {
//   static const WCHAR stdole2tlb[] = {'s','t','d','o','l','e','2','.','t','l','b',0};
//   ITypeLib *tl;
//   HRESULT hres;
//
//   OLEFontImpl *this = impl_from_IDispatch(iface);
//   TRACE("(%p, iTInfo=%d, lcid=%04x, %p)\n", this, iTInfo, (int)lcid, ppTInfo);
//   if (iTInfo != 0)
//     return E_FAIL;
//   hres = LoadTypeLib(stdole2tlb, &tl);
//   if (FAILED(hres)) {
//     ERR("Could not load the stdole2.tlb?\n");
//     return hres;
//   }
//   hres = ITypeLib_GetTypeInfoOfGuid(tl, &IID_IFontDisp, ppTInfo);
//   ITypeLib_Release(tl);
//   if (FAILED(hres)) {
//     FIXME("Did not IDispatch typeinfo from typelib, hres %x\n",hres);
//   }
//   return hres;
// }
//
// /************************************************************************
//  * OLEFontImpl_GetIDsOfNames (IDispatch)
//  */
// static HRESULT WINAPI OLEFontImpl_GetIDsOfNames(
//   IDispatch*  iface,
//   REFIID      riid,
//   LPOLESTR* rgszNames,
//   UINT      cNames,
//   LCID        lcid,
//   DISPID*     rgDispId)
// {
//   ITypeInfo * pTInfo;
//   HRESULT hres;
//
//   OLEFontImpl *this = impl_from_IDispatch(iface);
//
//   TRACE("(%p,%s,%p,cNames=%d,lcid=%04x,%p)\n", this, debugstr_guid(riid),
//         rgszNames, cNames, (int)lcid, rgDispId);
//
//   if (cNames == 0) return E_INVALIDARG;
//
//   hres = IDispatch_GetTypeInfo(iface, 0, lcid, &pTInfo);
//   if (FAILED(hres))
//   {
//     ERR("GetTypeInfo failed.\n");
//     return hres;
//   }
//
//   /* convert names to DISPIDs */
//   hres = DispGetIDsOfNames (pTInfo, rgszNames, cNames, rgDispId);
//   ITypeInfo_Release(pTInfo);
//
//   return hres;
// }
//
// /************************************************************************
//  * OLEFontImpl_Invoke (IDispatch)
//  * 
//  */
// static HRESULT WINAPI OLEFontImpl_Invoke(
//   IDispatch*  iface,
//   DISPID      dispIdMember,
//   REFIID      riid,
//   LCID        lcid,
//   WORD        wFlags,
//   DISPPARAMS* pDispParams,
//   VARIANT*    pVarResult,
//   EXCEPINFO*  pExepInfo,
//   UINT*     puArgErr)
// {
//   OLEFontImpl *this = impl_from_IDispatch(iface);
//   HRESULT hr;
//
//   TRACE("%p->(%d,%s,0x%x,0x%x,%p,%p,%p,%p)\n", this, dispIdMember,
//     debugstr_guid(riid), lcid, wFlags, pDispParams, pVarResult, pExepInfo,
//     puArgErr);
//
//   /* validate parameters */
//
//   if (!IsEqualIID(riid, &IID_NULL))
//   {
//     ERR("riid was %s instead of IID_NULL\n", debugstr_guid(riid));
//     return DISP_E_UNKNOWNINTERFACE;
//   }
//
//   if (wFlags & DISPATCH_PROPERTYGET)
//   {
//     if (!pVarResult)
//     {
//       ERR("null pVarResult not allowed when DISPATCH_PROPERTYGET specified\n");
//       return DISP_E_PARAMNOTOPTIONAL;
//     }
//   }
//   else if (wFlags & DISPATCH_PROPERTYPUT)
//   {
//     if (!pDispParams)
//     {
//       ERR("null pDispParams not allowed when DISPATCH_PROPERTYPUT specified\n");
//       return DISP_E_PARAMNOTOPTIONAL;
//     }
//     if (pDispParams->cArgs != 1)
//     {
//       ERR("param count for DISPATCH_PROPERTYPUT was %d instead of 1\n", pDispParams->cArgs);
//       return DISP_E_BADPARAMCOUNT;
//     }
//   }
//   else
//   {
//     ERR("one of DISPATCH_PROPERTYGET or DISPATCH_PROPERTYPUT must be specified\n");
//     return DISP_E_MEMBERNOTFOUND;
//   }
//
//   switch (dispIdMember) {
//   case DISPID_FONT_NAME:
//     if (wFlags & DISPATCH_PROPERTYGET) {
//       V_VT(pVarResult) = VT_BSTR;
//       return IFont_get_Name(&this->IFont_iface, &V_BSTR(pVarResult));
//     } else {
//       VARIANTARG vararg;
//
//       VariantInit(&vararg);
//       hr = VariantChangeTypeEx(&vararg, &pDispParams->rgvarg[0], lcid, 0, VT_BSTR);
//       if (FAILED(hr))
//         return hr;
//
//       hr = IFont_put_Name(&this->IFont_iface, V_BSTR(&vararg));
//
//       VariantClear(&vararg);
//       return hr;
//     }
//     break;
//   case DISPID_FONT_BOLD:
//     if (wFlags & DISPATCH_PROPERTYGET) {
//       BOOL value;
//       hr = IFont_get_Bold(&this->IFont_iface, &value);
//       V_VT(pVarResult) = VT_BOOL;
//       V_BOOL(pVarResult) = value ? VARIANT_TRUE : VARIANT_FALSE;
//       return hr;
//     } else {
//       VARIANTARG vararg;
//
//       VariantInit(&vararg);
//       hr = VariantChangeTypeEx(&vararg, &pDispParams->rgvarg[0], lcid, 0, VT_BOOL);
//       if (FAILED(hr))
//         return hr;
//
//       hr = IFont_put_Bold(&this->IFont_iface, V_BOOL(&vararg));
//
//       VariantClear(&vararg);
//       return hr;
//     }
//     break;
//   case DISPID_FONT_ITALIC:
//     if (wFlags & DISPATCH_PROPERTYGET) {
//       BOOL value;
//       hr = IFont_get_Italic(&this->IFont_iface, &value);
//       V_VT(pVarResult) = VT_BOOL;
//       V_BOOL(pVarResult) = value ? VARIANT_TRUE : VARIANT_FALSE;
//       return hr;
//     } else {
//       VARIANTARG vararg;
//
//       VariantInit(&vararg);
//       hr = VariantChangeTypeEx(&vararg, &pDispParams->rgvarg[0], lcid, 0, VT_BOOL);
//       if (FAILED(hr))
//         return hr;
//
//       hr = IFont_put_Italic(&this->IFont_iface, V_BOOL(&vararg));
//
//       VariantClear(&vararg);
//       return hr;
//     }
//     break;
//   case DISPID_FONT_UNDER:
//     if (wFlags & DISPATCH_PROPERTYGET) {
//       BOOL value;
//       hr = IFont_get_Underline(&this->IFont_iface, &value);
//       V_VT(pVarResult) = VT_BOOL;
//       V_BOOL(pVarResult) = value ? VARIANT_TRUE : VARIANT_FALSE;
//       return hr;
//     } else {
//       VARIANTARG vararg;
//
//       VariantInit(&vararg);
//       hr = VariantChangeTypeEx(&vararg, &pDispParams->rgvarg[0], lcid, 0, VT_BOOL);
//       if (FAILED(hr))
//         return hr;
//
//       hr = IFont_put_Underline(&this->IFont_iface, V_BOOL(&vararg));
//
//       VariantClear(&vararg);
//       return hr;
//     }
//     break;
//   case DISPID_FONT_STRIKE:
//     if (wFlags & DISPATCH_PROPERTYGET) {
//       BOOL value;
//       hr = IFont_get_Strikethrough(&this->IFont_iface, &value);
//       V_VT(pVarResult) = VT_BOOL;
//       V_BOOL(pVarResult) = value ? VARIANT_TRUE : VARIANT_FALSE;
//       return hr;
//     } else {
//       VARIANTARG vararg;
//
//       VariantInit(&vararg);
//       hr = VariantChangeTypeEx(&vararg, &pDispParams->rgvarg[0], lcid, 0, VT_BOOL);
//       if (FAILED(hr))
//         return hr;
//
//       hr = IFont_put_Strikethrough(&this->IFont_iface, V_BOOL(&vararg));
//
//       VariantClear(&vararg);
//       return hr;
//     }
//     break;
//   case DISPID_FONT_SIZE:
//     if (wFlags & DISPATCH_PROPERTYGET) {
//       V_VT(pVarResult) = VT_CY;
//       return IFont_get_Size(&this->IFont_iface, &V_CY(pVarResult));
//     } else {
//       VARIANTARG vararg;
//
//       VariantInit(&vararg);
//       hr = VariantChangeTypeEx(&vararg, &pDispParams->rgvarg[0], lcid, 0, VT_CY);
//       if (FAILED(hr))
//         return hr;
//
//       hr = IFont_put_Size(&this->IFont_iface, V_CY(&vararg));
//
//       VariantClear(&vararg);
//       return hr;
//     }
//     break;
//   case DISPID_FONT_WEIGHT:
//     if (wFlags & DISPATCH_PROPERTYGET) {
//       V_VT(pVarResult) = VT_I2;
//       return IFont_get_Weight(&this->IFont_iface, &V_I2(pVarResult));
//     } else {
//       VARIANTARG vararg;
//
//       VariantInit(&vararg);
//       hr = VariantChangeTypeEx(&vararg, &pDispParams->rgvarg[0], lcid, 0, VT_I2);
//       if (FAILED(hr))
//         return hr;
//
//       hr = IFont_put_Weight(&this->IFont_iface, V_I2(&vararg));
//
//       VariantClear(&vararg);
//       return hr;
//     }
//     break;
//   case DISPID_FONT_CHARSET:
//     if (wFlags & DISPATCH_PROPERTYGET) {
//       V_VT(pVarResult) = VT_I2;
//       return OLEFontImpl_get_Charset(&this->IFont_iface, &V_I2(pVarResult));
//     } else {
//       VARIANTARG vararg;
//
//       VariantInit(&vararg);
//       hr = VariantChangeTypeEx(&vararg, &pDispParams->rgvarg[0], lcid, 0, VT_I2);
//       if (FAILED(hr))
//         return hr;
//
//       hr = IFont_put_Charset(&this->IFont_iface, V_I2(&vararg));
//
//       VariantClear(&vararg);
//       return hr;
//     }
//     break;
//   default:
//     ERR("member not found for dispid 0x%x\n", dispIdMember);
//     return DISP_E_MEMBERNOTFOUND;
//   }
// }
//
// static const IDispatchVtbl OLEFontImpl_IDispatch_VTable =
// {
//   OLEFontImpl_IDispatch_QueryInterface,
//   OLEFontImpl_IDispatch_AddRef,
//   OLEFontImpl_IDispatch_Release,
//   OLEFontImpl_GetTypeInfoCount,
//   OLEFontImpl_GetTypeInfo,
//   OLEFontImpl_GetIDsOfNames,
//   OLEFontImpl_Invoke
// };
//
// /************************************************************************
//  * OLEFontImpl_IPersistStream_QueryInterface (IUnknown)
//  */
// static HRESULT WINAPI OLEFontImpl_IPersistStream_QueryInterface(
//   IPersistStream* iface,
//   REFIID     riid,
//   VOID**     ppvoid)
// {
//   OLEFontImpl *this = impl_from_IPersistStream(iface);
//
//   return IFont_QueryInterface(&this->IFont_iface, riid, ppvoid);
// }
//
// /************************************************************************
//  * OLEFontImpl_IPersistStream_Release (IUnknown)
//  */
// static ULONG WINAPI OLEFontImpl_IPersistStream_Release(
//   IPersistStream* iface)
// {
//   OLEFontImpl *this = impl_from_IPersistStream(iface);
//
//   return IFont_Release(&this->IFont_iface);
// }
//
// /************************************************************************
//  * OLEFontImpl_IPersistStream_AddRef (IUnknown)
//  */
// static ULONG WINAPI OLEFontImpl_IPersistStream_AddRef(
//   IPersistStream* iface)
// {
//   OLEFontImpl *this = impl_from_IPersistStream(iface);
//
//   return IFont_AddRef(&this->IFont_iface);
// }
//
// /************************************************************************
//  * OLEFontImpl_GetClassID (IPersistStream)
//  */
// static HRESULT WINAPI OLEFontImpl_GetClassID(
//   IPersistStream* iface,
//   CLSID*                pClassID)
// {
//   TRACE("(%p,%p)\n",iface,pClassID);
//   if (pClassID==0)
//     return E_POINTER;
//
//   *pClassID = CLSID_StdFont;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_IsDirty (IPersistStream)
//  *
//  * See Windows documentation for more details on IPersistStream methods.
//  */
// static HRESULT WINAPI OLEFontImpl_IsDirty(
//   IPersistStream*  iface)
// {
//   TRACE("(%p)\n",iface);
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_Load (IPersistStream)
//  *
//  * See Windows documentation for more details on IPersistStream methods.
//  *
//  * This is the format of the standard font serialization as far as I
//  * know
//  *
//  * Offset   Type   Value           Comment
//  * 0x0000   Byte   Unknown         Probably a version number, contains 0x01
//  * 0x0001   Short  Charset         Charset value from the FONTDESC structure
//  * 0x0003   Byte   Attributes      Flags defined as follows:
//  *                                     00000010 - Italic
//  *                                     00000100 - Underline
//  *                                     00001000 - Strikethrough
//  * 0x0004   Short  Weight          Weight value from FONTDESC structure
//  * 0x0006   DWORD  size            "Low" portion of the cySize member of the FONTDESC
//  *                                 structure/
//  * 0x000A   Byte   name length     Length of the font name string (no null character)
//  * 0x000B   String name            Name of the font (ASCII, no nul character)
//  */
// static HRESULT WINAPI OLEFontImpl_Load(
//   IPersistStream*  iface,
//   IStream*         pLoadStream)
// {
//   OLEFontImpl *this = impl_from_IPersistStream(iface);
//   BYTE  version, attributes, string_size;
//   char readBuffer[0x100];
//   ULONG cbRead;
//   INT len;
//
//   /* Version */
//   IStream_Read(pLoadStream, &version, sizeof(BYTE), &cbRead);
//   if ((cbRead != sizeof(BYTE)) || (version != 0x01)) return E_FAIL;
//
//   /* Charset */
//   IStream_Read(pLoadStream, &this->description.sCharset, sizeof(WORD), &cbRead);
//   if (cbRead != sizeof(WORD)) return E_FAIL;
//
//   /* Attributes */
//   IStream_Read(pLoadStream, &attributes, sizeof(BYTE), &cbRead);
//   if (cbRead != sizeof(BYTE)) return E_FAIL;
//
//   this->description.fItalic        = (attributes & FONTPERSIST_ITALIC) != 0;
//   this->description.fStrikethrough = (attributes & FONTPERSIST_STRIKETHROUGH) != 0;
//   this->description.fUnderline     = (attributes & FONTPERSIST_UNDERLINE) != 0;
//
//   /* Weight */
//   IStream_Read(pLoadStream, &this->description.sWeight, sizeof(WORD), &cbRead);
//   if (cbRead != sizeof(WORD)) return E_FAIL;
//
//   /* Size */
//   IStream_Read(pLoadStream, &this->description.cySize.s.Lo, sizeof(DWORD), &cbRead);
//   if (cbRead != sizeof(DWORD)) return E_FAIL;
//
//   this->description.cySize.s.Hi = 0;
//
//   /* Name */
//   IStream_Read(pLoadStream, &string_size, sizeof(BYTE), &cbRead);
//   if (cbRead != sizeof(BYTE)) return E_FAIL;
//
//   IStream_Read(pLoadStream, readBuffer, string_size, &cbRead);
//   if (cbRead != string_size) return E_FAIL;
//
//   HeapFree(GetProcessHeap(), 0, this->description.lpstrName);
//
//   len = MultiByteToWideChar( CP_ACP, 0, readBuffer, string_size, NULL, 0 );
//   this->description.lpstrName = HeapAlloc( GetProcessHeap(), 0, (len+1) * sizeof(WCHAR) );
//   MultiByteToWideChar( CP_ACP, 0, readBuffer, string_size, this->description.lpstrName, len );
//   this->description.lpstrName[len] = 0;
//
//   /* Ensure use of this font causes a new one to be created */
//   dec_int_ref(this->gdiFont);
//   this->gdiFont = 0;
//   this->dirty = TRUE;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_Save (IPersistStream)
//  */
// static HRESULT WINAPI OLEFontImpl_Save(
//   IPersistStream*  iface,
//   IStream*         pOutStream,
//   BOOL             fClearDirty)
// {
//   OLEFontImpl *this = impl_from_IPersistStream(iface);
//   BYTE  attributes, string_size;
//   const BYTE version = 0x01;
//   char* writeBuffer = NULL;
//   ULONG written;
//
//   TRACE("(%p)->(%p %d)\n", this, pOutStream, fClearDirty);
//
//   /* Version */
//   IStream_Write(pOutStream, &version, sizeof(BYTE), &written);
//   if (written != sizeof(BYTE)) return E_FAIL;
//
//   /* Charset */
//   IStream_Write(pOutStream, &this->description.sCharset, sizeof(WORD), &written);
//   if (written != sizeof(WORD)) return E_FAIL;
//
//   /* Attributes */
//   attributes = 0;
//
//   if (this->description.fItalic)
//     attributes |= FONTPERSIST_ITALIC;
//
//   if (this->description.fStrikethrough)
//     attributes |= FONTPERSIST_STRIKETHROUGH;
//
//   if (this->description.fUnderline)
//     attributes |= FONTPERSIST_UNDERLINE;
//
//   IStream_Write(pOutStream, &attributes, sizeof(BYTE), &written);
//   if (written != sizeof(BYTE)) return E_FAIL;
//
//   /* Weight */
//   IStream_Write(pOutStream, &this->description.sWeight, sizeof(WORD), &written);
//   if (written != sizeof(WORD)) return E_FAIL;
//
//   /* Size */
//   IStream_Write(pOutStream, &this->description.cySize.s.Lo, sizeof(DWORD), &written);
//   if (written != sizeof(DWORD)) return E_FAIL;
//
//   /* FontName */
//   if (this->description.lpstrName)
//     string_size = WideCharToMultiByte( CP_ACP, 0, this->description.lpstrName,
//                                        lstrlenW(this->description.lpstrName), NULL, 0, NULL, NULL );
//   else
//     string_size = 0;
//
//   IStream_Write(pOutStream, &string_size, sizeof(BYTE), &written);
//   if (written != sizeof(BYTE)) return E_FAIL;
//
//   if (string_size)
//   {
//       if (!(writeBuffer = HeapAlloc( GetProcessHeap(), 0, string_size ))) return E_OUTOFMEMORY;
//       WideCharToMultiByte( CP_ACP, 0, this->description.lpstrName,
//                            lstrlenW(this->description.lpstrName),
//                            writeBuffer, string_size, NULL, NULL );
//
//       IStream_Write(pOutStream, writeBuffer, string_size, &written);
//       HeapFree(GetProcessHeap(), 0, writeBuffer);
//
//       if (written != string_size) return E_FAIL;
//   }
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEFontImpl_GetSizeMax (IPersistStream)
//  */
// static HRESULT WINAPI OLEFontImpl_GetSizeMax(
//   IPersistStream*  iface,
//   ULARGE_INTEGER*  pcbSize)
// {
//   OLEFontImpl *this = impl_from_IPersistStream(iface);
//
//   if (pcbSize==NULL)
//     return E_POINTER;
//
//   pcbSize->u.HighPart = 0;
//   pcbSize->u.LowPart = 0;
//
//   pcbSize->u.LowPart += sizeof(BYTE);  /* Version */
//   pcbSize->u.LowPart += sizeof(WORD);  /* Lang code */
//   pcbSize->u.LowPart += sizeof(BYTE);  /* Flags */
//   pcbSize->u.LowPart += sizeof(WORD);  /* Weight */
//   pcbSize->u.LowPart += sizeof(DWORD); /* Size */
//   pcbSize->u.LowPart += sizeof(BYTE);  /* StrLength */
//
//   if (this->description.lpstrName!=0)
//       pcbSize->u.LowPart += WideCharToMultiByte( CP_ACP, 0, this->description.lpstrName,
//                                                  lstrlenW(this->description.lpstrName),
//                                                  NULL, 0, NULL, NULL );
//
//   return S_OK;
// }
//
// static const IPersistStreamVtbl OLEFontImpl_IPersistStream_VTable =
// {
//   OLEFontImpl_IPersistStream_QueryInterface,
//   OLEFontImpl_IPersistStream_AddRef,
//   OLEFontImpl_IPersistStream_Release,
//   OLEFontImpl_GetClassID,
//   OLEFontImpl_IsDirty,
//   OLEFontImpl_Load,
//   OLEFontImpl_Save,
//   OLEFontImpl_GetSizeMax
// };
//
// /************************************************************************
//  * OLEFontImpl_IConnectionPointContainer_QueryInterface (IUnknown)
//  */
// static HRESULT WINAPI OLEFontImpl_IConnectionPointContainer_QueryInterface(
//   IConnectionPointContainer* iface,
//   REFIID     riid,
//   VOID**     ppvoid)
// {
//   OLEFontImpl *this = impl_from_IConnectionPointContainer(iface);
//
//   return IFont_QueryInterface(&this->IFont_iface, riid, ppvoid);
// }
//
// /************************************************************************
//  * OLEFontImpl_IConnectionPointContainer_Release (IUnknown)
//  */
// static ULONG WINAPI OLEFontImpl_IConnectionPointContainer_Release(
//   IConnectionPointContainer* iface)
// {
//   OLEFontImpl *this = impl_from_IConnectionPointContainer(iface);
//
//   return IFont_Release(&this->IFont_iface);
// }
//
// /************************************************************************
//  * OLEFontImpl_IConnectionPointContainer_AddRef (IUnknown)
//  */
// static ULONG WINAPI OLEFontImpl_IConnectionPointContainer_AddRef(
//   IConnectionPointContainer* iface)
// {
//   OLEFontImpl *this = impl_from_IConnectionPointContainer(iface);
//
//   return IFont_AddRef(&this->IFont_iface);
// }
//
// /************************************************************************
//  * OLEFontImpl_EnumConnectionPoints (IConnectionPointContainer)
//  */
// static HRESULT WINAPI OLEFontImpl_EnumConnectionPoints(
//   IConnectionPointContainer* iface,
//   IEnumConnectionPoints **ppEnum)
// {
//   OLEFontImpl *this = impl_from_IConnectionPointContainer(iface);
//
//   FIXME("(%p)->(%p): stub\n", this, ppEnum);
//   return E_NOTIMPL;
// }
//
// /************************************************************************
//  * OLEFontImpl_FindConnectionPoint (IConnectionPointContainer)
//  */
// static HRESULT WINAPI OLEFontImpl_FindConnectionPoint(
//    IConnectionPointContainer* iface,
//    REFIID riid,
//    IConnectionPoint **ppCp)
// {
//   OLEFontImpl *this = impl_from_IConnectionPointContainer(iface);
//   TRACE("(%p)->(%s, %p)\n", this, debugstr_guid(riid), ppCp);
//
//   if(IsEqualIID(riid, &IID_IPropertyNotifySink)) {
//     return IConnectionPoint_QueryInterface(this->pPropertyNotifyCP, &IID_IConnectionPoint,
//                                            (void**)ppCp);
//   } else if(IsEqualIID(riid, &IID_IFontEventsDisp)) {
//     return IConnectionPoint_QueryInterface(this->pFontEventsCP, &IID_IConnectionPoint,
//                                            (void**)ppCp);
//   } else {
//     FIXME("no connection point for %s\n", debugstr_guid(riid));
//     return CONNECT_E_NOCONNECTION;
//   }
// }
//
// static const IConnectionPointContainerVtbl
//      OLEFontImpl_IConnectionPointContainer_VTable =
// {
//   OLEFontImpl_IConnectionPointContainer_QueryInterface,
//   OLEFontImpl_IConnectionPointContainer_AddRef,
//   OLEFontImpl_IConnectionPointContainer_Release,
//   OLEFontImpl_EnumConnectionPoints,
//   OLEFontImpl_FindConnectionPoint
// };
//
// /************************************************************************
//  * OLEFontImpl implementation of IPersistPropertyBag.
//  */
// static HRESULT WINAPI OLEFontImpl_IPersistPropertyBag_QueryInterface(
//    IPersistPropertyBag *iface, REFIID riid, LPVOID *ppvObj
// ) {
//   OLEFontImpl *this = impl_from_IPersistPropertyBag(iface);
//   return IFont_QueryInterface(&this->IFont_iface,riid,ppvObj);
// }
//
// static ULONG WINAPI OLEFontImpl_IPersistPropertyBag_AddRef(
//    IPersistPropertyBag *iface
// ) {
//   OLEFontImpl *this = impl_from_IPersistPropertyBag(iface);
//   return IFont_AddRef(&this->IFont_iface);
// }
//
// static ULONG WINAPI OLEFontImpl_IPersistPropertyBag_Release(
//    IPersistPropertyBag *iface
// ) {
//   OLEFontImpl *this = impl_from_IPersistPropertyBag(iface);
//   return IFont_Release(&this->IFont_iface);
// }
//
// static HRESULT WINAPI OLEFontImpl_IPersistPropertyBag_GetClassID(
//    IPersistPropertyBag *iface, CLSID *classid
// ) {
//   FIXME("(%p,%p), stub!\n", iface, classid);
//   return E_FAIL;
// }
//
// static HRESULT WINAPI OLEFontImpl_IPersistPropertyBag_InitNew(
//    IPersistPropertyBag *iface
// ) {
//   FIXME("(%p), stub!\n", iface);
//   return S_OK;
// }
//
// static HRESULT WINAPI OLEFontImpl_IPersistPropertyBag_Load(
//    IPersistPropertyBag *iface, IPropertyBag* pPropBag, IErrorLog* pErrorLog
// ) {
// /* (from Visual Basic 6 property bag)
//          Name            =   "MS Sans Serif"
//          Size            =   13.8
//          Charset         =   0
//          Weight          =   400
//          Underline       =   0   'False
//          Italic          =   0   'False
//          Strikethrough   =   0   'False
// */
//     static const WCHAR sAttrName[] = {'N','a','m','e',0};
//     static const WCHAR sAttrSize[] = {'S','i','z','e',0};
//     static const WCHAR sAttrCharset[] = {'C','h','a','r','s','e','t',0};
//     static const WCHAR sAttrWeight[] = {'W','e','i','g','h','t',0};
//     static const WCHAR sAttrUnderline[] = {'U','n','d','e','r','l','i','n','e',0};
//     static const WCHAR sAttrItalic[] = {'I','t','a','l','i','c',0};
//     static const WCHAR sAttrStrikethrough[] = {'S','t','r','i','k','e','t','h','r','o','u','g','h',0};
//     OLEFontImpl *this = impl_from_IPersistPropertyBag(iface);
//     VARIANT value;
//     HRESULT iRes;
//
//     VariantInit(&value);
//
//     iRes = IPropertyBag_Read(pPropBag, sAttrName, &value, pErrorLog);
//     if (iRes == S_OK)
//     {
//         iRes = VariantChangeType(&value, &value, 0, VT_BSTR);
//         if (iRes == S_OK)
//             iRes = IFont_put_Name(&this->IFont_iface, V_BSTR(&value));
//     }
//     else if (iRes == E_INVALIDARG)
//         iRes = S_OK;
//
//     VariantClear(&value);
//
//     if (iRes == S_OK) {
//         iRes = IPropertyBag_Read(pPropBag, sAttrSize, &value, pErrorLog);
//         if (iRes == S_OK)
//         {
//             iRes = VariantChangeType(&value, &value, 0, VT_CY);
//             if (iRes == S_OK)
//                 iRes = IFont_put_Size(&this->IFont_iface, V_CY(&value));
//         }
//         else if (iRes == E_INVALIDARG)
//             iRes = S_OK;
//
//         VariantClear(&value);
//     }
//
//     if (iRes == S_OK) {
//         iRes = IPropertyBag_Read(pPropBag, sAttrCharset, &value, pErrorLog);
//         if (iRes == S_OK)
//         {
//             iRes = VariantChangeType(&value, &value, 0, VT_I2);
//             if (iRes == S_OK)
//                 iRes = IFont_put_Charset(&this->IFont_iface, V_I2(&value));
//         }
//         else if (iRes == E_INVALIDARG)
//             iRes = S_OK;
//
//         VariantClear(&value);
//     }
//
//     if (iRes == S_OK) {
//         iRes = IPropertyBag_Read(pPropBag, sAttrWeight, &value, pErrorLog);
//         if (iRes == S_OK)
//         {
//             iRes = VariantChangeType(&value, &value, 0, VT_I2);
//             if (iRes == S_OK)
//                 iRes = IFont_put_Weight(&this->IFont_iface, V_I2(&value));
//         }
//         else if (iRes == E_INVALIDARG)
//             iRes = S_OK;
//
//         VariantClear(&value);
//     }
//
//     if (iRes == S_OK) {
//         iRes = IPropertyBag_Read(pPropBag, sAttrUnderline, &value, pErrorLog);
//         if (iRes == S_OK)
//         {
//             iRes = VariantChangeType(&value, &value, 0, VT_BOOL);
//             if (iRes == S_OK)
//                 iRes = IFont_put_Underline(&this->IFont_iface, V_BOOL(&value));
//         }
//         else if (iRes == E_INVALIDARG)
//             iRes = S_OK;
//
//         VariantClear(&value);
//     }
//
//     if (iRes == S_OK) {
//         iRes = IPropertyBag_Read(pPropBag, sAttrItalic, &value, pErrorLog);
//         if (iRes == S_OK)
//         {
//             iRes = VariantChangeType(&value, &value, 0, VT_BOOL);
//             if (iRes == S_OK)
//                 iRes = IFont_put_Italic(&this->IFont_iface, V_BOOL(&value));
//         }
//         else if (iRes == E_INVALIDARG)
//             iRes = S_OK;
//
//         VariantClear(&value);
//     }
//
//     if (iRes == S_OK) {
//         iRes = IPropertyBag_Read(pPropBag, sAttrStrikethrough, &value, pErrorLog);
//         if (iRes == S_OK)
//         {
//             iRes = VariantChangeType(&value, &value, 0, VT_BOOL);
//             if (iRes == S_OK)
//                 IFont_put_Strikethrough(&this->IFont_iface, V_BOOL(&value));
//         }
//         else if (iRes == E_INVALIDARG)
//             iRes = S_OK;
//
//         VariantClear(&value);
//     }
//
//     if (FAILED(iRes))
//         WARN("-- 0x%08x\n", iRes);
//     return iRes;
// }
//
// static HRESULT WINAPI OLEFontImpl_IPersistPropertyBag_Save(
//    IPersistPropertyBag *iface, IPropertyBag* pPropBag, BOOL fClearDirty,
//    BOOL fSaveAllProperties
// ) {
//   FIXME("(%p,%p,%d,%d), stub!\n", iface, pPropBag, fClearDirty, fSaveAllProperties);
//   return E_FAIL;
// }
//
// static const IPersistPropertyBagVtbl OLEFontImpl_IPersistPropertyBag_VTable = 
// {
//   OLEFontImpl_IPersistPropertyBag_QueryInterface,
//   OLEFontImpl_IPersistPropertyBag_AddRef,
//   OLEFontImpl_IPersistPropertyBag_Release,
//
//   OLEFontImpl_IPersistPropertyBag_GetClassID,
//   OLEFontImpl_IPersistPropertyBag_InitNew,
//   OLEFontImpl_IPersistPropertyBag_Load,
//   OLEFontImpl_IPersistPropertyBag_Save
// };
//
// /************************************************************************
//  * OLEFontImpl implementation of IPersistStreamInit.
//  */
// static HRESULT WINAPI OLEFontImpl_IPersistStreamInit_QueryInterface(
//    IPersistStreamInit *iface, REFIID riid, LPVOID *ppvObj
// ) {
//   OLEFontImpl *this = impl_from_IPersistStreamInit(iface);
//   return IFont_QueryInterface(&this->IFont_iface,riid,ppvObj);
// }
//
// static ULONG WINAPI OLEFontImpl_IPersistStreamInit_AddRef(
//    IPersistStreamInit *iface
// ) {
//   OLEFontImpl *this = impl_from_IPersistStreamInit(iface);
//   return IFont_AddRef(&this->IFont_iface);
// }
//
// static ULONG WINAPI OLEFontImpl_IPersistStreamInit_Release(
//    IPersistStreamInit *iface
// ) {
//   OLEFontImpl *this = impl_from_IPersistStreamInit(iface);
//   return IFont_Release(&this->IFont_iface);
// }
//
// static HRESULT WINAPI OLEFontImpl_IPersistStreamInit_GetClassID(
//    IPersistStreamInit *iface, CLSID *classid
// ) {
//   FIXME("(%p,%p), stub!\n", iface, classid);
//   return E_FAIL;
// }
//
// static HRESULT WINAPI OLEFontImpl_IPersistStreamInit_IsDirty(
//    IPersistStreamInit *iface
// ) {
//   FIXME("(%p), stub!\n", iface);
//   return E_FAIL;
// }
//
// static HRESULT WINAPI OLEFontImpl_IPersistStreamInit_Load(
//    IPersistStreamInit *iface, LPSTREAM pStm
// ) {
//   FIXME("(%p,%p), stub!\n", iface, pStm);
//   return E_FAIL;
// }
//
// static HRESULT WINAPI OLEFontImpl_IPersistStreamInit_Save(
//    IPersistStreamInit *iface, LPSTREAM pStm, BOOL fClearDirty
// ) {
//   FIXME("(%p,%p,%d), stub!\n", iface, pStm, fClearDirty);
//   return E_FAIL;
// }
//
// static HRESULT WINAPI OLEFontImpl_IPersistStreamInit_GetSizeMax(
//    IPersistStreamInit *iface, ULARGE_INTEGER *pcbSize
// ) {
//   FIXME("(%p,%p), stub!\n", iface, pcbSize);
//   return E_FAIL;
// }
//
// static HRESULT WINAPI OLEFontImpl_IPersistStreamInit_InitNew(
//    IPersistStreamInit *iface
// ) {
//   FIXME("(%p), stub!\n", iface);
//   return S_OK;
// }
//
// static const IPersistStreamInitVtbl OLEFontImpl_IPersistStreamInit_VTable = 
// {
//   OLEFontImpl_IPersistStreamInit_QueryInterface,
//   OLEFontImpl_IPersistStreamInit_AddRef,
//   OLEFontImpl_IPersistStreamInit_Release,
//
//   OLEFontImpl_IPersistStreamInit_GetClassID,
//   OLEFontImpl_IPersistStreamInit_IsDirty,
//   OLEFontImpl_IPersistStreamInit_Load,
//   OLEFontImpl_IPersistStreamInit_Save,
//   OLEFontImpl_IPersistStreamInit_GetSizeMax,
//   OLEFontImpl_IPersistStreamInit_InitNew
// };
//
// /************************************************************************
//  * OLEFontImpl_Construct
//  *
//  * This method will construct a new instance of the OLEFontImpl
//  * class.
//  *
//  * The caller of this method must release the object when it's
//  * done with it.
//  */
// static OLEFontImpl* OLEFontImpl_Construct(const FONTDESC *fontDesc)
// {
//   OLEFontImpl* newObject;
//
//   newObject = HeapAlloc(GetProcessHeap(), 0, sizeof(OLEFontImpl));
//
//   if (newObject==0)
//     return newObject;
//
//   newObject->IFont_iface.lpVtbl = &OLEFontImpl_VTable;
//   newObject->IDispatch_iface.lpVtbl = &OLEFontImpl_IDispatch_VTable;
//   newObject->IPersistStream_iface.lpVtbl = &OLEFontImpl_IPersistStream_VTable;
//   newObject->IConnectionPointContainer_iface.lpVtbl = &OLEFontImpl_IConnectionPointContainer_VTable;
//   newObject->IPersistPropertyBag_iface.lpVtbl = &OLEFontImpl_IPersistPropertyBag_VTable;
//   newObject->IPersistStreamInit_iface.lpVtbl = &OLEFontImpl_IPersistStreamInit_VTable;
//
//   newObject->ref = 1;
//
//   newObject->description.cbSizeofstruct = sizeof(FONTDESC);
//   newObject->description.lpstrName      = strdupW(fontDesc->lpstrName);
//   newObject->description.cySize         = fontDesc->cySize;
//   newObject->description.sWeight        = fontDesc->sWeight;
//   newObject->description.sCharset       = fontDesc->sCharset;
//   newObject->description.fItalic        = fontDesc->fItalic;
//   newObject->description.fUnderline     = fontDesc->fUnderline;
//   newObject->description.fStrikethrough = fontDesc->fStrikethrough;
//
//   newObject->gdiFont  = 0;
//   newObject->dirty = TRUE;
//   newObject->cyLogical  = GetDeviceCaps(get_dc(), LOGPIXELSY);
//   newObject->cyHimetric = 2540L;
//   newObject->pPropertyNotifyCP = NULL;
//   newObject->pFontEventsCP = NULL;
//
//   CreateConnectionPoint((IUnknown*)&newObject->IFont_iface, &IID_IPropertyNotifySink, &newObject->pPropertyNotifyCP);
//   CreateConnectionPoint((IUnknown*)&newObject->IFont_iface, &IID_IFontEventsDisp, &newObject->pFontEventsCP);
//
//   if (!newObject->pPropertyNotifyCP || !newObject->pFontEventsCP)
//   {
//     OLEFontImpl_Destroy(newObject);
//     return NULL;
//   }
//
//   InterlockedIncrement(&ifont_cnt);
//
//   TRACE("returning %p\n", newObject);
//   return newObject;
// }
//
// /************************************************************************
//  * OLEFontImpl_Destroy
//  *
//  * This method is called by the Release method when the reference
//  * count goes down to 0. It will free all resources used by
//  * this object.
//  */
// static void OLEFontImpl_Destroy(OLEFontImpl* fontDesc)
// {
//   TRACE("(%p)\n", fontDesc);
//
//   HeapFree(GetProcessHeap(), 0, fontDesc->description.lpstrName);
//
//   if (fontDesc->pPropertyNotifyCP)
//       IConnectionPoint_Release(fontDesc->pPropertyNotifyCP);
//   if (fontDesc->pFontEventsCP)
//       IConnectionPoint_Release(fontDesc->pFontEventsCP);
//
//   HeapFree(GetProcessHeap(), 0, fontDesc);
// }
//
// /*******************************************************************************
//  * StdFont ClassFactory
//  */
// typedef struct
// {
//     /* IUnknown fields */
//     IClassFactory IClassFactory_iface;
//     LONG          ref;
// } IClassFactoryImpl;
//
// static inline IClassFactoryImpl *impl_from_IClassFactory(IClassFactory *iface)
// {
//         return CONTAINING_RECORD(iface, IClassFactoryImpl, IClassFactory_iface);
// }
//
// static HRESULT WINAPI SFCF_QueryInterface(IClassFactory *iface, REFIID riid, void **obj)
// {
//     IClassFactoryImpl *This = impl_from_IClassFactory(iface);
//
//     TRACE("(%p)->(%s, %p)\n", This, debugstr_guid(riid), obj);
//
//     *obj = NULL;
//
//     if (IsEqualIID(&IID_IClassFactory, riid) || IsEqualIID(&IID_IUnknown, riid))
//     {
//         *obj = iface;
//         IClassFactory_AddRef(iface);
//         return S_OK;
//     }
//
//     return E_NOINTERFACE;
// }
//
// static ULONG WINAPI
// SFCF_AddRef(LPCLASSFACTORY iface) {
// 	IClassFactoryImpl *This = impl_from_IClassFactory(iface);
// 	return InterlockedIncrement(&This->ref);
// }
//
// static ULONG WINAPI SFCF_Release(LPCLASSFACTORY iface) {
// 	IClassFactoryImpl *This = impl_from_IClassFactory(iface);
// 	/* static class, won't be  freed */
// 	return InterlockedDecrement(&This->ref);
// }
//
// static HRESULT WINAPI SFCF_CreateInstance(
// 	LPCLASSFACTORY iface,LPUNKNOWN pOuter,REFIID riid,LPVOID *ppobj
// ) {
// 	return OleCreateFontIndirect(NULL,riid,ppobj);
//
// }
//
// static HRESULT WINAPI SFCF_LockServer(LPCLASSFACTORY iface,BOOL dolock) {
// 	IClassFactoryImpl *This = impl_from_IClassFactory(iface);
// 	FIXME("(%p)->(%d),stub!\n",This,dolock);
// 	return S_OK;
// }
//
// static const IClassFactoryVtbl SFCF_Vtbl = {
// 	SFCF_QueryInterface,
// 	SFCF_AddRef,
// 	SFCF_Release,
// 	SFCF_CreateInstance,
// 	SFCF_LockServer
// };
// static IClassFactoryImpl STDFONT_CF = {{&SFCF_Vtbl}, 1 };
//
// void _get_STDFONT_CF(LPVOID *ppv) { *ppv = &STDFONT_CF; }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
