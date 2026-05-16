/*
 * COPYRIGHT:   See COPYING in the top level directory
 * PROJECT:     ReactOS kernel
 * PURPOSE:     Windows
 * PROGRAMER:   Casper S. Hornstrup (chorns@users.sourceforge.net)
 *              Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
 */

#include <win32k.h>
#include <safe_user_io.h>
#include <immdev.h>
DBG_DEFAULT_CHANNEL(UserWinpos);

/* GLOBALS *******************************************************************/

#define MINMAX_NOSWP  (0x00010000)

#define SWP_EX_NOCOPY 0x0001
#define SWP_EX_PAINTSELF 0x0002

#define  SWP_AGG_NOGEOMETRYCHANGE \
    (SWP_NOSIZE | SWP_NOCLIENTSIZE | SWP_NOZORDER)
#define  SWP_AGG_NOPOSCHANGE \
    (SWP_NOSIZE | SWP_NOMOVE | SWP_NOCLIENTSIZE | SWP_NOCLIENTMOVE | SWP_NOZORDER)
#define  SWP_AGG_STATUSFLAGS \
    (SWP_AGG_NOPOSCHANGE | SWP_FRAMECHANGED | SWP_HIDEWINDOW | SWP_SHOWWINDOW)
#define SWP_AGG_NOCLIENTCHANGE \
    (SWP_NOCLIENTSIZE | SWP_NOCLIENTMOVE)

#define EMPTYPOINT(pt) ((pt).x == -1 && (pt).y == -1)
#define PLACE_MIN               0x0001
#define PLACE_MAX               0x0002
#define PLACE_RECT              0x0004

/* FUNCTIONS *****************************************************************/

#if DBG
/***********************************************************************
 *           dump_winpos_flags
 */
static void dump_winpos_flags(UINT flags)
{
    static const DWORD dumped_flags = (SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOREDRAW |
                                       SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW |
                                       SWP_HIDEWINDOW | SWP_NOCOPYBITS | SWP_NOOWNERZORDER |
                                       SWP_NOSENDCHANGING | SWP_DEFERERASE | SWP_ASYNCWINDOWPOS |
                                       SWP_NOCLIENTSIZE | SWP_NOCLIENTMOVE | SWP_STATECHANGED);
    TRACE("flags:");
    if(flags & SWP_NOSIZE) TRACE(" SWP_NOSIZE");
    if(flags & SWP_NOMOVE) TRACE(" SWP_NOMOVE");
    if(flags & SWP_NOZORDER) TRACE(" SWP_NOZORDER");
    if(flags & SWP_NOREDRAW) TRACE(" SWP_NOREDRAW");
    if(flags & SWP_NOACTIVATE) TRACE(" SWP_NOACTIVATE");
    if(flags & SWP_FRAMECHANGED) TRACE(" SWP_FRAMECHANGED");
    if(flags & SWP_SHOWWINDOW) TRACE(" SWP_SHOWWINDOW");
    if(flags & SWP_HIDEWINDOW) TRACE(" SWP_HIDEWINDOW");
    if(flags & SWP_NOCOPYBITS) TRACE(" SWP_NOCOPYBITS");
    if(flags & SWP_NOOWNERZORDER) TRACE(" SWP_NOOWNERZORDER");
    if(flags & SWP_NOSENDCHANGING) TRACE(" SWP_NOSENDCHANGING");
    if(flags & SWP_DEFERERASE) TRACE(" SWP_DEFERERASE");
    if(flags & SWP_ASYNCWINDOWPOS) TRACE(" SWP_ASYNCWINDOWPOS");
    if(flags & SWP_NOCLIENTSIZE) TRACE(" SWP_NOCLIENTSIZE");
    if(flags & SWP_NOCLIENTMOVE) TRACE(" SWP_NOCLIENTMOVE");
    if(flags & SWP_STATECHANGED) TRACE(" SWP_STATECHANGED");

    if(flags & ~dumped_flags) TRACE(" %08x", flags & ~dumped_flags);
    TRACE("\n");
}
#endif

BOOL FASTCALL
IntGetClientOrigin(PWND Window OPTIONAL, LPPOINT Point)
{
   Window = Window ? Window : UserGetDesktopWindow();
   if (Window == NULL)
   {
      Point->x = Point->y = 0;
      return FALSE;
   }
   Point->x = Window->rcClient.left;
   Point->y = Window->rcClient.top;

   return TRUE;
}

/*!
 * Internal function.
 * Returns client window rectangle relative to the upper-left corner of client area.
 *
 * \note Does not check the validity of the parameters
*/
VOID FASTCALL
IntGetClientRect(PWND Wnd, RECTL *Rect)
{
   ASSERT( Wnd );
   ASSERT( Rect );
   if (Wnd->style & WS_MINIMIZED)
   {
      Rect->left = Rect->top = 0;
      Rect->right = UserGetSystemMetrics(SM_CXMINIMIZED);
      Rect->bottom = UserGetSystemMetrics(SM_CYMINIMIZED);
      return;
   }
   if (!UserIsDesktopWindow(Wnd))
   {
      *Rect = Wnd->rcClient;
      RECTL_vOffsetRect(Rect, -Wnd->rcClient.left, -Wnd->rcClient.top);
   }
   else
   {
      Rect->left = Rect->top = 0;
      Rect->right  = Wnd->rcClient.right;
      Rect->bottom = Wnd->rcClient.bottom;
   /* Do this until Init bug is fixed. This sets 640x480, see InitMetrics.
      Rect->right  = UserGetSystemMetrics(SM_CXSCREEN);
      Rect->bottom = UserGetSystemMetrics(SM_CYSCREEN);
   */
   }
}

BOOL FASTCALL
IntGetWindowRect(PWND Wnd, RECTL *Rect)
{
   ASSERT( Wnd );
   ASSERT( Rect );
   if (!Wnd) return FALSE;
   if (!UserIsDesktopWindow(Wnd))
   {
       *Rect = Wnd->rcWindow;
   }
   else
   {
       Rect->left = Rect->top = 0;
       Rect->right = Wnd->rcWindow.right;
       Rect->bottom = Wnd->rcWindow.bottom;
/* Do this until Init bug is fixed. This sets 640x480, see InitMetrics.
       Rect->right = GetSystemMetrics(SM_CXSCREEN);
       Rect->bottom = GetSystemMetrics(SM_CYSCREEN);
*/   }
   return TRUE;
}


INT FASTCALL
IntMapWindowPoints(PWND FromWnd, PWND ToWnd, LPPOINT lpPoints, UINT cPoints)
{
    BOOL mirror_from, mirror_to;
    POINT Delta;
    UINT i;
    int Change = 1;

    /* Note: Desktop Top and Left is always 0! */
    Delta.x = Delta.y = 0;
    mirror_from = mirror_to = FALSE;

    if (FromWnd && !UserIsDesktopWindow(FromWnd))
    {
       if (FromWnd->ExStyle & WS_EX_LAYOUTRTL)
       {
          mirror_from = TRUE;
          Change = -Change;
          Delta.x = -FromWnd->rcClient.right;
       }
       else
          Delta.x = FromWnd->rcClient.left;
       Delta.y = FromWnd->rcClient.top;
    }

    if (ToWnd && !UserIsDesktopWindow(ToWnd))
    {
       if (ToWnd->ExStyle & WS_EX_LAYOUTRTL)
       {
          mirror_to = TRUE;
          Change = -Change;
          Delta.x += Change * ToWnd->rcClient.right;
       }
       else
          Delta.x -= Change * ToWnd->rcClient.left;
       Delta.y -= ToWnd->rcClient.top;
    }

    for (i = 0; i != cPoints; i++)
    {
        lpPoints[i].x += Delta.x;
        lpPoints[i].x *= Change;
        lpPoints[i].y += Delta.y;
    }

    if ((mirror_from || mirror_to) && cPoints == 2)  /* special case for rectangle */
    {
       int tmp = min(lpPoints[0].x, lpPoints[1].x);
       lpPoints[1].x = max(lpPoints[0].x, lpPoints[1].x);
       lpPoints[0].x = tmp;
    }

    return MAKELONG(LOWORD(Delta.x), LOWORD(Delta.y));
}

BOOL FASTCALL
IntClientToScreen(PWND Wnd, LPPOINT lpPoint)
{
   if (Wnd && Wnd->fnid != FNID_DESKTOP )
   {
      if (Wnd->ExStyle & WS_EX_LAYOUTRTL)
         lpPoint->x = Wnd->rcClient.right - lpPoint->x;
      else
         lpPoint->x += Wnd->rcClient.left;
      lpPoint->y += Wnd->rcClient.top;
   }
   return TRUE;
}

BOOL FASTCALL
IntScreenToClient(PWND Wnd, LPPOINT lpPoint)
{
    if (Wnd && Wnd->fnid != FNID_DESKTOP )
    {
       if (Wnd->ExStyle & WS_EX_LAYOUTRTL)
          lpPoint->x = Wnd->rcClient.right - lpPoint->x;
       else
          lpPoint->x -= Wnd->rcClient.left;
       lpPoint->y -= Wnd->rcClient.top;
    }
    return TRUE;
}

BOOL FASTCALL IsChildVisible(PWND pWnd)
{
    do
    {
        if ( (pWnd->style & (WS_POPUP|WS_CHILD)) != WS_CHILD ||
            !(pWnd = pWnd->spwndParent) )
           return TRUE;
    }
    while (pWnd->style & WS_VISIBLE);
    return FALSE;
}

PWND FASTCALL IntGetLastTopMostWindow(VOID)
{
    PWND pWnd;
    PDESKTOP rpdesk = gptiCurrent->rpdesk;

    if ( rpdesk &&
        (pWnd = rpdesk->pDeskInfo->spwnd->spwndChild) &&
         pWnd->ExStyle & WS_EX_TOPMOST)
    {
        for (;;)
        {
           if (!pWnd->spwndNext) break;
           if (!(pWnd->spwndNext->ExStyle & WS_EX_TOPMOST)) break;
           pWnd = pWnd->spwndNext;
        }
        return pWnd;
    }
    return NULL;
}

VOID
SelectWindowRgn(PWND Window, HRGN hRgnClip)
{
    if (Window->hrgnClip)
    {
        /* Delete no longer needed region handle */
        IntGdiSetRegionOwner(Window->hrgnClip, GDI_OBJ_HMGR_POWNED);
        GreDeleteObject(Window->hrgnClip);
        Window->hrgnClip = NULL;       
    }

    if (hRgnClip > HRGN_WINDOW)
    {
        /*if (!UserIsDesktopWindow(Window))
        {
            NtGdiOffsetRgn(hRgnClip, Window->rcWindow.left, Window->rcWindow.top);
        }*/
        /* Set public ownership */
        IntGdiSetRegionOwner(hRgnClip, GDI_OBJ_HMGR_PUBLIC);

        Window->hrgnClip = hRgnClip;
    }
}

//
// This helps with CORE-6129 forcing modal dialog active when another app is minimized or closed.
//
BOOL FASTCALL ActivateOtherWindowMin(PWND Wnd)
{
    BOOL ActivePrev, FindTopWnd;
    PWND pWndTopMost, pWndChild, pWndSetActive, pWndTemp, pWndDesk;
    USER_REFERENCE_ENTRY Ref;
    PTHREADINFO pti = gptiCurrent;

    //ERR("AOWM 1 %p\n", UserHMGetHandle(Wnd));
    ActivePrev = (pti->MessageQueue->spwndActivePrev != NULL);
    FindTopWnd = TRUE;

    if ((pWndTopMost = IntGetLastTopMostWindow()))
       pWndChild = pWndTopMost->spwndNext;
    else
       pWndChild = Wnd->spwndParent->spwndChild;

    for (;;)
    {
       if ( ActivePrev )
          pWndSetActive = pti->MessageQueue->spwndActivePrev;
       else
          pWndSetActive = pWndChild;

       pWndTemp = NULL;

       while(pWndSetActive)
       {
          if ( VerifyWnd(pWndSetActive) &&
              !(pWndSetActive->ExStyle & WS_EX_NOACTIVATE) &&
               (pWndSetActive->style & (WS_VISIBLE|WS_DISABLED)) == WS_VISIBLE &&
               (!(pWndSetActive->style & WS_ICONIC) /* FIXME MinMax pos? */ ) )
          {
             if (!(pWndSetActive->ExStyle & WS_EX_TOOLWINDOW) )
             {
                UserRefObjectCo(pWndSetActive, &Ref);
                //ERR("ActivateOtherWindowMin Set FG 1\n");
                co_IntSetForegroundWindow(pWndSetActive);
                UserDerefObjectCo(pWndSetActive);
                //ERR("AOWM 2 Exit Good %p\n", UserHMGetHandle(pWndSetActive));
                return TRUE;
             }
             if (!pWndTemp ) pWndTemp = pWndSetActive;
          }
          if ( ActivePrev )
          {
             ActivePrev = FALSE;
             pWndSetActive = pWndChild;
          }
          else
             pWndSetActive = pWndSetActive->spwndNext;
       }

       if ( !FindTopWnd ) break;
       FindTopWnd = FALSE;

       if ( pWndChild )
       {
          pWndChild = pWndChild->spwndParent->spwndChild;
          continue;
       }

       if (!(pWndDesk = IntGetThreadDesktopWindow(pti)))
       {
          pWndChild = NULL;
          continue;
       }
       pWndChild = pWndDesk->spwndChild;
    }

    if ((pWndSetActive = pWndTemp))
    {
       UserRefObjectCo(pWndSetActive, &Ref);
       //ERR("ActivateOtherWindowMin Set FG 2\n");
       co_IntSetForegroundWindow(pWndSetActive);
       UserDerefObjectCo(pWndSetActive);
       //ERR("AOWM 3 Exit Good %p\n", UserHMGetHandle(pWndSetActive));
       return TRUE;
    }
    //ERR("AOWM 4 Bad\n");
    return FALSE;
}

/*******************************************************************
 *         can_activate_window
 *
 * Check if we can activate the specified window.
 */
static
BOOL FASTCALL can_activate_window( PWND Wnd OPTIONAL)
{
    LONG style;

    if (!Wnd) return FALSE;

    style = Wnd->style;
    if (!(style & WS_VISIBLE)) return FALSE;
    if (style & WS_MINIMIZE) return FALSE;
    if ((style & (WS_POPUP|WS_CHILD)) == WS_CHILD) return FALSE;
    if (Wnd->ExStyle & WS_EX_NOACTIVATE) return FALSE;
    return TRUE;
    /* FIXME: This window could be disable because the child that closed
              was a popup. */
    //return !(style & WS_DISABLED);
}


/*******************************************************************
 *         WinPosActivateOtherWindow
 *
 *  Activates window other than pWnd.
 */
VOID FASTCALL
co_WinPosActivateOtherWindow(PWND Wnd)
{
   PWND WndTo = NULL;
   USER_REFERENCE_ENTRY Ref;

   ASSERT_REFS_CO(Wnd);

   if (IntIsDesktopWindow(Wnd))
   {
      //ERR("WinPosActivateOtherWindow Set Focus Msg Q No window!\n");
      IntSetFocusMessageQueue(NULL);
      return;
   }

   /* If this is popup window, try to activate the owner first. */
   if ((Wnd->style & WS_POPUP) && (WndTo = Wnd->spwndOwner))
   {
      TRACE("WPAOW Popup with Owner\n");
      WndTo = UserGetAncestor( WndTo, GA_ROOT );
      if (can_activate_window(WndTo)) goto done;
   }

   /* Pick a next top-level window. */
   /* FIXME: Search for non-tooltip windows first. */
   WndTo = Wnd;
   for (;;)
   {
      if (!(WndTo = WndTo->spwndNext))  break;
      if (can_activate_window( WndTo )) goto done;
   }

   /*
      Fixes wine win.c:test_SetParent last ShowWindow test after popup dies.
      Check for previous active window to bring to top.
   */
   if (Wnd)
   {
      WndTo = Wnd->head.pti->MessageQueue->spwndActivePrev;
      if (can_activate_window( WndTo )) goto done;
   }

   // Find any window to bring to top. Works Okay for wine since it does not see X11 windows.
   WndTo = UserGetDesktopWindow();
   if ((WndTo == NULL) || (WndTo->spwndChild == NULL))
   {
      //ERR("WinPosActivateOtherWindow No window!\n");
      return;
   }
   WndTo = WndTo->spwndChild;
   for (;;)
   {
      if (WndTo == Wnd)
      {
         WndTo = NULL;
         break;
      }
      if (can_activate_window( WndTo )) goto done;
      if (!(WndTo = WndTo->spwndNext))  break;
   }

done:
   if (WndTo) UserRefObjectCo(WndTo, &Ref);

   if (gpqForeground && (!gpqForeground->spwndActive || Wnd == gpqForeground->spwndActive))
   {
      /* ReactOS can pass WndTo = NULL to co_IntSetForegroundWindow and returns FALSE. */
      //ERR("WinPosActivateOtherWindow Set FG 0x%p hWnd %p\n", WndTo, WndTo ? UserHMGetHandle(WndTo) : NULL);
      if (co_IntSetForegroundWindow(WndTo))
      {
         if (WndTo) UserDerefObjectCo(WndTo);
         return;
      }
   }
   //ERR("WinPosActivateOtherWindow Set Active  0x%p\n",WndTo);
   if (!UserSetActiveWindow(WndTo))  /* Ok for WndTo to be NULL here */
   {
      //ERR("WPAOW SA 1\n");
      UserSetActiveWindow(NULL);
   }
   if (WndTo) UserDerefObjectCo(WndTo);
}

VOID FASTCALL
WinPosInitInternalPos(PWND Wnd, RECTL *RestoreRect)
{
   POINT Size;
   RECTL Rect = *RestoreRect;

   if (Wnd->spwndParent && !UserIsDesktopWindow(Wnd->spwndParent))
   {
      RECTL_vOffsetRect(&Rect,
                        -Wnd->spwndParent->rcClient.left,
                        -Wnd->spwndParent->rcClient.top);
   }

   Size.x = Rect.left;
   Size.y = Rect.top;

   if (!Wnd->InternalPosInitialized)
   {
      // FIXME: Use check point Atom..
      Wnd->InternalPos.flags = 0;
      Wnd->InternalPos.MaxPos.x  = Wnd->InternalPos.MaxPos.y  = -1;
      Wnd->InternalPos.IconPos.x = Wnd->InternalPos.IconPos.y = -1;
      Wnd->InternalPos.NormalRect = Rect;
      Wnd->InternalPosInitialized = TRUE;
   }

   if (Wnd->style & WS_MINIMIZE)
   {
      Wnd->InternalPos.IconPos = Size;
      Wnd->InternalPos.flags |= WPF_MININIT;
   }
   else if (Wnd->style & WS_MAXIMIZE)
   {
      Wnd->InternalPos.flags |= WPF_MAXINIT;

      if ( Wnd->spwndParent == Wnd->head.rpdesk->pDeskInfo->spwnd )
      {
         if (Wnd->state & WNDS_MAXIMIZESTOMONITOR)
         {
            Wnd->InternalPos.flags &= ~WPF_MAXINIT;
            Wnd->InternalPos.MaxPos.x = Wnd->InternalPos.MaxPos.y = -1;
         }
         else
         {
            RECTL WorkArea;
            PMONITOR pmonitor = UserMonitorFromRect(&Rect, MONITOR_DEFAULTTOPRIMARY );
            // FIXME: support DPI aware, rcWorkDPI/Real etc..
            WorkArea = pmonitor->rcMonitor;

            if (Wnd->style & WS_MAXIMIZEBOX)
            {  // Support (Wnd->state & WNDS_HASCAPTION) || pmonitor->cFullScreen too.
               if ((Wnd->style & WS_CAPTION) == WS_CAPTION || !(Wnd->style & (WS_CHILD | WS_POPUP)))
               {
                  WorkArea = pmonitor->rcWork;
                  //ERR("rcWork\n");
               }
            }

            Wnd->InternalPos.MaxPos.x = Rect.left - WorkArea.left;
            Wnd->InternalPos.MaxPos.y = Rect.top  - WorkArea.top;

            /*ERR("WinPosIP 2 X %d = R.l %d - W.l %d | Y %d = R.t %d - W.t %d\n",
                                         Wnd->InternalPos.MaxPos.x,
                                         Rect.left, WorkArea.left,
                                         Wnd->InternalPos.MaxPos.y,
                                         Rect.top, WorkArea.top);*/
         }
      }
      else
         Wnd->InternalPos.MaxPos = Size;
   }
   else
   {
      /* Lie about the snap; Windows does this so applications don't save their
       * position as a snap but rather the unsnapped "real" position. */
      if (!IntIsWindowSnapped(Wnd) ||
          RECTL_bIsEmptyRect(&Wnd->InternalPos.NormalRect))
      {
         Wnd->InternalPos.NormalRect = Rect;
      }
   }
}

BOOL
FASTCALL
IntGetWindowPlacement(PWND Wnd, WINDOWPLACEMENT *lpwndpl)
{
   if (!Wnd) return FALSE;

   if(lpwndpl->length != sizeof(WINDOWPLACEMENT))
   {
      ERR("length mismatch: %u\n", lpwndpl->length);
      return FALSE;
   }

   lpwndpl->flags = 0;

   WinPosInitInternalPos(Wnd, &Wnd->rcWindow);

   lpwndpl->showCmd = SW_HIDE;

   if ( Wnd->style & WS_MINIMIZE )
      lpwndpl->showCmd = SW_SHOWMINIMIZED;
   else
      lpwndpl->showCmd = ( Wnd->style & WS_MAXIMIZE ) ? SW_SHOWMAXIMIZED : SW_SHOWNORMAL ;

   lpwndpl->rcNormalPosition = Wnd->InternalPos.NormalRect;

   if (Wnd->InternalPos.flags & WPF_MININIT) // Return if it was set!
   {
      lpwndpl->ptMinPosition.x = Wnd->InternalPos.IconPos.x;
      lpwndpl->ptMinPosition.y = Wnd->InternalPos.IconPos.y;
   }
   else
      lpwndpl->ptMinPosition.x = lpwndpl->ptMinPosition.y = -1;

   if ( Wnd->InternalPos.flags & WPF_MAXINIT && // Return if set and not maximized to monitor!
        !(Wnd->state & WNDS_MAXIMIZESTOMONITOR))
   {
      lpwndpl->ptMaxPosition.x = Wnd->InternalPos.MaxPos.x;
      lpwndpl->ptMaxPosition.y = Wnd->InternalPos.MaxPos.y;
   }
   else
      lpwndpl->ptMaxPosition.x = lpwndpl->ptMaxPosition.y = -1;

   if ( Wnd->spwndParent == Wnd->head.rpdesk->pDeskInfo->spwnd &&
       !(Wnd->ExStyle & WS_EX_TOOLWINDOW))
   {
      PMONITOR pmonitor = UserMonitorFromRect(&lpwndpl->rcNormalPosition, MONITOR_DEFAULTTOPRIMARY );

      // FIXME: support DPI aware, rcWorkDPI/Real etc..
      if (Wnd->InternalPos.flags & WPF_MININIT)
      {
         lpwndpl->ptMinPosition.x -= (pmonitor->rcWork.left - pmonitor->rcMonitor.left);
         lpwndpl->ptMinPosition.y -= (pmonitor->rcWork.top - pmonitor->rcMonitor.top);
      }
      RECTL_vOffsetRect(&lpwndpl->rcNormalPosition,
                         pmonitor->rcMonitor.left - pmonitor->rcWork.left,
                         pmonitor->rcMonitor.top  - pmonitor->rcWork.top);
   }

   if ( Wnd->InternalPos.flags & WPF_RESTORETOMAXIMIZED || Wnd->style & WS_MAXIMIZE )
      lpwndpl->flags |= WPF_RESTORETOMAXIMIZED;

   if ( ((Wnd->style & (WS_CHILD|WS_POPUP)) == WS_CHILD) && Wnd->InternalPos.flags & WPF_SETMINPOSITION)
      lpwndpl->flags |= WPF_SETMINPOSITION;

   return TRUE;
}

/* make sure the specified rect is visible on screen */
static void make_rect_onscreen( RECT *rect )
{
    PMONITOR pmonitor = UserMonitorFromRect( rect, MONITOR_DEFAULTTONEAREST ); // Wine uses this.

    //  FIXME: support DPI aware, rcWorkDPI/Real etc..
    if (!pmonitor) return;
    /* FIXME: map coordinates from rcWork to rcMonitor */
    if (rect->right <= pmonitor->rcWork.left)
    {
        rect->right += pmonitor->rcWork.left - rect->left;
        rect->left = pmonitor->rcWork.left;
    }
    else if (rect->left >= pmonitor->rcWork.right)
    {
        rect->left += pmonitor->rcWork.right - rect->right;
        rect->right = pmonitor->rcWork.right;
    }
    if (rect->bottom <= pmonitor->rcWork.top)
    {
        rect->bottom += pmonitor->rcWork.top - rect->top;
        rect->top = pmonitor->rcWork.top;
    }
    else if (rect->top >= pmonitor->rcWork.bottom)
    {
        rect->top += pmonitor->rcWork.bottom - rect->bottom;
        rect->bottom = pmonitor->rcWork.bottom;
    }
}

/* make sure the specified point is visible on screen */
static void make_point_onscreen( POINT *pt )
{
    RECT rect;

    RECTL_vSetRect( &rect, pt->x, pt->y, pt->x + 1, pt->y + 1 );
    make_rect_onscreen( &rect );
    pt->x = rect.left;
    pt->y = rect.top;
}

BOOL FASTCALL
IntSetWindowPlacement(PWND Wnd, WINDOWPLACEMENT *wpl, UINT Flags)
{
   BOOL sAsync;
   UINT SWP_Flags;

   if ( Flags & PLACE_MIN) make_point_onscreen( &wpl->ptMinPosition );
   if ( Flags & PLACE_MAX) make_point_onscreen( &wpl->ptMaxPosition );
   if ( Flags & PLACE_RECT) make_rect_onscreen( &wpl->rcNormalPosition );

   if (!Wnd || Wnd == Wnd->head.rpdesk->pDeskInfo->spwnd) return FALSE;

   if ( Flags & PLACE_MIN ) Wnd->InternalPos.IconPos = wpl->ptMinPosition;
   if ( Flags & PLACE_MAX ) Wnd->InternalPos.MaxPos = wpl->ptMaxPosition;
   if ( Flags & PLACE_RECT) Wnd->InternalPos.NormalRect = wpl->rcNormalPosition;

   SWP_Flags = SWP_NOZORDER | SWP_NOACTIVATE | ((wpl->flags & WPF_ASYNCWINDOWPLACEMENT) ? SWP_ASYNCWINDOWPOS : 0);

   if (Wnd->style & WS_MINIMIZE )
   {
      if (Flags & PLACE_MIN || Wnd->InternalPos.flags & WPF_SETMINPOSITION)
      {
         co_WinPosSetWindowPos(Wnd, HWND_TOP,
                               wpl->ptMinPosition.x, wpl->ptMinPosition.y, 0, 0,
                               SWP_NOSIZE | SWP_Flags);
         Wnd->InternalPos.flags |= WPF_MININIT;
      }
   }
   else if (Wnd->style & WS_MAXIMIZE )
   {
      if (Flags & PLACE_MAX)
      {
         co_WinPosSetWindowPos(Wnd, HWND_TOP,
                               wpl->ptMaxPosition.x, wpl->ptMaxPosition.y, 0, 0,
                               SWP_NOSIZE | SWP_Flags);
         Wnd->InternalPos.flags |= WPF_MAXINIT;
      }
   }
   else if (Flags & PLACE_RECT)
   {
      co_WinPosSetWindowPos(Wnd, HWND_TOP,
                            wpl->rcNormalPosition.left, wpl->rcNormalPosition.top,
                            wpl->rcNormalPosition.right - wpl->rcNormalPosition.left,
                            wpl->rcNormalPosition.bottom - wpl->rcNormalPosition.top,
                            SWP_Flags);
   }

   sAsync = (Wnd->head.pti->MessageQueue != gptiCurrent->MessageQueue && wpl->flags & WPF_ASYNCWINDOWPLACEMENT);

   if ( sAsync )
      co_IntSendMessageNoWait( UserHMGetHandle(Wnd), WM_ASYNC_SHOWWINDOW, wpl->showCmd, 0 );
   else
      co_WinPosShowWindow(Wnd, wpl->showCmd);

   if ( Wnd->style & WS_MINIMIZE && !sAsync )
   {
      if ( wpl->flags & WPF_SETMINPOSITION )
         Wnd->InternalPos.flags |= WPF_SETMINPOSITION;

      if ( wpl->flags & WPF_RESTORETOMAXIMIZED )
         Wnd->InternalPos.flags |= WPF_RESTORETOMAXIMIZED;
   }
   return TRUE;
}

UINT
FASTCALL
co_WinPosArrangeIconicWindows(PWND parent)
{
   RECTL rectParent;
   PWND Child;
   INT x, y, xspacing, yspacing, sx, sy;

   ASSERT_REFS_CO(parent);

   IntGetClientRect( parent, &rectParent );
   // FIXME: Support Minimize Metrics gspv.mm.iArrange.
   // Default: ARW_BOTTOMLEFT
   x = rectParent.left;
   y = rectParent.bottom;

   xspacing = UserGetSystemMetrics(SM_CXMINIMIZED);
   yspacing = UserGetSystemMetrics(SM_CYMINIMIZED);

   Child = parent->spwndChild;
   while(Child)
   {
      if((Child->style & WS_MINIMIZE) != 0 )
      {
         USER_REFERENCE_ENTRY Ref;
         UserRefObjectCo(Child, &Ref);

         sx = x + UserGetSystemMetrics(SM_CXBORDER);
         sy = y - yspacing - UserGetSystemMetrics(SM_CYBORDER);

         Child->InternalPos.IconPos.x = sx;
         Child->InternalPos.IconPos.y = sy;
         Child->InternalPos.flags |= WPF_MININIT;

         co_WinPosSetWindowPos( Child, 0, sx, sy, xspacing, yspacing, SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_ASYNCWINDOWPOS);

         UserDerefObjectCo(Child);

         if (x <= rectParent.right - xspacing)
            x += xspacing;
         else
         {
            x = rectParent.left;
            y -= yspacing;
         }
      }
      Child = Child->spwndNext;
   }
   return yspacing;
}

static VOID FASTCALL
WinPosFindIconPos(PWND Window, POINT *Pos)
{
   RECT rectParent;
   PWND pwndChild, pwndParent;
   int x, y, xspacing, yspacing;

   pwndParent = Window->spwndParent;
   if (UserIsDesktopWindow(pwndParent))
   {
      ERR("FIXME: Parent is Desktop, Min off screen!\n");
      /* FIXME: ReactOS doesn't support iconic minimize to desktop */
      Pos->x = Pos->y = -32000;
      Window->InternalPos.flags |= WPF_MININIT;
      Window->InternalPos.IconPos.x = Pos->x;
      Window->InternalPos.IconPos.y = Pos->y;
      return;
   }

   IntGetClientRect( pwndParent, &rectParent );
   // FIXME: Support Minimize Metrics gspv.mm.iArrange.
   // Default: ARW_BOTTOMLEFT
   x = rectParent.left;
   y = rectParent.bottom;

   xspacing = UserGetSystemMetrics(SM_CXMINIMIZED);
   yspacing = UserGetSystemMetrics(SM_CYMINIMIZED);

   // Set to default position when minimized.
   Pos->x = x + UserGetSystemMetrics(SM_CXBORDER);
   Pos->y = y - yspacing - UserGetSystemMetrics(SM_CYBORDER);

   for (pwndChild = pwndParent->spwndChild; pwndChild; pwndChild = pwndChild->spwndNext)
   {
        if (pwndChild == Window) continue;

        if ((pwndChild->style & (WS_VISIBLE|WS_MINIMIZE)) != (WS_VISIBLE|WS_MINIMIZE) )
        {
            continue;
        }

        if ( pwndChild->InternalPos.IconPos.x != Pos->x && pwndChild->InternalPos.IconPos.y != Pos->y )
        {
           break;
        }
        if (x <= rectParent.right - xspacing)
            x += xspacing;
        else
        {
            x = rectParent.left;
            y -= yspacing;
        }
        Pos->x = x + UserGetSystemMetrics(SM_CXBORDER);
        Pos->y = y - yspacing - UserGetSystemMetrics(SM_CYBORDER);
   }

   Window->InternalPos.IconPos.x = Pos->x;
   Window->InternalPos.IconPos.y = Pos->y;
   Window->InternalPos.flags |= WPF_MININIT;
   TRACE("Position is set! X:%d Y:%d\n",Pos->x,Pos->y);
   return;
}

BOOL
UserHasWindowEdge(DWORD Style, DWORD ExStyle)
{
   if (Style & WS_MINIMIZE)
      return TRUE;
   if (ExStyle & WS_EX_DLGMODALFRAME)
      return TRUE;
   if (ExStyle & WS_EX_STATICEDGE)
      return FALSE;
   if (Style & WS_THICKFRAME)
      return TRUE;
   Style &= WS_CAPTION;
   if (Style == WS_DLGFRAME || Style == WS_CAPTION)
      return TRUE;
   return FALSE;
}

VOID FASTCALL
IntGetWindowBorderMeasures(PWND Wnd, UINT *cx, UINT *cy)
{
   if(HAS_DLGFRAME(Wnd->style, Wnd->ExStyle) && !(Wnd->style & WS_MINIMIZE))
   {
      *cx = UserGetSystemMetrics(SM_CXDLGFRAME);
      *cy = UserGetSystemMetrics(SM_CYDLGFRAME);
   }
   else
   {
      if(HAS_THICKFRAME(Wnd->style, Wnd->ExStyle)&& !(Wnd->style & WS_MINIMIZE))
      {
         *cx = UserGetSystemMetrics(SM_CXFRAME);
         *cy = UserGetSystemMetrics(SM_CYFRAME);
      }
      else if(HAS_THINFRAME(Wnd->style, Wnd->ExStyle))
      {
         *cx = UserGetSystemMetrics(SM_CXBORDER);
         *cy = UserGetSystemMetrics(SM_CYBORDER);
      }
      else
      {
         *cx = *cy = 0;
      }
   }
}

VOID
UserGetWindowBorders(DWORD Style, DWORD ExStyle, SIZE *Size, BOOL WithClient)
{
   DWORD Border = 0;

   if (UserHasWindowEdge(Style, ExStyle))
      Border += 2;
   else if ((ExStyle & (WS_EX_STATICEDGE|WS_EX_DLGMODALFRAME)) == WS_EX_STATICEDGE)
      Border += 1; /* for the outer frame always present */
   if ((ExStyle & WS_EX_CLIENTEDGE) && WithClient)
      Border += 2;
   if (Style & WS_CAPTION || ExStyle & WS_EX_DLGMODALFRAME)
      Border ++; /* The other border */
   Size->cx = Size->cy = Border;
   if ((Style & WS_THICKFRAME) && !(Style & WS_MINIMIZE)) /* The resize border */
   {
      Size->cx += UserGetSystemMetrics(SM_CXFRAME) - UserGetSystemMetrics(SM_CXDLGFRAME);
      Size->cy += UserGetSystemMetrics(SM_CYFRAME) - UserGetSystemMetrics(SM_CYDLGFRAME);
   }
   Size->cx *= UserGetSystemMetrics(SM_CXBORDER);
   Size->cy *= UserGetSystemMetrics(SM_CYBORDER);
}

//
// Fix CORE-5177
// See winetests:user32:win.c:wine_AdjustWindowRectEx, 
// Simplified version.
//
DWORD IntGetWindowBorders(DWORD Style, DWORD ExStyle)
{
    DWORD adjust = 0;

    if ( ExStyle & WS_EX_WINDOWEDGE )      // 1st
        adjust = 2; /* outer */
    else if ( ExStyle & WS_EX_STATICEDGE ) // 2nd
        adjust = 1; /* for the outer frame always present */

    if (ExStyle & WS_EX_CLIENTEDGE)
       adjust += 2;

    if ( Style & WS_CAPTION || ExStyle & WS_EX_DLGMODALFRAME )
        adjust++; /* The other border */

    return adjust;
}

UINT FASTCALL
co_WinPosGetMinMaxInfo(PWND Window, POINT* MaxSize, POINT* MaxPos,
                       POINT* MinTrack, POINT* MaxTrack)
{
    MINMAXINFO MinMax;
    PMONITOR monitor;
    INT xinc, yinc;
    LONG style = Window->style;
    LONG adjustedStyle;
    LONG exstyle = Window->ExStyle;
    RECT rc;
    DWORD adjust;

    ASSERT_REFS_CO(Window);

    /* Compute default values */

    rc = Window->rcWindow;
    MinMax.ptReserved.x = rc.left;
    MinMax.ptReserved.y = rc.top;

    if ((style & WS_CAPTION) == WS_CAPTION)
        adjustedStyle = style & ~WS_BORDER; /* WS_CAPTION = WS_DLGFRAME | WS_BORDER */
    else
        adjustedStyle = style;

    if (Window->spwndParent)
        IntGetClientRect(Window->spwndParent, &rc);

    adjust = IntGetWindowBorders(adjustedStyle, exstyle);

    // Handle special case while maximized. CORE-15893
    if ((adjustedStyle & WS_THICKFRAME) && !(adjustedStyle & WS_CHILD) && !(adjustedStyle & WS_MINIMIZE))
         adjust += 1;

    xinc = yinc = adjust;

    if ((adjustedStyle & WS_THICKFRAME) && (adjustedStyle & WS_CHILD) && !(adjustedStyle & WS_MINIMIZE))
    {
        xinc += UserGetSystemMetrics(SM_CXFRAME) - UserGetSystemMetrics(SM_CXDLGFRAME);
        yinc += UserGetSystemMetrics(SM_CYFRAME) - UserGetSystemMetrics(SM_CYDLGFRAME);
    }

    RECTL_vInflateRect( &rc,
                        xinc * UserGetSystemMetrics(SM_CXBORDER),
                        yinc * UserGetSystemMetrics(SM_CYBORDER) );

    xinc = -rc.left;
    yinc = -rc.top;

    MinMax.ptMaxSize.x = rc.right - rc.left;
    MinMax.ptMaxSize.y = rc.bottom - rc.top;
    if (style & (WS_DLGFRAME | WS_BORDER))
    {
        MinMax.ptMinTrackSize.x = UserGetSystemMetrics(SM_CXMINTRACK);
        MinMax.ptMinTrackSize.y = UserGetSystemMetrics(SM_CYMINTRACK);
    }
    else
    {
        MinMax.ptMinTrackSize.x = 2 * xinc;
        MinMax.ptMinTrackSize.y = 2 * yinc;
    }
    MinMax.ptMaxTrackSize.x = UserGetSystemMetrics(SM_CXMAXTRACK);
    MinMax.ptMaxTrackSize.y = UserGetSystemMetrics(SM_CYMAXTRACK);
    MinMax.ptMaxPosition.x = -xinc;
    MinMax.ptMaxPosition.y = -yinc;

    if (!EMPTYPOINT(Window->InternalPos.MaxPos)) MinMax.ptMaxPosition = Window->InternalPos.MaxPos;

    co_IntSendMessage(UserHMGetHandle(Window), WM_GETMINMAXINFO, 0, (LPARAM)&MinMax);

    /* if the app didn't change the values, adapt them for the current monitor */
    if ((monitor = UserGetPrimaryMonitor()))
    {
        RECT rc_work;

        rc_work = monitor->rcMonitor;

        if (style & WS_MAXIMIZEBOX)
        {
            if ((style & WS_CAPTION) == WS_CAPTION || !(style & (WS_CHILD | WS_POPUP)))
               rc_work = monitor->rcWork;
        }

        if (MinMax.ptMaxSize.x == UserGetSystemMetrics(SM_CXSCREEN) + 2 * xinc &&
            MinMax.ptMaxSize.y == UserGetSystemMetrics(SM_CYSCREEN) + 2 * yinc)
        {
            MinMax.ptMaxSize.x = (rc_work.right - rc_work.left) + 2 * xinc;
            MinMax.ptMaxSize.y = (rc_work.bottom - rc_work.top) + 2 * yinc;
        }
        if (MinMax.ptMaxPosition.x == -xinc && MinMax.ptMaxPosition.y == -yinc)
        {
            MinMax.ptMaxPosition.x = rc_work.left - xinc;
            MinMax.ptMaxPosition.y = rc_work.top - yinc;
        }
        if (MinMax.ptMaxSize.x >= (monitor->rcMonitor.right - monitor->rcMonitor.left) &&
            MinMax.ptMaxSize.y >= (monitor->rcMonitor.bottom - monitor->rcMonitor.top) )
        {
            Window->state |= WNDS_MAXIMIZESTOMONITOR;
        }
        else
            Window->state &= ~WNDS_MAXIMIZESTOMONITOR;
    }


    MinMax.ptMaxTrackSize.x = max(MinMax.ptMaxTrackSize.x,
                                  MinMax.ptMinTrackSize.x);
    MinMax.ptMaxTrackSize.y = max(MinMax.ptMaxTrackSize.y,
                                  MinMax.ptMinTrackSize.y);

    if (MaxSize)
       *MaxSize = MinMax.ptMaxSize;
    if (MaxPos)
       *MaxPos = MinMax.ptMaxPosition;
    if (MinTrack)
       *MinTrack = MinMax.ptMinTrackSize;
    if (MaxTrack)
       *MaxTrack = MinMax.ptMaxTrackSize;

    return 0; // FIXME: What does it return? Wine returns MINMAXINFO.
}
/* --- pretranslation split: the remaining function groups live in winpos.split2..4.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "winpos.split2.c"
#include "winpos.split3.c"
#include "winpos.split4.c"
