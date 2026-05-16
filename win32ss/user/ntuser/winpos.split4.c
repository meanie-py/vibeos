/* winpos.split4.c — #include fragment of winpos.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original winpos.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


/*
 * @implemented
 */
HDWP APIENTRY
NtUserDeferWindowPos(HDWP WinPosInfo,
                     HWND Wnd,
                     HWND WndInsertAfter,
                     int x,
                     int y,
                     int cx,
                     int cy,
                     UINT Flags)
{
   PWND pWnd, pWndIA;
   HDWP Ret = NULL;
   UINT Tmp = ~(SWP_ASYNCWINDOWPOS|SWP_DEFERERASE|SWP_NOSENDCHANGING|SWP_NOREPOSITION|
                SWP_NOCOPYBITS|SWP_HIDEWINDOW|SWP_SHOWWINDOW|SWP_FRAMECHANGED|
                SWP_NOACTIVATE|SWP_NOREDRAW|SWP_NOZORDER|SWP_NOMOVE|SWP_NOSIZE);

   TRACE("Enter NtUserDeferWindowPos\n");
   UserEnterExclusive();

   if ( Flags & Tmp )
   {
      EngSetLastError(ERROR_INVALID_FLAGS);
      goto Exit;
   }

   pWnd = UserGetWindowObject(Wnd);
   if (!pWnd || UserIsDesktopWindow(pWnd) || UserIsMessageWindow(pWnd))
   {
      goto Exit;
   }

   if ( WndInsertAfter &&
        WndInsertAfter != HWND_BOTTOM &&
        WndInsertAfter != HWND_TOPMOST &&
        WndInsertAfter != HWND_NOTOPMOST )
   {
      pWndIA = UserGetWindowObject(WndInsertAfter);
      if (!pWndIA || UserIsDesktopWindow(pWndIA) || UserIsMessageWindow(pWndIA))
      {
         goto Exit;
      }
   }

   Ret = IntDeferWindowPos(WinPosInfo, Wnd, WndInsertAfter, x, y, cx, cy, Flags);

Exit:
   TRACE("Leave NtUserDeferWindowPos, ret=%p\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * @implemented
 */
DWORD APIENTRY
NtUserGetInternalWindowPos( HWND hWnd,
                            LPRECT rectWnd,
                            LPPOINT ptIcon)
{
   PWND Window;
   DWORD Ret = 0;
   BOOL Hit = FALSE;
   WINDOWPLACEMENT wndpl;
   NTSTATUS Status;

   UserEnterShared();

   if (!(Window = UserGetWindowObject(hWnd)))
   {
      Hit = FALSE;
      goto Exit;
   }

   Status = STATUS_SUCCESS;
   if (rectWnd)
   {
       Status = SafeProbeForWrite(rectWnd,
                                  sizeof(RECT),
                                  1);
   }
   if (NT_SUCCESS(Status) && ptIcon)
   {
       Status = SafeProbeForWrite(ptIcon,
                                  sizeof(POINT),
                                  1);
   }
   if (!NT_SUCCESS(Status))
   {
       SetLastNtError(Status);
       Hit = TRUE;
   }

   wndpl.length = sizeof(WINDOWPLACEMENT);

   if (IntGetWindowPlacement(Window, &wndpl) && !Hit)
   {
      _SEH2_TRY
      {
          if (rectWnd)
          {
             RtlCopyMemory(rectWnd, &wndpl.rcNormalPosition , sizeof(RECT));
          }
          if (ptIcon)
          {
             RtlCopyMemory(ptIcon, &wndpl.ptMinPosition, sizeof(POINT));
          }

      }
      _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
      {
          SetLastNtError(_SEH2_GetExceptionCode());
          Hit = TRUE;
      }
      _SEH2_END;

      if (!Hit) Ret = wndpl.showCmd;
   }
Exit:
   UserLeave();
   return Ret;
}


/*
 * @implemented
 */
BOOL APIENTRY
NtUserGetWindowPlacement(HWND hWnd,
                         WINDOWPLACEMENT *lpwndpl)
{
   PWND Wnd;
   WINDOWPLACEMENT Safepl;
   NTSTATUS Status;
   BOOL Ret = FALSE;

   TRACE("Enter NtUserGetWindowPlacement\n");
   UserEnterShared();

   if (!(Wnd = UserGetWindowObject(hWnd)))
      goto Exit; // Return FALSE

   Status = MmCopyFromCaller(&Safepl, lpwndpl, sizeof(WINDOWPLACEMENT));
   if (!NT_SUCCESS(Status))
   {
      SetLastNtError(Status);
      goto Exit; // Return FALSE
   }

   // This function doesn't check the length. Just overwrite it
   Safepl.length = sizeof(WINDOWPLACEMENT);

   IntGetWindowPlacement(Wnd, &Safepl);

   Status = MmCopyToCaller(lpwndpl, &Safepl, sizeof(WINDOWPLACEMENT));
   if (!NT_SUCCESS(Status))
   {
      SetLastNtError(Status);
      goto Exit; // Return FALSE
   }

   Ret = TRUE;

Exit:
   TRACE("Leave NtUserGetWindowPlacement, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

DWORD
APIENTRY
NtUserMinMaximize(
    HWND hWnd,
    UINT cmd, // Wine SW_ commands
    BOOL Hide)
{
  PWND pWnd;

  TRACE("Enter NtUserMinMaximize\n");
  UserEnterExclusive();

  pWnd = UserGetWindowObject(hWnd);
  if (!pWnd || UserIsDesktopWindow(pWnd) || UserIsMessageWindow(pWnd))
  {
     goto Exit;
  }

  if ( cmd > SW_MAX || pWnd->state2 & WNDS2_INDESTROY)
  {
     EngSetLastError(ERROR_INVALID_PARAMETER);
     goto Exit;
  }

  cmd |= Hide ? SW_HIDE : 0;

  co_WinPosShowWindow(pWnd, cmd);

Exit:
  TRACE("Leave NtUserMinMaximize\n");
  UserLeave();
  return 0; // Always NULL?
}

/*
 * @implemented
 */
BOOL APIENTRY
NtUserMoveWindow(
   HWND hWnd,
   int X,
   int Y,
   int nWidth,
   int nHeight,
   BOOL bRepaint)
{
   return NtUserSetWindowPos(hWnd, 0, X, Y, nWidth, nHeight,
                             (bRepaint ? SWP_NOZORDER | SWP_NOACTIVATE :
                              SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOREDRAW));
}

/*
 * @implemented
 */
HWND APIENTRY
NtUserRealChildWindowFromPoint(HWND Parent,
                               LONG x,
                               LONG y)
{
   PWND pwndParent;
   TRACE("Enter NtUserRealChildWindowFromPoint\n");
   UserEnterShared();
   if ((pwndParent = UserGetWindowObject(Parent)))
   {
      pwndParent = IntRealChildWindowFromPoint(pwndParent, x, y);
   }
   UserLeave();
   TRACE("Leave NtUserRealChildWindowFromPoint\n");
   return pwndParent ? UserHMGetHandle(pwndParent) : NULL;
}

/*
 * @implemented
 */
BOOL APIENTRY
NtUserSetWindowPos(
   HWND hWnd,
   HWND hWndInsertAfter,
   int X,
   int Y,
   int cx,
   int cy,
   UINT uFlags)
{
   PWND Window, pWndIA;
   BOOL ret = FALSE;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserSetWindowPos\n");
   UserEnterExclusive();

   if (!(Window = UserGetWindowObject(hWnd)) ||
        UserIsDesktopWindow(Window) || UserIsMessageWindow(Window))
   {
      ERR("NtUserSetWindowPos bad window handle!\n");
      goto Exit; // Return FALSE
   }

   if ( hWndInsertAfter != HWND_TOP &&
        hWndInsertAfter != HWND_BOTTOM &&
        hWndInsertAfter != HWND_TOPMOST &&
        hWndInsertAfter != HWND_NOTOPMOST )
   {
      if (!(pWndIA = UserGetWindowObject(hWndInsertAfter)) ||
            UserIsDesktopWindow(pWndIA) || UserIsMessageWindow(pWndIA))
      {
         ERR("NtUserSetWindowPos bad insert window handle!\n");
         goto Exit; // Return FALSE
      }
   }

   /* First make sure that coordinates are valid for WM_WINDOWPOSCHANGING */
   if (!(uFlags & SWP_NOMOVE))
   {
      if (X < -32768) X = -32768;
      else if (X > 32767) X = 32767;
      if (Y < -32768) Y = -32768;
      else if (Y > 32767) Y = 32767;
   }
   if (!(uFlags & SWP_NOSIZE))
   {
      if (cx < 0) cx = 0;
      else if (cx > 32767) cx = 32767;
      if (cy < 0) cy = 0;
      else if (cy > 32767) cy = 32767;
   }

   UserRefObjectCo(Window, &Ref);
   ret = co_WinPosSetWindowPos(Window, hWndInsertAfter, X, Y, cx, cy, uFlags);
   UserDerefObjectCo(Window);

Exit:
   TRACE("Leave NtUserSetWindowPos, ret=%i\n", ret);
   UserLeave();
   return ret;
}

/*
 * @implemented
 */
INT APIENTRY
NtUserSetWindowRgn(
   HWND hWnd,
   HRGN hRgn,
   BOOL bRedraw)
{
   HRGN hrgnCopy = NULL;
   PWND Window;
   INT flags = (SWP_NOCLIENTSIZE|SWP_NOCLIENTMOVE|SWP_NOACTIVATE|SWP_FRAMECHANGED|SWP_NOSIZE|SWP_NOMOVE);
   INT Ret = 0;

   TRACE("Enter NtUserSetWindowRgn\n");
   UserEnterExclusive();

   if (!(Window = UserGetWindowObject(hWnd)) ||
        UserIsDesktopWindow(Window) || UserIsMessageWindow(Window))
   {
      goto Exit; // Return 0
   }

   if (hRgn) // The region will be deleted in user32.
   {
      if (GreIsHandleValid(hRgn))
      {
         hrgnCopy = NtGdiCreateRectRgn(0, 0, 0, 0);
      /* The coordinates of a window's window region are relative to the
         upper-left corner of the window, not the client area of the window. */
         NtGdiCombineRgn( hrgnCopy, hRgn, 0, RGN_COPY);
      }
      else
         goto Exit; // Return 0
   }

   //// HACK 1 : Work around the lack of supporting DeferWindowPos.
   if (hrgnCopy)
   {
       Window->hrgnNewFrame = hrgnCopy; // Should be PSMWP->acvr->hrgnClip
   }
   else
   {
       Window->hrgnNewFrame = HRGN_WINDOW;
   }
   //// HACK 2
   Ret = (INT)co_WinPosSetWindowPos(Window, HWND_TOP, 0, 0, 0, 0, bRedraw ? flags : (flags | SWP_NOREDRAW));

Exit:
   TRACE("Leave NtUserSetWindowRgn, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * @implemented
 */
DWORD APIENTRY
NtUserSetInternalWindowPos(
   HWND    hwnd,
   UINT    showCmd,
   LPRECT  lprect,
   LPPOINT lppt)
{
   WINDOWPLACEMENT wndpl;
   UINT flags;
   PWND Wnd;
   RECT rect;
   POINT pt = {0};
   BOOL Ret = FALSE;
   USER_REFERENCE_ENTRY Ref;
   NTSTATUS Status;

   TRACE("Enter NtUserSetWindowPlacement\n");
   UserEnterExclusive();

   if (!(Wnd = UserGetWindowObject(hwnd)) || // FIXME:
        UserIsDesktopWindow(Wnd) || UserIsMessageWindow(Wnd))
   {
      goto Exit; // Return FALSE
   }

   if (lppt)
   {
      Status = SafeProbeForRead(lppt, sizeof(POINT), 1);
   }
   else
   {
      Status = STATUS_SUCCESS;
   }
   if (NT_SUCCESS(Status) && lprect)
   {
      Status = SafeProbeForRead(lprect, sizeof(RECT), 1);
   }
   if (!NT_SUCCESS(Status))
   {
      SetLastNtError(Status);
      goto Exit;
   }
   else
   {
      if (lppt)
      {
         RtlCopyMemory(&pt, lppt, sizeof(POINT));
      }
      if (lprect)
      {
         RtlCopyMemory(&rect, lprect, sizeof(RECT));
      }
   }

   wndpl.length  = sizeof(wndpl);
   wndpl.showCmd = showCmd;
   wndpl.flags = flags = 0;

   if ( lppt )
   {
      flags |= PLACE_MIN;
      wndpl.flags |= WPF_SETMINPOSITION;
      wndpl.ptMinPosition = pt;
   }
   if ( lprect )
   {
      flags |= PLACE_RECT;
      wndpl.rcNormalPosition = rect;
   }

   UserRefObjectCo(Wnd, &Ref);
   IntSetWindowPlacement(Wnd, &wndpl, flags);
   UserDerefObjectCo(Wnd);
   Ret = TRUE;

Exit:
   TRACE("Leave NtUserSetWindowPlacement, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}


/*
 * @implemented
 */
BOOL APIENTRY
NtUserSetWindowPlacement(HWND hWnd,
                         WINDOWPLACEMENT *lpwndpl)
{
   PWND Wnd;
   WINDOWPLACEMENT Safepl;
   UINT Flags;
   BOOL Ret = FALSE;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserSetWindowPlacement\n");
   UserEnterExclusive();

    {
        NTSTATUS __SehStatus = SafeProbeForRead(lpwndpl, sizeof(*lpwndpl), 1);
        if (!NT_SUCCESS(__SehStatus))
        {
            SetLastNtError(__SehStatus);
            goto Exit; // Return FALSE
        }
    }
    Safepl = *lpwndpl;

    /* Backwards-compatibility: Win 3.x doesn't check the length */
    if (LOWORD(gptiCurrent->dwExpWinVer) < WINVER_WINNT4)
        Safepl.length = sizeof(Safepl);

    if (Safepl.length != sizeof(Safepl))
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        goto Exit;
    }

    Flags = PLACE_MAX | PLACE_RECT;
    if (Safepl.flags & WPF_SETMINPOSITION)
        Flags |= PLACE_MIN;

    Wnd = UserGetWindowObject(hWnd);
    if (!Wnd)
        goto Exit; // Return FALSE

    UserRefObjectCo(Wnd, &Ref);
    if (!UserIsDesktopWindow(Wnd) && !UserIsMessageWindow(Wnd))
        Ret = IntSetWindowPlacement(Wnd, &Safepl, Flags);
    UserDerefObjectCo(Wnd);

Exit:
   TRACE("Leave NtUserSetWindowPlacement, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * @implemented
 */
BOOL APIENTRY
NtUserShowWindowAsync(HWND hWnd, LONG nCmdShow)
{
   PWND Window;
   LRESULT Result;
   BOOL ret = FALSE;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserShowWindowAsync\n");
   UserEnterExclusive();

   if (!(Window = UserGetWindowObject(hWnd)) ||
        UserIsDesktopWindow(Window) || UserIsMessageWindow(Window))
   {
      goto Exit; // Return FALSE
   }

   if ( nCmdShow > SW_MAX )
   {
      EngSetLastError(ERROR_INVALID_PARAMETER);
      goto Exit; // Return FALSE
   }

   UserRefObjectCo(Window, &Ref);
   Result = co_IntSendMessageNoWait( hWnd, WM_ASYNC_SHOWWINDOW, nCmdShow, 0 );
   UserDerefObjectCo(Window);
   if (Result != -1 && Result != 0) ret = TRUE;

Exit:
   TRACE("Leave NtUserShowWindowAsync, ret=%i\n", ret);
   UserLeave();
   return ret;
}

/*
 * @implemented
 */
BOOL APIENTRY
NtUserShowWindow(HWND hWnd, LONG nCmdShow)
{
   PWND Window;
   BOOL ret = FALSE;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserShowWindow hWnd %p SW_ %d\n",hWnd, nCmdShow);
   UserEnterExclusive();

   if (!(Window = UserGetWindowObject(hWnd)) ||
        UserIsDesktopWindow(Window) || UserIsMessageWindow(Window))
   {
      goto Exit; // Return FALSE
   }

   if ( nCmdShow > SW_MAX || Window->state2 & WNDS2_INDESTROY)
   {
      EngSetLastError(ERROR_INVALID_PARAMETER);
      goto Exit; // Return FALSE
   }

   UserRefObjectCo(Window, &Ref);
   ret = co_WinPosShowWindow(Window, nCmdShow);
   UserDerefObjectCo(Window);

Exit:
   TRACE("Leave NtUserShowWindow, ret=%i\n", ret);
   UserLeave();
   return ret;
}


/*
 *    @implemented
 */
HWND APIENTRY
NtUserWindowFromPoint(LONG X, LONG Y)
{
   POINT pt;
   HWND Ret = NULL;
   PWND DesktopWindow, Window;
   USHORT hittest;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserWindowFromPoint\n");
   UserEnterExclusive();

   if ((DesktopWindow = UserGetWindowObject(IntGetDesktopWindow())))
   {
      //PTHREADINFO pti;

      pt.x = X;
      pt.y = Y;

      // Hmm... Threads live on desktops thus we have a reference on the desktop and indirectly the desktop window.
      // It is possible this referencing is useless, though it should not hurt...
      UserRefObjectCo(DesktopWindow, &Ref);

      //pti = PsGetCurrentThreadWin32Thread();
      Window = co_WinPosWindowFromPoint(DesktopWindow, &pt, &hittest, FALSE);
      if (Window)
      {
         Ret = UserHMGetHandle(Window);
      }

      UserDerefObjectCo(DesktopWindow);
   }

   TRACE("Leave NtUserWindowFromPoint, ret=%p\n", Ret);
   UserLeave();
   return Ret;
}

/* Windows 10 (1903?)
BOOL APIENTRY
NtUserIsWindowArranged(HWND hWnd)
{
    PWND pwnd = UserGetWindowObject(hWnd);
    return pwnd && IntIsWindowSnapped(pwnd);
}
*/

UINT FASTCALL
IntGetWindowSnapEdge(PWND Wnd)
{
    if (Wnd->ExStyle2 & WS_EX2_VERTICALLYMAXIMIZEDLEFT) return HTLEFT;
    if (Wnd->ExStyle2 & WS_EX2_VERTICALLYMAXIMIZEDRIGHT) return HTRIGHT;
    return HTNOWHERE;
}

VOID FASTCALL
co_IntCalculateSnapPosition(PWND Wnd, UINT Edge, OUT RECT *Pos)
{
    POINT maxs, mint, maxt;
    UINT width, height;
    UserSystemParametersInfo(SPI_GETWORKAREA, 0, Pos, 0); /* FIXME: MultiMon of PWND */

    co_WinPosGetMinMaxInfo(Wnd, &maxs, NULL, &mint, &maxt);
    width = Pos->right - Pos->left;
    width = min(min(max(width / 2, mint.x), maxt.x), width);
    height = Pos->bottom - Pos->top;
    height = min(max(height, mint.y), maxt.y);

    switch (Edge)
    {
    case HTTOP: /* Maximized (Calculate RECT snap preview for SC_MOVE) */
        height = min(Pos->bottom - Pos->top, maxs.y);
        break; 
    case HTLEFT:
        Pos->right = width;
        break;
    case HTRIGHT:
        Pos->left = Pos->right - width;
        break;
    default:
        ERR("Unexpected snap edge %#x\n", Edge);
    }
    Pos->bottom = Pos->top + height;
}

VOID FASTCALL
co_IntSnapWindow(PWND Wnd, UINT Edge)
{
    RECT newPos;
    BOOLEAN wasSnapped = IntIsWindowSnapped(Wnd);
    UINT normal = !(Wnd->style & (WS_MAXIMIZE | WS_MINIMIZE));
    USER_REFERENCE_ENTRY ref;
    BOOLEAN hasRef = FALSE;

    if (Edge == HTTOP)
    {
        co_IntSendMessage(UserHMGetHandle(Wnd), WM_SYSCOMMAND, SC_MAXIMIZE, 0);
        return;
    }
    else if (Edge != HTNOWHERE)
    {
        UserRefObjectCo(Wnd, &ref);
        hasRef = TRUE;
        co_IntCalculateSnapPosition(Wnd, Edge, &newPos);
        IntSetSnapInfo(Wnd, Edge, (wasSnapped || !normal) ? NULL : &Wnd->rcWindow);
    }
    else if (wasSnapped)
    {
        if (!normal)
        {
            IntSetSnapEdge(Wnd, HTNOWHERE);
            return;
        }
        newPos = Wnd->InternalPos.NormalRect; /* Copy RECT now before it is lost */
        IntSetSnapInfo(Wnd, HTNOWHERE, NULL);
    }
    else
    {
        return; /* Already unsnapped, do nothing */
    }

    TRACE("WindowSnap: %d->%d\n", IntGetWindowSnapEdge(Wnd), Edge);
    co_WinPosSetWindowPos(Wnd, HWND_TOP,
                          newPos.left,
                          newPos.top,
                          newPos.right - newPos.left,
                          newPos.bottom - newPos.top,
                          0);
    if (hasRef)
        UserDerefObjectCo(Wnd);
}

VOID FASTCALL
IntSetSnapEdge(PWND Wnd, UINT Edge)
{
    UINT styleMask = WS_EX2_VERTICALLYMAXIMIZEDLEFT | WS_EX2_VERTICALLYMAXIMIZEDRIGHT;
    UINT style = 0;
    switch (Edge)
    {
    case HTNOWHERE:
        style = 0;
        break;
    case HTTOP: /* Maximize throws away the snap */
        style = 0;
        break;
    case HTLEFT:
        style = WS_EX2_VERTICALLYMAXIMIZEDLEFT;
        break;
    case HTRIGHT:
        style = WS_EX2_VERTICALLYMAXIMIZEDRIGHT;
        break;
    default:
        ERR("Unexpected snap edge %#x\n", Edge);
    }
    Wnd->ExStyle2 = (Wnd->ExStyle2 & ~styleMask) | style;
}

VOID FASTCALL
IntSetSnapInfo(PWND Wnd, UINT Edge, IN const RECT *Pos OPTIONAL)
{
    RECT r;
    IntSetSnapEdge(Wnd, Edge);
    if (Edge == HTNOWHERE)
    {
        RECTL_vSetEmptyRect(&r);
        Pos = (Wnd->style & WS_MINIMIZE) ? NULL : &r;
    }
    if (Pos)
    {
        Wnd->InternalPos.NormalRect = *Pos;
    }
}

/* EOF */
