use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut icdload_cs: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: -(1 as ::core::ffi::c_int) as LONG,
    RecursionCount: 0 as LONG,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0 as ULONG_PTR,
};

static mut ICD_Data_List: *mut ICD_Data = ::core::ptr::null::<ICD_Data>() as *mut ICD_Data;

static mut OpenGLDrivers_Key: [WCHAR; 59] = unsafe {
    ::core::mem::transmute::<
        [u8; 118],
        [WCHAR; 59],
    >(
        *b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0 \0N\0T\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0O\0p\0e\0n\0G\0L\0D\0r\0i\0v\0e\0r\0s\0\0\0",
    )
};

static mut CustomDrivers_Key: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0R\0e\0a\0c\0t\0O\0S\0\\\0O\0p\0e\0n\0G\0L\0\0\0",
    )
};

static mut CustomDrvInfo: Drv_Opengl_Info = Drv_Opengl_Info {
    Version: 0,
    DriverVersion: 0,
    DriverName: [0; 261],
};

static mut CustomDriverState: CUSTOM_DRIVER_STATE = OGL_CD_NOT_QUERIED;

#[repr(C)]
pub struct Drv_Opengl_Info {
    pub Version: ULONG,
    pub DriverVersion: ULONG,
    pub DriverName: [WCHAR; MAX_PATH as usize + 1],
}
pub type pDrv_Opengl_Info = *mut Drv_Opengl_Info;

pub const OPENGL_GETINFO_DRVNAME: i32 = 0;

pub type CUSTOM_DRIVER_STATE = i32;
pub const OGL_CD_NOT_QUERIED: i32 = 0;
pub const OGL_CD_NONE: i32 = 1;
pub const OGL_CD_ROSSWI: i32 = 2;
pub const OGL_CD_CUSTOM_ICD: i32 = 3;
unsafe extern "system" fn wglSetCurrentValue(value: PVOID) {
    IntSetCurrentICDPrivate(value);
}
unsafe extern "system" fn wglGetCurrentValue() -> PVOID {
    IntGetCurrentICDPrivate()
}
unsafe extern "system" fn wglGetDHGLRC(context: *mut wgl_context) -> DHGLRC {
    (*context).dhglrc
}
#[no_mangle]
pub unsafe extern "C" fn IntDeleteAllICDs() {
    let mut data: *mut ICD_Data = core::ptr::null_mut::<ICD_Data>();

    EnterCriticalSection(core::ptr::addr_of_mut!(icdload_cs));

    while !ICD_Data_List.is_null() {
        data = ICD_Data_List;
        ICD_Data_List = (*data).next;

        FreeLibrary((*data).hModule);
        HeapFree(GetProcessHeap(), 0, data as PVOID);
    }
}

// IntGetIcdData — hand-port of dll/opengl/opengl32/icdload.c:62-359.
// Paste into sdk/lib/rust/opengl32-rust/src/opengl32/icdload.rs.
//
// Hand-written because rustconv's model degenerated on it (max_tokens twice, and it sealed the
// local `DriverVersion` away as a fact token). The DRV_LOAD(x) macro is expanded below as 16
// explicit loads, in the C's order.
//
// Assumed from the module's c2rust file-scope preamble (the same one IntDeleteAllICDs uses):
// `icdload_cs`, `ICD_Data_List`, `OpenGLDrivers_Key`, `CustomDrivers_Key`, `CustomDrvInfo`,
// `CustomDriverState`, and the module fns `wglSetCurrentValue` / `wglGetCurrentValue` /
// `wglGetDHGLRC`. Win32/CRT externs (RegOpenKeyExW, RegQueryValueExW, RegCloseKey, ExtEscape,
// _wcsicmp, wcslen, wcscpy, LoadLibraryW, GetProcAddress, FreeLibrary, HeapAlloc, HeapFree,
// GetProcessHeap, Enter/LeaveCriticalSection, ros_dbg_log), HKEY_CURRENT_USER /
// HKEY_LOCAL_MACHINE and the __WINE_DBCL_* / __wine_dbch___default debug-channel items come from
// the prelude (`rustconv port resolve`), as in every other module.

// icdload.c:56-59 — the C declares these win32k (gdi32) entry points itself.
extern "system" {
    fn GdiDescribePixelFormat(
        hdc: HDC,
        ipfd: INT,
        cjpfd: UINT,
        ppfd: PPIXELFORMATDESCRIPTOR,
    ) -> INT;
    fn GdiSetPixelFormat(hdc: HDC, ipfd: INT) -> BOOL;
    fn GdiSwapBuffers(hdc: HDC) -> BOOL;
}
// ---- end helper items ----

/* Retrieves the ICD data (driver version + relevant DLL entry points) for a device context */
#[no_mangle]
pub unsafe extern "C" fn IntGetIcdData(mut hdc: HDC) -> *mut ICD_Data {
    let mut ret: ::core::ffi::c_int = 0;
    let mut dwInput: DWORD = 0;
    let mut dwValueType: DWORD = 0;
    let mut Version: DWORD = 0;
    let mut DriverVersion: DWORD = 0;
    let mut Flags: DWORD = 0;
    let mut DrvInfo: Drv_Opengl_Info = core::mem::zeroed();
    let mut pDrvInfo: pDrv_Opengl_Info = core::ptr::null_mut();
    let mut data: *mut ICD_Data = core::ptr::null_mut();
    let mut OglKey: HKEY = core::ptr::null_mut();
    let mut DrvKey: HKEY = core::ptr::null_mut();
    let mut CustomKey: HKEY = core::ptr::null_mut();
    let mut DllName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut DrvValidateVersion: Option<unsafe extern "system" fn(DWORD) -> BOOL> = None;
    let mut DrvSetCallbackProcs: Option<unsafe extern "system" fn(::core::ffi::c_int, *mut PROC)> =
        None;

    /* The following code is ReactOS specific and allows us to easily load an arbitrary ICD:
     * It checks HKCU\Software\ReactOS\OpenGL for a custom ICD and will always load it
     * no matter what driver the DC is associated with. It can also force using the
     * built-in Software Implementation*/
    if CustomDriverState == OGL_CD_NOT_QUERIED {
        /* Only do this once so there's not any significant performance penalty */
        CustomDriverState = OGL_CD_NONE;
        memset(
            core::ptr::addr_of_mut!(CustomDrvInfo) as *mut ::core::ffi::c_void,
            0,
            core::mem::size_of::<Drv_Opengl_Info>(),
        );

        'custom_end: {
            ret = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                core::ptr::addr_of!(CustomDrivers_Key) as LPCWSTR,
                0,
                KEY_READ as REGSAM,
                core::ptr::addr_of_mut!(CustomKey),
            ) as ::core::ffi::c_int;
            if ret != ERROR_SUCCESS as ::core::ffi::c_int {
                break 'custom_end;
            }

            dwInput =
                core::mem::size_of_val(&*core::ptr::addr_of!(CustomDrvInfo.DriverName)) as DWORD;
            ret = RegQueryValueExW(
                CustomKey,
                wide_bytes!(b"\0\0"),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwValueType),
                core::ptr::addr_of_mut!(CustomDrvInfo.DriverName) as LPBYTE,
                core::ptr::addr_of_mut!(dwInput),
            ) as ::core::ffi::c_int;
            RegCloseKey(CustomKey);
            // Divergence: the C closes CustomKey here AND again at custom_end (a double close
            // that can close a recycled handle). Forget it once closed; custom_end skips null.
            CustomKey = core::ptr::null_mut();

            if (ret != ERROR_SUCCESS as ::core::ffi::c_int)
                || (dwValueType != REG_SZ)
                || wcslen(core::ptr::addr_of!(CustomDrvInfo.DriverName) as *const WCHAR) == 0
            {
                break 'custom_end;
            }

            if _wcsicmp(
                core::ptr::addr_of!(CustomDrvInfo.DriverName) as *const WCHAR,
                wide_bytes!(b"R\0e\0a\0c\0t\0O\0S\0 \0S\0o\0f\0t\0w\0a\0r\0e\0 \0I\0m\0p\0l\0e\0m\0e\0n\0t\0a\0t\0i\0o\0n\0\0\0"),
            ) == 0
            {
                /* Always announce the fact that we're forcing ROSSWI */
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8 as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        99 as ::core::ffi::c_int,
                        b"Forcing ReactOS Software Implementation\n\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                CustomDriverState = OGL_CD_ROSSWI;
                return core::ptr::null_mut();
            }

            ret = RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                core::ptr::addr_of!(OpenGLDrivers_Key) as LPCWSTR,
                0,
                KEY_READ as REGSAM,
                core::ptr::addr_of_mut!(OglKey),
            ) as ::core::ffi::c_int;
            if ret != ERROR_SUCCESS as ::core::ffi::c_int {
                break 'custom_end;
            }

            // Divergence: the C opens the subkey straight into OglKey
            // (`RegOpenKeyExW(OglKey, ..., &OglKey)`), overwriting — and so leaking — the
            // OpenGLDrivers handle. Open into a scratch key and close the parent first.
            DrvKey = core::ptr::null_mut();
            ret = RegOpenKeyExW(
                OglKey,
                core::ptr::addr_of!(CustomDrvInfo.DriverName) as LPCWSTR,
                0,
                KEY_READ as REGSAM,
                core::ptr::addr_of_mut!(DrvKey),
            ) as ::core::ffi::c_int;
            RegCloseKey(OglKey);
            OglKey = core::ptr::null_mut();
            if ret != ERROR_SUCCESS as ::core::ffi::c_int {
                break 'custom_end;
            }
            OglKey = DrvKey;
            DrvKey = core::ptr::null_mut();

            dwInput = core::mem::size_of::<ULONG>() as DWORD; // sizeof(CustomDrvInfo.Version)
            ret = RegQueryValueExW(
                OglKey,
                wide_bytes!(b"V\0e\0r\0s\0i\0o\0n\0\0\0"),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwValueType),
                core::ptr::addr_of_mut!(CustomDrvInfo.Version) as LPBYTE,
                core::ptr::addr_of_mut!(dwInput),
            ) as ::core::ffi::c_int;
            if (ret != ERROR_SUCCESS as ::core::ffi::c_int) || (dwValueType != REG_DWORD) {
                break 'custom_end;
            }

            dwInput = core::mem::size_of::<DWORD>() as DWORD; // sizeof(DriverVersion)
            ret = RegQueryValueExW(
                OglKey,
                wide_bytes!(b"D\0r\0i\0v\0e\0r\0V\0e\0r\0s\0i\0o\0n\0\0\0"),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwValueType),
                core::ptr::addr_of_mut!(CustomDrvInfo.DriverVersion) as LPBYTE,
                core::ptr::addr_of_mut!(dwInput),
            ) as ::core::ffi::c_int;
            CustomDriverState = OGL_CD_CUSTOM_ICD;

            /* Always announce the fact that we're overriding the default driver */
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/opengl/opengl32/icdload.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                    122 as ::core::ffi::c_int,
                    b"Overriding the default OGL ICD with %S\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    core::ptr::addr_of_mut!(CustomDrvInfo.DriverName) as *mut WCHAR,
                );
            }
        }
        // custom_end:
        if !OglKey.is_null() {
            RegCloseKey(OglKey);
        }
        OglKey = core::ptr::null_mut();
        if !CustomKey.is_null() {
            RegCloseKey(CustomKey);
        }
    }

    /* If there's a custom ICD or ROSSWI was requested use it, otherwise proceed as usual */
    if CustomDriverState == OGL_CD_CUSTOM_ICD {
        pDrvInfo = core::ptr::addr_of_mut!(CustomDrvInfo);
    } else if CustomDriverState == OGL_CD_ROSSWI {
        return core::ptr::null_mut();
    } else {
        /* First, see if the driver supports this */
        dwInput = OPENGL_GETINFO as DWORD;
        ret = ExtEscape(
            hdc,
            QUERYESCSUPPORT as ::core::ffi::c_int,
            core::mem::size_of::<DWORD>() as ::core::ffi::c_int,
            core::ptr::addr_of!(dwInput) as LPCSTR,
            0,
            core::ptr::null_mut(),
        );

        /* Driver doesn't support opengl */
        if ret <= 0 {
            return core::ptr::null_mut();
        }

        /* Query for the ICD DLL name and version */
        dwInput = (OPENGL_GETINFO_DRVNAME as u32);
        ret = ExtEscape(
            hdc,
            OPENGL_GETINFO as ::core::ffi::c_int,
            core::mem::size_of::<DWORD>() as ::core::ffi::c_int,
            core::ptr::addr_of!(dwInput) as LPCSTR,
            core::mem::size_of::<Drv_Opengl_Info>() as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(DrvInfo) as LPSTR,
        );

        if ret <= 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/opengl/opengl32/icdload.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                    155 as ::core::ffi::c_int,
                    b"Driver claims to support OPENGL_GETINFO escape code, but doesn't. ret: %X\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    ret,
                );
            }
            return core::ptr::null_mut();
        }

        pDrvInfo = core::ptr::addr_of_mut!(DrvInfo);
    }

    /* Protect the list while we are loading*/
    EnterCriticalSection(core::ptr::addr_of_mut!(icdload_cs));

    /* Search for it in the list of already loaded modules */
    data = ICD_Data_List;
    while !data.is_null() {
        if _wcsicmp(
            core::ptr::addr_of!((*data).DriverName) as *const WCHAR,
            core::ptr::addr_of!((*pDrvInfo).DriverName) as *const WCHAR,
        ) == 0
        {
            /* Found it */
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/opengl/opengl32/icdload.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                    172 as ::core::ffi::c_int,
                    b"Found already loaded %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    data,
                );
            }
            LeaveCriticalSection(core::ptr::addr_of_mut!(icdload_cs));
            return data;
        }
        data = (*data).next;
    }

    'fail: {
        'end: {
            /* It was still not loaded, look for it in the registry */
            ret = RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                core::ptr::addr_of!(OpenGLDrivers_Key) as LPCWSTR,
                0,
                KEY_READ as REGSAM,
                core::ptr::addr_of_mut!(OglKey),
            ) as ::core::ffi::c_int;
            if ret != ERROR_SUCCESS as ::core::ffi::c_int {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        183 as ::core::ffi::c_int,
                        b"Failed to open the OpenGLDrivers key.\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                break 'end;
            }
            ret = RegOpenKeyExW(
                OglKey,
                core::ptr::addr_of!((*pDrvInfo).DriverName) as LPCWSTR,
                0,
                KEY_READ as REGSAM,
                core::ptr::addr_of_mut!(DrvKey),
            ) as ::core::ffi::c_int;
            if ret != ERROR_SUCCESS as ::core::ffi::c_int {
                /* Some driver installer just provide the DLL name, like the Matrox G400 */
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        190 as ::core::ffi::c_int,
                        b"No driver subkey for %S, trying to get DLL name directly.\n\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!((*pDrvInfo).DriverName) as *mut WCHAR,
                    );
                }
                dwInput = core::mem::size_of_val(&DllName) as DWORD;
                ret = RegQueryValueExW(
                    OglKey,
                    core::ptr::addr_of!((*pDrvInfo).DriverName) as LPCWSTR,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dwValueType),
                    DllName.as_mut_ptr() as LPBYTE,
                    core::ptr::addr_of_mut!(dwInput),
                ) as ::core::ffi::c_int;
                if (ret != ERROR_SUCCESS as ::core::ffi::c_int) || (dwValueType != REG_SZ) {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/opengl/opengl32/icdload.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                            195 as ::core::ffi::c_int,
                            b"Unable to get ICD DLL name!\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    }
                    RegCloseKey(OglKey);
                    break 'end;
                }
                Flags = 0;
                DriverVersion = Flags;
                Version = DriverVersion;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        200 as ::core::ffi::c_int,
                        b"DLL name is %S.\n\0" as *const u8 as *const ::core::ffi::c_char,
                        DllName.as_mut_ptr(),
                    );
                }
            } else {
                /* The driver have a subkey for the ICD */
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        205 as ::core::ffi::c_int,
                        b"Querying details from registry for %S.\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!((*pDrvInfo).DriverName) as *mut WCHAR,
                    );
                }
                dwInput = core::mem::size_of_val(&DllName) as DWORD;
                ret = RegQueryValueExW(
                    DrvKey,
                    wide_bytes!(b"D\0l\0l\0\0\0"),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dwValueType),
                    DllName.as_mut_ptr() as LPBYTE,
                    core::ptr::addr_of_mut!(dwInput),
                ) as ::core::ffi::c_int;
                if (ret != ERROR_SUCCESS as ::core::ffi::c_int) || (dwValueType != REG_SZ) {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/opengl/opengl32/icdload.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                            210 as ::core::ffi::c_int,
                            b"Unable to get ICD DLL name!.\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    }
                    RegCloseKey(DrvKey);
                    RegCloseKey(OglKey);
                    break 'end;
                }

                dwInput = core::mem::size_of::<DWORD>() as DWORD; // sizeof(Version)
                ret = RegQueryValueExW(
                    DrvKey,
                    wide_bytes!(b"V\0e\0r\0s\0i\0o\0n\0\0\0"),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dwValueType),
                    core::ptr::addr_of_mut!(Version) as LPBYTE,
                    core::ptr::addr_of_mut!(dwInput),
                ) as ::core::ffi::c_int;
                if (ret != ERROR_SUCCESS as ::core::ffi::c_int) || (dwValueType != REG_DWORD) {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int))
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/opengl/opengl32/icdload.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                            220 as ::core::ffi::c_int,
                            b"No version in driver subkey\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    }
                } else if Version != (*pDrvInfo).Version as DWORD {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/opengl/opengl32/icdload.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                            224 as ::core::ffi::c_int,
                            b"Version mismatch between registry (%lu) and display driver (%lu).\n\0"
                                as *const u8
                                as *const ::core::ffi::c_char,
                            Version,
                            (*pDrvInfo).Version,
                        );
                    }
                    RegCloseKey(DrvKey);
                    RegCloseKey(OglKey);
                    break 'end;
                }

                dwInput = core::mem::size_of::<DWORD>() as DWORD; // sizeof(DriverVersion)
                ret = RegQueryValueExW(
                    DrvKey,
                    wide_bytes!(b"D\0r\0i\0v\0e\0r\0V\0e\0r\0s\0i\0o\0n\0\0\0"),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dwValueType),
                    core::ptr::addr_of_mut!(DriverVersion) as LPBYTE,
                    core::ptr::addr_of_mut!(dwInput),
                ) as ::core::ffi::c_int;
                if (ret != ERROR_SUCCESS as ::core::ffi::c_int) || (dwValueType != REG_DWORD) {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int))
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/opengl/opengl32/icdload.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                            234 as ::core::ffi::c_int,
                            b"No driver version in driver subkey\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    }
                } else if DriverVersion != (*pDrvInfo).DriverVersion as DWORD {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/opengl/opengl32/icdload.c\0" as *const u8 as *const ::core::ffi::c_char,
                            b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                            238 as ::core::ffi::c_int,
                            b"Driver version mismatch between registry (%lu) and display driver (%lu).\n\0" as *const u8 as *const ::core::ffi::c_char,
                            DriverVersion,
                            (*pDrvInfo).DriverVersion,
                        );
                    }
                    RegCloseKey(DrvKey);
                    RegCloseKey(OglKey);
                    break 'end;
                }

                dwInput = core::mem::size_of::<DWORD>() as DWORD; // sizeof(Flags)
                ret = RegQueryValueExW(
                    DrvKey,
                    wide_bytes!(b"F\0l\0a\0g\0s\0\0\0"),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dwValueType),
                    core::ptr::addr_of_mut!(Flags) as LPBYTE,
                    core::ptr::addr_of_mut!(dwInput),
                ) as ::core::ffi::c_int;
                if (ret != ERROR_SUCCESS as ::core::ffi::c_int) || (dwValueType != REG_DWORD) {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_WARN as ::core::ffi::c_int))
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/opengl/opengl32/icdload.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                            248 as ::core::ffi::c_int,
                            b"No driver version in driver subkey\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    }
                    Flags = 0;
                }

                /* We're done */
                RegCloseKey(DrvKey);
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        254 as ::core::ffi::c_int,
                        b"DLL name is %S, Version %lx, DriverVersion %lx, Flags %lx.\n\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        DllName.as_mut_ptr(),
                        Version,
                        DriverVersion,
                        Flags,
                    );
                }
            }
            /* No need for this anymore */
            RegCloseKey(OglKey);

            /* So far so good, allocate data */
            data = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<ICD_Data>() as SIZE_T,
            ) as *mut ICD_Data;
            if data.is_null() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        263 as ::core::ffi::c_int,
                        b"Unable to allocate ICD data!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                break 'end;
            }

            /* Load the library */
            (*data).hModule = LoadLibraryW(DllName.as_ptr());
            if (*data).hModule.is_null() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        271 as ::core::ffi::c_int,
                        b"Could not load the ICD DLL: %S.\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        DllName.as_mut_ptr(),
                    );
                }
                HeapFree(GetProcessHeap(), 0, data as PVOID);
                data = core::ptr::null_mut();
                break 'end;
            }

            /*
             * Validate version, if needed.
             * Some drivers (at least VBOX), initialize stuff upon this call.
             */
            DrvValidateVersion = core::mem::transmute::<
                FARPROC,
                Option<unsafe extern "system" fn(DWORD) -> BOOL>,
            >(GetProcAddress(
                (*data).hModule,
                b"DrvValidateVersion\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if let Some(DrvValidateVersion) = DrvValidateVersion {
                if DrvValidateVersion((*pDrvInfo).DriverVersion as DWORD) == 0 {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/opengl/opengl32/icdload.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                            286 as ::core::ffi::c_int,
                            b"DrvValidateVersion failed!.\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    }
                    break 'fail;
                }
            }

            /* Pass the callbacks */
            DrvSetCallbackProcs = core::mem::transmute::<
                FARPROC,
                Option<unsafe extern "system" fn(::core::ffi::c_int, *mut PROC)>,
            >(GetProcAddress(
                (*data).hModule,
                b"DrvSetCallbackProcs\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if let Some(DrvSetCallbackProcs) = DrvSetCallbackProcs {
                // (PROC)fn casts: through a data pointer, so the cast is independent of how the
                // module spells the three callbacks' ABI.
                let mut callbacks: [PROC; 3] = [
                    core::mem::transmute::<*const ::core::ffi::c_void, PROC>(
                        wglSetCurrentValue as *const ::core::ffi::c_void,
                    ),
                    core::mem::transmute::<*const ::core::ffi::c_void, PROC>(
                        wglGetCurrentValue as *const ::core::ffi::c_void,
                    ),
                    core::mem::transmute::<*const ::core::ffi::c_void, PROC>(
                        wglGetDHGLRC as *const ::core::ffi::c_void,
                    ),
                ];
                DrvSetCallbackProcs(
                    ARRAYSIZE!(callbacks) as ::core::ffi::c_int,
                    callbacks.as_mut_ptr(),
                );
            }

            /* Get the DLL exports */
            // DRV_LOAD(DrvCopyContext);
            (*data).DrvCopyContext = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvCopyContext\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvCopyContext.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvCopyContext!\n\0" as *const u8 as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvCreateContext);
            (*data).DrvCreateContext = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvCreateContext\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvCreateContext.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvCreateContext!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvCreateLayerContext);
            (*data).DrvCreateLayerContext = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvCreateLayerContext\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvCreateLayerContext.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvCreateLayerContext!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvDeleteContext);
            (*data).DrvDeleteContext = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvDeleteContext\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvDeleteContext.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvDeleteContext!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvDescribeLayerPlane);
            (*data).DrvDescribeLayerPlane = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvDescribeLayerPlane\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvDescribeLayerPlane.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvDescribeLayerPlane!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvDescribePixelFormat);
            (*data).DrvDescribePixelFormat = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvDescribePixelFormat\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvDescribePixelFormat.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvDescribePixelFormat!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvGetLayerPaletteEntries);
            (*data).DrvGetLayerPaletteEntries = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvGetLayerPaletteEntries\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvGetLayerPaletteEntries.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvGetLayerPaletteEntries!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvGetProcAddress);
            (*data).DrvGetProcAddress = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvGetProcAddress\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvGetProcAddress.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvGetProcAddress!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvReleaseContext);
            (*data).DrvReleaseContext = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvReleaseContext\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvReleaseContext.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvReleaseContext!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvRealizeLayerPalette);
            (*data).DrvRealizeLayerPalette = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvRealizeLayerPalette\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvRealizeLayerPalette.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvRealizeLayerPalette!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvSetContext);
            (*data).DrvSetContext = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvSetContext\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvSetContext.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvSetContext!\n\0" as *const u8 as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvSetLayerPaletteEntries);
            (*data).DrvSetLayerPaletteEntries = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvSetLayerPaletteEntries\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvSetLayerPaletteEntries.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvSetLayerPaletteEntries!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvSetPixelFormat);
            (*data).DrvSetPixelFormat = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvSetPixelFormat\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvSetPixelFormat.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvSetPixelFormat!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvShareLists);
            (*data).DrvShareLists = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvShareLists\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvShareLists.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvShareLists!\n\0" as *const u8 as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvSwapBuffers);
            (*data).DrvSwapBuffers = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvSwapBuffers\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvSwapBuffers.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvSwapBuffers!\n\0" as *const u8 as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }
            // DRV_LOAD(DrvSwapLayerBuffers);
            (*data).DrvSwapLayerBuffers = core::mem::transmute::<FARPROC, _>(GetProcAddress(
                (*data).hModule,
                b"DrvSwapLayerBuffers\0" as *const u8 as *const ::core::ffi::c_char,
            ));
            if (*data).DrvSwapLayerBuffers.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_int,
                        b"%S lacks DrvSwapLayerBuffers!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        core::ptr::addr_of_mut!(DllName) as *mut WCHAR,
                    );
                }
                break 'fail;
            }

            /* Let's see if GDI should handle this instead of the ICD DLL */
            // FIXME: maybe there is a better way
            if GdiDescribePixelFormat(hdc, 0, 0, core::ptr::null_mut()) != 0 {
                /* GDI knows what to do with that. Override */
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/opengl/opengl32/icdload.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                        334 as ::core::ffi::c_int,
                        b"Forwarding WGL calls to win32k!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                (*data).DrvDescribePixelFormat = Some(GdiDescribePixelFormat);
                (*data).DrvSetPixelFormat = Some(GdiSetPixelFormat);
                (*data).DrvSwapBuffers = Some(GdiSwapBuffers);
            }

            /* Copy the DriverName */
            wcscpy(
                core::ptr::addr_of_mut!((*data).DriverName) as *mut WCHAR,
                core::ptr::addr_of!((*pDrvInfo).DriverName) as *const WCHAR,
            );

            /* Push the list */
            (*data).next = ICD_Data_List;
            ICD_Data_List = data;

            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/opengl/opengl32/icdload.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                    347 as ::core::ffi::c_int,
                    b"Returning %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    data,
                );
            }
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/opengl/opengl32/icdload.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"IntGetIcdData\0" as *const u8 as *const ::core::ffi::c_char,
                    348 as ::core::ffi::c_int,
                    b"ICD driver %S (%S) successfully loaded.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    core::ptr::addr_of_mut!((*pDrvInfo).DriverName) as *mut WCHAR,
                    DllName.as_mut_ptr(),
                );
            }
        }
        // end:
        /* Unlock and return */
        LeaveCriticalSection(core::ptr::addr_of_mut!(icdload_cs));
        return data;
    }
    // fail:
    LeaveCriticalSection(core::ptr::addr_of_mut!(icdload_cs));
    FreeLibrary((*data).hModule);
    HeapFree(GetProcessHeap(), 0, data as PVOID);
    return core::ptr::null_mut();
}
