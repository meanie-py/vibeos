use crate::prelude::*;
use crate::rapps::geninst_cpp_h::ExtractAndRunGeneratedInstaller;
use crate::rapps::gui_cpp_h::MainWindowLoop;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_075DE2B906DB: [WCHAR; 2] = [0x000A, 0x0000];
pub static CPP_COM_WSTR_481F1A6D4120: [WCHAR; 5] = [0x0025, 0x006C, 0x0073, 0x000A, 0x0000];
pub static CPP_COM_WSTR_6F6F9C60D430: [WCHAR; 8] = [0x0049, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_949D86BB7DFA: [WCHAR; 3] = [0x002F, 0x004B, 0x0000];
pub static CPP_COM_WSTR_9D313EA1711F: [WCHAR; 3] = [0x000A, 0x000A, 0x0000];
pub static CPP_COM_WSTR_B42E8543313E: [WCHAR; 3] = [0x002F, 0x0053, 0x0000];
pub static CPP_COM_WSTR_C890C70025FA: [WCHAR; 9] = [0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x000A, 0x0000];
pub static CPP_COM_WSTR_CBCB2E783EF5: [WCHAR; 8] = [0x0052, 0x0041, 0x0050, 0x0050, 0x0057, 0x0049, 0x005A, 0x0000];
pub static CPP_COM_WSTR_ED70ADC50169: [WCHAR; 6] = [0x0052, 0x0041, 0x0050, 0x0050, 0x0053, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn MatchCmdOption(argvOption: LPWSTR, szOptToMacth: LPCWSTR) -> BOOL {
    let FirstCharList: [u16; 2] = [0x002Du16, 0x002Fu16];

    let mut i: u32 = 0;
    while i < _countof!(FirstCharList) as u32 {
        if *argvOption.add(0 as usize) == FirstCharList[i as usize] {
            return (StrCmpIW(argvOption.add(1 as usize), szOptToMacth) == 0) as BOOL;
        }
        i += 1;
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn InitRappsConsole() {
    // First, try to attach to our parent's console
    if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
        // Did we already have a console?
        if GetLastError() != ERROR_ACCESS_DENIED {
            // No, try to open a new one
            AllocConsole();
        }
    }
    ConInitStdStreams(); // Initialize the Console Standard Streams
}

unsafe extern "C" fn SearchForAppWithDisplayName(db: &mut CAppDB, Type: AppsCategories, Name: LPCWSTR) -> *mut CAppInfo {
    let mut List: CAtlList<*mut CAppInfo> = CAtlList::new();
    (*db).GetApps(&mut List, Type);
    let mut it = List.GetHeadPosition();
    while !(it as usize == 0) {
        let Info: *mut CAppInfo = *List.GetNext(&mut it);
        if ((SearchPatternMatch((&(*Info).szDisplayName).GetString(), Name)) != 0) {
            return Info;
        }
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn HandleInstallCommand(db: *mut CAppDB, szCommand: LPWSTR, mut argcLeft: i32, mut argvLeft: *mut LPWSTR) -> i32 {
    let mut flags: u32 = DAF_MODAL;
    if argcLeft >= 1 && !(StrCmpIW(CPP_COM_WSTR_B42E8543313E.as_ptr(), *argvLeft.add(0 as usize)) == 0) {
        flags |= DAF_SILENT;
        argvLeft = argvLeft.add(1 as usize);
        argcLeft -= 1;
    }

    if argcLeft < 1 {
        InitRappsConsole();
        ConResMsgPrintf(core::ptr::addr_of_mut!(csStdOut), 0, IDS_CMD_NEED_PACKAGE_NAME, szCommand);
        return (FALSE as i32);
    }

    let mut Applications: CAtlList<*mut CAppInfo> = CAtlList::new();
    let mut i: i32 = 0;
    while i < argcLeft {
        let PackageName: LPCWSTR = *argvLeft.add(i as usize);
        let AppInfo: *mut CAppInfo = (*db).FindByPackageName(&CStringW::from_ptr(PackageName));
        if !AppInfo.is_null() {
            Applications.AddTail(AppInfo);
        }
        i += 1;
    }

    return DownloadListOfApplications(&Applications, flags);
}

unsafe extern "C" fn HandleSetupCommand(db: *mut CAppDB, szCommand: LPWSTR, mut argcLeft: i32, mut argvLeft: *mut LPWSTR) -> BOOL {
    let mut flags: UINT = DAF_MODAL;
    if argcLeft >= 1 && StrCmpIW(CPP_COM_WSTR_B42E8543313E.as_ptr(), *argvLeft.add(0 as usize)) == 0
    {
        flags |= DAF_SILENT;
        argvLeft = argvLeft.add(1 as usize);
        argcLeft -= 1;
    }

    if argcLeft != 1
    {
        InitRappsConsole();
        ConResMsgPrintf(core::ptr::addr_of_mut!(csStdOut), 0, IDS_CMD_NEED_FILE_NAME, szCommand);
        return (FALSE) as BOOL;
    }

    let mut Applications: CAtlList<*mut CAppInfo> = CAtlList::new();
    let InfHandle: HINF = SetupOpenInfFileW(*argvLeft.add(0 as usize), core::ptr::null_mut(), INF_STYLE_WIN4, core::ptr::null_mut());
    if InfHandle == INVALID_HANDLE_VALUE
    {
        return (FALSE) as BOOL;
    }

    let mut Context: INFCONTEXT = core::mem::zeroed();
    if SetupFindFirstLineW(InfHandle, CPP_COM_WSTR_ED70ADC50169.as_ptr(), CPP_COM_WSTR_6F6F9C60D430.as_ptr(), core::ptr::addr_of_mut!(Context)) != 0
    {
        let mut szPkgName: [u16; (MAX_PATH) as _] = [0u16; (MAX_PATH) as _];
        loop
        {
            if SetupGetStringFieldW(core::ptr::addr_of_mut!(Context), 1, szPkgName.as_mut_ptr(), _countof!(szPkgName), core::ptr::null_mut()) != 0
            {
                let AppInfo: *mut CAppInfo = (*db).FindByPackageName(&CStringW::from_ptr(szPkgName.as_ptr()));
                if !AppInfo.is_null()
                {
                    Applications.AddTail(AppInfo);
                }
            }
            if !(SetupFindNextLine(core::ptr::addr_of_mut!(Context), core::ptr::addr_of_mut!(Context)) != 0) {
                break;
            }
        }
    }
    SetupCloseInfFile(InfHandle);

    return DownloadListOfApplications(&Applications, flags);
}

unsafe extern "C" fn HandleUninstallCommand(db: &mut CAppDB, argcLeft: UINT, argvLeft: *mut LPWSTR) -> BOOL {
    let mut argcLeft: UINT = argcLeft;
    let mut argi: UINT = 0;
    let mut silent: UINT = FALSE;
    let mut byregkeyname: UINT = FALSE;
    while argcLeft != 0 {
        if StrCmpIW(*argvLeft.add(argi as usize), CPP_COM_WSTR_B42E8543313E.as_ptr()) == 0 {
            silent += 1;
        } else if StrCmpIW(*argvLeft.add(argi as usize), CPP_COM_WSTR_949D86BB7DFA.as_ptr()) == 0 {
            byregkeyname += 1;
        } else {
            break;
        }
        argi += 1;
        argcLeft -= 1;
    }
    if argcLeft != 1 {
        return (FALSE) as BOOL;
    }

    let mut buf = CStringW::new();
    let mut name: LPCWSTR = *argvLeft.add(argi as usize);
    let mut retval: BOOL = (FALSE as i32);
    let mut pInfo: *mut CAppInfo = core::ptr::null_mut();
    let mut pDelete: *mut CAppInfo = core::ptr::null_mut();

    if byregkeyname == 0 {
        let mut i: UINT = ENUM_INSTALLED_MIN;
        while pInfo.is_null() && i <= ENUM_INSTALLED_MAX {
            pInfo = SearchForAppWithDisplayName(db, i as AppsCategories, name);
            i += 1;
        }

        if pInfo.is_null() {
            let mut p: *mut CAvailableApplicationInfo =
                db.FindAvailableByPackageName(&CStringW::from_ptr(name));
            if !p.is_null() && (*p).IsInstalled(core::ptr::addr_of_mut!(buf)) {
                name = buf.GetString();
                byregkeyname = TRUE;
            }
        }
    }

    if byregkeyname != 0 {
        // Force a specific key type if requested (<M|U>[32|64]<\\KeyName>)
        if *name.add(0 as usize) != 0 {
            let mut wow: REGSAM = 0;
            let mut i: UINT = 1;
            if *name.add(i as usize) == '3' as u16 && *name.add((i + 1) as usize) != 0 {
                wow = KEY_WOW64_32KEY;
                i += 2;
            } else if *name.add(i as usize) == '6' as u16 && *name.add((i + 1) as usize) != 0 {
                wow = KEY_WOW64_64KEY;
                i += 2;
            }

            let __idx: UINT = i;
            i += 1;
            if *name.add(__idx as usize) == '\\' as u16 {
                pInfo = core::ptr::addr_of_mut!(
                    (*CAppDB::CreateInstalledAppInstance(
                        name.add(i as usize),
                        (*name.add(0 as usize) == 'U' as u16) as i32,
                        wow,
                    ))
                    .__base0
                );
            }
        }

        if pInfo.is_null() {
            pInfo = core::ptr::addr_of_mut!(
                (*CAppDB::CreateInstalledAppByRegistryKey__1(name)).__base0
            );
        }
        pDelete = pInfo;
    }

    if !pInfo.is_null() {
        retval = (*pInfo).UninstallApplication(
            (if silent != 0 { UCF_SILENT } else { UCF_NONE })
                | UCF_SAMEPROCESS,
        );
    }
    CAppInfo__delete(pDelete);
    retval
}

unsafe extern "C" fn HandleGenerateInstallerCommand(db: &mut CAppDB, mut argcLeft: UINT, mut argvLeft: *mut LPWSTR) -> BOOL {
    let mut bSilent: bool = false;
    if argcLeft != 0 && StrCmpIW(*argvLeft.add(0 as usize), CPP_COM_WSTR_B42E8543313E.as_ptr()) == 0
    {
        bSilent = true;
        argcLeft -= 1;
        argvLeft = argvLeft.add(1);
    }

    if argcLeft != 2 {
        return (FALSE) as BOOL;
    }

    let pAI: *mut CAvailableApplicationInfo = db.FindAvailableByPackageName(&CStringW::from_ptr(*argvLeft.add(0 as usize)));
    if pAI.is_null() {
        return (FALSE) as BOOL;
    }

    return ExtractAndRunGeneratedInstaller(&mut *pAI, *argvLeft.add(1 as usize), bSilent);
}

unsafe extern "C" fn HandleFindCommand(db: *mut CAppDB, szCommand: LPWSTR, argcLeft: i32, argvLeft: *mut LPWSTR) -> BOOL {
    if argcLeft < 1 {
        ConResMsgPrintf(core::ptr::addr_of_mut!(csStdOut), 0, IDS_CMD_NEED_PARAMS, szCommand);
        return (FALSE) as BOOL;
    }

    let mut List: CAtlList<*mut CAppInfo> = CAtlList::new();
    (*db).GetApps(&mut List, ENUM_ALL_AVAILABLE);

    let mut i: i32 = 0;
    while i < argcLeft {
        let lpszSearch: LPCWSTR = *argvLeft.offset(i as isize) as LPCWSTR;
        ConResMsgPrintf(core::ptr::addr_of_mut!(csStdOut), 0, IDS_CMD_FIND_RESULT_FOR, lpszSearch);

        let mut CurrentListPosition: POSITION = List.GetHeadPosition();
        while !CurrentListPosition.is_null() {
            let Info: *mut CAppInfo = *List.GetNext(&mut CurrentListPosition);

            if SearchPatternMatch((&(*Info).szDisplayName).GetString(), lpszSearch) != 0 || SearchPatternMatch((&(*Info).szComments).GetString(), lpszSearch) != 0 {
                ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_C890C70025FA.as_ptr(), (*Info).szDisplayName.GetString(), (*Info).szIdentifier.GetString());
            }
        }

        ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_075DE2B906DB.as_ptr());
        i += 1;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn HandleInfoCommand(db: *mut CAppDB, szCommand: LPWSTR, argcLeft: i32, argvLeft: *mut LPWSTR) -> BOOL {
    if argcLeft < 1 {
        ConResMsgPrintf(core::ptr::addr_of_mut!(csStdOut), 0, IDS_CMD_NEED_PARAMS, szCommand);
        return (FALSE) as BOOL;
    }

    let mut i: i32 = 0;
    while i < argcLeft {
        let PackageName: LPCWSTR = *argvLeft.offset(i as isize);
        let AppInfo: *mut CAppInfo = (*db).FindByPackageName(&CStringW::from_ptr(PackageName));
        if AppInfo.is_null() {
            ConResMsgPrintf(core::ptr::addr_of_mut!(csStdOut), 0, IDS_CMD_PACKAGE_NOT_FOUND, PackageName);
        } else {
            ConResMsgPrintf(core::ptr::addr_of_mut!(csStdOut), 0, IDS_CMD_PACKAGE_INFO, PackageName);

            ConPuts(core::ptr::addr_of_mut!(csStdOut), (&(*AppInfo).szDisplayName).GetString());

            if !(*AppInfo).szDisplayVersion.IsEmpty() {
                ConResPrintf(core::ptr::addr_of_mut!(csStdOut), IDS_AINFO_VERSION);
                ConPuts(core::ptr::addr_of_mut!(csStdOut), (&(*AppInfo).szDisplayVersion).GetString());
            }

            let mut License = CStringW::new();
            let mut Size = CStringW::new();
            let mut UrlSite = CStringW::new();
            let mut UrlDownload = CStringW::new();
            (*AppInfo).GetDisplayInfo(&mut License, &mut Size, &mut UrlSite, &mut UrlDownload);

            if !License.IsEmpty() {
                ConResPrintf(core::ptr::addr_of_mut!(csStdOut), IDS_AINFO_LICENSE);
                ConPuts(core::ptr::addr_of_mut!(csStdOut), (&License).GetString());
            }

            if !Size.IsEmpty() {
                ConResPrintf(core::ptr::addr_of_mut!(csStdOut), IDS_AINFO_SIZE);
                ConPuts(core::ptr::addr_of_mut!(csStdOut), (&Size).GetString());
            }

            if !UrlSite.IsEmpty() {
                ConResPrintf(core::ptr::addr_of_mut!(csStdOut), IDS_AINFO_URLSITE);
                ConPuts(core::ptr::addr_of_mut!(csStdOut), (&UrlSite).GetString());
            }

            if !(*AppInfo).szComments.GetString().is_null() {
                ConResPrintf(core::ptr::addr_of_mut!(csStdOut), IDS_AINFO_DESCRIPTION);
                ConPuts(core::ptr::addr_of_mut!(csStdOut), (&(*AppInfo).szComments).GetString());
            }

            if !UrlDownload.IsEmpty() {
                ConResPrintf(core::ptr::addr_of_mut!(csStdOut), IDS_AINFO_URLDOWNLOAD);
                ConPuts(core::ptr::addr_of_mut!(csStdOut), (&UrlDownload).GetString());
            }
            ConPuts(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_075DE2B906DB.as_ptr());
        }
        ConPuts(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_075DE2B906DB.as_ptr());
        i += 1;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn PrintHelpCommand() {
    ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_075DE2B906DB.as_ptr());
    ConResPuts(core::ptr::addr_of_mut!(csStdOut), IDS_APPTITLE);
    ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_9D313EA1711F.as_ptr());

    ConResPuts(core::ptr::addr_of_mut!(csStdOut), IDS_CMD_USAGE);
    ConPrintf(core::ptr::addr_of_mut!(csStdOut), CPP_COM_WSTR_481F1A6D4120.as_ptr(), UsageString.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn ParseCmdAndExecute(lpCmdLine: LPWSTR, bIsFirstLaunch: BOOL, nCmdShow: i32) -> BOOL {
    let mut argc: INT = 0;
    let argv: *mut LPWSTR = CommandLineToArgvW(lpCmdLine, core::ptr::addr_of_mut!(argc));
    if argv.is_null() {
        return (FALSE) as BOOL;
    }

    let mut db = CAppDB__zeroed();
    db.__ctor_0(&CAppDB::GetDefaultPath());

    let bAppwizMode: BOOL = (argc > 1
        && MatchCmdOption(*argv.add(1 as usize), CMD_KEY_APPWIZ.as_ptr()) != 0) as BOOL;
    if bAppwizMode == 0 {
        let mut lock = CUpdateDatabaseMutex__make_0();
        if SettingsInfo.bUpdateAtStart != 0 || bIsFirstLaunch != 0 {
            db.RemoveCached();
        }

        db.UpdateAvailable();
    }

    db.UpdateInstalled();

    if argc == 1 || bAppwizMode != 0 {
        // Check whether the RAPPS MainWindow is already launched in another process
        let mut szWindowText = CStringW::from_ptr(MAKEINTRESOURCEW!(
            if bAppwizMode != 0 { IDS_APPWIZ_TITLE } else { IDS_APPTITLE }
        ));
        let pszMutex: LPCWSTR = if bAppwizMode != 0 {
            CPP_COM_WSTR_CBCB2E783EF5.as_ptr()
        } else {
            szWindowClass
        };

        let hMutex: HANDLE = CreateMutexW(core::ptr::null_mut(), (FALSE) as BOOL, pszMutex);
        if hMutex.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            /* If already started, find its window */
            let mut hWindow: HWND = core::ptr::null_mut();
            let mut wait: i32 = 2500;
            let mut inter: i32 = 250;
            while wait > 0 {
                hWindow = FindWindowW(szWindowClass, szWindowText.GetString());
                if !hWindow.is_null() {
                    break;
                }
                Sleep(inter as DWORD);
                wait -= inter;
            }

            if !hWindow.is_null() {
                /* Activate the window in the other instance */
                ShowWindow(hWindow, (SW_SHOWNA as i32));
                SwitchToThisWindow(hWindow, (TRUE) as BOOL);
                if bAppwizMode != 0 {
                    PostMessage(hWindow, WM_COMMAND, (ID_ACTIVATE_APPWIZ as _), 0);
                }

                if !hMutex.is_null() {
                    CloseHandle(hMutex);
                }

                return (FALSE) as BOOL;
            }
        }
        szWindowText.Empty();

        let mut wnd = CMainWindow__make_0(core::ptr::addr_of_mut!(db), bAppwizMode);
        MainWindowLoop((core::ptr::addr_of_mut!(wnd) as HWND).cast(), nCmdShow);

        if !hMutex.is_null() {
            CloseHandle(hMutex);
        }

        return (TRUE) as BOOL;
    }

    if MatchCmdOption(*argv.add(1 as usize), CMD_KEY_INSTALL.as_ptr()) != 0 {
        return HandleInstallCommand(core::ptr::addr_of_mut!(db), *argv.add(1 as usize), argc - 2, argv.add(2 as usize));
    } else if MatchCmdOption(*argv.add(1 as usize), CMD_KEY_SETUP.as_ptr()) != 0 {
        return HandleSetupCommand(core::ptr::addr_of_mut!(db), *argv.add(1 as usize), argc - 2, argv.add(2 as usize));
    } else if MatchCmdOption(*argv.add(1 as usize), CMD_KEY_UNINSTALL.as_ptr()) != 0 {
        return HandleUninstallCommand(&mut db, (argc - 2) as UINT, argv.add(2 as usize));
    } else if MatchCmdOption(*argv.add(1 as usize), CMD_KEY_GENINST.as_ptr()) != 0 {
        return HandleGenerateInstallerCommand(&mut db, (argc - 2) as UINT, argv.add(2 as usize));
    }

    InitRappsConsole();

    if MatchCmdOption(*argv.add(1 as usize), CMD_KEY_FIND.as_ptr()) != 0 {
        return HandleFindCommand(core::ptr::addr_of_mut!(db), *argv.add(1 as usize), argc - 2, argv.add(2 as usize));
    } else if MatchCmdOption(*argv.add(1 as usize), CMD_KEY_INFO.as_ptr()) != 0 {
        return HandleInfoCommand(core::ptr::addr_of_mut!(db), *argv.add(1 as usize), argc - 2, argv.add(2 as usize));
    } else if MatchCmdOption(*argv.add(1 as usize), CMD_KEY_HELP.as_ptr()) != 0
        || MatchCmdOption(*argv.add(1 as usize), CMD_KEY_HELP_ALT.as_ptr()) != 0
    {
        PrintHelpCommand();
        return (TRUE) as BOOL;
    } else {
        // unrecognized/invalid options
        ConResPuts(core::ptr::addr_of_mut!(csStdOut), IDS_CMD_INVALID_OPTION);
        PrintHelpCommand();
        return (FALSE) as BOOL;
    }
}
