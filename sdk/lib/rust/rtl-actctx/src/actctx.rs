use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ACTIVATION_CONTEXT_STACK, FILE_BOTH_DIR_INFORMATION, FILE_END_OF_FILE_INFORMATION, FileBothDirectoryInformation, FileEndOfFileInformation, GUID, IMAGE_RESOURCE_DATA_ENTRY, IMAGE_RESOURCE_DIRECTORY, IMAGE_RESOURCE_DIRECTORY_ENTRY, _IO_STATUS_BLOCK, LARGE_INTEGER, LDR_DATA_TABLE_ENTRY, LDR_RESOURCE_INFO, LPCWSTR, LPWSTR, LdrAccessResource, LdrFindResource_U, LdrLockLoaderLock, LdrUnlockLoaderLock, NtClose, NtCurrentTeb, NtGetTickCount, NtOpenFile, NtQueryDirectoryFile, NtQueryInformationFile, NtReadFile, _OBJECT_ATTRIBUTES, PACTIVATION_CONTEXT, PACTIVATION_CONTEXT_STACK, RtlAllocateHeap, RtlCompareUnicodeStrings, RtlCreateUnicodeString, RtlDetermineDosPathNameType_U, RtlDosPathNameToNtPathName_U, RtlEqualUnicodeString, RtlFindActivationContextSectionGuid, RtlFreeHeap, RtlFreeUnicodeString, RtlGUIDFromString, RtlGetFullPathName_U, RtlHashUnicodeString, RtlImageDirectoryEntryToData, RtlImageNtHeader, RtlInitUnicodeString, RtlIsTextUnicode, RtlSizeHeap, RtlUnicodeStringToInteger, RtlUniform, RtlUpcaseUnicodeString, RtlUshortByteSwap, TEB};
use core::ffi::c_void;

extern "C" {
    fn wcslen(s: *const WCHAR) -> usize;
}

unsafe fn strdupW(str: *const WCHAR) -> *mut WCHAR {
    let ptr = RtlAllocateHeap(GetProcessHeap(), 0, (wcslen(str) + 1) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
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
    (wcsncmp((*xmlstr).ptr, str, (*xmlstr).len as usize) == 0
        && *str.add((*xmlstr).len as usize) == 0) as BOOL
}

unsafe fn xmlstr_cmpi(xmlstr: *const xmlstr_t, str: *const WCHAR) -> BOOL {
    (((wcsnicmp((*xmlstr).ptr, str, (*xmlstr).len as usize) == 0)
        && (*str.add((*xmlstr).len as usize) == 0)) as BOOL)
}

unsafe fn xml_attr_cmp(attr: *const xml_attr, str: *const WCHAR) -> BOOL {
    xmlstr_cmp(core::ptr::addr_of!((*attr).name), str)
}

unsafe fn xml_name_cmp(elem1: *const xml_elem, elem2: *const xml_elem) -> BOOL {
    ((*elem1).name.len == (*elem2).name.len
        && (*elem1).ns.len == (*elem2).ns.len
        && wcsncmp((*elem1).name.ptr, (*elem2).name.ptr, (*elem1).name.len as usize) == 0
        && wcsncmp((*elem1).ns.ptr, (*elem2).ns.ptr, (*elem1).ns.len as usize) == 0) as BOOL
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

unsafe fn debugstr_xml_elem(elem: *const xml_elem) -> *const core::ffi::c_char {
    wine_dbg_sprintf(
        b"%s ns %s\0".as_ptr() as *const core::ffi::c_char,
        debugstr_wn((*elem).name.ptr, (*elem).name.len),
        debugstr_wn((*elem).ns.ptr, (*elem).ns.len),
    )
}

unsafe fn debugstr_xml_attr(attr: *const xml_attr) -> *const core::ffi::c_char {
    wine_dbg_sprintf(
        b"%s=%s\0".as_ptr() as *const core::ffi::c_char,
        debugstr_wn((*attr).name.ptr, (*attr).name.len),
        debugstr_wn((*attr).value.ptr, (*attr).value.len),
    )
}

unsafe fn debugstr_version(ver: *const assembly_version) -> *const core::ffi::c_char {
    wine_dbg_sprintf(
        b"%u.%u.%u.%u\0".as_ptr() as *const core::ffi::c_char,
        (*ver).major as core::ffi::c_uint,
        (*ver).minor as core::ffi::c_uint,
        (*ver).build as core::ffi::c_uint,
        (*ver).revision as core::ffi::c_uint,
    )
}

unsafe fn get_module_filename(module: HMODULE, str: *mut _UNICODE_STRING, extra_len: core::ffi::c_uint) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut magic: ULONG_PTR = 0;
    let mut pldr: *mut LDR_DATA_TABLE_ENTRY = core::ptr::null_mut();

    LdrLockLoaderLock(0, core::ptr::null_mut(), &mut magic);
    status = LdrFindEntryForAddress(module, &mut pldr);
    if status == STATUS_SUCCESS {
        (*str).Buffer = RtlAllocateHeap(
            GetProcessHeap(),
            0,
            ((*pldr).FullDllName.Length as usize)
                + extra_len as usize
                + core::mem::size_of::<u16>(),
        ) as *mut u16;
        if !(*str).Buffer.is_null() {
            memcpy(
                (*str).Buffer as *mut core::ffi::c_void,
                (*pldr).FullDllName.Buffer as *const core::ffi::c_void,
                (*pldr).FullDllName.Length as usize + core::mem::size_of::<u16>(),
            );
            (*str).Length = (*pldr).FullDllName.Length;
            (*str).MaximumLength =
                (*pldr).FullDllName.Length + extra_len as u16 + core::mem::size_of::<u16>() as u16;
        } else {
            status = STATUS_NO_MEMORY;
        }
    }
    LdrUnlockLoaderLock(0, magic);
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
                new_count as usize * core::mem::size_of::<assembly>(),
            );
        } else {
            new_count = 4;
            ptr = RtlAllocateHeap(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                new_count as usize * core::mem::size_of::<assembly>(),
            );
        }
        if ptr.is_null() {
            return core::ptr::null_mut();
        }
        (*actctx).assemblies = ptr as *mut assembly;
        (*actctx).allocated_assemblies = new_count;
    }

    let idx = (*actctx).num_assemblies;
    (*actctx).num_assemblies += 1;
    assembly = (*actctx).assemblies.add(idx as usize);
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
                (new_count as usize) * core::mem::size_of::<dll_redirect>(),
            );
        } else {
            new_count = 4;
            ptr = RtlAllocateHeap(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (new_count as usize) * core::mem::size_of::<dll_redirect>(),
            );
        }
        if ptr.is_null() {
            return core::ptr::null_mut();
        }
        (*assembly).dlls = ptr as *mut dll_redirect;
        (*assembly).allocated_dlls = new_count;
    }
    let index = (*assembly).num_dlls;
    (*assembly).num_dlls += 1;
    (*assembly).dlls.add(index as usize)
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
    RtlFreeHeap(GetProcessHeap(), 0, (*ai).name as *mut core::ffi::c_void);
    RtlFreeHeap(GetProcessHeap(), 0, (*ai).arch as *mut core::ffi::c_void);
    RtlFreeHeap(GetProcessHeap(), 0, (*ai).public_key as *mut core::ffi::c_void);
    RtlFreeHeap(GetProcessHeap(), 0, (*ai).language as *mut core::ffi::c_void);
    RtlFreeHeap(GetProcessHeap(), 0, (*ai).type_ as *mut core::ffi::c_void);
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
        let entity = (*array).base.add(i as usize);
        match (*entity).kind {
            ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION => {
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.comclass.clsid as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.comclass.tlbid as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.comclass.progid as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.comclass.name as *mut _);
                RtlFreeHeap(GetProcessHeap(), 0, (*entity).u.comclass.version as *mut _);
                let mut j: core::ffi::c_uint = 0;
                while j < (*entity).u.comclass.progids.num {
                    RtlFreeHeap(GetProcessHeap(), 0, *(*entity).u.comclass.progids.progids.add(j as usize) as *mut _);
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
                FIXME(b"Unknown entity kind %d\n\0".as_ptr() as *const _, (*entity).kind);
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
        && RtlCompareUnicodeStrings(str1, wcslen(str1), str2, wcslen(str2), 1u8) == 0
    {
        1
    } else {
        0
    }
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

    let ret = RtlAllocateHeap(GetProcessHeap(), 0, size) as *mut WCHAR;
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
        (*ai).version.major, (*ai).version.minor, (*ai).version.build, (*ai).version.revision);
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

unsafe fn set_error(xmlbuf: *mut xmlbuf_t) -> BOOL {
    (*xmlbuf).error = 1;
    0
}

unsafe fn is_xmlns_attr(attr: *const xml_attr) -> BOOL {
    let len = wcslen(xmlnsW.as_ptr()) as i32;
    if ((*attr).name.len as i32) < len {
        return 0;
    }
    if wcsncmp((*attr).name.ptr, xmlnsW.as_ptr(), len as usize) != 0 {
        return 0;
    }
    if (*attr).name.len as i32 == len || *(*attr).name.ptr.offset(len as isize) == ':' as u16 {
        1
    } else {
        0
    }
}

unsafe fn find_xmlns(xmlbuf: *mut xmlbuf_t, name: *const xmlstr_t) -> xmlstr_t {
    let mut i: core::ffi::c_int = (*xmlbuf).ns_pos - 1;
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

unsafe fn read_xml_elem(xmlbuf: *mut xmlbuf_t, elem: *mut xml_elem) {
    let mut ptr = (*xmlbuf).ptr;

    (*elem).ns = empty_xmlstr;
    (*elem).name.ptr = ptr;
    while ptr < (*xmlbuf).end
        && isxmlspace(*ptr) == 0
        && *ptr != b'>' as WCHAR
        && *ptr != b'/' as WCHAR
    {
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

unsafe fn parse_xml_header(xmlbuf: *mut xmlbuf_t) -> BOOL {
    let mut ptr = (*xmlbuf).ptr;
    while ptr < (*xmlbuf).end.offset(-1) {
        if *ptr.offset(0) == b'?' as WCHAR && *ptr.offset(1) == b'>' as WCHAR {
            (*xmlbuf).ptr = ptr.offset(2);
            return 1;
        }
        ptr = ptr.offset(1);
    }
    0
}

unsafe fn parse_com_class_threadingmodel(value: *mut xmlstr_t) -> comclass_threadingmodel {
    let apartW: [WCHAR; 10] = [b'A' as WCHAR, b'p' as WCHAR, b'a' as WCHAR, b'r' as WCHAR, b't' as WCHAR, b'm' as WCHAR, b'e' as WCHAR, b'n' as WCHAR, b't' as WCHAR, 0];
    let neutralW: [WCHAR; 8] = [b'N' as WCHAR, b'e' as WCHAR, b'u' as WCHAR, b't' as WCHAR, b'r' as WCHAR, b'a' as WCHAR, b'l' as WCHAR, 0];
    let freeW: [WCHAR; 5] = [b'F' as WCHAR, b'r' as WCHAR, b'e' as WCHAR, b'e' as WCHAR, 0];
    let bothW: [WCHAR; 5] = [b'B' as WCHAR, b'o' as WCHAR, b't' as WCHAR, b'h' as WCHAR, 0];

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
    let mut min: core::ffi::c_int;
    let mut max: core::ffi::c_int;

    min = 0;
    max = (olemisc_values.len() as core::ffi::c_int) - 1;

    while min <= max {
        let n: core::ffi::c_int = (min + max) / 2;

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

    WARN(b"unknown flag %s\n\0".as_ptr() as *const i8, debugstr_wn(str, len));
    0
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

    *(*progids).progids.add((*progids).num as usize) = xmlstrdupW(progid);
    if (*(*progids).progids.add((*progids).num as usize)).is_null() {
        return 0;
    }
    (*progids).num += 1;

    1
}

unsafe fn aligned_string_len(len: i32) -> i32 {
    (len + 3) & !3
}

unsafe fn get_assembly_version(assembly: *mut assembly, mut ret: *mut WCHAR) -> i32 {
    static fmtW: [WCHAR; 10] = [
        '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR,
        '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR,
    ];
    let fmtW_full: [WCHAR; 9] = [
        '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR,
        '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR, 0,
    ];
    let _ = fmtW;
    let _ = fmtW_full;
    let fmt: [WCHAR; 10] = [
        '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR,
        '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR,
    ];
    let _ = fmt;
    let fmtW_arr: [WCHAR; 12] = [
        '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR,
        '.' as WCHAR, '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '%' as WCHAR,
        'u' as WCHAR, 0,
    ];
    let ver: *mut assembly_version = core::ptr::addr_of_mut!((*assembly).id.version);
    let mut buff: [WCHAR; 25] = [0; 25];

    if ret.is_null() {
        ret = buff.as_mut_ptr();
    }
    swprintf(
        ret,
        buff.len(),
        fmtW_arr.as_ptr(),
        (*ver).major as core::ffi::c_uint,
        (*ver).minor as core::ffi::c_uint,
        (*ver).build as core::ffi::c_uint,
        (*ver).revision as core::ffi::c_uint,
    )
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

unsafe fn find_first_id_entry(
    dir: *mut IMAGE_RESOURCE_DIRECTORY,
    root: *mut core::ffi::c_void,
    want_dir: core::ffi::c_int,
) -> *mut IMAGE_RESOURCE_DIRECTORY {
    let entry = dir.add(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut pos = (*dir).NumberOfNamedEntries as core::ffi::c_int;
    let end = (*dir).NumberOfNamedEntries as core::ffi::c_int
        + (*dir).NumberOfIdEntries as core::ffi::c_int;
    while pos < end {
        let e = &*entry.add(pos as usize);
        if ((e.DataIsDirectory() == 0) as core::ffi::c_int) == ((want_dir == 0) as core::ffi::c_int) {
            return (root as *mut core::ffi::c_char).add(e.OffsetToDirectory() as usize)
                as *mut IMAGE_RESOURCE_DIRECTORY;
        }
        pos += 1;
    }
    core::ptr::null_mut()
}

unsafe fn lookup_manifest_file(dir: HANDLE, ai: *mut assembly_identity) -> *mut WCHAR {
    static lookup_fmtW: [WCHAR; 37] = [
        '%' as WCHAR, 's' as WCHAR, '_' as WCHAR, '%' as WCHAR, 's' as WCHAR, '_' as WCHAR,
        '%' as WCHAR, 's' as WCHAR, '_' as WCHAR, '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR,
        '%' as WCHAR, 'u' as WCHAR, '.' as WCHAR, '*' as WCHAR, '.' as WCHAR, '*' as WCHAR,
        '_' as WCHAR, '%' as WCHAR, 's' as WCHAR, '_' as WCHAR, '*' as WCHAR, '.' as WCHAR,
        'm' as WCHAR, 'a' as WCHAR, 'n' as WCHAR, 'i' as WCHAR, 'f' as WCHAR, 'e' as WCHAR,
        's' as WCHAR, 't' as WCHAR, 0,
        0, 0, 0, 0,
    ];
    static wine_trailerW: [WCHAR; 17] = [
        'd' as WCHAR, 'e' as WCHAR, 'a' as WCHAR, 'd' as WCHAR, 'b' as WCHAR, 'e' as WCHAR,
        'e' as WCHAR, 'f' as WCHAR, '.' as WCHAR, 'm' as WCHAR, 'a' as WCHAR, 'n' as WCHAR,
        'i' as WCHAR, 'f' as WCHAR, 'e' as WCHAR, 's' as WCHAR, 't' as WCHAR,
    ];

    let mut lookup: *mut WCHAR;
    let mut ret: *mut WCHAR = core::ptr::null_mut();
    let mut lookup_us: _UNICODE_STRING = core::mem::zeroed();
    let mut io: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut lang: *const WCHAR = (*ai).language;
    let mut data_pos: u32 = 0;
    let mut data_len: u32;
    let len: u32;
    let mut buffer: [u8; 8192] = [0; 8192];

    if lang.is_null() || wcsicmp(lang, neutralW.as_ptr()) == 0 {
        lang = wildcardW.as_ptr();
    }

    len = (wcslen((*ai).arch) + wcslen((*ai).name) + wcslen((*ai).public_key)
        + wcslen(lang) + 20 + 37) as u32;

    lookup = RtlAllocateHeap(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
    if lookup.is_null() {
        return core::ptr::null_mut();
    }

    swprintf(lookup, len as usize, lookup_fmtW.as_ptr(), (*ai).arch, (*ai).name, (*ai).public_key,
             (*ai).version.major as u32, (*ai).version.minor as u32, lang);
    RtlInitUnicodeString(&mut lookup_us, lookup);

    if NtQueryDirectoryFile(dir, core::ptr::null_mut(), None, core::ptr::null_mut(), &mut io,
                            buffer.as_mut_ptr() as *mut _, buffer.len() as u32,
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
                                        buffer.as_mut_ptr() as *mut _, buffer.len() as u32,
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

            let star = wcschr(lookup, '*' as i32);
            tmp = ((*dir_info).FileName.as_mut_ptr()).offset(star.offset_from(lookup));
            build = wcstoul(tmp, core::ptr::null_mut(), 10);
            if build < min_build {
                continue;
            }
            tmp = wcschr(tmp, '.' as i32).add(1);
            revision = wcstoul(tmp, core::ptr::null_mut(), 10);
            if build == min_build && revision < min_revision {
                continue;
            }
            tmp = wcschr(tmp, '_' as i32).add(1);
            tmp = wcschr(tmp, '_' as i32).add(1);

            if (*dir_info).FileNameLength as usize
                - (tmp.offset_from((*dir_info).FileName.as_mut_ptr()) as usize) * core::mem::size_of::<WCHAR>()
                == core::mem::size_of_val(&wine_trailerW)
                && wcsnicmp(tmp, wine_trailerW.as_ptr(), 17) == 0
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
            RtlFreeHeap(GetProcessHeap(), 0, ret as *mut _);
            ret = RtlAllocateHeap(GetProcessHeap(), 0,
                (*dir_info).FileNameLength as usize + core::mem::size_of::<WCHAR>()) as *mut WCHAR;
            if !ret.is_null() {
                core::ptr::copy_nonoverlapping(
                    (*dir_info).FileName.as_ptr() as *const u8,
                    ret as *mut u8,
                    (*dir_info).FileNameLength as usize);
                *ret.add((*dir_info).FileNameLength as usize / core::mem::size_of::<WCHAR>()) = 0;
            }
        }
    }

    RtlFreeHeap(GetProcessHeap(), 0, lookup as *mut _);
    ret
}

unsafe fn find_query_actctx( handle: *mut HANDLE, flags: DWORD, class: ULONG ) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;

    if (flags & RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT) != 0 {
        if !(*handle).is_null() { return STATUS_INVALID_PARAMETER; }

        if !(*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame.is_null() {
            *handle = (*(*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame).ActivationContext;
        }
    }
    else if (flags & (RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_ADDRESS | RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_HMODULE)) != 0 {
        let mut magic: ULONG_PTR = 0;
        let mut pldr: *mut LDR_DATA_TABLE_ENTRY = core::ptr::null_mut();

        if (*handle).is_null() { return STATUS_INVALID_PARAMETER; }

        LdrLockLoaderLock( 0, core::ptr::null_mut(), &mut magic );
        if LdrFindEntryForAddress( *handle, &mut pldr ) == 0 {
            if (flags & RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_HMODULE) != 0 && *handle != (*pldr).DllBase {
                status = STATUS_DLL_NOT_FOUND;
            } else {
                *handle = (*pldr).EntryPointActivationContext;
            }
        } else {
            status = STATUS_DLL_NOT_FOUND;
        }
        LdrUnlockLoaderLock( 0, magic );
    }
    else if (*handle).is_null() && (class != ActivationContextBasicInformation) {
        *handle = process_actctx as HANDLE;
    }

    status
}

unsafe fn find_string_index(section: *const strsection_header, name: *const _UNICODE_STRING) -> *mut string_index {
    let mut index: *mut string_index = core::ptr::null_mut();
    let mut str: _UNICODE_STRING = core::mem::zeroed();
    let mut hash: ULONG = 0;
    let mut i: ULONG;

    RtlHashUnicodeString(name, 1u8, HASH_STRING_ALGORITHM_X65599, &mut hash);
    let mut iter = ((section as *const u8).add((*section).index_offset as usize)) as *mut string_index;

    i = 0;
    while i < (*section).count {
        if (*iter).hash == hash {
            str.Buffer = (section as *const u8).add((*iter).name_offset as usize) as *mut WCHAR;
            str.Length = (*iter).name_len as USHORT;
            if RtlEqualUnicodeString(&str, name, 1u8) != 0 {
                index = iter;
                break;
            }
        }
        iter = iter.add(1);
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

unsafe fn get_wndclass_first_index(actctx: *mut ACTIVATION_CONTEXT) -> *mut string_index {
    ((*actctx).wndclass_section as *mut u8).add((*(*actctx).wndclass_section).index_offset as usize) as *mut string_index
}

unsafe fn get_wndclass_data(ctxt: *mut ACTIVATION_CONTEXT, index: *mut string_index) -> *mut wndclass_redirect_data {
    ((*ctxt).wndclass_section as *mut u8).add((*index).data_offset as usize) as *mut wndclass_redirect_data
}

unsafe fn get_tlib_data(actctx: *mut ACTIVATION_CONTEXT, index: *mut guid_index) -> *mut tlibredirect_data {
    ((*actctx).tlib_section as *mut u8).add((*index).data_offset as usize) as *mut tlibredirect_data
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

unsafe fn get_comclass_data(actctx: *mut ACTIVATION_CONTEXT, index: *mut guid_index) -> *mut comclassredirect_data {
    ((*actctx).comserver_section as *mut u8).offset((*index).data_offset as isize) as *mut comclassredirect_data
}

unsafe fn get_ifaceps_data(actctx: *mut ACTIVATION_CONTEXT, index: *mut guid_index) -> *mut ifacepsredirect_data {
    ((*actctx).ifaceps_section as *mut u8).add((*index).data_offset as usize) as *mut ifacepsredirect_data
}

unsafe fn get_surrogate_data(actctx: *mut ACTIVATION_CONTEXT, index: *const guid_index) -> *mut clrsurrogate_data {
    ((*actctx).clrsurrogate_section as *mut u8).offset((*index).data_offset as isize) as *mut clrsurrogate_data
}

unsafe fn get_progid_data(actctx: *mut ACTIVATION_CONTEXT, index: *const string_index) -> *mut progidredirect_data {
    ((*actctx).progid_section as *mut u8).offset((*index).data_offset as isize) as *mut progidredirect_data
}

unsafe fn find_app_settings(actctx: *mut ACTIVATION_CONTEXT, settings: *const WCHAR, ns: *const WCHAR) -> *const WCHAR {
    let mut i: libc::c_uint = 0;
    while i < (*actctx).num_assemblies {
        let assembly = (*actctx).assemblies.add(i as usize);
        let mut j: libc::c_uint = 0;
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
pub unsafe extern "system" fn RtlZombifyActivationContext(handle: HANDLE) -> NTSTATUS {
    FIXME(b"%p: stub\n\0".as_ptr() as *const i8, handle);

    if handle == 0xf00baa as HANDLE {
        return STATUS_SUCCESS;
    }

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsActivationContextActive(handle: HANDLE) -> BOOLEAN {
    let mut frame = (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame;
    while !frame.is_null() {
        if (*frame).ActivationContext == handle {
            return 1u8;
        }
        frame = (*frame).Previous;
    }
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpFindActivationContextSection_CheckParameters(
    flags: ULONG,
    guid: *const GUID,
    section_kind: ULONG,
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
pub unsafe extern "system" fn RtlAllocateActivationContextStack(Stack: *mut PACTIVATION_CONTEXT_STACK) -> NTSTATUS {
    let ContextStack: PACTIVATION_CONTEXT_STACK;

    if !(*Stack).is_null() {
        return STATUS_SUCCESS;
    }

    ContextStack = RtlAllocateHeap(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<ACTIVATION_CONTEXT_STACK>() as _) as PACTIVATION_CONTEXT_STACK;
    if ContextStack.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*ContextStack).Flags = 0;
    (*ContextStack).ActiveFrame = core::ptr::null_mut();
    InitializeListHead(core::ptr::addr_of_mut!((*ContextStack).FrameListCache));
    (*ContextStack).NextCookieSequenceNumber = 1;
    (*ContextStack).StackId = 1;

    *Stack = ContextStack;

    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn RtlActivateActivationContextUnsafeFast(
    Frame: PRTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
    Context: PVOID,
) -> PRTL_ACTIVATION_CONTEXT_STACK_FRAME {
    let NewFrame: *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME;
    let ActiveFrame: *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME;

    ActiveFrame = (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame;

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
    (*Frame).Frame.ActivationContext = Context;
    (*Frame).Frame.Flags = RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_ACTIVATED;

    if (!ActiveFrame.is_null() && (*ActiveFrame).ActivationContext != Context)
        || !Context.is_null()
    {
        (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame = NewFrame;
        return NewFrame;
    }

    (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame = NewFrame;

    NewFrame
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn RtlActivateActivationContextUnsafeFast(
    Frame: PRTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
    Context: PVOID,
) -> PRTL_ACTIVATION_CONTEXT_STACK_FRAME {
    let NewFrame: *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME;
    let ActiveFrame: *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME;

    ActiveFrame = (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame;

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
    (*Frame).Frame.ActivationContext = Context;
    (*Frame).Frame.Flags = RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_ACTIVATED;

    if (!ActiveFrame.is_null() && (*ActiveFrame).ActivationContext != Context)
        || !Context.is_null()
    {
        (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame = NewFrame;
        return NewFrame;
    }

    (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame = NewFrame;

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

    ActiveFrame = (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame;

    if ActiveFrame.is_null() {
        // TODO: Perform some additional checks here
    }

    /* Special handling for not-really-activated */
    if ((*Frame).Frame.Flags & RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_NOT_REALLY_ACTIVATED) != 0 {
        (*Frame).Frame.Flags |= RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_DEACTIVATED;
        return core::ptr::addr_of_mut!((*Frame).Frame);
    }

    /* find the right frame */
    NewFrame = core::ptr::addr_of_mut!((*Frame).Frame);

    /* Pop everything up to and including frame */
    (*(*NtCurrentTeb()).ActivationContextStackPointer).ActiveFrame = (*NewFrame).Previous;

    (*Frame).Frame.Flags |= RTL_ACTIVATION_CONTEXT_STACK_FRAME_FLAG_DEACTIVATED;
    (*NewFrame).Previous
}
