// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_43C66C260828: [u16; 2] = [0x000Du16, 0x0000];
pub static RUSTCONV_WSTR_60D2AC7E566A: [u16; 3] = [0x0020u16, 0x0008, 0x0000];
pub static RUSTCONV_WSTR_D55BBD9863B9: [u16; 8] = [0x004Eu16, 0x004F, 0x0042, 0x0052, 0x0045, 0x0041, 0x004B, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn PrintError(dwError: DWORD) {
    if dwError == ERROR_SUCCESS as DWORD {
        return;
    }
    ConMsgPuts(
        StdErr,
        FORMAT_MESSAGE_FROM_SYSTEM as DWORD,
        core::ptr::null_mut(),
        dwError,
        LANG_USER_DEFAULT as DWORD,
    );
    ConPuts(
        StdErr,
        RUSTCONV_WSTR_075DE2B906DB.as_ptr(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CtrlCIntercept(dwCtrlType: DWORD) -> BOOL {
    match dwCtrlType {
        CTRL_C_EVENT => {
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            SetConsoleCtrlHandler(None, (FALSE) as i32);
            ExitProcess(EXIT_FAILURE);
            return (TRUE) as BOOL;
        }
        _ => {}
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn InputWait(mut bNoBreak: BOOL, mut timerValue: INT) -> INT {
    let mut Status: INT = (EXIT_SUCCESS as i32);
    let mut hInput: HANDLE = core::ptr::null_mut();
    let mut bUseTimer: BOOL = (timerValue != -1) as BOOL;
    let mut hTimer: HANDLE = core::ptr::null_mut();
    let mut dwStartTime: DWORD = 0;
    let mut timeElapsed: LONG = 0;
    let mut dwWaitState: DWORD = 0;
    let mut InputRecords: [INPUT_RECORD; 5] = ::core::mem::zeroed();
    let mut NumRecords: ULONG = 0;
    let mut i: ULONG = 0;
    let mut DisplayMsg: BOOL = (TRUE) as i32;
    let mut WaitMsgId: UINT = (if bNoBreak != 0 {
        IDS_NOBREAK_INPUT
    } else {
        IDS_USER_INPUT
    }) as UINT;
    let mut WaitCountMsgId: UINT = (if bNoBreak != 0 {
        IDS_NOBREAK_INPUT_COUNT
    } else {
        IDS_USER_INPUT_COUNT
    }) as UINT;

    /* Retrieve the current input handle */
    hInput = ConStreamGetOSHandle(&raw mut csStdIn);
    if hInput == INVALID_HANDLE_VALUE {
        ConResPrintf(
            &raw mut csStdErr,
            IDS_ERROR_INVALID_HANDLE_VALUE,
            GetLastError(),
        );
        return (EXIT_FAILURE) as INT;
    }

    /* Start a new wait if we use the timer */
    if bNoBreak != 0 && bUseTimer != 0 {
        hTimer = CreateWaitableTimer(
            core::ptr::null_mut(),
            (TRUE) as i32,
            core::ptr::null(),
        );
        if hTimer.is_null() {
            /* A problem happened, bail out */
            PrintError(GetLastError());
            return (EXIT_FAILURE) as INT;
        }
    }
    if bUseTimer != 0 {
        dwStartTime = GetTickCount();
    }

    /* If /NOBREAK is used, monitor for Ctrl-C input */
    if bNoBreak != 0 {
        SetConsoleCtrlHandler(Some(CtrlCIntercept), (TRUE) as i32);
    }

    'quit: {
        /* Initially flush the console input queue to remove any pending events */
        if GetNumberOfConsoleInputEvents(hInput, &raw mut NumRecords) == 0
            || FlushConsoleInputBuffer(hInput) == 0
        {
            /* A problem happened, bail out */
            PrintError(GetLastError());
            Status = (EXIT_FAILURE as i32);
            break 'quit;
        }

        ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        /* If the timer is not used, just show the message */
        if bUseTimer == 0 {
            ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_43C66C260828.as_ptr());
            ConResPuts(&raw mut csStdOut, WaitMsgId);
        }

        'stop: {
            loop {
                /* Decrease the timer if we use it */
                if bUseTimer != 0 {
                    /*
                     * Compute how much time the previous operations took.
                     */
                    timeElapsed = GetTickCount().wrapping_sub(dwStartTime) as LONG;
                    if timeElapsed >= 1000 {
                        /* Increase dwStartTime by steps of 1 second */
                        timeElapsed /= 1000;
                        dwStartTime = dwStartTime.wrapping_add(
                            (1000 * timeElapsed) as DWORD,
                        );

                        if timeElapsed <= timerValue as LONG {
                            timerValue = (timerValue as LONG - timeElapsed) as INT;
                        } else {
                            timerValue = 0;
                        }

                        DisplayMsg = (TRUE) as i32;
                    }

                    if DisplayMsg != 0 {
                        ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_43C66C260828.as_ptr());
                        ConResPrintf(&raw mut csStdOut, WaitCountMsgId, timerValue);
                        ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_60D2AC7E566A.as_ptr());

                        DisplayMsg = (FALSE) as i32;
                    }

                    /* Stop when the timer reaches zero */
                    if timerValue <= 0 {
                        break;
                    }
                }

                /* If /NOBREAK is used, only allow Ctrl-C input which is handled by the console handler */
                if bNoBreak != 0 {
                    if bUseTimer != 0 {
                        let mut DueTime: LARGE_INTEGER = ::core::mem::zeroed();

                        /* We use the timer: use a passive wait of maximum 1 second */
                        timeElapsed = GetTickCount().wrapping_sub(dwStartTime) as LONG;
                        if timeElapsed < 1000 {
                            DueTime.QuadPart =
                                ((1000 - timeElapsed) as LONGLONG) * (-10000 as LONGLONG);
                            SetWaitableTimer(
                                hTimer,
                                &raw mut DueTime,
                                0,
                                None,
                                core::ptr::null_mut(),
                                (FALSE) as BOOL,
                            );
                            dwWaitState = WaitForSingleObject(hTimer, INFINITE);

                            /* Check whether the timer has been signaled */
                            if dwWaitState != WAIT_OBJECT_0 {
                                /* An error happened, bail out */
                                PrintError(GetLastError());
                                Status = (EXIT_FAILURE as i32);
                                break;
                            }
                        }
                    } else {
                        /* No timer is used: wait indefinitely */
                        Sleep(INFINITE);
                    }

                    continue;
                }

                /* /NOBREAK is not used, check for user key presses */
                if bUseTimer != 0 {
                    /* Wait a maximum of 1 second for input events */
                    timeElapsed = GetTickCount().wrapping_sub(dwStartTime) as LONG;
                    if timeElapsed < 1000 {
                        dwWaitState = WaitForSingleObject(
                            hInput,
                            (1000 - timeElapsed) as DWORD,
                        );
                    } else {
                        dwWaitState = WAIT_TIMEOUT;
                    }

                    /* Check whether the input event has been signaled, or a timeout happened */
                    if dwWaitState == WAIT_TIMEOUT {
                        continue;
                    }
                    if dwWaitState != WAIT_OBJECT_0 {
                        /* An error happened, bail out */
                        PrintError(GetLastError());
                        Status = (EXIT_FAILURE as i32);
                        break;
                    }

                    /* Be sure there is something in the console input queue */
                    if PeekConsoleInputW(
                        hInput,
                        &raw mut InputRecords as PINPUT_RECORD,
                        ARRAYSIZE!(InputRecords),
                        &raw mut NumRecords,
                    ) == 0
                    {
                        /* An error happened, bail out */
                        ConResPrintf(
                            &raw mut csStdErr,
                            IDS_ERROR_READ_INPUT,
                            GetLastError(),
                        );
                        Status = (EXIT_FAILURE as i32);
                        break;
                    }

                    if NumRecords == 0 {
                        continue;
                    }
                }

                /*
                 * Some events have been detected, pop them out from the input queue.
                 * In case we do not use the timer, wait indefinitely until an input
                 * event appears.
                 */
                if ReadConsoleInputW(
                    hInput,
                    &raw mut InputRecords as PINPUT_RECORD,
                    ARRAYSIZE!(InputRecords),
                    &raw mut NumRecords,
                ) == 0
                {
                    /* An error happened, bail out */
                    ConResPrintf(
                        &raw mut csStdErr,
                        IDS_ERROR_READ_INPUT,
                        GetLastError(),
                    );
                    Status = (EXIT_FAILURE as i32);
                    break;
                }

                /* Check the input events for a key press */
                i = 0 as ULONG;
                while i < NumRecords {
                    'next: {
                        /* Ignore any non-key event */
                        if InputRecords[i as usize].EventType as ::core::ffi::c_int
                            != (KEY_EVENT as i32)
                        {
                            break 'next;
                        }

                        /* Ignore any system key event */
                        if (InputRecords[i as usize].Event.KeyEvent.wVirtualKeyCode
                            as ::core::ffi::c_int
                            == (VK_CONTROL as i32))
                            || (InputRecords[i as usize].Event.KeyEvent.wVirtualKeyCode
                                as ::core::ffi::c_int
                                == (VK_MENU as i32))
                        {
                            break 'next;
                        }

                        /* This is a non-system key event, stop waiting */
                        break 'stop;
                    }
                    i = i.wrapping_add(1);
                }
            }
        }

        ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    }

    /* Quit: */
    if bNoBreak != 0 {
        SetConsoleCtrlHandler(None, (FALSE) as i32);
    }

    if bNoBreak != 0 && bUseTimer != 0 {
        CloseHandle(hTimer);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn wmain(mut argc: ::core::ffi::c_int, mut argv: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut timerValue: INT = -1;
    let mut pszNext: PWCHAR = core::ptr::null_mut();
    let mut bDisableInput: BOOL = (FALSE) as i32;
    let mut fTimerFlags: BOOL = 0;
    let mut index: ::core::ffi::c_int = 0;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    if argc == 1 {
        ConResPrintf(StdOut, IDS_USAGE);
        return (EXIT_SUCCESS as i32);
    }

    /* Parse the command line for options */
    index = 1;
    while index < argc {
        'body: {
            if *(*argv.offset(index as isize)).offset(0 as isize) == 0x002Du16
                || *(*argv.offset(index as isize)).offset(0 as isize) == 0x002Fu16
            {
                match (towupper(
                    (*(*argv.offset(index as isize)).offset(1 as isize) as u32) as i32,
                ) as ::core::ffi::c_int) as u16
                {
                    0x003Fu16 => {
                        /* Help */
                        ConResPrintf(StdOut, IDS_USAGE);
                        return (EXIT_SUCCESS as i32);
                    }

                    0x0054u16 => {
                        /* Timer */
                        /* Consecutive /T switches are invalid */
                        if (fTimerFlags & 2) != 0 {
                            ConResPrintf(StdErr, IDS_ERROR_ONE_TIME);
                            return (EXIT_FAILURE as i32);
                        }

                        /* Remember that a /T switch has been encountered */
                        fTimerFlags |= 2;

                        /* Go to the next (timer) value */
                        break 'body;
                    }

                    _ => {}
                }

                /* This flag is used to ignore any keyboard keys but Ctrl-C */
                if _wcsicmp(
                    (*argv.offset(index as isize)).offset(1 as isize),
                    RUSTCONV_WSTR_D55BBD9863B9.as_ptr(),
                ) == 0
                {
                    bDisableInput = (TRUE) as i32;

                    /* Go to next value */
                    break 'body;
                }
            }

            /* The timer value can also be specified without the /T switch */

            /* Only one timer value is supported */
            if (fTimerFlags & 1) != 0 {
                ConResPrintf(StdErr, IDS_ERROR_ONE_TIME);
                return (EXIT_FAILURE as i32);
            }

            timerValue = wcstol(
                *argv.offset(index as isize),
                core::ptr::addr_of_mut!(pszNext),
                10,
            ) as INT;
            if *pszNext != 0 {
                ConResPrintf(StdErr, IDS_ERROR_OUT_OF_RANGE);
                return (EXIT_FAILURE as i32);
            }

            /* Remember that the timer value has been set */
            fTimerFlags |= 1;
        }
        index += 1;
    }

    /* A timer value is mandatory in order to continue */
    if (fTimerFlags & 1) == 0 {
        ConResPrintf(StdErr, IDS_ERROR_NO_TIMER_VALUE);
        return (EXIT_FAILURE as i32);
    }

    /* Make sure the timer value is within range */
    if (timerValue < -1) || (timerValue > 99999) {
        ConResPrintf(StdErr, IDS_ERROR_OUT_OF_RANGE);
        return (EXIT_FAILURE as i32);
    }

    return InputWait(bDisableInput, timerValue) as ::core::ffi::c_int;
}
