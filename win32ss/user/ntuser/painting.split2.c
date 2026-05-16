/* painting.split2.c — #include fragment of painting.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original painting.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


VOID FASTCALL
PaintSuspendedWindow(PWND pwnd, HRGN hrgnOrig)
{
   if (pwnd->hrgnUpdate)
   {
      HDC hDC;
      INT Flags = DC_NC|DC_NOSENDMSG;
      HRGN hrgnTemp;
      RECT Rect;
      INT type;
      PREGION prgn;

      if (pwnd->hrgnUpdate > HRGN_WINDOW)
      {
         hrgnTemp = NtGdiCreateRectRgn(0, 0, 0, 0);
         type = NtGdiCombineRgn( hrgnTemp, pwnd->hrgnUpdate, 0, RGN_COPY);
         if (type == ERROR)
         {
            GreDeleteObject(hrgnTemp);
            hrgnTemp = HRGN_WINDOW;
         }
      }
      else
      {
         hrgnTemp = GreCreateRectRgnIndirect(&pwnd->rcWindow);
      }

      if ( hrgnOrig &&
           hrgnTemp > HRGN_WINDOW &&
           NtGdiCombineRgn(hrgnTemp, hrgnTemp, hrgnOrig, RGN_AND) == NULLREGION)
      {
         GreDeleteObject(hrgnTemp);
         return;
      }

      hDC = UserGetDCEx(pwnd, hrgnTemp, DCX_WINDOW|DCX_INTERSECTRGN|DCX_USESTYLE|DCX_KEEPCLIPRGN);

      Rect = pwnd->rcWindow;
      RECTL_vOffsetRect(&Rect, -pwnd->rcWindow.left, -pwnd->rcWindow.top);

      // Clear out client area!
      FillRect(hDC, &Rect, IntGetSysColorBrush(COLOR_WINDOW));

      NC_DoNCPaint(pwnd, hDC, Flags); // Redraw without MENUs.

      UserReleaseDC(pwnd, hDC, FALSE);

      prgn = REGION_LockRgn(hrgnTemp);
      IntInvalidateWindows(pwnd, prgn, RDW_INVALIDATE | RDW_FRAME | RDW_ERASE | RDW_ALLCHILDREN);
      REGION_UnlockRgn(prgn);

      // Set updates for this window.
      pwnd->state |= WNDS_SENDNCPAINT|WNDS_SENDERASEBACKGROUND|WNDS_UPDATEDIRTY;

      // DCX_KEEPCLIPRGN is set. Check it anyway.
      if (hrgnTemp > HRGN_WINDOW && GreIsHandleValid(hrgnTemp)) GreDeleteObject(hrgnTemp);
   }
}

VOID FASTCALL
UpdateTheadChildren(PWND pWnd, HRGN hRgn)
{
   PaintSuspendedWindow( pWnd, hRgn );

   if (!(pWnd->style & WS_CLIPCHILDREN))
      return;

   pWnd = pWnd->spwndChild; // invalidate children if any.
   while (pWnd)
   {
      UpdateTheadChildren( pWnd, hRgn );
      pWnd = pWnd->spwndNext;
   }
}

VOID FASTCALL
UpdateThreadWindows(PWND pWnd, PTHREADINFO pti, HRGN hRgn)
{
   PWND pwndTemp;

   for ( pwndTemp = pWnd;
         pwndTemp;
         pwndTemp = pwndTemp->spwndNext )
   {
      if (pwndTemp->head.pti == pti)
      {
          UserUpdateWindows(pwndTemp, RDW_ALLCHILDREN);
      }
      else
      {
          if (IsThreadSuspended(pwndTemp->head.pti) || MsqIsHung(pwndTemp->head.pti, MSQ_HUNG))
          {
             UpdateTheadChildren(pwndTemp, hRgn);
          }
          else
             UserUpdateWindows(pwndTemp, RDW_ALLCHILDREN);
      }
   }
}

BOOL FASTCALL
IntIsWindowDirty(PWND Wnd)
{
   return ( Wnd->style & WS_VISIBLE &&
           ( Wnd->hrgnUpdate != NULL ||
             Wnd->state & WNDS_INTERNALPAINT ) );
}

/*
   Conditions to paint any window:

   1. Update region is not null.
   2. Internal paint flag is set.
   3. Paint count is not zero.

 */
PWND FASTCALL
IntFindWindowToRepaint(PWND Window, PTHREADINFO Thread)
{
   PWND hChild;
   PWND TempWindow;

   for (; Window != NULL; Window = Window->spwndNext)
   {
      if (IntWndBelongsToThread(Window, Thread))
      {
         if (IntIsWindowDirty(Window))
         {
            /* Make sure all non-transparent siblings are already drawn. */
            if (Window->ExStyle & WS_EX_TRANSPARENT)
            {
               for (TempWindow = Window->spwndNext; TempWindow != NULL;
                    TempWindow = TempWindow->spwndNext)
               {
                  if (!(TempWindow->ExStyle & WS_EX_TRANSPARENT) &&
                       IntWndBelongsToThread(TempWindow, Thread) &&
                       IntIsWindowDirty(TempWindow))
                  {
                     return TempWindow;
                  }
               }
            }
            return Window;
         }
      }
      /* find a child of the specified window that needs repainting */
      if (Window->spwndChild)
      {
         hChild = IntFindWindowToRepaint(Window->spwndChild, Thread);
         if (hChild != NULL)
            return hChild;
      }
   }
   return Window;
}

//
// Internal painting of windows.
//
VOID FASTCALL
IntPaintWindow( PWND Window )
{
   // Handle normal painting.
   co_IntPaintWindows( Window, RDW_NOCHILDREN, FALSE );
}

BOOL FASTCALL
IntGetPaintMessage(
   PWND Window,
   UINT MsgFilterMin,
   UINT MsgFilterMax,
   PTHREADINFO Thread,
   MSG *Message,
   BOOL Remove)
{
   PWND PaintWnd, StartWnd;

   if ((MsgFilterMin != 0 || MsgFilterMax != 0) &&
         (MsgFilterMin > WM_PAINT || MsgFilterMax < WM_PAINT))
      return FALSE;

   if (Thread->TIF_flags & TIF_SYSTEMTHREAD )
   {
      ERR("WM_PAINT is in a System Thread!\n");
   }

   StartWnd = UserGetDesktopWindow();
   PaintWnd = IntFindWindowToRepaint(StartWnd, Thread);

   Message->hwnd = PaintWnd ? UserHMGetHandle(PaintWnd) : NULL;

   if (Message->hwnd == NULL && Thread->cPaintsReady)
   {
      // Find note in window.c:"PAINTING BUG".
      ERR("WARNING SOMETHING HAS GONE WRONG: Thread marked as containing dirty windows, but no dirty windows found! Counts %u\n",Thread->cPaintsReady);
      /* Hack to stop spamming the debug log ! */
      Thread->cPaintsReady = 0;
      return FALSE;
   }

   if (Message->hwnd == NULL)
      return FALSE;

   if (!(Window == NULL ||
         PaintWnd == Window ||
         IntIsChildWindow(Window, PaintWnd))) /* check that it is a child of the specified parent */
      return FALSE;

   if (PaintWnd->state & WNDS_INTERNALPAINT)
   {
      PaintWnd->state &= ~WNDS_INTERNALPAINT;
      if (!PaintWnd->hrgnUpdate)
         MsqDecPaintCountQueue(Thread);
   }
   PaintWnd->state2 &= ~WNDS2_STARTPAINT;
   PaintWnd->state &= ~WNDS_UPDATEDIRTY;

   Window = PaintWnd;
   while (Window && !UserIsDesktopWindow(Window))
   {
      // Role back and check for clip children, do not set if any.
      if (Window->spwndParent && !(Window->spwndParent->style & WS_CLIPCHILDREN))
      {
         PaintWnd->state2 |= WNDS2_WMPAINTSENT;
      }
      Window = Window->spwndParent;
   }

   Message->wParam = Message->lParam = 0;
   Message->message = WM_PAINT;
   return TRUE;
}

BOOL
FASTCALL
IntPrintWindow(
    PWND pwnd,
    HDC hdcBlt,
    UINT nFlags)
{
    HDC hdcSrc;
    INT cx, cy, xSrc, ySrc;

    if ( nFlags & PW_CLIENTONLY)
    {
       cx = pwnd->rcClient.right - pwnd->rcClient.left;
       cy = pwnd->rcClient.bottom - pwnd->rcClient.top;
       xSrc = pwnd->rcClient.left - pwnd->rcWindow.left;
       ySrc = pwnd->rcClient.top - pwnd->rcWindow.top;
    }
    else
    {
       cx = pwnd->rcWindow.right - pwnd->rcWindow.left;
       cy = pwnd->rcWindow.bottom - pwnd->rcWindow.top;
       xSrc = 0;
       ySrc = 0;
    }

    // TODO: Setup Redirection for Print.
    return FALSE;

    /* Update the window just incase. */
    co_IntUpdateWindows( pwnd, RDW_ALLCHILDREN, FALSE);

    hdcSrc = UserGetDCEx( pwnd, NULL, DCX_CACHE|DCX_WINDOW);
    /* Print window to printer context. */
    NtGdiBitBlt( hdcBlt,
                 0,
                 0,
                 cx,
                 cy,
                 hdcSrc,
                 xSrc,
                 ySrc,
                 SRCCOPY,
                 CLR_INVALID,
                 0);

    UserReleaseDC( pwnd, hdcSrc, FALSE);

    // TODO: Release Redirection from Print.

    return TRUE;
}

BOOL
FASTCALL
IntFlashWindowEx(PWND pWnd, PFLASHWINFO pfwi)
{
   DWORD_PTR FlashState;
   UINT uCount = pfwi->uCount;
   BOOL Activate = FALSE, Ret = FALSE;

   ASSERT(pfwi);

   FlashState = (DWORD_PTR)UserGetProp(pWnd, AtomFlashWndState, TRUE);

   if (FlashState == FLASHW_FINISHED)
   {
      // Cycle has finished, kill timer and set this to Stop.
      FlashState |= FLASHW_KILLSYSTIMER;
      pfwi->dwFlags = FLASHW_STOP;
   }
   else
   {
      if (FlashState)
      {
         if (pfwi->dwFlags == FLASHW_SYSTIMER)
         {
             // Called from system timer, restore flags, counts and state.
             pfwi->dwFlags = LOWORD(FlashState);
             uCount = HIWORD(FlashState);
             FlashState = MAKELONG(LOWORD(FlashState),0);
         }
         else
         {
             // Clean out the trash! Fix SeaMonkey crash after restart.
             FlashState = 0;
         }
      }

      if (FlashState == 0)
      {  // First time in cycle, setup flash state.
         if ( pWnd->state & WNDS_ACTIVEFRAME ||
             (pfwi->dwFlags & FLASHW_CAPTION && pWnd->style & (WS_BORDER|WS_DLGFRAME)))
         {
             FlashState = FLASHW_STARTED|FLASHW_ACTIVE;
         }
      }

      // Set previous window state.
      Ret = !!(FlashState & FLASHW_ACTIVE);

      if ( (pfwi->dwFlags & FLASHW_TIMERNOFG) == FLASHW_TIMERNOFG &&
           gpqForeground == pWnd->head.pti->MessageQueue )
      {
          // Flashing until foreground, set this to Stop.
          pfwi->dwFlags = FLASHW_STOP;
      }
   }

   // Toggle activate flag.
   if ( pfwi->dwFlags == FLASHW_STOP )
   {
      if (gpqForeground && gpqForeground->spwndActive == pWnd)
         Activate = TRUE;
      else
         Activate = FALSE;
   }
   else
   {
      Activate = (FlashState & FLASHW_ACTIVE) == 0;
   }

   if ( pfwi->dwFlags == FLASHW_STOP || pfwi->dwFlags & FLASHW_CAPTION )
   {
      co_IntSendMessage(UserHMGetHandle(pWnd), WM_NCACTIVATE, Activate, 0);
   }

   // FIXME: Check for a Stop Sign here.
   if ( pfwi->dwFlags & FLASHW_TRAY )
   {
      // Need some shell work here too.
      TRACE("FIXME: Flash window no Tray support!\n");
   }

   if ( pfwi->dwFlags == FLASHW_STOP )
   {
      if (FlashState & FLASHW_KILLSYSTIMER)
      {
         IntKillTimer(pWnd, ID_EVENT_SYSTIMER_FLASHWIN, TRUE);
      }

      UserRemoveProp(pWnd, AtomFlashWndState, TRUE);
   }
   else
   {  // Have a count and started, set timer.
      if ( uCount )
      {
         FlashState |= FLASHW_COUNT;

         if (!(Activate ^ !!(FlashState & FLASHW_STARTED)))
             uCount--;

         if (!(FlashState & FLASHW_KILLSYSTIMER))
             pfwi->dwFlags |= FLASHW_TIMER;
      }

      if (pfwi->dwFlags & FLASHW_TIMER)
      {
         FlashState |= FLASHW_KILLSYSTIMER;

         IntSetTimer( pWnd,
                      ID_EVENT_SYSTIMER_FLASHWIN,
                      pfwi->dwTimeout ? pfwi->dwTimeout : gpsi->dtCaretBlink,
                      SystemTimerProc,
                      TMRF_SYSTEM );
      }

      if (FlashState & FLASHW_COUNT && uCount == 0)
      {
         // Keep spinning? Nothing else to do.
         FlashState = FLASHW_FINISHED;
      }
      else
      {
         // Save state and flags so this can be restored next time through.
         FlashState ^= (FlashState ^ -!!(Activate)) & FLASHW_ACTIVE;
         FlashState ^= (FlashState ^ pfwi->dwFlags) & (FLASHW_MASK & ~FLASHW_TIMER);
      }
      FlashState = MAKELONG(LOWORD(FlashState),uCount);
      UserSetProp(pWnd, AtomFlashWndState, (HANDLE)FlashState, TRUE);
   }
   return Ret;
}

// Win: xxxBeginPaint
HDC FASTCALL
IntBeginPaint(PWND Window, PPAINTSTRUCT Ps)
{
   RECT Rect;
   INT type;
   BOOL Erase = FALSE;

   co_UserHideCaret(Window);

   Window->state2 |= WNDS2_STARTPAINT;
   Window->state &= ~WNDS_PAINTNOTPROCESSED;

   if (Window->state & WNDS_SENDNCPAINT)
   {
      HRGN hRgn;
      // Application can keep update dirty.
      do
      {
         Window->state &= ~WNDS_UPDATEDIRTY;
         hRgn = IntGetNCUpdateRgn(Window, FALSE);
         IntSendNCPaint(Window, hRgn);
         if (hRgn > HRGN_WINDOW && GreIsHandleValid(hRgn))
         {
            /* NOTE: The region can already be deleted! */
            GreDeleteObject(hRgn);
         }
      }
      while(Window->state & WNDS_UPDATEDIRTY);
   }
   else
   {
      Window->state &= ~WNDS_UPDATEDIRTY;
   }

   RtlZeroMemory(Ps, sizeof(PAINTSTRUCT));

   if (Window->state2 & WNDS2_ENDPAINTINVALIDATE)
   {
      ERR("BP: Another thread invalidated this window\n");
   }

   Ps->hdc = UserGetDCEx( Window,
                          Window->hrgnUpdate,
                          DCX_INTERSECTRGN | DCX_USESTYLE);
   if (!Ps->hdc)
   {
      return NULL;
   }

   // If set, always clear flags out due to the conditions later on for sending the message.
   if (Window->state & WNDS_SENDERASEBACKGROUND)
   {
      Window->state &= ~(WNDS_SENDERASEBACKGROUND|WNDS_ERASEBACKGROUND);
      Erase = TRUE;
   }

   if (Window->hrgnUpdate != NULL)
   {
      MsqDecPaintCountQueue(Window->head.pti);
      IntGdiSetRegionOwner(Window->hrgnUpdate, GDI_OBJ_HMGR_POWNED);
      /* The region is part of the dc now and belongs to the process! */
      Window->hrgnUpdate = NULL;
   }
   else
   {
      if (Window->state & WNDS_INTERNALPAINT)
         MsqDecPaintCountQueue(Window->head.pti);
   }

   type = GdiGetClipBox(Ps->hdc, &Ps->rcPaint);

   IntGetClientRect(Window, &Rect);

   Window->state &= ~WNDS_INTERNALPAINT;

   if ( Erase &&               // Set to erase,
        type != NULLREGION &&  // don't erase if the clip box is empty,
        (!(Window->pcls->style & CS_PARENTDC) || // not parent dc or
         RECTL_bIntersectRect( &Rect, &Rect, &Ps->rcPaint) ) ) // intersecting.
   {
      Ps->fErase = !co_IntSendMessage(UserHMGetHandle(Window), WM_ERASEBKGND, (WPARAM)Ps->hdc, 0);
      if ( Ps->fErase )
      {
         Window->state |= (WNDS_SENDERASEBACKGROUND|WNDS_ERASEBACKGROUND);
      }
   }
   else
   {
      Ps->fErase = FALSE;
   }

   IntSendChildNCPaint(Window);

   return Ps->hdc;
}

// Win: xxxEndPaint
BOOL FASTCALL
IntEndPaint(PWND Wnd, PPAINTSTRUCT Ps)
{
   HDC hdc = NULL;

   hdc = Ps->hdc;

   UserReleaseDC(Wnd, hdc, TRUE);

   if (Wnd->state2 & WNDS2_ENDPAINTINVALIDATE)
   {
      ERR("EP: Another thread invalidated this window\n");
      Wnd->state2 &= ~WNDS2_ENDPAINTINVALIDATE;
   }

   Wnd->state2 &= ~(WNDS2_WMPAINTSENT|WNDS2_STARTPAINT);

   co_UserShowCaret(Wnd);

   return TRUE;
}

// Win: xxxFillWindow
BOOL FASTCALL
IntFillWindow(PWND pWndParent,
              PWND pWnd,
              HDC  hDC,
              HBRUSH hBrush)
{
   RECT Rect, Rect1;
   INT type;

   if (!pWndParent)
      pWndParent = pWnd;

   type = GdiGetClipBox(hDC, &Rect);

   IntGetClientRect(pWnd, &Rect1);

   if ( type != NULLREGION && // Clip box is not empty,
       (!(pWnd->pcls->style & CS_PARENTDC) || // not parent dc or
         RECTL_bIntersectRect( &Rect, &Rect, &Rect1) ) ) // intersecting.
   {
      POINT ppt;
      INT x = 0, y = 0;

      if (!UserIsDesktopWindow(pWndParent))
      {
          x = pWndParent->rcClient.left - pWnd->rcClient.left;
          y = pWndParent->rcClient.top  - pWnd->rcClient.top;
      }

      GreSetBrushOrg(hDC, x, y, &ppt);

      if ( hBrush < (HBRUSH)CTLCOLOR_MAX )
          hBrush = GetControlColor( pWndParent, pWnd, hDC, HandleToUlong(hBrush) + WM_CTLCOLORMSGBOX);

      FillRect(hDC, &Rect, hBrush);

      GreSetBrushOrg(hDC, ppt.x, ppt.y, NULL);

      return TRUE;
   }
   else
      return FALSE;
}

/* PUBLIC FUNCTIONS ***********************************************************/

/*
 * NtUserBeginPaint
 *
 * Status
 *    @implemented
 */

HDC APIENTRY
NtUserBeginPaint(HWND hWnd, PAINTSTRUCT* UnsafePs)
{
   PWND Window;
   PAINTSTRUCT Ps;
   NTSTATUS Status;
   HDC hDC;
   USER_REFERENCE_ENTRY Ref;
   HDC Ret = NULL;

   TRACE("Enter NtUserBeginPaint\n");
   UserEnterExclusive();

   if (!(Window = UserGetWindowObject(hWnd)))
   {
      goto Cleanup; // Return NULL
   }

   UserRefObjectCo(Window, &Ref);

   hDC = IntBeginPaint(Window, &Ps);

   Status = MmCopyToCaller(UnsafePs, &Ps, sizeof(PAINTSTRUCT));
   if (! NT_SUCCESS(Status))
   {
      SetLastNtError(Status);
      goto Cleanup; // Return NULL
   }

   Ret = hDC;

Cleanup:
   if (Window) UserDerefObjectCo(Window);

   TRACE("Leave NtUserBeginPaint, ret=%p\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * NtUserEndPaint
 *
 * Status
 *    @implemented
 */

BOOL APIENTRY
NtUserEndPaint(HWND hWnd, CONST PAINTSTRUCT* pUnsafePs)
{
   NTSTATUS Status = STATUS_SUCCESS;
   PWND Window;
   PAINTSTRUCT Ps;
   USER_REFERENCE_ENTRY Ref;
   BOOL Ret = FALSE;

   TRACE("Enter NtUserEndPaint\n");
   UserEnterExclusive();

   if (!(Window = UserGetWindowObject(hWnd)))
   {
      goto Cleanup; // Return FALSE
   }

   UserRefObjectCo(Window, &Ref); // Here for the exception.

   Status = SafeProbeForRead(pUnsafePs, sizeof(*pUnsafePs), 1);
   if (NT_SUCCESS(Status))
   {
       RtlCopyMemory(&Ps, pUnsafePs, sizeof(PAINTSTRUCT));
   }
   if (!NT_SUCCESS(Status))
   {
      goto Cleanup; // Return FALSE
   }

   Ret = IntEndPaint(Window, &Ps);

Cleanup:
   if (Window) UserDerefObjectCo(Window);

   TRACE("Leave NtUserEndPaint, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * FillWindow: Called from User; Dialog, Edit and ListBox procs during a WM_ERASEBKGND.
 */
/*
 * @implemented
 */
BOOL APIENTRY
NtUserFillWindow(HWND hWndParent,
                 HWND hWnd,
                 HDC  hDC,
                 HBRUSH hBrush)
{
   BOOL ret = FALSE;
   PWND pWnd, pWndParent = NULL;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserFillWindow\n");
   UserEnterExclusive();

   if (!hDC)
   {
      goto Exit;
   }

   if (!(pWnd = UserGetWindowObject(hWnd)))
   {
      goto Exit;
   }

   if (hWndParent && !(pWndParent = UserGetWindowObject(hWndParent)))
   {
      goto Exit;
   }

   UserRefObjectCo(pWnd, &Ref);
   ret = IntFillWindow( pWndParent, pWnd, hDC, hBrush );
   UserDerefObjectCo(pWnd);

Exit:
   TRACE("Leave NtUserFillWindow, ret=%i\n",ret);
   UserLeave();
   return ret;
}

/*
 * @implemented
 */
BOOL APIENTRY
NtUserFlashWindowEx(IN PFLASHWINFO pfwi)
{
   PWND pWnd;
   FLASHWINFO finfo = {0};
   BOOL Ret = FALSE;

   UserEnterExclusive();

   {
       NTSTATUS __SehStatus = SafeProbeForRead(pfwi, sizeof(FLASHWINFO), 1);
       if (!NT_SUCCESS(__SehStatus))
       {
           SetLastNtError(__SehStatus);
           goto Exit;
       }
   }
   RtlCopyMemory(&finfo, pfwi, sizeof(FLASHWINFO));

   if (!( pWnd = ValidateHwndNoErr(finfo.hwnd)) ||
        finfo.cbSize != sizeof(FLASHWINFO) ||
        finfo.dwFlags & ~(FLASHW_ALL|FLASHW_TIMER|FLASHW_TIMERNOFG) )
   {
      EngSetLastError(ERROR_INVALID_PARAMETER);
      goto Exit;
   }

   Ret = IntFlashWindowEx(pWnd, &finfo);

Exit:
   UserLeave();
   return Ret;
}

/*
    GetUpdateRgn, this fails the same as the old one.
 */
INT FASTCALL
co_UserGetUpdateRgn(PWND Window, HRGN hRgn, BOOL bErase)
{
   int RegionType;
   BOOL Type;
   RECTL Rect;

   ASSERT_REFS_CO(Window);

   if (bErase)
   {
      USER_REFERENCE_ENTRY Ref;
      UserRefObjectCo(Window, &Ref);
      co_IntPaintWindows(Window, RDW_NOCHILDREN, FALSE);
      UserDerefObjectCo(Window);
   }

   Window->state &= ~WNDS_UPDATEDIRTY;

   if (Window->hrgnUpdate == NULL)
   {
       NtGdiSetRectRgn(hRgn, 0, 0, 0, 0);
       return NULLREGION;
   }

   Rect = Window->rcClient;
   Type = IntIntersectWithParents(Window, &Rect);

   if (Window->hrgnUpdate == HRGN_WINDOW)
   {
      // Trap it out.
      ERR("GURn: Caller is passing Window Region 1\n");
      if (!Type)
      {
         NtGdiSetRectRgn(hRgn, 0, 0, 0, 0);
         return NULLREGION;
      }

      RegionType = SIMPLEREGION;

      if (!UserIsDesktopWindow(Window))
      {
         RECTL_vOffsetRect(&Rect,
                          -Window->rcClient.left,
                          -Window->rcClient.top);
      }
      GreSetRectRgnIndirect(hRgn, &Rect);
   }
   else
   {
      HRGN hrgnTemp = GreCreateRectRgnIndirect(&Rect);

      RegionType = NtGdiCombineRgn(hRgn, hrgnTemp, Window->hrgnUpdate, RGN_AND);

      if (RegionType == ERROR || RegionType == NULLREGION)
      {
         if (hrgnTemp) GreDeleteObject(hrgnTemp);
         NtGdiSetRectRgn(hRgn, 0, 0, 0, 0);
         return RegionType;
      }

      if (!UserIsDesktopWindow(Window))
      {
         NtGdiOffsetRgn(hRgn,
                       -Window->rcClient.left,
                       -Window->rcClient.top);
      }
      if (hrgnTemp) GreDeleteObject(hrgnTemp);
   }
   return RegionType;
}

BOOL FASTCALL
co_UserGetUpdateRect(PWND Window, PRECT pRect, BOOL bErase)
{
   INT RegionType;
   BOOL Ret = TRUE;

   if (bErase)
   {
      USER_REFERENCE_ENTRY Ref;
      UserRefObjectCo(Window, &Ref);
      co_IntPaintWindows(Window, RDW_NOCHILDREN, FALSE);
      UserDerefObjectCo(Window);
   }

   Window->state &= ~WNDS_UPDATEDIRTY;

   if (Window->hrgnUpdate == NULL)
   {
      pRect->left = pRect->top = pRect->right = pRect->bottom = 0;
      Ret = FALSE;
   }
   else
   {
      /* Get the update region bounding box. */
      if (Window->hrgnUpdate == HRGN_WINDOW)
      {
         *pRect = Window->rcClient;
         ERR("GURt: Caller is retrieving Window Region 1\n");
      }
      else
      {
         RegionType = IntGdiGetRgnBox(Window->hrgnUpdate, pRect);

         if (RegionType != ERROR && RegionType != NULLREGION)
            RECTL_bIntersectRect(pRect, pRect, &Window->rcClient);
      }

      if (IntIntersectWithParents(Window, pRect))
      {
         if (!UserIsDesktopWindow(Window))
         {
            RECTL_vOffsetRect(pRect,
                              -Window->rcClient.left,
                              -Window->rcClient.top);
         }
         if (Window->pcls->style & CS_OWNDC)
         {
            HDC hdc;
            //DWORD layout;
            hdc = UserGetDCEx(Window, NULL, DCX_USESTYLE);
            //layout = NtGdiSetLayout(hdc, -1, 0);
            //IntMapWindowPoints( 0, Window, (LPPOINT)pRect, 2 );
            GreDPtoLP( hdc, (LPPOINT)pRect, 2 );
            //NtGdiSetLayout(hdc, -1, layout);
            UserReleaseDC(Window, hdc, FALSE);
         }
      }
      else
      {
         pRect->left = pRect->top = pRect->right = pRect->bottom = 0;
      }
   }
   return Ret;
}

/*
 * NtUserGetUpdateRgn
 *
 * Status
 *    @implemented
 */

INT APIENTRY
NtUserGetUpdateRgn(HWND hWnd, HRGN hRgn, BOOL bErase)
{
   PWND Window;
   INT ret = ERROR;

   TRACE("Enter NtUserGetUpdateRgn\n");
   UserEnterExclusive();

   Window = UserGetWindowObject(hWnd);
   if (Window)
   {
      ret = co_UserGetUpdateRgn(Window, hRgn, bErase);
   }

   TRACE("Leave NtUserGetUpdateRgn, ret=%i\n", ret);
   UserLeave();
   return ret;
}

/*
 * NtUserGetUpdateRect
 *
 * Status
 *    @implemented
 */

BOOL APIENTRY
NtUserGetUpdateRect(HWND hWnd, LPRECT UnsafeRect, BOOL bErase)
{
   PWND Window;
   RECTL Rect;
   NTSTATUS Status;
   BOOL Ret = FALSE;

   TRACE("Enter NtUserGetUpdateRect\n");
   UserEnterExclusive();

   if (!(Window = UserGetWindowObject(hWnd)))
   {
      goto Exit; // Return FALSE
   }

   Ret = co_UserGetUpdateRect(Window, &Rect, bErase);

   if (UnsafeRect != NULL)
   {
      Status = MmCopyToCaller(UnsafeRect, &Rect, sizeof(RECTL));
      if (!NT_SUCCESS(Status))
      {
         EngSetLastError(ERROR_INVALID_PARAMETER);
         Ret = FALSE;
      }
   }

Exit:
   TRACE("Leave NtUserGetUpdateRect, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * NtUserRedrawWindow
 *
 * Status
 *    @implemented
 */

BOOL APIENTRY
NtUserRedrawWindow(
   HWND hWnd,
   CONST RECT *lprcUpdate,
   HRGN hrgnUpdate,
   UINT flags)
{
   RECTL SafeUpdateRect;
   PWND Wnd;
   BOOL Ret = FALSE;
   USER_REFERENCE_ENTRY Ref;
   NTSTATUS Status = STATUS_SUCCESS;
   PREGION RgnUpdate = NULL;

   TRACE("Enter NtUserRedrawWindow\n");
   UserEnterExclusive();

   if (!(Wnd = UserGetWindowObject(hWnd ? hWnd : IntGetDesktopWindow())))
   {
      goto Exit; // Return FALSE
   }

   if (lprcUpdate)
   {
      Status = SafeProbeForRead(lprcUpdate, sizeof(RECTL), 1);
      if (NT_SUCCESS(Status))
      {
          RtlCopyMemory(&SafeUpdateRect, lprcUpdate, sizeof(RECTL));
      }
      if (!NT_SUCCESS(Status))
      {
         EngSetLastError(RtlNtStatusToDosError(Status));
         goto Exit; // Return FALSE
      }
   }

   if ( flags & ~(RDW_ERASE|RDW_FRAME|RDW_INTERNALPAINT|RDW_INVALIDATE|
                  RDW_NOERASE|RDW_NOFRAME|RDW_NOINTERNALPAINT|RDW_VALIDATE|
                  RDW_ERASENOW|RDW_UPDATENOW|RDW_ALLCHILDREN|RDW_NOCHILDREN) )
   {
      /* RedrawWindow fails only in case that flags are invalid */
      EngSetLastError(ERROR_INVALID_FLAGS);
      goto Exit; // Return FALSE
   }

   /* We can't hold lock on GDI objects while doing roundtrips to user mode,
    * so it will be copied.
    */
   if (hrgnUpdate > HRGN_WINDOW)
   {
       RgnUpdate = REGION_LockRgn(hrgnUpdate);
       if (!RgnUpdate)
       {
           EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
           goto Exit; // Return FALSE
       }
       REGION_UnlockRgn(RgnUpdate);
   }
   else if (hrgnUpdate == HRGN_WINDOW) // Trap it out.
   {
       ERR("NTRW: Caller is passing Window Region 1\n");
   }

   UserRefObjectCo(Wnd, &Ref);

   Ret = co_UserRedrawWindow( Wnd,
                              lprcUpdate ? &SafeUpdateRect : NULL,
                              RgnUpdate,
                              flags);

   UserDerefObjectCo(Wnd);

Exit:
   TRACE("Leave NtUserRedrawWindow, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}