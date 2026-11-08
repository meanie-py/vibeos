use crate::d3dwine::utils::debug_d3ddevicetype;
use crate::d3dwine::utils::debug_d3dformat;
use crate::d3dwine::utils::debug_d3dresourcetype;
use crate::d3dwine::utils::debug_d3dusage;
use crate::d3dwine::utils::debug_uvec4;
use crate::d3dwine::device::device_context_add;
use crate::d3dwine::device::device_context_remove;
use crate::d3dwine::utils::pixelformat_for_depth;
use crate::d3dwine::ffp_gl::state_nop;
use crate::d3dwine::adapter_gl::wined3d_adapter_gl_create;
use crate::d3dwine::utils::wined3d_adapter_no3d_init_format_info;
use crate::d3dwine::adapter_vk::wined3d_adapter_vk_create;
use crate::d3dwine::utils::wined3d_array_reserve;
use crate::d3dwine::buffer::wined3d_buffer_cleanup;
use crate::d3dwine::buffer::wined3d_buffer_no3d_init;
use crate::d3dwine::context::wined3d_context_cleanup;
use crate::d3dwine::context::wined3d_context_no3d_init;
use crate::d3dwine::surface::wined3d_cpu_blitter_create;
use crate::d3dwine::cs::wined3d_cs_destroy_object;
use crate::d3dwine::utils::wined3d_debug_bind_flags;
use crate::d3dwine::utils::wined3d_debug_view_desc;
use crate::d3dwine::device::wined3d_device_cleanup;
use crate::d3dwine::device::wined3d_device_decref;
use crate::d3dwine::device::wined3d_device_incref;
use crate::d3dwine::device::wined3d_device_init;
use crate::d3dwine::utils::wined3d_format_calculate_pitch;
use crate::d3dwine::utils::wined3d_get_format;
use crate::d3dwine::utils::wined3d_get_primary_adapter_luid;
use crate::d3dwine::wined3d_main::wined3d_mutex_lock;
use crate::d3dwine::wined3d_main::wined3d_mutex_unlock;
use crate::d3dwine::view::wined3d_rendertarget_view_cleanup;
use crate::d3dwine::view::wined3d_rendertarget_view_no3d_init;
use crate::d3dwine::swapchain::wined3d_swapchain_cleanup;
use crate::d3dwine::swapchain::wined3d_swapchain_no3d_init;
use crate::d3dwine::texture::wined3d_texture_cleanup;
use crate::d3dwine::texture::wined3d_texture_no3d_init;
use crate::d3dwine::texture::wined3d_texture_sub_resources_destroyed;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_54 {
    pub vendor: wined3d_pci_vendor,
    pub cards: *const wined3d_fallback_card,
}

pub static RUSTCONV_WSTR_E2974A023688: [u16; 35] = [
    0x0077u16, 0x0069, 0x006E, 0x0065, 0x0064, 0x0033, 0x0064, 0x005F, 0x0062, 0x0075,
    0x0064, 0x0067, 0x0065, 0x0074, 0x005F, 0x0063, 0x0068, 0x0061, 0x006E, 0x0067,
    0x0065, 0x005F, 0x006E, 0x006F, 0x0074, 0x0069, 0x0066, 0x0069, 0x0063, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut adapter_budget_change_notifications: list = unsafe {
    list {
        next: &raw const adapter_budget_change_notifications as *mut list,
        prev: &raw const adapter_budget_change_notifications as *mut list,
    }
};

static mut notification_thread_stop_event: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut notification_thread: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut IID_D3DDEVICE_D3DUID: GUID = _GUID {
    Data1: 0xaeb2cdd4 as ::core::ffi::c_uint,
    Data2: 0x6e41 as ::core::ffi::c_ushort,
    Data3: 0x43ea as ::core::ffi::c_ushort,
    Data4: [
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x61 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut minMipLookup: [min_lookup; 3] = [
    min_lookup {
        mip: [
            GL_NEAREST as GLenum,
            GL_NEAREST as GLenum,
            GL_NEAREST as GLenum,
        ],
    },
    min_lookup {
        mip: [
            GL_NEAREST as GLenum,
            GL_NEAREST_MIPMAP_NEAREST as GLenum,
            GL_NEAREST_MIPMAP_LINEAR as GLenum,
        ],
    },
    min_lookup {
        mip: [
            GL_LINEAR as GLenum,
            GL_LINEAR_MIPMAP_NEAREST as GLenum,
            GL_LINEAR_MIPMAP_LINEAR as GLenum,
        ],
    },
];

#[no_mangle]
pub static mut magLookup: [GLenum; 3] = [
    GL_NEAREST as GLenum,
    GL_NEAREST as GLenum,
    GL_LINEAR as GLenum,
];

static mut driver_version_table: [driver_version_information; 26] = [
    driver_version_information {
        driver: DRIVER_AMD_RAGE_128PRO,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"ati2dvaa.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 3279 as WORD,
        build: 0 as WORD,
    },
    driver_version_information {
        driver: DRIVER_AMD_R100,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"ati2dvag.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 6614 as WORD,
    },
    driver_version_information {
        driver: DRIVER_AMD_R300,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"ati2dvag.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 6764 as WORD,
    },
    driver_version_information {
        driver: DRIVER_AMD_R600,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"ati2dvag.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 1280 as WORD,
    },
    driver_version_information {
        driver: DRIVER_AMD_R300,
        driver_model: DRIVER_MODEL_NT6X,
        driver_name: b"atiumdag.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 741 as WORD,
    },
    driver_version_information {
        driver: DRIVER_AMD_R600,
        driver_model: DRIVER_MODEL_NT6X,
        driver_name: b"atiumdag.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 1280 as WORD,
    },
    driver_version_information {
        driver: DRIVER_AMD_RX,
        driver_model: DRIVER_MODEL_NT6X,
        driver_name: b"aticfx32.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 23013 as WORD,
        build: 1023 as WORD,
    },
    driver_version_information {
        driver: DRIVER_INTEL_GMA800,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"ialmrnt5.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 3889 as WORD,
    },
    driver_version_information {
        driver: DRIVER_INTEL_GMA900,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"igxprd32.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 4764 as WORD,
    },
    driver_version_information {
        driver: DRIVER_INTEL_GMA950,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"igxprd32.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 4926 as WORD,
    },
    driver_version_information {
        driver: DRIVER_INTEL_GMA3000,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"igxprd32.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 5218 as WORD,
    },
    driver_version_information {
        driver: DRIVER_INTEL_GMA950,
        driver_model: DRIVER_MODEL_NT6X,
        driver_name: b"igdumd32.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 1504 as WORD,
    },
    driver_version_information {
        driver: DRIVER_INTEL_GMA3000,
        driver_model: DRIVER_MODEL_NT6X,
        driver_name: b"igdumd32.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 1666 as WORD,
    },
    driver_version_information {
        driver: DRIVER_INTEL_HD4000,
        driver_model: DRIVER_MODEL_NT6X,
        driver_name: b"igdumdim32.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 101 as WORD,
        build: 4577 as WORD,
    },
    driver_version_information {
        driver: DRIVER_NVIDIA_TNT,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"nv4_disp.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 7186 as WORD,
    },
    driver_version_information {
        driver: DRIVER_NVIDIA_GEFORCE2MX,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"nv4_disp.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 10 as WORD,
        build: 9371 as WORD,
    },
    driver_version_information {
        driver: DRIVER_NVIDIA_GEFORCEFX,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"nv4_disp.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 11 as WORD,
        build: 7516 as WORD,
    },
    driver_version_information {
        driver: DRIVER_NVIDIA_GEFORCE6,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"nv4_disp.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 13 as WORD,
        build: 783 as WORD,
    },
    driver_version_information {
        driver: DRIVER_NVIDIA_GEFORCE8,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"nv4_disp.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 13 as WORD,
        build: 4052 as WORD,
    },
    driver_version_information {
        driver: DRIVER_NVIDIA_GEFORCE6,
        driver_model: DRIVER_MODEL_NT6X,
        driver_name: b"nvd3dum.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 13 as WORD,
        build: 783 as WORD,
    },
    driver_version_information {
        driver: DRIVER_NVIDIA_GEFORCE8,
        driver_model: DRIVER_MODEL_NT6X,
        driver_name: b"nvd3dum.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 13 as WORD,
        build: 4052 as WORD,
    },
    driver_version_information {
        driver: DRIVER_NVIDIA_FERMI,
        driver_model: DRIVER_MODEL_NT6X,
        driver_name: b"nvd3dum.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 13 as WORD,
        build: 9135 as WORD,
    },
    driver_version_information {
        driver: DRIVER_NVIDIA_KEPLER,
        driver_model: DRIVER_MODEL_NT6X,
        driver_name: b"nvd3dum.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 15 as WORD,
        build: 3118 as WORD,
    },
    driver_version_information {
        driver: DRIVER_REDHAT_VIRGL,
        driver_model: DRIVER_MODEL_GENERIC,
        driver_name: b"virgl.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 0 as WORD,
        build: 0 as WORD,
    },
    driver_version_information {
        driver: DRIVER_VMWARE,
        driver_model: DRIVER_MODEL_NT5X,
        driver_name: b"vm3dum.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 1 as WORD,
        build: 1134 as WORD,
    },
    driver_version_information {
        driver: DRIVER_WINE,
        driver_model: DRIVER_MODEL_GENERIC,
        driver_name: b"wined3d.dll\0" as *const u8 as *const ::core::ffi::c_char,
        subversion: 0 as WORD,
        build: 0 as WORD,
    },
];

static mut gpu_description_table: [wined3d_gpu_description; 318] = [
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_RIVA_128,
        description: b"NVIDIA RIVA 128\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_TNT,
        vidmem: 4 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_RIVA_TNT,
        description: b"NVIDIA RIVA TNT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_TNT,
        vidmem: 16 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_RIVA_TNT2,
        description: b"NVIDIA RIVA TNT2/TNT2 Pro\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_TNT,
        vidmem: 32 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE,
        description: b"NVIDIA GeForce 256\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_TNT,
        vidmem: 32 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE2,
        description: b"NVIDIA GeForce2 GTS/GeForce2 Pro\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_TNT,
        vidmem: 32 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE2_MX,
        description: b"NVIDIA GeForce2 MX/MX 400\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE2MX,
        vidmem: 32 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE3,
        description: b"NVIDIA GeForce3\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE2MX,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE4_MX,
        description: b"NVIDIA GeForce4 MX 460\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE2MX,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE4_TI4200,
        description: b"NVIDIA GeForce4 Ti 4200\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE2MX,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCEFX_5200,
        description: b"NVIDIA GeForce FX 5200\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCEFX,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCEFX_5600,
        description: b"NVIDIA GeForce FX 5600\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCEFX,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCEFX_5800,
        description: b"NVIDIA GeForce FX 5800\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCEFX,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_6200,
        description: b"NVIDIA GeForce 6200\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE6,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_6600GT,
        description: b"NVIDIA GeForce 6600 GT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE6,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_6800,
        description: b"NVIDIA GeForce 6800\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE6,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_7300,
        description: b"NVIDIA GeForce Go 7300\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE6,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_7400,
        description: b"NVIDIA GeForce Go 7400\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE6,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_7600,
        description: b"NVIDIA GeForce 7600 GT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE6,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_7800GT,
        description: b"NVIDIA GeForce 7800 GT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE6,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_8200,
        description: b"NVIDIA GeForce 8200\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_8300GS,
        description: b"NVIDIA GeForce 8300 GS\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_8400GS,
        description: b"NVIDIA GeForce 8400 GS\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_8500GT,
        description: b"NVIDIA GeForce 8500 GT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_8600GT,
        description: b"NVIDIA GeForce 8600 GT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_8600MGT,
        description: b"NVIDIA GeForce 8600M GT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_8800GTS,
        description: b"NVIDIA GeForce 8800 GTS\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 320 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_8800GTX,
        description: b"NVIDIA GeForce 8800 GTX\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 768 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_9200,
        description: b"NVIDIA GeForce 9200\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_9300,
        description: b"NVIDIA GeForce 9300\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_9400M,
        description: b"NVIDIA GeForce 9400M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_9400GT,
        description: b"NVIDIA GeForce 9400 GT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_9500GT,
        description: b"NVIDIA GeForce 9500 GT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_9600GT,
        description: b"NVIDIA GeForce 9600 GT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_9700MGT,
        description: b"NVIDIA GeForce 9700M GT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_9800GT,
        description: b"NVIDIA GeForce 9800 GT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_210,
        description: b"NVIDIA GeForce 210\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT220,
        description: b"NVIDIA GeForce GT 220\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT240,
        description: b"NVIDIA GeForce GT 240\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTS250,
        description: b"NVIDIA GeForce GTS 250\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX260,
        description: b"NVIDIA GeForce GTX 260\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX275,
        description: b"NVIDIA GeForce GTX 275\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 896 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX280,
        description: b"NVIDIA GeForce GTX 280\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_315M,
        description: b"NVIDIA GeForce 315M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_320M,
        description: b"NVIDIA GeForce 320M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT320M,
        description: b"NVIDIA GeForce GT 320M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT325M,
        description: b"NVIDIA GeForce GT 325M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT330,
        description: b"NVIDIA GeForce GT 330\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTS350M,
        description: b"NVIDIA GeForce GTS 350M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_GEFORCE8,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_410M,
        description: b"NVIDIA GeForce 410M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT420,
        description: b"NVIDIA GeForce GT 420\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT425M,
        description: b"NVIDIA GeForce GT 425M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT430,
        description: b"NVIDIA GeForce GT 430\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT440,
        description: b"NVIDIA GeForce GT 440\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTS450,
        description: b"NVIDIA GeForce GTS 450\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX460,
        description: b"NVIDIA GeForce GTX 460\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 768 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX460M,
        description: b"NVIDIA GeForce GTX 460M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX465,
        description: b"NVIDIA GeForce GTX 465\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX470,
        description: b"NVIDIA GeForce GTX 470\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1280 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX480,
        description: b"NVIDIA GeForce GTX 480\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT520,
        description: b"NVIDIA GeForce GT 520\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT525M,
        description: b"NVIDIA GeForce GT 525M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT540M,
        description: b"NVIDIA GeForce GT 540M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX550,
        description: b"NVIDIA GeForce GTX 550 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT555M,
        description: b"NVIDIA GeForce GT 555M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX560TI,
        description: b"NVIDIA GeForce GTX 560 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX560M,
        description: b"NVIDIA GeForce GTX 560M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX560,
        description: b"NVIDIA GeForce GTX 560\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX570,
        description: b"NVIDIA GeForce GTX 570\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1280 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX580,
        description: b"NVIDIA GeForce GTX 580\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT610,
        description: b"NVIDIA GeForce GT 610\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT630,
        description: b"NVIDIA GeForce GT 630\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT630M,
        description: b"NVIDIA GeForce GT 630M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT640,
        description: b"NVIDIA GeForce GT 640\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT640M,
        description: b"NVIDIA GeForce GT 640M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT650M,
        description: b"NVIDIA GeForce GT 650M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX650,
        description: b"NVIDIA GeForce GTX 650\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX650TI,
        description: b"NVIDIA GeForce GTX 650 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX660,
        description: b"NVIDIA GeForce GTX 660\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX660M,
        description: b"NVIDIA GeForce GTX 660M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX660TI,
        description: b"NVIDIA GeForce GTX 660 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX670,
        description: b"NVIDIA GeForce GTX 670\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX670MX,
        description: b"NVIDIA GeForce GTX 670MX\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX675MX_1,
        description: b"NVIDIA GeForce GTX 675MX\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX675MX_2,
        description: b"NVIDIA GeForce GTX 675MX\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX680,
        description: b"NVIDIA GeForce GTX 680\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX690,
        description: b"NVIDIA GeForce GTX 690\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT720,
        description: b"NVIDIA GeForce GT 720\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT730,
        description: b"NVIDIA GeForce GT 730\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT730M,
        description: b"NVIDIA GeForce GT 730M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT740M,
        description: b"NVIDIA GeForce GT 740M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT750M,
        description: b"NVIDIA GeForce GT 750M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT755M,
        description: b"NVIDIA GeForce GT 755M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX750,
        description: b"NVIDIA GeForce GTX 750\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX750TI,
        description: b"NVIDIA GeForce GTX 750 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX760,
        description: b"NVIDIA GeForce GTX 760\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX760TI,
        description: b"NVIDIA GeForce GTX 760 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX765M,
        description: b"NVIDIA GeForce GTX 765M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX770M,
        description: b"NVIDIA GeForce GTX 770M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX770,
        description: b"NVIDIA GeForce GTX 770\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX775M,
        description: b"NVIDIA GeForce GTX 775M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX780,
        description: b"NVIDIA GeForce GTX 780\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX780M,
        description: b"NVIDIA GeForce GTX 780M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX780TI,
        description: b"NVIDIA GeForce GTX 780 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTXTITAN,
        description: b"NVIDIA GeForce GTX TITAN\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 6144 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTXTITANB,
        description: b"NVIDIA GeForce GTX TITAN Black\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 6144 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTXTITANX,
        description: b"NVIDIA GeForce GTX TITAN X\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 12288 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTXTITANZ,
        description: b"NVIDIA GeForce GTX TITAN Z\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 12288 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_820M,
        description: b"NVIDIA GeForce 820M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_FERMI,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_830M,
        description: b"NVIDIA GeForce 830M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_840M,
        description: b"NVIDIA GeForce 840M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_845M,
        description: b"NVIDIA GeForce 845M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX850M,
        description: b"NVIDIA GeForce GTX 850M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX860M,
        description: b"NVIDIA GeForce GTX 860M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX870M,
        description: b"NVIDIA GeForce GTX 870M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX880M,
        description: b"NVIDIA GeForce GTX 880M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_940M,
        description: b"NVIDIA GeForce 940M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX950,
        description: b"NVIDIA GeForce GTX 950\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX950M,
        description: b"NVIDIA GeForce GTX 950M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX960,
        description: b"NVIDIA GeForce GTX 960\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX960M,
        description: b"NVIDIA GeForce GTX 960M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX970,
        description: b"NVIDIA GeForce GTX 970\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX970M,
        description: b"NVIDIA GeForce GTX 970M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX980,
        description: b"NVIDIA GeForce GTX 980\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX980TI,
        description: b"NVIDIA GeForce GTX 980 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 6144 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GT1030,
        description: b"NVIDIA GeForce GT 1030\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1050,
        description: b"NVIDIA GeForce GTX 1050\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1050TI,
        description: b"NVIDIA GeForce GTX 1050 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1060_3GB,
        description: b"NVIDIA GeForce GTX 1060 3GB\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1060,
        description: b"NVIDIA GeForce GTX 1060\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 6144 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1060M,
        description: b"NVIDIA GeForce GTX 1060M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 6144 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1070,
        description: b"NVIDIA GeForce GTX 1070\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1070M,
        description: b"NVIDIA GeForce GTX 1070M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1080,
        description: b"NVIDIA GeForce GTX 1080\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1080M,
        description: b"NVIDIA GeForce GTX 1080M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1080TI,
        description: b"NVIDIA GeForce GTX 1080 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 11264 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_TITANX_PASCAL,
        description: b"NVIDIA TITAN X (Pascal)\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 12288 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_TITANV,
        description: b"NVIDIA TITAN V\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 12288 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1650,
        description: b"NVIDIA GeForce GTX 1650\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1650SUPER,
        description: b"NVIDIA GeForce GTX 1650 SUPER\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1660SUPER,
        description: b"NVIDIA GeForce GTX 1660 SUPER\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 6144 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_GTX1660TI,
        description: b"NVIDIA GeForce GTX 1660 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 6144 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX2060,
        description: b"NVIDIA GeForce RTX 2060\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 6144 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX2070,
        description: b"NVIDIA GeForce RTX 2070\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX2080,
        description: b"NVIDIA GeForce RTX 2080\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX2080TI,
        description: b"NVIDIA GeForce RTX 2080 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 11264 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3050,
        description: b"NVIDIA GeForce RTX 3050\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 6144 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3060,
        description: b"NVIDIA GeForce RTX 3060\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3060_LHR,
        description: b"NVIDIA GeForce RTX 3060 (Low Hash Rate)\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3060TI_GA103,
        description: b"NVIDIA GeForce RTX 3060 Ti (GA103)\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3060TI_GA104,
        description: b"NVIDIA GeForce RTX 3060 Ti (GA104)\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3060TI_GA104_LHR,
        description: b"NVIDIA GeForce RTX 3060 Ti (GA104, Low Hash Rate)\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3070,
        description: b"NVIDIA GeForce RTX 3070\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3070_LHR,
        description: b"NVIDIA GeForce RTX 3070 (Low Hash Rate)\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3070_MOBILE,
        description: b"NVIDIA GeForce RTX 3070 (mobile)\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3070TI,
        description: b"NVIDIA GeForce RTX 3070 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3080_10GB,
        description: b"NVIDIA GeForce RTX 3080 10GB\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 10240 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3080_10GB_LHR,
        description: b"NVIDIA GeForce RTX 3080 10GB (Low Hash Rate)\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 10240 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3080_12GB,
        description: b"NVIDIA GeForce RTX 3080 12GB\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 10240 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3080TI,
        description: b"NVIDIA GeForce RTX 3080 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 12288 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3090,
        description: b"NVIDIA GeForce RTX 3090\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 24576 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX3090TI,
        description: b"NVIDIA GeForce RTX 3090 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 24576 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_TESLA_T4,
        description: b"NVIDIA Tesla T4\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 16384 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_AMPERE_A10,
        description: b"NVIDIA Ampere A10\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 24576 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX4060,
        description: b"NVIDIA GeForce RTX 4060\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX4060TI8G,
        description: b"NVIDIA GeForce RTX 4060 Ti 8GB\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX4060TI16G,
        description: b"NVIDIA GeForce RTX 4060 Ti 16GB\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 16384 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX4070,
        description: b"NVIDIA GeForce RTX 4070\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 12288 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX4070SUPER,
        description: b"NVIDIA GeForce RTX 4070 SUPER\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 12288 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX4070TI,
        description: b"NVIDIA GeForce RTX 4070 Ti\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 12288 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX4070TISUPER,
        description: b"NVIDIA GeForce RTX 4070 Ti SUPER\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 16384 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX4080,
        description: b"NVIDIA GeForce RTX 4080\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 16384 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX4080SUPER,
        description: b"NVIDIA GeForce RTX 4080 SUPER\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 16384 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_NVIDIA,
        device: CARD_NVIDIA_GEFORCE_RTX4090,
        description: b"NVIDIA GeForce RTX 4090\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_NVIDIA_KEPLER,
        vidmem: 24576 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RAGE_128PRO,
        description: b"ATI Rage Fury\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RAGE_128PRO,
        vidmem: 16 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_7200,
        description: b"ATI RADEON 7200 SERIES\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R100,
        vidmem: 32 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_8500,
        description: b"ATI RADEON 8500 SERIES\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R100,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_9500,
        description: b"ATI Radeon 9500\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R300,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_XPRESS_200M,
        description: b"ATI RADEON XPRESS 200M Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R300,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_X700,
        description: b"ATI Radeon X700 SE\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R300,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_X1600,
        description: b"ATI Radeon X1600 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R300,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD2350,
        description: b"ATI Mobility Radeon HD 2350\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD2600,
        description: b"ATI Mobility Radeon HD 2600\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD2900,
        description: b"ATI Radeon HD 2900 XT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD3200,
        description: b"ATI Radeon HD 3200 Graphics\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD3850,
        description: b"ATI Radeon HD 3850 AGP\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD4200M,
        description: b"ATI Mobility Radeon HD 4200\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD4350,
        description: b"ATI Radeon HD 4350\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 256 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD4600,
        description: b"ATI Radeon HD 4600 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD4700,
        description: b"ATI Radeon HD 4700 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD4800,
        description: b"ATI Radeon HD 4800 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD5400,
        description: b"ATI Radeon HD 5400 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD5600,
        description: b"ATI Radeon HD 5600 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD5700,
        description: b"ATI Radeon HD 5700 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD5800,
        description: b"ATI Radeon HD 5800 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD5900,
        description: b"ATI Radeon HD 5900 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD6300,
        description: b"AMD Radeon HD 6300 series Graphics\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD6400,
        description: b"AMD Radeon HD 6400 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD6410D,
        description: b"AMD Radeon HD 6410D\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD6480G,
        description: b"AMD Radeon HD 6480G\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD6490M,
        description: b"AMD Radeon HD 6490M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD6550D,
        description: b"AMD Radeon HD 6550D\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD6600,
        description: b"AMD Radeon HD 6600 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD6600M,
        description: b"AMD Radeon HD 6600M Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD6700,
        description: b"AMD Radeon HD 6700 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD6800,
        description: b"AMD Radeon HD 6800 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD6900,
        description: b"AMD Radeon HD 6900 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD7660D,
        description: b"AMD Radeon HD 7660D\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD7700,
        description: b"AMD Radeon HD 7700 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD7800,
        description: b"AMD Radeon HD 7800 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD7870,
        description: b"AMD Radeon HD 7870 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD7900,
        description: b"AMD Radeon HD 7900 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD8600M,
        description: b"AMD Radeon HD 8600M Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD8670,
        description: b"AMD Radeon HD 8670\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_HD8770,
        description: b"AMD Radeon HD 8770\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_R3,
        description: b"AMD Radeon HD 8400 / R3 Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_R7,
        description: b"AMD Radeon(TM) R7 Graphics\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_R600,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_R9_285,
        description: b"AMD Radeon R9 285\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_R9_290,
        description: b"AMD Radeon R9 290\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_R9_290X,
        description: b"AMD Radeon R9 290X\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_R9_FURY,
        description: b"AMD Radeon (TM) R9 Fury Series\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_R9_M370X,
        description: b"AMD Radeon R9 M370X\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_R9_M380,
        description: b"AMD Radeon R9 M380\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_R9_M395X,
        description: b"AMD Radeon R9 M395X\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_RX_460,
        description: b"Radeon(TM) RX 460 Graphics\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_RX_480,
        description: b"Radeon (TM) RX 480 Graphics\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_RX_VEGA_10,
        description: b"Radeon RX Vega\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_RX_VEGA_12,
        description: b"Radeon Pro Vega 20\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_RAVEN,
        description: b"AMD Radeon(TM) Vega 10 Mobile Graphics\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_RX_VEGA_20,
        description: b"Radeon RX Vega 20\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_RX_NAVI_10,
        description: b"Radeon RX 5700 / 5700 XT\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 8192 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_RX_NAVI_14,
        description: b"Radeon RX 5500M\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_RX_NAVI_21,
        description: b"Radeon RX 6800/6800 XT / 6900 XT\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 16384 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_PRO_V620,
        description: b"Radeon Pro V620\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 32768 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RADEON_PRO_V620_VF,
        description: b"Radeon Pro V620 VF\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 32768 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_VANGOGH,
        description: b"AMD VANGOGH\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_AMD,
        device: CARD_AMD_RAPHAEL,
        description: b"AMD Radeon(TM) Graphics\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_AMD_RX,
        vidmem: 4096 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_REDHAT,
        device: CARD_REDHAT_VIRGL,
        description: b"Red Hat VirtIO GPU\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_REDHAT_VIRGL,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_VMWARE,
        device: CARD_VMWARE_SVGA3D,
        description: b"VMware SVGA 3D (Microsoft Corporation - WDDM)\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_VMWARE,
        vidmem: 1024 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_830M,
        description: b"Intel(R) 82830M Graphics Controller\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA800,
        vidmem: 32 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_855GM,
        description: b"Intel(R) 82852/82855 GM/GME Graphics Controller\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA800,
        vidmem: 32 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_845G,
        description: b"Intel(R) 845G\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA800,
        vidmem: 32 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_865G,
        description: b"Intel(R) 82865G Graphics Controller\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA800,
        vidmem: 32 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_915G,
        description: b"Intel(R) 82915G/GV/910GL Express Chipset Family\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA900,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_E7221G,
        description: b"Intel(R) E7221G\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA900,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_915GM,
        description: b"Mobile Intel(R) 915GM/GMS,910GML Express Chipset Family\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA900,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_945G,
        description: b"Intel(R) 945G\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA950,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_945GM,
        description: b"Mobile Intel(R) 945GM Express Chipset Family\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA950,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_945GME,
        description: b"Intel(R) 945GME\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA950,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_Q35,
        description: b"Intel(R) Q35\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA950,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_G33,
        description: b"Intel(R) G33\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA950,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_Q33,
        description: b"Intel(R) Q33\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA950,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_PNVG,
        description: b"Intel(R) IGD\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA950,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_PNVM,
        description: b"Intel(R) IGD\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA950,
        vidmem: 64 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_965Q,
        description: b"Intel(R) 965Q\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_965G,
        description: b"Intel(R) 965G\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_946GZ,
        description: b"Intel(R) 946GZ\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_965GM,
        description: b"Mobile Intel(R) 965 Express Chipset Family\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_965GME,
        description: b"Intel(R) 965GME\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 128 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_GM45,
        description: b"Mobile Intel(R) GM45 Express Chipset Family\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IGD,
        description: b"Intel(R) Integrated Graphics Device\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_G45,
        description: b"Intel(R) G45/G43\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_Q45,
        description: b"Intel(R) Q45/Q43\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_G41,
        description: b"Intel(R) G41\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_B43,
        description: b"Intel(R) B43\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 512 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_ILKD,
        description: b"Intel(R) HD Graphics\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_ILKM,
        description: b"Intel(R) HD Graphics\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_SNBD,
        description: b"Intel(R) HD Graphics 3000\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_SNBM,
        description: b"Intel(R) HD Graphics 3000\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_SNBS,
        description: b"Intel(R) HD Graphics Family\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_GMA3000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IVBD,
        description: b"Intel(R) HD Graphics 4000\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IVBM,
        description: b"Intel(R) HD Graphics 4000\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IVBS,
        description: b"Intel(R) HD Graphics Family\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HWD,
        description: b"Intel(R) HD Graphics 4600\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HWM,
        description: b"Intel(R) HD Graphics 4600\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD5000_1,
        description: b"Intel(R) HD Graphics 5000\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD5000_2,
        description: b"Intel(R) HD Graphics 5000\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_I5100_1,
        description: b"Intel(R) Iris(TM) Graphics 5100\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_I5100_2,
        description: b"Intel(R) Iris(TM) Graphics 5100\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_I5100_3,
        description: b"Intel(R) Iris(TM) Graphics 5100\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_I5100_4,
        description: b"Intel(R) Iris(TM) Graphics 5100\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IP5200_1,
        description: b"Intel(R) Iris(TM) Pro Graphics 5200\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IP5200_2,
        description: b"Intel(R) Iris(TM) Pro Graphics 5200\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IP5200_3,
        description: b"Intel(R) Iris(TM) Pro Graphics 5200\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IP5200_4,
        description: b"Intel(R) Iris(TM) Pro Graphics 5200\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IP5200_5,
        description: b"Intel(R) Iris(TM) Pro Graphics 5200\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 1536 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IP5200_6,
        description: b"Intel(R) Iris(TM) Pro Graphics 5200\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD5300,
        description: b"Intel(R) HD Graphics 5300\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD5500,
        description: b"Intel(R) HD Graphics 5500\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD5600,
        description: b"Intel(R) HD Graphics 5600\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD6000,
        description: b"Intel(R) HD Graphics 6000\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_I6100,
        description: b"Intel(R) Iris(TM) Graphics 6100\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IP6200,
        description: b"Intel(R) Iris(TM) Pro Graphics 6200\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IPP6300,
        description: b"Intel(R) Iris(TM) Pro Graphics P6300\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD510_1,
        description: b"Intel(R) HD Graphics 510\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD510_2,
        description: b"Intel(R) HD Graphics 510\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD510_3,
        description: b"Intel(R) HD Graphics 510\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD515,
        description: b"Intel(R) HD Graphics 515\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD520_1,
        description: b"Intel(R) HD Graphics 520\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD520_2,
        description: b"Intel(R) HD Graphics 520\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD530_1,
        description: b"Intel(R) HD Graphics 530\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD530_2,
        description: b"Intel(R) HD Graphics 530\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HDP530,
        description: b"Intel(R) HD Graphics P530\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_I540,
        description: b"Intel(R) Iris(TM) Graphics 540\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_I550,
        description: b"Intel(R) Iris(TM) Graphics 550\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_I555,
        description: b"Intel(R) Iris(TM) Graphics 555\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IP555,
        description: b"Intel(R) Iris(TM) Graphics P555\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IP580_1,
        description: b"Intel(R) Iris(TM) Pro Graphics 580\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IP580_2,
        description: b"Intel(R) Iris(TM) Pro Graphics 580\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IPP580_1,
        description: b"Intel(R) Iris(TM) Pro Graphics P580\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_IPP580_2,
        description: b"Intel(R) Iris(TM) Pro Graphics P580\0" as *const u8
            as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_UHD617,
        description: b"Intel(R) UHD Graphics 617\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_UHD620,
        description: b"Intel(R) UHD Graphics 620\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD615,
        description: b"Intel(R) HD Graphics 615\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 2048 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD620,
        description: b"Intel(R) HD Graphics 620\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD630_1,
        description: b"Intel(R) HD Graphics 630\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_HD630_2,
        description: b"Intel(R) HD Graphics 630\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_UHD630_1,
        description: b"Intel(R) UHD Graphics 630\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
    wined3d_gpu_description {
        vendor: HW_VENDOR_INTEL,
        device: CARD_INTEL_UHD630_2,
        description: b"Intel(R) UHD Graphics 630\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_INTEL_HD4000,
        vidmem: 3072 as ::core::ffi::c_uint,
    },
];

static mut misc_state_template_no3d: [wined3d_state_entry_template; 35] = [
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int)
                as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_LINEPATTERN as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_ZFUNC as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_DITHERENABLE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_MULTISAMPLEANTIALIAS as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: 0 as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: 0,
                apply: None,
            },
            extension: 0,
        },
    ];

static mut wined3d_adapter_no3d_ops: wined3d_adapter_ops = unsafe {
    wined3d_adapter_ops {
        adapter_destroy: Some(
            adapter_no3d_destroy as unsafe extern "C" fn(*mut wined3d_adapter) -> (),
        ),
        adapter_create_device: Some(
            adapter_no3d_create_device
                as unsafe extern "C" fn(
                    *mut wined3d,
                    *const wined3d_adapter,
                    wined3d_device_type,
                    HWND,
                    ::core::ffi::c_uint,
                    BYTE,
                    *const wined3d_feature_level,
                    ::core::ffi::c_uint,
                    *mut wined3d_device_parent,
                    *mut *mut wined3d_device,
                ) -> HRESULT,
        ),
        adapter_destroy_device: Some(
            adapter_no3d_destroy_device as unsafe extern "C" fn(*mut wined3d_device) -> (),
        ),
        adapter_acquire_context: Some(
            adapter_no3d_acquire_context
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                ) -> *mut wined3d_context,
        ),
        adapter_release_context: Some(
            adapter_no3d_release_context as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        adapter_get_wined3d_caps: Some(
            adapter_no3d_get_wined3d_caps
                as unsafe extern "C" fn(*const wined3d_adapter, *mut wined3d_caps) -> (),
        ),
        adapter_check_format: Some(
            adapter_no3d_check_format
                as unsafe extern "C" fn(
                    *const wined3d_adapter,
                    *const wined3d_format,
                    *const wined3d_format,
                    *const wined3d_format,
                ) -> BOOL,
        ),
        adapter_init_3d: Some(
            adapter_no3d_init_3d as unsafe extern "C" fn(*mut wined3d_device) -> HRESULT,
        ),
        adapter_uninit_3d: Some(
            adapter_no3d_uninit_3d as unsafe extern "C" fn(*mut wined3d_device) -> (),
        ),
        adapter_map_bo_address: Some(
            adapter_no3d_map_bo_address
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_bo_address,
                    size_t,
                    uint32_t,
                ) -> *mut ::core::ffi::c_void,
        ),
        adapter_unmap_bo_address: Some(
            adapter_no3d_unmap_bo_address
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_bo_address,
                    ::core::ffi::c_uint,
                    *const wined3d_range,
                ) -> (),
        ),
        adapter_copy_bo_address: Some(
            adapter_no3d_copy_bo_address
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_bo_address,
                    *const wined3d_bo_address,
                    ::core::ffi::c_uint,
                    *const wined3d_range,
                    uint32_t,
                ) -> (),
        ),
        adapter_flush_bo_address: Some(
            adapter_no3d_flush_bo_address
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_const_bo_address,
                    size_t,
                ) -> (),
        ),
        adapter_alloc_bo: Some(
            adapter_no3d_alloc_bo
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut wined3d_bo_address,
                ) -> bool,
        ),
        adapter_destroy_bo: Some(
            adapter_no3d_destroy_bo
                as unsafe extern "C" fn(*mut wined3d_context, *mut wined3d_bo) -> (),
        ),
        adapter_create_swapchain: Some(
            adapter_no3d_create_swapchain
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_swapchain_desc,
                    *mut wined3d_swapchain_state_parent,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_swapchain,
                ) -> HRESULT,
        ),
        adapter_destroy_swapchain: Some(
            adapter_no3d_destroy_swapchain as unsafe extern "C" fn(*mut wined3d_swapchain) -> (),
        ),
        adapter_create_buffer: Some(
            adapter_no3d_create_buffer
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_buffer_desc,
                    *const wined3d_sub_resource_data,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_buffer,
                ) -> HRESULT,
        ),
        adapter_destroy_buffer: Some(
            adapter_no3d_destroy_buffer as unsafe extern "C" fn(*mut wined3d_buffer) -> (),
        ),
        adapter_create_texture: Some(
            adapter_no3d_create_texture
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_resource_desc,
                    ::core::ffi::c_uint,
                    ::core::ffi::c_uint,
                    uint32_t,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_texture,
                ) -> HRESULT,
        ),
        adapter_destroy_texture: Some(
            adapter_no3d_destroy_texture as unsafe extern "C" fn(*mut wined3d_texture) -> (),
        ),
        adapter_create_rendertarget_view: Some(
            adapter_no3d_create_rendertarget_view
                as unsafe extern "C" fn(
                    *const wined3d_view_desc,
                    *mut wined3d_resource,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_rendertarget_view,
                ) -> HRESULT,
        ),
        adapter_destroy_rendertarget_view: Some(
            adapter_no3d_destroy_rendertarget_view
                as unsafe extern "C" fn(*mut wined3d_rendertarget_view) -> (),
        ),
        adapter_create_shader_resource_view: Some(
            adapter_no3d_create_shader_resource_view
                as unsafe extern "C" fn(
                    *const wined3d_view_desc,
                    *mut wined3d_resource,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_shader_resource_view,
                ) -> HRESULT,
        ),
        adapter_destroy_shader_resource_view: Some(
            adapter_no3d_destroy_shader_resource_view
                as unsafe extern "C" fn(*mut wined3d_shader_resource_view) -> (),
        ),
        adapter_create_unordered_access_view: Some(
            adapter_no3d_create_unordered_access_view
                as unsafe extern "C" fn(
                    *const wined3d_view_desc,
                    *mut wined3d_resource,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_unordered_access_view,
                ) -> HRESULT,
        ),
        adapter_destroy_unordered_access_view: Some(
            adapter_no3d_destroy_unordered_access_view
                as unsafe extern "C" fn(*mut wined3d_unordered_access_view) -> (),
        ),
        adapter_create_sampler: Some(
            adapter_no3d_create_sampler
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_sampler_desc,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_sampler,
                ) -> HRESULT,
        ),
        adapter_destroy_sampler: Some(
            adapter_no3d_destroy_sampler as unsafe extern "C" fn(*mut wined3d_sampler) -> (),
        ),
        adapter_create_query: Some(
            adapter_no3d_create_query
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    wined3d_query_type,
                    *mut ::core::ffi::c_void,
                    *const wined3d_parent_ops,
                    *mut *mut wined3d_query,
                ) -> HRESULT,
        ),
        adapter_destroy_query: Some(
            adapter_no3d_destroy_query as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
        adapter_flush_context: Some(
            adapter_no3d_flush_context as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        adapter_draw_primitive: Some(
            adapter_no3d_draw_primitive
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_state,
                    *const wined3d_draw_parameters,
                ) -> (),
        ),
        adapter_dispatch_compute: Some(
            adapter_no3d_dispatch_compute
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_state,
                    *const wined3d_dispatch_parameters,
                ) -> (),
        ),
        adapter_clear_uav: Some(
            adapter_no3d_clear_uav
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *mut wined3d_unordered_access_view,
                    *const wined3d_uvec4,
                    bool,
                ) -> (),
        ),
        adapter_generate_mipmap: None,
    }
};

#[no_mangle]
pub static mut wined3d_null_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            wined3d_null_wined3d_object_destroyed,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d) -- debug channel macro, no Rust declaration emitted
// WINE_DECLARE_DEBUG_CHANNEL(winediag) -- debug channel macro, no Rust declaration emitted

pub const DEFAULT_REFRESH_RATE: u32 = 0;

pub type wined3d_driver_model = i32;
pub const DRIVER_MODEL_GENERIC: wined3d_driver_model = 0;
pub const DRIVER_MODEL_WIN9X: wined3d_driver_model = 1;
pub const DRIVER_MODEL_NT40: wined3d_driver_model = 2;
pub const DRIVER_MODEL_NT5X: wined3d_driver_model = 3;
pub const DRIVER_MODEL_NT6X: wined3d_driver_model = 4;

#[repr(C)]
pub struct wined3d_adapter_budget_change_notification {
    pub adapter: *const wined3d_adapter,
    pub event: HANDLE,
    pub cookie: DWORD,
    pub last_local_budget: UINT64,
    pub last_non_local_budget: UINT64,
    pub entry: list,
}

#[repr(C)]
pub struct driver_version_information {
    pub driver: wined3d_display_driver,
    pub driver_model: wined3d_driver_model,
    pub driver_name: *const i8,
    pub subversion: WORD,
    pub build: WORD,
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_output_release_ownership(output: *const wined3d_output) {
    let mut set_owner_desc: D3DKMT_SETVIDPNSOURCEOWNER = core::mem::zeroed();
    set_owner_desc.hDevice = (*output).kmt_device;
    D3DKMTSetVidPnSourceOwner(core::ptr::addr_of_mut!(set_owner_desc));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_output_take_ownership(output: *const wined3d_output, exclusive: BOOL) -> HRESULT {
    let mut set_owner_desc: D3DKMT_SETVIDPNSOURCEOWNER = D3DKMT_SETVIDPNSOURCEOWNER {
        hDevice: 0,
        pType: core::ptr::null_mut(),
        pVidPnSourceId: core::ptr::null(),
        VidPnSourceCount: 0,
    };
    let mut owner_type: D3DKMT_VIDPNSOURCEOWNER_TYPE;
    let mut status: NTSTATUS;

    TRACE!("output %p, exclusive %#x.\n", output, exclusive);

    owner_type = if exclusive != 0 { D3DKMT_VIDPNSOURCEOWNER_EXCLUSIVE } else { D3DKMT_VIDPNSOURCEOWNER_SHARED };
    set_owner_desc.pType = core::ptr::addr_of_mut!(owner_type);
    set_owner_desc.pVidPnSourceId = core::ptr::addr_of!((*output).vidpn_source_id);
    set_owner_desc.VidPnSourceCount = 1;
    set_owner_desc.hDevice = (*output).kmt_device;
    status = D3DKMTSetVidPnSourceOwner(core::ptr::addr_of_mut!(set_owner_desc)) as NTSTATUS;

    match status {
        STATUS_GRAPHICS_VIDPN_SOURCE_IN_USE => {
            return DXGI_ERROR_NOT_CURRENTLY_AVAILABLE;
        }
        STATUS_INVALID_PARAMETER => {
            return E_INVALIDARG;
        }
        STATUS_PROCEDURE_NOT_FOUND => {
            return E_NOINTERFACE;
        }
        STATUS_SUCCESS => {
            return S_OK;
        }
        _ => {
            FIXME!("Unhandled error %#lx.\n", status);
            return E_FAIL;
        }
    }
}

unsafe extern "C" fn wined3d_output_cleanup(output: *const wined3d_output) {
    let mut destroy_device_desc: D3DKMT_DESTROYDEVICE = D3DKMT_DESTROYDEVICE { hDevice: 0 };
    TRACE!("output %p.\n", output);
    destroy_device_desc.hDevice = (*output).kmt_device;
    D3DKMTDestroyDevice(core::ptr::addr_of_mut!(destroy_device_desc));
}

unsafe extern "C" fn wined3d_output_init(
    mut output: *mut wined3d_output,
    mut ordinal: ::core::ffi::c_uint,
    mut adapter: *mut wined3d_adapter,
    mut device_name: *const WCHAR,
) -> HRESULT {
    let mut open_adapter_desc: D3DKMT_OPENADAPTERFROMGDIDISPLAYNAME =
        _D3DKMT_OPENADAPTERFROMGDIDISPLAYNAME {
            DeviceName: [0; 32],
            hAdapter: 0,
            AdapterLuid: _LUID {
                LowPart: 0,
                HighPart: 0,
            },
            VidPnSourceId: 0,
        };
    let mut create_device_desc: D3DKMT_CREATEDEVICE = _D3DKMT_CREATEDEVICE {
        __bindgen_anon_1: _D3DKMT_CREATEDEVICE__bindgen_ty_1 { hAdapter: 0 as D3DKMT_HANDLE,
        },
        Flags: _D3DKMT_CREATEDEVICEFLAGS {
            _bindgen_align: [0; 0],
            _bitfield_1: Default::default(),
        },
        hDevice: 0,
        pCommandBuffer: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        CommandBufferSize: 0,
        pAllocationList: ::core::ptr::null_mut::<D3DDDI_ALLOCATIONLIST>(),
        AllocationListSize: 0,
        pPatchLocationList: ::core::ptr::null_mut::<D3DDDI_PATCHLOCATIONLIST>(),
        PatchLocationListSize: 0,
    };
    let mut close_adapter_desc: D3DKMT_CLOSEADAPTER = _D3DKMT_CLOSEADAPTER { hAdapter: 0 };
    WINE_TRACE!("output %p, device_name %s.\n", output, wine_dbgstr_w(device_name));
    lstrcpyW(
        &raw mut open_adapter_desc.DeviceName as LPWSTR,
        device_name as LPCWSTR,
    );
    if D3DKMTOpenAdapterFromGdiDisplayName(&raw mut open_adapter_desc) != 0 {
        return E_INVALIDARG;
    }
    close_adapter_desc.hAdapter = open_adapter_desc.hAdapter;
    D3DKMTCloseAdapter(&raw mut close_adapter_desc);
    create_device_desc.__bindgen_anon_1.hAdapter = (*adapter).kmt_adapter;
    if D3DKMTCreateDevice(&raw mut create_device_desc) != 0 {
        return E_FAIL;
    }
    (*output).ordinal = ordinal;
    lstrcpyW(
        &raw mut (*output).device_name as LPWSTR,
        device_name as LPCWSTR,
    );
    (*output).adapter = adapter;
    (*output).screen_format = WINED3DFMT_UNKNOWN;
    (*output).kmt_device = create_device_desc.hDevice;
    (*output).vidpn_source_id = open_adapter_desc.VidPnSourceId;
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn adapter_adjust_memory(adapter: *mut wined3d_adapter, amount: INT64) -> UINT64 {
    (*adapter).vram_bytes_used = (*adapter).vram_bytes_used.wrapping_add(amount as UINT64);
    WINE_TRACE!("Adjusted used adapter memory by 0x%s to 0x%s.\n", wine_dbgstr_longlong(amount as UINT64), wine_dbgstr_longlong((*adapter).vram_bytes_used as UINT64));
    return (*adapter).vram_bytes_used;
}
#[no_mangle]
pub unsafe extern "C" fn adapter_adjust_mapped_memory(mut adapter: *mut wined3d_adapter, mut size: ssize_t) -> ssize_t {
    let mut ret: ssize_t = InterlockedExchangeAddSizeT(
        core::ptr::addr_of_mut!((*adapter).mapped_size),
        size as _,
    ) as ssize_t + size;
    TRACE!("Adjusted mapped adapter memory by %Id to %Id.\n", size, ret);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_cleanup(adapter: *mut wined3d_adapter) {
    let mut close_adapter_desc: D3DKMT_CLOSEADAPTER = D3DKMT_CLOSEADAPTER { hAdapter: 0 };
    let mut output_idx: ::core::ffi::c_uint = 0;
    output_idx = 0 as ::core::ffi::c_uint;
    while (output_idx as SIZE_T) < (*adapter).output_count {
        wined3d_output_cleanup(
            (*adapter).outputs.offset(output_idx as isize) as *mut wined3d_output
        );
        output_idx = output_idx.wrapping_add(1);
    }
    free((*adapter).outputs as *mut ::core::ffi::c_void);
    free((*adapter).formats as *mut ::core::ffi::c_void);
    close_adapter_desc.hAdapter = (*adapter).kmt_adapter;
    D3DKMTCloseAdapter(core::ptr::addr_of_mut!(close_adapter_desc));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_incref(mut wined3d: *mut wined3d) -> ULONG {
    let mut refcount: u32 = InterlockedIncrement(core::ptr::addr_of_mut!((*wined3d).ref_)) as u32;
    TRACE!("%p increasing refcount to %u.\n", wined3d, refcount);
    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_decref(mut wined3d: *mut wined3d) -> ULONG {
    let mut refcount: u32 = InterlockedDecrement(
        core::ptr::addr_of_mut!((*wined3d).ref_) as *mut i32,
    ) as u32;
    // WINE_TRACE line (debug-channel call) is not present in the C source above; omitted.
    if refcount == 0 {
        let mut i: u32 = 0;
        wined3d_mutex_lock();
        i = 0;
        while i < (*wined3d).adapter_count {
            let mut adapter: *mut wined3d_adapter =
                *(core::ptr::addr_of_mut!((*wined3d).adapters) as *mut *mut wined3d_adapter).offset(i as isize);
            (((*(*adapter).adapter_ops).adapter_destroy).unwrap_unchecked())(adapter);
            i = i.wrapping_add(1);
        }
        free(wined3d as *mut core::ffi::c_void);
        wined3d_mutex_unlock();
    }
    return refcount as ULONG;
}

unsafe extern "C" fn get_driver_version_info(
    mut driver: wined3d_display_driver,
    mut driver_model: wined3d_driver_model,
) -> *const driver_version_information {
    let mut i: ::core::ffi::c_uint = 0;
    WINE_TRACE!("Looking up version info for driver %#x, driver_model %#x.\n", driver as ::core::ffi::c_uint, driver_model as ::core::ffi::c_uint);
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(driver_version_table) as usize {
        let mut entry: *const driver_version_information =
            (&raw const driver_version_table as *const driver_version_information)
                .offset(i as isize) as *const driver_version_information;
        if (*entry).driver as ::core::ffi::c_uint == driver as ::core::ffi::c_uint
            && (driver_model as ::core::ffi::c_uint
                == DRIVER_MODEL_GENERIC as ::core::ffi::c_uint
                || (*entry).driver_model as ::core::ffi::c_uint
                    == driver_model as ::core::ffi::c_uint)
        {
            WINE_TRACE!("Found driver \"%s\", subversion %u, build %u.\n", (*entry).driver_name, (*entry).subversion as ::core::ffi::c_int, (*entry).build as ::core::ffi::c_int);
            return entry;
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null::<driver_version_information>();
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_get_gpu_description(mut vendor: wined3d_pci_vendor, mut device: wined3d_pci_device) -> *const wined3d_gpu_description {
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(gpu_description_table) as usize {
        if vendor as ::core::ffi::c_uint
            == gpu_description_table[i as usize].vendor as ::core::ffi::c_uint
            && device as ::core::ffi::c_uint
                == gpu_description_table[i as usize].device as ::core::ffi::c_uint
        {
            return (core::ptr::addr_of!(gpu_description_table) as *const wined3d_gpu_description)
                .offset(i as isize) as *const wined3d_gpu_description;
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null::<wined3d_gpu_description>();
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_get_user_override_gpu_description(mut vendor: wined3d_pci_vendor, mut device: wined3d_pci_device) -> *const wined3d_gpu_description {
    let mut gpu_desc: *const wined3d_gpu_description = core::ptr::null();
    #[allow(non_upper_case_globals)]
    static mut once: u32 = 0;

    if wined3d_settings.pci_vendor_id as i32 == (PCI_VENDOR_NONE as i32)
        && wined3d_settings.pci_device_id as i32 == (PCI_DEVICE_NONE as i32)
    {
        return core::ptr::null();
    }

    if wined3d_settings.pci_vendor_id as i32 != (PCI_VENDOR_NONE as i32) {
        vendor = wined3d_settings.pci_vendor_id as wined3d_pci_vendor;
        TRACE!("Overriding vendor PCI ID with 0x%04x.\n", vendor as u32);
    }
    if wined3d_settings.pci_device_id as i32 != (PCI_DEVICE_NONE as i32) {
        device = wined3d_settings.pci_device_id as wined3d_pci_device;
        TRACE!("Overriding device PCI ID with 0x%04x.\n", device as u32);
    }

    gpu_desc = wined3d_get_gpu_description(vendor, device);
    if gpu_desc.is_null() && {
        let fresh = once;
        once = once.wrapping_add(1);
        fresh == 0
    } {
        WINE_ERR_!(__wine_dbch_winediag, "Invalid GPU override %04x:%04x specified, ignoring.\n", vendor as u32, device as u32);
    }

    return gpu_desc;
}

unsafe extern "C" fn wined3d_copy_name(
    mut dst: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut dst_size: ::core::ffi::c_uint,
) {
    let mut len: usize = 0;
    if dst_size != 0 {
        len = min!(
            strlen(src) as usize,
            (dst_size - 1) as usize,
        );
        memcpy(
            dst as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            len,
        );
        memset(
            dst.add(len) as *mut ::core::ffi::c_void,
            0,
            (dst_size as usize) - len,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_driver_info_init(driver_info: *mut wined3d_driver_info, gpu_desc: *const wined3d_gpu_description, feature_level: wined3d_feature_level, vram_bytes: UINT64, sysmem_bytes: UINT64) -> bool {
    let mut version_info: *const driver_version_information = core::ptr::null();
    let mut driver_os_version: WORD;
    let mut driver_feature_level: WORD = 10;
    let mut driver_model: wined3d_driver_model;
    let mut driver: wined3d_display_driver;
    let mut memory_status: MEMORYSTATUSEX = core::mem::zeroed();
    let mut os_version: OSVERSIONINFOW = core::mem::zeroed();

    memset(
        core::ptr::addr_of_mut!(os_version) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<OSVERSIONINFOW>(),
    );
    os_version.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOW>() as DWORD;
    if GetVersionExW(core::ptr::addr_of_mut!(os_version)) == 0 {
        ERR!("Failed to get OS version, reporting 2000/XP.\n");
        driver_os_version = 6;
        driver_model = DRIVER_MODEL_NT5X;
    } else {
        match os_version.dwMajorVersion {
            2 | 3 | 4 => {
                /* If needed we could distinguish between 9x and NT4, but this code won't make
                 * sense for NT4 since it had no way to obtain this info through DirectDraw 3.0.
                 */
                driver_os_version = 4;
                driver_model = DRIVER_MODEL_WIN9X;
            }
            5 => {
                driver_os_version = 6;
                driver_model = DRIVER_MODEL_NT5X;
            }
            6 => {
                if os_version.dwMinorVersion == 0 {
                    driver_os_version = 7;
                    driver_model = DRIVER_MODEL_NT6X;
                } else if os_version.dwMinorVersion == 1 {
                    driver_os_version = 8;
                    driver_model = DRIVER_MODEL_NT6X;
                } else if os_version.dwMinorVersion == 2 {
                    driver_os_version = 9;
                    driver_model = DRIVER_MODEL_NT6X;
                } else {
                    if os_version.dwMinorVersion > 3 {
                        FIXME!("Unhandled OS version %lu.%lu, reporting Windows 8.1.\n",
                            os_version.dwMajorVersion, os_version.dwMinorVersion);
                    }
                    driver_os_version = 10;
                    driver_model = DRIVER_MODEL_NT6X;
                }
            }
            10 => {
                driver_os_version = 31;
                driver_model = DRIVER_MODEL_NT6X;
            }
            _ => {
                FIXME!("Unhandled OS version %lu.%lu, reporting Windows 7.\n",
                    os_version.dwMajorVersion, os_version.dwMinorVersion);
                driver_os_version = 8;
                driver_model = DRIVER_MODEL_NT6X;
            }
        }
    }

    match feature_level as ::core::ffi::c_uint {
        WINED3D_FEATURE_LEVEL_NONE | WINED3D_FEATURE_LEVEL_5 => {
            if driver_model == DRIVER_MODEL_WIN9X {
                driver_feature_level = 5;
            } else {
                driver_feature_level = 10;
            }
        }
        WINED3D_FEATURE_LEVEL_6 => {
            driver_feature_level = 11;
        }
        WINED3D_FEATURE_LEVEL_7 => {
            driver_feature_level = 12;
        }
        WINED3D_FEATURE_LEVEL_8 => {
            driver_feature_level = 13;
        }
        WINED3D_FEATURE_LEVEL_9_1 | WINED3D_FEATURE_LEVEL_9_2 | WINED3D_FEATURE_LEVEL_9_3 => {
            driver_feature_level = 14;
        }
        WINED3D_FEATURE_LEVEL_10 => {
            driver_feature_level = 15;
        }
        WINED3D_FEATURE_LEVEL_10_1 => {
            driver_feature_level = 16;
        }
        WINED3D_FEATURE_LEVEL_11 => {
            driver_feature_level = 17;
        }
        WINED3D_FEATURE_LEVEL_11_1 => {
            /* Advertise support for everything up to FL 12_x. */
            driver_feature_level = 21;
        }
        _ => {}
    }
    if os_version.dwMajorVersion == 6 && os_version.dwMinorVersion == 0 {
        driver_feature_level = min!(driver_feature_level, 18);
    } else if os_version.dwMajorVersion < 6 {
        driver_feature_level = min!(driver_feature_level, 14);
    }

    /* Drivers with the OS version 30+ all have the feature level word set to 0 */
    if driver_os_version >= 30 {
        driver_feature_level = 0;
    }

    (*driver_info).vendor = (*gpu_desc).vendor;
    (*driver_info).device = (*gpu_desc).device;
    wined3d_copy_name(
        core::ptr::addr_of_mut!((*driver_info).description) as *mut ::core::ffi::c_char,
        core::ptr::addr_of!((*gpu_desc).description) as *const ::core::ffi::c_char,
        ARRAY_SIZE!((*driver_info).description) as ::core::ffi::c_uint,
    );
    (*driver_info).vram_bytes = if vram_bytes != 0 {
        vram_bytes
    } else {
        ((*gpu_desc).vidmem as UINT64)
            .wrapping_mul(1024)
            .wrapping_mul(1024)
    };
    driver = (*gpu_desc).driver;

    if wined3d_settings.emulated_textureram != 0 {
        (*driver_info).vram_bytes = wined3d_settings.emulated_textureram;
        TRACE!("Overriding amount of video memory with 0x%s bytes.\n",
            wine_dbgstr_longlong((*driver_info).vram_bytes as ULONGLONG));
    }

    /**
     * Diablo 2 crashes when the amount of video memory is greater than 0x7fffffff.
     * In order to avoid this application bug we limit the amount of video memory
     * to LONG_MAX for older Windows versions.
     */
    if driver_model < DRIVER_MODEL_NT6X && (*driver_info).vram_bytes > (LONG_MAX as UINT64) {
        TRACE!("Limiting amount of video memory to %#lx bytes for OS version older than Vista.\n",
            2147483647 as ::core::ffi::c_long);
        (*driver_info).vram_bytes = (LONG_MAX as _);
    }

    (*driver_info).sysmem_bytes = sysmem_bytes;
    if (*driver_info).sysmem_bytes == 0 {
        (*driver_info).sysmem_bytes =
            (64 * 1024 * 1024) as UINT64;
        memory_status.dwLength = core::mem::size_of::<MEMORYSTATUSEX>() as DWORD;
        if GlobalMemoryStatusEx(core::ptr::addr_of_mut!(memory_status)) != 0 {
            (*driver_info).sysmem_bytes = max!(
                memory_status.ullTotalPhys / 2,
                (*driver_info).sysmem_bytes,
            );
        } else {
            ERR!("Failed to get global memory status.\n");
        }
    }

    /* Try to obtain driver version information for the current Windows version. This fails in
     * some cases:
     * - the gpu is not available on the currently selected OS version:
     *   - Geforce GTX480 on Win98. When running applications in compatibility mode on Windows,
     *     version information for the current Windows version is returned instead of faked info.
     *     We do the same and assume the default Windows version to emulate is WinXP.
     *
     *   - Videocard is a Riva TNT but winver is set to win7 (there are no drivers for this beast)
     *     For now return the XP driver info. Perhaps later on we should return VESA.
     *
     * - the gpu is not in our database (can happen when the user overrides the vendor_id / device_id)
     *   This could be an indication that our database is not up to date, so this should be fixed.
     */
    version_info = get_driver_version_info(driver, driver_model);
    if !version_info.is_null() || {
        version_info = get_driver_version_info(driver, DRIVER_MODEL_GENERIC);
        !version_info.is_null()
    } {
        (*driver_info).name = (*version_info).driver_name;
        (*driver_info).version_high = MAKEDWORD_VERSION!(driver_os_version, driver_feature_level);
        (*driver_info).version_low =
            MAKEDWORD_VERSION!((*version_info).subversion, (*version_info).build);

        return true;
    }

    ERR!("No driver version info found for device %04x:%04x, driver model %#x.\n",
        (*driver_info).vendor as ::core::ffi::c_uint,
        (*driver_info).device as ::core::ffi::c_uint,
        driver_model as ::core::ffi::c_uint);
    return false;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_gpu_from_feature_level(mut vendor: *mut wined3d_pci_vendor, mut feature_level: wined3d_feature_level) -> wined3d_pci_device {
    static mut card_fallback_nvidia: [wined3d_fallback_card; 9] = [
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_5,
            device_id: CARD_NVIDIA_RIVA_128,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_6,
            device_id: CARD_NVIDIA_RIVA_TNT,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_7,
            device_id: CARD_NVIDIA_GEFORCE,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_8,
            device_id: CARD_NVIDIA_GEFORCE3,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_9_2,
            device_id: CARD_NVIDIA_GEFORCEFX_5800,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_9_3,
            device_id: CARD_NVIDIA_GEFORCE_6800,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_10,
            device_id: CARD_NVIDIA_GEFORCE_8800GTX,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_11,
            device_id: CARD_NVIDIA_GEFORCE_GTX470,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_NONE,
            device_id: 0,
        },
    ];
    static mut card_fallback_amd: [wined3d_fallback_card; 8] = [
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_5,
            device_id: CARD_AMD_RAGE_128PRO,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_7,
            device_id: CARD_AMD_RADEON_7200,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_8,
            device_id: CARD_AMD_RADEON_8500,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_9_1,
            device_id: CARD_AMD_RADEON_9500,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_9_3,
            device_id: CARD_AMD_RADEON_X1600,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_10,
            device_id: CARD_AMD_RADEON_HD2900,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_11,
            device_id: CARD_AMD_RADEON_HD5600,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_NONE,
            device_id: 0,
        },
    ];
    static mut card_fallback_intel: [wined3d_fallback_card; 6] = [
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_5,
            device_id: CARD_INTEL_845G,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_8,
            device_id: CARD_INTEL_915G,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_9_3,
            device_id: CARD_INTEL_945G,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_10,
            device_id: CARD_INTEL_G45,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_11,
            device_id: CARD_INTEL_IVBD,
        },
        wined3d_fallback_card {
            feature_level: WINED3D_FEATURE_LEVEL_NONE,
            device_id: 0,
        },
    ];
    static mut fallbacks: [C2RustUnnamed_54; 4] = unsafe {
        [
            C2RustUnnamed_54 {
                vendor: HW_VENDOR_AMD,
                cards: &raw const card_fallback_amd as *const wined3d_fallback_card,
            },
            C2RustUnnamed_54 {
                vendor: HW_VENDOR_NVIDIA,
                cards: &raw const card_fallback_nvidia as *const wined3d_fallback_card,
            },
            C2RustUnnamed_54 {
                vendor: HW_VENDOR_VMWARE,
                cards: &raw const card_fallback_amd as *const wined3d_fallback_card,
            },
            C2RustUnnamed_54 {
                vendor: HW_VENDOR_INTEL,
                cards: &raw const card_fallback_intel as *const wined3d_fallback_card,
            },
        ]
    };
    let mut cards: *const wined3d_fallback_card = ::core::ptr::null::<wined3d_fallback_card>();
    let mut device_id: wined3d_pci_device = 0;
    let mut i: ::core::ffi::c_uint = 0;
    cards = ::core::ptr::null::<wined3d_fallback_card>();
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < (ARRAY_SIZE!(fallbacks) as usize) {
        if *vendor as ::core::ffi::c_uint == fallbacks[i as usize].vendor as ::core::ffi::c_uint {
            cards = fallbacks[i as usize].cards;
        }
        i = i.wrapping_add(1);
    }
    if cards.is_null() {
        *vendor = HW_VENDOR_NVIDIA;
        cards = &raw const card_fallback_nvidia as *const wined3d_fallback_card
            as *const wined3d_fallback_card;
    }
    device_id = (*cards).device_id;
    i = 0 as ::core::ffi::c_uint;
    while (*cards.offset(i as isize)).feature_level as u64 != 0 {
        if feature_level as ::core::ffi::c_uint
            >= (*cards.offset(i as isize)).feature_level as ::core::ffi::c_uint
        {
            device_id = (*cards.offset(i as isize)).device_id;
        }
        i = i.wrapping_add(1);
    }
    return device_id;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_get_adapter(
    mut wined3d: *const wined3d,
    mut idx: ::core::ffi::c_uint,
) -> *mut wined3d_adapter {
    TRACE!("wined3d %p, idx %u.\n", wined3d, idx);
    if idx >= (*wined3d).adapter_count {
        return core::ptr::null_mut();
    }
    return *(core::ptr::addr_of!((*wined3d).adapters) as *const *mut wined3d_adapter).offset(idx as isize);
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_get_adapter_count(wined3d: *const wined3d) -> UINT {
    // TRACE("wined3d %p, reporting %u adapters.\n", wined3d, wined3d->adapter_count);
    (*wined3d).adapter_count
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_get_output(
    adapter: *const wined3d_adapter,
    idx: ::core::ffi::c_uint,
) -> *mut wined3d_output {
    TRACE!("adapter %p, idx %u.\n", adapter, idx);
    if idx as SIZE_T >= (*adapter).output_count {
        return core::ptr::null_mut();
    }
    return (*adapter).outputs.offset(idx as isize) as *mut wined3d_output;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_get_output_count(
    adapter: *const wined3d_adapter,
) -> u32 {
    // TRACE("adapter %p, reporting %Iu outputs.\n", adapter, adapter->output_count);
    ((*adapter).output_count) as u32
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_get_video_memory_info(
    adapter: *const wined3d_adapter,
    node_idx: u32,
    group: wined3d_memory_segment_group,
    info: *mut wined3d_video_memory_info,
) -> HRESULT {
    static mut once: u32 = 0;
    let mut query_memory_info: D3DKMT_QUERYVIDEOMEMORYINFO = core::mem::zeroed();
    let mut adapter_id: wined3d_adapter_identifier = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut hr: HRESULT;

    if group as u32 > WINED3D_MEMORY_SEGMENT_GROUP_NON_LOCAL {
        WARN!("Invalid memory segment group %#x.\n", group as u32);
        return E_INVALIDARG;
    }

    query_memory_info.hProcess = core::ptr::null_mut();
    query_memory_info.hAdapter = (*adapter).kmt_adapter;
    query_memory_info.PhysicalAdapterIndex = node_idx;
    query_memory_info.MemorySegmentGroup = group as u32;
    status = D3DKMTQueryVideoMemoryInfo(core::ptr::addr_of_mut!(query_memory_info));
    if status == STATUS_SUCCESS {
        (*info).budget = query_memory_info.Budget;
        (*info).current_usage = query_memory_info.CurrentUsage;
        (*info).current_reservation = query_memory_info.CurrentReservation;
        (*info).available_reservation = query_memory_info.AvailableForReservation;
        return WINED3D_OK;
    }

    /* D3DKMTQueryVideoMemoryInfo() failed, fallback to fake memory info */
    let fresh = once;
    once = once.wrapping_add(1);
    if fresh == 0 {
        FIXME!("Returning fake video memory info.\n");
    }

    if node_idx != 0 {
        FIXME!("Ignoring node index %u.\n", node_idx);
    }

    adapter_id.driver_size = 0;
    adapter_id.description_size = 0;
    hr = wined3d_adapter_get_identifier(adapter, 0, core::ptr::addr_of_mut!(adapter_id));
    if hr < 0 {
        return hr;
    }

    match group as u32 {
        WINED3D_MEMORY_SEGMENT_GROUP_LOCAL => {
            (*info).budget = adapter_id.video_memory as u64;
            (*info).current_usage = (*adapter).vram_bytes_used;
            (*info).available_reservation = (adapter_id.video_memory / 2) as u64;
            (*info).current_reservation = 0 as u64;
        }
        WINED3D_MEMORY_SEGMENT_GROUP_NON_LOCAL => {
            memset(
                info as *mut core::ffi::c_void,
                0 as core::ffi::c_int,
                core::mem::size_of::<wined3d_video_memory_info>() as size_t,
            );
        }
        _ => {}
    }
    return WINED3D_OK;
}

unsafe extern "system" fn notification_thread_func(stop_event: *mut core::ffi::c_void) -> DWORD {
    let mut notification: *mut wined3d_adapter_budget_change_notification;
    let mut info: wined3d_video_memory_info = wined3d_video_memory_info {
        budget: 0,
        current_usage: 0,
        current_reservation: 0,
        available_reservation: 0,
    };
    let mut hr: HRESULT;

    SetThreadDescription(GetCurrentThread(), RUSTCONV_WSTR_E2974A023688.as_ptr());

    loop {
        wined3d_mutex_lock();
        notification = (adapter_budget_change_notifications.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_adapter_budget_change_notification, entry))
            as *mut wined3d_adapter_budget_change_notification;
        while core::ptr::addr_of_mut!((*notification).entry)
            != core::ptr::addr_of_mut!(adapter_budget_change_notifications)
        {
            hr = wined3d_adapter_get_video_memory_info(
                (*notification).adapter,
                0,
                WINED3D_MEMORY_SEGMENT_GROUP_LOCAL,
                core::ptr::addr_of_mut!(info),
            );
            if hr >= 0 && info.budget != (*notification).last_local_budget {
                (*notification).last_local_budget = info.budget;
                SetEvent((*notification).event);
            } else {
                hr = wined3d_adapter_get_video_memory_info(
                    (*notification).adapter,
                    0,
                    WINED3D_MEMORY_SEGMENT_GROUP_NON_LOCAL,
                    core::ptr::addr_of_mut!(info),
                );
                if hr >= 0
                    && info.budget != (*notification).last_non_local_budget
                {
                    (*notification).last_non_local_budget = info.budget;
                    SetEvent((*notification).event);
                }
            }
            notification = ((*notification).entry.next as *mut u8)
                .sub(core::mem::offset_of!(wined3d_adapter_budget_change_notification, entry))
                as *mut wined3d_adapter_budget_change_notification;
        }
        wined3d_mutex_unlock();

        if WaitForSingleObject(stop_event as HANDLE, 1000)
            == WAIT_OBJECT_0
        {
            break;
        }
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_register_budget_change_notification(
    adapter: *const wined3d_adapter,
    event: HANDLE,
    cookie: *mut DWORD,
) -> HRESULT {
    static mut cookie_counter: DWORD = 0;
    static mut wrapped: BOOL = 0;
    let mut notification: *mut wined3d_adapter_budget_change_notification =
        core::ptr::null_mut::<wined3d_adapter_budget_change_notification>();
    let mut new_notification: *mut wined3d_adapter_budget_change_notification =
        core::ptr::null_mut::<wined3d_adapter_budget_change_notification>();
    let mut found: BOOL = (FALSE as i32);

    new_notification = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<wined3d_adapter_budget_change_notification>() as size_t) as _,
    ) as *mut wined3d_adapter_budget_change_notification;
    if new_notification.is_null() {
        return E_OUTOFMEMORY;
    }

    wined3d_mutex_lock();
    (*new_notification).adapter = adapter;
    (*new_notification).event = event;
    let fresh18 = cookie_counter;
    cookie_counter = cookie_counter.wrapping_add(1);
    (*new_notification).cookie = fresh18;
    if cookie_counter < (*new_notification).cookie {
        wrapped = (TRUE as i32);
    }
    if wrapped != 0 {
        loop {
            notification = ((*core::ptr::addr_of_mut!(adapter_budget_change_notifications)).next
                as *mut core::ffi::c_char)
                .offset(
                    -(core::mem::offset_of!(
                        wined3d_adapter_budget_change_notification,
                        entry
                    ) as isize),
                ) as *mut wined3d_adapter_budget_change_notification;
            while core::ptr::addr_of_mut!((*notification).entry)
                != core::ptr::addr_of_mut!(adapter_budget_change_notifications)
            {
                if (*notification).cookie == (*new_notification).cookie {
                    found = (TRUE as i32);
                    break;
                } else {
                    notification = (((*notification).entry).next as *mut core::ffi::c_char)
                        .offset(
                            -(core::mem::offset_of!(
                                wined3d_adapter_budget_change_notification,
                                entry
                            ) as isize),
                        ) as *mut wined3d_adapter_budget_change_notification;
                }
            }

            if found == 0 {
                break;
            }
            let fresh19 = cookie_counter;
            cookie_counter = cookie_counter.wrapping_add(1);
            (*new_notification).cookie = fresh19;
        }
    }

    *cookie = (*new_notification).cookie;
    list_add_head(
        core::ptr::addr_of_mut!(adapter_budget_change_notifications),
        core::ptr::addr_of_mut!((*new_notification).entry),
    );

    if notification_thread.is_null() {
        notification_thread_stop_event = CreateEventW(
            core::ptr::null_mut(),
            (FALSE) as BOOL,
            (FALSE) as BOOL,
            core::ptr::null_mut(),
        );
        notification_thread = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(notification_thread_func),
            notification_thread_stop_event as PVOID,
            0,
            core::ptr::null_mut(),
        );
    }
    wined3d_mutex_unlock();
    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_unregister_budget_change_notification(cookie: DWORD) -> HRESULT {
    let mut notification: *mut wined3d_adapter_budget_change_notification = core::ptr::null_mut();
    let mut thread: HANDLE = core::ptr::null_mut();
    let mut thread_stop_event: HANDLE = core::ptr::null_mut();

    wined3d_mutex_lock();
    notification = (adapter_budget_change_notifications.next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_adapter_budget_change_notification, entry))
        as *mut wined3d_adapter_budget_change_notification;
    while core::ptr::addr_of_mut!((*notification).entry)
        != core::ptr::addr_of_mut!(adapter_budget_change_notifications)
    {
        if (*notification).cookie == cookie {
            list_remove(core::ptr::addr_of_mut!((*notification).entry));
            free(notification as *mut core::ffi::c_void);
            break;
        } else {
            notification = ((*notification).entry.next as *mut u8)
                .sub(core::mem::offset_of!(wined3d_adapter_budget_change_notification, entry))
                as *mut wined3d_adapter_budget_change_notification;
        }
    }

    if list_empty(core::ptr::addr_of_mut!(adapter_budget_change_notifications)) == 0 {
        wined3d_mutex_unlock();
        return WINED3D_OK;
    }

    thread = notification_thread;
    thread_stop_event = notification_thread_stop_event;
    notification_thread = core::ptr::null_mut();
    notification_thread_stop_event = core::ptr::null_mut();
    wined3d_mutex_unlock();
    SetEvent(thread_stop_event);
    WaitForSingleObject(thread, INFINITE);
    CloseHandle(thread);
    CloseHandle(thread_stop_event);
    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_register_software_device(
    wined3d: *mut wined3d,
    init_function: *mut core::ffi::c_void,
) -> HRESULT {
    // FIXME("wined3d %p, init_function %p stub!\n", wined3d, init_function);
    WINED3D_OK
}

unsafe extern "system" fn enum_monitor_proc(
    monitor: HMONITOR,
    hdc: HDC,
    rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let desc: *mut wined3d_output_desc = lparam as *mut wined3d_output_desc;
    let mut monitor_info: MONITORINFOEXW = core::mem::zeroed::<MONITORINFOEXW>();

    monitor_info.cbSize = ::core::mem::size_of::<MONITORINFOEXW>() as DWORD;
    if GetMonitorInfoW(monitor, &raw mut monitor_info as LPMONITORINFO) != 0
        && lstrcmpiW(
            &raw mut (*desc).device_name as *mut WCHAR as LPCWSTR,
            &raw mut monitor_info.szDevice as *mut WCHAR as LPCWSTR,
        ) == 0
    {
        (*desc).monitor = monitor;
        (*desc).desktop_rect = monitor_info.rcMonitor;
        (*desc).attached_to_desktop = (TRUE as i32);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_output_get_desc(
    output: *const wined3d_output,
    desc: *mut wined3d_output_desc,
) -> HRESULT {
    TRACE!("output %p, desc %p.\n", output, desc);
    memset(desc as *mut core::ffi::c_void, 0, core::mem::size_of::<wined3d_output_desc>() as size_t);
    (*desc).ordinal = (*output).ordinal;
    lstrcpyW(
        core::ptr::addr_of_mut!((*desc).device_name) as LPWSTR,
        core::ptr::addr_of!((*output).device_name) as LPCWSTR,
    );
    EnumDisplayMonitors(
        core::ptr::null_mut(),
        core::ptr::null(),
        Some(enum_monitor_proc),
        desc as LPARAM,
    );
    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_output_update_modes(output: *mut wined3d_output, cached: bool) {
    let mut wined3d_mode: *mut wined3d_display_mode = ::core::ptr::null_mut::<wined3d_display_mode>();
    let mut mode: DEVMODEW = core::mem::zeroed();
    mode.dmSize = ::core::mem::size_of::<DEVMODEW>() as WORD;
    let mut i: ::core::ffi::c_uint = 0;

    if (*output).modes_valid && cached {
        return;
    }

    (*output).mode_count = 0 as SIZE_T;

    i = 0 as ::core::ffi::c_uint;
    while EnumDisplaySettingsExW(
        core::ptr::addr_of_mut!((*output).device_name) as *mut WCHAR as LPCWSTR,
        i as DWORD,
        core::ptr::addr_of_mut!(mode),
        0 as DWORD,
    ) != 0
    {
        if wined3d_array_reserve(
            core::ptr::addr_of_mut!((*output).modes) as *mut *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!((*output).modes_size),
            (*output).mode_count.wrapping_add(1 as SIZE_T),
            ::core::mem::size_of::<wined3d_display_mode>() as SIZE_T,
        ) == 0
        {
            return;
        }

        let fresh15 = (*output).mode_count;
        (*output).mode_count = (*output).mode_count.wrapping_add(1);
        wined3d_mode = (*output).modes.offset(fresh15 as isize) as *mut wined3d_display_mode;
        (*wined3d_mode).width = mode.dmPelsWidth as UINT;
        (*wined3d_mode).height = mode.dmPelsHeight as UINT;
        (*wined3d_mode).format_id = pixelformat_for_depth(mode.dmBitsPerPel);

        if mode.dmFields & DM_DISPLAYFREQUENCY as DWORD != 0 {
            (*wined3d_mode).refresh_rate = mode.dmDisplayFrequency as UINT;
        } else {
            (*wined3d_mode).refresh_rate = DEFAULT_REFRESH_RATE as UINT;
        }

        if mode.dmFields & DM_DISPLAYFLAGS as DWORD != 0 {
            if mode.__bindgen_anon_2.dmDisplayFlags & DM_INTERLACED as DWORD != 0 {
                (*wined3d_mode).scanline_ordering = WINED3D_SCANLINE_ORDERING_INTERLACED;
            } else {
                (*wined3d_mode).scanline_ordering = WINED3D_SCANLINE_ORDERING_PROGRESSIVE;
            }
        } else {
            (*wined3d_mode).scanline_ordering = WINED3D_SCANLINE_ORDERING_UNKNOWN;
        }

        i = i.wrapping_add(1);
    }

    (*output).modes_valid = true;
}

unsafe extern "C" fn mode_matches_filter(
    adapter: *const wined3d_adapter,
    mode: *const wined3d_display_mode,
    format: *const wined3d_format,
    scanline_ordering: wined3d_scanline_ordering,
) -> bool {
    if scanline_ordering as core::ffi::c_uint != WINED3D_SCANLINE_ORDERING_UNKNOWN as core::ffi::c_uint
        && (*mode).scanline_ordering as core::ffi::c_uint != WINED3D_SCANLINE_ORDERING_UNKNOWN as core::ffi::c_uint
        && scanline_ordering as core::ffi::c_uint != (*mode).scanline_ordering as core::ffi::c_uint
    {
        return false;
    }

    if (*format).id as core::ffi::c_uint == WINED3DFMT_UNKNOWN as core::ffi::c_uint {
        /* This is for d3d8, do not enumerate P8 here. */
        if (*mode).format_id as core::ffi::c_uint != WINED3DFMT_B5G6R5_UNORM as core::ffi::c_uint
            && (*mode).format_id as core::ffi::c_uint != WINED3DFMT_B8G8R8X8_UNORM as core::ffi::c_uint
        {
            return false;
        }
    } else {
        let mode_format: *const wined3d_format = wined3d_get_format(
            adapter,
            (*mode).format_id,
            WINED3D_BIND_RENDER_TARGET as core::ffi::c_uint,
        );

        if (*format).byte_count != (*mode_format).byte_count {
            return false;
        }
    }

    return true;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_output_get_mode_count(
    mut output: *mut wined3d_output,
    mut format_id: wined3d_format_id,
    mut scanline_ordering: wined3d_scanline_ordering,
    mut cached: bool,
) -> ::core::ffi::c_uint {
    let mut adapter: *const wined3d_adapter = ::core::ptr::null::<wined3d_adapter>();
    let mut format: *const wined3d_format = ::core::ptr::null::<wined3d_format>();
    let mut count: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut i: SIZE_T = 0;
    WINE_TRACE!("output %p, format %s, scanline_ordering %#x.\n", output, debug_d3dformat(format_id), scanline_ordering as ::core::ffi::c_uint);
    adapter = (*output).adapter;
    format = wined3d_get_format(
        adapter,
        format_id,
        WINED3D_BIND_RENDER_TARGET as ::core::ffi::c_uint,
    );
    wined3d_output_update_modes(output, cached);
    i = 0 as SIZE_T;
    while i < (*output).mode_count {
        if mode_matches_filter(
            adapter,
            (*output).modes.offset(i as isize) as *mut wined3d_display_mode,
            format,
            scanline_ordering,
        ) {
            count = count.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    WINE_TRACE!("Returning %u matching modes (out of %Iu total).\n", count, (*output).mode_count);
    return count;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_output_get_mode(output: *mut wined3d_output, format_id: wined3d_format_id, scanline_ordering: wined3d_scanline_ordering, mode_idx: ::core::ffi::c_uint, mode: *mut wined3d_display_mode, cached: bool) -> HRESULT {
    let mut adapter: *const wined3d_adapter = core::ptr::null::<wined3d_adapter>();
    let mut format: *const wined3d_format = core::ptr::null::<wined3d_format>();
    let mut i: SIZE_T = 0;
    let mut match_idx: SIZE_T = 0;

    if mode.is_null() {
        return WINED3DERR_INVALIDCALL;
    }

    adapter = (*output).adapter;
    format = wined3d_get_format(adapter, format_id, WINED3D_BIND_RENDER_TARGET);

    wined3d_output_update_modes(output, cached);

    while i < (*output).mode_count {
        let wined3d_mode: *const wined3d_display_mode = (*output).modes.offset(i as isize);

        if mode_matches_filter(adapter, wined3d_mode, format, scanline_ordering) && {
            let fresh25 = match_idx;
            match_idx = match_idx.wrapping_add(1);
            fresh25 == mode_idx as SIZE_T
        } {
            *mode = *wined3d_mode;
            if format_id as ::core::ffi::c_uint != WINED3DFMT_UNKNOWN as ::core::ffi::c_uint {
                (*mode).format_id = format_id;
            }
            return WINED3D_OK;
        }
        i = i.wrapping_add(1);
    }

    WARN!("Invalid mode_idx %u.\n", mode_idx);
    return WINED3DERR_INVALIDCALL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_output_find_closest_matching_mode(
    output: *mut wined3d_output,
    mode: *mut wined3d_display_mode,
) -> HRESULT {
    let mut i: u32 = 0;
    let mut j: u32 = 0;
    let mut mode_count: u32 = 0;
    let mut matching_mode_count: u32 = 0;
    let mut closest: u32 = 0;
    let mut matching_modes: *mut *mut wined3d_display_mode = core::ptr::null_mut();
    let mut modes: *mut wined3d_display_mode = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!("output %p, mode %p.\n", output, mode);

    mode_count = wined3d_output_get_mode_count(
        output,
        (*mode).format_id,
        WINED3D_SCANLINE_ORDERING_UNKNOWN,
        false,
    );
    if mode_count == 0 {
        WARN!("Output has 0 matching modes.\n");
        return E_FAIL;
    }

    modes = calloc(
        (mode_count as usize) as _,
        (core::mem::size_of::<wined3d_display_mode>()) as _,
    ) as *mut wined3d_display_mode;
    if modes.is_null() {
        return E_OUTOFMEMORY;
    }
    matching_modes = calloc(
        (mode_count as usize) as _,
        (core::mem::size_of::<*mut wined3d_display_mode>()) as _,
    ) as *mut *mut wined3d_display_mode;
    if matching_modes.is_null() {
        free(modes as *mut core::ffi::c_void);
        return E_OUTOFMEMORY;
    }

    i = 0;
    while i < mode_count {
        hr = wined3d_output_get_mode(
            output,
            (*mode).format_id,
            WINED3D_SCANLINE_ORDERING_UNKNOWN,
            i,
            modes.offset(i as isize),
            true,
        );
        if hr < 0 {
            free(matching_modes as *mut core::ffi::c_void);
            free(modes as *mut core::ffi::c_void);
            return hr;
        }
        *matching_modes.offset(i as isize) = modes.offset(i as isize);
        i = i.wrapping_add(1);
    }

    matching_mode_count = mode_count;

    if (*mode).scanline_ordering as u32 != WINED3D_SCANLINE_ORDERING_UNKNOWN as u32 {
        i = 0;
        j = 0;
        while i < matching_mode_count {
            if (**matching_modes.offset(i as isize)).scanline_ordering as u32
                == (*mode).scanline_ordering as u32
            {
                let fresh21 = j;
                j = j.wrapping_add(1);
                *matching_modes.offset(fresh21 as isize) = *matching_modes.offset(i as isize);
            }
            i = i.wrapping_add(1);
        }
        if j > 0 {
            matching_mode_count = j;
        }
    }

    if (*mode).refresh_rate != 0 {
        i = 0;
        j = 0;
        while i < matching_mode_count {
            if (**matching_modes.offset(i as isize)).refresh_rate == (*mode).refresh_rate {
                let fresh23 = j;
                j = j.wrapping_add(1);
                *matching_modes.offset(fresh23 as isize) = *matching_modes.offset(i as isize);
            }
            i = i.wrapping_add(1);
        }
        if j > 0 {
            matching_mode_count = j;
        }
    }

    if (*mode).width == 0 || (*mode).height == 0 {
        let mut current_mode: wined3d_display_mode = core::mem::zeroed();
        hr = wined3d_output_get_display_mode(
            output,
            core::ptr::addr_of_mut!(current_mode),
            core::ptr::null_mut(),
        );
        if hr < 0 {
            free(matching_modes as *mut core::ffi::c_void);
            free(modes as *mut core::ffi::c_void);
            return hr;
        }
        (*mode).width = current_mode.width;
        (*mode).height = current_mode.height;
    }

    closest = !(0 as u32);
    i = 0;
    j = 0;
    while i < matching_mode_count {
        let d: u32 = (abs(
            (*mode)
                .width
                .wrapping_sub((**matching_modes.offset(i as isize)).width) as i32,
        )
        .wrapping_add(abs(
            (*mode)
                .height
                .wrapping_sub((**matching_modes.offset(i as isize)).height) as i32,
        ))) as u32;

        if closest > d {
            closest = d;
            j = i;
        }
        i = i.wrapping_add(1);
    }

    *mode = **matching_modes.offset(j as isize);

    free(matching_modes as *mut core::ffi::c_void);
    free(modes as *mut core::ffi::c_void);

    TRACE!(
        "Returning %ux%u@%u %s %#x.\n",
        (*mode).width,
        (*mode).height,
        (*mode).refresh_rate,
        debug_d3dformat((*mode).format_id),
        (*mode).scanline_ordering as u32,
    );

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_output_get_adapter(
    output: *const wined3d_output,
) -> *mut wined3d_adapter {
    // TRACE("output %p.\n", output);
    (*output).adapter
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_output_get_display_mode(
    output: *const wined3d_output,
    mode: *mut wined3d_display_mode,
    rotation: *mut wined3d_display_rotation,
) -> HRESULT {
    let mut m: DEVMODEW = core::mem::zeroed();

    if mode.is_null() {
        return WINED3DERR_INVALIDCALL;
    }

    memset(core::ptr::addr_of_mut!(m) as *mut core::ffi::c_void, 0, core::mem::size_of::<DEVMODEW>());
    m.dmSize = core::mem::size_of::<DEVMODEW>() as WORD;

    EnumDisplaySettingsExW(
        core::ptr::addr_of!((*output).device_name) as *const u16,
        ENUM_CURRENT_SETTINGS,
        core::ptr::addr_of_mut!(m),
        0,
    );
    (*mode).width = m.dmPelsWidth as UINT;
    (*mode).height = m.dmPelsHeight as UINT;
    (*mode).refresh_rate = DEFAULT_REFRESH_RATE as UINT;
    if (m.dmFields & DM_DISPLAYFREQUENCY) != 0 {
        (*mode).refresh_rate = m.dmDisplayFrequency as UINT;
    }
    (*mode).format_id = pixelformat_for_depth(m.dmBitsPerPel);

    /* Lie about the format. X11 can't change the color depth, and some apps
     * are pretty angry if they SetDisplayMode from 24 to 16 bpp and find out
     * that GetDisplayMode still returns 24 bpp. This should probably be
     * handled in winex11 instead. */
    if (*output).screen_format as ::core::ffi::c_uint != 0
        && (*output).screen_format as ::core::ffi::c_uint
            != (*mode).format_id as ::core::ffi::c_uint
    {
        WARN!("Overriding format %s with stored format %s.\n",
                debug_d3dformat((*mode).format_id),
                debug_d3dformat((*output).screen_format));
        (*mode).format_id = (*output).screen_format;
    }

    if (m.dmFields & DM_DISPLAYFLAGS) == 0 {
        (*mode).scanline_ordering = WINED3D_SCANLINE_ORDERING_UNKNOWN;
    } else if (m.__bindgen_anon_2.dmDisplayFlags & DM_INTERLACED) != 0 {
        (*mode).scanline_ordering = WINED3D_SCANLINE_ORDERING_INTERLACED;
    } else {
        (*mode).scanline_ordering = WINED3D_SCANLINE_ORDERING_PROGRESSIVE;
    }

    if !rotation.is_null() {
        match m.__bindgen_anon_1.__bindgen_anon_2.dmDisplayOrientation {
            DMDO_DEFAULT => {
                *rotation = WINED3D_DISPLAY_ROTATION_0;
            }
            DMDO_90 => {
                *rotation = WINED3D_DISPLAY_ROTATION_90;
            }
            DMDO_180 => {
                *rotation = WINED3D_DISPLAY_ROTATION_180;
            }
            DMDO_270 => {
                *rotation = WINED3D_DISPLAY_ROTATION_270;
            }
            _ => {
                FIXME!("Unhandled display rotation %#lx.\n", m.__bindgen_anon_1.__bindgen_anon_2.dmDisplayOrientation);
                *rotation = WINED3D_DISPLAY_ROTATION_UNSPECIFIED;
            }
        }
    }

    return WINED3D_OK;
}

unsafe extern "C" fn equal_display_mode(
    mut mode1: *const DEVMODEW,
    mut mode2: *const DEVMODEW,
) -> BOOL {
    if (*mode1).dmFields & (*mode2).dmFields & DM_PELSWIDTH as DWORD != 0
        && (*mode1).dmPelsWidth != (*mode2).dmPelsWidth
    {
        return (FALSE) as BOOL;
    }
    if (*mode1).dmFields & (*mode2).dmFields & DM_PELSHEIGHT as DWORD != 0
        && (*mode1).dmPelsHeight != (*mode2).dmPelsHeight
    {
        return (FALSE) as BOOL;
    }
    if (*mode1).dmFields & (*mode2).dmFields & DM_BITSPERPEL as DWORD != 0
        && (*mode1).dmBitsPerPel != (*mode2).dmBitsPerPel
    {
        return (FALSE) as BOOL;
    }
    if (*mode1).dmFields & (*mode2).dmFields & DM_DISPLAYFLAGS as DWORD != 0
        && (*mode1).__bindgen_anon_2.dmDisplayFlags != (*mode2).__bindgen_anon_2.dmDisplayFlags
    {
        return (FALSE) as BOOL;
    }
    if (*mode1).dmFields & (*mode2).dmFields & DM_DISPLAYFREQUENCY as DWORD != 0
        && (*mode1).dmDisplayFrequency != (*mode2).dmDisplayFrequency
    {
        return (FALSE) as BOOL;
    }
    if (*mode1).dmFields & (*mode2).dmFields & DM_DISPLAYORIENTATION as DWORD != 0
        && (*mode1)


            .__bindgen_anon_1.__bindgen_anon_2.dmDisplayOrientation
            != (*mode2)


                .__bindgen_anon_1.__bindgen_anon_2.dmDisplayOrientation
    {
        return (FALSE) as BOOL;
    }
    if (*mode1).dmFields & (*mode2).dmFields & DM_POSITION as DWORD != 0
        && ((*mode1).__bindgen_anon_1.__bindgen_anon_2.dmPosition.x
            != (*mode2).__bindgen_anon_1.__bindgen_anon_2.dmPosition.x
            || (*mode1).__bindgen_anon_1.__bindgen_anon_2.dmPosition.y
                != (*mode2).__bindgen_anon_1.__bindgen_anon_2.dmPosition.y)
    {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_restore_display_modes(wined3d: *mut wined3d) -> HRESULT {
    let mut adapter_idx: ::core::ffi::c_uint = 0;
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut current_mode: DEVMODEW = core::mem::zeroed();
    let mut registry_mode: DEVMODEW = core::mem::zeroed();
    let mut adapter: *mut wined3d_adapter = core::ptr::null_mut::<wined3d_adapter>();
    let mut display_device: DISPLAY_DEVICEW = core::mem::zeroed();
    let mut output: *mut wined3d_output = core::ptr::null_mut::<wined3d_output>();
    let mut do_mode_change: BOOL = (FALSE as i32);
    let mut ret: LONG = 0;

    TRACE!("wined3d %p.\n", wined3d);

    memset(
        &raw mut current_mode as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<DEVMODEW>() as size_t,
    );
    memset(
        &raw mut registry_mode as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<DEVMODEW>() as size_t,
    );
    current_mode.dmSize = ::core::mem::size_of::<DEVMODEW>() as WORD;
    registry_mode.dmSize = ::core::mem::size_of::<DEVMODEW>() as WORD;
    display_device.cb = ::core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD;
    loop {
        let fresh16 = output_idx;
        output_idx = output_idx.wrapping_add(1);
        if !(EnumDisplayDevicesW(
            core::ptr::null::<WCHAR>(),
            fresh16,
            &raw mut display_device,
            0 as DWORD,
        ) != 0)
        {
            break;
        }
        if EnumDisplaySettingsExW(
            &raw mut display_device.DeviceName as *mut WCHAR as LPCWSTR,
            ENUM_CURRENT_SETTINGS,
            &raw mut current_mode,
            0 as DWORD,
        ) == 0
        {
            ERR!(
                "Failed to read the current display mode for %s.\n",
                wine_dbgstr_w(&raw mut display_device.DeviceName as *mut WCHAR),
            );
            return WINED3DERR_NOTAVAILABLE;
        }
        if EnumDisplaySettingsExW(
            &raw mut display_device.DeviceName as *mut WCHAR as LPCWSTR,
            ENUM_REGISTRY_SETTINGS,
            &raw mut registry_mode,
            0 as DWORD,
        ) == 0
        {
            ERR!(
                "Failed to read the registry display mode for %s.\n",
                wine_dbgstr_w(&raw mut display_device.DeviceName as *mut WCHAR),
            );
            return WINED3DERR_NOTAVAILABLE;
        }
        if !(equal_display_mode(&raw mut current_mode, &raw mut registry_mode) == 0) {
            continue;
        }
        do_mode_change = (TRUE as i32);
        break;
    }

    if do_mode_change != 0 {
        ret = ChangeDisplaySettingsExW(
            core::ptr::null::<WCHAR>(),
            core::ptr::null_mut::<_devicemodeW>(),
            core::ptr::null_mut(),
            0 as DWORD,
            core::ptr::null_mut(),
        );
        if ret != (DISP_CHANGE_SUCCESSFUL as LONG) {
            ERR!(
                "Failed to restore all outputs to their registry display settings, error %ld.\n",
                ret,
            );
            return WINED3DERR_NOTAVAILABLE;
        }
    } else {
        TRACE!("Skipping redundant mode setting call.\n");
    }

    adapter_idx = 0;
    while adapter_idx < (*wined3d).adapter_count {
        adapter = *(&raw mut (*wined3d).adapters as *mut *mut wined3d_adapter)
            .offset(adapter_idx as isize);
        output_idx = 0;
        while (output_idx as SIZE_T) < (*adapter).output_count {
            output = (*adapter).outputs.offset(output_idx as isize) as *mut wined3d_output;

            if EnumDisplaySettingsExW(
                &raw mut (*output).device_name as *mut WCHAR as LPCWSTR,
                ENUM_CURRENT_SETTINGS,
                &raw mut current_mode,
                0 as DWORD,
            ) == 0
            {
                ERR!(
                    "Failed to read the current display mode for %s.\n",
                    wine_dbgstr_w(&raw mut (*output).device_name as *mut WCHAR),
                );
                return WINED3DERR_NOTAVAILABLE;
            }

            (*output).screen_format = pixelformat_for_depth(current_mode.dmBitsPerPel);
            output_idx = output_idx.wrapping_add(1);
        }
        adapter_idx = adapter_idx.wrapping_add(1);
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_output_set_display_mode(
    output: *mut wined3d_output,
    mode: *const wined3d_display_mode,
) -> HRESULT {
    let mut new_format_id: wined3d_format_id = core::mem::zeroed();
    let mut format: *const wined3d_format = core::ptr::null();
    let mut new_mode: DEVMODEW = core::mem::zeroed();
    let mut current_mode: DEVMODEW = core::mem::zeroed();
    let mut ret: LONG = 0;

    memset(
        core::ptr::addr_of_mut!(new_mode) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<DEVMODEW>() as usize,
    );
    new_mode.dmSize = core::mem::size_of::<DEVMODEW>() as WORD;
    memset(
        core::ptr::addr_of_mut!(current_mode) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<DEVMODEW>() as usize,
    );
    current_mode.dmSize = core::mem::size_of::<DEVMODEW>() as WORD;

    format = wined3d_get_format(
        (*output).adapter,
        (*mode).format_id,
        WINED3D_BIND_RENDER_TARGET as core::ffi::c_uint,
    );

    new_mode.dmFields = (DM_BITSPERPEL | DM_PELSWIDTH | DM_PELSHEIGHT) as DWORD;
    new_mode.dmBitsPerPel = (*format).byte_count.wrapping_mul(CHAR_BIT as UINT) as DWORD;
    new_mode.dmPelsWidth = (*mode).width as DWORD;
    new_mode.dmPelsHeight = (*mode).height as DWORD;
    new_mode.dmDisplayFrequency = (*mode).refresh_rate as DWORD;
    if (*mode).refresh_rate != 0 {
        new_mode.dmFields |= DM_DISPLAYFREQUENCY as DWORD;
    }
    if (*mode).scanline_ordering as core::ffi::c_uint
        != WINED3D_SCANLINE_ORDERING_UNKNOWN as core::ffi::c_uint
    {
        new_mode.dmFields |= DM_DISPLAYFLAGS as DWORD;
        if (*mode).scanline_ordering as core::ffi::c_uint
            == WINED3D_SCANLINE_ORDERING_INTERLACED as core::ffi::c_uint
        {
            new_mode.__bindgen_anon_2.dmDisplayFlags |= DM_INTERLACED as DWORD;
        }
    }
    new_format_id = (*mode).format_id;

    /* Only change the mode if necessary. */
    if EnumDisplaySettingsW(
        core::ptr::addr_of_mut!((*output).device_name) as *mut WCHAR as LPCWSTR,
        ENUM_CURRENT_SETTINGS,
        core::ptr::addr_of_mut!(current_mode),
    ) == 0
    {
        ERR!("Failed to get current display mode.\n");
    } else if equal_display_mode(
        core::ptr::addr_of!(current_mode),
        core::ptr::addr_of!(new_mode),
    ) != 0
    {
        (*output).screen_format = new_format_id;
        return WINED3D_OK;
    }

    ret = ChangeDisplaySettingsExW(
        core::ptr::addr_of_mut!((*output).device_name) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(new_mode),
        core::ptr::null_mut(),
        CDS_FULLSCREEN as DWORD,
        core::ptr::null_mut(),
    );
    if ret != DISP_CHANGE_SUCCESSFUL as LONG {
        if (new_mode.dmFields & (DM_DISPLAYFREQUENCY as DWORD)) != 0 {
            WARN!("ChangeDisplaySettingsExW failed, trying without the refresh rate.\n");
            new_mode.dmFields &= !(DM_DISPLAYFREQUENCY as DWORD);
            new_mode.dmDisplayFrequency = 0 as DWORD;
            ret = ChangeDisplaySettingsExW(
                core::ptr::addr_of_mut!((*output).device_name) as *mut WCHAR as LPCWSTR,
                core::ptr::addr_of_mut!(new_mode),
                core::ptr::null_mut(),
                CDS_FULLSCREEN as DWORD,
                core::ptr::null_mut(),
            );
        }
        if ret != DISP_CHANGE_SUCCESSFUL as LONG {
            return WINED3DERR_NOTAVAILABLE;
        }
    }

    /* Store the new values. */
    (*output).screen_format = new_format_id;

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_output_set_gamma_ramp(output: *mut wined3d_output, ramp: *const wined3d_gamma_ramp) -> HRESULT {
    let mut dc: HDC = core::ptr::null_mut();

    dc = CreateDCW(
        core::ptr::addr_of_mut!((*output).device_name) as LPCWSTR,
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null(),
    );
    SetDeviceGammaRamp(dc, ramp as *mut core::ffi::c_void);
    DeleteDC(dc);

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_output_get_gamma_ramp(
    output: *mut wined3d_output,
    ramp: *mut wined3d_gamma_ramp,
) -> HRESULT {
    let mut dc: HDC = core::ptr::null_mut();

    dc = CreateDCW(
        core::ptr::addr_of_mut!((*output).device_name) as *mut WCHAR as LPCWSTR,
        core::ptr::null::<WCHAR>(),
        core::ptr::null::<WCHAR>(),
        core::ptr::null::<DEVMODEW>(),
    );
    GetDeviceGammaRamp(dc, ramp as LPVOID);
    DeleteDC(dc);

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_get_identifier(
    adapter: *const wined3d_adapter,
    flags: uint32_t,
    identifier: *mut wined3d_adapter_identifier,
) -> HRESULT {
    wined3d_mutex_lock();

    wined3d_copy_name(
        (*identifier).driver,
        (*adapter).driver_info.name,
        (*identifier).driver_size,
    );
    wined3d_copy_name(
        (*identifier).description,
        core::ptr::addr_of!((*adapter).driver_info.description) as *const ::core::ffi::c_char,
        (*identifier).description_size,
    );

    (*identifier).driver_version.u.HighPart = (*adapter).driver_info.version_high as LONG;
    (*identifier).driver_version.u.LowPart = (*adapter).driver_info.version_low as DWORD;
    (*identifier).vendor_id = (*adapter).driver_info.vendor as DWORD;
    (*identifier).device_id = (*adapter).driver_info.device as DWORD;
    (*identifier).subsystem_id = 0 as DWORD;
    (*identifier).revision = 0 as DWORD;
    (*identifier).device_identifier = IID_D3DDEVICE_D3DUID;
    (*identifier).driver_uuid = (*adapter).driver_uuid;
    (*identifier).device_uuid = (*adapter).device_uuid;
    (*identifier).whql_level = (if (flags & (WINED3DENUM_WHQL_LEVEL as uint32_t)) != 0 {
        1
    } else {
        0
    }) as DWORD;
    (*identifier).adapter_luid = (*adapter).luid;
    (*identifier).video_memory = min!(
        !(0 as SIZE_T),
        (*adapter).driver_info.vram_bytes as SIZE_T,
    ) as SIZE_T;
    (*identifier).shared_system_memory = min!(
        !(0 as SIZE_T),
        (*adapter).driver_info.sysmem_bytes as SIZE_T,
    ) as SIZE_T;

    wined3d_mutex_unlock();

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_output_get_raster_status(
    output: *const wined3d_output,
    raster_status: *mut wined3d_raster_status,
) -> HRESULT {
    let mut freq_per_frame: LONGLONG = 0;
    let mut freq_per_line: LONGLONG = 0;
    let mut counter: LARGE_INTEGER = core::mem::zeroed();
    let mut freq_per_sec: LARGE_INTEGER = core::mem::zeroed();
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    static mut once: UINT = 0;

    let fresh_once = once;
    once = once.wrapping_add(1);
    if fresh_once == 0 {
        FIXME!(
            "output %p, raster_status %p semi-stub!\n",
            output,
            raster_status
        );
    } else {
        WARN!(
            "output %p, raster_status %p semi-stub!\n",
            output,
            raster_status
        );
    }

    if QueryPerformanceCounter(&raw mut counter) == 0
        || QueryPerformanceFrequency(&raw mut freq_per_sec) == 0
    {
        return WINED3DERR_INVALIDCALL;
    }
    if wined3d_output_get_display_mode(output, &raw mut mode, core::ptr::null_mut()) < 0 {
        return WINED3DERR_INVALIDCALL;
    }
    if mode.refresh_rate == DEFAULT_REFRESH_RATE as UINT {
        mode.refresh_rate = 60 as UINT;
    }

    freq_per_frame = freq_per_sec.QuadPart / mode.refresh_rate as LONGLONG;
    freq_per_line = freq_per_frame / (mode.height.wrapping_add(20 as UINT) as LONGLONG);
    (*raster_status).scan_line =
        ((counter.QuadPart % freq_per_frame) / freq_per_line) as UINT;
    if (*raster_status).scan_line < mode.height {
        (*raster_status).in_vblank = FALSE as BOOL;
    } else {
        (*raster_status).scan_line = 0 as UINT;
        (*raster_status).in_vblank = TRUE as BOOL;
    }

    TRACE!(
        "Returning fake value, in_vblank %u, scan_line %u.\n",
        (*raster_status).in_vblank,
        (*raster_status).scan_line
    );

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_check_depth_stencil_match(
    adapter: *const wined3d_adapter,
    device_type: wined3d_device_type,
    adapter_format_id: wined3d_format_id,
    render_target_format_id: wined3d_format_id,
    depth_stencil_format_id: wined3d_format_id,
) -> HRESULT {
    let mut rt_format: *const wined3d_format = core::ptr::null::<wined3d_format>();
    let mut ds_format: *const wined3d_format = core::ptr::null::<wined3d_format>();

    TRACE!(
        "adapter %p, device_type %s, adapter_format %s, render_target_format %s, depth_stencil_format %s.\n",
        adapter,
        debug_d3ddevicetype(device_type),
        debug_d3dformat(adapter_format_id),
        debug_d3dformat(render_target_format_id),
        debug_d3dformat(depth_stencil_format_id),
    );

    rt_format = wined3d_get_format(adapter, render_target_format_id, WINED3D_BIND_RENDER_TARGET);
    ds_format = wined3d_get_format(adapter, depth_stencil_format_id, WINED3D_BIND_DEPTH_STENCIL);

    if ((*rt_format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] & WINED3D_FORMAT_CAP_RENDERTARGET) == 0 {
        WARN!("Format %s is not render target format.\n", debug_d3dformat((*rt_format).id));
        return WINED3DERR_NOTAVAILABLE;
    }
    if ((*ds_format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] & WINED3D_FORMAT_CAP_DEPTH_STENCIL) == 0 {
        WARN!("Format %s is not depth/stencil format.\n", debug_d3dformat((*ds_format).id));
        return WINED3DERR_NOTAVAILABLE;
    }

    if ((*(*adapter).adapter_ops)
        .adapter_check_format
        .unwrap_unchecked())(
        adapter,
        core::ptr::null::<wined3d_format>(),
        rt_format,
        ds_format,
    ) != 0
    {
        TRACE!("Formats match.\n");
        return WINED3D_OK;
    }

    TRACE!(
        "Unsupported format pair: %s and %s.\n",
        debug_d3dformat(render_target_format_id),
        debug_d3dformat(depth_stencil_format_id),
    );

    return WINED3DERR_NOTAVAILABLE;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_check_device_multisample_type(
    adapter: *const wined3d_adapter,
    device_type: wined3d_device_type,
    surface_format_id: wined3d_format_id,
    windowed: BOOL,
    multisample_type: wined3d_multisample_type,
    quality_levels: *mut ::core::ffi::c_uint,
) -> HRESULT {
    let mut format: *const wined3d_format = ::core::ptr::null::<wined3d_format>();
    let mut hr: HRESULT = WINED3D_OK;
    WINE_TRACE!("adapter %p, device_type %s, surface_format %s, windowed %#x, multisample_type %#x, quality_levels %p.\n", adapter, debug_d3ddevicetype(device_type), debug_d3dformat(surface_format_id), windowed, multisample_type as ::core::ffi::c_uint, quality_levels);
    if surface_format_id as ::core::ffi::c_uint
        == WINED3DFMT_UNKNOWN as ::core::ffi::c_uint
    {
        return WINED3DERR_INVALIDCALL;
    }
    if (multisample_type as ::core::ffi::c_uint)
        < WINED3D_MULTISAMPLE_NONE as ::core::ffi::c_uint
    {
        return WINED3DERR_INVALIDCALL;
    }
    if multisample_type as ::core::ffi::c_uint
        > WINED3D_MULTISAMPLE_16_SAMPLES as ::core::ffi::c_uint
    {
        WINE_FIXME!("multisample_type %u not handled yet.\n", multisample_type as ::core::ffi::c_uint);
        return WINED3DERR_NOTAVAILABLE;
    }
    format = wined3d_get_format(
        adapter,
        surface_format_id,
        0 as ::core::ffi::c_uint,
    );
    if multisample_type as ::core::ffi::c_uint != WINED3D_MULTISAMPLE_NON_MASKABLE as ::core::ffi::c_uint
        && (*format).multisample_types as ::core::ffi::c_uint
            & (1 as ::core::ffi::c_uint)
                << (multisample_type as ::core::ffi::c_uint)
                    .wrapping_sub(1 as ::core::ffi::c_uint)
            == 0
    {
        hr = WINED3DERR_NOTAVAILABLE;
    }
    if hr >= 0
        || multisample_type as ::core::ffi::c_uint
            == WINED3D_MULTISAMPLE_NON_MASKABLE as ::core::ffi::c_uint
            && (*format).multisample_types != 0
    {
        if !quality_levels.is_null() {
            if multisample_type as ::core::ffi::c_uint
                == WINED3D_MULTISAMPLE_NON_MASKABLE as ::core::ffi::c_uint
            {
                *quality_levels =
                    wined3d_popcount((*format).multisample_types as ::core::ffi::c_uint);
            } else {
                *quality_levels = 1 as ::core::ffi::c_uint;
            }
        }
        return WINED3D_OK;
    }
    WINE_TRACE!("Returning not supported.\n");
    return hr;
}

unsafe extern "C" fn wined3d_check_depth_stencil_format(
    adapter: *const wined3d_adapter,
    adapter_format: *const wined3d_format,
    ds_format: *const wined3d_format,
    gl_type: wined3d_gl_resource_type,
) -> BOOL {
    if (*ds_format).depth_size == 0 && (*ds_format).stencil_size == 0 {
        return (FALSE) as BOOL;
    }

    return (*(*adapter).adapter_ops)
        .adapter_check_format
        .unwrap_unchecked()(
        adapter,
        adapter_format,
        core::ptr::null::<wined3d_format>(),
        ds_format,
    );
}

unsafe extern "C" fn wined3d_check_surface_format(format: *const wined3d_format) -> BOOL {
    if ((*format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] | (*format).caps[WINED3D_GL_RES_TYPE_RB as usize])
        & WINED3D_FORMAT_CAP_BLIT != 0
    {
        return (TRUE) as BOOL;
    }

    if (*format).attrs & WINED3D_FORMAT_ATTR_EXTENSION != 0
        && (*format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] & WINED3D_FORMAT_CAP_TEXTURE != 0
    {
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_check_device_format(
    wined3d: *const wined3d,
    adapter: *const wined3d_adapter,
    device_type: wined3d_device_type,
    adapter_format_id: wined3d_format_id,
    usage: uint32_t,
    bind_flags: ::core::ffi::c_uint,
    resource_type: wined3d_resource_type,
    check_format_id: wined3d_format_id,
) -> HRESULT {
    let mut adapter_format: *const wined3d_format = core::ptr::null::<wined3d_format>();
    let mut format: *const wined3d_format = core::ptr::null::<wined3d_format>();
    let mut gl_type: wined3d_gl_resource_type = 0;
    let mut gl_type_end: wined3d_gl_resource_type = 0;
    let mut format_caps: ::core::ffi::c_uint = 0;
    let mut format_attrs: ::core::ffi::c_uint = 0;
    let mut mipmap_gen_supported: BOOL = (TRUE as i32);
    let mut allowed_bind_flags: ::core::ffi::c_uint = 0;
    let mut allowed_usage: uint32_t = 0;

    TRACE!(
        "wined3d %p, adapter %p, device_type %s, adapter_format %s, usage %s, bind_flags %s, resource_type %s, check_format %s.\n",
        wined3d,
        adapter,
        debug_d3ddevicetype(device_type),
        debug_d3dformat(adapter_format_id),
        debug_d3dusage(usage),
        wined3d_debug_bind_flags(bind_flags as uint32_t),
        debug_d3dresourcetype(resource_type),
        debug_d3dformat(check_format_id),
    );

    adapter_format = wined3d_get_format(adapter, adapter_format_id, WINED3D_BIND_RENDER_TARGET);
    format = wined3d_get_format(adapter, check_format_id, bind_flags);

    'sw: {
        match resource_type {
            WINED3D_RTYPE_NONE => {
                allowed_usage = 0;
                allowed_bind_flags = WINED3D_BIND_RENDER_TARGET
                    | WINED3D_BIND_DEPTH_STENCIL
                    | WINED3D_BIND_UNORDERED_ACCESS;
                gl_type = WINED3D_GL_RES_TYPE_TEX_2D;
                gl_type_end = WINED3D_GL_RES_TYPE_TEX_3D;
                break 'sw;
            }
            WINED3D_RTYPE_TEXTURE_1D => {
                allowed_usage = WINED3DUSAGE_DYNAMIC
                    | WINED3DUSAGE_SOFTWAREPROCESSING
                    | WINED3DUSAGE_QUERY_FILTER
                    | WINED3DUSAGE_QUERY_GENMIPMAP
                    | WINED3DUSAGE_QUERY_POSTPIXELSHADER_BLENDING
                    | WINED3DUSAGE_QUERY_SRGBREAD
                    | WINED3DUSAGE_QUERY_SRGBWRITE
                    | WINED3DUSAGE_QUERY_VERTEXTEXTURE
                    | WINED3DUSAGE_QUERY_WRAPANDMIP;
                allowed_bind_flags = WINED3D_BIND_SHADER_RESOURCE
                    | WINED3D_BIND_UNORDERED_ACCESS;
                gl_type = WINED3D_GL_RES_TYPE_TEX_1D;
                gl_type_end = gl_type;
                break 'sw;
            }
            WINED3D_RTYPE_TEXTURE_2D => {
                allowed_usage = WINED3DUSAGE_QUERY_POSTPIXELSHADER_BLENDING;
                if (bind_flags & WINED3D_BIND_RENDER_TARGET) != 0 {
                    allowed_usage |= WINED3DUSAGE_QUERY_SRGBWRITE;
                }
                allowed_bind_flags = WINED3D_BIND_RENDER_TARGET
                    | WINED3D_BIND_DEPTH_STENCIL
                    | WINED3D_BIND_UNORDERED_ACCESS;
                gl_type = WINED3D_GL_RES_TYPE_TEX_2D;
                gl_type_end = gl_type;
                if (bind_flags & WINED3D_BIND_SHADER_RESOURCE) == 0 {
                    if wined3d_check_surface_format(format) == 0 {
                        TRACE!("%s is not supported for plain surfaces.\n", debug_d3dformat((*format).id));
                        return WINED3DERR_NOTAVAILABLE;
                    }
                    break 'sw;
                }
                allowed_usage |= WINED3DUSAGE_DYNAMIC
                    | WINED3DUSAGE_LEGACY_CUBEMAP
                    | WINED3DUSAGE_SOFTWAREPROCESSING
                    | WINED3DUSAGE_QUERY_FILTER
                    | WINED3DUSAGE_QUERY_GENMIPMAP
                    | WINED3DUSAGE_QUERY_LEGACYBUMPMAP
                    | WINED3DUSAGE_QUERY_SRGBREAD
                    | WINED3DUSAGE_QUERY_SRGBWRITE
                    | WINED3DUSAGE_QUERY_VERTEXTEXTURE
                    | WINED3DUSAGE_QUERY_WRAPANDMIP;
                allowed_bind_flags |= WINED3D_BIND_SHADER_RESOURCE;
                if (usage & WINED3DUSAGE_LEGACY_CUBEMAP) != 0 {
                    allowed_usage &= !WINED3DUSAGE_QUERY_LEGACYBUMPMAP;
                    allowed_bind_flags &= !WINED3D_BIND_DEPTH_STENCIL;
                    gl_type = WINED3D_GL_RES_TYPE_TEX_CUBE;
                    gl_type_end = gl_type;
                }
                break 'sw;
            }
            WINED3D_RTYPE_TEXTURE_3D => {
                allowed_usage = WINED3DUSAGE_DYNAMIC
                    | WINED3DUSAGE_SOFTWAREPROCESSING
                    | WINED3DUSAGE_QUERY_FILTER
                    | WINED3DUSAGE_QUERY_POSTPIXELSHADER_BLENDING
                    | WINED3DUSAGE_QUERY_SRGBREAD
                    | WINED3DUSAGE_QUERY_SRGBWRITE
                    | WINED3DUSAGE_QUERY_VERTEXTEXTURE
                    | WINED3DUSAGE_QUERY_WRAPANDMIP;
                allowed_bind_flags = WINED3D_BIND_SHADER_RESOURCE
                    | WINED3D_BIND_UNORDERED_ACCESS;
                gl_type = WINED3D_GL_RES_TYPE_TEX_3D;
                gl_type_end = gl_type;
                break 'sw;
            }
            WINED3D_RTYPE_BUFFER => {
                if wined3d_format_is_typeless(format) != 0 {
                    TRACE!(
                        "Requested WINED3D_RTYPE_BUFFER, but format %s is typeless.\n",
                        debug_d3dformat(check_format_id),
                    );
                    return WINED3DERR_NOTAVAILABLE;
                }

                allowed_usage = WINED3DUSAGE_DYNAMIC;
                allowed_bind_flags = WINED3D_BIND_SHADER_RESOURCE
                    | WINED3D_BIND_UNORDERED_ACCESS
                    | WINED3D_BIND_VERTEX_BUFFER
                    | WINED3D_BIND_INDEX_BUFFER;
                gl_type = WINED3D_GL_RES_TYPE_BUFFER;
                gl_type_end = gl_type;
                break 'sw;
            }
            _ => {
                FIXME!("Unhandled resource type %s.\n", debug_d3dresourcetype(resource_type));
                return WINED3DERR_NOTAVAILABLE;
            }
        }
    }

    if (usage & allowed_usage) != usage {
        TRACE!(
            "Requested usage %#x, but resource type %s only allows %#x.\n",
            usage,
            debug_d3dresourcetype(resource_type),
            allowed_usage,
        );
        return WINED3DERR_NOTAVAILABLE;
    }

    if (bind_flags & allowed_bind_flags) != bind_flags {
        TRACE!(
            "Requested bind flags %s, but resource type %s only allows %s.\n",
            wined3d_debug_bind_flags(bind_flags as uint32_t),
            debug_d3dresourcetype(resource_type),
            wined3d_debug_bind_flags(allowed_bind_flags as uint32_t),
        );
        return WINED3DERR_NOTAVAILABLE;
    }

    if (bind_flags & WINED3D_BIND_SHADER_RESOURCE) != 0 {
        format_caps |= WINED3D_FORMAT_CAP_TEXTURE;
    }
    if (bind_flags & WINED3D_BIND_RENDER_TARGET) != 0 {
        format_caps |= WINED3D_FORMAT_CAP_RENDERTARGET;
    }
    if (bind_flags & WINED3D_BIND_DEPTH_STENCIL) != 0 {
        format_caps |= WINED3D_FORMAT_CAP_DEPTH_STENCIL;
    }
    if (bind_flags & WINED3D_BIND_UNORDERED_ACCESS) != 0 {
        format_caps |= WINED3D_FORMAT_CAP_UNORDERED_ACCESS;
    }
    if (bind_flags & WINED3D_BIND_VERTEX_BUFFER) != 0 {
        format_caps |= WINED3D_FORMAT_CAP_VERTEX_ATTRIBUTE;
    }
    if (bind_flags & WINED3D_BIND_INDEX_BUFFER) != 0 {
        format_caps |= WINED3D_FORMAT_CAP_INDEX_BUFFER;
    }

    if (usage & WINED3DUSAGE_QUERY_FILTER) != 0 {
        format_caps |= WINED3D_FORMAT_CAP_FILTERING;
    }
    if (usage & WINED3DUSAGE_QUERY_POSTPIXELSHADER_BLENDING) != 0 {
        format_caps |= WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING;
    }
    if (usage & WINED3DUSAGE_QUERY_SRGBREAD) != 0 {
        format_caps |= WINED3D_FORMAT_CAP_SRGB_READ;
    }
    if (usage & WINED3DUSAGE_QUERY_SRGBWRITE) != 0 {
        format_caps |= WINED3D_FORMAT_CAP_SRGB_WRITE;
    }
    if (usage & WINED3DUSAGE_QUERY_VERTEXTEXTURE) != 0 {
        format_caps |= WINED3D_FORMAT_CAP_VTF;
    }
    if (usage & WINED3DUSAGE_QUERY_LEGACYBUMPMAP) != 0 {
        format_attrs |= WINED3D_FORMAT_ATTR_BUMPMAP;
    }

    if (format_caps & WINED3D_FORMAT_CAP_TEXTURE) != 0 && ((*wined3d).flags & WINED3D_NO3D) != 0 {
        TRACE!("Requested texturing support, but wined3d was created with WINED3D_NO3D.\n");
        return WINED3DERR_NOTAVAILABLE;
    }

    if ((*format).attrs & format_attrs) != format_attrs {
        TRACE!(
            "Requested format attributes %#x, but format %s only has %#x.\n",
            format_attrs,
            debug_d3dformat(check_format_id),
            (*format).attrs,
        );
        return WINED3DERR_NOTAVAILABLE;
    }

    while gl_type <= gl_type_end {
        let mut caps: ::core::ffi::c_uint = (*format).caps[gl_type as usize];

        if gl_type == WINED3D_GL_RES_TYPE_TEX_2D && (bind_flags & WINED3D_BIND_SHADER_RESOURCE) == 0 {
            caps |= (*format).caps[WINED3D_GL_RES_TYPE_RB as usize];
        }

        if (bind_flags & WINED3D_BIND_RENDER_TARGET) != 0
            && (*(*adapter).adapter_ops)
                .adapter_check_format
                .unwrap_unchecked()(
                adapter,
                adapter_format,
                format,
                core::ptr::null::<wined3d_format>(),
            ) == 0
        {
            TRACE!(
                "Requested WINED3D_BIND_RENDER_TARGET, but format %s is not supported for render targets.\n",
                debug_d3dformat(check_format_id),
            );
            return WINED3DERR_NOTAVAILABLE;
        }

        /* 3D depth / stencil textures are never supported. */
        if !(bind_flags == WINED3D_BIND_DEPTH_STENCIL && gl_type == WINED3D_GL_RES_TYPE_TEX_3D) {
            if (bind_flags & WINED3D_BIND_DEPTH_STENCIL) != 0
                && wined3d_check_depth_stencil_format(adapter, adapter_format, format, gl_type) == 0
            {
                TRACE!(
                    "Requested WINED3D_BIND_DEPTH_STENCIL, but format %s is not supported for depth/stencil buffers.\n",
                    debug_d3dformat(check_format_id),
                );
                return WINED3DERR_NOTAVAILABLE;
            }

            if (bind_flags & WINED3D_BIND_UNORDERED_ACCESS) != 0 && wined3d_format_is_typeless(format) != 0 {
                TRACE!(
                    "Requested WINED3D_BIND_UNORDERED_ACCESS, but format %s is typeless.\n",
                    debug_d3dformat(check_format_id),
                );
                return WINED3DERR_NOTAVAILABLE;
            }

            if (caps & format_caps) != format_caps {
                TRACE!(
                    "Requested format caps %#x, but format %s only has %#x.\n",
                    format_caps,
                    debug_d3dformat(check_format_id),
                    caps,
                );
                return WINED3DERR_NOTAVAILABLE;
            }

            if (caps & WINED3D_FORMAT_CAP_GEN_MIPMAP) == 0 {
                mipmap_gen_supported = (FALSE as i32);
            }
        }

        gl_type += 1;
    }

    if (usage & WINED3DUSAGE_QUERY_GENMIPMAP) != 0 && mipmap_gen_supported == 0 {
        TRACE!("No WINED3DUSAGE_AUTOGENMIPMAP support, returning WINED3DOK_NOAUTOGEN.\n");
        return WINED3DOK_NOMIPGEN;
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_calculate_format_pitch(
    mut adapter: *const wined3d_adapter,
    mut format_id: wined3d_format_id,
    mut width: ::core::ffi::c_uint,
) -> ::core::ffi::c_uint {
    let mut row_pitch: ::core::ffi::c_uint = 0;
    let mut slice_pitch: ::core::ffi::c_uint = 0;
    WINE_TRACE!("adapter %p, format_id %s, width %u.\n", adapter, debug_d3dformat(format_id), width);
    wined3d_format_calculate_pitch(
        wined3d_get_format(adapter, format_id, 0),
        1,
        width,
        1,
        &raw mut row_pitch,
        &raw mut slice_pitch,
    );
    return row_pitch;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_check_device_format_conversion(
    output: *const wined3d_output,
    device_type: wined3d_device_type,
    src_format_id: wined3d_format_id,
    dst_format_id: wined3d_format_id,
) -> HRESULT {
    let src_format: *const wined3d_format =
        wined3d_get_format((*output).adapter, src_format_id, 0);
    let dst_format: *const wined3d_format =
        wined3d_get_format((*output).adapter, dst_format_id, 0);

    if !(((*src_format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] & WINED3D_FORMAT_CAP_BLIT) != 0) {
        return WINED3DERR_NOTAVAILABLE;
    }

    if !(((*dst_format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] & WINED3D_FORMAT_CAP_BLIT) != 0) {
        return WINED3DERR_NOTAVAILABLE;
    }

    /* Source cannot be depth/stencil (although it can be YUV or compressed,
     * and AMD also allows blitting from luminance formats). */
    if (*src_format).depth_size != 0 || (*src_format).stencil_size != 0 {
        return WINED3DERR_NOTAVAILABLE;
    }

    /* The destination format must be a simple RGB format (no luminance, YUV,
     * compression, etc.) All such formats have a nonzero red_size; the only
     * exceptions are X24G8 (not supported in d3d9) and A8 (which, it turns out,
     * no vendor reports support for converting to). */
    if (*dst_format).red_size == 0 {
        return WINED3DERR_NOTAVAILABLE;
    }

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_check_device_type(
    wined3d: *const wined3d,
    output: *mut wined3d_output,
    device_type: wined3d_device_type,
    display_format: wined3d_format_id,
    mut backbuffer_format: wined3d_format_id,
    windowed: BOOL,
) -> HRESULT {
    let mut present_conversion: BOOL = ((*wined3d).flags & WINED3D_PRESENT_CONVERSION) as BOOL;

    if !(display_format == WINED3DFMT_B5G6R5_UNORM
        || display_format == WINED3DFMT_B5G5R5X1_UNORM
        || display_format == WINED3DFMT_B8G8R8X8_UNORM
        || display_format == WINED3DFMT_B10G10R10A2_UNORM)
    {
        TRACE!(
            "Format %s is not supported as display format.\n",
            debug_d3dformat(display_format)
        );
        return WINED3DERR_NOTAVAILABLE;
    }

    if windowed == 0 {
        if wined3d_output_get_mode_count(
            output,
            display_format,
            WINED3D_SCANLINE_ORDERING_UNKNOWN,
            false,
        ) == 0
        {
            TRACE!(
                "No available modes for display format %s.\n",
                debug_d3dformat(display_format)
            );
            return WINED3DERR_NOTAVAILABLE;
        }

        present_conversion = FALSE as BOOL;
    } else if display_format == WINED3DFMT_B10G10R10A2_UNORM {
        TRACE!(
            "Unsupported format combination %s / %s in windowed mode.\n",
            debug_d3dformat(display_format),
            debug_d3dformat(backbuffer_format)
        );
        return WINED3DERR_NOTAVAILABLE;
    }

    if present_conversion != 0 {
        if backbuffer_format == WINED3DFMT_UNKNOWN {
            backbuffer_format = display_format;
        }

        if (wined3d_check_device_format_conversion(output, device_type, backbuffer_format, display_format))
            < 0
        {
            TRACE!(
                "Format conversion from %s to %s not supported.\n",
                debug_d3dformat(backbuffer_format),
                debug_d3dformat(display_format)
            );
            return WINED3DERR_NOTAVAILABLE;
        }
    } else {
        if display_format == WINED3DFMT_B5G6R5_UNORM && backbuffer_format != WINED3DFMT_B5G6R5_UNORM {
            TRACE!(
                "Unsupported format combination %s / %s.\n",
                debug_d3dformat(display_format),
                debug_d3dformat(backbuffer_format)
            );
            return WINED3DERR_NOTAVAILABLE;
        }

        if display_format == WINED3DFMT_B5G5R5X1_UNORM
            && !(backbuffer_format == WINED3DFMT_B5G5R5X1_UNORM || backbuffer_format == WINED3DFMT_B5G5R5A1_UNORM)
        {
            TRACE!(
                "Unsupported format combination %s / %s.\n",
                debug_d3dformat(display_format),
                debug_d3dformat(backbuffer_format)
            );
            return WINED3DERR_NOTAVAILABLE;
        }

        if display_format == WINED3DFMT_B8G8R8X8_UNORM
            && !(backbuffer_format == WINED3DFMT_B8G8R8X8_UNORM || backbuffer_format == WINED3DFMT_B8G8R8A8_UNORM)
        {
            TRACE!(
                "Unsupported format combination %s / %s.\n",
                debug_d3dformat(display_format),
                debug_d3dformat(backbuffer_format)
            );
            return WINED3DERR_NOTAVAILABLE;
        }

        if display_format == WINED3DFMT_B10G10R10A2_UNORM && backbuffer_format != WINED3DFMT_B10G10R10A2_UNORM {
            TRACE!(
                "Unsupported format combination %s / %s.\n",
                debug_d3dformat(display_format),
                debug_d3dformat(backbuffer_format)
            );
            return WINED3DERR_NOTAVAILABLE;
        }
    }

    if (wined3d_check_device_format(
        wined3d,
        (*output).adapter,
        device_type,
        display_format,
        0,
        WINED3D_BIND_RENDER_TARGET,
        WINED3D_RTYPE_TEXTURE_2D,
        backbuffer_format,
    )) < 0
    {
        TRACE!(
            "Format %s not allowed for render targets.\n",
            debug_d3dformat(backbuffer_format)
        );
        return WINED3DERR_NOTAVAILABLE;
    }

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_get_device_caps(
    adapter: *const wined3d_adapter,
    device_type: wined3d_device_type,
    caps: *mut wined3d_caps,
) -> HRESULT {
    let mut d3d_info: *const wined3d_d3d_info = core::ptr::null::<wined3d_d3d_info>();
    let mut vertex_caps: wined3d_vertex_caps = core::mem::zeroed();
    let mut ckey_caps: DWORD = 0;
    let mut blit_caps: DWORD = 0;
    let mut fx_caps: DWORD = 0;
    let mut fragment_caps: fragment_caps = core::mem::zeroed();
    let mut shader_caps: shader_caps = core::mem::zeroed();

    WINE_TRACE!("adapter %p, device_type %s, caps %p.\n", adapter, debug_d3ddevicetype(device_type), caps);

    d3d_info = &raw const (*adapter).d3d_info;

    (*caps).DeviceType = (if device_type as ::core::ffi::c_uint
        == WINED3D_DEVICE_TYPE_HAL as ::core::ffi::c_uint
    {
        WINED3D_DEVICE_TYPE_HAL as ::core::ffi::c_int
    } else {
        WINED3D_DEVICE_TYPE_REF as ::core::ffi::c_int
    }) as wined3d_device_type;

    (*caps).Caps = 0 as DWORD;
    (*caps).Caps2 = (WINED3DCAPS2_CANRENDERWINDOWED
        | WINED3DCAPS2_FULLSCREENGAMMA
        | WINED3DCAPS2_DYNAMICTEXTURES) as DWORD;

    (*caps).Caps3 = (WINED3DCAPS3_ALPHA_FULLSCREEN_FLIP_OR_DISCARD
        | WINED3DCAPS3_COPY_TO_VIDMEM
        | WINED3DCAPS3_COPY_TO_SYSTEMMEM) as DWORD;

    (*caps).CursorCaps = (WINED3DCURSORCAPS_COLOR
        | WINED3DCURSORCAPS_LOWRES) as DWORD;

    (*caps).DevCaps = (WINED3DDEVCAPS_FLOATTLVERTEX
        | WINED3DDEVCAPS_EXECUTESYSTEMMEMORY
        | WINED3DDEVCAPS_TLVERTEXSYSTEMMEMORY
        | WINED3DDEVCAPS_TLVERTEXVIDEOMEMORY
        | WINED3DDEVCAPS_DRAWPRIMTLVERTEX
        | WINED3DDEVCAPS_HWTRANSFORMANDLIGHT
        | WINED3DDEVCAPS_EXECUTEVIDEOMEMORY
        | WINED3DDEVCAPS_PUREDEVICE
        | WINED3DDEVCAPS_HWRASTERIZATION
        | WINED3DDEVCAPS_TEXTUREVIDEOMEMORY
        | WINED3DDEVCAPS_TEXTURESYSTEMMEMORY
        | WINED3DDEVCAPS_CANRENDERAFTERFLIP
        | WINED3DDEVCAPS_DRAWPRIMITIVES2
        | WINED3DDEVCAPS_DRAWPRIMITIVES2EX) as DWORD;

    (*caps).PrimitiveMiscCaps = (WINED3DPMISCCAPS_CULLNONE
        | WINED3DPMISCCAPS_CULLCCW
        | WINED3DPMISCCAPS_CULLCW
        | WINED3DPMISCCAPS_COLORWRITEENABLE
        | WINED3DPMISCCAPS_CLIPTLVERTS
        | WINED3DPMISCCAPS_CLIPPLANESCALEDPOINTS
        | WINED3DPMISCCAPS_MASKZ
        | WINED3DPMISCCAPS_MRTPOSTPIXELSHADERBLENDING) as DWORD;
    /* TODO:
        WINED3DPMISCCAPS_NULLREFERENCE
        WINED3DPMISCCAPS_FOGANDSPECULARALPHA
        WINED3DPMISCCAPS_FOGVERTEXCLAMPED */

    (*caps).RasterCaps = (WINED3DPRASTERCAPS_DITHER
        | WINED3DPRASTERCAPS_PAT
        | WINED3DPRASTERCAPS_WFOG
        | WINED3DPRASTERCAPS_ZFOG
        | WINED3DPRASTERCAPS_FOGVERTEX
        | WINED3DPRASTERCAPS_FOGTABLE
        | WINED3DPRASTERCAPS_STIPPLE
        | WINED3DPRASTERCAPS_SUBPIXEL
        | WINED3DPRASTERCAPS_ZTEST
        | WINED3DPRASTERCAPS_SCISSORTEST
        | WINED3DPRASTERCAPS_SLOPESCALEDEPTHBIAS
        | WINED3DPRASTERCAPS_DEPTHBIAS) as DWORD;

    (*caps).ZCmpCaps = (WINED3DPCMPCAPS_ALWAYS
        | WINED3DPCMPCAPS_EQUAL
        | WINED3DPCMPCAPS_GREATER
        | WINED3DPCMPCAPS_GREATEREQUAL
        | WINED3DPCMPCAPS_LESS
        | WINED3DPCMPCAPS_LESSEQUAL
        | WINED3DPCMPCAPS_NEVER
        | WINED3DPCMPCAPS_NOTEQUAL) as DWORD;

    /* WINED3DPBLENDCAPS_BOTHINVSRCALPHA and WINED3DPBLENDCAPS_BOTHSRCALPHA
     * are legacy settings for srcblend only. */
    (*caps).SrcBlendCaps = (WINED3DPBLENDCAPS_BOTHINVSRCALPHA
        | WINED3DPBLENDCAPS_BOTHSRCALPHA
        | WINED3DPBLENDCAPS_DESTALPHA
        | WINED3DPBLENDCAPS_DESTCOLOR
        | WINED3DPBLENDCAPS_INVDESTALPHA
        | WINED3DPBLENDCAPS_INVDESTCOLOR
        | WINED3DPBLENDCAPS_INVSRCALPHA
        | WINED3DPBLENDCAPS_INVSRCCOLOR
        | WINED3DPBLENDCAPS_ONE
        | WINED3DPBLENDCAPS_SRCALPHA
        | WINED3DPBLENDCAPS_SRCALPHASAT
        | WINED3DPBLENDCAPS_SRCCOLOR
        | WINED3DPBLENDCAPS_ZERO) as DWORD;

    (*caps).DestBlendCaps = (WINED3DPBLENDCAPS_DESTALPHA
        | WINED3DPBLENDCAPS_DESTCOLOR
        | WINED3DPBLENDCAPS_INVDESTALPHA
        | WINED3DPBLENDCAPS_INVDESTCOLOR
        | WINED3DPBLENDCAPS_INVSRCALPHA
        | WINED3DPBLENDCAPS_INVSRCCOLOR
        | WINED3DPBLENDCAPS_ONE
        | WINED3DPBLENDCAPS_SRCALPHA
        | WINED3DPBLENDCAPS_SRCCOLOR
        | WINED3DPBLENDCAPS_ZERO) as DWORD;

    (*caps).AlphaCmpCaps = (WINED3DPCMPCAPS_ALWAYS
        | WINED3DPCMPCAPS_EQUAL
        | WINED3DPCMPCAPS_GREATER
        | WINED3DPCMPCAPS_GREATEREQUAL
        | WINED3DPCMPCAPS_LESS
        | WINED3DPCMPCAPS_LESSEQUAL
        | WINED3DPCMPCAPS_NEVER
        | WINED3DPCMPCAPS_NOTEQUAL) as DWORD;

    (*caps).ShadeCaps = (WINED3DPSHADECAPS_SPECULARGOURAUDRGB
        | WINED3DPSHADECAPS_COLORGOURAUDRGB
        | WINED3DPSHADECAPS_ALPHAFLATBLEND
        | WINED3DPSHADECAPS_ALPHAGOURAUDBLEND
        | WINED3DPSHADECAPS_COLORFLATRGB
        | WINED3DPSHADECAPS_FOGFLAT
        | WINED3DPSHADECAPS_FOGGOURAUD
        | WINED3DPSHADECAPS_SPECULARFLATRGB) as DWORD;

    (*caps).TextureCaps = (WINED3DPTEXTURECAPS_ALPHA
        | WINED3DPTEXTURECAPS_ALPHAPALETTE
        | WINED3DPTEXTURECAPS_TRANSPARENCY
        | WINED3DPTEXTURECAPS_BORDER
        | WINED3DPTEXTURECAPS_MIPMAP
        | WINED3DPTEXTURECAPS_PROJECTED
        | WINED3DPTEXTURECAPS_PERSPECTIVE) as DWORD;
    if (*d3d_info).unconditional_npot() == 0 {
        (*caps).TextureCaps |=
            (WINED3DPTEXTURECAPS_POW2 | WINED3DPTEXTURECAPS_NONPOW2CONDITIONAL) as DWORD;
    }

    (*caps).TextureFilterCaps = (WINED3DPTFILTERCAPS_MAGFLINEAR
        | WINED3DPTFILTERCAPS_MAGFPOINT
        | WINED3DPTFILTERCAPS_MINFLINEAR
        | WINED3DPTFILTERCAPS_MINFPOINT
        | WINED3DPTFILTERCAPS_MIPFLINEAR
        | WINED3DPTFILTERCAPS_MIPFPOINT
        | WINED3DPTFILTERCAPS_LINEAR
        | WINED3DPTFILTERCAPS_LINEARMIPLINEAR
        | WINED3DPTFILTERCAPS_LINEARMIPNEAREST
        | WINED3DPTFILTERCAPS_MIPLINEAR
        | WINED3DPTFILTERCAPS_MIPNEAREST
        | WINED3DPTFILTERCAPS_NEAREST) as DWORD;

    (*caps).CubeTextureFilterCaps = 0 as DWORD;
    (*caps).VolumeTextureFilterCaps = 0 as DWORD;

    (*caps).TextureAddressCaps = (WINED3DPTADDRESSCAPS_INDEPENDENTUV
        | WINED3DPTADDRESSCAPS_CLAMP
        | WINED3DPTADDRESSCAPS_WRAP) as DWORD;

    (*caps).VolumeTextureAddressCaps = 0 as DWORD;

    (*caps).LineCaps = (WINED3DLINECAPS_TEXTURE
        | WINED3DLINECAPS_ZTEST
        | WINED3DLINECAPS_BLEND
        | WINED3DLINECAPS_ALPHACMP
        | WINED3DLINECAPS_FOG) as DWORD;
    /* WINED3DLINECAPS_ANTIALIAS is not supported on Windows, and dx and gl seem to have a different
     * idea how generating the smoothing alpha values works; the result is different
     */

    (*caps).MaxTextureWidth = (*d3d_info).limits.texture_size as DWORD;
    (*caps).MaxTextureHeight = (*d3d_info).limits.texture_size as DWORD;

    (*caps).MaxVolumeExtent = 0 as DWORD;

    (*caps).MaxTextureRepeat = 32768 as DWORD;
    (*caps).MaxTextureAspectRatio = (*d3d_info).limits.texture_size as DWORD;
    (*caps).MaxVertexW = 1e10f32;

    (*caps).GuardBandLeft = -32768.0f32;
    (*caps).GuardBandTop = -32768.0f32;
    (*caps).GuardBandRight = 32768.0f32;
    (*caps).GuardBandBottom = 32768.0f32;

    (*caps).ExtentsAdjust = 0.0f32;

    (*caps).StencilCaps = (WINED3DSTENCILCAPS_DECRSAT
        | WINED3DSTENCILCAPS_INCRSAT
        | WINED3DSTENCILCAPS_INVERT
        | WINED3DSTENCILCAPS_KEEP
        | WINED3DSTENCILCAPS_REPLACE
        | WINED3DSTENCILCAPS_ZERO) as DWORD;

    (*caps).MaxAnisotropy = 0 as DWORD;
    (*caps).MaxPointSize = (*d3d_info).limits.pointsize_max;

    (*caps).MaxPrimitiveCount = 0x555555 as DWORD; /* Taken from an AMD Radeon HD 5700 (Evergreen) GPU. */
    (*caps).MaxVertexIndex = 0xffffff as DWORD; /* Taken from an AMD Radeon HD 5700 (Evergreen) GPU. */
    (*caps).MaxStreams = WINED3D_MAX_STREAMS as DWORD;
    (*caps).MaxStreamStride = 1024 as DWORD;

    /* d3d9.dll sets D3DDEVCAPS2_CAN_STRETCHRECT_FROM_TEXTURES here because StretchRects is implemented in d3d9 */
    (*caps).DevCaps2 = (WINED3DDEVCAPS2_STREAMOFFSET
        | WINED3DDEVCAPS2_VERTEXELEMENTSCANSHARESTREAMOFFSET) as DWORD;
    (*caps).MaxNpatchTessellationLevel = 0 as ::core::ffi::c_float;

    (*caps).NumSimultaneousRTs = (*d3d_info).limits.max_rt_count as DWORD;

    (*caps).StretchRectFilterCaps = (WINED3DPTFILTERCAPS_MINFPOINT
        | WINED3DPTFILTERCAPS_MAGFPOINT
        | WINED3DPTFILTERCAPS_MINFLINEAR
        | WINED3DPTFILTERCAPS_MAGFLINEAR) as DWORD;
    (*caps).VertexTextureFilterCaps = 0 as DWORD;

    (*(*adapter).shader_backend)
        .shader_get_caps
        .unwrap_unchecked()(adapter, &raw mut shader_caps);
    (*(*adapter).fragment_pipe)
        .get_caps
        .unwrap_unchecked()(adapter, &raw mut fragment_caps);
    (*(*adapter).vertex_pipe)
        .vp_get_caps
        .unwrap_unchecked()(adapter, &raw mut vertex_caps);

    /* Add shader misc caps. Only some of them belong to the shader parts of the pipeline */
    (*caps).PrimitiveMiscCaps |= fragment_caps.PrimitiveMiscCaps;

    (*caps).VertexShaderVersion = shader_caps.vs_version as DWORD;
    (*caps).MaxVertexShaderConst = shader_caps.vs_uniform_count as DWORD;

    (*caps).PixelShaderVersion = shader_caps.ps_version as DWORD;
    (*caps).PixelShader1xMaxValue = shader_caps.ps_1x_max_value;

    (*caps).TextureOpCaps = fragment_caps.TextureOpCaps as DWORD;
    (*caps).MaxTextureBlendStages = fragment_caps.max_blend_stages as DWORD;
    (*caps).MaxSimultaneousTextures = fragment_caps.max_textures as DWORD;

    (*caps).MaxUserClipPlanes = vertex_caps.max_user_clip_planes;
    (*caps).MaxActiveLights = vertex_caps.max_active_lights;
    (*caps).MaxVertexBlendMatrices = vertex_caps.max_vertex_blend_matrices;
    (*caps).MaxVertexBlendMatrixIndex = vertex_caps.max_vertex_blend_matrix_index;
    (*caps).VertexProcessingCaps = vertex_caps.vertex_processing_caps;
    (*caps).FVFCaps = vertex_caps.fvf_caps;
    (*caps).RasterCaps |= vertex_caps.raster_caps;

    /* The following caps are shader specific, but they are things we cannot detect, or which
     * are the same among all shader models. So to avoid code duplication set the shader version
     * specific, but otherwise constant caps here
     */
    if (*caps).VertexShaderVersion >= 3 as DWORD {
        /* Where possible set the caps based on OpenGL extensions and if they
         * aren't set (in case of software rendering) use the VS 3.0 from
         * MSDN or else if there's OpenGL spec use a hardcoded value minimum
         * VS3.0 value. */
        (*caps).VS20Caps.caps = WINED3DVS20CAPS_PREDICATION as DWORD;
        /* VS 3.0 requires MAX_DYNAMICFLOWCONTROLDEPTH (24) */
        (*caps).VS20Caps.dynamic_flow_control_depth = WINED3DVS20_MAX_DYNAMICFLOWCONTROLDEPTH as INT;
        (*caps).VS20Caps.temp_count = 32 as INT;
        /* level of nesting in loops / if-statements; VS 3.0 requires MAX (4) */
        (*caps).VS20Caps.static_flow_control_depth = WINED3DVS20_MAX_STATICFLOWCONTROLDEPTH as INT;

        (*caps).MaxVShaderInstructionsExecuted = 65535 as DWORD; /* VS 3.0 needs at least 65535, some cards even use 2^32-1 */
        (*caps).MaxVertexShader30InstructionSlots = WINED3DMIN30SHADERINSTRUCTIONS as DWORD;
        (*caps).VertexTextureFilterCaps = (WINED3DPTFILTERCAPS_MINFPOINT | WINED3DPTFILTERCAPS_MAGFPOINT) as DWORD;
    } else if (*caps).VertexShaderVersion == 2 as DWORD {
        (*caps).VS20Caps.caps = 0 as DWORD;
        (*caps).VS20Caps.dynamic_flow_control_depth = WINED3DVS20_MIN_DYNAMICFLOWCONTROLDEPTH as INT;
        (*caps).VS20Caps.temp_count = WINED3DVS20_MIN_NUMTEMPS as INT;
        (*caps).VS20Caps.static_flow_control_depth = 1 as INT;

        (*caps).MaxVShaderInstructionsExecuted = 65535 as DWORD;
        (*caps).MaxVertexShader30InstructionSlots = 0 as DWORD;
    } else {
        /* VS 1.x */
        (*caps).VS20Caps.caps = 0 as DWORD;
        (*caps).VS20Caps.dynamic_flow_control_depth = 0 as INT;
        (*caps).VS20Caps.temp_count = 0 as INT;
        (*caps).VS20Caps.static_flow_control_depth = 0 as INT;

        (*caps).MaxVShaderInstructionsExecuted = 0 as DWORD;
        (*caps).MaxVertexShader30InstructionSlots = 0 as DWORD;
    }

    if (*caps).PixelShaderVersion >= 3 as DWORD {
        /* Where possible set the caps based on OpenGL extensions and if they
         * aren't set (in case of software rendering) use the PS 3.0 from
         * MSDN or else if there's OpenGL spec use a hardcoded value minimum
         * PS 3.0 value. */

        /* Caps is more or less undocumented on MSDN but it appears to be
         * used for PS20Caps based on results from R9600/FX5900/Geforce6800
         * cards from Windows */
        (*caps).PS20Caps.caps = (WINED3DPS20CAPS_ARBITRARYSWIZZLE |
                WINED3DPS20CAPS_GRADIENTINSTRUCTIONS |
                WINED3DPS20CAPS_PREDICATION |
                WINED3DPS20CAPS_NODEPENDENTREADLIMIT |
                WINED3DPS20CAPS_NOTEXINSTRUCTIONLIMIT) as DWORD;
        /* PS 3.0 requires MAX_DYNAMICFLOWCONTROLDEPTH (24) */
        (*caps).PS20Caps.dynamic_flow_control_depth = WINED3DPS20_MAX_DYNAMICFLOWCONTROLDEPTH as INT;
        (*caps).PS20Caps.temp_count = 32 as INT;
        /* PS 3.0 requires MAX_STATICFLOWCONTROLDEPTH (4) */
        (*caps).PS20Caps.static_flow_control_depth = WINED3DPS20_MAX_STATICFLOWCONTROLDEPTH as INT;
        /* PS 3.0 requires MAX_NUMINSTRUCTIONSLOTS (512) */
        (*caps).PS20Caps.instruction_slot_count = WINED3DPS20_MAX_NUMINSTRUCTIONSLOTS as INT;

        (*caps).MaxPShaderInstructionsExecuted = 65535 as DWORD;
        (*caps).MaxPixelShader30InstructionSlots = WINED3DMIN30SHADERINSTRUCTIONS as DWORD;
    } else if (*caps).PixelShaderVersion == 2 as DWORD {
        /* Below we assume PS2.0 specs, not extended 2.0a(GeforceFX)/2.0b(Radeon R3xx) ones */
        (*caps).PS20Caps.caps = 0 as DWORD;
        (*caps).PS20Caps.dynamic_flow_control_depth = 0 as INT; /* WINED3DVS20_MIN_DYNAMICFLOWCONTROLDEPTH = 0 */
        (*caps).PS20Caps.temp_count = WINED3DPS20_MIN_NUMTEMPS as INT;
        (*caps).PS20Caps.static_flow_control_depth = WINED3DPS20_MIN_STATICFLOWCONTROLDEPTH as INT; /* Minimum: 1 */
        /* Minimum number (64 ALU + 32 Texture), a GeforceFX uses 512 */
        (*caps).PS20Caps.instruction_slot_count = WINED3DPS20_MIN_NUMINSTRUCTIONSLOTS as INT;

        (*caps).MaxPShaderInstructionsExecuted = 512 as DWORD; /* Minimum value, a GeforceFX uses 1024 */
        (*caps).MaxPixelShader30InstructionSlots = 0 as DWORD;
    } else {
        /* PS 1.x */
        (*caps).PS20Caps.caps = 0 as DWORD;
        (*caps).PS20Caps.dynamic_flow_control_depth = 0 as INT;
        (*caps).PS20Caps.temp_count = 0 as INT;
        (*caps).PS20Caps.static_flow_control_depth = 0 as INT;
        (*caps).PS20Caps.instruction_slot_count = 0 as INT;

        (*caps).MaxPShaderInstructionsExecuted = 0 as DWORD;
        (*caps).MaxPixelShader30InstructionSlots = 0 as DWORD;
    }

    if (*caps).VertexShaderVersion >= 2 as DWORD {
        /* OpenGL supports all the formats below, perhaps not always
         * without conversion, but it supports them.
         * Further GLSL doesn't seem to have an official unsigned type so
         * don't advertise it yet as I'm not sure how we handle it.
         * We might need to add some clamping in the shader engine to
         * support it.
         * TODO: WINED3DDTCAPS_USHORT2N, WINED3DDTCAPS_USHORT4N, WINED3DDTCAPS_UDEC3, WINED3DDTCAPS_DEC3N */
        (*caps).DeclTypes = (WINED3DDTCAPS_UBYTE4
            | WINED3DDTCAPS_UBYTE4N
            | WINED3DDTCAPS_SHORT2N
            | WINED3DDTCAPS_SHORT4N) as DWORD;
    } else {
        (*caps).DeclTypes = 0 as DWORD;
    }

    /* Set DirectDraw helper Caps */
    ckey_caps = (WINEDDCKEYCAPS_DESTBLT
        | WINEDDCKEYCAPS_SRCBLT) as DWORD;
    fx_caps = (WINEDDFXCAPS_BLTALPHA
        | WINEDDFXCAPS_BLTMIRRORLEFTRIGHT
        | WINEDDFXCAPS_BLTMIRRORUPDOWN
        | WINEDDFXCAPS_BLTROTATION90
        | WINEDDFXCAPS_BLTSHRINKX
        | WINEDDFXCAPS_BLTSHRINKXN
        | WINEDDFXCAPS_BLTSHRINKY
        | WINEDDFXCAPS_BLTSHRINKYN
        | WINEDDFXCAPS_BLTSTRETCHX
        | WINEDDFXCAPS_BLTSTRETCHXN
        | WINEDDFXCAPS_BLTSTRETCHY
        | WINEDDFXCAPS_BLTSTRETCHYN) as DWORD;
    blit_caps = (WINEDDCAPS_BLT
        | WINEDDCAPS_BLTCOLORFILL
        | WINEDDCAPS_BLTDEPTHFILL
        | WINEDDCAPS_BLTSTRETCH
        | WINEDDCAPS_CANBLTSYSMEM
        | WINEDDCAPS_CANCLIP
        | WINEDDCAPS_CANCLIPSTRETCHED
        | WINEDDCAPS_COLORKEY
        | WINEDDCAPS_COLORKEYHWASSIST) as DWORD;

    /* Fill the ddraw caps structure */
    (*caps).ddraw_caps.caps = (WINEDDCAPS_GDI
        | WINEDDCAPS_PALETTE
        | blit_caps) as DWORD;
    (*caps).ddraw_caps.caps2 = (WINEDDCAPS2_CERTIFIED
        | WINEDDCAPS2_NOPAGELOCKREQUIRED
        | WINEDDCAPS2_PRIMARYGAMMA
        | WINEDDCAPS2_WIDESURFACES
        | WINEDDCAPS2_CANRENDERWINDOWED) as DWORD;
    (*caps).ddraw_caps.color_key_caps = ckey_caps;
    (*caps).ddraw_caps.fx_caps = fx_caps;
    (*caps).ddraw_caps.svb_caps = blit_caps;
    (*caps).ddraw_caps.svb_color_key_caps = ckey_caps;
    (*caps).ddraw_caps.svb_fx_caps = fx_caps;
    (*caps).ddraw_caps.vsb_caps = blit_caps;
    (*caps).ddraw_caps.vsb_color_key_caps = ckey_caps;
    (*caps).ddraw_caps.vsb_fx_caps = fx_caps;
    (*caps).ddraw_caps.ssb_caps = blit_caps;
    (*caps).ddraw_caps.ssb_color_key_caps = ckey_caps;
    (*caps).ddraw_caps.ssb_fx_caps = fx_caps;

    (*caps).ddraw_caps.dds_caps = (WINEDDSCAPS_FLIP
        | WINEDDSCAPS_OFFSCREENPLAIN
        | WINEDDSCAPS_PALETTE
        | WINEDDSCAPS_PRIMARYSURFACE
        | WINEDDSCAPS_TEXTURE
        | WINEDDSCAPS_ZBUFFER
        | WINEDDSCAPS_MIPMAP) as DWORD;

    (*caps).shader_double_precision = (*d3d_info).shader_double_precision() as BOOL;
    (*caps).viewport_array_index_any_shader = (*d3d_info).viewport_array_index_any_shader() as BOOL;
    (*caps).stencil_export = (*d3d_info).stencil_export() as BOOL;

    (*caps).max_feature_level = (*d3d_info).feature_level;

    (*(*adapter).adapter_ops)
        .adapter_get_wined3d_caps
        .unwrap_unchecked()(adapter, caps);

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_device_create(
    wined3d: *mut wined3d,
    adapter: *mut wined3d_adapter,
    device_type: wined3d_device_type,
    focus_window: HWND,
    flags: uint32_t,
    surface_alignment: BYTE,
    feature_levels: *const wined3d_feature_level,
    feature_level_count: ::core::ffi::c_uint,
    device_parent: *mut wined3d_device_parent,
    device: *mut *mut wined3d_device,
) -> HRESULT {
    let mut object: *mut wined3d_device = ::core::ptr::null_mut::<wined3d_device>();
    let mut hr: HRESULT = 0;
    WINE_TRACE!("wined3d %p, adapter %p, device_type %#x, focus_window %p, flags %#x, surface_alignment %u, feature_levels %p, feature_level_count %u, device_parent %p, device %p.\n", wined3d, adapter, device_type as ::core::ffi::c_uint, focus_window, flags, surface_alignment as ::core::ffi::c_int, feature_levels, feature_level_count, device_parent, device);
    hr = (*(*adapter).adapter_ops)
        .adapter_create_device
        .unwrap_unchecked()(
        wined3d,
        adapter,
        device_type,
        focus_window,
        flags as ::core::ffi::c_uint,
        surface_alignment,
        feature_levels,
        feature_level_count,
        device_parent,
        core::ptr::addr_of_mut!(object),
    );
    if hr < 0 {
        return hr;
    }
    WINE_TRACE!("Created device %p.\n", object);
    *device = object;
    (*(*device_parent).ops)
        .wined3d_device_created
        .unwrap_unchecked()(device_parent, *device);
    return WINED3D_OK;
}

unsafe extern "C" fn adapter_no3d_destroy(adapter: *mut wined3d_adapter) {
    wined3d_adapter_cleanup(adapter);
    free((adapter).cast());
}

unsafe extern "C" fn adapter_no3d_create_device(
    wined3d: *mut wined3d,
    adapter: *const wined3d_adapter,
    device_type: wined3d_device_type,
    focus_window: HWND,
    flags: u32,
    surface_alignment: BYTE,
    levels: *const wined3d_feature_level,
    level_count: u32,
    device_parent: *mut wined3d_device_parent,
    device: *mut *mut wined3d_device,
) -> HRESULT {
    /* No extensions in the state table, only extension 0, which is implicitly supported. */
    static mut supported_extensions: [BOOL; 1] = [(TRUE as i32)];
    let mut device_no3d: *mut wined3d_device_no3d = core::ptr::null_mut();
    let mut hr: HRESULT;

    device_no3d = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<wined3d_device_no3d>() as size_t) as _,
    ) as *mut wined3d_device_no3d;
    if device_no3d.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_device_init(
        core::ptr::addr_of_mut!((*device_no3d).d),
        wined3d,
        (*adapter).ordinal,
        device_type,
        focus_window,
        flags,
        surface_alignment,
        levels,
        level_count,
        core::ptr::addr_of!(supported_extensions) as *const BOOL,
        device_parent,
    );
    if hr < 0 {
        WARN!("Failed to initialize device, hr %#lx.\n", hr);
        free(device_no3d as *mut core::ffi::c_void);
        return hr;
    }

    *device = core::ptr::addr_of_mut!((*device_no3d).d);

    return WINED3D_OK;
}

unsafe extern "C" fn adapter_no3d_destroy_device(device: *mut wined3d_device) {
    wined3d_device_cleanup(device);
    free((device).cast());
}

unsafe extern "C" fn adapter_no3d_acquire_context(
    device: *mut wined3d_device,
    texture: *mut wined3d_texture,
    sub_resource_idx: ::core::ffi::c_uint,
) -> *mut wined3d_context {
    wined3d_from_cs((*device).cs);

    if (*device).context_count == 0 {
        return core::ptr::null_mut();
    }

    return core::ptr::addr_of_mut!((*wined3d_device_no3d(device)).context_no3d);
}

unsafe extern "C" fn adapter_no3d_release_context(context: *mut wined3d_context) {
    // TRACE("context %p.\n", context);
}

unsafe extern "C" fn adapter_no3d_get_wined3d_caps(
    adapter: *const wined3d_adapter,
    caps: *mut wined3d_caps,
) {}

unsafe extern "C" fn adapter_no3d_check_format(
    adapter: *const wined3d_adapter,
    adapter_format: *const wined3d_format,
    rt_format: *const wined3d_format,
    ds_format: *const wined3d_format,
) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn adapter_no3d_init_3d(mut device: *mut wined3d_device) -> HRESULT {
    let mut context_no3d: *mut wined3d_context = core::ptr::null_mut();
    let mut hr: HRESULT;

    context_no3d = core::ptr::addr_of_mut!((*wined3d_device_no3d(device)).context_no3d);
    hr = wined3d_context_no3d_init(
        context_no3d,
        *(*device).swapchains.add(0 as usize),
    );
    if hr < 0 {
        WARN!("Failed to initialise context.\n");
        return hr;
    }

    if device_context_add(device, context_no3d) == 0 {
        ERR!("Failed to add the newly created context to the context list.\n");
        wined3d_context_cleanup(context_no3d);
        return E_FAIL;
    }

    (*device).blitter = wined3d_cpu_blitter_create();
    if (*device).blitter.is_null() {
        ERR!("Failed to create CPU blitter.\n");
        device_context_remove(device, context_no3d);
        wined3d_context_cleanup(context_no3d);
        return E_FAIL;
    }

    return WINED3D_OK;
}

unsafe extern "C" fn adapter_no3d_uninit_3d(mut device: *mut wined3d_device) {
    let mut context_no3d: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    WINE_TRACE!("device %p.\n", device);
    context_no3d = &raw mut (*wined3d_device_no3d(device)).context_no3d;
    (*(*(*device).blitter).ops)
        .blitter_destroy
        .unwrap_unchecked()(
        (*device).blitter,
        core::ptr::null_mut::<wined3d_context>(),
    );
    wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);
    device_context_remove(device, context_no3d);
    wined3d_context_cleanup(context_no3d);
}

unsafe extern "C" fn adapter_no3d_map_bo_address(
    context: *mut wined3d_context,
    data: *const wined3d_bo_address,
    size: size_t,
    map_flags: uint32_t,
) -> *mut core::ffi::c_void {
    if !(*data).buffer_object.is_null() {
        ERR!("Unsupported buffer object %p.\n", (*data).buffer_object);
        return core::ptr::null_mut();
    }

    (*data).addr as *mut core::ffi::c_void
}

unsafe extern "C" fn adapter_no3d_unmap_bo_address(
    context: *mut wined3d_context,
    data: *const wined3d_bo_address,
    range_count: ::core::ffi::c_uint,
    ranges: *const wined3d_range,
) {
    if !(*data).buffer_object.is_null() {
        ERR!("Unsupported buffer object %p.\n", (*data).buffer_object);
    }
}

unsafe extern "C" fn adapter_no3d_copy_bo_address(
    context: *mut wined3d_context,
    dst: *const wined3d_bo_address,
    src: *const wined3d_bo_address,
    range_count: core::ffi::c_uint,
    ranges: *const wined3d_range,
    map_flags: u32,
) {
    let mut i: core::ffi::c_uint = 0;

    if !(*dst).buffer_object.is_null() {
        ERR!("Unsupported dst buffer object %p.\n", (*dst).buffer_object);
    }
    if !(*src).buffer_object.is_null() {
        ERR!("Unsupported src buffer object %p.\n", (*src).buffer_object);
    }
    if !(*dst).buffer_object.is_null() || !(*src).buffer_object.is_null() {
        return;
    }

    while i < range_count {
        memcpy(
            (*dst).addr.offset((*ranges.offset(i as isize)).offset as isize) as *mut core::ffi::c_void,
            (*src).addr.offset((*ranges.offset(i as isize)).offset as isize) as *const core::ffi::c_void,
            (*ranges.offset(i as isize)).size as usize,
        );
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn adapter_no3d_flush_bo_address(
    context: *mut wined3d_context,
    data: *const wined3d_const_bo_address,
    size: size_t,
) {}

unsafe extern "C" fn adapter_no3d_alloc_bo(
    device: *mut wined3d_device,
    resource: *mut wined3d_resource,
    sub_resource_idx: ::core::ffi::c_uint,
    addr: *mut wined3d_bo_address,
) -> bool {
    return false;
}

unsafe extern "C" fn adapter_no3d_destroy_bo(
    context: *mut wined3d_context,
    bo: *mut wined3d_bo,
) {}

unsafe extern "C" fn adapter_no3d_create_swapchain(
    device: *mut wined3d_device,
    desc: *const wined3d_swapchain_desc,
    state_parent: *mut wined3d_swapchain_state_parent,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    swapchain: *mut *mut wined3d_swapchain,
) -> HRESULT {
    let mut swapchain_no3d: *mut wined3d_swapchain = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    swapchain_no3d = calloc(
        1,
        (core::mem::size_of::<wined3d_swapchain>()) as _,
    ) as *mut wined3d_swapchain;
    if swapchain_no3d.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_swapchain_no3d_init(
        swapchain_no3d,
        device,
        desc,
        state_parent,
        parent,
        parent_ops,
    );
    if hr < 0 {
        WARN!("Failed to initialise swapchain, hr %#lx.\n", hr);
        free(swapchain_no3d as *mut core::ffi::c_void);
        return hr;
    }

    *swapchain = swapchain_no3d;

    return hr;
}

unsafe extern "C" fn adapter_no3d_destroy_swapchain(swapchain: *mut wined3d_swapchain) {
    wined3d_swapchain_cleanup(swapchain);
    free(swapchain as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn adapter_no3d_create_buffer(
    device: *mut wined3d_device,
    desc: *const wined3d_buffer_desc,
    data: *const wined3d_sub_resource_data,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    buffer: *mut *mut wined3d_buffer,
) -> HRESULT {
    let mut buffer_no3d: *mut wined3d_buffer = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    buffer_no3d = calloc(
        1,
        (core::mem::size_of::<wined3d_buffer>() as size_t) as _,
    ) as *mut wined3d_buffer;
    if buffer_no3d.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_buffer_no3d_init(buffer_no3d, device, desc, data, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise buffer, hr %#lx.\n", hr);
        free(buffer_no3d as *mut core::ffi::c_void);
        return hr;
    }

    *buffer = buffer_no3d;

    return hr;
}

unsafe extern "C" fn adapter_no3d_destroy_buffer(mut buffer: *mut wined3d_buffer) {
    let mut device: *mut wined3d_device = (*buffer).resource.device;
    let mut swapchain_count: ::core::ffi::c_uint = (*device).swapchain_count as ::core::ffi::c_uint;
    WINE_TRACE!("buffer %p.\n", buffer);
    if swapchain_count != 0 {
        wined3d_device_incref(device);
    }
    wined3d_buffer_cleanup(buffer);
    wined3d_cs_destroy_object(
        (*device).cs as *mut wined3d_cs,
        Some(free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        buffer as *mut ::core::ffi::c_void,
    );
    if swapchain_count != 0 {
        wined3d_device_decref(device);
    }
}

unsafe extern "C" fn adapter_no3d_create_texture(device: *mut wined3d_device, desc: *const wined3d_resource_desc, layer_count: u32, level_count: u32, flags: u32, parent: *mut core::ffi::c_void, parent_ops: *const wined3d_parent_ops, texture: *mut *mut wined3d_texture) -> HRESULT {
    let mut texture_no3d: *mut wined3d_texture;
    let mut hr: HRESULT = 0;

    texture_no3d = wined3d_texture_allocate_object_memory(core::mem::size_of::<wined3d_texture>() as SIZE_T, level_count as SIZE_T, layer_count as SIZE_T) as *mut wined3d_texture;
    if texture_no3d.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_texture_no3d_init(texture_no3d, device, desc, layer_count, level_count, flags, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise texture, hr %#lx.\n", hr);
        free(texture_no3d as *mut core::ffi::c_void);
        return hr;
    }

    *texture = texture_no3d;

    return hr;
}

unsafe extern "C" fn adapter_no3d_destroy_texture(texture: *mut wined3d_texture) {
    let device: *mut wined3d_device = (*texture).resource.device;
    let swapchain_count: core::ffi::c_uint = (*device).swapchain_count as core::ffi::c_uint;

    if swapchain_count != 0 {
        wined3d_device_incref(device);
    }

    wined3d_texture_sub_resources_destroyed(texture);
    (*(*texture).resource.parent_ops)
        .wined3d_object_destroyed
        .unwrap_unchecked()((*texture).resource.parent);

    wined3d_texture_cleanup(texture);
    wined3d_cs_destroy_object(
        (*device).cs as *mut wined3d_cs,
        Some(free as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()),
        texture as *mut core::ffi::c_void,
    );

    if swapchain_count != 0 {
        wined3d_device_decref(device);
    }
}

unsafe extern "C" fn adapter_no3d_create_rendertarget_view(
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    view: *mut *mut wined3d_rendertarget_view,
) -> HRESULT {
    let mut view_no3d: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!(
        "desc %s, resource %p, parent %p, parent_ops %p, view %p.\n",
        wined3d_debug_view_desc(desc, resource),
        resource,
        parent,
        parent_ops,
        view
    );

    view_no3d = calloc(
        1,
        (core::mem::size_of::<wined3d_rendertarget_view>()) as _,
    ) as *mut wined3d_rendertarget_view;
    if view_no3d.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_rendertarget_view_no3d_init(view_no3d, desc, resource, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialise view, hr %#lx.\n", hr);
        free(view_no3d as *mut core::ffi::c_void);
        return hr;
    }

    TRACE!("Created render target view %p.\n", view_no3d);

    *view = view_no3d;

    return hr;
}

unsafe extern "C" fn adapter_no3d_destroy_rendertarget_view(view: *mut wined3d_rendertarget_view) {
    let mut device: *mut wined3d_device = (*(*view).resource).device;
    let mut swapchain_count: ::core::ffi::c_uint = (*device).swapchain_count as ::core::ffi::c_uint;
    WINE_TRACE!("view %p.\n", view);
    if swapchain_count != 0 {
        wined3d_device_incref(device);
    }
    wined3d_rendertarget_view_cleanup(view);
    wined3d_cs_destroy_object(
        (*device).cs as *mut wined3d_cs,
        Some(free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        view as *mut ::core::ffi::c_void,
    );
    if swapchain_count != 0 {
        wined3d_device_decref(device);
    }
}

unsafe extern "C" fn adapter_no3d_create_shader_resource_view(
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    parent: *mut ::core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    view: *mut *mut wined3d_shader_resource_view,
) -> HRESULT {
    WINE_TRACE!("desc %s, resource %p, parent %p, parent_ops %p, view %p.\n", wined3d_debug_view_desc(desc, resource), resource, parent, parent_ops, view);
    return E_NOTIMPL;
}

unsafe extern "C" fn adapter_no3d_destroy_shader_resource_view(
    view: *mut wined3d_shader_resource_view,
) {
    // TRACE("view %p.\n", view);
}

unsafe extern "C" fn adapter_no3d_create_unordered_access_view(
    desc: *const wined3d_view_desc,
    resource: *mut wined3d_resource,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    view: *mut *mut wined3d_unordered_access_view,
) -> HRESULT {
    TRACE!(
        "desc %s, resource %p, parent %p, parent_ops %p, view %p.\n",
        wined3d_debug_view_desc(desc, resource),
        resource,
        parent,
        parent_ops,
        view
    );

    return E_NOTIMPL;
}

unsafe extern "C" fn adapter_no3d_destroy_unordered_access_view(
    view: *mut wined3d_unordered_access_view,
) {
    // TRACE("view %p.\n", view);
}

unsafe extern "C" fn adapter_no3d_create_sampler(
    device: *mut wined3d_device,
    desc: *const wined3d_sampler_desc,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    sampler: *mut *mut wined3d_sampler,
) -> HRESULT {
    // TRACE("device %p, desc %p, parent %p, parent_ops %p, sampler %p.\n", device, desc, parent, parent_ops, sampler);
    E_NOTIMPL
}

unsafe extern "C" fn adapter_no3d_destroy_sampler(sampler: *mut wined3d_sampler) {
    // TRACE("sampler %p.\n", sampler);
}

unsafe extern "C" fn adapter_no3d_create_query(
    device: *mut wined3d_device,
    r#type: wined3d_query_type,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    query: *mut *mut wined3d_query,
) -> HRESULT {
    // TRACE("device %p, type %#x, parent %p, parent_ops %p, query %p.\n", device, type, parent, parent_ops, query);
    WINED3DERR_NOTAVAILABLE
}

unsafe extern "C" fn adapter_no3d_destroy_query(query: *mut wined3d_query) {
    // TRACE("query %p.\n", query);
}

unsafe extern "C" fn adapter_no3d_flush_context(context: *mut wined3d_context) {
    // TRACE("context %p.\n", context);
}

unsafe extern "C" fn adapter_no3d_draw_primitive(
    device: *mut wined3d_device,
    state: *const wined3d_state,
    parameters: *const wined3d_draw_parameters,
) {
    ERR!("device %p, state %p, parameters %p.\n", device, state, parameters);
}

unsafe extern "C" fn adapter_no3d_dispatch_compute(
    device: *mut wined3d_device,
    state: *const wined3d_state,
    parameters: *const wined3d_dispatch_parameters,
) {
    ERR!("device %p, state %p, parameters %p.\n", device, state, parameters);
}

unsafe extern "C" fn adapter_no3d_clear_uav(context: *mut wined3d_context, view: *mut wined3d_unordered_access_view, clear_value: *const wined3d_uvec4, fp: bool) {
    ERR!("context %p, view %p, clear_value %s, fp %#x.\n", context, view, debug_uvec4(clear_value), fp as i32);
}

unsafe extern "C" fn wined3d_adapter_no3d_init_d3d_info(adapter: *mut wined3d_adapter, wined3d_creation_flags: u32) {
    let d3d_info: *mut wined3d_d3d_info = core::ptr::addr_of_mut!((*adapter).d3d_info);

    (*d3d_info).wined3d_creation_flags = wined3d_creation_flags;
    (*d3d_info).set_unconditional_npot((true) as _);
    (*d3d_info).feature_level = WINED3D_FEATURE_LEVEL_5;
}

unsafe extern "C" fn wined3d_adapter_no3d_create(
    mut ordinal: ::core::ffi::c_uint,
    mut wined3d_creation_flags: ::core::ffi::c_uint,
) -> *mut wined3d_adapter {
    let mut adapter: *mut wined3d_adapter = ::core::ptr::null_mut::<wined3d_adapter>();
    let mut primary_luid: LUID = LUID {
        LowPart: 0,
        HighPart: 0,
    };
    let mut luid: *mut LUID = ::core::ptr::null_mut::<LUID>();
    static mut gpu_description: wined3d_gpu_description = wined3d_gpu_description {
        vendor: HW_VENDOR_SOFTWARE,
        device: CARD_WINE,
        description: b"WineD3D DirectDraw Emulation\0" as *const u8 as *const ::core::ffi::c_char,
        driver: DRIVER_WINE,
        vidmem: 128,
    };
    WINE_TRACE!("ordinal %u, wined3d_creation_flags %#x.\n", ordinal, wined3d_creation_flags);
    adapter = calloc(
        1,
        (::core::mem::size_of::<wined3d_adapter>() as size_t) as _,
    ) as *mut wined3d_adapter;
    if adapter.is_null() {
        return ::core::ptr::null_mut::<wined3d_adapter>();
    }
    if ordinal == 0
        && wined3d_get_primary_adapter_luid(&raw mut primary_luid) != 0
    {
        luid = &raw mut primary_luid;
    }
    if wined3d_adapter_init(adapter, ordinal, luid, &raw const wined3d_adapter_no3d_ops) == 0 {
        free(adapter as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<wined3d_adapter>();
    }
    if wined3d_adapter_no3d_init_format_info(adapter) == 0 {
        wined3d_adapter_cleanup(adapter);
        free(adapter as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<wined3d_adapter>();
    }
    if !wined3d_driver_info_init(
        &raw mut (*adapter).driver_info,
        &raw const gpu_description,
        WINED3D_FEATURE_LEVEL_NONE,
        0,
        0,
    ) {
        wined3d_adapter_cleanup(adapter);
        free(adapter as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<wined3d_adapter>();
    }
    (*adapter).vram_bytes_used = 0;
    WINE_TRACE!("Emulating 0x%s bytes of video ram.\n", wine_dbgstr_longlong((*adapter).driver_info.vram_bytes as ULONGLONG));
    (*adapter).vertex_pipe = &raw const none_vertex_pipe;
    (*adapter).fragment_pipe = &raw const none_fragment_pipe;
    (*adapter).misc_state_template =
        &raw const misc_state_template_no3d as *const wined3d_state_entry_template;
    (*adapter).shader_backend = &raw const none_shader_backend;
    wined3d_adapter_no3d_init_d3d_info(adapter, wined3d_creation_flags);
    WINE_TRACE!("Created adapter %p.\n", adapter);
    return adapter;
}

unsafe extern "C" fn wined3d_adapter_create_output(
    adapter: *mut wined3d_adapter,
    output_name: *const WCHAR,
) -> BOOL {
    let mut hr: HRESULT;

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*adapter).outputs) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*adapter).outputs_size),
        (*adapter).output_count + 1,
        core::mem::size_of::<wined3d_output>() as SIZE_T,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    hr = wined3d_output_init(
        (*adapter).outputs.add((*adapter).output_count as usize),
        ((*adapter).output_count) as u32,
        adapter,
        output_name,
    );
    if hr < 0 {
        ERR!(
            "Failed to initialise output %s, hr %#lx.\n",
            wine_dbgstr_w(output_name),
            hr
        );
        return (FALSE) as BOOL;
    }

    (*adapter).output_count += 1;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_init(adapter: *mut wined3d_adapter, ordinal: ::core::ffi::c_uint, luid: *const LUID, adapter_ops: *const wined3d_adapter_ops) -> BOOL {
    let mut open_adapter_desc: D3DKMT_OPENADAPTERFROMLUID = core::mem::zeroed();
    let mut output_idx: ::core::ffi::c_uint = 0;
    let mut primary_idx: ::core::ffi::c_uint = 0;
    let mut close_adapter_desc: D3DKMT_CLOSEADAPTER = core::mem::zeroed();
    let mut display_device: DISPLAY_DEVICEW = core::mem::zeroed();
    let mut ret: BOOL = (FALSE as i32);

    (*adapter).ordinal = ordinal;
    (*adapter).output_count = 0;
    (*adapter).outputs = core::ptr::null_mut();

    if !luid.is_null() {
        (*adapter).luid = *luid;
    } else {
        WARN!("Allocating a random LUID.\n");
        if AllocateLocallyUniqueId(&raw mut (*adapter).luid) == 0 {
            ERR!(
                "Failed to allocate a LUID, error %#lx.\n",
                GetLastError()
            );
            return (FALSE) as BOOL;
        }
    }

    TRACE!(
        "adapter %p LUID %08lx:%08lx.\n",
        adapter,
        (*adapter).luid.HighPart,
        (*adapter).luid.LowPart
    );

    open_adapter_desc.AdapterLuid = (*adapter).luid;
    if D3DKMTOpenAdapterFromLuid(&raw mut open_adapter_desc) != 0 {
        return (FALSE) as BOOL;
    }
    (*adapter).kmt_adapter = open_adapter_desc.hAdapter;

    display_device.cb = core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD;

    'done: {
        loop {
            let fresh28 = output_idx;
            output_idx = output_idx.wrapping_add(1);
            if !(EnumDisplayDevicesW(
                core::ptr::null::<WCHAR>(),
                fresh28,
                &raw mut display_device,
                0,
            ) != 0)
            {
                break;
            }

            /* Detached outputs are not enumerated */
            if (display_device.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) == 0 {
                continue;
            }

            if (display_device.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0 {
                primary_idx = (*adapter).output_count as ::core::ffi::c_uint;
            }

            if wined3d_adapter_create_output(
                adapter,
                &raw mut display_device.DeviceName as *mut WCHAR,
            ) == 0
            {
                break 'done;
            }
        }

        TRACE!(
            "Initialised %Iu outputs for adapter %p.\n",
            (*adapter).output_count,
            adapter
        );

        /* Make the primary output first */
        if primary_idx != 0 {
            let mut tmp: wined3d_output =
                *(*adapter).outputs.offset(0 as isize);
            *(*adapter).outputs.offset(0 as isize) =
                *(*adapter).outputs.offset(primary_idx as isize);
            (*(*adapter).outputs.offset(0 as isize)).ordinal =
                0;
            *(*adapter).outputs.offset(primary_idx as isize) = tmp;
            (*(*adapter).outputs.offset(primary_idx as isize)).ordinal = primary_idx;
        }

        memset(
            &raw mut (*adapter).driver_uuid as *mut ::core::ffi::c_void,
            0,
            core::mem::size_of::<GUID>() as size_t,
        );
        memset(
            &raw mut (*adapter).device_uuid as *mut ::core::ffi::c_void,
            0,
            core::mem::size_of::<GUID>() as size_t,
        );

        (*adapter).formats = core::ptr::null_mut();
        (*adapter).adapter_ops = adapter_ops;
        ret = (TRUE as i32);
    }

    if ret == 0 {
        output_idx = 0;
        while (output_idx as size_t) < ((*adapter).output_count) as _ {
            wined3d_output_cleanup((*adapter).outputs.offset(output_idx as isize));
            output_idx = output_idx.wrapping_add(1);
        }
        free((*adapter).outputs as *mut ::core::ffi::c_void);
        close_adapter_desc.hAdapter = (*adapter).kmt_adapter;
        D3DKMTCloseAdapter(&raw mut close_adapter_desc);
        /* Leave nothing for wined3d_adapter_gl_create()'s cleanup to release twice. */
        (*adapter).outputs = core::ptr::null_mut();
        (*adapter).output_count = 0;
        (*adapter).kmt_adapter = 0;
    }
    return ret;
}

unsafe extern "C" fn wined3d_adapter_create(
    mut ordinal: ::core::ffi::c_uint,
    mut wined3d_creation_flags: DWORD,
) -> *mut wined3d_adapter {
    if wined3d_creation_flags & (WINED3D_NO3D as DWORD) != 0 {
        return wined3d_adapter_no3d_create(ordinal, wined3d_creation_flags);
    }
    if wined3d_settings.renderer as ::core::ffi::c_uint
        == WINED3D_RENDERER_VULKAN as ::core::ffi::c_uint
    {
        return wined3d_adapter_vk_create(ordinal, wined3d_creation_flags);
    }
    return wined3d_adapter_gl_create(ordinal, wined3d_creation_flags);
}
unsafe extern "system" fn wined3d_null_wined3d_object_destroyed(
    parent: *mut core::ffi::c_void,
) {}
#[no_mangle]
pub unsafe extern "C" fn wined3d_init(wined3d: *mut wined3d, flags: uint32_t) -> HRESULT {
    (*wined3d).ref_ = 1 as LONG;
    (*wined3d).flags = flags as ::core::ffi::c_uint;

    TRACE!("Initialising adapters.\n");

    let adapter = wined3d_adapter_create(0 as ::core::ffi::c_uint, flags as DWORD);
    (*(*wined3d).adapters.as_mut_ptr().add(0 as usize)) = adapter;
    if adapter.is_null() {
        WARN!("Failed to create adapter.\n");
        return E_FAIL;
    }
    (*wined3d).adapter_count = 1 as ::core::ffi::c_uint;

    return WINED3D_OK;
}
