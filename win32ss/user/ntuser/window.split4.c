/* window.split4.c — #include fragment of window.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original window.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


/*
 * FUNCTION:
 *   Searches a window's children for a window with the specified
 *   class and name
 * ARGUMENTS:
 *   hwndParent     = The window whose childs are to be searched.
 *       NULL = desktop
 *       HWND_MESSAGE = message-only windows
 *
 *   hwndChildAfter = Search starts after this child window.
 *       NULL = start from beginning
 *
 *   ucClassName    = Class name to search for
 *       Reguired parameter.
 *
 *   ucWindowName   = Window name
 *       ->Buffer == NULL = don't care
 *
 * RETURNS:
 *   The HWND of the window if it was found, otherwise NULL
 */
/*
 * @implemented
 */
HWND APIENTRY
NtUserFindWindowEx(HWND hwndParent,
                   HWND hwndChildAfter,
                   PUNICODE_STRING ucClassName,
                   PUNICODE_STRING ucWindowName,
                   DWORD dwUnknown)
{
   PWND Parent, ChildAfter;
   UNICODE_STRING ClassName = {0}, WindowName = {0};
   HWND Desktop, Ret = NULL;
   BOOL DoMessageWnd = FALSE;
   RTL_ATOM ClassAtom = (RTL_ATOM)0;

   TRACE("Enter NtUserFindWindowEx\n");
   UserEnterShared();

   if (ucClassName != NULL || ucWindowName != NULL)
   {
       _SEH2_TRY
       {
           if (ucClassName != NULL)
           {
               ClassName = ProbeForReadUnicodeString(ucClassName);
               if (ClassName.Length != 0)
               {
                   ProbeForRead(ClassName.Buffer,
                                ClassName.Length,
                                sizeof(WCHAR));
               }
               else if (!IS_ATOM(ClassName.Buffer))
               {
                   EngSetLastError(ERROR_INVALID_PARAMETER);
                   _SEH2_LEAVE;
               }

               if (!IntGetAtomFromStringOrAtom(&ClassName,
                                               &ClassAtom))
               {
                   EngSetLastError(ERROR_CANNOT_FIND_WND_CLASS);
                   _SEH2_LEAVE;
               }
           }

           if (ucWindowName != NULL)
           {
               WindowName = ProbeForReadUnicodeString(ucWindowName);
               if (WindowName.Length != 0)
               {
                   ProbeForRead(WindowName.Buffer,
                                WindowName.Length,
                                sizeof(WCHAR));
               }
           }
       }
       _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
       {
           SetLastNtError(_SEH2_GetExceptionCode());
           _SEH2_YIELD(goto Exit); // Return NULL
       }
       _SEH2_END;

       if (ucClassName != NULL)
       {
           if (ClassName.Length == 0 && ClassName.Buffer != NULL &&
               !IS_ATOM(ClassName.Buffer))
           {
               EngSetLastError(ERROR_INVALID_PARAMETER);
               goto Exit; // Return NULL
           }
           else if (ClassAtom == (RTL_ATOM)0)
           {
               /* LastError code was set by IntGetAtomFromStringOrAtom */
               goto Exit; // Return NULL
           }
       }
   }

   Desktop = IntGetCurrentThreadDesktopWindow();

   if(hwndParent == NULL)
   {
      hwndParent = Desktop;
      DoMessageWnd = TRUE;
   }
   else if(hwndParent == HWND_MESSAGE)
   {
     hwndParent = IntGetMessageWindow();
   }

   if(!(Parent = UserGetWindowObject(hwndParent)))
   {
      goto Exit; // Return NULL
   }

   ChildAfter = NULL;
   if(hwndChildAfter && !(ChildAfter = UserGetWindowObject(hwndChildAfter)))
   {
      goto Exit; // Return NULL
   }

   _SEH2_TRY
   {
       if(UserHMGetHandle(Parent) == Desktop)
       {
          HWND *List, *phWnd;
          PWND TopLevelWindow;
          BOOLEAN CheckWindowName;
          BOOLEAN WindowMatches;
          BOOLEAN ClassMatches;

          /* windows searches through all top-level windows if the parent is the desktop
             window */

          if((List = IntWinListChildren(Parent)))
          {
             phWnd = List;

             if(ChildAfter)
             {
                /* skip handles before and including ChildAfter */
                while(*phWnd && (*(phWnd++) != UserHMGetHandle(ChildAfter)))
                   ;
             }

             CheckWindowName = WindowName.Buffer != 0;

             /* search children */
             while(*phWnd)
             {
                 UNICODE_STRING ustr;

                if(!(TopLevelWindow = UserGetWindowObject(*(phWnd++))))
                {
                   continue;
                }

                /* Do not send WM_GETTEXT messages in the kernel mode version!
                   The user mode version however calls GetWindowText() which will
                   send WM_GETTEXT messages to windows belonging to its processes */
                ustr.Buffer = TopLevelWindow->strName.Buffer;
                ustr.Length = (USHORT)min(TopLevelWindow->strName.Length, MAXUSHORT); // FIXME:LARGE_STRING truncated
                ustr.MaximumLength = (USHORT)min(TopLevelWindow->strName.MaximumLength, MAXUSHORT);
                WindowMatches = !CheckWindowName ||
                                (TopLevelWindow->strName.Length < 0xFFFF &&
                                 !RtlCompareUnicodeString(&WindowName, &ustr, TRUE));
                ClassMatches = (ClassAtom == (RTL_ATOM)0) ||
                               ClassAtom == TopLevelWindow->pcls->atomNVClassName;

                if (WindowMatches && ClassMatches)
                {
                   Ret = UserHMGetHandle(TopLevelWindow);
                   break;
                }

                if (IntFindWindow(TopLevelWindow, NULL, ClassAtom, &WindowName))
                {
                   /* window returns the handle of the top-level window, in case it found
                      the child window */
                   Ret = UserHMGetHandle(TopLevelWindow);
                   break;
                }

             }
             ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
          }
       }
       else
       {
          TRACE("FindWindowEx: Not Desktop Parent!\n");
          Ret = IntFindWindow(Parent, ChildAfter, ClassAtom, &WindowName);
       }

       if (Ret == NULL && DoMessageWnd)
       {
          PWND MsgWindows;

          if((MsgWindows = UserGetWindowObject(IntGetMessageWindow())))
          {
             Ret = IntFindWindow(MsgWindows, ChildAfter, ClassAtom, &WindowName);
          }
       }
   }
   _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
   {
       SetLastNtError(_SEH2_GetExceptionCode());
       Ret = NULL;
   }
   _SEH2_END;

Exit:
   TRACE("Leave NtUserFindWindowEx, ret %p\n", Ret);
   UserLeave();
   return Ret;
}

/* @implemented */
PWND FASTCALL
UserGetAncestor(_In_ PWND pWnd, _In_ UINT uType)
{
    PWND WndAncestor, Parent, pwndMessage;
    PDESKTOP pDesktop;
    PWND pwndDesktop;

    pDesktop = pWnd->head.rpdesk;
    ASSERT(pDesktop);
    ASSERT(pDesktop->pDeskInfo);

    pwndDesktop = pDesktop->pDeskInfo->spwnd;
    if (pWnd == pwndDesktop)
        return NULL;

    pwndMessage = pDesktop->spwndMessage;
    if (pWnd == pwndMessage)
        return NULL;

    Parent = pWnd->spwndParent;
    if (!Parent)
        return NULL;

    switch (uType)
    {
        case GA_PARENT:
            return Parent;

        case GA_ROOT:
            WndAncestor = pWnd;
            if (Parent == pwndDesktop)
                break;

            do
            {
                if (Parent == pwndMessage)
                    break;

                WndAncestor = Parent;

                pDesktop = Parent->head.rpdesk;
                ASSERT(pDesktop);
                ASSERT(pDesktop->pDeskInfo);

                Parent = Parent->spwndParent;
            } while (Parent != pDesktop->pDeskInfo->spwnd);
            break;

        case GA_ROOTOWNER:
            WndAncestor = pWnd;
            for (PWND pwndNode = IntGetParent(pWnd); pwndNode; pwndNode = IntGetParent(pwndNode))
            {
                WndAncestor = pwndNode;
            }
            break;

        default:
            return NULL;
    }

    return WndAncestor;
}

/* @implemented */
HWND APIENTRY
NtUserGetAncestor(_In_ HWND hWnd, _In_ UINT uType)
{
    PWND Window, pwndAncestor;
    HWND hwndAncestor = NULL;

    TRACE("Enter NtUserGetAncestor\n");
    UserEnterShared();

    Window = UserGetWindowObject(hWnd);
    if (!Window)
        goto Quit;

    if (!uType || uType > GA_ROOTOWNER)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        goto Quit;
    }

    pwndAncestor = UserGetAncestor(Window, uType);
    if (pwndAncestor)
        hwndAncestor = UserHMGetHandle(pwndAncestor);

Quit:
    UserLeave();
    TRACE("Leave NtUserGetAncestor returning %p\n", hwndAncestor);
    return hwndAncestor;
}

////
//// ReactOS work around! Keep it the sames as in Combo.c and Controls.h
////
/* combo state struct */
typedef struct
{
   HWND           self;
   HWND           owner;
   UINT           dwStyle;
   HWND           hWndEdit;
   HWND           hWndLBox;
   UINT           wState;
   HFONT          hFont;
   RECT           textRect;
   RECT           buttonRect;
   RECT           droppedRect;
   INT            droppedIndex;
   INT            fixedOwnerDrawHeight;
   INT            droppedWidth;   /* last two are not used unless set */
   INT            editHeight;     /* explicitly */
   LONG           UIState;
} HEADCOMBO,*LPHEADCOMBO;

// Window Extra data container.
typedef struct _WND2CBOX
{
  WND;
  LPHEADCOMBO pCBox;
} WND2CBOX, *PWND2CBOX;

#define CBF_BUTTONDOWN          0x0002
////
////
////
BOOL
APIENTRY
NtUserGetComboBoxInfo(
   HWND hWnd,
   PCOMBOBOXINFO pcbi)
{
   PWND Wnd;
   PPROCESSINFO ppi;
   BOOL NotSameppi = FALSE;
   BOOL Ret = TRUE;

   TRACE("Enter NtUserGetComboBoxInfo\n");
   UserEnterShared();

   if (!(Wnd = UserGetWindowObject(hWnd)))
   {
      Ret = FALSE;
      goto Exit;
   }
   {
       NTSTATUS __SehStatus = SafeProbeForWrite(pcbi, sizeof(COMBOBOXINFO), 1);
       if (!NT_SUCCESS(__SehStatus))
       {
           SetLastNtError(__SehStatus);
           Ret = FALSE;
           goto Exit;
       }
   }

   if (pcbi->cbSize < sizeof(COMBOBOXINFO))
   {
      EngSetLastError(ERROR_INVALID_PARAMETER);
      Ret = FALSE;
      goto Exit;
   }

   // Pass the user pointer, it was already probed.
   if ((Wnd->pcls->atomClassName != gpsi->atomSysClass[ICLS_COMBOBOX]) && Wnd->fnid != FNID_COMBOBOX)
   {
      Ret = (BOOL)co_IntSendMessage(UserHMGetHandle(Wnd), CB_GETCOMBOBOXINFO, 0, (LPARAM)pcbi);
      goto Exit;
   }

   ppi = PsGetCurrentProcessWin32Process();
   NotSameppi = ppi != Wnd->head.pti->ppi;
   if (NotSameppi)
   {
      KeAttachProcess(&Wnd->head.pti->ppi->peProcess->Pcb);
   }

   _SEH2_TRY
   {
      LPHEADCOMBO lphc = ((PWND2CBOX)Wnd)->pCBox;
      pcbi->rcItem = lphc->textRect;
      pcbi->rcButton = lphc->buttonRect;
      pcbi->stateButton = 0;
      if (lphc->wState & CBF_BUTTONDOWN)
         pcbi->stateButton |= STATE_SYSTEM_PRESSED;
      if (RECTL_bIsEmptyRect(&lphc->buttonRect))
         pcbi->stateButton |= STATE_SYSTEM_INVISIBLE;
      pcbi->hwndCombo = lphc->self;
      pcbi->hwndItem = lphc->hWndEdit;
      pcbi->hwndList = lphc->hWndLBox;
   }
   _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
   {
      Ret = FALSE;
      SetLastNtError(_SEH2_GetExceptionCode());
   }
   _SEH2_END;

Exit:
   if (NotSameppi) KeDetachProcess();
   TRACE("Leave NtUserGetComboBoxInfo, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

////
//// ReactOS work around! Keep it the sames as in Listbox.c
////
/* Listbox structure */
typedef struct
{
    HWND        self;           /* Our own window handle */
    HWND        owner;          /* Owner window to send notifications to */
    UINT        style;          /* Window style */
    INT         width;          /* Window width */
    INT         height;         /* Window height */
    VOID       *items;          /* Array of items */
    INT         nb_items;       /* Number of items */
    INT         top_item;       /* Top visible item */
    INT         selected_item;  /* Selected item */
    INT         focus_item;     /* Item that has the focus */
    INT         anchor_item;    /* Anchor item for extended selection */
    INT         item_height;    /* Default item height */
    INT         page_size;      /* Items per listbox page */
    INT         column_width;   /* Column width for multi-column listboxes */
} LB_DESCR;

// Window Extra data container.
typedef struct _WND2LB
{
  WND;
  LB_DESCR * pLBiv;
} WND2LB, *PWND2LB;
////
////
////
DWORD
APIENTRY
NtUserGetListBoxInfo(
   HWND hWnd)
{
   PWND Wnd;
   PPROCESSINFO ppi;
   BOOL NotSameppi = FALSE;
   DWORD Ret = 0;

   TRACE("Enter NtUserGetListBoxInfo\n");
   UserEnterShared();

   if (!(Wnd = UserGetWindowObject(hWnd)))
   {
      goto Exit; // Return 0
   }

   if ((Wnd->pcls->atomClassName != gpsi->atomSysClass[ICLS_LISTBOX]) && Wnd->fnid != FNID_LISTBOX)
   {
      Ret = (DWORD)co_IntSendMessage(UserHMGetHandle(Wnd), LB_GETLISTBOXINFO, 0, 0);
      goto Exit;
   }

   // wine lisbox:test_GetListBoxInfo lb_getlistboxinfo = 0, should not send a message!
   ppi = PsGetCurrentProcessWin32Process();
   NotSameppi = ppi != Wnd->head.pti->ppi;
   if (NotSameppi)
   {
      KeAttachProcess(&Wnd->head.pti->ppi->peProcess->Pcb);
   }

   _SEH2_TRY
   {
      LB_DESCR *descr = ((PWND2LB)Wnd)->pLBiv;
      // See Controls ListBox.c:LB_GETLISTBOXINFO must match...
      Ret = descr->page_size;
   }
   _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
   {
      Ret = 0;
      SetLastNtError(_SEH2_GetExceptionCode());
   }
   _SEH2_END;

Exit:
   if (NotSameppi) KeDetachProcess();
   TRACE("Leave NtUserGetListBoxInfo, ret=%lu\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * NtUserSetParent
 *
 * The NtUserSetParent function changes the parent window of the specified
 * child window.
 *
 * Remarks
 *    The new parent window and the child window must belong to the same
 *    application. If the window identified by the hWndChild parameter is
 *    visible, the system performs the appropriate redrawing and repainting.
 *    For compatibility reasons, NtUserSetParent does not modify the WS_CHILD
 *    or WS_POPUP window styles of the window whose parent is being changed.
 *
 * Status
 *    @implemented
 */

HWND APIENTRY
NtUserSetParent(HWND hWndChild, HWND hWndNewParent)
{
   HWND Ret;

   TRACE("Enter NtUserSetParent\n");
   UserEnterExclusive();

   /*
      Check Parent first from user space, set it here.
    */
   if (!hWndNewParent)
   {
      hWndNewParent = IntGetDesktopWindow();
   }
   else if (hWndNewParent == HWND_MESSAGE)
   {
      hWndNewParent = IntGetMessageWindow();
   }

   Ret = co_UserSetParent(hWndChild, hWndNewParent);

   TRACE("Leave NtUserSetParent, ret=%p\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * UserGetShellWindow
 *
 * Returns a handle to shell window that was set by NtUserSetShellWindowEx.
 *
 * Status
 *    @implemented
 */
HWND FASTCALL UserGetShellWindow(VOID)
{
   PWINSTATION_OBJECT WinStaObject;
   HWND Ret;

   NTSTATUS Status = IntValidateWindowStationHandle(PsGetCurrentProcess()->Win32WindowStation,
                     UserMode,
                     0,
                     &WinStaObject,
                     0);

   if (!NT_SUCCESS(Status))
   {
      SetLastNtError(Status);
      return NULL;
   }

   Ret = (HWND)WinStaObject->ShellWindow;

   ObDereferenceObject(WinStaObject);
   return Ret;
}

/*
 * NtUserSetShellWindowEx
 *
 * This is undocumented function to set global shell window. The global
 * shell window has special handling of window position.
 *
 * Status
 *    @implemented
 */
BOOL APIENTRY
NtUserSetShellWindowEx(HWND hwndShell, HWND hwndListView)
{
   PWINSTATION_OBJECT WinStaObject;
   PWND WndShell, WndListView;
   BOOL Ret = FALSE;
   USER_REFERENCE_ENTRY Ref;
   NTSTATUS Status;
   PTHREADINFO ti;

   TRACE("Enter NtUserSetShellWindowEx\n");
   UserEnterExclusive();

   if (!(WndShell = UserGetWindowObject(hwndShell)))
   {
      goto Exit; // Return FALSE
   }

   if (!(WndListView = UserGetWindowObject(hwndListView)))
   {
      goto Exit; // Return FALSE
   }

   Status = IntValidateWindowStationHandle(PsGetCurrentProcess()->Win32WindowStation,
                     UserMode,
                     0,
                     &WinStaObject,
                     0);

   if (!NT_SUCCESS(Status))
   {
      SetLastNtError(Status);
      goto Exit; // Return FALSE
   }

   /*
    * Test if we are permitted to change the shell window.
    */
   if (WinStaObject->ShellWindow)
   {
      ObDereferenceObject(WinStaObject);
      goto Exit; // Return FALSE
   }

   /*
    * Move shell window into background.
    */
   if (hwndListView && hwndListView != hwndShell)
   {
      /*
       * Disabled for now to get Explorer working.
       * -- Filip, 01/nov/2003
       */
#if 0
      co_WinPosSetWindowPos(WndListView, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE|SWP_NOSIZE|SWP_NOACTIVATE);
#endif

      if (WndListView->ExStyle & WS_EX_TOPMOST)
      {
         ObDereferenceObject(WinStaObject);
         goto Exit; // Return FALSE
      }
   }

   if (WndShell->ExStyle & WS_EX_TOPMOST)
   {
      ObDereferenceObject(WinStaObject);
      goto Exit; // Return FALSE
   }

   UserRefObjectCo(WndShell, &Ref);
   WndShell->state2 |= WNDS2_BOTTOMMOST;
   co_WinPosSetWindowPos(WndShell, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE|SWP_NOSIZE|SWP_NOACTIVATE);

   WinStaObject->ShellWindow = hwndShell;
   WinStaObject->ShellListView = hwndListView;

   ti = GetW32ThreadInfo();
   if (ti->pDeskInfo)
   {
       ti->pDeskInfo->hShellWindow = hwndShell;
       ti->pDeskInfo->spwndShell = WndShell;
       ti->pDeskInfo->spwndBkGnd = WndListView;
       ti->pDeskInfo->ppiShellProcess = ti->ppi;
   }

   UserRegisterHotKey(WndShell, SC_TASKLIST, MOD_CONTROL, VK_ESCAPE);

   UserDerefObjectCo(WndShell);

   ObDereferenceObject(WinStaObject);
   Ret = TRUE;

Exit:
   TRACE("Leave NtUserSetShellWindowEx, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

// Fixes wine Win test_window_styles and todo tests...
static BOOL FASTCALL
IntCheckFrameEdge(ULONG Style, ULONG ExStyle)
{
   if (ExStyle & WS_EX_DLGMODALFRAME)
      return TRUE;
   else if (!(ExStyle & WS_EX_STATICEDGE) && (Style & (WS_DLGFRAME | WS_THICKFRAME)))
      return TRUE;
   else
      return FALSE;
}

static LONG_PTR
co_IntSetWindowLongPtr(HWND hWnd, DWORD Index, LONG_PTR NewValue, BOOL Ansi, ULONG Size, BOOL bAlter)
{
   PWND Window, Parent;
   PWINSTATION_OBJECT WindowStation;
   LONG_PTR OldValue;
   STYLESTRUCT Style;

   if (!(Window = UserGetWindowObject(hWnd)))
   {
      return 0;
   }

   if ((INT)Index >= 0)
   {
      if ((Index + Size) > Window->cbwndExtra)
      {
         EngSetLastError(ERROR_INVALID_INDEX);
         return 0;
      }

      PVOID Address = (PUCHAR)(&Window[1]) + Index;

#ifdef _WIN64
      if (Size == sizeof(LONG))
      {
         OldValue = ReadUnalignedU32(Address);
         WriteUnalignedU32(Address, NewValue);
      }
      else
#endif
      {
         OldValue = ReadUnalignedUlongPtr(Address);
         /*
         if ( Index == DWLP_DLGPROC && Wnd->state & WNDS_DIALOGWINDOW)
         {
            OldValue = (LONG_PTR)IntSetWindowProc( Wnd, (WNDPROC)NewValue, Ansi);
            if (!OldValue) return 0;
         }
         */
         WriteUnalignedUlongPtr(Address, NewValue);
      }

   }
   else
   {
#ifdef _WIN64
      if (Size == sizeof(LONG))
      {
         if ((Index != GWL_STYLE) &&
             (Index != GWL_EXSTYLE) &&
             (Index != GWL_ID) &&
             (Index != GWL_USERDATA))
         {
            ERR("NtUserSetWindowLong(): Index requires pointer size: %lu\n", Index);
            EngSetLastError(ERROR_INVALID_INDEX);
            return 0;
         }
      }
#endif

      switch (Index)
      {
         case GWL_EXSTYLE: // LONG
            OldValue = (LONG) Window->ExStyle;
            Style.styleOld = OldValue;
            Style.styleNew = NewValue;

            co_IntSendMessage(hWnd, WM_STYLECHANGING, GWL_EXSTYLE, (LPARAM) &Style);

            /*
             * Remove extended window style bit WS_EX_TOPMOST for shell windows.
             */
            WindowStation = Window->head.pti->rpdesk->rpwinstaParent;
            if(WindowStation)
            {
               if (hWnd == WindowStation->ShellWindow || hWnd == WindowStation->ShellListView)
                  Style.styleNew &= ~WS_EX_TOPMOST;
            }
            /* WS_EX_WINDOWEDGE depends on some other styles */
            if (IntCheckFrameEdge(Window->style, NewValue))
               Style.styleNew |= WS_EX_WINDOWEDGE;
            else
               Style.styleNew &= ~WS_EX_WINDOWEDGE;

            if (!(Window->ExStyle & WS_EX_LAYERED))
            {
               SetLayeredStatus(Window, 0);
            }

            Window->ExStyle = (DWORD)Style.styleNew;

            co_IntSendMessage(hWnd, WM_STYLECHANGED, GWL_EXSTYLE, (LPARAM) &Style);
            break;

         case GWL_STYLE: // LONG
            OldValue = (LONG) Window->style;
            Style.styleOld = OldValue;
            Style.styleNew = NewValue;

            if (!bAlter)
                co_IntSendMessage(hWnd, WM_STYLECHANGING, GWL_STYLE, (LPARAM) &Style);

            /* WS_CLIPSIBLINGS can't be reset on top-level windows */
            if (UserIsDesktopWindow(Window->spwndParent)) Style.styleNew |= WS_CLIPSIBLINGS;
            /* WS_MINIMIZE can't be reset */
            if (OldValue & WS_MINIMIZE) Style.styleNew |= WS_MINIMIZE;
            /* Fixes wine FIXME: changing WS_DLGFRAME | WS_THICKFRAME is supposed to change WS_EX_WINDOWEDGE too */
            if (IntCheckFrameEdge(NewValue, Window->ExStyle))
               Window->ExStyle |= WS_EX_WINDOWEDGE;
            else
               Window->ExStyle &= ~WS_EX_WINDOWEDGE;

            if ((OldValue & (WS_CHILD | WS_POPUP)) == WS_CHILD)
            {
               if ((NewValue & (WS_CHILD | WS_POPUP)) != WS_CHILD)
               {
                  //// From child to non-child it should be null already.
                  ERR("IDMenu going null! %d\n",Window->IDMenu);
                  Window->IDMenu = 0; // Window->spmenu = 0;
               }
            }
            else
            {
               if ((NewValue & (WS_CHILD | WS_POPUP)) == WS_CHILD)
               {
                  PMENU pMenu = UserGetMenuObject(UlongToHandle(Window->IDMenu));
                  Window->state &= ~WNDS_HASMENU;
                  if (pMenu)
                  {
                     ERR("IDMenu released 0x%p\n",pMenu);
                     // ROS may not hold a lock after setting menu to window. But it should!
                     //IntReleaseMenuObject(pMenu);
                  }
               }
            }

            if ((Style.styleOld ^ Style.styleNew) & WS_VISIBLE)
            {
               if (Style.styleOld & WS_VISIBLE) Window->head.pti->cVisWindows--;
               if (Style.styleNew & WS_VISIBLE) Window->head.pti->cVisWindows++;
               DceResetActiveDCEs( Window );
            }
            Window->style = (DWORD)Style.styleNew;

            if (!bAlter)
                co_IntSendMessage(hWnd, WM_STYLECHANGED, GWL_STYLE, (LPARAM) &Style);
            break;

         case GWLP_WNDPROC: // LONG_PTR
         {
            if ( Window->head.pti->ppi != PsGetCurrentProcessWin32Process() ||
                 Window->fnid & FNID_FREED)
            {
               EngSetLastError(ERROR_ACCESS_DENIED);
               return 0;
            }
            OldValue = (LONG_PTR)IntSetWindowProc(Window,
                                                  (WNDPROC)NewValue,
                                                  Ansi);
            break;
         }

         case GWLP_HINSTANCE: // LONG_PTR
            OldValue = (LONG_PTR) Window->hModule;
            Window->hModule = (HINSTANCE) NewValue;
            break;

         case GWLP_HWNDPARENT: // LONG_PTR
            Parent = Window->spwndParent;
            if (Parent && (UserHMGetHandle(Parent) == IntGetDesktopWindow()))
               OldValue = (LONG_PTR)IntSetOwner(UserHMGetHandle(Window), (HWND)NewValue);
            else
               OldValue = (LONG_PTR)co_UserSetParent(UserHMGetHandle(Window), (HWND)NewValue);
            break;

         case GWLP_ID: // LONG
            OldValue = (LONG) Window->IDMenu;
            Window->IDMenu = (UINT) NewValue;
            break;

         case GWLP_USERDATA: // LONG or LONG_PTR
            OldValue = Window->dwUserData;
            Window->dwUserData = NewValue;
            break;

         default:
            ERR("NtUserSetWindowLong(): Unsupported index %lu\n", Index);
            EngSetLastError(ERROR_INVALID_INDEX);
            OldValue = 0;
            break;
      }
   }

   return OldValue;
}

LONG FASTCALL
co_UserSetWindowLong(HWND hWnd, DWORD Index, LONG NewValue, BOOL Ansi)
{
    return (LONG)co_IntSetWindowLongPtr(hWnd, Index, NewValue, Ansi, sizeof(LONG), FALSE);
}

LONG_PTR FASTCALL
co_UserSetWindowLongPtr(HWND hWnd, DWORD Index, LONG_PTR NewValue, BOOL Ansi)
{
    return co_IntSetWindowLongPtr(hWnd, Index, NewValue, Ansi, sizeof(LONG_PTR), FALSE);
}

/*
 * NtUserSetWindowLong
 *
 * The NtUserSetWindowLong function changes an attribute of the specified
 * window. The function also sets the 32-bit (long) value at the specified
 * offset into the extra window memory.
 *
 * Status
 *    @implemented
 */

LONG APIENTRY
NtUserSetWindowLong(HWND hWnd, DWORD Index, LONG NewValue, BOOL Ansi)
{
   LONG ret;

   UserEnterExclusive();

   if (hWnd == IntGetDesktopWindow())
   {
      EngSetLastError(STATUS_ACCESS_DENIED);
      UserLeave();
      return 0;
   }

   ret = (LONG)co_IntSetWindowLongPtr(hWnd, Index, NewValue, Ansi, sizeof(LONG), FALSE);

   UserLeave();

   return ret;
}

#ifdef _WIN64
LONG_PTR APIENTRY
NtUserSetWindowLongPtr(HWND hWnd, DWORD Index, LONG_PTR NewValue, BOOL Ansi)
{
    LONG_PTR ret;

    UserEnterExclusive();

    if (hWnd == IntGetDesktopWindow())
    {
        EngSetLastError(STATUS_ACCESS_DENIED);
        UserLeave();
        return 0;
    }

    ret = co_IntSetWindowLongPtr(hWnd, Index, NewValue, Ansi, sizeof(LONG_PTR), FALSE);

    UserLeave();

    return ret;
}
#endif // _WIN64

DWORD APIENTRY
NtUserAlterWindowStyle(HWND hWnd, DWORD Index, LONG NewValue)
{
   LONG ret;

   UserEnterExclusive();

   if (hWnd == IntGetDesktopWindow())
   {
      EngSetLastError(STATUS_ACCESS_DENIED);
      UserLeave();
      return 0;
   }

   ret = co_IntSetWindowLongPtr(hWnd, Index, NewValue, FALSE, sizeof(LONG), TRUE);

   UserLeave();

   return ret;
}


/*
 * NtUserSetWindowWord
 *
 * Legacy function similar to NtUserSetWindowLong.
 *
 * Status
 *    @implemented
 */

WORD APIENTRY
NtUserSetWindowWord(HWND hWnd, INT Index, WORD NewValue)
{
   PWND Window;
   WORD OldValue;
   WORD Ret = 0;

   TRACE("Enter NtUserSetWindowWord\n");
   UserEnterExclusive();

   if (hWnd == IntGetDesktopWindow())
   {
      EngSetLastError(STATUS_ACCESS_DENIED);
      goto Exit; // Return 0
   }

   if (!(Window = UserGetWindowObject(hWnd)))
   {
      goto Exit; // Return 0
   }

   switch (Index)
   {
      case GWL_ID:
      case GWL_HINSTANCE:
      case GWL_HWNDPARENT:
         Ret = (WORD)co_UserSetWindowLong(UserHMGetHandle(Window), Index, (UINT)NewValue, TRUE);
         goto Exit;

      default:
         if (Index < 0)
         {
            EngSetLastError(ERROR_INVALID_INDEX);
            goto Exit; // Return 0
         }
   }

   if ((ULONG)Index > (Window->cbwndExtra - sizeof(WORD)))
   {
      EngSetLastError(ERROR_INVALID_INDEX);
      goto Exit; // Return 0
   }

   OldValue = *((WORD *)((PCHAR)(Window + 1) + Index));
   *((WORD *)((PCHAR)(Window + 1) + Index)) = NewValue;

   Ret = OldValue;

Exit:
   TRACE("Leave NtUserSetWindowWord, ret=%u\n", Ret);
   UserLeave();
   return Ret;
}

/*
 QueryWindow based on KJK::Hyperion and James Tabor.

 0 = QWUniqueProcessId
 1 = QWUniqueThreadId
 2 = QWActiveWindow
 3 = QWFocusWindow
 4 = QWIsHung            Implements IsHungAppWindow found
                                by KJK::Hyperion.

 9 = QWKillWindow        When I called this with hWnd ==
                           DesktopWindow, it shutdown the system
                           and rebooted.
*/
/*
 * @implemented
 */
DWORD_PTR APIENTRY
NtUserQueryWindow(HWND hWnd, DWORD Index)
{
/* Console Leader Process CID Window offsets */
#define GWLP_CONSOLE_LEADER_PID 0
#define GWLP_CONSOLE_LEADER_TID 4

   DWORD_PTR Result = 0;
   PWND pWnd, pwndActive;
   PTHREADINFO pti, ptiActive;

   TRACE("Enter NtUserQueryWindow\n");
   UserEnterShared();

   if (!(pWnd = UserGetWindowObject(hWnd)))
   {
      goto Exit; // Return 0
   }

   switch(Index)
   {
      case QUERY_WINDOW_UNIQUE_PROCESS_ID:
      {
         if ( (pWnd->head.pti->TIF_flags & TIF_CSRSSTHREAD) &&
              (pWnd->pcls->atomClassName == gaGuiConsoleWndClass) )
         {
            // IntGetWindowLong(offset == GWLP_CONSOLE_LEADER_PID)
            Result = (DWORD_PTR)(*((LONG_PTR*)((PCHAR)(pWnd + 1) + GWLP_CONSOLE_LEADER_PID)));
         }
         else
         {
            Result = (DWORD_PTR)IntGetWndProcessId(pWnd);
         }
         break;
      }

      case QUERY_WINDOW_UNIQUE_THREAD_ID:
      {
         if ( (pWnd->head.pti->TIF_flags & TIF_CSRSSTHREAD) &&
              (pWnd->pcls->atomClassName == gaGuiConsoleWndClass) )
         {
            // IntGetWindowLong(offset == GWLP_CONSOLE_LEADER_TID)
            Result = (DWORD_PTR)(*((LONG_PTR*)((PCHAR)(pWnd + 1) + GWLP_CONSOLE_LEADER_TID)));
         }
         else
         {
            Result = (DWORD_PTR)IntGetWndThreadId(pWnd);
         }
         break;
      }

      case QUERY_WINDOW_ACTIVE:
         Result = (DWORD_PTR)(pWnd->head.pti->MessageQueue->spwndActive ? UserHMGetHandle(pWnd->head.pti->MessageQueue->spwndActive) : 0);
         break;

      case QUERY_WINDOW_FOCUS:
         Result = (DWORD_PTR)(pWnd->head.pti->MessageQueue->spwndFocus ? UserHMGetHandle(pWnd->head.pti->MessageQueue->spwndFocus) : 0);
         break;

      case QUERY_WINDOW_ISHUNG:
         Result = (pWnd->fnid == FNID_GHOST) || MsqIsHung(pWnd->head.pti, MSQ_HUNG);
         break;

      case QUERY_WINDOW_REAL_ID:
         Result = (DWORD_PTR)pWnd->head.pti->pEThread->Cid.UniqueProcess;
         break;

      case QUERY_WINDOW_FOREGROUND:
         Result = (pWnd->head.pti->MessageQueue == gpqForeground);
         break;

      case QUERY_WINDOW_DEFAULT_IME: /* default IME window */
         if (pWnd->head.pti->spwndDefaultIme)
            Result = (DWORD_PTR)UserHMGetHandle(pWnd->head.pti->spwndDefaultIme);
         break;

      case QUERY_WINDOW_DEFAULT_ICONTEXT: /* default input context handle */
         if (pWnd->head.pti->spDefaultImc)
            Result = (DWORD_PTR)UserHMGetHandle(pWnd->head.pti->spDefaultImc);
         break;

      case QUERY_WINDOW_ACTIVE_IME:
         if (gpqForeground && gpqForeground->spwndActive)
         {
             pwndActive = gpqForeground->spwndActive;
             pti = PsGetCurrentThreadWin32Thread();
             if (pti->rpdesk == pwndActive->head.rpdesk)
             {
                ptiActive = pwndActive->head.pti;
                if (ptiActive->spwndDefaultIme)
                   Result = (DWORD_PTR)UserHMGetHandle(ptiActive->spwndDefaultIme);
             }
         }
         break;
   }

Exit:
   TRACE("Leave NtUserQueryWindow, ret=%u\n", Result);
   UserLeave();
   return Result;
}