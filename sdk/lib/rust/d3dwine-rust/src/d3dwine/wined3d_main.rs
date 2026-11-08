use crate::d3dwine::context_gl::context_get_tls_idx;
use crate::d3dwine::context_gl::context_set_tls_idx;
use crate::d3dwine::device::device_process_message;
use crate::d3dwine::context_gl::wined3d_context_gl_set_current;
use crate::d3dwine::swapchain::wined3d_swapchain_state_set_fullscreen;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wined3d_settings_tag {
    pub cs_multithreaded: ::core::ffi::c_uint,
    pub max_gl_version: ::core::ffi::c_uint,
    pub pci_vendor_id: ::core::ffi::c_ushort,
    pub pci_device_id: ::core::ffi::c_ushort,
    pub emulated_textureram: UINT64,
    pub logo: *mut ::core::ffi::c_char,
    pub multisample_textures: ::core::ffi::c_uint,
    pub sample_count: ::core::ffi::c_uint,
    pub strict_shader_math: ::core::ffi::c_uint,
    pub max_sm_vs: ::core::ffi::c_uint,
    pub max_sm_hs: ::core::ffi::c_uint,
    pub max_sm_ds: ::core::ffi::c_uint,
    pub max_sm_gs: ::core::ffi::c_uint,
    pub max_sm_ps: ::core::ffi::c_uint,
    pub max_sm_cs: ::core::ffi::c_uint,
    pub renderer: wined3d_renderer,
    pub shader_backend: wined3d_shader_backend,
    pub check_float_constants: bool,
    pub cb_access_map_w: bool,
    pub ffp_hlsl: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
    pub Flags: DWORD,
    pub CreatorBackTraceIndexHigh: WORD,
    pub SpareWORD: WORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_10 {
    pub WineDebugString: DWORD_PTR,
    pub Spare: [DWORD_PTR; 1],
    pub c2rust_unnamed: C2RustUnnamed_11,
}

pub const WINED3D_OPENGL_WINDOW_CLASS_NAME: [::core::ffi::c_char; 15] =
    unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"WineD3D_OpenGL\0") };

static mut wndproc_table: wined3d_wndproc_table = wined3d_wndproc_table {
    entries: ::core::ptr::null::<wined3d_wndproc>() as *mut wined3d_wndproc,
    count: 0,
    size: 0,
};

static mut swapchain_state_table: wined3d_swapchain_state_table = wined3d_swapchain_state_table {
    hooks: ::core::ptr::null::<wined3d_window_hook>() as *mut wined3d_window_hook,
    hooks_size: 0,
    hook_count: 0,
    states: ::core::ptr::null::<wined3d_registered_swapchain_state>()
        as *mut wined3d_registered_swapchain_state,
    states_size: 0,
    state_count: 0,
};

static mut wined3d_cs_debug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut wined3d_cs,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut wined3d_cs_debug.ProcessLocksList,
            Blink: &raw mut wined3d_cs_debug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    }};

static mut wined3d_cs: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const wined3d_cs_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as LONG),
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

static mut wined3d_wndproc_cs_debug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut wined3d_wndproc_cs,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut wined3d_wndproc_cs_debug.ProcessLocksList,
            Blink: &raw mut wined3d_wndproc_cs_debug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    }};

static mut wined3d_wndproc_cs: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const wined3d_wndproc_cs_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as LONG),
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

#[no_mangle]
pub static mut wined3d_command_cs: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const wined3d_command_cs_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as LONG),
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

static mut wined3d_command_cs_debug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut wined3d_command_cs,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut wined3d_command_cs_debug.ProcessLocksList,
            Blink: &raw mut wined3d_command_cs_debug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    }};

#[no_mangle]
pub static mut wined3d_settings: wined3d_settings_tag = wined3d_settings_tag {
    cs_multithreaded: WINED3D_CSMT_ENABLE as ::core::ffi::c_uint,
    max_gl_version: (4 as ::core::ffi::c_uint & 0xffff as ::core::ffi::c_uint)
        << 16 as ::core::ffi::c_int
        | 4 as ::core::ffi::c_uint & 0xffff as ::core::ffi::c_uint,
    pci_vendor_id: PCI_VENDOR_NONE as ::core::ffi::c_ushort,
    pci_device_id: PCI_DEVICE_NONE as ::core::ffi::c_ushort,
    emulated_textureram: 0,
    logo: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    multisample_textures: TRUE as ::core::ffi::c_uint,
    sample_count: !(0 as ::core::ffi::c_uint),
    strict_shader_math: 0,
    max_sm_vs: UINT_MAX,
    max_sm_hs: UINT_MAX,
    max_sm_ds: UINT_MAX,
    max_sm_gs: UINT_MAX,
    max_sm_ps: UINT_MAX,
    max_sm_cs: UINT_MAX,
    renderer: WINED3D_RENDERER_AUTO,
    shader_backend: WINED3D_SHADER_BACKEND_AUTO,
    check_float_constants: false,
    cb_access_map_w: false,
    ffp_hlsl: false,
};

// #define VKD3D_NO_VULKAN_H / VKD3D_NO_WIN32_TYPES / VK_NO_PROTOTYPES: empty flag macros, no Rust declaration.
// WINE_DEFAULT_DEBUG_CHANNEL(d3d), WINE_DECLARE_DEBUG_CHANNEL(vkd3d/winediag): function-like macros that
// expand to debug-channel statics; not emitted here.

#[repr(C)]
pub struct wined3d_wndproc {
    pub wined3d: *mut wined3d,
    pub window: HWND,
    pub unicode: BOOL,
    pub filter: BOOL,
    pub proc_: WNDPROC,
    pub device: *mut wined3d_device,
    pub flags: u32,
}

#[repr(C)]
pub struct wined3d_wndproc_table {
    pub entries: *mut wined3d_wndproc,
    pub count: SIZE_T,
    pub size: SIZE_T,
}

#[repr(C)]
pub struct wined3d_window_hook {
    pub hook: HHOOK,
    pub thread_id: DWORD,
    pub count: u32,
}

#[repr(C)]
pub struct wined3d_registered_swapchain_state {
    pub state: *mut wined3d_swapchain_state,
    pub thread_id: DWORD,
}

#[repr(C)]
pub struct wined3d_swapchain_state_table {
    pub hooks: *mut wined3d_window_hook,
    pub hooks_size: SIZE_T,
    pub hook_count: SIZE_T,

    pub states: *mut wined3d_registered_swapchain_state,
    pub states_size: SIZE_T,
    pub state_count: SIZE_T,
}

// static CRITICAL_SECTION_DEBUG wined3d_cs_debug = { 0, 0, &wined3d_cs, {&..ProcessLocksList, &..ProcessLocksList}, 0, 0, {(DWORD_PTR)(__FILE__ ": wined3d_cs")} };
// NOTE: the Spare[0] entry is (DWORD_PTR)"<file>: wined3d_cs" in C. A pointer->integer cast is not
// const-evaluable in a Rust static initializer, so it is zeroed here; it is only a debug string tag.

// static CRITICAL_SECTION_DEBUG wined3d_wndproc_cs_debug = { ... "wined3d_wndproc_cs" };
// Spare[0] zeroed for the same reason as above.

// static CRITICAL_SECTION_DEBUG wined3d_command_cs_debug = { ... "wined3d_command_cs" };
// Spare[0] zeroed for the same reason as above.

// CRITICAL_SECTION wined3d_command_cs; (tentative definition, completed by the initializer below)

// Constants used by wined3d_settings (values from Wine headers).
// MAKEDWORD_VERSION(4, 4) = ((4 << 16) | 4)
pub const WINED3D_MAX_GL_VERSION_4_4: u32 = (4u32 << 16) | 4u32;
#[no_mangle]
pub unsafe extern "C" fn wined3d_get_renderer() -> wined3d_renderer {
    if wined3d_settings.renderer as ::core::ffi::c_uint
        == WINED3D_RENDERER_AUTO as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return WINED3D_RENDERER_OPENGL;
    }

    return wined3d_settings.renderer;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_create(mut flags: u32) -> *mut wined3d {
    let mut object: *mut wined3d;
    let mut hr: HRESULT;

    object = calloc((1 as usize) as _, (FIELD_OFFSET!(wined3d, adapters[1]) as usize) as _) as *mut wined3d;
    if object.is_null() {
        ERR!("Failed to allocate wined3d object memory.\n");
        return core::ptr::null_mut();
    }

    if wined3d_settings.renderer == WINED3D_RENDERER_NO3D {
        flags |= WINED3D_NO3D;
    }

    hr = wined3d_init(object, flags);
    if hr < 0 {
        WARN!("Failed to initialize wined3d object, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    return object;
}

unsafe extern "C" fn is_option_separator(c: core::ffi::c_char) -> bool {
    return c as core::ffi::c_int == ',' as i32
        || c as core::ffi::c_int == ';' as i32
        || c as core::ffi::c_int == '\0' as i32;
}

unsafe extern "C" fn config_list_get_value(
    string: *const ::core::ffi::c_char,
    key: *const ::core::ffi::c_char,
    len: *mut size_t,
) -> *const ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut prev_char: ::core::ffi::c_char = 0;
    p = string;
    while !p.is_null() {
        p = strstr(p, key) as *const ::core::ffi::c_char;
        if !p.is_null() {
            prev_char = (if p > string {
                *p.offset(-(1) as isize) as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as ::core::ffi::c_char;
            p = p.offset(strlen(key) as isize);
            if is_option_separator(prev_char) && *p as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
                end = strpbrk(
                    p.offset(1 as isize),
                    b",;\0".as_ptr() as *const ::core::ffi::c_char,
                ) as *const ::core::ffi::c_char;
                if !end.is_null() {
                    *len = end.offset_from(p.offset(1 as isize))
                        as ::core::ffi::c_longlong as size_t;
                } else {
                    *len = strlen(p.offset(1 as isize)) as size_t;
                }
                return p.offset(1 as isize);
            }
        }
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}

unsafe extern "C" fn get_config_key(mut defkey: HKEY, mut appkey: HKEY, mut env: *const ::core::ffi::c_char, mut name: *const ::core::ffi::c_char, mut buffer: *mut ::core::ffi::c_char, mut size: DWORD) -> DWORD {
    let mut env_value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut env_len: size_t = 0;
    if {
        env_value = config_list_get_value(env, name, core::ptr::addr_of_mut!(env_len));
        !env_value.is_null()
    } && env_len < size as size_t {
        memcpy(buffer as *mut ::core::ffi::c_void, env_value as *const ::core::ffi::c_void, env_len);
        *buffer.add(env_len) = 0;
        return 0;
    }
    if !appkey.is_null()
        && RegQueryValueExA(
            appkey,
            name as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut::<DWORD>(),
            buffer as LPBYTE,
            core::ptr::addr_of_mut!(size),
        ) == 0
    {
        return 0;
    }
    if !defkey.is_null()
        && RegQueryValueExA(
            defkey,
            name as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut::<DWORD>(),
            buffer as LPBYTE,
            core::ptr::addr_of_mut!(size),
        ) == 0
    {
        return 0;
    }
    return ERROR_FILE_NOT_FOUND;
}

unsafe extern "C" fn get_config_key_dword(defkey: HKEY, appkey: HKEY, env: *const i8, name: *const i8, value: *mut u32) -> DWORD {
    let mut type_0: DWORD = 0;
    let mut data: DWORD = 0;
    let mut size: DWORD;
    let mut env_value: *const i8;
    let mut env_len: usize = 0;
    let mut end: *mut i8 = core::ptr::null_mut();

    env_value = config_list_get_value(env, name, core::ptr::addr_of_mut!(env_len));
    if !env_value.is_null() {
        *value = strtoul(env_value, core::ptr::addr_of_mut!(end), 0) as u32;
        if end as *const i8 != env_value {
            return 0 as DWORD;
        }
    }

    size = core::mem::size_of::<DWORD>() as DWORD;
    'success: {
        if !appkey.is_null()
            && RegQueryValueExA(
                appkey,
                name as LPCSTR,
                0 as *mut DWORD,
                core::ptr::addr_of_mut!(type_0),
                core::ptr::addr_of_mut!(data) as LPBYTE,
                core::ptr::addr_of_mut!(size),
            ) == 0
            && type_0 == REG_DWORD as DWORD
        {
            break 'success;
        }

        size = core::mem::size_of::<DWORD>() as DWORD;
        if !defkey.is_null()
            && RegQueryValueExA(
                defkey,
                name as LPCSTR,
                0 as *mut DWORD,
                core::ptr::addr_of_mut!(type_0),
                core::ptr::addr_of_mut!(data) as LPBYTE,
                core::ptr::addr_of_mut!(size),
            ) == 0
            && type_0 == REG_DWORD as DWORD
        {
            break 'success;
        }

        return ERROR_FILE_NOT_FOUND as DWORD;
    }

    *value = data as u32;
    return 0 as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_get_app_name(app_name: *mut ::core::ffi::c_char, app_name_size: ::core::ffi::c_uint) -> BOOL {
    let mut buffer: [core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut len: core::ffi::c_uint;
    let mut p: *mut core::ffi::c_char;
    let mut name: *mut core::ffi::c_char;
    len = GetModuleFileNameA(core::ptr::null_mut(), core::ptr::addr_of_mut!(buffer) as LPSTR, ARRAY_SIZE!(buffer) as DWORD) as core::ffi::c_uint;
    if !(len != 0 && len < (MAX_PATH) as core::ffi::c_uint) {
        return (FALSE) as BOOL;
    }
    name = core::ptr::addr_of_mut!(buffer) as *mut core::ffi::c_char;
    p = strrchr(name, '/' as core::ffi::c_int);
    if !p.is_null() {
        name = p.offset(1 as isize);
    }
    p = strrchr(name, '\\' as core::ffi::c_int);
    if !p.is_null() {
        name = p.offset(1 as isize);
    }
    len = (strlen(name) as core::ffi::c_uint).wrapping_add(1 as core::ffi::c_uint);
    if app_name_size < len {
        return (FALSE) as BOOL;
    }
    memcpy(app_name as *mut core::ffi::c_void, name as *const core::ffi::c_void, len as usize);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn vkd3d_log_callback(fmt: *const ::core::ffi::c_char, args: va_list) {
    let mut buffer: [::core::ffi::c_char; 1024] = [0; 1024];

    vsnprintf(
        core::ptr::addr_of_mut!(buffer) as *mut ::core::ffi::c_char,
        core::mem::size_of::<[::core::ffi::c_char; 1024]>() as usize,
        fmt,
        args,
    );
    __wine_dbg_output(core::ptr::addr_of_mut!(buffer) as *mut ::core::ffi::c_char);
}

unsafe extern "C" fn wined3d_dll_init(hInstDLL: HINSTANCE) -> BOOL {
    let mut wined3d_context_tls_idx: DWORD;
    let mut buffer: [::core::ffi::c_char; (MAX_PATH + 10) as _] =
        [0; (MAX_PATH + 10) as _];
    let mut size: DWORD = core::mem::size_of::<[::core::ffi::c_char; (MAX_PATH + 10) as _]>() as DWORD;
    let mut env: *const ::core::ffi::c_char;
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut appkey: HKEY = core::ptr::null_mut();
    let mut tmpvalue: ::core::ffi::c_uint = core::mem::zeroed();
    let mut wc: WNDCLASSA = core::mem::zeroed();

    wined3d_context_tls_idx = TlsAlloc();
    if wined3d_context_tls_idx == TLS_OUT_OF_INDEXES {
        let err: ::core::ffi::c_uint = GetLastError() as ::core::ffi::c_uint;
        ERR!("Failed to allocate context TLS index, err %#x.\n", err);
        return (FALSE) as BOOL;
    }
    context_set_tls_idx(wined3d_context_tls_idx);

    /* We need our own window class for a fake window which we use to retrieve GL capabilities */
    /* We might need CS_OWNDC in the future if we notice strange things on Windows.
     * Various articles/posts about OpenGL problems on Windows recommend this. */
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = Some(DefWindowProcA);
    wc.cbClsExtra = 0;
    wc.cbWndExtra = 0;
    wc.hInstance = hInstDLL;
    wc.hIcon = LoadIconA(core::ptr::null_mut(), IDI_WINLOGO as *const ::core::ffi::c_char);
    wc.hCursor = LoadCursorA(core::ptr::null_mut(), IDC_ARROW as *const ::core::ffi::c_char);
    wc.hbrBackground = core::ptr::null_mut();
    wc.lpszMenuName = core::ptr::null();
    wc.lpszClassName = (WINED3D_OPENGL_WINDOW_CLASS_NAME).as_ptr();

    if RegisterClassA(core::ptr::addr_of_mut!(wc)) == 0 {
        ERR!("Failed to register window class 'WineD3D_OpenGL'!\n");
        if TlsFree(wined3d_context_tls_idx) == 0 {
            let err: ::core::ffi::c_uint = GetLastError() as ::core::ffi::c_uint;
            ERR!("Failed to free context TLS index, err %#x.\n", err);
        }
        return (FALSE) as BOOL;
    }

    DisableThreadLibraryCalls(hInstDLL);

    /* @@ Wine registry key: HKCU\Software\Wine\Direct3D */
    if RegOpenKeyA(
        HKEY_CURRENT_USER,
        b"Software\\Wine\\Direct3D\0".as_ptr() as *const ::core::ffi::c_char,
        core::ptr::addr_of_mut!(hkey),
    ) != 0
    {
        hkey = core::ptr::null_mut();
    }

    if wined3d_get_app_name(buffer.as_mut_ptr(), ARRAY_SIZE!(buffer) as ::core::ffi::c_uint) != 0 {
        let mut tmpkey: HKEY = core::ptr::null_mut();
        /* @@ Wine registry key: HKCU\Software\Wine\AppDefaults\app.exe\Direct3D */
        if RegOpenKeyA(
            HKEY_CURRENT_USER,
            b"Software\\Wine\\AppDefaults\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(tmpkey),
        ) == 0
        {
            strcat(
                buffer.as_mut_ptr(),
                b"\\Direct3D\0".as_ptr() as *const ::core::ffi::c_char,
            );

            if RegOpenKeyA(tmpkey, buffer.as_mut_ptr(), core::ptr::addr_of_mut!(appkey)) != 0 {
                appkey = core::ptr::null_mut();
            }
            RegCloseKey(tmpkey);
        }
    }

    /* Allow modifying settings using the WINE_D3D_CONFIG environment variable,
     * which takes precedence over registry keys. An example is as follows:
     *
     *     WINE_D3D_CONFIG=csmt=0x1,shader_backend=glsl
     */
    env = getenv(b"WINE_D3D_CONFIG\0".as_ptr() as *const ::core::ffi::c_char);

    if !hkey.is_null() || !appkey.is_null() || !env.is_null() {
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"csmt\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(wined3d_settings.cs_multithreaded),
        ) == 0
        {
            WINE_ERR_!(__wine_dbch_winediag, "Setting multithreaded command stream to %#x.\n", wined3d_settings.cs_multithreaded);
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"MaxVersionGL\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(tmpvalue),
        ) == 0
        {
            WINE_ERR_!(__wine_dbch_winediag,
                "Setting maximum allowed wined3d GL version to %u.%u.\n",
                tmpvalue >> 16,
                tmpvalue & 0xffff
            );
            wined3d_settings.max_gl_version = tmpvalue;
        }
        if get_config_key(
            hkey,
            appkey,
            env,
            b"shader_backend\0".as_ptr() as *const ::core::ffi::c_char,
            buffer.as_mut_ptr(),
            size,
        ) == 0
        {
            if stricmp(buffer.as_mut_ptr(), b"glsl-vkd3d\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
                WINE_ERR_!(__wine_dbch_winediag, "Using the vkd3d-shader GLSL shader backend.\n");
                wined3d_settings.shader_backend = WINED3D_SHADER_BACKEND_GLSL_VKD3D;
            }
            if stricmp(buffer.as_mut_ptr(), b"glsl\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
                WINE_ERR_!(__wine_dbch_winediag, "Using the GLSL shader backend.\n");
                wined3d_settings.shader_backend = WINED3D_SHADER_BACKEND_GLSL;
            }
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"VideoPciDeviceID\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(tmpvalue),
        ) == 0
        {
            let pci_device_id: ::core::ffi::c_int = tmpvalue as ::core::ffi::c_int;

            /* A pci device id is 16-bit */
            if pci_device_id > 0xffff {
                ERR!("Invalid value for VideoPciDeviceID. The value should be smaller or equal to 65535 or 0xffff\n");
            } else {
                wined3d_settings.pci_device_id = pci_device_id as ::core::ffi::c_ushort;
            }
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"VideoPciVendorID\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(tmpvalue),
        ) == 0
        {
            let pci_vendor_id: ::core::ffi::c_int = tmpvalue as ::core::ffi::c_int;

            /* A pci device id is 16-bit */
            if pci_vendor_id > 0xffff {
                ERR!("Invalid value for VideoPciVendorID. The value should be smaller or equal to 65535 or 0xffff\n");
            } else {
                wined3d_settings.pci_vendor_id = pci_vendor_id as ::core::ffi::c_ushort;
            }
        }
        if get_config_key(
            hkey,
            appkey,
            env,
            b"VideoMemorySize\0".as_ptr() as *const ::core::ffi::c_char,
            buffer.as_mut_ptr(),
            size,
        ) == 0
        {
            let TmpVideoMemorySize: ::core::ffi::c_int = atoi(buffer.as_mut_ptr());
            if TmpVideoMemorySize > 0 {
                wined3d_settings.emulated_textureram = (TmpVideoMemorySize as UINT64)
                    * 1024
                    * 1024;
            } else {
                ERR!("VideoMemorySize is %i but must be >0\n", TmpVideoMemorySize);
            }
        }
        if get_config_key(
            hkey,
            appkey,
            env,
            b"WineLogo\0".as_ptr() as *const ::core::ffi::c_char,
            buffer.as_mut_ptr(),
            size,
        ) == 0
        {
            let len: size_t = strlen(buffer.as_mut_ptr()) as size_t + 1 as size_t;

            wined3d_settings.logo = malloc(len) as *mut ::core::ffi::c_char;
            if wined3d_settings.logo.is_null() {
                ERR!("Failed to allocate logo path memory.\n");
            } else {
                memcpy(
                    wined3d_settings.logo as *mut ::core::ffi::c_void,
                    buffer.as_mut_ptr() as *const ::core::ffi::c_void,
                    len,
                );
            }
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"MultisampleTextures\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(wined3d_settings.multisample_textures),
        ) == 0
        {
            WINE_ERR_!(__wine_dbch_winediag, "Setting multisample textures to %#x.\n", wined3d_settings.multisample_textures);
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"SampleCount\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(wined3d_settings.sample_count),
        ) == 0
        {
            WINE_ERR_!(__wine_dbch_winediag,
                "Forcing sample count to %u. This may not be compatible with all applications.\n",
                wined3d_settings.sample_count
            );
        }
        if get_config_key(
            hkey,
            appkey,
            env,
            b"CheckFloatConstants\0".as_ptr() as *const ::core::ffi::c_char,
            buffer.as_mut_ptr(),
            size,
        ) == 0
            && strcmp(buffer.as_mut_ptr(), b"enabled\0".as_ptr() as *const ::core::ffi::c_char) == 0
        {
            /* Checking relative addressing indices in float constants. */
            wined3d_settings.check_float_constants = TRUE != 0;
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"strict_shader_math\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(wined3d_settings.strict_shader_math),
        ) == 0
        {
            WINE_ERR_!(__wine_dbch_winediag, "Setting strict shader math to %#x.\n", wined3d_settings.strict_shader_math);
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"MaxShaderModelVS\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(wined3d_settings.max_sm_vs),
        ) == 0
        {
            TRACE!("Limiting VS shader model to %u.\n", wined3d_settings.max_sm_vs);
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"MaxShaderModelHS\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(wined3d_settings.max_sm_hs),
        ) == 0
        {
            TRACE!("Limiting HS shader model to %u.\n", wined3d_settings.max_sm_hs);
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"MaxShaderModelDS\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(wined3d_settings.max_sm_ds),
        ) == 0
        {
            TRACE!("Limiting DS shader model to %u.\n", wined3d_settings.max_sm_ds);
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"MaxShaderModelGS\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(wined3d_settings.max_sm_gs),
        ) == 0
        {
            TRACE!("Limiting GS shader model to %u.\n", wined3d_settings.max_sm_gs);
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"MaxShaderModelPS\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(wined3d_settings.max_sm_ps),
        ) == 0
        {
            TRACE!("Limiting PS shader model to %u.\n", wined3d_settings.max_sm_ps);
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"MaxShaderModelCS\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(wined3d_settings.max_sm_cs),
        ) == 0
        {
            TRACE!("Limiting CS shader model to %u.\n", wined3d_settings.max_sm_cs);
        }
        if get_config_key(
            hkey,
            appkey,
            env,
            b"renderer\0".as_ptr() as *const ::core::ffi::c_char,
            buffer.as_mut_ptr(),
            size,
        ) == 0
        {
            if strcmp(buffer.as_mut_ptr(), b"vulkan\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
                WINE_ERR_!(__wine_dbch_winediag, "Using the Vulkan renderer.\n");
                wined3d_settings.renderer = WINED3D_RENDERER_VULKAN;
            } else if strcmp(buffer.as_mut_ptr(), b"gl\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
                WINE_ERR_!(__wine_dbch_winediag, "Using the OpenGL renderer.\n");
                wined3d_settings.renderer = WINED3D_RENDERER_OPENGL;
            } else if strcmp(buffer.as_mut_ptr(), b"gdi\0".as_ptr() as *const ::core::ffi::c_char) == 0
                || strcmp(buffer.as_mut_ptr(), b"no3d\0".as_ptr() as *const ::core::ffi::c_char) == 0
            {
                WINE_ERR_!(__wine_dbch_winediag, "Disabling 3D support.\n");
                wined3d_settings.renderer = WINED3D_RENDERER_NO3D;
            }
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"cb_access_map_w\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(tmpvalue),
        ) == 0
            && tmpvalue != 0
        {
            /* Forcing all constant buffers to be write-mappable. */
            wined3d_settings.cb_access_map_w = TRUE != 0;
        }
        if get_config_key_dword(
            hkey,
            appkey,
            env,
            b"ffp_hlsl\0".as_ptr() as *const ::core::ffi::c_char,
            core::ptr::addr_of_mut!(tmpvalue),
        ) == 0
        {
            WINE_ERR_!(__wine_dbch_winediag, "Using the HLSL-based FFP backend.\n");
            wined3d_settings.ffp_hlsl = tmpvalue != 0;
        }
    }

    if !appkey.is_null() {
        RegCloseKey(appkey);
    }
    if !hkey.is_null() {
        RegCloseKey(hkey);
    }

    if getenv(b"VKD3D_DEBUG\0".as_ptr() as *const ::core::ffi::c_char).is_null() {
        if TRACE_ON!(__wine_dbch_vkd3d) != 0 {
            putenv(b"VKD3D_DEBUG=trace\0".as_ptr() as *const ::core::ffi::c_char);
        } else if WARN_ON!(__wine_dbch_vkd3d) != 0 {
            putenv(b"VKD3D_DEBUG=warn\0".as_ptr() as *const ::core::ffi::c_char);
        } else if FIXME_ON!(__wine_dbch_vkd3d) != 0 {
            putenv(b"VKD3D_DEBUG=fixme\0".as_ptr() as *const ::core::ffi::c_char);
        } else if ERR_ON!(__wine_dbch_vkd3d) != 0 {
            putenv(b"VKD3D_DEBUG=err\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            putenv(b"VKD3D_DEBUG=none\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
    if getenv(b"VKD3D_SHADER_DEBUG\0".as_ptr() as *const ::core::ffi::c_char).is_null() {
        if TRACE_ON!(__wine_dbch_vkd3d) != 0 {
            putenv(b"VKD3D_SHADER_DEBUG=trace\0".as_ptr() as *const ::core::ffi::c_char);
        } else if WARN_ON!(__wine_dbch_vkd3d) != 0 {
            putenv(b"VKD3D_SHADER_DEBUG=warn\0".as_ptr() as *const ::core::ffi::c_char);
        } else if FIXME_ON!(__wine_dbch_vkd3d) != 0 {
            putenv(b"VKD3D_SHADER_DEBUG=fixme\0".as_ptr() as *const ::core::ffi::c_char);
        } else if ERR_ON!(__wine_dbch_vkd3d) != 0 {
            putenv(b"VKD3D_SHADER_DEBUG=err\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            putenv(b"VKD3D_SHADER_DEBUG=none\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }

    vkd3d_set_log_callback(Some(vkd3d_log_callback));

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_dll_destroy(hInstDLL: HINSTANCE) -> BOOL {
    let wined3d_context_tls_idx: DWORD = context_get_tls_idx();
    let mut i: ::core::ffi::c_uint;

    if TlsFree(wined3d_context_tls_idx) == 0 {
        let err: ::core::ffi::c_uint = GetLastError() as ::core::ffi::c_uint;
        ERR!("Failed to free context TLS index, err %#x.\n", err);
    }

    i = 0;
    while (i as SIZE_T) < wndproc_table.count {
        /* Trying to unregister these would be futile. These entries can only
         * exist if either we skipped them in wined3d_unregister_window() due
         * to the application replacing the wndproc after the entry was
         * registered, or if the application still has an active wined3d
         * device. In the latter case the application has bigger problems than
         * these entries. */
        WARN!("Leftover wndproc table entry %p.\n", wndproc_table.entries.add(i as usize) as *mut wined3d_wndproc);
        i = i.wrapping_add(1);
    }
    free(wndproc_table.entries as *mut ::core::ffi::c_void);

    free(swapchain_state_table.states as *mut ::core::ffi::c_void);
    i = 0;
    while (i as SIZE_T) < swapchain_state_table.hook_count {
        WARN!("Leftover swapchain state hook %p.\n", swapchain_state_table.hooks.add(i as usize) as *mut wined3d_window_hook);
        UnhookWindowsHookEx((*swapchain_state_table.hooks.add(i as usize)).hook);
        i = i.wrapping_add(1);
    }
    free(swapchain_state_table.hooks as *mut ::core::ffi::c_void);

    free(wined3d_settings.logo as *mut ::core::ffi::c_void);
    UnregisterClassA((WINED3D_OPENGL_WINDOW_CLASS_NAME).as_ptr() as LPCSTR, hInstDLL);

    DeleteCriticalSection(core::ptr::addr_of_mut!(wined3d_command_cs));

    DeleteCriticalSection(core::ptr::addr_of_mut!(wined3d_wndproc_cs));
    DeleteCriticalSection(core::ptr::addr_of_mut!(wined3d_cs));
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn wined3d_mutex_lock() {
    EnterCriticalSection(core::ptr::addr_of_mut!(wined3d_cs));
}

#[no_mangle]
pub unsafe extern "system" fn wined3d_mutex_unlock() {
    LeaveCriticalSection(core::ptr::addr_of_mut!(wined3d_cs));
}

unsafe extern "C" fn wined3d_wndproc_mutex_lock() {
    EnterCriticalSection(core::ptr::addr_of_mut!(wined3d_wndproc_cs));
}

unsafe extern "C" fn wined3d_wndproc_mutex_unlock() {
    LeaveCriticalSection(core::ptr::addr_of_mut!(wined3d_wndproc_cs));
}

unsafe extern "C" fn wined3d_get_output_from_window(wined3d: *const wined3d, hwnd: HWND) -> *mut wined3d_output {
    let mut adapter_idx: ::core::ffi::c_uint = 0;
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut adapter: *mut wined3d_adapter = ::core::ptr::null_mut::<wined3d_adapter>();
    let mut monitor_info: MONITORINFOEXW = core::mem::zeroed();
    let mut monitor: HMONITOR = ::core::ptr::null_mut::<HMONITOR__>();

    TRACE!("wined3d %p, hwnd %p.\n", wined3d, hwnd);

    monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST as DWORD);
    monitor_info.cbSize = core::mem::size_of::<MONITORINFOEXW>() as DWORD;
    if GetMonitorInfoW(monitor, &raw mut monitor_info as LPMONITORINFO) == 0 {
        ERR!("GetMonitorInfoW failed, error %#lx.\n", GetLastError());
        return ::core::ptr::null_mut::<wined3d_output>();
    }

    adapter_idx = 0 as ::core::ffi::c_uint;
    while adapter_idx < (*wined3d).adapter_count {
        adapter = *(&raw const (*wined3d).adapters as *const *mut wined3d_adapter)
            .offset(adapter_idx as isize);
        output_idx = 0 as ::core::ffi::c_uint;
        while (output_idx as SIZE_T) < (*adapter).output_count {
            if lstrcmpiW(
                &raw mut (*(*adapter).outputs.offset(output_idx as isize)).device_name as *mut WCHAR
                    as LPCWSTR,
                &raw mut monitor_info.szDevice as *mut WCHAR as LPCWSTR,
            ) == 0
            {
                return (*adapter).outputs.offset(output_idx as isize) as *mut wined3d_output;
            }
            output_idx = output_idx.wrapping_add(1);
        }
        adapter_idx = adapter_idx.wrapping_add(1);
    }

    return ::core::ptr::null_mut::<wined3d_output>();
}

unsafe extern "C" fn wined3d_find_wndproc(window: HWND, wined3d: *mut wined3d) -> *mut wined3d_wndproc {
    let mut i: ::core::ffi::c_uint;
    i = 0 as ::core::ffi::c_uint;
    while (i as SIZE_T) < wndproc_table.count {
        let entry: *mut wined3d_wndproc = wndproc_table.entries.offset(i as isize) as *mut wined3d_wndproc;
        if (*entry).window == window && (*entry).wined3d == wined3d {
            return entry;
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_filter_messages(window: HWND, filter: BOOL) -> BOOL {
    let mut entry: *mut wined3d_wndproc;
    let mut ret: BOOL;

    wined3d_wndproc_mutex_lock();

    entry = wined3d_find_wndproc(window, core::ptr::null_mut());
    if entry.is_null() {
        wined3d_wndproc_mutex_unlock();
        return (FALSE) as BOOL;
    }

    ret = (*entry).filter;
    (*entry).filter = filter;

    wined3d_wndproc_mutex_unlock();

    return ret;
}

unsafe extern "system" fn wined3d_wndproc(
    mut window: HWND,
    mut message: UINT,
    mut wparam: WPARAM,
    mut lparam: LPARAM,
) -> LRESULT {
    let mut entry: *mut wined3d_wndproc = core::ptr::null_mut();
    let mut device: *mut wined3d_device = core::ptr::null_mut();
    let mut unicode: BOOL = 0;
    let mut filter: BOOL = 0;
    let mut proc_0: WNDPROC = None;

    wined3d_wndproc_mutex_lock();

    entry = wined3d_find_wndproc(window, core::ptr::null_mut());
    if entry.is_null() {
        wined3d_wndproc_mutex_unlock();
        ERR!("Window %p is not registered with wined3d.\n", window);
        return DefWindowProcW(window, message, wparam, lparam);
    }

    device = (*entry).device;
    unicode = (*entry).unicode;
    filter = (*entry).filter;
    proc_0 = (*entry).proc_;
    wined3d_wndproc_mutex_unlock();

    if !device.is_null() {
        if filter != 0 && message != WM_DISPLAYCHANGE {
            TRACE!(
                "Filtering message: window %p, message %#x, wparam %#Ix, lparam %#Ix.\n",
                window,
                message,
                wparam,
                lparam
            );

            if unicode != 0 {
                return DefWindowProcW(window, message, wparam, lparam);
            }
            return DefWindowProcA(window, message, wparam, lparam);
        }

        return device_process_message(device, window, unicode, message, wparam, lparam, proc_0);
    }
    if unicode != 0 {
        return CallWindowProcW(proc_0, window, message, wparam, lparam);
    }
    return CallWindowProcA(proc_0, window, message, wparam, lparam);
}

unsafe extern "system" fn wined3d_hook_proc(mut code: c_int, mut wparam: WPARAM, mut lparam: LPARAM) -> LRESULT {
    let mut swapchain_desc: wined3d_swapchain_desc = core::mem::zeroed();
    let mut state: *mut wined3d_swapchain_state = core::ptr::null_mut();
    let mut entry: *mut wined3d_wndproc = core::ptr::null_mut();
    let mut output: *mut wined3d_output = core::ptr::null_mut();
    let mut msg: *mut MSG = lparam as *mut MSG;
    let mut i: c_uint = 0;

    /* Handle Alt+Enter. */
    if code == (HC_ACTION as c_int)
        && (*msg).message == WM_SYSKEYDOWN
        && (*msg).wParam == VK_RETURN as WPARAM
        && ((*msg).lParam & ((KF_ALTDOWN << 16) as LPARAM)) != 0
    {
        wined3d_wndproc_mutex_lock();

        i = 0;
        while (i as SIZE_T) < swapchain_state_table.state_count {
            state = (*swapchain_state_table.states.offset(i as isize)).state;

            if !((*state).device_window != (*msg).hwnd) {
                entry = wined3d_find_wndproc((*msg).hwnd, (*state).wined3d);
                if !(!entry.is_null()
                    && ((*entry).flags
                        & ((WINED3D_REGISTER_WINDOW_NO_WINDOW_CHANGES | WINED3D_REGISTER_WINDOW_NO_ALT_ENTER) as uint32_t))
                        != 0)
                {
                    swapchain_desc = (*state).desc;
                    swapchain_desc.windowed =
                        (swapchain_desc.windowed == 0) as ::core::ffi::c_int as BOOL;
                    output = wined3d_get_output_from_window((*state).wined3d, (*state).device_window);
                    if output.is_null() {
                        ERR!("Failed to get output from window %p.\n", (*state).device_window);
                        break;
                    } else {
                        swapchain_desc.output = output;
                        wined3d_swapchain_state_set_fullscreen(
                            state,
                            core::ptr::addr_of_mut!(swapchain_desc),
                            core::ptr::null::<wined3d_display_mode>(),
                        );

                        wined3d_wndproc_mutex_unlock();

                        return 1;
                    }
                }
            }
            i = i.wrapping_add(1);
        }

        wined3d_wndproc_mutex_unlock();
    }

    return CallNextHookEx(core::ptr::null_mut(), code, wparam, lparam);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_register_window(
    mut wined3d: *mut wined3d,
    mut window: HWND,
    mut device: *mut wined3d_device,
    mut flags: ::core::ffi::c_uint,
) -> BOOL {
    let mut entry: *mut wined3d_wndproc = ::core::ptr::null_mut::<wined3d_wndproc>();

    wined3d_wndproc_mutex_lock();

    entry = wined3d_find_wndproc(window, wined3d);
    if !entry.is_null() {
        if wined3d.is_null() {
            WARN!("Window %p is already registered with wined3d.\n", window);
        }
        (*entry).flags = flags as uint32_t;
        wined3d_wndproc_mutex_unlock();
        return (TRUE) as BOOL;
    }

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!(wndproc_table.entries) as *mut *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(wndproc_table.size),
        wndproc_table.count + 1 as SIZE_T,
        ::core::mem::size_of::<wined3d_wndproc>() as SIZE_T,
    ) == 0
    {
        wined3d_wndproc_mutex_unlock();
        ERR!("Failed to grow table.\n");
        return (FALSE) as BOOL;
    }

    let fresh15 = wndproc_table.count;
    wndproc_table.count = wndproc_table.count + 1;
    entry = wndproc_table.entries.offset(fresh15 as isize);
    (*entry).window = window;
    (*entry).unicode = IsWindowUnicode(window);
    if wined3d.is_null() {
        if (*entry).unicode != 0 {
            (*entry).proc_ = ::core::mem::transmute::<LONG_PTR, WNDPROC>(
                SetWindowLongPtrW(
                    window,
                    GWLP_WNDPROC,
                    (wined3d_wndproc as usize) as LONG_PTR,
                ),
            );
        } else {
            (*entry).proc_ = ::core::mem::transmute::<LONG_PTR, WNDPROC>(
                SetWindowLongPtrA(
                    window,
                    GWLP_WNDPROC,
                    ((wined3d_wndproc as usize) as LONG_PTR) as _,
                ),
            );
        }
    } else {
        (*entry).proc_ = None;
    }
    (*entry).device = device;
    (*entry).wined3d = wined3d;
    (*entry).flags = flags as uint32_t;

    wined3d_wndproc_mutex_unlock();

    return (TRUE) as BOOL;
}

unsafe extern "C" fn restore_wndproc(entry: *mut wined3d_wndproc) -> BOOL {
    let mut proc: LONG_PTR = 0;

    if (*entry).unicode != 0 {
        proc = GetWindowLongPtrW((*entry).window, GWLP_WNDPROC);
        if proc != (wined3d_wndproc as usize as LONG_PTR) {
            return (FALSE) as BOOL;
        }
        SetWindowLongPtrW(
            (*entry).window,
            GWLP_WNDPROC,
            ::core::mem::transmute::<WNDPROC, LONG_PTR>((*entry).proc_),
        );
    } else {
        proc = GetWindowLongPtrA((*entry).window, GWLP_WNDPROC);
        if proc != (wined3d_wndproc as usize as LONG_PTR) {
            return (FALSE) as BOOL;
        }
        SetWindowLongPtrA(
            (*entry).window,
            GWLP_WNDPROC,
            (::core::mem::transmute::<WNDPROC, LONG_PTR>((*entry).proc_)) as _,
        );
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unregister_window(window: HWND) {
    let mut entry: *mut wined3d_wndproc = core::ptr::null_mut();
    let mut last: *mut wined3d_wndproc = core::ptr::null_mut();

    wined3d_wndproc_mutex_lock();

    entry = wined3d_find_wndproc(window, core::ptr::null_mut());
    if entry.is_null() {
        wined3d_wndproc_mutex_unlock();
        ERR!("Window %p is not registered with wined3d.\n", window);
        return;
    }

    if (*entry).proc_.is_some() && restore_wndproc(entry) == 0 {
        (*entry).device = core::ptr::null_mut();
        WARN!("Not unregistering window %p, current window proc doesn't match wined3d window proc.\n", window);
        wined3d_wndproc_mutex_unlock();
        return;
    }

    wndproc_table.count = wndproc_table.count.wrapping_sub(1);
    last = wndproc_table.entries.offset(wndproc_table.count as isize) as *mut wined3d_wndproc;
    if entry != last {
        *entry = core::ptr::read(last);
    }

    wined3d_wndproc_mutex_unlock();
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_unregister_windows(wined3d: *mut wined3d) {
    let mut entry: *mut wined3d_wndproc = core::ptr::null_mut();
    let mut last: *mut wined3d_wndproc = core::ptr::null_mut();
    let mut i: core::ffi::c_uint = 0;

    TRACE!("wined3d %p.\n", wined3d);

    wined3d_wndproc_mutex_lock();

    while (i as SIZE_T) < wndproc_table.count {
        entry = wndproc_table.entries.offset(i as isize) as *mut wined3d_wndproc;

        if (*entry).wined3d != wined3d {
            i = i.wrapping_add(1);
            continue;
        }

        if (*entry).proc_.is_some() && restore_wndproc(entry) == 0 {
            (*entry).device = core::ptr::null_mut();
            WARN!(
                "Not unregistering window %p, current window proc doesn't match wined3d window proc.\n",
                (*entry).window
            );
            i = i.wrapping_add(1);
            continue;
        }

        wndproc_table.count = wndproc_table.count.wrapping_sub(1);
        last = wndproc_table.entries.offset(wndproc_table.count as isize) as *mut wined3d_wndproc;
        if entry != last {
            *entry = core::ptr::read(last);
        } else {
            i = i.wrapping_add(1);
        }
    }

    wined3d_wndproc_mutex_unlock();
}

unsafe extern "C" fn wined3d_find_hook(mut thread_id: DWORD) -> *mut wined3d_window_hook {
    let mut i: ::core::ffi::c_uint = 0;
    while (i as SIZE_T) < swapchain_state_table.hook_count {
        if (*swapchain_state_table.hooks.add(i as usize)).thread_id == thread_id {
            return swapchain_state_table.hooks.add(i as usize);
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_state_register(state: *mut wined3d_swapchain_state) {
    let mut state_entry: *mut wined3d_registered_swapchain_state = core::ptr::null_mut();
    let mut hook: *mut wined3d_window_hook = core::ptr::null_mut();

    wined3d_wndproc_mutex_lock();

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!(swapchain_state_table.states) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(swapchain_state_table.states_size),
        swapchain_state_table.state_count + 1,
        core::mem::size_of::<wined3d_registered_swapchain_state>() as SIZE_T,
    ) == 0
    {
        wined3d_wndproc_mutex_unlock();
        return;
    }

    state_entry = swapchain_state_table
        .states
        .offset(swapchain_state_table.state_count as isize);
    swapchain_state_table.state_count += 1;
    (*state_entry).state = state;
    (*state_entry).thread_id = GetWindowThreadProcessId((*state).device_window, core::ptr::null_mut());

    hook = wined3d_find_hook((*state_entry).thread_id);
    if !hook.is_null() {
        (*hook).count += 1;
        wined3d_wndproc_mutex_unlock();
        return;
    }

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!(swapchain_state_table.hooks) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(swapchain_state_table.hooks_size),
        swapchain_state_table.hook_count + 1,
        core::mem::size_of::<wined3d_window_hook>() as SIZE_T,
    ) == 0
    {
        swapchain_state_table.state_count -= 1;
        wined3d_wndproc_mutex_unlock();
        return;
    }

    hook = swapchain_state_table
        .hooks
        .offset(swapchain_state_table.hook_count as isize);
    swapchain_state_table.hook_count += 1;
    (*hook).thread_id = (*state_entry).thread_id;
    (*hook).hook = SetWindowsHookExW(
        (WH_GETMESSAGE as i32),
        Some(wined3d_hook_proc),
        core::ptr::null_mut(),
        (*hook).thread_id,
    );
    (*hook).count = 1;

    wined3d_wndproc_mutex_unlock();
}

unsafe extern "C" fn wined3d_swapchain_state_unregister(mut state: *mut wined3d_swapchain_state) {
    let mut state_entry: *mut wined3d_registered_swapchain_state =
        ::core::ptr::null_mut::<wined3d_registered_swapchain_state>();
    let mut last_state_entry: *mut wined3d_registered_swapchain_state =
        ::core::ptr::null_mut::<wined3d_registered_swapchain_state>();
    let mut hook: *mut wined3d_window_hook = ::core::ptr::null_mut::<wined3d_window_hook>();
    let mut last_hook: *mut wined3d_window_hook = ::core::ptr::null_mut::<wined3d_window_hook>();
    let mut i: ::core::ffi::c_uint = 0;

    wined3d_wndproc_mutex_lock();

    i = 0 as ::core::ffi::c_uint;
    while (i as SIZE_T) < swapchain_state_table.state_count {
        state_entry = swapchain_state_table.states.offset(i as isize)
            as *mut wined3d_registered_swapchain_state;

        if (*state_entry).state != state {
            i = i.wrapping_add(1);
            continue;
        }

        hook = wined3d_find_hook((*state_entry).thread_id);
        if !hook.is_null() && {
            (*hook).count = (*hook).count.wrapping_sub(1);
            (*hook).count == 0
        } {
            UnhookWindowsHookEx((*hook).hook);
            swapchain_state_table.hook_count = swapchain_state_table.hook_count.wrapping_sub(1);
            last_hook = swapchain_state_table
                .hooks
                .offset(swapchain_state_table.hook_count as isize)
                as *mut wined3d_window_hook;
            if hook != last_hook {
                *hook = ::core::ptr::read(last_hook);
            }
        }

        swapchain_state_table.state_count = swapchain_state_table.state_count.wrapping_sub(1);
        last_state_entry = swapchain_state_table
            .states
            .offset(swapchain_state_table.state_count as isize)
            as *mut wined3d_registered_swapchain_state;
        if state_entry != last_state_entry {
            *state_entry = ::core::ptr::read(last_state_entry);
        }

        break;
    }

    wined3d_wndproc_mutex_unlock();
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_state_cleanup(state: *mut wined3d_swapchain_state) {
    wined3d_swapchain_state_unregister(state);
}
#[no_mangle]
pub unsafe extern "system" fn DllMain(inst: HINSTANCE, reason: DWORD, reserved: *mut core::ffi::c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            return wined3d_dll_init(inst);
        }
        DLL_PROCESS_DETACH => {
            if reserved.is_null() {
                return wined3d_dll_destroy(inst);
            }
        }
        DLL_THREAD_DETACH => {
            if wined3d_context_gl_set_current(core::ptr::null_mut()) == 0 {
                ERR!("Failed to clear current context.\n");
            }
            return (TRUE) as BOOL;
        }
        _ => {}
    }
    return (TRUE) as BOOL;
}
