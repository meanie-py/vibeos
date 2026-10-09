use crate::usetup::consup::CONSOLE_ClearScreen;
use crate::usetup::consup::CONSOLE_ConInKey;
use crate::usetup::consup::CONSOLE_ConInKeyPeek;
use crate::usetup::consup::CONSOLE_Flush;
use crate::usetup::consup::CONSOLE_Init;
use crate::usetup::consup::CONSOLE_InvertTextXY;
use crate::usetup::consup::CONSOLE_NormalTextXY;
use crate::usetup::consup::CONSOLE_PrintTextXY;
use crate::usetup::consup::CONSOLE_SetCursorType;
use crate::usetup::consup::CONSOLE_SetCursorXY;
use crate::usetup::consup::CONSOLE_SetInputTextXY;
use crate::usetup::consup::CONSOLE_SetStatusText;
use crate::usetup::consup::CONSOLE_SetStatusTextV;
use crate::usetup::consup::CONSOLE_SetTextXY;
use crate::usetup::fslist::CreateFileSystemList;
use crate::usetup::progress::CreateProgressBar;
use crate::usetup::progress::CreateProgressBarEx;
use crate::usetup::fslist::DestroyFileSystemList;
use crate::usetup::progress::DestroyProgressBar;
use crate::usetup::partlist::DiskDescription;
use crate::usetup::fslist::DrawFileSystemList;
use crate::usetup::genlist::DrawGenericList;
use crate::usetup::genlist::DrawGenericListCurrentItem;
use crate::usetup::partlist::DrawPartitionList;
use crate::usetup::devinst::EnableUserModePnpManager;
use crate::usetup::fmtchk::EndCheck;
use crate::usetup::fmtchk::EndFormat;
use crate::usetup::console::FillConsoleOutputAttribute;
use crate::usetup::console::FillConsoleOutputCharacterA;
use crate::usetup::console::FreeConsole;
use crate::usetup::genlist::GenericListKeyPress;
use crate::usetup::genlist::InitGenericListUi;
use crate::usetup::partlist::InitPartitionListUi;
use crate::usetup::devinst::InitializeUserModePnpManager;
use crate::usetup::mui::MUIClearPage;
use crate::usetup::mui::MUIClearStyledText;
use crate::usetup::mui::MUIDisplayError;
use crate::usetup::mui::MUIDisplayErrorV;
use crate::usetup::mui::MUIDisplayPage;
use crate::usetup::mui::MUIGetString;
use crate::usetup::partlist::PartitionDescription;
use crate::usetup::progress::ProgressNextStep;
use crate::usetup::progress::ProgressSetStep;
use crate::usetup::progress::ProgressSetStepCount;
use crate::usetup::cmdcons::RecoveryConsole;
use crate::usetup::genlist::RedrawGenericList;
use crate::usetup::genlist::RestoreGenericListUiState;
use crate::usetup::fslist::ScrollDownFileSystemList;
use crate::usetup::genlist::ScrollDownGenericList;
use crate::usetup::genlist::ScrollPageDownGenericList;
use crate::usetup::genlist::ScrollPageUpGenericList;
use crate::usetup::genlist::ScrollToPositionGenericList;
use crate::usetup::partlist::ScrollUpDownPartitionList;
use crate::usetup::fslist::ScrollUpFileSystemList;
use crate::usetup::genlist::ScrollUpGenericList;
use crate::usetup::mui::SetConsoleCodePage;
use crate::usetup::fmtchk::StartCheck;
use crate::usetup::fmtchk::StartFormat;
use crate::usetup::devinst::TerminateUserModePnpManager;
use crate::usetup::devinst::WaitNoPendingInstallEvents;
use crate::usetup::console::WriteConsoleOutputCharacterA;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_04F17891E646: [u16; 11] = [
    0x0062u16, 0x0061, 0x0074, 0x0061, 0x006E, 0x0067, 0x002E, 0x0074, 0x0074, 0x0063,
    0x0000,
];
pub static RUSTCONV_WSTR_0946A42E27B2: [u16; 13] = [
    0x006Du16, 0x0073, 0x006D, 0x0069, 0x006E, 0x0063, 0x0068, 0x006F, 0x002E, 0x0074,
    0x0074, 0x0063, 0x0000,
];
pub static RUSTCONV_WSTR_123946225403: [u16; 17] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0046, 0x006C,
    0x006F, 0x0070, 0x0070, 0x0079, 0x0030, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_28AC66B7B537: [u16; 9] = [0x005Cu16, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_603653C2EA23: [u16; 10] = [0x0067u16, 0x0075, 0x006C, 0x0069, 0x006D, 0x002E, 0x0074, 0x0074, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_840220BD5E07: [u16; 12] = [
    0x006Du16, 0x0069, 0x006E, 0x0067, 0x006C, 0x0069, 0x0075, 0x002E, 0x0074, 0x0074,
    0x0063, 0x0000,
];
pub static RUSTCONV_WSTR_99A57B2B061D: [u16; 11] = [
    0x006Du16, 0x0073, 0x0073, 0x006F, 0x006E, 0x0067, 0x002E, 0x0074, 0x0074, 0x0066,
    0x0000,
];
pub static RUSTCONV_WSTR_9CA37DD1B2AA: [u16; 11] = [
    0x0073u16, 0x0069, 0x006D, 0x0073, 0x0075, 0x006E, 0x002E, 0x0074, 0x0074, 0x0063,
    0x0000,
];
pub static RUSTCONV_WSTR_A953DDDD78B1: [u16; 4] = [0x0046u16, 0x0041, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_AC374BD06A63: [u16; 13] = [
    0x006Du16, 0x0073, 0x0067, 0x006F, 0x0074, 0x0068, 0x0069, 0x0063, 0x002E, 0x0074,
    0x0074, 0x0063, 0x0000,
];
pub static RUSTCONV_WSTR_BCF3913447D5: [u16; 4] = [0x0025u16, 0x0073, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_BE0DEB469A7B: [u16; 6] = [0x0042u16, 0x0054, 0x0052, 0x0046, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_DD8FED8124A3: [u16; 4] = [0x004Du16, 0x0042, 0x0052, 0x0000];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut ProcessHeap: HANDLE = core::ptr::null_mut();
pub static mut USetupData: USETUP_DATA = unsafe { core::mem::zeroed() };
pub static mut IsUnattendedSetup: BOOLEAN = 0;
pub static mut InstallPartition: PPARTENTRY = core::ptr::null_mut();
pub static mut SystemPartition: PPARTENTRY = core::ptr::null_mut();
pub static mut SelectedLanguageId: PCWSTR = core::ptr::null_mut();
pub static mut DefaultLanguage: [WCHAR; 20] = [0; 20];
pub static mut DefaultKBLayout: [WCHAR; 20] = [0; 20];
pub static mut RepairUpdateFlag: BOOLEAN = 0;
pub static mut PartitionList: PPARTLIST = core::ptr::null_mut();
pub static mut CurrentPartition: PPARTENTRY = core::ptr::null_mut();
pub const PartTypeData: i32 = 0;
pub const PartTypeExtended: i32 = 1;
pub static mut PartCreateType: i32 = PartTypeData;
pub const VOLUME_NEW_AUTOCREATE: i32 = 0x80;
pub static mut FileSystemList: PFILE_SYSTEM_LIST = core::ptr::null_mut();
pub static mut CurrentInstallation: PNTOS_INSTALLATION = core::ptr::null_mut();
pub static mut NtOsInstallsList: PGENERIC_LIST = core::ptr::null_mut();
pub static mut s_SubstSettings: FONTSUBSTSETTINGS = FONTSUBSTSETTINGS {
    bFoundFontMINGLIU: 0,
    bFoundFontMSGOTHIC: 0,
    bFoundFontMSMINCHO: 0,
    bFoundFontMSSONG: 0,
    bFoundFontSIMSUN: 0,
    bFoundFontGULIM: 0,
    bFoundFontBATANG: 0,
};
pub const PARTITION_SIZE_INPUT_FIELD_LENGTH: i32 = 9;
pub const PARTITION_MAXSIZE: f64 = 99_999_999.0;
#[repr(C)]
pub struct _FSVOL_CONTEXT {
    pub Ir: PINPUT_RECORD,
    pub NextPageOnAbort: PAGE_NUMBER,
}
pub type FSVOL_CONTEXT = _FSVOL_CONTEXT;
pub type PFSVOL_CONTEXT = *mut FSVOL_CONTEXT;
#[repr(C)]
pub struct _COPYCONTEXT {
    pub TotalOperations: ULONG,
    pub CompletedOperations: ULONG,
    pub ProgressBar: PPROGRESSBAR,
    pub MemoryBars: [PPROGRESSBAR; 4],
}
pub type COPYCONTEXT = _COPYCONTEXT;
pub type PCOPYCONTEXT = *mut COPYCONTEXT;

unsafe extern "C" fn DoWatchDestFileName(FileName: LPCWSTR) {
    if *FileName.offset(0 as isize) == 'm' as u16
        || *FileName.offset(0 as isize) == 'M' as u16
    {
        if _wcsicmp(FileName, RUSTCONV_WSTR_840220BD5E07.as_ptr()) == 0 {
            DPRINT!(b"mingliu.ttc found\n\0".as_ptr() as *const i8);
            s_SubstSettings.bFoundFontMINGLIU = (TRUE) as i32;
        } else if _wcsicmp(FileName, RUSTCONV_WSTR_AC374BD06A63.as_ptr()) == 0 {
            DPRINT!(b"msgothic.ttc found\n\0".as_ptr() as *const i8);
            s_SubstSettings.bFoundFontMSGOTHIC = (TRUE) as i32;
        } else if _wcsicmp(FileName, RUSTCONV_WSTR_0946A42E27B2.as_ptr()) == 0 {
            DPRINT!(b"msmincho.ttc found\n\0".as_ptr() as *const i8);
            s_SubstSettings.bFoundFontMSMINCHO = (TRUE) as i32;
        } else if _wcsicmp(FileName, RUSTCONV_WSTR_99A57B2B061D.as_ptr()) == 0 {
            DPRINT!(b"mssong.ttf found\n\0".as_ptr() as *const i8);
            s_SubstSettings.bFoundFontMSSONG = (TRUE) as i32;
        }
    } else {
        if _wcsicmp(FileName, RUSTCONV_WSTR_9CA37DD1B2AA.as_ptr()) == 0 {
            DPRINT!(b"simsun.ttc found\n\0".as_ptr() as *const i8);
            s_SubstSettings.bFoundFontSIMSUN = (TRUE) as i32;
        } else if _wcsicmp(FileName, RUSTCONV_WSTR_603653C2EA23.as_ptr()) == 0 {
            DPRINT!(b"gulim.ttc found\n\0".as_ptr() as *const i8);
            s_SubstSettings.bFoundFontGULIM = (TRUE) as i32;
        } else if _wcsicmp(FileName, RUSTCONV_WSTR_04F17891E646.as_ptr()) == 0 {
            DPRINT!(b"batang.ttc found\n\0".as_ptr() as *const i8);
            s_SubstSettings.bFoundFontBATANG = (TRUE) as i32;
        }
    }
}

extern "C" {
    /// C-variadic, file-`static` in usetup.c: DEFINED below by `va_entry!` under a
    /// crate-private symbol name (a `static` has no external name to claim), declared here
    /// so this module's callers can pass it varargs.
    #[link_name = "usetup_static_PrintString"]
    fn PrintString(fmt: PCSTR, ...);
}

/// `static VOID PrintString(IN PCSTR fmt, ...)`, defined through varargs-rust's naked thunk:
/// `a.get(0)` is `fmt`, `a.va(1)` is the `va_list` of `va_start(ap, fmt)`.
mod PrintString_export {
    use super::*;
    varargs_rust::va_entry!(usetup_static_PrintString => |a| -> () {
        let fmt = a.get(0) as PCSTR;
        let mut buffer: [CHAR; 512] = [0; 512];
        let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();
        let mut AnsiString: ANSI_STRING = core::mem::zeroed();

        vsprintf(buffer.as_mut_ptr(), fmt, a.va(1) as va_list);

        RtlInitAnsiString(&mut AnsiString, buffer.as_ptr());
        RtlAnsiStringToUnicodeString(&mut UnicodeString, &mut AnsiString, TRUE as BOOLEAN);
        NtDisplayString(&mut UnicodeString);
        RtlFreeUnicodeString(&mut UnicodeString);
    });
}

unsafe extern "C" fn DrawBox(
    xLeft: i16,
    yTop: i16,
    Width: i16,
    Height: i16,
) {
    let mut coPos: COORD = core::mem::zeroed();
    let mut Written: u32 = 0;

    /* Draw upper left corner */
    coPos.X = xLeft;
    coPos.Y = yTop;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharUpperLeftCorner,
        1,
        coPos,
        &mut Written,
    );

    /* Draw upper edge */
    coPos.X = xLeft + 1;
    coPos.Y = yTop;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharHorizontalLine,
        (Width - 2) as DWORD,
        coPos,
        &mut Written,
    );

    /* Draw upper right corner */
    coPos.X = xLeft + Width - 1;
    coPos.Y = yTop;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharUpperRightCorner,
        1,
        coPos,
        &mut Written,
    );

    /* Draw right edge, inner space and left edge */
    coPos.Y = yTop + 1;
    while coPos.Y < yTop + Height - 1 {
        coPos.X = xLeft;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharVerticalLine,
            1,
            coPos,
            &mut Written,
        );

        coPos.X = xLeft + 1;
        FillConsoleOutputCharacterA(
            StdOutput,
            (' ') as CHAR,
            (Width - 2) as DWORD,
            coPos,
            &mut Written,
        );

        coPos.X = xLeft + Width - 1;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharVerticalLine,
            1,
            coPos,
            &mut Written,
        );

        coPos.Y += 1;
    }

    /* Draw lower left corner */
    coPos.X = xLeft;
    coPos.Y = yTop + Height - 1;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharLowerLeftCorner,
        1,
        coPos,
        &mut Written,
    );

    /* Draw lower edge */
    coPos.X = xLeft + 1;
    coPos.Y = yTop + Height - 1;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharHorizontalLine,
        (Width - 2) as DWORD,
        coPos,
        &mut Written,
    );

    /* Draw lower right corner */
    coPos.X = xLeft + Width - 1;
    coPos.Y = yTop + Height - 1;
    FillConsoleOutputCharacterA(
        StdOutput,
        CharLowerRightCorner,
        1,
        coPos,
        &mut Written,
    );
}
#[no_mangle]
pub unsafe extern "C" fn PopupError(Text: PCCH, Status: PCCH, Ir: PINPUT_RECORD, WaitEvent: ULONG) {
    let mut yTop: SHORT;
    let mut xLeft: SHORT;
    let mut coPos: COORD = core::mem::zeroed();
    let mut Written: DWORD = core::mem::zeroed();
    let mut Length: ULONG;
    let mut MaxLength: ULONG;
    let mut Lines: ULONG;
    let mut p: PCHAR;
    let mut pnext: PCCH;
    let mut LastLine: BOOLEAN;
    let mut Width: SHORT;
    let mut Height: SHORT;

    // Not in the C: every popup also goes to the debug port. Its text otherwise exists only
    // on the screen, and an unattended setup that hits one (POPUP_WAIT_ENTER) idles forever
    // with nothing on serial -- the install smoke treats this line as fatal.
    DPRINT1!(b"[USETUP] error popup: %s\n\0".as_ptr() as *const i8, Text);

    MaxLength = 0;
    Lines = 0;
    pnext = Text;

    loop {
        p = strchr(pnext, '\n' as i32);

        if p.is_null() {
            Length = (strlen(pnext)) as _;
            LastLine = (TRUE) as u8;
        } else {
            Length = (p as usize - pnext as usize) as ULONG;
            LastLine = (FALSE) as u8;
        }

        Lines += 1;

        if Length > MaxLength {
            MaxLength = Length;
        }

        if LastLine != 0 {
            break;
        }

        pnext = p.add(1);
    }

    if !Status.is_null() {
        Length = (strlen(Status)) as _;

        if Length > MaxLength {
            MaxLength = Length;
        }
    }

    Width = (MaxLength + 4) as SHORT;
    Height = (Lines + 2) as SHORT;

    if !Status.is_null() {
        Height += 2 as SHORT;
    }

    yTop = ((yScreen - Height) / 2) as SHORT;
    xLeft = ((xScreen - Width) / 2) as SHORT;

    coPos.X = xLeft;
    coPos.Y = yTop;
    while coPos.Y < yTop + Height {
        FillConsoleOutputAttribute(
            StdOutput,
            FOREGROUND_RED | (BACKGROUND_WHITE) as u16,
            Width as DWORD,
            coPos,
            &mut Written,
        );
        coPos.Y += 1;
    }

    DrawBox(xLeft, yTop, Width, Height);

    coPos.Y = yTop + 1 as SHORT;
    pnext = Text;
    loop {
        p = strchr(pnext, '\n' as i32);

        if p.is_null() {
            Length = (strlen(pnext)) as _;
            LastLine = (TRUE) as u8;
        } else {
            Length = (p as usize - pnext as usize) as ULONG;
            LastLine = (FALSE) as u8;
        }

        if Length != 0 {
            coPos.X = xLeft + 2 as SHORT;
            WriteConsoleOutputCharacterA(
                StdOutput,
                pnext,
                Length,
                coPos,
                &mut Written,
            );
        }

        if LastLine != 0 {
            break;
        }

        coPos.Y += 1;
        pnext = p.add(1);
    }

    if !Status.is_null() {
        coPos.Y = yTop + Height - 3 as SHORT;
        coPos.X = xLeft;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharVertLineAndRightHorizLine,
            1,
            coPos,
            &mut Written,
        );

        coPos.X = xLeft + 1 as SHORT;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharHorizontalLine,
            (Width - 2 as SHORT) as DWORD,
            coPos,
            &mut Written,
        );

        coPos.X = xLeft + Width - 1 as SHORT;
        FillConsoleOutputCharacterA(
            StdOutput,
            CharLeftHorizLineAndVertLine,
            1,
            coPos,
            &mut Written,
        );

        coPos.Y += 1;
        coPos.X = xLeft + 2 as SHORT;
        WriteConsoleOutputCharacterA(
            StdOutput,
            Status,
            min(
                (strlen(Status)) as _,
                ((Width as usize - 4) as SIZE_T) as u32,
            ),
            coPos,
            &mut Written,
        );
    }

    if WaitEvent == POPUP_WAIT_NONE {
        return;
    }

    loop {
        CONSOLE_ConInKey(Ir);

        if WaitEvent == POPUP_WAIT_ANY_KEY
            || ((*Ir).Event.KeyEvent.uChar.AsciiChar) as i32 == (0x0D as u8) as i32
        {
            return;
        }
    }
}

unsafe extern "C" fn ConfirmQuit(Ir: PINPUT_RECORD) -> i32 {
    let mut Result: i32 = (FALSE) as i32;
    MUIDisplayError(ERROR_NOT_INSTALLED, core::ptr::null_mut(), POPUP_WAIT_NONE);

    loop {
        CONSOLE_ConInKey(Ir);

        if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
            (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3)
        {
            Result = (TRUE) as i32;
            break;
        }
        else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x0D {
            Result = (FALSE) as i32;
            break;
        }
    }

    Result
}

unsafe extern "C" fn UpdateKBLayout() {
    let mut ListEntry: *mut GENERIC_LIST_ENTRY;
    let mut newLayout: KLID;

    newLayout = MUIDefaultKeyboardLayout(SelectedLanguageId);

    if USetupData.LayoutList.is_null() {
        USetupData.LayoutList = CreateKeyboardLayoutList(USetupData.SetupInf, SelectedLanguageId, DefaultKBLayout.as_mut_ptr());
        if USetupData.LayoutList.is_null() {
            /* FIXME: Handle error! */
            return;
        }
    }

    /* Search for default layout (if provided) */
    if newLayout != 0 {
        ListEntry = GetFirstListEntry(USetupData.LayoutList);
        while !ListEntry.is_null() {
            let pszLayoutId: PCWSTR = (*(*(GetListEntryData(ListEntry) as *mut PGENENTRY))).Id;
            let LayoutId: KLID = if !pszLayoutId.is_null() {
                wcstoul(pszLayoutId, core::ptr::null_mut(), 16)
            } else {
                0
            };
            if newLayout == LayoutId {
                SetCurrentListEntry(USetupData.LayoutList, ListEntry);
                break;
            }
            ListEntry = GetNextListEntry(ListEntry);
        }
    }
}

unsafe extern "system" fn GetSettingDescription(
    Entry: *mut GENERIC_LIST_ENTRY,
    Buffer: PSTR,
    cchBufferSize: SIZE_T,
) -> i32 {
    return RtlStringCchPrintfA!(
        Buffer,
        cchBufferSize as usize,
        b"%S\0".as_ptr() as *const i8,
        (*(*(GetListEntryData(Entry) as *mut PGENENTRY))).Value,
    );
}

unsafe extern "system" fn GetNTOSInstallationName(
    Entry: *mut GENERIC_LIST_ENTRY,
    Buffer: PSTR,
    cchBufferSize: SIZE_T,
) -> NTSTATUS {
    let NtOsInstall = GetListEntryData(Entry) as *mut NTOS_INSTALLATION;
    let VolInfo = if !(*NtOsInstall).Volume.is_null() {
        &mut (*(*NtOsInstall).Volume).Info as *mut VOLINFO
    } else {
        core::ptr::null_mut()
    };

    if !VolInfo.is_null() && (*VolInfo).DriveLetter != 0 {
        return RtlStringCchPrintfA!(
            Buffer,
            cchBufferSize as usize,
            b"%C:%S  \"%S\"\0".as_ptr() as *const i8,
            (*VolInfo).DriveLetter as core::ffi::c_uint,
            (*NtOsInstall).PathComponent,
            (*NtOsInstall).InstallationName,
        );
    } else {
        return RtlStringCchPrintfA!(
            Buffer,
            cchBufferSize as usize,
            b"%wZ  \"%S\"\0".as_ptr() as *const i8,
            &mut (*NtOsInstall).SystemNtPath as *mut _UNICODE_STRING,
            (*NtOsInstall).InstallationName,
        );
    }
}

extern "C" {
    /// C-variadic, file-`static` in usetup.c (a PSETUP_ERROR_ROUTINE): DEFINED below by
    /// `va_entry!` under a crate-private symbol name, declared here so it can be passed as
    /// the `...` function pointer InitializeSetup takes.
    #[link_name = "usetup_static_USetupErrorRoutine"]
    fn USetupErrorRoutine(pSetupData: PUSETUP_DATA, ...);
}

/// `static VOID __cdecl USetupErrorRoutine(IN PUSETUP_DATA pSetupData, ...)`, defined through
/// varargs-rust's naked thunk: `a.get(0)` is `pSetupData`, `a.va(1)` is the `va_list` of
/// `va_start(arg_ptr, pSetupData)`.
mod USetupErrorRoutine_export {
    use super::*;
    varargs_rust::va_entry!(usetup_static_USetupErrorRoutine => |a| -> () {
        let pSetupData = a.get(0) as PUSETUP_DATA;
        let mut Ir: INPUT_RECORD = core::mem::zeroed();

        if (*pSetupData).LastErrorNumber >= ERROR_SUCCESS
            && (*pSetupData).LastErrorNumber < ERROR_LAST_ERROR_CODE
        {
            // Note: the "POPUP_WAIT_ENTER" actually depends on the LastErrorNumber...
            MUIDisplayErrorV((*pSetupData).LastErrorNumber, &mut Ir, POPUP_WAIT_ENTER, a.va(1) as va_list);
        }
    });
}

unsafe extern "C" fn SetupStartPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut Error: ULONG;
    let mut ListEntry: PGENERIC_LIST_ENTRY;
    let mut LocaleId: PCWSTR;

    MUIDisplayPage(SETUP_INIT_PAGE);

    Error = InitializeSetup(
        &mut USetupData,
        Some(USetupErrorRoutine),
        &mut SpFileExports,
        &mut SpInfExports,
    );
    if Error != ERROR_SUCCESS {
        MUIDisplayError(Error, Ir, POPUP_WAIT_ENTER);
        return QUIT_PAGE;
    }

    if ((EnableUserModePnpManager()) == 0) {
        DPRINT1!(
            b"The user-mode PnP manager could not initialize, expect unavailable devices!\n\0"
                .as_ptr() as *const i8,
        );
    }

    if WaitNoPendingInstallEvents(core::ptr::null_mut()) != STATUS_WAIT_0 {
        DPRINT1!(b"WaitNoPendingInstallEvents() failed to wait!\n\0".as_ptr() as *const i8);
    }

    IsUnattendedSetup = CheckUnattendedSetup(&mut USetupData);
    if IsUnattendedSetup != 0 {
        USetupData.ComputerList = CreateComputerTypeList(USetupData.SetupInf);
        USetupData.DisplayList = CreateDisplayDriverList(USetupData.SetupInf);
        USetupData.KeyboardList = CreateKeyboardDriverList(USetupData.SetupInf);

        USetupData.LanguageList = CreateLanguageList(USetupData.SetupInf, DefaultLanguage.as_mut_ptr());

        SelectedLanguageId = DefaultLanguage.as_ptr();
        wcscpy(DefaultLanguage.as_mut_ptr(), (USetupData.LocaleID).as_ptr());
        USetupData.LanguageId = (wcstol(
            SelectedLanguageId,
            core::ptr::null_mut(),
            16,
        ) & 0xFFFF) as LANGID;

        USetupData.LayoutList = CreateKeyboardLayoutList(
            USetupData.SetupInf,
            SelectedLanguageId,
            DefaultKBLayout.as_mut_ptr(),
        );

        ListEntry = GetFirstListEntry(USetupData.LanguageList);
        while !ListEntry.is_null() {
            LocaleId = (*(GetListEntryData(ListEntry) as PGENENTRY)).Id;
            if ((_wcsicmp((USetupData.LocaleID).as_ptr(), LocaleId)) == 0) {
                DPRINT!(b"found %S in LanguageList\n\0".as_ptr() as *const i8, LocaleId);
                SetCurrentListEntry(USetupData.LanguageList, ListEntry);
                break;
            }
            ListEntry = GetNextListEntry(ListEntry);
        }

        ListEntry = GetFirstListEntry(USetupData.LayoutList);
        while !ListEntry.is_null() {
            // C: ((PGENENTRY)GetListEntryData(ListEntry))->Id. A cast to *mut PGENENTRY
            // plus .read() went one level deeper and used the Id TEXT as a pointer.
            LocaleId = (*(GetListEntryData(ListEntry) as PGENENTRY)).Id;
            if ((_wcsicmp((USetupData.LocaleID).as_ptr(), LocaleId)) == 0) {
                DPRINT!(b"found %S in LayoutList\n\0".as_ptr() as *const i8, LocaleId);
                SetCurrentListEntry(USetupData.LayoutList, ListEntry);
                break;
            }
            ListEntry = GetNextListEntry(ListEntry);
        }

        SetConsoleCodePage();

        return INSTALL_INTRO_PAGE;
    }

    return LANGUAGE_PAGE;
}

unsafe extern "C" fn LanguagePage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut ListUi: GENERIC_LIST_UI = core::mem::zeroed();
    let mut NewLanguageId: PCWSTR = core::ptr::null_mut();
    let mut RefreshPage: BOOL = (FALSE) as i32;

    if USetupData.LanguageList.is_null() {
        USetupData.LanguageList = CreateLanguageList(USetupData.SetupInf, DefaultLanguage.as_ptr() as *mut u16);
        if USetupData.LanguageList.is_null() {
            PopupError(
                b"Setup failed to initialize available translations\0".as_ptr() as *const i8,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                POPUP_WAIT_NONE,
            );
            return WELCOME_PAGE;
        }
    }

    SelectedLanguageId = DefaultLanguage.as_ptr();
    USetupData.LanguageId = 0;

    SetConsoleCodePage();
    UpdateKBLayout();

    if GetNumberOfListEntries(USetupData.LanguageList) <= 1 {
        USetupData.LanguageId = (wcstol(
            SelectedLanguageId,
            core::ptr::null_mut(),
            16,
        ) & 0xFFFF) as LANGID;
        return WELCOME_PAGE;
    }

    InitGenericListUi(
        &mut ListUi,
        USetupData.LanguageList,
        Some(GetSettingDescription),
    );
    DrawGenericList(
        &mut ListUi,
        2,
        18,
        xScreen - 3,
        yScreen - 3,
    );

    ScrollToPositionGenericList(&mut ListUi, GetDefaultLanguageIndex());

    MUIDisplayPage(LANGUAGE_PAGE);

    loop {
        CONSOLE_ConInKey(Ir);

        if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_DOWN
        {
            ScrollDownGenericList(&mut ListUi);
            RefreshPage = (TRUE) as i32;
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_UP
        {
            ScrollUpGenericList(&mut ListUi);
            RefreshPage = (TRUE) as i32;
        }
        if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_NEXT
        {
            ScrollPageDownGenericList(&mut ListUi);
            RefreshPage = (TRUE) as i32;
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_PRIOR
        {
            ScrollPageUpGenericList(&mut ListUi);
            RefreshPage = (TRUE) as i32;
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3
        {
            if ConfirmQuit(Ir) != 0 {
                return QUIT_PAGE;
            }
            RedrawGenericList(&mut ListUi);
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x0D {
            SelectedLanguageId = (*(*(GetListEntryData(
                GetCurrentListEntry(USetupData.LanguageList),
            ) as *mut PGENENTRY)))
                .Id;

            USetupData.LanguageId = (wcstol(
                SelectedLanguageId,
                core::ptr::null_mut(),
                16,
            ) & 0xFFFF) as LANGID;

            if wcscmp(SelectedLanguageId, DefaultLanguage.as_ptr()) != 0 {
                UpdateKBLayout();
            }

            SetConsoleCodePage();

            return WELCOME_PAGE;
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar > 0x60
            && (*Ir).Event.KeyEvent.uChar.AsciiChar < 0x7b
        {
            GenericListKeyPress(&mut ListUi, (*Ir).Event.KeyEvent.uChar.AsciiChar);
            RefreshPage = (TRUE) as i32;
        }

        if RefreshPage != 0 {
            NewLanguageId = (*(*(GetListEntryData(
                GetCurrentListEntry(USetupData.LanguageList),
            ) as *mut PGENENTRY)))
                .Id;

            if wcscmp(SelectedLanguageId, NewLanguageId) != 0 {
                MUIClearPage(LANGUAGE_PAGE);

                SelectedLanguageId = NewLanguageId;

                SetConsoleCodePage();

                DrawGenericList(
                    &mut ListUi,
                    2,
                    18,
                    xScreen - 3,
                    yScreen - 3,
                );

                MUIDisplayPage(LANGUAGE_PAGE);
            }

            RefreshPage = (FALSE) as i32;
        }
    }

    return WELCOME_PAGE;
}

unsafe extern "C" fn WelcomePage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    MUIDisplayPage(WELCOME_PAGE);

    loop {
        CONSOLE_ConInKey(Ir);

        if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
            (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3)
        {
            if ConfirmQuit(Ir) != 0 {
                return QUIT_PAGE;
            }
            break;
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x0D {
            return INSTALL_INTRO_PAGE;
        } else if toupper(((*Ir).Event.KeyEvent.uChar.AsciiChar) as _) == 'R' as i32 {
            return RECOVERY_PAGE;
        } else if toupper(((*Ir).Event.KeyEvent.uChar.AsciiChar) as _) == 'L' as i32 {
            return LICENSE_PAGE;
        }
    }

    return WELCOME_PAGE;
}

unsafe extern "C" fn LicensePage(Ir: *mut INPUT_RECORD) -> PAGE_NUMBER {
    MUIDisplayPage(LICENSE_PAGE);

    loop {
        CONSOLE_ConInKey(Ir);

        if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x0D {
            return WELCOME_PAGE;
        }
    }

    return LICENSE_PAGE;
}

unsafe extern "C" fn RepairIntroPage(Ir: *mut INPUT_RECORD) -> PAGE_NUMBER {
    MUIDisplayPage(REPAIR_INTRO_PAGE);

    loop {
        CONSOLE_ConInKey(Ir);

        if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x0D {
            return REBOOT_PAGE;
        } else if toupper(((*Ir).Event.KeyEvent.uChar.AsciiChar) as _) == ('U' as u8) as _ {
            RepairUpdateFlag = (TRUE) as u8;
            return INSTALL_INTRO_PAGE;
        } else if toupper(((*Ir).Event.KeyEvent.uChar.AsciiChar) as _) == ('R' as u8) as _ {
            return RECOVERY_PAGE;
        } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_ESCAPE {
            return WELCOME_PAGE;
        }
    }

    return REPAIR_INTRO_PAGE;
}

unsafe extern "C" fn UpgradeRepairPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut ListUi: GENERIC_LIST_UI = core::mem::zeroed();

    if PartitionList.is_null() {
        PartitionList = CreatePartitionList();
        if PartitionList.is_null() {
            MUIDisplayError(ERROR_DRIVE_INFORMATION, Ir, POPUP_WAIT_ENTER);
            return QUIT_PAGE;
        } else if IsListEmpty(&(*PartitionList).DiskListHead) != false {
            MUIDisplayError(ERROR_NO_HDD, Ir, POPUP_WAIT_ENTER);
            return QUIT_PAGE;
        }
    }

    NtOsInstallsList = CreateNTOSInstallationsList(PartitionList);
    if NtOsInstallsList.is_null() {
        DPRINT1!(b"Failed to get a list of NTOS installations; continue installation...\n\0".as_ptr() as *const i8);
    }

    if NtOsInstallsList.is_null() || GetNumberOfListEntries(NtOsInstallsList) == 0 {
        RepairUpdateFlag = (FALSE) as u8;
        return DEVICE_SETTINGS_PAGE;
    }

    MUIDisplayPage(UPGRADE_REPAIR_PAGE);

    InitGenericListUi(&mut ListUi, NtOsInstallsList, Some(GetNTOSInstallationName));
    DrawGenericList(
        &mut ListUi,
        2,
        23,
        xScreen - 3,
        yScreen - 3,
    );

    loop {
        CONSOLE_ConInKey(Ir);

        if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00 {
            match ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 {
                VK_DOWN => {
                    ScrollDownGenericList(&mut ListUi);
                }
                VK_UP => {
                    ScrollUpGenericList(&mut ListUi);
                }
                VK_NEXT => {
                    ScrollPageDownGenericList(&mut ListUi);
                }
                VK_PRIOR => {
                    ScrollPageUpGenericList(&mut ListUi);
                }
                VK_F3 => {
                    if ConfirmQuit(Ir) != 0 {
                        return QUIT_PAGE;
                    }
                    RedrawGenericList(&mut ListUi);
                }
                VK_ESCAPE => {
                    RestoreGenericListUiState(&mut ListUi);
                    return DEVICE_SETTINGS_PAGE;
                }
                _ => {}
            }
        } else {
            if toupper(((*Ir).Event.KeyEvent.uChar.AsciiChar) as _) == 'U' as i32 {
                CurrentInstallation = GetListEntryData(GetCurrentListEntry(NtOsInstallsList)) as PNTOS_INSTALLATION;

                DPRINT1!(
                    b"Selected installation for repair: \"%S\" ; DiskNumber = %d , PartitionNumber = %d\n\0".as_ptr() as *const i8,
                    (*CurrentInstallation).InstallationName,
                    (*CurrentInstallation).DiskNumber,
                    (*CurrentInstallation).PartitionNumber,
                );

                RepairUpdateFlag = (TRUE) as u8;
                return INSTALL_INTRO_PAGE;
            } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar > 0x60)
                && ((*Ir).Event.KeyEvent.uChar.AsciiChar < 0x7b)
            {
                GenericListKeyPress(&mut ListUi, (*Ir).Event.KeyEvent.uChar.AsciiChar);
            }
        }
    }

    return UPGRADE_REPAIR_PAGE;
}

unsafe extern "C" fn InstallIntroPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    if ((RepairUpdateFlag) != 0) {
        return DEVICE_SETTINGS_PAGE;
    }

    if ((IsUnattendedSetup) != 0) {
        return SELECT_PARTITION_PAGE;
    }

    MUIDisplayPage(INSTALL_INTRO_PAGE);

    loop {
        CONSOLE_ConInKey(Ir);

        if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3
        {
            if ((ConfirmQuit(Ir)) != 0) {
                return QUIT_PAGE;
            }
            break;
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x0D {
            return UPGRADE_REPAIR_PAGE;
        }
    }

    return INSTALL_INTRO_PAGE;
}

unsafe extern "C" fn DeviceSettingsPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    static mut Line: ULONG = 16;

    if USetupData.ComputerList.is_null() {
        USetupData.ComputerList = CreateComputerTypeList(USetupData.SetupInf);
        if USetupData.ComputerList.is_null() {
            MUIDisplayError(ERROR_LOAD_COMPUTER, Ir, POPUP_WAIT_ENTER);
            return QUIT_PAGE;
        }
    }

    if USetupData.DisplayList.is_null() {
        USetupData.DisplayList = CreateDisplayDriverList(USetupData.SetupInf);
        if USetupData.DisplayList.is_null() {
            MUIDisplayError(ERROR_LOAD_DISPLAY, Ir, POPUP_WAIT_ENTER);
            return QUIT_PAGE;
        }
    }

    if USetupData.KeyboardList.is_null() {
        USetupData.KeyboardList = CreateKeyboardDriverList(USetupData.SetupInf);
        if USetupData.KeyboardList.is_null() {
            MUIDisplayError(ERROR_LOAD_KEYBOARD, Ir, POPUP_WAIT_ENTER);
            return QUIT_PAGE;
        }
    }

    if USetupData.LayoutList.is_null() {
        USetupData.LayoutList = CreateKeyboardLayoutList(USetupData.SetupInf, SelectedLanguageId, DefaultKBLayout.as_mut_ptr());
        if USetupData.LayoutList.is_null() {
            MUIDisplayError(ERROR_LOAD_KBLAYOUT, Ir, POPUP_WAIT_ENTER);
            return QUIT_PAGE;
        }
    }

    if RepairUpdateFlag != 0 {
        return SELECT_PARTITION_PAGE;
    }

    MUIDisplayPage(DEVICE_SETTINGS_PAGE);

    DrawGenericListCurrentItem(USetupData.ComputerList, Some(GetSettingDescription), 25, 11);
    DrawGenericListCurrentItem(USetupData.DisplayList, Some(GetSettingDescription), 25, 12);
    DrawGenericListCurrentItem(USetupData.KeyboardList, Some(GetSettingDescription), 25, 13);
    DrawGenericListCurrentItem(USetupData.LayoutList, Some(GetSettingDescription), 25, 14);

    CONSOLE_InvertTextXY(24, (Line) as SHORT, 48, 1);

    loop {
        CONSOLE_ConInKey(Ir);

        if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_DOWN
        {
            CONSOLE_NormalTextXY(24, (Line) as SHORT, 48, 1);

            if Line == 14 {
                Line = 16;
            } else if Line == 16 {
                Line = 11;
            } else {
                Line += 1;
            }

            CONSOLE_InvertTextXY(24, (Line) as SHORT, 48, 1);
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_UP
        {
            CONSOLE_NormalTextXY(24, (Line) as SHORT, 48, 1);

            if Line == 11 {
                Line = 16;
            } else if Line == 16 {
                Line = 14;
            } else {
                Line -= 1;
            }

            CONSOLE_InvertTextXY(24, (Line) as SHORT, 48, 1);
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3
        {
            if ConfirmQuit(Ir) != 0 {
                return QUIT_PAGE;
            }
            break;
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x0D {
            if Line == 11 {
                return COMPUTER_SETTINGS_PAGE;
            } else if Line == 12 {
                return DISPLAY_SETTINGS_PAGE;
            } else if Line == 13 {
                return KEYBOARD_SETTINGS_PAGE;
            } else if Line == 14 {
                return LAYOUT_SETTINGS_PAGE;
            } else if Line == 16 {
                return SELECT_PARTITION_PAGE;
            }
        }
    }

    return DEVICE_SETTINGS_PAGE;
}

unsafe extern "C" fn HandleGenericList(
    ListUi: *mut GENERIC_LIST_UI,
    nextPage: PAGE_NUMBER,
    Ir: *mut INPUT_RECORD,
) -> PAGE_NUMBER {
    loop {
        CONSOLE_ConInKey(Ir);

        if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00)
            && (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_DOWN)
        {
            ScrollDownGenericList(ListUi);
        } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00)
            && (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_UP)
        {
            ScrollUpGenericList(ListUi);
        } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00)
            && (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_NEXT)
        {
            ScrollPageDownGenericList(ListUi);
        } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00)
            && (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_PRIOR)
        {
            ScrollPageUpGenericList(ListUi);
        } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00)
            && (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3)
        {
            if ConfirmQuit(Ir) != 0 {
                return QUIT_PAGE;
            }
            RedrawGenericList(ListUi);
        } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_ESCAPE {
            RestoreGenericListUiState(ListUi);
            return nextPage;
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x0D {
            return nextPage;
        } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar > 0x60
            && (*Ir).Event.KeyEvent.uChar.AsciiChar < 0x7b)
        {
            GenericListKeyPress(ListUi, (*Ir).Event.KeyEvent.uChar.AsciiChar);
        }
    }
}

unsafe extern "C" fn ComputerSettingsPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut ListUi: GENERIC_LIST_UI = core::mem::zeroed();
    MUIDisplayPage(COMPUTER_SETTINGS_PAGE);

    InitGenericListUi(
        &mut ListUi,
        USetupData.ComputerList,
        Some(GetSettingDescription),
    );
    DrawGenericList(
        &mut ListUi,
        2,
        18,
        xScreen - 3,
        yScreen - 3,
    );

    return HandleGenericList(&mut ListUi, DEVICE_SETTINGS_PAGE, Ir);
}

unsafe extern "C" fn DisplaySettingsPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut ListUi: GENERIC_LIST_UI = core::mem::zeroed();
    MUIDisplayPage(DISPLAY_SETTINGS_PAGE);

    InitGenericListUi(&mut ListUi, USetupData.DisplayList, Some(GetSettingDescription));
    DrawGenericList(
        &mut ListUi,
        2,
        18,
        xScreen - 3,
        yScreen - 3,
    );

    return HandleGenericList(&mut ListUi, DEVICE_SETTINGS_PAGE, Ir);
}

unsafe extern "C" fn KeyboardSettingsPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut ListUi: GENERIC_LIST_UI = core::mem::zeroed();
    MUIDisplayPage(KEYBOARD_SETTINGS_PAGE);

    InitGenericListUi(
        &mut ListUi,
        USetupData.KeyboardList,
        Some(GetSettingDescription),
    );
    DrawGenericList(
        &mut ListUi,
        2,
        18,
        xScreen - 3,
        yScreen - 3,
    );

    return HandleGenericList(&mut ListUi, DEVICE_SETTINGS_PAGE, Ir);
}

unsafe extern "C" fn LayoutSettingsPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut ListUi: GENERIC_LIST_UI = core::mem::zeroed();
    MUIDisplayPage(LAYOUT_SETTINGS_PAGE);

    InitGenericListUi(
        &mut ListUi,
        USetupData.LayoutList,
        Some(GetSettingDescription),
    );
    DrawGenericList(
        &mut ListUi,
        2,
        18,
        xScreen - 3,
        yScreen - 3,
    );

    return HandleGenericList(&mut ListUi, DEVICE_SETTINGS_PAGE, Ir);
}

unsafe extern "C" fn IsMediumLargeEnough(SizeInBytes: u64) -> u8 {
    let SizeInMB: u64 = RoundingDivide(SizeInBytes, MB);

    if SizeInMB < USetupData.RequiredPartitionDiskSpace as u64 {
        DPRINT1!(
            b"Partition/Volume is too small (size: %I64u MB), required space is %lu MB\n\0"
                .as_ptr() as *const i8,
            SizeInMB,
            USetupData.RequiredPartitionDiskSpace,
        );
        return (FALSE) as u8;
    }
    return (TRUE) as u8;
}

unsafe extern "C" fn SelectPartitionPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut ListUi: PARTLIST_UI = core::mem::zeroed();
    let mut Error: ULONG;
    let mut MaxTargetSize: ULONGLONG;

    if PartitionList.is_null() {
        PartitionList = CreatePartitionList();
        if PartitionList.is_null() {
            MUIDisplayError(ERROR_DRIVE_INFORMATION, Ir, POPUP_WAIT_ENTER);
            return QUIT_PAGE;
        } else if IsListEmpty(&mut (*PartitionList).DiskListHead) != false {
            MUIDisplayError(ERROR_NO_HDD, Ir, POPUP_WAIT_ENTER);
            return QUIT_PAGE;
        }
    }

    if RepairUpdateFlag != 0 {
        ASSERT!(CurrentInstallation);

        /* Determine the selected installation disk & partition.
         * It must exist and be valid, since this is the partition
         * where the existing installation already resides. */
        InstallPartition = SelectPartition(
            PartitionList,
            (*CurrentInstallation).DiskNumber,
            (*CurrentInstallation).PartitionNumber,
        );
        if InstallPartition.is_null() {
            DPRINT1!(
                b"RepairUpdateFlag == TRUE, SelectPartition() returned FALSE, assert!\n\0".as_ptr()
                    as *const i8,
            );
            ASSERT!(FALSE);
        }
        ASSERT!((*InstallPartition).IsPartitioned);
        ASSERT!((*InstallPartition).Volume);

        return START_PARTITION_OPERATIONS_PAGE;
    }

    MUIDisplayPage(SELECT_PARTITION_PAGE);

    InitPartitionListUi(
        &mut ListUi,
        PartitionList,
        CurrentPartition,
        2,
        21,
        xScreen - 3,
        yScreen - 3,
    );
    DrawPartitionList(&mut ListUi);

    // C: every `goto CreateInstallPartition;` jumps forward past the selection loop; that
    // label is the end of this block, so each goto is `break 'CreateInstallPartition`.
    'CreateInstallPartition: {
        if IsUnattendedSetup != 0 {
            // C: `do { ... } while (0);` -- `break` leaves it.
            'once: {
                /* If DestinationDiskNumber or DestinationPartitionNumber are invalid
                 * (see below), don't select the partition and show the list instead */
                if USetupData.DestinationDiskNumber == -1
                    || USetupData.DestinationPartitionNumber == -1
                {
                    break 'once;
                }

                /* Determine the selected installation disk & partition */
                CurrentPartition = SelectPartition(
                    PartitionList,
                    (USetupData.DestinationDiskNumber) as ULONG,
                    (USetupData.DestinationPartitionNumber) as ULONG,
                );

                /* Now reset DestinationDiskNumber and DestinationPartitionNumber
                 * to *invalid* values, so that if the corresponding partition is
                 * determined to be invalid by the code below or in CreateInstallPartition,
                 * we don't reselect it when SelectPartitionPage() is called again */
                USetupData.DestinationDiskNumber = -1;
                USetupData.DestinationPartitionNumber = -1;

                if !CurrentPartition.is_null()
                    && IsContainerPartition!((*CurrentPartition).PartitionType) == false
                {
                    break 'CreateInstallPartition;
                }

                if USetupData.AutoPartition != 0 {
                    CurrentPartition = ListUi.CurrentPartition;
                    // TODO: Do more checks, and loop until we find a valid partition.
                    break 'CreateInstallPartition;
                }
            }
        }

        loop {
            let mut uID: ULONG;

            CurrentPartition = ListUi.CurrentPartition;

            /* Update status text */
            if CurrentPartition.is_null() {
                uID = STRING_INSTALLCREATEPARTITION;
            } else if (*CurrentPartition).IsPartitioned != 0 {
                uID = STRING_INSTALLDELETEPARTITION;
                if (*CurrentPartition).LogicalPartition == 0
                    && IsContainerPartition!((*CurrentPartition).PartitionType) != false
                {
                    uID = STRING_DELETEPARTITION;
                }
            } else {
                uID = STRING_INSTALLCREATEPARTITION;
                if (*CurrentPartition).LogicalPartition != 0 {
                    uID = STRING_INSTALLCREATELOGICAL;
                }
            }
            CONSOLE_SetStatusText(MUIGetString(uID));

            CONSOLE_ConInKey(Ir);

            if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
                && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3
            {
                if ConfirmQuit(Ir) != 0 {
                    DestroyPartitionList(PartitionList);
                    PartitionList = core::ptr::null_mut();
                    return QUIT_PAGE;
                }
                return SELECT_PARTITION_PAGE;
            } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
                && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_DOWN
            {
                ScrollUpDownPartitionList(&mut ListUi, (TRUE) as BOOLEAN);
            } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
                && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_UP
            {
                ScrollUpDownPartitionList(&mut ListUi, (FALSE) as BOOLEAN);
            } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_RETURN {
                ASSERT!(CurrentPartition);

                /* Don't select an extended partition for OS installation */
                if IsContainerPartition!((*CurrentPartition).PartitionType) != false {
                    continue;
                }

                /*
                 * Check whether the user wants to install ReactOS on a disk that
                 * is not recognized by the computer's firmware and if so, display
                 * a warning since such disks may not be bootable.
                 */
                if (*(*CurrentPartition).DiskEntry).MediaType == FixedMedia
                    && (*(*CurrentPartition).DiskEntry).BiosFound == 0
                {
                    PopupError(
                        b"The disk you have selected for installing ReactOS\nis not visible by the firmware of your computer,\nand so may not be bootable.\nPress ENTER to continue anyway.\0"
                            .as_ptr() as *const i8,
                        MUIGetString(STRING_CONTINUE),
                        Ir,
                        POPUP_WAIT_ENTER,
                    );
                    // return SELECT_PARTITION_PAGE;
                }

                break 'CreateInstallPartition;
            } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == 'C' as u32 {
                ASSERT!(CurrentPartition);

                Error = PartitionCreateChecks(CurrentPartition, 0, 0);
                if Error != NOT_AN_ERROR {
                    MUIDisplayError(Error, Ir, POPUP_WAIT_ANY_KEY);
                    return SELECT_PARTITION_PAGE;
                }

                PartCreateType = PartTypeData;
                return CREATE_PARTITION_PAGE;
            } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == 'E' as u32 {
                ASSERT!(CurrentPartition);

                /* Don't create an extended partition within a logical partition */
                if (*CurrentPartition).LogicalPartition != 0 {
                    continue;
                }

                Error = PartitionCreateChecks(
                    CurrentPartition,
                    0,
                    (PARTITION_EXTENDED) as ULONG_PTR,
                );
                if Error != NOT_AN_ERROR {
                    MUIDisplayError(Error, Ir, POPUP_WAIT_ANY_KEY);
                    return SELECT_PARTITION_PAGE;
                }

                PartCreateType = PartTypeExtended;
                return CREATE_PARTITION_PAGE;
            } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == 'D' as u32 {
                ASSERT!(CurrentPartition);

                /* Ignore deletion in case this is not a partitioned entry */
                if (*CurrentPartition).IsPartitioned == 0 {
                    continue;
                }

                if !(*CurrentPartition).Volume.is_null()
                    && (*(*CurrentPartition).Volume).New == 0
                    && (*(*CurrentPartition).Volume).FormatState != Unformatted
                {
                    let mut CurrentPartitionU: _UNICODE_STRING = core::mem::zeroed();
                    let mut PathBuffer: [u16; RTL_NUMBER_OF_FIELD!(
                        VOLINFO,
                        DeviceName,
                    ) + 1] = [0; RTL_NUMBER_OF_FIELD!(
                        VOLINFO,
                        DeviceName,
                    ) + 1];

                    ASSERT!((*CurrentPartition).PartitionNumber != 0);

                    RtlStringCchPrintfW!(
                        PathBuffer.as_mut_ptr(),
                        _countof!(PathBuffer),
                        RUSTCONV_WSTR_BCF3913447D5.as_ptr(),
                        (*(*CurrentPartition).Volume).Info.DeviceName,
                    );
                    RtlInitUnicodeString(&mut CurrentPartitionU, PathBuffer.as_mut_ptr());

                    /*
                     * Check whether the user attempts to delete the partition on which
                     * the installation source is present. If so, fail with an error.
                     */
                    if RtlPrefixUnicodeString(
                        &mut CurrentPartitionU,
                        &mut USetupData.SourcePath,
                        (TRUE) as BOOLEAN,
                    ) != 0
                    {
                        MUIDisplayError(ERROR_SOURCE_PATH, Ir, POPUP_WAIT_ENTER);
                        return SELECT_PARTITION_PAGE;
                    }
                }

                if CurrentPartition == (*PartitionList).SystemPartition
                    || IsPartitionActive(CurrentPartition) != 0
                {
                    return CONFIRM_DELETE_SYSTEM_PARTITION_PAGE;
                }

                return DELETE_PARTITION_PAGE;
            }
        }
    }

    // CreateInstallPartition:
    ASSERT!(CurrentPartition);
    ASSERT!(IsContainerPartition!((*CurrentPartition).PartitionType) == false);

    /* Create the partition if the selected region is empty */
    if (*CurrentPartition).IsPartitioned == 0 {
        Error = PartitionCreateChecks(CurrentPartition, 0, 0);
        if Error != NOT_AN_ERROR {
            MUIDisplayError(Error, Ir, POPUP_WAIT_ANY_KEY);
            return SELECT_PARTITION_PAGE;
        }

        /* Automatically create the partition on the whole empty space;
         * it will be formatted later with default parameters */
        CreatePartition(PartitionList, CurrentPartition, 0, 0);
        ASSERT!((*CurrentPartition).IsPartitioned);
        if !(*CurrentPartition).Volume.is_null() {
            (*(*CurrentPartition).Volume).New |= (VOLUME_NEW_AUTOCREATE) as u8;
        }
    }

    /* Verify the target medium size */
    MaxTargetSize = GetPartEntrySizeInBytes!(CurrentPartition);
    if IsMediumLargeEnough(MaxTargetSize) == 0 {
        MUIDisplayError(
            ERROR_INSUFFICIENT_PARTITION_SIZE,
            Ir,
            POPUP_WAIT_ANY_KEY,
            USetupData.RequiredPartitionDiskSpace,
        );
        return SELECT_PARTITION_PAGE; /* Let the user select another partition */
    }

    InstallPartition = CurrentPartition;
    return START_PARTITION_OPERATIONS_PAGE;
}

unsafe extern "C" fn ShowPartitionSizeInputBox(
    Left: i16,
    Top: i16,
    Right: i16,
    Bottom: i16,
    MaxSize: u32,
    InputBuffer: PWSTR,
    Quit: PBOOLEAN,
    Cancel: PBOOLEAN,
) {
    let mut Ir: INPUT_RECORD = core::mem::zeroed();
    let mut coPos: COORD = core::mem::zeroed();
    let mut Written: u32 = 0;
    let mut Buffer: [i8; 128] = [0; 128];
    let mut Length: i32;
    let mut Pos_local: i32;
    let mut ch: u16;
    let mut iLeft: i16;
    let mut iTop: i16;

    if !Quit.is_null() {
        *Quit = (FALSE) as u8;
    }

    if !Cancel.is_null() {
        *Cancel = (FALSE) as u8;
    }

    DrawBox(
        Left,
        Top,
        Right - Left + 1,
        Bottom - Top + 1,
    );

    coPos.X = Left + 2;
    coPos.Y = Top + 2;
    strcpy(Buffer.as_mut_ptr(), MUIGetString(STRING_PARTITIONSIZE));
    iLeft = coPos.X + strlen(Buffer.as_ptr()) as u16 as i16 + 1;
    iTop = coPos.Y;

    WriteConsoleOutputCharacterA(
        StdOutput,
        Buffer.as_ptr(),
        (strlen(Buffer.as_ptr())) as DWORD,
        coPos,
        &mut Written,
    );

    sprintf(Buffer.as_mut_ptr(), MUIGetString(STRING_MAXSIZE), MaxSize);
    coPos.X = iLeft + (PARTITION_SIZE_INPUT_FIELD_LENGTH) as i16 + 1;
    coPos.Y = iTop;
    WriteConsoleOutputCharacterA(
        StdOutput,
        Buffer.as_ptr(),
        (strlen(Buffer.as_ptr())) as DWORD,
        coPos,
        &mut Written,
    );

    _swprintf(InputBuffer, RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(), MaxSize);
    Length = (wcslen(InputBuffer)) as i32;
    Pos_local = Length;
    CONSOLE_SetInputTextXY(iLeft, iTop, (PARTITION_SIZE_INPUT_FIELD_LENGTH) as SHORT, InputBuffer);
    CONSOLE_SetCursorXY(iLeft + Length as i16, iTop);
    CONSOLE_SetCursorType((TRUE) as BOOL, (TRUE) as BOOL);

    loop {
        CONSOLE_ConInKey(&mut Ir);

        if Ir.Event.KeyEvent.uChar.AsciiChar == 0x00
            && (Ir.Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3
        {
            if !Quit.is_null() {
                *Quit = (TRUE) as u8;
            }

            *InputBuffer.add(0) = UNICODE_NULL;
            break;
        } else if (Ir.Event.KeyEvent.wVirtualKeyCode) as u32 == VK_RETURN {
            break;
        } else if (Ir.Event.KeyEvent.wVirtualKeyCode) as u32 == VK_ESCAPE {
            if !Cancel.is_null() {
                *Cancel = (TRUE) as u8;
            }

            *InputBuffer.add(0) = UNICODE_NULL;
            break;
        } else if Ir.Event.KeyEvent.uChar.AsciiChar == 0x00
            && (Ir.Event.KeyEvent.wVirtualKeyCode) as u32 == VK_HOME
        {
            Pos_local = 0;
            CONSOLE_SetCursorXY(iLeft + (Pos_local) as i16, iTop);
        } else if Ir.Event.KeyEvent.uChar.AsciiChar == 0x00
            && (Ir.Event.KeyEvent.wVirtualKeyCode) as u32 == VK_END
        {
            Pos_local = Length;
            CONSOLE_SetCursorXY(iLeft + (Pos_local) as i16, iTop);
        } else if Ir.Event.KeyEvent.uChar.AsciiChar == 0x00
            && (Ir.Event.KeyEvent.wVirtualKeyCode) as u32 == VK_LEFT
        {
            if Pos_local > 0 {
                Pos_local -= 1;
                CONSOLE_SetCursorXY(iLeft + (Pos_local) as i16, iTop);
            }
        } else if Ir.Event.KeyEvent.uChar.AsciiChar == 0x00
            && (Ir.Event.KeyEvent.wVirtualKeyCode) as u32 == VK_RIGHT
        {
            if Pos_local < Length {
                Pos_local += 1;
                CONSOLE_SetCursorXY(iLeft + (Pos_local) as i16, iTop);
            }
        } else if Ir.Event.KeyEvent.uChar.AsciiChar == 0x00
            && (Ir.Event.KeyEvent.wVirtualKeyCode) as u32 == VK_DELETE
        {
            if Pos_local < Length {
                memmove(
                    (InputBuffer.add((Pos_local) as usize)).cast(),
                    InputBuffer.add((Pos_local + 1) as usize).cast(),
                    (Length - Pos_local - 1) as usize
                        * core::mem::size_of::<u16>(),
                );
                *InputBuffer.add(Length as usize - 1) = UNICODE_NULL;

                Length -= 1;
                CONSOLE_SetInputTextXY(iLeft, iTop, (PARTITION_SIZE_INPUT_FIELD_LENGTH) as SHORT, InputBuffer);
                CONSOLE_SetCursorXY(iLeft + (Pos_local) as i16, iTop);
            }
        } else if (Ir.Event.KeyEvent.wVirtualKeyCode) as u32 == VK_BACK {
            if Pos_local > 0 {
                if Pos_local < Length {
                    memmove(
                        (InputBuffer.add((Pos_local - 1) as usize)).cast(),
                        InputBuffer.add((Pos_local) as usize).cast(),
                        (Length - Pos_local) as usize
                            * core::mem::size_of::<u16>(),
                    );
                }
                *InputBuffer.add(Length as usize - 1) = UNICODE_NULL;

                Pos_local -= 1;
                Length -= 1;
                CONSOLE_SetInputTextXY(iLeft, iTop, (PARTITION_SIZE_INPUT_FIELD_LENGTH) as SHORT, InputBuffer);
                CONSOLE_SetCursorXY(iLeft + (Pos_local) as i16, iTop);
            }
        } else if Ir.Event.KeyEvent.uChar.AsciiChar != 0x00 {
            if Length < PARTITION_SIZE_INPUT_FIELD_LENGTH - 1 {
                ch = Ir.Event.KeyEvent.uChar.AsciiChar as u16;

                if ch >= 0x0030u16 && ch <= 0x0039u16 {
                    if Pos_local < Length {
                        memmove(
                            (InputBuffer.add((Pos_local + 1) as usize)).cast(),
                            InputBuffer.add((Pos_local) as usize).cast(),
                            (Length - Pos_local) as usize
                                * core::mem::size_of::<u16>(),
                        );
                    }
                    *InputBuffer.add(Length as usize + 1) = UNICODE_NULL;
                    *InputBuffer.add((Pos_local) as usize) = ch;

                    Pos_local += 1;
                    Length += 1;
                    CONSOLE_SetInputTextXY(iLeft, iTop, (PARTITION_SIZE_INPUT_FIELD_LENGTH) as SHORT, InputBuffer);
                    CONSOLE_SetCursorXY(iLeft + (Pos_local) as i16, iTop);
                }
            }
        }
    }

    CONSOLE_SetCursorType((TRUE) as BOOL, (FALSE) as BOOL);
}

unsafe extern "C" fn CreatePartitionPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut PartEntry: PPARTENTRY;
    let mut DiskEntry: PDISKENTRY;
    let mut uID: ULONG;
    let mut MaxSize: ULONG;
    let mut MaxPartSize: ULONGLONG;
    let mut PartSize: ULONGLONG;
    let mut Quit: BOOLEAN = 0;
    let mut Cancel: BOOLEAN = 0;
    let mut InputBuffer: [WCHAR; 50] = [0; 50];
    let mut LineBuffer: [CHAR; 100] = [0; 100];

    if PartitionList.is_null() || CurrentPartition.is_null() {
        return QUIT_PAGE;
    }

    if PartCreateType == PartTypeData {
        uID = STRING_CHOOSE_NEW_PARTITION;
        if (*CurrentPartition).LogicalPartition != 0 {
            uID = STRING_CHOOSE_NEW_LOGICAL_PARTITION;
        }
    } else {
        uID = STRING_CHOOSE_NEW_EXTENDED_PARTITION;
    }

    CONSOLE_SetTextXY(6, 8, MUIGetString(uID));

    PartEntry = CurrentPartition;
    DiskEntry = (*CurrentPartition).DiskEntry;

    DiskDescription(DiskEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
    CONSOLE_PrintTextXY(
        6,
        10,
        MUIGetString(STRING_HDDISK1),
        LineBuffer.as_ptr(),
    );

    CONSOLE_SetTextXY(
        6,
        12,
        MUIGetString(STRING_HDPARTSIZE),
    );

    CONSOLE_SetStatusText(MUIGetString(STRING_CREATEPARTITION));

    MaxPartSize = GetPartEntrySizeInBytes!(PartEntry);

    loop {
        MaxSize = RoundingDivide(MaxPartSize, MB) as ULONG;
        // C: min(MaxSize, PARTITION_MAXSIZE) compares as double (pow()) and truncates back
        // into the ULONG; 99999999.0 and every ULONG are exact doubles, so the integer
        // min is the same value without floating point.
        MaxSize = min(MaxSize, PARTITION_MAXSIZE as ULONG);

        ShowPartitionSizeInputBox(
            12,
            14,
            xScreen - 12,
            17,
            MaxSize,
            InputBuffer.as_mut_ptr(),
            &mut Quit,
            &mut Cancel,
        );
        if Quit != 0 {
            if ConfirmQuit(Ir) != 0 {
                return QUIT_PAGE;
            }
            break;
        } else if Cancel != 0 {
            return SELECT_PARTITION_PAGE;
        }

        PartSize = _wcstoui64(InputBuffer.as_ptr(), core::ptr::null_mut(), 10);

        if (PartSize < 1) || (PartSize > MaxSize as ULONGLONG) {
            continue;
        }

        if PartSize == MaxSize as ULONGLONG {
            PartSize = MaxPartSize;
        } else {
            PartSize *= MB;
        }
        DPRINT!("Partition size: %I64u bytes\n", PartSize);

        CreatePartition(
            PartitionList,
            CurrentPartition,
            PartSize,
            if PartCreateType == PartTypeData {
                0
            } else {
                (PARTITION_EXTENDED) as _
            },
        );

        return SELECT_PARTITION_PAGE;
    }

    return CREATE_PARTITION_PAGE;
}

unsafe extern "C" fn ConfirmDeleteSystemPartitionPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    MUIDisplayPage(CONFIRM_DELETE_SYSTEM_PARTITION_PAGE);

    loop {
        CONSOLE_ConInKey(Ir);

        if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
            (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3)
        {
            if ConfirmQuit(Ir) != 0 {
                return QUIT_PAGE;
            }
            break;
        } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_RETURN {
            return DELETE_PARTITION_PAGE;
        } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_ESCAPE {
            return SELECT_PARTITION_PAGE;
        }
    }

    return CONFIRM_DELETE_SYSTEM_PARTITION_PAGE;
}

unsafe extern "C" fn DeletePartitionPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut PartEntry: PPARTENTRY;
    let mut DiskEntry: PDISKENTRY;
    let mut LineBuffer: [i8; 100] = [0; 100];

    if PartitionList.is_null() || CurrentPartition.is_null() {
        return QUIT_PAGE;
    }

    PartEntry = CurrentPartition;
    DiskEntry = (*CurrentPartition).DiskEntry;

    MUIDisplayPage(DELETE_PARTITION_PAGE);

    PartitionDescription(PartEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
    CONSOLE_SetTextXY(6, 10, LineBuffer.as_ptr());

    DiskDescription(DiskEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
    CONSOLE_PrintTextXY(
        6,
        12,
        MUIGetString(STRING_HDDISK2),
        LineBuffer.as_ptr(),
    );

    loop {
        CONSOLE_ConInKey(Ir);

        if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3
        {
            if ConfirmQuit(Ir) != 0 {
                return QUIT_PAGE;
            }
            break;
        } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_ESCAPE {
            return SELECT_PARTITION_PAGE;
        } else if (*Ir).Event.KeyEvent.wVirtualKeyCode == 'L' as u16 {
            DeletePartition(
                PartitionList,
                CurrentPartition,
                &mut CurrentPartition,
            );
            return SELECT_PARTITION_PAGE;
        }
    }

    return DELETE_PARTITION_PAGE;
}

unsafe extern "C" fn StartPartitionOperationsPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut FsVolContext: FSVOL_CONTEXT = FSVOL_CONTEXT {
        Ir,
        NextPageOnAbort: QUIT_PAGE,
    };
    let mut Success: BOOLEAN;

    if PartitionList.is_null() || InstallPartition.is_null() {
        return QUIT_PAGE;
    }

    Success = InitSystemPartition(
        PartitionList,
        InstallPartition,
        &mut SystemPartition,
        Some(FsVolCallback),
        &mut FsVolContext as *mut FSVOL_CONTEXT as *mut core::ffi::c_void,
    );
    if Success == 0 {
        return FsVolContext.NextPageOnAbort;
    }

    if (*SystemPartition).New != 0 && !(*SystemPartition).Volume.is_null() {
        (*(*SystemPartition).Volume).New |= (VOLUME_NEW_AUTOCREATE) as u8;
    }

    CONSOLE_ClearScreen();
    CONSOLE_Flush();

    Success = FsVolCommitOpsQueue(
        PartitionList,
        (*SystemPartition).Volume,
        (*InstallPartition).Volume,
        Some(FsVolCallback),
        &mut FsVolContext as *mut FSVOL_CONTEXT as *mut core::ffi::c_void,
    );
    if Success == 0 {
        return FsVolContext.NextPageOnAbort;
    }
    return BOOTLOADER_SELECT_PAGE;
}

unsafe extern "C" fn ChangeSystemPartitionPage(
    Ir: *mut INPUT_RECORD,
    // C names this parameter `SystemPartition`, shadowing the file-static of that name;
    // Rust cannot shadow a static, so it is renamed. It is only ever the parameter here.
    SystemPartitionArg: *mut PARTENTRY,
) -> BOOLEAN {
    let mut PartEntry: *mut PARTENTRY;
    let mut DiskEntry: *mut DISKENTRY;
    let mut LineBuffer: [i8; 100] = [0; 100];

    MUIDisplayPage(CHANGE_SYSTEM_PARTITION);

    PartEntry = (*PartitionList).SystemPartition;
    DiskEntry = (*PartEntry).DiskEntry;

    PartitionDescription(PartEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
    CONSOLE_SetTextXY(8, 10, LineBuffer.as_ptr());

    DiskDescription(DiskEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
    CONSOLE_PrintTextXY(
        8,
        14,
        MUIGetString(STRING_HDDISK1),
        LineBuffer.as_ptr(),
    );

    PartEntry = SystemPartitionArg;
    DiskEntry = (*PartEntry).DiskEntry;

    PartitionDescription(PartEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
    CONSOLE_SetTextXY(8, 23, LineBuffer.as_ptr());

    loop {
        CONSOLE_ConInKey(Ir);

        if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_RETURN {
            break;
        } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_ESCAPE {
            return (FALSE) as BOOLEAN;
        }
    }

    return (TRUE) as BOOLEAN;
}

unsafe extern "C" fn ResetFileSystemList() {
    if FileSystemList.is_null() {
        return;
    }

    DestroyFileSystemList(FileSystemList);
    FileSystemList = core::ptr::null_mut();
}

unsafe extern "C" fn SelectFileSystemPage(
    FsVolContext: *mut FSVOL_CONTEXT,
    Volume: *mut VOLENTRY,
) -> FSVOL_OP {
    let Ir: *mut INPUT_RECORD = (*FsVolContext).Ir;
    let PartEntry: *mut PARTENTRY = (*Volume).PartEntry;
    let DiskEntry: *mut DISKENTRY = (*PartEntry).DiskEntry;
    let mut DefaultFs: PCWSTR;
    let ForceFormat: BOOLEAN;
    let mut LineBuffer: [i8; 100] = [0; 100];

    DPRINT!("SelectFileSystemPage()\n");

    ForceFormat = ((*Volume).New != 0 || (*Volume).FormatState == Unformatted) as BOOLEAN;

    'Restart: loop {
        ResetFileSystemList();

        CONSOLE_ClearScreen();
        CONSOLE_Flush();
        MUIDisplayPage(SELECT_FILE_SYSTEM_PAGE);

        if ((*Volume).New as i32 & VOLUME_NEW_AUTOCREATE) != 0 {
            (*Volume).New &= (!VOLUME_NEW_AUTOCREATE) as u8;

            CONSOLE_SetTextXY(
                6,
                8,
                MUIGetString(STRING_NEWPARTITION),
            );
        } else if (*Volume).New != 0 {
            let uID: ULONG;

            if Volume == (*SystemPartition).Volume {
                uID = STRING_NONFORMATTEDSYSTEMPART;
            } else if Volume == (*InstallPartition).Volume {
                uID = STRING_NONFORMATTEDPART;
            } else {
                uID = STRING_NONFORMATTEDOTHERPART;
            }

            CONSOLE_SetTextXY(
                6,
                8,
                MUIGetString(uID),
            );
        } else {
            CONSOLE_SetTextXY(
                6,
                8,
                MUIGetString(STRING_INSTALLONPART),
            );
        }

        PartitionDescription(PartEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
        CONSOLE_SetTextXY(6, 10, LineBuffer.as_ptr());

        DiskDescription(DiskEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
        CONSOLE_PrintTextXY(
            6,
            12,
            MUIGetString(STRING_HDDISK2),
            LineBuffer.as_ptr(),
        );

        if ForceFormat != 0 {
            CONSOLE_SetTextXY(
                6,
                14,
                MUIGetString(STRING_PARTFORMAT),
            );
        }

        if IsUnattendedSetup != 0 {
            match USetupData.FsType {
                1 => {
                    DefaultFs = RUSTCONV_WSTR_BE0DEB469A7B.as_ptr();
                }
                _ => {
                    DefaultFs = RUSTCONV_WSTR_A953DDDD78B1.as_ptr();
                }
            }
        } else {
            DefaultFs = RUSTCONV_WSTR_A953DDDD78B1.as_ptr();
        }

        FileSystemList = CreateFileSystemList(
            6,
            26,
            ForceFormat,
            DefaultFs,
        );
        if FileSystemList.is_null() {
            (*FsVolContext).NextPageOnAbort = QUIT_PAGE;
            return FSVOL_ABORT;
        }

        if IsUnattendedSetup != 0 {
            if USetupData.FormatPartition != 0 {
                return FSVOL_DOIT;
            } else if ForceFormat == 0 {
                return FSVOL_SKIP;
            }
        }

        DrawFileSystemList(FileSystemList);

        loop {
            CONSOLE_ConInKey(Ir);

            if (*Ir).Event.KeyEvent.uChar.AsciiChar as u32 == 0x00
                && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3
            {
                if ConfirmQuit(Ir) != 0 {
                    (*FsVolContext).NextPageOnAbort = QUIT_PAGE;
                    return FSVOL_ABORT;
                }
                break;
            } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_ESCAPE {
                (*FsVolContext).NextPageOnAbort = SELECT_PARTITION_PAGE;
                return FSVOL_ABORT;
            } else if (*Ir).Event.KeyEvent.uChar.AsciiChar as u32 == 0x00
                && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_DOWN
            {
                ScrollDownFileSystemList(FileSystemList);
            } else if (*Ir).Event.KeyEvent.uChar.AsciiChar as u32 == 0x00
                && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_UP
            {
                ScrollUpFileSystemList(FileSystemList);
            } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_RETURN {
                if (*(*FileSystemList).Selected).FileSystem.is_null() {
                    if Volume != (*SystemPartition).Volume && Volume != (*InstallPartition).Volume {
                        (*Volume).NeedsCheck = (FALSE) as u8;
                    }

                    return FSVOL_SKIP;
                } else {
                    return FSVOL_DOIT;
                }
            }
        }

        continue 'Restart;
    }
}

unsafe extern "C" fn FormatPartitionPage(
    FsVolContext: *mut FSVOL_CONTEXT,
    Volume: *mut VOLENTRY,
) -> FSVOL_OP {
    let Ir: *mut INPUT_RECORD = (*FsVolContext).Ir;
    let PartEntry: *mut PARTENTRY = (*Volume).PartEntry;
    let DiskEntry: *mut DISKENTRY = (*PartEntry).DiskEntry;
    let mut LineBuffer: [i8; 100] = [0; 100];

    'Restart: loop {
        CONSOLE_ClearScreen();
        CONSOLE_Flush();
        MUIDisplayPage(FORMAT_PARTITION_PAGE);

        PartitionDescription(PartEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
        CONSOLE_SetTextXY(6, 10, LineBuffer.as_ptr());

        DiskDescription(DiskEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
        CONSOLE_PrintTextXY(
            6,
            12,
            MUIGetString(STRING_HDDISK2),
            LineBuffer.as_ptr(),
        );

        loop {
            if IsUnattendedSetup == 0 {
                CONSOLE_ConInKey(Ir);
            }

            if ((*Ir).Event.KeyEvent.uChar.AsciiChar as i32 == 0x00)
                && (((*Ir).Event.KeyEvent.wVirtualKeyCode as i32) as u32 == VK_F3)
            {
                if ConfirmQuit(Ir) != 0 {
                    (*FsVolContext).NextPageOnAbort = QUIT_PAGE;
                    return FSVOL_ABORT;
                }
                continue 'Restart;
            } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode as i32) as u32 == VK_ESCAPE {
                (*FsVolContext).NextPageOnAbort = SELECT_PARTITION_PAGE;
                return FSVOL_ABORT;
            } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode as i32) as u32 == VK_RETURN
                || IsUnattendedSetup != 0
            {
                MUIClearStyledText(FORMAT_PARTITION_PAGE, (TEXT_ID_FORMAT_PROMPT) as INT, (TEXT_TYPE_REGULAR) as INT);
                CONSOLE_SetStatusText(MUIGetString(STRING_PLEASEWAIT));

                return FSVOL_DOIT;
            }
        }
    }
}

unsafe extern "C" fn CheckFileSystemPage(Volume: *mut VOLENTRY) {
    let PartEntry = (*Volume).PartEntry;
    let DiskEntry = (*PartEntry).DiskEntry;
    let mut LineBuffer: [i8; 100] = [0; 100];

    CONSOLE_ClearScreen();
    CONSOLE_Flush();
    MUIDisplayPage(CHECK_FILE_SYSTEM_PAGE);

    PartitionDescription(PartEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
    CONSOLE_SetTextXY(6, 10, LineBuffer.as_ptr());

    DiskDescription(DiskEntry, LineBuffer.as_mut_ptr(), (ARRAYSIZE!(LineBuffer)) as _);
    CONSOLE_PrintTextXY(6, 12, MUIGetString(STRING_HDDISK2),
                        LineBuffer.as_ptr());
}

unsafe extern "system" fn FsVolCallback(
    Context: *mut core::ffi::c_void,
    FormatStatus: FSVOLNOTIFY,
    Param1: ULONG_PTR,
    Param2: ULONG_PTR,
) -> FSVOL_OP {
    unsafe {
        let FsVolContext: *mut FSVOL_CONTEXT = Context as *mut FSVOL_CONTEXT;
        let Ir: *mut INPUT_RECORD = (*FsVolContext).Ir;

        match FormatStatus {
            ChangeSystemPartition => {
                // C: `PPARTENTRY SystemPartition` (shadows the file-static; renamed).
                let SystemPartitionArg: *mut PARTENTRY = Param1 as *mut PARTENTRY;

                (*FsVolContext).NextPageOnAbort = SELECT_PARTITION_PAGE;
                if ChangeSystemPartitionPage(Ir, SystemPartitionArg) != 0 {
                    return FSVOL_DOIT;
                }
                return FSVOL_ABORT;
            }

            FSVOLNOTIFY_PARTITIONERROR => {
                match (Param1) as i32 {
                    STATUS_PARTITION_FAILURE => {
                        MUIDisplayError(ERROR_WRITE_PTABLE, Ir, POPUP_WAIT_ENTER);
                        (*FsVolContext).NextPageOnAbort = QUIT_PAGE;
                    }

                    ERROR_SYSTEM_PARTITION_NOT_FOUND => {
                        PopupError(
                            b"The ReactOS Setup could not find a supported system partition\non your system or could not create a new one. Without such a partition\nthe Setup program cannot install ReactOS.\nPress ENTER to return to the partition selection list.\0"
                                .as_ptr() as *const i8,
                            MUIGetString(STRING_CONTINUE),
                            Ir,
                            POPUP_WAIT_ENTER,
                        );

                        (*FsVolContext).NextPageOnAbort = SELECT_PARTITION_PAGE;
                    }

                    _ => {}
                }
                return FSVOL_ABORT;
            }

            FSVOLNOTIFY_STARTQUEUE | FSVOLNOTIFY_ENDQUEUE => {
                return FSVOL_DOIT;
            }

            FSVOLNOTIFY_STARTSUBQUEUE => {
                if (Param1 as FSVOL_OP) == FSVOL_FORMAT {
                    if RepairUpdateFlag != 0 {
                        return FSVOL_SKIP;
                    }
                }
                return FSVOL_DOIT;
            }

            FSVOLNOTIFY_ENDSUBQUEUE => {
                return 0;
            }

            FSVOLNOTIFY_FORMATERROR => {
                let FmtInfo: *mut FORMAT_VOLUME_INFO = Param1 as *mut FORMAT_VOLUME_INFO;
                let mut Buffer: [i8; MAX_PATH as usize] =
                    [0; MAX_PATH as usize];

                if (*FmtInfo).ErrorStatus == STATUS_PARTITION_FAILURE {
                    MUIDisplayError(ERROR_WRITE_PTABLE, Ir, POPUP_WAIT_ENTER);
                    (*FsVolContext).NextPageOnAbort = QUIT_PAGE;
                    return FSVOL_ABORT;
                } else if (*FmtInfo).ErrorStatus == STATUS_UNRECOGNIZED_VOLUME {
                    (*FsVolContext).NextPageOnAbort = QUIT_PAGE;
                    return FSVOL_ABORT;
                } else if (*FmtInfo).ErrorStatus == STATUS_NOT_SUPPORTED {
                    RtlStringCbPrintfA!(
                        Buffer.as_mut_ptr(),
                        core::mem::size_of_val(&Buffer),
                        b"Setup is currently unable to format a partition in %S.\n\n  \x07  Press ENTER to continue Setup.\n  \x07  Press F3 to quit Setup.\0".as_ptr() as *const i8,
                        (*FmtInfo).FileSystemName,
                    );

                    PopupError(
                        Buffer.as_mut_ptr(),
                        MUIGetString(STRING_QUITCONTINUE),
                        core::ptr::null_mut(),
                        POPUP_WAIT_NONE,
                    );

                    loop {
                        CONSOLE_ConInKey(Ir);

                        if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
                            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3
                        {
                            if ConfirmQuit(Ir) != 0 {
                                (*FsVolContext).NextPageOnAbort = QUIT_PAGE;
                                return FSVOL_ABORT;
                            }
                            return FSVOL_RETRY;
                        } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar) as u32 == VK_RETURN {
                            return FSVOL_RETRY;
                        }
                    }
                } else if ((NT_SUCCESS!((*FmtInfo).ErrorStatus)) == 0) {
                    DPRINT1!(
                        b"FormatPartition() failed: Status 0x%08lx\n\0".as_ptr() as *const i8,
                        (*FmtInfo).ErrorStatus,
                    );
                    MUIDisplayError(
                        ERROR_FORMATTING_PARTITION,
                        Ir,
                        POPUP_WAIT_ANY_KEY,
                        (*(*FmtInfo).Volume).Info.DeviceName,
                    );
                    (*FsVolContext).NextPageOnAbort = QUIT_PAGE;
                    return FSVOL_ABORT;
                }
                return FSVOL_RETRY;
            }

            FSVOLNOTIFY_CHECKERROR => {
                let ChkInfo: *mut CHECK_VOLUME_INFO = Param1 as *mut CHECK_VOLUME_INFO;
                let mut Buffer: [i8; MAX_PATH as usize] =
                    [0; MAX_PATH as usize];

                if (*ChkInfo).ErrorStatus == STATUS_NOT_SUPPORTED {
                    RtlStringCbPrintfA!(
                        Buffer.as_mut_ptr(),
                        core::mem::size_of_val(&Buffer),
                        b"Setup is currently unable to check a partition formatted in %S.\n\n  \x07  Press ENTER to continue Setup.\n  \x07  Press F3 to quit Setup.\0".as_ptr() as *const i8,
                        (*(*ChkInfo).Volume).Info.FileSystem,
                    );

                    PopupError(
                        Buffer.as_mut_ptr(),
                        MUIGetString(STRING_QUITCONTINUE),
                        core::ptr::null_mut(),
                        POPUP_WAIT_NONE,
                    );

                    loop {
                        CONSOLE_ConInKey(Ir);

                        if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00
                            && ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3
                        {
                            if ConfirmQuit(Ir) != 0 {
                                (*FsVolContext).NextPageOnAbort = QUIT_PAGE;
                                return FSVOL_ABORT;
                            }
                            return FSVOL_SKIP;
                        } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar) as u32 == VK_RETURN {
                            return FSVOL_SKIP;
                        }
                    }
                } else if ((NT_SUCCESS!((*ChkInfo).ErrorStatus)) == 0) {
                    DPRINT1!(
                        b"ChkdskPartition() failed: Status 0x%08lx\n\0".as_ptr() as *const i8,
                        (*ChkInfo).ErrorStatus,
                    );

                    RtlStringCbPrintfA!(
                        Buffer.as_mut_ptr(),
                        core::mem::size_of_val(&Buffer),
                        b"ChkDsk detected some disk errors.\n(Status 0x%08lx).\n\0".as_ptr() as *const i8,
                        (*ChkInfo).ErrorStatus,
                    );

                    PopupError(
                        Buffer.as_mut_ptr(),
                        MUIGetString(STRING_CONTINUE),
                        Ir,
                        POPUP_WAIT_ENTER,
                    );
                    return FSVOL_SKIP;
                }
                return FSVOL_SKIP;
            }

            FSVOLNOTIFY_STARTFORMAT => {
                let FmtInfo: *mut FORMAT_VOLUME_INFO = Param1 as *mut FORMAT_VOLUME_INFO;
                let mut Result: FSVOL_OP;

                Result = SelectFileSystemPage(FsVolContext, (*FmtInfo).Volume);
                if Result != FSVOL_DOIT {
                    return Result;
                }

                Result = FormatPartitionPage(FsVolContext, (*FmtInfo).Volume);
                if Result != FSVOL_DOIT {
                    return Result;
                }

                StartFormat(FmtInfo, (*FileSystemList).Selected);
                return FSVOL_DOIT;
            }

            FSVOLNOTIFY_ENDFORMAT => {
                let FmtInfo: *mut FORMAT_VOLUME_INFO = Param1 as *mut FORMAT_VOLUME_INFO;
                EndFormat((*FmtInfo).ErrorStatus);

                ResetFileSystemList();
                return 0;
            }

            FSVOLNOTIFY_STARTCHECK => {
                let ChkInfo: *mut CHECK_VOLUME_INFO = Param1 as *mut CHECK_VOLUME_INFO;

                CheckFileSystemPage((*ChkInfo).Volume);
                StartCheck(ChkInfo);
                return FSVOL_DOIT;
            }

            FSVOLNOTIFY_ENDCHECK => {
                let ChkInfo: *mut CHECK_VOLUME_INFO = Param1 as *mut CHECK_VOLUME_INFO;
                EndCheck((*ChkInfo).ErrorStatus);
                return 0;
            }

            _ => {
                return 0;
            }
        }
    }
}

unsafe extern "C" fn InstallDirectoryPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut Status: i32;
    let mut Length: u32;
    let mut Pos_local: u32;
    let mut c: u16;
    let mut InstallDir: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    'InitInstallDir: {
        if PartitionList.is_null() || InstallPartition.is_null() {
            return QUIT_PAGE;
        }

        if RepairUpdateFlag != 0 {
            wcscpy(
                core::ptr::addr_of_mut!(InstallDir) as *mut u16,
                (*CurrentInstallation).PathComponent,
            );
        } else if USetupData.InstallationDirectory[0 as usize] != 0 {
            wcscpy(
                core::ptr::addr_of_mut!(InstallDir) as *mut u16,
                USetupData.InstallationDirectory.as_ptr(),
            );
        } else {
            wcscpy(
                core::ptr::addr_of_mut!(InstallDir) as *mut u16,
                RUSTCONV_WSTR_28AC66B7B537.as_ptr(),
            );
        }

        if RepairUpdateFlag != 0 || IsUnattendedSetup != 0 {
            if IsValidInstallDirectory(core::ptr::addr_of!(InstallDir) as *const u16) != 0 {
                break 'InitInstallDir;
            }

            MUIDisplayError(ERROR_DIRECTORY_NAME, Ir, POPUP_WAIT_ENTER);
        }

        Length = wcslen(core::ptr::addr_of!(InstallDir) as *const u16) as u32;
        Pos_local = Length;

        MUIDisplayPage(INSTALL_DIRECTORY_PAGE);
        CONSOLE_SetInputTextXY(
            8,
            11,
            51,
            core::ptr::addr_of!(InstallDir) as *const u16,
        );
        CONSOLE_SetCursorXY(
            (8 + Pos_local) as SHORT,
            11,
        );
        CONSOLE_SetCursorType((TRUE) as BOOL, (TRUE) as BOOL);

        while 1 != 0 {
            CONSOLE_ConInKey(Ir);

            if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
                (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3)
            {
                CONSOLE_SetCursorType((TRUE) as BOOL, (FALSE) as BOOL);

                if ConfirmQuit(Ir) != 0 {
                    return QUIT_PAGE;
                }
                return INSTALL_DIRECTORY_PAGE;
            } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
                (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_DELETE)
            {
                if Pos_local < Length {
                    memmove(
                        (core::ptr::addr_of_mut!(InstallDir) as *mut u16)
                            .add(Pos_local as usize)
                            as *mut core::ffi::c_void,
                        (core::ptr::addr_of_mut!(InstallDir) as *mut u16)
                            .add((Pos_local + 1) as usize)
                            as *const core::ffi::c_void,
                        ((Length - Pos_local - 1) as usize)
                            * core::mem::size_of::<u16>(),
                    );
                    *(core::ptr::addr_of_mut!(InstallDir) as *mut u16)
                        .add((Length - 1) as usize) =
                        UNICODE_NULL as u16;

                    Length -= 1;
                    CONSOLE_SetInputTextXY(
                        8,
                        11,
                        51,
                        core::ptr::addr_of!(InstallDir) as *const u16,
                    );
                    CONSOLE_SetCursorXY(
                        (8 + Pos_local) as SHORT,
                        11,
                    );
                }
            } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
                (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_HOME)
            {
                Pos_local = 0;
                CONSOLE_SetCursorXY(
                    (8 + Pos_local) as SHORT,
                    11,
                );
            } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
                (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_END)
            {
                Pos_local = Length;
                CONSOLE_SetCursorXY(
                    (8 + Pos_local) as SHORT,
                    11,
                );
            } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
                (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_LEFT)
            {
                if Pos_local > 0 {
                    Pos_local -= 1;
                    CONSOLE_SetCursorXY(
                        (8 + Pos_local) as SHORT,
                        11,
                    );
                }
            } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
                (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_RIGHT)
            {
                if Pos_local < Length {
                    Pos_local += 1;
                    CONSOLE_SetCursorXY(
                        (8 + Pos_local) as SHORT,
                        11,
                    );
                }
            } else if ((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_ESCAPE {
                *(core::ptr::addr_of_mut!(InstallDir) as *mut u16) = UNICODE_NULL as u16;
                Length = 0;
                Pos_local = Length;
                CONSOLE_SetInputTextXY(
                    8,
                    11,
                    51,
                    core::ptr::addr_of!(InstallDir) as *const u16,
                );
                CONSOLE_SetCursorXY(
                    (8 + Pos_local) as SHORT,
                    11,
                );
            } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x0D {
                CONSOLE_SetCursorType((TRUE) as BOOL, (FALSE) as BOOL);

                if IsValidInstallDirectory(core::ptr::addr_of!(InstallDir) as *const u16) != 0 {
                    break 'InitInstallDir;
                }

                MUIDisplayError(ERROR_DIRECTORY_NAME, Ir, POPUP_WAIT_ENTER);
                return INSTALL_DIRECTORY_PAGE;
            } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x08 {
                if Pos_local > 0 {
                    if Pos_local < Length {
                        memmove(
                            (core::ptr::addr_of_mut!(InstallDir) as *mut u16)
                                .add((Pos_local - 1) as usize)
                                as *mut core::ffi::c_void,
                            (core::ptr::addr_of_mut!(InstallDir) as *mut u16)
                                .add(Pos_local as usize)
                                as *const core::ffi::c_void,
                            ((Length - Pos_local) as usize)
                                * core::mem::size_of::<u16>(),
                        );
                    }
                    *(core::ptr::addr_of_mut!(InstallDir) as *mut u16)
                        .add((Length - 1) as usize) =
                        UNICODE_NULL as u16;

                    Pos_local -= 1;
                    Length -= 1;
                    CONSOLE_SetInputTextXY(
                        8,
                        11,
                        51,
                        core::ptr::addr_of!(InstallDir) as *const u16,
                    );
                    CONSOLE_SetCursorXY(
                        (8 + Pos_local) as SHORT,
                        11,
                    );
                }
            } else if isprint((*Ir).Event.KeyEvent.uChar.AsciiChar as i32) != 0 {
                if Length < 50 {
                    c = (*Ir).Event.KeyEvent.uChar.AsciiChar as u16;
                    if IS_VALID_INSTALL_PATH_CHAR!(c) {
                        if Pos_local < Length {
                            memmove(
                                (core::ptr::addr_of_mut!(InstallDir) as *mut u16)
                                    .add((Pos_local + 1) as usize)
                                    as *mut core::ffi::c_void,
                                (core::ptr::addr_of_mut!(InstallDir) as *mut u16)
                                    .add(Pos_local as usize)
                                    as *const core::ffi::c_void,
                                ((Length - Pos_local) as usize)
                                    * core::mem::size_of::<u16>(),
                            );
                        }
                        *(core::ptr::addr_of_mut!(InstallDir) as *mut u16)
                            .add((Length + 1) as usize) =
                            UNICODE_NULL as u16;
                        *(core::ptr::addr_of_mut!(InstallDir) as *mut u16)
                            .add(Pos_local as usize) = c;

                        Pos_local += 1;
                        Length += 1;
                        CONSOLE_SetInputTextXY(
                            8,
                            11,
                            51,
                            core::ptr::addr_of!(InstallDir) as *const u16,
                        );
                        CONSOLE_SetCursorXY(
                            (8 + Pos_local) as SHORT,
                            11,
                        );
                    }
                }
            }
        }
    }

    Status = InitDestinationPaths(
        core::ptr::addr_of_mut!(USetupData),
        core::ptr::addr_of!(InstallDir) as *const u16,
        (*InstallPartition).Volume,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"InitDestinationPaths() failed: Status 0x%lx\n\0".as_ptr() as *const i8,
            Status,
        );
        MUIDisplayError(ERROR_NO_BUILD_PATH, Ir, POPUP_WAIT_ENTER);
        return QUIT_PAGE;
    }

    if RtlPrefixUnicodeString(
        core::ptr::addr_of_mut!(USetupData.SourcePath),
        core::ptr::addr_of_mut!(USetupData.DestinationPath),
        (TRUE) as BOOLEAN,
    ) != 0
    {
        MUIDisplayError(ERROR_SOURCE_DIR, Ir, POPUP_WAIT_ENTER);
        return INSTALL_DIRECTORY_PAGE;
    }

    return PREPARE_COPY_PAGE;
}

unsafe extern "C" fn PrepareCopyPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let Success: BOOLEAN;

    MUIDisplayPage(PREPARE_COPY_PAGE);

    Success = PrepareFileCopy(&mut USetupData, None);
    // C: if (!Success). `!` on a u8 is BITWISE: !1 == 0xFE, so success quit setup.
    if Success == 0 {
        return QUIT_PAGE;
    }

    return FILE_COPY_PAGE;
}

unsafe extern "C" fn SetupUpdateMemoryInfo(
    CopyContext: *mut COPYCONTEXT,
    First: i32,
) {
    let mut PerfInfo: SYSTEM_PERFORMANCE_INFORMATION = core::mem::zeroed();

    NtQuerySystemInformation(
        SystemPerformanceInformation,
        &mut PerfInfo as *mut SYSTEM_PERFORMANCE_INFORMATION as *mut core::ffi::c_void,
        (core::mem::size_of::<SYSTEM_PERFORMANCE_INFORMATION>()) as ULONG,
        core::ptr::null_mut(),
    );

    if First != 0 {
        ProgressSetStepCount(
            (*CopyContext).MemoryBars[0],
            PerfInfo.CommitLimit,
        );
        ProgressSetStepCount(
            (*CopyContext).MemoryBars[1],
            PerfInfo.CommitLimit,
        );
        ProgressSetStepCount(
            (*CopyContext).MemoryBars[2],
            PerfInfo.CommitLimit,
        );
    }

    ProgressSetStep(
        (*CopyContext).MemoryBars[0],
        PerfInfo.PagedPoolPages + PerfInfo.NonPagedPoolPages,
    );
    ProgressSetStep(
        (*CopyContext).MemoryBars[1],
        PerfInfo.ResidentSystemCachePage,
    );
    ProgressSetStep(
        (*CopyContext).MemoryBars[2],
        PerfInfo.AvailablePages,
    );
}

unsafe extern "system" fn FileCopyCallback(
    Context: *mut core::ffi::c_void,
    Notification: u32,
    Param1: UINT_PTR,
    mut Param2: UINT_PTR,
) -> u32 {
    let CopyContext: *mut COPYCONTEXT = Context as *mut COPYCONTEXT;
    let FilePathInfo: PFILEPATHS_W;
    let mut SrcFileName: PCWSTR;
    let mut DstFileName: PCWSTR;

    match Notification {
        SPFILENOTIFY_STARTSUBQUEUE => {
            (*CopyContext).TotalOperations = Param2 as u32;
            (*CopyContext).CompletedOperations = 0;
            ProgressSetStepCount((*CopyContext).ProgressBar, (*CopyContext).TotalOperations);
            SetupUpdateMemoryInfo(CopyContext, (TRUE) as i32);
        }

        SPFILENOTIFY_STARTDELETE | SPFILENOTIFY_STARTRENAME | SPFILENOTIFY_STARTCOPY => {
            FilePathInfo = Param1 as PFILEPATHS_W;

            if Notification == SPFILENOTIFY_STARTDELETE {
                DstFileName = wcsrchr((*FilePathInfo).Target, 0x005Cu16);
                if !DstFileName.is_null() {
                    DstFileName = DstFileName.add(1);
                } else {
                    DstFileName = (*FilePathInfo).Target;
                }

                CONSOLE_SetStatusText(
                    MUIGetString(STRING_DELETING),
                    DstFileName,
                );
            } else if Notification == SPFILENOTIFY_STARTRENAME {
                SrcFileName = wcsrchr((*FilePathInfo).Source, 0x005Cu16);
                if !SrcFileName.is_null() {
                    SrcFileName = SrcFileName.add(1);
                } else {
                    SrcFileName = (*FilePathInfo).Source;
                }

                DstFileName = wcsrchr((*FilePathInfo).Target, 0x005Cu16);
                if !DstFileName.is_null() {
                    DstFileName = DstFileName.add(1);
                } else {
                    DstFileName = (*FilePathInfo).Target;
                }

                if _wcsicmp(SrcFileName, DstFileName) == 0 {
                    Param2 = (STRING_MOVING) as _;
                } else {
                    Param2 = (STRING_RENAMING) as _;
                }

                CONSOLE_SetStatusText(
                    MUIGetString((Param2) as ULONG),
                    SrcFileName,
                    DstFileName,
                );
            } else if Notification == SPFILENOTIFY_STARTCOPY {
                static mut s_pszCopying: PCSTR = core::ptr::null_mut();

                DstFileName = wcsrchr((*FilePathInfo).Target, 0x005Cu16);
                if !DstFileName.is_null() {
                    DstFileName = DstFileName.add(1);
                } else {
                    DstFileName = (*FilePathInfo).Target;
                }

                if s_pszCopying.is_null() {
                    s_pszCopying = MUIGetString(STRING_COPYING);
                }
                CONSOLE_SetStatusText(s_pszCopying, DstFileName);
                DoWatchDestFileName(DstFileName);
            }

            SetupUpdateMemoryInfo(CopyContext, (FALSE) as i32);
        }

        SPFILENOTIFY_COPYERROR => {
            FilePathInfo = Param1 as PFILEPATHS_W;

            DPRINT1!(
                b"An error happened while trying to copy file '%S' (error 0x%08lx), skipping it...\n\0"
                    .as_ptr() as *const i8,
                (*FilePathInfo).Target,
                (*FilePathInfo).Win32Error,
            );
            return FILEOP_SKIP;
        }

        SPFILENOTIFY_ENDDELETE | SPFILENOTIFY_ENDRENAME | SPFILENOTIFY_ENDCOPY => {
            (*CopyContext).CompletedOperations += 1;

            if (*CopyContext).TotalOperations >> 1 == (*CopyContext).CompletedOperations {
                DPRINT1!(
                    b"CHECKPOINT:HALF_COPIED\n\0".as_ptr() as *const i8,
                );
            }

            ProgressNextStep((*CopyContext).ProgressBar);
            SetupUpdateMemoryInfo(CopyContext, (FALSE) as i32);
        }

        _ => {}
    }

    return FILEOP_DOIT;
}

unsafe extern "C" fn FileCopyPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut CopyContext: COPYCONTEXT = core::mem::zeroed();
    let mut MemBarWidth: UINT = 0;

    MUIDisplayPage(FILE_COPY_PAGE);

    CopyContext.TotalOperations = 0;
    CopyContext.CompletedOperations = 0;

    CopyContext.ProgressBar = CreateProgressBar(
        13,
        26,
        xScreen - 13,
        yScreen - 20,
        10,
        24,
        (TRUE) as BOOLEAN,
        MUIGetString(STRING_SETUPCOPYINGFILES),
    );

    MemBarWidth = ((xScreen - 26) / 5) as u32;
    MemBarWidth -= MemBarWidth % 2;

    CopyContext.MemoryBars[0 as usize] = CreateProgressBar(
        13,
        40,
        (13 + MemBarWidth) as SHORT,
        43,
        13,
        44,
        (FALSE) as BOOLEAN,
        b"Kernel Pool\0".as_ptr() as *const i8,
    );

    CopyContext.MemoryBars[1 as usize] = CreateProgressBar(
        (xScreen / 2) - ((MemBarWidth / 2)) as i16,
        40,
        (xScreen / 2) + ((MemBarWidth / 2)) as i16,
        43,
        (xScreen / 2) - ((MemBarWidth / 2)) as i16,
        44,
        (FALSE) as BOOLEAN,
        b"Kernel Cache\0".as_ptr() as *const i8,
    );

    CopyContext.MemoryBars[2 as usize] = CreateProgressBar(
        xScreen - 13 - (MemBarWidth) as i16,
        40,
        xScreen - 13,
        43,
        xScreen - 13 - (MemBarWidth) as i16,
        44,
        (FALSE) as BOOLEAN,
        b"Free Memory\0".as_ptr() as *const i8,
    );

    DoFileCopy(
        &mut USetupData as *mut USETUP_DATA,
        Some(FileCopyCallback),
        (&mut CopyContext as *mut COPYCONTEXT).cast(),
    );

    DestroyProgressBar(CopyContext.ProgressBar);
    DestroyProgressBar(CopyContext.MemoryBars[0 as usize]);
    DestroyProgressBar(CopyContext.MemoryBars[1 as usize]);
    DestroyProgressBar(CopyContext.MemoryBars[2 as usize]);

    InstallSetupInfFile(&mut USetupData as *mut USETUP_DATA);

    return REGISTRY_PAGE;
}

extern "C" {
    /// C-variadic, file-`static` in usetup.c (a PREGISTRY_STATUS_ROUTINE): DEFINED below by
    /// `va_entry!` under a crate-private symbol name, declared here so it can be passed as
    /// the `...` function pointer UpdateRegistry takes.
    #[link_name = "usetup_static_RegistryStatus"]
    fn RegistryStatus(RegStatus: REGISTRY_STATUS, ...);
}

/// `static VOID __cdecl RegistryStatus(IN REGISTRY_STATUS RegStatus, ...)`, defined through
/// varargs-rust's naked thunk: `a.get(0)` is `RegStatus`, `a.va(1)` is the `va_list` of
/// `va_start(args, RegStatus)`.
mod RegistryStatus_export {
    use super::*;
    varargs_rust::va_entry!(usetup_static_RegistryStatus => |a| -> () {
        /* WARNING: Please keep this lookup table in sync with the resources! */
        static StringIDs: [UINT; 8] = [
            STRING_DONE,                  /* Success */
            STRING_REGHIVEUPDATE,         /* RegHiveUpdate */
            STRING_IMPORTFILE,            /* ImportRegHive */
            STRING_DISPLAYSETTINGSUPDATE, /* DisplaySettingsUpdate */
            STRING_LOCALESETTINGSUPDATE,  /* LocaleSettingsUpdate */
            STRING_ADDKBLAYOUTS,          /* KeybLayouts */
            STRING_KEYBOARDSETTINGSUPDATE, /* KeybSettingsUpdate */
            STRING_CODEPAGEINFOUPDATE,    /* CodePageInfoUpdate */
        ];
        let RegStatus = a.get(0) as REGISTRY_STATUS;

        // C compares the enum against a size_t: the usual arithmetic conversions.
        if (RegStatus as usize) < ARRAYSIZE!(StringIDs) as usize {
            CONSOLE_SetStatusTextV(MUIGetString(StringIDs[RegStatus as usize]), a.va(1) as va_list);
        } else {
            CONSOLE_SetStatusText(b"Unknown status %d\0".as_ptr() as *const c_char, RegStatus);
        }
    });
}

unsafe extern "C" fn RegistryPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut Error: ULONG;

    MUIDisplayPage(REGISTRY_PAGE);

    Error = UpdateRegistry(
        &mut USetupData,
        RepairUpdateFlag,
        PartitionList,
        (*(*InstallPartition).Volume).Info.DriveLetter,
        SelectedLanguageId,
        Some(RegistryStatus),
        &mut s_SubstSettings,
    );
    if Error != ERROR_SUCCESS {
        MUIDisplayError(Error, Ir, POPUP_WAIT_ENTER);
        return QUIT_PAGE;
    } else {
        CONSOLE_SetStatusText(MUIGetString(STRING_DONE));
        return BOOTLOADER_INSTALL_PAGE;
    }
}

unsafe extern "C" fn BootLoaderSelectPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut Line: USHORT = 12;

    CONSOLE_SetStatusText(MUIGetString(STRING_PLEASEWAIT));

    if RepairUpdateFlag != 0 {
        USetupData.BootLoaderLocation = 0;
        return INSTALL_DIRECTORY_PAGE;
    }

    if IsUnattendedSetup != 0 {
        if (USetupData.BootLoaderLocation == 0) ||
            (USetupData.BootLoaderLocation == 1)
        {
            return INSTALL_DIRECTORY_PAGE;
        }
    }

    if IsUnattendedSetup != 0 {
        if (USetupData.BootLoaderLocation == 2) ||
            (USetupData.BootLoaderLocation == 3)
        {
            return INSTALL_DIRECTORY_PAGE;
        }
    }

    MUIDisplayPage(BOOTLOADER_SELECT_PAGE);
    CONSOLE_InvertTextXY(8, (Line) as SHORT, 60, 1);

    while 1 != 0 {
        CONSOLE_ConInKey(Ir);

        if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
            (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_DOWN)
        {
            CONSOLE_NormalTextXY(8, (Line) as SHORT, 60, 1);

            Line = Line.wrapping_add(1);
            if Line < 12 {
                Line = 15;
            }

            if Line > 15 {
                Line = 12;
            }

            CONSOLE_InvertTextXY(8, (Line) as SHORT, 60, 1);
        } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
                 (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_UP)
        {
            CONSOLE_NormalTextXY(8, (Line) as SHORT, 60, 1);

            Line = Line.wrapping_sub(1);
            if Line < 12 {
                Line = 15;
            }

            if Line > 15 {
                Line = 12;
            }

            CONSOLE_InvertTextXY(8, (Line) as SHORT, 60, 1);
        } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
                 (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_HOME)
        {
            CONSOLE_NormalTextXY(8, (Line) as SHORT, 60, 1);

            Line = 12;

            CONSOLE_InvertTextXY(8, (Line) as SHORT, 60, 1);
        } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
                 (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_END)
        {
            CONSOLE_NormalTextXY(8, (Line) as SHORT, 60, 1);

            Line = 15;

            CONSOLE_InvertTextXY(8, (Line) as SHORT, 60, 1);
        } else if ((*Ir).Event.KeyEvent.uChar.AsciiChar == 0x00) &&
                 (((*Ir).Event.KeyEvent.wVirtualKeyCode) as u32 == VK_F3)
        {
            if ConfirmQuit(Ir) != 0 {
                return QUIT_PAGE;
            }
            break;
        } else if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x0D {
            if Line == 12 {
                USetupData.BootLoaderLocation = 2;
                break;
            } else if Line == 13 {
                USetupData.BootLoaderLocation = 3;
                break;
            } else if Line == 14 {
                USetupData.BootLoaderLocation = 1;
                break;
            } else if Line == 15 {
                USetupData.BootLoaderLocation = 0;
                break;
            }

            return BOOTLOADER_SELECT_PAGE;
        }
    }

    return INSTALL_DIRECTORY_PAGE;
}

unsafe extern "C" fn BootLoaderRemovableDiskPage(Ir: PINPUT_RECORD) -> i32 {
    let mut Status: NTSTATUS;

    let mut at = 0u8;
    loop {
        match at {
            0 => {
                CONSOLE_ClearScreen();
                CONSOLE_Flush();
                MUIDisplayPage(BOOTLOADER_REMOVABLE_DISK_PAGE);
                at = 1;
                continue;
            }
            1 => {
                CONSOLE_ConInKey(Ir);

                if ((*Ir).Event.KeyEvent.uChar.AsciiChar as i32 == 0x00) &&
                    (((*Ir).Event.KeyEvent.wVirtualKeyCode as i32) as u32 == VK_F3)
                {
                    if ConfirmQuit(Ir) != 0 {
                        return (FALSE) as i32;
                    }
                    at = 0;
                    continue;
                } else if (*Ir).Event.KeyEvent.uChar.AsciiChar as i32 == 0x0D {
                    static mut FloppyDrive: _UNICODE_STRING = _UNICODE_STRING {
                        Length: 0,
                        MaximumLength: 0,
                        Buffer: RUSTCONV_WSTR_123946225403.as_ptr() as *mut u16,
                    };
                    Status = InstallBootcodeToRemovable(
                        USetupData.ArchType,
                        &FloppyDrive,
                        &USetupData.SourceRootPath,
                        &USetupData.DestinationArcPath,
                    );
                    if Status == STATUS_SUCCESS {
                        return (TRUE) as i32;
                    }

                    if Status == STATUS_DEVICE_NOT_READY {
                        MUIDisplayError(ERROR_NO_FLOPPY, Ir, POPUP_WAIT_ENTER);
                    } else if ((NT_SUCCESS!(Status)) == 0) {
                        let mut Buffer: [i8; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

                        DPRINT1!(
                            b"InstallBootcodeToRemovable() failed: Status 0x%lx\n\0".as_ptr() as *const i8,
                            Status,
                        );
                        RtlStringCbPrintfA!(
                            Buffer.as_mut_ptr(),
                            core::mem::size_of::<[i8; (MAX_PATH) as _]>(),
                            b"Setup could not install the bootloader.\n(Status 0x%08lx).\nPress ENTER to continue anyway.\0".as_ptr() as *const i8,
                            Status,
                        );
                        PopupError(
                            Buffer.as_mut_ptr(),
                            MUIGetString(STRING_CONTINUE),
                            Ir,
                            POPUP_WAIT_ENTER,
                        );
                    }
                    at = 0;
                    continue;
                }
                at = 1;
                continue;
            }
            _ => break,
        }
    }
    return 0;
}

unsafe extern "C" fn BootLoaderHardDiskPage(Ir: PINPUT_RECORD) -> BOOLEAN {
    let mut Status: NTSTATUS;

    Status = InstallBootManagerAndBootEntries(
        USetupData.ArchType,
        &USetupData.SystemRootPath,
        &USetupData.SourceRootPath,
        &USetupData.DestinationArcPath,
        if USetupData.BootLoaderLocation == 2 {
            1
        } else {
            0
        },
    );
    if Status == STATUS_SUCCESS {
        return (TRUE) as BOOLEAN;
    }

    if (Status) as u32 == ERROR_WRITE_BOOT {
        MUIDisplayError(
            ERROR_WRITE_BOOT,
            Ir,
            POPUP_WAIT_ENTER,
            (*(*SystemPartition).Volume).Info.FileSystem,
        );
    } else if (Status) as u32 == ERROR_INSTALL_BOOTCODE {
        MUIDisplayError(
            ERROR_INSTALL_BOOTCODE,
            Ir,
            POPUP_WAIT_ENTER,
            RUSTCONV_WSTR_DD8FED8124A3.as_ptr(),
        );
    } else if Status == STATUS_NOT_SUPPORTED {
        PopupError(
            b"Setup does not currently support installing\n\0".as_ptr() as PCCH,
            MUIGetString(STRING_CONTINUE),
            Ir,
            POPUP_WAIT_ENTER,
        );
    } else if ((NT_SUCCESS!(Status)) == 0) {
        let mut Buffer: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

        DPRINT1!(
            b"InstallBootManagerAndBootEntries() failed: Status 0x%lx\n\0".as_ptr() as PCCH,
            Status,
        );
        RtlStringCbPrintfA!(
            Buffer.as_mut_ptr(),
            core::mem::size_of::<[CHAR; (MAX_PATH) as _]>(),
            b"Setup could not install the bootloader.\n\0".as_ptr() as PCCH,
            Status,
        );
        PopupError(
            Buffer.as_mut_ptr(),
            MUIGetString(STRING_CONTINUE),
            Ir,
            POPUP_WAIT_ENTER,
        );
    }
    return (FALSE) as BOOLEAN;
}

unsafe extern "C" fn BootLoaderInstallPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    let mut PathBuffer: [u16; RTL_NUMBER_OF_FIELD!(PARTENTRY, DeviceName) + 1] = [0; RTL_NUMBER_OF_FIELD!(PARTENTRY, DeviceName) + 1];

    RtlFreeUnicodeString(&mut USetupData.SystemRootPath);
    RtlStringCchPrintfW!(
        PathBuffer.as_mut_ptr(),
        _countof!(PathBuffer),
        RUSTCONV_WSTR_BCF3913447D5.as_ptr(),
        (*SystemPartition).DeviceName,
    );
    RtlCreateUnicodeString(&mut USetupData.SystemRootPath, PathBuffer.as_mut_ptr());
    DPRINT1!(b"SystemRootPath: %wZ\n\0".as_ptr() as *const i8, &USetupData.SystemRootPath);

    if USetupData.BootLoaderLocation != 0 {
        MUIDisplayPage(BOOTLOADER_INSTALL_PAGE);
    }

    match USetupData.BootLoaderLocation {
        1 => {
            if BootLoaderRemovableDiskPage(Ir) != 0 {
                return SUCCESS_PAGE;
            } else {
                return QUIT_PAGE;
            }
        }
        2 | 3 => {
            if BootLoaderHardDiskPage(Ir) != 0 {
                return SUCCESS_PAGE;
            } else {
                return QUIT_PAGE;
            }
        }
        0 | _ => {
            return SUCCESS_PAGE;
        }
    }
}

unsafe extern "system" fn ProgressTimeOutStringHandler(
    Bar: *mut PROGRESSBAR,
    AlwaysUpdate: BOOLEAN,
    Buffer: PSTR,
    cchBufferSize: SIZE_T,
) -> BOOLEAN {
    let OldProgress: ULONG = (*Bar).Progress;

    if (*Bar).StepCount == 0 {
        (*Bar).Progress = 0;
    } else {
        (*Bar).Progress = (*Bar).StepCount - (*Bar).CurrentStep;
    }

    if !(*Bar).ProgressFormatText.is_null()
        && (AlwaysUpdate != 0 || ((*Bar).Progress != OldProgress))
    {
        RtlStringCchPrintfA!(
            Buffer,
            cchBufferSize,
            (*Bar).ProgressFormatText,
            (*Bar).Progress / (max(1, (*Bar).Width)) as u32 + 1,
        );

        return (TRUE) as BOOLEAN;
    }

    return (FALSE) as BOOLEAN;
}

unsafe extern "C" fn ProgressCountdown(
    Ir: *mut INPUT_RECORD,
    TimeOut: i32,
) {
    let mut Status: i32;
    let mut StartTime: u32;
    let mut BarWidth: u32;
    let mut TimerDiv: u32;
    let mut TimeElapsed: i32;
    let mut TimerValue: i32;
    let mut OldTimerValue: i32;
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut ProgressBar: *mut PROGRESSBAR;
    let mut RefreshProgress: u8 = (TRUE) as u8;

    if TimeOut <= 0 {
        return;
    }

    ProgressBar = CreateProgressBarEx(
        13,
        26,
        xScreen - 13,
        yScreen - 20,
        10,
        24,
        (TRUE) as BOOLEAN,
        (FOREGROUND_RED | BACKGROUND_BLUE) as SHORT,
        0,
        core::ptr::null_mut(),
        MUIGetString(STRING_REBOOTPROGRESSBAR),
        Some(ProgressTimeOutStringHandler),
    );

    BarWidth = max(1, ((*ProgressBar).Width) as u32);
    TimerValue = TimeOut * BarWidth as i32;
    ProgressSetStepCount(ProgressBar, (TimerValue) as ULONG);

    StartTime = NtGetTickCount();
    CONSOLE_Flush();

    TimerDiv = 1000 / BarWidth;
    TimerDiv = max(1, TimerDiv);
    OldTimerValue = TimerValue;
    'exit: loop {
        TimeElapsed = NtGetTickCount() as i32 - StartTime as i32;
        if TimeElapsed >= TimerDiv as i32 {
            TimeElapsed /= TimerDiv as i32;
            StartTime += TimerDiv * TimeElapsed as u32;

            if TimeElapsed <= TimerValue {
                TimerValue -= TimeElapsed;
            } else {
                TimerValue = 0;
            }

            RefreshProgress = (TRUE) as u8;
        }

        if RefreshProgress != 0 {
            ProgressSetStep(ProgressBar, (OldTimerValue - TimerValue) as ULONG);
            RefreshProgress = (FALSE) as u8;
        }

        if TimerValue <= 0 {
            break 'exit;
        }

        TimeElapsed = NtGetTickCount() as i32 - StartTime as i32;
        if TimeElapsed < TimerDiv as i32 {
            Timeout.QuadPart = ((TimerDiv as i32 - TimeElapsed) * -10000) as _;
            Status = NtWaitForSingleObject(StdInput, (FALSE) as BOOLEAN, &mut Timeout);
        } else {
            Status = STATUS_TIMEOUT;
        }

        if Status == STATUS_TIMEOUT {
            continue;
        }
        if Status != STATUS_WAIT_0 {
            DPRINT1!(
                b"NtWaitForSingleObject() failed, Status 0x%08lx\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'exit;
        }

        while CONSOLE_ConInKeyPeek(Ir) != 0 {
            if (*Ir).Event.KeyEvent.uChar.AsciiChar == 0x0D {
                break 'exit;
            }
        }
    }

    DestroyProgressBar(ProgressBar);
}

unsafe extern "C" fn QuitPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    MUIDisplayPage(QUIT_PAGE);
    // Not in the C: tells the install smoke setup gave up rather than finished.
    DPRINT1!(b"[USETUP] setup quit\n\0".as_ptr() as *const i8);

    /* Destroy the NTOS installations list */
    if NtOsInstallsList != core::ptr::null_mut() {
        DestroyGenericList(NtOsInstallsList, (TRUE) as BOOLEAN);
        NtOsInstallsList = core::ptr::null_mut();
    }

    /* Destroy the partition list */
    if PartitionList != core::ptr::null_mut() {
        DestroyPartitionList(PartitionList);
        PartitionList = core::ptr::null_mut();
    }

    CONSOLE_SetStatusText(MUIGetString(STRING_REBOOTCOMPUTER2));

    if IsUnattendedSetup != 0 {
        return FLUSH_PAGE;
    }

    /* Wait for maximum 15 seconds or an ENTER key before quitting */
    ProgressCountdown(Ir, 15);
    return FLUSH_PAGE;
}

unsafe extern "C" fn SuccessPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    MUIDisplayPage(SUCCESS_PAGE);
    // Not in the C: stage 1's end marker for the install smoke.
    DPRINT1!(b"[USETUP] installation succeeded\n\0".as_ptr() as *const i8);

    if IsUnattendedSetup != 0 {
        return FLUSH_PAGE;
    }

    ProgressCountdown(Ir, 15);
    return FLUSH_PAGE;
}

unsafe extern "C" fn FlushPage(Ir: PINPUT_RECORD) -> PAGE_NUMBER {
    MUIDisplayPage(FLUSH_PAGE);
    return REBOOT_PAGE;
}
#[no_mangle]
pub unsafe extern "C" fn RunUSetup() -> i32 {
    let mut Status: i32;
    let mut Ir: INPUT_RECORD = core::mem::zeroed();
    let mut Page: PAGE_NUMBER;
    let mut Old: u8 = 0;

    InfSetHeap(ProcessHeap);

    Status = NtInitializeRegistry((CM_BOOT_FLAG_SETUP) as USHORT);
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtInitializeRegistry() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
    }

    Status = InitializeUserModePnpManager(&mut USetupData.SetupInf);
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"The user-mode PnP manager could not initialize (Status 0x%08lx), expect unavailable devices!\n\0"
                .as_ptr() as *const i8,
            Status,
        );
    }

    if ((CONSOLE_Init()) == 0) {
        PrintString(MUIGetString(STRING_CONSOLEFAIL1));
        PrintString(MUIGetString(STRING_CONSOLEFAIL2));
        PrintString(MUIGetString(STRING_CONSOLEFAIL3));

        return STATUS_APP_INIT_FAILURE;
    }

    CONSOLE_SetCursorType((TRUE) as BOOL, (FALSE) as BOOL);
    CONSOLE_ClearScreen();
    CONSOLE_Flush();

    Page = SetupStartPage(&mut Ir);

    while Page != REBOOT_PAGE && Page != RECOVERY_PAGE {
        // Not in the C. The install smoke (tests/qemu/test_install.py) has no screen
        // to read, so the page sequence on the debug port is how it says where an
        // unattended setup stopped.
        DPRINT1!(b"[USETUP] page %d\n\0".as_ptr() as *const i8, Page as i32);
        CONSOLE_ClearScreen();
        CONSOLE_Flush();

        match Page {
            LANGUAGE_PAGE => {
                Page = LanguagePage(&mut Ir);
            }
            WELCOME_PAGE => {
                Page = WelcomePage(&mut Ir);
            }
            LICENSE_PAGE => {
                Page = LicensePage(&mut Ir);
            }
            INSTALL_INTRO_PAGE => {
                Page = InstallIntroPage(&mut Ir);
            }
            DEVICE_SETTINGS_PAGE => {
                Page = DeviceSettingsPage(&mut Ir);
            }
            COMPUTER_SETTINGS_PAGE => {
                Page = ComputerSettingsPage(&mut Ir);
            }
            DISPLAY_SETTINGS_PAGE => {
                Page = DisplaySettingsPage(&mut Ir);
            }
            KEYBOARD_SETTINGS_PAGE => {
                Page = KeyboardSettingsPage(&mut Ir);
            }
            LAYOUT_SETTINGS_PAGE => {
                Page = LayoutSettingsPage(&mut Ir);
            }
            SELECT_PARTITION_PAGE => {
                Page = SelectPartitionPage(&mut Ir);
            }
            CREATE_PARTITION_PAGE => {
                Page = CreatePartitionPage(&mut Ir);
            }
            CONFIRM_DELETE_SYSTEM_PARTITION_PAGE => {
                Page = ConfirmDeleteSystemPartitionPage(&mut Ir);
            }
            DELETE_PARTITION_PAGE => {
                Page = DeletePartitionPage(&mut Ir);
            }
            START_PARTITION_OPERATIONS_PAGE => {
                Page = StartPartitionOperationsPage(&mut Ir);
            }
            BOOTLOADER_SELECT_PAGE => {
                Page = BootLoaderSelectPage(&mut Ir);
            }
            INSTALL_DIRECTORY_PAGE => {
                Page = InstallDirectoryPage(&mut Ir);
            }
            PREPARE_COPY_PAGE => {
                Page = PrepareCopyPage(&mut Ir);
            }
            FILE_COPY_PAGE => {
                Page = FileCopyPage(&mut Ir);
            }
            REGISTRY_PAGE => {
                Page = RegistryPage(&mut Ir);
            }
            BOOTLOADER_INSTALL_PAGE => {
                Page = BootLoaderInstallPage(&mut Ir);
            }
            REPAIR_INTRO_PAGE => {
                Page = RepairIntroPage(&mut Ir);
            }
            UPGRADE_REPAIR_PAGE => {
                Page = UpgradeRepairPage(&mut Ir);
            }
            SUCCESS_PAGE => {
                Page = SuccessPage(&mut Ir);
            }
            FLUSH_PAGE => {
                Page = FlushPage(&mut Ir);
            }
            QUIT_PAGE => {
                Page = QuitPage(&mut Ir);
            }
            SETUP_INIT_PAGE | SELECT_FILE_SYSTEM_PAGE | FORMAT_PARTITION_PAGE
            | REBOOT_PAGE | RECOVERY_PAGE => {}
            _ => {}
        }
    }

    TerminateUserModePnpManager();

    FinishSetup(&mut USetupData);

    if Page == RECOVERY_PAGE {
        RecoveryConsole();
    }

    FreeConsole();

    RtlAdjustPrivilege(
        SE_SHUTDOWN_PRIVILEGE,
        (TRUE) as BOOLEAN,
        (FALSE) as BOOLEAN,
        &mut Old,
    );
    NtShutdownSystem(ShutdownReboot);
    RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, Old, (FALSE) as BOOLEAN, &mut Old);

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NtProcessStartup(Peb: PPEB) {
    let mut Status: i32;
    let mut Time: LARGE_INTEGER = core::mem::zeroed();

    RtlNormalizeProcessParams((*Peb).ProcessParameters);

    ProcessHeap = (*Peb).ProcessHeap;

    NtQuerySystemTime(&mut Time);

    Status = RunUSetup();

    if NT_SUCCESS!(Status) != 0 {
        Time.QuadPart += 50000000;
        NtDelayExecution((FALSE) as BOOLEAN, &mut Time);
    } else {
        Status = NtRaiseHardError(
            STATUS_SYSTEM_PROCESS_TERMINATED,
            0,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
    }

    NtTerminateProcess(NtCurrentProcess(), Status);
}
