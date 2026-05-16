/* window.split5.c — #include fragment of window.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original window.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


/*
 * @implemented
 */
UINT APIENTRY
NtUserRegisterWindowMessage(PUNICODE_STRING MessageNameUnsafe)
{
   UNICODE_STRING SafeMessageName;
   NTSTATUS Status;
   UINT Ret = 0;

   TRACE("Enter NtUserRegisterWindowMessage\n");
   UserEnterExclusive();

   if(MessageNameUnsafe == NULL)
   {
      EngSetLastError(ERROR_INVALID_PARAMETER);
      goto Exit; // Return 0
   }

   Status = IntSafeCopyUnicodeStringTerminateNULL(&SafeMessageName, MessageNameUnsafe);
   if(!NT_SUCCESS(Status))
   {
      SetLastNtError(Status);
      goto Exit; // Return 0
   }

   Ret = (UINT)IntAddAtom(SafeMessageName.Buffer);
   if (SafeMessageName.Buffer)
      ExFreePoolWithTag(SafeMessageName.Buffer, TAG_STRING);

Exit:
   TRACE("Leave NtUserRegisterWindowMessage, ret=%u\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * @implemented
 */
BOOL APIENTRY
NtUserSetWindowFNID(HWND hWnd,
                    WORD fnID)
{
   PWND Wnd;
   BOOL Ret = FALSE;

   TRACE("Enter NtUserSetWindowFNID\n");
   UserEnterExclusive();

   if (!(Wnd = UserGetWindowObject(hWnd)))
   {
      goto Exit; // Return FALSE
   }

   if (Wnd->head.pti->ppi != PsGetCurrentProcessWin32Process())
   {
      EngSetLastError(ERROR_ACCESS_DENIED);
      goto Exit; // Return FALSE
   }

   // From user land we only set these.
   if (fnID != FNID_DESTROY)
   {
      /* HACK: The minimum should be FNID_BUTTON, but menu code relies on this */
      if (fnID < FNID_FIRST || fnID > FNID_GHOST ||
          Wnd->fnid != 0)
      {
         EngSetLastError(ERROR_INVALID_PARAMETER);
         goto Exit; // Return FALSE
      }
   }

   Wnd->fnid |= fnID;
   Ret = TRUE;

Exit:
   TRACE("Leave NtUserSetWindowFNID\n");
   UserLeave();
   return Ret;
}

BOOL APIENTRY
DefSetText(PWND Wnd, PCWSTR WindowText)
{
   UNICODE_STRING UnicodeString;
   BOOL Ret = FALSE;

   RtlInitUnicodeString(&UnicodeString, WindowText);

   if (UnicodeString.Length != 0)
   {
      if (Wnd->strName.MaximumLength > 0 &&
          UnicodeString.Length <= Wnd->strName.MaximumLength - sizeof(UNICODE_NULL))
      {
         ASSERT(Wnd->strName.Buffer != NULL);

         Wnd->strName.Length = UnicodeString.Length;
         Wnd->strName.Buffer[UnicodeString.Length / sizeof(WCHAR)] = L'\0';
         RtlCopyMemory(Wnd->strName.Buffer,
                              UnicodeString.Buffer,
                              UnicodeString.Length);
      }
      else
      {
         PWCHAR buf;
         Wnd->strName.MaximumLength = Wnd->strName.Length = 0;
         buf = Wnd->strName.Buffer;
         Wnd->strName.Buffer = NULL;
         if (buf != NULL)
         {
            DesktopHeapFree(Wnd->head.rpdesk, buf);
         }

         Wnd->strName.Buffer = DesktopHeapAlloc(Wnd->head.rpdesk,
                                                   UnicodeString.Length + sizeof(UNICODE_NULL));
         if (Wnd->strName.Buffer != NULL)
         {
            Wnd->strName.Buffer[UnicodeString.Length / sizeof(WCHAR)] = L'\0';
            RtlCopyMemory(Wnd->strName.Buffer,
                                 UnicodeString.Buffer,
                                 UnicodeString.Length);
            Wnd->strName.MaximumLength = UnicodeString.Length + sizeof(UNICODE_NULL);
            Wnd->strName.Length = UnicodeString.Length;
         }
         else
         {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            goto Exit;
         }
      }
   }
   else
   {
      Wnd->strName.Length = 0;
      if (Wnd->strName.Buffer != NULL)
          Wnd->strName.Buffer[0] = L'\0';
   }

   // FIXME: HAX! Windows does not do this in here!
   // In User32, these are called after: NotifyWinEvent EVENT_OBJECT_NAMECHANGE than
   // RepaintButton, StaticRepaint, NtUserCallHwndLock HWNDLOCK_ROUTINE_REDRAWFRAMEANDHOOK, etc.
   /* Send shell notifications */
   if (!Wnd->spwndOwner && !IntGetParent(Wnd))
   {
      co_IntShellHookNotify(HSHELL_REDRAW, (WPARAM) UserHMGetHandle(Wnd), FALSE); // FIXME Flashing?
   }

   Ret = TRUE;
Exit:
   if (UnicodeString.Buffer) RtlFreeUnicodeString(&UnicodeString);
   return Ret;
}

/*
 * NtUserDefSetText
 *
 * Undocumented function that is called from DefWindowProc to set
 * window text.
 *
 * Status
 *    @implemented
 */
BOOL APIENTRY
NtUserDefSetText(HWND hWnd, PLARGE_STRING WindowText)
{
   PWND Wnd;
   LARGE_STRING SafeText;
   UNICODE_STRING UnicodeString;
   BOOL Ret = TRUE;

   TRACE("Enter NtUserDefSetText\n");

   if (WindowText != NULL)
   {
      _SEH2_TRY
      {
         SafeText = ProbeForReadLargeString(WindowText);
      }
      _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
      {
         Ret = FALSE;
         SetLastNtError(_SEH2_GetExceptionCode());
      }
      _SEH2_END;

      if (!Ret)
         return FALSE;
   }
   else
      return TRUE;

   UserEnterExclusive();

   if(!(Wnd = UserGetWindowObject(hWnd)))
   {
      UserLeave();
      return FALSE;
   }

   // ReactOS uses Unicode and not mixed. Up/Down converting will take time.
   // Brought to you by: The Wine Project! Dysfunctional Thought Processes!
   // Now we know what the bAnsi is for.
   RtlInitUnicodeString(&UnicodeString, NULL);
   if (SafeText.Buffer)
   {
      _SEH2_TRY
      {
         if (SafeText.bAnsi)
            ProbeForRead(SafeText.Buffer, SafeText.Length, sizeof(CHAR));
         else
            ProbeForRead(SafeText.Buffer, SafeText.Length, sizeof(WCHAR));
         Ret = RtlLargeStringToUnicodeString(&UnicodeString, &SafeText);
      }
      _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
      {
         Ret = FALSE;
         SetLastNtError(_SEH2_GetExceptionCode());
      }
      _SEH2_END;
      if (!Ret) goto Exit;
   }

   if (UnicodeString.Length != 0)
   {
      if (Wnd->strName.MaximumLength > 0 &&
          UnicodeString.Length <= Wnd->strName.MaximumLength - sizeof(UNICODE_NULL))
      {
         ASSERT(Wnd->strName.Buffer != NULL);

         Wnd->strName.Length = UnicodeString.Length;
         Wnd->strName.Buffer[UnicodeString.Length / sizeof(WCHAR)] = L'\0';
         RtlCopyMemory(Wnd->strName.Buffer,
                              UnicodeString.Buffer,
                              UnicodeString.Length);
      }
      else
      {
         PWCHAR buf;
         Wnd->strName.MaximumLength = Wnd->strName.Length = 0;
         buf = Wnd->strName.Buffer;
         Wnd->strName.Buffer = NULL;
         if (buf != NULL)
         {
            DesktopHeapFree(Wnd->head.rpdesk, buf);
         }

         Wnd->strName.Buffer = DesktopHeapAlloc(Wnd->head.rpdesk,
                                                   UnicodeString.Length + sizeof(UNICODE_NULL));
         if (Wnd->strName.Buffer != NULL)
         {
            Wnd->strName.Buffer[UnicodeString.Length / sizeof(WCHAR)] = L'\0';
            RtlCopyMemory(Wnd->strName.Buffer,
                                 UnicodeString.Buffer,
                                 UnicodeString.Length);
            Wnd->strName.MaximumLength = UnicodeString.Length + sizeof(UNICODE_NULL);
            Wnd->strName.Length = UnicodeString.Length;
         }
         else
         {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            Ret = FALSE;
            goto Exit;
         }
      }
   }
   else
   {
      Wnd->strName.Length = 0;
      if (Wnd->strName.Buffer != NULL)
          Wnd->strName.Buffer[0] = L'\0';
   }

   // FIXME: HAX! Windows does not do this in here!
   // In User32, these are called after: NotifyWinEvent EVENT_OBJECT_NAMECHANGE than
   // RepaintButton, StaticRepaint, NtUserCallHwndLock HWNDLOCK_ROUTINE_REDRAWFRAMEANDHOOK, etc.
   /* Send shell notifications */
   if (!Wnd->spwndOwner && !IntGetParent(Wnd))
   {
      co_IntShellHookNotify(HSHELL_REDRAW, (WPARAM) hWnd, FALSE); // FIXME Flashing?
   }

   Ret = TRUE;
Exit:
   if (UnicodeString.Buffer) RtlFreeUnicodeString(&UnicodeString);
   TRACE("Leave NtUserDefSetText, ret=%i\n", Ret);
   UserLeave();
   return Ret;
}

/*
 * NtUserInternalGetWindowText
 *
 * Status
 *    @implemented
 */

INT APIENTRY
NtUserInternalGetWindowText(HWND hWnd, LPWSTR lpString, INT nMaxCount)
{
   PWND Wnd;
   NTSTATUS Status;
   INT Result = 0;

   TRACE("Enter NtUserInternalGetWindowText\n");
   UserEnterShared();

   if(lpString && (nMaxCount <= 1))
   {
      EngSetLastError(ERROR_INVALID_PARAMETER);
      goto Exit; // Return 0
   }

   if(!(Wnd = UserGetWindowObject(hWnd)))
   {
      goto Exit; // Return 0
   }

   Result = Wnd->strName.Length / sizeof(WCHAR);
   if(lpString)
   {
      const WCHAR Terminator = L'\0';
      INT Copy;
      WCHAR *Buffer = (WCHAR*)lpString;

      Copy = min(nMaxCount - 1, Result);
      if(Copy > 0)
      {
         Status = MmCopyToCaller(Buffer, Wnd->strName.Buffer, Copy * sizeof(WCHAR));
         if(!NT_SUCCESS(Status))
         {
            SetLastNtError(Status);
            Result = 0;
            goto Exit;
         }
         Buffer += Copy;
      }

      Status = MmCopyToCaller(Buffer, &Terminator, sizeof(WCHAR));
      if(!NT_SUCCESS(Status))
      {
         SetLastNtError(Status);
         Result = 0;
         goto Exit;
      }

      Result = Copy;
   }

Exit:
   TRACE("Leave NtUserInternalGetWindowText, ret=%i\n", Result);
   UserLeave();
   return Result;
}

/*
  API Call
*/
BOOL
FASTCALL
IntShowOwnedPopups(PWND OwnerWnd, BOOL fShow )
{
   int count = 0;
   PWND pWnd;
   HWND *win_array;

//   ASSERT(OwnerWnd);

   TRACE("Enter ShowOwnedPopups Show: %s\n", (fShow ? "TRUE" : "FALSE"));

   /* NOTE: Popups are not children */
   win_array = IntWinListOwnedPopups(OwnerWnd);

   if (!win_array)
      return TRUE;

   while (win_array[count])
      count++;
   while (--count >= 0)
   {
      if (!(pWnd = ValidateHwndNoErr( win_array[count] )))
         continue;
      ASSERT(pWnd->spwndOwner == OwnerWnd);

      if (fShow)
      {
         if (pWnd->state & WNDS_HIDDENPOPUP)
         {
            /* In Windows, ShowOwnedPopups(TRUE) generates
             * WM_SHOWWINDOW messages with SW_PARENTOPENING,
             * regardless of the state of the owner
             */
            co_IntSendMessage(win_array[count], WM_SHOWWINDOW, SW_SHOWNORMAL, SW_PARENTOPENING);
            pWnd->state &= ~WNDS_HIDDENPOPUP;
            continue;
         }
      }
      else
      {
         if (pWnd->style & WS_VISIBLE)
         {
            /* In Windows, ShowOwnedPopups(FALSE) generates
             * WM_SHOWWINDOW messages with SW_PARENTCLOSING,
             * regardless of the state of the owner
             */
            co_IntSendMessage(win_array[count], WM_SHOWWINDOW, SW_HIDE, SW_PARENTCLOSING);
            pWnd->state |= WNDS_HIDDENPOPUP;
            continue;
         }
      }
   }
   ExFreePoolWithTag(win_array, USERTAG_WINDOWLIST);
   TRACE("Leave ShowOwnedPopups\n");
   return TRUE;
}

/* EOF */
