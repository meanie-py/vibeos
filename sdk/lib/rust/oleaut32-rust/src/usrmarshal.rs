use crate::prelude::*;
use core::ffi::c_void;

// ---- MSHCTX / VARENUM members (wtypes.idl), under the `_`-prefixed spelling the translation uses. Undefined here, so `_MSHCTX_LOCAL` matched every flag value ----
pub const _MSHCTX_LOCAL: u32 = 0;
pub const _MSHCTX_NOSHAREDMEM: u32 = 1;
pub const _MSHCTX_DIFFERENTMACHINE: u32 = 2;
pub const _MSHCTX_INPROC: u32 = 3;
pub const _MSHCTX_CROSSCTX: u32 = 4;

// WINAPI / __RPC_USER / STDMETHODCALLTYPE externs — stdcall on i386, so
// extern "system" (was mis-declared extern "C" → referenced _Name instead of
// _Name@N and failed the i386 link). CRT memcpy/memset come from the prelude.
extern "system" {
    fn WdtpInterfacePointer_UserUnmarshal( pFlags: *mut ULONG, Buffer: *mut u8, ppunk: *mut *mut IUnknown, riid: *const IID, ) -> *mut u8;
    fn IClassFactory2_RemoteCreateInstanceLic_Proxy( This: *mut IClassFactory2, riid: REFIID, bstrKey: BSTR, ppvObj: *mut *mut IUnknown, ) -> HRESULT;
}
extern "system" {
    fn WdtpInterfacePointer_UserMarshal( pFlags: *mut ULONG, flags: ULONG, Buffer: *mut u8, punk: *mut IUnknown, riid: *const GUID, ) -> *mut u8;
    fn IEnumVARIANT_RemoteNext_Proxy( This: *mut IEnumVARIANT, celt: ULONG, rgVar: *mut VARIANT, pCeltFetched: *mut ULONG, ) -> HRESULT;
    fn ITypeComp_RemoteBind_Proxy( This: *mut ITypeComp, name: LPOLESTR, lHashVal: ULONG, flags: WORD, ti: *mut *mut ITypeInfo, desckind: *mut DESCKIND, funcdesc: *mut *mut FUNCDESC, vardesc: *mut *mut VARDESC, typecomp: *mut *mut ITypeComp, stg: *mut CLEANLOCALSTORAGE ) -> HRESULT;
    fn ITypeComp_RemoteBindType_Proxy( This: *mut ITypeComp, name: LPOLESTR, lHashVal: ULONG, ti: *mut *mut ITypeInfo, ) -> HRESULT;
    fn ITypeInfo_RemoteGetFuncDesc_Proxy( This: *mut ITypeInfo, index: UINT, ppFuncDesc: *mut *mut FUNCDESC, stg: *mut CLEANLOCALSTORAGE, ) -> HRESULT;
    fn ITypeInfo_RemoteGetDllEntry_Proxy( This: *mut ITypeInfo, memid: MEMBERID, invkind: INVOKEKIND, flags: DWORD, dll_name: *mut BSTR, name: *mut BSTR, ordinal: *mut WORD, ) -> HRESULT;
    fn ITypeInfo2_RemoteGetDocumentation2_Proxy( This: *mut ITypeInfo2, memid: MEMBERID, lcid: LCID, flags: DWORD, help_string: *mut BSTR, help_context: *mut DWORD, help_dll: *mut BSTR, ) -> HRESULT;
    fn ITypeLib_RemoteGetLibAttr_Proxy( This: *mut ITypeLib, ppTLibAttr: *mut *mut TLIBATTR, stg: *mut CLEANLOCALSTORAGE, ) -> HRESULT;
    fn ITypeLib_RemoteGetDocumentation_Proxy( This: *mut ITypeLib, index: INT, flags: DWORD, name: *mut BSTR, doc_string: *mut BSTR, help_context: *mut DWORD, help_file: *mut BSTR, ) -> HRESULT;
    fn IPropertyBag_RemoteRead_Proxy( This: *mut IPropertyBag, pszPropName: LPCOLESTR, pVar: *mut VARIANT, pErrorLog: *mut IErrorLog, vt: DWORD, pUnk: *mut IUnknown, ) -> HRESULT;
    fn IEnumConnectionPoints_RemoteNext_Proxy( This: *mut IEnumConnectionPoints, cConnections: ULONG, ppCP: *mut *mut IConnectionPoint, pcFetched: *mut ULONG, ) -> HRESULT;
    fn IPersistMemory_RemoteLoad_Proxy( This: *mut IPersistMemory, pMem: LPVOID, cbSize: ULONG, ) -> HRESULT;
    fn IPersistMemory_RemoteSave_Proxy( This: *mut IPersistMemory, pMem: LPVOID, fClearDirty: BOOL, cbSize: ULONG, ) -> HRESULT;
    fn IEnumOleUndoUnits_RemoteNext_Proxy( This: *mut IEnumOleUndoUnits, cElt: ULONG, rgElt: *mut *mut IOleUndoUnit, pcEltFetched: *mut ULONG, ) -> HRESULT;
    fn CoTaskMemFree(pv: LPVOID);
}

// RPC-generated Remote proxies + still-C-side SafeArrayCreateEx.
// These are STDMETHODCALLTYPE / WINAPI (stdcall on i386) -> extern "system".
extern "system" {
    fn IDispatch_RemoteInvoke_Proxy( This: *mut IDispatch, dispIdMember: DISPID, riid: REFIID, lcid: LCID, dwFlags: DWORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, pArgErr: *mut UINT, cVarRef: UINT, rgVarRefIdx: *mut UINT, rgVarRef: *mut VARIANTARG, ) -> HRESULT;
    fn ITypeInfo_RemoteGetTypeAttr_Proxy( This: *mut ITypeInfo, ppTypeAttr: *mut LPTYPEATTR, pDummy: *mut CLEANLOCALSTORAGE, ) -> HRESULT;
    fn ITypeInfo_RemoteGetVarDesc_Proxy( This: *mut ITypeInfo, index: UINT, ppVarDesc: *mut LPVARDESC, pDummy: *mut CLEANLOCALSTORAGE, ) -> HRESULT;
    fn ITypeInfo_RemoteGetNames_Proxy( This: *mut ITypeInfo, memid: MEMBERID, rgBstrNames: *mut BSTR, cMaxNames: UINT, pcNames: *mut UINT, ) -> HRESULT;
    fn ITypeInfo_RemoteGetDocumentation_Proxy( This: *mut ITypeInfo, memid: MEMBERID, refPtrFlags: DWORD, pBstrName: *mut BSTR, pBstrDocString: *mut BSTR, pdwHelpContext: *mut DWORD, pBstrHelpFile: *mut BSTR, ) -> HRESULT;
    fn ITypeInfo_RemoteGetContainingTypeLib_Proxy( This: *mut ITypeInfo, ppTLib: *mut *mut ITypeLib, pIndex: *mut UINT, ) -> HRESULT;
    fn ITypeLib_RemoteGetTypeInfoCount_Proxy( This: *mut ITypeLib, pcTInfo: *mut UINT, ) -> HRESULT;
    fn ITypeLib2_RemoteGetDocumentation2_Proxy( This: *mut ITypeLib2, index: INT, lcid: LCID, refPtrFlags: DWORD, pbstrHelpString: *mut BSTR, pdwHelpStringContext: *mut DWORD, pbstrHelpStringDll: *mut BSTR, ) -> HRESULT;
    fn IEnumConnections_RemoteNext_Proxy( This: *mut IEnumConnections, cConnections: ULONG, rgcd: LPCONNECTDATA, pcFetched: *mut ULONG, ) -> HRESULT;
    fn IAdviseSinkEx_RemoteOnViewStatusChange_Proxy( This: *mut IAdviseSinkEx, dwViewStatus: DWORD, ) -> HRESULT;
    fn SafeArrayCreateEx( vt: VARTYPE, cDims: UINT, rgsabound: *mut SAFEARRAYBOUND, pvExtra: LPVOID, ) -> *mut SAFEARRAY;
}

// ---- file-scope preamble (translated) ----
pub const ALIGNED_LENGTH: fn(usize, usize) -> usize = |len, align| (len + align) & !align;

pub const CLS_FUNCDESC: DWORD = b'f' as DWORD;
pub const CLS_LIBATTR: DWORD = b'l' as DWORD;
pub const CLS_TYPEATTR: DWORD = b't' as DWORD;
pub const CLS_VARDESC: DWORD = b'v' as DWORD;

#[repr(C)]
pub struct bstr_wire_t {
    pub len: DWORD,
    pub byte_len: DWORD,
    pub len2: DWORD,
}

#[repr(C)]
pub struct variant_wire_t {
    pub clSize: DWORD,
    pub rpcReserved: DWORD,
    pub vt: USHORT,
    pub wReserved1: USHORT,
    pub wReserved2: USHORT,
    pub wReserved3: USHORT,
    pub switch_is: DWORD,
}

pub const FADF_AUTOSETFLAGS: USHORT = FADF_HAVEIID | FADF_RECORD | FADF_HAVEVARTYPE |
                                        FADF_BSTR | FADF_UNKNOWN | FADF_DISPATCH |
                                        FADF_VARIANT | FADF_CREATEVECTOR ;

unsafe fn dump_user_flags(pFlags: *const DWORD) {
    if HIWORD(*pFlags) as u32 == NDR_LOCAL_DATA_REPRESENTATION {
        TRACE!(b"MAKELONG(NDR_LOCAL_REPRESENTATION, \0".as_ptr() as *const i8);
    } else {
        TRACE!(b"MAKELONG(0x%04x, \0".as_ptr() as *const i8, HIWORD(*pFlags));
    }
    match LOWORD(*pFlags) as u32 {
        _MSHCTX_LOCAL => TRACE!(b"MSHCTX_LOCAL)\0".as_ptr() as *const i8),
        _MSHCTX_NOSHAREDMEM => TRACE!(b"MSHCTX_NOSHAREDMEM)\0".as_ptr() as *const i8),
        _MSHCTX_DIFFERENTMACHINE => TRACE!(b"MSHCTX_DIFFERENTMACHINE)\0".as_ptr() as *const i8),
        _MSHCTX_INPROC => TRACE!(b"MSHCTX_INPROC)\0".as_ptr() as *const i8),
        _ => TRACE!(b"%d)\0".as_ptr() as *const i8, LOWORD(*pFlags)),
    }
}

#[no_mangle]
pub unsafe extern "system" fn CLEANLOCALSTORAGE_UserSize(
    _pFlags: *mut ULONG,
    mut Start: ULONG,
    _pstg: *mut CLEANLOCALSTORAGE,
) -> ULONG {
    Start = ((Start) + (3u32)) & !(3u32);
    Start + core::mem::size_of::<DWORD>() as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn CLEANLOCALSTORAGE_UserMarshal(
    _pFlags: *mut ULONG,
    mut Buffer: *mut u8,
    pstg: *mut CLEANLOCALSTORAGE,
) -> *mut u8 {
    ALIGN_POINTER!(Buffer, 3);
    *(Buffer as *mut DWORD) = (*pstg).flags;

    if (*pstg).pInterface.is_null() {
        return Buffer.add(core::mem::size_of::<DWORD>());
    }

    match (*pstg).flags {
        CLS_LIBATTR => {
            ITypeLib_ReleaseTLibAttr(
                (*pstg).pInterface as *mut ITypeLib,
                *((*pstg).pStorage as *mut *mut TLIBATTR),
            );
        }
        CLS_TYPEATTR => {
            ITypeInfo_ReleaseTypeAttr(
                (*pstg).pInterface as *mut ITypeInfo,
                *((*pstg).pStorage as *mut *mut TYPEATTR),
            );
        }
        CLS_FUNCDESC => {
            ITypeInfo_ReleaseFuncDesc(
                (*pstg).pInterface as *mut ITypeInfo,
                *((*pstg).pStorage as *mut *mut FUNCDESC),
            );
        }
        CLS_VARDESC => {
            ITypeInfo_ReleaseVarDesc(
                (*pstg).pInterface as *mut ITypeInfo,
                *((*pstg).pStorage as *mut *mut VARDESC),
            );
        }
        _ => {
            ERR!(b"Unknown type %x\n\0".as_ptr() as PCHAR, (*pstg).flags);
        }
    }

    *((*pstg).pStorage as *mut LPVOID) = core::ptr::null_mut();
    IUnknown_Release((*pstg).pInterface as *mut IUnknown);
    (*pstg).pInterface = core::ptr::null_mut();

    Buffer.add(core::mem::size_of::<DWORD>())
}

#[no_mangle]
pub unsafe extern "system" fn CLEANLOCALSTORAGE_UserUnmarshal(
    _pFlags: *mut ULONG,
    mut Buffer: *mut u8,
    pstr: *mut CLEANLOCALSTORAGE,
) -> *mut u8 {
    ALIGN_POINTER!(Buffer, 3);
    (*pstr).flags = *(Buffer as *const DWORD);
    Buffer.offset(core::mem::size_of::<DWORD>() as isize)
}

#[no_mangle]
pub unsafe extern "system" fn CLEANLOCALSTORAGE_UserFree(
    _pFlags: *mut ULONG,
    _pstr: *mut CLEANLOCALSTORAGE,
) {
}

#[no_mangle]
pub unsafe extern "system" fn BSTR_UserSize(
    _pFlags: *mut ULONG,
    Start: ULONG,
    pstr: *mut BSTR,
) -> ULONG {

    TRACE!(
        b"(%x,%d,%p) => %p\n\0".as_ptr() as *const i8,
        *pFlags,
        Start,
        pstr,
        *pstr,
    );
    if !(*pstr).is_null() {
        TRACE!(
            b"string=%s\n\0".as_ptr() as *const i8,
            debugstr_w(*pstr),
        );
    }
    let mut Start = Start;
    ALIGN_LENGTH!(Start, 3);
    Start += core::mem::size_of::<bstr_wire_t>() as ULONG
        + ((SysStringByteLen(*pstr) + 1) & !1);
    TRACE!(b"returning %d\n\0".as_ptr() as *const i8, Start);
    Start
}
#[no_mangle]
pub unsafe extern "system" fn BSTR_UserMarshal(_pFlags: *mut ULONG, mut Buffer: *mut u8, pstr: *mut *mut u16) -> *mut u8 {

    let len: DWORD = SysStringByteLen(*pstr);

    TRACE!(b"(%x,%p,%p) => %p\n\0".as_ptr() as *const i8, *pFlags, Buffer, pstr, *pstr);
    if !(*pstr).is_null() {
        TRACE!(b"string=%s\n\0".as_ptr() as *const i8, debugstr_w(*pstr));
    }

    ALIGN_POINTER!(Buffer, 3);
    let header = Buffer as *mut bstr_wire_t;
    (*header).len = (len + 1) / 2;
    (*header).len2 = (len + 1) / 2;
    if !(*pstr).is_null() {
        (*header).byte_len = len;
        memcpy((header.offset(1)) as *mut core::ffi::c_void, *pstr as *const core::ffi::c_void, ((*header).len * 2) as usize);
    } else {
        (*header).byte_len = 0xffffffff;
    }

    Buffer.offset((core::mem::size_of::<bstr_wire_t>() + (core::mem::size_of::<u16>() * (*header).len as usize)) as isize)
}

#[no_mangle]
pub unsafe extern "system" fn BSTR_UserUnmarshal(
    _pFlags: *mut ULONG,
    Buffer: *mut u8,
    pstr: *mut *mut u16,
) -> *mut u8 {
    let mut buffer_ptr = Buffer;
    
    TRACE!(
        b"(%x,%p,%p) => %p\n\0".as_ptr() as *const i8,
        *pFlags,
        Buffer,
        pstr,
        *pstr,
    );
    
    ALIGN_POINTER!(buffer_ptr, 3);

    let header = buffer_ptr as *const bstr_wire_t;
    
    if (*header).len != (*header).len2 {
        FIXME!(
            b"len %08x != len2 %08x\n\0".as_ptr() as *const i8,
            (*header).len,
            (*header).len2,
        );
    }
    
    if (*header).byte_len == 0xffffffff {
        SysFreeString(*pstr);
        *pstr = core::ptr::null_mut();
    } else {
        SysReAllocStringLen(
            pstr,
            (header.add(1)) as *const u16,
            (*header).len,
        );
    }
    
    if !(*pstr).is_null() {
        TRACE!(
            b"string=%s\n\0".as_ptr() as *const i8,
            debugstr_w(*pstr),
        );
    }
    
    buffer_ptr
        .add(core::mem::size_of::<bstr_wire_t>())
        .add((core::mem::size_of::<u16>() as DWORD * (*header).len) as usize)
}

#[no_mangle]
pub unsafe extern "system" fn BSTR_UserFree(_pFlags: *mut ULONG, pstr: *mut BSTR) {
    TRACE!(
        b"(%x,%p) => %p\n\0".as_ptr() as *const i8,
        *pFlags,
        pstr,
        *pstr,
    );
    SysFreeString(*pstr);
    *pstr = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn get_type_size(_pFlags: *mut ULONG, vt: VARTYPE) -> u32 {
    if vt & VT_ARRAY != 0 {
        return 4;
    }

    match vt & !VT_BYREF {
        VT_EMPTY | VT_NULL => 0,
        VT_I1 | VT_UI1 => core::mem::size_of::<CHAR>() as u32,
        VT_I2 | VT_UI2 => core::mem::size_of::<SHORT>() as u32,
        VT_I4 | VT_UI4 | VT_HRESULT => core::mem::size_of::<LONG>() as u32,
        VT_INT | VT_UINT => core::mem::size_of::<INT>() as u32,
        VT_I8 | VT_UI8 => core::mem::size_of::<LONGLONG>() as u32,
        VT_R4 => core::mem::size_of::<FLOAT>() as u32,
        VT_R8 => core::mem::size_of::<DOUBLE>() as u32,
        VT_BOOL => core::mem::size_of::<VARIANT_BOOL>() as u32,
        VT_ERROR => core::mem::size_of::<SCODE>() as u32,
        VT_DATE => core::mem::size_of::<DATE>() as u32,
        VT_CY => core::mem::size_of::<CY>() as u32,
        VT_DECIMAL => core::mem::size_of::<DECIMAL>() as u32,
        VT_BSTR => core::mem::size_of::<ULONG>() as u32,
        VT_VARIANT => core::mem::size_of::<VARIANT>() as u32,
        VT_UNKNOWN | VT_DISPATCH | VT_RECORD => 0,
        _ => {
            FIXME!(b"unhandled VT %d\n\0".as_ptr() as PCHAR, vt);
            0
        }
    }
}

unsafe fn get_type_alignment(pFlags: *mut ULONG, vt: VARTYPE) -> u32 {
    let size = get_type_size(pFlags, vt);
    if (vt & VT_BYREF as u16) != 0 {
        return 3;
    }
    if size == 0 {
        return 0;
    }
    if size <= 4 {
        return size - 1;
    }
    return 7;
}

/* WdtpInterfacePointer_UserSize takes care of 2 additional DWORDs to store marshalling buffer size */
unsafe fn interface_user_size(pFlags: *mut ULONG, Start: ULONG, riid: *const GUID, punk: *mut IUnknown) -> u32 {
    let mut size: ULONG = 0;

    if !punk.is_null() {
        size = WdtpInterfacePointer_UserSize(pFlags, LOWORD(*pFlags) as ULONG, 0, punk, riid);
        if size == 0 {
            ERR!(b"interface variant buffer size calculation failed\n\0".as_ptr() as *const i8);
            return 0;
        }
    }
    size += core::mem::size_of::<ULONG>() as u32;
    TRACE!(b"wire-size extra of interface variant is %d\n\0".as_ptr() as *const i8, size);
    return Start + size;
}

unsafe fn wire_extra_user_size(pFlags: *mut ULONG, Start: ULONG, pvar: *mut VARIANT) -> ULONG {
    if V_ISARRAY(pvar) {
        if V_ISBYREF(pvar) {
            return LPSAFEARRAY_UserSize(pFlags, Start, *V_ARRAYREF_ref(pvar) );
        } else {
            return LPSAFEARRAY_UserSize(pFlags, Start, core::ptr::addr_of_mut!((*V_ARRAY_ref(pvar))));
        }
    }

    match *V_VT_ref(pvar)  {
        VT_BSTR => {
            return BSTR_UserSize(pFlags, Start, core::ptr::addr_of_mut!((*V_BSTR_ref(pvar))));
        }
        vt if vt == (VT_BSTR | VT_BYREF) => {
            return BSTR_UserSize(pFlags, Start, *V_BSTRREF_ref(pvar) );
        }
        vt if vt == (VT_VARIANT | VT_BYREF) => {
            return VARIANT_UserSize(pFlags, Start, *V_VARIANTREF_ref(pvar) );
        }
        VT_UNKNOWN => {
            return interface_user_size(pFlags, Start, &IID_IUnknown, *V_UNKNOWN_ref(pvar) );
        }
        vt if vt == (VT_UNKNOWN | VT_BYREF) => {
            return interface_user_size(pFlags, Start, &IID_IUnknown, *(*V_UNKNOWNREF_ref(pvar)));
        }
        VT_DISPATCH => {
            return interface_user_size(pFlags, Start, &IID_IDispatch, (*V_DISPATCH_ref(pvar)) as *mut IUnknown);
        }
        vt if vt == (VT_DISPATCH | VT_BYREF) => {
            return interface_user_size(pFlags, Start, &IID_IDispatch, *(*V_DISPATCHREF_ref(pvar)) as *mut IUnknown);
        }
        VT_RECORD => {
            FIXME!(b"wire-size record\n\0".as_ptr() as PCHAR);
            return Start;
        }
        VT_SAFEARRAY => {
            FIXME!(b"wire-size safearray: shouldn't be marshaling this\n\0".as_ptr() as PCHAR);
            return Start;
        }
        vt if vt == (VT_SAFEARRAY | VT_BYREF) => {
            FIXME!(b"wire-size safearray: shouldn't be marshaling this\n\0".as_ptr() as PCHAR);
            return Start;
        }
        _ => {
            return Start;
        }
    }
}
unsafe fn interface_user_marshal(
    pFlags: *mut ULONG,
    Buffer: *mut u8,
    riid: *const GUID,
    punk: *mut IUnknown,
) -> *mut u8 {

    TRACE!(
        b"pFlags=%d, Buffer=%p, pUnk=%p\n\0".as_ptr() as *const i8,
        *pFlags,
        Buffer,
        punk,
    );

    if punk.is_null() {
        core::ptr::write_bytes(Buffer, 0, core::mem::size_of::<ULONG>()); // nolint: silent-failure (Buffer is *mut u8; zeroes sizeof(ULONG)=4 bytes, matches C memset(Buffer,0,sizeof(ULONG)))
        return Buffer.add(core::mem::size_of::<ULONG>());
    } else {
        *(Buffer as *mut DWORD) = punk as DWORD_PTR as DWORD;
        let Buffer = Buffer.add(core::mem::size_of::<DWORD>());
        return WdtpInterfacePointer_UserMarshal(
            pFlags,
            LOWORD(*pFlags) as ULONG,
            Buffer,
            punk,
            riid,
        );
    }
}

#[cfg(not(feature = "__REACTOS__"))]
unsafe fn interface_variant_marshal(
    pFlags: *mut ULONG,
    Buffer: *mut u8,
    riid: *const GUID,
    punk: *mut IUnknown,
) -> *mut u8 {

    TRACE!(
        b"pFlags=%d, Buffer=%p, pUnk=%p\n\0".as_ptr() as *const i8,
        *pFlags,
        Buffer,
        punk,
    );

    if punk.is_null() {
        core::ptr::write_bytes(Buffer, 0, core::mem::size_of::<ULONG>()); // nolint: silent-failure (Buffer is *mut u8; zeroes sizeof(ULONG)=4 bytes, matches C memset(Buffer,0,sizeof(ULONG)))
        return Buffer.add(core::mem::size_of::<ULONG>());
    } else {
        *(Buffer as *mut DWORD) = punk as DWORD_PTR as DWORD;
        let Buffer = Buffer.add(core::mem::size_of::<DWORD>());
        return WdtpInterfacePointer_UserMarshal(
            pFlags,
            LOWORD(*pFlags) as ULONG,
            Buffer,
            punk,
            riid,
        );
    }
}

unsafe fn interface_user_unmarshal(
    pFlags: *mut ULONG,
    mut Buffer: *mut u8,
    riid: *const IID,
    ppunk: *mut *mut IUnknown,
) -> *mut u8 {

    let ptr: DWORD;

    TRACE!(
        b"pFlags=%d, Buffer=%p, ppUnk=%p\n\0".as_ptr(),
        *pFlags,
        Buffer,
        ppunk,
    );

    ptr = *(Buffer as *const DWORD);
    Buffer = Buffer.add(core::mem::size_of::<DWORD>());

    if ptr == 0 {
        return Buffer;
    }

    WdtpInterfacePointer_UserUnmarshal(pFlags, Buffer, ppunk, riid)
}

#[no_mangle]
pub unsafe extern "system" fn VARIANT_UserSize(
    pFlags: *mut ULONG,
    Start: ULONG,
    pvar: *mut VARIANT,
) -> ULONG {
    let mut Start = Start;
    let align: i32;

    Start = (Start + 7) & !7 ;
    Start += core::mem::size_of::<variant_wire_t>() as ULONG;
    
    if (*pvar).n1.n2.vt & VT_BYREF as u16 != 0 {
        Start += 4;
    }

    align = get_type_alignment(pFlags, (*pvar).n1.n2.vt) as i32;
    Start = (Start + align as ULONG) & !(align as ULONG) ;

    if (*pvar).n1.n2.vt == (VT_VARIANT | VT_BYREF) as u16 {
        Start += 4;
    } else {
        Start += get_type_size(pFlags, (*pvar).n1.n2.vt);
    }
    
    Start = wire_extra_user_size(pFlags, Start, pvar);

    Start
}

#[no_mangle]
pub unsafe extern "system" fn VARIANT_UserMarshal(
    pFlags: *mut ULONG,
    Buffer: *mut u8,
    pvar: *mut VARIANT,
) -> *mut u8 {
    let header: *mut variant_wire_t;
    let type_size: ULONG;
    let align: i32;
    let mut Pos: *mut u8;

    TRACE!(
        b"(%x,%p,%p)\n\0".as_ptr() as *const i8,
        *pFlags,
        Buffer,
        pvar,
    );
    TRACE!(b"vt=%04x\n\0".as_ptr() as *const i8, (*V_VT_ref(pvar)));

    let mut aligned_buffer = Buffer;
    ALIGN_POINTER!(aligned_buffer, 7);
    let Buffer = aligned_buffer;

    header = Buffer as *mut variant_wire_t;

    (*header).clSize = 0;
    (*header).rpcReserved = 0;
    (*header).vt = (*pvar).n1.n2.vt;
    (*header).wReserved1 = (*pvar).n1.n2.wReserved1;
    (*header).wReserved2 = (*pvar).n1.n2.wReserved2;
    (*header).wReserved3 = (*pvar).n1.n2.wReserved3;
    (*header).switch_is = (*pvar).n1.n2.vt as DWORD;
    if (*header).switch_is & VT_ARRAY as DWORD != 0 {
        (*header).switch_is &= !(VT_TYPEMASK as DWORD);
    }

    Pos = header.offset(1) as *mut u8;
    type_size = get_type_size(pFlags, *V_VT_ref(pvar) );
    align = get_type_alignment(pFlags, *V_VT_ref(pvar) ) as i32;
    ALIGN_POINTER!(Pos, align);

    if (*header).vt & VT_BYREF as USHORT != 0 {
        *(Pos as *mut DWORD) = max(type_size, 4);
        Pos = Pos.offset(4);
        if (*header).vt & VT_TYPEMASK as USHORT != VT_VARIANT as USHORT {
            core::ptr::copy_nonoverlapping(
                (*pvar).n1.n2.n3.byref as *const u8,
                Pos,
                type_size as usize,
            );
            Pos = Pos.offset(type_size as isize);
        } else {
            *(Pos as *mut DWORD) = 
                (b'U' as DWORD) | 
                ((b's' as DWORD) << 8) | 
                ((b'e' as DWORD) << 16) | 
                ((b'r' as DWORD) << 24);
            Pos = Pos.offset(4);
        }
    } else {
        if (*header).vt & VT_TYPEMASK as USHORT == VT_DECIMAL as USHORT {
            core::ptr::copy_nonoverlapping(
                pvar as *const u8,
                Pos,
                type_size as usize,
            );
        } else {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*pvar).n1.n2.n3) as *const u8,
                Pos,
                type_size as usize,
            );
        }
        Pos = Pos.offset(type_size as isize);
    }

    if (*header).vt & VT_ARRAY as USHORT != 0 {
        if (*header).vt & VT_BYREF as USHORT != 0 {
            Pos = LPSAFEARRAY_UserMarshal(pFlags, Pos, *V_ARRAYREF_ref(pvar) );
        } else {
            Pos = LPSAFEARRAY_UserMarshal(pFlags, Pos, core::ptr::addr_of_mut!((*V_ARRAY_ref(pvar))));
        }
    } else {
        match (*header).vt {
            x if x == VT_BSTR as USHORT => {
                Pos = BSTR_UserMarshal(pFlags, Pos, core::ptr::addr_of_mut!((*V_BSTR_ref(pvar))));
            }
            x if x == (VT_BSTR | VT_BYREF) as USHORT => {
                Pos = BSTR_UserMarshal(pFlags, Pos, *V_BSTRREF_ref(pvar) );
            }
            x if x == (VT_VARIANT | VT_BYREF) as USHORT => {
                Pos = VARIANT_UserMarshal(pFlags, Pos, *V_VARIANTREF_ref(pvar) );
            }
            x if x == VT_UNKNOWN as USHORT => {
                #[cfg(feature = "__REACTOS__")]
                {
                    Pos = interface_user_marshal(pFlags, Pos, &IID_IUnknown, (*V_UNKNOWN_ref(pvar)));
                }
                #[cfg(not(feature = "__REACTOS__"))]
                {
                    Pos = interface_variant_marshal(pFlags, Pos, &IID_IUnknown, *V_UNKNOWN_ref(pvar) );
                }
            }
            x if x == (VT_UNKNOWN | VT_BYREF) as USHORT => {
                #[cfg(feature = "__REACTOS__")]
                {
                    Pos = interface_user_marshal(pFlags, Pos, &IID_IUnknown, *(*V_UNKNOWNREF_ref(pvar)));
                }
                #[cfg(not(feature = "__REACTOS__"))]
                {
                    Pos = interface_variant_marshal(pFlags, Pos, &IID_IUnknown, *(*V_UNKNOWNREF_ref(pvar)));
                }
            }
            x if x == VT_DISPATCH as USHORT => {
                #[cfg(feature = "__REACTOS__")]
                {
                    Pos = interface_user_marshal(
                        pFlags,
                        Pos,
                        &IID_IDispatch,
                        (*V_DISPATCH_ref(pvar)) as *mut IUnknown,
                    );
                }
                #[cfg(not(feature = "__REACTOS__"))]
                {
                    Pos = interface_variant_marshal(
                        pFlags,
                        Pos,
                        &IID_IDispatch,
                        (*V_DISPATCH_ref(pvar)) as *mut IUnknown,
                    );
                }
            }
            x if x == (VT_DISPATCH | VT_BYREF) as USHORT => {
                #[cfg(feature = "__REACTOS__")]
                {
                    Pos = interface_user_marshal(
                        pFlags,
                        Pos,
                        &IID_IDispatch,
                        *(*V_DISPATCHREF_ref(pvar)) as *mut IUnknown,
                    );
                }
                #[cfg(not(feature = "__REACTOS__"))]
                {
                    Pos = interface_variant_marshal(
                        pFlags,
                        Pos,
                        &IID_IDispatch,
                        *(*V_DISPATCHREF_ref(pvar)) as *mut IUnknown,
                    );
                }
            }
            x if x == VT_RECORD as USHORT => {
                FIXME!(b"handle BRECORD by val\n\0".as_ptr() as *const i8);
            }
            x if x == (VT_RECORD | VT_BYREF) as USHORT => {
                FIXME!(b"handle BRECORD by ref\n\0".as_ptr() as *const i8);
            }
            _ => {}
        }
    }
    (*header).clSize = ((Pos.offset_from(Buffer) as usize + 7) >> 3) as DWORD;
    TRACE!(
        b"marshalled size=%d\n\0".as_ptr() as *const i8,
        (*header).clSize,
    );
    Pos
}

#[no_mangle]
pub unsafe extern "system" fn VARIANT_UserUnmarshal(
    pFlags: *mut ULONG,
    mut Buffer: *mut u8,
    pvar: *mut VARIANT,
) -> *mut u8 {
    let header: *mut variant_wire_t;
    let type_size: ULONG;
    let align: i32;
    let mut Pos: *mut u8;

    TRACE!(
        b"(%x,%p,%p)\n\0".as_ptr() as PCHAR,
        *pFlags,
        Buffer,
        pvar,
    );

    ALIGN_POINTER!(Buffer, 7);

    header = Buffer as *mut variant_wire_t;

    Pos = (header.offset(1)) as *mut u8;
    type_size = get_type_size(pFlags, (*header).vt);
    align = get_type_alignment(pFlags, (*header).vt) as i32;
    ALIGN_POINTER!(Pos, align);

    if (*header).vt & VT_BYREF != 0 {
        let mem_size: ULONG;
        Pos = Pos.offset(4);

        // Wine 10.0: a byref ARRAY holds a SAFEARRAY pointer. get_type_size says
        // 4 for any VT_ARRAY (its WIRE size), so the pre-10 code allocated 4
        // bytes and the unmarshal then stored an 8-byte pointer: a heap
        // overflow on amd64/arm64.
        if (*header).vt & VT_ARRAY != 0 {
            mem_size = core::mem::size_of::<*mut core::ffi::c_void>() as ULONG;
        } else {
            match (*header).vt & !VT_BYREF {
                VT_UNKNOWN | VT_DISPATCH | VT_BSTR => {
                    mem_size = core::mem::size_of::<*mut core::ffi::c_void>() as ULONG;
                }
                _ => {
                    mem_size = type_size;
                }
            }
        }

        if (*V_VT_ref(pvar)) != (*header).vt {
            VariantClear(pvar);
            *V_BYREF_ref(pvar) = CoTaskMemAlloc(mem_size as SIZE_T);
            memset(*V_BYREF_ref(pvar), 0, mem_size as usize);
        } else if (*V_BYREF_ref(pvar)).is_null() {
            *V_BYREF_ref(pvar) = CoTaskMemAlloc(mem_size as SIZE_T);
            memset(*V_BYREF_ref(pvar), 0, mem_size as usize);
        }

        if ((*header).vt & VT_ARRAY) == 0
            && ((*header).vt & VT_TYPEMASK) != VT_BSTR
            && ((*header).vt & VT_TYPEMASK) != VT_VARIANT
            && ((*header).vt & VT_TYPEMASK) != VT_UNKNOWN
            && ((*header).vt & VT_TYPEMASK) != VT_DISPATCH
            && ((*header).vt & VT_TYPEMASK) != VT_RECORD
        {
            memcpy(*V_BYREF_ref(pvar), Pos as LPVOID, type_size as usize);
        }

        if ((*header).vt & VT_TYPEMASK) != VT_VARIANT {
            Pos = Pos.offset(type_size as isize);
        } else {
            Pos = Pos.offset(4);
        }
    } else {
        VariantClear(pvar);
        if (*header).vt & VT_ARRAY != 0 {
            (*V_ARRAY_ref(pvar)) = core::ptr::null_mut();
        } else if ((*header).vt & VT_TYPEMASK) == VT_BSTR {
            (*V_BSTR_ref(pvar)) = core::ptr::null_mut();
        } else if ((*header).vt & VT_TYPEMASK) == VT_UNKNOWN {
            (*V_UNKNOWN_ref(pvar)) = core::ptr::null_mut();
        } else if ((*header).vt & VT_TYPEMASK) == VT_DISPATCH {
            (*V_DISPATCH_ref(pvar)) = core::ptr::null_mut();
        } else if ((*header).vt & VT_TYPEMASK) == VT_RECORD {
            *V_RECORD_ref(pvar) = core::ptr::null_mut();
        } else if ((*header).vt & VT_TYPEMASK) == VT_DECIMAL {
            memcpy(pvar as LPVOID, Pos as LPVOID, type_size as usize);
        } else {
            memcpy(
                core::ptr::addr_of_mut!((*pvar).n1.n2.n3) as LPVOID,
                Pos as LPVOID,
                type_size as usize,
            );
        }
        Pos = Pos.offset(type_size as isize);
    }

    (*pvar).n1.n2.vt = (*header).vt;
    (*pvar).n1.n2.wReserved1 = (*header).wReserved1;
    (*pvar).n1.n2.wReserved2 = (*header).wReserved2;
    (*pvar).n1.n2.wReserved3 = (*header).wReserved3;

    if (*header).vt & VT_ARRAY != 0 {
        if (*header).vt & VT_BYREF != 0 {
            Pos = LPSAFEARRAY_UserUnmarshal(pFlags, Pos, *V_ARRAYREF_ref(pvar) );
        } else {
            Pos = LPSAFEARRAY_UserUnmarshal(pFlags, Pos, V_ARRAY_ref(pvar));
        }
    } else {
        match (*header).vt {
            VT_BSTR => {
                Pos = BSTR_UserUnmarshal(pFlags, Pos, V_BSTR_ref(pvar));
            }
            _ if (*header).vt == (VT_BSTR | VT_BYREF) => {
                Pos = BSTR_UserUnmarshal(pFlags, Pos, *V_BSTRREF_ref(pvar));
            }
            _ if (*header).vt == (VT_VARIANT | VT_BYREF) => {
                Pos = VARIANT_UserUnmarshal(pFlags, Pos, *V_VARIANTREF_ref(pvar));
            }
            VT_UNKNOWN => {
                Pos = interface_user_unmarshal(pFlags, Pos, &IID_IUnknown, V_UNKNOWN_ref(pvar));
            }
            _ if (*header).vt == (VT_UNKNOWN | VT_BYREF) => {
                Pos = interface_user_unmarshal(pFlags, Pos, &IID_IUnknown, *V_UNKNOWNREF_ref(pvar));
            }
            VT_DISPATCH => {
                Pos = interface_user_unmarshal(
                    pFlags,
                    Pos,
                    &IID_IDispatch,
                    V_DISPATCH_ref(pvar) as *mut *mut IUnknown,
                );
            }
            _ if (*header).vt == (VT_DISPATCH | VT_BYREF) => {
                Pos = interface_user_unmarshal(
                    pFlags,
                    Pos,
                    &IID_IDispatch,
                    *V_DISPATCHREF_ref(pvar) as *mut *mut IUnknown,
                );
            }
            VT_RECORD => {
                FIXME!(b"handle BRECORD by val\n\0".as_ptr() as PCHAR);
            }
            _ if (*header).vt == (VT_RECORD | VT_BYREF) => {
                FIXME!(b"handle BRECORD by ref\n\0".as_ptr() as PCHAR);
            }
            _ => {}
        }
    }
    Pos
}

#[no_mangle]
pub unsafe extern "system" fn VARIANT_UserFree(pFlags: *mut ULONG, pvar: *mut VARIANT) {

    let vt: VARTYPE = *V_VT_ref(pvar) ;
    let mut ref_: LPVOID = core::ptr::null_mut();

    TRACE!(b"(%x,%p)\n\0".as_ptr(), *pFlags, pvar);
    TRACE!(b"vt=%04x\n\0".as_ptr(), (*V_VT_ref(pvar)));

    if (vt & VT_BYREF) != 0 {
        ref_ = (*pvar).n1.n2.n3.byref;
    }

    VariantClear(pvar);
    if ref_.is_null() {
        return;
    }

    if (vt & VT_ARRAY) != 0 {
        if (vt & VT_BYREF) != 0 {
            LPSAFEARRAY_UserFree(pFlags, *V_ARRAYREF_ref(pvar) );
        } else {
            LPSAFEARRAY_UserFree(pFlags, V_ARRAY_ref(pvar));
        }
    } else {
        match vt {
            _ if vt == (VT_BSTR | VT_BYREF) => {
                BSTR_UserFree(pFlags, *V_BSTRREF_ref(pvar) );
            }
            _ if vt == (VT_VARIANT | VT_BYREF) => {
                VARIANT_UserFree(pFlags, *V_VARIANTREF_ref(pvar) );
            }
            _ if vt == (VT_RECORD | VT_BYREF) => {
                FIXME!(b"handle BRECORD by ref\n\0".as_ptr());
            }
            _ if vt == (VT_UNKNOWN | VT_BYREF) || vt == (VT_DISPATCH | VT_BYREF) => {
                if !(*(*V_UNKNOWNREF_ref(pvar))).is_null() {
                    IUnknown_Release(*(*V_UNKNOWNREF_ref(pvar)));
                }
            }
            _ => {}
        }
    }

    CoTaskMemFree(ref_);
}

unsafe fn SAFEARRAY_GetCellCount(psa: *const SAFEARRAY) -> ULONG {
    let mut psab: *const SAFEARRAYBOUND = core::ptr::addr_of!((*psa).rgsabound) as *const SAFEARRAYBOUND;
    let mut cCount: USHORT = (*psa).cDims;
    let mut ulNumCells: ULONG = 1;

    while cCount > 0 {
        cCount = cCount.wrapping_sub(1);
        if (*psab).cElements == 0 {
            return 0;
        }
        ulNumCells = ulNumCells.wrapping_mul((*psab).cElements);
        psab = psab.offset(1);
    }
    return ulNumCells;
}

unsafe fn SAFEARRAY_GetUnionType(psa: *mut SAFEARRAY) -> SF_TYPE {
    let mut vt: VARTYPE = 0;
    let hr: HRESULT;

    hr = SafeArrayGetVartype(psa, &mut vt);
    if FAILED(hr) {
        if ((*psa).fFeatures & FADF_VARIANT) != 0 {
            return SF_VARIANT;
        }

        match (*psa).cbElements {
            1 => vt = VT_I1,
            2 => vt = VT_I2,
            4 => vt = VT_I4,
            8 => vt = VT_I8,
            _ => RpcRaiseException(hr as u32),
        }
    }

    if ((*psa).fFeatures & FADF_HAVEIID) != 0 {
        return SF_HAVEIID;
    }

    match vt {
        VT_I1 | VT_UI1 => SF_I1,
        VT_BOOL | VT_I2 | VT_UI2 => SF_I2,
        VT_INT | VT_UINT | VT_I4 | VT_UI4 | VT_R4 => SF_I4,
        VT_DATE | VT_CY | VT_R8 | VT_I8 | VT_UI8 => SF_I8,
        VT_INT_PTR | VT_UINT_PTR => {
            if core::mem::size_of::<UINT_PTR>() == 4 {
                SF_I4
            } else {
                SF_I8
            }
        }
        VT_BSTR => SF_BSTR,
        VT_DISPATCH => SF_DISPATCH,
        VT_VARIANT => SF_VARIANT,
        VT_UNKNOWN => SF_UNKNOWN,
        VT_RECORD => SF_RECORD,
        _ => SF_ERROR,
    }
}

#[no_mangle]
pub unsafe extern "system" fn LPSAFEARRAY_UserSize(
    pFlags: *mut ULONG,
    StartingSize: ULONG,
    ppsa: *mut LPSAFEARRAY,
) -> ULONG {
    let mut size = StartingSize;

    TRACE!(b"(\0".as_ptr() as *const i8);
    dump_user_flags(pFlags);
    TRACE!(b", %d, %p\n\0".as_ptr() as *const i8, StartingSize, *ppsa);

    ALIGN_LENGTH!(size, 3);
    size += core::mem::size_of::<ULONG>() as ULONG;
    if !(*ppsa).is_null() {
        let psa = *ppsa;
        let mut ulCellCount = SAFEARRAY_GetCellCount(psa);
        let sftype: SF_TYPE;
        let hr: HRESULT;

        size += core::mem::size_of::<ULONG>() as ULONG;
        size += 2 * core::mem::size_of::<USHORT>() as ULONG + 2 * core::mem::size_of::<ULONG>() as ULONG;

        sftype = SAFEARRAY_GetUnionType(psa);
        size += core::mem::size_of::<ULONG>() as ULONG;

        size += core::mem::size_of::<ULONG>() as ULONG;
        size += core::mem::size_of::<ULONG>() as ULONG;
        if sftype == SF_HAVEIID {
            size += core::mem::size_of::<IID>() as ULONG;
        }

        size += (core::mem::size_of_val(&(*psa).rgsabound[0]) * (*psa).cDims as usize) as ULONG;

        size += core::mem::size_of::<ULONG>() as ULONG;

        match sftype {
            SF_BSTR => {
                let mut lpBstr = (*psa).pvData as *mut BSTR;

                while ulCellCount > 0 {
                    size = BSTR_UserSize(pFlags, size, lpBstr);
                    ulCellCount -= 1;
                    lpBstr = lpBstr.offset(1);
                }
            }
            SF_DISPATCH | SF_UNKNOWN | SF_HAVEIID => {
                let mut lpUnk = (*psa).pvData as *mut *mut IUnknown;
                let mut guid: GUID = core::mem::zeroed();

                if sftype == SF_HAVEIID {
                    SafeArrayGetIID(psa, &mut guid);
                } else if sftype == SF_UNKNOWN {
                    guid = IID_IUnknown;
                } else {
                    guid = IID_IDispatch;
                }

                while ulCellCount > 0 {
                    size = interface_user_size(pFlags, size, &guid, *lpUnk);
                    ulCellCount -= 1;
                    lpUnk = lpUnk.offset(1);
                }
            }
            SF_VARIANT => {
                let mut lpVariant = (*psa).pvData as *mut VARIANT;

                while ulCellCount > 0 {
                    size = VARIANT_UserSize(pFlags, size, lpVariant);
                    ulCellCount -= 1;
                    lpVariant = lpVariant.offset(1);
                }
            }
            SF_RECORD => {
                let mut pRecInfo: *mut IRecordInfo = core::ptr::null_mut();

                hr = SafeArrayGetRecordInfo(psa, &mut pRecInfo);
                if FAILED(hr) {
                    RpcRaiseException(hr as u32);
                }

                if !pRecInfo.is_null() {
                    FIXME!(b"size record info %p\n\0".as_ptr() as *const i8, pRecInfo);

                    IRecordInfo_Release(pRecInfo);
                }
            }
            SF_I8 => {
                ALIGN_LENGTH!(size, 7);
                size += ulCellCount * (*psa).cbElements;
            }
            SF_I1 | SF_I2 | SF_I4 => {
                size += ulCellCount * (*psa).cbElements;
            }
            _ => {}
        }
    }

    size
}

unsafe fn elem_wire_size(lpsa: LPSAFEARRAY, sftype: SF_TYPE) -> DWORD {
    match sftype {
        SF_BSTR | SF_HAVEIID | SF_UNKNOWN | SF_DISPATCH => core::mem::size_of::<DWORD>() as DWORD,
        SF_VARIANT => {
            (core::mem::size_of::<variant_wire_t>() - core::mem::size_of::<DWORD>()) as DWORD
        }
        _ => (*lpsa).cbElements,
    }
}

unsafe fn elem_mem_size(wiresa: wireSAFEARRAY, sftype: SF_TYPE) -> DWORD {
    match sftype {
        SF_HAVEIID | SF_UNKNOWN | SF_DISPATCH => core::mem::size_of::<*mut c_void>() as DWORD,
        SF_BSTR => core::mem::size_of::<BSTR>() as DWORD,
        SF_VARIANT => core::mem::size_of::<VARIANT>() as DWORD,
        _ => (*wiresa).cbElements,
    }
}

#[no_mangle]
pub unsafe extern "system" fn LPSAFEARRAY_UserMarshal(
    pFlags: *mut ULONG,
    mut Buffer: *mut u8,
    ppsa: *mut LPSAFEARRAY,
) -> *mut u8 {
    let mut hr: HRESULT;

    TRACE!(b"(\0".as_ptr() as *const i8);
    dump_user_flags(pFlags);
    TRACE!(b", %p, &%p\n\0".as_ptr() as *const i8, Buffer, *ppsa);

    ALIGN_POINTER!(Buffer, 3);
    *(Buffer as *mut ULONG) = if !(*ppsa).is_null() { 0x1 } else { 0x0 };
    Buffer = Buffer.add(core::mem::size_of::<ULONG>());
    if !(*ppsa).is_null() {
        let mut vt: VARTYPE = 0;
        let psa: *mut SAFEARRAY = *ppsa;
        let mut ulCellCount: ULONG = SAFEARRAY_GetCellCount(psa);
        let mut bound: *mut SAFEARRAYBOUND;
        let sftype: SF_TYPE;
        let mut guid: GUID = core::mem::zeroed();
        let mut i: INT;

        sftype = SAFEARRAY_GetUnionType(psa);

        *(Buffer as *mut ULONG) = (*psa).cDims as ULONG;
        Buffer = Buffer.add(core::mem::size_of::<ULONG>());
        *(Buffer as *mut USHORT) = (*psa).cDims;
        Buffer = Buffer.add(core::mem::size_of::<USHORT>());
        *(Buffer as *mut USHORT) = (*psa).fFeatures;
        Buffer = Buffer.add(core::mem::size_of::<USHORT>());
        *(Buffer as *mut ULONG) = elem_wire_size(psa, sftype);
        Buffer = Buffer.add(core::mem::size_of::<ULONG>());

        hr = SafeArrayGetVartype(psa, &mut vt);
        if ((*psa).fFeatures & FADF_HAVEIID) != 0 || FAILED(hr) {
            vt = 0;
        }

        *(Buffer as *mut ULONG) = (*psa).cLocks as USHORT as ULONG | ((vt as ULONG) << 16);
        Buffer = Buffer.add(core::mem::size_of::<ULONG>());

        *(Buffer as *mut ULONG) = sftype;
        Buffer = Buffer.add(core::mem::size_of::<ULONG>());

        *(Buffer as *mut ULONG) = ulCellCount;
        Buffer = Buffer.add(core::mem::size_of::<ULONG>());
        *(Buffer as *mut ULONG) = if !(*psa).pvData.is_null() { 0x2 } else { 0x0 };
        Buffer = Buffer.add(core::mem::size_of::<ULONG>());
        if sftype == SF_HAVEIID {
            SafeArrayGetIID(psa, &mut guid);
            memcpy(
                Buffer as *mut c_void,
                core::ptr::addr_of!(guid) as *const c_void,
                core::mem::size_of::<GUID>(),
            );
            Buffer = Buffer.add(core::mem::size_of::<GUID>());
        }

        /* bounds are marshaled in opposite order comparing to storage layout */
        let rgsabound_ptr = (*psa).rgsabound.as_ptr();
        bound = Buffer as *mut SAFEARRAYBOUND;
        i = 0;
        while i < (*psa).cDims as INT {
            memcpy(
                bound as *mut c_void,
                rgsabound_ptr.add((*psa).cDims as usize - i as usize - 1) as *const c_void,
                core::mem::size_of::<SAFEARRAYBOUND>(),
            );
            bound = bound.add(1);
            i += 1;
        }
        Buffer = Buffer.add(core::mem::size_of::<SAFEARRAYBOUND>() * (*psa).cDims as usize);

        *(Buffer as *mut ULONG) = ulCellCount;
        Buffer = Buffer.add(core::mem::size_of::<ULONG>());

        if !(*psa).pvData.is_null() {
            match sftype {
                SF_BSTR => {
                    let mut lpBstr = (*psa).pvData as *mut BSTR;
                    while ulCellCount != 0 {
                        Buffer = BSTR_UserMarshal(pFlags, Buffer, lpBstr);
                        ulCellCount -= 1;
                        lpBstr = lpBstr.add(1);
                    }
                }
                SF_DISPATCH | SF_UNKNOWN | SF_HAVEIID => {
                    let mut lpUnk = (*psa).pvData as *mut *mut IUnknown;
                    let iid: *const GUID = if sftype == SF_HAVEIID {
                        core::ptr::addr_of!(guid)
                    } else if sftype == SF_UNKNOWN {
                        &IID_IUnknown
                    } else {
                        &IID_IDispatch
                    };

                    while ulCellCount != 0 {
                        Buffer = interface_user_marshal(pFlags, Buffer, iid, *lpUnk);
                        ulCellCount -= 1;
                        lpUnk = lpUnk.add(1);
                    }
                }
                SF_VARIANT => {
                    let mut lpVariant = (*psa).pvData as *mut VARIANT;
                    while ulCellCount != 0 {
                        Buffer = VARIANT_UserMarshal(pFlags, Buffer, lpVariant);
                        ulCellCount -= 1;
                        lpVariant = lpVariant.add(1);
                    }
                }
                SF_RECORD => {
                    let mut pRecInfo: *mut IRecordInfo = core::ptr::null_mut();

                    hr = SafeArrayGetRecordInfo(psa, &mut pRecInfo);
                    if FAILED(hr) {
                        RpcRaiseException(hr as u32);
                    }

                    if !pRecInfo.is_null() {
                        FIXME!(b"write record info %p\n\0".as_ptr() as *const i8, pRecInfo);

                        IRecordInfo_Release(pRecInfo);
                    }
                }
                SF_I8 => {
                    ALIGN_POINTER!(Buffer, 7);
                    memcpy(
                        Buffer as *mut c_void,
                        (*psa).pvData as *const c_void,
                        (ulCellCount * (*psa).cbElements) as usize,
                    );
                    Buffer = Buffer.add((ulCellCount * (*psa).cbElements) as usize);
                }
                SF_I1 | SF_I2 | SF_I4 => {
                    /* Just copy the data over */
                    memcpy(
                        Buffer as *mut c_void,
                        (*psa).pvData as *const c_void,
                        (ulCellCount * (*psa).cbElements) as usize,
                    );
                    Buffer = Buffer.add((ulCellCount * (*psa).cbElements) as usize);
                }
                _ => {}
            }
        }
    }
    Buffer
}

#[no_mangle]
pub unsafe extern "system" fn LPSAFEARRAY_UserUnmarshal(
    pFlags: *mut ULONG,
    mut Buffer: *mut u8,
    ppsa: *mut *mut SAFEARRAY,
) -> *mut u8 {
    let mut ptr: ULONG;
    let wiresa: wireSAFEARRAY;
    let cDims: ULONG;
    let mut hr: HRESULT;
    let sftype: ULONG;
    let cell_count: ULONG;
    let mut guid: GUID = core::mem::zeroed();
    let vt: VARTYPE;
    let wiresab: *mut SAFEARRAYBOUND;

    TRACE!(b"(\0".as_ptr() as _);
    dump_user_flags(pFlags);
    TRACE!(b", %p, %p\n\0".as_ptr() as _, Buffer, ppsa);

    ALIGN_POINTER!(Buffer, 3);
    ptr = *(Buffer as *mut ULONG);
    Buffer = Buffer.offset(core::mem::size_of::<ULONG>() as isize);

    if ptr == 0 {
        SafeArrayDestroy(*ppsa);
        *ppsa = core::ptr::null_mut();

        TRACE!(b"NULL safe array unmarshaled\n\0".as_ptr() as _);

        return Buffer;
    }

    cDims = *(Buffer as *mut ULONG);
    Buffer = Buffer.offset(core::mem::size_of::<ULONG>() as isize);

    wiresa = Buffer as wireSAFEARRAY;
    Buffer = Buffer.offset((2 * core::mem::size_of::<USHORT>() + 2 * core::mem::size_of::<ULONG>()) as isize);

    if cDims != (*wiresa).cDims as ULONG {
        RpcRaiseException(RPC_S_INVALID_BOUND as u32);
    }

    vt = HIWORD((*wiresa).cLocks);

    sftype = *(Buffer as *mut ULONG);
    Buffer = Buffer.offset(core::mem::size_of::<ULONG>() as isize);

    cell_count = *(Buffer as *mut ULONG);
    Buffer = Buffer.offset(core::mem::size_of::<ULONG>() as isize);
    ptr = *(Buffer as *mut ULONG);
    Buffer = Buffer.offset(core::mem::size_of::<ULONG>() as isize);
    if sftype == SF_HAVEIID {
        memcpy(
            core::ptr::addr_of_mut!(guid) as *mut _,
            Buffer as *const _,
            core::mem::size_of::<GUID>(),
        );
        Buffer = Buffer.offset(core::mem::size_of::<GUID>() as isize);
    }

    wiresab = Buffer as *mut SAFEARRAYBOUND;
    Buffer = Buffer.offset((core::mem::size_of::<SAFEARRAYBOUND>() * (*wiresa).cDims as usize) as isize);

    if !(*ppsa).is_null() && (**ppsa).cDims == (*wiresa).cDims {
        if ((**ppsa).fFeatures & !FADF_AUTOSETFLAGS) != ((*wiresa).fFeatures & !FADF_AUTOSETFLAGS) {
            RpcRaiseException(0x80020009u32);
        }

        if SAFEARRAY_GetCellCount(*ppsa) * (**ppsa).cbElements
            != cell_count * elem_mem_size(wiresa, sftype)
        {
            // FADF_AUTO(0x1) | FADF_STATIC(0x2) | FADF_EMBEDDED(0x4) | FADF_FIXEDSIZE(0x10);
            // FADF_AUTO and FADF_EMBEDDED are absent from consts.rs (see SHARED-SURFACE REQUESTS).
            if ((**ppsa).fFeatures & (0x0001u16 | FADF_STATIC | 0x0004u16 | FADF_FIXEDSIZE)) != 0 {
                RpcRaiseException(0x80020009u32);
            }

            hr = SafeArrayDestroyData(*ppsa);
            if FAILED(hr) {
                RpcRaiseException(hr as u32);
            }
        }
        memcpy(
            (**ppsa).rgsabound.as_mut_ptr() as *mut _,
            wiresab as *const _,
            core::mem::size_of::<SAFEARRAYBOUND>() * (*wiresa).cDims as usize,
        );

        if ((**ppsa).fFeatures & FADF_HAVEVARTYPE) != 0 {
            *(((*ppsa) as *mut DWORD).offset(-1)) = vt as DWORD;
        }
    } else if vt != 0 {
        SafeArrayDestroy(*ppsa);
        *ppsa = SafeArrayCreateEx(vt, (*wiresa).cDims as UINT, wiresab, core::ptr::null_mut());
        if (*ppsa).is_null() {
            RpcRaiseException(0x8007000Eu32);
        }
    } else {
        SafeArrayDestroy(*ppsa);
        if FAILED(SafeArrayAllocDescriptor((*wiresa).cDims as UINT, ppsa)) {
            RpcRaiseException(0x8007000Eu32);
        }
        memcpy(
            (**ppsa).rgsabound.as_mut_ptr() as *mut _,
            wiresab as *const _,
            core::mem::size_of::<SAFEARRAYBOUND>() * (*wiresa).cDims as usize,
        );
    }

    (**ppsa).fFeatures &= FADF_AUTOSETFLAGS;
    (**ppsa).fFeatures |= (*wiresa).fFeatures & !FADF_AUTOSETFLAGS;
    (**ppsa).cbElements = elem_mem_size(wiresa, sftype);

    if (**ppsa).pvData.is_null() {
        hr = SafeArrayAllocData(*ppsa);
        if FAILED(hr) {
            RpcRaiseException(hr as u32);
        }
    }

    if (*(Buffer as *mut ULONG) != cell_count) || (SAFEARRAY_GetCellCount(*ppsa) != cell_count) {
        RpcRaiseException(RPC_S_INVALID_BOUND as u32);
    }
    Buffer = Buffer.offset(core::mem::size_of::<ULONG>() as isize);

    if ptr != 0 {
        match sftype {
            SF_BSTR => {
                let mut lpBstr: *mut BSTR = (**ppsa).pvData as *mut BSTR;
                let mut count = cell_count;

                while count > 0 {
                    Buffer = BSTR_UserUnmarshal(pFlags, Buffer, lpBstr);
                    count -= 1;
                    lpBstr = lpBstr.offset(1);
                }
            }
            SF_DISPATCH | SF_UNKNOWN | SF_HAVEIID => {
                let mut lpUnk: *mut *mut IUnknown = (**ppsa).pvData as *mut *mut IUnknown;
                let iid: *const GUID = if sftype == SF_HAVEIID {
                    core::ptr::addr_of!(guid)
                } else if sftype == SF_UNKNOWN {
                    &IID_IUnknown
                } else {
                    &IID_IDispatch
                };

                let mut count = cell_count;
                while count > 0 {
                    Buffer = interface_user_unmarshal(pFlags, Buffer, iid, lpUnk);
                    count -= 1;
                    lpUnk = lpUnk.offset(1);
                }
            }
            SF_VARIANT => {
                let mut lpVariant: *mut VARIANT = (**ppsa).pvData as *mut VARIANT;
                let mut count = cell_count;

                while count > 0 {
                    Buffer = VARIANT_UserUnmarshal(pFlags, Buffer, lpVariant);
                    count -= 1;
                    lpVariant = lpVariant.offset(1);
                }
            }
            SF_RECORD => {
                FIXME!(b"set record info\n\0".as_ptr() as _);
            }
            SF_I8 => {
                ALIGN_POINTER!(Buffer, 7);
                memcpy(
                    (**ppsa).pvData,
                    Buffer as *const _,
                    cell_count as usize * (**ppsa).cbElements as usize,
                );
                Buffer = Buffer.offset((cell_count * (**ppsa).cbElements) as isize);
            }
            SF_I1 | SF_I2 | SF_I4 => {
                memcpy(
                    (**ppsa).pvData,
                    Buffer as *const _,
                    cell_count as usize * (**ppsa).cbElements as usize,
                );
                Buffer = Buffer.offset((cell_count * (**ppsa).cbElements) as isize);
            }
            _ => {}
        }
    }

    TRACE!(b"safe array unmarshaled: %p\n\0".as_ptr() as _, *ppsa);

    return Buffer;
}
#[no_mangle]
pub unsafe extern "system" fn LPSAFEARRAY_UserFree(pFlags: *mut ULONG, ppsa: *mut LPSAFEARRAY) {

    TRACE!(b"(\0".as_ptr());
    dump_user_flags(pFlags);
    TRACE!(b", &%p\n\0".as_ptr(), *ppsa);

    SafeArrayDestroy(*ppsa);
    *ppsa = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn HFONT_UserSize(
    _pFlags: *mut ULONG,
    _Start: ULONG,
    _phfont: *mut HFONT,
) -> ULONG {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    0
}

#[no_mangle]
pub unsafe extern "system" fn HFONT_UserMarshal(
    _pFlags: *mut ULONG,
    _Buffer: *mut u8,
    _phfont: *mut HFONT,
) -> *mut u8 {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn HFONT_UserUnmarshal(
    _pFlags: *mut ULONG,
    _Buffer: *mut u8,
    _phfont: *mut HFONT,
) -> *mut u8 {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn HFONT_UserFree(_pFlags: *mut ULONG, _phfont: *mut HFONT) {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
}
#[no_mangle]
pub unsafe extern "system" fn IDispatch_Invoke_Proxy(
    This: *mut IDispatch,
    dispIdMember: DISPID,
    riid: REFIID,
    lcid: LCID,
    wFlags: WORD,
    pDispParams: *mut DISPPARAMS,
    pVarResult: *mut VARIANT,
    pExcepInfo: *mut EXCEPINFO,
    puArgErr: *mut UINT,
) -> HRESULT {
    let hr: HRESULT;
    let mut VarResult: VARIANT = core::mem::zeroed();
    let mut rgVarRefIdx: *mut UINT = core::ptr::null_mut();
    let mut rgVarRef: *mut VARIANTARG = core::ptr::null_mut();
    let mut u: UINT;
    let mut cVarRef: UINT;
    let mut uArgErr: UINT = 0;
    let mut ExcepInfo: EXCEPINFO = core::mem::zeroed();

    let mut pVarResult = pVarResult;
    let mut puArgErr = puArgErr;
    let mut pExcepInfo = pExcepInfo;

    TRACE!(
        b"(%p)->(%d,%s,%x,%x,%p,%p,%p,%p)\n\0".as_ptr() as PCHAR,
        This,
        dispIdMember,
        debugstr_guid(riid),
        lcid,
        wFlags,
        pDispParams,
        pVarResult,
        pExcepInfo,
        puArgErr,
    );

    if pVarResult.is_null() {
        pVarResult = &mut VarResult;
    }
    if puArgErr.is_null() {
        puArgErr = &mut uArgErr;
    }
    if pExcepInfo.is_null() {
        pExcepInfo = &mut ExcepInfo;
    }

    cVarRef = 0;
    u = 0;
    while u < (*pDispParams).cArgs {
        let arg: *mut VARIANTARG = (*pDispParams).rgvarg.offset(u as isize);
        if V_ISBYREF(arg) {
            cVarRef += 1;
        }
        u += 1;
    }

    if cVarRef != 0 {
        rgVarRefIdx = CoTaskMemAlloc((core::mem::size_of::<UINT>() * cVarRef as usize) as SIZE_T) as *mut UINT;
        rgVarRef = CoTaskMemAlloc((core::mem::size_of::<VARIANTARG>() * cVarRef as usize) as SIZE_T) as *mut VARIANTARG;
        cVarRef = 0;
        u = 0;
        while u < (*pDispParams).cArgs {
            let arg: *mut VARIANTARG = (*pDispParams).rgvarg.offset(u as isize);
            if V_ISBYREF(arg) {
                *rgVarRefIdx.offset(cVarRef as isize) = u;
                VariantInit(rgVarRef.offset(cVarRef as isize));
                VariantCopy(rgVarRef.offset(cVarRef as isize), arg);
                VariantClear(arg);
                cVarRef += 1;
            }
            u += 1;
        }
    } else {
        rgVarRefIdx = puArgErr;
        rgVarRef = pVarResult as *mut VARIANTARG;
    }

    TRACE!(b"passed by ref: %d args\n\0".as_ptr() as PCHAR, cVarRef);

    hr = IDispatch_RemoteInvoke_Proxy(
        This,
        dispIdMember,
        riid,
        lcid,
        wFlags as DWORD,
        pDispParams,
        pVarResult,
        pExcepInfo,
        puArgErr,
        cVarRef,
        rgVarRefIdx,
        rgVarRef,
    );

    if cVarRef != 0 {
        u = 0;
        while u < cVarRef {
            let i: u32 = *rgVarRefIdx.offset(u as isize);
            VariantCopy(
                (*pDispParams).rgvarg.offset(i as isize),
                rgVarRef.offset(u as isize),
            );
            VariantClear(rgVarRef.offset(u as isize));
            u += 1;
        }
        CoTaskMemFree(rgVarRef as LPVOID);
        CoTaskMemFree(rgVarRefIdx as LPVOID);
    }

    if pExcepInfo == &mut ExcepInfo as *mut EXCEPINFO {
        SysFreeString((*pExcepInfo).bstrSource);
        SysFreeString((*pExcepInfo).bstrDescription);
        SysFreeString((*pExcepInfo).bstrHelpFile);
    }

    hr
}
#[no_mangle]
pub unsafe extern "system" fn IDispatch_Invoke_Stub(
    This: *mut IDispatch,
    dispIdMember: DISPID,
    riid: REFIID,
    lcid: LCID,
    dwFlags: DWORD,
    pDispParams: *mut DISPPARAMS,
    pVarResult: *mut VARIANT,
    pExcepInfo: *mut EXCEPINFO,
    pArgErr: *mut UINT,
    cVarRef: UINT,
    rgVarRefIdx: *mut UINT,
    rgVarRef: *mut VARIANTARG,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let rgvarg: *mut VARIANTARG;
    let arg: *mut VARIANTARG;
    let mut u: UINT;

    VariantInit(pVarResult);
    core::ptr::write_bytes(pExcepInfo, 0, 1);
    *pArgErr = 0;

    rgvarg = (*pDispParams).rgvarg;
    arg = CoTaskMemAlloc((core::mem::size_of::<VARIANTARG>() * (*pDispParams).cArgs as usize) as _) as *mut VARIANTARG;
    if arg.is_null() {
        return E_OUTOFMEMORY;
    }

    u = 0;
    while u < (*pDispParams).cArgs {
        VariantInit(&mut *arg.add(u as usize));
        u += 1;
    }

    u = 0;
    while u < (*pDispParams).cArgs {
        hr = VariantCopy(arg.add(u as usize), rgvarg.add(u as usize));
        if FAILED(hr) {
            break;
        }
        u += 1;
    }

    if SUCCEEDED(hr) {
        u = 0;
        while u < cVarRef {
            let i: u32 = *rgVarRefIdx.add(u as usize);
            VariantCopy(arg.add(i as usize), rgVarRef.add(u as usize));
            u += 1;
        }

        (*pDispParams).rgvarg = arg;

        hr = IDispatch_Invoke(
            This,
            dispIdMember,
            riid,
            lcid,
            dwFlags as WORD,
            pDispParams,
            pVarResult,
            pExcepInfo,
            pArgErr,
        );

        u = 0;
        while u < cVarRef {
            let i: u32 = *rgVarRefIdx.add(u as usize);
            VariantCopy(rgVarRef.add(u as usize), arg.add(i as usize));
            u += 1;
        }
    }

    u = 0;
    while u < (*pDispParams).cArgs {
        VariantClear(&mut *arg.add(u as usize));
        u += 1;
    }

    (*pDispParams).rgvarg = rgvarg;
    CoTaskMemFree(arg as *mut _);

    hr
}
#[no_mangle]
pub unsafe extern "system" fn IEnumVARIANT_Next_Proxy(
    This: *mut IEnumVARIANT,
    celt: ULONG,
    rgVar: *mut VARIANT,
    pCeltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    let pCeltFetched = if pCeltFetched.is_null() {
        core::ptr::addr_of_mut!(fetched)
    } else {
        pCeltFetched
    };
    
    
    IEnumVARIANT_RemoteNext_Proxy(This, celt, rgVar, pCeltFetched)
}
#[no_mangle]
pub unsafe extern "system" fn IEnumVARIANT_Next_Stub(
    This: *mut IEnumVARIANT,
    celt: ULONG,
    rgVar: *mut VARIANT,
    pCeltFetched: *mut ULONG,
) -> HRESULT {

    let hr: HRESULT;
    *pCeltFetched = 0;
    hr = IEnumVARIANT_Next(This, celt, rgVar, pCeltFetched);
    if hr == S_OK {
        *pCeltFetched = celt;
    }
    hr
}

unsafe fn free_embedded_arraydesc(adesc: *mut ARRAYDESC) {
    match (*adesc).tdescElem.vt {
        VT_PTR | VT_SAFEARRAY => {
            free_embedded_typedesc((*adesc).tdescElem.__bindgen_anon_1.lptdesc);
            CoTaskMemFree((*adesc).tdescElem.__bindgen_anon_1.lptdesc as LPVOID);
        }
        VT_CARRAY => {
            free_embedded_arraydesc((*adesc).tdescElem.__bindgen_anon_1.lpadesc);
            CoTaskMemFree((*adesc).tdescElem.__bindgen_anon_1.lpadesc as LPVOID);
        }
        _ => {}
    }
}

unsafe fn free_embedded_typedesc(tdesc: *mut TYPEDESC) {
    match (*tdesc).vt {
        VT_PTR | VT_SAFEARRAY => {
            free_embedded_typedesc((*tdesc).__bindgen_anon_1.lptdesc);
            CoTaskMemFree((*tdesc).__bindgen_anon_1.lptdesc as *mut core::ffi::c_void);
        }
        VT_CARRAY => {
            free_embedded_arraydesc((*tdesc).__bindgen_anon_1.lpadesc);
            CoTaskMemFree((*tdesc).__bindgen_anon_1.lpadesc as *mut core::ffi::c_void);
        }
        _ => {}
    }
}

unsafe fn free_embedded_elemdesc(edesc: *mut ELEMDESC) {
    free_embedded_typedesc(core::ptr::addr_of_mut!((*edesc).tdesc));
    if (*edesc).__bindgen_anon_1.paramdesc.wParamFlags & PARAMFLAG_FHASDEFAULT as u16 != 0 {
        CoTaskMemFree((*edesc).__bindgen_anon_1.paramdesc.pparamdescex as LPVOID);
    }
}
#[no_mangle]
pub unsafe extern "system" fn ITypeComp_Bind_Proxy(
    This: *mut ITypeComp,
    name: LPOLESTR,
    lHashVal: ULONG,
    flags: WORD,
    ti: *mut *mut ITypeInfo,
    desckind: *mut DESCKIND,
    bindptr: *mut BINDPTR
) -> HRESULT {

    let mut stg: CLEANLOCALSTORAGE = core::mem::zeroed();
    let mut typecomp: *mut ITypeComp = core::ptr::null_mut();
    let mut funcdesc: *mut FUNCDESC = core::ptr::null_mut();
    let mut vardesc: *mut VARDESC = core::ptr::null_mut();

    TRACE!(
        b"(%p, %s, %#x, %#x, %p, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_w(name),
        lHashVal,
        flags,
        ti,
        desckind,
        bindptr
    );

    *desckind = DESCKIND_NONE;
    memset(bindptr as *mut core::ffi::c_void, 0, core::mem::size_of::<BINDPTR>());

    let hr = ITypeComp_RemoteBind_Proxy(
        This,
        name,
        lHashVal,
        flags,
        ti,
        desckind,
        &mut funcdesc,
        &mut vardesc,
        &mut typecomp,
        &mut stg
    );

    if hr == S_OK {
        match *desckind {
            DESCKIND_FUNCDESC => {
                (*bindptr).lpfuncdesc = funcdesc;
            }
            DESCKIND_VARDESC | DESCKIND_IMPLICITAPPOBJ => {
                (*bindptr).lpvardesc = vardesc;
            }
            DESCKIND_TYPECOMP => {
                (*bindptr).lptcomp = typecomp;
            }
            _ => {}
        }
    }

    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeComp_Bind_Stub(
    This: *mut ITypeComp,
    name: LPOLESTR,
    lHashVal: ULONG,
    flags: WORD,
    ti: *mut *mut ITypeInfo,
    desckind: *mut DESCKIND,
    funcdesc: *mut *mut FUNCDESC,
    vardesc: *mut *mut VARDESC,
    typecomp: *mut *mut ITypeComp,
    stg: *mut CLEANLOCALSTORAGE,
) -> HRESULT {

    let mut bindptr: BINDPTR = core::mem::zeroed();
    let hr: HRESULT;

    TRACE!(
        b"(%p, %s, %#x, %#x, %p, %p, %p, %p, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_w(name),
        lHashVal,
        flags,
        ti,
        desckind,
        funcdesc,
        vardesc,
        typecomp,
        stg,
    );

    memset(stg as *mut core::ffi::c_void, 0, core::mem::size_of::<CLEANLOCALSTORAGE>());
    memset(&mut bindptr as *mut BINDPTR as *mut core::ffi::c_void, 0, core::mem::size_of::<BINDPTR>());

    *funcdesc = core::ptr::null_mut();
    *vardesc = core::ptr::null_mut();
    *typecomp = core::ptr::null_mut();
    *ti = core::ptr::null_mut();

    hr = ITypeComp_Bind(This, name, lHashVal, flags, ti, desckind, &mut bindptr);
    if hr != S_OK {
        return hr;
    }

    match *desckind {
        DESCKIND_FUNCDESC => {
            *funcdesc = bindptr.lpfuncdesc;
            (*stg).pInterface = *ti as *mut IUnknown;
            (*stg).pStorage = funcdesc as *mut core::ffi::c_void;
            (*stg).flags = CLS_FUNCDESC;
        }
        DESCKIND_VARDESC | DESCKIND_IMPLICITAPPOBJ => {
            *vardesc = bindptr.lpvardesc;
            (*stg).pInterface = *ti as *mut IUnknown;
            (*stg).pStorage = vardesc as *mut core::ffi::c_void;
            (*stg).flags = CLS_VARDESC;
        }
        DESCKIND_TYPECOMP => {
            *typecomp = bindptr.lptcomp;
        }
        _ => {}
    }

    if !(*stg).pInterface.is_null() {
        IUnknown_AddRef((*stg).pInterface);
    }

    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeComp_BindType_Proxy(
    This: *mut ITypeComp,
    name: LPOLESTR,
    lHashVal: ULONG,
    ti: *mut *mut ITypeInfo,
    typecomp: *mut *mut ITypeComp,
) -> HRESULT {

    let hr: HRESULT;

    TRACE!(
        b"(%p, %s, %#x, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_w(name),
        lHashVal,
        ti,
        typecomp,
    );

    hr = ITypeComp_RemoteBindType_Proxy(This, name, lHashVal, ti);
    if hr == 0 {
        ITypeInfo_GetTypeComp(*ti, typecomp);
    } else if !typecomp.is_null() {
        *typecomp = core::ptr::null_mut();
    }

    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeComp_BindType_Stub(
    This: *mut ITypeComp,
    name: LPOLESTR,
    lHashVal: ULONG,
    ti: *mut *mut ITypeInfo,
) -> HRESULT {
    let mut typecomp: *mut ITypeComp = core::ptr::null_mut();
    let hr: HRESULT;

    TRACE!(
        b"(%p, %s, %#x, %p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_w(name),
        lHashVal,
        ti,
    );

    hr = ITypeComp_BindType(This, name, lHashVal, ti, &mut typecomp);

    if !typecomp.is_null() {
        ITypeComp_Release(typecomp);
    }

    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetTypeAttr_Proxy(
    This: *mut ITypeInfo,
    ppTypeAttr: *mut *mut TYPEATTR,
) -> HRESULT {
    let mut stg: CLEANLOCALSTORAGE = core::mem::zeroed();
    
    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const i8, This, ppTypeAttr);
    
    stg.flags = 0;
    stg.pStorage = core::ptr::null_mut();
    stg.pInterface = core::ptr::null_mut();
    
    ITypeInfo_RemoteGetTypeAttr_Proxy(This, ppTypeAttr, &mut stg)
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetTypeAttr_Stub(
    This: *mut ITypeInfo,
    ppTypeAttr: *mut LPTYPEATTR,
    pDummy: *mut CLEANLOCALSTORAGE,
) -> HRESULT {
    let hr: HRESULT;
    TRACE!(b"(%p, %p)\n\0".as_ptr() as PCHAR, This, ppTypeAttr);

    hr = ITypeInfo_GetTypeAttr(This, ppTypeAttr);
    if hr != S_OK {
        return hr;
    }

    (*pDummy).flags = CLS_TYPEATTR;
    ITypeInfo_AddRef(This);
    (*pDummy).pInterface = This as *mut IUnknown;
    (*pDummy).pStorage = ppTypeAttr as *mut core::ffi::c_void;
    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetFuncDesc_Proxy(
    This: *mut ITypeInfo,
    index: UINT,
    ppFuncDesc: *mut *mut FUNCDESC,
) -> HRESULT {

    let mut stg: CLEANLOCALSTORAGE = core::mem::zeroed();
    TRACE!(b"(%p, %d, %p)\n\0".as_ptr(), This, index, ppFuncDesc);

    stg.flags = 0;
    stg.pStorage = core::ptr::null_mut();
    stg.pInterface = core::ptr::null_mut();

    ITypeInfo_RemoteGetFuncDesc_Proxy(This, index, ppFuncDesc, &mut stg)
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetFuncDesc_Stub(
    This: *mut ITypeInfo,
    index: UINT,
    ppFuncDesc: *mut LPFUNCDESC,
    pDummy: *mut CLEANLOCALSTORAGE,
) -> HRESULT {

    let hr: HRESULT;
    TRACE!(b"(%p, %d, %p)\n\0".as_ptr() as *const i8, This, index, ppFuncDesc);

    hr = ITypeInfo_GetFuncDesc(This, index, ppFuncDesc);
    if hr != S_OK {
        return hr;
    }

    (*pDummy).flags = CLS_FUNCDESC;
    ITypeInfo_AddRef(This);
    (*pDummy).pInterface = This as *mut IUnknown;
    (*pDummy).pStorage = ppFuncDesc as LPVOID;
    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetVarDesc_Proxy(
    This: *mut ITypeInfo,
    index: UINT,
    ppVarDesc: *mut *mut VARDESC,
) -> HRESULT {
    let mut stg: CLEANLOCALSTORAGE = core::mem::zeroed();
    TRACE!(b"(%p, %d, %p)\n\0".as_ptr() as PCHAR, This, index, ppVarDesc);

    stg.flags = 0;
    stg.pStorage = core::ptr::null_mut();
    stg.pInterface = core::ptr::null_mut();

    ITypeInfo_RemoteGetVarDesc_Proxy(This, index, ppVarDesc, core::ptr::addr_of_mut!(stg))
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetVarDesc_Stub(
    This: *mut ITypeInfo,
    index: UINT,
    ppVarDesc: *mut LPVARDESC,
    pDummy: *mut CLEANLOCALSTORAGE,
) -> HRESULT {
    let hr: HRESULT;
    TRACE!(b"(%p, %d, %p)\n\0".as_ptr() as PCHAR, This, index, ppVarDesc);

    hr = ITypeInfo_GetVarDesc(This, index, ppVarDesc);
    if hr != S_OK {
        return hr;
    }

    (*pDummy).flags = CLS_VARDESC;
    ITypeInfo_AddRef(This);
    (*pDummy).pInterface = This as *mut IUnknown;
    (*pDummy).pStorage = ppVarDesc as LPVOID;
    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetNames_Proxy(
    This: *mut ITypeInfo,
    memid: MEMBERID,
    rgBstrNames: *mut BSTR,
    cMaxNames: UINT,
    pcNames: *mut UINT,
) -> HRESULT {
    TRACE!(
        b"(%p, %08x, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        memid,
        rgBstrNames,
        cMaxNames,
        pcNames,
    );

    ITypeInfo_RemoteGetNames_Proxy(This, memid, rgBstrNames, cMaxNames, pcNames)
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetNames_Stub(
    This: *mut ITypeInfo,
    memid: MEMBERID,
    rgBstrNames: *mut BSTR,
    cMaxNames: UINT,
    pcNames: *mut UINT,
) -> HRESULT {
    TRACE!(
        b"(%p, %08x, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        memid,
        rgBstrNames,
        cMaxNames,
        pcNames,
    );

    ITypeInfo_GetNames(This, memid, rgBstrNames, cMaxNames, pcNames)
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetIDsOfNames_Proxy(
    _This: *mut ITypeInfo,
    _rgszNames: *mut LPOLESTR,
    _cNames: UINT,
    _pMemId: *mut MEMBERID,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as *const i8);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetIDsOfNames_Stub(
    _This: *mut ITypeInfo
) -> HRESULT {
    FIXME!("not implemented\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_Invoke_Proxy(
    _This: *mut ITypeInfo,
    _pvInstance: PVOID,
    _memid: MEMBERID,
    _wFlags: WORD,
    _pDispParams: *mut DISPPARAMS,
    _pVarResult: *mut VARIANT,
    _pExcepInfo: *mut EXCEPINFO,
    _puArgErr: *mut UINT,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as PCHAR);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_Invoke_Stub(
    _This: *mut ITypeInfo,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as *const i8);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetDocumentation_Proxy(
    This: *mut ITypeInfo,
    memid: MEMBERID,
    name: *mut BSTR,
    doc_string: *mut BSTR,
    help_context: *mut DWORD,
    help_file: *mut BSTR,
) -> HRESULT {
    let mut dummy_help_context: DWORD = 0;
    let mut flags: DWORD = 0;
    let mut dummy_name: BSTR = core::ptr::null_mut();
    let mut dummy_doc_string: BSTR = core::ptr::null_mut();
    let mut dummy_help_file: BSTR = core::ptr::null_mut();
    let hr: HRESULT;

    TRACE!(
        b"(%p, %08x, %p, %p, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        memid,
        name,
        doc_string,
        help_context,
        help_file,
    );

    let name = if name.is_null() {
        &mut dummy_name as *mut BSTR
    } else {
        flags = 1;
        name
    };

    let doc_string = if doc_string.is_null() {
        &mut dummy_doc_string as *mut BSTR
    } else {
        flags |= 2;
        doc_string
    };

    let help_context = if help_context.is_null() {
        &mut dummy_help_context as *mut DWORD
    } else {
        flags |= 4;
        help_context
    };

    let help_file = if help_file.is_null() {
        &mut dummy_help_file as *mut BSTR
    } else {
        flags |= 8;
        help_file
    };

    hr = ITypeInfo_RemoteGetDocumentation_Proxy(
        This,
        memid,
        flags,
        name,
        doc_string,
        help_context,
        help_file,
    );

    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetDocumentation_Stub(
    This: *mut ITypeInfo,
    memid: MEMBERID,
    flags: DWORD,
    name: *mut BSTR,
    doc_string: *mut BSTR,
    help_context: *mut DWORD,
    help_file: *mut BSTR,
) -> HRESULT {

    TRACE!(
        b"(%p, %08x, %08x, %p, %p, %p, %p)\n\0".as_ptr() as PCHAR,
        This,
        memid,
        flags,
        name,
        doc_string,
        help_context,
        help_file,
    );

    *name = core::ptr::null_mut();
    *doc_string = core::ptr::null_mut();
    *help_file = core::ptr::null_mut();
    *help_context = 0;

    let name_ptr = if (flags & 1) == 0 { core::ptr::null_mut() } else { name };
    let doc_string_ptr = if (flags & 2) == 0 { core::ptr::null_mut() } else { doc_string };
    let help_context_ptr = if (flags & 4) == 0 { core::ptr::null_mut() } else { help_context };
    let help_file_ptr = if (flags & 8) == 0 { core::ptr::null_mut() } else { help_file };

    ITypeInfo_GetDocumentation(This, memid, name_ptr, doc_string_ptr, help_context_ptr, help_file_ptr)
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetDllEntry_Proxy(
    This: *mut ITypeInfo,
    memid: MEMBERID,
    invkind: INVOKEKIND,
    dll_name: *mut BSTR,
    name: *mut BSTR,
    ordinal: *mut WORD,
) -> HRESULT {

    let mut flags: DWORD = 0;
    let mut dummy_dll_name: BSTR = core::ptr::null_mut();
    let mut dummy_name: BSTR = core::ptr::null_mut();
    let mut dummy_ordinal: WORD = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%p, %08x, %x, %p, %p, %p)\n\0".as_ptr(),
        This,
        memid,
        invkind,
        dll_name,
        name,
        ordinal,
    );

    let dll_name = if dll_name.is_null() {
        &mut dummy_dll_name as *mut BSTR
    } else {
        flags = 1;
        dll_name
    };

    let name = if name.is_null() {
        &mut dummy_name as *mut BSTR
    } else {
        flags |= 2;
        name
    };

    let ordinal = if ordinal.is_null() {
        &mut dummy_ordinal as *mut WORD
    } else {
        flags |= 4;
        ordinal
    };

    hr = ITypeInfo_RemoteGetDllEntry_Proxy(This, memid, invkind, flags, dll_name, name, ordinal);

    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetDllEntry_Stub(
    This: *mut ITypeInfo,
    memid: MEMBERID,
    invkind: INVOKEKIND,
    flags: DWORD,
    dll_name: *mut BSTR,
    name: *mut BSTR,
    ordinal: *mut WORD
) -> HRESULT {
    TRACE!(
        b"(%p, %08x, %x, %p, %p, %p)\n\0".as_ptr() as PCHAR,
        This,
        memid,
        invkind,
        dll_name,
        name,
        ordinal
    );

    *dll_name = core::ptr::null_mut();
    *name = core::ptr::null_mut();
    *ordinal = 0;

    let dll_name_ptr = if (flags & 1) == 0 { core::ptr::null_mut() } else { dll_name };
    let name_ptr = if (flags & 2) == 0 { core::ptr::null_mut() } else { name };
    let ordinal_ptr = if (flags & 4) == 0 { core::ptr::null_mut() } else { ordinal };

    ITypeInfo_GetDllEntry(This, memid, invkind, dll_name_ptr, name_ptr, ordinal_ptr)
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_AddressOfMember_Proxy(
    _This: *mut ITypeInfo,
    _memid: MEMBERID,
    _invKind: INVOKEKIND,
    _ppv: *mut PVOID,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as *const i8);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_AddressOfMember_Stub(
    _This: *mut ITypeInfo
) -> HRESULT {
    FIXME!("not implemented\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_CreateInstance_Proxy(
    _This: *mut ITypeInfo,
    _pUnkOuter: *mut IUnknown,
    _riid: *const IID,
    _ppvObj: *mut LPVOID,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr());
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_CreateInstance_Stub(
    _This: *mut ITypeInfo,
    _riid: *const GUID,
    _ppvObj: *mut *mut IUnknown,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as *const i8);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetContainingTypeLib_Proxy(
    This: *mut ITypeInfo,
    ppTLib: *mut *mut ITypeLib,
    pIndex: *mut UINT,
) -> HRESULT {
    let mut pTL: *mut ITypeLib = core::ptr::null_mut();
    let mut index: UINT = 0;
    let hr: HRESULT;

    TRACE!(b"(%p, %p, %p)\n\0".as_ptr() as *const i8, This, ppTLib, pIndex);

    hr = ITypeInfo_RemoteGetContainingTypeLib_Proxy(This, &mut pTL, &mut index);
    if SUCCEEDED(hr) {
        if !pIndex.is_null() {
            *pIndex = index;
        }

        if !ppTLib.is_null() {
            *ppTLib = pTL;
        } else {
            ITypeLib_Release(pTL);
        }
    }
    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_GetContainingTypeLib_Stub(
    This: *mut ITypeInfo,
    ppTLib: *mut *mut ITypeLib,
    pIndex: *mut UINT,
) -> HRESULT {
    TRACE!(b"(%p, %p, %p)\n\0".as_ptr() as PCHAR, This, ppTLib, pIndex);
    ITypeInfo_GetContainingTypeLib(This, ppTLib, pIndex)
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_ReleaseTypeAttr_Proxy(
    _This: *mut ITypeInfo,
    pTypeAttr: *mut TYPEATTR,
) {
    
    TRACE!(b"(%p, %p)\n\0".as_ptr(), This, pTypeAttr);
    free_embedded_typedesc(core::ptr::addr_of_mut!((*pTypeAttr).tdescAlias));
    CoTaskMemFree(pTypeAttr as LPVOID);
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_ReleaseTypeAttr_Stub(
    _This: *mut ITypeInfo,
) -> HRESULT {
    TRACE!(b"nothing to do\n\0".as_ptr() as *const i8);
    0
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_ReleaseFuncDesc_Proxy(
    _This: *mut ITypeInfo,
    pFuncDesc: *mut FUNCDESC) {
    let mut param: SHORT;
    TRACE!(b"(%p, %p)\n\0".as_ptr() as PCHAR, This, pFuncDesc);

    param = 0;
    while param < (*pFuncDesc).cParams {
        free_embedded_elemdesc((*pFuncDesc).lprgelemdescParam.offset(param as isize));
        param += 1;
    }
    if param != 0 {
        CoTaskMemFree((*pFuncDesc).lprgelemdescParam as LPVOID);
    }

    free_embedded_elemdesc(core::ptr::addr_of_mut!((*pFuncDesc).elemdescFunc));

    if (*pFuncDesc).cScodes != 0 && (*pFuncDesc).cScodes != -1 {
        CoTaskMemFree((*pFuncDesc).lprgscode as LPVOID);
    }

    CoTaskMemFree(pFuncDesc as LPVOID);
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_ReleaseFuncDesc_Stub(
    _This: *mut ITypeInfo
) -> HRESULT {
    TRACE!(b"nothing to do\n\0".as_ptr() as PCHAR);
    S_OK
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_ReleaseVarDesc_Proxy(
    _This: *mut ITypeInfo,
    pVarDesc: *mut VARDESC,
) {
    TRACE!(b"(%p, %p)\n\0".as_ptr() as PCHAR, This, pVarDesc);

    CoTaskMemFree((*pVarDesc).lpstrSchema as LPVOID);

    if (*pVarDesc).varkind == VAR_CONST {
        CoTaskMemFree((*pVarDesc).__bindgen_anon_1.lpvarValue as LPVOID);
    }

    free_embedded_elemdesc(core::ptr::addr_of_mut!((*pVarDesc).elemdescVar));
    CoTaskMemFree(pVarDesc as LPVOID);
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo_ReleaseVarDesc_Stub(
    _This: *mut core::ffi::c_void,
) -> HRESULT {
    TRACE!(b"nothing to do\n\0".as_ptr() as *const i8);
    0
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo2_GetDocumentation2_Proxy(
    This: *mut ITypeInfo2,
    memid: MEMBERID,
    lcid: LCID,
    help_string: *mut BSTR,
    help_context: *mut DWORD,
    help_dll: *mut BSTR,
) -> HRESULT {
    let mut dummy_help_context: DWORD = 0;
    let mut flags: DWORD = 0;
    let mut dummy_help_string: BSTR = core::ptr::null_mut();
    let mut dummy_help_dll: BSTR = core::ptr::null_mut();
    let hr: HRESULT;

    TRACE!(
        b"(%p, %08x, %08x, %p, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        memid,
        lcid,
        help_string,
        help_context,
        help_dll,
    );

    let help_string = if help_string.is_null() {
        &mut dummy_help_string as *mut BSTR
    } else {
        flags = 1;
        help_string
    };

    let help_context = if help_context.is_null() {
        &mut dummy_help_context as *mut DWORD
    } else {
        flags |= 2;
        help_context
    };

    let help_dll = if help_dll.is_null() {
        &mut dummy_help_dll as *mut BSTR
    } else {
        flags |= 4;
        help_dll
    };

    hr = ITypeInfo2_RemoteGetDocumentation2_Proxy(This, memid, lcid, flags, help_string, help_context, help_dll);

    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeInfo2_GetDocumentation2_Stub(
    This: *mut ITypeInfo2,
    memid: MEMBERID,
    lcid: LCID,
    flags: DWORD,
    help_string: *mut BSTR,
    help_context: *mut DWORD,
    help_dll: *mut BSTR
) -> HRESULT {
    
    TRACE!(b"(%p, %08x, %08x, %08x, %p, %p, %p)\n\0".as_ptr(), This, memid, lcid, flags, help_string, help_context, help_dll);
    
    *help_string = core::ptr::null_mut();
    *help_dll = core::ptr::null_mut();
    *help_context = 0;
    
    let help_string = if flags & 1 == 0 { core::ptr::null_mut() } else { help_string };
    let help_context = if flags & 2 == 0 { core::ptr::null_mut() } else { help_context };
    let help_dll = if flags & 4 == 0 { core::ptr::null_mut() } else { help_dll };
    
    ITypeInfo2_GetDocumentation2(This, memid, lcid, help_string, help_context, help_dll)
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_GetTypeInfoCount_Proxy(
    This: *mut ITypeLib
) -> UINT {
    let mut count: UINT = 0;
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);
    
    ITypeLib_RemoteGetTypeInfoCount_Proxy(This, &mut count);
    
    count
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_GetTypeInfoCount_Stub(
    This: *mut ITypeLib,
    pcTInfo: *mut UINT
) -> HRESULT {
    
    TRACE!(b"(%p, %p)\n\0".as_ptr() as PCHAR, This, pcTInfo);
    *pcTInfo = ITypeLib_GetTypeInfoCount(This);
    S_OK
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_GetLibAttr_Proxy(
    This: *mut ITypeLib,
    ppTLibAttr: *mut *mut TLIBATTR,
) -> HRESULT {

    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const i8, This, ppTLibAttr);

    let mut stg = core::mem::zeroed::<CLEANLOCALSTORAGE>();
    stg.flags = 0;
    stg.pStorage = core::ptr::null_mut();
    stg.pInterface = core::ptr::null_mut();

    ITypeLib_RemoteGetLibAttr_Proxy(This, ppTLibAttr, &mut stg)
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_GetLibAttr_Stub(
    This: *mut ITypeLib,
    ppTLibAttr: *mut LPTLIBATTR,
    pDummy: *mut CLEANLOCALSTORAGE,
) -> HRESULT {
    let hr: HRESULT;
    TRACE!(b"(%p, %p)\n\0".as_ptr() as PCHAR, This, ppTLibAttr);
    
    hr = ITypeLib_GetLibAttr(This, ppTLibAttr);
    if hr != S_OK {
        return hr;
    }

    (*pDummy).flags = CLS_LIBATTR;
    ITypeLib_AddRef(This);
    (*pDummy).pInterface = This as *mut IUnknown;
    (*pDummy).pStorage = ppTLibAttr as LPVOID;
    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_GetDocumentation_Proxy(
    This: *mut ITypeLib,
    index: INT,
    name: *mut BSTR,
    doc_string: *mut BSTR,
    help_context: *mut DWORD,
    help_file: *mut BSTR,
) -> HRESULT {
    let mut dummy_help_context: DWORD = 0;
    let mut flags: DWORD = 0;
    let mut dummy_name: BSTR = core::ptr::null_mut();
    let mut dummy_doc_string: BSTR = core::ptr::null_mut();
    let mut dummy_help_file: BSTR = core::ptr::null_mut();
    let hr: HRESULT;

    TRACE!(
        b"(%p, %d, %p, %p, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        index,
        name,
        doc_string,
        help_context,
        help_file,
    );

    let name_ptr = if name.is_null() {
        core::ptr::addr_of_mut!(dummy_name)
    } else {
        flags = 1;
        name
    };

    let doc_string_ptr = if doc_string.is_null() {
        core::ptr::addr_of_mut!(dummy_doc_string)
    } else {
        flags |= 2;
        doc_string
    };

    let help_context_ptr = if help_context.is_null() {
        core::ptr::addr_of_mut!(dummy_help_context)
    } else {
        flags |= 4;
        help_context
    };

    let help_file_ptr = if help_file.is_null() {
        core::ptr::addr_of_mut!(dummy_help_file)
    } else {
        flags |= 8;
        help_file
    };

    hr = ITypeLib_RemoteGetDocumentation_Proxy(
        This,
        index,
        flags,
        name_ptr,
        doc_string_ptr,
        help_context_ptr,
        help_file_ptr,
    );

    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_GetDocumentation_Stub(
    This: *mut ITypeLib,
    index: INT,
    flags: DWORD,
    name: *mut BSTR,
    doc_string: *mut BSTR,
    help_context: *mut DWORD,
    help_file: *mut BSTR,
) -> HRESULT {

    *name = core::ptr::null_mut();
    *doc_string = core::ptr::null_mut();
    *help_file = core::ptr::null_mut();
    *help_context = 0;

    let name_ptr = if (flags & 1) != 0 { name } else { core::ptr::null_mut() };
    let doc_string_ptr = if (flags & 2) != 0 { doc_string } else { core::ptr::null_mut() };
    let help_context_ptr = if (flags & 4) != 0 { help_context } else { core::ptr::null_mut() };
    let help_file_ptr = if (flags & 8) != 0 { help_file } else { core::ptr::null_mut() };

    ITypeLib_GetDocumentation(This, index, name_ptr, doc_string_ptr, help_context_ptr, help_file_ptr)
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_IsName_Proxy(
    _This: *mut ITypeLib,
    _szNameBuf: LPOLESTR,
    _lHashVal: ULONG,
    _pfName: *mut BOOL,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as *const i8);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_IsName_Stub(
    _This: *mut ITypeLib,
    _szNameBuf: LPOLESTR,
    _lHashVal: ULONG,
    _pfName: *mut BOOL,
    _pBstrLibName: *mut BSTR,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as PCHAR);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_FindName_Proxy(
    _This: *mut ITypeLib,
    _szNameBuf: LPOLESTR,
    _lHashVal: ULONG,
    _ppTInfo: *mut *mut ITypeInfo,
    _rgMemId: *mut MEMBERID,
    _pcFound: *mut USHORT,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as *const i8);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_FindName_Stub(
    _This: *mut ITypeLib,
    _szNameBuf: LPOLESTR,
    _lHashVal: ULONG,
    _ppTInfo: *mut *mut ITypeInfo,
    _rgMemId: *mut MEMBERID,
    _pcFound: *mut USHORT,
    _pBstrLibName: *mut BSTR,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as PCHAR);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_ReleaseTLibAttr_Proxy(
    _This: *mut ITypeLib,
    pTLibAttr: *mut TLIBATTR,
) {
    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const i8, This, pTLibAttr);
    CoTaskMemFree(pTLibAttr as LPVOID);
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib_ReleaseTLibAttr_Stub(
    _This: *mut ITypeLib
) -> HRESULT {
    TRACE!(b"nothing to do\n\0".as_ptr() as *const i8);
    return 0; // S_OK
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib2_GetLibStatistics_Proxy(
    _This: *mut ITypeLib2,
    _pcUniqueNames: *mut ULONG,
    _pcchUniqueNames: *mut ULONG,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as *const i8);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib2_GetLibStatistics_Stub(
    _This: *mut ITypeLib2,
    _pcUniqueNames: *mut ULONG,
    _pcchUniqueNames: *mut ULONG,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as PCHAR);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib2_GetDocumentation2_Proxy(
    This: *mut ITypeLib2,
    index: INT,
    lcid: LCID,
    help_string: *mut BSTR,
    help_context: *mut DWORD,
    help_dll: *mut BSTR,
) -> HRESULT {
    let mut dummy_help_context: DWORD = 0;
    let mut flags: DWORD = 0;
    let mut dummy_help_string: BSTR = core::ptr::null_mut();
    let mut dummy_help_dll: BSTR = core::ptr::null_mut();
    let hr: HRESULT;
    
    TRACE!(
        b"(%p, %d, %08x, %p, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        index,
        lcid,
        help_string,
        help_context,
        help_dll,
    );

    let help_string = if help_string.is_null() {
        &mut dummy_help_string as *mut BSTR
    } else {
        flags = 1;
        help_string
    };

    let help_context = if help_context.is_null() {
        &mut dummy_help_context as *mut DWORD
    } else {
        flags |= 2;
        help_context
    };

    let help_dll = if help_dll.is_null() {
        &mut dummy_help_dll as *mut BSTR
    } else {
        flags |= 4;
        help_dll
    };

    hr = ITypeLib2_RemoteGetDocumentation2_Proxy(
        This,
        index,
        lcid,
        flags,
        help_string,
        help_context,
        help_dll,
    );

    hr
}
#[no_mangle]
pub unsafe extern "system" fn ITypeLib2_GetDocumentation2_Stub(
    This: *mut ITypeLib2,
    index: INT,
    lcid: LCID,
    flags: DWORD,
    help_string: *mut BSTR,
    help_context: *mut DWORD,
    help_dll: *mut BSTR,
) -> HRESULT {

    TRACE!(
        b"(%p, %d, %08x, %08x, %p, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        index,
        lcid,
        flags,
        help_string,
        help_context,
        help_dll,
    );

    *help_string = core::ptr::null_mut();
    *help_dll = core::ptr::null_mut();
    *help_context = 0;

    let help_string_arg = if (flags & 1) != 0 { help_string } else { core::ptr::null_mut() };
    let help_context_arg = if (flags & 2) != 0 { help_context } else { core::ptr::null_mut() };
    let help_dll_arg = if (flags & 4) != 0 { help_dll } else { core::ptr::null_mut() };

    ITypeLib2_GetDocumentation2(This, index, lcid, help_string_arg, help_context_arg, help_dll_arg)
}
#[no_mangle]
pub unsafe extern "system" fn IPropertyBag_Read_Proxy(
    This: *mut IPropertyBag,
    pszPropName: LPCOLESTR,
    pVar: *mut VARIANT,
    pErrorLog: *mut IErrorLog,
) -> HRESULT {

    let mut pUnk: *mut IUnknown = core::ptr::null_mut();
    TRACE!(
        b"(%p, %s, %p, %p)\n\0".as_ptr() as PCHAR,
        This,
        debugstr_w(pszPropName),
        pVar,
        pErrorLog,
    );

    if pVar.is_null() {
        return E_POINTER;
    }

    if (*V_VT_ref(pVar)) & (VT_BYREF | VT_ARRAY | VT_VECTOR) != 0 {
        FIXME!(
            b"Variant type %x is byref, array or vector. Not implemented.\n\0".as_ptr() as PCHAR,
            (*V_VT_ref(pVar)),
        );
        return E_NOTIMPL;
    }

    match *V_VT_ref(pVar)  {
        VT_DISPATCH => {
            pUnk = (*V_DISPATCH_ref(pVar)) as *mut IUnknown;
        }
        VT_UNKNOWN => {
            pUnk = *V_UNKNOWN_ref(pVar) ;
        }
        VT_SAFEARRAY => {
            FIXME!(b"Safearray support not yet implemented.\n\0".as_ptr() as PCHAR);
            return E_NOTIMPL;
        }
        _ => {
            FIXME!(
                b"Unknown V_VT %d - support not yet implemented.\n\0".as_ptr() as PCHAR,
                (*V_VT_ref(pVar)),
            );
            return E_NOTIMPL;
        }
    }

    return IPropertyBag_RemoteRead_Proxy(This, pszPropName, pVar, pErrorLog, (*V_VT_ref(pVar)) as DWORD, pUnk);
}
#[no_mangle]
pub unsafe extern "system" fn IPropertyBag_Read_Stub(
    This: *mut IPropertyBag,
    pszPropName: LPCOLESTR,
    pVar: *mut VARIANT,
    pErrorLog: *mut IErrorLog,
    varType: DWORD,
    pUnkObj: *mut IUnknown,
) -> HRESULT {
    static emptyWstr: [u16; 1] = [0];
    let mut disp: *mut IDispatch = core::ptr::null_mut();
    let mut hr: HRESULT;
    
    TRACE!(
        b"(%p, %s, %p, %p, %x, %p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_w(pszPropName),
        pVar,
        pErrorLog,
        varType,
        pUnkObj,
    );

    if (varType & (VT_BYREF | VT_ARRAY | VT_VECTOR) as DWORD) != 0 {
        FIXME!(
            b"Variant type %x is byref, array or vector. Not implemented.\n\0".as_ptr() as *const i8,
            (*V_VT_ref(pVar)),
        );
        return E_NOTIMPL;
    }

    (*V_VT_ref(pVar)) = varType as u16;

    match varType {
        _ if varType == VT_DISPATCH as DWORD => {
            hr = IUnknown_QueryInterface(
                pUnkObj,
                &IID_IDispatch,
                &mut disp as *mut *mut IDispatch as *mut LPVOID,
            );
            if FAILED(hr) {
                return hr;
            }
            IUnknown_Release(pUnkObj);
            (*V_DISPATCH_ref(pVar)) = disp;
        }
        _ if varType == VT_UNKNOWN as DWORD => {
            (*V_UNKNOWN_ref(pVar)) = pUnkObj;
        }
        _ if varType == VT_BSTR as DWORD => {
            (*V_BSTR_ref(pVar)) = SysAllocString(emptyWstr.as_ptr());
        }
        _ if varType == VT_SAFEARRAY as DWORD => {
            FIXME!(b"Safearray support not yet implemented.\n\0".as_ptr() as *const i8);
            return E_NOTIMPL;
        }
        _ => {}
    }
    
    hr = IPropertyBag_Read(This, pszPropName, pVar, pErrorLog);
    if FAILED(hr) {
        VariantClear(pVar);
    }

    hr
}
#[no_mangle]
pub unsafe extern "system" fn IClassFactory2_CreateInstanceLic_Proxy(
    This: *mut IClassFactory2,
    pUnkOuter: *mut IUnknown,
    _pUnkReserved: *mut IUnknown,
    riid: REFIID,
    bstrKey: BSTR,
    ppvObj: *mut PVOID,
) -> HRESULT {

    TRACE!(
        b"(%p, %s, %p)\n\0".as_ptr(),
        pUnkOuter,
        debugstr_guid(riid),
        ppvObj,
    );

    *ppvObj = core::ptr::null_mut();

    if !pUnkOuter.is_null() {
        ERR!(b"aggregation is not allowed on remote objects\n\0".as_ptr());
        return CLASS_E_NOAGGREGATION;
    }

    IClassFactory2_RemoteCreateInstanceLic_Proxy(This, riid, bstrKey, ppvObj as *mut *mut IUnknown)
}
#[no_mangle]
pub unsafe extern "system" fn IClassFactory2_CreateInstanceLic_Stub(
    This: *mut IClassFactory2,
    riid: REFIID,
    bstrKey: BSTR,
    ppvObj: *mut *mut IUnknown,
) -> HRESULT {
    
    TRACE!(b"(%s, %p)\n\0".as_ptr() as *const i8, debugstr_guid(riid), ppvObj);
    IClassFactory2_CreateInstanceLic(
        This,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        riid,
        bstrKey,
        ppvObj as *mut *mut core::ffi::c_void,
    )
}
#[no_mangle]
pub unsafe extern "system" fn IEnumConnections_Next_Proxy(
    This: *mut IEnumConnections,
    cConnections: ULONG,
    rgcd: LPCONNECTDATA,
    pcFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;

    TRACE!(b"(%u, %p %p)\n\0".as_ptr() as *const i8, cConnections, rgcd, pcFetched);

    let pcFetched = if pcFetched.is_null() {
        core::ptr::addr_of_mut!(fetched)
    } else {
        pcFetched
    };

    IEnumConnections_RemoteNext_Proxy(This, cConnections, rgcd, pcFetched)
}
#[no_mangle]
pub unsafe extern "system" fn IEnumConnections_Next_Stub(
    This: *mut IEnumConnections,
    cConnections: ULONG,
    rgcd: LPCONNECTDATA,
    pcFetched: *mut ULONG,
) -> HRESULT {
    let hr: HRESULT;

    TRACE!(b"(%u, %p, %p)\n\0".as_ptr() as PCHAR, cConnections, rgcd, pcFetched);

    *pcFetched = 0;
    hr = IEnumConnections_Next(This, cConnections, rgcd, pcFetched);
    if hr == S_OK {
        *pcFetched = cConnections;
    }

    hr
}
#[no_mangle]
pub unsafe extern "system" fn IEnumConnectionPoints_Next_Proxy(
    This: *mut IEnumConnectionPoints,
    cConnections: ULONG,
    ppCP: *mut *mut IConnectionPoint,
    pcFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;

    TRACE!(
        b"(%u, %p %p)\n\0".as_ptr() as *const i8,
        cConnections,
        ppCP,
        pcFetched,
    );

    let pcFetched = if pcFetched.is_null() {
        &mut fetched as *mut ULONG
    } else {
        pcFetched
    };

    IEnumConnectionPoints_RemoteNext_Proxy(This, cConnections, ppCP, pcFetched)
}
#[no_mangle]
pub unsafe extern "system" fn IEnumConnectionPoints_Next_Stub(
    This: *mut IEnumConnectionPoints,
    cConnections: ULONG,
    ppCP: *mut *mut IConnectionPoint,
    pcFetched: *mut ULONG,
) -> HRESULT {

    let hr: HRESULT;

    TRACE!(
        b"(%u, %p, %p)\n\0".as_ptr() as *const i8,
        cConnections,
        ppCP,
        pcFetched,
    );

    *pcFetched = 0;
    hr = IEnumConnectionPoints_Next(This, cConnections, ppCP, pcFetched);
    if hr == S_OK {
        *pcFetched = cConnections;
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn IPersistMemory_Load_Proxy(
    This: *mut IPersistMemory,
    pMem: LPVOID,
    cbSize: ULONG,
) -> HRESULT {

    TRACE!(b"(%p, %u)\n\0".as_ptr() as *const i8, pMem, cbSize);

    if pMem.is_null() {
        return E_POINTER;
    }

    IPersistMemory_RemoteLoad_Proxy(This, pMem, cbSize)
}
#[no_mangle]
pub unsafe extern "system" fn IPersistMemory_Load_Stub(
    This: *mut IPersistMemory,
    pMem: *mut BYTE,
    cbSize: ULONG,
) -> HRESULT {

    TRACE!(b"(%p, %u)\n\0".as_ptr(), pMem, cbSize);
    IPersistMemory_Load(This, pMem as *mut core::ffi::c_void, cbSize)
}
#[no_mangle]
pub unsafe extern "system" fn IPersistMemory_Save_Proxy(
    This: *mut IPersistMemory,
    pMem: LPVOID,
    fClearDirty: BOOL,
    cbSize: ULONG,
) -> HRESULT {

    TRACE!(b"(%p, %d, %u)\n\0".as_ptr(), pMem, fClearDirty, cbSize);

    if pMem.is_null() {
        return E_POINTER;
    }

    IPersistMemory_RemoteSave_Proxy(This, pMem, fClearDirty, cbSize)
}
#[no_mangle]
pub unsafe extern "system" fn IPersistMemory_Save_Stub(
    This: *mut IPersistMemory,
    pMem: *mut BYTE,
    fClearDirty: BOOL,
    cbSize: ULONG,
) -> HRESULT {
    
    TRACE!(b"(%p, %d, %u)\n\0".as_ptr() as PCSTR, pMem, fClearDirty, cbSize);
    IPersistMemory_Save(This, pMem as LPVOID, fClearDirty, cbSize)
}
#[no_mangle]
pub unsafe extern "system" fn IAdviseSinkEx_OnViewStatusChange_Proxy(
    This: *mut IAdviseSinkEx,
    dwViewStatus: DWORD,
) {
    TRACE!(b"(%p, 0x%08x)\n\0".as_ptr() as *const i8, This, dwViewStatus);
    IAdviseSinkEx_RemoteOnViewStatusChange_Proxy(This, dwViewStatus);
}
#[no_mangle]
pub unsafe extern "system" fn IAdviseSinkEx_OnViewStatusChange_Stub(
    This: *mut IAdviseSinkEx,
    dwViewStatus: DWORD,
) -> HRESULT {
    
    TRACE!(b"(%p, 0x%08x)\n\0".as_ptr(), This, dwViewStatus);
    IAdviseSinkEx_OnViewStatusChange(This, dwViewStatus);
    S_OK
}
#[no_mangle]
pub unsafe extern "system" fn IEnumOleUndoUnits_Next_Proxy(
    This: *mut IEnumOleUndoUnits,
    cElt: ULONG,
    rgElt: *mut *mut IOleUndoUnit,
    pcEltFetched: *mut ULONG,
) -> HRESULT {

    let mut fetched: ULONG = 0;
    
    TRACE!(
        b"(%u, %p %p)\n\0".as_ptr() as *const i8,
        cElt,
        rgElt,
        pcEltFetched,
    );

    let pcEltFetched = if pcEltFetched.is_null() {
        &mut fetched as *mut ULONG
    } else {
        pcEltFetched
    };

    IEnumOleUndoUnits_RemoteNext_Proxy(This, cElt, rgElt, pcEltFetched)
}
#[no_mangle]
pub unsafe extern "system" fn IEnumOleUndoUnits_Next_Stub(
    This: *mut IEnumOleUndoUnits,
    cElt: ULONG,
    rgElt: *mut *mut IOleUndoUnit,
    pcEltFetched: *mut ULONG
) -> HRESULT {

    let hr: HRESULT;

    TRACE!(b"(%u, %p, %p)\n\0".as_ptr(), cElt, rgElt, pcEltFetched);

    *pcEltFetched = 0;
    hr = IEnumOleUndoUnits_Next(This, cElt, rgElt, pcEltFetched);
    if hr == S_OK {
        *pcEltFetched = cElt;
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn IQuickActivate_QuickActivate_Proxy(
    _This: *mut IQuickActivate,
    _pQaContainer: *mut QACONTAINER,
    _pQaControl: *mut QACONTROL,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "system" fn IQuickActivate_QuickActivate_Stub(
    _This: *mut IQuickActivate,
    _pQaContainer: *mut QACONTAINER,
    _pQaControl: *mut QACONTROL,
) -> HRESULT {
    FIXME!(b"not implemented\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
