// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_012CB25B6A3D: [u16; 7] = [0x004Cu16, 0x0069, 0x0067, 0x0068, 0x0074, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_1FE40012AC31: [u16; 6] = [0x0042u16, 0x006F, 0x006C, 0x0064, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_391C9B250281: [u16; 11] = [
    0x0053u16, 0x0074, 0x0072, 0x0069, 0x006B, 0x0065, 0x006F, 0x0075, 0x0074, 0x0020,
    0x0000,
];
pub static RUSTCONV_WSTR_6E71BB9D8C2E: [u16; 8] = [0x0049u16, 0x0074, 0x0061, 0x006C, 0x0069, 0x0063, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_9A1624E374D4: [u16; 10] = [0x0043u16, 0x006F, 0x0075, 0x0072, 0x0069, 0x0065, 0x0072, 0x002D, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_EB250E9398EA: [u16; 12] = [
    0x0055u16, 0x006E, 0x0064, 0x0065, 0x0072, 0x0073, 0x0063, 0x006F, 0x0072, 0x0065,
    0x0020, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{IMAGE_DOS_HEADER, LPWSTR, PIMAGE_DOS_HEADER};
use core::ffi::c_void;

unsafe extern "C" fn IsValidPtr(
    mut p: PVOID,
    mut cjSize: ULONG,
    mut pStart: PVOID,
    mut pEnd: PVOID,
    mut cjAlign: ULONG,
) -> BOOLEAN {
    if (p as ULONG_PTR) < (pStart as ULONG_PTR)
        || (p as ULONG_PTR).wrapping_add(cjSize as ULONG_PTR) >= (pEnd as ULONG_PTR)
        || ((p as ULONG_PTR) & (cjAlign.wrapping_sub(1 as ULONG) as ULONG_PTR)) != 0
    {
        return FALSE;
    }
    return TRUE;
}

unsafe extern "C" fn FillFaceInfo(mut pface: PBMFD_FACE, mut pFontInfo: PFONTINFO16) -> BOOL {
    let mut ansi: [CHAR; 4 as usize] = [0; 4 as usize];
    let mut unicode: [WCHAR; 4 as usize] = [0; 4 as usize];
    let mut written: ULONG = 0;
    let mut dfFlags: DWORD = 0;

    (*pface).pFontInfo = pFontInfo;
    (*pface).ulVersion = GETVAL!((*pFontInfo).dfVersion) as ULONG;
    (*pface).cGlyphs = (((*pFontInfo).dfLastChar as i32 - (*pFontInfo).dfFirstChar as i32)
        + 1 as i32) as ULONG;

    /* Convert chars to unicode */
    ansi[0 as usize] = (*pFontInfo).dfFirstChar as CHAR;
    ansi[1 as usize] = (*pFontInfo).dfLastChar as CHAR;
    ansi[2 as usize] =
        ((*pFontInfo).dfFirstChar as i32 + (*pFontInfo).dfDefaultChar as i32) as CHAR;
    ansi[3 as usize] =
        ((*pFontInfo).dfFirstChar as i32 + (*pFontInfo).dfBreakChar as i32) as CHAR;
    EngMultiByteToUnicodeN(
        core::ptr::addr_of_mut!(unicode) as LPWSTR,
        ((4 as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as ULONG,
        core::ptr::addr_of_mut!(written),
        core::ptr::addr_of_mut!(ansi) as PCHAR,
        4 as ULONG,
    );
    (*pface).wcFirstChar = unicode[0 as usize];
    (*pface).wcLastChar = unicode[1 as usize];
    (*pface).wcDefaultChar = unicode[2 as usize];
    (*pface).wcBreakChar = unicode[3 as usize];

    /* Copy some values */
    (*pface).wPixHeight = GETVAL!((*pFontInfo).dfPixHeight) as WORD;
    (*pface).wPixWidth = GETVAL!((*pFontInfo).dfPixWidth) as WORD;
    (*pface).wWidthBytes = GETVAL!((*pFontInfo).dfWidthBytes) as WORD;
    (*pface).wAscent = GETVAL!((*pFontInfo).dfAscent) as WORD;
    (*pface).wDescent = ((*pface).wPixHeight as i32 - (*pface).wAscent as i32) as WORD;

    /* Some version specific members */
    if (*pface).ulVersion >= 0x300 as ULONG {
        dfFlags = GETVAL!((*pFontInfo).dfFlags) as DWORD;
        (*pface).wA = GETVAL!((*pFontInfo).dfAspace) as WORD;
        (*pface).wB = GETVAL!((*pFontInfo).dfBspace) as WORD;
        (*pface).wC = GETVAL!((*pFontInfo).dfCspace) as WORD;
        (*pface).pCharTable = ((*pFontInfo).dfCharTable).as_mut_ptr() as PBYTE;
        (*pface).cjEntrySize = core::mem::size_of::<GLYPHENTRY30>() as ULONG;
    } else {
        dfFlags = DFF_1COLOR as DWORD;
        (*pface).wA = 0 as WORD;
        (*pface).wB = 0 as WORD;
        (*pface).wC = 0 as WORD;
        (*pface).pCharTable =
            (&raw mut (*pFontInfo).dfReserved).offset(1 as isize) as PBYTE;
        (*pface).cjEntrySize = core::mem::size_of::<GLYPHENTRY20>() as ULONG;
    }

    (*pface).flInfo = FM_INFO_MASK as FLONG;

    /* If dfWidth is non-null, we have a fixed width font */
    if (dfFlags & (DFF_FIXED as DWORD)) != 0 || (*pface).wPixWidth as i32 != 0 {
        (*pface).flInfo |= FM_INFO_CONSTANT_WIDTH as FLONG;
    }

    /* Initialize color depth flags */
    if (dfFlags & (DFF_1COLOR as DWORD)) != 0 {
        (*pface).flInfo |= FM_INFO_1BPP as FLONG;
    } else if (dfFlags & (DFF_16COLOR as DWORD)) != 0 {
        (*pface).flInfo |= FM_INFO_4BPP as FLONG;
    } else if (dfFlags & (DFF_256COLOR as DWORD)) != 0 {
        (*pface).flInfo |= FM_INFO_8BPP as FLONG;
    } else if (dfFlags & (DFF_RGBCOLOR as DWORD)) != 0 {
        (*pface).flInfo |= FM_INFO_24BPP as FLONG;
    }

    // TODO: walk through all glyphs and veryfy them and calculate max values

    // FIXME: After this point, the whole font data should be verified!

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ParseFntFile(pvView: PVOID, cjView: ULONG) -> PVOID {
    NULL
}

unsafe extern "C" fn ParseFonFile(pvView: PVOID, cjView: ULONG) -> PVOID {
    let mut pDosHeader: PIMAGE_DOS_HEADER = pvView as PIMAGE_DOS_HEADER;
    let mut pOs2Header: PIMAGE_OS2_HEADER;
    let mut pResTable: PNE_RESTABLE;
    let mut pTInfo: PNE_TYPEINFO;
    let mut pFontInfo: PFONTINFO16;
    let mut pStart: PCHAR;
    let mut pEnd: PCHAR;
    let mut pfile: PBMFD_FILE = core::ptr::null_mut();
    let mut wShift: WORD;
    let mut i: ULONG;
    let mut cjOffset: ULONG;
    let mut cjLength: ULONG;
    let mut type_id: ULONG;
    let mut count: ULONG;

    /* Initial margins for valid pointers */
    pStart = pvView as PCHAR;
    pEnd = pStart.offset(cjView as isize);

    /* Check for image dos header */
    if GETVAL!((*pDosHeader).e_magic) as ULONG != IMAGE_DOS_MAGIC {
        return core::ptr::null_mut();
    }

    /* Get pointer to OS2 header and veryfy it is valid */
    pOs2Header = ((pDosHeader as PCHAR).offset(GETVAL!((*pDosHeader).e_lfanew) as isize)) as PVOID as PIMAGE_OS2_HEADER;
    pStart = pStart.offset(core::mem::size_of::<IMAGE_DOS_HEADER>() as isize);
    if IsValidPtr(
        pOs2Header as PVOID,
        core::mem::size_of::<IMAGE_OS2_HEADER>() as ULONG,
        pStart as PVOID,
        pEnd as PVOID,
        4,
    ) == 0
    {
        DbgPrint(
            b"e_lfanew is invalid: 0x%lx\n\0".as_ptr() as PCHAR,
            (*pDosHeader).e_lfanew,
        );
        return core::ptr::null_mut();
    }

    /* Get pointer to resource table and verify it is valid */
    pResTable = ((pOs2Header as PCHAR).offset(GETVAL!((*pOs2Header).ne_rsrctab) as isize)) as PVOID as PNE_RESTABLE;
    pStart = pOs2Header as PCHAR;
    if IsValidPtr(
        pResTable as PVOID,
        core::mem::size_of::<NE_RESTABLE>() as ULONG,
        pStart as PVOID,
        pEnd as PVOID,
        1,
    ) == 0
    {
        DbgPrint(
            b"pTInfo is invalid: 0x%p\n\0".as_ptr() as PCHAR,
            pResTable,
        );
        return core::ptr::null_mut();
    }

    wShift = GETVAL!((*pResTable).size_shift) as WORD;
    pTInfo = &raw mut (*pResTable).typeinfo as *mut NE_TYPEINFO as PNE_TYPEINFO;
    type_id = GETVAL!((*pTInfo).type_id) as ULONG;

    /* Loop the resource table to find a font resource */
    while type_id != 0 {
        /* Get number of nameinfo entries */
        count = GETVAL!((*pTInfo).count) as ULONG;

        /* Look for a font resource */
        if type_id == NE_RSCTYPE_FONT && count > 0 {
            DbgPrint(b"Found NE_RSCTYPE_FONT\n\0".as_ptr() as PCHAR);

            /* Allocate an info structure for this font and all faces */
            cjLength = (core::mem::size_of::<BMFD_FILE>() as ULONG)
                .wrapping_add((count.wrapping_sub(1)).wrapping_mul(
                    core::mem::size_of::<BMFD_FACE>() as ULONG,
                ));
            pfile = EngAllocMem(0, cjLength, TAG_FONTINFO) as PBMFD_FILE;
            if pfile.is_null() {
                DbgPrint(
                    b"Not enough memory: %ld\n\0".as_ptr() as PCHAR,
                    cjLength,
                );
                return core::ptr::null_mut();
            }

            (*pfile).cNumFaces = count;

            /* Fill all face info structures */
            i = 0;
            while i < count {
                cjOffset = (GETVAL!((*core::ptr::addr_of!((*pTInfo).nameinfo[0]).add(i as usize)).offset) as ULONG) << wShift;
                cjLength = (GETVAL!((*core::ptr::addr_of!((*pTInfo).nameinfo[0]).add(i as usize)).length) as ULONG) << wShift;
                pFontInfo = (pDosHeader as PCHAR).offset(cjOffset as isize) as PVOID as PFONTINFO16;

                if IsValidPtr(
                    pFontInfo as PVOID,
                    cjLength,
                    pStart as PVOID,
                    pEnd as PVOID,
                    1,
                ) == 0
                {
                    DbgPrint(
                        b"pFontInfo is invalid: 0x%p\n\0".as_ptr() as PCHAR,
                        pFontInfo,
                    );
                    EngFreeMem(pfile as PVOID);
                    return core::ptr::null_mut();
                }

                /* Validate FONTINFO and fill face info */
                if FillFaceInfo(
                    (&raw mut (*pfile).aface as *mut BMFD_FACE).offset(i as isize) as PBMFD_FACE,
                    pFontInfo,
                ) == 0
                {
                    DbgPrint(
                        b"pFontInfo is invalid: 0x%p\n\0".as_ptr() as PCHAR,
                        pFontInfo,
                    );
                    EngFreeMem(pfile as PVOID);
                    return core::ptr::null_mut();
                }

                i = i.wrapping_add(1);
            }

            /* Break out of the loop */
            break;
        }

        /* Following pointers must be bigger than this */
        pStart = pTInfo as PCHAR;

        /* Goto next entry in resource table */
        pTInfo = (&raw mut (*pTInfo).nameinfo as *mut NE_NAMEINFO).offset(count as isize) as *mut NE_NAMEINFO as PVOID as PNE_TYPEINFO;

        /* Verify that the new pTInfo pointer is valid */
        if IsValidPtr(
            pTInfo as PVOID,
            core::mem::size_of::<NE_TYPEINFO>() as ULONG,
            pStart as PVOID,
            pEnd as PVOID,
            1,
        ) == 0
        {
            DbgPrint(
                b"pTInfo is invalid: 0x%p\n\0".as_ptr() as PCHAR,
                pTInfo,
            );
            return core::ptr::null_mut();
        }

        type_id = GETVAL!((*pTInfo).type_id) as ULONG;
    }

    return pfile as PVOID;
}
#[no_mangle]
pub unsafe extern "system" fn BmfdLoadFontFile(cFiles: ULONG, piFile: *mut ULONG_PTR, ppvView: *mut PVOID, pcjView: *mut ULONG, pdv: *mut DESIGNVECTOR, ulLangID: ULONG, ulFastCheckSum: ULONG) -> ULONG_PTR {
    let mut pfile: PBMFD_FILE = core::ptr::null_mut::<BMFD_FILE>();
    let mut pvView: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut cjView: ULONG = 0;

    DbgPrint(b"BmfdLoadFontFile()\n\0".as_ptr() as *const core::ffi::c_char as PCHAR);
    __debugbreak();

    /* Check parameters */
    if cFiles != 1 {
        DbgPrint(
            b"Only 1 File is allowed, got %ld!\n\0".as_ptr() as *const core::ffi::c_char as PCHAR,
            cFiles,
        );
        return HFF_INVALID;
    }

    /* Map the font file */
    if EngMapFontFileFD(
        *piFile,
        core::ptr::addr_of_mut!(pvView) as *mut PULONG,
        core::ptr::addr_of_mut!(cjView),
    ) == 0
    {
        DbgPrint(
            b"Could not map font file!\n\0".as_ptr() as *const core::ffi::c_char as PCHAR,
            cFiles,
        );
        return HFF_INVALID;
    }

    DbgPrint(
        b"mapped font file to %p, site if %ld\n\0".as_ptr() as *const core::ffi::c_char as PCHAR,
        pvView,
        cjView,
    );

    /* Try to parse a .fon file */
    pfile = ParseFonFile(pvView, cjView) as PBMFD_FILE;

    if pfile.is_null() {
        /* Could be a .fnt file */
        pfile = ParseFntFile(pvView, cjView) as PBMFD_FILE;
    }

    /* Check whether we succeeded finding a font */
    if pfile.is_null() {
        DbgPrint(b"No font data found\n\0".as_ptr() as *const core::ffi::c_char as PCHAR);

        /* Unmap the file */
        EngUnmapFontFileFD(*piFile);

        /* Failure! */
        return HFF_INVALID;
    }

    (*pfile).iFile = *piFile;
    (*pfile).pvView = pvView;

    /* Success, return the pointer to font info structure */
    return pfile as ULONG_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn BmfdUnloadFontFile(iFile: ULONG_PTR) -> BOOL {
    let pfile: PBMFD_FILE = iFile as PBMFD_FILE;

    DbgPrint(b"BmfdUnloadFontFile()\n\0".as_ptr() as *const i8);

    /* Unmap the font file */
    EngUnmapFontFileFD((*pfile).iFile);

    /* Free the memory that was allocated for the font */
    EngFreeMem(pfile as PVOID);

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn BmfdQueryFontFile(iFile: ULONG_PTR, ulMode: ULONG, cjBuf: ULONG, pulBuf: *mut ULONG) -> LONG {
    let pfile: PBMFD_FILE = iFile as PBMFD_FILE;

    DbgPrint(b"BmfdQueryFontFile()\n\0".as_ptr() as PCHAR);
//    __debugbreak();

    match ulMode {
        QFF_DESCRIPTION => {
            /* We copy the face name of the 1st face */
            let pDesc: PCHAR = (*pfile).aface[0].pszFaceName;
            let mut cOutSize: ULONG = 0;
            if !pulBuf.is_null() {
                EngMultiByteToUnicodeN(
                    pulBuf as LPWSTR,
                    cjBuf,
                    core::ptr::addr_of_mut!(cOutSize),
                    pDesc,
                    strnlen(pDesc as *const c_char, LF_FACESIZE as size_t) as ULONG,
                );
            } else {
                cOutSize = ((strnlen(pDesc as *const c_char, LF_FACESIZE as size_t) as usize
                    + 1)
                    * core::mem::size_of::<WCHAR>()) as ULONG;
            }
            return cOutSize as LONG;
        }

        QFF_NUMFACES => {
            /* return the number of faces in the file */
            return (*pfile).cNumFaces as LONG;
        }

        _ => {
            return (FD_ERROR) as LONG;
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn BmfdQueryFontCaps(culCaps: ULONG, pulCaps: *mut ULONG) -> LONG {
    DbgPrint(b"BmfdQueryFontCaps()\n\0".as_ptr() as *const i8);

    /* We need room for 2 ULONGs */
    if culCaps < 2 as ULONG {
        return FD_ERROR as LONG;
    }

    /* We only support 1 bpp */
    *pulCaps.offset(0 as isize) = 2 as ULONG;
    *pulCaps.offset(1 as isize) = QC_1BIT as ULONG;

    return 2 as LONG;
}
#[no_mangle]
pub unsafe extern "system" fn BmfdQueryFontTree(dhpdev: DHPDEV, iFile: ULONG_PTR, iFace: ULONG, iMode: ULONG, pid: *mut ULONG_PTR) -> PVOID {
    let mut pfile: PBMFD_FILE = iFile as PBMFD_FILE;
    let mut pface: PBMFD_FACE = core::ptr::null_mut();
    let mut i: ULONG = 0;
    let mut j: ULONG = 0;
    let mut cjSize: ULONG = 0;
    let mut cGlyphs: ULONG = 0;
    let mut cRuns: ULONG = 0;
    let mut ch: CHAR = 0;
    let mut chFirst: CHAR = 0;
    let mut ach: [CHAR; 256] = [0; 256];
    let mut wc: WCHAR = 0;
    let mut awc: [WCHAR; 256] = [0; 256];
    let mut pGlyphSet: PFD_GLYPHSET = core::ptr::null_mut();
    let mut pwcrun: *mut WCRUN = core::ptr::null_mut();
    let mut phglyphs: PHGLYPH = core::ptr::null_mut();

    DbgPrint(
        b"DrvQueryFontTree(iMode=%ld)\n\0".as_ptr() as PCHAR,
        iMode,
    );

    /* Check parameters, we only support QFT_GLYPHSET */
    if iFace == 0 || iFace > (*pfile).cNumFaces || iMode != QFT_GLYPHSET as ULONG {
        DbgPrint(
            b"iFace = %ld, cNumFaces = %ld\n\0".as_ptr() as PCHAR,
            iFace,
            (*pfile).cNumFaces,
        );
        return core::ptr::null_mut();
    }

    /* Get a pointer to the face data */
    pface = ((&raw mut (*pfile).aface) as *mut BMFD_FACE)
        .offset(iFace.wrapping_sub(1 as ULONG) as isize) as PBMFD_FACE;

    /* Get the number of characters in the face */
    cGlyphs = (*pface).cGlyphs;

    chFirst = (*(*pface).pFontInfo).dfFirstChar as CHAR;

    /* Build array of supported chars */
    i = 0 as ULONG;
    while i < cGlyphs {
        ach[i as usize] = (chFirst as ULONG).wrapping_add(i) as CHAR;
        i = i.wrapping_add(1);
    }

    /* Convert the chars to unicode */
    EngMultiByteToUnicodeN(
        (&raw mut awc) as LPWSTR,
        core::mem::size_of::<[WCHAR; 256]>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
        (&raw mut ach) as PCHAR,
        cGlyphs,
    );

    /* Sort both arrays in wchar order */
    i = 0 as ULONG;
    while i < cGlyphs.wrapping_sub(1 as ULONG) {
        wc = awc[i as usize];
        j = i.wrapping_add(1 as ULONG);
        while j < cGlyphs {
            if (awc[j as usize] as i32) < (wc as i32) {
                awc[i as usize] = awc[j as usize];
                awc[j as usize] = wc;
                wc = awc[i as usize];
                ch = ach[i as usize];
                ach[i as usize] = ach[j as usize];
                ach[j as usize] = ch;
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    /* Find number of WCRUNs */
    cRuns = 1 as ULONG;
    i = 1 as ULONG;
    while i < cGlyphs {
        if (awc[i as usize] as i32)
            != (awc[i.wrapping_sub(1 as ULONG) as usize] as i32)
                + 1 as i32
        {
            cRuns = cRuns.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    /* Calculate FD_GLYPHSET size */
    cjSize = (core::mem::size_of::<FD_GLYPHSET>() as ULONG)
        .wrapping_add(
            cRuns
                .wrapping_sub(1 as ULONG)
                .wrapping_mul(core::mem::size_of::<WCRUN>() as ULONG),
        )
        .wrapping_add(
            cGlyphs.wrapping_mul(core::mem::size_of::<HGLYPH>() as ULONG),
        );

    /* Allocate the FD_GLYPHSET structure */
    pGlyphSet = EngAllocMem(
        0 as ULONG,
        cjSize,
        TAG_GLYPHSET as ULONG,
    ) as PFD_GLYPHSET;
    if pGlyphSet.is_null() {
        return core::ptr::null_mut();
    }

    /* Initialize FD_GLYPHSET */
    (*pGlyphSet).cjThis = cjSize;
    (*pGlyphSet).flAccel = 0 as FLONG;
    (*pGlyphSet).cGlyphsSupported = cGlyphs;
    (*pGlyphSet).cRuns = cRuns;

    /* Initialize 1st WCRUN */
    pwcrun = (&raw mut (*pGlyphSet).awcrun) as *mut WCRUN;
    phglyphs = ((&raw mut (*pGlyphSet).awcrun) as *mut WCRUN).offset(cRuns as isize) as PHGLYPH;
    (*pwcrun.offset(0 as isize)).wcLow = awc[0 as usize];
    (*pwcrun.offset(0 as isize)).cGlyphs = 1 as USHORT;
    (*pwcrun.offset(0 as isize)).phg = phglyphs;
    *phglyphs.offset(0 as isize) = 0 as HGLYPH;

    /* Walk through all supported chars */
    i = 1 as ULONG;
    j = 0 as ULONG;
    while i < cGlyphs {
        /* Use offset to glyph entry as hglyph */
        *phglyphs.offset(i as isize) = ((ach[i as usize] as i32 - chFirst as i32) as ULONG)
            .wrapping_mul((*pface).cjEntrySize) as HGLYPH;

        /* Check whether we can append the wchar to a run */
        if (awc[i as usize] as i32)
            == (awc[i.wrapping_sub(1 as ULONG) as usize] as i32)
                + 1 as i32
        {
            /* Append to current WCRUN */
            (*pwcrun.offset(j as isize)).cGlyphs =
                (*pwcrun.offset(j as isize)).cGlyphs.wrapping_add(1);
        } else {
            /* Add a new WCRUN */
            j = j.wrapping_add(1);
            (*pwcrun.offset(j as isize)).wcLow = awc[i as usize];
            (*pwcrun.offset(j as isize)).cGlyphs = 1 as USHORT;
            (*pwcrun.offset(j as isize)).phg = phglyphs.offset(i as isize) as PHGLYPH;
        }
        i = i.wrapping_add(1);
    }

    /* Set *pid to the allocated structure for use in BmfdFree */
    *pid = pGlyphSet as ULONG_PTR;

    return pGlyphSet as PVOID;
}
#[no_mangle]
pub unsafe extern "system" fn BmfdQueryFont(dhpdev: DHPDEV, iFile: ULONG_PTR, iFace: ULONG, pid: *mut ULONG_PTR) -> PIFIMETRICS {
    let mut pfile: PBMFD_FILE = iFile as PBMFD_FILE;
    let mut pface: PBMFD_FACE = core::ptr::null_mut();
    let mut pFontInfo: PFONTINFO16 = core::ptr::null_mut();
    let mut pifi: PIFIMETRICS = core::ptr::null_mut();
    let mut pifiX: PBMFD_IFIMETRICS = core::ptr::null_mut();
    let mut panose: PANOSE = core::mem::zeroed::<PANOSE>();

    DbgPrint(b"BmfdQueryFont()\n\0".as_ptr() as PCHAR);

    /* Validate parameters */
    if iFace > (*pfile).cNumFaces || pid.is_null() {
        return core::ptr::null_mut();
    }

    pface = (&raw mut (*pfile).aface as *mut BMFD_FACE)
        .offset(iFace.wrapping_sub(1 as ULONG) as isize) as PBMFD_FACE;
    pFontInfo = (*pface).pFontInfo;

    /* Allocate the structure */
    pifiX = EngAllocMem(
        FL_ZERO_MEMORY as ULONG,
        core::mem::size_of::<BMFD_IFIMETRICS>() as ULONG,
        TAG_IFIMETRICS as ULONG,
    ) as PBMFD_IFIMETRICS;
    if pifiX.is_null() {
        return core::ptr::null_mut();
    }

    /* Return a pointer to free it later */
    *pid = pifiX as ULONG_PTR;

    /* Fill IFIMETRICS */
    pifi = &raw mut (*pifiX).ifim as PIFIMETRICS;
    (*pifi).cjThis = core::mem::size_of::<BMFD_IFIMETRICS>() as ULONG;
    (*pifi).cjIfiExtra = 0 as ULONG;
    (*pifi).dpwszFamilyName = core::mem::offset_of!(BMFD_IFIMETRICS, wszFamilyName) as PTRDIFF;
    (*pifi).dpwszStyleName = core::mem::offset_of!(BMFD_IFIMETRICS, wszFamilyName) as PTRDIFF;
    (*pifi).dpwszFaceName = core::mem::offset_of!(BMFD_IFIMETRICS, wszFaceName) as PTRDIFF;
    (*pifi).dpwszUniqueName = core::mem::offset_of!(BMFD_IFIMETRICS, wszFaceName) as PTRDIFF;
    (*pifi).dpFontSim = 0 as PTRDIFF;
    (*pifi).lEmbedId = 0 as LONG;
    (*pifi).lItalicAngle = 0 as LONG;
    (*pifi).lCharBias = 0 as LONG;
    (*pifi).dpCharSets = 0 as PTRDIFF;
    (*pifi).jWinCharSet = (*pFontInfo).dfCharSet;
    (*pifi).jWinPitchAndFamily = (*pFontInfo).dfPitchAndFamily;
    (*pifi).usWinWeight = GETVAL!((*pFontInfo).dfWeight) as USHORT;
    (*pifi).flInfo = (*pface).flInfo as ULONG;
    (*pifi).fsSelection = 0 as USHORT;
    (*pifi).fsType = 0 as USHORT;
    (*pifi).fwdUnitsPerEm = GETVAL!((*pFontInfo).dfPixHeight) as FWORD;
    (*pifi).fwdLowestPPEm = 0 as FWORD;
    (*pifi).fwdWinAscender = GETVAL!((*pFontInfo).dfAscent) as FWORD;
    (*pifi).fwdWinDescender = ((*pifi).fwdUnitsPerEm as ::core::ffi::c_int
        - (*pifi).fwdWinAscender as ::core::ffi::c_int) as FWORD;
    (*pifi).fwdMacAscender = (*pifi).fwdWinAscender;
    (*pifi).fwdMacDescender = (-((*pifi).fwdWinDescender as ::core::ffi::c_int)) as FWORD;
    (*pifi).fwdMacLineGap = 0 as FWORD;
    (*pifi).fwdTypoAscender = (*pifi).fwdWinAscender;
    (*pifi).fwdTypoDescender = (-((*pifi).fwdWinDescender as ::core::ffi::c_int)) as FWORD;
    (*pifi).fwdTypoLineGap = 0 as FWORD;
    (*pifi).fwdAveCharWidth = GETVAL!((*pFontInfo).dfAvgWidth) as FWORD;
    (*pifi).fwdMaxCharInc = GETVAL!((*pFontInfo).dfMaxWidth) as FWORD;
    (*pifi).fwdCapHeight = ((*pifi).fwdUnitsPerEm as ::core::ffi::c_int
        / 2 as ::core::ffi::c_int) as FWORD;
    (*pifi).fwdXHeight = ((*pifi).fwdUnitsPerEm as ::core::ffi::c_int
        / 4 as ::core::ffi::c_int) as FWORD;
    (*pifi).fwdSubscriptXSize = 0 as FWORD;
    (*pifi).fwdSubscriptYSize = 0 as FWORD;
    (*pifi).fwdSubscriptXOffset = 0 as FWORD;
    (*pifi).fwdSubscriptYOffset = 0 as FWORD;
    (*pifi).fwdSuperscriptXSize = 0 as FWORD;
    (*pifi).fwdSuperscriptYSize = 0 as FWORD;
    (*pifi).fwdSuperscriptXOffset = 0 as FWORD;
    (*pifi).fwdSuperscriptYOffset = 0 as FWORD;
    (*pifi).fwdUnderscoreSize = 0o1 as FWORD;
    (*pifi).fwdUnderscorePosition =
        (-(1 as ::core::ffi::c_int)) as FWORD;
    (*pifi).fwdStrikeoutSize = 1 as FWORD;
    (*pifi).fwdStrikeoutPosition = ((*pifi).fwdXHeight as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as FWORD;
    (*pifi).chFirstChar = (*pFontInfo).dfFirstChar;
    (*pifi).chLastChar = (*pFontInfo).dfLastChar;
    (*pifi).chDefaultChar = ((*pFontInfo).dfFirstChar as ::core::ffi::c_int
        + (*pFontInfo).dfDefaultChar as ::core::ffi::c_int) as BYTE;
    (*pifi).chBreakChar = ((*pFontInfo).dfFirstChar as ::core::ffi::c_int
        + (*pFontInfo).dfBreakChar as ::core::ffi::c_int) as BYTE;
    (*pifi).wcFirstChar = (*pface).wcFirstChar;
    (*pifi).wcLastChar = (*pface).wcLastChar;
    (*pifi).wcDefaultChar = (*pface).wcDefaultChar;
    (*pifi).wcBreakChar = (*pface).wcBreakChar;
    (*pifi).ptlBaseline.x = 1 as LONG;
    (*pifi).ptlBaseline.y = 0 as LONG;
    (*pifi).ptlAspect.x = (*pFontInfo).dfVertRes as LONG; // CHECKME
    (*pifi).ptlAspect.y = (*pFontInfo).dfHorizRes as LONG;
    (*pifi).ptlCaret.x = 0 as LONG;
    (*pifi).ptlCaret.y = 1 as LONG;
    (*pifi).rclFontBox.left = 0 as LONG;
    (*pifi).rclFontBox.right = (*pifi).fwdAveCharWidth as LONG;
    (*pifi).rclFontBox.top = (*pifi).fwdWinAscender as LONG;
    (*pifi).rclFontBox.bottom = (-((*pifi).fwdWinDescender as ::core::ffi::c_int)) as LONG;
    *(&raw mut (*pifi).achVendId as *mut DWORD) = 0x30303030 as DWORD; // FIXME
    (*pifi).cKerningPairs = 0 as ULONG;
    (*pifi).ulPanoseCulture = FM_PANOSE_CULTURE_LATIN as ULONG;
    (*pifi).panose = panose;

    /* Set char sets */
    (*pifiX).ajCharSet[0 as usize] = (*pifi).jWinCharSet;
    (*pifiX).ajCharSet[1 as usize] = DEFAULT_CHARSET as BYTE;

    if ((*pface).flInfo & (FM_INFO_CONSTANT_WIDTH as FLONG)) != 0 {
        (*pifi).jWinPitchAndFamily = ((*pifi).jWinPitchAndFamily as ::core::ffi::c_int
            | FIXED_PITCH as ::core::ffi::c_int) as BYTE;
    }

    wcscpy(
        (&raw mut (*pifiX).wszFaceName) as *mut u16,
        RUSTCONV_WSTR_9A1624E374D4.as_ptr(),
    );
    wcscpy(
        (&raw mut (*pifiX).wszFamilyName) as *mut u16,
        RUSTCONV_WSTR_9A1624E374D4.as_ptr(),
    );

    /* Initialize font weight style flags and string */
    if ((*pifi).usWinWeight as ::core::ffi::c_int) == (FW_REGULAR as ::core::ffi::c_int) {
        //     pifi->fsSelection |= FM_SEL_REGULAR;
    } else if ((*pifi).usWinWeight as ::core::ffi::c_int) > (FW_SEMIBOLD as ::core::ffi::c_int) {
        (*pifi).fsSelection = ((*pifi).fsSelection as ::core::ffi::c_int
            | FM_SEL_BOLD as ::core::ffi::c_int) as USHORT;
        wcscat(
            (&raw mut (*pifiX).wszStyleName) as *mut u16,
            RUSTCONV_WSTR_1FE40012AC31.as_ptr(),
        );
    } else if ((*pifi).usWinWeight as ::core::ffi::c_int) <= (FW_LIGHT as ::core::ffi::c_int) {
        wcscat(
            (&raw mut (*pifiX).wszStyleName) as *mut u16,
            RUSTCONV_WSTR_012CB25B6A3D.as_ptr(),
        );
    }

    if (*pFontInfo).dfItalic != 0 {
        (*pifi).fsSelection = ((*pifi).fsSelection as ::core::ffi::c_int
            | FM_SEL_ITALIC as ::core::ffi::c_int) as USHORT;
        wcscat(
            (&raw mut (*pifiX).wszStyleName) as *mut u16,
            RUSTCONV_WSTR_6E71BB9D8C2E.as_ptr(),
        );
    }

    if (*pFontInfo).dfUnderline != 0 {
        (*pifi).fsSelection = ((*pifi).fsSelection as ::core::ffi::c_int
            | FM_SEL_UNDERSCORE as ::core::ffi::c_int) as USHORT;
        wcscat(
            (&raw mut (*pifiX).wszStyleName) as *mut u16,
            RUSTCONV_WSTR_EB250E9398EA.as_ptr(),
        );
    }

    if (*pFontInfo).dfStrikeOut != 0 {
        (*pifi).fsSelection = ((*pifi).fsSelection as ::core::ffi::c_int
            | FM_SEL_STRIKEOUT as ::core::ffi::c_int) as USHORT;
        wcscat(
            (&raw mut (*pifiX).wszStyleName) as *mut u16,
            RUSTCONV_WSTR_391C9B250281.as_ptr(),
        );
    }

    return pifi;
}
#[no_mangle]
pub unsafe extern "system" fn BmfdFree(pv: PVOID, id: ULONG_PTR) {
    DbgPrint(b"BmfdFree()\n\0".as_ptr() as PCHAR);
    if id != 0 {
        EngFreeMem(id as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn BmfdDestroyFont(pfo: *mut FONTOBJ) {
    /* Free the font realization info */
    EngFreeMem((*pfo).pvProducer);
    (*pfo).pvProducer = core::ptr::null_mut();
}
