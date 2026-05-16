/*
 * PROJECT:         ReactOS win32 kernel mode subsystem
 * LICENSE:         GPL - See COPYING in the top level directory
 * FILE:            win32ss/gdi/ntgdi/dibobj.c
 * PURPOSE:         Dib object functions
 * PROGRAMMER:
 */

#include <win32k.h>
#include <safe_user_io.h>

#define NDEBUG
#include <debug.h>

static const RGBQUAD DefLogPaletteQuads[20] =   /* Copy of Default Logical Palette */
{
    /* rgbBlue, rgbGreen, rgbRed, rgbReserved */
    { 0x00, 0x00, 0x00, 0x00 },
    { 0x00, 0x00, 0x80, 0x00 },
    { 0x00, 0x80, 0x00, 0x00 },
    { 0x00, 0x80, 0x80, 0x00 },
    { 0x80, 0x00, 0x00, 0x00 },
    { 0x80, 0x00, 0x80, 0x00 },
    { 0x80, 0x80, 0x00, 0x00 },
    { 0xc0, 0xc0, 0xc0, 0x00 },
    { 0xc0, 0xdc, 0xc0, 0x00 },
    { 0xf0, 0xca, 0xa6, 0x00 },
    { 0xf0, 0xfb, 0xff, 0x00 },
    { 0xa4, 0xa0, 0xa0, 0x00 },
    { 0x80, 0x80, 0x80, 0x00 },
    { 0x00, 0x00, 0xff, 0x00 },
    { 0x00, 0xff, 0x00, 0x00 },
    { 0x00, 0xff, 0xff, 0x00 },
    { 0xff, 0x00, 0x00, 0x00 },
    { 0xff, 0x00, 0xff, 0x00 },
    { 0xff, 0xff, 0x00, 0x00 },
    { 0xff, 0xff, 0xff, 0x00 }
};

PPALETTE
NTAPI
CreateDIBPalette(
    _In_ const BITMAPINFO *pbmi,
    _In_ PDC pdc,
    _In_ ULONG iUsage)
{
    PPALETTE ppal;
    ULONG i, cBitsPixel, cColors;

    if (pbmi->bmiHeader.biSize < sizeof(BITMAPINFOHEADER))
    {
        PBITMAPCOREINFO pbci = (PBITMAPCOREINFO)pbmi;
        cBitsPixel = pbci->bmciHeader.bcBitCount;
    }
    else
    {
        cBitsPixel = pbmi->bmiHeader.biBitCount;
    }

    /* Check if the colors are indexed */
    if (cBitsPixel <= 8)
    {
        /* We create a "full" palette */
        cColors = 1 << cBitsPixel;

        /* Allocate the palette */
        ppal = PALETTE_AllocPalette(PAL_INDEXED | PAL_DIBSECTION,
                                    cColors,
                                    NULL,
                                    0,
                                    0,
                                    0);
        if (ppal == NULL)
        {
            DPRINT1("Failed to allocate palette.\n");
            return NULL;
        }

        /* Check if the BITMAPINFO specifies how many colors to use */
        if ((pbmi->bmiHeader.biSize >= sizeof(BITMAPINFOHEADER)) &&
            (pbmi->bmiHeader.biClrUsed != 0))
        {
            /* This is how many colors we can actually process */
            cColors = min(cColors, pbmi->bmiHeader.biClrUsed);
        }

        /* Check how to use the colors */
        if (iUsage == DIB_PAL_COLORS)
        {
            COLORREF crColor;

            /* The colors are an array of WORD indices into the DC palette */
            PWORD pwColors = (PWORD)((PCHAR)pbmi + pbmi->bmiHeader.biSize);

            /* Use the DCs palette or, if no DC is given, the default one */
            PPALETTE ppalDC = pdc ? pdc->dclevel.ppal : gppalDefault;

            /* Loop all color indices in the DIB */
            for (i = 0; i < cColors; i++)
            {
                /* Get the palette index and handle wraparound when exceeding
                   the number of colors in the DC palette */
                WORD wIndex = pwColors[i] % ppalDC->NumColors;

                /* USe the RGB value from the DC palette */
                crColor = PALETTE_ulGetRGBColorFromIndex(ppalDC, wIndex);
                PALETTE_vSetRGBColorForIndex(ppal, i, crColor);
            }
        }
        else if (iUsage == DIB_PAL_BRUSHHACK)
        {
            /* The colors are an array of WORD indices into the DC palette */
            PWORD pwColors = (PWORD)((PCHAR)pbmi + pbmi->bmiHeader.biSize);

            /* Loop all color indices in the DIB */
            for (i = 0; i < cColors; i++)
            {
                /* Set the index directly as the RGB color, the real palette
                   containing RGB values will be calculated when the brush is
                   realized */
                PALETTE_vSetRGBColorForIndex(ppal, i, pwColors[i]);
            }

            /* Mark the palette as a brush hack palette */
            ppal->flFlags |= PAL_BRUSHHACK;
        }
//        else if (iUsage == 2)
//        {
            // FIXME: this one is undocumented
//            ASSERT(FALSE);
//        }
        else if (pbmi->bmiHeader.biSize >= sizeof(BITMAPINFOHEADER))
        {
            /* The colors are an array of RGBQUAD values */
            RGBQUAD *prgb = (RGBQUAD*)((PCHAR)pbmi + pbmi->bmiHeader.biSize);
            RGBQUAD colors[256] = {{0}};

            // FIXME: do we need to handle PALETTEINDEX / PALETTERGB macro?

            /* Use SEH to verify we can READ prgb[] succesfully */
            _SEH2_TRY
            {
                RtlCopyMemory(colors, prgb, cColors * sizeof(colors[0]));
            }
            _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
            {
              /* Do Nothing */
            }
            _SEH2_END;

            for (i = 0; i < cColors; ++i)
            {
                /* Get the color value and translate it to a COLORREF */
                COLORREF crColor = RGB(colors[i].rgbRed, colors[i].rgbGreen, colors[i].rgbBlue);

                /* Set the RGB value in the palette */
                PALETTE_vSetRGBColorForIndex(ppal, i, crColor);
            }
        }
        else
        {
            /* The colors are an array of RGBTRIPLE values */
            RGBTRIPLE *prgb = (RGBTRIPLE*)((PCHAR)pbmi + pbmi->bmiHeader.biSize);

            /* Loop all color indices in the DIB */
            for (i = 0; i < cColors; i++)
            {
                /* Get the color value and translate it to a COLORREF */
                RGBTRIPLE rgb = prgb[i];
                COLORREF crColor = RGB(rgb.rgbtRed, rgb.rgbtGreen, rgb.rgbtBlue);

                /* Set the RGB value in the palette */
                PALETTE_vSetRGBColorForIndex(ppal, i, crColor);
            }
        }
    }
    else
    {
        /* This is a bitfield / RGB palette */
        ULONG flRedMask, flGreenMask, flBlueMask;

        /* Check if the DIB contains bitfield values */
        if ((pbmi->bmiHeader.biSize >= sizeof(BITMAPINFOHEADER)) &&
            (pbmi->bmiHeader.biCompression == BI_BITFIELDS))
        {
            /* Check if we have a v4/v5 header */
            if (pbmi->bmiHeader.biSize >= sizeof(BITMAPV4HEADER))
            {
                /* The masks are dedicated fields in the header */
                PBITMAPV4HEADER pbmV4Header = (PBITMAPV4HEADER)&pbmi->bmiHeader;
                flRedMask = pbmV4Header->bV4RedMask;
                flGreenMask = pbmV4Header->bV4GreenMask;
                flBlueMask = pbmV4Header->bV4BlueMask;
            }
            else
            {
                /* The masks are the first 3 values in the DIB color table */
                PDWORD pdwColors = (PVOID)((PCHAR)pbmi + pbmi->bmiHeader.biSize);
                flRedMask = pdwColors[0];
                flGreenMask = pdwColors[1];
                flBlueMask = pdwColors[2];
            }
        }
        else
        {
            /* Check what bit depth we have. Note: optimization flags are
               calculated in PALETTE_AllocPalette()  */
            if (cBitsPixel == 16)
            {
                /* This is an RGB 555 palette */
                flRedMask = 0x7C00;
                flGreenMask = 0x03E0;
                flBlueMask = 0x001F;
            }
            else
            {
                /* This is an RGB 888 palette */
                flRedMask = 0xFF0000;
                flGreenMask = 0x00FF00;
                flBlueMask = 0x0000FF;
            }
        }

        /* Allocate the bitfield palette */
        ppal = PALETTE_AllocPalette(PAL_BITFIELDS | PAL_DIBSECTION,
                                    0,
                                    NULL,
                                    flRedMask,
                                    flGreenMask,
                                    flBlueMask);
    }

    /* We're done, return the palette */
    return ppal;
}

// Converts a DIB to a device-dependent bitmap
static INT
FASTCALL
IntSetDIBits(
    PDC   DC,
    HBITMAP  hBitmap,
    UINT  StartScan,
    UINT  ScanLines,
    CONST VOID  *Bits,
    ULONG cjMaxBits,
    CONST BITMAPINFO  *bmi,
    UINT  ColorUse)
{
    HBITMAP     SourceBitmap;
    PSURFACE    psurfDst, psurfSrc;
    INT         result = 0;
    RECT		rcDst;
    POINTL		ptSrc;
    EXLATEOBJ	exlo;
    PPALETTE    ppalDIB = 0;
    ULONG cjSizeImage;

    if (!bmi || !Bits) return 0;

    /* Check for uncompressed formats */
    if ((bmi->bmiHeader.biCompression == BI_RGB) ||
             (bmi->bmiHeader.biCompression == BI_BITFIELDS))
    {
        /* Calculate the image size */
        cjSizeImage = DIB_GetDIBImageBytes(bmi->bmiHeader.biWidth,
                                           ScanLines,
                                           bmi->bmiHeader.biBitCount);
    }
    /* Check if the header provided an image size */
    else if (bmi->bmiHeader.biSizeImage != 0)
    {
        /* Use the given size */
        cjSizeImage = bmi->bmiHeader.biSizeImage;
    }
    else
    {
        /* Compressed format without a size. This is invalid. */
        DPRINT1("Compressed format without a size\n");
        return 0;
    }

    /* Check if the size that we have is ok */
    if ((cjSizeImage > cjMaxBits) || (cjSizeImage == 0))
    {
        DPRINT1("Invalid bitmap size! cjSizeImage = %lu, cjMaxBits = %lu\n",
                cjSizeImage, cjMaxBits);
        return 0;
    }

    SourceBitmap = GreCreateBitmapEx(bmi->bmiHeader.biWidth,
                                     ScanLines,
                                     0,
                                     BitmapFormat(bmi->bmiHeader.biBitCount,
                                                  bmi->bmiHeader.biCompression),
                                     bmi->bmiHeader.biHeight < 0 ? BMF_TOPDOWN : 0,
                                     cjSizeImage,
                                     (PVOID)Bits,
                                     0);
    if (!SourceBitmap)
    {
        DPRINT1("Error: Could not create a bitmap.\n");
        EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
        return 0;
    }

    psurfDst = SURFACE_ShareLockSurface(hBitmap);
    psurfSrc = SURFACE_ShareLockSurface(SourceBitmap);

    if(!(psurfSrc && psurfDst))
    {
        DPRINT1("Error: Could not lock surfaces\n");
        goto cleanup;
    }

    /* Create a palette for the DIB */
    ppalDIB = CreateDIBPalette(bmi, DC, ColorUse);
    if (!ppalDIB)
    {
        EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
        goto cleanup;
    }

    /* Initialize EXLATEOBJ */
    EXLATEOBJ_vInitialize(&exlo,
                          ppalDIB,
                          psurfDst->ppal,
                          RGB(0xff, 0xff, 0xff),
                          RGB(0xff, 0xff, 0xff), //DC->pdcattr->crBackgroundClr,
                          0); // DC->pdcattr->crForegroundClr);

    rcDst.top = StartScan;
    rcDst.left = 0;
    rcDst.bottom = rcDst.top + ScanLines;
    rcDst.right = psurfDst->SurfObj.sizlBitmap.cx;
    ptSrc.x = 0;
    ptSrc.y = 0;

    result = IntEngCopyBits(&psurfDst->SurfObj,
                            &psurfSrc->SurfObj,
                            NULL,
                            &exlo.xlo,
                            &rcDst,
                            &ptSrc);
    if(result)
        result = ScanLines;

    EXLATEOBJ_vCleanup(&exlo);

cleanup:
    if (ppalDIB) PALETTE_ShareUnlockPalette(ppalDIB);
    if(psurfSrc) SURFACE_ShareUnlockSurface(psurfSrc);
    if(psurfDst) SURFACE_ShareUnlockSurface(psurfDst);
    GreDeleteObject(SourceBitmap);

    return result;
}

static
HBITMAP
IntGdiCreateMaskFromRLE(
    DWORD Width,
    DWORD Height,
    ULONG Compression,
    const BYTE* Bits,
    DWORD BitsSize)
{
    HBITMAP Mask;
    DWORD x, y;
    SURFOBJ* SurfObj;
    UINT i = 0;
    BYTE Data, NumPixels, ToSkip;

    ASSERT((Compression == BI_RLE8) || (Compression == BI_RLE4));

    /* Create the bitmap */
    Mask = GreCreateBitmapEx(Width, Height, 0, BMF_1BPP, 0, 0, NULL, 0);
    if (!Mask)
        return NULL;

    SurfObj = EngLockSurface((HSURF)Mask);
    if (!SurfObj)
    {
        GreDeleteObject(Mask);
        return NULL;
    }
    ASSERT(SurfObj->pvBits != NULL);

    x = y = 0;

    while (i < BitsSize)
    {
        NumPixels = Bits[i];
        Data = Bits[i + 1];
        i += 2;

        if (NumPixels != 0)
        {
            if ((x + NumPixels) > Width)
                NumPixels = Width - x;

            if (NumPixels == 0)
                continue;

            DIB_1BPP_HLine(SurfObj, x, x + NumPixels, y, 1);
            x += NumPixels;
            continue;
        }

        if (Data < 3)
        {
            switch (Data)
            {
                case 0:
                    /* End of line */
                    y++;
                    if (y == Height)
                        goto done;
                    x = 0;
                    break;
                case 1:
                    /* End of file */
                    goto done;
                case 2:
                    /* Jump */
                    if (i >= (BitsSize - 1))
                        goto done;
                    x += Bits[i];
                    if (x > Width)
                        x = Width;
                    y += Bits[i + 1];
                    if (y >= Height)
                        goto done;
                    i += 2;
                    break;
            }
            /* Done for this run */
            continue;
        }

        /* Embedded data into the RLE */
        NumPixels = Data;
        if (Compression == BI_RLE8)
            ToSkip = NumPixels;
        else
            ToSkip = (NumPixels / 2) + (NumPixels & 1);

        if ((i + ToSkip) > BitsSize)
            goto done;
        ToSkip = (ToSkip + 1) & ~1;

        if ((x + NumPixels) > Width)
            NumPixels = Width - x;

        if (NumPixels != 0)
        {
            DIB_1BPP_HLine(SurfObj, x, x + NumPixels, y, 1);
            x += NumPixels;
        }
        i += ToSkip;
    }

done:
    EngUnlockSurface(SurfObj);
    return Mask;
}

W32KAPI
INT
APIENTRY
NtGdiSetDIBitsToDeviceInternal(
    IN HDC hDC,
    IN INT XDest,
    IN INT YDest,
    IN DWORD Width,
    IN DWORD Height,
    IN INT XSrc,
    IN INT YSrc,
    IN DWORD StartScan,
    IN DWORD ScanLines,
    IN LPBYTE Bits,
    IN LPBITMAPINFO bmi,
    IN DWORD ColorUse,
    IN UINT cjMaxBits,
    IN UINT cjMaxInfo,
    IN BOOL bTransformCoordinates,
    IN OPTIONAL HANDLE hcmXform)
{
    INT ret;
    PDC pDC = NULL;
    HBITMAP hSourceBitmap = NULL, hMaskBitmap = NULL;
    SURFOBJ *pDestSurf, *pSourceSurf = NULL, *pMaskSurf = NULL;
    SURFACE *pSurf;
    RECTL rcDest;
    POINTL ptSource;
    //INT DIBWidth;
    SIZEL SourceSize;
    EXLATEOBJ exlo;
    PPALETTE ppalDIB = NULL;
    LPBITMAPINFO pbmiSafe;
    BOOL bResult;
    NTSTATUS Status;

    if (!Bits) return 0;

    pbmiSafe = ExAllocatePoolWithTag(PagedPool, cjMaxInfo, 'pmTG');
    if (!pbmiSafe) return 0;

    Status = SafeProbeForRead(bmi, cjMaxInfo, 1);
    if (NT_SUCCESS(Status)) Status = SafeProbeForRead(Bits, cjMaxBits, 1);
    if (!NT_SUCCESS(Status))
    {
        ret = 0;
        goto Exit;
    }
    else
    {
        RtlCopyMemory(pbmiSafe, bmi, cjMaxInfo);
        bmi = pbmiSafe;
    }

    DPRINT("StartScan %d ScanLines %d Bits %p bmi %p ColorUse %d\n"
           "    Height %d Width %d biSizeImage %d\n"
           "    biHeight %d biWidth %d biBitCount %d\n"
           "    XSrc %d YSrc %d XDest %d YDest %d\n",
           StartScan, ScanLines, Bits, bmi, ColorUse,
           Height, Width, bmi->bmiHeader.biSizeImage,
           bmi->bmiHeader.biHeight, bmi->bmiHeader.biWidth,
           bmi->bmiHeader.biBitCount,
           XSrc, YSrc, XDest, YDest);

    if (YDest < 0)
    {
        ScanLines = min(ScanLines, abs(bmi->bmiHeader.biHeight) - StartScan);
    }

    if (ScanLines == 0)
    {
        DPRINT1("ScanLines == 0\n");
        ret = 0;
        goto Exit;
    }

    pDC = DC_LockDc(hDC);
    if (!pDC)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        ret = 0;
        goto Exit;
    }

    if (pDC->dctype == DCTYPE_INFO)
    {
        ret = 0;
        goto Exit;
    }

    rcDest.left = XDest;
    rcDest.top = YDest;
    if (bTransformCoordinates)
    {
        IntLPtoDP(pDC, (LPPOINT)&rcDest, 2);
    }
    rcDest.left += pDC->ptlDCOrig.x;
    rcDest.top += pDC->ptlDCOrig.y;
    rcDest.right = rcDest.left + Width;
    rcDest.bottom = rcDest.top + Height;
    rcDest.top += StartScan;

    if (pDC->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
    {
       IntUpdateBoundsRect(pDC, &rcDest);
    }

    ptSource.x = XSrc;
    ptSource.y = YSrc;

    SourceSize.cx = bmi->bmiHeader.biWidth;
    SourceSize.cy = ScanLines;

    //DIBWidth = WIDTH_BYTES_ALIGN32(SourceSize.cx, bmi->bmiHeader.biBitCount);

    hSourceBitmap = GreCreateBitmapEx(bmi->bmiHeader.biWidth,
                                      ScanLines,
                                      0,
                                      BitmapFormat(bmi->bmiHeader.biBitCount,
                                                   bmi->bmiHeader.biCompression),
                                      bmi->bmiHeader.biHeight < 0 ? BMF_TOPDOWN : 0,
                                      bmi->bmiHeader.biSizeImage,
                                      Bits,
                                      0);

    if (!hSourceBitmap)
    {
        EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
        ret = 0;
        goto Exit;
    }

    pSourceSurf = EngLockSurface((HSURF)hSourceBitmap);
    if (!pSourceSurf)
    {
        ret = 0;
        goto Exit;
    }

    /* HACK: If this is a RLE bitmap, only the relevant pixels must be set. */
    if ((bmi->bmiHeader.biCompression == BI_RLE8) || (bmi->bmiHeader.biCompression == BI_RLE4))
    {
        hMaskBitmap = IntGdiCreateMaskFromRLE(bmi->bmiHeader.biWidth,
            ScanLines,
            bmi->bmiHeader.biCompression,
            Bits,
            cjMaxBits);
        if (!hMaskBitmap)
        {
            EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
            ret = 0;
            goto Exit;
        }
        pMaskSurf = EngLockSurface((HSURF)hMaskBitmap);
        if (!pMaskSurf)
        {
            ret = 0;
            goto Exit;
        }
    }

    /* Create a palette for the DIB */
    ppalDIB = CreateDIBPalette(bmi, pDC, ColorUse);
    if (!ppalDIB)
    {
        EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
        ret = 0;
        goto Exit;
    }

    /* This is actually a blit */
    DC_vPrepareDCsForBlit(pDC, &rcDest, NULL, NULL);
    pSurf = pDC->dclevel.pSurface;
    if (!pSurf)
    {
        DC_vFinishBlit(pDC, NULL);
        ret = ScanLines;
        goto Exit;
    }

    ASSERT(pSurf->ppal);

    /* Initialize EXLATEOBJ */
    EXLATEOBJ_vInitialize(&exlo,
                          ppalDIB,
                          pSurf->ppal,
                          RGB(0xff, 0xff, 0xff),
                          pDC->pdcattr->crBackgroundClr,
                          pDC->pdcattr->crForegroundClr);

    pDestSurf = &pSurf->SurfObj;

    /* Copy the bits */
    DPRINT("BitsToDev with rcDest=(%d|%d) (%d|%d), ptSource=(%d|%d) w=%d h=%d\n",
           rcDest.left, rcDest.top, rcDest.right, rcDest.bottom,
           ptSource.x, ptSource.y, SourceSize.cx, SourceSize.cy);

    /* This fixes the large Google text on Google.com from being upside down */
    if (rcDest.top > rcDest.bottom)
    {
        RECTL_vMakeWellOrdered(&rcDest);
        ptSource.y -= SourceSize.cy;
    }

    bResult = IntEngBitBlt(pDestSurf,
                          pSourceSurf,
                          pMaskSurf,
                          (CLIPOBJ *)&pDC->co,
                          &exlo.xlo,
                          &rcDest,
                          &ptSource,
                          pMaskSurf ? &ptSource : NULL,
                          NULL,
                          NULL,
                          pMaskSurf ? ROP4_MASK : ROP4_FROM_INDEX(R3_OPINDEX_SRCCOPY));

    /* Cleanup EXLATEOBJ */
    EXLATEOBJ_vCleanup(&exlo);

    /* We're done */
    DC_vFinishBlit(pDC, NULL);

    ret = bResult ? ScanLines : 0;

Exit:

    if (ppalDIB) PALETTE_ShareUnlockPalette(ppalDIB);
    if (pSourceSurf) EngUnlockSurface(pSourceSurf);
    if (hSourceBitmap) EngDeleteSurface((HSURF)hSourceBitmap);
    if (pMaskSurf) EngUnlockSurface(pMaskSurf);
    if (hMaskBitmap) EngDeleteSurface((HSURF)hMaskBitmap);
    if (pDC) DC_UnlockDc(pDC);
    ExFreePoolWithTag(pbmiSafe, 'pmTG');

    return ret;
}


/* Converts a device-dependent bitmap to a DIB */
INT
APIENTRY
GreGetDIBitsInternal(
    HDC hDC,
    HBITMAP hBitmap,
    UINT StartScan,
    UINT ScanLines,
    LPBYTE Bits,
    LPBITMAPINFO Info,
    UINT Usage,
    UINT MaxBits,
    UINT MaxInfo)
{
    BITMAPCOREINFO* pbmci = NULL;
    PSURFACE psurf = NULL;
    PDC pDC;
    LONG width, height;
    WORD planes, bpp;
    DWORD compr, size ;
    USHORT i;
    int bitmap_type;
    RGBQUAD* rgbQuads;
    VOID* colorPtr;

    DPRINT("Entered GreGetDIBitsInternal()\n");

    if ((Usage && Usage != DIB_PAL_COLORS) || !Info || !hBitmap)
        return 0;

    pDC = DC_LockDc(hDC);
    if (pDC == NULL || pDC->dctype == DCTYPE_INFO)
    {
        ScanLines = 0;
        goto done;
    }

    /* Get a pointer to the source bitmap object */
    psurf = SURFACE_ShareLockSurface(hBitmap);
    if (psurf == NULL)
    {
        ScanLines = 0;
        goto done;
    }

    colorPtr = (LPBYTE)Info + Info->bmiHeader.biSize;
    rgbQuads = colorPtr;

    bitmap_type = DIB_GetBitmapInfo(&Info->bmiHeader,
                                    &width,
                                    &height,
                                    &planes,
                                    &bpp,
                                    &compr,
                                    &size);
    if(bitmap_type == -1)
    {
        DPRINT1("Wrong bitmap format\n");
        EngSetLastError(ERROR_INVALID_PARAMETER);
        ScanLines = 0;
        goto done;
    }
    else if(bitmap_type == 0)
    {
        /* We need a BITMAPINFO to create a DIB, but we have to fill
         * the BITMAPCOREINFO we're provided */
        pbmci = (BITMAPCOREINFO*)Info;
        /* fill in the the bit count, so we can calculate the right ColorsSize during the conversion */
        pbmci->bmciHeader.bcBitCount = BitsPerFormat(psurf->SurfObj.iBitmapFormat);
        Info = DIB_ConvertBitmapInfo((BITMAPINFO*)pbmci, Usage);
        if(Info == NULL)
        {
            DPRINT1("Error, could not convert the BITMAPCOREINFO!\n");
            ScanLines = 0;
            goto done;
        }
        rgbQuads = Info->bmiColors;
    }

    /* Validate input:
       - negative width is always an invalid value
       - non-null Bits and zero bpp is an invalid combination
       - only check the rest of the input params if either bpp is non-zero or Bits are set */
    if (width < 0 || (bpp == 0 && Bits))
    {
        ScanLines = 0;
        goto done;
    }

    if (Bits || bpp)
    {
        if ((height == 0 || width == 0) || (compr && compr != BI_BITFIELDS && compr != BI_RGB))
        {
            ScanLines = 0;
            goto done;
        }
    }

    Info->bmiHeader.biClrUsed = 0;
    Info->bmiHeader.biClrImportant = 0;

    /* Fill in the structure */
    switch(bpp)
    {
    case 0: /* Only info */
        Info->bmiHeader.biWidth = psurf->SurfObj.sizlBitmap.cx;
        Info->bmiHeader.biHeight = (psurf->SurfObj.fjBitmap & BMF_TOPDOWN) ?
                                   -psurf->SurfObj.sizlBitmap.cy :
                                   psurf->SurfObj.sizlBitmap.cy;
        Info->bmiHeader.biPlanes = 1;
        Info->bmiHeader.biBitCount = BitsPerFormat(psurf->SurfObj.iBitmapFormat);
        Info->bmiHeader.biSizeImage = DIB_GetDIBImageBytes( Info->bmiHeader.biWidth,
                                      Info->bmiHeader.biHeight,
                                      Info->bmiHeader.biBitCount);
        Info->bmiHeader.biCompression = (Info->bmiHeader.biBitCount == 16 || Info->bmiHeader.biBitCount == 32) ?
                                        BI_BITFIELDS : BI_RGB;
        Info->bmiHeader.biXPelsPerMeter = 0;
        Info->bmiHeader.biYPelsPerMeter = 0;

        if (Info->bmiHeader.biBitCount <= 8 && Info->bmiHeader.biClrUsed == 0)
            Info->bmiHeader.biClrUsed = 1 << Info->bmiHeader.biBitCount;

        ScanLines = 1;
        goto done;

    case 1:
    case 4:
    case 8:
        Info->bmiHeader.biClrUsed = 1 << bpp;

        /* If the bitmap is a DIB section and has the same format as what
         * is requested, go ahead! */
        if((psurf->hSecure) &&
                (BitsPerFormat(psurf->SurfObj.iBitmapFormat) == bpp))
        {
            if(Usage == DIB_RGB_COLORS)
            {
                ULONG colors = min(psurf->ppal->NumColors, 256);
                if(colors != 256) Info->bmiHeader.biClrUsed = colors;
                for(i = 0; i < colors; i++)
                {
                    rgbQuads[i].rgbRed = psurf->ppal->IndexedColors[i].peRed;
                    rgbQuads[i].rgbGreen = psurf->ppal->IndexedColors[i].peGreen;
                    rgbQuads[i].rgbBlue = psurf->ppal->IndexedColors[i].peBlue;
                    rgbQuads[i].rgbReserved = 0;
                }
            }
            else
            {
                for(i = 0; i < 256; i++)
                    ((WORD*)rgbQuads)[i] = i;
            }
        }
        else
        {
            if(Usage == DIB_PAL_COLORS)
            {
                for(i = 0; i < 256; i++)
                {
                    ((WORD*)rgbQuads)[i] = i;
                }
            }
            else if(bpp > 1 && bpp == BitsPerFormat(psurf->SurfObj.iBitmapFormat))
            {
                /* For color DDBs in native depth (mono DDBs always have
                   a black/white palette):
                   Generate the color map from the selected palette */
                PPALETTE pDcPal = PALETTE_ShareLockPalette(pDC->dclevel.hpal);
                if(!pDcPal)
                {
                    ScanLines = 0 ;
                    goto done ;
                }
                for (i = 0; i < pDcPal->NumColors; i++)
                {
                    rgbQuads[i].rgbRed      = pDcPal->IndexedColors[i].peRed;
                    rgbQuads[i].rgbGreen    = pDcPal->IndexedColors[i].peGreen;
                    rgbQuads[i].rgbBlue     = pDcPal->IndexedColors[i].peBlue;
                    rgbQuads[i].rgbReserved = 0;
                }
                PALETTE_ShareUnlockPalette(pDcPal);
            }
            else
            {
                switch (bpp)
                {
                case 1:
                    rgbQuads[0].rgbRed = rgbQuads[0].rgbGreen = rgbQuads[0].rgbBlue = 0;
                    rgbQuads[0].rgbReserved = 0;
                    rgbQuads[1].rgbRed = rgbQuads[1].rgbGreen = rgbQuads[1].rgbBlue = 0xff;
                    rgbQuads[1].rgbReserved = 0;
                    break;

                case 4:
                    /* The EGA palette is the first and last 8 colours of the default palette
                       with the innermost pair swapped */
                    RtlCopyMemory(rgbQuads,     DefLogPaletteQuads,      7 * sizeof(RGBQUAD));
                    RtlCopyMemory(rgbQuads + 7, DefLogPaletteQuads + 12, 1 * sizeof(RGBQUAD));
                    RtlCopyMemory(rgbQuads + 8, DefLogPaletteQuads +  7, 1 * sizeof(RGBQUAD));
                    RtlCopyMemory(rgbQuads + 9, DefLogPaletteQuads + 13, 7 * sizeof(RGBQUAD));
                    break;

                case 8:
                {
                    INT i;

                    memcpy(rgbQuads, DefLogPaletteQuads, 10 * sizeof(RGBQUAD));
                    memcpy(rgbQuads + 246, DefLogPaletteQuads + 10, 10 * sizeof(RGBQUAD));

                    for (i = 10; i < 246; i++)
                    {
                        rgbQuads[i].rgbRed = (i & 0x07) << 5;
                        rgbQuads[i].rgbGreen = (i & 0x38) << 2;
                        rgbQuads[i].rgbBlue = i & 0xc0;
                        rgbQuads[i].rgbReserved = 0;
                    }
                }
                }
            }
        }
        break;

    case 15:
        if (Info->bmiHeader.biCompression == BI_BITFIELDS)
        {
            ((PDWORD)Info->bmiColors)[0] = 0x7c00;
            ((PDWORD)Info->bmiColors)[1] = 0x03e0;
            ((PDWORD)Info->bmiColors)[2] = 0x001f;
        }
        break;

    case 16:
        if (Info->bmiHeader.biCompression == BI_BITFIELDS)
        {
            if (psurf->hSecure)
            {
                ((PDWORD)Info->bmiColors)[0] = psurf->ppal->RedMask;
                ((PDWORD)Info->bmiColors)[1] = psurf->ppal->GreenMask;
                ((PDWORD)Info->bmiColors)[2] = psurf->ppal->BlueMask;
            }
            else
            {
                ((PDWORD)Info->bmiColors)[0] = 0xf800;
                ((PDWORD)Info->bmiColors)[1] = 0x07e0;
                ((PDWORD)Info->bmiColors)[2] = 0x001f;
            }
        }
        break;

    case 24:
    case 32:
        if (Info->bmiHeader.biCompression == BI_BITFIELDS)
        {
            if (psurf->hSecure)
            {
                ((PDWORD)Info->bmiColors)[0] = psurf->ppal->RedMask;
                ((PDWORD)Info->bmiColors)[1] = psurf->ppal->GreenMask;
                ((PDWORD)Info->bmiColors)[2] = psurf->ppal->BlueMask;
            }
            else
            {
                ((PDWORD)Info->bmiColors)[0] = 0xff0000;
                ((PDWORD)Info->bmiColors)[1] = 0x00ff00;
                ((PDWORD)Info->bmiColors)[2] = 0x0000ff;
            }
        }
        break;

    default:
        ScanLines = 0;
        goto done;
    }

    Info->bmiHeader.biSizeImage = DIB_GetDIBImageBytes(width, height, bpp);
    Info->bmiHeader.biPlanes = 1;

    if(Bits && ScanLines)
    {
        /* Create a DIBSECTION, blt it, profit */
        PVOID pDIBits ;
        HBITMAP hBmpDest;
        PSURFACE psurfDest;
        EXLATEOBJ exlo;
        RECT rcDest;
        POINTL srcPoint;
        BOOL ret ;
        int newLines = -1;

        if (StartScan >= abs(Info->bmiHeader.biHeight))
        {
            ScanLines = 1;
            goto done;
        }
        else
        {
            ScanLines = min(ScanLines, abs(Info->bmiHeader.biHeight) - StartScan);
        }

        if (abs(Info->bmiHeader.biHeight) < psurf->SurfObj.sizlBitmap.cy)
        {
            StartScan += psurf->SurfObj.sizlBitmap.cy - abs(Info->bmiHeader.biHeight);
        }
        /* Fixup values */
        Info->bmiHeader.biHeight = (height < 0) ?
                                   -(LONG)ScanLines : ScanLines;
        /* Create the DIB */
        hBmpDest = DIB_CreateDIBSection(pDC, Info, Usage, &pDIBits, NULL, 0, 0);
        /* Restore them */
        Info->bmiHeader.biHeight = height;

        if(!hBmpDest)
        {
            DPRINT1("Unable to create a DIB Section!\n");
            EngSetLastError(ERROR_INVALID_PARAMETER);
            ScanLines = 0;
            goto done ;
        }

        psurfDest = SURFACE_ShareLockSurface(hBmpDest);

        RECTL_vSetRect(&rcDest, 0, 0, Info->bmiHeader.biWidth, ScanLines);
        Info->bmiHeader.biWidth = width;
        srcPoint.x = 0;

        if (abs(Info->bmiHeader.biHeight) <= psurf->SurfObj.sizlBitmap.cy)
        {
            srcPoint.y = psurf->SurfObj.sizlBitmap.cy - StartScan - ScanLines;
        }
        else
        {
            /*  Determine the actual number of lines copied from the  */
            /*  original bitmap. It might be different from ScanLines. */
            newLines = abs(Info->bmiHeader.biHeight) - psurf->SurfObj.sizlBitmap.cy;
            newLines = min((int)(StartScan + ScanLines - newLines), psurf->SurfObj.sizlBitmap.cy);
            if (newLines > 0)
            {
                srcPoint.y = psurf->SurfObj.sizlBitmap.cy - newLines;
                if (StartScan > psurf->SurfObj.sizlBitmap.cy)
                {
                    newLines -= (StartScan - psurf->SurfObj.sizlBitmap.cy);
                }
            }
            else
            {
                newLines = 0;
                srcPoint.y = psurf->SurfObj.sizlBitmap.cy;
            }
        }

        EXLATEOBJ_vInitialize(&exlo, psurf->ppal, psurfDest->ppal, 0xffffff, 0xffffff, 0);

        ret = IntEngCopyBits(&psurfDest->SurfObj,
                             &psurf->SurfObj,
                             NULL,
                             &exlo.xlo,
                             &rcDest,
                             &srcPoint);

        SURFACE_ShareUnlockSurface(psurfDest);

        if(!ret)
            ScanLines = 0;
        else
        {
            RtlCopyMemory(Bits, pDIBits, DIB_GetDIBImageBytes (width, ScanLines, bpp));
        }
        /* Update if line count has changed */
        if (newLines != -1)
        {
            ScanLines = (UINT)newLines;
        }
        GreDeleteObject(hBmpDest);
        EXLATEOBJ_vCleanup(&exlo);
    }
    else
    {
        /* Signals success and not the actual number of scan lines*/
        ScanLines = 1;
    }

done:

    if (pbmci)
        DIB_FreeConvertedBitmapInfo(Info, (BITMAPINFO*)pbmci, Usage);

    if (psurf)
        SURFACE_ShareUnlockSurface(psurf);

    if (pDC)
        DC_UnlockDc(pDC);

    return ScanLines;
}
/* --- pretranslation split: the remaining function groups live in dibobj.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "dibobj.split2.c"
