/* winpos.split3.c — #include fragment of winpos.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original winpos.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


LRESULT FASTCALL
co_WinPosGetNonClientSize(PWND Window, RECT* WindowRect, RECT* ClientRect)
{
   LRESULT Result;

   ASSERT_REFS_CO(Window);

   *ClientRect = *WindowRect;
   Result = co_IntSendMessageNoWait(UserHMGetHandle(Window), WM_NCCALCSIZE, FALSE, (LPARAM)ClientRect);

   FixClientRect(ClientRect, WindowRect);

   return Result;
}

void FASTCALL
co_WinPosSendSizeMove(PWND Wnd)
{
    RECTL Rect;
    LPARAM lParam;
    WPARAM wParam = SIZE_RESTORED;

    IntGetClientRect(Wnd, &Rect);
    lParam = MAKELONG(Rect.right-Rect.left, Rect.bottom-Rect.top);

    Wnd->state &= ~WNDS_SENDSIZEMOVEMSGS;

    if (Wnd->style & WS_MAXIMIZE)
    {
        wParam = SIZE_MAXIMIZED;
    }
    else if (Wnd->style & WS_MINIMIZE)
    {
        wParam = SIZE_MINIMIZED;
        lParam = 0;
    }

    co_IntSendMessageNoWait(UserHMGetHandle(Wnd), WM_SIZE, wParam, lParam);

    if (UserIsDesktopWindow(Wnd->spwndParent))
       lParam = MAKELONG(Wnd->rcClient.left, Wnd->rcClient.top);
    else
       lParam = MAKELONG(Wnd->rcClient.left-Wnd->spwndParent->rcClient.left, Wnd->rcClient.top-Wnd->spwndParent->rcClient.top);

    co_IntSendMessageNoWait(UserHMGetHandle(Wnd), WM_MOVE, 0, lParam);

    IntEngWindowChanged(Wnd, WOC_RGN_CLIENT);
}

UINT FASTCALL
co_WinPosMinMaximize(PWND Wnd, UINT ShowFlag, RECT* NewPos)
{
   POINT Size;
   WINDOWPLACEMENT wpl;
   LONG old_style;
   UINT SwpFlags = 0;

   ASSERT_REFS_CO(Wnd);

   wpl.length = sizeof(wpl);
   IntGetWindowPlacement( Wnd, &wpl );

   if (co_HOOK_CallHooks(WH_CBT, HCBT_MINMAX, (WPARAM)UserHMGetHandle(Wnd), ShowFlag))
   {
      ERR("WinPosMinMaximize WH_CBT Call Hook return!\n");
      return SWP_NOSIZE | SWP_NOMOVE;
   }
      if (Wnd->style & WS_MINIMIZE)
      {
         switch (ShowFlag)
         {
         case SW_MINIMIZE:
         case SW_SHOWMINNOACTIVE:
         case SW_SHOWMINIMIZED:
         case SW_FORCEMINIMIZE:
             return SWP_NOSIZE | SWP_NOMOVE;
         }
         if (!co_IntSendMessageNoWait(UserHMGetHandle(Wnd), WM_QUERYOPEN, 0, 0))
         {
            return(SWP_NOSIZE | SWP_NOMOVE);
         }
         SwpFlags |= SWP_NOCOPYBITS;
      }
      switch (ShowFlag)
      {
         case SW_MINIMIZE:
         case SW_SHOWMINNOACTIVE:
         case SW_SHOWMINIMIZED:
         case SW_FORCEMINIMIZE:
            {
               //ERR("MinMaximize Minimize\n");
               if (Wnd->style & WS_MAXIMIZE)
               {
                  Wnd->InternalPos.flags |= WPF_RESTORETOMAXIMIZED;
               }
               else
               {
                  Wnd->InternalPos.flags &= ~WPF_RESTORETOMAXIMIZED;
               }

               old_style = IntSetStyle( Wnd, WS_MINIMIZE, WS_MAXIMIZE );

               co_UserRedrawWindow(Wnd, NULL, 0, RDW_VALIDATE | RDW_NOERASE | RDW_NOINTERNALPAINT);

               if (!(Wnd->InternalPos.flags & WPF_SETMINPOSITION))
                  Wnd->InternalPos.flags &= ~WPF_MININIT;

               WinPosFindIconPos(Wnd, &wpl.ptMinPosition);

               if (!(old_style & WS_MINIMIZE))
               {
                  SwpFlags |= SWP_STATECHANGED;
                  IntShowOwnedPopups(Wnd, FALSE);
               }

               RECTL_vSetRect(NewPos, wpl.ptMinPosition.x, wpl.ptMinPosition.y,
                             wpl.ptMinPosition.x + UserGetSystemMetrics(SM_CXMINIMIZED),
                             wpl.ptMinPosition.y + UserGetSystemMetrics(SM_CYMINIMIZED));
               SwpFlags |= SWP_NOCOPYBITS;
               break;
            }

         case SW_MAXIMIZE:
            {
               //ERR("MinMaximize Maximize\n");
               IntSetSnapEdge(Wnd, HTNOWHERE); /* Mark as not snapped (for Win+Left,Up,Down) */
               if ((Wnd->style & WS_MAXIMIZE) && (Wnd->style & WS_VISIBLE))
               {
                  SwpFlags = SWP_NOSIZE | SWP_NOMOVE;
                  break;
               }
               co_WinPosGetMinMaxInfo(Wnd, &Size, &wpl.ptMaxPosition, NULL, NULL);

               /*ERR("Maximize: %d,%d %dx%d\n",
                      wpl.ptMaxPosition.x, wpl.ptMaxPosition.y, Size.x, Size.y);
                */
               old_style = IntSetStyle( Wnd, WS_MAXIMIZE, WS_MINIMIZE );
               /*if (old_style & WS_MINIMIZE)
               {
                  IntShowOwnedPopups(Wnd, TRUE);
               }*/

               if (!(old_style & WS_MAXIMIZE)) SwpFlags |= SWP_STATECHANGED;
               RECTL_vSetRect(NewPos, wpl.ptMaxPosition.x, wpl.ptMaxPosition.y,
                              //wpl.ptMaxPosition.x + Size.x, wpl.ptMaxPosition.y + Size.y);
                              Size.x, Size.y);
               break;
            }

         case SW_SHOWNOACTIVATE:
            Wnd->InternalPos.flags &= ~WPF_RESTORETOMAXIMIZED;
            /* fall through */
         case SW_SHOWNORMAL:
         case SW_RESTORE:
         case SW_SHOWDEFAULT: /* FIXME: should have its own handler */
            {
               //ERR("MinMaximize Restore\n");
               old_style = IntSetStyle( Wnd, 0, WS_MINIMIZE | WS_MAXIMIZE );
               if (old_style & WS_MINIMIZE)
               {
                  IntShowOwnedPopups(Wnd, TRUE);

                  if (Wnd->InternalPos.flags & WPF_RESTORETOMAXIMIZED)
                  {
                     co_WinPosGetMinMaxInfo(Wnd, &Size, &wpl.ptMaxPosition, NULL, NULL);
                     IntSetStyle( Wnd, WS_MAXIMIZE, 0 );
                     SwpFlags |= SWP_STATECHANGED;
                     RECTL_vSetRect(NewPos, wpl.ptMaxPosition.x, wpl.ptMaxPosition.y,
                                    wpl.ptMaxPosition.x + Size.x, wpl.ptMaxPosition.y + Size.y);
                     break;
                  }
                  else
                  {
                     *NewPos = wpl.rcNormalPosition;
                     if (ShowFlag != SW_SHOWNORMAL && ShowFlag != SW_SHOWDEFAULT)
                     {
                        UINT edge = IntGetWindowSnapEdge(Wnd);
                        if (edge)
                           co_IntCalculateSnapPosition(Wnd, edge, NewPos);
                     }
                     NewPos->right -= NewPos->left;
                     NewPos->bottom -= NewPos->top;
                     break;
                  }
               }
               else
               {
                  if (!(old_style & WS_MAXIMIZE))
                  {
                     break;
                  }
                  SwpFlags |= SWP_STATECHANGED;
                  Wnd->InternalPos.flags &= ~WPF_RESTORETOMAXIMIZED;
                  *NewPos = wpl.rcNormalPosition;
                  NewPos->right -= NewPos->left;
                  NewPos->bottom -= NewPos->top;
                  break;
               }
            }
      }
   return SwpFlags;
}

// SW_FORCEMINIMIZE
void IntForceMinimizeWindow(PWND pWnd)
{
    HRGN hRgn;
    PREGION pRgn;

    if ((pWnd->style & (WS_MINIMIZE | WS_VISIBLE)) != WS_VISIBLE)
        return;

    if (pWnd->state & WNDS_DESTROYED)
        return;

    pWnd->ExStyle &= ~WS_EX_MAKEVISIBLEWHENUNGHOSTED;

    IntSetStyle(pWnd, 0, WS_VISIBLE);

    // Invalidate and redraw the window region
    hRgn = GreCreateRectRgnIndirect(&pWnd->rcWindow);
    pRgn = REGION_LockRgn(hRgn);
    co_UserRedrawWindow(UserGetDesktopWindow(), NULL, pRgn, RDW_ALLCHILDREN | RDW_ERASE | RDW_INVALIDATE);
    REGION_UnlockRgn(pRgn);
    GreDeleteObject(hRgn);

    // Activate the other window if necessary
    if (pWnd->spwndParent == pWnd->head.rpdesk->pDeskInfo->spwnd)
        co_WinPosActivateOtherWindow(pWnd);
}

/*
   ShowWindow does not set SWP_FRAMECHANGED!!! Fix wine msg test_SetParent:WmSetParentSeq_2:23 wParam bits!
 */
BOOLEAN FASTCALL
co_WinPosShowWindow(PWND Wnd, INT Cmd)
{
   BOOLEAN WasVisible;
   UINT Swp = 0, EventMsg = 0;
   RECTL NewPos = {0, 0, 0, 0};
   BOOLEAN ShowFlag;
   LONG style;
   PWND Parent;
   PTHREADINFO pti;
   //HRGN VisibleRgn;
   BOOL ShowOwned = FALSE;
   BOOL FirstTime = FALSE;
   ASSERT_REFS_CO(Wnd);
   //KeRosDumpStackFrames(NULL, 20);
   pti = PsGetCurrentThreadWin32Thread();
   WasVisible = (Wnd->style & WS_VISIBLE) != 0;
   style = Wnd->style;

   TRACE("co_WinPosShowWindow START hwnd %p Cmd %d usicmd %u\n",
         UserHMGetHandle(Wnd), Cmd, pti->ppi->usi.wShowWindow);

   if ( pti->ppi->usi.dwFlags & STARTF_USESHOWWINDOW )
   {
      if ((Wnd->style & (WS_POPUP|WS_CHILD)) != WS_CHILD)
      {
         if ((Wnd->style & WS_CAPTION) == WS_CAPTION)
         {
            if (Wnd->spwndOwner == NULL)
            {
               if ( Cmd == SW_SHOWNORMAL || Cmd == SW_SHOW)
               {
                    Cmd = SW_SHOWDEFAULT;
               }
               FirstTime = TRUE;
               TRACE("co_WPSW FT 1\n");
            }
         }
      }
   }

   if ( Cmd == SW_SHOWDEFAULT )
   {
      if ( pti->ppi->usi.dwFlags & STARTF_USESHOWWINDOW )
      {
         Cmd = pti->ppi->usi.wShowWindow;
         FirstTime = TRUE;
         TRACE("co_WPSW FT 2\n");
      }
   }

   if (FirstTime)
   {
      pti->ppi->usi.dwFlags &= ~(STARTF_USEPOSITION|STARTF_USESIZE|STARTF_USESHOWWINDOW);
   }

   switch (Cmd)
   {
      case SW_HIDE:
         {
            if (!WasVisible)
            {
               //ERR("co_WinPosShowWindow Exit Bad\n");
               return FALSE;
            }
            Swp |= SWP_HIDEWINDOW | SWP_NOSIZE | SWP_NOMOVE;
            if (Wnd != pti->MessageQueue->spwndActive)
               Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
            break;
         }

      case SW_FORCEMINIMIZE:
         IntForceMinimizeWindow(Wnd);
         return WasVisible;

      case SW_SHOWMINNOACTIVE:
         Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
         /* Fall through. */
      case SW_SHOWMINIMIZED:
      case SW_MINIMIZE: /* CORE-15669: SW_MINIMIZE also shows */
         Swp |= SWP_SHOWWINDOW;
         {
            Swp |= SWP_NOACTIVATE;
            if (!(style & WS_MINIMIZE))
            {
               IntShowOwnedPopups(Wnd, FALSE );
               // Fix wine Win test_SetFocus todo #1 & #2,
               if (Cmd == SW_SHOWMINIMIZED)
               {
                  //ERR("co_WinPosShowWindow Set focus 1\n");
                  if ((style & (WS_CHILD | WS_POPUP)) == WS_CHILD)
                     co_UserSetFocus(Wnd->spwndParent);
                  else
                     co_UserSetFocus(0);
               }

               Swp |= co_WinPosMinMaximize(Wnd, Cmd, &NewPos);

               EventMsg = EVENT_SYSTEM_MINIMIZESTART;
            }
            else
            {
               if (WasVisible)
               {
                  //ERR("co_WinPosShowWindow Exit Good\n");
                  return TRUE;
               }
               Swp |= SWP_NOSIZE | SWP_NOMOVE;
            }
            break;
         }

      case SW_SHOWMAXIMIZED:
         {
            Swp |= SWP_SHOWWINDOW;
            if (!(style & WS_MAXIMIZE))
            {
               ShowOwned = TRUE;

               Swp |= co_WinPosMinMaximize(Wnd, SW_MAXIMIZE, &NewPos);

               EventMsg = EVENT_SYSTEM_MINIMIZEEND;
            }
            else
            {
               if (WasVisible)
               {
                  //ERR("co_WinPosShowWindow Exit Good 1\n");
                  return TRUE;
               }
               Swp |= SWP_NOSIZE | SWP_NOMOVE;
            }
            break;
         }

      case SW_SHOWNA:
         Swp |= SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_NOSIZE | SWP_NOMOVE;
         if (style & WS_CHILD && !(Wnd->ExStyle & WS_EX_MDICHILD)) Swp |= SWP_NOZORDER;
         break;
      case SW_SHOW:
         if (WasVisible) return(TRUE); // Nothing to do!
         Swp |= SWP_SHOWWINDOW | SWP_NOSIZE | SWP_NOMOVE;
         /* Don't activate the topmost window. */
         if (style & WS_CHILD && !(Wnd->ExStyle & WS_EX_MDICHILD)) Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
         break;

      case SW_SHOWNOACTIVATE:
         Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
         /* Fall through. */
      case SW_SHOWNORMAL:
      case SW_SHOWDEFAULT:
      case SW_RESTORE:
         if (!WasVisible) Swp |= SWP_SHOWWINDOW;
         if (style & (WS_MINIMIZE | WS_MAXIMIZE))
         {
            Swp |= co_WinPosMinMaximize(Wnd, Cmd, &NewPos);
            if (style & WS_MINIMIZE) EventMsg = EVENT_SYSTEM_MINIMIZEEND;
         }
         else
         {
            if (WasVisible)
            {
               //ERR("co_WinPosShowWindow Exit Good 3\n");
               return TRUE;
            }
            Swp |= SWP_NOSIZE | SWP_NOMOVE;
         }
         if ( style & WS_CHILD &&
             !(Wnd->ExStyle & WS_EX_MDICHILD) &&
             !(Swp & SWP_STATECHANGED))
            Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
         break;

      default:
         //ERR("co_WinPosShowWindow Exit Good 4\n");
         return FALSE;
   }

   ShowFlag = (Cmd != SW_HIDE);

   if ((ShowFlag != WasVisible || Cmd == SW_SHOWNA) && Cmd != SW_SHOWMAXIMIZED && !(Swp & SWP_STATECHANGED))
   {
      co_IntSendMessageNoWait(UserHMGetHandle(Wnd), WM_SHOWWINDOW, ShowFlag, 0);
#if 0 // Fix wine msg test_SetParent:WmSetParentSeq_1:2
      if (!(Wnd->state2 & WNDS2_WIN31COMPAT)) // <------------- XP sets this bit!
         co_IntSendMessageNoWait(UserHMGetHandle(Wnd), WM_SETVISIBLE, ShowFlag, 0);
#endif
      if (!VerifyWnd(Wnd)) return WasVisible;
   }

   /* We can't activate a child window */
   if ((Wnd->style & WS_CHILD) &&
       !(Wnd->ExStyle & WS_EX_MDICHILD) &&
       Cmd != SW_SHOWNA)
   {
      //ERR("SWP Child No active and ZOrder\n");
      Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
   }

#if 0 // Explorer issues with common controls? Someone does not know how CS_SAVEBITS works.
      // Breaks startup and shutdown active window...
   if ((Wnd->style & (WS_POPUP|WS_CHILD)) != WS_CHILD &&
        Wnd->pcls->style & CS_SAVEBITS &&
        ((Cmd == SW_SHOW) || (Cmd == SW_NORMAL)))
   {
      ERR("WinPosShowWindow Set active\n");
      //UserSetActiveWindow(Wnd);
      co_IntSetForegroundWindow(Wnd); // HACK
      Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
   }
#endif

   if (IsChildVisible(Wnd) || Swp & SWP_STATECHANGED)
   {
       TRACE("Child is Vis %s or State changed %s. ShowFlag %s Swp %04x\n",
             (IsChildVisible(Wnd) ? "TRUE" : "FALSE"), (Swp & SWP_STATECHANGED ? "TRUE" : "FALSE"),
             (ShowFlag ? "TRUE" : "FALSE"),LOWORD(Swp));
   co_WinPosSetWindowPos( Wnd,
                          0 != (Wnd->ExStyle & WS_EX_TOPMOST) ? HWND_TOPMOST : HWND_TOP,
                          NewPos.left,
                          NewPos.top,
                          NewPos.right, // NewPos.right - NewPos.left, when minimized and restore, the window becomes smaller.
                          NewPos.bottom,// NewPos.bottom - NewPos.top,
                          LOWORD(Swp));
   }
   else
   {
      TRACE("Parent Vis?\n");
      /* if parent is not visible simply toggle WS_VISIBLE and return */
      if (ShowFlag) IntSetStyle( Wnd, WS_VISIBLE, 0 );
      else IntSetStyle( Wnd, 0, WS_VISIBLE );
   }

   if ( EventMsg ) IntNotifyWinEvent(EventMsg, Wnd, OBJID_WINDOW, CHILDID_SELF, WEF_SETBYWNDPTI);

   if ( ShowOwned ) IntShowOwnedPopups(Wnd, TRUE );

   if ((Cmd == SW_HIDE) || (Cmd == SW_MINIMIZE))
   {
      if ( Wnd == pti->MessageQueue->spwndActive && pti->MessageQueue == IntGetFocusMessageQueue()  )
      {
          if (UserIsDesktopWindow(Wnd->spwndParent))
          {
              if (!ActivateOtherWindowMin(Wnd))
              {
                co_WinPosActivateOtherWindow(Wnd);
              }
          }
          else
          {
              co_WinPosActivateOtherWindow(Wnd);
          }
      }

      /* Revert focus to parent */
      if (Wnd == pti->MessageQueue->spwndFocus)
      {
         Parent = Wnd->spwndParent;
         if (UserIsDesktopWindow(Wnd->spwndParent))
             Parent = 0;
         co_UserSetFocus(Parent);
      }
      // Hide, just return.
      if (Cmd == SW_HIDE) return WasVisible;
   }

   /* FIXME: Check for window destruction. */

   if ((Wnd->state & WNDS_SENDSIZEMOVEMSGS) &&
       !(Wnd->state2 & WNDS2_INDESTROY))
   {
        co_WinPosSendSizeMove(Wnd);
   }

   /* if previous state was minimized Windows sets focus to the window */
   if (style & WS_MINIMIZE)
   {
      co_UserSetFocus(Wnd);
      // Fix wine Win test_SetFocus todo #3,
      if (!(style & WS_CHILD)) co_IntSendMessage(UserHMGetHandle(Wnd), WM_ACTIVATE, WA_ACTIVE, 0);
   }
   //ERR("co_WinPosShowWindow EXIT\n");
   return WasVisible;
}

static PWND
co_WinPosSearchChildren(
   IN PWND ScopeWin,
   IN POINT *Point,
   IN OUT USHORT *HitTest,
   IN BOOL Ignore
   )
{
    HWND *List, *phWnd;
    PWND pwndChild = NULL;

    /* not visible */
    if (!(ScopeWin->style & WS_VISIBLE))
    {
        return NULL;
    }

    /* not in window or in window region */
    if (!IntPtInWindow(ScopeWin, Point->x, Point->y))
    {
        return NULL;
    }

    /* transparent */
    if ((ScopeWin->ExStyle & (WS_EX_LAYERED|WS_EX_TRANSPARENT)) == (WS_EX_LAYERED|WS_EX_TRANSPARENT))
    {
        return NULL;
    }

    if (!Ignore && (ScopeWin->style & WS_DISABLED))
    {   /* disabled child */
        if ((ScopeWin->style & (WS_POPUP|WS_CHILD)) == WS_CHILD) return NULL;
        /* process the hit error */
        *HitTest = HTERROR;
        return ScopeWin;
    }

    /* not minimized and check if point is inside the window */
    if (!(ScopeWin->style & WS_MINIMIZE) &&
         RECTL_bPointInRect(&ScopeWin->rcClient, Point->x, Point->y) )
    {
        UserReferenceObject(ScopeWin);

        List = IntWinListChildren(ScopeWin);
        if (List)
        {
            for (phWnd = List; *phWnd; ++phWnd)
            {
                if (!(pwndChild = ValidateHwndNoErr(*phWnd)))
                {
                    continue;
                }

                pwndChild = co_WinPosSearchChildren(pwndChild, Point, HitTest, Ignore);

                if (pwndChild != NULL)
                {
                    /* We found a window. Don't send any more WM_NCHITTEST messages */
                    ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
                    UserDereferenceObject(ScopeWin);
                    return pwndChild;
                }
            }
            ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
        }
        UserDereferenceObject(ScopeWin);
    }

    if (ScopeWin->head.pti == PsGetCurrentThreadWin32Thread())
    {
       *HitTest = (USHORT)co_IntSendMessage(UserHMGetHandle(ScopeWin), WM_NCHITTEST, 0, MAKELONG(Point->x, Point->y));

       if ((*HitTest) == (USHORT)HTTRANSPARENT)
       {
           return NULL;
       }
    }
    else
    {
       if (*HitTest == HTNOWHERE && pwndChild == NULL) *HitTest = HTCLIENT;
    }

    return ScopeWin;
}

PWND APIENTRY
co_WinPosWindowFromPoint(
   IN PWND ScopeWin,
   IN POINT *WinPoint,
   IN OUT USHORT* HitTest,
   IN BOOL Ignore)
{
   PWND Window;
   POINT Point = *WinPoint;
   USER_REFERENCE_ENTRY Ref;

   if( ScopeWin == NULL )
   {
       ScopeWin = UserGetDesktopWindow();
       if(ScopeWin == NULL)
           return NULL;
   }

   *HitTest = HTNOWHERE;

   ASSERT_REFS_CO(ScopeWin);
   UserRefObjectCo(ScopeWin, &Ref);

   Window = co_WinPosSearchChildren(ScopeWin, &Point, HitTest, Ignore);

   UserDerefObjectCo(ScopeWin);
   if (Window)
       ASSERT_REFS_CO(Window);
   ASSERT_REFS_CO(ScopeWin);

   return Window;
}

/* Win: _RealChildWindowFromPoint */
PWND FASTCALL
IntRealChildWindowFromPoint(PWND Parent, LONG x, LONG y)
{
   POINTL Pt;
   HWND *List, *phWnd;
   PWND pwndHit = NULL;

   Pt.x = x;
   Pt.y = y;

   if (!UserIsDesktopWindow(Parent))
   {
      Pt.x += Parent->rcClient.left;
      Pt.y += Parent->rcClient.top;
   }

   if (!IntPtInWindow(Parent, Pt.x, Pt.y)) return NULL;

   if ((List = IntWinListChildren(Parent)))
   {
      for (phWnd = List; *phWnd; phWnd++)
      {
         PWND Child;
         if ((Child = ValidateHwndNoErr(*phWnd)))
         {
            if ( Child->style & WS_VISIBLE && IntPtInWindow(Child, Pt.x, Pt.y) )
            {
               if ( Child->pcls->atomClassName != gpsi->atomSysClass[ICLS_BUTTON] ||
                   (Child->style & BS_TYPEMASK) != BS_GROUPBOX )
               {
                  ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
                  return Child;
               }
               pwndHit = Child;
            }
         }
      }
      ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
   }
   return pwndHit ? pwndHit : Parent;
}

/* Win: _ChildWindowFromPointEx */
PWND APIENTRY
IntChildWindowFromPointEx(PWND Parent, LONG x, LONG y, UINT uiFlags)
{
   POINTL Pt;
   HWND *List, *phWnd;
   PWND pwndHit = NULL;

   Pt.x = x;
   Pt.y = y;

   if (!UserIsDesktopWindow(Parent))
   {
      if (Parent->ExStyle & WS_EX_LAYOUTRTL)
         Pt.x = Parent->rcClient.right - Pt.x;
      else
         Pt.x += Parent->rcClient.left;
      Pt.y += Parent->rcClient.top;
   }

   if (!IntPtInWindow(Parent, Pt.x, Pt.y)) return NULL;

   if ((List = IntWinListChildren(Parent)))
   {
      for (phWnd = List; *phWnd; phWnd++)
      {
         PWND Child;
         if ((Child = ValidateHwndNoErr(*phWnd)))
         {
            if (uiFlags & (CWP_SKIPINVISIBLE|CWP_SKIPDISABLED))
            {
               if (!(Child->style & WS_VISIBLE) && (uiFlags & CWP_SKIPINVISIBLE)) continue;
               if ((Child->style & WS_DISABLED) && (uiFlags & CWP_SKIPDISABLED)) continue;
            }

            if (uiFlags & CWP_SKIPTRANSPARENT)
            {
               if (Child->ExStyle & WS_EX_TRANSPARENT) continue;
            }

            if (IntPtInWindow(Child, Pt.x, Pt.y))
            {
               pwndHit = Child;
               break;
            }
         }
      }
      ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
   }
   return pwndHit ? pwndHit : Parent;
}

/* Win: _DeferWindowPos(PSMWP, PWND, PWND, ...) */
HDWP
FASTCALL
IntDeferWindowPos( HDWP hdwp,
                   HWND hwnd,
                   HWND hwndAfter,
                   INT x,
                   INT y,
                   INT cx,
                   INT cy,
                   UINT flags )
{
    PSMWP pDWP;
    int i;
    HDWP retvalue = hdwp;

    TRACE("hdwp %p, hwnd %p, after %p, %d,%d (%dx%d), flags %08x\n",
          hdwp, hwnd, hwndAfter, x, y, cx, cy, flags);

    if (flags & ~(SWP_NOSIZE | SWP_NOMOVE |
                  SWP_NOZORDER | SWP_NOREDRAW |
                  SWP_NOACTIVATE | SWP_NOCOPYBITS |
                  SWP_NOOWNERZORDER|SWP_SHOWWINDOW |
                  SWP_HIDEWINDOW | SWP_FRAMECHANGED))
    {
       EngSetLastError(ERROR_INVALID_PARAMETER);
       return NULL;
    }

    if (!(pDWP = (PSMWP)UserGetObject(gHandleTable, hdwp, TYPE_SETWINDOWPOS)))
    {
       EngSetLastError(ERROR_INVALID_DWP_HANDLE);
       return NULL;
    }

    for (i = 0; i < pDWP->ccvr; i++)
    {
        if (pDWP->acvr[i].pos.hwnd == hwnd)
        {
              /* Merge with the other changes */
            if (!(flags & SWP_NOZORDER))
            {
                pDWP->acvr[i].pos.hwndInsertAfter = hwndAfter;
            }
            if (!(flags & SWP_NOMOVE))
            {
                pDWP->acvr[i].pos.x = x;
                pDWP->acvr[i].pos.y = y;
            }
            if (!(flags & SWP_NOSIZE))
            {
                pDWP->acvr[i].pos.cx = cx;
                pDWP->acvr[i].pos.cy = cy;
            }
            pDWP->acvr[i].pos.flags &= flags | ~(SWP_NOSIZE | SWP_NOMOVE |
                                               SWP_NOZORDER | SWP_NOREDRAW |
                                               SWP_NOACTIVATE | SWP_NOCOPYBITS|
                                               SWP_NOOWNERZORDER);
            pDWP->acvr[i].pos.flags |= flags & (SWP_SHOWWINDOW | SWP_HIDEWINDOW |
                                              SWP_FRAMECHANGED);
            goto END;
        }
    }
    if (pDWP->ccvr >= pDWP->ccvrAlloc)
    {
        PCVR newpos = ExAllocatePoolWithTag(PagedPool, pDWP->ccvrAlloc * 2 * sizeof(CVR), USERTAG_SWP);
        if (!newpos)
        {
            retvalue = NULL;
            goto END;
        }
        RtlZeroMemory(newpos, pDWP->ccvrAlloc * 2 * sizeof(CVR));
        RtlCopyMemory(newpos, pDWP->acvr, pDWP->ccvrAlloc * sizeof(CVR));
        ExFreePoolWithTag(pDWP->acvr, USERTAG_SWP);
        pDWP->ccvrAlloc *= 2;
        pDWP->acvr = newpos;
    }
    pDWP->acvr[pDWP->ccvr].pos.hwnd = hwnd;
    pDWP->acvr[pDWP->ccvr].pos.hwndInsertAfter = hwndAfter;
    pDWP->acvr[pDWP->ccvr].pos.x = x;
    pDWP->acvr[pDWP->ccvr].pos.y = y;
    pDWP->acvr[pDWP->ccvr].pos.cx = cx;
    pDWP->acvr[pDWP->ccvr].pos.cy = cy;
    pDWP->acvr[pDWP->ccvr].pos.flags = flags;
    pDWP->acvr[pDWP->ccvr].hrgnClip = NULL;
    pDWP->acvr[pDWP->ccvr].hrgnInterMonitor = NULL;
    pDWP->ccvr++;
END:
    return retvalue;
}

/* Win: xxxEndDeferWindowPosEx */
BOOL FASTCALL IntEndDeferWindowPosEx(HDWP hdwp, BOOL bAsync)
{
    PSMWP pDWP;
    PCVR winpos;
    BOOL res = TRUE;
    int i;

    TRACE("%p\n", hdwp);

    if (!(pDWP = (PSMWP)UserGetObject(gHandleTable, hdwp, TYPE_SETWINDOWPOS)))
    {
       EngSetLastError(ERROR_INVALID_DWP_HANDLE);
       return FALSE;
    }

    for (i = 0, winpos = pDWP->acvr; res && i < pDWP->ccvr; i++, winpos++)
    {
        PWND pwnd;
        USER_REFERENCE_ENTRY Ref;

        TRACE("hwnd %p, after %p, %d,%d (%dx%d), flags %08x\n",
               winpos->pos.hwnd, winpos->pos.hwndInsertAfter, winpos->pos.x, winpos->pos.y,
               winpos->pos.cx, winpos->pos.cy, winpos->pos.flags);

        pwnd = ValidateHwndNoErr(winpos->pos.hwnd);
        if (!pwnd)
           continue;

        UserRefObjectCo(pwnd, &Ref);

        if (bAsync)
        {
           LRESULT lRes;
           PWINDOWPOS ppos = ExAllocatePoolWithTag(PagedPool, sizeof(WINDOWPOS), USERTAG_SWP);
           if ( ppos )
           {
              *ppos = winpos->pos;
              /* Yes it's a pointer inside Win32k! */
              lRes = co_IntSendMessageNoWait( winpos->pos.hwnd, WM_ASYNC_SETWINDOWPOS, 0, (LPARAM)ppos);
              /* We handle this the same way as Event Hooks and Hooks. */
              if ( !lRes )
              {
                 ExFreePoolWithTag(ppos, USERTAG_SWP);
              }
           }
        }
        else
           res = co_WinPosSetWindowPos( pwnd,
                                        winpos->pos.hwndInsertAfter,
                                        winpos->pos.x,
                                        winpos->pos.y,
                                        winpos->pos.cx,
                                        winpos->pos.cy,
                                        winpos->pos.flags);

        // Hack to pass tests.... Must have some work to do so clear the error.
        if (res && (winpos->pos.flags & (SWP_NOMOVE|SWP_NOSIZE|SWP_NOZORDER)) == SWP_NOZORDER )
           EngSetLastError(ERROR_SUCCESS);

        UserDerefObjectCo(pwnd);
    }

    ExFreePoolWithTag(pDWP->acvr, USERTAG_SWP);
    UserDereferenceObject(pDWP);
    UserDeleteObject(hdwp, TYPE_SETWINDOWPOS);
    return res;
}

/*
 * @implemented
 */
HWND APIENTRY
NtUserChildWindowFromPointEx(HWND hwndParent,
                             LONG x,
                             LONG y,
                             UINT uiFlags)
{
   PWND pwndParent;
   TRACE("Enter NtUserChildWindowFromPointEx\n");
   UserEnterExclusive();
   if ((pwndParent = UserGetWindowObject(hwndParent)))
   {
      pwndParent = IntChildWindowFromPointEx(pwndParent, x, y, uiFlags);
   }
   UserLeave();
   TRACE("Leave NtUserChildWindowFromPointEx\n");
   return pwndParent ? UserHMGetHandle(pwndParent) : NULL;
}

/*
 * @implemented
 */
BOOL APIENTRY
NtUserEndDeferWindowPosEx(HDWP WinPosInfo,
                          BOOL bAsync)
{
   BOOL Ret;
   TRACE("Enter NtUserEndDeferWindowPosEx\n");
   UserEnterExclusive();
   Ret = IntEndDeferWindowPosEx(WinPosInfo, bAsync);
   TRACE("Leave NtUserEndDeferWindowPosEx, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}