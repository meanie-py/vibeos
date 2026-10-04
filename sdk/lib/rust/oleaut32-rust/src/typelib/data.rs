//! data.rs — the TLB in-memory data model (typelib.c lines ~1057–2028):
//! TLBGuid/TLBCustData/TLBImpLib/TLBString/ITypeLibImpl/TLBRefType/TLBParDesc/
//! TLBFuncDesc/TLBVarDesc/TLBImplType/ITypeInfoImpl/TLBContext + the TLB_*
//! helpers. Lane A2 continuation; faithful 1:1 (see MANIFEST for deviations —
//! notably: the dump_* TRACE-only debug helpers are compiled out like all
//! TRACE paths in this lane).

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_variables, unused_mut, unused_unsafe, unused_imports)]

use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{null, null_mut};

use super::super::olesys::*;
use super::sys::*;

// local debug-print macros (same convention as typelib.rs). TRACE!/WARN!
// compile out; ERR!/FIXME! -> OutputDebugStringA.
macro_rules! TRACE {
    ($($t:tt)*) => {{}};
}
macro_rules! WARN {
    ($($t:tt)*) => {{}};
}
// NOTE: paths in macro bodies resolve at the EXPANSION site — all users are
// `typelib/*.rs` siblings (same module depth), so the relative path holds.
macro_rules! ERR {
    ($fmt:literal $(, $e:expr)* $(,)?) => {
        super::super::olesys::dbg_print(format_args!(concat!("err:ole:typelib ", $fmt, "\n") $(, $e)*))
    };
}
macro_rules! FIXME {
    ($fmt:literal $(, $e:expr)* $(,)?) => {
        super::super::olesys::dbg_print(format_args!(concat!("fixme:ole:typelib ", $fmt, "\n") $(, $e)*))
    };
}
pub(crate) use {ERR, FIXME, TRACE, WARN};

// ===========================================================================
// ITypeLib implementation data
// ===========================================================================

#[repr(C)]
pub struct TLBGuid {
    pub guid: GUID,
    pub hreftype: INT,
    pub offset: UINT,
    pub entry: list,
}

#[repr(C)]
pub struct TLBCustData {
    pub guid: *mut TLBGuid,
    pub data: VARIANT,
    pub entry: list,
}

/// data structure for import typelibs
#[repr(C)]
pub struct TLBImpLib {
    /// offset in the file (MSFT), offset in nametable (SLTG); just used to
    /// identify library while reading data from file
    pub offset: i32,
    pub guid: *mut TLBGuid,     /* libid */
    pub name: BSTR,             /* name */
    pub lcid: LCID,             /* lcid of imported typelib */
    pub wVersionMajor: WORD,    /* major version number */
    pub wVersionMinor: WORD,    /* minor version number */
    /// pointer to loaded typelib, or NULL if not yet loaded
    pub pImpTypeLib: *mut ITypeLibImpl,
    pub entry: list,
}

#[repr(C)]
pub struct TLBString {
    pub str_: BSTR,
    pub offset: UINT,
    pub entry: list,
}

/// internal ITypeLib data
#[repr(C)]
pub struct ITypeLibImpl {
    pub ITypeLib2_iface: ITypeLib2,
    pub ITypeComp_iface: ITypeComp,
    pub ICreateTypeLib2_iface: ICreateTypeLib2,
    pub ref_: LONG,
    pub guid: *mut TLBGuid,
    pub lcid: LCID,
    pub syskind: SYSKIND,
    pub ptr_size: i32,
    pub ver_major: WORD,
    pub ver_minor: WORD,
    pub libflags: WORD,
    pub set_lcid: LCID,

    /* strings can be stored in tlb as multibyte strings BUT they are *always*
     * exported to the application as a UNICODE string. */
    pub string_list: list,
    pub name_list: list,
    pub guid_list: list,

    pub Name: *const TLBString,
    pub DocString: *const TLBString,
    pub HelpFile: *const TLBString,
    pub HelpStringDll: *const TLBString,
    pub dwHelpContext: DWORD,
    pub TypeInfoCount: i32, /* nr of typeinfo's in library */
    pub typeinfos: *mut *mut ITypeInfoImpl,
    pub custdata_list: list,
    pub implib_list: list,
    pub ctTypeDesc: i32,        /* number of items in type desc array */
    /// array of TypeDescriptions found in the library. Only used while
    /// reading MSFT typelibs
    pub pTypeDesc: *mut TYPEDESC,
    pub ref_list: list,         /* list of ref types in this typelib */
    pub dispatch_href: HREFTYPE, /* reference to IDispatch, -1 if unused */

    /* typelibs are cached, keyed by path and index, so store the linked list
     * info within them */
    pub entry: list,
    pub path: *mut WCHAR,
    pub index: INT,
}

#[inline]
pub unsafe fn impl_from_ITypeLib2(iface: *mut ITypeLib2) -> *mut ITypeLibImpl {
    containing_record!(iface, ITypeLibImpl, ITypeLib2_iface)
}
#[inline]
pub unsafe fn impl_from_ITypeLib(iface: *mut ITypeLib) -> *mut ITypeLibImpl {
    impl_from_ITypeLib2(iface as *mut ITypeLib2)
}
#[inline]
pub unsafe fn tlb_impl_from_ITypeComp(iface: *mut ITypeComp) -> *mut ITypeLibImpl {
    containing_record!(iface, ITypeLibImpl, ITypeComp_iface)
}
#[inline]
pub unsafe fn impl_from_ICreateTypeLib2(iface: *mut ICreateTypeLib2) -> *mut ITypeLibImpl {
    containing_record!(iface, ITypeLibImpl, ICreateTypeLib2_iface)
}

// ===========================================================================
// ITypeInfo implementation data
// ===========================================================================

/// data for referenced types
#[repr(C)]
pub struct TLBRefType {
    /// Type index for internal ref or for external ref if the format is SLTG.
    /// -2 (TLB_REF_USE_GUID) indicates to use guid.
    pub index: INT,
    pub tkind: TYPEKIND,
    pub guid: *mut TLBGuid, /* guid of the referenced type if index == TLB_REF_USE_GUID */
    pub reference: HREFTYPE, /* the href of this ref */
    /// If ref is external, ptr to library data; TLB_REF_INTERNAL for internal
    /// refs; TLB_REF_NOT_FOUND for broken refs.
    pub pImpTLInfo: *mut TLBImpLib,
    pub entry: list,
}

pub const TLB_REF_USE_GUID: INT = -2;
pub const TLB_REF_INTERNAL: *mut TLBImpLib = -2isize as usize as *mut TLBImpLib;
pub const TLB_REF_NOT_FOUND: *mut TLBImpLib = -1isize as usize as *mut TLBImpLib;

/// internal Parameter data
#[repr(C)]
pub struct TLBParDesc {
    pub Name: *const TLBString,
    pub custdata_list: list,
}

/// internal Function data
#[repr(C)]
pub struct TLBFuncDesc {
    pub funcdesc: FUNCDESC, /* lots of info on the function and its attributes. */
    pub Name: *const TLBString, /* the name of this function */
    pub pParamDesc: *mut TLBParDesc, /* array with param names and custom data */
    pub helpcontext: i32,
    pub HelpStringContext: i32,
    pub HelpString: *const TLBString,
    /// if IS_INTRESOURCE true, it's numeric; if -1 it isn't present
    pub Entry: *const TLBString,
    pub custdata_list: list,
}

/// internal Variable data
#[repr(C)]
pub struct TLBVarDesc {
    pub vardesc: VARDESC, /* lots of info on the variable and its attributes. */
    pub vardesc_create: *mut VARDESC, /* additional data needed for storing VARDESC */
    pub Name: *const TLBString, /* the name of this variable */
    pub HelpContext: i32,
    pub HelpStringContext: i32,
    pub HelpString: *const TLBString,
    pub custdata_list: list,
}

/// internal implemented interface data
#[repr(C)]
pub struct TLBImplType {
    pub hRef: HREFTYPE, /* hRef of interface */
    pub implflags: i32, /* IMPLFLAG_*s */
    pub custdata_list: list,
}

/// internal TypeInfo data
#[repr(C)]
pub struct ITypeInfoImpl {
    pub ITypeInfo2_iface: ITypeInfo2,
    pub ITypeComp_iface: ITypeComp,
    pub ICreateTypeInfo2_iface: ICreateTypeInfo2,
    pub ref_: LONG,
    pub not_attached_to_typelib: BOOL,
    pub needs_layout: BOOL,

    pub guid: *mut TLBGuid,
    pub typeattr: TYPEATTR,
    pub tdescAlias: *mut TYPEDESC,

    pub pTypeLib: *mut ITypeLibImpl, /* back pointer to typelib */
    pub index: i32,                  /* index in this typelib */
    pub hreftype: HREFTYPE,          /* hreftype for app object binding */

    /* type libs seem to store the doc strings in ascii
     * so why should we do it in unicode? */
    pub Name: *const TLBString,
    pub DocString: *const TLBString,
    pub DllName: *const TLBString,
    pub Schema: *const TLBString,
    pub dwHelpContext: DWORD,
    pub dwHelpStringContext: DWORD,

    /* functions */
    pub funcdescs: *mut TLBFuncDesc,
    /* variables */
    pub vardescs: *mut TLBVarDesc,
    /* Implemented Interfaces */
    pub impltypes: *mut TLBImplType,

    pub pcustdata_list: *mut list,
    pub custdata_list: list,
}

#[inline]
pub unsafe fn info_impl_from_ITypeComp(iface: *mut ITypeComp) -> *mut ITypeInfoImpl {
    containing_record!(iface, ITypeInfoImpl, ITypeComp_iface)
}
#[inline]
pub unsafe fn impl_from_ITypeInfo2(iface: *mut ITypeInfo2) -> *mut ITypeInfoImpl {
    containing_record!(iface, ITypeInfoImpl, ITypeInfo2_iface)
}
#[inline]
pub unsafe fn impl_from_ITypeInfo(iface: *mut ITypeInfo) -> *mut ITypeInfoImpl {
    impl_from_ITypeInfo2(iface as *mut ITypeInfo2)
}
#[inline]
pub unsafe fn info_impl_from_ICreateTypeInfo2(iface: *mut ICreateTypeInfo2) -> *mut ITypeInfoImpl {
    containing_record!(iface, ITypeInfoImpl, ICreateTypeInfo2_iface)
}

/// MSFT-loader read context.
#[repr(C)]
pub struct TLBContext {
    pub oStart: u32,  /* start of TLB in file */
    pub pos: u32,     /* current pos */
    pub length: u32,  /* total length */
    pub mapping: *mut c_void, /* memory mapping */
    pub pTblDir: *mut super::MSFT_SegDir,
    pub pLibInfo: *mut ITypeLibImpl,
}

// ===========================================================================
// small inline helpers
// ===========================================================================

#[inline]
pub unsafe fn TLB_get_bstr(str_: *const TLBString) -> BSTR {
    if !str_.is_null() {
        (*str_).str_
    } else {
        null_mut()
    }
}

#[inline]
pub unsafe fn TLB_str_memcmp(left: *mut c_void, str_: *const TLBString, len: DWORD) -> i32 {
    if str_.is_null() {
        return 1;
    }
    memcmp(left as *const c_void, (*str_).str_ as *const c_void, len as usize)
}

#[inline]
pub unsafe fn TLB_get_guidref(guid: *const TLBGuid) -> *const GUID {
    if !guid.is_null() {
        &raw const (*guid).guid
    } else {
        null()
    }
}

#[inline]
pub unsafe fn TLB_get_guid_null(guid: *const TLBGuid) -> *const GUID {
    if !guid.is_null() {
        &raw const (*guid).guid
    } else {
        &raw const GUID_NULL
    }
}

pub fn get_ptr_size(syskind: SYSKIND) -> i32 {
    match syskind {
        SYS_WIN64 => return 8,
        SYS_WIN32 | SYS_MAC | SYS_WIN16 => return 4,
        _ => {}
    }
    // WARN("Unhandled syskind: 0x%x\n", syskind) — compiled out
    4
}

// ---------------------------------------------------------------------------
// debug dumps (dump_TypeDesc/ELEMDESC/FUNCDESC/TLBFuncDesc/TLBVarDesc/
// TLBImpLib/TLBRefType/TLBImplType/DispParms/TypeInfo/VARDESC, typekind_desc):
// only reachable under TRACE_ON(...), which this lane compiles out (MANIFEST
// "debug channel" deviation) — intentionally not carried.
// ---------------------------------------------------------------------------

/// `std_typedesc[vt]` — flat TYPEDESCs for the simple VTs (never written).
/// VT_LPWSTR is the largest type that may appear in a type description.
pub static mut std_typedesc: [TYPEDESC; VT_LPWSTR as usize + 1] = {
    let mut a = [TYPEDESC { u: TYPEDESC_U { hreftype: 0 }, vt: 0 }; VT_LPWSTR as usize + 1];
    let mut i = 0;
    while i < a.len() {
        a[i].vt = i as VARTYPE; /* index 15 stays 15, "unused in VARENUM", like the C */
        i += 1;
    }
    a
};

pub unsafe fn TLB_abort() {
    DebugBreak();
}

/// returns the size required for a deep copy of a typedesc into a flat buffer
pub unsafe fn TLB_SizeTypeDesc(tdesc: *const TYPEDESC, alloc_initial_space: BOOL) -> SIZE_T {
    let mut size: SIZE_T = 0;

    if alloc_initial_space != FALSE {
        size += size_of::<TYPEDESC>();
    }

    match (*tdesc).vt {
        VT_PTR | VT_SAFEARRAY => {
            size += TLB_SizeTypeDesc((*tdesc).u.lptdesc, TRUE);
        }
        VT_CARRAY => {
            // FIELD_OFFSET(ARRAYDESC, rgbounds[cDims])
            size += core::mem::offset_of!(ARRAYDESC, rgbounds)
                + size_of::<SAFEARRAYBOUND>() * (*(*tdesc).u.lpadesc).cDims as usize;
            size += TLB_SizeTypeDesc(&raw const (*(*tdesc).u.lpadesc).tdescElem, FALSE);
        }
        _ => {}
    }
    size
}

/// deep copy a typedesc into a flat buffer
pub unsafe fn TLB_CopyTypeDesc(
    dest: *mut TYPEDESC,
    src: *const TYPEDESC,
    buffer: *mut c_void,
) -> *mut c_void {
    let mut dest = dest;
    let mut buffer = buffer;

    if dest.is_null() {
        dest = buffer as *mut TYPEDESC;
        buffer = (buffer as *mut u8).add(size_of::<TYPEDESC>()) as *mut c_void;
    }

    *dest = *src;

    match (*src).vt {
        VT_PTR | VT_SAFEARRAY => {
            (*dest).u.lptdesc = buffer as *mut TYPEDESC;
            buffer = TLB_CopyTypeDesc(null_mut(), (*src).u.lptdesc, buffer);
        }
        VT_CARRAY => {
            (*dest).u.lpadesc = buffer as *mut ARRAYDESC;
            let adesc_size = core::mem::offset_of!(ARRAYDESC, rgbounds)
                + size_of::<SAFEARRAYBOUND>() * (*(*src).u.lpadesc).cDims as usize;
            memcpy(
                (*dest).u.lpadesc as *mut c_void,
                (*src).u.lpadesc as *const c_void,
                adesc_size,
            );
            buffer = (buffer as *mut u8).add(adesc_size) as *mut c_void;
            buffer = TLB_CopyTypeDesc(
                &raw mut (*(*dest).u.lpadesc).tdescElem,
                &raw const (*(*src).u.lpadesc).tdescElem,
                buffer,
            );
        }
        _ => {}
    }
    buffer
}

/// free custom data allocated by MSFT_CustData
pub unsafe fn TLB_FreeCustData(custdata_list: *mut list) {
    // LIST_FOR_EACH_ENTRY_SAFE
    let mut cur = (*custdata_list).next;
    while cur != custdata_list {
        let cd = list_entry!(cur, TLBCustData, entry);
        let next = (*cur).next;
        list_remove(&raw mut (*cd).entry);
        VariantClear(&raw mut (*cd).data);
        heap_free(cd as *mut c_void);
        cur = next;
    }
}

pub unsafe fn TLB_MultiByteToBSTR(ptr: *const i8) -> BSTR {
    let len = MultiByteToWideChar(CP_ACP, 0, ptr, -1, null_mut(), 0);
    let ret = SysAllocStringLen(null(), (len - 1) as UINT);
    if ret.is_null() {
        return ret;
    }
    MultiByteToWideChar(CP_ACP, 0, ptr, -1, ret, len);
    ret
}

#[inline]
pub unsafe fn TLB_get_funcdesc_by_memberid(
    mut funcdescs: *mut TLBFuncDesc,
    mut n: UINT,
    memid: MEMBERID,
) -> *mut TLBFuncDesc {
    while n != 0 {
        if (*funcdescs).funcdesc.memid == memid {
            return funcdescs;
        }
        funcdescs = funcdescs.add(1);
        n -= 1;
    }
    null_mut()
}

#[inline]
pub unsafe fn TLB_get_vardesc_by_memberid(
    mut vardescs: *mut TLBVarDesc,
    mut n: UINT,
    memid: MEMBERID,
) -> *mut TLBVarDesc {
    while n != 0 {
        if (*vardescs).vardesc.memid == memid {
            return vardescs;
        }
        vardescs = vardescs.add(1);
        n -= 1;
    }
    null_mut()
}

#[inline]
pub unsafe fn TLB_get_vardesc_by_name(
    mut vardescs: *mut TLBVarDesc,
    mut n: UINT,
    name: *const OLECHAR,
) -> *mut TLBVarDesc {
    while n != 0 {
        if lstrcmpiW(TLB_get_bstr((*vardescs).Name), name) == 0 {
            return vardescs;
        }
        vardescs = vardescs.add(1);
        n -= 1;
    }
    null_mut()
}

#[inline]
pub unsafe fn TLB_get_custdata_by_guid(
    custdata_list: *mut list,
    guid: REFGUID,
) -> *mut TLBCustData {
    let mut cur = (*custdata_list).next;
    while cur != custdata_list {
        let cust_data = list_entry!(cur, TLBCustData, entry);
        if guid_eq(TLB_get_guid_null((*cust_data).guid), guid) {
            return cust_data;
        }
        cur = (*cur).next;
    }
    null_mut()
}

#[inline]
pub unsafe fn TLB_get_typeinfo_by_name(
    mut typeinfos: *mut *mut ITypeInfoImpl,
    mut n: UINT,
    name: *const OLECHAR,
) -> *mut ITypeInfoImpl {
    while n != 0 {
        if lstrcmpiW(TLB_get_bstr((**typeinfos).Name), name) == 0 {
            return *typeinfos;
        }
        typeinfos = typeinfos.add(1);
        n -= 1;
    }
    null_mut()
}

pub unsafe fn TLBVarDesc_Constructor(var_desc: *mut TLBVarDesc) {
    list_init(&raw mut (*var_desc).custdata_list);
}

pub unsafe fn TLBVarDesc_Alloc(n: UINT) -> *mut TLBVarDesc {
    let ret = heap_alloc_zero(size_of::<TLBVarDesc>() * n as usize) as *mut TLBVarDesc;
    if ret.is_null() {
        return null_mut();
    }
    let mut n = n;
    while n != 0 {
        TLBVarDesc_Constructor(ret.add(n as usize - 1));
        n -= 1;
    }
    ret
}

pub unsafe fn TLBParDesc_Constructor(n: UINT) -> *mut TLBParDesc {
    let ret = heap_alloc_zero(size_of::<TLBParDesc>() * n as usize) as *mut TLBParDesc;
    if ret.is_null() {
        return null_mut();
    }
    let mut n = n;
    while n != 0 {
        list_init(&raw mut (*ret.add(n as usize - 1)).custdata_list);
        n -= 1;
    }
    ret
}

pub unsafe fn TLBFuncDesc_Constructor(func_desc: *mut TLBFuncDesc) {
    list_init(&raw mut (*func_desc).custdata_list);
}

pub unsafe fn TLBFuncDesc_Alloc(n: UINT) -> *mut TLBFuncDesc {
    let ret = heap_alloc_zero(size_of::<TLBFuncDesc>() * n as usize) as *mut TLBFuncDesc;
    if ret.is_null() {
        return null_mut();
    }
    let mut n = n;
    while n != 0 {
        TLBFuncDesc_Constructor(ret.add(n as usize - 1));
        n -= 1;
    }
    ret
}

pub unsafe fn TLBImplType_Constructor(impl_: *mut TLBImplType) {
    list_init(&raw mut (*impl_).custdata_list);
}

pub unsafe fn TLBImplType_Alloc(n: UINT) -> *mut TLBImplType {
    let ret = heap_alloc_zero(size_of::<TLBImplType>() * n as usize) as *mut TLBImplType;
    if ret.is_null() {
        return null_mut();
    }
    let mut n = n;
    while n != 0 {
        TLBImplType_Constructor(ret.add(n as usize - 1));
        n -= 1;
    }
    ret
}

pub unsafe fn TLB_append_guid(
    guid_list: *mut list,
    new_guid: *const GUID,
    hreftype: HREFTYPE,
) -> *mut TLBGuid {
    let mut cur = (*guid_list).next;
    while cur != guid_list {
        let guid = list_entry!(cur, TLBGuid, entry);
        if guid_eq(&raw const (*guid).guid, new_guid) {
            return guid;
        }
        cur = (*cur).next;
    }

    let guid = heap_alloc(size_of::<TLBGuid>()) as *mut TLBGuid;
    if guid.is_null() {
        return null_mut();
    }

    memcpy(
        &raw mut (*guid).guid as *mut c_void,
        new_guid as *const c_void,
        size_of::<GUID>(),
    );
    (*guid).hreftype = hreftype as INT;

    list_add_tail(guid_list, &raw mut (*guid).entry);

    guid
}

pub unsafe fn TLB_set_custdata(
    custdata_list: *mut list,
    tlbguid: *mut TLBGuid,
    var: *mut VARIANT,
) -> HRESULT {
    match (*var).vt {
        VT_I4 | VT_R4 | VT_UI4 | VT_INT | VT_UINT | VT_HRESULT | VT_BSTR => {}
        _ => return DISP_E_BADVARTYPE,
    }

    let mut cust_data = TLB_get_custdata_by_guid(custdata_list, TLB_get_guid_null(tlbguid));

    if cust_data.is_null() {
        cust_data = heap_alloc(size_of::<TLBCustData>()) as *mut TLBCustData;
        if cust_data.is_null() {
            return E_OUTOFMEMORY;
        }

        (*cust_data).guid = tlbguid;
        VariantInit(&raw mut (*cust_data).data);

        list_add_tail(custdata_list, &raw mut (*cust_data).entry);
    } else {
        VariantClear(&raw mut (*cust_data).data);
    }

    VariantCopy(&raw mut (*cust_data).data, var)
}

pub unsafe fn TLB_append_str(string_list: *mut list, new_str: BSTR) -> *mut TLBString {
    if new_str.is_null() {
        return null_mut();
    }

    let mut cur = (*string_list).next;
    while cur != string_list {
        let str_ = list_entry!(cur, TLBString, entry);
        if wcscmp((*str_).str_, new_str) == 0 {
            return str_;
        }
        cur = (*cur).next;
    }

    let str_ = heap_alloc(size_of::<TLBString>()) as *mut TLBString;
    if str_.is_null() {
        return null_mut();
    }

    (*str_).str_ = SysAllocString(new_str);
    if (*str_).str_.is_null() {
        heap_free(str_ as *mut c_void);
        return null_mut();
    }

    list_add_tail(string_list, &raw mut (*str_).entry);

    str_
}

pub unsafe fn TLB_get_size_from_hreftype(
    info: *mut ITypeInfoImpl,
    href: HREFTYPE,
    size: *mut ULONG,
    align: *mut WORD,
) -> HRESULT {
    let mut other: *mut ITypeInfo = null_mut();
    let mut attr: *mut TYPEATTR = null_mut();

    let iface = &raw mut (*info).ITypeInfo2_iface;
    let hr = ((*(*iface).lpVtbl).GetRefTypeInfo)(iface, href, &raw mut other);
    if FAILED(hr) {
        return hr;
    }

    let hr = ((*(*other).lpVtbl).GetTypeAttr)(other, &raw mut attr);
    if FAILED(hr) {
        ((*(*other).lpVtbl).Release)(other);
        return hr;
    }

    if !size.is_null() {
        *size = (*attr).cbSizeInstance;
    }
    if !align.is_null() {
        *align = (*attr).cbAlignment;
    }

    ((*(*other).lpVtbl).ReleaseTypeAttr)(other, attr);
    ((*(*other).lpVtbl).Release)(other);

    S_OK
}

pub unsafe fn TLB_size_instance(
    info: *mut ITypeInfoImpl,
    sys: SYSKIND,
    tdesc: *mut TYPEDESC,
    size: *mut ULONG,
    align: *mut WORD,
) -> HRESULT {
    let ptr_size: ULONG = get_ptr_size(sys) as ULONG;

    match (*tdesc).vt {
        VT_VOID => {
            *size = 0;
        }
        VT_I1 | VT_UI1 => {
            *size = 1;
        }
        VT_I2 | VT_BOOL | VT_UI2 => {
            *size = 2;
        }
        VT_I4 | VT_R4 | VT_ERROR | VT_UI4 | VT_INT | VT_UINT | VT_HRESULT => {
            *size = 4;
        }
        VT_R8 | VT_I8 | VT_UI8 => {
            *size = 8;
        }
        VT_BSTR | VT_DISPATCH | VT_UNKNOWN | VT_PTR | VT_SAFEARRAY | VT_LPSTR | VT_LPWSTR => {
            *size = ptr_size;
        }
        VT_DATE => {
            *size = size_of::<DATE>() as ULONG;
        }
        VT_VARIANT => {
            *size = size_of::<VARIANT>() as ULONG;
            // a VARIANT is 8 bytes larger on 64-bit, in either direction (Wine 10.0)
            if get_ptr_size(sys) as usize != size_of::<*mut c_void>() {
                if cfg!(target_pointer_width = "64") {
                    *size -= 8;
                } else {
                    *size += 8;
                }
            }
        }
        VT_DECIMAL => {
            *size = size_of::<DECIMAL>() as ULONG;
        }
        VT_CY => {
            *size = size_of::<CY>() as ULONG;
        }
        VT_CARRAY => {
            *size = 0;
            let adesc = (*tdesc).u.lpadesc;
            let mut i: ULONG = 0;
            while i < (*adesc).cDims as ULONG {
                *size += (*(&raw const (*adesc).rgbounds)
                    .cast::<SAFEARRAYBOUND>()
                    .add(i as usize))
                .cElements;
                i += 1;
            }
            let mut sub: ULONG = 0;
            let hr = TLB_size_instance(info, sys, &raw mut (*adesc).tdescElem, &raw mut sub, align);
            if FAILED(hr) {
                return hr;
            }
            *size *= sub;
            return S_OK;
        }
        VT_USERDEFINED => {
            return TLB_get_size_from_hreftype(info, (*tdesc).u.hreftype, size, align);
        }
        _ => {
            FIXME!("Unsized VT: 0x{:x}", (*tdesc).vt);
            return E_FAIL;
        }
    }

    if !align.is_null() {
        if *size < 4 {
            *align = *size as WORD;
        } else {
            *align = 4;
        }
    }

    S_OK
}

/// TLB_is_propgetput (typelib.c:2332)
pub fn TLB_is_propgetput(invkind: INVOKEKIND) -> bool {
    invkind == INVOKE_PROPERTYGET
        || invkind == INVOKE_PROPERTYPUT
        || invkind == INVOKE_PROPERTYPUTREF
}
