use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_1BC7E4D9627D: [u16; 9] = [0x0043u16, 0x004F, 0x004D, 0x0050, 0x0055, 0x0054, 0x0045, 0x0052, 0x0000];
pub static RUSTCONV_WSTR_239E714E0983: [u16; 5] = [0x0046u16, 0x0049, 0x004C, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_278084D1A822: [u16; 4] = [0x0055u16, 0x0053, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_2E04FE4C293B: [u16; 6] = [0x0047u16, 0x0052, 0x004F, 0x0055, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_31AAE1E3535F: [u16; 11] = [
    0x0053u16, 0x0054, 0x0041, 0x0054, 0x0049, 0x0053, 0x0054, 0x0049, 0x0043, 0x0053,
    0x0000,
];
pub static RUSTCONV_WSTR_392D47E88EB5: [u16; 11] = [
    0x004Cu16, 0x004F, 0x0043, 0x0041, 0x004C, 0x0047, 0x0052, 0x004F, 0x0055, 0x0050,
    0x0000,
];
pub static RUSTCONV_WSTR_4C2B4508C156: [u16; 9] = [0x0041u16, 0x0043, 0x0043, 0x004F, 0x0055, 0x004E, 0x0054, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_595FA2D85B6D: [u16; 7] = [0x0043u16, 0x004F, 0x004E, 0x0046, 0x0049, 0x0047, 0x0000];
pub static RUSTCONV_WSTR_718E488ABEFF: [u16; 6] = [0x0053u16, 0x0048, 0x0041, 0x0052, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_7EBE39473B12: [u16; 7] = [0x0053u16, 0x0045, 0x0052, 0x0056, 0x0045, 0x0052, 0x0000];
pub static RUSTCONV_WSTR_7FE3BF6A4C4F: [u16; 7] = [0x0053u16, 0x0059, 0x004E, 0x0054, 0x0041, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_85289DCA13CE: [u16; 5] = [0x0054u16, 0x0049, 0x004D, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_89046131630F: [u16; 6] = [0x0053u16, 0x0054, 0x0041, 0x0052, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_8A5B2ACCC16C: [u16; 8] = [0x0053u16, 0x0045, 0x0053, 0x0053, 0x0049, 0x004F, 0x004E, 0x0000];
pub static RUSTCONV_WSTR_94C738EF9FB7: [u16; 9] = [0x0043u16, 0x004F, 0x004E, 0x0054, 0x0049, 0x004E, 0x0055, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_BF1B16BA3061: [u16; 8] = [0x0048u16, 0x0045, 0x004C, 0x0050, 0x004D, 0x0053, 0x0047, 0x0000];
pub static RUSTCONV_WSTR_C112FCF344AA: [u16; 5] = [0x0055u16, 0x0053, 0x0045, 0x0052, 0x0000];
pub static RUSTCONV_WSTR_C7970F87CF07: [u16; 5] = [0x0053u16, 0x0054, 0x004F, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_EA087132735C: [u16; 6] = [0x0050u16, 0x0041, 0x0055, 0x0053, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_EF5A440DAF4B: [u16; 5] = [0x0056u16, 0x0049, 0x0045, 0x0057, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn cmdHelp(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    PrintMessageString(4381);
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    if argc != 3 {
        PrintNetMessage(MSG_HELP_SYNTAX);
        PrintNetMessage(MSG_HELP_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_4C2B4508C156.as_ptr()) == 0 {
        PrintNetMessage(MSG_ACCOUNTS_SYNTAX);
        PrintNetMessage(MSG_ACCOUNTS_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_1BC7E4D9627D.as_ptr()) == 0 {
        PrintNetMessage(MSG_COMPUTER_SYNTAX);
        PrintNetMessage(MSG_COMPUTER_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_595FA2D85B6D.as_ptr()) == 0 {
        if (argc > 3) && (_wcsicmp(*argv.offset(3 as isize), RUSTCONV_WSTR_7EBE39473B12.as_ptr()) == 0) {
            PrintNetMessage(MSG_CONFIG_SERVER_SYNTAX);
            PrintNetMessage(MSG_CONFIG_SERVER_HELP);
            return 0;
        } else {
            PrintNetMessage(MSG_CONFIG_SYNTAX);
            PrintNetMessage(MSG_CONFIG_HELP);
            return 0;
        }
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_94C738EF9FB7.as_ptr()) == 0 {
        PrintNetMessage(MSG_CONTINUE_SYNTAX);
        PrintNetMessage(MSG_CONTINUE_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_239E714E0983.as_ptr()) == 0 {
        PrintNetMessage(MSG_FILE_SYNTAX);
        PrintNetMessage(MSG_FILE_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_2E04FE4C293B.as_ptr()) == 0 {
        PrintNetMessage(MSG_GROUP_SYNTAX);
        PrintNetMessage(MSG_GROUP_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_BF1B16BA3061.as_ptr()) == 0 {
        PrintNetMessage(MSG_HELPMSG_SYNTAX);
        PrintNetMessage(MSG_HELPMSG_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_392D47E88EB5.as_ptr()) == 0 {
        PrintNetMessage(MSG_LOCALGROUP_SYNTAX);
        PrintNetMessage(MSG_LOCALGROUP_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_EA087132735C.as_ptr()) == 0 {
        PrintNetMessage(MSG_PAUSE_SYNTAX);
        PrintNetMessage(MSG_PAUSE_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_8A5B2ACCC16C.as_ptr()) == 0 {
        PrintNetMessage(MSG_SESSION_SYNTAX);
        PrintNetMessage(MSG_SESSION_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_718E488ABEFF.as_ptr()) == 0 {
        PrintNetMessage(MSG_SHARE_SYNTAX);
        PrintNetMessage(MSG_SHARE_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_89046131630F.as_ptr()) == 0 {
        PrintNetMessage(MSG_START_SYNTAX);
        PrintNetMessage(MSG_START_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_31AAE1E3535F.as_ptr()) == 0 {
        PrintNetMessage(MSG_STATISTICS_SYNTAX);
        PrintNetMessage(MSG_STATISTICS_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_C7970F87CF07.as_ptr()) == 0 {
        PrintNetMessage(MSG_STOP_SYNTAX);
        PrintNetMessage(MSG_STOP_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_85289DCA13CE.as_ptr()) == 0 {
        PrintNetMessage(MSG_TIME_SYNTAX);
        PrintNetMessage(MSG_TIME_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_278084D1A822.as_ptr()) == 0 {
        PrintNetMessage(MSG_USE_SYNTAX);
        PrintNetMessage(MSG_USE_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_C112FCF344AA.as_ptr()) == 0 {
        PrintNetMessage(MSG_USER_SYNTAX);
        PrintNetMessage(MSG_USER_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_EF5A440DAF4B.as_ptr()) == 0 {
        PrintNetMessage(MSG_VIEW_SYNTAX);
        PrintNetMessage(MSG_VIEW_HELP);
        return 0;
    }

    if _wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_7FE3BF6A4C4F.as_ptr()) == 0 {
        PrintNetMessage(MSG_SYNTAX_HELP);
        return 0;
    }

    PrintNetMessage(MSG_HELP_SYNTAX);
    PrintNetMessage(MSG_HELP_HELP);

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn cmdSyntax(argc: INT, argv: *mut *mut WCHAR) -> INT {
    PrintMessageString(4381);
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    PrintNetMessage(MSG_SYNTAX_HELP);
    return 0;
}
