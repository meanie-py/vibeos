/* dibobj.split2.c — #include fragment of dibobj.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original dibobj.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


_Success_(return!=0)
__kernel_entry
INT
APIENTRY
NtGdiGetDIBitsInternal(
    _In_ HDC hdc,
    _In_ HBITMAP hbm,
    _In_ UINT iStartScan,
    _In_ UINT cScans,
    _Out_writes_bytes_opt_(cjMaxBits) LPBYTE pjBits,
    _Inout_ LPBITMAPINFO pbmi,
    _In_ UINT iUsage,
    _In_ UINT cjMaxBits,
    _In_ UINT cjMaxInfo)
{
    PBITMAPINFO pbmiSafe;
    HANDLE hSecure = NULL;
    INT iResult = 0;
    UINT cjAlloc;

    /* Check for bad iUsage */
    if (iUsage > 2) return 0;

    /* Check if the size of the bitmap info is large enough */
    if (cjMaxInfo < sizeof(BITMAPCOREHEADER))
    {
        return 0;
    }

    /* Use maximum size */
    cjMaxInfo = min(cjMaxInfo, sizeof(BITMAPV5HEADER) + 256 * sizeof(RGBQUAD));

    // HACK: the underlying code sucks and doesn't care for the size, so we
    // give it the maximum ever needed
    cjAlloc = sizeof(BITMAPV5HEADER) + 256 * sizeof(RGBQUAD);

    /* Allocate a buffer the bitmapinfo */
    pbmiSafe = ExAllocatePoolWithTag(PagedPool, cjAlloc, 'imBG');
    if (!pbmiSafe)
    {
        /* Fail */
        return 0;
    }

    /* Use SEH */
    {
        NTSTATUS __SehStatus = SafeProbeForRead(pbmi, cjMaxInfo, 1);
        if (!NT_SUCCESS(__SehStatus))
        {
            goto cleanup;
        }
    }
    /* Probe and copy the BITMAPINFO */

    RtlCopyMemory(pbmiSafe, pbmi, cjMaxInfo);

    /* Check if the header size is large enough */
    if ((pbmiSafe->bmiHeader.biSize < sizeof(BITMAPCOREHEADER)) ||
        (pbmiSafe->bmiHeader.biSize > cjMaxInfo))
    {
        goto cleanup;
    }

    /* Check if the caller provided bitmap bits */
    if (pjBits)
    {
        /* Secure the user mode memory */
        hSecure = EngSecureMem(pjBits, cjMaxBits);
        if (!hSecure)
        {
            goto cleanup;
        }
    }

    /* Now call the internal function */
    iResult = GreGetDIBitsInternal(hdc,
                                   hbm,
                                   iStartScan,
                                   cScans,
                                   pjBits,
                                   pbmiSafe,
                                   iUsage,
                                   cjMaxBits,
                                   cjMaxInfo);

    /* Check for success */
    if (iResult)
    {
        /* Use SEH to copy back to user mode */
        _SEH2_TRY
        {
            /* Copy the data back */
            cjMaxInfo = min(cjMaxInfo, (UINT)DIB_BitmapInfoSize(pbmiSafe, (WORD)iUsage));
            ProbeForWrite(pbmi, cjMaxInfo, 1);
            RtlCopyMemory(pbmi, pbmiSafe, cjMaxInfo);
        }
        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        {
            /* Ignore */
            (VOID)0;
        }
        _SEH2_END;
    }

cleanup:
    if (hSecure) EngUnsecureMem(hSecure);
    ExFreePoolWithTag(pbmiSafe, 'imBG');

    return iResult;
}

W32KAPI
INT
APIENTRY
NtGdiStretchDIBitsInternal(
    IN HDC hdc,
    IN INT xDst,
    IN INT yDst,
    IN INT cxDst,
    IN INT cyDst,
    IN INT xSrc,
    IN INT ySrc,
    IN INT cxSrc,
    IN INT cySrc,
    IN OPTIONAL LPBYTE pjInit,
    IN LPBITMAPINFO pbmi,
    IN DWORD dwUsage,
    IN DWORD dwRop, // MS ntgdi.h says dwRop4(?)
    IN UINT cjMaxInfo,
    IN UINT cjMaxBits,
    IN HANDLE hcmXform)
{
    SIZEL sizel;
    RECTL rcSrc, rcDst;
    PDC pdc;
    HBITMAP hbmTmp = 0;
    PSURFACE psurfTmp = 0, psurfDst = 0;
    PPALETTE ppalDIB = 0;
    EXLATEOBJ exlo;
    PBYTE pvBits;

    LPBITMAPINFO pbmiSafe;
    UINT cjAlloc;
    HBITMAP hBitmap, hOldBitmap = NULL;
    HDC hdcMem;
    HPALETTE hPal = NULL;
    ULONG BmpFormat = 0;
    INT LinesCopied = 0;

    /* Check for bad iUsage */
    if (dwUsage > 2) return 0;

    /* We must have LPBITMAPINFO */
    if (!pbmi)
    {
        DPRINT1("Error, Invalid Parameter.\n");
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Check if the size of the bitmap info is large enough */
    if (cjMaxInfo < sizeof(BITMAPCOREHEADER))
    {
        return 0;
    }

    /* Use maximum size */
    cjMaxInfo = min(cjMaxInfo, sizeof(BITMAPV5HEADER) + 256 * sizeof(RGBQUAD));

    // HACK: the underlying code sucks and doesn't care for the size, so we
    // give it the maximum ever needed
    cjAlloc = sizeof(BITMAPV5HEADER) + 256 * sizeof(RGBQUAD);

    /* Allocate a buffer the bitmapinfo */
    pbmiSafe = ExAllocatePoolWithTag(PagedPool, cjAlloc, 'imBG');
    if (!pbmiSafe)
    {
        /* Fail */
        return 0;
    }

    /* Use SEH */
    {
        NTSTATUS __SehStatus = SafeProbeForRead(pbmi, cjMaxInfo, 1);
        if (!NT_SUCCESS(__SehStatus))
        {
            ExFreePoolWithTag(pbmiSafe, 'imBG');
            return 0;
        }
    }
    /* Probe and copy the BITMAPINFO */

    RtlCopyMemory(pbmiSafe, pbmi, cjMaxInfo);

    /* Check if the header size is large enough */
    if ((pbmiSafe->bmiHeader.biSize < sizeof(BITMAPCOREHEADER)) ||
        (pbmiSafe->bmiHeader.biSize > cjMaxInfo))
    {
        ExFreePoolWithTag(pbmiSafe, 'imBG');
        return 0;
    }

    if (!(pdc = DC_LockDc(hdc)))
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    /* Check for info / mem DC without surface */
    if (!pdc->dclevel.pSurface)
    {
        DC_UnlockDc(pdc);
        // CHECKME
        return TRUE;
    }

    /* Transform dest size */
    sizel.cx = cxDst;
    sizel.cy = cyDst;
    IntLPtoDP(pdc, (POINTL*)&sizel, 1);
    DC_UnlockDc(pdc);

    if (pjInit && (cjMaxBits > 0))
    {
        pvBits = ExAllocatePoolWithTag(PagedPool, cjMaxBits, TAG_DIB);
        if (!pvBits)
        {
            return 0;
        }

        {
            NTSTATUS __SehStatus = SafeProbeForRead(pjInit, cjMaxBits, 1);
            if (!NT_SUCCESS(__SehStatus))
            {
                ExFreePoolWithTag(pvBits, TAG_DIB);
                return 0;
            }
        }
        RtlCopyMemory(pvBits, pjInit, cjMaxBits);
    }
    else
    {
        pvBits = NULL;
    }

    /* Here we select between the dwRop with SRCCOPY or not. */
    if (dwRop == SRCCOPY)
    {
        hdcMem = NtGdiCreateCompatibleDC(hdc);
        if (hdcMem == NULL)
        {
            DPRINT1("NtGdiCreateCompatibleDC failed to create hdc.\n");
            EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
            return 0;
        }

        hBitmap = NtGdiCreateCompatibleBitmap(hdc,
                                              abs(pbmiSafe->bmiHeader.biWidth),
                                              abs(pbmiSafe->bmiHeader.biHeight));
        if (hBitmap == NULL)
        {
            DPRINT1("NtGdiCreateCompatibleBitmap failed to create bitmap.\n");
            DPRINT1("hdc : 0x%08x \n", hdc);
            DPRINT1("width : 0x%08x \n", pbmiSafe->bmiHeader.biWidth);
            DPRINT1("height : 0x%08x \n", pbmiSafe->bmiHeader.biHeight);
            EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
            return 0;
        }

        /* Select the bitmap into hdcMem, and save a handle to the old bitmap */
        hOldBitmap = NtGdiSelectBitmap(hdcMem, hBitmap);

        if (dwUsage == DIB_PAL_COLORS)
        {
            hPal = NtGdiGetDCObject(hdc, GDI_OBJECT_TYPE_PALETTE);
            hPal = GdiSelectPalette(hdcMem, hPal, FALSE);
        }

        pdc = DC_LockDc(hdcMem);
        if (pdc != NULL)
        {
            IntSetDIBits(pdc, hBitmap, 0, abs(pbmiSafe->bmiHeader.biHeight), pvBits,
                         cjMaxBits, pbmiSafe, dwUsage);
            DC_UnlockDc(pdc);
        }

        /* Origin for DIBitmap may be bottom left (positive biHeight) or top
           left (negative biHeight) */
        if (cxSrc == cxDst && cySrc == cyDst)
        {
            NtGdiBitBlt(hdc, xDst, yDst, cxDst, cyDst,
                        hdcMem, xSrc, abs(pbmiSafe->bmiHeader.biHeight) - cySrc - ySrc,
                        dwRop, CLR_INVALID, 0);
        }
        else
        {
            NtGdiStretchBlt(hdc, xDst, yDst, cxDst, cyDst,
                            hdcMem, xSrc, abs(pbmiSafe->bmiHeader.biHeight) - cySrc - ySrc,
                            cxSrc, cySrc, dwRop, CLR_INVALID);
        }

        /* cleanup */
        if (hPal)
            GdiSelectPalette(hdcMem, hPal, FALSE);

        if (hOldBitmap)
            NtGdiSelectBitmap(hdcMem, hOldBitmap);

        NtGdiDeleteObjectApp(hdcMem);
        GreDeleteObject(hBitmap);

    } /* End of dwRop == SRCCOPY */
    else
    { /* Start of dwRop != SRCCOPY */
        /* FIXME: Locking twice is cheesy, coord tranlation in UM will fix it */
        if (!(pdc = DC_LockDc(hdc)))
        {
            DPRINT1("Could not lock dc\n");
            EngSetLastError(ERROR_INVALID_HANDLE);
            goto cleanup;
        }

        /* Calculate source and destination rect */
        rcSrc.left = xSrc;
        rcSrc.top = ySrc;
        rcSrc.right = xSrc + abs(cxSrc);
        rcSrc.bottom = ySrc + abs(cySrc);
        rcDst.left = xDst;
        rcDst.top = yDst;
        rcDst.right = rcDst.left + cxDst;
        rcDst.bottom = rcDst.top + cyDst;
        IntLPtoDP(pdc, (POINTL*)&rcDst, 2);
        RECTL_vOffsetRect(&rcDst, pdc->ptlDCOrig.x, pdc->ptlDCOrig.y);

        if (pdc->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
        {
           IntUpdateBoundsRect(pdc, &rcDst);
        }

        BmpFormat = BitmapFormat(pbmiSafe->bmiHeader.biBitCount,
                                 pbmiSafe->bmiHeader.biCompression);

        hbmTmp = GreCreateBitmapEx(pbmiSafe->bmiHeader.biWidth,
                                   abs(pbmiSafe->bmiHeader.biHeight),
                                   0,
                                   BmpFormat,
                                   pbmiSafe->bmiHeader.biHeight < 0 ? BMF_TOPDOWN : 0,
                                   cjMaxBits,
                                   pvBits,
                                   0);

        if (!hbmTmp)
        {
            goto cleanup;
        }

        psurfTmp = SURFACE_ShareLockSurface(hbmTmp);
        if (!psurfTmp)
        {
            goto cleanup;
        }

        /* Create a palette for the DIB */
        ppalDIB = CreateDIBPalette(pbmiSafe, pdc, dwUsage);
        if (!ppalDIB)
        {
            goto cleanup;
        }

        /* Prepare DC for blit */
        DC_vPrepareDCsForBlit(pdc, &rcDst, NULL, NULL);

        psurfDst = pdc->dclevel.pSurface;

        /* Initialize XLATEOBJ */
        EXLATEOBJ_vInitialize(&exlo,
                              ppalDIB,
                              psurfDst->ppal,
                              RGB(0xff, 0xff, 0xff),
                              pdc->pdcattr->crBackgroundClr,
                              pdc->pdcattr->crForegroundClr);

        /* Perform the stretch operation */
        IntEngStretchBlt(&psurfDst->SurfObj,
                         &psurfTmp->SurfObj,
                         NULL,
                         (CLIPOBJ *)&pdc->co,
                         &exlo.xlo,
                         &pdc->dclevel.ca,
                         &rcDst,
                         &rcSrc,
                         NULL,
                         &pdc->eboFill.BrushObject,
                         NULL,
                         WIN32_ROP3_TO_ENG_ROP4(dwRop));

        /* Cleanup */
        DC_vFinishBlit(pdc, NULL);
        EXLATEOBJ_vCleanup(&exlo);

    cleanup:
        if (ppalDIB) PALETTE_ShareUnlockPalette(ppalDIB);
        if (psurfTmp) SURFACE_ShareUnlockSurface(psurfTmp);
        if (hbmTmp) GreDeleteObject(hbmTmp);
        if (pdc) DC_UnlockDc(pdc);
    }

    if (pvBits) ExFreePoolWithTag(pvBits, TAG_DIB);

    /* This is not what MSDN says is returned from this function, but it
     * follows Wine's dlls/gdi32/dib.c function nulldrv_StretchDIBits
     * and it fixes over 100 gdi32:dib regression tests. */
    if (dwRop == SRCCOPY)
    {
        LinesCopied = abs(pbmiSafe->bmiHeader.biHeight);
    }
    else
    {
        LinesCopied = pbmiSafe->bmiHeader.biHeight;
    }

    ExFreePoolWithTag(pbmiSafe, 'imBG');

    return LinesCopied;
}

HBITMAP
FASTCALL
IntCreateDIBitmap(
    PDC Dc,
    INT width,
    INT height,
    UINT planes,
    UINT bpp,
    ULONG compression,
    DWORD init,
    LPBYTE bits,
    ULONG cjMaxBits,
    PBITMAPINFO data,
    DWORD coloruse)
{
    HBITMAP handle;
    BOOL fColor;
    ULONG BmpFormat = 0;

    if (planes && bpp)
        BmpFormat = BitmapFormat(planes * bpp, compression);

    // Check if we should create a monochrome or color bitmap. We create a monochrome bitmap only if it has exactly 2
    // colors, which are black followed by white, nothing else. In all other cases, we create a color bitmap.

    if (BmpFormat != BMF_1BPP) fColor = TRUE;
    else if ((coloruse > DIB_RGB_COLORS) || ((init & CBM_INIT) == 0) || !data) fColor = FALSE;
    else
    {
        const RGBQUAD *rgb = (RGBQUAD*)((PBYTE)data + data->bmiHeader.biSize);
        DWORD col = RGB(rgb->rgbRed, rgb->rgbGreen, rgb->rgbBlue);

        // Check if the first color of the colormap is black
        if (col == RGB(0, 0, 0))
        {
            rgb++;
            col = RGB(rgb->rgbRed, rgb->rgbGreen, rgb->rgbBlue);

            // If the second color is white, create a monochrome bitmap
            fColor = (col != RGB(0xff,0xff,0xff));
        }
        else fColor = TRUE;
    }

    // Now create the bitmap
    if (fColor)
    {
        if (init & CBM_CREATDIB)
        {
            PSURFACE Surface;
            PPALETTE Palette;

            /* Undocumented flag which creates a DDB of the format specified by the bitmap info. */
            handle = IntCreateCompatibleBitmap(Dc, width, height, planes, bpp);
            if (!handle)
            {
                DPRINT1("IntCreateCompatibleBitmap() failed!\n");
                return NULL;
            }

            /* The palette must also match the given data */
            Surface = SURFACE_ShareLockSurface(handle);
            ASSERT(Surface);
            Palette = CreateDIBPalette(data, Dc, coloruse);
            if (Palette == NULL)
            {
                SURFACE_ShareUnlockSurface(Surface);
                GreDeleteObject(handle);
                return NULL;
            }

            SURFACE_vSetPalette(Surface, Palette);

            PALETTE_ShareUnlockPalette(Palette);
            SURFACE_ShareUnlockSurface(Surface);
        }
        else
        {
            /* Create a regular compatible bitmap, in the same format as the device */
            handle = IntCreateCompatibleBitmap(Dc, width, height, 0, 0);
        }
    }
    else
    {
        handle = GreCreateBitmap(width,
                                 abs(height),
                                 1,
                                 1,
                                 NULL);
    }

    if (height < 0)
        height = -height;

    if ((NULL != handle) && (CBM_INIT & init))
    {
        IntSetDIBits(Dc, handle, 0, height, bits, cjMaxBits, data, coloruse);
    }

    return handle;
}

// The CreateDIBitmap function creates a device-dependent bitmap (DDB) from a DIB and, optionally, sets the bitmap bits
// The DDB that is created will be whatever bit depth your reference DC is
HBITMAP
APIENTRY
NtGdiCreateDIBitmapInternal(
    IN HDC hDc,
    IN INT cx,
    IN INT cy,
    IN DWORD fInit,
    IN OPTIONAL LPBYTE pjInit,
    IN OPTIONAL LPBITMAPINFO pbmi,
    IN DWORD iUsage,
    IN UINT cjMaxInitInfo,
    IN UINT cjMaxBits,
    IN FLONG fl,
    IN HANDLE hcmXform)
{
    NTSTATUS Status = STATUS_SUCCESS;
    PBYTE safeBits = NULL;
    HBITMAP hbmResult = NULL;

    if (pjInit == NULL)
    {
        fInit &= ~CBM_INIT;
    }

    if(pjInit && (fInit & CBM_INIT))
    {
        if (cjMaxBits == 0) return NULL;
        safeBits = ExAllocatePoolWithTag(PagedPool, cjMaxBits, TAG_DIB);
        if(!safeBits)
        {
            DPRINT1("Failed to allocate %lu bytes\n", cjMaxBits);
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return NULL;
        }
    }

    if (pbmi)
    {
        Status = SafeProbeForRead(pbmi, cjMaxInitInfo, 1);
    }
    if (NT_SUCCESS(Status) && pjInit && (fInit & CBM_INIT))
    {
        Status = SafeProbeForRead(pjInit, cjMaxBits, 1);
    }
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("Got an exception! pjInit = %p\n", pjInit);
        SetLastNtError(Status);
        goto cleanup;
    }
    else
    {
        if(pjInit && (fInit & CBM_INIT))
        {
            RtlCopyMemory(safeBits, pjInit, cjMaxBits);
        }
    }

    hbmResult =  GreCreateDIBitmapInternal(hDc,
                                           cx,
                                           cy,
                                           fInit,
                                           safeBits,
                                           pbmi,
                                           iUsage,
                                           fl,
                                           cjMaxBits,
                                           hcmXform);

cleanup:
    if (safeBits) ExFreePoolWithTag(safeBits, TAG_DIB);
    return hbmResult;
}


HBITMAP
NTAPI
GreCreateDIBitmapInternal(
    IN HDC hDc,
    IN INT cx,
    IN INT cy,
    IN DWORD fInit,
    IN OPTIONAL LPBYTE pjInit,
    IN OPTIONAL PBITMAPINFO pbmi,
    IN DWORD iUsage,
    IN FLONG fl,
    IN UINT cjMaxBits,
    IN HANDLE hcmXform)
{
    PDC Dc;
    HBITMAP Bmp;
    USHORT bpp, planes;
    DWORD compression;
    HDC hdcDest;

    if (!hDc) /* 1bpp monochrome bitmap */
    {
        // Should use System Bitmap DC hSystemBM, with CreateCompatibleDC for this.
        hdcDest = NtGdiCreateCompatibleDC(0);
        if(!hdcDest)
        {
            DPRINT1("NtGdiCreateCompatibleDC failed\n");
            return NULL;
        }
    }
    else
    {
        hdcDest = hDc;
    }

    Dc = DC_LockDc(hdcDest);
    if (!Dc)
    {
        DPRINT1("Failed to lock hdcDest %p\n", hdcDest);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return NULL;
    }
    /* It's OK to set bpp=0 here, as IntCreateDIBitmap will create a compatible Bitmap
     * if bpp != 1 and ignore the real value that was passed */
    if (pbmi)
    {
        if (pbmi->bmiHeader.biSize == sizeof(BITMAPCOREHEADER))
        {
            BITMAPCOREHEADER* CoreHeader = (BITMAPCOREHEADER*)&pbmi->bmiHeader;
            bpp = CoreHeader->bcBitCount;
            planes = CoreHeader->bcPlanes ? CoreHeader->bcPlanes : 1;
            compression = BI_RGB;
        }
        else
        {
            bpp = pbmi->bmiHeader.biBitCount;
            planes = pbmi->bmiHeader.biPlanes ? pbmi->bmiHeader.biPlanes : 1;
            compression = pbmi->bmiHeader.biCompression;
        }
    }
    else
    {
        bpp = 0;
        planes = 0;
        compression = 0;
    }
    Bmp = IntCreateDIBitmap(Dc, cx, cy, planes, bpp, compression, fInit, pjInit, cjMaxBits, pbmi, iUsage);
    DC_UnlockDc(Dc);

    if(!hDc)
    {
        NtGdiDeleteObjectApp(hdcDest);
    }
    return Bmp;
}

HBITMAP
NTAPI
GreCreateDIBitmapFromPackedDIB(
    _In_reads_(cjPackedDIB )PVOID pvPackedDIB,
    _In_ UINT cjPackedDIB,
    _In_ ULONG uUsage)
{
    PBITMAPINFO pbmi;
    PBYTE pjBits;
    UINT cjInfo, cjBits;
    HBITMAP hbm;

    /* We only support BITMAPINFOHEADER, make sure the size is ok */
    if (cjPackedDIB < sizeof(BITMAPINFOHEADER))
    {
        return NULL;
    }

    /* The packed DIB starts with the BITMAPINFOHEADER */
    pbmi = pvPackedDIB;

    if (cjPackedDIB < pbmi->bmiHeader.biSize)
    {
        return NULL;
    }

    /* Calculate the info size and make sure the packed DIB is large enough */
    cjInfo = DIB_BitmapInfoSize(pbmi, uUsage);
    if (cjPackedDIB <= cjInfo)
    {
        return NULL;
    }

    /* The bitmap bits start after the header */
    pjBits = (PBYTE)pvPackedDIB + cjInfo;
    cjBits = cjPackedDIB - cjInfo;

    hbm = GreCreateDIBitmapInternal(NULL,
                                    pbmi->bmiHeader.biWidth,
                                    abs(pbmi->bmiHeader.biHeight),
                                    CBM_INIT | CBM_CREATDIB,
                                    pjBits,
                                    pbmi,
                                    uUsage,
                                    0,
                                    cjBits,
                                    NULL);

    return hbm;
}

HBITMAP
APIENTRY
NtGdiCreateDIBSection(
    IN HDC hDC,
    IN OPTIONAL HANDLE hSection,
    IN DWORD dwOffset,
    IN BITMAPINFO* bmi,
    IN DWORD Usage,
    IN UINT cjHeader,
    IN FLONG fl,
    IN ULONG_PTR dwColorSpace,
    OUT PVOID *Bits)
{
    HBITMAP hbitmap = 0;
    DC *dc;
    BOOL bDesktopDC = FALSE;
    NTSTATUS Status = STATUS_SUCCESS;

    if (!bmi) return hbitmap; // Make sure.

    _SEH2_TRY
    {
        ProbeForRead(&bmi->bmiHeader.biSize, sizeof(DWORD), 1);
        ProbeForRead(bmi, bmi->bmiHeader.biSize, 1);
        ProbeForRead(bmi, DIB_BitmapInfoSize(bmi, (WORD)Usage), 1);
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        Status = _SEH2_GetExceptionCode();
    }
    _SEH2_END;

    if(!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        return NULL;
    }

    // If the reference hdc is null, take the desktop dc
    if (hDC == 0)
    {
        hDC = NtGdiCreateCompatibleDC(0);
        bDesktopDC = TRUE;
    }

    if ((dc = DC_LockDc(hDC)))
    {
        hbitmap = DIB_CreateDIBSection(dc,
                                       bmi,
                                       Usage,
                                       Bits,
                                       hSection,
                                       dwOffset,
                                       0);
        DC_UnlockDc(dc);
    }
    else
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
    }

    if (bDesktopDC)
        NtGdiDeleteObjectApp(hDC);

    return hbitmap;
}

HBITMAP
APIENTRY
DIB_CreateDIBSection(
    PDC dc,
    CONST BITMAPINFO *bmi,
    UINT usage,
    LPVOID *bits,
    HANDLE section,
    DWORD offset,
    DWORD ovr_pitch)
{
    HBITMAP res = 0;
    SURFACE *bmp = NULL;
    void *mapBits = NULL;
    PPALETTE ppalDIB = NULL;

    // Fill BITMAP32 structure with DIB data
    CONST BITMAPINFOHEADER *bi = &bmi->bmiHeader;
    INT effHeight;
    ULONG totalSize;
    BITMAP bm;
    //SIZEL Size;
    HANDLE hSecure;

    DPRINT("format (%ld,%ld), planes %u, bpp %u, size %lu, colors %lu (%s)\n",
           bi->biWidth, bi->biHeight, bi->biPlanes, bi->biBitCount,
           bi->biSizeImage, bi->biClrUsed, usage == DIB_PAL_COLORS? "PAL" : "RGB");

    /* CreateDIBSection should fail for compressed formats */
    if (bi->biCompression == BI_RLE4 || bi->biCompression == BI_RLE8)
    {
        DPRINT1("no compressed format allowed\n");
        return (HBITMAP)NULL;
    }

    effHeight = bi->biHeight >= 0 ? bi->biHeight : -bi->biHeight;
    bm.bmType = 0;
    bm.bmWidth = bi->biWidth;
    bm.bmHeight = effHeight;
    bm.bmWidthBytes = ovr_pitch ? ovr_pitch : WIDTH_BYTES_ALIGN32(bm.bmWidth, bi->biBitCount);

    bm.bmPlanes = bi->biPlanes;
    bm.bmBitsPixel = bi->biBitCount;
    bm.bmBits = NULL;

    // Get storage location for DIB bits.  Only use biSizeImage if it's valid and
    // we're dealing with a compressed bitmap.  Otherwise, use width * height.
    totalSize = (bi->biSizeImage && (bi->biCompression != BI_RGB) && (bi->biCompression != BI_BITFIELDS))
                ? bi->biSizeImage : (ULONG)(bm.bmWidthBytes * effHeight);

    if (section)
    {
        SYSTEM_BASIC_INFORMATION Sbi;
        NTSTATUS Status;
        DWORD mapOffset;
        LARGE_INTEGER SectionOffset;
        SIZE_T mapSize;

        Status = ZwQuerySystemInformation(SystemBasicInformation,
                                          &Sbi,
                                          sizeof Sbi,
                                          0);
        if (!NT_SUCCESS(Status))
        {
            DPRINT1("ZwQuerySystemInformation failed (0x%lx)\n", Status);
            return NULL;
        }

        mapOffset = offset - (offset % Sbi.AllocationGranularity);
        mapSize = totalSize + (offset - mapOffset);

        SectionOffset.LowPart  = mapOffset;
        SectionOffset.HighPart = 0;

        Status = ZwMapViewOfSection(section,
                                    NtCurrentProcess(),
                                    &mapBits,
                                    0,
                                    0,
                                    &SectionOffset,
                                    &mapSize,
                                    ViewShare,
                                    0,
                                    PAGE_READWRITE);
        if (!NT_SUCCESS(Status))
        {
            DPRINT1("ZwMapViewOfSection failed (0x%lx)\n", Status);
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return NULL;
        }

        if (mapBits) bm.bmBits = (char *)mapBits + (offset - mapOffset);
    }
    else if (ovr_pitch && offset)
        bm.bmBits = UlongToPtr(offset);
    else
    {
        offset = 0;
        bm.bmBits = EngAllocUserMem(totalSize, 0);
        if(!bm.bmBits)
        {
            DPRINT1("Failed to allocate memory\n");
            goto cleanup;
        }
    }

//  hSecure = MmSecureVirtualMemory(bm.bmBits, totalSize, PAGE_READWRITE);
    hSecure = (HANDLE)0x1; // HACK OF UNIMPLEMENTED KERNEL STUFF !!!!

    // Create Device Dependent Bitmap and add DIB pointer
    //Size.cx = bm.bmWidth;
    //Size.cy = abs(bm.bmHeight);
    res = GreCreateBitmapEx(bm.bmWidth,
                            abs(bm.bmHeight),
                            bm.bmWidthBytes,
                            BitmapFormat(bi->biBitCount * bi->biPlanes, bi->biCompression),
                            BMF_DONTCACHE | BMF_USERMEM | BMF_NOZEROINIT |
                            ((bi->biHeight < 0) ? BMF_TOPDOWN : 0),
                            totalSize,
                            bm.bmBits,
                            0);
    if (!res)
    {
        DPRINT1("GreCreateBitmapEx failed\n");
        EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
        goto cleanup;
    }
    bmp = SURFACE_ShareLockSurface(res); // HACK
    if (NULL == bmp)
    {
        DPRINT1("SURFACE_LockSurface failed\n");
        EngSetLastError(ERROR_INVALID_HANDLE);
        goto cleanup;
    }

    /* WINE NOTE: WINE makes use of a colormap, which is a color translation
                  table between the DIB and the X physical device. Obviously,
                  this is left out of the ReactOS implementation. Instead,
                  we call NtGdiSetDIBColorTable. */
    bmp->hDIBSection = section;
    bmp->hSecure = hSecure;
    bmp->dwOffset = offset;
    bmp->flags = API_BITMAP;
    bmp->biClrImportant = bi->biClrImportant;

    /* Create a palette for the DIB */
    ppalDIB = CreateDIBPalette(bmi, dc, usage);

    // Clean up in case of errors
cleanup:
    if (!res || !bmp || !bm.bmBits || !ppalDIB)
    {
        DPRINT("Got an error res=%p, bmp=%p, bm.bmBits=%p\n", res, bmp, bm.bmBits);
        if (bm.bmBits)
        {
            // MmUnsecureVirtualMemory(hSecure); // FIXME: Implement this!
            if (section)
            {
                ZwUnmapViewOfSection(NtCurrentProcess(), mapBits);
                bm.bmBits = NULL;
            }
            else if (!offset)
                EngFreeUserMem(bm.bmBits), bm.bmBits = NULL;
        }

        if (bmp)
        {
            SURFACE_ShareUnlockSurface(bmp);
            bmp = NULL;
        }

        if (res)
        {
            GreDeleteObject(res);
            res = 0;
        }

        if(ppalDIB)
        {
            PALETTE_ShareUnlockPalette(ppalDIB);
        }
    }

    if (bmp)
    {
        /* If we're here, everything went fine */
        SURFACE_vSetPalette(bmp, ppalDIB);
        PALETTE_ShareUnlockPalette(ppalDIB);
        SURFACE_ShareUnlockSurface(bmp);
    }

    // Return BITMAP handle and storage location
    if (NULL != bm.bmBits && NULL != bits)
    {
        *bits = bm.bmBits;
    }

    return res;
}

/***********************************************************************
 *           DIB_GetBitmapInfo
 *
 * Get the info from a bitmap header.
 * Return 0 for COREHEADER, 1 for INFOHEADER, -1 for error.
 */
int
FASTCALL
DIB_GetBitmapInfo( const BITMAPINFOHEADER *header, LONG *width,
                   LONG *height, WORD *planes, WORD *bpp, DWORD *compr, DWORD *size )
{
    if (header->biSize == sizeof(BITMAPCOREHEADER))
    {
        const BITMAPCOREHEADER *core = (const BITMAPCOREHEADER *)header;
        *width  = core->bcWidth;
        *height = core->bcHeight;
        *planes = core->bcPlanes;
        *bpp    = core->bcBitCount;
        *compr  = BI_RGB;
        *size   = 0;
        return 0;
    }
    if (header->biSize >= sizeof(BITMAPINFOHEADER)) /* Assume BITMAPINFOHEADER */
    {
        *width  = header->biWidth;
        *height = header->biHeight;
        *planes = header->biPlanes;
        *bpp    = header->biBitCount;
        *compr  = header->biCompression;
        *size   = header->biSizeImage;
        return 1;
    }
    DPRINT1("(%u): unknown/wrong size for header\n", header->biSize );
    return -1;
}

/***********************************************************************
 *           DIB_GetDIBImageBytes
 *
 * Return the number of bytes used to hold the image in a DIB bitmap.
 * 11/16/1999 (RJJ) lifted from wine
 */

INT APIENTRY DIB_GetDIBImageBytes(INT  width, INT height, INT depth)
{
    return WIDTH_BYTES_ALIGN32(width, depth) * (height < 0 ? -height : height);
}

/***********************************************************************
 *           DIB_BitmapInfoSize
 *
 * Return the size of the bitmap info structure including color table.
 * 11/16/1999 (RJJ) lifted from wine
 */

INT FASTCALL DIB_BitmapInfoSize(const BITMAPINFO * info, WORD coloruse)
{
    unsigned int colors, size, masks = 0;
    unsigned int colorsize;

    colorsize = (coloruse == DIB_RGB_COLORS) ? sizeof(RGBQUAD) :
                (coloruse == DIB_PAL_INDICES) ? 0 :
                sizeof(WORD);

    if (info->bmiHeader.biSize == sizeof(BITMAPCOREHEADER))
    {
        const BITMAPCOREHEADER *core = (const BITMAPCOREHEADER *)info;
        colors = (core->bcBitCount <= 8) ? 1 << core->bcBitCount : 0;
        return sizeof(BITMAPCOREHEADER) + colors * colorsize;
    }
    else  /* Assume BITMAPINFOHEADER */
    {
        colors = info->bmiHeader.biClrUsed;
        if (colors > 256) colors = 256;
        if (!colors && (info->bmiHeader.biBitCount <= 8))
            colors = 1 << info->bmiHeader.biBitCount;
        if (info->bmiHeader.biCompression == BI_BITFIELDS) masks = 3;
        size = max( info->bmiHeader.biSize, sizeof(BITMAPINFOHEADER) + masks * sizeof(DWORD) );
        return size + colors * colorsize;
    }
}

HPALETTE
FASTCALL
DIB_MapPaletteColors(PPALETTE ppalDc, CONST BITMAPINFO* lpbmi)
{
    PPALETTE ppalNew;
    ULONG nNumColors,i;
    USHORT *lpIndex;
    HPALETTE hpal;

    if (!(ppalDc->flFlags & PAL_INDEXED))
    {
        return NULL;
    }

    nNumColors = 1 << lpbmi->bmiHeader.biBitCount;
    if (lpbmi->bmiHeader.biClrUsed)
    {
        nNumColors = min(nNumColors, lpbmi->bmiHeader.biClrUsed);
    }

    ppalNew = PALETTE_AllocPalWithHandle(PAL_INDEXED, nNumColors, NULL, 0, 0, 0);
    if (ppalNew == NULL)
    {
        DPRINT1("Could not allocate palette\n");
        return NULL;
    }

    lpIndex = (USHORT *)((PBYTE)lpbmi + lpbmi->bmiHeader.biSize);

    for (i = 0; i < nNumColors; i++)
    {
        ULONG iColorIndex = *lpIndex % ppalDc->NumColors;
        ppalNew->IndexedColors[i] = ppalDc->IndexedColors[iColorIndex];
        lpIndex++;
    }

    hpal = ppalNew->BaseObject.hHmgr;
    PALETTE_UnlockPalette(ppalNew);

    return hpal;
}

/* Converts a BITMAPCOREINFO to a BITMAPINFO structure,
 * or does nothing if it's already a BITMAPINFO (or V4 or V5) */
BITMAPINFO*
FASTCALL
DIB_ConvertBitmapInfo (CONST BITMAPINFO* pbmi, DWORD Usage)
{
    CONST BITMAPCOREINFO* pbmci = (BITMAPCOREINFO*)pbmi;
    BITMAPINFO* pNewBmi ;
    UINT numColors = 0, ColorsSize = 0;

    if(pbmi->bmiHeader.biSize >= sizeof(BITMAPINFOHEADER)) return (BITMAPINFO*)pbmi;
    if(pbmi->bmiHeader.biSize != sizeof(BITMAPCOREHEADER)) return NULL;

    if(pbmci->bmciHeader.bcBitCount <= 8)
    {
        numColors = 1 << pbmci->bmciHeader.bcBitCount;
        if(Usage == DIB_PAL_COLORS)
        {
            ColorsSize = numColors * sizeof(WORD);
        }
        else
        {
            ColorsSize = numColors * sizeof(RGBQUAD);
        }
    }
    else if (Usage == DIB_PAL_COLORS)
    {
        /* Invalid at high-res */
        return NULL;
    }

    pNewBmi = ExAllocatePoolWithTag(PagedPool, sizeof(BITMAPINFOHEADER) + ColorsSize, TAG_DIB);
    if(!pNewBmi) return NULL;

    RtlZeroMemory(pNewBmi, sizeof(BITMAPINFOHEADER) + ColorsSize);

    pNewBmi->bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    pNewBmi->bmiHeader.biBitCount = pbmci->bmciHeader.bcBitCount;
    pNewBmi->bmiHeader.biWidth = pbmci->bmciHeader.bcWidth;
    pNewBmi->bmiHeader.biHeight = pbmci->bmciHeader.bcHeight;
    pNewBmi->bmiHeader.biPlanes = pbmci->bmciHeader.bcPlanes;
    pNewBmi->bmiHeader.biCompression = BI_RGB ;
    pNewBmi->bmiHeader.biSizeImage = DIB_GetDIBImageBytes(pNewBmi->bmiHeader.biWidth,
                                     pNewBmi->bmiHeader.biHeight,
                                     pNewBmi->bmiHeader.biBitCount);
    pNewBmi->bmiHeader.biClrUsed = numColors;

    if(Usage == DIB_PAL_COLORS)
    {
        RtlCopyMemory(pNewBmi->bmiColors, pbmci->bmciColors, ColorsSize);
    }
    else
    {
        UINT i;
        for(i=0; i<numColors; i++)
        {
            pNewBmi->bmiColors[i].rgbRed = pbmci->bmciColors[i].rgbtRed;
            pNewBmi->bmiColors[i].rgbGreen = pbmci->bmciColors[i].rgbtGreen;
            pNewBmi->bmiColors[i].rgbBlue = pbmci->bmciColors[i].rgbtBlue;
        }
    }

    return pNewBmi ;
}

/* Frees a BITMAPINFO created with DIB_ConvertBitmapInfo */
VOID
FASTCALL
DIB_FreeConvertedBitmapInfo(BITMAPINFO* converted, BITMAPINFO* orig, DWORD usage)
{
    BITMAPCOREINFO* pbmci;
    if(converted == orig)
        return;

    if(usage == -1)
    {
        /* Caller don't want any conversion */
        ExFreePoolWithTag(converted, TAG_DIB);
        return;
    }

    /* Perform inverse conversion */
    pbmci = (BITMAPCOREINFO*)orig;

    ASSERT(pbmci->bmciHeader.bcSize == sizeof(BITMAPCOREHEADER));
    pbmci->bmciHeader.bcBitCount = converted->bmiHeader.biBitCount;
    pbmci->bmciHeader.bcWidth = converted->bmiHeader.biWidth;
    pbmci->bmciHeader.bcHeight = converted->bmiHeader.biHeight;
    pbmci->bmciHeader.bcPlanes = converted->bmiHeader.biPlanes;

    if(pbmci->bmciHeader.bcBitCount <= 8)
    {
        UINT numColors = converted->bmiHeader.biClrUsed;
        if(!numColors) numColors = 1 << pbmci->bmciHeader.bcBitCount;
        if(usage == DIB_PAL_COLORS)
        {
            RtlZeroMemory(pbmci->bmciColors, (1 << pbmci->bmciHeader.bcBitCount) * sizeof(WORD));
            RtlCopyMemory(pbmci->bmciColors, converted->bmiColors, numColors * sizeof(WORD));
        }
        else
        {
            UINT i;
            RtlZeroMemory(pbmci->bmciColors, (1 << pbmci->bmciHeader.bcBitCount) * sizeof(RGBTRIPLE));
            for(i=0; i<numColors; i++)
            {
                pbmci->bmciColors[i].rgbtRed = converted->bmiColors[i].rgbRed;
                pbmci->bmciColors[i].rgbtGreen = converted->bmiColors[i].rgbGreen;
                pbmci->bmciColors[i].rgbtBlue = converted->bmiColors[i].rgbBlue;
            }
        }
    }
    /* Now free it, it's not needed anymore */
    ExFreePoolWithTag(converted, TAG_DIB);
}

/* EOF */
