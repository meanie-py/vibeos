use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut IID_ITaskService: GUID = _GUID {
    Data1: 0x2faba4c7 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4da9 as ::core::ffi::c_ushort,
    Data3: 0x4013 as ::core::ffi::c_ushort,
    Data4: [
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut CLSID_TaskScheduler: GUID = _GUID {
    Data1: 0xf87369f as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xa4e5 as ::core::ffi::c_ushort,
    Data3: 0x4cfc as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x73 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x72 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut change_optW: [WCHAR; 8] = [
    '/' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut create_optW: [WCHAR; 8] = [
    '/' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut delete_optW: [WCHAR; 8] = [
    '/' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut enable_optW: [WCHAR; 8] = [
    '/' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut f_optW: [WCHAR; 3] = [
    '/' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut ru_optW: [WCHAR; 4] = [
    '/' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut tn_optW: [WCHAR; 4] = [
    '/' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut tr_optW: [WCHAR; 4] = [
    '/' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut xml_optW: [WCHAR; 5] = [
    '/' as i32 as WCHAR,
    'x' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

unsafe extern "C" fn get_tasks_root_folder() -> *mut ITaskFolder {
    let mut service: *mut ITaskService = core::ptr::null_mut::<ITaskService>();
    let mut root: *mut ITaskFolder = core::ptr::null_mut::<ITaskFolder>();
    let mut empty: VARIANT = core::mem::zeroed::<VARIANT>();
    let mut hres: HRESULT = 0;

    hres = CoCreateInstance(
        &raw const CLSID_TaskScheduler,
        (core::ptr::null_mut::<IUnknown>()).cast(),
        CLSCTX_INPROC_SERVER as DWORD,
        &raw const IID_ITaskService,
        &raw mut service as *mut LPVOID,
    );
    if hres < 0 as HRESULT {
        return core::ptr::null_mut::<ITaskFolder>();
    }

    V_VT!(&raw mut empty) = VT_EMPTY as VARTYPE;
    hres = ITaskService_Connect(service, empty, empty, empty, empty);
    if hres < 0 as HRESULT {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                (__dbcl) as u32,
                __dbch,
                b"base/applications/cmdutils/schtasks/schtasks.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"get_tasks_root_folder\0" as *const u8 as *const core::ffi::c_char,
                54 as core::ffi::c_int,
                b"Connect failed: %08x\n\0" as *const u8 as *const core::ffi::c_char,
                hres,
            );
        }
        return core::ptr::null_mut::<ITaskFolder>();
    }

    hres = ITaskService_GetFolder(service, core::ptr::null_mut::<OLECHAR>(), &raw mut root);
    ITaskService_Release(service);
    if hres < 0 as HRESULT {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                (__dbcl_0) as u32,
                __dbch_0,
                b"base/applications/cmdutils/schtasks/schtasks.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"get_tasks_root_folder\0" as *const u8 as *const core::ffi::c_char,
                61 as core::ffi::c_int,
                b"GetFolder failed: %08x\n\0" as *const u8 as *const core::ffi::c_char,
                hres,
            );
        }
        return core::ptr::null_mut::<ITaskFolder>();
    }

    return root;
}

unsafe extern "C" fn get_registered_task(name: *const WCHAR) -> *mut IRegisteredTask {
    let mut registered_task: *mut IRegisteredTask = core::ptr::null_mut();
    let mut root: *mut ITaskFolder = core::ptr::null_mut();
    let mut str: BSTR = core::ptr::null_mut();
    let mut hres: HRESULT;

    root = get_tasks_root_folder();
    if root.is_null() {
        return core::ptr::null_mut();
    }

    str = SysAllocString(name as *const OLECHAR);
    hres = ITaskFolder_GetTask(root, str, core::ptr::addr_of_mut!(registered_task));
    SysFreeString(str);
    ITaskFolder_Release(root);
    if hres < 0 {
        return core::ptr::null_mut();
    }

    return registered_task;
}

unsafe extern "C" fn read_file_to_bstr(file_name: *const WCHAR) -> BSTR {
    let mut file_size: LARGE_INTEGER = core::mem::zeroed();
    let mut read_size: DWORD = 0;
    let mut size: DWORD = 0;
    let mut data: *mut u8 = core::ptr::null_mut();
    let mut file: HANDLE = core::ptr::null_mut();
    let mut r: BOOL = FALSE as BOOL;
    let mut ret: BSTR = core::ptr::null_mut();

    file = CreateFileW(
        file_name,
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if file == INVALID_HANDLE_VALUE {
        DPRINT1!("Could not open file\n");
        return core::ptr::null_mut();
    }

    if GetFileSizeEx(file, core::ptr::addr_of_mut!(file_size)) == 0
        || file_size.QuadPart == 0
    {
        DPRINT1!("Empty file\n");
        CloseHandle(file);
        return core::ptr::null_mut();
    }

    data = HeapAlloc(
        GetProcessHeap(),
        0,
        file_size.QuadPart as SIZE_T,
    ) as *mut u8;
    if !data.is_null() {
        r = ReadFile(
            file,
            data as PVOID,
            file_size.QuadPart as DWORD,
            core::ptr::addr_of_mut!(read_size),
            core::ptr::null_mut(),
        );
    }
    CloseHandle(file);
    if r == 0 {
        DPRINT1!("Read filed\n");
        HeapFree(
            GetProcessHeap(),
            0,
            data as PVOID,
        );
        return core::ptr::null_mut();
    }

    if read_size > 2 as DWORD
        && *data.offset(0 as isize) as ::core::ffi::c_int
            == 0xff as ::core::ffi::c_int
        && *data.offset(1 as isize) as ::core::ffi::c_int
            == 0xfe as ::core::ffi::c_int
    {
        ret = SysAllocStringLen(
            data.offset(2 as isize) as *const WCHAR,
            (((read_size - 2 as DWORD) as usize)
                / ::core::mem::size_of::<WCHAR>()) as UINT,
        );
    } else {
        size = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            data as LPCSTR,
            read_size as ::core::ffi::c_int,
            core::ptr::null_mut(),
            0 as ::core::ffi::c_int,
        ) as DWORD;
        ret = SysAllocStringLen(core::ptr::null(), size as UINT);
        if !ret.is_null() {
            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                data as LPCSTR,
                read_size as ::core::ffi::c_int,
                ret as LPWSTR,
                size as ::core::ffi::c_int,
            );
        }
    }
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        data as PVOID,
    );

    return ret;
}

unsafe extern "C" fn change_command(mut argc: i32, mut argv: *mut *mut WCHAR) -> i32 {
    let mut have_option: BOOL = (FALSE as i32);
    let mut enable: BOOL = (FALSE as i32);
    let mut task_name: *const WCHAR = core::ptr::null();
    let mut task: *mut IRegisteredTask = core::ptr::null_mut();
    let mut hres: HRESULT = 0;

    while argc != 0 {
        if strcmpiW!(
            *argv.add(0 as usize),
            core::ptr::addr_of!(tn_optW) as *const WCHAR,
        ) == 0
        {
            if argc < 2 {
                return 1;
            }

            if !task_name.is_null() {
                return 1;
            }

            task_name = *argv.add(1 as usize);
            argc -= 2;
            argv = argv.add(2 as usize);
        } else if strcmpiW!(
            *argv.add(0 as usize),
            core::ptr::addr_of!(enable_optW) as *const WCHAR,
        ) == 0
        {
            enable = (TRUE as i32);
            have_option = (TRUE as i32);
            argc -= 1;
            argv = argv.add(1);
        } else if strcmpiW!(
            *argv.add(0 as usize),
            core::ptr::addr_of!(tr_optW) as *const WCHAR,
        ) == 0
        {
            if argc < 2 {
                return 1;
            }

            have_option = (TRUE as i32);
            argc -= 2;
            argv = argv.add(2 as usize);
        } else {
            return 1;
        }
    }

    if task_name.is_null() {
        return 1;
    }

    if have_option == 0 {
        return 1;
    }

    task = get_registered_task(task_name);
    if task.is_null() {
        return 1;
    }

    if enable != 0 {
        hres = IRegisteredTask_put_Enabled(task, VARIANT_TRUE as VARIANT_BOOL);
        if hres < 0 {
            IRegisteredTask_Release(task);
            return 1;
        }
    }

    IRegisteredTask_Release(task);
    return 0;
}

unsafe extern "C" fn create_command(mut argc: i32, mut argv: *mut *mut WCHAR) -> i32 {
    let mut task_name: *const WCHAR = core::ptr::null();
    let mut xml_file: *const WCHAR = core::ptr::null();
    let mut root: *mut ITaskFolder = core::ptr::null_mut();
    let mut flags: LONG = (TASK_CREATE) as i32;
    let mut task: *mut IRegisteredTask = core::ptr::null_mut();
    let mut empty: VARIANT = core::mem::zeroed();
    let mut str: BSTR;
    let mut xml: BSTR;
    let mut hres: HRESULT;

    while argc != 0 {
        if strcmpiW!(*argv.offset(0 as isize), xml_optW.as_ptr()) == 0 {
            if argc < 2 {
                return 1;
            }

            if !xml_file.is_null() {
                return 1;
            }

            xml_file = *argv.offset(1 as isize);
            argc -= 2;
            argv = argv.offset(2 as isize);
        } else if strcmpiW!(*argv.offset(0 as isize), tn_optW.as_ptr()) == 0 {
            if argc < 2 {
                return 1;
            }

            if !task_name.is_null() {
                return 1;
            }

            task_name = *argv.offset(1 as isize);
            argc -= 2;
            argv = argv.offset(2 as isize);
        } else if strcmpiW!(*argv.offset(0 as isize), f_optW.as_ptr()) == 0 {
            flags = (TASK_CREATE_OR_UPDATE) as i32;
            argc -= 1;
            argv = argv.offset(1);
        } else if strcmpiW!(*argv.offset(0 as isize), ru_optW.as_ptr()) == 0 {
            if argc < 2 {
                return 1;
            }

            argc -= 2;
            argv = argv.offset(2 as isize);
        } else {
            return 1;
        }
    }

    if task_name.is_null() {
        return 1;
    }

    if xml_file.is_null() {
        return E_FAIL;
    }

    xml = read_file_to_bstr(xml_file);
    if xml.is_null() {
        return 1;
    }

    root = get_tasks_root_folder();
    if root.is_null() {
        SysFreeString(xml);
        return 1;
    }

    V_VT!(core::ptr::addr_of_mut!(empty)) = VT_EMPTY;
    str = SysAllocString(task_name);
    hres = ITaskFolder_RegisterTask(
        root,
        str,
        xml,
        flags,
        empty,
        empty,
        TASK_LOGON_NONE,
        empty,
        core::ptr::addr_of_mut!(task),
    );
    SysFreeString(str);
    SysFreeString(xml);
    ITaskFolder_Release(root);
    if ((hres) < 0) {
        return 1;
    }

    IRegisteredTask_Release(task);
    return 0;
}

unsafe extern "C" fn delete_command(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut WCHAR,
) -> ::core::ffi::c_int {
    let mut task_name: *const WCHAR = core::ptr::null();
    let mut root: *mut ITaskFolder = core::ptr::null_mut();
    let mut str: BSTR = core::ptr::null_mut();
    let mut hres: HRESULT = 0;

    while argc != 0 {
        if strcmpiW!(
            *argv.offset(0 as isize),
            core::ptr::addr_of!(f_optW) as *const WCHAR,
        ) == 0
        {
            argc -= 1;
            argv = argv.offset(1);
        } else if strcmpiW!(
            *argv.offset(0 as isize),
            core::ptr::addr_of!(tn_optW) as *const WCHAR,
        ) == 0
        {
            if argc < 2 {
                return 1;
            }

            if !task_name.is_null() {
                return 1;
            }

            task_name = *argv.offset(1 as isize);
            argc -= 2;
            argv = argv.offset(2 as isize);
        } else {
            return 1;
        }
    }

    if task_name.is_null() {
        return 1;
    }

    root = get_tasks_root_folder();
    if root.is_null() {
        return 1;
    }

    str = SysAllocString(task_name as *const OLECHAR);
    hres = ITaskFolder_DeleteTask(root, str, 0);
    SysFreeString(str);
    ITaskFolder_Release(root);
    if hres < 0 {
        return 1;
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn wmain(mut argc: core::ffi::c_int, mut argv: *mut *mut WCHAR) -> core::ffi::c_int {
    let mut i: core::ffi::c_int = 0;
    let mut ret: core::ffi::c_int = 0;

    i = 0;
    while i < argc {
        TRACE!(" %s", wine_dbgstr_w(*argv.offset(i as isize)));
        i += 1;
    }

    CoInitialize(core::ptr::null_mut());

    if argc < 2 {
        FIXME!("Print current tasks state\n");
    } else if strcmpiW!(*argv.offset(1 as isize), core::ptr::addr_of!(change_optW) as *const WCHAR) == 0 {
        ret = change_command(argc - 2, argv.offset(2 as isize));
    } else if strcmpiW!(*argv.offset(1 as isize), core::ptr::addr_of!(create_optW) as *const WCHAR) == 0 {
        ret = create_command(argc - 2, argv.offset(2 as isize));
    } else if strcmpiW!(*argv.offset(1 as isize), core::ptr::addr_of!(delete_optW) as *const WCHAR) == 0 {
        ret = delete_command(argc - 2, argv.offset(2 as isize));
    } else {
        FIXME!("Unsupported command %s\n", debugstr_w(*argv.offset(1 as isize)));
    }

    CoUninitialize();
    return ret;
}
