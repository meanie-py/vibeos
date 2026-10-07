use crate::diskpart::convert::ConvertGPT;
use crate::diskpart::convert::ConvertMBR;
use crate::diskpart::create::CreateEfiPartition;
use crate::diskpart::create::CreateExtendedPartition;
use crate::diskpart::create::CreateLogicalPartition;
use crate::diskpart::create::CreateMsrPartition;
use crate::diskpart::create::CreatePrimaryPartition;
use crate::diskpart::delete::DeletePartition;
use crate::diskpart::detail::DetailDisk;
use crate::diskpart::detail::DetailPartition;
use crate::diskpart::detail::DetailVolume;
use crate::diskpart::dump::DumpDisk;
use crate::diskpart::dump::DumpPartition;
use crate::diskpart::help::HelpCommand;
use crate::diskpart::help::HelpCommandList;
use crate::diskpart::active::active_main;
use crate::diskpart::assign::assign_main;
use crate::diskpart::automount::automount_main;
use crate::diskpart::clean::clean_main;
use crate::diskpart::filesystems::filesystems_main;
use crate::diskpart::format::format_main;
use crate::diskpart::gpt::gpt_main;
use crate::diskpart::help::help_main;
use crate::diskpart::inactive::inactive_main;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_5C9738DE6AF0: [u16; 5] = [0x0065u16, 0x0078, 0x0069, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_DEBCEBA069C0: [u16; 4] = [0x0072u16, 0x0065, 0x006D, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_1C74719938D0: [u16; 5] = [0x004Cu16, 0x0049, 0x0053, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_1D6778824391: [u16; 7] = [0x0041u16, 0x0043, 0x0054, 0x0049, 0x0056, 0x0045, 0x0000];
pub static RUSTCONV_C2R_WSTR_259554D1F6B4: [u16; 5] = [0x0044u16, 0x0055, 0x004D, 0x0050, 0x0000];
pub static RUSTCONV_C2R_WSTR_25A3C738CEA1: [u16; 7] = [0x0052u16, 0x0045, 0x004D, 0x004F, 0x0056, 0x0045, 0x0000];
pub static RUSTCONV_C2R_WSTR_38CD0149DAD0: [u16; 7] = [0x0044u16, 0x0045, 0x004C, 0x0045, 0x0054, 0x0045, 0x0000];
pub static RUSTCONV_C2R_WSTR_3EA478C1E7E2: [u16; 6] = [0x0053u16, 0x0045, 0x0054, 0x0049, 0x0044, 0x0000];
pub static RUSTCONV_C2R_WSTR_404D0EB1C566: [u16; 8] = [0x0050u16, 0x0052, 0x0049, 0x004D, 0x0041, 0x0052, 0x0059, 0x0000];
pub static RUSTCONV_C2R_WSTR_436B282D5224: [u16; 8] = [0x0043u16, 0x004F, 0x004E, 0x0056, 0x0045, 0x0052, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_49E591691CF7: [u16; 5] = [0x0045u16, 0x0058, 0x0049, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_4BD88B8826CF: [u16; 7] = [0x0046u16, 0x004F, 0x0052, 0x004D, 0x0041, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_4C1F1F391AB5: [u16; 10] = [0x0041u16, 0x0055, 0x0054, 0x004F, 0x004D, 0x004F, 0x0055, 0x004E, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_540F335EA2F9: [u16; 9] = [0x0055u16, 0x004E, 0x0049, 0x0051, 0x0055, 0x0045, 0x0049, 0x0044, 0x0000];
pub static RUSTCONV_C2R_WSTR_5627986FBAED: [u16; 7] = [0x0041u16, 0x0053, 0x0053, 0x0049, 0x0047, 0x004E, 0x0000];
pub static RUSTCONV_C2R_WSTR_5ADC56DCA853: [u16; 9] = [0x0045u16, 0x0058, 0x0054, 0x0045, 0x004E, 0x0044, 0x0045, 0x0044, 0x0000];
pub static RUSTCONV_C2R_WSTR_5D58467A1130: [u16; 5] = [0x0044u16, 0x0049, 0x0053, 0x004B, 0x0000];
pub static RUSTCONV_C2R_WSTR_6016BA0C86F8: [u16; 9] = [0x0049u16, 0x004E, 0x0041, 0x0043, 0x0054, 0x0049, 0x0056, 0x0045, 0x0000];
pub static RUSTCONV_C2R_WSTR_6522E61166AF: [u16; 7] = [0x0053u16, 0x0045, 0x004C, 0x0045, 0x0043, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_72F435F677A8: [u16; 7] = [0x0044u16, 0x0045, 0x0054, 0x0041, 0x0049, 0x004C, 0x0000];
pub static RUSTCONV_C2R_WSTR_764FF54BB22D: [u16; 4] = [0x0053u16, 0x0045, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_7AC6B9716F5E: [u16; 10] = [0x0050u16, 0x0041, 0x0052, 0x0054, 0x0049, 0x0054, 0x0049, 0x004F, 0x004E, 0x0000];
pub static RUSTCONV_C2R_WSTR_9B997430E721: [u16; 7] = [0x0043u16, 0x0052, 0x0045, 0x0041, 0x0054, 0x0045, 0x0000];
pub static RUSTCONV_C2R_WSTR_9C878F9610E9: [u16; 6] = [0x0043u16, 0x004C, 0x0045, 0x0041, 0x004E, 0x0000];
pub static RUSTCONV_C2R_WSTR_B745EAE41AE4: [u16; 7] = [0x0056u16, 0x004F, 0x004C, 0x0055, 0x004D, 0x0045, 0x0000];
pub static RUSTCONV_C2R_WSTR_C3AAC603AFA3: [u16; 8] = [0x004Cu16, 0x004F, 0x0047, 0x0049, 0x0043, 0x0041, 0x004C, 0x0000];
pub static RUSTCONV_C2R_WSTR_CFDFF1FB5CCC: [u16; 7] = [0x0052u16, 0x0045, 0x0053, 0x0043, 0x0041, 0x004E, 0x0000];
pub static RUSTCONV_C2R_WSTR_D59F02F16056: [u16; 4] = [0x0045u16, 0x0046, 0x0049, 0x0000];
pub static RUSTCONV_C2R_WSTR_DAB59FC76188: [u16; 12] = [
    0x0046u16, 0x0049, 0x004C, 0x0045, 0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D,
    0x0053, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_DD8FED8124A3: [u16; 4] = [0x004Du16, 0x0042, 0x0052, 0x0000];
pub static RUSTCONV_C2R_WSTR_E066D47523FF: [u16; 4] = [0x0047u16, 0x0050, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_E076BFB34195: [u16; 4] = [0x004Du16, 0x0053, 0x0052, 0x0000];
pub static RUSTCONV_C2R_WSTR_E20712666EC4: [u16; 5] = [0x0048u16, 0x0045, 0x004C, 0x0050, 0x0000];
pub static RUSTCONV_C2R_WSTR_E305D3022030: [u16; 4] = [0x0052u16, 0x0045, 0x004D, 0x0000];

#[no_mangle]
pub static mut cmds: [COMMAND; 45] = unsafe {
    [
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_1D6778824391.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(active_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_ACTIVE) as i32,
            help_detail: MSG_COMMAND_ACTIVE,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_5627986FBAED.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(assign_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_ASSIGN) as i32,
            help_detail: MSG_COMMAND_ASSIGN,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_4C1F1F391AB5.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(automount_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_AUTOMOUNT) as i32,
            help_detail: MSG_COMMAND_AUTOMOUNT,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_9C878F9610E9.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(clean_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_CLEAN) as i32,
            help_detail: MSG_COMMAND_CLEAN,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_436B282D5224.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: (IDS_HELP_CONVERT) as i32,
            help_detail: MSG_NONE as DWORD,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_436B282D5224.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_E066D47523FF.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(ConvertGPT as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_CONVERT_GPT) as i32,
            help_detail: MSG_COMMAND_CONVERT_GPT,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_436B282D5224.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_DD8FED8124A3.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(ConvertMBR as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_CONVERT_MBR) as i32,
            help_detail: MSG_COMMAND_CONVERT_MBR,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_9B997430E721.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: (IDS_HELP_CREATE) as i32,
            help_detail: MSG_NONE as DWORD,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_9B997430E721.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_7AC6B9716F5E.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: (IDS_HELP_CREATE_PARTITION) as i32,
            help_detail: MSG_NONE as DWORD,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_9B997430E721.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_7AC6B9716F5E.as_ptr() as PWSTR,
            cmd3: RUSTCONV_C2R_WSTR_D59F02F16056.as_ptr() as PWSTR,
            func: Some(CreateEfiPartition as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_CREATE_PARTITION_EFI) as i32,
            help_detail: MSG_COMMAND_CREATE_PARTITION_EFI,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_9B997430E721.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_7AC6B9716F5E.as_ptr() as PWSTR,
            cmd3: RUSTCONV_C2R_WSTR_5ADC56DCA853.as_ptr() as PWSTR,
            func: Some(
                CreateExtendedPartition as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE,
            ),
            help: (IDS_HELP_CREATE_PARTITION_EXTENDED) as i32,
            help_detail: MSG_COMMAND_CREATE_PARTITION_EXTENDED,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_9B997430E721.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_7AC6B9716F5E.as_ptr() as PWSTR,
            cmd3: RUSTCONV_C2R_WSTR_C3AAC603AFA3.as_ptr() as PWSTR,
            func: Some(
                CreateLogicalPartition as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE,
            ),
            help: (IDS_HELP_CREATE_PARTITION_LOGICAL) as i32,
            help_detail: MSG_COMMAND_CREATE_PARTITION_LOGICAL,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_9B997430E721.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_7AC6B9716F5E.as_ptr() as PWSTR,
            cmd3: RUSTCONV_C2R_WSTR_E076BFB34195.as_ptr() as PWSTR,
            func: Some(CreateMsrPartition as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_CREATE_PARTITION_MSR) as i32,
            help_detail: MSG_COMMAND_CREATE_PARTITION_MSR,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_9B997430E721.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_7AC6B9716F5E.as_ptr() as PWSTR,
            cmd3: RUSTCONV_C2R_WSTR_404D0EB1C566.as_ptr() as PWSTR,
            func: Some(
                CreatePrimaryPartition as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE,
            ),
            help: (IDS_HELP_CREATE_PARTITION_PRIMARY) as i32,
            help_detail: MSG_COMMAND_CREATE_PARTITION_PRIMARY,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_38CD0149DAD0.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: (IDS_HELP_DELETE) as i32,
            help_detail: MSG_NONE as DWORD,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_38CD0149DAD0.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_7AC6B9716F5E.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(DeletePartition as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_DELETE_PARTITION) as i32,
            help_detail: MSG_COMMAND_DELETE_PARTITION,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_72F435F677A8.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: (IDS_HELP_DETAIL) as i32,
            help_detail: MSG_NONE as DWORD,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_72F435F677A8.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_5D58467A1130.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(DetailDisk as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_DETAIL_DISK) as i32,
            help_detail: MSG_COMMAND_DETAIL_DISK,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_72F435F677A8.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_7AC6B9716F5E.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(DetailPartition as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_DETAIL_PARTITION) as i32,
            help_detail: MSG_COMMAND_DETAIL_PARTITION,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_72F435F677A8.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_B745EAE41AE4.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(DetailVolume as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_DETAIL_VOLUME) as i32,
            help_detail: MSG_COMMAND_DETAIL_VOLUME,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_259554D1F6B4.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: IDS_NONE,
            help_detail: MSG_NONE as DWORD,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_259554D1F6B4.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_5D58467A1130.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(DumpDisk as unsafe extern "C" fn(INT, *mut LPWSTR) -> EXIT_CODE),
            help: IDS_NONE,
            help_detail: MSG_NONE as DWORD,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_259554D1F6B4.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_7AC6B9716F5E.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(DumpPartition as unsafe extern "C" fn(INT, *mut LPWSTR) -> EXIT_CODE),
            help: IDS_NONE,
            help_detail: MSG_NONE as DWORD,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_49E591691CF7.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: (IDS_HELP_EXIT) as i32,
            help_detail: MSG_COMMAND_EXIT,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_DAB59FC76188.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(filesystems_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_FILESYSTEMS) as i32,
            help_detail: MSG_COMMAND_FILESYSTEMS,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_4BD88B8826CF.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(format_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_FORMAT) as i32,
            help_detail: MSG_COMMAND_FORMAT,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_E066D47523FF.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(gpt_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_GPT) as i32,
            help_detail: MSG_COMMAND_GPT,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_E20712666EC4.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(help_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_HELP) as i32,
            help_detail: MSG_COMMAND_HELP,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_6016BA0C86F8.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(inactive_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_INACTIVE) as i32,
            help_detail: MSG_COMMAND_INACTIVE,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_1C74719938D0.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: (IDS_HELP_LIST) as i32,
            help_detail: MSG_NONE as DWORD,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_1C74719938D0.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_5D58467A1130.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(ListDisk as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_LIST_DISK) as i32,
            help_detail: MSG_COMMAND_LIST_DISK,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_1C74719938D0.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_7AC6B9716F5E.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(ListPartition as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_LIST_PARTITION) as i32,
            help_detail: MSG_COMMAND_LIST_PARTITION,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_1C74719938D0.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_B745EAE41AE4.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(ListVolume as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_LIST_VOLUME) as i32,
            help_detail: MSG_COMMAND_LIST_VOLUME,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_E305D3022030.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: (IDS_HELP_REM) as i32,
            help_detail: MSG_COMMAND_REM,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_25A3C738CEA1.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(remove_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_REMOVE) as i32,
            help_detail: MSG_COMMAND_REMOVE,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_CFDFF1FB5CCC.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(rescan_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_RESCAN) as i32,
            help_detail: MSG_COMMAND_RESCAN,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_6522E61166AF.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: (IDS_HELP_SELECT) as i32,
            help_detail: MSG_NONE as DWORD,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_6522E61166AF.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_5D58467A1130.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(SelectDisk as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_SELECT_DISK) as i32,
            help_detail: MSG_COMMAND_SELECT_DISK,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_6522E61166AF.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_7AC6B9716F5E.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(SelectPartition as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_SELECT_PARTITION) as i32,
            help_detail: MSG_COMMAND_SELECT_PARTITION,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_6522E61166AF.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_B745EAE41AE4.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(SelectVolume as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_SELECT_VOLUME) as i32,
            help_detail: MSG_COMMAND_SELECT_VOLUME,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_764FF54BB22D.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(setid_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_SETID) as i32,
            help_detail: MSG_COMMAND_SETID,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_3EA478C1E7E2.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(setid_main as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_SETID) as i32,
            help_detail: MSG_COMMAND_SETID,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_540F335EA2F9.as_ptr() as PWSTR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: (IDS_HELP_UNIQUEID) as i32,
            help_detail: MSG_NONE as DWORD,
        },
        _COMMAND {
            cmd1: RUSTCONV_C2R_WSTR_540F335EA2F9.as_ptr() as PWSTR,
            cmd2: RUSTCONV_C2R_WSTR_5D58467A1130.as_ptr() as PWSTR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: Some(UniqueIdDisk as unsafe extern "C" fn(INT, *mut PWSTR) -> EXIT_CODE),
            help: (IDS_HELP_UNIQUEID_DISK) as i32,
            help_detail: MSG_COMMAND_UNIQUEID_DISK,
        },
        _COMMAND {
            cmd1: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd2: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            cmd3: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            func: None,
            help: IDS_NONE,
            help_detail: MSG_NONE as DWORD,
        },
    ]
};
#[no_mangle]
pub unsafe extern "C" fn InterpretCmd(
    argc: i32,
    argv: *mut LPWSTR,
) -> i32 {
    let mut cmdptr: *mut COMMAND;
    let mut cmdptr1: *mut COMMAND = core::ptr::null_mut();
    let mut cmdptr2: *mut COMMAND = core::ptr::null_mut();
    let mut cmdptr3: *mut COMMAND = core::ptr::null_mut();

    if argc < 1 {
        return (EXIT_SUCCESS) as i32;
    }

    if _wcsicmp(*argv.offset(0 as isize), RUSTCONV_WSTR_5C9738DE6AF0.as_ptr()) == 0 {
        return (EXIT_EXIT) as i32;
    }

    if _wcsicmp(*argv.offset(0 as isize), RUSTCONV_WSTR_DEBCEBA069C0.as_ptr()) == 0 {
        return (EXIT_SUCCESS) as i32;
    }

    cmdptr = cmds.as_mut_ptr();
    while !(*cmdptr).cmd1.is_null() {
        if cmdptr1.is_null()
            && !(*cmdptr).cmd1.is_null()
            && _wcsicmp(*argv.offset(0 as isize), (*cmdptr).cmd1) == 0
        {
            cmdptr1 = cmdptr;
        }

        if cmdptr2.is_null()
            && argc >= 2
            && !(*cmdptr).cmd1.is_null()
            && _wcsicmp(*argv.offset(0 as isize), (*cmdptr).cmd1) == 0
            && !(*cmdptr).cmd2.is_null()
            && _wcsicmp(*argv.offset(1 as isize), (*cmdptr).cmd2) == 0
        {
            cmdptr2 = cmdptr;
        }

        if cmdptr3.is_null()
            && argc >= 3
            && !(*cmdptr).cmd1.is_null()
            && _wcsicmp(*argv.offset(0 as isize), (*cmdptr).cmd1) == 0
            && !(*cmdptr).cmd2.is_null()
            && _wcsicmp(*argv.offset(1 as isize), (*cmdptr).cmd2) == 0
            && !(*cmdptr).cmd3.is_null()
            && _wcsicmp(*argv.offset(2 as isize), (*cmdptr).cmd3) == 0
        {
            cmdptr3 = cmdptr;
        }

        cmdptr = cmdptr.offset(1);
    }

    if !cmdptr3.is_null() {
        if (*cmdptr3).func.is_none() {
            return (HelpCommand(cmdptr3)) as i32;
        } else {
            return ((((*cmdptr3).func).unwrap_unchecked())(argc, argv)) as i32;
        }
    } else if !cmdptr2.is_null() {
        if (*cmdptr2).func.is_none() {
            return (HelpCommand(cmdptr2)) as i32;
        } else {
            return ((((*cmdptr2).func).unwrap_unchecked())(argc, argv)) as i32;
        }
    } else if !cmdptr1.is_null() {
        if (*cmdptr1).func.is_none() {
            return (HelpCommand(cmdptr1)) as i32;
        } else {
            return ((((*cmdptr1).func).unwrap_unchecked())(argc, argv)) as i32;
        }
    }

    HelpCommandList();

    return (EXIT_SUCCESS) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn InterpretScript(input_line: PWSTR) -> EXIT_CODE {
    let mut args_vector: [PWSTR; (MAX_ARGS_COUNT) as usize] = [core::ptr::null_mut(); (MAX_ARGS_COUNT) as usize];
    let mut args_count: INT = 0;
    let mut bWhiteSpace: BOOL = TRUE;
    let mut bQuote: BOOL = FALSE;
    let mut ptr: PWSTR;

    memset(
        args_vector.as_mut_ptr() as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[PWSTR; (MAX_ARGS_COUNT) as usize]>(),
    );

    bQuote = FALSE;
    ptr = input_line;
    while *ptr != 0 {
        if *ptr == 0x0022u16 {
            bQuote = (bQuote == 0) as BOOL; // C `!bQuote`: logical not
        }

        if (((iswspace(*ptr)) != 0) && (bQuote == FALSE)) || *ptr == 0x000Au16 {
            *ptr = 0;
            bWhiteSpace = TRUE;
        } else {
            if (bWhiteSpace != FALSE) && (bQuote == FALSE) && (((args_count) as u32) < MAX_ARGS_COUNT) {
                args_vector[args_count as usize] = ptr;
                args_count += 1;
            }

            bWhiteSpace = FALSE;
        }

        ptr = ptr.offset(1);
    }

    return (InterpretCmd(args_count, (args_vector.as_mut_ptr() as LPWSTR).cast())) as EXIT_CODE;
}
#[no_mangle]
pub unsafe extern "C" fn InterpretMain() {
    let mut input_line: [u16; (MAX_STRING_SIZE) as usize] = [0; (MAX_STRING_SIZE) as usize];
    let mut args_vector: [LPWSTR; (MAX_ARGS_COUNT) as usize] = [core::ptr::null_mut(); (MAX_ARGS_COUNT) as usize];
    let mut args_count: i32 = 0;
    let mut bWhiteSpace: i32 = TRUE;
    let mut bQuote: i32 = FALSE;
    let mut ExitCode: i32 = (EXIT_SUCCESS) as i32;
    let mut ptr: LPWSTR;

    while (ExitCode) as u32 != EXIT_EXIT {
        args_count = 0;
        memset(
            args_vector.as_mut_ptr() as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<[LPWSTR; (MAX_ARGS_COUNT) as usize]>(),
        );

        ConResPuts(StdOut, (IDS_APP_PROMPT) as u32);

        fgetws(input_line.as_mut_ptr(), (MAX_STRING_SIZE) as i32, stdin());

        bQuote = FALSE;
        ptr = input_line.as_mut_ptr();
        while *ptr != 0 {
            if *ptr == 0x0022u16 {
                bQuote = (bQuote == 0) as BOOL; // C `!bQuote`: logical not
            }

            if (((iswspace(*ptr)) != 0) && (bQuote == FALSE)) || *ptr == 0x000Au16 {
                *ptr = 0;
                bWhiteSpace = TRUE;
            } else {
                if (bWhiteSpace != FALSE)
                    && (bQuote == FALSE)
                    && (((args_count) as u32) < MAX_ARGS_COUNT)
                {
                    args_vector[args_count as usize] = ptr;
                    args_count += 1;
                }
                bWhiteSpace = FALSE;
            }
            ptr = ptr.add(1);
        }

        ExitCode = InterpretCmd(args_count, args_vector.as_mut_ptr());
    }
}
