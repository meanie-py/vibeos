//! Byte-exact Rust mirror of the private + public types `actctx.c` uses.
//!
//! Two classes of struct live here:
//!
//!  * **On-disk section-data layouts** (`strsection_header`,
//!    `string_index`, `guidsection_header`, `guid_index`, and the
//!    `*redirect_data` family). These are serialized into the
//!    activation-context blob and read back by name/guid lookup, so they
//!    are a hard ABI contract — every field offset and the total size
//!    must match the C exactly. Compile-time size assertions guard them.
//!
//!  * **In-memory bookkeeping** (`struct assembly`, `entity`,
//!    `ACTIVATION_CONTEXT`, `actctx_loader`, the XML cursor types). Once
//!    actctx.c is fully Rust these are private to this crate (callers get
//!    an opaque `HANDLE`), but the field order is kept faithful to the C.
//!
//!  * **Public ActCtx API types** (`ACTCTXW`,
//!    `ACTIVATION_CONTEXT_*_INFORMATION`, …) transcribed from
//!    `sdk/include/psdk/winbase.h` + `sdk/include/xdk/winnt_old.h` — the
//!    kernel `ntkrnl-sys` bindgen does not carry these user-mode types.

// Scalars `base` already provides are imported (NOT redefined) so a
// `use crate::types::*` alongside `use ntkrnl_sys::base::*` in actctx.rs
// doesn't create a double-glob ambiguity.
use ntkrnl_sys::base::{LIST_ENTRY, LONG, PCWSTR, PVOID, SIZE_T, ULONG, UNICODE_STRING, USHORT, WCHAR};
use ntkrnl_sys::ddk::{GUID, LANGID, LARGE_INTEGER};

// ── primitive aliases the C uses that base/ddk lack ─────────────────
pub type DWORD = u32;
pub type WORD = u16;
pub type BYTE = u8;
pub type BOOL = i32;
pub type HANDLE = PVOID;
pub type HMODULE = PVOID;
pub type LPWSTR = *mut WCHAR;
pub type LPCWSTR = *const WCHAR;
// `enum comclass_threadingmodel` used as a type in one draft signature.
pub type comclass_threadingmodel = u32;

// ── file-level magic / flag constants (actctx.c top) ────────────────
pub const ACTCTX_MAGIC: ULONG = 0xC07E_3E11;
pub const STRSECTION_MAGIC: DWORD = 0x6448_7353; // 'dHsS'
pub const GUIDSECTION_MAGIC: DWORD = 0x6448_7347; // 'dHsG'
pub const ACTCTX_FAKE_HANDLE: HANDLE = 0xf00baa as HANDLE;
pub const MAX_NAMESPACES: usize = 64;

// RT_MANIFEST / CREATEPROCESS_MANIFEST_RESOURCE_ID (actctx.c re-defines
// these locally to avoid including winuser.h).
pub const RT_MANIFEST: usize = 24;
pub const CREATEPROCESS_MANIFEST_RESOURCE_ID: usize = 1;

// ── ACTCTX_FLAG_* (winbase.h) + the ACTCTX_FLAGS_ALL aggregate ──────
pub const ACTCTX_FLAG_PROCESSOR_ARCHITECTURE_VALID: DWORD = 0x0000_0001;
pub const ACTCTX_FLAG_LANGID_VALID: DWORD = 0x0000_0002;
pub const ACTCTX_FLAG_ASSEMBLY_DIRECTORY_VALID: DWORD = 0x0000_0004;
pub const ACTCTX_FLAG_RESOURCE_NAME_VALID: DWORD = 0x0000_0008;
pub const ACTCTX_FLAG_SET_PROCESS_DEFAULT: DWORD = 0x0000_0010;
pub const ACTCTX_FLAG_APPLICATION_NAME_VALID: DWORD = 0x0000_0020;
pub const ACTCTX_FLAG_SOURCE_IS_ASSEMBLYREF: DWORD = 0x0000_0040;
pub const ACTCTX_FLAG_HMODULE_VALID: DWORD = 0x0000_0080;
pub const ACTCTX_FLAGS_ALL: DWORD = ACTCTX_FLAG_PROCESSOR_ARCHITECTURE_VALID
    | ACTCTX_FLAG_LANGID_VALID
    | ACTCTX_FLAG_ASSEMBLY_DIRECTORY_VALID
    | ACTCTX_FLAG_RESOURCE_NAME_VALID
    | ACTCTX_FLAG_SET_PROCESS_DEFAULT
    | ACTCTX_FLAG_APPLICATION_NAME_VALID
    | ACTCTX_FLAG_SOURCE_IS_ASSEMBLYREF
    | ACTCTX_FLAG_HMODULE_VALID;

pub const DEACTIVATE_ACTCTX_FLAG_FORCE_EARLY_DEACTIVATION: DWORD = 0x0000_0001;
pub const QUERY_ACTCTX_FLAG_USE_ACTIVE_ACTCTX: DWORD = 0x0000_0004;
pub const QUERY_ACTCTX_FLAG_ACTCTX_IS_HMODULE: DWORD = 0x0000_0008;
pub const QUERY_ACTCTX_FLAG_ACTCTX_IS_ADDRESS: DWORD = 0x0000_0010;
pub const QUERY_ACTCTX_FLAG_NO_ADDREF: DWORD = 0x8000_0000;

// ── ACTIVATION_CONTEXT_PATH_TYPE_* (winnt_old.h) ────────────────────
pub const ACTIVATION_CONTEXT_PATH_TYPE_NONE: ULONG = 1;
pub const ACTIVATION_CONTEXT_PATH_TYPE_WIN32_FILE: ULONG = 2;
pub const ACTIVATION_CONTEXT_PATH_TYPE_URL: ULONG = 3;
pub const ACTIVATION_CONTEXT_PATH_TYPE_ASSEMBLYREF: ULONG = 4;

// ── ACTIVATION_CONTEXT_SECTION_* (winnt_old.h) ──────────────────────
pub const ACTIVATION_CONTEXT_SECTION_ASSEMBLY_INFORMATION: ULONG = 1;
pub const ACTIVATION_CONTEXT_SECTION_DLL_REDIRECTION: ULONG = 2;
pub const ACTIVATION_CONTEXT_SECTION_WINDOW_CLASS_REDIRECTION: ULONG = 3;
pub const ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION: ULONG = 4;
pub const ACTIVATION_CONTEXT_SECTION_COM_INTERFACE_REDIRECTION: ULONG = 5;
pub const ACTIVATION_CONTEXT_SECTION_COM_TYPE_LIBRARY_REDIRECTION: ULONG = 6;
pub const ACTIVATION_CONTEXT_SECTION_COM_PROGID_REDIRECTION: ULONG = 7;
pub const ACTIVATION_CONTEXT_SECTION_GLOBAL_OBJECT_RENAME_TABLE: ULONG = 8;
pub const ACTIVATION_CONTEXT_SECTION_CLR_SURROGATES: ULONG = 9;
// winnt_old.h:1130 — was 16 here; only an internal entity kind, but keep it the C value.
pub const ACTIVATION_CONTEXT_SECTION_APPLICATION_SETTINGS: ULONG = 10;
pub const ACTIVATION_CONTEXT_SECTION_KEYED_DATA: ULONG = 0;

// ── ACTIVATION_CONTEXT_INFO_CLASS (winnt_old.h) ─────────────────────
pub const ActivationContextBasicInformation: u32 = 1;
pub const ActivationContextDetailedInformation: u32 = 2;
pub const AssemblyDetailedInformationInActivationContext: u32 = 3;
pub const FileInformationInAssemblyOfAssemblyInActivationContext: u32 = 4;
pub const RunlevelInformationInActivationContext: u32 = 5;
pub const CompatibilityInformationInActivationContext: u32 = 6;
pub const ActivationContextManifestResourceName: u32 = 7;
pub const MaxActivationContextInfoClass: u32 = 8;

// ── ACTCTX_REQUESTED_RUN_LEVEL (winnt_old.h) ────────────────────────
pub type ACTCTX_REQUESTED_RUN_LEVEL = DWORD;
pub const ACTCTX_RUN_LEVEL_UNSPECIFIED: ACTCTX_REQUESTED_RUN_LEVEL = 0;
pub const ACTCTX_RUN_LEVEL_AS_INVOKER: ACTCTX_REQUESTED_RUN_LEVEL = 1;
pub const ACTCTX_RUN_LEVEL_HIGHEST_AVAILABLE: ACTCTX_REQUESTED_RUN_LEVEL = 2;
pub const ACTCTX_RUN_LEVEL_REQUIRE_ADMIN: ACTCTX_REQUESTED_RUN_LEVEL = 3;
pub const ACTCTX_RUN_LEVEL_NUMBERS: ACTCTX_REQUESTED_RUN_LEVEL = 4;

// ── ACTCTX_COMPATIBILITY_ELEMENT_TYPE (winnt_old.h) ─────────────────
pub type ACTCTX_COMPATIBILITY_ELEMENT_TYPE = DWORD;
pub const ACTCTX_COMPATIBILITY_ELEMENT_TYPE_UNKNOWN: ACTCTX_COMPATIBILITY_ELEMENT_TYPE = 0;
pub const ACTCTX_COMPATIBILITY_ELEMENT_TYPE_OS: ACTCTX_COMPATIBILITY_ELEMENT_TYPE = 1;
pub const ACTCTX_COMPATIBILITY_ELEMENT_TYPE_MITIGATION: ACTCTX_COMPATIBILITY_ELEMENT_TYPE = 2;
pub const ACTCTX_COMPATIBILITY_ELEMENT_TYPE_MAXVERSIONTESTED: ACTCTX_COMPATIBILITY_ELEMENT_TYPE = 3;

// ════════════════════════════════════════════════════════════════════
//  Public ActCtx API structs (winbase.h / winnt_old.h)
// ════════════════════════════════════════════════════════════════════

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ACTCTXW {
    pub cbSize: ULONG,
    pub dwFlags: DWORD,
    pub lpSource: LPCWSTR,
    pub wProcessorArchitecture: USHORT,
    pub wLangId: LANGID,
    pub lpAssemblyDirectory: LPCWSTR,
    pub lpResourceName: LPCWSTR,
    pub lpApplicationName: LPCWSTR,
    pub hModule: HMODULE,
}
pub type PCACTCTXW = *const ACTCTXW;

#[repr(C)]
pub struct ACTIVATION_CONTEXT_BASIC_INFORMATION {
    pub hActCtx: HANDLE,
    pub dwFlags: DWORD,
}
pub type PACTIVATION_CONTEXT_BASIC_INFORMATION = *mut ACTIVATION_CONTEXT_BASIC_INFORMATION;

#[repr(C)]
pub struct ACTCTX_SECTION_KEYED_DATA_ASSEMBLY_METADATA {
    pub lpInformation: PVOID,
    pub lpSectionBase: PVOID,
    pub ulSectionLength: ULONG,
    pub lpSectionGlobalDataBase: PVOID,
    pub ulSectionGlobalDataLength: ULONG,
}

#[repr(C)]
pub struct ACTCTX_SECTION_KEYED_DATA {
    pub cbSize: ULONG,
    pub ulDataFormatVersion: ULONG,
    pub lpData: PVOID,
    pub ulLength: ULONG,
    pub lpSectionGlobalData: PVOID,
    pub ulSectionGlobalDataLength: ULONG,
    pub lpSectionBase: PVOID,
    pub ulSectionTotalLength: ULONG,
    pub hActCtx: HANDLE,
    pub ulAssemblyRosterIndex: ULONG,
    pub ulFlags: ULONG,
    pub AssemblyMetadata: ACTCTX_SECTION_KEYED_DATA_ASSEMBLY_METADATA,
}
pub type PACTCTX_SECTION_KEYED_DATA = *mut ACTCTX_SECTION_KEYED_DATA;

#[repr(C)]
pub struct ACTIVATION_CONTEXT_ASSEMBLY_DETAILED_INFORMATION {
    pub ulFlags: DWORD,
    pub ulEncodedAssemblyIdentityLength: DWORD,
    pub ulManifestPathType: DWORD,
    pub ulManifestPathLength: DWORD,
    pub liManifestLastWriteTime: LARGE_INTEGER,
    pub ulPolicyPathType: DWORD,
    pub ulPolicyPathLength: DWORD,
    pub liPolicyLastWriteTime: LARGE_INTEGER,
    pub ulMetadataSatelliteRosterIndex: DWORD,
    pub ulManifestVersionMajor: DWORD,
    pub ulManifestVersionMinor: DWORD,
    pub ulPolicyVersionMajor: DWORD,
    pub ulPolicyVersionMinor: DWORD,
    pub ulAssemblyDirectoryNameLength: DWORD,
    pub lpAssemblyEncodedAssemblyIdentity: PCWSTR,
    pub lpAssemblyManifestPath: PCWSTR,
    pub lpAssemblyPolicyPath: PCWSTR,
    pub lpAssemblyDirectoryName: PCWSTR,
    pub ulFileCount: DWORD,
}
pub type PACTIVATION_CONTEXT_ASSEMBLY_DETAILED_INFORMATION =
    *mut ACTIVATION_CONTEXT_ASSEMBLY_DETAILED_INFORMATION;

#[repr(C)]
pub struct ACTIVATION_CONTEXT_DETAILED_INFORMATION {
    pub dwFlags: DWORD,
    pub ulFormatVersion: DWORD,
    pub ulAssemblyCount: DWORD,
    pub ulRootManifestPathType: DWORD,
    pub ulRootManifestPathChars: DWORD,
    pub ulRootConfigurationPathType: DWORD,
    pub ulRootConfigurationPathChars: DWORD,
    pub ulAppDirPathType: DWORD,
    pub ulAppDirPathChars: DWORD,
    pub lpRootManifestPath: PCWSTR,
    pub lpRootConfigurationPath: PCWSTR,
    pub lpAppDirPath: PCWSTR,
}
pub type PACTIVATION_CONTEXT_DETAILED_INFORMATION = *mut ACTIVATION_CONTEXT_DETAILED_INFORMATION;

#[repr(C)]
pub struct ACTIVATION_CONTEXT_QUERY_INDEX {
    pub ulAssemblyIndex: DWORD,
    pub ulFileIndexInAssembly: DWORD,
}
pub type PACTIVATION_CONTEXT_QUERY_INDEX = *mut ACTIVATION_CONTEXT_QUERY_INDEX;

#[repr(C)]
pub struct ASSEMBLY_FILE_DETAILED_INFORMATION {
    pub ulFlags: DWORD,
    pub ulFilenameLength: DWORD,
    pub ulPathLength: DWORD,
    pub lpFileName: PCWSTR,
    pub lpFilePath: PCWSTR,
}
pub type PASSEMBLY_FILE_DETAILED_INFORMATION = *mut ASSEMBLY_FILE_DETAILED_INFORMATION;

#[repr(C)]
pub struct ACTIVATION_CONTEXT_RUN_LEVEL_INFORMATION {
    pub ulFlags: DWORD,
    pub RunLevel: ACTCTX_REQUESTED_RUN_LEVEL,
    pub UiAccess: DWORD,
}
pub type PACTIVATION_CONTEXT_RUN_LEVEL_INFORMATION = *mut ACTIVATION_CONTEXT_RUN_LEVEL_INFORMATION;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct COMPATIBILITY_CONTEXT_ELEMENT {
    pub Id: GUID,
    pub Type: ACTCTX_COMPATIBILITY_ELEMENT_TYPE,
}
pub type PCOMPATIBILITY_CONTEXT_ELEMENT = *mut COMPATIBILITY_CONTEXT_ELEMENT;

// ════════════════════════════════════════════════════════════════════
//  Private OLE/typelib enums (actctx.c, !__REACTOS__ block re-typed)
// ════════════════════════════════════════════════════════════════════

pub type OLEMISC = DWORD;
pub const OLEMISC_RECOMPOSEONRESIZE: OLEMISC = 0x1;
pub const OLEMISC_ONLYICONIC: OLEMISC = 0x2;
pub const OLEMISC_INSERTNOTREPLACE: OLEMISC = 0x4;
pub const OLEMISC_STATIC: OLEMISC = 0x8;
pub const OLEMISC_CANTLINKINSIDE: OLEMISC = 0x10;
pub const OLEMISC_CANLINKBYOLE1: OLEMISC = 0x20;
pub const OLEMISC_ISLINKOBJECT: OLEMISC = 0x40;
pub const OLEMISC_INSIDEOUT: OLEMISC = 0x80;
pub const OLEMISC_ACTIVATEWHENVISIBLE: OLEMISC = 0x100;
pub const OLEMISC_RENDERINGISDEVICEINDEPENDENT: OLEMISC = 0x200;
pub const OLEMISC_INVISIBLEATRUNTIME: OLEMISC = 0x400;
pub const OLEMISC_ALWAYSRUN: OLEMISC = 0x800;
pub const OLEMISC_ACTSLIKEBUTTON: OLEMISC = 0x1000;
pub const OLEMISC_ACTSLIKELABEL: OLEMISC = 0x2000;
pub const OLEMISC_NOUIACTIVATE: OLEMISC = 0x4000;
pub const OLEMISC_ALIGNABLE: OLEMISC = 0x8000;
pub const OLEMISC_SIMPLEFRAME: OLEMISC = 0x10000;
pub const OLEMISC_SETCLIENTSITEFIRST: OLEMISC = 0x20000;
pub const OLEMISC_IMEMODE: OLEMISC = 0x40000;
pub const OLEMISC_IGNOREACTIVATEWHENVISIBLE: OLEMISC = 0x80000;
pub const OLEMISC_WANTSTOMENUMERGE: OLEMISC = 0x100000;
pub const OLEMISC_SUPPORTSMULTILEVELUNDO: OLEMISC = 0x200000;

// comclass_threadingmodel
pub const ThreadingModel_Apartment: DWORD = 1;
pub const ThreadingModel_Free: DWORD = 2;
pub const ThreadingModel_No: DWORD = 3;
pub const ThreadingModel_Both: DWORD = 4;
pub const ThreadingModel_Neutral: DWORD = 5;

// comclass_miscfields
pub const MiscStatus: DWORD = 1;
pub const MiscStatusIcon: DWORD = 2;
pub const MiscStatusContent: DWORD = 4;
pub const MiscStatusThumbnail: DWORD = 8;
pub const MiscStatusDocPrint: DWORD = 16;

// ifaceps_mask
pub const NumMethods: DWORD = 1;
pub const BaseIface: DWORD = 2;

// LIBFLAGS
pub const LIBFLAG_FRESTRICTED: DWORD = 0x1;
pub const LIBFLAG_FCONTROL: DWORD = 0x2;
pub const LIBFLAG_FHIDDEN: DWORD = 0x4;
pub const LIBFLAG_FHASDISKIMAGE: DWORD = 0x8;

// ════════════════════════════════════════════════════════════════════
//  XML cursor / string types
// ════════════════════════════════════════════════════════════════════

#[repr(C)]
#[derive(Clone, Copy)]
pub struct xmlstr_t {
    pub ptr: *const WCHAR,
    pub len: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct xml_elem {
    pub name: xmlstr_t,
    pub ns: xmlstr_t,
    pub ns_pos: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct xml_attr {
    pub name: xmlstr_t,
    pub value: xmlstr_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct xmlbuf_t {
    pub ptr: *const WCHAR,
    pub end: *const WCHAR,
    pub namespaces: [xml_attr; MAX_NAMESPACES],
    pub ns_pos: i32,
    pub error: BOOL,
}

// ════════════════════════════════════════════════════════════════════
//  In-memory bookkeeping structs
// ════════════════════════════════════════════════════════════════════

#[repr(C)]
pub struct file_info {
    pub type_: ULONG,
    pub info: *mut WCHAR,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct assembly_version {
    pub major: USHORT,
    pub minor: USHORT,
    pub build: USHORT,
    pub revision: USHORT,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct assembly_identity {
    pub name: *mut WCHAR,
    pub arch: *mut WCHAR,
    pub public_key: *mut WCHAR,
    pub language: *mut WCHAR,
    pub type_: *mut WCHAR,
    pub version: assembly_version,
    pub optional: BOOL,
    pub delayed: BOOL,
}

// ── on-disk section headers/indices (BYTE-EXACT — ABI contract) ──────

#[repr(C)]
pub struct strsection_header {
    pub magic: DWORD,
    pub size: ULONG,
    pub unk1: [DWORD; 3],
    pub count: ULONG,
    pub index_offset: ULONG,
    pub unk2: [DWORD; 2],
    pub global_offset: ULONG,
    pub global_len: ULONG,
}

#[repr(C)]
pub struct string_index {
    pub hash: ULONG,
    pub name_offset: ULONG,
    pub name_len: ULONG,
    pub data_offset: ULONG,
    pub data_len: ULONG,
    pub rosterindex: ULONG,
}

#[repr(C)]
pub struct guidsection_header {
    pub magic: DWORD,
    pub size: ULONG,
    pub unk: [DWORD; 3],
    pub count: ULONG,
    pub index_offset: ULONG,
    pub unk2: DWORD,
    pub names_offset: ULONG,
    pub names_len: ULONG,
}

#[repr(C)]
pub struct guid_index {
    pub guid: GUID,
    pub data_offset: ULONG,
    pub data_len: ULONG,
    pub rosterindex: ULONG,
}

#[repr(C)]
pub struct wndclass_redirect_data {
    pub size: ULONG,
    pub res: DWORD,
    pub name_len: ULONG,
    pub name_offset: ULONG,
    pub module_len: ULONG,
    pub module_offset: ULONG,
}

#[repr(C)]
pub struct dllredirect_data {
    pub size: ULONG,
    pub unk: ULONG,
    pub res: [DWORD; 3],
}

#[repr(C)]
pub struct tlibredirect_data {
    pub size: ULONG,
    pub res: DWORD,
    pub name_len: ULONG,
    pub name_offset: ULONG,
    pub langid: LANGID,
    pub flags: WORD,
    pub help_len: ULONG,
    pub help_offset: ULONG,
    pub major_version: WORD,
    pub minor_version: WORD,
}

#[repr(C)]
pub struct comclassredirect_data {
    pub size: ULONG,
    pub flags: ULONG,
    pub model: DWORD,
    pub clsid: GUID,
    pub alias: GUID,
    pub clsid2: GUID,
    pub tlbid: GUID,
    pub name_len: ULONG,
    pub name_offset: ULONG,
    pub progid_len: ULONG,
    pub progid_offset: ULONG,
    pub clrdata_len: ULONG,
    pub clrdata_offset: ULONG,
    pub miscstatus: DWORD,
    pub miscstatuscontent: DWORD,
    pub miscstatusthumbnail: DWORD,
    pub miscstatusicon: DWORD,
    pub miscstatusdocprint: DWORD,
}

#[repr(C)]
pub struct ifacepsredirect_data {
    pub size: ULONG,
    pub mask: DWORD,
    pub iid: GUID,
    pub nummethods: ULONG,
    pub tlbid: GUID,
    pub base: GUID,
    pub name_len: ULONG,
    pub name_offset: ULONG,
}

#[repr(C)]
pub struct clrsurrogate_data {
    pub size: ULONG,
    pub res: DWORD,
    pub clsid: GUID,
    pub version_offset: ULONG,
    pub version_len: ULONG,
    pub name_offset: ULONG,
    pub name_len: ULONG,
}

#[repr(C)]
pub struct clrclass_data {
    pub size: ULONG,
    pub res: [DWORD; 2],
    pub module_len: ULONG,
    pub module_offset: ULONG,
    pub name_len: ULONG,
    pub name_offset: ULONG,
    pub version_len: ULONG,
    pub version_offset: ULONG,
    pub res2: [DWORD; 2],
}

#[repr(C)]
pub struct progidredirect_data {
    pub size: ULONG,
    pub reserved: DWORD,
    pub clsid_offset: ULONG,
}

// ── entity (tagged union over redirect kinds) ───────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
pub struct progids {
    pub progids: *mut *mut WCHAR,
    pub num: u32,
    pub allocated: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct entity_typelib {
    pub tlbid: *mut WCHAR,
    pub helpdir: *mut WCHAR,
    pub flags: WORD,
    pub major: WORD,
    pub minor: WORD,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct entity_comclass {
    pub clsid: *mut WCHAR,
    pub tlbid: *mut WCHAR,
    pub progid: *mut WCHAR,
    pub name: *mut WCHAR,
    pub version: *mut WCHAR,
    pub model: DWORD,
    pub miscstatus: DWORD,
    pub miscstatuscontent: DWORD,
    pub miscstatusthumbnail: DWORD,
    pub miscstatusicon: DWORD,
    pub miscstatusdocprint: DWORD,
    pub progids: progids,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct entity_ifaceps {
    pub iid: *mut WCHAR,
    pub base: *mut WCHAR,
    pub tlib: *mut WCHAR,
    pub name: *mut WCHAR,
    pub ps32: *mut WCHAR,
    pub mask: DWORD,
    pub nummethods: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct entity_class {
    pub name: *mut WCHAR,
    pub versioned: BOOL,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct entity_clrsurrogate {
    pub name: *mut WCHAR,
    pub clsid: *mut WCHAR,
    pub version: *mut WCHAR,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct entity_settings {
    pub name: *mut WCHAR,
    pub value: *mut WCHAR,
    pub ns: *mut WCHAR,
}

#[repr(C)]
pub union entity_u {
    pub typelib: entity_typelib,
    pub comclass: entity_comclass,
    pub ifaceps: entity_ifaceps,
    pub class: entity_class,
    pub clrsurrogate: entity_clrsurrogate,
    pub settings: entity_settings,
}

#[repr(C)]
pub struct entity {
    pub kind: DWORD,
    pub u: entity_u,
}

#[repr(C)]
pub struct entity_array {
    pub base: *mut entity,
    pub num: u32,
    pub allocated: u32,
}

#[repr(C)]
pub struct dll_redirect {
    pub name: *mut WCHAR,
    pub hash: *mut WCHAR,
    pub entities: entity_array,
}

pub type assembly_type = u32;
pub const APPLICATION_MANIFEST: assembly_type = 0;
pub const ASSEMBLY_MANIFEST: assembly_type = 1;
pub const ASSEMBLY_SHARED_MANIFEST: assembly_type = 2;

#[repr(C)]
pub struct assembly {
    pub type_: assembly_type,
    pub id: assembly_identity,
    pub manifest: file_info,
    pub directory: *mut WCHAR,
    pub no_inherit: BOOL,
    pub dlls: *mut dll_redirect,
    pub num_dlls: u32,
    pub allocated_dlls: u32,
    pub entities: entity_array,
    pub compat_contexts: *mut COMPATIBILITY_CONTEXT_ELEMENT,
    pub num_compat_contexts: ULONG,
    pub run_level: ACTCTX_REQUESTED_RUN_LEVEL,
    pub ui_access: ULONG,
}

// ── context_sections bitflags ───────────────────────────────────────
pub const WINDOWCLASS_SECTION: DWORD = 1;
pub const DLLREDIRECT_SECTION: DWORD = 2;
pub const TLIBREDIRECT_SECTION: DWORD = 4;
pub const SERVERREDIRECT_SECTION: DWORD = 8;
pub const IFACEREDIRECT_SECTION: DWORD = 16;
pub const CLRSURROGATES_SECTION: DWORD = 32;
pub const PROGIDREDIRECT_SECTION: DWORD = 64;

// ── ASSEMBLY_STORAGE_MAP (__REACTOS__ fields of ACTIVATION_CONTEXT) ──
#[repr(C)]
pub struct ASSEMBLY_STORAGE_MAP_ENTRY {
    pub Flags: ULONG,
    pub DosPath: UNICODE_STRING,
    pub Handle: HANDLE,
}
pub type PASSEMBLY_STORAGE_MAP_ENTRY = *mut ASSEMBLY_STORAGE_MAP_ENTRY;

#[repr(C)]
pub struct ASSEMBLY_STORAGE_MAP {
    pub Flags: ULONG,
    pub AssemblyCount: ULONG,
    pub AssemblyArray: *mut PASSEMBLY_STORAGE_MAP_ENTRY,
}

// Opaque: the activation-context blob the loader hands to the PEB.
pub type PACTIVATION_CONTEXT_DATA = PVOID;

#[repr(C)]
pub struct ACTIVATION_CONTEXT {
    pub magic: ULONG,
    pub ref_count: LONG,
    // __REACTOS__ fields
    pub Flags: ULONG,
    pub Links: LIST_ENTRY,
    pub ActivationContextData: PACTIVATION_CONTEXT_DATA,
    pub NotificationRoutine: PVOID,
    pub NotificationContext: PVOID,
    pub SentNotifications: [ULONG; 8],
    pub DisabledNotifications: [ULONG; 8],
    pub StorageMap: ASSEMBLY_STORAGE_MAP,
    pub InlineStorageMapEntries: PASSEMBLY_STORAGE_MAP_ENTRY,
    pub StackTraceIndex: ULONG,
    pub StackTraces: [[PVOID; 4]; 4],
    // common fields
    pub config: file_info,
    pub appdir: file_info,
    pub assemblies: *mut assembly,
    pub num_assemblies: u32,
    pub allocated_assemblies: u32,
    pub sections: DWORD,
    pub wndclass_section: *mut strsection_header,
    pub dllredirect_section: *mut strsection_header,
    pub progid_section: *mut strsection_header,
    pub tlib_section: *mut guidsection_header,
    pub comserver_section: *mut guidsection_header,
    pub ifaceps_section: *mut guidsection_header,
    pub clrsurrogate_section: *mut guidsection_header,
}
pub type PACTIVATION_CONTEXT = *mut ACTIVATION_CONTEXT;
// rustconv draft uses the bindgen-style underscore name in some sigs.
pub type _ACTIVATION_CONTEXT = ACTIVATION_CONTEXT;
pub type _ACTCTX_SECTION_KEYED_DATA = ACTCTX_SECTION_KEYED_DATA;

// ── activation-context stack frame (ndk/rtltypes.h; ddk bindgen only
//    has an opaque stub for these). ──────────────────────────────────
#[repr(C)]
pub struct RTL_ACTIVATION_CONTEXT_STACK_FRAME {
    pub Previous: *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME,
    pub ActivationContext: PACTIVATION_CONTEXT,
    pub Flags: ULONG,
}
pub type PRTL_ACTIVATION_CONTEXT_STACK_FRAME = *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME;

#[repr(C)]
pub struct RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED {
    pub Size: SIZE_T,
    pub Format: ULONG,
    pub Frame: RTL_ACTIVATION_CONTEXT_STACK_FRAME,
    pub Extra1: PVOID,
    pub Extra2: PVOID,
    pub Extra3: PVOID,
    pub Extra4: PVOID,
}
pub type PRTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED =
    *mut RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED;

#[repr(C)]
pub struct actctx_loader {
    pub actctx: *mut ACTIVATION_CONTEXT,
    pub dependencies: *mut assembly_identity,
    pub num_dependencies: u32,
    pub allocated_dependencies: u32,
}

#[repr(C)]
pub struct olemisc_entry {
    pub name: *const WCHAR,
    pub value: OLEMISC,
}

// These hold raw pointers into read-only static string data; they back
// `static` tables (empty_xmlstr, olemisc_values) so they must be Sync.
unsafe impl Sync for olemisc_entry {}
unsafe impl Sync for xmlstr_t {}

// ════════════════════════════════════════════════════════════════════
//  Compile-time size assertions for the ON-DISK section structs.
//  (sizes are arch-independent: all fields are 32-bit or GUID/LANGID.)
// ════════════════════════════════════════════════════════════════════
const _: () = assert!(core::mem::size_of::<GUID>() == 16);
const _: () = assert!(core::mem::size_of::<strsection_header>() == 44);
const _: () = assert!(core::mem::size_of::<string_index>() == 24);
const _: () = assert!(core::mem::size_of::<guidsection_header>() == 40);
const _: () = assert!(core::mem::size_of::<guid_index>() == 28);
const _: () = assert!(core::mem::size_of::<wndclass_redirect_data>() == 24);
const _: () = assert!(core::mem::size_of::<dllredirect_data>() == 20);
const _: () = assert!(core::mem::size_of::<tlibredirect_data>() == 32);
const _: () = assert!(core::mem::size_of::<comclassredirect_data>() == 120);
const _: () = assert!(core::mem::size_of::<ifacepsredirect_data>() == 68);
const _: () = assert!(core::mem::size_of::<clrsurrogate_data>() == 40);
const _: () = assert!(core::mem::size_of::<clrclass_data>() == 44);
const _: () = assert!(core::mem::size_of::<progidredirect_data>() == 12);
const _: () = assert!(core::mem::size_of::<COMPATIBILITY_CONTEXT_ELEMENT>() == 20);
