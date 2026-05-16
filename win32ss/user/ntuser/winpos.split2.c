/* winpos.split2.c — #include fragment of winpos.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original winpos.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


static
BOOL
IntValidateParent(PWND Child, PREGION ValidateRgn)
{
   PWND ParentWnd = Child->spwndParent;

   while (ParentWnd)
   {
      if (ParentWnd->style & WS_CLIPCHILDREN)
         break;

      if (ParentWnd->hrgnUpdate != 0)
      {
         IntInvalidateWindows( ParentWnd,
                               ValidateRgn,
                               RDW_VALIDATE | RDW_NOCHILDREN);
      }

      ParentWnd = ParentWnd->spwndParent;
   }

   return TRUE;
}

static
VOID FASTCALL
FixClientRect(PRECTL ClientRect, PRECTL WindowRect)
{
   if (ClientRect->left < WindowRect->left)
   {
      ClientRect->left = WindowRect->left;
   }
   else if (WindowRect->right < ClientRect->left)
   {
      ClientRect->left = WindowRect->right;
   }
   if (ClientRect->right < WindowRect->left)
   {
      ClientRect->right = WindowRect->left;
   }
   else if (WindowRect->right < ClientRect->right)
   {
      ClientRect->right = WindowRect->right;
   }
   if (ClientRect->top < WindowRect->top)
   {
      ClientRect->top = WindowRect->top;
   }
   else if (WindowRect->bottom < ClientRect->top)
   {
      ClientRect->top = WindowRect->bottom;
   }
   if (ClientRect->bottom < WindowRect->top)
   {
      ClientRect->bottom = WindowRect->top;
   }
   else if (WindowRect->bottom < ClientRect->bottom)
   {
      ClientRect->bottom = WindowRect->bottom;
   }
}
/***********************************************************************
 *           get_valid_rects
 *
 * Compute the valid rects from the old and new client rect and WVR_* flags.
 * Helper for WM_NCCALCSIZE handling.
 */
static
VOID FASTCALL
get_valid_rects( RECTL *old_client, RECTL *new_client, UINT flags, RECTL *valid )
{
    int cx, cy;

    if (flags & WVR_REDRAW)
    {
        RECTL_vSetEmptyRect( &valid[0] );
        RECTL_vSetEmptyRect( &valid[1] );
        return;
    }

    if (flags & WVR_VALIDRECTS)
    {
        if (!RECTL_bIntersectRect( &valid[0], &valid[0], new_client ) ||
            !RECTL_bIntersectRect( &valid[1], &valid[1], old_client ))
        {
            RECTL_vSetEmptyRect( &valid[0] );
            RECTL_vSetEmptyRect( &valid[1] );
            return;
        }
        flags = WVR_ALIGNLEFT | WVR_ALIGNTOP;
    }
    else
    {
        valid[0] = *new_client;
        valid[1] = *old_client;
    }

    /* make sure the rectangles have the same size */
    cx = min( valid[0].right - valid[0].left, valid[1].right - valid[1].left );
    cy = min( valid[0].bottom - valid[0].top, valid[1].bottom - valid[1].top );

    if (flags & WVR_ALIGNBOTTOM)
    {
        valid[0].top = valid[0].bottom - cy;
        valid[1].top = valid[1].bottom - cy;
    }
    else
    {
        valid[0].bottom = valid[0].top + cy;
        valid[1].bottom = valid[1].top + cy;
    }
    if (flags & WVR_ALIGNRIGHT)
    {
        valid[0].left = valid[0].right - cx;
        valid[1].left = valid[1].right - cx;
    }
    else
    {
        valid[0].right = valid[0].left + cx;
        valid[1].right = valid[1].left + cx;
    }
}

static
LONG FASTCALL
co_WinPosDoNCCALCSize(PWND Window, PWINDOWPOS WinPos, RECTL* WindowRect, RECTL* ClientRect, RECTL* validRects)
{
   PWND Parent;
   UINT wvrFlags = 0;

   ASSERT_REFS_CO(Window);

   /* Send WM_NCCALCSIZE message to get new client area */
   if ((WinPos->flags & (SWP_FRAMECHANGED | SWP_NOSIZE)) != SWP_NOSIZE)
   {
      NCCALCSIZE_PARAMS params;
      WINDOWPOS winposCopy;

      params.rgrc[0] = *WindowRect;      // new coordinates of a window that has been moved or resized
      params.rgrc[1] = Window->rcWindow; // window before it was moved or resized
      params.rgrc[2] = Window->rcClient; // client area before the window was moved or resized

      Parent = Window->spwndParent;
      if (0 != (Window->style & WS_CHILD) && Parent)
      {
         RECTL_vOffsetRect(&(params.rgrc[0]), - Parent->rcClient.left, - Parent->rcClient.top);
         RECTL_vOffsetRect(&(params.rgrc[1]), - Parent->rcClient.left, - Parent->rcClient.top);
         RECTL_vOffsetRect(&(params.rgrc[2]), - Parent->rcClient.left, - Parent->rcClient.top);
      }

      params.lppos = &winposCopy;
      winposCopy = *WinPos;

      wvrFlags = co_IntSendMessage(UserHMGetHandle(Window), WM_NCCALCSIZE, TRUE, (LPARAM)&params);

      /* If the application send back garbage, ignore it */
      if (params.rgrc[0].left <= params.rgrc[0].right &&
          params.rgrc[0].top <= params.rgrc[0].bottom)
      {
         *ClientRect = params.rgrc[0]; // First rectangle contains the coordinates of the new client rectangle resulting from the move or resize
         if ((Window->style & WS_CHILD) && Parent)
         {
            RECTL_vOffsetRect(ClientRect, Parent->rcClient.left, Parent->rcClient.top);
         }
         FixClientRect(ClientRect, WindowRect);
      }

      if (ClientRect->left != Window->rcClient.left ||
          ClientRect->top != Window->rcClient.top)
      {
         WinPos->flags &= ~SWP_NOCLIENTMOVE;
      }

      if (ClientRect->right - ClientRect->left != Window->rcClient.right - Window->rcClient.left)
      {
         WinPos->flags &= ~SWP_NOCLIENTSIZE;
      }
      else
         wvrFlags &= ~WVR_HREDRAW;

      if (ClientRect->bottom - ClientRect->top != Window->rcClient.bottom - Window->rcClient.top)
      {
         WinPos->flags &= ~SWP_NOCLIENTSIZE;
      }
      else
         wvrFlags &= ~WVR_VREDRAW;

      validRects[0] = params.rgrc[1]; // second rectangle contains the valid destination rectangle
      validRects[1] = params.rgrc[2]; // third rectangle contains the valid source rectangle
   }
   else
   {
      if (!(WinPos->flags & SWP_NOMOVE) &&
          (ClientRect->left != Window->rcClient.left ||
           ClientRect->top != Window->rcClient.top))
      {
         WinPos->flags &= ~SWP_NOCLIENTMOVE;
      }
   }

   if (WinPos->flags & (SWP_NOCOPYBITS | SWP_NOREDRAW | SWP_SHOWWINDOW | SWP_HIDEWINDOW))
   {
      RECTL_vSetEmptyRect( &validRects[0] );
      RECTL_vSetEmptyRect( &validRects[1] );
   }
   else get_valid_rects( &Window->rcClient, ClientRect, wvrFlags, validRects );

   return wvrFlags;
}

static
BOOL FASTCALL
co_WinPosDoWinPosChanging(PWND Window,
                          PWINDOWPOS WinPos,
                          PRECTL WindowRect,
                          PRECTL ClientRect)
{
   ASSERT_REFS_CO(Window);

   /* Send WM_WINDOWPOSCHANGING message */

   if (!(WinPos->flags & SWP_NOSENDCHANGING)
          && !((WinPos->flags & SWP_AGG_NOCLIENTCHANGE) && (WinPos->flags & SWP_SHOWWINDOW)))
   {
      TRACE("Sending WM_WINDOWPOSCHANGING to hwnd %p flags %04x.\n", UserHMGetHandle(Window), WinPos->flags);
      co_IntSendMessage(UserHMGetHandle(Window), WM_WINDOWPOSCHANGING, 0, (LPARAM)WinPos);
   }

   /* Calculate new position and size */

   *WindowRect = Window->rcWindow;
   *ClientRect = (Window->style & WS_MINIMIZE) ? Window->rcWindow : Window->rcClient;

   if (!(WinPos->flags & SWP_NOSIZE))
   {
      if (Window->style & WS_MINIMIZE)
      {
         WindowRect->right  = WindowRect->left + UserGetSystemMetrics(SM_CXMINIMIZED);
         WindowRect->bottom = WindowRect->top  + UserGetSystemMetrics(SM_CYMINIMIZED);
      }
      else
      {
         WindowRect->right = WindowRect->left + WinPos->cx;
         WindowRect->bottom = WindowRect->top + WinPos->cy;
      }
   }

   if (!(WinPos->flags & SWP_NOMOVE))
   {
      INT X, Y;
      PWND Parent;
      X = WinPos->x;
      Y = WinPos->y;

      Parent = Window->spwndParent;

      // Parent child position issue is in here. SetParent_W7 test CORE-6651.
      if (//((Window->style & WS_CHILD) != 0) && <- Fixes wine msg test_SetParent: "rects do not match", the last test.
           Parent &&
           Parent != Window->head.rpdesk->pDeskInfo->spwnd)
      {
         TRACE("Not SWP_NOMOVE 1 Parent client offset X %d Y %d\n",X,Y);
         X += Parent->rcClient.left;
         Y += Parent->rcClient.top;
         TRACE("Not SWP_NOMOVE 2 Parent client offset X %d Y %d\n",X,Y);
      }

      WindowRect->left    = X;
      WindowRect->top     = Y;
      WindowRect->right  += X - Window->rcWindow.left;
      WindowRect->bottom += Y - Window->rcWindow.top;

      RECTL_vOffsetRect(ClientRect, X - Window->rcWindow.left,
                                    Y - Window->rcWindow.top);
   }
   WinPos->flags |= SWP_NOCLIENTMOVE | SWP_NOCLIENTSIZE;

   TRACE( "hwnd %p, after %p, swp %d,%d %dx%d flags %08x\n",
           WinPos->hwnd, WinPos->hwndInsertAfter, WinPos->x, WinPos->y,
           WinPos->cx, WinPos->cy, WinPos->flags );
   TRACE("WindowRect: %d %d %d %d\n", WindowRect->left,WindowRect->top,WindowRect->right,WindowRect->bottom);
   TRACE("ClientRect: %d %d %d %d\n", ClientRect->left,ClientRect->top,ClientRect->right,ClientRect->bottom);

   return TRUE;
}

/*
 * Fix Z order taking into account owned popups -
 * basically we need to maintain them above the window that owns them
 *
 * FIXME: hide/show owned popups when owner visibility changes.
 *
 * ReactOS: See bug CORE-6129 and CORE-6554.
 *
 */
 ////
 // Pass all the win:test_children/popup_zorder tests except "move hwnd_F and its popups down" which is if'ed out.
 // Side effect, breaks more of the DeferWindowPos api tests, but wine breaks more!!!!
static
HWND FASTCALL
WinPosDoOwnedPopups(PWND Window, HWND hWndInsertAfter)
{
   HWND *List = NULL;
   HWND Owner;
   LONG Style;
   PWND DesktopWindow, ChildObject;
   int i;

   TRACE("(%p) hInsertAfter = %p\n", Window, hWndInsertAfter );

   Style = Window->style;

   if (Style & WS_CHILD)
   {
      TRACE("Window is child\n");
      return hWndInsertAfter;
   }

   Owner = (Window->spwndOwner ? UserHMGetHandle(Window->spwndOwner) : NULL);

   if (Owner)
   {
      /* Make sure this popup stays above the owner */

      if (hWndInsertAfter != HWND_TOPMOST)
      {
         DesktopWindow = UserGetDesktopWindow();
         List = IntWinListChildren(DesktopWindow);

         if (List != NULL)
         {
            for (i = 0; List[i]; i++)
            {
               BOOL topmost = FALSE;

               ChildObject = ValidateHwndNoErr(List[i]);
               if (ChildObject)
               {
                  topmost = (ChildObject->ExStyle & WS_EX_TOPMOST) != 0;
               }

               if (List[i] == Owner)
               {
                  /* We found its Owner, so we must handle it here. */
                  if (i > 0)
                  {
                     if (List[i - 1] != UserHMGetHandle(Window))
                     {
                        /*
                         * If the popup to be inserted is not already just
                         * before the Owner, insert it there. The modified
                         * hWndInsertAfter will be handled below.
                         *
                         * (NOTE: Do not allow hWndInsertAfter to become equal
                         * to the popup's window handle, as this would cause
                         * the popup to link to itself).
                         */
                        hWndInsertAfter = List[i - 1];
                     }
                     else
                     {
                        /* If the popup to be inserted is already
                         * before the Owner, we are done. */
                        ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
                        return hWndInsertAfter;
                     }
                  }
                  else
                  {
                     hWndInsertAfter = topmost ? HWND_TOPMOST : HWND_TOP;
                  }
                  break;
               }

               if (hWndInsertAfter == HWND_TOP || hWndInsertAfter ==  HWND_NOTOPMOST)
               {
                  if (!topmost) break;
               }
               else if (List[i] == hWndInsertAfter) break;
            }
         }
         else
            return hWndInsertAfter;
      }
   }

   if (hWndInsertAfter == HWND_BOTTOM)
   {
      ERR("Window is HWND_BOTTOM hwnd %p\n",hWndInsertAfter);
      if (List) ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
      goto done;
   }

   if (!List)
   {
      DesktopWindow = UserGetDesktopWindow();
      List = IntWinListChildren(DesktopWindow);
   }

   if (List != NULL)
   {
      i = 0;

      if (hWndInsertAfter == HWND_TOP || hWndInsertAfter == HWND_NOTOPMOST)
      {
         if (hWndInsertAfter == HWND_NOTOPMOST || !(Window->ExStyle & WS_EX_TOPMOST))
         {
            TRACE("skip all the topmost windows\n");
            /* skip all the topmost windows */
            while (List[i] &&
                   (ChildObject = ValidateHwndNoErr(List[i])) &&
                   (ChildObject->ExStyle & WS_EX_TOPMOST)) i++;
         }
      }
      else if (hWndInsertAfter != HWND_TOPMOST)
      {
        /* skip windows that are already placed correctly */
        for (i = 0; List[i]; i++)
        {
            if (List[i] == hWndInsertAfter) break;
            if (List[i] == UserHMGetHandle(Window))
            {
               ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
               goto done;  /* nothing to do if window is moving backwards in z-order */
            }
        }
      }

      for (; List[i]; i++)
      {
         PWND Wnd;
         USER_REFERENCE_ENTRY Ref;

         if (List[i] == UserHMGetHandle(Window))
            break;

         if (!(Wnd = ValidateHwndNoErr(List[i])))
            continue;

         Owner = (Wnd->spwndOwner ? UserHMGetHandle(Wnd->spwndOwner) : NULL);

         if (Owner != UserHMGetHandle(Window)) continue;

         UserRefObjectCo(Wnd, &Ref);
         TRACE( "moving %p owned by %p after %p\n", List[i], UserHMGetHandle(Window), hWndInsertAfter );
         co_WinPosSetWindowPos(Wnd, hWndInsertAfter, 0, 0, 0, 0,
                               SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOSENDCHANGING| SWP_DEFERERASE);

         UserDerefObjectCo(Wnd);
         hWndInsertAfter = List[i];
      }
      ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
   }
done:
   return hWndInsertAfter;
}
////

/***********************************************************************
 *      WinPosInternalMoveWindow
 *
 * Update WindowRect and ClientRect of Window and all of its children
 * We keep both WindowRect and ClientRect in screen coordinates internally
 */
static
VOID FASTCALL
WinPosInternalMoveWindow(PWND Window, INT MoveX, INT MoveY)
{
   PWND Child;

   ASSERT(Window != Window->spwndChild);
   TRACE("InternalMoveWin  X %d Y %d\n", MoveX, MoveY);

   Window->rcWindow.left += MoveX;
   Window->rcWindow.right += MoveX;
   Window->rcWindow.top += MoveY;
   Window->rcWindow.bottom += MoveY;

   Window->rcClient.left += MoveX;
   Window->rcClient.right += MoveX;
   Window->rcClient.top += MoveY;
   Window->rcClient.bottom += MoveY;

   for(Child = Window->spwndChild; Child; Child = Child->spwndNext)
   {
      WinPosInternalMoveWindow(Child, MoveX, MoveY);
   }
}

/*
 * WinPosFixupSWPFlags
 *
 * Fix redundant flags and values in the WINDOWPOS structure.
 */
static
BOOL FASTCALL
WinPosFixupFlags(WINDOWPOS *WinPos, PWND Wnd)
{
   PWND Parent;
   POINT pt;

   /* Finally make sure that all coordinates are valid */
   if (WinPos->x < -32768) WinPos->x = -32768;
   else if (WinPos->x > 32767) WinPos->x = 32767;
   if (WinPos->y < -32768) WinPos->y = -32768;
   else if (WinPos->y > 32767) WinPos->y = 32767;

   WinPos->cx = max(WinPos->cx, 0);
   WinPos->cy = max(WinPos->cy, 0);

   Parent = UserGetAncestor( Wnd, GA_PARENT );
   if (!IntIsWindowVisible( Parent ) &&
      /* Fix B : wine msg test_SetParent:WmSetParentSeq_2:25 wParam bits! */
       (WinPos->flags & SWP_AGG_STATUSFLAGS) == SWP_AGG_NOPOSCHANGE) WinPos->flags |= SWP_NOREDRAW;

   if (Wnd->style & WS_VISIBLE) WinPos->flags &= ~SWP_SHOWWINDOW;
   else
   {
      WinPos->flags &= ~SWP_HIDEWINDOW;
      if (!(WinPos->flags & SWP_SHOWWINDOW)) WinPos->flags |= SWP_NOREDRAW;
   }

   /* Check for right size */
   if (Wnd->rcWindow.right - Wnd->rcWindow.left == WinPos->cx &&
       Wnd->rcWindow.bottom - Wnd->rcWindow.top == WinPos->cy)
   {
      WinPos->flags |= SWP_NOSIZE;
   }

   pt.x = WinPos->x;
   pt.y = WinPos->y;
   IntClientToScreen( Parent, &pt );
   TRACE("WPFU C2S wpx %d wpy %d ptx %d pty %d\n",WinPos->x,WinPos->y,pt.x,pt.y);
   /* Check for right position */
   if (Wnd->rcWindow.left == pt.x && Wnd->rcWindow.top == pt.y)
   {
      //ERR("In right pos\n");
      WinPos->flags |= SWP_NOMOVE;
   }

   if ( WinPos->hwnd != UserGetForegroundWindow() && (Wnd->style & (WS_POPUP | WS_CHILD)) != WS_CHILD)
   {
      /* Bring to the top when activating */
      if (!(WinPos->flags & (SWP_NOACTIVATE|SWP_HIDEWINDOW)) &&
           (WinPos->flags & SWP_NOZORDER ||
           (WinPos->hwndInsertAfter != HWND_TOPMOST && WinPos->hwndInsertAfter != HWND_NOTOPMOST)))
      {
         WinPos->flags &= ~SWP_NOZORDER;
         WinPos->hwndInsertAfter = (0 != (Wnd->ExStyle & WS_EX_TOPMOST) ? HWND_TOPMOST : HWND_TOP);
      }
   }

   /* Check hwndInsertAfter */
   if (!(WinPos->flags & SWP_NOZORDER))
   {
      /* Fix sign extension */
      if (WinPos->hwndInsertAfter == (HWND)0xffff)
      {
         WinPos->hwndInsertAfter = HWND_TOPMOST;
      }
      else if (WinPos->hwndInsertAfter == (HWND)0xfffe)
      {
         WinPos->hwndInsertAfter = HWND_NOTOPMOST;
      }

      if (WinPos->hwndInsertAfter == HWND_TOP)
      {
         /* Keep it topmost when it's already topmost */
         if ((Wnd->ExStyle & WS_EX_TOPMOST) != 0)
            WinPos->hwndInsertAfter = HWND_TOPMOST;

         if (IntGetWindow(WinPos->hwnd, GW_HWNDFIRST) == WinPos->hwnd)
         {
            WinPos->flags |= SWP_NOZORDER;
         }
      }
      else if (WinPos->hwndInsertAfter == HWND_BOTTOM)
      {
         if (!(Wnd->ExStyle & WS_EX_TOPMOST) && IntGetWindow(WinPos->hwnd, GW_HWNDLAST) == WinPos->hwnd)
            WinPos->flags |= SWP_NOZORDER;
      }
      else if (WinPos->hwndInsertAfter == HWND_TOPMOST)
      {
          if ((Wnd->ExStyle & WS_EX_TOPMOST) && IntGetWindow(WinPos->hwnd, GW_HWNDFIRST) == WinPos->hwnd)
             WinPos->flags |= SWP_NOZORDER;
      }
      else if (WinPos->hwndInsertAfter == HWND_NOTOPMOST)
      {
         if (!(Wnd->ExStyle & WS_EX_TOPMOST))
            WinPos->flags |= SWP_NOZORDER;
      }
      else /* hwndInsertAfter must be a sibling of the window */
      {
         PWND InsAfterWnd;

         InsAfterWnd = ValidateHwndNoErr(WinPos->hwndInsertAfter);
         if(!InsAfterWnd)
         {
             return TRUE;
         }

         if (InsAfterWnd->spwndParent != Wnd->spwndParent)
         {
            /* Note from wine User32 Win test_SetWindowPos:
               "Returns TRUE also for windows that are not siblings"
               "Does not seem to do anything even without passing flags, still returns TRUE"
               "Same thing the other way around."
               ".. and with these windows."
             */
            return FALSE;
         }
         else
         {
            /*
             * We don't need to change the Z order of hwnd if it's already
             * inserted after hwndInsertAfter or when inserting hwnd after
             * itself.
             */
            if ((WinPos->hwnd == WinPos->hwndInsertAfter) ||
                ((InsAfterWnd->spwndNext) && (WinPos->hwnd == UserHMGetHandle(InsAfterWnd->spwndNext))))
            {
               WinPos->flags |= SWP_NOZORDER;
            }
         }
      }
   }

   return TRUE;
}

//
// This is a NC HACK fix for forcing painting of non client areas.
// Further troubleshooting in painting.c is required to remove this hack.
// See CORE-7166 & CORE-15934
//
VOID
ForceNCPaintErase(PWND Wnd, HRGN hRgn, PREGION pRgn)
{
   HDC hDC;
   PREGION RgnUpdate;
   UINT RgnType;
   BOOL Create = FALSE;

   if (Wnd->hrgnUpdate == NULL)
   {
       Wnd->hrgnUpdate = NtGdiCreateRectRgn(0, 0, 0, 0);
       IntGdiSetRegionOwner(Wnd->hrgnUpdate, GDI_OBJ_HMGR_PUBLIC);
       Create = TRUE;
   }

   if (Wnd->hrgnUpdate != HRGN_WINDOW)
   {
       RgnUpdate = REGION_LockRgn(Wnd->hrgnUpdate);
       if (RgnUpdate)
       {
           RgnType = IntGdiCombineRgn(RgnUpdate, RgnUpdate, pRgn, RGN_OR);
           REGION_UnlockRgn(RgnUpdate);
           if (RgnType == NULLREGION)
           {
               IntGdiSetRegionOwner(Wnd->hrgnUpdate, GDI_OBJ_HMGR_POWNED);
               GreDeleteObject(Wnd->hrgnUpdate);
               Wnd->hrgnUpdate = NULL;
               Create = FALSE;
           }
       }
   }

   IntSendNCPaint( Wnd, hRgn ); // Region can be deleted by the application.

   if (Wnd->hrgnUpdate)
   {
       hDC = UserGetDCEx( Wnd,
                          Wnd->hrgnUpdate,
                          DCX_CACHE|DCX_USESTYLE|DCX_INTERSECTRGN|DCX_KEEPCLIPRGN);

      Wnd->state &= ~(WNDS_SENDERASEBACKGROUND|WNDS_ERASEBACKGROUND);
      // Kill the loop, so Clear before we send.
      if (!co_IntSendMessage(UserHMGetHandle(Wnd), WM_ERASEBKGND, (WPARAM)hDC, 0))
      {
          Wnd->state |= (WNDS_SENDERASEBACKGROUND|WNDS_ERASEBACKGROUND);
      }
      UserReleaseDC(Wnd, hDC, FALSE);
   }

   if (Create)
   {
      IntGdiSetRegionOwner(Wnd->hrgnUpdate, GDI_OBJ_HMGR_POWNED);
      GreDeleteObject(Wnd->hrgnUpdate);
      Wnd->hrgnUpdate = NULL;
   }
}

static VOID FASTCALL IntImeWindowPosChanged(VOID)
{
    HWND *phwnd;
    PWND pwndNode, pwndDesktop = UserGetDesktopWindow();
    PWINDOWLIST pWL;
    USER_REFERENCE_ENTRY Ref;

    if (!pwndDesktop)
        return;

    /* Enumerate the windows to get the IME windows (of default and non-default) */
    pWL = IntBuildHwndList(pwndDesktop->spwndChild, IACE_LIST, gptiCurrent);
    if (!pWL)
        return;

    for (phwnd = pWL->ahwnd; *phwnd != HWND_TERMINATOR; ++phwnd)
    {
        if (gptiCurrent->TIF_flags & TIF_INCLEANUP)
            break;

        pwndNode = ValidateHwndNoErr(*phwnd);
        if (pwndNode == NULL ||
            pwndNode->head.pti != gptiCurrent ||
            pwndNode->pcls->atomClassName != gpsi->atomSysClass[ICLS_IME])
        {
            continue;
        }

        /* Now hwndNode is an IME window of the current thread */
        UserRefObjectCo(pwndNode, &Ref);
        co_IntSendMessage(*phwnd, WM_IME_SYSTEM, IMS_UPDATEIMEUI, 0);
        UserDerefObjectCo(pwndNode);
    }

    IntFreeHwndList(pWL);
}

/* x and y are always screen relative */
BOOLEAN FASTCALL
co_WinPosSetWindowPos(
   PWND Window,
   HWND WndInsertAfter,
   INT x,
   INT y,
   INT cx,
   INT cy,
   UINT flags
   )
{
   WINDOWPOS WinPos;
   RECTL NewWindowRect;
   RECTL NewClientRect;
   RECTL valid_rects[2];
   PREGION VisBefore = NULL;
   PREGION VisBeforeJustClient = NULL;
   PREGION VisAfter = NULL;
   PREGION CopyRgn = NULL;
   ULONG WvrFlags = 0;
   RECTL OldWindowRect, OldClientRect;
   int RgnType;
   HDC Dc;
   RECTL CopyRect;
   PWND Ancestor;
   BOOL bPointerInWindow, PosChanged = FALSE;
   PTHREADINFO pti = PsGetCurrentThreadWin32Thread();

   ASSERT_REFS_CO(Window);

   TRACE("pwnd %p, after %p, %d,%d (%dx%d), flags 0x%x\n",
         Window, WndInsertAfter, x, y, cx, cy, flags);
#if DBG
   dump_winpos_flags(flags);
#endif

   /* FIXME: Get current active window from active queue. Why? since r2915. */

   bPointerInWindow = IntPtInWindow(Window, gpsi->ptCursor.x, gpsi->ptCursor.y);

   WinPos.hwnd = UserHMGetHandle(Window);
   WinPos.hwndInsertAfter = WndInsertAfter;
   WinPos.x = x;
   WinPos.y = y;
   WinPos.cx = cx;
   WinPos.cy = cy;
   WinPos.flags = flags;

   if ( flags & SWP_ASYNCWINDOWPOS )
   {
      LRESULT lRes;
      PWINDOWPOS ppos = ExAllocatePoolWithTag(PagedPool, sizeof(WINDOWPOS), USERTAG_SWP);
      if ( ppos )
      {
         WinPos.flags &= ~SWP_ASYNCWINDOWPOS; // Clear flag.
         *ppos = WinPos;
         /* Yes it's a pointer inside Win32k! */
         lRes = co_IntSendMessageNoWait( WinPos.hwnd, WM_ASYNC_SETWINDOWPOS, 0, (LPARAM)ppos);
         /* We handle this the same way as Event Hooks and Hooks. */
         if ( !lRes )
         {
            ExFreePoolWithTag(ppos, USERTAG_SWP);
            return FALSE;
         }
         return TRUE;
      }
      return FALSE;
   }

   co_WinPosDoWinPosChanging(Window, &WinPos, &NewWindowRect, &NewClientRect);

   /* Does the window still exist? */
   if (!IntIsWindow(WinPos.hwnd))
   {
      TRACE("WinPosSetWindowPos: Invalid handle 0x%p!\n",WinPos.hwnd);
      EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
      return FALSE;
   }

   /* Fix up the flags. */
   if (!WinPosFixupFlags(&WinPos, Window))
   {
      // See Note.
      return TRUE;
   }

   Ancestor = UserGetAncestor(Window, GA_PARENT);
   if ( (WinPos.flags & (SWP_NOZORDER | SWP_HIDEWINDOW | SWP_SHOWWINDOW)) != SWP_NOZORDER &&
         Ancestor && UserHMGetHandle(Ancestor) == IntGetDesktopWindow() )
   {
      WinPos.hwndInsertAfter = WinPosDoOwnedPopups(Window, WinPos.hwndInsertAfter);
   }

   if (!(WinPos.flags & SWP_NOREDRAW))
   {
      /* Compute the visible region before the window position is changed */
      if (!(WinPos.flags & SWP_SHOWWINDOW) &&
           (WinPos.flags & (SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER |
                             SWP_HIDEWINDOW | SWP_FRAMECHANGED)) !=
            (SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER))
      {
         VisBefore = VIS_ComputeVisibleRegion(Window, FALSE, FALSE,
                                              (Window->style & WS_CLIPSIBLINGS) ? TRUE : FALSE);

         if ( VisBefore != NULL &&
              REGION_Complexity(VisBefore) == NULLREGION )
         {
            REGION_Delete(VisBefore);
            VisBefore = NULL;
         }
         else if(VisBefore)
         {
            REGION_bOffsetRgn(VisBefore, -Window->rcWindow.left, -Window->rcWindow.top);
         }

         /* Calculate the non client area for resizes, as this is used in the copy region */
         if ((WinPos.flags & (SWP_NOSIZE | SWP_FRAMECHANGED)) != SWP_NOSIZE)
         {
             VisBeforeJustClient = VIS_ComputeVisibleRegion(Window, TRUE, FALSE,
                 (Window->style & WS_CLIPSIBLINGS) ? TRUE : FALSE);

             if ( VisBeforeJustClient != NULL &&
                 REGION_Complexity(VisBeforeJustClient) == NULLREGION )
             {
                 REGION_Delete(VisBeforeJustClient);
                 VisBeforeJustClient = NULL;
             }
             else if(VisBeforeJustClient)
             {
                 REGION_bOffsetRgn(VisBeforeJustClient, -Window->rcWindow.left, -Window->rcWindow.top);
             }
         }
      }
   }

   //// HACK 3
   if (Window->hrgnNewFrame)
   {
       SelectWindowRgn( Window, Window->hrgnNewFrame ); // Should be PSMWP->acvr->hrgnClip
       Window->hrgnNewFrame = NULL;
   }

   WvrFlags = co_WinPosDoNCCALCSize(Window, &WinPos, &NewWindowRect, &NewClientRect, valid_rects);

//   ERR("co_WinPosDoNCCALCSize returned 0x%x\n valid dest: %d %d %d %d\n valid src : %d %d %d %d\n", WvrFlags,
//      valid_rects[0].left,valid_rects[0].top,valid_rects[0].right,valid_rects[0].bottom,
//      valid_rects[1].left,valid_rects[1].top,valid_rects[1].right,valid_rects[1].bottom);

   /* Validate link windows. (also take into account shell window in hwndShellWindow) */
   if (!(WinPos.flags & SWP_NOZORDER) && WinPos.hwnd != UserGetShellWindow())
   {
      IntLinkHwnd(Window, WinPos.hwndInsertAfter);
   }

   OldWindowRect = Window->rcWindow;
   OldClientRect = Window->rcClient;

   if (NewClientRect.left != OldClientRect.left ||
       NewClientRect.top  != OldClientRect.top)
   {
      // Move child window if their parent is moved. Keep Child window relative to Parent...
      WinPosInternalMoveWindow(Window,
                               NewClientRect.left - OldClientRect.left,
                               NewClientRect.top - OldClientRect.top);
      PosChanged = TRUE;
   }

   Window->rcWindow = NewWindowRect;
   Window->rcClient = NewClientRect;

   /* erase parent when hiding or resizing child */
   if (WinPos.flags & SWP_HIDEWINDOW)
   {
      /* Clear the update region */
      co_UserRedrawWindow( Window,
                           NULL,
                           0,
                           RDW_VALIDATE | RDW_NOFRAME | RDW_NOERASE | RDW_NOINTERNALPAINT | RDW_ALLCHILDREN);

      if (UserIsDesktopWindow(Window->spwndParent))
         co_IntShellHookNotify(HSHELL_WINDOWDESTROYED, (WPARAM)UserHMGetHandle(Window), 0);

      Window->style &= ~WS_VISIBLE; //IntSetStyle( Window, 0, WS_VISIBLE );
      Window->head.pti->cVisWindows--;
      IntNotifyWinEvent(EVENT_OBJECT_HIDE, Window, OBJID_WINDOW, CHILDID_SELF, WEF_SETBYWNDPTI);
   }
   else if (WinPos.flags & SWP_SHOWWINDOW)
   {
      if (Window->style & WS_CHILD)
      {
         if ((Window->style & WS_POPUP) && (Window->ExStyle & WS_EX_APPWINDOW))
         {
            co_IntShellHookNotify(HSHELL_WINDOWCREATED, (WPARAM)UserHMGetHandle(Window), 0);
            if (!(WinPos.flags & SWP_NOACTIVATE))
               UpdateShellHook(Window);
         }
      }
      else if ((Window->ExStyle & WS_EX_APPWINDOW) ||
          (!(Window->ExStyle & WS_EX_TOOLWINDOW) && !Window->spwndOwner &&
           (!Window->spwndParent || UserIsDesktopWindow(Window->spwndParent))))
      {
         if (!UserIsDesktopWindow(Window))
         {
            co_IntShellHookNotify(HSHELL_WINDOWCREATED, (WPARAM)UserHMGetHandle(Window), 0);
            if (!(WinPos.flags & SWP_NOACTIVATE))
               UpdateShellHook(Window);
         }
      }

      Window->style |= WS_VISIBLE; //IntSetStyle( Window, WS_VISIBLE, 0 );
      Window->head.pti->cVisWindows++;
      IntNotifyWinEvent(EVENT_OBJECT_SHOW, Window, OBJID_WINDOW, CHILDID_SELF, WEF_SETBYWNDPTI);
   }
   else
   {
      IntCheckFullscreen(Window);
   }

   if (Window->hrgnUpdate != NULL && Window->hrgnUpdate != HRGN_WINDOW)
   {
      NtGdiOffsetRgn(Window->hrgnUpdate,
                     NewWindowRect.left - OldWindowRect.left,
                     NewWindowRect.top - OldWindowRect.top);
   }

   DceResetActiveDCEs(Window); // For WS_VISIBLE changes.

   // Change or update, set send non-client paint flag.
   if ( Window->style & WS_VISIBLE &&
       (WinPos.flags & SWP_STATECHANGED || (!(Window->state2 & WNDS2_WIN31COMPAT) && WinPos.flags & SWP_NOREDRAW ) ) )
   {
      TRACE("Set WNDS_SENDNCPAINT %p\n",Window);
      Window->state |= WNDS_SENDNCPAINT;
   }

   if ((!(WinPos.flags & SWP_NOREDRAW) && ((WinPos.flags & SWP_AGG_STATUSFLAGS) != SWP_AGG_NOPOSCHANGE)) ||
       ((WinPos.flags & SWP_NOZORDER) && (WinPos.flags & SWP_NOOWNERZORDER)))
   {
      /* Determine the new visible region */
      VisAfter = VIS_ComputeVisibleRegion(Window, FALSE, FALSE,
                                          (Window->style & WS_CLIPSIBLINGS) ? TRUE : FALSE);

      if ( VisAfter != NULL &&
           REGION_Complexity(VisAfter) == NULLREGION )
      {
         REGION_Delete(VisAfter);
         VisAfter = NULL;
      }
      else if(VisAfter)
      {
          /* Clip existing update region to new window size */
          if (Window->hrgnUpdate != NULL)
          {
              PREGION RgnUpdate = REGION_LockRgn(Window->hrgnUpdate);
              if (RgnUpdate)
              {
                  RgnType = IntGdiCombineRgn(RgnUpdate, RgnUpdate, VisAfter, RGN_AND);
                  REGION_UnlockRgn(RgnUpdate);
              }
          }
          REGION_bOffsetRgn(VisAfter, -Window->rcWindow.left, -Window->rcWindow.top);
      }

      /*
       * Determine which pixels can be copied from the old window position
       * to the new. Those pixels must be visible in both the old and new
       * position. Also, check the class style to see if the windows of this
       * class need to be completely repainted on (horizontal/vertical) size
       * change.
       */
      if (VisBefore != NULL &&
          VisAfter != NULL &&
          !(WvrFlags & WVR_REDRAW) &&
          !(WinPos.flags & SWP_NOCOPYBITS) &&
          !(Window->ExStyle & WS_EX_TRANSPARENT))
      {

         /*
          * If this is (also) a window resize, the whole nonclient area
          * needs to be repainted. So we limit the copy to the client area,
          * 'cause there is no use in copying it (would possibly cause
          * "flashing" too). However, if the copy region is already empty,
          * we don't have to crop (can't take anything away from an empty
          * region...)
          */

         CopyRgn = IntSysCreateRectpRgn(0, 0, 0, 0);
         if ((WinPos.flags & SWP_NOSIZE) && (WinPos.flags & SWP_NOCLIENTSIZE))
            RgnType = IntGdiCombineRgn(CopyRgn, VisAfter, VisBefore, RGN_AND);
         else if (VisBeforeJustClient != NULL)
         {
            RgnType = IntGdiCombineRgn(CopyRgn, VisAfter, VisBeforeJustClient, RGN_AND);
         }

         /* Now use in copying bits which are in the update region. */
         if (Window->hrgnUpdate != NULL)
         {
            PREGION RgnUpdate = REGION_LockRgn(Window->hrgnUpdate);
            if (RgnUpdate)
            {
                REGION_bOffsetRgn(CopyRgn, NewWindowRect.left, NewWindowRect.top);
                IntGdiCombineRgn(CopyRgn, CopyRgn, RgnUpdate, RGN_DIFF);
                REGION_bOffsetRgn(CopyRgn, -NewWindowRect.left, -NewWindowRect.top);
                REGION_UnlockRgn(RgnUpdate);
            }
         }

         /*
          * Now, get the bounding box of the copy region. If it's empty
          * there's nothing to copy. Also, it's no use copying bits onto
          * themselves.
          */
         if (REGION_GetRgnBox(CopyRgn, &CopyRect) == NULLREGION)
         {
            /* Nothing to copy, clean up */
            REGION_Delete(CopyRgn);
            CopyRgn = NULL;
         }
         else if ( OldWindowRect.left != NewWindowRect.left ||
                   OldWindowRect.top != NewWindowRect.top ||
                  (WinPos.flags & SWP_FRAMECHANGED) )
         {
             HRGN DcRgn = NtGdiCreateRectRgn(0, 0, 0, 0);
             PREGION DcRgnObj = REGION_LockRgn(DcRgn);

          /*
           * Small trick here: there is no function to bitblt a region. So
           * we set the region as the clipping region, take the bounding box
           * of the region and bitblt that. Since nothing outside the clipping
           * region is copied, this has the effect of bitblt'ing the region.
           *
           * Since NtUserGetDCEx takes ownership of the clip region, we need
           * to create a copy of CopyRgn and pass that. We need CopyRgn later
           */
            IntGdiCombineRgn(DcRgnObj, CopyRgn, NULL, RGN_COPY);
            REGION_bOffsetRgn(DcRgnObj, NewWindowRect.left, NewWindowRect.top);
            REGION_UnlockRgn(DcRgnObj);
            Dc = UserGetDCEx( Window,
                              DcRgn,
                              DCX_WINDOW|DCX_CACHE|DCX_INTERSECTRGN|DCX_CLIPSIBLINGS|DCX_KEEPCLIPRGN); // DCX_WINDOW will set first, go read WinDC.c.
            NtGdiBitBlt( Dc,
                         CopyRect.left, CopyRect.top,
                         CopyRect.right - CopyRect.left,
                         CopyRect.bottom - CopyRect.top,
                         Dc,
                         CopyRect.left + (OldWindowRect.left - NewWindowRect.left),
                         CopyRect.top + (OldWindowRect.top - NewWindowRect.top),
                         SRCCOPY,
                         CLR_INVALID,
                         0);

            UserReleaseDC(Window, Dc, FALSE);
            IntValidateParent(Window, CopyRgn);
            GreDeleteObject(DcRgn);
         }
      }
      else
      {
         CopyRgn = NULL;
      }

      /* We need to redraw what wasn't visible before or force a redraw */
      if (VisAfter != NULL)
      {
         PREGION DirtyRgn = IntSysCreateRectpRgn(0, 0, 0, 0);
         if (DirtyRgn)
         {
             if (CopyRgn != NULL)
             {
                RgnType = IntGdiCombineRgn(DirtyRgn, VisAfter, CopyRgn, RGN_DIFF);
             }
             else
             {
                RgnType = IntGdiCombineRgn(DirtyRgn, VisAfter, 0, RGN_COPY);
             }

             if (RgnType != ERROR && RgnType != NULLREGION) // Regions moved.
             {
            /* old code
                NtGdiOffsetRgn(DirtyRgn, Window->rcWindow.left, Window->rcWindow.top);
                IntInvalidateWindows( Window,
                                      DirtyRgn,
                   RDW_ERASE | RDW_FRAME | RDW_INVALIDATE | RDW_ALLCHILDREN);
             }
             GreDeleteObject(DirtyRgn);
             */

                PWND Parent = Window->spwndParent;

                REGION_bOffsetRgn( DirtyRgn, Window->rcWindow.left, Window->rcWindow.top);

                if ( (Window->style & WS_CHILD) && (Parent) && !(Parent->style & WS_CLIPCHILDREN))
                {
                   IntInvalidateWindows( Parent, DirtyRgn, RDW_ERASE | RDW_INVALIDATE);
                   co_IntPaintWindows(Parent, RDW_NOCHILDREN, FALSE);
                }
                IntInvalidateWindows(Window, DirtyRgn, RDW_ERASE | RDW_FRAME | RDW_INVALIDATE | RDW_ALLCHILDREN);
             }
             else if (RgnType != ERROR && RgnType == NULLREGION) // Must be the same. See CORE-7166 & CORE-15934, NC HACK fix.
             {
                if ( !PosChanged &&
                     !(WinPos.flags & SWP_DEFERERASE) &&
                      (WinPos.flags & SWP_FRAMECHANGED) )
                {
                    PWND pwnd = Window;
                    PWND Parent = Window->spwndParent;

                    if ( pwnd->style & WS_CHILD ) // Fix ProgMan menu bar drawing.
                    {
                        TRACE("SWP_FRAMECHANGED win child %p Parent %p\n",pwnd,Parent);
                        pwnd = Parent ? Parent : pwnd;
                    }

                    if ( !(pwnd->style & WS_CHILD) )
                    {
                        /*
                         * Check if we have these specific windows style bits set/reset.
                         * FIXME: There may be other combinations of styles that need this handling as well.
                         * This fixes the ReactOS Calculator buttons disappearing in CORE-16827.
                         */
                        if ((Window->style & WS_CLIPSIBLINGS) && !(Window->style & (WS_POPUP | WS_CLIPCHILDREN | WS_SIZEBOX)))
                        {
                            IntSendNCPaint(pwnd, HRGN_WINDOW); // Paint the whole frame.
                        }
                        else  // Use region handling
                        {
                            HRGN DcRgn = NtGdiCreateRectRgn(0, 0, 0, 0);
                            PREGION DcRgnObj = REGION_LockRgn(DcRgn);
                            TRACE("SWP_FRAMECHANGED win %p hRgn %p\n",pwnd, DcRgn);
                            IntGdiCombineRgn(DcRgnObj, VisBefore, NULL, RGN_COPY);
                            REGION_UnlockRgn(DcRgnObj);
                            ForceNCPaintErase(pwnd, DcRgn, DcRgnObj);
                            GreDeleteObject(DcRgn);
                        }
                    }
                }
             }
             REGION_Delete(DirtyRgn);
         }
      }

      /* Expose what was covered before but not covered anymore */
      if (VisBefore != NULL)
      {
         PREGION ExposedRgn = IntSysCreateRectpRgn(0, 0, 0, 0);
         if (ExposedRgn)
         {
             RgnType = IntGdiCombineRgn(ExposedRgn, VisBefore, NULL, RGN_COPY);
             REGION_bOffsetRgn(ExposedRgn,
                               OldWindowRect.left - NewWindowRect.left,
                               OldWindowRect.top  - NewWindowRect.top);

             if (VisAfter != NULL)
                RgnType = IntGdiCombineRgn(ExposedRgn, ExposedRgn, VisAfter, RGN_DIFF);

             if (RgnType != ERROR && RgnType != NULLREGION)
             {
                co_VIS_WindowLayoutChanged(Window, ExposedRgn);
             }
             REGION_Delete(ExposedRgn);
         }
      }
   }

   if (!(WinPos.flags & (SWP_NOACTIVATE|SWP_HIDEWINDOW)))
   {
      if ((Window->style & (WS_CHILD | WS_POPUP)) == WS_CHILD)
      {
         co_IntSendMessageNoWait(WinPos.hwnd, WM_CHILDACTIVATE, 0, 0);
      }
      else
      {
         //TRACE("Activate hwnd=%p\n", WinPos.hwnd);
         if ( pti->MessageQueue->spwndActive != Window ||
              pti->MessageQueue != gpqForeground )
         {
            if (!(Window->state & WNDS_BEINGACTIVATED)) // Inside SAW?
            {
               co_IntSetForegroundWindow(Window);
            }
         }
      }
   }
   else
   {
      TRACE("Activation skip hwnd=%p flags=0x%x\n", WinPos.hwnd, WinPos.flags);
   }

   if ( !PosChanged &&
         (WinPos.flags & SWP_FRAMECHANGED) &&
        !(WinPos.flags & SWP_DEFERERASE) &&    // Prevent sending WM_SYNCPAINT message. 
         VisAfter )
   {
       PWND Parent = Window->spwndParent;
       if ( !(Window->style & WS_CHILD) && (Parent) && (Parent->style & WS_CLIPCHILDREN))
       {
           TRACE("SWP_FRAMECHANGED Parent %p WS_CLIPCHILDREN %p\n",Parent,Window);
           UserSyncAndPaintWindows(Parent, RDW_CLIPCHILDREN); // NC should redraw here, see NC HACK fix.
       }
   }

   // Fix wine msg test_SetFocus, prevents sending WM_WINDOWPOSCHANGED.
   if ( VisBefore == NULL &&
        VisBeforeJustClient == NULL &&
       !(Window->ExStyle & WS_EX_TOPMOST) &&
        (WinPos.flags & SWP_AGG_STATUSFLAGS) == (SWP_AGG_NOPOSCHANGE & ~SWP_NOZORDER))
   {
      TRACE("No drawing, set no Z order and no redraw!\n");
      WinPos.flags |= SWP_NOZORDER|SWP_NOREDRAW;
   }

   if (VisBefore != NULL)
   {
      REGION_Delete(VisBefore);
      VisBefore = NULL;
   }
   if (VisBeforeJustClient != NULL)
   {
      REGION_Delete(VisBeforeJustClient);
      VisBeforeJustClient = NULL;
   }
   if (VisAfter != NULL)
   {
      REGION_Delete(VisAfter);
      VisAfter = NULL;
   }
   if (CopyRgn != NULL)
   {
      REGION_Delete(CopyRgn);
      CopyRgn = NULL;
   }

   if(!(flags & SWP_DEFERERASE))
   {
       /* erase parent when hiding or resizing child */
       if ((flags & SWP_HIDEWINDOW) ||
         (!(flags & SWP_SHOWWINDOW) &&
          (WinPos.flags & SWP_AGG_STATUSFLAGS) != SWP_AGG_NOGEOMETRYCHANGE))
       {
           PWND Parent = Window->spwndParent;
           if (!Parent || UserIsDesktopWindow(Parent)) Parent = Window;
           UserSyncAndPaintWindows( Parent, RDW_ERASENOW);
       }

       /* Give newly shown windows a chance to redraw */
       if(((WinPos.flags & SWP_AGG_STATUSFLAGS) != SWP_AGG_NOPOSCHANGE)
                && !(flags & SWP_AGG_NOCLIENTCHANGE) && (flags & SWP_SHOWWINDOW))
       {
           UserSyncAndPaintWindows( Window, RDW_ERASENOW);
       }
   }

   /* And last, send the WM_WINDOWPOSCHANGED message */

   TRACE("\tstatus hwnd %p flags = %04x\n", Window ? UserHMGetHandle(Window) : NULL, WinPos.flags & SWP_AGG_STATUSFLAGS);

   if (((WinPos.flags & SWP_AGG_STATUSFLAGS) != SWP_AGG_NOPOSCHANGE)
            && !((flags & SWP_AGG_NOCLIENTCHANGE) && (flags & SWP_SHOWWINDOW)))
   {
      /* WM_WINDOWPOSCHANGED is sent even if SWP_NOSENDCHANGING is set
         and always contains final window position.
       */
      WinPos.x = NewWindowRect.left;
      WinPos.y = NewWindowRect.top;
      WinPos.cx = NewWindowRect.right - NewWindowRect.left;
      WinPos.cy = NewWindowRect.bottom - NewWindowRect.top;
      TRACE("WM_WINDOWPOSCHANGED hwnd %p Flags %04x\n",WinPos.hwnd,WinPos.flags);
      co_IntSendMessageNoWait(WinPos.hwnd, WM_WINDOWPOSCHANGED, 0, (LPARAM) &WinPos);
   }

   if ( WinPos.flags & SWP_FRAMECHANGED  || WinPos.flags & SWP_STATECHANGED ||
      !(WinPos.flags & SWP_NOCLIENTSIZE) || !(WinPos.flags & SWP_NOCLIENTMOVE) )
   {
      PWND pWnd = ValidateHwndNoErr(WinPos.hwnd);
      if (pWnd)
         IntNotifyWinEvent(EVENT_OBJECT_LOCATIONCHANGE, pWnd, OBJID_WINDOW, CHILDID_SELF, WEF_SETBYWNDPTI);
   }

   /* Send WM_IME_SYSTEM:IMS_UPDATEIMEUI to the IME windows if necessary */
   if ((WinPos.flags & (SWP_NOMOVE | SWP_NOSIZE)) != (SWP_NOMOVE | SWP_NOSIZE))
   {
      if (IS_IMM_MODE())
          IntImeWindowPosChanged();
   }

   if(bPointerInWindow != IntPtInWindow(Window, gpsi->ptCursor.x, gpsi->ptCursor.y))
   {
      /* Generate mouse move message */
      MSG msg;
      msg.message = WM_MOUSEMOVE;
      msg.wParam = UserGetMouseButtonsState();
      msg.lParam = MAKELPARAM(gpsi->ptCursor.x, gpsi->ptCursor.y);
      msg.pt = gpsi->ptCursor;
      co_MsqInsertMouseMessage(&msg, 0, 0, TRUE);
   }

   return TRUE;
}