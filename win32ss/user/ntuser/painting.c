/*
 *  COPYRIGHT:        See COPYING in the top level directory
 *  PROJECT:          ReactOS Win32k subsystem
 *  PURPOSE:          Window painting function
 *  FILE:             win32ss/user/ntuser/painting.c
 *  PROGRAMER:        Filip Navara (xnavara@volny.cz)
 */

#include <win32k.h>
#include <safe_user_io.h>
DBG_DEFAULT_CHANNEL(UserPainting);

BOOL UserExtTextOutW(HDC hdc, INT x, INT y, UINT flags, PRECTL lprc,
                     LPCWSTR lpString, UINT count);

/* PRIVATE FUNCTIONS **********************************************************/

/**
 * @name IntIntersectWithParents
 *
 * Intersect window rectangle with all parent client rectangles.
 *
 * @param Child
 *        Pointer to child window to start intersecting from.
 * @param WindowRect
 *        Pointer to rectangle that we want to intersect in screen
 *        coordinates on input and intersected rectangle on output (if TRUE
 *        is returned).
 *
 * @return
 *    If any parent is minimized or invisible or the resulting rectangle
 *    is empty then FALSE is returned. Otherwise TRUE is returned.
 */

BOOL FASTCALL
IntIntersectWithParents(PWND Child, RECTL *WindowRect)
{
   PWND ParentWnd;

   if (Child->ExStyle & WS_EX_REDIRECTED)
      return TRUE;

   ParentWnd = Child->spwndParent;
   while (ParentWnd != NULL)
   {
      if (!(ParentWnd->style & WS_VISIBLE)  ||
           (ParentWnd->style & WS_MINIMIZE) ||
          !RECTL_bIntersectRect(WindowRect, WindowRect, &ParentWnd->rcClient) )
      {
         return FALSE;
      }

      if (ParentWnd->ExStyle & WS_EX_REDIRECTED)
         return TRUE;

      ParentWnd = ParentWnd->spwndParent;
   }

   return TRUE;
}

BOOL FASTCALL
IntValidateParents(PWND Child, BOOL Recurse)
{
   RECTL ParentRect, Rect;
   BOOL Start, Ret = TRUE;
   PWND ParentWnd = Child;
   PREGION Rgn = NULL;

   if (ParentWnd->style & WS_CHILD)
   {
      do
         ParentWnd = ParentWnd->spwndParent;
      while (ParentWnd->style & WS_CHILD);
   }

   // No pending nonclient paints.
   if (!(ParentWnd->state & WNDS_SYNCPAINTPENDING)) Recurse = FALSE;

   Start = TRUE;
   ParentWnd = Child->spwndParent;
   while (ParentWnd)
   {
      if (ParentWnd->style & WS_CLIPCHILDREN)
         break;

      if (ParentWnd->hrgnUpdate != 0)
      {
         if (Recurse)
         {
            Ret = FALSE;
            break;
         }
         // Start with child clipping.
         if (Start)
         {
            Start = FALSE;

            Rect = Child->rcWindow;

            if (!IntIntersectWithParents(Child, &Rect)) break;

            Rgn = IntSysCreateRectpRgnIndirect(&Rect);

            if (Child->hrgnClip)
            {
               PREGION RgnClip = REGION_LockRgn(Child->hrgnClip);
               IntGdiCombineRgn(Rgn, Rgn, RgnClip, RGN_AND);
               REGION_UnlockRgn(RgnClip);
            }
         }

         ParentRect = ParentWnd->rcWindow;

         if (!IntIntersectWithParents(ParentWnd, &ParentRect)) break;

         IntInvalidateWindows( ParentWnd,
                               Rgn,
                               RDW_VALIDATE | RDW_NOCHILDREN | RDW_NOUPDATEDIRTY);
      }
      ParentWnd = ParentWnd->spwndParent;
   }

   if (Rgn) REGION_Delete(Rgn);

   return Ret;
}

/*
  Synchronize painting to the top-level windows of other threads.
*/
VOID FASTCALL
IntSendSyncPaint(PWND Wnd, ULONG Flags)
{
   PTHREADINFO ptiCur, ptiWnd;
   PUSER_SENT_MESSAGE Message;
   PLIST_ENTRY Entry;
   BOOL bSend = TRUE;

   ptiWnd = Wnd->head.pti;
   ptiCur = PsGetCurrentThreadWin32Thread();
   /*
      Not the current thread, Wnd is in send Nonclient paint also in send erase background and it is visiable.
   */
   if ( Wnd->head.pti != ptiCur &&
        Wnd->state & WNDS_SENDNCPAINT &&
        Wnd->state & WNDS_SENDERASEBACKGROUND &&
        Wnd->style & WS_VISIBLE)
   {
      // For testing, if you see this, break out the Champagne and have a party!
      TRACE("SendSyncPaint Wnd in State!\n");
      if (!IsListEmpty(&ptiWnd->SentMessagesListHead))
      {
         // Scan sent queue messages to see if we received sync paint messages.
         Entry = ptiWnd->SentMessagesListHead.Flink;
         Message = CONTAINING_RECORD(Entry, USER_SENT_MESSAGE, ListEntry);
         do
         {
            TRACE("LOOP it\n");
            if (Message->Msg.message == WM_SYNCPAINT &&
                Message->Msg.hwnd == UserHMGetHandle(Wnd))
            {  // Already received so exit out.
                ERR("SendSyncPaint Found one in the Sent Msg Queue!\n");
                bSend = FALSE;
                break;
            }
            Entry = Message->ListEntry.Flink;
            Message = CONTAINING_RECORD(Entry, USER_SENT_MESSAGE, ListEntry);
         }
         while (Entry != &ptiWnd->SentMessagesListHead);
      }
      if (bSend)
      {
         TRACE("Sending WM_SYNCPAINT\n");
         // This message has no parameters. But it does! Pass Flags along.
         co_IntSendMessageNoWait(UserHMGetHandle(Wnd), WM_SYNCPAINT, Flags, 0);
         Wnd->state |= WNDS_SYNCPAINTPENDING;
      }
   }

   // Send to all the children if this is the desktop window.
   if (UserIsDesktopWindow(Wnd))
   {
      if ( Flags & RDW_ALLCHILDREN ||
          ( !(Flags & RDW_NOCHILDREN) && Wnd->style & WS_CLIPCHILDREN))
      {
         PWND spwndChild = Wnd->spwndChild;
         while(spwndChild)
         {
            if ( spwndChild->style & WS_CHILD &&
                 spwndChild->head.pti != ptiCur)
            {
               spwndChild = spwndChild->spwndNext;
               continue;
            }
            IntSendSyncPaint( spwndChild, Flags );
            spwndChild = spwndChild->spwndNext;
         }
      }
   }
}

/*
 * @name IntCalcWindowRgn
 *
 * Get a window or client region.
 */

HRGN FASTCALL
IntCalcWindowRgn(PWND Wnd, BOOL Client)
{
   HRGN hRgnWindow;

   if (Client)
   {
      hRgnWindow = NtGdiCreateRectRgn(
          Wnd->rcClient.left,
          Wnd->rcClient.top,
          Wnd->rcClient.right,
          Wnd->rcClient.bottom);
   }
   else
   {
      hRgnWindow = NtGdiCreateRectRgn(
          Wnd->rcWindow.left,
          Wnd->rcWindow.top,
          Wnd->rcWindow.right,
          Wnd->rcWindow.bottom);
   }

   if (Wnd->hrgnClip != NULL && !(Wnd->style & WS_MINIMIZE))
   {
      NtGdiOffsetRgn(hRgnWindow,
         -Wnd->rcWindow.left,
         -Wnd->rcWindow.top);
      NtGdiCombineRgn(hRgnWindow, hRgnWindow, Wnd->hrgnClip, RGN_AND);
      NtGdiOffsetRgn(hRgnWindow,
         Wnd->rcWindow.left,
         Wnd->rcWindow.top);
   }

   return hRgnWindow;
}

/*
 * @name IntGetNCUpdateRgn
 *
 * Get non-client update region of a window and optionally validate it.
 *
 * @param Window
 *        Pointer to window to get the NC update region from.
 * @param Validate
 *        Set to TRUE to force validating the NC update region.
 *
 * @return
 *    Handle to NC update region. The caller is responsible for deleting
 *    it.
 */

HRGN FASTCALL
IntGetNCUpdateRgn(PWND Window, BOOL Validate)
{
   HRGN hRgnNonClient;
   HRGN hRgnWindow;
   UINT RgnType, NcType;
   RECT update;

   if (Window->hrgnUpdate != NULL &&
       Window->hrgnUpdate != HRGN_WINDOW)
   {
      hRgnNonClient = IntCalcWindowRgn(Window, FALSE);

      /*
       * If region creation fails it's safe to fallback to whole
       * window region.
       */
      if (hRgnNonClient == NULL)
      {
         return HRGN_WINDOW;
      }

      hRgnWindow = IntCalcWindowRgn(Window, TRUE);
      if (hRgnWindow == NULL)
      {
         GreDeleteObject(hRgnNonClient);
         return HRGN_WINDOW;
      }

      NcType = IntGdiGetRgnBox(hRgnNonClient, &update);

      RgnType = NtGdiCombineRgn(hRgnNonClient, hRgnNonClient, hRgnWindow, RGN_DIFF);

      if (RgnType == ERROR)
      {
         GreDeleteObject(hRgnWindow);
         GreDeleteObject(hRgnNonClient);
         return HRGN_WINDOW;
      }
      else if (RgnType == NULLREGION)
      {
         GreDeleteObject(hRgnWindow);
         GreDeleteObject(hRgnNonClient);
         Window->state &= ~WNDS_UPDATEDIRTY;
         return NULL;
      }

      /*
       * Remove the nonclient region from the standard update region if
       * we were asked for it.
       */

      if (Validate)
      {
         if (NtGdiCombineRgn(Window->hrgnUpdate, Window->hrgnUpdate, hRgnWindow, RGN_AND) == NULLREGION)
         {
            IntGdiSetRegionOwner(Window->hrgnUpdate, GDI_OBJ_HMGR_POWNED);
            GreDeleteObject(Window->hrgnUpdate);
            Window->state &= ~WNDS_UPDATEDIRTY;
            Window->hrgnUpdate = NULL;
            if (!(Window->state & WNDS_INTERNALPAINT))
               MsqDecPaintCountQueue(Window->head.pti);
         }
      }

      /* check if update rgn contains complete nonclient area */
      if (NcType == SIMPLEREGION)
      {
         RECT window;
         IntGetWindowRect( Window, &window );

         if (IntEqualRect( &window, &update ))
         {
            GreDeleteObject(hRgnNonClient);
            hRgnNonClient = HRGN_WINDOW;
         }
      }

      GreDeleteObject(hRgnWindow);

      return hRgnNonClient;
   }
   else
   {
      return Window->hrgnUpdate;
   }
}

VOID FASTCALL
IntSendNCPaint(PWND pWnd, HRGN hRgn)
{
   pWnd->state &= ~WNDS_SENDNCPAINT;

   if ( pWnd == GetW32ThreadInfo()->MessageQueue->spwndActive &&
       !(pWnd->state & WNDS_ACTIVEFRAME))
   {
      pWnd->state |= WNDS_ACTIVEFRAME;
      pWnd->state &= ~WNDS_NONCPAINT;
      hRgn = HRGN_WINDOW;
   }

   if (pWnd->state2 & WNDS2_FORCEFULLNCPAINTCLIPRGN)
   {
      pWnd->state2 &= ~WNDS2_FORCEFULLNCPAINTCLIPRGN;
      hRgn = HRGN_WINDOW;
   }

   if (hRgn) co_IntSendMessage(UserHMGetHandle(pWnd), WM_NCPAINT, (WPARAM)hRgn, 0);
}

VOID FASTCALL
IntSendChildNCPaint(PWND pWnd)
{
    pWnd = pWnd->spwndChild;
    while (pWnd)
    {
        if ((pWnd->hrgnUpdate == NULL) && (pWnd->state & WNDS_SENDNCPAINT))
        {
            PWND Next;
            USER_REFERENCE_ENTRY Ref;

            /* Reference, IntSendNCPaint leaves win32k */
            UserRefObjectCo(pWnd, &Ref);
            IntSendNCPaint(pWnd, HRGN_WINDOW);

            /* Make sure to grab next one before dereferencing/freeing */
            Next = pWnd->spwndNext;
            UserDerefObjectCo(pWnd);
            pWnd = Next;
        }
        else
        {
            pWnd = pWnd->spwndNext;
        }
    }
}

/*
 * IntPaintWindows
 *
 * Internal function used by IntRedrawWindow.
 */

VOID FASTCALL
co_IntPaintWindows(PWND Wnd, ULONG Flags, BOOL Recurse)
{
   HDC hDC;
   HWND hWnd = UserHMGetHandle(Wnd);
   HRGN TempRegion = NULL;

   Wnd->state &= ~WNDS_PAINTNOTPROCESSED;

   if (Wnd->state & WNDS_SENDNCPAINT ||
       Wnd->state & WNDS_SENDERASEBACKGROUND)
   {
      if (!(Wnd->style & WS_VISIBLE))
      {
         Wnd->state &= ~(WNDS_SENDNCPAINT|WNDS_SENDERASEBACKGROUND|WNDS_ERASEBACKGROUND);
         return;
      }
      else
      {
         if (Wnd->hrgnUpdate == NULL)
         {
            Wnd->state &= ~(WNDS_SENDERASEBACKGROUND|WNDS_ERASEBACKGROUND);
         }

         if (Wnd->head.pti == PsGetCurrentThreadWin32Thread())
         {
            if (Wnd->state & WNDS_SENDNCPAINT)
            {
               TempRegion = IntGetNCUpdateRgn(Wnd, TRUE);

               IntSendNCPaint(Wnd, TempRegion);

               if (TempRegion > HRGN_WINDOW && GreIsHandleValid(TempRegion))
               {
                  /* NOTE: The region can already be deleted! */
                  GreDeleteObject(TempRegion);
               }
            }

            if (Wnd->state & WNDS_SENDERASEBACKGROUND)
            {
               PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
               if (Wnd->hrgnUpdate)
               {
                  hDC = UserGetDCEx( Wnd,
                                     Wnd->hrgnUpdate,
                                     DCX_CACHE|DCX_USESTYLE|DCX_INTERSECTRGN|DCX_KEEPCLIPRGN);

                  if (Wnd->head.pti->ppi != pti->ppi)
                  {
                     ERR("Sending DC to another Process!!!\n");
                  }

                  Wnd->state &= ~(WNDS_SENDERASEBACKGROUND|WNDS_ERASEBACKGROUND);
                  // Kill the loop, so Clear before we send.
                  if (!co_IntSendMessage(hWnd, WM_ERASEBKGND, (WPARAM)hDC, 0))
                  {
                     Wnd->state |= (WNDS_SENDERASEBACKGROUND|WNDS_ERASEBACKGROUND);
                  }
                  UserReleaseDC(Wnd, hDC, FALSE);
               }
            }
         }

      }
   }

   /*
    * Check that the window is still valid at this point
    */
   if (!IntIsWindow(hWnd))
   {
      return;
   }

   /*
    * Paint child windows.
    */

   if (!(Flags & RDW_NOCHILDREN) &&
       !(Wnd->style & WS_MINIMIZE) &&
        ( Flags & RDW_ALLCHILDREN ||
         (Flags & RDW_CLIPCHILDREN && Wnd->style & WS_CLIPCHILDREN) ) )
   {
      HWND *List, *phWnd;
      PTHREADINFO pti = PsGetCurrentThreadWin32Thread();

      if ((List = IntWinListChildren(Wnd)))
      {
         for (phWnd = List; *phWnd; ++phWnd)
         {
            if ((Wnd = UserGetWindowObject(*phWnd)) == NULL)
               continue;

            if (Wnd->head.pti != pti && Wnd->style & WS_CHILD)
               continue;

            if (Wnd->style & WS_VISIBLE)
            {
               USER_REFERENCE_ENTRY Ref;
               UserRefObjectCo(Wnd, &Ref);
               co_IntPaintWindows(Wnd, Flags, TRUE);
               UserDerefObjectCo(Wnd);
            }
         }
         ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
      }
   }
}

/*
 * IntUpdateWindows
 *
 * Internal function used by IntRedrawWindow, simplecall.
 */

VOID FASTCALL
co_IntUpdateWindows(PWND Wnd, ULONG Flags, BOOL Recurse)
{
   HWND hWnd = UserHMGetHandle(Wnd);
   USER_REFERENCE_ENTRY Ref;

   if ( Wnd->hrgnUpdate != NULL || Wnd->state & WNDS_INTERNALPAINT )
   {
      if (Wnd->hrgnUpdate)
      {
         if (!IntValidateParents(Wnd, Recurse))
         {
            return;
         }
      }

      if (Wnd->state & WNDS_INTERNALPAINT)
      {
          Wnd->state &= ~WNDS_INTERNALPAINT;

          if (Wnd->hrgnUpdate == NULL)
             MsqDecPaintCountQueue(Wnd->head.pti);
      }

      Wnd->state |= WNDS_PAINTNOTPROCESSED;
      Wnd->state &= ~WNDS_UPDATEDIRTY;

      Wnd->state2 |= WNDS2_WMPAINTSENT;

      UserRefObjectCo(Wnd, &Ref);
      co_IntSendMessage(hWnd, WM_PAINT, 0, 0);

      if (Wnd->state & WNDS_PAINTNOTPROCESSED)
      {
         co_IntPaintWindows(Wnd, RDW_NOCHILDREN, FALSE);
      }
      UserDerefObjectCo(Wnd);
   }

   // Force flags as a toggle. Fixes msg:test_paint_messages:WmChildPaintNc.
   Flags = (Flags & RDW_NOCHILDREN) ? RDW_NOCHILDREN : RDW_ALLCHILDREN; // All children is the default.

  /*
   * Update child windows.
   */

   if (!(Flags & RDW_NOCHILDREN)  &&
        (Flags & RDW_ALLCHILDREN) &&
        !UserIsDesktopWindow(Wnd))
   {
      PWND Child;

      for (Child = Wnd->spwndChild; Child; Child = Child->spwndNext)
      {
         /* transparent window, check for non-transparent sibling to paint first, then skip it */
         if ( Child->ExStyle & WS_EX_TRANSPARENT &&
             ( Child->hrgnUpdate != NULL || Child->state & WNDS_INTERNALPAINT ) )
         {
            PWND Next = Child->spwndNext;
            while (Next)
            {
               if ( Next->hrgnUpdate != NULL || Next->state & WNDS_INTERNALPAINT ) break;

               Next = Next->spwndNext;
            }

            if (Next) continue;
         }

         if (Child->style & WS_VISIBLE)
         {
             USER_REFERENCE_ENTRY Ref;
             UserRefObjectCo(Child, &Ref);
             co_IntUpdateWindows(Child, Flags, TRUE);
             UserDerefObjectCo(Child);
         }
      }
   }
}

VOID FASTCALL
UserUpdateWindows(PWND pWnd, ULONG Flags)
{
   // If transparent and any sibling windows below needs to be painted, leave.
   if (pWnd->ExStyle & WS_EX_TRANSPARENT)
   {
      PWND Next = pWnd->spwndNext;

      while(Next)
      {
         if ( Next->head.pti == pWnd->head.pti &&
            ( Next->hrgnUpdate != NULL || Next->state & WNDS_INTERNALPAINT) )
         {
            return;
         }

         Next = Next->spwndNext;
      }
   }
   co_IntUpdateWindows(pWnd, Flags, FALSE);
}

VOID FASTCALL
UserSyncAndPaintWindows(PWND pWnd, ULONG Flags)
{
   PWND Parent = pWnd;
   // Find parent, if it needs to be painted, leave.
   while(TRUE)
   {
      if ((Parent = Parent->spwndParent) == NULL) break;
      if ( Parent->style & WS_CLIPCHILDREN ) break;
      if ( Parent->hrgnUpdate != NULL || Parent->state & WNDS_INTERNALPAINT ) return;
   }

   IntSendSyncPaint(pWnd, Flags);
   co_IntPaintWindows(pWnd, Flags, FALSE);
}

/*
 * IntInvalidateWindows
 *
 * Internal function used by IntRedrawWindow, UserRedrawDesktop,
 * co_WinPosSetWindowPos, co_UserRedrawWindow.
 */
VOID FASTCALL
IntInvalidateWindows(PWND Wnd, PREGION Rgn, ULONG Flags)
{
   INT RgnType = NULLREGION;
   BOOL HadPaintMessage;

   TRACE("IntInvalidateWindows start Rgn %p\n",Rgn);

   if ( Rgn > PRGN_WINDOW )
   {
      /*
       * If the nonclient is not to be redrawn, clip the region to the client
       * rect
       */
      if ((Flags & RDW_INVALIDATE) != 0 && (Flags & RDW_FRAME) == 0)
      {
         PREGION RgnClient;

         RgnClient = IntSysCreateRectpRgnIndirect(&Wnd->rcClient);
         if (RgnClient)
         {
             RgnType = IntGdiCombineRgn(Rgn, Rgn, RgnClient, RGN_AND);
             REGION_Delete(RgnClient);
         }
      }

      /*
       * Clip the given region with window rectangle (or region)
       */

      if (!Wnd->hrgnClip || (Wnd->style & WS_MINIMIZE))
      {
         PREGION RgnWindow = IntSysCreateRectpRgnIndirect(&Wnd->rcWindow);
         if (RgnWindow)
         {
             RgnType = IntGdiCombineRgn(Rgn, Rgn, RgnWindow, RGN_AND);
             REGION_Delete(RgnWindow);
         }
      }
      else
      {
          PREGION RgnClip = REGION_LockRgn(Wnd->hrgnClip);
          if (RgnClip)
          {
              REGION_bOffsetRgn(Rgn,
                                -Wnd->rcWindow.left,
                                -Wnd->rcWindow.top);
              RgnType = IntGdiCombineRgn(Rgn, Rgn, RgnClip, RGN_AND);
              REGION_bOffsetRgn(Rgn,
                                Wnd->rcWindow.left,
                                Wnd->rcWindow.top);
              REGION_UnlockRgn(RgnClip);
          }
      }
   }
   else
   {
      RgnType = NULLREGION;
   }

   /* Nothing to paint, just return */
   if ((RgnType == NULLREGION && (Flags & RDW_INVALIDATE)) || RgnType == ERROR)
   {
      return;
   }

   /*
    * Save current state of pending updates
    */

   HadPaintMessage = IntIsWindowDirty(Wnd);

   /*
    * Update the region and flags
    */

   // The following flags are used to invalidate the window.
   if (Flags & (RDW_INVALIDATE|RDW_INTERNALPAINT|RDW_ERASE|RDW_FRAME))
   {
      if (Flags & RDW_INTERNALPAINT)
      {
         Wnd->state |= WNDS_INTERNALPAINT;
      }

      if (Flags & RDW_INVALIDATE )
      {
         PREGION RgnUpdate;

         Wnd->state &= ~WNDS_NONCPAINT;

         /* If not the same thread set it dirty. */
         if (Wnd->head.pti != PsGetCurrentThreadWin32Thread())
         {
            Wnd->state |= WNDS_UPDATEDIRTY;
            if (Wnd->state2 & WNDS2_WMPAINTSENT)
               Wnd->state2 |= WNDS2_ENDPAINTINVALIDATE;
         }

         if (Flags & RDW_FRAME)
            Wnd->state |= WNDS_SENDNCPAINT;

         if (Flags & RDW_ERASE)
            Wnd->state |= WNDS_SENDERASEBACKGROUND;

         if (RgnType != NULLREGION && Rgn > PRGN_WINDOW)
         {
            if (Wnd->hrgnUpdate == NULL)
            {
               Wnd->hrgnUpdate = NtGdiCreateRectRgn(0, 0, 0, 0);
               IntGdiSetRegionOwner(Wnd->hrgnUpdate, GDI_OBJ_HMGR_PUBLIC);
            }

            if (Wnd->hrgnUpdate != HRGN_WINDOW)
            {
               RgnUpdate = REGION_LockRgn(Wnd->hrgnUpdate);
               if (RgnUpdate)
               {
                  RgnType = IntGdiCombineRgn(RgnUpdate, RgnUpdate, Rgn, RGN_OR);
                  REGION_UnlockRgn(RgnUpdate);
                  if (RgnType == NULLREGION)
                  {
                     IntGdiSetRegionOwner(Wnd->hrgnUpdate, GDI_OBJ_HMGR_POWNED);
                     GreDeleteObject(Wnd->hrgnUpdate);
                     Wnd->hrgnUpdate = NULL;
                  }
               }
            }
         }

         Flags |= RDW_ERASE|RDW_FRAME; // For children.

      }

      if (!HadPaintMessage && IntIsWindowDirty(Wnd))
      {
         MsqIncPaintCountQueue(Wnd->head.pti);
      }

   }    // The following flags are used to validate the window.
   else if (Flags & (RDW_VALIDATE|RDW_NOINTERNALPAINT|RDW_NOERASE|RDW_NOFRAME))
   {
      if (Wnd->state & WNDS_UPDATEDIRTY && !(Flags & RDW_NOUPDATEDIRTY))
         return;

      if (Flags & RDW_NOINTERNALPAINT)
      {
         Wnd->state &= ~WNDS_INTERNALPAINT;
      }

      if (Flags & RDW_VALIDATE)
      {
         if (Flags & RDW_NOFRAME)
            Wnd->state &= ~WNDS_SENDNCPAINT;

         if (Flags & RDW_NOERASE)
            Wnd->state &= ~(WNDS_SENDERASEBACKGROUND|WNDS_ERASEBACKGROUND);

         if (Wnd->hrgnUpdate > HRGN_WINDOW && RgnType != NULLREGION && Rgn > PRGN_WINDOW)
         {
             PREGION RgnUpdate = REGION_LockRgn(Wnd->hrgnUpdate);

             if (RgnUpdate)
             {
                 RgnType = IntGdiCombineRgn(RgnUpdate, RgnUpdate, Rgn, RGN_DIFF);
                 REGION_UnlockRgn(RgnUpdate);

                 if (RgnType == NULLREGION)
                 {
                     IntGdiSetRegionOwner(Wnd->hrgnUpdate, GDI_OBJ_HMGR_POWNED);
                     GreDeleteObject(Wnd->hrgnUpdate);
                     Wnd->hrgnUpdate = NULL;
                 }
             }
         }
         // If update is null, do not erase.
         if (Wnd->hrgnUpdate == NULL)
         {
            Wnd->state &= ~(WNDS_SENDERASEBACKGROUND|WNDS_ERASEBACKGROUND);
         }
      }

      if (HadPaintMessage && !IntIsWindowDirty(Wnd))
      {
         MsqDecPaintCountQueue(Wnd->head.pti);
      }
   }

   /*
    * Process children if needed
    */

   if (!(Flags & RDW_NOCHILDREN) &&
       !(Wnd->style & WS_MINIMIZE) &&
         ((Flags & RDW_ALLCHILDREN) || !(Wnd->style & WS_CLIPCHILDREN)))
   {
      PWND Child;

      for (Child = Wnd->spwndChild; Child; Child = Child->spwndNext)
      {
         if (Child->style & WS_VISIBLE)
         {
            /*
             * Recursive call to update children hrgnUpdate
             */
            PREGION RgnTemp = IntSysCreateRectpRgn(0, 0, 0, 0);
            if (RgnTemp)
            {
                if (Rgn > PRGN_WINDOW) IntGdiCombineRgn(RgnTemp, Rgn, 0, RGN_COPY);
                IntInvalidateWindows(Child, ((Rgn > PRGN_WINDOW)?RgnTemp:Rgn), Flags);
                REGION_Delete(RgnTemp);
            }
         }
      }
   }
   TRACE("IntInvalidateWindows exit\n");
}

/*
 * IntIsWindowDrawable
 *
 * Remarks
 *    Window is drawable when it is visible and all parents are not
 *    minimized.
 */

BOOL FASTCALL
IntIsWindowDrawable(PWND Wnd)
{
   PWND WndObject;

   for (WndObject = Wnd; WndObject != NULL; WndObject = WndObject->spwndParent)
   {
      if ( WndObject->state2 & WNDS2_INDESTROY ||
           WndObject->state & WNDS_DESTROYED ||
           !WndObject ||
           !(WndObject->style & WS_VISIBLE) ||
            ((WndObject->style & WS_MINIMIZE) && (WndObject != Wnd)))
      {
         return FALSE;
      }
   }

   return TRUE;
}

/*
 * IntRedrawWindow
 *
 * Internal version of NtUserRedrawWindow that takes WND as
 * first parameter.
 */

BOOL FASTCALL
co_UserRedrawWindow(
   PWND Window,
   const RECTL* UpdateRect,
   PREGION UpdateRgn,
   ULONG Flags)
{
   PREGION TmpRgn = NULL;
   TRACE("co_UserRedrawWindow start Rgn %p\n",UpdateRgn);

   /*
    * Step 1.
    * Validation of passed parameters.
    */

   if (!IntIsWindowDrawable(Window))
   {
      return TRUE; // Just do nothing!!!
   }

   if (Window == NULL)
   {
      Window = UserGetDesktopWindow();
   }

   /*
    * Step 2.
    * Transform the parameters UpdateRgn and UpdateRect into
    * a region hRgn specified in screen coordinates.
    */

   if (Flags & (RDW_INVALIDATE | RDW_VALIDATE)) // Both are OKAY!
   {
      /* We can't hold lock on GDI objects while doing roundtrips to user mode,
       * so use a copy instead */
      if (UpdateRgn)
      {
          TmpRgn = IntSysCreateRectpRgn(0, 0, 0, 0);

          if (UpdateRgn > PRGN_WINDOW)
          {
             IntGdiCombineRgn(TmpRgn, UpdateRgn, NULL, RGN_COPY);
          }

          if (Window != UserGetDesktopWindow())
          {
             REGION_bOffsetRgn(TmpRgn, Window->rcClient.left, Window->rcClient.top);
          }
      }
      else
      {
         if (UpdateRect != NULL)
         {
            if (Window == UserGetDesktopWindow())
            {
               TmpRgn = IntSysCreateRectpRgnIndirect(UpdateRect);
            }
            else
            {
               TmpRgn = IntSysCreateRectpRgn(Window->rcClient.left + UpdateRect->left,
                                             Window->rcClient.top  + UpdateRect->top,
                                             Window->rcClient.left + UpdateRect->right,
                                             Window->rcClient.top  + UpdateRect->bottom);
            }
         }
         else
         {
            if ((Flags & (RDW_INVALIDATE | RDW_FRAME)) == (RDW_INVALIDATE | RDW_FRAME) ||
                (Flags & (RDW_VALIDATE | RDW_NOFRAME)) == (RDW_VALIDATE | RDW_NOFRAME))
            {
               if (!RECTL_bIsEmptyRect(&Window->rcWindow))
                   TmpRgn = IntSysCreateRectpRgnIndirect(&Window->rcWindow);
            }
            else
            {
               if (!RECTL_bIsEmptyRect(&Window->rcClient))
                   TmpRgn = IntSysCreateRectpRgnIndirect(&Window->rcClient);
            }
         }
      }
   }

   /* Fixes test RDW_INTERNALPAINT behavior */
   if (TmpRgn == NULL)
   {
      TmpRgn = PRGN_WINDOW; // Need a region so the bits can be set!!!
   }

   /*
    * Step 3.
    * Adjust the window update region depending on hRgn and flags.
    */

   if (Flags & (RDW_INVALIDATE | RDW_VALIDATE | RDW_INTERNALPAINT | RDW_NOINTERNALPAINT) &&
       TmpRgn != NULL)
   {
      IntInvalidateWindows(Window, TmpRgn, Flags);
   }

   /*
    * Step 4.
    * Repaint and erase windows if needed.
    */

   if (Flags & RDW_UPDATENOW)
   {
      UserUpdateWindows(Window, Flags);
   }
   else if (Flags & RDW_ERASENOW)
   {
      if ((Flags & (RDW_NOCHILDREN|RDW_ALLCHILDREN)) == 0)
         Flags |= RDW_CLIPCHILDREN;

      UserSyncAndPaintWindows(Window, Flags);
   }

   /*
    * Step 5.
    * Cleanup ;-)
    */

   if (TmpRgn > PRGN_WINDOW)
   {
      REGION_Delete(TmpRgn);
   }
   TRACE("co_UserRedrawWindow exit\n");

   return TRUE;
}
/* --- pretranslation split: the remaining function groups live in painting.split2..3.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "painting.split2.c"
#include "painting.split3.c"
