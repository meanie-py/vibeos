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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDICreateDCFromMemory(desc: *mut D3DKMT_CREATEDCFROMMEMORY) -> u32 {
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

    // The C dereferences `desc` field by field with no probe and no SEH (a C
    // bug: any process can hand it a kernel or unmapped address). Capture the
    // descriptor ONCE -- Width/Height/Pitch are validated and then used to size
    // the surface, so they must not be re-fetched -- and write the two output
    // handles back fault-tolerantly at the end.
    let d: D3DKMT_CREATEDCFROMMEMORY = match crate::safe_user_io::capture_user(desc as *const D3DKMT_CREATEDCFROMMEMORY) {
        Ok(v) => v,
        Err(st) => return st as u32,
    };

    if d.pMemory.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    let mut format: *const d3dddi_format_info = core::ptr::null();

    let mut i: u32 = 0;
    while (i as usize) < format_info.len() {
        if format_info[i as usize].format == d.Format {
            format = &format_info[i as usize];
            break;
        }
        i += 1;
    }

    if format.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    // `Width * bit_count` wraps in the C's unsigned arithmetic; keep that rather
    // than panic on overflow.
    if d.Width > (UINT_MAX & !3) / ((*format).bit_count / 8)
        || d.Pitch == 0
        || d.Pitch < (((d.Width.wrapping_mul((*format).bit_count)).wrapping_add(31) >> 3) & !3)
        || d.Height == 0
        || d.Height > UINT_MAX / d.Pitch
    {
        return STATUS_INVALID_PARAMETER as u32;
    }

    let hDC: HDC;
    if d.hDeviceDc.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }
    hDC = NtGdiCreateCompatibleDC(d.hDeviceDc);
    if hDC.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    let psurf: PSURFACE = SURFACE_AllocSurface(
        STYPE_BITMAP as u16,
        d.Width,
        d.Height,
        BitmapFormat((*format).bit_count, (*format).compression),
        (BMF_TOPDOWN | BMF_NOZEROINIT) as u32,
        d.Pitch,
        0,
        d.pMemory,
    );
    // Divergence: the C dereferences psurf unchecked. Fail like Wine's original.
    if psurf.is_null() {
        NtGdiDeleteObjectApp(hDC as _);
        return 0xC0000017_u32; // STATUS_NO_MEMORY
    }

    (*psurf).flags |= (DDB_SURFACE | API_BITMAP) as u32;

    let hBitmap: HANDLE = (*psurf).SurfObj.hsurf as HANDLE;

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

    NtGdiSelectBitmap(hDC, hBitmap as HBITMAP);

    // desc->hDc = hDC; desc->hBitmap = psurf->SurfObj.hsurf;
    let published = ntkrnl_sys::probe::write_back(core::ptr::addr_of_mut!((*desc).hDc), hDC)
        .and_then(|()| ntkrnl_sys::probe::write_back(core::ptr::addr_of_mut!((*desc).hBitmap), hBitmap));
    if let Err(st) = published {
        // The caller can never learn these handles; don't leak them.
        NtGdiDeleteObjectApp(hDC as _);
        NtGdiDeleteObjectApp(hBitmap as _);
        return st as u32;
    }

    STATUS_SUCCESS as u32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIDestroyDCFromMemory(
    desc: *const D3DKMT_DESTROYDCFROMMEMORY,
) -> DWORD {
    if desc.is_null() {
        return STATUS_INVALID_PARAMETER as u32;
    }

    // Captured once (the C reads each handle twice, unprobed and outside SEH):
    // the type check must be about the same handles that get deleted.
    let d: D3DKMT_DESTROYDCFROMMEMORY = match crate::safe_user_io::capture_user(desc) {
        Ok(v) => v,
        Err(st) => return st as u32,
    };

    if GDI_HANDLE_GET_TYPE(d.hDc as HGDIOBJ) != GDI_OBJECT_TYPE_DC as _
        || GDI_HANDLE_GET_TYPE(d.hBitmap) != GDI_OBJECT_TYPE_BITMAP as _
    {
        return STATUS_INVALID_PARAMETER as u32;
    }

    NtGdiDeleteObjectApp(d.hBitmap as _);
    NtGdiDeleteObjectApp(d.hDc as _);

    STATUS_SUCCESS as u32
}
