use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ACTIVATION_CONTEXT_STACK, FILE_BOTH_DIR_INFORMATION, FILE_END_OF_FILE_INFORMATION, GUID, IMAGE_RESOURCE_DATA_ENTRY, IMAGE_RESOURCE_DIRECTORY, IMAGE_RESOURCE_DIRECTORY_ENTRY, _IO_STATUS_BLOCK, LARGE_INTEGER, LDR_DATA_TABLE_ENTRY, LDR_RESOURCE_INFO, LdrAccessResource, LdrFindResource_U, LdrLockLoaderLock, LdrUnlockLoaderLock, NtClose, NtOpenFile, NtQueryDirectoryFile, NtQueryInformationFile, NtReadFile, _OBJECT_ATTRIBUTES, PACTIVATION_CONTEXT_STACK, RtlCompareUnicodeStrings, RtlCreateUnicodeString, RtlDetermineDosPathNameType_U, RtlDosPathNameToNtPathName_U, RtlEqualUnicodeString, RtlFreeHeap, RtlFreeUnicodeString, RtlGUIDFromString, RtlGetFullPathName_U, RtlHashUnicodeString, RtlImageDirectoryEntryToData, RtlImageNtHeader, RtlInitUnicodeString, RtlIsTextUnicode, RtlUnicodeStringToInteger, RtlUniform, RtlUpcaseUnicodeString, RtlUshortByteSwap, TEB};

// The byte-exact private/public type substrate + the file-static string
// constants (machine-extracted). LPCWSTR/LPWSTR/PACTIVATION_CONTEXT and
// the ActCtx structs come from crate::types (the kernel bindgen lacks
// the user-mode ActCtx surface).
use crate::types::*;
use crate::strings::*;
use crate::shim::*;
// The draft uses the bindgen underscore name; unify both spellings on
// ddk's _UNICODE_STRING (layout-identical to base's UNICODE_STRING) so
// the ddk Rtl*/Nt* signatures line up.
use ntkrnl_sys::ddk::{_UNICODE_STRING, _UNICODE_STRING as UNICODE_STRING};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// ── file-static globals (actctx.c; the "preamble" rustconv couldn't
//    emit — hand-written here). ───────────────────────────────────────
static empty_xmlstr: xmlstr_t = xmlstr_t { ptr: core::ptr::null(), len: 0 };

static mut system_actctx: ACTIVATION_CONTEXT = ACTIVATION_CONTEXT {
    magic: ACTCTX_MAGIC,
    ref_count: 1,
    Flags: 0,
    Links: LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() },
    ActivationContextData: core::ptr::null_mut(),
    NotificationRoutine: core::ptr::null_mut(),
    NotificationContext: core::ptr::null_mut(),
    SentNotifications: [0; 8],
    DisabledNotifications: [0; 8],
    StorageMap: ASSEMBLY_STORAGE_MAP { Flags: 0, AssemblyCount: 0, AssemblyArray: core::ptr::null_mut() },
    InlineStorageMapEntries: core::ptr::null_mut(),
    StackTraceIndex: 0,
    StackTraces: [[core::ptr::null_mut(); 4]; 4],
    config: file_info { type_: 0, info: core::ptr::null_mut() },
    appdir: file_info { type_: 0, info: core::ptr::null_mut() },
    assemblies: core::ptr::null_mut(),
    num_assemblies: 0,
    allocated_assemblies: 0,
    sections: 0,
    wndclass_section: core::ptr::null_mut(),
    dllredirect_section: core::ptr::null_mut(),
    progid_section: core::ptr::null_mut(),
    tlib_section: core::ptr::null_mut(),
    comserver_section: core::ptr::null_mut(),
    ifaceps_section: core::ptr::null_mut(),
    clrsurrogate_section: core::ptr::null_mut(),
};
static mut process_actctx: *mut ACTIVATION_CONTEXT = core::ptr::addr_of_mut!(system_actctx);
static mut implicit_actctx: *mut ACTIVATION_CONTEXT = core::ptr::addr_of_mut!(system_actctx);

// RtlpNotAllowingMultipleActivation (actctx.c file global, __REACTOS__).
#[no_mangle]
pub static mut RtlpNotAllowingMultipleActivation: BOOLEAN = 0;

// olemisc_values — sorted name->OLEMISC table (get_olemisc_value bsearch).
static olemisc_values: [olemisc_entry; 22] = [
    olemisc_entry { name: activatewhenvisibleW.as_ptr(), value: OLEMISC_ACTIVATEWHENVISIBLE },
    olemisc_entry { name: actslikebuttonW.as_ptr(), value: OLEMISC_ACTSLIKEBUTTON },
    olemisc_entry { name: actslikelabelW.as_ptr(), value: OLEMISC_ACTSLIKELABEL },
    olemisc_entry { name: alignableW.as_ptr(), value: OLEMISC_ALIGNABLE },
    olemisc_entry { name: alwaysrunW.as_ptr(), value: OLEMISC_ALWAYSRUN },
    olemisc_entry { name: canlinkbyole1W.as_ptr(), value: OLEMISC_CANLINKBYOLE1 },
    olemisc_entry { name: cantlinkinsideW.as_ptr(), value: OLEMISC_CANTLINKINSIDE },
    olemisc_entry { name: ignoreactivatewhenvisibleW.as_ptr(), value: OLEMISC_IGNOREACTIVATEWHENVISIBLE },
    olemisc_entry { name: imemodeW.as_ptr(), value: OLEMISC_IMEMODE },
    olemisc_entry { name: insertnotreplaceW.as_ptr(), value: OLEMISC_INSERTNOTREPLACE },
    olemisc_entry { name: insideoutW.as_ptr(), value: OLEMISC_INSIDEOUT },
    olemisc_entry { name: invisibleatruntimeW.as_ptr(), value: OLEMISC_INVISIBLEATRUNTIME },
    olemisc_entry { name: islinkobjectW.as_ptr(), value: OLEMISC_ISLINKOBJECT },
    olemisc_entry { name: nouiactivateW.as_ptr(), value: OLEMISC_NOUIACTIVATE },
    olemisc_entry { name: onlyiconicW.as_ptr(), value: OLEMISC_ONLYICONIC },
    olemisc_entry { name: recomposeonresizeW.as_ptr(), value: OLEMISC_RECOMPOSEONRESIZE },
    olemisc_entry { name: renderingisdeviceindependentW.as_ptr(), value: OLEMISC_RENDERINGISDEVICEINDEPENDENT },
    olemisc_entry { name: setclientsitefirstW.as_ptr(), value: OLEMISC_SETCLIENTSITEFIRST },
    olemisc_entry { name: simpleframeW.as_ptr(), value: OLEMISC_SIMPLEFRAME },
    olemisc_entry { name: staticW.as_ptr(), value: OLEMISC_STATIC },
    olemisc_entry { name: supportsmultilevelundoW.as_ptr(), value: OLEMISC_SUPPORTSMULTILEVELUNDO },
    olemisc_entry { name: wantstomenumergeW.as_ptr(), value: OLEMISC_WANTSTOMENUMERGE },
];

unsafe fn strdupW(str: *const WCHAR) -> *mut WCHAR {
    let ptr: *mut WCHAR = RtlAllocateHeap(GetProcessHeap(), 0, (wcslen(str) + 1) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    wcscpy(ptr, str)
}

unsafe fn xmlstrdupW(str: *const xmlstr_t) -> *mut WCHAR {
    let strW: *mut WCHAR = RtlAllocateHeap(
        GetProcessHeap(),
        0,
        (((*str).len + 1) as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as *mut WCHAR;

    if !strW.is_null() {
        core::ptr::copy_nonoverlapping(
            (*str).ptr as *const u8,
            strW as *mut u8,
            (*str).len as usize * core::mem::size_of::<WCHAR>(),
        );
        *strW.add((*str).len as usize) = 0;
    }
    strW
}

unsafe fn xmlstr_cmp(xmlstr: *const xmlstr_t, str: *const WCHAR) -> BOOL {
    ((wcsncmp((*xmlstr).ptr, str, (*xmlstr).len as usize) == 0)
        && (*str.add((*xmlstr).len as usize) == 0)) as BOOL
}

unsafe fn xmlstr_cmpi(xmlstr: *const xmlstr_t, str: *const WCHAR) -> BOOL {
    let result = wcsnicmp((*xmlstr).ptr, str, (*xmlstr).len as usize) == 0
        && *str.add((*xmlstr).len as usize) == 0;
    if result { 1 } else { 0 }
}

unsafe fn xml_attr_cmp(attr: *const xml_attr, str: *const WCHAR) -> BOOL {
    xmlstr_cmp(core::ptr::addr_of!((*attr).name), str)
}

unsafe fn xml_name_cmp(elem1: *const xml_elem, elem2: *const xml_elem) -> BOOL {
    (((*elem1).name.len == (*elem2).name.len
        && (*elem1).ns.len == (*elem2).ns.len
        && wcsncmp((*elem1).name.ptr, (*elem2).name.ptr, (*elem1).name.len as usize) == 0
        && wcsncmp((*elem1).ns.ptr, (*elem2).ns.ptr, (*elem1).ns.len as usize) == 0) as BOOL)
}

unsafe fn xml_elem_cmp(elem: *const xml_elem, str: *const WCHAR, namespace: *const WCHAR) -> BOOL {
    if xmlstr_cmp(core::ptr::addr_of!((*elem).name), str) == 0 {
        return 0;
    }
    if xmlstr_cmp(core::ptr::addr_of!((*elem).ns), namespace) != 0 {
        return 1;
    }
    if wcscmp(namespace, asmv1W.as_ptr()) == 0 {
        if xmlstr_cmp(core::ptr::addr_of!((*elem).ns), asmv2W.as_ptr()) != 0 {
            return 1;
        }
        if xmlstr_cmp(core::ptr::addr_of!((*elem).ns), asmv3W.as_ptr()) != 0 {
            return 1;
        }
    } else if wcscmp(namespace, asmv2W.as_ptr()) == 0 {
        if xmlstr_cmp(core::ptr::addr_of!((*elem).ns), asmv3W.as_ptr()) != 0 {
            return 1;
        }
    }
    0
}

unsafe fn isxmlspace(ch: WCHAR) -> BOOL {
    (ch == b' ' as WCHAR || ch == b'\r' as WCHAR || ch == b'\n' as WCHAR || ch == b'\t' as WCHAR) as BOOL
}

unsafe fn debugstr_xmlstr(str: *const xmlstr_t) -> *const core::ffi::c_char {
    debugstr_wn((*str).ptr, (*str).len)
}

unsafe fn debugstr_xml_elem(_elem: *const xml_elem) -> *const core::ffi::c_char {
    wine_dbg_sprintf!(
        b"%s ns %s\0".as_ptr() as *const core::ffi::c_char,
        debugstr_wn((*elem).name.ptr, (*elem).name.len),
        debugstr_wn((*elem).ns.ptr, (*elem).ns.len),
    )
}

unsafe fn debugstr_xml_attr(_attr: *const xml_attr) -> *const core::ffi::c_char {
    wine_dbg_sprintf!(
        b"%s=%s\0".as_ptr() as *const core::ffi::c_char,
        debugstr_wn((*attr).name.ptr, (*attr).name.len),
        debugstr_wn((*attr).value.ptr, (*attr).value.len),
    )
}

unsafe fn debugstr_version(_ver: *const assembly_version) -> *const core::ffi::c_char {
    wine_dbg_sprintf!(
        b"%u.%u.%u.%u\0".as_ptr() as *const core::ffi::c_char,
        (*ver).major as core::ffi::c_uint,
        (*ver).minor as core::ffi::c_uint,
        (*ver).build as core::ffi::c_uint,
        (*ver).revision as core::ffi::c_uint,
    )
}

unsafe fn get_module_filename( module: HMODULE, str: *mut _UNICODE_STRING, extra_len: core::ffi::c_uint ) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut magic: ULONG_PTR = 0;
    let mut pldr: *mut LDR_DATA_TABLE_ENTRY = core::ptr::null_mut();

    LdrLockLoaderLock(0, core::ptr::null_mut(), core::ptr::addr_of_mut!(magic) as *mut _);
    status = LdrFindEntryForAddress(module, &mut pldr);
    if status == STATUS_SUCCESS {
        (*str).Buffer = RtlAllocateHeap(
            GetProcessHeap(),
            0,
            ((*pldr).FullDllName.Length as usize) + (extra_len as usize) + core::mem::size_of::<u16>(),
        ) as *mut u16;
        if !(*str).Buffer.is_null() {
            memcpy(
                (*str).Buffer as *mut core::ffi::c_void,
                (*pldr).FullDllName.Buffer as *const core::ffi::c_void,
                (*pldr).FullDllName.Length as usize + core::mem::size_of::<u16>(),
            );
            (*str).Length = (*pldr).FullDllName.Length;
            (*str).MaximumLength = (*pldr).FullDllName.Length + (extra_len as u16) + core::mem::size_of::<u16>() as u16;
        } else {
            status = STATUS_NO_MEMORY;
        }
    }
    LdrUnlockLoaderLock(0, magic as _);
    status
}

unsafe fn add_assembly(actctx: *mut ACTIVATION_CONTEXT, at: assembly_type) -> *mut assembly {
    let assembly: *mut assembly;

    if (*actctx).num_assemblies == (*actctx).allocated_assemblies {
        let ptr: *mut core::ffi::c_void;
        let new_count: core::ffi::c_uint;
        if !(*actctx).assemblies.is_null() {
            new_count = (*actctx).allocated_assemblies * 2;
            ptr = RtlReAllocateHeap(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (*actctx).assemblies as *mut core::ffi::c_void,
                new_count as usize * core::mem::size_of::<self::assembly>(),
            );
        } else {
            new_count = 4;
            ptr = RtlAllocateHeap(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                new_count as usize * core::mem::size_of::<self::assembly>(),
            );
        }
        if ptr.is_null() {
            return core::ptr::null_mut();
        }
        (*actctx).assemblies = ptr as *mut self::assembly;
        (*actctx).allocated_assemblies = new_count;
    }

    assembly = (*actctx).assemblies.add((*actctx).num_assemblies as usize);
    (*actctx).num_assemblies += 1;
    (*assembly).type_ = at;
    assembly
}

unsafe fn add_dll_redirect(assembly: *mut assembly) -> *mut dll_redirect {
    if (*assembly).num_dlls == (*assembly).allocated_dlls {
        let ptr: *mut core::ffi::c_void;
        let new_count: core::ffi::c_uint;
        if !(*assembly).dlls.is_null() {
            new_count = (*assembly).allocated_dlls * 2;
            ptr = RtlReAllocateHeap(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (*assembly).dlls as *mut core::ffi::c_void,
                new_count as usize * core::mem::size_of::<dll_redirect>(),
            );
        } else {
            new_count = 4;
            ptr = RtlAllocateHeap(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                new_count as usize * core::mem::size_of::<dll_redirect>(),
            );
        }
        if ptr.is_null() {
            return core::ptr::null_mut();
        }
        (*assembly).dlls = ptr as *mut dll_redirect;
        (*assembly).allocated_dlls = new_count;
    }
    let result = (*assembly).dlls.add((*assembly).num_dlls as usize);
    (*assembly).num_dlls += 1;
    result
}

unsafe fn add_compat_context(assembly: *mut assembly) -> PCOMPATIBILITY_CONTEXT_ELEMENT {
    let ptr: *mut core::ffi::c_void;
    if (*assembly).num_compat_contexts != 0 {
        let new_count = (*assembly).num_compat_contexts + 1;
        ptr = RtlReAllocateHeap(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*assembly).compat_contexts as *mut core::ffi::c_void,
            new_count as usize * core::mem::size_of::<COMPATIBILITY_CONTEXT_ELEMENT>(),
        );
    } else {
        ptr = RtlAllocateHeap(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<COMPATIBILITY_CONTEXT_ELEMENT>(),
        );
    }
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    (*assembly).compat_contexts = ptr as *mut COMPATIBILITY_CONTEXT_ELEMENT;
    let result = (*assembly).compat_contexts.add((*assembly).num_compat_contexts as usize);
    (*assembly).num_compat_contexts += 1;
    result
}

unsafe fn free_assembly_identity(ai: *mut assembly_identity) {
    RtlFreeHeap(GetProcessHeap(), 0, (*ai).name as *mut _);
    RtlFreeHeap(GetProcessHeap(), 0, (*ai).arch as *mut _);
    RtlFreeHeap(GetProcessHeap(), 0, (*ai).public_key as *mut _);
    RtlFreeHeap(GetProcessHeap(), 0, (*ai).language as *mut _);
    RtlFreeHeap(GetProcessHeap(), 0, (*ai).type_ as *mut _);
}

unsafe fn add_entity(array: *mut entity_array, kind: DWORD) -> *mut entity {
    let entity: *mut entity;

    if (*array).num == (*array).allocated {
        let ptr: *mut core::ffi::c_void;
        let new_count: core::ffi::c_uint;
        if !(*array).base.is_null() {
            new_count = (*array).allocated * 2;
            ptr = RtlReAllocateHeap(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (*array).base as *mut core::ffi::c_void,
                new_count as usize * core::mem::size_of::<entity>(),
            );
        } else {
            new_count = 4;
            ptr = RtlAllocateHeap(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                new_count as usize * core::mem::size_of::<entity>(),
            );
        }
        if ptr.is_null() {
            return core::ptr::null_mut();
        }
        (*array).base = ptr as *mut entity;
        (*array).allocated = new_count;
    }
    entity = (*array).base.add((*array).num as usize);
    (*array).num += 1;
    (*entity).kind = kind;
    entity
}

unsafe fn free_entity_array(array: *mut entity_array) {
    let mut i: core::ffi::c_uint = 0;
    while i < (*array).num {
        let entity = (*array).base.offset(i as isize);
        match (*entity).kind {
            ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION => {
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.comclass.clsid as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.comclass.tlbid as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.comclass.progid as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.comclass.name as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.comclass.version as *mut _);
                let mut j: core::ffi::c_uint = 0;
                while j < (*entity).u.comclass.progids.num {
                    RtlFreeHeap(GetProcessHeap(), 0, *(*entity).u.comclass.progids.progids.offset(j as isize) as *mut _);
                    j += 1;
                }
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.comclass.progids.progids as *mut _);
            }
            ACTIVATION_CONTEXT_SECTION_COM_INTERFACE_REDIRECTION => {
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.ifaceps.iid as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.ifaceps.base as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.ifaceps.ps32 as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.ifaceps.name as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.ifaceps.tlib as *mut _);
            }
            ACTIVATION_CONTEXT_SECTION_COM_TYPE_LIBRARY_REDIRECTION => {
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.typelib.tlbid as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.typelib.helpdir as *mut _);
            }
            ACTIVATION_CONTEXT_SECTION_WINDOW_CLASS_REDIRECTION => {
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.class.name as *mut _);
            }
            ACTIVATION_CONTEXT_SECTION_CLR_SURROGATES => {
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.clrsurrogate.name as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.clrsurrogate.clsid as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.clrsurrogate.version as *mut _);
            }
            ACTIVATION_CONTEXT_SECTION_APPLICATION_SETTINGS => {
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.settings.name as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.settings.value as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.settings.ns as *mut _);
            }
            _ => {
                FIXME!("Unknown entity kind %d\n", (*entity).kind);
            }
        }
        i += 1;
    }
    RtlFreeHeap(GetProcessHeap(), 0, (*array).base as *mut _);
}

unsafe fn is_matching_string(str1: *const WCHAR, str2: *const WCHAR) -> BOOL {
    if str1.is_null() {
        return if str2.is_null() { 1 } else { 0 };
    }
    if !str2.is_null()
        && RtlCompareUnicodeStrings(str1, wcslen(str1) as _, str2, wcslen(str2) as _, 1u8) == 0
    {
        1
    } else {
        0
    }
}

unsafe fn is_matching_identity(id1: *const assembly_identity, id2: *const assembly_identity) -> BOOL {
    if is_matching_string((*id1).name, (*id2).name) == 0 {
        return 0;
    }
    if is_matching_string((*id1).arch, (*id2).arch) == 0 {
        return 0;
    }
    if is_matching_string((*id1).public_key, (*id2).public_key) == 0 {
        return 0;
    }

    if !(*id1).language.is_null()
        && !(*id2).language.is_null()
        && is_matching_string((*id1).language, (*id2).language) == 0
    {
        if wcscmp(wildcardW.as_ptr(), (*id1).language) != 0
            && wcscmp(wildcardW.as_ptr(), (*id2).language) != 0
        {
            return 0;
        }
    }
    if (*id1).version.major != (*id2).version.major {
        return 0;
    }
    if (*id1).version.minor != (*id2).version.minor {
        return 0;
    }
    if (*id1).version.build > (*id2).version.build {
        return 0;
    }
    if (*id1).version.build == (*id2).version.build
        && (*id1).version.revision > (*id2).version.revision
    {
        return 0;
    }
    1
}

unsafe fn add_dependent_assembly_id(acl: *mut actctx_loader, ai: *mut assembly_identity) -> BOOL {
    let mut i: core::ffi::c_uint = 0;

    while i < (*(*acl).actctx).num_assemblies {
        if is_matching_identity(ai, core::ptr::addr_of!((*(*(*acl).actctx).assemblies.offset(i as isize)).id)) != 0 {
            return 1;
        }
        i += 1;
    }

    i = 0;
    while i < (*acl).num_dependencies {
        if is_matching_identity(ai, (*acl).dependencies.offset(i as isize)) != 0 {
            return 1;
        }
        i += 1;
    }

    if (*acl).num_dependencies == (*acl).allocated_dependencies {
        let ptr: *mut core::ffi::c_void;
        let new_count: core::ffi::c_uint;
        if !(*acl).dependencies.is_null() {
            new_count = (*acl).allocated_dependencies * 2;
            ptr = RtlReAllocateHeap(GetProcessHeap(), 0, (*acl).dependencies as *mut core::ffi::c_void,
                                    new_count as usize * core::mem::size_of::<assembly_identity>());
        } else {
            new_count = 4;
            ptr = RtlAllocateHeap(GetProcessHeap(), 0, new_count as usize * core::mem::size_of::<assembly_identity>());
        }
        if ptr.is_null() {
            return 0;
        }
        (*acl).dependencies = ptr as *mut assembly_identity;
        (*acl).allocated_dependencies = new_count;
    }

    *(*acl).dependencies.offset((*acl).num_dependencies as isize) = *ai;
    (*acl).num_dependencies += 1;

    1
}

unsafe fn free_depend_manifests(acl: *mut actctx_loader) {
    let mut i: core::ffi::c_uint = 0;
    while i < (*acl).num_dependencies {
        free_assembly_identity((*acl).dependencies.add(i as usize));
        i += 1;
    }
    RtlFreeHeap(GetProcessHeap(), 0, (*acl).dependencies as *mut _);
}

unsafe fn build_assembly_dir(ai: *mut assembly_identity) -> *mut WCHAR {
    static undW: [WCHAR; 2] = [b'_' as WCHAR, 0];
    static noneW: [WCHAR; 5] = [b'n' as WCHAR, b'o' as WCHAR, b'n' as WCHAR, b'e' as WCHAR, 0];
    static mskeyW: [WCHAR; 9] = [b'd' as WCHAR, b'e' as WCHAR, b'a' as WCHAR, b'd' as WCHAR, b'b' as WCHAR, b'e' as WCHAR, b'e' as WCHAR, b'f' as WCHAR, 0];

    let arch: *const WCHAR = if !(*ai).arch.is_null() { (*ai).arch } else { noneW.as_ptr() };
    let key: *const WCHAR = if !(*ai).public_key.is_null() { (*ai).public_key } else { noneW.as_ptr() };
    let lang: *const WCHAR = if !(*ai).language.is_null() { (*ai).language } else { noneW.as_ptr() };
    let name: *const WCHAR = if !(*ai).name.is_null() { (*ai).name } else { noneW.as_ptr() };

    let size: SIZE_T = (wcslen(arch) + 1 + wcslen(name) + 1 + wcslen(key) + 24 + 1 +
        wcslen(lang) + 1) * core::mem::size_of::<WCHAR>() + core::mem::size_of_val(&mskeyW);

    let ret: *mut WCHAR = RtlAllocateHeap(GetProcessHeap(), 0, size) as *mut WCHAR;
    if ret.is_null() {
        return core::ptr::null_mut();
    }

    wcscpy(ret, arch);
    wcscat(ret, undW.as_ptr());
    wcscat(ret, name);
    wcscat(ret, undW.as_ptr());
    wcscat(ret, key);
    wcscat(ret, undW.as_ptr());
    swprintf(ret.add(wcslen(ret)), size - wcslen(ret), version_formatW.as_ptr(),
        (*ai).version.major as u32, (*ai).version.minor as u32, (*ai).version.build as u32, (*ai).version.revision as u32);
    wcscat(ret, undW.as_ptr());
    wcscat(ret, lang);
    wcscat(ret, undW.as_ptr());
    wcscat(ret, mskeyW.as_ptr());
    ret
}

unsafe fn append_string(buffer: *mut WCHAR, prefix: *const WCHAR, str: *const WCHAR) {
    let mut p = buffer;

    if str.is_null() {
        return;
    }
    wcscat(buffer, prefix);
    p = p.add(wcslen(p) as usize);
    *p = b'"' as WCHAR;
    p = p.add(1);
    wcscpy(p, str);
    p = p.add(wcslen(p) as usize);
    *p = b'"' as WCHAR;
    p = p.add(1);
    *p = 0;
}

unsafe fn build_assembly_id(ai: *const assembly_identity) -> *mut WCHAR {
    static archW: [WCHAR; 24] = [
        b',' as WCHAR, b'p' as WCHAR, b'r' as WCHAR, b'o' as WCHAR, b'c' as WCHAR,
        b'e' as WCHAR, b's' as WCHAR, b's' as WCHAR, b'o' as WCHAR, b'r' as WCHAR,
        b'A' as WCHAR, b'r' as WCHAR, b'c' as WCHAR, b'h' as WCHAR, b'i' as WCHAR,
        b't' as WCHAR, b'e' as WCHAR, b'c' as WCHAR, b't' as WCHAR, b'u' as WCHAR,
        b'r' as WCHAR, b'e' as WCHAR, b'=' as WCHAR, 0,
    ];
    static public_keyW: [WCHAR; 17] = [
        b',' as WCHAR, b'p' as WCHAR, b'u' as WCHAR, b'b' as WCHAR, b'l' as WCHAR,
        b'i' as WCHAR, b'c' as WCHAR, b'K' as WCHAR, b'e' as WCHAR, b'y' as WCHAR,
        b'T' as WCHAR, b'o' as WCHAR, b'k' as WCHAR, b'e' as WCHAR, b'n' as WCHAR,
        b'=' as WCHAR,
     0,];
    static typeW2: [WCHAR; 7] = [
        b',' as WCHAR, b't' as WCHAR, b'y' as WCHAR, b'p' as WCHAR, b'e' as WCHAR,
        b'=' as WCHAR, 0,
    ];
    static versionW2: [WCHAR; 10] = [
        b',' as WCHAR, b'v' as WCHAR, b'e' as WCHAR, b'r' as WCHAR, b's' as WCHAR,
        b'i' as WCHAR, b'o' as WCHAR, b'n' as WCHAR, b'=' as WCHAR, 0,
    ];

    let mut version: [WCHAR; 64] = [0; 64];
    let ret: *mut WCHAR;
    let mut size: SIZE_T = 0;

    swprintf(
        version.as_mut_ptr(),
        version.len(),
        version_formatW.as_ptr(),
        (*ai).version.major as core::ffi::c_int,
        (*ai).version.minor as core::ffi::c_int,
        (*ai).version.build as core::ffi::c_int,
        (*ai).version.revision as core::ffi::c_int,
    );
    if !(*ai).name.is_null() {
        size += wcslen((*ai).name) as SIZE_T * core::mem::size_of::<WCHAR>();
    }
    if !(*ai).arch.is_null() {
        size += wcslen(archW.as_ptr()) as SIZE_T + wcslen((*ai).arch) as SIZE_T + 2;
    }
    if !(*ai).public_key.is_null() {
        size += wcslen(public_keyW.as_ptr()) as SIZE_T + wcslen((*ai).public_key) as SIZE_T + 2;
    }
    if !(*ai).type_.is_null() {
        size += wcslen(typeW2.as_ptr()) as SIZE_T + wcslen((*ai).type_) as SIZE_T + 2;
    }
    size += wcslen(versionW2.as_ptr()) as SIZE_T + wcslen(version.as_ptr()) as SIZE_T + 2;

    ret = RtlAllocateHeap(GetProcessHeap(), 0, (size + 1) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
    if ret.is_null() {
        return core::ptr::null_mut();
    }

    if !(*ai).name.is_null() {
        wcscpy(ret, (*ai).name);
    } else {
        *ret = 0;
    }
    append_string(ret, archW.as_ptr(), (*ai).arch);
    append_string(ret, public_keyW.as_ptr(), (*ai).public_key);
    append_string(ret, typeW2.as_ptr(), (*ai).type_);
    append_string(ret, versionW2.as_ptr(), version.as_ptr());
    ret
}

unsafe fn check_actctx(h: HANDLE) -> *mut ACTIVATION_CONTEXT {
    let mut ret: *mut ACTIVATION_CONTEXT = core::ptr::null_mut();
    let actctx: *mut ACTIVATION_CONTEXT = h as *mut ACTIVATION_CONTEXT;

    if h.is_null() || h == INVALID_HANDLE_VALUE as HANDLE {
        return core::ptr::null_mut();
    }

    if (*actctx).magic == ACTCTX_MAGIC {
        ret = actctx;
    }

    ret
}

unsafe fn actctx_addref(actctx: *mut ACTIVATION_CONTEXT) {
    InterlockedIncrement(core::ptr::addr_of_mut!((*actctx).ref_count));
}

unsafe fn actctx_release(actctx: *mut _ACTIVATION_CONTEXT) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*actctx).ref_count)) == 0 {
        let mut i: core::ffi::c_uint = 0;
        while i < (*actctx).num_assemblies {
            let assembly = (*actctx).assemblies.offset(i as isize);
            let mut j: core::ffi::c_uint = 0;
            while j < (*assembly).num_dlls {
                let dll = (*assembly).dlls.offset(j as isize);
                free_entity_array(core::ptr::addr_of_mut!((*dll).entities));
                RtlFreeHeap(GetProcessHeap(), 0, (*dll).name as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*dll).hash as *mut _);
                j += 1;
            }
            RtlFreeHeap(GetProcessHeap(), 0, (*assembly).dlls as *mut _);
            RtlFreeHeap(GetProcessHeap(), 0, (*assembly).manifest.info as *mut _);
            RtlFreeHeap(GetProcessHeap(), 0, (*assembly).directory as *mut _);
            RtlFreeHeap(GetProcessHeap(), 0, (*assembly).compat_contexts as *mut _);
            free_entity_array(core::ptr::addr_of_mut!((*assembly).entities));
            free_assembly_identity(core::ptr::addr_of_mut!((*assembly).id));
            i += 1;
        }
        RtlFreeHeap(GetProcessHeap(), 0, (*actctx).config.info as *mut _);
        RtlFreeHeap(GetProcessHeap(), 0, (*actctx).appdir.info as *mut _);
        RtlFreeHeap(GetProcessHeap(), 0, (*actctx).assemblies as *mut _);
        RtlFreeHeap(GetProcessHeap(), 0, (*actctx).dllredirect_section as *mut _);
        RtlFreeHeap(GetProcessHeap(), 0, (*actctx).wndclass_section as *mut _);
        RtlFreeHeap(GetProcessHeap(), 0, (*actctx).tlib_section as *mut _);
        RtlFreeHeap(GetProcessHeap(), 0, (*actctx).comserver_section as *mut _);
        RtlFreeHeap(GetProcessHeap(), 0, (*actctx).ifaceps_section as *mut _);
        RtlFreeHeap(GetProcessHeap(), 0, (*actctx).clrsurrogate_section as *mut _);
        RtlFreeHeap(GetProcessHeap(), 0, (*actctx).progid_section as *mut _);
        (*actctx).magic = 0;
        RtlFreeHeap(GetProcessHeap(), 0, actctx as *mut _);
    }
}

unsafe fn set_error(xmlbuf: *mut xmlbuf_t) -> BOOL {
    (*xmlbuf).error = 1;
    0
}

unsafe fn is_xmlns_attr(attr: *const xml_attr) -> BOOL {
    let len = wcslen(xmlnsW.as_ptr()) as core::ffi::c_int;
    if ((*attr).name.len as core::ffi::c_int) < len {
        return 0;
    }
    if wcsncmp((*attr).name.ptr, xmlnsW.as_ptr(), len as usize) != 0 {
        return 0;
    }
    if (*attr).name.len as core::ffi::c_int == len
        || *(*attr).name.ptr.offset(len as isize) == b':' as WCHAR
    {
        1
    } else {
        0
    }
}

unsafe fn push_xmlns(xmlbuf: *mut xmlbuf_t, attr: *const xml_attr) {
    let len = wcslen(xmlnsW.as_ptr()) as core::ffi::c_int;
    let ns: *mut xml_attr;

    if (*xmlbuf).ns_pos == MAX_NAMESPACES as core::ffi::c_int - 1 {
        FIXME!("too many namespaces in manifest\n");
        set_error(xmlbuf);
        return;
    }
    ns = core::ptr::addr_of_mut!((*xmlbuf).namespaces[(*xmlbuf).ns_pos as usize]);
    (*xmlbuf).ns_pos += 1;
    (*ns).value = (*attr).value;
    if (*attr).name.len as core::ffi::c_int > len {
        (*ns).name.ptr = (*attr).name.ptr.offset((len + 1) as isize);
        (*ns).name.len = (*attr).name.len - len as core::ffi::c_uint - 1;
    } else {
        (*ns).name = empty_xmlstr;
    }
}

unsafe fn find_xmlns(xmlbuf: *mut xmlbuf_t, name: *const xmlstr_t) -> xmlstr_t {
    let mut i: i32 = (*xmlbuf).ns_pos - 1;
    while i >= 0 {
        if (*xmlbuf).namespaces[i as usize].name.len == (*name).len
            && wcsncmp(
                (*xmlbuf).namespaces[i as usize].name.ptr,
                (*name).ptr,
                (*name).len as usize,
            ) == 0
        {
            return (*xmlbuf).namespaces[i as usize].value;
        }
        i -= 1;
    }
    empty_xmlstr
}

unsafe fn next_xml_attr(xmlbuf: *mut xmlbuf_t, attr: *mut xml_attr, end: *mut BOOL) -> BOOL {
    let mut ptr: *const WCHAR;
    let quote: WCHAR;

    if (*xmlbuf).error != 0 {
        return 0;
    }

    while (*xmlbuf).ptr < (*xmlbuf).end && isxmlspace(*(*xmlbuf).ptr) != 0 {
        (*xmlbuf).ptr = (*xmlbuf).ptr.add(1);
    }

    if (*xmlbuf).ptr == (*xmlbuf).end {
        return set_error(xmlbuf);
    }

    if *(*xmlbuf).ptr == b'/' as WCHAR {
        (*xmlbuf).ptr = (*xmlbuf).ptr.add(1);
        if (*xmlbuf).ptr == (*xmlbuf).end || *(*xmlbuf).ptr != b'>' as WCHAR {
            return set_error(xmlbuf);
        }

        (*xmlbuf).ptr = (*xmlbuf).ptr.add(1);
        *end = 1;
        return 0;
    }

    if *(*xmlbuf).ptr == b'>' as WCHAR {
        (*xmlbuf).ptr = (*xmlbuf).ptr.add(1);
        return 0;
    }

    ptr = (*xmlbuf).ptr;
    while ptr < (*xmlbuf).end && *ptr != b'=' as WCHAR && *ptr != b'>' as WCHAR && isxmlspace(*ptr) == 0 {
        ptr = ptr.add(1);
    }

    if ptr == (*xmlbuf).end {
        return set_error(xmlbuf);
    }

    (*attr).name.ptr = (*xmlbuf).ptr;
    (*attr).name.len = ptr.offset_from((*xmlbuf).ptr) as core::ffi::c_uint;
    (*xmlbuf).ptr = ptr;

    while ptr < (*xmlbuf).end && *ptr != b'=' as WCHAR && isxmlspace(*ptr) != 0 {
        ptr = ptr.add(1);
    }
    if ptr == (*xmlbuf).end || *ptr != b'=' as WCHAR {
        return set_error(xmlbuf);
    }

    ptr = ptr.add(1);
    if ptr == (*xmlbuf).end {
        return set_error(xmlbuf);
    }

    while ptr < (*xmlbuf).end && *ptr != b'"' as WCHAR && *ptr != b'\'' as WCHAR && isxmlspace(*ptr) != 0 {
        ptr = ptr.add(1);
    }

    if ptr == (*xmlbuf).end || (*ptr != b'"' as WCHAR && *ptr != b'\'' as WCHAR) {
        return set_error(xmlbuf);
    }

    quote = *ptr;
    ptr = ptr.add(1);
    (*attr).value.ptr = ptr;
    if ptr == (*xmlbuf).end {
        return set_error(xmlbuf);
    }

    while ptr < (*xmlbuf).end && *ptr != quote {
        ptr = ptr.add(1);
    }
    if ptr == (*xmlbuf).end {
        (*xmlbuf).ptr = (*xmlbuf).end;
        return set_error(xmlbuf);
    }

    (*attr).value.len = ptr.offset_from((*attr).value.ptr) as core::ffi::c_uint;
    (*xmlbuf).ptr = ptr.add(1);
    if (*xmlbuf).ptr != (*xmlbuf).end {
        return 1;
    }

    set_error(xmlbuf)
}

unsafe fn read_xml_elem(xmlbuf: *mut xmlbuf_t, elem: *mut xml_elem) {
    let mut ptr = (*xmlbuf).ptr;

    (*elem).ns = empty_xmlstr;
    (*elem).name.ptr = ptr;
    while ptr < (*xmlbuf).end && isxmlspace(*ptr) == 0 && *ptr != b'>' as WCHAR && *ptr != b'/' as WCHAR {
        if *ptr == b':' as WCHAR {
            (*elem).ns.ptr = (*elem).name.ptr;
            (*elem).ns.len = ptr.offset_from((*elem).ns.ptr) as core::ffi::c_uint;
            (*elem).name.ptr = ptr.add(1);
        }
        ptr = ptr.add(1);
    }
    (*elem).name.len = ptr.offset_from((*elem).name.ptr) as core::ffi::c_uint;
    (*xmlbuf).ptr = ptr;
}

unsafe fn next_xml_elem(xmlbuf: *mut xmlbuf_t, elem: *mut xml_elem, parent: *const xml_elem) -> BOOL {
    let mut ptr: *const WCHAR;
    let mut attr: xml_attr = core::mem::zeroed();
    let mut attr_buf: xmlbuf_t;
    let mut end: BOOL = 0;

    (*xmlbuf).ns_pos = (*parent).ns_pos;

    if (*xmlbuf).error != 0 {
        return 0;
    }

    loop {
        ptr = (*xmlbuf).ptr;
        while ptr < (*xmlbuf).end {
            if *ptr == b'<' as WCHAR {
                break;
            }
            ptr = ptr.add(1);
        }
        if ptr == (*xmlbuf).end {
            (*xmlbuf).ptr = (*xmlbuf).end;
            return set_error(xmlbuf);
        }
        ptr = ptr.add(1);
        if ptr.add(3) < (*xmlbuf).end
            && *ptr.offset(0) == b'!' as WCHAR
            && *ptr.offset(1) == b'-' as WCHAR
            && *ptr.offset(2) == b'-' as WCHAR
        {
            ptr = ptr.add(3);
            while ptr.add(3) <= (*xmlbuf).end {
                if *ptr.offset(0) == b'-' as WCHAR
                    && *ptr.offset(1) == b'-' as WCHAR
                    && *ptr.offset(2) == b'>' as WCHAR
                {
                    break;
                }
                ptr = ptr.add(1);
            }

            if ptr.add(3) > (*xmlbuf).end {
                (*xmlbuf).ptr = (*xmlbuf).end;
                return set_error(xmlbuf);
            }
            (*xmlbuf).ptr = ptr.add(3);
        } else {
            break;
        }
    }

    (*xmlbuf).ptr = ptr;
    if ptr < (*xmlbuf).end && *ptr == b'/' as WCHAR {
        (*xmlbuf).ptr = (*xmlbuf).ptr.add(1);
        read_xml_elem(xmlbuf, elem);
        (*elem).ns = find_xmlns(xmlbuf, core::ptr::addr_of_mut!((*elem).ns));
        if xml_name_cmp(elem, parent) == 0 {
            ERR!(
                "wrong closing element %s for %s\n",
                debugstr_xmlstr(core::ptr::addr_of!((*elem).name)),
                debugstr_xmlstr(core::ptr::addr_of!((*parent).name)),
            );
            return set_error(xmlbuf);
        }
        while (*xmlbuf).ptr < (*xmlbuf).end && isxmlspace(*(*xmlbuf).ptr) != 0 {
            (*xmlbuf).ptr = (*xmlbuf).ptr.add(1);
        }
        if (*xmlbuf).ptr == (*xmlbuf).end {
            return set_error(xmlbuf);
        }
        let c = *(*xmlbuf).ptr;
        (*xmlbuf).ptr = (*xmlbuf).ptr.add(1);
        if c != b'>' as WCHAR {
            return set_error(xmlbuf);
        }
        return 0;
    }

    read_xml_elem(xmlbuf, elem);

    attr_buf = *xmlbuf;
    while next_xml_attr(&mut attr_buf, &mut attr, &mut end) != 0 {
        if is_xmlns_attr(&attr) != 0 {
            push_xmlns(xmlbuf, &attr);
        }
    }
    (*elem).ns = find_xmlns(xmlbuf, core::ptr::addr_of_mut!((*elem).ns));
    (*elem).ns_pos = (*xmlbuf).ns_pos;

    if (*xmlbuf).ptr != (*xmlbuf).end {
        return 1;
    }

    set_error(xmlbuf)
}

unsafe fn parse_xml_header(xmlbuf: *mut xmlbuf_t) -> BOOL {
    let mut ptr = (*xmlbuf).ptr;
    while ptr < (*xmlbuf).end.offset(-1) {
        if *ptr.offset(0) == '?' as WCHAR && *ptr.offset(1) == '>' as WCHAR {
            (*xmlbuf).ptr = ptr.offset(2);
            return 1;
        }
        ptr = ptr.offset(1);
    }
    0
}

unsafe fn parse_text_content(xmlbuf: *mut xmlbuf_t, content: *mut xmlstr_t) -> BOOL {
    let mut ptr: *const WCHAR;

    if (*xmlbuf).error != 0 {
        return 0;
    }

    ptr = (*xmlbuf).ptr;
    while ptr < (*xmlbuf).end {
        if *ptr == b'<' as WCHAR {
            break;
        }
        ptr = ptr.add(1);
    }
    if ptr == (*xmlbuf).end {
        return set_error(xmlbuf);
    }

    (*content).ptr = (*xmlbuf).ptr;
    (*content).len = ptr.offset_from((*xmlbuf).ptr) as core::ffi::c_uint;
    (*xmlbuf).ptr = ptr;

    1
}

unsafe fn parse_version(str: *const xmlstr_t, version: *mut assembly_version) -> BOOL {
    let mut ver: [core::ffi::c_uint; 4] = [0, 0, 0, 0];
    let mut pos: core::ffi::c_uint = 0;
    let mut curr: *const WCHAR;

    let result = 'error: {
        curr = (*str).ptr;
        while curr < (*str).ptr.add((*str).len as usize) {
            let c = *curr;
            if c >= b'0' as WCHAR && c <= b'9' as WCHAR {
                ver[pos as usize] = ver[pos as usize] * 10 + (c as core::ffi::c_uint) - (b'0' as core::ffi::c_uint);
                if ver[pos as usize] >= 0x10000 {
                    break 'error 0u8;
                }
            } else if c == b'.' as WCHAR {
                pos += 1;
                if pos >= 4 {
                    break 'error 0u8;
                }
            } else {
                break 'error 0u8;
            }
            curr = curr.add(1);
        }
        (*version).major = ver[0] as USHORT;
        (*version).minor = ver[1] as USHORT;
        (*version).build = ver[2] as USHORT;
        (*version).revision = ver[3] as USHORT;
        return 1;
    };

    FIXME!("Wrong version definition in manifest file (%s)\n", debugstr_xmlstr(str));
    result as BOOL
}

unsafe fn parse_expect_no_attr(xmlbuf: *mut xmlbuf_t, end: *mut BOOL) {
    let mut attr: xml_attr = core::mem::zeroed();

    while next_xml_attr(xmlbuf, &mut attr, end) != 0 {
        if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unexpected attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }
}

unsafe fn parse_expect_end_elem(xmlbuf: *mut xmlbuf_t, parent: *const xml_elem) {
    let mut elem: xml_elem = core::mem::zeroed();

    if next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        FIXME!(
            "unexpected element %s\n",
            debugstr_xml_elem(&elem),
        );
        set_error(xmlbuf);
    }
}

unsafe fn parse_unknown_elem(xmlbuf: *mut xmlbuf_t, parent: *const xml_elem) {
    let mut elem: xml_elem = core::mem::zeroed();
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {}
    if end != 0 {
        return;
    }

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        parse_unknown_elem(xmlbuf, &elem);
    }
}

unsafe fn parse_assembly_identity_elem(
    xmlbuf: *mut xmlbuf_t,
    _actctx: *mut ACTIVATION_CONTEXT,
    ai: *mut assembly_identity,
    parent: *const xml_elem,
) {
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, g_nameW.as_ptr()) != 0 {
            (*ai).name = xmlstrdupW(&attr.value);
            if (*ai).name.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, typeW.as_ptr()) != 0 {
            (*ai).type_ = xmlstrdupW(&attr.value);
            if (*ai).type_.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, versionW.as_ptr()) != 0 {
            if parse_version(&attr.value, core::ptr::addr_of_mut!((*ai).version)) == 0 {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, processorArchitectureW.as_ptr()) != 0 {
            (*ai).arch = xmlstrdupW(&attr.value);
            if (*ai).arch.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, publicKeyTokenW.as_ptr()) != 0 {
            (*ai).public_key = xmlstrdupW(&attr.value);
            if (*ai).public_key.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, languageW.as_ptr()) != 0 {
            (*ai).language = xmlstrdupW(&attr.value);
            if (*ai).language.is_null() {
                set_error(xmlbuf);
            }
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    TRACE!(
        b"name=%s version=%s arch=%s\n\0".as_ptr() as *const core::ffi::c_char,
        debugstr_w((*ai).name),
        debugstr_version(core::ptr::addr_of!((*ai).version)),
        debugstr_w((*ai).arch),
    );

    if end == 0 {
        parse_expect_end_elem(xmlbuf, parent);
    }
}

unsafe fn parse_com_class_threadingmodel(value: *mut xmlstr_t) -> comclass_threadingmodel {
    static apartW: [WCHAR; 10] = [
        b'A' as WCHAR, b'p' as WCHAR, b'a' as WCHAR, b'r' as WCHAR, b't' as WCHAR,
        b'm' as WCHAR, b'e' as WCHAR, b'n' as WCHAR, b't' as WCHAR, 0,
    ];
    static neutralW: [WCHAR; 8] = [
        b'N' as WCHAR, b'e' as WCHAR, b'u' as WCHAR, b't' as WCHAR, b'r' as WCHAR,
        b'a' as WCHAR, b'l' as WCHAR, 0,
    ];
    static freeW: [WCHAR; 5] = [
        b'F' as WCHAR, b'r' as WCHAR, b'e' as WCHAR, b'e' as WCHAR, 0,
    ];
    static bothW: [WCHAR; 5] = [
        b'B' as WCHAR, b'o' as WCHAR, b't' as WCHAR, b'h' as WCHAR, 0,
    ];

    if (*value).len == 0 {
        return comclass_threadingmodel_ThreadingModel_No;
    }
    if xmlstr_cmp(value, apartW.as_ptr()) != 0 {
        comclass_threadingmodel_ThreadingModel_Apartment
    } else if xmlstr_cmp(value, freeW.as_ptr()) != 0 {
        comclass_threadingmodel_ThreadingModel_Free
    } else if xmlstr_cmp(value, bothW.as_ptr()) != 0 {
        comclass_threadingmodel_ThreadingModel_Both
    } else if xmlstr_cmp(value, neutralW.as_ptr()) != 0 {
        comclass_threadingmodel_ThreadingModel_Neutral
    } else {
        comclass_threadingmodel_ThreadingModel_No
    }
}

unsafe fn get_olemisc_value(str: *const WCHAR, len: core::ffi::c_int) -> OLEMISC {
    let mut min: core::ffi::c_int = 0;
    let mut max: core::ffi::c_int = (olemisc_values.len() as core::ffi::c_int) - 1;

    while min <= max {
        let n = (min + max) / 2;

        let c = wcsncmp(olemisc_values[n as usize].name, str, len as usize);
        if c == 0 && *olemisc_values[n as usize].name.add(len as usize) == 0 {
            return olemisc_values[n as usize].value;
        }

        if c >= 0 {
            max = n - 1;
        } else {
            min = n + 1;
        }
    }

    WARN!(b"unknown flag %s\n\0".as_ptr() as *const core::ffi::c_char, debugstr_wn(str, len));
    0
}

unsafe fn parse_com_class_misc(value: *const xmlstr_t) -> DWORD {
    let mut str: *const WCHAR = (*value).ptr;
    let mut start: *const WCHAR;
    let mut flags: DWORD = 0;
    let mut i: core::ffi::c_int = 0;

    while i < (*value).len as core::ffi::c_int {
        start = str;
        while *str != b',' as WCHAR && {
            let cond = i < (*value).len as core::ffi::c_int;
            i += 1;
            cond
        } {
            str = str.add(1);
        }

        flags |= get_olemisc_value(start, str.offset_from(start) as core::ffi::c_int) as DWORD;

        str = str.add(1);
        i += 1;
    }

    flags
}

unsafe fn com_class_add_progid(progid: *const xmlstr_t, entity: *mut entity) -> BOOL {
    let progids: *mut progids = core::ptr::addr_of_mut!((*entity).u.comclass.progids);

    if (*progids).allocated == 0 {
        (*progids).allocated = 4;
        (*progids).progids = RtlAllocateHeap(
            GetProcessHeap(),
            0,
            ((*progids).allocated as usize * core::mem::size_of::<*mut WCHAR>()) as SIZE_T,
        ) as *mut *mut WCHAR;
        if (*progids).progids.is_null() {
            return 0;
        }
    }

    if (*progids).allocated == (*progids).num {
        let new_progids = RtlReAllocateHeap(
            GetProcessHeap(),
            0,
            (*progids).progids as PVOID,
            (2 * (*progids).allocated as usize * core::mem::size_of::<*mut WCHAR>()) as SIZE_T,
        ) as *mut *mut WCHAR;
        if new_progids.is_null() {
            return 0;
        }
        (*progids).allocated *= 2;
        (*progids).progids = new_progids;
    }

    let dup = xmlstrdupW(progid);
    *(*progids).progids.add((*progids).num as usize) = dup;
    if dup.is_null() {
        return 0;
    }
    (*progids).num += 1;

    1
}

unsafe fn parse_com_class_progid(xmlbuf: *mut xmlbuf_t, entity: *mut entity, parent: *const xml_elem) {
    let mut content: xmlstr_t = core::mem::zeroed();
    let mut end: BOOL = 0;

    parse_expect_no_attr(xmlbuf, &mut end);
    if end != 0 {
        set_error(xmlbuf);
    }
    if parse_text_content(xmlbuf, &mut content) == 0 {
        return;
    }

    if com_class_add_progid(&content, entity) == 0 {
        set_error(xmlbuf);
    }
    parse_expect_end_elem(xmlbuf, parent);
}

unsafe fn parse_com_class_elem(
    xmlbuf: *mut xmlbuf_t,
    dll: *mut dll_redirect,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;
    let entity: *mut entity;

    entity = add_entity(
        core::ptr::addr_of_mut!((*dll).entities),
        ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION,
    );
    if entity.is_null() {
        set_error(xmlbuf);
        return;
    }

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, clsidW.as_ptr()) != 0 {
            (*entity).u.comclass.clsid = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.comclass.clsid.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, progidW.as_ptr()) != 0 {
            (*entity).u.comclass.progid = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.comclass.progid.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, tlbidW.as_ptr()) != 0 {
            (*entity).u.comclass.tlbid = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.comclass.tlbid.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, threadingmodelW.as_ptr()) != 0 {
            (*entity).u.comclass.model =
                parse_com_class_threadingmodel(core::ptr::addr_of_mut!(attr.value)) as DWORD;
        } else if xml_attr_cmp(&attr, miscstatusW.as_ptr()) != 0 {
            (*entity).u.comclass.miscstatus = parse_com_class_misc(core::ptr::addr_of!(attr.value));
        } else if xml_attr_cmp(&attr, miscstatuscontentW.as_ptr()) != 0 {
            (*entity).u.comclass.miscstatuscontent =
                parse_com_class_misc(core::ptr::addr_of!(attr.value));
        } else if xml_attr_cmp(&attr, miscstatusthumbnailW.as_ptr()) != 0 {
            (*entity).u.comclass.miscstatusthumbnail =
                parse_com_class_misc(core::ptr::addr_of!(attr.value));
        } else if xml_attr_cmp(&attr, miscstatusiconW.as_ptr()) != 0 {
            (*entity).u.comclass.miscstatusicon =
                parse_com_class_misc(core::ptr::addr_of!(attr.value));
        } else if xml_attr_cmp(&attr, miscstatusdocprintW.as_ptr()) != 0 {
            (*entity).u.comclass.miscstatusdocprint =
                parse_com_class_misc(core::ptr::addr_of!(attr.value));
        } else if xml_attr_cmp(&attr, descriptionW.as_ptr()) != 0 {
            /* not stored */
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    (*(*acl).actctx).sections |= SERVERREDIRECT_SECTION as DWORD;
    if !(*entity).u.comclass.progid.is_null() {
        (*(*acl).actctx).sections |= PROGIDREDIRECT_SECTION as DWORD;
    }

    if end != 0 {
        return;
    }

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, progidW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_com_class_progid(xmlbuf, entity, &elem);
        } else {
            WARN!(
                b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }

    if (*entity).u.comclass.progids.num != 0 {
        (*(*acl).actctx).sections |= PROGIDREDIRECT_SECTION as DWORD;
    }
}

unsafe fn parse_nummethods(str: *const xmlstr_t, entity: *mut entity) -> BOOL {
    let mut num: ULONG = 0;

    let mut curr = (*str).ptr;
    while curr < (*str).ptr.add((*str).len as usize) {
        if *curr >= b'0' as WCHAR && *curr <= b'9' as WCHAR {
            num = num * 10 + (*curr as ULONG) - b'0' as ULONG;
        } else {
            return 0;
        }
        curr = curr.add(1);
    }
    (*entity).u.ifaceps.nummethods = num;

    1
}

unsafe fn parse_add_interface_class(xmlbuf: *mut xmlbuf_t, entities: *mut entity_array, acl: *mut actctx_loader, clsid: *mut WCHAR) {
    let entity: *mut entity;
    let str: *mut WCHAR;

    if clsid.is_null() {
        return;
    }

    str = strdupW(clsid);
    if str.is_null() {
        set_error(xmlbuf);
        return;
    }

    entity = add_entity(entities, ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION);
    if entity.is_null() {
        RtlFreeHeap(GetProcessHeap(), 0, str as *mut core::ffi::c_void);
        set_error(xmlbuf);
        return;
    }

    (*entity).u.comclass.clsid = str;
    (*entity).u.comclass.model = ThreadingModel_Both as DWORD;

    (*(*acl).actctx).sections |= SERVERREDIRECT_SECTION as DWORD;
}

unsafe fn parse_cominterface_proxy_stub_elem(xmlbuf: *mut xmlbuf_t, dll: *mut dll_redirect, acl: *mut actctx_loader, parent: *const xml_elem) {
    let mut psclsid: *mut WCHAR = core::ptr::null_mut();
    let entity: *mut entity;
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;

    entity = add_entity(core::ptr::addr_of_mut!((*dll).entities), ACTIVATION_CONTEXT_SECTION_COM_INTERFACE_REDIRECTION);
    if entity.is_null() {
        set_error(xmlbuf);
        return;
    }

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, iidW.as_ptr()) != 0 {
            (*entity).u.ifaceps.iid = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.ifaceps.iid.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, g_nameW.as_ptr()) != 0 {
            (*entity).u.ifaceps.name = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.ifaceps.name.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, baseInterfaceW.as_ptr()) != 0 {
            (*entity).u.ifaceps.base = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.ifaceps.base.is_null() {
                set_error(xmlbuf);
            }
            (*entity).u.ifaceps.mask |= BaseIface as DWORD;
        } else if xml_attr_cmp(&attr, nummethodsW.as_ptr()) != 0 {
            if parse_nummethods(core::ptr::addr_of!(attr.value), entity) == 0 {
                set_error(xmlbuf);
            }
            (*entity).u.ifaceps.mask |= NumMethods as DWORD;
        } else if xml_attr_cmp(&attr, tlbidW.as_ptr()) != 0 {
            (*entity).u.ifaceps.tlib = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.ifaceps.tlib.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, proxyStubClsid32W.as_ptr()) != 0 {
            psclsid = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if psclsid.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, threadingmodelW.as_ptr()) != 0 {
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    (*(*acl).actctx).sections |= IFACEREDIRECT_SECTION as DWORD;
    if end == 0 {
        parse_expect_end_elem(xmlbuf, parent);
    }

    let clsid = if !psclsid.is_null() { psclsid } else { (*entity).u.ifaceps.iid };
    parse_add_interface_class(xmlbuf, core::ptr::addr_of_mut!((*dll).entities), acl, clsid);

    RtlFreeHeap(GetProcessHeap(), 0, psclsid as *mut core::ffi::c_void);
}

unsafe fn parse_typelib_flags(value: *const xmlstr_t, entity: *mut entity) -> BOOL {
    let flags: *mut WORD = core::ptr::addr_of_mut!((*entity).u.typelib.flags);
    let mut str: *const WCHAR = (*value).ptr;
    let mut start: *const WCHAR;
    let mut i: core::ffi::c_int = 0;

    *flags = 0;

    /* it's comma separated list of flags */
    while (i as core::ffi::c_uint) < (*value).len {
        start = str;
        while *str != (',' as WCHAR) && {
            let cond = (i as core::ffi::c_uint) < (*value).len;
            i += 1;
            cond
        } {
            str = str.add(1);
        }

        let len = str.offset_from(start) as core::ffi::c_int;

        if wcsnicmp(start, restrictedW.as_ptr(), len as _) == 0 {
            *flags |= LIBFLAG_FRESTRICTED as WORD;
        } else if wcsnicmp(start, controlW.as_ptr(), len as _) == 0 {
            *flags |= LIBFLAG_FCONTROL as WORD;
        } else if wcsnicmp(start, hiddenW.as_ptr(), len as _) == 0 {
            *flags |= LIBFLAG_FHIDDEN as WORD;
        } else if wcsnicmp(start, hasdiskimageW.as_ptr(), len as _) == 0 {
            *flags |= LIBFLAG_FHASDISKIMAGE as WORD;
        } else {
            WARN!(b"unknown flags value %s\n\0".as_ptr() as *const core::ffi::c_char, debugstr_xmlstr(value));
            return 0;
        }

        /* skip separator */
        str = str.add(1);
        i += 1;
    }

    1
}

unsafe fn parse_typelib_version(str: *const xmlstr_t, entity: *mut entity) -> BOOL {
    let mut ver: [core::ffi::c_uint; 2] = [0, 0];
    let mut pos: core::ffi::c_uint = 0;
    let mut curr: *const WCHAR;

    'error: {
        curr = (*str).ptr;
        while curr < (*str).ptr.add((*str).len as usize) {
            let c = *curr;
            if c >= b'0' as WCHAR && c <= b'9' as WCHAR {
                ver[pos as usize] = ver[pos as usize] * 10 + (c as core::ffi::c_uint) - (b'0' as core::ffi::c_uint);
                if ver[pos as usize] >= 0x10000 {
                    break 'error;
                }
            } else if c == b'.' as WCHAR {
                pos += 1;
                if pos >= 2 {
                    break 'error;
                }
            } else {
                break 'error;
            }
            curr = curr.add(1);
        }
        (*entity).u.typelib.major = ver[0] as WORD;
        (*entity).u.typelib.minor = ver[1] as WORD;
        return 1u8 as BOOL;
    }

    FIXME!("wrong typelib version value (%s)\n", debugstr_xmlstr(str));
    0u8 as BOOL
}

unsafe fn parse_typelib_elem(
    xmlbuf: *mut xmlbuf_t,
    dll: *mut dll_redirect,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;
    let entity: *mut entity;

    entity = add_entity(
        core::ptr::addr_of_mut!((*dll).entities),
        ACTIVATION_CONTEXT_SECTION_COM_TYPE_LIBRARY_REDIRECTION,
    );
    if entity.is_null() {
        set_error(xmlbuf);
        return;
    }

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, tlbidW.as_ptr()) != 0 {
            (*entity).u.typelib.tlbid = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.typelib.tlbid.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, versionW.as_ptr()) != 0 {
            if parse_typelib_version(core::ptr::addr_of!(attr.value), entity) == 0 {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, helpdirW.as_ptr()) != 0 {
            (*entity).u.typelib.helpdir = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.typelib.helpdir.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, flagsW.as_ptr()) != 0 {
            if parse_typelib_flags(core::ptr::addr_of!(attr.value), entity) == 0 {
                set_error(xmlbuf);
            }
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    (*(*acl).actctx).sections |= TLIBREDIRECT_SECTION as DWORD;
    if end == 0 {
        parse_expect_end_elem(xmlbuf, parent);
    }
}

unsafe fn aligned_string_len(len: i32) -> i32 {
    (len + 3) & !3
}

unsafe fn get_assembly_version(assembly: *mut assembly, mut ret: *mut WCHAR) -> i32 {
    static fmtW: [WCHAR; 10] = [
        '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR,
        '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR,
    ];
    let fmtW_full: [WCHAR; 11] = [
        '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR,
        '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR,
        'u' as WCHAR,
    ];
    let _ = fmtW;
    let fmt_buf: [WCHAR; 12] = [
        '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR,
        '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR,
        'u' as WCHAR, 0,
    ];
    let _ = fmtW_full;

    let ver: *mut assembly_version = core::ptr::addr_of_mut!((*assembly).id.version);
    let mut buff: [WCHAR; 25] = [0; 25];

    if ret.is_null() {
        ret = buff.as_mut_ptr();
    }
    swprintf(
        ret,
        buff.len(),
        fmt_buf.as_ptr(),
        (*ver).major as core::ffi::c_uint,
        (*ver).minor as core::ffi::c_uint,
        (*ver).build as core::ffi::c_uint,
        (*ver).revision as core::ffi::c_uint,
    )
}

unsafe fn parse_window_class_elem(
    xmlbuf: *mut xmlbuf_t,
    dll: *mut dll_redirect,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();
    let mut attr: xml_attr = core::mem::zeroed();
    let mut content: xmlstr_t = core::mem::zeroed();
    let mut end: BOOL = 0;
    let entity: *mut entity;

    entity = add_entity(
        core::ptr::addr_of_mut!((*dll).entities),
        ACTIVATION_CONTEXT_SECTION_WINDOW_CLASS_REDIRECTION,
    );
    if entity.is_null() {
        set_error(xmlbuf);
        return;
    }
    (*entity).u.class.versioned = 1;
    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, versionedW.as_ptr()) != 0 {
            if xmlstr_cmpi(core::ptr::addr_of!(attr.value), noW.as_ptr()) != 0 {
                (*entity).u.class.versioned = 0;
            } else if xmlstr_cmpi(core::ptr::addr_of!(attr.value), yesW.as_ptr()) == 0 {
                set_error(xmlbuf);
            }
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    if end != 0 {
        return;
    }

    if parse_text_content(xmlbuf, &mut content) == 0 {
        return;
    }
    (*entity).u.class.name = xmlstrdupW(&content);
    if (*entity).u.class.name.is_null() {
        set_error(xmlbuf);
    }

    (*(*acl).actctx).sections |= WINDOWCLASS_SECTION as DWORD;

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        WARN!(
            b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
            debugstr_xml_elem(&elem),
        );
        parse_unknown_elem(xmlbuf, &elem);
    }
}

unsafe fn parse_binding_redirect_elem(xmlbuf: *mut xmlbuf_t, parent: *const xml_elem) {
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, oldVersionW.as_ptr()) != 0 {
            FIXME!(
                "Not stored yet %s\n",
                debugstr_xml_attr(&attr),
            );
        } else if xml_attr_cmp(&attr, newVersionW.as_ptr()) != 0 {
            FIXME!(
                "Not stored yet %s\n",
                debugstr_xml_attr(&attr),
            );
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    if end == 0 {
        parse_expect_end_elem(xmlbuf, parent);
    }
}

unsafe fn parse_description_elem(xmlbuf: *mut xmlbuf_t, parent: *const xml_elem) {
    let mut elem: xml_elem = core::mem::zeroed();
    let mut attr: xml_attr = core::mem::zeroed();
    let mut content: xmlstr_t = core::mem::zeroed();
    let mut end: BOOL = 0;

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    if end != 0 {
        return;
    }
    if parse_text_content(xmlbuf, &mut content) == 0 {
        return;
    }

    TRACE!(
        b"Got description %s\n\0".as_ptr() as *const core::ffi::c_char,
        debugstr_xmlstr(&content),
    );

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        WARN!(
            b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
            debugstr_xml_elem(&elem),
        );
        parse_unknown_elem(xmlbuf, &elem);
    }
}

unsafe fn parse_com_interface_external_proxy_stub_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;
    let entity: *mut entity;

    entity = add_entity(
        core::ptr::addr_of_mut!((*assembly).entities),
        ACTIVATION_CONTEXT_SECTION_COM_INTERFACE_REDIRECTION,
    );
    if entity.is_null() {
        set_error(xmlbuf);
        return;
    }

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, iidW.as_ptr()) != 0 {
            (*entity).u.ifaceps.iid = xmlstrdupW(&attr.value);
            if (*entity).u.ifaceps.iid.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, g_nameW.as_ptr()) != 0 {
            (*entity).u.ifaceps.name = xmlstrdupW(&attr.value);
            if (*entity).u.ifaceps.name.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, baseInterfaceW.as_ptr()) != 0 {
            (*entity).u.ifaceps.base = xmlstrdupW(&attr.value);
            if (*entity).u.ifaceps.base.is_null() {
                set_error(xmlbuf);
            }
            (*entity).u.ifaceps.mask |= BaseIface as DWORD;
        } else if xml_attr_cmp(&attr, nummethodsW.as_ptr()) != 0 {
            if parse_nummethods(&attr.value, entity) == 0 {
                set_error(xmlbuf);
            }
            (*entity).u.ifaceps.mask |= NumMethods as DWORD;
        } else if xml_attr_cmp(&attr, proxyStubClsid32W.as_ptr()) != 0 {
            (*entity).u.ifaceps.ps32 = xmlstrdupW(&attr.value);
            if (*entity).u.ifaceps.ps32.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, tlbidW.as_ptr()) != 0 {
            (*entity).u.ifaceps.tlib = xmlstrdupW(&attr.value);
            if (*entity).u.ifaceps.tlib.is_null() {
                set_error(xmlbuf);
            }
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    (*(*acl).actctx).sections |= IFACEREDIRECT_SECTION as DWORD;
    if end == 0 {
        parse_expect_end_elem(xmlbuf, parent);
    }
}

unsafe fn parse_clr_class_elem(xmlbuf: *mut xmlbuf_t, assembly: *mut assembly, acl: *mut actctx_loader, parent: *const xml_elem) {
    let mut elem: xml_elem = core::mem::zeroed();
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;
    let entity: *mut entity;

    entity = add_entity(core::ptr::addr_of_mut!((*assembly).entities), ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION);
    if entity.is_null() {
        set_error(xmlbuf);
        return;
    }

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, g_nameW.as_ptr()) != 0 {
            (*entity).u.comclass.name = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.comclass.name.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, clsidW.as_ptr()) != 0 {
            (*entity).u.comclass.clsid = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.comclass.clsid.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, progidW.as_ptr()) != 0 {
            (*entity).u.comclass.progid = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.comclass.progid.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, tlbidW.as_ptr()) != 0 {
            (*entity).u.comclass.tlbid = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.comclass.tlbid.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, threadingmodelW.as_ptr()) != 0 {
            (*entity).u.comclass.model = parse_com_class_threadingmodel(core::ptr::addr_of_mut!(attr.value)) as DWORD;
        } else if xml_attr_cmp(&attr, runtimeVersionW.as_ptr()) != 0 {
            (*entity).u.comclass.version = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.comclass.version.is_null() {
                set_error(xmlbuf);
            }
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    (*(*acl).actctx).sections |= SERVERREDIRECT_SECTION as DWORD;
    if !(*entity).u.comclass.progid.is_null() {
        (*(*acl).actctx).sections |= PROGIDREDIRECT_SECTION as DWORD;
    }
    if end != 0 {
        return;
    }

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, progidW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_com_class_progid(xmlbuf, entity, &elem);
        } else {
            WARN!(
                b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }

    if (*entity).u.comclass.progids.num != 0 {
        (*(*acl).actctx).sections |= PROGIDREDIRECT_SECTION as DWORD;
    }
}

unsafe fn parse_clr_surrogate_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;
    let entity: *mut entity;

    entity = add_entity(
        core::ptr::addr_of_mut!((*assembly).entities),
        ACTIVATION_CONTEXT_SECTION_CLR_SURROGATES,
    );
    if entity.is_null() {
        set_error(xmlbuf);
        return;
    }

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, g_nameW.as_ptr()) != 0 {
            (*entity).u.clrsurrogate.name = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.clrsurrogate.name.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, clsidW.as_ptr()) != 0 {
            (*entity).u.clrsurrogate.clsid = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.clrsurrogate.clsid.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, runtimeVersionW.as_ptr()) != 0 {
            (*entity).u.clrsurrogate.version = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*entity).u.clrsurrogate.version.is_null() {
                set_error(xmlbuf);
            }
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    (*(*acl).actctx).sections |= CLRSURROGATES_SECTION as DWORD;
    if end == 0 {
        parse_expect_end_elem(xmlbuf, parent);
    }
}

unsafe fn parse_dependent_assembly_elem(
    xmlbuf: *mut xmlbuf_t,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
    optional: BOOL,
) {
    let mut elem: xml_elem = core::mem::zeroed();
    let mut attr: xml_attr = core::mem::zeroed();
    let mut ai: assembly_identity = core::mem::zeroed();
    let mut end: BOOL = 0;

    memset(
        core::ptr::addr_of_mut!(ai) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<assembly_identity>(),
    );
    ai.optional = optional;

    const allowDelayedBindingW: [WCHAR; 19] = [
        b'a' as WCHAR, b'l' as WCHAR, b'l' as WCHAR, b'o' as WCHAR, b'w' as WCHAR,
        b'D' as WCHAR, b'e' as WCHAR, b'l' as WCHAR, b'a' as WCHAR, b'y' as WCHAR,
        b'e' as WCHAR, b'd' as WCHAR, b'B' as WCHAR, b'i' as WCHAR, b'n' as WCHAR,
        b'd' as WCHAR, b'i' as WCHAR, b'n' as WCHAR, b'g' as WCHAR,
    ];
    const trueW: [WCHAR; 4] = [b't' as WCHAR, b'r' as WCHAR, b'u' as WCHAR, b'e' as WCHAR];
    let allowDelayedBindingW0: [WCHAR; 20] = {
        let mut a = [0 as WCHAR; 20];
        let mut i = 0;
        while i < 19 {
            a[i] = allowDelayedBindingW[i];
            i += 1;
        }
        a
    };
    let trueW0: [WCHAR; 5] = {
        let mut a = [0 as WCHAR; 5];
        let mut i = 0;
        while i < 4 {
            a[i] = trueW[i];
            i += 1;
        }
        a
    };

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, allowDelayedBindingW0.as_ptr()) != 0 {
            ai.delayed = xmlstr_cmp(core::ptr::addr_of!(attr.value), trueW0.as_ptr());
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    if end != 0 {
        return;
    }

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, assemblyIdentityW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_assembly_identity_elem(xmlbuf, (*acl).actctx, &mut ai, &elem);
            if !ai.arch.is_null() && wcscmp(ai.arch, wildcardW.as_ptr()) == 0 {
                RtlFreeHeap(GetProcessHeap(), 0, ai.arch as *mut core::ffi::c_void);
                ai.arch = strdupW(current_archW.as_ptr());
            }
            TRACE!(
                b"adding name=%s version=%s arch=%s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_w(ai.name),
                debugstr_version(core::ptr::addr_of!(ai.version)),
                debugstr_w(ai.arch),
            );
            if add_dependent_assembly_id(acl, &mut ai) == 0 {
                set_error(xmlbuf);
            }
        } else if xml_elem_cmp(&elem, bindingRedirectW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_binding_redirect_elem(xmlbuf, &elem);
        } else {
            WARN!(
                b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }
}

unsafe fn parse_dependency_elem(
    xmlbuf: *mut xmlbuf_t,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;
    let mut optional: BOOL = 0;

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, optionalW.as_ptr()) != 0 {
            optional = xmlstr_cmpi(core::ptr::addr_of!(attr.value), yesW.as_ptr());
            TRACE!(
                b"optional=%s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xmlstr(core::ptr::addr_of!(attr.value)),
            );
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, dependentAssemblyW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_dependent_assembly_elem(xmlbuf, acl, &elem, optional);
        } else {
            WARN!(
                b"unknown element %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }
}

unsafe fn parse_noinherit_elem(xmlbuf: *mut xmlbuf_t, parent: *const xml_elem) {
    let mut end: BOOL = 0;

    parse_expect_no_attr(xmlbuf, &mut end);
    if end == 0 {
        parse_expect_end_elem(xmlbuf, parent);
    }
}

unsafe fn parse_noinheritable_elem(xmlbuf: *mut xmlbuf_t, parent: *const xml_elem) {
    let mut end: BOOL = 0;

    parse_expect_no_attr(xmlbuf, &mut end);
    if end == 0 {
        parse_expect_end_elem(xmlbuf, parent);
    }
}

unsafe fn parse_file_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;
    let dll: *mut dll_redirect;

    dll = add_dll_redirect(assembly);
    if dll.is_null() {
        set_error(xmlbuf);
        return;
    }

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, g_nameW.as_ptr()) != 0 {
            (*dll).name = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*dll).name.is_null() {
                set_error(xmlbuf);
            }
            TRACE!(
                b"name=%s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xmlstr(core::ptr::addr_of!(attr.value)),
            );
        } else if xml_attr_cmp(&attr, hashW.as_ptr()) != 0 {
            (*dll).hash = xmlstrdupW(core::ptr::addr_of!(attr.value));
            if (*dll).hash.is_null() {
                set_error(xmlbuf);
            }
        } else if xml_attr_cmp(&attr, hashalgW.as_ptr()) != 0 {
            static sha1W: [WCHAR; 5] = [
                b'S' as WCHAR,
                b'H' as WCHAR,
                b'A' as WCHAR,
                b'1' as WCHAR,
                0,
            ];
            if xmlstr_cmpi(core::ptr::addr_of!(attr.value), sha1W.as_ptr()) == 0 {
                FIXME!(
                    "hashalg should be SHA1, got %s\n",
                    debugstr_xmlstr(core::ptr::addr_of!(attr.value)),
                );
            }
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    if (*dll).name.is_null() {
        set_error(xmlbuf);
    }

    (*(*acl).actctx).sections |= DLLREDIRECT_SECTION as DWORD;

    if end != 0 {
        return;
    }

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, comClassW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_com_class_elem(xmlbuf, dll, acl, &elem);
        } else if xml_elem_cmp(&elem, comInterfaceProxyStubW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_cominterface_proxy_stub_elem(xmlbuf, dll, acl, &elem);
        } else if xml_elem_cmp(&elem, hashW.as_ptr(), asmv2W.as_ptr()) != 0 {
            WARN!(b"asmv2:hash (undocumented) not supported\n\0".as_ptr() as *const core::ffi::c_char);
            parse_unknown_elem(xmlbuf, &elem);
        } else if xml_elem_cmp(&elem, typelibW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_typelib_elem(xmlbuf, dll, acl, &elem);
        } else if xml_elem_cmp(&elem, windowClassW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_window_class_elem(xmlbuf, dll, acl, &elem);
        } else {
            WARN!(
                b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }
}

unsafe fn parse_supportedos_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    _acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, IdW.as_ptr()) != 0 {
            let compat: *mut COMPATIBILITY_CONTEXT_ELEMENT;
            let mut str: _UNICODE_STRING = core::mem::zeroed();
            let mut compat_id: GUID = core::mem::zeroed();

            str.Buffer = attr.value.ptr as PWSTR;
            str.Length = (attr.value.len as USHORT)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as USHORT);
            str.MaximumLength = str.Length;
            if RtlGUIDFromString(&mut str, &mut compat_id) == STATUS_SUCCESS as NTSTATUS {
                compat = add_compat_context(assembly);
                if compat.is_null() {
                    set_error(xmlbuf);
                    return;
                }
                (*compat).Type = ACTCTX_COMPATIBILITY_ELEMENT_TYPE_OS;
                (*compat).Id = compat_id;
            } else {
                WARN!(
                    b"Invalid guid %s\n\0".as_ptr() as *const core::ffi::c_char,
                    debugstr_xmlstr(core::ptr::addr_of!(attr.value)),
                );
            }
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    if end == 0 {
        parse_expect_end_elem(xmlbuf, parent);
    }
}

unsafe fn parse_compatibility_application_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, supportedOSW.as_ptr(), compatibilityNSW.as_ptr()) != 0 {
            parse_supportedos_elem(xmlbuf, assembly, acl, &elem);
        } else {
            WARN!(
                b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }
}

unsafe fn parse_compatibility_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, applicationW.as_ptr(), compatibilityNSW.as_ptr()) != 0 {
            parse_compatibility_application_elem(xmlbuf, assembly, acl, &elem);
        } else {
            WARN!(
                b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }
}

unsafe fn parse_settings_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    _acl: *mut actctx_loader,
    parent: *mut xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();
    let mut attr: xml_attr = core::mem::zeroed();
    let mut content: xmlstr_t = core::mem::zeroed();
    let mut end: BOOL = 0;
    let entity: *mut entity;

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    if end != 0 {
        return;
    }

    if parse_text_content(xmlbuf, &mut content) == 0 {
        return;
    }
    TRACE!(
        b"got %s %s\n\0".as_ptr() as *const core::ffi::c_char,
        debugstr_xmlstr(&(*parent).name),
        debugstr_xmlstr(&content),
    );

    entity = add_entity(
        core::ptr::addr_of_mut!((*assembly).entities),
        ACTIVATION_CONTEXT_SECTION_APPLICATION_SETTINGS,
    );
    if entity.is_null() {
        set_error(xmlbuf);
        return;
    }
    (*entity).u.settings.name = xmlstrdupW(&(*parent).name);
    (*entity).u.settings.value = xmlstrdupW(&content);
    (*entity).u.settings.ns = xmlstrdupW(&(*parent).ns);

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        WARN!(
            b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
            debugstr_xml_elem(&elem),
        );
        parse_unknown_elem(xmlbuf, &elem);
    }
}

unsafe fn parse_windows_settings_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, autoElevateW.as_ptr(), windowsSettings2005NSW.as_ptr()) != 0
            || xml_elem_cmp(&elem, disableThemingW.as_ptr(), windowsSettings2005NSW.as_ptr()) != 0
            || xml_elem_cmp(&elem, disableWindowFilteringW.as_ptr(), windowsSettings2011NSW.as_ptr()) != 0
            || xml_elem_cmp(&elem, dpiAwareW.as_ptr(), windowsSettings2005NSW.as_ptr()) != 0
            || xml_elem_cmp(&elem, dpiAwarenessW.as_ptr(), windowsSettings2016NSW.as_ptr()) != 0
            || xml_elem_cmp(&elem, gdiScalingW.as_ptr(), windowsSettings2017NSW.as_ptr()) != 0
            || xml_elem_cmp(&elem, highResolutionScrollingAwareW.as_ptr(), windowsSettings2017NSW.as_ptr()) != 0
            || xml_elem_cmp(&elem, longPathAwareW.as_ptr(), windowsSettings2016NSW.as_ptr()) != 0
            || xml_elem_cmp(&elem, magicFutureSettingW.as_ptr(), windowsSettings2017NSW.as_ptr()) != 0
            || xml_elem_cmp(&elem, printerDriverIsolationW.as_ptr(), windowsSettings2011NSW.as_ptr()) != 0
            || xml_elem_cmp(&elem, ultraHighResolutionScrollingAwareW.as_ptr(), windowsSettings2017NSW.as_ptr()) != 0
        {
            parse_settings_elem(xmlbuf, assembly, acl, &mut elem);
        } else {
            WARN!(
                b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }
}

unsafe fn parse_application_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, windowsSettingsW.as_ptr(), asmv3W.as_ptr()) != 0 {
            parse_windows_settings_elem(xmlbuf, assembly, acl, &elem);
        } else {
            parse_unknown_elem(xmlbuf, &elem);
        }
    }
}

unsafe fn parse_requested_execution_level_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    _acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    const levelW: [WCHAR; 6] = [b'l' as WCHAR, b'e' as WCHAR, b'v' as WCHAR, b'e' as WCHAR, b'l' as WCHAR, 0];
    const asInvokerW: [WCHAR; 10] = [b'a' as WCHAR, b's' as WCHAR, b'I' as WCHAR, b'n' as WCHAR, b'v' as WCHAR, b'o' as WCHAR, b'k' as WCHAR, b'e' as WCHAR, b'r' as WCHAR, 0];
    const requireAdministratorW: [WCHAR; 19] = [b'r' as WCHAR, b'e' as WCHAR, b'q' as WCHAR, b'u' as WCHAR, b'i' as WCHAR, b'r' as WCHAR, b'e' as WCHAR, b'A' as WCHAR, b'd' as WCHAR, b'm' as WCHAR, b'i' as WCHAR, b'n' as WCHAR, b'i' as WCHAR, b's' as WCHAR, b't' as WCHAR, b'r' as WCHAR, b'a' as WCHAR, b't' as WCHAR, b'o' as WCHAR];
    const requireAdministratorW2: [WCHAR; 20] = [b'r' as WCHAR, b'e' as WCHAR, b'q' as WCHAR, b'u' as WCHAR, b'i' as WCHAR, b'r' as WCHAR, b'e' as WCHAR, b'A' as WCHAR, b'd' as WCHAR, b'm' as WCHAR, b'i' as WCHAR, b'n' as WCHAR, b'i' as WCHAR, b's' as WCHAR, b't' as WCHAR, b'r' as WCHAR, b'a' as WCHAR, b't' as WCHAR, b'o' as WCHAR, b'r' as WCHAR];
    const requireAdministratorWfull: [WCHAR; 21] = [b'r' as WCHAR, b'e' as WCHAR, b'q' as WCHAR, b'u' as WCHAR, b'i' as WCHAR, b'r' as WCHAR, b'e' as WCHAR, b'A' as WCHAR, b'd' as WCHAR, b'm' as WCHAR, b'i' as WCHAR, b'n' as WCHAR, b'i' as WCHAR, b's' as WCHAR, b't' as WCHAR, b'r' as WCHAR, b'a' as WCHAR, b't' as WCHAR, b'o' as WCHAR, b'r' as WCHAR, 0];
    const highestAvailableW: [WCHAR; 16] = [b'h' as WCHAR, b'i' as WCHAR, b'g' as WCHAR, b'h' as WCHAR, b'e' as WCHAR, b's' as WCHAR, b't' as WCHAR, b'A' as WCHAR, b'v' as WCHAR, b'a' as WCHAR, b'i' as WCHAR, b'l' as WCHAR, b'a' as WCHAR, b'b' as WCHAR, b'l' as WCHAR, 0];
    const highestAvailableWfull: [WCHAR; 17] = [b'h' as WCHAR, b'i' as WCHAR, b'g' as WCHAR, b'h' as WCHAR, b'e' as WCHAR, b's' as WCHAR, b't' as WCHAR, b'A' as WCHAR, b'v' as WCHAR, b'a' as WCHAR, b'i' as WCHAR, b'l' as WCHAR, b'a' as WCHAR, b'b' as WCHAR, b'l' as WCHAR, b'e' as WCHAR, 0];
    const uiAccessW: [WCHAR; 9] = [b'u' as WCHAR, b'i' as WCHAR, b'A' as WCHAR, b'c' as WCHAR, b'c' as WCHAR, b'e' as WCHAR, b's' as WCHAR, b's' as WCHAR, 0];
    const falseW: [WCHAR; 6] = [b'f' as WCHAR, b'a' as WCHAR, b'l' as WCHAR, b's' as WCHAR, b'e' as WCHAR, 0];
    const trueW: [WCHAR; 5] = [b't' as WCHAR, b'r' as WCHAR, b'u' as WCHAR, b'e' as WCHAR, 0];

    let mut elem: xml_elem = core::mem::zeroed();
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;

    if (*assembly).run_level != ACTCTX_RUN_LEVEL_UNSPECIFIED {
        set_error(xmlbuf);
    }

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, levelW.as_ptr()) != 0 {
            if xmlstr_cmpi(core::ptr::addr_of!(attr.value), asInvokerW.as_ptr()) != 0 {
                (*assembly).run_level = ACTCTX_RUN_LEVEL_AS_INVOKER;
            } else if xmlstr_cmpi(core::ptr::addr_of!(attr.value), highestAvailableWfull.as_ptr()) != 0 {
                (*assembly).run_level = ACTCTX_RUN_LEVEL_HIGHEST_AVAILABLE;
            } else if xmlstr_cmpi(core::ptr::addr_of!(attr.value), requireAdministratorWfull.as_ptr()) != 0 {
                (*assembly).run_level = ACTCTX_RUN_LEVEL_REQUIRE_ADMIN;
            } else {
                FIXME!(
                    "unknown execution level: %s\n",
                    debugstr_xmlstr(core::ptr::addr_of!(attr.value)),
                );
            }
        } else if xml_attr_cmp(&attr, uiAccessW.as_ptr()) != 0 {
            if xmlstr_cmpi(core::ptr::addr_of!(attr.value), falseW.as_ptr()) != 0 {
                (*assembly).ui_access = 0;
            } else if xmlstr_cmpi(core::ptr::addr_of!(attr.value), trueW.as_ptr()) != 0 {
                (*assembly).ui_access = 1;
            } else {
                FIXME!(
                    "unknown uiAccess value: %s\n",
                    debugstr_xmlstr(core::ptr::addr_of!(attr.value)),
                );
            }
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    if end != 0 {
        return;
    }

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        WARN!(
            b"unknown element %s\n\0".as_ptr() as *const core::ffi::c_char,
            debugstr_xml_elem(&elem),
        );
        parse_unknown_elem(xmlbuf, &elem);
    }
}

unsafe fn parse_requested_privileges_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, requestedExecutionLevelW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_requested_execution_level_elem(xmlbuf, assembly, acl, &elem);
        } else {
            WARN!(
                b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }
}

unsafe fn parse_security_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, requestedPrivilegesW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_requested_privileges_elem(xmlbuf, assembly, acl, &elem);
        } else {
            WARN!(
                b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }
}

unsafe fn parse_trust_info_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
) {
    let mut elem: xml_elem = core::mem::zeroed();

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if xml_elem_cmp(&elem, securityW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_security_elem(xmlbuf, assembly, acl, &elem);
        } else {
            WARN!(
                b"unknown elem %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }
}

unsafe fn parse_assembly_elem(
    xmlbuf: *mut xmlbuf_t,
    assembly: *mut assembly,
    acl: *mut actctx_loader,
    parent: *const xml_elem,
    expected_ai: *mut assembly_identity,
) {
    let mut elem: xml_elem = core::mem::zeroed();
    let mut attr: xml_attr = core::mem::zeroed();
    let mut end: BOOL = 0;
    let mut version: BOOL = 0;

    TRACE!(b"(%p)\n\0".as_ptr() as *const core::ffi::c_char, xmlbuf);

    while next_xml_attr(xmlbuf, &mut attr, &mut end) != 0 {
        if xml_attr_cmp(&attr, manifestVersionW.as_ptr()) != 0 {
            static v10W: [WCHAR; 4] = [b'1' as WCHAR, b'.' as WCHAR, b'0' as WCHAR, 0];
            if xmlstr_cmp(core::ptr::addr_of!(attr.value), v10W.as_ptr()) == 0 {
                FIXME!(
                    "wrong version %s\n",
                    debugstr_xmlstr(core::ptr::addr_of!(attr.value)),
                );
                break;
            }
            version = 1;
        } else if is_xmlns_attr(&attr) == 0 {
            WARN!(
                b"unknown attr %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_attr(&attr),
            );
        }
    }

    if end != 0 || version == 0 {
        set_error(xmlbuf);
        return;
    }

    while next_xml_elem(xmlbuf, &mut elem, parent) != 0 {
        if (*assembly).type_ == APPLICATION_MANIFEST
            && xml_elem_cmp(&elem, noInheritW.as_ptr(), asmv1W.as_ptr()) != 0
        {
            parse_noinherit_elem(xmlbuf, &elem);
            (*assembly).no_inherit = 1;
        } else if xml_elem_cmp(&elem, noInheritableW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_noinheritable_elem(xmlbuf, &elem);
        } else if xml_elem_cmp(&elem, descriptionW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_description_elem(xmlbuf, &elem);
        } else if xml_elem_cmp(&elem, comInterfaceExternalProxyStubW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_com_interface_external_proxy_stub_elem(xmlbuf, assembly, acl, &elem);
        } else if xml_elem_cmp(&elem, dependencyW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_dependency_elem(xmlbuf, acl, &elem);
        } else if xml_elem_cmp(&elem, fileW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_file_elem(xmlbuf, assembly, acl, &elem);
        } else if xml_elem_cmp(&elem, clrClassW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_clr_class_elem(xmlbuf, assembly, acl, &elem);
        } else if xml_elem_cmp(&elem, clrSurrogateW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_clr_surrogate_elem(xmlbuf, assembly, acl, &elem);
        } else if xml_elem_cmp(&elem, trustInfoW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_trust_info_elem(xmlbuf, assembly, acl, &elem);
        } else if xml_elem_cmp(&elem, assemblyIdentityW.as_ptr(), asmv1W.as_ptr()) != 0 {
            parse_assembly_identity_elem(
                xmlbuf,
                (*acl).actctx,
                core::ptr::addr_of_mut!((*assembly).id),
                &elem,
            );

            if (*xmlbuf).error == 0 && !expected_ai.is_null() {
                if (*assembly).type_ == ASSEMBLY_MANIFEST
                    && memcmp(
                        core::ptr::addr_of!((*assembly).id.version) as *const core::ffi::c_void,
                        core::ptr::addr_of!((*expected_ai).version) as *const core::ffi::c_void,
                        core::mem::size_of_val(&(*assembly).id.version),
                    ) != 0
                {
                    FIXME!(
                        "wrong version for assembly manifest: %u.%u.%u.%u / %u.%u.%u.%u\n",
                        (*expected_ai).version.major as core::ffi::c_uint,
                        (*expected_ai).version.minor as core::ffi::c_uint,
                        (*expected_ai).version.build as core::ffi::c_uint,
                        (*expected_ai).version.revision as core::ffi::c_uint,
                        (*assembly).id.version.major as core::ffi::c_uint,
                        (*assembly).id.version.minor as core::ffi::c_uint,
                        (*assembly).id.version.build as core::ffi::c_uint,
                        (*assembly).id.version.revision as core::ffi::c_uint,
                    );
                    set_error(xmlbuf);
                } else if (*assembly).type_ == ASSEMBLY_SHARED_MANIFEST
                    && ((*assembly).id.version.major != (*expected_ai).version.major
                        || (*assembly).id.version.minor != (*expected_ai).version.minor
                        || (*assembly).id.version.build < (*expected_ai).version.build
                        || ((*assembly).id.version.build == (*expected_ai).version.build
                            && (*assembly).id.version.revision < (*expected_ai).version.revision))
                {
                    FIXME!("wrong version for shared assembly manifest\n");
                    set_error(xmlbuf);
                }
            }
        } else if xml_elem_cmp(&elem, compatibilityW.as_ptr(), compatibilityNSW.as_ptr()) != 0 {
            parse_compatibility_elem(xmlbuf, assembly, acl, &elem);
        } else if xml_elem_cmp(&elem, applicationW.as_ptr(), asmv3W.as_ptr()) != 0 {
            parse_application_elem(xmlbuf, assembly, acl, &elem);
        } else {
            WARN!(
                b"unknown element %s\n\0".as_ptr() as *const core::ffi::c_char,
                debugstr_xml_elem(&elem),
            );
            parse_unknown_elem(xmlbuf, &elem);
        }
    }

    if ((*assembly).type_ == ASSEMBLY_MANIFEST || (*assembly).type_ == ASSEMBLY_SHARED_MANIFEST)
        && (*assembly).no_inherit != 0
    {
        set_error(xmlbuf);
    }
}

unsafe fn parse_manifest_buffer(
    acl: *mut actctx_loader,
    assembly: *mut assembly,
    ai: *mut assembly_identity,
    xmlbuf: *mut xmlbuf_t,
) -> NTSTATUS {
    let mut elem: xml_elem = core::mem::zeroed();
    let parent: xml_elem = core::mem::zeroed();

    (*xmlbuf).error = 0;
    (*xmlbuf).ns_pos = 0;

    if next_xml_elem(xmlbuf, &mut elem, &parent) == 0 {
        return STATUS_SXS_CANT_GEN_ACTCTX;
    }

    if xmlstr_cmp(core::ptr::addr_of!(elem.name), g_xmlW.as_ptr()) != 0
        && (parse_xml_header(xmlbuf) == 0 || next_xml_elem(xmlbuf, &mut elem, &parent) == 0)
    {
        return STATUS_SXS_CANT_GEN_ACTCTX;
    }

    if xml_elem_cmp(&elem, assemblyW.as_ptr(), asmv1W.as_ptr()) == 0 {
        FIXME!(
            "root element is %s, not <assembly>\n",
            debugstr_xml_elem(&elem),
        );
        return STATUS_SXS_CANT_GEN_ACTCTX;
    }

    parse_assembly_elem(xmlbuf, assembly, acl, &elem, ai);
    if (*xmlbuf).error != 0 {
        FIXME!(
            "failed to parse manifest %s\n",
            debugstr_w((*assembly).manifest.info),
        );
        return STATUS_SXS_CANT_GEN_ACTCTX;
    }

    if next_xml_elem(xmlbuf, &mut elem, &parent) != 0 {
        FIXME!(
            "unexpected element %s\n",
            debugstr_xml_elem(&elem),
        );
        return STATUS_SXS_CANT_GEN_ACTCTX;
    }

    if (*xmlbuf).ptr != (*xmlbuf).end {
        FIXME!("parse error\n");
        return STATUS_SXS_CANT_GEN_ACTCTX;
    }
    STATUS_SUCCESS
}

unsafe fn parse_manifest(
    acl: *mut actctx_loader,
    ai: *mut assembly_identity,
    filename: LPCWSTR,
    module: HANDLE,
    directory: LPCWSTR,
    shared: BOOL,
    buffer: *const core::ffi::c_void,
    size: SIZE_T,
) -> NTSTATUS {
    let mut xmlbuf: xmlbuf_t = core::mem::zeroed();
    let mut status: NTSTATUS;
    let assembly: *mut assembly;
    let mut unicode_tests: core::ffi::c_int;

    TRACE!(b"parsing manifest loaded from %s base dir %s\n\0".as_ptr() as *const i8,
          debugstr_w(filename), debugstr_w(directory));

    assembly = add_assembly(
        (*acl).actctx,
        if shared != 0 { ASSEMBLY_SHARED_MANIFEST } else { ASSEMBLY_MANIFEST },
    );
    if assembly.is_null() {
        return STATUS_SXS_CANT_GEN_ACTCTX;
    }

    if !directory.is_null() {
        (*assembly).directory = strdupW(directory);
        if (*assembly).directory.is_null() {
            return STATUS_NO_MEMORY;
        }
    }

    if filename.is_null() {
        let mut module_path: _UNICODE_STRING = core::mem::zeroed();
        status = get_module_filename(module as HMODULE, &mut module_path, 0);
        if status != 0 {
            return status;
        }
        (*assembly).manifest.info = module_path.Buffer;
    } else {
        (*assembly).manifest.info = strdupW(filename.add(4));
        if (*assembly).manifest.info.is_null() {
            return STATUS_NO_MEMORY;
        }
    }

    (*assembly).manifest.type_ = if !(*assembly).manifest.info.is_null() {
        ACTIVATION_CONTEXT_PATH_TYPE_WIN32_FILE
    } else {
        ACTIVATION_CONTEXT_PATH_TYPE_NONE
    } as ULONG;

    unicode_tests = (IS_TEXT_UNICODE_SIGNATURE | IS_TEXT_UNICODE_REVERSE_SIGNATURE) as core::ffi::c_int;
    if RtlIsTextUnicode(buffer as *mut core::ffi::c_void, size as core::ffi::c_int, &mut unicode_tests) != 0 {
        xmlbuf.ptr = buffer as *const WCHAR;
        xmlbuf.end = xmlbuf.ptr.add(size as usize / core::mem::size_of::<WCHAR>());
        status = parse_manifest_buffer(acl, assembly, ai, &mut xmlbuf);
    } else if unicode_tests & (IS_TEXT_UNICODE_REVERSE_SIGNATURE as core::ffi::c_int) != 0 {
        let buf: *const WCHAR = buffer as *const WCHAR;
        let new_buff: *mut WCHAR;
        let mut i: core::ffi::c_uint;

        new_buff = RtlAllocateHeap(GetProcessHeap(), 0, size as usize) as *mut WCHAR;
        if new_buff.is_null() {
            return STATUS_NO_MEMORY;
        }
        i = 0;
        while (i as usize) < size as usize / core::mem::size_of::<WCHAR>() {
            *new_buff.add(i as usize) = RtlUshortByteSwap(*buf.add(i as usize));
            i += 1;
        }
        xmlbuf.ptr = new_buff;
        xmlbuf.end = xmlbuf.ptr.add(size as usize / core::mem::size_of::<WCHAR>());
        status = parse_manifest_buffer(acl, assembly, ai, &mut xmlbuf);
        RtlFreeHeap(GetProcessHeap(), 0, new_buff as *mut core::ffi::c_void);
    } else {
        let mut len: DWORD = 0;
        let new_buff: *mut WCHAR;

        status = RtlUTF8ToUnicodeN(core::ptr::null_mut(), 0, &mut len, buffer, size as u32);
        if !NT_SUCCESS(status) {
            return STATUS_SXS_CANT_GEN_ACTCTX;
        }

        new_buff = RtlAllocateHeap(GetProcessHeap(), 0, len as usize) as *mut WCHAR;
        if new_buff.is_null() {
            return STATUS_NO_MEMORY;
        }
        status = RtlUTF8ToUnicodeN(new_buff as *mut core::ffi::c_void, len, &mut len, buffer, size as u32);
        if !NT_SUCCESS(status) {
            return STATUS_SXS_CANT_GEN_ACTCTX;
        }

        xmlbuf.ptr = new_buff;
        xmlbuf.end = xmlbuf.ptr.add(len as usize / core::mem::size_of::<WCHAR>());
        status = parse_manifest_buffer(acl, assembly, ai, &mut xmlbuf);
        RtlFreeHeap(GetProcessHeap(), 0, new_buff as *mut core::ffi::c_void);
    }
    status
}

unsafe fn open_nt_file(handle: *mut HANDLE, name: *mut _UNICODE_STRING) -> NTSTATUS {
    let mut attr: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut io: _IO_STATUS_BLOCK = core::mem::zeroed();

    attr.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    attr.RootDirectory = core::ptr::null_mut();
    attr.Attributes = OBJ_CASE_INSENSITIVE;
    attr.ObjectName = name;
    attr.SecurityDescriptor = core::ptr::null_mut();
    attr.SecurityQualityOfService = core::ptr::null_mut();
    NtOpenFile(
        handle,
        GENERIC_READ | SYNCHRONIZE,
        &mut attr,
        &mut io,
        FILE_SHARE_READ,
        FILE_SYNCHRONOUS_IO_ALERT,
    )
}

unsafe fn get_manifest_in_module(
    acl: *mut actctx_loader,
    ai: *mut assembly_identity,
    filename: LPCWSTR,
    directory: LPCWSTR,
    shared: BOOL,
    hModule: HANDLE,
    resname: LPCWSTR,
    lang: ULONG,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut nameW: _UNICODE_STRING = core::mem::zeroed();
    let mut info: LDR_RESOURCE_INFO = core::mem::zeroed();
    let mut entry: *mut IMAGE_RESOURCE_DATA_ENTRY = core::ptr::null_mut();
    let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();

    if resname.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    info.Type = RT_MANIFEST as _;
    info.Language = lang as _;

    if (resname as ULONG_PTR) >> 16 == 0 {
        info.Name = resname as _;
        status = LdrFindResource_U(hModule, &mut info, 3, &mut entry);
    } else if *resname == ('#' as u16) {
        let mut value: ULONG = 0;
        RtlInitUnicodeString(&mut nameW, resname.add(1));
        if RtlUnicodeStringToInteger(&mut nameW, 10, &mut value) != STATUS_SUCCESS
            || HIWORD!(value) != 0
        {
            return STATUS_INVALID_PARAMETER;
        }
        info.Name = value as _;
        status = LdrFindResource_U(hModule, &mut info, 3, &mut entry);
    } else {
        RtlCreateUnicodeString(&mut nameW, resname);
        RtlUpcaseUnicodeString(&mut nameW, &mut nameW, 0u8);
        info.Name = nameW.Buffer as _;
        status = LdrFindResource_U(hModule, &mut info, 3, &mut entry);
        RtlFreeUnicodeString(&mut nameW);
    }

    if status == STATUS_SUCCESS {
        status = LdrAccessResource(hModule, entry, &mut ptr, core::ptr::null_mut());
    }

    if status == STATUS_SUCCESS {
        status = parse_manifest(
            acl,
            ai,
            filename,
            hModule,
            directory,
            shared,
            ptr,
            (*entry).Size as SIZE_T,
        );
    }

    status
}

unsafe fn find_first_id_entry(
    dir: *mut IMAGE_RESOURCE_DIRECTORY,
    root: *mut core::ffi::c_void,
    want_dir: core::ffi::c_int,
) -> *mut IMAGE_RESOURCE_DIRECTORY {
    let entry = dir.offset(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;

    let mut pos = (*dir).NumberOfNamedEntries as core::ffi::c_int;
    let end = (*dir).NumberOfNamedEntries as core::ffi::c_int + (*dir).NumberOfIdEntries as core::ffi::c_int;
    while pos < end {
        let e = &*entry.offset(pos as isize);
        let data_is_dir = e.__bindgen_anon_2.__bindgen_anon_1.DataIsDirectory();
        if ((data_is_dir == 0) as core::ffi::c_int) == ((want_dir == 0) as core::ffi::c_int) {
            return (root as *mut core::ffi::c_char)
                .offset(e.__bindgen_anon_2.__bindgen_anon_1.OffsetToDirectory() as isize)
                as *mut IMAGE_RESOURCE_DIRECTORY;
        }
        pos += 1;
    }
    core::ptr::null_mut()
}

unsafe fn search_manifest_in_module(
    acl: *mut actctx_loader,
    ai: *mut assembly_identity,
    filename: LPCWSTR,
    directory: LPCWSTR,
    shared: BOOL,
    hModule: HANDLE,
    _lang: ULONG,
) -> NTSTATUS {
    let mut size: ULONG = 0;
    let mut ptr: PVOID;
    let mut resdirptr: *mut IMAGE_RESOURCE_DIRECTORY;
    let entry: *mut IMAGE_RESOURCE_DATA_ENTRY;
    let mut status: NTSTATUS;

    let root = RtlImageDirectoryEntryToData(
        hModule,
        1u8,
        IMAGE_DIRECTORY_ENTRY_RESOURCE as USHORT,
        &mut size,
    );
    if root.is_null() {
        return STATUS_RESOURCE_DATA_NOT_FOUND;
    }
    if (size as usize) < core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>() {
        return STATUS_RESOURCE_DATA_NOT_FOUND;
    }
    resdirptr = root as *mut IMAGE_RESOURCE_DIRECTORY;

    ptr = find_entry_by_name(resdirptr, RT_MANIFEST as LPCWSTR, root, 1) as PVOID;
    if ptr.is_null() {
        return STATUS_RESOURCE_TYPE_NOT_FOUND;
    }

    resdirptr = ptr as *mut IMAGE_RESOURCE_DIRECTORY;
    ptr = find_first_id_entry(resdirptr, root, 1) as PVOID;
    if ptr.is_null() {
        return STATUS_RESOURCE_TYPE_NOT_FOUND;
    }

    resdirptr = ptr as *mut IMAGE_RESOURCE_DIRECTORY;
    ptr = find_first_entry(resdirptr, root, 0) as PVOID;
    if ptr.is_null() {
        return STATUS_RESOURCE_TYPE_NOT_FOUND;
    }

    entry = ptr as *mut IMAGE_RESOURCE_DATA_ENTRY;
    status = LdrAccessResource(hModule, entry, &mut ptr, core::ptr::null_mut());

    if status == STATUS_SUCCESS {
        status = parse_manifest(
            acl,
            ai,
            filename,
            hModule,
            directory,
            shared,
            ptr,
            (*entry).Size as SIZE_T,
        );
    }

    status
}

unsafe fn get_manifest_in_pe_file(
    acl: *mut actctx_loader,
    ai: *mut assembly_identity,
    filename: LPCWSTR,
    directory: LPCWSTR,
    shared: BOOL,
    file: HANDLE,
    resname: LPCWSTR,
    lang: ULONG,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let base: *mut core::ffi::c_void;
    let mut resnameBuf: [u16; 20] = [0; 20];
    let mut resptr: LPCWSTR = resname;
    let mut info: ntkrnl_sys::ddk::_FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut io: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut offset: LARGE_INTEGER = core::mem::zeroed();
    let count: usize;

    if (resname as ULONG_PTR) >> 16 == 0 {
        _swprintf(
            resnameBuf.as_mut_ptr(),
            w!("#").as_ptr() as LPCWSTR,
            PtrToUlong(resname as *mut core::ffi::c_void),
        );
        resptr = resnameBuf.as_ptr();
    }
    let _ = resptr;

    status = NtQueryInformationFile(
        file,
        &mut io,
        &mut info as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<ntkrnl_sys::ddk::_FILE_STANDARD_INFORMATION>() as ULONG,
        // FileEndOfFileInformation is SET-only on NT: the query returns
        // STATUS_INVALID_INFO_CLASS (ERROR_INVALID_PARAMETER), so every manifest-file
        // activation context failed and comctl32 never got its v5/v6 context. Wine's
        // ntdll accepts that query, which is why the Wine-derived code (and upstream
        // ReactOS) uses it. FileStandardInformation carries the same EndOfFile.
        ntkrnl_sys::ddk::_FILE_INFORMATION_CLASS_FileStandardInformation as _,
    );
    if status != STATUS_SUCCESS {
        return status;
    }
    if info.EndOfFile.QuadPart == 0 || info.EndOfFile.QuadPart > 64 * 1024 * 1024 {
        return STATUS_INVALID_IMAGE_FORMAT;
    }

    count = info.EndOfFile.QuadPart as usize;
    base = RtlAllocateHeap(GetProcessHeap(), 0, count);
    if base.is_null() {
        return STATUS_NO_MEMORY;
    }

    offset.QuadPart = 0;
    status = NtReadFile(
        file,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut io,
        base,
        count as ULONG,
        &mut offset,
        core::ptr::null_mut(),
    );
    if status != STATUS_SUCCESS {
        RtlFreeHeap(GetProcessHeap(), 0, base);
        return status;
    }

    if !RtlImageNtHeader(base).is_null() {
        let module: HANDLE = ((base as ULONG_PTR) | 1) as HANDLE;
        if !resname.is_null() {
            status = get_manifest_in_module(acl, ai, filename, directory, shared, module, resname, lang);
        } else {
            status = search_manifest_in_module(acl, ai, filename, directory, shared, module, lang);
        }
    } else {
        status = STATUS_INVALID_IMAGE_FORMAT;
    }

    RtlFreeHeap(GetProcessHeap(), 0, base);
    status
}

unsafe fn get_manifest_in_manifest_file(
    acl: *mut actctx_loader,
    ai: *mut assembly_identity,
    filename: LPCWSTR,
    directory: LPCWSTR,
    shared: BOOL,
    file: HANDLE,
) -> NTSTATUS {
    let mut info: ntkrnl_sys::ddk::_FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut io: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut offset: LARGE_INTEGER = core::mem::zeroed();
    let mut status: NTSTATUS;
    let count: SIZE_T;
    let base: *mut core::ffi::c_void;

    TRACE!(b"loading manifest file %s\n\0".as_ptr() as *const i8, debugstr_w(filename));

    status = NtQueryInformationFile(
        file,
        &mut io,
        &mut info as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<ntkrnl_sys::ddk::_FILE_STANDARD_INFORMATION>() as ULONG,
        // FileEndOfFileInformation is SET-only on NT: the query returns
        // STATUS_INVALID_INFO_CLASS (ERROR_INVALID_PARAMETER), so every manifest-file
        // activation context failed and comctl32 never got its v5/v6 context. Wine's
        // ntdll accepts that query, which is why the Wine-derived code (and upstream
        // ReactOS) uses it. FileStandardInformation carries the same EndOfFile.
        ntkrnl_sys::ddk::_FILE_INFORMATION_CLASS_FileStandardInformation as _,
    );
    if status != STATUS_SUCCESS {
        return status;
    }
    if info.EndOfFile.QuadPart == 0 || info.EndOfFile.QuadPart > 1024 * 1024 {
        return STATUS_SXS_CANT_GEN_ACTCTX;
    }

    count = info.EndOfFile.QuadPart as SIZE_T;
    base = RtlAllocateHeap(GetProcessHeap(), 0, count);
    if base.is_null() {
        return STATUS_NO_MEMORY;
    }

    offset.QuadPart = 0;
    status = NtReadFile(
        file,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut io,
        base,
        count as ULONG,
        &mut offset,
        core::ptr::null_mut(),
    );
    if status == STATUS_SUCCESS {
        status = parse_manifest(
            acl,
            ai,
            filename,
            core::ptr::null_mut(),
            directory,
            shared,
            base,
            count as SIZE_T,
        );
    }

    RtlFreeHeap(GetProcessHeap(), 0, base);
    status
}

unsafe fn get_manifest_in_associated_manifest(
    acl: *mut actctx_loader,
    ai: *mut assembly_identity,
    filename: LPCWSTR,
    directory: LPCWSTR,
    module: HMODULE,
    resname: LPCWSTR,
) -> NTSTATUS {
    static fmtW: [WCHAR; 5] = [b'.' as u16, b'%' as u16, b'l' as u16, b'u' as u16, 0];
    let buffer: *mut WCHAR;
    let mut status: NTSTATUS;
    let mut nameW: _UNICODE_STRING = core::mem::zeroed();
    let mut file: HANDLE = core::ptr::null_mut();
    let mut resid: ULONG_PTR = CREATEPROCESS_MANIFEST_RESOURCE_ID as ULONG_PTR;

    if ((resname as ULONG_PTR) >> 16) == 0 {
        resid = (resname as ULONG_PTR) & 0xffff;
    }

    TRACE!(b"looking for manifest associated with %s id %lu\n\0".as_ptr() as *const i8, debugstr_w(filename), resid);

    if !module.is_null() {
        let mut name: _UNICODE_STRING = core::mem::zeroed();

        status = get_module_filename(
            module,
            &mut name,
            (core::mem::size_of_val(&dotManifestW) + 10 * core::mem::size_of::<WCHAR>()) as core::ffi::c_uint,
        );
        if status == 0 {
            if resid != 1 {
                swprintf(name.Buffer.add(wcslen(name.Buffer)), 10, fmtW.as_ptr(), resid);
            }
            wcscat(name.Buffer, dotManifestW.as_ptr());
            if RtlDosPathNameToNtPathName_U(name.Buffer, &mut nameW, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
                status = STATUS_RESOURCE_DATA_NOT_FOUND;
            }
            RtlFreeUnicodeString(&mut name);
        }
        if status != 0 {
            return status;
        }
    } else {
        buffer = RtlAllocateHeap(
            GetProcessHeap(),
            0,
            (wcslen(filename) + 10) * core::mem::size_of::<WCHAR>() + core::mem::size_of_val(&dotManifestW),
        ) as *mut WCHAR;
        if buffer.is_null() {
            return STATUS_NO_MEMORY;
        }
        wcscpy(buffer, filename);
        if resid != 1 {
            swprintf(buffer.add(wcslen(buffer)), 10, fmtW.as_ptr(), resid);
        }
        wcscat(buffer, dotManifestW.as_ptr());
        RtlInitUnicodeString(&mut nameW, buffer);
    }

    if open_nt_file(&mut file, &mut nameW) == 0 {
        status = get_manifest_in_manifest_file(acl, ai, nameW.Buffer, directory, 0, file);
        NtClose(file);
    } else {
        status = STATUS_RESOURCE_TYPE_NOT_FOUND;
    }
    RtlFreeUnicodeString(&mut nameW);
    status
}

unsafe fn lookup_manifest_file(dir: HANDLE, ai: *mut assembly_identity) -> *mut WCHAR {
    static lookup_fmtW: [WCHAR; 37] = [
        b'%' as WCHAR, b's' as WCHAR, b'_' as WCHAR, b'%' as WCHAR, b's' as WCHAR, b'_' as WCHAR,
        b'%' as WCHAR, b's' as WCHAR, b'_' as WCHAR, b'%' as WCHAR, b'u' as WCHAR, b'.' as WCHAR,
        b'%' as WCHAR, b'u' as WCHAR, b'.' as WCHAR, b'*' as WCHAR, b'.' as WCHAR, b'*' as WCHAR,
        b'_' as WCHAR, b'%' as WCHAR, b's' as WCHAR, b'_' as WCHAR, b'*' as WCHAR, b'.' as WCHAR,
        b'm' as WCHAR, b'a' as WCHAR, b'n' as WCHAR, b'i' as WCHAR, b'f' as WCHAR, b'e' as WCHAR,
        b's' as WCHAR, b't' as WCHAR, 0,
        0, 0, 0, 0,
    ];
    static wine_trailerW: [WCHAR; 17] = [
        b'd' as WCHAR, b'e' as WCHAR, b'a' as WCHAR, b'd' as WCHAR, b'b' as WCHAR, b'e' as WCHAR,
        b'e' as WCHAR, b'f' as WCHAR, b'.' as WCHAR, b'm' as WCHAR, b'a' as WCHAR, b'n' as WCHAR,
        b'i' as WCHAR, b'f' as WCHAR, b'e' as WCHAR, b's' as WCHAR, b't' as WCHAR,
    ];

    let lookup: *mut WCHAR;
    let mut ret: *mut WCHAR = core::ptr::null_mut();
    let mut lookup_us: _UNICODE_STRING = core::mem::zeroed();
    let mut io: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut lang = (*ai).language as *const WCHAR;
    let mut data_pos: u32 = 0;
    let mut data_len: u32;
    let len: u32;
    let mut buffer: [u8; 8192] = [0; 8192];

    if lang.is_null() || wcsicmp(lang, neutralW.as_ptr()) == 0 {
        lang = wildcardW.as_ptr();
    }

    len = (wcslen((*ai).arch) as u32) + (wcslen((*ai).name) as u32)
        + (wcslen((*ai).public_key) as u32) + (wcslen(lang) as u32) + 20
        + (lookup_fmtW.len() as u32);

    lookup = RtlAllocateHeap(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
    if lookup.is_null() {
        return core::ptr::null_mut();
    }

    swprintf(lookup, len as usize, lookup_fmtW.as_ptr(), (*ai).arch, (*ai).name, (*ai).public_key,
             (*ai).version.major as core::ffi::c_uint, (*ai).version.minor as core::ffi::c_uint, lang);
    RtlInitUnicodeString(&mut lookup_us, lookup);

    if NtQueryDirectoryFile(dir, core::ptr::null_mut(), None, core::ptr::null_mut(), &mut io,
                            buffer.as_mut_ptr() as *mut core::ffi::c_void, buffer.len() as u32,
                            FileBothDirectoryInformation, 0u8, &mut lookup_us, 1u8) == 0
    {
        let mut min_build: u32 = (*ai).version.build as u32;
        let mut min_revision: u32 = (*ai).version.revision as u32;
        let mut dir_info: *mut FILE_BOTH_DIR_INFORMATION;
        let mut tmp: *mut WCHAR;
        let mut build: u32;
        let mut revision: u32;

        data_len = io.Information as u32;

        loop {
            if data_pos >= data_len {
                if NtQueryDirectoryFile(dir, core::ptr::null_mut(), None, core::ptr::null_mut(), &mut io,
                                        buffer.as_mut_ptr() as *mut core::ffi::c_void, buffer.len() as u32,
                                        FileBothDirectoryInformation, 0u8, &mut lookup_us, 0u8) != 0
                {
                    break;
                }
                data_len = io.Information as u32;
                data_pos = 0;
            }
            dir_info = buffer.as_mut_ptr().add(data_pos as usize) as *mut FILE_BOTH_DIR_INFORMATION;

            if (*dir_info).NextEntryOffset != 0 {
                data_pos += (*dir_info).NextEntryOffset;
            } else {
                data_pos = data_len;
            }

            tmp = ((*dir_info).FileName.as_mut_ptr())
                .offset(wcschr(lookup, b'*' as core::ffi::c_int).offset_from(lookup));
            build = wcstoul(tmp, core::ptr::null_mut(), 10) as u32;
            if build < min_build {
                continue;
            }
            tmp = wcschr(tmp, b'.' as core::ffi::c_int).add(1);
            revision = wcstoul(tmp, core::ptr::null_mut(), 10) as u32;
            if build == min_build && revision < min_revision {
                continue;
            }
            tmp = wcschr(tmp, b'_' as core::ffi::c_int).add(1);
            tmp = wcschr(tmp, b'_' as core::ffi::c_int).add(1);
            if (*dir_info).FileNameLength as usize
                - (tmp.offset_from((*dir_info).FileName.as_mut_ptr()) as usize) * core::mem::size_of::<WCHAR>()
                == core::mem::size_of_val(&wine_trailerW)
                && wcsnicmp(tmp, wine_trailerW.as_ptr(), wine_trailerW.len()) == 0
            {
                if !ret.is_null() {
                    continue;
                }
            } else {
                min_build = build;
                min_revision = revision;
            }
            (*ai).version.build = build as USHORT;
            (*ai).version.revision = revision as USHORT;
            RtlFreeHeap(GetProcessHeap(), 0, ret as *mut core::ffi::c_void);
            ret = RtlAllocateHeap(GetProcessHeap(), 0,
                                  (*dir_info).FileNameLength as usize + core::mem::size_of::<WCHAR>()) as *mut WCHAR;
            if !ret.is_null() {
                core::ptr::copy_nonoverlapping((*dir_info).FileName.as_ptr() as *const u8, ret as *mut u8,
                                               (*dir_info).FileNameLength as usize);
                *ret.add((*dir_info).FileNameLength as usize / core::mem::size_of::<WCHAR>()) = 0;
            }
        }
    } else {
        WARN!(b"no matching file for %s\n\0".as_ptr(), debugstr_w(lookup));
    }
    RtlFreeHeap(GetProcessHeap(), 0, lookup as *mut core::ffi::c_void);
    ret
}

unsafe fn lookup_winsxs(acl: *mut actctx_loader, ai: *mut assembly_identity) -> NTSTATUS {
    let mut sxs_ai: assembly_identity = core::mem::zeroed();
    let mut path_us: _UNICODE_STRING = core::mem::zeroed();
    let mut attr: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut io: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut path: *mut WCHAR;
    let mut file: *mut WCHAR = core::ptr::null_mut();
    let mut handle: HANDLE = core::mem::zeroed();

    static manifest_dirW: [WCHAR; 18] = [
        b'\\' as WCHAR, b'w' as WCHAR, b'i' as WCHAR, b'n' as WCHAR, b's' as WCHAR,
        b'x' as WCHAR, b's' as WCHAR, b'\\' as WCHAR, b'm' as WCHAR, b'a' as WCHAR,
        b'n' as WCHAR, b'i' as WCHAR, b'f' as WCHAR, b'e' as WCHAR, b's' as WCHAR,
        b't' as WCHAR, b's' as WCHAR, 0,
    ];

    if (*ai).arch.is_null() || (*ai).name.is_null() || (*ai).public_key.is_null() {
        return STATUS_NO_SUCH_FILE;
    }

    path = RtlAllocateHeap(
        GetProcessHeap(),
        0,
        core::mem::size_of_val(&manifest_dirW)
            + wcslen(windows_dir()) * core::mem::size_of::<WCHAR>(),
    ) as *mut WCHAR;
    if path.is_null() {
        return STATUS_NO_MEMORY;
    }

    wcscpy(path, windows_dir());
    wcscat(path, manifest_dirW.as_ptr());

    if RtlDosPathNameToNtPathName_U(
        path,
        &mut path_us,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        RtlFreeHeap(GetProcessHeap(), 0, path as *mut core::ffi::c_void);
        return STATUS_NO_SUCH_FILE;
    }
    RtlFreeHeap(GetProcessHeap(), 0, path as *mut core::ffi::c_void);

    attr.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    attr.RootDirectory = core::ptr::null_mut();
    attr.Attributes = OBJ_CASE_INSENSITIVE;
    attr.ObjectName = &mut path_us;
    attr.SecurityDescriptor = core::ptr::null_mut();
    attr.SecurityQualityOfService = core::ptr::null_mut();

    if NtOpenFile(
        &mut handle,
        GENERIC_READ | SYNCHRONIZE,
        &mut attr,
        &mut io,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
    ) == 0
    {
        sxs_ai = *ai;
        file = lookup_manifest_file(handle, &mut sxs_ai);
        NtClose(handle);
    }
    if file.is_null() {
        RtlFreeUnicodeString(&mut path_us);
        return STATUS_NO_SUCH_FILE;
    }

    path = RtlReAllocateHeap(
        GetProcessHeap(),
        0,
        path_us.Buffer as *mut core::ffi::c_void,
        path_us.Length as usize + (wcslen(file) + 2) * core::mem::size_of::<WCHAR>(),
    ) as *mut WCHAR;
    if path.is_null() {
        RtlFreeHeap(GetProcessHeap(), 0, file as *mut core::ffi::c_void);
        RtlFreeUnicodeString(&mut path_us);
        return STATUS_NO_MEMORY;
    }

    *path.add(path_us.Length as usize / core::mem::size_of::<WCHAR>()) = b'\\' as WCHAR;
    wcscpy(
        path.add(path_us.Length as usize / core::mem::size_of::<WCHAR>() + 1),
        file,
    );
    RtlInitUnicodeString(&mut path_us, path);
    *wcsrchr(file, b'.' as core::ffi::c_int) = 0;

    if open_nt_file(&mut handle, &mut path_us) == 0 {
        io.__bindgen_anon_1.Status = get_manifest_in_manifest_file(
            acl,
            &mut sxs_ai,
            path_us.Buffer,
            file,
            1,
            handle,
        );
        NtClose(handle);
    } else {
        io.__bindgen_anon_1.Status = STATUS_NO_SUCH_FILE;
    }

    RtlFreeHeap(GetProcessHeap(), 0, file as *mut core::ffi::c_void);
    RtlFreeUnicodeString(&mut path_us);
    io.__bindgen_anon_1.Status
}
unsafe fn find_query_actctx( handle: *mut HANDLE, flags: DWORD, class: ULONG ) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;

    if (flags & RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT) != 0
    {
        if !(*handle).is_null() { return STATUS_INVALID_PARAMETER; }

        if !(*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame.is_null()
        {
            *handle = (*((*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame
                as PRTL_ACTIVATION_CONTEXT_STACK_FRAME))
                .ActivationContext as HANDLE;
        }
    }
    else if (flags & (RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_ADDRESS | RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_HMODULE)) != 0
    {
        let mut magic: ULONG_PTR = 0;
        let mut pldr: *mut LDR_DATA_TABLE_ENTRY = core::ptr::null_mut();

        if (*handle).is_null() { return STATUS_INVALID_PARAMETER; }

        LdrLockLoaderLock( 0, core::ptr::null_mut(), core::ptr::addr_of_mut!(magic) as *mut _ );
        if LdrFindEntryForAddress( *handle, core::ptr::addr_of_mut!(pldr) ) == 0
        {
            if (flags & RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_HMODULE) != 0 && *handle != (*pldr).DllBase
            {
                status = STATUS_DLL_NOT_FOUND;
            }
            else
            {
                *handle = (*pldr).EntryPointActivationContext;
            }
        }
        else
        {
            status = STATUS_DLL_NOT_FOUND;
        }
        LdrUnlockLoaderLock( 0, magic as _ );
    }
    else if (*handle).is_null() && (class != ActivationContextBasicInformation as ULONG)
    {
        *handle = process_actctx as HANDLE;
    }

    status
}

unsafe fn build_dllredirect_section(actctx: *mut ACTIVATION_CONTEXT, section: *mut *mut strsection_header) -> NTSTATUS {
    let mut total_len: u32 = 0;
    let mut dll_count: u32 = 0;
    let header: *mut strsection_header;
    let mut data: *mut dllredirect_data;
    let mut index: *mut string_index;
    let mut name_offset: ULONG;

    let mut i: u32 = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.add(i as usize);
        let mut j: u32 = 0;
        while j < (*assembly).num_dlls {
            let dll = (*assembly).dlls.add(j as usize);

            total_len += core::mem::size_of::<string_index>() as u32;
            total_len += core::mem::size_of::<dllredirect_data>() as u32;
            total_len += aligned_string_len(((wcslen((*dll).name) + 1) * core::mem::size_of::<WCHAR>()) as i32) as u32;

            j += 1;
        }
        dll_count += (*assembly).num_dlls;
        i += 1;
    }

    total_len += core::mem::size_of::<strsection_header>() as u32;

    header = RtlAllocateHeap(GetProcessHeap(), 0, total_len as usize) as *mut strsection_header;
    if header.is_null() {
        return STATUS_NO_MEMORY;
    }

    memset(header as *mut _, 0, core::mem::size_of::<strsection_header>());
    (*header).magic = STRSECTION_MAGIC as u32;
    (*header).size = core::mem::size_of::<strsection_header>() as u32;
    (*header).count = dll_count;
    (*header).index_offset = core::mem::size_of::<strsection_header>() as u32;
    index = (header as *mut u8).add((*header).index_offset as usize) as *mut string_index;
    name_offset = (*header).index_offset + (*header).count * core::mem::size_of::<string_index>() as u32;

    let mut i: u32 = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.add(i as usize);

        let mut j: u32 = 0;
        while j < (*assembly).num_dlls {
            let dll = (*assembly).dlls.add(j as usize);
            let mut str: _UNICODE_STRING = core::mem::zeroed();
            let ptrW: *mut WCHAR;

            str.Buffer = (*dll).name;
            str.Length = (wcslen((*dll).name) * core::mem::size_of::<WCHAR>()) as u16;
            str.MaximumLength = str.Length + core::mem::size_of::<WCHAR>() as u16;

            RtlHashUnicodeString(&mut str, 1u8, HASH_STRING_ALGORITHM_X65599, core::ptr::addr_of_mut!((*index).hash));

            (*index).name_offset = name_offset;
            (*index).name_len = str.Length as u32;
            (*index).data_offset = (*index).name_offset + aligned_string_len(str.MaximumLength as i32) as u32;
            (*index).data_len = core::mem::size_of::<dllredirect_data>() as u32;
            (*index).rosterindex = i + 1;

            data = (header as *mut u8).add((*index).data_offset as usize) as *mut dllredirect_data;
            (*data).size = core::mem::size_of::<dllredirect_data>() as u32;
            (*data).unk = 2;
            memset(core::ptr::addr_of_mut!((*data).res) as *mut _, 0, core::mem::size_of_val(&(*data).res));

            ptrW = (header as *mut u8).add((*index).name_offset as usize) as *mut WCHAR;
            memcpy(ptrW as *mut _, (*dll).name as *const _, (*index).name_len as usize);
            *ptrW.add(((*index).name_len as usize) / core::mem::size_of::<WCHAR>()) = 0;

            name_offset += core::mem::size_of::<dllredirect_data>() as u32 + aligned_string_len(str.MaximumLength as i32) as u32;

            index = index.add(1);

            j += 1;
        }
        i += 1;
    }

    *section = header;

    STATUS_SUCCESS
}

unsafe fn find_string_index(section: *const strsection_header, name: *const _UNICODE_STRING) -> *mut string_index {
    let mut index: *mut string_index = core::ptr::null_mut();
    let mut str: _UNICODE_STRING = core::mem::zeroed();
    let mut hash: ULONG = 0;
    let mut i: ULONG;

    RtlHashUnicodeString(name, 1u8, HASH_STRING_ALGORITHM_X65599, &mut hash);
    let mut iter = ((section as *mut BYTE).offset((*section).index_offset as isize)) as *mut string_index;

    i = 0;
    while i < (*section).count {
        if (*iter).hash == hash {
            str.Buffer = ((section as *const BYTE).offset((*iter).name_offset as isize)) as *mut WCHAR;
            str.Length = (*iter).name_len as u16;
            if RtlEqualUnicodeString(&str, name, 1u8) != 0 {
                index = iter;
                break;
            }
        }
        iter = iter.offset(1);
        i += 1;
    }

    index
}

unsafe fn find_guid_index(section: *const guidsection_header, guid: *const GUID) -> *mut guid_index {
    let mut index: *mut guid_index = core::ptr::null_mut();

    let mut iter = ((section as *const u8).add((*section).index_offset as usize)) as *mut guid_index;

    let mut i: ULONG = 0;
    while i < (*section).count {
        if memcmp(
            guid as *const core::ffi::c_void,
            core::ptr::addr_of!((*iter).guid) as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        ) == 0
        {
            index = iter;
            break;
        }
        iter = iter.add(1);
        i += 1;
    }

    index
}

unsafe fn get_dllredirect_data(ctxt: *mut ACTIVATION_CONTEXT, index: *mut string_index) -> *mut dllredirect_data {
    ((*ctxt).dllredirect_section as *mut u8).add((*index).data_offset as usize) as *mut dllredirect_data
}

unsafe fn find_dll_redirection(actctx: *mut ACTIVATION_CONTEXT, name: *const _UNICODE_STRING, data: PACTCTX_SECTION_KEYED_DATA) -> NTSTATUS {
    let dll: *mut dllredirect_data;
    let index: *mut string_index;

    if ((*actctx).sections & DLLREDIRECT_SECTION as u32) == 0 {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if (*actctx).dllredirect_section.is_null() {
        let mut section: *mut strsection_header = core::ptr::null_mut();

        let status = build_dllredirect_section(actctx, &mut section);
        if status != 0 {
            return status;
        }

        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*actctx).dllredirect_section) as *mut *mut core::ffi::c_void,
            section as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        ).is_null() {
            RtlFreeHeap(GetProcessHeap(), 0, section as *mut core::ffi::c_void);
        }
    }

    index = find_string_index((*actctx).dllredirect_section, name);
    if index.is_null() {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if !data.is_null() {
        dll = get_dllredirect_data(actctx, index);

        (*data).ulDataFormatVersion = 1;
        (*data).lpData = dll as *mut core::ffi::c_void;
        (*data).ulLength = (*dll).size;
        (*data).lpSectionGlobalData = core::ptr::null_mut();
        (*data).ulSectionGlobalDataLength = 0;
        (*data).lpSectionBase = (*actctx).dllredirect_section as *mut core::ffi::c_void;
        (*data).ulSectionTotalLength = RtlSizeHeap(GetProcessHeap(), 0, (*actctx).dllredirect_section as *mut core::ffi::c_void) as ULONG;
        (*data).hActCtx = core::ptr::null_mut();

        if (*data).cbSize as usize >= FIELD_OFFSET!(ACTCTX_SECTION_KEYED_DATA, ulAssemblyRosterIndex) + core::mem::size_of::<ULONG>() {
            (*data).ulAssemblyRosterIndex = (*index).rosterindex;
        }
    }

    STATUS_SUCCESS
}

unsafe fn get_wndclass_first_index(actctx: *mut ACTIVATION_CONTEXT) -> *mut string_index {
    ((*actctx).wndclass_section as *mut u8)
        .add((*(*actctx).wndclass_section).index_offset as usize) as *mut string_index
}

unsafe fn get_wndclass_data(ctxt: *mut ACTIVATION_CONTEXT, index: *mut string_index) -> *mut wndclass_redirect_data {
    ((*ctxt).wndclass_section as *mut u8).add((*index).data_offset as usize) as *mut wndclass_redirect_data
}

unsafe fn build_wndclass_section(actctx: *mut ACTIVATION_CONTEXT, section: *mut *mut strsection_header) -> NTSTATUS {
    let mut total_len: u32 = 0;
    let mut class_count: u32 = 0;
    let mut data: *mut wndclass_redirect_data;
    let header: *mut strsection_header;
    let mut index: *mut string_index;
    let mut name_offset: ULONG;

    let mut i: u32 = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.add(i as usize);
        let mut j: u32 = 0;
        while j < (*assembly).num_dlls {
            let dll = (*assembly).dlls.add(j as usize);
            let mut k: u32 = 0;
            while k < (*dll).entities.num {
                let entity = (*dll).entities.base.add(k as usize);
                if (*entity).kind == ACTIVATION_CONTEXT_SECTION_WINDOW_CLASS_REDIRECTION {
                    let class_len = wcslen((*entity).u.class.name) as i32 + 1;
                    let len: i32;

                    total_len += core::mem::size_of::<string_index>() as u32;
                    total_len += core::mem::size_of::<wndclass_redirect_data>() as u32;
                    total_len += aligned_string_len(class_len * core::mem::size_of::<WCHAR>() as i32) as u32;
                    if (*entity).u.class.versioned != 0 {
                        len = get_assembly_version(assembly, core::ptr::null_mut()) + class_len + 1;
                    } else {
                        len = class_len;
                    }
                    let len = len + wcslen((*dll).name) as i32 + 1;
                    total_len += aligned_string_len(len * core::mem::size_of::<WCHAR>() as i32) as u32;

                    class_count += 1;
                }
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }

    total_len += core::mem::size_of::<strsection_header>() as u32;

    header = RtlAllocateHeap(GetProcessHeap(), 0, total_len as usize) as *mut strsection_header;
    if header.is_null() {
        return STATUS_NO_MEMORY;
    }

    memset(header as *mut _, 0, core::mem::size_of::<strsection_header>());
    (*header).magic = STRSECTION_MAGIC;
    (*header).size = core::mem::size_of::<strsection_header>() as ULONG;
    (*header).count = class_count;
    (*header).index_offset = core::mem::size_of::<strsection_header>() as ULONG;
    index = ((header as *mut u8).add((*header).index_offset as usize)) as *mut string_index;
    name_offset = (*header).index_offset + (*header).count * core::mem::size_of::<string_index>() as u32;

    let mut i: u32 = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.add(i as usize);
        let mut j: u32 = 0;
        while j < (*assembly).num_dlls {
            let dll = (*assembly).dlls.add(j as usize);
            let mut k: u32 = 0;
            while k < (*dll).entities.num {
                let entity = (*dll).entities.base.add(k as usize);
                if (*entity).kind == ACTIVATION_CONTEXT_SECTION_WINDOW_CLASS_REDIRECTION {
                    static exclW: [WCHAR; 2] = ['!' as WCHAR, 0];
                    let versioned_len: ULONG;
                    let module_len: ULONG;
                    let mut str: _UNICODE_STRING = core::mem::zeroed();
                    let mut ptrW: *mut WCHAR;

                    str.Buffer = (*entity).u.class.name;
                    str.Length = (wcslen((*entity).u.class.name) * core::mem::size_of::<WCHAR>()) as u16;
                    str.MaximumLength = str.Length + core::mem::size_of::<WCHAR>() as u16;
                    RtlHashUnicodeString(&str, 1u8, HASH_STRING_ALGORITHM_X65599, core::ptr::addr_of_mut!((*index).hash));

                    if (*entity).u.class.versioned != 0 {
                        versioned_len = (get_assembly_version(assembly, core::ptr::null_mut()) as u32 + 1) * core::mem::size_of::<WCHAR>() as u32 + str.Length as u32;
                    } else {
                        versioned_len = str.Length as u32;
                    }
                    module_len = (wcslen((*dll).name) * core::mem::size_of::<WCHAR>()) as u32;

                    (*index).name_offset = name_offset;
                    (*index).name_len = str.Length as u32;
                    (*index).data_offset = (*index).name_offset + aligned_string_len(str.MaximumLength as i32) as u32;
                    (*index).data_len = core::mem::size_of::<wndclass_redirect_data>() as u32 + versioned_len + module_len + 2 * core::mem::size_of::<WCHAR>() as u32;
                    (*index).rosterindex = i + 1;

                    data = ((header as *mut u8).add((*index).data_offset as usize)) as *mut wndclass_redirect_data;
                    (*data).size = core::mem::size_of::<wndclass_redirect_data>() as ULONG;
                    (*data).res = 0;
                    (*data).name_len = versioned_len;
                    (*data).name_offset = core::mem::size_of::<wndclass_redirect_data>() as ULONG;
                    (*data).module_len = module_len;
                    (*data).module_offset = (*index).data_offset + (*data).name_offset + (*data).name_len + core::mem::size_of::<WCHAR>() as u32;

                    ptrW = ((header as *mut u8).add((*index).name_offset as usize)) as *mut WCHAR;
                    memcpy(ptrW as *mut _, (*entity).u.class.name as *const _, (*index).name_len as usize);
                    *ptrW.add(((*index).name_len / core::mem::size_of::<WCHAR>() as u32) as usize) = 0;

                    ptrW = ((header as *mut u8).add((*data).module_offset as usize)) as *mut WCHAR;
                    memcpy(ptrW as *mut _, (*dll).name as *const _, (*data).module_len as usize);
                    *ptrW.add(((*data).module_len / core::mem::size_of::<WCHAR>() as u32) as usize) = 0;

                    ptrW = ((data as *mut u8).add((*data).name_offset as usize)) as *mut WCHAR;
                    if (*entity).u.class.versioned != 0 {
                        get_assembly_version(assembly, ptrW);
                        wcscat(ptrW, exclW.as_ptr());
                        wcscat(ptrW, (*entity).u.class.name);
                    } else {
                        memcpy(ptrW as *mut _, (*entity).u.class.name as *const _, (*index).name_len as usize);
                        *ptrW.add(((*index).name_len / core::mem::size_of::<WCHAR>() as u32) as usize) = 0;
                    }

                    name_offset += core::mem::size_of::<wndclass_redirect_data>() as u32;
                    name_offset += aligned_string_len(str.MaximumLength as i32) as u32 + aligned_string_len((versioned_len + module_len + 2 * core::mem::size_of::<WCHAR>() as u32) as i32) as u32;

                    index = index.add(1);
                }
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }

    *section = header;

    STATUS_SUCCESS
}

unsafe fn find_window_class(actctx: *mut ACTIVATION_CONTEXT, name: *const _UNICODE_STRING, data: PACTCTX_SECTION_KEYED_DATA) -> NTSTATUS {
    let mut index: *mut string_index = core::ptr::null_mut();
    let mut str: _UNICODE_STRING = core::mem::zeroed();
    let mut hash: ULONG;

    if ((*actctx).sections & (WINDOWCLASS_SECTION as u32)) == 0 {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if (*actctx).wndclass_section.is_null() {
        let mut section: *mut strsection_header = core::ptr::null_mut();

        let status = build_wndclass_section(actctx, &mut section);
        if status != 0 {
            return status;
        }

        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*actctx).wndclass_section) as *mut *mut core::ffi::c_void,
            section as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        ).is_null() {
            RtlFreeHeap(GetProcessHeap(), 0, section as *mut core::ffi::c_void);
        }
    }

    hash = 0;
    RtlHashUnicodeString(name, 1u8, HASH_STRING_ALGORITHM_X65599, &mut hash);
    let mut iter = get_wndclass_first_index(actctx);

    let mut i: i32 = 0;
    while (i as ULONG) < (*(*actctx).wndclass_section).count {
        if (*iter).hash == hash {
            str.Buffer = ((*actctx).wndclass_section as *mut u8).add((*iter).name_offset as usize) as *mut u16;
            str.Length = (*iter).name_len as u16;
            if RtlEqualUnicodeString(&str, name, 1u8) != 0 {
                index = iter;
                break;
            }
        }
        iter = iter.add(1);
        i += 1;
    }

    if index.is_null() {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if !data.is_null() {
        let class = get_wndclass_data(actctx, index);

        (*data).ulDataFormatVersion = 1;
        (*data).lpData = class as *mut core::ffi::c_void;
        (*data).ulLength = (*class).size + (*class).name_len + (*class).module_len + 2 * core::mem::size_of::<u16>() as u32;
        (*data).lpSectionGlobalData = core::ptr::null_mut();
        (*data).ulSectionGlobalDataLength = 0;
        (*data).lpSectionBase = (*actctx).wndclass_section as *mut core::ffi::c_void;
        (*data).ulSectionTotalLength = RtlSizeHeap(GetProcessHeap(), 0, (*actctx).wndclass_section as *mut core::ffi::c_void) as u32;
        (*data).hActCtx = core::ptr::null_mut();

        if (*data).cbSize as usize >= FIELD_OFFSET!(ACTCTX_SECTION_KEYED_DATA, ulAssemblyRosterIndex) + core::mem::size_of::<ULONG>() {
            (*data).ulAssemblyRosterIndex = (*index).rosterindex;
        }
    }

    STATUS_SUCCESS
}

unsafe fn build_tlib_section(actctx: *mut ACTIVATION_CONTEXT, section: *mut *mut guidsection_header) -> NTSTATUS {
    let mut total_len: u32 = 0;
    let mut tlib_count: u32 = 0;
    let mut names_len: u32 = 0;
    let header: *mut guidsection_header;
    let mut module_offset: ULONG;
    let mut data_offset: ULONG;
    let mut data: *mut tlibredirect_data;
    let mut index: *mut guid_index;

    /* compute section length */
    let mut i: u32 = 0;
    while i < (*actctx).num_assemblies {
        let assembly = &mut *(*actctx).assemblies.add(i as usize);
        let mut j: u32 = 0;
        while j < assembly.num_dlls {
            let dll = &mut *assembly.dlls.add(j as usize);
            let mut k: u32 = 0;
            while k < dll.entities.num {
                let entity = &mut *dll.entities.base.add(k as usize);
                if entity.kind == ACTIVATION_CONTEXT_SECTION_COM_TYPE_LIBRARY_REDIRECTION {
                    total_len += core::mem::size_of::<guid_index>() as u32;
                    total_len += core::mem::size_of::<tlibredirect_data>() as u32;
                    if *entity.u.typelib.helpdir != 0 {
                        total_len += aligned_string_len(((wcslen(entity.u.typelib.helpdir) + 1) * core::mem::size_of::<WCHAR>()) as i32) as u32;
                    }
                    names_len += ((wcslen(dll.name) + 1) * core::mem::size_of::<WCHAR>()) as u32;
                    tlib_count += 1;
                }
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }

    total_len += aligned_string_len(names_len as i32) as u32;
    total_len += core::mem::size_of::<guidsection_header>() as u32;

    header = RtlAllocateHeap(GetProcessHeap(), 0, total_len as usize) as *mut guidsection_header;
    if header.is_null() {
        return STATUS_NO_MEMORY;
    }

    memset(header as *mut _, 0, core::mem::size_of::<guidsection_header>());
    (*header).magic = GUIDSECTION_MAGIC;
    (*header).size = core::mem::size_of::<guidsection_header>() as ULONG;
    (*header).count = tlib_count;
    (*header).index_offset = core::mem::size_of::<guidsection_header>() as ULONG + aligned_string_len(names_len as i32) as ULONG;
    index = (header as *mut u8).add((*header).index_offset as usize) as *mut guid_index;
    module_offset = core::mem::size_of::<guidsection_header>() as ULONG;
    data_offset = (*header).index_offset + tlib_count * core::mem::size_of::<guid_index>() as u32;

    let mut i: u32 = 0;
    while i < (*actctx).num_assemblies {
        let assembly = &mut *(*actctx).assemblies.add(i as usize);
        let mut j: u32 = 0;
        while j < assembly.num_dlls {
            let dll = &mut *assembly.dlls.add(j as usize);
            let mut k: u32 = 0;
            while k < dll.entities.num {
                let entity = &mut *dll.entities.base.add(k as usize);
                if entity.kind == ACTIVATION_CONTEXT_SECTION_COM_TYPE_LIBRARY_REDIRECTION {
                    let module_len: ULONG;
                    let help_len: ULONG;
                    let mut str: _UNICODE_STRING = core::mem::zeroed();
                    let mut ptrW: *mut WCHAR;
                    let Status: NTSTATUS;

                    if *entity.u.typelib.helpdir != 0 {
                        help_len = (wcslen(entity.u.typelib.helpdir) * core::mem::size_of::<WCHAR>()) as ULONG;
                    } else {
                        help_len = 0;
                    }

                    module_len = (wcslen(dll.name) * core::mem::size_of::<WCHAR>()) as ULONG;

                    RtlInitUnicodeString(&mut str, entity.u.typelib.tlbid);
                    Status = RtlGUIDFromString(&mut str, &mut (*index).guid);
                    if !NT_SUCCESS(Status) {
                        RtlFreeHeap(GetProcessHeap(), 0, header as *mut _);
                        return Status;
                    }
                    (*index).data_offset = data_offset;
                    (*index).data_len = core::mem::size_of::<tlibredirect_data>() as u32 + aligned_string_len(help_len as i32) as u32;
                    (*index).rosterindex = i + 1;

                    data = (header as *mut u8).add((*index).data_offset as usize) as *mut tlibredirect_data;
                    (*data).size = core::mem::size_of::<tlibredirect_data>() as ULONG;
                    (*data).res = 0;
                    (*data).name_len = module_len;
                    (*data).name_offset = module_offset;
                    (*data).langid = 0;
                    (*data).flags = entity.u.typelib.flags;
                    (*data).help_len = help_len;
                    (*data).help_offset = core::mem::size_of::<tlibredirect_data>() as ULONG;
                    (*data).major_version = entity.u.typelib.major;
                    (*data).minor_version = entity.u.typelib.minor;

                    ptrW = (header as *mut u8).add((*data).name_offset as usize) as *mut WCHAR;
                    memcpy(ptrW as *mut _, dll.name as *const _, (*data).name_len as usize);
                    *ptrW.add(((*data).name_len as usize) / core::mem::size_of::<WCHAR>()) = 0;

                    if (*data).help_len != 0 {
                        ptrW = (data as *mut u8).add((*data).help_offset as usize) as *mut WCHAR;
                        memcpy(ptrW as *mut _, entity.u.typelib.helpdir as *const _, (*data).help_len as usize);
                        *ptrW.add(((*data).help_len as usize) / core::mem::size_of::<WCHAR>()) = 0;
                    }

                    data_offset += core::mem::size_of::<tlibredirect_data>() as u32;
                    if help_len != 0 {
                        data_offset += aligned_string_len((help_len + core::mem::size_of::<WCHAR>() as ULONG) as i32) as u32;
                    }

                    module_offset += module_len + core::mem::size_of::<WCHAR>() as ULONG;

                    index = index.add(1);
                }
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }

    *section = header;

    STATUS_SUCCESS
}

unsafe fn get_tlib_data(actctx: *mut ACTIVATION_CONTEXT, index: *mut guid_index) -> *mut tlibredirect_data {
    ((*actctx).tlib_section as *mut u8).add((*index).data_offset as usize) as *mut tlibredirect_data
}

unsafe fn find_tlib_redirection(actctx: *mut ACTIVATION_CONTEXT, guid: *const GUID, data: *mut ACTCTX_SECTION_KEYED_DATA) -> NTSTATUS {
    let index: *mut guid_index;
    let tlib: *mut tlibredirect_data;

    if ((*actctx).sections & TLIBREDIRECT_SECTION) == 0 {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if (*actctx).tlib_section.is_null() {
        let mut section: *mut guidsection_header = core::ptr::null_mut();

        let status = build_tlib_section(actctx, &mut section);
        if status != 0 {
            return status;
        }

        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*actctx).tlib_section) as *mut *mut core::ffi::c_void,
            section as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        )
        .is_null()
        {
            RtlFreeHeap(GetProcessHeap(), 0, section as *mut core::ffi::c_void);
        }
    }

    index = find_guid_index((*actctx).tlib_section, guid);
    if index.is_null() {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    tlib = get_tlib_data(actctx, index);

    (*data).ulDataFormatVersion = 1;
    (*data).lpData = tlib as *mut core::ffi::c_void;
    (*data).ulLength = (*tlib).size + (*tlib).help_len + core::mem::size_of::<WCHAR>() as ULONG;
    (*data).lpSectionGlobalData =
        ((*actctx).tlib_section as *mut u8).add((*(*actctx).tlib_section).names_offset as usize) as *mut core::ffi::c_void;
    (*data).ulSectionGlobalDataLength = (*(*actctx).tlib_section).names_len;
    (*data).lpSectionBase = (*actctx).tlib_section as *mut core::ffi::c_void;
    (*data).ulSectionTotalLength =
        RtlSizeHeap(GetProcessHeap(), 0, (*actctx).tlib_section as *mut core::ffi::c_void) as ULONG;
    (*data).hActCtx = core::ptr::null_mut();

    if (*data).cbSize as usize
        >= FIELD_OFFSET!(ACTCTX_SECTION_KEYED_DATA, ulAssemblyRosterIndex) + core::mem::size_of::<ULONG>()
    {
        (*data).ulAssemblyRosterIndex = (*index).rosterindex;
    }

    STATUS_SUCCESS
}

unsafe fn generate_uuid(seed: *mut ULONG, guid: *mut GUID) {
    let mut ptr = guid as *mut ULONG;
    let count = core::mem::size_of::<GUID>() / core::mem::size_of::<ULONG>();

    for _ in 0..count {
        *ptr = RtlUniform(seed);
        ptr = ptr.add(1);
    }

    (*guid).Data3 &= 0x0fff;
    (*guid).Data3 |= 4 << 12;
    (*guid).Data4[0] &= 0x3f;
    (*guid).Data4[0] |= 0x80;
}
unsafe fn get_comserver_datalen(entities: *const entity_array, dll: *const dll_redirect,
    count: *mut core::ffi::c_uint, len: *mut core::ffi::c_uint, module_len: *mut core::ffi::c_uint) {
    let mut i: core::ffi::c_uint;

    i = 0;
    while i < (*entities).num {
        let entity: *mut entity = (*entities).base.offset(i as isize);
        if (*entity).kind == ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION {
            /* each entry needs two index entries, extra one goes for alias GUID */
            *len += 2 * core::mem::size_of::<guid_index>() as core::ffi::c_uint;
            /* To save some memory we don't allocated two data structures,
               instead alias index and normal index point to the same data structure. */
            *len += core::mem::size_of::<comclassredirect_data>() as core::ffi::c_uint;

            /* for clrClass store some more */
            if !(*entity).u.comclass.name.is_null() {
                let mut str_len: core::ffi::c_uint;

                /* all string data is stored together in aligned block */
                str_len = (wcslen((*entity).u.comclass.name) + 1) as core::ffi::c_uint;
                if !(*entity).u.comclass.progid.is_null() {
                    str_len += (wcslen((*entity).u.comclass.progid) + 1) as core::ffi::c_uint;
                }
                if !(*entity).u.comclass.version.is_null() {
                    str_len += (wcslen((*entity).u.comclass.version) + 1) as core::ffi::c_uint;
                }

                *len += core::mem::size_of::<clrclass_data>() as core::ffi::c_uint;
                *len += aligned_string_len((str_len * core::mem::size_of::<WCHAR>() as core::ffi::c_uint) as i32) as core::ffi::c_uint;

                /* module name is forced to mscoree.dll, and stored two times with different case */
                *module_len += (core::mem::size_of_val(&mscoreeW) + core::mem::size_of_val(&mscoree2W)) as core::ffi::c_uint;
            } else {
                /* progid string is stored separately */
                if !(*entity).u.comclass.progid.is_null() {
                    *len += aligned_string_len(((wcslen((*entity).u.comclass.progid) + 1) * core::mem::size_of::<WCHAR>()) as i32) as core::ffi::c_uint;
                }

                *module_len += ((wcslen((*dll).name) + 1) * core::mem::size_of::<WCHAR>()) as core::ffi::c_uint;
            }

            *count += 1;
        }
        i += 1;
    }
}

unsafe fn add_comserver_record(
    section: *const guidsection_header,
    entities: *const entity_array,
    dll: *const dll_redirect,
    index: *mut *mut guid_index,
    data_offset: *mut ULONG,
    module_offset: *mut ULONG,
    seed: *mut ULONG,
    rosterindex: ULONG,
) -> NTSTATUS {
    let mut i: u32 = 0;
    let mut Status: NTSTATUS;

    while i < (*entities).num {
        let entity = (*entities).base.add(i as usize);
        if (*entity).kind == ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION {
            let module_len: ULONG;
            let progid_len: ULONG;
            let mut str_len: ULONG;
            let mut miscmask: ULONG;
            let data: *mut comclassredirect_data;
            let alias_index: *mut guid_index;
            let clrdata: *mut clrclass_data;
            let mut str: _UNICODE_STRING = core::mem::zeroed();
            let mut ptrW: *mut WCHAR;

            if !(*entity).u.comclass.progid.is_null() {
                progid_len = (wcslen((*entity).u.comclass.progid) * core::mem::size_of::<WCHAR>()) as ULONG;
            } else {
                progid_len = 0;
            }

            module_len = if !dll.is_null() {
                (wcslen((*dll).name) * core::mem::size_of::<WCHAR>()) as ULONG
            } else {
                (wcslen(mscoreeW.as_ptr()) * core::mem::size_of::<WCHAR>()) as ULONG
            };

            RtlInitUnicodeString(&mut str, (*entity).u.comclass.clsid);
            Status = RtlGUIDFromString(&mut str, &mut (**index).guid);
            if !NT_SUCCESS(Status) {
                return Status;
            }

            (**index).data_offset = *data_offset;
            (**index).data_len = core::mem::size_of::<comclassredirect_data>() as ULONG;
            (**index).rosterindex = rosterindex;

            alias_index = (*index).add(((*section).count / 2) as usize);
            generate_uuid(seed, &mut (*alias_index).guid);
            (*alias_index).data_offset = (**index).data_offset;
            (*alias_index).data_len = 0;
            (*alias_index).rosterindex = (**index).rosterindex;

            data = (section as *mut u8).add((**index).data_offset as usize) as *mut comclassredirect_data;
            (*data).size = core::mem::size_of::<comclassredirect_data>() as ULONG;
            (*data).model = (*entity).u.comclass.model;
            (*data).clsid = (**index).guid;
            (*data).alias = (*alias_index).guid;
            (*data).clsid2 = (*data).clsid;
            if !(*entity).u.comclass.tlbid.is_null() {
                RtlInitUnicodeString(&mut str, (*entity).u.comclass.tlbid);
                Status = RtlGUIDFromString(&mut str, &mut (*data).tlbid);
                if !NT_SUCCESS(Status) {
                    return Status;
                }
            } else {
                memset(
                    core::ptr::addr_of_mut!((*data).tlbid) as *mut _,
                    0,
                    core::mem::size_of_val(&(*data).tlbid),
                );
            }
            (*data).name_len = module_len;
            (*data).name_offset = *module_offset;
            (*data).progid_len = progid_len;
            (*data).progid_offset = if (*data).progid_len != 0 { (*data).size } else { 0 };
            (*data).clrdata_len = 0;
            (*data).clrdata_offset = if !(*entity).u.comclass.name.is_null() {
                core::mem::size_of::<comclassredirect_data>() as ULONG
            } else {
                0
            };
            (*data).miscstatus = (*entity).u.comclass.miscstatus;
            (*data).miscstatuscontent = (*entity).u.comclass.miscstatuscontent;
            (*data).miscstatusthumbnail = (*entity).u.comclass.miscstatusthumbnail;
            (*data).miscstatusicon = (*entity).u.comclass.miscstatusicon;
            (*data).miscstatusdocprint = (*entity).u.comclass.miscstatusdocprint;

            miscmask = 0;
            if (*data).miscstatus != 0 {
                miscmask |= MiscStatus as ULONG;
            }
            if (*data).miscstatuscontent != 0 {
                miscmask |= MiscStatusContent as ULONG;
            }
            if (*data).miscstatusthumbnail != 0 {
                miscmask |= MiscStatusThumbnail as ULONG;
            }
            if (*data).miscstatusicon != 0 {
                miscmask |= MiscStatusIcon as ULONG;
            }
            if (*data).miscstatusdocprint != 0 {
                miscmask |= MiscStatusDocPrint as ULONG;
            }
            (*data).flags = miscmask << 8;

            if (*data).clrdata_offset != 0 {
                clrdata = (data as *mut u8).add((*data).clrdata_offset as usize) as *mut clrclass_data;

                (*clrdata).size = core::mem::size_of::<clrclass_data>() as ULONG;
                (*clrdata).res[0] = 0;
                (*clrdata).res[1] = 2;
                (*clrdata).module_len = (wcslen(mscoreeW.as_ptr()) * core::mem::size_of::<WCHAR>()) as ULONG;
                (*clrdata).module_offset = *module_offset + (*data).name_len + core::mem::size_of::<WCHAR>() as ULONG;
                (*clrdata).name_len = (wcslen((*entity).u.comclass.name) * core::mem::size_of::<WCHAR>()) as ULONG;
                (*clrdata).name_offset = (*clrdata).size;
                (*clrdata).version_len = if !(*entity).u.comclass.version.is_null() {
                    (wcslen((*entity).u.comclass.version) * core::mem::size_of::<WCHAR>()) as ULONG
                } else {
                    0
                };
                (*clrdata).version_offset = if (*clrdata).version_len != 0 {
                    (*clrdata).name_offset + (*clrdata).name_len + core::mem::size_of::<WCHAR>() as ULONG
                } else {
                    0
                };
                (*clrdata).res2[0] = 0;
                (*clrdata).res2[1] = 0;

                (*data).clrdata_len = (*clrdata).size + (*clrdata).name_len + core::mem::size_of::<WCHAR>() as ULONG;

                ptrW = (section as *mut u8).add((*clrdata).module_offset as usize) as *mut WCHAR;
                memcpy(ptrW as *mut _, mscoree2W.as_ptr() as *const _, (*clrdata).module_len as usize);
                *ptrW.add(((*clrdata).module_len / core::mem::size_of::<WCHAR>() as ULONG) as usize) = 0;

                ptrW = (section as *mut u8).add((*data).name_offset as usize) as *mut WCHAR;
                memcpy(ptrW as *mut _, mscoreeW.as_ptr() as *const _, (*data).name_len as usize);
                *ptrW.add(((*data).name_len / core::mem::size_of::<WCHAR>() as ULONG) as usize) = 0;

                ptrW = (clrdata as *mut u8).add((*clrdata).name_offset as usize) as *mut WCHAR;
                memcpy(ptrW as *mut _, (*entity).u.comclass.name as *const _, (*clrdata).name_len as usize);
                *ptrW.add(((*clrdata).name_len / core::mem::size_of::<WCHAR>() as ULONG) as usize) = 0;

                if (*clrdata).version_len != 0 {
                    (*data).clrdata_len += (*clrdata).version_len + core::mem::size_of::<WCHAR>() as ULONG;

                    ptrW = (clrdata as *mut u8).add((*clrdata).version_offset as usize) as *mut WCHAR;
                    memcpy(ptrW as *mut _, (*entity).u.comclass.version as *const _, (*clrdata).version_len as usize);
                    *ptrW.add(((*clrdata).version_len / core::mem::size_of::<WCHAR>() as ULONG) as usize) = 0;
                }

                if (*data).progid_len != 0 {
                    (*data).progid_offset += (*data).clrdata_len;
                }
                (**index).data_len += core::mem::size_of::<clrclass_data>() as ULONG;
            } else {
                clrdata = core::ptr::null_mut();

                ptrW = (section as *mut u8).add((*data).name_offset as usize) as *mut WCHAR;
                memcpy(ptrW as *mut _, (*dll).name as *const _, (*data).name_len as usize);
                *ptrW.add(((*data).name_len / core::mem::size_of::<WCHAR>() as ULONG) as usize) = 0;
            }

            if (*data).progid_len != 0 {
                ptrW = (data as *mut u8).add((*data).progid_offset as usize) as *mut WCHAR;
                memcpy(ptrW as *mut _, (*entity).u.comclass.progid as *const _, (*data).progid_len as usize);
                *ptrW.add(((*data).progid_len / core::mem::size_of::<WCHAR>() as ULONG) as usize) = 0;
            }

            str_len = 0;
            if !clrdata.is_null() {
                str_len += (*clrdata).name_len + core::mem::size_of::<WCHAR>() as ULONG;
                if (*clrdata).version_len != 0 {
                    str_len += (*clrdata).version_len + core::mem::size_of::<WCHAR>() as ULONG;
                }
            }
            if progid_len != 0 {
                str_len += progid_len + core::mem::size_of::<WCHAR>() as ULONG;
            }

            (**index).data_len += aligned_string_len(str_len as i32) as ULONG;
            (*alias_index).data_len = (**index).data_len;

            *data_offset += core::mem::size_of::<comclassredirect_data>() as ULONG + aligned_string_len(str_len as i32) as ULONG;
            *module_offset += module_len + core::mem::size_of::<WCHAR>() as ULONG;

            if !clrdata.is_null() {
                *data_offset += core::mem::size_of::<clrclass_data>() as ULONG;
                *module_offset += (*clrdata).module_len + core::mem::size_of::<WCHAR>() as ULONG;
            }
            *index = (*index).add(1);
        }
        i += 1;
    }

    STATUS_SUCCESS
}

unsafe fn build_comserver_section(
    actctx: *mut ACTIVATION_CONTEXT,
    section: *mut *mut guidsection_header,
) -> NTSTATUS {
    let mut i: core::ffi::c_uint;
    let mut j: core::ffi::c_uint;
    let mut total_len: core::ffi::c_uint = 0;
    let mut class_count: core::ffi::c_uint = 0;
    let mut names_len: core::ffi::c_uint = 0;
    let header: *mut guidsection_header;
    let mut module_offset: ULONG;
    let mut data_offset: ULONG;
    let mut index: *mut guid_index;
    let mut seed: ULONG;
    let mut Status: NTSTATUS;

    i = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.offset(i as isize);
        get_comserver_datalen(
            core::ptr::addr_of_mut!((*assembly).entities),
            core::ptr::null_mut(),
            &mut class_count,
            &mut total_len,
            &mut names_len,
        );
        j = 0;
        while j < (*assembly).num_dlls {
            let dll = (*assembly).dlls.offset(j as isize);
            get_comserver_datalen(
                core::ptr::addr_of_mut!((*dll).entities),
                dll,
                &mut class_count,
                &mut total_len,
                &mut names_len,
            );
            j += 1;
        }
        i += 1;
    }

    total_len += aligned_string_len(names_len as i32) as core::ffi::c_uint;
    total_len += core::mem::size_of::<guidsection_header>() as core::ffi::c_uint;

    header = RtlAllocateHeap(GetProcessHeap(), 0, total_len as usize) as *mut guidsection_header;
    if header.is_null() {
        return STATUS_NO_MEMORY;
    }

    core::ptr::write_bytes(header as *mut u8, 0, core::mem::size_of::<guidsection_header>());
    (*header).magic = GUIDSECTION_MAGIC;
    (*header).size = core::mem::size_of::<guidsection_header>() as ULONG;
    (*header).count = 2 * class_count;
    (*header).index_offset = core::mem::size_of::<guidsection_header>() as ULONG
        + aligned_string_len(names_len as i32) as ULONG;
    index = (header as *mut u8).add((*header).index_offset as usize) as *mut guid_index;
    module_offset = core::mem::size_of::<guidsection_header>() as ULONG;
    data_offset = (*header).index_offset
        + 2 * class_count * core::mem::size_of::<guid_index>() as ULONG;

    seed = RtlGetTickCount();
    i = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.offset(i as isize);
        Status = add_comserver_record(
            header,
            core::ptr::addr_of_mut!((*assembly).entities),
            core::ptr::null_mut(),
            &mut index,
            &mut data_offset,
            &mut module_offset,
            &mut seed,
            i + 1,
        );
        if !NT_SUCCESS(Status) {
            RtlFreeHeap(GetProcessHeap(), 0, header as *mut core::ffi::c_void);
            return Status;
        }
        j = 0;
        while j < (*assembly).num_dlls {
            let dll = (*assembly).dlls.offset(j as isize);
            Status = add_comserver_record(
                header,
                core::ptr::addr_of_mut!((*dll).entities),
                dll,
                &mut index,
                &mut data_offset,
                &mut module_offset,
                &mut seed,
                i + 1,
            );
            if !NT_SUCCESS(Status) {
                RtlFreeHeap(GetProcessHeap(), 0, header as *mut core::ffi::c_void);
                return Status;
            }
            j += 1;
        }
        i += 1;
    }

    *section = header;

    STATUS_SUCCESS
}

unsafe fn get_comclass_data(actctx: *mut ACTIVATION_CONTEXT, index: *mut guid_index) -> *mut comclassredirect_data {
    ((*actctx).comserver_section as *mut u8).add((*index).data_offset as usize) as *mut comclassredirect_data
}

unsafe fn find_comserver_redirection(
    actctx: *mut ACTIVATION_CONTEXT,
    guid: *const GUID,
    data: *mut ACTCTX_SECTION_KEYED_DATA,
) -> NTSTATUS {
    let comclass: *mut comclassredirect_data;
    let index: *mut guid_index;

    if ((*actctx).sections & SERVERREDIRECT_SECTION as u32) == 0 {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if (*actctx).comserver_section.is_null() {
        let mut section: *mut guidsection_header = core::ptr::null_mut();

        let status = build_comserver_section(actctx, &mut section);
        if status != 0 {
            return status;
        }

        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*actctx).comserver_section) as *mut *mut core::ffi::c_void,
            section as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        )
        .is_null()
        {
            RtlFreeHeap(GetProcessHeap(), 0, section as *mut core::ffi::c_void);
        }
    }

    index = find_guid_index((*actctx).comserver_section, guid);
    if index.is_null() {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    comclass = get_comclass_data(actctx, index);

    (*data).ulDataFormatVersion = 1;
    (*data).lpData = comclass as *mut core::ffi::c_void;
    (*data).ulLength = (*comclass).size + (*comclass).clrdata_len;
    if (*comclass).progid_len != 0 {
        (*data).ulLength += (*comclass).progid_len + core::mem::size_of::<WCHAR>() as ULONG;
    }
    (*data).lpSectionGlobalData = ((*actctx).comserver_section as *mut u8)
        .add((*(*actctx).comserver_section).names_offset as usize)
        as *mut core::ffi::c_void;
    (*data).ulSectionGlobalDataLength = (*(*actctx).comserver_section).names_len;
    (*data).lpSectionBase = (*actctx).comserver_section as *mut core::ffi::c_void;
    (*data).ulSectionTotalLength =
        RtlSizeHeap(GetProcessHeap(), 0, (*actctx).comserver_section as *mut core::ffi::c_void) as ULONG;
    (*data).hActCtx = core::ptr::null_mut();

    if (*data).cbSize as usize
        >= FIELD_OFFSET!(ACTCTX_SECTION_KEYED_DATA, ulAssemblyRosterIndex)
            + core::mem::size_of::<ULONG>()
    {
        (*data).ulAssemblyRosterIndex = (*index).rosterindex;
    }

    STATUS_SUCCESS
}

unsafe fn get_ifaceps_datalen(entities: *const entity_array, count: *mut u32, len: *mut u32) {
    let mut i: u32 = 0;
    while i < (*entities).num {
        let entity = (*entities).base.add(i as usize);
        if (*entity).kind == ACTIVATION_CONTEXT_SECTION_COM_INTERFACE_REDIRECTION {
            *len += (core::mem::size_of::<guid_index>() + core::mem::size_of::<ifacepsredirect_data>()) as u32;
            if !(*entity).u.ifaceps.name.is_null() {
                *len += aligned_string_len(((wcslen((*entity).u.ifaceps.name) + 1) * core::mem::size_of::<WCHAR>()) as i32) as u32;
            }
            *count += 1;
        }
        i += 1;
    }
}

unsafe fn add_ifaceps_record(
    section: *mut guidsection_header,
    entities: *mut entity_array,
    index: *mut *mut guid_index,
    data_offset: *mut ULONG,
    rosterindex: ULONG,
) -> NTSTATUS {
    let mut i: u32 = 0;
    while i < (*entities).num {
        let entity = (*entities).base.add(i as usize);
        if (*entity).kind == ACTIVATION_CONTEXT_SECTION_COM_INTERFACE_REDIRECTION {
            let data = (section as *mut u8).add(*data_offset as usize) as *mut ifacepsredirect_data;
            let mut str: _UNICODE_STRING = core::mem::zeroed();
            let name_len: ULONG;
            let mut Status: NTSTATUS;

            if !(*entity).u.ifaceps.name.is_null() {
                name_len = (wcslen((*entity).u.ifaceps.name) * core::mem::size_of::<u16>()) as ULONG;
            } else {
                name_len = 0;
            }

            RtlInitUnicodeString(&mut str, (*entity).u.ifaceps.iid);
            Status = RtlGUIDFromString(&mut str, &mut (**index).guid);
            if !NT_SUCCESS(Status) {
                return Status;
            }
            (**index).data_offset = *data_offset;
            (**index).data_len = if (core::mem::size_of::<ifacepsredirect_data>() as ULONG + name_len) != 0 {
                aligned_string_len((name_len + core::mem::size_of::<u16>() as ULONG) as i32) as ULONG
            } else {
                0
            };
            (**index).rosterindex = rosterindex;

            (*data).size = core::mem::size_of::<ifacepsredirect_data>() as ULONG;
            (*data).mask = (*entity).u.ifaceps.mask;

            if !(*entity).u.ifaceps.ps32.is_null() {
                RtlInitUnicodeString(&mut str, (*entity).u.ifaceps.ps32);
                Status = RtlGUIDFromString(&mut str, &mut (*data).iid);
                if !NT_SUCCESS(Status) {
                    return Status;
                }
            } else {
                (*data).iid = (**index).guid;
            }

            (*data).nummethods = (*entity).u.ifaceps.nummethods;

            if !(*entity).u.ifaceps.tlib.is_null() {
                RtlInitUnicodeString(&mut str, (*entity).u.ifaceps.tlib);
                Status = RtlGUIDFromString(&mut str, &mut (*data).tlbid);
                if !NT_SUCCESS(Status) {
                    return Status;
                }
            } else {
                core::ptr::write_bytes(core::ptr::addr_of_mut!((*data).tlbid), 0, 1);
            }

            if !(*entity).u.ifaceps.base.is_null() {
                RtlInitUnicodeString(&mut str, (*entity).u.ifaceps.base);
                Status = RtlGUIDFromString(&mut str, &mut (*data).base);
                if !NT_SUCCESS(Status) {
                    return Status;
                }
            } else {
                core::ptr::write_bytes(core::ptr::addr_of_mut!((*data).base), 0, 1);
            }

            (*data).name_len = name_len;
            (*data).name_offset = if (*data).name_len != 0 {
                core::mem::size_of::<ifacepsredirect_data>() as ULONG
            } else {
                0
            };

            if (*data).name_len != 0 {
                let ptrW = (data as *mut u8).add((*data).name_offset as usize) as *mut u16;
                core::ptr::copy_nonoverlapping(
                    (*entity).u.ifaceps.name as *const u8,
                    ptrW as *mut u8,
                    (*data).name_len as usize,
                );
                *ptrW.add(((*data).name_len as usize) / core::mem::size_of::<u16>()) = 0;
            }

            *index = (*index).add(1);
            *data_offset += core::mem::size_of::<ifacepsredirect_data>() as ULONG;
            if (*data).name_len != 0 {
                *data_offset += aligned_string_len(((*data).name_len + core::mem::size_of::<u16>() as ULONG) as i32) as ULONG;
            }
        }
        i += 1;
    }

    STATUS_SUCCESS
}

unsafe fn build_ifaceps_section(actctx: *mut ACTIVATION_CONTEXT, section: *mut *mut guidsection_header) -> NTSTATUS {
    let mut total_len: u32 = 0;
    let mut count: u32 = 0;

    let mut i: u32 = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.add(i as usize);

        get_ifaceps_datalen(core::ptr::addr_of!((*assembly).entities), &mut count, &mut total_len);
        let mut j: u32 = 0;
        while j < (*assembly).num_dlls {
            let dll = (*assembly).dlls.add(j as usize);
            get_ifaceps_datalen(core::ptr::addr_of!((*dll).entities), &mut count, &mut total_len);
            j += 1;
        }
        i += 1;
    }

    total_len += core::mem::size_of::<guidsection_header>() as u32;

    let header = RtlAllocateHeap(GetProcessHeap(), 0, total_len as usize) as *mut guidsection_header;
    if header.is_null() {
        return STATUS_NO_MEMORY;
    }

    core::ptr::write_bytes(header as *mut u8, 0, core::mem::size_of::<guidsection_header>());
    (*header).magic = GUIDSECTION_MAGIC;
    (*header).size = core::mem::size_of::<guidsection_header>() as ULONG;
    (*header).count = count;
    (*header).index_offset = core::mem::size_of::<guidsection_header>() as ULONG;
    let mut index = (header as *mut u8).add((*header).index_offset as usize) as *mut guid_index;
    let mut data_offset: ULONG = (*header).index_offset + count * core::mem::size_of::<guid_index>() as u32;

    let mut i: u32 = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.add(i as usize);

        let Status = add_ifaceps_record(header, core::ptr::addr_of_mut!((*assembly).entities), &mut index, &mut data_offset, i + 1);
        if !NT_SUCCESS(Status) {
            RtlFreeHeap(GetProcessHeap(), 0, header as *mut _);
            return Status;
        }

        let mut j: u32 = 0;
        while j < (*assembly).num_dlls {
            let dll = (*assembly).dlls.add(j as usize);
            let Status = add_ifaceps_record(header, core::ptr::addr_of_mut!((*dll).entities), &mut index, &mut data_offset, i + 1);
            if !NT_SUCCESS(Status) {
                RtlFreeHeap(GetProcessHeap(), 0, header as *mut _);
                return Status;
            }
            j += 1;
        }
        i += 1;
    }

    *section = header;

    STATUS_SUCCESS
}

unsafe fn get_ifaceps_data(actctx: *mut ACTIVATION_CONTEXT, index: *mut guid_index) -> *mut ifacepsredirect_data {
    ((*actctx).ifaceps_section as *mut u8).add((*index).data_offset as usize) as *mut ifacepsredirect_data
}

unsafe fn find_cominterface_redirection(actctx: *mut ACTIVATION_CONTEXT, guid: *const GUID, data: *mut ACTCTX_SECTION_KEYED_DATA) -> NTSTATUS {
    let index: *mut guid_index;

    if ((*actctx).sections & IFACEREDIRECT_SECTION as u32) == 0 {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if (*actctx).ifaceps_section.is_null() {
        let mut section: *mut guidsection_header = core::ptr::null_mut();

        let status = build_ifaceps_section(actctx, &mut section);
        if status != 0 {
            return status;
        }

        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*actctx).ifaceps_section) as *mut *mut core::ffi::c_void,
            section as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        )
        .is_null()
        {
            RtlFreeHeap(GetProcessHeap(), 0, section as *mut core::ffi::c_void);
        }
    }

    index = find_guid_index((*actctx).ifaceps_section, guid);
    if index.is_null() {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    let iface = get_ifaceps_data(actctx, index);

    (*data).ulDataFormatVersion = 1;
    (*data).lpData = iface as *mut core::ffi::c_void;
    (*data).ulLength = (*iface).size
        + if (*iface).name_len != 0 {
            (*iface).name_len + core::mem::size_of::<WCHAR>() as u32
        } else {
            0
        };
    (*data).lpSectionGlobalData = core::ptr::null_mut();
    (*data).ulSectionGlobalDataLength = 0;
    (*data).lpSectionBase = (*actctx).ifaceps_section as *mut core::ffi::c_void;
    (*data).ulSectionTotalLength =
        RtlSizeHeap(GetProcessHeap(), 0, (*actctx).ifaceps_section as *mut core::ffi::c_void) as u32;
    (*data).hActCtx = core::ptr::null_mut();

    if (*data).cbSize as usize
        >= FIELD_OFFSET!(ACTCTX_SECTION_KEYED_DATA, ulAssemblyRosterIndex)
            + core::mem::size_of::<ULONG>()
    {
        (*data).ulAssemblyRosterIndex = (*index).rosterindex;
    }

    STATUS_SUCCESS
}

unsafe fn build_clr_surrogate_section(actctx: *mut ACTIVATION_CONTEXT, section: *mut *mut guidsection_header) -> NTSTATUS {
    let mut i: u32;
    let mut j: u32;
    let mut total_len: u32 = 0;
    let mut count: u32 = 0;
    let header: *mut guidsection_header;
    let mut data: *mut clrsurrogate_data;
    let mut index: *mut guid_index;
    let mut data_offset: ULONG;

    /* compute section length */
    i = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.add(i as usize);
        j = 0;
        while j < (*assembly).entities.num {
            let entity = (*assembly).entities.base.add(j as usize);
            if (*entity).kind == ACTIVATION_CONTEXT_SECTION_CLR_SURROGATES {
                let mut len: ULONG;

                total_len += (core::mem::size_of::<guid_index>() + core::mem::size_of::<clrsurrogate_data>()) as u32;
                len = (wcslen((*entity).u.clrsurrogate.name) + 1) as ULONG;
                if !(*entity).u.clrsurrogate.version.is_null() {
                    len += (wcslen((*entity).u.clrsurrogate.version) + 1) as ULONG;
                }
                total_len += aligned_string_len((len as usize * core::mem::size_of::<WCHAR>()) as i32) as u32;

                count += 1;
            }
            j += 1;
        }
        i += 1;
    }

    total_len += core::mem::size_of::<guidsection_header>() as u32;

    header = RtlAllocateHeap(GetProcessHeap(), 0, total_len as usize) as *mut guidsection_header;
    if header.is_null() {
        return STATUS_NO_MEMORY;
    }

    memset(header as *mut _, 0, core::mem::size_of::<guidsection_header>());
    (*header).magic = GUIDSECTION_MAGIC;
    (*header).size = core::mem::size_of::<guidsection_header>() as ULONG;
    (*header).count = count;
    (*header).index_offset = core::mem::size_of::<guidsection_header>() as ULONG;
    index = ((header as *mut u8).add((*header).index_offset as usize)) as *mut guid_index;
    data_offset = (*header).index_offset + count * core::mem::size_of::<guid_index>() as u32;

    i = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.add(i as usize);
        j = 0;
        while j < (*assembly).entities.num {
            let entity = (*assembly).entities.base.add(j as usize);
            if (*entity).kind == ACTIVATION_CONTEXT_SECTION_CLR_SURROGATES {
                let version_len: ULONG;
                let name_len: ULONG;
                let mut str: _UNICODE_STRING = core::mem::zeroed();
                let mut ptrW: *mut WCHAR;
                let Status: NTSTATUS;

                if !(*entity).u.clrsurrogate.version.is_null() {
                    version_len = (wcslen((*entity).u.clrsurrogate.version) * core::mem::size_of::<WCHAR>()) as ULONG;
                } else {
                    version_len = 0;
                }
                name_len = (wcslen((*entity).u.clrsurrogate.name) * core::mem::size_of::<WCHAR>()) as ULONG;

                /* setup new index entry */
                RtlInitUnicodeString(&mut str, (*entity).u.clrsurrogate.clsid);
                Status = RtlGUIDFromString(&mut str, core::ptr::addr_of_mut!((*index).guid));
                if !NT_SUCCESS(Status) {
                    RtlFreeHeap(GetProcessHeap(), 0, header as *mut _);
                    return Status;
                }

                (*index).data_offset = data_offset;
                (*index).data_len = core::mem::size_of::<clrsurrogate_data>() as ULONG
                    + aligned_string_len((name_len + core::mem::size_of::<WCHAR>() as ULONG
                        + if version_len != 0 { version_len + core::mem::size_of::<WCHAR>() as ULONG } else { 0 }) as i32) as ULONG;
                (*index).rosterindex = i + 1;

                /* setup data */
                data = ((header as *mut u8).add((*index).data_offset as usize)) as *mut clrsurrogate_data;
                (*data).size = core::mem::size_of::<clrsurrogate_data>() as ULONG;
                (*data).res = 0;
                (*data).clsid = (*index).guid;
                (*data).version_offset = if version_len != 0 { (*data).size } else { 0 };
                (*data).version_len = version_len;
                (*data).name_offset = (*data).size + version_len;
                if version_len != 0 {
                    (*data).name_offset += core::mem::size_of::<WCHAR>() as ULONG;
                }
                (*data).name_len = name_len;

                /* surrogate name */
                ptrW = ((data as *mut u8).add((*data).name_offset as usize)) as *mut WCHAR;
                memcpy(ptrW as *mut _, (*entity).u.clrsurrogate.name as *const _, (*data).name_len as usize);
                *ptrW.add(((*data).name_len as usize) / core::mem::size_of::<WCHAR>()) = 0;

                /* runtime version */
                if (*data).version_len != 0 {
                    ptrW = ((data as *mut u8).add((*data).version_offset as usize)) as *mut WCHAR;
                    memcpy(ptrW as *mut _, (*entity).u.clrsurrogate.version as *const _, (*data).version_len as usize);
                    *ptrW.add(((*data).version_len as usize) / core::mem::size_of::<WCHAR>()) = 0;
                }

                data_offset += (*index).data_offset;
                index = index.add(1);
            }
            j += 1;
        }
        i += 1;
    }

    *section = header;

    STATUS_SUCCESS
}

unsafe fn get_surrogate_data(actctx: *mut ACTIVATION_CONTEXT, index: *const guid_index) -> *mut clrsurrogate_data {
    ((*actctx).clrsurrogate_section as *mut u8).add((*index).data_offset as usize) as *mut clrsurrogate_data
}

unsafe fn find_clr_surrogate(actctx: *mut ACTIVATION_CONTEXT, guid: *const GUID, data: *mut ACTCTX_SECTION_KEYED_DATA) -> NTSTATUS {
    let surrogate: *mut clrsurrogate_data;
    let index: *mut guid_index;

    if ((*actctx).sections & CLRSURROGATES_SECTION as u32) == 0 {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if (*actctx).clrsurrogate_section.is_null() {
        let mut section: *mut guidsection_header = core::ptr::null_mut();

        let status = build_clr_surrogate_section(actctx, &mut section);
        if status != 0 {
            return status;
        }

        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*actctx).clrsurrogate_section) as *mut *mut core::ffi::c_void,
            section as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        )
        .is_null()
        {
            RtlFreeHeap(GetProcessHeap(), 0, section as *mut core::ffi::c_void);
        }
    }

    index = find_guid_index((*actctx).clrsurrogate_section, guid);
    if index.is_null() {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    surrogate = get_surrogate_data(actctx, index);

    (*data).ulDataFormatVersion = 1;
    (*data).lpData = surrogate as *mut core::ffi::c_void;
    /* full length includes string length with nulls */
    (*data).ulLength = (*surrogate).size + (*surrogate).name_len + core::mem::size_of::<WCHAR>() as ULONG;
    if (*surrogate).version_len != 0 {
        (*data).ulLength += (*surrogate).version_len + core::mem::size_of::<WCHAR>() as ULONG;
    }

    (*data).lpSectionGlobalData = core::ptr::null_mut();
    (*data).ulSectionGlobalDataLength = 0;
    (*data).lpSectionBase = (*actctx).clrsurrogate_section as *mut core::ffi::c_void;
    (*data).ulSectionTotalLength =
        RtlSizeHeap(GetProcessHeap(), 0, (*actctx).clrsurrogate_section as *mut core::ffi::c_void) as ULONG;
    (*data).hActCtx = core::ptr::null_mut();

    if (*data).cbSize as usize
        >= FIELD_OFFSET!(ACTCTX_SECTION_KEYED_DATA, ulAssemblyRosterIndex) + core::mem::size_of::<ULONG>()
    {
        (*data).ulAssemblyRosterIndex = (*index).rosterindex;
    }

    STATUS_SUCCESS
}

unsafe fn get_progid_datalen(entities: *mut entity_array, count: *mut core::ffi::c_uint, total_len: *mut core::ffi::c_uint) {
    let single_len: core::ffi::c_uint = (core::mem::size_of::<progidredirect_data>()
        + core::mem::size_of::<string_index>()
        + core::mem::size_of::<GUID>()) as core::ffi::c_uint;

    let mut i: core::ffi::c_uint = 0;
    while i < (*entities).num {
        let entity = (*entities).base.add(i as usize);
        if (*entity).kind == ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION {
            if !(*entity).u.comclass.progid.is_null() {
                *total_len += single_len
                    + aligned_string_len(
                        ((wcslen((*entity).u.comclass.progid) + 1)
                            * core::mem::size_of::<WCHAR>()) as i32,
                    ) as core::ffi::c_uint;
                *count += 1;
            }

            let mut j: core::ffi::c_uint = 0;
            while j < (*entity).u.comclass.progids.num {
                *total_len += aligned_string_len(
                    ((wcslen(*(*entity).u.comclass.progids.progids.add(j as usize)) + 1)
                        * core::mem::size_of::<WCHAR>()) as i32,
                ) as core::ffi::c_uint;
                j += 1;
            }

            *total_len += single_len * (*entity).u.comclass.progids.num;
            *count += (*entity).u.comclass.progids.num;
        }
        i += 1;
    }
}

unsafe fn write_progid_record(
    section: *mut strsection_header,
    progid: *const WCHAR,
    alias: *const GUID,
    index: *mut *mut string_index,
    data_offset: *mut ULONG,
    global_offset: *mut ULONG,
    rosterindex: ULONG,
) {
    let mut str: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut str, progid);
    RtlHashUnicodeString(&str, 1u8, HASH_STRING_ALGORITHM_X65599, &mut (**index).hash);

    (**index).name_offset = *data_offset;
    (**index).name_len = str.Length as ULONG;
    (**index).data_offset = (**index).name_offset
        + aligned_string_len(str.MaximumLength as i32) as ULONG;
    (**index).data_len = core::mem::size_of::<progidredirect_data>() as ULONG;
    (**index).rosterindex = rosterindex;

    *data_offset += aligned_string_len(str.MaximumLength as i32) as ULONG;

    let data = (section as *mut u8).add(*data_offset as usize) as *mut progidredirect_data;
    (*data).size = core::mem::size_of::<progidredirect_data>() as ULONG;
    (*data).reserved = 0;
    (*data).clsid_offset = *global_offset;

    let ptrW = (section as *mut u8).add((**index).name_offset as usize) as *mut WCHAR;
    memcpy(
        ptrW as *mut _,
        progid as *const _,
        (**index).name_len as usize,
    );
    *ptrW.add((**index).name_len as usize / core::mem::size_of::<WCHAR>()) = 0;

    let guid_ptr = (section as *mut u8).add((*data).clsid_offset as usize) as *mut GUID;
    *guid_ptr = *alias;

    *global_offset += core::mem::size_of::<GUID>() as ULONG;
    *data_offset += (*data).size;
    *index = (*index).add(1);
}

unsafe fn add_progid_record(
    actctx: *mut ACTIVATION_CONTEXT,
    section: *mut strsection_header,
    entities: *const entity_array,
    index: *mut *mut string_index,
    data_offset: *mut ULONG,
    global_offset: *mut ULONG,
    rosterindex: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;

    let mut i: core::ffi::c_uint = 0;
    while i < (*entities).num {
        let entity: *mut entity = (*entities).base.add(i as usize);
        if (*entity).kind == ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION {
            let progids: *const progids = core::ptr::addr_of!((*entity).u.comclass.progids);
            let comclass: *mut comclassredirect_data;
            let guid_index: *mut guid_index;
            let mut str: _UNICODE_STRING = core::mem::zeroed();
            let mut clsid: GUID = core::mem::zeroed();

            RtlInitUnicodeString(&mut str, (*entity).u.comclass.clsid);
            Status = RtlGUIDFromString(&mut str, &mut clsid);
            if !NT_SUCCESS(Status) {
                return Status;
            }

            guid_index = find_guid_index((*actctx).comserver_section, &clsid);
            comclass = get_comclass_data(actctx, guid_index);

            if !(*entity).u.comclass.progid.is_null() {
                write_progid_record(
                    section,
                    (*entity).u.comclass.progid,
                    core::ptr::addr_of!((*comclass).alias),
                    index,
                    data_offset,
                    global_offset,
                    rosterindex,
                );
            }

            let mut j: core::ffi::c_uint = 0;
            while j < (*progids).num {
                write_progid_record(
                    section,
                    *(*progids).progids.add(j as usize),
                    core::ptr::addr_of!((*comclass).alias),
                    index,
                    data_offset,
                    global_offset,
                    rosterindex,
                );
                j += 1;
            }
        }
        i += 1;
    }
    Status
}

unsafe fn build_progid_section(actctx: *mut ACTIVATION_CONTEXT, section: *mut *mut strsection_header) -> NTSTATUS {
    let mut total_len: core::ffi::c_uint = 0;
    let mut count: core::ffi::c_uint = 0;
    let mut Status: NTSTATUS;

    let mut i: core::ffi::c_uint = 0;
    while i < (*actctx).num_assemblies {
        let assembly: *mut assembly = (*actctx).assemblies.add(i as usize);

        get_progid_datalen(core::ptr::addr_of_mut!((*assembly).entities), &mut count, &mut total_len);
        let mut j: core::ffi::c_uint = 0;
        while j < (*assembly).num_dlls {
            let dll: *mut dll_redirect = (*assembly).dlls.add(j as usize);
            get_progid_datalen(core::ptr::addr_of_mut!((*dll).entities), &mut count, &mut total_len);
            j += 1;
        }
        i += 1;
    }

    total_len += core::mem::size_of::<strsection_header>() as core::ffi::c_uint;

    let header: *mut strsection_header = RtlAllocateHeap(GetProcessHeap(), 0, total_len as usize) as *mut strsection_header;
    if header.is_null() {
        return STATUS_NO_MEMORY;
    }

    memset(header as *mut core::ffi::c_void, 0, core::mem::size_of::<strsection_header>());
    (*header).magic = STRSECTION_MAGIC;
    (*header).size = core::mem::size_of::<strsection_header>() as ULONG;
    (*header).count = count;
    (*header).global_offset = (*header).size;
    (*header).global_len = count * core::mem::size_of::<GUID>() as ULONG;
    (*header).index_offset = (*header).size + (*header).global_len;

    let mut index: *mut string_index = ((header as *mut u8).add((*header).index_offset as usize)) as *mut string_index;
    let mut data_offset: ULONG = (*header).index_offset + count * core::mem::size_of::<string_index>() as ULONG;
    let mut global_offset: ULONG = (*header).global_offset;

    let mut i: core::ffi::c_uint = 0;
    while i < (*actctx).num_assemblies {
        let assembly: *mut assembly = (*actctx).assemblies.add(i as usize);

        Status = add_progid_record(actctx, header, core::ptr::addr_of_mut!((*assembly).entities), &mut index, &mut data_offset, &mut global_offset, i + 1);
        if !NT_SUCCESS(Status) {
            RtlFreeHeap(GetProcessHeap(), 0, header as *mut core::ffi::c_void);
            return Status;
        }

        let mut j: core::ffi::c_uint = 0;
        while j < (*assembly).num_dlls {
            let dll: *mut dll_redirect = (*assembly).dlls.add(j as usize);
            Status = add_progid_record(actctx, header, core::ptr::addr_of_mut!((*dll).entities), &mut index, &mut data_offset, &mut global_offset, i + 1);
            if !NT_SUCCESS(Status) {
                RtlFreeHeap(GetProcessHeap(), 0, header as *mut core::ffi::c_void);
                return Status;
            }
            j += 1;
        }
        i += 1;
    }

    *section = header;

    STATUS_SUCCESS
}

unsafe fn get_progid_data(actctx: *mut ACTIVATION_CONTEXT, index: *const string_index) -> *mut progidredirect_data {
    ((*actctx).progid_section as *mut u8).add((*index).data_offset as usize) as *mut progidredirect_data
}

unsafe fn find_progid_redirection(
    actctx: *mut ACTIVATION_CONTEXT,
    name: *const _UNICODE_STRING,
    data: PACTCTX_SECTION_KEYED_DATA,
) -> NTSTATUS {
    let progid: *mut progidredirect_data;
    let index: *mut string_index;

    if (*actctx).sections & PROGIDREDIRECT_SECTION as u32 == 0 {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if (*actctx).comserver_section.is_null() {
        let mut section: *mut guidsection_header = core::ptr::null_mut();

        let status = build_comserver_section(actctx, &mut section);
        if status != 0 {
            return status;
        }

        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*actctx).comserver_section) as *mut *mut core::ffi::c_void,
            section as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        )
        .is_null()
        {
            RtlFreeHeap(GetProcessHeap(), 0, section as *mut core::ffi::c_void);
        }
    }

    if (*actctx).progid_section.is_null() {
        let mut section: *mut strsection_header = core::ptr::null_mut();

        let status = build_progid_section(actctx, &mut section);
        if status != 0 {
            return status;
        }

        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*actctx).progid_section) as *mut *mut core::ffi::c_void,
            section as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        )
        .is_null()
        {
            RtlFreeHeap(GetProcessHeap(), 0, section as *mut core::ffi::c_void);
        }
    }

    index = find_string_index((*actctx).progid_section, name);
    if index.is_null() {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if !data.is_null() {
        progid = get_progid_data(actctx, index);

        (*data).ulDataFormatVersion = 1;
        (*data).lpData = progid as *mut core::ffi::c_void;
        (*data).ulLength = (*progid).size;
        (*data).lpSectionGlobalData = ((*actctx).progid_section as *mut BYTE)
            .add((*(*actctx).progid_section).global_offset as usize)
            as *mut core::ffi::c_void;
        (*data).ulSectionGlobalDataLength = (*(*actctx).progid_section).global_len;
        (*data).lpSectionBase = (*actctx).progid_section as *mut core::ffi::c_void;
        (*data).ulSectionTotalLength =
            RtlSizeHeap(GetProcessHeap(), 0, (*actctx).progid_section as *mut core::ffi::c_void) as ULONG;
        (*data).hActCtx = core::ptr::null_mut();

        if (*data).cbSize as usize
            >= FIELD_OFFSET!(ACTCTX_SECTION_KEYED_DATA, ulAssemblyRosterIndex)
                + core::mem::size_of::<ULONG>()
        {
            (*data).ulAssemblyRosterIndex = (*index).rosterindex;
        }
    }

    STATUS_SUCCESS
}

unsafe fn find_string(
    actctx: *mut ACTIVATION_CONTEXT,
    section_kind: ULONG,
    section_name: *const _UNICODE_STRING,
    flags: DWORD,
    data: PACTCTX_SECTION_KEYED_DATA,
) -> NTSTATUS {
    let status: NTSTATUS;

    match section_kind {
        ACTIVATION_CONTEXT_SECTION_ASSEMBLY_INFORMATION => {
            return STATUS_SXS_KEY_NOT_FOUND;
        }
        ACTIVATION_CONTEXT_SECTION_DLL_REDIRECTION => {
            status = find_dll_redirection(actctx, section_name, data);
        }
        ACTIVATION_CONTEXT_SECTION_WINDOW_CLASS_REDIRECTION => {
            status = find_window_class(actctx, section_name, data);
        }
        ACTIVATION_CONTEXT_SECTION_COM_PROGID_REDIRECTION => {
            status = find_progid_redirection(actctx, section_name, data);
        }
        ACTIVATION_CONTEXT_SECTION_GLOBAL_OBJECT_RENAME_TABLE => {
            return STATUS_SXS_SECTION_NOT_FOUND;
        }
        _ => {
            return STATUS_SXS_SECTION_NOT_FOUND;
        }
    }

    if status != STATUS_SUCCESS {
        return status;
    }

    if !data.is_null() && (flags & FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX) != 0 {
        actctx_addref(actctx);
        (*data).hActCtx = actctx as PVOID;
    }

    STATUS_SUCCESS
}

unsafe fn find_guid(
    actctx: *mut ACTIVATION_CONTEXT,
    section_kind: ULONG,
    guid: *const GUID,
    flags: DWORD,
    data: PACTCTX_SECTION_KEYED_DATA,
) -> NTSTATUS {
    let status: NTSTATUS;

    match section_kind {
        ACTIVATION_CONTEXT_SECTION_COM_TYPE_LIBRARY_REDIRECTION => {
            status = find_tlib_redirection(actctx, guid, data);
        }
        ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION => {
            status = find_comserver_redirection(actctx, guid, data);
        }
        ACTIVATION_CONTEXT_SECTION_COM_INTERFACE_REDIRECTION => {
            status = find_cominterface_redirection(actctx, guid, data);
        }
        ACTIVATION_CONTEXT_SECTION_CLR_SURROGATES => {
            status = find_clr_surrogate(actctx, guid, data);
        }
        _ => {
            return STATUS_SXS_SECTION_NOT_FOUND;
        }
    }

    if status != STATUS_SUCCESS {
        return status;
    }

    if (flags & FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX) != 0 {
        actctx_addref(actctx);
        (*data).hActCtx = actctx as *mut _;
    }

    STATUS_SUCCESS
}

unsafe fn find_app_settings(actctx: *mut ACTIVATION_CONTEXT, settings: *const WCHAR, ns: *const WCHAR) -> *const WCHAR {
    let mut i: u32 = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.add(i as usize);
        let mut j: u32 = 0;
        while j < (*assembly).entities.num {
            let entity = (*assembly).entities.base.add(j as usize);
            if (*entity).kind == ACTIVATION_CONTEXT_SECTION_APPLICATION_SETTINGS
                && wcscmp((*entity).u.settings.name, settings) == 0
                && wcscmp((*entity).u.settings.ns, ns) == 0
            {
                return (*entity).u.settings.value;
            }
            j += 1;
        }
        i += 1;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddRefActivationContext(handle: HANDLE) {
    let actctx = check_actctx(handle);
    if !actctx.is_null() {
        actctx_addref(actctx);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlReleaseActivationContext(handle: HANDLE) {
    let actctx = check_actctx(handle);
    if !actctx.is_null() {
        actctx_release(actctx);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlZombifyActivationContext(handle: HANDLE) -> NTSTATUS {
    FIXME!("%p: stub\n", handle);

    if handle == ACTCTX_FAKE_HANDLE {
        return STATUS_SUCCESS;
    }

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlActivateActivationContext(
    _unknown: ULONG,
    handle: HANDLE,
    cookie: PULONG_PTR,
) -> NTSTATUS {
    RtlActivateActivationContextEx(0, NtCurrentTeb(), handle, cookie)
}

#[no_mangle]
pub unsafe extern "system" fn RtlActivateActivationContextEx(
    _flags: ULONG,
    teb: *mut TEB,
    handle: HANDLE,
    cookie: *mut ULONG_PTR,
) -> NTSTATUS {
    let frame = RtlAllocateHeap(
        GetProcessHeap(),
        0,
        core::mem::size_of::<RTL_ACTIVATION_CONTEXT_STACK_FRAME>(),
    ) as *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME;

    if frame.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*frame).Previous = (*(*teb).ActivationContextStackPointer).ActiveFrame as PRTL_ACTIVATION_CONTEXT_STACK_FRAME;
    (*frame).ActivationContext = handle as PACTIVATION_CONTEXT;
    (*frame).Flags = 0;

    (*(*teb).ActivationContextStackPointer).ActiveFrame = frame as *mut _;
    RtlAddRefActivationContext(handle);

    core::ptr::write_volatile(cookie, frame as ULONG_PTR);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeactivateActivationContext(flags: ULONG, cookie: ULONG_PTR) -> NTSTATUS {
    let mut frame: *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME;
    let mut top: *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME;

    top = (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame as PRTL_ACTIVATION_CONTEXT_STACK_FRAME;
    frame = top;
    while !frame.is_null() {
        if frame as ULONG_PTR == cookie {
            break;
        }
        frame = (*frame).Previous;
    }

    if frame.is_null() {
        RtlRaiseStatus(STATUS_SXS_INVALID_DEACTIVATION);
    }

    if frame != top
        && (flags & RTL_DEACTIVATE_ACTIVATION_CONTEXT_FLAG_FORCE_EARLY_DEACTIVATION) == 0
    {
        RtlRaiseStatus(STATUS_SXS_EARLY_DEACTIVATION);
    }

    (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame =
        (*frame).Previous as *mut _;

    while top != (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame as PRTL_ACTIVATION_CONTEXT_STACK_FRAME {
        frame = (*top).Previous;
        RtlReleaseActivationContext((*top).ActivationContext as HANDLE);
        RtlFreeHeap(GetProcessHeap(), 0, top as PVOID);
        top = frame;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlFreeActivationContextStack(Stack: PACTIVATION_CONTEXT_STACK) {
    if Stack.is_null() {
        return;
    }

    let mut ActiveFrame = (*Stack).ActiveFrame as PRTL_ACTIVATION_CONTEXT_STACK_FRAME;

    while !ActiveFrame.is_null() {
        let PrevFrame = (*ActiveFrame).Previous;
        RtlReleaseActivationContext((*ActiveFrame).ActivationContext as HANDLE);
        RtlFreeHeap(GetProcessHeap(), 0, ActiveFrame as *mut _);
        ActiveFrame = PrevFrame;
    }

    (*Stack).ActiveFrame = core::ptr::null_mut();

    RtlFreeHeap(GetProcessHeap(), 0, Stack as *mut _);
}

#[no_mangle]
pub unsafe extern "system" fn RtlFreeThreadActivationContextStack() {
    RtlFreeActivationContextStack((*NtCurrentTeb()).ActivationContextStackPointer as PACTIVATION_CONTEXT_STACK);
    (*NtCurrentTeb()).ActivationContextStackPointer = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetActiveActivationContext(handle: *mut HANDLE) -> NTSTATUS {
    if !(*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame.is_null() {
        core::ptr::write_volatile(handle, (*((*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame
            as PRTL_ACTIVATION_CONTEXT_STACK_FRAME))
            .ActivationContext as HANDLE);
        RtlAddRefActivationContext(*handle);
    } else {
        core::ptr::write_volatile(handle, core::ptr::null_mut());
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsActivationContextActive(handle: HANDLE) -> BOOLEAN {
    let mut frame = (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame
        as PRTL_ACTIVATION_CONTEXT_STACK_FRAME;
    while !frame.is_null() {
        if (*frame).ActivationContext == handle as PACTIVATION_CONTEXT {
            return 1u8;
        }
        frame = (*frame).Previous;
    }
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueryInformationActivationContext(
    flags: ULONG,
    mut handle: HANDLE,
    subinst: PVOID,
    class: ULONG,
    buffer: PVOID,
    bufsize: SIZE_T,
    retlen: *mut SIZE_T,
) -> NTSTATUS {
    let actctx: *mut ACTIVATION_CONTEXT;
    let status: NTSTATUS;

    if !retlen.is_null() {
        core::ptr::write_volatile(retlen, 0);
    }

    status = find_query_actctx(&mut handle, flags, class);
    if status != 0 {
        return status;
    }

    match class {
        x if x == ActivationContextBasicInformation => {
            let info = buffer as *mut ACTIVATION_CONTEXT_BASIC_INFORMATION;

            if !retlen.is_null() {
                core::ptr::write_volatile(retlen, core::mem::size_of::<ACTIVATION_CONTEXT_BASIC_INFORMATION>() as SIZE_T);
            }
            if info.is_null()
                || (bufsize as usize) < core::mem::size_of::<ACTIVATION_CONTEXT_BASIC_INFORMATION>()
            {
                return STATUS_BUFFER_TOO_SMALL;
            }

            (*info).hActCtx = handle;
            (*info).dwFlags = 0;
            if (flags & RTL_QUERY_ACTIVATION_CONTEXT_FLAG_NO_ADDREF) == 0 {
                RtlAddRefActivationContext(handle);
            }
        }

        x if x == ActivationContextDetailedInformation => {
            let acdi = buffer as *mut ACTIVATION_CONTEXT_DETAILED_INFORMATION;
            let mut assembly: *mut assembly = core::ptr::null_mut();
            let mut manifest_len: SIZE_T = 0;
            let mut config_len: SIZE_T = 0;
            let mut appdir_len: SIZE_T = 0;
            let len: SIZE_T;
            let mut ptr: LPWSTR;

            let actctx = check_actctx(handle);
            if actctx.is_null() {
                return STATUS_INVALID_PARAMETER;
            }

            if (*actctx).num_assemblies != 0 {
                assembly = (*actctx).assemblies;
            }

            if !assembly.is_null() && !(*assembly).manifest.info.is_null() {
                manifest_len = (wcslen((*assembly).manifest.info) + 1) as SIZE_T;
            }
            if !(*actctx).config.info.is_null() {
                config_len = (wcslen((*actctx).config.info) + 1) as SIZE_T;
            }
            if !(*actctx).appdir.info.is_null() {
                appdir_len = (wcslen((*actctx).appdir.info) + 1) as SIZE_T;
            }
            len = (core::mem::size_of::<ACTIVATION_CONTEXT_DETAILED_INFORMATION>() as SIZE_T)
                + (manifest_len + config_len + appdir_len)
                    * core::mem::size_of::<WCHAR>() as SIZE_T;

            if !retlen.is_null() {
                core::ptr::write_volatile(retlen, len);
            }
            if buffer.is_null() || bufsize < len {
                return STATUS_BUFFER_TOO_SMALL;
            }

            (*acdi).dwFlags = 0;
            (*acdi).ulFormatVersion = if !assembly.is_null() { 1 } else { 0 };
            (*acdi).ulAssemblyCount = (*actctx).num_assemblies;
            (*acdi).ulRootManifestPathType = if !assembly.is_null() {
                (*assembly).manifest.type_
            } else {
                0
            };
            (*acdi).ulRootManifestPathChars =
                if !assembly.is_null() && !(*assembly).manifest.info.is_null() {
                    (manifest_len - 1) as ULONG
                } else {
                    0
                };
            (*acdi).ulRootConfigurationPathType = (*actctx).config.type_;
            (*acdi).ulRootConfigurationPathChars = if !(*actctx).config.info.is_null() {
                (config_len - 1) as ULONG
            } else {
                0
            };
            (*acdi).ulAppDirPathType = (*actctx).appdir.type_;
            (*acdi).ulAppDirPathChars = if !(*actctx).appdir.info.is_null() {
                (appdir_len - 1) as ULONG
            } else {
                0
            };
            ptr = acdi.add(1) as LPWSTR;
            if manifest_len != 0 {
                (*acdi).lpRootManifestPath = ptr;
                memcpy(
                    ptr as PVOID,
                    (*assembly).manifest.info as PVOID,
                    (manifest_len as usize) * core::mem::size_of::<WCHAR>(),
                );
                ptr = ptr.add(manifest_len as usize);
            } else {
                (*acdi).lpRootManifestPath = core::ptr::null_mut();
            }
            if config_len != 0 {
                (*acdi).lpRootConfigurationPath = ptr;
                memcpy(
                    ptr as PVOID,
                    (*actctx).config.info as PVOID,
                    (config_len as usize) * core::mem::size_of::<WCHAR>(),
                );
                ptr = ptr.add(config_len as usize);
            } else {
                (*acdi).lpRootConfigurationPath = core::ptr::null_mut();
            }
            if appdir_len != 0 {
                (*acdi).lpAppDirPath = ptr;
                memcpy(
                    ptr as PVOID,
                    (*actctx).appdir.info as PVOID,
                    (appdir_len as usize) * core::mem::size_of::<WCHAR>(),
                );
            } else {
                (*acdi).lpAppDirPath = core::ptr::null_mut();
            }
        }

        x if x == AssemblyDetailedInformationInActivationContext => {
            let afdi = buffer as *mut ACTIVATION_CONTEXT_ASSEMBLY_DETAILED_INFORMATION;
            let assembly: *mut assembly;
            let assembly_id: *mut WCHAR;
            let index: DWORD;
            let len: SIZE_T;
            let id_len: SIZE_T;
            let mut ad_len: SIZE_T = 0;
            let mut path_len: SIZE_T = 0;
            let mut ptr: LPWSTR;

            let actctx = check_actctx(handle);
            if actctx.is_null() {
                return STATUS_INVALID_PARAMETER;
            }
            if subinst.is_null() {
                return STATUS_INVALID_PARAMETER;
            }

            index = *(subinst as *mut DWORD);
            if index == 0 || index > (*actctx).num_assemblies {
                return STATUS_INVALID_PARAMETER;
            }

            assembly = (*actctx).assemblies.add((index - 1) as usize);

            assembly_id = build_assembly_id(core::ptr::addr_of_mut!((*assembly).id));
            if assembly_id.is_null() {
                return STATUS_NO_MEMORY;
            }
            id_len = (wcslen(assembly_id) + 1) as SIZE_T;
            if !(*assembly).directory.is_null() {
                ad_len = (wcslen((*assembly).directory) + 1) as SIZE_T;
            }

            if !(*assembly).manifest.info.is_null()
                && ((*assembly).type_ == ASSEMBLY_MANIFEST
                    || (*assembly).type_ == ASSEMBLY_SHARED_MANIFEST)
            {
                path_len = (wcslen((*assembly).manifest.info) + 1) as SIZE_T;
            }

            len = (core::mem::size_of::<ACTIVATION_CONTEXT_ASSEMBLY_DETAILED_INFORMATION>()
                as SIZE_T)
                + (id_len + ad_len + path_len) * core::mem::size_of::<WCHAR>() as SIZE_T;

            if !retlen.is_null() {
                core::ptr::write_volatile(retlen, len);
            }
            if buffer.is_null() || bufsize < len {
                RtlFreeHeap(GetProcessHeap(), 0, assembly_id as PVOID);
                return STATUS_BUFFER_TOO_SMALL;
            }

            (*afdi).ulFlags = 0;
            (*afdi).ulEncodedAssemblyIdentityLength =
                ((id_len - 1) * core::mem::size_of::<WCHAR>() as SIZE_T) as ULONG;
            (*afdi).ulManifestPathType = (*assembly).manifest.type_;
            (*afdi).ulManifestPathLength = if !(*assembly).manifest.info.is_null() {
                ((path_len - 1) * core::mem::size_of::<WCHAR>() as SIZE_T) as ULONG
            } else {
                0
            };
            (*afdi).ulPolicyPathType = ACTIVATION_CONTEXT_PATH_TYPE_NONE;
            (*afdi).ulPolicyPathLength = 0;
            (*afdi).ulMetadataSatelliteRosterIndex = 0;
            (*afdi).ulManifestVersionMajor = 1;
            (*afdi).ulManifestVersionMinor = 0;
            (*afdi).ulPolicyVersionMajor = 0;
            (*afdi).ulPolicyVersionMinor = 0;
            (*afdi).ulAssemblyDirectoryNameLength = if ad_len != 0 {
                ((ad_len - 1) * core::mem::size_of::<WCHAR>() as SIZE_T) as ULONG
            } else {
                0
            };
            ptr = afdi.add(1) as LPWSTR;
            (*afdi).lpAssemblyEncodedAssemblyIdentity = ptr;
            memcpy(
                ptr as PVOID,
                assembly_id as PVOID,
                (id_len as usize) * core::mem::size_of::<WCHAR>(),
            );
            ptr = ptr.add(id_len as usize);
            if path_len != 0 {
                (*afdi).lpAssemblyManifestPath = ptr;
                memcpy(
                    ptr as PVOID,
                    (*assembly).manifest.info as PVOID,
                    (path_len as usize) * core::mem::size_of::<WCHAR>(),
                );
                ptr = ptr.add(path_len as usize);
            } else {
                (*afdi).lpAssemblyManifestPath = core::ptr::null_mut();
            }
            (*afdi).lpAssemblyPolicyPath = core::ptr::null_mut();
            if ad_len != 0 {
                (*afdi).lpAssemblyDirectoryName = ptr;
                memcpy(
                    ptr as PVOID,
                    (*assembly).directory as PVOID,
                    (ad_len as usize) * core::mem::size_of::<WCHAR>(),
                );
            } else {
                (*afdi).lpAssemblyDirectoryName = core::ptr::null_mut();
            }
            RtlFreeHeap(GetProcessHeap(), 0, assembly_id as PVOID);
        }

        x if x == FileInformationInAssemblyOfAssemblyInActivationContext => {
            let acqi = subinst as *const ACTIVATION_CONTEXT_QUERY_INDEX;
            let afdi = buffer as *mut ASSEMBLY_FILE_DETAILED_INFORMATION;
            let assembly: *mut assembly;
            let dll: *mut dll_redirect;
            let len: SIZE_T;
            let mut dll_len: SIZE_T = 0;
            let ptr: LPWSTR;

            let actctx = check_actctx(handle);
            if actctx.is_null() {
                return STATUS_INVALID_PARAMETER;
            }
            if acqi.is_null() {
                return STATUS_INVALID_PARAMETER;
            }

            if (*acqi).ulAssemblyIndex >= (*actctx).num_assemblies {
                return STATUS_INVALID_PARAMETER;
            }
            assembly = (*actctx).assemblies.add((*acqi).ulAssemblyIndex as usize);

            if (*acqi).ulFileIndexInAssembly >= (*assembly).num_dlls {
                return STATUS_INVALID_PARAMETER;
            }
            dll = (*assembly).dlls.add((*acqi).ulFileIndexInAssembly as usize);

            if !(*dll).name.is_null() {
                dll_len = (wcslen((*dll).name) + 1) as SIZE_T;
            }
            len = (core::mem::size_of::<ASSEMBLY_FILE_DETAILED_INFORMATION>() as SIZE_T)
                + dll_len * core::mem::size_of::<WCHAR>() as SIZE_T;

            if buffer.is_null() || bufsize < len {
                if !retlen.is_null() {
                    core::ptr::write_volatile(retlen, len);
                }
                return STATUS_BUFFER_TOO_SMALL;
            }
            if !retlen.is_null() {
                core::ptr::write_volatile(retlen, 0);
            }
            (*afdi).ulFlags = ACTIVATION_CONTEXT_SECTION_DLL_REDIRECTION;
            (*afdi).ulFilenameLength = if dll_len != 0 {
                ((dll_len - 1) * core::mem::size_of::<WCHAR>() as SIZE_T) as ULONG
            } else {
                0
            };
            (*afdi).ulPathLength = 0;
            ptr = afdi.add(1) as LPWSTR;
            if dll_len != 0 {
                (*afdi).lpFileName = ptr;
                memcpy(
                    ptr as PVOID,
                    (*dll).name as PVOID,
                    (dll_len as usize) * core::mem::size_of::<WCHAR>(),
                );
            } else {
                (*afdi).lpFileName = core::ptr::null_mut();
            }
            (*afdi).lpFilePath = core::ptr::null_mut();
        }

        x if x == CompatibilityInformationInActivationContext => {
            let acci = buffer as *mut DWORD;
            let elements: *mut COMPATIBILITY_CONTEXT_ELEMENT;
            let mut assembly: *mut assembly = core::ptr::null_mut();
            let mut num_compat_contexts: ULONG = 0;
            let mut n: ULONG;
            let len: SIZE_T;

            let actctx = check_actctx(handle);
            if actctx.is_null() {
                return STATUS_INVALID_PARAMETER;
            }

            if (*actctx).num_assemblies != 0 {
                assembly = (*actctx).assemblies;
            }

            if !assembly.is_null() {
                num_compat_contexts = (*assembly).num_compat_contexts;
            }
            len = (core::mem::size_of::<DWORD>() as SIZE_T)
                + (num_compat_contexts as SIZE_T)
                    * core::mem::size_of::<COMPATIBILITY_CONTEXT_ELEMENT>() as SIZE_T;

            if !retlen.is_null() {
                core::ptr::write_volatile(retlen, len);
            }
            if buffer.is_null() || bufsize < len {
                return STATUS_BUFFER_TOO_SMALL;
            }

            *acci = num_compat_contexts;
            elements = acci.add(1) as *mut COMPATIBILITY_CONTEXT_ELEMENT;
            n = 0;
            while n < num_compat_contexts {
                *elements.add(n as usize) = *(*assembly).compat_contexts.add(n as usize);
                n += 1;
            }
        }

        x if x == RunlevelInformationInActivationContext => {
            let acrli = buffer as *mut ACTIVATION_CONTEXT_RUN_LEVEL_INFORMATION;
            let assembly: *mut assembly;
            let len: SIZE_T;

            let actctx = check_actctx(handle);
            if actctx.is_null() {
                return STATUS_INVALID_PARAMETER;
            }

            len = core::mem::size_of::<ACTIVATION_CONTEXT_RUN_LEVEL_INFORMATION>() as SIZE_T;
            if !retlen.is_null() {
                core::ptr::write_volatile(retlen, len);
            }
            if buffer.is_null() || bufsize < len {
                return STATUS_BUFFER_TOO_SMALL;
            }

            assembly = (*actctx).assemblies;

            (*acrli).ulFlags = 0;
            (*acrli).RunLevel = if !assembly.is_null() {
                (*assembly).run_level
            } else {
                ACTCTX_RUN_LEVEL_UNSPECIFIED
            };
            (*acrli).UiAccess = if !assembly.is_null() {
                (*assembly).ui_access
            } else {
                0
            };
        }

        _ => {
            FIXME!("class %u not implemented\n", class);
            return STATUS_NOT_IMPLEMENTED;
        }
    }

    let _ = actctx;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueryInformationActiveActivationContext(
    ulInfoClass: ULONG,
    pvBuffer: PVOID,
    cbBuffer: SIZE_T,
    pcbWrittenOrRequired: *mut SIZE_T,
) -> NTSTATUS {
    RtlQueryInformationActivationContext(
        RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT as ULONG,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        ulInfoClass,
        pvBuffer,
        cbBuffer,
        pcbWrittenOrRequired,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlpFindActivationContextSection_CheckParameters(
    flags: ULONG,
    _guid: *const GUID,
    _section_kind: ULONG,
    section_name: *const _UNICODE_STRING,
    data: PACTCTX_SECTION_KEYED_DATA,
) -> NTSTATUS {
    const FIND_ACTCTX_RETURN_FLAGS: ULONG = 0x00000002;
    const FIND_ACTCTX_RETURN_ASSEMBLY_METADATA: ULONG = 0x00000004;
    const FIND_ACTCTX_VALID_MASK: ULONG =
        FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX | FIND_ACTCTX_RETURN_FLAGS | FIND_ACTCTX_RETURN_ASSEMBLY_METADATA;

    if section_name.is_null()
        || (*section_name).Buffer.is_null()
        || (flags & !FIND_ACTCTX_VALID_MASK) != 0
        || ((flags & FIND_ACTCTX_VALID_MASK) != 0 && data.is_null())
        || (!data.is_null()
            && ((*data).cbSize as usize)
                < core::mem::offset_of!(_ACTCTX_SECTION_KEYED_DATA, ulAssemblyRosterIndex))
    {
        return STATUS_INVALID_PARAMETER;
    }

    if (flags & FIND_ACTCTX_RETURN_FLAGS) != 0
        || (flags & FIND_ACTCTX_RETURN_ASSEMBLY_METADATA) != 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlFindActivationContextSectionString(
    flags: ULONG,
    guid: *const GUID,
    section_kind: ULONG,
    section_name: *const _UNICODE_STRING,
    ptr: PVOID,
) -> NTSTATUS {
    let data: PACTCTX_SECTION_KEYED_DATA = ptr as PACTCTX_SECTION_KEYED_DATA;
    let mut status: NTSTATUS;

    status = RtlpFindActivationContextSection_CheckParameters(flags, guid, section_kind, section_name, data);
    if !NT_SUCCESS(status) {
        return status;
    }

    status = STATUS_SXS_KEY_NOT_FOUND;

    if data.is_null() {
        return status;
    }

    let teb = NtCurrentTeb();
    let active_frame = (*(*teb).ActivationContextStackPointer).ActiveFrame
        as PRTL_ACTIVATION_CONTEXT_STACK_FRAME;
    if !active_frame.is_null() {
        let actctx = check_actctx((*active_frame).ActivationContext as HANDLE);
        if !actctx.is_null() {
            status = find_string(actctx, section_kind, section_name, flags, data);
        }
    }

    if status != STATUS_SUCCESS {
        status = find_string(process_actctx, section_kind, section_name, flags, data);
    }

    if status != STATUS_SUCCESS {
        status = find_string(implicit_actctx, section_kind, section_name, flags, data);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn RtlFindActivationContextSectionGuid(
    flags: ULONG,
    extguid: *const GUID,
    section_kind: ULONG,
    guid: *const GUID,
    ptr: *mut core::ffi::c_void,
) -> NTSTATUS {
    let data: PACTCTX_SECTION_KEYED_DATA = ptr as PACTCTX_SECTION_KEYED_DATA;
    let mut status: NTSTATUS = STATUS_SXS_KEY_NOT_FOUND;

    if !extguid.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if flags & !FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if data.is_null()
        || ((*data).cbSize as usize)
            < FIELD_OFFSET!(ACTCTX_SECTION_KEYED_DATA, ulAssemblyRosterIndex)
        || guid.is_null()
    {
        return STATUS_INVALID_PARAMETER;
    }

    if !(*(*NtCurrentTeb()).ActivationContextStackPointer)
        .ActiveFrame
        .is_null()
    {
        let actctx: *mut ACTIVATION_CONTEXT = check_actctx(
            (*((*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame
                as PRTL_ACTIVATION_CONTEXT_STACK_FRAME))
                .ActivationContext as HANDLE,
        );
        if !actctx.is_null() {
            status = find_guid(actctx, section_kind, guid, flags, data);
        }
    }

    if status != STATUS_SUCCESS {
        status = find_guid(process_actctx, section_kind, guid, flags, data);
    }

    if status != STATUS_SUCCESS {
        status = find_guid(implicit_actctx, section_kind, guid, flags, data);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueryActivationContextApplicationSettings(
    flags: DWORD,
    mut handle: HANDLE,
    mut ns: *const WCHAR,
    settings: *const WCHAR,
    buffer: *mut WCHAR,
    size: SIZE_T,
    written: *mut SIZE_T,
) -> NTSTATUS {
    let actctx: *mut ACTIVATION_CONTEXT;
    let res: *const WCHAR;

    if flags != 0 {
        WARN!(b"unknown flags %08x\n\0".as_ptr() as *const i8, flags);
        return STATUS_INVALID_PARAMETER;
    }

    if !ns.is_null() {
        if wcscmp(ns, windowsSettings2005NSW.as_ptr()) != 0
            && wcscmp(ns, windowsSettings2011NSW.as_ptr()) != 0
            && wcscmp(ns, windowsSettings2016NSW.as_ptr()) != 0
            && wcscmp(ns, windowsSettings2017NSW.as_ptr()) != 0
        {
            return STATUS_INVALID_PARAMETER;
        }
    } else {
        ns = windowsSettings2005NSW.as_ptr();
    }

    if handle.is_null() {
        handle = process_actctx as HANDLE;
    }
    actctx = check_actctx(handle);
    if actctx.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    res = find_app_settings(actctx, settings, ns);
    if res.is_null() {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if !written.is_null() {
        core::ptr::write_volatile(written, (wcslen(res) + 1) as SIZE_T);
    }
    if size < wcslen(res) as SIZE_T {
        return STATUS_BUFFER_TOO_SMALL;
    }
    wcscpy(buffer, res);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlAllocateActivationContextStack(Stack: *mut PACTIVATION_CONTEXT_STACK) -> NTSTATUS {
    let ContextStack: PACTIVATION_CONTEXT_STACK;

    if !(*Stack).is_null() {
        return STATUS_SUCCESS;
    }

    ContextStack = RtlAllocateHeap(GetProcessHeap(), 0x00000008, core::mem::size_of::<ACTIVATION_CONTEXT_STACK>() as usize) as PACTIVATION_CONTEXT_STACK;
    if ContextStack.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*ContextStack).Flags = 0;
    (*ContextStack).ActiveFrame = core::ptr::null_mut();
    InitializeListHead(core::ptr::addr_of_mut!((*ContextStack).FrameListCache));
    (*ContextStack).NextCookieSequenceNumber = 1;
    (*ContextStack).StackId = 1;

    core::ptr::write_volatile(Stack, ContextStack);

    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn RtlActivateActivationContextUnsafeFast(
    Frame: PRTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
    Context: PVOID,
) -> PRTL_ACTIVATION_CONTEXT_STACK_FRAME {
    RtlActivateActivationContextUnsafeFast_impl(Frame, Context)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn RtlActivateActivationContextUnsafeFast(
    Frame: PRTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
    Context: PVOID,
) -> PRTL_ACTIVATION_CONTEXT_STACK_FRAME {
    RtlActivateActivationContextUnsafeFast_impl(Frame, Context)
}

unsafe fn RtlActivateActivationContextUnsafeFast_impl(
    Frame: PRTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
    Context: PVOID,
) -> PRTL_ACTIVATION_CONTEXT_STACK_FRAME {
    let NewFrame: *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME;
    let ActiveFrame: *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME;

    ActiveFrame = (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame as PRTL_ACTIVATION_CONTEXT_STACK_FRAME;

    if (*Frame).Size as usize
        >= core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED>()
    {
        (*Frame).Extra1 = !(ActiveFrame as ULONG_PTR) as PVOID;
        (*Frame).Extra2 = !(Context as ULONG_PTR) as PVOID;
    }

    if !ActiveFrame.is_null() {
        if ((*ActiveFrame).Flags & RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_HEAP_ALLOCATED) == 0 {
            // TODO: Perform some additional checks if it was not heap allocated
        }
    }

    NewFrame = core::ptr::addr_of_mut!((*Frame).Frame);

    (*Frame).Frame.Previous = ActiveFrame;
    (*Frame).Frame.ActivationContext = Context as *mut _;
    (*Frame).Frame.Flags = RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_ACTIVATED;

    if (!ActiveFrame.is_null() && ((*ActiveFrame).ActivationContext as PVOID != Context))
        || !Context.is_null()
    {
        (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame = NewFrame as *mut _;
        return NewFrame;
    }

    (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame = NewFrame as *mut _;

    NewFrame
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn RtlDeactivateActivationContextUnsafeFast(
    Frame: PRTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
) -> PRTL_ACTIVATION_CONTEXT_STACK_FRAME {
    RtlDeactivateActivationContextUnsafeFast_impl(Frame)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn RtlDeactivateActivationContextUnsafeFast(
    Frame: PRTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
) -> PRTL_ACTIVATION_CONTEXT_STACK_FRAME {
    RtlDeactivateActivationContextUnsafeFast_impl(Frame)
}

unsafe fn RtlDeactivateActivationContextUnsafeFast_impl(
    Frame: PRTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
) -> PRTL_ACTIVATION_CONTEXT_STACK_FRAME {
    let ActiveFrame: PRTL_ACTIVATION_CONTEXT_STACK_FRAME;
    let NewFrame: PRTL_ACTIVATION_CONTEXT_STACK_FRAME;

    ActiveFrame = (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame as PRTL_ACTIVATION_CONTEXT_STACK_FRAME;

    if (*Frame).Frame.Flags & RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_NOT_REALLY_ACTIVATED != 0 {
        (*Frame).Frame.Flags |= RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_DEACTIVATED;
        return core::ptr::addr_of_mut!((*Frame).Frame);
    }

    NewFrame = core::ptr::addr_of_mut!((*Frame).Frame);
    let _ = ActiveFrame;

    (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame = (*NewFrame).Previous as *mut _;

    (*Frame).Frame.Flags |= RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_DEACTIVATED;
    (*NewFrame).Previous
}

// ════════════════════════════════════════════════════════════════════
//  Hand-translated functions — the 5 that rustconv dropped when the API
//  credit balance ran out mid-run (faithful ports of actctx.c).
// ════════════════════════════════════════════════════════════════════

unsafe fn lookup_assembly(acl: *mut actctx_loader, ai: *mut assembly_identity) -> NTSTATUS {
    static dotDllW: [WCHAR; 5] = [b'.' as WCHAR, b'd' as WCHAR, b'l' as WCHAR, b'l' as WCHAR, 0];
    let mut i: u32;
    let buffer: *mut WCHAR;
    let mut p: *mut WCHAR;
    let directory: *mut WCHAR;
    let mut status: NTSTATUS;
    let mut nameW: UNICODE_STRING = core::mem::zeroed();
    let mut file: HANDLE = core::ptr::null_mut();
    let len: u32;

    status = lookup_winsxs(acl, ai);
    if status != STATUS_NO_SUCH_FILE {
        return status;
    }

    /* FIXME: add support for language specific lookup */

    len = core::cmp::max(
        RtlGetFullPathName_U(
            (*(*(*acl).actctx).assemblies).manifest.info,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) / core::mem::size_of::<WCHAR>() as u32,
        wcslen((*(*acl).actctx).appdir.info) as u32,
    );

    nameW.Buffer = core::ptr::null_mut();
    buffer = RtlAllocateHeap(
        GetProcessHeap(),
        0,
        (len as usize + 2 * wcslen((*ai).name) + 2) * core::mem::size_of::<WCHAR>()
            + core::mem::size_of_val(&dotManifestW),
    ) as *mut WCHAR;
    if buffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    directory = build_assembly_dir(ai);
    if directory.is_null() {
        RtlFreeHeap(GetProcessHeap(), 0, buffer as PVOID);
        return STATUS_NO_MEMORY;
    }

    /* Lookup in <dir>\name.dll, <dir>\name.manifest, <dir>\name\name.dll,
       <dir>\name\name.manifest — first appdir, then app manifest path. */
    wcscpy(buffer, (*(*acl).actctx).appdir.info);
    p = buffer.add(wcslen(buffer));
    i = 0;
    while i < 4 {
        if i == 2 {
            let assembly = (*(*acl).actctx).assemblies;
            if RtlGetFullPathName_U(
                (*assembly).manifest.info,
                len * core::mem::size_of::<WCHAR>() as u32,
                buffer,
                &mut p,
            ) == 0
            {
                break;
            }
        } else {
            *p = b'\\' as WCHAR;
            p = p.add(1);
        }

        wcscpy(p, (*ai).name);
        p = p.add(wcslen(p));

        wcscpy(p, dotDllW.as_ptr());
        if RtlDosPathNameToNtPathName_U(buffer, &mut nameW, core::ptr::null_mut(), core::ptr::null_mut()) != 0 {
            status = open_nt_file(&mut file, &mut nameW);
            if status == 0 {
                status = get_manifest_in_pe_file(
                    acl,
                    ai,
                    nameW.Buffer,
                    directory as LPCWSTR,
                    0,
                    file,
                    CREATEPROCESS_MANIFEST_RESOURCE_ID as LPCWSTR,
                    0,
                );
                NtClose(file);
                if status == STATUS_SUCCESS {
                    break;
                }
            }
            RtlFreeUnicodeString(&mut nameW);
        }

        wcscpy(p, dotManifestW.as_ptr());
        if RtlDosPathNameToNtPathName_U(buffer, &mut nameW, core::ptr::null_mut(), core::ptr::null_mut()) != 0 {
            status = open_nt_file(&mut file, &mut nameW);
            if status == 0 {
                status = get_manifest_in_manifest_file(acl, ai, nameW.Buffer, directory as LPCWSTR, 0, file);
                NtClose(file);
                break;
            }
            RtlFreeUnicodeString(&mut nameW);
        }
        status = STATUS_SXS_ASSEMBLY_NOT_FOUND;
        i += 1;
    }
    RtlFreeUnicodeString(&mut nameW);
    RtlFreeHeap(GetProcessHeap(), 0, directory as PVOID);
    RtlFreeHeap(GetProcessHeap(), 0, buffer as PVOID);
    status
}

unsafe fn parse_depend_manifests(acl: *mut actctx_loader) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut i: u32 = 0;
    while i < (*acl).num_dependencies {
        let dep = (*acl).dependencies.add(i as usize);
        if lookup_assembly(acl, dep) != STATUS_SUCCESS {
            if (*dep).optional == 0 && (*dep).delayed == 0 {
                status = STATUS_SXS_CANT_GEN_ACTCTX;
                break;
            }
        }
        i += 1;
    }
    /* FIXME should now iterate through all refs */
    status
}

/// RtlCreateActivationContext (NTDLL.@) — create an activation context.
#[no_mangle]
pub unsafe extern "system" fn RtlCreateActivationContext(
    _Flags: ULONG,
    ActivationContextData: PVOID,
    _ExtraBytes: ULONG,
    _NotificationRoutine: PVOID,
    _NotificationContext: PVOID,
    ActCtx: *mut PACTIVATION_CONTEXT,
) -> NTSTATUS {
    let pActCtx = ActivationContextData as *const ACTCTXW;
    let mut directory: *const WCHAR = core::ptr::null();
    let actctx: *mut ACTIVATION_CONTEXT;
    let mut nameW: UNICODE_STRING = core::mem::zeroed();
    let mut lang: ULONG = 0;
    let mut status: NTSTATUS;
    let mut file: HANDLE = core::ptr::null_mut();
    let mut acl: actctx_loader = core::mem::zeroed();

    if pActCtx.is_null()
        || (*pActCtx).cbSize < core::mem::size_of::<ACTCTXW>() as ULONG
        || ((*pActCtx).dwFlags & !ACTCTX_FLAGS_ALL) != 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    actctx = RtlAllocateHeap(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<ACTIVATION_CONTEXT>(),
    ) as *mut ACTIVATION_CONTEXT;
    if actctx.is_null() {
        return STATUS_NO_MEMORY;
    }

    // goto-error replacement: cleanup + return (matches the C `error:` label).
    macro_rules! goto_error {
        ($s:expr) => {{
            if !file.is_null() {
                NtClose(file);
            }
            actctx_release(actctx);
            return $s;
        }};
    }

    (*actctx).magic = ACTCTX_MAGIC;
    (*actctx).ref_count = 1;
    (*actctx).config.type_ = ACTIVATION_CONTEXT_PATH_TYPE_NONE;
    (*actctx).config.info = core::ptr::null_mut();
    (*actctx).appdir.type_ = ACTIVATION_CONTEXT_PATH_TYPE_WIN32_FILE;

    if (*pActCtx).dwFlags & ACTCTX_FLAG_APPLICATION_NAME_VALID != 0 {
        (*actctx).appdir.info = strdupW((*pActCtx).lpApplicationName);
        if (*actctx).appdir.info.is_null() {
            goto_error!(status_no_mem());
        }
    } else {
        let mut dir: UNICODE_STRING = core::mem::zeroed();
        let module: HMODULE = if (*pActCtx).dwFlags & ACTCTX_FLAG_HMODULE_VALID != 0 {
            (*pActCtx).hModule
        } else {
            (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ImageBaseAddress
        };

        status = get_module_filename(module, &mut dir, 0);
        if !NT_SUCCESS(status) {
            goto_error!(status);
        }
        let p = wcsrchr(dir.Buffer, b'\\' as core::ffi::c_int);
        if !p.is_null() {
            *p.add(1) = 0;
        }
        (*actctx).appdir.info = dir.Buffer;
    }

    nameW.Buffer = core::ptr::null_mut();

    /* open file only if it's going to be used */
    if !(*pActCtx).lpSource.is_null()
        && !((*pActCtx).dwFlags & ACTCTX_FLAG_RESOURCE_NAME_VALID != 0
            && (*pActCtx).dwFlags & ACTCTX_FLAG_HMODULE_VALID != 0)
    {
        let mut source: *mut WCHAR = core::ptr::null_mut();

        if (*pActCtx).dwFlags & ACTCTX_FLAG_ASSEMBLY_DIRECTORY_VALID != 0
            && RtlDetermineDosPathNameType_U((*pActCtx).lpSource) == RtlPathTypeRelative
        {
            let dir_len = wcslen((*pActCtx).lpAssemblyDirectory);
            let source_len = wcslen((*pActCtx).lpSource);
            source = RtlAllocateHeap(
                GetProcessHeap(),
                0,
                (dir_len + source_len + 2) * core::mem::size_of::<WCHAR>(),
            ) as *mut WCHAR;
            if source.is_null() {
                goto_error!(status_no_mem());
            }
            memcpy(
                source as PVOID,
                (*pActCtx).lpAssemblyDirectory as PVOID,
                dir_len * core::mem::size_of::<WCHAR>(),
            );
            *source.add(dir_len) = b'\\' as WCHAR;
            memcpy(
                source.add(dir_len + 1) as PVOID,
                (*pActCtx).lpSource as PVOID,
                (source_len + 1) * core::mem::size_of::<WCHAR>(),
            );
        }

        let ret = RtlDosPathNameToNtPathName_U(
            if !source.is_null() { source } else { (*pActCtx).lpSource },
            &mut nameW,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        RtlFreeHeap(GetProcessHeap(), 0, source as PVOID);
        if ret == 0 {
            goto_error!(STATUS_NO_SUCH_FILE);
        }
        status = open_nt_file(&mut file, &mut nameW);
        if !NT_SUCCESS(status) {
            RtlFreeUnicodeString(&mut nameW);
            goto_error!(status);
        }
    }

    acl.actctx = actctx;
    acl.dependencies = core::ptr::null_mut();
    acl.num_dependencies = 0;
    acl.allocated_dependencies = 0;

    if (*pActCtx).dwFlags & ACTCTX_FLAG_LANGID_VALID != 0 {
        lang = (*pActCtx).wLangId as ULONG;
    }
    if (*pActCtx).dwFlags & ACTCTX_FLAG_ASSEMBLY_DIRECTORY_VALID != 0 {
        directory = (*pActCtx).lpAssemblyDirectory;
    }

    if (*pActCtx).dwFlags & ACTCTX_FLAG_RESOURCE_NAME_VALID != 0 {
        /* if we have a resource it's a PE file */
        if (*pActCtx).dwFlags & ACTCTX_FLAG_HMODULE_VALID != 0 {
            status = get_manifest_in_module(
                &mut acl,
                core::ptr::null_mut(),
                core::ptr::null(),
                directory,
                0,
                (*pActCtx).hModule,
                (*pActCtx).lpResourceName,
                lang,
            );
            if status != 0 && status != STATUS_SXS_CANT_GEN_ACTCTX {
                status = get_manifest_in_associated_manifest(
                    &mut acl,
                    core::ptr::null_mut(),
                    core::ptr::null(),
                    directory,
                    (*pActCtx).hModule,
                    (*pActCtx).lpResourceName,
                );
            }
        } else if !(*pActCtx).lpSource.is_null() && !(*pActCtx).lpResourceName.is_null() {
            status = get_manifest_in_pe_file(
                &mut acl,
                core::ptr::null_mut(),
                nameW.Buffer,
                directory,
                0,
                file,
                (*pActCtx).lpResourceName,
                lang,
            );
            if status != 0 && status != STATUS_SXS_CANT_GEN_ACTCTX {
                status = get_manifest_in_associated_manifest(
                    &mut acl,
                    core::ptr::null_mut(),
                    nameW.Buffer,
                    directory,
                    core::ptr::null_mut(),
                    (*pActCtx).lpResourceName,
                );
            }
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    } else {
        status = get_manifest_in_manifest_file(&mut acl, core::ptr::null_mut(), nameW.Buffer, directory, 0, file);
    }

    if !file.is_null() {
        NtClose(file);
        file = core::ptr::null_mut();
    }
    RtlFreeUnicodeString(&mut nameW);

    if NT_SUCCESS(status) {
        status = parse_depend_manifests(&mut acl);
    }
    free_depend_manifests(&mut acl);

    if NT_SUCCESS(status) {
        // output pointer write — volatile to defeat captures(none) elision.
        core::ptr::write_volatile(ActCtx, actctx);
    } else {
        actctx_release(actctx);
    }
    status
}

// STATUS_NO_MEMORY helper (the value the C `error:` paths set before goto).
#[inline(always)]
fn status_no_mem() -> NTSTATUS {
    STATUS_NO_MEMORY
}

pub unsafe fn actctx_init() {
    let mut ctx: ACTCTXW = core::mem::zeroed();
    let mut handle: HANDLE = core::ptr::null_mut();

    ctx.cbSize = core::mem::size_of::<ACTCTXW>() as ULONG;
    ctx.lpSource = core::ptr::null();
    ctx.dwFlags = ACTCTX_FLAG_RESOURCE_NAME_VALID | ACTCTX_FLAG_HMODULE_VALID;
    ctx.hModule = (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ImageBaseAddress;
    ctx.lpResourceName = CREATEPROCESS_MANIFEST_RESOURCE_ID as LPCWSTR;

    if NT_SUCCESS(RtlCreateActivationContext(
        0,
        core::ptr::addr_of_mut!(ctx) as PVOID,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut handle as *mut HANDLE as *mut PACTIVATION_CONTEXT,
    )) {
        process_actctx = check_actctx(handle);
    }

    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ActivationContextData =
        (*process_actctx).ActivationContextData as *mut _;
}

#[no_mangle]
pub unsafe extern "system" fn RtlpInitializeActCtx(pOldShimData: *mut PVOID) -> NTSTATUS {
    static systemcompatW: [WCHAR; 44] = [
        0x005c, 0x0077, 0x0069, 0x006e, 0x0073, 0x0078, 0x0073, 0x005c, 0x006d, 0x0061, 0x006e,
        0x0069, 0x0066, 0x0065, 0x0073, 0x0074, 0x0073, 0x005c, 0x0073, 0x0079, 0x0073, 0x0074,
        0x0065, 0x006d, 0x0063, 0x006f, 0x006d, 0x0070, 0x0061, 0x0074, 0x0069, 0x0062, 0x006c,
        0x0065, 0x002e, 0x006d, 0x0061, 0x006e, 0x0069, 0x0066, 0x0065, 0x0073, 0x0074, 0x0000,
    ];
    let mut ctx: ACTCTXW = core::mem::zeroed();
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut buffer: [WCHAR; 1024] = [0; 1024];
    let status: NTSTATUS;

    actctx_init();

    /* ReactOS specific: now that we have process_actctx, init process compat */
    LdrpInitializeProcessCompat(process_actctx as PVOID, pOldShimData);

    ctx.cbSize = core::mem::size_of::<ACTCTXW>() as ULONG;
    ctx.dwFlags = 0;
    ctx.hModule = core::ptr::null_mut();
    ctx.lpResourceName = core::ptr::null();
    ctx.lpSource = buffer.as_ptr();
    RtlStringCchCopyW(buffer.as_mut_ptr(), buffer.len(), windows_dir());
    RtlStringCchCatW(buffer.as_mut_ptr(), buffer.len(), systemcompatW.as_ptr());

    status = RtlCreateActivationContext(
        0,
        core::ptr::addr_of_mut!(ctx) as PVOID,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut handle as *mut HANDLE as *mut PACTIVATION_CONTEXT,
    );
    if NT_SUCCESS(status) {
        implicit_actctx = check_actctx(handle);
    }

    status
}
