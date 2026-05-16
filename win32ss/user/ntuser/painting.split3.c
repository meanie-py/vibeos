/* painting.split3.c — #include fragment of painting.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original painting.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


BOOL
UserDrawCaptionText(
   PWND pWnd,
   HDC hDc,
   const PUNICODE_STRING Text,
   const RECTL *lpRc,
   UINT uFlags,
   HFONT hFont)
{
   HFONT hOldFont = NULL;
   COLORREF OldTextColor;
   NONCLIENTMETRICSW nclm;
   NTSTATUS Status;
   BOOLEAN bDeleteFont = FALSE;
   SIZE Size;
   BOOL Ret = TRUE;
   ULONG fit = 0, Length;
   RECTL r = *lpRc;

   TRACE("UserDrawCaptionText: %wZ\n", Text);

   nclm.cbSize = sizeof(nclm);
   if (!UserSystemParametersInfo(SPI_GETNONCLIENTMETRICS, nclm.cbSize, &nclm, 0))
   {
      ERR("UserSystemParametersInfo() failed!\n");
      return FALSE;
   }

   if (!hFont)
   {
      if(uFlags & DC_SMALLCAP)
         Status = TextIntCreateFontIndirect(&nclm.lfSmCaptionFont, &hFont);
      else
         Status = TextIntCreateFontIndirect(&nclm.lfCaptionFont, &hFont);

      if(!NT_SUCCESS(Status))
      {
         ERR("TextIntCreateFontIndirect() failed! Status: 0x%x\n", Status);
         return FALSE;
      }

      bDeleteFont = TRUE;
   }

   IntGdiSetBkMode(hDc, TRANSPARENT);

   hOldFont = NtGdiSelectFont(hDc, hFont);

   if(uFlags & DC_INBUTTON)
      OldTextColor = IntGdiSetTextColor(hDc, IntGetSysColor(COLOR_BTNTEXT));
   else
      OldTextColor = IntGdiSetTextColor(hDc,
                                        IntGetSysColor(uFlags & DC_ACTIVE ? COLOR_CAPTIONTEXT : COLOR_INACTIVECAPTIONTEXT));

   // Adjust for system menu.
   if (pWnd && pWnd->style & WS_SYSMENU)
   {
      r.right -= UserGetSystemMetrics(SM_CYCAPTION) - 1;
      if ((pWnd->style & (WS_MAXIMIZEBOX | WS_MINIMIZEBOX)) && !(pWnd->ExStyle & WS_EX_TOOLWINDOW))
      {
         r.right -= UserGetSystemMetrics(SM_CXSIZE) + 1;
         r.right -= UserGetSystemMetrics(SM_CXSIZE) + 1;
      }
   }

   GreGetTextExtentExW(hDc, Text->Buffer, Text->Length/sizeof(WCHAR), r.right - r.left, &fit, 0, &Size, 0);

   Length = (Text->Length/sizeof(WCHAR) == fit ? fit : fit+1);

   if (Text->Length/sizeof(WCHAR) > Length)
   {
      Ret = FALSE;
   }

   if (Ret)
   {  // Faster while in setup.
      UserExtTextOutW( hDc,
                      lpRc->left,
                      lpRc->top + (lpRc->bottom - lpRc->top - Size.cy) / 2, // DT_SINGLELINE && DT_VCENTER
                      ETO_CLIPPED,
                     (RECTL *)lpRc,
                      Text->Buffer,
                      Length);
   }
   else
   {
      DrawTextW( hDc,
                 Text->Buffer,
                 Text->Length/sizeof(WCHAR),
                (RECTL *)&r,
                 DT_END_ELLIPSIS|DT_SINGLELINE|DT_VCENTER|DT_NOPREFIX|DT_LEFT);
   }

   IntGdiSetTextColor(hDc, OldTextColor);

   if (hOldFont)
      NtGdiSelectFont(hDc, hOldFont);

   if (bDeleteFont)
      GreDeleteObject(hFont);

   return Ret;
}

//
// This draws Buttons, Icons and Text...
//
BOOL UserDrawCaption(
   PWND pWnd,
   HDC hDc,
   RECTL *lpRc,
   HFONT hFont,
   HICON hIcon,
   const PUNICODE_STRING Str,
   UINT uFlags)
{
   BOOL Ret = FALSE;
   HBRUSH hBgBrush, hOldBrush = NULL;
   RECTL Rect = *lpRc;
   BOOL HasIcon;

   RECTL_vMakeWellOrdered(lpRc);

   /* Determine whether the icon needs to be displayed */
   if (!hIcon && pWnd != NULL)
   {
     HasIcon = (uFlags & DC_ICON) && !(uFlags & DC_SMALLCAP) &&
               (pWnd->style & WS_SYSMENU) && !(pWnd->ExStyle & WS_EX_TOOLWINDOW);
   }
   else
     HasIcon = (hIcon != NULL);

   // Draw the caption background
   if((uFlags & DC_GRADIENT) && !(uFlags & DC_INBUTTON))
   {
      static GRADIENT_RECT gcap = {0, 1};
      TRIVERTEX Vertices[2];
      COLORREF Colors[2];

      Colors[0] = IntGetSysColor((uFlags & DC_ACTIVE) ?
            COLOR_ACTIVECAPTION : COLOR_INACTIVECAPTION);

      Colors[1] = IntGetSysColor((uFlags & DC_ACTIVE) ?
            COLOR_GRADIENTACTIVECAPTION : COLOR_GRADIENTINACTIVECAPTION);

      Vertices[0].x = Rect.left;
      Vertices[0].y = Rect.top;
      Vertices[0].Red = (WORD)Colors[0]<<8;
      Vertices[0].Green = (WORD)Colors[0] & 0xFF00;
      Vertices[0].Blue = (WORD)(Colors[0]>>8) & 0xFF00;
      Vertices[0].Alpha = 0;

      Vertices[1].x = Rect.right;
      Vertices[1].y = Rect.bottom;
      Vertices[1].Red = (WORD)Colors[1]<<8;
      Vertices[1].Green = (WORD)Colors[1] & 0xFF00;
      Vertices[1].Blue = (WORD)(Colors[1]>>8) & 0xFF00;
      Vertices[1].Alpha = 0;

      if(!GreGradientFill(hDc, Vertices, 2, &gcap, 1, GRADIENT_FILL_RECT_H))
      {
         ERR("GreGradientFill() failed!\n");
         goto cleanup;
      }
   }
   else
   {
      if(uFlags & DC_INBUTTON)
         hBgBrush = IntGetSysColorBrush(COLOR_3DFACE);
      else if(uFlags & DC_ACTIVE)
         hBgBrush = IntGetSysColorBrush(COLOR_ACTIVECAPTION);
      else
         hBgBrush = IntGetSysColorBrush(COLOR_INACTIVECAPTION);

      hOldBrush = NtGdiSelectBrush(hDc, hBgBrush);

      if(!hOldBrush)
      {
         ERR("NtGdiSelectBrush() failed!\n");
         goto cleanup;
      }

      if(!NtGdiPatBlt(hDc, Rect.left, Rect.top,
         Rect.right - Rect.left,
         Rect.bottom - Rect.top,
         PATCOPY))
      {
         ERR("NtGdiPatBlt() failed!\n");
         goto cleanup;
      }
   }

   /* Draw icon */
   if (HasIcon)
   {
      PCURICON_OBJECT pIcon = NULL;

      if (hIcon)
      {
          pIcon = UserGetCurIconObject(hIcon);
      }
      else if (pWnd)
      {
          pIcon = NC_IconForWindow(pWnd);
          // FIXME: NC_IconForWindow should reference it for us */
          if (pIcon)
              UserReferenceObject(pIcon);
      }

      if (pIcon)
      {
         LONG cx = UserGetSystemMetrics(SM_CXSMICON);
         LONG cy = UserGetSystemMetrics(SM_CYSMICON);
         LONG x = Rect.left - cx/2 + 1 + (Rect.bottom - Rect.top)/2; // this is really what Window does
         LONG y = (Rect.top + Rect.bottom - cy)/2; // center
         UserDrawIconEx(hDc, x, y, pIcon, cx, cy, 0, NULL, DI_NORMAL);
         UserDereferenceObject(pIcon);
      }
      else
      {
          HasIcon = FALSE;
      }
   }

   if (HasIcon)
      Rect.left += Rect.bottom - Rect.top;

   if((uFlags & DC_TEXT))
   {
      BOOL Set = FALSE;
      Rect.left += 2;

      if (Str)
         Set = UserDrawCaptionText(pWnd, hDc, Str, &Rect, uFlags, hFont);
      else if (pWnd != NULL) // FIXME: Windows does not do that
      {
         UNICODE_STRING ustr;
         ustr.Buffer = pWnd->strName.Buffer; // FIXME: LARGE_STRING truncated!
         ustr.Length = (USHORT)min(pWnd->strName.Length, MAXUSHORT);
         ustr.MaximumLength = (USHORT)min(pWnd->strName.MaximumLength, MAXUSHORT);
         Set = UserDrawCaptionText(pWnd, hDc, &ustr, &Rect, uFlags, hFont);
      }
      if (pWnd)
      {
         if (Set)
            pWnd->state2 &= ~WNDS2_CAPTIONTEXTTRUNCATED;
         else
            pWnd->state2 |= WNDS2_CAPTIONTEXTTRUNCATED;
      }
   }

   Ret = TRUE;

cleanup:
   if (hOldBrush) NtGdiSelectBrush(hDc, hOldBrush);

   return Ret;
}

INT
FASTCALL
UserRealizePalette(HDC hdc)
{
  HWND hWnd, hWndDesktop;
  DWORD Ret;

  Ret = IntGdiRealizePalette(hdc);
  if (Ret) // There was a change.
  {
      hWnd = IntWindowFromDC(hdc);
      if (hWnd) // Send broadcast if dc is associated with a window.
      {  // FYI: Thread locked in CallOneParam.
         hWndDesktop = IntGetDesktopWindow();
         if ( hWndDesktop != hWnd )
         {
            PWND pWnd = UserGetWindowObject(hWndDesktop);
            ERR("RealizePalette Desktop.\n");
            hdc = UserGetWindowDC(pWnd);
            IntPaintDesktop(hdc);
            UserReleaseDC(pWnd,hdc,FALSE);
         }
         UserSendNotifyMessage((HWND)HWND_BROADCAST, WM_PALETTECHANGED, (WPARAM)hWnd, 0);
      }
  }
  return Ret;
}

BOOL
APIENTRY
NtUserDrawCaptionTemp(
   HWND hWnd,
   HDC hDC,
   LPCRECT lpRc,
   HFONT hFont,
   HICON hIcon,
   const PUNICODE_STRING str,
   UINT uFlags)
{
   PWND pWnd = NULL;
   UNICODE_STRING SafeStr = {0};
   NTSTATUS Status = STATUS_SUCCESS;
   RECTL SafeRect;
   BOOL Ret;

   UserEnterExclusive();

   if (hWnd != NULL)
   {
     if(!(pWnd = UserGetWindowObject(hWnd)))
     {
        UserLeave();
        return FALSE;
     }
   }

   Status = SafeProbeForRead(lpRc, sizeof(RECTL), sizeof(ULONG));
   if (NT_SUCCESS(Status))
       Status = SafeProbeForRead(SafeStr.Buffer, SafeStr.Length, sizeof(WCHAR));
   if (NT_SUCCESS(Status))
   {
       RtlCopyMemory(&SafeRect, lpRc, sizeof(RECTL));
       if (str != NULL)
       {
          SafeStr = ProbeForReadUnicodeString(str);
          if (SafeStr.Length != 0)
          {

          }
       }
   }

   if (Status != STATUS_SUCCESS)
   {
      SetLastNtError(Status);
      UserLeave();
      return FALSE;
   }

   if (str != NULL)
      Ret = UserDrawCaption(pWnd, hDC, &SafeRect, hFont, hIcon, &SafeStr, uFlags);
   else
   {
      if ( RECTL_bIsEmptyRect(&SafeRect) && hFont == 0 && hIcon == 0 )
      {
         Ret = TRUE;
         if (uFlags & DC_DRAWCAPTIONMD)
         {
            ERR("NC Caption Mode\n");
            UserDrawCaptionBar(pWnd, hDC, uFlags);
            goto Exit;
         }
         else if (uFlags & DC_DRAWFRAMEMD)
         {
            ERR("NC Paint Mode\n");
            NC_DoNCPaint(pWnd, hDC, uFlags); // Update Menus too!
            goto Exit;
         }
      }
      Ret = UserDrawCaption(pWnd, hDC, &SafeRect, hFont, hIcon, NULL, uFlags);
   }
Exit:
   UserLeave();
   return Ret;
}

BOOL
APIENTRY
NtUserDrawCaption(HWND hWnd,
   HDC hDC,
   LPCRECT lpRc,
   UINT uFlags)
{
   return NtUserDrawCaptionTemp(hWnd, hDC, lpRc, 0, 0, NULL, uFlags);
}

INT FASTCALL
co_UserExcludeUpdateRgn(HDC hDC, PWND Window)
{
    POINT pt;
    RECT rc;

    if (Window->hrgnUpdate)
    {
        if (Window->hrgnUpdate == HRGN_WINDOW)
        {
            return NtGdiIntersectClipRect(hDC, 0, 0, 0, 0);
        }
        else
        {
            INT ret = ERROR;
            HRGN hrgn = NtGdiCreateRectRgn(0,0,0,0);

            if ( hrgn && GreGetDCPoint( hDC, GdiGetDCOrg, &pt) )
            {
                if ( NtGdiGetRandomRgn( hDC, hrgn, CLIPRGN) == NULLREGION )
                {
                    NtGdiOffsetRgn(hrgn, pt.x, pt.y);
                }
                else
                {
                    HRGN hrgnScreen;
                    PMONITOR pm = UserGetPrimaryMonitor();
                    hrgnScreen = NtGdiCreateRectRgn(0,0,0,0);
                    NtGdiCombineRgn(hrgnScreen, hrgnScreen, pm->hrgnMonitor, RGN_OR);

                    NtGdiCombineRgn(hrgn, hrgnScreen, NULL, RGN_COPY);

                    GreDeleteObject(hrgnScreen);
                }

                NtGdiCombineRgn(hrgn, hrgn, Window->hrgnUpdate, RGN_DIFF);

                NtGdiOffsetRgn(hrgn, -pt.x, -pt.y);

                ret = NtGdiExtSelectClipRgn(hDC, hrgn, RGN_COPY);

                GreDeleteObject(hrgn);
            }
            return ret;
        }
    }
    else
    {
        return GdiGetClipBox( hDC, &rc);
    }
}

INT
APIENTRY
NtUserExcludeUpdateRgn(
    HDC hDC,
    HWND hWnd)
{
    INT ret = ERROR;
    PWND pWnd;

    TRACE("Enter NtUserExcludeUpdateRgn\n");
    UserEnterExclusive();

    pWnd = UserGetWindowObject(hWnd);

    if (hDC && pWnd)
        ret = co_UserExcludeUpdateRgn(hDC, pWnd);

    TRACE("Leave NtUserExcludeUpdateRgn, ret=%i\n", ret);

    UserLeave();
    return ret;
}

BOOL
APIENTRY
NtUserInvalidateRect(
    HWND hWnd,
    CONST RECT *lpUnsafeRect,
    BOOL bErase)
{
    UINT flags = RDW_INVALIDATE | (bErase ? RDW_ERASE : 0);
    if (!hWnd)
    {
       flags = RDW_ALLCHILDREN | RDW_INVALIDATE | RDW_FRAME | RDW_ERASE | RDW_ERASENOW;
       lpUnsafeRect = NULL;
    }
    return NtUserRedrawWindow(hWnd, lpUnsafeRect, NULL, flags);
}

BOOL
APIENTRY
NtUserInvalidateRgn(
    HWND hWnd,
    HRGN hRgn,
    BOOL bErase)
{
    if (!hWnd)
    {
       EngSetLastError( ERROR_INVALID_WINDOW_HANDLE );
       return FALSE;
    }
    return NtUserRedrawWindow(hWnd, NULL, hRgn, RDW_INVALIDATE | (bErase? RDW_ERASE : 0));
}

BOOL
APIENTRY
NtUserPrintWindow(
    HWND hwnd,
    HDC  hdcBlt,
    UINT nFlags)
{
    PWND Window;
    BOOL Ret = FALSE;

    UserEnterExclusive();

    if (hwnd)
    {
       if (!(Window = UserGetWindowObject(hwnd)) ||
            UserIsDesktopWindow(Window) || UserIsMessageWindow(Window))
       {
          goto Exit;
       }

       if ( Window )
       {
          /* Validate flags and check it as a mask for 0 or 1. */
          if ( (nFlags & PW_CLIENTONLY) == nFlags)
             Ret = IntPrintWindow( Window, hdcBlt, nFlags);
          else
             EngSetLastError(ERROR_INVALID_PARAMETER);
       }
    }
Exit:
    UserLeave();
    return Ret;
}

/* ValidateRect gets redirected to NtUserValidateRect:
   https://blog.csdn.net/ntdll/article/details/509299 */
BOOL
APIENTRY
NtUserValidateRect(
    HWND hWnd,
    const RECT *lpRect)
{
    UINT flags = RDW_VALIDATE;
    if (!hWnd)
    {
       flags = RDW_ALLCHILDREN | RDW_INVALIDATE | RDW_FRAME | RDW_ERASE | RDW_ERASENOW;
       lpRect = NULL;
    }
    return NtUserRedrawWindow(hWnd, lpRect, NULL, flags);
}

/* EOF */
