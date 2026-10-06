//! Base Win32/COM/OA declarations for the oleaut32 Lane-A translations
//! (dispatch.rs, typelib.rs). HAND-DECLARED, kept in this ONE module so the
//! swap to the future bindgen `com-sys` crate is mechanical.
//!
//! Self-contained (no dependency on the ole32 lane modules): at integration
//! these decls dedupe into com-sys. Slot orders verified against
//! sdk/include/psdk/{oaidl,unknwn}.idl — NEVER reorder.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use core::ffi::c_void;

// ---------------------------------------------------------------------------
// Base typedefs
// ---------------------------------------------------------------------------

pub type BOOL = i32;
pub type BYTE = u8;
pub type WORD = u16;
pub type USHORT = u16;
pub type SHORT = i16;
pub type DWORD = u32;
pub type LONG = i32;
pub type ULONG = u32;
pub type UINT = u32;
pub type INT = i32;
pub type WCHAR = u16;
pub type CHAR = i8;
pub type HRESULT = i32;
pub type SCODE = i32;
pub type LCID = u32;
pub type LANGID = u16;
pub type SYSKIND = i32;
pub type ULONG_PTR = usize;
pub type LONG_PTR = isize;
pub type SIZE_T = usize;
pub type HANDLE = *mut c_void;
pub type HKEY = *mut c_void;
pub type HMODULE = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type LPWSTR = *mut u16;
pub type LPCWSTR = *const u16;
pub type LPOLESTR = *mut u16;
pub type LPCOLESTR = *const u16;
pub type OLECHAR = u16;
pub type LPSTR = *mut i8;
pub type LPCSTR = *const i8;
pub type PVOID = *mut c_void;
pub type LPVOID = *mut c_void;
pub type BSTR = *mut u16;
pub type DISPID = i32;
pub type MEMBERID = DISPID;
pub type HREFTYPE = DWORD;
pub type VARTYPE = u16;
pub type TYPEKIND = i32;
pub type CALLCONV = i32;
pub type FUNCKIND = i32;
pub type INVOKEKIND = i32;
pub type VARKIND = i32;
pub type DESCKIND = i32;
pub type REGKIND = i32;
pub type DOUBLE = f64;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
pub const MAX_PATH: usize = 260;

// ---------------------------------------------------------------------------
// GUID / IID / CLSID
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}
pub type IID = GUID;
pub type CLSID = GUID;
pub type TLBID = GUID;
pub type REFIID = *const IID;
pub type REFCLSID = *const CLSID;
pub type REFGUID = *const GUID;

#[inline]
pub unsafe fn guid_eq(a: *const GUID, b: *const GUID) -> bool {
    let a = &*(a as *const [u8; 16]);
    let b = &*(b as *const [u8; 16]);
    a == b
}

macro_rules! def_guid {
    ($name:ident, $d1:expr, $d2:expr, $d3:expr, $($d4:expr),+) => {
        pub static $name: GUID = GUID {
            Data1: $d1, Data2: $d2, Data3: $d3, Data4: [$($d4),+],
        };
    };
}

def_guid!(GUID_NULL,          0x00000000, 0x0000, 0x0000, 0,0,0,0,0,0,0,0);
pub static IID_NULL: GUID = GUID_NULL;
def_guid!(IID_IUnknown,       0x00000000, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IDispatch,      0x00020400, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_ITypeLib,       0x00020402, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_ITypeLib2,      0x00020411, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_ITypeInfo,      0x00020401, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_ITypeInfo2,     0x00020412, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_ITypeComp,      0x00020403, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IEnumVARIANT,   0x00020404, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);

/// `debugstr_guid` equivalent for ERR!/FIXME!.
pub struct DbgGuid(pub *const GUID);
impl core::fmt::Display for DbgGuid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0.is_null() {
            return f.write_str("(null)");
        }
        let g = unsafe { &*self.0 };
        write!(
            f,
            "{{{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}}}",
            g.Data1, g.Data2, g.Data3, g.Data4[0], g.Data4[1], g.Data4[2], g.Data4[3],
            g.Data4[4], g.Data4[5], g.Data4[6], g.Data4[7]
        )
    }
}

/// `debugstr_w` equivalent (lossy ASCII, capped).
pub struct DbgW(pub *const u16);
impl core::fmt::Display for DbgW {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0.is_null() {
            return f.write_str("(null)");
        }
        f.write_str("\"")?;
        let mut i = 0usize;
        loop {
            let c = unsafe { *self.0.add(i) };
            if c == 0 || i >= 200 {
                break;
            }
            let ch = if (0x20..0x7f).contains(&c) { c as u8 as char } else { '?' };
            core::fmt::Write::write_char(f, ch)?;
            i += 1;
        }
        f.write_str("\"")
    }
}

// ---------------------------------------------------------------------------
// HRESULTs (winerror.h)
// ---------------------------------------------------------------------------

pub const S_OK: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const E_NOTIMPL: HRESULT = 0x8000_4001u32 as HRESULT;
pub const E_NOINTERFACE: HRESULT = 0x8000_4002u32 as HRESULT;
pub const E_POINTER: HRESULT = 0x8000_4003u32 as HRESULT;
pub const E_FAIL: HRESULT = 0x8000_4005u32 as HRESULT;
pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;
pub const E_ACCESSDENIED: HRESULT = 0x8007_0005u32 as HRESULT;
pub const E_OUTOFMEMORY: HRESULT = 0x8007_000Eu32 as HRESULT;
pub const E_INVALIDARG: HRESULT = 0x8007_0057u32 as HRESULT;

pub const DISP_E_UNKNOWNINTERFACE: HRESULT = 0x8002_0001u32 as HRESULT;
pub const DISP_E_MEMBERNOTFOUND: HRESULT = 0x8002_0003u32 as HRESULT;
pub const DISP_E_PARAMNOTFOUND: HRESULT = 0x8002_0004u32 as HRESULT;
pub const DISP_E_TYPEMISMATCH: HRESULT = 0x8002_0005u32 as HRESULT;
pub const DISP_E_UNKNOWNNAME: HRESULT = 0x8002_0006u32 as HRESULT;
pub const DISP_E_NONAMEDARGS: HRESULT = 0x8002_0007u32 as HRESULT;
pub const DISP_E_BADVARTYPE: HRESULT = 0x8002_0008u32 as HRESULT;
pub const DISP_E_EXCEPTION: HRESULT = 0x8002_0009u32 as HRESULT;
pub const DISP_E_OVERFLOW: HRESULT = 0x8002_000Au32 as HRESULT;
pub const DISP_E_BADINDEX: HRESULT = 0x8002_000Bu32 as HRESULT;
pub const DISP_E_UNKNOWNLCID: HRESULT = 0x8002_000Cu32 as HRESULT;
pub const DISP_E_ARRAYISLOCKED: HRESULT = 0x8002_000Du32 as HRESULT;
pub const DISP_E_BADPARAMCOUNT: HRESULT = 0x8002_000Eu32 as HRESULT;
pub const DISP_E_PARAMNOTOPTIONAL: HRESULT = 0x8002_000Fu32 as HRESULT;

pub const TYPE_E_IOERROR: HRESULT = 0x8002_8CA2u32 as HRESULT;
pub const TYPE_E_CANTLOADLIBRARY: HRESULT = 0x8002_9C4Au32 as HRESULT;
pub const TYPE_E_LIBNOTREGISTERED: HRESULT = 0x8002_801Du32 as HRESULT;
pub const TYPE_E_REGISTRYACCESS: HRESULT = 0x8002_801Cu32 as HRESULT;
pub const TYPE_E_INVALIDSTATE: HRESULT = 0x8002_8029u32 as HRESULT;
pub const TYPE_E_ELEMENTNOTFOUND: HRESULT = 0x8002_802Bu32 as HRESULT;
pub const TYPE_E_TYPEMISMATCH: HRESULT = 0x8002_8CA0u32 as HRESULT;
pub const TYPE_E_OUTOFBOUNDS: HRESULT = 0x8002_8CA1u32 as HRESULT;
pub const TYPE_E_INVDATAREAD: HRESULT = 0x8002_8018u32 as HRESULT;
pub const TYPE_E_UNSUPFORMAT: HRESULT = 0x8002_8019u32 as HRESULT;
pub const TYPE_E_BADMODULEKIND: HRESULT = 0x8002_88BDu32 as HRESULT;
pub const TYPE_E_WRONGTYPEKIND: HRESULT = 0x8002_802Au32 as HRESULT;
pub const TYPE_E_INVALIDID: HRESULT = 0x8002_88CFu32 as HRESULT;
pub const TYPE_E_AMBIGUOUSNAME: HRESULT = 0x8002_802Cu32 as HRESULT;
pub const TYPE_E_DUPLICATEID: HRESULT = 0x8002_88C6u32 as HRESULT;

#[inline]
pub fn SUCCEEDED(hr: HRESULT) -> bool { hr >= 0 }
#[inline]
pub fn FAILED(hr: HRESULT) -> bool { hr < 0 }

// ---------------------------------------------------------------------------
// VARENUM / VARIANT / DISPPARAMS / EXCEPINFO (oaidl.idl / wtypes.idl)
// ---------------------------------------------------------------------------

pub const VT_EMPTY: VARTYPE = 0;
pub const VT_NULL: VARTYPE = 1;
pub const VT_I2: VARTYPE = 2;
pub const VT_I4: VARTYPE = 3;
pub const VT_R4: VARTYPE = 4;
pub const VT_R8: VARTYPE = 5;
pub const VT_CY: VARTYPE = 6;
pub const VT_DATE: VARTYPE = 7;
pub const VT_BSTR: VARTYPE = 8;
pub const VT_DISPATCH: VARTYPE = 9;
pub const VT_ERROR: VARTYPE = 10;
pub const VT_BOOL: VARTYPE = 11;
pub const VT_VARIANT: VARTYPE = 12;
pub const VT_UNKNOWN: VARTYPE = 13;
pub const VT_DECIMAL: VARTYPE = 14;
pub const VT_I1: VARTYPE = 16;
pub const VT_UI1: VARTYPE = 17;
pub const VT_UI2: VARTYPE = 18;
pub const VT_UI4: VARTYPE = 19;
pub const VT_I8: VARTYPE = 20;
pub const VT_UI8: VARTYPE = 21;
pub const VT_INT: VARTYPE = 22;
pub const VT_UINT: VARTYPE = 23;
pub const VT_VOID: VARTYPE = 24;
pub const VT_HRESULT: VARTYPE = 25;
pub const VT_PTR: VARTYPE = 26;
pub const VT_SAFEARRAY: VARTYPE = 27;
pub const VT_CARRAY: VARTYPE = 28;
pub const VT_USERDEFINED: VARTYPE = 29;
pub const VT_LPSTR: VARTYPE = 30;
pub const VT_LPWSTR: VARTYPE = 31;
pub const VT_RECORD: VARTYPE = 36;
pub const VT_INT_PTR: VARTYPE = 37;
pub const VT_UINT_PTR: VARTYPE = 38;
pub const VT_FILETIME: VARTYPE = 64;
pub const VT_BLOB: VARTYPE = 65;
pub const VT_STREAM: VARTYPE = 66;
pub const VT_STORAGE: VARTYPE = 67;
pub const VT_STREAMED_OBJECT: VARTYPE = 68;
pub const VT_STORED_OBJECT: VARTYPE = 69;
pub const VT_BLOB_OBJECT: VARTYPE = 70;
pub const VT_CF: VARTYPE = 71;
pub const VT_CLSID: VARTYPE = 72;
pub const VT_VECTOR: VARTYPE = 0x1000;
pub const VT_ARRAY: VARTYPE = 0x2000;
pub const VT_BYREF: VARTYPE = 0x4000;
pub const VT_TYPEMASK: VARTYPE = 0x0fff;

/// VARIANT: 16 bytes on 32-bit, 24 on 64-bit (tag + 3 reserved + data union
/// whose largest arm is the {pvRecord,pRecInfo} pair).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VARIANT_RECORD_ARM {
    pub pvRecord: PVOID,
    pub pRecInfo: PVOID,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union VARIANT_DATA {
    pub llVal: i64,
    pub ullVal: u64,
    pub lVal: LONG,
    pub ulVal: ULONG,
    pub iVal: SHORT,
    pub uiVal: USHORT,
    pub bVal: BYTE,
    pub cVal: CHAR,
    pub fltVal: f32,
    pub dblVal: f64,
    pub boolVal: i16, // VARIANT_BOOL
    pub scode: SCODE,
    pub bstrVal: BSTR,
    pub punkVal: *mut IUnknown,
    pub pdispVal: *mut c_void,
    pub byref: PVOID,
    pub record: VARIANT_RECORD_ARM,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VARIANT {
    pub vt: VARTYPE,
    pub wReserved1: WORD,
    pub wReserved2: WORD,
    pub wReserved3: WORD,
    pub n3: VARIANT_DATA,
}
pub type VARIANTARG = VARIANT;
pub type LPVARIANT = *mut VARIANT;
pub type VARIANT_BOOL = i16;
pub const VARIANT_TRUE: VARIANT_BOOL = -1;
pub const VARIANT_FALSE: VARIANT_BOOL = 0;

const _: () = assert!(
    core::mem::size_of::<VARIANT>()
        == if core::mem::size_of::<usize>() == 8 { 24 } else { 16 }
);

#[repr(C)]
pub struct DISPPARAMS {
    pub rgvarg: *mut VARIANTARG,
    pub rgdispidNamedArgs: *mut DISPID,
    pub cArgs: UINT,
    pub cNamedArgs: UINT,
}

#[repr(C)]
pub struct EXCEPINFO {
    pub wCode: WORD,
    pub wReserved: WORD,
    pub bstrSource: BSTR,
    pub bstrDescription: BSTR,
    pub bstrHelpFile: BSTR,
    pub dwHelpContext: DWORD,
    pub pvReserved: PVOID,
    pub pfnDeferredFillIn: Option<unsafe extern "system" fn(*mut EXCEPINFO) -> HRESULT>,
    pub scode: SCODE,
}

// ---------------------------------------------------------------------------
// Type-description structures (oaidl.idl) — exact layouts; typelib.c builds
// and hands these out through ITypeInfo::GetTypeAttr/GetFuncDesc/GetVarDesc.
// ---------------------------------------------------------------------------

/// TYPEDESC: union { TYPEDESC *lptdesc; ARRAYDESC *lpadesc; HREFTYPE hreftype } + VARTYPE vt
#[repr(C)]
#[derive(Clone, Copy)]
pub union TYPEDESC_U {
    pub lptdesc: *mut TYPEDESC,
    pub lpadesc: *mut ARRAYDESC,
    pub hreftype: HREFTYPE,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TYPEDESC {
    pub u: TYPEDESC_U,
    pub vt: VARTYPE,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SAFEARRAYBOUND {
    pub cElements: ULONG,
    pub lLbound: LONG,
}

/// ARRAYDESC — tdescElem + cDims + rgbounds[1] (variable length).
#[repr(C)]
pub struct ARRAYDESC {
    pub tdescElem: TYPEDESC,
    pub cDims: USHORT,
    pub rgbounds: [SAFEARRAYBOUND; 1],
}

/// IDLDESC
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IDLDESC {
    pub dwReserved: ULONG_PTR,
    pub wIDLFlags: USHORT,
}

/// PARAMDESCEX
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PARAMDESCEX {
    pub cBytes: ULONG,
    pub varDefaultValue: VARIANT,
}

/// PARAMDESC
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PARAMDESC {
    pub pparamdescex: *mut PARAMDESCEX,
    pub wParamFlags: USHORT,
}

pub const PARAMFLAG_NONE: USHORT = 0;
pub const PARAMFLAG_FIN: USHORT = 0x1;
pub const PARAMFLAG_FOUT: USHORT = 0x2;
pub const PARAMFLAG_FLCID: USHORT = 0x4;
pub const PARAMFLAG_FRETVAL: USHORT = 0x8;
pub const PARAMFLAG_FOPT: USHORT = 0x10;
pub const PARAMFLAG_FHASDEFAULT: USHORT = 0x20;
pub const PARAMFLAG_FHASCUSTDATA: USHORT = 0x40;

/// ELEMDESC — tdesc + union{idldesc, paramdesc}
#[repr(C)]
#[derive(Clone, Copy)]
pub union ELEMDESC_U {
    pub idldesc: IDLDESC,
    pub paramdesc: PARAMDESC,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ELEMDESC {
    pub tdesc: TYPEDESC,
    pub u: ELEMDESC_U,
}

/// TYPEATTR
#[repr(C)]
pub struct TYPEATTR {
    pub guid: GUID,
    pub lcid: LCID,
    pub dwReserved: DWORD,
    pub memidConstructor: MEMBERID,
    pub memidDestructor: MEMBERID,
    pub lpstrSchema: LPOLESTR,
    pub cbSizeInstance: ULONG,
    pub typekind: TYPEKIND,
    pub cFuncs: WORD,
    pub cVars: WORD,
    pub cImplTypes: WORD,
    pub cbSizeVft: WORD,
    pub cbAlignment: WORD,
    pub wTypeFlags: WORD,
    pub wMajorVerNum: WORD,
    pub wMinorVerNum: WORD,
    pub tdescAlias: TYPEDESC,
    pub idldescType: IDLDESC,
}

/// FUNCDESC
#[repr(C)]
pub struct FUNCDESC {
    pub memid: MEMBERID,
    pub lprgscode: *mut SCODE,
    pub lprgelemdescParam: *mut ELEMDESC,
    pub funckind: FUNCKIND,
    pub invkind: INVOKEKIND,
    pub callconv: CALLCONV,
    pub cParams: SHORT,
    pub cParamsOpt: SHORT,
    pub oVft: SHORT,
    pub cScodes: SHORT,
    pub elemdescFunc: ELEMDESC,
    pub wFuncFlags: WORD,
}

/// VARDESC — memid, lpstrSchema, union{oInst, lpvarValue}, elemdescVar,
/// wVarFlags, varkind
#[repr(C)]
pub union VARDESC_U {
    pub oInst: ULONG,
    pub lpvarValue: *mut VARIANT,
}
#[repr(C)]
pub struct VARDESC {
    pub memid: MEMBERID,
    pub lpstrSchema: LPOLESTR,
    pub u: VARDESC_U,
    pub elemdescVar: ELEMDESC,
    pub wVarFlags: WORD,
    pub varkind: VARKIND,
}

/// TLIBATTR
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TLIBATTR {
    pub guid: GUID,
    pub lcid: LCID,
    pub syskind: SYSKIND,
    pub wMajorVerNum: WORD,
    pub wMinorVerNum: WORD,
    pub wLibFlags: WORD,
}

/// CUSTDATAITEM / CUSTDATA
#[repr(C)]
pub struct CUSTDATAITEM {
    pub guid: GUID,
    pub varValue: VARIANTARG,
}
#[repr(C)]
pub struct CUSTDATA {
    pub cCustData: DWORD,
    pub prgCustData: *mut CUSTDATAITEM,
}

/// BINDPTR (oaidl.idl) — union of pointers.
#[repr(C)]
pub union BINDPTR {
    pub lpfuncdesc: *mut FUNCDESC,
    pub lpvardesc: *mut VARDESC,
    pub lptcomp: *mut ITypeComp,
}

// enums (values from oaidl.idl / wtypes.idl)
pub const TKIND_ENUM: TYPEKIND = 0;
pub const TKIND_RECORD: TYPEKIND = 1;
pub const TKIND_MODULE: TYPEKIND = 2;
pub const TKIND_INTERFACE: TYPEKIND = 3;
pub const TKIND_DISPATCH: TYPEKIND = 4;
pub const TKIND_COCLASS: TYPEKIND = 5;
pub const TKIND_ALIAS: TYPEKIND = 6;
pub const TKIND_UNION: TYPEKIND = 7;
pub const TKIND_MAX: TYPEKIND = 8;

pub const SYS_WIN16: SYSKIND = 0;
pub const SYS_WIN32: SYSKIND = 1;
pub const SYS_MAC: SYSKIND = 2;
pub const SYS_WIN64: SYSKIND = 3;

pub const CC_FASTCALL: CALLCONV = 0;
pub const CC_CDECL: CALLCONV = 1;
pub const CC_MSCPASCAL: CALLCONV = 2;
pub const CC_PASCAL: CALLCONV = 2;
pub const CC_MACPASCAL: CALLCONV = 3;
pub const CC_STDCALL: CALLCONV = 4;
pub const CC_FPFASTCALL: CALLCONV = 5;
pub const CC_SYSCALL: CALLCONV = 6;
pub const CC_MPWCDECL: CALLCONV = 7;
pub const CC_MPWPASCAL: CALLCONV = 8;
pub const CC_MAX: CALLCONV = 9;

pub const FUNC_VIRTUAL: FUNCKIND = 0;
pub const FUNC_PUREVIRTUAL: FUNCKIND = 1;
pub const FUNC_NONVIRTUAL: FUNCKIND = 2;
pub const FUNC_STATIC: FUNCKIND = 3;
pub const FUNC_DISPATCH: FUNCKIND = 4;

pub const INVOKE_FUNC: INVOKEKIND = 1;
pub const INVOKE_PROPERTYGET: INVOKEKIND = 2;
pub const INVOKE_PROPERTYPUT: INVOKEKIND = 4;
pub const INVOKE_PROPERTYPUTREF: INVOKEKIND = 8;

pub const VAR_PERINSTANCE: VARKIND = 0;
pub const VAR_STATIC: VARKIND = 1;
pub const VAR_CONST: VARKIND = 2;
pub const VAR_DISPATCH: VARKIND = 3;

pub const DESCKIND_NONE: DESCKIND = 0;
pub const DESCKIND_FUNCDESC: DESCKIND = 1;
pub const DESCKIND_VARDESC: DESCKIND = 2;
pub const DESCKIND_TYPECOMP: DESCKIND = 3;
pub const DESCKIND_IMPLICITAPPOBJ: DESCKIND = 4;

pub const REGKIND_DEFAULT: REGKIND = 0;
pub const REGKIND_REGISTER: REGKIND = 1;
pub const REGKIND_NONE: REGKIND = 2;

// TYPEFLAGS (oaidl.idl)
pub const TYPEFLAG_FAPPOBJECT: WORD = 0x1;
pub const TYPEFLAG_FCANCREATE: WORD = 0x2;
pub const TYPEFLAG_FLICENSED: WORD = 0x4;
pub const TYPEFLAG_FPREDECLID: WORD = 0x8;
pub const TYPEFLAG_FHIDDEN: WORD = 0x10;
pub const TYPEFLAG_FCONTROL: WORD = 0x20;
pub const TYPEFLAG_FDUAL: WORD = 0x40;
pub const TYPEFLAG_FNONEXTENSIBLE: WORD = 0x80;
pub const TYPEFLAG_FOLEAUTOMATION: WORD = 0x100;
pub const TYPEFLAG_FRESTRICTED: WORD = 0x200;
pub const TYPEFLAG_FAGGREGATABLE: WORD = 0x400;
pub const TYPEFLAG_FREPLACEABLE: WORD = 0x800;
pub const TYPEFLAG_FDISPATCHABLE: WORD = 0x1000;
pub const TYPEFLAG_FREVERSEBIND: WORD = 0x2000;
pub const TYPEFLAG_FPROXY: WORD = 0x4000;

// FUNCFLAGS
pub const FUNCFLAG_FRESTRICTED: WORD = 0x1;
pub const FUNCFLAG_FSOURCE: WORD = 0x2;
pub const FUNCFLAG_FBINDABLE: WORD = 0x4;
pub const FUNCFLAG_FREQUESTEDIT: WORD = 0x8;
pub const FUNCFLAG_FDISPLAYBIND: WORD = 0x10;
pub const FUNCFLAG_FDEFAULTBIND: WORD = 0x20;
pub const FUNCFLAG_FHIDDEN: WORD = 0x40;
pub const FUNCFLAG_FUSESGETLASTERROR: WORD = 0x80;
pub const FUNCFLAG_FDEFAULTCOLLELEM: WORD = 0x100;
pub const FUNCFLAG_FUIDEFAULT: WORD = 0x200;
pub const FUNCFLAG_FNONBROWSABLE: WORD = 0x400;
pub const FUNCFLAG_FREPLACEABLE: WORD = 0x800;
pub const FUNCFLAG_FIMMEDIATEBIND: WORD = 0x1000;

// VARFLAGS
pub const VARFLAG_FREADONLY: WORD = 0x1;

// LIBFLAGS
pub const LIBFLAG_FRESTRICTED: WORD = 0x1;
pub const LIBFLAG_FCONTROL: WORD = 0x2;
pub const LIBFLAG_FHIDDEN: WORD = 0x4;
pub const LIBFLAG_FHASDISKIMAGE: WORD = 0x8;

// IMPLTYPEFLAGS
pub const IMPLTYPEFLAG_FDEFAULT: INT = 0x1;
pub const IMPLTYPEFLAG_FSOURCE: INT = 0x2;
pub const IMPLTYPEFLAG_FRESTRICTED: INT = 0x4;
pub const IMPLTYPEFLAG_FDEFAULTVTABLE: INT = 0x8;

// MEMBERID / DISPID specials
pub const MEMBERID_NIL: MEMBERID = -1;
pub const DISPID_UNKNOWN: DISPID = -1;
pub const DISPID_VALUE: DISPID = 0;
pub const DISPID_PROPERTYPUT: DISPID = -3;
pub const DISPID_NEWENUM: DISPID = -4;
pub const DISPID_CONSTRUCTOR: DISPID = -6;
pub const DISPID_DESTRUCTOR: DISPID = -7;

// DISPATCH_* invoke flags
pub const DISPATCH_METHOD: WORD = 0x1;
pub const DISPATCH_PROPERTYGET: WORD = 0x2;
pub const DISPATCH_PROPERTYPUT: WORD = 0x4;
pub const DISPATCH_PROPERTYPUTREF: WORD = 0x8;

// ---------------------------------------------------------------------------
// IUnknown / IDispatch
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IUnknown, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IUnknown) -> ULONG,
}
#[repr(C)]
pub struct IUnknown {
    pub lpVtbl: *const IUnknownVtbl,
}

#[inline]
pub unsafe fn IUnknown_QueryInterface(
    p: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    ((*(*p).lpVtbl).QueryInterface)(p, riid, ppv)
}
#[inline]
pub unsafe fn IUnknown_AddRef(p: *mut IUnknown) -> ULONG {
    ((*(*p).lpVtbl).AddRef)(p)
}
#[inline]
pub unsafe fn IUnknown_Release(p: *mut IUnknown) -> ULONG {
    ((*(*p).lpVtbl).Release)(p)
}

#[repr(C)]
pub struct IDispatchVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IDispatch, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IDispatch) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IDispatch) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(*mut IDispatch, *mut UINT) -> HRESULT,
    pub GetTypeInfo:
        unsafe extern "system" fn(*mut IDispatch, UINT, LCID, *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(
        *mut IDispatch,
        REFIID,
        *mut LPOLESTR,
        UINT,
        LCID,
        *mut DISPID,
    ) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(
        *mut IDispatch,
        DISPID,
        REFIID,
        LCID,
        WORD,
        *mut DISPPARAMS,
        *mut VARIANT,
        *mut EXCEPINFO,
        *mut UINT,
    ) -> HRESULT,
}
#[repr(C)]
pub struct IDispatch {
    pub lpVtbl: *const IDispatchVtbl,
}
pub type LPDISPATCH = *mut IDispatch;

// ---------------------------------------------------------------------------
// ITypeLib / ITypeLib2 (oaidl.idl)
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct ITypeLibVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut ITypeLib, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut ITypeLib) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut ITypeLib) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(*mut ITypeLib) -> UINT,
    pub GetTypeInfo:
        unsafe extern "system" fn(*mut ITypeLib, UINT, *mut *mut ITypeInfo) -> HRESULT,
    pub GetTypeInfoType: unsafe extern "system" fn(*mut ITypeLib, UINT, *mut TYPEKIND) -> HRESULT,
    pub GetTypeInfoOfGuid:
        unsafe extern "system" fn(*mut ITypeLib, REFGUID, *mut *mut ITypeInfo) -> HRESULT,
    pub GetLibAttr: unsafe extern "system" fn(*mut ITypeLib, *mut *mut TLIBATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(*mut ITypeLib, *mut *mut ITypeComp) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(
        *mut ITypeLib,
        INT,
        *mut BSTR,
        *mut BSTR,
        *mut DWORD,
        *mut BSTR,
    ) -> HRESULT,
    pub IsName:
        unsafe extern "system" fn(*mut ITypeLib, LPOLESTR, ULONG, *mut BOOL) -> HRESULT,
    pub FindName: unsafe extern "system" fn(
        *mut ITypeLib,
        LPOLESTR,
        ULONG,
        *mut *mut ITypeInfo,
        *mut MEMBERID,
        *mut USHORT,
    ) -> HRESULT,
    pub ReleaseTLibAttr: unsafe extern "system" fn(*mut ITypeLib, *mut TLIBATTR),
}
#[repr(C)]
pub struct ITypeLib {
    pub lpVtbl: *const ITypeLibVtbl,
}

#[repr(C)]
pub struct ITypeLib2Vtbl {
    // ITypeLib
    pub QueryInterface:
        unsafe extern "system" fn(*mut ITypeLib2, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut ITypeLib2) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut ITypeLib2) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(*mut ITypeLib2) -> UINT,
    pub GetTypeInfo:
        unsafe extern "system" fn(*mut ITypeLib2, UINT, *mut *mut ITypeInfo) -> HRESULT,
    pub GetTypeInfoType:
        unsafe extern "system" fn(*mut ITypeLib2, UINT, *mut TYPEKIND) -> HRESULT,
    pub GetTypeInfoOfGuid:
        unsafe extern "system" fn(*mut ITypeLib2, REFGUID, *mut *mut ITypeInfo) -> HRESULT,
    pub GetLibAttr: unsafe extern "system" fn(*mut ITypeLib2, *mut *mut TLIBATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(*mut ITypeLib2, *mut *mut ITypeComp) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(
        *mut ITypeLib2,
        INT,
        *mut BSTR,
        *mut BSTR,
        *mut DWORD,
        *mut BSTR,
    ) -> HRESULT,
    pub IsName:
        unsafe extern "system" fn(*mut ITypeLib2, LPOLESTR, ULONG, *mut BOOL) -> HRESULT,
    pub FindName: unsafe extern "system" fn(
        *mut ITypeLib2,
        LPOLESTR,
        ULONG,
        *mut *mut ITypeInfo,
        *mut MEMBERID,
        *mut USHORT,
    ) -> HRESULT,
    pub ReleaseTLibAttr: unsafe extern "system" fn(*mut ITypeLib2, *mut TLIBATTR),
    // ITypeLib2
    pub GetCustData:
        unsafe extern "system" fn(*mut ITypeLib2, REFGUID, *mut VARIANT) -> HRESULT,
    pub GetLibStatistics:
        unsafe extern "system" fn(*mut ITypeLib2, *mut ULONG, *mut ULONG) -> HRESULT,
    pub GetDocumentation2: unsafe extern "system" fn(
        *mut ITypeLib2,
        INT,
        LCID,
        *mut BSTR,
        *mut DWORD,
        *mut BSTR,
    ) -> HRESULT,
    pub GetAllCustData: unsafe extern "system" fn(*mut ITypeLib2, *mut CUSTDATA) -> HRESULT,
}
#[repr(C)]
pub struct ITypeLib2 {
    pub lpVtbl: *const ITypeLib2Vtbl,
}

// ---------------------------------------------------------------------------
// ITypeInfo / ITypeInfo2 (oaidl.idl)
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct ITypeInfoVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut ITypeInfo, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut ITypeInfo) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut ITypeInfo) -> ULONG,
    pub GetTypeAttr: unsafe extern "system" fn(*mut ITypeInfo, *mut *mut TYPEATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(*mut ITypeInfo, *mut *mut ITypeComp) -> HRESULT,
    pub GetFuncDesc:
        unsafe extern "system" fn(*mut ITypeInfo, UINT, *mut *mut FUNCDESC) -> HRESULT,
    pub GetVarDesc:
        unsafe extern "system" fn(*mut ITypeInfo, UINT, *mut *mut VARDESC) -> HRESULT,
    pub GetNames: unsafe extern "system" fn(
        *mut ITypeInfo,
        MEMBERID,
        *mut BSTR,
        UINT,
        *mut UINT,
    ) -> HRESULT,
    pub GetRefTypeOfImplType:
        unsafe extern "system" fn(*mut ITypeInfo, UINT, *mut HREFTYPE) -> HRESULT,
    pub GetImplTypeFlags: unsafe extern "system" fn(*mut ITypeInfo, UINT, *mut INT) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(
        *mut ITypeInfo,
        *mut LPOLESTR,
        UINT,
        *mut MEMBERID,
    ) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(
        *mut ITypeInfo,
        PVOID,
        MEMBERID,
        WORD,
        *mut DISPPARAMS,
        *mut VARIANT,
        *mut EXCEPINFO,
        *mut UINT,
    ) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(
        *mut ITypeInfo,
        MEMBERID,
        *mut BSTR,
        *mut BSTR,
        *mut DWORD,
        *mut BSTR,
    ) -> HRESULT,
    pub GetDllEntry: unsafe extern "system" fn(
        *mut ITypeInfo,
        MEMBERID,
        INVOKEKIND,
        *mut BSTR,
        *mut BSTR,
        *mut WORD,
    ) -> HRESULT,
    pub GetRefTypeInfo:
        unsafe extern "system" fn(*mut ITypeInfo, HREFTYPE, *mut *mut ITypeInfo) -> HRESULT,
    pub AddressOfMember:
        unsafe extern "system" fn(*mut ITypeInfo, MEMBERID, INVOKEKIND, *mut PVOID) -> HRESULT,
    pub CreateInstance: unsafe extern "system" fn(
        *mut ITypeInfo,
        *mut IUnknown,
        REFIID,
        *mut PVOID,
    ) -> HRESULT,
    pub GetMops: unsafe extern "system" fn(*mut ITypeInfo, MEMBERID, *mut BSTR) -> HRESULT,
    pub GetContainingTypeLib:
        unsafe extern "system" fn(*mut ITypeInfo, *mut *mut ITypeLib, *mut UINT) -> HRESULT,
    pub ReleaseTypeAttr: unsafe extern "system" fn(*mut ITypeInfo, *mut TYPEATTR),
    pub ReleaseFuncDesc: unsafe extern "system" fn(*mut ITypeInfo, *mut FUNCDESC),
    pub ReleaseVarDesc: unsafe extern "system" fn(*mut ITypeInfo, *mut VARDESC),
}
#[repr(C)]
pub struct ITypeInfo {
    pub lpVtbl: *const ITypeInfoVtbl,
}

#[repr(C)]
pub struct ITypeInfo2Vtbl {
    // ITypeInfo (same 22 slots, self-typed)
    pub QueryInterface:
        unsafe extern "system" fn(*mut ITypeInfo2, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut ITypeInfo2) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut ITypeInfo2) -> ULONG,
    pub GetTypeAttr: unsafe extern "system" fn(*mut ITypeInfo2, *mut *mut TYPEATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(*mut ITypeInfo2, *mut *mut ITypeComp) -> HRESULT,
    pub GetFuncDesc:
        unsafe extern "system" fn(*mut ITypeInfo2, UINT, *mut *mut FUNCDESC) -> HRESULT,
    pub GetVarDesc:
        unsafe extern "system" fn(*mut ITypeInfo2, UINT, *mut *mut VARDESC) -> HRESULT,
    pub GetNames: unsafe extern "system" fn(
        *mut ITypeInfo2,
        MEMBERID,
        *mut BSTR,
        UINT,
        *mut UINT,
    ) -> HRESULT,
    pub GetRefTypeOfImplType:
        unsafe extern "system" fn(*mut ITypeInfo2, UINT, *mut HREFTYPE) -> HRESULT,
    pub GetImplTypeFlags: unsafe extern "system" fn(*mut ITypeInfo2, UINT, *mut INT) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(
        *mut ITypeInfo2,
        *mut LPOLESTR,
        UINT,
        *mut MEMBERID,
    ) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(
        *mut ITypeInfo2,
        PVOID,
        MEMBERID,
        WORD,
        *mut DISPPARAMS,
        *mut VARIANT,
        *mut EXCEPINFO,
        *mut UINT,
    ) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(
        *mut ITypeInfo2,
        MEMBERID,
        *mut BSTR,
        *mut BSTR,
        *mut DWORD,
        *mut BSTR,
    ) -> HRESULT,
    pub GetDllEntry: unsafe extern "system" fn(
        *mut ITypeInfo2,
        MEMBERID,
        INVOKEKIND,
        *mut BSTR,
        *mut BSTR,
        *mut WORD,
    ) -> HRESULT,
    pub GetRefTypeInfo:
        unsafe extern "system" fn(*mut ITypeInfo2, HREFTYPE, *mut *mut ITypeInfo) -> HRESULT,
    pub AddressOfMember:
        unsafe extern "system" fn(*mut ITypeInfo2, MEMBERID, INVOKEKIND, *mut PVOID) -> HRESULT,
    pub CreateInstance: unsafe extern "system" fn(
        *mut ITypeInfo2,
        *mut IUnknown,
        REFIID,
        *mut PVOID,
    ) -> HRESULT,
    pub GetMops: unsafe extern "system" fn(*mut ITypeInfo2, MEMBERID, *mut BSTR) -> HRESULT,
    pub GetContainingTypeLib:
        unsafe extern "system" fn(*mut ITypeInfo2, *mut *mut ITypeLib, *mut UINT) -> HRESULT,
    pub ReleaseTypeAttr: unsafe extern "system" fn(*mut ITypeInfo2, *mut TYPEATTR),
    pub ReleaseFuncDesc: unsafe extern "system" fn(*mut ITypeInfo2, *mut FUNCDESC),
    pub ReleaseVarDesc: unsafe extern "system" fn(*mut ITypeInfo2, *mut VARDESC),
    // ITypeInfo2
    pub GetTypeKind: unsafe extern "system" fn(*mut ITypeInfo2, *mut TYPEKIND) -> HRESULT,
    pub GetTypeFlags: unsafe extern "system" fn(*mut ITypeInfo2, *mut ULONG) -> HRESULT,
    pub GetFuncIndexOfMemId: unsafe extern "system" fn(
        *mut ITypeInfo2,
        MEMBERID,
        INVOKEKIND,
        *mut UINT,
    ) -> HRESULT,
    pub GetVarIndexOfMemId:
        unsafe extern "system" fn(*mut ITypeInfo2, MEMBERID, *mut UINT) -> HRESULT,
    pub GetCustData:
        unsafe extern "system" fn(*mut ITypeInfo2, REFGUID, *mut VARIANT) -> HRESULT,
    pub GetFuncCustData:
        unsafe extern "system" fn(*mut ITypeInfo2, UINT, REFGUID, *mut VARIANT) -> HRESULT,
    pub GetParamCustData: unsafe extern "system" fn(
        *mut ITypeInfo2,
        UINT,
        UINT,
        REFGUID,
        *mut VARIANT,
    ) -> HRESULT,
    pub GetVarCustData:
        unsafe extern "system" fn(*mut ITypeInfo2, UINT, REFGUID, *mut VARIANT) -> HRESULT,
    pub GetImplTypeCustData:
        unsafe extern "system" fn(*mut ITypeInfo2, UINT, REFGUID, *mut VARIANT) -> HRESULT,
    pub GetDocumentation2: unsafe extern "system" fn(
        *mut ITypeInfo2,
        MEMBERID,
        LCID,
        *mut BSTR,
        *mut DWORD,
        *mut BSTR,
    ) -> HRESULT,
    pub GetAllCustData: unsafe extern "system" fn(*mut ITypeInfo2, *mut CUSTDATA) -> HRESULT,
    pub GetAllFuncCustData:
        unsafe extern "system" fn(*mut ITypeInfo2, UINT, *mut CUSTDATA) -> HRESULT,
    pub GetAllParamCustData:
        unsafe extern "system" fn(*mut ITypeInfo2, UINT, UINT, *mut CUSTDATA) -> HRESULT,
    pub GetAllVarCustData:
        unsafe extern "system" fn(*mut ITypeInfo2, UINT, *mut CUSTDATA) -> HRESULT,
    pub GetAllImplTypeCustData:
        unsafe extern "system" fn(*mut ITypeInfo2, UINT, *mut CUSTDATA) -> HRESULT,
}
#[repr(C)]
pub struct ITypeInfo2 {
    pub lpVtbl: *const ITypeInfo2Vtbl,
}

// ---------------------------------------------------------------------------
// ITypeComp (oaidl.idl)
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct ITypeCompVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut ITypeComp, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut ITypeComp) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut ITypeComp) -> ULONG,
    pub Bind: unsafe extern "system" fn(
        *mut ITypeComp,
        LPOLESTR,
        ULONG,
        WORD,
        *mut *mut ITypeInfo,
        *mut DESCKIND,
        *mut BINDPTR,
    ) -> HRESULT,
    pub BindType: unsafe extern "system" fn(
        *mut ITypeComp,
        LPOLESTR,
        ULONG,
        *mut *mut ITypeInfo,
        *mut *mut ITypeComp,
    ) -> HRESULT,
}
#[repr(C)]
pub struct ITypeComp {
    pub lpVtbl: *const ITypeCompVtbl,
}

// ---------------------------------------------------------------------------
// ICreateTypeInfo2 / ICreateTypeLib2 (oaidl.idl — slot order verified against
// sdk/include/psdk/oaidl.idl). Added for the Lane A2 typelib continuation.
// ---------------------------------------------------------------------------

def_guid!(IID_ICreateTypeInfo,  0x00020405, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_ICreateTypeInfo2, 0x0002040E, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_ICreateTypeLib,   0x00020406, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_ICreateTypeLib2,  0x0002040F, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);

#[repr(C)]
pub struct ICreateTypeInfo2Vtbl {
    // IUnknown
    pub QueryInterface:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut ICreateTypeInfo2) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut ICreateTypeInfo2) -> ULONG,
    // ICreateTypeInfo
    pub SetGuid: unsafe extern "system" fn(*mut ICreateTypeInfo2, REFGUID) -> HRESULT,
    pub SetTypeFlags: unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT) -> HRESULT,
    pub SetDocString: unsafe extern "system" fn(*mut ICreateTypeInfo2, LPOLESTR) -> HRESULT,
    pub SetHelpContext: unsafe extern "system" fn(*mut ICreateTypeInfo2, DWORD) -> HRESULT,
    pub SetVersion: unsafe extern "system" fn(*mut ICreateTypeInfo2, WORD, WORD) -> HRESULT,
    pub AddRefTypeInfo:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, *mut ITypeInfo, *mut HREFTYPE) -> HRESULT,
    pub AddFuncDesc:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, *mut FUNCDESC) -> HRESULT,
    pub AddImplType:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, HREFTYPE) -> HRESULT,
    pub SetImplTypeFlags:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, INT) -> HRESULT,
    pub SetAlignment: unsafe extern "system" fn(*mut ICreateTypeInfo2, WORD) -> HRESULT,
    pub SetSchema: unsafe extern "system" fn(*mut ICreateTypeInfo2, LPOLESTR) -> HRESULT,
    pub AddVarDesc:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, *mut VARDESC) -> HRESULT,
    pub SetFuncAndParamNames:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, *mut LPOLESTR, UINT) -> HRESULT,
    pub SetVarName: unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, LPOLESTR) -> HRESULT,
    pub SetTypeDescAlias:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, *mut TYPEDESC) -> HRESULT,
    pub DefineFuncAsDllEntry:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, LPOLESTR, LPOLESTR) -> HRESULT,
    pub SetFuncDocString:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, LPOLESTR) -> HRESULT,
    pub SetVarDocString:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, LPOLESTR) -> HRESULT,
    pub SetFuncHelpContext:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, DWORD) -> HRESULT,
    pub SetVarHelpContext:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, DWORD) -> HRESULT,
    pub SetMops: unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, BSTR) -> HRESULT,
    pub SetTypeIdldesc:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, *mut IDLDESC) -> HRESULT,
    pub LayOut: unsafe extern "system" fn(*mut ICreateTypeInfo2) -> HRESULT,
    // ICreateTypeInfo2
    pub DeleteFuncDesc: unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT) -> HRESULT,
    pub DeleteFuncDescByMemId:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, MEMBERID, INVOKEKIND) -> HRESULT,
    pub DeleteVarDesc: unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT) -> HRESULT,
    pub DeleteVarDescByMemId:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, MEMBERID) -> HRESULT,
    pub DeleteImplType: unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT) -> HRESULT,
    pub SetCustData:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, REFGUID, *mut VARIANT) -> HRESULT,
    pub SetFuncCustData:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, REFGUID, *mut VARIANT) -> HRESULT,
    pub SetParamCustData: unsafe extern "system" fn(
        *mut ICreateTypeInfo2,
        UINT,
        UINT,
        REFGUID,
        *mut VARIANT,
    ) -> HRESULT,
    pub SetVarCustData:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, REFGUID, *mut VARIANT) -> HRESULT,
    pub SetImplTypeCustData:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, REFGUID, *mut VARIANT) -> HRESULT,
    pub SetHelpStringContext:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, ULONG) -> HRESULT,
    pub SetFuncHelpStringContext:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, ULONG) -> HRESULT,
    pub SetVarHelpStringContext:
        unsafe extern "system" fn(*mut ICreateTypeInfo2, UINT, ULONG) -> HRESULT,
    pub Invalidate: unsafe extern "system" fn(*mut ICreateTypeInfo2) -> HRESULT,
    pub SetName: unsafe extern "system" fn(*mut ICreateTypeInfo2, LPOLESTR) -> HRESULT,
}
#[repr(C)]
pub struct ICreateTypeInfo2 {
    pub lpVtbl: *const ICreateTypeInfo2Vtbl,
}

#[repr(C)]
pub struct ICreateTypeLib2Vtbl {
    // IUnknown
    pub QueryInterface:
        unsafe extern "system" fn(*mut ICreateTypeLib2, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut ICreateTypeLib2) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut ICreateTypeLib2) -> ULONG,
    // ICreateTypeLib
    pub CreateTypeInfo: unsafe extern "system" fn(
        *mut ICreateTypeLib2,
        LPOLESTR,
        TYPEKIND,
        *mut *mut ICreateTypeInfo2, // ICreateTypeInfo** in the IDL; impl hands out the 2 flavor
    ) -> HRESULT,
    pub SetName: unsafe extern "system" fn(*mut ICreateTypeLib2, LPOLESTR) -> HRESULT,
    pub SetVersion: unsafe extern "system" fn(*mut ICreateTypeLib2, WORD, WORD) -> HRESULT,
    pub SetGuid: unsafe extern "system" fn(*mut ICreateTypeLib2, REFGUID) -> HRESULT,
    pub SetDocString: unsafe extern "system" fn(*mut ICreateTypeLib2, LPOLESTR) -> HRESULT,
    pub SetHelpFileName: unsafe extern "system" fn(*mut ICreateTypeLib2, LPOLESTR) -> HRESULT,
    pub SetHelpContext: unsafe extern "system" fn(*mut ICreateTypeLib2, DWORD) -> HRESULT,
    pub SetLcid: unsafe extern "system" fn(*mut ICreateTypeLib2, LCID) -> HRESULT,
    pub SetLibFlags: unsafe extern "system" fn(*mut ICreateTypeLib2, UINT) -> HRESULT,
    pub SaveAllChanges: unsafe extern "system" fn(*mut ICreateTypeLib2) -> HRESULT,
    // ICreateTypeLib2
    pub DeleteTypeInfo: unsafe extern "system" fn(*mut ICreateTypeLib2, LPOLESTR) -> HRESULT,
    pub SetCustData:
        unsafe extern "system" fn(*mut ICreateTypeLib2, REFGUID, *mut VARIANT) -> HRESULT,
    pub SetHelpStringContext:
        unsafe extern "system" fn(*mut ICreateTypeLib2, ULONG) -> HRESULT,
    pub SetHelpStringDll: unsafe extern "system" fn(*mut ICreateTypeLib2, LPOLESTR) -> HRESULT,
}
#[repr(C)]
pub struct ICreateTypeLib2 {
    pub lpVtbl: *const ICreateTypeLib2Vtbl,
}

// ---------------------------------------------------------------------------
// System imports shared by dispatch.rs/typelib.rs
// ---------------------------------------------------------------------------

extern "system" {
    // ole32
    pub fn CoTaskMemAlloc(size: SIZE_T) -> PVOID;
    pub fn CoTaskMemFree(pv: PVOID);
    // kernel32
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> PVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
    pub fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID, dwBytes: SIZE_T) -> PVOID;
    pub fn OutputDebugStringA(lpOutputString: LPCSTR);
}

pub const HEAP_ZERO_MEMORY: DWORD = 0x8;

/// wine/heap.h helpers.
#[inline]
pub unsafe fn heap_alloc(len: usize) -> PVOID {
    HeapAlloc(GetProcessHeap(), 0, len)
}
#[inline]
pub unsafe fn heap_alloc_zero(len: usize) -> PVOID {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}
#[inline]
pub unsafe fn heap_realloc(mem: PVOID, len: usize) -> PVOID {
    HeapReAlloc(GetProcessHeap(), 0, mem, len)
}
#[inline]
pub unsafe fn heap_free(mem: PVOID) -> BOOL {
    HeapFree(GetProcessHeap(), 0, mem)
}

// Interlocked* — winnt.h inlines → core atomics (C return semantics).
#[inline]
pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (&*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        + 1
}
#[inline]
pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (&*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1
}

/// Debug print sink (ERR!/FIXME!/WARN! forward here; no alloc, no panic).
pub fn dbg_print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    struct Buf {
        b: [u8; 512],
        n: usize,
    }
    impl Write for Buf {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for &c in s.as_bytes() {
                if self.n < self.b.len() - 1 {
                    self.b[self.n] = c;
                    self.n += 1;
                }
            }
            Ok(())
        }
    }
    let mut buf = Buf { b: [0u8; 512], n: 0 };
    let _ = buf.write_fmt(args);
    buf.b[buf.n] = 0;
    unsafe { OutputDebugStringA(buf.b.as_ptr() as *const i8) };
}

/// Compile-time UTF-16 NUL-terminated literal (ASCII only).
macro_rules! wide {
    ($s:literal) => {{
        const B: &[u8] = $s.as_bytes();
        const N: usize = B.len() + 1;
        const W: [u16; N] = {
            let mut a = [0u16; N];
            let mut i = 0;
            while i < B.len() {
                a[i] = B[i] as u16;
                i += 1;
            }
            a
        };
        &W
    }};
}
pub(crate) use wide;
