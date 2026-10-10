use crate::shutdown::misc::DisplayError;
use crate::shutdown::misc::ParseReasonCode;
use crate::shutdown::gui::ShutdownGuiMain;
use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn ParseArguments(pOpts: *mut CommandLineOptions, argc: ::core::ffi::c_int, argv: *mut *mut WCHAR) -> DWORD {
    let mut index: ::core::ffi::c_int;

    if pOpts.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    /* Reset all flags in struct */
    (*pOpts).abort = (FALSE) as i32;
    (*pOpts).force = (FALSE) as i32;
    (*pOpts).logoff = (FALSE) as i32;
    (*pOpts).restart = (FALSE) as i32;
    (*pOpts).shutdown = (FALSE) as i32;
    (*pOpts).document_reason = (FALSE) as i32;
    (*pOpts).hibernate = (FALSE) as i32;
    (*pOpts).shutdown_delay = 30;
    (*pOpts).remote_system = core::ptr::null_mut();
    (*pOpts).reason = ParseReasonCode(core::ptr::null::<WCHAR>()); /* NOTE: NEVER use 0 here since it can delay the shutdown. */
    (*pOpts).message = core::ptr::null_mut();
    (*pOpts).show_gui = (FALSE) as i32;

    /*
     * Determine which flags the user has specified
     * to the program so we can use them later.
     */
    index = 1;
    while index < argc {
        if *(*argv.offset(index as isize)).offset(0 as isize) == 0x002Du16
            || *(*argv.offset(index as isize)).offset(0 as isize) == 0x002Fu16
        {
            match towlower(
                (*(*argv.offset(index as isize)).offset(1 as isize) as u32) as u16,
            ) as u16
            {
                0x003Fu16 => {
                    /* Help */
                    ConResPuts(core::ptr::addr_of_mut!(csStdOut), IDS_USAGE);
                    return ERROR_SUCCESS;
                }
                0x0061u16 => {
                    /* Cancel delayed shutdown */
                    (*pOpts).abort = (TRUE) as i32;
                }
                0x0063u16 => {
                    /* Comment on reason for shutdown */
                    if index + 1 >= argc {
                        return ERROR_INVALID_DATA;
                    }
                    if (*argv.offset((index + 1) as isize)).is_null()
                        || wcslen(*argv.offset((index + 1) as isize))
                            <= 512
                    {
                        (*pOpts).message = *argv.offset((index + 1) as isize);
                        index += 1;
                    } else {
                        ConResPuts(core::ptr::addr_of_mut!(csStdErr), IDS_ERROR_MAX_COMMENT_LENGTH);
                        return ERROR_BAD_LENGTH;
                    }
                }
                0x0064u16 => {
                    /* Reason code [p|u:]xx:yy */
                    if index + 1 >= argc {
                        return ERROR_INVALID_DATA;
                    }
                    (*pOpts).reason =
                        ParseReasonCode(*argv.offset((index + 1) as isize));
                    index += 1;
                }
                0x0065u16 => {
                    /* Documents reason for shutdown */
                    /* TODO: Determine what this flag does exactly. */
                    (*pOpts).document_reason = (TRUE) as i32;
                }
                0x0066u16 => {
                    /* Force shutdown without warning */
                    (*pOpts).force = (TRUE) as i32;
                }
                0x0068u16 => {
                    /* Hibernate the local computer */
                    (*pOpts).hibernate = (TRUE) as i32;
                }
                0x0069u16 => {
                    /* Shows GUI version of the tool */
                    (*pOpts).show_gui = (TRUE) as i32;
                }
                0x006Cu16 => {
                    /* Logoff the current user */
                    (*pOpts).logoff = (TRUE) as i32;
                }
                0x006Du16 => {
                    /* Target remote systems (UNC name/IP address) */
                    if index + 1 >= argc {
                        return ERROR_INVALID_DATA;
                    }
                    (*pOpts).remote_system = *argv.offset((index + 1) as isize);
                    index += 1;
                }
                0x0070u16 => {
                    /* Turn off local computer with no warning/time-out */
                    (*pOpts).force = (TRUE) as i32;
                    (*pOpts).shutdown_delay = 0;
                }
                0x0072u16 => {
                    /* Restart computer */
                    (*pOpts).restart = (TRUE) as i32;
                }
                0x0073u16 => {
                    /* Shutdown */
                    (*pOpts).shutdown = (TRUE) as i32;
                }
                0x0074u16 => {
                    /* Shutdown delay */
                    if index + 1 >= argc {
                        return ERROR_INVALID_DATA;
                    }
                    (*pOpts).shutdown_delay =
                        _wtoi(*argv.offset((index + 1) as isize)) as DWORD;
                    if (*pOpts).shutdown_delay > 0 {
                        (*pOpts).force = (TRUE) as i32;
                    }
                    index += 1;
                }
                _ => {
                    /* Unknown arguments will exit the program. */
                    ConResPuts(core::ptr::addr_of_mut!(csStdOut), IDS_USAGE);
                    return ERROR_SUCCESS;
                }
            }
        }
        index += 1;
    }

    return ERROR_SUCCESS;
}

unsafe extern "C" fn EnablePrivilege(lpszPrivilegeName: LPCWSTR, bEnablePrivilege: BOOL) -> DWORD {
    let mut dwRet: DWORD = ERROR_SUCCESS as DWORD;
    let mut hToken: HANDLE = core::ptr::null_mut();

    if OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_ADJUST_PRIVILEGES as DWORD,
        &raw mut hToken,
    ) != 0
    {
        let mut tp: TOKEN_PRIVILEGES = core::mem::zeroed();

        tp.PrivilegeCount = 1 as DWORD;
        tp.Privileges[0 as usize].Attributes = (if bEnablePrivilege != 0 {
            SE_PRIVILEGE_ENABLED
        } else {
            0
        }) as DWORD;

        if LookupPrivilegeValueW(
            core::ptr::null::<WCHAR>(),
            lpszPrivilegeName,
            &raw mut tp.Privileges[0 as usize].Luid,
        ) != 0
        {
            if AdjustTokenPrivileges(
                hToken,
                FALSE as BOOL,
                &raw mut tp,
                0 as DWORD,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) != 0
            {
                if GetLastError() == ERROR_NOT_ALL_ASSIGNED as DWORD {
                    dwRet = ERROR_NOT_ALL_ASSIGNED as DWORD;
                }
            } else {
                dwRet = GetLastError();
            }
        } else {
            dwRet = GetLastError();
        }

        CloseHandle(hToken);
    } else {
        dwRet = GetLastError();
    }

    /* Display the error description if any */
    if dwRet != ERROR_SUCCESS as DWORD {
        DisplayError(dwRet);
    }

    return dwRet;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: ::core::ffi::c_int, argv: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut error: DWORD = ERROR_SUCCESS;
    let mut opts: CommandLineOptions = CommandLineOptions {
        abort: 0,
        force: 0,
        logoff: 0,
        restart: 0,
        shutdown: 0,
        document_reason: 0,
        hibernate: 0,
        shutdown_delay: 0,
        remote_system: core::ptr::null_mut::<WCHAR>(),
        message: core::ptr::null_mut::<WCHAR>(),
        reason: 0,
        show_gui: 0,
    };

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    if argc == 1 {
        /* i.e. no commandline arguments given */
        ConResPuts(&raw mut csStdOut, IDS_USAGE);
        return (EXIT_SUCCESS as i32);
    }

    error = ParseArguments(&raw mut opts, argc, argv);
    if error != ERROR_SUCCESS {
        DisplayError(error);
        return (EXIT_FAILURE as i32);
    }

    /* If the user wants to abort a shutdown */
    if opts.abort != 0 {
        /* First, the program has to determine if the shutdown/restart is local
        or remote. This is done since each one requires separate privileges. */
        if opts.remote_system.is_null() {
            EnablePrivilege(SE_SHUTDOWN_NAME, (TRUE) as BOOL);
        } else {
            EnablePrivilege(SE_REMOTE_SHUTDOWN_NAME, (TRUE) as BOOL);
        }

        /* Abort the delayed system shutdown specified. */
        if AbortSystemShutdownW(opts.remote_system) == 0 {
            ConResPuts(&raw mut csStdErr, IDS_ERROR_ABORT);
            DisplayError(GetLastError());
            return (EXIT_FAILURE as i32);
        } else {
            return (EXIT_SUCCESS as i32);
        }
    }

    /*
     * If the user wants to hibernate the computer. Assume
     * that the user wants to wake the computer up from
     * hibernation and it should not force it on the system.
     */
    if opts.hibernate != 0 {
        if IsPwrHibernateAllowed() != 0 {
            EnablePrivilege(SE_SHUTDOWN_NAME, (TRUE) as BOOL);

            /* The shutdown utility cannot hibernate remote systems */
            if !opts.remote_system.is_null() {
                return (EXIT_FAILURE as i32);
            }

            if SetSuspendState((TRUE) as BOOLEAN, (FALSE) as BOOLEAN, (FALSE) as BOOLEAN) == 0 {
                ConResPuts(&raw mut csStdErr, IDS_ERROR_HIBERNATE);
                DisplayError(GetLastError());
                return (EXIT_FAILURE as i32);
            } else {
                ConResPuts(&raw mut csStdOut, IDS_ERROR_HIBERNATE_ENABLED);
                return (EXIT_SUCCESS as i32);
            }
        } else {
            return (EXIT_FAILURE as i32);
        }
    }

    /* Both shutdown and restart flags cannot both be true */
    if opts.shutdown != 0 && opts.restart != 0 {
        ConResPuts(&raw mut csStdErr, IDS_ERROR_SHUTDOWN_REBOOT);
        return (EXIT_FAILURE as i32);
    }

    /* Ensure that the timeout amount is not too high or a negative number */
    if opts.shutdown_delay > MAX_SHUTDOWN_TIMEOUT {
        ConResPrintf(&raw mut csStdErr, IDS_ERROR_TIMEOUT, opts.shutdown_delay);
        return (EXIT_FAILURE as i32);
    }

    /* If the user wants a GUI environment */
    if opts.show_gui != 0 {
        if ShutdownGuiMain(opts) != 0 {
            return (EXIT_SUCCESS as i32);
        } else {
            return (EXIT_FAILURE as i32);
        }
    }

    if opts.logoff != 0 && opts.remote_system.is_null() {
        /*
         * NOTE: Sometimes, shutdown and logoff are used together. If the logoff
         * flag is used by itself, then simply logoff. But if used with shutdown,
         * then skip logging off of the computer and eventually go to the action
         * for shutdown.
         */
        if opts.shutdown == 0 && opts.restart == 0 {
            EnablePrivilege(SE_SHUTDOWN_NAME, (TRUE) as BOOL);

            if ExitWindowsEx(EWX_LOGOFF, opts.reason) != 0 {
                return (EXIT_SUCCESS as i32);
            } else {
                ConResPuts(&raw mut csStdErr, IDS_ERROR_LOGOFF);
                DisplayError(GetLastError());
                return (EXIT_FAILURE as i32);
            }
        }
    }

    /*
     * Since both shutting down the system and restarting calls the exact same
     * function, all we need to know is if we wanted to restart or shutdown.
     */
    if opts.shutdown != 0 || opts.restart != 0 {
        /*
         * First, the program has to determine if the shutdown/restart is local
         * or remote. This is done since each one requires separate privileges.
         */
        if opts.remote_system.is_null() {
            EnablePrivilege(SE_SHUTDOWN_NAME, (TRUE) as BOOL);
        } else {
            /* TODO: Remote shutdown is not supported yet */
            // EnablePrivilege(SE_REMOTE_SHUTDOWN_NAME, TRUE);
            return (EXIT_SUCCESS as i32);
        }

        /* Initiate the shutdown */
        if InitiateSystemShutdownExW(
            opts.remote_system,
            opts.message,
            opts.shutdown_delay,
            opts.force,
            opts.restart,
            opts.reason,
        ) == 0
        {
            /*
             * If there is an error, give the proper output depending
             * on whether the user wanted to shutdown or restart.
             */
            if opts.restart != 0 {
                ConResPuts(&raw mut csStdErr, IDS_ERROR_RESTART);
            } else {
                ConResPuts(&raw mut csStdErr, IDS_ERROR_SHUTDOWN);
            }

            DisplayError(GetLastError());
            return (EXIT_FAILURE as i32);
        } else {
            return (EXIT_SUCCESS as i32);
        }
    }

    return (EXIT_SUCCESS as i32);
}
