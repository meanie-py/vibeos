use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn HackFixup(pvView: PVOID, cjView: ULONG) -> PVOID {
    let mut pc: *mut CHAR = core::ptr::null_mut();

    pc = EngAllocMem(0, cjView, 1953329184u32) as *mut CHAR; // 'tmp '
    memcpy(pc as *mut core::ffi::c_void, pvView as *const core::ffi::c_void, cjView as usize);

    *pc = 0 as CHAR;

    return pc as PVOID;
}

#[no_mangle]
pub unsafe extern "system" fn FtfdLoadFontFile(cFiles: ULONG, piFile: *mut ULONG_PTR, ppvView: *mut PVOID, pcjView: *mut ULONG, pdv: *mut DESIGNVECTOR, ulLangID: ULONG, ulFastCheckSum: ULONG) -> ULONG_PTR {
    let mut pvView: PVOID = core::ptr::null_mut();
    let mut cjView: ULONG = 0;
    let mut i: ULONG = 0;
    let mut fterror: FT_Error = 0;
    let mut ftface: FT_Face = core::ptr::null_mut();
    let mut pfile: PFTFD_FILE = core::ptr::null_mut();
    let mut cjSize: ULONG = 0;
    let mut cNumFaces: ULONG = 0;

    FT_Message(b"FtfdLoadFontFile()\n\0".as_ptr() as *const core::ffi::c_char);

    /* Check parameters */
    if cFiles != 1 {
        FT_Message(
            b"Only 1 File is allowed, got %ld!\n\0".as_ptr() as *const core::ffi::c_char,
            cFiles,
        );
        return HFF_INVALID;
    }

    /* Map the font file */
    if EngMapFontFileFD(*piFile, &raw mut pvView as *mut PULONG, &raw mut cjView) == 0 {
        FT_Message(b"Could not map font file!\n\0".as_ptr() as *const core::ffi::c_char);
        return HFF_INVALID;
    }

    // HACK!!!
    pvView = HackFixup(pvView, cjView);

    fterror = FT_New_Memory_Face(
        gftlibrary,
        pvView as *const FT_Byte,
        cjView as FT_Long,
        0,
        &raw mut ftface,
    );
    if fterror != 0 {
        FT_Message(b"No faces found in file\n\0".as_ptr() as *const core::ffi::c_char);

        /* Unmap the file */
        EngUnmapFontFileFD(*piFile);

        /* Failure! */
        return HFF_INVALID;
    }

    /* Get number of faces from the first face */
    cNumFaces = (*ftface).num_faces as ULONG;

    cjSize = (core::mem::size_of::<FTFD_FILE>() as usize)
        .wrapping_add((cNumFaces as usize).wrapping_mul(core::mem::size_of::<FT_Face>() as usize))
        as ULONG;
    pfile = EngAllocMem(0, cjSize, 0x64667446u32) as PFTFD_FILE;
    if pfile.is_null() {
        FT_Message(b"EngAllocMem() failed.\n\0".as_ptr() as *const core::ffi::c_char);

        /* Unmap the file */
        EngUnmapFontFileFD(*piFile);

        /* Failure! */
        return HFF_INVALID;
    }

    (*pfile).cNumFaces = cNumFaces;
    (*pfile).iFile = *piFile;
    (*pfile).pvView = pvView;
    (*pfile).cjView = cjView;

    i = 0;
    while i < (*pfile).cNumFaces {
        (*core::ptr::addr_of_mut!((*pfile).aftface[0]).add(i as usize)) = ftface;
        FT_Select_Charmap(ftface, FT_ENCODING_UNICODE);
        i = i.wrapping_add(1);
    }

    FT_Message(
        b"Success! Returning %ld faces\n\0".as_ptr() as *const core::ffi::c_char,
        cNumFaces,
    );

    return pfile as ULONG_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn FtfdUnloadFontFile(iFile: ULONG_PTR) -> BOOL {
    let pfile: PFTFD_FILE = iFile as PFTFD_FILE;
    let mut i: ULONG = 0;

    FT_Message(b"FtfdUnloadFontFile()\n\0".as_ptr() as *const i8);

    // HACK!!!
    EngFreeMem((*pfile).pvView);

    /* Cleanup faces */
    while i < (*pfile).cNumFaces {
        FT_Done_Face(*(core::ptr::addr_of_mut!((*pfile).aftface) as *mut FT_Face).offset(i as isize));
        i = i.wrapping_add(1);
    }

    /* Unmap the font file */
    EngUnmapFontFileFD((*pfile).iFile);

    /* Free the memory that was allocated for the font */
    EngFreeMem(pfile as PVOID);

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn FtfdQueryFontFile(iFile: ULONG_PTR, ulMode: ULONG, cjBuf: ULONG, pulBuf: *mut ULONG) -> LONG {
    let pfile: PFTFD_FILE = iFile as PFTFD_FILE;
    FT_Message(b"FtfdQueryFontFile(ulMode=%ld)\n\0".as_ptr() as *const i8, ulMode);
//    __debugbreak();

    match ulMode {
        QFF_DESCRIPTION => {
            return 0;
        }

        QFF_NUMFACES => {
            /* return the number of faces in the file */
            return (*pfile).cNumFaces as LONG;
        }

        _ => {}
    }

    return (FD_ERROR) as LONG;
}
#[no_mangle]
pub unsafe extern "system" fn FtfdQueryFont(dhpdev: DHPDEV, iFile: ULONG_PTR, iFace: ULONG, pid: *mut ULONG_PTR) -> PIFIMETRICS {
    let mut pfile: PFTFD_FILE = iFile as PFTFD_FILE;
    let mut pifiX: PFTFD_IFIMETRICS = core::ptr::null_mut();
    let mut pifi: PIFIMETRICS = core::ptr::null_mut();
    let mut ftface: FT_Face = core::ptr::null_mut();
    let mut fterror: FT_Error = 0;
    let mut i: ULONG = 0;

    FT_Message(b"FtfdQueryFont()\n\0".as_ptr() as *const i8);

    /* Validate parameters */
    if iFace > (*pfile).cNumFaces || pid.is_null() {
        FT_Message(b"iFace > pfile->cNumFaces || !pid\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    fterror = FT_New_Memory_Face(
        gftlibrary,
        (*pfile).pvView as *const FT_Byte,
        (*pfile).cjView as FT_Long,
        iFace.wrapping_sub(1 as ULONG) as FT_Long,
        core::ptr::addr_of_mut!(ftface),
    );
    if fterror != 0 {
        FT_Message(b"FT_New_Memory_Face failed\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    /* Allocate the ifi metrics structure */
    pifiX = EngAllocMem(
        FL_ZERO_MEMORY,
        core::mem::size_of::<FTFD_IFIMETRICS>() as ULONG,
        TAG_IFIMETRICS,
    ) as PFTFD_IFIMETRICS;
    if pifiX.is_null() {
        FT_Message(b"EngAllocMem() failed.\n\0".as_ptr() as *const i8);
        FT_Done_Face(ftface);
        return core::ptr::null_mut();
    }

    /* Fill IFIMETRICS */
    pifi = core::ptr::addr_of_mut!((*pifiX).ifim) as PIFIMETRICS;
    (*pifi).cjThis = core::mem::size_of::<FTFD_IFIMETRICS>() as ULONG;
    (*pifi).cjIfiExtra = 0;

    /* Relative offsets */
    (*pifi).dpwszFamilyName = FIELD_OFFSET!(FTFD_IFIMETRICS, wszFamilyName) as PTRDIFF;
    (*pifi).dpwszStyleName = FIELD_OFFSET!(FTFD_IFIMETRICS, wszStyleName) as PTRDIFF;
    (*pifi).dpwszFaceName = FIELD_OFFSET!(FTFD_IFIMETRICS, wszFaceName) as PTRDIFF;
    (*pifi).dpwszUniqueName = FIELD_OFFSET!(FTFD_IFIMETRICS, wszFaceName) as PTRDIFF;
    (*pifi).dpCharSets = FIELD_OFFSET!(FTFD_IFIMETRICS, ajCharSet) as PTRDIFF;
    (*pifi).dpFontSim = 0;

    /* Charsets */
    (*pifi).jWinCharSet = (ANSI_CHARSET as u8);
    (*pifiX).ajCharSet[0 as usize] = (*pifi).jWinCharSet;
    i = 1;
    while i < 16 {
        (*pifiX).ajCharSet[i as usize] = (DEFAULT_CHARSET as u8);
        i = i.wrapping_add(1);
    }

    (*pifi).lEmbedId = 0;
    (*pifi).lItalicAngle = 0;
    (*pifi).lCharBias = 0;
    (*pifi).jWinPitchAndFamily = (VARIABLE_PITCH | FF_DONTCARE) as BYTE; // FIXME
    (*pifi).usWinWeight = FW_MEDIUM as USHORT; // FIXME
    (*pifi).flInfo = (FM_INFO_TECH_TRUETYPE
        | FM_INFO_ARB_XFORMS
        | FM_INFO_1BPP
        | FM_INFO_4BPP
        | FM_INFO_RETURNS_OUTLINES
        | FM_INFO_RETURNS_BITMAPS
        | FM_INFO_RIGHT_HANDED) as ULONG;
    (*pifi).fsSelection = 0 as USHORT;
    (*pifi).fsType = 0 as USHORT;

    /* Font resolution */
    (*pifi).fwdUnitsPerEm = (*ftface).units_per_EM as FWORD;
    (*pifi).fwdLowestPPEm = 8 as FWORD; // FIXME

    /* Font metrics */
    (*pifi).fwdWinAscender = (*ftface).ascender as FWORD;
    (*pifi).fwdWinDescender = -((*ftface).descender as ::core::ffi::c_int) as FWORD;
    (*pifi).fwdMacAscender = (*pifi).fwdWinAscender;
    (*pifi).fwdMacDescender = -((*pifi).fwdWinDescender as ::core::ffi::c_int) as FWORD;
    (*pifi).fwdMacLineGap = 0 as FWORD;
    (*pifi).fwdTypoAscender = (*pifi).fwdWinAscender;
    (*pifi).fwdTypoDescender = 0 as FWORD; // FIXME!!! - pifi->fwdWinDescender;
    (*pifi).fwdTypoLineGap = 0 as FWORD;
    (*pifi).fwdAveCharWidth = 1085 as FWORD; // FIXME
    (*pifi).fwdMaxCharInc = (*ftface).max_advance_width as FWORD;
    (*pifi).fwdCapHeight =
        ((*pifi).fwdUnitsPerEm as ::core::ffi::c_int / 2 as ::core::ffi::c_int) as FWORD;
    (*pifi).fwdXHeight =
        ((*pifi).fwdUnitsPerEm as ::core::ffi::c_int / 4 as ::core::ffi::c_int) as FWORD;
    (*pifi).fwdSubscriptXSize = 0 as FWORD;
    (*pifi).fwdSubscriptYSize = 0 as FWORD;
    (*pifi).fwdSubscriptXOffset = 0 as FWORD;
    (*pifi).fwdSubscriptYOffset = 0 as FWORD;
    (*pifi).fwdSuperscriptXSize = 0 as FWORD;
    (*pifi).fwdSuperscriptYSize = 0 as FWORD;
    (*pifi).fwdSuperscriptXOffset = 0 as FWORD;
    (*pifi).fwdSuperscriptYOffset = 0 as FWORD;
    (*pifi).fwdUnderscoreSize = 1 as FWORD;
    (*pifi).fwdUnderscorePosition = (-(1 as ::core::ffi::c_int)) as FWORD;
    (*pifi).fwdStrikeoutSize = 1 as FWORD;
    (*pifi).fwdStrikeoutPosition =
        ((*pifi).fwdXHeight as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as FWORD;

    (*pifi).ptlBaseline.x = 1;
    (*pifi).ptlBaseline.y = 0;
    (*pifi).ptlAspect.x = 1;
    (*pifi).ptlAspect.y = 1;
    (*pifi).ptlCaret.x = 0;
    (*pifi).ptlCaret.y = 1;

    /* Set the biggest characters bounding box */
    (*pifi).rclFontBox.left = (*ftface).bbox.xMin as LONG;
    (*pifi).rclFontBox.right = (*ftface).bbox.xMax as LONG;
    (*pifi).rclFontBox.top = (*ftface).bbox.yMax as LONG;
    (*pifi).rclFontBox.bottom = (*ftface).bbox.yMin as LONG;

    /* Special characters */
    (*pifi).chFirstChar = 0x1c as BYTE; // FIXME
    (*pifi).chLastChar = 0x79 as BYTE;
    (*pifi).chDefaultChar = 0x1d as BYTE;
    (*pifi).chBreakChar = 0x1e as BYTE;
    (*pifi).wcFirstChar = 0x1e as WCHAR;
    (*pifi).wcLastChar = 0x79 as WCHAR;
    (*pifi).wcDefaultChar = 0x1d as WCHAR;
    (*pifi).wcBreakChar = 0x1e as WCHAR;

    *(core::ptr::addr_of_mut!((*pifi).achVendId) as *mut DWORD) = 0x30303030 as DWORD; // FIXME
    (*pifi).cKerningPairs = 0 as ULONG;
    (*pifi).ulPanoseCulture = FM_PANOSE_CULTURE_LATIN as ULONG;
    //    pifi->panose = panose;

    EngMultiByteToUnicodeN(
        core::ptr::addr_of_mut!((*pifiX).wszFamilyName) as LPWSTR,
        LF_FACESIZE as ULONG,
        core::ptr::null_mut::<ULONG>(),
        (*ftface).family_name as PCHAR,
        strnlen((*ftface).family_name, MAX_PATH as size_t) as ULONG,
    );

    EngMultiByteToUnicodeN(
        core::ptr::addr_of_mut!((*pifiX).wszStyleName) as LPWSTR,
        LF_FACESIZE as ULONG,
        core::ptr::null_mut::<ULONG>(),
        (*ftface).style_name as PCHAR,
        strnlen((*ftface).style_name, MAX_PATH as size_t) as ULONG,
    );

    EngMultiByteToUnicodeN(
        core::ptr::addr_of_mut!((*pifiX).wszFaceName) as LPWSTR,
        LF_FACESIZE as ULONG,
        core::ptr::null_mut::<ULONG>(),
        (*ftface).family_name as PCHAR,
        strnlen((*ftface).family_name, MAX_PATH as size_t) as ULONG,
    );

    FT_Done_Face(ftface);

    FT_Message(
        b"Finished with the ifi: %p\n\0".as_ptr() as *const i8,
        pifiX,
    );
    __debugbreak();

    return pifi;
}

#[no_mangle]
pub unsafe extern "system" fn FtfdQueryFontCaps(culCaps: ULONG, pulCaps: *mut ULONG) -> LONG {
    FT_Message(b"FtfdQueryFontCaps()\n\0".as_ptr() as *const i8);

    /* We need room for 2 ULONGs */
    if culCaps < 2 {
        return (FD_ERROR) as LONG;
    }

    /* We only support 1 bpp */
    *pulCaps.offset(0 as isize) = 2;
    *pulCaps.offset(1 as isize) = QC_1BIT;

    return 2;
}

#[no_mangle]
pub unsafe extern "system" fn FtfdQueryFontTree(dhpdev: DHPDEV, iFile: ULONG_PTR, iFace: ULONG, iMode: ULONG, pid: *mut ULONG_PTR) -> PVOID {
    let mut pfile: PFTFD_FILE = iFile as PFTFD_FILE;
    let mut ftface: FT_Face = core::ptr::null_mut();
    let mut fterror: FT_Error = 0;
    let mut pcp: *mut FTFD_CHARPAIR = core::ptr::null_mut();
    let mut pGlyphSet: *mut FD_GLYPHSET = core::ptr::null_mut();
    let mut charcode: FT_ULong = 0;
    let mut i: ULONG = 0;
    let mut j: ULONG = 0;
    let mut cGlyphs: ULONG = 0;
    let mut cRuns: ULONG = 0;
    let mut cjSize: ULONG = 0;
    let mut pwcrun: *mut WCRUN = core::ptr::null_mut();
    let mut phglyphs: *mut HGLYPH = core::ptr::null_mut();

    FT_Message(b"FtfdQueryFontTree()\n\0".as_ptr() as *const i8);

    fterror = FT_New_Memory_Face(
        gftlibrary,
        (*pfile).pvView as *const FT_Byte,
        (*pfile).cjView as FT_Long,
        iFace.wrapping_sub(1 as ULONG) as FT_Long,
        core::ptr::addr_of_mut!(ftface),
    );
    if fterror != 0 {
        FT_Message(b"FT_New_Memory_Face() failed.\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    /* Get initial value for cGlyphs from ftface */
    cGlyphs = ((*ftface).num_glyphs + 1 as FT_Long) as ULONG;

    /* Allocate a buffer for the char codes and glyph indexes */
    pcp = EngAllocMem(
        0 as ULONG,
        ((cGlyphs as usize).wrapping_mul(core::mem::size_of::<FTFD_CHARPAIR>() as usize)) as ULONG,
        1885564960u32 as ULONG, // 'pcp '
    ) as *mut FTFD_CHARPAIR;
    if pcp.is_null() {
        FT_Message(b"EngAllocMem() failed.\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    /* Gather char codes and indexes and count WCRUNs */
    (*pcp.offset(0 as isize)).code = FT_Get_First_Char(
        ftface,
        core::ptr::addr_of_mut!((*pcp.offset(0 as isize)).index),
    );
    charcode = (*pcp.offset(0 as isize)).code;
    i = 1 as ULONG;
    cRuns = 1 as ULONG;
    while charcode != 0 && i < cGlyphs {
        charcode = FT_Get_Next_Char(
            ftface,
            charcode,
            core::ptr::addr_of_mut!((*pcp.offset(i as isize)).index),
        );
        FT_Message(
            b"charcode=0x%lx, index=0x%lx\n\0".as_ptr() as *const i8,
            charcode,
            (*pcp.offset(i as isize)).index,
        );
        (*pcp.offset(i as isize)).code = charcode;
        if charcode
            != (*pcp.offset(i.wrapping_sub(1 as ULONG) as isize))
                .code
                .wrapping_add(1 as FT_ULong)
        {
            cRuns = cRuns.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    /* Update cGlyphs to real value */
    cGlyphs = i.wrapping_sub(1 as ULONG);

    /* Calculate FD_GLYPHSET size */
    cjSize = (core::mem::size_of::<FD_GLYPHSET>() as usize)
        .wrapping_add(
            (cRuns.wrapping_sub(1 as ULONG) as usize)
                .wrapping_mul(core::mem::size_of::<WCRUN>() as usize),
        )
        .wrapping_add((cGlyphs as usize).wrapping_mul(core::mem::size_of::<HGLYPH>() as usize))
        as ULONG;

    /* Allocate the FD_GLYPHSET structure */
    pGlyphSet = EngAllocMem(
        0 as ULONG,
        cjSize,
        TAG_GLYPHSET as ULONG,
    ) as *mut FD_GLYPHSET;
    if pGlyphSet.is_null() {
        FT_Message(b"EngAllocMem() failed.\n\0".as_ptr() as *const i8);
        EngFreeMem(pcp as PVOID);
        return core::ptr::null_mut();
    }

    /* Initialize FD_GLYPHSET */
    (*pGlyphSet).cjThis = cjSize;
    (*pGlyphSet).flAccel = 0 as FLONG;
    (*pGlyphSet).cGlyphsSupported = cGlyphs;
    (*pGlyphSet).cRuns = cRuns;

    /* Initialize 1st WCRUN */
    pwcrun = core::ptr::addr_of_mut!((*pGlyphSet).awcrun) as *mut WCRUN;
    phglyphs = (core::ptr::addr_of_mut!((*pGlyphSet).awcrun) as *mut WCRUN)
        .offset(cRuns as isize) as *mut HGLYPH;
    (*pwcrun.offset(0 as isize)).wcLow =
        (*pcp.offset(0 as isize)).code as WCHAR;
    (*pwcrun.offset(0 as isize)).cGlyphs = 1 as USHORT;
    (*pwcrun.offset(0 as isize)).phg =
        phglyphs.offset(0 as isize);
    *phglyphs.offset(0 as isize) =
        (*pcp.offset(0 as isize)).index as HGLYPH;

    FT_Message(
        b"pcp[0].index = 0x%lx\n\0".as_ptr() as *const i8,
        (*pcp.offset(0 as isize)).index,
    );

    /* Walk through all supported chars */
    i = 1 as ULONG;
    j = 0 as ULONG;
    while i < cGlyphs {
        /* Use glyph index as HGLYPH */
        *phglyphs.offset(i as isize) = (*pcp.offset(i as isize)).index as HGLYPH;

        /* Check whether we can append the wchar to a run */
        if (*pcp.offset(i as isize)).code
            == (*pcp.offset(i.wrapping_sub(1 as ULONG) as isize))
                .code
                .wrapping_add(1 as FT_ULong)
        {
            /* Append to current WCRUN */
            (*pwcrun.offset(j as isize)).cGlyphs =
                (*pwcrun.offset(j as isize)).cGlyphs.wrapping_add(1);
        } else {
            /* Add a new WCRUN */
            FT_Message(b"adding new run\n\0".as_ptr() as *const i8);
            j = j.wrapping_add(1);
            (*pwcrun.offset(j as isize)).wcLow = (*pcp.offset(i as isize)).code as WCHAR;
            (*pwcrun.offset(j as isize)).cGlyphs = 1 as USHORT;
            (*pwcrun.offset(j as isize)).phg = phglyphs.offset(i as isize);
        }
        i = i.wrapping_add(1);
    }

    /* Free the temporary buffer */
    EngFreeMem(pcp as PVOID);

    /* Set *pid to the allocated structure for use in FtfdFree */
    *pid = pGlyphSet as ULONG_PTR;

    FT_Message(
        b"pGlyphSet=%p\n\0".as_ptr() as *const i8,
        pGlyphSet,
    );
    __debugbreak();

    return pGlyphSet as PVOID;
}
#[no_mangle]
pub unsafe extern "system" fn FtfdFree(pv: PVOID, id: ULONG_PTR) {
    FT_Message(b"FtfdFree()\n\0" as *const u8 as *const ::core::ffi::c_char);
    if id != 0 {
        EngFreeMem(id as PVOID);
    }
}
