use crate::prelude::*;
use crate::msctf::categorymgr_cpp_h::CategoryMgr_Constructor;
use crate::msctf::displayattributemgr_cpp_h::DisplayAttributeMgr_Constructor;
use crate::msctf::inputprocessor_cpp_h::InputProcessorProfiles_Constructor;
use crate::msctf::langbarmgr_cpp_h::LangBarMgr_Constructor;
use crate::msctf::utils_cpp_h::ProcessAttach;
use crate::msctf::utils_cpp_h::ProcessDetach;
use crate::msctf::threadmgr_cpp_h::ThreadMgr_Constructor;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct CookieInternal {
    pub id: DWORD,
    pub magic: DWORD,
    pub data: LPVOID,
}
pub type PCookieInternal = *mut CookieInternal;

#[repr(C)]
pub struct ActivatedTextService {
    pub LanguageProfile: TF_LANGUAGEPROFILE,
    pub pITfTextInputProcessor: *mut ITfTextInputProcessor,
    pub pITfThreadMgrEx: *mut ITfThreadMgrEx,
    pub pITfKeyEventSink: *mut ITfKeyEventSink,
    pub tid: TfClientId,
}
pub type PActivatedTextService = *mut ActivatedTextService;

// wine/list.h's `struct list` is winelist-rust's, shared with every wine-derived crate.
pub type list = winelist_rust::list;

#[repr(C)]
pub struct AtsEntry {
    pub entry: list,
    pub ats: *mut ActivatedTextService,
}
pub type PAtsEntry = *mut AtsEntry;

pub type LPFNCONSTRUCTOR =
    Option<unsafe extern "C" fn(pUnkOuter: *mut IUnknown, ppvOut: *mut *mut IUnknown) -> HRESULT>;

pub static mut MSCTF_hinstance: HINSTANCE = unsafe { core::mem::zeroed() };
pub static mut cookies: *mut CookieInternal = core::ptr::null_mut();
pub static mut id_last: UINT = 0;
pub static mut array_size: UINT = 0;
// `static struct list AtsList = LIST_INIT(AtsList);`: an empty list points at itself. A static
// may take its own address, so this is the C initializer exactly (null links would make the
// first list walk follow a null pointer -- nothing re-initializes them at attach).
pub static mut AtsList: list = list {
    next: &raw mut AtsList,
    prev: &raw mut AtsList,
};
pub static mut activated: UINT = 0;
#[no_mangle]
pub static mut g_processId: TfClientId = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut g_globalCompartmentMgr: *mut ITfCompartmentMgr = core::ptr::null_mut();

#[repr(C)]
pub struct ClassesTableEntry {
    pub clsid: *const CLSID,
    pub constructor: LPFNCONSTRUCTOR,
}
unsafe impl Sync for ClassesTableEntry {}

pub static ClassesTable: [ClassesTableEntry; 6] = [
    ClassesTableEntry {
        clsid: &raw const CLSID_TF_ThreadMgr,
        constructor: Some(ThreadMgr_Constructor),
    },
    ClassesTableEntry {
        clsid: &raw const CLSID_TF_InputProcessorProfiles,
        constructor: Some(InputProcessorProfiles_Constructor),
    },
    ClassesTableEntry {
        clsid: &raw const CLSID_TF_CategoryMgr,
        constructor: Some(CategoryMgr_Constructor),
    },
    ClassesTableEntry {
        clsid: &raw const CLSID_TF_LangBarMgr,
        constructor: Some(LangBarMgr_Constructor),
    },
    ClassesTableEntry {
        clsid: &raw const CLSID_TF_DisplayAttributeMgr,
        constructor: Some(DisplayAttributeMgr_Constructor),
    },
    ClassesTableEntry {
        clsid: core::ptr::null(),
        constructor: None,
    },
];

const IID_IClassFactory: IID = GUID { Data1: 0x00000001, Data2: 0x0000, Data3: 0x0000, Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] }; // IID_IClassFactory
const IID_IUnknown: IID = GUID { Data1: 0x00000000, Data2: 0x0000, Data3: 0x0000, Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] }; // IID_IUnknown










unsafe extern "C" fn ClassFactory_Constructor(ctor: LPFNCONSTRUCTOR, ppvOut: *mut LPVOID) -> HRESULT {
    let This: *mut CClassFactory = CClassFactory__new_in_0(cpp_alloc_CicNoThrow::<CClassFactory>(), ctor);
    *ppvOut = core::ptr::addr_of_mut!((*This).__base0) as LPVOID;

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn generate_Cookie(magic: DWORD, data: LPVOID) -> DWORD {
    let mut i: UINT;

    /* try to reuse IDs if possible */
    i = 0;
    while i < id_last {
        if (*cookies.add(i as usize)).id == 0 {
            break;
        }
        i += 1;
    }

    if i == array_size {
        if array_size == 0 {
            cookies = cicMemAllocClear((10 * core::mem::size_of::<CookieInternal>()) as _) as *mut CookieInternal;
            if cookies.is_null() {
                return 0;
            }
            array_size = 10;
        } else {
            let new_cookies: *mut CookieInternal = cicMemReCalloc(
                cookies as LPVOID,
                ((array_size * 2) as usize) as _,
                (core::mem::size_of::<CookieInternal>()) as _,
            ) as *mut CookieInternal;
            if new_cookies.is_null() {
                return 0;
            }
            cookies = new_cookies;
            array_size *= 2;
        }
    }

    (*cookies.add(i as usize)).id = i + 1; /* a return of 0 is used for failure */
    (*cookies.add(i as usize)).magic = magic;
    (*cookies.add(i as usize)).data = data;

    if i == id_last {
        id_last += 1;
    }

    return (*cookies.add(i as usize)).id;
}
#[no_mangle]
pub unsafe extern "C" fn get_Cookie_magic(id: DWORD) -> DWORD {
    let index: UINT = id.wrapping_sub(1);

    if index >= id_last {
        return 0;
    }

    if (*cookies.add(index as usize)).id == 0 {
        return 0;
    }

    (*cookies.add(index as usize)).magic
}
#[no_mangle]
pub unsafe extern "C" fn get_Cookie_data(id: DWORD) -> LPVOID {
    let index: UINT = id.wrapping_sub(1);

    if index >= id_last {
        return core::ptr::null_mut();
    }

    if (*cookies.add(index as usize)).id == 0 {
        return core::ptr::null_mut();
    }

    (*cookies.add(index as usize)).data
}
#[no_mangle]
pub unsafe extern "C" fn remove_Cookie(id: DWORD) -> LPVOID {
    let mut index: UINT = id.wrapping_sub(1);

    if index >= id_last {
        return core::ptr::null_mut();
    }

    if (*cookies.add(index as usize)).id == 0 {
        return core::ptr::null_mut();
    }

    (*cookies.add(index as usize)).id = 0;
    return (*cookies.add(index as usize)).data;
}
#[no_mangle]
pub unsafe extern "C" fn enumerate_Cookie(magic: DWORD, index: *mut DWORD) -> DWORD {
    let mut i: UINT;
    i = *index;
    while i < id_last {
        if (*cookies.add(i as usize)).id != 0 && (*cookies.add(i as usize)).magic == magic {
            *index = i + 1;
            return (*cookies.add(i as usize)).id;
        }
        i += 1;
    }
    return 0x0;
}
#[no_mangle]
pub unsafe extern "C" fn advise_sink(sink_list: *mut list, riid: REFIID, cookie_magic: DWORD, unk: *mut IUnknown, cookie: *mut DWORD) -> HRESULT {
    let sink: *mut Sink = cicMemAlloc(core::mem::size_of::<Sink>()) as *mut Sink;
    if sink.is_null() {
        return E_OUTOFMEMORY;
    }

    let hr: HRESULT = (*unk).QueryInterface(riid, core::ptr::addr_of_mut!((*sink).interfaces.pIUnknown) as *mut LPVOID);
    if ((hr) < 0)
    {
        cicMemFree(sink as LPVOID);
        return CONNECT_E_CANNOTCONNECT;
    }

    list_add_head((sink_list as LPVOID).cast(), (core::ptr::addr_of_mut!((*sink).entry) as LPVOID).cast());
    *cookie = generate_Cookie(cookie_magic, sink as LPVOID);

    return S_OK;
}

unsafe extern "C" fn free_sink(sink: *mut Sink) {
    list_remove(core::ptr::addr_of_mut!((*sink).entry));
    (*(*sink).interfaces.pIUnknown).Release();
    cicMemFree(sink as LPVOID);
}
#[no_mangle]
pub unsafe extern "C" fn unadvise_sink(cookie: DWORD) -> HRESULT {
    let sink: *mut Sink = remove_Cookie(cookie) as *mut Sink;
    if sink.is_null() {
        return CONNECT_E_NOCONNECTION;
    }

    free_sink(sink);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn free_sinks(sink_list: *mut list) {
    while list_empty(sink_list) == 0 {
        let sink: *mut Sink = LIST_ENTRY!((*sink_list).next, Sink, entry);
        free_sink(sink);
    }
}

unsafe extern "C" fn activate_given_ts(actsvr: *mut ActivatedTextService, tm: *mut ITfThreadMgrEx) -> HRESULT {
    let mut hr: HRESULT;

    /* Already Active? */
    if !(*actsvr).pITfTextInputProcessor.is_null() {
        return S_OK;
    }

    hr = CoCreateInstance(core::ptr::addr_of!((*actsvr).LanguageProfile.clsid), core::ptr::null_mut(), CLSCTX_INPROC_SERVER,
                          core::ptr::addr_of!(IID_ITfTextInputProcessor), core::ptr::addr_of_mut!((*actsvr).pITfTextInputProcessor) as *mut *mut core::ffi::c_void);
    if hr < 0 {
        return hr;
    }

    hr = (((*(*(*actsvr).pITfTextInputProcessor).lpVtbl).Activate).unwrap_unchecked())((*actsvr).pITfTextInputProcessor, tm as *mut ITfThreadMgr, (*actsvr).tid);
    if hr < 0 {
        (((*(*(*actsvr).pITfTextInputProcessor).lpVtbl).Release).unwrap_unchecked())((*actsvr).pITfTextInputProcessor);
        (*actsvr).pITfTextInputProcessor = core::ptr::null_mut();
        return hr;
    }

    (*actsvr).pITfThreadMgrEx = tm;
    (((*(*tm).lpVtbl).AddRef).unwrap_unchecked())(tm);
    return hr;
}

unsafe extern "C" fn deactivate_given_ts(actsvr: *mut ActivatedTextService) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if !(*actsvr).pITfTextInputProcessor.is_null() {
        hr = (*(*actsvr).pITfTextInputProcessor).Deactivate();
        (*(*actsvr).pITfTextInputProcessor).Release();
        (*(*actsvr).pITfThreadMgrEx).Release();
        (*actsvr).pITfTextInputProcessor = core::ptr::null_mut();
        (*actsvr).pITfThreadMgrEx = core::ptr::null_mut();
    }

    hr
}

unsafe extern "C" fn deactivate_remove_conflicting_ts(catid: REFCLSID) {
    let mut ats: *mut AtsEntry;
    let mut cursor2: *mut AtsEntry;

    // LIST_FOR_EACH_ENTRY_SAFE(ats, cursor2, &AtsList, AtsEntry, entry) expanded inline
    let list: *mut list = core::ptr::addr_of_mut!(AtsList);
    ats = ((*list).next as *mut u8).sub(core::mem::offset_of!(AtsEntry, entry)) as *mut AtsEntry;
    cursor2 = ((*ats).entry.next as *mut u8).sub(core::mem::offset_of!(AtsEntry, entry)) as *mut AtsEntry;
    while (core::ptr::addr_of_mut!((*ats).entry)) != list {
        {
            if catid == &((*(*ats).ats).LanguageProfile.catid) {
                deactivate_given_ts((*ats).ats);
                list_remove((core::ptr::addr_of_mut!((*ats).entry)).cast());
                cicMemFree(((*ats).ats).cast());
                cicMemFree((ats).cast());
                /* we are guaranteeing there is only 1 */
                break;
            }
        }
        ats = cursor2;
        cursor2 = ((*cursor2).entry.next as *mut u8).sub(core::mem::offset_of!(AtsEntry, entry)) as *mut AtsEntry;
    }
}
#[no_mangle]
pub unsafe extern "C" fn add_active_textservice(lp: *mut TF_LANGUAGEPROFILE) -> HRESULT {
    let mut actsvr: *mut ActivatedTextService;
    let mut catmgr: *mut ITfCategoryMgr = core::ptr::null_mut();
    let mut entry: *mut AtsEntry;
    let tm: *mut ITfThreadMgrEx = TlsGetValue(g_dwTLSIndex) as *mut ITfThreadMgrEx;
    let mut clientid: *mut ITfClientId = core::ptr::null_mut();

    if tm.is_null() {
        return E_UNEXPECTED;
    }

    actsvr = cicMemAlloc(core::mem::size_of::<ActivatedTextService>()) as *mut ActivatedTextService;
    if actsvr.is_null() {
        return E_OUTOFMEMORY;
    }

    (*tm).QueryInterface(
        core::ptr::addr_of!(IID_ITfClientId),
        core::ptr::addr_of_mut!(clientid) as *mut *mut core::ffi::c_void,
    );
    (*clientid).GetClientId(
        core::ptr::addr_of!((*lp).clsid),
        core::ptr::addr_of_mut!((*actsvr).tid),
    );
    (*clientid).Release();

    if (*actsvr).tid == 0 {
        cicMemFree(actsvr as *mut core::ffi::c_void);
        return E_OUTOFMEMORY;
    }

    (*actsvr).pITfTextInputProcessor = core::ptr::null_mut();
    (*actsvr).LanguageProfile = *lp;
    (*actsvr).pITfKeyEventSink = core::ptr::null_mut();

    /* get TIP category */
    if CategoryMgr_Constructor(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(catmgr) as *mut *mut IUnknown,
    ) >= 0
    {
        static mut LIST: [*const GUID; 3] = [
            core::ptr::addr_of!(GUID_TFCAT_TIP_SPEECH),
            core::ptr::addr_of!(GUID_TFCAT_TIP_KEYBOARD),
            core::ptr::addr_of!(GUID_TFCAT_TIP_HANDWRITING),
        ];

        (*catmgr).FindClosestCategory(
            core::ptr::addr_of!((*actsvr).LanguageProfile.clsid),
            core::ptr::addr_of_mut!((*actsvr).LanguageProfile.catid),
            LIST.as_ptr() as *mut *const GUID,
            3,
        );
        (*catmgr).Release();
    } else {
        (*actsvr).LanguageProfile.catid = GUID_NULL;
    }

    if !IsEqualGUID!(&((*actsvr).LanguageProfile.catid), &(GUID_NULL)) {
        deactivate_remove_conflicting_ts(core::ptr::addr_of!((*actsvr).LanguageProfile.catid));
    }

    if activated > 0 {
        activate_given_ts(actsvr, tm);
    }

    entry = cicMemAlloc(core::mem::size_of::<AtsEntry>()) as *mut AtsEntry;
    if entry.is_null() {
        cicMemFree(actsvr as *mut core::ffi::c_void);
        return E_OUTOFMEMORY;
    }

    (*entry).ats = actsvr;
    list_add_head(
        (core::ptr::addr_of_mut!(AtsList)).cast(),
        (core::ptr::addr_of_mut!((*entry).entry)).cast(),
    );

    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn get_active_textservice(rclsid: REFCLSID, profile: *mut TF_LANGUAGEPROFILE) -> BOOL {
    let mut ats: *mut AtsEntry;

    LIST_FOR_EACH_ENTRY!(ats, core::ptr::addr_of_mut!(AtsList), AtsEntry, entry, {
        if rclsid == core::ptr::addr_of!((*(*ats).ats).LanguageProfile.clsid) {
            if !profile.is_null() {
                *profile = (*(*ats).ats).LanguageProfile;
            }
            return (TRUE) as BOOL;
        }
    });
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn activate_textservices(tm: *mut ITfThreadMgrEx) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut ats: *mut AtsEntry;

    activated += 1;
    if activated > 1 {
        return S_OK;
    }

    ats = ((*core::ptr::addr_of_mut!(AtsList)).next as *mut u8)
        .sub(core::mem::offset_of!(AtsEntry, entry)) as *mut AtsEntry;
    while core::ptr::addr_of_mut!((*ats).entry) != core::ptr::addr_of_mut!(AtsList) {
        hr = activate_given_ts((*ats).ats, tm);
        if hr < 0 {
            FIXME!("Failed to activate text service\n");
        }
        ats = ((*ats).entry.next as *mut u8)
            .sub(core::mem::offset_of!(AtsEntry, entry)) as *mut AtsEntry;
    }
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn deactivate_textservices() -> HRESULT {
    let mut ats: *mut AtsEntry;

    if activated > 0 {
        activated -= 1;
    }

    if activated == 0 {
        let head: *mut list = core::ptr::addr_of_mut!(AtsList);
        let mut cur: *mut list = (*head).next;
        while cur != head {
            ats = (cur as *mut u8).sub(core::mem::offset_of!(AtsEntry, entry)) as *mut AtsEntry;
            deactivate_given_ts((*ats).ats);
            cur = (*cur).next;
        }
    }
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn get_textservice_clsid(tid: TfClientId) -> CLSID {
    let mut ats: *mut AtsEntry = core::ptr::null_mut();

    LIST_FOR_EACH_ENTRY!(ats, core::ptr::addr_of_mut!(AtsList), AtsEntry, entry, {
        if (*(*ats).ats).tid == tid {
            return (*(*ats).ats).LanguageProfile.clsid;
        }
    });
    return GUID_NULL;
}
#[no_mangle]
pub unsafe extern "C" fn get_textservice_sink(tid: TfClientId, iid: REFCLSID, sink: *mut *mut IUnknown) -> HRESULT {
    let mut ats: *mut AtsEntry;

    if iid != core::ptr::addr_of!(IID_ITfKeyEventSink) as REFCLSID {
        return E_NOINTERFACE;
    }

    LIST_FOR_EACH_ENTRY!(ats, core::ptr::addr_of_mut!(AtsList), AtsEntry, entry, {
        if (*(*ats).ats).tid == tid {
            *sink = (*(*ats).ats).pITfKeyEventSink as *mut IUnknown;
            return S_OK;
        }
    });

    return E_FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn set_textservice_sink(tid: TfClientId, iid: REFCLSID, sink: *mut IUnknown) -> HRESULT {
    let mut ats: *mut AtsEntry;

    if iid != addr_of!(IID_ITfKeyEventSink) {
        return E_NOINTERFACE;
    }

    LIST_FOR_EACH_ENTRY!(ats, addr_of_mut!(AtsList), AtsEntry, entry, {
        if (*(*ats).ats).tid == tid {
            (*(*ats).ats).pITfKeyEventSink = sink as *mut ITfKeyEventSink;
            return S_OK;
        }
    });

    return E_FAIL;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinst: HINSTANCE, fdwReason: u32, fImpLoad: LPVOID) -> i32 {
    __cxx_global_init();

    match fdwReason {
        DLL_PROCESS_ATTACH => {
            MSCTF_hinstance = hinst;
            return ProcessAttach(hinst);
        }
        DLL_PROCESS_DETACH => {
            ProcessDetach(hinst);
        }
        _ => {}
    }
    return (TRUE) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(clsid: *const GUID, iid: *const IID, ppvOut: *mut LPVOID) -> HRESULT {
    let mut i: i32;

    *ppvOut = core::ptr::null_mut();
    if !IsEqualGUID!(iid, &IID_IUnknown) && !IsEqualGUID!(iid, &IID_IClassFactory) {
        return E_NOINTERFACE;
    }

    i = 0;
    while !ClassesTable[i as usize].clsid.is_null() {
        if IsEqualGUID!(&(*ClassesTable[i as usize].clsid), &(*clsid)) {
            // LPFNCONSTRUCTOR is nullable (Option), as the C function pointer is.
            return ClassFactory_Constructor(ClassesTable[i as usize].constructor, ppvOut);
        }
        i += 1;
    }

    return CLASS_E_CLASSNOTAVAILABLE;
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    return __wine_register_resources(MSCTF_hinstance);
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    return __wine_unregister_resources(MSCTF_hinstance);
}

#[no_mangle]
pub unsafe extern "system" fn TF_CreateThreadMgr(pptim: *mut *mut ITfThreadMgr) -> HRESULT {
    return ThreadMgr_Constructor(core::ptr::null_mut(), pptim as *mut *mut IUnknown);
}

#[no_mangle]
pub unsafe extern "system" fn TF_GetThreadMgr(pptim: *mut *mut ITfThreadMgr) -> HRESULT {
    *pptim = TlsGetValue(g_dwTLSIndex) as *mut ITfThreadMgr;

    if !(*pptim).is_null() {
        (**pptim).AddRef();
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn SetInputScope(hwnd: HWND, inputscope: InputScope) -> HRESULT {
    // FIXME("STUB: %p %i\n", hwnd, inputscope);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SetInputScopes(hwnd: HWND, pInputScopes: *const InputScope, cInputScopes: UINT, ppszPhraseList: *mut *mut WCHAR, cPhrases: UINT, pszRegExp: *mut WCHAR, pszSRGS: *mut WCHAR) -> HRESULT {
    let mut i: UINT;
    i = 0;
    while i < cInputScopes {
        TRACE!("\tScope[%u] = %i\n", i, *pInputScopes.add(i as usize));
        i += 1;
    }
    i = 0;
    while i < cPhrases {
        TRACE!("\tPhrase[%u] = %s\n", i, debugstr_w(*ppszPhraseList.add(i as usize)));
        i += 1;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn TF_CreateInputProcessorProfiles(ppipr: *mut *mut ITfInputProcessorProfiles) -> HRESULT {
    return InputProcessorProfiles_Constructor(core::ptr::null_mut(), ppipr as *mut *mut IUnknown);
}

#[no_mangle]
pub unsafe extern "system" fn TF_InvalidAssemblyListCacheIfExist() -> HRESULT {
    // FIXME("Stub\n");
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn TF_CreateLangBarMgr(pppbm: *mut *mut ITfLangBarMgr) -> HRESULT {
    return LangBarMgr_Constructor(core::ptr::null_mut(), pppbm as *mut *mut IUnknown);
}

#[no_mangle]
pub unsafe extern "system" fn TF_CreateLangBarItemMgr(pplbim: *mut *mut ITfLangBarItemMgr) -> HRESULT {
    *pplbim = core::ptr::null_mut();

    return E_NOTIMPL;
}
#[no_mangle]
pub unsafe extern "C" fn __cxx_global_init() {
    (*core::ptr::addr_of_mut!(g_IconList)).__ctor_0();
}

impl CClassFactory {
    pub unsafe fn __ctor_0(&mut self, ctor: LPFNCONSTRUCTOR) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_cRefs = 1;
        (*this_).m_ctor = ctor;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        // TRACE("Destroying class factory %p\n", this_);
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        *ppvObj = core::ptr::null_mut();
        if IsEqualGUID!(&(*riid), &(IID_IClassFactory)) || IsEqualGUID!(&(*riid), &(IID_IUnknown)) {
            (*this_).AddRef();
            *ppvObj = core::ptr::addr_of_mut!((*this_).__base0) as *mut core::ffi::c_void;
            return S_OK;
        }
    
        return E_NOINTERFACE;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        return InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        let ret: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_cRefs)) as ULONG;
        if ret == 0 {
            CClassFactory__delete(this_);
        }
        return ret;
    }
    pub unsafe fn CreateInstance(&mut self, pUnkOuter: *mut IUnknown, riid: &IID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        let mut obj: *mut IUnknown = core::ptr::null_mut();
        let mut ret: HRESULT = (((*this_).m_ctor).unwrap_unchecked())(pUnkOuter, core::ptr::addr_of_mut!(obj));
        if ret < 0 {
            return ret;
        }
        ret = (*obj).QueryInterface(riid, ppvObject);
        (*obj).Release();
        return ret;
    }
    pub unsafe fn LockServer(&mut self, fLock: BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // TRACE("(%p)->(%x)\n", this_, fLock);
        S_OK
    }
}
