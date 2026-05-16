/* scrollbar.split2.c — #include fragment of scrollbar.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original scrollbar.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


void
IntDrawScrollBar(PWND Wnd, HDC DC, INT Bar)
{
   PTHREADINFO pti;
   SCROLLBARINFO Info;
   BOOL Vertical;

   pti = PsGetCurrentThreadWin32Thread();

   // Get scrollbar info
   switch (Bar)
   {
      case SB_HORZ:
        Vertical = FALSE;
        break;

      case SB_VERT:
        Vertical = TRUE;
        break;

      case SB_CTL:
        Vertical = (Wnd->style & SBS_VERT) != 0;
        break;

      default:
        return;
   }

   if (!co_IntGetScrollBarInfo(Wnd, IntScrollGetObjectId(Bar), &Info))
      return;

   if (RECTL_bIsEmptyRect(&Info.rcScrollBar))
      return;

   // Draw arrows
   if (Info.dxyLineButton)
      IntDrawScrollArrows(DC, &Info, Vertical);

   // Draw interior
   IntDrawScrollInterior(Wnd, DC, Bar, Vertical, &Info);

   // If scrollbar has focus, reposition the caret
   if (Wnd == pti->MessageQueue->spwndFocus && Bar == SB_CTL)
   {
      if (Vertical)
         co_IntSetCaretPos(Info.rcScrollBar.top + 1, Info.dxyLineButton + 1);
      else
         co_IntSetCaretPos(Info.dxyLineButton + 1, Info.rcScrollBar.top + 1);
   }
}

LRESULT APIENTRY
ScrollBarWndProc(HWND hWnd, UINT Msg, WPARAM wParam, LPARAM lParam)
{
   LRESULT lResult = 0;
   PWND pWnd;
   pWnd = UserGetWindowObject(hWnd);
   if (!pWnd) return 0;

   switch(Msg)
   {
      case WM_ENABLE:
         if (pWnd->pSBInfo)
            pWnd->pSBInfo->WSBflags = wParam ? ESB_ENABLE_BOTH : ESB_DISABLE_BOTH;
         break;
   }
   return lResult;
}

BOOL
APIENTRY
NtUserGetScrollBarInfo(HWND hWnd, LONG idObject, PSCROLLBARINFO psbi)
{
   NTSTATUS Status;
   SCROLLBARINFO sbi;
   PWND Window;
   BOOL Ret = FALSE;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserGetScrollBarInfo\n");
   UserEnterExclusive();

   Status = MmCopyFromCaller(&sbi, psbi, sizeof(SCROLLBARINFO));
   if(!NT_SUCCESS(Status) || (sbi.cbSize != sizeof(SCROLLBARINFO)))
   {
      SetLastNtError(Status);
      goto Exit; // Return FALSE
   }

   if(!(Window = UserGetWindowObject(hWnd)))
      goto Exit; // Return FALSE

   UserRefObjectCo(Window, &Ref);
   Ret = co_IntGetScrollBarInfo(Window, idObject, &sbi);
   UserDerefObjectCo(Window);

   Status = MmCopyToCaller(psbi, &sbi, sizeof(SCROLLBARINFO));
   if(!NT_SUCCESS(Status))
   {
      SetLastNtError(Status);
      Ret = FALSE;
   }

Exit:
   TRACE("Leave NtUserGetScrollBarInfo, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

BOOL
APIENTRY
NtUserSBGetParms(
  HWND hWnd,
  int fnBar,
  PSBDATA pSBData,
  LPSCROLLINFO lpsi)
{
   PWND Window;
   SCROLLINFO psi;
   BOOL Ret = FALSE;
   SBDATA SBDataSafe;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserGetScrollInfo\n");
   UserEnterShared();

   _SEH2_TRY
   {
      RtlCopyMemory(&psi, lpsi, sizeof(SCROLLINFO));
      if (pSBData)
         RtlCopyMemory(&SBDataSafe, pSBData, sizeof(SBDATA));
   }
   _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
   {
      ERR("NtUserGetScrollInfo Failed size\n");
      SetLastNtError(_SEH2_GetExceptionCode());
      _SEH2_YIELD(goto Exit); // Return FALSE
   }
   _SEH2_END

   if(!(Window = UserGetWindowObject(hWnd)))
   {
      ERR("NtUserGetScrollInfo Bad window\n");
      goto Exit; // Return FALSE
   }

   UserRefObjectCo(Window, &Ref);
   Ret = co_IntGetScrollInfo(Window, fnBar, &SBDataSafe, &psi);
   UserDerefObjectCo(Window);

   _SEH2_TRY
   {
      RtlCopyMemory(lpsi, &psi, sizeof(SCROLLINFO));
   }
   _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
   {
      ERR("NtUserGetScrollInfo Failed copy to user\n");
      SetLastNtError(_SEH2_GetExceptionCode());
      Ret = FALSE;
      _SEH2_YIELD(goto Exit);
   }
   _SEH2_END

Exit:
   TRACE("Leave NtUserGetScrollInfo, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

BOOL
APIENTRY
NtUserEnableScrollBar(
   HWND hWnd,
   UINT wSBflags,
   UINT wArrows)
{
   UINT OrigArrows;
   PWND Window = NULL;
   PSCROLLBARINFO InfoV = NULL, InfoH = NULL;
   BOOL Chg = FALSE;
   BOOL Ret = FALSE;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserEnableScrollBar\n");
   UserEnterExclusive();

   if (!(Window = UserGetWindowObject(hWnd)) || UserIsDesktopWindow(Window) || UserIsMessageWindow(Window))
      goto Cleanup; // Return FALSE

   UserRefObjectCo(Window, &Ref);

   if (!co_IntCreateScrollBars(Window))
      goto Cleanup; // Return FALSE

   OrigArrows = Window->pSBInfo->WSBflags;
   Window->pSBInfo->WSBflags = wArrows;

   if (wSBflags == SB_CTL)
   {
      if ((wArrows == ESB_DISABLE_BOTH || wArrows == ESB_ENABLE_BOTH))
         IntEnableWindow(hWnd, (wArrows == ESB_ENABLE_BOTH));

      Ret = TRUE;
      goto Cleanup;
   }

   if(wSBflags != SB_BOTH && !SBID_IS_VALID(wSBflags))
   {
      EngSetLastError(ERROR_INVALID_PARAMETER);
      ERR("Trying to set scrollinfo for unknown scrollbar type %u\n", wSBflags);
      goto Cleanup; // Return FALSE
   }

   switch(wSBflags)
   {
      case SB_BOTH:
         InfoV = IntGetScrollbarInfoFromWindow(Window, SB_VERT);
         // Fall through
      case SB_HORZ:
         InfoH = IntGetScrollbarInfoFromWindow(Window, SB_HORZ);
         break;
      case SB_VERT:
         InfoV = IntGetScrollbarInfoFromWindow(Window, SB_VERT);
         break;
      default:
         goto Cleanup; // Return FALSE
   }

   if(InfoV)
      Chg = IntEnableScrollBar(FALSE, InfoV, wArrows);

   if(InfoH)
      Chg = (IntEnableScrollBar(TRUE, InfoH, wArrows) || Chg);

   ERR("FIXME: EnableScrollBar wSBflags %u wArrows %u Chg %d\n", wSBflags, wArrows, Chg);
// Done in user32:
//   SCROLL_RefreshScrollBar(hwnd, nBar, TRUE, TRUE);

   Ret = Chg;
   goto Cleanup; // FIXME

   if (OrigArrows == wArrows)
   {
      Ret = FALSE;
      goto Cleanup;
   }

   Ret = TRUE;

Cleanup:
   if (Window)
      UserDerefObjectCo(Window);

   TRACE("Leave NtUserEnableScrollBar, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

DWORD
APIENTRY
NtUserSetScrollInfo(
   HWND hWnd,
   int fnBar,
   LPCSCROLLINFO lpsi,
   BOOL bRedraw)
{
   PWND Window = NULL;
   NTSTATUS Status;
   SCROLLINFO ScrollInfo;
   DWORD Ret = 0;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserSetScrollInfo\n");
   UserEnterExclusive();

   if(!(Window = UserGetWindowObject(hWnd)) || UserIsDesktopWindow(Window) || UserIsMessageWindow(Window))
      goto Cleanup; // Return 0

   UserRefObjectCo(Window, &Ref);

   Status = MmCopyFromCaller(&ScrollInfo, lpsi, sizeof(SCROLLINFO) - sizeof(ScrollInfo.nTrackPos));
   if(!NT_SUCCESS(Status))
   {
      SetLastNtError(Status);
      goto Cleanup; // Return 0
   }

   Ret = co_IntSetScrollInfo(Window, fnBar, &ScrollInfo, bRedraw);

Cleanup:
   if (Window)
      UserDerefObjectCo(Window);

   TRACE("Leave NtUserSetScrollInfo, ret=%lu\n", Ret);
   UserLeave();
   return Ret;
}

DWORD APIENTRY
NtUserShowScrollBar(HWND hWnd, int nBar, DWORD bShow)
{
   PWND Window;
   DWORD ret = 0;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserShowScrollBar\n");
   UserEnterExclusive();

   Window = UserGetWindowObject(hWnd);
   if (Window)
   {
      UserRefObjectCo(Window, &Ref);
      ret = co_UserShowScrollBar(Window, nBar, (nBar == SB_VERT) ? 0 : bShow,
                                               (nBar == SB_HORZ) ? 0 : bShow);
      UserDerefObjectCo(Window);
   }

   TRACE("Leave NtUserShowScrollBar, ret=%lu\n", ret);
   UserLeave();
   return ret;
}

// Ugly NtUser API

BOOL
APIENTRY
NtUserSetScrollBarInfo(
   HWND hWnd,
   LONG idObject,
   SETSCROLLBARINFO *info)
{
   PWND Window = NULL;
   SETSCROLLBARINFO Safeinfo;
   PSCROLLBARINFO sbi;
   LPSCROLLINFO psi;
   NTSTATUS Status;
   LONG Obj;
   BOOL Ret = FALSE;
   USER_REFERENCE_ENTRY Ref;

   TRACE("Enter NtUserSetScrollBarInfo\n");
   UserEnterExclusive();

   if(!(Window = UserGetWindowObject(hWnd)))
      goto Cleanup; // Return FALSE

   UserRefObjectCo(Window, &Ref);

   Obj = SBOBJ_TO_SBID(idObject);
   if(!SBID_IS_VALID(Obj))
   {
      EngSetLastError(ERROR_INVALID_PARAMETER);
      ERR("Trying to set scrollinfo for unknown scrollbar type %d\n", Obj);
      goto Cleanup; // Return FALSE
   }

   if(!co_IntCreateScrollBars(Window))
      goto Cleanup; // Return FALSE

   Status = MmCopyFromCaller(&Safeinfo, info, sizeof(SETSCROLLBARINFO));
   if(!NT_SUCCESS(Status))
   {
      SetLastNtError(Status);
      goto Cleanup; // Return FALSE
   }

   sbi = IntGetScrollbarInfoFromWindow(Window, Obj);
   psi = IntGetScrollInfoFromWindow(Window, Obj);

   psi->nTrackPos = Safeinfo.nTrackPos;
   sbi->reserved = Safeinfo.reserved;
   RtlCopyMemory(&sbi->rgstate, &Safeinfo.rgstate, sizeof(Safeinfo.rgstate));

   Ret = TRUE;

Cleanup:
   if (Window)
      UserDerefObjectCo(Window);

   TRACE("Leave NtUserSetScrollBarInfo, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

/* EOF */
