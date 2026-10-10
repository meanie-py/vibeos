use crate::cmd::alias::CommandAlias;
use crate::cmd::assoc::CommandAssoc;
use crate::cmd::choice::CommandChoice;
use crate::cmd::beep::cmd_beep;
use crate::cmd::call::cmd_call;
use crate::cmd::cls::cmd_cls;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_032B79795A8E: [u16; 5] = [0x0070u16, 0x0061, 0x0074, 0x0068, 0x0000];
pub static RUSTCONV_C2R_WSTR_08F79C40983C: [u16; 7] = [0x0064u16, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_091778E8300A: [u16; 6] = [0x0065u16, 0x0063, 0x0068, 0x006F, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_0E24EFDCDDBF: [u16; 5] = [0x0065u16, 0x0063, 0x0068, 0x006F, 0x0000];
pub static RUSTCONV_C2R_WSTR_14ECF914DD1A: [u16; 3] = [0x0072u16, 0x0064, 0x0000];
pub static RUSTCONV_C2R_WSTR_1B9E0FE7B740: [u16; 2] = [0x003Fu16, 0x0000];
pub static RUSTCONV_C2R_WSTR_1C9DC775D96D: [u16; 5] = [0x0063u16, 0x0074, 0x0074, 0x0079, 0x0000];
pub static RUSTCONV_C2R_WSTR_1E55342C191E: [u16; 8] = [0x0065u16, 0x0063, 0x0068, 0x006F, 0x0065, 0x0072, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_1F509DCAEB1D: [u16; 5] = [0x0067u16, 0x006F, 0x0074, 0x006F, 0x0000];
pub static RUSTCONV_C2R_WSTR_1F9BE24D4991: [u16; 4] = [0x0064u16, 0x0069, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_2A522F376CF2: [u16; 5] = [0x0062u16, 0x0065, 0x0065, 0x0070, 0x0000];
pub static RUSTCONV_C2R_WSTR_2BFE0A590FD6: [u16; 5] = [0x0063u16, 0x0061, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_C2R_WSTR_2F458E01A72D: [u16; 4] = [0x0066u16, 0x006F, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_36701C1B5A9F: [u16; 7] = [0x0073u16, 0x0063, 0x0072, 0x0065, 0x0065, 0x006E, 0x0000];
pub static RUSTCONV_C2R_WSTR_377C04EA0393: [u16; 5] = [0x0063u16, 0x006F, 0x0070, 0x0079, 0x0000];
pub static RUSTCONV_C2R_WSTR_3B8BF4000A7A: [u16; 5] = [0x0074u16, 0x0079, 0x0070, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_3F89D9B038EA: [u16; 7] = [0x0070u16, 0x0072, 0x006F, 0x006D, 0x0070, 0x0074, 0x0000];
pub static RUSTCONV_C2R_WSTR_40D6F3569433: [u16; 4] = [0x0064u16, 0x0065, 0x006C, 0x0000];
pub static RUSTCONV_C2R_WSTR_4270EF7B1AA0: [u16; 9] = [0x0073u16, 0x0065, 0x0074, 0x006C, 0x006F, 0x0063, 0x0061, 0x006C, 0x0000];
pub static RUSTCONV_C2R_WSTR_42A469BB0711: [u16; 7] = [0x0072u16, 0x0065, 0x006E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_496BFBF67486: [u16; 9] = [0x0065u16, 0x0063, 0x0068, 0x006F, 0x0073, 0x0065, 0x0072, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_4A44DE231043: [u16; 8] = [0x0068u16, 0x0069, 0x0073, 0x0074, 0x006F, 0x0072, 0x0079, 0x0000];
pub static RUSTCONV_C2R_WSTR_4CDE8A491581: [u16; 6] = [0x0065u16, 0x0072, 0x0061, 0x0073, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_4E2171725127: [u16; 9] = [0x0065u16, 0x006E, 0x0064, 0x006C, 0x006F, 0x0063, 0x0061, 0x006C, 0x0000];
pub static RUSTCONV_C2R_WSTR_4F9E8005A358: [u16; 6] = [0x0070u16, 0x0061, 0x0075, 0x0073, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_5B3605B65962: [u16; 7] = [0x006Du16, 0x0065, 0x006D, 0x006F, 0x0072, 0x0079, 0x0000];
pub static RUSTCONV_C2R_WSTR_5B9F893DE302: [u16; 6] = [0x0063u16, 0x006F, 0x006C, 0x006F, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_5C4147782415: [u16; 3] = [0x006Du16, 0x0064, 0x0000];
pub static RUSTCONV_C2R_WSTR_5C9738DE6AF0: [u16; 5] = [0x0065u16, 0x0078, 0x0069, 0x0074, 0x0000];
pub static RUSTCONV_C2R_WSTR_6A216AD47A67: [u16; 4] = [0x0072u16, 0x0065, 0x006E, 0x0000];
pub static RUSTCONV_C2R_WSTR_6A8D52266886: [u16; 7] = [0x0063u16, 0x0068, 0x006F, 0x0069, 0x0063, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_73BCE7399733: [u16; 6] = [0x0073u16, 0x0068, 0x0069, 0x0066, 0x0074, 0x0000];
pub static RUSTCONV_C2R_WSTR_8A0F6CB5912D: [u16; 5] = [0x006Du16, 0x006F, 0x0076, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_923BEF676D76: [u16; 7] = [0x006Du16, 0x006B, 0x006C, 0x0069, 0x006E, 0x006B, 0x0000];
pub static RUSTCONV_C2R_WSTR_96F19EF35C88: [u16; 5] = [0x0066u16, 0x0072, 0x0065, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_9BE6D558E725: [u16; 5] = [0x0070u16, 0x006F, 0x0070, 0x0064, 0x0000];
pub static RUSTCONV_C2R_WSTR_9E442C20507D: [u16; 5] = [0x0074u16, 0x0069, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_A0E70D05E916: [u16; 6] = [0x0061u16, 0x006C, 0x0069, 0x0061, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_A698681B378D: [u16; 5] = [0x0064u16, 0x0061, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_A7CFC14CDE1B: [u16; 4] = [0x0063u16, 0x006C, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_B33584180CB3: [u16; 3] = [0x0069u16, 0x0066, 0x0000];
pub static RUSTCONV_C2R_WSTR_BACDE87AB274: [u16; 4] = [0x0076u16, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_BEB8E1991CF9: [u16; 6] = [0x0073u16, 0x0074, 0x0061, 0x0072, 0x0074, 0x0000];
pub static RUSTCONV_C2R_WSTR_C2CB504125C9: [u16; 6] = [0x0063u16, 0x0068, 0x0064, 0x0069, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_C77B70FA9459: [u16; 6] = [0x0064u16, 0x0065, 0x006C, 0x0061, 0x0079, 0x0000];
pub static RUSTCONV_C2R_WSTR_CADC4736A517: [u16; 6] = [0x006Du16, 0x006B, 0x0064, 0x0069, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_D0EB759CFFE4: [u16; 5] = [0x0064u16, 0x0069, 0x0072, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_D349EA348AF0: [u16; 6] = [0x0061u16, 0x0073, 0x0073, 0x006F, 0x0063, 0x0000];
pub static RUSTCONV_C2R_WSTR_D85A0CBFB039: [u16; 3] = [0x0063u16, 0x0064, 0x0000];
pub static RUSTCONV_C2R_WSTR_DAB33750E84A: [u16; 4] = [0x0076u16, 0x006F, 0x006C, 0x0000];
pub static RUSTCONV_C2R_WSTR_DEBCEBA069C0: [u16; 4] = [0x0072u16, 0x0065, 0x006D, 0x0000];
pub static RUSTCONV_C2R_WSTR_E88850D59C7F: [u16; 7] = [0x0076u16, 0x0065, 0x0072, 0x0069, 0x0066, 0x0079, 0x0000];
pub static RUSTCONV_C2R_WSTR_EAFD7A71F5E7: [u16; 6] = [0x0070u16, 0x0075, 0x0073, 0x0068, 0x0064, 0x0000];
pub static RUSTCONV_C2R_WSTR_F6B842C1451E: [u16; 6] = [0x0072u16, 0x006D, 0x0064, 0x0069, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_F9D71367F36B: [u16; 6] = [0x0074u16, 0x0069, 0x006D, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_C2R_WSTR_FA9FFEFCB3A3: [u16; 6] = [0x0074u16, 0x0069, 0x0074, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_FCABAF3CA758: [u16; 4] = [0x0073u16, 0x0065, 0x0074, 0x0000];

#[no_mangle]
pub static mut cmds: [COMMAND; 58] = unsafe {
    [
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_1B9E0FE7B740.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandShowCommands as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_A0E70D05E916.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandAlias as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_D349EA348AF0.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandAssoc as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_2A522F376CF2.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_beep as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_2BFE0A590FD6.as_ptr() as LPTSTR,
            flags: CMD_BATCHONLY,
            func: Some(cmd_call as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_D85A0CBFB039.as_ptr() as LPTSTR,
            flags: CMD_SPECIAL,
            func: Some(cmd_chdir as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_C2CB504125C9.as_ptr() as LPTSTR,
            flags: CMD_SPECIAL,
            func: Some(cmd_chdir as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_6A8D52266886.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandChoice as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_A7CFC14CDE1B.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_cls as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_5B9F893DE302.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandColor as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_377C04EA0393.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_copy as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_1C9DC775D96D.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_ctty as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_A698681B378D.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_date as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_40D6F3569433.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandDelete as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_08F79C40983C.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandDelete as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_C77B70FA9459.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandDelay as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_1F9BE24D4991.as_ptr() as LPTSTR,
            flags: CMD_SPECIAL,
            func: Some(CommandDir as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_D0EB759CFFE4.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandDirs as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_0E24EFDCDDBF.as_ptr() as LPTSTR,
            flags: CMD_SPECIAL,
            func: Some(CommandEcho as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_091778E8300A.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandEchos as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_1E55342C191E.as_ptr() as LPTSTR,
            flags: CMD_SPECIAL,
            func: Some(CommandEchoerr as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_496BFBF67486.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandEchoserr as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_4E2171725127.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_endlocal as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_4CDE8A491581.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandDelete as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_5C9738DE6AF0.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandExit as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_2F458E01A72D.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_for as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_96F19EF35C88.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandFree as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_1F509DCAEB1D.as_ptr() as LPTSTR,
            flags: CMD_BATCHONLY,
            func: Some(cmd_goto as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_4A44DE231043.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandHistory as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_B33584180CB3.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_if as unsafe extern "C" fn(*mut TCHAR) -> ::core::ffi::c_int),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_5B3605B65962.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandMemory as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_5C4147782415.as_ptr() as LPTSTR,
            flags: CMD_SPECIAL,
            func: Some(cmd_mkdir as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_CADC4736A517.as_ptr() as LPTSTR,
            flags: CMD_SPECIAL,
            func: Some(cmd_mkdir as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_923BEF676D76.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_mklink as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_8A0F6CB5912D.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_move as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_032B79795A8E.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_path as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_4F9E8005A358.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_pause as unsafe extern "C" fn(*mut TCHAR) -> ::core::ffi::c_int),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_9BE6D558E725.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandPopd as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_3F89D9B038EA.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_prompt as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_EAFD7A71F5E7.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandPushd as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_14ECF914DD1A.as_ptr() as LPTSTR,
            flags: CMD_SPECIAL,
            func: Some(cmd_rmdir as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_F6B842C1451E.as_ptr() as LPTSTR,
            flags: CMD_SPECIAL,
            func: Some(cmd_rmdir as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_DEBCEBA069C0.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandRem as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_6A216AD47A67.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_rename as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_42A469BB0711.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_rename as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_36701C1B5A9F.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandScreen as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_FCABAF3CA758.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_set as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_4270EF7B1AA0.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_setlocal as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_73BCE7399733.as_ptr() as LPTSTR,
            flags: CMD_BATCHONLY,
            func: Some(cmd_shift as unsafe extern "C" fn(*mut TCHAR) -> ::core::ffi::c_int),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_BEB8E1991CF9.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_start as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_9E442C20507D.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_time as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_F9D71367F36B.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(CommandTimer as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_FA9FFEFCB3A3.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_title as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_3B8BF4000A7A.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_type as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_BACDE87AB274.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_ver as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_E88850D59C7F.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_verify as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: RUSTCONV_C2R_WSTR_DAB33750E84A.as_ptr() as LPTSTR,
            flags: 0 as INT,
            func: Some(cmd_vol as unsafe extern "C" fn(LPTSTR) -> INT),
        },
        tagCOMMAND {
            name: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            flags: 0 as INT,
            func: None,
        },
    ]
};

// Assumes all INCLUDE_CMD_* and FEATURE_* conditionals are enabled.
// CMD_BATCHONLY and CMD_SPECIAL are referenced by name; their values come from the caller's headers.
#[no_mangle]
pub unsafe extern "C" fn PrintCommandList() {
    let mut cmdptr: LPCOMMAND = ::core::ptr::null_mut::<tagCOMMAND>();
    let mut y: INT = 0;

    y = 0 as INT;
    cmdptr = core::ptr::addr_of_mut!(cmds) as *mut COMMAND as LPCOMMAND;
    while !(*cmdptr).name.is_null() {
        if ((*cmdptr).flags as INT & CMD_HIDE as INT) == 0 {
            y += 1;
            if y == 8 as INT {
                ConOutPuts((*cmdptr).name);
                ConOutChar(_T('\n' as u16));
                y = 0 as INT;
            } else {
                ConOutPrintf(_T("%-10s" as *const u8 as *const u16), (*cmdptr).name);
            }
        }

        cmdptr = cmdptr.offset(1);
    }

    if y != 0 as INT {
        ConOutChar(_T('\n' as u16));
    }
}
