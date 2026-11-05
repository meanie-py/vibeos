use crate::ddraw::main::GetSurfaceFromDC;
use crate::ddraw::device::IID_D3DDEVICE_WineD3D;
use crate::ddraw::main::force_refresh_rate;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub LowPart: DWORD,
    pub HighPart: LONG,
}

static mut compare: [compare_info; 15] = [
        compare_info {
            flag: DDSD_ALPHABITDEPTH as DWORD,
            offset: 28 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DWORD>() as size_t,
        },
        compare_info {
            flag: DDSD_BACKBUFFERCOUNT as DWORD,
            offset: 20 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DWORD>() as size_t,
        },
        compare_info {
            flag: DDSD_CAPS as DWORD,
            offset: 112 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DDSCAPS2>() as size_t,
        },
        compare_info {
            flag: DDSD_CKDESTBLT as DWORD,
            offset: 56 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DDCOLORKEY>() as size_t,
        },
        compare_info {
            flag: DDSD_CKDESTOVERLAY as DWORD,
            offset: 48 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DDCOLORKEY>() as size_t,
        },
        compare_info {
            flag: DDSD_CKSRCBLT as DWORD,
            offset: 72 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DDCOLORKEY>() as size_t,
        },
        compare_info {
            flag: DDSD_CKSRCOVERLAY as DWORD,
            offset: 64 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DDCOLORKEY>() as size_t,
        },
        compare_info {
            flag: DDSD_HEIGHT as DWORD,
            offset: 8 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DWORD>() as size_t,
        },
        compare_info {
            flag: DDSD_LINEARSIZE as DWORD,
            offset: 16 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DWORD>() as size_t,
        },
        compare_info {
            flag: DDSD_LPSURFACE as DWORD,
            offset: 40 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<LPVOID>() as size_t,
        },
        compare_info {
            flag: DDSD_MIPMAPCOUNT as DWORD,
            offset: 24 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DWORD>() as size_t,
        },
        compare_info {
            flag: DDSD_PITCH as DWORD,
            offset: 16 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<LONG>() as size_t,
        },
        compare_info {
            flag: DDSD_REFRESHRATE as DWORD,
            offset: 24 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DWORD>() as size_t,
        },
        compare_info {
            flag: DDSD_TEXTURESTAGE as DWORD,
            offset: 128 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DWORD>() as size_t,
        },
        compare_info {
            flag: DDSD_WIDTH as DWORD,
            offset: 12 as ::core::ffi::c_ulonglong as ptrdiff_t,
            size: ::core::mem::size_of::<DWORD>() as size_t,
        },
    ];

pub static DDRAW_WINDOW_CLASS_NAME: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"DirectDrawDeviceWnd\0")
};

static mut exclusive_ddraw: *const ddraw = ::core::ptr::null::<ddraw>();

static mut exclusive_window: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut deviceidentifier: DDDEVICEIDENTIFIER2 = unsafe {
    tagDDDEVICEIDENTIFIER2 {
        szDriver: ::core::mem::transmute::<
            [u8; 512],
            [::core::ffi::c_char; 512],
        >(
            *b"vga.dll\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        szDescription: ::core::mem::transmute::<
            [u8; 512],
            [::core::ffi::c_char; 512],
        >(
            *b"DirectDraw HAL\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        liDriverVersion: _LARGE_INTEGER {
            __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 { LowPart: 0x10001 as ::core::ffi::c_int as DWORD,
                HighPart: 0x10001 as LONG,
            },
        },
        dwVendorId: 0 as DWORD,
        dwDeviceId: 0 as DWORD,
        dwSubSysId: 0 as DWORD,
        dwRevision: 0 as DWORD,
        guidDeviceIdentifier: _GUID {
            Data1: 0xa8373c10 as ::core::ffi::c_uint,
            Data2: 0x7ac4 as ::core::ffi::c_ushort,
            Data3: 0x4deb as ::core::ffi::c_ushort,
            Data4: [
                0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x98 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x2d as ::core::ffi::c_int as ::core::ffi::c_uchar,
            ],
        },
        dwWHQLLevel: 0 as DWORD,
    }
};

static mut device_list7: [enum_device_entry; 3] = unsafe {
    [
        enum_device_entry {
            interface_name: ::core::mem::transmute::<
                [u8; 100],
                [::core::ffi::c_char; 100],
            >(
                *b"WINE Direct3D7 Hardware Transform and Lighting acceleration using WineD3D\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            device_name: ::core::mem::transmute::<
                [u8; 100],
                [::core::ffi::c_char; 100],
            >(
                *b"Wine D3D7 T&L HAL\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            device_guid: &raw const IID_IDirect3DTnLHalDevice,
            unsupported_caps: 0 as DWORD,
        },
        enum_device_entry {
            interface_name: ::core::mem::transmute::<
                [u8; 100],
                [::core::ffi::c_char; 100],
            >(
                *b"WINE Direct3D7 Hardware acceleration using WineD3D\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            device_name: ::core::mem::transmute::<
                [u8; 100],
                [::core::ffi::c_char; 100],
            >(
                *b"Direct3D HAL\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            device_guid: &raw const IID_IDirect3DHALDevice,
            unsupported_caps: D3DDEVCAPS_HWTRANSFORMANDLIGHT as DWORD,
        },
        enum_device_entry {
            interface_name: ::core::mem::transmute::<
                [u8; 100],
                [::core::ffi::c_char; 100],
            >(
                *b"WINE Direct3D7 RGB Software Emulation using WineD3D\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            device_name: ::core::mem::transmute::<
                [u8; 100],
                [::core::ffi::c_char; 100],
            >(
                *b"Wine D3D7 RGB\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            device_guid: &raw const IID_IDirect3DRGBDevice,
            unsupported_caps: (D3DDEVCAPS_HWTRANSFORMANDLIGHT
                | D3DDEVCAPS_DRAWPRIMITIVES2EX | D3DDEVCAPS_HWRASTERIZATION) as DWORD,
        },
    ]
};

#[no_mangle]
pub static mut ddraw_null_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            ddraw_null_wined3d_object_destroyed,
        ),
    }
};

static mut ddraw_swapchain_state_parent_ops: wined3d_swapchain_state_parent_ops = unsafe {
    wined3d_swapchain_state_parent_ops {
        windowed_state_changed: Some(
            ddraw_swapchain_windowed_state_changed
                as unsafe extern "C" fn(*mut wined3d_swapchain_state_parent, BOOL) -> (),
        ),
    }
};

static mut ddraw_wined3d_device_parent_ops: wined3d_device_parent_ops = unsafe {
    wined3d_device_parent_ops {
        wined3d_device_created: Some(
            device_parent_wined3d_device_created
                as unsafe extern "C" fn(*mut wined3d_device_parent, *mut wined3d_device) -> (),
        ),
        mode_changed: Some(
            device_parent_mode_changed as unsafe extern "C" fn(*mut wined3d_device_parent) -> (),
        ),
        activate: Some(
            device_parent_activate as unsafe extern "C" fn(*mut wined3d_device_parent, BOOL) -> (),
        ),
        texture_sub_resource_created: Some(
            device_parent_texture_sub_resource_created
                as unsafe extern "C" fn(
                    *mut wined3d_device_parent,
                    wined3d_resource_type,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    *mut *mut ::core::ffi::c_void,
                    *mut *const wined3d_parent_ops,
                ) -> HRESULT,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(ddraw);


#[repr(C)]
pub struct enum_device_entry {
    pub interface_name: [i8; 100],
    pub device_name: [i8; 100],
    pub device_guid: *const GUID,
    pub unsupported_caps: DWORD,
}

#[repr(C)]
pub struct displaymodescallback_context {
    pub func: LPDDENUMMODESCALLBACK,
    pub context: *mut core::ffi::c_void,
}

#[repr(C)]
pub struct surfacescallback2_context {
    pub func: LPDDENUMSURFACESCALLBACK2,
    pub context: *mut core::ffi::c_void,
}

#[repr(C)]
pub struct surfacescallback_context {
    pub func: LPDDENUMSURFACESCALLBACK,
    pub context: *mut core::ffi::c_void,
}

#[repr(C)]
pub struct enum_surface_mode_params {
    pub ddraw: *mut IDirectDraw7,
    pub desc: *const DDSURFACEDESC2,
    pub callback: LPDDENUMSURFACESCALLBACK7,
    pub context: *mut core::ffi::c_void,
}

unsafe extern "system" fn ddraw_null_wined3d_object_destroyed(
    parent: *mut core::ffi::c_void,
) {}

unsafe extern "C" fn ddraw_swapchain_windowed_state_changed(
    parent: *mut wined3d_swapchain_state_parent,
    windowed: BOOL,
) {
    // TRACE("parent %p, windowed %d.\n", parent, windowed);
}

unsafe extern "C" fn impl_from_IDirectDraw(iface: *mut IDirectDraw) -> *mut ddraw {
    return CONTAINING_RECORD!(iface, ddraw, IDirectDraw_iface);
}

unsafe extern "C" fn impl_from_IDirectDraw2(iface: *mut IDirectDraw2) -> *mut ddraw {
    return CONTAINING_RECORD!(iface, ddraw, IDirectDraw2_iface);
}

unsafe extern "C" fn impl_from_IDirectDraw4(iface: *mut IDirectDraw4) -> *mut ddraw {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(ddraw, IDirectDraw4_iface)) as *mut ddraw;
}

unsafe extern "C" fn impl_from_IDirectDraw7(iface: *mut IDirectDraw7) -> *mut ddraw {
    return CONTAINING_RECORD!(iface, ddraw, IDirectDraw7_iface);
}

unsafe extern "C" fn impl_from_IDirect3D(iface: *mut IDirect3D) -> *mut ddraw {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(ddraw, IDirect3D_iface)) as *mut ddraw;
}

unsafe extern "C" fn impl_from_IDirect3D2(iface: *mut IDirect3D2) -> *mut ddraw {
    return CONTAINING_RECORD!(iface, ddraw, IDirect3D2_iface);
}

unsafe extern "C" fn impl_from_IDirect3D3(iface: *mut IDirect3D3) -> *mut ddraw {
    return CONTAINING_RECORD!(iface, ddraw, IDirect3D3_iface);
}

unsafe extern "C" fn impl_from_IDirect3D7(iface: *mut IDirect3D7) -> *mut ddraw {
    return CONTAINING_RECORD!(iface, ddraw, IDirect3D7_iface);
}

unsafe extern "system" fn ddraw7_QueryInterface(iface: *mut IDirectDraw7, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);

    if riid.is_null() {
        *out = core::ptr::null_mut();
        return DDERR_INVALIDPARAMS;
    }

    /* The refcount unit test revealed that an IDirect3D7 interface can only
     * be queried from a DirectDraw object that was created as an IDirectDraw7
     * interface. The older interfaces can query any IDirect3D version except
     * 7, because they are all initially created as IDirectDraw. This isn't
     * really crucial behavior, and messy to implement with the common
     * creation function, so it has been left out here. */
    if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectDraw7), riid)
        || IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid)
    {
        *out = core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectDraw4), riid) {
        *out = core::ptr::addr_of_mut!((*ddraw).IDirectDraw4_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectDraw2), riid) {
        *out = core::ptr::addr_of_mut!((*ddraw).IDirectDraw2_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID!(core::ptr::addr_of!(IID_IDirectDraw), riid) {
        *out = core::ptr::addr_of_mut!((*ddraw).IDirectDraw_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3D7), riid) {
        (*ddraw).d3dversion = 7;
        *out = core::ptr::addr_of_mut!((*ddraw).IDirect3D7_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3D3), riid) {
        (*ddraw).d3dversion = 3;
        *out = core::ptr::addr_of_mut!((*ddraw).IDirect3D3_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3D2), riid) {
        (*ddraw).d3dversion = 2;
        *out = core::ptr::addr_of_mut!((*ddraw).IDirect3D2_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3D), riid) {
        (*ddraw).d3dversion = 1;
        *out = core::ptr::addr_of_mut!((*ddraw).IDirect3D_iface) as *mut core::ffi::c_void;
    } else {
        /* Unknown interface */
        WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef((*out) as *mut IUnknown);
    return S_OK;
}
unsafe extern "system" fn ddraw4_QueryInterface(
    iface: *mut IDirectDraw4,
    riid: *const GUID,
    object: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    WINE_TRACE!("iface %p, riid %s, object %p.\n", iface, debugstr_guid(riid), object);
    return ddraw7_QueryInterface(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), riid, object);
}

unsafe extern "system" fn ddraw2_QueryInterface(iface: *mut IDirectDraw2, riid: *const GUID, object: *mut *mut core::ffi::c_void) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);

    return ddraw7_QueryInterface(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), riid, object);
}

unsafe extern "system" fn ddraw1_QueryInterface(
    iface: *mut IDirectDraw,
    riid: *const IID,
    object: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    WINE_TRACE!("iface %p, riid %s, object %p.\n", iface, debugstr_guid(riid as *const _GUID), object);
    return ddraw7_QueryInterface(&raw mut (*ddraw).IDirectDraw7_iface, riid, object);
}

unsafe extern "system" fn d3d7_QueryInterface(
    mut iface: *mut IDirect3D7,
    riid: *const GUID,
    mut object: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirect3D7(iface);
    WINE_TRACE!("iface %p, riid %s, object %p.\n", iface, debugstr_guid(riid as *const _GUID), object);
    return ddraw7_QueryInterface(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        riid,
        object,
    );
}
unsafe extern "system" fn d3d3_QueryInterface(
    mut iface: *mut IDirect3D3,
    riid: *const IID,
    mut object: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirect3D3(iface);
    WINE_TRACE!("iface %p, riid %s, object %p.\n", iface, debugstr_guid(riid as *const _GUID), object);
    return ddraw7_QueryInterface(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), riid, object);
}

unsafe extern "system" fn d3d2_QueryInterface(iface: *mut IDirect3D2, riid: *const GUID, object: *mut *mut core::ffi::c_void) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D2(iface);

    return ddraw7_QueryInterface(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), riid, object);
}

unsafe extern "system" fn d3d1_QueryInterface(iface: *mut IDirect3D, riid: REFIID, object: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirect3D(iface);
    TRACE!(
        "iface %p, riid %s, object %p.\n",
        iface,
        debugstr_guid(riid as *const GUID),
        object
    );
    return ddraw7_QueryInterface(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), riid, object);
}

unsafe extern "system" fn ddraw7_AddRef(iface: *mut IDirectDraw7) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut ref_0: ULONG = InterlockedIncrement(
        &raw mut (*This).ref7 as *mut ::core::ffi::c_long,
    ) as ULONG;

    TRACE!("%p increasing refcount to %lu.\n", This, ref_0);

    if ref_0 == 1 as ULONG {
        InterlockedIncrement(
            &raw mut (*This).numIfaces as *mut ::core::ffi::c_long,
        );
    }

    return ref_0;
}

unsafe extern "system" fn ddraw4_AddRef(iface: *mut IDirectDraw4) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirectDraw4(iface);
    let mut ref_0: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*This).ref4) as *mut ::core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("%p increasing refcount to %lu.\n", This, ref_0);
    if ref_0 == 1 {
        InterlockedIncrement(
            core::ptr::addr_of_mut!((*This).numIfaces) as *mut ::core::ffi::c_long,
        );
    }
    return ref_0;
}

unsafe extern "system" fn ddraw2_AddRef(iface: *mut IDirectDraw2) -> ULONG {
    let This: *mut ddraw = impl_from_IDirectDraw2(iface);
    let mut ref_0: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*This).ref2) as *mut i32,
    ) as ULONG;

    if ref_0 == 1 {
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).numIfaces) as *mut i32);
    }

    return ref_0;
}

unsafe extern "system" fn ddraw1_AddRef(iface: *mut IDirectDraw) -> ULONG {
    let This: *mut ddraw = impl_from_IDirectDraw(iface);
    let mut ref_: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*This).ref1) as *mut core::ffi::c_long,
    ) as ULONG;

    TRACE!("%p increasing refcount to %lu.\n", This, ref_);

    if ref_ == 1 {
        InterlockedIncrement(
            core::ptr::addr_of_mut!((*This).numIfaces) as *mut core::ffi::c_long,
        );
    }

    return ref_;
}
unsafe extern "system" fn d3d7_AddRef(mut iface: *mut IDirect3D7) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirect3D7(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return ddraw7_AddRef(&raw mut (*This).IDirectDraw7_iface);
}
unsafe extern "system" fn d3d3_AddRef(iface: *mut IDirect3D3) -> ULONG {
    let This: *mut ddraw = impl_from_IDirect3D3(iface);

    TRACE!("iface %p.\n", iface);

    return ddraw1_AddRef(core::ptr::addr_of_mut!((*This).IDirectDraw_iface));
}

unsafe extern "system" fn d3d2_AddRef(iface: *mut IDirect3D2) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirect3D2(iface);

    TRACE!("iface %p.\n", iface);

    return ddraw1_AddRef(core::ptr::addr_of_mut!((*This).IDirectDraw_iface));
}

unsafe extern "system" fn d3d1_AddRef(iface: *mut IDirect3D) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirect3D(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return ddraw1_AddRef(core::ptr::addr_of_mut!((*This).IDirectDraw_iface));
}

unsafe extern "C" fn ddraw_destroy_swapchain(ddraw: *mut ddraw) {
    let mut i: ::core::ffi::c_uint = 0;

    WINE_TRACE!("Destroying the swapchain.\n");

    wined3d_swapchain_decref((*ddraw).wined3d_swapchain);

    i = 0 as ::core::ffi::c_uint;
    while i < (*ddraw).numConvertedDecls {
        wined3d_vertex_declaration_decref((*(*ddraw).decls.offset(i as isize)).decl);
        i = i.wrapping_add(1);
    }
    free((*ddraw).decls as *mut ::core::ffi::c_void);
    (*ddraw).numConvertedDecls = 0 as UINT;

    wined3d_swapchain_decref((*ddraw).wined3d_swapchain);
    (*ddraw).wined3d_swapchain = ::core::ptr::null_mut::<wined3d_swapchain>();

    /* Free the d3d window if one was created. */
    if !(*ddraw).d3d_window.is_null() && (*ddraw).d3d_window != (*ddraw).dest_window {
        WINE_TRACE!("Destroying the hidden render window %p.\n", (*ddraw).d3d_window);
        DestroyWindow((*ddraw).d3d_window);
        (*ddraw).d3d_window = core::ptr::null_mut();
    }

    (*ddraw).flags &= !(DDRAW_D3D_INITIALIZED as DWORD);

    ddraw_set_swapchain_window(ddraw, ::core::ptr::null_mut::<HWND__>());

    WINE_TRACE!("Swapchain destroyed.\n");
}

unsafe extern "C" fn ddraw_destroy(This: *mut ddraw) {
    let mut device: *mut d3d_device;
    IDirectDraw7_SetCooperativeLevel(core::ptr::addr_of_mut!((*This).IDirectDraw7_iface), core::ptr::null_mut(), DDSCL_NORMAL);
    IDirectDraw7_RestoreDisplayMode(core::ptr::addr_of_mut!((*This).IDirectDraw7_iface));

    /* Destroy the device window if we created one */
    if (*This).devicewindow != core::ptr::null_mut() {
        DestroyWindow((*This).devicewindow);
        (*This).devicewindow = core::ptr::null_mut();
    }

    wined3d_mutex_lock();
    list_remove(core::ptr::addr_of_mut!((*This).ddraw_list_entry));
    wined3d_mutex_unlock();

    if !(*This).wined3d_swapchain.is_null() {
        ddraw_destroy_swapchain(This);
    }
    wined3d_device_decref((*This).wined3d_device);
    wined3d_decref((*This).wined3d);

    /* LIST_FOR_EACH_ENTRY(device, &This->d3ddevice_list, struct d3d_device, ddraw_entry) */
    device = ((*This).d3ddevice_list.next as *mut u8)
        .sub(core::mem::offset_of!(d3d_device, ddraw_entry)) as *mut d3d_device;
    while core::ptr::addr_of_mut!((*device).ddraw_entry)
        != core::ptr::addr_of_mut!((*This).d3ddevice_list)
    {
        (*device).ddraw = core::ptr::null_mut();
        device = ((*device).ddraw_entry.next as *mut u8)
            .sub(core::mem::offset_of!(d3d_device, ddraw_entry)) as *mut d3d_device;
    }

    /* Now free the object */
    free(This as *mut core::ffi::c_void);
}
unsafe extern "system" fn ddraw7_Release(iface: *mut IDirectDraw7) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut ref_0: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*This).ref7) as *mut core::ffi::c_long,
    ) as ULONG;

    if ref_0 == 0
        && InterlockedDecrement(
            core::ptr::addr_of_mut!((*This).numIfaces) as *mut core::ffi::c_long,
        ) == 0
    {
        ddraw_destroy(This);
    }

    return ref_0;
}

unsafe extern "system" fn ddraw4_Release(iface: *mut IDirectDraw4) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirectDraw4(iface);
    let mut ref_0: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*This).ref4) as *mut ::core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("%p decreasing refcount to %lu.\n", This, ref_0);
    if ref_0 == 0
        && InterlockedDecrement(
            core::ptr::addr_of_mut!((*This).numIfaces) as *mut ::core::ffi::c_long,
        ) == 0
    {
        ddraw_destroy(This);
    }
    return ref_0;
}

unsafe extern "system" fn ddraw2_Release(iface: *mut IDirectDraw2) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirectDraw2(iface);
    let mut ref_0: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*This).ref2) as *mut ::core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("%p decreasing refcount to %lu.\n", This, ref_0);
    if ref_0 == 0
        && InterlockedDecrement(
            core::ptr::addr_of_mut!((*This).numIfaces) as *mut ::core::ffi::c_long,
        ) == 0
    {
        ddraw_destroy(This);
    }
    return ref_0;
}

unsafe extern "system" fn ddraw1_Release(iface: *mut IDirectDraw) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirectDraw(iface);
    let mut ref_0: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*This).ref1) as *mut i32,
    ) as ULONG;

    TRACE!("%p decreasing refcount to %lu.\n", This, ref_0);

    if ref_0 == 0
        && InterlockedDecrement(core::ptr::addr_of_mut!((*This).numIfaces) as *mut i32) == 0
    {
        ddraw_destroy(This);
    }

    return ref_0;
}

unsafe extern "system" fn d3d7_Release(iface: *mut IDirect3D7) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirect3D7(iface);

    return ddraw7_Release(core::ptr::addr_of_mut!((*This).IDirectDraw7_iface));
}
unsafe extern "system" fn d3d3_Release(mut iface: *mut IDirect3D3) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirect3D3(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return ddraw1_Release(&raw mut (*This).IDirectDraw_iface);
}
unsafe extern "system" fn d3d2_Release(iface: *mut IDirect3D2) -> ULONG {
    let mut This: *mut ddraw = impl_from_IDirect3D2(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return ddraw1_Release(&raw mut (*This).IDirectDraw_iface);
}

unsafe extern "system" fn d3d1_Release(iface: *mut IDirect3D) -> ULONG {
    let This: *mut ddraw = impl_from_IDirect3D(iface);

    WINE_TRACE!("iface %p.\n", iface);

    return ddraw1_Release(core::ptr::addr_of_mut!((*This).IDirectDraw_iface));
}

unsafe extern "C" fn ddraw_set_focus_window(ddraw: *mut ddraw, window: HWND) -> HRESULT {
    if (*ddraw).cooperative_level & DDSCL_EXCLUSIVE as DWORD != 0 && !(*ddraw).dest_window.is_null()
    {
        WINE_TRACE!("Setting DDSCL_SETFOCUSWINDOW with an already set window, returning DDERR_HWNDALREADYSET.\n");
        return DDERR_HWNDALREADYSET;
    }

    (*ddraw).focuswindow = window;

    return DD_OK;
}

unsafe extern "C" fn ddraw_attach_d3d_device(
    ddraw: *mut ddraw,
    mut window: HWND,
    cooplevel: DWORD,
    wined3d_swapchain: *mut *mut wined3d_swapchain,
) -> HRESULT {
    let mut swapchain_desc: wined3d_swapchain_desc = core::mem::zeroed();
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    hr = wined3d_output_get_display_mode((*ddraw).wined3d_output, core::ptr::addr_of_mut!(mode), core::ptr::null_mut());
    if hr < 0
    {
        ERR!("Failed to get display mode.\n");
        return hr;
    }

    memset(core::ptr::addr_of_mut!(swapchain_desc) as *mut core::ffi::c_void, 0, core::mem::size_of::<wined3d_swapchain_desc>());
    swapchain_desc.output = (*ddraw).wined3d_output;
    swapchain_desc.backbuffer_width = mode.width;
    swapchain_desc.backbuffer_height = mode.height;
    swapchain_desc.backbuffer_format = mode.format_id;
    swapchain_desc.backbuffer_bind_flags = 0;
    swapchain_desc.backbuffer_count = 1;
    swapchain_desc.swap_effect = WINED3D_SWAP_EFFECT_DISCARD;
    swapchain_desc.device_window = window;
    swapchain_desc.windowed = ((cooplevel & DDSCL_FULLSCREEN) == 0) as BOOL;
    swapchain_desc.flags = DDRAW_WINED3D_SWAPCHAIN_FLAGS;

    if ((cooplevel & DDSCL_NOWINDOWCHANGES) != 0) || window != GetForegroundWindow()
    {
        swapchain_desc.flags |= WINED3D_SWAPCHAIN_NO_WINDOW_CHANGES;
    }

    if ((*ddraw).flags & DDRAW_NO3D) != 0 {
        return wined3d_swapchain_create((*ddraw).wined3d_device, core::ptr::addr_of_mut!(swapchain_desc),
                core::ptr::addr_of_mut!((*ddraw).state_parent), core::ptr::null_mut(), core::ptr::addr_of!(ddraw_null_wined3d_parent_ops), wined3d_swapchain);
    }

    if window.is_null() || window == GetDesktopWindow()
    {
        window = CreateWindowExA(0, (DDRAW_WINDOW_CLASS_NAME).as_ptr(), b"Hidden D3D Window\0" as *const u8 as LPCSTR,
                WS_DISABLED, 0, 0, GetSystemMetrics((SM_CXSCREEN as i32)), GetSystemMetrics((SM_CYSCREEN as i32)),
                core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
        if window.is_null()
        {
            ERR!("Failed to create window, last error %#lx.\n", GetLastError());
            return E_FAIL;
        }

        ShowWindow(window, (SW_HIDE as i32));   /* Just to be sure */
        WARN!("No window for the Direct3DDevice, created hidden window %p.\n", window);

        swapchain_desc.device_window = window;
    }
    else
    {
        TRACE!("Using existing window %p for Direct3D rendering.\n", window);
    }
    (*ddraw).d3d_window = window;

    /* Set this NOW, otherwise creating the depth stencil surface will cause a
     * recursive loop until ram or emulated video memory is full. */
    (*ddraw).flags |= DDRAW_D3D_INITIALIZED;
    hr = wined3d_swapchain_create((*ddraw).wined3d_device, core::ptr::addr_of_mut!(swapchain_desc),
            core::ptr::addr_of_mut!((*ddraw).state_parent), core::ptr::null_mut(), core::ptr::addr_of!(ddraw_null_wined3d_parent_ops), wined3d_swapchain);
    if hr < 0
    {
        (*ddraw).flags &= !DDRAW_D3D_INITIALIZED;
        DestroyWindow(window);
        (*ddraw).d3d_window = core::ptr::null_mut();
        return hr;
    }

    (*ddraw).declArraySize = 2;
    (*ddraw).decls = calloc(((*ddraw).declArraySize as usize) as _, (core::mem::size_of::<FvfToDecl>()) as _) as *mut FvfToDecl;
    if (*ddraw).decls.is_null()
    {
        ERR!("Error allocating an array for the converted vertex decls.\n");
        (*ddraw).declArraySize = 0;
        wined3d_swapchain_decref(*wined3d_swapchain);
        DestroyWindow(window);
        (*ddraw).d3d_window = core::ptr::null_mut();
        return E_OUTOFMEMORY;
    }

    TRACE!("Successfully initialized 3D.\n");

    return DD_OK;
}

unsafe extern "C" fn ddraw_create_swapchain(ddraw: *mut ddraw, window: HWND, cooplevel: DWORD) -> HRESULT {
    let mut hr: HRESULT;

    if !(*ddraw).wined3d_swapchain.is_null() {
        ERR!("Swapchain already created.\n");
        return E_FAIL;
    }

    hr = ddraw_attach_d3d_device(
        ddraw,
        window,
        cooplevel,
        core::ptr::addr_of_mut!((*ddraw).wined3d_swapchain),
    );
    if hr < 0 {
        ERR!("Failed to create swapchain, hr %#lx.\n", hr);
        return hr;
    }
    wined3d_swapchain_incref((*ddraw).wined3d_swapchain);

    ddraw_set_swapchain_window(ddraw, window);

    if !(*ddraw).primary.is_null() && !(*(*ddraw).primary).palette.is_null() {
        wined3d_swapchain_set_palette(
            (*ddraw).wined3d_swapchain,
            (*(*(*ddraw).primary).palette).wined3d_palette,
        );
    }

    return DD_OK;
}

unsafe extern "system" fn ddraw7_RestoreDisplayMode(iface: *mut IDirectDraw7) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    let mut clip_rect: RECT = core::mem::zeroed();
    let mut hr: HRESULT;

    wined3d_mutex_lock();

    if ((*ddraw).flags & DDRAW_RESTORE_MODE) == 0 {
        wined3d_mutex_unlock();
        return DD_OK;
    }

    if !exclusive_ddraw.is_null() && exclusive_ddraw != ddraw as *const ddraw {
        wined3d_mutex_unlock();
        return DDERR_NOEXCLUSIVEMODE;
    }

    hr = wined3d_restore_display_modes((*ddraw).wined3d);
    if hr >= 0 {
        (*ddraw).flags &= !DDRAW_RESTORE_MODE;
        let mut cond: bool = ((*ddraw).cooperative_level & DDSCL_EXCLUSIVE) != 0;
        if cond {
            hr = wined3d_output_get_display_mode(
                (*ddraw).wined3d_output,
                core::ptr::addr_of_mut!(mode),
                core::ptr::null_mut(),
            );
            cond = hr >= 0;
        }
        if cond {
            SetRect(
                core::ptr::addr_of_mut!(clip_rect),
                0,
                0,
                mode.width as i32,
                mode.height as i32,
            );
            ClipCursor(core::ptr::addr_of_mut!(clip_rect) as LPCRECT);
        }
    }

    InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*ddraw).device_state) as *mut core::ffi::c_long,
        DDRAW_DEVICE_STATE_NOT_RESTORED as core::ffi::c_long,
        DDRAW_DEVICE_STATE_OK as core::ffi::c_long,
    );

    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn ddraw4_RestoreDisplayMode(mut iface: *mut IDirectDraw4) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return ddraw7_RestoreDisplayMode(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface));
}

unsafe extern "system" fn ddraw2_RestoreDisplayMode(mut iface: *mut IDirectDraw2) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return ddraw7_RestoreDisplayMode(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface));
}
unsafe extern "system" fn ddraw1_RestoreDisplayMode(iface: *mut IDirectDraw) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);

    return ddraw7_RestoreDisplayMode(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface));
}

unsafe extern "C" fn ddraw_set_cooperative_level(
    ddraw: *mut ddraw,
    mut window: HWND,
    cooplevel: DWORD,
    restore_mode_on_normal: BOOL,
) -> HRESULT {
    let mut restore_state: BOOL = (FALSE as i32);
    let mut device: *mut d3d_device = core::ptr::null_mut();
    let mut clip_rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hr: HRESULT = 0;

    TRACE!(
        "ddraw %p, window %p, flags %#lx, restore_mode_on_normal %x.\n",
        ddraw,
        window,
        cooplevel,
        restore_mode_on_normal,
    );
    DDRAW_dump_cooperativelevel(cooplevel);

    wined3d_mutex_lock();

    'done: {
        if ((*ddraw).flags & DDRAW_SCL_RECURSIVE) != 0 {
            WARN!("Recursive call, returning DD_OK.\n");
            hr = DD_OK;
            break 'done;
        }
        (*ddraw).flags |= DDRAW_SCL_RECURSIVE;

        /* Tests suggest that we need one of them: */
        if (cooplevel
            & (DDSCL_SETFOCUSWINDOW | DDSCL_NORMAL | DDSCL_EXCLUSIVE))
            == 0
        {
            TRACE!("Incorrect cooplevel flags, returning DDERR_INVALIDPARAMS\n");
            hr = DDERR_INVALIDPARAMS;
            break 'done;
        }

        if (cooplevel & DDSCL_CREATEDEVICEWINDOW) != 0 && (cooplevel & DDSCL_EXCLUSIVE) == 0 {
            WARN!("DDSCL_CREATEDEVICEWINDOW requires DDSCL_EXCLUSIVE.\n");
            hr = DDERR_INVALIDPARAMS;
            break 'done;
        }

        /* Handle those levels first which set various hwnds */
        if (cooplevel & DDSCL_SETFOCUSWINDOW) != 0 && (cooplevel & DDSCL_CREATEDEVICEWINDOW) == 0 {
            /* This isn't compatible with a lot of flags */
            if (cooplevel
                & (DDSCL_MULTITHREADED
                    | DDSCL_FPUSETUP
                    | DDSCL_FPUPRESERVE
                    | DDSCL_ALLOWREBOOT
                    | DDSCL_ALLOWMODEX
                    | DDSCL_SETDEVICEWINDOW
                    | DDSCL_NORMAL
                    | DDSCL_EXCLUSIVE
                    | DDSCL_FULLSCREEN))
                != 0
            {
                WARN!("Called with incompatible flags, returning DDERR_INVALIDPARAMS.\n");
                hr = DDERR_INVALIDPARAMS;
                break 'done;
            }

            hr = ddraw_set_focus_window(ddraw, window);
            break 'done;
        }

        if (cooplevel & DDSCL_EXCLUSIVE) != 0 {
            if (cooplevel & DDSCL_FULLSCREEN) == 0
                || !(!window.is_null() || (cooplevel & DDSCL_CREATEDEVICEWINDOW) != 0)
            {
                WARN!("DDSCL_EXCLUSIVE requires DDSCL_FULLSCREEN and a window.\n");
                hr = DDERR_INVALIDPARAMS;
                break 'done;
            }

            if (cooplevel & DDSCL_CREATEDEVICEWINDOW) != 0 {
                let mut device_window: HWND = core::ptr::null_mut();

                if (*ddraw).focuswindow.is_null() && (cooplevel & DDSCL_SETFOCUSWINDOW) == 0 {
                    WARN!("No focus window set.\n");
                    hr = DDERR_NOFOCUSWINDOW;
                    break 'done;
                }

                device_window = CreateWindowExA(
                    0,
                    (DDRAW_WINDOW_CLASS_NAME).as_ptr(),
                    b"DirectDrawDeviceWnd\0".as_ptr() as LPCSTR,
                    WS_POPUP,
                    0,
                    0,
                    GetSystemMetrics((SM_CXSCREEN as i32)),
                    GetSystemMetrics((SM_CYSCREEN as i32)),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if device_window.is_null() {
                    ERR!(
                        "Failed to create window, last error %#lx.\n",
                        GetLastError(),
                    );
                    hr = E_FAIL;
                    break 'done;
                }

                ShowWindow(device_window, (SW_SHOW as i32));
                TRACE!("Created a device window %p.\n", device_window);

                /* Native apparently leaks the created device window if setting the
                 * focus window below fails. */
                (*ddraw).cooperative_level |= DDSCL_CREATEDEVICEWINDOW;
                (*ddraw).devicewindow = device_window;

                if (cooplevel & DDSCL_SETFOCUSWINDOW) != 0 {
                    if window.is_null() {
                        hr = DDERR_NOHWND;
                        break 'done;
                    }

                    hr = ddraw_set_focus_window(ddraw, window);
                    if hr < 0 {
                        break 'done;
                    }
                }

                window = device_window;
            }
        } else {
            if ((*ddraw).cooperative_level & DDSCL_CREATEDEVICEWINDOW) != 0 {
                DestroyWindow((*ddraw).devicewindow);
            }
            (*ddraw).devicewindow = core::ptr::null_mut();
            (*ddraw).focuswindow = core::ptr::null_mut();
        }

        if (cooplevel & DDSCL_EXCLUSIVE) != 0 && exclusive_window != window {
            (*ddraw).device_state = (DDRAW_DEVICE_STATE_NOT_RESTORED) as i32;
            exclusive_window = window;
        }

        if (cooplevel & DDSCL_MULTITHREADED) != 0
            && ((*ddraw).cooperative_level & DDSCL_MULTITHREADED) == 0
        {
            wined3d_device_set_multithreaded((*ddraw).wined3d_device);
        }

        if !(*ddraw).wined3d_swapchain.is_null() {
            if ((*ddraw).flags & DDRAW_NO3D) == 0 {
                restore_state = (TRUE as i32);

                device = ((*ddraw).d3ddevice_list.next as *mut u8).offset(
                    -(core::mem::offset_of!(d3d_device, ddraw_entry) as isize),
                ) as *mut d3d_device;
                while core::ptr::addr_of_mut!((*device).ddraw_entry)
                    != core::ptr::addr_of_mut!((*ddraw).d3ddevice_list)
                {
                    hr = wined3d_stateblock_create(
                        (*ddraw).wined3d_device,
                        (*device).state,
                        WINED3D_SBT_ALL,
                        core::ptr::addr_of_mut!((*device).saved_state),
                    );
                    if hr < 0 {
                        let mut entry: *mut list = core::ptr::null_mut();

                        ERR!("Failed to create stateblock, hr %#lx.\n", hr);
                        entry = core::ptr::addr_of_mut!((*device).ddraw_entry);
                        loop {
                            entry = list_prev(
                                core::ptr::addr_of_mut!((*ddraw).d3ddevice_list),
                                entry,
                            );
                            if entry.is_null() {
                                break;
                            }
                            device = (entry as *mut u8).offset(
                                -(core::mem::offset_of!(d3d_device, ddraw_entry) as isize),
                            ) as *mut d3d_device;
                            wined3d_stateblock_decref((*device).saved_state);
                            (*device).saved_state = core::ptr::null_mut();
                        }
                        break 'done;
                    }
                    device = ((*device).ddraw_entry.next as *mut u8).offset(
                        -(core::mem::offset_of!(d3d_device, ddraw_entry) as isize),
                    ) as *mut d3d_device;
                }
            }

            ddraw_destroy_swapchain(ddraw);
        }

        hr = ddraw_create_swapchain(ddraw, window, cooplevel);
        if hr < 0 {
            ERR!("Failed to create swapchain, hr %#lx.\n", hr);
        }

        if restore_state != 0 {
            device = ((*ddraw).d3ddevice_list.next as *mut u8).offset(
                -(core::mem::offset_of!(d3d_device, ddraw_entry) as isize),
            ) as *mut d3d_device;
            while core::ptr::addr_of_mut!((*device).ddraw_entry)
                != core::ptr::addr_of_mut!((*ddraw).d3ddevice_list)
            {
                wined3d_stateblock_apply((*device).saved_state, (*device).state);
                wined3d_stateblock_decref((*device).saved_state);
                (*device).saved_state = core::ptr::null_mut();
                device = ((*device).ddraw_entry.next as *mut u8).offset(
                    -(core::mem::offset_of!(d3d_device, ddraw_entry) as isize),
                ) as *mut d3d_device;
            }
        }

        if (cooplevel & DDSCL_EXCLUSIVE) == 0
            && ((*ddraw).cooperative_level & DDSCL_EXCLUSIVE) != 0
        {
            /* When going from exclusive mode to normal, ddraw removes the
               topmost bit unless the DDSCL_NOWINDOWCHANGES flag is set in
               this call that sets it to normal, not in the old coop level. */
            if (cooplevel & DDSCL_NOWINDOWCHANGES) == 0 {
                SetWindowPos(
                    window,
                    HWND_NOTOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }

            if restore_mode_on_normal != 0
                && (ddraw7_RestoreDisplayMode(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface)))
                    < 0
            {
                ERR!("RestoreDisplayMode failed\n");
            }
            ClipCursor(core::ptr::null());
        }

        if ((*ddraw).cooperative_level & DDSCL_EXCLUSIVE) != 0
            && (window != (*ddraw).dest_window || (cooplevel & DDSCL_EXCLUSIVE) == 0)
        {
            wined3d_device_release_focus_window((*ddraw).wined3d_device);
        }

        if (cooplevel & DDSCL_EXCLUSIVE) != 0
            && (window != (*ddraw).dest_window
                || ((*ddraw).cooperative_level & DDSCL_EXCLUSIVE) == 0)
        {
            hr = wined3d_device_acquire_focus_window((*ddraw).wined3d_device, window);
            if hr < 0 {
                ERR!("Failed to acquire focus window, hr %#lx.\n", hr);
                break 'done;
            }
        }

        if (cooplevel & DDSCL_EXCLUSIVE) != 0 {
            SetRect(
                core::ptr::addr_of_mut!(clip_rect),
                0,
                0,
                GetSystemMetrics((SM_CXSCREEN as i32)),
                GetSystemMetrics((SM_CYSCREEN as i32)),
            );
            ClipCursor(core::ptr::addr_of_mut!(clip_rect));
        }

        /* Unhandled flags */
        if (cooplevel & DDSCL_ALLOWREBOOT) != 0 {
            WARN!("Unhandled flag DDSCL_ALLOWREBOOT, harmless\n");
        }
        if (cooplevel & DDSCL_ALLOWMODEX) != 0 {
            WARN!("Unhandled flag DDSCL_ALLOWMODEX, harmless\n");
        }
        if (cooplevel & DDSCL_FPUSETUP) != 0 {
            WARN!("Unhandled flag DDSCL_FPUSETUP, harmless\n");
        }

        if (cooplevel & DDSCL_EXCLUSIVE) != 0 {
            exclusive_ddraw = ddraw as *const ddraw;
        } else if exclusive_ddraw == ddraw as *const ddraw {
            exclusive_ddraw = core::ptr::null();
        }

        /* Store the cooperative_level */
        (*ddraw).cooperative_level = cooplevel;
        (*ddraw).dest_window = window;

        TRACE!("SetCooperativeLevel returning DD_OK\n");
        hr = DD_OK;
    }
    (*ddraw).flags &= !DDRAW_SCL_RECURSIVE;
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn ddraw7_SetCooperativeLevel(iface: *mut IDirectDraw7, window: HWND, flags: DWORD) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    TRACE!("iface %p, window %p, flags %#lx.\n", iface, window, flags);
    return ddraw_set_cooperative_level(ddraw, window, flags, (((*ddraw).flags & DDRAW_SCL_DDRAW1) == 0) as i32);
}

unsafe extern "system" fn ddraw4_SetCooperativeLevel(
    mut iface: *mut IDirectDraw4,
    mut window: HWND,
    mut flags: DWORD,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);

    return ddraw_set_cooperative_level(
        ddraw,
        window,
        flags,
        (((*ddraw).flags & DDRAW_SCL_DDRAW1) == 0) as BOOL,
    );
}

unsafe extern "system" fn ddraw2_SetCooperativeLevel(iface: *mut IDirectDraw2, window: HWND, flags: DWORD) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);

    return ddraw_set_cooperative_level(ddraw, window, flags, (((*ddraw).flags & (DDRAW_SCL_DDRAW1 as DWORD)) == 0) as i32);
}

unsafe extern "system" fn ddraw1_SetCooperativeLevel(iface: *mut IDirectDraw, window: HWND, flags: DWORD) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    let mut hr: HRESULT;
    WINE_TRACE!("iface %p, window %p, flags %#lx.\n", iface, window, flags);
    hr = ddraw_set_cooperative_level(ddraw, window, flags, (FALSE) as BOOL);
    if hr >= 0 {
        (*ddraw).flags |= DDRAW_SCL_DDRAW1 as DWORD;
    }
    return hr;
}

unsafe extern "system" fn ddraw7_SetDisplayMode(iface: *mut IDirectDraw7, width: DWORD, height: DWORD, bpp: DWORD, mut refresh_rate: DWORD, flags: DWORD) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    let mut format: wined3d_format_id = WINED3DFMT_UNKNOWN;
    let mut clip_rect: RECT = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, width %lu, height %lu, bpp %lu, refresh_rate %lu, flags %#lx.\n", iface, width, height, bpp, refresh_rate, flags);

    if force_refresh_rate != 0
    {
        TRACE!("ForceRefreshRate overriding passed-in refresh rate (%lu Hz) to %lu Hz\n", refresh_rate, force_refresh_rate);
        refresh_rate = force_refresh_rate;
    }

    wined3d_mutex_lock();

    if !exclusive_ddraw.is_null() && exclusive_ddraw != ddraw as *const ddraw
    {
        wined3d_mutex_unlock();
        return DDERR_NOEXCLUSIVEMODE;
    }

    if width == 0 || height == 0
    {
        /* It looks like Need for Speed Porsche Unleashed expects DD_OK here. */
        wined3d_mutex_unlock();
        return DD_OK;
    }

    match bpp {
        8 => { format = WINED3DFMT_P8_UINT; }
        15 => { format = WINED3DFMT_B5G5R5X1_UNORM; }
        16 => { format = WINED3DFMT_B5G6R5_UNORM; }
        24 => { format = WINED3DFMT_B8G8R8_UNORM; }
        32 => { format = WINED3DFMT_B8G8R8X8_UNORM; }
        _ => { format = WINED3DFMT_UNKNOWN; }
    }

    mode.width = width;
    mode.height = height;
    mode.refresh_rate = refresh_rate;
    mode.format_id = format;
    mode.scanline_ordering = WINED3D_SCANLINE_ORDERING_UNKNOWN;

    /* TODO: The possible return values from msdn suggest that the screen mode
     * can't be changed if a surface is locked or some drawing is in progress. */
    hr = wined3d_output_set_display_mode((*ddraw).wined3d_output, core::ptr::addr_of_mut!(mode));
    if hr >= 0
    {
        if !(*ddraw).primary.is_null()
        {
            let surface_desc: *mut DDSURFACEDESC2 = core::ptr::addr_of_mut!((*(*ddraw).primary).surface_desc);

            hr = wined3d_swapchain_resize_buffers((*ddraw).wined3d_swapchain, 0,
                    (*surface_desc).dwWidth, (*surface_desc).dwHeight, mode.format_id, WINED3D_MULTISAMPLE_NONE, 0);
            if hr < 0
            {
                ERR!("Failed to resize buffers, hr %#lx.\n", hr);
            }
            else
            {
                ddrawformat_from_wined3dformat(core::ptr::addr_of_mut!((*(*ddraw).primary).surface_desc.__bindgen_anon_5.ddpfPixelFormat), mode.format_id);
            }
        }
        (*ddraw).flags |= DDRAW_RESTORE_MODE;

        if ((*ddraw).cooperative_level & DDSCL_EXCLUSIVE) != 0
        {
            SetRect(core::ptr::addr_of_mut!(clip_rect), 0, 0, width as i32, height as i32);
            ClipCursor(core::ptr::addr_of_mut!(clip_rect) as LPCRECT);
        }
    }

    InterlockedCompareExchange(core::ptr::addr_of_mut!((*ddraw).device_state), (DDRAW_DEVICE_STATE_NOT_RESTORED) as i32, (DDRAW_DEVICE_STATE_OK) as i32);

    wined3d_mutex_unlock();

    return hr_ddraw_from_wined3d(hr);
}

unsafe extern "system" fn ddraw4_SetDisplayMode(iface: *mut IDirectDraw4, width: DWORD, height: DWORD, bpp: DWORD, refresh_rate: DWORD, flags: DWORD) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    WINE_TRACE!("iface %p, width %lu, height %lu, bpp %lu, refresh_rate %lu, flags %#lx.\n", iface, width, height, bpp, refresh_rate, flags);
    return ddraw7_SetDisplayMode(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        width,
        height,
        bpp,
        refresh_rate,
        flags,
    );
}

unsafe extern "system" fn ddraw2_SetDisplayMode(
    iface: *mut IDirectDraw2,
    width: DWORD,
    height: DWORD,
    bpp: DWORD,
    refresh_rate: DWORD,
    flags: DWORD,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    WINE_TRACE!("iface %p, width %lu, height %lu, bpp %lu, refresh_rate %lu, flags %#lx.\n", iface, width, height, bpp, refresh_rate, flags);
    return ddraw7_SetDisplayMode(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        width,
        height,
        bpp,
        refresh_rate,
        flags,
    );
}
unsafe extern "system" fn ddraw1_SetDisplayMode(iface: *mut IDirectDraw, width: DWORD, height: DWORD, bpp: DWORD) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);

    return ddraw7_SetDisplayMode(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), width, height, bpp, 0, 0);
}
#[no_mangle]
pub unsafe extern "C" fn ddraw_d3dcaps1_from_7(
    mut caps1: *mut D3DDEVICEDESC,
    mut caps7: *mut D3DDEVICEDESC7,
) {
    memset(
        caps1 as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<D3DDEVICEDESC>() as size_t,
    );
    (*caps1).dwSize = ::core::mem::size_of::<D3DDEVICEDESC>() as DWORD;
    (*caps1).dwFlags = (D3DDD_COLORMODEL
        | D3DDD_DEVCAPS
        | D3DDD_TRANSFORMCAPS
        | D3DDD_BCLIPPING
        | D3DDD_LIGHTINGCAPS
        | D3DDD_LINECAPS
        | D3DDD_TRICAPS
        | D3DDD_DEVICERENDERBITDEPTH
        | D3DDD_DEVICEZBUFFERBITDEPTH
        | D3DDD_MAXBUFFERSIZE
        | D3DDD_MAXVERTEXCOUNT) as DWORD;
    (*caps1).dcmColorModel = D3DCOLOR_RGB as D3DCOLORMODEL;
    (*caps1).dwDevCaps = (*caps7).dwDevCaps;
    (*caps1).dtcTransformCaps.dwSize = ::core::mem::size_of::<D3DTRANSFORMCAPS>() as DWORD;
    (*caps1).dtcTransformCaps.dwCaps = D3DTRANSFORMCAPS_CLIP as DWORD;
    (*caps1).bClipping = TRUE as BOOL;
    (*caps1).dlcLightingCaps.dwSize = ::core::mem::size_of::<D3DLIGHTINGCAPS>() as DWORD;
    (*caps1).dlcLightingCaps.dwCaps = (D3DLIGHTCAPS_DIRECTIONAL
        | D3DLIGHTCAPS_PARALLELPOINT
        | D3DLIGHTCAPS_POINT
        | D3DLIGHTCAPS_SPOT) as DWORD;
    (*caps1).dlcLightingCaps.dwLightingModel = D3DLIGHTINGMODEL_RGB as DWORD;
    (*caps1).dlcLightingCaps.dwNumLights = (*caps7).dwMaxActiveLights;
    (*caps1).dpcLineCaps = (*caps7).dpcLineCaps;
    (*caps1).dpcTriCaps = (*caps7).dpcTriCaps;
    (*caps1).dwDeviceRenderBitDepth = (*caps7).dwDeviceRenderBitDepth;
    (*caps1).dwDeviceZBufferBitDepth = (*caps7).dwDeviceZBufferBitDepth;

    /* This value is zero on all Windows drivers we have seen so far. */
    (*caps1).dwMaxBufferSize = 0 as DWORD;

    /* AMD GPUs on Windows XP and newer report 1024 for HAL devices.
     * Nvidia GPUs report 65534. On Windows 9x we have seen 2048 on
     * AMD cards and 32768 on Nvidia. The API defined limit is 65536
     * because ddraw only supports 16 bit indices.
     *
     * Some games, for example Prince of Persia 3D (bug 44863), create a
     * vertex buffer to hold as many vertices as we report here. The
     * game requests a video memory buffer, maps it, writes a handful
     * of vertices and draws. Because IDirect3DVertexBuffer::Lock does
     * not allow passing a range, and prior to d3d7 doesn't support
     * NOOVERWRITE/DISCARD, the upload becomes slow if the buffer is too
     * large. So we don't want to report a value that's too high here.
     *
     * Why 2048? Windows 9x was the contemporaneous system when d3d 1-4
     * titles were written.
     *
     * Regardless of this value, we put ddraw4 vertex buffers into
     * system memory because games are using them poorly. This mitigates
     * negative effects of a large vertex count.
     *
     * Windows doesn't enforce this limit. A larger vertex buffer can
     * be created and used in draws. More vertices than dwMaxVertexCount
     * can be rendered in one draw call. SetExecuteData with a vertex
     * count exceeding dwMaxVertexCount succeeds too. */
    (*caps1).dwMaxVertexCount = 2048 as DWORD;

    (*caps1).dwMinTextureWidth = (*caps7).dwMinTextureWidth;
    (*caps1).dwMinTextureHeight = (*caps7).dwMinTextureHeight;
    (*caps1).dwMaxTextureWidth = (*caps7).dwMaxTextureWidth;
    (*caps1).dwMaxTextureHeight = (*caps7).dwMaxTextureHeight;
    (*caps1).dwMinStippleWidth = 1 as DWORD;
    (*caps1).dwMinStippleHeight = 1 as DWORD;
    (*caps1).dwMaxStippleWidth = 32 as DWORD;
    (*caps1).dwMaxStippleHeight = 32 as DWORD;
    (*caps1).dwMaxTextureRepeat = (*caps7).dwMaxTextureRepeat;
    (*caps1).dwMaxTextureAspectRatio = (*caps7).dwMaxTextureAspectRatio;
    (*caps1).dwMaxAnisotropy = (*caps7).dwMaxAnisotropy;
    (*caps1).dvGuardBandLeft = (*caps7).dvGuardBandLeft;
    (*caps1).dvGuardBandTop = (*caps7).dvGuardBandTop;
    (*caps1).dvGuardBandRight = (*caps7).dvGuardBandRight;
    (*caps1).dvGuardBandBottom = (*caps7).dvGuardBandBottom;
    (*caps1).dvExtentsAdjust = (*caps7).dvExtentsAdjust;
    (*caps1).dwStencilCaps = (*caps7).dwStencilCaps;
    (*caps1).dwFVFCaps = (*caps7).dwFVFCaps;
    (*caps1).dwTextureOpCaps = (*caps7).dwTextureOpCaps;
    (*caps1).wMaxTextureBlendStages = (*caps7).wMaxTextureBlendStages;
    (*caps1).wMaxSimultaneousTextures = (*caps7).wMaxSimultaneousTextures;
}
#[no_mangle]
pub unsafe extern "C" fn ddraw_get_d3dcaps(mut ddraw: *const ddraw, mut caps: *mut D3DDEVICEDESC7) -> HRESULT {
    let mut wined3d_caps: wined3d_caps = core::mem::zeroed();
    let mut hr: HRESULT;

    memset(
        core::ptr::addr_of_mut!(wined3d_caps) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<wined3d_caps>(),
    );

    wined3d_mutex_lock();
    hr = wined3d_get_device_caps(
        (*ddraw).wined3d_adapter,
        WINED3D_DEVICE_TYPE_HAL,
        core::ptr::addr_of_mut!(wined3d_caps),
    );
    wined3d_mutex_unlock();
    if hr < 0 as HRESULT {
        WARN!("Failed to get device caps, hr %#lx.\n", hr);
        return hr;
    }

    (*caps).dwDevCaps = wined3d_caps.DevCaps;
    (*caps).dpcLineCaps.dwMiscCaps = wined3d_caps.PrimitiveMiscCaps;
    (*caps).dpcLineCaps.dwRasterCaps = wined3d_caps.RasterCaps;
    (*caps).dpcLineCaps.dwZCmpCaps = wined3d_caps.ZCmpCaps;
    (*caps).dpcLineCaps.dwSrcBlendCaps = wined3d_caps.SrcBlendCaps;
    (*caps).dpcLineCaps.dwDestBlendCaps = wined3d_caps.DestBlendCaps;
    (*caps).dpcLineCaps.dwAlphaCmpCaps = wined3d_caps.AlphaCmpCaps;
    (*caps).dpcLineCaps.dwShadeCaps = wined3d_caps.ShadeCaps;
    (*caps).dpcLineCaps.dwTextureCaps = wined3d_caps.TextureCaps;
    (*caps).dpcLineCaps.dwTextureFilterCaps = wined3d_caps.TextureFilterCaps;
    (*caps).dpcLineCaps.dwTextureAddressCaps = wined3d_caps.TextureAddressCaps;

    (*caps).dwMaxTextureWidth = wined3d_caps.MaxTextureWidth;
    (*caps).dwMaxTextureHeight = wined3d_caps.MaxTextureHeight;

    (*caps).dwMaxTextureRepeat = wined3d_caps.MaxTextureRepeat;
    (*caps).dwMaxTextureAspectRatio = wined3d_caps.MaxTextureAspectRatio;
    (*caps).dwMaxAnisotropy = wined3d_caps.MaxAnisotropy;
    (*caps).dvMaxVertexW = wined3d_caps.MaxVertexW as D3DVALUE;

    (*caps).dvGuardBandLeft = wined3d_caps.GuardBandLeft as D3DVALUE;
    (*caps).dvGuardBandTop = wined3d_caps.GuardBandTop as D3DVALUE;
    (*caps).dvGuardBandRight = wined3d_caps.GuardBandRight as D3DVALUE;
    (*caps).dvGuardBandBottom = wined3d_caps.GuardBandBottom as D3DVALUE;

    (*caps).dvExtentsAdjust = wined3d_caps.ExtentsAdjust as D3DVALUE;
    (*caps).dwStencilCaps = wined3d_caps.StencilCaps;

    (*caps).dwFVFCaps = wined3d_caps.FVFCaps;
    (*caps).dwTextureOpCaps = wined3d_caps.TextureOpCaps;

    (*caps).dwVertexProcessingCaps = wined3d_caps.VertexProcessingCaps;
    (*caps).dwMaxActiveLights = wined3d_caps.MaxActiveLights;

    /* Remove all non-d3d7 caps */
    (*caps).dwDevCaps &= (D3DDEVCAPS_FLOATTLVERTEX
        | D3DDEVCAPS_SORTINCREASINGZ
        | D3DDEVCAPS_SORTDECREASINGZ
        | D3DDEVCAPS_SORTEXACT
        | D3DDEVCAPS_EXECUTESYSTEMMEMORY
        | D3DDEVCAPS_EXECUTEVIDEOMEMORY
        | D3DDEVCAPS_TLVERTEXSYSTEMMEMORY
        | D3DDEVCAPS_TLVERTEXVIDEOMEMORY
        | D3DDEVCAPS_TEXTURESYSTEMMEMORY
        | D3DDEVCAPS_TEXTUREVIDEOMEMORY
        | D3DDEVCAPS_DRAWPRIMTLVERTEX
        | D3DDEVCAPS_CANRENDERAFTERFLIP
        | D3DDEVCAPS_TEXTURENONLOCALVIDMEM
        | D3DDEVCAPS_DRAWPRIMITIVES2
        | D3DDEVCAPS_SEPARATETEXTUREMEMORIES
        | D3DDEVCAPS_DRAWPRIMITIVES2EX
        | D3DDEVCAPS_HWTRANSFORMANDLIGHT
        | D3DDEVCAPS_CANBLTSYSTONONLOCAL
        | D3DDEVCAPS_HWRASTERIZATION) as DWORD;

    (*caps).dwStencilCaps &= (D3DSTENCILCAPS_KEEP
        | D3DSTENCILCAPS_ZERO
        | D3DSTENCILCAPS_REPLACE
        | D3DSTENCILCAPS_INCRSAT
        | D3DSTENCILCAPS_DECRSAT
        | D3DSTENCILCAPS_INVERT
        | D3DSTENCILCAPS_INCR
        | D3DSTENCILCAPS_DECR) as DWORD;

    /* FVF caps ?*/

    (*caps).dwTextureOpCaps &= (D3DTEXOPCAPS_DISABLE
        | D3DTEXOPCAPS_SELECTARG1
        | D3DTEXOPCAPS_SELECTARG2
        | D3DTEXOPCAPS_MODULATE
        | D3DTEXOPCAPS_MODULATE2X
        | D3DTEXOPCAPS_MODULATE4X
        | D3DTEXOPCAPS_ADD
        | D3DTEXOPCAPS_ADDSIGNED
        | D3DTEXOPCAPS_ADDSIGNED2X
        | D3DTEXOPCAPS_SUBTRACT
        | D3DTEXOPCAPS_ADDSMOOTH
        | D3DTEXOPCAPS_BLENDTEXTUREALPHA
        | D3DTEXOPCAPS_BLENDFACTORALPHA
        | D3DTEXOPCAPS_BLENDTEXTUREALPHAPM
        | D3DTEXOPCAPS_BLENDCURRENTALPHA
        | D3DTEXOPCAPS_PREMODULATE
        | D3DTEXOPCAPS_MODULATEALPHA_ADDCOLOR
        | D3DTEXOPCAPS_MODULATECOLOR_ADDALPHA
        | D3DTEXOPCAPS_MODULATEINVALPHA_ADDCOLOR
        | D3DTEXOPCAPS_MODULATEINVCOLOR_ADDALPHA
        | D3DTEXOPCAPS_BUMPENVMAP
        | D3DTEXOPCAPS_BUMPENVMAPLUMINANCE
        | D3DTEXOPCAPS_DOTPRODUCT3) as DWORD;

    (*caps).dwVertexProcessingCaps &= (D3DVTXPCAPS_TEXGEN
        | D3DVTXPCAPS_MATERIALSOURCE7
        | D3DVTXPCAPS_VERTEXFOG
        | D3DVTXPCAPS_DIRECTIONALLIGHTS
        | D3DVTXPCAPS_POSITIONALLIGHTS
        | D3DVTXPCAPS_LOCALVIEWER) as DWORD;

    (*caps).dpcLineCaps.dwMiscCaps &= (D3DPMISCCAPS_MASKPLANES
        | D3DPMISCCAPS_MASKZ
        | D3DPMISCCAPS_LINEPATTERNREP
        | D3DPMISCCAPS_CONFORMANT
        | D3DPMISCCAPS_CULLNONE
        | D3DPMISCCAPS_CULLCW
        | D3DPMISCCAPS_CULLCCW) as DWORD;

    (*caps).dpcLineCaps.dwRasterCaps &= (D3DPRASTERCAPS_DITHER
        | D3DPRASTERCAPS_ROP2
        | D3DPRASTERCAPS_XOR
        | D3DPRASTERCAPS_PAT
        | D3DPRASTERCAPS_ZTEST
        | D3DPRASTERCAPS_SUBPIXEL
        | D3DPRASTERCAPS_SUBPIXELX
        | D3DPRASTERCAPS_FOGVERTEX
        | D3DPRASTERCAPS_FOGTABLE
        | D3DPRASTERCAPS_STIPPLE
        | D3DPRASTERCAPS_ANTIALIASSORTDEPENDENT
        | D3DPRASTERCAPS_ANTIALIASSORTINDEPENDENT
        | D3DPRASTERCAPS_ANTIALIASEDGES
        | D3DPRASTERCAPS_MIPMAPLODBIAS
        | D3DPRASTERCAPS_ZBUFFERLESSHSR
        | D3DPRASTERCAPS_FOGRANGE
        | D3DPRASTERCAPS_ANISOTROPY
        | D3DPRASTERCAPS_WBUFFER
        | D3DPRASTERCAPS_TRANSLUCENTSORTINDEPENDENT
        | D3DPRASTERCAPS_WFOG
        | D3DPRASTERCAPS_ZFOG
        | WINED3DPRASTERCAPS_DEPTHBIAS) as DWORD;
    if ((*caps).dpcLineCaps.dwRasterCaps & (WINED3DPRASTERCAPS_DEPTHBIAS as DWORD)) != 0 {
        (*caps).dpcLineCaps.dwRasterCaps = ((*caps).dpcLineCaps.dwRasterCaps
            | (D3DPRASTERCAPS_ZBIAS as DWORD))
            & !(WINED3DPRASTERCAPS_DEPTHBIAS as DWORD);
    }
    if (wined3d_caps.LineCaps & (WINED3DLINECAPS_ANTIALIAS as DWORD)) != 0 {
        (*caps).dpcLineCaps.dwRasterCaps |= D3DPRASTERCAPS_ANTIALIASEDGES as DWORD;
    }

    (*caps).dpcLineCaps.dwZCmpCaps &= (D3DPCMPCAPS_NEVER
        | D3DPCMPCAPS_LESS
        | D3DPCMPCAPS_EQUAL
        | D3DPCMPCAPS_LESSEQUAL
        | D3DPCMPCAPS_GREATER
        | D3DPCMPCAPS_NOTEQUAL
        | D3DPCMPCAPS_GREATEREQUAL
        | D3DPCMPCAPS_ALWAYS) as DWORD;

    (*caps).dpcLineCaps.dwSrcBlendCaps &= (D3DPBLENDCAPS_ZERO
        | D3DPBLENDCAPS_ONE
        | D3DPBLENDCAPS_SRCCOLOR
        | D3DPBLENDCAPS_INVSRCCOLOR
        | D3DPBLENDCAPS_SRCALPHA
        | D3DPBLENDCAPS_INVSRCALPHA
        | D3DPBLENDCAPS_DESTALPHA
        | D3DPBLENDCAPS_INVDESTALPHA
        | D3DPBLENDCAPS_DESTCOLOR
        | D3DPBLENDCAPS_INVDESTCOLOR
        | D3DPBLENDCAPS_SRCALPHASAT
        | D3DPBLENDCAPS_BOTHSRCALPHA
        | D3DPBLENDCAPS_BOTHINVSRCALPHA) as DWORD;

    (*caps).dpcLineCaps.dwDestBlendCaps &= (D3DPBLENDCAPS_ZERO
        | D3DPBLENDCAPS_ONE
        | D3DPBLENDCAPS_SRCCOLOR
        | D3DPBLENDCAPS_INVSRCCOLOR
        | D3DPBLENDCAPS_SRCALPHA
        | D3DPBLENDCAPS_INVSRCALPHA
        | D3DPBLENDCAPS_DESTALPHA
        | D3DPBLENDCAPS_INVDESTALPHA
        | D3DPBLENDCAPS_DESTCOLOR
        | D3DPBLENDCAPS_INVDESTCOLOR
        | D3DPBLENDCAPS_SRCALPHASAT
        | D3DPBLENDCAPS_BOTHSRCALPHA
        | D3DPBLENDCAPS_BOTHINVSRCALPHA) as DWORD;

    (*caps).dpcLineCaps.dwAlphaCmpCaps &= (D3DPCMPCAPS_NEVER
        | D3DPCMPCAPS_LESS
        | D3DPCMPCAPS_EQUAL
        | D3DPCMPCAPS_LESSEQUAL
        | D3DPCMPCAPS_GREATER
        | D3DPCMPCAPS_NOTEQUAL
        | D3DPCMPCAPS_GREATEREQUAL
        | D3DPCMPCAPS_ALWAYS) as DWORD;

    (*caps).dpcLineCaps.dwShadeCaps &= (D3DPSHADECAPS_COLORFLATMONO
        | D3DPSHADECAPS_COLORFLATRGB
        | D3DPSHADECAPS_COLORGOURAUDMONO
        | D3DPSHADECAPS_COLORGOURAUDRGB
        | D3DPSHADECAPS_COLORPHONGMONO
        | D3DPSHADECAPS_COLORPHONGRGB
        | D3DPSHADECAPS_SPECULARFLATMONO
        | D3DPSHADECAPS_SPECULARFLATRGB
        | D3DPSHADECAPS_SPECULARGOURAUDMONO
        | D3DPSHADECAPS_SPECULARGOURAUDRGB
        | D3DPSHADECAPS_SPECULARPHONGMONO
        | D3DPSHADECAPS_SPECULARPHONGRGB
        | D3DPSHADECAPS_ALPHAFLATBLEND
        | D3DPSHADECAPS_ALPHAFLATSTIPPLED
        | D3DPSHADECAPS_ALPHAGOURAUDBLEND
        | D3DPSHADECAPS_ALPHAGOURAUDSTIPPLED
        | D3DPSHADECAPS_ALPHAPHONGBLEND
        | D3DPSHADECAPS_ALPHAPHONGSTIPPLED
        | D3DPSHADECAPS_FOGFLAT
        | D3DPSHADECAPS_FOGGOURAUD
        | D3DPSHADECAPS_FOGPHONG) as DWORD;

    (*caps).dpcLineCaps.dwTextureCaps &= (D3DPTEXTURECAPS_PERSPECTIVE
        | D3DPTEXTURECAPS_POW2
        | D3DPTEXTURECAPS_ALPHA
        | D3DPTEXTURECAPS_TRANSPARENCY
        | D3DPTEXTURECAPS_BORDER
        | D3DPTEXTURECAPS_SQUAREONLY
        | D3DPTEXTURECAPS_TEXREPEATNOTSCALEDBYSIZE
        | D3DPTEXTURECAPS_ALPHAPALETTE
        | (D3DPTEXTURECAPS_NONPOW2CONDITIONAL as u32)
        | D3DPTEXTURECAPS_PROJECTED
        | D3DPTEXTURECAPS_CUBEMAP
        | D3DPTEXTURECAPS_COLORKEYBLEND) as DWORD;

    (*caps).dpcLineCaps.dwTextureFilterCaps &= (D3DPTFILTERCAPS_NEAREST
        | D3DPTFILTERCAPS_LINEAR
        | D3DPTFILTERCAPS_MIPNEAREST
        | D3DPTFILTERCAPS_MIPLINEAR
        | D3DPTFILTERCAPS_LINEARMIPNEAREST
        | D3DPTFILTERCAPS_LINEARMIPLINEAR
        | D3DPTFILTERCAPS_MINFPOINT
        | D3DPTFILTERCAPS_MINFLINEAR
        | D3DPTFILTERCAPS_MINFANISOTROPIC
        | D3DPTFILTERCAPS_MIPFPOINT
        | D3DPTFILTERCAPS_MIPFLINEAR
        | D3DPTFILTERCAPS_MAGFPOINT
        | D3DPTFILTERCAPS_MAGFLINEAR
        | D3DPTFILTERCAPS_MAGFANISOTROPIC
        | D3DPTFILTERCAPS_MAGFAFLATCUBIC
        | D3DPTFILTERCAPS_MAGFGAUSSIANCUBIC) as DWORD;

    (*caps).dpcLineCaps.dwTextureAddressCaps &= (D3DPTADDRESSCAPS_WRAP
        | D3DPTADDRESSCAPS_MIRROR
        | D3DPTADDRESSCAPS_CLAMP
        | D3DPTADDRESSCAPS_BORDER
        | D3DPTADDRESSCAPS_INDEPENDENTUV) as DWORD;

    if ((*caps).dpcLineCaps.dwTextureCaps & (D3DPTEXTURECAPS_POW2 as DWORD)) == 0 {
        /* DirectX7 always has the np2 flag set, no matter what the card
         * supports. Some old games (Rollcage) check the caps incorrectly.
         * If wined3d supports nonpow2 textures it also has np2 conditional
         * support. */
        (*caps).dpcLineCaps.dwTextureCaps |= (D3DPTEXTURECAPS_POW2 | (D3DPTEXTURECAPS_NONPOW2CONDITIONAL as u32)) as DWORD;
    }

    /* Fill the missing members, and do some fixup */
    (*caps).dpcLineCaps.dwSize = core::mem::size_of::<D3DPRIMCAPS>() as DWORD;
    (*caps).dpcLineCaps.dwTextureBlendCaps = (D3DPTBLENDCAPS_ADD
        | D3DPTBLENDCAPS_MODULATEMASK
        | D3DPTBLENDCAPS_COPY
        | D3DPTBLENDCAPS_DECAL
        | D3DPTBLENDCAPS_DECALALPHA
        | D3DPTBLENDCAPS_DECALMASK
        | D3DPTBLENDCAPS_MODULATE
        | D3DPTBLENDCAPS_MODULATEALPHA) as DWORD;
    (*caps).dpcLineCaps.dwStippleWidth = 32 as DWORD;
    (*caps).dpcLineCaps.dwStippleHeight = 32 as DWORD;
    /* Use the same for the TriCaps */
    (*caps).dpcTriCaps = (*caps).dpcLineCaps;

    (*caps).dwDeviceRenderBitDepth = (DDBD_16 | DDBD_24 | DDBD_32) as DWORD;
    (*caps).dwDeviceZBufferBitDepth = (DDBD_16 | DDBD_24) as DWORD;
    (*caps).dwMinTextureWidth = 1 as DWORD;
    (*caps).dwMinTextureHeight = 1 as DWORD;

    /* Convert DWORDs safely to WORDs */
    (*caps).wMaxTextureBlendStages = min!(wined3d_caps.MaxTextureBlendStages, 0xffff as DWORD) as WORD;
    (*caps).wMaxSimultaneousTextures = min!(wined3d_caps.MaxSimultaneousTextures, 0xffff as DWORD) as WORD;
    (*caps).wMaxUserClipPlanes = min!(wined3d_caps.MaxUserClipPlanes, D3DMAXUSERCLIPPLANES as DWORD) as WORD;
    (*caps).wMaxVertexBlendMatrices = min!(wined3d_caps.MaxVertexBlendMatrices, 0xffff as DWORD) as WORD;

    (*caps).deviceGUID = IID_IDirect3DTnLHalDevice;

    (*caps).dwReserved1 = 0 as DWORD;
    (*caps).dwReserved2 = 0 as DWORD;
    (*caps).dwReserved3 = 0 as DWORD;
    (*caps).dwReserved4 = 0 as DWORD;

    return DD_OK as HRESULT;
}

unsafe extern "system" fn ddraw7_GetCaps(iface: *mut IDirectDraw7, DriverCaps: *mut DDCAPS, HELCaps: *mut DDCAPS) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut ddscaps: DDSCAPS2 = DDSCAPS2 {
        dwCaps: 0,
        dwCaps2: 0,
        dwCaps3: 0,
        __bindgen_anon_1: _DDSCAPS2__bindgen_ty_1 { dwCaps4: 0,
        },
    };
    let mut winecaps: wined3d_caps = core::mem::zeroed();
    let mut caps: DDCAPS = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    /* One structure must be != NULL */
    if DriverCaps.is_null() && HELCaps.is_null() {
        WARN!("Invalid parameters.\n");
        return DDERR_INVALIDPARAMS;
    }

    memset(
        core::ptr::addr_of_mut!(caps) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<DDCAPS>() as usize,
    );
    memset(
        core::ptr::addr_of_mut!(winecaps) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<wined3d_caps>() as usize,
    );
    caps.dwSize = core::mem::size_of::<DDCAPS>() as DWORD;

    wined3d_mutex_lock();
    hr = wined3d_device_get_device_caps((*ddraw).wined3d_device, core::ptr::addr_of_mut!(winecaps));
    if hr < 0 {
        WARN!("Failed to get device caps, %#lx.\n", hr);
        wined3d_mutex_unlock();
        return hr;
    }

    hr = IDirectDraw7_GetAvailableVidMem(
        iface,
        core::ptr::addr_of_mut!(ddscaps),
        core::ptr::addr_of_mut!(caps.dwVidMemTotal),
        core::ptr::addr_of_mut!(caps.dwVidMemFree),
    );
    if hr < 0 {
        WARN!("IDirectDraw7::GetAvailableVidMem failed\n");
        wined3d_mutex_unlock();
        return hr;
    }

    hr = IDirectDraw7_GetFourCCCodes(
        iface,
        core::ptr::addr_of_mut!(caps.dwNumFourCCCodes),
        core::ptr::null_mut(),
    );
    wined3d_mutex_unlock();
    if hr < 0 {
        WARN!("IDirectDraw7::GetFourCCCodes failed\n");
        return hr;
    }

    caps.dwCaps = winecaps.ddraw_caps.caps;
    caps.dwCaps2 = winecaps.ddraw_caps.caps2;
    caps.dwCKeyCaps = winecaps.ddraw_caps.color_key_caps;
    caps.dwFXCaps = winecaps.ddraw_caps.fx_caps;
    caps.dwPalCaps = DDPCAPS_8BIT | DDPCAPS_PRIMARYSURFACE;
    caps.ddsCaps.dwCaps = winecaps.ddraw_caps.dds_caps;
    caps.dwSVBCaps = winecaps.ddraw_caps.svb_caps;
    caps.dwSVBCKeyCaps = winecaps.ddraw_caps.svb_color_key_caps;
    caps.dwSVBFXCaps = winecaps.ddraw_caps.svb_fx_caps;
    caps.dwVSBCaps = winecaps.ddraw_caps.vsb_caps;
    caps.dwVSBCKeyCaps = winecaps.ddraw_caps.vsb_color_key_caps;
    caps.dwVSBFXCaps = winecaps.ddraw_caps.vsb_fx_caps;
    caps.dwSSBCaps = winecaps.ddraw_caps.ssb_caps;
    caps.dwSSBCKeyCaps = winecaps.ddraw_caps.ssb_color_key_caps;
    caps.dwSSBFXCaps = winecaps.ddraw_caps.ssb_fx_caps;

    caps.dwCaps |= DDCAPS_ALIGNSTRIDE;
    caps.dwAlignStrideAlign = DDRAW_STRIDE_ALIGNMENT;

    caps.ddsOldCaps.dwCaps = caps.ddsCaps.dwCaps;

    if !DriverCaps.is_null() {
        DD_STRUCT_COPY_BYSIZE!(DriverCaps, core::ptr::addr_of_mut!(caps));
        if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
            DDRAW_dump_DDCAPS(DriverCaps);
        }
    }
    if !HELCaps.is_null() {
        DD_STRUCT_COPY_BYSIZE!(HELCaps, core::ptr::addr_of_mut!(caps));
        if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
            DDRAW_dump_DDCAPS(HELCaps);
        }
    }

    return DD_OK;
}

unsafe extern "system" fn ddraw4_GetCaps(iface: *mut IDirectDraw4, driver_caps: *mut DDCAPS, hel_caps: *mut DDCAPS) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);

    return ddraw7_GetCaps(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), driver_caps, hel_caps);
}

unsafe extern "system" fn ddraw2_GetCaps(iface: *mut IDirectDraw2, driver_caps: *mut DDCAPS, hel_caps: *mut DDCAPS) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);

    TRACE!("iface %p, driver_caps %p, hel_caps %p.\n", iface, driver_caps, hel_caps);

    return ddraw7_GetCaps(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), driver_caps, hel_caps);
}
unsafe extern "system" fn ddraw1_GetCaps(iface: *mut IDirectDraw, driver_caps: *mut DDCAPS, hel_caps: *mut DDCAPS) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);

    TRACE!("iface %p, driver_caps %p, hel_caps %p.\n", iface, driver_caps, hel_caps);

    return ddraw7_GetCaps(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), driver_caps, hel_caps);
}
unsafe extern "system" fn ddraw7_Compact(iface: *mut IDirectDraw7) -> HRESULT {
    // TRACE("iface %p.\n", iface);
    DD_OK
}

unsafe extern "system" fn ddraw4_Compact(iface: *mut IDirectDraw4) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return ddraw7_Compact(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface));
}
unsafe extern "system" fn ddraw2_Compact(iface: *mut IDirectDraw2) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);

    return ddraw7_Compact(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface));
}

unsafe extern "system" fn ddraw1_Compact(iface: *mut IDirectDraw) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return ddraw7_Compact(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface));
}

unsafe extern "system" fn ddraw7_GetDisplayMode(iface: *mut IDirectDraw7, DDSD: *mut DDSURFACEDESC2) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut mode: wined3d_display_mode = wined3d_display_mode {
        width: 0,
        height: 0,
        refresh_rate: 0,
        format_id: WINED3DFMT_UNKNOWN,
        scanline_ordering: WINED3D_SCANLINE_ORDERING_UNKNOWN,
    };
    let mut hr: HRESULT = 0;

    /* This seems sane */
    if DDSD.is_null()
        || ((*DDSD).dwSize as usize != core::mem::size_of::<DDSURFACEDESC>()
            && (*DDSD).dwSize as usize != core::mem::size_of::<DDSURFACEDESC2>())
    {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    hr = wined3d_output_get_display_mode((*ddraw).wined3d_output, &raw mut mode, core::ptr::null_mut());
    if hr < 0 {
        ERR!("Failed to get display mode, hr %#lx.\n", hr);
        wined3d_mutex_unlock();
        return hr;
    }

    memset(DDSD as *mut core::ffi::c_void, 0, (*DDSD).dwSize as usize);
    (*DDSD).dwSize = core::mem::size_of_val(&*DDSD) as DWORD;
    (*DDSD).dwFlags = (DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT | DDSD_PITCH | DDSD_REFRESHRATE) as DWORD;
    (*DDSD).dwWidth = mode.width as DWORD;
    (*DDSD).dwHeight = mode.height as DWORD;
    (*DDSD).__bindgen_anon_3.dwRefreshRate = mode.refresh_rate as DWORD;
    (*DDSD).__bindgen_anon_5.ddpfPixelFormat.dwSize = core::mem::size_of_val(&(*DDSD).__bindgen_anon_5.ddpfPixelFormat) as DWORD;
    ddrawformat_from_wined3dformat(&raw mut (*DDSD).__bindgen_anon_5.ddpfPixelFormat, mode.format_id);
    (*DDSD).__bindgen_anon_1.lPitch = (mode.width as u32).wrapping_mul((*DDSD).__bindgen_anon_5.ddpfPixelFormat.__bindgen_anon_1.dwRGBBitCount as u32).wrapping_div(8 as u32) as LONG;

    if TRACE_ON!(__wine_dbch_ddraw) != 0 {
        DDRAW_dump_surface_desc(DDSD);
    }

    wined3d_mutex_unlock();

    return DD_OK;
}

unsafe extern "system" fn ddraw4_GetDisplayMode(
    iface: *mut IDirectDraw4,
    surface_desc: *mut DDSURFACEDESC2,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    WINE_TRACE!("iface %p, surface_desc %p.\n", iface, surface_desc);
    return ddraw7_GetDisplayMode(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), surface_desc);
}

unsafe extern "system" fn ddraw2_GetDisplayMode(iface: *mut IDirectDraw2, surface_desc: *mut DDSURFACEDESC) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    let mut hr: HRESULT = 0;

    hr = ddraw7_GetDisplayMode(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), surface_desc as *mut DDSURFACEDESC2);
    if hr >= 0 {
        (*surface_desc).dwSize = core::mem::size_of::<DDSURFACEDESC>() as DWORD;
    }
    return hr;
}

unsafe extern "system" fn ddraw1_GetDisplayMode(
    iface: *mut IDirectDraw,
    surface_desc: *mut DDSURFACEDESC,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    let mut hr: HRESULT;

    WINE_TRACE!("iface %p, surface_desc %p.\n", iface, surface_desc);

    hr = ddraw7_GetDisplayMode(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        surface_desc as *mut DDSURFACEDESC2,
    );
    if hr >= 0 {
        (*surface_desc).dwSize = core::mem::size_of::<DDSURFACEDESC>() as DWORD;
    }
    return hr;
}

unsafe extern "system" fn ddraw7_GetFourCCCodes(
    iface: *mut IDirectDraw7,
    NumCodes: *mut DWORD,
    Codes: *mut DWORD,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    static formats: [wined3d_format_id; 11] = [
        WINED3DFMT_YUY2, WINED3DFMT_UYVY, WINED3DFMT_YV12,
        WINED3DFMT_DXT1, WINED3DFMT_DXT2, WINED3DFMT_DXT3, WINED3DFMT_DXT4, WINED3DFMT_DXT5,
        WINED3DFMT_ATI2N, WINED3DFMT_NVHU, WINED3DFMT_NVHS
    ];
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    let mut count: DWORD = 0;
    let mut i: DWORD;
    let mut outsize: DWORD;
    let mut hr: HRESULT;

    TRACE!(
        "iface %p, codes_count %p, codes %p.\n",
        iface,
        NumCodes,
        Codes
    );

    hr = wined3d_output_get_display_mode(
        (*ddraw).wined3d_output,
        core::ptr::addr_of_mut!(mode),
        core::ptr::null_mut(),
    );
    if hr < 0 {
        ERR!("Failed to get display mode, hr %#lx.\n", hr);
        return hr;
    }

    outsize = if !NumCodes.is_null() && !Codes.is_null() {
        *NumCodes
    } else {
        0
    };

    i = 0;
    while (i as usize) < ARRAY_SIZE!(formats) as usize {
        if wined3d_check_device_format(
            (*ddraw).wined3d,
            (*ddraw).wined3d_adapter,
            WINED3D_DEVICE_TYPE_HAL,
            mode.format_id,
            0,
            0,
            WINED3D_RTYPE_TEXTURE_2D,
            formats[i as usize],
        ) >= 0
        {
            if count < outsize {
                *Codes.offset(count as isize) = formats[i as usize] as DWORD;
            }
            count = count.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if !NumCodes.is_null() {
        TRACE!("Returning %lu FourCC codes\n", count);
        *NumCodes = count;
    }

    return DD_OK;
}

unsafe extern "system" fn ddraw4_GetFourCCCodes(
    iface: *mut IDirectDraw4,
    codes_count: *mut DWORD,
    codes: *mut DWORD,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    WINE_TRACE!("iface %p, codes_count %p, codes %p.\n", iface, codes_count, codes);
    return ddraw7_GetFourCCCodes(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        codes_count,
        codes,
    );
}
unsafe extern "system" fn ddraw2_GetFourCCCodes(
    mut iface: *mut IDirectDraw2,
    mut codes_count: *mut DWORD,
    mut codes: *mut DWORD,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    WINE_TRACE!("iface %p, codes_count %p, codes %p.\n", iface, codes_count, codes);
    return ddraw7_GetFourCCCodes(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        codes_count,
        codes,
    );
}

unsafe extern "system" fn ddraw1_GetFourCCCodes(
    iface: *mut IDirectDraw,
    codes_count: *mut DWORD,
    codes: *mut DWORD,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    WINE_TRACE!("iface %p, codes_count %p, codes %p.\n", iface, codes_count, codes);
    return ddraw7_GetFourCCCodes(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        codes_count,
        codes,
    );
}

unsafe extern "system" fn ddraw7_GetMonitorFrequency(iface: *mut IDirectDraw7, frequency: *mut DWORD) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    let mut hr: HRESULT;

    TRACE!("iface %p, frequency %p.\n", iface, frequency);
    wined3d_mutex_lock();
    hr = wined3d_output_get_display_mode((*ddraw).wined3d_output, &raw mut mode, core::ptr::null_mut());
    wined3d_mutex_unlock();
    if hr < 0 {
        WARN!("Failed to get display mode, hr %#lx.\n", hr);
        return hr;
    }

    *frequency = mode.refresh_rate as DWORD;

    return DD_OK;
}

unsafe extern "system" fn ddraw4_GetMonitorFrequency(iface: *mut IDirectDraw4, frequency: *mut DWORD) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);

    return ddraw7_GetMonitorFrequency(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), frequency);
}

unsafe extern "system" fn ddraw2_GetMonitorFrequency(iface: *mut IDirectDraw2, frequency: *mut DWORD) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    WINE_TRACE!("iface %p, frequency %p.\n", iface, frequency);
    return ddraw7_GetMonitorFrequency(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), frequency);
}

unsafe extern "system" fn ddraw1_GetMonitorFrequency(
    iface: *mut IDirectDraw,
    frequency: *mut DWORD,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    WINE_TRACE!("iface %p, frequency %p.\n", iface, frequency);
    return ddraw7_GetMonitorFrequency(&raw mut (*ddraw).IDirectDraw7_iface, frequency);
}
unsafe extern "system" fn ddraw7_GetVerticalBlankStatus(iface: *mut IDirectDraw7, status: *mut BOOL) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut raster_status: wined3d_raster_status = wined3d_raster_status {
        in_vblank: 0,
        scan_line: 0,
    };
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, status %p.\n", iface, status);

    if status.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    hr = wined3d_output_get_raster_status((*ddraw).wined3d_output, &raw mut raster_status);
    wined3d_mutex_unlock();
    if hr < 0 {
        WARN!("Failed to get raster status, hr %#lx.\n", hr);
        return hr;
    }

    *status = raster_status.in_vblank;

    return DD_OK;
}

unsafe extern "system" fn ddraw4_GetVerticalBlankStatus(
    mut iface: *mut IDirectDraw4,
    mut status: *mut BOOL,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    WINE_TRACE!("iface %p, status %p.\n", iface, status);
    return ddraw7_GetVerticalBlankStatus(&raw mut (*ddraw).IDirectDraw7_iface, status);
}

unsafe extern "system" fn ddraw2_GetVerticalBlankStatus(iface: *mut IDirectDraw2, status: *mut BOOL) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);

    return ddraw7_GetVerticalBlankStatus(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), status);
}

unsafe extern "system" fn ddraw1_GetVerticalBlankStatus(iface: *mut IDirectDraw, status: *mut BOOL) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);

    return ddraw7_GetVerticalBlankStatus(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), status);
}

unsafe extern "system" fn ddraw7_GetAvailableVidMem(iface: *mut IDirectDraw7, caps: *mut DDSCAPS2, total: *mut DWORD, free: *mut DWORD) -> HRESULT {
    let mut framebuffer_size: u32;
    let mut total_vidmem: u32;
    let mut free_vidmem: u32;
    let ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    let mut hr: HRESULT = DD_OK as HRESULT;

    if total.is_null() && free.is_null() {
        return DDERR_INVALIDPARAMS as HRESULT;
    }

    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        DDRAW_dump_DDSCAPS2(caps);
    }
    wined3d_mutex_lock();

    if {
        hr = wined3d_output_get_display_mode((*ddraw).wined3d_output, core::ptr::addr_of_mut!(mode), core::ptr::null_mut());
        hr
    } < 0 as HRESULT
    {
        WARN!("Failed to get display mode, hr %#lx.\n", hr);
        wined3d_mutex_unlock();
        return hr;
    }
    framebuffer_size = wined3d_calculate_format_pitch((*ddraw).wined3d_adapter, mode.format_id, mode.width as u32);
    framebuffer_size = framebuffer_size.wrapping_mul(mode.height as u32);

    if !free.is_null() {
        free_vidmem = wined3d_device_get_available_texture_mem((*ddraw).wined3d_device) as u32;
        *free = if framebuffer_size > free_vidmem {
            0 as DWORD
        } else {
            free_vidmem.wrapping_sub(framebuffer_size) as DWORD
        };
    }

    if !total.is_null() {
        let mut desc: wined3d_adapter_identifier = core::mem::zeroed();

        hr = wined3d_adapter_get_identifier((*ddraw).wined3d_adapter, 0 as uint32_t, core::ptr::addr_of_mut!(desc));
        total_vidmem = min!(UINT_MAX as u64, desc.video_memory as u64) as u32;
        *total = if framebuffer_size > total_vidmem {
            0 as DWORD
        } else {
            total_vidmem.wrapping_sub(framebuffer_size) as DWORD
        };
    }

    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn ddraw4_GetAvailableVidMem(
    iface: *mut IDirectDraw4,
    caps: *mut DDSCAPS2,
    total: *mut DWORD,
    free: *mut DWORD,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);

    return ddraw7_GetAvailableVidMem(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), caps, total, free);
}
unsafe extern "system" fn ddraw2_GetAvailableVidMem(
    mut iface: *mut IDirectDraw2,
    mut caps: *mut DDSCAPS,
    mut total: *mut DWORD,
    mut free: *mut DWORD,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    let mut caps2: DDSCAPS2 = core::mem::zeroed();
    TRACE!(
        "iface %p, caps %p, total %p, free %p.\n",
        iface,
        caps,
        total,
        free
    );
    DDRAW_Convert_DDSCAPS_1_To_2(caps, core::ptr::addr_of_mut!(caps2));
    return ddraw7_GetAvailableVidMem(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        core::ptr::addr_of_mut!(caps2),
        total,
        free,
    );
}

unsafe extern "system" fn ddraw7_Initialize(iface: *mut IDirectDraw7, guid: *mut GUID) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);

    if ((*ddraw).flags & DDRAW_INITIALIZED) != 0 {
        return DDERR_ALREADYINITIALIZED;
    }

    /* FIXME: To properly take the GUID into account we should call
     * ddraw_init() here instead of in DDRAW_Create(). */
    if !guid.is_null() {
        FIXME!("Ignoring guid %s.\n", debugstr_guid(guid));
    }

    (*ddraw).flags |= DDRAW_INITIALIZED;
    return DD_OK;
}
unsafe extern "system" fn ddraw4_Initialize(
    mut iface: *mut IDirectDraw4,
    mut guid: *mut GUID,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    WINE_TRACE!("iface %p, guid %s.\n", iface, debugstr_guid(guid));
    return ddraw7_Initialize(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), guid);
}
unsafe extern "system" fn ddraw2_Initialize(iface: *mut IDirectDraw2, guid: *mut GUID) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);

    return ddraw7_Initialize(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), guid);
}
unsafe extern "system" fn ddraw1_Initialize(
    mut iface: *mut IDirectDraw,
    mut guid: *mut GUID,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    WINE_TRACE!("iface %p, guid %s.\n", iface, debugstr_guid(guid));
    return ddraw7_Initialize(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), guid);
}
unsafe extern "system" fn d3d1_Initialize(iface: *mut IDirect3D, riid: REFIID) -> HRESULT {
    // TRACE("iface %p, riid %s.\n", iface, debugstr_guid(riid));
    DDERR_ALREADYINITIALIZED
}

unsafe extern "system" fn ddraw7_FlipToGDISurface(iface: *mut IDirectDraw7) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut gdi_surface: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut gdi_impl: *mut ddraw_surface = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();

    hr = IDirectDraw7_GetGDISurface(iface, core::ptr::addr_of_mut!(gdi_surface));
    if hr < 0 {
        WARN!("Failed to retrieve GDI surface, hr %#lx.\n", hr);
        wined3d_mutex_unlock();
        return hr;
    }

    gdi_impl = impl_from_IDirectDrawSurface7(gdi_surface);
    if ((*gdi_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_FRONTBUFFER) != 0 {
        hr = DD_OK;
    } else {
        hr = IDirectDrawSurface7_Flip(
            core::ptr::addr_of_mut!((*(*ddraw).primary).IDirectDrawSurface7_iface),
            gdi_surface,
            DDFLIP_WAIT,
        );
    }
    IDirectDrawSurface7_Release(gdi_surface);

    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn ddraw4_FlipToGDISurface(iface: *mut IDirectDraw4) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);

    return ddraw7_FlipToGDISurface(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface));
}

unsafe extern "system" fn ddraw2_FlipToGDISurface(iface: *mut IDirectDraw2) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);

    return ddraw7_FlipToGDISurface(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface));
}

unsafe extern "system" fn ddraw1_FlipToGDISurface(iface: *mut IDirectDraw) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);

    TRACE!("iface %p.\n", iface);

    return ddraw7_FlipToGDISurface(&raw mut (*ddraw).IDirectDraw7_iface);
}
unsafe extern "system" fn ddraw7_WaitForVerticalBlank(
    mut iface: *mut IDirectDraw7,
    mut Flags: DWORD,
    mut event: HANDLE,
) -> HRESULT {
    static mut hide: BOOL = 0;
    WINE_TRACE!("iface %p, flags %#lx, event %p.\n", iface, Flags, event);
    if hide == 0 {
        WINE_FIXME!("iface %p, flags %#lx, event %p stub!\n", iface, Flags, event);
        hide = (TRUE as i32);
    }
    if (Flags & DDWAITVB_BLOCKBEGINEVENT as DWORD) != 0 {
        return DDERR_UNSUPPORTED;
    }
    return DD_OK;
}

unsafe extern "system" fn ddraw4_WaitForVerticalBlank(iface: *mut IDirectDraw4, flags: DWORD, event: HANDLE) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);

    return ddraw7_WaitForVerticalBlank(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), flags, event);
}

unsafe extern "system" fn ddraw2_WaitForVerticalBlank(iface: *mut IDirectDraw2, flags: DWORD, event: HANDLE) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);

    return ddraw7_WaitForVerticalBlank(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), flags, event);
}
unsafe extern "system" fn ddraw1_WaitForVerticalBlank(iface: *mut IDirectDraw, flags: DWORD, event: HANDLE) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);

    return ddraw7_WaitForVerticalBlank(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), flags, event);
}
unsafe extern "system" fn ddraw7_GetScanLine(iface: *mut IDirectDraw7, Scanline: *mut DWORD) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut raster_status: wined3d_raster_status = wined3d_raster_status {
        in_vblank: 0,
        scan_line: 0,
    };
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, line %p.\n", iface, Scanline);

    wined3d_mutex_lock();
    hr = wined3d_output_get_raster_status((*ddraw).wined3d_output, core::ptr::addr_of_mut!(raster_status));
    wined3d_mutex_unlock();
    if hr < 0 {
        WARN!("Failed to get raster status, hr %#lx.\n", hr);
        return hr;
    }

    *Scanline = raster_status.scan_line;

    if raster_status.in_vblank != 0 {
        return DDERR_VERTICALBLANKINPROGRESS;
    }

    return DD_OK;
}
unsafe extern "system" fn ddraw4_GetScanLine(iface: *mut IDirectDraw4, line: *mut DWORD) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    WINE_TRACE!("iface %p, line %p.\n", iface, line);
    return ddraw7_GetScanLine(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), line);
}

unsafe extern "system" fn ddraw2_GetScanLine(iface: *mut IDirectDraw2, line: *mut DWORD) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    WINE_TRACE!("iface %p, line %p.\n", iface, line);
    return ddraw7_GetScanLine(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), line);
}
unsafe extern "system" fn ddraw1_GetScanLine(iface: *mut IDirectDraw, line: *mut DWORD) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    WINE_TRACE!("iface %p, line %p.\n", iface, line);
    return ddraw7_GetScanLine(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), line);
}

unsafe extern "system" fn ddraw7_TestCooperativeLevel(iface: *mut IDirectDraw7) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);

    return if ((*ddraw).device_state) as u32 == DDRAW_DEVICE_STATE_LOST { DDERR_NOEXCLUSIVEMODE } else { DD_OK };
}
unsafe extern "system" fn ddraw4_TestCooperativeLevel(mut iface: *mut IDirectDraw4) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return ddraw7_TestCooperativeLevel(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface));
}

unsafe extern "system" fn ddraw7_GetGDISurface(
    iface: *mut IDirectDraw7,
    surface: *mut *mut IDirectDrawSurface7,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut ddraw_surface: *mut ddraw_surface = core::ptr::null_mut();

    wined3d_mutex_lock();

    if (*ddraw).gdi_surface.is_null() || {
        ddraw_surface = wined3d_texture_get_sub_resource_parent((*ddraw).gdi_surface, 0) as *mut ddraw_surface;
        ddraw_surface.is_null()
    } {
        WARN!("GDI surface not available.\n");
        *surface = core::ptr::null_mut();
        wined3d_mutex_unlock();
        return DDERR_NOTFOUND;
    }
    *surface = core::ptr::addr_of_mut!((*ddraw_surface).IDirectDrawSurface7_iface);
    IDirectDrawSurface7_AddRef(*surface);

    wined3d_mutex_unlock();

    return DD_OK;
}

unsafe extern "system" fn ddraw4_GetGDISurface(
    iface: *mut IDirectDraw4,
    surface: *mut *mut IDirectDrawSurface4,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    let mut surface_impl: *mut ddraw_surface = core::ptr::null_mut();
    let mut surface7: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = ddraw7_GetGDISurface(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), core::ptr::addr_of_mut!(surface7));
    if hr < 0 {
        *surface = core::ptr::null_mut();
        return hr;
    }
    surface_impl = impl_from_IDirectDrawSurface7(surface7);
    *surface = core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface4_iface);
    IDirectDrawSurface4_AddRef(*surface);
    IDirectDrawSurface7_Release(surface7);

    return hr;
}

unsafe extern "system" fn ddraw2_GetGDISurface(iface: *mut IDirectDraw2, surface: *mut *mut IDirectDrawSurface) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    let mut surface_impl: *mut ddraw_surface = core::ptr::null_mut();
    let mut surface7: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = ddraw7_GetGDISurface(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), core::ptr::addr_of_mut!(surface7));
    if hr < 0 {
        *surface = core::ptr::null_mut();
        return hr;
    }
    surface_impl = impl_from_IDirectDrawSurface7(surface7);
    *surface = core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface_iface);
    IDirectDrawSurface_AddRef(*surface);
    IDirectDrawSurface7_Release(surface7);

    return hr;
}
unsafe extern "system" fn ddraw1_GetGDISurface(
    mut iface: *mut IDirectDraw,
    mut surface: *mut *mut IDirectDrawSurface,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    let mut surface_impl: *mut ddraw_surface = core::ptr::null_mut();
    let mut surface7: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, surface %p.\n", iface, surface);

    hr = ddraw7_GetGDISurface(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        core::ptr::addr_of_mut!(surface7),
    );
    if hr < 0 {
        *surface = core::ptr::null_mut();
        return hr;
    }
    surface_impl = impl_from_IDirectDrawSurface7(surface7);
    *surface = core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface_iface);
    IDirectDrawSurface_AddRef(*surface);
    IDirectDrawSurface7_Release(surface7);

    return hr;
}

unsafe extern "system" fn EnumDisplayModesCallbackThunk(surface_desc: *mut DDSURFACEDESC2, context: *mut core::ffi::c_void) -> HRESULT {
    let mut cbcontext: *mut displaymodescallback_context = context as *mut displaymodescallback_context;
    let mut desc: DDSURFACEDESC = core::mem::zeroed();

    DDSD2_to_DDSD(surface_desc, core::ptr::addr_of_mut!(desc));
    return (*cbcontext).func.unwrap_unchecked()(
        core::ptr::addr_of_mut!(desc),
        (*cbcontext).context as LPVOID,
    );
}

unsafe extern "system" fn ddraw7_EnumDisplayModes(
    mut iface: *mut IDirectDraw7,
    mut Flags: DWORD,
    mut DDSD: *mut DDSURFACEDESC2,
    mut Context: *mut ::core::ffi::c_void,
    mut cb: LPDDENUMMODESCALLBACK2,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut enum_modes: *mut wined3d_display_mode = core::ptr::null_mut();
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    let mut modenum: ::core::ffi::c_uint;
    let mut fmt: ::core::ffi::c_uint;
    let mut callback_sd: DDSURFACEDESC2 = core::mem::zeroed();
    let mut enum_mode_count: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut enum_mode_array_size: ::core::ffi::c_uint = 16 as ::core::ffi::c_uint;
    let mut pixelformat: DDPIXELFORMAT = core::mem::zeroed();

    static checkFormatList: [wined3d_format_id; 3] = [
        WINED3DFMT_B8G8R8X8_UNORM,
        WINED3DFMT_B5G6R5_UNORM,
        WINED3DFMT_P8_UINT,
    ];

    if cb.is_none() {
        return DDERR_INVALIDPARAMS;
    }

    enum_modes = malloc(
        (enum_mode_array_size as size_t)
            .wrapping_mul(::core::mem::size_of::<wined3d_display_mode>() as size_t),
    ) as *mut wined3d_display_mode;
    if enum_modes.is_null() {
        return DDERR_OUTOFMEMORY;
    }

    wined3d_mutex_lock();

    pixelformat.dwSize = ::core::mem::size_of::<DDPIXELFORMAT>() as DWORD;
    fmt = 0 as ::core::ffi::c_uint;
    while (fmt as usize) < (ARRAY_SIZE!(checkFormatList) as usize) {
        modenum = 0 as ::core::ffi::c_uint;
        loop {
            let fresh15 = modenum;
            modenum = modenum.wrapping_add(1);
            if !(wined3d_output_get_mode(
                (*ddraw).wined3d_output,
                checkFormatList[fmt as usize],
                WINED3D_SCANLINE_ORDERING_UNKNOWN,
                fresh15,
                &raw mut mode,
                false,
            ) == WINED3D_OK)
            {
                break;
            }
            let mut found: BOOL = (FALSE as i32);
            let mut i: ::core::ffi::c_uint;

            ddrawformat_from_wined3dformat(&raw mut pixelformat, mode.format_id);
            if !DDSD.is_null() {
                if (*DDSD).dwFlags & (DDSD_WIDTH as DWORD) != 0
                    && mode.width != (*DDSD).dwWidth
                {
                    continue;
                }
                if (*DDSD).dwFlags & (DDSD_HEIGHT as DWORD) != 0
                    && mode.height != (*DDSD).dwHeight
                {
                    continue;
                }
                if (*DDSD).dwFlags & (DDSD_REFRESHRATE as DWORD) != 0
                    && mode.refresh_rate
                        != (*DDSD).__bindgen_anon_3.dwRefreshRate
                {
                    continue;
                }
                if (*DDSD).dwFlags & (DDSD_PIXELFORMAT as DWORD) != 0
                    && pixelformat.__bindgen_anon_1.dwRGBBitCount
                        != (*DDSD)

                            .__bindgen_anon_5.ddpfPixelFormat

                            .__bindgen_anon_1.dwRGBBitCount
                {
                    continue;
                }
            }

            /* DX docs state EnumDisplayMode should return only unique modes */
            i = 0 as ::core::ffi::c_uint;
            while i < enum_mode_count {
                if (*enum_modes.offset(i as isize)).width == mode.width
                    && (*enum_modes.offset(i as isize)).height == mode.height
                    && (*enum_modes.offset(i as isize)).format_id as ::core::ffi::c_uint
                        == mode.format_id as ::core::ffi::c_uint
                    && ((*enum_modes.offset(i as isize)).refresh_rate == mode.refresh_rate
                        || (Flags & (DDEDM_REFRESHRATES as DWORD)) == 0)
                {
                    found = (TRUE as i32);
                    break;
                } else {
                    i = i.wrapping_add(1);
                }
            }
            if found != 0 {
                continue;
            }

            memset(
                &raw mut callback_sd as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<DDSURFACEDESC2>() as size_t,
            );
            callback_sd.dwSize = ::core::mem::size_of::<DDSURFACEDESC2>() as DWORD;
            callback_sd.__bindgen_anon_5.ddpfPixelFormat.dwSize =
                ::core::mem::size_of::<DDPIXELFORMAT>() as DWORD;

            callback_sd.dwFlags = (DDSD_HEIGHT
                | DDSD_WIDTH
                | DDSD_PIXELFORMAT
                | DDSD_PITCH
                | DDSD_REFRESHRATE) as DWORD;
            if Flags & (DDEDM_REFRESHRATES as DWORD) != 0 {
                callback_sd.__bindgen_anon_3.dwRefreshRate = mode.refresh_rate as DWORD;
            }

            callback_sd.dwWidth = mode.width as DWORD;
            callback_sd.dwHeight = mode.height as DWORD;

            callback_sd.__bindgen_anon_5.ddpfPixelFormat = pixelformat;

            /* Calc pitch and DWORD align like MSDN says */
            callback_sd.__bindgen_anon_1.lPitch = (callback_sd

                .__bindgen_anon_5.ddpfPixelFormat

                .__bindgen_anon_1.dwRGBBitCount as UINT)
                .wrapping_div(8 as UINT)
                .wrapping_mul(mode.width) as LONG;
            callback_sd.__bindgen_anon_1.lPitch = ((callback_sd.__bindgen_anon_1.lPitch
                as ::core::ffi::c_int
                + 3 as ::core::ffi::c_int)
                & !(3 as ::core::ffi::c_int))
                as LONG;

            WINE_TRACE!("Enumerating %lux%lux%lu @%lu\n", callback_sd.dwWidth, callback_sd.dwHeight, callback_sd .__bindgen_anon_5.ddpfPixelFormat .__bindgen_anon_1.dwRGBBitCount, callback_sd.__bindgen_anon_3.dwRefreshRate);

            if (cb.unwrap_unchecked())(&raw mut callback_sd, Context as LPVOID)
                == (DDENUMRET_CANCEL as i32)
            {
                WINE_TRACE!("Application asked to terminate the enumeration\n");
                free(enum_modes as *mut ::core::ffi::c_void);
                wined3d_mutex_unlock();
                return DD_OK;
            }

            if enum_mode_count == enum_mode_array_size {
                let mut new_enum_modes: *mut wined3d_display_mode = core::ptr::null_mut();

                enum_mode_array_size =
                    enum_mode_array_size.wrapping_mul(2 as ::core::ffi::c_uint);
                new_enum_modes = realloc(
                    enum_modes as *mut ::core::ffi::c_void,
                    (enum_mode_array_size as size_t)
                        .wrapping_mul(::core::mem::size_of::<wined3d_display_mode>() as size_t),
                ) as *mut wined3d_display_mode;
                if new_enum_modes.is_null() {
                    free(enum_modes as *mut ::core::ffi::c_void);
                    wined3d_mutex_unlock();
                    return DDERR_OUTOFMEMORY;
                }

                enum_modes = new_enum_modes;
            }
            let fresh16 = enum_mode_count;
            enum_mode_count = enum_mode_count.wrapping_add(1);
            *enum_modes.offset(fresh16 as isize) = mode;
        }
        fmt = fmt.wrapping_add(1);
    }

    free(enum_modes as *mut ::core::ffi::c_void);
    wined3d_mutex_unlock();

    return DD_OK;
}

unsafe extern "system" fn ddraw4_EnumDisplayModes(iface: *mut IDirectDraw4, flags: DWORD, surface_desc: *mut DDSURFACEDESC2, context: *mut core::ffi::c_void, callback: LPDDENUMMODESCALLBACK2) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);

    return ddraw7_EnumDisplayModes(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), flags, surface_desc, context, callback);
}

unsafe extern "system" fn ddraw2_EnumDisplayModes(
    iface: *mut IDirectDraw2,
    flags: DWORD,
    surface_desc: *mut DDSURFACEDESC,
    context: *mut core::ffi::c_void,
    callback: LPDDENUMMODESCALLBACK,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    let mut cbcontext: displaymodescallback_context = displaymodescallback_context {
        func: None,
        context: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut surface_desc2: DDSURFACEDESC2 = core::mem::zeroed();
    WINE_TRACE!("iface %p, flags %#lx, surface_desc %p, context %p, callback %p.\n", iface, flags, surface_desc, context, callback);
    cbcontext.func = callback;
    cbcontext.context = context;

    if !surface_desc.is_null() {
        DDSD_to_DDSD2(surface_desc, &raw mut surface_desc2);
    }
    return ddraw7_EnumDisplayModes(
        &raw mut (*ddraw).IDirectDraw7_iface,
        flags,
        if !surface_desc.is_null() {
            &raw mut surface_desc2
        } else {
            core::ptr::null_mut::<DDSURFACEDESC2>()
        },
        &raw mut cbcontext as *mut core::ffi::c_void,
        Some(
            EnumDisplayModesCallbackThunk,
        ),
    );
}
unsafe extern "system" fn ddraw1_EnumDisplayModes(
    iface: *mut IDirectDraw,
    flags: DWORD,
    surface_desc: *mut DDSURFACEDESC,
    context: *mut core::ffi::c_void,
    callback: LPDDENUMMODESCALLBACK,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    let mut cbcontext: displaymodescallback_context = displaymodescallback_context {
        func: None,
        context: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut surface_desc2: DDSURFACEDESC2 = core::mem::zeroed::<DDSURFACEDESC2>();

    WINE_TRACE!("iface %p, flags %#lx, surface_desc %p, context %p, callback %p.\n", iface, flags, surface_desc, context, callback);

    cbcontext.func = callback;
    cbcontext.context = context;

    if !surface_desc.is_null() {
        DDSD_to_DDSD2(surface_desc, &raw mut surface_desc2);
    }
    return ddraw7_EnumDisplayModes(
        &raw mut (*ddraw).IDirectDraw7_iface,
        flags,
        if !surface_desc.is_null() {
            &raw mut surface_desc2
        } else {
            core::ptr::null_mut::<DDSURFACEDESC2>()
        },
        &raw mut cbcontext as *mut core::ffi::c_void,
        Some(
            EnumDisplayModesCallbackThunk,
        ),
    );
}
unsafe extern "system" fn ddraw7_EvaluateMode(
    iface: *mut IDirectDraw7,
    Flags: DWORD,
    Timeout: *mut DWORD,
) -> HRESULT {
    // FIXME("iface %p, flags %#lx, timeout %p stub!\n", iface, Flags, Timeout);
    DD_OK
}

unsafe extern "system" fn ddraw7_GetDeviceIdentifier(
    mut iface: *mut IDirectDraw7,
    mut DDDI: *mut DDDEVICEIDENTIFIER2,
    mut Flags: DWORD,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut adapter_id: wined3d_adapter_identifier = core::mem::zeroed();
    let mut hr: HRESULT = S_OK;

    TRACE!(
        "iface %p, device_identifier %p, flags %#lx.\n",
        iface,
        DDDI,
        Flags
    );

    if DDDI.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if (Flags & DDGDI_GETHOSTIDENTIFIER as DWORD) != 0 {
        /* The DDGDI_GETHOSTIDENTIFIER returns the information about the 2D
         * host adapter, if there's a secondary 3D adapter. This doesn't apply
         * to any modern hardware, nor is it interesting for Wine, so ignore it.
         * Size of DDDEVICEIDENTIFIER2 may be aligned to 8 bytes and thus 4
         * bytes too long. So only copy the relevant part of the structure
         */

        memcpy(
            DDDI as *mut ::core::ffi::c_void,
            core::ptr::addr_of!(deviceidentifier) as *const ::core::ffi::c_void,
            core::mem::offset_of!(DDDEVICEIDENTIFIER2, dwWHQLLevel)
                + core::mem::size_of::<DWORD>(),
        );
        return DD_OK;
    }

    /* Drakan: Order of the Flame expects accurate D3D device information from ddraw */
    adapter_id.driver = core::ptr::addr_of_mut!((*DDDI).szDriver)
        as *mut ::core::ffi::c_char;
    adapter_id.driver_size =
        core::mem::size_of_val(&(*DDDI).szDriver) as ::core::ffi::c_uint;
    adapter_id.description = core::ptr::addr_of_mut!((*DDDI).szDescription)
        as *mut ::core::ffi::c_char;
    adapter_id.description_size =
        core::mem::size_of_val(&(*DDDI).szDescription) as ::core::ffi::c_uint;
    wined3d_mutex_lock();
    hr = wined3d_adapter_get_identifier(
        (*ddraw).wined3d_adapter,
        WINED3DENUM_WHQL_LEVEL as uint32_t,
        core::ptr::addr_of_mut!(adapter_id),
    );
    wined3d_mutex_unlock();
    if hr < 0 {
        return hr;
    }

    (*DDDI).liDriverVersion = adapter_id.driver_version;
    (*DDDI).dwVendorId = adapter_id.vendor_id;
    (*DDDI).dwDeviceId = adapter_id.device_id;
    (*DDDI).dwSubSysId = adapter_id.subsystem_id;
    (*DDDI).dwRevision = adapter_id.revision;
    (*DDDI).guidDeviceIdentifier = adapter_id.device_identifier;
    (*DDDI).dwWHQLLevel = adapter_id.whql_level;
    return DD_OK;
}

unsafe extern "system" fn ddraw4_GetDeviceIdentifier(iface: *mut IDirectDraw4, identifier: *mut DDDEVICEIDENTIFIER, flags: DWORD) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    let mut identifier2: DDDEVICEIDENTIFIER2 = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = ddraw7_GetDeviceIdentifier(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), core::ptr::addr_of_mut!(identifier2), flags);
    DDRAW_Convert_DDDEVICEIDENTIFIER_2_To_1(core::ptr::addr_of!(identifier2), identifier);

    return hr;
}

unsafe extern "system" fn ddraw7_GetSurfaceFromDC(
    iface: *mut IDirectDraw7,
    dc: HDC,
    surface: *mut *mut IDirectDrawSurface7,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut surface_impl: *mut ddraw_surface;

    if surface.is_null() {
        return E_INVALIDARG;
    }

    if !dc.is_null() {
        wined3d_mutex_lock();
        surface_impl = ((*ddraw).surface_list.next as *mut u8)
            .sub(core::mem::offset_of!(ddraw_surface, surface_list_entry))
            as *mut ddraw_surface;
        while core::ptr::addr_of_mut!((*surface_impl).surface_list_entry)
            != core::ptr::addr_of_mut!((*ddraw).surface_list)
        {
            if (*surface_impl).dc != dc {
                surface_impl = ((*surface_impl).surface_list_entry.next as *mut u8)
                    .sub(core::mem::offset_of!(ddraw_surface, surface_list_entry))
                    as *mut ddraw_surface;
            } else {
                TRACE!("Found surface %p for dc %p.\n", surface_impl, dc);
                *surface = core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface7_iface);
                IDirectDrawSurface7_AddRef(*surface);
                wined3d_mutex_unlock();
                return DD_OK;
            }
        }
        wined3d_mutex_unlock();
    }

    TRACE!("No surface found for dc %p.\n", dc);
    *surface = core::ptr::null_mut();
    return DDERR_NOTFOUND;
}

unsafe extern "system" fn ddraw4_GetSurfaceFromDC(iface: *mut IDirectDraw4, dc: HDC, surface: *mut *mut IDirectDrawSurface4) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    let mut surface_impl: *mut ddraw_surface = core::ptr::null_mut::<ddraw_surface>();
    let mut surface7: *mut IDirectDrawSurface7 = core::ptr::null_mut::<IDirectDrawSurface7>();
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, dc %p, surface %p.\n", iface, dc, surface);
    if surface.is_null() {
        return E_INVALIDARG;
    }
    hr = ddraw7_GetSurfaceFromDC(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), dc, core::ptr::addr_of_mut!(surface7));
    if hr < 0 {
        *surface = core::ptr::null_mut::<IDirectDrawSurface4>();
        return hr;
    }
    surface_impl = impl_from_IDirectDrawSurface7(surface7);
    /* Tests say this is true */
    *surface = core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface_iface) as *mut IDirectDrawSurface4;
    IDirectDrawSurface_AddRef(core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface_iface));
    IDirectDrawSurface7_Release(surface7);

    return hr;
}

unsafe extern "system" fn restore_callback(surface: *mut IDirectDrawSurface7, desc: *mut DDSURFACEDESC2, context: *mut core::ffi::c_void) -> HRESULT {
    IDirectDrawSurface_Restore((surface).cast());
    IDirectDrawSurface_Release((surface).cast());

    return (DDENUMRET_OK) as HRESULT;
}

unsafe extern "system" fn ddraw7_RestoreAllSurfaces(iface: *mut IDirectDraw7) -> HRESULT {
    TRACE!("iface %p.\n", iface);

    return IDirectDraw7_EnumSurfaces(
        iface,
        DDENUMSURFACES_ALL | DDENUMSURFACES_DOESEXIST,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Some(restore_callback),
    );
}

unsafe extern "system" fn ddraw4_RestoreAllSurfaces(iface: *mut IDirectDraw4) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);

    TRACE!("iface %p.\n", iface);

    return ddraw7_RestoreAllSurfaces(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface));
}
unsafe extern "system" fn ddraw7_StartModeTest(iface: *mut IDirectDraw7, Modes: *mut SIZE, NumModes: DWORD, Flags: DWORD) -> HRESULT {
    DPRINT1!("iface %p, modes %p, mode_count %lu, flags %#lx partial stub!\n", iface, Modes, NumModes, Flags);

    /* This looks sane */
    if Modes.is_null() || NumModes == 0 {
        return DDERR_INVALIDPARAMS;
    }

    /* DDSMT_ISTESTREQUIRED asks if a mode test is necessary.
     * As it is not, DDERR_TESTFINISHED is returned
     * (hopefully that's correct
     *
    if(Flags & DDSMT_ISTESTREQUIRED) return DDERR_TESTFINISHED;
     * well, that value doesn't (yet) exist in the wine headers, so ignore it
     */

    return DD_OK;
}

unsafe extern "system" fn ddraw7_CreateSurface(
    iface: *mut IDirectDraw7,
    surface_desc: *mut DDSURFACEDESC2,
    surface: *mut *mut IDirectDrawSurface7,
    outer_unknown: *mut IUnknown,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut impl_: *mut ddraw_surface = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();

    if ((*ddraw).cooperative_level & (DDSCL_NORMAL | DDSCL_EXCLUSIVE)) == 0 {
        WARN!("Cooperative level not set.\n");
        wined3d_mutex_unlock();
        return DDERR_NOCOOPERATIVELEVELSET;
    }

    if surface_desc.is_null() || (*surface_desc).dwSize as usize != core::mem::size_of::<DDSURFACEDESC2>() {
        WARN!("Application supplied invalid surface descriptor\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    // __TRY .. __EXCEPT_PAGE_FAULT: DUMMY_PSEH compiled the handler out of the C;
    // page_fault_try makes it real (comctl32's is_valid, the same idiom).
    if ntdll_sys::seh::page_fault_try(|| {
            *surface = core::ptr::null_mut();
    })
    .is_none()
    {
        WARN!("Surface pointer %p is invalid.\n", surface);
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    if (*surface_desc).ddsCaps.dwCaps & (DDSCAPS_FRONTBUFFER | DDSCAPS_BACKBUFFER) != 0 {
        if TRACE_ON!(__wine_dbch_ddraw) != 0 {
            DDRAW_dump_surface_desc(surface_desc);
        }

        WARN!("Application tried to create an explicit front or back buffer\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDCAPS;
    }

    hr = ddraw_surface_create(
        ddraw,
        surface_desc,
        core::ptr::addr_of_mut!(impl_),
        outer_unknown,
        7,
    );
    wined3d_mutex_unlock();
    if hr < 0 {
        return hr;
    }

    *surface = core::ptr::addr_of_mut!((*impl_).IDirectDrawSurface7_iface);
    IDirectDraw7_AddRef(iface);
    (*impl_).ifaceToRelease = iface as *mut IUnknown;

    return hr;
}

unsafe extern "system" fn ddraw4_CreateSurface(iface: *mut IDirectDraw4, surface_desc: *mut DDSURFACEDESC2, surface: *mut *mut IDirectDrawSurface4, outer_unknown: *mut IUnknown) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    let mut impl_0: *mut ddraw_surface = core::ptr::null_mut();
    let mut hr: HRESULT;

    wined3d_mutex_lock();

    if ((*ddraw).cooperative_level & (DDSCL_NORMAL | DDSCL_EXCLUSIVE)) == 0 {
        WARN!("Cooperative level not set.\n");
        wined3d_mutex_unlock();
        return DDERR_NOCOOPERATIVELEVELSET;
    }

    if surface_desc == core::ptr::null_mut() || (*surface_desc).dwSize as usize != core::mem::size_of::<DDSURFACEDESC2>() {
        WARN!("Application supplied invalid surface descriptor\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    // __TRY .. __EXCEPT_PAGE_FAULT: DUMMY_PSEH compiled the handler out of the C;
    // page_fault_try makes it real (comctl32's is_valid, the same idiom).
    if ntdll_sys::seh::page_fault_try(|| {
            *surface = core::ptr::null_mut();
    })
    .is_none()
    {
        WARN!("Surface pointer %p is invalid.\n", surface);
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    if (*surface_desc).ddsCaps.dwCaps & (DDSCAPS_FRONTBUFFER | DDSCAPS_BACKBUFFER) as DWORD != 0 {
        if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
            DDRAW_dump_surface_desc(surface_desc);
        }

        WARN!("Application tried to create an explicit front or back buffer\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDCAPS;
    }

    hr = ddraw_surface_create(ddraw, surface_desc, core::ptr::addr_of_mut!(impl_0), outer_unknown, 4);
    wined3d_mutex_unlock();
    if (hr) < 0 {
        return hr;
    }

    *surface = core::ptr::addr_of_mut!((*impl_0).IDirectDrawSurface4_iface);
    IDirectDraw4_AddRef(iface);
    (*impl_0).ifaceToRelease = iface as *mut IUnknown;

    return hr;
}
unsafe extern "system" fn ddraw2_CreateSurface(
    iface: *mut IDirectDraw2,
    surface_desc: *mut DDSURFACEDESC,
    surface: *mut *mut IDirectDrawSurface,
    outer_unknown: *mut IUnknown,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    let mut impl_: *mut ddraw_surface = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    let mut surface_desc2: DDSURFACEDESC2 = core::mem::zeroed();

    TRACE!("iface %p, surface_desc %p, surface %p, outer_unknown %p.\n", iface, surface_desc, surface, outer_unknown);

    wined3d_mutex_lock();

    if ((*ddraw).cooperative_level & ((DDSCL_NORMAL | DDSCL_EXCLUSIVE) as DWORD)) == 0 {
        WARN!("Cooperative level not set.\n");
        wined3d_mutex_unlock();
        return DDERR_NOCOOPERATIVELEVELSET;
    }

    if surface_desc.is_null()
        || (*surface_desc).dwSize as usize != core::mem::size_of::<DDSURFACEDESC>() as usize
    {
        WARN!("Application supplied invalid surface descriptor\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    // __TRY .. __EXCEPT_PAGE_FAULT: DUMMY_PSEH compiled the handler out of the C;
    // page_fault_try makes it real (comctl32's is_valid, the same idiom).
    if ntdll_sys::seh::page_fault_try(|| {
        *surface = core::ptr::null_mut();
    })
    .is_none()
    {
        WARN!("Surface pointer %p is invalid.\n", surface);
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    DDSD_to_DDSD2(surface_desc, &raw mut surface_desc2);
    if (*surface_desc).ddsCaps.dwCaps & ((DDSCAPS_FRONTBUFFER | DDSCAPS_BACKBUFFER) as DWORD) != 0 {
        if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
            DDRAW_dump_surface_desc(surface_desc as *mut DDSURFACEDESC2);
        }

        WARN!("Application tried to create an explicit front or back buffer\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDCAPS;
    }

    hr = ddraw_surface_create(
        ddraw,
        &raw mut surface_desc2,
        &raw mut impl_,
        outer_unknown,
        2,
    );
    wined3d_mutex_unlock();
    if hr < 0 {
        return hr;
    }

    *surface = &raw mut (*impl_).IDirectDrawSurface_iface;
    (*impl_).ifaceToRelease = core::ptr::null_mut();

    hr
}

unsafe extern "system" fn ddraw1_CreateSurface(
    mut iface: *mut IDirectDraw,
    mut surface_desc: *mut DDSURFACEDESC,
    mut surface: *mut *mut IDirectDrawSurface,
    mut outer_unknown: *mut IUnknown,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    let mut impl_: *mut ddraw_surface = core::ptr::null_mut::<ddraw_surface>();
    let mut hr: HRESULT = 0;
    let mut surface_desc2: DDSURFACEDESC2 = core::mem::zeroed();

    TRACE!(
        "iface %p, surface_desc %p, surface %p, outer_unknown %p.\n",
        iface,
        surface_desc,
        surface,
        outer_unknown
    );

    wined3d_mutex_lock();

    if (*ddraw).cooperative_level & (DDSCL_NORMAL | DDSCL_EXCLUSIVE) == 0 {
        WARN!("Cooperative level not set.\n");
        wined3d_mutex_unlock();
        return DDERR_NOCOOPERATIVELEVELSET;
    }

    if surface_desc.is_null()
        || (*surface_desc).dwSize as usize != core::mem::size_of::<DDSURFACEDESC>() as usize
    {
        WARN!("Application supplied invalid surface descriptor\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    // __TRY .. __EXCEPT_PAGE_FAULT: DUMMY_PSEH compiled the handler out of the C;
    // page_fault_try makes it real (comctl32's is_valid, the same idiom).
    if ntdll_sys::seh::page_fault_try(|| {
        *surface = core::ptr::null_mut::<IDirectDrawSurface>();
    })
    .is_none()
    {
        WARN!("Surface pointer %p is invalid.\n", surface);
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    if ((*surface_desc).ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE | DDSCAPS_BACKBUFFER) as DWORD)
        == (DDSCAPS_PRIMARYSURFACE | DDSCAPS_BACKBUFFER) as DWORD
        || ((*surface_desc).ddsCaps.dwCaps & (DDSCAPS_FLIP | DDSCAPS_FRONTBUFFER) as DWORD)
            == (DDSCAPS_FLIP | DDSCAPS_FRONTBUFFER) as DWORD
    {
        WARN!("Application tried to create an explicit front or back buffer.\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDCAPS;
    }

    DDSD_to_DDSD2(surface_desc, core::ptr::addr_of_mut!(surface_desc2));
    hr = ddraw_surface_create(
        ddraw,
        core::ptr::addr_of_mut!(surface_desc2),
        core::ptr::addr_of_mut!(impl_),
        outer_unknown,
        1,
    );
    wined3d_mutex_unlock();
    if hr < 0 {
        return hr;
    }

    *surface = core::ptr::addr_of_mut!((*impl_).IDirectDrawSurface_iface);
    (*impl_).ifaceToRelease = core::ptr::null_mut::<IUnknown>();

    return hr;
}

unsafe extern "C" fn Main_DirectDraw_DDPIXELFORMAT_Match(
    mut requested: *const DDPIXELFORMAT,
    mut provided: *const DDPIXELFORMAT,
) -> BOOL {
    static mut must_match: DWORD = (DDPF_PALETTEINDEXED1 as DWORD)
        | (DDPF_PALETTEINDEXED2 as DWORD)
        | (DDPF_PALETTEINDEXED4 as DWORD)
        | (DDPF_PALETTEINDEXED8 as DWORD)
        | (DDPF_FOURCC as DWORD)
        | (DDPF_ZBUFFER as DWORD)
        | (DDPF_STENCILBUFFER as DWORD);
    if ((*requested).dwFlags & (*provided).dwFlags) != (*requested).dwFlags {
        return (FALSE) as BOOL;
    }
    if ((*requested).dwFlags & must_match) != ((*provided).dwFlags & must_match) {
        return (FALSE) as BOOL;
    }
    if (*requested).dwFlags & (DDPF_FOURCC as DWORD) != 0 {
        if (*requested).dwFourCC != (*provided).dwFourCC {
            return (FALSE) as BOOL;
        }
    }
    if (*requested).dwFlags
        & ((DDPF_RGB
            | DDPF_YUV
            | DDPF_ZBUFFER
            | DDPF_ALPHA
            | DDPF_LUMINANCE
            | DDPF_BUMPDUDV) as DWORD)
        != 0
    {
        if (*requested).__bindgen_anon_1.dwRGBBitCount != (*provided).__bindgen_anon_1.dwRGBBitCount {
            return (FALSE) as BOOL;
        }
    }
    if (*requested).dwFlags
        & ((DDPF_RGB
            | DDPF_YUV
            | DDPF_STENCILBUFFER
            | DDPF_LUMINANCE
            | DDPF_BUMPDUDV) as DWORD)
        != 0
    {
        if (*requested).__bindgen_anon_2.dwRBitMask != (*provided).__bindgen_anon_2.dwRBitMask {
            return (FALSE) as BOOL;
        }
    }
    if (*requested).dwFlags
        & ((DDPF_RGB
            | DDPF_YUV
            | DDPF_ZBUFFER
            | DDPF_BUMPDUDV) as DWORD)
        != 0
    {
        if (*requested).__bindgen_anon_3.dwGBitMask != (*provided).__bindgen_anon_3.dwGBitMask {
            return (FALSE) as BOOL;
        }
    }
    /* I could be wrong about the bumpmapping. MSDN docs are vague. */
    if (*requested).dwFlags
        & ((DDPF_RGB
            | DDPF_YUV
            | DDPF_STENCILBUFFER
            | DDPF_BUMPDUDV) as DWORD)
        != 0
    {
        if (*requested).__bindgen_anon_4.dwBBitMask != (*provided).__bindgen_anon_4.dwBBitMask {
            return (FALSE) as BOOL;
        }
    }
    if (*requested).dwFlags
        & ((DDPF_ALPHAPIXELS | DDPF_ZPIXELS) as DWORD)
        != 0
    {
        if (*requested).__bindgen_anon_5.dwRGBAlphaBitMask
            != (*provided).__bindgen_anon_5.dwRGBAlphaBitMask
        {
            return (FALSE) as BOOL;
        }
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn ddraw_match_surface_desc(
    requested: *const DDSURFACEDESC2,
    provided: *const DDSURFACEDESC2,
) -> BOOL {
    #[allow(non_camel_case_types)]
    #[repr(C)]
    struct compare_info {
        flag: DWORD,
        offset: isize,
        size: usize,
    }

    // C: `#define CMP(FLAG, FIELD) { DDSD_##FLAG, offsetof(DDSURFACEDESC2, FIELD),
    // sizeof(((DDSURFACEDESC2 *)NULL)->FIELD) }`. Token pasting has no macro_rules! form, so
    // the table is written out: offset_of! through bindgen's anonymous unions, and each
    // field's own type for the size.
    static compare: [compare_info; 15] = [
        compare_info { flag: DDSD_ALPHABITDEPTH, offset: core::mem::offset_of!(DDSURFACEDESC2, dwAlphaBitDepth) as isize, size: core::mem::size_of::<DWORD>() },
        compare_info { flag: DDSD_BACKBUFFERCOUNT, offset: core::mem::offset_of!(DDSURFACEDESC2, __bindgen_anon_2.dwBackBufferCount) as isize, size: core::mem::size_of::<DWORD>() },
        compare_info { flag: DDSD_CAPS, offset: core::mem::offset_of!(DDSURFACEDESC2, ddsCaps) as isize, size: core::mem::size_of::<DDSCAPS2>() },
        compare_info { flag: DDSD_CKDESTBLT, offset: core::mem::offset_of!(DDSURFACEDESC2, ddckCKDestBlt) as isize, size: core::mem::size_of::<DDCOLORKEY>() },
        compare_info { flag: DDSD_CKDESTOVERLAY, offset: core::mem::offset_of!(DDSURFACEDESC2, __bindgen_anon_4.ddckCKDestOverlay) as isize, size: core::mem::size_of::<DDCOLORKEY>() },
        compare_info { flag: DDSD_CKSRCBLT, offset: core::mem::offset_of!(DDSURFACEDESC2, ddckCKSrcBlt) as isize, size: core::mem::size_of::<DDCOLORKEY>() },
        compare_info { flag: DDSD_CKSRCOVERLAY, offset: core::mem::offset_of!(DDSURFACEDESC2, ddckCKSrcOverlay) as isize, size: core::mem::size_of::<DDCOLORKEY>() },
        compare_info { flag: DDSD_HEIGHT, offset: core::mem::offset_of!(DDSURFACEDESC2, dwHeight) as isize, size: core::mem::size_of::<DWORD>() },
        compare_info { flag: DDSD_LINEARSIZE, offset: core::mem::offset_of!(DDSURFACEDESC2, __bindgen_anon_1.dwLinearSize) as isize, size: core::mem::size_of::<DWORD>() },
        compare_info { flag: DDSD_LPSURFACE, offset: core::mem::offset_of!(DDSURFACEDESC2, lpSurface) as isize, size: core::mem::size_of::<LPVOID>() },
        compare_info { flag: DDSD_MIPMAPCOUNT, offset: core::mem::offset_of!(DDSURFACEDESC2, __bindgen_anon_3.dwMipMapCount) as isize, size: core::mem::size_of::<DWORD>() },
        compare_info { flag: DDSD_PITCH, offset: core::mem::offset_of!(DDSURFACEDESC2, __bindgen_anon_1.lPitch) as isize, size: core::mem::size_of::<LONG>() },
        /* PIXELFORMAT: manual */
        compare_info { flag: DDSD_REFRESHRATE, offset: core::mem::offset_of!(DDSURFACEDESC2, __bindgen_anon_3.dwRefreshRate) as isize, size: core::mem::size_of::<DWORD>() },
        compare_info { flag: DDSD_TEXTURESTAGE, offset: core::mem::offset_of!(DDSURFACEDESC2, dwTextureStage) as isize, size: core::mem::size_of::<DWORD>() },
        compare_info { flag: DDSD_WIDTH, offset: core::mem::offset_of!(DDSURFACEDESC2, dwWidth) as isize, size: core::mem::size_of::<DWORD>() },
        /* ZBUFFERBITDEPTH: "obsolete" */
    ];

    let mut i: u32;

    if (*requested).dwFlags & (*provided).dwFlags != (*requested).dwFlags {
        return (FALSE) as BOOL;
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!(compare) as usize {
        if (*requested).dwFlags & compare[i as usize].flag != 0
            && memcmp(
                (provided as *const u8).offset(compare[i as usize].offset) as *const core::ffi::c_void,
                (requested as *const u8).offset(compare[i as usize].offset) as *const core::ffi::c_void,
                compare[i as usize].size,
            ) != 0
        {
            return (FALSE) as BOOL;
        }
        i = i.wrapping_add(1);
    }

    if (*requested).dwFlags & DDSD_PIXELFORMAT != 0 {
        if Main_DirectDraw_DDPIXELFORMAT_Match(
            core::ptr::addr_of!((*requested).__bindgen_anon_5.ddpfPixelFormat),
            core::ptr::addr_of!((*provided).__bindgen_anon_5.ddpfPixelFormat),
        ) == 0
        {
            return (FALSE) as BOOL;
        }
    }

    return (TRUE) as BOOL;
}

unsafe extern "system" fn EnumSurfacesCallback2Thunk(
    surface: *mut IDirectDrawSurface7,
    surface_desc: *mut DDSURFACEDESC2,
    context: *mut ::core::ffi::c_void,
) -> HRESULT {
    let surface_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface7(surface);
    let cbcontext: *mut surfacescallback2_context = context as *mut surfacescallback2_context;

    if surface.is_null() {
        return (*cbcontext).func.unwrap_unchecked()(
            core::ptr::null_mut::<IDirectDrawSurface4>(),
            surface_desc as LPDDSURFACEDESC2,
            (*cbcontext).context as LPVOID,
        );
    }

    IDirectDrawSurface4_AddRef(core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface4_iface));
    IDirectDrawSurface7_Release(surface);

    return (*cbcontext).func.unwrap_unchecked()(
        core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface4_iface),
        surface_desc as LPDDSURFACEDESC2,
        (*cbcontext).context as LPVOID,
    );
}

unsafe extern "system" fn EnumSurfacesCallbackThunk(
    surface: *mut IDirectDrawSurface7,
    surface_desc: *mut DDSURFACEDESC2,
    context: *mut core::ffi::c_void,
) -> HRESULT {
    let surface_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface7(surface);
    let cbcontext: *mut surfacescallback_context = context as *mut surfacescallback_context;

    if surface.is_null() {
        return (*cbcontext).func.unwrap_unchecked()(
            core::ptr::null_mut::<IDirectDrawSurface>(),
            surface_desc as LPDDSURFACEDESC,
            (*cbcontext).context as LPVOID,
        );
    }

    IDirectDrawSurface_AddRef(core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface_iface));
    IDirectDrawSurface7_Release(surface);

    return (*cbcontext).func.unwrap_unchecked()(
        core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface_iface),
        surface_desc as LPDDSURFACEDESC,
        (*cbcontext).context as LPVOID,
    );
}

unsafe extern "system" fn enum_surface_mode_callback(
    surface_desc: *mut DDSURFACEDESC2,
    context: *mut core::ffi::c_void,
) -> HRESULT {
    let params: *const enum_surface_mode_params = context as *const enum_surface_mode_params;
    let mut desc: DDSURFACEDESC2 = *(*params).desc;
    let mut surface: *mut IDirectDrawSurface7 = core::ptr::null_mut();

    desc.dwFlags |= DDSD_WIDTH | DDSD_HEIGHT | DDSD_PITCH | DDSD_PIXELFORMAT;
    desc.dwWidth = (*surface_desc).dwWidth;
    desc.dwHeight = (*surface_desc).dwHeight;
    desc.__bindgen_anon_1.lPitch = (*surface_desc).__bindgen_anon_1.lPitch;
    desc.__bindgen_anon_5.ddpfPixelFormat = (*surface_desc).__bindgen_anon_5.ddpfPixelFormat;

    if (ddraw7_CreateSurface(
        (*params).ddraw,
        core::ptr::addr_of_mut!(desc),
        core::ptr::addr_of_mut!(surface),
        core::ptr::null_mut(),
    )) >= 0
    {
        IDirectDrawSurface7_Release(surface);
        return (*params).callback.unwrap_unchecked()(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(desc),
            (*params).context,
        );
    }
    return (DDENUMRET_OK) as HRESULT;
}
unsafe extern "system" fn ddraw7_EnumSurfaces(
    mut iface: *mut IDirectDraw7,
    mut flags: DWORD,
    mut surface_desc: *mut DDSURFACEDESC2,
    mut context: *mut ::core::ffi::c_void,
    mut callback: LPDDENUMSURFACESCALLBACK7,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut hr: HRESULT = DD_OK;
    TRACE!(
        "iface %p, flags %#lx, surface_desc %p, context %p, callback %p.\n",
        iface,
        flags,
        surface_desc,
        context,
        callback
    );
    if callback.is_none() {
        return DDERR_INVALIDPARAMS;
    }
    if (flags & (DDENUMSURFACES_CANBECREATED as DWORD)) != 0 {
        let mut surface: *mut IDirectDrawSurface7 = ::core::ptr::null_mut::<IDirectDrawSurface7>();
        if (flags & ((DDENUMSURFACES_ALL | DDENUMSURFACES_MATCH | DDENUMSURFACES_NOMATCH) as DWORD))
            != (DDENUMSURFACES_MATCH as DWORD)
        {
            return DDERR_INVALIDPARAMS;
        }
        wined3d_mutex_lock();
        if ((*surface_desc).dwFlags & ((DDSD_WIDTH | DDSD_HEIGHT) as DWORD)) != 0 {
            if ddraw7_CreateSurface(
                iface,
                surface_desc,
                &raw mut surface,
                ::core::ptr::null_mut::<IUnknown>(),
            ) >= 0
            {
                let mut surface_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface7(surface);
                callback.unwrap_unchecked()(
                    ::core::ptr::null_mut::<IDirectDrawSurface7>(),
                    &raw mut (*surface_impl).surface_desc,
                    context as LPVOID,
                );
                IDirectDrawSurface7_Release(surface);
            }
        } else {
            let mut desc: DDSURFACEDESC2 = core::mem::zeroed::<DDSURFACEDESC2>();
            desc.dwSize = core::mem::size_of::<DDSURFACEDESC2>() as DWORD;
            desc.dwFlags = DDSD_PIXELFORMAT as DWORD;
            desc.__bindgen_anon_5.ddpfPixelFormat.dwSize =
                core::mem::size_of::<DDPIXELFORMAT>() as DWORD;
            let mut params: enum_surface_mode_params = enum_surface_mode_params {
                ddraw: iface,
                desc: surface_desc,
                callback: callback,
                context: context,
            };
            let mut mode: wined3d_display_mode = wined3d_display_mode {
                width: 0,
                height: 0,
                refresh_rate: 0,
                format_id: WINED3DFMT_UNKNOWN,
                scanline_ordering: WINED3D_SCANLINE_ORDERING_UNKNOWN,
            };
            hr = wined3d_output_get_display_mode(
                (*ddraw).wined3d_output,
                &raw mut mode,
                ::core::ptr::null_mut::<wined3d_display_rotation>(),
            );
            if hr < 0 {
                ERR!("Failed to get display mode, hr %#lx.\n", hr);
                wined3d_mutex_unlock();
                return hr_ddraw_from_wined3d(hr);
            }
            ddrawformat_from_wined3dformat(&raw mut desc.__bindgen_anon_5.ddpfPixelFormat, mode.format_id);
            hr = ddraw7_EnumDisplayModes(
                iface,
                0 as DWORD,
                &raw mut desc,
                &raw mut params as *mut ::core::ffi::c_void,
                Some(
                    enum_surface_mode_callback,
                ),
            );
        }
        wined3d_mutex_unlock();
    } else if (flags & (DDENUMSURFACES_DOESEXIST as DWORD)) != 0 {
        let mut surface_0: *mut ddraw_surface = ::core::ptr::null_mut::<ddraw_surface>();
        let mut cursor: *mut ddraw_surface = ::core::ptr::null_mut::<ddraw_surface>();
        let mut nomatch: BOOL = ((flags & (DDENUMSURFACES_NOMATCH as DWORD)) != 0) as ::core::ffi::c_int;
        wined3d_mutex_lock();
        surface_0 = ((*ddraw).surface_list.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(ddraw_surface, surface_list_entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut ddraw_surface;
        cursor = ((*surface_0).surface_list_entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(ddraw_surface, surface_list_entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut ddraw_surface;
        while &raw mut (*surface_0).surface_list_entry != &raw mut (*ddraw).surface_list {
            if (*surface_0).iface_count == 0 {
                WARN!(
                    "Not enumerating surface %p because it doesn't have any references.\n",
                    surface_0
                );
            } else if (flags & (DDENUMSURFACES_ALL as DWORD)) != 0
                || nomatch
                    != ddraw_match_surface_desc(surface_desc, &raw mut (*surface_0).surface_desc)
            {
                let mut desc_0: DDSURFACEDESC2 = (*surface_0).surface_desc;
                IDirectDrawSurface7_AddRef(&raw mut (*surface_0).IDirectDrawSurface7_iface);
                if callback.unwrap_unchecked()(
                    &raw mut (*surface_0).IDirectDrawSurface7_iface,
                    &raw mut desc_0,
                    context as LPVOID,
                ) != (DDENUMRET_OK as i32)
                {
                    break;
                }
            }
            surface_0 = cursor;
            cursor = ((*surface_0).surface_list_entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(ddraw_surface, surface_list_entry)
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut ddraw_surface;
        }
        wined3d_mutex_unlock();
    } else {
        return DDERR_INVALIDPARAMS;
    }
    return hr;
}
unsafe extern "system" fn ddraw4_EnumSurfaces(iface: *mut IDirectDraw4, flags: DWORD, surface_desc: *mut DDSURFACEDESC2, context: *mut core::ffi::c_void, callback: LPDDENUMSURFACESCALLBACK2) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    let mut cbcontext: surfacescallback2_context = surfacescallback2_context {
        func: None,
        context: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    WINE_TRACE!("iface %p, flags %#lx, surface_desc %p, context %p, callback %p.\n", iface, flags, surface_desc, context, callback);
    cbcontext.func = callback;
    cbcontext.context = context;

    return ddraw7_EnumSurfaces(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        flags,
        surface_desc,
        core::ptr::addr_of_mut!(cbcontext) as *mut core::ffi::c_void,
        Some(
            EnumSurfacesCallback2Thunk,
        ),
    );
}
unsafe extern "system" fn ddraw2_EnumSurfaces(
    iface: *mut IDirectDraw2,
    flags: DWORD,
    surface_desc: *mut DDSURFACEDESC,
    context: *mut core::ffi::c_void,
    callback: LPDDENUMSURFACESCALLBACK,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    let mut cbcontext: surfacescallback_context = core::mem::zeroed();
    let mut surface_desc2: DDSURFACEDESC2 = core::mem::zeroed();
    WINE_TRACE!("iface %p, flags %#lx, surface_desc %p, context %p, callback %p.\n", iface, flags, surface_desc, context, callback);
    cbcontext.func = callback;
    cbcontext.context = context;
    if !surface_desc.is_null() {
        DDSD_to_DDSD2(surface_desc, &raw mut surface_desc2);
    }
    return ddraw7_EnumSurfaces(
        &raw mut (*ddraw).IDirectDraw7_iface,
        flags,
        if !surface_desc.is_null() {
            &raw mut surface_desc2
        } else {
            core::ptr::null_mut::<DDSURFACEDESC2>()
        },
        &raw mut cbcontext as *mut core::ffi::c_void,
        Some(
            EnumSurfacesCallbackThunk,
        ),
    );
}
unsafe extern "system" fn ddraw1_EnumSurfaces(
    mut iface: *mut IDirectDraw,
    mut flags: DWORD,
    mut surface_desc: *mut DDSURFACEDESC,
    mut context: *mut ::core::ffi::c_void,
    mut callback: LPDDENUMSURFACESCALLBACK,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    let mut cbcontext: surfacescallback_context = core::mem::zeroed();
    let mut surface_desc2: DDSURFACEDESC2 = core::mem::zeroed();
    WINE_TRACE!("iface %p, flags %#lx, surface_desc %p, context %p, callback %p.\n", iface, flags, surface_desc, context, callback);
    cbcontext.func = callback;
    cbcontext.context = context;
    if !surface_desc.is_null() {
        DDSD_to_DDSD2(surface_desc, &raw mut surface_desc2);
    }
    return ddraw7_EnumSurfaces(
        &raw mut (*ddraw).IDirectDraw7_iface,
        flags,
        if !surface_desc.is_null() {
            &raw mut surface_desc2
        } else {
            ::core::ptr::null_mut::<DDSURFACEDESC2>()
        },
        &raw mut cbcontext as *mut ::core::ffi::c_void,
        Some(
            EnumSurfacesCallbackThunk,
        ),
    );
}

#[no_mangle]
pub unsafe extern "system" fn DirectDrawCreateClipper(
    flags: DWORD,
    clipper: *mut *mut IDirectDrawClipper,
    outer_unknown: *mut IUnknown,
) -> HRESULT {
    let mut object: *mut ddraw_clipper;
    let mut hr: HRESULT;

    if !outer_unknown.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    wined3d_mutex_lock();

    object = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<ddraw_clipper>() as usize) as _,
    ) as *mut ddraw_clipper;
    if object.is_null() {
        wined3d_mutex_unlock();
        return E_OUTOFMEMORY;
    }

    hr = ddraw_clipper_init(object);
    if (hr) < 0 {
        WARN!("Failed to initialize clipper, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        wined3d_mutex_unlock();
        return hr;
    }

    *clipper = core::ptr::addr_of_mut!((*object).IDirectDrawClipper_iface);
    wined3d_mutex_unlock();

    return DD_OK;
}
unsafe extern "system" fn ddraw7_CreateClipper(iface: *mut IDirectDraw7, Flags: DWORD, Clipper: *mut *mut IDirectDrawClipper, UnkOuter: *mut IUnknown) -> HRESULT {
    WINE_TRACE!("iface %p, flags %#lx, clipper %p, outer_unknown %p.\n", iface, Flags, Clipper, UnkOuter);
    return DirectDrawCreateClipper(Flags, Clipper, UnkOuter);
}
unsafe extern "system" fn ddraw4_CreateClipper(iface: *mut IDirectDraw4, flags: DWORD, clipper: *mut *mut IDirectDrawClipper, outer_unknown: *mut IUnknown) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);

    return ddraw7_CreateClipper(core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface), flags, clipper, outer_unknown);
}

unsafe extern "system" fn ddraw2_CreateClipper(
    iface: *mut IDirectDraw2,
    flags: DWORD,
    clipper: *mut *mut IDirectDrawClipper,
    outer_unknown: *mut IUnknown,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);

    return ddraw7_CreateClipper(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        flags,
        clipper,
        outer_unknown,
    );
}
unsafe extern "system" fn ddraw1_CreateClipper(
    iface: *mut IDirectDraw,
    flags: DWORD,
    clipper: *mut *mut IDirectDrawClipper,
    outer_unknown: *mut IUnknown,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    WINE_TRACE!("iface %p, flags %#lx, clipper %p, outer_unknown %p.\n", iface, flags, clipper, outer_unknown);
    return ddraw7_CreateClipper(
        &raw mut (*ddraw).IDirectDraw7_iface,
        flags,
        clipper,
        outer_unknown,
    );
}

unsafe extern "system" fn ddraw7_CreatePalette(iface: *mut IDirectDraw7, Flags: DWORD, ColorTable: *mut PALETTEENTRY, Palette: *mut *mut IDirectDrawPalette, pUnkOuter: *mut IUnknown) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw7(iface);
    let mut object: *mut ddraw_palette = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if !pUnkOuter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    wined3d_mutex_lock();

    /* The refcount test shows that a cooplevel is required for this */
    if (*ddraw).cooperative_level == 0 {
        WARN!("No cooperative level set, returning DDERR_NOCOOPERATIVELEVELSET\n");
        wined3d_mutex_unlock();
        return DDERR_NOCOOPERATIVELEVELSET;
    }

    object = malloc(core::mem::size_of::<ddraw_palette>()) as *mut ddraw_palette;
    if object.is_null() {
        ERR!("Out of memory when allocating memory for a palette implementation\n");
        wined3d_mutex_unlock();
        return E_OUTOFMEMORY;
    }

    hr = ddraw_palette_init(object, ddraw, Flags, ColorTable);
    if (hr) < 0 {
        WARN!("Failed to initialize palette, hr %#lx.\n", hr);
        free((object).cast());
        wined3d_mutex_unlock();
        return hr;
    }

    *Palette = core::ptr::addr_of_mut!((*object).IDirectDrawPalette_iface);
    wined3d_mutex_unlock();

    return DD_OK;
}

unsafe extern "system" fn ddraw4_CreatePalette(
    iface: *mut IDirectDraw4,
    flags: DWORD,
    entries: *mut PALETTEENTRY,
    palette: *mut *mut IDirectDrawPalette,
    outer_unknown: *mut IUnknown,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    let mut hr: HRESULT;

    TRACE!(
        "iface %p, flags %#lx, entries %p, palette %p, outer_unknown %p.\n",
        iface,
        flags,
        entries,
        palette,
        outer_unknown
    );

    hr = ddraw7_CreatePalette(
        &raw mut (*ddraw).IDirectDraw7_iface,
        flags,
        entries,
        palette,
        outer_unknown,
    );
    if (hr >= 0) && !(*palette).is_null() {
        let impl_0: *mut ddraw_palette = impl_from_IDirectDrawPalette(*palette);
        IDirectDraw7_Release(&raw mut (*ddraw).IDirectDraw7_iface);
        IDirectDraw4_AddRef(iface);
        (*impl_0).ifaceToRelease = iface as *mut IUnknown;
    }
    return hr;
}
unsafe extern "system" fn ddraw2_CreatePalette(
    iface: *mut IDirectDraw2,
    flags: DWORD,
    entries: *mut PALETTEENTRY,
    palette: *mut *mut IDirectDrawPalette,
    outer_unknown: *mut IUnknown,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    let mut hr: HRESULT;

    hr = ddraw7_CreatePalette(&raw mut (*ddraw).IDirectDraw7_iface, flags, entries, palette, outer_unknown);
    if ((hr) >= 0) && !(*palette).is_null() {
        let impl_0: *mut ddraw_palette = impl_from_IDirectDrawPalette(*palette);
        IDirectDraw7_Release(&raw mut (*ddraw).IDirectDraw7_iface);
        (*impl_0).ifaceToRelease = core::ptr::null_mut();
    }

    return hr;
}

unsafe extern "system" fn ddraw1_CreatePalette(
    iface: *mut IDirectDraw,
    flags: DWORD,
    entries: *mut PALETTEENTRY,
    palette: *mut *mut IDirectDrawPalette,
    outer_unknown: *mut IUnknown,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    let mut hr: HRESULT = 0;

    hr = ddraw7_CreatePalette(
        &raw mut (*ddraw).IDirectDraw7_iface,
        flags,
        entries,
        palette,
        outer_unknown,
    );
    if hr >= 0 && !(*palette).is_null() {
        let impl_: *mut ddraw_palette = impl_from_IDirectDrawPalette(*palette);
        IDirectDraw7_Release(&raw mut (*ddraw).IDirectDraw7_iface);
        (*impl_).ifaceToRelease = core::ptr::null_mut::<IUnknown>();
    }

    return hr;
}

unsafe extern "system" fn ddraw7_DuplicateSurface(
    iface: *mut IDirectDraw7,
    Src: *mut IDirectDrawSurface7,
    Dest: *mut *mut IDirectDrawSurface7,
) -> HRESULT {
    let src_surface: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(Src);

    FIXME!(
        "iface %p, src %p, dst %p partial stub!\n",
        iface,
        Src,
        Dest
    );

    /* For now, simply create a new, independent surface */
    return IDirectDraw7_CreateSurface(
        iface,
        core::ptr::addr_of_mut!((*src_surface).surface_desc),
        Dest,
        core::ptr::null_mut(),
    );
}

unsafe extern "system" fn ddraw4_DuplicateSurface(iface: *mut IDirectDraw4, src: *mut IDirectDrawSurface4, dst: *mut *mut IDirectDrawSurface4) -> HRESULT {
    let mut src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface4(src);
    let mut ddraw: *mut ddraw = impl_from_IDirectDraw4(iface);
    let mut dst_impl: *mut ddraw_surface = core::ptr::null_mut();
    let mut dst7: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    hr = ddraw7_DuplicateSurface(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        if !src_impl.is_null() {
            core::ptr::addr_of_mut!((*src_impl).IDirectDrawSurface7_iface)
        } else {
            core::ptr::null_mut::<IDirectDrawSurface7>()
        },
        core::ptr::addr_of_mut!(dst7),
    );
    if hr < 0 {
        *dst = core::ptr::null_mut::<IDirectDrawSurface4>();
        return hr;
    }
    dst_impl = impl_from_IDirectDrawSurface7(dst7);
    *dst = core::ptr::addr_of_mut!((*dst_impl).IDirectDrawSurface4_iface);
    IDirectDrawSurface4_AddRef(*dst);
    IDirectDrawSurface7_Release(dst7);

    return hr;
}

unsafe extern "system" fn ddraw2_DuplicateSurface(iface: *mut IDirectDraw2, src: *mut IDirectDrawSurface, dst: *mut *mut IDirectDrawSurface) -> HRESULT {
    let src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface(src);
    let ddraw: *mut ddraw = impl_from_IDirectDraw2(iface);
    let mut dst_impl: *mut ddraw_surface = core::ptr::null_mut();
    let mut dst7: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    hr = ddraw7_DuplicateSurface(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        if !src_impl.is_null() {
            core::ptr::addr_of_mut!((*src_impl).IDirectDrawSurface7_iface)
        } else {
            core::ptr::null_mut()
        },
        &mut dst7,
    );
    if hr < 0 {
        return hr;
    }
    dst_impl = impl_from_IDirectDrawSurface7(dst7);
    *dst = core::ptr::addr_of_mut!((*dst_impl).IDirectDrawSurface_iface);
    IDirectDrawSurface_AddRef(*dst);
    IDirectDrawSurface7_Release(dst7);

    return hr;
}
unsafe extern "system" fn ddraw1_DuplicateSurface(
    iface: *mut IDirectDraw,
    src: *mut IDirectDrawSurface,
    dst: *mut *mut IDirectDrawSurface,
) -> HRESULT {
    let src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface(src);
    let ddraw: *mut ddraw = impl_from_IDirectDraw(iface);
    let mut dst_impl: *mut ddraw_surface = core::ptr::null_mut::<ddraw_surface>();
    let mut dst7: *mut IDirectDrawSurface7 = core::ptr::null_mut::<IDirectDrawSurface7>();
    let mut hr: HRESULT = 0;

    hr = ddraw7_DuplicateSurface(
        core::ptr::addr_of_mut!((*ddraw).IDirectDraw7_iface),
        if !src_impl.is_null() {
            core::ptr::addr_of_mut!((*src_impl).IDirectDrawSurface7_iface)
        } else {
            core::ptr::null_mut::<IDirectDrawSurface7>()
        },
        core::ptr::addr_of_mut!(dst7),
    );
    if hr < 0 {
        return hr;
    }
    dst_impl = impl_from_IDirectDrawSurface7(dst7);
    *dst = core::ptr::addr_of_mut!((*dst_impl).IDirectDrawSurface_iface);
    IDirectDrawSurface_AddRef(*dst);
    IDirectDrawSurface7_Release(dst7);

    return hr;
}
unsafe extern "system" fn d3d7_EnumDevices(iface: *mut IDirect3D7, callback: LPD3DENUMDEVICESCALLBACK7, context: *mut core::ffi::c_void) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D7(iface);
    let mut device_desc7: D3DDEVICEDESC7 = core::mem::zeroed();
    let mut dev_caps: DWORD;
    let mut hr: HRESULT;
    let mut i: usize;

    if callback.is_none() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    hr = ddraw_get_d3dcaps(ddraw, core::ptr::addr_of_mut!(device_desc7));
    if hr < 0 {
        wined3d_mutex_unlock();
        return hr;
    }
    dev_caps = device_desc7.dwDevCaps;

    i = 0 as usize;
    while i < ARRAY_SIZE!(device_list7) as usize {
        let mut ret: HRESULT;

        device_desc7.deviceGUID = *device_list7[i].device_guid;
        device_desc7.dwDevCaps = dev_caps & !device_list7[i].unsupported_caps;
        ret = (callback.unwrap_unchecked())(
            core::ptr::addr_of_mut!(device_list7[i].interface_name) as *mut core::ffi::c_char,
            core::ptr::addr_of_mut!(device_list7[i].device_name) as *mut core::ffi::c_char,
            core::ptr::addr_of_mut!(device_desc7),
            context,
        );
        if ret != (DDENUMRET_OK as HRESULT) {
            WINE_TRACE!("Application cancelled the enumeration.\n");
            wined3d_mutex_unlock();
            return D3D_OK;
        }
        i = i.wrapping_add(1);
    }

    WINE_TRACE!("End of enumeration.\n");

    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d3_EnumDevices(
    iface: *mut IDirect3D3,
    callback: LPD3DENUMDEVICESCALLBACK,
    context: *mut ::core::ffi::c_void,
) -> HRESULT {
    static mut wined3d_description: [CHAR; 40] = unsafe {
        ::core::mem::transmute::<[u8; 40], [CHAR; 40]>(
            *b"Wine D3DDevice using WineD3D and OpenGL\0",
        )
    };

    let ddraw: *mut ddraw = impl_from_IDirect3D3(iface);
    let mut device_desc1: D3DDEVICEDESC = core::mem::zeroed();
    let mut hal_desc: D3DDEVICEDESC = core::mem::zeroed();
    let mut hel_desc: D3DDEVICEDESC = core::mem::zeroed();
    let mut device_desc7: D3DDEVICEDESC7 = core::mem::zeroed();
    let mut hr: HRESULT;

    /* Some games (Motoracer 2 demo) have the bad idea to modify the device
     * name string. Let's put the string in a sufficiently sized array in
     * writable memory. */
    let mut device_name: [::core::ffi::c_char; 50] =
        [0; 50];
    strcpy(
        core::ptr::addr_of_mut!(device_name) as *mut ::core::ffi::c_char,
        b"Direct3D HEL\0".as_ptr() as *const ::core::ffi::c_char,
    );

    if callback.is_none() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    hr = ddraw_get_d3dcaps(ddraw, core::ptr::addr_of_mut!(device_desc7));
    if hr < 0 {
        wined3d_mutex_unlock();
        return hr;
    }
    ddraw_d3dcaps1_from_7(
        core::ptr::addr_of_mut!(device_desc1),
        core::ptr::addr_of_mut!(device_desc7),
    );

    /* Do I have to enumerate the reference id? Note from old d3d7:
     * "It seems that enumerating the reference IID on Direct3D 1 games
     * (AvP / Motoracer2) breaks them". So do not enumerate this iid in V1
     *
     * There's a registry key HKLM\Software\Microsoft\Direct3D\Drivers,
     * EnumReference which enables / disables enumerating the reference
     * rasterizer. It's a DWORD, 0 means disabled, 2 means enabled. The
     * enablerefrast.reg and disablerefrast.reg files in the DirectX 7.0 sdk
     * demo directory suggest this.
     *
     * Some games(GTA 2) seem to use the second enumerated device, so I have
     * to enumerate at least 2 devices. So enumerate the reference device to
     * have 2 devices.
     *
     * Other games (Rollcage) tell emulation and hal device apart by certain
     * flags. Rollcage expects D3DPTEXTURECAPS_POW2 to be set (yeah, it is a
     * limitation flag), and it refuses all devices that have the perspective
     * flag set. This way it refuses the emulation device, and HAL devices
     * never have POW2 unset in d3d7 on windows. */
    if (*ddraw).d3dversion != 1 {
        /* Tomb Raider 3 overwrites the reference device description buffer
         * with its own custom string. Reserve some extra space in the array
         * to avoid a buffer overrun. */
        static mut reference_description: [CHAR; 64] = unsafe {
            ::core::mem::transmute::<[u8; 64], [CHAR; 64]>(
                *b"RGB Direct3D emulation\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            )
        };

        hal_desc = device_desc1;
        hel_desc = device_desc1;
        /* The rgb device has the pow2 flag set in the hel caps, but not in the hal caps. */
        hal_desc.dpcLineCaps.dwTextureCaps &= !(D3DPTEXTURECAPS_POW2
            | (D3DPTEXTURECAPS_NONPOW2CONDITIONAL as u32)
            | D3DPTEXTURECAPS_PERSPECTIVE) as DWORD;
        hal_desc.dpcTriCaps.dwTextureCaps &= !(D3DPTEXTURECAPS_POW2
            | (D3DPTEXTURECAPS_NONPOW2CONDITIONAL as u32)
            | D3DPTEXTURECAPS_PERSPECTIVE) as DWORD;
        /* RGB, RAMP and MMX devices have a HAL dcmColorModel of 0 */
        hal_desc.dcmColorModel = 0 as D3DCOLORMODEL;
        /* RGB, RAMP and MMX devices cannot report HAL hardware flags */
        hal_desc.dwFlags = 0 as DWORD;
        /* RGB, REF, RAMP and MMX devices don't report hardware transform and lighting capability */
        hal_desc.dwDevCaps &= !(D3DDEVCAPS_HWTRANSFORMANDLIGHT
            | D3DDEVCAPS_DRAWPRIMITIVES2EX
            | D3DDEVCAPS_HWRASTERIZATION) as DWORD;
        hel_desc.dwDevCaps &= !(D3DDEVCAPS_HWTRANSFORMANDLIGHT
            | D3DDEVCAPS_DRAWPRIMITIVES2EX
            | D3DDEVCAPS_HWRASTERIZATION) as DWORD;

        hr = (callback.unwrap_unchecked())(
            &raw const IID_IDirect3DRGBDevice as *mut GUID,
            core::ptr::addr_of_mut!(reference_description) as *mut ::core::ffi::c_char,
            core::ptr::addr_of_mut!(device_name) as *mut ::core::ffi::c_char,
            core::ptr::addr_of_mut!(hal_desc),
            core::ptr::addr_of_mut!(hel_desc),
            context,
        );
        if hr != (D3DENUMRET_OK as HRESULT) {
            wined3d_mutex_unlock();
            return D3D_OK;
        }
    }

    strcpy(
        core::ptr::addr_of_mut!(device_name) as *mut ::core::ffi::c_char,
        b"Direct3D HAL\0".as_ptr() as *const ::core::ffi::c_char,
    );

    hal_desc = device_desc1;
    hel_desc = device_desc1;

    /* The hal device does not have the pow2 flag set in hel, but in hal. */
    hel_desc.dpcLineCaps.dwTextureCaps &= !(D3DPTEXTURECAPS_POW2
        | (D3DPTEXTURECAPS_NONPOW2CONDITIONAL as u32)
        | D3DPTEXTURECAPS_PERSPECTIVE) as DWORD;
    hel_desc.dpcTriCaps.dwTextureCaps &= !(D3DPTEXTURECAPS_POW2
        | (D3DPTEXTURECAPS_NONPOW2CONDITIONAL as u32)
        | D3DPTEXTURECAPS_PERSPECTIVE) as DWORD;
    /* HAL devices have a HEL dcmColorModel of 0 */
    hel_desc.dcmColorModel = 0 as D3DCOLORMODEL;
    /* HAL devices report hardware transform and lighting capability, but not in hel */
    hel_desc.dwDevCaps &= !(D3DDEVCAPS_HWTRANSFORMANDLIGHT | D3DDEVCAPS_DRAWPRIMITIVES2EX) as DWORD;

    hr = (callback.unwrap_unchecked())(
        &raw const IID_IDirect3DHALDevice as *mut GUID,
        core::ptr::addr_of_mut!(wined3d_description) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!(device_name) as *mut ::core::ffi::c_char,
        core::ptr::addr_of_mut!(hal_desc),
        core::ptr::addr_of_mut!(hel_desc),
        context,
    );
    if hr != (D3DENUMRET_OK as HRESULT) {
        wined3d_mutex_unlock();
        return D3D_OK;
    }

    wined3d_mutex_unlock();

    D3D_OK
}

unsafe extern "system" fn d3d2_EnumDevices(iface: *mut IDirect3D2, callback: LPD3DENUMDEVICESCALLBACK, context: *mut core::ffi::c_void) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirect3D2(iface);
    WINE_TRACE!("iface %p, callback %p, context %p.\n", iface, callback, context);
    return d3d3_EnumDevices(core::ptr::addr_of_mut!((*ddraw).IDirect3D3_iface), callback, context);
}

unsafe extern "system" fn d3d1_EnumDevices(iface: *mut IDirect3D, callback: LPD3DENUMDEVICESCALLBACK, context: *mut core::ffi::c_void) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirect3D(iface);

    return d3d3_EnumDevices(core::ptr::addr_of_mut!((*ddraw).IDirect3D3_iface), callback, context);
}

unsafe extern "system" fn d3d3_CreateLight(iface: *mut IDirect3D3, light: *mut *mut IDirect3DLight, outer_unknown: *mut IUnknown) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirect3D3(iface);
    let mut object: *mut d3d_light;

    TRACE!("iface %p, light %p, outer_unknown %p.\n", iface, light, outer_unknown);

    if !outer_unknown.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    object = calloc(1, (core::mem::size_of::<d3d_light>()) as _) as *mut d3d_light;
    if object.is_null() {
        ERR!("Failed to allocate light memory.\n");
        return DDERR_OUTOFMEMORY;
    }

    d3d_light_init(object, ddraw);

    TRACE!("Created light %p.\n", object);

    *light = core::ptr::addr_of_mut!((*object).IDirect3DLight_iface);

    return D3D_OK;
}
unsafe extern "system" fn d3d2_CreateLight(
    mut iface: *mut IDirect3D2,
    mut light: *mut *mut IDirect3DLight,
    mut outer_unknown: *mut IUnknown,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirect3D2(iface);
    WINE_TRACE!("iface %p, light %p, outer_unknown %p.\n", iface, light, outer_unknown);
    return d3d3_CreateLight(&raw mut (*ddraw).IDirect3D3_iface, light, outer_unknown);
}

unsafe extern "system" fn d3d1_CreateLight(iface: *mut IDirect3D, light: *mut *mut IDirect3DLight, outer_unknown: *mut IUnknown) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirect3D(iface);

    return d3d3_CreateLight(core::ptr::addr_of_mut!((*ddraw).IDirect3D3_iface), light, outer_unknown);
}

unsafe extern "system" fn d3d3_CreateMaterial(iface: *mut IDirect3D3, material: *mut *mut IDirect3DMaterial3, outer_unknown: *mut IUnknown) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D3(iface);
    let mut object: *mut d3d_material;

    if !outer_unknown.is_null() { return CLASS_E_NOAGGREGATION; }

    object = d3d_material_create(ddraw);
    if object.is_null()
    {
        ERR!("Failed to allocate material memory.\n");
        return DDERR_OUTOFMEMORY;
    }

    *material = core::ptr::addr_of_mut!((*object).IDirect3DMaterial3_iface);

    return D3D_OK;
}

unsafe extern "system" fn d3d2_CreateMaterial(iface: *mut IDirect3D2, material: *mut *mut IDirect3DMaterial2, outer_unknown: *mut IUnknown) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D2(iface);
    let mut object: *mut d3d_material;

    object = d3d_material_create(ddraw);
    if object.is_null() {
        ERR!("Failed to allocate material memory.\n");
        return DDERR_OUTOFMEMORY;
    }

    *material = core::ptr::addr_of_mut!((*object).IDirect3DMaterial2_iface);

    return D3D_OK;
}

unsafe extern "system" fn d3d1_CreateMaterial(iface: *mut IDirect3D, material: *mut *mut IDirect3DMaterial, outer_unknown: *mut IUnknown) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D(iface);
    let mut object: *mut d3d_material;

    object = d3d_material_create(ddraw);
    if object.is_null() {
        ERR!("Failed to allocate material memory.\n");
        return DDERR_OUTOFMEMORY;
    }

    *material = core::ptr::addr_of_mut!((*object).IDirect3DMaterial_iface);

    return D3D_OK;
}

unsafe extern "system" fn d3d3_CreateViewport(iface: *mut IDirect3D3, viewport: *mut *mut IDirect3DViewport3, outer_unknown: *mut IUnknown) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirect3D3(iface);
    let mut object: *mut d3d_viewport;

    if !outer_unknown.is_null() { return CLASS_E_NOAGGREGATION; }

    object = calloc(1, (core::mem::size_of::<d3d_viewport>()) as _) as *mut d3d_viewport;
    if object.is_null()
    {
        ERR!("Failed to allocate viewport memory.\n");
        return DDERR_OUTOFMEMORY;
    }

    d3d_viewport_init(object, ddraw);

    *viewport = core::ptr::addr_of_mut!((*object).IDirect3DViewport3_iface);

    return D3D_OK;
}

unsafe extern "system" fn d3d2_CreateViewport(iface: *mut IDirect3D2, viewport: *mut *mut IDirect3DViewport2, outer_unknown: *mut IUnknown) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D2(iface);

    return d3d3_CreateViewport(core::ptr::addr_of_mut!((*ddraw).IDirect3D3_iface), viewport as *mut *mut IDirect3DViewport3, outer_unknown);
}

unsafe extern "system" fn d3d1_CreateViewport(
    iface: *mut IDirect3D,
    viewport: *mut *mut IDirect3DViewport,
    outer_unknown: *mut IUnknown,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirect3D(iface);
    WINE_TRACE!("iface %p, viewport %p, outer_unknown %p.\n", iface, viewport, outer_unknown);
    return d3d3_CreateViewport(
        &raw mut (*ddraw).IDirect3D3_iface,
        viewport as *mut *mut IDirect3DViewport3,
        outer_unknown,
    );
}

unsafe extern "C" fn ddraw_find_device(
    mut ddraw: *mut ddraw,
    mut fds: *const D3DFINDDEVICESEARCH,
    mut fdr: *mut D3DFINDDEVICERESULT,
    mut guid_count: ::core::ffi::c_uint,
    mut guids: *const *const GUID,
    mut device_desc_size: DWORD,
) -> HRESULT {
    let mut fdr1: *mut ddraw_find_device_result_v1 = core::ptr::null_mut();
    let mut fdr2: *mut ddraw_find_device_result_v2 = core::ptr::null_mut();
    let mut desc7: D3DDEVICEDESC7 = core::mem::zeroed();
    let mut desc1: D3DDEVICEDESC = core::mem::zeroed();
    let mut i: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    if fds.is_null() || fdr.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if (*fds).dwSize as usize != core::mem::size_of::<D3DFINDDEVICESEARCH>() as usize {
        WARN!("Got invalid search structure size %lu.\n", (*fds).dwSize);
        return DDERR_INVALIDPARAMS;
    }

    if (*fdr).dwSize as usize != core::mem::size_of::<D3DFINDDEVICERESULT>() as usize
        && (*fdr).dwSize as usize != core::mem::size_of::<ddraw_find_device_result_v2>() as usize
        && (*fdr).dwSize as usize != core::mem::size_of::<ddraw_find_device_result_v1>() as usize
    {
        WARN!("Got invalid result structure size %lu.\n", (*fdr).dwSize);
        return DDERR_INVALIDPARAMS;
    }

    if ((*fds).dwFlags & D3DFDS_COLORMODEL) != 0 {
        WARN!("Ignoring colour model %#lx.\n", (*fds).dcmColorModel);
    }

    if ((*fds).dwFlags & D3DFDS_GUID) != 0 {
        let mut found: BOOL = (FALSE as i32);

        if (((*ddraw).flags & DDRAW_NO3D) != 0)
            && IsEqualGUID!(&raw const (*fds).guid, &raw const IID_IDirect3DHALDevice) != false
        {
            WARN!("HAL device not available without 3D support.\n");
            return DDERR_NOTFOUND;
        }

        i = 0;
        while i < guid_count {
            if IsEqualGUID!(*guids.offset(i as isize), &raw const (*fds).guid) != false {
                found = (TRUE as i32);
                break;
            }
            i = i.wrapping_add(1);
        }

        if found == 0 {
            WARN!(
                "Failed to match GUID %s.\n",
                debugstr_guid(&raw const (*fds).guid)
            );
            return DDERR_NOTFOUND;
        }
    }

    /* Get the caps */
    hr = ddraw_get_d3dcaps(ddraw, &raw mut desc7);
    if hr < 0 {
        return hr;
    }

    /* Now return our own GUID */
    ddraw_d3dcaps1_from_7(&raw mut desc1, &raw mut desc7);
    (*fdr).guid = IID_D3DDEVICE_WineD3D;

    /* Note that "device_desc_size" doesn't necessarily have any relation to
     * the actual structure size. However, this matches the behaviour of
     * Windows since at least Windows 2000. */
    if (*fdr).dwSize as usize == core::mem::size_of::<ddraw_find_device_result_v1>() as usize {
        fdr1 = fdr as *mut ddraw_find_device_result_v1 as *mut ddraw_find_device_result_v1;
        memcpy(
            &raw mut (*fdr1).hw_desc as *mut ::core::ffi::c_void,
            &raw mut desc1 as *const ::core::ffi::c_void,
            core::mem::size_of::<D3DDEVICEDESC_V1>() as usize,
        );
        (*fdr1).hw_desc.dwSize = device_desc_size;
        memcpy(
            &raw mut (*fdr1).sw_desc as *mut ::core::ffi::c_void,
            &raw mut desc1 as *const ::core::ffi::c_void,
            core::mem::size_of::<D3DDEVICEDESC_V1>() as usize,
        );
        (*fdr1).sw_desc.dwSize = device_desc_size;
    } else if (*fdr).dwSize as usize
        == core::mem::size_of::<ddraw_find_device_result_v2>() as usize
    {
        fdr2 = fdr as *mut ddraw_find_device_result_v2 as *mut ddraw_find_device_result_v2;
        memcpy(
            &raw mut (*fdr2).hw_desc as *mut ::core::ffi::c_void,
            &raw mut desc1 as *const ::core::ffi::c_void,
            core::mem::size_of::<D3DDEVICEDESC_V2>() as usize,
        );
        (*fdr2).hw_desc.dwSize = device_desc_size;
        memcpy(
            &raw mut (*fdr2).sw_desc as *mut ::core::ffi::c_void,
            &raw mut desc1 as *const ::core::ffi::c_void,
            core::mem::size_of::<D3DDEVICEDESC_V2>() as usize,
        );
        (*fdr2).sw_desc.dwSize = device_desc_size;
    } else {
        (*fdr).ddHwDesc = desc1;
        (*fdr).ddHwDesc.dwSize = device_desc_size;
        (*fdr).ddSwDesc = desc1;
        (*fdr).ddSwDesc.dwSize = device_desc_size;
    }

    return D3D_OK;
}

unsafe extern "system" fn d3d3_FindDevice(iface: *mut IDirect3D3, fds: *mut D3DFINDDEVICESEARCH, fdr: *mut D3DFINDDEVICERESULT) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D3(iface);
    static mut guids: [*const GUID; 3] = unsafe {
        [
            core::ptr::addr_of!(IID_D3DDEVICE_WineD3D),
            core::ptr::addr_of!(IID_IDirect3DHALDevice),
            core::ptr::addr_of!(IID_IDirect3DRGBDevice),
        ]
    };

    TRACE!("iface %p, fds %p, fdr %p.\n", iface, fds, fdr);

    return ddraw_find_device(
        ddraw,
        fds,
        fdr,
        ARRAY_SIZE!(guids) as core::ffi::c_uint,
        core::ptr::addr_of!(guids) as *const *const GUID,
        core::mem::size_of::<D3DDEVICEDESC_V3>() as DWORD,
    );
}

unsafe extern "system" fn d3d2_FindDevice(
    iface: *mut IDirect3D2,
    fds: *mut D3DFINDDEVICESEARCH,
    fdr: *mut D3DFINDDEVICERESULT,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D2(iface);
    static mut guids: [*const GUID; 5] = unsafe {
        [
            &raw const IID_D3DDEVICE_WineD3D,
            &raw const IID_IDirect3DHALDevice,
            &raw const IID_IDirect3DMMXDevice,
            &raw const IID_IDirect3DRGBDevice,
            &raw const IID_IDirect3DRampDevice,
        ]
    };

    return ddraw_find_device(
        ddraw,
        fds,
        fdr,
        ARRAY_SIZE!(guids) as ::core::ffi::c_uint,
        &raw const guids as *const *const GUID,
        (core::mem::size_of::<D3DDEVICEDESC_V2>()) as DWORD,
    );
}

unsafe extern "system" fn d3d1_FindDevice(
    iface: *mut IDirect3D,
    fds: *mut D3DFINDDEVICESEARCH,
    fdr: *mut D3DFINDDEVICERESULT,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D(iface);
    static mut guids: [*const GUID; 4] = [
        core::ptr::addr_of!(IID_D3DDEVICE_WineD3D),
        core::ptr::addr_of!(IID_IDirect3DHALDevice),
        core::ptr::addr_of!(IID_IDirect3DRGBDevice),
        core::ptr::addr_of!(IID_IDirect3DRampDevice),
    ];

    return ddraw_find_device(
        ddraw,
        fds,
        fdr,
        ARRAY_SIZE!(guids) as u32,
        core::ptr::addr_of!(guids) as *const *const GUID,
        core::mem::size_of::<D3DDEVICEDESC_V1>() as DWORD,
    );
}

unsafe extern "system" fn d3d7_CreateDevice(
    iface: *mut IDirect3D7,
    riid: *const CLSID,
    surface: *mut IDirectDrawSurface7,
    device: *mut *mut IDirect3DDevice7,
) -> HRESULT {
    let mut target: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(surface);
    let mut ddraw: *mut ddraw = impl_from_IDirect3D7(iface);
    let mut object: *mut d3d_device = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!(
        "iface %p, riid %s, surface %p, device %p.\n",
        iface,
        debugstr_guid(riid as *const GUID),
        surface,
        device
    );
    wined3d_mutex_lock();
    hr = d3d_device_create(
        ddraw,
        riid as *const GUID,
        target,
        surface as *mut IUnknown,
        7 as UINT,
        core::ptr::addr_of_mut!(object),
        core::ptr::null_mut::<IUnknown>(),
    );
    if hr >= 0 {
        *device = core::ptr::addr_of_mut!((*object).IDirect3DDevice7_iface);
    } else {
        WARN!("Failed to create device, hr %#lx.\n", hr);
        *device = core::ptr::null_mut::<IDirect3DDevice7>();
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d3_CreateDevice(
    iface: *mut IDirect3D3,
    riid: *const GUID,
    surface: *mut IDirectDrawSurface4,
    device: *mut *mut IDirect3DDevice3,
    outer_unknown: *mut IUnknown,
) -> HRESULT {
    let mut surface_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface4(surface);
    let mut ddraw: *mut ddraw = impl_from_IDirect3D3(iface);
    let mut device_impl: *mut d3d_device = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!(
        "iface %p, riid %s, surface %p, device %p, outer_unknown %p.\n",
        iface,
        debugstr_guid(riid),
        surface,
        device,
        outer_unknown
    );

    if !outer_unknown.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    wined3d_mutex_lock();
    hr = d3d_device_create(
        ddraw,
        riid,
        surface_impl,
        surface as *mut IUnknown,
        3,
        core::ptr::addr_of_mut!(device_impl),
        core::ptr::null_mut::<IUnknown>(),
    );
    if hr >= 0 {
        *device = core::ptr::addr_of_mut!((*device_impl).IDirect3DDevice3_iface);
    } else {
        WARN!("Failed to create device, hr %#lx.\n", hr);
        *device = core::ptr::null_mut::<IDirect3DDevice3>();
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d2_CreateDevice(iface: *mut IDirect3D2, riid: *const CLSID, surface: *mut IDirectDrawSurface, device: *mut *mut IDirect3DDevice2) -> HRESULT {
    let surface_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface(surface);
    let ddraw: *mut ddraw = impl_from_IDirect3D2(iface);
    let mut device_impl: *mut d3d_device = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();
    TRACE!(
        "iface %p, riid %s, surface %p, device %p.\n",
        iface,
        debugstr_guid(riid as *const GUID),
        surface,
        device
    );
    hr = d3d_device_create(
        ddraw,
        riid as *const GUID,
        surface_impl,
        surface as *mut IUnknown,
        2 as UINT,
        core::ptr::addr_of_mut!(device_impl),
        core::ptr::null_mut::<IUnknown>(),
    );
    if hr >= 0 {
        *device = core::ptr::addr_of_mut!((*device_impl).IDirect3DDevice2_iface);
    } else {
        WARN!("Failed to create device, hr %#lx.\n", hr);
        *device = core::ptr::null_mut::<IDirect3DDevice2>();
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d7_CreateVertexBuffer(
    iface: *mut IDirect3D7,
    desc: *mut D3DVERTEXBUFFERDESC,
    vertex_buffer: *mut *mut IDirect3DVertexBuffer7,
    flags: DWORD,
) -> HRESULT {
    let mut ddraw: *mut ddraw = impl_from_IDirect3D7(iface);
    let mut object: *mut d3d_vertex_buffer = core::ptr::null_mut();
    let mut hr: HRESULT;

    if vertex_buffer.is_null() || desc.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    hr = d3d_vertex_buffer_create(core::ptr::addr_of_mut!(object), ddraw, desc);
    if hr == D3D_OK {
        *vertex_buffer = core::ptr::addr_of_mut!((*object).IDirect3DVertexBuffer7_iface);
    } else {
        WARN!("Failed to create vertex buffer, hr %#lx.\n", hr);
    }

    return hr;
}

unsafe extern "system" fn d3d3_CreateVertexBuffer(iface: *mut IDirect3D3, desc: *mut D3DVERTEXBUFFERDESC, vertex_buffer: *mut *mut IDirect3DVertexBuffer, flags: DWORD, outer_unknown: *mut IUnknown) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D3(iface);
    let mut object: *mut d3d_vertex_buffer = core::ptr::null_mut();
    let mut hr: HRESULT;

    if !outer_unknown.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    if vertex_buffer.is_null() || desc.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    hr = d3d_vertex_buffer_create(core::ptr::addr_of_mut!(object), ddraw, desc);
    if hr == D3D_OK {
        *vertex_buffer = core::ptr::addr_of_mut!((*object).IDirect3DVertexBuffer7_iface) as *mut IDirect3DVertexBuffer;
    } else {
        WARN!("Failed to create vertex buffer, hr %#lx.\n", hr);
    }

    return hr;
}

unsafe extern "system" fn d3d7_EnumZBufferFormats(
    iface: *mut IDirect3D7,
    device_iid: REFCLSID,
    callback: LPD3DENUMPIXELFORMATSCALLBACK,
    context: *mut ::core::ffi::c_void,
) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D7(iface);
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    let mut type_0: wined3d_device_type;
    let mut i: ::core::ffi::c_uint;
    let mut hr: HRESULT;

    /* Order matters. Specifically, BattleZone II (full version) expects the
     * 16-bit depth formats to be listed before the 24 and 32 ones. */
    static formats: [wined3d_format_id; 6] = [
        WINED3DFMT_S1_UINT_D15_UNORM,
        WINED3DFMT_D16_UNORM,
        WINED3DFMT_X8D24_UNORM,
        WINED3DFMT_S4X4_UINT_D24_UNORM,
        WINED3DFMT_D24_UNORM_S8_UINT,
        WINED3DFMT_D32_UNORM,
    ];

    TRACE!(
        "iface %p, device_iid %s, callback %p, context %p.\n",
        iface,
        debugstr_guid(device_iid),
        callback,
        context
    );

    if callback.is_none() {
        return DDERR_INVALIDPARAMS;
    }

    if IsEqualGUID!(device_iid, core::ptr::addr_of!(IID_IDirect3DHALDevice)) != false
        || IsEqualGUID!(device_iid, core::ptr::addr_of!(IID_IDirect3DTnLHalDevice)) != false
        || IsEqualGUID!(device_iid, core::ptr::addr_of!(IID_D3DDEVICE_WineD3D)) != false
    {
        TRACE!("Asked for HAL device.\n");
        type_0 = WINED3D_DEVICE_TYPE_HAL;
    } else if IsEqualGUID!(device_iid, core::ptr::addr_of!(IID_IDirect3DRGBDevice)) != false
        || IsEqualGUID!(device_iid, core::ptr::addr_of!(IID_IDirect3DMMXDevice)) != false
    {
        TRACE!("Asked for SW device.\n");
        type_0 = WINED3D_DEVICE_TYPE_SW;
    } else if IsEqualGUID!(device_iid, core::ptr::addr_of!(IID_IDirect3DRefDevice)) != false {
        TRACE!("Asked for REF device.\n");
        type_0 = WINED3D_DEVICE_TYPE_REF;
    } else if IsEqualGUID!(device_iid, core::ptr::addr_of!(IID_IDirect3DNullDevice)) != false {
        TRACE!("Asked for NULLREF device.\n");
        type_0 = WINED3D_DEVICE_TYPE_NULLREF;
    } else {
        FIXME!("Unexpected device GUID %s.\n", debugstr_guid(device_iid));
        type_0 = WINED3D_DEVICE_TYPE_HAL;
    }

    wined3d_mutex_lock();
    /* We need an adapter format from somewhere to please wined3d and WGL.
     * Use the current display mode. So far all cards offer the same depth
     * stencil format for all modes, but if some do not and applications do
     * not like that we'll have to find some workaround, like iterating over
     * all imaginable formats and collecting all the depth stencil formats we
     * can get. */
    hr = wined3d_output_get_display_mode(
        (*ddraw).wined3d_output,
        core::ptr::addr_of_mut!(mode),
        core::ptr::null_mut(),
    );
    if hr < 0 {
        ERR!("Failed to get display mode, hr %#lx.\n", hr);
        wined3d_mutex_unlock();
        return hr;
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!(formats) as usize {
        if wined3d_check_device_format(
            (*ddraw).wined3d,
            (*ddraw).wined3d_adapter,
            type_0,
            mode.format_id,
            0,
            WINED3D_BIND_DEPTH_STENCIL,
            WINED3D_RTYPE_TEXTURE_2D,
            formats[i as usize],
        ) >= 0
        {
            let mut pformat: DDPIXELFORMAT = core::mem::zeroed();

            memset(
                core::ptr::addr_of_mut!(pformat) as *mut ::core::ffi::c_void,
                0,
                core::mem::size_of::<DDPIXELFORMAT>(),
            );
            pformat.dwSize = core::mem::size_of::<DDPIXELFORMAT>() as DWORD;
            ddrawformat_from_wined3dformat(core::ptr::addr_of_mut!(pformat), formats[i as usize]);

            TRACE!(
                "Enumerating wined3d format %#x.\n",
                formats[i as usize] as ::core::ffi::c_uint
            );
            hr = (callback.unwrap_unchecked())(core::ptr::addr_of_mut!(pformat), context);
            if hr != (DDENUMRET_OK as HRESULT) {
                TRACE!("Format enumeration cancelled by application.\n");
                wined3d_mutex_unlock();
                return D3D_OK;
            }
        }
        i = i.wrapping_add(1);
    }

    /* Historically some windows drivers used dwZBufferBitDepth=24 for WINED3DFMT_X8D24_UNORM,
     * while others used dwZBufferBitDepth=32. In either case the pitch matches a 32 bits per
     * pixel format, so we use dwZBufferBitDepth=32. Some games expect 24. Windows Vista and
     * newer enumerate both versions, so we do the same(bug 22434) */
    if wined3d_check_device_format(
        (*ddraw).wined3d,
        (*ddraw).wined3d_adapter,
        type_0,
        mode.format_id,
        0,
        WINED3D_BIND_DEPTH_STENCIL,
        WINED3D_RTYPE_TEXTURE_2D,
        WINED3DFMT_X8D24_UNORM,
    ) >= 0
    {
        let mut x8d24: DDPIXELFORMAT = _DDPIXELFORMAT {
            dwSize: core::mem::size_of::<DDPIXELFORMAT>() as DWORD,
            dwFlags: DDPF_ZBUFFER,
            dwFourCC: 0,
            __bindgen_anon_1: _DDPIXELFORMAT__bindgen_ty_1 { dwRGBBitCount: 24,
            },
            __bindgen_anon_2: _DDPIXELFORMAT__bindgen_ty_2 { dwRBitMask: 0x00000000,
            },
            __bindgen_anon_3: _DDPIXELFORMAT__bindgen_ty_3 { dwGBitMask: 0x00ffffff,
            },
            __bindgen_anon_4: _DDPIXELFORMAT__bindgen_ty_4 { dwBBitMask: 0x00000000,
            },
            __bindgen_anon_5: _DDPIXELFORMAT__bindgen_ty_5 { dwRGBAlphaBitMask: 0,
            },
        };
        TRACE!("Enumerating WINED3DFMT_X8D24_UNORM, dwZBufferBitDepth=24 version\n");
        (callback.unwrap_unchecked())(core::ptr::addr_of_mut!(x8d24), context);
    }

    TRACE!("End of enumeration.\n");

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d3_EnumZBufferFormats(iface: *mut IDirect3D3, device_iid: REFCLSID, callback: LPD3DENUMPIXELFORMATSCALLBACK, context: *mut core::ffi::c_void) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D3(iface);

    return d3d7_EnumZBufferFormats(core::ptr::addr_of_mut!((*ddraw).IDirect3D7_iface), device_iid, callback, context);
}

unsafe extern "system" fn d3d7_EvictManagedTextures(iface: *mut IDirect3D7) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D7(iface);

    wined3d_mutex_lock();
    if ((*ddraw).flags & DDRAW_D3D_INITIALIZED as DWORD) != 0 {
        wined3d_device_evict_managed_resources((*ddraw).wined3d_device);
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d3_EvictManagedTextures(iface: *mut IDirect3D3) -> HRESULT {
    let ddraw: *mut ddraw = impl_from_IDirect3D3(iface);

    return d3d7_EvictManagedTextures(core::ptr::addr_of_mut!((*ddraw).IDirect3D7_iface));
}
#[no_mangle]
pub unsafe extern "C" fn ddraw_find_decl(This: *mut ddraw, fvf: DWORD) -> *mut wined3d_vertex_declaration {
    let mut pDecl: *mut wined3d_vertex_declaration = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    let mut p: ::core::ffi::c_int = 0;
    let mut low: ::core::ffi::c_int = 0;
    let mut high: ::core::ffi::c_int = 0;
    let mut convertedDecls: *mut FvfToDecl = (*This).decls;

    low = 0 as ::core::ffi::c_int;
    high = (*This).numConvertedDecls.wrapping_sub(1 as UINT) as ::core::ffi::c_int;
    while low <= high {
        p = (low + high) >> (1 as ::core::ffi::c_int);
        if (*convertedDecls.offset(p as isize)).fvf == fvf {
            return (*convertedDecls.offset(p as isize)).decl;
        } else if (*convertedDecls.offset(p as isize)).fvf < fvf {
            low = p + (1 as ::core::ffi::c_int);
        } else {
            high = p - (1 as ::core::ffi::c_int);
        }
    }

    hr = wined3d_vertex_declaration_create_from_fvf(
        (*This).wined3d_device,
        fvf,
        This as *mut ::core::ffi::c_void,
        core::ptr::addr_of!(ddraw_null_wined3d_parent_ops),
        core::ptr::addr_of_mut!(pDecl),
    );
    if hr != S_OK {
        return core::ptr::null_mut();
    }

    if (*This).declArraySize == (*This).numConvertedDecls {
        let mut grow: ::core::ffi::c_uint = if max!((*This).declArraySize / (2 as UINT), 8 as UINT) > 0 {
            0
        } else {
            0
        };
        grow = max!((*This).declArraySize / (2 as UINT), 8 as UINT);

        convertedDecls = realloc(
            convertedDecls as *mut ::core::ffi::c_void,
            (((*This).numConvertedDecls as ::core::ffi::c_uint).wrapping_add(grow) as size_t)
                .wrapping_mul(::core::mem::size_of::<FvfToDecl>() as size_t),
        ) as *mut FvfToDecl;
        if convertedDecls.is_null() {
            wined3d_vertex_declaration_decref(pDecl);
            return core::ptr::null_mut();
        }
        (*This).decls = convertedDecls;
        (*This).declArraySize = ((*This).declArraySize as ::core::ffi::c_uint).wrapping_add(grow) as UINT;
    }

    memmove(
        convertedDecls.offset(low as isize + 1 as isize) as *mut ::core::ffi::c_void,
        convertedDecls.offset(low as isize) as *const ::core::ffi::c_void,
        core::mem::size_of_val(&*convertedDecls.offset(0 as isize))
            .wrapping_mul((*This).numConvertedDecls.wrapping_sub(low as UINT) as usize),
    );
    (*convertedDecls.offset(low as isize)).decl = pDecl;
    (*convertedDecls.offset(low as isize)).fvf = fvf;
    (*This).numConvertedDecls = (*This).numConvertedDecls.wrapping_add(1);

    return pDecl;
}

unsafe extern "C" fn ddraw_from_device_parent(
    device_parent: *mut wined3d_device_parent,
) -> *mut ddraw {
    return CONTAINING_RECORD!(device_parent, ddraw, device_parent);
}

unsafe extern "C" fn device_parent_wined3d_device_created(
    device_parent: *mut wined3d_device_parent,
    device: *mut wined3d_device,
) {
    // TRACE("device_parent %p, device %p.\n", device_parent, device);
}

unsafe extern "C" fn device_parent_mode_changed(device_parent: *mut wined3d_device_parent) {
    let ddraw: *mut ddraw = ddraw_from_device_parent(device_parent);
    let mut output_desc: wined3d_output_desc = core::mem::zeroed();
    let mut r: *mut RECT = core::ptr::null_mut();

    if ((*ddraw).cooperative_level & DDSCL_EXCLUSIVE) == 0 || (*ddraw).swapchain_window.is_null()
    {
        return;
    }

    if (wined3d_output_get_desc((*ddraw).wined3d_output, core::ptr::addr_of_mut!(output_desc)))
        < 0
    {
        ERR!("Failed to get output description.\n");
        return;
    }

    r = core::ptr::addr_of_mut!(output_desc.desktop_rect);

    if SetWindowPos(
        (*ddraw).swapchain_window,
        HWND_TOP,
        (*r).left,
        (*r).top,
        (*r).right - (*r).left,
        (*r).bottom - (*r).top,
        SWP_SHOWWINDOW | SWP_NOACTIVATE,
    ) == 0
    {
        ERR!("Failed to resize window.\n");
    }

    InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*ddraw).device_state) as *mut core::ffi::c_long,
        (DDRAW_DEVICE_STATE_NOT_RESTORED) as i32,
        (DDRAW_DEVICE_STATE_OK) as i32,
    );
}

unsafe extern "C" fn device_parent_activate(device_parent: *mut wined3d_device_parent, activate: BOOL) {
    let mut ddraw: *mut ddraw = ddraw_from_device_parent(device_parent);

    TRACE!("device_parent %p, activate %#x.\n", device_parent, activate);

    if activate == 0 {
        (*ddraw).device_state = DDRAW_DEVICE_STATE_LOST as LONG;
        exclusive_window = core::ptr::null_mut();
    } else {
        let _old: LONG = {
            let dest: *mut LONG = core::ptr::addr_of_mut!((*ddraw).device_state);
            let old: LONG = core::ptr::read(dest);
            if old == DDRAW_DEVICE_STATE_LOST as LONG {
                core::ptr::write(dest, DDRAW_DEVICE_STATE_NOT_RESTORED as LONG);
            }
            old
        };
    }
}
#[no_mangle]
pub unsafe extern "C" fn ddraw_update_lost_surfaces(mut ddraw: *mut ddraw) {
    let mut surface: *mut ddraw_surface = ::core::ptr::null_mut::<ddraw_surface>();

    /* Railroad Tycoon 2 tries to restore surfaces from within a
     * WM_QUERYNEWPALETTE message handler and expects it to succeed. We
     * haven't received the WM_ACTIVATEAPP message by that point, so the
     * device state is still DDRAW_DEVICE_STATE_LOST, even though we are in
     * the foreground. */
    if ((*ddraw).device_state) as u32 == DDRAW_DEVICE_STATE_LOST {
        let mut window: HWND = if !(*ddraw).focuswindow.is_null() {
            (*ddraw).focuswindow
        } else {
            (*ddraw).dest_window
        };

        if !window.is_null() && GetForegroundWindow() == window {
            (*ddraw).device_state = (DDRAW_DEVICE_STATE_NOT_RESTORED) as i32;
        }
    }

    if ((*ddraw).device_state) as u32 != DDRAW_DEVICE_STATE_NOT_RESTORED {
        return;
    }

    surface = ((*ddraw).surface_list.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(ddraw_surface, surface_list_entry)
            as ::core::ffi::c_ulonglong as isize),
    ) as *mut ddraw_surface;
    while &raw mut (*surface).surface_list_entry != &raw mut (*ddraw).surface_list {
        (*surface).set_is_lost((ddraw_surface_can_be_lost(surface) as ::core::ffi::c_uint) as _);
        surface = ((*surface).surface_list_entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(ddraw_surface, surface_list_entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut ddraw_surface;
    }

    (*ddraw).device_state = (DDRAW_DEVICE_STATE_OK) as i32;
}

unsafe extern "C" fn device_parent_texture_sub_resource_created(
    mut device_parent: *mut wined3d_device_parent,
    mut type_0: wined3d_resource_type,
    mut wined3d_texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut parent: *mut *mut ::core::ffi::c_void,
    mut parent_ops: *mut *const wined3d_parent_ops,
) -> HRESULT {
    WINE_TRACE!("device_parent %p, type %#x, wined3d_texture %p, sub_resource_idx %u, parent %p, parent_ops %p.\n", device_parent, type_0 as ::core::ffi::c_uint, wined3d_texture, sub_resource_idx, parent, parent_ops);
    *parent = core::ptr::null_mut();
    *parent_ops = core::ptr::addr_of!(ddraw_null_wined3d_parent_ops);
    return DD_OK;
}
#[no_mangle]
pub unsafe extern "C" fn ddraw_init(ddraw: *mut ddraw, mut flags: DWORD, device_type: wined3d_device_type) -> HRESULT {
    let mut caps: wined3d_caps = core::mem::zeroed();
    let mut hr: HRESULT;

    static feature_levels: [wined3d_feature_level; 3] = [
        WINED3D_FEATURE_LEVEL_7,
        WINED3D_FEATURE_LEVEL_6,
        WINED3D_FEATURE_LEVEL_5,
    ];

    (*ddraw).IDirectDraw7_iface.lpVtbl = core::ptr::addr_of!(ddraw7_vtbl) as *const IDirectDraw7Vtbl;
    (*ddraw).IDirectDraw_iface.lpVtbl = (&raw const ddraw1_vtbl).cast_mut();
    (*ddraw).IDirectDraw2_iface.lpVtbl = core::ptr::addr_of!(ddraw2_vtbl) as *const IDirectDraw2Vtbl;
    (*ddraw).IDirectDraw4_iface.lpVtbl = core::ptr::addr_of!(ddraw4_vtbl) as *const IDirectDraw4Vtbl;
    (*ddraw).IDirect3D_iface.lpVtbl = core::ptr::addr_of!(d3d1_vtbl) as *const IDirect3DVtbl;
    (*ddraw).IDirect3D2_iface.lpVtbl = core::ptr::addr_of!(d3d2_vtbl) as *const IDirect3D2Vtbl;
    (*ddraw).IDirect3D3_iface.lpVtbl = core::ptr::addr_of!(d3d3_vtbl) as *const IDirect3D3Vtbl;
    (*ddraw).IDirect3D7_iface.lpVtbl = core::ptr::addr_of!(d3d7_vtbl) as *const IDirect3D7Vtbl;
    (*ddraw).device_parent.ops =
        core::ptr::addr_of!(ddraw_wined3d_device_parent_ops) as *const wined3d_device_parent_ops;
    (*ddraw).state_parent.ops =
        core::ptr::addr_of!(ddraw_swapchain_state_parent_ops) as *const wined3d_swapchain_state_parent_ops;
    (*ddraw).numIfaces = 1;
    (*ddraw).ref7 = 1;

    flags |= DDRAW_WINED3D_FLAGS;
    (*ddraw).wined3d = wined3d_create(flags);
    if (*ddraw).wined3d.is_null() {
        flags |= WINED3D_NO3D;
        (*ddraw).wined3d = wined3d_create(flags);
        if (*ddraw).wined3d.is_null() {
            WARN!("Failed to create a wined3d object.\n");
            return E_FAIL;
        }
    }

    (*ddraw).wined3d_adapter = wined3d_get_adapter((*ddraw).wined3d, WINED3DADAPTER_DEFAULT);
    if (*ddraw).wined3d_adapter.is_null() {
        WARN!("Failed to get the default wined3d adapter.\n");
        wined3d_decref((*ddraw).wined3d);
        return E_FAIL;
    }

    (*ddraw).wined3d_output = wined3d_adapter_get_output((*ddraw).wined3d_adapter, 0);
    if (*ddraw).wined3d_output.is_null() {
        WARN!("Failed to get the default wined3d output.\n");
        wined3d_decref((*ddraw).wined3d);
        return E_FAIL;
    }

    hr = wined3d_get_device_caps((*ddraw).wined3d_adapter, device_type, core::ptr::addr_of_mut!(caps));
    if hr < 0 {
        ERR!("Failed to get device caps, hr %#lx.\n", hr);
        wined3d_decref((*ddraw).wined3d);
        return E_FAIL;
    }

    if caps.ddraw_caps.caps & WINEDDCAPS_3D == 0 {
        WARN!("Created a wined3d object without 3D support.\n");
        (*ddraw).flags |= DDRAW_NO3D;
    }

    hr = wined3d_device_create(
        (*ddraw).wined3d,
        (*ddraw).wined3d_adapter,
        device_type,
        core::ptr::null_mut(),
        0,
        (DDRAW_STRIDE_ALIGNMENT) as BYTE,
        core::ptr::addr_of!(feature_levels) as *const wined3d_feature_level,
        ARRAY_SIZE!(feature_levels),
        core::ptr::addr_of_mut!((*ddraw).device_parent),
        core::ptr::addr_of_mut!((*ddraw).wined3d_device),
    );
    if hr < 0 {
        WARN!("Failed to create a wined3d device, hr %#lx.\n", hr);
        wined3d_decref((*ddraw).wined3d);
        return hr;
    }
    (*ddraw).immediate_context = wined3d_device_get_immediate_context((*ddraw).wined3d_device);

    list_init(core::ptr::addr_of_mut!((*ddraw).surface_list));
    list_init(core::ptr::addr_of_mut!((*ddraw).d3ddevice_list));
    return DD_OK;
}

// COM vtable tables — generated from ddraw.c by vtbl_gen.py
/// ddraw.c:4698 `static const IDirectDraw7Vtbl ddraw7_vtbl`  (slot order: rust:types_amd64.rs)
pub static ddraw7_vtbl: IDirectDraw7Vtbl = IDirectDraw7Vtbl {
    QueryInterface: Some(ddraw7_QueryInterface),
    AddRef: Some(ddraw7_AddRef),
    Release: Some(ddraw7_Release),
    Compact: Some(ddraw7_Compact),
    CreateClipper: Some(ddraw7_CreateClipper),
    CreatePalette: Some(ddraw7_CreatePalette),
    CreateSurface: Some(ddraw7_CreateSurface),
    DuplicateSurface: Some(ddraw7_DuplicateSurface),
    EnumDisplayModes: Some(ddraw7_EnumDisplayModes),
    EnumSurfaces: Some(ddraw7_EnumSurfaces),
    FlipToGDISurface: Some(ddraw7_FlipToGDISurface),
    GetCaps: Some(ddraw7_GetCaps),
    GetDisplayMode: Some(ddraw7_GetDisplayMode),
    GetFourCCCodes: Some(ddraw7_GetFourCCCodes),
    GetGDISurface: Some(ddraw7_GetGDISurface),
    GetMonitorFrequency: Some(ddraw7_GetMonitorFrequency),
    GetScanLine: Some(ddraw7_GetScanLine),
    GetVerticalBlankStatus: Some(ddraw7_GetVerticalBlankStatus),
    Initialize: Some(ddraw7_Initialize),
    RestoreDisplayMode: Some(ddraw7_RestoreDisplayMode),
    SetCooperativeLevel: Some(ddraw7_SetCooperativeLevel),
    SetDisplayMode: Some(ddraw7_SetDisplayMode),
    WaitForVerticalBlank: Some(ddraw7_WaitForVerticalBlank),
    GetAvailableVidMem: Some(ddraw7_GetAvailableVidMem),
    GetSurfaceFromDC: Some(ddraw7_GetSurfaceFromDC),
    RestoreAllSurfaces: Some(ddraw7_RestoreAllSurfaces),
    TestCooperativeLevel: Some(ddraw7_TestCooperativeLevel),
    GetDeviceIdentifier: Some(ddraw7_GetDeviceIdentifier),
    StartModeTest: Some(ddraw7_StartModeTest),
    EvaluateMode: Some(ddraw7_EvaluateMode),
};

/// ddraw.c:4738 `static const IDirectDraw4Vtbl ddraw4_vtbl`  (slot order: rust:types_amd64.rs)
pub static ddraw4_vtbl: IDirectDraw4Vtbl = IDirectDraw4Vtbl {
    QueryInterface: Some(ddraw4_QueryInterface),
    AddRef: Some(ddraw4_AddRef),
    Release: Some(ddraw4_Release),
    Compact: Some(ddraw4_Compact),
    CreateClipper: Some(ddraw4_CreateClipper),
    CreatePalette: Some(ddraw4_CreatePalette),
    CreateSurface: Some(ddraw4_CreateSurface),
    DuplicateSurface: Some(ddraw4_DuplicateSurface),
    EnumDisplayModes: Some(ddraw4_EnumDisplayModes),
    EnumSurfaces: Some(ddraw4_EnumSurfaces),
    FlipToGDISurface: Some(ddraw4_FlipToGDISurface),
    GetCaps: Some(ddraw4_GetCaps),
    GetDisplayMode: Some(ddraw4_GetDisplayMode),
    GetFourCCCodes: Some(ddraw4_GetFourCCCodes),
    GetGDISurface: Some(ddraw4_GetGDISurface),
    GetMonitorFrequency: Some(ddraw4_GetMonitorFrequency),
    GetScanLine: Some(ddraw4_GetScanLine),
    GetVerticalBlankStatus: Some(ddraw4_GetVerticalBlankStatus),
    Initialize: Some(ddraw4_Initialize),
    RestoreDisplayMode: Some(ddraw4_RestoreDisplayMode),
    SetCooperativeLevel: Some(ddraw4_SetCooperativeLevel),
    SetDisplayMode: Some(ddraw4_SetDisplayMode),
    WaitForVerticalBlank: Some(ddraw4_WaitForVerticalBlank),
    GetAvailableVidMem: Some(ddraw4_GetAvailableVidMem),
    GetSurfaceFromDC: Some(ddraw4_GetSurfaceFromDC),
    RestoreAllSurfaces: Some(ddraw4_RestoreAllSurfaces),
    TestCooperativeLevel: Some(ddraw4_TestCooperativeLevel),
    GetDeviceIdentifier: Some(ddraw4_GetDeviceIdentifier),
};

/// ddraw.c:4775 `static const IDirectDraw2Vtbl ddraw2_vtbl`  (slot order: rust:types_amd64.rs)
pub static ddraw2_vtbl: IDirectDraw2Vtbl = IDirectDraw2Vtbl {
    QueryInterface: Some(ddraw2_QueryInterface),
    AddRef: Some(ddraw2_AddRef),
    Release: Some(ddraw2_Release),
    Compact: Some(ddraw2_Compact),
    CreateClipper: Some(ddraw2_CreateClipper),
    CreatePalette: Some(ddraw2_CreatePalette),
    CreateSurface: Some(ddraw2_CreateSurface),
    DuplicateSurface: Some(ddraw2_DuplicateSurface),
    EnumDisplayModes: Some(ddraw2_EnumDisplayModes),
    EnumSurfaces: Some(ddraw2_EnumSurfaces),
    FlipToGDISurface: Some(ddraw2_FlipToGDISurface),
    GetCaps: Some(ddraw2_GetCaps),
    GetDisplayMode: Some(ddraw2_GetDisplayMode),
    GetFourCCCodes: Some(ddraw2_GetFourCCCodes),
    GetGDISurface: Some(ddraw2_GetGDISurface),
    GetMonitorFrequency: Some(ddraw2_GetMonitorFrequency),
    GetScanLine: Some(ddraw2_GetScanLine),
    GetVerticalBlankStatus: Some(ddraw2_GetVerticalBlankStatus),
    Initialize: Some(ddraw2_Initialize),
    RestoreDisplayMode: Some(ddraw2_RestoreDisplayMode),
    SetCooperativeLevel: Some(ddraw2_SetCooperativeLevel),
    SetDisplayMode: Some(ddraw2_SetDisplayMode),
    WaitForVerticalBlank: Some(ddraw2_WaitForVerticalBlank),
    GetAvailableVidMem: Some(ddraw2_GetAvailableVidMem),
};

/// ddraw.c:4807 `static const IDirectDrawVtbl ddraw1_vtbl`  (slot order: rust:types_amd64.rs)
pub static ddraw1_vtbl: IDirectDrawVtbl = IDirectDrawVtbl {
    QueryInterface: Some(ddraw1_QueryInterface),
    AddRef: Some(ddraw1_AddRef),
    Release: Some(ddraw1_Release),
    Compact: Some(ddraw1_Compact),
    CreateClipper: Some(ddraw1_CreateClipper),
    CreatePalette: Some(ddraw1_CreatePalette),
    CreateSurface: Some(ddraw1_CreateSurface),
    DuplicateSurface: Some(ddraw1_DuplicateSurface),
    EnumDisplayModes: Some(ddraw1_EnumDisplayModes),
    EnumSurfaces: Some(ddraw1_EnumSurfaces),
    FlipToGDISurface: Some(ddraw1_FlipToGDISurface),
    GetCaps: Some(ddraw1_GetCaps),
    GetDisplayMode: Some(ddraw1_GetDisplayMode),
    GetFourCCCodes: Some(ddraw1_GetFourCCCodes),
    GetGDISurface: Some(ddraw1_GetGDISurface),
    GetMonitorFrequency: Some(ddraw1_GetMonitorFrequency),
    GetScanLine: Some(ddraw1_GetScanLine),
    GetVerticalBlankStatus: Some(ddraw1_GetVerticalBlankStatus),
    Initialize: Some(ddraw1_Initialize),
    RestoreDisplayMode: Some(ddraw1_RestoreDisplayMode),
    SetCooperativeLevel: Some(ddraw1_SetCooperativeLevel),
    SetDisplayMode: Some(ddraw1_SetDisplayMode),
    WaitForVerticalBlank: Some(ddraw1_WaitForVerticalBlank),
};

/// ddraw.c:4836 `static const IDirect3D7Vtbl d3d7_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d7_vtbl: IDirect3D7Vtbl = IDirect3D7Vtbl {
    QueryInterface: Some(d3d7_QueryInterface),
    AddRef: Some(d3d7_AddRef),
    Release: Some(d3d7_Release),
    EnumDevices: Some(d3d7_EnumDevices),
    CreateDevice: Some(d3d7_CreateDevice),
    CreateVertexBuffer: Some(d3d7_CreateVertexBuffer),
    EnumZBufferFormats: Some(d3d7_EnumZBufferFormats),
    EvictManagedTextures: Some(d3d7_EvictManagedTextures),
};

/// ddraw.c:4850 `static const IDirect3D3Vtbl d3d3_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d3_vtbl: IDirect3D3Vtbl = IDirect3D3Vtbl {
    QueryInterface: Some(d3d3_QueryInterface),
    AddRef: Some(d3d3_AddRef),
    Release: Some(d3d3_Release),
    EnumDevices: Some(d3d3_EnumDevices),
    CreateLight: Some(d3d3_CreateLight),
    CreateMaterial: Some(d3d3_CreateMaterial),
    CreateViewport: Some(d3d3_CreateViewport),
    FindDevice: Some(d3d3_FindDevice),
    CreateDevice: Some(d3d3_CreateDevice),
    CreateVertexBuffer: Some(d3d3_CreateVertexBuffer),
    EnumZBufferFormats: Some(d3d3_EnumZBufferFormats),
    EvictManagedTextures: Some(d3d3_EvictManagedTextures),
};

/// ddraw.c:4868 `static const IDirect3D2Vtbl d3d2_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d2_vtbl: IDirect3D2Vtbl = IDirect3D2Vtbl {
    QueryInterface: Some(d3d2_QueryInterface),
    AddRef: Some(d3d2_AddRef),
    Release: Some(d3d2_Release),
    EnumDevices: Some(d3d2_EnumDevices),
    CreateLight: Some(d3d2_CreateLight),
    CreateMaterial: Some(d3d2_CreateMaterial),
    CreateViewport: Some(d3d2_CreateViewport),
    FindDevice: Some(d3d2_FindDevice),
    CreateDevice: Some(d3d2_CreateDevice),
};

/// ddraw.c:4883 `static const IDirect3DVtbl d3d1_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d1_vtbl: IDirect3DVtbl = IDirect3DVtbl {
    QueryInterface: Some(d3d1_QueryInterface),
    AddRef: Some(d3d1_AddRef),
    Release: Some(d3d1_Release),
    Initialize: Some(d3d1_Initialize),
    EnumDevices: Some(d3d1_EnumDevices),
    CreateLight: Some(d3d1_CreateLight),
    CreateMaterial: Some(d3d1_CreateMaterial),
    CreateViewport: Some(d3d1_CreateViewport),
    FindDevice: Some(d3d1_FindDevice),
};
