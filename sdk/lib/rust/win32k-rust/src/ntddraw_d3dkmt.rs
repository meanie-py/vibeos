use win32k_sys::win32k::*;
use ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
use ntkrnl_sys::base::STATUS_SUCCESS;

#[cfg(target_arch = "x86")]
extern "fastcall" { fn BitmapFormat(cBits: u32, iCompression: u32) -> u32; }
#[cfg(not(target_arch = "x86"))]
extern "C" { fn BitmapFormat(cBits: u32, iCompression: u32) -> u32; }

#[inline(always)]
unsafe fn GDI_HANDLE_GET_TYPE(h: HGDIOBJ) -> u32 { ((h as usize) & 0x007f0000) as u32 }

#[inline(always)]
unsafe fn PALETTE_ShareUnlockPalette(p: PPALETTE) { GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*p).BaseObject)); }

#[inline(always)]
unsafe fn SURFACE_UnlockSurface(p: PSURFACE) { GDIOBJ_vUnlockObject(p as POBJ); }
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdDDICreateDCFromMemory(desc: *mut D3DKMT_CREATEDCFROMMEMORY) -> u32 {
    struct d3dddi_format_info {
        format: D3DDDIFORMAT,
        bit_count: u32,
        compression: u32,
        palette_size: u32,
        mask_r: u32,
        mask_g: u32,
        mask_b: u32,
    }

    let format_info: [d3dddi_format_info; 9] = [
        d3dddi_format_info { format: D3DDDIFMT_R8G8B8, bit_count: 24, compression: BI_RGB, palette_size: 0, mask_r: 0x00000000, mask_g: 0x00000000, mask_b: 0x00000000 },
        d3dddi_format_info { format: D3DDDIFMT_A8R8G8B8, bit_count: 32, compression: BI_RGB, palette_size: 0, mask_r: 0x00000000, mask_g: 0x00000000, mask_b: 0x00000000 },
        d3dddi_format_info { format: D3DDDIFMT_X8R8G8B8, bit_count: 32, compression: BI_RGB, palette_size: 0, mask_r: 0x00000000, mask_g: 0x00000000, mask_b: 0x00000000 },
        d3dddi_format_info { format: D3DDDIFMT_R5G6B5, bit_count: 16, compression: BI_BITFIELDS, palette_size: 0, mask_r: 0x0000f800, mask_g: 0x000007e0, mask_b: 0x0000001f },
        d3dddi_format_info { format: D3DDDIFMT_X1R5G5B5, bit_count: 16, compression: BI_BITFIELDS, palette_size: 0, mask_r: 0x00007c00, mask_g: 0x000003e0, mask_b: 0x0000001f },
        d3dddi_format_info { format: D3DDDIFMT_A1R5G5B5, bit_count: 16, compression: BI_BITFIELDS, palette_size: 0, mask_r: 0x00007c00, mask_g: 0x000003e0, mask_b: 0x0000001f },
        d3dddi_format_info { format: D3DDDIFMT_A4R4G4B4, bit_count: 16, compression: BI_BITFIELDS, palette_size: 0, mask_r: 0x00000f00, mask_g: 0x000000f0, mask_b: 0x0000000f },
        d3dddi_format_info { format: D3DDDIFMT_X4R4G4B4, bit_count: 16, compression: BI_BITFIELDS, palette_size: 0, mask_r: 0x00000f00, mask_g: 0x000000f0, mask_b: 0x0000000f },
        d3dddi_format_info { format: D3DDDIFMT_P8, bit_count: 8, compression: BI_RGB, palette_size: 256, mask_r: 0x00000000, mask_g: 0x00000000, mask_b: 0x00000000 },
    ];

    if desc.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    if (*desc).pMemory.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    let mut format: *const d3dddi_format_info = core::ptr::null();

    let mut i: u32 = 0;
    while (i as usize) < format_info.len() {
        if format_info[i as usize].format == (*desc).Format {
            format = &format_info[i as usize];
            break;
        }
        i += 1;
    }

    if format.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    if (*desc).Width > (UINT_MAX & !3) / ((*format).bit_count / 8)
        || (*desc).Pitch == 0
        || (*desc).Pitch < ((((*desc).Width * (*format).bit_count + 31) >> 3) & !3)
        || (*desc).Height == 0
        || (*desc).Height > UINT_MAX / (*desc).Pitch
    {
        return STATUS_INVALID_PARAMETER as u32;
    }

    let hDC: HDC;
    if (*desc).hDeviceDc.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }
    hDC = NtGdiCreateCompatibleDC((*desc).hDeviceDc);
    if hDC.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    let psurf: PSURFACE = SURFACE_AllocSurface(
        STYPE_BITMAP as u16,
        (*desc).Width,
        (*desc).Height,
        BitmapFormat((*format).bit_count, (*format).compression),
        (BMF_TOPDOWN | BMF_NOZEROINIT) as u32,
        (*desc).Pitch,
        0,
        (*desc).pMemory,
    );

    (*psurf).flags |= (DDB_SURFACE | API_BITMAP) as u32;

    (*desc).hDc = hDC;
    (*desc).hBitmap = (*psurf).SurfObj.hsurf as HANDLE;

    if (*format).bit_count <= 8 {
        let palette: PPALETTE = PALETTE_AllocPalette(
            PAL_INDEXED as u32,
            1u32 << (*format).bit_count,
            core::ptr::null_mut(),
            0,
            0,
            0,
        );
        if !palette.is_null() {
            SURFACE_vSetPalette(psurf, palette);
            PALETTE_ShareUnlockPalette(palette);
        }
    }

    SURFACE_UnlockSurface(psurf);

    NtGdiSelectBitmap((*desc).hDc, (*desc).hBitmap as HBITMAP);

    STATUS_SUCCESS as u32
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdDDICreateDCFromMemory(desc: *mut D3DKMT_CREATEDCFROMMEMORY) -> u32 {
    struct d3dddi_format_info {
        format: D3DDDIFORMAT,
        bit_count: u32,
        compression: u32,
        palette_size: u32,
        mask_r: u32,
        mask_g: u32,
        mask_b: u32,
    }

    let format_info: [d3dddi_format_info; 9] = [
        d3dddi_format_info { format: D3DDDIFMT_R8G8B8, bit_count: 24, compression: BI_RGB, palette_size: 0, mask_r: 0x00000000, mask_g: 0x00000000, mask_b: 0x00000000 },
        d3dddi_format_info { format: D3DDDIFMT_A8R8G8B8, bit_count: 32, compression: BI_RGB, palette_size: 0, mask_r: 0x00000000, mask_g: 0x00000000, mask_b: 0x00000000 },
        d3dddi_format_info { format: D3DDDIFMT_X8R8G8B8, bit_count: 32, compression: BI_RGB, palette_size: 0, mask_r: 0x00000000, mask_g: 0x00000000, mask_b: 0x00000000 },
        d3dddi_format_info { format: D3DDDIFMT_R5G6B5, bit_count: 16, compression: BI_BITFIELDS, palette_size: 0, mask_r: 0x0000f800, mask_g: 0x000007e0, mask_b: 0x0000001f },
        d3dddi_format_info { format: D3DDDIFMT_X1R5G5B5, bit_count: 16, compression: BI_BITFIELDS, palette_size: 0, mask_r: 0x00007c00, mask_g: 0x000003e0, mask_b: 0x0000001f },
        d3dddi_format_info { format: D3DDDIFMT_A1R5G5B5, bit_count: 16, compression: BI_BITFIELDS, palette_size: 0, mask_r: 0x00007c00, mask_g: 0x000003e0, mask_b: 0x0000001f },
        d3dddi_format_info { format: D3DDDIFMT_A4R4G4B4, bit_count: 16, compression: BI_BITFIELDS, palette_size: 0, mask_r: 0x00000f00, mask_g: 0x000000f0, mask_b: 0x0000000f },
        d3dddi_format_info { format: D3DDDIFMT_X4R4G4B4, bit_count: 16, compression: BI_BITFIELDS, palette_size: 0, mask_r: 0x00000f00, mask_g: 0x000000f0, mask_b: 0x0000000f },
        d3dddi_format_info { format: D3DDDIFMT_P8, bit_count: 8, compression: BI_RGB, palette_size: 256, mask_r: 0x00000000, mask_g: 0x00000000, mask_b: 0x00000000 },
    ];

    if desc.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    if (*desc).pMemory.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    let mut format: *const d3dddi_format_info = core::ptr::null();

    let mut i: u32 = 0;
    while (i as usize) < format_info.len() {
        if format_info[i as usize].format == (*desc).Format {
            format = &format_info[i as usize];
            break;
        }
        i += 1;
    }

    if format.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    if (*desc).Width > (UINT_MAX & !3) / ((*format).bit_count / 8)
        || (*desc).Pitch == 0
        || (*desc).Pitch < ((((*desc).Width * (*format).bit_count + 31) >> 3) & !3)
        || (*desc).Height == 0
        || (*desc).Height > UINT_MAX / (*desc).Pitch
    {
        return STATUS_INVALID_PARAMETER as u32;
    }

    let hDC: HDC;
    if (*desc).hDeviceDc.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }
    hDC = NtGdiCreateCompatibleDC((*desc).hDeviceDc);
    if hDC.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    let psurf: PSURFACE = SURFACE_AllocSurface(
        STYPE_BITMAP as u16,
        (*desc).Width,
        (*desc).Height,
        BitmapFormat((*format).bit_count, (*format).compression),
        (BMF_TOPDOWN | BMF_NOZEROINIT) as u32,
        (*desc).Pitch,
        0,
        (*desc).pMemory,
    );

    (*psurf).flags |= (DDB_SURFACE | API_BITMAP) as u32;

    (*desc).hDc = hDC;
    (*desc).hBitmap = (*psurf).SurfObj.hsurf as HANDLE;

    if (*format).bit_count <= 8 {
        let palette: PPALETTE = PALETTE_AllocPalette(
            PAL_INDEXED as u32,
            1u32 << (*format).bit_count,
            core::ptr::null_mut(),
            0,
            0,
            0,
        );
        if !palette.is_null() {
            SURFACE_vSetPalette(psurf, palette);
            PALETTE_ShareUnlockPalette(palette);
        }
    }

    SURFACE_UnlockSurface(psurf);

    NtGdiSelectBitmap((*desc).hDc, (*desc).hBitmap as HBITMAP);

    STATUS_SUCCESS as u32
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdDDIDestroyDCFromMemory(
    desc: *const D3DKMT_DESTROYDCFROMMEMORY,
) -> DWORD {
    if desc.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    if GDI_HANDLE_GET_TYPE((*desc).hDc as HGDIOBJ) != GDI_OBJECT_TYPE_DC as _
        || GDI_HANDLE_GET_TYPE((*desc).hBitmap) != GDI_OBJECT_TYPE_BITMAP as _
    {
        return STATUS_INVALID_PARAMETER as u32;
    }

    NtGdiDeleteObjectApp((*desc).hBitmap as _);
    NtGdiDeleteObjectApp((*desc).hDc as _);

    STATUS_SUCCESS as u32
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdDDIDestroyDCFromMemory(
    desc: *const D3DKMT_DESTROYDCFROMMEMORY,
) -> DWORD {
    if desc.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    if GDI_HANDLE_GET_TYPE((*desc).hDc as HGDIOBJ) != GDI_OBJECT_TYPE_DC as _
        || GDI_HANDLE_GET_TYPE((*desc).hBitmap) != GDI_OBJECT_TYPE_BITMAP as _
    {
        return STATUS_INVALID_PARAMETER as u32;
    }

    NtGdiDeleteObjectApp((*desc).hBitmap as _);
    NtGdiDeleteObjectApp((*desc).hDc as _);

    STATUS_SUCCESS as u32
}
